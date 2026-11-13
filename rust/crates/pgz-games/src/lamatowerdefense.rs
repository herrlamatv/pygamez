//! lamatowerdefense.rs
//! ===================
//! Tower Defense - Wellen abwehren, Tuerme bauen und ausbauen.
//!
//! Uebertragen aus `games/lamatowerdefense.py`: 4 Karten, Endlos-Wellen aus
//! einem Budget-Komponisten, 3 Modi, 11 Turmtypen mit Stufen und A/B-Zweigen,
//! 11 Gegnertypen (Boss, Flieger, Tarnung, Heiler, Teiler ...), Oekonomie
//! mit Zinsen und Aktiv-Faehigkeiten (Maximal).

use std::collections::HashSet;
use std::f64::consts::{PI, TAU};

use pgz_core::store;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const GRID_W: i32 = 18;
const GRID_H: i32 = 12;
const MAX_ENEMIES: usize = 90;
const MAX_SHOTS: usize = 120;
const MAX_FX: usize = 150;
const UNITS_PER_WAVE: usize = 140;

type Rgb = (i32, i32, i32);

const COL_GRASS1: Rgb = (52, 74, 48);
const COL_GRASS2: Rgb = (47, 68, 44);
const COL_PATH: Rgb = (122, 100, 70);
const COL_PATH_EDGE: Rgb = (86, 70, 50);
const COL_BUILD_OK: Rgb = (110, 220, 130);
const COL_BUILD_BAD: Rgb = (235, 100, 90);
const COL_HP_BG: Rgb = (40, 30, 30);
const COL_HP: Rgb = (110, 220, 110);
const COL_AIR_SHADOW: Rgb = (25, 30, 22);

#[derive(Clone, Copy)]
struct Rules {
    letter: char,
    levels: i64,
    gold: i64,
    lives: i64,
    boss: i64,
    branch: bool,
    abil: bool,
}

fn rules_for(mode: &str) -> Rules {
    match mode {
        "compact" => Rules { letter: 'c', levels: 2, gold: 260, lives: 20, boss: 8, branch: false, abil: false },
        "maximal" => Rules { letter: 'M', levels: 3, gold: 220, lives: 20, boss: 8, branch: true, abil: true },
        _ => Rules { letter: 'C', levels: 3, gold: 220, lives: 20, boss: 8, branch: false, abil: false },
    }
}

#[derive(Clone, Copy, Default)]
struct TowerDef {
    cost: i64,
    dmg: f64,
    rng: f64,
    cd: f64,
    proj: Option<f64>,
    modes: &'static str,
    col: Rgb,
    splash: f64,
    slow: f64,
    slow_t: f64,
    detect: bool,
    hitscan: bool,
    air: bool,
    strong: bool,
    pierce: bool,
    dot: f64,
    dot_t: f64,
    chain: i64,
    buff: f64,
    min_rng: f64,
    lob: bool,
    air_only: bool,
    beam: bool,
    income: f64,
}

const TOWER_KEYS: [&str; 11] = ["arrow", "cannon", "frost", "sniper", "poison", "tesla", "support", "mortar", "flak", "laser", "bank"];

fn tower_def(k: &str) -> TowerDef {
    let d = TowerDef::default();
    match k {
        "arrow" => TowerDef { cost: 50, dmg: 8.0, rng: 2.6, cd: 0.55, proj: Some(9.0), modes: "cCM", col: (240, 205, 100), ..d },
        "cannon" => TowerDef { cost: 90, dmg: 18.0, rng: 2.2, cd: 1.40, proj: Some(7.0), modes: "cCM", col: (205, 125, 85), splash: 0.9, ..d },
        "frost" => TowerDef { cost: 70, dmg: 2.0, rng: 2.0, cd: 0.80, proj: Some(8.0), modes: "cCM", col: (125, 200, 250), slow: 0.45, slow_t: 1.6, detect: true, ..d },
        "sniper" => TowerDef { cost: 120, dmg: 45.0, rng: 5.5, cd: 2.60, modes: "cCM", col: (180, 230, 140), hitscan: true, air: true, detect: true, strong: true, pierce: true, ..d },
        "poison" => TowerDef { cost: 85, dmg: 4.0, rng: 2.2, cd: 0.90, proj: Some(8.0), modes: "CM", col: (150, 220, 90), dot: 12.0, dot_t: 3.0, ..d },
        "tesla" => TowerDef { cost: 130, dmg: 14.0, rng: 2.0, cd: 1.10, modes: "CM", col: (150, 175, 255), chain: 3, air: true, ..d },
        "support" => TowerDef { cost: 100, dmg: 0.0, rng: 1.8, cd: 0.0, modes: "CM", col: (255, 170, 220), buff: 0.20, detect: true, ..d },
        "mortar" => TowerDef { cost: 140, dmg: 30.0, rng: 8.5, cd: 3.00, modes: "M", col: (190, 190, 170), splash: 1.2, min_rng: 2.2, lob: true, ..d },
        "flak" => TowerDef { cost: 95, dmg: 22.0, rng: 3.0, cd: 0.90, proj: Some(10.0), modes: "M", col: (255, 140, 120), air: true, air_only: true, ..d },
        "laser" => TowerDef { cost: 160, dmg: 26.0, rng: 2.8, cd: 0.0, modes: "M", col: (255, 95, 95), beam: true, air: true, ..d },
        _ => TowerDef { cost: 110, dmg: 0.0, rng: 0.0, cd: 0.0, modes: "M", col: (250, 215, 110), income: 12.0, ..d },
    }
}

const BRANCH_COST: f64 = 1.6;

#[derive(Clone, Copy, Default)]
struct EnemyDef {
    hp: f64,
    spd: f64,
    bounty: f64,
    score: i64,
    r: f64,
    modes: &'static str,
    intro: i64,
    col: Rgb,
    group: usize,
    armor: f64,
    fly: bool,
    regen: f64,
    stealth: bool,
    splits: i64,
    heal: f64,
    boss: bool,
    leak: i64,
}

const ENEMY_KEYS: [&str; 11] = ["runt", "fast", "swarm", "tank", "shield", "flyer", "regen", "stealth", "splitter", "healer", "boss"];

fn enemy_def(k: &str) -> EnemyDef {
    let d = EnemyDef { group: 1, leak: 1, ..Default::default() };
    match k {
        "runt" => EnemyDef { hp: 22.0, spd: 1.9, bounty: 4.0, score: 10, r: 0.26, modes: "cCM", intro: 1, col: (235, 120, 90), ..d },
        "fast" => EnemyDef { hp: 14.0, spd: 3.2, bounty: 5.0, score: 12, r: 0.22, modes: "cCM", intro: 2, col: (255, 180, 80), ..d },
        "swarm" => EnemyDef { hp: 8.0, spd: 2.4, bounty: 2.0, score: 6, r: 0.17, modes: "cCM", intro: 4, group: 6, col: (250, 220, 110), ..d },
        "tank" => EnemyDef { hp: 95.0, spd: 1.05, bounty: 9.0, score: 18, r: 0.34, modes: "cCM", intro: 5, col: (185, 100, 145), ..d },
        "shield" => EnemyDef { hp: 60.0, spd: 1.5, bounty: 8.0, score: 16, r: 0.29, modes: "CM", intro: 6, armor: 6.0, col: (150, 160, 220), ..d },
        "flyer" => EnemyDef { hp: 40.0, spd: 2.2, bounty: 8.0, score: 16, r: 0.25, modes: "M", intro: 7, fly: true, col: (160, 205, 255), ..d },
        "regen" => EnemyDef { hp: 70.0, spd: 1.4, bounty: 8.0, score: 16, r: 0.29, modes: "CM", intro: 9, regen: 4.0, col: (130, 215, 140), ..d },
        "stealth" => EnemyDef { hp: 55.0, spd: 1.7, bounty: 9.0, score: 18, r: 0.27, modes: "M", intro: 10, stealth: true, col: (175, 175, 195), ..d },
        "splitter" => EnemyDef { hp: 48.0, spd: 1.6, bounty: 6.0, score: 14, r: 0.29, modes: "CM", intro: 11, splits: 2, col: (230, 150, 200), ..d },
        "healer" => EnemyDef { hp: 65.0, spd: 1.3, bounty: 10.0, score: 18, r: 0.29, modes: "M", intro: 12, heal: 6.0, col: (120, 230, 200), ..d },
        _ => EnemyDef { hp: 900.0, spd: 0.85, bounty: 60.0, score: 100, r: 0.5, modes: "cCM", intro: 0, boss: true, leak: 5, col: (255, 90, 110), ..d },
    }
}

struct MapDef {
    id: &'static str,
    diff: f64,
    unlock: i64,
    ring: bool,
    path: &'static [(i32, i32)],
}

const MAPS: [MapDef; 4] = [
    MapDef { id: "meadow", diff: 1.00, unlock: 0, ring: false, path: &[(-1, 3), (13, 3), (13, 7), (4, 7), (4, 10), (18, 10)] },
    MapDef { id: "canyon", diff: 1.70, unlock: 5, ring: false, path: &[(-1, 1), (16, 1), (16, 4), (1, 4), (1, 7), (16, 7), (16, 10), (-1, 10)] },
    MapDef { id: "crossing", diff: 1.85, unlock: 10, ring: false, path: &[(-1, 6), (8, 6), (8, 2), (14, 2), (14, 9), (8, 9), (8, 6), (18, 6)] },
    MapDef { id: "gauntlet", diff: 1.20, unlock: 15, ring: true, path: &[(-1, 5), (8, 5), (8, 8), (18, 8)] },
];

fn budget(n: i64) -> f64 {
    45.0 + 12.0 * n as f64 + 1.6 * n as f64 * n as f64
}

fn hp_mult(n: i64) -> f64 {
    let mut m = 1.0 + 0.14 * (n - 1) as f64 + 0.012 * py::pow((n - 1) as f64, 2.0);
    if n > 25 {
        m *= py::pow(1.06, (n - 25) as f64);
    }
    m
}

fn bounty_mult(n: i64) -> f64 {
    0.35f64.max(1.0 - 0.018 * (n - 1) as f64)
}

fn map_hpf(n: i64, diff: f64) -> f64 {
    1.0 + (diff - 1.0) * 1f64.min(n as f64 / 10.0)
}

fn unit_cost(d: &EnemyDef) -> f64 {
    d.hp / 10.0 + d.spd * 4.0
}

fn sq(x: f64) -> f64 {
    py::pow(x, 2.0)
}

#[derive(Clone, Copy, Default)]
struct Stats {
    dmg: f64,
    rng: f64,
    cd: f64,
    splash: f64,
    slow: f64,
    slow_t: f64,
    dot: f64,
    dot_t: f64,
    chain: i64,
    buff: f64,
    income: f64,
    air: bool,
    air_only: bool,
    targets: usize,
    execute: f64,
    stun: f64,
    aura_dps: f64,
    ramp_max: f64,
    ramp_rate: f64,
    splits_dot: bool,
    acid: bool,
    ground_frac: f64,
    cluster: usize,
    buff_cd: bool,
}

struct Tower {
    kind: &'static str,
    cell: (i32, i32),
    level: i64,
    branch: Option<char>,
    invested: i64,
    cd_left: f64,
    angle: f64,
    ramp: f64,
    beam: Option<u64>,
    buff_dmg: f64,
    buff_cd_f: f64,
    st: Stats,
}

#[derive(Clone)]
struct Enemy {
    id: u64,
    kind: &'static str,
    hp: f64,
    maxhp: f64,
    d: f64,
    spd: f64,
    r: f64,
    armor: f64,
    regen: f64,
    fly: bool,
    stealth: bool,
    heal: f64,
    splits: i64,
    boss: bool,
    leak: i64,
    bounty: f64,
    score: i64,
    col: Rgb,
    slow_t: f64,
    slow_f: f64,
    stun_t: f64,
    dot_t: f64,
    dot: f64,
    seen: bool,
    dead: bool,
    plague: bool,
}

enum Shot {
    Lob { x: f64, y: f64, tx: f64, ty: f64, t: f64, dur: f64, dmg: f64, splash: f64, stun: f64, col: Rgb },
    Proj { x: f64, y: f64, tgt: Option<u64>, lx: f64, ly: f64, spd: f64, dmg: f64, splash: f64, slow: f64, slow_t: f64, dot: f64, dot_t: f64, stun: f64, acid: bool, splits_dot: bool, col: Rgb },
}

