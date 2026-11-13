//! minigolf_edit.rs
//! ================
//! Eigene Minigolf-Bahnen: der MAPS-Reiter ([`MapList`]) und der Bahn-Editor
//! ([`MapEditor`]) - uebertragen aus `games/minigolf_edit.py`.
//!
//! Beide haengen an `MiniGolfGame` und zeichnen auf dessen Flaeche. Was
//! Python ueber Rueckrufe ins Spiel erledigt (Editor oeffnen/schliessen,
//! Bahn spielen, Test-Spielen), liefern die Rust-Fassungen als
//! [`EditAction`] zurueck; das Spiel fuehrt sie danach aus.

use pgz_core::swear;
use pgz_engine::filepick;
use pgz_engine::prelude::*;
use pgz_engine::ui::TextInput;
use serde_json::{json, Map, Value};

use crate::minigolf_draw::{self as mdraw, View};
use crate::minigolf_gen::{self as gen, hole_with, Hole, BALL_R, BORDER};
use crate::ugc;

/// Was das Spiel nach einer Eingabe tun soll.
pub enum EditAction {
    /// Nichts.
    None,
    /// Neue Bahn anlegen und in den Editor.
    NewMap,
    /// Diese Bahn bearbeiten.
    Edit(Map<String, Value>),
    /// Genau diese eigene Bahn spielen.
    Play(String),
    /// Editor schliessen (zurueck in den MAPS-Reiter).
    Close,
    /// Bahn ausprobieren.
    Test(Hole),
}

type Rgb = (i32, i32, i32);

const TOOLS: [&str; 4] = ["select", "erase", "tee", "cup"];
const DIRS: [(f64, f64); 8] = [(0.0, -1.0), (0.7071, -0.7071), (1.0, 0.0), (0.7071, 0.7071), (0.0, 1.0), (-0.7071, 0.7071), (-1.0, 0.0), (-0.7071, -0.7071)];

fn dir_index(dx: f64, dy: f64) -> usize {
    let (mut best, mut bd) = (0, -2.0);
    let n0 = py::hypot(dx, dy);
    let n = if n0 != 0.0 { n0 } else { 1.0 };
    for (i, (ux, uy)) in DIRS.iter().enumerate() {
        let d = (dx / n) * ux + (dy / n) * uy;
        if d > bd {
            best = i;
            bd = d;
        }
    }
    best
}

/// Ein einstellbarer Wert eines Hindernisses.
#[derive(Clone, Copy)]
struct Param {
    key: &'static str,
    idx: i64,
    lo: f64,
    hi: f64,
    step: f64,
    kind: &'static str,
}

const fn p(key: &'static str, idx: i64, lo: f64, hi: f64, step: f64, kind: &'static str) -> Param {
    Param { key, idx, lo, hi, step, kind }
}

const WH: Param = p("w", 2, 4.0, 160.0, 2.0, "num");
const HH: Param = p("h", 3, 4.0, 240.0, 2.0, "num");
const DIR: Param = p("dir", 4, 0.0, 0.0, 1.0, "dir");

/// Die 15 Hindernis-Typen: (Schluessel, Art, Parameter).
const PALETTE: [(&str, &str, &[Param]); 15] = [
    ("walls", "rect", &[p("w", 2, 2.0, 160.0, 1.0, "num"), p("h", 3, 2.0, 240.0, 1.0, "num")]),
    ("sand", "rect", &[WH, HH]),
    ("water", "rect", &[WH, HH]),
    ("ice", "rect", &[WH, HH]),
    ("sticky", "rect", &[WH, HH]),
    ("slopes", "rect", &[WH, HH, DIR, p("accel", -1, 6.0, 60.0, 2.0, "mag")]),
    ("boosters", "rect", &[WH, HH, DIR, p("boost", 6, 40.0, 200.0, 10.0, "num")]),
    ("gates", "rect", &[WH, p("h", 3, 2.0, 240.0, 1.0, "num"), DIR]),
    ("jumps", "rect", &[WH, HH, DIR, p("dist", 6, 10.0, 90.0, 5.0, "num")]),
    ("movers", "rect", &[WH, p("h", 3, 2.0, 240.0, 1.0, "num"), p("dir", 4, 0.0, 0.0, 1.0, "movedir"), p("dist", -2, 6.0, 80.0, 2.0, "span"), p("speed", 6, 6.0, 60.0, 2.0, "num")]),
    ("bumpers", "circle", &[p("r", 2, 3.0, 16.0, 1.0, "num")]),
    ("magnets", "circle", &[p("r", 2, 8.0, 60.0, 2.0, "num"), p("force", 3, -260.0, 260.0, 20.0, "num")]),
    ("spinners", "circle", &[p("r", 2, 5.0, 30.0, 1.0, "num"), p("speed", 3, -5.0, 5.0, 0.5, "num")]),
    ("mills", "circle", &[p("len", 2, 5.0, 40.0, 1.0, "num"), p("arms", 3, 2.0, 4.0, 1.0, "num"), p("speed", 4, -3.0, 3.0, 0.2, "num")]),
    ("tunnels", "pair", &[p("r", 4, 3.0, 12.0, 1.0, "num")]),
];

fn palette_keys() -> impl DoubleEndedIterator<Item = &'static str> {
    PALETTE.iter().map(|e| e.0)
}

fn pal_of(key: &str) -> (&'static str, &'static [Param]) {
    PALETTE.iter().find(|e| e.0 == key).map(|e| (e.1, e.2)).unwrap_or(("rect", &[]))
}

fn swatch(key: &str) -> Rgb {
    match key {
        "walls" => mdraw::COL_WALL,
        "sand" => mdraw::COL_SAND,
        "water" => mdraw::COL_WATER,
        "ice" => mdraw::COL_ICE,
        "sticky" => mdraw::COL_STICKY,
        "slopes" => mdraw::COL_SLOPE,
        "boosters" => mdraw::COL_BOOST,
        "gates" => mdraw::COL_GATE,
        "jumps" => mdraw::COL_JUMP,
        "movers" => mdraw::COL_WALL_HI,
        "bumpers" => mdraw::COL_BUMPER,
        "magnets" => mdraw::COL_MAGNET,
        "spinners" => mdraw::COL_SPIN,
        "mills" => mdraw::COL_MILL,
        _ => mdraw::COL_TUNNEL,
    }
}

const ROUND_KEYS: [&str; 5] = ["bumpers", "magnets", "spinners", "mills", "tunnels"];

/// `new_item(key, x, y, w, h, x2, y2)`
fn new_item(key: &str, x: f64, y: f64, w: f64, h: f64, x2: f64, y2: f64) -> Vec<f64> {
    match key {
        "walls" | "sand" | "water" | "ice" | "sticky" => vec![x, y, w, h],
        "slopes" => vec![x, y, w, h, 0.0, 24.0],
        "boosters" => vec![x, y, w, h, 0.0, -1.0, 120.0],
        "gates" => vec![x, y, w, h, 0.0, -1.0],
        "jumps" => vec![x, y, w, h, 0.0, -1.0, 40.0],
        "movers" => vec![x, y, w, h, 30f64.min(w * 1.5), 0.0, 24.0],
        "bumpers" => vec![x, y, 6.0],
        "magnets" => vec![x, y, 24.0, 120.0],
        "spinners" => vec![x, y, 12.0, 2.0],
        "mills" => vec![x, y, 13.0, 2.0, 1.5],
        "tunnels" => vec![x, y, x2, y2, 5.0],
        _ => vec![x, y, w, h],
    }
}

// ---------------------------------------------------------------------------
//  Vorlagen
// ---------------------------------------------------------------------------

type It<'a> = (&'a str, &'a [&'a [f64]]);

fn tpl(par: i64, tee: (f64, f64), cup: (f64, f64), items: &[It]) -> Hole {
    hole_with(par, tee, cup, items)
}

const TEMPLATE_KEYS: [&str; 12] = ["blank", "tunnel", "island", "mill", "zigzag", "water", "bumper", "ramp", "ice", "maze", "magnet", "jump"];

