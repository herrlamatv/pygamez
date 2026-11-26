//! crossyroad.rs
//! =============
//! Crossy Road - endlos ueber Wiesen, Strassen, Fluesse und Gleise huepfen.
//!
//! Uebertragen aus `games/crossyroad.py`. Welt und Regeln in
//! [`crate::crossyroad_world`], Voxel-Grafik in [`crate::crossyroad_draw`].
//! Hier: Eingabe, Ablauf, Kamera, Effekte, HUD, Setup und Figuren-Shop.
//!
//! mem.json-Section "crossy": {coins, unlocked, selected, daily: {date, best},
//! arch: [[tag, bestwert, versuche], ...]} - "daily" ist der heutige Tag,
//! "arch" das Tages-Archiv aller gespielten Tage (`pgz_engine::dailyarchive`).

use std::collections::HashSet;
use std::f64::consts::{PI, TAU};

use pgz_core::{highscore, matches, seedrand, store};
use pgz_engine::dailyarchive::{self as da, Picker, Wahl};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::crossyroad_draw::{self as cd, Voxel};
use crate::crossyroad_world::{self as cw, Kind, Row, World};

const LAND_T: f64 = 0.12;
const BUMP_T: f64 = 0.14;
const NIGHT_TINT: (i32, i32, i32) = (62, 76, 140);
const FOCUS_Y: f64 = 0.66;
const GRID_COLS: usize = 5;
const TABS: [&str; 2] = ["play", "chars"];

fn death_dur(cause: &str) -> f64 {
    if cause == "eagle" {
        1.75
    } else {
        1.25
    }
}

