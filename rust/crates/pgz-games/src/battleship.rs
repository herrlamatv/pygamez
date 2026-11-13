//! battleship.rs
//! =============
//! Schiffe versenken - gegen die KI oder zu zweit am selben Rechner.
//!
//! Uebertragen aus `games/battleship.py`. Regeln und KI stecken in
//! [`crate::battleship_core`]. Setup (KI-Staerke, drei Regel-Schalter),
//! Aufstellen per Drag & Drop oder Tastatur, Gefecht mit Radar-Sweep,
//! Granatenflug, Spritzern, Feuer und Rauch, Uebergabe-Bildschirm im
//! 2-Spieler-Modus.

use std::collections::{HashMap, HashSet};
use std::f64::consts::{PI, TAU};

use pgz_core::i18n;
use pgz_engine::prelude::*;
use serde_json::json;

use crate::battleship_core::{self as core, cells_of, in_board, Sea, Shot, ShotAI, FLEET, N, SHIP_KEYS};

type Rgb = (i32, i32, i32);

const COL_SEA_TOP: Rgb = (22, 70, 116);
const COL_SEA_BOT: Rgb = (10, 38, 72);
const COL_RADAR_TOP: Rgb = (14, 54, 78);
const COL_RADAR_BOT: Rgb = (6, 28, 46);
const COL_GRID: Rgb = (70, 128, 176);
const COL_GRID_RADAR: Rgb = (48, 128, 120);
const COL_WAVE: Rgb = (150, 205, 240);
const COL_RING: Rgb = (90, 200, 160);
const COL_HULL: Rgb = (118, 128, 144);
const COL_HULL_DARK: Rgb = (44, 50, 62);
const COL_DECK: Rgb = (150, 160, 174);
const COL_BRIDGE: Rgb = (196, 202, 212);
const COL_TURRET: Rgb = (82, 90, 104);
const COL_SUB: Rgb = (70, 82, 96);
const COL_FLAME: Rgb = (255, 120, 40);
const COL_FLAME_IN: Rgb = (255, 220, 110);
const COL_SCORCH: Rgb = (40, 18, 14);
const COL_MISS: Rgb = (215, 234, 248);
const COL_VALID: Rgb = (80, 220, 120);
const COL_INVALID: Rgb = (240, 70, 70);
const COL_AIM: Rgb = (255, 214, 110);
const COL_AI_AIM: Rgb = (255, 96, 80);
const COL_P1: Rgb = (120, 200, 255);
const COL_P2: Rgb = (255, 150, 110);

const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const RULES: [&str; 3] = ["touch", "salvo", "extra_shot"];
const RULE_TEXT: [&str; 3] = ["bs.rule.touch", "bs.rule.salvo", "bs.rule.extra"];
const BUTTONS: [&str; 4] = ["rotate", "random", "clear", "ready"];
const LETTERS: [&str; 10] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];

const MAX_PARTICLES: usize = 420;

#[derive(Clone, Copy, PartialEq)]
enum St {
    Setup,
    Place,
    Handover,
    Play,
    Over,
}

fn rgba_of(c: impl Col, a: i32) -> Color {
    let c = c.c();
    rgba(c.r, c.g, c.b, a)
}

fn c3(c: Color) -> Rgb {
    (c.r as i32, c.g as i32, c.b as i32)
}

/// `_mix(c1, c2, f)` - eigene Mischung des Spiels (int-Abschnitt).
fn bmix(c1: Rgb, c2: Rgb, f: f64) -> Rgb {
    let f = 0f64.max(1f64.min(f));
    (
        (c1.0 as f64 + (c2.0 - c1.0) as f64 * f) as i32,
        (c1.1 as f64 + (c2.1 - c1.1) as f64 * f) as i32,
        (c1.2 as f64 + (c2.2 - c1.2) as f64 * f) as i32,
    )
}