enum FxKind {
    Tracer((f64, f64), (f64, f64), Rgb),
    Zigzag(Vec<(f64, f64)>, Rgb),
    Ring((f64, f64), f64, Rgb),
    Burst((f64, f64), Rgb),
    Nova((f64, f64)),
}

struct Fx {
    kind: FxKind,
    age: f64,
    ttl: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    MapSel,
    Build,
    Wave,
    GameOver,
}

const ABIL_KEYS: [&str; 3] = ["meteor", "nova", "gold"];
const ABIL_CDS: [f64; 3] = [45.0, 60.0, 90.0];

/// `LamaTowerDefenseGame`
pub struct LamaTowerDefenseGame {
    b: GameBase,
    anim_t: f64,
    rules: Rules,
    bar_keys: Vec<&'static str>,
    best: Vec<(String, i64)>,
    map_idx: usize,
    towers: Vec<Tower>,
    enemies: Vec<Enemy>,
    next_id: u64,
    shots: Vec<Shot>,
    fx: Vec<Fx>,
    spawn_queue: Vec<(&'static str, f64)>,
    spawn_t: f64,
    hp_bonus: f64,
    wave: i64,
    gold: i64,
    lives: i64,
    kills: i64,
    ff: i32,
    show_ranges: bool,
    sel_card: Option<&'static str>,
    sel_tower: Option<(i32, i32)>,
    armed: Option<&'static str>,
    abil_cd: [f64; 3],
    mapsel_idx: usize,
    mouse: Option<(i32, i32)>,
    bar_scroll: i32,
    over_t: f64,
    bg_cache: Option<((usize, i32, i32, i32), Surface)>,
    ui_bg_cache: Option<((i32, i32, Color, Color), Surface)>,
    speed_rect: Option<Rect>,
    small: Font,
    tiny: Font,
    hud_h: i32,
    bar_w: i32,
    cell: i32,
    ox: i32,
    oy: i32,
    field_rect: Rect,
    bar_rect: Rect,
    abil_rects: Vec<(usize, Rect)>,
    card_rects: Vec<Rect>,
    cards_top: i32,
    cards_h: i32,
    cards_view_h: i32,
    start_rect: Rect,
    info_rect: Rect,
    up_rect: Rect,
    sell_rect: Rect,
    br_a_rect: Rect,
    br_b_rect: Rect,
    map_rects: Vec<Rect>,
    path_pts: Vec<(f64, f64)>,
    path_cum: Vec<f64>,
    path_total: f64,
    fly_a: (f64, f64),
    fly_b: (f64, f64),
    fly_total: f64,
    path_cells: HashSet<(i32, i32)>,
    build_cells: Option<Vec<(i32, i32)>>,
    state: State,
}

impl LamaTowerDefenseGame {
    /// `LamaTowerDefenseGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = LamaTowerDefenseGame {
            b,
            anim_t: 0.0,
            rules: rules_for("classic"),
            bar_keys: Vec::new(),
            best: Vec::new(),
            map_idx: 0,
            towers: Vec::new(),
            enemies: Vec::new(),
            next_id: 0,
            shots: Vec::new(),
            fx: Vec::new(),
            spawn_queue: Vec::new(),
            spawn_t: 0.0,
            hp_bonus: 1.0,
            wave: 1,
            gold: 0,
            lives: 0,
            kills: 0,
            ff: 1,
            show_ranges: false,
            sel_card: None,
            sel_tower: None,
            armed: None,
            abil_cd: [0.0; 3],
            mapsel_idx: 0,
            mouse: None,
            bar_scroll: 0,
            over_t: 0.0,
            bg_cache: None,
            ui_bg_cache: None,
            speed_rect: None,
            small: f.clone(),
            tiny: f,
            hud_h: 34,
            bar_w: 148,
            cell: 10,
            ox: 0,
            oy: 0,
            field_rect: z,
            bar_rect: z,
            abil_rects: Vec::new(),
            card_rects: Vec::new(),
            cards_top: 0,
            cards_h: 0,
            cards_view_h: 40,
            start_rect: z,
            info_rect: z,
            up_rect: z,
            sell_rect: z,
            br_a_rect: z,
            br_b_rect: z,
            map_rects: Vec::new(),
            path_pts: Vec::new(),
            path_cum: Vec::new(),
            path_total: 0.0,
            fly_a: (0.0, 0.0),
            fly_b: (0.0, 0.0),
            fly_total: 0.0,
            path_cells: HashSet::new(),
            build_cells: None,
            state: State::MapSel,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.b.font = ui::f(18.max(26.min(h / 26)));
        self.b.big_font = ui::fb(30.max(50.min(h / 13)));
        self.small = ui::f(13.max(19.min(h / 34)));
        self.tiny = ui::f(11.max(15.min(h / 44)));
    }

    fn layout(&mut self) {
        self.make_fonts();
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 34.max(h / 13);
        self.bar_w = 148.max((w as f64 * 0.20) as i32);
        let avail_w = w - self.bar_w;
        let avail_h = h - self.hud_h;
        self.cell = 10.max((avail_w / GRID_W).min(avail_h / GRID_H));
        self.ox = (avail_w - self.cell * GRID_W) / 2;
        self.oy = self.hud_h + (avail_h - self.cell * GRID_H) / 2;
        self.field_rect = Rect::new(self.ox, self.oy, self.cell * GRID_W, self.cell * GRID_H);
        self.bar_rect = Rect::new(w - self.bar_w, 0, self.bar_w, h);
        self.build_bar_layout();
        self.build_mapsel_layout();
        self.setup_path(self.map_idx);
        if self.mouse.is_none() {
            self.mouse = Some(self.field_rect.center());
        }
    }

    fn build_bar_layout(&mut self) {
        let bx = self.bar_rect.x + 10;
        let bw = self.bar_w - 20;
        let mut y = 12;
        self.abil_rects.clear();
        if self.rules.abil {
            let aw = (bw - 12) / 3;
            for i in 0..3 {
                self.abil_rects.push((i, Rect::new(bx + i as i32 * (aw + 6), y, aw, 30.max(self.cell))));
            }
            y += 30.max(self.cell) + 10;
        }
        let cw = (bw - 8) / 2;
        let ch = 44.max((self.cell as f64 * 1.35) as i32);
        self.card_rects = (0..self.bar_keys.len() as i32).map(|i| Rect::new(bx + (i % 2) * (cw + 8), y + (i / 2) * (ch + 8), cw, ch)).collect();
        self.cards_top = y;
        let rows = (self.bar_keys.len() as i32 + 1) / 2;
        self.cards_h = rows * (ch + 8);
        let bh = 40.max((self.b.height as f64 * 0.07) as i32);
        self.start_rect = Rect::new(bx, self.b.height - bh - 12, bw, bh);
        let ih = 120.max((self.b.height as f64 * 0.30) as i32);
        self.info_rect = Rect::new(bx, self.start_rect.y - ih - 10, bw, ih);
        self.cards_view_h = 40.max(self.info_rect.y - y - 8);
        let byy = self.info_rect.bottom() - 34;
        let half = (bw - 6) / 2;
        self.up_rect = Rect::new(bx, byy - 38, bw, 32);
        self.sell_rect = Rect::new(bx, byy, bw, 30);
        self.br_a_rect = Rect::new(bx, byy - 38, half, 32);
        self.br_b_rect = Rect::new(bx + half + 6, byy - 38, half, 32);
    }

    fn build_mapsel_layout(&mut self) {
        let cw = 300.min((self.b.width - 80) / 2);
        let ch = 190.min((self.b.height - 170) / 2);
        let cx = self.b.width / 2;
        let y0 = 120.max((self.b.height as f64 * 0.24) as i32);
        self.map_rects = (0..MAPS.len() as i32).map(|i| Rect::new(cx - cw - 14 + (i % 2) * (cw + 28), y0 + (i / 2) * (ch + 22), cw, ch)).collect();
    }

    fn setup_path(&mut self, idx: usize) {
        let m = &MAPS[idx];
        let wps = m.path;
        self.path_pts = wps.iter().map(|&(c, r)| (c as f64 + 0.5, r as f64 + 0.5)).collect();
        self.path_cum = vec![0.0];
        let mut total = 0.0;
        for w in self.path_pts.windows(2) {
            total += (w[1].0 - w[0].0).abs() + (w[1].1 - w[0].1).abs();
            self.path_cum.push(total);
        }
        self.path_total = total;
        self.fly_a = self.path_pts[0];
        self.fly_b = *self.path_pts.last().unwrap_or(&self.fly_a);
        self.fly_total = py::hypot(self.fly_b.0 - self.fly_a.0, self.fly_b.1 - self.fly_a.1);
        let mut cells = HashSet::new();
        for w in wps.windows(2) {
            let ((c0, r0), (c1, r1)) = (w[0], w[1]);
            if c0 == c1 {
                for r in r0.min(r1)..=r0.max(r1) {
                    cells.insert((c0, r));
                }
            } else {
                for c in c0.min(c1)..=c0.max(c1) {
                    cells.insert((c, r0));
                }
            }
        }
        self.path_cells = cells.into_iter().filter(|&(c, r)| (0..GRID_W).contains(&c) && (0..GRID_H).contains(&r)).collect();
        if m.ring {
            let mut ring = HashSet::new();
            for &(c, r) in &self.path_cells {
                for dc in -1..=1 {
                    for dr in -1..=1 {
                        ring.insert((c + dc, r + dr));
                    }
                }
            }
            let mut bc: Vec<(i32, i32)> = ring.into_iter().filter(|p| !self.path_cells.contains(p) && (0..GRID_W).contains(&p.0) && (0..GRID_H).contains(&p.1)).collect();
            bc.sort_unstable();
            self.build_cells = Some(bc);
        } else {
            self.build_cells = None;
        }
    }

    fn pos_at(&self, d: f64, fly: bool) -> (f64, f64) {
        if fly {
            let f = 0f64.max(1f64.min(d / 0.001f64.max(self.fly_total)));
            return (self.fly_a.0 + (self.fly_b.0 - self.fly_a.0) * f, self.fly_a.1 + (self.fly_b.1 - self.fly_a.1) * f);
        }
        let (pts, cum) = (&self.path_pts, &self.path_cum);
        if d <= 0.0 {
            return pts[0];
        }
        if d >= *cum.last().unwrap_or(&0.0) {
            return *pts.last().unwrap_or(&pts[0]);
        }
        for i in 1..cum.len() {
            if d <= cum[i] {
                let f = (d - cum[i - 1]) / 0.001f64.max(cum[i] - cum[i - 1]);
                let (x0, y0) = pts[i - 1];
                let (x1, y1) = pts[i];
                return (x0 + (x1 - x0) * f, y0 + (y1 - y0) * f);
            }
        }
        *pts.last().unwrap_or(&pts[0])
    }

    fn px(&self, cx: f64, cy: f64) -> (f64, f64) {
        (self.ox as f64 + cx * self.cell as f64, self.oy as f64 + cy * self.cell as f64)
    }

    // ===================================================== Lauf-Verwaltung

    fn best_of(&self, id: &str) -> i64 {
        self.best.iter().find(|(k, _)| k == id).map(|(_, v)| *v).unwrap_or(0)
    }

    fn unlocked(&self, m: &MapDef) -> bool {
        let top = self.best.iter().map(|(_, v)| *v).max().unwrap_or(0);
        m.unlock <= top
    }

    fn start_run(&mut self, idx: usize) {
        self.map_idx = idx;
        self.setup_path(idx);
        self.bg_cache = None;
        self.towers.clear();
        self.enemies.clear();
        self.shots.clear();
        self.fx.clear();
        self.spawn_queue.clear();
        self.b.score = 0;
        self.kills = 0;
        self.wave = 1;
        self.gold = self.rules.gold;
        self.lives = self.rules.lives;
        self.ff = 1;
        self.show_ranges = false;
        self.sel_card = None;
        self.sel_tower = None;
        self.armed = None;
        self.abil_cd = [0.0; 3];
        self.bar_scroll = 0;
        self.b.game_over = false;
        self.state = State::Build;
        self.b.play_sound("select");
    }

    fn save_best(&mut self) {
        let cleared = self.wave - 1;
        let mid = MAPS[self.map_idx].id;
        if cleared > self.best_of(mid) {
            match self.best.iter_mut().find(|(k, _)| k == mid) {
                Some(e) => e.1 = cleared,
                None => self.best.push((mid.to_string(), cleared)),
            }
            let mut m = Map::new();
            for (k, v) in &self.best {
                m.insert(k.clone(), json!(v));
            }
            let mut sec = Map::new();
            sec.insert("best".into(), Value::Object(m));
            store::save_section("lamatowerdef", sec);
        }
    }

