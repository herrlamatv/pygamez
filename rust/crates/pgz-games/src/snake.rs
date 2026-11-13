//! snake.rs
//! ========
//! Snake - Deluxe-Version mit Spielmodi, Boost, 3D-Ansicht und Competitive.
//!
//! Uebertragen aus `games/snake.py`. Hilfsmodule: [`crate::prestige`],
//! [`crate::competitive`], [`crate::ngb`]. Mengen von Zellen (Aepfel,
//! Hindernisse) laufen ueber `py::CellSet`, damit sie in Pythons
//! Reihenfolge gezeichnet werden.

use std::collections::HashSet;
use std::f64::consts::{PI, TAU};

use pgz_core::{highscore, i18n, replay};
use pgz_engine::prelude::*;
use pgz_engine::py::CellSet;
use serde_json::{json, Map, Value};

use crate::{competitive, ngb, prestige};

type Rgb = (i32, i32, i32);
type Cell = (i32, i32);

const CELL: i32 = 20;
const BASE_INTERVAL: f64 = 0.12;
const MIN_INTERVAL: f64 = 0.055;
const MIN_LENGTH: i64 = 3;
const STAMINA_MAX: f64 = 1.0;
const STAMINA_REGEN: f64 = 0.22;
const BOOST_DRAIN: f64 = 0.045;
const BOOST_MIN_START: f64 = 0.15;
const HARDCORE_BOOST_LEN_COST: i64 = 1;
const BOOST_KEYS_P1: [&str; 2] = ["space", "Shift_L"];
const BOOST_KEYS_P2: [&str; 3] = ["Return", "Shift_R", "KP_Enter"];
const GOLDEN_LIFETIME: f64 = 6.0;
const GOLDEN_CHANCE: f64 = 0.20;
const TIMED_SECONDS: f64 = 60.0;
const SPECIAL_LIFETIME: f64 = 8.0;
const BLUE_CHANCE: f64 = 0.12;
const PURPLE_CHANCE: f64 = 0.16;
const SPAWN_BONUS_TIME: f64 = 10.0;
const SLOT_REEL_STOPS: [f64; 3] = [0.9, 1.35, 1.8];
const SLOT_SHOW: f64 = 1.7;
const SLOT_SPIN_SPEED: f64 = 16.0;
const BANNER_TIME: f64 = 2.0;
const APPLE_COUNTS: [i64; 4] = [1, 2, 3, 5];
const FOOD_SPAWN_ANIM: f64 = 0.28;
const FOOD_EAT_ANIM: f64 = 0.30;

fn ease_out_back(p: f64) -> f64 {
    if p >= 1.0 {
        return 1.0;
    }
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * py::pow(p - 1.0, 3.0) + c1 * py::pow(p - 1.0, 2.0)
}

#[derive(Clone, Copy, PartialEq)]
enum St {
    Setup,
    Play,
    Personalize,
    Cam3d,
}

const COL_BG: Rgb = (15, 15, 25);
const COL_GRID: Rgb = (25, 25, 40);
const COL_FOOD: Rgb = (240, 90, 90);
const COL_GOLD: Rgb = (255, 205, 70);
const COL_WALL: Rgb = (70, 78, 98);
const COL_BLUE: Rgb = (90, 150, 245);
const COL_PURPLE: Rgb = (185, 110, 240);
const COL_HARDCORE: Rgb = (235, 45, 55);
const SNAKE_COLORS: [(Rgb, Rgb); 2] = [((80, 220, 120), (150, 255, 180)), ((90, 160, 240), (160, 205, 255))];
const BOOST_GLOW: [Rgb; 2] = [(150, 255, 190), (170, 210, 255)];
const PORTAL_COLORS: [Rgb; 3] = [(255, 140, 60), (180, 120, 255), (90, 220, 220)];
const MODE_KEYS: [&str; 6] = ["classic", "speed", "walls", "portal", "timed", "competitive"];
const MODES_3D: [&str; 2] = ["classic", "walls"];

const NEAR: f64 = 0.12;
const FOG_START: f64 = 7.0;
const FOG_END: f64 = 24.0;
const FOV_NORMAL: f64 = 1.12;
const FOV_BOOST: f64 = 0.94;
const CAM_BACK: f64 = 3.6;
const CAM_H: f64 = 2.6;
const CAM_AHEAD: f64 = 2.6;
const CAM_LOOK_H: f64 = 0.35;
const CAM_SMOOTH: f64 = 6.0;
const FOV_BOOST_DELTA: f64 = FOV_NORMAL - FOV_BOOST;
const CAM_FOV_MIN: f64 = 0.80;
const CAM_FOV_MAX: f64 = 1.50;
const CAM_FOV_STEP: f64 = 0.02;
const CAM_H_MIN: f64 = 1.6;
const CAM_H_MAX: f64 = 4.2;
const CAM_H_STEP: f64 = 0.1;
const COL_SKY_TOP: Rgb = (8, 10, 22);
const COL_SKY_HOR: Rgb = (46, 52, 86);
const COL_FOG: Rgb = (34, 39, 66);
const COL_TILE_A: Rgb = (26, 29, 48);
const COL_TILE_B: Rgb = (32, 36, 58);
const COL_BORDER: Rgb = (84, 96, 138);

const REC_CHAPTER: usize = 300;

fn rotate_dir(d: Cell, turn: char) -> Cell {
    let (dx, dy) = d;
    if turn == 'L' {
        (dy, -dx)
    } else {
        (-dy, dx)
    }
}

/// `f"{x:g}"` fuer die kleinen Faktoren (hoechstens zwei Nachkommastellen).
fn fmt_g(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        return format!("{}", x as i64);
    }
    format!("{x}")
}

#[derive(Clone)]
struct Snk {
    body: Vec<Cell>,
    direction: Cell,
    next_direction: Cell,
    alive: bool,
    score: i64,
    apples: i64,
    grow: i64,
    stamina: f64,
    boost_on: bool,
    size_frac: f64,
    prev_body: Vec<Cell>,
    turn_queue: Vec<char>,
}

impl Snk {
    fn new(body: Vec<Cell>, direction: Cell) -> Snk {
        Snk { prev_body: body.clone(), body, direction, next_direction: direction, alive: true, score: 0, apples: 0, grow: 0, stamina: STAMINA_MAX, boost_on: false, size_frac: 0.0, turn_queue: Vec::new() }
    }
}

#[derive(Clone)]
struct Special {
    blue: bool,
    timer: f64,
}

struct Slot {
    snake: usize,
    reels: [&'static str; 3],
    mult: f64,
    result: &'static str,
    stake: i64,
    t: f64,
    applied: bool,
    snd: [bool; 3],
}

struct FloatText {
    x: f64,
    y: f64,
    text: String,
    color: Color,
    t: f64,
}

struct BannerInfo {
    text: String,
    color: Color,
    sub: Option<String>,
    t: f64,
}

type Item3 = (f64, Vec<(f64, f64)>, Rgb, Option<Rgb>);

/// `SnakeGame`
pub struct SnakeGame {
    b: GameBase,
    winner: Option<usize>,
    effects_done: bool,
    cols: i32,
    rows: i32,
    wrap: bool,
    bonus: bool,
    hardcore: bool,
    apple_count: i64,
    mode_index: usize,
    view3d: bool,
    cam_smooth: bool,
    cam_fov: f64,
    cam_height: f64,
    cam_turn_shake: bool,
    small: Font,
    tiny: Font,
    highscore: i64,
    rec: Option<replay::Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_at: Option<(usize, usize)>,
    rep_state: Vec<i64>,
    rec_delta: replay::Delta,
    rec_moves: Vec<[i64; 5]>,
    rec_last: Vec<(usize, Cell, usize)>,
    rec_layout: usize,
    rec_n: usize,
    particles: Vec<(f64, f64, f64, f64, f64, Rgb)>,
    anim_t: f64,
    ngb_menu: Option<ngb::PersonalizeMenu>,
    grid_surf: Option<Surface>,
    grid_key: Option<(i32, i32, Vec<Rgb>)>,
    grid_font: Font,
    particles3d: Vec<[f64; 7]>,
    particles3d_col: Vec<Rgb>,
    shake: f64,
    sky_cache: Option<((i32, i32), Surface)>,
    stars: Vec<(f64, f64, i32, f64)>,
    state: St,
    // Lauf
    apples_total: i64,
    apples_bank: i64,
    prestige: i64,
    speed_apples: i64,
    comp_level: i64,
    spawn_bonus: i64,
    spawn_bonus_t: f64,
    snakes: Vec<Snk>,
    timer: f64,
    interval: f64,
    obstacles: CellSet,
    portals: Vec<(Cell, Cell)>,
    portal_pairs: Vec<(Cell, Cell, Rgb)>,
    foods: CellSet,
    food_anim: Vec<(Cell, f64)>,
    eat_fx: Vec<(Cell, f64)>,
    golden: Option<Cell>,
    golden_timer: f64,
    time_left: f64,
    specials: Vec<(Cell, Special)>,
    slot: Option<Slot>,
    float_texts: Vec<FloatText>,
    banner: Option<BannerInfo>,
    purple_pending: Option<usize>,
    slot_pending: Option<usize>,
    // Kamera
    run_t: f64,
    orbit_a: f64,
    go_last: Option<f64>,
    fov_mul: f64,
    cam_dir: (f64, f64),
    cam_pos: (f64, f64, f64),
    cam_look: (f64, f64, f64),
    scx: f64,
    scy: f64,
    f: f64,
    basis: ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64)),
    hud_h: i32,
    // Setup-Layout
    brush_rect: Rect,
    mode_panel: Rect,
    mode_left: Rect,
    mode_right: Rect,
    view_rect: Rect,
    wrap_rect: Rect,
    bonus_rect: Rect,
    apples_rect: Rect,
    start_rect: Rect,
    cam_smooth_rect: Rect,
    cam_fov_rect: Rect,
    cam_height_rect: Rect,
    cam_turn_rect: Rect,
    cam_back_rect: Rect,
    cam_fov_minus: Rect,
    cam_fov_plus: Rect,
    cam_height_minus: Rect,
    cam_height_plus: Rect,
}

