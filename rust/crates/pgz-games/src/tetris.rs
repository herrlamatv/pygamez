//! tetris.rs
//! =========
//! Tetris nach moderner Guideline - Solo (Marathon/Sprint/Ultra), Versus
//! gegen die KI und zu zweit. Uebertragen aus `games/tetris.py`; Regeln in
//! [`crate::tetris_core`], KI in [`crate::tetris_ai`].

use std::collections::{HashMap, HashSet};
use std::f64::consts::{PI, TAU};

use pgz_core::{highscore, replay, store};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::tetris_ai::{self as ai, Act};
use crate::tetris_core::{self as core, Board, Event, LockInfo, Queue, BUFFER, COLS, VISIBLE};

type Rgb = (i32, i32, i32);

fn color_of(k: char) -> Rgb {
    match k {
        'I' => (80, 210, 220),
        'O' => (240, 220, 90),
        'T' => (190, 110, 220),
        'S' => (110, 220, 120),
        'Z' => (235, 100, 100),
        'J' => (100, 130, 230),
        'L' => (240, 160, 80),
        'X' => (96, 100, 112),
        _ => (128, 134, 150),
    }
}
const COLOR_ORDER: [char; 7] = ['I', 'O', 'T', 'S', 'Z', 'J', 'L'];
const COL_P1: Rgb = (110, 220, 120);
const COL_P2: Rgb = (80, 210, 220);
const COL_B2B: Rgb = (255, 206, 92);
const COL_PC: Rgb = (255, 236, 150);

#[derive(Clone, Copy, PartialEq)]
enum St {
    Setup,
    Count,
    Play,
    Finish,
    Over,
}

const VARIANTS: [&str; 3] = ["marathon", "sprint", "ultra"];
const SPRINT_LINES: i64 = 40;
const ULTRA_TIME: f64 = 120.0;
const SPRINT_ACH_TIME: f64 = 120.0;
const COUNT_TIME: f64 = 1.25;
const GO_TIME: f64 = 0.7;
const OVER_LOCK: f64 = 0.5;
const VS_LEVEL_EVERY: f64 = 40.0;
const VS_LEVEL_MAX: i64 = 12;
const CLEAR_ANIM: f64 = 0.2;
const DAS_RANGE: (i64, i64, i64) = (50, 400, 10);
const ARR_RANGE: (i64, i64, i64) = (0, 200, 5);
const REC_KINDS: [char; 9] = [' ', 'I', 'J', 'L', 'O', 'S', 'T', 'Z', 'G'];
const REC_CHAPTER: usize = 300;

fn finish_time(reason: &str) -> f64 {
    match reason {
        "topout" => 1.5,
        "ko" => 2.0,
        _ => 1.4,
    }
}

fn rgba_of(c: impl Col, a: f64) -> Color {
    let c = c.c();
    rgba(c.r, c.g, c.b, (a as i32).clamp(0, 255))
}

fn ease_out(p: f64) -> f64 {
    let p = 0f64.max(1f64.min(p));
    1.0 - py::pow(1.0 - p, 3.0)
}

/// `fmt_time(sec, cents)` - "m:ss.cc" bzw. "m:ss".
pub fn fmt_time(sec: f64, cents: bool) -> String {
    let sec = 0f64.max(sec);
    let m = py::ffdiv(sec, 60.0) as i64;
    let s = sec - 60.0 * m as f64;
    if cents {
        return format!("{}:{:05.2}", m, (s * 100.0).floor() / 100.0);
    }
    format!("{}:{:02}", m, s as i64)
}

fn rec_idx(k: Option<char>) -> i64 {
    match k {
        None => 0,
        Some(c) => REC_KINDS.iter().position(|x| *x == c).unwrap_or(0) as i64,
    }
}

#[derive(Default)]
struct Pad {
    left: HashSet<String>,
    right: HashSet<String>,
    down: HashSet<String>,
    dir: i32,
    held_ms: f64,
    auto: i64,
}

impl Pad {
    fn press(&mut self, d: i32, key: &str) {
        if d < 0 {
            self.left.insert(key.into());
        } else {
            self.right.insert(key.into());
        }
        self.dir = d;
        self.held_ms = 0.0;
        self.auto = 0;
    }

    fn release(&mut self, key: &str) {
        self.down.remove(key);
        for d in [-1, 1] {
            let had = if d < 0 { self.left.remove(key) } else { self.right.remove(key) };
            if had {
                let empty = if d < 0 { self.left.is_empty() } else { self.right.is_empty() };
                if empty && self.dir == d {
                    let other = if d < 0 { !self.right.is_empty() } else { !self.left.is_empty() };
                    self.dir = if other { -d } else { 0 };
                    self.held_ms = 0.0;
                    self.auto = 0;
                }
            }
        }
    }
}

struct Fx {
    shake: f64,
    clear_t: f64,
    clear_rows: Vec<(i32, [Option<char>; 10])>,
    shift: HashMap<i32, i32>,
    flash_cells: Vec<(i32, i32)>,
    flash_t: f64,
    trail: Option<(Vec<(i32, (i32, i32))>, char)>,
    trail_t: f64,
    level_t: f64,
    rise_t: f64,
    rise_n: i64,
    cancel_t: f64,
    ko_t: f64,
}

impl Fx {
    fn new() -> Fx {
        Fx { shake: 0.0, clear_t: 9.0, clear_rows: Vec::new(), shift: HashMap::new(), flash_cells: Vec::new(), flash_t: 9.0, trail: None, trail_t: 9.0, level_t: 9.0, rise_t: 9.0, rise_n: 0, cancel_t: 9.0, ko_t: -1.0 }
    }
}

struct AiPlayer {
    level: usize,
    cfg: ai::Level,
    rng: py::Random,
    piece: i64,
    acts: Vec<Act>,
    timer: f64,
    replanned: bool,
}

impl AiPlayer {
    fn new(level: i64, seed: u64) -> AiPlayer {
        let level = 0.max(2.min(level)) as usize;
        AiPlayer { level, cfg: ai::LEVELS[level], rng: py::Random::new(seed), piece: -1, acts: Vec::new(), timer: 0.0, replanned: false }
    }

    fn update(&mut self, dt: f64, board: &mut Board) {
        if !board.active {
            return;
        }
        if board.piece_id != self.piece {
            self.piece = board.piece_id;
            self.acts = ai::plan(board, self.level, Some(&mut self.rng), true);
            self.timer = self.cfg.think * self.rng.uniform(0.7, 1.3);
            self.replanned = false;
        }
        self.timer -= dt;
        let mut guard = 0;
        while self.timer <= 0.0 && !self.acts.is_empty() && board.active && guard < 16 {
            guard += 1;
            let act = self.acts.remove(0);
            let ok = ai::apply(board, act);
            if act == Act::Drop {
                self.acts.clear();
                break;
            }
            if act == Act::Hold {
                self.piece = board.piece_id;
            } else if !ok && !self.replanned {
                self.acts = ai::plan(board, self.level, Some(&mut self.rng), false);
                self.replanned = true;
            }
            self.timer += self.cfg.step * self.rng.uniform(0.8, 1.2);
        }
    }
}

#[derive(Clone, Copy)]
struct Side {
    field: Rect,
    cell: i32,
    hold: Rect,
    next: Rect,
    stats: Option<Rect>,
    bar: Option<Rect>,
    mini: i32,
    mini2: i32,
    pad: i32,
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    maxl: f64,
    col: Rgb,
    size: i32,
}

struct Callout {
    side: usize,
    surfs: Vec<Surface>,
    t: f64,
    dur: f64,
}

struct Missile {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    t: f64,
    dur: f64,
    n: i64,
    color: Rgb,
}

#[derive(Clone)]
struct Stats {
    lines: i64,
    score: i64,
    level: i64,
    pps: f64,
    attack: i64,
    tetrises: i64,
    tspins: i64,
}

struct ResultInfo {
    reason: &'static str,
    winner: Option<usize>,
    time: f64,
    stats: Vec<Stats>,
    new_best: bool,
    prev_best_time: Option<f64>,
    prev_best_score: Option<i64>,
}

struct Best {
    sprint: Option<f64>,
    sprint_pps: f64,
    ultra: i64,
    ultra_lines: i64,
    vs: [[i64; 2]; 3],
}

#[derive(Default)]
struct SetupRects {
    choice: Vec<Rect>,
    keys: Option<Rect>,
    level: Option<Rect>,
    ghost: Option<Rect>,
    das: Option<Rect>,
    arr: Option<Rect>,
    start: Rect,
}

