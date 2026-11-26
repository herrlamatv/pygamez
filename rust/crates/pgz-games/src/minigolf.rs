//! minigolf.rs
//! ===========
//! Minigolf - 360 Bahnen in 40 Kursen, allein oder zu zweit lokal, dazu
//! eigene Bahnen (MAPS-Reiter + Editor in [`crate::minigolf_edit`]).
//! Uebertragen aus `games/minigolf.py`; Bahnen in [`crate::minigolf_gen`],
//! Zeichnen in [`crate::minigolf_draw`].
//!
//! Die Physik laeuft in Teilschritten mit Reibung (Gruen/Sand/Eis/Kleber),
//! Rampen, Magneten, Drehscheiben, Schub, Sprungrampen, Rohren, Banden,
//! Gummipuffern, Muehlen und Wanderbloecken.

use std::collections::HashMap;
use std::f64::consts::PI;

use pgz_core::{replay, store};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::minigolf_draw::{self as mdraw, View, COL_AIM, COL_BALL, COL_GREEN, COL_LOCK};
use crate::minigolf_edit::{EditAction, MapEditor, MapList};
use crate::minigolf_gen::{self as gen, hole_with, Hole, BORDER, CH, CW, HOLES_PER_ROUND};
use crate::ugc;

const BR: f64 = gen::BALL_R;
const CUP_R: f64 = 3.0;
const ARM_W: f64 = 1.5;
const FRIC_GREEN: f64 = 1.15;
const FRIC_SAND: f64 = 4.60;
const FRIC_ICE: f64 = 0.25;
const FRIC_STICKY: f64 = 12.0;
const WALL_E: f64 = 0.72;
const BUMP_E: f64 = 1.18;
const STOP_EPS: f64 = 2.2;
const MAX_SPEED: f64 = 205.0;
const CAPTURE_SPEED: f64 = 74.0;
const MAX_SHOT_TIME: f64 = 14.0;
const MAX_STROKES: i64 = 8;
const TUNNEL_KEEP: f64 = 0.95;
const TUNNEL_COOLDOWN: f64 = 0.35;
const BOOST_COOLDOWN: f64 = 0.25;
const SPIN_PUSH: f64 = 0.55;
const JUMP_MIN_SPEED: f64 = 30.0;
const OVER_PAD: i32 = 14;
const COURSES: [&str; 5] = ["classic", "pro", "tour", "random", "ugc"];
const TABS: [&str; 2] = ["play", "maps"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Setup,
    Play,
    HoleDone,
    Over,
    Edit,
}

type It<'a> = (&'a str, &'a [&'a [f64]]);

fn h(par: i64, tee: (f64, f64), cup: (f64, f64), items: &[It]) -> Hole {
    hole_with(par, tee, cup, items)
}

/// Kurs 1: Classic.
fn holes_classic() -> Vec<Hole> {
    vec![
        h(2, (50.0, 140.0), (50.0, 26.0), &[("walls", &[&[20.0, 60.0, 14.0, 8.0], &[66.0, 60.0, 14.0, 8.0]])]),
        h(3, (24.0, 140.0), (76.0, 30.0), &[("walls", &[&[38.0, 40.0, 10.0, 74.0], &[60.0, 96.0, 26.0, 8.0]]), ("sand", &[&[58.0, 118.0, 26.0, 14.0]])]),
        h(3, (50.0, 142.0), (50.0, 22.0), &[("walls", &[&[38.0, 66.0, 24.0, 20.0]]), ("sand", &[&[14.0, 62.0, 20.0, 28.0], &[66.0, 62.0, 20.0, 28.0]])]),
        h(3, (26.0, 142.0), (76.0, 26.0), &[("walls", &[&[66.0, 74.0, 8.0, 50.0]]), ("water", &[&[12.0, 56.0, 44.0, 44.0]])]),
        h(4, (20.0, 144.0), (80.0, 24.0), &[("walls", &[&[30.0, 108.0, 62.0, 8.0], &[8.0, 66.0, 62.0, 8.0], &[30.0, 30.0, 46.0, 8.0]])]),
        h(3, (50.0, 142.0), (50.0, 22.0), &[("bumpers", &[&[30.0, 96.0, 6.0], &[70.0, 96.0, 6.0], &[50.0, 66.0, 7.0], &[30.0, 44.0, 5.0], &[70.0, 44.0, 5.0]])]),
        h(4, (50.0, 144.0), (50.0, 22.0), &[("walls", &[&[6.0, 84.0, 30.0, 8.0], &[64.0, 84.0, 30.0, 8.0]]), ("mills", &[&[50.0, 88.0, 13.0, 2.0, 1.5]])]),
        h(3, (50.0, 144.0), (50.0, 20.0), &[("sand", &[&[20.0, 112.0, 24.0, 16.0], &[56.0, 112.0, 24.0, 16.0]]), ("slopes", &[&[20.0, 50.0, 60.0, 56.0, 0.0, 34.0]])]),
        h(
            4,
            (26.0, 142.0),
            (74.0, 26.0),
            &[("walls", &[&[44.0, 34.0, 8.0, 46.0], &[44.0, 106.0, 8.0, 40.0]]), ("bumpers", &[&[78.0, 96.0, 6.0]]), ("movers", &[&[20.0, 80.0, 24.0, 8.0, 40.0, 0.0, 26.0]])],
        ),
    ]
}

/// Kurs 2: Pro.
fn holes_pro() -> Vec<Hole> {
    vec![
        h(
            3,
            (50.0, 144.0),
            (50.0, 36.0),
            &[
                ("walls", &[&[26.0, 62.0, 14.0, 6.0], &[60.0, 62.0, 14.0, 6.0]]),
                ("water", &[&[10.0, 14.0, 80.0, 12.0], &[10.0, 26.0, 26.0, 22.0], &[64.0, 26.0, 26.0, 22.0], &[10.0, 48.0, 34.0, 12.0], &[56.0, 48.0, 34.0, 12.0]]),
                ("slopes", &[&[20.0, 72.0, 30.0, 40.0, 11.0, -8.0], &[50.0, 72.0, 30.0, 40.0, -11.0, -8.0]]),
            ],
        ),
        h(4, (50.0, 146.0), (50.0, 18.0), &[("walls", &[&[28.0, 20.0, 6.0, 100.0], &[66.0, 20.0, 6.0, 100.0]]), ("mills", &[&[50.0, 106.0, 15.0, 2.0, -1.8], &[50.0, 56.0, 15.0, 2.0, 2.2]])]),
        h(
            4,
            (18.0, 142.0),
            (82.0, 26.0),
            &[
                ("walls", &[&[34.0, 100.0, 8.0, 46.0], &[58.0, 46.0, 8.0, 46.0]]),
                ("sand", &[&[60.0, 116.0, 26.0, 20.0]]),
                ("bumpers", &[&[50.0, 122.0, 7.0], &[24.0, 74.0, 6.0], &[76.0, 74.0, 6.0], &[50.0, 34.0, 6.0]]),
            ],
        ),
        h(5, (14.0, 146.0), (86.0, 20.0), &[("walls", &[&[24.0, 118.0, 70.0, 7.0], &[6.0, 92.0, 70.0, 7.0], &[24.0, 66.0, 70.0, 7.0], &[6.0, 40.0, 70.0, 7.0]])]),
        h(
            4,
            (50.0, 146.0),
            (50.0, 18.0),
            &[
                ("walls", &[&[6.0, 108.0, 34.0, 7.0], &[60.0, 108.0, 34.0, 7.0], &[6.0, 56.0, 34.0, 7.0], &[60.0, 56.0, 34.0, 7.0]]),
                ("movers", &[&[40.0, 108.0, 12.0, 7.0, 8.0, 0.0, 14.0], &[48.0, 56.0, 12.0, 7.0, -8.0, 0.0, 12.0]]),
            ],
        ),
        h(
            5,
            (20.0, 146.0),
            (80.0, 18.0),
            &[
                ("walls", &[&[46.0, 128.0, 8.0, 22.0]]),
                ("sand", &[&[8.0, 92.0, 84.0, 34.0]]),
                ("slopes", &[&[20.0, 30.0, 60.0, 54.0, 0.0, 26.0]]),
                ("bumpers", &[&[20.0, 60.0, 6.0], &[80.0, 60.0, 6.0]]),
            ],
        ),
        h(
            4,
            (50.0, 146.0),
            (50.0, 20.0),
            &[("sand", &[&[40.0, 116.0, 20.0, 14.0]]), ("water", &[&[8.0, 56.0, 28.0, 52.0], &[64.0, 56.0, 28.0, 52.0]]), ("bumpers", &[&[24.0, 34.0, 6.0], &[76.0, 34.0, 6.0]])],
        ),
        h(4, (18.0, 144.0), (82.0, 24.0), &[("walls", &[&[6.0, 96.0, 26.0, 8.0], &[68.0, 96.0, 26.0, 8.0]]), ("sand", &[&[66.0, 118.0, 24.0, 18.0]]), ("mills", &[&[50.0, 100.0, 14.0, 3.0, 1.1]])]),
        h(
            5,
            (50.0, 148.0),
            (50.0, 18.0),
            &[
                ("walls", &[&[38.0, 92.0, 8.0, 6.0], &[54.0, 92.0, 8.0, 6.0]]),
                ("water", &[&[8.0, 96.0, 30.0, 30.0], &[62.0, 96.0, 30.0, 30.0]]),
                ("slopes", &[&[24.0, 40.0, 52.0, 44.0, 0.0, 30.0]]),
                ("bumpers", &[&[50.0, 30.0, 7.0]]),
                ("movers", &[&[24.0, 62.0, 22.0, 8.0, 30.0, 0.0, 24.0]]),
            ],
        ),
    ]
}

