//! geodash.rs
//! ==========
//! Geometry Dash - der Rhythmus-Plattformer: Tippen, Halten, Fliegen,
//! Sterben, sofort weiter. Uebertragen aus `games/geodash.py`.
//!
//! Aufbau wie in Python:
//!   geodash_core.rs   Physik in Festkomma-Ganzzahlen (bitgleich)
//!   geodash_draw.rs   Zeichnen von Welt, Objekten und Spieler
//!   geodash_music.rs  der Soundtrack je Level
//!   geodash_edit.rs   LEVELS-Reiter und Level-Editor
//!   geodash_extra.rs  WEITERE-Reiter (Level-Bibliothek)
//!   geodash.rs        Level-Auswahl, Spielablauf, Fortschritt
//!
//! Die Physik laeuft in festen 240-Hz-Schritten; jede Eingabe bekommt beim
//! Eintreffen einen Zeitstempel (`perf_counter`) und wird dem Schritt
//! zugeordnet, in dem sie passiert ist.

use std::collections::{BTreeSet, HashMap};
use std::f64::consts::TAU;

use pgz_core::store;
use pgz_engine::pg::surface_alpha;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::geodash_core::{self as core, Level, State};
use crate::geodash_draw::{self as gdraw, Renderer};
use crate::geodash_edit::{self as edit, best_get, count_get, count_set, Best, Counts, GdAction, LevelEditor, LevelList};
use crate::geodash_extra::ExtraList;
use crate::geodash_music as music;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Select,
    Play,
    Complete,
    Edit,
}

const TABS: [&str; 3] = ["play", "extra", "levels"];
/// Schwierigkeits-Schluessel.
pub const DIFF_KEYS: [&str; 6] = ["easy", "normal", "hard", "harder", "insane", "demon"];
const DOTS_MAX: usize = 10;
const DEATH_PAUSE: f64 = 0.85;
const AUTO_CHECK_STEPS: i64 = 2 * core::HZ;
const VIEW_ROWS: f64 = 12.5;
const SOURCES_MOUSE: &str = "mouse";

static LEVELS_JSON: &str = include_str!("../../../../games/levels/geodash.json");
static EXTRA_JSON: &str = include_str!("../../../../games/levels/geodash_extra.json");
static TEMPLATES_JSON: &str = include_str!("../../../../games/levels/geodash_templates.json");

fn load_level_text(text: &str) -> Vec<Map<String, Value>> {
    let Ok(data) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    data.get("levels")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|d| d.as_object()).filter(|d| truthy(d.get("id"))).map(core::normalize_level).collect())
        .unwrap_or_default()
}

/// Die eingebauten Level (normalisiert).
pub fn load_builtin_levels() -> Vec<Map<String, Value>> {
    let levels = load_level_text(LEVELS_JSON);
    if !levels.is_empty() {
        return levels;
    }
    let objs: Vec<Value> = (0..8).map(|i| json!(["spike", 20 + 10 * i, 0, 0])).collect();
    let mut d = Map::new();
    d.insert("id".into(), json!("lama-launch"));
    d.insert("name".into(), json!("Lama Launch"));
    d.insert("difficulty".into(), json!(0));
    d.insert("stars".into(), json!(1));
    d.insert("objects".into(), Value::Array(objs));
    d.insert("length".into(), json!(110));
    vec![core::normalize_level(&d)]
}

/// Die Level-Bibliothek des Reiters "Weitere".
pub fn load_extra_levels() -> Vec<Map<String, Value>> {
    load_level_text(EXTRA_JSON)
}

/// Die Vorlagen fuer eigene Level.
pub fn load_templates() -> Vec<Map<String, Value>> {
    load_level_text(TEMPLATES_JSON)
}

fn total_coins_mask(mask: i64) -> i64 {
    (mask & 7).count_ones() as i64
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// Pythons `int(v)` (ohne Fehler: None).
fn py_int(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().filter(|f| f.is_finite()).map(|f| f.trunc() as i64)),
        Value::Bool(b) => Some(*b as i64),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn i_of(m: &Map<String, Value>, key: &str) -> i64 {
    m.get(key).and_then(py_int).unwrap_or(0)
}

fn s_of<'a>(m: &'a Map<String, Value>, key: &str) -> &'a str {
    m.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    col: (i32, i32, i32),
    size: f64,
}

#[derive(Clone, Default)]
struct Info {
    attempts: i64,
    jumps: i64,
    time: f64,
    coins: i64,
    coin_count: i64,
    stars: i64,
    new_coins: i64,
    practice: bool,
    verified: bool,
}