/// `TetrisGame`
pub struct TetrisGame {
    b: GameBase,
    versus: bool,
    variant: &'static str,
    start_level: i64,
    ai_level: i64,
    das: i64,
    arr: i64,
    ghost: bool,
    huge: Font,
    big: Font,
    small: Font,
    small_b: Font,
    tiny: Font,
    f_label: Font,
    f_value: Font,
    f_hero: Font,
    f_call: Font,
    f_call_s: Font,
    anim_t: f64,
    wins: [i64; 2],
    best: Best,
    particles: Vec<Particle>,
    callouts: Vec<Callout>,
    missiles: Vec<Missile>,
    sprites: HashMap<(char, i32, u8, bool), Surface>,
    pieces: HashMap<(char, i32, i32, bool), Surface>,
    caches: HashMap<String, Surface>,
    stack_cache: HashMap<usize, ((i64, i32, i32, bool), Surface)>,
    boards: Vec<Board>,
    pads: Vec<Pad>,
    fx: Vec<Fx>,
    ai: Option<AiPlayer>,
    result: Option<ResultInfo>,
    elapsed: f64,
    go_t: f64,
    count_t: f64,
    finish_t: f64,
    finish_dur: f64,
    over_at: f64,
    prev_four: bool,
    setup_focus: i64,
    over_rects: Vec<(&'static str, Rect)>,
    rec: Option<replay::Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_at: Option<(usize, usize)>,
    rec_delta: replay::Delta,
    rec_n: usize,
    state: St,
    deco: Vec<(char, f64, f64, f64, i64)>,
    sides: Vec<Side>,
    setup_rects: SetupRects,
    setup_title_y: i32,
    setup_sub_y: i32,
    setup_desc_y: i32,
    setup_help_y: i32,
    setup_footer_y: (i32, i32),
}

impl TetrisGame {
    /// `TetrisGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = TetrisGame {
            b,
            versus: false,
            variant: "marathon",
            start_level: 1,
            ai_level: 1,
            das: 170,
            arr: 50,
            ghost: true,
            huge: f.clone(),
            big: f.clone(),
            small: f.clone(),
            small_b: f.clone(),
            tiny: f.clone(),
            f_label: f.clone(),
            f_value: f.clone(),
            f_hero: f.clone(),
            f_call: f.clone(),
            f_call_s: f,
            anim_t: 0.0,
            wins: [0, 0],
            best: Best { sprint: None, sprint_pps: 0.0, ultra: 0, ultra_lines: 0, vs: [[0, 0]; 3] },
            particles: Vec::new(),
            callouts: Vec::new(),
            missiles: Vec::new(),
            sprites: HashMap::new(),
            pieces: HashMap::new(),
            caches: HashMap::new(),
            stack_cache: HashMap::new(),
            boards: Vec::new(),
            pads: Vec::new(),
            fx: Vec::new(),
            ai: None,
            result: None,
            elapsed: 0.0,
            go_t: 0.0,
            count_t: 0.0,
            finish_t: 0.0,
            finish_dur: 0.0,
            over_at: 0.0,
            prev_four: false,
            setup_focus: 0,
            over_rects: Vec::new(),
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_at: None,
            rec_delta: replay::Delta::new(),
            rec_n: 0,
            state: St::Setup,
            deco: Vec::new(),
            sides: Vec::new(),
            setup_rects: SetupRects::default(),
            setup_title_y: 0,
            setup_sub_y: 0,
            setup_desc_y: 0,
            setup_help_y: 0,
            setup_footer_y: (0, 0),
        };
        g.reset();
        Box::new(g)
    }

    fn now() -> f64 {
        py::perf_counter()
    }

    fn load_options(&mut self) {
        let num = |me: &Self, key: &str, def: i64, lo: i64, hi: i64| -> i64 {
            let v = match me.b.opt_value("tetris", key) {
                Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64().unwrap_or(def),
                _ => def,
            };
            lo.max(hi.min(v))
        };
        let solo = self.b.opt_value("tetris", "solo").and_then(|v| v.as_str().map(|s| s.to_string()));
        self.variant = VARIANTS.iter().copied().find(|v| Some(v.to_string()) == solo).unwrap_or("marathon");
        self.start_level = num(self, "start_level", 1, 1, 15);
        self.ai_level = num(self, "ai_level", 1, 0, 2);
        self.das = num(self, "das", 170, DAS_RANGE.0, DAS_RANGE.1);
        self.arr = num(self, "arr", 50, ARR_RANGE.0, ARR_RANGE.1);
        self.ghost = match self.b.opt_value("tetris", "ghost") {
            None => true,
            Some(Value::Bool(b)) => b,
            Some(_) => true,
        };
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("tetris", key, value);
    }

    fn load_best() -> Best {
        let data = store::load_section("tetris");
        let mut out = Best { sprint: None, sprint_pps: 0.0, ultra: 0, ultra_lines: 0, vs: [[0, 0]; 3] };
        if let Some(Value::Number(n)) = data.get("sprint") {
            if let Some(f) = n.as_f64() {
                if f > 0.0 {
                    out.sprint = Some(f);
                }
            }
        }
        if let Some(Value::Number(n)) = data.get("sprint_pps") {
            out.sprint_pps = n.as_f64().unwrap_or(0.0);
        }
        for (key, slot) in [("ultra", 0), ("ultra_lines", 1)] {
            if let Some(Value::Number(n)) = data.get(key) {
                if let Some(v) = n.as_i64().filter(|v| *v > 0 && (n.is_i64() || n.is_u64())) {
                    if slot == 0 {
                        out.ultra = v;
                    } else {
                        out.ultra_lines = v;
                    }
                }
            }
        }
        if let Some(Value::Array(vs)) = data.get("vs") {
            if vs.len() == 3 {
                for (i, pair) in vs.iter().enumerate() {
                    if let Some(p) = pair.as_array() {
                        let ints: Vec<i64> = p.iter().filter_map(|v| if v.is_i64() || v.is_u64() { v.as_i64() } else { None }).filter(|v| *v >= 0).collect();
                        if p.len() == 2 && ints.len() == 2 {
                            out.vs[i] = [ints[0], ints[1]];
                        }
                    }
                }
            }
        }
        out
    }

    fn save_best(&self) {
        let mut m = Map::new();
        m.insert("ultra".into(), json!(self.best.ultra));
        m.insert("ultra_lines".into(), json!(self.best.ultra_lines));
        m.insert("vs".into(), json!(self.best.vs));
        if let Some(sp) = self.best.sprint {
            m.insert("sprint".into(), json!(py::round_to(sp, 3)));
            m.insert("sprint_pps".into(), json!(py::round_to(self.best.sprint_pps, 2)));
        }
        store::save_section("tetris", m);
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.huge = ui::font(26.max(64.min(h / 11)), true, false);
        self.big = ui::font(20.max(44.min(h / 16)), true, false);
        self.small = ui::font(13.max(22.min(h / 30)), false, false);
        self.small_b = ui::font(13.max(22.min(h / 30)), true, false);
        self.tiny = ui::font(11.max(18.min(h / 38)), false, false);
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.sides.clear();
        if !self.versus {
            let m = 8.max(h / 28);
            let c = 8.max(((h - 2 * m) / VISIBLE).min((w - 32) / 22));
            let (fw, fh) = (COLS * c, VISIBLE * c);
            let (fx, fy) = (py::fdiv32(w - fw, 2), py::fdiv32(h - fh, 2));
            let gap = 8.max(c / 2);
            let side_w = (3 * c).max(((6.8 * c as f64) as i32).min(fx - gap - 10));
            let (lx, rx) = (fx - gap - side_w, fx + fw + gap);
            let hold = Rect::new(lx, fy, side_w, (4.0 * c as f64) as i32);
            let stats = Rect::new(lx, hold.bottom() + gap, side_w, fy + fh - hold.bottom() - gap);
            let nxt = Rect::new(rx, fy, side_w, (14.2 * c as f64) as i32);
            self.sides.push(Side { field: Rect::new(fx, fy, fw, fh), cell: c, hold, next: nxt, stats: Some(stats), bar: None, mini: 5.max((c as f64 * 0.78) as i32), mini2: 4.max((c as f64 * 0.62) as i32), pad: 6.max(c / 3) });
        } else {
            let top = 24.max(h / 14);
            let bottom = 20.max(h / 20);
            let c = 6.max(py::fdiv32(h - top - bottom - 6, VISIBLE).min(((w - 16) as f64 / 35.6) as i32));
            let (fw, fh) = (COLS * c, VISIBLE * c);
            let side = 16.max((2.95 * c as f64) as i32);
            let barw = 4.max((0.42 * c as f64) as i32);
            let gap = 3.max(c / 4);
            let side_w = side + gap + barw + fw + gap + side;
            let center = 10.max(c);
            let x0 = py::fdiv32(w - (2 * side_w + center), 2);
            let fy = top + py::fdiv32(h - top - bottom - fh, 2);
            let mini = 4.max((c as f64 * 0.6) as i32);
            for i in 0..2 {
                let sx = x0 + i * (side_w + center);
                let hold = Rect::new(sx, fy, side, (3.3 * c as f64) as i32);
                let bar = Rect::new(sx + side + gap, fy, barw, fh);
                let field = Rect::new(bar.right(), fy, fw, fh);
                let nxt = Rect::new(field.right() + gap, fy, side, (12.4 * c as f64) as i32);
                self.sides.push(Side { field, cell: c, hold, next: nxt, stats: None, bar: Some(bar), mini, mini2: 4.max((c as f64 * 0.5) as i32), pad: 4.max(c / 4) });
            }
        }
        self.hud_fonts();
    }

    fn hud_fonts(&mut self) {
        let c = self.sides.first().map(|s| s.cell).unwrap_or(20) as f64;
        let compact = self.versus;
        self.f_label = ui::font(10.max(17.min((c * if compact { 0.56 } else { 0.62 }) as i32)), true, false);
        self.f_value = ui::font(12.max(30.min((c * if compact { 0.8 } else { 0.95 }) as i32)), true, false);
        self.f_hero = ui::font(16.max(46.min((c * 1.35) as i32)), true, false);
        self.f_call = ui::font(12.max(34.min((c * if compact { 0.78 } else { 0.92 }) as i32)), true, false);
        self.f_call_s = ui::font(10.max(24.min((c * 0.62) as i32)), true, false);
    }

    // ===================================================== Setup

    fn setup_items(&self) -> Vec<&'static str> {
        let mut items = Vec::new();
        if self.b.mode == "solo" {
            items.extend(["variant", "level"]);
        } else if self.b.mode == "versus_ai" {
            items.push("ai");
        }
        items.extend(["ghost", "das", "arr", "start"]);
        items
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = (500.max((w as f64 * 0.64) as i32)).min(w - 40);
        let tiny_h = self.tiny.get_height();
        let lab_h = tiny_h + 3;
        let btn_h = 28.max(48.min(h / 13));
        let line_h = tiny_h + 5;
        self.setup_title_y = (h as f64 * 0.095) as i32;
        self.setup_sub_y = (h as f64 * 0.165) as i32;
        let top = (h as f64 * 0.215) as i32;
        let bottom = h - (2 * tiny_h + 20);
        let mut blocks: Vec<(&str, i32)> = Vec::new();
        if self.b.mode == "solo" || self.b.mode == "versus_ai" {
            blocks.push(("choice", lab_h + btn_h + 2 * line_h + 2));
        } else {
            blocks.push(("keys", 2 * line_h + 18));
        }
        blocks.push(("opts", lab_h + btn_h + line_h + 2));
        blocks.push(("start", btn_h + 4));
        let total: i32 = blocks.iter().map(|b| b.1).sum();
        let sp = 4.max(py::fdiv32(bottom - top - total, blocks.len() as i32 + 1));
        let mut y = top + sp;
        let gap = 8;
        self.setup_rects = SetupRects::default();
        for (kind, bh) in blocks {
            match kind {
                "choice" => {
                    let n = 3;
                    let cw = (bw - gap * (n - 1)) / n;
                    self.setup_rects.choice = (0..n).map(|i| Rect::new(cx - bw / 2 + i * (cw + gap), y + lab_h, cw, btn_h)).collect();
                    self.setup_desc_y = y + lab_h + btn_h + 3;
                }
                "keys" => self.setup_rects.keys = Some(Rect::new(cx - bw / 2, y, bw, bh)),
                "opts" => {
                    let mut names = if self.b.mode == "solo" { vec!["level"] } else { Vec::new() };
                    names.extend(["ghost", "das", "arr"]);
                    let n = names.len() as i32;
                    let cw = (bw - gap * (n - 1)) / n;
                    for (i, name) in names.iter().enumerate() {
                        let r = Some(Rect::new(cx - bw / 2 + i as i32 * (cw + gap), y + lab_h, cw, btn_h));
                        match *name {
                            "level" => self.setup_rects.level = r,
                            "ghost" => self.setup_rects.ghost = r,
                            "das" => self.setup_rects.das = r,
                            _ => self.setup_rects.arr = r,
                        }
                    }
                    self.setup_help_y = y + lab_h + btn_h + 3;
                }
                _ => {
                    let sw = bw.min(180.max(w / 3));
                    self.setup_rects.start = Rect::new(cx - sw / 2, y, sw, btn_h);
                }
            }
            y += bh + sp;
        }
        self.setup_footer_y = (h - 2 * tiny_h - 12, h - tiny_h - 6);
    }

    fn setup_change(&mut self, item: &str, d: i64) {
        match item {
            "variant" => {
                let i = py::pmod(VARIANTS.iter().position(|v| *v == self.variant).unwrap_or(0) as i64 + d, 3);
                self.variant = VARIANTS[i as usize];
                let v = self.variant;
                self.save_setting("solo", json!(v));
            }
            "ai" => {
                self.ai_level = py::pmod(self.ai_level + d, 3);
                let v = self.ai_level;
                self.save_setting("ai_level", json!(v));
            }
            "level" => {
                if self.variant != "marathon" {
                    return;
                }
                self.start_level = 1.max(15.min(self.start_level + d));
                let v = self.start_level;
                self.save_setting("start_level", json!(v));
            }
            "ghost" => {
                self.ghost = !self.ghost;
                let v = self.ghost;
                self.save_setting("ghost", json!(v));
            }
            "das" => {
                self.das = DAS_RANGE.0.max(DAS_RANGE.1.min(self.das + d * DAS_RANGE.2));
                let v = self.das;
                self.save_setting("das", json!(v));
            }
            "arr" => {
                self.arr = ARR_RANGE.0.max(ARR_RANGE.1.min(self.arr + d * ARR_RANGE.2));
                let v = self.arr;
                self.save_setting("arr", json!(v));
            }
            _ => return,
        }
        self.b.play_sound("move");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let items = self.setup_items();
        let n = items.len() as i64;
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                self.setup_focus = 0.max((n - 1).min(self.setup_focus));
                let item = items[self.setup_focus as usize];
                if ["Up", "w", "W"].contains(&k) {
                    self.setup_focus = py::pmod(self.setup_focus - 1, n);
                    self.b.play_sound("click");
                } else if ["Down", "s", "S", "Tab"].contains(&k) {
                    self.setup_focus = py::pmod(self.setup_focus + 1, n);
                    self.b.play_sound("click");
                } else if ["Left", "a", "A", "minus", "KP_Subtract"].contains(&k) {
                    self.setup_change(item, -1);
                } else if ["Right", "d", "D", "plus", "KP_Add"].contains(&k) {
                    self.setup_change(item, 1);
                } else if ["1", "2", "3"].contains(&k) && (self.b.mode == "solo" || self.b.mode == "versus_ai") {
                    let i = k.parse::<usize>().unwrap_or(1) - 1;
                    if self.b.mode == "solo" {
                        self.variant = VARIANTS[i];
                        let v = self.variant;
                        self.save_setting("solo", json!(v));
                    } else {
                        self.ai_level = i as i64;
                        let v = self.ai_level;
                        self.save_setting("ai_level", json!(v));
                    }
                    self.b.play_sound("click");
                } else if k == "g" || k == "G" {
                    self.setup_change("ghost", 1);
                } else if ["Return", "KP_Enter", "space"].contains(&k) && !ev.repeat {
                    if item != "start" && k == "space" && item == "ghost" {
                        self.setup_change(item, 1);
                    } else {
                        self.start();
                    }
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                let Some((item, value)) = self.setup_hit(ev.pos) else {
                    return;
                };
                if let Some(i) = items.iter().position(|x| *x == item) {
                    self.setup_focus = i as i64;
                }
                if item == "start" {
                    self.start();
                } else if item == "variant" || item == "ai" {
                    if item == "variant" {
                        self.variant = VARIANTS[value as usize];
                        let v = self.variant;
                        self.save_setting("solo", json!(v));
                    } else {
                        self.ai_level = value;
                        let v = self.ai_level;
                        self.save_setting("ai_level", json!(v));
                    }
                    self.b.play_sound("click");
                } else {
                    self.setup_change(item, value);
                }
            }
            _ => {}
        }
    }

    fn setup_hit(&self, pos: (i32, i32)) -> Option<(&'static str, i64)> {
        let r = &self.setup_rects;
        for (i, c) in r.choice.iter().enumerate() {
            if c.hit(pos) {
                return Some((if self.b.mode == "solo" { "variant" } else { "ai" }, i as i64));
            }
        }
        for (name, rr) in [("level", r.level), ("das", r.das), ("arr", r.arr)] {
            if let Some(rr) = rr {
                if rr.hit(pos) {
                    return Some((name, if pos.0 < rr.centerx() { -1 } else { 1 }));
                }
            }
        }
        if r.ghost.is_some_and(|g| g.hit(pos)) {
            return Some(("ghost", 1));
        }
        if r.start.hit(pos) {
            return Some(("start", 0));
        }
        None
    }

    // ===================================================== Partie

    fn start(&mut self) {
        let seed = py::with(|r| r.getrandbits(32));
        let s32 = seed as u32;
        if self.versus {
            self.boards = vec![Board::new(s32, 1, true), Board::new(s32, 1, true)];
        } else if self.variant == "marathon" {
            self.boards = vec![Board::new(s32, self.start_level, false)];
        } else {
            self.boards = vec![Board::new(s32, 1, true)];
        }
        self.pads = self.boards.iter().map(|_| Pad::default()).collect();
        self.fx = self.boards.iter().map(|_| Fx::new()).collect();
        self.ai = if self.b.mode == "versus_ai" { Some(AiPlayer::new(self.ai_level, seed)) } else { None };
        self.particles.clear();
        self.callouts.clear();
        self.missiles.clear();
        self.caches.clear();
        self.stack_cache.clear();
        self.elapsed = 0.0;
        self.count_t = COUNT_TIME;
        self.go_t = 0.0;
        self.result = None;
        self.b.score = 0;
        self.b.game_over = false;
        self.prev_four = false;
        self.b.new_round_result();
        self.state = St::Count;
        self.rec_new();
        self.layout();
        self.b.play_sound("select");
    }

    fn human_slots(&self) -> Vec<(usize, Option<&'static str>, usize)> {
        if self.b.multiplayer {
            return vec![(0, Some("p1"), 1), (1, Some("p2"), 2)];
        }
        vec![(0, None, 0)]
    }

    fn extra_keys(set: usize, what: &str) -> &'static [&'static str] {
        match (set, what) {
            (0, "hold") => &["c", "C", "Shift_L", "Shift_R"],
            (0, "ccw") => &["z", "Z", "y", "Y", "Control_R"],
            (0, "cw") => &["x", "X"],
            (1, "hold") => &["q", "Q"],
            (1, "ccw") => &["e", "E"],
            (2, "hold") => &["Shift_R"],
            (2, "ccw") => &["Control_R"],
            _ => &[],
        }
    }

    fn play_key(&mut self, slot: usize, player: Option<&str>, extra: usize, key: &str, live: bool) {
        let act = |a: &str| self.b.is_action(key, a, player);
        if act("left") || act("right") {
            let d = if act("left") { -1 } else { 1 };
            self.pads[slot].press(d, key);
            if live && self.boards[slot].mv(d) {
                self.b.play_sound("move");
            }
        } else if act("down") {
            self.pads[slot].down.insert(key.into());
            if live {
                self.boards[slot].soft_step();
            }
        } else if act("up") {
            if live {
                self.rotate(slot, 1);
            }
        } else if act("action") {
            if live {
                self.hard_drop(slot);
            }
        } else if Self::extra_keys(extra, "hold").contains(&key) && self.b.key_is_free(key) {
            if live && self.boards[slot].hold() {
                self.b.play_sound("select");
            }
        } else if Self::extra_keys(extra, "ccw").contains(&key) && self.b.key_is_free(key) {
            if live {
                self.rotate(slot, -1);
            }
        } else if Self::extra_keys(extra, "cw").contains(&key) && self.b.key_is_free(key) && live {
            self.rotate(slot, 1);
        }
    }

    fn rotate(&mut self, slot: usize, d: i32) {
        if self.boards[slot].rotate(d) {
            self.b.play_sound("rotate");
        }
    }

    fn hard_drop(&mut self, slot: usize) {
        let b = &self.boards[slot];
        if !b.active {
            return;
        }
        let gy = b.ghost_y();
        if gy > b.y {
            let mut cols: Vec<(i32, (i32, i32))> = Vec::new();
            for (x, y) in b.cells_at(None) {
                let (top, _bot) = cols.iter().find(|c| c.0 == x).map(|c| c.1).unwrap_or((y, y));
                let v = (top.min(y), y + gy - b.y);
                match cols.iter_mut().find(|c| c.0 == x) {
                    Some(e) => e.1 = v,
                    None => cols.push((x, v)),
                }
            }
            let kind = b.kind;
            self.fx[slot].trail = Some((cols, kind));
            self.fx[slot].trail_t = 0.0;
        }
        self.fx[slot].shake = self.fx[slot].shake.max(0.12);
        self.boards[slot].hard_drop();
    }

    fn handle_over(&mut self, ev: &InputEvent) {
        if Self::now() - self.over_at < OVER_LOCK {
            return;
        }
        if ev.kind == Kind::KeyDown && !ev.repeat {
            let k = ev.key.as_str();
            if k == "r" || k == "R" {
                self.start();
            } else if (k == "p" || k == "P") && self.replay.is_some() {
                self.open_replay();
            } else if k == "s" || k == "S" {
                self.to_setup();
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            if let Some(&(name, _)) = self.over_rects.iter().find(|(_, r)| r.hit(ev.pos)) {
                if name == "again" {
                    self.start();
                } else {
                    self.to_setup();
                }
            }
        }
    }

    fn to_setup(&mut self) {
        self.b.game_over = false;
        self.state = St::Setup;
        self.b.score = 0;
        self.boards.clear();
        self.particles.clear();
        self.callouts.clear();
        self.missiles.clear();
        self.layout();
        self.build_setup_layout();
        self.b.play_sound("click");
    }

    // ===================================================== Logik

    fn das_step(&mut self, slot: usize, dt: f64) {
        let pad = &mut self.pads[slot];
        if pad.dir == 0 {
            return;
        }
        pad.held_ms += dt * 1000.0;
        if pad.held_ms < self.das as f64 || !self.boards[slot].active {
            return;
        }
        let mut moved = false;
        let dir = pad.dir;
        if self.arr <= 0 {
            while self.boards[slot].mv(dir) {
                moved = true;
            }
        } else {
            let target = 1 + py::ffdiv(pad.held_ms - self.das as f64, self.arr as f64) as i64;
            let n = (target - pad.auto).min(COLS as i64);
            pad.auto = target;
            for _ in 0..n {
                if !self.boards[slot].mv(dir) {
                    break;
                }
                moved = true;
            }
        }
        if moved {
            self.b.play_sound("move");
        }
    }

    fn process_events(&mut self, i: usize) {
        let events = std::mem::take(&mut self.boards[i].events);
        let human = self.is_human(i);
        for ev in events {
            match ev {
                Event::Lock(info) => self.on_lock(i, &info),
                Event::Level(n) => {
                    self.fx[i].level_t = 0.0;
                    self.callout(i, vec![(tf("tetris.call.level", &[("n", &n.to_string())]), ui::pal().gold)], vec![]);
                    self.b.play_sound("level");
                }
                Event::Garbage(n) => {
                    self.fx[i].rise_t = 0.0;
                    self.fx[i].rise_n = n;
                    self.fx[i].shake = self.fx[i].shake.max(0.2);
                    if human {
                        self.b.play_sound("hit");
                        self.b.rumble(140);
                    }
                }
                _ => {}
            }
        }
    }

    fn is_human(&self, i: usize) -> bool {
        i == 0 || self.b.multiplayer
    }

    fn on_lock(&mut self, i: usize, info: &LockInfo) {
        let human = self.is_human(i);
        let side = self.sides[i];
        let c = side.cell;
        let field = side.field;
        self.fx[i].flash_cells = info.cells.clone();
        self.fx[i].flash_t = 0.0;
        let lines = info.lines;
        if human || lines != 0 {
            self.b.play_sound(if lines == 0 { "lock" } else { "line" });
        }
        if lines != 0 {
            let f = &mut self.fx[i];
            f.clear_t = 0.0;
            f.clear_rows = info.rows.iter().enumerate().map(|(k, r)| (*r, info.cleared[k])).collect();
            let full: HashSet<i32> = info.rows.iter().copied().collect();
            let kept: Vec<i32> = (0..core::ROWS).filter(|o| !full.contains(o)).collect();
            f.shift.clear();
            for (j, o) in kept.iter().enumerate() {
                let k = lines as i32 + j as i32 - o;
                if k > 0 && lines as i32 + j as i32 >= BUFFER {
                    f.shift.insert(lines as i32 + j as i32, k);
                }
            }
            let rows = f.clear_rows.clone();
            for (r, row) in rows {
                if r < BUFFER {
                    continue;
                }
                let y = (field.y + (r - BUFFER) * c + c / 2) as f64;
                for (x, kind) in row.iter().enumerate() {
                    let col = color_of(kind.unwrap_or('G'));
                    for _ in 0..(if human { 2 } else { 1 }) {
                        self.particle((field.x + x as i32 * c + c / 2) as f64, y, col, c, 1.0);
                    }
                }
            }
            if lines == 4 || info.spin.is_some() {
                self.fx[i].shake = self.fx[i].shake.max(0.18);
                if human {
                    self.b.rumble(160);
                }
            }
        }
        let p = ui::pal();
        let mut rows: Vec<(String, Color)> = Vec::new();
        let spin = info.spin;
        if let Some(sp) = spin {
            let mut word = t(if sp == "mini" { "tetris.call.tspin_mini" } else { "tetris.call.tspin" });
            if lines != 0 {
                word = tf("tetris.call.spin_lines", &[("spin", &word), ("lines", &t(&format!("tetris.call.lines{lines}")))]);
            }
            rows.push((word, color_of('T').c()));
        } else if lines == 4 {
            rows.push((t("tetris.call.tetris"), color_of('I').c()));
        }
        if info.b2b {
            rows.push((t("tetris.call.b2b"), COL_B2B.c()));
        }
        if info.combo >= 1 {
            rows.push((tf("tetris.call.combo", &[("n", &info.combo.to_string())]), ui::mix(color_of('L'), (255, 255, 255), 0.15)));
        }
        if info.pc {
            rows.push((t("tetris.call.pc"), COL_PC.c()));
        }
        if !rows.is_empty() {
            let mut small = Vec::new();
            if !self.versus && info.points != 0 && self.variant != "sprint" {
                small.push(format!("+{}", info.points));
            } else if self.versus && info.sent != 0 {
                small.push(format!("+{}", info.sent));
            }
            self.callout(i, rows, small);
        }
        if let Some(sp) = spin {
            if human {
                audio::tone(if sp == "full" { 880.0 } else { 660.0 }, 0.12, Some(&self.b.settings), "square", 0.3);
            }
        }
        if info.pc {
            self.b.play_sound("win");
            for _ in 0..40 {
                let x = field.centerx() as f64 + py::uniform(-field.w as f64 / 2.0, field.w as f64 / 2.0);
                let y = field.bottom() as f64 - py::uniform(0.0, field.h as f64 * 0.3);
                let col = py::choice(&[COL_PC, color_of('I'), color_of('T')]);
                self.particle(x, y, col, c, 1.0);
            }
        } else if lines == 4 && human {
            self.b.play_sound("powerup");
        }
        if human && !self.b.multiplayer {
            if lines == 4 {
                self.b.ach_event("tetris_four", None);
                if self.prev_four {
                    self.b.ach_event("tetris_b2b", None);
                }
            }
            if lines != 0 {
                self.prev_four = lines == 4;
            }
            if spin == Some("full") && lines == 2 {
                self.b.ach_event("tetris_tspin", None);
            }
            if info.pc {
                self.b.ach_event("tetris_pc", None);
            }
        }
        if self.versus {
            if info.cancelled != 0 {
                self.fx[i].cancel_t = 0.0;
            }
            if info.sent != 0 {
                let other = 1 - i;
                self.boards[other].receive(info.sent);
                let src = self.sides[i].field;
                let dst = self.sides[other].bar.unwrap_or(src);
                self.missiles.push(Missile {
                    x0: src.centerx() as f64,
                    y0: src.y as f64 + src.h as f64 * 0.4,
                    x1: dst.centerx() as f64,
                    y1: (dst.bottom() - c) as f64,
                    t: 0.0,
                    dur: 0.38,
                    n: info.sent,
                    color: if i == 0 { COL_P1 } else { COL_P2 },
                });
            }
        }
        let _ = p;
    }

    fn check_end(&mut self) {
        if !self.versus {
            let b = &self.boards[0];
            if b.dead {
                self.finish("topout", None);
            } else if self.variant == "sprint" && b.lines >= SPRINT_LINES {
                self.finish("sprint", None);
            } else if self.variant == "ultra" && self.elapsed >= ULTRA_TIME {
                self.elapsed = ULTRA_TIME;
                self.finish("ultra", None);
            }
            return;
        }
        let dead: Vec<usize> = (0..self.boards.len()).filter(|i| self.boards[*i].dead).collect();
        if !dead.is_empty() {
            let winner = if dead.len() == 2 { None } else { Some(1 - dead[0]) };
            self.finish("ko", winner);
        }
    }

    fn stats_of(&self, i: usize) -> Stats {
        let b = &self.boards[i];
        let el = 0.001f64.max(self.elapsed);
        Stats { lines: b.lines, score: b.score, level: b.level, pps: b.pieces as f64 / el, attack: b.attack_sent, tetrises: b.tetrises, tspins: b.tspins }
    }

    fn finish(&mut self, reason: &'static str, winner: Option<usize>) {
        self.state = St::Finish;
        self.finish_t = 0.0;
        self.finish_dur = finish_time(reason);
        for b in self.boards.iter_mut() {
            b.active = false;
        }
        let stats: Vec<Stats> = (0..self.boards.len()).map(|i| self.stats_of(i)).collect();
        let mut res = ResultInfo { reason, winner, time: self.elapsed, stats, new_best: false, prev_best_time: None, prev_best_score: None };
        let (b0score, b0lines) = (self.boards[0].score, self.boards[0].lines);
        if !self.versus {
            if reason == "topout" {
                self.fx[0].ko_t = 0.0;
                self.b.play_sound("gameover");
                self.b.rumble(220);
                if self.variant == "marathon" {
                    self.b.score = b0score;
                }
            } else if reason == "sprint" {
                res.prev_best_time = self.best.sprint;
                if self.best.sprint.is_none_or(|b| self.elapsed < b) {
                    self.best.sprint = Some(self.elapsed);
                    self.best.sprint_pps = res.stats[0].pps;
                    res.new_best = true;
                    self.save_best();
                }
                if self.elapsed < SPRINT_ACH_TIME {
                    self.b.ach_event("tetris_sprint", None);
                }
                self.b.play_sound("win");
                self.confetti(0);
            } else {
                res.prev_best_score = if self.best.ultra != 0 { Some(self.best.ultra) } else { None };
                if b0score > self.best.ultra {
                    self.best.ultra = b0score;
                    self.best.ultra_lines = b0lines;
                    res.new_best = true;
                    self.save_best();
                }
                self.b.play_sound("win");
                self.confetti(0);
            }
        } else {
            if let Some(w) = winner {
                self.wins[w] += 1;
            }
            for i in 0..self.boards.len() {
                if self.boards[i].dead {
                    self.fx[i].ko_t = 0.0;
                    let field = self.sides[i].field;
                    let c = self.sides[i].cell;
                    for _ in 0..50 {
                        let x = py::uniform(field.x as f64, field.right() as f64);
                        let y = py::uniform(field.centery() as f64, field.bottom() as f64);
                        let col = color_of(py::choice(&COLOR_ORDER));
                        self.particle(x, y, col, c, 1.8);
                    }
                }
            }
            self.b.play_sound("explode");
            self.b.rumble(260);
            if self.b.mode == "versus_ai" && winner.is_some() {
                let w = winner.unwrap_or(0);
                self.best.vs[self.ai_level as usize][if w == 0 { 0 } else { 1 }] += 1;
                self.save_best();
                self.b.report_result(w == 0);
            } else if self.b.multiplayer && winner.is_some() {
                self.b.report_duel(winner.unwrap_or(0));
            }
            if let Some(w) = winner {
                self.confetti(w);
            }
        }
        self.result = Some(res);
        self.rec_finish();
    }

    // ===================================================== Replay-Aufnahme

    fn rec_new(&mut self) {
        self.replay = None;
        self.rec_n = 0;
        self.rec_delta.reset();
        let mut meta = Map::new();
        meta.insert("mode".into(), json!(self.b.mode));
        meta.insert("variant".into(), json!(self.variant));
        meta.insert("level".into(), json!(self.start_level));
        meta.insert("ai".into(), json!(self.ai_level));
        meta.insert("ghost".into(), json!(self.ghost));
        meta.insert("boards".into(), json!(self.boards.len()));
        self.rec = replay::recorder("tetris", shell::replay_enabled(&self.b.settings), meta);
        self.rec_chapter();
    }

    fn rec_chapter(&mut self) {
        if self.rec.is_none() {
            return;
        }
        self.rec_n = 0;
        self.rec_delta.reset();
        let mut boards = Vec::new();
        for i in 0..self.boards.len() {
            let rows = self.rec_rows_full(i);
            let st = self.rec_stat(i);
            let p = Self::rec_piece(&self.boards[i]);
            boards.push(json!({"rows": rows, "st": st, "p": p}));
        }
        let mut info = Map::new();
        let n = self.rec.as_ref().map(|r| r.scene_count()).unwrap_or(0) + 1;
        info.insert("n".into(), json!(n));
        info.insert("t0".into(), replay::rnd(self.elapsed, 3));
        info.insert("boards".into(), Value::Array(boards));
        if let Some(r) = self.rec.as_mut() {
            r.scene(false, info);
        }
    }

    fn rec_piece(b: &Board) -> [i64; 4] {
        let k = core::kind_index(b.kind) as i64 + 1;
        [k * 8 + b.rot as i64 * 2 + b.active as i64, b.x as i64, b.y as i64, (b.lock_timer * 100.0) as i64]
    }

    fn rec_row(b: &Board, y: usize) -> [i64; 10] {
        let mut out = [0i64; 10];
        for (x, c) in b.cells[y].iter().enumerate() {
            out[x] = rec_idx(*c);
        }
        out
    }

    fn rec_rows_full(&self, i: usize) -> Vec<i64> {
        let b = &self.boards[i];
        let mut out = Vec::new();
        for y in 0..core::ROWS as usize {
            if b.rows[y] == 0 {
                continue;
            }
            out.push(y as i64);
            out.extend(Self::rec_row(b, y));
        }
        out
    }

    fn rec_rows_delta(&mut self, i: usize) -> Vec<i64> {
        let mut out = Vec::new();
        let ver = self.boards[i].version;
        if !self.rec_delta.push(1000 + i as i64, &[json!(ver)]) {
            return out;
        }
        for y in 0..core::ROWS as usize {
            let row = if self.boards[i].rows[y] != 0 { Self::rec_row(&self.boards[i], y) } else { [0; 10] };
            let vals: Vec<Value> = row.iter().map(|v| json!(v)).collect();
            if self.rec_delta.push(10000 + i as i64 * 100 + y as i64, &vals) {
                out.push(y as i64);
                out.extend(row);
            }
        }
        out
    }

    fn rec_stat(&mut self, i: usize) -> Vec<i64> {
        let b = &mut self.boards[i];
        let mut nxt = b.queue.peek(5);
        while nxt.len() < 5 {
            nxt.push('I');
        }
        let mut st = vec![b.score, b.lines, b.level, b.pieces, rec_idx(b.hold_kind), b.hold_used as i64];
        st.extend(nxt.iter().take(5).map(|k| rec_idx(Some(*k))));
        st.extend([b.dead as i64, b.combo, b.b2b as i64, b.attack_sent, b.pending.len() as i64]);
        for (n, hole, age) in &b.pending {
            st.extend([*n, *hole, (age * 100.0) as i64]);
        }
        st
    }

    fn rec_sample(&mut self) -> Vec<Value> {
        let mut out: Vec<i64> = Vec::new();
        for i in 0..self.boards.len() {
            out.extend(Self::rec_piece(&self.boards[i]));
            let rows = self.rec_rows_delta(i);
            out.push(rows.len() as i64 / (COLS as i64 + 1));
            out.extend(rows);
            let st = self.rec_stat(i);
            let vals: Vec<Value> = st.iter().map(|v| json!(v)).collect();
            if self.rec_delta.push(2000 + i as i64, &vals) {
                out.push(st.len() as i64);
                out.extend(st);
            } else {
                out.push(0);
            }
        }
        self.rec_n += 1;
        out.into_iter().map(|v| json!(v)).collect()
    }

    fn rec_tick(&mut self, dt: f64) {
        if self.rec.is_none() {
            return;
        }
        if self.rec_n >= REC_CHAPTER {
            self.rec_chapter();
        }
        let mut rec = self.rec.take();
        if let Some(r) = rec.as_mut() {
            r.tick(dt, || Some(self.rec_sample()));
        }
        self.rec = rec;
    }

    fn rec_sub(&self) -> String {
        let b = &self.boards[0];
        if self.versus {
            let win = self.result.as_ref().and_then(|r| r.winner);
            let Some(w) = win else {
                return t("common.draw");
            };
            if self.b.mode == "versus_ai" {
                return if w == 0 { t("tetris.res.you_win") } else { t("tetris.res.ai_wins") };
            }
            return tf("common.player_wins_name", &[("name", &self.b.pname(w))]);
        }
        if self.variant == "sprint" {
            return t("tetris.hud.lines") + &format!(" {} \u{00B7} {}", b.lines, fmt_time(self.elapsed, true));
        }
        if self.variant == "ultra" {
            return tf("common.points", &[("score", &b.score.to_string())]);
        }
        tf("common.points", &[("score", &b.score.to_string())]) + "  \u{00B7}  " + &t("tetris.hud.lines") + &format!(" {}", b.lines)
    }

    fn rec_finish(&mut self) {
        if self.rec.is_none() {
            return;
        }
        let last = self.rec_sample();
        let mut rec = self.rec.take();
        if let Some(r) = rec.as_mut() {
            let mut info = Map::new();
            info.insert("final".into(), json!(true));
            r.close(Some(last), info);
        }
        let title = if !self.versus { t(&format!("tetris.variant.{}", self.variant)) } else { t(&format!("tetris.mode.{}", self.b.mode)) };
        let mut meta = Map::new();
        meta.insert("score".into(), json!(self.boards[0].score));
        meta.insert("time".into(), replay::rnd(self.elapsed, 2));
        let sub = self.rec_sub();
        self.replay = rec.as_mut().and_then(|r| r.result(&title, &sub, meta));
        self.rec = None;
    }

    fn open_replay(&mut self) {
        if self.replay.is_some() {
            self.replay_request = self.replay.clone();
            self.b.play_sound("click");
        }
    }

    // ===================================================== Replay-Wiedergabe

    /// `replay_begin(rep)`
    pub fn replay_begin(&mut self, rep: &Value) {
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = Some(rep.clone());
        self.rep_at = None;
        let meta = rep.get("meta").and_then(|m| m.as_object()).cloned().unwrap_or_default();
        if let Some(m) = meta.get("mode").and_then(|v| v.as_str()) {
            if ["solo", "versus_ai", "multi"].contains(&m) {
                self.b.mode = m.to_string();
            }
        }
        self.b.multiplayer = self.b.mode == "multi";
        self.versus = self.b.mode != "solo";
        if let Some(v) = meta.get("variant").and_then(|v| v.as_str()) {
            if let Some(vv) = VARIANTS.iter().find(|x| **x == v) {
                self.variant = vv;
            }
        }
        self.ai_level = 0.max(2.min(meta.get("ai").and_then(|v| v.as_i64()).unwrap_or(0)));
        self.ghost = meta.get("ghost").map(|v| v.as_bool().unwrap_or(true)).unwrap_or(true);
        let n = if self.versus { 2 } else { 1 };
        self.boards = (0..n).map(|_| Board::new(1, 1, true)).collect();
        for b in self.boards.iter_mut() {
            b.queue = Queue::Rep(Vec::new());
        }
        self.fx = (0..n).map(|_| Fx::new()).collect();
        self.pads = (0..n).map(|_| Pad::default()).collect();
        self.ai = None;
        self.result = None;
        self.particles.clear();
        self.callouts.clear();
        self.missiles.clear();
        self.caches.clear();
        self.stack_cache.clear();
        self.elapsed = 0.0;
        self.go_t = 0.0;
        self.count_t = 0.0;
        self.b.game_over = false;
        self.state = St::Play;
        self.layout();
        self.replay_seek(0, 0);
    }

    /// `replay_seek(index, frame)`
    pub fn replay_seek(&mut self, index: usize, frame: usize) {
        let Some(rep) = self.rep.clone() else {
            return;
        };
        let scenes = rep.get("scenes").and_then(|s| s.as_array()).cloned().unwrap_or_default();
        if scenes.is_empty() {
            return;
        }
        let index = index.min(scenes.len() - 1);
        let sc = &scenes[index];
        let frames = sc.get("f").and_then(|f| f.as_array()).cloned().unwrap_or_default();
        let n = 1.max(frames.len());
        let frame = frame.min(n - 1);
        let ints = |v: Option<&Value>| -> Vec<i64> { v.and_then(|a| a.as_array()).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as i64).collect()).unwrap_or_default() };
        let start;
        if self.rep_at.is_none_or(|(i, f)| i != index || frame < f) {
            let heads = sc.get("boards").and_then(|b| b.as_array()).cloned().unwrap_or_default();
            for i in 0..self.boards.len() {
                let head = heads.get(i).cloned().unwrap_or(json!({}));
                let b = &mut self.boards[i];
                b.rows = vec![0; core::ROWS as usize];
                b.cells = vec![[None; 10]; core::ROWS as usize];
                let flat = ints(head.get("rows"));
                let mut k = 0;
                while k + (COLS as usize) < flat.len() {
                    let y = flat[k];
                    if (0..core::ROWS as i64).contains(&y) {
                        Self::rep_row(b, y as usize, &flat[k + 1..k + 1 + COLS as usize]);
                    }
                    k += COLS as usize + 1;
                }
                b.version += 1;
                let p = ints(head.get("p"));
                Self::rep_piece(b, if p.len() >= 4 { &p } else { &[0, 0, 0, 0] });
                let st = ints(head.get("st"));
                self.rep_stat(i, &st);
            }
            start = 0;
        } else {
            start = self.rep_at.map(|(_, f)| f + 1).unwrap_or(0);
        }
        for fr in frames.iter().take(frame + 1).skip(start) {
            let v: Vec<f64> = fr.as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()).unwrap_or_default();
            self.rep_apply(&v);
        }
        let rate = rep.get("rate").and_then(|r| r.as_f64()).filter(|r| *r != 0.0).unwrap_or(replay::RATE);
        self.elapsed = sc.get("t0").and_then(|v| v.as_f64()).unwrap_or(0.0) + frame as f64 / rate;
        self.b.score = self.boards[0].score;
        self.rep_at = Some((index, frame));
    }

    fn rep_row(b: &mut Board, y: usize, cells: &[i64]) {
        let mut mask = 0u32;
        for x in 0..COLS as usize {
            let idx = *cells.get(x).unwrap_or(&0);
            b.cells[y][x] = if idx != 0 { REC_KINDS.get(idx as usize).copied() } else { None };
            if idx != 0 {
                mask |= 1 << x;
            }
        }
        b.rows[y] = mask;
    }

    fn rep_piece(b: &mut Board, p: &[i64]) {
        let code = p[0];
        let k = code / 8;
        b.kind = if (1..=7).contains(&k) { core::KINDS[(k - 1) as usize] } else { core::KINDS[0] };
        b.rot = ((code % 8) / 2) as i32;
        b.active = code % 2 != 0;
        b.x = p[1] as i32;
        b.y = p[2] as i32;
        b.lock_timer = p[3] as f64 / 100.0;
        b.lowest = b.y;
    }

    fn rep_stat(&mut self, i: usize, st: &[i64]) {
        if st.len() < 16 {
            return;
        }
        let b = &mut self.boards[i];
        b.score = st[0];
        b.lines = st[1];
        b.level = st[2];
        b.pieces = st[3];
        b.hold_kind = if st[4] != 0 { REC_KINDS.get(st[4] as usize).copied() } else { None };
        b.hold_used = st[5] != 0;
        b.queue = Queue::Rep(st[6..11].iter().map(|v| *REC_KINDS.get(*v as usize).unwrap_or(&'I')).collect());
        b.dead = st[11] != 0;
        b.combo = st[12];
        b.b2b = st[13] != 0;
        b.attack_sent = st[14];
        b.pending.clear();
        let np = st[15] as usize;
        let mut p = 16;
        for _ in 0..np {
            if p + 2 >= st.len() {
                break;
            }
            b.pending.push((st[p], st[p + 1], st[p + 2] as f64 / 100.0));
            p += 3;
        }
        self.fx[i].ko_t = if b.dead { 0.8 } else { -1.0 };
    }

    fn rep_apply(&mut self, fr: &[f64]) {
        let mut p = 0;
        for i in 0..self.boards.len() {
            if p + 4 > fr.len() {
                return;
            }
            let piece: Vec<i64> = fr[p..p + 4].iter().map(|v| *v as i64).collect();
            Self::rep_piece(&mut self.boards[i], &piece);
            p += 4;
            let nrows = *fr.get(p).unwrap_or(&0.0) as usize;
            p += 1;
            for _ in 0..nrows {
                let y = fr[p] as i64;
                if (0..core::ROWS as i64).contains(&y) {
                    let cells: Vec<i64> = fr[p + 1..p + 1 + COLS as usize].iter().map(|v| *v as i64).collect();
                    Self::rep_row(&mut self.boards[i], y as usize, &cells);
                }
                p += COLS as usize + 1;
            }
            if nrows != 0 {
                self.boards[i].version += 1;
            }
            let k = *fr.get(p).unwrap_or(&0.0) as usize;
            p += 1;
            if k != 0 {
                let st: Vec<i64> = fr[p..p + k].iter().map(|v| *v as i64).collect();
                self.rep_stat(i, &st);
                p += k;
            }
        }
    }

    /// `replay_draw(aiming, banner)`
    pub fn replay_draw(&mut self, banner: bool) {
        let mut s = self.b.sf();
        self.anim_t += 1.0 / replay::RATE;
        ui::draw_background(&mut s, self.b.width, self.b.height);
        for i in 0..self.boards.len() {
            self.draw_side(&mut s, i);
        }
        if self.versus {
            self.draw_vs_hud(&mut s);
        }
        if banner {
            let d = self.dim(140);
            s.bl(&d, (0, 0));
            let head = self.huge.render(&t("common.game_over"), true, self.b.accent);
            s.blr(&head, head.get_rect().center_at((self.b.width / 2, self.b.height / 2 - 18)));
            let sub_t = self.rep.as_ref().and_then(|r| r.get("sub")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let sub = self.big.render(&sub_t, true, ui::pal().text);
            s.blr(&sub, sub.get_rect().center_at((self.b.width / 2, self.b.height / 2 + 26)));
        }
    }

    // ===================================================== Effekte

    fn particle(&mut self, x: f64, y: f64, color: Rgb, c: i32, power: f64) {
        if self.particles.len() > 420 {
            return;
        }
        let sp = c as f64 * 9.0 * power;
        let a = py::uniform(0.0, TAU);
        let v = py::uniform(0.25, 1.0) * sp;
        let life = py::uniform(0.45, 0.9);
        let size = 2.max((c as f64 * py::uniform(0.14, 0.3)) as i32);
        self.particles.push(Particle { x, y, vx: a.cos() * v, vy: a.sin() * v - sp * 0.5, life, maxl: life, col: color, size });
    }

    fn confetti(&mut self, i: usize) {
        let field = self.sides[i].field;
        let c = self.sides[i].cell;
        for _ in 0..60 {
            let x = py::uniform(field.x as f64, field.right() as f64);
            let y = field.y as f64 + py::uniform(0.0, field.h as f64 * 0.5);
            let col = color_of(py::choice(&COLOR_ORDER));
            self.particle(x, y, col, c, 1.3);
        }
    }

    fn callout(&mut self, i: usize, rows: Vec<(String, Color)>, small: Vec<String>) {
        let field = self.sides[i].field;
        let max_w = field.w - 8;
        let mut surfs = Vec::new();
        for (text, color) in rows {
            surfs.push(Self::fit_render(&text, &self.f_call, color, max_w));
        }
        for text in small {
            surfs.push(Self::fit_render(&text, &self.f_call_s, ui::pal().text, max_w));
        }
        self.callouts.retain(|cl| cl.side != i);
        self.callouts.push(Callout { side: i, surfs, t: 0.0, dur: 1.25 });
    }

    /// `_fit_render` - immer eine eigene Flaeche (Python rendert neu).
    fn fit_render(text: &str, fnt: &Font, color: impl Col, max_w: i32) -> Surface {
        let img = fnt.render(text, true, color.c()).copy();
        if img.w() > max_w && max_w > 10 {
            let h = 6.max(((img.h() * max_w) as f64 / img.w() as f64) as i32);
            return transform::smoothscale(&img, max_w, h);
        }
        img
    }

    fn update_effects(&mut self, dt: f64) {
        for f in self.fx.iter_mut() {
            f.shake = 0f64.max(f.shake - dt);
            f.clear_t += dt;
            f.flash_t += dt;
            f.trail_t += dt;
            f.level_t += dt;
            f.rise_t += dt;
            f.cancel_t += dt;
        }
        let mut alive = Vec::new();
        for mut p in std::mem::take(&mut self.particles) {
            p.life -= dt;
            if p.life <= 0.0 {
                continue;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 1400.0 * dt * (p.size as f64 / 6.0);
            alive.push(p);
        }
        self.particles = alive;
        for cl in self.callouts.iter_mut() {
            cl.t += dt;
        }
        self.callouts.retain(|cl| cl.t < cl.dur);
        let mut keep = Vec::new();
        let mut boom = Vec::new();
        for mut m in std::mem::take(&mut self.missiles) {
            m.t += dt;
            if m.t < m.dur {
                keep.push(m);
            } else {
                boom.push((m.x1, m.y1, m.color));
            }
        }
        self.missiles = keep;
        for (x, y, col) in boom {
            for _ in 0..8 {
                self.particle(x, y, col, 12, 0.8);
            }
        }
    }

    fn new_deco(anywhere: bool) -> (char, f64, f64, f64, i64) {
        let k = py::choice(&core::KINDS);
        let fx = py::uniform(0.0, 1.0);
        let fy = if anywhere { py::uniform(0.0, 1.0) } else { py::uniform(-0.3, -0.1) };
        let sp = py::uniform(0.02, 0.06);
        let r = py::randint(0, 3);
        (k, fx, fy, sp, r)
    }

    fn update_deco(&mut self, dt: f64) {
        for d in self.deco.iter_mut() {
            d.2 += d.3 * dt;
        }
        let old = std::mem::take(&mut self.deco);
        self.deco = old.into_iter().map(|d| if d.2 < 1.15 { d } else { Self::new_deco(false) }).collect();
    }

    // ===================================================== Zeichen-Helfer

    fn flat() -> bool {
        ui::fx().style == "v1"
    }

    /// style: 0 = Block, 1 = Geist, 2 = weiss.
    fn sprite(&mut self, kind: char, c: i32, style: u8) -> Surface {
        let flat = Self::flat();
        let key = (kind, c, style, flat);
        if let Some(s) = self.sprites.get(&key) {
            return s.clone();
        }
        let base = color_of(kind);
        let mut surf = pg::surface_alpha(c, c);
        let s = c - 1;
        if style == 1 {
            draw::rect_r(&mut surf, rgba_of(base, 50.0), Rect::new(1, 1, s - 1, s - 1), 0, 1.max(c / 7));
            draw::rect_r(&mut surf, rgba_of(base, 190.0), Rect::new(1, 1, s - 1, s - 1), 1.max(c / 12), 1.max(c / 7));
        } else if style == 2 {
            draw::rect_r(&mut surf, rgba(255, 255, 255, 255), Rect::new(0, 0, s, s), 0, 1.max(c / 8));
        } else if flat || c < 9 {
            draw::rect_r(&mut surf, base, Rect::new(0, 0, s, s), 0, 0.max(c / 9));
            draw::rect_r(&mut surf, ((base.0 as f64 * 0.7) as i32, (base.1 as f64 * 0.7) as i32, (base.2 as f64 * 0.7) as i32), Rect::new(0, 0, s, s), 1, 0.max(c / 9));
        } else {
            let bev = 2.max(c / 6);
            let light = ui::mix(base, (255, 255, 255), 0.42);
            let dark = ((base.0 as f64 * 0.58) as i32, (base.1 as f64 * 0.58) as i32, (base.2 as f64 * 0.58) as i32);
            let face = ui::mix(base, (255, 255, 255), 0.06);
            draw::rect_r(&mut surf, dark, Rect::new(0, 0, s, s), 0, 1.max(c / 10));
            draw::polygon(&mut surf, light, &[(0, 0), (s - 1, 0), (s - bev, bev), (bev, bev), (bev, s - bev), (0, s - 1)], 0);
            draw::rect(&mut surf, face, Rect::new(bev, bev, s - 2 * bev, s - 2 * bev), 0);
            let mut hl = pg::surface_alpha(1.max((s - 2 * bev) / 2), 1.max(bev / 2 + 1));
            hl.fill_c(rgba(255, 255, 255, 80));
            surf.bl(&hl, (bev + 1, bev + 1));
        }
        self.sprites.insert(key, surf.clone());
        surf
    }

    fn piece_img(&mut self, kind: char, mc: i32, alpha: i32) -> Surface {
        let key = (kind, mc, alpha, Self::flat());
        if let Some(s) = self.pieces.get(&key) {
            return s.clone();
        }
        let sh = core::shape(kind, 0);
        let w = (sh.maxx - sh.minx + 1) * mc;
        let h = (sh.maxy - sh.miny + 1) * mc;
        let mut img = pg::surface_alpha(w, h);
        let spr = self.sprite(if alpha >= 255 { kind } else { 'X' }, mc, 0);
        for &(cx, cy) in &sh.cells {
            img.bl(&spr, ((cx - sh.minx) * mc, (cy - sh.miny) * mc));
        }
        if 0 < alpha && alpha < 255 {
            img.set_alpha(Some(alpha as u8));
        }
        self.pieces.insert(key, img.clone());
        img
    }

    fn field_bg(&mut self, w: i32, h: i32, c: i32) -> Surface {
        let p = ui::pal();
        let key = format!("fieldbg{w},{h},{c},{:?},{:?},{:?}", p.panel, p.border, p.bg_bottom);
        if let Some(s) = self.caches.get(&key) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(w, h);
        let base = ui::mix(p.bg_bottom, p.panel, 0.35);
        for y in 0..h {
            let f = y as f64 / 1.max(h - 1) as f64;
            draw::line(&mut surf, rgba_of(ui::mix(base, p.panel, f * 0.5), 238.0), (0, y), (w, y), 1);
        }
        let grid = rgba_of(ui::mix(p.panel, p.border, 0.45), 110.0);
        for cx in 1..COLS {
            draw::line(&mut surf, grid, (cx * c, 0), (cx * c, h), 1);
        }
        for cy in 1..VISIBLE {
            draw::line(&mut surf, grid, (0, cy * c), (w, cy * c), 1);
        }
        self.caches.insert(key, surf.clone());
        surf
    }

    fn stack_surface(&mut self, i: usize, c: i32, gray_rows: i32) -> Surface {
        let key = (self.boards[i].version, c, gray_rows, Self::flat());
        if let Some((k, s)) = self.stack_cache.get(&i) {
            if *k == key {
                return s.clone();
            }
        }
        let mut surf = pg::surface_alpha(COLS * c, VISIBLE * c);
        for y in BUFFER..core::ROWS {
            if self.boards[i].rows[y as usize] == 0 {
                continue;
            }
            let row = self.boards[i].cells[y as usize];
            let gray = (core::ROWS - y) <= gray_rows;
            for x in 0..COLS {
                if let Some(kind) = row[x as usize] {
                    let spr = self.sprite(if gray { 'X' } else { kind }, c, 0);
                    surf.bl(&spr, (x * c, (y - BUFFER) * c));
                }
            }
        }
        self.stack_cache.insert(i, (key, surf.clone()));
        surf
    }

    fn white(&mut self, w: i32, h: i32) -> Surface {
        let key = format!("white{w},{h}");
        if let Some(s) = self.caches.get(&key) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(1.max(w), 1.max(h));
        surf.fill_c(rgba(255, 255, 255, 255));
        self.caches.insert(key, surf.clone());
        surf
    }

    fn dim(&mut self, alpha: i32) -> Surface {
        let key = format!("dim{},{},{alpha}", self.b.width, self.b.height);
        if let Some(s) = self.caches.get(&key) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(self.b.width, self.b.height);
        surf.fill_c(rgba(8, 10, 16, alpha));
        self.caches.insert(key, surf.clone());
        surf
    }

    fn panel(&mut self, s: &mut Surface, rect: Rect, alpha: i32) {
        let p = ui::pal();
        let key = format!("panel{},{},{:?},{alpha}", rect.w, rect.h, p.panel);
        let surf = match self.caches.get(&key) {
            Some(sf) => sf.clone(),
            None => {
                let mut sf = pg::surface_alpha(rect.w, rect.h);
                let r = sf.get_rect();
                draw::rect_r(&mut sf, rgba_of(p.panel, alpha as f64), r, 0, 4.max(12.min(rect.h / 6)));
                self.caches.insert(key, sf.clone());
                sf
            }
        };
        s.bl(&surf, (rect.x, rect.y));
        draw::rect_r(s, p.border, rect, 1, 4.max(12.min(rect.h / 6)));
    }

    fn label(&self, s: &mut Surface, text: &str, rect: Rect, y: i32) -> i32 {
        let col = ui::pal().text_dim;
        let max_w = rect.w - 6;
        let mut img = self.f_label.render(text, true, col);
        if img.w() > max_w {
            let small = ui::font(9.max((self.f_label.get_height() as f64 * 0.72) as i32), true, false);
            img = small.render(text, true, col);
            if img.w() > max_w {
                return y;
            }
        }
        s.blr(&img, img.get_rect().midtop_at((rect.centerx(), y)));
        y + img.h()
    }

    // ===================================================== Zeichnen

    fn draw_side(&mut self, s: &mut Surface, i: usize) {
        let l = self.sides[i];
        let mut field = l.field;
        let f = &self.fx[i];
        if f.shake > 0.0 {
            let amp = 1.max(l.cell / 7) as f64;
            field = field.move_(0, (amp * (f.shake * 70.0).sin() * f.shake * 6.0) as i32);
        }
        self.draw_field(s, i, field, l.cell);
        self.draw_hold(s, i, l);
        self.draw_next(s, i, l);
        if l.bar.is_some() {
            self.draw_bar(s, i, l);
        }
        if l.stats.is_some() {
            self.draw_stats(s, l);
        }
    }

    fn draw_field(&mut self, s: &mut Surface, i: usize, rect: Rect, c: i32) {
        let p = ui::pal();
        let bg = self.field_bg(rect.w, rect.h, c);
        s.bl(&bg, (rect.x, rect.y));
        let mut gray_rows = 0;
        if self.fx[i].ko_t >= 0.0 {
            gray_rows = (1f64.min(self.fx[i].ko_t / 0.8) * VISIBLE as f64 + 0.999) as i32;
        }
        let stack = self.stack_surface(i, c, gray_rows);
        let prev_clip = s.get_clip();
        s.set_clip(Some(rect));
        let (clear_t, rise_t, rise_n) = (self.fx[i].clear_t, self.fx[i].rise_t, self.fx[i].rise_n);
        if clear_t < CLEAR_ANIM && !self.fx[i].shift.is_empty() {
            let e = ease_out(clear_t / CLEAR_ANIM);
            for vy in 0..VISIBLE {
                let k = *self.fx[i].shift.get(&(vy + BUFFER)).unwrap_or(&0);
                let dy = -((k as f64 * c as f64 * (1.0 - e)) as i32);
                s.bl_area(&stack, (rect.x, rect.y + vy * c + dy), Rect::new(0, vy * c, rect.w, c));
            }
        } else if rise_t < 0.14 && rise_n != 0 {
            let e = ease_out(rise_t / 0.14);
            s.bl(&stack, (rect.x, rect.y + (rise_n as f64 * c as f64 * (1.0 - e)) as i32));
        } else {
            s.bl(&stack, (rect.x, rect.y));
        }
        if clear_t < 0.32 {
            let pp = clear_t / 0.32;
            let mut bar = self.white(rect.w, c);
            for (r, _row) in self.fx[i].clear_rows.clone() {
                if r < BUFFER {
                    continue;
                }
                let w = (rect.w as f64 * (0.55 + 0.45 * ease_out(pp * 1.6))) as i32;
                bar.set_alpha(Some((230.0 * (1.0 - pp)) as i32 as u8));
                s.bl_area(&bar, (rect.centerx() - w / 2, rect.y + (r - BUFFER) * c), Rect::new(0, 0, w, c));
            }
        }
        if let Some((cols, kind)) = self.fx[i].trail.clone() {
            if self.fx[i].trail_t < 0.2 {
                let mut strip = self.trail_strip(kind, c);
                strip.set_alpha(Some((200.0 * (1.0 - self.fx[i].trail_t / 0.2)) as i32 as u8));
                for (x, (top, bot)) in cols {
                    let y0 = top.max(BUFFER) - BUFFER;
                    let y1 = bot - BUFFER;
                    if y1 <= y0 {
                        continue;
                    }
                    let hgt = (y1 - y0) * c;
                    s.bl_area(&strip, (rect.x + x * c, rect.y + y0 * c), Rect::new(0, strip.h() - hgt, c, hgt));
                }
            }
        }
        if self.boards[i].active {
            if self.ghost {
                let gy = self.boards[i].ghost_y();
                if gy != self.boards[i].y {
                    let spr = self.sprite(self.boards[i].kind, c, 1);
                    for (x, y) in self.boards[i].cells_at(Some(gy)) {
                        if y >= BUFFER {
                            s.bl(&spr, (rect.x + x * c, rect.y + (y - BUFFER) * c));
                        }
                    }
                }
            }
            let spr = self.sprite(self.boards[i].kind, c, 0);
            let dimv = if self.boards[i].grounded() { 1f64.min(self.boards[i].lock_timer / core::LOCK_DELAY) } else { 0.0 };
            for (x, y) in self.boards[i].cells_at(None) {
                if y >= BUFFER - 2 {
                    let (px, py_) = (rect.x + x * c, rect.y + (y - BUFFER) * c);
                    s.bl(&spr, (px, py_));
                    if dimv > 0.05 {
                        let mut w = self.white(c - 1, c - 1);
                        w.set_alpha(Some((70.0 * dimv) as i32 as u8));
                        s.bl(&w, (px, py_));
                    }
                }
            }
        }
        if self.fx[i].flash_t < 0.14 {
            let mut w = self.white(c - 1, c - 1);
            w.set_alpha(Some((150.0 * (1.0 - self.fx[i].flash_t / 0.14)) as i32 as u8));
            for (x, y) in self.fx[i].flash_cells.clone() {
                if y >= BUFFER {
                    s.bl(&w, (rect.x + x * c, rect.y + (y - BUFFER) * c));
                }
            }
        }
        s.set_clip(Some(prev_clip));
        let b = &self.boards[i];
        let mut border = p.border_light;
        let mut width = 2;
        let height = b.stack_height();
        let danger = !b.dead && (height >= 17 || height as i64 + b.pending_lines() >= 19);
        if danger {
            border = ui::mix(p.border_light, p.red, ui::pulse(7.0, 0.35, 1.0));
            width = 3;
        }
        let lt = self.fx[i].level_t;
        if lt < 0.8 {
            border = ui::mix(p.gold, border, lt / 0.8);
            width = 3;
        }
        draw::rect_r(s, border, rect.inflate(4, 4), width, 3.max(c / 5));
    }

    fn trail_strip(&mut self, kind: char, c: i32) -> Surface {
        let key = format!("trail{kind},{c}");
        if let Some(s) = self.caches.get(&key) {
            return s.clone();
        }
        let h = VISIBLE * c;
        let mut surf = pg::surface_alpha(c, h);
        let col = color_of(kind);
        for y in 0..h {
            let a = (110.0 * py::pow(y as f64 / h as f64, 2.0)) as i32;
            draw::line(&mut surf, rgba(col.0, col.1, col.2, a), (c / 5, y), (c - c / 5 - 1, y), 1);
        }
        self.caches.insert(key, surf.clone());
        surf
    }

    fn draw_hold(&mut self, s: &mut Surface, i: usize, l: Side) {
        let r = l.hold;
        self.panel(s, r, 205);
        let y = self.label(s, &t("tetris.hud.hold"), r, r.y + l.pad / 2 + 1);
        if let Some(hk) = self.boards[i].hold_kind {
            let img = self.piece_img(hk, l.mini, if !self.boards[i].hold_used { 255 } else { 120 });
            let area_top = y + 2;
            let cy = area_top + (r.bottom() - area_top) / 2;
            s.blr(&img, img.get_rect().center_at((r.centerx(), cy)));
        }
    }

    fn draw_next(&mut self, s: &mut Surface, i: usize, l: Side) {
        let r = l.next;
        self.panel(s, r, 205);
        let y = self.label(s, &t("tetris.hud.next"), r, r.y + l.pad / 2 + 1) + l.pad / 2;
        let kinds = self.boards[i].queue.peek(5);
        let slot1 = (l.mini as f64 * 2.9) as i32;
        let rest = (r.bottom() - y - slot1 - 4) as f64 / 4.0;
        for (k, kind) in kinds.iter().enumerate() {
            let mc = if k == 0 { l.mini } else { l.mini2 };
            let img = self.piece_img(*kind, mc, 255);
            let cy = if k == 0 { (y + slot1 / 2) as f64 } else { (y + slot1) as f64 + (rest * (k as f64 - 0.5)) as i32 as f64 };
            s.blr(&img, img.get_rect().center_at((r.centerx(), cy as i32)));
        }
    }

    fn draw_bar(&mut self, s: &mut Surface, i: usize, l: Side) {
        let p = ui::pal();
        let Some(r) = l.bar else {
            return;
        };
        let c = l.cell;
        draw::rect_r(s, rgba_of(p.panel, 255.0), r, 0, 2);
        let mut y = r.bottom();
        for &(n, _hole, age) in &self.boards[i].pending {
            let h = (n as i32 * c).min(y - r.y);
            if h <= 0 {
                break;
            }
            let ready = age >= core::GARBAGE_DELAY;
            let col = if ready { ui::mix(p.red, (255, 255, 255), 0.25 * ui::pulse(9.0, 0.0, 1.0)) } else { p.gold };
            draw::rect_r(s, col, Rect::new(r.x, y - h, r.w, h - 1), 0, 2);
            y -= h;
        }
        if self.fx[i].cancel_t < 0.3 {
            let mut w = self.white(r.w + 4, r.h);
            w.set_alpha(Some((120.0 * (1.0 - self.fx[i].cancel_t / 0.3)) as i32 as u8));
            s.bl(&w, (r.x - 2, r.y));
        }
        draw::rect_r(s, p.border, r.inflate(2, 2), 1, 2);
    }

    fn stat_rows(&self) -> Vec<(String, String, bool, Option<Color>)> {
        let p = ui::pal();
        let b = &self.boards[0];
        let el = self.elapsed;
        let pps = format!("{:.2}", if el > 0.5 { b.pieces as f64 / el } else { 0.0 });
        if self.variant == "sprint" {
            let left = 0.max(SPRINT_LINES - b.lines);
            return vec![
                (t("tetris.hud.time"), fmt_time(el, true), true, None),
                (t("tetris.hud.lines"), format!("{} / {}", b.lines.min(SPRINT_LINES), SPRINT_LINES), false, if left <= 5 { Some(p.green) } else { None }),
                (t("tetris.hud.pps"), pps, false, None),
                (t("tetris.hud.level"), b.level.to_string(), false, None),
            ];
        }
        if self.variant == "ultra" {
            let rest = 0f64.max(ULTRA_TIME - el);
            return vec![
                (t("tetris.hud.left"), fmt_time(rest, true), true, if rest < 10.0 { Some(p.red) } else { None }),
                (t("tetris.hud.score"), b.score.to_string(), false, None),
                (t("tetris.hud.lines"), b.lines.to_string(), false, None),
                (t("tetris.hud.pps"), pps, false, None),
            ];
        }
        vec![
            (t("tetris.hud.score"), b.score.to_string(), true, None),
            (t("tetris.hud.level"), b.level.to_string(), false, None),
            (t("tetris.hud.lines"), b.lines.to_string(), false, None),
            (t("tetris.hud.time"), fmt_time(el, false), false, None),
            (t("tetris.hud.pps"), pps, false, None),
        ]
    }

    fn draw_stats(&mut self, s: &mut Surface, l: Side) {
        let p = ui::pal();
        let Some(r) = l.stats else {
            return;
        };
        let rows = self.stat_rows();
        let pad = l.pad;
        let lab_h = self.f_label.get_height();
        let mut y = r.y + pad;
        let mut used = Vec::new();
        for (label, value, hero, col) in rows {
            let fh = if hero { self.f_hero.get_height() } else { self.f_value.get_height() };
            let h = lab_h + fh + pad;
            if y + h > r.bottom() {
                break;
            }
            used.push((label, value, hero, col, y));
            y += h;
        }
        let panel = Rect::new(r.x, r.y, r.w, y - r.y);
        self.panel(s, panel, 205);
        for (label, value, hero, col, yy) in used {
            let mut img = self.f_label.render(&label, true, p.text_dim);
            if img.w() > r.w - 12 {
                img = transform::smoothscale(&img, r.w - 12, img.h());
            }
            s.blr(&img, img.get_rect().midtop_at((r.centerx(), yy)));
            let fnt = if hero { &self.f_hero } else { &self.f_value };
            let vc = col.unwrap_or(if hero { self.b.accent } else { p.text });
            let val = Self::fit_render(&value, fnt, vc, r.w - 10);
            s.blr(&val, val.get_rect().midtop_at((r.centerx(), yy + lab_h)));
        }
    }

    fn draw_vs_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        for i in 0..2 {
            let l = self.sides[i];
            let field = l.field;
            let col = if i == 0 { COL_P1 } else { COL_P2 };
            let name = if self.b.mode == "versus_ai" {
                if i == 0 {
                    t("tetris.player.you")
                } else {
                    tf("tetris.player.ai", &[("level", &t(&format!("tetris.ai.{}", self.ai_level)))])
                }
            } else {
                self.b.pname(i).chars().take(8).collect()
            };
            let text = format!("{}   {}", name, self.wins[i]);
            let img = Self::fit_render(&text, &self.f_value, col, l.next.right() - l.hold.x);
            s.blr(&img, img.get_rect().midbottom_at((field.centerx(), field.y - 6)));
            let b = &self.boards[i];
            let line = tf("tetris.vs_line", &[("lines", &b.lines.to_string()), ("attack", &b.attack_sent.to_string())]);
            let img = Self::fit_render(&line, &self.f_label, p.text_dim, l.next.right() - l.hold.x);
            s.blr(&img, img.get_rect().midtop_at((field.centerx(), field.bottom() + 6)));
        }
        let tm = self.f_label.render(&fmt_time(self.elapsed, false), true, p.text_dim);
        let gap_mid = (self.sides[0].next.right() + self.sides[1].hold.x) / 2;
        s.blr(&tm, tm.get_rect().midbottom_at((gap_mid, self.sides[0].field.y - 6)));
    }

    fn draw_particles(&self, s: &mut Surface) {
        for pt in &self.particles {
            let k = pt.life / pt.maxl;
            let sz = 1.max((pt.size as f64 * (0.5 + 0.5 * k)) as i32);
            s.fill_r(pt.col, Rect::new(pt.x as i32 - sz / 2, pt.y as i32 - sz / 2, sz, sz));
        }
    }

    fn draw_missiles(&self, s: &mut Surface) {
        for m in &self.missiles {
            let pp = m.t / m.dur;
            for k in 0..5 {
                let q = 0f64.max(pp - k as f64 * 0.05);
                let e = ease_out(q);
                let x = m.x0 + (m.x1 - m.x0) * e;
                let y = m.y0 + (m.y1 - m.y0) * e - (q * PI).sin() * 40.0;
                let rad = 2.max(((6 + 6.min(m.n)) as f64 * (1.0 - k as f64 * 0.18)) as i32);
                let col = ui::mix(m.color, (255, 255, 255), if k == 0 { 0.5 } else { 0.0 });
                draw::circle(s, col, (x as i32, y as i32), rad, 0);
            }
        }
    }

    fn draw_callouts(&mut self, s: &mut Surface) {
        let mut cls = std::mem::take(&mut self.callouts);
        for cl in cls.iter_mut() {
            let field = self.sides[cl.side].field;
            let pp = cl.t / cl.dur;
            let alpha = (255.0 * 1f64.min(cl.t / 0.08) * 1f64.min((1.0 - pp) / 0.3)) as i32;
            let total: i32 = cl.surfs.iter().map(|i| i.h()).sum();
            let mut y = field.y + (field.h as f64 * 0.3) as i32 - total / 2 - (pp * field.h as f64 * 0.05) as i32;
            let band = Rect::new(field.x + 2, y - 4, field.w - 4, total + 8);
            let key = format!("callbg{},{}", band.w, band.h);
            let mut bg = match self.caches.get(&key) {
                Some(b) => b.clone(),
                None => {
                    let mut b = pg::surface_alpha(band.w, band.h);
                    b.fill_c(rgba(6, 8, 14, 120));
                    self.caches.insert(key, b.clone());
                    b
                }
            };
            bg.set_alpha(Some(alpha.clamp(0, 255) as u8));
            s.bl(&bg, (band.x, band.y));
            for img in cl.surfs.iter_mut() {
                img.set_alpha(Some(alpha.clamp(0, 255) as u8));
                s.blr(img, img.get_rect().midtop_at((field.centerx(), y)));
                y += img.h();
            }
        }
        self.callouts = cls;
    }

    fn center_of_play(&self) -> (i32, i32) {
        if self.versus {
            return (self.b.width / 2, self.sides[0].field.centery());
        }
        self.sides[0].field.center()
    }

    fn draw_countdown(&mut self, s: &mut Surface) {
        let (cx, cy) = self.center_of_play();
        let pp = 1.0 - self.count_t / COUNT_TIME;
        let img = self.big.render(&t("tetris.ready"), true, ui::pal().text);
        let k = 1.0 + 0.06 * (pp * TAU * 2.0).sin();
        let w = (img.w() as f64 * k) as i32;
        let h = (img.h() as f64 * k) as i32;
        let img = transform::smoothscale(&img, 1.max(w), 1.max(h));
        self.badge(s, &img, cx, cy, 225);
    }

    fn draw_go(&self, s: &mut Surface) {
        let (cx, cy) = self.center_of_play();
        let pp = 1.0 - self.go_t / GO_TIME;
        let img = self.huge.render(&t("tetris.go"), true, ui::pal().green);
        let k = 0.8 + 0.5 * ease_out(pp);
        let mut img = transform::smoothscale(&img, 1.max((img.w() as f64 * k) as i32), 1.max((img.h() as f64 * k) as i32));
        img.set_alpha(Some((255.0 * 1f64.min((1.0 - pp) * 2.5)) as i32 as u8));
        s.blr(&img, img.get_rect().center_at((cx, cy)));
    }

    fn badge(&mut self, s: &mut Surface, img: &Surface, cx: i32, cy: i32, alpha: i32) {
        let bx = img.get_rect().center_at((cx, cy)).inflate(30, 14);
        self.panel(s, bx, alpha);
        s.blr(img, img.get_rect().center_at(bx.center()));
    }

    fn draw_finish_marks(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let Some(res) = self.result.as_ref() else {
            return;
        };
        let (reason, winner) = (res.reason, res.winner);
        let ft = if self.state == St::Finish { self.finish_t } else { 9.0 };
        if reason == "ko" {
            for i in 0..self.fx.len() {
                let field = self.sides[i].field;
                if self.fx[i].ko_t >= 0.0 {
                    self.stamp(s, &t("tetris.res.ko"), p.red, field, ft - 0.35);
                } else if winner == Some(i) {
                    self.stamp(s, &t("tetris.res.win"), p.gold, field, ft - 0.7);
                }
            }
        } else if (reason == "sprint" || reason == "ultra") && self.state == St::Finish {
            let field = self.sides[0].field;
            let text = if reason == "sprint" { tf("tetris.res.sprint", &[("n", &SPRINT_LINES.to_string())]) } else { t("tetris.res.time_up") };
            self.stamp(s, &text, p.gold, field, ft - 0.1);
        }
    }

    fn stamp(&mut self, s: &mut Surface, text: &str, color: Color, field: Rect, tt: f64) {
        if tt < 0.0 {
            return;
        }
        let k = 1.0 + 1.2 * (1.0 - ease_out(1f64.min(tt / 0.25)));
        let img = Self::fit_render(text, &self.f_hero, color, (field.w as f64 * 0.9) as i32);
        let w = (img.w() as f64 * k) as i32;
        let h = (img.h() as f64 * k) as i32;
        let mut img = transform::smoothscale(&img, 1.max(w), 1.max(h));
        img.set_alpha(Some((255.0 * 1f64.min(tt / 0.15)) as i32 as u8));
        let bx = img.get_rect().center_at(field.center()).inflate(20, 10);
        if tt >= 0.25 {
            self.panel(s, bx, 215);
        }
        s.blr(&img, img.get_rect().center_at(field.center()));
    }

    // ----- Ergebnis ---------------------------------------------------------

    #[allow(clippy::type_complexity)]
    fn result_content(&self) -> (String, Color, Vec<(String, String, Color)>, Option<String>, Option<Vec<(String, String, String)>>) {
        let p = ui::pal();
        let res = self.result.as_ref().expect("result");
        let st = &res.stats;
        if !self.versus {
            let s0 = &st[0];
            let mut rows = Vec::new();
            let mut badge = None;
            let (title, color);
            if res.reason == "sprint" {
                title = tf("tetris.res.sprint", &[("n", &SPRINT_LINES.to_string())]);
                color = p.gold;
                rows.push((t("tetris.stat.time"), fmt_time(res.time, true), p.text));
                if res.new_best {
                    badge = Some(t("tetris.res.new_best_time"));
                } else if let Some(pb) = res.prev_best_time {
                    rows.push((t("tetris.stat.best"), fmt_time(pb, true), p.text_dim));
                }
            } else if res.reason == "ultra" {
                title = t("tetris.res.time_up");
                color = p.gold;
                rows.push((t("tetris.stat.score"), s0.score.to_string(), p.text));
                if res.new_best {
                    badge = Some(t("tetris.res.new_best"));
                } else if let Some(pb) = res.prev_best_score {
                    rows.push((t("tetris.stat.best"), pb.to_string(), p.text_dim));
                }
            } else {
                title = t("common.game_over");
                color = p.red;
                if self.variant != "sprint" {
                    rows.push((t("tetris.stat.score"), s0.score.to_string(), p.text));
                }
                rows.push((t("tetris.stat.time"), fmt_time(res.time, true), p.text));
            }
            rows.push((t("tetris.stat.lines"), s0.lines.to_string(), p.text));
            if self.variant == "marathon" {
                rows.push((t("tetris.stat.level"), s0.level.to_string(), p.text));
            }
            rows.push((t("tetris.stat.pps"), format!("{:.2}", s0.pps), p.text));
            return (title, color, rows, badge, None);
        }
        let (title, color) = match res.winner {
            None => (t("common.draw"), p.text_dim),
            Some(w) => {
                let title = if self.b.mode == "versus_ai" {
                    if w == 0 {
                        t("tetris.res.you_win")
                    } else {
                        t("tetris.res.ai_wins")
                    }
                } else {
                    tf("common.player_wins_name", &[("name", &self.b.pname(w).chars().take(10).collect::<String>())])
                };
                (title, if w == 0 { COL_P1.c() } else { COL_P2.c() })
            }
        };
        let table = vec![
            (t("tetris.stat.lines"), st[0].lines.to_string(), st[1].lines.to_string()),
            (t("tetris.stat.attack"), st[0].attack.to_string(), st[1].attack.to_string()),
            (t("tetris.stat.pps"), format!("{:.2}", st[0].pps), format!("{:.2}", st[1].pps)),
            (t("tetris.stat.tetrises"), st[0].tetrises.to_string(), st[1].tetrises.to_string()),
            (t("tetris.stat.tspins"), st[0].tspins.to_string(), st[1].tspins.to_string()),
        ];
        (title, color, Vec::new(), Some(format!("{} : {}", self.wins[0], self.wins[1])), Some(table))
    }

    fn draw_results(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let d = self.dim(140);
        s.bl(&d, (0, 0));
        let (title, color, rows, badge, table) = self.result_content();
        let (w, h) = (self.b.width, self.b.height);
        let title_img = Self::fit_render(&title, &self.big, color, w - 70);
        let row_h = self.small.get_height() + 4;
        let btn_h = 28.max(42.min(h / 14));
        let n_rows = match &table {
            None => rows.len() as i32,
            Some(tb) => tb.len() as i32 + 1,
        };
        let ph = 18 + title_img.h() + 8 + if badge.is_some() { self.small_b.get_height() + 6 } else { 0 } + n_rows * row_h + 12 + btn_h + 8 + self.tiny.get_height() + 14;
        let pw = (w - 30).min(300.max((w as f64 * 0.46) as i32));
        let reserve = if self.show_banner() { 46 } else { 0 };
        let cy = py::fdiv32(h - reserve, 2);
        let mut panel = Rect::new(0, 0, pw, ph).center_at((w / 2, cy));
        if panel.y < 6 {
            panel.y = 6;
        }
        let bw = (pw - 3 * 14) / 2;
        let by = panel.bottom() - 14 - self.tiny.get_height() - 8 - btn_h;
        let again = Rect::new(panel.x + 14, by, bw, btn_h);
        let setup = Rect::new(again.right() + 14, by, bw, btn_h);
        ui::draw_panel(s, panel);
        draw::rect(s, self.b.accent, Rect::new(panel.x + 14, panel.y, panel.w - 28, 2), 0);
        let cx = panel.centerx();
        let mut y = panel.y + 18;
        s.blr(&title_img, title_img.get_rect().midtop_at((cx, y)));
        y += title_img.h() + 8;
        if let Some(bg) = &badge {
            let col = ui::mix(p.gold, (255, 255, 255), ui::pulse(4.0, 0.0, 0.5));
            let img = Self::fit_render(bg, &self.small_b, col, panel.w - 24);
            s.blr(&img, img.get_rect().midtop_at((cx, y)));
            y += self.small_b.get_height() + 6;
        }
        let left = panel.x + 24;
        let right = panel.right() - 24;
        match &table {
            None => {
                for (label, value, col) in &rows {
                    let li = self.small.render(label, true, p.text_dim);
                    let vi = self.small_b.render(value, true, *col);
                    s.bl(&li, (left, y));
                    s.blr(&vi, vi.get_rect().topright_at((right, y)));
                    y += row_h;
                }
            }
            Some(tb) => {
                let head = [
                    (if self.b.mode == "versus_ai" { t("tetris.player.you") } else { self.b.pname(0).chars().take(6).collect() }, COL_P1),
                    (if self.b.mode == "versus_ai" { t("common.ai") } else { self.b.pname(1).chars().take(6).collect() }, COL_P2),
                ];
                let mut colw = 0;
                for (hh, _) in &head {
                    colw = colw.max(self.small_b.size(hh).0);
                }
                for (_l, a, b) in tb {
                    colw = colw.max(self.small_b.size(a).0).max(self.small_b.size(b).0);
                }
                colw += 22;
                let c2 = panel.right() - 24;
                let c1 = c2 - colw;
                for ((text, col), xx) in head.iter().zip([c1, c2]) {
                    let img = self.small_b.render(text, true, col.c());
                    s.blr(&img, img.get_rect().topright_at((xx, y)));
                }
                y += row_h;
                for (label, v1, v2) in tb {
                    let li = Self::fit_render(label, &self.small, p.text_dim, c1 - left - self.small_b.size(v1).0 - 10);
                    s.bl(&li, (left, y));
                    for (v, xx) in [(v1, c1), (v2, c2)] {
                        let img = self.small_b.render(v, true, p.text);
                        s.blr(&img, img.get_rect().topright_at((xx, y)));
                    }
                    y += row_h;
                }
            }
        }
        let ready = Self::now() - self.over_at >= OVER_LOCK;
        self.over_rects = vec![("again", again), ("setup", setup)];
        let again_key = if self.versus { "tetris.res.rematch" } else { "tetris.res.again" };
        for (name, key, r) in [("again", again_key, again), ("setup", "tetris.res.setup", setup)] {
            ui::draw_button_ex(s, r, &t(key), &self.small, name == "again" && ready, None, None, Some(self.b.accent));
        }
        let mut hint_txt = t("tetris.res.hint");
        if self.replay.is_some() && self.rep.is_none() {
            hint_txt += &("  \u{00B7}  ".to_string() + &t("tetris.res.replay"));
        }
        let hint = Self::fit_render(&hint_txt, &self.tiny, p.text_faint, panel.w - 20);
        s.blr(&hint, hint.get_rect().midbottom_at((cx, panel.bottom() - 10)));
    }

    fn show_banner(&self) -> bool {
        self.b.mode == "solo" && self.variant == "marathon"
    }

    // ----- Setup zeichnen ------------------------------------------------

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let dc = 10.max(h / 20);
        for (kind, fx, fy, _sp, _r) in self.deco.clone() {
            let key = format!("deco{kind},{dc}");
            let ghost = match self.caches.get(&key) {
                Some(g) => g.clone(),
                None => {
                    let img = self.piece_img(kind, dc, 255);
                    let mut g = img.copy();
                    g.set_alpha(Some(34));
                    self.caches.insert(key, g.clone());
                    g
                }
            };
            s.bl(&ghost, ((fx * (w - 3 * dc) as f64) as i32, (fy * h as f64) as i32));
        }
        let title = ui::grad_text(&self.huge, "TETRIS", Some(rgb(250, 250, 255)), Some(ui::mix(self.b.accent, (255, 255, 255), 0.2)));
        s.blr(&title, title.get_rect().center_at((cx, self.setup_title_y)));
        let sub = Self::fit_render(&t(&format!("tetris.subtitle.{}", self.b.mode)), &self.small, p.text_dim, w - 30);
        s.blr(&sub, sub.get_rect().center_at((cx, self.setup_sub_y)));
        let items = self.setup_items();
        let focus = items[0.max((items.len() as i64 - 1).min(self.setup_focus)) as usize];
        let tiny = self.tiny.clone();
        let lbl = |s: &mut Surface, text: &str, x: i32, y: i32, wd: i32, active: bool| {
            let img = Self::fit_render(text, &tiny, if active { p.text } else { p.text_dim }, wd);
            s.blr(&img, img.get_rect().midbottom_at((x, y)));
        };
        if !self.setup_rects.choice.is_empty() {
            let chs = self.setup_rects.choice.clone();
            let full = chs[0].union(chs[chs.len() - 1]);
            let (names, cur, desc, best): (Vec<String>, usize, String, String);
            if self.b.mode == "solo" {
                names = VARIANTS.iter().map(|v| t(&format!("tetris.variant.{v}"))).collect();
                cur = VARIANTS.iter().position(|v| *v == self.variant).unwrap_or(0);
                lbl(s, &t("tetris.lbl.variant"), full.centerx(), chs[0].y - 3, full.w, focus == "variant");
                desc = t(&format!("tetris.variant_desc.{}", self.variant));
                best = self.best_text();
            } else {
                names = (0..3).map(|k| t(&format!("tetris.ai.{k}"))).collect();
                cur = self.ai_level as usize;
                lbl(s, &t("tetris.lbl.ai"), full.centerx(), chs[0].y - 3, full.w, focus == "ai");
                desc = t(&format!("tetris.ai_desc.{}", self.ai_level));
                let wl = self.best.vs[self.ai_level as usize];
                best = tf("tetris.vs_record", &[("w", &wl[0].to_string()), ("l", &wl[1].to_string())]);
            }
            for (k, r) in chs.iter().enumerate() {
                self.opt_button(s, *r, &names[k], k == cur, (focus == "variant" || focus == "ai") && k == cur);
            }
            let img = Self::fit_render(&desc, &self.tiny, p.text, full.w);
            s.blr(&img, img.get_rect().midtop_at((cx, self.setup_desc_y)));
            let img = Self::fit_render(&best, &self.tiny, p.gold, full.w);
            s.blr(&img, img.get_rect().midtop_at((cx, self.setup_desc_y + self.tiny.get_height() + 3)));
        }
        if let Some(r) = self.setup_rects.keys {
            self.panel(s, r, 190);
            let mut yy = r.y + 9;
            for (text, col) in [(t("tetris.keys_p1"), COL_P1), (t("tetris.keys_p2"), COL_P2)] {
                let img = Self::fit_render(&text, &self.tiny, col, r.w - 16);
                s.blr(&img, img.get_rect().midtop_at((r.centerx(), yy)));
                yy += self.tiny.get_height() + 5;
            }
        }
        let opts: Vec<(&str, Rect)> = [("level", self.setup_rects.level), ("ghost", self.setup_rects.ghost), ("das", self.setup_rects.das), ("arr", self.setup_rects.arr)].iter().filter_map(|(n, r)| r.map(|r| (*n, r))).collect();
        for &(name, r) in &opts {
            let active = focus == name;
            lbl(s, &t(&format!("tetris.lbl.{name}")), r.centerx(), r.y - 3, r.w + 6, active);
            if name == "ghost" {
                let val = if self.ghost { t("common.on") } else { t("common.off") };
                self.opt_button(s, r, &val, self.ghost, active);
            } else {
                let enabled = !(name == "level" && self.variant != "marathon");
                let val = match name {
                    "level" => {
                        if enabled {
                            self.start_level.to_string()
                        } else {
                            "1".into()
                        }
                    }
                    "das" => tf("tetris.ms", &[("n", &self.das.to_string())]),
                    _ => tf("tetris.ms", &[("n", &self.arr.to_string())]),
                };
                self.stepper(s, r, &val, active, enabled);
            }
        }
        let help_key = match focus {
            "level" => "tetris.help.level",
            "ghost" => "tetris.help.ghost",
            "arr" => "tetris.help.arr",
            _ => "tetris.help.das",
        };
        let full = opts[0].1.union(opts[opts.len() - 1].1);
        let img = Self::fit_render(&t(help_key), &self.tiny, p.text_faint, full.w);
        s.blr(&img, img.get_rect().midtop_at((cx, self.setup_help_y)));
        let start = self.setup_rects.start;
        if focus == "start" {
            let glow = ui::mix(self.b.accent, (255, 255, 255), ui::pulse(3.0, 0.0, 0.5));
            draw::rect_r(s, glow, start.inflate(8, 8), 2, 12);
        }
        ui::draw_button_ex(s, start, &t("common.start"), &self.small_b, true, None, None, Some(self.b.accent));
        let (y1, y2) = self.setup_footer_y;
        let img = Self::fit_render(&t("tetris.setup_hint"), &self.tiny, p.text_faint, w - 20);
        s.blr(&img, img.get_rect().midtop_at((cx, y1)));
        if self.b.mode != "multi" {
            let img = Self::fit_render(&t("tetris.keys_solo"), &self.tiny, ui::mix(self.b.accent, p.text, 0.45), w - 20);
            s.blr(&img, img.get_rect().midtop_at((cx, y2)));
        }
    }

    fn best_text(&self) -> String {
        if self.variant == "sprint" {
            return match self.best.sprint {
                None => t("tetris.best_none"),
                Some(sp) => tf("tetris.best_time", &[("time", &fmt_time(sp, true))]),
            };
        }
        if self.variant == "ultra" {
            if self.best.ultra == 0 {
                return t("tetris.best_none");
            }
            return tf("tetris.best_score", &[("score", &self.best.ultra.to_string())]);
        }
        let hs = highscore::get(self.b.key);
        if hs != 0 {
            tf("tetris.best_score", &[("score", &hs.to_string())])
        } else {
            t("tetris.best_none")
        }
    }

    fn opt_button(&self, s: &mut Surface, r: Rect, text: &str, on: bool, focus: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, r, 0, 8);
        let border = if on || focus { self.b.accent } else { p.border };
        draw::rect_r(s, border, r, if focus { 2 } else { 1 }, 8);
        if focus {
            let glow = ui::mix(self.b.accent, (255, 255, 255), ui::pulse(3.0, 0.0, 0.4));
            draw::rect_r(s, glow, r.inflate(4, 4), 1, 10);
        }
        let img = Self::fit_render(text, if on { &self.small_b } else { &self.small }, if on { p.text } else { p.text_dim }, r.w - 10);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn stepper(&self, s: &mut Surface, r: Rect, text: &str, focus: bool, enabled: bool) {
        let p = ui::pal();
        draw::rect_r(s, p.btn, r, 0, 8);
        draw::rect_r(s, if focus { self.b.accent } else { p.border }, r, if focus { 2 } else { 1 }, 8);
        let col = if enabled { p.text } else { p.text_faint };
        let aw = 3.max(r.h / 8);
        let arrow = if focus && enabled { self.b.accent } else { p.text_dim };
        for sx in [-1, 1] {
            let ax = r.centerx() + sx * (r.w / 2 - aw - 5);
            let tip = ax + py::fdiv32(sx * aw, 2);
            let back = ax - py::fdiv32(sx * aw, 2);
            draw::polygon(s, arrow, &[(tip, r.centery()), (back, r.centery() - aw), (back, r.centery() + aw)], 0);
        }
        let img = Self::fit_render(text, if focus { &self.small_b } else { &self.small }, col, r.w - 4 * aw - 22);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }
}

