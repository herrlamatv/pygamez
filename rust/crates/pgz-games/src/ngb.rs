//! ngb.rs
//! ======
//! NGB - die visuelle Personalisierung (nur Optik) - uebertragen aus `ngb.py`.
//!
//! Kopffarbe der Snake-Schlange, Raster-Overlay (Farbreihenfolge), Banner-
//! Einstellungen und das Personalisierungs-Menue mit drei Reitern. Gespeichert
//! im Profil-Abschnitt "ngb"; ein alter `mem-ngb.json` neben der .exe wird
//! einmalig uebernommen.

use std::cell::RefCell;

use pgz_core::{achievements, i18n, store};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

type Rgb = (i32, i32, i32);

/// Kopffarben-Vorlagen (`None` = eigene Farbe).
pub const HEAD_PRESETS: [(&str, Option<Rgb>); 7] = [
    ("aqua1", Some((90, 165, 245))),
    ("aqua2", Some((72, 195, 240))),
    ("aqua3", Some((58, 218, 224))),
    ("aqua4", Some((52, 234, 198))),
    ("red", Some((255, 95, 95))),
    ("orange", Some((255, 160, 60))),
    ("custom", None),
];
const DEFAULT_HEAD_ID: &str = "aqua3";
const DEFAULT_CUSTOM: Rgb = (200, 120, 255);

/// Raster-Vorlagen (`None` = aus bzw. eigene Farben).
pub fn grid_presets() -> Vec<(&'static str, Option<Vec<Rgb>>)> {
    vec![
        ("off", None),
        ("grid1", Some(vec![(44, 66, 108), (26, 40, 70)])),
        ("grid2", Some(vec![(36, 74, 58), (24, 50, 42)])),
        ("grid3", Some(vec![(70, 46, 98), (44, 32, 66)])),
        ("grid4", Some(vec![(92, 68, 34), (60, 46, 26)])),
        ("grid5", Some(vec![(58, 62, 74), (30, 33, 42), (44, 47, 58)])),
        ("custom", None),
    ]
}
const DEFAULT_GRID_ID: &str = "off";
const DEFAULT_GRID_CUSTOM: [Rgb; 2] = [(46, 64, 96), (28, 40, 66)];

const BANNER_SIZE_MIN: f64 = 0.6;
const BANNER_SIZE_MAX: f64 = 1.6;
const BANNER_OP_MIN: f64 = 0.2;
const BANNER_OP_MAX: f64 = 1.0;

const COL_BG: Rgb = (14, 15, 24);
const COL_TEXT: Rgb = (232, 232, 238);
const COL_DIM: Rgb = (150, 158, 176);
const COL_ACCENT: Rgb = (90, 160, 240);
const COL_BTN: Rgb = (44, 50, 66);
const COL_BTN_ON: Rgb = (60, 120, 80);
const COL_SEL: Rgb = (255, 255, 255);
const COL_TRACK: Rgb = (36, 40, 54);
const COL_TAB_OFF: Rgb = (34, 38, 52);

