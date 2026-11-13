//! geodash_edit.rs
//! ===============
//! Eigene Geometry-Dash-Level: der LEVELS-Reiter ([`LevelList`]) und der
//! Level-Editor ([`LevelEditor`]) - uebertragen aus `games/geodash_edit.py`.
//!
//! Was Python ueber Rueckrufe ins Spiel erledigt (Editor oeffnen/schliessen,
//! Level spielen, Test-Spielen), liefern die Rust-Fassungen als
//! [`GdAction`] zurueck; das Spiel fuehrt sie danach aus.

use std::collections::HashMap;
use std::f64::consts::PI;

use pgz_core::swear;
use pgz_engine::filepick;
use pgz_engine::pg::surface_alpha;
use pgz_engine::prelude::*;
use pgz_engine::ui::TextInput;
use serde_json::{json, Map, Value};

use crate::geodash_core::{self as core, Level, Obj};
use crate::geodash_draw::{self as gdraw, Renderer};
use crate::geodash_music as music;
use crate::ugc;

/// Art der eigenen Level in `ugc.json`.
pub const GAME: &str = "geodash";

/// Was das Spiel nach einer Eingabe tun soll.
pub enum GdAction {
    /// Nichts.
    None,
    /// Neues Level (leer oder aus einer Vorlage) und in den Editor.
    NewLevel(Option<Map<String, Value>>),
    /// Dieses Level bearbeiten.
    Edit(Map<String, Value>),
    /// Aus der Bibliothek kopiert: Listen neu laden, dann bearbeiten.
    EditCopied(Map<String, Value>),
    /// Eigenes Level spielen.
    Play(Map<String, Value>),
    /// Level der Bibliothek spielen.
    PlayExtra(Map<String, Value>),
    /// Level ausprobieren (ab Start oder ab Block).
    Test(Map<String, Value>, Option<i64>),
    /// Editor schliessen.
    Close,
}

/// Bestwerte `{key: [normal, uebung]}` in Einfuege-Reihenfolge.
pub type Best = Vec<(String, [i64; 2])>;
/// Zahlen je Schluessel (Muenzen, Versuche) in Einfuege-Reihenfolge.
pub type Counts = Vec<(String, i64)>;

/// `best.get(key, [0, 0])`
pub fn best_get(best: &Best, key: &str) -> [i64; 2] {
    best.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or([0, 0])
}

/// `counts.get(key, 0)`
pub fn count_get(c: &Counts, key: &str) -> i64 {
    c.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(0)
}

/// `counts[key] = v`
pub fn count_set(c: &mut Counts, key: &str, v: i64) {
    match c.iter_mut().find(|e| e.0 == key) {
        Some(e) => e.1 = v,
        None => c.push((key.to_string(), v)),
    }
}

// ---------------------------------------------------------------------------
//  Palette
// ---------------------------------------------------------------------------

const TOOLS: [&str; 2] = ["select", "erase"];
const GROUPS: [(&str, &[&str]); 6] = [
    ("blocks", &["block", "half"]),
    ("hazards", &["spike", "spike_s", "pit"]),
    ("orbs", &["pad_y", "pad_p", "pad_b", "orb_y", "orb_p", "orb_b"]),
    ("portals", &["p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm", "g_flip"]),
    ("speed", &["s_slow", "s_norm", "s_fast", "s_vfast"]),
    ("extras", &["coin", "color"]),
];

fn group_items(key: &str) -> &'static [&'static str] {
    GROUPS.iter().find(|g| g.0 == key).map(|g| g.1).unwrap_or(&[])
}

fn group_icon(key: &str) -> &'static str {
    match key {
        "blocks" => "block",
        "hazards" => "spike",
        "orbs" => "orb_y",
        "portals" => "p_ship",
        "speed" => "s_fast",
        _ => "coin",
    }
}

/// Farbvorlagen fuer Hintergrund, Boden und Farb-Trigger.
pub const PRESETS: [(i64, i64, i64); 12] = [
    (40, 110, 255),
    (20, 70, 200),
    (170, 60, 255),
    (255, 60, 150),
    (230, 90, 40),
    (250, 200, 40),
    (40, 170, 110),
    (0, 170, 200),
    (140, 0, 40),
    (30, 30, 50),
    (120, 120, 140),
    (255, 255, 255),
];

fn param_labels(kind: &str) -> &'static [&'static str] {
    match kind {
        "p_ship" | "p_ball" | "p_ufo" | "p_wave" => &["gd.ed.par.height"],
        "color" => &["gd.ed.par.target", "R", "G", "B", "gd.ed.par.dur"],
        _ => &[],
    }
}

const UNDO_MAX: usize = 60;
const LAYER_PIT: i64 = 1;
const LAYER_TRIGGER: i64 = 2;
const LAYER_MAIN: i64 = 0;

fn layer_of(kind: &str) -> i64 {
    match kind {
        "pit" => LAYER_PIT,
        "color" => LAYER_TRIGGER,
        _ => LAYER_MAIN,
    }
}

fn s_of<'a>(m: &'a Map<String, Value>, key: &str) -> &'a str {
    m.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

fn rgb_of(v: Option<&Value>, def: [i64; 3]) -> Color {
    match v.and_then(|x| x.as_array()) {
        Some(a) => {
            let g = |i: usize| a.get(i).and_then(|x| x.as_i64()).unwrap_or(0);
            rgb(g(0), g(1), g(2))
        }
        None => rgb(def[0], def[1], def[2]),
    }
}

fn chars_take(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `new_level(name, level_id, author, template)` - frisches Level.
pub fn new_level(name: &str, level_id: &str, author: Option<&str>, template: Option<&Map<String, Value>>) -> Map<String, Value> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let mut m = Map::new();
    m.insert("v".into(), json!(1));
    m.insert("id".into(), json!(level_id));
    m.insert("name".into(), json!(name));
    m.insert("author".into(), json!(match author {
        Some(a) => a.to_string(),
        None => ugc::last_author(),
    }));
    m.insert("created".into(), json!(now));
    m.insert("edited".into(), json!(now));
    m.insert("speed".into(), json!(1));
    m.insert("mode".into(), json!(core::CUBE));
    m.insert("bg".into(), json!(core::DEFAULT_BG));
    m.insert("ground".into(), json!(core::DEFAULT_GROUND));
    m.insert("music".into(), json!("drive"));
    m.insert("objects".into(), json!([]));
    if let Some(tpl) = template {
        for key in ["speed", "mode", "bg", "ground", "music", "length"] {
            if let Some(v) = tpl.get(key) {
                m.insert(key.into(), v.clone());
            }
        }
        m.insert("objects".into(), tpl.get("objects").cloned().filter(|v| v.is_array()).unwrap_or(json!([])));
        m.insert("from".into(), json!(s_of(tpl, "id")));
        if s_of(&m, "name").is_empty() {
            let tid = s_of(tpl, "tpl");
            let nm = if !tid.is_empty() { t(&format!("gd.tpl.{tid}")) } else { s_of(tpl, "name").to_string() };
            m.insert("name".into(), json!(chars_take(&nm, ugc::MAX_NAME)));
        }
        if s_of(&m, "id").is_empty() {
            let sl = ugc::slug(s_of(&m, "name"));
            let id = ugc::unique_id(if sl.is_empty() { "level" } else { &sl }, GAME, None);
            m.insert("id".into(), json!(id));
        }
    }
    let mut m = core::normalize_level(&m);
    if let Some(tpl) = template {
        if truthy(tpl.get("verified")) {
            let h = core::content_hash(&m);
            m.insert("verified".into(), json!(h));
        }
    }
    m
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

/// `is_verified(m)` - traegt das Level ein gueltiges Abzeichen?
pub fn is_verified(m: &Map<String, Value>) -> bool {
    match m.get("verified") {
        Some(Value::String(v)) => !v.is_empty() && *v == core::content_hash(m),
        _ => false,
    }
}

/// `default_param(kind)`
pub fn default_param(kind: &str) -> Vec<i64> {
    core::params(kind).iter().map(|p| p.0).collect()
}

// ---------------------------------------------------------------------------
//  Zeichenhelfer
// ---------------------------------------------------------------------------

/// Text kuerzen, bis er mit `suffix` in `room` passt.
pub fn kurz(fnt: &Font, text: &str, room: i32, min: usize, suffix: &str) -> String {
    let mut k: Vec<char> = text.chars().collect();
    while k.len() > min && fnt.size(&(k.iter().collect::<String>() + suffix)).0 > room {
        k.pop();
    }
    k.into_iter().collect::<String>() + suffix
}

/// `_btn(s, rc, text, fnt, on, accent, enabled)`
pub fn btn(s: &mut Surface, rc: Rect, text: &str, fnt: &Font, on: bool, accent: Option<Color>, enabled: bool) -> Rect {
    let p = ui::pal();
    draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 7);
    draw::rect_r(s, if on { accent.unwrap_or(p.accent) } else { p.border }, rc, if on { 2 } else { 1 }, 7);
    let col = if enabled || on { p.text } else { p.text_faint };
    let mut img = fnt.render(text, true, col);
    if img.w() > rc.w - 8 {
        img = fnt.render(&kurz(fnt, text, rc.w - 8, 1, "..."), true, col);
    }
    s.blr(&img, img.get_rect().center_at(rc.center()));
    rc
}

fn icon(s: &mut Surface, rc: Rect, key: &str, col: Color) {
    let (cx, cy) = rc.center();
    let (cxf, cyf) = (cx as f64, cy as f64);
    let r = 4.max(rc.w.min(rc.h) / 4);
    let rf = r as f64;
    let lw = 2;
    match key {
        "undo" | "redo" => {
            let sign = if key == "undo" { -1 } else { 1 };
            let bx = rect(0, 0, r * 2, r * 2).center_at((cx, cy + r / 3));
            draw::arc(s, col, bx, 0.3, 2.9, lw);
            let tip = ((cx - sign * r) as f64, (cy + r / 3) as f64);
            draw::polygon(s, col, &[(tip.0 - rf * 0.5, tip.1 - 1.0), (tip.0 + rf * 0.5, tip.1 - 1.0), (tip.0, tip.1 + rf * 0.6)], 0);
        }
        "rotate" => {
            let bx = rect(0, 0, r * 2, r * 2).center_at((cx, cy));
            draw::arc(s, col, bx, -0.4, 4.2, lw);
            draw::polygon(s, col, &[(cxf + rf, cyf - rf * 0.2), (cxf + rf * 0.3, cyf - rf * 0.2), (cxf + rf * 0.9, cyf + rf * 0.6)], 0);
        }
        "delete" => {
            draw::line(s, col, (cx - r, cy - r), (cx + r, cy + r), lw + 1);
            draw::line(s, col, (cx + r, cy - r), (cx - r, cy + r), lw + 1);
        }
        "grid" => {
            let bx = rect(0, 0, r * 2, r * 2).center_at((cx, cy));
            draw::rect(s, col, bx, 1);
            draw::line(s, col, (bx.centerx(), bx.y), (bx.centerx(), bx.bottom()), 1);
            draw::line(s, col, (bx.x, bx.centery()), (bx.right(), bx.centery()), 1);
        }
        "settings" => {
            draw::circle(s, col, (cx, cy), r, lw);
            for k in 0..6 {
                let a = k as f64 * PI / 3.0;
                draw::line(s, col, (cxf + a.cos() * rf, cyf + a.sin() * rf), (cxf + a.cos() * (rf + rf * 0.55), cyf + a.sin() * (rf + rf * 0.55)), lw + 1);
            }
            draw::circle(s, col, (cx, cy), 1.max(r / 3), 0);
        }
        "select" => {
            draw::polygon(s, col, &[(cxf - rf, cyf - rf), (cxf + rf, cyf), (cxf, cyf + rf * 0.3), (cxf + rf * 0.3, cyf + rf)], 2);
        }
        "erase" => {
            let bx = rect(0, 0, r * 2, (rf * 1.5) as i32).center_at((cx, cy));
            draw::rect_r(s, col, bx, 2, 2);
            draw::line(s, col, (bx.x, bx.bottom()), (bx.right(), bx.y), 2);
        }
        "test" => {
            draw::polygon(s, col, &[(cxf - rf * 0.7, cyf - rf), (cxf + rf, cyf), (cxf - rf * 0.7, cyf + rf)], 0);
        }
        "test_here" => {
            draw::polygon(s, col, &[(cxf - rf * 0.2, cyf - rf), (cxf + rf * 1.2, cyf), (cxf - rf * 0.2, cyf + rf)], 0);
            draw::line(s, col, (cx - r, cy - r), (cx - r, cy + r), 2);
        }
        _ => {}
    }
}