impl SnakeGame {
    /// `SnakeGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = SnakeGame {
            b,
            winner: None,
            effects_done: false,
            cols: 1,
            rows: 1,
            wrap: false,
            bonus: false,
            hardcore: false,
            apple_count: 1,
            mode_index: 0,
            view3d: false,
            cam_smooth: true,
            cam_fov: FOV_NORMAL,
            cam_height: CAM_H,
            cam_turn_shake: false,
            small: f.clone(),
            tiny: f.clone(),
            highscore: 0,
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_at: None,
            rep_state: Vec::new(),
            rec_delta: replay::Delta::new(),
            rec_moves: Vec::new(),
            rec_last: Vec::new(),
            rec_layout: 0,
            rec_n: 0,
            particles: Vec::new(),
            anim_t: 0.0,
            ngb_menu: None,
            grid_surf: None,
            grid_key: None,
            grid_font: f,
            particles3d: Vec::new(),
            particles3d_col: Vec::new(),
            shake: 0.0,
            sky_cache: None,
            stars: Vec::new(),
            state: St::Setup,
            apples_total: 0,
            apples_bank: 0,
            prestige: 0,
            speed_apples: 0,
            comp_level: 0,
            spawn_bonus: 0,
            spawn_bonus_t: 0.0,
            snakes: Vec::new(),
            timer: 0.0,
            interval: BASE_INTERVAL,
            obstacles: CellSet::cells(),
            portals: Vec::new(),
            portal_pairs: Vec::new(),
            foods: CellSet::cells(),
            food_anim: Vec::new(),
            eat_fx: Vec::new(),
            golden: None,
            golden_timer: 0.0,
            time_left: TIMED_SECONDS,
            specials: Vec::new(),
            slot: None,
            float_texts: Vec::new(),
            banner: None,
            purple_pending: None,
            slot_pending: None,
            run_t: 0.0,
            orbit_a: 0.0,
            go_last: None,
            fov_mul: FOV_NORMAL,
            cam_dir: (1.0, 0.0),
            cam_pos: (0.0, 0.0, 0.0),
            cam_look: (0.0, 0.0, 0.0),
            scx: 0.0,
            scy: 0.0,
            f: 1.0,
            basis: ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)),
            hud_h: 0,
            brush_rect: z,
            mode_panel: z,
            mode_left: z,
            mode_right: z,
            view_rect: z,
            wrap_rect: z,
            bonus_rect: z,
            apples_rect: z,
            start_rect: z,
            cam_smooth_rect: z,
            cam_fov_rect: z,
            cam_height_rect: z,
            cam_turn_rect: z,
            cam_back_rect: z,
            cam_fov_minus: z,
            cam_fov_plus: z,
            cam_height_minus: z,
            cam_height_plus: z,
        };
        g.reset();
        Box::new(g)
    }

    // ----- Einstellungen / Eigenschaften -----------------------------

    fn opt_truthy(&self, key: &str, default: bool) -> bool {
        match self.b.opt_value("snake", key) {
            None => default,
            Some(Value::Null) => false,
            Some(Value::Bool(v)) => v,
            Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
        }
    }

    fn mode_key(&self) -> &'static str {
        MODE_KEYS[self.mode_index]
    }

    fn view3d_active(&self) -> bool {
        self.view3d && !self.b.multiplayer
    }

    fn competitive(&self) -> bool {
        self.mode_key() == "competitive" && !self.b.multiplayer
    }

    fn hardcore_active(&self) -> bool {
        self.competitive() && self.hardcore
    }

    fn wrap_active(&self) -> bool {
        self.wrap && !self.view3d_active()
    }

    fn allowed_modes(&self) -> Vec<usize> {
        if self.view3d_active() {
            return (0..MODE_KEYS.len()).filter(|i| MODES_3D.contains(&MODE_KEYS[*i])).collect();
        }
        if self.b.multiplayer {
            return (0..MODE_KEYS.len()).filter(|i| MODE_KEYS[*i] != "competitive").collect();
        }
        (0..MODE_KEYS.len()).collect()
    }

    fn mode_idx(key: &str) -> usize {
        MODE_KEYS.iter().position(|k| *k == key).unwrap_or(0)
    }

    fn reset_run_stats(&mut self) {
        self.apples_total = 0;
        self.apples_bank = 0;
        self.prestige = 0;
        self.speed_apples = 0;
        self.comp_level = 0;
        self.spawn_bonus = 0;
        self.spawn_bonus_t = 0.0;
    }

    fn new_board(&mut self) {
        self.cols = self.b.width / CELL;
        self.rows = self.b.height / CELL;
        let cy = self.rows / 2;
        self.snakes.clear();
        if self.b.multiplayer {
            self.snakes.push(Snk::new(vec![(2, cy - 3), (3, cy - 3), (4, cy - 3)], (1, 0)));
            self.snakes.push(Snk::new(vec![(2, cy + 3), (3, cy + 3), (4, cy + 3)], (1, 0)));
        } else {
            let cx = self.cols / 2;
            self.snakes.push(Snk::new(vec![(cx - 2, cy), (cx - 1, cy), (cx, cy)], (1, 0)));
        }
        self.timer = 0.0;
        self.interval = BASE_INTERVAL;
        self.build_mode_layout();
        self.place_food();
        self.reset_camera();
    }

    fn reset_camera(&mut self) {
        self.run_t = 0.0;
        self.orbit_a = 0.0;
        self.go_last = None;
        self.fov_mul = self.cam_fov;
        let sn = &self.snakes[0];
        let (hx, hy) = *sn.body.last().unwrap_or(&(0, 0));
        let (fx, fz) = (sn.direction.0 as f64, sn.direction.1 as f64);
        self.cam_dir = (fx, fz);
        let head = (hx as f64 + 0.5, 0.0, hy as f64 + 0.5);
        self.cam_pos = (head.0 - fx * CAM_BACK, self.cam_height, head.2 - fz * CAM_BACK);
        self.cam_look = (head.0 + fx * CAM_AHEAD, CAM_LOOK_H, head.2 + fz * CAM_AHEAD);
    }

    fn start_play(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.winner = None;
        self.effects_done = false;
        self.b.new_round_result();
        self.particles.clear();
        self.particles3d.clear();
        self.particles3d_col.clear();
        self.shake = 0.0;
        self.reset_run_stats();
        self.new_board();
        self.rec_new();
        self.state = St::Play;
    }

    // ----- Modus-Layout ------------------------------------------------

    fn build_mode_layout(&mut self) {
        self.obstacles = CellSet::cells();
        self.portals.clear();
        self.portal_pairs.clear();
        self.foods = CellSet::cells();
        self.food_anim.clear();
        self.eat_fx.clear();
        self.golden = None;
        self.golden_timer = 0.0;
        self.time_left = TIMED_SECONDS;
        self.specials.clear();
        self.slot = None;
        self.float_texts.clear();
        self.banner = None;
        self.purple_pending = None;
        self.slot_pending = None;
        let mut tabu: HashSet<Cell> = HashSet::new();
        for sn in &self.snakes {
            for &(x, y) in &sn.body {
                for ddx in -1..8 {
                    tabu.insert((x + ddx, y));
                }
                tabu.insert((x, y - 1));
                tabu.insert((x, y + 1));
            }
        }
        if self.mode_key() == "walls" {
            self.make_obstacles(&tabu);
        } else if self.mode_key() == "portal" {
            self.make_portals(&mut tabu);
        }
    }

    fn make_obstacles(&mut self, tabu: &HashSet<Cell>) {
        let anzahl = (self.cols as f64 * self.rows as f64 * 0.05) as usize;
        let mut versuche = 0;
        while self.obstacles.len() < anzahl && versuche < anzahl * 30 {
            versuche += 1;
            let bx = py::randint(1, (self.cols - 2) as i64) as i32;
            let by = py::randint(2, (self.rows - 2) as i64) as i32;
            let mut cluster = vec![(bx, by)];
            if py::random() < 0.5 {
                cluster.push((bx + py::choice(&[-1, 1]), by));
            }
            if py::random() < 0.4 {
                cluster.push((bx, by + py::choice(&[-1, 1])));
            }
            if cluster.iter().all(|c| !tabu.contains(c) && 0 < c.0 && c.0 < self.cols - 1 && 1 < c.1 && c.1 < self.rows - 1) {
                for c in cluster {
                    self.obstacles.add(c);
                }
            }
        }
    }

    fn portal_of(&self, c: Cell) -> Option<Cell> {
        self.portals.iter().find(|(a, _)| *a == c).map(|(_, b)| *b)
    }

    fn make_portals(&mut self, tabu: &mut HashSet<Cell>) {
        let paare = if self.cols * self.rows > 500 { 2 } else { 1 };
        for k in 0..paare {
            let mut enden: Vec<Cell> = Vec::new();
            let mut versuche = 0;
            while enden.len() < 2 && versuche < 300 {
                versuche += 1;
                let x = py::randint(1, (self.cols - 2) as i64) as i32;
                let y = py::randint(2, (self.rows - 2) as i64) as i32;
                let p = (x, y);
                if tabu.contains(&p) || self.portal_of(p).is_some() || enden.contains(&p) {
                    continue;
                }
                if !enden.is_empty() && (enden[0].0 - p.0).abs() + (enden[0].1 - p.1).abs() < 6 {
                    continue;
                }
                enden.push(p);
                tabu.insert(p);
            }
            if enden.len() == 2 {
                let (a, b) = (enden[0], enden[1]);
                self.portals.push((a, b));
                self.portals.push((b, a));
                self.portal_pairs.push((a, b, PORTAL_COLORS[k % PORTAL_COLORS.len()]));
            }
        }
    }

    fn blocked_cells(&self) -> HashSet<Cell> {
        let mut belegt: HashSet<Cell> = self.obstacles.iter().collect();
        for (a, _) in &self.portals {
            belegt.insert(*a);
        }
        for sn in &self.snakes {
            belegt.extend(sn.body.iter().copied());
        }
        belegt.extend(self.foods.iter());
        belegt.extend(self.specials.iter().map(|(c, _)| *c));
        if let Some(g) = self.golden {
            belegt.insert(g);
        }
        belegt
    }

    fn apple_target(&self) -> i64 {
        if self.competitive() {
            let base = competitive::apples_on_field(self.comp_level);
            let bonus = if self.spawn_bonus_t > 0.0 { self.spawn_bonus } else { 0 };
            return (competitive::max_apples() + 5).min(base + bonus);
        }
        self.apple_count
    }

    fn free_cells(&self) -> Vec<Cell> {
        let belegt = self.blocked_cells();
        let mut frei = Vec::new();
        for x in 0..self.cols {
            for y in 0..self.rows {
                if !belegt.contains(&(x, y)) {
                    frei.push((x, y));
                }
            }
        }
        frei
    }

    fn set_food_anim(&mut self, cell: Cell, v: f64) {
        match self.food_anim.iter_mut().find(|(c, _)| *c == cell) {
            Some(e) => e.1 = v,
            None => self.food_anim.push((cell, v)),
        }
    }

    fn place_food(&mut self) {
        let mut frei = self.free_cells();
        py::shuffle(&mut frei);
        while (self.foods.len() as i64) < self.apple_target() {
            let Some(cell) = frei.pop() else {
                break;
            };
            self.foods.add(cell);
            self.set_food_anim(cell, 0.0);
        }
    }

    fn maybe_spawn_special(&mut self) {
        if !self.specials.is_empty() {
            return;
        }
        let r = py::random();
        if r < BLUE_CHANCE {
            self.place_special(true);
        } else if r < BLUE_CHANCE + PURPLE_CHANCE {
            self.place_special(false);
        }
    }

    fn place_special(&mut self, blue: bool) {
        let frei = self.free_cells();
        if !frei.is_empty() {
            let c = py::choice(&frei);
            self.specials.push((c, Special { blue, timer: SPECIAL_LIFETIME }));
        }
    }

    fn place_golden(&mut self) {
        let frei = self.free_cells();
        if !frei.is_empty() {
            self.golden = Some(py::choice(&frei));
            self.golden_timer = GOLDEN_LIFETIME;
        }
    }

    // ===================================================== Setup-Screen

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 420.min(self.b.width - 60);
        self.brush_rect = Rect::new(self.b.width - 42, 8, 34, 34);
        self.mode_panel = Rect::new(cx - bw / 2, 106, bw, 58);
        self.mode_left = Rect::new(self.mode_panel.x, 106, 40, 58);
        self.mode_right = Rect::new(self.mode_panel.right() - 40, 106, 40, 58);
        let (bh, gap) = (40, 6);
        let y0 = 180;
        self.view_rect = Rect::new(cx - bw / 2, y0, bw, bh);
        self.wrap_rect = Rect::new(cx - bw / 2, y0 + (bh + gap), bw, bh);
        self.bonus_rect = Rect::new(cx - bw / 2, y0 + 2 * (bh + gap), bw, bh);
        self.apples_rect = Rect::new(cx - bw / 2, y0 + 3 * (bh + gap), bw, bh);
        self.start_rect = Rect::new(cx - 95, y0 + 4 * (bh + gap) + 4, 190, 48);
        let (cbw, cbh, cgap, cy0) = (440.min(self.b.width - 50), 46, 10, 150);
        self.cam_smooth_rect = Rect::new(cx - cbw / 2, cy0, cbw, cbh);
        self.cam_fov_rect = Rect::new(cx - cbw / 2, cy0 + (cbh + cgap), cbw, cbh);
        self.cam_height_rect = Rect::new(cx - cbw / 2, cy0 + 2 * (cbh + cgap), cbw, cbh);
        self.cam_turn_rect = Rect::new(cx - cbw / 2, cy0 + 3 * (cbh + cgap), cbw, cbh);
        self.cam_back_rect = Rect::new(cx - 95, cy0 + 4 * (cbh + cgap) + 8, 190, 46);
        let pm = |r: Rect| (Rect::new(r.right() - 148, r.centery() - 16, 32, 32), Rect::new(r.right() - 46, r.centery() - 16, 32, 32));
        let (a, b) = pm(self.cam_fov_rect);
        self.cam_fov_minus = a;
        self.cam_fov_plus = b;
        let (a, b) = pm(self.cam_height_rect);
        self.cam_height_minus = a;
        self.cam_height_plus = b;
    }

    fn save_snake_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("snake", key, value);
    }

    fn toggle_setting(&mut self, wrap: bool) {
        if wrap && self.view3d_active() {
            return;
        }
        let neu = if wrap { !self.wrap } else { !self.bonus };
        if wrap {
            self.wrap = neu;
        } else {
            self.bonus = neu;
        }
        self.save_snake_setting(if wrap { "wrap" } else { "bonus_apple" }, json!(neu));
        self.b.play_sound("select");
    }

    fn toggle_hardcore(&mut self) {
        if !self.competitive() {
            return;
        }
        self.hardcore = !self.hardcore;
        let v = self.hardcore;
        self.save_snake_setting("hardcore", json!(v));
        self.b.play_sound("select");
    }

    fn cycle_apples(&mut self) {
        if self.competitive() {
            return;
        }
        let i = APPLE_COUNTS.iter().position(|c| *c == self.apple_count).unwrap_or(0);
        self.apple_count = APPLE_COUNTS[(i + 1) % APPLE_COUNTS.len()];
        let v = self.apple_count;
        self.save_snake_setting("apples", json!(v));
        self.b.play_sound("select");
    }

    fn cycle_mode(&mut self, step: i64) {
        let allowed = self.allowed_modes();
        if let Some(i) = allowed.iter().position(|m| *m == self.mode_index) {
            self.mode_index = allowed[py::pmod(i as i64 + step, allowed.len() as i64) as usize];
        } else {
            self.mode_index = allowed[0];
        }
        let k = self.mode_key();
        self.save_snake_setting("mode", json!(k));
        self.b.play_sound("click");
    }

    fn toggle_view(&mut self) {
        if self.b.multiplayer {
            return;
        }
        self.view3d = !self.view3d;
        if self.view3d && !MODES_3D.contains(&self.mode_key()) {
            self.mode_index = Self::mode_idx("classic");
            let k = self.mode_key();
            self.save_snake_setting("mode", json!(k));
        }
        let v = self.view3d;
        self.save_snake_setting("view3d", json!(v));
        self.b.play_sound("select");
    }

    fn open_personalize(&mut self) {
        self.ngb_menu = Some(ngb::open_head_color_menu(self.b.width, self.b.height));
        self.state = St::Personalize;
        self.b.play_sound("click");
    }

    fn open_cam3d(&mut self) {
        self.state = St::Cam3d;
        self.b.play_sound("click");
    }

    fn adjust_fov(&mut self, d: f64) {
        self.cam_fov = py::round_to(CAM_FOV_MAX.min(CAM_FOV_MIN.max(self.cam_fov + d)), 2);
        let v = self.cam_fov;
        self.save_snake_setting("cam_fov", json!(v));
        self.b.play_sound("click");
    }

    fn adjust_height(&mut self, d: f64) {
        self.cam_height = py::round_to(CAM_H_MAX.min(CAM_H_MIN.max(self.cam_height + d)), 2);
        let v = self.cam_height;
        self.save_snake_setting("cam_height", json!(v));
        self.b.play_sound("click");
    }

    fn handle_cam3d_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if ["Escape", "Return", "space", "k", "K"].contains(&k) {
                self.state = St::Setup;
                self.b.play_sound("click");
            } else if ["Left", "a", "A"].contains(&k) {
                self.adjust_fov(-CAM_FOV_STEP);
            } else if ["Right", "d", "D"].contains(&k) {
                self.adjust_fov(CAM_FOV_STEP);
            } else if ["Up", "w", "W"].contains(&k) {
                self.adjust_height(CAM_H_STEP);
            } else if ["Down", "s", "S"].contains(&k) {
                self.adjust_height(-CAM_H_STEP);
            }
            return;
        }
        if ev.kind != Kind::MouseDown {
            return;
        }
        let p = ev.pos;
        if self.cam_smooth_rect.hit(p) {
            self.cam_smooth = !self.cam_smooth;
            let v = self.cam_smooth;
            self.save_snake_setting("cam_smooth", json!(v));
            self.b.play_sound("select");
        } else if self.cam_turn_rect.hit(p) {
            self.cam_turn_shake = !self.cam_turn_shake;
            let v = self.cam_turn_shake;
            self.save_snake_setting("cam_turn_shake", json!(v));
            self.b.play_sound("select");
        } else if self.cam_fov_minus.hit(p) {
            self.adjust_fov(-CAM_FOV_STEP);
        } else if self.cam_fov_plus.hit(p) {
            self.adjust_fov(CAM_FOV_STEP);
        } else if self.cam_height_minus.hit(p) {
            self.adjust_height(-CAM_H_STEP);
        } else if self.cam_height_plus.hit(p) {
            self.adjust_height(CAM_H_STEP);
        } else if self.cam_back_rect.hit(p) {
            self.state = St::Setup;
            self.b.play_sound("click");
        }
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if ["Left", "a", "A"].contains(&k) {
                self.cycle_mode(-1);
            } else if ["Right", "d", "D"].contains(&k) {
                self.cycle_mode(1);
            } else if ["1", "2", "3", "4", "5", "6"].contains(&k) {
                let allowed = self.allowed_modes();
                let idx = k.parse::<usize>().unwrap_or(1) - 1;
                if idx < allowed.len() {
                    self.mode_index = allowed[idx];
                    let mk = self.mode_key();
                    self.save_snake_setting("mode", json!(mk));
                    self.b.play_sound("click");
                }
            } else if k == "v" || k == "V" {
                self.toggle_view();
            } else if k == "w" || k == "W" {
                self.toggle_setting(true);
            } else if k == "b" || k == "B" {
                self.toggle_setting(false);
            } else if k == "f" || k == "F" {
                self.cycle_apples();
            } else if k == "h" || k == "H" {
                self.toggle_hardcore();
            } else if k == "c" || k == "C" {
                self.open_personalize();
            } else if k == "k" || k == "K" {
                if self.view3d_active() {
                    self.open_cam3d();
                }
            } else if k == "Return" || k == "space" {
                self.b.play_sound("click");
                self.start_play();
            }
        } else if ev.kind == Kind::MouseDown {
            let p = ev.pos;
            if self.brush_rect.hit(p) {
                self.open_personalize();
            } else if self.mode_left.hit(p) {
                self.cycle_mode(-1);
            } else if self.mode_right.hit(p) || self.mode_panel.hit(p) {
                self.cycle_mode(1);
            } else if self.view_rect.hit(p) {
                self.toggle_view();
            } else if self.wrap_rect.hit(p) {
                if self.view3d_active() {
                    self.open_cam3d();
                } else {
                    self.toggle_setting(true);
                }
            } else if self.bonus_rect.hit(p) {
                self.toggle_setting(false);
            } else if self.apples_rect.hit(p) {
                if self.competitive() {
                    self.toggle_hardcore();
                } else {
                    self.cycle_apples();
                }
            } else if self.start_rect.hit(p) {
                self.b.play_sound("click");
                self.start_play();
            }
        }
    }

    fn set_boost(&mut self, i: usize, on: bool) {
        let sn = &mut self.snakes[i];
        if !sn.alive {
            return;
        }
        if on {
            if !sn.boost_on && sn.stamina >= BOOST_MIN_START {
                sn.boost_on = true;
                self.b.play_sound("rotate");
            }
        } else {
            sn.boost_on = false;
        }
    }

    fn steer_3d(&mut self, key: &str) {
        let turn = if self.b.is_action(key, "left", None) || key == "Left" {
            Some('L')
        } else if self.b.is_action(key, "right", None) || key == "Right" {
            Some('R')
        } else {
            None
        };
        let sn = &mut self.snakes[0];
        if let Some(t) = turn {
            if sn.turn_queue.len() < 2 {
                sn.turn_queue.push(t);
            }
        }
    }

    fn turn(&mut self, i: usize, key: &str, player: Option<&str>) {
        let (dx, dy) = self.snakes[i].direction;
        let nd = if self.b.is_action(key, "up", player) && dy == 0 {
            Some((0, -1))
        } else if self.b.is_action(key, "down", player) && dy == 0 {
            Some((0, 1))
        } else if self.b.is_action(key, "left", player) && dx == 0 {
            Some((-1, 0))
        } else if self.b.is_action(key, "right", player) && dx == 0 {
            Some((1, 0))
        } else {
            None
        };
        if let Some(d) = nd {
            self.snakes[i].next_direction = d;
        }
    }

    // ----- Prestige ----------------------------------------------------

    fn can_prestige(&self) -> (Option<prestige::Level>, bool) {
        if self.b.multiplayer {
            return (None, false);
        }
        let Some(req) = prestige::next_requirement(self.prestige) else {
            return (None, false);
        };
        let sn = &self.snakes[0];
        let ok = self.apples_bank >= req.apples && sn.body.len() as i64 - req.length >= MIN_LENGTH;
        (Some(req), ok)
    }

    fn try_prestige(&mut self) {
        if self.b.game_over || self.competitive() {
            return;
        }
        let (req, ok) = self.can_prestige();
        let Some(req) = req.filter(|_| ok) else {
            return;
        };
        self.apples_bank -= req.apples;
        let sn = &mut self.snakes[0];
        sn.body.drain(..(req.length as usize).min(sn.body.len()));
        sn.grow = 0;
        self.prestige += 1;
        self.b.ach_event("snake_prestige", None);
        self.b.play_sound("select");
        self.b.rumble(120);
    }

    // ===================================================== Spiellogik

    fn tick(&mut self) {
        let alive: Vec<usize> = (0..self.snakes.len()).filter(|i| self.snakes[*i].alive).collect();
        self.advance(&alive);
        if self.b.game_over || self.slot.is_some() {
            return;
        }
        let booster: Vec<usize> = alive.iter().copied().filter(|i| self.snakes[*i].alive && self.snakes[*i].boost_on && self.snakes[*i].stamina > 0.0).collect();
        if !booster.is_empty() {
            self.advance(&booster);
            for &i in &booster {
                self.snakes[i].stamina = 0f64.max(self.snakes[i].stamina - BOOST_DRAIN);
                if self.snakes[i].boost_on {
                    let head = *self.snakes[i].body.last().unwrap_or(&(0, 0));
                    self.spawn_particles(head, BOOST_GLOW[i % 2], 2);
                    if self.hardcore_active() {
                        self.hardcore_boost_cost(i);
                    }
                }
                if self.snakes[i].stamina <= 0.0 {
                    self.snakes[i].boost_on = false;
                }
            }
            self.rec_mark();
        }
    }

    fn hardcore_boost_cost(&mut self, i: usize) {
        let mut cost = HARDCORE_BOOST_LEN_COST;
        if self.snakes[i].grow > 0 {
            let used = self.snakes[i].grow.min(cost);
            self.snakes[i].grow -= used;
            cost -= used;
        }
        if cost > 0 {
            let schnitt = cost.min(self.snakes[i].body.len() as i64 - MIN_LENGTH);
            if schnitt > 0 {
                let tail = self.snakes[i].body[0];
                self.spawn_particles(tail, COL_HARDCORE, 3);
                let sn = &mut self.snakes[i];
                sn.body.drain(..schnitt as usize);
                sn.prev_body = sn.body.clone();
            } else {
                self.snakes[i].boost_on = false;
            }
        }
    }

    fn advance(&mut self, movers: &[usize]) {
        let mut new_heads: Vec<(usize, Cell)> = Vec::new();
        let v3 = self.view3d_active();
        let wrap = self.wrap_active();
        for &i in movers {
            if !self.snakes[i].alive {
                continue;
            }
            self.snakes[i].prev_body = self.snakes[i].body.clone();
            if v3 && !self.snakes[i].turn_queue.is_empty() {
                let alt = self.snakes[i].direction;
                let t = self.snakes[i].turn_queue.remove(0);
                let nd = rotate_dir(alt, t);
                self.snakes[i].direction = nd;
                self.snakes[i].next_direction = nd;
                if self.cam_turn_shake && nd != alt {
                    self.shake = self.shake.max(0.22);
                }
            } else {
                self.snakes[i].direction = self.snakes[i].next_direction;
            }
            let sn = &self.snakes[i];
            let (hx, hy) = *sn.body.last().unwrap_or(&(0, 0));
            let (mut nx, mut ny) = (hx + sn.direction.0, hy + sn.direction.1);
            if let Some((px, py_)) = self.portal_of((nx, ny)) {
                nx = px + sn.direction.0;
                ny = py_ + sn.direction.1;
            }
            if wrap {
                nx = py::pmod32(nx, self.cols);
                ny = py::pmod32(ny, self.rows);
            }
            new_heads.push((i, (nx, ny)));
        }
        let head_of = |i: usize| new_heads.iter().find(|(j, _)| *j == i).map(|(_, c)| *c);
        let mut tot: HashSet<usize> = HashSet::new();
        if !wrap {
            for &(i, (nx, ny)) in &new_heads {
                if nx < 0 || nx >= self.cols || ny < 0 || ny >= self.rows {
                    tot.insert(i);
                }
            }
        }
        for &(i, kopf) in &new_heads {
            if self.obstacles.contains(&kopf) {
                tot.insert(i);
            }
        }
        for &(i, hi) in &new_heads {
            for &(j, hj) in &new_heads {
                if i < j && hi == hj {
                    tot.insert(i);
                    tot.insert(j);
                }
            }
        }
        let mut belegt: HashSet<Cell> = HashSet::new();
        for (i, sn) in self.snakes.iter().enumerate() {
            if !sn.alive {
                continue;
            }
            if movers.contains(&i) {
                let nh = head_of(i);
                let waechst = nh.is_some_and(|h| self.foods.contains(&h)) || (self.golden.is_some() && nh == self.golden) || sn.grow > 0;
                let koerper: &[Cell] = if waechst { &sn.body } else { &sn.body[1..] };
                belegt.extend(koerper.iter().copied());
            } else {
                belegt.extend(sn.body.iter().copied());
            }
        }
        for &(i, kopf) in &new_heads {
            if belegt.contains(&kopf) {
                tot.insert(i);
            }
        }
        let (mut ate, mut ate_gold) = (false, false);
        for &i in movers {
            if !self.snakes[i].alive {
                continue;
            }
            if tot.contains(&i) {
                self.snakes[i].alive = false;
                let head = *self.snakes[i].body.last().unwrap_or(&(0, 0));
                self.spawn_particles(head, SNAKE_COLORS[i % 2].1, 12);
                if self.view3d_active() {
                    self.shake = 0.6;
                }
                continue;
            }
            let Some(kopf) = head_of(i) else {
                continue;
            };
            self.snakes[i].body.push(kopf);
            if self.foods.contains(&kopf) {
                self.foods.discard(&kopf);
                self.food_anim.retain(|(c, _)| *c != kopf);
                if !self.view3d_active() {
                    self.spawn_eat_fx(kopf);
                }
                self.eat_food(i);
                ate = true;
            } else if self.golden.is_some() && Some(kopf) == self.golden {
                self.eat_golden(i);
                ate_gold = true;
            } else if self.specials.iter().any(|(c, _)| *c == kopf) {
                self.eat_special(i, kopf);
            }
            let sn = &mut self.snakes[i];
            if sn.grow > 0 {
                sn.grow -= 1;
            } else {
                sn.body.remove(0);
            }
        }
        if let Some(p) = self.purple_pending.take() {
            self.apply_purple(p);
        }
        if let Some(p) = self.slot_pending.take() {
            self.open_slot(p);
        }
        if ate {
            self.b.play_sound("eat");
            self.place_food();
            if self.golden.is_none() && py::random() < GOLDEN_CHANCE {
                self.place_golden();
            }
            if self.competitive() {
                self.maybe_spawn_special();
            }
        }
        if ate_gold {
            self.golden = None;
        }
        self.rec_mark();
        self.check_end(!tot.is_empty());
    }

    fn eat_food(&mut self, i: usize) {
        let gain = if self.bonus { py::randint(1, 2) } else { 1 };
        let bpa = prestige::blocks_per_apple(self.prestige);
        self.snakes[i].grow += gain * bpa;
        if self.snake_size(i) >= 50.0 {
            self.b.ach_event("snake_long", None);
        }
        let mult = self.score_multiplier();
        let sn = &mut self.snakes[i];
        sn.stamina = STAMINA_MAX.min(sn.stamina + 0.12);
        sn.apples += gain;
        if self.b.multiplayer {
            sn.score += gain * 10;
        } else {
            self.apples_total += gain;
            self.apples_bank += gain;
            sn.score += gain * 10 * mult;
        }
        self.speed_apples += gain;
        if self.mode_key() == "speed" {
            self.interval = MIN_INTERVAL.max(BASE_INTERVAL - self.speed_apples as f64 * 0.0025);
        }
    }

    fn eat_golden(&mut self, i: usize) {
        let bpa = prestige::blocks_per_apple(self.prestige);
        let mult = self.score_multiplier();
        let sn = &mut self.snakes[i];
        sn.grow += 2 * bpa;
        sn.stamina = STAMINA_MAX;
        let bonus = 3;
        sn.apples += bonus;
        if self.b.multiplayer {
            sn.score += 50;
        } else {
            self.apples_total += bonus;
            self.apples_bank += bonus;
            sn.score += 50 * mult;
        }
        let head = *self.snakes[i].body.last().unwrap_or(&(0, 0));
        self.spawn_particles(head, COL_GOLD, 16);
        self.b.play_sound("point");
        self.b.rumble(80);
    }

    fn score_multiplier(&self) -> i64 {
        if self.competitive() {
            return competitive::score_multiplier(self.comp_level);
        }
        prestige::score_multiplier(self.prestige)
    }

    fn eat_special(&mut self, i: usize, cell: Cell) {
        let Some(pos) = self.specials.iter().position(|(c, _)| *c == cell) else {
            return;
        };
        let sp = self.specials.remove(pos).1;
        if sp.blue {
            self.slot_pending = Some(i);
        } else {
            self.purple_pending = Some(i);
        }
    }

    fn snake_size(&self, i: usize) -> f64 {
        let sn = &self.snakes[i];
        (sn.body.len() as i64 + sn.grow) as f64 + sn.size_frac
    }

    fn set_snake_size(&mut self, i: usize, size: f64) {
        let size = (MIN_LENGTH as f64).max(size);
        let target_len = (size + 1e-9).floor() as i64;
        let sn = &mut self.snakes[i];
        sn.size_frac = size - target_len as f64;
        let delta = target_len - (sn.body.len() as i64 + sn.grow);
        if delta > 0 {
            sn.grow += delta;
        } else if delta < 0 {
            let mut need = -delta;
            let used = sn.grow.min(need);
            sn.grow -= used;
            need -= used;
            if need > 0 {
                let schnitt = need.min(sn.body.len() as i64 - MIN_LENGTH);
                if schnitt > 0 {
                    sn.body.drain(..schnitt as usize);
                    sn.prev_body = sn.body.clone();
                }
            }
        }
    }

    fn apply_purple(&mut self, i: usize) {
        let hc = self.hardcore_active();
        let stake = competitive::purple_stake(hc);
        let factor = competitive::purple_factor(hc);
        let size = self.snake_size(i);
        let new_size = (MIN_LENGTH as f64).max(size * (1.0 - stake) + size * stake * factor);
        self.set_snake_size(i, new_size);
        let gewonnen = new_size >= size;
        let p = ui::pal();
        let col = if gewonnen { p.green } else { p.red };
        self.b.play_sound(if gewonnen { "powerup" } else { "hit" });
        self.b.rumble(80);
        let head = *self.snakes[i].body.last().unwrap_or(&(0, 0));
        self.spawn_particles(head, COL_PURPLE, 14);
        self.add_float_text(head, &format!("x{}", fmt_g(factor)), col);
        self.show_banner(format!("{:.0}% \u{00D7}{}", stake * 100.0, fmt_g(factor)), col, Some(i18n::t("snake.purple_banner")));
    }

    fn show_banner(&mut self, text: String, color: Color, sub: Option<String>) {
        self.banner = Some(BannerInfo { text, color, sub, t: BANNER_TIME });
    }

    fn update_banner(&mut self, dt: f64) {
        if let Some(b) = self.banner.as_mut() {
            b.t -= dt;
            if b.t <= 0.0 {
                self.banner = None;
            }
        }
    }

    fn open_slot(&mut self, i: usize) {
        let reels = competitive::spin_reels();
        let (mult, result) = competitive::slot_outcome(&reels);
        let stake = 2.max(self.snakes[i].body.len() as i64 / 4);
        self.slot = Some(Slot { snake: i, reels, mult, result, stake, t: 0.0, applied: false, snd: [false; 3] });
        self.b.play_sound("point");
    }

    fn update_slot(&mut self, dt: f64) {
        let mut sounds = 0;
        let mut apply = false;
        let mut close = false;
        if let Some(sl) = self.slot.as_mut() {
            sl.t += dt;
            for (k, ts) in SLOT_REEL_STOPS.iter().enumerate() {
                if sl.t >= *ts && !sl.snd[k] {
                    sl.snd[k] = true;
                    sounds += 1;
                }
            }
            let last = SLOT_REEL_STOPS[2];
            if sl.t >= last && !sl.applied {
                sl.applied = true;
                apply = true;
            }
            close = sl.t >= last + SLOT_SHOW;
        }
        for _ in 0..sounds {
            self.b.play_sound("lock");
        }
        if apply {
            self.apply_slot();
        }
        if close {
            self.slot = None;
        }
    }

    fn apply_slot(&mut self) {
        let Some(sl) = self.slot.as_ref() else {
            return;
        };
        let (i, stake, mult, result) = (sl.snake, sl.stake, sl.mult, sl.result);
        let netto = py::round(stake as f64 * (mult - 1.0));
        if netto > 0 {
            self.snakes[i].grow += netto;
        } else if netto < 0 {
            let sn = &mut self.snakes[i];
            let schnitt = (-netto).min(sn.body.len() as i64 - MIN_LENGTH);
            if schnitt > 0 {
                sn.body.drain(..schnitt as usize);
                sn.prev_body = sn.body.clone();
            }
        }
        self.spawn_bonus = self.spawn_bonus.max(py::round(mult));
        self.spawn_bonus_t = SPAWN_BONUS_TIME;
        self.place_food();
        match result {
            "jackpot" => {
                self.b.play_sound("win");
                self.b.rumble(160);
            }
            "pair" => {
                self.b.play_sound("powerup");
                self.b.rumble(80);
            }
            _ => self.b.play_sound("hit"),
        }
        let head = *self.snakes[i].body.last().unwrap_or(&(0, 0));
        self.add_float_text(head, &format!("x{}", fmt_g(mult)), COL_BLUE.c());
    }

    fn add_float_text(&mut self, cell: Cell, text: &str, color: Color) {
        self.float_texts.push(FloatText { x: (cell.0 * CELL) as f64 + CELL as f64 / 2.0, y: (cell.1 * CELL) as f64, text: text.to_string(), color, t: 1.1 });
    }

    fn update_float_texts(&mut self, dt: f64) {
        for ft in self.float_texts.iter_mut() {
            ft.y -= dt * 26.0;
            ft.t -= dt;
        }
        self.float_texts.retain(|ft| ft.t > 0.0);
    }

    fn update_competitive(&mut self, dt: f64) {
        let lvl = competitive::level_for_apples(self.apples_total);
        if lvl > self.comp_level {
            self.comp_level = lvl;
            self.b.ach_event("snake_comp5", Some(lvl));
            self.b.play_sound("level");
            self.b.rumble(90);
            let head = *self.snakes[0].body.last().unwrap_or(&(0, 0));
            self.add_float_text(head, &i18n::tf("snake.comp.levelup", &[("n", &lvl.to_string())]), ui::pal().gold);
            self.place_food();
        }
        if self.spawn_bonus_t > 0.0 {
            self.spawn_bonus_t = 0f64.max(self.spawn_bonus_t - dt);
            if self.spawn_bonus_t == 0.0 {
                self.spawn_bonus = 0;
            }
        }
        for (_, sp) in self.specials.iter_mut() {
            sp.timer -= dt;
        }
        self.specials.retain(|(_, sp)| sp.timer > 0.0);
    }

    fn check_end(&mut self, any_tot: bool) {
        if self.b.multiplayer {
            self.b.score = self.snakes.iter().map(|s| s.score).max().unwrap_or(0);
            let lebende: Vec<usize> = (0..self.snakes.len()).filter(|i| self.snakes[*i].alive).collect();
            if lebende.len() <= 1 && (any_tot || lebende.len() < self.snakes.len()) {
                self.b.game_over = true;
                self.winner = if lebende.len() == 1 { Some(lebende[0]) } else { self.winner_by_score() };
                self.end_effects();
            }
        } else {
            self.b.score = self.snakes[0].score;
            if !self.snakes[0].alive {
                self.b.game_over = true;
                self.end_effects();
            }
        }
    }

    fn winner_by_score(&self) -> Option<usize> {
        let (s0, s1) = (self.snakes[0].score, self.snakes[1].score);
        if s0 > s1 {
            Some(0)
        } else if s1 > s0 {
            Some(1)
        } else {
            None
        }
    }

    fn finish_timed(&mut self) {
        self.b.game_over = true;
        if self.b.multiplayer {
            let (a0, a1) = (self.snakes[0].apples, self.snakes[1].apples);
            self.winner = if a0 > a1 {
                Some(0)
            } else if a1 > a0 {
                Some(1)
            } else {
                None
            };
            self.b.score = self.snakes.iter().map(|s| s.score).max().unwrap_or(0);
        } else {
            self.b.score = self.snakes[0].score;
        }
        self.end_effects();
    }

    fn end_effects(&mut self) {
        if self.effects_done {
            return;
        }
        self.effects_done = true;
        if self.b.multiplayer {
            if let Some(w) = self.winner {
                self.b.report_duel(w);
            }
        }
        self.highscore = self.highscore.max(self.b.score);
        self.rec_finish();
        self.b.play_sound("gameover");
        self.b.rumble(200);
    }

    // ===================================================== Replay-Aufnahme

    fn rec_new(&mut self) {
        self.replay = None;
        self.rec_moves.clear();
        self.rec_n = 0;
        self.rec_last.clear();
        self.rec_delta.reset();
        let mut meta = Map::new();
        meta.insert("mode".into(), json!(self.mode_key()));
        meta.insert("wrap".into(), json!(self.wrap_active()));
        meta.insert("players".into(), json!(self.snakes.len()));
        meta.insert("cols".into(), json!(self.cols));
        meta.insert("rows".into(), json!(self.rows));
        meta.insert("hardcore".into(), json!(self.hardcore_active()));
        meta.insert("view3d".into(), json!(self.view3d_active()));
        self.rec = replay::recorder("snake", shell::replay_enabled(&self.b.settings), meta);
        if self.rec.is_none() {
            return;
        }
        let mut obs: Vec<Cell> = self.obstacles.to_vec();
        obs.sort_unstable();
        let mut lay = Map::new();
        lay.insert("cols".into(), json!(self.cols));
        lay.insert("rows".into(), json!(self.rows));
        lay.insert("obs".into(), json!(obs.iter().flat_map(|c| [c.0, c.1]).collect::<Vec<i32>>()));
        lay.insert("por".into(), Value::Array(self.portal_pairs.iter().map(|(a, b, col)| json!([a.0, a.1, b.0, b.1, [col.0, col.1, col.2]])).collect()));
        if let Some(rec) = self.rec.as_mut() {
            self.rec_layout = rec.layout(Value::Object(lay));
        }
        self.rec_chapter();
    }

    fn rec_chapter(&mut self) {
        if self.rec.is_none() {
            return;
        }
        self.rec_n = 0;
        self.rec_moves.clear();
        self.rec_delta.reset();
        let st = self.rec_state();
        let mut info = Map::new();
        info.insert("layout".into(), json!(self.rec_layout));
        let n = self.rec.as_ref().map(|r| r.scene_count()).unwrap_or(0) + 1;
        info.insert("n".into(), json!(n));
        info.insert("bodies".into(), Value::Array(self.snakes.iter().map(|sn| json!(sn.body.iter().flat_map(|c| [c.0, c.1]).collect::<Vec<i32>>())).collect()));
        info.insert("dirs".into(), Value::Array(self.snakes.iter().map(|sn| json!([sn.direction.0, sn.direction.1])).collect()));
        info.insert("st".into(), json!(st));
        if let Some(rec) = self.rec.as_mut() {
            rec.scene(false, info);
        }
    }

    fn rec_state(&self) -> Vec<i64> {
        let mut st = vec![
            self.apples_total,
            self.apples_bank,
            self.prestige,
            self.comp_level,
            if self.mode_key() == "timed" { (self.time_left * 10.0) as i64 } else { -1 },
            (self.interval * 1000.0) as i64,
        ];
        let mut foods = self.foods.to_vec();
        foods.sort_unstable();
        st.push(foods.len() as i64);
        for (x, y) in foods {
            st.push(x as i64);
            st.push(y as i64);
        }
        match self.golden {
            None => st.extend([0, 0, 0, 0]),
            Some(g) => st.extend([1, g.0 as i64, g.1 as i64, (self.golden_timer * 10.0) as i64]),
        }
        let mut sp: Vec<(Cell, Special)> = self.specials.clone();
        sp.sort_by_key(|(c, _)| *c);
        st.push(sp.len() as i64);
        for ((x, y), info) in sp {
            st.extend([x as i64, y as i64, if info.blue { 1 } else { 0 }, (info.timer * 10.0) as i64]);
        }
        for sn in &self.snakes {
            st.extend([sn.score, sn.apples, sn.alive as i64, sn.boost_on as i64, (sn.stamina * 100.0) as i64, (sn.size_frac * 100.0) as i64]);
        }
        st
    }

    fn rec_mark(&mut self) {
        if self.rec.is_none() {
            return;
        }
        for i in 0..self.snakes.len() {
            let sn = &self.snakes[i];
            let Some(&head) = sn.body.last() else {
                continue;
            };
            let blen = sn.body.len();
            let prev = self.rec_last.iter().find(|(j, _, _)| *j == i).map(|(_, h, l)| (*h, *l));
            if prev == Some((head, blen)) {
                continue;
            }
            match self.rec_last.iter_mut().find(|(j, _, _)| *j == i) {
                Some(e) => {
                    e.1 = head;
                    e.2 = blen;
                }
                None => self.rec_last.push((i, head, blen)),
            }
            let step = if prev.is_none_or(|p| p.0 != head) { 1 } else { 0 };
            self.rec_moves.push([i as i64, head.0 as i64, head.1 as i64, blen as i64, step]);
        }
    }

    fn rec_sample(&mut self) -> Vec<Value> {
        let mut out: Vec<Value> = vec![json!(self.rec_moves.len())];
        for m in std::mem::take(&mut self.rec_moves) {
            out.extend(m.iter().map(|v| json!(v)));
        }
        let st = self.rec_state();
        let stv: Vec<Value> = st.iter().map(|v| json!(v)).collect();
        if self.rec_delta.push(0, &stv) {
            out.push(json!(st.len()));
            out.extend(stv);
        } else {
            out.push(json!(0));
        }
        self.rec_n += 1;
        out
    }

    fn rec_tick(&mut self, dt: f64) {
        if self.rec.is_none() {
            return;
        }
        if self.rec_n >= REC_CHAPTER && self.rec_moves.is_empty() {
            self.rec_chapter();
        }
        let mut rec = self.rec.take();
        if let Some(r) = rec.as_mut() {
            r.tick(dt, || Some(self.rec_sample()));
        }
        self.rec = rec;
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
        let sub = if self.b.multiplayer {
            match self.winner {
                None => i18n::t("common.draw"),
                Some(w) => i18n::tf("common.player_wins_name", &[("name", &self.b.pname(w))]),
            }
        } else {
            i18n::tf("snake.replay_sub", &[("score", &self.b.score.to_string()), ("apples", &self.apples_total.to_string()), ("len", &self.snakes[0].body.len().to_string())])
        };
        let mut meta = Map::new();
        meta.insert("score".into(), json!(self.b.score));
        meta.insert("winner".into(), json!(self.winner.map(|w| w as i64).unwrap_or(-1)));
        self.replay = rec.as_mut().and_then(|r| r.result(&i18n::t(&format!("snake.mode.{}", self.mode_key())), &sub, meta));
        self.rec = None;
    }

    fn open_replay(&mut self) {
        if self.replay.is_some() {
            self.replay_request = self.replay.clone();
            self.b.play_sound("click");
        }
    }

    // ===================================================== Replay-Wiedergabe

    /// `replay_begin(rep)` - auf reine Wiedergabe umschalten.
    pub fn replay_begin(&mut self, rep: &Value) {
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = Some(rep.clone());
        self.rep_at = None;
        self.rep_state.clear();
        let meta = rep.get("meta").and_then(|m| m.as_object()).cloned().unwrap_or_default();
        if let Some(m) = meta.get("mode").and_then(|v| v.as_str()) {
            if MODE_KEYS.contains(&m) {
                self.mode_index = Self::mode_idx(m);
            }
        }
        let tr = |v: Option<&Value>| v.is_some_and(|x| x.as_bool().unwrap_or(false) || x.as_f64().is_some_and(|f| f != 0.0));
        self.wrap = tr(meta.get("wrap"));
        self.hardcore = tr(meta.get("hardcore"));
        self.view3d = false;
        self.b.multiplayer = meta.get("players").and_then(|v| v.as_i64()).filter(|p| *p != 0).unwrap_or(1) > 1;
        let lay = rep.get("layouts").and_then(|l| l.as_array()).and_then(|l| l.first()).and_then(|l| l.as_object()).cloned().unwrap_or_default();
        let nz = |v: Option<&Value>| v.and_then(|x| x.as_i64()).filter(|x| *x != 0);
        self.cols = nz(lay.get("cols")).or(nz(meta.get("cols"))).unwrap_or(self.cols as i64) as i32;
        self.rows = nz(lay.get("rows")).or(nz(meta.get("rows"))).unwrap_or(self.rows as i64) as i32;
        let obs: Vec<i64> = lay.get("obs").and_then(|o| o.as_array()).map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
        self.obstacles = CellSet::cells();
        let mut i = 0;
        while i + 1 < obs.len() {
            self.obstacles.add((obs[i] as i32, obs[i + 1] as i32));
            i += 2;
        }
        self.portals.clear();
        self.portal_pairs.clear();
        for e in lay.get("por").and_then(|p| p.as_array()).cloned().unwrap_or_default() {
            let a = e.as_array().cloned().unwrap_or_default();
            let g = |k: usize| a.get(k).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let (pa, pb) = ((g(0), g(1)), (g(2), g(3)));
            let col = a.get(4).and_then(|c| c.as_array()).map(|c| {
                let h = |k: usize| c.get(k).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                (h(0), h(1), h(2))
            });
            self.portals.push((pa, pb));
            self.portals.push((pb, pa));
            self.portal_pairs.push((pa, pb, col.unwrap_or((255, 255, 255))));
        }
        self.foods = CellSet::cells();
        self.food_anim.clear();
        self.eat_fx.clear();
        self.golden = None;
        self.golden_timer = 0.0;
        self.specials.clear();
        self.slot = None;
        self.float_texts.clear();
        self.banner = None;
        self.purple_pending = None;
        self.slot_pending = None;
        self.particles.clear();
        self.particles3d.clear();
        self.particles3d_col.clear();
        self.shake = 0.0;
        self.time_left = TIMED_SECONDS;
        self.interval = BASE_INTERVAL;
        self.b.game_over = false;
        let win = meta.get("winner").and_then(|v| v.as_i64()).unwrap_or(-1);
        self.winner = if win >= 0 { Some(win as usize) } else { None };
        self.run_t = 99.0;
        self.hud_h = 78;
        self.state = St::Play;
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
        let start;
        if self.rep_at.is_none_or(|(i, f)| i != index || frame < f) {
            self.snakes.clear();
            let dirs = sc.get("dirs").and_then(|d| d.as_array()).cloned().unwrap_or_default();
            for (i, flat) in sc.get("bodies").and_then(|b| b.as_array()).cloned().unwrap_or_default().iter().enumerate() {
                let fl: Vec<i64> = flat.as_array().map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
                let mut body = Vec::new();
                let mut k = 0;
                while k + 1 < fl.len() {
                    body.push((fl[k] as i32, fl[k + 1] as i32));
                    k += 2;
                }
                let d = dirs.get(i).and_then(|d| d.as_array()).map(|d| (d.first().and_then(|v| v.as_i64()).unwrap_or(1) as i32, d.get(1).and_then(|v| v.as_i64()).unwrap_or(0) as i32)).unwrap_or((1, 0));
                self.snakes.push(Snk::new(body, d));
            }
            self.rep_state = sc.get("st").and_then(|s| s.as_array()).map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
            self.rep_state_apply();
            start = 0;
        } else {
            start = self.rep_at.map(|(_, f)| f + 1).unwrap_or(0);
        }
        for fr in frames.iter().take(frame + 1).skip(start) {
            let v: Vec<i64> = fr.as_array().map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
            self.rep_apply(&v);
        }
        self.b.score = self.snakes.iter().map(|s| s.score).max().unwrap_or(0);
        self.rep_at = Some((index, frame));
    }

    fn rep_apply(&mut self, fr: &[i64]) {
        if fr.is_empty() {
            return;
        }
        let nm = fr[0];
        let mut p = 1;
        for _ in 0..nm {
            if p + 5 > fr.len() {
                return;
            }
            let (i, hx, hy, blen, step) = (fr[p], fr[p + 1] as i32, fr[p + 2] as i32, fr[p + 3] as usize, fr[p + 4]);
            p += 5;
            if i >= 0 && (i as usize) < self.snakes.len() {
                let sn = &mut self.snakes[i as usize];
                if step != 0 {
                    sn.body.push((hx, hy));
                }
                if sn.body.len() > blen {
                    let cut = sn.body.len() - blen;
                    sn.body.drain(..cut);
                }
                sn.prev_body = sn.body.clone();
            }
        }
        let k = *fr.get(p).unwrap_or(&0) as usize;
        if k != 0 {
            self.rep_state = fr.iter().skip(p + 1).take(k).copied().collect();
            self.rep_state_apply();
        }
    }

    fn rep_state_apply(&mut self) {
        let st = self.rep_state.clone();
        if st.len() < 7 {
            return;
        }
        self.apples_total = st[0];
        self.apples_bank = st[1];
        self.prestige = st[2];
        self.comp_level = st[3];
        self.time_left = if st[4] >= 0 { 0f64.max(st[4] as f64 / 10.0) } else { TIMED_SECONDS };
        self.interval = 0.001f64.max(st[5] as f64 / 1000.0);
        let g = |k: usize| *st.get(k).unwrap_or(&0);
        let mut p = 6;
        let nf = g(p) as usize;
        p += 1;
        self.foods = CellSet::cells();
        for i in 0..nf {
            self.foods.add((g(p + 2 * i) as i32, g(p + 2 * i + 1) as i32));
        }
        p += 2 * nf;
        self.food_anim = self.foods.iter().map(|c| (c, FOOD_SPAWN_ANIM)).collect();
        self.golden = if g(p) != 0 { Some((g(p + 1) as i32, g(p + 2) as i32)) } else { None };
        self.golden_timer = g(p + 3) as f64 / 10.0;
        p += 4;
        let ns = g(p) as usize;
        p += 1;
        self.specials.clear();
        for _ in 0..ns {
            self.specials.push(((g(p) as i32, g(p + 1) as i32), Special { blue: g(p + 2) != 0, timer: g(p + 3) as f64 / 10.0 }));
            p += 4;
        }
        for sn in self.snakes.iter_mut() {
            sn.score = g(p);
            sn.apples = g(p + 1);
            sn.alive = g(p + 2) != 0;
            sn.boost_on = g(p + 3) != 0;
            sn.stamina = g(p + 4) as f64 / 100.0;
            sn.size_frac = g(p + 5) as f64 / 100.0;
            p += 6;
        }
    }

    /// `replay_draw(aiming, banner)`
    pub fn replay_draw(&mut self, banner: bool) {
        self.anim_t += 1.0 / replay::RATE;
        let mut s = self.b.sf();
        self.draw_world_2d(&mut s);
        self.b.game_over = true;
        self.draw_hud(&mut s);
        if banner {
            self.draw_game_over(&mut s);
        }
        self.b.game_over = false;
    }

    // ----- Partikel ----------------------------------------------------

    fn spawn_particles(&mut self, cell: Cell, color: Rgb, n: usize) {
        if self.view3d_active() {
            let (wx, wz) = (cell.0 as f64 + 0.5, cell.1 as f64 + 0.5);
            for _ in 0..n {
                let ang = py::uniform(0.0, TAU);
                let spd = py::uniform(1.0, 4.0);
                let y = py::uniform(0.2, 0.6);
                let vy = py::uniform(1.0, 4.5);
                let life = py::uniform(0.35, 0.7);
                self.particles3d.push([wx, y, wz, ang.cos() * spd, vy, ang.sin() * spd, life]);
                self.particles3d_col.push(color);
            }
            return;
        }
        let cx = (cell.0 * CELL) as f64 + CELL as f64 / 2.0;
        let cy = (cell.1 * CELL) as f64 + CELL as f64 / 2.0;
        for _ in 0..n {
            let ang = py::uniform(0.0, TAU);
            let spd = py::uniform(30.0, 130.0);
            let life = py::uniform(0.25, 0.55);
            self.particles.push((cx, cy, ang.cos() * spd, ang.sin() * spd, life, color));
        }
    }

    fn update_particles3d(&mut self, dt: f64) {
        let mut keep_p = Vec::new();
        let mut keep_c = Vec::new();
        for (mut p, c) in std::mem::take(&mut self.particles3d).into_iter().zip(std::mem::take(&mut self.particles3d_col)) {
            p[0] += p[3] * dt;
            p[1] += p[4] * dt;
            p[2] += p[5] * dt;
            p[4] -= 9.0 * dt;
            if p[1] < 0.04 {
                p[1] = 0.04;
                p[4] *= -0.4;
            }
            p[6] -= dt;
            if p[6] > 0.0 {
                keep_p.push(p);
                keep_c.push(c);
            }
        }
        self.particles3d = keep_p;
        self.particles3d_col = keep_c;
    }

    fn update_camera(&mut self, dt: f64) {
        let cells = self.interp_cells(0);
        let (hx, hz) = *cells.last().unwrap_or(&(0.0, 0.0));
        let head = (hx + 0.5, 0.0, hz + 0.5);
        let (tx, tz, back, look);
        if self.b.game_over {
            self.orbit_a += dt * 0.55;
            tx = self.orbit_a.sin();
            tz = self.orbit_a.cos();
            back = CAM_BACK + 1.6;
            look = Some((head.0, 0.25, head.2));
        } else {
            let d = self.snakes[0].direction;
            tx = d.0 as f64;
            tz = d.1 as f64;
            back = CAM_BACK;
            look = None;
        }
        let dir_rate = if self.cam_smooth { 2.6 } else { 4.5 };
        let pos_rate = CAM_SMOOTH * if self.cam_smooth { 0.55 } else { 1.0 };
        let k = 1f64.min(dt * dir_rate);
        let fx = self.cam_dir.0 + (tx - self.cam_dir.0) * k;
        let fz = self.cam_dir.1 + (tz - self.cam_dir.1) * k;
        let ln = py::hypot(fx, fz);
        if ln > 1e-6 {
            self.cam_dir = (fx / ln, fz / ln);
        }
        let (fx, fz) = self.cam_dir;
        let tgt_pos = (head.0 - fx * back, self.cam_height, head.2 - fz * back);
        let tgt_look = look.unwrap_or((head.0 + fx * CAM_AHEAD, CAM_LOOK_H, head.2 + fz * CAM_AHEAD));
        let kp = 1f64.min(dt * pos_rate);
        let l = |a: f64, b: f64| a + (b - a) * kp;
        self.cam_pos = (l(self.cam_pos.0, tgt_pos.0), l(self.cam_pos.1, tgt_pos.1), l(self.cam_pos.2, tgt_pos.2));
        self.cam_look = (l(self.cam_look.0, tgt_look.0), l(self.cam_look.1, tgt_look.1), l(self.cam_look.2, tgt_look.2));
        let sn = &self.snakes[0];
        let boost = sn.alive && sn.boost_on && sn.stamina > 0.0;
        let want = self.cam_fov - if boost { FOV_BOOST_DELTA } else { 0.0 };
        self.fov_mul += (want - self.fov_mul) * 1f64.min(dt * 6.0);
    }

    fn interp_cells(&self, i: usize) -> Vec<(f64, f64)> {
        let sn = &self.snakes[i];
        let frac = if self.b.game_over || !sn.alive { 1.0 } else { 0f64.max(1f64.min(self.timer / 1e-6f64.max(self.interval))) };
        let (n, m) = (sn.body.len() as i64, sn.prev_body.len() as i64);
        let mut out = Vec::new();
        for (k, cur) in sn.body.iter().enumerate() {
            let pi = m - (n - k as i64);
            let mut prev = if 0 <= pi && pi < m { sn.prev_body[pi as usize] } else { *cur };
            if ((cur.0 - prev.0).abs() + (cur.1 - prev.1).abs()) as f64 > 1.5 {
                prev = *cur;
            }
            out.push((prev.0 as f64 + (cur.0 - prev.0) as f64 * frac, prev.1 as f64 + (cur.1 - prev.1) as f64 * frac));
        }
        out
    }

    fn update_particles(&mut self, dt: f64) {
        for p in self.particles.iter_mut() {
            p.0 += p.2 * dt;
            p.1 += p.3 * dt;
            p.4 -= dt;
        }
        self.particles.retain(|p| p.4 > 0.0);
    }

    fn update_food_anim(&mut self, dt: f64) {
        let foods = &self.foods;
        self.food_anim.retain(|(c, _)| foods.contains(c));
        for (_, v) in self.food_anim.iter_mut() {
            *v = FOOD_SPAWN_ANIM.min(*v + dt);
        }
        for e in self.eat_fx.iter_mut() {
            e.1 -= dt;
        }
        self.eat_fx.retain(|e| e.1 > 0.0);
    }

    fn spawn_eat_fx(&mut self, cell: Cell) {
        self.eat_fx.push((cell, FOOD_EAT_ANIM));
        self.spawn_particles(cell, COL_FOOD, 8);
    }

    // ===================================================== Zeichnen 2D

    fn draw_grid(&mut self, s: &mut Surface) {
        s.fill_c(COL_BG);
        let Some(seq) = ngb::grid_sequence() else {
            for x in (0..self.cols * CELL).step_by(CELL as usize) {
                draw::line(s, COL_GRID, (x, 0), (x, self.rows * CELL), 1);
            }
            for y in (0..self.rows * CELL).step_by(CELL as usize) {
                draw::line(s, COL_GRID, (0, y), (self.cols * CELL, y), 1);
            }
            return;
        };
        let key = (self.cols, self.rows, seq.clone());
        if self.grid_key.as_ref() != Some(&key) {
            self.grid_key = Some(key);
            self.grid_surf = Some(self.build_grid_overlay(&seq));
        }
        if let Some(g) = &self.grid_surf {
            s.bl(g, (0, 0));
        }
    }

    fn col_label(i: i32) -> String {
        let mut name = String::new();
        let mut i = i + 1;
        while i > 0 {
            let r = (i - 1) % 26;
            i = (i - 1) / 26;
            name.insert(0, (b'a' + r as u8) as char);
        }
        name
    }

    fn faded_text(&self, text: &str, color: Rgb, alpha: i32) -> Surface {
        let mut img = self.grid_font.render(text, true, color.c()).copy();
        img.fill_f(rgba(255, 255, 255, alpha), BLEND_RGBA_MULT);
        img
    }

    fn build_grid_overlay(&self, seq: &[Rgb]) -> Surface {
        let n = seq.len();
        let (w, h) = (self.cols * CELL, self.rows * CELL);
        let mut surf = pg::surface(w, h);
        surf.fill_c(COL_BG);
        let bg = [COL_BG.0, COL_BG.1, COL_BG.2];
        for gy in 0..self.rows {
            let band = seq[gy as usize % n];
            let bd = [band.0, band.1, band.2];
            let tint = ((bd[0] + bg[0] * 3) / 4, (bd[1] + bg[1] * 3) / 4, (bd[2] + bg[2] * 3) / 4);
            surf.fill_r(tint, Rect::new(0, gy * CELL, w, CELL));
        }
        let s0 = seq[0];
        let line = ((s0.0 + bg[0]) / 2, (s0.1 + bg[1]) / 2, (s0.2 + bg[2]) / 2);
        for gx in 0..=self.cols {
            draw::line(&mut surf, line, (gx * CELL, 0), (gx * CELL, h), 1);
        }
        for gy in 0..=self.rows {
            draw::line(&mut surf, line, (0, gy * CELL), (w, gy * CELL), 1);
        }
        for gy in 0..self.rows {
            let c = seq[gy as usize % n];
            let col = (255.min(c.0 + 110), 255.min(c.1 + 110), 255.min(c.2 + 110));
            let edge = self.faded_text(&(gy + 1).to_string(), col, 170);
            let cy = gy * CELL + CELL / 2;
            surf.blr(&edge, edge.get_rect().midleft_at((3, cy)));
            surf.blr(&edge, edge.get_rect().midright_at((w - 3, cy)));
        }
        for gx in 0..self.cols {
            let lab = self.faded_text(&Self::col_label(gx), (205, 212, 226), 155);
            let cx = gx * CELL + CELL / 2;
            surf.blr(&lab, lab.get_rect().midtop_at((cx, 1)));
            surf.blr(&lab, lab.get_rect().midbottom_at((cx, h - 1)));
        }
        surf
    }

    fn draw_world_2d(&mut self, s: &mut Surface) {
        self.draw_grid(s);
        for (x, y) in self.obstacles.iter() {
            draw::rect(s, COL_WALL, Rect::new(x * CELL, y * CELL, CELL, CELL), 0);
            draw::rect(s, (95, 104, 128), Rect::new(x * CELL, y * CELL, CELL, CELL), 1);
        }
        for (a, b, col) in self.portal_pairs.clone() {
            for (px, py_) in [a, b] {
                let (cx, cy) = (px * CELL + CELL / 2, py_ * CELL + CELL / 2);
                let r = CELL / 2 - 1 + (1.5 * (self.anim_t * 6.0).sin()) as i32;
                draw::circle(s, col, (cx, cy), r, 3);
                draw::circle(s, (col.0 / 2, col.1 / 2, col.2 / 2), (cx, cy), 2.max(r - 4), 0);
            }
        }
        for cell in self.foods.to_vec() {
            let fa = self.food_anim.iter().find(|(c, _)| *c == cell).map(|(_, v)| *v).unwrap_or(FOOD_SPAWN_ANIM);
            let p = 1f64.min(fa / FOOD_SPAWN_ANIM);
            Self::draw_apple(s, cell, ease_out_back(p), 255);
            if p < 1.0 {
                Self::draw_fx_ring(s, cell, (CELL as f64 * 0.28 + CELL as f64 * 0.5 * p) as i32, (255, 150, 150), (150.0 * (1.0 - p)) as i32);
            }
        }
        if let Some((gx, gy)) = self.golden {
            let blink = self.golden_timer > 1.5 || ((self.anim_t * 8.0) as i64) % 2 == 0;
            if blink {
                let (cx, cy) = (gx * CELL + CELL / 2, gy * CELL + CELL / 2);
                let r = CELL / 2 - 1 + (1.5 * (self.anim_t * 8.0).sin()) as i32;
                draw::circle(s, COL_GOLD, (cx, cy), r, 0);
                draw::circle(s, (255, 245, 200), (cx - 2, cy - 2), 2, 0);
            }
        }
        self.draw_specials(s);
        for p in &self.particles {
            let a = 0.max(255.min((255.0 * (p.4 / 0.55)) as i32));
            let mut surf = pg::surface_alpha(6, 6);
            draw::circle(&mut surf, rgba(p.5 .0, p.5 .1, p.5 .2, a), (3, 3), 3, 0);
            s.bl(&surf, (p.0 - 3.0, p.1 - 3.0));
        }
        for idx in 0..self.snakes.len() {
            self.draw_snake(s, idx);
        }
        self.draw_eat_fx(s);
        self.draw_float_texts(s);
    }

    fn draw_specials(&self, s: &mut Surface) {
        for (cell, info) in &self.specials {
            if info.timer < 1.5 && ((self.anim_t * 8.0) as i64) % 2 == 0 {
                continue;
            }
            let col = if info.blue { COL_BLUE } else { COL_PURPLE };
            let inner = (255.min(col.0 + 70), 255.min(col.1 + 70), 255.min(col.2 + 70));
            let cx = cell.0 * CELL + CELL / 2;
            let cy = cell.1 * CELL + CELL / 2;
            let r = CELL / 2 - 2 + (1.5 * (self.anim_t * 6.0 + cell.0 as f64).sin()) as i32;
            let pts = [(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)];
            draw::polygon(s, col, &pts, 0);
            draw::polygon(s, inner, &pts, 1);
            let glyph = if info.blue { "$" } else { "\u{00B1}" };
            let g = self.tiny.render(glyph, true, rgb(20, 20, 30));
            s.blr(&g, g.get_rect().center_at((cx, cy)));
        }
    }

    fn draw_float_texts(&self, s: &mut Surface) {
        for ft in &self.float_texts {
            let mut img = self.small.render(&ft.text, true, ft.color).copy();
            img.set_alpha(Some(0.max(255.min((255.0 * (ft.t / 1.1)) as i32)) as u8));
            s.blr(&img, img.get_rect().center_at((ft.x as i32, ft.y as i32)));
        }
    }

    fn draw_banner(&mut self, s: &mut Surface) {
        let Some(b) = self.banner.as_ref() else {
            return;
        };
        let cfg = ngb::get_banner();
        if !cfg.on {
            return;
        }
        let appear = 1f64.min((BANNER_TIME - b.t) / 0.18);
        let fade = 1f64.min(0f64.max(b.t / 0.5));
        let a = (255.0 * appear.min(fade) * cfg.opacity) as i32;
        if a <= 0 {
            return;
        }
        let p = ui::pal();
        let big = self.b.big_font.render(&b.text, true, b.color);
        let sub = b.sub.as_ref().map(|t| self.small.render(t, true, p.text_dim));
        let pad = 18;
        let tw = big.w().max(sub.as_ref().map(|s| s.w()).unwrap_or(0));
        let th = big.h() + sub.as_ref().map(|s| s.h() + 4).unwrap_or(0);
        let (pw, ph) = (tw + pad * 2, th + pad);
        let mut panel = pg::surface_alpha(pw, ph);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(p.panel.r, p.panel.g, p.panel.b, 215), pr, 0, 12);
        draw::rect_r(&mut panel, rgba(b.color.r, b.color.g, b.color.b, 255), pr, 2, 12);
        let mut y = pad / 2;
        if let Some(sub) = &sub {
            panel.blr(sub, sub.get_rect().midtop_at((pw / 2, 5)));
            y = 5 + sub.h() + 2;
        }
        panel.blr(&big, big.get_rect().midtop_at((pw / 2, y)));
        let scale = (0.82 + 0.18 * appear) * cfg.size;
        if (scale - 1.0).abs() > 0.01 {
            panel = transform::rotozoom(&panel, 0.0, scale);
        }
        panel.fill_f(rgba(255, 255, 255, a), BLEND_RGBA_MULT);
        let top = 44 + (-14.0 * (1.0 - appear)) as i32;
        s.blr(&panel, panel.get_rect().midtop_at((self.b.width / 2, top)));
    }

    fn draw_apple(s: &mut Surface, cell: Cell, scale: f64, alpha: i32) {
        if scale <= 0.03 {
            return;
        }
        let size = (CELL - 4) as f64 * scale;
        let cx = (cell.0 * CELL) as f64 + CELL as f64 / 2.0;
        let cy = (cell.1 * CELL) as f64 + CELL as f64 / 2.0;
        let rad = ((size / 2.0).min(6.0 + (size / 2.0) * (1.0 - scale))) as i32;
        if alpha >= 255 {
            let rect = Rect::new(0, 0, size as i32, size as i32).center_at((cx as i32, cy as i32));
            draw::rect_r(s, COL_FOOD, rect, 0, rad);
            let hl = 1.max((2.2 * scale) as i32);
            draw::circle(s, (255, 170, 170), ((cx - size * 0.14) as i32, (cy - size * 0.14) as i32), hl, 0);
        } else {
            let d = size as i32 + 2;
            let mut surf = pg::surface_alpha(d, d);
            draw::rect_r(&mut surf, rgba(COL_FOOD.0, COL_FOOD.1, COL_FOOD.2, alpha), Rect::new(1, 1, size as i32, size as i32), 0, rad);
            s.bl(&surf, ((cx - d as f64 / 2.0) as i32, (cy - d as f64 / 2.0) as i32));
        }
    }

    fn draw_fx_ring(s: &mut Surface, cell: Cell, r: i32, color: Rgb, a: i32) {
        if a <= 4 || r <= 0 {
            return;
        }
        let d = r * 2 + 4;
        let mut rs = pg::surface_alpha(d, d);
        draw::circle(&mut rs, rgba(color.0, color.1, color.2, a), (d / 2, d / 2), r, 2);
        let cx = cell.0 * CELL + CELL / 2;
        let cy = cell.1 * CELL + CELL / 2;
        s.bl(&rs, ((cx as f64 - d as f64 / 2.0) as i32, (cy as f64 - d as f64 / 2.0) as i32));
    }

    fn draw_eat_fx(&self, s: &mut Surface) {
        for (cell, t) in &self.eat_fx {
            let p = 1.0 - 0f64.max(t / FOOD_EAT_ANIM);
            Self::draw_apple(s, *cell, 0f64.max((1.0 - p) * 0.9), (210.0 * (1.0 - p)) as i32);
            Self::draw_fx_ring(s, *cell, (CELL as f64 * 0.32 + CELL as f64 * 0.5 * p) as i32, COL_FOOD, (150.0 * (1.0 - p)) as i32);
        }
    }

    fn head_color(idx: usize) -> Rgb {
        if idx == 0 {
            return ngb::head_color();
        }
        SNAKE_COLORS[idx % 2].1
    }

    fn draw_snake(&self, s: &mut Surface, idx: usize) {
        let sn = &self.snakes[idx];
        let mut koerper = SNAKE_COLORS[idx % 2].0;
        let mut kopf = Self::head_color(idx);
        if !sn.alive {
            koerper = (koerper.0 / 2, koerper.1 / 2, koerper.2 / 2);
            kopf = koerper;
        }
        let n = sn.body.len();
        if sn.alive && sn.boost_on && sn.stamina > 0.0 {
            let (hx, hy) = *sn.body.last().unwrap_or(&(0, 0));
            let mut glow = pg::surface_alpha(CELL * 3, CELL * 3);
            let pulse = 120 + (60.0 * (self.anim_t * 14.0).sin()) as i32;
            let g = BOOST_GLOW[idx % 2];
            draw::circle(&mut glow, rgba(g.0, g.1, g.2, pulse), (CELL * 3 / 2, CELL * 3 / 2), CELL, 0);
            s.bl(&glow, (hx * CELL + CELL / 2 - CELL * 3 / 2, hy * CELL + CELL / 2 - CELL * 3 / 2));
        }
        for (i, &(x, y)) in sn.body.iter().enumerate() {
            let farbe = if i == n - 1 {
                kopf
            } else {
                let t = i as f64 / 1.max(n as i64 - 1) as f64;
                let m = |k: i32, c: i32| (k as f64 + (c - k) as f64 * (1.0 - t * 0.5)) as i32;
                (m(kopf.0, koerper.0), m(kopf.1, koerper.1), m(kopf.2, koerper.2))
            };
            draw::rect_r(s, farbe, Rect::new(x * CELL + 1, y * CELL + 1, CELL - 2, CELL - 2), 0, 5);
        }
        if !sn.body.is_empty() {
            Self::draw_eyes(s, sn);
        }
    }

    fn draw_eyes(s: &mut Surface, sn: &Snk) {
        let (hx, hy) = *sn.body.last().unwrap_or(&(0, 0));
        let (dx, dy) = (sn.direction.0 as f64, sn.direction.1 as f64);
        let (cx, cy) = ((hx * CELL) as f64 + CELL as f64 / 2.0, (hy * CELL) as f64 + CELL as f64 / 2.0);
        let (px, py_) = (-dy, dx);
        for sign in [-1.0, 1.0] {
            let ex = cx + dx * 3.0 + px * sign * 4.0;
            let ey = cy + dy * 3.0 + py_ * sign * 4.0;
            draw::circle(s, (250, 250, 250), (ex as i32, ey as i32), 3, 0);
            draw::circle(s, (20, 20, 30), ((ex + dx * 1.5) as i32, (ey + dy * 1.5) as i32), 1, 0);
        }
    }

    // ===================================================== 3D-Renderer

    fn draw_world_3d(&mut self, s: &mut Surface) {
        if self.b.game_over {
            self.tick_gameover_anim();
        }
        self.scx = self.b.width as f64 / 2.0;
        self.scy = self.b.height as f64 * 0.5;
        if self.shake > 0.0 {
            let amp = 16.0 * self.shake * if self.cam_smooth { 0.5 } else { 1.0 };
            self.scx += py::uniform(-amp, amp);
            self.scy += py::uniform(-amp, amp);
        }
        self.f = self.b.height as f64 * self.fov_mul;
        self.basis = self.view_basis();
        self.draw_sky(s);
        self.draw_stars3d(s);
        self.draw_floor3d(s);
        let mut items: Vec<Item3> = Vec::new();
        self.add_border_walls(&mut items);
        for (ox, oz) in self.obstacles.to_vec() {
            self.add_box(&mut items, ox as f64, oz as f64, 0.94, 1.05, COL_WALL, Some((95, 104, 128)));
        }
        for f in self.foods.to_vec() {
            let r = 0.30 + 0.05 * (self.anim_t * 5.0 + f.0 as f64 + f.1 as f64).sin();
            self.add_octa(&mut items, f, r, COL_FOOD);
        }
        if let Some(g) = self.golden {
            if self.golden_timer > 1.5 || ((self.anim_t * 8.0) as i64) % 2 == 0 {
                let r = 0.38 + 0.06 * (self.anim_t * 8.0).sin();
                self.add_octa(&mut items, g, r, COL_GOLD);
            }
        }
        for idx in 0..self.snakes.len() {
            self.add_snake3d(&mut items, idx);
        }
        items.sort_by(|a, b| (-a.0).partial_cmp(&(-b.0)).unwrap_or(std::cmp::Ordering::Equal));
        for (_, pts, col, outline) in &items {
            draw::polygon(s, *col, pts, 0);
            if let Some(o) = outline {
                draw::polygon(s, *o, pts, 1);
            }
        }
        self.draw_eyes3d(s);
        self.draw_boost_glow3d(s);
        self.draw_particles3d_pass(s);
    }

    fn tick_gameover_anim(&mut self) {
        let now = py::perf_counter();
        let last = self.go_last.unwrap_or(now);
        self.go_last = Some(now);
        let dt = 0.05f64.min(0f64.max(now - last));
        if dt <= 0.0 {
            return;
        }
        self.anim_t += dt;
        self.update_particles3d(dt);
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - dt * 1.6);
        }
        self.update_camera(dt);
    }

    fn view_basis(&self) -> ((f64, f64, f64), (f64, f64, f64), (f64, f64, f64)) {
        let (cx, cy, cz) = self.cam_pos;
        let fx = self.cam_look.0 - cx;
        let fy = self.cam_look.1 - cy;
        let fz = self.cam_look.2 - cz;
        let mut fl = (fx * fx + fy * fy + fz * fz).sqrt();
        if fl == 0.0 {
            fl = 1.0;
        }
        let f = (fx / fl, fy / fl, fz / fl);
        let mut rl = py::hypot(f.2, f.0);
        if rl == 0.0 {
            rl = 1.0;
        }
        let r = (-f.2 / rl, 0.0, f.0 / rl);
        let u = (r.1 * f.2 - r.2 * f.1, r.2 * f.0 - r.0 * f.2, r.0 * f.1 - r.1 * f.0);
        (r, u, f)
    }

    fn to_cam(&self, p: (f64, f64, f64)) -> (f64, f64, f64) {
        let (r, u, f) = self.basis;
        let dx = p.0 - self.cam_pos.0;
        let dy = p.1 - self.cam_pos.1;
        let dz = p.2 - self.cam_pos.2;
        (dx * r.0 + dz * r.2, dx * u.0 + dy * u.1 + dz * u.2, dx * f.0 + dy * f.1 + dz * f.2)
    }

    fn proj(&self, c: (f64, f64, f64)) -> (f64, f64) {
        let k = self.f / c.2;
        (self.scx + c.0 * k, self.scy - c.1 * k)
    }

    fn clip_near(pts: &[(f64, f64, f64)]) -> Vec<(f64, f64, f64)> {
        let mut out = Vec::new();
        let n = pts.len();
        for i in 0..n {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let (a_in, b_in) = (a.2 >= NEAR, b.2 >= NEAR);
            if a_in {
                out.push(a);
            }
            if a_in != b_in {
                let t = (NEAR - a.2) / (b.2 - a.2);
                out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, NEAR));
            }
        }
        out
    }

    fn shade_col(col: Rgb, k: f64) -> Rgb {
        (255.min((col.0 as f64 * k) as i32), 255.min((col.1 as f64 * k) as i32), 255.min((col.2 as f64 * k) as i32))
    }

    fn fog_color(col: Rgb, depth: f64) -> Rgb {
        let t = (depth - FOG_START) / (FOG_END - FOG_START);
        if t <= 0.0 {
            return col;
        }
        let t = 1f64.min(t);
        (
            (col.0 as f64 + (COL_FOG.0 - col.0) as f64 * t) as i32,
            (col.1 as f64 + (COL_FOG.1 - col.1) as f64 * t) as i32,
            (col.2 as f64 + (COL_FOG.2 - col.2) as f64 * t) as i32,
        )
    }

    fn add_poly(&self, items: &mut Vec<Item3>, world_pts: &[(f64, f64, f64)], color: Rgb, shade: f64, outline: Option<Rgb>) {
        let cs: Vec<(f64, f64, f64)> = world_pts.iter().map(|p| self.to_cam(*p)).collect();
        if cs.iter().all(|c| c.2 < NEAR) {
            return;
        }
        let cs = Self::clip_near(&cs);
        if cs.len() < 3 {
            return;
        }
        let depth = py::sum(&cs.iter().map(|c| c.2).collect::<Vec<f64>>()) / cs.len() as f64;
        if depth > FOG_END + 6.0 {
            return;
        }
        let pts: Vec<(f64, f64)> = cs.iter().map(|c| self.proj(*c)).collect();
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if pts.iter().all(|p| p.0 < -40.0) || pts.iter().all(|p| p.0 > w + 40.0) || pts.iter().all(|p| p.1 < -40.0) || pts.iter().all(|p| p.1 > h + 40.0) {
            return;
        }
        let col = Self::fog_color(Self::shade_col(color, shade), depth);
        items.push((depth, pts, col, if depth < FOG_START + 4.0 { outline } else { None }));
    }

    #[allow(clippy::too_many_arguments)]
    fn add_box(&self, items: &mut Vec<Item3>, gx: f64, gz: f64, w: f64, h: f64, color: Rgb, outline: Option<Rgb>) {
        let (cx, cz) = (gx + 0.5, gz + 0.5);
        let (x0, x1) = (cx - w / 2.0, cx + w / 2.0);
        let (z0, z1) = (cz - w / 2.0, cz + w / 2.0);
        let (px, py_, pz) = self.cam_pos;
        if py_ > h {
            self.add_poly(items, &[(x0, h, z0), (x1, h, z0), (x1, h, z1), (x0, h, z1)], color, 1.0, outline);
        }
        if px < x0 {
            self.add_poly(items, &[(x0, 0.0, z0), (x0, 0.0, z1), (x0, h, z1), (x0, h, z0)], color, 0.62, outline);
        } else if px > x1 {
            self.add_poly(items, &[(x1, 0.0, z0), (x1, 0.0, z1), (x1, h, z1), (x1, h, z0)], color, 0.62, outline);
        }
        if pz < z0 {
            self.add_poly(items, &[(x0, 0.0, z0), (x1, 0.0, z0), (x1, h, z0), (x0, h, z0)], color, 0.76, outline);
        } else if pz > z1 {
            self.add_poly(items, &[(x0, 0.0, z1), (x1, 0.0, z1), (x1, h, z1), (x0, h, z1)], color, 0.76, outline);
        }
    }

    fn add_octa(&self, items: &mut Vec<Item3>, cell: Cell, r: f64, color: Rgb) {
        let y = 0.45;
        let (cx, cz) = (cell.0 as f64 + 0.5, cell.1 as f64 + 0.5);
        let top = (cx, y + r, cz);
        let bot = (cx, 0.04f64.max(y - r), cz);
        let a0 = self.anim_t * 2.4;
        let eq: Vec<(f64, f64, f64)> = (0..4).map(|i| (cx + (a0 + i as f64 * PI / 2.0).cos() * r, y, cz + (a0 + i as f64 * PI / 2.0).sin() * r)).collect();
        let (px, py_, pz) = self.cam_pos;
        let (lx, ly, lz) = (0.42, -0.82, 0.39);
        for i in 0..4 {
            for tri in [[top, eq[i], eq[(i + 1) % 4]], [bot, eq[(i + 1) % 4], eq[i]]] {
                let (a, b, c) = (tri[0], tri[1], tri[2]);
                let (ux, uy, uz) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
                let (vx, vy, vz) = (c.0 - a.0, c.1 - a.1, c.2 - a.2);
                let (mut nx, mut ny, mut nz) = (uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx);
                let fcx = (a.0 + b.0 + c.0) / 3.0;
                let fcy = (a.1 + b.1 + c.1) / 3.0;
                let fcz = (a.2 + b.2 + c.2) / 3.0;
                if nx * (fcx - cx) + ny * (fcy - y) + nz * (fcz - cz) < 0.0 {
                    nx = -nx;
                    ny = -ny;
                    nz = -nz;
                }
                if nx * (fcx - px) + ny * (fcy - py_) + nz * (fcz - pz) >= 0.0 {
                    continue;
                }
                let mut nl = (nx * nx + ny * ny + nz * nz).sqrt();
                if nl == 0.0 {
                    nl = 1.0;
                }
                let lit = 0f64.max(-(nx * lx + ny * ly + nz * lz) / nl);
                self.add_poly(items, &tri, color, 0.55 + 0.5 * lit, None);
            }
        }
    }

    fn add_border_walls(&self, items: &mut Vec<Item3>) {
        let h = 0.55;
        let (cols, rows) = (self.cols as f64, self.rows as f64);
        for gx in 0..self.cols {
            let sh = if gx % 2 == 0 { 0.9 } else { 0.78 };
            let g = gx as f64;
            self.add_poly(items, &[(g, 0.0, 0.0), (g + 1.0, 0.0, 0.0), (g + 1.0, h, 0.0), (g, h, 0.0)], COL_BORDER, sh, None);
            self.add_poly(items, &[(g, 0.0, rows), (g + 1.0, 0.0, rows), (g + 1.0, h, rows), (g, h, rows)], COL_BORDER, sh, None);
        }
        for gz in 0..self.rows {
            let sh = if gz % 2 == 0 { 0.84 } else { 0.7 };
            let g = gz as f64;
            self.add_poly(items, &[(0.0, 0.0, g), (0.0, 0.0, g + 1.0), (0.0, h, g + 1.0), (0.0, h, g)], COL_BORDER, sh, None);
            self.add_poly(items, &[(cols, 0.0, g), (cols, 0.0, g + 1.0), (cols, h, g + 1.0), (cols, h, g)], COL_BORDER, sh, None);
        }
    }

    fn add_snake3d(&self, items: &mut Vec<Item3>, idx: usize) {
        let sn = &self.snakes[idx];
        let mut col_body = SNAKE_COLORS[idx % 2].0;
        let mut col_head = Self::head_color(idx);
        if !sn.alive {
            col_body = (col_body.0 / 2, col_body.1 / 2, col_body.2 / 2);
            col_head = col_body;
        }
        let cells = self.interp_cells(idx);
        let n = cells.len();
        for (k, &(gx, gz)) in cells.iter().enumerate() {
            let (farbe, w, h) = if k == n - 1 {
                (col_head, 0.92, 0.78)
            } else {
                let t = k as f64 / 1.max(n as i64 - 1) as f64;
                let m = |kc: i32, bc: i32| (kc as f64 + (bc - kc) as f64 * (1.0 - t * 0.5)) as i32;
                ((m(col_head.0, col_body.0), m(col_head.1, col_body.1), m(col_head.2, col_body.2)), 0.8, 0.58)
            };
            self.add_box(items, gx, gz, w, h, farbe, Some((farbe.0 / 3, farbe.1 / 3, farbe.2 / 3)));
        }
    }

    fn draw_sky(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        if self.sky_cache.as_ref().map(|c| c.0) != Some((w, h)) {
            let mut surf = pg::surface(w, h);
            let hor = (h as f64 * 0.40) as i32;
            let haze = (h as f64 * 0.55) as i32;
            for y in 0..h {
                let c = if y < hor {
                    let t = y as f64 / 1.max(hor) as f64;
                    let m = |a: i32, b: i32| (a as f64 + (b - a) as f64 * t) as i32;
                    (m(COL_SKY_TOP.0, COL_SKY_HOR.0), m(COL_SKY_TOP.1, COL_SKY_HOR.1), m(COL_SKY_TOP.2, COL_SKY_HOR.2))
                } else if y < haze {
                    let t = (y - hor) as f64 / 1.max(haze - hor) as f64;
                    let m = |a: i32, b: i32| (a as f64 + (b - a) as f64 * t) as i32;
                    (m(COL_SKY_HOR.0, COL_FOG.0), m(COL_SKY_HOR.1, COL_FOG.1), m(COL_SKY_HOR.2, COL_FOG.2))
                } else {
                    COL_FOG
                };
                draw::line(&mut surf, c, (0, y), (w, y), 1);
            }
            self.sky_cache = Some(((w, h), surf));
        }
        if let Some((_, sky)) = &self.sky_cache {
            s.bl(sky, (0, 0));
        }
    }

    fn draw_stars3d(&self, s: &mut Surface) {
        let f = self.basis.2;
        let yaw = f.0.atan2(f.2);
        let hor = self.b.height as f64 * 0.40;
        for &(az, hf, size, ph) in &self.stars {
            let d = py::fpmod(az - yaw + PI, TAU) - PI;
            if d.abs() > 0.7 {
                continue;
            }
            let sx = self.scx + d.tan() * self.f;
            if sx < -4.0 || sx > self.b.width as f64 + 4.0 {
                continue;
            }
            let sy = hor * (0.08 + 0.84 * hf);
            let tw = 0.55 + 0.45 * (self.anim_t * 1.7 + ph).sin();
            let c = (120.0 + 110.0 * tw) as i32;
            draw::circle(s, (c, c, 255.min(c + 25)), (sx as i32, sy as i32), size, 0);
        }
    }

    fn draw_floor3d(&self, s: &mut Surface) {
        let mut corners: std::collections::HashMap<(i32, i32), (f64, f64, f64)> = std::collections::HashMap::new();
        let mut corner = |gx: i32, gz: i32, me: &Self| *corners.entry((gx, gz)).or_insert_with(|| me.to_cam((gx as f64, 0.0, gz as f64)));
        let lim = FOG_END + 1.0;
        for gz in 0..self.rows {
            for gx in 0..self.cols {
                let c00 = corner(gx, gz, self);
                if c00.2 > lim || c00.2 < -1.8 {
                    continue;
                }
                if c00.0.abs() > c00.2 * 1.7 + 2.5 {
                    continue;
                }
                let quad = [c00, corner(gx + 1, gz, self), corner(gx + 1, gz + 1, self), corner(gx, gz + 1, self)];
                if quad.iter().all(|c| c.2 < NEAR) {
                    continue;
                }
                let poly = Self::clip_near(&quad);
                if poly.len() < 3 {
                    continue;
                }
                let depth = py::sum(&poly.iter().map(|c| c.2).collect::<Vec<f64>>()) / poly.len() as f64;
                let col = if (gx + gz) & 1 == 0 { COL_TILE_A } else { COL_TILE_B };
                let pts: Vec<(f64, f64)> = poly.iter().map(|c| self.proj(*c)).collect();
                draw::polygon(s, Self::fog_color(col, depth), &pts, 0);
            }
        }
    }

    fn draw_eyes3d(&self, s: &mut Surface) {
        let sn = &self.snakes[0];
        if !sn.alive {
            return;
        }
        let cells = self.interp_cells(0);
        let (hx, hz) = *cells.last().unwrap_or(&(0.0, 0.0));
        let (dx, dz) = (sn.direction.0 as f64, sn.direction.1 as f64);
        let (fx, fz) = (hx + 0.5 + dx * 0.47, hz + 0.5 + dz * 0.47);
        if (fx - self.cam_pos.0) * dx + (fz - self.cam_pos.2) * dz > 0.0 {
            return;
        }
        let (px, pz) = (-dz, dx);
        for sign in [-1.0, 1.0] {
            let e = (fx + px * sign * 0.17, 0.52, fz + pz * sign * 0.17);
            let c = self.to_cam(e);
            if c.2 < NEAR {
                continue;
            }
            let (sx, sy) = self.proj(c);
            let rr = 1.max((self.f * 0.05 / c.2) as i32);
            draw::circle(s, (250, 250, 250), (sx as i32, sy as i32), rr, 0);
            let pupil = self.to_cam((e.0 + dx * 0.06, 0.5, e.2 + dz * 0.06));
            if pupil.2 >= NEAR {
                let pxy = self.proj(pupil);
                draw::circle(s, (20, 20, 30), (pxy.0 as i32, pxy.1 as i32), 1.max(rr / 2), 0);
            }
        }
    }

    fn draw_boost_glow3d(&self, s: &mut Surface) {
        let sn = &self.snakes[0];
        if !(sn.alive && sn.boost_on && sn.stamina > 0.0) {
            return;
        }
        let cells = self.interp_cells(0);
        let (hx, hz) = *cells.last().unwrap_or(&(0.0, 0.0));
        let c = self.to_cam((hx + 0.5, 0.42, hz + 0.5));
        if c.2 < NEAR {
            return;
        }
        let (sx, sy) = self.proj(c);
        let r = 8.max((self.f * 0.62 / c.2) as i32);
        let pulse = 90 + (50.0 * (self.anim_t * 14.0).sin()) as i32;
        let mut glow = pg::surface_alpha(r * 2, r * 2);
        let g = BOOST_GLOW[0];
        draw::circle(&mut glow, rgba(g.0, g.1, g.2, pulse), (r, r), r, 0);
        s.bl(&glow, (sx - r as f64, sy - r as f64));
    }

    fn draw_particles3d_pass(&self, s: &mut Surface) {
        for (p, col) in self.particles3d.iter().zip(&self.particles3d_col) {
            let c = self.to_cam((p[0], p[1], p[2]));
            if c.2 < NEAR {
                continue;
            }
            let (sx, sy) = self.proj(c);
            if sx < -8.0 || sx > self.b.width as f64 + 8.0 || sy < -8.0 || sy > self.b.height as f64 + 8.0 {
                continue;
            }
            let r = 1.max((self.f * 0.045 / c.2) as i32);
            let t = 0f64.max(1f64.min(p[6] / 0.7));
            let m = |pc: i32, fc: i32| (fc as f64 + (pc - fc) as f64 * t) as i32;
            let cc = (m(col.0, COL_FOG.0), m(col.1, COL_FOG.1), m(col.2, COL_FOG.2));
            draw::circle(s, Self::fog_color(cc, c.2), (sx as i32, sy as i32), r, 0);
        }
    }

    // ===================================================== HUD

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        if self.view3d_active() {
            let badge = self.b.font.render(&i18n::t("snake.view_3d"), true, p.accent);
            s.blr(&badge, badge.get_rect().midtop_at((w / 2, 6)));
        } else {
            let wls_col = if self.wrap_active() { p.green } else { p.text_faint };
            let wls = self.b.font.render(&i18n::t("snake.wls"), true, wls_col);
            s.blr(&wls, wls.get_rect().midtop_at((w / 2, 6)));
        }
        let mimg = self.small.render(&i18n::t(&format!("snake.mode.{}", self.mode_key())), true, p.accent);
        s.blr(&mimg, mimg.get_rect().midtop_at((w / 2, 30)));
        if self.view3d_active() && !self.b.game_over && self.run_t < 6.0 {
            let hint = self.tiny.render(&i18n::t("snake.steer_hint"), true, p.text_dim);
            s.blr(&hint, hint.get_rect().midtop_at((w / 2, 50)));
        }
        if self.mode_key() == "timed" {
            let col = if self.time_left <= 10.0 { p.red } else { p.text };
            let timg = ui::font(22, false, true).render(&i18n::tf("snake.time", &[("t", &format!("{:04.1}", self.time_left))]), true, col);
            s.blr(&timg, timg.get_rect().midtop_at((w / 2, 50)));
        }
        if self.b.multiplayer {
            let n0: String = self.b.pname(0).chars().take(6).collect();
            let n1: String = self.b.pname(1).chars().take(6).collect();
            let p1 = self.b.font.render(&format!("{}: {}", n0, self.snakes[0].score), true, SNAKE_COLORS[0].1.c());
            let p2 = self.b.font.render(&format!("{}: {}", n1, self.snakes[1].score), true, SNAKE_COLORS[1].1.c());
            s.bl(&p1, (10, 8));
            s.bl(&p2, (w - p2.w() - 10, 8));
            self.draw_boost_bar(s, 10, self.b.height - 22, 150, 0);
            self.draw_boost_bar(s, w - 160, self.b.height - 22, 150, 1);
            return;
        }
        let pts = self.b.font.render(&i18n::tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.bl(&pts, (10, 8));
        if self.competitive() {
            self.draw_comp_hud(s);
        } else {
            let a = self.small.render(&i18n::tf("snake.apples_bank", &[("total", &self.apples_total.to_string()), ("bank", &self.apples_bank.to_string())]), true, COL_FOOD.c());
            s.bl(&a, (10, 34));
            if self.prestige > 0 {
                let blocks = prestige::blocks_per_apple(self.prestige);
                let info = self.small.render(&i18n::tf("snake.prestige_info", &[("roman", prestige::roman(self.prestige)), ("blocks", &blocks.to_string())]), true, p.gold);
                s.bl(&info, (10, 54));
            }
        }
        let best = self.small.render(&i18n::tf("snake.best", &[("hs", &self.highscore.to_string())]), true, p.text_dim);
        s.bl(&best, (w - best.w() - 10, 10));
        self.draw_boost_bar(s, 10, self.b.height - 22, 180, 0);
        if !self.b.game_over && self.competitive() {
            self.draw_comp_footer(s);
        } else if !self.b.game_over && self.mode_key() != "timed" {
            self.draw_next_prestige(s);
        }
    }

    fn draw_comp_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let lvl = self.comp_level;
        let line = self.small.render(
            &i18n::tf("snake.comp.stats", &[("apples", &self.apples_total.to_string()), ("level", &lvl.to_string()), ("mult", &competitive::score_multiplier(lvl).to_string())]),
            true,
            COL_FOOD.c(),
        );
        s.bl(&line, (10, 34));
        let groesse = self.snake_size(0);
        let gtxt = format!("{groesse:.2}");
        let gtxt = gtxt.trim_end_matches('0').trim_end_matches('.').to_string();
        let gimg = self.small.render(&i18n::tf("snake.comp.size", &[("size", &gtxt)]), true, p.gold);
        s.bl(&gimg, (10, 54));
        if self.spawn_bonus_t > 0.0 && self.spawn_bonus > 0 {
            let b = self.small.render(&i18n::tf("snake.comp.bonus", &[("n", &self.spawn_bonus.to_string()), ("t", &format!("{:.0}", self.spawn_bonus_t))]), true, COL_BLUE.c());
            s.bl(&b, (10, 74));
        }
        if self.hardcore {
            let puls = 0.5 + 0.5 * (pg::ticks() as f64 * 0.006).sin();
            let col = (255, (70.0 + 80.0 * puls) as i32, (70.0 + 80.0 * puls) as i32);
            let tag = self.small.render(&i18n::t("snake.hardcore_tag"), true, col.c());
            let tr = tag.get_rect().midtop_at((self.b.width / 2, 48));
            let bx = tr.inflate(16, 8);
            let mut glow = pg::surface_alpha(bx.w, bx.h);
            let gr = glow.get_rect();
            draw::rect_r(&mut glow, rgba(COL_HARDCORE.0, COL_HARDCORE.1, COL_HARDCORE.2, (40.0 + 60.0 * puls) as i32), gr, 0, 8);
            s.bl(&glow, (bx.x, bx.y));
            draw::rect_r(s, COL_HARDCORE, bx, 1, 8);
            s.blr(&tag, tr);
        }
    }

    fn draw_comp_footer(&self, s: &mut Surface) {
        let p = ui::pal();
        let Some((have, need)) = competitive::next_step(self.apples_total) else {
            let img = self.small.render(&i18n::tf("snake.comp.max", &[("level", &self.comp_level.to_string())]), true, p.gold);
            s.blr(&img, img.get_rect().midbottom_at((self.b.width / 2, self.b.height - 8)));
            return;
        };
        let (w, y) = (220, self.b.height - 20);
        let x = self.b.width / 2 - w / 2;
        draw::rect_r(s, p.panel, Rect::new(x, y, w, 12), 0, 5);
        let fw = ((w - 2) as f64 * 0f64.max(1f64.min(have as f64 / 1.max(need) as f64))) as i32;
        draw::rect_r(s, p.gold, Rect::new(x + 1, y + 1, fw, 10), 0, 5);
        let lab = self.tiny.render(&i18n::tf("snake.comp.next", &[("level", &(self.comp_level + 1).to_string()), ("have", &have.to_string()), ("need", &need.to_string())]), true, p.text_dim);
        s.blr(&lab, lab.get_rect().midbottom_at((self.b.width / 2, y - 2)));
    }

    fn draw_slot(&self, s: &mut Surface) {
        let p = ui::pal();
        let Some(sl) = self.slot.as_ref() else {
            return;
        };
        let mut ov = pg::surface_alpha(self.b.width, self.b.height);
        ov.fill_c(rgba(0, 0, 0, 170));
        s.bl(&ov, (0, 0));
        let (pw, ph) = (360.min(self.b.width - 40), 260);
        let panel = Rect::new((self.b.width - pw) / 2, (self.b.height - ph) / 2, pw, ph);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, COL_BLUE, panel, 3, 14);
        let title = self.b.font.render(&i18n::t("snake.slot.title"), true, COL_BLUE.c());
        s.blr(&title, title.get_rect().midtop_at((panel.centerx(), panel.y + 12)));
        let stake = self.small.render(&i18n::tf("snake.slot.stake", &[("n", &sl.stake.to_string())]), true, p.text_dim);
        s.blr(&stake, stake.get_rect().midtop_at((panel.centerx(), panel.y + 40)));
        let (rw, rh, gap) = (78, 96, 12);
        let total = rw * 3 + gap * 2;
        let x0 = panel.centerx() - total / 2;
        let ry = panel.y + 74;
        let cell_h = rh / 3;
        let reel = competitive::reel();
        let nr = reel.len() as i64;
        for k in 0..3 {
            let win = Rect::new(x0 + k as i32 * (rw + gap), ry, rw, rh);
            draw::rect_r(s, (16, 18, 30), win, 0, 8);
            let old_clip = s.get_clip();
            s.set_clip(Some(win));
            if sl.t >= SLOT_REEL_STOPS[k] {
                let anchor = reel.iter().position(|r| *r == sl.reels[k]).unwrap_or(0) as i64;
                for row in [-1i64, 0, 1] {
                    let sym = reel[py::pmod(anchor + row, nr) as usize];
                    self.draw_slot_symbol(s, sym, win.centerx(), win.centery() + row as i32 * cell_h, cell_h / 2 - 4);
                }
            } else {
                let base = (sl.t * SLOT_SPIN_SPEED) as i64 + k as i64 * 5;
                let scroll = py::fpmod(sl.t * SLOT_SPIN_SPEED, 1.0) * cell_h as f64;
                for row in [-1i64, 0, 1, 2] {
                    let sym = reel[py::pmod(base + row, nr) as usize];
                    self.draw_slot_symbol(s, sym, win.centerx(), (win.centery() as f64 + (row as i32 * cell_h) as f64 - scroll) as i32, cell_h / 2 - 4);
                }
            }
            s.set_clip(Some(old_clip));
            draw::rect_r(s, p.border, win, 2, 8);
        }
        draw::line(s, COL_BLUE, (x0 - 4, ry + rh / 2), (x0 + total + 4, ry + rh / 2), 1);
        if sl.applied {
            let res = i18n::t(&format!("snake.slot.{}", sl.result));
            let col = if sl.result == "jackpot" {
                p.gold
            } else if sl.result == "pair" {
                p.green
            } else {
                p.text_dim
            };
            let rimg = self.b.font.render(&format!("{res}  x{}", fmt_g(sl.mult)), true, col);
            s.blr(&rimg, rimg.get_rect().midbottom_at((panel.centerx(), panel.bottom() - 14)));
        }
    }

    fn draw_slot_symbol(&self, s: &mut Surface, key: &str, cx: i32, cy: i32, r: i32) {
        let col = competitive::symbol_color(key);
        match key {
            "seven" => {
                let img = self.b.font.render("7", true, col.c());
                s.blr(&img, img.get_rect().center_at((cx, cy)));
            }
            "gem" => {
                let pts = [(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)];
                draw::polygon(s, col, &pts, 0);
                draw::polygon(s, (255, 255, 255), &pts, 1);
            }
            "bell" => {
                draw::circle(s, col, (cx, cy - py::fdiv32(r, 6)), (r as f64 * 0.8) as i32, 0);
                draw::rect_r(s, col, Rect::new(cx - r, cy + py::fdiv32(r, 3), r * 2, 2.max(py::fdiv32(r, 3))), 0, 2);
                draw::circle(s, (40, 40, 30), (cx, cy + py::fdiv32(r, 2)), 1.max(py::fdiv32(r, 5)), 0);
            }
            "apple" => {
                draw::circle(s, col, (cx, cy), (r as f64 * 0.85) as i32, 0);
                draw::circle(s, (255, 180, 180), (cx - py::fdiv32(r, 4), cy - py::fdiv32(r, 4)), 1.max(py::fdiv32(r, 6)), 0);
            }
            _ => {
                for off in [py::fdiv32(-r, 2), py::fdiv32(r, 2)] {
                    draw::circle(s, col, (cx + off, cy + py::fdiv32(r, 3)), (r as f64 * 0.45) as i32, 0);
                }
                draw::line(s, (120, 200, 120), (cx, cy - r), (cx, cy + py::fdiv32(r, 3)), 1);
            }
        }
    }

    fn draw_boost_bar(&self, s: &mut Surface, x: i32, y: i32, w: i32, idx: usize) {
        let p = ui::pal();
        let sn = &self.snakes[idx];
        draw::rect_r(s, p.panel, Rect::new(x, y, w, 14), 0, 5);
        let fw = ((w - 2) as f64 * 0f64.max(1f64.min(sn.stamina))) as i32;
        let col: Color = if sn.boost_on && sn.stamina > 0.0 {
            p.gold
        } else if sn.stamina >= BOOST_MIN_START {
            BOOST_GLOW[idx % 2].c()
        } else {
            rgb(120, 90, 90)
        };
        draw::rect_r(s, col, Rect::new(x + 1, y + 1, fw, 12), 0, 5);
        let lab = self.tiny.render(&i18n::t("snake.boost"), true, rgb(15, 15, 20));
        s.bl(&lab, (x + 6, y));
    }

    fn draw_next_prestige(&self, s: &mut Surface) {
        let p = ui::pal();
        let (req, ok) = self.can_prestige();
        let (txt, col) = match req {
            None => (i18n::tf("snake.max_prestige", &[("roman", prestige::roman(self.prestige))]), p.gold),
            Some(req) => {
                let mut txt = i18n::tf("snake.next_prestige", &[("roman", req.roman), ("apples", &req.apples.to_string()), ("length", &req.length.to_string())]);
                if ok {
                    txt += &i18n::t("snake.press_p");
                    (txt, p.gold)
                } else {
                    (txt, p.text_dim)
                }
            }
        };
        let img = self.small.render(&txt, true, col);
        s.blr(&img, img.get_rect().midbottom_at((self.b.width / 2, self.b.height - 8)));
    }

    fn draw_game_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let mut ov = pg::surface_alpha(self.b.width, self.b.height);
        ov.fill_c(rgba(0, 0, 0, 150));
        s.bl(&ov, (0, 0));
        if self.b.multiplayer {
            let (text, farbe) = match self.winner {
                None => (i18n::t("common.draw"), p.text),
                Some(w) => (i18n::tf("common.player_wins_name", &[("name", &self.b.pname(w).chars().take(10).collect::<String>())]), SNAKE_COLORS[w].1.c()),
            };
            self.b.draw_center_text(&text, &self.b.big_font, farbe, -30);
            let (a0, a1) = (self.snakes[0].apples, self.snakes[1].apples);
            self.b.draw_center_text(&i18n::tf("snake.apples_pp", &[("a0", &a0.to_string()), ("a1", &a1.to_string())]), &self.b.font, p.text_dim, 14);
            self.b.draw_center_text(&i18n::t("common.enter_restart"), &self.b.font, p.text, 48);
            self.draw_replay_hint(78);
            return;
        }
        let titel = if self.mode_key() == "timed" && self.snakes[0].alive { i18n::t("snake.time_up") } else { i18n::t("common.game_over") };
        self.b.draw_center_text(&titel, &self.b.big_font, p.red, -60);
        self.b.draw_center_text(&i18n::tf("snake.apples_collected", &[("n", &self.apples_total.to_string())]), &self.b.font, COL_FOOD.c(), -14);
        if self.competitive() {
            self.b.draw_center_text(&i18n::tf("snake.comp.result", &[("level", &self.comp_level.to_string())]), &self.b.font, p.gold, 16);
        } else if self.prestige > 0 {
            self.b.draw_center_text(&i18n::tf("snake.prestige", &[("roman", prestige::roman(self.prestige))]), &self.b.font, p.gold, 16);
        }
        self.b.draw_center_text(&i18n::t("common.enter_restart"), &self.b.font, p.text, 46);
        self.draw_replay_hint(76);
    }

    fn draw_replay_hint(&self, dy: i32) {
        if self.replay.is_none() || self.rep.is_some() {
            return;
        }
        self.b.draw_center_text(&i18n::t("snake.replay_hint"), &self.small, ui::pal().text_dim, dy);
    }

    // ----- Setup zeichnen ----------------------------------------------

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let title = self.b.big_font.render("SNAKE", true, p.text);
        s.blr(&title, title.get_rect().center_at((w / 2, 52)));
        let modus = if self.b.multiplayer { i18n::t("snake.multiplayer") } else { i18n::t("snake.singleplayer") };
        let sub = self.small.render(&i18n::tf("snake.subtitle", &[("mode", &modus), ("hs", &self.highscore.to_string())]), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((w / 2, 88)));
        let mk = self.mode_key();
        draw::rect_r(s, p.panel_light, self.mode_panel, 0, 10);
        draw::rect_r(s, p.accent, self.mode_panel, 2, 10);
        let name = self.b.font.render(&i18n::t(&format!("snake.mode.{mk}")), true, p.text);
        s.blr(&name, name.get_rect().center_at((self.mode_panel.centerx(), self.mode_panel.y + 18)));
        let desc = self.small.render(&i18n::t(&format!("snake.mode.{mk}.desc")), true, p.text_dim);
        s.blr(&desc, desc.get_rect().center_at((self.mode_panel.centerx(), self.mode_panel.y + 41)));
        for (r, sym) in [(self.mode_left, "<"), (self.mode_right, ">")] {
            let arr = self.b.big_font.render(sym, true, p.accent);
            s.blr(&arr, arr.get_rect().center_at(r.center()));
        }
        let allowed = self.allowed_modes();
        let pos = allowed.iter().position(|m| *m == self.mode_index).unwrap_or(0);
        let dots: Vec<&str> = (0..allowed.len()).map(|i| if i == pos { "*" } else { "." }).collect();
        let d = self.tiny.render(&dots.join(" "), true, p.text_dim);
        s.blr(&d, d.get_rect().center_at((w / 2, self.mode_panel.bottom() + 9)));
        if self.b.multiplayer {
            self.draw_disabled_row(s, self.view_rect, &i18n::t("snake.view_toggle"), &i18n::t("snake.view_mp_hint"));
        } else {
            draw::rect_r(s, if self.view3d { p.btn_sel } else { p.btn }, self.view_rect, 0, 8);
            draw::rect_r(s, if self.view3d { p.accent } else { p.border }, self.view_rect, 1, 8);
            let lab = self.b.font.render(&i18n::t("snake.view_toggle"), true, p.text);
            s.bl(&lab, (self.view_rect.x + 16, self.view_rect.centery() - lab.h() / 2));
            let wert = if self.view3d { i18n::t("snake.view_3d") } else { i18n::t("snake.view_2d") };
            let col = if self.view3d { p.accent } else { p.text_dim };
            let img = self.b.font.render(&format!("< {wert} >"), true, col);
            s.bl(&img, (self.view_rect.right() - img.w() - 16, self.view_rect.centery() - img.h() / 2));
        }
        if self.view3d_active() {
            self.draw_cam_button(s, self.wrap_rect);
        } else {
            self.draw_setup_toggle(s, self.wrap_rect, &i18n::t("snake.wrap_toggle"), self.wrap);
        }
        self.draw_setup_toggle(s, self.bonus_rect, &i18n::t("snake.bonus_toggle"), self.bonus);
        if self.competitive() {
            self.draw_hardcore_toggle(s, self.apples_rect, self.hardcore);
        } else {
            self.draw_setup_value(s, self.apples_rect, &i18n::t("snake.apples_toggle"), &self.apple_count.to_string(), self.apple_count > 1);
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        let st = self.b.font.render(&i18n::t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        if self.view3d_active() {
            let extra = self.tiny.render(&i18n::t("snake.hint_3d"), true, p.accent);
            s.blr(&extra, extra.get_rect().center_at((w / 2, h - 54)));
        }
        let hint = self.small.render(&i18n::t("snake.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 34)));
        if self.competitive() && self.hardcore {
            let puls = 0.5 + 0.5 * (pg::ticks() as f64 * 0.006).sin();
            let hc = self.tiny.render(&i18n::t("snake.hardcore_hint"), true, rgb(255, (80.0 + 60.0 * puls) as i32, (80.0 + 60.0 * puls) as i32));
            s.blr(&hc, hc.get_rect().center_at((w / 2, h - 14)));
        } else {
            let boost = self.tiny.render(&i18n::t("snake.boost_hint"), true, p.green);
            s.blr(&boost, boost.get_rect().center_at((w / 2, h - 14)));
        }
        self.draw_brush_button(s);
    }

    fn draw_brush_button(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.brush_rect;
        draw::rect_r(s, p.btn, r, 0, 8);
        draw::rect_r(s, p.accent, r, 1, 8);
        let tip = ngb::head_color();
        let (hx0, hy0) = (r.x + 10, r.bottom() - 10);
        let (hx1, hy1) = (r.right() - 8, r.y + 8);
        draw::line(s, (170, 140, 95), (hx0 + 4, hy0 - 4), (hx1, hy1), 4);
        let fx = hx0 as f64 + (hx1 - hx0) as f64 * 0.30;
        let fy = hy0 as f64 + (hy1 - hy0) as f64 * 0.30;
        draw::circle(s, (205, 208, 216), (fx as i32, fy as i32), 3, 0);
        draw::polygon(s, tip, &[(hx0 - 4, hy0 + 2), (hx0 + 5, hy0 - 5), (hx0 + 2, hy0 + 6)], 0);
        draw::circle(s, tip, (hx0 - 1, hy0 + 2), 3, 0);
    }

    fn draw_cam_button(&self, s: &mut Surface, rect: Rect) {
        let p = ui::pal();
        draw::rect_r(s, p.btn, rect, 0, 8);
        draw::rect_r(s, p.accent, rect, 1, 8);
        let lab = self.b.font.render(&i18n::t("snake.cam.open"), true, p.text);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let stat = if self.cam_smooth { i18n::t("common.on") } else { i18n::t("common.off") };
        let img = self.b.font.render(&format!("{stat}  \u{203A}"), true, p.accent);
        s.bl(&img, (rect.right() - img.w() - 14, rect.centery() - img.h() / 2));
    }

    fn draw_cam3d(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let title = self.b.big_font.render(&i18n::t("snake.cam.title"), true, p.text);
        s.blr(&title, title.get_rect().center_at((w / 2, 60)));
        let sub = self.small.render(&i18n::t("snake.cam.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((w / 2, 100)));
        self.draw_setup_toggle(s, self.cam_smooth_rect, &i18n::t("snake.cam.smooth"), self.cam_smooth);
        self.draw_cam_value(s, self.cam_fov_rect, &i18n::t("snake.cam.fov"), &format!("{:.2}", self.cam_fov), self.cam_fov_minus, self.cam_fov_plus);
        self.draw_cam_value(s, self.cam_height_rect, &i18n::t("snake.cam.height"), &format!("{:.1}", self.cam_height), self.cam_height_minus, self.cam_height_plus);
        self.draw_setup_toggle(s, self.cam_turn_rect, &i18n::t("snake.cam.turn_shake"), self.cam_turn_shake);
        draw::rect_r(s, p.btn_sel, self.cam_back_rect, 0, 10);
        let bt = self.b.font.render(&i18n::t("snake.cam.back"), true, p.text);
        s.blr(&bt, bt.get_rect().center_at(self.cam_back_rect.center()));
        let hint = self.tiny.render(&i18n::t("snake.cam.hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 16)));
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_cam_value(&self, s: &mut Surface, rect: Rect, label: &str, value: &str, minus: Rect, plus: Rect) {
        let p = ui::pal();
        draw::rect_r(s, p.btn, rect, 0, 8);
        draw::rect_r(s, p.border, rect, 1, 8);
        let lab = self.b.font.render(label, true, p.text);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        for (r, sym) in [(minus, "-"), (plus, "+")] {
            draw::rect_r(s, p.btn_sel, r, 0, 6);
            let g = self.b.font.render(sym, true, p.text);
            s.blr(&g, g.get_rect().center_at(r.center()));
        }
        let val = self.b.font.render(value, true, p.green);
        s.blr(&val, val.get_rect().center_at(((minus.right() + plus.x) / 2, rect.centery())));
    }

    fn draw_disabled_row(&self, s: &mut Surface, rect: Rect, label: &str, note: &str) {
        let p = ui::pal();
        draw::rect_r(s, p.panel, rect, 0, 8);
        draw::rect_r(s, p.border, rect, 1, 8);
        let lab = self.b.font.render(label, true, p.text_dim);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let info = self.small.render(note, true, p.text_dim);
        s.bl(&info, (rect.right() - info.w() - 16, rect.centery() - info.h() / 2));
    }

    fn draw_hardcore_toggle(&self, s: &mut Surface, rect: Rect, an: bool) {
        let p = ui::pal();
        let (lab_col, val_col): (Color, Color);
        if an {
            let puls = 0.5 + 0.5 * (pg::ticks() as f64 * 0.006).sin();
            let mut glow = pg::surface_alpha(rect.w + 22, rect.h + 22);
            let gr = glow.get_rect();
            draw::rect_r(&mut glow, rgba(COL_HARDCORE.0, COL_HARDCORE.1, COL_HARDCORE.2, (55.0 + 90.0 * puls) as i32), gr, 0, 14);
            s.blr(&glow, glow.get_rect().center_at(rect.center()));
            draw::rect_r(s, (95, 18, 22), rect, 0, 8);
            draw::rect_r(s, COL_HARDCORE, rect, 2, 8);
            lab_col = rgb(255, 210, 210);
            val_col = rgb(255, 120, 120);
        } else {
            draw::rect_r(s, p.btn, rect, 0, 8);
            draw::rect_r(s, p.border, rect, 1, 8);
            lab_col = p.text;
            val_col = p.text_dim;
        }
        let lab = self.b.font.render(&i18n::t("snake.hardcore_toggle"), true, lab_col);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let wert = if an { i18n::t("common.on") } else { i18n::t("common.off") };
        let img = self.b.font.render(&format!("< {wert} >"), true, val_col);
        s.bl(&img, (rect.right() - img.w() - 16, rect.centery() - img.h() / 2));
    }

    fn draw_setup_toggle(&self, s: &mut Surface, rect: Rect, label: &str, an: bool) {
        let wert = if an { i18n::t("common.on") } else { i18n::t("common.off") };
        self.draw_setup_value(s, rect, label, &wert, an);
    }

    fn draw_setup_value(&self, s: &mut Surface, rect: Rect, label: &str, wert: &str, hervor: bool) {
        let p = ui::pal();
        draw::rect_r(s, if hervor { p.btn_sel } else { p.btn }, rect, 0, 8);
        draw::rect_r(s, p.border, rect, 1, 8);
        let lab = self.b.font.render(label, true, p.text);
        s.bl(&lab, (rect.x + 16, rect.centery() - lab.h() / 2));
        let col = if hervor { p.green } else { p.text_dim };
        let img = self.b.font.render(&format!("< {wert} >"), true, col);
        s.bl(&img, (rect.right() - img.w() - 16, rect.centery() - img.h() / 2));
    }
}

impl Game for SnakeGame {
    fn replay_begin(&mut self, rep: &Value) -> bool {
        SnakeGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        SnakeGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        let _ = aiming;
        SnakeGame::replay_draw(self, banner);
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
        self.winner = None;
        self.effects_done = false;
        self.cols = self.b.width / CELL;
        self.rows = self.b.height / CELL;
        self.wrap = self.opt_truthy("wrap", false);
        self.bonus = self.opt_truthy("bonus_apple", false);
        self.hardcore = self.opt_truthy("hardcore", false);
        let ac = self.b.opt_int_py("snake", "apples", 1).unwrap_or(1);
        self.apple_count = if APPLE_COUNTS.contains(&ac) { ac } else { 1 };
        let mk = self.b.opt_value("snake", "mode").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "classic".into());
        self.mode_index = MODE_KEYS.iter().position(|k| *k == mk).unwrap_or(0);
        self.view3d = self.opt_truthy("view3d", false);
        self.cam_smooth = self.opt_truthy("cam_smooth", true);
        self.cam_fov = CAM_FOV_MAX.min(CAM_FOV_MIN.max(self.b.opt_float_py("snake", "cam_fov", FOV_NORMAL).unwrap_or(FOV_NORMAL)));
        self.cam_height = CAM_H_MAX.min(CAM_H_MIN.max(self.b.opt_float_py("snake", "cam_height", CAM_H).unwrap_or(CAM_H)));
        self.cam_turn_shake = self.opt_truthy("cam_turn_shake", false);
        if self.view3d_active() && !MODES_3D.contains(&self.mode_key()) {
            self.mode_index = Self::mode_idx("classic");
        }
        if self.b.multiplayer && self.mode_key() == "competitive" {
            self.mode_index = Self::mode_idx("classic");
        }
        self.small = ui::font(16, false, false);
        self.tiny = ui::font(13, false, false);
        self.highscore = highscore::get(self.b.key);
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.rep_at = None;
        self.rep_state.clear();
        self.rec_delta = replay::Delta::new();
        self.rec_moves.clear();
        self.rec_last.clear();
        self.rec_layout = 0;
        self.rec_n = 0;
        self.particles.clear();
        self.anim_t = 0.0;
        self.ngb_menu = None;
        self.grid_surf = None;
        self.grid_key = None;
        self.grid_font = ui::font(9.max(CELL - 8), false, true);
        self.particles3d.clear();
        self.particles3d_col.clear();
        self.shake = 0.0;
        self.sky_cache = None;
        self.stars = (0..70)
            .map(|_| {
                let az = py::uniform(0.0, TAU);
                let hf = py::random();
                let size = py::choice(&[1, 1, 2]);
                let ph = py::uniform(0.0, TAU);
                (az, hf, size, ph)
            })
            .collect();
        self.build_setup_layout();
        self.reset_run_stats();
        self.new_board();
        self.state = St::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.b.font = ui::font(22, false, false);
        self.b.big_font = ui::font(48, true, false);
        self.small = ui::font(16, false, false);
        self.tiny = ui::font(13, false, false);
        self.grid_font = ui::font(9.max(CELL - 8), false, true);
        self.grid_surf = None;
        self.grid_key = None;
        self.sky_cache = None;
        self.build_setup_layout();
    }

    fn take_replay_request(&mut self) -> Option<Value> {
        self.replay_request.take()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == St::Personalize {
            if let Some(menu) = self.ngb_menu.as_mut() {
                menu.handle_event(ev);
                let sounds = menu.take_sounds();
                let done = menu.done;
                for snd in sounds {
                    self.b.play_sound(snd);
                }
                if done {
                    self.ngb_menu = None;
                    self.state = St::Setup;
                }
            }
            return;
        }
        if self.state == St::Cam3d {
            self.handle_cam3d_event(ev);
            return;
        }
        if self.state == St::Setup {
            self.handle_setup_event(ev);
            return;
        }
        if self.slot.is_some() {
            return;
        }
        let k = ev.key.as_str();
        if ev.kind == Kind::KeyUp {
            if self.b.multiplayer {
                if BOOST_KEYS_P1.contains(&k) {
                    self.set_boost(0, false);
                } else if BOOST_KEYS_P2.contains(&k) {
                    self.set_boost(1, false);
                }
            } else if BOOST_KEYS_P1.contains(&k) || BOOST_KEYS_P2.contains(&k) {
                self.set_boost(0, false);
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        if self.b.game_over {
            if k == "Return" || k == "space" {
                self.start_play();
            } else if (k == "p" || k == "P") && self.replay.is_some() {
                self.open_replay();
            }
            return;
        }
        if self.b.multiplayer {
            if BOOST_KEYS_P1.contains(&k) {
                self.set_boost(0, true);
                return;
            }
            if BOOST_KEYS_P2.contains(&k) {
                self.set_boost(1, true);
                return;
            }
        } else if BOOST_KEYS_P1.contains(&k) || BOOST_KEYS_P2.contains(&k) {
            self.set_boost(0, true);
            return;
        }
        if k == "p" || k == "P" {
            self.try_prestige();
            return;
        }
        if self.b.multiplayer {
            self.turn(0, k, Some("p1"));
            self.turn(1, k, Some("p2"));
        } else if self.view3d_active() {
            self.steer_3d(k);
        } else {
            self.turn(0, k, None);
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        self.update_particles(dt);
        self.update_particles3d(dt);
        self.update_food_anim(dt);
        self.update_float_texts(dt);
        self.update_banner(dt);
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - dt * 1.6);
        }
        if self.state == St::Play && self.view3d_active() {
            self.update_camera(dt);
        }
        if self.state != St::Play || self.b.game_over {
            return;
        }
        if self.slot.is_some() {
            self.update_slot(dt);
            return;
        }
        self.run_t += dt;
        if self.competitive() {
            self.update_competitive(dt);
        }
        if let Some((gx, gy)) = self.golden {
            self.golden_timer -= dt;
            if self.golden_timer <= 0.0 {
                self.golden = None;
            } else if self.view3d_active() && py::random() < dt * 6.0 {
                let x = gx as f64 + 0.5 + py::uniform(-0.2, 0.2);
                let z = gy as f64 + 0.5 + py::uniform(-0.2, 0.2);
                let vx = py::uniform(-0.4, 0.4);
                let vy = py::uniform(0.8, 1.8);
                let vz = py::uniform(-0.4, 0.4);
                self.particles3d.push([x, 0.55, z, vx, vy, vz, 0.55]);
                self.particles3d_col.push(COL_GOLD);
            }
        }
        if self.mode_key() == "timed" {
            self.time_left -= dt;
            if self.time_left <= 0.0 {
                self.time_left = 0.0;
                self.finish_timed();
                return;
            }
        }
        for sn in self.snakes.iter_mut() {
            if sn.alive && !sn.boost_on {
                sn.stamina = STAMINA_MAX.min(sn.stamina + STAMINA_REGEN * dt);
            }
        }
        self.timer += dt;
        let mut guard = 0;
        while self.timer >= self.interval && !self.b.game_over && self.slot.is_none() {
            self.timer -= self.interval;
            self.tick();
            guard += 1;
            if guard > 8 {
                self.timer = 0.0;
                break;
            }
        }
        self.rec_tick(dt);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.state == St::Personalize {
            if let Some(m) = self.ngb_menu.as_mut() {
                m.draw(&mut s);
            }
            return;
        }
        if self.state == St::Cam3d {
            self.draw_cam3d(&mut s);
            return;
        }
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            return;
        }
        if self.view3d_active() {
            self.draw_world_3d(&mut s);
        } else {
            self.draw_world_2d(&mut s);
        }
        self.draw_hud(&mut s);
        self.draw_banner(&mut s);
        if self.slot.is_some() {
            self.draw_slot(&mut s);
        }
        if self.b.game_over {
            self.draw_game_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "snake",
    class_name: "SnakeGame",
    supports_multiplayer: true,
    modes: &[],
    make: SnakeGame::new,
};