/// `template(key)` - fertige Bahn einer Vorlage.
pub fn template(key: &str) -> Hole {
    match key {
        "tunnel" => tpl(3, (24.0, 146.0), (76.0, 24.0), &[("walls", &[&[6.0, 84.0, 62.0, 8.0]]), ("sand", &[&[70.0, 104.0, 22.0, 16.0]]), ("tunnels", &[&[22.0, 118.0, 30.0, 44.0, 5.0]])]),
        "island" => tpl(3, (50.0, 146.0), (50.0, 34.0), &[("sand", &[&[40.0, 96.0, 20.0, 14.0]]), ("water", &[&[6.0, 20.0, 26.0, 34.0], &[68.0, 20.0, 26.0, 34.0], &[6.0, 54.0, 24.0, 16.0], &[70.0, 54.0, 24.0, 16.0]])]),
        "mill" => tpl(3, (50.0, 146.0), (50.0, 18.0), &[("walls", &[&[22.0, 26.0, 8.0, 92.0], &[70.0, 26.0, 8.0, 92.0]]), ("mills", &[&[50.0, 92.0, 14.0, 2.0, 1.4]])]),
        "zigzag" => tpl(4, (18.0, 146.0), (82.0, 20.0), &[("walls", &[&[28.0, 118.0, 66.0, 7.0], &[6.0, 84.0, 66.0, 7.0], &[28.0, 50.0, 66.0, 7.0]])]),
        "water" => tpl(3, (50.0, 146.0), (50.0, 22.0), &[("sand", &[&[40.0, 116.0, 20.0, 14.0]]), ("water", &[&[6.0, 60.0, 30.0, 44.0], &[64.0, 60.0, 30.0, 44.0]])]),
        "bumper" => tpl(3, (50.0, 146.0), (50.0, 20.0), &[("bumpers", &[&[30.0, 100.0, 6.0], &[70.0, 100.0, 6.0], &[50.0, 70.0, 7.0], &[28.0, 44.0, 5.0], &[72.0, 44.0, 5.0]])]),
        "ramp" => tpl(3, (50.0, 148.0), (50.0, 20.0), &[("sand", &[&[18.0, 116.0, 22.0, 14.0], &[60.0, 116.0, 22.0, 14.0]]), ("slopes", &[&[20.0, 52.0, 60.0, 56.0, 0.0, 30.0]])]),
        "ice" => tpl(3, (50.0, 148.0), (50.0, 20.0), &[("walls", &[&[40.0, 90.0, 20.0, 8.0]]), ("ice", &[&[10.0, 44.0, 80.0, 74.0]]), ("sticky", &[&[38.0, 26.0, 24.0, 12.0]])]),
        "maze" => tpl(4, (16.0, 148.0), (84.0, 18.0), &[("walls", &[&[30.0, 122.0, 64.0, 7.0], &[6.0, 96.0, 64.0, 7.0], &[30.0, 70.0, 64.0, 7.0], &[6.0, 44.0, 64.0, 7.0]]), ("sand", &[&[74.0, 106.0, 18.0, 12.0]])]),
        "magnet" => tpl(3, (50.0, 146.0), (50.0, 22.0), &[("walls", &[&[46.0, 118.0, 8.0, 14.0]]), ("magnets", &[&[18.0, 96.0, 30.0, 140.0], &[82.0, 62.0, 30.0, -140.0]])]),
        "jump" => tpl(3, (50.0, 150.0), (50.0, 20.0), &[("walls", &[&[24.0, 92.0, 52.0, 8.0]]), ("sand", &[&[10.0, 104.0, 16.0, 14.0], &[74.0, 104.0, 16.0, 14.0]]), ("jumps", &[&[42.0, 124.0, 16.0, 10.0, 0.0, -1.0, 46.0]])]),
        _ => tpl(2, (50.0, 146.0), (50.0, 20.0), &[]),
    }
}

/// `validate(hole)` - "" oder ein Fehlerschluessel.
pub fn validate(hole: &Hole) -> &'static str {
    let (cw, ch) = (hole.w, hole.h);
    let lo = BORDER + BALL_R;
    for (px, py_) in [hole.tee, hole.cup] {
        if !(lo <= px && px <= cw - lo && lo <= py_ && py_ <= ch - lo) {
            return "bounds";
        }
    }
    let clear = BALL_R + 1.0;
    for key in ["walls", "water", "gates", "jumps"] {
        for r in hole.get(key) {
            for (px, py_) in [hole.tee, hole.cup] {
                if r[0] - clear < px && px < r[0] + r[2] + clear && r[1] - clear < py_ && py_ < r[1] + r[3] + clear {
                    return "blocked";
                }
            }
        }
    }
    for b in hole.get("bumpers") {
        for (px, py_) in [hole.tee, hole.cup] {
            if py::hypot(b[0] - px, b[1] - py_) < b[2] + clear {
                return "blocked";
            }
        }
    }
    if py::hypot(hole.tee.0 - hole.cup.0, hole.tee.1 - hole.cup.1) < 12.0 {
        return "tooclose";
    }
    ""
}

// ---------------------------------------------------------------------------
//  Gemeinsame Zeichenhelfer
// ---------------------------------------------------------------------------

fn shorten(fnt: &Font, text: &str, w: i32, min_len: usize) -> String {
    let mut kurz: Vec<char> = text.chars().collect();
    while kurz.len() > min_len && fnt.size(&(kurz.iter().collect::<String>() + "...")).0 > w {
        kurz.pop();
    }
    kurz.iter().collect::<String>() + "..."
}

fn btn(s: &mut Surface, rc: Rect, text: &str, fnt: &Font, on: bool, accent: Option<Color>, enabled: bool) -> Rect {
    let p = ui::pal();
    draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 7);
    draw::rect_r(s, if on { accent.unwrap_or(p.accent) } else { p.border }, rc, if on { 2 } else { 1 }, 7);
    let mut col = if enabled { p.text } else { p.text_faint };
    if on {
        col = p.text;
    }
    let mut img = fnt.render(text, true, col);
    if img.w() > rc.w - 8 {
        img = fnt.render(&shorten(fnt, text, rc.w - 8, 1), true, col);
    }
    s.blr(&img, img.get_rect().center_at(rc.center()));
    rc
}

fn arrow(s: &mut Surface, center: (f64, f64), dx: f64, dy: f64, size: f64, col: Color) {
    let n0 = py::hypot(dx, dy);
    let n = if n0 != 0.0 { n0 } else { 1.0 };
    let (ux, uy) = (dx / n, dy / n);
    let (cx, cy) = center;
    let tip = (cx + ux * size, cy + uy * size);
    let tail = (cx - ux * size, cy - uy * size);
    draw::line(s, col, tail, tip, 2);
    let head = size * 0.7;
    draw::polygon(s, col, &[tip, (tip.0 - ux * head - uy * head * 0.6, tip.1 - uy * head + ux * head * 0.6), (tip.0 - ux * head + uy * head * 0.6, tip.1 - uy * head - ux * head * 0.6)], 0);
}

fn tool_icon(s: &mut Surface, rc: Rect, key: &str, col: Color) {
    let (cx, cy) = rc.center();
    let r = 4.max(rc.w.min(rc.h) / 4);
    let (cxf, cyf, rf) = (cx as f64, cy as f64, r as f64);
    match key {
        "select" => {
            draw::polygon(s, col, &[(cxf - rf, cyf - rf), (cxf + rf, cyf), (cxf, cyf + rf * 0.3), (cxf + rf * 0.3, cyf + rf)], 2);
        }
        "erase" => {
            let bx = Rect::new(0, 0, r * 2, (rf * 1.5) as i32).center_at((cx, cy));
            draw::rect_r(s, col, bx, 2, 2);
            draw::line(s, col, (bx.x, bx.bottom()), (bx.right(), bx.y), 2);
        }
        "tee" => {
            draw::circle(s, col, (cx, cy), r, 2);
            draw::circle(s, col, (cx, cy), 1.max(r / 3), 0);
        }
        _ => {
            draw::circle(s, col, (cx + r / 2, cy + r), 2.max(r / 2), 0);
            draw::line(s, col, (cx + r / 2, cy + r), (cx + r / 2, cy - r), 2);
            draw::polygon(s, col, &[(cx + r / 2, cy - r), (cx - r, cy - r / 2), (cx + r / 2, cy)], 0);
        }
    }
}

fn edit_icon(s: &mut Surface, rc: Rect, key: &str, col: Color) {
    let (cx, cy) = rc.center();
    let r = 4.max(rc.w.min(rc.h) / 4);
    match key {
        "grid" => {
            let bx = Rect::new(0, 0, r * 2, r * 2).center_at((cx, cy));
            draw::rect(s, col, bx, 1);
            draw::line(s, col, (bx.centerx(), bx.y), (bx.centerx(), bx.bottom()), 1);
            draw::line(s, col, (bx.x, bx.centery()), (bx.right(), bx.centery()), 1);
        }
        "undo" | "redo" => {
            let sign = if key == "undo" { -1 } else { 1 };
            let rect = Rect::new(0, 0, r * 2, r * 2).center_at((cx, cy + r / 2));
            let (start, end) = if key == "undo" { (0.35, 2.9) } else { (0.25, 2.8) };
            draw::arc(s, col, rect, start, end, 2);
            let tipx = cx - sign * r;
            arrow(s, (tipx as f64, (cy + r / 2) as f64), 0.0, 1.0, 3.max(r / 2) as f64, col);
        }
        _ => {
            let d = r;
            draw::line(s, col, (cx - d, cy - d), (cx + d, cy + d), 2);
            draw::line(s, col, (cx + d, cy - d), (cx - d, cy + d), 2);
        }
    }
}