/// Gruenes Verifiziert-Haekchen.
pub fn check_badge(s: &mut Surface, cx: f64, cy: f64, r: f64) {
    draw::circle(s, (60, 200, 110), (cx as i32, cy as i32), r as i32, 0);
    draw::lines(s, (255, 255, 255), false, &[(cx - r * 0.5, cy), (cx - r * 0.1, cy + r * 0.4), (cx + r * 0.55, cy - r * 0.45)], 2.max(py::ffdiv(r, 3.0) as i32));
}

/// Toast-Plakette (LEVELS/WEITERE).
pub fn draw_list_toast(s: &mut Surface, fnt: &Font, text: &str, width: i32, list_bottom: i32) {
    let p = ui::pal();
    let img = fnt.render(text, true, p.text);
    let (w, h) = (img.w() + 26, img.h() + 12);
    let r = rect(width / 2 - w / 2, list_bottom - h - 2, w, h);
    ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
    draw::rect_r(s, p.accent2, r, 1, h / 2);
    s.blr(&img, img.get_rect().center_at(r.center()));
}

// ---------------------------------------------------------------------------
//  LEVELS-Reiter
// ---------------------------------------------------------------------------

const LIST_BUTTONS: [&str; 6] = ["new", "edit", "play", "delete", "share", "import"];
const NEEDS_SEL: [&str; 4] = ["edit", "play", "delete", "share"];

/// Der LEVELS-Reiter: eigene Level auswaehlen, bauen, spielen, teilen.
pub struct LevelList {
    /// Eigene Level.
    pub items: Vec<Map<String, Value>>,
    /// Auswahl.
    pub sel: usize,
    first: usize,
    toast: String,
    toast_t: f64,
    confirm: String,
    /// Teilen-Dialog (offen, wenn Some).
    pub share: Option<Map<String, Value>>,
    f_author: TextInput,
    f_file: TextInput,
    focus: usize,
    err: String,
    /// Vorlagen-Auswahl.
    pub picker: Option<Vec<Map<String, Value>>>,
    pick_sel: usize,
    fnt: Font,
    tiny: Font,
    two_lines: bool,
    row_h: i32,
    /// Knoepfe.
    pub btn_rects: Vec<(&'static str, Rect)>,
    list_top: i32,
    list_bottom: i32,
    rows_visible: usize,
    list_w: i32,
    share_rect: Rect,
    author_rect: Rect,
    file_rect: Rect,
    /// Knoepfe im Teilen-Dialog.
    pub share_btn: Vec<(&'static str, Rect)>,
    width: i32,
    height: i32,
}

impl LevelList {
    /// `LevelList(game)`
    pub fn new(b: &GameBase, tab_bottom: i32) -> LevelList {
        let z = Rect::new(0, 0, 0, 0);
        let mut ml = LevelList {
            items: Vec::new(),
            sel: 0,
            first: 0,
            toast: String::new(),
            toast_t: 0.0,
            confirm: String::new(),
            share: None,
            f_author: TextInput::new("", ugc::MAX_AUTHOR, None, ""),
            f_file: TextInput::new("", ugc::MAX_ID, Some(TextInput::ID_CHARS), ""),
            focus: 0,
            err: String::new(),
            picker: None,
            pick_sel: 0,
            fnt: b.font.clone(),
            tiny: b.font.clone(),
            two_lines: true,
            row_h: 26,
            btn_rects: Vec::new(),
            list_top: 0,
            list_bottom: 0,
            rows_visible: 1,
            list_w: 0,
            share_rect: z,
            author_rect: z,
            file_rect: z,
            share_btn: Vec::new(),
            width: b.width,
            height: b.height,
        };
        ml.reload();
        ml.layout(b, tab_bottom);
        ml
    }

    /// `reload()`
    pub fn reload(&mut self) {
        self.items = ugc::load_maps(GAME);
        self.sel = 0.max((self.items.len() as i64 - 1).min(self.sel as i64)) as usize;
        self.clamp();
    }

    fn selected(&self) -> Option<&Map<String, Value>> {
        self.items.get(self.sel)
    }

    /// `layout()`
    pub fn layout(&mut self, b: &GameBase, tab_bottom: i32) {
        let (w, h) = (b.width, b.height);
        self.width = w;
        self.height = h;
        self.fnt = ui::font(13.max(h / 30), false, false);
        self.tiny = ui::font(10.max(h / 40), false, false);
        let (fh, th) = (self.fnt.get_height(), self.tiny.get_height());
        self.two_lines = h >= 400;
        self.row_h = if self.two_lines { fh + th + 10 } else { 26.max(fh + 12) };
        let cols = if w >= 720 { 6 } else { 3 };
        let bh = 24.max(34.min(h / 13)).max(th + 12);
        let gap = 6;
        let rows = (LIST_BUTTONS.len() as i32 + cols - 1) / cols;
        let bw = (w - 24 - gap * (cols - 1)) as f64 / cols as f64;
        let bottom = h - 12;
        self.btn_rects.clear();
        for (i, key) in LIST_BUTTONS.iter().enumerate() {
            let (r, c) = (i as i32 / cols, i as i32 % cols);
            let y = bottom - (rows - r) * (bh + gap) + gap;
            self.btn_rects.push((key, Rect::new((12.0 + c as f64 * (bw + gap as f64)) as i32, y, bw as i32, bh)));
        }
        self.list_top = tab_bottom + 8;
        self.list_bottom = bottom - rows * (bh + gap) - th - 6;
        self.rows_visible = 1.max(py::fdiv32(self.list_bottom - self.list_top - 4, self.row_h)) as usize;
        self.list_w = w - 30;
        self.clamp();
        let pw = (w - 40).min(380.max((w as f64 * 0.5) as i32));
        let ff = 22.max(30.min(h / 14)).max(th + 10);
        let head = fh + 18;
        let lab = th + 4;
        let ph = (h - 20).min(head + 2 * lab + 4 * ff + 30 + th + 14);
        self.share_rect = Rect::new(py::fdiv32(w - pw, 2), py::fdiv32(h - ph, 2), pw, ph);
        let fx = self.share_rect.x + 18;
        let fw = pw - 36;
        let fh = ff;
        self.author_rect = Rect::new(fx, self.share_rect.y + head + lab, fw, fh);
        self.file_rect = Rect::new(fx, self.author_rect.bottom() + 8 + lab, fw, fh);
        let sy = self.file_rect.bottom() + 16;
        let sbw = (fw - 10) as f64 / 2.0;
        self.share_btn = vec![
            ("as", Rect::new(fx, sy, sbw as i32, fh)),
            ("dl", Rect::new((fx as f64 + sbw + 10.0) as i32, sy, sbw as i32, fh)),
            ("cancel", Rect::new(fx, sy + fh + 6, fw, fh)),
        ];
    }

    fn sbtn(&self, key: &str) -> Rect {
        self.share_btn.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(Rect::new(0, 0, 0, 0))
    }

    fn clamp(&mut self) {
        let n = self.items.len();
        let vis = self.rows_visible;
        self.first = self.first.min(n.saturating_sub(vis));
        if self.sel < self.first {
            self.first = self.sel;
        } else if self.sel >= self.first + vis {
            self.first = self.sel + 1 - vis;
        }
    }

    fn toast(&mut self, text: String) {
        self.toast = text;
        self.toast_t = 2.8;
    }

    /// `update(dt)`
    pub fn update(&mut self, dt: f64) {
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.toast.clear();
            }
        }
    }

    fn row_rect(&self, i: usize) -> Rect {
        Rect::new(12, self.list_top + i as i32 * self.row_h, self.list_w, self.row_h - 3)
    }