impl Game for TetrisGame {
    fn replay_begin(&mut self, rep: &Value) -> bool {
        TetrisGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        TetrisGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        let _ = aiming;
        TetrisGame::replay_draw(self, banner);
    }

    fn replay_hud_h(&self) -> i32 {
        40
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
        if !["solo", "versus_ai", "multi"].contains(&self.b.mode.as_str()) {
            self.b.mode = "solo".into();
        }
        self.versus = self.b.mode != "solo";
        self.load_options();
        self.make_fonts();
        self.anim_t = 0.0;
        self.wins = [0, 0];
        self.best = Self::load_best();
        self.particles.clear();
        self.callouts.clear();
        self.missiles.clear();
        self.sprites.clear();
        self.pieces.clear();
        self.caches.clear();
        self.stack_cache.clear();
        self.boards.clear();
        self.pads.clear();
        self.fx.clear();
        self.ai = None;
        self.result = None;
        self.elapsed = 0.0;
        self.go_t = 0.0;
        self.over_at = 0.0;
        self.prev_four = false;
        self.setup_focus = 0;
        self.over_rects.clear();
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.rep_at = None;
        self.rec_delta = replay::Delta::new();
        self.rec_n = 0;
        self.state = St::Setup;
        self.deco = (0..9).map(|_| Self::new_deco(true)).collect();
        self.layout();
        self.build_setup_layout();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.sprites.clear();
        self.pieces.clear();
        self.caches.clear();
        self.stack_cache.clear();
        self.layout();
        self.build_setup_layout();
    }

