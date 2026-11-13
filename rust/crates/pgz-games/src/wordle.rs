//! wordle.rs
//! =========
//! Wordle - errate das gesuchte Wort in wenigen Versuchen.
//!
//! Uebertragen aus `games/wordle.py`. Vier Modi: Endlos, Tageswort, Dordle
//! (2 Bretter) und Quordle (4 Bretter); Setup mit Wortlaenge 4-7, hartem
//! Modus und Farbenblind-Palette; Statistik je Sprache/Laenge/Modus und
//! "Ergebnis teilen" als Emoji-Raster in die Zwischenablage.

use std::collections::HashMap;
use std::f64::consts::PI;

use pgz_core::{achievements, highscore, i18n, seedrand, store};
use pgz_engine::clip;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::wordle_words::{has_length, words_for, Lists, DEFAULT_LENGTH, LENGTHS};

// ----------------------------------------------------------------- Modi
const MODES: &[(&str, &str)] = &[
    ("endless", "wd.mode.endless"),
    ("daily", "wd.mode.daily"),
    ("dordle", "wd.mode.dordle"),
    ("quordle", "wd.mode.quordle"),
];

fn boards_of(mode: &str) -> usize {
    match mode {
        "dordle" => 2,
        "quordle" => 4,
        _ => 1,
    }
}

fn max_rows(mode: &str) -> usize {
    match mode {
        "dordle" => 7,
        "quordle" => 9,
        _ => 6,
    }
}

fn share_name(mode: &str) -> &'static str {
    match mode {
        "dordle" => "Dordle",
        "quordle" => "Quordle",
        _ => "Wordle",
    }
}

/// Nur Endlos mit dieser Wortlaenge fuellt den Highscore.
const SCORE_LENGTH: usize = 5;

#[derive(Clone, Copy, PartialEq)]
enum St {
    Setup,
    Play,
    Reveal,
    Solved,
    Done,
}

// ----------------------------------------------------------------- Farben
type Rgb = (i32, i32, i32);

fn palette(colorblind: bool) -> (Rgb, Rgb) {
    if colorblind {
        ((245, 121, 58), (133, 192, 249))
    } else {
        ((106, 170, 100), (201, 180, 88))
    }
}
const COL_ABSENT: Rgb = (58, 58, 62);
const COL_LETTER: Rgb = (240, 241, 246);
const COL_LOSE: Rgb = (225, 110, 100);

const EMOJI_ABSENT: &str = "\u{2B1B}";
const EMOJI_EMPTY: &str = "\u{2B1C}";
const EMOJI_FAIL: &str = "\u{1F7E5}";
const KEYCAP: &str = "\u{FE0F}\u{20E3}";

fn emoji(colorblind: bool, r: Res) -> &'static str {
    match (colorblind, r) {
        (false, Res::Correct) => "\u{1F7E9}",
        (false, Res::Present) => "\u{1F7E8}",
        (true, Res::Correct) => "\u{1F7E7}",
        (true, Res::Present) => "\u{1F7E6}",
        (_, Res::Absent) => EMOJI_ABSENT,
    }
}

// ----------------------------------------------------------------- Zeiten
const FLIP_TIME: f64 = 0.26;
const MESSAGE_TIME: f64 = 1.6;
const SHAKE_TIME: f64 = 0.4;
const POP_TIME: f64 = 0.11;
const BOUNCE_TIME: f64 = 0.5;
const PANEL_DELAY: f64 = 0.45;
const BANNER_SPACE: i32 = 50;

// ----------------------------------------------------------------- Tastatur
fn keyboard(lang: &str) -> [&'static str; 3] {
    match lang {
        "de" | "cs" | "sl" | "hr" => ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"],
        "fr" => ["AZERTYUIOP", "QSDFGHJKLM", "WXCVBN"],
        _ => ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"],
    }
}

/// Bewertung einer Stelle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Res {
    /// Grau - nicht enthalten.
    Absent,
    /// Gelb - im Wort, falsche Stelle.
    Present,
    /// Gruen - richtig.
    Correct,
}

fn rank(r: Option<Res>) -> i32 {
    match r {
        None => 0,
        Some(Res::Absent) => 1,
        Some(Res::Present) => 2,
        Some(Res::Correct) => 3,
    }
}

// =====================================================================
//  Spiellogik
// =====================================================================

/// `evaluate(guess, answer)` - Standard-Wordle-Bewertung.
pub fn evaluate(guess: &str, answer: &str) -> Vec<Res> {
    let g: Vec<char> = guess.chars().collect();
    let a: Vec<char> = answer.chars().collect();
    let n = a.len();
    let mut result = vec![Res::Absent; n];
    let mut rest: Vec<Option<char>> = a.iter().map(|c| Some(*c)).collect();
    for i in 0..n {
        if g[i] == a[i] {
            result[i] = Res::Correct;
            rest[i] = None;
        }
    }
    for i in 0..n {
        if result[i] == Res::Correct {
            continue;
        }
        if let Some(j) = rest.iter().position(|c| *c == Some(g[i])) {
            result[i] = Res::Present;
            rest[j] = None;
        }
    }
    result
}

/// `hard_problem(guess, history)` - (i18n-Schluessel, Werte) der ersten Verletzung.
fn hard_problem(guess: &str, history: &[(String, Vec<Res>)]) -> Option<(&'static str, Vec<(&'static str, String)>)> {
    let g: Vec<char> = guess.chars().collect();
    for (word, result) in history {
        let w: Vec<char> = word.chars().collect();
        for (i, st) in result.iter().enumerate() {
            if *st == Res::Correct && g[i] != w[i] {
                return Some(("wd.hard_green", vec![("n", (i + 1).to_string()), ("c", w[i].to_string())]));
            }
        }
        let mut need: Vec<(char, usize)> = Vec::new();
        for (i, st) in result.iter().enumerate() {
            if *st != Res::Absent {
                match need.iter_mut().find(|(c, _)| *c == w[i]) {
                    Some(e) => e.1 += 1,
                    None => need.push((w[i], 1)),
                }
            }
        }
        for (ch, count) in need {
            if g.iter().filter(|c| **c == ch).count() < count {
                return Some(("wd.hard_yellow", vec![("c", ch.to_string())]));
            }
        }
    }
    None
}

thread_local! {
    static ORDER_CACHE: std::cell::RefCell<HashMap<(String, usize, usize), Vec<usize>>> = std::cell::RefCell::new(HashMap::new());
}

/// `daily_answer(words, lang, length, date)` - das Tageswort.
fn daily_answer(words: &[String], lang: &str, length: usize, date: Option<&str>) -> String {
    let n = words.len();
    let key = (lang.to_string(), length, n);
    let order = ORDER_CACHE.with(|c| {
        c.borrow_mut()
            .entry(key)
            .or_insert_with(|| {
                let mut order: Vec<usize> = (0..n).collect();
                seedrand::Rand::new(seedrand::seed_from(&["wordle", lang, &length.to_string()])).shuffle(&mut order);
                order
            })
            .clone()
    });
    words[order[seedrand::day_index(date).rem_euclid(n as i64) as usize]].clone()
}

/// `format_hms(seconds)`
fn format_hms(seconds: i64) -> String {
    let s = seconds.max(0);
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

fn seconds_to_midnight() -> i64 {
    seedrand::seconds_to_midnight(py::time_time())
}

// ----------------------------------------------------------------- Statistik

#[derive(Clone, Debug)]
struct Stats {
    played: i64,
    won: i64,
    streak: i64,
    best: i64,
    dist: Vec<i64>,
    last_day: Option<i64>,
}

impl Stats {
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("played".into(), json!(self.played));
        m.insert("won".into(), json!(self.won));
        m.insert("streak".into(), json!(self.streak));
        m.insert("best".into(), json!(self.best));
        m.insert("dist".into(), json!(self.dist));
        if let Some(d) = self.last_day {
            m.insert("last_day".into(), json!(d));
        }
        Value::Object(m)
    }
}

/// `isinstance(v, int)` fuer JSON-Werte (Wahrheitswerte zaehlen wie in Python mit).
fn py_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64(),
        Some(Value::Bool(b)) => Some(*b as i64),
        _ => None,
    }
}

fn stats_key(mode: &str, lang: &str, length: usize) -> String {
    format!("{mode}:{lang}:{length}")
}

/// `load_data()` - mem.json-Section 'wordle' mit Ergaenzungen.
fn load_data() -> Map<String, Value> {
    let mut data = store::load_section("wordle");
    for key in ["stats", "daily", "best"] {
        if !data.get(key).is_some_and(|v| v.is_object()) {
            data.insert(key.into(), Value::Object(Map::new()));
        }
    }
    data
}

fn save_data(data: &Map<String, Value>) {
    store::save_section("wordle", data.clone());
}

fn section<'a>(data: &'a Map<String, Value>, key: &str) -> Option<&'a Map<String, Value>> {
    data.get(key).and_then(|v| v.as_object())
}

fn section_mut<'a>(data: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    if !data.get(key).is_some_and(|v| v.is_object()) {
        data.insert(key.into(), Value::Object(Map::new()));
    }
    data.get_mut(key).and_then(|v| v.as_object_mut()).expect("section")
}

/// `get_stats(data, mode, lang, length)`
fn get_stats(data: &Map<String, Value>, mode: &str, lang: &str, length: usize) -> Stats {
    let rows = max_rows(mode);
    let raw = section(data, "stats").and_then(|s| s.get(&stats_key(mode, lang, length))).and_then(|v| v.as_object());
    let empty = Map::new();
    let raw = raw.unwrap_or(&empty);
    let num = |key: &str| py_int(raw.get(key)).filter(|v| *v >= 0).unwrap_or(0);
    let mut dist: Vec<i64> = match raw.get("dist") {
        Some(Value::Array(a)) => a.iter().map(|v| py_int(Some(v)).filter(|x| *x >= 0).unwrap_or(0)).take(rows).collect(),
        _ => Vec::new(),
    };
    while dist.len() < rows {
        dist.push(0);
    }
    let mut out = Stats { played: num("played"), won: num("won"), streak: num("streak"), best: num("best"), dist, last_day: None };
    if let Some(d) = py_int(raw.get("last_day")) {
        out.last_day = Some(d);
    }
    out.won = out.won.min(out.played);
    out
}

/// `record_game(data, mode, lang, length, won, tries, day)`
fn record_game(data: &mut Map<String, Value>, mode: &str, lang: &str, length: usize, won: bool, tries: usize, day: Option<i64>) -> Stats {
    let mut st = get_stats(data, mode, lang, length);
    st.played += 1;
    if won {
        st.won += 1;
        if 1 <= tries && tries <= st.dist.len() {
            st.dist[tries - 1] += 1;
        }
        match day {
            Some(d) if st.last_day != Some(d - 1) => st.streak = 1,
            _ => st.streak += 1,
        }
        st.best = st.best.max(st.streak);
        if let Some(d) = day {
            st.last_day = Some(d);
        }
    } else {
        st.streak = 0;
    }
    section_mut(data, "stats").insert(stats_key(mode, lang, length), st.to_value());
    st
}

/// `shown_streak(st, mode, today)`
fn shown_streak(st: &Stats, mode: &str) -> i64 {
    if mode != "daily" {
        return st.streak;
    }
    let today = seedrand::day_index(None);
    if st.last_day == Some(today) || st.last_day == Some(today - 1) {
        return st.streak;
    }
    0
}

// ----------------------------------------------------------------- Teilen