    /// `handle(event)`
    pub fn handle(&mut self, b: &mut GameBase, ev: &InputEvent) -> GdAction {
        if self.picker.is_some() {
            return self.handle_picker(b, ev);
        }
        if self.share.is_some() {
            self.handle_share(b, ev);
            return GdAction::None;
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if k == "Up" || k == "Left" || b.is_action(k, "up", None) {
                    self.mv(b, -1);
                } else if k == "Down" || k == "Right" || b.is_action(k, "down", None) {
                    self.mv(b, 1);
                } else if k == "Return" || k == "space" {
                    return self.action(b, if self.items.is_empty() { "new" } else { "play" });
                } else if k == "Delete" || k == "BackSpace" {
                    return self.action(b, "delete");
                } else if k == "n" || k == "N" {
                    return self.action(b, "new");
                } else if k == "e" || k == "E" {
                    return self.action(b, "edit");
                }
            }
            Kind::Wheel => {
                let maxf = self.items.len().saturating_sub(self.rows_visible) as i64;
                self.first = 0.max(maxf.min(self.first as i64 - ev.delta as i64)) as usize;
            }
            Kind::MouseDown => {
                if let Some(&(key, _)) = self.btn_rects.iter().find(|(_, r)| r.hit(ev.pos)) {
                    return self.action(b, key);
                }
                for i in 0..self.rows_visible {
                    let idx = self.first + i;
                    if idx >= self.items.len() {
                        break;
                    }
                    if self.row_rect(i).hit(ev.pos) {
                        if self.sel != idx {
                            self.confirm.clear();
                        }
                        self.sel = idx;
                        b.play_sound("move");
                        break;
                    }
                }
            }
            _ => {}
        }
        GdAction::None
    }

    fn mv(&mut self, b: &GameBase, d: i64) {
        if self.items.is_empty() {
            return;
        }
        self.sel = py::pmod(self.sel as i64 + d, self.items.len() as i64) as usize;
        self.confirm.clear();
        self.clamp();
        b.play_sound("move");
    }

    fn action(&mut self, b: &mut GameBase, key: &str) -> GdAction {
        if key != "delete" {
            self.confirm.clear();
        }
        if key == "new" {
            if ugc::is_full(GAME) {
                self.toast(tf("gd.ugc.err.full", &[("max", &ugc::max_items(GAME).to_string())]));
                return GdAction::None;
            }
            self.open_picker(b);
            return GdAction::None;
        }
        if key == "import" {
            self.do_import(b);
            return GdAction::None;
        }
        let Some(m) = self.selected().cloned() else {
            return GdAction::None;
        };
        let id = s_of(&m, "id").to_string();
        match key {
            "edit" => return GdAction::Edit(m),
            "play" => return GdAction::Play(m),
            "share" => self.open_share(b, m),
            "delete" => {
                if self.confirm != id {
                    self.confirm = id;
                    b.play_sound("select");
                    return GdAction::None;
                }
                ugc::delete_map(&id, GAME);
                self.confirm.clear();
                self.reload();
                self.toast(t("gd.ugc.deleted"));
                b.play_sound("click");
            }
            _ => {}
        }
        GdAction::None
    }

    // ----- Vorlagen -----------------------------------------------------

    fn open_picker(&mut self, b: &GameBase) {
        self.picker = Some(crate::geodash::load_templates());
        self.pick_sel = 0;
        b.play_sound("click");
    }

    /// Kacheln der Vorlagen-Auswahl.
    pub fn pick_rects(&self) -> (Rect, Vec<Rect>) {
        let (w, h) = (self.width, self.height);
        let n = self.picker.as_ref().map(|p| p.len()).unwrap_or(0) as i32;
        let cols = if w >= 760 { 4 } else { 3 };
        let rows = 1.max((n + cols - 1) / cols);
        let pw = (w - 30).min(420.max((w as f64 * 0.8) as i32));
        let th = self.tiny.get_height();
        let cell_h = 46.max(88.min(h / 7)).max(th * 3 + 14);
        let head = self.fnt.get_height() + 12;
        let foot = th + 18;
        let ph = (h - 16).min(head + rows * (cell_h + 8) + foot);
        let bx = Rect::new(py::fdiv32(w - pw, 2), py::fdiv32(h - ph, 2), pw, ph);
        let gap = 8;
        let cw = (pw - 24 - gap * (cols - 1)) as f64 / cols as f64;
        let mut rects = Vec::new();
        for i in 0..n {
            let (r, c) = (i / cols, i % cols);
            rects.push(Rect::new(
                (bx.x as f64 + 12.0 + c as f64 * (cw + gap as f64)) as i32,
                bx.y + head + r * (cell_h + gap),
                cw as i32,
                cell_h,
            ));
        }
        (bx, rects)
    }

    fn handle_picker(&mut self, _b: &GameBase, ev: &InputEvent) -> GdAction {
        let n = self.picker.as_ref().map(|p| p.len()).unwrap_or(0) as i64;
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if k == "Escape" || k == "BackSpace" {
                    self.picker = None;
                } else if k == "Left" || k == "Up" {
                    self.pick_sel = py::pmod(self.pick_sel as i64 - 1, n.max(1)) as usize;
                } else if k == "Right" || k == "Down" {
                    self.pick_sel = py::pmod(self.pick_sel as i64 + 1, n.max(1)) as usize;
                } else if k == "Return" || k == "space" || k == "KP_Enter" {
                    return self.use_template(self.pick_sel);
                }
            }
            Kind::MouseDown => {
                let (_bx, rects) = self.pick_rects();
                for (i, rc) in rects.iter().enumerate() {
                    if rc.hit(ev.pos) {
                        self.pick_sel = i;
                        return self.use_template(i);
                    }
                }
                self.picker = None;
            }
            _ => {}
        }
        GdAction::None
    }

    fn use_template(&mut self, i: usize) -> GdAction {
        let mut tpl = None;
        if let Some(p) = &self.picker {
            if let Some(x) = p.get(i) {
                if truthy(x.get("objects")) {
                    tpl = Some(x.clone());
                }
            }
        }
        self.picker = None;
        GdAction::NewLevel(tpl)
    }

    fn draw_picker(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let (bx, rects) = self.pick_rects();
        let mut overlay = surface_alpha(b.width, b.height);
        overlay.fill_c(rgba(0, 0, 0, 150));
        s.bl(&overlay, (0, 0));
        ui::draw_panel_ex(s, bx, Some(p.panel_light), None, Some(12), true, None);
        let title = self.fnt.render(&t("gd.tpl.title"), true, p.text);
        s.bl(&title, (bx.x + 14, bx.y + 6));
        let picker = self.picker.as_deref().unwrap_or(&[]);
        for (i, rc) in rects.iter().enumerate() {
            let tpl = &picker[i];
            let tid = s_of(tpl, "tpl");
            let sel = i == self.pick_sel;
            draw::rect_r(s, if !sel { p.panel } else { p.btn_sel }, *rc, 0, 7);
            draw::rect_r(s, if sel { b.accent } else { p.border }, *rc, if sel { 2 } else { 1 }, 7);
            let bg = rgb_of(tpl.get("bg"), core::DEFAULT_BG);
            draw::rect_r(s, bg, (rc.x + 4, rc.y + 4, 4, rc.h - 8), 0, 2);
            let name = self.tiny.render(&t(&format!("gd.tpl.{tid}")), true, p.text);
            s.bl(&name, (rc.x + 12, rc.y + 5));
            let desc = t(&format!("gd.tpl.{tid}.desc"));
            let mut img = self.tiny.render(&desc, true, p.text_dim);
            if img.w() > rc.w - 16 {
                img = self.tiny.render(&kurz(&self.tiny, &desc, rc.w - 16, 3, "..."), true, p.text_dim);
            }
            s.bl(&img, (rc.x + 12, rc.y + 7 + name.h()));
            if let Some(objs) = tpl.get("objects").and_then(|v| v.as_array()).filter(|a| !a.is_empty()) {
                let cnt = self.tiny.render(&tf("gd.ugc.objects", &[("n", &objs.len().to_string())]), true, p.text_faint);
                s.blr(&cnt, cnt.get_rect().bottomright_at((rc.right() - 8, rc.bottom() - 4)));
            }
        }
        let hint = self.tiny.render(&t("gd.tpl.hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().midbottom_at((bx.centerx(), bx.bottom() - 6)));
    }

    // ----- Teilen -------------------------------------------------------

    fn open_share(&mut self, b: &GameBase, m: Map<String, Value>) {
        self.err.clear();
        self.focus = 0;
        let author = s_of(&m, "author").to_string();
        self.f_author.set_text(&if author.is_empty() { ugc::last_author() } else { author });
        let id = s_of(&m, "id").to_string();
        self.f_file.set_text(if id.is_empty() { "level" } else { &id });
        self.share = Some(m);
        b.play_sound("click");
    }

    fn close_share(&mut self) {
        self.share = None;
        self.err.clear();
    }

    fn handle_share(&mut self, b: &mut GameBase, ev: &InputEvent) {
        if ev.kind == Kind::MouseDown {
            if self.author_rect.hit(ev.pos) {
                self.focus = 0;
            } else if self.file_rect.hit(ev.pos) {
                self.focus = 1;
            } else if self.sbtn("as").hit(ev.pos) {
                self.do_export(b, true);
            } else if self.sbtn("dl").hit(ev.pos) {
                self.do_export(b, false);
            } else if self.sbtn("cancel").hit(ev.pos) || !self.share_rect.hit(ev.pos) {
                self.close_share();
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        let consumed = if self.focus == 0 { self.f_author.handle(ev) } else { self.f_file.handle(ev) };
        if consumed {
            self.err.clear();
            return;
        }
        let k = ev.key.as_str();
        if ["Tab", "ISO_Left_Tab", "Up", "Down"].contains(&k) {
            self.focus = 1 - self.focus;
        } else if k == "Escape" {
            self.close_share();
        } else if k == "Return" || k == "KP_Enter" {
            self.do_export(b, false);
        }
    }

    fn do_export(&mut self, b: &mut GameBase, ask: bool) {
        let Some(m) = self.share.clone() else {
            return;
        };
        let author = self.f_author.text.trim().to_string();
        let mut name = self.f_file.text.trim().to_string();
        if name.is_empty() {
            name = if s_of(&m, "id").is_empty() { "level".into() } else { s_of(&m, "id").to_string() };
        }
        if !author.is_empty() && !swear::is_clean(&author) {
            self.err = t("gd.ugc.err.swear");
            b.play_sound("hit");
            return;
        }
        let mut out = m.clone();
        if !author.is_empty() {
            out.insert("author".into(), json!(author));
            ugc::set_last_author(&author);
        }
        let filename = name + ugc::ext(GAME);
        let path = if ask {
            if !filepick::available() {
                self.err = t("gd.ugc.err.nofile");
                return;
            }
            match filepick::save_as(&filename, &t("gd.ugc.share_title"), &[ugc::ext(GAME), ".json"]) {
                Some(p) => p,
                None => return,
            }
        } else {
            filepick::to_downloads(&filename)
        };
        let (ok, why) = ugc::export_to(&out, &path, GAME);
        if !ok {
            self.err = t(&format!("gd.ugc.err.{}", if why == "swear" { "swear" } else { "io" }));
            b.play_sound("hit");
            return;
        }
        if !author.is_empty() && s_of(&m, "author") != author {
            ugc::save_map(&out, GAME);
            self.reload();
        }
        self.close_share();
        self.toast(tf("gd.ugc.exported", &[("path", &filepick::short(&path, 44))]));
        b.play_sound("point");
    }

    fn do_import(&mut self, b: &mut GameBase) {
        if ugc::is_full(GAME) {
            self.toast(tf("gd.ugc.err.full", &[("max", &ugc::max_items(GAME).to_string())]));
            return;
        }
        if !filepick::available() {
            self.toast(t("gd.ugc.err.nofile"));
            return;
        }
        let Some(path) = filepick::open_file(&t("gd.ugc.btn_import"), &[ugc::ext(GAME), ".json"]) else {
            return;
        };
        self.import_path(b, &path);
    }

    /// `import_path(path)` - importiert eine Datei.
    pub fn import_path(&mut self, b: &GameBase, path: &str) -> bool {
        let stem = std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let wanted = ugc::slug(&stem);
        let (ok, why, m) = ugc::import_from(path, Some(GAME));
        let Some(m) = m.filter(|_| ok) else {
            let why = if ["swear", "full", "format", "io"].contains(&why) { why } else { "format" };
            self.toast(tf(&format!("gd.ugc.err.{why}"), &[("max", &ugc::max_items(GAME).to_string())]));
            b.play_sound("hit");
            return false;
        };
        self.reload();
        let id = s_of(&m, "id").to_string();
        if let Some(i) = self.items.iter().position(|x| s_of(x, "id") == id) {
            self.sel = i;
            self.clamp();
        }
        if !wanted.is_empty() && id != wanted {
            self.toast(tf("gd.ugc.renamed", &[("id", &id)]));
        } else {
            self.toast(tf("gd.ugc.imported", &[("name", s_of(&m, "name"))]));
        }
        b.play_sound("point");
        true
    }

    // ----- Zeichnen -----------------------------------------------------

    /// `draw(s)`
    pub fn draw(&self, s: &mut Surface, b: &GameBase, best: &Best) {
        let p = ui::pal();
        if self.items.is_empty() {
            self.draw_empty(s, b);
        } else {
            for i in 0..self.rows_visible {
                let idx = self.first + i;
                if idx >= self.items.len() {
                    break;
                }
                self.draw_row(s, b, best, &self.items[idx], idx, self.row_rect(i));
            }
            if self.items.len() > self.rows_visible {
                self.draw_scrollbar(s);
            }
        }
        let cnt = self.tiny.render(&tf("gd.ugc.count", &[("n", &self.items.len().to_string()), ("max", &ugc::max_items(GAME).to_string())]), true, p.text_dim);
        s.bl(&cnt, (12, self.list_bottom + 2));
        let has = self.selected().is_some();
        for &(key, rc) in &self.btn_rects {
            btn(s, rc, &t(&format!("gd.ugc.btn_{key}")), &self.tiny, false, None, has || !NEEDS_SEL.contains(&key));
        }
        if !self.toast.is_empty() {
            draw_list_toast(s, &self.tiny, &self.toast, self.width, self.list_bottom);
        }
        if self.share.is_some() {
            self.draw_share(s, b);
        }
        if self.picker.is_some() {
            self.draw_picker(s, b);
        }
    }

    fn draw_empty(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let bx = Rect::new(24, self.list_top + 10, b.width - 48, 60.max(self.list_bottom - self.list_top - 24));
        ui::draw_panel_ex(s, bx, None, None, None, false, None);
        for (i, line) in [t("gd.ugc.empty"), t("gd.ugc.empty2")].iter().enumerate() {
            let col = if i == 0 { p.text_dim } else { p.text_faint };
            let mut img = self.fnt.render(line, true, col);
            if img.w() > bx.w - 16 {
                img = self.tiny.render(line, true, col);
            }
            let step = self.fnt.get_height() + 8;
            s.blr(&img, img.get_rect().center_at((bx.centerx(), bx.centery() - step / 2 + i as i32 * step)));
        }
    }

    fn draw_row(&self, s: &mut Surface, b: &GameBase, best: &Best, m: &Map<String, Value>, idx: usize, rc: Rect) {
        let p = ui::pal();
        let sel = idx == self.sel;
        draw::rect_r(s, if sel { p.panel_light } else { p.panel }, rc, 0, 7);
        draw::rect_r(s, if sel { b.accent } else { p.border }, rc, if sel { 2 } else { 1 }, 7);
        let pad = 10;
        let bg = rgb_of(m.get("bg"), core::DEFAULT_BG);
        draw::rect_r(s, bg, (rc.x + 5, rc.y + 5, 5, rc.h - 10), 0, 2);
        let x = rc.x + pad + 6;
        let verified = is_verified(m);
        let name = self.fnt.render(s_of(m, "name"), true, p.text);
        s.bl(&name, (x, rc.y + 3));
        if verified {
            let br = 5.max(name.h() / 3);
            check_badge(s, (x + name.w() + br + 6) as f64, (rc.y + 3 + name.h() / 2) as f64, br as f64);
        }
        let lv_len = core::level_length(m);
        let n_obj = m.get("objects").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let mut sub = format!(
            "{}  \u{00B7}  {}  \u{00B7}  {}",
            s_of(m, "id"),
            tf("gd.ugc.objects", &[("n", &n_obj.to_string())]),
            tf("gd.ugc.length", &[("n", &lv_len.to_string())])
        );
        if !s_of(m, "author").is_empty() {
            sub += &format!("  \u{00B7}  {}", tf("gd.ugc.by", &[("name", s_of(m, "author"))]));
        }
        if !verified {
            sub += &format!("  \u{00B7}  {}", t("gd.ugc.unverified"));
        }
        let mut img = self.tiny.render(&sub, true, p.text_dim);
        if img.w() > rc.w - 90 {
            img = self.tiny.render(&kurz(&self.tiny, &sub, rc.w - 90, 4, "..."), true, p.text_dim);
        }
        if self.two_lines {
            s.bl(&img, (x, rc.bottom() - img.h() - 3));
        }
        let bst = best_get(best, &format!("ugc:{}", s_of(m, "id")))[0];
        if self.confirm == s_of(m, "id") {
            let warn = self.tiny.render(&t("gd.ugc.confirm_delete"), true, p.red);
            s.blr(&warn, warn.get_rect().midright_at((rc.right() - pad, rc.centery())));
        } else {
            let pct = self.fnt.render(&format!("{bst}%"), true, if bst >= 100 { rgb(185, 242, 58) } else { p.text_dim });
            s.blr(&pct, pct.get_rect().midright_at((rc.right() - pad, rc.centery())));
        }
    }

    fn draw_scrollbar(&self, s: &mut Surface) {
        let p = ui::pal();
        let track = Rect::new(self.list_w + 14, self.list_top, 4, self.rows_visible as i32 * self.row_h);
        draw::rect_r(s, p.panel, track, 0, 2);
        let frac = self.rows_visible as f64 / self.items.len() as f64;
        let hh = 24.max((track.h as f64 * frac) as i32);
        let pos = self.first as f64 / 1.max(self.items.len() as i64 - self.rows_visible as i64) as f64;
        let y = track.y + ((track.h - hh) as f64 * 1f64.min(pos)) as i32;
        draw::rect_r(s, p.border_light, (track.x, y, 4, hh), 0, 2);
    }

    fn draw_share(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let mut veil = surface_alpha(b.width, b.height);
        veil.fill_c(rgba(0, 0, 0, 150));
        s.bl(&veil, (0, 0));
        let r = self.share_rect;
        ui::draw_panel_ex(s, r, None, None, None, true, Some(b.accent));
        let head = self.fnt.render(&t("gd.ugc.share_title"), true, b.accent);
        s.blr(&head, head.get_rect().midtop_at((r.centerx(), r.y + 10)));
        for (i, (rc, key)) in [(self.author_rect, "gd.ugc.creator"), (self.file_rect, "gd.ugc.filename")].iter().enumerate() {
            let lab = self.tiny.render(&t(key), true, p.text_dim);
            s.bl(&lab, (rc.x, rc.y - lab.h() - 2));
            let field = if i == 0 { &self.f_author } else { &self.f_file };
            field.draw(s, *rc, &self.tiny, self.focus == i, false);
        }
        let ext = self.tiny.render(ugc::ext(GAME), true, p.text_faint);
        s.bl(&ext, (self.file_rect.right() - ext.w() - 8, self.file_rect.centery() - ext.h() / 2));
        btn(s, self.sbtn("as"), &t("gd.ugc.export_as"), &self.tiny, false, None, true);
        btn(s, self.sbtn("dl"), &t("gd.ugc.export_dl"), &self.tiny, false, None, true);
        btn(s, self.sbtn("cancel"), &t("gd.ugc.btn_cancel"), &self.tiny, false, None, true);
        if !self.err.is_empty() {
            let img = self.tiny.render(&self.err, true, p.red);
            s.blr(&img, img.get_rect().midbottom_at((r.centerx(), r.bottom() - 6)));
        }
    }
}

// ---------------------------------------------------------------------------
//  Der Level-Editor
// ---------------------------------------------------------------------------

const TEXT_BUTTONS: [&str; 2] = ["test", "test_here"];
const ICON_BUTTONS: [&str; 6] = ["undo", "redo", "rotate", "delete", "grid", "settings"];
const ROWS_VISIBLE: i32 = 11;
const SETTINGS_ROWS: [&str; 6] = ["speed", "mode", "music", "bpm", "bg", "ground"];
const PORTAL_KINDS: [&str; 11] = ["p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm", "g_flip", "s_slow", "s_norm", "s_fast", "s_vfast"];

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Pan { start: (i32, i32), cam_x: f64, moved: bool, cam_y: f64 },
    Map,
    Paint((i64, i64)),
    Erase,
    Move((i64, i64)),
}

type Snapshot = (Vec<Obj>, Map<String, Value>);

/// Level bauen: Leinwand links, Palette rechts, Uebersicht unten.
pub struct LevelEditor {
    meta: Map<String, Value>,
    objects: Vec<Obj>,
    /// Werkzeug bzw. Objektart.
    pub tool: &'static str,
    /// Paletten-Gruppe.
    pub group: &'static str,
    rot: i64,
    sel: Option<usize>,
    drag: Option<Drag>,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    dirty: bool,
    err: String,
    toast: String,
    toast_t: f64,
    confirm_back: bool,
    /// Einstellungs-Dialog offen.
    pub settings_open: bool,
    t: f64,
    /// Kamera x (Bloecke).
    pub cam_x: f64,
    cam_y: f64,
    hover: Option<(i64, i64)>,
    grid: bool,
    verified: String,
    f_name: TextInput,
    f_id: TextInput,
    /// Eingabefeld mit Fokus (-1 = keins).
    pub focus: i32,
    orig_id: String,
    renderer: Renderer,
    icon_cache: HashMap<(&'static str, i64, i32, i32, i32), Surface>,
    lv: Option<Level>,
    tiny: Font,
    small: Font,
    par_font: Font,
    bh: i32,
    /// Zurueck.
    pub back_rect: Rect,
    /// Name.
    pub name_rect: Rect,
    /// id.
    pub id_rect: Rect,
    /// Speichern.
    pub save_rect: Rect,
    /// Werkzeugleiste.
    pub edit_rects: Vec<(&'static str, Rect)>,
    pos_x: i32,
    head_bottom: i32,
    map_h: i32,
    /// Uebersichtsleiste.
    pub map_rect: Rect,
    par_lab_h: i32,
    par_h: i32,
    par_top: i32,
    pal_w: i32,
    /// Werkzeuge.
    pub tool_rects: Vec<(&'static str, Rect)>,
    /// Gruppen.
    pub group_rects: Vec<(&'static str, Rect)>,
    items_top: i32,
    pbw: i32,
    pbh: i32,
    px0: i32,
    /// Leinwand.
    pub canvas: Rect,
    ts: i32,
    /// Einstellungs-Dialog.
    pub set_rect: Rect,
    /// Zeilen des Dialogs.
    pub set_rows: Vec<(&'static str, Vec<Rect>)>,
    /// Schliessen-Knopf.
    pub set_close: Rect,
    lab_w: i32,
}

impl LevelEditor {
    /// `LevelEditor(game, m)`
    pub fn new(b: &GameBase, m: &Map<String, Value>, grid: bool) -> LevelEditor {
        let mut meta: Map<String, Value> = m.iter().filter(|(k, _)| k.as_str() != "objects").map(|(k, v)| (k.clone(), v.clone())).collect();
        let base = core::normalize_level(m);
        let objects = core::objects_of(&base);
        for key in ["speed", "mode", "bg", "ground", "music"] {
            meta.insert(key.into(), base[key].clone());
        }
        meta.entry("bpm").or_insert(json!(0));
        let z = Rect::new(0, 0, 0, 0);
        let verified = if is_verified(m) { s_of(m, "verified").to_string() } else { String::new() };
        let mut ed = LevelEditor {
            meta,
            objects,
            tool: "block",
            group: "blocks",
            rot: 0,
            sel: None,
            drag: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            err: String::new(),
            toast: String::new(),
            toast_t: 0.0,
            confirm_back: false,
            settings_open: false,
            t: 0.0,
            cam_x: -2.0,
            cam_y: -1.0,
            hover: None,
            grid,
            verified,
            f_name: TextInput::new(s_of(m, "name"), ugc::MAX_NAME, None, &t("gd.ugc.name")),
            f_id: TextInput::new(s_of(m, "id"), ugc::MAX_ID, Some(TextInput::ID_CHARS), &t("gd.ugc.id")),
            focus: -1,
            orig_id: s_of(m, "id").to_string(),
            renderer: Renderer::new(),
            icon_cache: HashMap::new(),
            lv: None,
            tiny: b.font.clone(),
            small: b.font.clone(),
            par_font: b.font.clone(),
            bh: 24,
            back_rect: z,
            name_rect: z,
            id_rect: z,
            save_rect: z,
            edit_rects: Vec::new(),
            pos_x: 0,
            head_bottom: 0,
            map_h: 12,
            map_rect: z,
            par_lab_h: 0,
            par_h: 0,
            par_top: 0,
            pal_w: 0,
            tool_rects: Vec::new(),
            group_rects: Vec::new(),
            items_top: 0,
            pbw: 30,
            pbh: 18,
            px0: 0,
            canvas: z,
            ts: 8,
            set_rect: z,
            set_rows: Vec::new(),
            set_close: z,
            lab_w: 0,
        };
        ed.layout(b);
        ed
    }

    // ----- Level-Daten ----------------------------------------------------

    /// `level_dict()`
    pub fn level_dict(&self) -> Map<String, Value> {
        let mut d = self.meta.clone();
        d.insert("objects".into(), Value::Array(self.objects.iter().map(|o| o.to_json()).collect()));
        let mut d = core::normalize_level(&d);
        if !self.verified.is_empty() && self.verified == core::content_hash(&d) {
            d.insert("verified".into(), json!(self.verified));
        } else {
            d.shift_remove("verified");
        }
        d
    }

    fn compiled(&mut self) -> &Level {
        if self.lv.is_none() {
            let lv = Level::new(&self.level_dict());
            gdraw::block_masks(&lv);
            self.lv = Some(lv);
        }
        self.lv.as_ref().expect("lv")
    }

    fn changed(&mut self) {
        self.lv = None;
        self.dirty = true;
        self.err.clear();
        self.confirm_back = false;
    }

    /// `mark_verified()` - der Ersteller hat sein Level geschafft.
    pub fn mark_verified(&mut self) {
        let d = self.level_dict();
        self.verified = core::content_hash(&d);
        self.lv = None;
        if !self.dirty && ugc::valid_id(&self.orig_id) && ugc::get(&self.orig_id, GAME).is_some() {
            let mut m = self.level_dict();
            m.insert("id".into(), json!(self.orig_id));
            let nm = self.f_name.text.trim().to_string();
            m.insert("name".into(), json!(if nm.is_empty() { s_of(&self.meta, "name").to_string() } else { nm }));
            m.insert("verified".into(), json!(self.verified));
            ugc::save_map(&m, GAME);
        }
        self.toast_msg(t("gd.ed.verified"));
    }

    fn meta_i(&self, key: &str) -> i64 {
        self.meta.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
    }

    // ----- Layout -------------------------------------------------------

    /// `layout()`
    pub fn layout(&mut self, b: &GameBase) {
        let (w, h) = (b.width, b.height);
        self.tiny = ui::font(10.max(h / 42), false, false);
        self.small = ui::font(12.max(h / 34), false, false);
        let th = self.tiny.get_height();
        let bh = 20.max(28.min(h / 15)).max(th + 8);
        let gap = 5;
        self.bh = bh;
        let y = 6;
        let back_w = 48.max((w as f64 * 0.11) as i32);
        let save_w = 62.max((w as f64 * 0.16) as i32);
        let field_w = (w - 24 - back_w - save_w - 3 * gap) as f64 / 2.0;
        self.back_rect = Rect::new(12, y, back_w, bh);
        self.name_rect = Rect::new(12 + back_w + gap, y, field_w as i32, bh);
        self.id_rect = Rect::new(self.name_rect.right() + gap, y, field_w as i32, bh);
        self.save_rect = Rect::new(self.id_rect.right() + gap, y, save_w, bh);
        let y2 = y + bh + gap;
        let icon_w = 22.max(bh + 2);
        let need = TEXT_BUTTONS.iter().map(|k| self.tiny.size(&t(&format!("gd.ed.btn_{k}"))).0).max().unwrap_or(0) + bh + 10;
        let text_w = 52.max(((w as f64 * 0.2) as i32).min(need));
        self.edit_rects.clear();
        let mut x = 12;
        for key in TEXT_BUTTONS {
            self.edit_rects.push((key, Rect::new(x, y2, text_w, bh)));
            x += text_w + gap;
        }
        for key in ICON_BUTTONS {
            self.edit_rects.push((key, Rect::new(x, y2, icon_w, bh)));
            x += icon_w + gap;
        }
        self.pos_x = x + 4;
        self.head_bottom = y2 + bh + 6;
        self.map_h = 12.max(h / 30);
        self.map_rect = Rect::new(12, h - self.map_h - 6, w - 24, self.map_h);
        self.par_font = ui::font(9.max(h / 52), false, false);
        self.par_lab_h = self.par_font.get_height();
        self.par_h = bh + 8 + self.par_lab_h;
        self.par_top = self.map_rect.y - self.par_h - 2;
        let pbw = 30.max(50.max(h / 13).min((w as f64 * 0.07) as i32));
        let cols = 2;
        let max_items = GROUPS.iter().map(|g| g.1.len()).max().unwrap_or(1) as i32;
        let rows = 1 + 3 + (max_items + 1) / 2;
        let avail = self.par_top - self.head_bottom - 14;
        let pbh = 18.max(pbw.min(py::fdiv32(avail - (rows - 1) * 3, rows)));
        self.pal_w = cols * pbw + 3;
        let px0 = w - self.pal_w - 8;
        self.tool_rects = TOOLS.iter().enumerate().map(|(i, k)| (*k, Rect::new(px0 + i as i32 * (pbw + 3), self.head_bottom, pbw, pbh))).collect();
        let gy = self.head_bottom + pbh + 6;
        self.group_rects.clear();
        for (i, (key, _)) in GROUPS.iter().enumerate() {
            let (r, c) = (i as i32 / cols, i as i32 % cols);
            self.group_rects.push((key, Rect::new(px0 + c * (pbw + 3), gy + r * (pbh + 3), pbw, pbh)));
        }
        self.items_top = gy + 3 * (pbh + 3) + 6;
        self.pbw = pbw;
        self.pbh = pbh;
        self.px0 = px0;
        self.canvas = Rect::new(12, self.head_bottom, px0 - 12 - 10, self.par_top - 4 - self.head_bottom);
        self.ts = 8.max(py::fdiv32(self.canvas.h, ROWS_VISIBLE));
        self.renderer.resize(self.canvas.w, self.canvas.h, self.ts as f64);
        self.icon_cache.clear();
        let pw = (w - 30).min(440.max((w as f64 * 0.5) as i32));
        let rh = 22.max(30.min(h / 16)).max(th + 8);
        let head = self.small.get_height() + 18;
        let ph = head + SETTINGS_ROWS.len() as i32 * (rh + 8) + rh + 18;
        self.set_rect = Rect::new(py::fdiv32(w - pw, 2), 6.max(py::fdiv32(h - ph, 2)), pw, (h - 12).min(ph));
        self.set_rows.clear();
        let lab_w = (pw as f64 * 0.34) as i32;
        for (i, key) in SETTINGS_ROWS.iter().enumerate() {
            let ry = self.set_rect.y + head + i as i32 * (rh + 8);
            let x0 = self.set_rect.x + 14 + lab_w;
            let vw = self.set_rect.right() - 14 - x0;
            if *key == "bg" || *key == "ground" {
                let n = PRESETS.len() as i32;
                let cw = 8.max(py::fdiv32(vw - (n - 1) * 3, n));
                self.set_rows.push((key, (0..n).map(|j| Rect::new(x0 + j * (cw + 3), ry, cw, rh)).collect()));
            } else {
                self.set_rows.push((key, vec![Rect::new(x0, ry, rh, rh), Rect::new(x0 + rh + 4, ry, vw - 2 * rh - 8, rh), Rect::new(x0 + vw - rh, ry, rh, rh)]));
            }
        }
        let cw = 140.max(self.tiny.size(&t("gd.ed.close")).0 + 40);
        self.set_close = Rect::new(self.set_rect.centerx() - cw / 2, self.set_rect.bottom() - rh - 10, cw, rh);
        self.lab_w = lab_w;
    }

    /// Objekt-Knoepfe der aktuellen Gruppe.
    pub fn item_rects(&self) -> Vec<(&'static str, Rect)> {
        group_items(self.group)
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let (r, c) = (i as i32 / 2, i as i32 % 2);
                (*k, Rect::new(self.px0 + c * (self.pbw + 3), self.items_top + r * (self.pbh + 3), self.pbw, self.pbh))
            })
            .collect()
    }

    fn edit_rect(&self, key: &str) -> Rect {
        self.edit_rects.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(Rect::new(0, 0, 0, 0))
    }

    // ----- Undo -------------------------------------------------------------

    fn snapshot(&self) -> Snapshot {
        (self.objects.clone(), self.meta.clone())
    }

    fn push(&mut self) {
        let snap = self.snapshot();
        self.undo_stack.push(snap);
        if self.undo_stack.len() > UNDO_MAX {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self, b: &GameBase) {
        let Some((o, m)) = self.undo_stack.pop() else {
            return;
        };
        let snap = self.snapshot();
        self.redo_stack.push(snap);
        self.objects = o;
        self.meta = m;
        self.sel = None;
        self.changed();
        b.play_sound("move");
    }

    fn redo(&mut self, b: &GameBase) {
        let Some((o, m)) = self.redo_stack.pop() else {
            return;
        };
        let snap = self.snapshot();
        self.undo_stack.push(snap);
        self.objects = o;
        self.meta = m;
        self.sel = None;
        self.changed();
        b.play_sound("move");
    }

    fn toast_msg(&mut self, text: String) {
        self.toast = text;
        self.toast_t = 2.6;
    }

    /// `update(dt)`
    pub fn update(&mut self, dt: f64) {
        self.t += dt;
        if self.toast_t > 0.0 {
            self.toast_t -= dt;
            if self.toast_t <= 0.0 {
                self.toast.clear();
            }
        }
    }

    // ----- Koordinaten ------------------------------------------------------

    /// `cell_at(pos)`
    pub fn cell_at(&self, pos: (i32, i32)) -> (i64, i64) {
        let x = (pos.0 - self.canvas.x) as f64 / self.ts as f64 + self.cam_x;
        let y = (self.canvas.bottom() - pos.1) as f64 / self.ts as f64 + self.cam_y;
        (x.floor() as i64, y.floor() as i64)
    }

    fn find(&self, cx: i64, cy: i64, layer: Option<i64>) -> Option<usize> {
        let mut best: Option<usize> = None;
        for (i, o) in self.objects.iter().enumerate() {
            if o.x() == cx && (o.y() == cy || (o.kind == "pit" && cy <= 0)) {
                if let Some(l) = layer {
                    if layer_of(o.kind) != l {
                        continue;
                    }
                }
                if best.is_none_or(|bi| layer_of(o.kind) <= layer_of(self.objects[bi].kind)) {
                    best = Some(i);
                }
            }
        }
        best
    }

    fn clamp_cam(&mut self) {
        let end = if !self.objects.is_empty() { core::level_length(&self.level_dict()) } else { core::MIN_LENGTH };
        self.cam_x = (-4.0f64).max((end as f64).min(self.cam_x));
        self.cam_y = (-1.0f64).max(((core::MAX_ROW - ROWS_VISIBLE as i64 + 2) as f64).min(self.cam_y));
    }

    // ----- Eingabe ------------------------------------------------------

    /// `handle(event)`
    pub fn handle(&mut self, b: &mut GameBase, ev: &InputEvent) -> GdAction {
        if self.settings_open {
            self.handle_settings(b, ev);
            return GdAction::None;
        }
        match ev.kind {
            Kind::KeyDown => self.handle_key(b, ev),
            Kind::MouseDown => self.handle_down(b, ev),
            Kind::MouseMove => {
                self.handle_move(b, ev);
                GdAction::None
            }
            Kind::MouseUp => {
                if let Some(Drag::Pan { moved: false, .. }) = self.drag {
                    let c = self.cell_at(ev.pos);
                    self.erase_at(b, c, true);
                }
                self.drag = None;
                GdAction::None
            }
            Kind::Wheel => {
                if self.map_rect.hit(ev.pos) || self.canvas.hit(ev.pos) {
                    self.cam_x -= (ev.delta * 3) as f64;
                    self.clamp_cam();
                }
                GdAction::None
            }
            _ => GdAction::None,
        }
    }

    fn handle_key(&mut self, b: &mut GameBase, ev: &InputEvent) -> GdAction {
        let k = ev.key.as_str();
        if self.focus >= 0 {
            let consumed = if self.focus == 0 { self.f_name.handle(ev) } else { self.f_id.handle(ev) };
            if consumed {
                self.dirty = true;
                self.err.clear();
                return GdAction::None;
            }
            if k == "Tab" || k == "ISO_Left_Tab" {
                self.focus = 1 - self.focus;
                return GdAction::None;
            }
            if k == "Return" || k == "KP_Enter" || k == "Escape" {
                self.focus = -1;
                return GdAction::None;
            }
        }
        match k {
            "Escape" => return self.back(b),
            "Left" | "a" | "A" => {
                self.cam_x -= if k == "Left" { 4.0 } else { 1.0 };
                self.clamp_cam();
            }
            "Right" | "d" | "D" => {
                self.cam_x += if k == "Right" { 4.0 } else { 1.0 };
                self.clamp_cam();
            }
            "Up" | "w" | "W" => {
                self.cam_y += 1.0;
                self.clamp_cam();
            }
            "Down" => {
                self.cam_y -= 1.0;
                self.clamp_cam();
            }
            "Home" => self.cam_x = -2.0,
            "End" => {
                self.cam_x = (core::level_length(&self.level_dict()) - 10) as f64;
                self.clamp_cam();
            }
            "Delete" | "BackSpace" => self.delete_selected(b),
            "r" | "R" => self.rotate(b),
            "u" | "U" | "z" | "Z" => self.undo(b),
            "y" | "Y" => self.redo(b),
            "g" | "G" => self.toggle_grid(b),
            "s" | "S" => {
                self.save(b);
            }
            "Return" | "KP_Enter" => return self.test(false),
            "t" | "T" => return self.test(true),
            "1" | "2" | "3" | "4" | "5" | "6" => {
                let i = k.parse::<usize>().unwrap_or(1) - 1;
                self.pick_group(b, GROUPS[i].0);
            }
            _ => {}
        }
        GdAction::None
    }

    fn handle_down(&mut self, b: &mut GameBase, ev: &InputEvent) -> GdAction {
        let pos = ev.pos;
        if ev.button == 3 {
            if self.canvas.hit(pos) {
                self.drag = Some(Drag::Pan { start: pos, cam_x: self.cam_x, moved: false, cam_y: self.cam_y });
            }
            return GdAction::None;
        }
        if self.name_rect.hit(pos) {
            self.focus = 0;
            return GdAction::None;
        }
        if self.id_rect.hit(pos) {
            self.focus = 1;
            return GdAction::None;
        }
        self.focus = -1;
        if self.back_rect.hit(pos) {
            return self.back(b);
        }
        if self.save_rect.hit(pos) {
            self.save(b);
            return GdAction::None;
        }
        if let Some(&(key, _)) = self.edit_rects.iter().find(|(_, rc)| rc.hit(pos)) {
            match key {
                "test" => return self.test(false),
                "test_here" => return self.test(true),
                "undo" => self.undo(b),
                "redo" => self.redo(b),
                "rotate" => self.rotate(b),
                "delete" => self.delete_selected(b),
                "grid" => self.toggle_grid(b),
                _ => {
                    self.settings_open = true;
                    b.play_sound("click");
                }
            }
            return GdAction::None;
        }
        if let Some(&(key, _)) = self.tool_rects.iter().find(|(_, rc)| rc.hit(pos)) {
            self.tool = key;
            self.sel = None;
            b.play_sound("select");
            return GdAction::None;
        }
        if let Some(&(key, _)) = self.group_rects.iter().find(|(_, rc)| rc.hit(pos)) {
            self.pick_group(b, key);
            return GdAction::None;
        }
        if let Some((key, _)) = self.item_rects().into_iter().find(|(_, rc)| rc.hit(pos)) {
            self.tool = key;
            self.sel = None;
            self.rot = if core::ROTATE_FLIP.contains(&key) && self.rot == 2 { 2 } else { 0 };
            b.play_sound("select");
            return GdAction::None;
        }
        if self.map_rect.hit(pos) {
            self.jump_map(pos);
            self.drag = Some(Drag::Map);
            return GdAction::None;
        }
        if self.param_click(b, pos) {
            return GdAction::None;
        }
        if self.canvas.hit(pos) {
            self.canvas_down(b, pos);
        }
        GdAction::None
    }

    fn pick_group(&mut self, b: &GameBase, key: &'static str) {
        self.group = key;
        let items = group_items(key);
        if !items.contains(&self.tool) {
            self.tool = items[0];
        }
        self.sel = None;
        b.play_sound("select");
    }

    fn handle_move(&mut self, b: &GameBase, ev: &InputEvent) {
        let pos = ev.pos;
        self.hover = if self.canvas.hit(pos) { Some(self.cell_at(pos)) } else { None };
        let Some(d) = self.drag else {
            return;
        };
        match d {
            Drag::Pan { start, cam_x, moved, cam_y } => {
                let dx = (pos.0 - start.0) as f64 / self.ts as f64;
                let dy = (pos.1 - start.1) as f64 / self.ts as f64;
                let moved = moved || (pos.0 - start.0).abs() + (pos.1 - start.1).abs() > 4;
                self.cam_x = cam_x - dx;
                self.cam_y = cam_y + dy;
                self.clamp_cam();
                self.drag = Some(Drag::Pan { start, cam_x, moved, cam_y });
            }
            Drag::Map => self.jump_map(pos),
            Drag::Paint(last) => {
                if self.canvas.hit(pos) {
                    let cell = self.cell_at(pos);
                    if cell != last {
                        self.place(b, cell, false);
                        self.drag = Some(Drag::Paint(cell));
                    }
                }
            }
            Drag::Erase => {
                if self.canvas.hit(pos) {
                    let c = self.cell_at(pos);
                    self.erase_at(b, c, false);
                }
            }
            Drag::Move(last) => {
                let cell = self.cell_at(pos);
                if let Some(si) = self.sel.filter(|_| cell != last) {
                    let o = &self.objects[si];
                    let nx = 0.max(core::MAX_LENGTH.min(o.x() + cell.0 - last.0));
                    let mut ny = 0.max(core::MAX_ROW.min(o.y() + cell.1 - last.1));
                    if o.kind == "pit" {
                        ny = 0;
                    }
                    let f = self.find(nx, ny, Some(layer_of(o.kind)));
                    if f.is_none() || f == Some(si) {
                        self.objects[si].v[0] = nx;
                        self.objects[si].v[1] = ny;
                        self.changed();
                    }
                    self.drag = Some(Drag::Move(cell));
                }
            }
        }
    }

    fn jump_map(&mut self, pos: (i32, i32)) {
        let end = core::MIN_LENGTH.max(core::level_length(&self.level_dict()));
        let f = (pos.0 - self.map_rect.x) as f64 / 1.max(self.map_rect.w) as f64;
        let view = self.canvas.w as f64 / self.ts as f64;
        self.cam_x = f * end as f64 - view / 2.0;
        self.clamp_cam();
    }

    fn canvas_down(&mut self, b: &GameBase, pos: (i32, i32)) {
        let cell = self.cell_at(pos);
        if cell.0 < 0 || cell.1 < 0 {
            return;
        }
        if self.tool == "select" {
            let i = self.find(cell.0, cell.1, None);
            self.sel = i;
            if i.is_some() {
                self.push();
                self.drag = Some(Drag::Move(cell));
                b.play_sound("move");
            }
            return;
        }
        if self.tool == "erase" {
            self.push();
            self.erase_at(b, cell, false);
            self.drag = Some(Drag::Erase);
            return;
        }
        self.push();
        self.place(b, cell, false);
        self.drag = Some(Drag::Paint(cell));
    }

    fn place(&mut self, b: &GameBase, cell: (i64, i64), push: bool) {
        let (x, mut y) = cell;
        if x < 0 || y < 0 || x > core::MAX_LENGTH || y > core::MAX_ROW {
            return;
        }
        let kind = self.tool;
        if kind == "pit" {
            y = 0;
        }
        if kind == "coin" {
            let coins: Vec<usize> = self.objects.iter().enumerate().filter(|(_, o)| o.kind == "coin").map(|(i, _)| i).collect();
            let f = self.find(x, y, None);
            if coins.len() >= core::MAX_COINS && !f.is_some_and(|fi| coins.contains(&fi)) {
                self.toast_msg(t("gd.ed.coins_max"));
                b.play_sound("hit");
                return;
            }
        }
        if push {
            self.push();
        }
        let old = self.find(x, y, Some(layer_of(kind)));
        let rot = if core::ROTATE_ALL.contains(&kind) || core::ROTATE_FLIP.contains(&kind) { self.rot } else { 0 };
        let mut raw = vec![json!(kind), json!(x), json!(y), json!(rot)];
        raw.extend(default_param(kind).into_iter().map(|v| json!(v)));
        let Some(obj) = core::normalize_object(&Value::Array(raw)) else {
            return;
        };
        if let Some(oi) = old {
            let o = &self.objects[oi];
            if o.kind == obj.kind && o.v[..3] == obj.v[..3] {
                self.sel = Some(oi);
                return;
            }
            self.objects[oi] = obj;
            self.sel = Some(oi);
        } else {
            self.objects.push(obj);
            self.sel = Some(self.objects.len() - 1);
        }
        self.changed();
        b.play_sound("click");
    }

    fn erase_at(&mut self, b: &GameBase, cell: (i64, i64), push: bool) {
        let Some(i) = self.find(cell.0, cell.1, None) else {
            return;
        };
        if push {
            self.push();
        }
        self.objects.remove(i);
        self.sel = None;
        self.changed();
        b.play_sound("hit");
    }

    fn delete_selected(&mut self, b: &GameBase) {
        let Some(si) = self.sel.filter(|&s| s < self.objects.len()) else {
            return;
        };
        self.push();
        self.objects.remove(si);
        self.sel = None;
        self.changed();
        b.play_sound("hit");
    }

    fn rotate(&mut self, b: &GameBase) {
        if let Some(si) = self.sel.filter(|&s| s < self.objects.len()) {
            let kind = self.objects[si].kind;
            if core::ROTATE_ALL.contains(&kind) || core::ROTATE_FLIP.contains(&kind) {
                self.push();
                let step = if core::ROTATE_FLIP.contains(&kind) { 2 } else { 1 };
                let o = &mut self.objects[si];
                o.v[2] = (o.v[2] + step).rem_euclid(4);
                self.changed();
                b.play_sound("rotate");
            }
            return;
        }
        self.rot = (self.rot + if core::ROTATE_FLIP.contains(&self.tool) { 2 } else { 1 }).rem_euclid(4);
        b.play_sound("rotate");
    }

    fn toggle_grid(&mut self, b: &mut GameBase) {
        self.grid = !self.grid;
        b.set_opt("geodash", "grid", json!(self.grid));
        b.play_sound("select");
    }

    // ----- Parameterzeile ---------------------------------------------------

    fn params(&self) -> Option<(&Obj, &'static [(i64, i64, i64)])> {
        let si = self.sel.filter(|&s| s < self.objects.len())?;
        let o = &self.objects[si];
        Some((o, core::params(o.kind)))
    }

    /// Knoepfe der Parameterzeile: (index, minus, wert, plus).
    pub fn param_rects(&self, width: i32) -> Vec<(usize, Rect, Rect, Rect)> {
        let Some((o, params)) = self.params() else {
            return Vec::new();
        };
        if params.is_empty() {
            return Vec::new();
        }
        let bh = self.bh;
        let step_w = 16.max((width as f64 * 0.032) as i32);
        let val_w = 30.max((width as f64 * 0.06) as i32);
        let grp = 2 * step_w + val_w;
        let n = params.len() as i32;
        let total = n * grp + (n - 1) * 6 + if o.kind == "color" { val_w / 2 } else { 0 };
        let mut x = 10.max(py::fdiv32(self.canvas.right() - total, 2));
        let y = self.par_top + self.par_lab_h + 2;
        let mut out = Vec::new();
        for i in 0..params.len() {
            let vw = val_w + if o.kind == "color" && i == 0 { val_w / 2 } else { 0 };
            out.push((i, Rect::new(x, y, step_w, bh), Rect::new(x + step_w, y, vw, bh), Rect::new(x + step_w + vw, y, step_w, bh)));
            x += 2 * step_w + vw + 6;
        }
        out
    }

    fn param_click(&mut self, b: &GameBase, pos: (i32, i32)) -> bool {
        let Some((o, params)) = self.params() else {
            return false;
        };
        let kind = o.kind;
        for (i, minus, _val, plus) in self.param_rects(b.width) {
            for (rc, d) in [(minus, -1), (plus, 1)] {
                if rc.hit(pos) {
                    let (_default, lo, hi) = params[i];
                    let stepv = if kind == "color" && (1..=3).contains(&i) { 15 } else { 1 };
                    self.push();
                    let si = self.sel.expect("sel");
                    let v = &mut self.objects[si].v[3 + i];
                    *v = lo.max(hi.min(*v + d * stepv));
                    self.changed();
                    b.play_sound("move");
                    return true;
                }
            }
        }
        false
    }

    // ----- Einstellungen ----------------------------------------------------

    fn handle_settings(&mut self, b: &GameBase, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown && (ev.key == "Escape" || ev.key == "Return") {
            self.settings_open = false;
            return;
        }
        if ev.kind != Kind::MouseDown {
            return;
        }
        let pos = ev.pos;
        if self.set_close.hit(pos) || !self.set_rect.hit(pos) {
            self.settings_open = false;
            return;
        }
        let rows = self.set_rows.clone();
        for (key, rects) in rows {
            for (j, rc) in rects.iter().enumerate() {
                if !rc.hit(pos) {
                    continue;
                }
                self.push();
                if key == "bg" || key == "ground" {
                    let p = PRESETS[j];
                    self.meta.insert(key.into(), json!([p.0, p.1, p.2]));
                } else if j == 0 || j == 2 {
                    let d: i64 = if j == 0 { -1 } else { 1 };
                    match key {
                        "speed" => {
                            let v = (self.meta_i("speed") + d).rem_euclid(4);
                            self.meta.insert("speed".into(), json!(v));
                        }
                        "mode" => {
                            let v = (self.meta_i("mode") + d).rem_euclid(5);
                            self.meta.insert("mode".into(), json!(v));
                        }
                        "music" => {
                            let cur = self.meta.get("music").and_then(|v| v.as_str()).unwrap_or("drive").to_string();
                            let i = core::MUSIC_STYLES.iter().position(|s| *s == cur).unwrap_or(0) as i64;
                            let n = core::MUSIC_STYLES.len() as i64;
                            self.meta.insert("music".into(), json!(core::MUSIC_STYLES[(i + d).rem_euclid(n) as usize]));
                        }
                        _ => {
                            let sb = music::style_bpm(&self.meta);
                            let cur = if sb != 0 { sb } else { 120 };
                            self.meta.insert("bpm".into(), json!(80.max(200.min(cur + 4 * d))));
                        }
                    }
                }
                self.changed();
                b.play_sound("move");
                return;
            }
        }
    }

    // ----- Speichern / Testen / Verlassen ----------------------------------------

    fn save(&mut self, b: &GameBase) -> bool {
        let name = self.f_name.text.trim().to_string();
        let mut level_id = self.f_id.text.trim().to_string();
        if level_id.is_empty() {
            level_id = ugc::slug(&name);
        }
        if level_id.is_empty() {
            return self.fail(b, "gd.ugc.err.id_empty");
        }
        if !ugc::valid_id(&level_id) {
            return self.fail(b, "gd.ugc.err.id_chars");
        }
        if name.is_empty() {
            return self.fail(b, "gd.ugc.err.name");
        }
        let taken = ugc::load_maps(GAME).iter().any(|m| s_of(m, "id") != self.orig_id && s_of(m, "id") == level_id);
        if taken {
            return self.fail(b, "gd.ugc.err.id_dup");
        }
        let author = s_of(&self.meta, "author").to_string();
        let texts: Vec<&str> = [name.as_str(), level_id.as_str(), author.as_str()].into_iter().filter(|x| !x.is_empty()).collect();
        if !swear::all_clean(&texts) {
            return self.fail(b, "gd.ugc.err.swear");
        }
        let mut m = self.level_dict();
        m.insert("id".into(), json!(level_id));
        m.insert("name".into(), json!(name));
        if !self.orig_id.is_empty() && self.orig_id != level_id {
            ugc::delete_map(&self.orig_id, GAME);
        }
        let (ok, why) = ugc::save_map(&m, GAME);
        if !ok {
            let why = if ["swear", "full", "io", "id"].contains(&why) { why } else { "invalid" };
            self.err = tf(&format!("gd.ugc.err.{why}"), &[("max", &ugc::max_items(GAME).to_string())]);
            b.play_sound("hit");
            return false;
        }
        self.meta.insert("id".into(), json!(level_id));
        self.meta.insert("name".into(), json!(name));
        self.orig_id = level_id.clone();
        self.dirty = false;
        self.f_id.set_text(&level_id);
        self.toast_msg(t("gd.ugc.saved"));
        b.play_sound("win");
        true
    }

    fn fail(&mut self, b: &GameBase, key: &str) -> bool {
        self.err = t(key);
        b.play_sound("hit");
        false
    }

    fn test(&mut self, here: bool) -> GdAction {
        let mut d = self.level_dict();
        let nm = self.f_name.text.trim().to_string();
        d.insert("name".into(), json!(if nm.is_empty() { t("gd.ed.untitled") } else { nm }));
        let fid = self.f_id.text.trim().to_string();
        let id = if !fid.is_empty() {
            fid
        } else if !self.orig_id.is_empty() {
            self.orig_id.clone()
        } else {
            "test".to_string()
        };
        d.insert("id".into(), json!(id));
        let mut start = None;
        if here {
            let st = 1.max((self.cam_x + 2.0).floor() as i64);
            if st < core::level_length(&d) - 2 {
                start = Some(st);
            }
        }
        GdAction::Test(d, start)
    }

    fn back(&mut self, b: &GameBase) -> GdAction {
        if self.settings_open {
            self.settings_open = false;
            return GdAction::None;
        }
        if self.dirty && !self.confirm_back {
            self.confirm_back = true;
            self.toast_msg(t("gd.ugc.unsaved"));
            b.play_sound("select");
            return GdAction::None;
        }
        GdAction::Close
    }

    // ----- Zeichnen -----------------------------------------------------

    /// `draw(s)`
    pub fn draw(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        self.draw_canvas(s, b);
        self.draw_head(s, b);
        self.draw_palette(s, b);
        self.draw_bottom(s, b);
        if !self.toast.is_empty() {
            let img = self.tiny.render(&self.toast, true, p.text);
            let (w, h) = (img.w() + 24, img.h() + 10);
            let r = rect(self.canvas.centerx() - w / 2, self.canvas.bottom() - h - 8, w, h);
            ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
            draw::rect_r(s, p.accent2, r, 1, h / 2);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        if self.settings_open {
            self.draw_settings(s, b);
        }
    }

    fn draw_canvas(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        self.compiled();
        let canvas = self.canvas;
        let ts = self.ts;
        let (cam_x, cam_y) = (self.cam_x, self.cam_y);
        let mut sub = s.subsurface(canvas);
        let lv = self.lv.as_ref().expect("lv");
        let r = &mut self.renderer;
        r.resize(canvas.w, canvas.h, ts as f64);
        let bg = gdraw::color_at(lv, cam_x + 4.0, 0);
        let ground = gdraw::color_at(lv, cam_x + 4.0, 1);
        r.draw_backdrop(&mut sub, bg, cam_x, cam_y, 0.0);
        if self.grid {
            let gcol = mix(bg, (255, 255, 255), 0.12);
            let x0 = cam_x.floor() as i64;
            for c in x0..x0 + (canvas.w / ts) as i64 + 2 {
                let x = r.sx(c as f64, cam_x) as i32;
                draw::line(&mut sub, gcol, (x, 0), (x, canvas.h), 1);
            }
            let y0 = cam_y.floor() as i64;
            for row in y0..y0 + ROWS_VISIBLE as i64 + 2 {
                let y = r.sy(row as f64, cam_y) as i32;
                draw::line(&mut sub, gcol, (0, y), (canvas.w, y), 1);
            }
        }
        let used = std::collections::BTreeSet::new();
        r.draw_objects(&mut sub, lv, cam_x, cam_y, self.t, 0.0, &used, true, None);
        r.draw_ground(&mut sub, Some(lv), ground, cam_x, cam_y, 0.0);
        for (xb, col) in [(0.0, rgb(90, 255, 140)), (lv.length as f64, rgb(255, 255, 255))] {
            let x = r.sx(xb, cam_x) as i32;
            if 0 <= x && x <= canvas.w {
                draw::line(&mut sub, col, (x, 0), (x, canvas.h), 2);
            }
        }
        let sx0 = r.sx(0.0, cam_x);
        if -2.0 < sx0 && sx0 < canvas.w as f64 {
            let mode = self.meta.get("mode").and_then(|v| v.as_i64()).unwrap_or(0);
            r.draw_player(&mut sub, -1.4, 0.0, mode, 1, 0.0, cam_x, cam_y, None, 255);
        }
        if let Some(si) = self.sel.filter(|&s| s < self.objects.len()) {
            let o = &self.objects[si];
            let mut rc = Rect::new(r.sx(o.x() as f64, cam_x) as i32, r.sy((o.y() + 1) as f64, cam_y) as i32, ts, ts);
            if PORTAL_KINDS.contains(&o.kind) {
                rc = rc.inflate(0, 2 * ts);
            }
            draw::rect_r(&mut sub, b.accent, rc.inflate(6, 6), 2, 4);
        }
        if let Some((hx, hy)) = self.hover.filter(|_| !TOOLS.contains(&self.tool)) {
            let rc = Rect::new(r.sx(hx as f64, cam_x) as i32, r.sy((hy + 1) as f64, cam_y) as i32, ts, ts);
            let mut ghost = surface_alpha(ts, ts);
            ghost.fill_c(rgba(255, 255, 255, 40));
            sub.blr(&ghost, rc);
            draw::rect(&mut sub, (255, 255, 255), rc, 1);
        }
        draw::rect_r(s, p.border, canvas.inflate(4, 4), 1, 4);
    }

    fn draw_head(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        btn(s, self.back_rect, &t("gd.ugc.btn_back"), &self.tiny, false, None, true);
        self.f_name.draw(s, self.name_rect, &self.tiny, self.focus == 0, !self.err.is_empty() && self.focus == 0);
        self.f_id.draw(s, self.id_rect, &self.tiny, self.focus == 1, !self.err.is_empty() && self.focus == 1);
        btn(s, self.save_rect, &t("gd.ugc.btn_save"), &self.tiny, self.dirty, Some(b.accent), true);
        for key in TEXT_BUTTONS {
            let rc = self.edit_rect(key);
            draw::rect_r(s, p.btn, rc, 0, 7);
            draw::rect_r(s, b.accent, rc, 1, 7);
            let ic = Rect::new(rc.x + 2, rc.y, rc.h, rc.h);
            icon(s, ic, key, b.accent);
            let lab = t(&format!("gd.ed.btn_{key}"));
            let mut img = self.tiny.render(&lab, true, p.text);
            let room = rc.w - rc.h - 4;
            if img.w() > room {
                img = self.tiny.render(&kurz(&self.tiny, &lab, room, 1, "."), true, p.text);
            }
            s.blr(&img, img.get_rect().midleft_at((ic.right(), rc.centery())));
        }
        for key in ICON_BUTTONS {
            let rc = self.edit_rect(key);
            let on = (key == "grid" && self.grid) || (key == "settings" && self.settings_open);
            let enabled = match key {
                "undo" => !self.undo_stack.is_empty(),
                "redo" => !self.redo_stack.is_empty(),
                "delete" => self.sel.is_some(),
                _ => true,
            };
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 7);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 7);
            icon(s, rc, key, if on || enabled { p.text } else { p.text_faint });
        }
        let length = self.compiled().length;
        let info = format!("x {}  \u{00B7}  {}", (self.cam_x + 2.0) as i64, tf("gd.ugc.length", &[("n", &length.to_string())]));
        let img = self.tiny.render(&info, true, p.text_dim);
        let mut right = self.canvas.right();
        let set_cy = self.edit_rect("settings").centery();
        if !self.verified.is_empty() && self.verified == core::content_hash(&self.level_dict()) {
            let br = 6.max(self.bh / 3);
            check_badge(s, (right - br) as f64, set_cy as f64, br as f64);
            right -= 2 * br + 6;
        }
        if self.pos_x + img.w() <= right {
            s.blr(&img, img.get_rect().midright_at((right, set_cy)));
        }
        if !self.err.is_empty() {
            let e = self.tiny.render(&self.err, true, p.red);
            let bx = e.get_rect().topleft_at((self.canvas.x + 6, self.canvas.y + 4));
            draw::rect_r(s, (0, 0, 0), bx.inflate(8, 4), 0, 4);
            s.blr(&e, bx);
        } else {
            let text = self.tool_label();
            let img = self.tiny.render(&text, true, b.accent);
            let bx = img.get_rect().topright_at((self.canvas.right() - 8, self.canvas.y + 5));
            draw::rect_r(s, (0, 0, 0), bx.inflate(12, 6), 0, 6);
            draw::rect_r(s, b.accent, bx.inflate(12, 6), 1, 6);
            s.blr(&img, bx);
        }
    }

    /// `tool_label()`
    pub fn tool_label(&self) -> String {
        if TOOLS.contains(&self.tool) {
            return t(&format!("gd.ed.tool.{}", self.tool));
        }
        format!("{}  \u{00B7}  {}", t(&format!("gd.ed.obj.{}", self.tool)), t(&format!("gd.ed.group.{}", self.group)))
    }

    fn draw_palette(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        for (key, rc) in self.tool_rects.clone() {
            let on = self.tool == key;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 6);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 6);
            icon(s, rc, key, if on { p.text } else { p.text_dim });
        }
        for (key, rc) in self.group_rects.clone() {
            let on = self.group == key;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 6);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 6);
            self.draw_item_icon(s, rc, group_icon(key), 0, true);
        }
        let top = self.group_rects.last().map(|g| g.1.bottom()).unwrap_or(0) + 2;
        draw::line(s, p.border, (self.px0, top + 1), (self.px0 + self.pal_w, top + 1), 1);
        for (key, rc) in self.item_rects() {
            let on = self.tool == key;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 6);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 6);
            let rot = if on && (core::ROTATE_ALL.contains(&key) || core::ROTATE_FLIP.contains(&key)) { self.rot } else { 0 };
            self.draw_item_icon(s, rc, key, rot, false);
        }
    }

    fn draw_item_icon(&mut self, s: &mut Surface, rc: Rect, kind: &'static str, rot: i64, small: bool) {
        let size = 8.max(rc.w.min(rc.h) - if small { 12 } else { 8 });
        let key = (kind, rot, size, rc.w, rc.h);
        if let std::collections::hash_map::Entry::Vacant(eintrag) = self.icon_cache.entry(key) {
            let mut tmp = surface_alpha(rc.w, rc.h);
            let mut raw = vec![json!(kind), json!(0), json!(0), json!(rot)];
            raw.extend(default_param(kind).into_iter().map(|v| json!(v)));
            let mut d0 = Map::new();
            d0.insert("objects".into(), json!([raw]));
            d0.insert("length".into(), json!(50));
            let mut d = core::normalize_level(&d0);
            if kind == "color" {
                if let Some(o) = d.get_mut("objects").and_then(|v| v.as_array_mut()).and_then(|a| a.get_mut(0)).and_then(|v| v.as_array_mut()) {
                    for (j, v) in [255, 90, 210].iter().enumerate() {
                        if 5 + j < o.len() {
                            o[5 + j] = json!(v);
                        }
                    }
                }
            }
            let lv = Level::new(&d);
            gdraw::block_masks(&lv);
            let used = std::collections::BTreeSet::new();
            if kind == "pit" {
                draw::rect(&mut tmp, (4, 2, 8), ((rc.w - size) / 2, rc.h / 2, size, rc.h / 2 - 3), 0);
                draw::line(&mut tmp, (255, 80, 80), ((rc.w - size) / 2, rc.h / 2), ((rc.w + size) / 2, rc.h / 2), 2);
            } else if kind.starts_with("p_") || kind.starts_with("g_") || kind.starts_with("s_") {
                let mut big = Renderer::new();
                big.resize(rc.w, rc.h, 6.max(size / 3 + 2) as f64);
                let bts = big.ts as f64;
                big.draw_objects(&mut tmp, &lv, -(rc.w as f64 - bts) / 2.0 / bts, -(rc.h as f64 - bts) / 2.0 / bts, 0.0, 0.0, &used, true, None);
            } else {
                let mut ir = Renderer::new();
                ir.resize(rc.w, rc.h, size as f64);
                let sz = size as f64;
                let cam_x = -(rc.w as f64 - sz) / 2.0 / sz;
                let cam_y = -(rc.h as f64 - sz) / 2.0 / sz;
                ir.draw_objects(&mut tmp, &lv, cam_x, cam_y, 0.3, 0.0, &used, true, None);
            }
            eintrag.insert(tmp);
        }
        s.bl(&self.icon_cache[&key], (rc.x, rc.y));
    }

    fn draw_bottom(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let rects = self.param_rects(b.width);
        if !rects.is_empty() {
            let (o, _params) = self.params().expect("params");
            let o = o.clone();
            for &(i, minus, val, plus) in &rects {
                btn(s, minus, "-", &self.tiny, false, None, true);
                btn(s, plus, "+", &self.tiny, false, None, true);
                draw::rect_r(s, p.panel, val, 0, 7);
                draw::rect_r(s, p.border, val, 1, 7);
                let v = o.v[3 + i];
                let txt = if o.kind == "color" && i == 0 {
                    if v == 0 {
                        t("gd.ed.target_bg")
                    } else {
                        t("gd.ed.target_ground")
                    }
                } else if o.kind != "color" && v == 0 {
                    (core::CORRIDOR / core::B).to_string()
                } else {
                    v.to_string()
                };
                let mut img = self.tiny.render(&txt, true, p.text);
                if img.w() > val.w - 4 {
                    img = ui::font(9.max(b.height / 56), false, false).render(&txt, true, p.text);
                }
                s.blr(&img, img.get_rect().center_at(val.center()));
                let labs = param_labels(o.kind);
                if let Some(lab) = labs.get(i).filter(|l| !l.is_empty()) {
                    let lab = if lab.starts_with("gd.") { t(lab) } else { lab.to_string() };
                    let li = self.par_font.render(&lab, true, p.text_faint);
                    s.blr(&li, li.get_rect().midbottom_at((val.centerx(), val.y - 1)));
                }
            }
            if o.kind == "color" {
                let r0 = rects[0].1;
                let sw = Rect::new(rects[rects.len() - 1].3.right() + 8, r0.y, r0.h, r0.h);
                if sw.right() < self.canvas.right() {
                    draw::rect_r(s, rgb(o.v[4], o.v[5], o.v[6]), sw, 0, 5);
                    draw::rect_r(s, p.border, sw, 1, 5);
                }
            }
        } else {
            let hint = match self.tool {
                "select" => t("gd.ed.hint_select"),
                "erase" => t("gd.ed.hint_erase"),
                _ => t("gd.ed.hint_place"),
            };
            let mut img = self.tiny.render(&hint, true, p.text_faint);
            if img.w() > self.canvas.w {
                img = ui::font(9.max(b.height / 52), false, false).render(&hint, true, p.text_faint);
            }
            s.blr(&img, img.get_rect().center_at((self.canvas.centerx(), self.par_top + self.par_h / 2)));
        }
        let mr = self.map_rect;
        draw::rect_r(s, p.panel, mr, 0, 4);
        let cam_x = self.cam_x;
        let view = self.canvas.w as f64 / self.ts as f64;
        let accent = b.accent;
        let lv = self.compiled();
        let end = core::MIN_LENGTH.max(lv.length) as f64;
        for i in 0..lv.n {
            let cls = lv.cls[i];
            if cls == core::C_TRIGGER {
                continue;
            }
            let x = mr.x + (lv.ox[i] as f64 / end * mr.w as f64) as i32;
            let y = mr.bottom() - 2 - (1f64.min(lv.oy[i] as f64 / 12.0) * (mr.h - 4) as f64) as i32;
            let col = if cls == core::C_SOLID {
                rgb(230, 230, 240)
            } else if cls == core::C_HAZARD || cls == core::C_PIT {
                rgb(255, 90, 90)
            } else {
                accent
            };
            s.fill_r(col, (x, y, 2, 2));
        }
        let vx = mr.x + (0f64.max(cam_x) / end * mr.w as f64) as i32;
        let vw = 4.max((view / end * mr.w as f64) as i32);
        draw::rect_r(s, accent, (vx, mr.y, vw.min(mr.right() - vx), mr.h), 1, 3);
    }

    fn draw_settings(&mut self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let mut veil = surface_alpha(b.width, b.height);
        veil.fill_c(rgba(0, 0, 0, 160));
        s.bl(&veil, (0, 0));
        let r = self.set_rect;
        ui::draw_panel_ex(s, r, None, None, None, true, Some(b.accent));
        let head = self.small.render(&t("gd.ed.settings"), true, b.accent);
        s.blr(&head, head.get_rect().midtop_at((r.centerx(), r.y + 10)));
        for (key, rects) in self.set_rows.clone() {
            let full = t(&format!("gd.ed.set_{key}"));
            let mut lab = self.tiny.render(&full, true, p.text_dim);
            if lab.w() > self.lab_w - 6 {
                lab = self.tiny.render(&kurz(&self.tiny, &full, self.lab_w - 6, 2, "."), true, p.text_dim);
            }
            s.blr(&lab, lab.get_rect().midleft_at((r.x + 14, rects[0].centery())));
            if key == "bg" || key == "ground" {
                let cur: Vec<i64> = self.meta.get(key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or_default();
                for (j, rc) in rects.iter().enumerate() {
                    let pr = PRESETS[j];
                    draw::rect_r(s, rgb(pr.0, pr.1, pr.2), *rc, 0, 4);
                    let on = cur == [pr.0, pr.1, pr.2];
                    draw::rect_r(s, if on { b.accent } else { p.border }, *rc, if on { 2 } else { 1 }, 4);
                }
                continue;
            }
            btn(s, rects[0], "<", &self.tiny, false, None, true);
            btn(s, rects[2], ">", &self.tiny, false, None, true);
            let txt = match key {
                "speed" => core::SPEED_LABELS[self.meta_i("speed").clamp(0, 3) as usize].to_string(),
                "mode" => t(&format!("gd.ed.mode.{}", core::MODE_NAMES[self.meta_i("mode").clamp(0, 4) as usize])),
                "music" => t(&format!("gd.ed.music.{}", self.meta.get("music").and_then(|v| v.as_str()).unwrap_or("drive"))),
                _ => {
                    let bpm = music::style_bpm(&self.meta);
                    if bpm != 0 {
                        bpm.to_string()
                    } else {
                        "-".to_string()
                    }
                }
            };
            btn(s, rects[1], &txt, &self.tiny, true, Some(b.accent), true);
        }
        btn(s, self.set_close, &t("gd.ed.close"), &self.tiny, false, None, true);
    }
}