    fn show_highscore_banner(&self) -> bool {
        self.show_banner()
    }

    fn take_replay_request(&mut self) -> Option<Value> {
        self.replay_request.take()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == St::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.state == St::Over {
            self.handle_over(ev);
            return;
        }
        if ev.kind == Kind::KeyUp {
            for pad in self.pads.iter_mut() {
                pad.release(&ev.key);
            }
            return;
        }
        if ev.kind != Kind::KeyDown || ev.repeat {
            return;
        }
        if !matches!(self.state, St::Count | St::Play) {
            return;
        }
        let key = ev.key.clone();
        if self.b.mode == "solo" && self.variant != "marathon" && (key == "r" || key == "R") && self.b.key_is_free(&key) {
            self.start();
            return;
        }
        let live = self.state == St::Play;
        for (slot, player, extra) in self.human_slots() {
            self.play_key(slot, player, extra, &key, live);
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_effects(dt);
        match self.state {
            St::Setup => {
                self.update_deco(dt);
                return;
            }
            St::Count => {
                self.count_t -= dt;
                if self.count_t <= 0.0 {
                    self.state = St::Play;
                    self.go_t = GO_TIME;
                    self.b.play_sound("powerup");
                }
                return;
            }
            St::Finish => {
                self.finish_t += dt;
                for f in self.fx.iter_mut() {
                    if f.ko_t >= 0.0 {
                        f.ko_t += dt;
                    }
                }
                if self.finish_t >= self.finish_dur {
                    self.state = St::Over;
                    self.b.game_over = true;
                    self.over_at = Self::now();
                }
                return;
            }
            St::Over => return,
            St::Play => {}
        }
        self.elapsed += dt;
        self.go_t = 0f64.max(self.go_t - dt);
        if self.versus {
            let lvl = VS_LEVEL_MAX.min(1 + py::ffdiv(self.elapsed, VS_LEVEL_EVERY) as i64);
            if lvl != self.boards[0].level {
                for i in 0..self.boards.len() {
                    self.boards[i].level = lvl;
                    self.callout(i, vec![(tf("tetris.call.level", &[("n", &lvl.to_string())]), ui::pal().gold)], vec![]);
                    self.fx[i].level_t = 0.0;
                }
                self.b.play_sound("level");
            }
        }
        let humans: Vec<usize> = self.human_slots().iter().map(|h| h.0).collect();
        for i in 0..self.boards.len() {
            if humans.contains(&i) {
                self.das_step(i, dt);
            }
            let soft = !self.pads[i].down.is_empty() && humans.contains(&i);
            self.boards[i].tick(dt, soft);
        }
        if let Some(a) = self.ai.as_mut() {
            a.update(dt, &mut self.boards[1]);
        }
        for i in 0..self.boards.len() {
            self.process_events(i);
        }
        self.check_end();
        if self.b.mode == "solo" && self.variant == "marathon" {
            self.b.score = self.boards[0].score;
        }
        self.rec_tick(dt);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            self.draw_particles(&mut s);
            return;
        }
        for i in 0..self.boards.len() {
            self.draw_side(&mut s, i);
        }
        if self.versus {
            self.draw_vs_hud(&mut s);
        }
        self.draw_missiles(&mut s);
        self.draw_particles(&mut s);
        self.draw_callouts(&mut s);
        if self.state == St::Count {
            self.draw_countdown(&mut s);
        } else if self.state == St::Play && self.go_t > 0.0 {
            self.draw_go(&mut s);
        } else if matches!(self.state, St::Finish | St::Over) {
            self.draw_finish_marks(&mut s);
        }
        if self.state == St::Over {
            self.draw_results(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "tetris",
    class_name: "TetrisGame",
    supports_multiplayer: true,
    modes: &[("solo", "tetris.mode.solo"), ("versus_ai", "tetris.mode.versus_ai"), ("multi", "tetris.mode.multi")],
    make: TetrisGame::new,
};