    // ===================================================== Wellen

    fn compose_wave(&self, n: i64) -> (Vec<(&'static str, f64)>, f64) {
        let mut bud = budget(n);
        let gap = 0.28f64.max(0.9 - 0.012 * n as f64);
        let mut queue: Vec<(&'static str, f64)> = Vec::new();
        let mut avail: Vec<&'static str> = ENEMY_KEYS
            .iter()
            .copied()
            .filter(|k| {
                let d = enemy_def(k);
                d.modes.contains(self.rules.letter) && !d.boss && d.intro <= n
            })
            .collect();
        let boss_wave = n % self.rules.boss == 0;
        if boss_wave {
            for _ in 0..1 + n / 32 {
                queue.push(("boss", 1.2));
            }
            bud *= 0.45;
        } else if n % 5 == 0 && n > 4 {
            avail = vec![avail[((n / 5) as usize) % avail.len()]];
        }
        let mut spent = 0.0;
        let mut units = queue.len();
        while spent < bud && units < UNITS_PER_WAVE {
            let k = if py::random() < 0.45 {
                let start = avail.len().saturating_sub(3);
                py::choice(&avail[start..])
            } else {
                py::choice(&avail)
            };
            let d = enemy_def(k);
            for i in 0..d.group {
                queue.push((k, if i != 0 { 0.12 } else { gap }));
            }
            spent += unit_cost(&d) * d.group as f64;
            units += d.group;
        }
        let hp_bonus = 1.0 + 0f64.max((bud - spent) / 1f64.max(bud));
        (queue, hp_bonus)
    }

    fn start_wave(&mut self) {
        if self.state != State::Build {
            return;
        }
        let (q, hb) = self.compose_wave(self.wave);
        self.spawn_queue = q;
        self.hp_bonus = hb;
        self.spawn_t = 0.5;
        self.state = State::Wave;
        self.b.play_sound("select");
    }

    fn spawn(&mut self, kind: &'static str) {
        if self.enemies.len() >= MAX_ENEMIES {
            self.spawn_t += 0.4;
            self.spawn_queue.insert(0, (kind, 0.0));
            return;
        }
        let d = enemy_def(kind);
        let hp = d.hp * hp_mult(self.wave) * map_hpf(self.wave, MAPS[self.map_idx].diff) * self.hp_bonus;
        self.next_id += 1;
        self.enemies.push(Enemy {
            id: self.next_id,
            kind,
            hp,
            maxhp: hp,
            d: 0.0,
            spd: d.spd,
            r: d.r,
            armor: d.armor,
            regen: d.regen,
            fly: d.fly,
            stealth: d.stealth,
            heal: d.heal,
            splits: d.splits,
            boss: d.boss,
            leak: d.leak,
            bounty: d.bounty,
            score: d.score,
            col: d.col,
            slow_t: 0.0,
            slow_f: 0.0,
            stun_t: 0.0,
            dot_t: 0.0,
            dot: 0.0,
            seen: !d.stealth,
            dead: false,
            plague: false,
        });
    }

    // ===================================================== Tuerme

    fn tower_stats(tw: &Tower) -> Stats {
        let base = tower_def(tw.kind);
        let lvl = (tw.level - 1) as f64;
        let mut st = Stats {
            dmg: base.dmg * py::pow(1.55, lvl),
            rng: base.rng * py::pow(1.12, lvl),
            cd: base.cd * py::pow(0.88, lvl),
            splash: base.splash,
            slow: base.slow,
            slow_t: base.slow_t,
            dot: base.dot * py::pow(1.55, lvl),
            dot_t: base.dot_t,
            chain: base.chain,
            buff: base.buff + 0.05 * lvl,
            income: base.income * py::pow(1.5, lvl),
            air: base.air,
            air_only: base.air_only,
            targets: 1,
            execute: 0.0,
            stun: 0.0,
            aura_dps: 0.0,
            ramp_max: 2.5,
            ramp_rate: 0.35,
            splits_dot: false,
            acid: false,
            ground_frac: 0.0,
            cluster: 1,
            buff_cd: false,
        };
        let k = tw.kind;
        match tw.branch {
            Some('a') => match k {
                "arrow" => {
                    st.targets = 2;
                    st.air = true;
                }
                "cannon" => {
                    st.splash *= 1.6;
                    st.stun = 0.3;
                }
                "frost" => st.stun = 0.35,
                "sniper" => st.execute = 0.20,
                "poison" => st.splits_dot = true,
                "tesla" => st.chain = 5,
                "support" => st.buff += 0.15,
                "mortar" => st.cluster = 3,
                "flak" => st.splash = 0.9,
                "laser" => st.targets = 3,
                "bank" => st.income *= 1.8,
                _ => {}
            },
            Some('b') => match k {
                "arrow" => st.dmg *= 1.85,
                "cannon" => {
                    st.dot = 10.0;
                    st.dot_t = 2.5;
                }
                "frost" => st.aura_dps = 10.0,
                "sniper" => st.cd *= 0.55,
                "poison" => st.acid = true,
                "tesla" => st.dmg *= 1.8,
                "support" => st.buff_cd = true,
                "mortar" => st.dmg *= 2.0,
                "flak" => {
                    st.ground_frac = 0.6;
                    st.air_only = false;
                }
                "laser" => {
                    st.ramp_max = 3.5;
                    st.ramp_rate = 0.7;
                }
                "bank" => st.income *= 1.2,
                _ => {}
            },
            _ => {}
        }
        st.dmg *= tw.buff_dmg;
        st.cd *= tw.buff_cd_f;
        st
    }

    fn refresh_towers(&mut self) {
        let mut sups: Vec<(f64, f64, f64, f64, bool)> = Vec::new();
        for tw in self.towers.iter_mut() {
            if tw.kind == "support" {
                tw.buff_dmg = 1.0;
                tw.buff_cd_f = 1.0;
                tw.st = Self::tower_stats(tw);
                sups.push((tw.cell.0 as f64 + 0.5, tw.cell.1 as f64 + 0.5, tw.st.rng, tw.st.buff, tw.st.buff_cd));
            }
        }
        for tw in self.towers.iter_mut() {
            if tw.kind == "support" {
                continue;
            }
            let (mut dmg_f, mut cd_f) = (1.0, 1.0f64);
            let (cx, cy) = (tw.cell.0 as f64 + 0.5, tw.cell.1 as f64 + 0.5);
            for &(sx, sy, rng, buff, bcd) in &sups {
                if sq(cx - sx) + sq(cy - sy) <= sq(rng) {
                    dmg_f += buff;
                    if bcd {
                        cd_f = cd_f.min(0.85);
                    }
                }
            }
            tw.buff_dmg = 1.6f64.min(dmg_f);
            tw.buff_cd_f = cd_f;
            tw.st = Self::tower_stats(tw);
        }
    }

    fn tower_at(&self, cell: (i32, i32)) -> Option<usize> {
        self.towers.iter().position(|t| t.cell == cell)
    }

    fn can_build(&self, cell: (i32, i32)) -> bool {
        let (c, r) = cell;
        if !(0..GRID_W).contains(&c) || !(0..GRID_H).contains(&r) {
            return false;
        }
        if self.path_cells.contains(&cell) || self.tower_at(cell).is_some() {
            return false;
        }
        if let Some(bc) = &self.build_cells {
            if !bc.contains(&cell) {
                return false;
            }
        }
        true
    }

    fn place_tower(&mut self, cell: (i32, i32)) {
        let Some(kind) = self.sel_card else {
            return;
        };
        let base = tower_def(kind);
        if self.gold < base.cost {
            self.b.play_sound("hit");
            return;
        }
        self.gold -= base.cost;
        let mut tw = Tower { kind, cell, level: 1, branch: None, invested: base.cost, cd_left: 0.0, angle: 0.0, ramp: 1.0, beam: None, buff_dmg: 1.0, buff_cd_f: 1.0, st: Stats::default() };
        tw.st = Self::tower_stats(&tw);
        self.towers.push(tw);
        self.refresh_towers();
        self.b.play_sound("lock");
        if self.gold < base.cost {
            self.sel_card = None;
        }
    }

    fn upgrade_cost(tw: &Tower) -> i64 {
        (tower_def(tw.kind).cost as f64 * 0.8 * tw.level as f64) as i64
    }

    fn branch_cost(tw: &Tower) -> i64 {
        (tower_def(tw.kind).cost as f64 * BRANCH_COST) as i64
    }

    fn sell_value(tw: &Tower) -> i64 {
        (tw.invested as f64 * 0.7) as i64
    }

    fn upgrade_tower(&mut self, i: usize) {
        let tw = &self.towers[i];
        if tw.level >= self.rules.levels || tw.branch.is_some() {
            return;
        }
        let cost = Self::upgrade_cost(tw);
        if self.gold < cost {
            self.b.play_sound("hit");
            return;
        }
        self.gold -= cost;
        self.towers[i].level += 1;
        self.towers[i].invested += cost;
        self.refresh_towers();
        self.b.play_sound("powerup");
        if self.towers[i].level >= self.rules.levels && !self.rules.branch {
            self.b.ach_event("td_maxed", None);
        }
    }

    fn branch_tower(&mut self, i: usize, which: char) {
        let tw = &self.towers[i];
        if !self.rules.branch || tw.branch.is_some() || tw.level < self.rules.levels {
            return;
        }
        let cost = Self::branch_cost(tw);
        if self.gold < cost {
            self.b.play_sound("hit");
            return;
        }
        self.gold -= cost;
        self.towers[i].branch = Some(which);
        self.towers[i].invested += cost;
        if self.towers[i].kind == "bank" && which == 'b' {
            self.gold += 200;
        }
        self.refresh_towers();
        self.b.play_sound("level");
        self.b.ach_event("td_maxed", None);
    }

    fn sell_tower(&mut self, cell: (i32, i32)) {
        let Some(i) = self.tower_at(cell) else {
            return;
        };
        let tw = self.towers.remove(i);
        self.gold += Self::sell_value(&tw);
        self.sel_tower = None;
        self.refresh_towers();
        self.b.play_sound("eat");
    }

    // ===================================================== Simulation

    fn enemy_idx(&self, id: u64) -> Option<usize> {
        self.enemies.iter().position(|e| e.id == id)
    }

    fn sim(&mut self, h: f64) {
        for v in self.abil_cd.iter_mut() {
            *v = 0f64.max(*v - h);
        }
        if self.state == State::Wave && !self.spawn_queue.is_empty() {
            self.spawn_t -= h;
            while !self.spawn_queue.is_empty() && self.spawn_t <= 0.0 {
                let (kind, gap) = self.spawn_queue.remove(0);
                self.spawn(kind);
                self.spawn_t += gap;
            }
        }
        self.sim_enemies(h);
        self.sim_towers(h);
        self.sim_shots(h);
        if self.state == State::Wave && self.spawn_queue.is_empty() && self.enemies.is_empty() {
            self.wave_cleared();
        }
    }

    fn sim_enemies(&mut self, h: f64) {
        let detect: Vec<(f64, f64, f64)> = self.towers.iter().filter(|t| tower_def(t.kind).detect).map(|t| (t.cell.0 as f64 + 0.5, t.cell.1 as f64 + 0.5, t.st.rng)).collect();
        let aura_frost: Vec<(f64, f64, f64, f64)> = self.towers.iter().filter(|t| t.st.aura_dps > 0.0).map(|t| (t.cell.0 as f64 + 0.5, t.cell.1 as f64 + 0.5, t.st.rng, t.st.aura_dps)).collect();
        let healers: Vec<u64> = self.enemies.iter().filter(|e| e.heal != 0.0 && e.hp > 0.0).map(|e| e.id).collect();
        let mut leaked = false;
        // Index-Schleife: getoetete Teiler haengen Kinder an, die Python
        // in derselben Schleife noch besucht.
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp <= 0.0 {
                i += 1;
                continue;
            }
            if self.enemies[i].dot_t > 0.0 {
                let e = &mut self.enemies[i];
                e.dot_t -= h;
                e.hp -= e.dot * h;
                if e.hp <= 0.0 {
                    self.kill(i);
                    i += 1;
                    continue;
                }
            }
            {
                let e = &mut self.enemies[i];
                if e.regen != 0.0 {
                    e.hp = e.maxhp.min(e.hp + e.regen * h);
                }
            }
            let (ex, ey) = self.pos_at(self.enemies[i].d, self.enemies[i].fly);
            for &(ax, ay, rng, dps) in &aura_frost {
                if sq(ex - ax) + sq(ey - ay) <= rng * rng {
                    self.enemies[i].hp -= dps * h;
                }
            }
            if self.enemies[i].hp <= 0.0 {
                self.kill(i);
                i += 1;
                continue;
            }
            if self.enemies[i].stealth {
                self.enemies[i].seen = detect.iter().any(|&(ax, ay, rng)| sq(ex - ax) + sq(ey - ay) <= rng * rng);
            }
            let e = &mut self.enemies[i];
            let mut v = e.spd;
            if e.slow_t > 0.0 {
                e.slow_t -= h;
                v *= 1.0 - e.slow_f;
            }
            if e.stun_t > 0.0 {
                e.stun_t -= h;
                v = 0.0;
            }
            e.d += v * h;
            let total = if e.fly { self.fly_total } else { self.path_total };
            if e.d >= total {
                e.hp = 0.0;
                self.lives -= e.leak;
                leaked = true;
            }
            i += 1;
        }
        for hid in healers {
            let Some(hi) = self.enemy_idx(hid) else {
                continue;
            };
            if self.enemies[hi].hp <= 0.0 {
                continue;
            }
            let (hx, hy) = self.pos_at(self.enemies[hi].d, self.enemies[hi].fly);
            let heal = self.enemies[hi].heal;
            for j in 0..self.enemies.len() {
                if j == hi || self.enemies[j].hp <= 0.0 {
                    continue;
                }
                let (ex, ey) = self.pos_at(self.enemies[j].d, self.enemies[j].fly);
                if sq(ex - hx) + sq(ey - hy) <= py::pow(1.5, 2.0) {
                    let e = &mut self.enemies[j];
                    e.hp = e.maxhp.min(e.hp + heal * h);
                }
            }
        }
        self.enemies.retain(|e| e.hp > 0.0);
        if leaked {
            self.b.play_sound("hit");
            self.b.rumble(150);
            if self.lives <= 0 {
                self.finish();
            }
        }
    }

    fn visible(e: &Enemy, st: &Stats) -> bool {
        if e.hp <= 0.0 || !e.seen {
            return false;
        }
        if e.fly {
            return st.air;
        }
        !st.air_only || st.ground_frac > 0.0
    }

    fn find_target(&self, tw: &Tower, st: &Stats) -> Option<usize> {
        let (cx, cy) = (tw.cell.0 as f64 + 0.5, tw.cell.1 as f64 + 0.5);
        let rng2 = sq(st.rng);
        let base = tower_def(tw.kind);
        let min2 = sq(base.min_rng);
        let mut best: Option<usize> = None;
        let mut best_v = -1.0;
        for (i, e) in self.enemies.iter().enumerate() {
            if !Self::visible(e, st) {
                continue;
            }
            let (ex, ey) = self.pos_at(e.d, e.fly);
            let d2 = sq(ex - cx) + sq(ey - cy);
            if d2 > rng2 || d2 < min2 {
                continue;
            }
            let v = if base.strong { e.hp } else { e.d + if e.boss { 1000.0 } else { 0.0 } };
            if v > best_v {
                best = Some(i);
                best_v = v;
            }
        }
        best
    }

    fn sim_towers(&mut self, h: f64) {
        for ti in 0..self.towers.len() {
            let st = self.towers[ti].st;
            let base = tower_def(self.towers[ti].kind);
            if base.income != 0.0 || self.towers[ti].kind == "support" {
                continue;
            }
            let (cx, cy) = (self.towers[ti].cell.0 as f64 + 0.5, self.towers[ti].cell.1 as f64 + 0.5);
            if base.beam {
                self.sim_laser(ti, &st, h);
                continue;
            }
            self.towers[ti].cd_left -= h;
            if self.towers[ti].cd_left > 0.0 {
                continue;
            }
            let Some(first) = self.find_target(&self.towers[ti], &st) else {
                continue;
            };
            let first_id = self.enemies[first].id;
            let mut targets = vec![first_id];
            if st.targets > 1 {
                for e in &self.enemies {
                    if e.id == first_id || !Self::visible(e, &st) {
                        continue;
                    }
                    let (ex, ey) = self.pos_at(e.d, e.fly);
                    if sq(ex - cx) + sq(ey - cy) <= sq(st.rng) {
                        targets.push(e.id);
                        if targets.len() >= st.targets {
                            break;
                        }
                    }
                }
            }
            for id in targets {
                self.fire(ti, &st, id);
            }
            if let Some(fi) = self.enemy_idx(first_id) {
                let (ex, ey) = self.pos_at(self.enemies[fi].d, self.enemies[fi].fly);
                self.towers[ti].angle = (ey - cy).atan2(ex - cx);
            } else {
                // Python haelt die Referenz: auch ein schon entfernter Gegner
                // hat noch Ort und Flug-Status - hier nie erreichbar, da erst
                // sim_enemies entfernt.
            }
            self.towers[ti].cd_left = st.cd;
        }
    }

    fn sim_laser(&mut self, ti: usize, st: &Stats, h: f64) {
        let (cx, cy) = (self.towers[ti].cell.0 as f64 + 0.5, self.towers[ti].cell.1 as f64 + 0.5);
        let mut e = self.towers[ti].beam.and_then(|id| self.enemy_idx(id).map(|i| (id, i)));
        if self.towers[ti].beam.is_some() {
            match e {
                Some((_, i)) => {
                    let en = &self.enemies[i];
                    let (ex, ey) = self.pos_at(en.d, en.fly);
                    if en.hp <= 0.0 || !Self::visible(en, st) || sq(ex - cx) + sq(ey - cy) > sq(st.rng) {
                        e = None;
                        self.towers[ti].ramp = 1.0;
                    }
                }
                None => self.towers[ti].ramp = 1.0,
            }
        }
        if e.is_none() {
            e = self.find_target(&self.towers[ti], st).map(|i| (self.enemies[i].id, i));
            self.towers[ti].ramp = 1.0;
        }
        self.towers[ti].beam = e.map(|x| x.0);
        let Some((eid, _)) = e else {
            return;
        };
        self.towers[ti].ramp = st.ramp_max.min(self.towers[ti].ramp + st.ramp_rate * h);
        let mut victims = vec![eid];
        if st.targets > 1 {
            for o in &self.enemies {
                if o.id == eid || !Self::visible(o, st) {
                    continue;
                }
                let (ox, oy) = self.pos_at(o.d, o.fly);
                if sq(ox - cx) + sq(oy - cy) <= sq(st.rng) {
                    victims.push(o.id);
                    if victims.len() >= st.targets {
                        break;
                    }
                }
            }
        }
        let ramp = self.towers[ti].ramp;
        for (i, v) in victims.iter().enumerate() {
            let frac = if i == 0 { 1.0 } else { 0.5 };
            if let Some(vi) = self.enemy_idx(*v) {
                self.damage(vi, st.dmg * ramp * h * frac, false);
            }
        }
        if let Some(i) = self.enemy_idx(eid) {
            let (ex, ey) = self.pos_at(self.enemies[i].d, self.enemies[i].fly);
            self.towers[ti].angle = (ey - cy).atan2(ex - cx);
        }
    }

    fn fire(&mut self, ti: usize, st: &Stats, eid: u64) {
        let base = tower_def(self.towers[ti].kind);
        let (cx, cy) = (self.towers[ti].cell.0 as f64 + 0.5, self.towers[ti].cell.1 as f64 + 0.5);
        let Some(ei) = self.enemy_idx(eid) else {
            return;
        };
        let (ex, ey) = self.pos_at(self.enemies[ei].d, self.enemies[ei].fly);
        let mut dmg = st.dmg;
        let efly = self.enemies[ei].fly;
        if efly && st.ground_frac == 0.0 && base.air_only {
        } else if !efly && st.ground_frac > 0.0 {
            dmg *= st.ground_frac;
        }
        if base.hitscan {
            self.add_fx(FxKind::Tracer((cx, cy), (ex, ey), base.col), 0.12);
            let e = &self.enemies[ei];
            if st.execute != 0.0 && e.hp / e.maxhp < st.execute && !e.boss {
                self.enemies[ei].hp = 0.0;
                self.kill(ei);
            } else {
                self.damage(ei, dmg, base.pierce);
            }
            self.b.play_sound("shoot");
            return;
        }
        if st.chain != 0 {
            let mut pts = vec![(cx, cy)];
            let mut victim = eid;
            let mut cdm = dmg;
            let mut hit: HashSet<u64> = HashSet::new();
            for _ in 0..st.chain {
                let Some(vi) = self.enemy_idx(victim) else {
                    break;
                };
                let (vx, vy) = self.pos_at(self.enemies[vi].d, self.enemies[vi].fly);
                pts.push((vx, vy));
                self.damage(vi, cdm, false);
                hit.insert(victim);
                cdm *= 0.65;
                let mut nxt = None;
                let mut nd = py::pow(1.5, 2.0);
                for o in &self.enemies {
                    if hit.contains(&o.id) || !Self::visible(o, st) {
                        continue;
                    }
                    let (ox, oy) = self.pos_at(o.d, o.fly);
                    let d2 = sq(ox - vx) + sq(oy - vy);
                    if d2 <= nd {
                        nxt = Some(o.id);
                        nd = d2;
                    }
                }
                let Some(n) = nxt else {
                    break;
                };
                victim = n;
            }
            self.add_fx(FxKind::Zigzag(pts, base.col), 0.15);
            return;
        }
        if base.lob {
            let dist = py::hypot(ex - cx, ey - cy);
            let fly_t = 0.35f64.max(1.2f64.min(dist / 6.0));
            let e = &self.enemies[ei];
            let aim = self.pos_at(e.d + e.spd * fly_t * 0.7, e.fly);
            let n = st.cluster;
            for i in 0..n {
                let jx = if i != 0 { py::uniform(-0.5, 0.5) } else { 0.0 };
                let jy = if i != 0 { py::uniform(-0.5, 0.5) } else { 0.0 };
                self.shots.push(Shot::Lob {
                    x: cx,
                    y: cy,
                    tx: aim.0 + jx,
                    ty: aim.1 + jy,
                    t: 0.0,
                    dur: fly_t,
                    dmg: dmg / if n > 1 { 1.6 } else { 1.0 },
                    splash: st.splash * if n > 1 { 0.75 } else { 1.0 },
                    stun: st.stun,
                    col: base.col,
                });
            }
            self.b.play_sound("shoot");
            return;
        }
        if self.shots.len() < MAX_SHOTS {
            self.shots.push(Shot::Proj {
                x: cx,
                y: cy,
                tgt: Some(eid),
                lx: ex,
                ly: ey,
                spd: base.proj.unwrap_or(9.0),
                dmg,
                splash: st.splash,
                slow: st.slow,
                slow_t: st.slow_t,
                dot: st.dot,
                dot_t: st.dot_t,
                stun: st.stun,
                acid: st.acid,
                splits_dot: st.splits_dot,
                col: base.col,
            });
        }
    }

    fn sim_shots(&mut self, h: f64) {
        let shots = std::mem::take(&mut self.shots);
        let mut rest = Vec::new();
        for mut s in shots {
            match &mut s {
                Shot::Lob { tx, ty, t, dur, dmg, splash, stun, .. } => {
                    *t += h;
                    if *t >= *dur {
                        let (p, d, sp, stn) = ((*tx, *ty), *dmg, *splash, *stun);
                        self.explode(p, d, sp, stn);
                    } else {
                        rest.push(s);
                    }
                    continue;
                }
                Shot::Proj { x, y, tgt, lx, ly, spd, dmg, splash, stun, .. } => {
                    let alive = tgt.and_then(|id| self.enemy_idx(id)).filter(|&i| self.enemies[i].hp > 0.0);
                    match alive {
                        Some(i) => {
                            let (a, b) = self.pos_at(self.enemies[i].d, self.enemies[i].fly);
                            *lx = a;
                            *ly = b;
                        }
                        None => *tgt = None,
                    }
                    let (dx, dy) = (*lx - *x, *ly - *y);
                    let dist = py::hypot(dx, dy);
                    let step = *spd * h;
                    if dist <= step.max(0.22) {
                        let (p, d, sp, stn) = ((*lx, *ly), *dmg, *splash, *stun);
                        if let Some(i) = alive {
                            self.impact(&s, i);
                        } else if sp != 0.0 {
                            self.explode(p, d, sp, stn);
                        }
                        continue;
                    }
                    *x += dx / dist * step;
                    *y += dy / dist * step;
                }
            }
            rest.push(s);
        }
        // explode/impact legen keine Schuesse an
        rest.extend(std::mem::take(&mut self.shots));
        self.shots = rest;
    }

    fn impact(&mut self, s: &Shot, ei: usize) {
        let Shot::Proj { lx, ly, dmg, splash, slow, slow_t, dot, dot_t, stun, acid, splits_dot, .. } = *s else {
            return;
        };
        if splash != 0.0 {
            self.explode((lx, ly), dmg, splash, stun);
        } else {
            let e = &mut self.enemies[ei];
            if acid {
                e.armor = 0.0;
            }
            if slow != 0.0 {
                e.slow_f = e.slow_f.max(slow);
                e.slow_t = e.slow_t.max(slow_t);
            }
            if stun != 0.0 {
                e.stun_t = e.stun_t.max(stun);
            }
            if dot != 0.0 {
                e.dot = e.dot.max(dot);
                e.dot_t = e.dot_t.max(dot_t);
                e.plague = splits_dot;
            }
            self.damage(ei, dmg, false);
        }
    }

    fn explode(&mut self, pos: (f64, f64), dmg: f64, radius: f64, stun: f64) {
        let (px, py_) = pos;
        let ids: Vec<u64> = self.enemies.iter().map(|e| e.id).collect();
        for id in ids {
            let Some(i) = self.enemy_idx(id) else {
                continue;
            };
            let e = &self.enemies[i];
            if e.hp <= 0.0 || e.fly {
                continue;
            }
            let (ex, ey) = self.pos_at(e.d, e.fly);
            if sq(ex - px) + sq(ey - py_) <= radius * radius {
                if stun != 0.0 {
                    self.enemies[i].stun_t = self.enemies[i].stun_t.max(stun);
                }
                self.damage(i, dmg, false);
            }
        }
        self.add_fx(FxKind::Ring(pos, radius, (255, 190, 120)), 0.25);
    }

    fn damage(&mut self, i: usize, dmg: f64, pierce: bool) {
        let e = &mut self.enemies[i];
        if e.hp <= 0.0 {
            return;
        }
        let mut dmg = dmg;
        if e.armor != 0.0 && !pierce {
            dmg = 1f64.max(dmg - e.armor);
        }
        e.hp -= dmg;
        if e.hp <= 0.0 {
            self.kill(i);
        }
    }

    fn kill(&mut self, i: usize) {
        if self.enemies[i].dead {
            return;
        }
        self.enemies[i].dead = true;
        self.enemies[i].hp = 0.0;
        let n = self.wave;
        let e = self.enemies[i].clone();
        self.gold += 1.max(py::round(e.bounty * bounty_mult(n)));
        self.b.score += e.score;
        self.kills += 1;
        let (ex, ey) = self.pos_at(e.d, e.fly);
        self.add_fx(FxKind::Burst((ex, ey), e.col), 0.4);
        if e.boss {
            self.b.play_sound("explode");
            self.b.ach_event("td_boss", None);
            let (px, py_) = self.px(ex, ey);
            ui::spawn_burst(px, py_, Some(rgb(e.col.0, e.col.1, e.col.2)), 24, 260.0);
        }
        if e.splits != 0 {
            for _ in 0..e.splits {
                if self.enemies.len() < MAX_ENEMIES {
                    let d = enemy_def("runt");
                    let hp = d.hp * hp_mult(n) * 0.6 * map_hpf(n, MAPS[self.map_idx].diff);
                    let dd = e.d - py::uniform(0.0, 0.5);
                    self.next_id += 1;
                    self.enemies.push(Enemy {
                        id: self.next_id,
                        kind: "runt",
                        hp,
                        maxhp: hp,
                        d: dd,
                        spd: d.spd,
                        r: d.r,
                        armor: 0.0,
                        regen: 0.0,
                        fly: false,
                        stealth: false,
                        heal: 0.0,
                        splits: 0,
                        boss: false,
                        leak: 1,
                        bounty: d.bounty,
                        score: d.score,
                        col: d.col,
                        slow_t: 0.0,
                        slow_f: 0.0,
                        stun_t: 0.0,
                        dot_t: 0.0,
                        dot: 0.0,
                        seen: true,
                        dead: false,
                        plague: false,
                    });
                }
            }
        }
        if e.plague && e.dot != 0.0 {
            for j in 0..self.enemies.len() {
                if self.enemies[j].id == e.id || self.enemies[j].hp <= 0.0 {
                    continue;
                }
                let (ox, oy) = self.pos_at(self.enemies[j].d, self.enemies[j].fly);
                if sq(ox - ex) + sq(oy - ey) <= py::pow(1.2, 2.0) {
                    let o = &mut self.enemies[j];
                    o.dot = o.dot.max(e.dot);
                    o.dot_t = o.dot_t.max(e.dot_t);
                }
            }
        }
    }

    fn wave_cleared(&mut self) {
        let n = self.wave;
        let diff = MAPS[self.map_idx].diff;
        let incomes: Vec<f64> = self.towers.iter().filter(|t| t.st.income != 0.0).map(|t| t.st.income).collect();
        let income = py::sum(&incomes);
        self.gold += 20 + 4 * n + income as i64;
        self.gold += 50.min((self.gold as f64 * 0.05) as i64);
        self.b.score += ((20 + 5 * n) as f64 * diff) as i64;
        self.b.play_sound("point");
        if n >= 20 {
            self.b.ach_event("td_wave20", None);
        }
        if n == 10 && self.lives == self.rules.lives {
            self.b.ach_event("td_perfect10", None);
        }
        self.wave = n + 1;
        self.save_best();
        self.state = State::Build;
    }

    fn finish(&mut self) {
        self.save_best();
        self.state = State::GameOver;
        self.b.game_over = true;
        self.over_t = self.anim_t;
        self.b.play_sound("gameover");
        self.b.rumble(300);
    }

    fn use_ability(&mut self, k: usize) {
        if !self.rules.abil || self.abil_cd[k] > 0.0 {
            return;
        }
        if ABIL_KEYS[k] == "meteor" {
            self.armed = if self.armed != Some("meteor") { Some("meteor") } else { None };
            self.b.play_sound("click");
            return;
        }
        self.abil_cd[k] = ABIL_CDS[k];
        if ABIL_KEYS[k] == "nova" {
            for e in self.enemies.iter_mut() {
                e.slow_f = e.slow_f.max(0.6);
                e.slow_t = e.slow_t.max(3.0);
            }
            self.add_fx(FxKind::Nova((GRID_W as f64 / 2.0, GRID_H as f64 / 2.0)), 0.5);
        } else {
            self.gold += 80 + 3 * self.wave;
        }
        self.b.play_sound("level");
    }

    fn cast_meteor(&mut self, cellpos: (f64, f64)) {
        self.armed = None;
        self.abil_cd[0] = ABIL_CDS[0];
        let dmg = (150 + 12 * self.wave) as f64;
        let (px, py_) = cellpos;
        let ids: Vec<u64> = self.enemies.iter().map(|e| e.id).collect();
        for id in ids {
            let Some(i) = self.enemy_idx(id) else {
                continue;
            };
            if self.enemies[i].hp <= 0.0 {
                continue;
            }
            let (ex, ey) = self.pos_at(self.enemies[i].d, self.enemies[i].fly);
            if sq(ex - px) + sq(ey - py_) <= py::pow(1.8, 2.0) {
                self.damage(i, dmg, true);
            }
        }
        self.add_fx(FxKind::Ring(cellpos, 1.8, (255, 150, 80)), 0.4);
        self.b.play_sound("explode");
    }

    fn add_fx(&mut self, kind: FxKind, ttl: f64) {
        if self.fx.len() < MAX_FX {
            self.fx.push(Fx { kind, age: 0.0, ttl });
        }
    }

    fn age_fx(&mut self, dt: f64) {
        for f in self.fx.iter_mut() {
            f.age += dt;
        }
        self.fx.retain(|f| f.age < f.ttl);
    }

    // ===================================================== Eingabe

    fn ev_mapsel(&mut self, ev: &InputEvent) {
        let n = MAPS.len();
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Left" | "a" => {
                    self.mapsel_idx = (self.mapsel_idx + n - 1) % n;
                    self.b.play_sound("move");
                }
                "Right" | "d" => {
                    self.mapsel_idx = (self.mapsel_idx + 1) % n;
                    self.b.play_sound("move");
                }
                "Up" | "w" | "Down" | "s" => {
                    self.mapsel_idx = (self.mapsel_idx + 2) % n;
                    self.b.play_sound("move");
                }
                "Return" | "space" => {
                    if self.unlocked(&MAPS[self.mapsel_idx]) {
                        let i = self.mapsel_idx;
                        self.start_run(i);
                    } else {
                        self.b.play_sound("hit");
                    }
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            if let Some(i) = self.map_rects.iter().position(|r| r.hit(ev.pos)) {
                self.mapsel_idx = i;
                if self.unlocked(&MAPS[i]) {
                    self.start_run(i);
                } else {
                    self.b.play_sound("hit");
                }
            }
        } else if ev.kind == Kind::MouseMove {
            self.mouse = Some(ev.pos);
        }
    }