/// `MiniGolfGame`
pub struct MiniGolfGame {
    pub(crate) b: GameBase,
    course: &'static str,
    guide: bool,
    autoaim: bool,
    pickup: bool,
    power_lock: bool,
    lock_t: f64,
    tour: i64,
    tour_par: i64,
    winner: Option<usize>,
    cw: f64,
    ch: f64,
    setup_tab: &'static str,
    maps: Option<MapList>,
    editor: Option<MapEditor>,
    single_map: String,
    test_return: bool,
    tun_cd: f64,
    boost_cd: f64,
    air: f64,
    rec: Option<replay::Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_layouts: Vec<Hole>,
    rep_pos: HashMap<i64, usize>,
    small: Font,
    tiny: Font,
    card: Font,
    huge: Font,
    hud_h: i32,
    scale: f64,
    ox: f64,
    oy: f64,
    card_x: i32,
    card_w: i32,
    over_cache: Option<Surface>,
    best: Vec<(String, i64)>,
    holes: Vec<Hole>,
    players: usize,
    cards: Vec<Vec<i64>>,
    points: Vec<i64>,
    hole_idx: usize,
    player: usize,
    hole: Hole,
    par: i64,
    strokes: i64,
    cup: (f64, f64),
    bx: f64,
    by: f64,
    vx: f64,
    vy: f64,
    safe: (f64, f64),
    phase: &'static str,
    power: f64,
    charging: bool,
    shot_time: f64,
    mill_a: f64,
    move_t: f64,
    trail: Vec<(f64, f64)>,
    msg: Option<String>,
    msg_t: f64,
    result_key: Option<String>,
    result_pts: i64,
    layout_id: usize,
    aim: f64,
    state: St,
    pub(crate) tab_bottom: i32,
    tab_rects: Vec<Rect>,
    course_rects: Vec<Rect>,
    tour_rects: Vec<Rect>,
    opt_w: i32,
    guide_rects: Vec<Rect>,
    autoaim_rects: Vec<Rect>,
    pickup_rects: Vec<Rect>,
    start_rect: Rect,
    over_y: HashMap<&'static str, i32>,
    over_h: i32,
    over_rects: Vec<(&'static str, Rect)>,
}

impl MiniGolfGame {
    /// `MiniGolfGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let first = holes_classic().remove(0);
        let mut g = MiniGolfGame {
            b,
            course: "classic",
            guide: true,
            autoaim: true,
            pickup: true,
            power_lock: false,
            lock_t: 0.0,
            tour: 1,
            tour_par: 0,
            winner: None,
            cw: CW,
            ch: CH,
            setup_tab: "play",
            maps: None,
            editor: None,
            single_map: String::new(),
            test_return: false,
            tun_cd: 0.0,
            boost_cd: 0.0,
            air: 0.0,
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_layouts: Vec::new(),
            rep_pos: HashMap::new(),
            small: f.clone(),
            tiny: f.clone(),
            card: f.clone(),
            huge: f,
            hud_h: 44,
            scale: 1.0,
            ox: 0.0,
            oy: 0.0,
            card_x: 0,
            card_w: 0,
            over_cache: None,
            best: Vec::new(),
            holes: Vec::new(),
            players: 1,
            cards: Vec::new(),
            points: Vec::new(),
            hole_idx: 0,
            player: 0,
            hole: first,
            par: 3,
            strokes: 0,
            cup: (0.0, 0.0),
            bx: 0.0,
            by: 0.0,
            vx: 0.0,
            vy: 0.0,
            safe: (0.0, 0.0),
            phase: "aim",
            power: 0.35,
            charging: false,
            shot_time: 0.0,
            mill_a: 0.0,
            move_t: 0.0,
            trail: Vec::new(),
            msg: None,
            msg_t: 0.0,
            result_key: None,
            result_pts: 0,
            layout_id: 0,
            aim: 0.0,
            state: St::Setup,
            tab_bottom: 0,
            tab_rects: Vec::new(),
            course_rects: Vec::new(),
            tour_rects: Vec::new(),
            opt_w: 0,
            guide_rects: Vec::new(),
            autoaim_rects: Vec::new(),
            pickup_rects: Vec::new(),
            start_rect: z,
            over_y: HashMap::new(),
            over_h: 0,
            over_rects: Vec::new(),
        };
        g.reset();
        Box::new(g)
    }

    fn truthy(v: Option<Value>, def: bool) -> bool {
        match v {
            None => def,
            Some(Value::Bool(b)) => b,
            Some(Value::Null) => false,
            Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
        }
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::font(14.max(h / 32), false, false);
        self.tiny = ui::font(11.max(h / 42), false, false);
        self.card = ui::font(11.max(h / 46), false, true);
        self.huge = ui::font(24.max(h / 13), true, false);
    }

    fn layout(&mut self) {
        let (cw, ch) = (self.cw, self.ch);
        self.hud_h = 44;
        let avail_h = (self.b.height - self.hud_h - 12) as f64;
        let card_w = if self.b.width >= 560 { 132 } else { 108 };
        self.scale = 1.2f64.max(((self.b.width - card_w - 40) as f64 / cw).min(avail_h / ch));
        self.ox = (self.b.width - card_w - 12) as f64 / 2.0 - cw * self.scale / 2.0;
        self.oy = self.hud_h as f64 + (avail_h - ch * self.scale) / 2.0 + 6.0;
        self.card_x = self.b.width - card_w - 6;
        self.card_w = card_w;
        self.build_over_layout();
    }

    // ------------------------------------------------------- Speicherstand

