//! sudoku.rs
//! =========
//! Sudoku in vier Varianten (je 4 Stufen x 100 Level), Tages-Sudoku und vier
//! Spielmodi (classic/notes/comfort/assist).
//!
//! Uebertragen aus `games/sudoku.py` und `games/sudoku_draw.py` (Zeichnen);
//! die Raetsel kommen aus `sudoku_gen.rs`. Spielstand in mem.json (Abschnitt
//! "sudoku": solved/best/saves/last/daily), angefangene Raetsel werden
//! gesichert und fortgesetzt.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::f64::consts::PI;

use crate::sudoku_gen::{self as gen, Layout, VARIANTS};
use pgz_core::{seedrand, store};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const DIFFICULTIES: [&str; 4] = ["easy", "normal", "hard", "expert"];

fn base_points(v: &str) -> [i64; 4] {
    match v {
        "x" => [1200, 2400, 4000, 5600],
        "killer" => [1500, 3000, 5000, 7000],
        "mini" => [500, 900, 1400, 2000],
        _ => [1000, 2000, 3500, 5000],
    }
}

fn target_time(v: &str) -> [i64; 4] {
    match v {
        "x" => [360, 600, 900, 1320],
        "killer" => [600, 960, 1440, 1920],
        "mini" => [90, 150, 240, 360],
        _ => [300, 540, 840, 1200],
    }
}

fn mode_mult(m: &str) -> Option<f64> {
    match m {
        "classic" => Some(2.0),
        "notes" => Some(1.5),
        "comfort" => Some(1.0),
        "assist" => Some(0.7),
        _ => None,
    }
}

const MAX_HINTS: i64 = 3;
const HINT_COST: i64 = 200;
const ERR_COST: i64 = 150;
const TIME_COST: i64 = 2;
const FAIL_LIMIT: i64 = 3;
const MARK_COUNT: u8 = 6;
const MAX_SAVES: usize = 40;
const SAVE_EVERY: f64 = 4.0;
const MAX_UNDO: usize = 500;

const COL_USER: Color = Color::rgb(110, 165, 255);
const MARK_COLORS: [(i32, i32, i32); 6] = [(232, 92, 92), (240, 150, 60), (228, 204, 70), (96, 196, 118), (84, 160, 240), (176, 116, 226)];
const PAD_FUNCS: [&str; 6] = ["erase", "undo", "redo", "note", "cands", "hint"];

fn pad_tip(k: &str) -> Option<&'static str> {
    Some(match k {
        "erase" => "sud.tip.erase",
        "undo" => "sud.tip.undo",
        "redo" => "sud.tip.redo",
        "note" => "sud.tip.note",
        "cands" => "sud.tip.cands",
        "hint" => "sud.tip.hint",
        _ => return None,
    })
}

fn pad_key(k: &str) -> &'static str {
    match k {
        "erase" => "0",
        "undo" => "U",
        "redo" => "Y",
        "note" => "N",
        "cands" => "C",
        _ => "H",
    }
}

fn mv(k: &str) -> Option<(i64, i64)> {
    Some(match k {
        "Up" | "w" | "W" => (-1, 0),
        "Down" | "s" | "S" => (1, 0),
        "Left" | "a" | "A" => (0, -1),
        "Right" | "d" | "D" => (0, 1),
        _ => return None,
    })
}

/// `fmt_time(sec)`
pub fn fmt_time(sec: f64) -> String {
    let sec = 0.max(sec as i64);
    if sec >= 3600 {
        return format!("{}:{:02}:{:02}", sec / 3600, sec / 60 % 60, sec % 60);
    }
    format!("{:02}:{:02}", sec / 60, sec % 60)
}

fn stars_for(variant: &str, diff: i64, secs: i64, errors: i64, hints: i64) -> i64 {
    let clean = errors == 0 && hints == 0;
    let fast = secs <= target_time(variant)[diff.clamp(0, 3) as usize];
    1 + clean as i64 + (clean && fast) as i64
}

fn star_points(cx: f64, cy: f64, r: f64) -> Vec<(f64, f64)> {
    (0..10)
        .map(|k| {
            let rad = if k % 2 == 0 { r } else { r * 0.45 };
            let a = -PI / 2.0 + k as f64 * PI / 5.0;
            (cx + a.cos() * rad, cy + a.sin() * rad)
        })
        .collect()
}

fn draw_star(s: &mut Surface, center: (f64, f64), r: f64, color: Color, filled: bool) {
    let pts = star_points(center.0, center.1, r);
    if filled {
        draw::polygon(s, color, &pts, 0);
    } else {
        draw::polygon(s, color, &pts, 1.max(py::ffdiv(r, 5.0) as i32));
    }
}

fn draw_check(s: &mut Surface, center: (i32, i32), size: f64, color: Color, width: i32) {
    let (cx, cy) = (center.0 as f64, center.1 as f64);
    draw::lines(s, color, false, &[(cx - size, cy), (cx - size * 0.35, cy + size * 0.6), (cx + size, cy - size * 0.7)], width);
}

fn wrap(text: &str, fnt: &Font, width: i32) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let test = format!("{line} {word}").trim().to_string();
        if fnt.size(&test).0 <= width || line.is_empty() {
            line = test;
        } else {
            out.push(line);
            line = word.to_string();
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn fit(text: &str, width: i32, fonts: &[&Font], color: Color) -> Surface {
    for fnt in fonts {
        if fnt.size(text).0 <= width {
            return fnt.render(text, true, color);
        }
    }
    let fnt = fonts[fonts.len() - 1];
    let mut chars: Vec<char> = text.chars().collect();
    while chars.len() > 1 && fnt.size(&format!("{}...", chars.iter().collect::<String>())).0 > width {
        chars.pop();
    }
    let t: String = chars.iter().collect();
    fnt.render(&format!("{}...", t.trim_end()), true, color)
}

fn v_int(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.trunc() as i64)),
        Value::String(s) => s.trim().parse().ok(),
        Value::Bool(b) => Some(*b as i64),
        _ => None,
    }
}

/// Python: `isinstance(v, int) and not isinstance(v, bool)`
fn strict_int(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        _ => None,
    }
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Generating,
    Play,
}

#[derive(Clone, PartialEq)]
enum Hover {
    Setup(&'static str, usize),
    Pad(String),
}

type CellSt = (u8, u32, u8);
type Change = (usize, CellSt, CellSt);

/// `SudokuGame`
pub struct SudokuGame {
    b: GameBase,
    won: bool,
    diff: i64,
    fail_limit: bool,
    last_level: Map<String, Value>,
    variant: &'static str,
    input_digit: bool,
    colors_on: bool,
    can_notes: bool,
    can_check: bool,
    can_hint: bool,
    daily: bool,
    daily_date: String,
    play_variant: &'static str,
    play_diff: i64,
    level: i64,
    lay: &'static Layout,
    fresh: bool,
    hover: Option<Hover>,
    msg: Option<String>,
    msg_t: f64,
    msg_col: Option<Color>,
    gen_drawn: bool,
    // Schriften
    small: Font,
    tiny: Font,
    huge: Font,
    title: Font,
    clock_font: Font,
    lv_font: Font,
    num_font: Font,
    note_font: Font,
    sum_font: Font,
    pad_font: Font,
    // Fortschritt
    prog_extra: Map<String, Value>,
    solved: [Vec<i64>; 4],
    best: HashMap<String, HashMap<String, Vec<(String, [i64; 2])>>>,
    saves: Map<String, Value>,
    last: HashMap<String, Map<String, Value>>,
    daily_state: Map<String, Value>,
    cursor: i64,
    // Setup-Layout
    su_title_y: i32,
    su_sub_y: i32,
    var_rects: Vec<Rect>,
    diff_rects: Vec<Rect>,
    lv_cell: i32,
    lv_x: i32,
    lv_y: i32,
    prog_y: i32,
    panel_rect: Rect,
    daily_rect: Rect,
    limit_rect: Rect,
    input_rect: Rect,
    colors_rect: Rect,
    info_rect: Rect,
    start_rect: Option<Rect>,
    state: State,
    // Spiel
    n: usize,
    cells: usize,
    puzzle: Vec<u8>,
    solution: Vec<u8>,
    cages: Vec<(i64, Vec<usize>)>,
    cage_of: Vec<i64>,
    peers: Vec<Vec<usize>>,
    groups: Vec<Vec<usize>>,
    groups_of: Vec<Vec<usize>>,
    board: Vec<u8>,
    given: Vec<bool>,
    locked: Vec<bool>,
    notes: Vec<u32>,
    marks: Vec<u8>,
    wrong: HashSet<usize>,
    hinted: BTreeSet<usize>,
    sel: usize,
    errors: i64,
    hints_used: i64,
    elapsed: f64,
    mult: f64,
    note_mode: bool,
    brush: u8,
    mark_brush: u8,
    reveal: bool,
    undo_stack: Vec<(i64, Vec<Change>)>,
    redo_stack: Vec<(i64, Vec<Change>)>,
    step: Option<(i64, Vec<(usize, CellSt)>)>,
    dirty: bool,
    save_t: f64,
    conflicts: HashSet<usize>,
    bad_cages: HashSet<usize>,
    fx_pop: HashMap<usize, u64>,
    fx_shake: HashMap<usize, u64>,
    fx_units: Vec<(Vec<usize>, usize, u64)>,
    win_ticks: u64,
    star_snd: HashSet<usize>,
    result_stars: i64,
    result_time: i64,
    best_time: i64,
    new_record: bool,
    // Spiel-Layout
    hud_h: i32,
    cell: i32,
    bx: i32,
    by: i32,
    pad_rects: Vec<(String, Rect)>,
    pad_x: i32,
    pad_w: i32,
    caption_y: i32,
    caption_bottom: i32,
    // Caches
    cage_cache: Option<((i32, Color), Surface, Vec<(i32, i32)>)>,
    diag_cache: Option<((i32, Color), Surface)>,
}

impl SudokuGame {
    /// `SudokuGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let lay = gen::layout_for("classic");
        let mut g = SudokuGame {
            b,
            won: false,
            diff: 0,
            fail_limit: true,
            last_level: Map::new(),
            variant: "classic",
            input_digit: false,
            colors_on: false,
            can_notes: false,
            can_check: false,
            can_hint: false,
            daily: false,
            daily_date: seedrand::today_str(),
            play_variant: "classic",
            play_diff: 0,
            level: 1,
            lay,
            fresh: false,
            hover: None,
            msg: None,
            msg_t: 0.0,
            msg_col: None,
            gen_drawn: false,
            small: f.clone(),
            tiny: f.clone(),
            huge: f.clone(),
            title: f.clone(),
            clock_font: f.clone(),
            lv_font: f.clone(),
            num_font: f.clone(),
            note_font: f.clone(),
            sum_font: f.clone(),
            pad_font: f,
            prog_extra: Map::new(),
            solved: Default::default(),
            best: HashMap::new(),
            saves: Map::new(),
            last: HashMap::new(),
            daily_state: Map::new(),
            cursor: 1,
            su_title_y: 0,
            su_sub_y: 0,
            var_rects: Vec::new(),
            diff_rects: Vec::new(),
            lv_cell: 20,
            lv_x: 0,
            lv_y: 0,
            prog_y: 0,
            panel_rect: z,
            daily_rect: z,
            limit_rect: z,
            input_rect: z,
            colors_rect: z,
            info_rect: z,
            start_rect: None,
            state: State::Setup,
            n: 9,
            cells: 81,
            puzzle: vec![0; 81],
            solution: vec![0; 81],
            cages: Vec::new(),
            cage_of: vec![-1; 81],
            peers: Vec::new(),
            groups: Vec::new(),
            groups_of: Vec::new(),
            board: vec![0; 81],
            given: vec![false; 81],
            locked: vec![false; 81],
            notes: vec![0; 81],
            marks: vec![0; 81],
            wrong: HashSet::new(),
            hinted: BTreeSet::new(),
            sel: 0,
            errors: 0,
            hints_used: 0,
            elapsed: 0.0,
            mult: 1.0,
            note_mode: false,
            brush: 0,
            mark_brush: 0,
            reveal: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            step: None,
            dirty: false,
            save_t: 0.0,
            conflicts: HashSet::new(),
            bad_cages: HashSet::new(),
            fx_pop: HashMap::new(),
            fx_shake: HashMap::new(),
            fx_units: Vec::new(),
            win_ticks: 0,
            star_snd: HashSet::new(),
            result_stars: 0,
            result_time: 0,
            best_time: 0,
            new_record: false,
            hud_h: 40,
            cell: 20,
            bx: 0,
            by: 0,
            pad_rects: Vec::new(),
            pad_x: 0,
            pad_w: 0,
            caption_y: 0,
            caption_bottom: 0,
            cage_cache: None,
            diag_cache: None,
        };
        g.reset();
        Box::new(g)
    }