#[derive(Clone)]
struct State {
    head_id: String,
    head_custom: Rgb,
    grid_id: String,
    grid_custom: [Rgb; 2],
    banner_on: bool,
    banner_size: f64,
    banner_opacity: f64,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    static REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `reset_cache()` - nach einem Profilwechsel.
pub fn reset_cache() {
    STATE.with(|s| *s.borrow_mut() = None);
}

fn clamp_color(c: Option<&Value>, fallback: Rgb) -> Rgb {
    let Some(Value::Array(a)) = c else {
        return fallback;
    };
    let mut out = Vec::new();
    for v in a {
        let n = match v {
            Value::Number(n) => n.as_f64().map(|f| f.trunc() as i64),
            Value::Bool(b) => Some(*b as i64),
            Value::String(s) => s.trim().parse::<i64>().ok(),
            _ => None,
        };
        let Some(n) = n else {
            return fallback;
        };
        out.push(n.clamp(0, 255) as i32);
    }
    if out.len() < 3 {
        return fallback;
    }
    (out[0], out[1], out[2])
}

fn num(v: Option<&Value>, lo: f64, hi: f64, fallback: f64) -> f64 {
    let f = match v {
        None => return fallback,
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::Bool(b)) => Some(*b as i64 as f64),
        Some(Value::String(s)) => s.trim().parse::<f64>().ok(),
        _ => None,
    };
    match f {
        Some(f) => lo.max(hi.min(f)),
        None => fallback,
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

fn read_legacy() -> Map<String, Value> {
    let p = store::path_for("mem-ngb.json");
    std::fs::read_to_string(p).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v.as_object().cloned()).unwrap_or_default()
}

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.is_none() {
            if !REGISTERED.with(|r| r.replace(true)) {
                store::on_switch(reset_cache);
            }
            let mut data = store::load_section("ngb");
            if data.is_empty() {
                data = read_legacy();
            }
            let mut hid = data.get("head_id").and_then(|v| v.as_str()).unwrap_or(DEFAULT_HEAD_ID).to_string();
            if !HEAD_PRESETS.iter().any(|p| p.0 == hid) {
                hid = DEFAULT_HEAD_ID.into();
            }
            let mut gid = data.get("grid_id").and_then(|v| v.as_str()).unwrap_or(DEFAULT_GRID_ID).to_string();
            if !grid_presets().iter().any(|p| p.0 == gid) {
                gid = DEFAULT_GRID_ID.into();
            }
            let gc = match data.get("grid_custom") {
                Some(Value::Array(a)) if a.len() == 2 => [clamp_color(a.first(), DEFAULT_GRID_CUSTOM[0]), clamp_color(a.get(1), DEFAULT_GRID_CUSTOM[1])],
                _ => DEFAULT_GRID_CUSTOM,
            };
            *s = Some(State {
                head_id: hid,
                head_custom: match data.get("head_custom") {
                    None => DEFAULT_CUSTOM,
                    v => clamp_color(v, DEFAULT_CUSTOM),
                },
                grid_id: gid,
                grid_custom: gc,
                banner_on: data.get("banner_on").map(truthy).unwrap_or(true),
                banner_size: num(data.get("banner_size"), BANNER_SIZE_MIN, BANNER_SIZE_MAX, 1.0),
                banner_opacity: num(data.get("banner_opacity"), BANNER_OP_MIN, BANNER_OP_MAX, 1.0),
            });
        }
        f(s.as_mut().expect("ngb"))
    })
}

fn save() {
    let st = with_state(|s| s.clone());
    let mut m = Map::new();
    m.insert("head_id".into(), json!(st.head_id));
    m.insert("head_custom".into(), json!([st.head_custom.0, st.head_custom.1, st.head_custom.2]));
    m.insert("grid_id".into(), json!(st.grid_id));
    m.insert("grid_custom".into(), json!([[st.grid_custom[0].0, st.grid_custom[0].1, st.grid_custom[0].2], [st.grid_custom[1].0, st.grid_custom[1].1, st.grid_custom[1].2]]));
    m.insert("banner_on".into(), json!(st.banner_on));
    m.insert("banner_size".into(), json!(py::round_to(st.banner_size, 3)));
    m.insert("banner_opacity".into(), json!(py::round_to(st.banner_opacity, 3)));
    store::save_section("ngb", m);
}

fn head_preset_color(pid: &str) -> Option<Rgb> {
    HEAD_PRESETS.iter().find(|p| p.0 == pid).unwrap_or(&HEAD_PRESETS[0]).1
}

/// `get_head_id()`
pub fn get_head_id() -> String {
    with_state(|s| s.head_id.clone())
}

/// `set_head_id(pid)`
pub fn set_head_id(pid: &str) {
    if HEAD_PRESETS.iter().any(|p| p.0 == pid) {
        with_state(|s| s.head_id = pid.to_string());
        save();
    }
}

/// `get_custom()`
pub fn get_custom() -> Rgb {
    with_state(|s| s.head_custom)
}

/// `set_custom(color)`
pub fn set_custom(c: Rgb) {
    with_state(|s| s.head_custom = (c.0.clamp(0, 255), c.1.clamp(0, 255), c.2.clamp(0, 255)));
    save();
}

/// `head_color()` - aktive Kopffarbe (Spieler 1).
pub fn head_color() -> Rgb {
    with_state(|s| if s.head_id == "custom" { s.head_custom } else { head_preset_color(&s.head_id).unwrap_or(DEFAULT_CUSTOM) })
}

/// `get_grid_id()`
pub fn get_grid_id() -> String {
    with_state(|s| s.grid_id.clone())
}

/// `set_grid_id(pid)`
pub fn set_grid_id(pid: &str) {
    if grid_presets().iter().any(|p| p.0 == pid) {
        with_state(|s| s.grid_id = pid.to_string());
        save();
    }
}

/// `get_grid_custom()`
pub fn get_grid_custom() -> [Rgb; 2] {
    with_state(|s| s.grid_custom)
}