fn smooth(x: f64) -> f64 {
    let x = 0f64.max(1f64.min(x));
    x * x * (3.0 - 2.0 * x)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Fs {
    Small,
    Tiny,
    Huge,
    Bold,
}

#[derive(Clone, Copy, PartialEq)]
enum PKind {
    Flash,
    Ring,
    Drop,
    Spark,
    Smoke,
}

#[derive(Clone, Copy)]
struct Particle {
    kind: PKind,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    g: f64,
    age: f64,
    life: f64,
    r: f64,
    r0: f64,
    r1: f64,
    size: f64,
    col: Rgb,
}

impl Particle {
    fn new(kind: PKind, x: f64, y: f64, life: f64) -> Particle {
        Particle { kind, x, y, vx: 0.0, vy: 0.0, g: 0.0, age: 0.0, life, r: 0.0, r0: 0.0, r1: 0.0, size: 0.0, col: (0, 0, 0) }
    }
}

#[derive(Clone, Copy)]
struct Held {
    idx: usize,
    horiz: bool,
    grab: i32,
    origin: Option<(i32, i32, bool)>,
    mouse: bool,
    press: (i32, i32),
    moved: bool,
}

#[derive(Clone, Copy)]
struct Flight {
    r: i32,
    c: i32,
    t: f64,
    shooter: usize,
    dur: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum After {
    Next,
    End,
    Win,
}

#[derive(Clone, Copy, PartialEq)]
enum HoKind {
    Place,
    Fire,
}

#[derive(Clone, Copy)]
struct Handover {
    to: usize,
    kind: HoKind,
    report: Option<(i64, i64)>,
    t: f64,
}

#[derive(Clone)]
struct RevealFx {
    side: usize,
    ship: core::Ship,
    t0: f64,
}

#[derive(Clone, Copy, Default)]
struct PStats {
    shots: i64,
    hits: i64,
    turns: i64,
}

#[derive(Default)]
struct Cache {
    glow: HashMap<(i32, Rgb), Surface>,
    puff: HashMap<i32, Surface>,
    ship: HashMap<(usize, i32, bool, bool), Surface>,
    base: HashMap<(i32, bool), Surface>,
    waves: HashMap<i32, Vec<Surface>>,
    tint: HashMap<(i32, Rgb, i32), Surface>,
    labels: HashMap<(i32, i32, Color), (Vec<Surface>, Vec<Surface>)>,
    banner: HashMap<(String, Fs, Color), (Surface, Surface)>,
    sweep: HashMap<i32, Surface>,
    dim: HashMap<(i32, i32), Surface>,
    curtain: HashMap<(i32, i32), Surface>,
}

/// `BattleshipGame`
pub struct BattleshipGame {
    b: GameBase,
    diff: i64,
    rules: [bool; 3],
    cache: Cache,
    small: Font,
    tiny: Font,
    huge: Font,
    bold: Font,
    label_font: Font,
    wins: [i64; 2],
    starter: usize,
    layouts: [Option<Vec<(usize, i32, i32, bool)>>; 2],
    setup_sel: i64,
    mouse: (i32, i32),
    last_draw: Option<f64>,
    state: St,
    seas: [Sea; 2],
    stats: [PStats; 2],
    turn: usize,
    shots_left: i64,
    turn_report: (i64, i64),
    place_player: usize,
    held: Option<Held>,
    place_horiz: bool,
    kcursor: (i32, i32),
    aim: [(i32, i32); 2],
    kb_mode: bool,
    flight: Option<Flight>,
    wait: f64,
    after: Option<After>,
    ai: ShotAI,
    ai_target: Option<(i32, i32)>,
    ai_timer: f64,
    ai_think: f64,
    ai_from: (f64, f64),
    ai_pos: (f64, f64),
    handover: Option<Handover>,
    reveals: Vec<RevealFx>,
    shake_t0: f64,
    shake_amp: f64,
    winner: Option<usize>,
    msg: Option<String>,
    msg_col: Color,
    msg_t: f64,
    over_t0: f64,
    particles: Vec<Particle>,
    // Layout
    hud_h: i32,
    status_h: i32,
    cb: i32,
    lab: i32,
    cs: i32,
    bx: i32,
    by: i32,
    col_x: i32,
    col_w: i32,
    sx: i32,
    sy: i32,
    status_y: i32,
    caption_y: i32,
    fleet_rects: [Rect; 2],
    btn_h: i32,
    dock_title_y: i32,
    dock_rects: Vec<Rect>,
    dock_cell: i32,
    btn_rects: Vec<(&'static str, Rect)>,
    diff_rects: Vec<Rect>,
    diff_label_y: i32,
    rules_label_y: i32,
    rule_rects: Vec<Rect>,
    start_rect: Rect,
}

impl BattleshipGame {
    /// `BattleshipGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = BattleshipGame {
            b,
            diff: 1,
            rules: [true, false, false],
            cache: Cache::default(),
            small: f.clone(),
            tiny: f.clone(),
            huge: f.clone(),
            bold: f.clone(),
            label_font: f,
            wins: [0, 0],
            starter: 0,
            layouts: [None, None],
            setup_sel: 0,
            mouse: (-1, -1),
            last_draw: None,
            state: St::Setup,
            seas: [Sea::new(), Sea::new()],
            stats: [PStats::default(); 2],
            turn: 0,
            shots_left: 0,
            turn_report: (0, 0),
            place_player: 0,
            held: None,
            place_horiz: true,
            kcursor: (4, 4),
            aim: [(4, 4), (4, 4)],
            kb_mode: false,
            flight: None,
            wait: 0.0,
            after: None,
            ai: ShotAI::new_placeholder(),
            ai_target: None,
            ai_timer: 0.0,
            ai_think: 1.0,
            ai_from: (4.5, 4.5),
            ai_pos: (4.5, 4.5),
            handover: None,
            reveals: Vec::new(),
            shake_t0: -9.0,
            shake_amp: 0.0,
            winner: None,
            msg: None,
            msg_col: rgb(0, 0, 0),
            msg_t: 0.0,
            over_t0: 0.0,
            particles: Vec::new(),
            hud_h: 40,
            status_h: 0,
            cb: 10,
            lab: 12,
            cs: 8,
            bx: 0,
            by: 0,
            col_x: 0,
            col_w: 0,
            sx: 0,
            sy: 0,
            status_y: 0,
            caption_y: 0,
            fleet_rects: [z, z],
            btn_h: 22,
            dock_title_y: 0,
            dock_rects: Vec::new(),
            dock_cell: 6,
            btn_rects: Vec::new(),
            diff_rects: Vec::new(),
            diff_label_y: 0,
            rules_label_y: 0,
            rule_rects: Vec::new(),
            start_rect: z,
        };
        g.reset();
        Box::new(g)
    }

    fn name_upper() -> String {
        let n = if i18n::get_language() == "de" { "Schiffe versenken" } else { "Battleship" };
        n.to_uppercase()
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::font(13.max(22.min(h / 30)), false, false);
        self.tiny = ui::font(11.max(18.min(h / 38)), false, false);
        self.huge = ui::font(26.max(h / 11), true, false);
        self.bold = ui::font(15.max(30.min(h / 24)), true, false);
    }

    fn font_of(&self, f: Fs) -> &Font {
        match f {
            Fs::Small => &self.small,
            Fs::Tiny => &self.tiny,
            Fs::Huge => &self.huge,
            Fs::Bold => &self.bold,
        }
    }

    fn rule(&self, key: &str) -> bool {
        self.rules[RULES.iter().position(|r| *r == key).unwrap_or(0)]
    }

    fn new_round(&mut self) {
        self.seas = [Sea::new(), Sea::new()];
        self.stats = [PStats::default(); 2];
        self.turn = self.starter;
        self.shots_left = 0;
        self.turn_report = (0, 0);
        self.place_player = 0;
        self.held = None;
        self.place_horiz = true;
        self.kcursor = (4, 4);
        self.aim = [(4, 4), (4, 4)];
        self.kb_mode = false;
        self.flight = None;
        self.wait = 0.0;
        self.after = None;
        self.ai = ShotAI::new(self.diff, self.rule("touch"));
        self.ai_target = None;
        self.ai_timer = 0.0;
        self.ai_think = 1.0;
        self.ai_from = (4.5, 4.5);
        self.ai_pos = (4.5, 4.5);
        self.handover = None;
        self.reveals.clear();
        self.shake_t0 = -9.0;
        self.shake_amp = 0.0;
        self.winner = None;
        self.msg = None;
        self.msg_col = ui::pal().text;
        self.msg_t = 0.0;
        self.over_t0 = 0.0;
        self.particles.clear();
        self.layout();
    }

    fn now() -> f64 {
        pg::ticks() as f64 / 1000.0
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 40.max((h as f64 * 0.085) as i32);
        let m = 8.max(w / 64);
        let gap = 12.max(w / 48);
        self.status_h = self.small.get_height() + 12;
        let top = self.hud_h + m;
        let mut cb = ((h - top - self.status_h - m) as f64 / 10.6) as i32;
        cb = cb.min(((w - 2 * m - gap) as f64 * 0.63 / 10.6) as i32);
        self.cb = 10.max(cb);
        self.lab = 12.max((self.cb as f64 * 0.6) as i32);
        let colw = w - 2 * m - gap - (self.lab + 10 * self.cb);
        self.cs = 8.max(((self.cb as f64 * 0.62).min(colw as f64 / 10.0)) as i32);
        let total = self.lab + 10 * self.cb + gap + (10 * self.cs).max(colw);
        let x0 = m.max(py::fdiv32(w - total, 2));
        self.bx = x0 + self.lab;
        self.by = top + self.lab;
        self.col_x = self.bx + 10 * self.cb + gap;
        self.col_w = w - m - self.col_x;
        self.sx = self.col_x + py::fdiv32(self.col_w - 10 * self.cs, 2);
        self.sy = self.by;
        let board_bottom = self.by + 10 * self.cb;
        self.status_y = board_bottom + self.status_h / 2 + 2;
        self.label_font = ui::font(10.max(24.min((self.cb as f64 * 0.42) as i32)), true, false);
        let cap_y = self.sy + 10 * self.cs + 4;
        self.caption_y = cap_y + self.tiny.get_height() / 2;
        let fy = cap_y + self.tiny.get_height() + 6.max(h / 80);
        let fh = 40.max((board_bottom + self.status_h - 6) - fy);
        let half = py::fdiv32(self.col_w - 10, 2);
        self.fleet_rects = [Rect::new(self.col_x, fy, half, fh), Rect::new(self.col_x + half + 10, fy, half, fh)];
        let title_h = self.small.get_height() + 8;
        self.btn_h = 22.max(40.min((h as f64 * 0.058) as i32));
        let bgap = 4.max(self.btn_h / 6);
        let ready_h = self.btn_h + 4.max(self.btn_h / 4);
        let buttons_h = 3 * self.btn_h + 3 * bgap + ready_h;
        let dock_top = self.by - self.lab + title_h;
        let dock_bottom = board_bottom - buttons_h - bgap;
        let row_h = 12.max(py::fdiv32(dock_bottom - dock_top, FLEET.len() as i32));
        self.dock_title_y = self.by - self.lab + title_h / 2;
        self.dock_rects = (0..FLEET.len() as i32).map(|i| Rect::new(self.col_x, dock_top + i * row_h, self.col_w, row_h)).collect();
        self.dock_cell = 6.max(((self.cb as f64 * 0.7).min(row_h as f64 * 0.62).min((self.col_w - 30) as f64 / 9.5)) as i32);
        let mut y = dock_top + FLEET.len() as i32 * row_h + bgap;
        self.btn_rects.clear();
        for key in ["rotate", "random", "clear"] {
            self.btn_rects.push((key, Rect::new(self.col_x, y, self.col_w, self.btn_h)));
            y += self.btn_h + bgap;
        }
        self.btn_rects.push(("ready", Rect::new(self.col_x, y, self.col_w, ready_h)));
    }

    fn viewer(&self) -> usize {
        if !self.b.multiplayer {
            return 0;
        }
        if self.state == St::Over {
            if let Some(w) = self.winner {
                return w;
            }
        }
        self.turn
    }

    fn board_geom(&self, side: usize) -> (i32, i32, i32) {
        let v = self.viewer();
        if self.state == St::Place {
            return (self.bx, self.by, self.cb);
        }
        if side == v {
            return (self.sx, self.sy, self.cs);
        }
        (self.bx, self.by, self.cb)
    }

    fn cell_at(pos: (i32, i32), x: i32, y: i32, cell: i32) -> Option<(i32, i32)> {
        let c = py::fdiv32(pos.0 - x, cell);
        let r = py::fdiv32(pos.1 - y, cell);
        if (0..N).contains(&r) && (0..N).contains(&c) {
            return Some((r, c));
        }
        None
    }

    // ===================================================== Setup-Screen

    fn setup_items(&self) -> Vec<&'static str> {
        let mut v = if self.b.multiplayer { Vec::new() } else { vec!["diff"] };
        v.extend(RULES);
        v.push("start");
        v
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let bw = (360.max((w as f64 * 0.7) as i32)).min(w - 40);
        let th = self.tiny.get_height();
        let bh = 26.max(46.min((h as f64 * 0.075) as i32));
        let gap = 5.max(bh / 6);
        let lab = th + 6;
        let block = (if self.b.multiplayer { 0 } else { lab + bh + 2 * gap }) + lab + 3 * bh + 2 * gap + 3 * gap + bh + 6;
        let top = (h as f64 * 0.25) as i32;
        let bottom = h - 26;
        let mut y = top + 0.max(py::fdiv32(bottom - top - block, 2));
        y = y.min(top + (h as f64 * 0.05) as i32);
        self.diff_rects.clear();
        if !self.b.multiplayer {
            y += lab;
            let cw = (bw - 2 * gap) as f64 / 3.0;
            self.diff_rects = (0..3).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap as f64)) as i32, y, cw as i32, bh)).collect();
            self.diff_label_y = y - 4;
            y += bh + 2 * gap;
        }
        y += lab;
        self.rules_label_y = y - 4;
        self.rule_rects = (0..3).map(|i| Rect::new(cx - bw / 2, y + i * (bh + gap), bw, bh)).collect();
        y += 3 * bh + 2 * gap + 3 * gap;
        let sw = 190.max((bw as f64 * 0.42) as i32);
        self.start_rect = Rect::new(cx - sw / 2, y, sw, bh + 6);
    }

    fn save_setting(&mut self, key: &str, value: serde_json::Value) {
        self.b.set_opt("battleship", key, value);
    }

    fn set_diff(&mut self, d: i64) {
        self.diff = py::pmod(d, 3);
        let v = self.diff;
        self.save_setting("difficulty", json!(v));
        self.b.play_sound("click");
    }

    fn toggle_rule(&mut self, key: &str) {
        let i = RULES.iter().position(|r| *r == key).unwrap_or(0);
        self.rules[i] = !self.rules[i];
        let v = self.rules[i];
        self.save_setting(key, json!(v));
        self.b.play_sound("select");
    }

    fn setup_item_rects(&self, item: &str) -> Vec<Rect> {
        if item == "diff" {
            return self.diff_rects.clone();
        }
        if let Some(i) = RULES.iter().position(|r| *r == item) {
            return vec![self.rule_rects[i]];
        }
        vec![self.start_rect]
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let items = self.setup_items();
        let n = items.len() as i64;
        self.setup_sel = 0.max((n - 1).min(self.setup_sel));
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let cur = items[self.setup_sel as usize];
                let is = |a: &str| self.b.is_action(k, a, None);
                if ["1", "2", "3"].contains(&k) && !self.b.multiplayer {
                    self.set_diff(k.parse::<i64>().unwrap_or(1) - 1);
                } else if k == "Up" || is("up") {
                    self.setup_sel = py::pmod(self.setup_sel - 1, n);
                    self.b.play_sound("move");
                } else if k == "Down" || is("down") {
                    self.setup_sel = py::pmod(self.setup_sel + 1, n);
                    self.b.play_sound("move");
                } else if k == "Left" || k == "Right" || is("left") || is("right") {
                    let step = if k == "Left" || is("left") { -1 } else { 1 };
                    if cur == "diff" {
                        self.set_diff(self.diff + step);
                    } else if RULES.contains(&cur) {
                        self.toggle_rule(cur);
                    }
                } else if k == "Return" || k == "KP_Enter" {
                    self.start_round();
                } else if k == "space" {
                    if cur == "diff" {
                        self.set_diff(self.diff + 1);
                    } else if RULES.contains(&cur) {
                        self.toggle_rule(cur);
                    } else {
                        self.start_round();
                    }
                }
            }
            Kind::MouseMove => {
                for (i, it) in items.iter().enumerate() {
                    if self.setup_item_rects(it).iter().any(|r| r.hit(ev.pos)) {
                        self.setup_sel = i as i64;
                    }
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                    self.setup_sel = 0;
                    self.set_diff(i as i64);
                    return;
                }
                if let Some(i) = self.rule_rects.iter().position(|r| r.hit(ev.pos)) {
                    self.setup_sel = items.iter().position(|x| *x == RULES[i]).unwrap_or(0) as i64;
                    self.toggle_rule(RULES[i]);
                    return;
                }
                if self.start_rect.hit(ev.pos) {
                    self.start_round();
                }
            }
            _ => {}
        }
    }

    // ===================================================== Rundenablauf

    fn start_round(&mut self) {
        self.b.game_over = false;
        self.new_round();
        for p in 0..2 {
            if self.b.multiplayer || p == 0 {
                self.restore_layout(p);
            }
        }
        self.place_player = 0;
        if self.b.multiplayer {
            self.handover_to(0, HoKind::Place, None);
        } else {
            self.state = St::Place;
        }
        self.b.play_sound("click");
    }

    fn restore_layout(&mut self, p: usize) {
        let touch = self.rule("touch");
        if let Some(lay) = self.layouts[p].clone() {
            for (idx, r, c, horiz) in lay {
                if self.seas[p].can_place(idx, r, c, horiz, touch) {
                    self.seas[p].place(idx, r, c, horiz);
                }
            }
        }
    }

    fn restart(&mut self) {
        self.starter = 1 - self.starter;
        self.start_round();
    }

    fn handover_to(&mut self, player: usize, kind: HoKind, report: Option<(i64, i64)>) {
        self.handover = Some(Handover { to: player, kind, report, t: 0.0 });
        self.held = None;
        self.flight = None;
        self.state = St::Handover;
    }

    fn confirm_handover(&mut self) {
        let Some(ho) = self.handover else {
            return;
        };
        if ho.t < 0.45 {
            return;
        }
        self.handover = None;
        self.b.play_sound("select");
        if ho.kind == HoKind::Place {
            self.place_player = ho.to;
            self.state = St::Place;
        } else {
            self.begin_turn(ho.to);
        }
    }

    fn ready(&mut self) {
        if self.held.is_some() {
            self.drop_held();
        }
        if !self.seas[self.place_player].complete() {
            self.say(t("bs.place_all"), ui::pal().gold, 1.6);
            self.b.play_sound("hit");
            return;
        }
        let pp = self.place_player;
        self.layouts[pp] = Some(self.seas[pp].ships.iter().map(|s| (s.idx, s.r, s.c, s.horiz)).collect());
        self.b.play_sound("level");
        if !self.b.multiplayer {
            let touch = self.rule("touch");
            self.seas[1].randomize(touch);
            self.begin_turn(self.starter);
        } else if self.place_player == 0 {
            self.handover_to(1, HoKind::Place, None);
        } else {
            self.handover_to(self.starter, HoKind::Fire, None);
        }
    }

    fn is_ai(&self, p: usize) -> bool {
        !self.b.multiplayer && p == 1
    }

    fn begin_turn(&mut self, p: usize) {
        self.turn = p;
        self.state = St::Play;
        self.shots_left = if self.rule("salvo") { self.seas[p].ships_left() } else { 1 };
        self.stats[p].turns += 1;
        self.turn_report = (0, 0);
        self.flight = None;
        self.after = None;
        self.wait = 0.0;
        self.ai_target = None;
        if !self.is_ai(p) {
            audio::tone(1320.0, 0.07, Some(&self.b.settings), "sine", 0.18);
        }
    }

    fn say(&mut self, text: String, color: Color, secs: f64) {
        self.msg = if text.is_empty() { None } else { Some(text) };
        self.msg_col = color;
        self.msg_t = secs;
    }

    fn fire(&mut self, r: i32, c: i32) -> bool {
        if self.state != St::Play || self.flight.is_some() {
            return false;
        }
        if !(self.after.is_none() || self.after == Some(After::Next)) || self.shots_left <= 0 {
            return false;
        }
        let target = &self.seas[1 - self.turn];
        if target.shots[r as usize][c as usize] != core::UNKNOWN {
            if !self.is_ai(self.turn) {
                self.say(t("bs.already"), ui::pal().text_dim, 1.2);
                self.b.play_sound("click");
            }
            return false;
        }
        self.after = None;
        self.wait = 0.0;
        self.shots_left -= 1;
        self.flight = Some(Flight { r, c, t: 0.0, shooter: self.turn, dur: if self.is_ai(self.turn) { 0.46 } else { 0.36 } });
        self.b.play_sound("shoot");
        true
    }

    fn impact(&mut self) {
        let Some(f) = self.flight.take() else {
            return;
        };
        let shooter = f.shooter;
        let side = 1 - shooter;
        let (res, si) = self.seas[side].fire(f.r, f.c);
        let Some(res) = res else {
            self.after = Some(After::Next);
            self.wait = 0.1;
            return;
        };
        self.stats[shooter].shots += 1;
        self.turn_report.0 += 1;
        let (x, y, cell) = self.board_geom(side);
        let px = (x + f.c * cell) as f64 + cell as f64 / 2.0;
        let py_ = (y + f.r * cell) as f64 + cell as f64 / 2.0;
        let mine = side == self.viewer();
        let pause: f64;
        if res == Shot::Miss {
            self.fx_splash(px, py_, cell);
            self.sfx_splash();
            self.say(t("bs.miss"), COL_MISS.c(), 1.0);
            pause = 0.55;
        } else {
            self.stats[shooter].hits += 1;
            self.turn_report.1 += 1;
            self.fx_explosion(px, py_, cell, res == Shot::Sunk, false);
            self.b.play_sound("explode");
            self.shake_t0 = Self::now();
            self.shake_amp = 2f64.max(cell as f64 * if res == Shot::Sunk { 0.16 } else { 0.09 });
            if mine {
                self.b.rumble(if res == Shot::Sunk { 260 } else { 140 });
            }
            if self.rule("extra_shot") {
                self.shots_left += 1;
            }
            if res == Shot::Sunk {
                let ship = self.seas[side].ships[si.unwrap_or(0)].clone();
                self.reveals.push(RevealFx { side, ship: ship.clone(), t0: Self::now() });
                for (rr, cc) in ship.cells() {
                    self.fx_explosion((x + cc * cell) as f64 + cell as f64 / 2.0, (y + rr * cell) as f64 + cell as f64 / 2.0, cell, false, true);
                }
                self.b.play_sound("line");
                let name = t(&format!("bs.ship.{}", SHIP_KEYS[ship.idx]));
                let col = if mine { COL_P2.c() } else { ui::pal().gold };
                self.say(tf("bs.sunk", &[("ship", &name)]), col, 2.2);
                pause = 1.15;
            } else {
                let key = if self.rule("extra_shot") { "bs.hit_again" } else { "bs.hit" };
                self.say(t(key), COL_FLAME.c(), 1.3);
                pause = 0.7;
            }
        }
        let target = &self.seas[side];
        if target.all_sunk() {
            self.after = Some(After::Win);
            self.wait = 1.7;
        } else if self.shots_left <= 0 || target.untouched() == 0 {
            self.after = Some(After::End);
            self.wait = pause.max(0.95);
        } else {
            self.after = Some(After::Next);
            self.wait = pause;
        }
    }

    fn continue_(&mut self) {
        let after = self.after.take();
        match after {
            Some(After::Win) => self.finish(self.turn),
            Some(After::End) => {
                let nxt = 1 - self.turn;
                if self.b.multiplayer {
                    let rep = self.turn_report;
                    self.handover_to(nxt, HoKind::Fire, Some(rep));
                } else {
                    self.begin_turn(nxt);
                }
            }
            _ => {}
        }
    }

    fn finish(&mut self, winner: usize) {
        self.winner = Some(winner);
        self.state = St::Over;
        self.over_t0 = Self::now();
        self.wins[winner] += 1;
        if !self.b.multiplayer {
            if winner == 0 {
                self.b.score = self.wins[0];
                self.b.play_sound("win");
                self.b.report_result(true);
                if !self.seas[0].ships.iter().any(|s| s.sunk()) {
                    self.b.ach_event("bs_flawless", None);
                }
                if self.diff == core::HARD {
                    self.b.ach_event("bs_hard", None);
                }
            } else {
                self.b.play_sound("gameover");
                self.b.report_result(false);
            }
        } else {
            self.b.report_duel(winner);
            self.b.play_sound("win");
        }
        self.b.game_over = true;
    }

    fn ai_step(&mut self, dt: f64) {
        if self.ai_target.is_none() {
            let pick = self.ai.choose(&self.seas[0]);
            let Some(pick) = pick else {
                self.after = Some(After::End);
                self.wait = 0.2;
                return;
            };
            self.ai_target = Some(pick);
            let first = self.turn_report.0 == 0;
            self.ai_think = (if first { 0.8 } else { 0.42 }) + py::uniform(0.0, 0.25);
            self.ai_timer = self.ai_think;
            self.ai_from = self.ai_pos;
        }
        self.ai_timer -= dt;
        let k = smooth(1.0 - 0f64.max(self.ai_timer) / self.ai_think);
        let (r, c) = self.ai_target.unwrap_or((0, 0));
        self.ai_pos = (self.ai_from.0 + (c as f64 + 0.5 - self.ai_from.0) * k, self.ai_from.1 + (r as f64 + 0.5 - self.ai_from.1) * k);
        if self.ai_timer <= 0.0 {
            self.ai_pos = (c as f64 + 0.5, r as f64 + 0.5);
            self.ai_target = None;
            self.fire(r, c);
        }
    }

    // ===================================================== Eingabe

    fn handle_play(&mut self, ev: &InputEvent) {
        if self.is_ai(self.turn) {
            return;
        }
        let tn = self.turn;
        match ev.kind {
            Kind::MouseMove => {
                if let Some(rc) = Self::cell_at(ev.pos, self.bx, self.by, self.cb) {
                    self.aim[tn] = rc;
                    self.kb_mode = false;
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                if let Some(rc) = Self::cell_at(ev.pos, self.bx, self.by, self.cb) {
                    self.aim[tn] = rc;
                    self.kb_mode = false;
                    self.fire(rc.0, rc.1);
                }
            }
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let is = |a: &str| self.b.is_action(k, a, None);
                let mut aim = self.aim[tn];
                let mut moved = true;
                if k == "Up" || is("up") {
                    aim.0 = py::pmod32(aim.0 - 1, N);
                } else if k == "Down" || is("down") {
                    aim.0 = py::pmod32(aim.0 + 1, N);
                } else if k == "Left" || is("left") {
                    aim.1 = py::pmod32(aim.1 - 1, N);
                } else if k == "Right" || is("right") {
                    aim.1 = py::pmod32(aim.1 + 1, N);
                } else {
                    moved = false;
                }
                let action = ["space", "Return", "KP_Enter"].contains(&k) || is("action");
                self.aim[tn] = aim;
                if moved {
                    self.kb_mode = true;
                    self.b.play_sound("move");
                } else if action {
                    self.kb_mode = true;
                    self.fire(aim.0, aim.1);
                }
            }
            _ => {}
        }
    }

    fn place_sea(&mut self) -> &mut Sea {
        &mut self.seas[self.place_player]
    }

    fn pick(&mut self, idx: usize, grab: i32, via_mouse: bool) {
        let pp = self.place_player;
        let mut origin = None;
        let mut horiz = self.place_horiz;
        if let Some(si) = self.seas[pp].ship_by_idx(idx) {
            let s = &self.seas[pp].ships[si];
            origin = Some((s.r, s.c, s.horiz));
            horiz = s.horiz;
            self.seas[pp].remove_idx(idx);
        }
        self.held = Some(Held { idx, horiz, grab: 0.max((FLEET[idx] - 1).min(grab)), origin, mouse: via_mouse, press: self.mouse, moved: false });
        self.b.play_sound("select");
    }

    fn held_anchor(&self) -> Option<(i32, i32)> {
        let hd = self.held?;
        let size = FLEET[hd.idx];
        let (mut r, mut c) = if hd.mouse { Self::cell_at(self.mouse, self.bx, self.by, self.cb)? } else { self.kcursor };
        if hd.horiz {
            c -= hd.grab;
        } else {
            r -= hd.grab;
        }
        if !hd.mouse {
            if hd.horiz {
                c = 0.max((N - size).min(c));
            } else {
                r = 0.max((N - size).min(r));
            }
        }
        Some((r, c))
    }

    fn held_valid(&self) -> bool {
        let Some(a) = self.held_anchor() else {
            return false;
        };
        let hd = self.held.expect("held");
        self.seas[self.place_player].can_place(hd.idx, a.0, a.1, hd.horiz, self.rule("touch"))
    }

    fn put_held(&mut self) -> bool {
        let Some(a) = self.held_anchor() else {
            return false;
        };
        let hd = self.held.expect("held");
        let touch = self.rule("touch");
        if !self.seas[self.place_player].can_place(hd.idx, a.0, a.1, hd.horiz, touch) {
            self.say(t("bs.place_invalid"), COL_INVALID.c(), 1.2);
            self.b.play_sound("hit");
            return false;
        }
        self.place_sea().place(hd.idx, a.0, a.1, hd.horiz);
        self.held = None;
        self.b.play_sound("lock");
        true
    }

    fn drop_held(&mut self) {
        let hd = self.held.take();
        if let Some(hd) = hd {
            if let Some((r, c, horiz)) = hd.origin {
                let touch = self.rule("touch");
                if self.seas[self.place_player].can_place(hd.idx, r, c, horiz, touch) {
                    self.place_sea().place(hd.idx, r, c, horiz);
                }
            }
        }
    }

    fn rotate_held(&mut self) {
        if let Some(hd) = self.held.as_mut() {
            hd.horiz = !hd.horiz;
            self.place_horiz = hd.horiz;
        }
        self.b.play_sound("rotate");
    }

    fn rotate_placed(&mut self, si: usize, pivot: (i32, i32)) -> bool {
        let ship = self.seas[self.place_player].ships[si].clone();
        let k = if ship.horiz { pivot.1 - ship.c } else { pivot.0 - ship.r };
        let horiz = !ship.horiz;
        let (r, c) = if horiz { (pivot.0, pivot.1 - k) } else { (pivot.0 - k, pivot.1) };
        let mut tries = vec![(r, c)];
        for d in [-1, 1, -2, 2, -3, 3, -4, 4] {
            tries.push(if horiz { (r, c + d) } else { (r + d, c) });
        }
        let touch = self.rule("touch");
        for (rr, cc) in tries {
            if self.seas[self.place_player].can_place(ship.idx, rr, cc, horiz, touch) {
                self.place_sea().place(ship.idx, rr, cc, horiz);
                self.b.play_sound("rotate");
                return true;
            }
        }
        self.say(t("bs.place_invalid"), COL_INVALID.c(), 1.2);
        self.b.play_sound("hit");
        false
    }

    fn press_button(&mut self, key: &str) {
        match key {
            "rotate" => {
                if self.held.is_some() {
                    self.rotate_held();
                } else {
                    let rc = if self.kb_mode { Some(self.kcursor) } else { Self::cell_at(self.mouse, self.bx, self.by, self.cb) };
                    let si = rc.and_then(|rc| self.seas[self.place_player].ship_at(rc.0, rc.1));
                    match (si, rc) {
                        (Some(si), Some(rc)) => {
                            self.rotate_placed(si, rc);
                        }
                        _ => {
                            self.place_horiz = !self.place_horiz;
                            self.b.play_sound("rotate");
                        }
                    }
                }
            }
            "random" => {
                self.held = None;
                let touch = self.rule("touch");
                self.place_sea().clear();
                self.place_sea().randomize(touch);
                self.b.play_sound("merge");
            }
            "clear" => {
                self.held = None;
                self.place_sea().clear();
                self.b.play_sound("click");
            }
            "ready" => self.ready(),
            _ => {}
        }
    }

    fn next_unplaced(&self) -> Option<usize> {
        (0..FLEET.len()).find(|i| self.seas[self.place_player].ship_by_idx(*i).is_none())
    }

    fn handle_place(&mut self, ev: &InputEvent) {
        let pp = self.place_player;
        match ev.kind {
            Kind::MouseMove => {
                let on_board = Self::cell_at(ev.pos, self.bx, self.by, self.cb).is_some();
                if let Some(hd) = self.held.as_mut() {
                    if !hd.mouse && on_board {
                        hd.mouse = true;
                    }
                    if (ev.pos.0 - hd.press.0).abs() + (ev.pos.1 - hd.press.1).abs() > 6 {
                        hd.moved = true;
                    }
                }
                if on_board {
                    self.kb_mode = false;
                }
            }
            Kind::MouseDown => {
                let pos = ev.pos;
                let rc = Self::cell_at(pos, self.bx, self.by, self.cb);
                if ev.button == 3 {
                    if self.held.is_some() {
                        self.rotate_held();
                    } else if let Some(si) = rc.and_then(|rc| self.seas[pp].ship_at(rc.0, rc.1)) {
                        self.rotate_placed(si, rc.unwrap_or((0, 0)));
                    } else {
                        self.place_horiz = !self.place_horiz;
                        self.b.play_sound("rotate");
                    }
                    return;
                }
                if ev.button != 1 {
                    return;
                }
                self.kb_mode = false;
                if self.held.is_some() {
                    if rc.is_some() {
                        if let Some(hd) = self.held.as_mut() {
                            hd.mouse = true;
                        }
                        self.put_held();
                        return;
                    }
                    if let Some(&(key, _)) = self.btn_rects.iter().find(|(_, r)| r.hit(pos)) {
                        self.drop_held();
                        self.press_button(key);
                        return;
                    }
                    self.drop_held();
                    self.b.play_sound("click");
                    return;
                }
                if let Some(&(key, _)) = self.btn_rects.iter().find(|(_, r)| r.hit(pos)) {
                    self.press_button(key);
                    return;
                }
                if let Some(rc) = rc {
                    if let Some(si) = self.seas[pp].ship_at(rc.0, rc.1) {
                        let s = &self.seas[pp].ships[si];
                        let grab = if s.horiz { rc.1 - s.c } else { rc.0 - s.r };
                        let idx = s.idx;
                        self.pick(idx, grab, true);
                    }
                    return;
                }
                if let Some(idx) = self.dock_rects.iter().position(|r| r.hit(pos)) {
                    let drc = self.dock_rects[idx];
                    let grab = py::fdiv32(pos.0 - (drc.x + 8), 1.max(self.dock_cell));
                    self.pick(idx, grab, true);
                    let none = self.seas[pp].ship_by_idx(idx).is_none();
                    if let Some(hd) = self.held.as_mut() {
                        hd.horiz = if none { true } else { hd.horiz };
                    }
                }
            }
            Kind::MouseUp => {
                let Some(hd) = self.held else {
                    return;
                };
                if !hd.moved || !hd.mouse {
                    return;
                }
                let on_board = Self::cell_at(ev.pos, self.bx, self.by, self.cb).is_some();
                if on_board && self.held_valid() {
                    self.put_held();
                } else {
                    if on_board {
                        self.say(t("bs.place_invalid"), COL_INVALID.c(), 1.2);
                        self.b.play_sound("hit");
                    }
                    self.drop_held();
                }
            }
            Kind::KeyDown => self.place_key(ev.key.as_str()),
            _ => {}
        }
    }

    fn place_key(&mut self, k: &str) {
        let pp = self.place_player;
        let is = |a: &str| self.b.is_action(k, a, None);
        let (mut dr, mut dc) = (0, 0);
        if k == "Up" || is("up") {
            dr = -1;
        } else if k == "Down" || is("down") {
            dr = 1;
        } else if k == "Left" || is("left") {
            dc = -1;
        } else if k == "Right" || is("right") {
            dc = 1;
        }
        if dr != 0 || dc != 0 {
            if !self.kb_mode && self.held.is_none() {
                if let Some(rc) = Self::cell_at(self.mouse, self.bx, self.by, self.cb) {
                    self.kcursor = rc;
                }
            }
            self.kb_mode = true;
            if self.held.is_some_and(|h| h.mouse) {
                if let Some(hd) = self.held.as_mut() {
                    hd.mouse = false;
                }
                if let Some(rc) = Self::cell_at(self.mouse, self.bx, self.by, self.cb) {
                    self.kcursor = rc;
                }
            }
            self.kcursor = (py::pmod32(self.kcursor.0 + dr, N), py::pmod32(self.kcursor.1 + dc, N));
            self.b.play_sound("move");
            return;
        }
        if (k == "r" || k == "R") && self.b.key_is_free(k) {
            self.press_button("rotate");
            return;
        }
        if (k == "x" || k == "X") && self.b.key_is_free(k) {
            self.press_button("random");
            return;
        }
        if ["c", "C", "Delete", "BackSpace"].contains(&k) && (k.chars().count() > 1 || self.b.key_is_free(k)) {
            self.press_button("clear");
            return;
        }
        let is_enter = k == "Return" || k == "KP_Enter";
        if k == "space" || is_enter || self.b.is_action(k, "action", None) {
            self.kb_mode = true;
            if self.held.is_some() {
                if let Some(hd) = self.held.as_mut() {
                    hd.mouse = false;
                }
                self.put_held();
                return;
            }
            let (kr, kc) = self.kcursor;
            let si = self.seas[pp].ship_at(kr, kc);
            if is_enter && self.seas[pp].complete() {
                self.ready();
            } else if si.is_some() && !is_enter {
                let s = &self.seas[pp].ships[si.unwrap_or(0)];
                let grab = if s.horiz { kc - s.c } else { kr - s.r };
                let idx = s.idx;
                self.pick(idx, grab, false);
            } else if let Some(idx) = self.next_unplaced() {
                self.pick(idx, 0, false);
            } else if let Some(si) = si {
                let s = &self.seas[pp].ships[si];
                let grab = if s.horiz { kc - s.c } else { kr - s.r };
                let idx = s.idx;
                self.pick(idx, grab, false);
            }
        }
    }

    // ===================================================== Effekte

    fn spawn(&mut self, p: Particle) {
        if self.particles.len() < MAX_PARTICLES {
            self.particles.push(p);
        }
    }

    fn fx_splash(&mut self, x: f64, y: f64, cell: i32) {
        let cell = cell as f64;
        let k = cell / 40.0;
        self.spawn(Particle { r: cell * 0.45, col: (210, 235, 255), ..Particle::new(PKind::Flash, x, y, 0.16) });
        self.spawn(Particle { r0: cell * 0.12, r1: cell * 0.62, ..Particle::new(PKind::Ring, x, y, 0.7) });
        self.spawn(Particle { r0: cell * 0.05, r1: cell * 0.9, ..Particle::new(PKind::Ring, x, y, 0.95) });
        for _ in 0..16 {
            let ang = py::uniform(PI * 1.05, PI * 1.95);
            let sp = py::uniform(70.0, 230.0) * k;
            let life = py::uniform(0.45, 0.8);
            let size = py::uniform(1.2, 2.8) * 1f64.max(k);
            self.spawn(Particle { vx: ang.cos() * sp, vy: ang.sin() * sp * 1.4, g: 620.0 * k, size, ..Particle::new(PKind::Drop, x, y, life) });
        }
    }

    fn fx_explosion(&mut self, x: f64, y: f64, cell: i32, big: bool, quiet: bool) {
        let cellf = cell as f64;
        let k = cellf / 40.0;
        let n = if quiet { 10 } else if big { 34 } else { 22 };
        self.spawn(Particle { r: cellf * if big { 1.1 } else { 0.8 }, col: (255, 190, 90), ..Particle::new(PKind::Flash, x, y, if !quiet { 0.22 } else { 0.16 }) });
        let cols = [(255, 230, 120), (255, 160, 60), (255, 110, 40)];
        for _ in 0..n {
            let ang = py::uniform(0.0, TAU);
            let sp = py::uniform(80.0, if big { 360.0 } else { 280.0 }) * k;
            let life = py::uniform(0.3, 0.7);
            let size = py::uniform(1.5, 3.2) * 1f64.max(k);
            let col = py::choice(&cols);
            self.spawn(Particle { vx: ang.cos() * sp, vy: ang.sin() * sp - 60.0 * k, g: 520.0 * k, size, col, ..Particle::new(PKind::Spark, x, y, life) });
        }
        for _ in 0..(if quiet { 3 } else { 6 }) {
            let px = x + py::uniform(-0.2, 0.2) * cellf;
            let py_ = y + py::uniform(-0.1, 0.2) * cellf;
            let vx = py::uniform(-12.0, 12.0) * k;
            let vy = py::uniform(-40.0, -18.0) * k;
            let life = py::uniform(0.9, 1.6);
            self.spawn(Particle { vx, vy, r0: cellf * 0.16, r1: cellf * 0.55, ..Particle::new(PKind::Smoke, px, py_, life) });
        }
    }

    fn sfx_splash(&self) {
        audio::tone(180.0, 0.22, Some(&self.b.settings), "noise", 0.32);
        self.b.play_sound("bounce");
    }

    fn glow(&mut self, radius: f64, color: Rgb) -> Surface {
        let radius = 4.max((radius as i32) / 3 * 3);
        if let Some(s) = self.cache.glow.get(&(radius, color)) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(radius * 2, radius * 2);
        let steps = 10;
        for i in 0..steps {
            let f = i as f64 / steps as f64;
            draw::circle(&mut surf, rgba_of(color, (210.0 * f * f) as i32), (radius, radius), 1.max((radius as f64 * (1.0 - f)) as i32), 0);
        }
        self.cache.glow.insert((radius, color), surf.clone());
        surf
    }

    fn puff(&mut self, radius: f64) -> Surface {
        let radius = 3.max((radius as i32) / 2 * 2);
        if let Some(s) = self.cache.puff.get(&radius) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(radius * 2, radius * 2);
        let steps = 6;
        for i in 0..steps {
            let f = i as f64 / steps as f64;
            draw::circle(&mut surf, rgba(70, 72, 78, (40.0 + 120.0 * f) as i32), (radius, radius), 1.max((radius as f64 * (1.0 - f * 0.8)) as i32), 0);
        }
        self.cache.puff.insert(radius, surf.clone());
        surf
    }

    fn step_particles(&mut self, s: &mut Surface, dt: f64) {
        let parts = std::mem::take(&mut self.particles);
        let mut alive = Vec::new();
        for mut p in parts {
            p.age += dt;
            if p.age >= p.life {
                continue;
            }
            let f = p.age / p.life;
            p.vy += p.g * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            let (x, y) = (p.x as i32, p.y as i32);
            match p.kind {
                PKind::Spark => {
                    let sz = 1.max((p.size * (1.0 - f * 0.6)) as i32);
                    s.fill_r(bmix(p.col, (120, 30, 10), f), Rect::new(x, y, sz, sz));
                }
                PKind::Drop => {
                    draw::circle(s, bmix((240, 250, 255), COL_SEA_TOP, f * 0.9), (x, y), 1.max(p.size as i32), 0);
                }
                PKind::Ring => {
                    let rr = p.r0 + (p.r1 - p.r0) * f.sqrt();
                    draw::circle(s, bmix((220, 240, 255), COL_SEA_TOP, f), (x, y), 2.max(rr as i32), 1.max((3.0 * (1.0 - f)) as i32 + 1));
                }
                PKind::Smoke => {
                    let rr = p.r0 + (p.r1 - p.r0) * f;
                    let mut img = self.puff(rr);
                    img.set_alpha(Some((170.0 * (1.0 - f)) as i32 as u8));
                    s.bl(&img, (x - img.w() / 2, y - img.h() / 2));
                    img.set_alpha(Some(255));
                }
                PKind::Flash => {
                    let mut img = self.glow(p.r * (0.6 + 0.6 * f), p.col);
                    img.set_alpha(Some((255.0 * (1.0 - f)) as i32 as u8));
                    s.bl(&img, (x - img.w() / 2, y - img.h() / 2));
                    img.set_alpha(Some(255));
                }
            }
            alive.push(p);
        }
        self.particles = alive;
    }

    // ===================================================== Sprites

    fn ship_sprite(&mut self, idx: usize, cell: i32, horiz: bool, sunk: bool) -> Surface {
        let key = (idx, cell, horiz, sunk);
        if let Some(s) = self.cache.ship.get(&key) {
            return s.clone();
        }
        let mut surf = if sunk {
            let mut base = self.ship_sprite(idx, cell, true, false).copy();
            base.fill_f(rgba(150, 96, 84, 255), BLEND_RGBA_MULT);
            let size = FLEET[idx];
            for i in 0..size {
                let cx = ((i as f64 + 0.5) * cell as f64) as i32;
                let off = (cell as f64 * 0.1 * ((i * 7) % 3 - 1) as f64) as i32;
                draw::circle(&mut base, rgba(24, 12, 10, 150), (cx + off, cell / 2), 2.max((cell as f64 * 0.2) as i32), 0);
            }
            base
        } else {
            Self::render_ship(idx, cell)
        };
        if !horiz {
            surf = transform::rotate(&surf, -90.0);
        }
        self.cache.ship.insert(key, surf.clone());
        surf
    }

    fn render_ship(idx: usize, cell: i32) -> Surface {
        let size = FLEET[idx];
        let (l, h) = (size * cell, cell);
        let mut surf = pg::surface_alpha(l, h);
        let p = 2.max((cell as f64 * 0.14) as i32);
        let (top, bot) = (p, h - p);
        let mid = h as f64 / 2.0;
        let hull_h = bot - top;
        let (x0, x1) = (p, l - 1.max(p / 2));
        let lw = 1.max(cell / 18);
        if SHIP_KEYS[idx] == "submarine" {
            let body = Rect::new(x0, (top as f64 + hull_h as f64 * 0.14) as i32, x1 - x0, 3.max((hull_h as f64 * 0.72) as i32));
            draw::ellipse(&mut surf, COL_SUB, body, 0);
            let mut hi = body.inflate((-(body.w as f64) * 0.16) as i32, (-(body.h as f64) * 0.55) as i32);
            hi.y = body.y + 1.max(body.h / 6);
            draw::ellipse(&mut surf, bmix(COL_SUB, (255, 255, 255), 0.22), hi, 0);
            draw::ellipse(&mut surf, COL_HULL_DARK, body, lw);
            let tw = 3.max((cell as f64 * 0.62) as i32);
            let th = 3.max((hull_h as f64 * 0.5) as i32);
            let tower = Rect::new((l as f64 * 0.4 - tw as f64 / 2.0) as i32, (mid - th as f64 / 2.0) as i32, tw, th);
            draw::rect_r(&mut surf, COL_DECK, tower, 0, 1.max(th / 2));
            draw::rect_r(&mut surf, COL_HULL_DARK, tower, lw, 1.max(th / 2));
            let ex = (x1 - p).min(tower.right() + (cell as f64 * 0.35) as i32);
            draw::line(&mut surf, COL_HULL_DARK, (tower.centerx(), mid as i32), (ex, mid as i32), lw);
            return surf;
        }
        let (x0f, x1f, topf, botf, pf) = (x0 as f64, x1 as f64, top as f64, bot as f64, p as f64);
        let bow = (cell as f64 * 0.95).min(l as f64 * 0.32);
        let pts = [(x0f + pf, topf), (x1f - bow, topf), (x1f, mid), (x1f - bow, botf), (x0f + pf, botf), (x0f, botf - pf * 0.7), (x0f, topf + pf * 0.7)];
        draw::polygon(&mut surf, COL_HULL, &pts, 0);
        let d = 1.max((hull_h as f64 * 0.2) as i32);
        let df = d as f64;
        let deck = [
            (x0f + pf + df * 0.5, topf + df),
            (x1f - bow + df * 0.2, topf + df),
            (x1f - df * 1.6, mid),
            (x1f - bow + df * 0.2, botf - df),
            (x0f + pf + df * 0.5, botf - df),
            (x0f + df, botf - pf * 0.7 - df * 0.3),
            (x0f + df, topf + pf * 0.7 + df * 0.3),
        ];
        draw::polygon(&mut surf, COL_DECK, &deck, 0);
        draw::polygon(&mut surf, COL_HULL_DARK, &pts, lw);
        let key = SHIP_KEYS[idx];
        if key == "carrier" {
            let dash = 2.max(cell / 4);
            let yy = mid as i32;
            let mut xx = (x0 + p + d) as i32;
            while (xx as f64) < x1f - bow {
                draw::line(&mut surf, (235, 238, 240), (xx, yy), ((xx + dash).min((x1f - bow) as i32), yy), 1.max(cell / 20));
                xx += dash * 2;
            }
            let iw = 3.max((cell as f64 * 0.7) as i32);
            let ih = 2.max((hull_h as f64 * 0.3) as i32);
            let island = Rect::new((l as f64 * 0.58) as i32, top + 1, iw, ih);
            draw::rect_r(&mut surf, COL_BRIDGE, island, 0, 1.max(ih / 3));
            draw::rect_r(&mut surf, COL_HULL_DARK, island, lw, 1.max(ih / 3));
            return surf;
        }
        let bw_ = 3.max((cell as f64 * 0.62) as i32);
        let bh_ = 3.max((hull_h as f64 * 0.52) as i32);
        let bridge = Rect::new((l as f64 * 0.44 - bw_ as f64 / 2.0) as i32, (mid - bh_ as f64 / 2.0) as i32, bw_, bh_);
        draw::rect_r(&mut surf, COL_BRIDGE, bridge, 0, 1.max(bh_ / 4));
        draw::rect_r(&mut surf, COL_HULL_DARK, bridge, lw, 1.max(bh_ / 4));
        if cell >= 20 {
            let lx = bridge.right() - 2.max(bw_ / 4);
            draw::line(&mut surf, (60, 90, 120), (lx, bridge.y + 2), (lx, bridge.bottom() - 3), 1.max(cell / 16));
        }
        let turrets: &[f64] = match key {
            "battleship" => &[0.18, 0.7, 0.84],
            "cruiser" => &[0.2, 0.76],
            "destroyer" => &[0.74],
            _ => &[],
        };
        let tr = 2.max((hull_h as f64 * 0.22) as i32);
        for fx in turrets {
            let mut tx = (l as f64 * fx) as i32;
            if (tx + tr) as f64 >= x1f - bow * 0.35 {
                tx = (x1f - bow * 0.35 - tr as f64 - 1.0) as i32;
            }
            draw::line(&mut surf, COL_HULL_DARK, (tx, mid as i32), (tx + (tr as f64 * 1.9) as i32, mid as i32), 1.max(tr / 2));
            draw::circle(&mut surf, COL_TURRET, (tx, mid as i32), tr, 0);
            draw::circle(&mut surf, COL_HULL_DARK, (tx, mid as i32), tr, 1.max(lw));
        }
        surf
    }

    fn board_base(&mut self, cell: i32, radar: bool) -> Surface {
        if let Some(s) = self.cache.base.get(&(cell, radar)) {
            return s.clone();
        }
        let size = cell * N;
        let mut surf = pg::surface(size, size);
        let (top, bot) = if radar { (COL_RADAR_TOP, COL_RADAR_BOT) } else { (COL_SEA_TOP, COL_SEA_BOT) };
        for yy in 0..size {
            draw::line(&mut surf, bmix(top, bot, yy as f64 / 1.max(size - 1) as f64), (0, yy), (size, yy), 1);
        }
        let shade = if !radar { (5, 8, 10) } else { (3, 9, 8) };
        for r in 0..N {
            for c in 0..N {
                if (r + c) % 2 != 0 {
                    surf.fill_rf(shade, Rect::new(c * cell, r * cell, cell, cell), BLEND_RGB_ADD);
                }
            }
        }
        if radar {
            let ctr = (size / 2, size / 2);
            for i in [1, 2, 3, 4] {
                draw::circle(&mut surf, bmix(COL_RADAR_TOP, COL_RING, 0.28), ctr, (size as f64 * 0.125 * i as f64) as i32, 1);
            }
            draw::line(&mut surf, bmix(COL_RADAR_TOP, COL_RING, 0.2), (ctr.0, 0), (ctr.0, size), 1);
            draw::line(&mut surf, bmix(COL_RADAR_TOP, COL_RING, 0.2), (0, ctr.1), (size, ctr.1), 1);
        }
        let grid = if radar { COL_GRID_RADAR } else { COL_GRID };
        for i in 0..=N {
            let col = bmix(bot, grid, 0.55);
            draw::line(&mut surf, col, (i * cell, 0), (i * cell, size), 1);
            draw::line(&mut surf, col, (0, i * cell), (size, i * cell), 1);
        }
        draw::rect(&mut surf, bmix(bot, grid, 0.9), Rect::new(0, 0, size, size), 1.max(cell / 20));
        self.cache.base.insert((cell, radar), surf.clone());
        surf
    }

    fn wave_glyphs(&mut self, cell: i32) -> Vec<Surface> {
        if let Some(g) = self.cache.waves.get(&cell) {
            return g.clone();
        }
        let gw = 6.max((cell as f64 * 0.5) as i32);
        let gh = 3.max((cell as f64 * 0.18) as i32);
        let mut glyphs = Vec::new();
        for alpha in [26, 52, 80, 110] {
            let mut g = pg::surface_alpha(gw, gh);
            let step = 1.max(gw / 10) as usize;
            let pts: Vec<(f64, f64)> = (0..=gw).step_by(step).map(|i| (i as f64, gh as f64 / 2.0 + (i as f64 / gw as f64 * TAU).sin() * gh as f64 * 0.38)).collect();
            draw::lines(&mut g, rgba_of(COL_WAVE, alpha), false, &pts, 1.max(cell / 22));
            glyphs.push(g);
        }
        self.cache.waves.insert(cell, glyphs.clone());
        glyphs
    }

    fn tint(&mut self, cell: i32, color: Rgb, alpha: i32) -> Surface {
        if let Some(s) = self.cache.tint.get(&(cell, color, alpha)) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(cell, cell);
        surf.fill_c(rgba_of(color, alpha));
        self.cache.tint.insert((cell, color, alpha), surf.clone());
        surf
    }

    /// `_fit(font, text, maxw, color)`
    fn fit(&self, font: Fs, text: &str, maxw: i32, color: impl Col) -> Surface {
        let mut chain = vec![font];
        if font == Fs::Huge {
            chain.push(Fs::Bold);
        }
        if font == Fs::Huge || font == Fs::Bold {
            chain.push(Fs::Small);
        }
        if font != Fs::Tiny {
            chain.push(Fs::Tiny);
        }
        for f in chain {
            let img = self.font_of(f).render(text, true, color.c());
            if img.w() <= maxw {
                return img;
            }
        }
        let mut kurz: Vec<char> = text.chars().collect();
        while kurz.len() > 2 && self.tiny.size(&(kurz.iter().collect::<String>() + "\u{2026}")).0 > maxw {
            kurz.pop();
        }
        self.tiny.render(&(kurz.iter().collect::<String>() + "\u{2026}"), true, color.c())
    }

    // ===================================================== Zeichnen: Brett

    fn draw_waves(&mut self, s: &mut Surface, x: i32, y: i32, cell: i32, now: f64, skip: &[[bool; 10]; 10]) {
        let glyphs = self.wave_glyphs(cell);
        let (gw, gh) = (glyphs[0].w(), glyphs[0].h());
        let cellf = cell as f64;
        for r in 0..N {
            for c in 0..N {
                if skip[r as usize][c as usize] {
                    continue;
                }
                let ph = r as f64 * 1.37 + c as f64 * 2.11;
                let a = (now * 1.25 + ph).sin();
                if a < -0.35 {
                    continue;
                }
                let lvl = 3.min(((a + 0.35) / 1.35 * 4.0) as usize);
                let dx = (now * 0.9 + ph).cos() * cellf * 0.12;
                let yy = (y + r * cell) as f64 + cellf * (0.3 + 0.35 * ((r * 3 + c * 5) % 3) as f64 / 2.0) - gh as f64 / 2.0;
                s.bl(&glyphs[lvl], ((x + c * cell) as f64 + (cell - gw) as f64 / 2.0 + dx, yy));
            }
        }
    }

    fn draw_labels(&mut self, s: &mut Surface, x: i32, y: i32, cell: i32) {
        let p = ui::pal();
        let f = self.label_font.clone();
        let key = (cell, f.get_height(), p.text_dim);
        let imgs = match self.cache.labels.get(&key) {
            Some(i) => i.clone(),
            None => {
                let i = (LETTERS.iter().map(|l| f.render(l, true, p.text_dim)).collect::<Vec<_>>(), (0..N).map(|i| f.render(&(i + 1).to_string(), true, p.text_dim)).collect::<Vec<_>>());
                self.cache.labels.insert(key, i.clone());
                i
            }
        };
        for i in 0..N as usize {
            let ii = i as i32;
            let a = &imgs.0[i];
            s.blr(a, a.get_rect().center_at((x + ii * cell + cell / 2, y - self.lab / 2)));
            let b = &imgs.1[i];
            s.blr(b, b.get_rect().center_at((x - self.lab / 2, y + ii * cell + cell / 2)));
        }
    }

    fn draw_miss(s: &mut Surface, x: i32, y: i32, cell: i32, now: f64, seed: i32) {
        let cellf = cell as f64;
        let (cx, cy) = ((x as f64 + cellf / 2.0) as i32, (y as f64 + cellf / 2.0) as i32);
        let fr = py::fpmod(now * 0.55 + seed as f64 * 0.137, 1.0);
        let rr = (cellf * (0.16 + 0.26 * fr)) as i32;
        if rr > 2 {
            draw::circle(s, bmix(COL_MISS, COL_SEA_BOT, 0.35 + 0.6 * fr), (cx, cy), rr, 1.max(cell / 26));
        }
        draw::circle(s, bmix(COL_MISS, COL_SEA_BOT, 0.25), (cx, cy), 2.max((cellf * 0.2) as i32), 1.max(cell / 16));
        draw::circle(s, COL_MISS, (cx, cy), 1.max((cellf * 0.08) as i32), 0);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_fire(&mut self, s: &mut Surface, x: i32, y: i32, cell: i32, now: f64, seed: i32, strong: bool) {
        let cellf = cell as f64;
        let (cx, cy) = (x as f64 + cellf / 2.0, y as f64 + cellf / 2.0);
        let mut glow = self.glow(cellf * if strong { 0.55 } else { 0.4 }, (255, 120, 40));
        glow.set_alpha(Some(((if strong { 150.0 } else { 90.0 }) + 50.0 * (now * 11.0 + seed as f64).sin()) as i32 as u8));
        s.bl(&glow, (cx - glow.w() as f64 / 2.0, cy - glow.h() as f64 / 2.0));
        glow.set_alpha(Some(255));
        draw::circle(s, COL_SCORCH, (cx as i32, (cy + cellf * 0.06) as i32), 2.max((cellf * 0.22) as i32), 0);
        let base = cy + cellf * 0.22;
        let scale = if strong { 1.0 } else { 0.6 };
        for i in 0..3 {
            let ph = now * 9.0 + seed as f64 * 1.7 + i as f64 * 2.1;
            let fh = cellf * (0.34 + 0.1 * ph.sin()) * scale * if i != 1 { 0.8 } else { 1.0 };
            let fx = cx + (i - 1) as f64 * cellf * 0.13;
            let sway = (ph * 1.3).sin() * cellf * 0.05;
            let w = cellf * 0.1 * scale;
            draw::polygon(s, COL_FLAME, &[(fx - w, base), (fx + sway, base - fh), (fx + w, base)], 0);
            draw::polygon(s, COL_FLAME_IN, &[(fx - w * 0.45, base), (fx + sway * 0.6, base - fh * 0.55), (fx + w * 0.45, base)], 0);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_sea(&mut self, s: &mut Surface, side: usize, x: i32, y: i32, cell: i32, own: bool, now: f64) {
        let base = self.board_base(cell, !own);
        s.bl(&base, (x, y));
        let mut occupied = [[false; 10]; 10];
        let reveal_all = self.state == St::Over;
        let shown: Vec<core::Ship> = self.seas[side].ships.iter().filter(|sh| own || sh.sunk() || reveal_all).cloned().collect();
        for sh in &shown {
            for (r, c) in sh.cells() {
                occupied[r as usize][c as usize] = true;
            }
        }
        for r in 0..10 {
            for c in 0..10 {
                if self.seas[side].shots[r][c] != core::UNKNOWN {
                    occupied[r][c] = true;
                }
            }
        }
        self.draw_waves(s, x, y, cell, now, &occupied);
        for sh in &shown {
            let pos = (x + sh.c * cell, y + sh.r * cell);
            if sh.sunk() {
                let img = self.ship_sprite(sh.idx, cell, sh.horiz, true);
                s.bl(&img, pos);
            } else if own {
                let img = self.ship_sprite(sh.idx, cell, sh.horiz, false);
                s.bl(&img, pos);
            } else {
                let mut img = self.ship_sprite(sh.idx, cell, sh.horiz, false);
                img.set_alpha(Some(150));
                s.bl(&img, pos);
                img.set_alpha(Some(255));
            }
        }
        let sunk_cells: HashSet<(i32, i32)> = self.seas[side].ships.iter().filter(|sh| sh.sunk()).flat_map(|sh| sh.cells()).collect();
        for r in 0..N {
            for c in 0..N {
                let v = self.seas[side].shots[r as usize][c as usize];
                if v == core::MISS {
                    Self::draw_miss(s, x + c * cell, y + r * cell, cell, now, r * 10 + c);
                } else if v == core::HIT {
                    self.draw_fire(s, x + c * cell, y + r * cell, cell, now, r * 10 + c, !sunk_cells.contains(&(r, c)));
                }
            }
        }
        if !own {
            self.draw_radar(s, x, y, cell * N, now);
        }
        let gold = ui::pal().gold;
        for rv in &self.reveals {
            if rv.side != side {
                continue;
            }
            let k = (now - rv.t0) / 1.4;
            if !(0.0..1.0).contains(&k) {
                continue;
            }
            let sh = &rv.ship;
            let w = (if sh.horiz { sh.size } else { 1 }) * cell;
            let h = (if sh.horiz { 1 } else { sh.size }) * cell;
            let grow = (cell as f64 * 0.35 * (1.0 - k)) as i32;
            let col = bmix((255, 255, 255), c3(gold), k);
            let rect = Rect::new(x + sh.c * cell, y + sh.r * cell, w, h).inflate(grow, grow);
            draw::rect_r(s, col, rect, 2.max((cell as f64 * 0.09 * (1.0 - k)) as i32 + 1), 2.max(cell / 5));
        }
    }

    fn draw_sunk_banners(&mut self, s: &mut Surface, now: f64, ox: i32, oy: i32) {
        let v = self.viewer();
        let gold = ui::pal().gold;
        for rv in self.reveals.clone() {
            let k = (now - rv.t0) / 1.6;
            if !(0.0..1.0).contains(&k) {
                continue;
            }
            let side = rv.side;
            let (x, y, cell) = self.board_geom(side);
            let sh = &rv.ship;
            let cellf = cell as f64;
            let cx = (x + ox) as f64 + (sh.c as f64 + if sh.horiz { sh.size as f64 / 2.0 } else { 0.5 }) * cellf;
            let cy = (y + oy) as f64 + (sh.r as f64 + if sh.horiz { 0.5 } else { sh.size as f64 / 2.0 }) * cellf;
            let font = if side != v { Fs::Bold } else { Fs::Small };
            let col = if side != v { gold } else { COL_P2.c() };
            let text = t("bs.sunk_banner");
            let key = (text.clone(), font, col);
            let imgs = match self.cache.banner.get(&key) {
                Some(i) => i.clone(),
                None => {
                    let f = self.font_of(font);
                    let i = (f.render(&text, true, rgb(8, 12, 20)), f.render(&text, true, col));
                    self.cache.banner.insert(key, i.clone());
                    i
                }
            };
            let pop = 1.0 + 0.25 * 0f64.max(1.0 - k * 6.0);
            let alpha = (255.0 * 1f64.min((1.0 - k) / 0.35)) as i32;
            let yy = cy - cellf * (0.2 + 0.9 * smooth(k));
            for (i, img0) in [imgs.0, imgs.1].into_iter().enumerate() {
                let mut img = img0;
                if pop > 1.01 {
                    img = transform::smoothscale(&img, (img.w() as f64 * pop) as i32, (img.h() as f64 * pop) as i32);
                }
                img.set_alpha(Some(alpha.clamp(0, 255) as u8));
                let off = if i == 0 { 2 } else { 0 };
                let half = img.w() / 2 + 4;
                let bx_ = (half as f64).max(((self.b.width - half) as f64).min(cx)) as i32;
                s.blr(&img, img.get_rect().center_at((bx_ + off, yy as i32 + off)));
                img.set_alpha(Some(255));
            }
        }
    }

    fn draw_radar(&mut self, s: &mut Surface, x: i32, y: i32, size: i32, now: f64) {
        let mut surf = match self.cache.sweep.get(&size) {
            Some(sf) => sf.clone(),
            None => {
                let sf = pg::surface(size, size);
                self.cache.sweep.insert(size, sf.clone());
                sf
            }
        };
        surf.fill_c(rgb(0, 0, 0));
        let ang = now * 1.5;
        let (cx, cy) = (size as f64 / 2.0, size as f64 / 2.0);
        let rr = size as f64 * 0.72;
        let steps = 12;
        let span = 1.05;
        for i in 0..steps {
            let a0 = ang - span * (i + 1) as f64 / steps as f64;
            let a1 = ang - span * i as f64 / steps as f64;
            let k = py::pow(1.0 - i as f64 / steps as f64, 2.0);
            draw::polygon(&mut surf, rgb((6.0 * k) as i32, (62.0 * k) as i32, (44.0 * k) as i32), &[(cx, cy), (cx + a0.cos() * rr, cy + a0.sin() * rr), (cx + a1.cos() * rr, cy + a1.sin() * rr)], 0);
        }
        draw::line(&mut surf, (40, 150, 105), (cx, cy), (cx + ang.cos() * rr, cy + ang.sin() * rr), 2.max(size / 220));
        s.bl_flags(&surf, (x, y), BLEND_RGB_ADD);
    }

    fn draw_brackets(s: &mut Surface, rect: Rect, color: impl Col, width: i32) {
        let l = 4.max(rect.w / 3);
        let (x0, y0, x1, y1) = (rect.x, rect.y, rect.right() - 1, rect.bottom() - 1);
        for (ax, ay, dx, dy) in [(x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)] {
            draw::line(s, color, (ax, ay), (ax + dx * l, ay), width);
            draw::line(s, color, (ax, ay), (ax, ay + dy * l), width);
        }
    }

    // ===================================================== Zeichnen: Gefecht

    fn shake_offset(&self, now: f64) -> (i32, i32) {
        let k = (now - self.shake_t0) / 0.32;
        if !(0.0..1.0).contains(&k) || self.b.paused {
            return (0, 0);
        }
        let a = self.shake_amp * (1.0 - k);
        (((now * 90.0).sin() * a) as i32, ((now * 77.0).cos() * a) as i32)
    }

    fn draw_battle(&mut self, s: &mut Surface, now: f64) {
        let p = ui::pal();
        let v = self.viewer();
        let (ox, oy) = self.shake_offset(now);
        let (bx, by, sx, sy) = (self.bx + ox, self.by + oy, self.sx + ox, self.sy + oy);
        let (cb, cs) = (self.cb, self.cs);
        self.draw_labels(s, self.bx, self.by, cb);
        self.draw_sea(s, 1 - v, bx, by, cb, false, now);
        self.draw_sea(s, v, sx, sy, cs, true, now);
        let human = self.state == St::Play && !self.is_ai(self.turn) && self.turn == v;
        if human && self.flight.is_none() {
            let (ar, ac) = self.aim[v];
            let rect = Rect::new(bx + ac * cb, by + ar * cb, cb, cb);
            let free = self.seas[1 - v].shots[ar as usize][ac as usize] == core::UNKNOWN;
            let pul = 0.5 + 0.5 * (now * 6.0).sin();
            let col: Color = if free { bmix(COL_AIM, (255, 255, 255), pul * 0.4).c() } else { p.text_faint };
            if free {
                let tn = self.tint(cb, COL_AIM, (24.0 + 22.0 * pul) as i32);
                s.bl(&tn, (rect.x, rect.y));
            }
            let d = -(2.max(cb / 10));
            Self::draw_brackets(s, rect.inflate(d, d), col, 2.max(cb / 16));
        }
        if self.state == St::Play && self.is_ai(self.turn) {
            let (px, py_) = self.ai_pos;
            let rect = Rect::new((sx as f64 + (px - 0.5) * cs as f64) as i32, (sy as f64 + (py_ - 0.5) * cs as f64) as i32, cs, cs);
            Self::draw_brackets(s, rect, COL_AI_AIM, 1.max(cs / 12));
            draw::circle(s, COL_AI_AIM, rect.center(), 2.max(cs / 3), 1);
        }
        if self.flight.is_some() {
            self.draw_flight(s);
        }
        self.draw_sunk_banners(s, now, ox, oy);
        let cap = if !self.b.multiplayer { t("bs.your_waters") } else { tf("bs.waters_of", &[("name", &self.pname_s(v))]) };
        let img = self.fit(Fs::Tiny, &cap, self.col_w, p.text_dim);
        s.blr(&img, img.get_rect().center_at((self.sx + 5 * cs, self.caption_y)));
        let (fr0, fr1) = (self.fleet_rects[0], self.fleet_rects[1]);
        self.draw_fleet(s, fr0, 1 - v, true);
        self.draw_fleet(s, fr1, v, false);
        self.draw_status(s);
    }

    fn draw_flight(&mut self, s: &mut Surface) {
        let Some(f) = self.flight else {
            return;
        };
        let side = 1 - f.shooter;
        let (x, y, cell) = self.board_geom(side);
        let cellf = cell as f64;
        let (tx, ty) = ((x + f.c * cell) as f64 + cellf / 2.0, (y + f.r * cell) as f64 + cellf / 2.0);
        let (ox, oy, ocell) = self.board_geom(f.shooter);
        let (sx_, sy_) = ((ox + ocell * 5) as f64, (oy + ocell * 5) as f64);
        let k = 1f64.min(f.t / f.dur);
        let dist = py::hypot(tx - sx_, ty - sy_);
        for j in (0..=6).rev() {
            let kk = 0f64.max(k - j as f64 * 0.035);
            let e = kk * kk * (3.0 - 2.0 * kk) * 0.35 + kk * 0.65;
            let px = sx_ + (tx - sx_) * e;
            let py_ = sy_ + (ty - sy_) * e - (PI * kk).sin() * dist * 0.28;
            let rad = 2.max((cellf * 0.16 * (1.0 - j as f64 / 8.0)) as i32);
            let col = bmix((255, 240, 190), (255, 120, 50), j as f64 / 6.0);
            if j == 0 {
                let g = self.glow(cellf * 0.35, (255, 200, 120));
                s.bl(&g, (px - g.w() as f64 / 2.0, py_ - g.h() as f64 / 2.0));
            }
            draw::circle(s, col, (px as i32, py_ as i32), rad, 0);
        }
        let mc = if side == self.viewer() { COL_AI_AIM } else { COL_AIM };
        draw::circle(s, mc, (tx as i32, ty as i32), 3.max((cellf * 0.42 * (1.0 - k * 0.6)) as i32), 1);
    }

    fn pname_s(&self, p: usize) -> String {
        if self.b.multiplayer {
            return self.b.pname(p).chars().take(10).collect();
        }
        if p == 0 {
            t("bs.you")
        } else {
            t("common.ai")
        }
    }

    fn draw_fleet(&mut self, s: &mut Surface, rect: Rect, side: usize, enemy: bool) {
        let title = if enemy { t("bs.fleet_enemy") } else { t("bs.fleet_own") };
        let left = self.seas[side].ships_left();
        let head = self.fit(Fs::Tiny, &format!("{title}  {left}/{}", FLEET.len()), rect.w, if enemy { COL_P2 } else { COL_P1 });
        s.blr(&head, head.get_rect().midtop_at((rect.centerx(), rect.y)));
        let y0 = rect.y + head.h() + 4;
        let row_h = 6.max(py::fdiv32(rect.bottom() - y0, FLEET.len() as i32));
        let u = 4.max((((rect.w - 6) as f64 / 5.0).min(row_h as f64 * 0.78)) as i32);
        for idx in 0..FLEET.len() {
            let ship = self.seas[side].ship_by_idx(idx).map(|i| self.seas[side].ships[i].clone());
            let size = FLEET[idx];
            let sw = size * u;
            let xx = rect.centerx() - (5 * u) / 2;
            let yy = y0 + idx as i32 * row_h + py::fdiv32(row_h - u, 2);
            let sunk = ship.as_ref().is_some_and(|sh| sh.sunk());
            let mut img = self.ship_sprite(idx, u, true, sunk);
            if sunk {
                img.set_alpha(Some(170));
            }
            s.bl(&img, (xx, yy));
            img.set_alpha(Some(255));
            if sunk {
                draw::line(s, COL_INVALID, (xx - 2, yy + u / 2), (xx + sw + 2, yy + u / 2), 2.max(u / 7));
            } else if !enemy {
                if let Some(sh) = ship {
                    for (r, c) in &sh.hits {
                        let i = if sh.horiz { c - sh.c } else { r - sh.r };
                        draw::circle(s, COL_FLAME, (xx + ((i as f64 + 0.5) * u as f64) as i32, yy + u / 2), 2.max(u / 4), 0);
                    }
                }
            }
        }
    }

    fn draw_status(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let y = self.status_y;
        let x0 = self.bx;
        let right = self.bx + 10 * self.cb;
        let mut pip_w = 0;
        let human = self.state == St::Play && !self.is_ai(self.turn);
        if self.state == St::Play && (self.rule("salvo") || self.shots_left > 1) {
            let r = 3.max(self.small.get_height() / 5);
            let n = 0.max(self.shots_left) as i32;
            pip_w = n * (2 * r + 4);
            for i in 0..n {
                let cx = right - r - i * (2 * r + 4);
                let col = if human { COL_AIM } else { COL_AI_AIM };
                draw::circle(s, col, (cx, y), r, 0);
                draw::circle(s, bmix(col, (0, 0, 0), 0.5), (cx, y), r, 1);
            }
            let lbl = self.tiny.render(&tf("bs.shots_left", &[("n", &n.to_string())]), true, p.text_dim);
            if right - pip_w - 8 - lbl.w() > x0 + 120 {
                s.blr(&lbl, lbl.get_rect().midright_at((right - pip_w - 6, y)));
                pip_w += lbl.w() + 10;
            }
        }
        let (text, col) = if let Some(m) = &self.msg {
            (m.clone(), self.msg_col)
        } else if self.state == St::Play && human {
            (t("bs.fire_hint"), p.text_faint)
        } else {
            (String::new(), p.text_dim)
        };
        if !text.is_empty() {
            let img = self.fit(Fs::Small, &text, right - x0 - pip_w - 8, col);
            s.blr(&img, img.get_rect().midleft_at((x0, y)));
        }
    }

    // ===================================================== Zeichnen: Aufstellen

    fn draw_place(&mut self, s: &mut Surface, now: f64) {
        let p = ui::pal();
        let pp = self.place_player;
        let (x, y, cell) = (self.bx, self.by, self.cb);
        self.draw_labels(s, x, y, cell);
        let base = self.board_base(cell, false);
        s.bl(&base, (x, y));
        let mut occ = [[false; 10]; 10];
        for sh in &self.seas[pp].ships {
            for (r, c) in sh.cells() {
                occ[r as usize][c as usize] = true;
            }
        }
        self.draw_waves(s, x, y, cell, now, &occ);
        if !self.rule("touch") {
            let halo = self.tint(cell, (255, 255, 255), 16);
            let mut seen = HashSet::new();
            for sh in self.seas[pp].ships.clone() {
                for (r, c) in sh.cells() {
                    for dr in [-1, 0, 1] {
                        for dc in [-1, 0, 1] {
                            let (rr, cc) = (r + dr, c + dc);
                            if in_board(rr, cc) && !occ[rr as usize][cc as usize] && !seen.contains(&(rr, cc)) {
                                seen.insert((rr, cc));
                                s.bl(&halo, (x + cc * cell, y + rr * cell));
                            }
                        }
                    }
                }
            }
        }
        for sh in self.seas[pp].ships.clone() {
            let img = self.ship_sprite(sh.idx, cell, sh.horiz, false);
            s.bl(&img, (x + sh.c * cell, y + sh.r * cell));
        }
        let hd = self.held;
        if let Some(hd) = hd {
            let size = FLEET[hd.idx];
            let anchor = self.held_anchor();
            let mut img = self.ship_sprite(hd.idx, cell, hd.horiz, false);
            if let Some(anchor) = anchor {
                let valid = self.held_valid();
                let tn = self.tint(cell, if valid { COL_VALID } else { COL_INVALID }, if valid { 70 } else { 90 });
                for (r, c) in cells_of(anchor.0, anchor.1, size, hd.horiz) {
                    if in_board(r, c) {
                        s.bl(&tn, (x + c * cell, y + r * cell));
                    }
                }
                let prev = s.get_clip();
                s.set_clip(Some(Rect::new(x, y, cell * N, cell * N)));
                img.set_alpha(Some(if valid { 235 } else { 150 }));
                s.bl(&img, (x + anchor.1 * cell, y + anchor.0 * cell));
                img.set_alpha(Some(255));
                s.set_clip(Some(prev));
                let rect = Rect::new(x + anchor.1 * cell, y + anchor.0 * cell, (if hd.horiz { size } else { 1 }) * cell, (if hd.horiz { 1 } else { size }) * cell);
                draw::rect_r(s, if valid { COL_VALID } else { COL_INVALID }, rect.clip(Rect::new(x, y, cell * N, cell * N)), 2.max(cell / 14), 2.max(cell / 6));
            } else {
                let (mx, my) = (self.mouse.0 as f64, self.mouse.1 as f64);
                let cellf = cell as f64;
                let off = (hd.grab as f64 + 0.5) * cellf;
                let px = mx - if hd.horiz { off } else { cellf / 2.0 };
                let py_ = my - if hd.horiz { cellf / 2.0 } else { off };
                img.set_alpha(Some(190));
                s.bl(&img, (px, py_));
                img.set_alpha(Some(255));
            }
        } else if self.kb_mode {
            let (r, c) = self.kcursor;
            Self::draw_brackets(s, Rect::new(x + c * cell, y + r * cell, cell, cell), COL_AIM, 2.max(cell / 16));
        }

        let title = t("bs.fleet_own") + &format!("  {}/{}", self.seas[pp].ships.len(), FLEET.len());
        let img = self.fit(Fs::Small, &title, self.col_w, if pp == 0 { COL_P1 } else { COL_P2 });
        s.blr(&img, img.get_rect().center_at((self.col_x + self.col_w / 2, self.dock_title_y)));
        let dc = self.dock_cell;
        for (idx, rc) in self.dock_rects.clone().into_iter().enumerate() {
            let placed = self.seas[pp].ship_by_idx(idx).is_some();
            let holding = hd.is_some_and(|h| h.idx == idx);
            let hover = rc.hit(self.mouse) && hd.is_none();
            let inner = rc.inflate(0, -(2.max(rc.h / 8)));
            draw::rect_r(s, if hover || holding { p.btn_sel } else { p.btn }, inner, 0, 4.max(inner.h / 4));
            draw::rect_r(s, if holding { self.b.accent } else { p.border }, inner, if holding { 2 } else { 1 }, 4.max(inner.h / 4));
            let mut img = self.ship_sprite(idx, dc, true, false);
            let sy_ = rc.centery() - dc / 2;
            if placed || holding {
                img.set_alpha(Some(80));
            }
            s.bl(&img, (rc.x + 8, sy_));
            img.set_alpha(Some(255));
            if placed {
                let ck = 4.max((rc.h / 4).min(dc / 2));
                let (cxk, cyk) = (rc.x + 8 + FLEET[idx] * dc / 2, rc.centery());
                draw::lines(s, COL_VALID, false, &[(cxk - ck, cyk), (cxk - ck / 3, cyk + ck * 2 / 3), (cxk + ck, cyk - ck * 2 / 3)], 2.max(ck / 3));
            }
            let tx = rc.x + 14 + 5 * dc;
            let room = rc.right() - tx - 8;
            let mut label = t(&format!("bs.ship.{}", SHIP_KEYS[idx]));
            let col = if placed { p.text_faint } else { p.text };
            if self.tiny.size(&label).0 > room {
                label = format!("\u{00D7}{}", FLEET[idx]);
            }
            if room > 16 {
                let lbl = self.fit(Fs::Tiny, &label, room, col);
                s.blr(&lbl, lbl.get_rect().midleft_at((tx, rc.centery())));
            }
        }
        let complete = self.seas[pp].complete();
        for key in BUTTONS {
            let rc = self.btn_rects.iter().find(|(k, _)| *k == key).map(|(_, r)| *r).unwrap_or(Rect::new(0, 0, 0, 0));
            let hover = rc.hit(self.mouse);
            let col;
            if key == "ready" {
                let on = complete;
                draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 10);
                draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 10);
                if on && hover {
                    draw::rect_r(s, (255, 255, 255), rc.inflate(-4, -4), 1, 9);
                }
                col = if on { p.text } else { p.text_faint };
            } else {
                draw::rect_r(s, if hover { p.btn_sel } else { p.btn }, rc, 0, 8);
                draw::rect_r(s, if hover { p.border_light } else { p.border }, rc, 1, 8);
                col = if hover { p.text } else { p.text_dim };
            }
            let (label, kl) = match key {
                "rotate" => (t("bs.btn_rotate"), "R"),
                "random" => (t("bs.btn_random"), "X"),
                "clear" => (t("bs.btn_clear"), "C"),
                _ => (t("bs.btn_ready"), "Enter"),
            };
            let kimg = self.tiny.render(kl, true, p.text_faint);
            let room = rc.w - kimg.w() - 24;
            let lbl = self.fit(Fs::Small, &label, room, col);
            s.blr(&lbl, lbl.get_rect().center_at((rc.x + 8 + room / 2, rc.centery())));
            s.blr(&kimg, kimg.get_rect().midright_at((rc.right() - 8, rc.centery())));
        }
        let (text, col) = if let Some(m) = &self.msg {
            (m.clone(), self.msg_col)
        } else if let Some(hd) = hd {
            (tf("bs.holding", &[("ship", &t(&format!("bs.ship.{}", SHIP_KEYS[hd.idx])))]), p.text_dim)
        } else if let Some(hv) = self.dock_rects.iter().position(|r| r.hit(self.mouse)) {
            (format!("{} ({})", t(&format!("bs.ship.{}", SHIP_KEYS[hv])), FLEET[hv]), p.text_dim)
        } else {
            (t("bs.place_hint"), p.text_faint)
        };
        let img = self.fit(Fs::Small, &text, self.col_x + self.col_w - x, col);
        s.blr(&img, img.get_rect().midleft_at((x, self.status_y)));
    }

    // ===================================================== HUD / Rundenende

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        draw::rect(s, p.panel, Rect::new(0, 0, self.b.width, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (self.b.width, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let img_l = self.small.render(&format!("{}: {}", self.pname_s(0), self.wins[0]), true, COL_P1.c());
        s.blr(&img_l, img_l.get_rect().midleft_at((12, cy)));
        let img_r = self.small.render(&format!("{} :{}", self.wins[1], self.pname_s(1)), true, COL_P2.c());
        s.blr(&img_r, img_r.get_rect().midright_at((self.b.width - 12, cy)));
        let (mid, col) = self.hud_center();
        if !mid.is_empty() {
            let room = self.b.width - 2 * img_l.w().max(img_r.w()) - 48;
            let img = self.fit(Fs::Small, &mid, room, col);
            s.blr(&img, img.get_rect().center_at((self.b.width / 2, cy)));
        }
    }

    fn hud_center(&self) -> (String, Color) {
        if self.state == St::Place {
            if self.b.multiplayer {
                return (tf("bs.place_title_p", &[("name", &self.pname_s(self.place_player))]), if self.place_player == 0 { COL_P1.c() } else { COL_P2.c() });
            }
            return (t("bs.place_title"), COL_P1.c());
        }
        if self.state == St::Play {
            let col = if self.turn == 0 { COL_P1.c() } else { COL_P2.c() };
            if self.is_ai(self.turn) {
                return (t("bs.ai_thinks"), col);
            }
            if !self.b.multiplayer {
                return (t("bs.your_turn"), col);
            }
            return (tf("bs.turn", &[("name", &self.pname_s(self.turn))]), col);
        }
        (String::new(), ui::pal().text)
    }

    fn over_rows(&self) -> Vec<(String, String, String)> {
        let mut rows = Vec::new();
        for key in ["bs.stat_shots", "bs.stat_hits", "bs.stat_acc", "bs.stat_turns", "bs.stat_left"] {
            let f = |p: usize| -> String {
                let st = self.stats[p];
                match key {
                    "bs.stat_shots" => st.shots.to_string(),
                    "bs.stat_hits" => st.hits.to_string(),
                    "bs.stat_acc" => format!("{}%", py::round((100 * st.hits) as f64 / 1.max(st.shots) as f64)),
                    "bs.stat_turns" => st.turns.to_string(),
                    _ => format!("{}/{}", self.seas[p].ships_left(), FLEET.len()),
                }
            };
            rows.push((t(key), f(0), f(1)));
        }
        rows
    }

    fn over_panel_rect(&self) -> Rect {
        let (w, h) = (self.b.width, self.b.height);
        let hh = self.huge.get_height();
        let rh = self.small.get_height() + 4;
        let th = self.tiny.get_height();
        let ph = 14 + hh + 10 + rh * 6 + 10 + th + 16;
        let pw = (w - 32).min(340.max((w as f64 * 0.56) as i32));
        let (top, bottom) = (self.hud_h + 6, h - 58);
        let py_ = top + 0.max(py::fdiv32(bottom - top - ph, 2));
        Rect::new(py::fdiv32(w - pw, 2), py_, pw, ph)
    }

    fn draw_over(&mut self, s: &mut Surface, now: f64) {
        let p = ui::pal();
        let mut rect = self.over_panel_rect();
        let k = smooth((now - self.over_t0) / 0.35);
        rect = rect.move_(0, ((1.0 - k) * 18.0) as i32);
        let key = (self.b.width, self.b.height);
        let mut dim = match self.cache.dim.get(&key) {
            Some(d) => d.clone(),
            None => {
                let mut d = pg::surface(self.b.width, self.b.height - self.hud_h - 1);
                d.fill_c(rgb(4, 8, 16));
                self.cache.dim.insert(key, d.clone());
                d
            }
        };
        dim.set_alpha(Some((110.0 * k) as i32 as u8));
        s.bl(&dim, (0, self.hud_h + 1));
        draw::rect_r(s, p.panel, rect, 0, 12);
        ui::draw_panel_ex(s, rect, None, None, None, true, Some(self.b.accent));
        let cx = rect.centerx();
        let mut y = rect.y + 14;
        let (head, col) = if self.b.multiplayer {
            let w = self.winner.unwrap_or(0);
            (tf("common.player_wins_name", &[("name", &self.pname_s(w))]), if self.winner == Some(0) { COL_P1.c() } else { COL_P2.c() })
        } else if self.winner == Some(0) {
            (t("bs.win_you"), p.gold)
        } else {
            (t("bs.win_ai"), COL_P2.c())
        };
        let img = self.fit(Fs::Huge, &head, rect.w - 24, col);
        s.blr(&img, img.get_rect().midtop_at((cx, y)));
        y += self.huge.get_height() + 10;
        let rh = self.small.get_height() + 4;
        let c1 = rect.x + 18;
        let c3_ = rect.right() - 18;
        let colw = self.small.size(&self.pname_s(0)).0.max(self.small.size(&self.pname_s(1)).0).max(self.small.size("100%").0) + 12;
        let c2 = c3_ - colw;
        for (i, txt) in [self.pname_s(0), self.pname_s(1)].iter().enumerate() {
            let im = self.small.render(txt, true, if i == 0 { COL_P1.c() } else { COL_P2.c() });
            s.blr(&im, im.get_rect().midright_at((if i == 0 { c2 } else { c3_ }, y + rh / 2)));
        }
        draw::line(s, p.border, (c1, y + rh), (c3_, y + rh), 1);
        y += rh + 2;
        for (label, a, b) in self.over_rows() {
            let im = self.fit(Fs::Small, &label, c2 - colw - c1 - 6, p.text_dim);
            s.blr(&im, im.get_rect().midleft_at((c1, y + rh / 2)));
            for (val, xx) in [(a, c2), (b, c3_)] {
                let im = self.small.render(&val, true, p.text);
                s.blr(&im, im.get_rect().midright_at((xx, y + rh / 2)));
            }
            y += rh;
        }
        y += 8;
        let hint = self.fit(Fs::Tiny, &t("bs.new_round"), rect.w - 20, p.text_dim);
        s.blr(&hint, hint.get_rect().midtop_at((cx, y)));
    }

    fn draw_handover(&mut self, s: &mut Surface, now: f64) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let bg = match self.cache.curtain.get(&(w, h)) {
            Some(b) => b.clone(),
            None => {
                let mut b = pg::surface(w, h);
                for yy in 0..h {
                    draw::line(&mut b, bmix((16, 50, 88), (4, 16, 34), yy as f64 / 1.max(h - 1) as f64), (0, yy), (w, yy), 1);
                }
                self.cache.curtain.insert((w, h), b.clone());
                b
            }
        };
        s.bl(&bg, (0, 0));
        let cell = 24.max(h / 12);
        let glyphs = self.wave_glyphs(cell);
        for row in 0..4 {
            let yy = (h as f64 * (0.62 + row as f64 * 0.085)) as i32;
            for i in -1..(w / cell + 2) {
                let ph = row as f64 * 1.7 + i as f64 * 0.9;
                let a = (now * 1.2 + ph).sin();
                if a < -0.3 {
                    continue;
                }
                let lvl = 3.min(((a + 0.3) / 1.3 * 4.0) as usize);
                let xx = (i * cell) as f64 + py::fpmod(now * (18 + row * 9) as f64, cell as f64) - cell as f64;
                s.bl(&glyphs[lvl], (xx, yy as f64 + (now * 1.6 + ph).sin() * 3.0));
            }
        }
        let ho = self.handover.unwrap_or(Handover { to: 0, kind: HoKind::Fire, report: None, t: 0.0 });
        let sc = 12.max(h / 22);
        let img = self.ship_sprite(1, sc, true, false);
        let rot = (now * 1.4).sin() * 4.0;
        let ship = transform::rotate(&img, rot);
        s.blr(&ship, ship.get_rect().center_at((w / 2, (h as f64 * 0.58 + (now * 1.9).sin() * 4.0) as i32)));
        let col = if ho.to == 0 { COL_P1.c() } else { COL_P2.c() };
        let mut cy = (h as f64 * 0.2) as i32;
        let title = self.fit(Fs::Huge, &tf("bs.handover_title", &[("name", &self.pname_s(ho.to))]), w - 40, col);
        s.blr(&title, title.get_rect().center_at((w / 2, cy)));
        cy += title.h() / 2 + self.bold.get_height() / 2 + 8;
        let sub = if ho.kind == HoKind::Place { t("bs.handover_place") } else { t("bs.handover_fire") };
        let img = self.fit(Fs::Bold, &sub, w - 40, p.text);
        s.blr(&img, img.get_rect().center_at((w / 2, cy)));
        cy += self.bold.get_height() / 2 + self.small.get_height() / 2 + 10;
        let img = self.fit(Fs::Small, &tf("bs.handover_away", &[("name", &self.pname_s(1 - ho.to))]), w - 40, p.text_dim);
        s.blr(&img, img.get_rect().center_at((w / 2, cy)));
        if let Some((shots, hits)) = ho.report {
            cy += self.small.get_height() + 6;
            let img = self.fit(Fs::Small, &tf("bs.handover_report", &[("shots", &shots.to_string()), ("hits", &hits.to_string())]), w - 40, p.gold);
            s.blr(&img, img.get_rect().center_at((w / 2, cy)));
        }
        if ho.t >= 0.45 {
            let pul = 0.55 + 0.45 * (now * 4.0).sin();
            let img = self.fit(Fs::Small, &t("bs.handover_ready"), w - 40, bmix(c3(p.text_dim), c3(p.text), pul));
            s.blr(&img, img.get_rect().center_at((w / 2, h - 24.max(h / 14))));
        }
    }

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let title = self.fit(Fs::Huge, &Self::name_upper(), w - 30, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.11) as i32)));
        let sub = self.fit(Fs::Small, &t("bs.subtitle"), w - 30, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.11) as i32 + title.h() / 2 + sub.h() / 2 + 6)));
        let items = self.setup_items();
        let sel = items[0.max((items.len() as i64 - 1).min(self.setup_sel)) as usize];
        if !self.b.multiplayer {
            let lbl = self.tiny.render(&t("bs.lbl_diff"), true, p.text_dim);
            s.blr(&lbl, lbl.get_rect().midbottom_at((cx, self.diff_label_y)));
            for (i, rc) in self.diff_rects.clone().into_iter().enumerate() {
                let on = i as i64 == self.diff;
                draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 9);
                draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 9);
                if sel == "diff" && on {
                    draw::rect_r(s, p.text, rc.inflate(4, 4), 1, 11);
                }
                let im = self.fit(Fs::Small, &t(&format!("bs.diff.{}", DIFFS[i])), rc.w - 10, if on { p.text } else { p.text_dim });
                s.blr(&im, im.get_rect().center_at(rc.center()));
            }
        }
        let lbl = self.tiny.render(&t("bs.lbl_rules"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midbottom_at((cx, self.rules_label_y)));
        let (on_txt, off_txt) = (t("common.on"), t("common.off"));
        for (i, rc) in self.rule_rects.clone().into_iter().enumerate() {
            let key = RULES[i];
            let on = self.rules[i];
            let focus = sel == key;
            draw::rect_r(s, if focus { p.btn_sel } else { p.btn }, rc, 0, 9);
            draw::rect_r(s, if focus { self.b.accent } else { p.border }, rc, if focus { 2 } else { 1 }, 9);
            let isz = 10.max((rc.h as f64 * 0.5) as i32);
            Self::draw_rule_icon(s, key, rc.x + 10 + isz / 2, rc.centery(), isz);
            let (pill_w, pill_h) = self.pill_size(rc.h);
            let pill = Rect::new(rc.right() - 10 - pill_w, rc.centery() - pill_h / 2, pill_w, pill_h);
            let fill: Color = if on { bmix(c3(p.btn), c3(p.green), 0.75).c() } else { p.panel };
            draw::rect_r(s, fill, pill, 0, pill_h / 2);
            draw::rect_r(s, if on { p.green } else { p.border_light }, pill, 1, pill_h / 2);
            let kr = pill_h / 2 - 3;
            let kx = if on { pill.right() - pill_h / 2 } else { pill.x + pill_h / 2 };
            draw::circle(s, if on { p.text } else { p.text_dim }, (kx, pill.centery()), kr, 0);
            let st = self.tiny.render(if on { &on_txt } else { &off_txt }, true, if on { rgb(20, 30, 24) } else { p.text_dim });
            let r = if on { st.get_rect().midleft_at((pill.x + 8, pill.centery())) } else { st.get_rect().midright_at((pill.right() - 8, pill.centery())) };
            s.blr(&st, r);
            let room = pill.x - (rc.x + 20 + isz) - 8;
            let im = self.fit(Fs::Small, &t(RULE_TEXT[i]), room, if on { p.text } else { p.text_dim });
            s.blr(&im, im.get_rect().midleft_at((rc.x + 20 + isz, rc.centery())));
        }
        let focus = sel == "start";
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, if focus { p.text } else { self.b.accent }, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.fit(Fs::Tiny, &t("bs.setup_hint"), w - 20, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 13)));
    }

    fn pill_size(&self, row_h: i32) -> (i32, i32) {
        let ph = 16.max((row_h as f64 * 0.62) as i32);
        let tw = self.tiny.size(&t("common.on")).0.max(self.tiny.size(&t("common.off")).0);
        (tw + ph + 16, ph)
    }

    fn draw_rule_icon(s: &mut Surface, key: &str, cx: i32, cy: i32, size: i32) {
        let col = ui::pal().text_dim;
        let u = 3.max(size / 4);
        if key == "touch" {
            draw::rect_r(s, col, Rect::new(cx - size / 2, cy - u - 1, size, u), 0, u / 2);
            draw::rect_r(s, col, Rect::new(cx - size / 2, cy + 1, size - u, u), 0, u / 2);
        } else if key == "salvo" {
            for i in [-1, 0, 1] {
                draw::circle(s, col, (cx + i * (u + 2), cy + py::pmod32(i, 2) * 2 - 1), 2.max(u / 2 + 1), 0);
            }
        } else {
            draw::circle(s, col, (cx, cy), size / 2, 1.max(size / 10));
            draw::line(s, col, (cx - u, cy), (cx + u, cy), 1.max(size / 8));
            draw::line(s, col, (cx, cy - u), (cx, cy + u), 1.max(size / 8));
        }
    }
}