    fn mode(&self) -> &str {
        self.b.mode.as_str()
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 30));
        self.tiny = ui::f(11.max(h / 36));
        self.huge = ui::fb(26.max(h / 11));
        self.title = ui::fb(24.max(h / 13));
        self.clock_font = ui::font(18.max(h / 22), false, true);
    }

    fn reset_caches(&mut self) {
        self.cage_cache = None;
        self.diag_cache = None;
    }

    // ===================================================== Persistenz

    fn load_progress(&mut self) {
        let data = store::load_section("sudoku");
        let known = ["solved", "best", "saves", "last", "daily"];
        self.prog_extra = data.iter().filter(|(k, _)| !known.contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect();
        let solved = data.get("solved").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        for (k, slot) in self.solved.iter_mut().enumerate() {
            let mut set: Vec<i64> = match solved.get(&k.to_string()) {
                Some(Value::Array(lst)) => lst.iter().filter_map(strict_int).filter(|v| (1..=gen::LEVELS).contains(v)).collect(),
                _ => Vec::new(),
            };
            set.sort_unstable();
            set.dedup();
            *slot = set;
        }
        let raw_best = data.get("best").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        self.best.clear();
        for v in VARIANTS {
            let vb = raw_best.get(v).and_then(|x| x.as_object()).cloned().unwrap_or_default();
            let mut out_v = HashMap::new();
            for d in ["0", "1", "2", "3"] {
                let db = vb.get(d).and_then(|x| x.as_object()).cloned().unwrap_or_default();
                let mut out = Vec::new();
                for (key, rec) in &db {
                    let Ok(lvl) = key.trim().parse::<i64>() else {
                        continue;
                    };
                    let (Some(stars), Some(secs)) = (rec.get(0).and_then(v_int), rec.get(1).and_then(v_int)) else {
                        continue;
                    };
                    if (1..=gen::LEVELS).contains(&lvl) {
                        out.push((lvl.to_string(), [stars.clamp(1, 3), secs.max(0)]));
                    }
                }
                if !out.is_empty() {
                    out_v.insert(d.to_string(), out);
                }
            }
            self.best.insert(v.to_string(), out_v);
        }
        self.saves = data.get("saves").and_then(|v| v.as_object()).map(|m| m.iter().filter(|(_, v)| v.is_object()).map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
        let raw_last = data.get("last").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        self.last.clear();
        for v in VARIANTS {
            let lv = raw_last.get(v).and_then(|x| x.as_object()).cloned().unwrap_or_default();
            let m: Map<String, Value> = lv
                .iter()
                .filter(|(d, _)| ["0", "1", "2", "3"].contains(&d.as_str()))
                .filter_map(|(d, n)| strict_int(n).map(|n| (d.clone(), json!(n.clamp(1, 100)))))
                .collect();
            self.last.insert(v.to_string(), m);
        }
        let dl = data.get("daily").and_then(|v| v.as_object()).cloned().unwrap_or_default();
        self.daily_state = Map::new();
        if let Some(Value::String(d)) = dl.get("date") {
            if d.chars().count() == 10 {
                self.daily_state.insert("date".into(), json!(d));
            }
        }
        for k in ["streak", "best", "time", "stars", "count"] {
            if let Some(n) = dl.get(k).and_then(strict_int) {
                self.daily_state.insert(k.into(), json!(n.max(0)));
            }
        }
    }

    fn save_progress(&mut self) {
        let mut data = self.prog_extra.clone();
        let mut solved = Map::new();
        for (k, v) in self.solved.iter().enumerate() {
            solved.insert(k.to_string(), json!(v));
        }
        let mut best = Map::new();
        for v in VARIANTS {
            let mut out_v = Map::new();
            if let Some(vb) = self.best.get(v) {
                for d in ["0", "1", "2", "3"] {
                    if let Some(db) = vb.get(d) {
                        let mut m = Map::new();
                        for (lvl, rec) in db {
                            m.insert(lvl.clone(), json!(rec));
                        }
                        out_v.insert(d.into(), Value::Object(m));
                    }
                }
            }
            best.insert(v.into(), Value::Object(out_v));
        }
        let mut last = Map::new();
        for v in VARIANTS {
            last.insert(v.into(), Value::Object(self.last.get(v).cloned().unwrap_or_default()));
        }
        data.insert("solved".into(), Value::Object(solved));
        data.insert("best".into(), Value::Object(best));
        data.insert("saves".into(), Value::Object(self.saves.clone()));
        data.insert("last".into(), Value::Object(last));
        data.insert("daily".into(), Value::Object(self.daily_state.clone()));
        store::save_section("sudoku", data);
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("sudoku", key, value);
    }

    fn get_last(&self, variant: &str, diff: i64) -> i64 {
        let n = if variant == "classic" {
            self.last_level.get(&diff.to_string()).cloned().unwrap_or(json!(1))
        } else {
            self.last.get(variant).and_then(|m| m.get(&diff.to_string())).cloned().unwrap_or(json!(1))
        };
        match n {
            Value::Number(x) if x.is_i64() => x.as_i64().unwrap_or(1).clamp(1, 100),
            Value::Bool(b) => (b as i64).clamp(1, 100),
            _ => 1,
        }
    }

    fn set_last(&mut self, variant: &str, diff: i64, level: i64) {
        if variant == "classic" {
            self.last_level.insert(diff.to_string(), json!(level));
            let v = Value::Object(self.last_level.clone());
            self.save_setting("last_level", v);
        } else {
            self.last.entry(variant.to_string()).or_default().insert(diff.to_string(), json!(level));
        }
    }

    fn record(&self, variant: &str, diff: i64, level: i64) -> (i64, Option<i64>) {
        if let Some(rec) = self.best.get(variant).and_then(|vb| vb.get(&diff.to_string())).and_then(|db| db.iter().find(|(k, _)| *k == level.to_string())) {
            return (rec.1[0], Some(rec.1[1]));
        }
        if variant == "classic" && self.solved[diff as usize].contains(&level) {
            return (1, None);
        }
        (0, None)
    }

    fn stage_summary(&self, variant: &str, diff: i64) -> (i64, i64) {
        let (mut n, mut stars) = (0, 0);
        for lvl in 1..=gen::LEVELS {
            let (s, _) = self.record(variant, diff, lvl);
            if s != 0 {
                n += 1;
                stars += s;
            }
        }
        (n, stars)
    }

    fn three_star_count(&self) -> i64 {
        self.best.values().flat_map(|vb| vb.values()).flat_map(|db| db.iter()).filter(|(_, r)| r[0] >= 3).count() as i64
    }

    fn save_key(&self) -> String {
        if self.daily {
            return format!("daily:{}", self.daily_date);
        }
        format!("{}:{}:{}", self.play_variant, self.play_diff, self.level)
    }

    fn level_save(&self, variant: &str, diff: i64, level: i64) -> Option<&Value> {
        self.saves.get(&format!("{variant}:{diff}:{level}"))
    }

    fn daily_streak(&self) -> i64 {
        let Some(last) = self.daily_state.get("date").and_then(|v| v.as_str()) else {
            return 0;
        };
        if last.is_empty() {
            return 0;
        }
        let gap = seedrand::day_index(Some(&self.daily_date)) - seedrand::day_index(Some(last));
        if gap == 0 || gap == 1 {
            self.daily_state.get("streak").and_then(|v| v.as_i64()).unwrap_or(0)
        } else {
            0
        }
    }

    fn daily_done_today(&self) -> bool {
        self.daily_state.get("date").and_then(|v| v.as_str()) == Some(self.daily_date.as_str())
    }

    // ===================================================== Setup-Screen

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let m = 10.max(w / 48);
        let gap = 6.max(h / 90);
        self.su_title_y = 20.max((h as f64 * 0.06) as i32);
        self.su_sub_y = self.su_title_y + self.title.get_height() / 2 + self.tiny.get_height() / 2 + 1;
        let row_w = if w < 900 { w - 2 * m } else { (w as f64 * 0.8) as i32 };
        let left = cx - row_w / 2;
        let bh = 26.max(44.min(h / 15));
        let y0 = self.su_sub_y + self.tiny.get_height() / 2 + gap + 2;
        let row = |y: i32, n: i32| -> Vec<Rect> {
            let cw = (row_w - gap * (n - 1)) as f64 / n as f64;
            (0..n).map(|i| Rect::new((left as f64 + i as f64 * (cw + gap as f64)) as i32, y, cw as i32, bh)).collect()
        };
        self.var_rects = row(y0, 4);
        self.diff_rects = row(y0 + bh + gap, 4);
        let mut body_top = y0 + 2 * bh + 3 * gap;
        let foot = self.tiny.get_height() + 10;
        let prog_h = self.small.get_height() + 6;
        let avail_h = h - foot - body_top - prog_h;
        let grid = avail_h.min((row_w as f64 * 0.57) as i32);
        let cell = 18.max(grid / 10);
        body_top += 0.max((avail_h - 10 * cell) / 3);
        self.lv_cell = cell;
        self.lv_x = left;
        self.lv_y = body_top;
        self.lv_font = ui::font(10.max(cell * 2 / 5), false, true);
        self.prog_y = body_top + 10 * cell + prog_h / 2 + 1;
        let px = left + 10 * cell + 2 * gap;
        let pw = left + row_w - px;
        self.panel_rect = Rect::new(px, body_top, pw, 10 * cell);
        let pad = 6.max(gap);
        let (ix, iw) = (px + pad, pw - 2 * pad);
        let dh = self.small.get_height() + 2 * self.tiny.get_height() + 12;
        self.daily_rect = Rect::new(ix, body_top + pad, iw, dh);
        let th = 22.max(34.min(h / 22));
        let ty = self.daily_rect.bottom() + gap;
        self.limit_rect = Rect::new(ix, ty, iw, th);
        self.input_rect = Rect::new(ix, ty + th + 4, iw, th);
        self.colors_rect = Rect::new(ix, ty + 2 * (th + 4), iw, th);
        let info_top = self.colors_rect.bottom() + gap;
        self.info_rect = Rect::new(ix, info_top, iw, 0.max(self.panel_rect.bottom() - pad - info_top));
        let need = th + 4 + self.small.get_height() + 2 * self.tiny.get_height() + 8;
        if self.info_rect.h >= need {
            let bh2 = th.max(40.min(h / 18));
            self.start_rect = Some(Rect::new(ix, self.info_rect.bottom() - bh2, iw, bh2));
            self.info_rect.h -= bh2 + 6;
        } else {
            self.start_rect = None;
        }
    }

    fn level_at(&self, pos: (i32, i32)) -> Option<i64> {
        let c = py::fdiv32(pos.0 - self.lv_x, self.lv_cell);
        let r = py::fdiv32(pos.1 - self.lv_y, self.lv_cell);
        if (0..10).contains(&c) && (0..10).contains(&r) {
            return Some((r * 10 + c + 1) as i64);
        }
        None
    }

    fn select_variant(&mut self, v: &'static str) {
        self.variant = v;
        self.save_setting("variant", json!(v));
        self.cursor = self.get_last(v, self.diff);
        self.b.play_sound("click");
    }

    fn select_difficulty(&mut self, i: i64) {
        self.diff = i.clamp(0, 3);
        self.save_setting("difficulty", json!(self.diff));
        self.cursor = self.get_last(self.variant, self.diff);
        self.b.play_sound("click");
    }

    fn toggle_fail_limit(&mut self) {
        self.fail_limit = !self.fail_limit;
        self.save_setting("fail_limit", json!(self.fail_limit));
        self.b.play_sound("select");
    }

    fn toggle_input(&mut self) {
        self.input_digit = !self.input_digit;
        self.save_setting("input", json!(if self.input_digit { "digit" } else { "cell" }));
        self.b.play_sound("select");
    }

    fn toggle_colors(&mut self) {
        self.colors_on = !self.colors_on;
        self.save_setting("colors", json!(self.colors_on));
        self.b.play_sound("select");
    }

    fn setup_hit(&self, pos: (i32, i32)) -> Option<Hover> {
        if let Some(i) = self.var_rects.iter().position(|r| r.hit(pos)) {
            return Some(Hover::Setup("variant", i));
        }
        if let Some(i) = self.diff_rects.iter().position(|r| r.hit(pos)) {
            return Some(Hover::Setup("diff", i));
        }
        for (key, r) in [("daily", Some(self.daily_rect)), ("limit", Some(self.limit_rect)), ("input", Some(self.input_rect)), ("colors", Some(self.colors_rect)), ("start", self.start_rect)] {
            if let Some(r) = r {
                if r.hit(pos) {
                    return Some(Hover::Setup(key, 0));
                }
            }
        }
        None
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" | "4" => self.select_difficulty(k.parse::<i64>().unwrap_or(1) - 1),
                "v" | "V" if self.b.key_is_free(k) => {
                    let step: i64 = if k == "V" { -1 } else { 1 };
                    let i = VARIANTS.iter().position(|v| *v == self.variant).unwrap_or(0) as i64;
                    self.select_variant(VARIANTS[py::pmod(i + step, VARIANTS.len() as i64) as usize]);
                }
                "f" | "F" if self.b.key_is_free(k) => self.toggle_fail_limit(),
                "i" | "I" if self.b.key_is_free(k) => self.toggle_input(),
                "m" | "M" if self.b.key_is_free(k) => self.toggle_colors(),
                "t" | "T" if self.b.key_is_free(k) => self.start_daily(false),
                "Left" | "a" | "A" => {
                    self.cursor = py::pmod(self.cursor - 2, 100) + 1;
                    self.b.play_sound("move");
                }
                "Right" | "d" | "D" => {
                    self.cursor = py::pmod(self.cursor, 100) + 1;
                    self.b.play_sound("move");
                }
                "Up" | "w" | "W" => {
                    self.cursor = py::pmod(self.cursor - 11, 100) + 1;
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.cursor = py::pmod(self.cursor + 9, 100) + 1;
                    self.b.play_sound("move");
                }
                "Return" | "space" | "KP_Enter" => {
                    let c = self.cursor;
                    self.start_level(c, false);
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseMove {
            if let Some(lv) = self.level_at(ev.pos) {
                self.cursor = lv;
            }
            self.hover = self.setup_hit(ev.pos);
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            if let Some(Hover::Setup(kind, i)) = self.setup_hit(ev.pos) {
                match kind {
                    "variant" => self.select_variant(VARIANTS[i]),
                    "diff" => self.select_difficulty(i as i64),
                    "daily" => self.start_daily(false),
                    "limit" => self.toggle_fail_limit(),
                    "input" => self.toggle_input(),
                    "colors" => self.toggle_colors(),
                    _ => {
                        let c = self.cursor;
                        self.start_level(c, false);
                    }
                }
                return;
            }
            if let Some(lv) = self.level_at(ev.pos) {
                self.start_level(lv, false);
            }
        }
    }

    // ===================================================== Level starten

    fn begin_generate(&mut self) {
        self.b.game_over = false;
        self.won = false;
        self.b.score = 0;
        self.reveal = false;
        self.hover = None;
        self.gen_drawn = false;
        self.state = State::Generating;
        self.b.play_sound("click");
    }

    fn start_level(&mut self, n: i64, fresh: bool) {
        if self.variant == "killer" && gen::killer_level(self.diff, 1).is_none() {
            self.say(t("sud.killer_missing"), Some(ui::pal().red), 2.5);
            self.b.play_sound("hit");
            return;
        }
        self.daily = false;
        self.play_variant = self.variant;
        self.play_diff = self.diff;
        self.level = n.clamp(1, gen::LEVELS);
        self.cursor = self.level;
        let (v, d, l) = (self.variant, self.diff, self.level);
        self.set_last(v, d, l);
        self.fresh = fresh;
        self.begin_generate();
    }

    fn start_daily(&mut self, fresh: bool) {
        self.daily = true;
        self.daily_date = seedrand::today_str();
        self.play_variant = "classic";
        self.play_diff = gen::daily_diff(&self.daily_date);
        self.level = 0;
        self.fresh = fresh;
        self.begin_generate();
    }

    fn restart(&mut self) {
        if self.daily {
            self.start_daily(true);
        } else {
            self.variant = self.play_variant;
            self.diff = self.play_diff;
            let l = self.level;
            self.start_level(l, true);
        }
    }

    fn back_to_setup(&mut self) {
        if self.state == State::Play && !self.b.game_over {
            self.write_save();
        }
        self.state = State::Setup;
        self.b.game_over = false;
        self.hover = None;
        self.msg = None;
        self.daily_date = seedrand::today_str();
        if !self.daily {
            self.variant = self.play_variant;
            self.diff = self.play_diff;
            self.cursor = self.level;
        }
        self.b.play_sound("click");
    }

    fn do_generate(&mut self) {
        let v = self.play_variant;
        let (puzzle, solution, cages): (Vec<u8>, Vec<u8>, Vec<(i64, Vec<usize>)>) = if self.daily {
            let (p, s, d) = gen::generate_daily(&self.daily_date);
            self.play_diff = d;
            (p, s, Vec::new())
        } else if v == "classic" {
            let (p, s) = gen::generate(self.play_diff, self.level);
            (p, s, Vec::new())
        } else if v == "killer" {
            match gen::killer_level(self.play_diff, self.level) {
                Some((p, s, c)) => (p.clone(), s.clone(), c.clone()),
                None => {
                    self.state = State::Setup;
                    self.say(t("sud.killer_missing"), Some(ui::pal().red), 2.5);
                    return;
                }
            }
        } else {
            let (p, s) = gen::generate_variant(v, self.play_diff, self.level);
            (p, s, Vec::new())
        };
        self.lay = gen::layout_for(if self.daily { "classic" } else { v });
        let lay = self.lay;
        self.n = lay.n;
        self.cells = lay.size;
        self.puzzle = puzzle.clone();
        self.solution = solution;
        self.cages = cages;
        self.cage_of = vec![-1; self.cells];
        for (ci, (_, cells)) in self.cages.iter().enumerate() {
            for &i in cells {
                self.cage_of[i] = ci as i64;
            }
        }
        self.peers = (0..self.cells)
            .map(|i| {
                let mut ps: BTreeSet<usize> = lay.peers[i].iter().copied().collect();
                if self.cage_of[i] >= 0 {
                    ps.extend(self.cages[self.cage_of[i] as usize].1.iter().copied());
                    ps.remove(&i);
                }
                ps.into_iter().collect()
            })
            .collect();
        self.groups = lay.units.clone();
        self.groups.extend(self.cages.iter().map(|(_, c)| c.clone()));
        self.groups_of = vec![Vec::new(); self.cells];
        for (g, cells) in self.groups.iter().enumerate() {
            for &i in cells {
                self.groups_of[i].push(g);
            }
        }
        self.board = puzzle.clone();
        self.given = puzzle.iter().map(|v| *v != 0).collect();
        self.locked = self.given.clone();
        self.notes = vec![0; self.cells];
        self.marks = vec![0; self.cells];
        self.wrong.clear();
        self.hinted.clear();
        self.sel = (0..self.cells).find(|&i| !self.given[i]).unwrap_or(0);
        self.errors = 0;
        self.hints_used = 0;
        self.elapsed = 0.0;
        self.mult = mode_mult(self.mode()).unwrap_or(1.0);
        self.note_mode = false;
        self.brush = 0;
        self.mark_brush = 0;
        self.reveal = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.step = None;
        self.dirty = false;
        self.save_t = 0.0;
        self.conflicts.clear();
        self.bad_cages.clear();
        self.fx_pop.clear();
        self.fx_shake.clear();
        self.fx_units.clear();
        self.win_ticks = 0;
        self.star_snd.clear();
        self.result_stars = 0;
        self.new_record = false;
        if self.fresh {
            if self.saves.shift_remove(&self.save_key()).is_some() {
                self.save_progress();
            }
        } else if self.restore_save() {
            self.say(t("sud.resumed"), Some(self.b.accent), 2.5);
        }
        self.fresh = false;
        self.update_conflicts();
        self.reset_caches();
        self.build_play_layout();
        self.state = State::Play;
    }

    // ----- Spielstand sichern / fortsetzen

    fn has_progress(&self) -> bool {
        self.errors != 0 || self.hints_used != 0 || self.notes.iter().any(|n| *n != 0) || self.marks.iter().any(|m| *m != 0) || self.board.iter().zip(&self.puzzle).any(|(b, p)| b != p)
    }

    fn write_save(&mut self) {
        self.dirty = false;
        self.save_t = 0.0;
        let key = self.save_key();
        if !self.has_progress() {
            if self.saves.shift_remove(&key).is_some() {
                self.save_progress();
            }
            return;
        }
        let todo = 1.max(self.puzzle.iter().filter(|p| **p == 0).count());
        let filled = (0..self.cells).filter(|&i| !self.given[i] && self.board[i] != 0).count();
        let mut rec = Map::new();
        rec.insert("b".into(), json!(self.board.iter().map(|v| v.to_string()).collect::<String>()));
        rec.insert("n".into(), json!(self.notes));
        rec.insert("k".into(), json!(self.marks.iter().map(|v| v.to_string()).collect::<String>()));
        rec.insert("l".into(), json!((0..self.cells).filter(|&i| self.locked[i] && !self.given[i]).collect::<Vec<_>>()));
        rec.insert("h".into(), json!(self.hinted.iter().copied().collect::<Vec<_>>()));
        rec.insert("e".into(), json!(self.errors));
        rec.insert("hu".into(), json!(self.hints_used));
        rec.insert("t".into(), json!(py::round_to(self.elapsed, 1)));
        rec.insert("m".into(), json!(self.mult));
        rec.insert("p".into(), json!((100.0 * filled as f64 / todo as f64) as i64));
        rec.insert("ts".into(), json!(py::time_time() as i64));
        self.saves.insert(key.clone(), Value::Object(rec));
        if self.saves.len() > MAX_SAVES {
            let mut old: Vec<(i64, String)> = self.saves.iter().filter(|(k, _)| **k != key).map(|(k, v)| (v.get("ts").and_then(strict_int).unwrap_or(0), k.clone())).collect();
            old.sort();
            let n = self.saves.len() - MAX_SAVES;
            for (_, k) in old.into_iter().take(n) {
                self.saves.shift_remove(&k);
            }
        }
        self.save_progress();
    }

    fn restore_save(&mut self) -> bool {
        let Some(raw) = self.saves.get(&self.save_key()).and_then(|v| v.as_object()).cloned() else {
            return false;
        };
        let (n, size) = (self.n as u8, self.cells);
        let Some(b) = raw.get("b").and_then(|v| v.as_str()) else {
            return false;
        };
        let board: Option<Vec<u8>> = b.chars().map(|ch| ch.to_digit(10).map(|d| d as u8)).collect();
        let Some(board) = board else {
            return false;
        };
        if board.len() != size || board.iter().any(|v| *v > n) {
            return false;
        }
        if (0..size).any(|i| self.given[i] && board[i] != self.puzzle[i]) {
            return false;
        }
        let mut notes: Vec<u32> = match raw.get("n") {
            Some(Value::Array(a)) if a.len() == size => {
                let v: Option<Vec<u32>> = a.iter().map(|x| v_int(x).map(|i| (i as u32) & self.lay.all)).collect();
                match v {
                    Some(v) => v,
                    None => return false,
                }
            }
            _ => vec![0; size],
        };
        let marks_s = raw.get("k").and_then(|v| v.as_str()).unwrap_or("");
        let marks: Option<Vec<u8>> = marks_s.chars().map(|ch| ch.to_digit(10).map(|d| d as u8)).collect();
        // int(ch) auf einem Nicht-Ziffern-Zeichen wirft in Python -> kein Fortsetzen
        let Some(mut marks) = marks else {
            return false;
        };
        if marks.len() != size || marks.iter().any(|v| *v > MARK_COUNT) {
            marks = vec![0; size];
        }
        let ints = |key: &str| -> Vec<usize> { raw.get(key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(v_int).filter(|i| *i >= 0 && (*i as usize) < size).map(|i| i as usize).collect()).unwrap_or_default() };
        let locked = ints("l");
        let hinted = ints("h");
        let errors = raw.get("e").and_then(v_int).unwrap_or(0).max(0);
        let hints = raw.get("hu").and_then(v_int).unwrap_or(0).clamp(0, MAX_HINTS);
        let elapsed = raw.get("t").and_then(|v| v.as_f64()).unwrap_or(0.0).max(0.0);
        let mult = raw.get("m").and_then(|v| v.as_f64()).unwrap_or(self.mult);
        if self.fail_limit && errors >= FAIL_LIMIT {
            return false;
        }
        self.board = board;
        for i in 0..size {
            if self.board[i] != 0 {
                notes[i] = 0;
            }
        }
        self.notes = notes;
        self.marks = marks;
        self.wrong = (0..size).filter(|&i| self.board[i] != 0 && self.board[i] != self.solution[i]).collect();
        for i in locked {
            if self.board[i] != 0 && self.board[i] == self.solution[i] {
                self.locked[i] = true;
            }
        }
        self.hinted = hinted.into_iter().filter(|&i| self.locked[i] && !self.given[i]).collect();
        self.errors = errors;
        self.hints_used = hints;
        self.elapsed = elapsed;
        if 0.0 < mult && mult <= 2.0 {
            self.mult = self.mult.min(mult);
        }
        self.sel = (0..size).find(|&i| self.board[i] == 0).unwrap_or(self.sel);
        true
    }

    // ===================================================== Spiel-Layout

    fn build_play_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let n = self.lay.n as i32;
        self.hud_h = 40.max(h / 12);
        let foot = self.tiny.get_height() + 12;
        let pad_w = 128.max(w / 5);
        let size = (h - self.hud_h - foot - 12).min(w - pad_w - 44);
        self.cell = 20.max(py::fdiv32(size, n));
        let bs = self.cell * n;
        self.bx = 12.max(py::fdiv32(w - pad_w - 24 - bs, 2));
        self.by = self.hud_h + 6.max(py::fdiv32(h - self.hud_h - foot - bs, 2));
        let cell = self.cell;
        self.num_font = ui::font(14.max(cell * 3 / 5), true, true);
        self.note_font = if !self.cages.is_empty() { ui::font(7.max(cell * 2 / 9), false, true) } else { ui::font(8.max(cell * 2 / 7), false, true) };
        self.sum_font = ui::fb(8.max(cell / 4));
        self.pad_font = ui::font(14.max((cell * 3 / 5).min(44)), true, true);
        let px = self.bx + bs + 24;
        let pb = 24.max(py::fdiv32(w - px - 12 - 8, 3).min(cell + 8));
        let fw = 3 * pb + 8;
        let fh = 22.max(pb * 2 / 3);
        let rows = (n + 2) / 3;
        let mut func_rows: Vec<Vec<&str>> = vec![vec!["erase", "undo", "redo"]];
        let second: Vec<&str> = [("note", self.can_notes), ("cands", self.can_notes), ("hint", self.can_hint)].iter().filter(|(_, ok)| *ok).map(|(k, _)| *k).collect();
        if !second.is_empty() {
            func_rows.push(second);
        }
        let sh = if self.colors_on { 14.max(fh * 2 / 3) } else { 0 };
        let cap_h = 2 * self.tiny.get_height() + 6;
        let total = rows * (pb + 4) + 4 + func_rows.len() as i32 * (fh + 4) + if sh != 0 { sh + 6 } else { 0 } + cap_h;
        let py_ = self.by + 0.max(py::fdiv32(bs - total, 2));
        self.pad_rects.clear();
        for d in 1..=n {
            let (r, c) = ((d - 1) / 3, (d - 1) % 3);
            self.pad_rects.push((d.to_string(), Rect::new(px + c * (pb + 4), py_ + r * (pb + 4), pb, pb)));
        }
        let mut y = py_ + rows * (pb + 4) + 4;
        for keys in &func_rows {
            for (c, key) in keys.iter().enumerate() {
                self.pad_rects.push((key.to_string(), Rect::new(px + c as i32 * (pb + 4), y, pb, fh)));
            }
            y += fh + 4;
        }
        if sh != 0 {
            y += 2;
            let sw = py::fdiv32(fw - (MARK_COUNT as i32 - 1) * 3, MARK_COUNT as i32);
            for k in 0..MARK_COUNT as i32 {
                self.pad_rects.push((format!("mark{}", k + 1), Rect::new(px + k * (sw + 3), y, sw, sh)));
            }
            y += sh + 4;
        }
        self.pad_x = px;
        self.pad_w = fw;
        self.caption_y = y + 2;
        self.caption_bottom = (h - foot).min(y + 2 + cap_h);
    }

    fn cell_at(&self, pos: (i32, i32)) -> Option<usize> {
        let c = py::fdiv32(pos.0 - self.bx, self.cell);
        let r = py::fdiv32(pos.1 - self.by, self.cell);
        let n = self.n as i32;
        if (0..n).contains(&c) && (0..n).contains(&r) {
            return Some((r * n + c) as usize);
        }
        None
    }

    fn pad_at(&self, pos: (i32, i32)) -> Option<String> {
        self.pad_rects.iter().find(|(_, r)| r.hit(pos)).map(|(k, _)| k.clone())
    }

    fn pad_rect(&self, key: &str) -> Option<Rect> {
        self.pad_rects.iter().find(|(k, _)| k == key).map(|(_, r)| *r)
    }

    // ===================================================== Eingabe

    fn pad_action(&mut self, key: &str) {
        match key {
            "erase" => self.erase(self.sel),
            "undo" => self.undo(),
            "redo" => self.redo(),
            "note" => self.toggle_note_mode(),
            "cands" => self.auto_candidates(),
            "hint" => self.use_hint(),
            _ => {}
        }
    }

    fn handle_play_key(&mut self, k: &str) {
        let n = self.n as i64;
        if let Some((dr, dc)) = mv(k) {
            let (r, c) = (self.sel as i64 / n, self.sel as i64 % n);
            self.sel = (py::pmod(r + dr, n) * n + py::pmod(c + dc, n)) as usize;
            self.b.play_sound("move");
            return;
        }
        let chars: Vec<char> = k.chars().collect();
        let digit: Option<i64> = if chars.len() == 1 && chars[0].is_ascii_digit() {
            chars[0].to_digit(10).map(|d| d as i64)
        } else if k.starts_with("KP_") && chars.len() == 4 && chars[3].is_ascii_digit() {
            chars[3].to_digit(10).map(|d| d as i64)
        } else {
            None
        };
        if let Some(d) = digit.filter(|d| (1..=n).contains(d)) {
            if self.input_digit {
                let nb = if self.brush as i64 == d { 0 } else { d as u8 };
                self.set_brush(nb);
            } else {
                self.enter_digit(self.sel, d as u8);
            }
        } else if digit == Some(0) || k == "BackSpace" || k == "Delete" {
            self.erase(self.sel);
        } else if matches!(k, "space" | "Return" | "KP_Enter") {
            if self.input_digit && self.brush != 0 {
                self.apply_brush(self.sel);
            }
        } else if matches!(k, "n" | "N") {
            self.toggle_note_mode();
        } else if matches!(k, "c" | "C") && self.b.key_is_free(k) {
            self.auto_candidates();
        } else if matches!(k, "u" | "U" | "z" | "Z") && self.b.key_is_free(k) {
            self.undo();
        } else if matches!(k, "y" | "Y") && self.b.key_is_free(k) {
            self.redo();
        } else if matches!(k, "m" | "M") && self.b.key_is_free(k) {
            if self.colors_on {
                let c = (self.marks[self.sel] + 1) % (MARK_COUNT + 1);
                self.set_mark(self.sel, c, false);
            }
        } else if matches!(k, "h" | "H") {
            self.use_hint();
        } else if matches!(k, "r" | "R") {
            self.restart();
        } else if matches!(k, "q" | "Q") {
            self.back_to_setup();
        }
    }

    fn set_brush(&mut self, d: u8) {
        self.brush = d;
        self.b.play_sound(if d != 0 { "select" } else { "move" });
    }

    fn apply_brush(&mut self, idx: usize) {
        let d = self.brush;
        if d == 0 || self.locked[idx] {
            self.b.play_sound("move");
            return;
        }
        if !self.note_mode && self.board[idx] == d {
            self.erase(idx);
        } else {
            self.enter_digit(idx, d);
        }
    }

    // ===================================================== Rueckgaengig/Wiederholen

    fn cell_state(&self, i: usize) -> CellSt {
        (self.board[i], self.notes[i], self.marks[i])
    }

    fn begin(&mut self, primary: i64) {
        self.step = Some((primary, Vec::new()));
    }

    fn snap(&mut self, cells: &[usize]) {
        let states: Vec<(usize, CellSt)> = cells.iter().map(|&i| (i, self.cell_state(i))).collect();
        if let Some((_, before)) = self.step.as_mut() {
            for (i, st) in states {
                if !before.iter().any(|(j, _)| *j == i) {
                    before.push((i, st));
                }
            }
        }
    }

    fn commit(&mut self) {
        let Some((primary, before)) = self.step.take() else {
            return;
        };
        let changes: Vec<Change> = before.into_iter().filter_map(|(i, b)| {
            let a = self.cell_state(i);
            if a != b {
                Some((i, b, a))
            } else {
                None
            }
        }).collect();
        if !changes.is_empty() {
            self.undo_stack.push((primary, changes));
            if self.undo_stack.len() > MAX_UNDO {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.dirty = true;
        }
    }

    fn apply_state(&mut self, i: usize, st: CellSt) {
        self.board[i] = st.0;
        self.notes[i] = st.1;
        self.marks[i] = st.2;
        if self.board[i] != 0 && self.board[i] != self.solution[i] {
            self.wrong.insert(i);
        } else {
            self.wrong.remove(&i);
        }
    }

    fn undo(&mut self) {
        if self.b.game_over {
            return;
        }
        while let Some((primary, changes)) = self.undo_stack.pop() {
            if primary >= 0 && self.locked[primary as usize] {
                continue;
            }
            for &(i, before, _) in &changes {
                if !self.locked[i] {
                    self.apply_state(i, before);
                }
            }
            self.redo_stack.push((primary, changes));
            if primary >= 0 {
                self.sel = primary as usize;
            }
            self.update_conflicts();
            self.dirty = true;
            self.b.play_sound("rotate");
            return;
        }
        self.b.play_sound("move");
    }

    fn redo(&mut self) {
        if self.b.game_over {
            return;
        }
        while let Some((primary, changes)) = self.redo_stack.pop() {
            if primary >= 0 && self.locked[primary as usize] {
                continue;
            }
            for &(i, _, after) in &changes {
                if !self.locked[i] {
                    self.apply_state(i, after);
                    if self.can_check && self.board[i] != 0 && self.board[i] == self.solution[i] {
                        self.locked[i] = true;
                    }
                }
            }
            self.undo_stack.push((primary, changes));
            if primary >= 0 {
                self.sel = primary as usize;
            }
            self.update_conflicts();
            self.dirty = true;
            self.b.play_sound("rotate");
            self.check_win();
            return;
        }
        self.b.play_sound("move");
    }

    // ===================================================== Spiellogik

    fn toggle_note_mode(&mut self) {
        if !self.can_notes {
            return;
        }
        self.note_mode = !self.note_mode;
        self.b.play_sound("select");
    }

    fn enter_digit(&mut self, idx: usize, d: u8) {
        if self.locked[idx] || d < 1 || d as usize > self.n {
            return;
        }
        if self.note_mode {
            self.toggle_note(idx, d);
            return;
        }
        if self.board[idx] == d {
            return;
        }
        self.begin(idx as i64);
        let mut cs = vec![idx];
        cs.extend(self.peers[idx].iter().copied());
        self.snap(&cs);
        self.board[idx] = d;
        self.notes[idx] = 0;
        if self.can_notes {
            self.prune_notes(idx, d);
        }
        let now = pg::ticks();
        if d == self.solution[idx] {
            self.wrong.remove(&idx);
            if self.can_check {
                self.locked[idx] = true;
            }
            self.commit();
            self.fx_pop.insert(idx, now);
            self.update_conflicts();
            if self.check_units(idx, now) {
                self.b.play_sound("line");
            } else {
                self.b.play_sound("select");
            }
            self.check_win();
        } else {
            self.wrong.insert(idx);
            self.errors += 1;
            self.commit();
            self.fx_shake.insert(idx, now);
            self.b.play_sound("hit");
            self.b.rumble(120);
            self.update_conflicts();
            if self.fail_limit && self.errors >= FAIL_LIMIT {
                self.lose();
            } else {
                self.check_win();
            }
        }
    }

    fn toggle_note(&mut self, idx: usize, d: u8) {
        if self.board[idx] != 0 || !self.can_notes {
            return;
        }
        self.begin(idx as i64);
        self.snap(&[idx]);
        self.notes[idx] ^= 1 << (d - 1);
        self.commit();
        self.b.play_sound("move");
    }

    fn erase(&mut self, idx: usize) {
        if self.locked[idx] {
            return;
        }
        if self.board[idx] != 0 || self.notes[idx] != 0 || self.marks[idx] != 0 {
            self.begin(idx as i64);
            self.snap(&[idx]);
            if self.board[idx] != 0 || self.notes[idx] != 0 {
                self.board[idx] = 0;
                self.notes[idx] = 0;
            } else {
                self.marks[idx] = 0;
            }
            self.wrong.remove(&idx);
            self.commit();
            self.update_conflicts();
            self.b.play_sound("move");
        }
    }

    fn set_mark(&mut self, idx: usize, color: u8, toggle: bool) {
        if !self.colors_on || color > MARK_COUNT {
            return;
        }
        let color = if toggle && self.marks[idx] == color { 0 } else { color };
        self.begin(idx as i64);
        self.snap(&[idx]);
        self.marks[idx] = color;
        self.commit();
        self.mark_brush = color;
        self.b.play_sound("select");
    }

    fn prune_notes(&mut self, idx: usize, d: u8) {
        let m = !(1u32 << (d - 1));
        for k in 0..self.peers[idx].len() {
            let j = self.peers[idx][k];
            self.notes[j] &= m;
        }
    }

    fn candidates(&self, i: usize) -> u32 {
        let mut used = 0u32;
        for &j in &self.peers[i] {
            if self.board[j] != 0 {
                used |= 1 << (self.board[j] - 1);
            }
        }
        self.lay.all & !used
    }

    fn auto_candidates(&mut self) {
        if !self.can_notes || self.b.game_over {
            return;
        }
        self.begin(-1);
        for i in 0..self.cells {
            if self.board[i] != 0 || self.locked[i] {
                continue;
            }
            self.snap(&[i]);
            self.notes[i] = self.candidates(i);
        }
        self.commit();
        self.say(t("sud.cands_done"), Some(self.b.accent), 2.5);
        self.b.play_sound("powerup");
    }

    fn use_hint(&mut self) {
        if !self.can_hint || self.hints_used >= MAX_HINTS || self.b.game_over {
            return;
        }
        let mut idx = self.sel;
        if self.locked[idx] || (self.board[idx] != 0 && !self.wrong.contains(&idx)) {
            let empties: Vec<usize> = (0..self.cells).filter(|&i| !self.locked[i] && self.board[i] != self.solution[i]).collect();
            if empties.is_empty() {
                return;
            }
            idx = py::choice(&empties);
        }
        let d = self.solution[idx];
        self.board[idx] = d;
        self.notes[idx] = 0;
        self.wrong.remove(&idx);
        self.locked[idx] = true;
        self.hinted.insert(idx);
        self.hints_used += 1;
        self.prune_notes(idx, d);
        self.sel = idx;
        let now = pg::ticks();
        self.fx_pop.insert(idx, now);
        self.dirty = true;
        self.b.play_sound("powerup");
        self.update_conflicts();
        self.check_units(idx, now);
        self.check_win();
    }

    fn check_units(&mut self, idx: usize, now: u64) -> bool {
        let mut done = false;
        for k in 0..self.groups_of[idx].len() {
            let g = self.groups_of[idx][k];
            let cells = &self.groups[g];
            if cells.iter().all(|&i| self.board[i] == self.solution[i]) {
                self.fx_units.push((cells.clone(), idx, now));
                done = true;
            }
        }
        if self.fx_units.len() > 12 {
            let cut = self.fx_units.len() - 12;
            self.fx_units.drain(..cut);
        }
        done
    }

    fn update_conflicts(&mut self) {
        if !self.can_check {
            return;
        }
        self.conflicts = (0..self.cells).filter(|&i| {
            let v = self.board[i];
            v != 0 && self.peers[i].iter().any(|&j| self.board[j] == v)
        }).collect();
        let mut bad = HashSet::new();
        for (ci, (total, cells)) in self.cages.iter().enumerate() {
            let vals: Vec<u8> = cells.iter().map(|&i| self.board[i]).filter(|v| *v != 0).collect();
            let s: i64 = vals.iter().map(|v| *v as i64).sum();
            let uniq: HashSet<u8> = vals.iter().copied().collect();
            if uniq.len() != vals.len() || s > *total || (vals.len() == cells.len() && s != *total) || (vals.len() < cells.len() && s >= *total) {
                bad.insert(ci);
            }
        }
        self.bad_cages = bad;
    }

    fn check_win(&mut self) {
        if self.board.contains(&0) {
            return;
        }
        if self.board == self.solution {
            self.win();
        } else if !self.can_check {
            self.say(t("sud.full_wrong"), Some(ui::pal().red), 2.5);
        }
    }

    fn say(&mut self, text: String, col: Option<Color>, secs: f64) {
        self.msg = Some(text);
        self.msg_col = col;
        self.msg_t = secs;
    }

    fn win(&mut self) {
        self.won = true;
        self.b.game_over = true;
        let (v, d) = (self.play_variant, self.play_diff);
        let secs = self.elapsed as i64;
        let base = base_points(v)[d as usize];
        let raw = base - TIME_COST * secs - ERR_COST * self.errors - HINT_COST * self.hints_used;
        self.b.score = (50.max(raw) as f64 * self.mult) as i64;
        let stars = stars_for(v, d, secs, self.errors, self.hints_used);
        self.result_stars = stars;
        self.result_time = secs;
        self.new_record = false;
        self.best_time = secs;
        if self.daily {
            let same_day = self.daily_state.get("date").and_then(|x| x.as_str()) == Some(self.daily_date.as_str());
            if same_day {
                let old = self.daily_state.get("time").and_then(|x| x.as_i64()).unwrap_or(secs + 1);
                self.new_record = secs < old;
                self.daily_state.insert("time".into(), json!(old.min(secs)));
                let st = self.daily_state.get("stars").and_then(|x| x.as_i64()).unwrap_or(0).max(stars);
                self.daily_state.insert("stars".into(), json!(st));
            } else {
                let ds = self.daily_streak();
                let streak = if ds != 0 { ds + 1 } else { 1 };
                let best = self.daily_state.get("best").and_then(|x| x.as_i64()).unwrap_or(0).max(streak);
                let count = self.daily_state.get("count").and_then(|x| x.as_i64()).unwrap_or(0) + 1;
                self.daily_state.insert("date".into(), json!(self.daily_date));
                self.daily_state.insert("streak".into(), json!(streak));
                self.daily_state.insert("best".into(), json!(best));
                self.daily_state.insert("time".into(), json!(secs));
                self.daily_state.insert("stars".into(), json!(stars));
                self.daily_state.insert("count".into(), json!(count));
            }
            self.best_time = self.daily_state.get("time").and_then(|x| x.as_i64()).unwrap_or(secs);
        } else {
            let lv = self.level.to_string();
            let levels = self.best.entry(v.to_string()).or_default().entry(d.to_string()).or_default();
            match levels.iter_mut().find(|(k, _)| *k == lv) {
                Some(e) => {
                    self.new_record = secs < e.1[1];
                    e.1 = [e.1[0].max(stars), e.1[1].min(secs)];
                    self.best_time = e.1[1];
                }
                None => {
                    levels.push((lv, [stars, secs]));
                    self.best_time = secs;
                }
            }
            if v == "classic" && !self.solved[d as usize].contains(&self.level) {
                self.solved[d as usize].push(self.level);
                self.solved[d as usize].sort_unstable();
            }
        }
        self.saves.shift_remove(&self.save_key());
        self.save_progress();
        self.b.report_result(true);
        if self.errors == 0 {
            self.b.ach_event("sudoku_clean", None);
        }
        if v == "killer" && !self.daily {
            self.b.ach_event("sudoku_killer", None);
        }
        let tsc = self.three_star_count();
        self.b.ach_event("sudoku_stars", Some(tsc));
        self.win_ticks = pg::ticks();
        self.star_snd.clear();
        self.b.play_sound("win");
        self.b.rumble(200);
        if stars == 3 {
            ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 70);
        }
    }

    fn lose(&mut self) {
        self.won = false;
        self.b.game_over = true;
        self.b.score = 0;
        if self.saves.shift_remove(&self.save_key()).is_some() {
            self.save_progress();
        }
        self.b.report_result(false);
        self.win_ticks = pg::ticks();
        self.b.play_sound("gameover");
        self.b.rumble(250);
    }

    fn next_level(&mut self) {
        if self.daily || self.level >= gen::LEVELS {
            self.back_to_setup();
            return;
        }
        self.variant = self.play_variant;
        self.diff = self.play_diff;
        let l = self.level + 1;
        self.start_level(l, false);
    }

    // ===================================================== Zeichnen: Setup

    fn diff_name(&self, d: i64) -> String {
        t(&format!("sud.diff.{}", DIFFICULTIES[d.clamp(0, 3) as usize]))
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let cx = w / 2;
        let title = self.title.render("SUDOKU", true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, self.su_title_y)));
        let mode_lbl = if mode_mult(self.mode()).is_some() { t(&format!("sud.mode.{}", self.mode())) } else { self.mode().to_string() };
        let sub = fit(&mode_lbl, w - 24, &[&self.tiny], p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, self.su_sub_y)));
        for (i, r) in self.var_rects.iter().enumerate() {
            let v = VARIANTS[i];
            let on = v == self.variant;
            ui::draw_button_ex(s, *r, "", &self.small, on, None, None, Some(self.b.accent));
            self.draw_variant_label(s, *r, v, on);
        }
        for (i, r) in self.diff_rects.iter().enumerate() {
            let label = self.diff_name(i as i64);
            let fnt = if self.small.size(&label).0 <= r.w - 10 { &self.small } else { &self.tiny };
            ui::draw_button_ex(s, *r, &label, fnt, i as i64 == self.diff, None, None, Some(self.b.accent));
        }
        self.draw_level_grid(s);
        let (n, stars) = self.stage_summary(self.variant, self.diff);
        let txt = format!("{}   ·   {}/300", tf("sud.progress", &[("n", &n.to_string())]), stars);
        let img = fit(&txt, 10 * self.lv_cell - 20, &[&self.small, &self.tiny], p.text_dim);
        let rc = img.get_rect().center_at((self.lv_x + 5 * self.lv_cell - 10, self.prog_y));
        s.blr(&img, rc);
        draw_star(s, ((rc.right() + 10) as f64, rc.centery() as f64), 5.max(img.h() / 3) as f64, p.gold, true);
        ui::draw_panel(s, self.panel_rect);
        self.draw_daily_card(s);
        self.draw_toggle(s, self.limit_rect, "limit", &t("sud.fail_limit"), None, self.fail_limit);
        let iv = t(&format!("sud.input.{}", if self.input_digit { "digit" } else { "cell" }));
        self.draw_toggle(s, self.input_rect, "input", &t("sud.opt_input"), Some(&iv), true);
        self.draw_toggle(s, self.colors_rect, "colors", &t("sud.opt_colors"), None, self.colors_on);
        self.draw_level_info(s);
        if let Some(sr) = self.start_rect {
            let save = self.level_save(self.variant, self.diff, self.cursor);
            let label = if save.is_some() { t("sud.resume") } else { tf("sud.start", &[("n", &self.cursor.to_string())]) };
            ui::draw_button_ex(s, sr, &label, &self.small, self.hover == Some(Hover::Setup("start", 0)), None, None, Some(self.b.accent));
        }
        self.draw_footer(s, &t("sud.setup_hint"), self.tiny.get_height() + 10);
    }

    fn draw_variant_label(&self, s: &mut Surface, r: Rect, v: &str, on: bool) {
        let p = ui::pal();
        let label = t(&format!("sud.variant.{v}"));
        let col = if on { p.text } else { p.text_dim };
        let mut img = self.small.render(&label, true, col);
        if img.w() > r.w - 10 {
            img = self.tiny.render(&label, true, col);
        }
        let icon = 10.max((r.h - 12).min(22));
        let total = icon + 7 + img.w();
        if total <= r.w - 12 {
            let x = r.centerx() - total / 2;
            self.draw_variant_icon(s, Rect::new(x, r.centery() - icon / 2, icon, icon), v, if on { self.b.accent } else { p.text_faint });
            s.blr(&img, img.get_rect().midleft_at((x + icon + 7, r.centery())));
        } else {
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
    }

    fn draw_variant_icon(&self, s: &mut Surface, r: Rect, v: &str, col: Color) {
        let n = if v == "mini" { 2 } else { 3 };
        draw::rect_r(s, col, r, 1, 2);
        for k in 1..n {
            let x = r.x + r.w * k / n;
            let y = r.y + r.h * k / n;
            draw::line(s, col, (x, r.y), (x, r.bottom() - 1), 1);
            draw::line(s, col, (r.x, y), (r.right() - 1, y), 1);
        }
        if v == "x" {
            draw::line(s, col, (r.x, r.y), (r.right() - 1, r.bottom() - 1), 2);
            draw::line(s, col, (r.right() - 1, r.y), (r.x, r.bottom() - 1), 2);
        } else if v == "killer" {
            // Python: -r.w // 3 = (-r.w) // 3 (Abrunden!)
            let inner = r.inflate(py::fdiv32(-r.w, 3), py::fdiv32(-r.h, 3));
            let mut k = 0;
            while k < inner.w {
                draw::line(s, col, (inner.x + k, inner.y), (inner.x + (k + 2).min(inner.w), inner.y), 1);
                draw::line(s, col, (inner.x + k, inner.bottom()), (inner.x + (k + 2).min(inner.w), inner.bottom()), 1);
                k += 4;
            }
            let mut k = 0;
            while k < inner.h {
                draw::line(s, col, (inner.x, inner.y + k), (inner.x, inner.y + (k + 2).min(inner.h)), 1);
                draw::line(s, col, (inner.right(), inner.y + k), (inner.right(), inner.y + (k + 2).min(inner.h)), 1);
                k += 4;
            }
        }
    }

    fn draw_level_grid(&self, s: &mut Surface) {
        let p = ui::pal();
        let (v, d) = (self.variant, self.diff);
        let cell = self.lv_cell;
        let solved_bg = ui::mix(p.panel, p.green, 0.20);
        let gold_bg = ui::mix(p.panel, p.gold, 0.22);
        let big = cell >= 30;
        for n in 1..=100i64 {
            let i = (n - 1) as i32;
            let x = self.lv_x + (i % 10) * cell;
            let y = self.lv_y + (i / 10) * cell;
            let rc = Rect::new(x + 1, y + 1, cell - 2, cell - 2);
            let (stars, _) = self.record(v, d, n);
            let save = self.level_save(v, d, n);
            let bg = if stars == 3 {
                gold_bg
            } else if stars != 0 {
                solved_bg
            } else {
                p.btn
            };
            draw::rect_r(s, bg, rc, 0, 4);
            if n == self.cursor {
                draw::rect_r(s, self.b.accent, rc, 2, 4);
            }
            let col = if stars == 3 {
                p.gold
            } else if stars != 0 {
                p.green
            } else {
                p.text_dim
            };
            let num = self.lv_font.render(&n.to_string(), true, col);
            let cy = rc.centery() - if big && stars != 0 { cell / 7 } else { 0 };
            s.blr(&num, num.get_rect().center_at((rc.centerx(), cy)));
            if stars != 0 {
                if big {
                    let sr = 3.max(cell / 10);
                    for k in 0..3 {
                        let sx = rc.centerx() + (k - 1) * (sr * 2 + 2);
                        let c = if (k as i64) < stars { p.gold } else { ui::mix(bg, p.text_faint, 0.5) };
                        draw_star(s, (sx as f64, (rc.bottom() - sr - 3) as f64), sr as f64, c, true);
                    }
                } else {
                    for k in 0..stars as i32 {
                        draw::circle(s, p.gold, (rc.centerx() + (k - 1) * 4, rc.bottom() - 3), 1, 0);
                    }
                }
            }
            if let Some(save) = save {
                let tri = 5.max(cell / 4);
                draw::polygon(s, self.b.accent, &[(rc.right() - tri, rc.y), (rc.right(), rc.y), (rc.right(), rc.y + tri)], 0);
                let pp = save.get("p").and_then(strict_int).unwrap_or(0);
                let bw = ((rc.w - 6) as f64 * pp.clamp(0, 100) as f64 / 100.0) as i32;
                if bw > 0 && stars == 0 {
                    draw::rect(s, self.b.accent, Rect::new(rc.x + 3, rc.bottom() - 4, bw, 2), 0);
                }
            }
        }
    }

    fn draw_daily_card(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.daily_rect;
        let hov = self.hover == Some(Hover::Setup("daily", 0));
        let done = self.daily_done_today();
        let base = ui::mix(p.panel_light, self.b.accent, 0.10 + if hov { 0.10 } else { 0.0 });
        draw::rect_r(s, base, r, 0, 8);
        let glow = if hov { self.b.accent } else { ui::mix(p.border, self.b.accent, 0.35 + 0.35 * ui::pulse(2.0, 0.0, 1.0) * (!done as i32 as f64)) };
        draw::rect_r(s, glow, r, if hov { 2 } else { 1 }, 8);
        let ih = (r.h - 10).min(34);
        let cal = Rect::new(r.x + 7, r.centery() - ih / 2, ih, ih);
        draw::rect_r(s, ui::mix(p.panel, p.text, 0.08), cal, 0, 4);
        draw::rect_corners(s, self.b.accent, Rect::new(cal.x, cal.y, cal.w, 4.max(ih / 4)), 0, 0, 4, 4, -1, -1);
        let dd: i64 = self.daily_date.get(8..10).and_then(|x| x.parse().ok()).unwrap_or(1);
        let day = self.tiny.render(&dd.to_string(), true, p.text);
        s.blr(&day, day.get_rect().center_at((cal.centerx(), cal.centery() + ih / 8)));
        let tx = cal.right() + 8;
        let tw = r.right() - 8 - tx;
        let diff = gen::daily_diff(&self.daily_date);
        let wd = t(&format!("sud.wd.{}", gen::weekday(&self.daily_date)));
        let line1 = fit(&t("sud.daily"), tw - 18, &[&self.small, &self.tiny], p.text);
        let y1 = r.y + 5;
        s.bl(&line1, (tx, y1));
        let ds = self.daily_streak();
        let streak = if ds != 0 { Some(tf("sud.daily_streak", &[("n", &ds.to_string())])) } else { None };
        let save = self.saves.get(&format!("daily:{}", self.daily_date));
        let mut head: Vec<Option<String>>;
        if done {
            head = vec![Some(fmt_time(self.daily_state.get("time").and_then(|v| v.as_i64()).unwrap_or(0) as f64))];
            draw_check(s, (r.right() - 14, y1 + line1.h() / 2), 5.0, p.green, 2);
        } else {
            head = vec![Some(wd), Some(self.diff_name(diff))];
            if let Some(pp) = save.and_then(|v| v.as_object()).and_then(|o| o.get("p")).and_then(strict_int) {
                head.push(Some(format!("{pp} %")));
            }
        }
        let with = |mut v: Vec<Option<String>>, x: &Option<String>| {
            v.push(x.clone());
            v
        };
        let options: Vec<Vec<Option<String>>> = if !done {
            vec![with(head.clone(), &streak), with(head[1..].to_vec(), &streak), head[1..].to_vec()]
        } else {
            vec![with(head.clone(), &streak), head.clone()]
        };
        let mut text = String::new();
        for parts in &options {
            text = parts.iter().flatten().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>().join("  ·  ");
            if self.tiny.size(&text).0 <= tw {
                break;
            }
        }
        let line2 = fit(&text, tw, &[&self.tiny], if done { p.green } else { p.text_dim });
        s.bl(&line2, (tx, r.bottom() - 5 - line2.h()));
    }

    fn draw_toggle(&self, s: &mut Surface, r: Rect, key: &'static str, label: &str, value: Option<&str>, on: bool) {
        let p = ui::pal();
        let hov = self.hover == Some(Hover::Setup(key, 0));
        draw::rect_r(s, if hov { p.btn_sel } else { p.btn }, r, 0, 6);
        draw::rect_r(s, if hov { self.b.accent } else { p.border }, r, 1, 6);
        let img;
        match value {
            None => {
                let sw = 24.max(r.h * 3 / 2);
                let sh = 12.max(r.h - 10);
                let pill = Rect::new(r.right() - 8 - sw, r.centery() - sh / 2, sw, sh);
                draw::rect_r(s, if on { p.green } else { ui::mix(p.btn, p.text_faint, 0.5) }, pill, 0, sh / 2);
                let kx = if on { pill.right() - sh / 2 } else { pill.x + sh / 2 };
                draw::circle(s, p.text, (kx, pill.centery()), sh / 2 - 2, 0);
                let room = pill.x - 8 - (r.x + 8);
                img = fit(label, room, &[&self.tiny], if on { p.text } else { p.text_dim });
            }
            Some(value) => {
                let val = self.tiny.render(value, true, self.b.accent);
                s.blr(&val, val.get_rect().midright_at((r.right() - 8, r.centery())));
                let room = r.w - 24 - val.w();
                img = fit(label, room, &[&self.tiny], p.text);
            }
        }
        s.blr(&img, img.get_rect().midleft_at((r.x + 8, r.centery())));
    }

    fn draw_level_info(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.info_rect;
        if r.h < self.tiny.get_height() {
            return;
        }
        let (v, d, n) = (self.variant, self.diff, self.cursor);
        let (stars, secs) = self.record(v, d, n);
        let save = self.level_save(v, d, n);
        let mut y = r.y;
        let head = self.small.render(&tf("sud.level", &[("n", &n.to_string())]), true, p.text);
        s.bl(&head, (r.x, y));
        let sr = 5.max(head.h() / 3);
        for k in 0..3 {
            let c = if 3 - k <= stars as i32 { p.gold } else { ui::mix(p.panel, p.text_faint, 0.6) };
            draw_star(s, ((r.right() - sr - k * (2 * sr + 3)) as f64, (y + head.h() / 2) as f64), sr as f64, c, true);
        }
        y += head.h() + 2;
        let mut lines: Vec<(String, Color)> = Vec::new();
        if let Some(secs) = secs {
            lines.push((tf("sud.info_best", &[("t", &fmt_time(secs as f64))]), p.green));
        } else if stars != 0 {
            lines.push((t("sud.info_solved"), p.green));
        } else {
            lines.push((t("sud.info_unsolved"), p.text_dim));
        }
        if let Some(save) = save.and_then(|v| v.as_object()) {
            let pp = save.get("p").and_then(strict_int).unwrap_or(0);
            lines.push((tf("sud.info_started", &[("p", &pp.to_string())]), self.b.accent));
        }
        lines.push((tf("sud.info_target", &[("t", &fmt_time(target_time(v)[d as usize] as f64))]), p.text_dim));
        let th = self.tiny.get_height();
        for (txt, col) in &lines {
            for line in wrap(txt, &self.tiny, r.w).iter().take(2) {
                if y + th > r.bottom() {
                    return;
                }
                let img = fit(line, r.w, &[&self.tiny], *col);
                s.bl(&img, (r.x, y));
                y += th;
            }
            y += 1;
        }
        y += 4;
        for line in wrap(&t(&format!("sud.vdesc.{v}")), &self.tiny, r.w) {
            if y + th > r.bottom() {
                return;
            }
            s.bl(&self.tiny.render(&line, true, p.text_faint), (r.x, y));
            y += th;
        }
    }

    // ===================================================== Zeichnen: Spiel

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, Rect::new(0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let clock = if self.b.game_over && self.reveal {
            self.small.render(&t("sud.hide_solution"), true, self.b.accent)
        } else {
            self.clock_font.render(&fmt_time(self.elapsed), true, self.b.accent)
        };
        let crect = clock.get_rect().center_at((w / 2, cy));
        s.blr(&clock, crect);
        let side = crect.x - 20;
        let (top, bottom) = if self.daily {
            (t("sud.daily"), format!("{}  ·  {}", t(&format!("sud.wd.{}", gen::weekday(&self.daily_date))), self.diff_name(self.play_diff)))
        } else {
            (t(&format!("sud.variant.{}", self.play_variant)), format!("{}  ·  {}", tf("sud.level", &[("n", &self.level.to_string())]), self.diff_name(self.play_diff)))
        };
        let a = fit(&top, side, &[&self.small, &self.tiny], self.b.accent);
        let bb = fit(&bottom, side, &[&self.tiny], p.text);
        s.blr(&a, a.get_rect().bottomleft_at((12, cy + 1)));
        s.blr(&bb, bb.get_rect().topleft_at((12, cy + 1)));
        let err_txt = if self.fail_limit {
            tf("sud.errors", &[("n", &self.errors.to_string()), ("m", &FAIL_LIMIT.to_string())])
        } else {
            tf("sud.errors_free", &[("n", &self.errors.to_string())])
        };
        let e = fit(&err_txt, side, &[&self.small, &self.tiny], if self.errors != 0 { p.red } else { p.text_dim });
        s.blr(&e, e.get_rect().bottomright_at((w - 12, cy + 1)));
        let mut right: Vec<(String, Color)> = Vec::new();
        if self.can_hint {
            right.push((tf("sud.hints", &[("n", &(MAX_HINTS - self.hints_used).to_string())]), p.text_dim));
        }
        if self.note_mode {
            right.push((t("sud.pad_note").to_uppercase(), self.b.accent));
        }
        if self.input_digit && self.brush != 0 {
            right.push((tf("sud.brush", &[("d", &self.brush.to_string())]), self.b.accent));
        }
        let mut x = w - 12;
        for (txt, col) in right.iter().rev() {
            let img = self.tiny.render(txt, true, *col);
            if x - img.w() < crect.right() + 20 {
                break;
            }
            s.blr(&img, img.get_rect().topright_at((x, cy + 1)));
            x -= img.w() + 12;
        }
    }

    fn cage_overlay(&mut self) -> (Surface, Vec<(i32, i32)>) {
        let p = ui::pal();
        let col = ui::mix(p.text_dim, p.text, 0.2);
        let key = (self.cell, col);
        if let Some((k, surf, pos)) = &self.cage_cache {
            if *k == key {
                return (surf.clone(), pos.clone());
            }
        }
        let n = self.n as i32;
        let cell = self.cell;
        let bs = cell * n;
        let mut surf = pg::surface_alpha(bs + 1, bs + 1);
        let c = rgba(col.r as i32, col.g as i32, col.b as i32, 215);
        let ins = 3.max(cell / 9);
        let dash = 3.max(cell / 8);
        let period = dash + 2.max(dash * 3 / 4);
        let cage_of = &self.cage_of;
        let same = |r: i32, cc: i32, k: i64| (0..n).contains(&r) && (0..n).contains(&cc) && cage_of[(r * n + cc) as usize] == k;
        let hline = |surf: &mut Surface, y: i32, xa: i32, xb: i32| {
            let mut x = xa;
            while x < xb {
                let ph = py::pmod32(x, period);
                if ph < dash {
                    let e = xb.min(x + dash - ph);
                    draw::line(surf, c, (x, y), (e, y), 1);
                    x = e;
                } else {
                    x += period - ph;
                }
            }
        };
        let vline = |surf: &mut Surface, x: i32, ya: i32, yb: i32| {
            let mut y = ya;
            while y < yb {
                let ph = py::pmod32(y, period);
                if ph < dash {
                    let e = yb.min(y + dash - ph);
                    draw::line(surf, c, (x, y), (x, e), 1);
                    y = e;
                } else {
                    y += period - ph;
                }
            }
        };
        for i in 0..n * n {
            let (r, cc) = (i / n, i % n);
            let k = cage_of[i as usize];
            let (x0, y0) = (cc * cell, r * cell);
            let (x1, y1) = (x0 + cell - 1, y0 + cell - 1);
            let (left, right) = (same(r, cc - 1, k), same(r, cc + 1, k));
            let (up, down) = (same(r - 1, cc, k), same(r + 1, cc, k));
            let xa = if left { x0 } else { x0 + ins };
            let xb = if right { x1 + 1 } else { x1 - ins };
            let ya = if up { y0 } else { y0 + ins };
            let yb = if down { y1 + 1 } else { y1 - ins };
            if !up {
                hline(&mut surf, y0 + ins, xa, xb);
            }
            if !down {
                hline(&mut surf, y1 - ins, xa, xb);
            }
            if !left {
                vline(&mut surf, x0 + ins, ya, yb);
            }
            if !right {
                vline(&mut surf, x1 - ins, ya, yb);
            }
            for (dr, dc) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                if same(r + dr, cc, k) && same(r, cc + dc, k) && !same(r + dr, cc + dc, k) {
                    let px = if dc < 0 { x0 + ins } else { x1 - ins };
                    let py_ = if dr < 0 { y0 + ins } else { y1 - ins };
                    if dr < 0 {
                        vline(&mut surf, px, y0, py_);
                    } else {
                        vline(&mut surf, px, py_, y1 + 1);
                    }
                    if dc < 0 {
                        hline(&mut surf, py_, x0, px);
                    } else {
                        hline(&mut surf, py_, px, x1 + 1);
                    }
                }
            }
        }
        let mut sum_pos = Vec::new();
        for (total, cells) in &self.cages {
            let first = *cells.iter().min().unwrap_or(&0) as i32;
            let (r, cc) = (first / n, first % n);
            let (w, h) = self.sum_font.size(&total.to_string());
            let (x, y) = (cc * cell + 2, r * cell + 1);
            surf.fill_r(rgba(0, 0, 0, 0), Rect::new(x - 1, y, w + 3, h));
            sum_pos.push((x, y));
        }
        self.cage_cache = Some((key, surf.clone(), sum_pos.clone()));
        (surf, sum_pos)
    }

    fn diag_overlay(&mut self) -> Surface {
        let p = ui::pal();
        let col = ui::mix(p.panel_light, self.b.accent, 0.55);
        let key = (self.cell, col);
        if let Some((k, surf)) = &self.diag_cache {
            if *k == key {
                return surf.clone();
            }
        }
        let bs = self.cell * self.n as i32;
        let mut surf = pg::surface_alpha(bs + 1, bs + 1);
        let c = rgba(col.r as i32, col.g as i32, col.b as i32, 90);
        let w = 1.max(self.cell / 14);
        draw::line(&mut surf, c, (0, 0), (bs, bs), w);
        draw::line(&mut surf, c, (bs, 0), (0, bs), w);
        self.diag_cache = Some((key, surf.clone()));
        surf
    }

    fn draw_board(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (n, cell, bx, by) = (self.n, self.cell, self.bx, self.by);
        let lay = self.lay;
        let now = pg::ticks();
        let sel = self.sel;
        let sel_val = self.board[sel];
        let sel_peers: HashSet<usize> = self.peers[sel].iter().copied().collect();
        let sel_cage = self.cage_of[sel];
        let same_val = if self.input_digit && self.brush != 0 { self.brush } else { sel_val };
        let c_cell = p.panel;
        let c_peer = p.panel_light;
        let c_sel = ui::mix(p.panel_light, self.b.accent, 0.35);
        let c_same = ui::mix(p.panel_light, self.b.accent, 0.18);
        let c_bad = ui::mix(p.panel, p.red, 0.35);
        let c_diag = ui::mix(p.panel, self.b.accent, 0.08);
        let c_cage = ui::mix(p.panel, self.b.accent, 0.13);
        let glow = ui::mix(self.b.accent, (255, 255, 255), 0.45);
        let show_solution = self.b.game_over && self.reveal;
        let mut bad_cells: HashSet<usize> = self.conflicts.clone();
        if self.can_check {
            for &ci in &self.bad_cages {
                bad_cells.extend(self.cages[ci].1.iter().copied());
            }
        }
        let mut flash: HashMap<usize, f64> = HashMap::new();
        let mut alive = Vec::new();
        for (cells, origin, t0) in std::mem::take(&mut self.fx_units) {
            let age_all = now as i64 - t0 as i64;
            if age_all > 900 {
                continue;
            }
            let (orr, occ) = ((origin / n) as i64, (origin % n) as i64);
            for &i in &cells {
                let (rr, c2) = ((i / n) as i64, (i % n) as i64);
                let age = age_all - 45 * ((rr - orr).abs() + (c2 - occ).abs());
                if (0..420).contains(&age) {
                    let a = (PI * age as f64 / 420.0).sin() * 0.6;
                    if a > *flash.get(&i).unwrap_or(&0.0) {
                        flash.insert(i, a);
                    }
                }
            }
            alive.push((cells, origin, t0));
        }
        self.fx_units = alive;
        let win_age: i64 = if self.b.game_over && self.won { now as i64 - self.win_ticks as i64 } else { -1 };

        for i in 0..n * n {
            let (r, c) = (i / n, i % n);
            let x = bx + c as i32 * cell;
            let y = by + r as i32 * cell;
            let mut bg = if i == sel {
                c_sel
            } else if self.can_check && bad_cells.contains(&i) {
                c_bad
            } else if self.can_check && same_val != 0 && self.board[i] == same_val {
                c_same
            } else if sel_cage >= 0 && self.cage_of[i] == sel_cage {
                c_cage
            } else if sel_peers.contains(&i) {
                c_peer
            } else if lay.diagonals && lay.on_diagonal(i) {
                c_diag
            } else {
                c_cell
            };
            if self.marks[i] != 0 {
                bg = ui::mix(bg, MARK_COLORS[self.marks[i] as usize - 1], 0.42);
            }
            if let Some(a) = flash.get(&i) {
                if *a != 0.0 {
                    bg = ui::mix(bg, glow, *a);
                }
            }
            if win_age >= 0 {
                let half = (n - 1) as f64 / 2.0;
                let dist = (r as f64 - half).abs() + (c as f64 - half).abs();
                let age = win_age as f64 - 60.0 * dist;
                if (0.0..520.0).contains(&age) {
                    bg = ui::mix(bg, p.green, (PI * age / 520.0).sin() * 0.55);
                }
            }
            draw::rect(s, bg, Rect::new(x, y, cell, cell), 0);
        }
        if lay.diagonals {
            let d = self.diag_overlay();
            s.bl(&d, (bx, by));
        }
        if !self.cages.is_empty() {
            let (ov, sum_pos) = self.cage_overlay();
            s.bl(&ov, (bx, by));
            for (ci, (total, _)) in self.cages.iter().enumerate() {
                let col = if self.can_check && self.bad_cages.contains(&ci) {
                    p.red
                } else if ci as i64 == sel_cage {
                    self.b.accent
                } else {
                    p.text_dim
                };
                let img = self.sum_font.render(&total.to_string(), true, col);
                let (sx, sy) = sum_pos[ci];
                s.bl(&img, (bx + sx, by + sy));
            }
        }
        let has_cages = !self.cages.is_empty();
        let ins = if has_cages { 3.max(cell / 9) + 2 } else { 0 };
        let note_top = if has_cages { ins.max(self.sum_font.get_height() - 1) } else { 0 };
        let rows = (n as i32 + 2) / 3;
        let cw = py::fdiv32(cell - 2 * ins, 3);
        let ch = py::fdiv32(cell - note_top - ins, rows);
        for i in 0..n * n {
            let (r, c) = (i / n, i % n);
            let x = bx + c as i32 * cell;
            let y = by + r as i32 * cell;
            if show_solution && self.board[i] != self.solution[i] {
                let img = self.num_font.render(&self.solution[i].to_string(), true, self.b.accent);
                s.blr(&img, img.get_rect().center_at((x + cell / 2, y + cell / 2)));
                continue;
            }
            let v = self.board[i];
            if v != 0 {
                let col = if self.given[i] {
                    p.text
                } else if self.can_check && self.wrong.contains(&i) {
                    p.red
                } else if self.locked[i] {
                    p.green
                } else {
                    COL_USER
                };
                let mut img = self.num_font.render(&v.to_string(), true, col);
                let mut cxm = x + cell / 2;
                let cym = y + cell / 2 + if has_cages { note_top / 3 } else { 0 };
                if let Some(&t0) = self.fx_pop.get(&i) {
                    let age = now as i64 - t0 as i64;
                    if age < 240 {
                        let k = 1.0 + 0.4 * (PI * age as f64 / 240.0).sin();
                        img = transform::smoothscale(&img, 1.max((img.w() as f64 * k) as i32), 1.max((img.h() as f64 * k) as i32));
                    } else {
                        self.fx_pop.remove(&i);
                    }
                }
                if let Some(&t1) = self.fx_shake.get(&i) {
                    let age = now as i64 - t1 as i64;
                    if age < 380 {
                        cxm += ((age as f64 / 22.0).sin() * cell as f64 * 0.09 * (1.0 - age as f64 / 380.0)) as i32;
                    } else {
                        self.fx_shake.remove(&i);
                    }
                }
                s.blr(&img, img.get_rect().center_at((cxm, cym)));
            } else if self.notes[i] != 0 {
                let nm = self.notes[i];
                for d in 1..=n as i32 {
                    if (nm >> (d - 1)) & 1 != 0 {
                        let col = if self.can_check && d == same_val as i32 { self.b.accent } else { p.text_faint };
                        let img = self.note_font.render(&d.to_string(), true, col);
                        let nx = x + ins + ((d - 1) % 3) * cw + cw / 2;
                        let ny = y + note_top + ((d - 1) / 3) * ch + ch / 2;
                        s.blr(&img, img.get_rect().center_at((nx, ny)));
                    }
                }
            }
        }
        let bs = cell * n as i32;
        for k in 0..=n {
            let x = bx + k as i32 * cell;
            let thick = k % lay.box_w == 0;
            draw::line(s, if thick { p.border_light } else { p.border }, (x, by), (x, by + bs), if thick { 2 } else { 1 });
        }
        for k in 0..=n {
            let y = by + k as i32 * cell;
            let thick = k % lay.box_h == 0;
            draw::line(s, if thick { p.border_light } else { p.border }, (bx, y), (bx + bs, y), if thick { 2 } else { 1 });
        }
        if !self.b.game_over {
            let (r, c) = ((sel / n) as i32, (sel % n) as i32);
            draw::rect(s, self.b.accent, Rect::new(bx + c * cell, by + r * cell, cell + 1, cell + 1), 2);
        }
    }

    fn hover_key(&self) -> Option<&str> {
        match &self.hover {
            Some(Hover::Pad(k)) => Some(k.as_str()),
            _ => None,
        }
    }

    fn draw_pad(&self, s: &mut Surface) {
        let p = ui::pal();
        let n = self.n;
        let mut counts = vec![0usize; n + 1];
        for &v in &self.board {
            counts[v as usize] += 1;
        }
        let th = self.tiny.get_height();
        let hk = self.hover_key();
        for d in 1..=n {
            let Some(r) = self.pad_rect(&d.to_string()) else {
                continue;
            };
            let left = n as i64 - counts[d] as i64;
            let done = left <= 0;
            let on = self.input_digit && self.brush as usize == d;
            let hov = hk == Some(d.to_string().as_str());
            let bg = if on {
                p.btn_sel
            } else if hov {
                ui::mix(p.btn, p.panel_light, 0.6)
            } else {
                p.btn
            };
            draw::rect_r(s, bg, r, 0, 6);
            draw::rect_r(s, if on { self.b.accent } else { p.border }, r, if on { 2 } else { 1 }, 6);
            let col = if done {
                p.text_faint
            } else if on {
                self.b.accent
            } else {
                p.text
            };
            let img = self.pad_font.render(&d.to_string(), true, col);
            s.blr(&img, img.get_rect().center_at((r.centerx(), r.centery() - th / 3)));
            if done {
                draw_check(s, (r.centerx(), r.bottom() - th / 2 - 2), 3.max(th / 3) as f64, p.green, 2);
            } else {
                let cnt = self.tiny.render(&left.to_string(), true, p.text_dim);
                s.blr(&cnt, cnt.get_rect().center_at((r.centerx(), r.bottom() - th / 2 - 1)));
            }
        }
        for key in PAD_FUNCS {
            let Some(r) = self.pad_rect(key) else {
                continue;
            };
            let hov = hk == Some(key);
            let active = key == "note" && self.note_mode;
            let enabled = match key {
                "undo" => !self.undo_stack.is_empty(),
                "redo" => !self.redo_stack.is_empty(),
                "hint" => self.hints_used < MAX_HINTS,
                _ => true,
            };
            let bg = if active {
                p.btn_sel
            } else if hov {
                ui::mix(p.btn, p.panel_light, 0.6)
            } else {
                p.btn
            };
            draw::rect_r(s, bg, r, 0, 6);
            draw::rect_r(s, if active || hov { self.b.accent } else { p.border }, r, if active { 2 } else { 1 }, 6);
            let col = if active {
                self.b.accent
            } else if enabled {
                p.text
            } else {
                p.text_faint
            };
            self.draw_pad_icon(s, key, r, col);
            let letter = if key == "hint" { format!("{} {}", pad_key(key), MAX_HINTS - self.hints_used) } else { pad_key(key).to_string() };
            if r.h >= 26 {
                let img = self.tiny.render(&letter, true, p.text_faint);
                s.blr(&img, img.get_rect().bottomright_at((r.right() - 3, r.bottom())));
            }
        }
        for k in 0..MARK_COUNT {
            let Some(r) = self.pad_rect(&format!("mark{}", k + 1)) else {
                continue;
            };
            draw::rect_r(s, MARK_COLORS[k as usize], r, 0, 4);
            if self.marks[self.sel] == k + 1 {
                draw::rect_r(s, p.text, r.inflate(2, 2), 2, 5);
            }
        }
        let tip = if let Some(k) = hk.and_then(pad_tip) {
            Some(t(k))
        } else if hk.is_some_and(|k| k.starts_with("mark")) {
            Some(t("sud.tip.mark"))
        } else if hk.is_some_and(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit())) {
            Some(t(if self.input_digit { "sud.tip.digit_first" } else { "sud.tip.digit" }))
        } else if self.input_digit && self.brush == 0 {
            Some(t("sud.tip.digit_first"))
        } else {
            None
        };
        if let Some(tip) = tip {
            if !tip.is_empty() {
                let mut y = self.caption_y;
                for line in wrap(&tip, &self.tiny, self.pad_w).iter().take(2) {
                    if y + th > self.caption_bottom {
                        break;
                    }
                    s.bl(&self.tiny.render(line, true, p.text_dim), (self.pad_x, y));
                    y += th;
                }
            }
        }
        let hint_key = if self.input_digit { "sud.hint_digit" } else { "sud.hint" };
        self.draw_footer(s, &t(hint_key), self.tiny.get_height() + 12);
    }

    fn draw_footer(&self, s: &mut Surface, text: &str, foot: i32) {
        let p = ui::pal();
        let (cx, cy) = (self.b.width / 2, self.b.height - foot / 2);
        if let Some(msg) = &self.msg {
            if !msg.is_empty() {
                let col = self.msg_col.unwrap_or(p.red);
                let img = fit(msg, self.b.width - 40, &[&self.small, &self.tiny], col);
                let r = img.get_rect().center_at((cx, cy));
                let mut bg = r.inflate(20, 4);
                let bottom = bg.bottom().min(self.b.height - 1);
                bg.y = bottom - bg.h;
                let r = r.center_at(bg.center());
                draw::rect_r(s, p.panel, bg, 0, bg.h / 2);
                draw::rect_r(s, col, bg, 1, bg.h / 2);
                s.blr(&img, r);
                return;
            }
        }
        let hint = fit(text, self.b.width - 12, &[&self.tiny], p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, cy)));
    }

    fn draw_pad_icon(&self, s: &mut Surface, key: &str, r: Rect, col: Color) {
        let (cx, cy) = (r.centerx(), r.centery() - if r.h >= 26 { 2 } else { 0 });
        let z = 5.max(r.w.min(r.h) * 3 / 10);
        let w = 2.max(z / 4);
        match key {
            "erase" => {
                let pts = [(cx - z, cy), (cx - z / 2, cy - z * 2 / 3), (cx + z, cy - z * 2 / 3), (cx + z, cy + z * 2 / 3), (cx - z / 2, cy + z * 2 / 3)];
                draw::polygon(s, col, &pts, 1.max(w / 2 + 1));
                let k = z / 3;
                draw::line(s, col, (cx - k + 2, cy - k), (cx + k + 2, cy + k), 1.max(w / 2 + 1));
                draw::line(s, col, (cx - k + 2, cy + k), (cx + k + 2, cy - k), 1.max(w / 2 + 1));
            }
            "undo" | "redo" => {
                let rr = Rect::new(cx - z, cy - z * 2 / 3, 2 * z, 2 * z);
                draw::arc(s, col, rr, 0.0, PI, 2.max(w));
                let tip = if key == "undo" { (rr.x + 1.max(w / 2), rr.centery() + 2) } else { (rr.right() - 1.max(w / 2), rr.centery() + 2) };
                draw::polygon(s, col, &[(tip.0 - z / 2 - 1, tip.1 - 2), (tip.0 + z / 2 + 1, tip.1 - 2), (tip.0, tip.1 + z / 2 + 1)], 0);
            }
            "note" => {
                let a = (cx - z, cy + z);
                let b = (cx + z * 2 / 3, cy - z * 2 / 3);
                draw::line(s, col, (a.0 + z / 3, a.1 - z / 3), b, 3.max(w + 1));
                draw::polygon(s, col, &[a, (a.0 + z / 2, a.1 - z / 6), (a.0 + z / 6, a.1 - z / 2)], 0);
            }
            "cands" => {
                let step = 3.max(z * 2 / 3);
                for rr in 0..3 {
                    for cc in 0..3 {
                        draw::circle(s, col, (cx + (cc - 1) * step, cy + (rr - 1) * step), 1.max(z / 6) + if (rr + cc) % 2 == 0 { 1 } else { 0 }, 0);
                    }
                }
            }
            _ => {
                let rad = 3.max(z * 2 / 3);
                draw::circle(s, col, (cx, cy - z / 4), rad, 1.max(w / 2 + 1));
                draw::rect(s, col, Rect::new(cx - rad / 2, cy - z / 4 + rad, rad, 2.max(z / 3)), 0);
            }
        }
    }

    fn draw_result(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(rgba(8, 10, 16, 175));
        s.bl(&ov, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let now = pg::ticks();
        let age = now as i64 - self.win_ticks as i64;
        let sh = self.small.get_height();
        let (head, lines, star_h) = if self.won {
            let head = fit(&tf("sud.win", &[("t", &fmt_time(self.result_time as f64))]), w - 40, &[&self.huge, &self.title, &self.small], p.green);
            let mut lines = vec![(tf("common.points", &[("score", &self.b.score.to_string())]), p.text)];
            if self.daily {
                lines.push((tf("sud.daily_solved", &[("n", &self.daily_streak().to_string())]), p.gold));
            }
            if self.new_record {
                lines.push((tf("sud.new_best", &[("t", &fmt_time(self.best_time as f64))]), p.gold));
            } else {
                lines.push((tf("sud.info_best", &[("t", &fmt_time(self.best_time as f64))]), p.text_dim));
            }
            lines.push((t(if self.daily { "sud.next_daily" } else { "sud.next" }), p.text_dim));
            lines.push((t("sud.show_solution"), p.text_dim));
            (head, lines, 26.max(h / 14))
        } else {
            let head = fit(&t("sud.lose"), w - 40, &[&self.huge, &self.title, &self.small], p.red);
            (head, vec![(t("sud.retry"), p.text_dim), (t("sud.show_solution"), p.text_dim)], 0)
        };
        let imgs: Vec<Surface> = lines.iter().map(|(txt, col)| fit(txt, w - 60, &[&self.small, &self.tiny], *col)).collect();
        let widest = imgs.iter().map(|i| i.w()).max().unwrap_or(0);
        let pw = ((w - 40).min(460)).max(head.w() + 40).max(widest + 40).min(w - 16);
        let ph = 20 + head.h() + if star_h != 0 { star_h + 10 } else { 4 } + imgs.len() as i32 * (sh + 6) + 14;
        let k = 1f64.min(0f64.max(age as f64 / 260.0));
        let oy = ((1.0 - k) * 24.0) as i32;
        let panel = Rect::new(cx - pw / 2, cy - ph / 2 + oy, pw, ph);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, self.b.accent, panel, 2, 14);
        let mut y = panel.y + 14;
        s.blr(&head, head.get_rect().midtop_at((cx, y)));
        y += head.h() + 6;
        if star_h != 0 {
            let r = star_h / 2;
            for n in 0..3usize {
                let sx = cx + (n as i32 - 1) * (star_h + 12);
                let sy = y + r;
                let earned = (n as i64) < self.result_stars;
                let t0 = 350 + n as i64 * 280;
                if earned && age >= t0 {
                    let pp = 1f64.min((age - t0) as f64 / 220.0);
                    let scale = if pp < 1.0 { 1.0 + 0.5 * (PI * pp).sin() } else { 1.0 };
                    draw_star(s, (sx as f64, sy as f64), r as f64 * scale, p.gold, true);
                    if !self.star_snd.contains(&n) {
                        self.star_snd.insert(n);
                        self.b.play_sound("point");
                    }
                } else {
                    draw_star(s, (sx as f64, sy as f64), r as f64, ui::mix(p.panel, p.text_faint, 0.7), true);
                    draw_star(s, (sx as f64, sy as f64), r as f64, ui::mix(p.panel, p.text_faint, 0.9), false);
                }
            }
            y += star_h + 10;
        }
        for img in &imgs {
            s.blr(img, img.get_rect().midtop_at((cx, y)));
            y += sh + 6;
        }
    }
}