    fn load_best() -> Vec<(String, i64)> {
        let data = store::load_section("minigolf");
        let mut out = Vec::new();
        if let Some(Value::Object(best)) = data.get("best") {
            for (k, v) in best {
                let n = match v {
                    Value::Number(n) => n.as_f64().map(|f| f as i64),
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

    fn best_key(&self) -> String {
        if self.course == "tour" {
            return format!("tour{}", self.tour);
        }
        if self.course == "ugc" && !self.single_map.is_empty() {
            return format!("ugc:{}", self.single_map);
        }
        self.course.to_string()
    }

    fn best_get(&self) -> Option<i64> {
        let k = self.best_key();
        self.best.iter().find(|e| e.0 == k).map(|e| e.1)
    }

    fn save_best(&mut self, strokes: i64) {
        let key = self.best_key();
        let old = self.best.iter().find(|e| e.0 == key).map(|e| e.1);
        if old.is_none_or(|o| strokes < o) {
            match self.best.iter_mut().find(|e| e.0 == key) {
                Some(e) => e.1 = strokes,
                None => self.best.push((key, strokes)),
            }
            let mut best = Map::new();
            for (k, v) in &self.best {
                best.insert(k.clone(), json!(v));
            }
            let mut m = Map::new();
            m.insert("best".into(), Value::Object(best));
            store::save_section("minigolf", m);
        }
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("minigolf", key, value);
    }

    // ------------------------------------------------------- Runde / Bahn

    fn round_holes(&mut self) -> Vec<Hole> {
        match self.course {
            "classic" => holes_classic(),
            "pro" => holes_pro(),
            "tour" => gen::course_holes(self.tour),
            "ugc" => self.ugc_holes(),
            _ => {
                let mut pool = holes_classic();
                pool.extend(holes_pro());
                for _ in 0..HOLES_PER_ROUND {
                    let c = py::randint(1, gen::TOUR_COURSES);
                    let i = py::randrange(0, HOLES_PER_ROUND as i64) as usize;
                    pool.push(gen::generate(c, i, None));
                }
                let mut picked = py::sample(&pool, HOLES_PER_ROUND);
                picked.sort_by_key(|h| h.par);
                picked.into_iter().map(|h| Self::mirror(h, py::random() < 0.5)).collect()
            }
        }
    }

    fn ugc_holes(&self) -> Vec<Hole> {
        if !self.single_map.is_empty() {
            if let Some(m) = ugc::get(&self.single_map, "minigolf") {
                return vec![ugc::to_hole(&m)];
            }
        }
        let maps = ugc::load_maps("minigolf");
        if maps.is_empty() {
            return vec![holes_classic().remove(0)];
        }
        maps.iter().take(HOLES_PER_ROUND).map(ugc::to_hole).collect()
    }

    /// `_mirror(hole, flip)` - an der Mittelachse spiegeln.
    fn mirror(hole: Hole, flip: bool) -> Hole {
        if !flip {
            return hole;
        }
        let cw = hole.w;
        let mx = |x: f64, w: f64| cw - x - w;
        let mut out = hole.clone();
        out.tee = (mx(hole.tee.0, 0.0), hole.tee.1);
        out.cup = (mx(hole.cup.0, 0.0), hole.cup.1);
        for key in ["walls", "sand", "water", "ice", "sticky"] {
            *out.get_mut(key) = hole.get(key).iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3]]).collect();
        }
        *out.get_mut("slopes") = hole.get("slopes").iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3], -r[4], r[5]]).collect();
        *out.get_mut("bumpers") = hole.get("bumpers").iter().map(|r| vec![mx(r[0], 0.0), r[1], r[2]]).collect();
        *out.get_mut("movers") = hole.get("movers").iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3], -r[4], r[5], r[6]]).collect();
        *out.get_mut("mills") = hole.get("mills").iter().map(|r| vec![mx(r[0], 0.0), r[1], r[2], r[3], -r[4]]).collect();
        *out.get_mut("tunnels") = hole.get("tunnels").iter().map(|r| vec![mx(r[0], 0.0), r[1], mx(r[2], 0.0), r[3], r[4]]).collect();
        *out.get_mut("boosters") = hole.get("boosters").iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3], -r[4], r[5], r[6]]).collect();
        *out.get_mut("magnets") = hole.get("magnets").iter().map(|r| vec![mx(r[0], 0.0), r[1], r[2], r[3]]).collect();
        *out.get_mut("gates") = hole.get("gates").iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3], -r[4], r[5]]).collect();
        *out.get_mut("spinners") = hole.get("spinners").iter().map(|r| vec![mx(r[0], 0.0), r[1], r[2], -r[3]]).collect();
        *out.get_mut("jumps") = hole.get("jumps").iter().map(|r| vec![mx(r[0], r[2]), r[1], r[2], r[3], -r[4], r[5], r[6]]).collect();
        out
    }

    fn new_round(&mut self) {
        self.b.new_round_result();
        self.holes = self.round_holes();
        self.players = if self.b.multiplayer { 2 } else { 1 };
        self.cards = vec![vec![0; self.holes.len()]; self.players];
        self.points = vec![0; self.players];
        self.hole_idx = 0;
        self.player = 0;
        self.winner = None;
        self.b.score = 0;
        self.b.game_over = false;
        self.rec_new();
        self.start_hole();
    }

    fn start_hole(&mut self) {
        let hl = self.holes[self.hole_idx].clone();
        if (hl.w, hl.h) != (self.cw, self.ch) {
            self.cw = hl.w;
            self.ch = hl.h;
            self.layout();
        }
        self.par = hl.par;
        self.strokes = 0;
        self.cup = hl.cup;
        self.bx = hl.tee.0;
        self.by = hl.tee.1;
        self.vx = 0.0;
        self.vy = 0.0;
        self.safe = (self.bx, self.by);
        self.phase = "aim";
        if !self.power_lock {
            self.power = 0.35;
        }
        self.charging = false;
        self.shot_time = 0.0;
        self.mill_a = 0.0;
        self.move_t = 0.0;
        self.tun_cd = 0.0;
        self.boost_cd = 0.0;
        self.air = 0.0;
        self.trail.clear();
        self.msg = None;
        self.msg_t = 0.0;
        self.result_key = None;
        self.result_pts = 0;
        self.hole = hl;
        let data = Value::Object(self.hole.to_json());
        self.layout_id = self.rec.as_mut().map(|r| r.layout(data)).unwrap_or(0);
        self.reset_aim(true);
    }

    // ===================================================== Setup-Screen

    fn build_setup_layout(&mut self) {
        let (w, hgt) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = (370.max((w as f64 * 0.58) as i32)).min(w - 50);
        let gap = 8.0;
        let tab_h = 22.max(30.min(hgt / 15));
        let tab_w = 150.min(py::fdiv32(w - 40, 2));
        let tab_y = (hgt as f64 * 0.215) as i32;
        self.tab_rects = vec![Rect::new(cx - tab_w - 4, tab_y, tab_w, tab_h), Rect::new(cx + 4, tab_y, tab_w, tab_h)];
        self.tab_bottom = tab_y + tab_h;
        let top = self.tab_bottom + 22.max(self.tiny.get_height() + 6);
        let bottom = hgt - 42;
        let bh = 26.max(42.min(((bottom - top - 68) as f64 / 4.0) as i32));
        let lab = 20 + 0.max(40.min(py::fdiv32(bottom - top - 8 - 4 * bh - 60, 6)));
        let step = bh + lab;
        let row = |y: i32, n: i32, wd: Option<f64>, x0: Option<f64>| -> Vec<Rect> {
            let wd = wd.unwrap_or(bw as f64);
            let x0 = x0.unwrap_or(cx as f64 - bw as f64 / 2.0);
            let cwid = (wd - gap * (n - 1) as f64) / n as f64;
            (0..n).map(|i| Rect::new((x0 + i as f64 * (cwid + gap)) as i32, y, cwid as i32, bh)).collect()
        };
        let mut y = top;
        self.course_rects = row(y, COURSES.len() as i32, None, None);
        y += step;
        let bwf = bw as f64;
        let cxf = cx as f64;
        self.tour_rects = vec![Rect::new((cxf - bwf / 2.0) as i32, y, 40, bh), Rect::new((cxf - bwf / 2.0 + 46.0) as i32, y, (bwf - 92.0) as i32, bh), Rect::new((cxf + bwf / 2.0 - 40.0) as i32, y, 40, bh)];
        y += step;
        let grp = (bwf - 2.0 * 18.0) / 3.0;
        self.opt_w = grp as i32;
        self.guide_rects = row(y, 2, Some(grp), None);
        self.autoaim_rects = row(y, 2, Some(grp), Some(cxf - grp / 2.0));
        self.pickup_rects = row(y, 2, Some(grp), Some(cxf + bwf / 2.0 - grp));
        y += step + 4;
        let sw = 190.max((bwf * 0.42) as i32);
        self.start_rect = Rect::new(cx - sw / 2, y, sw, bh + 4);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::MouseDown {
            if let Some(i) = self.tab_rects.iter().position(|r| r.hit(ev.pos)) {
                self.set_tab(TABS[i]);
                return;
            }
        } else if ev.kind == Kind::KeyDown && (ev.key == "Tab" || ev.key == "ISO_Left_Tab") {
            self.set_tab(if self.setup_tab == "play" { "maps" } else { "play" });
            return;
        }
        if self.setup_tab == "maps" {
            if ev.kind == Kind::KeyDown && ev.key == "Escape" {
                self.set_tab("play");
                return;
            }
            if let Some(mut ml) = self.maps.take() {
                let act = ml.handle(&mut self.b, ev);
                self.maps = Some(ml);
                self.edit_action(act);
            }
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if ["1", "2", "3", "4", "5"].contains(&k) {
                self.course = COURSES[k.parse::<usize>().unwrap_or(1) - 1];
                let c = self.course;
                self.save_setting("course", json!(c));
                self.b.play_sound("click");
            } else if k == "Left" || self.b.is_action(k, "left", None) {
                self.step_tour(-1);
            } else if k == "Right" || self.b.is_action(k, "right", None) {
                self.step_tour(1);
            } else if k == "g" || k == "G" {
                self.toggle_guide();
            } else if k == "z" || k == "Z" {
                self.toggle_autoaim();
            } else if k == "p" || k == "P" {
                self.toggle_pickup();
            } else if k == "Return" || k == "space" {
                self.start_play(None);
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.course_rects.iter().position(|r| r.hit(ev.pos)) {
                self.course = COURSES[i];
                let c = self.course;
                self.save_setting("course", json!(c));
                self.b.play_sound("click");
                return;
            }
            if self.course == "tour" {
                if self.tour_rects[0].hit(ev.pos) {
                    self.step_tour(-1);
                    return;
                }
                if self.tour_rects[2].hit(ev.pos) {
                    self.step_tour(1);
                    return;
                }
            }
            for which in 0..3 {
                let (rects, val) = match which {
                    0 => (self.guide_rects.clone(), self.guide),
                    1 => (self.autoaim_rects.clone(), self.autoaim),
                    _ => (self.pickup_rects.clone(), self.pickup),
                };
                if let Some(i) = rects.iter().position(|r| r.hit(ev.pos)) {
                    if val != (i == 0) {
                        match which {
                            0 => self.toggle_guide(),
                            1 => self.toggle_autoaim(),
                            _ => self.toggle_pickup(),
                        }
                    }
                    return;
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play(None);
            }
        }
    }

    fn step_tour(&mut self, d: i64) {
        if self.course != "tour" {
            return;
        }
        self.tour = py::pmod(self.tour - 1 + d, gen::TOUR_COURSES) + 1;
        self.tour_par = gen::course_par(self.tour);
        let t_ = self.tour;
        self.save_setting("tour", json!(t_));
        self.b.play_sound("move");
    }

    fn toggle_guide(&mut self) {
        self.guide = !self.guide;
        let v = self.guide;
        self.save_setting("guide", json!(v));
        self.b.play_sound("select");
    }

    fn toggle_autoaim(&mut self) {
        self.autoaim = !self.autoaim;
        let v = self.autoaim;
        self.save_setting("autoaim", json!(v));
        self.b.play_sound("select");
    }

    fn toggle_pickup(&mut self) {
        self.pickup = !self.pickup;
        let v = self.pickup;
        self.save_setting("pickup", json!(v));
        self.b.play_sound("select");
    }

    fn start_play(&mut self, single: Option<String>) {
        if self.course == "ugc" {
            self.single_map = single.unwrap_or_default();
        }
        self.new_round();
        self.state = St::Play;
        self.b.play_sound("click");
    }

    // ===================================================== Eigene Bahnen

    fn set_tab(&mut self, tab: &'static str) {
        if !TABS.contains(&tab) || tab == self.setup_tab {
            return;
        }
        self.setup_tab = tab;
        if tab == "maps" {
            match self.maps.as_mut() {
                None => self.maps = Some(MapList::new(&self.b, self.tab_bottom)),
                Some(ml) => {
                    ml.layout(&self.b, self.tab_bottom);
                    ml.reload();
                }
            }
        }
        self.b.play_sound("click");
    }

    fn edit_action(&mut self, act: EditAction) {
        match act {
            EditAction::None => {}
            EditAction::NewMap => {
                let m = ugc::new_map("", "", &ugc::last_author(), None);
                self.ugc_edit(m);
            }
            EditAction::Edit(m) => self.ugc_edit(m),
            EditAction::Play(id) => self.ugc_play(&id),
            EditAction::Close => self.ugc_close_editor(),
            EditAction::Test(hole) => self.ugc_test(hole),
        }
    }

    fn ugc_edit(&mut self, m: Map<String, Value>) {
        self.editor = Some(MapEditor::new(&self.b, m));
        self.state = St::Edit;
        self.b.play_sound("click");
    }

    fn ugc_close_editor(&mut self) {
        self.editor = None;
        self.state = St::Setup;
        self.setup_tab = "maps";
        if let Some(ml) = self.maps.as_mut() {
            ml.reload();
        }
        self.b.play_sound("click");
    }

    fn ugc_play(&mut self, map_id: &str) {
        self.course = "ugc";
        self.save_setting("course", json!("ugc"));
        self.save_setting("ugc_map", json!(map_id));
        self.best = Self::load_best();
        self.start_play(Some(map_id.to_string()));
    }

    fn ugc_test(&mut self, hole: Hole) {
        self.test_return = true;
        self.course = "ugc";
        self.holes = vec![hole];
        self.players = 1;
        self.cards = vec![vec![0; 1]];
        self.points = vec![0];
        self.hole_idx = 0;
        self.player = 0;
        self.winner = None;
        self.b.score = 0;
        self.b.game_over = false;
        self.rec = None;
        self.replay = None;
        self.start_hole();
        self.state = St::Play;
        self.b.play_sound("click");
    }

    fn back_to_editor(&mut self) {
        self.test_return = false;
        self.state = St::Edit;
        self.b.game_over = false;
        if self.editor.is_none() {
            self.ugc_close_editor();
        }
    }

    // ===================================================== Eingabe

    fn lock_power(&mut self, on: bool) {
        if on && (self.state != St::Play || self.phase != "aim") {
            return;
        }
        if on == self.power_lock {
            return;
        }
        self.power_lock = on;
        self.lock_t = 0.0;
        self.b.play_sound(if on { "select" } else { "click" });
    }

    fn cancel_shot(&mut self) {
        if !self.charging {
            return;
        }
        self.charging = false;
        if !self.power_lock {
            self.power = 0.35;
        }
        self.msg = Some(t("golf.cancel"));
        self.msg_t = 1.4;
        self.b.play_sound("click");
    }

    fn strike(&mut self) {
        let sp = MAX_SPEED * self.power;
        self.vx = self.aim.cos() * sp;
        self.vy = self.aim.sin() * sp;
        self.strokes += 1;
        self.phase = "rolling";
        self.shot_time = 0.0;
        self.safe = (self.bx, self.by);
        self.trail.clear();
        if let Some(r) = self.rec.as_mut() {
            let mut info = Map::new();
            info.insert("layout".into(), json!(self.layout_id));
            info.insert("hole".into(), json!(self.hole_idx));
            info.insert("player".into(), json!(self.player));
            info.insert("n".into(), json!(self.strokes));
            info.insert("aim".into(), replay::rnd(self.aim, 4));
            info.insert("power".into(), replay::rnd(self.power, 3));
            info.insert("mill".into(), replay::rnd(self.mill_a, 3));
            info.insert("move".into(), replay::rnd(self.move_t, 3));
            r.scene(true, info);
        }
        self.b.play_sound("shoot");
        self.b.rumble(50);
    }

    // ===================================================== Projektion

    fn view(&self) -> View {
        View { ox: self.ox, oy: self.oy, scale: self.scale }
    }

    fn project(&self, x: f64, y: f64) -> (f64, f64) {
        (self.ox + x * self.scale, self.oy + y * self.scale)
    }

    fn unproject(&self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.ox) / self.scale, (sy - self.oy) / self.scale)
    }

    // ===================================================== Physik

    fn physics(&mut self, dt: f64) {
        let speed = py::hypot(self.vx, self.vy);
        let steps = 2.max(24.min((speed * dt / BR) as i64 + 2));
        let hstep = dt / steps as f64;
        for _ in 0..steps {
            self.tun_cd = 0f64.max(self.tun_cd - hstep);
            self.boost_cd = 0f64.max(self.boost_cd - hstep);
            if self.air > 0.0 {
                let mv = py::hypot(self.vx, self.vy) * hstep;
                self.air -= mv;
                self.bx += self.vx * hstep;
                self.by += self.vy * hstep;
                self.collide_bounds();
                if self.air <= 0.0 || mv <= 0.0 {
                    self.air = 0.0;
                    self.b.play_sound("hit");
                }
                continue;
            }
            let fr = match self.terrain() {
                "sand" => FRIC_SAND,
                "ice" => FRIC_ICE,
                "sticky" => FRIC_STICKY,
                _ => FRIC_GREEN,
            };
            for s in self.hole.get("slopes").clone() {
                let (x, y, w, hh, ax, ay) = (s[0], s[1], s[2], s[3], s[4], s[5]);
                if x <= self.bx && self.bx <= x + w && y <= self.by && self.by <= y + hh {
                    self.vx += ax * hstep;
                    self.vy += ay * hstep;
                }
            }
            self.apply_magnets(hstep);
            let f = 0f64.max(1.0 - fr * hstep);
            self.vx *= f;
            self.vy *= f;
            self.bx += self.vx * hstep;
            self.by += self.vy * hstep;
            self.collide_bounds();
            for r in self.hole.get("walls").clone() {
                self.collide_rect(&r, (0.0, 0.0));
            }
            for g in self.hole.get("gates").clone() {
                if self.vx * g[4] + self.vy * g[5] > 0.0 {
                    continue;
                }
                self.collide_rect(&g, (0.0, 0.0));
            }
            for m in self.hole.get("movers").clone() {
                let (rect, vel) = mdraw::mover_rect(&m, self.move_t);
                self.collide_rect(&rect, vel);
            }
            for bmp in self.hole.get("bumpers").clone() {
                self.collide_bumper(&bmp);
            }
            for mill in self.hole.get("mills").clone() {
                self.collide_mill(&mill);
            }
            for sp in self.hole.get("spinners").clone() {
                self.collide_spinner(&sp, hstep);
            }
            self.apply_boosters();
            if self.check_jumps() || self.check_tunnels() {
                continue;
            }
            if self.check_cup() || self.check_water() {
                return;
            }
            if self.vx * self.vx + self.vy * self.vy < STOP_EPS * STOP_EPS {
                self.vx = 0.0;
                self.vy = 0.0;
                return;
            }
        }
        self.trail.push((self.bx, self.by));
        if self.trail.len() > 26 {
            self.trail.remove(0);
        }
    }

    fn terrain(&self) -> &'static str {
        for key in ["sticky", "sand", "ice"] {
            for r in self.hole.get(key) {
                if r[0] <= self.bx && self.bx <= r[0] + r[2] && r[1] <= self.by && self.by <= r[1] + r[3] {
                    return key;
                }
            }
        }
        "green"
    }

    fn apply_magnets(&mut self, hstep: f64) {
        for m in self.hole.get("magnets").clone() {
            let (x, y, r, force) = (m[0], m[1], m[2], m[3]);
            let (dx, dy) = (x - self.bx, y - self.by);
            let d = py::hypot(dx, dy);
            if d >= r || d < 0.4 {
                continue;
            }
            let pull = force * (1.0 - d / r) * hstep / d;
            self.vx += dx * pull;
            self.vy += dy * pull;
        }
    }

    fn collide_spinner(&mut self, sp: &[f64], hstep: f64) {
        let (x, y, r, speed) = (sp[0], sp[1], sp[2], sp[3]);
        let (dx, dy) = (self.bx - x, self.by - y);
        let d = py::hypot(dx, dy);
        if d >= r {
            return;
        }
        self.vx += (-dy * speed - self.vx) * SPIN_PUSH * hstep;
        self.vy += (dx * speed - self.vy) * SPIN_PUSH * hstep;
        if d > 0.4 {
            self.vx += dx / d * speed.abs() * 4.0 * hstep;
            self.vy += dy / d * speed.abs() * 4.0 * hstep;
        }
    }

    fn apply_boosters(&mut self) {
        if self.boost_cd > 0.0 {
            return;
        }
        for bo in self.hole.get("boosters").clone() {
            let (x, y, w, hh, dx, dy, boost) = (bo[0], bo[1], bo[2], bo[3], bo[4], bo[5], bo[6]);
            if !(x <= self.bx && self.bx <= x + w && y <= self.by && self.by <= y + hh) {
                continue;
            }
            let n = py::hypot(dx, dy);
            if n < 1e-6 || boost <= 0.0 {
                continue;
            }
            let (ux, uy) = (dx / n, dy / n);
            let along = self.vx * ux + self.vy * uy;
            if along < boost {
                self.vx += ux * (boost - along);
                self.vy += uy * (boost - along);
                self.boost_cd = BOOST_COOLDOWN;
                self.b.play_sound("bounce");
            }
            return;
        }
    }

    fn check_jumps(&mut self) -> bool {
        if self.air > 0.0 {
            return false;
        }
        for jp in self.hole.get("jumps").clone() {
            let (x, y, w, hh, dx, dy, dist) = (jp[0], jp[1], jp[2], jp[3], jp[4], jp[5], jp[6]);
            if !(x <= self.bx && self.bx <= x + w && y <= self.by && self.by <= y + hh) {
                continue;
            }
            let sp = py::hypot(self.vx, self.vy);
            if sp < JUMP_MIN_SPEED {
                return false;
            }
            let n = py::hypot(dx, dy);
            if n > 1e-6 {
                self.vx = dx / n * sp;
                self.vy = dy / n * sp;
            }
            self.air = 1f64.max(dist);
            self.b.play_sound("bounce");
            return true;
        }
        false
    }

    fn check_tunnels(&mut self) -> bool {
        if self.tun_cd > 0.0 {
            return false;
        }
        for tu in self.hole.get("tunnels").clone() {
            let (x1, y1, x2, y2, r) = (tu[0], tu[1], tu[2], tu[3], tu[4]);
            for (ex, ey, ox, oy) in [(x1, y1, x2, y2), (x2, y2, x1, y1)] {
                if py::hypot(self.bx - ex, self.by - ey) > r {
                    continue;
                }
                let sp = py::hypot(self.vx, self.vy) * TUNNEL_KEEP;
                if sp < 1e-6 {
                    return false;
                }
                let n = py::hypot(self.vx, self.vy);
                let (ux, uy) = (self.vx / n, self.vy / n);
                self.bx = ox + ux * (r + BR + 0.5);
                self.by = oy + uy * (r + BR + 0.5);
                self.vx = ux * sp;
                self.vy = uy * sp;
                self.tun_cd = TUNNEL_COOLDOWN;
                self.b.play_sound("hit");
                return true;
            }
        }
        false
    }

    fn collide_bounds(&mut self) {
        let lo = BORDER + BR;
        let (hix, hiy) = (self.cw - BORDER - BR, self.ch - BORDER - BR);
        if self.bx < lo {
            self.bx = lo;
            self.vx = self.vx.abs() * WALL_E;
            self.thud();
        } else if self.bx > hix {
            self.bx = hix;
            self.vx = -self.vx.abs() * WALL_E;
            self.thud();
        }
        if self.by < lo {
            self.by = lo;
            self.vy = self.vy.abs() * WALL_E;
            self.thud();
        } else if self.by > hiy {
            self.by = hiy;
            self.vy = -self.vy.abs() * WALL_E;
            self.thud();
        }
    }

    fn thud(&self) {
        if self.vx.abs() + self.vy.abs() > 45.0 {
            self.b.play_sound("bounce");
        }
    }

    fn collide_rect(&mut self, r: &[f64], vel: (f64, f64)) {
        let (x, y, w, hh) = (r[0], r[1], r[2], r[3]);
        let nx = x.max(self.bx.min(x + w));
        let ny = y.max(self.by.min(y + hh));
        let (dx, dy) = (self.bx - nx, self.by - ny);
        let d2 = dx * dx + dy * dy;
        if d2 >= BR * BR {
            return;
        }
        let (ux, uy, push);
        if d2 > 1e-9 {
            let d = d2.sqrt();
            ux = dx / d;
            uy = dy / d;
            push = BR - d;
        } else {
            let (left, right) = (self.bx - x, x + w - self.bx);
            let (top, bottom) = (self.by - y, y + hh - self.by);
            let m = left.min(right).min(top).min(bottom);
            if m == left {
                (ux, uy, push) = (-1.0, 0.0, left + BR);
            } else if m == right {
                (ux, uy, push) = (1.0, 0.0, right + BR);
            } else if m == top {
                (ux, uy, push) = (0.0, -1.0, top + BR);
            } else {
                (ux, uy, push) = (0.0, 1.0, bottom + BR);
            }
        }
        self.bx += ux * push;
        self.by += uy * push;
        let (mut rvx, mut rvy) = (self.vx - vel.0, self.vy - vel.1);
        let dot = rvx * ux + rvy * uy;
        if dot < 0.0 {
            rvx -= (1.0 + WALL_E) * dot * ux;
            rvy -= (1.0 + WALL_E) * dot * uy;
            self.vx = rvx + vel.0 * 1.4;
            self.vy = rvy + vel.1 * 1.4;
            self.thud();
        }
    }

    fn collide_bumper(&mut self, bmp: &[f64]) {
        let (x, y, r) = (bmp[0], bmp[1], bmp[2]);
        let (dx, dy) = (self.bx - x, self.by - y);
        let d = py::hypot(dx, dy);
        let rad = r + BR;
        if d >= rad || d < 1e-9 {
            return;
        }
        let (ux, uy) = (dx / d, dy / d);
        self.bx = x + ux * rad;
        self.by = y + uy * rad;
        let dot = self.vx * ux + self.vy * uy;
        if dot < 0.0 {
            self.vx -= (1.0 + BUMP_E) * dot * ux;
            self.vy -= (1.0 + BUMP_E) * dot * uy;
            self.b.play_sound("bounce");
        }
    }

    fn collide_mill(&mut self, mill: &[f64]) {
        let (x, y, length, arms, speed) = (mill[0], mill[1], mill[2], mill[3], mill[4]);
        let base = self.mill_a * speed;
        let n = arms as i64;
        for i in 0..n {
            let a = base + i as f64 * (2.0 * PI / n as f64);
            let (ex, ey) = (x + a.cos() * length, y + a.sin() * length);
            let (px, py_) = Self::closest_on_seg(x, y, ex, ey, self.bx, self.by);
            let (mut dx, mut dy) = (self.bx - px, self.by - py_);
            let mut d = py::hypot(dx, dy);
            let rad = BR + ARM_W;
            if d >= rad {
                continue;
            }
            if d < 1e-9 {
                (dx, dy, d) = (0.0, -1.0, 1.0);
            }
            let (ux, uy) = (dx / d, dy / d);
            self.bx = px + ux * rad;
            self.by = py_ + uy * rad;
            let (rvx, rvy) = (-(py_ - y) * speed, (px - x) * speed);
            let (mut relx, mut rely) = (self.vx - rvx, self.vy - rvy);
            let dot = relx * ux + rely * uy;
            if dot < 0.0 {
                relx -= 1.8 * dot * ux;
                rely -= 1.8 * dot * uy;
            }
            self.vx = relx + rvx;
            self.vy = rely + rvy;
            self.b.play_sound("hit");
            return;
        }
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

    fn check_cup(&mut self) -> bool {
        let (dx, dy) = (self.cup.0 - self.bx, self.cup.1 - self.by);
        let d = py::hypot(dx, dy);
        let sp = py::hypot(self.vx, self.vy);
        if d < CUP_R + BR && sp < CAPTURE_SPEED * 1.6 {
            let pull = 26.0 * (1.0 - d / (CUP_R + BR)) / d.max(0.4);
            self.vx += dx * pull;
            self.vy += dy * pull;
        }
        if d < CUP_R * 0.75 && sp < CAPTURE_SPEED {
            self.vx = 0.0;
            self.vy = 0.0;
            self.bx = self.cup.0;
            self.by = self.cup.1;
            self.holed();
            return true;
        }
        false
    }

    fn check_water(&mut self) -> bool {
        for r in self.hole.get("water").clone() {
            if r[0] <= self.bx && self.bx <= r[0] + r[2] && r[1] <= self.by && self.by <= r[1] + r[3] {
                self.vx = 0.0;
                self.vy = 0.0;
                self.strokes += 1;
                self.rec_end("water");
                self.bx = self.safe.0;
                self.by = self.safe.1;
                self.msg = Some(t("golf.penalty"));
                self.msg_t = 2.0;
                self.b.play_sound("hit");
                if self.pickup && self.strokes >= MAX_STROKES {
                    self.finish_hole(false);
                } else {
                    self.phase = "aim";
                    self.reset_aim(false);
                }
                return true;
            }
        }
        false
    }

    fn reset_aim(&mut self, new_hole: bool) {
        if self.autoaim {
            self.aim = (self.cup.1 - self.by).atan2(self.cup.0 - self.bx);
        } else if new_hole {
            self.aim = -PI / 2.0;
        }
        if !self.power_lock {
            self.power = 0.35;
        }
        self.charging = false;
    }

    fn after_shot(&mut self) {
        self.rec_end("stop");
        if self.pickup && self.strokes >= MAX_STROKES {
            self.msg = Some(t("golf.max_strokes"));
            self.msg_t = 2.4;
            self.finish_hole(false);
            return;
        }
        self.phase = "aim";
        self.reset_aim(false);
    }

    // ===================================================== Bahn abschliessen

    fn holed(&mut self) {
        self.rec_end("cup");
        self.b.play_sound(if self.strokes == 1 { "win" } else { "point" });
        self.b.rumble(90);
        let (px, py_) = self.project(self.cup.0, self.cup.1);
        ui::spawn_burst(px, py_, Some(self.b.accent), 16, 260.0);
        self.finish_hole(true);
    }

    fn finish_hole(&mut self, holed: bool) {
        self.cards[self.player][self.hole_idx] = self.strokes;
        let mut pts;
        if holed {
            pts = 100.max(600 + (self.par - self.strokes) * 300);
            if self.strokes == 1 {
                pts += 500;
                self.b.ach_event("golf_ace", None);
            }
        } else {
            pts = 100;
        }
        self.points[self.player] += pts;
        self.result_pts = pts;
        let key = Self::result_key(self.strokes, self.par, holed);
        self.result_key = Some(key.to_string());
        if let Some(r) = self.rec.as_mut() {
            let mut info = Map::new();
            info.insert("final".into(), json!(self.strokes));
            info.insert("result".into(), json!(key));
            info.insert("pts".into(), json!(pts));
            info.insert("holed".into(), json!(holed));
            r.set_last(info);
        }
        if self.player == 0 {
            self.b.score = self.points[0];
        }
        self.phase = "done";
        self.state = St::HoleDone;
    }

    fn result_key(strokes: i64, par: i64, holed: bool) -> &'static str {
        if !holed {
            return "golf.res.max";
        }
        if strokes == 1 {
            return "golf.res.ace";
        }
        match strokes - par {
            d if d <= -2 => "golf.res.eagle",
            -1 => "golf.res.birdie",
            0 => "golf.res.par",
            1 => "golf.res.bogey",
            2 => "golf.res.double",
            _ => "golf.res.over",
        }
    }

    fn advance(&mut self) {
        self.b.play_sound("click");
        if self.player + 1 < self.players {
            self.player += 1;
            self.state = St::Play;
            self.start_hole();
            return;
        }
        self.player = 0;
        if self.hole_idx + 1 < self.holes.len() {
            self.hole_idx += 1;
            self.state = St::Play;
            self.start_hole();
            return;
        }
        self.end_round();
    }

    fn next_course(&self) -> Option<(&'static str, i64)> {
        match self.course {
            "classic" => Some(("pro", self.tour)),
            "pro" => Some(("tour", 1)),
            "tour" if self.tour < gen::TOUR_COURSES => Some(("tour", self.tour + 1)),
            _ => None,
        }
    }

    fn next_course_label(&self) -> String {
        match self.next_course() {
            None => String::new(),
            Some((course, num)) => {
                let name = t(&format!("golf.course.{course}"));
                if course == "tour" {
                    format!("{name} {num}")
                } else {
                    name
                }
            }
        }
    }

    fn over_keys(&self) -> Vec<&'static str> {
        let mut keys = vec!["again", "setup"];
        if self.replay.is_some() {
            keys.insert(0, "replay");
        }
        if self.next_course().is_some() {
            keys.insert(0, "next");
        }
        keys
    }

    fn build_over_layout(&mut self) {
        let keys = self.over_keys();
        let head_h = self.huge.get_height();
        let line_h = self.small.get_height();
        let tiny_h = self.tiny.get_height();
        let bh = 26.max(38.min(line_h + 10));
        let mut y = OVER_PAD;
        self.over_y.clear();
        for (name, hh, gap) in [("head", head_h, 6), ("sub", line_h, 4), ("best", tiny_h, 10)] {
            self.over_y.insert(name, y + hh / 2);
            y += hh + gap;
        }
        let btn_top = y;
        y += bh + 8;
        self.over_y.insert("hint", y + tiny_h / 2);
        self.over_h = y + tiny_h + OVER_PAD;
        let gap = 8.0;
        let bw = ((self.b.width as f64 * 0.74) as i32).min(660) as f64;
        let n = keys.len() as f64;
        let cwid = (bw - gap * (n - 1.0)) / n;
        let cx = (self.b.width / 2) as f64;
        let top = self.b.height / 2 - self.over_h / 2 + btn_top;
        self.over_rects = keys.iter().enumerate().map(|(i, k)| (*k, Rect::new((cx - bw / 2.0 + i as f64 * (cwid + gap)) as i32, top, cwid as i32, bh))).collect();
    }

    fn continue_next(&mut self) {
        let Some((course, tour)) = self.next_course() else {
            self.restart();
            return;
        };
        self.course = course;
        self.tour = tour;
        self.tour_par = gen::course_par(self.tour);
        self.save_setting("course", json!(course));
        self.save_setting("tour", json!(tour));
        self.build_over_layout();
        self.new_round();
        self.state = St::Play;
        self.b.play_sound("click");
    }

    fn over_action(&mut self, key: &str) {
        match key {
            "replay" => self.open_replay(),
            "next" if self.next_course().is_some() => self.continue_next(),
            "setup" => {
                self.state = St::Setup;
                self.b.game_over = false;
                self.b.play_sound("click");
            }
            _ => self.restart(),
        }
    }

    fn reset_hole(&mut self) {
        if self.state != St::Play {
            return;
        }
        if let Some(r) = self.rec.as_mut() {
            let mut m = Map::new();
            m.insert("hole".into(), json!(self.hole_idx));
            m.insert("player".into(), json!(self.player));
            r.drop_where(&m);
        }
        self.start_hole();
        self.msg = Some(t("golf.reset"));
        self.msg_t = 1.4;
        self.b.play_sound("click");
    }

    fn end_round(&mut self) {
        if self.test_return {
            self.back_to_editor();
            return;
        }
        let total: i64 = self.cards[0].iter().sum();
        let par_total: i64 = self.holes.iter().map(|h| h.par).sum();
        self.save_best(total);
        if total < par_total {
            self.b.ach_event("golf_under_par", None);
        }
        if self.b.multiplayer {
            let t0: i64 = self.cards[0].iter().sum();
            let t1: i64 = self.cards[1].iter().sum();
            self.winner = if t0 < t1 {
                Some(0)
            } else if t1 < t0 {
                Some(1)
            } else {
                None
            };
            if let Some(w) = self.winner {
                self.b.report_duel(w);
            }
        } else {
            self.winner = None;
            self.b.report_result(total <= par_total);
        }
        self.b.score = self.points[0];
        self.rec_finish(total, par_total);
        self.build_over_layout();
        self.state = St::Over;
        self.b.game_over = true;
        self.b.play_sound("win");
    }

    fn restart(&mut self) {
        self.new_round();
        self.state = St::Play;
        self.b.play_sound("click");
    }

    // ===================================================== Replay-Aufnahme

    fn rec_new(&mut self) {
        self.replay = None;
        let mut meta = Map::new();
        meta.insert("course".into(), json!(self.course));
        meta.insert("tour".into(), json!(self.tour));
        meta.insert("players".into(), json!(self.players));
        self.rec = replay::recorder("minigolf", shell::replay_enabled(&self.b.settings), meta);
    }

    fn rec_sample(&self) -> Vec<Value> {
        vec![replay::rnd(self.bx, 2), replay::rnd(self.by, 2)]
    }

    fn rec_end(&mut self, end: &str) {
        let sample = self.rec_sample();
        if let Some(r) = self.rec.as_mut() {
            let mut info = Map::new();
            info.insert("end".into(), json!(end));
            info.insert("after".into(), json!(self.strokes));
            r.close(Some(sample), info);
        }
    }

    fn rec_finish(&mut self, total: i64, par_total: i64) {
        let Some(mut r) = self.rec.take() else {
            return;
        };
        let d = total - par_total;
        let mut name = t(&format!("golf.course.{}", self.course));
        if self.course == "tour" {
            name = format!("{} {}", name, self.tour);
        }
        let diff = if d != 0 { format!("{d:+}") } else { t("golf.even") };
        let sub = tf("golf.final", &[("strokes", &total.to_string()), ("diff", &diff), ("pts", &self.points[0].to_string())]);
        let mut meta = Map::new();
        meta.insert("total".into(), json!(total));
        meta.insert("par".into(), json!(par_total));
        meta.insert("points".into(), json!(self.points[0]));
        self.replay = r.result(&name, &sub, meta);
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
        self.rep_layouts = rep.get("layouts").and_then(|l| l.as_array()).map(|a| a.iter().map(Hole::from_json).collect()).unwrap_or_default();
        let meta = rep.get("meta").cloned().unwrap_or(json!({}));
        if let Some(c) = meta.get("course").and_then(|v| v.as_str()) {
            if let Some(cc) = COURSES.iter().find(|x| **x == c) {
                self.course = cc;
            }
        }
        self.tour = meta.get("tour").and_then(|v| v.as_f64()).map(|v| 1.max(gen::TOUR_COURSES.min(v as i64))).unwrap_or(1);
        self.players = meta.get("players").and_then(|v| v.as_f64()).map(|v| 1.max(2.min(v as i64))).unwrap_or(1) as usize;
        self.b.multiplayer = self.players > 1;
        let mut order: Vec<i64> = Vec::new();
        let mut holes: HashMap<i64, Hole> = HashMap::new();
        for sc in rep.get("scenes").and_then(|s| s.as_array()).cloned().unwrap_or_default() {
            let hidx = sc.get("hole").and_then(|v| v.as_i64()).unwrap_or(0);
            if let std::collections::hash_map::Entry::Vacant(eintrag) = holes.entry(hidx) {
                let idx = sc.get("layout").and_then(|v| v.as_i64()).unwrap_or(0);
                if idx >= 0 && (idx as usize) < self.rep_layouts.len() {
                    eintrag.insert(self.rep_layouts[idx as usize].clone());
                    order.push(hidx);
                }
            }
        }
        self.holes = order.iter().filter_map(|hh| holes.get(hh).cloned()).collect();
        self.rep_pos = order.iter().enumerate().map(|(i, hh)| (*hh, i)).collect();
        self.cards = vec![vec![0; self.holes.len()]; self.players];
        self.points = vec![0; self.players];
        self.state = St::Play;
        self.phase = "aim";
        self.b.game_over = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.charging = false;
        self.hole_idx = 0;
        self.player = 0;
        self.strokes = 0;
        self.trail.clear();
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
        let geti = |v: &Value, k: &str, d: i64| v.get(k).and_then(|x| x.as_f64()).map(|x| x as i64).unwrap_or(d);
        let lay = geti(sc, "layout", 0);
        if lay >= 0 && (lay as usize) < self.rep_layouts.len() {
            self.hole = self.rep_layouts[lay as usize].clone();
        }
        if (self.hole.w, self.hole.h) != (self.cw, self.ch) {
            self.cw = self.hole.w;
            self.ch = self.hole.h;
            self.layout();
        }
        self.hole_idx = *self.rep_pos.get(&geti(sc, "hole", 0)).unwrap_or(&0);
        self.par = self.hole.par;
        self.cup = self.hole.cup;
        self.player = (self.players as i64 - 1).min(0.max(geti(sc, "player", 0))) as usize;
        self.aim = sc.get("aim").and_then(|v| v.as_f64()).unwrap_or(0.0);
        self.power = sc.get("power").and_then(|v| v.as_f64()).unwrap_or(0.35);
        self.result_key = sc.get("result").and_then(|v| v.as_str()).map(|s| s.to_string());
        self.result_pts = geti(sc, "pts", 0);
        let pts: Vec<f64> = sc.get("f").and_then(|f| f.as_array()).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()).unwrap_or_default();
        let n = 1.max(pts.len() / 2);
        let frame = frame.min(n - 1);
        if pts.len() >= 2 * frame + 2 {
            self.bx = pts[2 * frame];
            self.by = pts[2 * frame + 1];
        }
        self.trail = (frame.saturating_sub(26)..frame).filter(|k| 2 * k + 1 < pts.len()).map(|k| (pts[2 * k], pts[2 * k + 1])).collect();
        let rate = rep.get("rate").and_then(|r| r.as_f64()).filter(|r| *r != 0.0).unwrap_or(replay::RATE);
        self.mill_a = sc.get("mill").and_then(|v| v.as_f64()).unwrap_or(0.0) + frame as f64 / rate;
        self.move_t = sc.get("move").and_then(|v| v.as_f64()).unwrap_or(0.0) + frame as f64 / rate;
        let last = frame >= n - 1;
        self.strokes = if last { sc.get("after").and_then(|v| v.as_i64()).unwrap_or_else(|| geti(sc, "n", 1)) } else { geti(sc, "n", 1) };
        self.cards = vec![vec![0; self.holes.len()]; self.players];
        self.points = vec![0; self.players];
        let mut prev_list: Vec<&Value> = scenes[..index].iter().collect();
        if last {
            prev_list.push(sc);
        }
        for prev in prev_list {
            let fin = geti(prev, "final", 0);
            let p = (self.players as i64 - 1).min(0.max(geti(prev, "player", 0))) as usize;
            if let Some(pos) = self.rep_pos.get(&geti(prev, "hole", 0)) {
                if fin != 0 {
                    self.cards[p][*pos] = fin;
                    self.points[p] += geti(prev, "pts", 0);
                }
            }
        }
        self.msg = if last && sc.get("end").and_then(|v| v.as_str()) == Some("water") { Some(t("golf.penalty")) } else { None };
    }

    /// `replay_draw(aiming, banner)`
    pub fn replay_draw(&mut self, aiming: bool, banner: bool) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        self.draw_course(&mut s);
        self.draw_ball(&mut s);
        if aiming {
            self.draw_aim(&mut s);
        }
        self.draw_hud(&mut s);
        self.draw_card(&mut s);
        if banner && self.result_key.is_some() {
            self.draw_hole_done(&mut s);
        }
    }

    // ===================================================== Zeichnen

    fn draw_course(&self, s: &mut Surface) {
        mdraw::draw_course(s, &self.hole, &self.view(), self.mill_a, self.move_t, CUP_R, false);
    }

    fn draw_ball(&self, s: &mut Surface) {
        let (px, py_) = self.project(self.bx, self.by);
        let r = 2.max((BR * self.scale) as i32);
        let n = self.trail.len();
        for (i, (tx, ty)) in self.trail.iter().enumerate() {
            let a = (i + 1) as f64 / (n + 1) as f64;
            let (tpx, tpy) = self.project(*tx, *ty);
            draw::circle(s, ui::mix(COL_GREEN, COL_BALL, a * 0.35), (tpx as i32, tpy as i32), 1.max((r as f64 * 0.6) as i32), 0);
        }
        draw::circle(s, (18, 46, 28), ((px + 2.0) as i32, (py_ + 2.0) as i32), r, 0);
        draw::circle(s, COL_BALL, (px as i32, py_ as i32), r, 0);
        draw::circle(s, (206, 210, 200), (px as i32, py_ as i32), r, 1);
        if r >= 5 {
            draw::circle(s, (255, 255, 255), ((px - (r / 3) as f64) as i32, (py_ - (r / 3) as f64) as i32), 1.max(r / 3), 0);
        }
    }

    fn draw_aim(&self, s: &mut Surface) {
        let (px, py_) = self.project(self.bx, self.by);
        let (ox, oy) = (self.aim.cos(), self.aim.sin());
        let lock = self.power_lock;
        let col = if lock { COL_LOCK } else { COL_AIM };
        if self.guide {
            let steps = 6.max(((30.0 + 60.0 * self.power) * self.scale / 9.0) as i64);
            for i in (0..steps).step_by(2) {
                let i = i as f64;
                draw::line(s, col, (px + ox * (i * 9.0 + 6.0), py_ + oy * (i * 9.0 + 6.0)), (px + ox * (i * 9.0 + 12.0), py_ + oy * (i * 9.0 + 12.0)), 2);
            }
        }
        if lock {
            let puls = 0.5 + 0.5 * (self.lock_t * 5.0).sin();
            let rr = (6f64.max(BR * self.scale) + 5.0 + puls * 4.0) as i32;
            draw::circle(s, COL_LOCK, (px as i32, py_ as i32), rr, 2);
        }
        let bx1 = px - ox * (10.0 + self.power * 26.0);
        let by1 = py_ - oy * (10.0 + self.power * 26.0);
        draw::line(s, (228, 228, 222), (bx1, by1), (bx1 - ox * 26.0, by1 - oy * 26.0), 3);
        draw::line(s, if lock { COL_LOCK } else { (150, 154, 160) }, (bx1, by1), (bx1 - oy * 7.0, by1 + ox * 7.0), 5);
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        draw::rect(s, p.panel, (0, 0, self.b.width, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (self.b.width, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let left_txt = tf("golf.hole", &[("n", &(self.hole_idx + 1).to_string()), ("total", &self.holes.len().to_string())]) + "  \u{00B7}  " + &tf("golf.par", &[("n", &self.par.to_string())]);
        let left = self.tiny.render(&left_txt, true, p.text_dim);
        s.blr(&left, left.get_rect().midleft_at((12, cy)));
        let mut col = self.b.accent;
        let mid = if self.power_lock && self.phase == "aim" {
            col = COL_LOCK.c();
            tf("golf.lock", &[("n", &self.power_pct().to_string())])
        } else if let Some(m) = self.msg.as_ref().filter(|m| !m.is_empty()) {
            m.clone()
        } else if self.b.multiplayer {
            tf("golf.turn", &[("name", &self.b.pname(self.player).chars().take(10).collect::<String>())])
        } else {
            tf("golf.strokes", &[("n", &self.strokes.to_string())])
        };
        let img = self.small.render(&mid, true, col);
        s.blr(&img, img.get_rect().center_at((self.b.width / 2, cy)));
        self.draw_power(s, cy);
    }

    fn power_pct(&self) -> i64 {
        py::round(self.power * 100.0)
    }

    fn draw_power(&self, s: &mut Surface, cy: i32) {
        let p = ui::pal();
        let (mw, mh) = (84, 10);
        let mx = self.b.width - mw - 14;
        let top = cy - mh / 2;
        let lock = self.power_lock;
        let col = if lock { COL_LOCK.c() } else { self.b.accent };
        draw::rect_r(s, p.btn, (mx, top, mw, mh), 0, 4);
        let fill = (mw as f64 * self.power) as i32;
        if fill > 0 {
            draw::rect_r(s, col, (mx, top, fill, mh), 0, 4);
        }
        let pct = self.tiny.render(&format!("{}%", self.power_pct()), true, if lock { col } else { p.text_dim });
        s.blr(&pct, pct.get_rect().midright_at((mx - if lock { 21 } else { 6 }, cy)));
        if !lock {
            return;
        }
        Self::draw_lock_icon(s, mx - 12, cy, col);
        let tick = mx + 1.max((mw - 1).min(fill));
        draw::line(s, (255, 250, 232), (tick, top - 3), (tick, top + mh + 2), 2);
        let puls = 0.5 + 0.5 * (self.lock_t * 5.0).sin();
        draw::rect_r(s, ui::mix(col, (255, 255, 255), 0.10 + 0.35 * puls), (mx - 2, top - 2, mw + 4, mh + 4), 1, 6);
    }

    fn draw_lock_icon(s: &mut Surface, x: i32, cy: i32, col: Color) {
        draw::arc(s, col, (x - 3, cy - 8, 7, 8), 0.0, PI, 2);
        draw::line(s, col, (x - 3, cy - 5), (x - 3, cy - 1), 1);
        draw::line(s, col, (x + 3, cy - 5), (x + 3, cy - 1), 1);
        draw::rect_r(s, col, (x - 5, cy - 1, 11, 8), 0, 2);
        draw::rect(s, ui::pal().panel, (x - 1, cy + 2, 2, 3), 0);
    }

    fn card_head(&self, p: usize) -> String {
        let name = self.b.pname(p);
        if name != t(&format!("common.player{}", p + 1)) {
            name.chars().take(2).collect()
        } else {
            (p + 1).to_string()
        }
    }

    fn draw_card(&self, s: &mut Surface) {
        let p = ui::pal();
        let (x, w) = (self.card_x, self.card_w);
        let y = self.hud_h + 10;
        let line = self.card.get_height() + 2;
        let rect = Rect::new(x, y, w, (self.holes.len() as i32 + 2) * line + 18);
        if rect.bottom() > self.b.height - 4 {
            return;
        }
        ui::draw_panel_ex(s, rect, None, None, Some(8), false, None);
        let mut cy = y + 8;
        let (par_x, me_x, p2_x) = (x + w - 62, x + w - 38, x + w - 16);
        s.bl(&self.card.render(&t("golf.card_hole"), true, p.text_dim), (x + 8, cy));
        s.bl(&self.card.render(&t("golf.card_par"), true, p.text_dim), (par_x, cy));
        s.bl(&self.card.render(&self.card_head(0), true, self.b.accent), (me_x, cy));
        if self.b.multiplayer {
            s.bl(&self.card.render(&self.card_head(1), true, p.gold), (p2_x, cy));
        }
        cy += line + 2;
        for (i, hole) in self.holes.iter().enumerate() {
            let active = i == self.hole_idx;
            s.bl(&self.card.render(&(i + 1).to_string(), true, if active { p.text } else { p.text_dim }), (x + 8, cy));
            s.bl(&self.card.render(&hole.par.to_string(), true, p.text_faint), (par_x, cy));
            for (pl, cx) in [(0usize, me_x), (1, p2_x)] {
                if pl >= self.players {
                    continue;
                }
                let mut v = self.cards[pl][i];
                if active && pl == self.player && self.state == St::Play {
                    v = self.strokes;
                }
                if v != 0 {
                    let col = if v < hole.par {
                        p.green
                    } else if v > hole.par {
                        p.red
                    } else {
                        p.text
                    };
                    s.bl(&self.card.render(&v.to_string(), true, col), (cx, cy));
                }
            }
            cy += line;
        }
        draw::line(s, p.border, (x + 6, cy + 1), (x + w - 6, cy + 1), 1);
        cy += 5;
        s.bl(&self.card.render(&t("golf.card_sum"), true, p.text_dim), (x + 8, cy));
        let par_sum: i64 = self.holes.iter().map(|h| h.par).sum();
        s.bl(&self.card.render(&par_sum.to_string(), true, p.text_faint), (par_x, cy));
        for (pl, cx) in [(0usize, me_x), (1, p2_x)] {
            if pl < self.players {
                let sum: i64 = self.cards[pl].iter().sum();
                s.bl(&self.card.render(&sum.to_string(), true, if pl == 0 { self.b.accent } else { p.gold }), (cx, cy));
            }
        }
    }

    fn banner(&mut self, s: &mut Surface, hh: i32) -> i32 {
        let fresh = self.over_cache.as_ref().is_some_and(|c| c.w() == self.b.width && c.h() == hh);
        if !fresh {
            let mut ov = pg::surface_alpha(self.b.width, hh);
            ov.fill_c(rgba(8, 14, 10, 214));
            self.over_cache = Some(ov);
        }
        let y = self.b.height / 2 - hh / 2;
        if let Some(c) = self.over_cache.as_ref() {
            s.bl(c, (0, y));
        }
        draw::line(s, self.b.accent, (0, y), (self.b.width, y), 1);
        draw::line(s, self.b.accent, (0, y + hh - 1), (self.b.width, y + hh - 1), 1);
        y
    }

    fn draw_hole_done(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let y = self.banner(s, 104);
        let cx = self.b.width / 2;
        let head = self.huge.render(&t(self.result_key.as_deref().unwrap_or("golf.res.par")), true, self.b.accent);
        s.blr(&head, head.get_rect().center_at((cx, y + 34)));
        let sub = self.small.render(&tf("golf.hole_result", &[("strokes", &self.strokes.to_string()), ("par", &self.par.to_string()), ("pts", &self.result_pts.to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, y + 66)));
        let hint = self.tiny.render(&t("golf.next"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + 90)));
    }

    fn draw_over(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let oh = self.over_h;
        let y = self.banner(s, oh);
        let cx = self.b.width / 2;
        let total: i64 = self.cards[0].iter().sum();
        let par_total: i64 = self.holes.iter().map(|h| h.par).sum();
        let head_txt = if self.b.multiplayer {
            match self.winner {
                None => t("common.draw"),
                Some(w) => tf("common.player_wins_name", &[("name", &self.b.pname(w).chars().take(10).collect::<String>())]),
            }
        } else {
            t("golf.round_done")
        };
        let head = self.huge.render(&head_txt, true, self.b.accent);
        s.blr(&head, head.get_rect().center_at((cx, y + self.over_y["head"])));
        let d = total - par_total;
        let diff = if d != 0 { format!("{d:+}") } else { t("golf.even") };
        let sub = self.small.render(&tf("golf.final", &[("strokes", &total.to_string()), ("diff", &diff), ("pts", &self.points[0].to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, y + self.over_y["sub"])));
        if let Some(best) = self.best_get().filter(|b| *b != 0) {
            let b = self.tiny.render(&tf("golf.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, y + self.over_y["best"])));
        }
        let first = self.over_rects.first().map(|e| e.0);
        for (key, rc) in self.over_rects.clone() {
            let label = match key {
                "next" => tf("golf.btn_next", &[("course", &self.next_course_label())]),
                "again" => t("golf.btn_again"),
                "setup" => t("golf.btn_setup"),
                _ => t("golf.btn_replay"),
            };
            self.btn(s, rc, &label, Some(key) == first);
        }
        let mut hint_txt = t(if self.next_course().is_some() { "golf.continue_hint" } else { "golf.new_round" });
        if self.replay.is_some() {
            hint_txt += &("  \u{00B7}  ".to_string() + &t("golf.replay_hint"));
        }
        let hint = self.tiny.render(&hint_txt, true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + self.over_y["hint"])));
    }

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let title = self.huge.render(&t("golf.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (self.b.height as f64 * 0.115) as i32)));
        let sub_key = if self.setup_tab == "play" { "golf.subtitle" } else { "golf.ugc.subtitle" };
        let sub = self.small.render(&t(sub_key), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (self.b.height as f64 * 0.18) as i32)));
        for (i, rc) in self.tab_rects.clone().iter().enumerate() {
            self.btn(s, *rc, &t(&format!("golf.tab_{}", TABS[i])), self.setup_tab == TABS[i]);
        }
        if self.setup_tab == "maps" {
            if self.maps.is_none() {
                self.maps = Some(MapList::new(&self.b, self.tab_bottom));
            }
            if let Some(ml) = self.maps.as_ref() {
                ml.draw(s, &self.b);
            }
            return;
        }
        let tiny = self.tiny.clone();
        let label = |s: &mut Surface, rects: &[Rect], txt: &str, w: Option<i32>| {
            let mut im = tiny.render(txt, true, p.text_dim);
            if let Some(w) = w {
                if im.w() > w {
                    let mut kurz: Vec<char> = txt.chars().collect();
                    while kurz.len() > 2 && tiny.size(&(kurz.iter().collect::<String>() + "...")).0 > w {
                        kurz.pop();
                    }
                    im = tiny.render(&(kurz.iter().collect::<String>() + "..."), true, p.text_dim);
                }
            }
            let mid = (rects[0].x + rects[rects.len() - 1].right()) / 2;
            s.blr(&im, im.get_rect().midbottom_at((mid, rects[0].y - 4)));
        };
        let cr = self.course_rects.clone();
        label(s, &cr, &t("golf.lbl_course"), None);
        for (i, rc) in cr.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("golf.course.{}", COURSES[i])), self.course == COURSES[i]);
        }
        self.draw_tour_row(s);
        let groups = [
            (self.guide_rects.clone(), "golf.lbl_guide", self.guide),
            (self.autoaim_rects.clone(), "golf.lbl_autoaim", self.autoaim),
            (self.pickup_rects.clone(), "golf.lbl_pickup", self.pickup),
        ];
        for (rects, key, on) in groups {
            label(s, &rects, &t(key), Some(self.opt_w));
            for (i, rc) in rects.iter().enumerate() {
                self.btn(s, *rc, &if i == 0 { t("common.on") } else { t("common.off") }, on == (i == 0));
            }
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        if let Some(best) = self.best_get().filter(|b| *b != 0) {
            let b = self.tiny.render(&tf("golf.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, self.start_rect.bottom() + 14)));
        }
        let hint = self.tiny.render(&t("golf.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, self.b.height - 12)));
    }

    fn draw_tour_row(&self, s: &mut Surface) {
        let p = ui::pal();
        let (left, mid, right) = (self.tour_rects[0], self.tour_rects[1], self.tour_rects[2]);
        let on = self.course == "tour";
        let im = self.tiny.render(&t("golf.lbl_tour"), true, if on { p.text_dim } else { p.text_faint });
        s.blr(&im, im.get_rect().midbottom_at((self.b.width / 2, left.y - 3)));
        let col = if on { self.b.accent } else { p.border };
        for (rc, arrow) in [(left, "<"), (right, ">")] {
            draw::rect_r(s, p.btn, rc, 0, 7);
            draw::rect_r(s, col, rc, 1, 7);
            let a = self.small.render(arrow, true, if on { p.text } else { p.text_faint });
            s.blr(&a, a.get_rect().center_at(rc.center()));
        }
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, mid, 0, 7);
        draw::rect_r(s, col, mid, if on { 2 } else { 1 }, 7);
        let mut txt = tf("golf.tour", &[("n", &self.tour.to_string()), ("total", &gen::TOUR_COURSES.to_string())]);
        if on {
            txt += &("  \u{00B7}  ".to_string() + &tf("golf.par", &[("n", &self.tour_par.to_string())]));
        }
        let im = self.small.render(&txt, true, if on { p.text } else { p.text_faint });
        s.blr(&im, im.get_rect().center_at(mid.center()));
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 8);
        let col = if on { p.text } else { p.text_dim };
        let mut im = self.small.render(text, true, col);
        if im.w() > rc.w - 12 {
            im = self.tiny.render(text, true, col);
        }
        if im.w() > rc.w - 8 {
            let mut kurz: Vec<char> = text.chars().collect();
            while kurz.len() > 2 && self.tiny.size(&(kurz.iter().collect::<String>() + "...")).0 > rc.w - 8 {
                kurz.pop();
            }
            im = self.tiny.render(&(kurz.iter().collect::<String>() + "..."), true, col);
        }
        s.blr(&im, im.get_rect().center_at(rc.center()));
    }
}

impl Game for MiniGolfGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, St::Setup)
    }
    fn replay_begin(&mut self, rep: &Value) -> bool {
        MiniGolfGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        MiniGolfGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        MiniGolfGame::replay_draw(self, aiming, banner);
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

    fn wants_right_click(&self) -> bool {
        true
    }

    fn wants_escape(&self) -> bool {
        self.state == St::Edit || self.test_return || (self.state == St::Setup && self.setup_tab == "maps")
    }

    fn take_replay_request(&mut self) -> Option<Value> {
        self.replay_request.take()
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let course = self.b.opt_value("minigolf", "course").and_then(|v| v.as_str().map(|s| s.to_string()));
        self.course = COURSES.iter().copied().find(|c| Some(c.to_string()) == course).unwrap_or("classic");
        self.guide = Self::truthy(self.b.opt_value("minigolf", "guide"), true);
        self.autoaim = Self::truthy(self.b.opt_value("minigolf", "autoaim"), true);
        self.pickup = Self::truthy(self.b.opt_value("minigolf", "pickup"), true);
        self.power_lock = false;
        self.lock_t = 0.0;
        let tour = match self.b.opt_value("minigolf", "tour") {
            None | Some(Value::Null) => 1,
            Some(Value::Bool(false)) => 1,
            Some(_) => self.b.opt_int_py("minigolf", "tour", 1).filter(|v| *v != 0).unwrap_or(1),
        };
        self.tour = 1.max(gen::TOUR_COURSES.min(tour));
        self.tour_par = gen::course_par(self.tour);
        self.winner = None;
        self.cw = CW;
        self.ch = CH;
        self.setup_tab = "play";
        self.maps = None;
        self.editor = None;
        self.single_map = match self.b.opt_value("minigolf", "ugc_map") {
            Some(Value::String(s)) => s,
            Some(Value::Null) | None => String::new(),
            Some(v) => v.to_string(),
        };
        self.test_return = false;
        self.tun_cd = 0.0;
        self.boost_cd = 0.0;
        self.air = 0.0;
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.build_fonts();
        self.layout();
        self.build_setup_layout();
        self.over_cache = None;
        self.best = Self::load_best();
        self.new_round();
        self.state = St::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.layout();
        self.build_setup_layout();
        self.over_cache = None;
        let tb = self.tab_bottom;
        if let Some(ml) = self.maps.as_mut() {
            ml.layout(&self.b, tb);
        }
        if let Some(ed) = self.editor.as_mut() {
            ed.layout(&self.b);
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if matches!(ev.kind, Kind::MouseDown | Kind::MouseUp) && ev.button == 3 {
            self.lock_power(ev.kind == Kind::MouseDown);
            return;
        }
        if self.state == St::Edit {
            if let Some(mut ed) = self.editor.take() {
                let act = ed.handle(&mut self.b, ev);
                self.editor = Some(ed);
                self.edit_action(act);
                return;
            }
        }
        if self.test_return && ev.kind == Kind::KeyDown && ev.key == "Escape" {
            self.back_to_editor();
            return;
        }
        if self.state == St::Setup {
            self.handle_setup(ev);
            return;
        }
        let key = ev.key.as_str();
        if ev.kind == Kind::KeyDown && (key == "g" || key == "G") {
            self.toggle_guide();
            return;
        }
        if ev.kind == Kind::KeyDown && (key == "z" || key == "Z") {
            self.toggle_autoaim();
            return;
        }
        if ev.kind == Kind::KeyDown && (key == "p" || key == "P") {
            if self.state == St::Over && self.replay.is_some() {
                self.open_replay();
            } else {
                self.toggle_pickup();
            }
            return;
        }
        if ev.kind == Kind::KeyDown && (key == "f" || key == "F") {
            self.reset_hole();
            return;
        }
        if self.state == St::HoleDone {
            if (ev.kind == Kind::KeyDown && (key == "Return" || key == "space")) || (ev.kind == Kind::MouseDown && ev.button == 1) {
                self.advance();
            }
            return;
        }
        if self.state == St::Over {
            if ev.kind == Kind::KeyDown {
                if key == "Return" || key == "space" {
                    let a = if self.next_course().is_some() { "next" } else { "again" };
                    self.over_action(a);
                } else if key == "r" || key == "R" {
                    self.over_action("again");
                } else if key == "s" || key == "S" {
                    self.over_action("setup");
                }
            } else if ev.kind == Kind::MouseDown && ev.button == 1 {
                if let Some(&(k, _)) = self.over_rects.iter().find(|(_, r)| r.hit(ev.pos)) {
                    self.over_action(k);
                }
            }
            return;
        }
        if self.state != St::Play || self.phase != "aim" {
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                let (mx, my) = self.unproject(ev.pos.0 as f64, ev.pos.1 as f64);
                if (mx - self.bx).abs() > 0.4 || (my - self.by).abs() > 0.4 {
                    self.aim = (my - self.by).atan2(mx - self.bx);
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                self.charging = true;
                if !self.power_lock {
                    self.power = 0.05;
                }
            }
            Kind::MouseUp if ev.button == 1 => {
                if self.charging {
                    self.charging = false;
                    self.strike();
                }
            }
            Kind::KeyDown => {
                let act = |a: &str| self.b.is_action(key, a, None);
                if key == "Left" || act("left") {
                    self.aim -= py::radians(2.5);
                } else if key == "Right" || act("right") {
                    self.aim += py::radians(2.5);
                } else if key == "Up" || act("up") {
                    if !self.power_lock {
                        self.power = 1f64.min(self.power + 0.05);
                    }
                } else if key == "Down" || act("down") {
                    if !self.power_lock {
                        self.power = 0.05f64.max(self.power - 0.05);
                    }
                } else if key == "r" || key == "R" {
                    self.cancel_shot();
                } else if key == "space" || key == "Return" {
                    self.strike();
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.state == St::Edit {
            if let Some(ed) = self.editor.as_mut() {
                ed.update(dt);
            }
            return;
        }
        if self.state == St::Setup && self.setup_tab == "maps" {
            if let Some(ml) = self.maps.as_mut() {
                ml.update(dt);
                return;
            }
        }
        if self.state != St::Play {
            return;
        }
        self.mill_a += dt;
        self.move_t += dt;
        if self.phase == "aim" {
            if self.power_lock {
                self.lock_t += dt;
            } else if self.charging {
                self.power = 1f64.min(self.power + dt * 0.80);
            }
        } else if self.phase == "rolling" {
            self.physics(dt);
            let sample = self.rec_sample();
            if let Some(r) = self.rec.as_mut() {
                r.tick(dt, || Some(sample.clone()));
            }
            self.shot_time += dt;
            if self.shot_time > MAX_SHOT_TIME {
                self.vx = 0.0;
                self.vy = 0.0;
                self.air = 0.0;
            }
            if self.phase == "rolling" && self.vx == 0.0 && self.vy == 0.0 {
                self.after_shot();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.power_lock && self.b.paused {
            self.power_lock = false;
        }
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == St::Edit {
            if let Some(ed) = self.editor.as_ref() {
                ed.draw(&mut s, &self.b);
                return;
            }
        }
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            return;
        }
        self.draw_course(&mut s);
        self.draw_ball(&mut s);
        if self.state == St::Play && self.phase == "aim" {
            self.draw_aim(&mut s);
        }
        self.draw_hud(&mut s);
        self.draw_card(&mut s);
        if self.state == St::HoleDone {
            self.draw_hole_done(&mut s);
        } else if self.state == St::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "minigolf",
    class_name: "MiniGolfGame",
    supports_multiplayer: true,
    modes: &[],
    make: MiniGolfGame::new,
};