fn dir_of(d: &str) -> (i32, i64) {
    match d {
        "up" => (0, 1),
        "down" => (0, -1),
        "left" => (-1, 0),
        _ => (1, 0),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
        None => String::new(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum St {
    Setup,
    Play,
}

#[derive(Clone, Copy)]
struct Hop {
    fc: f64,
    fr: i64,
    tc: f64,
    tr: i64,
    t: f64,
    drift: f64,
    z0: f64,
    z1: f64,
}

#[derive(Clone)]
struct Death {
    cause: &'static str,
    t: f64,
    dur: f64,
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    age: f64,
    life: f64,
    col: (i32, i32, i32),
    size: f64,
}

struct Data {
    coins: i64,
    unlocked: Vec<String>,
    selected: String,
    daily_date: String,
    daily_best: i64,
    /// Tages-Archiv: [tag, bestwert, versuche]
    arch: Vec<[i64; 3]>,
}

/// `CrossyRoadGame`
pub struct CrossyRoadGame {
    b: GameBase,
    shadows: bool,
    daynight: bool,
    data: Data,
    /// Gespielter Tag (Tagesstrecke, `dailyarchive.today_index()` oder Archiv-Tag).
    day: i64,
    /// Offenes Archiv-Fenster.
    picker: Option<Picker>,
    /// Datums-Leiste im Setup (nur Tagesmodus).
    day_rects: Option<[Rect; 4]>,
    anim: f64,
    state: St,
    tab: &'static str,
    cursor: usize,
    world: Option<World>,
    death: Option<Death>,
    particles: Vec<Particle>,
    ripples: Vec<[f64; 4]>,
    night: Option<Surface>,
    vignette: Option<Surface>,
    last_draw: Option<f64>,
    small: Font,
    tiny: Font,
    huge: Font,
    num: Font,
    mid: Font,
    vox: Voxel,
    vox_gen: u64,
    stage_vox: Option<Voxel>,
    stage_gen: u64,
    card_vox: Option<Voxel>,
    block_cache: Option<(u64, cd::Sprite)>,
    shadow_cache: (u64, Vec<(i32, Surface)>),
    // Setup-Layout
    tab_rects: Vec<Rect>,
    card_rect: Rect,
    opt_w: i32,
    shadow_rects: Vec<Rect>,
    night_rects: Vec<Rect>,
    start_rect: Rect,
    grid_rects: Vec<Rect>,
    detail_rect: Rect,
    buy_rect: Rect,
    // Game Over
    over_rect: Option<Rect>,
    over_btns: Vec<(&'static str, Rect)>,
    over_bg: Option<((i32, i32, Color), Surface)>,
    // Lauf
    clock: f64,
    col: f64,
    row: i64,
    facing: &'static str,
    hop: Option<Hop>,
    buffer: Option<&'static str>,
    log: Option<(usize, i64)>,
    best: i64,
    run_coins: i64,
    taken: HashSet<i64>,
    started: bool,
    creep: f64,
    cam: f64,
    cam_target: f64,
    land_t: f64,
    bump_t: f64,
    hops: i64,
    bell_next: f64,
    passes: HashSet<(i64, i64)>,
    record: bool,
    hs: i64,
    day_best: i64,
    setup_hs: Option<i64>,
    ox: i32,
    oy: i32,
}

impl CrossyRoadGame {
    /// `CrossyRoadGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = CrossyRoadGame {
            b,
            shadows: true,
            daynight: true,
            data: Data { coins: 0, unlocked: vec!["chicken".into()], selected: "chicken".into(), daily_date: String::new(), daily_best: 0, arch: Vec::new() },
            day: 0,
            picker: None,
            day_rects: None,
            anim: 0.0,
            state: St::Setup,
            tab: "play",
            cursor: 0,
            world: None,
            death: None,
            particles: Vec::new(),
            ripples: Vec::new(),
            night: None,
            vignette: None,
            last_draw: None,
            small: f.clone(),
            tiny: f.clone(),
            huge: f.clone(),
            num: f.clone(),
            mid: f,
            vox: Voxel::new(20, true),
            vox_gen: 0,
            stage_vox: None,
            stage_gen: 0,
            card_vox: None,
            block_cache: None,
            shadow_cache: (u64::MAX, Vec::new()),
            tab_rects: Vec::new(),
            card_rect: z,
            opt_w: 0,
            shadow_rects: Vec::new(),
            night_rects: Vec::new(),
            start_rect: z,
            grid_rects: Vec::new(),
            detail_rect: z,
            buy_rect: z,
            over_rect: None,
            over_btns: Vec::new(),
            over_bg: None,
            clock: 0.0,
            col: cw::START_COL as f64,
            row: 0,
            facing: "up",
            hop: None,
            buffer: None,
            log: None,
            best: 0,
            run_coins: 0,
            taken: HashSet::new(),
            started: false,
            creep: 0.0,
            cam: 0.0,
            cam_target: 0.0,
            land_t: 0.0,
            bump_t: 0.0,
            hops: 0,
            bell_next: 0.0,
            passes: HashSet::new(),
            record: false,
            hs: 0,
            day_best: 0,
            setup_hs: None,
            ox: 0,
            oy: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::font(13.max(22.min(h / 30)), false, false);
        self.tiny = ui::font(11.max(18.min(h / 38)), false, false);
        self.huge = ui::font(26.max(h / 11), true, false);
        self.num = ui::font(26.max(h / 10), true, false);
        self.mid = ui::font(15.max(26.min(h / 24)), true, false);
    }

    fn make_voxels(&mut self) {
        self.vox = Voxel::new(20.max(self.b.width / 14), self.shadows);
        self.vox_gen += 1;
        self.stage_vox = None;
        self.card_vox = None;
    }

    // ===================================================== Speicherstand

    fn load() -> Data {
        let raw = store::load_section("crossy");
        let coins = match raw.get("coins") {
            Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64().unwrap_or(0),
            _ => 0,
        };
        let unl: Vec<String> = match raw.get("unlocked") {
            Some(Value::Array(a)) => a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            _ => Vec::new(),
        };
        let unlocked: Vec<String> = cw::char_ids().into_iter().filter(|c| *c == "chicken" || unl.iter().any(|u| u == c)).map(|c| c.to_string()).collect();
        let mut selected = raw.get("selected").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !unlocked.contains(&selected) {
            selected = "chicken".into();
        }
        let daily = raw.get("daily").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let best = match daily.get("best") {
            Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64().unwrap_or(0),
            _ => 0,
        };
        let date = match daily.get("date") {
            None => String::new(),
            Some(Value::String(s)) => s.clone(),
            Some(Value::Null) => "None".into(),
            Some(Value::Bool(b)) => if *b { "True".into() } else { "False".into() },
            Some(v) => v.to_string(),
        };
        let daily_date: String = date.chars().take(10).collect();
        let daily_best = if best > 0 { best } else { 0 };
        let mut arch: Vec<[i64; 3]> = da::clean_rows(raw.get("arch"), 3)
            .into_iter()
            .filter_map(|r| {
                let v: Vec<i64> = r.iter().filter_map(|x| x.as_i64().filter(|i| *i >= 0)).collect();
                (v.len() == 3).then(|| [v[0], v[1], v[2]])
            })
            .collect();
        // Altbestand: den bisher einzigen Tagesbestwert ins Archiv uebernehmen.
        if daily_best > 0 {
            let idx = seedrand::day_index_checked(&daily_date).unwrap_or(-1);
            if idx >= 0 {
                match da::find(&arch, idx) {
                    None => da::upsert(&mut arch, [idx, daily_best, 1]),
                    Some(i) => {
                        if arch[i][1] < daily_best {
                            arch[i][1] = daily_best;
                        }
                    }
                }
            }
        }
        Data { coins: 0.max(coins), unlocked, selected, daily_date, daily_best, arch }
    }

    fn save(&self) {
        let mut m = Map::new();
        m.insert("coins".into(), json!(self.data.coins));
        m.insert("unlocked".into(), json!(self.data.unlocked));
        m.insert("selected".into(), json!(self.data.selected));
        let mut d = Map::new();
        d.insert("date".into(), json!(self.data.daily_date));
        d.insert("best".into(), json!(self.data.daily_best));
        m.insert("daily".into(), Value::Object(d));
        m.insert("arch".into(), json!(self.data.arch));
        store::save_section("crossy", m);
    }

    /// Bestwert des gespielten Tags (heute oder Archiv-Tag).
    fn daily_best(&self) -> i64 {
        da::find(&self.data.arch, self.day).map(|i| self.data.arch[i][1]).unwrap_or(0)
    }

    fn is_archive(&self) -> bool {
        self.b.mode == "daily" && self.day != da::today_index()
    }

    fn days(&self) -> Vec<i64> {
        da::available(self.data.arch.iter().map(|r| r[0]), None)
    }

    fn step_day(&mut self, d: i64) {
        let days = self.days();
        let i = days.iter().position(|x| *x == self.day).unwrap_or(days.len() - 1) as i64;
        let j = 0.max((days.len() as i64 - 1).min(i + d)) as usize;
        if days[j] != self.day {
            self.day = days[j];
            self.b.play_sound("move");
        }
    }

    fn open_picker(&mut self) {
        let info = self
            .data
            .arch
            .iter()
            .map(|r| (r[0], ("played".to_string(), tf("arch.best", &[("n", &r[1].to_string())]))))
            .collect();
        self.picker = Some(Picker::new("crossy", self.b.accent, self.day, info));
        self.b.play_sound("click");
    }

    fn setup_best(&mut self) -> i64 {
        if self.setup_hs.is_none() {
            self.setup_hs = Some(highscore::get(self.b.key));
        }
        self.setup_hs.unwrap_or(0)
    }

    fn save_setting(&mut self, key: &str, value: bool) {
        self.b.set_opt("crossy", key, json!(value));
    }

    // ===================================================== Runde

    fn wrow(&mut self, r: i64) -> Row {
        self.world.as_mut().expect("world").row(r).clone()
    }

    fn new_run(&mut self) {
        let seed: u64 = if self.b.mode == "daily" {
            seedrand::daily_seed("crossy", Some(&seedrand::date_of(self.day))) as u64
        } else {
            py::with(|r| r.getrandbits(32))
        };
        self.world = Some(World::new(seed));
        self.clock = 0.0;
        self.col = cw::START_COL as f64;
        self.row = 0;
        self.facing = "up";
        self.hop = None;
        self.buffer = None;
        self.log = None;
        self.best = 0;
        self.b.score = 0;
        self.run_coins = 0;
        self.taken.clear();
        self.started = false;
        self.creep = -cw::CREEP_LAG;
        self.cam = 0.0;
        self.cam_target = 0.0;
        self.death = None;
        self.particles.clear();
        self.ripples.clear();
        self.land_t = 0.0;
        self.bump_t = 0.0;
        self.hops = 0;
        self.bell_next = 0.0;
        self.passes.clear();
        self.record = false;
        self.b.game_over = false;
        self.state = St::Play;
        self.last_draw = None;
        self.hs = highscore::get(self.b.key);
        self.day_best = self.daily_best();
        self.b.play_sound("click");
    }

    fn finish_run(&mut self) {
        if self.b.mode == "daily" {
            let row = da::find(&self.data.arch, self.day).map(|i| self.data.arch[i]).unwrap_or([self.day, 0, 0]);
            self.record = self.best > row[1];
            da::upsert(&mut self.data.arch, [self.day, row[1].max(self.best), row[2] + 1]);
            let today = seedrand::today_str();
            if seedrand::date_of(self.day) == today {
                // "daily" bleibt der heutige Tag (wie bisher)
                if self.data.daily_date != today {
                    self.data.daily_date = today;
                    self.data.daily_best = 0;
                }
                if self.best > self.data.daily_best {
                    self.data.daily_best = self.best;
                }
            }
        } else {
            self.record = self.best > self.hs;
        }
        self.save();
        self.b.game_over = true;
        self.b.play_sound("gameover");
        self.layout_over();
    }

    // ===================================================== Eingabe

    fn press(&mut self, d: &'static str) {
        if self.hop.is_some() {
            self.buffer = Some(d);
            return;
        }
        self.try_hop(d);
    }

    fn try_hop(&mut self, d: &'static str) {
        let (dc, dr) = dir_of(d);
        self.facing = d;
        let fr = self.row;
        let tr = fr + dr;
        let fc = self.col;
        let cur = self.wrow(fr);
        let dst = self.wrow(tr);
        let on_log = self.log.is_some();
        let mut drift = 0.0;
        let tc = if dr == 0 {
            if on_log {
                drift = cur.dir as f64 * cur.speed;
            }
            fc + dc as f64
        } else if dst.kind == Kind::River && dst.pads.is_none() {
            fc
        } else {
            (fc + 0.5).floor()
        };
        if Self::blocked(&dst, tc) {
            self.bump_t = BUMP_T;
            audio::tone(170.0, 0.05, Some(&self.b.settings), "square", 0.10);
            return;
        }
        if !self.started {
            self.started = true;
        }
        self.hop = Some(Hop { fc, fr, tc, tr, t: 0.0, drift, z0: Self::stand_z(&cur), z1: Self::stand_z(&dst) });
        self.log = None;
        self.hops += 1;
        audio::tone((540 + 50 * (self.hops % 3)) as f64, 0.045, Some(&self.b.settings), "sine", 0.16);
    }

    fn blocked(row: &Row, tc: f64) -> bool {
        if row.kind == Kind::River && row.pads.is_none() {
            return !(-0.5 <= tc && tc <= cw::COLS as f64 - 0.5);
        }
        let c = (tc + 0.5).floor() as i64;
        if c < 0 || c >= cw::COLS as i64 {
            return true;
        }
        row.trees.as_ref().is_some_and(|t| !t.is_empty() && t[c as usize] != 0)
    }

    fn stand_z(row: &Row) -> f64 {
        let lvl = row.kind.level();
        if row.kind == Kind::River {
            return lvl + if row.pads.is_some() { 0.05 } else { 0.26 };
        }
        lvl
    }

    fn hop_frac(&self) -> f64 {
        match self.hop {
            Some(h) => 1f64.min(h.t / cw::HOP_T),
            None => 1.0,
        }
    }

    fn pos(&mut self) -> (f64, f64, f64) {
        let Some(h) = self.hop else {
            let r = self.row;
            let mut z = Self::stand_z(&self.wrow(r));
            if let Some(l) = self.log {
                z += 0.012 * (self.clock * 3.0 + l.0 as f64).sin();
            }
            return (self.col, self.row as f64, z);
        };
        let p = self.hop_frac();
        let x = h.fc + (h.tc - h.fc) * p + h.drift * h.t;
        let y = h.fr as f64 + (h.tr - h.fr) as f64 * p;
        let z = h.z0 + (h.z1 - h.z0) * p + cw::HOP_Z * (PI * p).sin();
        (x, y, z)
    }

    fn row_now(&self) -> i64 {
        match self.hop {
            None => self.row,
            Some(h) => {
                if self.hop_frac() >= 0.5 {
                    h.tr
                } else {
                    h.fr
                }
            }
        }
    }

    fn land(&mut self) {
        let Some(h) = self.hop.take() else {
            return;
        };
        self.row = h.tr;
        self.col = h.tc + h.drift * cw::HOP_T;
        self.land_t = LAND_T;
        let r = self.row;
        let row = self.wrow(r);
        if row.kind == Kind::River {
            if let Some(pads) = &row.pads {
                let c = (self.col + 0.5).floor() as i64;
                if (0..cw::COLS as i64).contains(&c) && pads[c as usize] != 0 {
                    self.col = c as f64;
                    self.ripple(c as f64 + 0.5, self.row as f64 + 0.5, 0.5);
                } else {
                    self.die("water");
                    return;
                }
            } else {
                let Some(hit) = cw::log_at(&row, self.col + 0.5, self.clock) else {
                    self.die("water");
                    return;
                };
                self.log = Some(hit);
                self.col = cw::obj_x(&row, &row.objs[hit.0], self.clock) + hit.1 as f64;
                self.ripple(self.col + 0.5, self.row as f64 + 0.5, 0.6);
            }
        } else {
            self.col = (self.col + 0.5).floor();
        }
        if let Some(coin) = row.coin {
            if !self.taken.contains(&self.row) && coin.0 as i64 == (self.col + 0.5).floor() as i64 {
                self.taken.insert(self.row);
                self.run_coins += coin.1 as i64;
                self.data.coins += coin.1 as i64;
                self.sparkle(coin.0 as f64 + 0.5, self.row as f64 + 0.5, Self::stand_z(&row) + 0.4, coin.1 > 1);
                self.b.play_sound(if coin.1 > 1 { "powerup" } else { "eat" });
            }
        }
        if self.row > self.best {
            self.best = self.row;
            if self.b.mode != "daily" {
                self.b.score = self.best;
            }
            let rr = self.row - 1;
            if row.kind != Kind::Rail && self.world.as_mut().expect("world").rail_run(rr) >= 5 {
                self.b.ach_event("crossy_train", None);
            }
        }
        if self.buffer.is_some() && self.death.is_none() {
            let d = self.buffer.take().unwrap_or("up");
            self.try_hop(d);
        }
    }

    fn collide(&mut self) {
        let (x, _, _) = self.pos();
        let cx = x + 0.5;
        let rr = self.row_now();
        let row = self.wrow(rr);
        if row.kind == Kind::Road {
            if cw::vehicle_at(&row, cx, self.clock) >= 0 {
                self.die("car");
            }
        } else if row.kind == Kind::Rail && cw::train_hits(&row, cx, self.clock) {
            self.die("train");
        }
    }

    fn camera(&mut self, dt: f64) {
        if self.started {
            self.creep += cw::creep_rate(self.best) * dt;
        }
        self.creep = self.creep.max(self.best as f64 - cw::CREEP_LAG);
        self.cam_target = self.creep.max(self.best as f64);
        self.cam += (self.cam_target - self.cam) * 1f64.min(dt * 5.0);
        if self.started && self.cam_target - self.row_now() as f64 > cw::EAGLE_ROWS {
            self.die("eagle");
        }
    }

    fn danger(&self) -> f64 {
        if self.world.is_none() || !self.started || self.death.is_some() {
            return 0.0;
        }
        let behind = self.cam_target - self.row_now() as f64;
        0f64.max(1f64.min((behind - (cw::EAGLE_ROWS - 1.3)) / 1.3))
    }

    fn sounds(&mut self) {
        let mut ring = false;
        for r in self.row - 3..self.row + 7 {
            if r < 0 {
                continue;
            }
            let row = self.wrow(r);
            if row.kind != Kind::Rail {
                continue;
            }
            let (warn, head, _len) = cw::train_state(&row, self.clock);
            if warn && head.is_none() {
                ring = true;
            }
            if head.is_some() && (r - self.row).abs() <= 3 {
                let n = py::ffdiv(self.clock + row.phase, row.period) as i64;
                if !self.passes.contains(&(r, n)) {
                    self.passes.insert((r, n));
                    audio::tone(90.0, 0.45, Some(&self.b.settings), "noise", 0.22);
                }
            }
        }
        if ring && self.clock >= self.bell_next {
            self.bell_next = self.clock + 0.3;
            audio::tone(1320.0, 0.07, Some(&self.b.settings), "square", 0.10);
        }
    }

    fn die(&mut self, cause: &'static str) {
        let (x, y, z) = self.pos();
        self.death = Some(Death { cause, t: 0.0, dur: death_dur(cause), x, y, z });
        self.hop = None;
        self.buffer = None;
        self.log = None;
        let col = cw::char_color(&self.data.selected);
        match cause {
            "car" => {
                self.b.play_sound("hit");
                audio::tone(620.0, 0.22, Some(&self.b.settings), "square", 0.12);
                self.burst(x + 0.5, y + 0.5, z + 0.3, col, 14, 2.2, 0.0, 3.5);
                self.b.rumble(180);
            }
            "train" => {
                self.b.play_sound("explode");
                let dir = self.wrow(py::round(y)).dir;
                self.burst(x + 0.5, y + 0.5, z + 0.4, col, 26, 4.0, dir as f64 * 6.0, 3.5);
                self.b.rumble(260);
            }
            "water" | "edge" => {
                audio::tone(160.0, 0.3, Some(&self.b.settings), "noise", 0.3);
                audio::tone(420.0, 0.12, Some(&self.b.settings), "sine", 0.15);
                let wz = Kind::River.level();
                if let Some(d) = self.death.as_mut() {
                    d.z = wz;
                }
                self.ripple(x + 0.5, y + 0.5, 1.0);
                self.ripple(x + 0.5, y + 0.5, 0.6);
                self.burst(x + 0.5, y + 0.5, wz + 0.1, (236, 250, 255), 22, 1.6, 0.0, 6.0);
                self.burst(x + 0.5, y + 0.5, wz + 0.1, (140, 214, 250), 12, 2.2, 0.0, 4.0);
                self.b.rumble(140);
            }
            _ => {
                self.b.play_sound("shoot");
                self.b.rumble(220);
            }
        }
    }

    // ----- Effekte -----------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn burst(&mut self, x: f64, y: f64, z: f64, col: (i32, i32, i32), n: usize, speed: f64, push: f64, up: f64) {
        for _ in 0..n {
            let a = py::uniform(0.0, TAU);
            let sp = py::uniform(0.3, 1.0) * speed;
            let vx = a.cos() * sp + push * py::uniform(0.4, 1.0);
            let vy = a.sin() * sp * 0.6;
            let vz = py::uniform(0.5, 1.0) * up;
            let life = py::uniform(0.6, 1.1);
            let size = py::uniform(0.06, 0.12);
            self.particles.push(Particle { x, y, z, vx, vy, vz, age: 0.0, life, col, size });
        }
    }

    fn sparkle(&mut self, x: f64, y: f64, z: f64, big: bool) {
        for _ in 0..(if big { 16 } else { 9 }) {
            let a = py::uniform(0.0, TAU);
            let sp = py::uniform(0.6, 1.6);
            let vz = py::uniform(1.5, 3.0);
            let life = py::uniform(0.35, 0.6);
            let size = py::uniform(0.05, 0.09);
            self.particles.push(Particle { x, y, z, vx: a.cos() * sp, vy: a.sin() * sp * 0.5, vz, age: 0.0, life, col: (255, 222, 90), size });
        }
    }

    fn ripple(&mut self, x: f64, y: f64, strength: f64) {
        self.ripples.push([x, y, 0.0, strength]);
    }

    fn step_effects(&mut self, dt: f64) {
        let mut keep = Vec::new();
        for mut p in std::mem::take(&mut self.particles) {
            p.age += dt;
            if p.age >= p.life {
                continue;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.z += p.vz * dt;
            p.vz -= 9.0 * dt;
            p.vx *= 1.0 - 1.5 * dt;
            if p.z < -0.2 {
                p.z = -0.2;
                p.vz = 0.0;
            }
            keep.push(p);
        }
        let start = keep.len().saturating_sub(240);
        self.particles = keep.split_off(start);
        self.ripples.retain(|r| r[2] + dt < 0.8);
        for r in self.ripples.iter_mut() {
            r[2] += dt;
        }
    }

    // ===================================================== Setup-Screen

    fn layout_setup(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let tab_h = 22.max(32.min(h / 15));
        let tab_w = 170.min((w - 40) / 2);
        let tab_y = (h as f64 * 0.205) as i32;
        self.tab_rects = vec![Rect::new(cx - tab_w - 4, tab_y, tab_w, tab_h), Rect::new(cx + 4, tab_y, tab_w, tab_h)];
        let top = tab_y + tab_h + 10.max(h / 36);
        let bw = (380.max((w as f64 * 0.62) as i32)).min(w - 40);
        let bh = 28.max(46.min(h / 16));
        let card_h = 96.max((h as f64 * 0.30) as i32);
        let lab = 20.max(self.tiny.get_height() + 10);
        let gap1 = lab + 0.max(py::fdiv32(h - 360, 40));
        let gap2 = 12.max(h / 30);
        let mut used = card_h + gap1 + bh + gap2 + bh + 6;
        if self.b.mode == "daily" {
            used += bh + gap2; // Datums-Leiste (Tages-Archiv)
        }
        let free = (h - 44) - top - used;
        let ptop = top + 0.max(py::fdiv32(free, 2));
        self.card_rect = Rect::new(cx - bw / 2, ptop, bw, card_h);
        let mut y = self.card_rect.bottom() + gap1;
        let grp = (bw - 18) as f64 / 2.0;
        self.opt_w = grp as i32;
        let pair = |x0: f64, y: i32| {
            let wd = (grp - 8.0) / 2.0;
            vec![Rect::new(x0 as i32, y, wd as i32, bh), Rect::new((x0 + wd + 8.0) as i32, y, wd as i32, bh)]
        };
        self.shadow_rects = pair(cx as f64 - bw as f64 / 2.0, y);
        self.night_rects = pair(cx as f64 + bw as f64 / 2.0 - grp, y);
        y += bh + gap2;
        self.day_rects = None;
        if self.b.mode == "daily" {
            self.day_rects = Some(da::day_row_rects(cx - bw / 2, y, bw, bh));
            y += bh + gap2;
        }
        let sw = 190.max((bw as f64 * 0.42) as i32);
        self.start_rect = Rect::new(cx - sw / 2, y, sw, bh + 6);
        let gw = (w - 36).min(760);
        let gap = 6.max(w / 100);
        let detail_h = bh + 6;
        let bottom = h - 26 - 8.max(h / 40);
        let avail = bottom - top - detail_h - gap * 2;
        let cwid = (gw - gap * (GRID_COLS as i32 - 1)) as f64 / GRID_COLS as f64;
        let chei = (cwid * 1.12).min((avail - gap) as f64 / 2.0);
        self.grid_rects = (0..cw::CHARACTERS.len())
            .map(|i| {
                let (gx, gy) = ((i % GRID_COLS) as f64, (i / GRID_COLS) as f64);
                Rect::new((cx as f64 - gw as f64 / 2.0 + gx * (cwid + gap as f64)) as i32, (top as f64 + gy * (chei + gap as f64)) as i32, cwid as i32, chei as i32)
            })
            .collect();
        let dy = self.grid_rects.last().map(|r| r.bottom()).unwrap_or(0) + gap * 2;
        self.detail_rect = Rect::new(cx - gw / 2, dy, gw, detail_h);
        self.buy_rect = Rect::new(self.detail_rect.right() - (gw as f64 * 0.36) as i32, dy + 3, (gw as f64 * 0.36) as i32 - 3, detail_h - 6);
        self.card_vox = None;
    }

    fn char_index(&self, cid: &str) -> usize {
        cw::char_ids().iter().position(|c| *c == cid).unwrap_or(0)
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if let Some(pk) = self.picker.as_mut() {
            if let Some(res) = pk.handle_event(ev, Some(&self.b)) {
                if let Wahl::Pick(day) = res {
                    self.day = day;
                    self.b.play_sound("select");
                } else {
                    self.b.play_sound("click");
                }
                self.picker = None;
            }
            return;
        }
        if ev.kind == Kind_::MouseDown {
            if let Some(i) = self.tab_rects.iter().position(|r| r.hit(ev.pos)) {
                self.set_tab(TABS[i]);
                return;
            }
        } else if ev.kind == Kind_::KeyDown && ev.key_in(&["Tab", "ISO_Left_Tab"]) {
            let t = if self.tab == "play" { "chars" } else { "play" };
            self.set_tab(t);
            return;
        }
        if self.tab == "chars" {
            self.handle_chars(ev);
            return;
        }
        if ev.kind == Kind_::KeyDown {
            let k = ev.key.as_str();
            if k == "Return" || k == "space" {
                self.new_run();
            } else if k == "Left" || self.b.is_action(k, "left", None) {
                self.cycle_char(-1);
            } else if k == "Right" || self.b.is_action(k, "right", None) {
                self.cycle_char(1);
            } else if (k == "h" || k == "H") && self.b.key_is_free(k) {
                self.toggle(true);
            } else if (k == "n" || k == "N") && self.b.key_is_free(k) {
                self.toggle(false);
            } else if self.b.mode == "daily" && k == "Prior" {
                self.step_day(-1);
            } else if self.b.mode == "daily" && k == "Next" {
                self.step_day(1);
            } else if self.b.mode == "daily" && (k == "k" || k == "K") && self.b.key_is_free(k) {
                self.open_picker();
            }
        } else if ev.kind == Kind_::MouseDown {
            if let Some([r_prev, r_date, r_next, r_arch]) = self.day_rects {
                if r_prev.hit(ev.pos) {
                    self.step_day(-1);
                    return;
                }
                if r_next.hit(ev.pos) {
                    self.step_day(1);
                    return;
                }
                if r_date.hit(ev.pos) || r_arch.hit(ev.pos) {
                    self.open_picker();
                    return;
                }
            }
            for (shadow_flag, rects) in [(true, self.shadow_rects.clone()), (false, self.night_rects.clone())] {
                for (i, rc) in rects.iter().enumerate() {
                    if rc.hit(ev.pos) {
                        let cur = if shadow_flag { self.shadows } else { self.daynight };
                        if cur != (i == 0) {
                            self.toggle(shadow_flag);
                        }
                        return;
                    }
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.new_run();
            } else if self.card_rect.hit(ev.pos) {
                self.set_tab("chars");
            }
        }
    }

    fn set_tab(&mut self, tab: &'static str) {
        if tab != self.tab {
            self.tab = tab;
            self.cursor = self.char_index(&self.data.selected.clone());
            self.b.play_sound("click");
        }
    }

    /// `_toggle("shadows")` (true) bzw. `_toggle("daynight")` (false).
    fn toggle(&mut self, shadows: bool) {
        if shadows {
            self.shadows = !self.shadows;
            let v = self.shadows;
            self.save_setting("shadows", v);
            self.make_voxels();
        } else {
            self.daynight = !self.daynight;
            let v = self.daynight;
            self.save_setting("daynight", v);
        }
        self.b.play_sound("click");
    }

    fn cycle_char(&mut self, step: i64) {
        let owned: Vec<&str> = cw::char_ids().into_iter().filter(|c| self.data.unlocked.iter().any(|u| u == c)).collect();
        let i = owned.iter().position(|c| *c == self.data.selected).unwrap_or(0) as i64;
        self.data.selected = owned[py::pmod(i + step, owned.len() as i64) as usize].to_string();
        self.save();
        self.b.play_sound("select");
    }

    fn handle_chars(&mut self, ev: &InputEvent) {
        let n = cw::CHARACTERS.len() as i64;
        if ev.kind == Kind_::KeyDown {
            let k = ev.key.as_str();
            let cur = self.cursor as i64;
            if k == "Escape" {
                self.set_tab("play");
            } else if k == "Return" || k == "space" {
                self.activate_char(self.cursor);
            } else if k == "Left" || self.b.is_action(k, "left", None) {
                self.cursor = py::pmod(cur - 1, n) as usize;
                self.b.play_sound("move");
            } else if k == "Right" || self.b.is_action(k, "right", None) {
                self.cursor = py::pmod(cur + 1, n) as usize;
                self.b.play_sound("move");
            } else if k == "Up" || self.b.is_action(k, "up", None) {
                self.cursor = py::pmod(cur - GRID_COLS as i64, n) as usize;
                self.b.play_sound("move");
            } else if k == "Down" || self.b.is_action(k, "down", None) {
                self.cursor = py::pmod(cur + GRID_COLS as i64, n) as usize;
                self.b.play_sound("move");
            }
        } else if ev.kind == Kind_::MouseDown {
            if let Some(i) = self.grid_rects.iter().position(|r| r.hit(ev.pos)) {
                self.cursor = i;
                if self.data.unlocked.iter().any(|u| u == cw::char_ids()[i]) {
                    self.activate_char(i);
                } else {
                    self.b.play_sound("move");
                }
                return;
            }
            if self.buy_rect.hit(ev.pos) {
                self.activate_char(self.cursor);
            }
        }
    }

    fn activate_char(&mut self, i: usize) -> bool {
        let cid = cw::char_ids()[i];
        if self.data.unlocked.iter().any(|u| u == cid) {
            if self.data.selected != cid {
                self.data.selected = cid.to_string();
                self.save();
            }
            self.b.play_sound("select");
            return true;
        }
        let price = cw::price(cid);
        if self.data.coins < price {
            self.b.play_sound("hit");
            return false;
        }
        self.data.coins -= price;
        let unl = self.data.unlocked.clone();
        self.data.unlocked = cw::char_ids().into_iter().filter(|c| unl.iter().any(|u| u == c) || *c == cid).map(|c| c.to_string()).collect();
        self.data.selected = cid.to_string();
        self.save();
        self.b.play_sound("win");
        let r = self.grid_rects[i];
        ui::spawn_burst(r.centerx() as f64, r.centery() as f64, Some(self.b.accent), 18, 260.0);
        self.b.ach_event("crossy_char", None);
        true
    }

    // ===================================================== Game Over

    fn text_h(f: &Font) -> i32 {
        f.size("\u{00C4}Ggy").1
    }

    fn layout_over(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let pw = (w - 30).min(360.max((w as f64 * 0.56) as i32));
        let bh = 28.max(42.min(h / 17));
        let rows_h = Self::text_h(&self.huge) + Self::text_h(&self.tiny) + Self::text_h(&self.b.font) + 2 * Self::text_h(&self.small) + 5 * 8;
        let ph = 14 + rows_h + 12 + bh + 8 + Self::text_h(&self.tiny) + 10;
        let top = 8.max(py::fdiv32(h - 56 - ph, 2));
        let rect = Rect::new(w / 2 - pw / 2, top, pw, ph);
        self.over_rect = Some(rect);
        let by = rect.bottom() - 10 - Self::text_h(&self.tiny) - 8 - bh;
        let gap = 8;
        let bw = (pw - 40 - 2 * gap) as f64 / 3.0;
        self.over_btns = ["again", "chars", "setup"].iter().enumerate().map(|(i, k)| (*k, Rect::new((rect.x as f64 + 20.0 + i as f64 * (bw + gap as f64)) as i32, by, bw as i32, bh))).collect();
    }

    fn handle_over(&mut self, ev: &InputEvent) {
        if ev.kind == Kind_::KeyDown {
            if ev.repeat {
                return;
            }
            match ev.key.as_str() {
                "Return" | "space" => self.new_run(),
                "f" | "F" => self.to_setup("chars"),
                "s" | "S" => self.to_setup("play"),
                _ => {}
            }
        } else if ev.kind == Kind_::MouseDown {
            if let Some(&(key, _)) = self.over_btns.iter().find(|(_, r)| r.hit(ev.pos)) {
                if key == "again" {
                    self.new_run();
                } else {
                    self.to_setup(if key == "chars" { "chars" } else { "play" });
                }
            }
        }
    }

    fn to_setup(&mut self, tab: &'static str) {
        self.setup_hs = None;
        self.b.game_over = false;
        self.b.score = 0;
        self.state = St::Setup;
        self.tab = tab;
        self.cursor = self.char_index(&self.data.selected.clone());
        self.b.play_sound("click");
    }

    // ===================================================== Zeichnen: Welt

    fn setup_proj(&mut self) {
        let v = &self.vox;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let f = self.cam;
        self.ox = py::round(w / 2.0 - (cw::COLS as f64 / 2.0) * v.tw as f64 - (f + 2.5) * v.sk as f64) as i32;
        self.oy = py::round(h * FOCUS_Y + (f + 0.5) * v.th as f64) as i32;
    }

    fn scr(&self, x: f64, y: f64, z: f64) -> (f64, f64) {
        let v = &self.vox;
        (self.ox as f64 + x * v.tw as f64 + y * v.sk as f64, self.oy as f64 - y * v.th as f64 - z * v.zh as f64)
    }

    fn blit_key(&mut self, s: &mut Surface, key: &str, x: f64, y: f64, z: f64) {
        let (surf, ax, ay) = self.vox.sprite(key, true);
        let (px, py_) = self.scr(x, y, z);
        s.bl(&surf, (py::round(px) as i32 - ax, py::round(py_) as i32 - ay));
    }

    fn draw_world(&mut self, s: &mut Surface) {
        let h = self.b.height;
        self.setup_proj();
        let (th, zh) = (self.vox.th as f64, self.vox.zh as f64);
        let r_hi = (self.oy as f64 / th).ceil() as i64 + 1;
        let r_lo = ((self.oy as f64 - h as f64 - 2.2 * zh) / th).floor() as i64 - 1;
        let clock = self.clock;
        let p_row: Option<i64> = match &self.death {
            None => Some(match self.hop {
                Some(hp) => hp.fr.min(hp.tr),
                None => self.row,
            }),
            Some(d) if ["car", "water", "edge"].contains(&d.cause) => Some((d.y + 0.5).floor() as i64),
            _ => None,
        };
        let mut r = r_hi;
        while r >= r_lo {
            let row = self.wrow(r);
            let kind = row.kind;
            let lvl = kind.level();
            let prev = self.wrow(r - 1);
            let plvl = prev.kind.level();
            let lower = if plvl < lvl { Some(plvl) } else { None };
            let line = kind == Kind::Road && self.wrow(r + 1).kind == Kind::Road;
            let (surf, ax, ay) = self.vox.strip(kind, r & 1, lower, line);
            let (sx, sy) = self.scr(0.0, r as f64, 0.0);
            s.bl(&surf, (sx as i32 - ax, sy as i32 - ay));
            // (Ebene, Sortier-x, Schluessel, x, z)
            let mut items: Vec<(i32, f64, Option<String>, f64, f64)> = Vec::new();
            match kind {
                Kind::Grass => {
                    let trees = row.trees.clone().unwrap_or_default();
                    for c in 0..cw::COLS {
                        if trees[c as usize] != 0 {
                            items.push((1, c as f64, Some(format!("tree{}", trees[c as usize])), c as f64, lvl));
                        }
                    }
                    for c in [-4, -3, -2, -1, cw::COLS, cw::COLS + 1, cw::COLS + 2, cw::COLS + 3] {
                        let hsh = cw::deco_hash(r, c as i64);
                        let lim = if c == -1 || c == cw::COLS { 70 } else { 45 };
                        if hsh % 100 < lim {
                            items.push((1, c as f64, Some(format!("tree{}", 1 + (hsh >> 8) % 3)), c as f64, lvl));
                        }
                    }
                }
                Kind::Road => {
                    for o in &row.objs {
                        let x = cw::obj_x(&row, o, clock);
                        if -7.0 < x && x < cw::COLS as f64 + 5.0 {
                            let name = if o.1 > 2.0 { format!("truck{}", o.2) } else { format!("car{}", o.2) };
                            items.push((1, x, Some(name + if row.dir > 0 { "" } else { "<" }), x, lvl));
                        }
                    }
                }
                Kind::River => {
                    if let Some(pads) = &row.pads {
                        for c in 0..cw::COLS {
                            if pads[c as usize] != 0 {
                                let bob = 0.012 * (clock * 2.0 + c as f64).sin();
                                let fl = (cw::deco_hash(r, c as i64) % 3 == 0) as i32;
                                items.push((0, c as f64, Some(format!("pad{fl}")), c as f64, lvl + bob));
                            }
                        }
                    } else {
                        for (i, o) in row.objs.iter().enumerate() {
                            let x = cw::obj_x(&row, o, clock);
                            if -7.0 < x && x < cw::COLS as f64 + 5.0 {
                                let bob = 0.012 * (clock * 3.0 + i as f64).sin();
                                items.push((0, x, Some(format!("log{}", o.1 as i64)), x, lvl + bob));
                            }
                        }
                    }
                }
                Kind::Rail => {
                    let (warn, head, _len) = cw::train_state(&row, clock);
                    let lit = if warn { 1 + ((clock * 6.0) as i64) % 2 } else { 0 };
                    items.push((1, -1.2, Some(format!("signal{lit}")), -1.2, lvl));
                    if let Some(head) = head {
                        let n = row.cars + 1;
                        for k in 0..n {
                            let (x, key) = if row.dir > 0 {
                                (head - cw::TRAIN_CAR * (k + 1) as f64, if k == 0 { "engine" } else { "wagon" })
                            } else {
                                (head + cw::TRAIN_CAR * k as f64, if k == 0 { "engine<" } else { "wagon<" })
                            };
                            if -9.0 < x && x < cw::COLS as f64 + 6.0 {
                                items.push((1, x, Some(key.to_string()), x, lvl));
                            }
                        }
                    }
                }
            }
            if let Some(coin) = row.coin {
                if !self.taken.contains(&r) {
                    let bob = 0.06 + 0.05 * (self.anim * 3.0 + r as f64).sin();
                    items.push((1, coin.0 as f64 - 0.01, Some(if coin.1 > 1 { "bigcoin".into() } else { "coin".into() }), coin.0 as f64, lvl + bob));
                }
            }
            if p_row == Some(r) {
                items.push((2, 0.0, None, 0.0, 0.0));
            }
            items.sort_by(|a, b| {
                let la = if a.0 != 2 { a.0 } else { 1 };
                let lb = if b.0 != 2 { b.0 } else { 1 };
                la.cmp(&lb).then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            });
            for (_layer, _sx, key, x, z) in items {
                match key {
                    None => self.draw_player(s),
                    Some(k) => self.blit_key(s, &k, x, r as f64, z),
                }
            }
            if !self.ripples.is_empty() {
                self.draw_ripples(s, r);
            }
            r -= 1;
        }
        self.draw_particles(s);
        self.draw_danger(s);
    }

    fn draw_player(&mut self, s: &mut Surface) {
        let cid = self.data.selected.clone();
        if let Some(d) = self.death.clone() {
            let (x, y, z) = (d.x, d.y, d.z);
            if d.cause == "car" {
                let q = 1f64.min(d.t / 0.08);
                let (sx, sy) = (1.0 + 0.4 * q, 1.0 - 0.8 * q);
                self.player_sprite(s, &cid, self.facing, x, y, z, sx, sy, false, None);
            } else if (d.cause == "water" || d.cause == "edge") && d.t < 0.14 {
                let sink = d.t / 0.14;
                self.player_sprite(s, &cid, self.facing, x, y, z - 0.5 * sink, 1.0, 1.0 - 0.6 * sink, false, None);
            }
            return;
        }
        let (x, y, z) = self.pos();
        let (sx, sy);
        if self.hop.is_some() {
            let k = (PI * self.hop_frac()).sin();
            sx = 1.0 - 0.10 * k;
            sy = 1.0 + 0.16 * k;
        } else if self.land_t > 0.0 {
            let q = self.land_t / LAND_T;
            sx = 1.0 + 0.16 * q;
            sy = 1.0 - 0.24 * q;
        } else if self.bump_t > 0.0 {
            let q = (PI * self.bump_t / BUMP_T).sin();
            sx = 1.0 + 0.08 * q;
            sy = 1.0 - 0.12 * q;
        } else {
            let k = 0.5 + 0.5 * (self.anim * 4.0).sin();
            sy = 1.0 - 0.03 * k;
            sx = 1.0 + 0.02 * k;
        }
        let rn = self.row_now();
        let ground = Self::stand_z(&self.wrow(rn));
        self.player_sprite(s, &cid, self.facing, x, y, z, sx, sy, true, Some(ground));
    }

    #[allow(clippy::too_many_arguments)]
    fn player_sprite(&mut self, s: &mut Surface, cid: &str, facing: &str, x: f64, y: f64, mut z: f64, sx: f64, sy: f64, shadow: bool, ground: Option<f64>) {
        if cw::hovers(cid) {
            z += 0.14 + 0.05 * (self.anim * 3.0).sin();
        }
        if shadow && self.shadows {
            let gz = ground.unwrap_or(z);
            let lift = 0f64.max(z - gz);
            self.shadow(s, x + 0.5, y + 0.5, gz, 0.45f64.max(1.0 - lift * 0.8));
        }
        let key = format!("{cid}:{facing}");
        let (surf, _ax, _ay) = self.vox.scaled(&key, sx, sy);
        let (_b, bax, bay) = self.vox.sprite(&key, false);
        let v = &self.vox;
        let fx = bax as f64 + 0.5 * v.tw as f64 + 0.5 * v.sk as f64;
        let fy = bay as f64 - 0.5 * v.th as f64;
        let (px, py_) = self.scr(x + 0.5, y + 0.5, z);
        s.bl(&surf, (py::round(px - fx * sx) as i32, py::round(py_ - fy * sy) as i32));
    }

    fn shadow(&mut self, s: &mut Surface, cx: f64, cy: f64, z: f64, scale: f64) {
        let key = (scale * 10.0) as i32;
        if self.shadow_cache.0 != self.vox_gen {
            self.shadow_cache = (self.vox_gen, Vec::new());
        }
        let surf = match self.shadow_cache.1.iter().find(|(k, _)| *k == key) {
            Some((_, sf)) => sf.clone(),
            None => {
                let v = &self.vox;
                let w = (v.tw as f64 * 0.7 * key as f64 / 10.0) as i32 + 2;
                let h = (v.th as f64 * 0.6 * key as f64 / 10.0) as i32 + 2;
                let mut sf = pg::surface_alpha(w, h);
                draw::ellipse(&mut sf, rgba(0, 0, 0, 70), Rect::new(0, 0, w, h), 0);
                self.shadow_cache.1.push((key, sf.clone()));
                sf
            }
        };
        let (px, py_) = self.scr(cx + 0.06, cy - 0.04, z);
        s.bl(&surf, (px as i32 - surf.w() / 2, py_ as i32 - surf.h() / 2));
    }

    fn draw_ripples(&mut self, s: &mut Surface, r: i64) {
        let (tw, th) = (self.vox.tw as f64, self.vox.th as f64);
        for rp in self.ripples.clone() {
            let [x, y, age, strength] = rp;
            if y.floor() as i64 != r {
                continue;
            }
            let f = age / 0.8;
            let rad = (0.2 + 0.75 * f) * strength;
            let (px, py_) = self.scr(x, y, Kind::River.level() + 0.02);
            let (w, h) = ((rad * 2.0 * tw) as i32, (rad * 2.0 * th * 0.9) as i32);
            if w < 4 || h < 3 {
                continue;
            }
            let col = ui::mix((255, 255, 255), (130, 206, 250), f);
            draw::ellipse(s, col, Rect::new((px - w as f64 / 2.0) as i32, (py_ - h as f64 / 2.0) as i32, w, h), 1.max((tw * 0.07 * (1.0 - f)) as i32));
        }
    }

    fn draw_particles(&mut self, s: &mut Surface) {
        let tw = self.vox.tw as f64;
        for p in self.particles.clone() {
            let (px, py_) = self.scr(p.x, p.y, p.z);
            let sz = 2.max((p.size * tw * (1.0 - 0.5 * p.age / p.life)) as i32);
            let (ix, iy) = (px as i32, py_ as i32);
            draw::rect(s, p.col, Rect::new(ix - sz / 2, iy - sz / 2, sz, sz), 0);
            draw::rect(s, cd::shade(p.col, 0.75), Rect::new(ix - sz / 2, iy + sz / 2 - 1.max(sz / 3), sz, 1.max(sz / 3)), 0);
        }
    }

    fn draw_danger(&mut self, s: &mut Surface) {
        let k = self.danger();
        if k <= 0.0 {
            return;
        }
        let (w, h) = (self.b.width, self.b.height);
        if self.vignette.as_ref().map(|v| (v.w(), v.h())) != Some((w, h / 3)) {
            let hh = h / 3;
            let mut vg = pg::surface_alpha(w, hh);
            for i in 0..hh {
                let a = (150.0 * py::pow(i as f64 / hh as f64, 2.0)) as i32;
                draw::line(&mut vg, rgba(220, 30, 40, a), (0, i), (w, i), 1);
            }
            self.vignette = Some(vg);
        }
        let Some(vg) = self.vignette.as_mut() else {
            return;
        };
        vg.set_alpha(Some((255.0 * k * (0.6 + 0.4 * ui::pulse(6.0, 0.35, 1.0))) as i32 as u8));
        let vgc = vg.clone();
        s.bl(&vgc, (0, h - vgc.h()));
    }

    fn draw_eagle(&mut self, s: &mut Surface) {
        let Some(d) = self.death.clone() else {
            return;
        };
        if d.cause != "eagle" {
            return;
        }
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        let zh = self.vox.zh as f64;
        let p = d.t / d.dur;
        let (px, py_) = self.scr(d.x + 0.5, d.y + 0.5, d.z);
        let grab = 0.42;
        let (ex, ey, carry, q);
        if p < grab {
            let mut qq = p / grab;
            qq = qq * qq * (3.0 - 2.0 * qq);
            ex = px + (1.0 - qq) * w * 0.25;
            ey = -h * 0.35 + (py_ - zh * 0.6 + h * 0.35) * qq;
            carry = false;
            q = qq;
        } else {
            let qq = (p - grab) / (1.0 - grab);
            ex = px + qq * w * 0.18;
            ey = py_ - zh * 0.6 - qq * qq * (py_ + h * 0.5);
            carry = true;
            q = qq;
        }
        let cid = self.data.selected.clone();
        if !carry {
            self.player_sprite(s, &cid, self.facing, d.x, d.y, d.z, 1.0, 1.0, true, None);
            self.shadow(s, d.x + 0.5 + (1.0 - q) * 1.5, d.y + 0.5, d.z, 0.6 + q * 1.6);
        } else {
            let (surf, ax, ay) = self.vox.sprite(&format!("{cid}:{}", self.facing), false);
            let v = &self.vox;
            let fx = ax as f64 + 0.5 * v.tw as f64 + 0.5 * v.sk as f64;
            let fy = ay as f64 - 0.5 * v.th as f64;
            s.bl(&surf, ((ex - fx) as i32, (ey + 1.25 * zh - fy) as i32));
        }
        let flap = (self.anim * 16.0).sin();
        let sy = 1.0 + 0.14 * flap;
        let (surf, _ax, _ay) = self.vox.scaled("eagle_down", 1.0, sy);
        let (_b, bax, bay) = self.vox.sprite("eagle_down", false);
        let (mx, my) = self.vox.proj(0.0, 0.0, 0.3);
        s.bl(&surf, ((ex - (bax as f64 + mx)) as i32, (ey - (bay as f64 + my) * sy) as i32));
    }

    fn draw_night(&mut self, s: &mut Surface) {
        let n = cw::night_factor(self.cam + 4.0);
        if n <= 0.01 {
            return;
        }
        let (w, h) = (self.b.width, self.b.height);
        if self.night.as_ref().map(|v| (v.w(), v.h())) != Some((w, h)) {
            self.night = Some(pg::surface(w, h));
        }
        let mut ov = self.night.clone().expect("night");
        let tint = ui::mix((255, 255, 255), NIGHT_TINT, n);
        ov.fill_c(tint);
        let soft = ui::mix(tint, (255, 238, 200), 0.55);
        let bright = ui::mix(tint, (255, 246, 222), 0.92);
        let clock = self.clock;
        let th = self.vox.th as f64;
        let tw = self.vox.tw as f64;
        let r_hi = (self.oy as f64 / th).ceil() as i64 + 1;
        let r_lo = ((self.oy as f64 - h as f64) / th).floor() as i64 - 1;
        let mut r = r_hi;
        while r >= r_lo {
            let row = self.wrow(r);
            let lvl = row.kind.level();
            let rf = r as f64;
            if let Some(coin) = row.coin {
                if !self.taken.contains(&r) {
                    let (px, py_) = self.scr(coin.0 as f64 + 0.5, rf + 0.5, lvl + 0.4);
                    let rad = (tw * if coin.1 > 1 { 0.5 } else { 0.36 }) as i32;
                    draw::circle(&mut ov, ui::mix(tint, (255, 236, 160), 0.7), (px as i32, py_ as i32), rad, 0);
                }
            }
            if row.kind == Kind::Road {
                let d = row.dir as f64;
                for o in &row.objs {
                    let x = cw::obj_x(&row, o, clock);
                    if !(-9.0 < x && x < cw::COLS as f64 + 7.0) {
                        continue;
                    }
                    let fx = if d > 0.0 { x + o.1 } else { x };
                    let p1: Vec<(f64, f64)> = [(fx, rf + 0.2), (fx + 3.6 * d, rf - 0.25), (fx + 3.6 * d, rf + 1.25), (fx, rf + 0.8)].iter().map(|&(a, b)| self.scr(a, b, lvl)).collect();
                    draw::polygon(&mut ov, soft, &p1, 0);
                    let p2: Vec<(f64, f64)> = [(fx, rf + 0.28), (fx + 2.4 * d, rf + 0.05), (fx + 2.4 * d, rf + 0.95), (fx, rf + 0.72)].iter().map(|&(a, b)| self.scr(a, b, lvl)).collect();
                    draw::polygon(&mut ov, bright, &p2, 0);
                }
            } else if row.kind == Kind::Rail {
                let (warn, head, _len) = cw::train_state(&row, clock);
                if let Some(head) = head {
                    let d = row.dir as f64;
                    let p1: Vec<(f64, f64)> = [(head, rf + 0.2), (head + 6.0 * d, rf - 0.4), (head + 6.0 * d, rf + 1.4), (head, rf + 0.8)].iter().map(|&(a, b)| self.scr(a, b, lvl)).collect();
                    draw::polygon(&mut ov, soft, &p1, 0);
                }
                if warn && ((clock * 6.0) as i64) % 2 == 0 {
                    let (px, py_) = self.scr(-1.0, rf + 0.1, lvl + 1.1);
                    let rad = (tw * 0.5) as i32;
                    draw::circle(&mut ov, ui::mix(tint, (255, 150, 140), 0.8), (px as i32, py_ as i32), rad, 0);
                }
            }
            r -= 1;
        }
        if self.death.as_ref().is_none_or(|d| d.cause != "eagle") {
            let (x, y, z) = match &self.death {
                None => self.pos(),
                Some(d) => (d.x, d.y, d.z),
            };
            let (px, py_) = self.scr(x + 0.5, y + 0.5, z + 0.3);
            let (ew, eh) = ((tw * 1.9) as i32, (th * 1.9) as i32);
            draw::ellipse(&mut ov, soft, Rect::new(px as i32 - ew / 2, py_ as i32 - eh / 2, ew, eh), 0);
        }
        s.bl_flags(&ov, (0, 0), BLEND_RGB_MULT);
    }

    // ===================================================== HUD

    fn shadow_text(s: &mut Surface, fnt: &Font, txt: &str, pos: (i32, i32), col: impl Col) {
        s.bl(&fnt.render(txt, true, rgb(20, 24, 36)), (pos.0 + 1, pos.1 + 2));
        s.bl(&fnt.render(txt, true, col.c()), pos);
    }

    fn coin_icon(s: &mut Surface, cx: f64, cy: f64, r: i32) {
        let (ix, iy) = (cx as i32, cy as i32);
        draw::circle(s, (176, 120, 20), (ix + 1, iy + 2), r, 0);
        draw::circle(s, (246, 190, 44), (ix, iy), r, 0);
        draw::circle(s, (255, 226, 110), (ix, iy), 2.max(r - 3), 0);
        draw::rect(s, (236, 170, 36), Rect::new(ix - 1.max(r / 4), iy - r / 2, 2.max(r / 2), r), 0);
    }

    fn draw_hud(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let pad = 8.max(w / 60);
        let txt = self.best.to_string();
        let sh = self.num.render(&txt, true, rgb(20, 24, 36));
        let img = self.num.render(&txt, true, rgb(255, 255, 255));
        s.bl(&sh, (pad + 2, pad + 3));
        s.bl(&img, (pad, pad));
        let y = pad + img.h() - 2;
        let best = if self.b.mode == "daily" { self.day_best } else { self.hs }.max(self.best);
        let lab = if self.is_archive() {
            tf("cr.hud_top_arch", &[("n", &best.to_string())])
        } else if self.b.mode == "daily" {
            tf("cr.hud_top_daily", &[("n", &best.to_string())])
        } else {
            tf("cr.hud_top", &[("n", &best.to_string())])
        };
        Self::shadow_text(s, &self.small, &lab, (pad, y), (255, 236, 150));
        let coins = self.data.coins.to_string();
        let img = self.mid.render(&coins, true, rgb(255, 255, 255));
        let cy = pad + img.h() / 2 + 2;
        let right = w - pad;
        Self::shadow_text(s, &self.mid, &coins, (right - img.w(), cy - img.h() / 2), (255, 255, 255));
        let r = 7.max(img.h() / 2 - 1);
        Self::coin_icon(s, (right - img.w() - r - 8) as f64, cy as f64, r);
        if self.run_coins != 0 {
            let plus = format!("+{}", self.run_coins);
            let pimg = self.small.render(&plus, true, rgb(255, 226, 110));
            Self::shadow_text(s, &self.small, &plus, (right - pimg.w(), cy + r + 4), (255, 226, 110));
        }
        if self.b.mode == "daily" {
            let key = if self.is_archive() { "cr.archive_tag" } else { "cr.daily_tag" };
            let tag = tf(key, &[("date", &seedrand::date_of(self.day))]);
            let timg = self.tiny.render(&tag, true, rgb(255, 255, 255));
            let rc = timg.get_rect().midtop_at((w / 2, pad));
            let bx = rc.inflate(18, 8);
            let mut ov = pg::surface_alpha(bx.w, bx.h);
            let orc = ov.get_rect();
            draw::rect_r(&mut ov, rgba(20, 24, 40, 150), orc, 0, bx.h / 2);
            s.bl(&ov, (bx.x, bx.y));
            s.blr(&timg, rc);
        }
        if !self.started && self.death.is_none() && !self.b.game_over {
            let hint = t("cr.hint");
            let img = self.small.render(&hint, true, rgb(255, 255, 255));
            let rc = img.get_rect().midbottom_at((w / 2, h - pad - 6));
            let bx = rc.inflate(24, 10);
            let mut ov = pg::surface_alpha(bx.w, bx.h);
            let a = (120.0 + 60.0 * ui::pulse(2.5, 0.35, 1.0)) as i32;
            let orc = ov.get_rect();
            draw::rect_r(&mut ov, rgba(20, 24, 40, a), orc, 0, bx.h / 2);
            s.bl(&ov, (bx.x, bx.y));
            s.blr(&img, rc);
        }
    }

    /// `_fit(fnt, txt, col, maxw)`
    fn fit(&self, fnt: &Font, txt: &str, col: impl Col, maxw: i32) -> Surface {
        let img = fnt.render(txt, true, col.c());
        if img.w() <= maxw {
            return img;
        }
        for smaller in [&self.small, &self.tiny] {
            if smaller.get_height() < fnt.get_height() {
                let img = smaller.render(txt, true, col.c());
                if img.w() <= maxw {
                    return img;
                }
            }
        }
        let mut kurz: Vec<char> = txt.chars().collect();
        while kurz.len() > 2 && self.tiny.size(&(kurz.iter().collect::<String>() + "...")).0 > maxw {
            kurz.pop();
        }
        self.tiny.render(&(kurz.iter().collect::<String>() + "..."), true, col.c())
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 8);
        let img = self.fit(&self.small, text, if on { p.text } else { p.text_dim }, rc.w - 10);
        s.blr(&img, img.get_rect().center_at(rc.center()));
    }

    fn draw_over(&mut self, s: &mut Surface) {
        let p = ui::pal();
        if self.over_rect.is_none() {
            self.layout_over();
        }
        let pr = self.over_rect.unwrap_or(Rect::new(0, 0, 0, 0));
        let key = (pr.w, pr.h, p.panel);
        if self.over_bg.as_ref().map(|o| o.0) != Some(key) {
            let mut ov = pg::surface_alpha(pr.w, pr.h);
            let orc = ov.get_rect();
            draw::rect_r(&mut ov, rgba(p.panel.r, p.panel.g, p.panel.b, 236), orc, 0, 14);
            self.over_bg = Some((key, ov));
        }
        if let Some((_, ov)) = &self.over_bg {
            s.bl(ov, (pr.x, pr.y));
        }
        draw::rect_r(s, self.b.accent, pr, 2, 14);
        let cx = pr.centerx();
        let maxw = pr.w - 28;
        let mut y = pr.y + 14;
        let cause = self.death.as_ref().map(|d| d.cause).unwrap_or("car");
        let rows = [
            (self.huge.clone(), t("common.game_over"), p.red),
            (self.tiny.clone(), t(&format!("cr.cause.{cause}")), p.text_dim),
            (self.b.font.clone(), tf("common.points", &[("score", &self.best.to_string())]), p.text),
        ];
        for (fnt, txt, col) in rows {
            let img = self.fit(&fnt, &txt, col, maxw);
            s.blr(&img, img.get_rect().midtop_at((cx, y)));
            y += img.h() + 8;
        }
        let mut line = if self.b.mode == "daily" {
            let best = self.best.max(self.daily_best());
            tf("cr.best_daily", &[("n", &best.to_string())])
        } else {
            tf("cr.best", &[("n", &self.best.max(self.hs).to_string())])
        };
        if self.record && self.best > 0 {
            line += &("  \u{00B7}  ".to_string() + &t("cr.new_record"));
        }
        let img = self.fit(&self.small, &line, p.gold, maxw);
        s.blr(&img, img.get_rect().midtop_at((cx, y)));
        y += img.h() + 8;
        let line = tf("cr.coins_run", &[("n", &self.run_coins.to_string()), ("total", &self.data.coins.to_string())]);
        let img = self.fit(&self.small, &line, p.text, maxw);
        let mut rc = img.get_rect().midtop_at((cx, y));
        let rr = 6.max(img.h() / 2 - 2);
        rc.x += rr + 3;
        Self::coin_icon(s, (rc.x - rr - 6) as f64, rc.centery() as f64, rr);
        s.blr(&img, rc);
        for (key, rc) in self.over_btns.clone() {
            let label = match key {
                "again" => t("cr.btn_again"),
                "chars" => capitalize(&t("cr.tab_chars")),
                _ => t("cr.btn_setup"),
            };
            self.btn(s, rc, &label, key == "again");
        }
        let hint = self.fit(&self.tiny, &t("cr.over_hint"), p.text_dim, maxw);
        s.blr(&hint, hint.get_rect().midbottom_at((cx, pr.bottom() - 10)));
    }

    // ===================================================== Setup zeichnen

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let cx = w / 2;
        let title = self.huge.render("CROSSY ROAD", true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.10) as i32)));
        let sub = self.fit(&self.small, &t(if self.tab == "play" { "cr.subtitle" } else { "cr.chars_sub" }), p.text_dim, w - 30);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.165) as i32)));
        for (i, rc) in self.tab_rects.clone().into_iter().enumerate() {
            self.btn(s, rc, &t(&format!("cr.tab_{}", TABS[i])), self.tab == TABS[i]);
        }
        if self.tab == "chars" {
            self.draw_chars(s);
        } else {
            self.draw_play_tab(s);
        }
        if let Some(pk) = self.picker.as_mut() {
            pk.draw(s);
        }
    }

    fn stage(&mut self) {
        let tw = 24.max((self.card_rect.h as f64 * 0.42) as i32);
        if self.stage_vox.as_ref().is_none_or(|v| v.tw != tw) {
            self.stage_vox = Some(Voxel::new(tw, self.shadows));
            self.stage_gen += 1;
        }
    }

    fn draw_play_tab(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let card = self.card_rect;
        ui::draw_panel_ex(s, card, None, None, Some(12), true, None);
        self.stage();
        let gen = self.stage_gen;
        let v = self.stage_vox.as_mut().expect("stage");
        let block: [cw::B; 2] = [(0.0, 0.0, -0.55, 1.0, 1.0, 0.0, (112, 160, 64)), (0.0, 0.0, 0.0, 1.0, 1.0, 0.16, cd::GRASS_A)];
        if self.block_cache.as_ref().is_none_or(|b| b.0 != gen) {
            self.block_cache = Some((gen, v.render(&block, false)));
        }
        let (bsurf, bax, bay) = self.block_cache.clone().expect("block").1;
        let (vtw, vsk, vth, vzh) = (v.tw as f64, v.sk as f64, v.th as f64, v.zh as f64);
        let stage_w = (1.0 * vtw + vsk) as i32;
        let sx0 = card.x + 14 + py::fdiv32(card.h - 16 - stage_w, 2);
        let base_y = card.bottom() - 10 - (0.55 * vzh) as i32;
        s.bl(&bsurf, (sx0 - bax, base_y - bay));
        let faces = ["down", "right", "down", "left"];
        let ph = py::fpmod(self.anim, 4.8) / 1.2;
        let facing = faces[(ph as usize).min(3)];
        let frac = ph - (ph as i64) as f64;
        let mut z = 0.16;
        let mut sy = 1.0;
        if frac > 0.72 {
            let k = (PI * (frac - 0.72) / 0.28).sin();
            z += 0.35 * k;
            sy = 1.0 + 0.12 * k;
        }
        let cid = self.data.selected.clone();
        if cw::hovers(&cid) {
            z += 0.12 + 0.05 * (self.anim * 3.0).sin();
        }
        let key = format!("{cid}:{facing}");
        let (surf, _ax, _ay) = v.scaled(&key, 1.0, sy);
        let (_b, bax2, bay2) = v.sprite(&key, false);
        let fx = bax2 as f64 + 0.5 * vtw + 0.5 * vsk;
        let fy = bay2 as f64 - 0.5 * vth;
        let ppx = sx0 as f64 + 0.5 * vtw + 0.5 * vsk;
        let ppy = base_y as f64 - 0.5 * vth - z * vzh;
        s.bl(&surf, ((ppx - fx) as i32, (ppy - fy * sy) as i32));
        let tx = card.x + card.h + 6;
        let tw = card.right() - 14 - tx;
        let mut lines: Vec<(Font, String, Color)> = Vec::new();
        let mode_key = if self.b.mode == "daily" { "cr.mode.daily" } else { "cr.mode.endless" };
        lines.push((self.mid.clone(), t(mode_key), self.b.accent));
        let desc = if self.is_archive() {
            tf("cr.mode_desc.archive", &[("date", &seedrand::date_of(self.day))])
        } else if self.b.mode == "daily" {
            tf("cr.mode_desc.daily", &[("date", &seedrand::date_of(self.day))])
        } else {
            t("cr.mode_desc.endless")
        };
        lines.push((self.tiny.clone(), desc, p.text_dim));
        if self.b.mode == "daily" {
            lines.push((self.small.clone(), tf("cr.best_daily", &[("n", &self.daily_best().to_string())]), p.gold));
        } else {
            let sb = self.setup_best();
            lines.push((self.small.clone(), tf("cr.best", &[("n", &sb.to_string())]), p.gold));
        }
        lines.push((self.small.clone(), tf("cr.char_line", &[("name", &t(&format!("cr.char.{cid}")))]), p.text));
        let imgs: Vec<Surface> = lines.iter().map(|(f, txt, col)| self.fit(f, txt, *col, tw)).collect();
        let total = imgs.iter().map(|i| i.h()).sum::<i32>() + 6 * (imgs.len() as i32 - 1) + self.mid.get_height() + 6;
        let mut y = card.y + 8.max(py::fdiv32(card.h - total, 2));
        for img in &imgs {
            s.bl(img, (tx, y));
            y += img.h() + 6;
        }
        let coins = self.data.coins.to_string();
        let cimg = self.mid.render(&coins, true, p.text);
        let r = 6.max(cimg.h() / 2 - 2);
        Self::coin_icon(s, (tx + r) as f64, (y + cimg.h() / 2) as f64, r);
        s.bl(&cimg, (tx + 2 * r + 8, y));
        for (rects, key, on) in [(self.shadow_rects.clone(), "cr.lbl_shadows", self.shadows), (self.night_rects.clone(), "cr.lbl_daynight", self.daynight)] {
            let img = self.fit(&self.tiny, &t(key), p.text_dim, self.opt_w);
            let mid = (rects[0].x + rects[rects.len() - 1].right()) / 2;
            s.blr(&img, img.get_rect().midbottom_at((mid, rects[0].y - 4)));
            for (i, rc) in rects.iter().enumerate() {
                self.btn(s, *rc, &if i == 0 { t("common.on") } else { t("common.off") }, on == (i == 0));
            }
        }
        if let Some(rects) = self.day_rects {
            let days = self.days();
            let (first, last) = (days[0], days[days.len() - 1]);
            let tiny = self.tiny.clone();
            da::draw_day_row(s, &rects, self.day, self.b.accent, self.day > first, self.day < last, &tiny, self.is_archive());
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint_key = if self.b.mode == "daily" { "cr.setup_hint_daily" } else { "cr.setup_hint" };
        let hint = self.fit(&self.tiny, &t(hint_key), p.text_faint, w - 20);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 30)));
        let ctrl = self.fit(&self.tiny, &t("cr.hint"), ui::mix(self.b.accent, p.text, 0.45), w - 20);
        s.blr(&ctrl, ctrl.get_rect().center_at((cx, h - 12)));
    }

    fn draw_chars(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let r0 = self.grid_rects[0];
        let tw = 16.max(((r0.w as f64 * 0.62).min((r0.h - self.tiny.get_height()) as f64 * 0.5)) as i32);
        if self.card_vox.as_ref().is_none_or(|v| v.tw != tw) {
            self.card_vox = Some(Voxel::new(tw, false));
        }
        let coins = self.data.coins;
        let ids = cw::char_ids();
        for (i, rc) in self.grid_rects.clone().into_iter().enumerate() {
            let cid = ids[i];
            let owned = self.data.unlocked.iter().any(|u| u == cid);
            let chosen = cid == self.data.selected;
            let cur = i == self.cursor;
            draw::rect_r(s, if cur { p.btn_sel } else { p.btn }, rc, 0, 10);
            let border = if cur || chosen { self.b.accent } else { p.border };
            draw::rect_r(s, border, rc, if cur || chosen { 2 } else { 1 }, 10);
            let name = self.fit(&self.tiny, &t(&format!("cr.char.{cid}")), if owned { p.text } else { p.text_dim }, rc.w - 6);
            let ny = rc.bottom() - name.h() - 4;
            let mut bob = 0.0;
            if cur {
                bob = 0f64.max((self.anim * 6.0).sin()) * 0.18;
            }
            let v = self.card_vox.as_mut().expect("card");
            let (mut surf, ax, ay) = v.sprite(&format!("{cid}:down"), false);
            if !owned {
                surf = surf.copy();
                surf.fill_f(rgba(70, 74, 90, 255), BLEND_RGBA_MULT);
                surf.fill_f(rgba(40, 44, 60, 0), BLEND_RGBA_ADD);
            }
            let (vtw, vsk, vth, vzh) = (v.tw as f64, v.sk as f64, v.th as f64, v.zh as f64);
            let fx = ax as f64 + 0.5 * vtw + 0.5 * vsk;
            let fy = ay as f64 - 0.5 * vth;
            let px = rc.centerx() as f64 - 0.25 * vsk;
            let room_top = rc.y + 4;
            let feet = room_top + py::fdiv32(ny - room_top, 2) + (0.55 * vzh) as i32;
            let py_ = (ny - (0.3 * vth) as i32).min(feet) - (bob * vzh) as i32;
            s.bl(&surf, ((px - fx) as i32, (py_ as f64 - fy) as i32));
            s.blr(&name, name.get_rect().midbottom_at((rc.centerx(), rc.bottom() - 4)));
            if chosen {
                draw::circle(s, self.b.accent, (rc.right() - 11, rc.y + 11), 6, 0);
                draw::lines(s, p.bg_top, false, &[(rc.right() - 14, rc.y + 11), (rc.right() - 12, rc.y + 14), (rc.right() - 8, rc.y + 8)], 2);
            } else if !owned {
                let price = cw::price(cid);
                let col = if coins >= price { p.gold } else { p.text_faint };
                let pim = self.tiny.render(&price.to_string(), true, col);
                let rr = 4.max(pim.h() / 2 - 2);
                let pr = pim.get_rect().topright_at((rc.right() - 6, rc.y + 5));
                s.blr(&pim, pr);
                Self::coin_icon(s, (pr.x - rr - 3) as f64, pr.centery() as f64, rr);
            }
        }
        let d = self.detail_rect;
        ui::draw_panel_ex(s, d, None, None, Some(10), false, None);
        let cid = ids[self.cursor];
        let owned = self.data.unlocked.iter().any(|u| u == cid);
        let cimg = self.mid.render(&coins.to_string(), true, p.text);
        let r = 6.max(cimg.h() / 2 - 2);
        Self::coin_icon(s, (d.x + 12 + r) as f64, d.centery() as f64, r);
        s.blr(&cimg, cimg.get_rect().midleft_at((d.x + 2 * r + 18, d.centery())));
        let name_x = d.x + 2 * r + 30 + cimg.w();
        let name = self.fit(&self.b.font, &t(&format!("cr.char.{cid}")), p.text, self.buy_rect.x - name_x - 8);
        s.blr(&name, name.get_rect().midleft_at((name_x, d.centery())));
        let (label, on) = if cid == self.data.selected {
            (t("cr.chosen"), false)
        } else if owned {
            (t("cr.choose"), true)
        } else {
            (tf("cr.buy", &[("n", &cw::price(cid).to_string())]), coins >= cw::price(cid))
        };
        let br = self.buy_rect;
        self.btn(s, br, &label, on);
        let hint = self.fit(&self.tiny, &t("cr.chars_hint"), p.text_faint, w - 20);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 13)));
    }
}