impl Game for SudokuGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.won = false;
        self.diff = self.b.opt_int_py("sudoku", "difficulty", 0).unwrap_or(0).clamp(0, 3);
        self.fail_limit = self.b.opt_value("sudoku", "fail_limit").map(|v| truthy(&v)).unwrap_or(true);
        self.last_level = match self.b.opt_value("sudoku", "last_level") {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        let v = self.b.opt_value("sudoku", "variant");
        self.variant = VARIANTS.iter().find(|x| v.as_ref().and_then(|v| v.as_str()) == Some(**x)).copied().unwrap_or("classic");
        self.input_digit = self.b.opt_value("sudoku", "input").and_then(|v| v.as_str().map(|s| s == "digit")).unwrap_or(false);
        self.colors_on = self.b.opt_value("sudoku", "colors") == Some(Value::Bool(true));
        let m = self.mode().to_string();
        self.can_notes = matches!(m.as_str(), "notes" | "comfort" | "assist");
        self.can_check = matches!(m.as_str(), "comfort" | "assist");
        self.can_hint = m == "assist";
        self.daily = false;
        self.daily_date = seedrand::today_str();
        self.play_variant = self.variant;
        self.play_diff = self.diff;
        self.level = 1;
        self.lay = gen::layout_for("classic");
        self.fresh = false;
        self.hover = None;
        self.msg = None;
        self.msg_t = 0.0;
        self.msg_col = None;
        self.gen_drawn = false;
        self.build_fonts();
        self.reset_caches();
        self.load_progress();
        self.cursor = self.get_last(self.variant, self.diff);
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.reset_caches();
        self.build_setup_layout();
        if self.state == State::Play {
            self.build_play_layout();
        }
    }

    fn on_exit(&mut self) {
        if self.state == State::Play && !self.b.game_over {
            self.write_save();
        }
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.state != State::Play {
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown {
                match ev.key.as_str() {
                    "Return" | "space" | "KP_Enter" => {
                        if self.won {
                            self.next_level();
                        } else {
                            self.restart();
                        }
                    }
                    "s" | "S" | "q" | "Q" => self.back_to_setup(),
                    "a" | "A" => {
                        self.reveal = !self.reveal;
                        self.b.play_sound("select");
                    }
                    _ => {}
                }
            } else if ev.kind == Kind::MouseDown && self.reveal {
                self.reveal = false;
            }
            return;
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.clone();
                self.handle_play_key(&k);
            }
            Kind::MouseMove => self.hover = self.pad_at(ev.pos).map(Hover::Pad),
            Kind::MouseDown => {
                let cell = self.cell_at(ev.pos);
                if ev.button == 3 {
                    if let Some(c) = cell {
                        self.sel = c;
                        self.erase(c);
                    }
                    return;
                }
                if let Some(c) = cell {
                    self.sel = c;
                    if self.input_digit && self.brush != 0 {
                        self.apply_brush(c);
                    } else {
                        self.b.play_sound("move");
                    }
                    return;
                }
                let Some(pad) = self.pad_at(ev.pos) else {
                    return;
                };
                if !pad.is_empty() && pad.chars().all(|c| c.is_ascii_digit()) {
                    let d: u8 = pad.parse().unwrap_or(0);
                    if self.input_digit {
                        let nb = if self.brush == d { 0 } else { d };
                        self.set_brush(nb);
                    } else {
                        self.enter_digit(self.sel, d);
                    }
                } else if let Some(m) = pad.strip_prefix("mark") {
                    let c: u8 = m.parse().unwrap_or(0);
                    self.set_mark(self.sel, c, true);
                } else {
                    self.pad_action(&pad);
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
        if self.state == State::Generating {
            if self.gen_drawn {
                self.do_generate();
            }
            return;
        }
        if self.state != State::Play || self.b.game_over {
            return;
        }
        self.elapsed += dt;
        if self.dirty {
            self.save_t += dt;
            if self.save_t >= SAVE_EVERY {
                self.write_save();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        match self.state {
            State::Setup => self.draw_setup(&mut s),
            State::Generating => {
                let lbl = self.b.font.render(&t("sud.generating"), true, ui::pal().text);
                s.blr(&lbl, lbl.get_rect().center_at((self.b.width / 2, self.b.height / 2)));
                self.gen_drawn = true;
            }
            State::Play => {
                self.draw_hud(&mut s);
                self.draw_board(&mut s);
                self.draw_pad(&mut s);
                if self.b.game_over && !self.reveal {
                    self.draw_result(&mut s);
                }
            }
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "sudoku",
    class_name: "SudokuGame",
    supports_multiplayer: false,
    modes: &[("classic", "sud.mode.classic"), ("notes", "sud.mode.notes"), ("comfort", "sud.mode.comfort"), ("assist", "sud.mode.assist")],
    make: SudokuGame::new,
};