fn s_of<'a>(m: &'a Map<String, Value>, key: &str) -> &'a str {
    m.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

// ---------------------------------------------------------------------------
//  MAPS-Reiter
// ---------------------------------------------------------------------------

const LIST_BUTTONS: [&str; 6] = ["new", "edit", "play", "delete", "share", "import"];
const NEEDS_SEL: [&str; 4] = ["edit", "play", "delete", "share"];

/// Der MAPS-Reiter: Bahnen auswaehlen, bauen, spielen, teilen, importieren.
pub struct MapList {
    items: Vec<Map<String, Value>>,
    sel: usize,
    first: usize,
    toast: String,
    toast_t: f64,
    confirm: String,
    share: Option<Map<String, Value>>,
    f_author: TextInput,
    f_file: TextInput,
    focus: usize,
    err: String,
    fnt: Font,
    tiny: Font,
    row_h: i32,
    btn_rects: Vec<(&'static str, Rect)>,
    list_top: i32,
    list_bottom: i32,
    rows_visible: usize,
    list_w: i32,
    share_rect: Rect,
    author_rect: Rect,
    file_rect: Rect,
    share_btn: Vec<(&'static str, Rect)>,
    width: i32,
}

impl MapList {
    /// `MapList(game)`
    pub fn new(b: &GameBase, tab_bottom: i32) -> MapList {
        let z = Rect::new(0, 0, 0, 0);
        let mut ml = MapList {
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
            fnt: b.font.clone(),
            tiny: b.font.clone(),
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
        };
        ml.reload();
        ml.layout(b, tab_bottom);
        ml
    }

    /// `reload()`
    pub fn reload(&mut self) {
        self.items = ugc::load_maps("minigolf");
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
        self.fnt = ui::font(13.max(h / 30), false, false);
        self.tiny = ui::font(10.max(h / 40), false, false);
        self.row_h = 26.max(40.min(h / 11));
        let cols = if w >= 720 { 6 } else { 3 };
        let bh = 24.max(34.min(h / 13));
        let gap = 6;
        let rows = (LIST_BUTTONS.len() as i32 + cols - 1) / cols;
        let bw = (w - 24 - gap * (cols - 1)) as f64 / cols as f64;
        let bottom = h - 18;
        self.btn_rects.clear();
        for (i, key) in LIST_BUTTONS.iter().enumerate() {
            let (r, c) = (i as i32 / cols, i as i32 % cols);
            let y = bottom - (rows - r) * (bh + gap) + gap;
            self.btn_rects.push((key, Rect::new((12.0 + c as f64 * (bw + gap as f64)) as i32, y, bw as i32, bh)));
        }
        self.list_top = tab_bottom + 8;
        self.list_bottom = bottom - rows * (bh + gap) - 4;
        self.rows_visible = 1.max(py::fdiv32(self.list_bottom - self.list_top - 14, self.row_h)) as usize;
        self.list_w = w - 30;
        self.clamp();
        let pw = (w - 40).min(380);
        let fh = 22.max(30.min(h / 14));
        let ph = (h - 30).min(78 + 3 * fh + 62);
        self.share_rect = Rect::new(py::fdiv32(w - pw, 2), py::fdiv32(h - ph, 2), pw, ph);
        let fx = self.share_rect.x + 18;
        let fw = pw - 36;
        self.author_rect = Rect::new(fx, self.share_rect.y + 46, fw, fh);
        self.file_rect = Rect::new(fx, self.author_rect.bottom() + 24, fw, fh);
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

    /// `handle(event)`
    pub fn handle(&mut self, b: &mut GameBase, ev: &InputEvent) -> EditAction {
        if self.share.is_some() {
            self.handle_share(b, ev);
            return EditAction::None;
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
                    let rc = Rect::new(12, self.list_top + i as i32 * self.row_h, self.list_w, self.row_h - 3);
                    if rc.hit(ev.pos) {
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
        EditAction::None
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

    fn action(&mut self, b: &mut GameBase, key: &str) -> EditAction {
        if key != "delete" {
            self.confirm.clear();
        }
        if key == "new" {
            if ugc::is_full("minigolf") {
                self.toast(tf("golf.ugc.err.full", &[("max", &ugc::MAX_MAPS.to_string())]));
                return EditAction::None;
            }
            return EditAction::NewMap;
        }
        if key == "import" {
            self.do_import(b);
            return EditAction::None;
        }
        let Some(m) = self.selected().cloned() else {
            return EditAction::None;
        };
        let id = s_of(&m, "id").to_string();
        match key {
            "edit" => return EditAction::Edit(m),
            "play" => return EditAction::Play(id),
            "share" => self.open_share(b, m),
            "delete" => {
                if self.confirm != id {
                    self.confirm = id;
                    b.play_sound("select");
                    return EditAction::None;
                }
                ugc::delete_map(&id, "minigolf");
                self.confirm.clear();
                self.reload();
                self.toast(t("golf.ugc.deleted"));
                b.play_sound("click");
            }
            _ => {}
        }
        EditAction::None
    }

    fn open_share(&mut self, b: &GameBase, m: Map<String, Value>) {
        self.err.clear();
        self.focus = 0;
        let author = s_of(&m, "author").to_string();
        self.f_author.set_text(&if author.is_empty() { ugc::last_author() } else { author });
        let id = s_of(&m, "id").to_string();
        self.f_file.set_text(if id.is_empty() { "map" } else { &id });
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
            name = if s_of(&m, "id").is_empty() { "map".into() } else { s_of(&m, "id").to_string() };
        }
        if !author.is_empty() && !swear::is_clean(&author) {
            self.err = t("golf.ugc.err.swear");
            b.play_sound("hit");
            return;
        }
        let mut out = m.clone();
        if !author.is_empty() {
            out.insert("author".into(), json!(author));
            ugc::set_last_author(&author);
        }
        let filename = name + ugc::EXT;
        let path = if ask {
            if !filepick::available() {
                self.err = t("golf.ugc.err.nofile");
                return;
            }
            match filepick::save_as(&filename, &t("golf.ugc.share_title"), &[ugc::EXT, ".json"]) {
                Some(p) => p,
                None => return,
            }
        } else {
            filepick::to_downloads(&filename)
        };
        let (ok, why) = ugc::export_to(&out, &path, "minigolf");
        if !ok {
            self.err = t(&format!("golf.ugc.err.{}", if why == "swear" { "swear" } else { "io" }));
            b.play_sound("hit");
            return;
        }
        if !author.is_empty() && s_of(&m, "author") != author {
            ugc::save_map(&out, "minigolf");
            self.reload();
        }
        self.close_share();
        self.toast(tf("golf.ugc.exported", &[("path", &filepick::short(&path, 44))]));
        b.play_sound("point");
    }

    fn do_import(&mut self, b: &mut GameBase) {
        if ugc::is_full("minigolf") {
            self.toast(tf("golf.ugc.err.full", &[("max", &ugc::MAX_MAPS.to_string())]));
            return;
        }
        if !filepick::available() {
            self.toast(t("golf.ugc.err.nofile"));
            return;
        }
        let Some(path) = filepick::open_file(&t("golf.ugc.btn_import"), &[ugc::EXT, ".json"]) else {
            return;
        };
        let stem = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let wanted = ugc::slug(&stem);
        let (ok, why, m) = ugc::import_from(&path, Some("minigolf"));
        let Some(m) = m.filter(|_| ok) else {
            let why = if ["swear", "full", "format", "io"].contains(&why) { why } else { "format" };
            self.toast(t(&format!("golf.ugc.err.{why}")));
            b.play_sound("hit");
            return;
        };
        self.reload();
        let id = s_of(&m, "id").to_string();
        if let Some(i) = self.items.iter().position(|x| s_of(x, "id") == id) {
            self.sel = i;
            self.clamp();
        }
        if !wanted.is_empty() && id != wanted {
            self.toast(tf("golf.ugc.renamed", &[("id", &id)]));
        } else {
            self.toast(tf("golf.ugc.imported", &[("name", s_of(&m, "name"))]));
        }
        b.play_sound("point");
    }

    /// `draw(s)`
    pub fn draw(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        if self.items.is_empty() {
            self.draw_empty(s, b);
        } else {
            for i in 0..self.rows_visible {
                let idx = self.first + i;
                if idx >= self.items.len() {
                    break;
                }
                self.draw_row(s, b, &self.items[idx], idx, Rect::new(12, self.list_top + i as i32 * self.row_h, self.list_w, self.row_h - 3));
            }
            if self.items.len() > self.rows_visible {
                self.draw_scrollbar(s);
            }
        }
        let cnt = self.tiny.render(&tf("golf.ugc.count", &[("n", &self.items.len().to_string()), ("max", &ugc::MAX_MAPS.to_string())]), true, p.text_dim);
        s.bl(&cnt, (12, self.list_bottom + 1));
        let has = self.selected().is_some();
        for &(key, rc) in &self.btn_rects {
            btn(s, rc, &t(&format!("golf.ugc.btn_{key}")), &self.tiny, false, None, has || !NEEDS_SEL.contains(&key));
        }
        if !self.toast.is_empty() {
            self.draw_toast(s);
        }
        if self.share.is_some() {
            self.draw_share(s, b);
        }
    }

    fn draw_empty(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let bx = Rect::new(24, self.list_top + 10, b.width - 48, 60.max(self.list_bottom - self.list_top - 24));
        ui::draw_panel_ex(s, bx, None, None, None, false, None);
        for (i, line) in [t("golf.ugc.empty"), t("golf.ugc.empty2")].iter().enumerate() {
            let img = self.fnt.render(line, true, if i == 0 { p.text_dim } else { p.text_faint });
            s.blr(&img, img.get_rect().center_at((bx.centerx(), bx.centery() - 12 + i as i32 * 22)));
        }
    }

    fn draw_row(&self, s: &mut Surface, b: &GameBase, m: &Map<String, Value>, idx: usize, rc: Rect) {
        let p = ui::pal();
        let sel = idx == self.sel;
        draw::rect_r(s, if sel { p.panel_light } else { p.panel }, rc, 0, 7);
        draw::rect_r(s, if sel { b.accent } else { p.border }, rc, if sel { 2 } else { 1 }, 7);
        let pad = 10;
        let name = self.fnt.render(s_of(m, "name"), true, p.text);
        s.bl(&name, (rc.x + pad, rc.y + 3));
        let num = |k: &str, d: f64| m.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
        let mut sub = format!(
            "{}  \u{00B7}  {}  \u{00B7}  {}",
            s_of(m, "id"),
            tf("golf.ugc.size", &[("w", &(num("w", gen::CW) as i64).to_string()), ("h", &(num("h", gen::CH) as i64).to_string())]),
            tf("golf.par", &[("n", &(num("par", 3.0) as i64).to_string())])
        );
        if !s_of(m, "author").is_empty() {
            sub += &("  \u{00B7}  ".to_string() + &tf("golf.ugc.by", &[("name", s_of(m, "author"))]));
        }
        let img = self.tiny.render(&sub, true, p.text_dim);
        if rc.h >= 32 {
            s.bl(&img, (rc.x + pad, rc.bottom() - img.h() - 3));
        }
        if self.confirm == s_of(m, "id") {
            let warn = self.tiny.render(&t("golf.ugc.confirm_delete"), true, p.red);
            s.blr(&warn, warn.get_rect().midright_at((rc.right() - pad, rc.centery())));
        }
    }

    fn draw_scrollbar(&self, s: &mut Surface) {
        let p = ui::pal();
        let track = Rect::new(self.list_w + 14, self.list_top, 4, self.rows_visible as i32 * self.row_h);
        draw::rect_r(s, p.panel, track, 0, 2);
        let frac = self.rows_visible as f64 / self.items.len() as f64;
        let hh = 24.max((track.h as f64 * frac) as i32);
        let pos = self.first as f64 / 1.max(self.items.len() - self.rows_visible) as f64;
        let y = track.y + ((track.h - hh) as f64 * 1f64.min(pos)) as i32;
        draw::rect_r(s, p.border_light, (track.x, y, 4, hh), 0, 2);
    }

    fn draw_toast(&self, s: &mut Surface) {
        let p = ui::pal();
        let img = self.tiny.render(&self.toast, true, p.text);
        let (w, h) = (img.w() + 26, img.h() + 12);
        let r = Rect::new(self.width / 2 - w / 2, self.list_bottom - h - 2, w, h);
        ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
        draw::rect_r(s, p.accent2, r, 1, h / 2);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn draw_share(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let mut veil = pg::surface_alpha(b.width, b.height);
        veil.fill_c(rgba(0, 0, 0, 150));
        s.bl(&veil, (0, 0));
        let r = self.share_rect;
        ui::draw_panel_ex(s, r, None, None, None, true, Some(b.accent));
        let head = self.fnt.render(&t("golf.ugc.share_title"), true, b.accent);
        s.blr(&head, head.get_rect().midtop_at((r.centerx(), r.y + 10)));
        for (i, (rect, key)) in [(self.author_rect, "golf.ugc.creator"), (self.file_rect, "golf.ugc.filename")].iter().enumerate() {
            let lab = self.tiny.render(&t(key), true, p.text_dim);
            s.bl(&lab, (rect.x, rect.y - lab.h() - 2));
            let field = if i == 0 { &self.f_author } else { &self.f_file };
            field.draw(s, *rect, &self.tiny, self.focus == i, false);
        }
        let ext = self.tiny.render(ugc::EXT, true, p.text_faint);
        s.bl(&ext, (self.file_rect.right() - ext.w() - 8, self.file_rect.centery() - ext.h() / 2));
        btn(s, self.sbtn("as"), &t("golf.ugc.export_as"), &self.tiny, false, None, true);
        btn(s, self.sbtn("dl"), &t("golf.ugc.export_dl"), &self.tiny, false, None, true);
        btn(s, self.sbtn("cancel"), &t("golf.ugc.btn_cancel"), &self.tiny, false, None, true);
        if !self.err.is_empty() {
            let img = self.tiny.render(&self.err, true, p.red);
            s.blr(&img, img.get_rect().midbottom_at((r.centerx(), r.bottom() - 6)));
        }
    }
}

// ---------------------------------------------------------------------------
//  Der Bahn-Editor
// ---------------------------------------------------------------------------

const TEXT_BUTTONS: [&str; 2] = ["template", "test"];
const ICON_BUTTONS: [&str; 4] = ["grid", "undo", "redo", "clear"];
const UNDO_MAX: usize = 40;

/// Bahn bauen: Leinwand links, Palette rechts, Parameter unten.
pub struct MapEditor {
    map: Map<String, Value>,
    hole: Hole,
    tool: &'static str,
    sel: Option<(&'static str, usize)>,
    drag: Option<(&'static str, f64, f64, f64, f64)>,
    mv: Option<(&'static str, usize, f64, f64)>,
    pending: Option<(f64, f64)>,
    undo_stack: Vec<Hole>,
    redo_stack: Vec<Hole>,
    dirty: bool,
    err: String,
    toast: String,
    toast_t: f64,
    picking: bool,
    confirm_back: bool,
    t: f64,
    grid: bool,
    f_name: TextInput,
    f_id: TextInput,
    focus: i32,
    orig_id: String,
    tiny: Font,
    small: Font,
    back_rect: Rect,
    name_rect: Rect,
    id_rect: Rect,
    save_rect: Rect,
    num_rects: Vec<(&'static str, (Rect, Rect, Rect))>,
    edit_rects: Vec<(&'static str, Rect)>,
    head_bottom: i32,
    par_h: i32,
    par_top: i32,
    pal_rects: Vec<(&'static str, Rect)>,
    scale: f64,
    ox: f64,
    oy: f64,
    canvas: Rect,
    tpl_rect: Rect,
    tpl_rects: Vec<(&'static str, Rect)>,
    width: i32,
}

fn key_static(key: &str) -> &'static str {
    gen::HOLE_LISTS.iter().copied().find(|k| *k == key).unwrap_or("walls")
}

impl MapEditor {
    /// `MapEditor(game, m)`
    pub fn new(b: &GameBase, m: Map<String, Value>) -> MapEditor {
        let z = Rect::new(0, 0, 0, 0);
        let hole = Hole::from_json(&Value::Object(ugc::golf_normalize(&m)));
        let grid = match b.opt_value("minigolf", "grid") {
            None => true,
            Some(Value::Bool(v)) => v,
            Some(Value::Null) => false,
            Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
            Some(Value::String(s)) => !s.is_empty(),
            Some(_) => true,
        };
        let mut ed = MapEditor {
            f_name: TextInput::new(s_of(&m, "name"), ugc::MAX_NAME, None, &t("golf.ugc.name")),
            f_id: TextInput::new(s_of(&m, "id"), ugc::MAX_ID, Some(TextInput::ID_CHARS), &t("golf.ugc.id")),
            orig_id: s_of(&m, "id").to_string(),
            map: m,
            hole,
            tool: "walls",
            sel: None,
            drag: None,
            mv: None,
            pending: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            dirty: false,
            err: String::new(),
            toast: String::new(),
            toast_t: 0.0,
            picking: false,
            confirm_back: false,
            t: 0.0,
            grid,
            focus: -1,
            tiny: b.font.clone(),
            small: b.font.clone(),
            back_rect: z,
            name_rect: z,
            id_rect: z,
            save_rect: z,
            num_rects: Vec::new(),
            edit_rects: Vec::new(),
            head_bottom: 0,
            par_h: 0,
            par_top: 0,
            pal_rects: Vec::new(),
            scale: 1.0,
            ox: 0.0,
            oy: 0.0,
            canvas: z,
            tpl_rect: z,
            tpl_rects: Vec::new(),
            width: b.width,
        };
        ed.layout(b);
        ed
    }

    /// `layout()`
    pub fn layout(&mut self, b: &GameBase) {
        let (w, h) = (b.width, b.height);
        self.width = w;
        self.tiny = ui::font(10.max(h / 42), false, false);
        self.small = ui::font(12.max(h / 34), false, false);
        let bh = 20.max(28.min(h / 15));
        let gap = 5;
        let y = 6;
        let back_w = 48.max((w as f64 * 0.11) as i32);
        let save_w = 62.max((w as f64 * 0.16) as i32);
        let field_w = (w - 24 - back_w - save_w - 3 * gap) as f64 / 2.0;
        self.back_rect = Rect::new(12, y, back_w, bh);
        self.name_rect = Rect::new(12 + back_w + gap, y, field_w as i32, bh);
        self.id_rect = Rect::new(self.name_rect.right() + gap, y, field_w as i32, bh);
        self.save_rect = Rect::new(self.id_rect.right() + gap, y, save_w, bh);
        let y2 = y + bh + gap;
        let step_w = 16.max((w as f64 * 0.036) as i32);
        let val_w = 30.max((w as f64 * 0.062) as i32);
        let grp = 2 * step_w + val_w;
        self.num_rects.clear();
        let mut x = 12;
        for key in ["par", "width", "height"] {
            self.num_rects.push((key, (Rect::new(x, y2, step_w, bh), Rect::new(x + step_w, y2, val_w, bh), Rect::new(x + step_w + val_w, y2, step_w, bh))));
            x += grp + gap;
        }
        let rest = w - 12 - x;
        let icon_w = 22.max(30.min(bh + 2));
        let icons = ICON_BUTTONS.len() as i32;
        let text_area = rest - icons * (icon_w + gap);
        let tbw = 34f64.max((text_area - gap * TEXT_BUTTONS.len() as i32) as f64 / TEXT_BUTTONS.len() as f64);
        self.edit_rects.clear();
        for (i, key) in TEXT_BUTTONS.iter().enumerate() {
            self.edit_rects.push((key, Rect::new((x as f64 + i as f64 * (tbw + gap as f64)) as i32, y2, tbw as i32, bh)));
        }
        let x2 = x as f64 + TEXT_BUTTONS.len() as f64 * (tbw + gap as f64);
        for (i, key) in ICON_BUTTONS.iter().enumerate() {
            self.edit_rects.push((key, Rect::new((x2 + i as f64 * (icon_w + gap) as f64) as i32, y2, icon_w, bh)));
        }
        self.head_bottom = y2 + bh + 5;
        self.par_h = bh + 16;
        self.par_top = h - self.par_h;
        let cols = 2;
        let n = (PALETTE.len() + TOOLS.len()) as i32;
        let rows = (n + cols - 1) / cols;
        let avail = self.par_top - self.head_bottom - 6;
        let pbh = 16.max(30.min(py::fdiv32(avail - (rows - 1) * 3, rows)));
        let pbw = 34.max(52.min((w as f64 * 0.085) as i32));
        let pal_w = cols * pbw + 3;
        let px0 = w - pal_w - 6;
        self.pal_rects.clear();
        for (i, key) in TOOLS.iter().copied().chain(palette_keys()).enumerate() {
            let (r, c) = (i as i32 / cols, i as i32 % cols);
            self.pal_rects.push((key, Rect::new(px0 + c * (pbw + 3), self.head_bottom + r * (pbh + 3), pbw, pbh)));
        }
        let (cw, ch) = (self.hole.w, self.hole.h);
        let area_w = (px0 - 18) as f64;
        let area_h = avail as f64;
        self.scale = 0.8f64.max((area_w / cw).min(area_h / ch));
        self.ox = 12.0 + (area_w - cw * self.scale) / 2.0;
        self.oy = self.head_bottom as f64 + (area_h - ch * self.scale) / 2.0;
        self.canvas = Rect::new(self.ox as i32, self.oy as i32, (cw * self.scale) as i32, (ch * self.scale) as i32);
        let pw = (w - 30).min(420);
        let tcols = 3;
        let trows = (TEMPLATE_KEYS.len() as i32 + tcols - 1) / tcols;
        let tbh = 22.max(32.min(py::fdiv32(h - 90, trows + 1)));
        let ph = 44 + trows * (tbh + 6) + 10;
        self.tpl_rect = Rect::new(py::fdiv32(w - pw, 2), 6.max(py::fdiv32(h - ph, 2)), pw, ph);
        let tbw = (pw - 24 - 6 * (tcols - 1)) as f64 / tcols as f64;
        self.tpl_rects = TEMPLATE_KEYS
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let (r, c) = (i as i32 / tcols, i as i32 % tcols);
                (*key, Rect::new((self.tpl_rect.x as f64 + 12.0 + c as f64 * (tbw + 6.0)) as i32, self.tpl_rect.y + 38 + r * (tbh + 6), tbw as i32, tbh))
            })
            .collect();
    }

    fn view(&self) -> View {
        View { ox: self.ox, oy: self.oy, scale: self.scale }
    }

    fn push(&mut self) {
        self.undo_stack.push(self.hole.clone());
        if self.undo_stack.len() > UNDO_MAX {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.dirty = true;
        self.err.clear();
    }

    fn undo(&mut self, b: &GameBase) {
        let Some(h) = self.undo_stack.pop() else {
            return;
        };
        self.redo_stack.push(std::mem::replace(&mut self.hole, h));
        self.sel = None;
        self.dirty = true;
        self.layout(b);
        b.play_sound("move");
    }

    fn redo(&mut self, b: &GameBase) {
        let Some(h) = self.redo_stack.pop() else {
            return;
        };
        self.undo_stack.push(std::mem::replace(&mut self.hole, h));
        self.sel = None;
        self.dirty = true;
        self.layout(b);
        b.play_sound("move");
    }

    fn toast(&mut self, text: String) {
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

    fn to_course(&self, pos: (i32, i32)) -> (f64, f64) {
        let x = (pos.0 as f64 - self.ox) / self.scale;
        let y = (pos.1 as f64 - self.oy) / self.scale;
        (self.snap(x), self.snap(y))
    }

    fn snap(&self, v: f64) -> f64 {
        if self.grid {
            py::round(v) as f64
        } else {
            py::round_to(v, 1)
        }
    }

    fn clampx(&self, x: f64) -> f64 {
        BORDER.max((self.hole.w - BORDER).min(x))
    }

    fn clampy(&self, y: f64) -> f64 {
        BORDER.max((self.hole.h - BORDER).min(y))
    }

    fn pick(&self, x: f64, y: f64) -> Option<(&'static str, usize)> {
        for key in palette_keys().rev() {
            let items = self.hole.get(key);
            for i in (0..items.len()).rev() {
                let it = &items[i];
                let hit = match key {
                    "bumpers" | "magnets" | "spinners" | "mills" => py::hypot(x - it[0], y - it[1]) <= 3f64.max(it[2]),
                    "tunnels" => py::hypot(x - it[0], y - it[1]) <= 3f64.max(it[4]) || py::hypot(x - it[2], y - it[3]) <= 3f64.max(it[4]),
                    _ => it[0] <= x && x <= it[0] + it[2] && it[1] <= y && y <= it[1] + it[3],
                };
                if hit {
                    return Some((key, i));
                }
            }
        }
        None
    }

    /// `handle(event)`
    pub fn handle(&mut self, b: &mut GameBase, ev: &InputEvent) -> EditAction {
        if self.picking {
            self.handle_template(b, ev);
            return EditAction::None;
        }
        match ev.kind {
            Kind::KeyDown => self.handle_key(b, ev),
            Kind::MouseDown => self.handle_down(b, ev),
            Kind::MouseMove => {
                if let Some(d) = self.drag {
                    let (x, y) = self.to_course(ev.pos);
                    self.drag = Some((d.0, d.1, d.2, self.clampx(x), self.clampy(y)));
                } else if self.mv.is_some() {
                    self.do_move(ev.pos);
                }
                EditAction::None
            }
            Kind::MouseUp => {
                self.handle_up(b);
                EditAction::None
            }
            _ => EditAction::None,
        }
    }

    fn handle_key(&mut self, b: &mut GameBase, ev: &InputEvent) -> EditAction {
        let k = ev.key.as_str();
        if self.focus >= 0 {
            let consumed = if self.focus == 0 { self.f_name.handle(ev) } else { self.f_id.handle(ev) };
            if consumed {
                self.dirty = true;
                self.err.clear();
                return EditAction::None;
            }
            if k == "Tab" || k == "ISO_Left_Tab" {
                self.focus = 1 - self.focus;
                return EditAction::None;
            }
            if ["Return", "KP_Enter", "Escape"].contains(&k) {
                self.focus = -1;
                return EditAction::None;
            }
        }
        match k {
            "Escape" => return self.back(b),
            "Delete" | "BackSpace" => self.delete_selected(b),
            "u" | "U" => self.undo(b),
            "y" | "Y" => self.redo(b),
            "g" | "G" => self.toggle_grid(b),
            "s" | "S" => {
                self.save(b);
            }
            "Return" | "KP_Enter" => return self.test(b),
            _ => {}
        }
        EditAction::None
    }

    fn handle_down(&mut self, b: &mut GameBase, ev: &InputEvent) -> EditAction {
        let pos = ev.pos;
        if self.name_rect.hit(pos) {
            self.focus = 0;
            return EditAction::None;
        }
        if self.id_rect.hit(pos) {
            self.focus = 1;
            return EditAction::None;
        }
        self.focus = -1;
        if self.back_rect.hit(pos) {
            return self.back(b);
        }
        if self.save_rect.hit(pos) {
            self.save(b);
            return EditAction::None;
        }
        for (key, (minus, _val, plus)) in self.num_rects.clone() {
            if minus.hit(pos) {
                self.step_num(b, key, -1);
                return EditAction::None;
            }
            if plus.hit(pos) {
                self.step_num(b, key, 1);
                return EditAction::None;
            }
        }
        if let Some(&(key, _)) = self.edit_rects.iter().find(|(_, r)| r.hit(pos)) {
            match key {
                "template" => {
                    self.picking = true;
                    b.play_sound("click");
                }
                "grid" => self.toggle_grid(b),
                "undo" => self.undo(b),
                "redo" => self.redo(b),
                "clear" => self.clear(b),
                _ => return self.test(b),
            }
            return EditAction::None;
        }
        if let Some(&(key, _)) = self.pal_rects.iter().find(|(_, r)| r.hit(pos)) {
            self.tool = key;
            self.pending = None;
            self.sel = None;
            b.play_sound("select");
            return EditAction::None;
        }
        if self.param_click(b, pos) {
            return EditAction::None;
        }
        if self.canvas.hit(pos) {
            self.canvas_down(b, pos);
        }
        EditAction::None
    }

    fn canvas_down(&mut self, b: &GameBase, pos: (i32, i32)) {
        let (x, y) = self.to_course(pos);
        let (x, y) = (self.clampx(x), self.clampy(y));
        let tool = self.tool;
        match tool {
            "tee" => {
                self.push();
                self.hole.tee = (x, y);
                b.play_sound("click");
            }
            "cup" => {
                self.push();
                self.hole.cup = (x, y);
                b.play_sound("click");
            }
            "erase" => {
                if let Some((k, i)) = self.pick(x, y) {
                    self.push();
                    self.hole.get_mut(k).remove(i);
                    self.sel = None;
                    b.play_sound("hit");
                }
            }
            "select" => {
                let hit = self.pick(x, y);
                self.sel = hit;
                if let Some((k, i)) = hit {
                    let it = &self.hole.get(k)[i];
                    self.mv = Some((k, i, it[0] - x, it[1] - y));
                    b.play_sound("move");
                }
            }
            _ => {
                let key = key_static(tool);
                match pal_of(tool).0 {
                    "circle" => {
                        self.push();
                        self.hole.get_mut(key).push(new_item(key, x, y, 0.0, 0.0, 0.0, 0.0));
                        self.sel = Some((key, self.hole.get(key).len() - 1));
                        b.play_sound("click");
                    }
                    "pair" => match self.pending {
                        None => {
                            self.pending = Some((x, y));
                            b.play_sound("select");
                        }
                        Some((px, py_)) => {
                            self.push();
                            self.hole.get_mut(key).push(new_item(key, px, py_, 0.0, 0.0, x, y));
                            self.sel = Some((key, self.hole.get(key).len() - 1));
                            self.pending = None;
                            b.play_sound("click");
                        }
                    },
                    _ => self.drag = Some((key, x, y, x, y)),
                }
            }
        }
    }

    fn do_move(&mut self, pos: (i32, i32)) {
        let Some((key, idx, offx, offy)) = self.mv else {
            return;
        };
        if idx >= self.hole.get(key).len() {
            self.mv = None;
            return;
        }
        let (x, y) = self.to_course(pos);
        let mut it = self.hole.get(key)[idx].clone();
        let (nx, ny) = (self.clampx(x + offx), self.clampy(y + offy));
        if key == "tunnels" {
            let (dx, dy) = (nx - it[0], ny - it[1]);
            it[0] = nx;
            it[1] = ny;
            it[2] = self.clampx(it[2] + dx);
            it[3] = self.clampy(it[3] + dy);
        } else {
            it[0] = nx;
            it[1] = ny;
        }
        self.hole.get_mut(key)[idx] = it;
        self.dirty = true;
    }

    fn handle_up(&mut self, b: &GameBase) {
        if self.mv.is_some() {
            self.mv = None;
            return;
        }
        let Some((key, x0, y0, x1, y1)) = self.drag.take() else {
            return;
        };
        let (x, y) = (x0.min(x1), y0.min(y1));
        let (w, h) = ((x1 - x0).abs(), (y1 - y0).abs());
        if w < 2.0 || h < 2.0 {
            return;
        }
        self.push();
        self.hole.get_mut(key).push(new_item(key, x, y, w, h, 0.0, 0.0));
        self.sel = Some((key, self.hole.get(key).len() - 1));
        b.play_sound("click");
    }

    fn delete_selected(&mut self, b: &GameBase) {
        let Some((key, idx)) = self.sel else {
            return;
        };
        if idx < self.hole.get(key).len() {
            self.push();
            self.hole.get_mut(key).remove(idx);
        }
        self.sel = None;
        b.play_sound("hit");
    }

    fn step_num(&mut self, b: &GameBase, key: &str, d: i64) {
        self.push();
        match key {
            "par" => self.hole.par = ugc::MIN_PAR.max(ugc::MAX_PAR.min(self.hole.par + d)),
            "width" => {
                self.hole.w = ugc::MIN_W.max(ugc::MAX_W.min(self.hole.w + d as f64 * ugc::STEP_W));
                self.clamp_all();
                self.layout(b);
            }
            _ => {
                self.hole.h = ugc::MIN_H.max(ugc::MAX_H.min(self.hole.h + d as f64 * ugc::STEP_H));
                self.clamp_all();
                self.layout(b);
            }
        }
        b.play_sound("move");
    }

    fn clamp_all(&mut self) {
        let (cw, ch) = (self.hole.w, self.hole.h);
        self.hole.tee = (self.clampx(self.hole.tee.0), self.clampy(self.hole.tee.1));
        self.hole.cup = (self.clampx(self.hole.cup.0), self.clampy(self.hole.cup.1));
        for key in palette_keys() {
            let items = self.hole.get(key).clone();
            let mut kept = Vec::new();
            for mut it in items {
                if ROUND_KEYS.contains(&key) {
                    if key == "tunnels" {
                        it[0] = self.clampx(it[0]);
                        it[1] = self.clampy(it[1]);
                        it[2] = self.clampx(it[2]);
                        it[3] = self.clampy(it[3]);
                    } else {
                        it[0] = self.clampx(it[0]);
                        it[1] = self.clampy(it[1]);
                    }
                } else {
                    it[2] = it[2].min(cw - 2.0 * BORDER);
                    it[3] = it[3].min(ch - 2.0 * BORDER);
                    it[0] = BORDER.max((cw - BORDER - it[2]).min(it[0]));
                    it[1] = BORDER.max((ch - BORDER - it[3]).min(it[1]));
                }
                kept.push(it);
            }
            *self.hole.get_mut(key) = kept;
        }
    }

    fn toggle_grid(&mut self, b: &mut GameBase) {
        self.grid = !self.grid;
        let g = self.grid;
        b.set_opt("minigolf", "grid", json!(g));
        b.play_sound("select");
    }

    fn clear(&mut self, b: &GameBase) {
        self.push();
        for key in palette_keys() {
            self.hole.get_mut(key).clear();
        }
        self.sel = None;
        b.play_sound("hit");
    }

    fn handle_template(&mut self, b: &GameBase, ev: &InputEvent) {
        if ev.kind == Kind::MouseDown {
            if let Some(&(key, _)) = self.tpl_rects.iter().find(|(_, r)| r.hit(ev.pos)) {
                self.apply_template(b, key);
                return;
            }
            self.picking = false;
            return;
        }
        if ev.kind == Kind::KeyDown && ev.key == "Escape" {
            self.picking = false;
        }
    }

    fn apply_template(&mut self, b: &GameBase, key: &str) {
        self.push();
        let tp = template(key);
        self.hole.par = tp.par;
        self.hole.tee = tp.tee;
        self.hole.cup = tp.cup;
        self.hole.w = tp.w;
        self.hole.h = tp.h;
        for k in palette_keys() {
            *self.hole.get_mut(k) = tp.get(k).clone();
        }
        self.sel = None;
        self.picking = false;
        self.layout(b);
        self.toast(t(&format!("golf.ugc.tpl.{key}")));
        b.play_sound("point");
    }

    fn params(&mut self) -> Option<(&'static str, Vec<f64>, &'static [Param])> {
        let (key, idx) = self.sel?;
        if idx >= self.hole.get(key).len() {
            self.sel = None;
            return None;
        }
        Some((key, self.hole.get(key)[idx].clone(), pal_of(key).1))
    }

    fn params_ro(&self) -> Option<(&'static str, Vec<f64>, &'static [Param])> {
        let (key, idx) = self.sel?;
        let it = self.hole.get(key).get(idx)?;
        Some((key, it.clone(), pal_of(key).1))
    }

    fn param_rects(&self) -> Vec<(Param, Rect, Rect, Rect)> {
        let Some((_, _, params)) = self.params_ro() else {
            return Vec::new();
        };
        if params.is_empty() {
            return Vec::new();
        }
        let w = self.width;
        let bh = self.par_h - 10;
        let step_w = 16.max((w as f64 * 0.036) as i32);
        let val_w = 34.max((w as f64 * 0.07) as i32);
        let grp = 2 * step_w + val_w;
        let n = params.len() as i32;
        let total = n * grp + (n - 1) * 8;
        let mut x = 10.max(py::fdiv32(w - total, 2));
        let y = self.par_top + 5;
        let mut out = Vec::new();
        for pp in params {
            out.push((*pp, Rect::new(x, y, step_w, bh), Rect::new(x + step_w, y, val_w, bh), Rect::new(x + step_w + val_w, y, step_w, bh)));
            x += grp + 8;
        }
        out
    }

    fn param_click(&mut self, b: &GameBase, pos: (i32, i32)) -> bool {
        let _ = self.params();
        for (pp, minus, _val, plus) in self.param_rects() {
            if minus.hit(pos) {
                self.step_param(b, pp, -1);
                return true;
            }
            if plus.hit(pos) {
                self.step_param(b, pp, 1);
                return true;
            }
        }
        false
    }

    fn step_param(&mut self, b: &GameBase, pp: Param, d: i64) {
        let Some((key, item, _)) = self.params() else {
            return;
        };
        self.push();
        let mut it = item;
        let df = d as f64;
        match pp.kind {
            "dir" | "movedir" => {
                let pi = pp.idx as usize;
                let i = py::pmod(dir_index(it[pi], it[pi + 1]) as i64 + d, DIRS.len() as i64) as usize;
                let (ux, uy) = DIRS[i];
                if pp.kind == "movedir" {
                    let s0 = py::hypot(it[4], it[5]);
                    let span = if s0 != 0.0 { s0 } else { 20.0 };
                    it[4] = ux * span;
                    it[5] = uy * span;
                } else {
                    it[pi] = ux;
                    it[pi + 1] = uy;
                }
            }
            "mag" | "span" => {
                let (ax, ay) = (it[4], it[5]);
                let n0 = py::hypot(ax, ay);
                let n = if n0 != 0.0 { n0 } else { 1.0 };
                let v = pp.lo.max(pp.hi.min(n + df * pp.step));
                it[4] = ax / n * v;
                it[5] = ay / n * v;
            }
            _ => {
                let i = pp.idx as usize;
                let v = pp.lo.max(pp.hi.min(it[i] + df * pp.step));
                it[i] = py::round_to(v, 2);
            }
        }
        if let Some((_, idx)) = self.sel {
            self.hole.get_mut(key)[idx] = it;
        }
        b.play_sound("move");
    }

    fn param_text(pp: &Param, item: &[f64]) -> Option<String> {
        match pp.kind {
            "dir" | "movedir" => None,
            "mag" | "span" => Some(py::round(py::hypot(item[4], item[5])).to_string()),
            _ => {
                let v = item[pp.idx as usize];
                if (v - py::round(v) as f64).abs() < 0.05 {
                    Some(py::round(v).to_string())
                } else {
                    Some(format!("{v:.1}"))
                }
            }
        }
    }

    fn save(&mut self, b: &GameBase) -> bool {
        let name = self.f_name.text.trim().to_string();
        let mut map_id = self.f_id.text.trim().to_string();
        if map_id.is_empty() {
            map_id = ugc::slug(&name);
        }
        if map_id.is_empty() {
            self.err = t("golf.ugc.err.id_empty");
            return self.fail(b);
        }
        if !ugc::valid_id(&map_id) {
            self.err = t("golf.ugc.err.id_chars");
            return self.fail(b);
        }
        if name.is_empty() {
            self.err = t("golf.ugc.err.name");
            return self.fail(b);
        }
        let taken = ugc::load_maps("minigolf").iter().any(|m| s_of(m, "id") != self.orig_id && s_of(m, "id") == map_id);
        if taken {
            self.err = t("golf.ugc.err.id_dup");
            return self.fail(b);
        }
        let bad = validate(&self.hole);
        if !bad.is_empty() {
            self.err = t(&format!("golf.ugc.err.{bad}"));
            return self.fail(b);
        }
        if !swear::all_clean(&[&name, &map_id]) {
            self.err = t("golf.ugc.err.swear");
            return self.fail(b);
        }
        let mut m = self.map.clone();
        for (k, v) in self.hole.to_json() {
            m.insert(k, v);
        }
        m.insert("id".into(), json!(map_id));
        m.insert("name".into(), json!(name));
        if !self.orig_id.is_empty() && self.orig_id != map_id {
            ugc::delete_map(&self.orig_id, "minigolf");
        }
        let (ok, why) = ugc::save_map(&m, "minigolf");
        if !ok {
            let why = if ["swear", "full", "io", "id"].contains(&why) { why } else { "invalid" };
            self.err = t(&format!("golf.ugc.err.{why}"));
            return self.fail(b);
        }
        self.map = m;
        self.orig_id = map_id.clone();
        self.dirty = false;
        self.f_id.set_text(&map_id);
        self.toast(t("golf.ugc.saved"));
        b.play_sound("win");
        true
    }

    fn fail(&self, b: &GameBase) -> bool {
        b.play_sound("hit");
        false
    }

    fn test(&mut self, b: &GameBase) -> EditAction {
        let bad = validate(&self.hole);
        if !bad.is_empty() {
            self.err = t(&format!("golf.ugc.err.{bad}"));
            self.fail(b);
            return EditAction::None;
        }
        EditAction::Test(self.hole.clone())
    }

    fn back(&mut self, b: &GameBase) -> EditAction {
        if self.dirty && !self.confirm_back {
            self.confirm_back = true;
            self.toast(t("golf.ugc.unsaved"));
            b.play_sound("select");
            return EditAction::None;
        }
        EditAction::Close
    }

    // ----- Zeichnen -----------------------------------------------------

    /// `draw(s)`
    pub fn draw(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        self.draw_canvas(s, b);
        self.draw_head(s, b);
        self.draw_palette(s, b);
        self.draw_params(s);
        if !self.toast.is_empty() {
            let img = self.tiny.render(&self.toast, true, p.text);
            let (w, h) = (img.w() + 24, img.h() + 10);
            let r = Rect::new(self.canvas.centerx() - w / 2, self.canvas.bottom() - h - 6, w, h);
            ui::draw_panel_ex(s, r, None, None, Some(h / 2), false, None);
            draw::rect_r(s, p.accent2, r, 1, h / 2);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        if self.picking {
            self.draw_template(s, b);
        }
    }

    fn draw_canvas(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        let view = self.view();
        mdraw::draw_course(s, &self.hole, &view, self.t, self.t, 3.0, true);
        if self.grid {
            self.draw_grid(s, &view);
        }
        if let Some((key, idx)) = self.sel {
            if let Some(it) = self.hole.get(key).get(idx) {
                self.outline(s, &view, key, it, b.accent);
            }
        }
        if let Some((key, x0, y0, x1, y1)) = self.drag {
            let rc = view.rect_px(&[x0.min(x1), y0.min(y1), 1f64.max((x1 - x0).abs()), 1f64.max((y1 - y0).abs())]);
            draw::rect_r(s, swatch(key), rc, 2, 3);
        }
        if let Some((px, py_)) = self.pending {
            let (px, py_) = view.project(px, py_);
            draw::circle(s, mdraw::COL_TUNNEL, (px as i32, py_ as i32), 4.max((5.0 * self.scale) as i32), 2);
        }
        draw::rect_r(s, p.border, self.canvas.inflate(8, 8), 1, 8);
    }

    fn draw_grid(&self, s: &mut Surface, view: &View) {
        let mut step = 10.0;
        while step * self.scale < 14.0 {
            step *= 2.0;
        }
        let col = ui::mix(mdraw::COL_GREEN, (255, 255, 255), 0.10);
        let mut x = step;
        while x < self.hole.w {
            let px = view.project(x, 0.0).0 as i32;
            draw::line(s, col, (px, self.canvas.y), (px, self.canvas.bottom()), 1);
            x += step;
        }
        let mut y = step;
        while y < self.hole.h {
            let py_ = view.project(0.0, y).1 as i32;
            draw::line(s, col, (self.canvas.x, py_), (self.canvas.right(), py_), 1);
            y += step;
        }
    }

    fn outline(&self, s: &mut Surface, view: &View, key: &str, it: &[f64], col: Color) {
        match key {
            "bumpers" | "magnets" | "spinners" | "mills" => {
                let (px, py_) = view.project(it[0], it[1]);
                draw::circle(s, col, (px as i32, py_ as i32), 4.max((it[2] * self.scale) as i32) + 3, 2);
            }
            "tunnels" => {
                for (cx, cy) in [(it[0], it[1]), (it[2], it[3])] {
                    let (px, py_) = view.project(cx, cy);
                    draw::circle(s, col, (px as i32, py_ as i32), 4.max((it[4] * self.scale) as i32) + 3, 2);
                }
            }
            _ => {
                draw::rect_r(s, col, view.rect_px(it).inflate(6, 6), 2, 4);
            }
        }
    }

    fn draw_head(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        btn(s, self.back_rect, &t("golf.ugc.btn_back"), &self.tiny, false, None, true);
        self.f_name.draw(s, self.name_rect, &self.tiny, self.focus == 0, !self.err.is_empty() && self.focus == 0);
        self.f_id.draw(s, self.id_rect, &self.tiny, self.focus == 1, !self.err.is_empty() && self.focus == 1);
        btn(s, self.save_rect, &t("golf.ugc.btn_save"), &self.tiny, self.dirty, Some(b.accent), true);
        for (key, (minus, val, plus)) in &self.num_rects {
            btn(s, *minus, "-", &self.tiny, false, None, true);
            btn(s, *plus, "+", &self.tiny, false, None, true);
            let num = match *key {
                "par" => self.hole.par,
                "width" => self.hole.w as i64,
                _ => self.hole.h as i64,
            };
            draw::rect_r(s, p.panel, *val, 0, 7);
            draw::rect_r(s, p.border, *val, 1, 7);
            let mut img = self.tiny.render(&format!("{} {}", t(&format!("golf.ugc.short_{key}")), num), true, p.text);
            if img.w() > val.w - 4 {
                img = self.tiny.render(&num.to_string(), true, p.text);
            }
            s.blr(&img, img.get_rect().center_at(val.center()));
        }
        let er = |k: &str| self.edit_rects.iter().find(|e| e.0 == k).map(|e| e.1).unwrap_or(Rect::new(0, 0, 0, 0));
        for key in TEXT_BUTTONS {
            btn(s, er(key), &t(&format!("golf.ugc.btn_{key}")), &self.tiny, false, Some(b.accent), true);
        }
        for key in ICON_BUTTONS {
            let rc = er(key);
            let on = key == "grid" && self.grid;
            let enabled = match key {
                "undo" => !self.undo_stack.is_empty(),
                "redo" => !self.redo_stack.is_empty(),
                _ => true,
            };
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 7);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 7);
            edit_icon(s, rc, key, if on || enabled { p.text } else { p.text_faint });
        }
        if !self.err.is_empty() {
            let img = self.tiny.render(&self.err, true, p.red);
            s.bl(&img, (12, self.head_bottom - 2));
        }
    }

    fn draw_palette(&self, s: &mut Surface, b: &GameBase) {
        let p = ui::pal();
        for &(key, rc) in &self.pal_rects {
            let on = self.tool == key;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 6);
            draw::rect_r(s, if on { b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 6);
            if TOOLS.contains(&key) {
                tool_icon(s, rc, key, if on { p.text } else { p.text_dim });
            } else {
                let inner = rc.inflate(-10, -10);
                let col = swatch(key);
                if ROUND_KEYS.contains(&key) {
                    draw::circle(s, col, inner.center(), 3.max(inner.w.min(inner.h) / 2), 0);
                } else {
                    draw::rect_r(s, col, inner, 0, 3);
                }
            }
        }
        let name = t(&format!("{}{}", if TOOLS.contains(&self.tool) { "golf.ugc.tool." } else { "golf.ugc.obj." }, self.tool));
        let img = self.tiny.render(&name, true, b.accent);
        if let Some(&(_, last)) = self.pal_rects.last() {
            s.blr(&img, img.get_rect().midbottom_at((last.centerx(), self.par_top - 2)));
        }
    }

    fn draw_params(&self, s: &mut Surface) {
        let p = ui::pal();
        let rects = self.param_rects();
        if rects.is_empty() {
            let hint = if self.pending.is_some() {
                t("golf.ugc.hint_tunnel")
            } else if self.tool == "select" {
                t("golf.ugc.hint_select")
            } else {
                t("golf.ugc.hint_place")
            };
            let img = self.tiny.render(&hint, true, p.text_faint);
            s.blr(&img, img.get_rect().center_at((self.width / 2, self.par_top + self.par_h / 2)));
            return;
        }
        let Some((_, item, _)) = self.params_ro() else {
            return;
        };
        for (pp, minus, val, plus) in rects {
            btn(s, minus, "-", &self.tiny, false, None, true);
            btn(s, plus, "+", &self.tiny, false, None, true);
            draw::rect_r(s, p.panel, val, 0, 7);
            draw::rect_r(s, p.border, val, 1, 7);
            match Self::param_text(&pp, &item) {
                None => {
                    let pi = pp.idx as usize;
                    let i = dir_index(item[pi], item[pi + 1]);
                    let c = val.center();
                    arrow(s, (c.0 as f64, c.1 as f64), DIRS[i].0, DIRS[i].1, 4.max(val.h / 4) as f64, p.text);
                }
                Some(txt) => {
                    let img = self.tiny.render(&txt, true, p.text);
                    s.blr(&img, img.get_rect().center_at(val.center()));
                }
            }
            let lab = self.tiny.render(&t(&format!("golf.ugc.par.{}", pp.key)), true, p.text_faint);
            s.blr(&lab, lab.get_rect().midbottom_at((val.centerx(), val.y - 1)));
        }
    }

    fn draw_template(&self, s: &mut Surface, b: &GameBase) {
        let mut veil = pg::surface_alpha(b.width, b.height);
        veil.fill_c(rgba(0, 0, 0, 160));
        s.bl(&veil, (0, 0));
        ui::draw_panel_ex(s, self.tpl_rect, None, None, None, true, Some(b.accent));
        let head = self.small.render(&t("golf.ugc.tpl_title"), true, b.accent);
        s.blr(&head, head.get_rect().midtop_at((self.tpl_rect.centerx(), self.tpl_rect.y + 10)));
        for &(key, rc) in &self.tpl_rects {
            btn(s, rc, &t(&format!("golf.ugc.tpl.{key}")), &self.tiny, false, None, true);
        }
    }
}