    fn ev_gameover(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Return" | "space" => {
                    let i = self.map_idx;
                    self.start_run(i);
                }
                "m" | "M" => {
                    self.b.game_over = false;
                    self.state = State::MapSel;
                    self.b.play_sound("click");
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown && self.anim_t - self.over_t > 0.5 {
            let i = self.map_idx;
            self.start_run(i);
        }
    }

    fn cell_at(&self, pos: (i32, i32)) -> Option<(i32, i32)> {
        if !self.field_rect.hit(pos) {
            return None;
        }
        Some((py::fdiv32(pos.0 - self.ox, self.cell), py::fdiv32(pos.1 - self.oy, self.cell)))
    }

    fn cellpos_at(&self, pos: (i32, i32)) -> (f64, f64) {
        ((pos.0 - self.ox) as f64 / self.cell as f64, (pos.1 - self.oy) as f64 / self.cell as f64)
    }

    fn ev_play(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.clone();
                self.ev_key(&k);
            }
            Kind::MouseMove => self.mouse = Some(ev.pos),
            Kind::Wheel => {
                if self.bar_rect.hit(ev.pos) {
                    let over = 0.max(self.cards_h - self.cards_view_h);
                    self.bar_scroll = 0.max(over.min(self.bar_scroll - ev.delta * 30));
                }
            }
            Kind::MouseDown if ev.button == 3 => {
                if self.armed.is_some() || self.sel_card.is_some() {
                    self.armed = None;
                    self.sel_card = None;
                } else {
                    self.sel_tower = None;
                }
                self.b.play_sound("click");
            }
            Kind::MouseDown if ev.button == 1 => {
                self.mouse = Some(ev.pos);
                self.ev_click(ev.pos);
            }
            _ => {}
        }
    }

    fn ev_key(&mut self, key: &str) {
        let sel_idx = self.sel_tower.and_then(|c| self.tower_at(c));
        match key {
            "space" | "Return" => self.start_wave(),
            "f" | "F" => {
                self.ff = if self.ff == 1 { 2 } else { 1 };
                self.b.play_sound("click");
            }
            "g" | "G" => self.show_ranges = !self.show_ranges,
            "u" | "U" if sel_idx.is_some() => self.upgrade_tower(sel_idx.unwrap_or(0)),
            "x" | "X" => {
                if self.sel_card.is_some() || self.armed.is_some() {
                    self.sel_card = None;
                    self.armed = None;
                } else if let (Some(c), Some(_)) = (self.sel_tower, sel_idx) {
                    self.sell_tower(c);
                }
            }
            "q" | "Q" => self.use_ability(0),
            "w" | "W" => self.use_ability(1),
            "e" | "E" => self.use_ability(2),
            _ => {
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_digit()) {
                    let v: i64 = key.parse().unwrap_or(0);
                    let i = py::pmod(v - 1, 10) as usize;
                    if i < self.bar_keys.len() {
                        let k = self.bar_keys[i];
                        self.select_card(k);
                    }
                }
            }
        }
    }

    fn select_card(&mut self, key: &'static str) {
        if tower_def(key).cost > self.gold {
            self.b.play_sound("hit");
            return;
        }
        self.sel_card = if self.sel_card != Some(key) { Some(key) } else { None };
        self.sel_tower = None;
        self.armed = None;
        self.b.play_sound("click");
    }

    fn ev_click(&mut self, pos: (i32, i32)) {
        if self.bar_rect.hit(pos) {
            if let Some(&(k, _)) = self.abil_rects.iter().find(|(_, r)| r.hit(pos)) {
                self.use_ability(k);
                return;
            }
            if self.start_rect.hit(pos) && self.state == State::Build {
                self.start_wave();
                return;
            }
            if let Some(ti) = self.sel_tower.and_then(|c| self.tower_at(c)) {
                if self.info_rect.hit(pos) {
                    let tw = &self.towers[ti];
                    let branchable = self.rules.branch && tw.branch.is_none() && tw.level >= self.rules.levels;
                    if branchable && self.br_a_rect.hit(pos) {
                        self.branch_tower(ti, 'a');
                        return;
                    }
                    if branchable && self.br_b_rect.hit(pos) {
                        self.branch_tower(ti, 'b');
                        return;
                    }
                    if !branchable && self.up_rect.hit(pos) && tw.level < self.rules.levels && tw.branch.is_none() {
                        self.upgrade_tower(ti);
                        return;
                    }
                    if self.sell_rect.hit(pos) {
                        if let Some(c) = self.sel_tower {
                            self.sell_tower(c);
                        }
                        return;
                    }
                }
            }
            for (i, r) in self.card_rects.clone().iter().enumerate() {
                let rr = r.move_(0, -self.bar_scroll);
                if rr.bottom() < self.cards_top || rr.y > self.cards_top + self.cards_view_h {
                    continue;
                }
                if rr.hit(pos) {
                    let k = self.bar_keys[i];
                    self.select_card(k);
                    return;
                }
            }
            return;
        }
        if self.speed_rect.is_some_and(|r| r.hit(pos)) {
            self.ff = if self.ff == 1 { 2 } else { 1 };
            self.b.play_sound("click");
            return;
        }
        if self.armed == Some("meteor") {
            if self.field_rect.hit(pos) {
                let cp = self.cellpos_at(pos);
                self.cast_meteor(cp);
            }
            return;
        }
        let Some(cell) = self.cell_at(pos) else {
            return;
        };
        if self.sel_card.is_some() {
            if self.can_build(cell) {
                self.place_tower(cell);
            } else {
                self.b.play_sound("hit");
            }
            return;
        }
        if self.tower_at(cell).is_some() {
            self.sel_tower = Some(cell);
            self.b.play_sound("click");
        } else {
            self.sel_tower = None;
        }
    }

    // ===================================================== Zeichnen

    fn field_bg(&mut self) -> Surface {
        let key = (self.map_idx, self.b.width, self.b.height, self.cell);
        if let Some((k, s)) = &self.bg_cache {
            if *k == key {
                return s.clone();
            }
        }
        let p = ui::pal();
        let mut surf = pg::surface(self.b.width, self.b.height);
        surf.fill_c(ui::mix(COL_GRASS2, (0, 0, 0), 0.35));
        let cs = self.cell;
        for r in 0..GRID_H {
            for c in 0..GRID_W {
                let col = if (c + r) % 2 == 0 { COL_GRASS1 } else { COL_GRASS2 };
                draw::rect(&mut surf, col, Rect::new(self.ox + c * cs, self.oy + r * cs, cs, cs), 0);
            }
        }
        if let Some(bc) = &self.build_cells {
            let mc = ui::mix(COL_GRASS1, (255, 255, 255), 0.10);
            for &(c, r) in bc {
                draw::rect(&mut surf, mc, Rect::new(self.ox + c * cs + 2, self.oy + r * cs + 2, cs - 4, cs - 4), 1);
            }
        }
        let pts: Vec<(f64, f64)> = self.path_pts.iter().map(|&(x, y)| self.px(x, y)).collect();
        for (width, col) in [((cs as f64 * 0.86) as i32, COL_PATH_EDGE), ((cs as f64 * 0.62) as i32, COL_PATH)] {
            for w in pts.windows(2) {
                draw::line(&mut surf, col, w[0], w[1], width);
            }
            for p in &pts {
                draw::circle(&mut surf, col, *p, width / 2, 0);
            }
        }
        let (sx, sy) = pts[0];
        let (ex, ey) = *pts.last().unwrap_or(&pts[0]);
        draw::circle(&mut surf, rgb(30, 26, 40), (ex as i32, ey as i32), (cs as f64 * 0.42) as i32, 0);
        draw::circle(&mut surf, rgb(120, 90, 200), (ex as i32, ey as i32), (cs as f64 * 0.42) as i32, 3);
        let d = if sx < self.field_rect.centerx() as f64 { 1.0 } else { -1.0 };
        let csf = cs as f64;
        draw::polygon(&mut surf, rgb(240, 220, 130), &[(sx - d * csf * 0.3, sy - csf * 0.35), (sx + d * csf * 0.45, sy), (sx - d * csf * 0.3, sy + csf * 0.35)], 0);
        draw::rect(&mut surf, p.border, self.field_rect, 2);
        self.bg_cache = Some((key, surf.clone()));
        surf
    }

    fn draw_tower(&self, s: &mut Surface, kind: &str, level: i64, branch: Option<char>, angle: f64, px: i32, py_: i32, ghost: bool) {
        let cs = self.cell as f64;
        let col = tower_def(kind).col;
        let base_r = (cs * 0.38) as i32;
        draw::circle(s, rgb(38, 42, 50), (px, py_), base_r, 0);
        draw::circle(s, col, (px, py_), base_r, 2);
        let a = angle;
        let blen = cs * 0.45;
        let (pxf, pyf) = (px as f64, py_ as f64);
        let (bx, by) = (pxf + a.cos() * blen, pyf + a.sin() * blen);
        match kind {
            "arrow" => {
                draw::line(s, col, (px, py_), (bx, by), 3);
            }
            "cannon" => {
                draw::line(s, col, (px, py_), (bx, by), 4.max(self.cell / 6));
                draw::circle(s, col, (px, py_), (cs * 0.2) as i32, 0);
            }
            "frost" => {
                let pts: Vec<(f64, f64)> = (0..6).map(|i| (pxf + (a + i as f64 * TAU / 6.0).cos() * cs * 0.24, pyf + (a + i as f64 * TAU / 6.0).sin() * cs * 0.24)).collect();
                draw::polygon(s, col, &pts, 2);
            }
            "sniper" => {
                draw::line(s, col, (px, py_), (pxf + a.cos() * cs * 0.62, pyf + a.sin() * cs * 0.62), 2);
                draw::circle(s, col, (px, py_), (cs * 0.12) as i32, 0);
            }
            "poison" => {
                draw::circle(s, col, (px, py_), (cs * 0.2) as i32, 0);
                draw::circle(s, ui::mix(col, (0, 0, 0), 0.4), (px, py_ - (cs * 0.12) as i32), (cs * 0.1) as i32, 0);
            }
            "tesla" => {
                draw::circle(s, col, (px, py_), (cs * 0.14) as i32, 0);
                for i in 0..3 {
                    let aa = self.anim_t * 2.0 + i as f64 * TAU / 3.0;
                    draw::line(s, col, (px, py_), (pxf + aa.cos() * cs * 0.3, pyf + aa.sin() * cs * 0.3), 2);
                }
            }
            "support" => {
                let rr = (cs * (0.2 + 0.06 * ui::pulse(2.0, 0.35, 1.0))) as i32;
                draw::circle(s, col, (px, py_), rr, 2);
            }
            "mortar" => {
                draw::circle(s, col, (px, py_), (cs * 0.26) as i32, 4);
            }
            "flak" => {
                for off in [-0.08, 0.08] {
                    let ox = (a + PI / 2.0).cos() * cs * off;
                    let oy = (a + PI / 2.0).sin() * cs * off;
                    draw::line(s, col, (pxf + ox, pyf + oy), (bx + ox, by + oy), 2);
                }
            }
            "laser" => {
                let pts = [
                    (pxf + a.cos() * cs * 0.3, pyf + a.sin() * cs * 0.3),
                    (pxf + (a + 2.2).cos() * cs * 0.2, pyf + (a + 2.2).sin() * cs * 0.2),
                    (pxf + (a - 2.2).cos() * cs * 0.2, pyf + (a - 2.2).sin() * cs * 0.2),
                ];
                draw::polygon(s, col, &pts, 0);
            }
            _ => {
                let rr = (cs * 0.2) as i32;
                draw::rect(s, col, Rect::new(px - rr, py_ - rr, rr * 2, rr * 2), 0);
                draw::rect(s, ui::mix(col, (0, 0, 0), 0.4), Rect::new(px - rr, py_ - rr, rr * 2, rr * 2), 2);
            }
        }
        if ghost {
            return;
        }
        for i in 0..(level - 1) as i32 {
            draw::rect(s, col, Rect::new(px - base_r + i * 6, py_ + base_r - 2, 4, 4), 0);
        }
        if let Some(br) = branch {
            let bc = if br == 'a' { rgb(255, 255, 255) } else { rgb(255, 200, 90) };
            draw::circle(s, bc, (px + base_r - 3, py_ - base_r + 3), 3, 0);
        }
    }

    fn draw_towers(&self, s: &mut Surface) {
        let cs = self.cell;
        for tw in &self.towers {
            let (pxf, pyf) = self.px(tw.cell.0 as f64 + 0.5, tw.cell.1 as f64 + 0.5);
            let (px, py_) = (pxf as i32, pyf as i32);
            let sel = Some(tw.cell) == self.sel_tower;
            if sel || self.show_ranges {
                let rng = tw.st.rng;
                if rng != 0.0 {
                    draw::circle(s, ui::mix(tower_def(tw.kind).col, (255, 255, 255), 0.2), (px, py_), (rng * cs as f64) as i32, 1);
                }
            }
            if sel {
                draw::rect(s, self.b.accent, Rect::new(self.ox + tw.cell.0 * cs, self.oy + tw.cell.1 * cs, cs, cs), 2);
            }
            self.draw_tower(s, tw.kind, tw.level, tw.branch, tw.angle, px, py_, false);
            if tower_def(tw.kind).beam {
                if let Some(e) = tw.beam.and_then(|id| self.enemies.iter().find(|e| e.id == id)) {
                    if e.hp > 0.0 {
                        let (ex, ey) = self.pos_at(e.d, e.fly);
                        let (epx, epy) = self.px(ex, ey);
                        let w = 2 + tw.ramp as i32;
                        draw::line(s, rgb(255, 120, 110), (px, py_), (epx, epy), w);
                    }
                }
            }
        }
    }

    fn draw_enemies(&self, s: &mut Surface) {
        let cs = self.cell;
        let order: Vec<&Enemy> = self.enemies.iter().filter(|e| !e.fly).chain(self.enemies.iter().filter(|e| e.fly)).collect();
        for e in order {
            let (ex, ey) = self.pos_at(e.d, e.fly);
            let (pxf, pyf) = self.px(ex, ey);
            let (px, mut py_) = (pxf as i32, pyf as i32);
            let r = 3.max((e.r * cs as f64) as i32);
            if e.fly {
                draw::ellipse(s, COL_AIR_SHADOW, Rect::new(px - r, py_ + (cs as f64 * 0.28) as i32, r * 2, 3.max(r / 2)), 0);
                py_ -= (cs as f64 * 0.22) as i32;
            }
            if e.stealth && !e.seen {
                draw::circle(s, ui::mix(COL_GRASS1, (255, 255, 255), 0.18), (px, py_), r, 1);
                continue;
            }
            let mut col = rgb(e.col.0, e.col.1, e.col.2);
            if e.slow_t > 0.0 {
                col = ui::mix(col, (130, 200, 255), 0.45);
            }
            match e.kind {
                "tank" => {
                    draw::rect_r(s, col, Rect::new(px - r, py_ - r, r * 2, r * 2), 0, 3);
                    draw::rect_r(s, ui::mix(col, (0, 0, 0), 0.4), Rect::new(px - r, py_ - r, r * 2, r * 2), 2, 3);
                }
                "fast" => {
                    draw::polygon(s, col, &[(px + r, py_), (px - r, py_ - r), (px - r, py_ + r)], 0);
                }
                "boss" => {
                    draw::circle(s, col, (px, py_), r, 0);
                    draw::circle(s, ui::mix(col, (0, 0, 0), 0.45), (px, py_), r, 3);
                    let (pxf, pyf, rf) = (px as f64, py_ as f64, r as f64);
                    for i in 0..6 {
                        let aa = self.anim_t * 1.5 + i as f64 * TAU / 6.0;
                        draw::line(s, col, (pxf + aa.cos() * rf, pyf + aa.sin() * rf), (pxf + aa.cos() * (rf + cs as f64 * 0.18), pyf + aa.sin() * (rf + cs as f64 * 0.18)), 2);
                    }
                }
                _ => {
                    draw::circle(s, col, (px, py_), r, 0);
                    draw::circle(s, ui::mix(col, (0, 0, 0), 0.4), (px, py_), r, 2);
                }
            }
            match e.kind {
                "shield" => {
                    draw::circle(s, rgb(220, 225, 255), (px, py_), r + 2, 1);
                }
                "regen" => {
                    draw::line(s, rgb(240, 255, 240), (px - 3, py_), (px + 3, py_), 2);
                    draw::line(s, rgb(240, 255, 240), (px, py_ - 3), (px, py_ + 3), 2);
                }
                "healer" => {
                    draw::circle(s, rgb(230, 255, 245), (px, py_), r + 3, 1);
                }
                "splitter" => {
                    let mc = ui::mix(e.col, (0, 0, 0), 0.35);
                    draw::circle(s, mc, (px - r / 3, py_), 2, 0);
                    draw::circle(s, mc, (px + r / 3, py_), 2, 0);
                }
                _ => {}
            }
            if e.dot_t > 0.0 {
                draw::circle(s, rgb(140, 220, 80), (px + r - 1, py_ - r + 1), 3, 0);
            }
            if e.hp < e.maxhp {
                let bw = r * 2;
                let frac = 0f64.max(e.hp / e.maxhp);
                let y0 = py_ - r - 6;
                draw::rect(s, COL_HP_BG, Rect::new(px - r, y0, bw, 3), 0);
                draw::rect(s, ui::mix((220, 80, 70), COL_HP, frac), Rect::new(px - r, y0, (bw as f64 * frac) as i32, 3), 0);
            }
        }
    }

    fn draw_shots(&self, s: &mut Surface) {
        let cs = self.cell as f64;
        for sh in &self.shots {
            match sh {
                Shot::Lob { x, y, tx, ty, t, dur, col, .. } => {
                    let f = t / dur;
                    let (px, py_) = self.px(x + (tx - x) * f, y + (ty - y) * f);
                    let (ttx, tty) = self.px(*tx, *ty);
                    draw::circle(s, rgb(20, 20, 20), (ttx as i32, tty as i32), 3, 0);
                    let arc = (f * PI).sin() * cs * 1.1;
                    draw::circle(s, *col, (px as i32, (py_ - arc) as i32), 4, 0);
                }
                Shot::Proj { x, y, col, .. } => {
                    let (px, py_) = self.px(*x, *y);
                    draw::circle(s, *col, (px as i32, py_ as i32), 3, 0);
                }
            }
        }
    }

    fn draw_fx(&self, s: &mut Surface) {
        let cs = self.cell as f64;
        for f in &self.fx {
            let fr = f.age / f.ttl;
            match &f.kind {
                FxKind::Tracer(a, b, col) => {
                    draw::line(s, *col, self.px(a.0, a.1), self.px(b.0, b.1), 2);
                }
                FxKind::Zigzag(pts, col) => {
                    let pp: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| self.px(x, y)).collect();
                    for w in pp.windows(2) {
                        let mx = (w[0].0 + w[1].0) / 2.0 + py::uniform(-4.0, 4.0);
                        let my = (w[0].1 + w[1].1) / 2.0 + py::uniform(-4.0, 4.0);
                        draw::lines(s, *col, false, &[w[0], (mx, my), w[1]], 2);
                    }
                }
                FxKind::Ring(pos, r, col) => {
                    let (px, py_) = self.px(pos.0, pos.1);
                    let rr = (r * cs * (0.3 + 0.7 * fr)) as i32;
                    draw::circle(s, *col, (px as i32, py_ as i32), rr, 2);
                }
                FxKind::Burst(pos, col) => {
                    let (px, py_) = self.px(pos.0, pos.1);
                    for i in 0..5 {
                        let aa = i as f64 * TAU / 5.0 + fr * 2.0;
                        let d = fr * cs * 0.7;
                        draw::circle(s, *col, ((px + aa.cos() * d) as i32, (py_ + aa.sin() * d) as i32), 2, 0);
                    }
                }
                FxKind::Nova(pos) => {
                    let (px, py_) = self.px(pos.0, pos.1);
                    let rr = (fr * self.field_rect.w as f64 * 0.7) as i32;
                    draw::circle(s, rgb(140, 210, 255), (px as i32, py_ as i32), 2.max(rr), 2);
                }
            }
        }
    }

    fn draw_ghost(&self, s: &mut Surface) {
        let Some(mouse) = self.mouse else {
            return;
        };
        if self.state == State::GameOver {
            return;
        }
        let cs = self.cell;
        if self.armed == Some("meteor") {
            if self.field_rect.hit(mouse) {
                draw::circle(s, rgb(255, 150, 80), mouse, (1.8 * cs as f64) as i32, 2);
            }
            return;
        }
        let Some(card) = self.sel_card else {
            return;
        };
        let Some(cell) = self.cell_at(mouse) else {
            return;
        };
        let ok = self.can_build(cell);
        let (pxf, pyf) = self.px(cell.0 as f64 + 0.5, cell.1 as f64 + 0.5);
        let (px, py_) = (pxf as i32, pyf as i32);
        let col = if ok { COL_BUILD_OK } else { COL_BUILD_BAD };
        draw::rect(s, col, Rect::new(self.ox + cell.0 * cs, self.oy + cell.1 * cs, cs, cs), 2);
        let rng = tower_def(card).rng;
        if rng != 0.0 {
            draw::circle(s, col, (px, py_), (rng * cs as f64) as i32, 1);
        }
        self.draw_tower(s, card, 1, None, 0.0, px, py_, true);
    }

    fn heart(s: &mut Surface, x: i32, y: i32, r: i32, col: Rgb) {
        draw::circle(s, col, (x - r / 2, y - r / 3), r / 2 + 1, 0);
        draw::circle(s, col, (x + r / 2, y - r / 3), r / 2 + 1, 0);
        draw::polygon(s, col, &[(x - r, y - r / 4), (x + r, y - r / 4), (x, y + r)], 0);
    }

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width - self.bar_w;
        draw::rect(s, p.panel, Rect::new(0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h - 1), (w, self.hud_h - 1), 1);
        let cy = self.hud_h / 2;
        let mut x = 14;
        Self::heart(s, x + 5, cy, 7, (235, 90, 100));
        let img = self.small.render(&0.max(self.lives).to_string(), true, p.text);
        s.bl(&img, (x + 18, cy - img.h() / 2));
        x += 26 + img.w() + 14;
        draw::circle(s, p.gold, (x + 6, cy), 7, 0);
        draw::circle(s, ui::mix(p.gold, (0, 0, 0), 0.4), (x + 6, cy), 7, 2);
        let img = self.small.render(&self.gold.to_string(), true, p.gold);
        s.bl(&img, (x + 18, cy - img.h() / 2));
        x += 26 + img.w() + 16;
        let img = self.small.render(&tf("td.wave", &[("n", &self.wave.to_string())]), true, self.b.accent);
        s.bl(&img, (x, cy - img.h() / 2));
        x += img.w() + 16;
        if self.state == State::Wave {
            let left = self.enemies.len() + self.spawn_queue.len();
            let img = self.tiny.render(&tf("td.left", &[("n", &left.to_string())]), true, p.text_dim);
            s.bl(&img, (x, cy - img.h() / 2));
        }
        let img = self.small.render(&self.b.score.to_string(), true, p.text);
        let chip_w = 40.max(self.tiny.size("2x").0 + 18);
        let sr = Rect::new(w - chip_w - 10, 5, chip_w, self.hud_h - 10);
        self.speed_rect = Some(sr);
        s.bl(&img, (sr.x - img.w() - 14, cy - img.h() / 2));
        let sel = self.ff > 1;
        draw::rect_r(s, if sel { p.panel_light } else { p.panel }, sr, 0, 6);
        draw::rect_r(s, if sel { self.b.accent } else { p.border }, sr, 1, 6);
        let img = self.tiny.render(&format!("{}x", self.ff), true, if sel { self.b.accent } else { p.text_dim });
        s.blr(&img, img.get_rect().center_at(sr.center()));
    }

    fn draw_sidebar(&self, s: &mut Surface) {
        let p = ui::pal();
        draw::rect(s, p.panel, self.bar_rect, 0);
        draw::line(s, p.border, (self.bar_rect.x, 0), (self.bar_rect.x, self.b.height), 1);
        for &(k, r) in &self.abil_rects {
            let cd = self.abil_cd[k];
            let ready = cd <= 0.0;
            let armed = self.armed == Some(ABIL_KEYS[k]);
            draw::rect_r(s, if ready { p.panel_light } else { p.panel }, r, 0, 6);
            draw::rect_r(s, if armed { self.b.accent } else { p.border }, r, 1 + armed as i32, 6);
            let (cx, cy) = r.center();
            match ABIL_KEYS[k] {
                "meteor" => {
                    draw::circle(s, rgb(255, 150, 80), (cx, cy - 2), 5, 0);
                    draw::line(s, rgb(255, 150, 80), (cx + 3, cy - 5), (cx + 8, cy - 10), 2);
                }
                "nova" => {
                    draw::circle(s, rgb(140, 210, 255), (cx, cy - 2), 6, 2);
                }
                _ => {
                    draw::circle(s, p.gold, (cx, cy - 2), 6, 0);
                }
            }
            if !ready {
                let img = self.tiny.render(&(cd as i64 + 1).to_string(), true, p.text_dim);
                s.blr(&img, img.get_rect().midbottom_at((cx, r.bottom() - 1)));
            }
        }
        let clip_old = s.get_clip();
        s.set_clip(Some(Rect::new(self.bar_rect.x, self.cards_top, self.bar_w, self.cards_view_h)));
        let mut hover_key = None;
        for (key, r) in self.bar_keys.iter().zip(&self.card_rects) {
            let rr = r.move_(0, -self.bar_scroll);
            let base = tower_def(key);
            let afford = self.gold >= base.cost;
            let sel = self.sel_card == Some(*key);
            draw::rect_r(s, if sel { p.panel_light } else { p.panel }, rr, 0, 8);
            draw::rect_r(s, if sel { self.b.accent } else { p.border }, rr, if sel { 2 } else { 1 }, 8);
            self.draw_tower(s, key, 1, None, -PI / 2.0, rr.centerx(), rr.centery() - 6, true);
            let img = self.tiny.render(&base.cost.to_string(), true, if afford { p.gold } else { p.text_faint });
            s.blr(&img, img.get_rect().midbottom_at((rr.centerx(), rr.bottom() - 3)));
            if self.mouse.is_some_and(|m| rr.hit(m)) {
                hover_key = Some(*key);
            }
        }
        s.set_clip(Some(clip_old));
        self.draw_info(s, hover_key);
        if self.state == State::Build {
            ui::draw_button_ex(s, self.start_rect, &t("td.start_wave"), &self.small, true, None, None, Some(self.b.accent));
        } else {
            draw::rect_r(s, p.panel_light, self.start_rect, 0, 8);
            draw::rect_r(s, p.border, self.start_rect, 1, 8);
            let img = self.small.render(&tf("td.wave", &[("n", &self.wave.to_string())]), true, p.text_dim);
            s.blr(&img, img.get_rect().center_at(self.start_rect.center()));
        }
    }

    fn info_lines(key: &str, tw: Option<&Tower>) -> String {
        let base = tower_def(key);
        let st = tw.map(|t| t.st);
        let dmg = st.map(|s| s.dmg).unwrap_or(base.dmg);
        let rng = st.map(|s| s.rng).unwrap_or(base.rng);
        let cd = st.map(|s| s.cd).unwrap_or(base.cd);
        let mut lines = Vec::new();
        if base.income != 0.0 {
            let inc = st.map(|s| s.income).unwrap_or(base.income);
            lines.push(format!("+{}/W", inc as i64));
        } else if base.buff != 0.0 {
            let b = st.map(|s| s.buff).unwrap_or(base.buff);
            lines.push(format!("+{}%", (b * 100.0) as i64));
        } else if base.beam {
            lines.push(format!("{}/s", dmg as i64));
        } else {
            lines.push(format!("{}  |  {:.1}s", dmg as i64, cd));
        }
        if rng != 0.0 {
            lines.push(format!("R {rng:.1}"));
        }
        lines.join("   ")
    }

    fn draw_info(&self, s: &mut Surface, hover_key: Option<&'static str>) {
        let p = ui::pal();
        let r = self.info_rect;
        draw::rect_r(s, p.panel_light, r, 0, 8);
        draw::rect_r(s, p.border, r, 1, 8);
        let (x, mut y) = (r.x + 10, r.y + 8);
        if let Some(tw) = self.sel_tower.and_then(|c| self.tower_at(c)).map(|i| &self.towers[i]) {
            let base = tower_def(tw.kind);
            let img = self.small.render(&t(&format!("td.tower.{}", tw.kind)), true, base.col);
            s.bl(&img, (x, y));
            y += img.h() + 2;
            let img = self.tiny.render(&tf("td.level", &[("n", &tw.level.to_string())]), true, p.text_dim);
            s.bl(&img, (x, y));
            y += img.h() + 2;
            let img = self.tiny.render(&Self::info_lines(tw.kind, Some(tw)), true, p.text);
            s.bl(&img, (x, y));
            let branchable = self.rules.branch && tw.branch.is_none() && tw.level >= self.rules.levels;
            if branchable {
                for (which, rect) in [('a', self.br_a_rect), ('b', self.br_b_rect)] {
                    let cost = Self::branch_cost(tw);
                    let ok = self.gold >= cost;
                    draw::rect_r(s, p.panel, rect, 0, 6);
                    draw::rect_r(s, if ok { self.b.accent } else { p.border }, rect, 1, 6);
                    let img = self.tiny.render(&t(&format!("td.br.{}.{}", tw.kind, which)), true, if ok { p.text } else { p.text_faint });
                    s.blr(&img, img.get_rect().center_at((rect.centerx(), rect.centery() - 6)));
                    let img = self.tiny.render(&cost.to_string(), true, if ok { p.gold } else { p.text_faint });
                    s.blr(&img, img.get_rect().center_at((rect.centerx(), rect.centery() + 8)));
                }
            } else if tw.level < self.rules.levels && tw.branch.is_none() {
                let cost = Self::upgrade_cost(tw);
                let ok = self.gold >= cost;
                draw::rect_r(s, p.panel, self.up_rect, 0, 6);
                draw::rect_r(s, if ok { self.b.accent } else { p.border }, self.up_rect, 1, 6);
                let img = self.tiny.render(&tf("td.upgrade", &[("c", &cost.to_string())]), true, if ok { p.text } else { p.text_faint });
                s.blr(&img, img.get_rect().center_at(self.up_rect.center()));
            } else {
                let img = self.tiny.render(&t("td.max"), true, p.gold);
                s.blr(&img, img.get_rect().center_at(self.up_rect.center()));
            }
            draw::rect_r(s, p.panel, self.sell_rect, 0, 6);
            draw::rect_r(s, p.border, self.sell_rect, 1, 6);
            let img = self.tiny.render(&tf("td.sell", &[("c", &Self::sell_value(tw).to_string())]), true, p.text_dim);
            s.blr(&img, img.get_rect().center_at(self.sell_rect.center()));
            return;
        }
        if let Some(key) = hover_key.or(self.sel_card) {
            let base = tower_def(key);
            let img = self.small.render(&t(&format!("td.tower.{key}")), true, base.col);
            s.bl(&img, (x, y));
            y += img.h() + 2;
            let img = self.tiny.render(&Self::info_lines(key, None), true, p.text);
            s.bl(&img, (x, y));
            y += img.h() + 4;
            y = Self::wrap_text(s, &t(&format!("td.tower.{key}.desc")), x, y, r.w - 20, &self.tiny, p.text_dim);
            if self.sel_card.is_some() {
                y += 4;
                Self::wrap_text(s, &t("td.place_hint"), x, y, r.w - 20, &self.tiny, p.text_faint);
            }
            return;
        }
        let hint = t(if self.state == State::Build { "td.hint.build" } else { "td.hint.wave" });
        Self::wrap_text(s, &hint, x, y, r.w - 20, &self.tiny, p.text_dim);
    }

    fn wrap_text(s: &mut Surface, text: &str, x: i32, mut y: i32, maxw: i32, fnt: &Font, col: Color) -> i32 {
        let mut line = String::new();
        for wd in text.split_whitespace() {
            let test = format!("{line} {wd}").trim().to_string();
            if fnt.size(&test).0 > maxw && !line.is_empty() {
                let img = fnt.render(&line, true, col);
                s.bl(&img, (x, y));
                y += img.h() + 1;
                line = wd.to_string();
            } else {
                line = test;
            }
        }
        if !line.is_empty() {
            let img = fnt.render(&line, true, col);
            s.bl(&img, (x, y));
            y += img.h() + 1;
        }
        y
    }

    fn ui_bg(&mut self) -> Surface {
        let p = ui::pal();
        let key = (self.b.width, self.b.height, p.bg_top, p.bg_bottom);
        if let Some((k, s)) = &self.ui_bg_cache {
            if *k == key {
                return s.clone();
            }
        }
        let mut surf = pg::surface(self.b.width, self.b.height);
        ui::draw_background(&mut surf, self.b.width, self.b.height);
        self.ui_bg_cache = Some((key, surf.clone()));
        surf
    }

    fn draw_mapsel(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let bg = self.ui_bg();
        s.bl(&bg, (0, 0));
        ui::draw_title_ex(s, self.b.width, "TOWER DEFENSE", Some(&t("td.choose_map")), 52, None, None, Some(self.b.accent));
        for (i, (m, r)) in MAPS.iter().zip(self.map_rects.clone()).enumerate() {
            let unlocked = self.unlocked(m);
            let sel = i == self.mapsel_idx || self.mouse.is_some_and(|mm| r.hit(mm));
            ui::draw_panel_ex(s, r, None, None, None, true, if sel { Some(self.b.accent) } else { None });
            if sel {
                draw::rect_r(s, self.b.accent, r, 2, 10);
            }
            let pad = 18.0;
            let (pw, ph) = (r.w as f64 - pad * 2.0, r.h as f64 - pad * 2.0 - 34.0);
            let pts: Vec<(f64, f64)> = m
                .path
                .iter()
                .map(|&(c, rr)| ((c as f64 + 0.5).clamp(0.0, GRID_W as f64), (rr as f64 + 0.5).clamp(0.0, GRID_H as f64)))
                .map(|(x, y)| (r.x as f64 + pad + (x / GRID_W as f64) * pw, r.y as f64 + pad + (y / GRID_H as f64) * ph))
                .collect();
            let col = if unlocked { p.text_dim } else { p.text_faint };
            draw::lines(s, col, false, &pts, 3);
            draw::circle(s, if unlocked { p.green } else { p.text_faint }, (pts[0].0 as i32, pts[0].1 as i32), 4, 0);
            let last = *pts.last().unwrap_or(&pts[0]);
            draw::circle(s, if unlocked { self.b.accent } else { p.text_faint }, (last.0 as i32, last.1 as i32), 4, 0);
            let img = self.small.render(&t(&format!("td.map.{}", m.id)), true, if unlocked { p.text } else { p.text_faint });
            s.bl(&img, (r.x + 14, r.bottom() - 30));
            for j in 0..4 {
                let cc = if j <= i { self.b.accent } else { p.border };
                draw::circle(s, cc, (r.right() - 60 + j as i32 * 12, r.bottom() - 22), 3, 0);
            }
            if unlocked {
                let best = self.best_of(m.id);
                if best != 0 {
                    let img = self.tiny.render(&tf("td.map.best", &[("n", &best.to_string())]), true, p.gold);
                    s.bl(&img, (r.x + 14, r.y + 6));
                }
            } else {
                let img = self.tiny.render(&tf("td.map.locked", &[("n", &m.unlock.to_string())]), true, p.text_faint);
                s.bl(&img, (r.x + 14, r.y + 6));
            }
        }
        ui::draw_footer(s, self.b.width, self.b.height, &t("td.setup_hint"));
    }

    fn draw_gameover(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut dim = pg::surface_alpha(w, h);
        dim.fill_c(rgba(10, 12, 20, 150));
        s.bl(&dim, (0, 0));
        let cx = w / 2;
        let img = self.b.big_font.render(&t("common.game_over"), true, self.b.accent);
        s.blr(&img, img.get_rect().center_at((cx, (h as f64 * 0.2) as i32)));
        let lh = self.small.get_height() + 8;
        let pw = 380.min(w - 40);
        let panel = Rect::new(cx - pw / 2, (h as f64 * 0.34) as i32, pw, lh * 4 + 36);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(self.b.accent));
        let (x, mut y) = (panel.x + 20, panel.y + 16);
        let mid = MAPS[self.map_idx].id;
        let rows = [
            (tf("td.gameover.wave", &[("n", &(self.wave - 1).to_string())]), p.text),
            (tf("common.points", &[("score", &self.b.score.to_string())]), p.text),
            (tf("td.kills", &[("n", &self.kills.to_string())]), p.text_dim),
            (tf("td.map.best", &[("n", &self.best_of(mid).to_string())]), p.gold),
        ];
        for (text, col) in &rows {
            let img = self.small.render(text, true, *col);
            s.bl(&img, (x, y));
            y += lh;
        }
        let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.4, 0.0, 1.0));
        let img = self.small.render(&t("td.restart_hint"), true, hint_col);
        s.blr(&img, img.get_rect().center_at((cx, panel.bottom() + 26)));
    }
}