use pgz_engine::Kind as Kind_;

impl Game for CrossyRoadGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, St::Setup)
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
        if self.b.mode != "endless" && self.b.mode != "daily" {
            self.b.mode = "endless".into();
        }
        self.shadows = self.b.opt_bool("crossy", "shadows", true);
        self.daynight = self.b.opt_bool("crossy", "daynight", true);
        self.data = Self::load();
        self.day = da::today_index(); // gespielter Tag (Tagesstrecke)
        self.picker = None; // offenes Archiv-Fenster
        self.anim = 0.0;
        self.state = St::Setup;
        self.tab = "play";
        self.cursor = self.char_index(&self.data.selected.clone());
        self.world = None;
        self.death = None;
        self.particles.clear();
        self.ripples.clear();
        self.night = None;
        self.vignette = None;
        self.last_draw = None;
        self.make_fonts();
        self.make_voxels();
        self.layout_setup();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.make_voxels();
        self.layout_setup();
        if let Some(pk) = self.picker.as_mut() {
            pk.rect = None;
        }
        self.night = None;
        self.vignette = None;
    }

    fn on_exit(&mut self) {
        if self.state == St::Play && self.world.is_some() && !self.b.game_over && self.run_coins != 0 {
            self.save();
        }
    }

    fn show_highscore_banner(&self) -> bool {
        self.b.mode != "daily"
    }

    fn wants_escape(&self) -> bool {
        self.state == St::Setup && (self.tab == "chars" || self.picker.is_some())
    }

    // ----- Match-Historie ----------------------------------------------
    fn match_score(&self) -> i64 {
        self.best
    }

    fn match_info(&self) -> Option<Map<String, Value>> {
        let mut info = info_map(vec![("c", self.run_coins.into()), ("ch", self.data.selected.clone().into())]);
        if self.b.mode == "daily" {
            let mut m = info_map(vec![("day", self.day.into())]);
            m.append(&mut info);
            info = m;
        }
        Some(info)
    }

    fn match_flags(&self) -> i64 {
        if self.b.mode != "daily" {
            return 0;
        }
        matches::F_DAILY | if self.is_archive() { matches::F_ARCHIVE } else { 0 }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == St::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.b.game_over {
            self.handle_over(ev);
            return;
        }
        if self.death.is_some() {
            return;
        }
        if ev.kind == Kind_::KeyDown {
            if ev.repeat {
                return;
            }
            let k = ev.key.as_str();
            let is = |a: &str| self.b.is_action(k, a, None);
            if is("up") || is("action") || k == "Up" {
                self.press("up");
            } else if is("down") || k == "Down" {
                self.press("down");
            } else if is("left") || k == "Left" {
                self.press("left");
            } else if is("right") || k == "Right" {
                self.press("right");
            }
        } else if ev.kind == Kind_::MouseDown && ev.button == 1 {
            self.press("up");
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim += dt;
        if self.state != St::Play || self.world.is_none() {
            return;
        }
        let dt = dt.min(0.05);
        self.clock += dt;
        self.step_effects(dt);
        if let Some(d) = self.death.as_mut() {
            d.t += dt;
            if d.t >= d.dur {
                self.finish_run();
            }
            return;
        }
        self.land_t = 0f64.max(self.land_t - dt);
        self.bump_t = 0f64.max(self.bump_t - dt);
        if let Some(h) = self.hop.as_mut() {
            h.t += dt;
            if h.t >= cw::HOP_T {
                self.land();
            }
        } else if let Some((idx, seg)) = self.log {
            let r = self.row;
            let row = self.wrow(r);
            self.col = cw::obj_x(&row, &row.objs[idx], self.clock) + seg as f64;
            let cx = self.col + 0.5;
            if cx < 0.0 || cx > cw::COLS as f64 {
                self.die("edge");
                return;
            }
        }
        if self.death.is_none() {
            self.collide();
        }
        if self.death.is_none() {
            self.camera(dt);
        }
        self.sounds();
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            return;
        }
        let now = py::perf_counter();
        if self.b.game_over {
            if let Some(last) = self.last_draw {
                let step = 0f64.max(0.1f64.min(now - last));
                self.clock += step;
                self.anim += step;
            }
        }
        self.last_draw = Some(now);
        self.draw_world(&mut s);
        self.draw_eagle(&mut s);
        if self.daynight {
            self.draw_night(&mut s);
        }
        if self.b.game_over {
            self.draw_over(&mut s);
        } else {
            self.draw_hud(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "crossy",
    class_name: "CrossyRoadGame",
    supports_multiplayer: false,
    modes: &[("endless", "cr.mode.endless"), ("daily", "cr.mode.daily")],
    make: CrossyRoadGame::new,
};