/// Geometry Dash.
pub struct GeometryDashGame {
    b: GameBase,
    levels: Vec<Map<String, Value>>,
    sel: usize,
    practice: bool,
    music_on: bool,
    show_bar: bool,
    auto_cp: bool,
    setup_tab: &'static str,
    lists: Option<LevelList>,
    extra: Option<ExtraList>,
    editor: Option<LevelEditor>,
    test: Option<bool>,
    renderer: Renderer,
    t: f64,
    card_anim: f64,
    card_dir: i64,
    best: Best,
    coins: Counts,
    attempts: Counts,
    jumps: i64,
    dirty: bool,
    huge: Font,
    mid: Font,
    small: Font,
    tiny: Font,
    attempt_font: Font,
    btn_font: Font,
    tab_rects: Vec<Rect>,
    tab_bottom: i32,
    card_rect: Rect,
    arrow_rects: [Rect; 2],
    dots_y: i32,
    mode_rects: [Rect; 2],
    start_rect: Rect,
    opt_rects: Vec<(&'static str, Rect)>,
    state: St,
    cur: Option<Map<String, Value>>,
    cur_kind: &'static str,
    lv: Option<Level>,
    run: Option<State>,
    voices: Option<Vec<audio::Voice>>,
    voices_key: Option<(String, i64, String)>,
    particles: Vec<Particle>,
    toast: String,
    toast_t: f64,
    start_block: Option<i64>,
    checkpoints: Vec<State>,
    session_attempts: i64,
    session_jumps: i64,
    session_time: f64,
    /// `_gd_logged` - Level-Besuch schon in der Match-Historie (vor dem
    /// ersten Level wie `getattr(self, "_gd_logged", True)`).
    gd_logged: bool,
    /// `_gd_best_pct` - bester Fortschritt dieses Besuchs.
    gd_best_pct: i64,
    bpm: i64,
    acc: f64,
    queue: Vec<(f64, bool, String)>,
    sources: BTreeSet<String>,
    dead_t: f64,
    last_cp_step: i64,
    angle: f64,
    trail: Vec<(f64, f64)>,
    cam_y: f64,
    run_time: f64,
    attempt_label_x: f64,
    attempt_label_y: f64,
    new_best: Option<(i64, f64)>,
    shock: Option<(f64, f64, f64)>,
    music_t: f64,
    complete_t: f64,
    complete_info: Info,
    thumbs: HashMap<(String, i32, i32), Surface>,
}

impl GeometryDashGame {
    /// `GeometryDashGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = GeometryDashGame {
            b,
            levels: Vec::new(),
            sel: 0,
            practice: false,
            music_on: true,
            show_bar: true,
            auto_cp: true,
            setup_tab: "play",
            lists: None,
            extra: None,
            editor: None,
            test: None,
            renderer: Renderer::new(),
            t: 0.0,
            card_anim: 0.0,
            card_dir: 1,
            best: Vec::new(),
            coins: Vec::new(),
            attempts: Vec::new(),
            jumps: 0,
            dirty: false,
            huge: f.clone(),
            mid: f.clone(),
            small: f.clone(),
            tiny: f.clone(),
            attempt_font: f.clone(),
            btn_font: f,
            tab_rects: Vec::new(),
            tab_bottom: 0,
            card_rect: z,
            arrow_rects: [z, z],
            dots_y: 0,
            mode_rects: [z, z],
            start_rect: z,
            opt_rects: Vec::new(),
            state: St::Select,
            cur: None,
            cur_kind: "main",
            lv: None,
            run: None,
            voices: None,
            voices_key: None,
            particles: Vec::new(),
            toast: String::new(),
            toast_t: 0.0,
            start_block: None,
            checkpoints: Vec::new(),
            session_attempts: 0,
            session_jumps: 0,
            session_time: 0.0,
            gd_logged: true,
            gd_best_pct: 0,
            bpm: 0,
            acc: 0.0,
            queue: Vec::new(),
            sources: BTreeSet::new(),
            dead_t: -1.0,
            last_cp_step: 0,
            angle: 0.0,
            trail: Vec::new(),
            cam_y: 0.0,
            run_time: 0.0,
            attempt_label_x: 0.0,
            attempt_label_y: 0.0,
            new_best: None,
            shock: None,
            music_t: 0.0,
            complete_t: 0.0,
            complete_info: Info::default(),
            thumbs: HashMap::new(),
        };
        g.reset();
        Box::new(g)
    }

    fn gs_truthy(&self, key: &str, def: bool) -> bool {
        match self.b.opt_value("geodash", key) {
            None => def,
            Some(v) => truthy(Some(&v)),
        }
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("geodash", key, value);
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.huge = ui::font(24.max(h / 13), true, false);
        self.mid = ui::font(16.max(h / 24), true, false);
        self.small = ui::font(14.max(h / 32), false, false);
        self.tiny = ui::font(11.max(h / 42), false, false);
        self.attempt_font = ui::font(18.max(h / 17), true, false);
        self.btn_font = ui::font(15.max(h / 30), true, false);
    }

    // ===================================================== Fortschritt

    fn load_progress(&mut self) {
        let data = store::load_section("geodash");
        self.best.clear();
        if let Some(Value::Object(m)) = data.get("best") {
            for (k, v) in m {
                if let Some(a) = v.as_array().filter(|a| a.len() >= 2) {
                    if let (Some(x), Some(y)) = (py_int(&a[0]), py_int(&a[1])) {
                        self.best.push((k.clone(), [x.clamp(0, 100), y.clamp(0, 100)]));
                    }
                }
            }
        }
        self.coins.clear();
        if let Some(Value::Object(m)) = data.get("coins") {
            for (k, v) in m {
                if let Some(x) = py_int(v) {
                    self.coins.push((k.clone(), x & 7));
                }
            }
        }
        self.attempts.clear();
        if let Some(Value::Object(m)) = data.get("attempts") {
            for (k, v) in m {
                if let Some(x) = py_int(v) {
                    self.attempts.push((k.clone(), x.max(0)));
                }
            }
        }
        self.jumps = data.get("jumps").map(|v| py_int(v).unwrap_or(0)).unwrap_or(0).max(0);
        self.dirty = false;
        self.b.score = self.total_stars();
    }

    fn save_progress(&mut self) {
        if !self.dirty {
            return;
        }
        let mut best = Map::new();
        for (k, v) in &self.best {
            best.insert(k.clone(), json!(v));
        }
        let mut coins = Map::new();
        for (k, v) in &self.coins {
            coins.insert(k.clone(), json!(v));
        }
        let mut att = Map::new();
        for (k, v) in &self.attempts {
            att.insert(k.clone(), json!(v));
        }
        let mut m = Map::new();
        m.insert("best".into(), Value::Object(best));
        m.insert("coins".into(), Value::Object(coins));
        m.insert("attempts".into(), Value::Object(att));
        m.insert("jumps".into(), json!(self.jumps));
        store::save_section("geodash", m);
        self.dirty = false;
    }

    fn total_stars(&self) -> i64 {
        let mut n = 0;
        for d in &self.levels {
            let lid = s_of(d, "id");
            if best_get(&self.best, lid)[0] >= 100 {
                n += i_of(d, "stars");
            }
            n += total_coins_mask(count_get(&self.coins, lid));
        }
        n
    }

    fn max_stars(&self) -> i64 {
        self.levels.iter().map(|d| i_of(d, "stars") + 3).sum()
    }

    fn best_key(&self) -> String {
        let Some(cur) = &self.cur else {
            return String::new();
        };
        match self.cur_kind {
            "main" => s_of(cur, "id").to_string(),
            "extra" => format!("x:{}", s_of(cur, "id")),
            _ => format!("ugc:{}", s_of(cur, "id")),
        }
    }

    fn best_mut(&mut self, key: &str) -> &mut [i64; 2] {
        if let Some(i) = self.best.iter().position(|e| e.0 == key) {
            return &mut self.best[i].1;
        }
        self.best.push((key.to_string(), [0, 0]));
        let n = self.best.len();
        &mut self.best[n - 1].1
    }

    // ===================================================== Level starten

    fn start_level(&mut self, level: &Map<String, Value>, kind: &'static str, start_block: Option<i64>, practice: Option<bool>) {
        let cur = core::normalize_level(level);
        let lv = Level::new(&cur);
        gdraw::block_masks(&lv);
        self.bpm = music::style_bpm(&cur);
        self.cur = Some(cur);
        self.cur_kind = kind;
        self.lv = Some(lv);
        self.start_block = start_block;
        if let Some(p) = practice {
            self.practice = p;
        }
        self.checkpoints.clear();
        self.session_attempts = 0;
        self.session_jumps = 0;
        self.session_time = 0.0;
        self.gd_logged = false;
        self.gd_best_pct = 0;
        self.state = St::Play;
        self.new_attempt(false);
        self.b.play_sound("select");
    }

    fn new_attempt(&mut self, from_checkpoint: bool) {
        let lv = self.lv.as_ref().expect("lv");
        self.run = Some(if from_checkpoint && !self.checkpoints.is_empty() {
            self.checkpoints[self.checkpoints.len() - 1].clone()
        } else {
            core::new_state(lv, self.start_block)
        });
        self.session_attempts += 1;
        if self.cur_kind != "test" {
            let key = self.best_key();
            let n = count_get(&self.attempts, &key);
            count_set(&mut self.attempts, &key, n + 1);
            self.dirty = true;
        }
        self.acc = 0.0;
        self.queue.clear();
        self.sources.clear();
        self.dead_t = -1.0;
        self.last_cp_step = self.run.as_ref().expect("run").step;
        self.angle = 0.0;
        self.trail.clear();
        let (px, pyy) = self.player_blocks();
        self.cam_y = self.cam_target(pyy);
        self.run_time = 0.0;
        self.attempt_label_x = px + 2.5;
        self.attempt_label_y = pyy + 4.0;
        self.new_best = None;
        self.shock = None;
        if !(from_checkpoint && self.practice && audio::music_playing(Some(self.ident()))) {
            self.start_music();
        }
    }

    fn start_music(&mut self) {
        audio::music_stop(Some(self.ident()));
        self.music_t = 0.0;
        let Some(cur) = &self.cur else {
            return;
        };
        if !self.music_on || s_of(cur, "music") == "none" {
            return;
        }
        let key = (s_of(cur, "music").to_string(), self.bpm, s_of(cur, "id").to_string());
        if self.voices_key.as_ref() != Some(&key) {
            self.voices = music::build_voices(&key.0, key.1, &key.2);
            self.voices_key = Some(key);
        }
        if let Some(v) = self.voices.as_ref().filter(|v| !v.is_empty()) {
            audio::music_play(v, self.ident(), Some(&self.b.settings), 1.0);
        }
    }

    fn player_blocks(&self) -> (f64, f64) {
        let s = self.run.as_ref().expect("run");
        (s.x as f64 / core::B as f64, s.y as f64 / core::B as f64)
    }

    // ===================================================== Eingabe

    fn is_jump_key(&self, k: &str) -> bool {
        ["space", "Up", "w", "W", "Return", "KP_Enter"].contains(&k) || self.b.is_action(k, "up", None) || self.b.is_action(k, "action", None)
    }

    fn handle_play(&mut self, ev: &InputEvent) {
        let now = py::perf_counter();
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if ev.repeat {
                    return;
                }
                if k == "Escape" && self.test.is_some() {
                    self.back_to_editor();
                    return;
                }
                if self.is_jump_key(k) {
                    self.queue.push((now, true, format!("k:{}", k.to_lowercase())));
                    return;
                }
                let low = k.to_lowercase();
                if low == "z" && self.b.key_is_free(k) && self.practice {
                    self.place_checkpoint();
                } else if low == "x" && self.b.key_is_free(k) && self.practice {
                    self.remove_checkpoint();
                } else if low == "p" && self.b.key_is_free(k) {
                    self.toggle_practice_live();
                } else if low == "r" && self.b.key_is_free(k) {
                    self.restart_now();
                } else if (k == "BackSpace" || k == "q" || k == "Q") && self.b.key_is_free(k) {
                    if self.test.is_some() {
                        self.back_to_editor();
                    } else {
                        self.leave_to_select();
                    }
                }
            }
            Kind::KeyUp => {
                let k = ev.key.as_str();
                if self.is_jump_key(k) {
                    self.queue.push((now, false, format!("k:{}", k.to_lowercase())));
                }
            }
            Kind::MouseDown if ev.button == 1 => self.queue.push((now, true, SOURCES_MOUSE.into())),
            Kind::MouseUp if ev.button == 1 => self.queue.push((now, false, SOURCES_MOUSE.into())),
            _ => {}
        }
    }

    fn toggle_practice_live(&mut self) {
        if self.cur_kind == "test" && self.test == Some(true) {
            self.test = Some(false);
        }
        self.practice = !self.practice;
        self.checkpoints.clear();
        self.b.play_sound("select");
        if self.practice {
            if let Some(run) = &self.run {
                if run.dead == 0 && run.won == 0 && (run.ground != 0 || run.mode != core::CUBE) {
                    self.checkpoints.push(run.clone());
                }
            }
        } else {
            self.new_attempt(false);
        }
    }

    fn place_checkpoint(&mut self) {
        let Some(s) = &self.run else {
            return;
        };
        if s.dead != 0 || s.won != 0 {
            return;
        }
        let step = s.step;
        self.checkpoints.push(s.clone());
        if self.checkpoints.len() > 200 {
            self.checkpoints.remove(0);
        }
        self.last_cp_step = step;
        self.b.play_sound("powerup");
    }

    fn remove_checkpoint(&mut self) {
        if self.checkpoints.pop().is_some() {
            self.b.play_sound("click");
        }
    }

    fn restart_now(&mut self) {
        let Some(run) = &self.run else {
            return;
        };
        let pct = core::progress(run, self.lv.as_ref().expect("lv"));
        self.record_best(pct);
        self.new_attempt(self.practice);
    }

    /// `_log_level_match(won)` - Match-Historie: Level-Besuch abschliessen
    /// (einmal je Besuch; Test-Spielen aus dem Editor zaehlt nicht).
    fn log_level_match(&mut self, won: bool) {
        if self.gd_logged || self.cur_kind == "test" || self.session_attempts <= 0 {
            return;
        }
        let Some(cur) = self.cur.as_ref() else { return };
        self.gd_logged = true;
        let name = match cur.get("name") {
            Some(Value::String(n)) => n.chars().take(40).collect::<String>(),
            Some(Value::Null) => "None".to_string(),
            Some(v) => v.to_string().chars().take(40).collect(),
            None => String::new(),
        };
        let mut info = info_map(vec![("lv", name.into()), ("g", self.session_attempts.into())]);
        if won {
            if let Some(run) = self.run.as_ref() {
                info.insert("c".into(), (run.coins.count_ones() as i64).into());
            }
        }
        let score = if won { 100 } else { self.gd_best_pct };
        let mode = if self.practice { "practice" } else { "normal" };
        self.b.log_match(score, Some(if won { 1 } else { 0 }), Some(info), Some(mode), 0);
    }

    fn leave_to_select(&mut self) {
        audio::music_stop(Some(self.ident()));
        self.log_level_match(false);
        self.save_progress();
        self.state = St::Select;
        self.cur = None;
        self.run = None;
        self.b.score = self.total_stars();
        self.b.play_sound("click");
    }

    // ===================================================== Update

    fn update_play(&mut self, dt: f64) {
        self.music_t += dt;
        let now = py::perf_counter();
        if self.run.as_ref().expect("run").dead != 0 {
            self.queue.clear();
            self.dead_t += dt;
            if self.dead_t >= DEATH_PAUSE {
                self.new_attempt(self.practice);
            }
            return;
        }
        self.acc += dt;
        let n = (self.acc * core::HZ as f64 + 1e-9) as i64;
        if n <= 0 {
            return;
        }
        self.acc -= n as f64 / core::HZ as f64;
        let t0 = now - 0f64.max(self.acc) - n as f64 / core::HZ as f64;
        let mut events = std::mem::take(&mut self.queue);
        let mut ei = 0;
        events.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let lv = self.lv.as_ref().expect("lv");
        let mut s = self.run.take().expect("run");
        for i in 0..n {
            let mut pressed = false;
            while ei < events.len() {
                let (te, down, ref src) = events[ei];
                if ((te - t0) * core::HZ as f64).floor() as i64 > i {
                    break;
                }
                ei += 1;
                if down {
                    if !self.sources.contains(src) {
                        self.sources.insert(src.clone());
                        pressed = true;
                        self.jumps += 1;
                        self.session_jumps += 1;
                        self.dirty = true;
                    }
                } else {
                    self.sources.remove(src);
                }
            }
            let held = !self.sources.is_empty();
            core::step(&mut s, lv, held || pressed, pressed);
            if s.dead != 0 || s.won != 0 {
                break;
            }
            if self.practice && self.auto_cp && s.step - self.last_cp_step >= AUTO_CHECK_STEPS && (s.ground != 0 || s.mode != core::CUBE) {
                self.checkpoints.push(s.clone());
                self.last_cp_step = s.step;
            }
        }
        if s.dead != 0 || s.won != 0 {
            for (_, down, src) in events.drain(ei..) {
                if down {
                    self.sources.insert(src);
                } else {
                    self.sources.remove(&src);
                }
            }
        } else {
            let mut rest: Vec<_> = events.drain(ei..).collect();
            rest.append(&mut self.queue);
            self.queue = rest;
        }
        let (dead, won) = (s.dead != 0, s.won != 0);
        self.run = Some(s);
        self.run_time += dt;
        self.session_time += dt;
        self.update_visuals(dt);
        if dead {
            self.on_death();
        } else if won {
            self.on_win();
        }
    }

    fn update_visuals(&mut self, dt: f64) {
        let (px, pyy) = self.player_blocks();
        let held = !self.sources.is_empty();
        let s = self.run.as_ref().expect("run");
        let mode = s.mode;
        let g = s.grav as f64;
        let speed_bps = (core::SPEEDS[s.speed as usize] * core::HZ) as f64 / core::B as f64;
        if mode == core::CUBE {
            if s.ground != 0 {
                let target = py::round(self.angle / 90.0) as f64 * 90.0;
                self.angle += (target - self.angle) * 1f64.min(dt * 22.0);
            } else {
                self.angle += 420.0 * dt * g;
            }
        } else if mode == core::BALL {
            self.angle += speed_bps / 0.5 * 57.3 * dt * g;
        } else if mode == core::SHIP {
            let vy = (s.vy * core::HZ) as f64 / core::B as f64;
            let want = py::degrees(vy.atan2(speed_bps)) * 0.8;
            self.angle += (want - self.angle) * 1f64.min(dt * 14.0);
        } else if mode == core::WAVE {
            self.angle = (if held { 45.0 } else { -45.0 }) * g;
        } else {
            self.angle *= 0f64.max(1.0 - dt * 10.0);
        }
        if mode == core::SHIP || mode == core::WAVE || mode == core::BALL || mode == core::UFO {
            self.trail.push((px, pyy));
            let limit = if mode == core::WAVE { 60 } else { 14 };
            if self.trail.len() > limit {
                self.trail.remove(0);
            }
        } else {
            self.trail.clear();
        }
        let ceil = s.ceil;
        let ty = self.cam_target(pyy);
        self.cam_y += (ty - self.cam_y) * 1f64.min(dt * if ceil != 0 { 5.0 } else { 3.5 });
    }

    fn cam_target(&self, pyy: f64) -> f64 {
        let view_h = self.b.height as f64 / self.ts() as f64;
        let s = self.run.as_ref();
        if let Some(s) = s.filter(|s| s.ceil > 0) {
            let c = s.ceil as f64 / core::B as f64;
            return c / 2.0 - view_h / 2.0;
        }
        let base = -view_h * 0.18;
        if s.is_some_and(|s| s.grav < 0) {
            return base.max(pyy - view_h * 0.62);
        }
        base.max(pyy - view_h * 0.45)
    }

    fn ts(&self) -> i32 {
        8.max((self.b.height as f64 / VIEW_ROWS) as i32)
    }

    // ----- Tod und Ziel -----------------------------------------------------

    fn on_death(&mut self) {
        self.dead_t = 0.0;
        let (px, pyy) = self.player_blocks();
        self.burst(px + 0.5, pyy + 0.5, gdraw::COL_P1, 34);
        self.burst(px + 0.5, pyy + 0.5, gdraw::COL_P2, 18);
        self.shock = Some((px + 0.5, pyy + 0.5, self.t));
        self.b.play_sound("explode");
        self.b.rumble(160);
        let pct = core::progress(self.run.as_ref().expect("run"), self.lv.as_ref().expect("lv"));
        self.gd_best_pct = self.gd_best_pct.max(pct);
        if self.record_best(pct) {
            self.new_best = Some((pct, self.t));
        }
        if !self.practice {
            audio::music_stop(Some(self.ident()));
        }
    }

    fn record_best(&mut self, pct: i64) -> bool {
        if self.cur_kind == "test" || self.start_block.is_some_and(|b| b != 0) {
            return false;
        }
        let key = self.best_key();
        let idx = if self.practice { 1 } else { 0 };
        let best = self.best_mut(&key);
        if pct > best[idx] {
            best[idx] = pct;
            self.dirty = true;
            self.save_progress();
            return true;
        }
        false
    }

    fn on_win(&mut self) {
        let s = self.run.clone().expect("run");
        self.state = St::Complete;
        self.complete_t = 0.0;
        self.b.play_sound("win");
        self.b.rumble(220);
        let (px, pyy) = self.player_blocks();
        for c in [gdraw::COL_P1, gdraw::COL_P2, gdraw::COL_COIN] {
            self.burst(px + 0.5, pyy + 0.5, c, 26);
        }
        ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 90);
        let mut info = Info {
            attempts: self.session_attempts,
            jumps: self.session_jumps,
            time: self.session_time,
            coins: s.coins,
            coin_count: self.lv.as_ref().expect("lv").coin_count,
            practice: self.practice,
            ..Info::default()
        };
        let new_record = self.record_best(100);
        self.log_level_match(true);
        let cur = self.cur.clone().expect("cur");
        if self.cur_kind == "main" && !self.practice {
            let lid = s_of(&cur, "id").to_string();
            let old_mask = count_get(&self.coins, &lid);
            let new_mask = old_mask | s.coins;
            info.new_coins = total_coins_mask(new_mask) - total_coins_mask(old_mask);
            if new_record {
                info.stars = i_of(&cur, "stars");
                self.b.ach_event("gd_first", None);
                if i_of(&cur, "difficulty") >= 5 {
                    self.b.ach_event("gd_demon", None);
                }
            }
            if new_mask != old_mask {
                count_set(&mut self.coins, &lid, new_mask);
                self.dirty = true;
            }
            if new_mask == 7 {
                self.b.ach_event("gd_coins", None);
            }
            self.b.score = self.total_stars();
        } else if self.cur_kind == "extra" && !self.practice {
            let key = self.best_key();
            let old_mask = count_get(&self.coins, &key);
            let new_mask = old_mask | s.coins;
            info.new_coins = total_coins_mask(new_mask) - total_coins_mask(old_mask);
            if new_mask != old_mask {
                count_set(&mut self.coins, &key, new_mask);
                self.dirty = true;
            }
        } else if self.cur_kind == "test" && self.test == Some(true) && !self.practice && !self.start_block.is_some_and(|b| b != 0) {
            info.verified = true;
            if let Some(ed) = self.editor.as_mut() {
                ed.mark_verified();
            }
        }
        self.save_progress();
        self.complete_info = info;
        audio::music_set_volume(0.45);
    }

    // ----- Partikel ------------------------------------------------------------

    fn burst(&mut self, xb: f64, yb: f64, col: (i32, i32, i32), n: usize) {
        for _ in 0..n {
            let a = py::uniform(0.0, TAU);
            let sp = py::uniform(2.0, 9.0);
            let life = py::uniform(0.4, 0.9);
            let size = py::uniform(0.08, 0.22);
            self.particles.push(Particle { x: xb, y: yb, vx: a.cos() * sp, vy: a.sin() * sp, life, col, size });
        }
    }

    fn update_particles(&mut self, dt: f64) {
        if self.particles.is_empty() {
            return;
        }
        let mut keep = Vec::with_capacity(self.particles.len());
        for mut p in self.particles.drain(..) {
            p.life -= dt;
            if p.life <= 0.0 {
                continue;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy -= 14.0 * dt;
            p.vx *= 1.0 - 1.5 * dt;
            keep.push(p);
        }
        let n = keep.len();
        self.particles = keep.split_off(n.saturating_sub(400));
    }

    // ===================================================== Level-Auswahl

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let ts = self.ts();
        self.renderer.resize(w, h, ts as f64);
        let sh = self.small.get_height();
        let th = self.tiny.get_height();
        let tab_h = 22.max(30.min(h / 15)).max(sh + 10);
        let n_tabs = TABS.len() as i32;
        let gap_t = 8;
        let tab_w = 110.max(w / (n_tabs + 2)).min((w - 40 - gap_t * (n_tabs - 1)) / n_tabs);
        let tab_y = (h as f64 * 0.20) as i32;
        let total_t = n_tabs * tab_w + gap_t * (n_tabs - 1);
        self.tab_rects = (0..n_tabs).map(|i| Rect::new(cx - total_t / 2 + i * (tab_w + gap_t), tab_y, tab_w, tab_h)).collect();
        self.tab_bottom = tab_y + tab_h;
        let foot = th * 2 + 14;
        let bh = 24.max(40.min(h / 14)).max(sh + 14);
        let gap = 6.max(h / 60);
        let opt_h = 22.max(34.min(h / 16)).max(th + 12);
        let card_top = self.tab_bottom + gap + 2;
        let card_bottom = h - foot - opt_h - bh - 3 * gap - 8.max(h / 50);
        let arrow = 28.max(52.min(w / 14));
        let card_w = (w - 2 * arrow - 44).min(620.max((w as f64 * 0.6) as i32));
        self.card_rect = Rect::new(cx - py::fdiv32(card_w, 2), card_top, card_w, 90.max(card_bottom - card_top));
        let cr = self.card_rect;
        self.arrow_rects = [
            Rect::new(cr.x - arrow - 10, cr.centery() - arrow / 2, arrow, arrow),
            Rect::new(cr.right() + 10, cr.centery() - arrow / 2, arrow, arrow),
        ];
        self.dots_y = cr.bottom() + 6.max(h / 70);
        let mut y = self.dots_y + 6.max(h / 70);
        let bw = (w - 40).min(380.max((w as f64 * 0.62) as i32));
        let x0 = cx - bw / 2;
        let mode_w = (bw as f64 * 0.29) as i32;
        self.mode_rects = [Rect::new(x0, y, mode_w, bh), Rect::new(x0 + mode_w + gap, y, mode_w, bh)];
        let sx = x0 + 2 * mode_w + 2 * gap + gap;
        self.start_rect = Rect::new(sx, y, x0 + bw - sx, bh);
        y += bh + gap;
        let ow = (bw - 2 * gap) / 3;
        self.opt_rects = ["music", "bar", "auto"].iter().enumerate().map(|(i, k)| (*k, Rect::new(x0 + i as i32 * (ow + gap), y, ow, opt_h))).collect();
    }

    fn set_tab(&mut self, tab: &'static str) {
        if tab == self.setup_tab {
            return;
        }
        self.setup_tab = tab;
        self.build_panel();
        let tb = self.tab_bottom;
        if self.setup_tab == "levels" {
            if let Some(l) = self.lists.as_mut() {
                l.layout(&self.b, tb);
                l.reload();
            }
        } else if self.setup_tab == "extra" {
            if let Some(x) = self.extra.as_mut() {
                x.layout(&self.b, tb);
                x.reload();
            }
        }
        self.save_setting("last_tab", json!(tab));
        self.b.play_sound("click");
    }

    /// `_panel(build=True)` - legt die Liste des aktiven Reiters bei Bedarf an.
    fn build_panel(&mut self) {
        if self.setup_tab == "levels" && self.lists.is_none() {
            self.lists = Some(LevelList::new(&self.b, self.tab_bottom));
        }
        if self.setup_tab == "extra" && self.extra.is_none() {
            self.extra = Some(ExtraList::new(&self.b, self.tab_bottom));
        }
    }

    fn do_action(&mut self, act: GdAction) {
        match act {
            GdAction::None => {}
            GdAction::NewLevel(tpl) => self.ugc_edit(edit::new_level("", "", None, tpl.as_ref())),
            GdAction::Edit(m) => self.ugc_edit(m),
            GdAction::EditCopied(m) => {
                if let Some(l) = self.lists.as_mut() {
                    l.reload();
                }
                self.ugc_edit(m);
            }
            GdAction::Play(m) => {
                self.test = None;
                self.start_level(&m, "ugc", None, None);
            }
            GdAction::PlayExtra(d) => self.start_level(&d, "extra", None, None),
            GdAction::Test(d, start) => {
                self.test = Some(start.is_none_or(|s| s == 0));
                self.start_level(&d, "test", start, Some(false));
            }
            GdAction::Close => self.ugc_close_editor(),
        }
    }

    fn handle_select(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::MouseDown {
            if let Some(i) = self.tab_rects.iter().position(|rc| rc.hit(ev.pos)) {
                self.set_tab(TABS[i]);
                return;
            }
        } else if ev.kind == Kind::KeyDown && (ev.key == "Tab" || ev.key == "ISO_Left_Tab") {
            let step = if ev.key == "ISO_Left_Tab" { -1 } else { 1 };
            let i = TABS.iter().position(|t| *t == self.setup_tab).unwrap_or(0) as i64;
            self.set_tab(TABS[(i + step).rem_euclid(TABS.len() as i64) as usize]);
            return;
        }
        self.build_panel();
        if self.setup_tab != "play" {
            if ev.kind == Kind::KeyDown && ev.key == "Escape" {
                self.set_tab("play");
                return;
            }
            let act = if self.setup_tab == "levels" {
                self.lists.as_mut().map(|l| l.handle(&mut self.b, ev))
            } else {
                self.extra.as_mut().map(|x| x.handle(&mut self.b, ev))
            };
            if let Some(a) = act {
                self.do_action(a);
            }
            return;
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let n = self.levels.len() as i64;
                if k == "Left" || self.b.is_action(k, "left", None) {
                    self.step_level(-1);
                } else if k == "Right" || self.b.is_action(k, "right", None) {
                    self.step_level(1);
                } else if ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"].contains(&k) {
                    let i = if k == "0" { 9 } else { k.parse::<i64>().unwrap_or(1) - 1 };
                    if i < n {
                        self.step_level(i - self.sel as i64);
                    }
                } else if k == "Home" {
                    self.step_level(-(self.sel as i64));
                } else if k == "End" {
                    self.step_level(n - 1 - self.sel as i64);
                } else if k == "Return" || k == "space" || k == "KP_Enter" {
                    self.play_selected();
                } else if k == "p" || k == "P" {
                    self.practice = !self.practice;
                    self.b.play_sound("select");
                } else if k == "m" || k == "M" {
                    self.toggle_option("music");
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                let pos = ev.pos;
                if self.arrow_rects[0].hit(pos) {
                    self.step_level(-1);
                } else if self.arrow_rects[1].hit(pos) {
                    self.step_level(1);
                } else if self.card_rect.hit(pos) || self.start_rect.hit(pos) {
                    self.play_selected();
                } else if self.mode_rects[0].hit(pos) {
                    if self.practice {
                        self.practice = false;
                        self.b.play_sound("select");
                    }
                } else if self.mode_rects[1].hit(pos) {
                    if !self.practice {
                        self.practice = true;
                        self.b.play_sound("select");
                    }
                } else {
                    if let Some(&(key, _)) = self.opt_rects.iter().find(|(_, rc)| rc.hit(pos)) {
                        self.toggle_option(key);
                        return;
                    }
                    let first = self.dots_first();
                    if let Some(i) = self.dot_rects().iter().position(|rc| rc.hit(pos)) {
                        self.step_level((first + i) as i64 - self.sel as i64);
                    }
                }
            }
            Kind::Wheel => self.step_level(if ev.delta > 0 { -1 } else { 1 }),
            _ => {}
        }
    }

    fn toggle_option(&mut self, key: &str) {
        match key {
            "music" => {
                self.music_on = !self.music_on;
                self.save_setting("music", json!(self.music_on));
                if !self.music_on {
                    audio::music_stop(Some(self.ident()));
                }
            }
            "bar" => {
                self.show_bar = !self.show_bar;
                self.save_setting("progress_bar", json!(self.show_bar));
            }
            _ => {
                self.auto_cp = !self.auto_cp;
                self.save_setting("auto_checkpoints", json!(self.auto_cp));
            }
        }
        self.b.play_sound("select");
    }

    fn step_level(&mut self, d: i64) {
        if d == 0 {
            return;
        }
        self.sel = (self.sel as i64 + d).rem_euclid(self.levels.len() as i64) as usize;
        self.card_anim = 1.0;
        self.card_dir = if d > 0 { 1 } else { -1 };
        self.save_setting("last_level", json!(self.sel));
        self.b.play_sound("move");
    }

    fn play_selected(&mut self) {
        self.test = None;
        let d = self.levels[self.sel].clone();
        self.start_level(&d, "main", None, None);
    }

    fn dots_first(&self) -> usize {
        let n = self.levels.len();
        if n <= DOTS_MAX {
            return 0;
        }
        0.max((self.sel as i64 - (DOTS_MAX / 2) as i64).min((n - DOTS_MAX) as i64)) as usize
    }

    fn dot_rects(&self) -> Vec<Rect> {
        let n = self.levels.len().min(DOTS_MAX) as i32;
        let r = 4.max(self.b.height / 110);
        let gap = r * 4;
        let x0 = self.b.width / 2 - py::fdiv32((n - 1) * gap, 2);
        (0..n).map(|i| Rect::new(x0 + i * gap - r - 2, self.dots_y - r - 2, 2 * r + 4, 2 * r + 4)).collect()
    }

    // ===================================================== Ergebnis

    fn complete_buttons(&self) -> (Rect, Vec<(&'static str, Rect)>) {
        let (w, h) = (self.b.width, self.b.height);
        let pw = (w - 30).min(460.max((w as f64 * 0.56) as i32));
        let sh = self.small.get_height();
        let bh = 26.max(38.min(h / 15)).max(sh + 12);
        let need = 12 + self.huge.get_height() + 4 + sh + 8 + 3 * (sh + 2) + 6 + 2 * 8.max(h / 40) + 8 + sh + 30 + bh + 14;
        let ph = (h - 30).min(220.max(need));
        let panel = Rect::new(py::fdiv32(w - pw, 2), py::fdiv32(h - ph, 2), pw, ph);
        let mut keys = vec!["again", "select"];
        if self.cur_kind == "main" && !self.levels.is_empty() && self.sel + 1 < self.levels.len() {
            keys.push("next");
        }
        if self.cur_kind == "extra" && self.extra.as_ref().is_some_and(|x| x.has_next()) {
            keys.push("next");
        }
        if self.cur_kind == "test" {
            keys = vec!["again", "editor"];
        }
        let gap = 8;
        let n = keys.len() as i32;
        let bw = py::fdiv32(pw - 30 - gap * (n - 1), n);
        let y = panel.bottom() - bh - 14;
        let rects = keys.iter().enumerate().map(|(i, k)| (*k, Rect::new(panel.x + 15 + i as i32 * (bw + gap), y, bw, bh))).collect();
        (panel, rects)
    }

    fn handle_complete(&mut self, ev: &InputEvent) {
        if self.complete_t < 0.35 {
            return;
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if ["Return", "space", "r", "R", "KP_Enter"].contains(&k) {
                    self.complete_action("again");
                } else if k == "Escape" || k == "BackSpace" {
                    self.complete_action(if self.cur_kind == "test" { "editor" } else { "select" });
                } else if k == "n" || k == "N" || k == "Right" {
                    self.complete_action("next");
                }
            }
            Kind::MouseDown if ev.button == 1 => {
                let (_p, rects) = self.complete_buttons();
                if let Some(&(key, _)) = rects.iter().find(|(_, rc)| rc.hit(ev.pos)) {
                    self.complete_action(key);
                }
            }
            _ => {}
        }
    }

    fn complete_action(&mut self, key: &str) {
        audio::music_set_volume(1.0);
        match key {
            "again" => {
                let cur = self.cur.clone().expect("cur");
                let (kind, sb) = (self.cur_kind, self.start_block);
                self.start_level(&cur, kind, sb, None);
            }
            "next" => {
                if self.cur_kind == "main" && self.sel + 1 < self.levels.len() {
                    self.sel += 1;
                    self.save_setting("last_level", json!(self.sel));
                    let d = self.levels[self.sel].clone();
                    self.start_level(&d, "main", None, None);
                } else if self.cur_kind == "extra" {
                    if let Some(x) = self.extra.as_mut() {
                        let act = x.play_next(&mut self.b);
                        self.do_action(act);
                    }
                }
            }
            "editor" => self.back_to_editor(),
            _ => {
                if self.cur_kind == "ugc" {
                    self.setup_tab = "levels";
                    if let Some(l) = self.lists.as_mut() {
                        l.reload();
                    }
                } else if self.cur_kind == "extra" {
                    self.setup_tab = "extra";
                }
                self.leave_to_select();
            }
        }
    }

    // ===================================================== Eigene Level

    fn ugc_edit(&mut self, m: Map<String, Value>) {
        audio::music_stop(Some(self.ident()));
        let grid = self.gs_truthy("grid", true);
        self.editor = Some(LevelEditor::new(&self.b, &m, grid));
        self.state = St::Edit;
        self.b.play_sound("click");
    }

    fn ugc_close_editor(&mut self) {
        self.editor = None;
        self.test = None;
        self.state = St::Select;
        self.setup_tab = "levels";
        if self.lists.is_none() {
            self.lists = Some(LevelList::new(&self.b, self.tab_bottom));
        }
        if let Some(l) = self.lists.as_mut() {
            l.reload();
        }
        self.b.play_sound("click");
    }

    fn back_to_editor(&mut self) {
        audio::music_stop(Some(self.ident()));
        audio::music_set_volume(1.0);
        self.test = None;
        self.run = None;
        if self.editor.is_none() {
            self.ugc_close_editor();
            return;
        }
        self.state = St::Edit;
        self.b.play_sound("click");
    }

    // ===================================================== Zeichnen

    fn draw_world(&mut self, s: &mut Surface) {
        let ts = self.ts();
        self.renderer.resize(self.b.width, self.b.height, ts as f64);
        let n = if self.cur_kind == "test" {
            self.session_attempts
        } else {
            let key = self.best_key();
            self.attempts.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(1)
        };
        let r = &mut self.renderer;
        let ts = r.ts;
        let run = self.run.as_ref().expect("run");
        let lv = self.lv.as_ref().expect("lv");
        let w = self.b.width;
        let px = run.x as f64 / core::B as f64;
        let pyy = run.y as f64 / core::B as f64;
        let view_w = w as f64 / ts as f64;
        let cam_x = px - view_w * 0.3;
        let cam_y = self.cam_y;
        let bg = gdraw::color_at(lv, px, 0);
        let ground = gdraw::color_at(lv, px, 1);
        let pulse = if self.bpm != 0 { music::beat_pulse(self.music_t, self.bpm) } else { 0.0 };
        r.draw_backdrop(s, bg, cam_x, cam_y, pulse);
        r.draw_objects(s, lv, cam_x, cam_y, self.t, pulse, &run.used, false, None);
        r.draw_ground(s, Some(lv), ground, cam_x, cam_y, pulse);
        if run.ceil > 0 {
            r.draw_ceiling(s, ground, run.ceil as f64 / core::B as f64, cam_y, pulse, 1.0);
        }
        let ax = r.sx(self.attempt_label_x, cam_x);
        let ay = r.sy(self.attempt_label_y, cam_y);
        if -(w as f64) < ax && ax < (w + 50) as f64 {
            let label = tf("gd.attempt", &[("n", &n.to_string())]);
            let img = self.attempt_font.render(&label, true, (255, 255, 255));
            let shd = self.attempt_font.render(&label, true, (0, 0, 0));
            s.bl(&shd, (ax + 3.0, ay + 3.0));
            s.bl(&img, (ax, ay));
        }
        if self.practice {
            let k0 = self.checkpoints.len().saturating_sub(30);
            for cp in &self.checkpoints[k0..] {
                let cx = r.sx(cp.x as f64 / core::B as f64 + 0.5, cam_x);
                let cy = r.sy(cp.y as f64 / core::B as f64 + 0.5, cam_y);
                if -(ts as f64) < cx && cx < (w + ts) as f64 {
                    let d = ts as f64 * 0.32;
                    let pts = [(cx, cy - d), (cx + d * 0.7, cy), (cx, cy + d), (cx - d * 0.7, cy)];
                    draw::polygon(s, gdraw::COL_CHECK, &pts, 0);
                    draw::polygon(s, (10, 40, 20), &pts, 1.max(ts / 16));
                }
            }
        }
        for p in &self.particles {
            let x = r.sx(p.x, cam_x);
            let y = r.sy(p.y, cam_y);
            let size = 2.max((ts as f64 * p.size * 1f64.min(p.life * 2.0)) as i32);
            draw::rect(s, p.col, (x as i32 - size / 2, y as i32 - size / 2, size, size), 0);
        }
        if let Some((sx0, sy0, st)) = self.shock {
            let age = self.t - st;
            if age < 0.45 {
                let f = age / 0.45;
                let rad = (ts as f64 * (0.4 + 2.6 * f)) as i32;
                let mut ring = surface_alpha(2 * rad + 4, 2 * rad + 4);
                draw::circle(&mut ring, rgba(255, 255, 255, (220.0 * (1.0 - f)) as i32), (rad + 2, rad + 2), rad, 2.max((ts as f64 * 0.18 * (1.0 - f)) as i32));
                s.bl(&ring, (r.sx(sx0, cam_x) - rad as f64 - 2.0, r.sy(sy0, cam_y) - rad as f64 - 2.0));
            }
        }
        if run.dead == 0 {
            r.draw_player(s, px, pyy, run.mode, run.grav, self.angle, cam_x, cam_y, Some(&self.trail), 255);
        }
    }

    fn draw_hud(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let pct = match (&self.run, &self.lv) {
            (Some(run), Some(lv)) => core::progress(run, lv),
            _ => 0,
        };
        if self.show_bar {
            let bw = (w as f64 * 0.42) as i32;
            let bh = 8.max(h / 64);
            let x = w / 2 - bw / 2;
            let y = 6.max(h / 60);
            draw::rect_r(s, (0, 0, 0), (x - 2, y - 2, bw + 4, bh + 4), 0, bh);
            draw::rect_r(s, (40, 40, 60), (x, y, bw, bh), 0, bh);
            let fill = (bw as f64 * pct as f64 / 100.0) as i32;
            if fill > 0 {
                draw::rect_r(s, gdraw::COL_P1, (x, y, fill, bh), 0, bh);
            }
            let txt = format!("{pct}%");
            let img = self.small.render(&txt, true, (255, 255, 255));
            let shd = self.small.render(&txt, true, (0, 0, 0));
            s.blr(&shd, shd.get_rect().midleft_at((x + bw + 11, y + bh / 2 + 1)));
            s.blr(&img, img.get_rect().midleft_at((x + bw + 10, y + bh / 2)));
        }
        if let (Some(run), Some(lv)) = (&self.run, &self.lv) {
            if lv.coin_count != 0 {
                let r = 6.max(h / 50);
                for i in 0..lv.coin_count {
                    let cx = w - 14 - r - i as i32 * (2 * r + 6);
                    let got = run.coins & (1 << (lv.coin_count - 1 - i)) != 0;
                    if got {
                        gdraw::draw_coin(s, cx as f64, (14 + r) as f64, r as f64, 0.0, false);
                    } else {
                        draw::circle(s, (0, 0, 0), (cx, 14 + r), r, 2);
                    }
                }
            }
        }
        let mut lines: Vec<(String, Color)> = Vec::new();
        if self.practice {
            lines.push((t("gd.practice_hud"), rgb(90, 255, 140)));
        }
        if self.cur_kind == "test" {
            lines.push((t("gd.test_hud"), rgb(255, 220, 120)));
        }
        let mut yb = h - 8;
        for (text, col) in lines.iter().rev() {
            let img = self.tiny.render(text, true, *col);
            let bx = img.get_rect().midbottom_at((w / 2, yb));
            let mut bg = surface_alpha(bx.w + 16, bx.h + 6);
            bg.fill_c(rgba(0, 0, 0, 140));
            s.bl(&bg, (bx.x - 8, bx.y - 3));
            s.blr(&img, bx);
            yb = bx.y - 8;
        }
        if let Some((nb, nt)) = self.new_best.filter(|_| self.state == St::Play) {
            let age = self.t - nt;
            if age < 1.6 {
                let txt = tf("gd.new_best", &[("pct", &nb.to_string())]);
                let sc = 1.0 + 0.25 * 0f64.max(0.25 - age) * 4.0;
                let f = ui::font((18.max(h / 14) as f64 * sc) as i32, true, false);
                let img = f.render(&txt, true, gdraw::COL_P1);
                let shd = f.render(&txt, true, (0, 0, 0));
                let pos = img.get_rect().center_at((w / 2, (h as f64 * 0.32) as i32));
                s.blr(&shd, pos.move_(3, 3));
                s.blr(&img, pos);
            }
        }
    }

    fn draw_complete(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let info = self.complete_info.clone();
        let a = 1f64.min(self.complete_t * 3.0);
        let mut veil = surface_alpha(w, h);
        veil.fill_c(rgba(0, 0, 0, (140.0 * a) as i32));
        s.bl(&veil, (0, 0));
        let (panel, rects) = self.complete_buttons();
        let panel = panel.move_(0, ((1.0 - a) * 40.0) as i32);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(self.b.accent));
        let cx = panel.centerx();
        let key = if info.practice { "gd.complete_practice" } else { "gd.complete" };
        let mut title = self.huge.render(&t(key), true, self.b.accent);
        if title.w() > panel.w - 20 {
            title = self.mid.render(&t(key), true, self.b.accent);
        }
        let mut y = panel.y + 12;
        s.blr(&title, title.get_rect().midtop_at((cx, y)));
        y += title.h() + 4;
        let name = self.small.render(s_of(self.cur.as_ref().expect("cur"), "name"), true, p.text);
        s.blr(&name, name.get_rect().midtop_at((cx, y)));
        y += name.h() + 8;
        let secs_total = info.time as i64;
        let (mins, secs) = (secs_total.div_euclid(60), secs_total.rem_euclid(60));
        let rows = [
            (t("gd.stat_attempts"), info.attempts.to_string()),
            (t("gd.stat_jumps"), info.jumps.to_string()),
            (t("gd.stat_time"), format!("{mins}:{secs:02}")),
        ];
        let lw = panel.w - 60;
        for (label, val) in &rows {
            let li = self.small.render(label, true, p.text_dim);
            let vi = self.small.render(val, true, p.text);
            s.bl(&li, (panel.x + 30, y));
            s.blr(&vi, vi.get_rect().topright_at((panel.x + 30 + lw, y)));
            y += li.h() + 2;
        }
        y += 6;
        let rr = 8.max(h / 40);
        let n = info.coin_count as i32;
        if n != 0 {
            let total_w = n * (2 * rr + 10);
            for i in 0..n {
                let ccx = cx - total_w / 2 + rr + i * (2 * rr + 10);
                if info.coins & (1 << i) != 0 {
                    gdraw::draw_coin(s, ccx as f64, (y + rr) as f64, rr as f64, self.t, false);
                } else {
                    draw::circle(s, p.text_faint, (ccx, y + rr), rr, 2);
                }
            }
            y += 2 * rr + 8;
        }
        let mut extra = Vec::new();
        if info.stars != 0 {
            extra.push(tf("gd.stars_won", &[("n", &info.stars.to_string())]));
        }
        if info.new_coins != 0 {
            extra.push(tf("gd.coins_won", &[("n", &info.new_coins.to_string())]));
        }
        if info.verified {
            extra.push(t("gd.verified_now"));
        }
        if info.practice {
            extra.push(t("gd.practice_note"));
        }
        if !extra.is_empty() {
            let line = extra.join("  \u{00B7}  ");
            let mut img = self.small.render(&line, true, p.gold);
            if img.w() > panel.w - 20 {
                img = self.tiny.render(&line, true, p.gold);
            }
            s.blr(&img, img.get_rect().midtop_at((cx, y)));
        }
        for (key, rc) in rects {
            let rc = rc.move_(0, ((1.0 - a) * 40.0) as i32);
            let label = match key {
                "again" => t("gd.btn_again"),
                "select" => t("gd.btn_levels"),
                "next" => t("gd.btn_next"),
                _ => t("gd.btn_editor"),
            };
            self.btn(s, rc, &label, key == "again", false);
        }
    }

    fn draw_select(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let cx = w / 2;
        let title = self.huge.render(&t("gd.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.085) as i32)));
        let sub_key = match self.setup_tab {
            "extra" => "gd.more.subtitle",
            "levels" => "gd.ugc.subtitle",
            _ => "gd.subtitle",
        };
        let sub = self.small.render(&t(sub_key), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.155) as i32)));
        for (i, rc) in self.tab_rects.clone().iter().enumerate() {
            self.btn(s, *rc, &t(&format!("gd.tab_{}", TABS[i])), self.setup_tab == TABS[i], false);
        }
        self.build_panel();
        if self.setup_tab == "levels" {
            if let Some(l) = &self.lists {
                l.draw(s, &self.b, &self.best);
            }
            return;
        }
        if self.setup_tab == "extra" {
            if let Some(x) = &self.extra {
                x.draw(s, &self.b, &self.best, &self.coins);
            }
            return;
        }
        self.draw_card(s);
        for (i, rc) in self.arrow_rects.iter().enumerate() {
            draw::rect_r(s, p.btn, *rc, 0, rc.w / 4);
            draw::rect_r(s, p.border, *rc, 1, rc.w / 4);
            let d = if i == 0 { -1.0 } else { 1.0 };
            let m = (rc.w / 4) as f64;
            let (ccx, ccy) = (rc.centerx() as f64, rc.centery() as f64);
            let pts = [(ccx - d * m * 0.6, ccy - m), (ccx + d * m * 0.8, ccy), (ccx - d * m * 0.6, ccy + m)];
            draw::polygon(s, self.b.accent, &pts, 0);
        }
        let first = self.dots_first();
        let rects = self.dot_rects();
        for (i, rc) in rects.iter().enumerate() {
            let idx = first + i;
            let on = idx == self.sel;
            let edge = (i == 0 && first > 0) || (i == rects.len() - 1 && first + rects.len() < self.levels.len());
            let col = if on { self.b.accent } else { p.border_light };
            let r = rc.w / 2 - if on { 2 } else { 4 } - if edge { 1 } else { 0 };
            draw::circle(s, col, rc.center(), 2.max(r), 0);
        }
        if self.levels.len() > DOTS_MAX {
            let num = self.tiny.render(&tf("gd.level_of", &[("n", &(self.sel + 1).to_string()), ("max", &self.levels.len().to_string())]), true, p.text_dim);
            s.blr(&num, num.get_rect().midtop_at((cx, self.dots_y + rects[0].h)));
        }
        let (m0, m1) = (self.mode_rects[0], self.mode_rects[1]);
        self.btn(s, m0, &t("gd.mode.normal"), !self.practice, false);
        self.btn(s, m1, &t("gd.mode.practice"), self.practice, false);
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let mut st = self.btn_font.render(&t("common.start"), true, p.text);
        if st.w() > self.start_rect.w - 10 {
            st = self.small.render(&t("common.start"), true, p.text);
        }
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        for (key, on) in [("music", self.music_on), ("bar", self.show_bar), ("auto", self.auto_cp)] {
            let label = format!("{}: {}", t(&format!("gd.opt_{key}")), if on { t("common.on") } else { t("common.off") });
            let rc = self.opt_rects.iter().find(|e| e.0 == key).map(|e| e.1).expect("opt");
            self.btn(s, rc, &label, on, true);
        }
        let stars = self.total_stars();
        let line = tf("gd.total_stars", &[("n", &stars.to_string()), ("max", &self.max_stars().to_string())]);
        let img = self.tiny.render(&line, true, p.gold);
        let hint_h = self.tiny.get_height();
        let yline = h - 6 - hint_h - self.tiny.get_height() / 2 - 2;
        let r = 5.max(self.tiny.get_height() / 2 - 1);
        let tw = img.w() + 2 * r + 6;
        gdraw::draw_star(s, (cx - tw / 2 + r) as f64, yline as f64, r as f64, p.gold, (20, 20, 30));
        s.blr(&img, img.get_rect().midleft_at((cx - tw / 2 + 2 * r + 6, yline)));
        let mut hint = self.tiny.render(&t("gd.select_hint"), true, p.text_dim);
        if hint.w() > w - 16 {
            hint = ui::font(10.max(h / 48), false, false).render(&t("gd.select_hint"), true, p.text_dim);
        }
        s.blr(&hint, hint.get_rect().midbottom_at((cx, h - 4)));
    }

    /// Aufteilung der Level-Karte.
    fn card_parts(&self, rect: Rect) -> (i32, i32, i32, i32, [Rect; 2], Rect, i32) {
        let pad = rect.x + 8 + 6.max(rect.w / 60) + 12;
        let (mh, sh, th) = (self.mid.get_height(), self.small.get_height(), self.tiny.get_height());
        let head_h = (mh + sh + th + 10).max(2 * 14.max((rect.h / 7).min(rect.w / 14).min(44)) + th + 6);
        let fr = 12.max((rect.h / 7).min(rect.w / 14).min(44).min(py::fdiv32(head_h - th - 6, 2)));
        let lab_h = th;
        let bar_h = 10.max(20.min(rect.h / 12));
        let side = rect.w >= 420;
        let block = lab_h + 2 + bar_h;
        let bars_top = rect.bottom() - 12 - if side { block } else { 2 * block + 6 };
        let bw_all = rect.right() - 14 - pad;
        let mut bars = [Rect::new(0, 0, 0, 0); 2];
        for (i, bar) in bars.iter_mut().enumerate() {
            let i = i as i32;
            *bar = if side {
                let bw = py::fdiv32(bw_all - 16, 2);
                Rect::new(pad + i * (bw + 16), bars_top + lab_h + 2, bw, bar_h)
            } else {
                Rect::new(pad, bars_top + i * (block + 6) + lab_h + 2, bw_all, bar_h)
            };
        }
        let head_top = rect.y + 12;
        let thumb = Rect::new(pad, head_top + head_h + 8, bw_all, bars_top - 8 - (head_top + head_h + 8));
        (pad, fr, head_top, head_h, bars, thumb, rect.right() - 14)
    }

    fn draw_card(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let d = self.levels[self.sel].clone();
        let lid = s_of(&d, "id").to_string();
        let rect = self.card_rect.move_((self.card_anim * 30.0 * self.card_dir as f64) as i32, 0);
        let diff = i_of(&d, "difficulty");
        let col = gdraw::DIFF_COLORS[diff.clamp(0, 5) as usize];
        ui::draw_panel_ex(s, rect, None, None, None, true, Some(rgb(col.0, col.1, col.2)));
        let bg = match d.get("bg").and_then(|v| v.as_array()) {
            Some(a) => rgb(a.first().and_then(|x| x.as_i64()).unwrap_or(0), a.get(1).and_then(|x| x.as_i64()).unwrap_or(0), a.get(2).and_then(|x| x.as_i64()).unwrap_or(0)),
            None => rgb(40, 110, 255),
        };
        let strip = Rect::new(rect.x + 8, rect.y + 8, 6.max(rect.w / 60), rect.h - 16);
        draw::rect_r(s, bg, strip, 0, strip.w / 2);
        let (pad, fr, top, _head_h, bars, thumb, right) = self.card_parts(rect);
        let face_c = (pad + fr, top + fr);
        gdraw::draw_face(s, face_c.0 as f64, face_c.1 as f64, fr as f64, diff);
        let dk = t(&format!("gd.diff.{}", DIFF_KEYS[diff.clamp(0, 5) as usize]));
        let mut dl = self.tiny.render(&dk, true, col);
        if dl.w() > 2 * fr + 30 {
            dl = ui::font(9.max(self.b.height / 52), false, false).render(&dk, true, col);
        }
        s.blr(&dl, dl.get_rect().midtop_at((face_c.0, face_c.1 + fr + 3)));
        let tx = pad + 2 * fr + 18.max(dl.w() / 2 - fr + 10);
        let name = s_of(&d, "name");
        let mut img = self.mid.render(name, true, p.text);
        if img.w() > right - tx {
            img = self.small.render(name, true, p.text);
        }
        s.bl(&img, (tx, top - 2));
        let ny = top - 2 + img.h() + 3;
        let stars = i_of(&d, "stars");
        let sr = 6.max(self.small.get_height() / 2 - 1);
        let done = best_get(&self.best, &lid)[0] >= 100;
        gdraw::draw_star(s, (tx + sr) as f64, (ny + sr) as f64, sr as f64, if done { p.gold } else { p.text_faint }, (20, 20, 30));
        let st = self.small.render(&stars.to_string(), true, if done { p.gold } else { p.text_dim });
        s.bl(&st, (tx + 2 * sr + 5, ny + sr - st.h() / 2));
        let mask = count_get(&self.coins, &lid);
        let cx0 = tx + 2 * sr + 5 + st.w() + 16;
        for i in 0..3 {
            let ccx = cx0 + sr + i * (2 * sr + 6);
            if mask & (1 << i) != 0 {
                gdraw::draw_coin(s, ccx as f64, (ny + sr) as f64, sr as f64, self.t + i as f64, false);
            } else {
                draw::circle(s, p.text_faint, (ccx, ny + sr), sr, 2);
            }
        }
        let att = self.tiny.render(&tf("gd.attempts_total", &[("n", &count_get(&self.attempts, &lid).to_string())]), true, p.text_faint);
        s.bl(&att, (tx, ny + 2 * sr + 5));
        if thumb.h >= 26 && thumb.w >= 60 {
            let img = self.thumb(&d, thumb.w, thumb.h);
            s.bl(&img, (thumb.x, thumb.y));
            draw::rect_r(s, p.border, thumb, 1, 6);
        }
        let best = best_get(&self.best, &lid);
        for (i, (key, bcol)) in [("gd.mode.normal", gdraw::COL_P1), ("gd.mode.practice", gdraw::COL_CHECK)].iter().enumerate() {
            let bar = bars[i];
            let lab = self.tiny.render(&tf("gd.best_label", &[("mode", &t(key))]), true, p.text_dim);
            s.bl(&lab, (bar.x, bar.y - lab.h() - 2));
            let pct = self.tiny.render(&format!("{}%", best[i]), true, p.text);
            s.blr(&pct, pct.get_rect().bottomright_at((bar.right(), bar.y - 2)));
            draw::rect_r(s, p.btn, bar, 0, bar.h / 2);
            let fill = (bar.w as f64 * best[i] as f64 / 100.0) as i32;
            if fill > 0 {
                draw::rect_r(s, *bcol, (bar.x, bar.y, bar.h.max(fill), bar.h), 0, bar.h / 2);
            }
        }
    }

    fn thumb(&mut self, d: &Map<String, Value>, w: i32, h: i32) -> Surface {
        let key = (s_of(d, "id").to_string(), w, h);
        if self.thumbs.len() > 24 {
            self.thumbs.clear();
        }
        if let Some(img) = self.thumbs.get(&key) {
            return img.clone();
        }
        let lv = Level::new(d);
        gdraw::block_masks(&lv);
        let mut ren = Renderer::new();
        let rows = 7.5;
        ren.resize(w, h, 8.max((h as f64 / rows) as i32) as f64);
        let mut img = pg::surface(w, h);
        let (cam_x, cam_y) = (12.0, -1.2);
        ren.draw_backdrop(&mut img, gdraw::color_at(&lv, cam_x, 0), cam_x, cam_y, 0.0);
        ren.draw_objects(&mut img, &lv, cam_x, cam_y, 0.2, 0.0, &BTreeSet::new(), false, None);
        ren.draw_ground(&mut img, Some(&lv), gdraw::color_at(&lv, cam_x, 1), cam_x, cam_y, 0.0);
        ren.draw_player(&mut img, cam_x + 3.0, 0.0, core::CUBE, 1, 0.0, cam_x, cam_y, None, 255);
        let mut mask = surface_alpha(w, h);
        draw::rect_r(&mut mask, rgba(255, 255, 255, 255), (0, 0, w, h), 0, 6);
        let mut out = surface_alpha(w, h);
        out.bl(&img, (0, 0));
        out.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
        self.thumbs.insert(key, out.clone());
        out
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool, small: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 8);
        let col = if on { p.text } else { p.text_dim };
        let mut im = (if small { &self.tiny } else { &self.small }).render(text, true, col);
        if im.w() > rc.w - 12 {
            im = self.tiny.render(text, true, col);
        }
        if im.w() > rc.w - 8 {
            im = self.tiny.render(&edit::kurz(&self.tiny, text, rc.w - 8, 2, "..."), true, col);
        }
        s.blr(&im, im.get_rect().center_at(rc.center()));
    }
}