/// `set_grid_custom(colors)`
pub fn set_grid_custom(c: [Rgb; 2]) {
    let cl = |c: Rgb| (c.0.clamp(0, 255), c.1.clamp(0, 255), c.2.clamp(0, 255));
    with_state(|s| s.grid_custom = [cl(c[0]), cl(c[1])]);
    save();
}

/// `grid_sequence()` - Farbreihenfolge oder `None` (aus).
pub fn grid_sequence() -> Option<Vec<Rgb>> {
    with_state(|s| {
        if s.grid_id == "off" {
            return None;
        }
        if s.grid_id == "custom" {
            return Some(s.grid_custom.to_vec());
        }
        grid_presets().into_iter().find(|p| p.0 == s.grid_id).and_then(|p| p.1)
    })
}

/// Banner-Einstellungen.
#[derive(Clone, Copy, Debug)]
pub struct Banner {
    /// Sichtbar?
    pub on: bool,
    /// Groesse.
    pub size: f64,
    /// Deckkraft.
    pub opacity: f64,
}

/// `get_banner()`
pub fn get_banner() -> Banner {
    with_state(|s| Banner { on: s.banner_on, size: s.banner_size, opacity: s.banner_opacity })
}

fn set_banner_on(v: bool) {
    with_state(|s| s.banner_on = v);
    save();
}

fn set_banner_size(v: f64) {
    with_state(|s| s.banner_size = BANNER_SIZE_MIN.max(BANNER_SIZE_MAX.min(v)));
    save();
}

fn set_banner_opacity(v: f64) {
    with_state(|s| s.banner_opacity = BANNER_OP_MIN.max(BANNER_OP_MAX.min(v)));
    save();
}

// ===================================================== Menue

#[derive(Clone, Copy)]
struct Slider {
    track: Rect,
    minus: Rect,
    plus: Rect,
}

const CHANNELS: [&str; 3] = ["R", "G", "B"];

fn ch_col(ch: &str) -> Rgb {
    match ch {
        "R" => (235, 90, 90),
        "G" => (90, 210, 120),
        _ => (90, 150, 245),
    }
}