impl ShotAI {
    /// Platzhalter vor dem ersten `reset()` (zieht keinen Zufall).
    fn new_placeholder() -> ShotAI {
        ShotAI::with_rng(core::MEDIUM, true, py::Random::new(0), 0)
    }
}

impl Game for BattleshipGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.diff = self.b.opt_int_py("battleship", "difficulty", 1).map(|d| 0.max(2.min(d))).unwrap_or(1);
        for (i, key) in RULES.iter().enumerate() {
            let def = key == &"touch";
            self.rules[i] = match self.b.opt_value("battleship", key) {
                None => def,
                Some(v) => match v {
                    serde_json::Value::Null => false,
                    serde_json::Value::Bool(b) => b,
                    serde_json::Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
                    serde_json::Value::String(st) => !st.is_empty(),
                    serde_json::Value::Array(a) => !a.is_empty(),
                    serde_json::Value::Object(o) => !o.is_empty(),
                },
            };
        }
        self.cache = Cache::default();
        self.make_fonts();
        self.wins = [0, 0];
        self.starter = 0;
        self.layouts = [None, None];
        self.setup_sel = 0;
        self.mouse = (-1, -1);
        self.last_draw = None;
        self.build_setup_layout();
        self.new_round();
        self.state = St::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.cache = Cache::default();
        self.particles.clear();
        self.make_fonts();
        self.build_setup_layout();
        self.layout();
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn show_highscore_banner(&self) -> bool {
        !self.b.multiplayer
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if matches!(ev.kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            self.mouse = ev.pos;
        }
        match self.state {
            St::Setup => self.handle_setup(ev),
            St::Handover => {
                if (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "KP_Enter", "space"])) || (ev.kind == Kind::MouseDown && ev.button == 1) {
                    self.confirm_handover();
                }
            }
            St::Place => self.handle_place(ev),
            St::Play => self.handle_play(ev),
            St::Over => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "KP_Enter", "space"]) {
                        self.restart();
                    } else if ev.key_in(&["s", "S"]) {
                        self.b.game_over = false;
                        self.state = St::Setup;
                        self.setup_sel = 0;
                        self.build_setup_layout();
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown && ev.button == 1 && Self::now() - self.over_t0 > 0.8 {
                    self.restart();
                }
            }
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.state == St::Handover {
            if let Some(ho) = self.handover.as_mut() {
                ho.t += dt;
                return;
            }
        }
        if self.state != St::Play {
            return;
        }
        if let Some(f) = self.flight.as_mut() {
            f.t += dt;
            if f.t >= f.dur {
                self.impact();
            }
            return;
        }
        if self.after.is_some() {
            self.wait -= dt;
            if self.wait <= 0.0 {
                self.continue_();
            }
            return;
        }
        if self.is_ai(self.turn) {
            self.ai_step(dt);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let now = Self::now();
        let mut dt = match self.last_draw {
            None => 0.0,
            Some(l) => 0f64.max(0.05f64.min(now - l)),
        };
        if self.b.paused {
            dt = 0.0;
        }
        self.last_draw = Some(now);
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            return;
        }
        if self.state == St::Handover {
            self.draw_handover(&mut s, now);
            return;
        }
        self.draw_hud(&mut s);
        if self.state == St::Place {
            self.draw_place(&mut s, now);
        } else {
            self.draw_battle(&mut s, now);
        }
        self.step_particles(&mut s, dt);
        if self.state == St::Over {
            self.draw_over(&mut s, now);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "battleship",
    class_name: "BattleshipGame",
    supports_multiplayer: true,
    modes: &[],
    make: BattleshipGame::new,
};