impl Game for GeometryDashGame {
    /// Match-Historie: das Spiel meldet seine Matches selbst (`log_match`).
    fn match_manual(&self) -> bool {
        true
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
        self.state == St::Edit || self.state == St::Complete || self.test.is_some() || (self.state == St::Select && self.setup_tab != "play")
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.levels = load_builtin_levels();
        let last = self.b.opt_int_py("geodash", "last_level", 0).unwrap_or(0);
        self.sel = 0.max((self.levels.len() as i64 - 1).min(last)) as usize;
        self.practice = self.b.mode == "practice";
        self.music_on = self.gs_truthy("music", true);
        self.show_bar = self.gs_truthy("progress_bar", true);
        self.auto_cp = self.gs_truthy("auto_checkpoints", true);
        let tab = self.b.opt_value("geodash", "last_tab").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "play".into());
        self.setup_tab = TABS.iter().find(|t| **t == tab).copied().unwrap_or("play");
        self.lists = None;
        self.extra = None;
        self.editor = None;
        self.test = None;
        self.renderer = Renderer::new();
        self.t = 0.0;
        self.card_anim = 0.0;
        self.load_progress();
        self.build_fonts();
        self.layout();
        self.state = St::Select;
        self.cur = None;
        self.lv = None;
        self.run = None;
        self.voices = None;
        self.voices_key = None;
        self.particles.clear();
        self.toast.clear();
        self.toast_t = 0.0;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.layout();
        let tb = self.tab_bottom;
        if let Some(l) = self.lists.as_mut() {
            l.layout(&self.b, tb);
        }
        if let Some(x) = self.extra.as_mut() {
            x.layout(&self.b, tb);
        }
        if let Some(e) = self.editor.as_mut() {
            e.layout(&self.b);
        }
    }

    fn on_exit(&mut self) {
        audio::music_stop(Some(self.ident()));
        self.log_level_match(false);
        self.save_progress();
    }

    /// Match-Historie: ein Match = ein Level-Besuch (Dauer nur im Level).
    fn match_active(&self) -> bool {
        self.run.is_some() && self.cur.is_some()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            St::Edit => {
                if let Some(e) = self.editor.as_mut() {
                    let act = e.handle(&mut self.b, ev);
                    self.do_action(act);
                }
            }
            St::Select => self.handle_select(ev),
            St::Complete => self.handle_complete(ev),
            St::Play => self.handle_play(ev),
        }
    }

    fn update(&mut self, dt: f64) {
        self.t += dt;
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.toast.clear();
            }
        }
        self.update_particles(dt);
        match self.state {
            St::Edit => {
                if let Some(e) = self.editor.as_mut() {
                    e.update(dt);
                }
            }
            St::Select => {
                self.card_anim = 0f64.max(self.card_anim - dt * 4.0);
                if self.setup_tab == "levels" {
                    if let Some(l) = self.lists.as_mut() {
                        l.update(dt);
                    }
                }
            }
            St::Complete => {
                self.complete_t += dt;
                self.music_t += dt;
            }
            St::Play => {
                if self.run.is_some() {
                    self.update_play(dt);
                }
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        match self.state {
            St::Edit if self.editor.is_some() => {
                ui::draw_background(&mut s, self.b.width, self.b.height);
                if let Some(e) = self.editor.as_mut() {
                    e.draw(&mut s, &self.b);
                }
            }
            St::Select => self.draw_select(&mut s),
            _ => {
                if self.run.is_none() {
                    return;
                }
                self.draw_world(&mut s);
                self.draw_hud(&mut s);
                if self.state == St::Complete {
                    self.draw_complete(&mut s);
                }
            }
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "geodash",
    class_name: "GeometryDashGame",
    supports_multiplayer: false,
    modes: &[("normal", "gd.mode.normal"), ("practice", "gd.mode.practice")],
    make: GeometryDashGame::new,
};