impl Game for LamaTowerDefenseGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.anim_t = 0.0;
        self.rules = rules_for(&self.b.mode);
        let letter = self.rules.letter;
        self.bar_keys = TOWER_KEYS.iter().copied().filter(|k| tower_def(k).modes.contains(letter)).collect();
        let saved = store::load_section("lamatowerdef");
        self.best = saved
            .get("best")
            .and_then(|v| v.as_object())
            .map(|m| m.iter().filter_map(|(k, v)| v.as_f64().filter(|_| v.is_number()).map(|f| (k.clone(), f.trunc() as i64))).collect())
            .unwrap_or_default();
        self.map_idx = 0;
        self.towers.clear();
        self.enemies.clear();
        self.shots.clear();
        self.fx.clear();
        self.spawn_queue.clear();
        self.wave = 1;
        self.gold = 0;
        self.lives = 0;
        self.kills = 0;
        self.ff = 1;
        self.show_ranges = false;
        self.sel_card = None;
        self.sel_tower = None;
        self.armed = None;
        self.abil_cd = [0.0; 3];
        self.mapsel_idx = 0;
        self.mouse = None;
        self.bar_scroll = 0;
        self.over_t = 0.0;
        self.bg_cache = None;
        self.ui_bg_cache = None;
        self.layout();
        self.state = State::MapSel;
    }

    fn on_surface_changed(&mut self) {
        self.layout();
        self.bg_cache = None;
        self.ui_bg_cache = None;
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::MapSel => self.ev_mapsel(ev),
            State::GameOver => self.ev_gameover(ev),
            _ => self.ev_play(ev),
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if matches!(self.state, State::MapSel | State::GameOver) {
            self.age_fx(dt);
            return;
        }
        for _ in 0..self.ff {
            let mut rest = dt;
            while rest > 1e-9 && !self.b.game_over {
                let h = rest.min(0.04);
                self.sim(h);
                rest -= h;
            }
        }
        self.age_fx(dt);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.state == State::MapSel {
            self.draw_mapsel(&mut s);
            return;
        }
        let bg = self.field_bg();
        s.bl(&bg, (0, 0));
        self.draw_towers(&mut s);
        self.draw_enemies(&mut s);
        self.draw_shots(&mut s);
        self.draw_fx(&mut s);
        self.draw_ghost(&mut s);
        self.draw_hud(&mut s);
        self.draw_sidebar(&mut s);
        if self.state == State::GameOver {
            self.draw_gameover(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "lamatowerdef",
    class_name: "LamaTowerDefenseGame",
    supports_multiplayer: false,
    modes: &[("classic", "td.mode.classic"), ("compact", "td.mode.compact"), ("maximal", "td.mode.maximal")],
    make: LamaTowerDefenseGame::new,
};