/// `share_text(mode, lang, length, boards, hard, colorblind, date)`
fn share_text(mode: &str, lang: &str, length: usize, boards: &[(Vec<Vec<Res>>, Option<usize>)], hard: bool, colorblind: bool, date: Option<&str>) -> String {
    let rows = max_rows(mode);
    let mut head = format!("PyGameZ {}", share_name(mode));
    if mode == "daily" {
        head += " ";
        head += &date.map(|d| d.to_string()).unwrap_or_else(seedrand::today_str);
    }
    head += &format!(" \u{00B7} {} \u{00B7} {}", lang.to_uppercase(), length);
    let star = if hard { "*" } else { "" };
    let line = |results: &Vec<Vec<Res>>, r: usize| -> String {
        if r >= results.len() {
            return EMOJI_EMPTY.repeat(length);
        }
        results[r].iter().map(|st| emoji(colorblind, *st)).collect()
    };
    let score = |solved: Option<usize>| solved.map(|s| s.to_string()).unwrap_or_else(|| "X".into());
    let mut lines: Vec<String> = Vec::new();
    if boards.len() == 1 {
        let (results, solved) = &boards[0];
        lines.push(format!("{head} {}/{rows}{star}", score(*solved)));
        lines.push(String::new());
        for r in 0..results.len() {
            lines.push(line(results, r));
        }
    } else if boards.len() == 2 {
        let sc: Vec<String> = boards.iter().map(|(_, s)| score(*s)).collect();
        lines.push(format!("{head} {}/{rows}{star}", sc.join("&")));
        lines.push(String::new());
        let used = boards.iter().map(|(r, _)| r.len()).max().unwrap_or(0);
        for r in 0..used {
            lines.push(line(&boards[0].0, r) + " " + &line(&boards[1].0, r));
        }
    } else {
        let badge = |solved: Option<usize>| match solved {
            Some(s) => s.to_string() + KEYCAP,
            None => EMOJI_FAIL.to_string(),
        };
        lines.push(head.clone() + star);
        lines.push(badge(boards[0].1) + &badge(boards[1].1));
        lines.push(badge(boards[2].1) + &badge(boards[3].1));
        lines.push(String::new());
        for (a, b) in [(0usize, 1usize), (2, 3)] {
            let used = boards[a].0.len().max(boards[b].0.len());
            for r in 0..used {
                lines.push(line(&boards[a].0, r) + " " + &line(&boards[b].0, r));
            }
            if a == 0 {
                lines.push(String::new());
            }
        }
    }
    lines.join("\n")
}

// =====================================================================
//  Ein Brett
// =====================================================================

struct Board {
    answer: String,
    rows: Vec<(String, Vec<Res>)>,
    solved_at: Option<usize>,
    keys: HashMap<char, Res>,
    solve_t: Option<f64>,
}

impl Board {
    fn new(answer: String) -> Board {
        Board { answer, rows: Vec::new(), solved_at: None, keys: HashMap::new(), solve_t: None }
    }

    fn solved(&self) -> bool {
        self.solved_at.is_some()
    }

    fn apply(&mut self, guess: &str, result: &[Res], anim_t: f64) {
        self.rows.push((guess.to_string(), result.to_vec()));
        for (ch, st) in guess.chars().zip(result) {
            if rank(Some(*st)) > rank(self.keys.get(&ch).copied()) {
                self.keys.insert(ch, *st);
            }
        }
        if result.iter().all(|s| *s == Res::Correct) {
            self.solved_at = Some(self.rows.len());
            self.solve_t = Some(anim_t);
        }
    }
}

struct Reveal {
    guess: String,
    results: Vec<Option<Vec<Res>>>,
    t0: f64,
    end: f64,
    ticks: usize,
}

struct ResultInfo {
    won: bool,
    tries: usize,
    stats: Stats,
    t0: f64,
    new_best: bool,
}

#[derive(Clone, Copy)]
enum Action {
    Share,
    Primary,
    Setup,
}

#[derive(Clone, Copy)]
enum Anc {
    Center(f64, f64),
    MidLeft(f64, f64),
    MidRight(f64, f64),
    MidTop(f64, f64),
    MidBottom(f64, f64),
}

fn place(r: Rect, a: Anc) -> Rect {
    match a {
        Anc::Center(x, y) => r.center_at((x, y)),
        Anc::MidLeft(x, y) => r.midleft_at((x, y)),
        Anc::MidRight(x, y) => r.midright_at((x, y)),
        Anc::MidTop(x, y) => r.midtop_at((x, y)),
        Anc::MidBottom(x, y) => r.midbottom_at((x, y)),
    }
}

fn ctr(p: (i32, i32)) -> Anc {
    Anc::Center(p.0 as f64, p.1 as f64)
}

