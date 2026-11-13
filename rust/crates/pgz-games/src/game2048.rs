//! game2048.rs
//! ===========
//! 2048 - das Zahlen-Schiebespiel, gross ausgebaut.
//!
//! Uebertragen aus `games/game2048.py`: Brettgroessen 3x3-8x8, Modi
//! Klassisch/Zeitangriff/Endlos, Rueckgaengig (aus/3/unbegrenzt), Autosave
//! mit "Fortsetzen", Gleit-/Pop-/Wachs-Animationen, gepufferte Eingaben,
//! Partikel, Ringe, Einblendungen, Sieg- und Endstand-Panel.

use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_core::{achievements, highscore, store};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const SIZES: [i64; 6] = [3, 4, 5, 6, 7, 8];
const MODE_KEYS: [&str; 3] = ["classic", "time", "endless"];
const UNDO_KEYS: [&str; 3] = ["off", "limited", "unlimited"];
const UNDO_LIMIT: i64 = 3;
const TIME_LIMIT: f64 = 180.0;
const WIN_TILE: i64 = 2048;
const SPAWN_FOUR: f64 = 0.1;
const HS_SIZE: i64 = 4;
const HS_MODE: &str = "classic";
const ACH_MAX_SIZE: i64 = 4;
const SECTION: &str = "g2048";
const STACK_MAX: usize = 256;
const SAVE_STACK: usize = 3;
const SAVE_EVERY: f64 = 2.5;
const SLIDE_T: f64 = 0.12;
const SLIDE_FAST: f64 = 0.07;
const POP_T: f64 = 0.18;
const SPAWN_T: f64 = 0.16;
const NUDGE_T: f64 = 0.16;
const OVER_DELAY: f64 = 0.6;
const FLASH_T: f64 = 1.3;
const MSG_T: f64 = 1.8;
const POPUP_T: f64 = 0.8;
const QUEUE_MAX: usize = 4;
const MAX_PARTICLES: usize = 260;

const COL_BOARD: (i32, i32, i32) = (40, 44, 58);
const COL_EMPTY: (i32, i32, i32) = (55, 60, 78);
const COL_TEXT_DARK: (i32, i32, i32) = (104, 94, 84);
const COL_TEXT_LIGHT: (i32, i32, i32) = (250, 247, 242);
const COL_TILE_BEYOND: (i32, i32, i32) = (44, 46, 60);
const COL_TEXT_BEYOND: (i32, i32, i32) = (255, 214, 102);

fn tile_known(v: i64) -> Option<(i32, i32, i32)> {
    Some(match v {
        2 => (238, 228, 218),
        4 => (237, 224, 200),
        8 => (242, 177, 121),
        16 => (245, 149, 99),
        32 => (246, 124, 95),
        64 => (246, 94, 59),
        128 => (237, 207, 114),
        256 => (237, 204, 97),
        512 => (237, 200, 80),
        1024 => (237, 197, 63),
        2048 => (237, 194, 46),
        4096 => (92, 196, 146),
        8192 => (52, 168, 196),
        16384 => (82, 122, 226),
        32768 => (138, 96, 222),
        65536 => (190, 80, 196),
        131072 => (228, 70, 118),
        _ => return None,
    })
}

fn tile_color(v: i64) -> (i32, i32, i32) {
    tile_known(v).unwrap_or(COL_TILE_BEYOND)
}

fn tile_text_color(v: i64) -> (i32, i32, i32) {
    if v <= 4 {
        return COL_TEXT_DARK;
    }
    if tile_known(v).is_some() {
        COL_TEXT_LIGHT
    } else {
        COL_TEXT_BEYOND
    }
}

fn digit_scale(n: usize) -> f64 {
    match n {
        1 => 0.52,
        2 => 0.50,
        3 => 0.42,
        4 => 0.34,
        5 => 0.28,
        6 => 0.24,
        _ => 0.20,
    }
}

fn dir_info(d: char) -> (&'static str, &'static str, (i64, i64)) {
    match d {
        'L' => ("Left", "left", (-1, 0)),
        'R' => ("Right", "right", (1, 0)),
        'U' => ("Up", "up", (0, -1)),
        _ => ("Down", "down", (0, 1)),
    }
}

type Grid = Vec<Vec<i64>>;
type Slide = (usize, usize, usize, usize, i64);

// ===================================================== Reine Spiellogik

fn slide_line(vals: &[i64]) -> (Vec<i64>, i64, Vec<(usize, usize, bool)>) {
    let mut out = vec![0; vals.len()];
    let mut wege = Vec::new();
    let mut punkte = 0;
    let mut pos: i64 = -1;
    let mut frei = false;
    for (i, &v) in vals.iter().enumerate() {
        if v == 0 {
            continue;
        }
        if frei && out[pos as usize] == v {
            out[pos as usize] = v * 2;
            punkte += v * 2;
            frei = false;
            wege.push((i, pos as usize, true));
        } else {
            pos += 1;
            out[pos as usize] = v;
            frei = true;
            wege.push((i, pos as usize, false));
        }
    }
    (out, punkte, wege)
}

fn line_cells(n: usize, d: char, k: usize) -> Vec<(usize, usize)> {
    match d {
        'L' => (0..n).map(|c| (k, c)).collect(),
        'R' => (0..n).rev().map(|c| (k, c)).collect(),
        'U' => (0..n).map(|r| (r, k)).collect(),
        _ => (0..n).rev().map(|r| (r, k)).collect(),
    }
}

struct Plan {
    grid: Grid,
    gained: i64,
    slides: Vec<Slide>,
    merges: Vec<(usize, usize, i64)>,
    moved: bool,
}

fn plan_move(grid: &Grid, d: char) -> Plan {
    let n = grid.len();
    let mut new = vec![vec![0; n]; n];
    let (mut slides, mut merges) = (Vec::new(), Vec::new());
    let mut gained = 0;
    for k in 0..n {
        let cells = line_cells(n, d, k);
        let vals: Vec<i64> = cells.iter().map(|&(r, c)| grid[r][c]).collect();
        let (out, pts, wege) = slide_line(&vals);
        gained += pts;
        for (i, &(r, c)) in cells.iter().enumerate() {
            new[r][c] = out[i];
        }
        for (von, nach, versch) in wege {
            let (fr, fc) = cells[von];
            let (tr, tc) = cells[nach];
            slides.push((fr, fc, tr, tc, vals[von]));
            if versch {
                merges.push((tr, tc, out[nach]));
            }
        }
    }
    let moved = new != *grid;
    Plan { grid: new, gained, slides, merges, moved }
}

fn spawn_tile(grid: &mut Grid, rng: &mut py::Random) -> Option<(usize, usize, i64)> {
    let frei: Vec<(usize, usize)> = grid.iter().enumerate().flat_map(|(r, row)| row.iter().enumerate().filter(|(_, v)| **v == 0).map(move |(c, _)| (r, c))).collect();
    if frei.is_empty() {
        return None;
    }
    let (r, c) = frei[rng.choice_index(frei.len())];
    let v = if rng.random() < SPAWN_FOUR { 4 } else { 2 };
    grid[r][c] = v;
    Some((r, c, v))
}

fn has_moves(grid: &Grid) -> bool {
    let n = grid.len();
    for r in 0..n {
        for c in 0..n {
            let v = grid[r][c];
            if v == 0 || (c + 1 < n && v == grid[r][c + 1]) || (r + 1 < n && v == grid[r + 1][c]) {
                return true;
            }
        }
    }
    false
}

fn max_tile(grid: &Grid) -> i64 {
    grid.iter().flatten().copied().max().unwrap_or(0)
}

fn strict_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.as_i64(),
        _ => None,
    }
}

fn v_int(v: Option<&Value>, lo: i64, hi: i64, default: i64) -> i64 {
    strict_int(v).map(|x| x.clamp(lo, hi)).unwrap_or(default)
}

fn v_num(v: Option<&Value>, lo: f64, hi: f64, default: f64) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().map(|x| lo.max(hi.min(x))).unwrap_or(default),
        _ => default,
    }
}

fn valid_grid(v: Option<&Value>, n: usize) -> Option<Grid> {
    let rows = v?.as_array()?;
    if rows.len() != n {
        return None;
    }
    let mut g = Vec::new();
    for row in rows {
        let row = row.as_array()?;
        if row.len() != n {
            return None;
        }
        let mut out = Vec::new();
        for x in row {
            let x = strict_int(Some(x))?;
            if x != 0 && (x < 2 || x > 1 << 40 || x & (x - 1) != 0) {
                return None;
            }
            out.push(x);
        }
        g.push(out);
    }
    Some(g)
}

#[derive(Clone)]
struct Entry {
    grid: Grid,
    points: i64,
    moves: i64,
    reached: bool,
    slides: Vec<Slide>,
    spawn: Option<(usize, usize, i64)>,
}

#[derive(Clone)]
struct Save {
    grid: Grid,
    points: i64,
    moves: i64,
    undo_used: i64,
    assisted: bool,
    reached: bool,
    continued: bool,
    peak: i64,
    time_left: f64,
    clock_on: bool,
    elapsed: f64,
    stack: Vec<Entry>,
}

fn is_true(v: Option<&Value>) -> bool {
    v == Some(&Value::Bool(true))
}