/// `PersonalizeMenu` - Kopf / Raster / Banner.
pub struct PersonalizeMenu {
    width: i32,
    height: i32,
    /// Soll das Menue geschlossen werden?
    pub done: bool,
    big: Font,
    font: Font,
    small: Font,
    tiny: Font,
    tab: &'static str,
    sel: String,
    custom: [i32; 3],
    grid_sel: String,
    grid_custom: [[i32; 3]; 2],
    grid_edit: usize,
    tab_rects: Vec<(&'static str, Rect)>,
    head_tiles: Vec<(Rect, usize)>,
    sliders_head: Vec<(&'static str, Slider)>,
    grid_tiles: Vec<(Rect, usize)>,
    grid_ab: Vec<(&'static str, Rect)>,
    sliders_grid: Vec<(&'static str, Slider)>,
    banner_toggle: Rect,
    banner_slider: Vec<(&'static str, Slider)>,
    done_rect: Rect,
    sounds: Vec<&'static str>,
}

/// `open_head_color_menu(width, height)`
pub fn open_head_color_menu(width: i32, height: i32) -> PersonalizeMenu {
    PersonalizeMenu::new(width, height)
}

impl PersonalizeMenu {
    fn new(width: i32, height: i32) -> PersonalizeMenu {
        achievements::event("painter", None);
        let c = get_custom();
        let g = get_grid_custom();
        let z = Rect::new(0, 0, 0, 0);
        let mut m = PersonalizeMenu {
            width,
            height,
            done: false,
            big: Font::sys("consolas", 32, true, false),
            font: Font::sys("consolas", 20, false, false),
            small: Font::sys("consolas", 15, false, false),
            tiny: Font::sys("consolas", 12, false, false),
            tab: "head",
            sel: get_head_id(),
            custom: [c.0, c.1, c.2],
            grid_sel: get_grid_id(),
            grid_custom: [[g[0].0, g[0].1, g[0].2], [g[1].0, g[1].1, g[1].2]],
            grid_edit: 0,
            tab_rects: Vec::new(),
            head_tiles: Vec::new(),
            sliders_head: Vec::new(),
            grid_tiles: Vec::new(),
            grid_ab: Vec::new(),
            sliders_grid: Vec::new(),
            banner_toggle: z,
            banner_slider: Vec::new(),
            done_rect: z,
            sounds: Vec::new(),
        };
        m.build_layout();
        m
    }

    /// Angeforderte Klaenge seit dem letzten Abholen (`play_sound`).
    pub fn take_sounds(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.sounds)
    }

    fn play(&mut self, name: &'static str) {
        self.sounds.push(name);
    }

    fn build_layout(&mut self) {
        let cx = self.width / 2;
        let (tabw, tgap) = (120, 8);
        let names = ["head", "grid", "banner"];
        let total = names.len() as i32 * tabw + (names.len() as i32 - 1) * tgap;
        let mut x = cx - total / 2;
        self.tab_rects.clear();
        for nm in names {
            self.tab_rects.push((nm, Rect::new(x, 46, tabw, 30)));
            x += tabw + tgap;
        }
        let (tw, th, gap) = (92, 46, 12);
        self.head_tiles.clear();
        let mut y = 116;
        for row in [0..4usize, 4..7usize] {
            let n = row.len() as i32;
            let total = n * tw + (n - 1) * gap;
            let mut x = cx - total / 2;
            for i in row {
                self.head_tiles.push((Rect::new(x, y, tw, th), i));
                x += tw + gap;
            }
            y += th + 18 + gap;
        }
        self.sliders_head = self.make_sliders(y + 4);
        let (gtw, gth, ggap) = (74, 46, 8);
        self.grid_tiles.clear();
        let n = grid_presets().len() as i32;
        let total = n * gtw + (n - 1) * ggap;
        let mut x = cx - total / 2;
        let gy = 116;
        for i in 0..n as usize {
            self.grid_tiles.push((Rect::new(x, gy, gtw, gth), i));
            x += gtw + ggap;
        }
        let aby = gy + gth + 22 + 12;
        self.grid_ab = vec![("A", Rect::new(cx - 104, aby, 92, 26)), ("B", Rect::new(cx + 12, aby, 92, 26))];
        self.sliders_grid = self.make_sliders(aby + 42);
        self.banner_toggle = Rect::new(cx - 150, 118, 300, 36);
        self.banner_slider = vec![("size", self.one_slider(200)), ("opacity", self.one_slider(258))];
        self.done_rect = Rect::new(cx - 80, self.height - 78, 160, 32);
    }

    fn one_slider(&self, top: i32) -> Slider {
        let cx = self.width / 2;
        let track = Rect::new(cx - 84, top, 168, 14);
        Slider { track, minus: Rect::new(track.x - 30, top - 4, 22, 22), plus: Rect::new(track.right() + 8, top - 4, 22, 22) }
    }

    fn make_sliders(&self, top: i32) -> Vec<(&'static str, Slider)> {
        let cx = self.width / 2;
        let mut sy = top;
        let mut d = Vec::new();
        for ch in CHANNELS {
            let track = Rect::new(cx - 100, sy, 200, 14);
            d.push((ch, Slider { track, minus: Rect::new(track.x - 30, sy - 4, 22, 22), plus: Rect::new(track.right() + 8, sy - 4, 22, 22) }));
            sy += 30;
        }
        d
    }

    fn active_sliders(&self) -> Vec<(&'static str, Slider)> {
        if self.tab == "head" {
            self.sliders_head.clone()
        } else {
            self.sliders_grid.clone()
        }
    }

    /// Die per RGB editierbare Farbe: 0 = Kopf, 1/2 = Raster A/B.
    fn editable(&self) -> Option<usize> {
        if self.tab == "head" && self.sel == "custom" {
            return Some(0);
        }
        if self.tab == "grid" && self.grid_sel == "custom" {
            return Some(1 + self.grid_edit);
        }
        None
    }

    fn ed_get(&self, ed: usize) -> [i32; 3] {
        if ed == 0 {
            self.custom
        } else {
            self.grid_custom[ed - 1]
        }
    }

    fn ed_set(&mut self, ed: usize, ch: usize, v: i32) {
        if ed == 0 {
            self.custom[ch] = v;
        } else {
            self.grid_custom[ed - 1][ch] = v;
        }
    }

    fn persist(&mut self) {
        if self.tab == "head" {
            set_custom((self.custom[0], self.custom[1], self.custom[2]));
        } else {
            let g = self.grid_custom;
            set_grid_custom([(g[0][0], g[0][1], g[0][2]), (g[1][0], g[1][1], g[1][2])]);
        }
    }

    fn head_color(&self) -> Rgb {
        if self.sel == "custom" {
            (self.custom[0], self.custom[1], self.custom[2])
        } else {
            head_preset_color(&self.sel).unwrap_or(DEFAULT_CUSTOM)
        }
    }

    fn slider_set(&mut self, ed: usize, ch: &str, value: i64) {
        let i = CHANNELS.iter().position(|c| *c == ch).unwrap_or(0);
        self.ed_set(ed, i, value.clamp(0, 255) as i32);
        self.persist();
    }

    fn step(&mut self, ed: usize, ch: &str, delta: i64) {
        let i = CHANNELS.iter().position(|c| *c == ch).unwrap_or(0);
        let v = self.ed_get(ed)[i] as i64 + delta;
        self.slider_set(ed, ch, v);
        self.play("click");
    }

    fn value_from_x(track: Rect, x: i32) -> i64 {
        py::round((x - track.x) as f64 / 1.max(track.w) as f64 * 255.0)
    }

    fn close(&mut self) {
        self.done = true;
        self.play("click");
    }

    fn range_from_x(track: Rect, x: i32, lo: f64, hi: f64) -> f64 {
        let frac = 0f64.max(1f64.min((x - track.x) as f64 / 1.max(track.w) as f64));
        lo + frac * (hi - lo)
    }

    fn banner_range(kind: &str) -> (f64, f64) {
        if kind == "size" {
            (BANNER_SIZE_MIN, BANNER_SIZE_MAX)
        } else {
            (BANNER_OP_MIN, BANNER_OP_MAX)
        }
    }

    fn set_banner(kind: &str, v: f64) {
        if kind == "size" {
            set_banner_size(v);
        } else {
            set_banner_opacity(v);
        }
    }

    fn on_click_banner(&mut self, p: (i32, i32)) -> bool {
        if self.banner_toggle.hit(p) {
            set_banner_on(!get_banner().on);
            self.play("select");
            return true;
        }
        let cfg = get_banner();
        for (kind, sl) in self.banner_slider.clone() {
            let (lo, hi) = Self::banner_range(kind);
            let cur = if kind == "size" { cfg.size } else { cfg.opacity };
            if sl.track.hit(p) {
                Self::set_banner(kind, Self::range_from_x(sl.track, p.0, lo, hi));
                self.play("click");
                return true;
            }
            if sl.minus.hit(p) {
                Self::set_banner(kind, cur - 0.1);
                self.play("click");
                return true;
            }
            if sl.plus.hit(p) {
                Self::set_banner(kind, cur + 0.1);
                self.play("click");
                return true;
            }
        }
        false
    }

    /// `handle_event(event)`
    pub fn handle_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if ["Escape", "Return", "space"].contains(&k) {
                self.close();
            } else if k == "Tab" {
                self.tab = if self.tab == "head" { "grid" } else { "head" };
                self.play("click");
            } else if k.len() == 1 && "1234567".contains(k) {
                let idx = k.parse::<usize>().unwrap_or(1) - 1;
                if self.tab == "head" && idx < HEAD_PRESETS.len() {
                    self.select_head(HEAD_PRESETS[idx].0);
                } else if self.tab == "grid" && idx < grid_presets().len() {
                    self.select_grid(grid_presets()[idx].0);
                }
            }
            return;
        }
        if ev.kind == Kind::MouseDown {
            self.on_click(ev.pos);
        }
    }

    fn select_head(&mut self, pid: &str) {
        self.sel = pid.to_string();
        set_head_id(pid);
        self.play("select");
    }

    fn select_grid(&mut self, pid: &str) {
        self.grid_sel = pid.to_string();
        set_grid_id(pid);
        self.play("select");
    }

    fn on_click(&mut self, p: (i32, i32)) {
        if let Some(&(name, _)) = self.tab_rects.iter().find(|(_, r)| r.hit(p)) {
            self.tab = name;
            self.play("click");
            return;
        }
        if self.tab == "head" {
            if let Some(&(_, i)) = self.head_tiles.iter().find(|(r, _)| r.hit(p)) {
                self.select_head(HEAD_PRESETS[i].0);
                return;
            }
        } else if self.tab == "grid" {
            if let Some(&(_, i)) = self.grid_tiles.iter().find(|(r, _)| r.hit(p)) {
                self.select_grid(grid_presets()[i].0);
                return;
            }
            if self.grid_sel == "custom" {
                if let Some(&(k, _)) = self.grid_ab.iter().find(|(_, r)| r.hit(p)) {
                    self.grid_edit = if k == "A" { 0 } else { 1 };
                    self.play("click");
                    return;
                }
            }
        } else if self.tab == "banner" && self.on_click_banner(p) {
            return;
        }
        if let Some(ed) = self.editable() {
            for (ch, sl) in self.active_sliders() {
                if sl.track.hit(p) {
                    let v = Self::value_from_x(sl.track, p.0);
                    self.slider_set(ed, ch, v);
                    self.play("click");
                    return;
                }
                if sl.minus.hit(p) {
                    self.step(ed, ch, -8);
                    return;
                }
                if sl.plus.hit(p) {
                    self.step(ed, ch, 8);
                    return;
                }
            }
        }
        if self.done_rect.hit(p) {
            self.close();
        }
    }

    /// `draw(s)`
    pub fn draw(&mut self, s: &mut Surface) {
        s.fill_c(COL_BG);
        let cx = self.width / 2;
        let title = self.big.render(&i18n::t("ngb.title"), true, COL_TEXT.c());
        s.blr(&title, title.get_rect().midtop_at((cx, 12)));
        let hc = self.head_color();
        Self::draw_head_preview(s, self.width - 44, 30, hc);
        self.draw_tabs(s);
        match self.tab {
            "head" => self.draw_head_page(s),
            "grid" => self.draw_grid_page(s),
            _ => self.draw_banner_page(s),
        }
        draw::rect_r(s, COL_BTN_ON, self.done_rect, 0, 10);
        let dt = self.font.render(&i18n::t("ngb.done"), true, COL_TEXT.c());
        s.blr(&dt, dt.get_rect().center_at(self.done_rect.center()));
        let sub = self.tiny.render(&i18n::t("ngb.subtitle"), true, COL_DIM.c());
        s.blr(&sub, sub.get_rect().midbottom_at((cx, self.height - 34)));
        let foot = self.tiny.render(&i18n::t("ngb.hint"), true, COL_DIM.c());
        s.blr(&foot, foot.get_rect().midbottom_at((cx, self.height - 16)));
    }

    fn draw_tabs(&self, s: &mut Surface) {
        for (name, r) in &self.tab_rects {
            let active = *name == self.tab;
            draw::rect_r(s, if active { COL_ACCENT } else { COL_TAB_OFF }, *r, 0, 8);
            draw::rect_r(s, if active { COL_SEL } else { COL_DIM }, *r, 1, 8);
            let lab = self.small.render(&i18n::t(&format!("ngb.tab_{name}")), true, if active { COL_BG.c() } else { COL_DIM.c() });
            s.blr(&lab, lab.get_rect().center_at(r.center()));
        }
    }

    fn draw_head_page(&self, s: &mut Surface) {
        let label = self.small.render(&i18n::t("ngb.head_label"), true, COL_ACCENT.c());
        let r0 = self.head_tiles[0].0;
        s.bl(&label, (r0.x, r0.y - 22));
        for (rect, i) in &self.head_tiles {
            let (pid, col) = HEAD_PRESETS[*i];
            let col = if pid == "custom" { (self.custom[0], self.custom[1], self.custom[2]) } else { col.unwrap_or(DEFAULT_CUSTOM) };
            self.draw_tile(s, *rect, col, pid == self.sel, &i18n::t(&format!("ngb.head.{pid}")));
        }
        if self.sel == "custom" {
            self.draw_sliders(s);
        } else {
            let hint = self.small.render(&i18n::t("ngb.custom_pick"), true, COL_DIM.c());
            let last = self.head_tiles[self.head_tiles.len() - 1].0;
            s.blr(&hint, hint.get_rect().midtop_at((self.width / 2, last.bottom() + 30)));
        }
    }

    fn draw_grid_page(&self, s: &mut Surface) {
        let label = self.small.render(&i18n::t("ngb.grid_label"), true, COL_ACCENT.c());
        let r0 = self.grid_tiles[0].0;
        s.bl(&label, (r0.x, r0.y - 22));
        let presets = grid_presets();
        for (rect, i) in &self.grid_tiles {
            let (pid, seq) = &presets[*i];
            let seq: Option<Vec<Rgb>> = if *pid == "custom" { Some(self.grid_custom.iter().map(|c| (c[0], c[1], c[2])).collect()) } else { seq.clone() };
            let selected = *pid == self.grid_sel;
            self.draw_grid_swatch(s, *rect, seq.as_deref());
            if selected {
                draw::rect_r(s, COL_SEL, rect.inflate(8, 8), 2, 10);
            }
            let name = self.tiny.render(&i18n::t(&format!("ngb.grid.{pid}")), true, if selected { COL_TEXT.c() } else { COL_DIM.c() });
            s.blr(&name, name.get_rect().midtop_at((rect.centerx(), rect.bottom() + 4)));
        }
        if self.grid_sel == "custom" {
            for (k, r) in &self.grid_ab {
                let idx = if *k == "A" { 0 } else { 1 };
                let c = self.grid_custom[idx];
                draw::rect_r(s, (c[0], c[1], c[2]), *r, 0, 7);
                if self.grid_edit == idx {
                    draw::rect_r(s, COL_SEL, *r, 2, 7);
                } else {
                    draw::rect_r(s, COL_DIM, *r, 1, 7);
                }
                let lab = self.small.render(k, true, COL_TEXT.c());
                s.blr(&lab, lab.get_rect().center_at(r.center()));
            }
            self.draw_sliders(s);
        } else {
            let hint = self.small.render(&i18n::t("ngb.grid_pick"), true, COL_DIM.c());
            s.blr(&hint, hint.get_rect().midtop_at((self.width / 2, r0.bottom() + 34)));
        }
    }

    fn draw_banner_page(&self, s: &mut Surface) {
        let cx = self.width / 2;
        let cfg = get_banner();
        let on = cfg.on;
        let label = self.small.render(&i18n::t("ngb.banner_label"), true, COL_ACCENT.c());
        s.bl(&label, (self.banner_toggle.x, self.banner_toggle.y - 22));
        draw::rect_r(s, if on { COL_BTN_ON } else { COL_BTN }, self.banner_toggle, 0, 8);
        draw::rect_r(s, if on { COL_SEL } else { COL_DIM }, self.banner_toggle, 1, 8);
        let tl = self.font.render(&i18n::t("ngb.banner_toggle"), true, COL_TEXT.c());
        s.bl(&tl, (self.banner_toggle.x + 16, self.banner_toggle.centery() - tl.h() / 2));
        let stat = i18n::t(if on { "common.on" } else { "common.off" });
        let col: Rgb = if on { (150, 235, 150) } else { COL_DIM };
        let stimg = self.font.render(&format!("< {stat} >"), true, col.c());
        s.bl(&stimg, (self.banner_toggle.right() - stimg.w() - 16, self.banner_toggle.centery() - stimg.h() / 2));
        self.draw_banner_slider(s, "size", cfg.size, on);
        self.draw_banner_slider(s, "opacity", cfg.opacity, on);
        if on {
            self.draw_banner_preview(s, cx, 344, cfg);
        } else {
            let hint = self.small.render(&i18n::t("ngb.banner_off_hint"), true, COL_DIM.c());
            s.blr(&hint, hint.get_rect().center_at((cx, 330)));
        }
    }

    fn draw_banner_slider(&self, s: &mut Surface, kind: &str, val: f64, enabled: bool) {
        let sl = self.banner_slider.iter().find(|(k, _)| *k == kind).map(|(_, s)| *s).expect("slider");
        let track = sl.track;
        let (lo, hi) = Self::banner_range(kind);
        let frac = (val - lo) / (hi - lo);
        let base: Rgb = if kind == "size" { COL_ACCENT } else { (150, 200, 120) };
        let col = if enabled { base } else { COL_DIM };
        let lab = self.small.render(&i18n::t(&format!("ngb.banner_{kind}")), true, col.c());
        s.blr(&lab, lab.get_rect().midright_at((sl.minus.x - 8, track.centery())));
        draw::rect_r(s, COL_TRACK, track, 0, 7);
        draw::rect_r(s, col, Rect::new(track.x, track.y, (track.w as f64 * frac) as i32, track.h), 0, 7);
        let knob = track.x + (track.w as f64 * frac) as i32;
        draw::circle(s, if enabled { COL_SEL } else { COL_DIM }, (knob, track.centery()), 7, 0);
        draw::circle(s, col, (knob, track.centery()), 5, 0);
        for (r, sym) in [(sl.minus, "-"), (sl.plus, "+")] {
            draw::rect_r(s, COL_BTN, r, 0, 5);
            let g = self.font.render(sym, true, if enabled { COL_TEXT.c() } else { COL_DIM.c() });
            s.blr(&g, g.get_rect().center_at(r.center()));
        }
        let pct = self.small.render(&format!("{}%", py::round(val * 100.0)), true, if enabled { COL_TEXT.c() } else { COL_DIM.c() });
        s.blr(&pct, pct.get_rect().midleft_at((sl.plus.right() + 8, track.centery())));
    }

    fn draw_banner_preview(&self, s: &mut Surface, cx: i32, cy: i32, cfg: Banner) {
        let big = self.big.render("\u{00D7}1.4", true, rgb(150, 235, 150));
        let sub = self.tiny.render(&i18n::t("snake.purple_banner"), true, COL_DIM.c());
        let pad = 14;
        let tw = big.w().max(sub.w());
        let th = big.h() + sub.h() + 4;
        let (pw, ph) = (tw + pad * 2, th + pad);
        let mut panel = pg::surface_alpha(pw, ph);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(18, 20, 32, 215), pr, 0, 12);
        draw::rect_r(&mut panel, rgba(150, 235, 150, 255), pr, 2, 12);
        panel.blr(&sub, sub.get_rect().midtop_at((pw / 2, 5)));
        panel.blr(&big, big.get_rect().midtop_at((pw / 2, 5 + sub.h() + 2)));
        if (cfg.size - 1.0).abs() > 0.01 {
            panel = transform::rotozoom(&panel, 0.0, cfg.size);
        }
        panel.fill_f(rgba(255, 255, 255, (255.0 * cfg.opacity) as i32), BLEND_RGBA_MULT);
        s.blr(&panel, panel.get_rect().center_at((cx, cy)));
    }

    fn draw_tile(&self, s: &mut Surface, rect: Rect, col: Rgb, selected: bool, name: &str) {
        draw::rect_r(s, col, rect, 0, 8);
        draw::rect_r(s, (255.min(col.0 + 50), 255.min(col.1 + 50), 255.min(col.2 + 50)), rect, 1, 8);
        if selected {
            draw::rect_r(s, COL_SEL, rect.inflate(8, 8), 2, 10);
        }
        let lab = self.tiny.render(name, true, if selected { COL_TEXT.c() } else { COL_DIM.c() });
        s.blr(&lab, lab.get_rect().midtop_at((rect.centerx(), rect.bottom() + 4)));
    }

    fn draw_grid_swatch(&self, s: &mut Surface, rect: Rect, seq: Option<&[Rgb]>) {
        let Some(seq) = seq.filter(|q| !q.is_empty()) else {
            draw::rect_r(s, (30, 32, 42), rect, 0, 8);
            draw::rect_r(s, COL_DIM, rect, 1, 8);
            let x = self.small.render("x", true, COL_DIM.c());
            s.blr(&x, x.get_rect().center_at(rect.center()));
            return;
        };
        let n = seq.len();
        let rows = 4;
        let rh = rect.h as f64 / rows as f64;
        for j in 0..rows {
            let col = seq[j % n];
            let r = Rect::new(rect.x, (rect.y as f64 + j as f64 * rh) as i32, rect.w, rh as i32 + 1);
            draw::rect(s, col, r, 0);
        }
        let hint = self.tiny.render("1a", true, rgb(232, 234, 240));
        s.blr(&hint, hint.get_rect().center_at(rect.center()));
        let c0 = seq[0];
        draw::rect_r(s, (255.min(c0.0 + 40), 255.min(c0.1 + 40), 255.min(c0.2 + 40)), rect, 1, 4);
    }

    fn draw_sliders(&self, s: &mut Surface) {
        let Some(ed) = self.editable() else {
            return;
        };
        let vals = self.ed_get(ed);
        for (i, (ch, sl)) in self.active_sliders().into_iter().enumerate() {
            let track = sl.track;
            let val = vals[i];
            draw::rect_r(s, COL_TRACK, track, 0, 7);
            let fill = Rect::new(track.x, track.y, track.w * val / 255, track.h);
            draw::rect_r(s, ch_col(ch), fill, 0, 7);
            let knob_x = track.x + track.w * val / 255;
            draw::circle(s, COL_SEL, (knob_x, track.centery()), 7, 0);
            draw::circle(s, ch_col(ch), (knob_x, track.centery()), 5, 0);
            let lab = self.small.render(ch, true, ch_col(ch).c());
            s.blr(&lab, lab.get_rect().midright_at((sl.minus.x - 6, track.centery())));
            for (r, sym) in [(sl.minus, "-"), (sl.plus, "+")] {
                draw::rect_r(s, COL_BTN, r, 0, 5);
                let g = self.font.render(sym, true, COL_TEXT.c());
                s.blr(&g, g.get_rect().center_at(r.center()));
            }
            let numi = self.small.render(&val.to_string(), true, COL_TEXT.c());
            s.blr(&numi, numi.get_rect().midleft_at((sl.plus.right() + 8, track.centery())));
        }
    }

    fn draw_head_preview(s: &mut Surface, cx: i32, cy: i32, color: Rgb) {
        let r = Rect::new(0, 0, 30, 30).center_at((cx, cy));
        draw::rect_r(s, color, r, 0, 8);
        for sign in [-1, 1] {
            let ex = cx + sign * 6;
            draw::circle(s, (250, 250, 250), (ex, cy - 2), 3, 0);
            draw::circle(s, (20, 20, 30), (ex + 1, cy - 1), 1, 0);
        }
    }
}