type BoardKey = (usize, i32, i32, usize, usize, usize, Option<usize>, bool, bool, &'static str, Color, Color);

/// `WordleGame`
pub struct WordleGame {
    b: GameBase,
    game_mode: &'static str,
    lang: String,
    length: usize,
    hard: bool,
    colorblind: bool,
    data: Map<String, Value>,
    anim: f64,
    updated: bool,
    last_draw: Option<f64>,
    state: St,
    setup_focus: i32,
    setup_t: f64,
    toggle_hard: Option<f64>,
    toggle_cb: Option<f64>,
    boards: Vec<Board>,
    guesses: Vec<String>,
    current: String,
    reveal: Option<Reveal>,
    points: i64,
    last_points: i64,
    solved_count: i64,
    solved_t: f64,
    message: String,
    message_t: f64,
    shake: f64,
    pops: HashMap<usize, f64>,
    result: Option<ResultInfo>,
    panel_hidden: bool,
    daily_date: Option<String>,
    daily_view: bool,
    words: Option<Lists>,
    fit_cache: HashMap<(String, i32, i32, bool), Font>,
    letter_cache: HashMap<(i32, char), Surface>,
    board_cache: HashMap<usize, (BoardKey, Surface)>,
    hud: Font,
    small: Font,
    small_b: Font,
    tiny: Font,
    big: Font,
    num: Font,
    key_font: Font,
    su_tile: i32,
    su_title_y: i32,
    su_sub_y: i32,
    su_label_y: i32,
    su_seg: Vec<Rect>,
    su_hard: Rect,
    su_cb: Rect,
    su_start: Rect,
    su_stats_rect: Rect,
    hud_h: i32,
    gear_rect: Rect,
    key_h: i32,
    key_gap: i32,
    kb_top: i32,
    key_rects: Vec<(char, Rect)>,
    enter_rect: Option<Rect>,
    del_rect: Option<Rect>,
    tile: i32,
    tile_gap: i32,
    board_w: i32,
    board_h: i32,
    board_inner: i32,
    board_pos: Vec<(i32, i32)>,
    grid_top: i32,
    grid_bottom: i32,
    msg_y: i32,
    done_buttons: Vec<(Rect, Action)>,
    done_panel: Option<Rect>,
    dim: Option<((i32, i32), Surface)>,
}

impl WordleGame {
    /// `WordleGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = WordleGame {
            b,
            game_mode: "endless",
            lang: String::new(),
            length: DEFAULT_LENGTH,
            hard: false,
            colorblind: false,
            data: Map::new(),
            anim: 0.0,
            updated: false,
            last_draw: None,
            state: St::Setup,
            setup_focus: 3,
            setup_t: 0.0,
            toggle_hard: None,
            toggle_cb: None,
            boards: Vec::new(),
            guesses: Vec::new(),
            current: String::new(),
            reveal: None,
            points: 0,
            last_points: 0,
            solved_count: 0,
            solved_t: 0.0,
            message: String::new(),
            message_t: 0.0,
            shake: 0.0,
            pops: HashMap::new(),
            result: None,
            panel_hidden: false,
            daily_date: None,
            daily_view: false,
            words: None,
            fit_cache: HashMap::new(),
            letter_cache: HashMap::new(),
            board_cache: HashMap::new(),
            hud: f.clone(),
            small: f.clone(),
            small_b: f.clone(),
            tiny: f.clone(),
            big: f.clone(),
            num: f.clone(),
            key_font: f,
            su_tile: 22,
            su_title_y: 10,
            su_sub_y: 0,
            su_label_y: 0,
            su_seg: Vec::new(),
            su_hard: z,
            su_cb: z,
            su_start: z,
            su_stats_rect: z,
            hud_h: 28,
            gear_rect: z,
            key_h: 22,
            key_gap: 2,
            kb_top: 0,
            key_rects: Vec::new(),
            enter_rect: None,
            del_rect: None,
            tile: 6,
            tile_gap: 1,
            board_w: 0,
            board_h: 0,
            board_inner: 0,
            board_pos: Vec::new(),
            grid_top: 0,
            grid_bottom: 0,
            msg_y: 0,
            done_buttons: Vec::new(),
            done_panel: None,
            dim: None,
        };
        g.reset();
        Box::new(g)
    }

    /// `_opts()` - (Laenge, hart, farbenblind) aus settings.json.
    fn opts(&self) -> (usize, bool, bool) {
        let length = match self.b.opt_value("wordle", "length") {
            None => DEFAULT_LENGTH,
            Some(v) => match v {
                Value::Number(n) if n.is_i64() || n.is_u64() => n.as_i64().filter(|l| LENGTHS.contains(&(*l as usize)) && *l > 0).map(|l| l as usize).unwrap_or(DEFAULT_LENGTH),
                _ => DEFAULT_LENGTH,
            },
        };
        let is_true = |k: &str| self.b.opt_value("wordle", k) == Some(Value::Bool(true));
        (length, is_true("hard"), is_true("colorblind"))
    }

    fn save_opt(&mut self, key: &str, value: Value) {
        self.b.set_opt("wordle", key, value);
    }

    fn rows(&self) -> usize {
        max_rows(self.game_mode)
    }

    fn n_boards(&self) -> usize {
        boards_of(self.game_mode)
    }

    fn counts_score(&self) -> bool {
        self.game_mode == "endless" && self.length == SCORE_LENGTH
    }

    fn pal_correct(&self) -> Rgb {
        palette(self.colorblind).0
    }

    fn k(&self) -> f64 {
        (self.b.width as f64 / 800.0).min(self.b.height as f64 / 600.0)
    }

    fn make_fonts(&mut self) {
        let k = self.k();
        self.hud = ui::font(12.max((17.0 * k) as i32), true, false);
        self.small = ui::font(11.max((15.0 * k) as i32), false, false);
        self.small_b = ui::font(11.max((15.0 * k) as i32), true, false);
        self.tiny = ui::font(10.max((12.0 * k) as i32), false, false);
        self.big = ui::font(18.max((30.0 * k) as i32), true, false);
        self.num = ui::font(15.max((26.0 * k) as i32), true, false);
    }

    /// `_fit(text, max_w, px, bold)` - groesste Schrift <= px, die passt.
    fn fit(&mut self, text: &str, max_w: i32, px: i32, bold: bool) -> Font {
        let key = (text.to_string(), max_w, px, bold);
        if let Some(f) = self.fit_cache.get(&key) {
            return f.clone();
        }
        let mut size = px;
        let mut f = ui::font(size, bold, false);
        while size > 9 && f.size(text).0 > max_w {
            size -= 1;
            f = ui::font(size, bold, false);
        }
        if self.fit_cache.len() > 600 {
            self.fit_cache.clear();
        }
        self.fit_cache.insert(key, f.clone());
        f
    }

    fn fits(&mut self, text: &str, max_w: i32, px: i32) -> bool {
        self.fit(text, max_w, px, false).size(text).0 <= max_w
    }

    fn fit_parts(&mut self, text: &str, max_w: i32, px: i32) -> String {
        let mut parts: Vec<&str> = text.split(" \u{00B7} ").collect();
        while parts.len() > 1 && !self.fits(&parts.join(" \u{00B7} "), max_w, px) {
            parts.pop();
        }
        parts.join(" \u{00B7} ")
    }

    fn ellipsis(f: &Font, text: &str, max_w: i32) -> String {
        let mut chars: Vec<char> = text.chars().collect();
        while chars.len() > 1 && f.size(&(chars.iter().collect::<String>() + "\u{2026}")).0 > max_w {
            chars.pop();
        }
        chars.iter().collect::<String>().trim_end().to_string() + "\u{2026}"
    }

    /// `_text(s, text, px, color, max_w, bold, **anchor)`
    #[allow(clippy::too_many_arguments)]
    fn text(&mut self, s: &mut Surface, text: &str, px: i32, color: impl Col, max_w: i32, bold: bool, anchor: Anc) -> Rect {
        let f = self.fit(text, max_w, px, bold);
        let mut text = text.to_string();
        if f.size(&text).0 > max_w && max_w > 8 {
            text = Self::ellipsis(&f, &text, max_w);
        }
        let img = f.render(&text, true, color.c());
        let rc = place(img.get_rect(), anchor);
        s.blr(&img, rc);
        rc
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        self.layout_setup();
        self.layout_play();
    }

    fn layout_setup(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let k = self.k();
        self.su_tile = 22.max((46.0 * k) as i32);
        self.su_title_y = 10.max((26.0 * k) as i32);
        self.su_sub_y = self.su_title_y + self.su_tile + 6.max((12.0 * k) as i32);
        let top = self.su_sub_y + self.small.get_height() + 8.max((16.0 * k) as i32);
        let footer = 22.max((34.0 * k) as i32);
        let bottom = h - footer;
        let margin = 12.max((34.0 * k) as i32);
        let gap = 10.max((22.0 * k) as i32);
        let col_w = py::fdiv32(w - 2 * margin - gap, 2);
        let left = Rect::new(margin, top, col_w, bottom - top);
        let label_h = self.small.get_height();
        let seg_h = 28.max((46.0 * k) as i32);
        let row_h = 36.max((58.0 * k) as i32);
        let start_h = 32.max((50.0 * k) as i32);
        let sp = 6.max((12.0 * k) as i32);
        let total = label_h + 4 + seg_h + sp + row_h + sp + row_h + sp * 2 + start_h;
        let mut y = left.y + 0.max(py::fdiv32(left.h - total, 2));
        self.su_label_y = y;
        y += label_h + 4;
        let sg = 4.max((8.0 * k) as i32);
        let seg_w = py::fdiv32(col_w - 3 * sg, 4);
        self.su_seg = (0..4).map(|i| Rect::new(left.x + i * (seg_w + sg), y, seg_w, seg_h)).collect();
        y += seg_h + sp;
        self.su_hard = Rect::new(left.x, y, col_w, row_h);
        y += row_h + sp;
        self.su_cb = Rect::new(left.x, y, col_w, row_h);
        y += row_h + sp * 2;
        self.su_start = Rect::new(left.x, y, col_w, start_h);
        let st_top = top.max(self.su_label_y - 4.max((6.0 * k) as i32));
        self.su_stats_rect = Rect::new(margin + col_w + gap, st_top, col_w, (self.su_start.bottom() - st_top).max((250.0 * k) as i32));
    }

    fn layout_play(&mut self) {
        let (w, _h) = (self.b.width, self.b.height);
        let k = self.k();
        self.hud_h = 28.max((44.0 * k) as i32);
        self.gear_rect = Rect::new(w - self.hud_h + 2, 3, self.hud_h - 6, self.hud_h - 6);
        self.build_keyboard();
        let n = self.n_boards();
        let pad = if n == 1 { 8.max((20.0 * k) as i32) } else { 4.max((14.0 * k) as i32) };
        let lane = if n == 1 { 16.max((30.0 * k) as i32) } else { 4.max((8.0 * k) as i32) };
        let top = self.hud_h + lane;
        let bottom = self.kb_top - 6.max((10.0 * k) as i32);
        let area = Rect::new(pad, top, w - 2 * pad, 40.max(bottom - top));
        let (l, r) = (self.length as f64, self.rows() as f64);
        let options: &[(i32, i32)] = match n {
            1 => &[(1, 1)],
            2 => &[(2, 1), (1, 2)],
            _ => &[(4, 1), (2, 2)],
        };
        let mut best: Option<(i32, i32, i32, i32, i32, f64)> = None;
        for &(bc, br) in options {
            let bgap = if n > 1 { 6.max((16.0 * k) as i32) } else { 0 };
            let inner = if n > 1 { 3.max((7.0 * k) as i32) } else { 0 };
            let cell_w = (area.w - (bc - 1) * bgap) as f64 / bc as f64 - (2 * inner) as f64;
            let cell_h = (area.h - (br - 1) * bgap) as f64 / br as f64 - (2 * inner) as f64;
            let tg_ratio = if n == 1 { 0.09 } else { 0.07 };
            let tile = (cell_w / (l + (l - 1.0) * tg_ratio)).min(cell_h / (r + (r - 1.0) * tg_ratio)) as i32;
            if best.is_none_or(|b| tile > b.0) {
                best = Some((tile, bc, br, bgap, inner, tg_ratio));
            }
        }
        let (tile, bc, br, bgap, inner, tg_ratio) = best.unwrap_or((6, 1, 1, 0, 0, 0.09));
        let cap = 20.max(((if n == 1 { 64.0 } else { 52.0 }) * k) as i32);
        let tile = 6.max(tile.min(cap));
        let tgap = 1.max(py::round(tile as f64 * tg_ratio) as i32);
        let (li, ri) = (self.length as i32, self.rows() as i32);
        let bw = li * tile + (li - 1) * tgap;
        let bh = ri * tile + (ri - 1) * tgap;
        let total_w = bc * (bw + 2 * inner) + (bc - 1) * bgap;
        let total_h = br * (bh + 2 * inner) + (br - 1) * bgap;
        let x0 = py::fdiv32(w - total_w, 2);
        let y0 = area.y + 0.max(py::fdiv32(area.h - total_h, 2));
        self.tile = tile;
        self.tile_gap = tgap;
        self.board_w = bw;
        self.board_h = bh;
        self.board_inner = inner;
        self.board_pos = (0..n as i32).map(|i| {
            let (cx, cy) = (i % bc, i / bc);
            (x0 + inner + cx * (bw + 2 * inner + bgap), y0 + inner + cy * (bh + 2 * inner + bgap))
        }).collect();
        self.grid_top = y0;
        self.grid_bottom = y0 + total_h;
        self.msg_y = if n == 1 { (self.hud_h + lane / 2 + 2).max(y0 - lane / 2) } else { y0 + 12.max((20.0 * k) as i32) };
        self.board_cache.clear();
        self.letter_cache.clear();
    }

    fn build_keyboard(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let k = self.k();
        let layout = keyboard(&self.lang);
        self.key_h = 22.max(((h as f64 * 0.078) as i32).min(64));
        self.key_gap = 2.max((5.0 * k) as i32);
        let pad = 6.max((12.0 * k) as i32);
        let mut kw = (self.key_h as f64 * 1.05) as i32;
        for (ri, row) in layout.iter().enumerate() {
            let units = row.len() as f64 + if ri == 2 { 3.0 } else { 0.0 };
            let items = row.len() as i32 + if ri == 2 { 2 } else { 0 };
            kw = kw.min(((w - 2 * pad - (items - 1) * self.key_gap) as f64 / units) as i32);
        }
        let kw = 12.max(kw);
        let special = (kw as f64 * 1.5) as i32;
        let kb_h = 3 * self.key_h + 2 * self.key_gap;
        self.kb_top = h - kb_h - 4.max((8.0 * k) as i32);
        self.key_rects.clear();
        self.enter_rect = None;
        self.del_rect = None;
        let mut y = self.kb_top;
        for (ri, row) in layout.iter().enumerate() {
            let n = row.len() as i32;
            let mut row_w = n * kw + (n - 1) * self.key_gap;
            if ri == 2 {
                row_w += 2 * (special + self.key_gap);
            }
            let mut x = py::fdiv32(w - row_w, 2);
            if ri == 2 {
                self.enter_rect = Some(Rect::new(x, y, special, self.key_h));
                x += special + self.key_gap;
            }
            for ch in row.chars() {
                self.key_rects.push((ch, Rect::new(x, y, kw, self.key_h)));
                x += kw + self.key_gap;
            }
            if ri == 2 {
                self.del_rect = Some(Rect::new(x, y, special, self.key_h));
            }
            y += self.key_h + self.key_gap;
        }
        self.key_font = ui::font(10.max(((self.key_h as f64 * 0.42) as i32).min((kw as f64 * 0.62) as i32)), true, false);
    }

    // ===================================================== Partie starten

    fn start(&mut self) {
        self.lang = i18n::get_language();
        let (length, hard, cb) = self.opts();
        self.length = length;
        self.hard = hard;
        self.colorblind = cb;
        if !has_length(&self.lang, self.length) {
            self.length = DEFAULT_LENGTH;
        }
        self.words = Some(words_for(&self.lang, self.length));
        self.data = load_data();
        self.b.game_over = false;
        self.result = None;
        self.panel_hidden = false;
        self.daily_view = false;
        self.points = 0;
        self.b.score = 0;
        self.solved_count = 0;
        self.layout_play();
        if self.game_mode == "daily" {
            self.start_daily();
        } else {
            self.new_round();
        }
        self.b.play_sound("click");
    }

    fn new_round(&mut self) {
        let pool = self.words.clone().unwrap_or_else(|| words_for(&self.lang, self.length));
        let pool = &pool.0;
        let n = self.n_boards();
        let answers: Vec<String> = if pool.len() >= n {
            let idx: Vec<usize> = (0..pool.len()).collect();
            py::sample(&idx, n).into_iter().map(|i| pool[i].clone()).collect()
        } else {
            (0..n).map(|_| pool[py::choice_index(pool.len())].clone()).collect()
        };
        self.boards = answers.into_iter().map(Board::new).collect();
        self.clear_round();
        self.state = St::Play;
    }

    fn clear_round(&mut self) {
        self.guesses.clear();
        self.current.clear();
        self.reveal = None;
        self.message.clear();
        self.message_t = 0.0;
        self.shake = 0.0;
        self.pops.clear();
        self.board_cache.clear();
    }

    fn daily_slot(&self) -> String {
        format!("{}:{}", self.lang, self.length)
    }

    fn start_daily(&mut self) {
        let today = seedrand::today_str();
        self.daily_date = Some(today.clone());
        let words = self.words.clone().unwrap_or_else(|| words_for(&self.lang, self.length));
        let answer = daily_answer(&words.0, &self.lang, self.length, Some(&today));
        self.boards = vec![Board::new(answer.clone())];
        self.clear_round();
        self.state = St::Play;
        let prog = section(&self.data, "daily").and_then(|d| d.get(&self.daily_slot())).and_then(|v| v.as_object()).cloned();
        let Some(prog) = prog else {
            return;
        };
        if prog.get("date") != Some(&Value::String(today.clone())) || prog.get("answer") != Some(&Value::String(answer.clone())) {
            return;
        }
        let rows = self.rows();
        let guesses: Vec<Value> = match prog.get("guesses") {
            Some(Value::Array(a)) => a.iter().take(rows).cloned().collect(),
            _ => Vec::new(),
        };
        for g in guesses {
            let Value::String(guess) = g else {
                break;
            };
            if !(guess.chars().count() == self.length && guess.is_ascii() && guess.chars().all(|c| c.is_ascii_alphabetic())) {
                break;
            }
            let guess = guess.to_uppercase();
            self.guesses.push(guess.clone());
            let res = evaluate(&guess, &answer);
            self.boards[0].apply(&guess, &res, -99.0);
            if self.boards[0].solved() {
                break;
            }
        }
        if self.boards[0].solved() || self.guesses.len() >= rows {
            let st = get_stats(&self.data, "daily", &self.lang, self.length);
            self.daily_view = true;
            let board = &self.boards[0];
            self.result = Some(ResultInfo { won: board.solved(), tries: board.solved_at.unwrap_or(0), stats: st, t0: self.anim - PANEL_DELAY, new_best: false });
            self.state = St::Done;
        }
    }

    fn save_daily(&mut self, done: bool) {
        let date = self.daily_date.clone();
        let slot = self.daily_slot();
        let mut m = Map::new();
        m.insert("date".into(), date.clone().map(Value::String).unwrap_or(Value::Null));
        m.insert("answer".into(), json!(self.boards[0].answer));
        m.insert("guesses".into(), json!(self.guesses));
        m.insert("done".into(), json!(done));
        let daily = section_mut(&mut self.data, "daily");
        daily.insert(slot, Value::Object(m));
        let date_v = date.map(Value::String).unwrap_or(Value::Null);
        let old: Vec<String> = daily
            .iter()
            .filter(|(_, p)| !p.as_object().is_some_and(|o| o.get("date").unwrap_or(&Value::Null) == &date_v))
            .map(|(s, _)| s.clone())
            .collect();
        for s in old {
            daily.shift_remove(&s);
        }
    }

    // ===================================================== Eingabe

    fn is_continue(ev: &InputEvent) -> bool {
        ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "KP_Enter", "space"]))
    }

    fn type_ch(&mut self, ch: char) {
        if self.current.chars().count() < self.length {
            self.pops.insert(self.current.chars().count(), self.anim);
            self.current.push(ch);
            self.b.play_sound("click");
        }
    }

    fn backspace(&mut self) {
        if !self.current.is_empty() {
            self.current.pop();
            self.pops.remove(&self.current.chars().count());
            self.b.play_sound("move");
        }
    }

    fn click_keyboard(&mut self, pos: (i32, i32)) {
        if self.enter_rect.is_some_and(|r| r.hit(pos)) {
            self.submit();
            return;
        }
        if self.del_rect.is_some_and(|r| r.hit(pos)) {
            self.backspace();
            return;
        }
        if let Some(&(ch, _)) = self.key_rects.iter().find(|(_, r)| r.hit(pos)) {
            self.type_ch(ch);
        }
    }

    fn reveal_step(&self) -> f64 {
        if self.length <= 5 {
            0.13
        } else {
            0.105
        }
    }

    fn submit(&mut self) {
        if self.current.chars().count() != self.length {
            let m = tf("wd.too_short", &[("n", &self.length.to_string())]);
            self.reject(m);
            return;
        }
        let allowed = self.words.as_ref().is_some_and(|w| w.1.contains(&self.current));
        if !allowed {
            self.reject(t("wd.not_a_word"));
            return;
        }
        if self.hard {
            let problem = self.hard_check(&self.current.clone());
            if !problem.is_empty() {
                self.reject(problem);
                return;
            }
        }
        let results: Vec<Option<Vec<Res>>> = self.boards.iter().map(|b| if b.solved() { None } else { Some(evaluate(&self.current, &b.answer)) }).collect();
        let step = self.reveal_step();
        self.reveal = Some(Reveal {
            guess: self.current.clone(),
            results,
            t0: self.anim,
            end: self.anim + (self.length - 1) as f64 * step + FLIP_TIME + 0.06,
            ticks: 0,
        });
        self.state = St::Reveal;
        self.b.play_sound("select");
    }

    fn hard_check(&self, guess: &str) -> String {
        let problems: Vec<_> = self.boards.iter().filter(|b| !b.solved()).map(|b| hard_problem(guess, &b.rows)).collect();
        if problems.is_empty() || problems.iter().any(|p| p.is_none()) {
            return String::new();
        }
        if self.boards.len() > 1 {
            return t("wd.hard_multi");
        }
        let (key, values) = problems[0].clone().unwrap_or(("", Vec::new()));
        let vals: Vec<(&str, &str)> = values.iter().map(|(k, v)| (*k, v.as_str())).collect();
        tf(key, &vals)
    }

    fn reject(&mut self, text: String) {
        self.say(text, MESSAGE_TIME);
        self.shake = SHAKE_TIME;
        self.b.play_sound("hit");
        self.b.rumble(60);
    }

    fn say(&mut self, text: String, seconds: f64) {
        self.message = text;
        self.message_t = seconds;
    }

    fn finish_reveal(&mut self) {
        let Some(rv) = self.reveal.take() else {
            return;
        };
        let guess = rv.guess.clone();
        self.guesses.push(guess.clone());
        let mut solved_now = Vec::new();
        for (i, result) in rv.results.iter().enumerate() {
            let Some(result) = result else {
                continue;
            };
            self.boards[i].apply(&guess, result, self.anim);
            if self.boards[i].solved() {
                solved_now.push(i);
            }
        }
        self.current.clear();
        self.pops.clear();
        self.board_cache.clear();
        let n = self.guesses.len();
        let all_solved = self.boards.iter().all(|b| b.solved());
        if self.game_mode == "daily" {
            self.save_daily(all_solved || n >= self.rows());
            save_data(&self.data);
        }
        for i in solved_now {
            if self.boards.len() > 1 && !all_solved {
                self.b.play_sound("point");
                let (bx, by) = self.board_pos[i];
                let c = self.pal_correct();
                ui::spawn_burst((bx + self.board_w / 2) as f64, (by + self.board_h / 2) as f64, Some(rgb(c.0, c.1, c.2)), 18, 260.0);
            }
        }
        if all_solved {
            self.round_won(n);
        } else if n >= self.rows() {
            self.round_lost(n);
        } else {
            self.state = St::Play;
        }
    }

    fn round_won(&mut self, n: usize) {
        let mode = self.game_mode;
        if mode == "endless" {
            let pts = 10 + (self.rows() as i64 - n as i64) * 10;
            self.last_points = pts;
            self.points += pts;
            if self.counts_score() {
                self.b.score = self.points;
            }
            self.solved_count += 1;
            record_game(&mut self.data, mode, &self.lang, self.length, true, n, None);
            save_data(&self.data);
            self.b.report_result(true);
            if n <= 2 {
                self.b.ach_event("wordle_two", None);
            }
            self.solved_t = self.anim;
            self.state = St::Solved;
            self.b.play_sound("win");
            return;
        }
        let day = if mode == "daily" { Some(seedrand::day_index(self.daily_date.as_deref())) } else { None };
        let st = record_game(&mut self.data, mode, &self.lang, self.length, true, n, day);
        save_data(&self.data);
        self.b.report_result(true);
        if mode == "daily" {
            if n <= 2 {
                self.b.ach_event("wordle_two", None);
            }
            self.b.ach_event("wordle_daily7", Some(st.streak));
        } else if mode == "quordle" {
            self.b.ach_event("wordle_quordle", None);
        }
        self.finish_round(true, n, st, false);
        ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 70);
        self.b.play_sound("win");
    }

    fn round_lost(&mut self, n: usize) {
        let mode = self.game_mode;
        let day = if mode == "daily" { Some(seedrand::day_index(self.daily_date.as_deref())) } else { None };
        let st = record_game(&mut self.data, mode, &self.lang, self.length, false, n, day);
        let mut new_best = false;
        if mode == "endless" {
            let key = self.length.to_string();
            let best = section(&self.data, "best").and_then(|b| py_int(b.get(&key))).unwrap_or(0);
            if self.points > best {
                section_mut(&mut self.data, "best").insert(key, json!(self.points));
                new_best = best > 0 || self.points > 0;
            }
            if self.solved_count == 0 {
                self.b.report_result(false);
            }
        } else {
            self.b.report_result(false);
        }
        save_data(&self.data);
        self.finish_round(false, n, st, new_best);
        self.b.play_sound("gameover");
        self.b.rumble(160);
    }

    fn finish_round(&mut self, won: bool, n: usize, st: Stats, new_best: bool) {
        self.result = Some(ResultInfo { won, tries: n, stats: st, t0: self.anim, new_best });
        self.panel_hidden = false;
        self.state = St::Done;
        self.b.game_over = true;
    }

    fn to_setup(&mut self) {
        if matches!(self.state, St::Play | St::Reveal | St::Solved) && self.game_mode == "endless" && self.points > 0 {
            self.bank_run();
        }
        self.b.game_over = false;
        self.state = St::Setup;
        self.setup_t = self.anim;
        self.b.score = 0;
        self.points = 0;
        self.b.play_sound("click");
    }

    fn bank_run(&mut self) {
        let key = self.length.to_string();
        // best = data["best"].get(key, 0); "not isinstance(best, int) or points > best"
        let best: Option<i64> = match section(&self.data, "best").and_then(|b| b.get(&key)) {
            None => Some(0),
            Some(v) => py_int(Some(v)),
        };
        if best.is_none_or(|b| self.points > b) {
            section_mut(&mut self.data, "best").insert(key, json!(self.points));
            save_data(&self.data);
        }
        if self.counts_score() && self.b.score > 0 {
            let (hs, _) = highscore::update(self.b.key, self.b.score);
            achievements::on_highscore(self.b.key, hs, false);
        }
    }

    fn handle_done(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Return" | "KP_Enter" => self.done_primary(),
                "c" | "C" => self.share(),
                "s" | "S" | "BackSpace" => self.to_setup(),
                "space" => {
                    self.panel_hidden = !self.panel_hidden;
                    self.b.play_sound("move");
                }
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if self.panel_hidden {
                self.panel_hidden = false;
                return;
            }
            if let Some(&(_, action)) = self.done_buttons.iter().find(|(r, _)| r.hit(ev.pos)) {
                match action {
                    Action::Share => self.share(),
                    Action::Primary => self.done_primary(),
                    Action::Setup => self.to_setup(),
                }
                return;
            }
            if self.done_panel.is_some_and(|p| !p.hit(ev.pos)) {
                self.panel_hidden = true;
            }
        }
    }

    fn done_primary(&mut self) {
        if self.game_mode == "daily" {
            if self.daily_date.as_deref() != Some(seedrand::today_str().as_str()) {
                self.b.game_over = false;
                self.b.new_round_result();
                self.start();
            } else {
                self.to_setup();
            }
            return;
        }
        self.b.game_over = false;
        self.b.new_round_result();
        self.start();
    }

    fn share(&mut self) {
        let boards: Vec<(Vec<Vec<Res>>, Option<usize>)> = self.boards.iter().map(|b| (b.rows.iter().map(|(_, r)| r.clone()).collect(), b.solved_at)).collect();
        let text = share_text(self.game_mode, &self.lang, self.length, &boards, self.hard, self.colorblind, self.daily_date.as_deref());
        if clip::copy_to_clipboard(&text) {
            self.say(t("wd.copied"), MESSAGE_TIME);
            self.b.play_sound("powerup");
        } else {
            self.say(t("wd.copy_failed"), MESSAGE_TIME);
            self.b.play_sound("hit");
        }
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let is = |a: &str| self.b.is_action(k, a, None);
                if k == "Return" || k == "KP_Enter" {
                    self.start();
                } else if ["4", "5", "6", "7"].contains(&k) {
                    self.set_length(k.parse().unwrap_or(DEFAULT_LENGTH));
                } else if k == "Left" || k == "Right" || is("left") || is("right") {
                    let step: i64 = if k == "Left" || is("left") { -1 } else { 1 };
                    let idx = LENGTHS.iter().position(|l| *l == self.length).unwrap_or(1) as i64;
                    let ni = (idx + step).clamp(0, LENGTHS.len() as i64 - 1) as usize;
                    self.set_length(LENGTHS[ni]);
                    self.setup_focus = 0;
                } else if k == "h" || k == "H" {
                    self.toggle(true);
                } else if k == "f" || k == "F" {
                    self.toggle(false);
                } else if k == "Up" || is("up") {
                    self.setup_focus = py::pmod32(self.setup_focus - 1, 4);
                    self.b.play_sound("move");
                } else if k == "Down" || is("down") {
                    self.setup_focus = py::pmod32(self.setup_focus + 1, 4);
                    self.b.play_sound("move");
                } else if k == "space" {
                    match self.setup_focus {
                        1 => self.toggle(true),
                        2 => self.toggle(false),
                        3 => self.start(),
                        _ => {}
                    }
                }
            }
            Kind::MouseDown => {
                if let Some(i) = self.su_seg.iter().position(|r| r.hit(ev.pos)) {
                    self.setup_focus = 0;
                    self.set_length(LENGTHS[i]);
                    return;
                }
                if self.su_hard.hit(ev.pos) {
                    self.setup_focus = 1;
                    self.toggle(true);
                } else if self.su_cb.hit(ev.pos) {
                    self.setup_focus = 2;
                    self.toggle(false);
                } else if self.su_start.hit(ev.pos) {
                    self.setup_focus = 3;
                    self.start();
                }
            }
            Kind::MouseMove => {
                for (i, rc) in [Rect::new(0, 0, 0, 0), self.su_hard, self.su_cb, self.su_start].iter().enumerate() {
                    if rc.hit(ev.pos) {
                        self.setup_focus = i as i32;
                    }
                }
                if self.su_seg.iter().any(|r| r.hit(ev.pos)) {
                    self.setup_focus = 0;
                }
            }
            _ => {}
        }
    }

    fn set_length(&mut self, length: usize) {
        if length == self.length || !LENGTHS.contains(&length) {
            return;
        }
        self.length = length;
        self.save_opt("length", json!(length));
        self.layout_play();
        self.b.play_sound("click");
    }

    /// `_toggle("hard")` (true) bzw. `_toggle("colorblind")` (false).
    fn toggle(&mut self, hard: bool) {
        if hard {
            self.hard = !self.hard;
            let v = self.hard;
            self.save_opt("hard", json!(v));
        } else {
            self.colorblind = !self.colorblind;
            let v = self.colorblind;
            self.save_opt("colorblind", json!(v));
        }
        self.board_cache.clear();
        self.b.play_sound("click");
    }

    // ===================================================== Update

    fn tick(&mut self, dt: f64) {
        self.anim += dt;
        if self.message_t > 0.0 {
            self.message_t = 0f64.max(self.message_t - dt);
            if self.message_t == 0.0 {
                self.message.clear();
            }
        }
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - dt);
        }
        if self.state == St::Reveal && self.reveal.is_some() {
            let step = self.reveal_step();
            let length = self.length;
            let anim = self.anim;
            let mut click = false;
            let mut fin = false;
            if let Some(rv) = self.reveal.as_mut() {
                let col = ((anim - rv.t0) / step) as i64;
                if col >= rv.ticks as i64 && rv.ticks < length {
                    rv.ticks = (col + 1) as usize;
                    click = true;
                }
                fin = anim >= rv.end;
            }
            if click {
                self.b.play_sound("click");
            }
            if fin {
                self.finish_reveal();
            }
        }
    }

    // ===================================================== Zeichnen

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.hud_h);
        draw::rect(s, p.panel, Rect::new(0, 0, w, h), 0);
        draw::line(s, p.border, (0, h), (w, h), 1);
        let cy = h / 2;
        let third = (w - self.hud_h) / 3 - 10;
        let mode = self.game_mode;
        let (left, mid) = if mode == "endless" {
            (tf("wd.score", &[("n", &self.points.to_string())]), tf("wd.solved_n", &[("n", &self.solved_count.to_string())]))
        } else if mode == "daily" {
            let st = get_stats(&self.data, "daily", &self.lang, self.length);
            (tf("wd.hud.daily", &[("d", &self.short_date())]), tf("wd.hud.streak", &[("n", &shown_streak(&st, "daily").to_string())]))
        } else {
            let a = self.boards.iter().filter(|b| b.solved()).count();
            (t(&format!("wd.mode.{mode}")), tf("wd.boards_solved", &[("a", &a.to_string()), ("b", &self.boards.len().to_string())]))
        };
        let px = self.hud.get_height();
        let acc = self.b.accent;
        self.text(s, &left, (px as f64 * 0.95) as i32, acc, third, true, Anc::MidLeft(12.0, cy as f64));
        self.text(s, &mid, (px as f64 * 0.8) as i32, p.text_dim, third, false, Anc::Center(((w - self.hud_h) / 2 + 6) as f64, cy as f64));
        let tries = (self.guesses.len() + if matches!(self.state, St::Play | St::Reveal) { 1 } else { 0 }).min(self.rows());
        let txt = tf("wd.tries", &[("a", &tries.to_string()), ("b", &self.rows().to_string())]);
        self.text(s, &txt, (px as f64 * 0.8) as i32, p.text_dim, third, false, Anc::MidRight((self.gear_rect.x - 8) as f64, cy as f64));
        Self::draw_gear(s, self.gear_rect);
    }

    fn short_date(&self) -> String {
        let d = self.daily_date.clone().unwrap_or_else(seedrand::today_str);
        let c: Vec<char> = d.chars().collect();
        let sl = |a: usize, b: usize| c.iter().skip(a).take(b.saturating_sub(a)).collect::<String>();
        format!("{}.{}.", sl(8, 10), sl(5, 7))
    }

    fn draw_gear(s: &mut Surface, rc: Rect) {
        let p = ui::pal();
        let col = p.text_dim;
        draw::rect_r(s, p.btn, rc, 0, 4.max(rc.h / 4));
        let (x0, x1) = (rc.x as f64 + rc.w as f64 * 0.24, rc.right() as f64 - rc.w as f64 * 0.24);
        let th = 1.max(rc.h / 14);
        for (i, f) in [0.3, 0.5, 0.7].iter().enumerate() {
            let y = (rc.y as f64 + rc.h as f64 * f) as i32;
            draw::line(s, col, (x0 as i32, y), (x1 as i32, y), th);
            let kx = (x0 + (x1 - x0) * [0.7, 0.3, 0.55][i]) as i32;
            draw::circle(s, col, (kx, y), 2.max(rc.h / 10), 0);
        }
    }

    fn state_color(&self, st: Res) -> Rgb {
        let (c, pr) = palette(self.colorblind);
        match st {
            Res::Absent => COL_ABSENT,
            Res::Correct => c,
            Res::Present => pr,
        }
    }

    fn empty_fill() -> Color {
        let p = ui::pal();
        ui::mix(p.bg_bottom, p.panel, 0.55)
    }

    fn letter(&mut self, size: i32, ch: char) -> Surface {
        if let Some(img) = self.letter_cache.get(&(size, ch)) {
            return img.clone();
        }
        let f = ui::font(7.max((size as f64 * 0.58) as i32), true, false);
        let img = f.render(&ch.to_string(), true, COL_LETTER.c());
        self.letter_cache.insert((size, ch), img.clone());
        img
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_tile(&mut self, s: &mut Surface, x: i32, y: i32, ch: Option<char>, state: Option<Res>, border: Option<Color>, scale_y: f64, scale: f64, dy: i32, alpha_fill: Option<Color>) {
        let size = self.tile;
        let fill: Color = match state {
            Some(st) => self.state_color(st).c(),
            None => alpha_fill.unwrap_or_else(Self::empty_fill),
        };
        let w = 1.max((size as f64 * scale) as i32);
        let h = 1.max((size as f64 * scale * scale_y) as i32);
        let rc = Rect::new(0, 0, w, h).center_at((x + size / 2, y + size / 2 + dy));
        let radius = 2.max(size / 9);
        draw::rect_r(s, fill, rc, 0, radius);
        if state.is_none() {
            draw::rect_r(s, border.unwrap_or(ui::pal().border), rc, 1.max(size / 24), radius);
        }
        if let Some(ch) = ch {
            if h > 3 {
                let mut img = self.letter(size, ch);
                if scale_y < 0.99 || (scale - 1.0).abs() > 0.01 {
                    img = transform::smoothscale(&img, 1.max((img.w() as f64 * scale) as i32), 1.max((img.h() as f64 * scale * scale_y) as i32));
                }
                let r = img.get_rect().center_at(rc.center());
                s.blr(&img, r);
            }
        }
    }

    fn tile_xy(&self, bi: usize, r: usize, c: usize) -> (i32, i32) {
        let (bx, by) = self.board_pos[bi];
        let step = self.tile + self.tile_gap;
        (bx + c as i32 * step, by + r as i32 * step)
    }

    fn bouncing(&self, bi: usize) -> bool {
        let board = &self.boards[bi];
        board.solved() && board.solve_t.is_some_and(|t| self.anim - t < BOUNCE_TIME + self.length as f64 * 0.08)
    }

    fn board_static(&mut self, bi: usize) -> Surface {
        let p = ui::pal();
        let bouncing = self.bouncing(bi);
        let board = &self.boards[bi];
        let key: BoardKey = (bi, self.tile, self.tile_gap, self.length, self.rows(), board.rows.len(), board.solved_at, bouncing, self.colorblind, ui::theme_name(), p.border, p.panel);
        if let Some((k, img)) = self.board_cache.get(&bi) {
            if *k == key {
                return img.clone();
            }
        }
        let mut img = pg::surface_alpha(self.board_w, self.board_h);
        let live = if board.solved() { None } else { Some(board.rows.len()) };
        let solved = board.solved();
        let solved_at = board.solved_at;
        let rows_data = board.rows.clone();
        let step = self.tile + self.tile_gap;
        let dim_fill = ui::mix(p.bg_bottom, p.panel, 0.3);
        for r in 0..self.rows() {
            if Some(r) == live {
                continue;
            }
            for c in 0..self.length {
                let (x, y) = (c as i32 * step, r as i32 * step);
                if r < rows_data.len() {
                    if bouncing && Some(r + 1) == solved_at {
                        continue;
                    }
                    let (word, res) = &rows_data[r];
                    let ch = word.chars().nth(c);
                    self.draw_tile(&mut img, x, y, ch, Some(res[c]), None, 1.0, 1.0, 0, None);
                } else if solved {
                    self.draw_tile(&mut img, x, y, None, None, Some(ui::mix(dim_fill, p.border, 0.4)), 1.0, 1.0, 0, Some(dim_fill));
                } else {
                    self.draw_tile(&mut img, x, y, None, None, None, 1.0, 1.0, 0, None);
                }
            }
        }
        self.board_cache.insert(bi, (key, img.clone()));
        img
    }

    fn draw_board(&mut self, s: &mut Surface, bi: usize) {
        let p = ui::pal();
        let (bx, by) = self.board_pos[bi];
        if self.boards.len() > 1 {
            let pad = self.board_inner;
            let frame = Rect::new(bx - pad, by - pad, self.board_w + 2 * pad, self.board_h + 2 * pad);
            let border: Option<Color> = if self.boards[bi].solved() {
                Some(self.pal_correct().c())
            } else if self.state == St::Done {
                Some(COL_LOSE.c())
            } else {
                None
            };
            ui::draw_panel_ex(s, frame, None, border, Some(6.max(pad * 2)), false, None);
            if let Some(bc) = border {
                draw::rect_r(s, bc, frame, 1.max(pad / 3), 6.max(pad * 2));
            }
        }
        let img = self.board_static(bi);
        s.bl(&img, (bx, by));
        if self.bouncing(bi) {
            let board = &self.boards[bi];
            let r = board.solved_at.unwrap_or(1) - 1;
            let (word, res) = board.rows[r].clone();
            let solve_t = board.solve_t.unwrap_or(0.0);
            for c in 0..self.length {
                let pp = (self.anim - solve_t - c as f64 * 0.08) / BOUNCE_TIME;
                let dy = if 0.0 < pp && pp < 1.0 { -(((PI * pp).sin() * self.tile as f64 * 0.35) as i32) } else { 0 };
                let (x, y) = self.tile_xy(bi, r, c);
                self.draw_tile(s, x, y, word.chars().nth(c), Some(res[c]), None, 1.0, 1.0, dy, None);
            }
        }
        if self.boards[bi].solved() {
            return;
        }
        let r = self.boards[bi].rows.len();
        if r >= self.rows() {
            if self.state == St::Done {
                self.draw_answer_pill(s, bi);
            }
            return;
        }
        let rv_data = self.reveal.as_ref().and_then(|rv| rv.results[bi].clone().map(|res| (rv.guess.clone(), res, rv.t0)));
        if let Some((guess, res, t0)) = rv_data {
            let step_t = self.reveal_step();
            let g: Vec<char> = guess.chars().collect();
            for c in 0..self.length {
                let (x, y) = self.tile_xy(bi, r, c);
                let pp = (self.anim - t0 - c as f64 * step_t) / FLIP_TIME;
                if pp <= 0.0 {
                    self.draw_tile(s, x, y, Some(g[c]), None, Some(p.text_dim), 1.0, 1.0, 0, None);
                } else if pp < 0.5 {
                    self.draw_tile(s, x, y, Some(g[c]), None, Some(p.text_dim), 0.02f64.max((pp * PI).cos()), 1.0, 0, None);
                } else {
                    let sy = if pp < 1.0 { 0.02f64.max(-(pp * PI).cos()) } else { 1.0 };
                    self.draw_tile(s, x, y, Some(g[c]), Some(res[c]), None, sy, 1.0, 0, None);
                }
            }
            return;
        }
        if self.state == St::Done {
            let ans: Vec<char> = self.boards[bi].answer.chars().collect();
            let fillc = ui::mix(Self::empty_fill(), COL_LOSE, 0.25);
            for c in 0..self.length {
                let (x, y) = self.tile_xy(bi, r, c);
                self.draw_tile(s, x, y, ans.get(c).copied(), None, Some(COL_LOSE.c()), 1.0, 1.0, 0, Some(fillc));
            }
            return;
        }
        let shake = if self.shake > 0.0 { (self.tile as f64 * 0.14 * (self.shake * 48.0).sin()) as i32 } else { 0 };
        let cur: Vec<char> = self.current.chars().collect();
        for c in 0..self.length {
            let (x, y) = self.tile_xy(bi, r, c);
            let ch = cur.get(c).copied();
            let mut scale = 1.0;
            if ch.is_some() {
                if let Some(t0) = self.pops.get(&c) {
                    let pp = (self.anim - t0) / POP_TIME;
                    if (0.0..1.0).contains(&pp) {
                        scale = 1.0 + 0.12 * (PI * pp).sin();
                    }
                }
            }
            let border = if ch.is_some() { Some(p.text_dim) } else { None };
            self.draw_tile(s, x + shake, y, ch, None, border, 1.0, scale, 0, None);
        }
    }

    fn draw_answer_pill(&mut self, s: &mut Surface, bi: usize) {
        let (bx, by) = self.board_pos[bi];
        let px = 10.max(((self.tile as f64 * 0.62) as i32).min(self.small_b.get_height()));
        let answer = self.boards[bi].answer.clone();
        let f = self.fit(&answer, self.board_w - 8, px, true);
        let img = f.render(&answer, true, COL_LETTER.c());
        let rc = img.get_rect().center_at((bx + self.board_w / 2, by + self.board_h));
        let bx_ = rc.inflate(12.max(px), 4.max(px / 3));
        draw::rect_r(s, COL_LOSE, bx_, 0, bx_.h / 2);
        s.blr(&img, rc);
    }

    fn draw_keyboard(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let radius = 3.max(self.key_h / 8);
        let n = self.boards.len();
        for (ch, rc) in self.key_rects.clone() {
            draw::rect_r(s, p.btn, rc, 0, radius);
            let states: Vec<Option<Res>> = self.boards.iter().map(|b| b.keys.get(&ch).copied()).collect();
            if n == 1 {
                if let Some(Some(st)) = states.first() {
                    draw::rect_r(s, self.state_color(*st), rc, 0, radius);
                }
            } else if n == 2 {
                let half = rc.w / 2;
                for (i, st) in states.iter().enumerate() {
                    let Some(st) = st else {
                        continue;
                    };
                    let part = Rect::new(rc.x + i as i32 * half, rc.y, if i == 0 { half } else { rc.w - half }, rc.h);
                    let a = if i == 0 { radius } else { 0 };
                    let b = if i == 1 { radius } else { 0 };
                    draw::rect_corners(s, self.state_color(*st), part, 0, 0, a, b, a, b);
                }
            } else {
                let (hw, hh) = (rc.w / 2, rc.h / 2);
                for (i, st) in states.iter().take(4).enumerate() {
                    let Some(st) = st else {
                        continue;
                    };
                    let (cx, cy) = ((i % 2) as i32, (i / 2) as i32);
                    let part = Rect::new(rc.x + cx * hw, rc.y + cy * hh, if cx == 0 { hw } else { rc.w - hw }, if cy == 0 { hh } else { rc.h - hh });
                    let r = |j: usize| if i == j { radius } else { 0 };
                    draw::rect_corners(s, self.state_color(*st), part, 0, 0, r(0), r(1), r(2), r(3));
                }
            }
            if n == 1 && states[0].is_none() {
                draw::rect_r(s, p.border, rc, 1, radius);
            }
            let img = self.key_font.render(&ch.to_string(), true, COL_LETTER.c());
            let mut sh = self.key_font.render(&ch.to_string(), true, rgb(0, 0, 0));
            sh.set_alpha(Some(90));
            let r = img.get_rect().center_at(rc.center());
            s.blr(&sh, r.move_(1, 1));
            s.blr(&img, r);
        }
        for (rc, enter) in [(self.enter_rect, true), (self.del_rect, false)] {
            let Some(rc) = rc else {
                continue;
            };
            draw::rect_r(s, p.btn_sel, rc, 0, radius);
            if enter {
                let px = 9.max(self.key_h * 3 / 8);
                self.text(s, &t("wd.enter"), px, p.text, rc.w - 6, true, ctr(rc.center()));
            } else {
                Self::draw_backspace(s, rc);
            }
        }
    }

    fn draw_backspace(s: &mut Surface, rc: Rect) {
        let p = ui::pal();
        let h = 8.max((rc.h as f64 * 0.36) as i32);
        let w = (h as f64 * 1.5) as i32;
        let x0 = rc.centerx() - w / 2;
        let y0 = rc.centery() - h / 2;
        let pts = [(x0, rc.centery()), (x0 + h / 2, y0), (x0 + w, y0), (x0 + w, y0 + h), (x0 + h / 2, y0 + h)];
        let th = 1.max(h / 8);
        draw::polygon(s, p.text, &pts, th);
        let cx = x0 + h / 2 + (w - h / 2) / 2;
        let d = 2.max(h / 5);
        draw::line(s, p.text, (cx - d, rc.centery() - d), (cx + d, rc.centery() + d), th);
        draw::line(s, p.text, (cx - d, rc.centery() + d), (cx + d, rc.centery() - d), th);
    }

    fn draw_message(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let alpha = 1f64.min(self.message_t / 0.35);
        let msg = self.message.clone();
        let f = self.fit(&msg, self.b.width - 40, self.small_b.get_height(), true);
        let mut img = f.render(&msg, true, p.text);
        let mut rc = img.get_rect().center_at((self.b.width / 2, self.msg_y));
        if self.state == St::Done && !self.panel_hidden {
            if let Some(panel) = self.done_panel {
                rc = rc.center_at((self.b.width / 2, rc.h.max(panel.y - rc.h)));
            }
        }
        let bx = rc.inflate(26, 12);
        let mut panel = pg::surface_alpha(bx.w, bx.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(p.panel_light.r, p.panel_light.g, p.panel_light.b, (240.0 * alpha) as i32), pr, 0, bx.h / 2);
        s.bl(&panel, (bx.x, bx.y));
        draw::rect_r(s, ui::mix(p.border_light, self.b.accent, 0.3), bx, 1, bx.h / 2);
        img.set_alpha(Some((255.0 * alpha) as i32 as u8));
        s.blr(&img, rc);
    }

    fn draw_solved(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let pp = (self.anim - self.solved_t - 0.35) / 0.25;
        if pp <= 0.0 {
            return;
        }
        let alpha = 1f64.min(pp);
        let k = self.k();
        let w = (self.b.width - 24).min((440.0 * k) as i32);
        let h = 56.max((88.0 * k) as i32);
        let row_y = self.tile_xy(0, self.boards[0].solved_at.unwrap_or(1) - 1, 0).1;
        let y = if row_y + self.tile / 2 < self.grid_top + self.board_h / 2 { self.grid_bottom - h } else { self.grid_top };
        let mut rc = Rect::new((self.b.width - w) / 2, y, w, h);
        rc.y -= ((1.0 - alpha) * 12.0) as i32;
        let col = self.pal_correct().c();
        ui::draw_panel_ex(s, rc, None, None, None, true, Some(col));
        let big = self.big.get_height();
        let txt = tf("wd.solved", &[("n", &self.last_points.to_string())]);
        self.text(s, &txt, big, col, w - 20, true, Anc::Center(rc.centerx() as f64, rc.y as f64 + h as f64 * 0.38));
        let txt = t("wd.next") + "   \u{00B7}   " + &t("wd.share_key");
        let tiny = self.tiny.get_height();
        self.text(s, &txt, tiny, p.text_dim, w - 20, false, Anc::Center(rc.centerx() as f64, rc.y as f64 + h as f64 * 0.76));
    }

    fn draw_hidden_hint(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let text = t("wd.result.show");
        let f = self.fit(&text, self.b.width - 30, self.small.get_height(), false);
        let img = f.render(&text, true, p.text);
        let rc = img.get_rect().midbottom_at((self.b.width / 2, self.kb_top - 4));
        let bx = rc.inflate(22, 10);
        ui::draw_panel_ex(s, bx, None, None, Some(bx.h / 2), false, None);
        s.blr(&img, rc);
    }

    fn draw_stats_block(&mut self, s: &mut Surface, rect: Rect, st: &Stats, highlight: Option<usize>, grow: f64) {
        let p = ui::pal();
        let k = self.k();
        let played = st.played;
        let rate = if played != 0 { py::round(100.0 * st.won as f64 / played as f64) } else { 0 };
        let values = [(played, "wd.stats.played"), (rate, "wd.stats.winrate"), (shown_streak(st, self.game_mode), "wd.stats.streak"), (st.best, "wd.stats.best")];
        let (num_px, lab_px, per_row) = self.stats_metrics(rect.w, rect.h);
        let col_w = rect.w / per_row;
        let mut y = rect.y;
        for (i, (v, key)) in values.iter().enumerate() {
            let i = i as i32;
            let cx = rect.x + col_w * (i % per_row) + col_w / 2;
            let cy = y + (i / per_row) * (num_px + lab_px + 6);
            self.text(s, &v.to_string(), num_px, p.text, col_w - 4, true, Anc::MidTop(cx as f64, cy as f64));
            self.text(s, &t(key), lab_px, p.text_dim, col_w - 4, false, Anc::MidTop(cx as f64, (cy + num_px + 2) as f64));
        }
        y += (4 / per_row) * (num_px + lab_px + 6) - 6 + 6.max((12.0 * k) as i32);
        let hp = 9.max((12.0 * k) as i32);
        self.text(s, &t("wd.stats.dist"), hp, p.text_dim, rect.w, true, Anc::MidTop(rect.centerx() as f64, y as f64));
        y += hp + 4.max((8.0 * k) as i32);
        let first = self.n_boards() - 1;
        let dist: Vec<i64> = st.dist.iter().skip(first).copied().collect();
        let bars = dist.len() as i32;
        let avail = rect.bottom() - y;
        let bar_gap = 2.max((4.0 * k) as i32);
        let bar_h = 6.max(((20.0 * k) as i32).min(py::fdiv32(avail - (bars - 1) * bar_gap, 1.max(bars))));
        let label_w = 12.max((18.0 * k) as i32);
        let top_v = *dist.iter().chain(std::iter::once(&1)).max().unwrap_or(&1);
        let correct = self.pal_correct().c();
        for (i, v) in dist.iter().enumerate() {
            let tries = first + i + 1;
            let by = y + i as i32 * (bar_h + bar_gap);
            if by + bar_h > rect.bottom() + 2 {
                break;
            }
            self.text(s, &tries.to_string(), 9.max(bar_h - 2), p.text_dim, label_w, true, Anc::MidLeft(rect.x as f64, (by + bar_h / 2) as f64));
            let full = rect.w - label_w - 6;
            let minw = 18.max((24.0 * k) as i32);
            let bw = (minw as f64 + (full - minw) as f64 * (*v as f64 / top_v as f64) * grow) as i32;
            let bar = Rect::new(rect.x + label_w + 6, by, bw, bar_h);
            let fill = if Some(tries) == highlight { correct } else { p.btn_sel };
            draw::rect_r(s, fill, bar, 0, 2.max(bar_h / 4));
            self.text(s, &v.to_string(), 9.max(bar_h - 3), COL_LETTER, bw - 6, true, Anc::MidRight((bar.right() - 5) as f64, bar.centery() as f64));
        }
    }

    fn stats_metrics(&mut self, width: i32, height: i32) -> (i32, i32, i32) {
        let k = self.k();
        let num_px = 14.max(((26.0 * k) as i32).min(py::fdiv32(height, 7)));
        let lab_px = 9.max((11.0 * k) as i32);
        let col_w = py::fdiv32(width, 4) - 4;
        let mut wide = true;
        for key in ["wd.stats.played", "wd.stats.winrate", "wd.stats.streak", "wd.stats.best"] {
            if !self.fits(&t(key), col_w, lab_px) {
                wide = false;
                break;
            }
        }
        (num_px, lab_px, if wide { 4 } else { 2 })
    }

    fn stats_height(&mut self, width: i32, bars: i32) -> i32 {
        let k = self.k();
        let (num_px, lab_px, per_row) = self.stats_metrics(width, 9999);
        let head = (4 / per_row) * (num_px + lab_px + 6) - 6 + 6.max((12.0 * k) as i32);
        let dist = 9.max((12.0 * k) as i32) + 4.max((8.0 * k) as i32);
        head + dist + bars * 10.max((20.0 * k) as i32) + (bars - 1) * 2.max((4.0 * k) as i32)
    }

    fn dim_surface(&mut self, alpha: f64) -> Surface {
        let key = (self.b.width, self.b.height);
        if self.dim.as_ref().map(|d| d.0) != Some(key) {
            let mut d = pg::surface_alpha(key.0, key.1);
            d.fill_c(rgba(6, 8, 14, 120));
            self.dim = Some((key, d));
        }
        let d = &mut self.dim.as_mut().expect("dim").1;
        d.set_alpha(Some((255.0 * 0f64.max(1f64.min(alpha))) as i32 as u8));
        d.clone()
    }

    fn draw_result(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let Some(res) = self.result.as_ref() else {
            return;
        };
        let (won, tries, stats, t0, new_best) = (res.won, res.tries, res.stats.clone(), res.t0, res.new_best);
        let pp = (self.anim - t0 - PANEL_DELAY) / 0.3;
        if pp <= 0.0 {
            self.done_buttons.clear();
            return;
        }
        let appear = 1f64.min(pp);
        let (w, h) = (self.b.width, self.b.height);
        let k = self.k();
        let dim = self.dim_surface(appear);
        s.bl(&dim, (0, 0));
        let pw = (w - 20).min((470.0 * k) as i32);
        let inner = pw - 2 * 12.max((22.0 * k) as i32);
        let mode = self.game_mode;
        let sub_px = 11.max((15.0 * k) as i32);
        let btn_h = 26.max((40.0 * k) as i32);
        let hint_px = 9.max((11.0 * k) as i32);
        let big_px = 18.max((30.0 * k) as i32);
        let bars = self.rows() as i32 - self.n_boards() as i32 + 1;
        let mut need = 8.max((16.0 * k) as i32) + big_px + 4.max((8.0 * k) as i32) + sub_px + 4.max((6.0 * k) as i32) + 2.max((6.0 * k) as i32)
            + self.stats_height(inner, bars)
            + 4.max((8.0 * k) as i32) + btn_h + 4.max((8.0 * k) as i32) + hint_px + 8.max((14.0 * k) as i32);
        if mode == "endless" {
            need += 10.max((13.0 * k) as i32) + 4.max((6.0 * k) as i32);
        }
        if mode == "daily" {
            need += sub_px + 4.max((8.0 * k) as i32);
        }
        let reserve = if self.show_banner() { BANNER_SPACE } else { 0 };
        let ph = (h - 16 - reserve).min(need);
        let panel = Rect::new(0, 0, pw, ph).center_at((w / 2, py::fdiv32(h - reserve, 2) + ((1.0 - appear) * 20.0) as i32));
        self.done_panel = Some(panel);
        let col: Color = if won { self.pal_correct().c() } else { COL_LOSE.c() };
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(col));
        let mut y = panel.y + 8.max((16.0 * k) as i32);
        let title = if mode == "endless" {
            t("wd.gameover")
        } else if won {
            t("wd.result.win")
        } else {
            t("wd.result.lose")
        };
        self.text(s, &title, big_px, col, inner, true, Anc::MidTop(panel.centerx() as f64, y as f64));
        y += big_px + 4.max((8.0 * k) as i32);
        let sub = if mode == "endless" {
            tf("wd.answer_was", &[("w", &self.boards[0].answer)]) + "  \u{00B7}  " + &tf("wd.result.points", &[("n", &self.points.to_string())])
        } else if self.boards.len() == 1 {
            if won {
                tf("wd.result.in", &[("n", &tries.to_string()), ("m", &self.rows().to_string())])
            } else {
                tf("wd.answer_was", &[("w", &self.boards[0].answer)])
            }
        } else {
            let ws: Vec<&str> = self.boards.iter().map(|b| b.answer.as_str()).collect();
            tf("wd.result.words", &[("w", &ws.join(" \u{00B7} "))])
        };
        self.text(s, &sub, sub_px, p.text, inner, false, Anc::MidTop(panel.centerx() as f64, y as f64));
        y += sub_px + 4.max((6.0 * k) as i32);
        if mode == "endless" {
            let best = section(&self.data, "best").and_then(|b| py_int(b.get(&self.length.to_string()))).unwrap_or(0);
            let line = if new_best { t("wd.new_best") } else { tf("wd.result.best", &[("n", &best.to_string())]) };
            let lp = 10.max((13.0 * k) as i32);
            self.text(s, &line, lp, p.gold, inner, false, Anc::MidTop(panel.centerx() as f64, y as f64));
            y += lp + 4.max((6.0 * k) as i32);
        }
        y += 2.max((6.0 * k) as i32);
        let mut bottom = panel.bottom() - 8.max((14.0 * k) as i32) - hint_px - 4.max((8.0 * k) as i32) - btn_h;
        if mode == "daily" {
            bottom -= sub_px + 4.max((8.0 * k) as i32);
        }
        let mut grow = 1f64.min(0f64.max((self.anim - t0 - PANEL_DELAY) / 0.7));
        grow = 1.0 - py::pow(1.0 - grow, 3.0);
        let highlight = if won { Some(tries) } else { None };
        let block = Rect::new(panel.centerx() - inner / 2, y, inner, bottom - y - 4.max((8.0 * k) as i32));
        self.draw_stats_block(s, block, &stats, highlight, grow);
        y = bottom;
        if mode == "daily" {
            let text = if self.daily_date.as_deref() != Some(seedrand::today_str().as_str()) {
                t("wd.daily.new")
            } else {
                tf("wd.daily.next", &[("t", &format_hms(seconds_to_midnight()))])
            };
            self.text(s, &text, sub_px, p.gold, inner, true, Anc::MidTop(panel.centerx() as f64, y as f64));
            y += sub_px + 4.max((8.0 * k) as i32);
        }
        let gap = 6.max((10.0 * k) as i32);
        let mut buttons: Vec<(String, Action)> = vec![(t("wd.share"), Action::Share)];
        if mode == "daily" {
            buttons.push((t("wd.setup"), Action::Setup));
        } else {
            buttons.push((t("wd.again"), Action::Primary));
            buttons.push((t("wd.setup"), Action::Setup));
        }
        let nb = buttons.len() as i32;
        let bw = (inner - (nb - 1) * gap) / nb;
        let mut longest = buttons[0].0.clone();
        for (l, _) in &buttons {
            if l.chars().count() > longest.chars().count() {
                longest = l.clone();
            }
        }
        let bfont = self.fit(&longest, bw - 12, 11.max((16.0 * k) as i32), true);
        self.done_buttons.clear();
        let acc = self.b.accent;
        for (i, (label, action)) in buttons.iter().enumerate() {
            let i = i as i32;
            let rc = Rect::new(panel.centerx() - inner / 2 + i * (bw + gap), y, bw, btn_h);
            let selected = i == nb - 2 || (nb == 2 && i == 0);
            ui::draw_button_ex(s, rc, label, &bfont, selected, None, None, Some(acc));
            self.done_buttons.push((rc, *action));
        }
        y += btn_h + 4.max((8.0 * k) as i32);
        let hint = if mode == "daily" { t("wd.result.hint_daily") } else { t("wd.result.hint") };
        let hint = self.fit_parts(&hint, inner, hint_px);
        self.text(s, &hint, hint_px, p.text_faint, inner, false, Anc::MidTop(panel.centerx() as f64, y as f64));
    }

    fn show_banner(&self) -> bool {
        self.game_mode == "endless" && self.length == SCORE_LENGTH
    }

    // ----- Setup -------------------------------------------------------

    fn draw_setup(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let k = self.k();
        ui::draw_background(s, w, h);
        self.draw_title_tiles(s);
        let sub = t(&format!("wd.mode.{}", self.game_mode)) + "  \u{00B7}  " + &t(&format!("wd.setup.sub.{}", self.game_mode));
        let sp = self.small.get_height();
        self.text(s, &sub, sp, p.text_dim, w - 30, false, Anc::MidTop((w / 2) as f64, self.su_sub_y as f64));
        let (correct, _) = palette(self.colorblind);
        let lab_px = self.small.get_height();
        let col_w = self.su_start.w;
        let seg0 = self.su_seg[0];
        self.text(s, &t("wd.setup.length"), lab_px, p.text, col_w, true, Anc::MidLeft((seg0.x + 2) as f64, (self.su_label_y + lab_px / 2) as f64));
        for (i, rc) in self.su_seg.clone().iter().enumerate() {
            let on = LENGTHS[i] == self.length;
            let fill: Color = if on { correct.c() } else { p.btn };
            let rad = 4.max(rc.h / 5);
            draw::rect_r(s, fill, *rc, 0, rad);
            if !on {
                draw::rect_r(s, p.border, *rc, 1, rad);
            }
            if self.setup_focus == 0 && on {
                draw::rect_r(s, p.text, rc.inflate(4, 4), 2, rad + 2);
            }
            let num_px = 14.max((rc.h as f64 * 0.5) as i32);
            let c: Color = if on { COL_LETTER.c() } else { p.text };
            self.text(s, &LENGTHS[i].to_string(), num_px, c, rc.w - 4, true, ctr(rc.center()));
        }
        let (hard, cb, f) = (self.hard, self.colorblind, self.setup_focus);
        self.draw_toggle(s, self.su_hard, true, &t("wd.setup.hard"), &t("wd.setup.hard_desc"), hard, f == 1, false);
        self.draw_toggle(s, self.su_cb, false, &t("wd.setup.colorblind"), &t("wd.setup.colorblind_desc"), cb, f == 2, true);
        let start = t("common.start");
        let sf = self.fit(&start, self.su_start.w - 20, 13.max((20.0 * k) as i32), true);
        ui::draw_button_ex(s, self.su_start, &start, &sf, self.setup_focus == 3, None, None, Some(self.b.accent));
        self.draw_setup_stats(s);
        let hp = 10.max((13.0 * k) as i32);
        let hint = self.fit_parts(&t("wd.setup.hint"), w - 20, hp);
        let hf = self.fit(&hint, w - 20, hp, false);
        ui::draw_footer_ex(s, w, h, &hint, Some(&hf));
    }

    fn draw_title_tiles(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let word: Vec<char> = "WORDLE".chars().collect();
        let size = self.su_tile;
        let gap = 3.max(size / 9);
        let n = word.len() as i32;
        let total = n * size + (n - 1) * gap;
        let x0 = py::fdiv32(self.b.width - total, 2);
        let (correct, present) = palette(self.colorblind);
        let states = [Res::Correct, Res::Absent, Res::Present, Res::Correct, Res::Absent, Res::Correct];
        let f = ui::font(10.max((size as f64 * 0.58) as i32), true, false);
        for (i, ch) in word.iter().enumerate() {
            let pp = (self.anim - self.setup_t - 0.08 * i as f64) / FLIP_TIME;
            let mut sy = 1.0;
            let mut state = Some(states[i]);
            if pp < 0.5 {
                state = None;
                sy = if pp > 0.0 { 0.05f64.max((0f64.max(pp) * PI).cos()) } else { 1.0 };
            } else if pp < 1.0 {
                sy = 0.05f64.max(-(pp * PI).cos());
            }
            let bob = (self.anim * 2.2 + i as f64 * 0.7).sin() * size as f64 * 0.04;
            let rc = Rect::new(0, 0, size, 1.max((size as f64 * sy) as i32)).center_at((x0 + i as i32 * (size + gap) + size / 2, self.su_title_y + size / 2 + bob as i32));
            let fill: Color = match state {
                Some(Res::Absent) => COL_ABSENT.c(),
                Some(Res::Correct) => correct.c(),
                Some(Res::Present) => present.c(),
                None => Self::empty_fill(),
            };
            draw::rect_r(s, fill, rc, 0, 3.max(size / 9));
            if state.is_none() {
                draw::rect_r(s, p.border_light, rc, 2, 3.max(size / 9));
            }
            let mut img = f.render(&ch.to_string(), true, COL_LETTER.c());
            if sy < 0.99 {
                img = transform::smoothscale(&img, img.w(), 1.max((img.h() as f64 * sy) as i32));
            }
            let r = img.get_rect().center_at(rc.center());
            s.blr(&img, r);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_toggle(&mut self, s: &mut Surface, rc: Rect, hard_key: bool, label: &str, desc: &str, value: bool, focus: bool, preview: bool) {
        let p = ui::pal();
        let k = self.k();
        let radius = 6.max(rc.h / 5);
        draw::rect_r(s, if focus { p.btn_sel } else { p.btn }, rc, 0, radius);
        draw::rect_r(s, if focus { self.b.accent } else { p.border }, rc, 1, radius);
        let sw_h = 16.max((rc.h as f64 * 0.42) as i32);
        let sw_w = (sw_h as f64 * 1.9) as i32;
        let sw = Rect::new(rc.right() - sw_w - 8.max((12.0 * k) as i32), rc.centery() - sw_h / 2, sw_w, sw_h);
        let target = if value { 1.0 } else { 0.0 };
        let slot = if hard_key { &mut self.toggle_hard } else { &mut self.toggle_cb };
        let mut v = slot.unwrap_or(target);
        v += (target - v) * 0.3;
        *slot = Some(v);
        let (on_col, present) = palette(self.colorblind);
        draw::rect_r(s, ui::mix(p.border_light, on_col, v), sw, 0, sw_h / 2);
        let knob_r = sw_h / 2 - 2;
        let kx = (sw.x as f64 + (sw_h / 2) as f64 + (sw_w - sw_h) as f64 * v) as i32;
        draw::circle(s, COL_LETTER, (kx, sw.centery()), knob_r, 0);
        let text_x = rc.x + 8.max((14.0 * k) as i32);
        let mut right = sw.x - 6.max((10.0 * k) as i32);
        if preview {
            let ps = 8.max((sw_h as f64 * 0.8) as i32);
            for (i, col) in [on_col, present].iter().enumerate() {
                let pr = Rect::new(right - (2 - i as i32) * (ps + 3), rc.centery() - ps / 2, ps, ps);
                draw::rect_r(s, *col, pr, 0, 2.max(ps / 5));
            }
            right -= 2 * (ps + 3) + 4;
        }
        let max_w = right - text_x;
        let lab_px = self.small_b.get_height();
        let desc_px = 9.max((11.0 * k) as i32);
        if self.fits(desc, max_w, desc_px) && rc.h >= lab_px + desc_px + 8 {
            self.text(s, label, lab_px, p.text, max_w, true, Anc::MidLeft(text_x as f64, rc.y as f64 + rc.h as f64 * 0.36));
            self.text(s, desc, desc_px, p.text_dim, max_w, false, Anc::MidLeft(text_x as f64, rc.y as f64 + rc.h as f64 * 0.72));
        } else {
            self.text(s, label, lab_px, p.text, max_w, true, Anc::MidLeft(text_x as f64, rc.centery() as f64));
        }
    }

    fn draw_setup_stats(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let k = self.k();
        let rect = self.su_stats_rect;
        ui::draw_panel_ex(s, rect, None, None, None, true, Some(self.b.accent));
        let pad = 8.max((16.0 * k) as i32);
        let inner = rect.inflate(-2 * pad, -2 * pad);
        let head = t("wd.stats.title") + "  \u{00B7}  " + &tf("wd.setup.letters", &[("n", &self.length.to_string())]);
        let head_px = self.small_b.get_height();
        self.text(s, &head, head_px, p.text, inner.w, true, Anc::MidTop(inner.centerx() as f64, inner.y as f64));
        let st = get_stats(&self.data, self.game_mode, &self.lang, self.length);
        let info_px = 10.max((13.0 * k) as i32);
        let info = self.setup_info();
        let g12 = 6.max((12.0 * k) as i32);
        let block = Rect::new(inner.x, inner.y + head_px + g12, inner.w, inner.h - head_px - g12 - info_px - 6.max((10.0 * k) as i32));
        self.draw_stats_block(s, block, &st, None, 1.0);
        self.text(s, &info, info_px, p.gold, inner.w, false, Anc::MidBottom(inner.centerx() as f64, inner.bottom() as f64));
    }

    fn setup_info(&self) -> String {
        let mode = self.game_mode;
        if mode == "endless" {
            let best = section(&self.data, "best").and_then(|b| py_int(b.get(&self.length.to_string()))).unwrap_or(0);
            return tf("wd.result.best", &[("n", &best.to_string())]);
        }
        if mode == "daily" {
            let prog = section(&self.data, "daily").and_then(|d| d.get(&self.daily_slot())).and_then(|v| v.as_object());
            let today = seedrand::today_str();
            if let Some(prog) = prog {
                if prog.get("date") == Some(&Value::String(today)) {
                    if prog.get("done").is_some_and(truthy) {
                        return tf("wd.daily.next", &[("t", &format_hms(seconds_to_midnight()))]);
                    }
                    let n = match prog.get("guesses") {
                        Some(Value::Array(a)) => a.len(),
                        Some(Value::String(st)) => st.chars().count(),
                        Some(Value::Object(o)) => o.len(),
                        _ => 0,
                    };
                    return tf("wd.daily.progress", &[("n", &n.to_string())]);
                }
            }
            return t("wd.daily.ready");
        }
        t(&format!("wd.setup.sub.{mode}"))
    }
}

/// Pythons Wahrheitswert eines JSON-Werts.
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

impl Game for WordleGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let mode = match self.b.mode.as_str() {
            "normal" | "hard" | "single" => "endless",
            m => m,
        };
        self.game_mode = MODES.iter().map(|(k, _)| *k).find(|k| *k == mode).unwrap_or("endless");
        self.lang = i18n::get_language();
        if self.b.mode == "hard" && !self.opts().1 {
            self.save_opt("hard", json!(true));
        }
        let (length, hard, cb) = self.opts();
        self.length = length;
        self.hard = hard;
        self.colorblind = cb;
        self.data = load_data();
        self.anim = 0.0;
        self.updated = false;
        self.last_draw = None;
        self.state = St::Setup;
        self.setup_focus = 3;
        self.setup_t = 0.0;
        self.toggle_hard = None;
        self.toggle_cb = None;
        self.boards.clear();
        self.guesses.clear();
        self.current.clear();
        self.reveal = None;
        self.points = 0;
        self.last_points = 0;
        self.solved_count = 0;
        self.solved_t = 0.0;
        self.message.clear();
        self.message_t = 0.0;
        self.shake = 0.0;
        self.pops.clear();
        self.result = None;
        self.panel_hidden = false;
        self.daily_date = None;
        self.daily_view = false;
        self.fit_cache.clear();
        self.letter_cache.clear();
        self.board_cache.clear();
        self.make_fonts();
        self.layout();
    }

    fn on_surface_changed(&mut self) {
        self.fit_cache.clear();
        self.letter_cache.clear();
        self.board_cache.clear();
        self.make_fonts();
        self.layout();
    }

    fn show_highscore_banner(&self) -> bool {
        self.show_banner()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == St::Setup {
            self.handle_setup(ev);
            return;
        }
        if ev.kind == Kind::MouseDown && matches!(self.state, St::Play | St::Reveal | St::Solved) && self.gear_rect.hit(ev.pos) {
            self.to_setup();
            return;
        }
        if self.state == St::Done {
            self.handle_done(ev);
            return;
        }
        if self.state == St::Solved {
            if Self::is_continue(ev) {
                self.new_round();
                self.b.play_sound("click");
            } else if ev.kind == Kind::KeyDown && ev.key_in(&["c", "C"]) {
                self.share();
            }
            return;
        }
        if self.state == St::Reveal {
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if k == "BackSpace" {
                self.backspace();
            } else if k == "Return" || k == "KP_Enter" {
                self.submit();
            } else {
                let ch = if !ev.char.is_empty() && ev.char.chars().count() == 1 { ev.char.as_str() } else { k };
                let mut it = ch.chars();
                if let (Some(c), None) = (it.next(), it.next()) {
                    if c.is_ascii_alphabetic() {
                        self.type_ch(c.to_ascii_uppercase());
                    }
                }
            }
        } else if ev.kind == Kind::MouseDown {
            self.click_keyboard(ev.pos);
        }
    }

    fn update(&mut self, dt: f64) {
        self.updated = true;
        self.tick(dt);
    }

    fn draw(&mut self) {
        let now = py::perf_counter();
        if !self.updated && self.b.game_over {
            if let Some(last) = self.last_draw {
                self.tick(0.1f64.min(now - last));
            }
        }
        self.updated = false;
        self.last_draw = Some(now);
        let mut s = self.b.sf();
        if self.state == St::Setup {
            self.draw_setup(&mut s);
            return;
        }
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        self.draw_hud(&mut s);
        for i in 0..self.boards.len() {
            self.draw_board(&mut s, i);
        }
        self.draw_keyboard(&mut s);
        if self.state == St::Solved {
            self.draw_solved(&mut s);
        }
        if self.state == St::Done {
            if self.panel_hidden {
                self.draw_hidden_hint(&mut s);
            } else {
                self.draw_result(&mut s);
            }
        }
        if !self.message.is_empty() {
            self.draw_message(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "wordle",
    class_name: "WordleGame",
    supports_multiplayer: false,
    modes: MODES,
    make: WordleGame::new,
};