fn clean_save(sv: &Value, n: usize) -> Option<Save> {
    let grid = valid_grid(sv.get("grid"), n)?;
    let mut out = Save {
        points: v_int(sv.get("points"), 0, 1_000_000_000_000, 0),
        moves: v_int(sv.get("moves"), 0, 1_000_000_000, 0),
        undo_used: v_int(sv.get("undo_used"), 0, 1_000_000_000, 0),
        assisted: is_true(sv.get("assisted")),
        reached: is_true(sv.get("reached")),
        continued: is_true(sv.get("continued")),
        peak: v_int(sv.get("peak"), 0, 1 << 40, 0),
        time_left: v_num(sv.get("time_left"), 0.0, TIME_LIMIT, TIME_LIMIT),
        clock_on: is_true(sv.get("clock_on")),
        elapsed: v_num(sv.get("elapsed"), 0.0, 1e7, 0.0),
        grid,
        stack: Vec::new(),
    };
    out.peak = out.peak.max(max_tile(&out.grid));
    if let Some(raw) = sv.get("stack").and_then(|v| v.as_array()) {
        let start = raw.len().saturating_sub(SAVE_STACK);
        for e in &raw[start..] {
            if !e.is_object() {
                continue;
            }
            let Some(g) = valid_grid(e.get("grid"), n) else {
                continue;
            };
            let slides = e
                .get("slides")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|sl| {
                            let sl = sl.as_array()?;
                            if sl.len() != 5 {
                                return None;
                            }
                            let xs: Option<Vec<i64>> = sl.iter().map(|x| strict_int(Some(x))).collect();
                            let xs = xs?;
                            if xs[..4].iter().all(|x| (0..n as i64).contains(x)) && xs[4] >= 2 {
                                Some((xs[0] as usize, xs[1] as usize, xs[2] as usize, xs[3] as usize, xs[4]))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            let spawn = e.get("spawn").and_then(|v| v.as_array()).and_then(|a| {
                if a.len() != 3 {
                    return None;
                }
                let xs: Option<Vec<i64>> = a.iter().map(|x| strict_int(Some(x))).collect();
                let xs = xs?;
                if (0..n as i64).contains(&xs[0]) && (0..n as i64).contains(&xs[1]) && (xs[2] == 2 || xs[2] == 4) {
                    Some((xs[0] as usize, xs[1] as usize, xs[2]))
                } else {
                    None
                }
            });
            out.stack.push(Entry {
                grid: g,
                points: v_int(e.get("points"), 0, 1_000_000_000_000, 0),
                moves: v_int(e.get("moves"), 0, 1_000_000_000, 0),
                reached: is_true(e.get("reached")),
                slides,
                spawn,
            });
        }
    }
    Some(out)
}

fn save_json(sv: &Save) -> Value {
    let stack: Vec<Value> = sv
        .stack
        .iter()
        .map(|e| {
            json!({
                "grid": e.grid, "points": e.points, "moves": e.moves, "reached": e.reached,
                "slides": e.slides.iter().map(|s| vec![s.0 as i64, s.1 as i64, s.2 as i64, s.3 as i64, s.4]).collect::<Vec<_>>(),
                "spawn": e.spawn.map(|s| vec![s.0 as i64, s.1 as i64, s.2]),
            })
        })
        .collect();
    json!({
        "grid": sv.grid, "points": sv.points, "moves": sv.moves, "undo_used": sv.undo_used,
        "assisted": sv.assisted, "reached": sv.reached, "continued": sv.continued, "peak": sv.peak,
        "time_left": sv.time_left, "clock_on": sv.clock_on, "elapsed": sv.elapsed, "stack": stack,
    })
}

fn fmt_time(sec: f64) -> String {
    let sec = 0.max((sec - 1e-6).ceil() as i64);
    format!("{}:{:02}", sec / 60, sec % 60)
}

fn ease_out_cubic(p: f64) -> f64 {
    1.0 - py::pow(1.0 - p, 3.0)
}

fn ease_out_back(p: f64) -> f64 {
    let c1 = 1.70158;
    let q = p - 1.0;
    1.0 + (c1 + 1.0) * py::pow(q, 3.0) + c1 * py::pow(q, 2.0)
}

fn shade(col: (i32, i32, i32), f: f64) -> (i32, i32, i32) {
    ((col.0 as f64 * f) as i32, (col.1 as f64 * f) as i32, (col.2 as f64 * f) as i32)
}

struct SlideAnim {
    t: f64,
    dur: f64,
    slides: Vec<Slide>,
    merges: Vec<(usize, usize, i64)>,
    spawn: Option<(usize, usize, i64)>,
    vanish: Option<(usize, usize, i64)>,
    undo: bool,
    flash: Option<i64>,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Win,
}

#[derive(Default)]
struct Data {
    best: Map<String, Value>,
    best_assisted: Map<String, Value>,
    best_tile: Map<String, Value>,
    top_tile: i64,
    // Python-dict: Einfuegereihenfolge (landet so in mem.json).
    saves: indexmap::IndexMap<String, Save>,
}

fn book_get(m: &Map<String, Value>, k: &str) -> i64 {
    m.get(k).and_then(|v| v.as_i64()).unwrap_or(0)
}

/// `Game2048`
pub struct Game2048 {
    b: GameBase,
    size: i64,
    gmode: &'static str,
    undo_rule: &'static str,
    rng: py::Random,
    data: Data,
    anim_t: f64,
    hover: Option<(i32, i32)>,
    drag: Option<(i32, i32)>,
    board_cache: Option<((i32, i64, i32, i32, String), Surface, i32)>,
    tile_cache: HashMap<(i64, i32), Surface>,
    setup_focus: i64,
    setup_act: i64,
    // Partie
    grid: Grid,
    points: i64,
    moves: i64,
    undo_used: i64,
    assisted: bool,
    reached: bool,
    continued: bool,
    peak: i64,
    max_tile: i64,
    time_left: f64,
    clock_on: bool,
    elapsed: f64,
    stack: Vec<Entry>,
    queue: Vec<String>,
    slide: Option<SlideAnim>,
    pops: Vec<((usize, usize), f64)>,
    spawns: Vec<((usize, usize), f64)>,
    particles: Vec<(f64, f64, f64, f64, f64, f64, Color, f64)>,
    rings: Vec<(f64, f64, f64, (i32, i32, i32))>,
    popups: Vec<(i64, f64)>,
    flash: Option<(String, f64, (i32, i32, i32))>,
    msg: Option<(String, f64)>,
    nudge: Option<(i64, i64, f64)>,
    over_t: Option<f64>,
    over_reason: &'static str,
    new_best: bool,
    win_pending: bool,
    panel_ticks: i64,
    best_start: (i64, i64),
    dirty: bool,
    save_cd: f64,
    state: State,
    off: (i32, i32),
    // Schriften
    tiny: Font,
    small: Font,
    lab_px: i32,
    big_px: i32,
    mid_px: i32,
    row_px: i32,
    f_label: Font,
    f_big: Font,
    f_mid: Font,
    f_row: Font,
    f_row_b: Font,
    f_title: Font,
    f_setup_title: Font,
    // Layout
    gap: i32,
    cell: i32,
    board_rect: Rect,
    time_rect: Rect,
    side_rect: Rect,
    pad: i32,
    tag_rect: Rect,
    score_rect: Rect,
    best_rect: Rect,
    row_h: i32,
    info_rect: Rect,
    btn_rects: Vec<Rect>,
    btn_stacked: bool,
    hint_area: Rect,
    title_y: i32,
    setup_sub: bool,
    size_rects: Vec<Rect>,
    mode_rects: Vec<Rect>,
    undo_rects: Vec<Rect>,
    info_rect_setup: Rect,
    info_pad: i32,
    cont_rect: Rect,
    new_rect: Rect,
    start_rect: Rect,
}

impl Game2048 {
    /// `Game2048(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = Game2048 {
            b,
            size: 4,
            gmode: "classic",
            undo_rule: "limited",
            rng: py::Random::new(0),
            data: Data::default(),
            anim_t: 0.0,
            hover: None,
            drag: None,
            board_cache: None,
            tile_cache: HashMap::new(),
            setup_focus: 0,
            setup_act: 0,
            grid: Vec::new(),
            points: 0,
            moves: 0,
            undo_used: 0,
            assisted: false,
            reached: false,
            continued: false,
            peak: 0,
            max_tile: 0,
            time_left: TIME_LIMIT,
            clock_on: false,
            elapsed: 0.0,
            stack: Vec::new(),
            queue: Vec::new(),
            slide: None,
            pops: Vec::new(),
            spawns: Vec::new(),
            particles: Vec::new(),
            rings: Vec::new(),
            popups: Vec::new(),
            flash: None,
            msg: None,
            nudge: None,
            over_t: None,
            over_reason: "stuck",
            new_best: false,
            win_pending: false,
            panel_ticks: -1_000_000_000,
            best_start: (0, 0),
            dirty: false,
            save_cd: SAVE_EVERY,
            state: State::Setup,
            off: (0, 0),
            tiny: f.clone(),
            small: f.clone(),
            lab_px: 10,
            big_px: 20,
            mid_px: 15,
            row_px: 12,
            f_label: f.clone(),
            f_big: f.clone(),
            f_mid: f.clone(),
            f_row: f.clone(),
            f_row_b: f.clone(),
            f_title: f.clone(),
            f_setup_title: f,
            gap: 4,
            cell: 20,
            board_rect: z,
            time_rect: z,
            side_rect: z,
            pad: 6,
            tag_rect: z,
            score_rect: z,
            best_rect: z,
            row_h: 12,
            info_rect: z,
            btn_rects: Vec::new(),
            btn_stacked: true,
            hint_area: z,
            title_y: 30,
            setup_sub: true,
            size_rects: Vec::new(),
            mode_rects: Vec::new(),
            undo_rects: Vec::new(),
            info_rect_setup: z,
            info_pad: 5,
            cont_rect: z,
            new_rect: z,
            start_rect: z,
        };
        g.reset();
        Box::new(g)
    }

    fn blank_game(&mut self) {
        let n = self.size as usize;
        self.grid = vec![vec![0; n]; n];
        self.points = 0;
        self.moves = 0;
        self.undo_used = 0;
        self.assisted = false;
        self.reached = false;
        self.continued = false;
        self.peak = 0;
        self.max_tile = 0;
        self.time_left = TIME_LIMIT;
        self.clock_on = false;
        self.elapsed = 0.0;
        self.stack.clear();
        self.queue.clear();
        self.slide = None;
        self.pops.clear();
        self.spawns.clear();
        self.particles.clear();
        self.rings.clear();
        self.popups.clear();
        self.flash = None;
        self.msg = None;
        self.nudge = None;
        self.over_t = None;
        self.over_reason = "stuck";
        self.new_best = false;
        self.win_pending = false;
        self.panel_ticks = -1_000_000_000;
        self.best_start = (0, 0);
        self.dirty = false;
        self.save_cd = SAVE_EVERY;
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.tiny = ui::f(11.max(h / 40));
        self.small = ui::f(13.max(h / 32));
        self.b.font = ui::f(15.max(h / 26));
        self.lab_px = 10.max(h / 44);
        self.big_px = 20.max(h / 15);
        self.mid_px = 15.max(h / 24);
        self.row_px = 12.max(h / 34);
        self.f_label = ui::f(self.lab_px);
        self.f_big = ui::fb(self.big_px);
        self.f_mid = ui::fb(self.mid_px);
        self.f_row = ui::f(self.row_px);
        self.f_row_b = ui::fb(self.row_px);
        self.f_title = ui::fb(24.max(h / 13));
        self.f_setup_title = ui::fb(28.max(h / 12));
    }

    // ----- Hilfen

    fn key(&self) -> String {
        format!("{}-{}", self.size, self.gmode)
    }

    fn counts(&self) -> bool {
        self.size == HS_SIZE && self.gmode == HS_MODE && !self.assisted
    }

    fn sync_score(&mut self) {
        self.b.score = if self.counts() { self.points } else { 0 };
    }

    fn undo_left(&self) -> i64 {
        if self.undo_rule == "limited" {
            0.max(UNDO_LIMIT - self.undo_used)
        } else {
            0
        }
    }

    fn can_undo(&self) -> bool {
        if self.undo_rule == "off" || self.stack.is_empty() {
            return false;
        }
        if self.b.game_over && self.over_reason == "time" {
            return false;
        }
        self.undo_rule == "unlimited" || self.undo_left() > 0
    }

    fn free(&self, key: &str) -> bool {
        [key.to_string(), key.to_lowercase(), key.to_uppercase()].iter().all(|k| self.b.key_is_free(k))
    }

    fn tone(&self, value: i64) {
        let step = 0.max((2.max(value) as f64).log2() as i64 - 6);
        audio::tone(330.0 * py::pow(2.0, (step * 2) as f64 / 12.0), 0.11, Some(&self.b.settings), "triangle", 0.22);
    }

    // ----- Persistenz

    fn valid_key(k: &str) -> bool {
        SIZES.iter().any(|n| MODE_KEYS.iter().any(|m| format!("{n}-{m}") == k))
    }

    fn load_data() -> Data {
        let raw = store::load_section(SECTION);
        let mut data = Data::default();
        for (name, slot) in [("best", 0), ("best_assisted", 1), ("best_tile", 2)] {
            if let Some(Value::Object(src)) = raw.get(name) {
                for (k, v) in src {
                    if Self::valid_key(k) && v_int(Some(v), 1, 1_000_000_000_000, 0) > 0 {
                        let target = match slot {
                            0 => &mut data.best,
                            1 => &mut data.best_assisted,
                            _ => &mut data.best_tile,
                        };
                        target.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        data.top_tile = v_int(raw.get("top_tile"), 0, 1 << 40, 0);
        if let Some(Value::Object(saves)) = raw.get("saves") {
            for (k, sv) in saves {
                if Self::valid_key(k) {
                    let n: usize = k.split('-').next().and_then(|x| x.parse().ok()).unwrap_or(4);
                    if let Some(c) = clean_save(sv, n) {
                        data.saves.insert(k.clone(), c);
                    }
                }
            }
        }
        data
    }

    fn persist(&self) {
        let mut sec = Map::new();
        sec.insert("best".into(), Value::Object(self.data.best.clone()));
        sec.insert("best_assisted".into(), Value::Object(self.data.best_assisted.clone()));
        sec.insert("best_tile".into(), Value::Object(self.data.best_tile.clone()));
        sec.insert("top_tile".into(), json!(self.data.top_tile));
        let mut saves = Map::new();
        for (k, v) in &self.data.saves {
            saves.insert(k.clone(), save_json(v));
        }
        sec.insert("saves".into(), Value::Object(saves));
        store::save_section(SECTION, sec);
    }

    fn snapshot(&self) -> Save {
        let start = self.stack.len().saturating_sub(SAVE_STACK);
        Save {
            grid: self.grid.clone(),
            points: self.points,
            moves: self.moves,
            undo_used: self.undo_used,
            assisted: self.assisted,
            reached: self.reached,
            continued: self.continued,
            peak: self.peak,
            time_left: py::round_to(self.time_left, 2),
            clock_on: self.clock_on,
            elapsed: py::round_to(self.elapsed, 1),
            stack: self.stack[start..].to_vec(),
        }
    }

    fn store_game(&mut self) {
        if matches!(self.state, State::Play | State::Win) && !self.b.game_over && self.moves > 0 {
            let k = self.key();
            let s = self.snapshot();
            self.data.saves.insert(k, s);
        }
    }

    fn note_best(&mut self) {
        let key = self.key();
        let book = if self.assisted { &mut self.data.best_assisted } else { &mut self.data.best };
        if self.points > book_get(book, &key) {
            book.insert(key.clone(), json!(self.points));
        }
        if self.max_tile > book_get(&self.data.best_tile, &key) {
            self.data.best_tile.insert(key, json!(self.max_tile));
        }
        if self.max_tile > self.data.top_tile {
            self.data.top_tile = self.max_tile;
        }
    }

    fn bank_highscore(&self) {
        if self.b.score <= 0 || self.b.is_menu {
            return;
        }
        let (hs, _) = highscore::update(self.b.key, self.b.score);
        achievements::on_highscore(self.b.key, hs, false);
    }

    // ----- Partie starten

    fn start(&mut self, resume: bool) {
        let key = self.key();
        let sv = if resume { self.data.saves.get(&key).cloned() } else { None };
        if sv.is_none() && !self.b.game_over {
            self.bank_highscore();
        }
        self.blank_game();
        self.layout();
        self.b.new_round_result();
        self.b.game_over = false;
        self.state = State::Play;
        if let Some(sv) = sv {
            self.grid = sv.grid.clone();
            self.points = sv.points;
            self.moves = sv.moves;
            self.undo_used = sv.undo_used;
            self.assisted = sv.assisted;
            self.reached = sv.reached;
            self.continued = sv.continued;
            self.peak = sv.peak;
            self.time_left = sv.time_left;
            self.clock_on = sv.clock_on;
            self.elapsed = sv.elapsed;
            self.stack = sv.stack.clone();
            if self.undo_rule == "off" {
                self.stack.clear();
            }
            let n = self.size as usize;
            self.spawns = (0..n).flat_map(|r| (0..n).map(move |c| (r, c))).filter(|&(r, c)| self.grid[r][c] != 0).map(|(r, c)| ((r, c), -0.02 * (r + c) as f64)).collect();
        } else {
            if self.data.saves.shift_remove(&key).is_some() {
                self.persist();
            }
            for _ in 0..2 {
                if let Some(sp) = spawn_tile(&mut self.grid, &mut self.rng) {
                    self.spawns.push(((sp.0, sp.1), 0.0));
                }
            }
        }
        self.max_tile = max_tile(&self.grid);
        self.peak = self.peak.max(self.max_tile);
        self.best_start = (book_get(&self.data.best, &key), book_get(&self.data.best_assisted, &key));
        self.sync_score();
        if self.reached && self.gmode == "classic" && !self.continued {
            self.state = State::Win;
            self.panel_ticks = pg::ticks() as i64;
        } else if self.gmode == "time" && !self.clock_on {
            self.message(t("g2048.clock_hint"));
        }
        self.b.play_sound("click");
    }

    fn to_setup(&mut self) {
        self.store_game();
        self.persist();
        self.dirty = false;
        self.queue.clear();
        self.slide = None;
        self.b.game_over = false;
        self.setup_act = 0;
        self.state = State::Setup;
        self.b.play_sound("click");
    }

    // ----- Layout

    fn layout(&mut self) {
        let (w, h, n) = (self.b.width, self.b.height, self.size as i32);
        let m = 10.max(h / 40);
        let side_w = 150f64.max(340f64.min(w as f64 * 0.27)) as i32;
        let bar = if self.gmode == "time" { 6.max(h / 64) } else { 0 };
        let bar_gap = if bar != 0 { 5.max(h / 90) } else { 0 };
        let area_w = w - side_w - 3 * m;
        let area_h = h - 2 * m - bar - bar_gap;
        let mut board = 60.max(area_w.min(area_h));
        let gap = 3.max((board as f64 * if n <= 4 { 0.13 } else { 0.10 } / (n + 1) as f64) as i32);
        let cell = 8.max(py::fdiv32(board - gap * (n + 1), n));
        board = cell * n + gap * (n + 1);
        let bx = m + py::fdiv32(area_w - board, 2);
        let by = m + py::fdiv32(area_h - board, 2);
        self.gap = gap;
        self.cell = cell;
        self.board_rect = Rect::new(bx, by, board, board);
        self.time_rect = Rect::new(bx, by + board + bar_gap, board, bar);
        self.side_rect = Rect::new(w - m - side_w, by, side_w, board + bar + bar_gap);
        self.tile_cache.clear();
        self.layout_side();
    }

    fn layout_side(&mut self) {
        let sd = self.side_rect;
        let h = self.b.height;
        let pad = 6.max(h / 64);
        let gap = 5.max(h / 90);
        self.pad = pad;
        let lh = self.f_label.get_height();
        let mut y = sd.y;
        self.tag_rect = Rect::new(sd.x, y, sd.w, lh);
        y += lh + gap;
        let ch = lh + self.f_big.get_height() + 2 * pad + 2;
        self.score_rect = Rect::new(sd.x, y, sd.w, ch);
        y += ch + gap;
        let ch = lh + self.f_mid.get_height() + 2 * pad + 2;
        self.best_rect = Rect::new(sd.x, y, sd.w, ch);
        y += ch + gap;
        self.row_h = self.f_row.get_height() + 4.max(h / 90);
        self.info_rect = Rect::new(sd.x, y, sd.w, 4 * self.row_h + 2 * pad);
        y = self.info_rect.bottom() + gap;
        let bh = 26.max(h / 17);
        let stacked_h = 3 * bh + 2 * gap;
        if sd.bottom() - y >= stacked_h {
            let top = sd.bottom() - stacked_h;
            self.btn_rects = (0..3).map(|i| Rect::new(sd.x, top + i * (bh + gap), sd.w, bh)).collect();
            self.btn_stacked = true;
        } else {
            let bw = py::fdiv32(sd.w - 2 * gap, 3);
            self.btn_rects = (0..3).map(|i| Rect::new(sd.x + i * (bw + gap), sd.bottom() - bh, bw, bh)).collect();
            self.btn_stacked = false;
        }
        self.hint_area = Rect::new(sd.x, y, sd.w, 0.max(self.btn_rects[0].y - gap - y));
    }

    fn cell_pos(&self, r: usize, c: usize) -> (i32, i32) {
        let br = self.board_rect;
        (br.x + self.off.0 + self.gap + c as i32 * (self.cell + self.gap), br.y + self.off.1 + self.gap + r as i32 * (self.cell + self.gap))
    }

    // ----- Setup

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let th = self.f_setup_title.get_height();
        let sh = self.small.get_height();
        let lh = self.tiny.get_height();
        self.title_y = 30.max((h as f64 * 0.085) as i32);
        let bottom = h - 36 - 4.max(h / 90);
        let g = 5.max(h / 72);
        let bw = 620.min(w - 40);
        let info_pad = 5.max(h / 90);
        let act_min = sh + lh + 18;
        let (mut top, mut info_h, mut avail, mut row_h, mut act_h, mut used, mut with_sub) = (0, 0, 0, 0, 0, 0, true);
        for ws in [true, false] {
            with_sub = ws;
            top = if ws { self.title_y + (th / 2 + 36).max(42) + sh / 2 + 10.max(h / 40) } else { self.title_y + th / 2 + 14 + 6.max(h / 60) };
            info_h = 3 * lh + 2 * info_pad + 4;
            avail = bottom - top;
            let fixed = 3 * (lh + 3) + 4 * g + info_h;
            row_h = ((avail - fixed - act_min) as f64 / 3.0) as i32;
            row_h = 22.max(row_h.min(36.max(h / 15)));
            act_h = act_min.max(row_h + 6);
            used = fixed + 3 * row_h + act_h;
            if used <= avail {
                break;
            }
        }
        self.setup_sub = with_sub;
        let extra = 0.max(avail - used);
        let vg = g + py::fdiv32(extra, 8).min(h / 28);
        used += 4 * (vg - g);
        let mut y = top + 0.max(py::fdiv32(avail - used, 2));
        let row = |yy: i32, n: i32| -> Vec<Rect> {
            let cw = (bw - g * (n - 1)) as f64 / n as f64;
            (0..n).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + g as f64)) as i32, yy, cw as i32, row_h)).collect()
        };
        self.size_rects = row(y + lh + 3, SIZES.len() as i32);
        y += lh + 3 + row_h + vg;
        self.mode_rects = row(y + lh + 3, MODE_KEYS.len() as i32);
        y += lh + 3 + row_h + vg;
        self.undo_rects = row(y + lh + 3, UNDO_KEYS.len() as i32);
        y += lh + 3 + row_h + vg;
        self.info_rect_setup = Rect::new(cx - bw / 2, y, bw, info_h);
        self.info_pad = info_pad;
        y += info_h + vg;
        let half = py::fdiv32(bw - g, 2);
        self.cont_rect = Rect::new(cx - bw / 2, y, half, act_h);
        self.new_rect = Rect::new(cx - bw / 2 + half + g, y, half, act_h);
        let sw = bw.min(200.max(w / 3));
        self.start_rect = Rect::new(cx - sw / 2, y, sw, act_h);
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt(SECTION, key, value);
    }

    fn saved(&self) -> Option<&Save> {
        self.data.saves.get(&self.key())
    }

    fn set_size(&mut self, n: i64) {
        if n == self.size {
            return;
        }
        self.size = n;
        self.save_setting("size", json!(n));
        self.setup_act = 0;
        self.layout();
        self.b.play_sound("click");
    }

    fn set_mode(&mut self, m: &'static str) {
        if m == self.gmode {
            return;
        }
        self.gmode = m;
        self.save_setting("mode", json!(m));
        self.setup_act = 0;
        self.layout();
        self.b.play_sound("click");
    }

    fn set_undo(&mut self, u: &'static str) {
        if u == self.undo_rule {
            return;
        }
        self.undo_rule = u;
        self.save_setting("undo", json!(u));
        self.b.play_sound("select");
    }

    fn setup_step(&mut self, step: i64) {
        match self.setup_focus {
            0 => {
                let i = py::pmod(SIZES.iter().position(|s| *s == self.size).unwrap_or(1) as i64 + step, SIZES.len() as i64);
                self.set_size(SIZES[i as usize]);
            }
            1 => {
                let i = py::pmod(MODE_KEYS.iter().position(|s| *s == self.gmode).unwrap_or(0) as i64 + step, 3);
                self.set_mode(MODE_KEYS[i as usize]);
            }
            2 => {
                let i = py::pmod(UNDO_KEYS.iter().position(|s| *s == self.undo_rule).unwrap_or(1) as i64 + step, 3);
                self.set_undo(UNDO_KEYS[i as usize]);
            }
            _ => {
                if self.saved().is_some() {
                    self.setup_act = 1 - self.setup_act;
                    self.b.play_sound("move");
                }
            }
        }
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::MouseMove {
            self.hover = Some(ev.pos);
            return;
        }
        if ev.kind == Kind::MouseDown && ev.button == 1 {
            let p = ev.pos;
            if let Some(i) = self.size_rects.iter().position(|r| r.hit(p)) {
                self.setup_focus = 0;
                self.set_size(SIZES[i]);
                return;
            }
            if let Some(i) = self.mode_rects.iter().position(|r| r.hit(p)) {
                self.setup_focus = 1;
                self.set_mode(MODE_KEYS[i]);
                return;
            }
            if let Some(i) = self.undo_rects.iter().position(|r| r.hit(p)) {
                self.setup_focus = 2;
                self.set_undo(UNDO_KEYS[i]);
                return;
            }
            if self.saved().is_some() {
                if self.cont_rect.hit(p) {
                    self.setup_focus = 3;
                    self.setup_act = 0;
                    self.start(true);
                } else if self.new_rect.hit(p) {
                    self.setup_focus = 3;
                    self.setup_act = 1;
                    self.start(false);
                }
            } else if self.start_rect.hit(p) {
                self.start(false);
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        match ev.key.as_str() {
            k @ ("3" | "4" | "5" | "6" | "7" | "8") => {
                self.setup_focus = 0;
                self.set_size(k.parse().unwrap_or(4));
            }
            "Up" | "w" | "W" => {
                self.setup_focus = py::pmod(self.setup_focus - 1, 4);
                self.b.play_sound("move");
            }
            "Down" | "s" | "S" => {
                self.setup_focus = py::pmod(self.setup_focus + 1, 4);
                self.b.play_sound("move");
            }
            "Left" | "a" | "A" => self.setup_step(-1),
            "Right" | "d" | "D" => self.setup_step(1),
            "m" | "M" => {
                self.setup_focus = 1;
                self.setup_step(1);
            }
            "u" | "U" => {
                self.setup_focus = 2;
                self.setup_step(1);
            }
            "Return" | "space" | "KP_Enter" => {
                let resume = self.saved().is_some() && self.setup_act == 0;
                self.start(resume);
            }
            _ => {}
        }
    }

    // ----- Eingabe (Spiel)

    fn swipe_dir(&self, a: (i32, i32), b: (i32, i32)) -> Option<char> {
        let (dx, dy) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
        let need = 18f64.max(self.cell as f64 * 0.3);
        if dx.abs().max(dy.abs()) < need {
            return None;
        }
        if dx.abs() > dy.abs() * 1.2 {
            return Some(if dx > 0.0 { 'R' } else { 'L' });
        }
        if dy.abs() > dx.abs() * 1.2 {
            return Some(if dy > 0.0 { 'D' } else { 'U' });
        }
        None
    }

    fn button(&mut self, act: &str) {
        match act {
            "undo" => self.input("undo"),
            "new" => self.start(false),
            _ => self.to_setup(),
        }
    }

    fn input(&mut self, what: &str) {
        if what == "undo" {
            if self.slide.is_some() {
                if self.queue.len() < QUEUE_MAX {
                    self.queue.push(what.into());
                }
                return;
            }
            self.undo();
            return;
        }
        if self.state != State::Play || self.b.game_over || self.over_t.is_some() {
            return;
        }
        if self.slide.is_some() {
            if self.queue.len() < QUEUE_MAX {
                self.queue.push(what.into());
            }
            return;
        }
        self.do_move(what.chars().next().unwrap_or('L'));
    }

    // ----- Zuege

    fn do_move(&mut self, d: char) -> bool {
        let plan = plan_move(&self.grid, d);
        if !plan.moved {
            let vec = dir_info(d).2;
            self.nudge = Some((vec.0, vec.1, 0.0));
            return false;
        }
        let mut entry = Entry { grid: self.grid.clone(), points: self.points, moves: self.moves, reached: self.reached, slides: plan.slides.clone(), spawn: None };
        self.grid = plan.grid.clone();
        self.points += plan.gained;
        self.moves += 1;
        let spawn = spawn_tile(&mut self.grid, &mut self.rng);
        entry.spawn = spawn;
        if self.undo_rule != "off" {
            self.stack.push(entry);
            let cap = if self.undo_rule == "unlimited" { STACK_MAX } else { UNDO_LIMIT as usize };
            if self.stack.len() > cap {
                self.stack.remove(0);
            }
        }
        if self.gmode == "time" {
            self.clock_on = true;
        }
        self.pops.clear();
        self.spawns.clear();
        self.slide = Some(SlideAnim { t: 0.0, dur: if self.queue.is_empty() { SLIDE_T } else { SLIDE_FAST }, slides: plan.slides.clone(), merges: plan.merges.clone(), spawn, vanish: None, undo: false, flash: None });
        self.after_move(&plan);
        true
    }

    fn after_move(&mut self, plan: &Plan) {
        if plan.gained != 0 {
            self.popups.push((plan.gained, 0.0));
            let n = self.popups.len();
            if n > 3 {
                self.popups.drain(..n - 3);
            }
        }
        let top = plan.merges.iter().map(|m| m.2).max().unwrap_or(0);
        if top != 0 {
            self.b.play_sound("merge");
            if top >= 128 {
                self.tone(top);
            }
        } else {
            self.b.play_sound("move");
        }
        self.max_tile = max_tile(&self.grid);
        let new_peak = self.max_tile > self.peak;
        self.peak = self.peak.max(self.max_tile);
        if !self.reached && self.max_tile >= WIN_TILE {
            self.reached = true;
            if !self.assisted {
                if self.gmode != "time" {
                    self.b.report_result(true);
                }
                if self.size <= ACH_MAX_SIZE {
                    self.b.ach_event("tile_2048", None);
                }
            }
            if self.gmode == "classic" && !self.continued {
                self.win_pending = true;
            }
        }
        if self.max_tile >= 4096 && !self.assisted && self.size <= ACH_MAX_SIZE {
            self.b.ach_event("tile_4096", None);
        }
        if new_peak && self.max_tile >= WIN_TILE && !self.win_pending {
            if let Some(sl) = self.slide.as_mut() {
                sl.flash = Some(self.max_tile);
            }
        }
        self.note_best();
        self.sync_score();
        self.dirty = true;
        if !has_moves(&self.grid) {
            self.over_t = Some(OVER_DELAY);
            self.queue.clear();
        }
    }

    fn undo(&mut self) -> bool {
        if self.undo_rule == "off" {
            self.message(t("g2048.undo_disabled"));
            self.b.play_sound("hit");
            return false;
        }
        if self.stack.is_empty() {
            self.b.play_sound("hit");
            return false;
        }
        if self.undo_rule == "limited" && self.undo_left() <= 0 {
            self.message(t("g2048.undo_empty"));
            self.b.play_sound("hit");
            return false;
        }
        if self.b.game_over && self.over_reason == "time" {
            return false;
        }
        let e = self.stack.pop().expect("stack");
        self.undo_used += 1;
        self.assisted = true;
        self.grid = e.grid.clone();
        self.points = e.points;
        self.moves = e.moves;
        self.reached = e.reached;
        self.max_tile = max_tile(&self.grid);
        if self.b.game_over {
            self.b.resume_from_game_over();
        }
        self.over_t = None;
        self.win_pending = false;
        self.state = State::Play;
        self.pops.clear();
        self.spawns.clear();
        if !e.slides.is_empty() {
            let rev: Vec<Slide> = e.slides.iter().map(|&(fr, fc, tr, tc, v)| (tr, tc, fr, fc, v)).collect();
            self.slide = Some(SlideAnim { t: 0.0, dur: SLIDE_T, slides: rev, merges: Vec::new(), spawn: None, vanish: e.spawn, undo: true, flash: None });
        } else {
            let n = self.size as usize;
            self.spawns = (0..n).flat_map(|r| (0..n).map(move |c| (r, c))).filter(|&(r, c)| self.grid[r][c] != 0).map(|p| (p, 0.0)).collect();
        }
        self.note_best();
        self.sync_score();
        self.dirty = true;
        self.b.play_sound("select");
        true
    }

    fn set_anim(list: &mut Vec<((usize, usize), f64)>, k: (usize, usize), v: f64) {
        match list.iter_mut().find(|e| e.0 == k) {
            Some(e) => e.1 = v,
            None => list.push((k, v)),
        }
    }

    fn end_slide(&mut self) {
        let Some(sl) = self.slide.take() else {
            return;
        };
        if !sl.undo {
            for &(r, c, v) in &sl.merges {
                Self::set_anim(&mut self.pops, (r, c), 0.0);
                if v >= 128 {
                    let (x, y) = self.cell_pos(r, c);
                    let (cx, cy) = (x as f64 + self.cell as f64 / 2.0, y as f64 + self.cell as f64 / 2.0);
                    let n = 22.min(4 + (v as f64).log2() as usize);
                    self.burst(cx, cy, tile_color(v), n, self.cell as f64 * 2.6);
                    if v >= WIN_TILE {
                        self.rings.push((cx, cy, 0.0, tile_color(v)));
                    }
                }
            }
            if let Some(sp) = sl.spawn {
                Self::set_anim(&mut self.spawns, (sp.0, sp.1), 0.0);
            }
            if let Some(v) = sl.flash {
                self.flash = Some((v.to_string(), 0.0, tile_color(v)));
                self.b.play_sound("powerup");
            }
        }
        if self.win_pending {
            self.win_pending = false;
            self.state = State::Win;
            self.queue.clear();
            self.panel_ticks = pg::ticks() as i64;
            let br = self.board_rect;
            for _ in 0..3 {
                let x = br.centerx() as f64 + py::uniform(-br.w as f64 * 0.3, br.w as f64 * 0.3);
                let y = br.centery() as f64 + py::uniform(-br.h as f64 * 0.3, br.h as f64 * 0.3);
                self.burst(x, y, tile_color(WIN_TILE), 18, self.cell as f64 * 3.2);
            }
            self.b.play_sound("win");
            self.b.rumble(180);
        }
    }

    fn finish(&mut self, timeout: bool) {
        self.over_t = None;
        self.queue.clear();
        self.slide = None;
        self.pops.clear();
        self.spawns.clear();
        self.over_reason = if timeout { "time" } else { "stuck" };
        self.state = State::Play;
        self.win_pending = false;
        self.note_best();
        let start = if self.assisted { self.best_start.1 } else { self.best_start.0 };
        self.new_best = self.points > 0.max(start);
        if !self.assisted && self.gmode == "classic" && !self.reached {
            self.b.report_result(false);
        }
        self.data.saves.shift_remove(&self.key());
        self.persist();
        self.dirty = false;
        self.panel_ticks = pg::ticks() as i64;
        self.sync_score();
        self.b.game_over = true;
        self.b.play_sound(if timeout { "level" } else { "gameover" });
        self.b.rumble(220);
    }

    fn panel_action(&mut self, act: &str) {
        match act {
            "continue" => {
                self.continued = true;
                self.state = State::Play;
                self.dirty = true;
                self.b.play_sound("click");
            }
            "new" => self.start(false),
            "undo" => self.input("undo"),
            "setup" => self.to_setup(),
            _ => {}
        }
    }

    fn message(&mut self, text: String) {
        self.msg = Some((text, 0.0));
    }

    fn burst(&mut self, x: f64, y: f64, color: (i32, i32, i32), n: usize, speed: f64) {
        let light = ui::mix(color, (255, 255, 255), 0.35);
        let size = 1.5f64.max(self.cell as f64 / 34.0);
        for _ in 0..n {
            if self.particles.len() >= MAX_PARTICLES {
                break;
            }
            let a = py::uniform(0.0, TAU);
            let sp = speed * py::uniform(0.35, 1.0);
            let life = py::uniform(0.35, 0.7);
            let col = if py::random() < 0.5 { light } else { rgb(color.0, color.1, color.2) };
            let sz = size * py::uniform(0.7, 1.6);
            self.particles.push((x, y, a.cos() * sp, a.sin() * sp, 0.0, life, col, sz));
        }
    }

    // ----- Zeichnen

    fn fit_font(&self, text: &str, px: i32, max_w: f64, bold: bool, min_px: i32) -> Font {
        let mut size = px;
        let mut f = ui::font(size, bold, false);
        while size > min_px && f.size(text).0 as f64 > max_w {
            size -= 1;
            f = ui::font(size, bold, false);
        }
        f
    }

    fn tile_surface(&mut self, v: i64) -> Surface {
        let key = (v, self.cell);
        if let Some(s) = self.tile_cache.get(&key) {
            return s.clone();
        }
        let cell = self.cell;
        let col = tile_color(v);
        let rad = 3.max(cell / 10);
        let glow = v >= 128;
        let pad = if glow { 2.max(cell / 7) } else { 0 };
        let size = cell + 2 * pad;
        let mut surf = pg::surface_alpha(size, size);
        if glow {
            let strength = 1f64.min(((v as f64).log2() - 6.0) / 6.0);
            let mut g = pg::surface_alpha(size, size);
            let gcol = if tile_known(v).is_some() { col } else { COL_TEXT_BEYOND };
            draw::rect_r(&mut g, rgba(gcol.0, gcol.1, gcol.2, (50.0 + 130.0 * strength) as i32), Rect::new(pad / 2, pad / 2, cell + pad, cell + pad), 0, rad + pad / 2);
            let small = transform::smoothscale(&g, 1.max(size / 5), 1.max(size / 5));
            surf.bl(&transform::smoothscale(&small, size, size), (0, 0));
        }
        let depth = 1.max(cell / 28);
        draw::rect_r(&mut surf, shade(col, 0.78), Rect::new(pad, pad + depth, cell, cell - depth), 0, rad);
        draw::rect_r(&mut surf, col, Rect::new(pad, pad, cell, cell - depth), 0, rad);
        if tile_known(v).is_none() {
            draw::rect_r(&mut surf, COL_TEXT_BEYOND, Rect::new(pad, pad, cell, cell - depth), 1.max(cell / 30), rad);
        }
        let text = v.to_string();
        let px = 7.max((cell as f64 * digit_scale(text.len())) as i32);
        let fnt = self.fit_font(&text, px, cell as f64 * 0.84, true, 6);
        let img = fnt.render(&text, true, tile_text_color(v));
        surf.blr(&img, img.get_rect().center_at((pad as f64 + cell as f64 / 2.0, pad as f64 + (cell - depth) as f64 / 2.0)));
        if self.tile_cache.len() > 120 {
            self.tile_cache.clear();
        }
        self.tile_cache.insert(key, surf.clone());
        surf
    }

    fn blit_tile(&mut self, s: &mut Surface, v: i64, x: f64, y: f64, scale: f64) {
        if scale <= 0.03 {
            return;
        }
        let surf = self.tile_surface(v);
        let pad = (surf.w() - self.cell) / 2;
        if (scale - 1.0).abs() < 0.01 {
            s.bl(&surf, (x as i32 - pad, y as i32 - pad));
            return;
        }
        let sz = 1.max((surf.w() as f64 * scale) as i32);
        let img = transform::smoothscale(&surf, sz, sz);
        let c = self.cell as f64;
        s.bl(&img, ((x + c / 2.0 - sz as f64 / 2.0) as i32, (y + c / 2.0 - sz as f64 / 2.0) as i32));
    }

    fn board_surface(&mut self) -> (Surface, i32) {
        let br = self.board_rect;
        let style = ui::fx().style.to_string();
        let key = (br.w, self.size, self.cell, self.gap, style.clone());
        if let Some((k, s, p)) = &self.board_cache {
            if *k == key {
                return (s.clone(), *p);
            }
        }
        let pad = if style == "v1" { 0 } else { 4.max(br.w / 40) };
        let mut surf = pg::surface_alpha(br.w + 2 * pad, br.h + 2 * pad);
        let rad = 6.max(br.w / 45);
        if pad != 0 {
            let (sw, shh) = (surf.w(), surf.h());
            let mut sh = pg::surface_alpha(sw, shh);
            draw::rect_r(&mut sh, rgba(0, 0, 0, 110), Rect::new(pad / 2, pad, br.w + pad, br.h + pad / 2), 0, rad + pad / 2);
            let small = transform::smoothscale(&sh, 1.max(sw / 6), 1.max(shh / 6));
            surf.bl(&transform::smoothscale(&small, sw, shh), (0, 0));
        }
        draw::rect_r(&mut surf, COL_BOARD, Rect::new(pad, pad, br.w, br.h), 0, rad);
        let crad = 3.max(self.cell / 10);
        for r in 0..self.size as i32 {
            for c in 0..self.size as i32 {
                draw::rect_r(&mut surf, COL_EMPTY, Rect::new(pad + self.gap + c * (self.cell + self.gap), pad + self.gap + r * (self.cell + self.gap), self.cell, self.cell), 0, crad);
            }
        }
        self.board_cache = Some((key, surf.clone(), pad));
        (surf, pad)
    }

    fn icon(&self, s: &mut Surface, kind: &str, cx: i32, cy: i32, size: i32, color: Color) {
        let w = 2.max(size / 8);
        match kind {
            "undo" => {
                let r = size as f64 * 0.36;
                let rect = Rect::new(0, 0, (2.0 * r) as i32, (2.0 * r) as i32).center_at((cx, (cy as f64 + size as f64 * 0.06) as i32));
                draw::arc(s, color, rect, -PI / 3.0, PI, w);
                let a = size as f64 * 0.2;
                let (tx, ty) = (rect.x as f64 + w as f64 / 2.0, rect.centery() as f64);
                draw::polygon(s, color, &[(tx - a, ty - a * 0.2), (tx + a, ty - a * 0.2), (tx, ty + a)], 0);
            }
            "new" => {
                let ln = (size as f64 * 0.34) as i32;
                draw::rect_r(s, color, Rect::new(cx - ln, cy - w / 2, 2 * ln, w), 0, w / 2);
                draw::rect_r(s, color, Rect::new(cx - w / 2, cy - ln, w, 2 * ln), 0, w / 2);
            }
            _ => {
                let ln = (size as f64 * 0.36) as i32;
                for (i, knob) in [(-1, 0.35), (0, -0.3), (1, 0.1)] {
                    let yy = cy + i * (size as f64 * 0.26) as i32;
                    draw::line(s, color, (cx - ln, yy), (cx + ln, yy), 1.max(w / 2));
                    draw::circle(s, color, ((cx as f64 + knob * ln as f64) as i32, yy), 2.max(w), 0);
                }
            }
        }
    }

    fn infinity(&self, s: &mut Surface, right: i32, cy: i32, h: i32, color: Color) -> i32 {
        let r = 3.max((h as f64 * 0.26) as i32);
        let w = 1.max(h / 9);
        draw::circle(s, color, (right - r, cy), r, w);
        draw::circle(s, color, (right - 3 * r + w, cy), r, w);
        4 * r
    }

    fn draw_board(&mut self, s: &mut Surface) {
        let mut off = (0, 0);
        if let Some((dx, dy, tt)) = self.nudge {
            let amp = 3f64.max(self.cell as f64 * 0.07) * (PI * 1f64.min(tt / NUDGE_T)).sin();
            off = ((dx as f64 * amp) as i32, (dy as f64 * amp) as i32);
        }
        self.off = off;
        let (surf, pad) = self.board_surface();
        s.bl(&surf, (self.board_rect.x - pad + off.0, self.board_rect.y - pad + off.1));
        if let Some(sl) = self.slide.as_ref() {
            let p = ease_out_cubic(1f64.min(sl.t / sl.dur));
            let vanish = sl.vanish;
            let slides = sl.slides.clone();
            if let Some((r, c, v)) = vanish {
                let (x, y) = self.cell_pos(r, c);
                self.blit_tile(s, v, x as f64, y as f64, 1.0 - p);
            }
            for (fr, fc, tr, tc, v) in slides {
                let (x0, y0) = self.cell_pos(fr, fc);
                let (x1, y1) = self.cell_pos(tr, tc);
                self.blit_tile(s, v, x0 as f64 + (x1 - x0) as f64 * p, y0 as f64 + (y1 - y0) as f64 * p, 1.0);
            }
            self.off = (0, 0);
            return;
        }
        let mut popping = Vec::new();
        let n = self.size as usize;
        for r in 0..n {
            for c in 0..n {
                let v = self.grid[r][c];
                if v == 0 {
                    continue;
                }
                let (x, y) = self.cell_pos(r, c);
                if let Some(&(_, tt)) = self.spawns.iter().find(|e| e.0 == (r, c)) {
                    self.blit_tile(s, v, x as f64, y as f64, ease_out_back(0f64.max(1f64.min(tt / SPAWN_T))));
                    continue;
                }
                if let Some(&(_, tt)) = self.pops.iter().find(|e| e.0 == (r, c)) {
                    popping.push((v, x, y, 1.0 + 0.2 * (PI * 1f64.min(tt / POP_T)).sin()));
                    continue;
                }
                self.blit_tile(s, v, x as f64, y as f64, 1.0);
            }
        }
        for (v, x, y, scale) in popping {
            self.blit_tile(s, v, x as f64, y as f64, scale);
        }
        self.off = (0, 0);
    }

    fn card(&self, s: &mut Surface, rect: Rect, label: &str, value: &str, px: i32, color: Color) {
        let p = ui::pal();
        ui::draw_panel_ex(s, rect, None, None, Some(6.max(rect.h / 5)), false, None);
        let pad = self.pad;
        let lf = self.fit_font(label, self.lab_px, (rect.w - 2 * pad) as f64, false, 8);
        let im = lf.render(label, true, p.text_dim);
        s.bl(&im, (rect.x + pad, rect.y + pad));
        let lh = self.f_label.get_height();
        let vf = self.fit_font(value, px, (rect.w - 2 * pad) as f64, true, 8);
        let im = vf.render(value, true, color);
        let free_h = rect.h - 2 * pad - lh;
        s.bl(&im, (rect.x + pad, rect.y + pad + lh + 2.max(py::fdiv32(free_h - im.h(), 2) + 2)));
    }

    fn best_value(&self) -> i64 {
        let book = if self.assisted { &self.data.best_assisted } else { &self.data.best };
        book_get(book, &self.key())
    }

    fn blit_value(&self, s: &mut Surface, text: &str, right: i32, cy: i32, color: Color) -> i32 {
        let im = self.f_row_b.render(text, true, color);
        s.blr(&im, im.get_rect().midright_at((right, cy)));
        im.w()
    }

    fn draw_side(&self, s: &mut Surface) {
        let p = ui::pal();
        let sd = self.side_rect;
        let mut tag = Some(format!("{}×{} · {}", self.size, self.size, t(&format!("g2048.mode.{}", self.gmode))));
        let tag_col = ui::mix(self.b.accent, p.text, 0.25);
        if self.assisted {
            let base = tag.clone().unwrap_or_default();
            let full = format!("{} · {}", base, t("g2048.assisted"));
            if self.f_label.size(&full).0 <= sd.w - 4 {
                tag = Some(full);
            } else {
                let lh = self.f_label.get_height();
                let f = self.fit_font(&base, self.lab_px, (sd.w - lh - 10) as f64, false, 8);
                let img = f.render(&base, true, tag_col);
                s.blr(&img, img.get_rect().midleft_at((sd.x + 2, self.tag_rect.centery())));
                self.icon(s, "undo", sd.x + 2 + img.w() + 4 + lh / 2, self.tag_rect.centery(), lh, p.text_dim);
                tag = None;
            }
        }
        if let Some(tag) = tag {
            let f = self.fit_font(&tag, self.lab_px, (sd.w - 4) as f64, false, 8);
            let img = f.render(&tag, true, tag_col);
            s.blr(&img, img.get_rect().midleft_at((sd.x + 2, self.tag_rect.centery())));
        }
        self.card(s, self.score_rect, &t("g2048.hud.score"), &self.points.to_string(), self.big_px, p.text);
        self.card(s, self.best_rect, &t("g2048.hud.best"), &0.max(self.best_value()).to_string(), self.mid_px, p.gold);
        for &(gained, tt) in &self.popups {
            let pp = tt / POPUP_T;
            let mut img = self.f_row_b.render(&format!("+{gained}"), true, p.green);
            img.set_alpha(Some((255.0 * py::pow(1.0 - pp, 0.8)) as i32 as u8));
            let r = self.score_rect;
            let y = r.bottom() - self.pad - img.h() - (pp * r.h as f64 * 0.55) as i32;
            s.bl(&img, (r.right() - self.pad - img.w(), y));
        }
        let ir = self.info_rect;
        let pad = self.pad;
        ui::draw_panel_ex(s, ir, None, None, Some(6.max(self.row_h / 2)), false, None);
        let rows = [("moves", t("g2048.hud.moves")), ("max", t("g2048.hud.max")), ("undo", t("g2048.hud.undo")), ("time", t("g2048.hud.time"))];
        let fh = self.f_row.get_height();
        for (i, (kind, label)) in rows.iter().enumerate() {
            let cy = ir.y + pad + i as i32 * self.row_h + self.row_h / 2;
            let right = ir.right() - pad;
            let vw = match *kind {
                "moves" => self.blit_value(s, &self.moves.to_string(), right, cy, p.text),
                "max" => {
                    let mut vw = self.blit_value(s, &self.max_tile.to_string(), right, cy, p.text);
                    let sq = 6.max((fh as f64 * 0.62) as i32);
                    let rc = Rect::new(right - vw - sq - 5, cy - sq / 2, sq, sq);
                    draw::rect_r(s, tile_color(2.max(self.max_tile)), rc, 0, 2.max(sq / 4));
                    if tile_known(self.max_tile).is_none() && self.max_tile != 0 {
                        draw::rect_r(s, COL_TEXT_BEYOND, rc, 1, 2.max(sq / 4));
                    }
                    vw += sq + 5;
                    vw
                }
                "undo" => match self.undo_rule {
                    "off" => self.blit_value(s, "–", right, cy, p.text_faint),
                    "limited" => {
                        let left = self.undo_left();
                        self.blit_value(s, &format!("{}/{}", left, UNDO_LIMIT), right, cy, if left != 0 { p.text } else { p.text_faint })
                    }
                    _ => self.infinity(s, right, cy, fh, p.text),
                },
                _ => {
                    if self.gmode == "time" {
                        let mut col = p.text;
                        if self.time_left < 15.0 && self.clock_on && !self.b.game_over {
                            col = ui::mix(p.red, p.text, 0.4 * ui::pulse(8.0, 0.0, 1.0));
                        }
                        self.blit_value(s, &fmt_time(self.time_left), right, cy, col)
                    } else {
                        self.blit_value(s, &fmt_time(self.elapsed), right, cy, p.text_dim)
                    }
                }
            };
            let lf = self.fit_font(label, self.row_px, (ir.w - 2 * pad - vw - 6) as f64, false, 8);
            let im = lf.render(label, true, p.text_dim);
            s.blr(&im, im.get_rect().midleft_at((ir.x + pad, cy)));
        }
        let specs = [("undo", t("g2048.btn.undo"), "U"), ("new", t("g2048.btn.new"), "R"), ("setup", t("g2048.btn.setup"), "Tab")];
        let blocked = self.state == State::Win || self.b.game_over;
        for ((kind, label, cap), rc) in specs.iter().zip(&self.btn_rects) {
            let hov = !blocked && self.hover.is_some_and(|h| rc.hit(h));
            ui::draw_button_ex(s, *rc, "", &self.f_row, hov, None, None, Some(self.b.accent));
            let enabled = *kind != "undo" || self.can_undo();
            let col = if enabled { p.text } else { p.text_faint };
            let isz = 12.max((rc.h as f64 * 0.56) as i32);
            if self.btn_stacked {
                let ix = rc.x + pad + isz / 2 + 2;
                self.icon(s, kind, ix, rc.centery(), isz, col);
                let cap_img = self.f_label.render(cap, true, p.text_faint);
                let cap_w = cap_img.w() + 8;
                let avail = rc.w - (ix + isz / 2 + 8 - rc.x) - pad - cap_w;
                if avail > 20 {
                    let lf = self.fit_font(label, self.row_px, avail as f64, false, 8);
                    let im = lf.render(label, true, col);
                    s.blr(&im, im.get_rect().midleft_at((ix + isz / 2 + 8, rc.centery())));
                    s.blr(&cap_img, cap_img.get_rect().midright_at((rc.right() - pad, rc.centery())));
                }
            } else {
                self.icon(s, kind, rc.centerx(), rc.centery(), isz, col);
            }
        }
        let area = self.hint_area;
        if area.h > 0 {
            let hint = t("g2048.play_hint");
            let parts: Vec<String> = hint.split('·').map(|x| x.trim().to_string()).collect();
            let mut lines: Vec<String> = Vec::new();
            let mut cur = String::new();
            let fnt = &self.f_label;
            for part in &parts {
                let trial = if cur.is_empty() { part.clone() } else { format!("{cur} · {part}") };
                if fnt.size(&trial).0 <= area.w {
                    cur = trial;
                } else {
                    if !cur.is_empty() {
                        lines.push(cur.clone());
                    }
                    cur = part.clone();
                }
            }
            if !cur.is_empty() {
                lines.push(cur);
            }
            let lh = fnt.get_height() + 2;
            if !lines.is_empty() && lines.len() as i32 * lh <= area.h && lines.iter().all(|ln| fnt.size(ln).0 <= area.w) {
                let mut y = area.centery() - py::fdiv32(lines.len() as i32 * lh, 2);
                for ln in &lines {
                    let im = fnt.render(ln, true, p.text_faint);
                    s.blr(&im, im.get_rect().midtop_at((area.centerx(), y)));
                    y += lh;
                }
            }
        }
    }

    fn draw_timebar(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.time_rect;
        if r.h <= 0 {
            return;
        }
        let rad = r.h / 2;
        draw::rect_r(s, COL_EMPTY, r, 0, rad);
        let frac = 0f64.max(1f64.min(self.time_left / TIME_LIMIT));
        let mut col = if frac > 0.5 {
            p.green
        } else if frac > 0.2 {
            p.gold
        } else {
            p.red
        };
        if self.time_left < 15.0 && self.clock_on && !self.b.game_over {
            col = ui::mix(p.red, (255, 255, 255), 0.35 * ui::pulse(8.0, 0.0, 1.0));
        }
        let fw = (r.w as f64 * frac) as i32;
        if fw > 0 {
            draw::rect_r(s, col, Rect::new(r.x, r.y, fw.max(r.h), r.h), 0, rad);
        }
    }

    fn draw_effects(&self, s: &mut Surface) {
        let p = ui::pal();
        for &(x, y, _, _, age, life, col, size) in &self.particles {
            let k = 1.0 - age / life;
            draw::circle(s, col, (x as i32, y as i32), 1.max((size * (0.4 + 0.6 * k)) as i32), 0);
        }
        let br = self.board_rect;
        if !self.rings.is_empty() {
            s.set_clip(Some(br));
            for &(cx, cy, tt, col) in &self.rings {
                let pp = tt / 0.55;
                let rad = (self.cell as f64 * (0.55 + 0.45 * ease_out_cubic(pp))) as i32;
                let wid = 1.max((2.max(self.cell / 14) as f64 * (1.0 - pp)) as i32) + 1;
                draw::circle(s, ui::mix(ui::mix(col, (255, 255, 255), 0.3), COL_BOARD, pp), (cx as i32, cy as i32), rad, wid);
            }
            s.set_clip(None);
        }
        if let Some((text, tt, col)) = &self.flash {
            let px = 30.max((br.w as f64 * if text.chars().count() <= 4 { 0.2 } else { 0.15 }) as i32);
            let fnt = ui::fb(px);
            let img = fnt.render(text, true, *col);
            let shadow = fnt.render(text, true, rgb(0, 0, 0));
            let scale = ease_out_back(1f64.min(tt / 0.22));
            let alpha = if *tt < FLASH_T - 0.4 { 255.0 } else { (255.0 * 0f64.max((FLASH_T - tt) / 0.4)) as i32 as f64 };
            let rise = (0f64.max(tt - (FLASH_T - 0.4)) * br.h as f64 * 0.15) as i32;
            for (src, dx, dy, a) in [(&shadow, 3, 4, alpha * 0.55), (&img, 0, 0, alpha)] {
                let sz = (1.max((src.w() as f64 * scale) as i32), 1.max((src.h() as f64 * scale) as i32));
                let mut im = if scale != 1.0 { transform::smoothscale(src, sz.0, sz.1) } else { src.copy() };
                im.set_alpha(Some((a as i32).clamp(0, 255) as u8));
                s.blr(&im, im.get_rect().center_at((br.centerx() + dx, br.centery() + dy - rise)));
            }
        }
        if let Some((text, tt)) = &self.msg {
            let fnt = self.fit_font(text, self.row_px, (br.w - 40) as f64, false, 8);
            let im = fnt.render(text, true, p.text);
            let mut pill = im.get_rect().midtop_at((br.centerx(), br.y + 10.max(br.h / 14))).inflate(28, 12);
            let drop = (1f64.min(tt / 0.18) * 6.0) as i32;
            pill.y += drop - 6;
            ui::draw_panel_ex(s, pill, None, None, Some(pill.h / 2), false, None);
            s.blr(&im, im.get_rect().center_at(pill.center()));
        }
    }

    fn panel_geom(&self, win: bool) -> (Rect, Vec<i32>, Vec<(Rect, &'static str)>, i32, i32) {
        let br = self.board_rect;
        let h = self.b.height;
        let pad = 10.max(h / 42);
        let gap = 4.max(h / 110);
        let bh = 26.max(h / 17);
        let rowh = self.f_row.get_height();
        let (heights, acts): (Vec<i32>, Vec<&'static str>) = if win {
            (vec![self.f_title.get_height(), rowh, rowh], vec!["continue", "new"])
        } else {
            let mut a = vec!["new"];
            if self.can_undo() {
                a.push("undo");
            }
            a.push("setup");
            (vec![self.f_title.get_height(), self.f_big.get_height(), rowh, rowh], a)
        };
        let ph = 2 * pad + heights.iter().sum::<i32>() + gap * heights.len() as i32 + bh + gap + self.f_label.get_height();
        let pw = (br.w - 2 * 6.max(br.w / 30)).min(((br.w as f64 * 0.86) as i32).max(200));
        let rect = Rect::new(0, 0, pw, ph).center_at(br.center());
        let k = acts.len() as i32;
        let bw = py::fdiv32(pw - 2 * pad - (k - 1) * gap, k);
        let by = rect.bottom() - pad - self.f_label.get_height() - gap - bh;
        let btns = acts.iter().enumerate().map(|(i, a)| (Rect::new(rect.x + pad + i as i32 * (bw + gap), by, bw, bh), *a)).collect();
        (rect, heights, btns, pad, gap)
    }

    fn panel_click(&mut self, pos: (i32, i32)) -> bool {
        let (_, _, btns, _, _) = self.panel_geom(self.state == State::Win);
        for (rc, act) in btns {
            if rc.hit(pos) {
                self.panel_action(act);
                return true;
            }
        }
        false
    }

    fn font_px(&self, which: u8) -> i32 {
        match which {
            0 => 24.max(self.b.height / 13),
            1 => self.big_px,
            2 => self.mid_px,
            _ => self.row_px,
        }
    }

    fn draw_panel(&self, s: &mut Surface, win: bool) {
        let p = ui::pal();
        let br = self.board_rect;
        let since = (pg::ticks() as i64 - self.panel_ticks) as f64 / 1000.0;
        let k = ease_out_cubic(0f64.max(1f64.min(since / 0.3)));
        let mut dim = pg::surface_alpha(br.w, br.h);
        draw::rect_r(&mut dim, rgba(12, 14, 22, 165), Rect::new(0, 0, br.w, br.h), 0, 6.max(br.w / 45));
        dim.set_alpha(Some((255.0 * k) as i32 as u8));
        s.bl(&dim, (br.x, br.y));
        let (rect, heights, btns, pad, gap) = self.panel_geom(win);
        let lift = ((1.0 - k) * 18.0) as i32;
        let rect = rect.move_(0, lift);
        let frame = if win || self.over_reason == "time" { p.gold } else { p.red };
        let radius = 10.max(self.b.height / 40);
        ui::draw_panel_ex(s, rect, None, None, Some(radius), true, Some(self.b.accent));
        draw::rect_r(s, frame, rect, 2, radius);
        let inner = rect.w - 2 * pad;
        let mut y = rect.y + pad;
        let key = self.key();
        let lines: Vec<(String, u8, bool, Color)> = if win {
            vec![
                (t("g2048.reached"), 0, true, p.gold),
                (t("g2048.keep_going"), 3, false, p.text),
                (tf("g2048.stats", &[("score", &self.points.to_string()), ("moves", &self.moves.to_string())]), 3, false, p.text_dim),
            ]
        } else {
            let title = if self.over_reason == "time" { t("g2048.time_up") } else { t("common.game_over") };
            let best_line = if self.new_best {
                (t("g2048.new_best"), 3, true, ui::mix(p.gold, (255, 255, 255), 0.35 * ui::pulse(4.0, 0.0, 1.0)))
            } else {
                let book = if self.assisted { &self.data.best_assisted } else { &self.data.best };
                (tf("g2048.best_is", &[("score", &book_get(book, &key).to_string())]), 3, false, p.text_dim)
            };
            vec![
                (title, 0, true, frame),
                (self.points.to_string(), 1, true, if self.new_best { p.gold } else { p.text }),
                best_line,
                (tf("g2048.over_stats", &[("moves", &self.moves.to_string()), ("tile", &self.max_tile.to_string())]), 3, false, p.text_dim),
            ]
        };
        for ((text, fnt, bold, col), hh) in lines.iter().zip(&heights) {
            let f = self.fit_font(text, self.font_px(*fnt), inner as f64, *bold, 8);
            let im = f.render(text, true, *col);
            s.blr(&im, im.get_rect().center_at((rect.centerx(), y + hh / 2)));
            y += hh + gap;
        }
        let n_btn = btns.len();
        for (i, (rc, act)) in btns.iter().enumerate() {
            let rc = rc.move_(0, lift);
            let hov = self.hover.is_some_and(|h| rc.hit(h));
            let label = match *act {
                "continue" => t("g2048.btn.continue"),
                "new" => t(if n_btn > 2 { "g2048.btn.new" } else { "g2048.btn.new_game" }),
                "undo" => t("g2048.btn.undo"),
                _ => t("g2048.btn.setup"),
            };
            let f = self.fit_font(&label, self.row_px, (rc.w - 12) as f64, false, 8);
            ui::draw_button_ex(s, rc, &label, &f, i == 0 || hov, None, None, Some(self.b.accent));
        }
        let hint = t(if win { "g2048.win_hint" } else { "g2048.over_hint" });
        let f = self.fit_font(&hint, self.lab_px, inner as f64, false, 8);
        let col = ui::mix(p.text_faint, p.text_dim, ui::pulse(2.0, 0.0, 1.0));
        let im = f.render(&hint, true, col);
        s.blr(&im, im.get_rect().midbottom_at((rect.centerx(), rect.bottom() - pad + 2)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let sub = t("g2048.subtitle");
        ui::draw_title_ex(s, w, "2048", if self.setup_sub { Some(&sub) } else { None }, self.title_y, Some(&self.f_setup_title), Some(&self.small), Some(self.b.accent));
        let size_labels: Vec<String> = SIZES.iter().map(|n| format!("{n}×{n}")).collect();
        let mode_labels: Vec<String> = MODE_KEYS.iter().map(|m| t(&format!("g2048.mode.{m}"))).collect();
        let undo_labels: Vec<String> = UNDO_KEYS.iter().map(|u| t(&format!("g2048.undo.{u}"))).collect();
        let groups: [(&Vec<Rect>, &str, &Vec<String>, Vec<bool>); 3] = [
            (&self.size_rects, "g2048.lbl_size", &size_labels, SIZES.iter().map(|n| *n == self.size).collect()),
            (&self.mode_rects, "g2048.lbl_mode", &mode_labels, MODE_KEYS.iter().map(|m| *m == self.gmode).collect()),
            (&self.undo_rects, "g2048.lbl_undo", &undo_labels, UNDO_KEYS.iter().map(|u| *u == self.undo_rule).collect()),
        ];
        let btn_px = 12.max((self.b.height / 30).min((self.size_rects[0].h as f64 * 0.5) as i32));
        for (gi, (rects, lbl, labels, sel)) in groups.iter().enumerate() {
            let col = if self.setup_focus == gi as i64 { self.b.accent } else { p.text_dim };
            let im = self.tiny.render(&t(lbl), true, col);
            s.blr(&im, im.get_rect().midbottom_at((cx, rects[0].y - 3)));
            for ((rc, label), on) in rects.iter().zip(labels.iter()).zip(sel) {
                let f = self.fit_font(label, btn_px, (rc.w - 14) as f64, false, 8);
                ui::draw_button_ex(s, *rc, label, &f, *on, None, None, Some(self.b.accent));
            }
        }
        let ir = self.info_rect_setup;
        ui::draw_panel_ex(s, ir, None, None, Some(8.max(ir.h / 5)), false, None);
        let key = self.key();
        let best = book_get(&self.data.best, &key);
        let best_a = book_get(&self.data.best_assisted, &key);
        let tile = book_get(&self.data.best_tile, &key);
        let (line2, col2) = if best != 0 || best_a != 0 || tile != 0 {
            let mut l = tf("g2048.best_line", &[("score", &best.to_string()), ("tile", &tile.to_string())]);
            if best_a > best {
                l += &format!("  ·  {}", tf("g2048.best_undo", &[("score", &best_a.to_string())]));
            }
            (l, p.gold)
        } else {
            (t("g2048.no_best"), p.text_faint)
        };
        let (line3, col3) = if self.size == HS_SIZE && self.gmode == HS_MODE { (t("g2048.hs_yes"), ui::mix(p.green, p.text, 0.25)) } else { (t("g2048.hs_no"), p.text_faint) };
        let lines = [(t(&format!("g2048.mode_desc.{}", self.gmode)), p.text_dim), (line2, col2), (line3, col3)];
        let lh = self.tiny.get_height();
        let mut y = ir.y + self.info_pad;
        for (text, col) in &lines {
            let f = self.fit_font(text, 11.max(h / 40), (ir.w - 2 * self.info_pad) as f64, false, 8);
            let im = f.render(text, true, *col);
            s.blr(&im, im.get_rect().center_at((cx, y + lh / 2)));
            y += lh + 2;
        }
        let act_px = 13.max(h / 30);
        if let Some(sv) = self.saved() {
            let sub = tf("g2048.stats", &[("score", &sv.points.to_string()), ("moves", &sv.moves.to_string())]);
            for (i, (rc, label)) in [(self.cont_rect, t("g2048.btn.resume")), (self.new_rect, t("g2048.btn.new_game"))].iter().enumerate() {
                let f = self.fit_font(label, act_px, (rc.w - 20) as f64, i == 0, 8);
                let sf = self.fit_font(&sub, 11.max(h / 42), (rc.w - 20) as f64, false, 8);
                ui::draw_button_ex(s, *rc, label, &f, self.setup_act == i as i64, if i == 0 { Some(&sub) } else { None }, Some(&sf), Some(if i == 0 { p.green } else { self.b.accent }));
            }
        } else {
            let f = self.fit_font(&t("common.start"), act_px, (self.start_rect.w - 20) as f64, true, 8);
            ui::draw_button_ex(s, self.start_rect, &t("common.start"), &f, true, None, None, Some(p.green));
        }
        let hint = t("g2048.setup_hint");
        let hf = self.fit_font(&hint, 14, (w - 24) as f64, false, 8);
        ui::draw_footer_ex(s, w, h, &hint, Some(&hf));
    }

    fn age_ok(v: &mut f64, dt: f64, life: f64) -> bool {
        *v += dt;
        *v < life
    }
}

impl Game for Game2048 {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let size = self.b.opt_value(SECTION, "size");
        self.size = match size {
            Some(Value::Number(n)) => n.as_i64().filter(|v| SIZES.contains(v)).unwrap_or(4),
            Some(Value::Bool(true)) => 4,
            _ => 4,
        };
        let m = self.b.opt_value(SECTION, "mode").and_then(|v| v.as_str().map(String::from));
        self.gmode = MODE_KEYS.iter().find(|k| Some(**k) == m.as_deref()).copied().unwrap_or("classic");
        let u = self.b.opt_value(SECTION, "undo").and_then(|v| v.as_str().map(String::from));
        self.undo_rule = UNDO_KEYS.iter().find(|k| Some(**k) == u.as_deref()).copied().unwrap_or("limited");
        self.rng = py::Random::unseeded();
        self.data = Self::load_data();
        self.anim_t = 0.0;
        self.hover = None;
        self.drag = None;
        self.tile_cache.clear();
        self.board_cache = None;
        self.setup_focus = 0;
        self.setup_act = 0;
        self.blank_game();
        self.make_fonts();
        self.layout();
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
        self.build_setup_layout();
        self.particles.clear();
        self.rings.clear();
        self.drag = None;
    }

    fn on_exit(&mut self) {
        self.store_game();
        self.persist();
        self.dirty = false;
    }

    fn show_highscore_banner(&self) -> bool {
        self.counts()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                self.hover = Some(ev.pos);
                return;
            }
            Kind::MouseDown => {
                if ev.button != 1 {
                    return;
                }
                self.drag = None;
                if self.state == State::Win || self.b.game_over {
                    self.panel_click(ev.pos);
                    return;
                }
                for (i, act) in ["undo", "new", "setup"].iter().enumerate() {
                    if self.btn_rects.get(i).is_some_and(|r| r.hit(ev.pos)) {
                        self.button(act);
                        return;
                    }
                }
                self.drag = Some(ev.pos);
                return;
            }
            Kind::MouseUp => {
                let start = self.drag.take();
                let Some(start) = start else {
                    return;
                };
                if ev.button != 1 {
                    return;
                }
                if let Some(d) = self.swipe_dir(start, ev.pos) {
                    self.input(&d.to_string());
                }
                return;
            }
            _ => {}
        }
        if ev.kind != Kind::KeyDown || ev.repeat {
            return;
        }
        let key = ev.key.clone();
        let k = key.as_str();
        let undo_key = matches!(k, "u" | "U" | "BackSpace") && self.free(k);
        let new_key = matches!(k, "r" | "R" | "n" | "N") && self.free(k);
        let setup_key = k == "Tab" || (matches!(k, "s" | "S") && self.free(k));
        if self.b.game_over {
            if matches!(k, "Return" | "space" | "KP_Enter") || new_key {
                self.start(false);
            } else if undo_key {
                self.input("undo");
            } else if setup_key {
                self.to_setup();
            }
            return;
        }
        if self.state == State::Win {
            if matches!(k, "Return" | "space" | "KP_Enter") {
                self.panel_action("continue");
            } else if new_key {
                self.start(false);
            } else if undo_key {
                self.input("undo");
            } else if setup_key {
                self.to_setup();
            }
            return;
        }
        for d in ['L', 'R', 'U', 'D'] {
            let (arrow, action, _) = dir_info(d);
            if k == arrow || self.b.is_action(k, action, None) {
                self.input(&d.to_string());
                return;
            }
        }
        if undo_key {
            self.input("undo");
        } else if new_key {
            self.start(false);
        } else if setup_key {
            self.to_setup();
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if self.state == State::Setup {
            return;
        }
        if self.state == State::Play && self.over_t.is_none() {
            if self.moves > 0 {
                self.elapsed += dt;
            }
            if self.gmode == "time" && self.clock_on {
                self.time_left -= dt;
                if self.time_left <= 0.0 {
                    self.time_left = 0.0;
                    self.finish(true);
                    return;
                }
            }
        }
        let mut end = false;
        if let Some(sl) = self.slide.as_mut() {
            sl.t += dt;
            end = sl.t >= sl.dur;
        }
        if end {
            self.end_slide();
        }
        while self.slide.is_none() && !self.queue.is_empty() && self.state == State::Play && self.over_t.is_none() && !self.b.game_over {
            let nxt = self.queue.remove(0);
            if nxt == "undo" {
                self.undo();
            } else {
                self.do_move(nxt.chars().next().unwrap_or('L'));
            }
        }
        for (list, dur) in [(&mut self.pops, POP_T), (&mut self.spawns, SPAWN_T)] {
            for e in list.iter_mut() {
                e.1 += dt;
            }
            list.retain(|e| e.1 < dur);
        }
        if !self.particles.is_empty() {
            let grav = self.cell as f64 * 3.0;
            for p in self.particles.iter_mut() {
                p.4 += dt;
                if p.4 < p.5 {
                    p.0 += p.2 * dt;
                    p.1 += p.3 * dt;
                    p.2 *= 0.92;
                    p.3 = p.3 * 0.92 + grav * dt;
                }
            }
            self.particles.retain(|p| p.4 < p.5);
        }
        self.rings.retain_mut(|rg| Self::age_ok(&mut rg.2, dt, 0.55));
        self.popups.retain_mut(|pp| Self::age_ok(&mut pp.1, dt, POPUP_T));
        if let Some(f) = self.flash.as_mut() {
            if !Self::age_ok(&mut f.1, dt, FLASH_T) {
                self.flash = None;
            }
        }
        if let Some(m) = self.msg.as_mut() {
            if !Self::age_ok(&mut m.1, dt, MSG_T) {
                self.msg = None;
            }
        }
        if let Some(n) = self.nudge.as_mut() {
            if !Self::age_ok(&mut n.2, dt, NUDGE_T) {
                self.nudge = None;
            }
        }
        if self.over_t.is_some() && self.slide.is_none() && self.state == State::Play {
            let ot = self.over_t.unwrap_or(0.0) - dt;
            self.over_t = Some(ot);
            if ot <= 0.0 {
                self.finish(false);
                return;
            }
        }
        self.save_cd -= dt;
        if self.dirty && self.save_cd <= 0.0 {
            self.store_game();
            self.persist();
            self.dirty = false;
            self.save_cd = SAVE_EVERY;
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        self.draw_board(&mut s);
        self.draw_side(&mut s);
        if self.gmode == "time" {
            self.draw_timebar(&mut s);
        }
        self.draw_effects(&mut s);
        if self.state == State::Win {
            self.draw_panel(&mut s, true);
        } else if self.b.game_over {
            self.draw_panel(&mut s, false);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "2048",
    class_name: "Game2048",
    supports_multiplayer: false,
    modes: &[],
    make: Game2048::new,
};
