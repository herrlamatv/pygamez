//! lamamines.rs
//! ============
//! Lama Mines - Edelstein-Suche mit Lama-Chips, uebertragen aus
//! `games/lamamines.py`. Auf einem Feld aus 4x4 bis 7x7 Steinquadern liegen
//! 1 bis N-1 Minen, unter allen anderen ein Edelstein. Jeder gefundene
//! Edelstein hebt den Multiplikator, eine Mine kostet den Einsatz. Auszahlen
//! ab dem ersten Edelstein; alle Edelsteine bzw. der Hoechstgewinn x100.000
//! zahlen automatisch aus. Regeln: [`crate::lamamines_logic`], Zeichnen:
//! [`crate::lamamines_draw`].
//!
//! - Fairness (Fenster F): Pruefcode vor der Runde, Feld aus Server-Seed,
//!   Client-Seed und Rundennummer (`pgz_core::fairness`); die letzten 12
//!   Runden lassen sich nachrechnen (mit Mini-Vorschau).
//! - Statistik (Fenster S): Kacheln + Bilanzkurve (`pgz_core::chipstats`).
//! - Auto-Modus (Fenster A): Felder markieren (je Feldgroesse gemerkt), jede
//!   Runde deckt genau diese auf und zahlt aus, wenn alle sicher sind.
//!
//! Konto: gemeinsame Lama-Bank. Einsatz bei SPIELEN abbuchen; die Runde wird
//! bei jedem aufgedeckten Feld gespeichert - wer mitten in der Runde geht,
//! macht beim naechsten Start genau dort weiter (Feld aus dem Seed).
//! Alle Tasten werden roh ausgewertet (nie ueber die WASD-Belegung).
//!
//! Python-Sonderfaelle, die Rust nicht nachbildet: Optionen, die in
//! settings.json als Kommazahl oder bool statt als ganze Zahl stehen (Python
//! wuerde `5.0` als Feldgroesse annehmen) - Rust nimmt dann die Vorgabe.

use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_core::chipstats::{self, ChipStats};
use pgz_core::{fairness, i18n, lamabank};
use pgz_engine::prelude::*;
use serde_json::{Map, Value};

use crate::casino_draw as cd;
use crate::lamamines_draw as d;
use crate::lamamines_logic as ml;

const KEY: &str = ml::KEY;

const FLIP_T: f64 = 0.24; // Aufklappen eines Feldes
const POP_T: f64 = 0.32; // Edelstein federt nach dem Aufklappen
const WAVE_T: f64 = 0.05; // Rest-Aufdeckung: Verzoegerung je Feld Abstand
const BOOM_T: f64 = 0.62; // Explosion -> Ergebnis-Tafel
const RESULT_T: f64 = 2.6; // Ergebnis-Tafel, danach Ruhezustand
const READ_T: f64 = 0.35; // Ergebnis fruehestens danach wegdrueckbar
const SHAKE_T: f64 = 0.55; // Kamera-Wackeln nach einer Mine
const AUTO_STEP_T: f64 = 0.26; // Auto: Abstand zwischen zwei Feldern
const AUTO_RESULT_T: f64 = 1.0; // Auto: Ergebnis-Anzeige
const AUTO_PAUSE_T: f64 = 0.35; // Auto: Pause vor der naechsten Runde
const TURBO_F: f64 = 0.4; // Turbo: alle Zeiten x 0,4
const BIG_MULT: i64 = 300; // ab x3: Muenzregen
const PARTY_MULT: i64 = 1000; // ab x10: Konfetti

const ROUND_OPTS: [i64; 5] = [5, 10, 25, 50, 100];
const LIMIT_OPTS: [i64; 7] = [0, 100, 500, 1000, 5000, 10000, 50000];
const TOOLS: [&str; 4] = ["auto", "stats", "fair", "turbo"];
/// Toene beim Finden: Dur-Pentatonik aufwaerts (Halbtoene ueber dem Grundton).
const GEM_STEPS: [i32; 16] = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28, 31, 33, 36];

fn tool_label(name: &str) -> String {
    t(&format!("chip.{name}"))
}

fn tool_key(name: &str) -> &'static str {
    match name {
        "auto" => "A",
        "stats" => "S",
        "fair" => "F",
        _ => "T",
    }
}

/// Pythons `isinstance(v, int) and not isinstance(v, bool)`.
fn py_int(v: Option<&Value>) -> Option<i64> {
    fairness::py_int(v)
}

/// Zahl fuer Pythons `==` (bool zaehlt als 0/1, 3.0 == 3).
fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}

/// Pythons Wahrheitswert eines JSON-Werts.
fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// `v in liste` mit Pythons Gleichheit (Zahlen).
fn in_list(v: &Value, list: &[i32]) -> bool {
    num(v).is_some_and(|x| list.iter().any(|f| *f as f64 == x))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Idle,
    Play,
    Boom,
    Cashed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Face {
    Hidden,
    Gem,
    Mine,
    Boom,
}

impl Face {
    fn name(self) -> &'static str {
        match self {
            Face::Hidden => "hidden",
            Face::Gem => "gem",
            Face::Mine => "mine",
            Face::Boom => "boom",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Win {
    Auto,
    Stats,
    Fair,
}

impl Win {
    fn name(self) -> &'static str {
        match self {
            Win::Auto => "auto",
            Win::Stats => "stats",
            Win::Fair => "fair",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mood {
    Idle,
    Happy,
    Cheer,
    Scared,
}

/// Anzeige-Zustand eines Platzes (`dict(face, flip, delay, dim, pop, mark, k)`).
#[derive(Clone, Copy)]
struct Cell {
    face: Face,
    flip: f64,
    delay: f64,
    dim: bool,
    pop: f64,
    mark: f64,
    k: i32,
}

impl Cell {
    fn blank() -> Cell {
        Cell { face: Face::Hidden, flip: 1.0, delay: 0.0, dim: false, pop: POP_T, mark: 0.0, k: 0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PKind {
    Fire,
    Smoke,
    Debris,
    Spark,
    Star,
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    r: f64,
    col: (i32, i32, i32),
    t: f64,
    life: f64,
    g: f64,
    kind: PKind,
}

struct CoinFx {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    spin: f64,
    vs: f64,
    r: f64,
}

struct Popup {
    x: i32,
    y: i32,
    text: String,
    color: Color,
    t: f64,
    dur: f64,
}

struct Flash {
    x: i32,
    y: i32,
    t: f64,
    dur: f64,
    r: i32,
    col: Option<(i32, i32, i32)>,
}

/// Laufender Auto-Modus (`dict(left, net, t, queue)`).
struct AutoRun {
    left: i64,
    net: i64,
    t: f64,
    queue: Vec<i32>,
}

#[derive(Clone, Copy)]
struct CashInfo {
    clear: bool,
    maxed: bool,
    m100: i64,
    shown: bool,
}

/// `LamaMinesGame`
pub struct LamaMinesGame {
    b: GameBase,
    rng: py::Random,
    grid: i32,
    mines: i32,
    bet: i64,
    turbo: bool,
    auto_rounds: i64,
    auto_win: i64,
    auto_loss: i64,
    hist: Vec<i64>,
    fair: fairness::State,
    fairlog: Vec<fairness::LogEntry>,
    stats: ChipStats,
    marks: ml::Marks,
    carry: i64,
    round: Option<ml::Round>,
    field: Vec<i32>,
    is_mine: Vec<bool>,
    cells: Vec<Cell>,
    hit: Option<usize>,
    state: State,
    result: Option<(String, String, Color)>,
    res_t: f64,
    res_delay: f64,
    boom_t: Option<f64>,
    win: i64,
    lost: i64,
    last_mult: i64,
    pending: i64,
    shown_chips: f64,
    particles: Vec<Particle>,
    coins_fx: Vec<CoinFx>,
    popups: Vec<Popup>,
    flashes: Vec<Flash>,
    toast: Option<(String, f64)>,
    banner: Option<(String, f64)>,
    shake: f64,
    time: f64,
    mouse: (i32, i32),
    broke: bool,
    window: Option<Win>,
    auto: Option<AutoRun>,
    cursor: i32,
    kb: bool,
    mood: (Mood, f64),
    sooty: bool,
    cart_fill: i32,
    fair_sel: Option<usize>,
    fair_scroll: i64,
    fair_checks: HashMap<i64, (bool, Option<Vec<i32>>)>,
    r_fair_rows: Vec<(usize, Rect)>,
    r_fair_new: Rect,
    r_fair_verify: Rect,
    cash_info: CashInfo,
    tick_cd: f64,
    lang: String,
    // ----- Layout
    k: f64,
    f_micro: Font,
    f_tiny: Font,
    f_small: Font,
    f_btn: Font,
    f_big: Font,
    f_mult: Font,
    f_huge: Font,
    f_mono: Font,
    margin: i32,
    hud_h: i32,
    strip_h: i32,
    tool_h: i32,
    strip: Rect,
    board_r: Rect,
    panel_r: Rect,
    hud_bg: Surface,
    strip_bg: Surface,
    overlay: Surface,
    board_pad: i32,
    cell: i32,
    tile_r: Vec<Rect>,
    tool_labels: bool,
    r_tools: Vec<(&'static str, Rect)>,
    tools_right: i32,
    r_minus: Rect,
    r_bet: Rect,
    r_plus: Rect,
    r_go: Rect,
    r_cash: Rect,
    random_label: bool,
    r_random: Rect,
    ppad: i32,
    r_mascot: Rect,
    cart_w: i32,
    head_x: i32,
    row_h: i32,
    lab_grid_y: i32,
    r_grids: Vec<Rect>,
    mines_row_y: i32,
    r_mval: Rect,
    r_mminus: Rect,
    r_mplus: Rect,
    r_presets: Vec<Rect>,
    info_y: i32,
    ladder_y: i32,
    ladder_lab_y: i32,
    r_pills: Vec<Rect>,
    hint_y: i32,
    panel_inner: (i32, i32),
    r_auto_close: Rect,
    auto_rows: Vec<(i32, Vec<Rect>)>,
    auto_info_y: i32,
    r_auto_start: Rect,
    r_auto_clear: Rect,
}

fn r0() -> Rect {
    Rect::new(0, 0, 0, 0)
}

impl LamaMinesGame {
    /// `LamaMinesGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = LamaMinesGame {
            b,
            rng: py::Random::new(0),
            grid: ml::DEFAULT_GRID,
            mines: ml::DEFAULT_MINES,
            bet: ml::BETS[0],
            turbo: false,
            auto_rounds: 10,
            auto_win: 0,
            auto_loss: 0,
            hist: Vec::new(),
            fair: fairness::State { server: String::new(), client: String::new(), nonce: 0 },
            fairlog: Vec::new(),
            stats: chipstats::blank(),
            marks: ml::Marks::default(),
            carry: 0,
            round: None,
            field: Vec::new(),
            is_mine: Vec::new(),
            cells: Vec::new(),
            hit: None,
            state: State::Idle,
            result: None,
            res_t: 0.0,
            res_delay: 0.0,
            boom_t: None,
            win: 0,
            lost: 0,
            last_mult: 0,
            pending: 0,
            shown_chips: 0.0,
            particles: Vec::new(),
            coins_fx: Vec::new(),
            popups: Vec::new(),
            flashes: Vec::new(),
            toast: None,
            banner: None,
            shake: 0.0,
            time: 0.0,
            mouse: (-1, -1),
            broke: false,
            window: None,
            auto: None,
            cursor: 0,
            kb: false,
            mood: (Mood::Idle, 0.0),
            sooty: false,
            cart_fill: 0,
            fair_sel: None,
            fair_scroll: 0,
            fair_checks: HashMap::new(),
            r_fair_rows: Vec::new(),
            r_fair_new: r0(),
            r_fair_verify: r0(),
            cash_info: CashInfo { clear: false, maxed: false, m100: 0, shown: true },
            tick_cd: 0.0,
            lang: String::new(),
            k: 1.0,
            f_micro: f.clone(),
            f_tiny: f.clone(),
            f_small: f.clone(),
            f_btn: f.clone(),
            f_big: f.clone(),
            f_mult: f.clone(),
            f_huge: f.clone(),
            f_mono: f,
            margin: 0,
            hud_h: 0,
            strip_h: 0,
            tool_h: 0,
            strip: r0(),
            board_r: r0(),
            panel_r: r0(),
            hud_bg: pg::surface_alpha(1, 1),
            strip_bg: pg::surface_alpha(1, 1),
            overlay: pg::surface_alpha(1, 1),
            board_pad: 0,
            cell: 0,
            tile_r: Vec::new(),
            tool_labels: false,
            r_tools: Vec::new(),
            tools_right: 0,
            r_minus: r0(),
            r_bet: r0(),
            r_plus: r0(),
            r_go: r0(),
            r_cash: r0(),
            random_label: false,
            r_random: r0(),
            ppad: 0,
            r_mascot: r0(),
            cart_w: 0,
            head_x: 0,
            row_h: 0,
            lab_grid_y: 0,
            r_grids: Vec::new(),
            mines_row_y: 0,
            r_mval: r0(),
            r_mminus: r0(),
            r_mplus: r0(),
            r_presets: Vec::new(),
            info_y: 0,
            ladder_y: 0,
            ladder_lab_y: 0,
            r_pills: Vec::new(),
            hint_y: 0,
            panel_inner: (0, 0),
            r_auto_close: r0(),
            auto_rows: Vec::new(),
            auto_info_y: 0,
            r_auto_start: r0(),
            r_auto_clear: r0(),
        };
        g.reset();
        Box::new(g)
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.k = (w as f64 / 800.0).min(h as f64 / 600.0);
        let k = self.k;
        self.f_micro = ui::font(10.max((11.0 * k) as i32), false, false);
        self.f_tiny = ui::font(11.max((13.0 * k) as i32), false, false);
        self.f_small = ui::font(12.max((15.0 * k) as i32), false, false);
        self.f_btn = ui::font(12.max((15.0 * k) as i32), true, false);
        self.f_big = ui::font(15.max((22.0 * k) as i32), true, false);
        self.f_mult = ui::font(18.max((32.0 * k) as i32), true, false);
        self.f_huge = ui::font(24.max((48.0 * k) as i32), true, false);
        self.f_mono = ui::font(10.max((12.0 * k) as i32), false, true);
        self.margin = 6.max((12.0 * k) as i32);
        let m = self.margin;
        self.hud_h = 28.max((42.0 * k) as i32);
        self.strip_h = 46.max((64.0 * k) as i32);
        self.tool_h = 26.max((34.0 * k) as i32);
        self.strip = Rect::new(0, h - self.strip_h, w, self.strip_h);
        let tool_y = self.hud_h + m;
        let area_y = tool_y + self.tool_h + m;
        let area_h = self.strip.y - m - area_y;
        let side = area_h.min((w as f64 * 0.58) as i32);
        self.board_r = Rect::new(m, area_y + py::fdiv32(area_h - side, 2), side, side);
        self.panel_r = Rect::new(self.board_r.right() + m, area_y, w - self.board_r.right() - 2 * m, area_h);
        self.hud_bg = pg::surface_alpha(w, self.hud_h);
        for y in 0..self.hud_h {
            let a = (210.0 - 60.0 * y as f64 / 1.max(self.hud_h - 1) as f64) as i32;
            self.hud_bg.fill_r(rgba(10, 8, 16, a), (0, y, w, 1));
        }
        self.strip_bg = pg::surface_alpha(self.strip.w, self.strip.h);
        for y in 0..self.strip_h {
            let a = (170.0 + 60.0 * y as f64 / 1.max(self.strip_h - 1) as f64) as i32;
            self.strip_bg.fill_r(rgba(10, 8, 16, a), (0, y, w, 1));
        }
        self.overlay = pg::surface_alpha(w, h);
        self.overlay.fill_c(rgba(6, 4, 12, 200));
        self.layout_board();
        self.layout_tools(tool_y);
        self.layout_strip();
        self.layout_panel();
    }

    /// Felder im Holzrahmen: Rand, Abstand, Feldgroesse und Rechtecke.
    fn layout_board(&mut self) {
        let side = self.board_r.w;
        let n = self.grid;
        self.board_pad = 6.max((side as f64 * 0.035) as i32);
        let pad = self.board_pad;
        let inner = side - 2 * pad;
        let gap = 2.max((inner as f64 * 0.014) as i32);
        let cell = py::fdiv32(inner - gap * (n + 1), n);
        let total = n * cell + (n + 1) * gap;
        let off = py::fdiv32(inner - total, 2);
        let x0 = self.board_r.x + pad + off + gap;
        let y0 = self.board_r.y + pad + off + gap;
        self.cell = cell;
        self.tile_r = (0..n * n).map(|i| Rect::new(x0 + (i % n) * (cell + gap), y0 + (i / n) * (cell + gap), cell, cell)).collect();
    }

    /// Werkzeug-Knoepfe oben links (mit Beschriftung, wenn Platz ist).
    fn layout_tools(&mut self, y: i32) {
        let m = self.margin;
        let h = self.tool_h;
        let pad = 6.max((10.0 * self.k) as i32);
        let step = 3.max((5.0 * self.k) as i32);
        let widths: Vec<i32> = TOOLS.iter().map(|name| h + pad + self.f_tiny.width(&tool_label(name))).collect();
        let mut total = 0;
        for w in &widths {
            total += w + step;
        }
        self.tool_labels = total <= self.board_r.w;
        let mut x = m;
        self.r_tools = Vec::new();
        for (i, name) in TOOLS.iter().enumerate() {
            let w = if self.tool_labels { widths[i] } else { h };
            self.r_tools.push((*name, Rect::new(x, y, w, h)));
            x += w + step;
        }
        self.tools_right = x;
    }

    /// Bedienleiste: Einsatz links, Zufallsfeld Mitte, AUSZAHLEN + SPIELEN rechts.
    fn layout_strip(&mut self) {
        let (m, k, w) = (self.margin, self.k, self.b.width);
        let sh = self.strip_h;
        let bh = 30.max((sh as f64 * 0.62) as i32);
        let by = self.strip.y + py::fdiv32(sh - bh, 2);
        let lbl_w = self.f_tiny.width(&t("cas.bet")).max(self.f_btn.width("10000")) + (14.0 * k) as i32;
        self.r_minus = Rect::new(m, by, bh, bh);
        self.r_bet = Rect::new(self.r_minus.right() + 4, by, lbl_w, bh);
        self.r_plus = Rect::new(self.r_bet.right() + 4, by, bh, bh);
        let pad = (22.0 * 0.7f64.max(k)) as i32;
        let mut go_w = self.f_btn.width(&t("lmi.play")).max(self.f_btn.width(&t("chip.auto_stop"))) + pad;
        go_w = go_w.max((110.0 * k) as i32);
        let mut cash_w = self.f_btn.width(&format!("{} 99.999.999", t("lmi.cashout"))) + pad;
        let room = w - m - (self.r_plus.right() + m) - 4.max((8.0 * k) as i32) - bh - 2 * m;
        if go_w + cash_w > room {
            let f = room as f64 / (go_w + cash_w) as f64;
            go_w = (go_w as f64 * f) as i32;
            cash_w = (cash_w as f64 * f) as i32;
        }
        self.r_go = Rect::new(w - m - go_w, by, go_w, bh);
        self.r_cash = Rect::new(self.r_go.x - 4.max((8.0 * k) as i32) - cash_w, by, cash_w, bh);
        // Zufallsfeld: so breit wie moeglich zwischen Einsatz und AUSZAHLEN
        let (left, right) = (self.r_plus.right() + m, self.r_cash.x - m);
        let want = bh + self.f_btn.width(&t("lmi.random")) + pad;
        let rw = bh.max(want.min(right - left));
        self.random_label = rw >= want;
        self.r_random = Rect::new(0, by, rw, bh).with_centerx(py::fdiv32(left + right, 2));
    }

    /// Seitenleiste: Kopf (Maskottchen, Lore, Multiplikator), Einstellungen
    /// bzw. Gewinn-Info, Leiste der naechsten Stufen, Tastenhinweis.
    fn layout_panel(&mut self) {
        let p = self.panel_r;
        let k = self.k;
        self.ppad = 6.max((10.0 * k) as i32);
        let pad = self.ppad;
        let (x0, w) = (p.x + pad, p.w - 2 * pad);
        let gap = 4.max((7.0 * k) as i32);
        let ms = (w as f64 * 0.30).min(104.0 * k).min(p.h as f64 * 0.30) as i32;
        self.r_mascot = Rect::new(x0, p.y + pad, ms, ms);
        self.cart_w = (ms as f64 * 0.62) as i32;
        self.head_x = x0 + ms + (self.cart_w as f64 * 0.95) as i32 + gap;
        let mut y = self.r_mascot.bottom() + gap;
        self.row_h = 22.max((30.0 * k) as i32);
        let rh = self.row_h;
        let lab = self.f_tiny.get_height();
        // Einstellungen (nur im Ruhezustand)
        self.lab_grid_y = y;
        y += lab + 2;
        let bw = py::fdiv32(w - 3 * 4, 4);
        self.r_grids = (0..4).map(|i| Rect::new(x0 + i * (bw + 4), y, bw, rh)).collect();
        y += rh + gap;
        self.mines_row_y = y;
        let vw = (self.f_btn.width("48") + (16.0 * k) as i32).max(rh + 6);
        self.r_mval = Rect::new(x0 + w - rh - 4 - vw, y, vw, rh);
        self.r_mminus = Rect::new(self.r_mval.x - 4 - rh, y, rh, rh);
        self.r_mplus = Rect::new(self.r_mval.right() + 4, y, rh, rh);
        y += rh + 4;
        let npre = ml::mine_presets(4).len() as i32;
        let pw = py::fdiv32(w - (npre - 1) * 4, npre);
        let ph = 18.max((rh as f64 * 0.82) as i32);
        self.r_presets = (0..npre).map(|i| Rect::new(x0 + i * (pw + 4), y, pw, ph)).collect();
        self.info_y = self.lab_grid_y;
        // Leiste der naechsten Stufen + Hinweis am unteren Rand
        let hint_h = 2 * self.f_micro.get_height() + 2;
        let pill_h = 20.max((28.0 * k) as i32);
        self.ladder_y = p.bottom() - pad - hint_h - gap - pill_h;
        self.ladder_lab_y = self.ladder_y - lab - 2;
        let pw5 = py::fdiv32(w - 16, 5);
        self.r_pills = (0..5).map(|i| Rect::new(x0 + i * (pw5 + 4), self.ladder_y, pw5, pill_h)).collect();
        self.hint_y = p.bottom() - pad - hint_h;
        self.panel_inner = (x0, w);
        // Auto-Fenster liegt genau ueber der Seitenleiste
        self.layout_auto();
    }

    fn layout_auto(&mut self) {
        let p = self.panel_r;
        let k = self.k;
        let pad = self.ppad;
        let (x0, w) = (p.x + pad, p.w - 2 * pad);
        let gap = 3.max((6.0 * k) as i32);
        let lab = self.f_tiny.get_height();
        let rh = 20.max((28.0 * k) as i32);
        let cb = 18.max((24.0 * k) as i32);
        self.r_auto_close = Rect::new(p.right() - pad - cb, p.y + pad, cb, cb);
        let mut y = p.y + pad + cb.max(self.f_btn.get_height()) + gap;
        let mut rows = Vec::new();
        for opts in [&ROUND_OPTS[..], &LIMIT_OPTS[..], &LIMIT_OPTS[..]] {
            let ly = y;
            y += lab + 2;
            let n = opts.len() as i32;
            let mut bw = py::fdiv32(w - (n - 1) * 3, n);
            let mut first = bw;
            if opts[0] == 0 {
                // "aus" ist je Sprache verschieden lang - der erste Knopf waechst mit
                let off = t("chip.auto_off");
                first = bw.max(self.f_tiny.width(&off).min(self.f_micro.width(&off) + 4) + 8);
                bw = py::fdiv32(w - first - (n - 1) * 3, n - 1);
            }
            let mut rects = Vec::new();
            let mut x = x0;
            for i in 0..n {
                let rw = if i == 0 { first } else { bw };
                rects.push(Rect::new(x, y, rw, rh));
                x += rw + 3;
            }
            rows.push((ly, rects));
            y += rh + gap;
        }
        self.auto_rows = rows;
        self.auto_info_y = y;
        let bh = 24.max((32.0 * k) as i32);
        let mut start = Rect::new(0, p.bottom() - pad - bh, 0, bh);
        let cw = (self.f_btn.width(&t("lmi.clear_marks")) + (18.0 * k) as i32).max(bh);
        self.r_auto_clear = Rect::new(x0, start.y, cw, bh);
        start.x = self.r_auto_clear.right() + gap;
        start.w = x0 + w - start.x;
        self.r_auto_start = start;
    }

    // ===================================================== Einstellungen

    /// `_opt(key, ...)` als ganze Zahl (nur int, kein bool/float).
    fn opt_int(&self, key: &str) -> Option<i64> {
        py_int(self.b.opt_value(KEY, key).as_ref())
    }

    fn set_opt(&mut self, key: &str, value: Value) {
        if self.b.opt_value(KEY, key).as_ref() != Some(&value) {
            self.b.set_opt(KEY, key, value);
        }
    }

    /// Leertaste/Enter (roh, nie ueber is_action/WASD).
    fn is_action(key: &str) -> bool {
        matches!(key, "space" | "Return" | "KP_Enter")
    }

    // ===================================================== Konto / Speichern

    fn extra(&self) -> Value {
        let mut m = Map::new();
        m.insert("round".into(), self.round.as_ref().map(|r| r.to_value()).unwrap_or(Value::Null));
        m.insert("hist".into(), Value::from(self.hist.clone()));
        m.insert("fair".into(), self.fair.to_value());
        m.insert("fairlog".into(), fairness::log_to_value(&self.fairlog));
        m.insert("stats".into(), self.stats.to_value());
        m.insert("auto_marks".into(), self.marks.to_value());
        m.insert("carry".into(), Value::from(self.carry));
        Value::Object(m)
    }

    /// Auszahlung nach k Edelsteinen - mit dem gespeicherten Uebertrag.
    fn win_at(&self, bet: i64, k: i32) -> i64 {
        ml::payout_carry(bet, self.grid, self.mines, k, self.carry).0
    }

    fn save_extra(&self) {
        lamabank::set_extra(KEY, self.extra());
    }

    /// Einsatz auf die hoechste bezahlbare Stufe senken (mind. Stufe 1).
    fn clamp_bet(&mut self) {
        let chips = lamabank::balance();
        if self.bet > chips {
            let ok: Vec<i64> = ml::BETS.iter().copied().filter(|b| *b <= chips).collect();
            self.bet = ok.last().copied().unwrap_or(ml::BETS[0]);
        }
    }

    fn check_broke(&mut self) {
        self.broke = self.state == State::Idle && self.auto.is_none() && lamabank::is_broke(KEY, None);
        if self.broke {
            self.window = None;
            self.b.play_sound("gameover");
        }
    }

    fn take_credit(&mut self) {
        if lamabank::refill_if_broke(KEY, None) {
            self.b.play_sound("powerup");
            ui::spawn_burst((self.b.width / 2) as f64, (self.b.height / 2) as f64, Some(ui::pal().gold), 26, 260.0);
        }
        self.broke = false;
        self.shown_chips = lamabank::balance() as f64;
    }

    fn round_bet(&self) -> i64 {
        self.round.as_ref().map(|r| r.bet).unwrap_or(self.bet)
    }

    fn say(&mut self, text: String, secs: f64) {
        self.toast = Some((text, secs));
    }

    /// Zeitfaktor: Turbo macht alles schneller.
    fn tf(&self) -> f64 {
        if self.turbo {
            TURBO_F
        } else {
            1.0
        }
    }

    // ===================================================== Formatierung

    fn sep(&self) -> String {
        t("lmi.decimal")
    }

    /// Ganze Zahl mit Tausenderpunkten (bzw. -kommas, je Sprache).
    fn group(&self, n: i64) -> String {
        let sep = if self.sep() == "," { "." } else { "," };
        let mut s = n.unsigned_abs().to_string();
        let mut out = String::new();
        while s.len() > 3 {
            let cut = s.len() - 3;
            out = format!("{sep}{}{out}", &s[cut..]);
            s.truncate(cut);
        }
        format!("{}{}{}", if n < 0 { "-" } else { "" }, s, out)
    }

    /// Multiplikator x100 als Text mit Tausendertrennung: x1.234,56.
    fn fmt(&self, m100: i64) -> String {
        if m100 >= 1_000_000 {
            return format!("\u{00D7}{}", self.group(py::fdiv(m100, 100)));
        }
        format!("\u{00D7}{}{}{:02}", self.group(py::fdiv(m100, 100)), self.sep(), py::pmod(m100, 100))
    }

    /// Kurzform fuer die Stufen-Pillen: x1,23 / x123,4 / x1234 / x12,3k / x100k.
    fn fmt_short(&self, m100: i64) -> String {
        let sep = self.sep();
        if m100 < 10000 {
            return format!("\u{00D7}{}{}{:02}", py::fdiv(m100, 100), sep, py::pmod(m100, 100));
        }
        if m100 < 100000 {
            return format!("\u{00D7}{}{}{}", py::fdiv(m100, 100), sep, py::pmod(py::fdiv(m100, 10), 10));
        }
        if m100 < 1000000 {
            return format!("\u{00D7}{}", py::fdiv(m100, 100));
        }
        if m100 < 10000000 {
            return format!("\u{00D7}{}{}{}k", py::fdiv(m100, 100000), sep, py::pmod(py::fdiv(m100, 10000), 10));
        }
        format!("\u{00D7}{}k", py::fdiv(m100, 100000))
    }

    fn fmt_pct(&self, p10: i64) -> String {
        ml::fmt_percent(p10, &self.sep())
    }

    // ===================================================== Spielfeld

    fn blank_cells(&mut self) {
        let n = (self.grid * self.grid) as usize;
        self.cells = vec![Cell::blank(); n];
    }

    /// Leeres, verdecktes Feld der aktuellen Groesse (Ruhezustand).
    fn blank_board(&mut self) {
        self.field = Vec::new();
        self.is_mine = vec![false; (self.grid * self.grid) as usize];
        self.hit = None;
        self.blank_cells();
    }

    /// Minenfeld einer Runde aus ihren Fairness-Werten berechnen.
    fn new_board(&mut self, rnd: &ml::Round) {
        self.field = ml::field_from_seed(rnd.grid, rnd.mines, &rnd.s, &rnd.c, rnd.n);
        let n = (rnd.grid * rnd.grid) as usize;
        self.is_mine = vec![false; n];
        for &i in &self.field {
            self.is_mine[i as usize] = true;
        }
        self.hit = None;
        self.blank_cells();
    }

    fn kk(&self) -> i32 {
        self.round.as_ref().map(|r| r.open.len() as i32).unwrap_or(0)
    }

    /// Edelsteine in der Lore (0..7) - waechst mit dem Anteil gefundener Steine.
    fn cart_level(&self) -> i32 {
        let g = ml::gems(self.grid, self.mines);
        let k = self.kk();
        if k <= 0 {
            return 0;
        }
        1.max(7.min(py::fdiv32(k * 7 + g - 1, g)))
    }

    fn entry(&self) -> fairness::Entry {
        let r = self.round.as_ref().expect("Runde");
        fairness::Entry { n: r.n, s: r.s.clone(), c: r.c.clone(), h: r.h.clone() }
    }

    // ===================================================== Ablauf

    /// SPIELEN: Einsatz abbuchen, Minenfeld aus dem Fairness-Zufall ziehen.
    fn play(&mut self) -> bool {
        if self.state != State::Idle || self.broke || self.round.is_some() {
            return false;
        }
        let bet = self.bet;
        if bet > lamabank::balance() {
            self.say(t("cas.not_enough"), 1.8);
            self.b.play_sound("hit");
            return false;
        }
        let entry = fairness::begin(&self.fair);
        {
            let rng = &mut self.rng;
            fairness::advance(&mut self.fair, &mut |k| rng.getrandbits(k));
        }
        self.round = Some(ml::Round {
            bet,
            grid: self.grid,
            mines: self.mines,
            n: entry.n,
            s: entry.s.clone(),
            c: entry.c.clone(),
            h: entry.h.clone(),
            open: Vec::new(),
        });
        if !lamabank::bet_and_pay(bet, 0, KEY, Some((KEY, self.extra()))) {
            self.round = None;
            return false;
        }
        self.field = ml::make_field(&mut fairness::entry_rand(KEY, &entry), self.grid, self.mines);
        self.is_mine = vec![false; (self.grid * self.grid) as usize];
        for &i in &self.field {
            self.is_mine[i as usize] = true;
        }
        self.hit = None;
        self.blank_cells();
        for c in self.cells.iter_mut() {
            c.flip = 0.0; // Felder "wachsen" kurz ein
            c.face = Face::Hidden;
        }
        let (g, tf) = (self.grid, self.tf());
        for (i, c) in self.cells.iter_mut().enumerate() {
            let i = i as i32;
            c.delay = 0.012 * ((i % g) + (i / g)) as f64 * tf;
        }
        self.state = State::Play;
        self.result = None;
        self.sooty = false;
        self.cart_fill = 0;
        self.mood = (Mood::Idle, 0.0);
        self.win = 0;
        self.lost = 0;
        self.b.play_sound("select");
        self.b.score = lamabank::score_for(KEY);
        true
    }

    fn xint(&self, name: &str) -> i64 {
        py_int(self.stats.x.get(name)).filter(|v| *v >= 0).unwrap_or(0)
    }

    fn xset(&mut self, name: &str, v: i64) {
        self.stats.x.insert(name.into(), Value::from(v));
    }

    /// Ein Feld aufdecken (Klick, Leertaste, R oder Auto).
    fn reveal(&mut self, i: i32) {
        if self.state != State::Play || self.round.is_none() || !(0 <= i && (i as usize) < self.cells.len()) {
            return;
        }
        let iu = i as usize;
        if self.round.as_ref().is_some_and(|r| r.open.contains(&i)) || self.cells[iu].face != Face::Hidden {
            return;
        }
        {
            let c = &mut self.cells[iu];
            c.flip = 0.0;
            c.delay = 0.0;
            c.dim = false;
        }
        if self.is_mine[iu] {
            // Mine: sofort verbuchen (Verlassen hilft nicht), Explosion nach dem Aufklappen
            self.cells[iu].face = Face::Boom;
            self.hit = Some(iu);
            let r = self.round.as_ref().expect("Runde");
            let bet = r.bet;
            let mut opened = r.open.clone();
            opened.push(i);
            self.lost = bet;
            self.last_mult = 0;
            self.finish(0, &opened);
            self.state = State::Boom;
            self.boom_t = None;
            self.res_delay = FLIP_T * self.tf() + BOOM_T * self.tf();
            self.res_t = 0.0;
            self.result = Some((t("lmi.boom"), tf("lmi.lost", &[("n", &self.group(bet))]), ui::pal().red));
            let v = self.xint("booms") + 1;
            self.xset("booms", v);
            self.save_extra();
            self.b.play_sound("move");
            return;
        }
        self.cells[iu].face = Face::Gem;
        let k = {
            let r = self.round.as_mut().expect("Runde");
            r.open.push(i);
            r.open.len() as i32
        };
        self.cells[iu].k = k;
        let v = self.xint("gems") + 1;
        self.xset("gems", v);
        self.b.play_sound("move");
        if ml::is_clear(self.grid, self.mines, k) || ml::is_max(self.grid, self.mines, k) {
            self.cash_out(true);
        } else {
            self.save_extra();
        }
    }

    /// Rundenende verbuchen: Statistik, Verlauf, Fairness-Log, Runde loeschen.
    fn finish(&mut self, paid: i64, opened: &[i32]) {
        let r = self.round.as_ref().expect("Runde");
        let bet = r.bet;
        let m100 = self.last_mult;
        let (grid, mines) = (r.grid, r.mines);
        self.stats.record(bet, paid, m100);
        self.hist.push(m100);
        if self.hist.len() > ml::HISTORY {
            let cut = self.hist.len() - ml::HISTORY;
            self.hist.drain(..cut);
        }
        let mut o = Map::new();
        o.insert("grid".into(), Value::from(grid));
        o.insert("mines".into(), Value::from(mines));
        o.insert("open".into(), Value::Array(opened.iter().map(|v| Value::from(*v)).collect()));
        o.insert("m".into(), Value::from(m100));
        o.insert("bet".into(), Value::from(bet));
        let entry = self.entry();
        fairness::record(&mut self.fairlog, &entry, &Value::Object(o));
        self.fair_sel = None;
        if let Some(a) = self.auto.as_mut() {
            a.net += paid - bet;
        }
        self.round = None;
    }

    /// Auszahlen (Enter/C/Knopf) - automatisch bei vollem Feld/Hoechstgewinn.
    fn cash_out(&mut self, auto: bool) {
        if self.state != State::Play || self.round.is_none() {
            return;
        }
        let k = self.kk();
        if k < 1 {
            if !auto {
                self.say(t("lmi.need_gem"), 1.2);
            }
            return;
        }
        let bet = self.round.as_ref().map(|r| r.bet).unwrap_or(0);
        let m100 = ml::mult100(self.grid, self.mines, k);
        let (win, carry) = ml::payout_carry(bet, self.grid, self.mines, k, self.carry);
        self.win = win;
        self.carry = carry;
        self.last_mult = m100;
        let opened = self.round.as_ref().map(|r| r.open.clone()).unwrap_or_default();
        let clear = ml::is_clear(self.grid, self.mines, k);
        let maxed = ml::is_max(self.grid, self.mines, k);
        lamabank::credit(self.win, KEY, false);
        self.pending += self.win;
        if clear {
            let v = self.xint("clears") + 1;
            self.xset("clears", v);
        }
        self.finish(self.win, &opened);
        self.save_extra();
        self.b.score = lamabank::score_for(KEY);
        self.state = State::Cashed;
        self.res_t = 0.0;
        self.res_delay = if auto { FLIP_T * self.tf() } else { 0.0 };
        let mut sub = format!("{}   \u{00B7}   {}: {}", self.fmt(m100), t("cas.bet"), self.group(bet));
        let title = if maxed {
            tf("lmi.cashed", &[("n", &self.group(self.win))])
        } else if clear {
            t("lmi.cleared")
        } else {
            tf("lmi.cashed", &[("n", &self.group(self.win))])
        };
        if clear && !maxed {
            sub = format!("{}   \u{00B7}   {}", tf("lmi.cashed", &[("n", &self.group(self.win))]), self.fmt(m100));
        }
        self.result = Some((title, sub, ui::pal().gold));
        self.cash_info = CashInfo { clear, maxed, m100, shown: false };
        // Erfolge sofort (nichts geht beim Verlassen verloren)
        if clear {
            self.b.ach_event("lmi_clear", None);
        }
        if m100 >= ml::BRAVE_MULT {
            self.b.ach_event("lmi_brave", None);
        }
        if m100 >= ml::LEGEND_MULT {
            self.b.ach_event("lmi_legend", None);
        }
        if !auto {
            self.celebrate();
        }
    }

    /// Gewinn zeigen: Zaehler, Popup, Muenzen, Konfetti, Rest des Feldes.
    fn celebrate(&mut self) {
        if self.cash_info.shown {
            return;
        }
        self.cash_info.shown = true;
        self.pending = 0.max(self.pending - self.win);
        let info = self.cash_info;
        let m100 = info.m100;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if info.maxed {
            self.banner = Some((t("chip.max_win"), 3.0));
            ui::spawn_confetti(w, h, 140);
            self.coin_shower(90);
            self.b.play_sound("level");
            self.b.rumble(260);
        } else if info.clear || m100 >= PARTY_MULT {
            ui::spawn_confetti(w, h, 110);
            self.coin_shower(60);
            self.b.play_sound("level");
            self.b.rumble(200);
        } else {
            if m100 >= BIG_MULT {
                self.coin_shower(28);
            }
            self.b.play_sound("win");
        }
        self.mood = (Mood::Cheer, 0.0);
        let origin = self.opened_last();
        self.reveal_rest(origin);
    }

    /// Ursprung der Rest-Aufdeckung: zuletzt gefundener Edelstein.
    fn opened_last(&self) -> i32 {
        let n = self.cells.len() as i64;
        let mut last: Option<i64> = None;
        if let Some(e) = self.fairlog.last() {
            if let Some(Value::Array(open)) = e.o.as_object().and_then(|o| o.get("open")) {
                if let Some(v) = open.last() {
                    last = match v {
                        Value::Bool(b) => Some(*b as i64),
                        _ => py_int(Some(v)),
                    };
                }
            }
        }
        match last {
            Some(v) if 0 <= v && v < n => v as i32,
            _ => (n / 2) as i32,
        }
    }

    /// Alle noch verdeckten Felder abgedunkelt aufdecken - als Welle ab origin.
    fn reveal_rest(&mut self, origin: i32) {
        let n = self.grid;
        let (ox, oy) = (py::pmod32(origin, n), py::fdiv32(origin, n));
        let tf = self.tf();
        for (i, c) in self.cells.iter_mut().enumerate() {
            if c.face != Face::Hidden {
                continue;
            }
            let i = i as i32;
            let (dx, dy) = ((i % n) - ox, (i / n) - oy);
            let dd = ((dx * dx + dy * dy) as f64).sqrt();
            c.face = if self.is_mine[i as usize] { Face::Mine } else { Face::Gem };
            c.dim = true;
            c.flip = 0.0;
            c.delay = (0.08 + WAVE_T * dd) * tf;
        }
    }

    /// Mine geht hoch: Blitz, Feuer, Rauch, Truemmer, Wackeln.
    fn explode(&mut self) {
        let Some(i) = self.hit else {
            return;
        };
        let r = self.tile_r[i];
        let (x, y) = r.center();
        self.boom_t = Some(0.0);
        self.shake = SHAKE_T;
        self.flashes.push(Flash { x, y, t: 0.0, dur: 0.45, r: (self.cell as f64 * 1.6) as i32, col: None });
        let (xf, yf) = (x as f64, y as f64);
        self.spawn(xf, yf, 26, (255, 196, 80), 380.0, false, 4.0, 0.45, 300.0, PKind::Fire);
        self.spawn(xf, yf, 18, (255, 110, 50), 300.0, false, 3.6, 0.55, 200.0, PKind::Fire);
        self.spawn(xf, yf, 14, (84, 78, 90), 120.0, true, 7.0, 1.3, -60.0, PKind::Smoke);
        self.spawn(xf, yf, 16, (70, 60, 66), 340.0, true, 3.0, 1.0, 1100.0, PKind::Debris);
        self.b.play_sound("explode");
        self.b.rumble(320);
        self.mood = (Mood::Scared, 0.0);
        self.sooty = true;
        self.cart_fill = 0;
        self.reveal_rest(i as i32);
        let tf = self.tf();
        for c in self.cells.iter_mut() {
            if c.dim {
                c.delay += 0.22 * tf;
            }
        }
    }

    /// Edelstein fertig aufgeklappt: Funkeln, steigender Ton, Popup.
    fn gem_found(&mut self, i: usize) {
        let r = self.tile_r[i];
        let k = 1.max(self.cells[i].k);
        let (cx, cy) = (r.centerx() as f64, r.centery() as f64);
        self.spawn(cx, cy, 12, (120, 255, 220), 200.0, false, 2.6, 0.5, 120.0, PKind::Spark);
        self.spawn(cx, cy, 6, (255, 255, 255), 150.0, false, 2.0, 0.4, 60.0, PKind::Star);
        self.flashes.push(Flash {
            x: r.centerx(),
            y: r.centery(),
            t: 0.0,
            dur: 0.3,
            r: (self.cell as f64 * 0.9) as i32,
            col: Some((40, 160, 130)),
        });
        let step = GEM_STEPS[(GEM_STEPS.len() as i32 - 1).min(0.max(k - 1)) as usize];
        audio::tone(523.25 * py::pow(2.0, step as f64 / 12.0), 0.12, Some(&self.b.settings), "sine", 0.28);
        if self.state == State::Play && self.round.is_some() {
            let m100 = ml::mult100(self.grid, self.mines, k);
            let txt = self.fmt(m100);
            self.popup(r.centerx(), r.centery() - (self.cell as f64 * 0.15) as i32, txt, Some(ui::pal().green), 0.9);
            self.mood = (Mood::Happy, 0.0);
            self.cart_fill = self.cart_level();
        } else if self.state == State::Cashed {
            self.cart_fill = if self.cash_info.clear { 7 } else { 1.max(self.cart_fill) };
        }
    }

    /// Ergebnis schliessen: Ruhezustand, das aufgedeckte Feld bleibt sichtbar.
    fn go_idle(&mut self) {
        if self.state == State::Cashed && !self.cash_info.shown {
            self.celebrate();
        }
        for c in self.cells.iter_mut() {
            if c.flip < 1.0 || c.delay > 0.0 {
                c.flip = 1.0;
                c.delay = 0.0;
            }
        }
        self.state = State::Idle;
        self.result = None;
        self.boom_t = None;
        self.res_t = 0.0;
        self.pending = 0;
        if self.mood.0 == Mood::Scared {
            self.mood = (Mood::Idle, 0.0);
        }
        self.clamp_bet();
        self.check_broke();
    }

    // ----- Einstellungen aendern ------------------------------------------------

    /// Vor einer Aenderung im Ruhezustand: altes Feld verdecken.
    fn idle_change(&mut self) {
        if !self.field.is_empty() || self.cells.iter().any(|c| c.face != Face::Hidden) {
            self.blank_board();
        }
    }

    fn set_bet(&mut self, direction: i32) {
        if self.state != State::Idle || self.auto.is_some() {
            return;
        }
        let i = ml::BETS.iter().position(|b| *b == self.bet).unwrap_or(0) as i32;
        let j = 0.max((ml::BETS.len() as i32 - 1).min(i + direction));
        if j != i {
            self.bet = ml::BETS[j as usize];
            self.set_opt("bet", Value::from(self.bet));
            self.b.play_sound("select");
            if self.bet > lamabank::balance() {
                self.say(t("cas.not_enough"), 1.2);
            }
        }
    }

    fn set_grid(&mut self, grid: i32) {
        if self.state != State::Idle || self.auto.is_some() || grid == self.grid || !ml::GRIDS.contains(&grid) {
            return;
        }
        self.grid = grid;
        self.mines = ml::clamp_mines(grid, self.mines);
        self.set_opt("grid", Value::from(grid));
        self.set_opt("mines", Value::from(self.mines));
        self.cursor = (grid * grid) / 2;
        self.layout();
        self.blank_board();
        self.cart_fill = 0;
        self.b.play_sound("click");
    }

    fn set_mines(&mut self, mines: i32) {
        if self.state != State::Idle || self.auto.is_some() {
            return;
        }
        let mines = ml::clamp_mines(self.grid, mines);
        if mines == self.mines {
            return;
        }
        self.mines = mines;
        self.set_opt("mines", Value::from(mines));
        self.idle_change();
        self.b.play_sound("click");
    }

    fn toggle_turbo(&mut self) {
        self.turbo = !self.turbo;
        self.set_opt("turbo", Value::from(self.turbo));
        let msg = if self.turbo { t("lmi.turbo_on") } else { t("lmi.turbo_off") };
        self.say(msg, 1.0);
        self.b.play_sound("click");
    }

    fn open_window(&mut self, name: Win) {
        if self.window == Some(name) {
            self.window = None;
            self.b.play_sound("click");
            return;
        }
        if name == Win::Auto {
            if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
                return;
            }
            if matches!(self.state, State::Boom | State::Cashed) && self.result_visible() && self.res_t >= READ_T {
                self.go_idle();
            }
            if self.state != State::Idle || self.broke {
                self.say(t("lmi.locked"), 1.2);
                return;
            }
            self.idle_change();
            self.kb = false;
        }
        self.window = Some(name);
        if name == Win::Fair {
            self.fair_scroll = 0;
            self.fair_sel = if self.fairlog.is_empty() { None } else { Some(self.fairlog.len() - 1) };
        }
        self.b.play_sound("click");
    }

    // ----- Auto-Modus ------------------------------------------------------------

    fn marks_now(&self) -> &Vec<i32> {
        self.marks.get(self.grid)
    }

    fn toggle_mark(&mut self, i: i32) {
        let gems = ml::gems(self.grid, self.mines);
        let grid = self.grid;
        if let Some(pos) = self.marks.get(grid).iter().position(|v| *v == i) {
            self.marks.get_mut(grid).remove(pos);
        } else {
            if self.marks.get(grid).len() as i32 >= gems {
                self.say(tf("lmi.auto_too_many", &[("n", &gems.to_string())]), 1.4);
                self.b.play_sound("hit");
                return;
            }
            let lst = self.marks.get_mut(grid);
            lst.push(i);
            lst.sort();
            self.cells[i as usize].mark = 0.0;
        }
        self.save_extra();
        self.b.play_sound("rotate");
    }

    fn clear_marks(&mut self) {
        *self.marks.get_mut(self.grid) = Vec::new();
        self.save_extra();
        self.b.play_sound("click");
    }

    fn start_auto(&mut self) {
        let n = self.marks_now().len() as i32;
        let gems = ml::gems(self.grid, self.mines);
        if n == 0 {
            self.say(t("lmi.auto_need"), 1.6);
            self.b.play_sound("hit");
            return;
        }
        if n > gems {
            self.say(tf("lmi.auto_too_many", &[("n", &gems.to_string())]), 1.6);
            self.b.play_sound("hit");
            return;
        }
        if self.bet > lamabank::balance() {
            self.say(t("cas.not_enough"), 1.8);
            self.b.play_sound("hit");
            return;
        }
        self.window = None;
        self.auto = Some(AutoRun { left: self.auto_rounds, net: 0, t: 0.0, queue: Vec::new() });
        self.b.play_sound("powerup");
    }

    /// Auto beenden. Eine laufende Runde bleibt offen.
    fn stop_auto(&mut self, msg: Option<String>) {
        if self.auto.is_none() {
            return;
        }
        self.auto = None;
        if let Some(m) = msg {
            if !m.is_empty() {
                self.say(m, 2.0);
            }
        }
        self.b.play_sound("click");
        if self.state == State::Idle {
            self.check_broke();
        }
    }

    fn auto_tick(&mut self, dt: f64) {
        {
            let Some(a) = self.auto.as_mut() else {
                return;
            };
            a.t -= dt;
            if a.t > 0.0 {
                return;
            }
        }
        let (left, net) = self.auto.as_ref().map(|a| (a.left, a.net)).unwrap_or((0, 0));
        if self.state == State::Idle {
            // Grenzen pruefen, dann die naechste Runde
            if left <= 0 {
                self.stop_auto(Some(t("chip.auto_done")));
                return;
            }
            if self.auto_win > 0 && net >= self.auto_win {
                self.stop_auto(Some(t("chip.auto_hit_win")));
                return;
            }
            if self.auto_loss > 0 && -net >= self.auto_loss {
                self.stop_auto(Some(t("chip.auto_hit_loss")));
                return;
            }
            if self.bet > lamabank::balance() {
                self.stop_auto(Some(t("chip.auto_no_chips")));
                return;
            }
            let lst = self.marks_now().clone();
            if lst.is_empty() || lst.len() as i32 > ml::gems(self.grid, self.mines) {
                self.stop_auto(Some(t("lmi.auto_need")));
                return;
            }
            if self.play() {
                let tf = self.tf();
                if let Some(a) = self.auto.as_mut() {
                    a.left -= 1;
                    a.queue = lst;
                    a.t = (AUTO_STEP_T + 0.1) * tf;
                }
            } else {
                self.stop_auto(Some(t("chip.auto_no_chips")));
            }
        } else if self.state == State::Play {
            if self.cells.iter().any(|c| c.flip < 1.0) {
                return;
            }
            let next = self.auto.as_mut().and_then(|a| if a.queue.is_empty() { None } else { Some(a.queue.remove(0)) });
            if let Some(i) = next {
                self.reveal(i);
                let tf = self.tf();
                if let Some(a) = self.auto.as_mut() {
                    a.t = AUTO_STEP_T * tf;
                }
            } else {
                self.cash_out(true);
                self.res_delay = 0.0;
                self.celebrate();
            }
        }
    }

    // ===================================================== Effekte

    fn spawn(&mut self, x: f64, y: f64, n: usize, col: (i32, i32, i32), speed: f64, up: bool, r: f64, life: f64, grav: f64, kind: PKind) {
        let k = self.k;
        for _ in 0..n {
            let a = if up { self.rng.uniform(PI * 1.1, PI * 1.9) } else { self.rng.uniform(0.0, TAU) };
            let sp = speed * self.rng.uniform(0.3, 1.0) * k;
            let pr = r * k * self.rng.uniform(0.6, 1.25);
            let pl = life * self.rng.uniform(0.7, 1.15);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                r: pr,
                col,
                t: 0.0,
                life: pl,
                g: grav * k,
                kind,
            });
        }
        if self.particles.len() > 260 {
            let cut = self.particles.len() - 260;
            self.particles.drain(..cut);
        }
    }

    fn coin_shower(&mut self, n: usize) {
        let w = self.b.width as f64;
        let h = self.b.height as f64;
        let k = self.k;
        for _ in 0..n {
            let x = self.rng.uniform(w * 0.05, w * 0.95);
            let y = self.rng.uniform(-h * 0.4, -10.0);
            let vx = self.rng.uniform(-60.0, 60.0) * k;
            let vy = self.rng.uniform(-80.0, 120.0) * k;
            let spin = self.rng.uniform(0.0, TAU);
            let vs = self.rng.uniform(6.0, 14.0);
            let r = self.rng.uniform(7.0, 12.0) * 0.7f64.max(k);
            self.coins_fx.push(CoinFx { x, y, vx, vy, spin, vs, r });
        }
        if self.coins_fx.len() > 180 {
            let cut = self.coins_fx.len() - 180;
            self.coins_fx.drain(..cut);
        }
    }

    fn popup(&mut self, x: i32, y: i32, text: String, color: Option<Color>, dur: f64) {
        let color = color.unwrap_or(ui::pal().gold);
        self.popups.push(Popup { x, y, text, color, t: 0.0, dur });
    }

    // ===================================================== Eingabe

    fn tile_at(&self, pos: (i32, i32)) -> Option<usize> {
        self.tile_r.iter().position(|r| r.hit(pos))
    }

    fn on_key(&mut self, key: &str, repeat: bool) {
        let single = key.chars().count() == 1;
        let low = if single { key.to_lowercase() } else { key.to_string() };
        // Fenster ----------------------------------------------------------
        if key == "Escape" {
            if self.window.is_some() {
                self.window = None;
                self.b.play_sound("click");
            } else if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
            }
            return;
        }
        if single && matches!(low.as_str(), "a" | "s" | "f" | "t") {
            if repeat {
                return;
            }
            match low.as_str() {
                "t" => self.toggle_turbo(),
                "a" => self.open_window(Win::Auto),
                "s" => self.open_window(Win::Stats),
                _ => self.open_window(Win::Fair),
            }
            return;
        }
        if self.window == Some(Win::Auto) {
            self.auto_key(key);
            return;
        }
        if self.window.is_some() {
            if Self::is_action(key) && !repeat {
                self.window = None;
            }
            return;
        }
        // Auto laeuft: Leertaste/Enter = STOPP --------------------------------
        if self.auto.is_some() {
            if Self::is_action(key) && !repeat {
                self.stop_auto(Some(t("chip.auto_done")));
            }
            return;
        }
        // Ergebnis: nach kurzer Lesezeit weiter wie im Ruhezustand ------------
        if matches!(self.state, State::Boom | State::Cashed) {
            if !self.result_visible() || self.res_t < READ_T || repeat {
                return;
            }
            self.go_idle();
            if self.broke {
                return;
            }
        }
        if self.state == State::Idle {
            if Self::is_action(key) {
                if !repeat {
                    self.play();
                }
            } else if matches!(key, "1" | "2" | "3" | "4") {
                let i = key.parse::<usize>().unwrap_or(1) - 1;
                self.set_grid(ml::GRIDS[i]);
            } else if matches!(key, "Left" | "Right") {
                let m = self.mines + if key == "Right" { 1 } else { -1 };
                self.set_mines(m);
            } else if matches!(key, "plus" | "KP_Add" | "Up" | "equal") {
                self.set_bet(1);
            } else if matches!(key, "minus" | "KP_Subtract" | "Down") {
                self.set_bet(-1);
            }
            return;
        }
        if self.state == State::Play {
            if matches!(key, "Left" | "Right" | "Up" | "Down") {
                self.move_cursor(key);
            } else if key == "space" {
                if !repeat {
                    if !self.kb {
                        self.kb = true;
                    } else {
                        self.reveal(self.cursor);
                    }
                }
            } else if matches!(key, "Return" | "KP_Enter") || low == "c" {
                if !repeat {
                    self.cash_out(false);
                }
            } else if low == "r" && !repeat {
                self.random_tile();
            }
        }
    }

    fn move_cursor(&mut self, key: &str) {
        let n = self.grid;
        if self.kb {
            let (mut x, mut y) = (py::pmod32(self.cursor, n), py::fdiv32(self.cursor, n));
            x += match key {
                "Left" => -1,
                "Right" => 1,
                _ => 0,
            };
            y += match key {
                "Up" => -1,
                "Down" => 1,
                _ => 0,
            };
            self.cursor = 0.max((n - 1).min(y)) * n + 0.max((n - 1).min(x));
        }
        self.kb = true;
    }

    /// Tasten im Auto-Fenster: Pfeile + Leertaste markieren, Enter startet.
    fn auto_key(&mut self, key: &str) {
        if matches!(key, "Left" | "Right" | "Up" | "Down") {
            self.move_cursor(key);
        } else if key == "space" {
            if self.kb {
                self.toggle_mark(self.cursor);
            }
            self.kb = true;
        } else if matches!(key, "Return" | "KP_Enter") {
            self.start_auto();
        } else if matches!(key, "Delete" | "BackSpace") {
            self.clear_marks();
        }
    }

    /// R: zufaelliges verdecktes Feld (Zufall der Oberflaeche, nicht der Runde).
    fn random_tile(&mut self) {
        if self.state != State::Play || self.round.is_none() {
            return;
        }
        let free: Vec<i32> = self.cells.iter().enumerate().filter(|(_, c)| c.face == Face::Hidden).map(|(i, _)| i as i32).collect();
        if free.is_empty() {
            return;
        }
        let i = free[self.rng.randrange0(free.len() as i64) as usize];
        self.cursor = i;
        self.reveal(i);
    }

    fn on_click(&mut self, pos: (i32, i32)) {
        // Werkzeug-Knoepfe (immer erreichbar)
        let tools = self.r_tools.clone();
        for (name, r) in tools {
            if r.hit(pos) {
                match name {
                    "turbo" => self.toggle_turbo(),
                    "auto" => self.open_window(Win::Auto),
                    "stats" => self.open_window(Win::Stats),
                    _ => self.open_window(Win::Fair),
                }
                return;
            }
        }
        if self.window == Some(Win::Auto) {
            self.auto_click(pos);
            return;
        }
        if self.window.is_some() {
            self.window_click(pos);
            return;
        }
        if self.auto.is_some() {
            if self.r_go.hit(pos) {
                self.stop_auto(Some(t("chip.auto_done")));
            }
            return;
        }
        if matches!(self.state, State::Boom | State::Cashed) {
            if !self.result_visible() || self.res_t < READ_T {
                return;
            }
            self.go_idle();
            if self.broke {
                return;
            }
            if !self.r_go.hit(pos) {
                return;
            }
        }
        if self.state == State::Idle {
            if self.r_minus.hit(pos) {
                self.set_bet(-1);
            } else if self.r_plus.hit(pos) {
                self.set_bet(1);
            } else if self.r_go.hit(pos) {
                self.play();
            } else if self.r_cash.hit(pos) {
                self.say(t("lmi.press_play"), 1.4);
            } else if self.board_r.hit(pos) {
                self.say(t("lmi.press_play"), 1.4);
                self.b.play_sound("hit");
            } else {
                for i in 0..self.r_grids.len() {
                    if self.r_grids[i].hit(pos) {
                        self.set_grid(ml::GRIDS[i]);
                        return;
                    }
                }
                if self.r_mminus.hit(pos) {
                    self.set_mines(self.mines - 1);
                } else if self.r_mplus.hit(pos) {
                    self.set_mines(self.mines + 1);
                } else {
                    for i in 0..self.r_presets.len() {
                        if self.r_presets[i].hit(pos) {
                            self.set_mines(ml::mine_presets(self.grid)[i]);
                            return;
                        }
                    }
                }
            }
            return;
        }
        if self.state == State::Play {
            if self.r_cash.hit(pos) {
                self.cash_out(false);
            } else if self.r_random.hit(pos) {
                self.random_tile();
            } else if let Some(i) = self.tile_at(pos) {
                self.kb = false;
                self.cursor = i as i32;
                self.reveal(i as i32);
            }
        }
    }

    fn auto_click(&mut self, pos: (i32, i32)) {
        if self.r_auto_close.hit(pos) {
            self.window = None;
            self.b.play_sound("click");
            return;
        }
        if let Some(i) = self.tile_at(pos) {
            self.kb = false;
            self.cursor = i as i32;
            self.toggle_mark(i as i32);
            return;
        }
        for row in 0..3 {
            let opts: &[i64] = if row == 0 { &ROUND_OPTS } else { &LIMIT_OPTS };
            let key = ["auto_rounds", "auto_win", "auto_loss"][row];
            let treffer = self.auto_rows[row].1.iter().position(|r| r.hit(pos));
            if let Some(j) = treffer {
                let v = opts[j];
                match row {
                    0 => self.auto_rounds = v,
                    1 => self.auto_win = v,
                    _ => self.auto_loss = v,
                }
                self.set_opt(key, Value::from(v));
                self.b.play_sound("click");
                return;
            }
        }
        if self.r_auto_clear.hit(pos) {
            self.clear_marks();
        } else if self.r_auto_start.hit(pos) {
            self.start_auto();
        }
    }

    /// Klick in Statistik-/Fairness-Fenster (ausserhalb = schliessen).
    fn window_click(&mut self, pos: (i32, i32)) {
        let (rect, close) = self.window_rect();
        if close.hit(pos) || !rect.hit(pos) {
            self.window = None;
            self.b.play_sound("click");
            return;
        }
        if self.window != Some(Win::Fair) {
            return;
        }
        if self.r_fair_new.hit(pos) {
            if self.round.is_some() {
                self.say(t("lmi.locked"), 1.4);
                self.b.play_sound("hit");
            } else {
                let rng = &mut self.rng;
                fairness::new_client(&mut self.fair, &mut |k| rng.getrandbits(k));
                self.save_extra();
                self.b.play_sound("select");
            }
            return;
        }
        if self.r_fair_verify.hit(pos) {
            if let Some(sel) = self.fair_sel {
                self.verify(sel);
                return;
            }
        }
        let rows = self.r_fair_rows.clone();
        for (idx, r) in rows {
            if r.hit(pos) {
                self.fair_sel = Some(idx);
                self.b.play_sound("click");
                return;
            }
        }
    }

    /// Nachrechnen: Pruefcode = sha256(Seed) und Minenfeld aus den Seeds.
    fn verify(&mut self, idx: usize) {
        if idx >= self.fairlog.len() {
            return;
        }
        let e = &self.fairlog[idx];
        let mut ok = e.verify();
        let mut field = None;
        if let Some(o) = e.o.as_object() {
            let grid = py_int(o.get("grid")).filter(|g| ml::GRIDS.iter().any(|x| *x as i64 == *g));
            let mines = py_int(o.get("mines"));
            if let (Some(g), Some(m)) = (grid, mines) {
                let g = g as i32;
                if 1 <= m && m <= ml::max_mines(g) as i64 {
                    let m = m as i32;
                    let f = ml::field_from_seed(g, m, &e.s, &e.c, e.n);
                    let opened: Vec<Value> = match o.get("open") {
                        Some(Value::Array(a)) => a.clone(),
                        _ => Vec::new(),
                    };
                    let mv = o.get("m");
                    if mv.and_then(num) == Some(0.0) {
                        ok = ok
                            && !opened.is_empty()
                            && in_list(&opened[opened.len() - 1], &f)
                            && !opened[..opened.len() - 1].iter().any(|v| in_list(v, &f));
                    } else {
                        let soll = ml::mult100(g, m, opened.len() as i32) as f64;
                        ok = ok && !opened.iter().any(|v| in_list(v, &f)) && mv.and_then(num) == Some(soll);
                    }
                    field = Some(f);
                }
            }
        }
        let n = e.n;
        self.fair_checks.insert(n, (ok, field));
        self.b.play_sound(if ok { "point" } else { "hit" });
    }

    fn result_visible(&self) -> bool {
        self.result.is_some() && self.res_delay <= 0.0
    }

    fn update_cells(&mut self, dt: f64) {
        let ft = FLIP_T * self.tf();
        for i in 0..self.cells.len() {
            let mut found = false;
            {
                let c = &mut self.cells[i];
                c.mark += dt;
                if c.delay > 0.0 {
                    c.delay -= dt;
                    if c.delay > 0.0 {
                        continue;
                    }
                }
                if c.flip < 1.0 {
                    c.flip = 1f64.min(c.flip + dt / ft);
                    if c.flip >= 1.0 {
                        c.pop = 0.0;
                        if c.face == Face::Gem && !c.dim {
                            found = true;
                        }
                    }
                } else if c.pop < POP_T {
                    c.pop = POP_T.min(c.pop + dt);
                }
            }
            if found {
                self.gem_found(i);
            }
        }
    }

    fn update_fx(&mut self, dt: f64) {
        for p in self.particles.iter_mut() {
            p.t += dt;
            p.vy += p.g * dt;
            if p.kind == PKind::Smoke {
                p.vx *= 0f64.max(1.0 - dt * 1.5);
                p.vy *= 0f64.max(1.0 - dt * 1.5);
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.t < p.life);
        let k = self.k;
        let h = self.b.height as f64;
        for c in self.coins_fx.iter_mut() {
            c.vy += 900.0 * k * dt;
            c.x += c.vx * dt;
            c.y += c.vy * dt;
            c.spin += c.vs * dt;
        }
        self.coins_fx.retain(|c| c.y < h + 30.0);
        for p in self.popups.iter_mut() {
            p.t += dt;
        }
        self.popups.retain(|p| p.t < p.dur);
        for f in self.flashes.iter_mut() {
            f.t += dt;
        }
        self.flashes.retain(|f| f.t < f.dur);
    }

    // ===================================================== Zeichnen: Helfer

    /// Text in der ersten Schrift, die passt - sonst gekuerzt mit '...'.
    fn fit(&self, text: &str, fonts: &[&Font], maxw: i32, col: impl Col) -> Surface {
        for f in fonts {
            if f.width(text) <= maxw {
                return f.render(text, true, col);
            }
        }
        let f = fonts[fonts.len() - 1];
        let mut chars: Vec<char> = text.chars().collect();
        loop {
            let s: String = chars.iter().collect();
            if chars.is_empty() || f.width(&(s.clone() + "\u{2026}")) <= maxw {
                return f.render(&(s + "\u{2026}"), true, col);
            }
            chars.pop();
        }
    }

    fn wrap(text: &str, font: &Font, maxw: i32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for wd in text.split(' ') {
            let test = if !cur.is_empty() { format!("{cur} {wd}") } else { wd.to_string() };
            if font.width(&test) <= maxw || cur.is_empty() {
                cur = test;
            } else {
                lines.push(cur);
                cur = wd.to_string();
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    fn var(&self, i: usize) -> i64 {
        (d::hash(i as i64 * 7 + self.grid as i64) % 4) as i64
    }

    fn shake_off(&self) -> (i32, i32) {
        if self.shake <= 0.0 {
            return (0, 0);
        }
        let a = 7.0 * self.k * self.shake / SHAKE_T;
        (((self.time * 71.0).sin() * a) as i32, ((self.time * 53.0).cos() * a * 0.7) as i32)
    }

    #[allow(clippy::too_many_arguments)]
    fn btn(&self, s: &mut Surface, r: Rect, label: &str, primary: bool, on: bool, hot: bool, active: bool, small: bool, icon: Option<&str>, danger: bool) {
        let p = ui::pal();
        let rad = 5.max(r.h / 4);
        let col: Color = if primary && on {
            let base = ui::mix(if danger { p.red } else { self.b.accent }, (0, 0, 0), if !danger { 0.30 } else { 0.1 });
            let fill = ui::mix(base, (255, 255, 255), if hot { 0.12 } else { 0.0 });
            draw::rect_r(s, ui::mix(base, (0, 0, 0), 0.4), r.move_(0, 2), 0, rad);
            draw::rect_r(s, fill, r, 0, rad);
            let pulse = 0.5 + 0.5 * (self.time * 2.4).sin();
            draw::rect_r(s, ui::mix(cd::COL_GOLD, (255, 255, 255), 0.35 * pulse), r, 2, rad);
            rgb(255, 255, 255)
        } else {
            let fill = if on && (hot || active) { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, r, 0, rad);
            let edge = if active { cd::COL_GOLD.c() } else if on { p.border_light } else { p.border };
            draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
            if on {
                p.text
            } else {
                p.text_faint
            }
        };
        let mut tx = r.centerx();
        if let Some(ic) = icon {
            let isz = (r.h as f64 * 0.62) as i32;
            if !label.is_empty() {
                let lw = self.f_btn.width(label);
                let ix = r.centerx() - py::fdiv32(isz + 6 + lw, 2);
                d::icon(s, ic, Rect::new(ix, r.centery() - isz / 2, isz, isz), col);
                tx = ix + isz + 6 + lw / 2;
            } else {
                d::icon(s, ic, Rect::new(r.centerx() - isz / 2, r.centery() - isz / 2, isz, isz), col);
            }
        }
        if !label.is_empty() {
            let img = if small {
                self.fit(label, &[&self.f_tiny, &self.f_micro], r.w - 6, col)
            } else {
                self.fit(label, &[&self.f_btn, &self.f_tiny, &self.f_micro], r.w - 6, col)
            };
            s.blr(&img, img.get_rect().center_at((tx, r.centery())));
        }
    }

    /// Kurzform fuer einfache Knoepfe: `_btn(s, r, label, on=, hot=, active=, small=)`.
    fn btn_s(&self, s: &mut Surface, r: Rect, label: &str, on: bool, hot: bool, active: bool, small: bool) {
        self.btn(s, r, label, false, on, hot, active, small, None, false);
    }

    // ===================================================== Zeichnen: Spielbrett

    fn draw_board(&self, s: &mut Surface) {
        let (ox, oy) = self.shake_off();
        let br = self.board_r.move_(ox, oy);
        s.bl(&d::board(br.w, self.board_pad), (br.x, br.y));
        let cell = self.cell;
        let play = self.state == State::Play && self.window.is_none() && self.auto.is_none();
        let marking = self.window == Some(Win::Auto) || self.auto.is_some();
        let hover = if (play || self.window == Some(Win::Auto)) && !self.kb { self.tile_at(self.mouse) } else { None };
        let empty = Vec::new();
        let marks = if marking { self.marks_now() } else { &empty };
        for (i, c) in self.cells.iter().enumerate() {
            let r = self.tile_r[i].move_(ox, oy);
            let f = c.flip;
            if c.face == Face::Hidden {
                if f < 1.0 {
                    // Einwachsen beim Rundenstart (Welle von links oben)
                    if c.delay > 0.0 {
                        continue;
                    }
                    let sz = 2.max((cell as f64 * (0.55 + 0.45 * f)) as i32);
                    let img = transform::smoothscale(&d::tile_hidden(cell, self.var(i), false), sz, sz);
                    s.blr(&img, img.get_rect().center_at(r.center()));
                    continue;
                }
                let hot = Some(i) == hover;
                let img = d::tile_hidden(cell, self.var(i), hot);
                if hot {
                    s.blr(&img, r.move_(0, -(1.max((2.0 * self.k) as i32))));
                } else {
                    s.blr(&img, r);
                }
                if marks.contains(&(i as i32)) {
                    d::draw_mark(s, r, self.b.accent, self.time, i as i32);
                }
                continue;
            }
            if c.delay > 0.0 {
                s.blr(&d::tile_hidden(cell, self.var(i), false), r); // wartet aufs Aufklappen
                continue;
            }
            if f < 1.0 {
                self.draw_flip(s, r, i, c);
                continue;
            }
            self.draw_face(s, r, i, c);
        }
        // Tastatur-Cursor
        if self.kb && (play || self.window == Some(Win::Auto)) {
            let g = 4.max((6.0 * self.k) as i32);
            let r = self.tile_r[self.cursor as usize].move_(ox, oy).inflate(g, g);
            let pulse = 0.5 + 0.5 * (self.time * 6.0).sin();
            let col = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.45 * pulse);
            draw::rect_r(s, col, r, 2.max((3.0 * self.k) as i32), 3.max((cell as f64 * 0.2) as i32));
        }
        self.draw_particles(s);
        for fl in &self.flashes {
            let p = fl.t / fl.dur;
            let rr = 2.max((fl.r as f64 * (0.4 + 0.8 * p)) as i32);
            let col = fl.col.unwrap_or((255, 200, 120));
            let fade = 1.0 - p;
            let g = d::glow(rr, ((col.0 as f64 * fade) as i32, (col.1 as f64 * fade) as i32, (col.2 as f64 * fade) as i32));
            s.bl_flags(&g, (fl.x - rr - 1 + ox, fl.y - rr - 1 + oy), BLEND_RGB_ADD);
            if fl.col.is_none() && p < 0.5 {
                let ring = 2.max((fl.r as f64 * (0.3 + 1.2 * p)) as i32);
                draw::circle(
                    s,
                    ui::mix((255, 240, 200), (255, 140, 60), p * 2.0),
                    (fl.x + ox, fl.y + oy),
                    ring,
                    1.max((4.0 * self.k * (1.0 - p * 2.0)) as i32),
                );
            }
        }
    }

    /// Aufklappen: erst schrumpft der Stein (Breite), dann waechst die Vorderseite.
    fn draw_flip(&self, s: &mut Surface, r: Rect, i: usize, c: &Cell) {
        let f = c.flip;
        let cell = self.cell as f64;
        let lift = (PI * f).sin() * 0.10;
        let (img, w) = if f < 0.5 {
            (d::tile_hidden(self.cell, self.var(i), false), 1.max((cell * (1.0 - 2.0 * f) * (1.0 + lift)) as i32))
        } else {
            (d::face(self.cell, c.face.name(), c.dim), 1.max((cell * (2.0 * f - 1.0) * (1.0 + lift)) as i32))
        };
        let h = 1.max((cell * (1.0 + lift)) as i32);
        let img = transform::scale(&img, w, h);
        s.blr(&img, img.get_rect().center_at((r.centerx(), r.centery() - (cell * lift * 0.5) as i32)));
    }

    /// Aufgedecktes Feld in Ruhe (Edelstein federt nach, funkelt).
    fn draw_face(&self, s: &mut Surface, r: Rect, i: usize, c: &Cell) {
        let cell = self.cell;
        let cf = cell as f64;
        let kind = c.face;
        if kind == Face::Gem && !c.dim && c.pop < POP_T {
            s.blr(&d::tile_open(cell), r);
            let p = c.pop / POP_T;
            let sc = 0.80 * (1.0 + 0.22 * (PI * p).sin() * (1.0 - p * 0.3));
            let g = 4.max((cf * sc) as i32 / 2 * 2);
            let img = d::gem(g, 0);
            s.blr(&img, img.get_rect().center_at(r.center()));
            return;
        }
        s.blr(&d::face(cell, kind.name(), c.dim), r);
        if kind == Face::Gem && !c.dim {
            let ph = (self.time * 2.6 + i as f64 * 1.7).sin();
            if ph > 0.55 {
                let a = (ph - 0.55) / 0.45;
                let v = d::hash(i as i64 * 13 + 5);
                let sx = r.x as f64 + cf * (0.30 + (v % 40) as f64 / 100.0);
                let sy = r.y as f64 + cf * (0.22 + ((v >> 8) % 30) as f64 / 100.0);
                d::sparkle(s, sx, sy, 1.5f64.max(cf * 0.10 * a), (220, 255, 246));
            }
        } else if kind == Face::Boom {
            // glimmender Rand um die getroffene Mine
            let pulse = 0.5 + 0.5 * (self.time * 5.0).sin();
            draw::rect_r(
                s,
                ui::mix((255, 90, 70), (255, 200, 120), pulse),
                r.inflate(2, 2),
                2.max((3.0 * self.k) as i32),
                2.max((cf * 0.16) as i32),
            );
        }
    }

    fn draw_particles(&self, s: &mut Surface) {
        for p in &self.particles {
            let f = p.t / p.life;
            let (r, col): (f64, Color) = match p.kind {
                PKind::Smoke => {
                    // weicher Rauch: additiver grauer Lichthof, wird groesser und blasser
                    let lvl = 5.min(((1.0 - f) * 6.0) as i32);
                    if lvl <= 0 {
                        continue;
                    }
                    let rr = 2.max((p.r * (1.0 + 2.2 * f)) as i32 / 2 * 2);
                    let a = lvl as f64 / 6.0;
                    let g = d::glow(rr, ((p.col.0 as f64 * a) as i32, (p.col.1 as f64 * a) as i32, (p.col.2 as f64 * a) as i32));
                    s.bl_flags(&g, (p.x as i32 - rr - 1, p.y as i32 - rr - 1), BLEND_RGB_ADD);
                    continue;
                }
                PKind::Star => {
                    d::sparkle(s, p.x, p.y, 1f64.max(p.r * 1.6 * (1.0 - f)), (255, 255, 255));
                    continue;
                }
                PKind::Fire => (p.r * (1.0 - f), ui::mix(p.col, (120, 40, 30), f)),
                _ => (p.r * (1.0 - f), p.col.c()),
            };
            if r >= 0.5 {
                draw::circle(s, col, (p.x as i32, p.y as i32), 1.max(r as i32), 0);
            }
        }
    }

    // ===================================================== Zeichnen: Seitenleiste

    fn draw_panel(&self, s: &mut Surface) {
        if self.window == Some(Win::Auto) {
            self.draw_auto(s); // eigenes Fenster an derselben Stelle
            return;
        }
        ui::draw_panel_ex(s, self.panel_r, None, None, None, true, Some(self.b.accent));
        self.draw_mascot(s, None);
        self.draw_head(s);
        if self.state == State::Idle && self.round.is_none() {
            self.draw_setup(s);
            self.draw_info(s, self.r_presets[0].bottom() + 4.max((8.0 * self.k) as i32));
        } else {
            self.draw_info(s, self.info_y);
        }
        self.draw_ladder(s);
        self.draw_hint(s);
    }

    /// Maskottchen mit Spitzhacke und Lore in r (Standard: Kopf der Seitenleiste).
    fn draw_mascot(&self, s: &mut Surface, r: Option<Rect>) {
        let pal = ui::pal();
        let r = r.unwrap_or(self.r_mascot);
        let ms = r.w;
        let msf = ms as f64;
        let k = self.k;
        let (mood, mt) = self.mood;
        let (mut pose, mut dx, mut dy) = ("stand", 0.0f64, 0.0f64);
        if mood == Mood::Happy && mt < 0.45 {
            pose = "hop";
            dy = -(PI * mt / 0.45).sin() * msf * 0.14;
        } else if mood == Mood::Cheer && mt < 1.35 {
            let ph = py::fpmod(mt, 0.45) / 0.45;
            pose = "hop";
            dy = -(PI * ph).sin() * msf * 0.16;
        } else if mood == Mood::Scared || (self.sooty && self.state == State::Boom) {
            pose = "fall";
            if mt < 0.6 {
                dx = (mt * 60.0).sin() * msf * 0.04 * (1.0 - mt / 0.6);
                dy = -(PI * 1f64.min(mt / 0.35)).sin() * msf * 0.08;
            }
        }
        if pose == "stand" {
            dy += (self.time * 2.2).sin() * msf * 0.012;
        }
        let ground = r.bottom() - 1.max((2.0 * k) as i32);
        // Felsvorsprung, Spitzhacke, Lore
        draw::ellipse(
            s,
            (34, 30, 44),
            (r.x - (msf * 0.04) as i32, ground - (msf * 0.05) as i32, (msf * 1.0 + self.cart_w as f64 * 1.05) as i32, (msf * 0.12) as i32),
            0,
        );
        d::draw_pickaxe(s, (r.x + (msf * 0.10) as i32) as f64, (ground - (msf * 0.02) as i32) as f64, msf * 0.40, k);
        let sh = Rect::new(0, 0, (msf * 0.5) as i32, 3.max((msf * 0.07) as i32)).center_at((r.x + (msf * 0.52) as i32, ground));
        draw::ellipse(s, (14, 12, 20), sh, 0);
        let img = d::miner_llama(ms, pose, self.b.accent, self.sooty);
        s.bl(&img, ((r.x as f64 + dx) as i32, (r.y as f64 + dy + msf * 0.04) as i32));
        // Lichthof der Stirnlampe (additiv)
        let lr = 3.max((msf * 0.10) as i32);
        let flick = 0.85 + 0.15 * (self.time * 9.0).sin() * (self.time * 3.3).sin();
        let g = d::glow(lr, ((150.0 * flick) as i32, (130.0 * flick) as i32, (60.0 * flick) as i32));
        let lx = r.x as f64 + dx + msf * 0.835;
        let ly = r.y as f64 + dy + msf * 0.04 + msf * 0.073;
        s.bl_flags(&g, ((lx - lr as f64 - 1.0) as i32, (ly - lr as f64 - 1.0) as i32), BLEND_RGB_ADD);
        // Lore mit Edelsteinen
        d::draw_cart(s, r.x + (msf * 0.92) as i32, ground, self.cart_w, k, self.cart_fill, 4.max((self.cart_w as f64 * 0.30) as i32));
        // Reaktion: Funkeln (Freude) bzw. Ausrufezeichen + Rauch (Schreck)
        let (hx, hy) = (r.x as f64 + dx + msf * 0.74, r.y as f64 + dy + msf * 0.04);
        if matches!(mood, Mood::Happy | Mood::Cheer) && mt < if mood == Mood::Happy { 0.6 } else { 1.4 } {
            for j in 0..3 {
                let a = self.time * 2.0 + j as f64 * 2.1;
                d::sparkle(
                    s,
                    hx + a.cos() * msf * 0.22,
                    hy - msf * 0.04 + a.sin() * msf * 0.10,
                    1.5f64.max(msf * 0.05 * (0.6 + 0.4 * (self.time * 9.0 + j as f64).sin())),
                    (200, 255, 236),
                );
            }
        } else if mood == Mood::Scared && mt < 1.6 {
            let (bx, by) = ((hx + msf * 0.18) as i32, (hy - msf * 0.02) as i32);
            let rr = 5.max((msf * 0.09) as i32);
            draw::circle(s, (250, 250, 250), (bx, by), rr, 0);
            draw::circle(s, pal.red, (bx, by), rr, 1.max((2.0 * k) as i32));
            let ex = self.f_btn.render("!", true, pal.red);
            s.blr(&ex, ex.get_rect().center_at((bx, by)));
            for j in 0..3 {
                let ph = py::fpmod(mt * 0.8 + j as f64 * 0.33, 1.0);
                let pr = 2.max((msf * (0.05 + 0.07 * ph)) as i32 / 2 * 2);
                let a = 1.0 - ph;
                let g = d::glow(pr, ((80.0 * a) as i32, (76.0 * a) as i32, (86.0 * a) as i32));
                let px = (r.x as f64 + dx + msf * (0.66 - 0.06 * j as f64)) as i32;
                let pyy = (r.y as f64 + dy - msf * 0.20 * ph + msf * 0.02) as i32;
                s.bl_flags(&g, (px - pr - 1, pyy - pr - 1), BLEND_RGB_ADD);
            }
        }
    }

    /// Kopfzeile rechts neben Maskottchen und Lore: grosser Multiplikator.
    fn draw_head(&self, s: &mut Surface) {
        let p = ui::pal();
        let x0 = self.head_x;
        let x1 = self.panel_r.right() - self.ppad;
        let w = x1 - x0;
        let mut y = self.r_mascot.y;
        let (cap, val, sub, col): (String, String, Option<String>, Color) = if self.state == State::Idle && self.round.is_none() {
            // Edelsteine/Minen als Bildchen (sprachunabhaengig)
            (t("lmi.max_mult"), self.fmt(ml::mult100(self.grid, self.mines, ml::gems(self.grid, self.mines))), None, cd::COL_GOLD.c())
        } else if self.state == State::Boom {
            (t("lmi.cur_mult"), "\u{00D7}0".to_string(), Some(tf("lmi.lost_short", &[("n", &self.group(self.lost))])), p.red)
        } else if self.state == State::Cashed {
            (t("lmi.cur_mult"), self.fmt(self.last_mult), Some(format!("+{}", self.group(self.win))), cd::COL_GOLD.c())
        } else {
            let k = self.kk();
            (
                t("lmi.cur_mult"),
                self.fmt(ml::mult100(self.grid, self.mines, k)),
                Some(tf("lmi.payout_now", &[("n", &self.group(self.win_at(self.round_bet(), k)))])),
                if k > 0 { p.green } else { p.text },
            )
        };
        let img = self.fit(&cap, &[&self.f_tiny, &self.f_micro], w, p.text_dim);
        s.bl(&img, (x0, y));
        y += img.h();
        let img = self.fit(&val, &[&self.f_mult, &self.f_big, &self.f_btn], w, col);
        s.bl(&img, (x0, y));
        y += img.h();
        let Some(sub) = sub else {
            let ih = self.f_small.get_height().max((self.f_big.get_height() as f64 * 0.9) as i32);
            self.draw_counts(s, x0, y + ih / 2 + 1, ml::gems(self.grid, self.mines), self.mines, &self.f_btn, ih, None);
            return;
        };
        let img = self.fit(&sub, &[&self.f_small, &self.f_tiny, &self.f_micro], w, p.text);
        s.bl(&img, (x0, y));
    }

    /// Edelstein- und Minen-Bildchen mit Anzahl (links bei x bzw. mittig in
    /// center_w), senkrecht mittig bei cy.
    #[allow(clippy::too_many_arguments)]
    fn draw_counts(&self, s: &mut Surface, x: i32, cy: i32, gems: i32, mines: i32, font: &Font, ih: i32, center_w: Option<i32>) {
        let parts = [
            (d::gem(ih, 0), font.render(&self.group(gems as i64), true, self.b.accent)),
            (d::mine(ih, true), font.render(&mines.to_string(), true, ui::pal().red)),
        ];
        let gap = 8.max(ih * 2 / 3);
        let mut total = 0;
        for (icon, txt) in &parts {
            total += icon.w() + 3 + txt.w();
        }
        total += gap;
        let mut x = x;
        if let Some(cw) = center_w {
            x += py::fdiv32(cw - total, 2);
        }
        for (icon, txt) in &parts {
            s.blr(icon, icon.get_rect().midleft_at((x, cy)));
            x += icon.w() + 3;
            s.blr(txt, txt.get_rect().midleft_at((x, cy)));
            x += txt.w() + gap;
        }
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (x0, w) = self.panel_inner;
        let hot = self.mouse;
        let img = self.fit(&t("lmi.grid"), &[&self.f_tiny], w, p.text_dim);
        s.bl(&img, (x0, self.lab_grid_y));
        for (i, r) in self.r_grids.iter().enumerate() {
            let g = ml::GRIDS[i];
            self.btn_s(s, *r, &format!("{g}\u{00D7}{g}"), true, r.hit(hot), g == self.grid, false);
            let key = self.f_micro.render(&(i + 1).to_string(), true, p.text_faint);
            s.bl(&key, (r.x + 4.max(r.h / 5), r.y + 2.max(r.h / 10)));
        }
        let y = self.mines_row_y;
        let rh = self.row_h;
        let lab = self.fit(&t("lmi.mines"), &[&self.f_small, &self.f_tiny], self.r_mminus.x - x0 - 4, p.text);
        s.blr(&lab, lab.get_rect().midleft_at((x0, y + rh / 2)));
        self.btn_s(s, self.r_mminus, "", self.mines > 1, self.r_mminus.hit(hot), false, false);
        let rm = self.r_mminus;
        d::icon(s, "left", rm.inflate(py::fdiv32(-rm.w, 3), py::fdiv32(-rm.h, 3)), if self.mines > 1 { p.text } else { p.text_faint });
        let mx = ml::max_mines(self.grid);
        self.btn_s(s, self.r_mplus, "", self.mines < mx, self.r_mplus.hit(hot), false, false);
        let rp = self.r_mplus;
        d::icon(s, "right", rp.inflate(py::fdiv32(-rp.w, 3), py::fdiv32(-rp.h, 3)), if self.mines < mx { p.text } else { p.text_faint });
        draw::rect_r(s, (16, 12, 26), self.r_mval, 0, 4.max(rh / 4));
        draw::rect_r(s, ui::mix(p.red, p.border, 0.4), self.r_mval, 1, 4.max(rh / 4));
        let val = self.f_btn.render(&self.mines.to_string(), true, p.red);
        s.blr(&val, val.get_rect().center_at(self.r_mval.center()));
        let presets = ml::mine_presets(self.grid);
        for (i, r) in self.r_presets.iter().enumerate() {
            let v = presets[i];
            self.btn_s(s, *r, &v.to_string(), true, r.hit(hot), v == self.mines, true);
        }
    }

    /// Zeilen der Gewinn-Info: [(Beschriftung, Wert, Farbe)], Index der Zeile
    /// mit dem Chancen-Balken (oder None) und die Chance in Zehntel-Prozent.
    fn info_rows(&self) -> (Vec<(String, String, Color)>, Option<usize>, i64) {
        let p = ui::pal();
        let (g, m) = (self.grid, self.mines);
        if matches!(self.state, State::Boom | State::Cashed) {
            let o = self.fairlog.last().and_then(|e| e.o.as_object());
            let bet = o.map(|o| py_int(o.get("bet")).unwrap_or(0)).unwrap_or(0);
            let paid = if self.state == State::Cashed { self.win } else { 0 };
            let rows = vec![
                (t("cas.bet"), self.group(bet), p.text),
                (t("lmi.found"), format!("{} / {}", self.last_k(), ml::gems(g, m)), p.text),
                (t("chip.st_paid"), self.group(paid), if paid != 0 { cd::COL_GOLD.c() } else { p.red }),
            ];
            return (rows, None, 0);
        }
        let k = if self.state == State::Play { self.kk() } else { 0 };
        let bet = self.round_bet();
        let nxt = ml::next_mult100(g, m, k);
        let p10 = ml::safe_percent(g, m, k);
        let mut rows = vec![
            (t("lmi.next_mult"), nxt.map(|v| self.fmt(v)).unwrap_or_else(|| "\u{2013}".into()), p.green),
            (t("lmi.safe"), format!("{} %", self.fmt_pct(p10)), p.text),
            (
                t("lmi.next_win"),
                if nxt.is_some() { self.group(self.win_at(bet, k + 1)) } else { "\u{2013}".into() },
                cd::COL_GOLD.c(),
            ),
        ];
        if self.state == State::Play {
            rows.push((t("lmi.found"), format!("{} / {}", k, ml::gems(g, m)), p.text));
        }
        (rows, Some(1), p10)
    }

    /// Gewinn-Info (Ruhe: erstes Feld, Runde: naechstes Feld, Ergebnis: Bilanz).
    fn draw_info(&self, s: &mut Surface, y0: i32) {
        let p = ui::pal();
        let (x0, w) = self.panel_inner;
        let mut y = y0;
        let (mut rows, mut bar_row, p10) = self.info_rows();
        let bh = 4.max((6.0 * self.k) as i32);
        let mut bar_total = if bar_row.is_some() { bh + 2.max((4.0 * self.k) as i32) } else { 0 };
        let avail = self.ladder_lab_y - 4.max((6.0 * self.k) as i32) - y;
        let min_lh = self.f_tiny.get_height() + 2;
        let max_lh = (self.f_btn.get_height() as f64 * 1.75) as i32;
        while !rows.is_empty() && py::fdiv32(avail - bar_total, rows.len() as i32) < min_lh {
            rows.pop();
            if let Some(b) = bar_row {
                if b >= rows.len() {
                    bar_row = None;
                    bar_total = 0;
                }
            }
        }
        if rows.is_empty() {
            return;
        }
        let lh = max_lh.min(py::fdiv32(avail - bar_total, rows.len() as i32));
        for (j, (lab, val, col)) in rows.iter().enumerate() {
            let vimg = self.fit(val, &[&self.f_btn, &self.f_tiny], w / 2, *col);
            let limg = self.fit(lab, &[&self.f_small, &self.f_tiny, &self.f_micro], w - vimg.w() - 8, p.text_dim);
            s.blr(&limg, limg.get_rect().midleft_at((x0, y + py::fdiv32(lh, 2))));
            s.blr(&vimg, vimg.get_rect().midright_at((x0 + w, y + py::fdiv32(lh, 2))));
            y += lh;
            if Some(j) == bar_row {
                // Balken: gruen = sicher, rot = Mine
                let bar = Rect::new(x0, y - 1.max(py::fdiv32(lh, 8)), w, bh);
                draw::rect_r(s, (110, 30, 40), bar, 0, bh / 2);
                let gw = ((w as i64 * p10) as f64 / 1000.0) as i32;
                if gw > 0 {
                    draw::rect_r(s, (40, 170, 120), (bar.x, bar.y, gw, bh), 0, bh / 2);
                }
                y += bar_total;
            }
        }
        // Noch im Feld: verbleibende Edelsteine und Minen als Bildchen
        let ih = 18.max((30.0 * self.k) as i32);
        let room = self.ladder_lab_y - 4.max((6.0 * self.k) as i32) - y;
        if self.state == State::Play && room >= ih + 8.max((12.0 * self.k) as i32) {
            let left = ml::gems(self.grid, self.mines) - self.kk();
            self.draw_counts(s, x0, y + py::fdiv32(room, 2), left, self.mines, &self.f_big, ih, Some(w));
        }
    }

    fn last_k(&self) -> i32 {
        if let Some(e) = self.fairlog.last() {
            if let Some(o) = e.o.as_object() {
                if let Some(Value::Array(open)) = o.get("open") {
                    let n = open.len() as i32;
                    return if o.get("m").and_then(num) == Some(0.0) { n - 1 } else { n };
                }
            }
        }
        0
    }

    /// Leiste der naechsten 5 Stufen (in Ruhe: die ersten 5).
    fn draw_ladder(&self, s: &mut Surface) {
        let p = ui::pal();
        let (x0, w) = self.panel_inner;
        let k = if self.state == State::Play { self.kk() } else { 0 };
        let steps = ml::ladder(self.grid, self.mines, k, 5);
        let img = self.fit(&t("lmi.ladder"), &[&self.f_tiny, &self.f_micro], w, p.text_dim);
        s.bl(&img, (x0, self.ladder_lab_y));
        for (i, r) in self.r_pills.iter().enumerate() {
            if i >= steps.len() {
                draw::rect_r(s, (20, 18, 30), *r, 1, r.h / 2);
                continue;
            }
            let (_j, m100) = steps[i];
            let nxt = i == 0 && self.state == State::Play;
            let bg: Color = if nxt { cd::COL_GOLD.c() } else { rgb(26, 24, 40) };
            let fg: Color = if nxt { rgb(30, 24, 10) } else { rgb(232, 232, 240) };
            draw::rect_r(s, bg, *r, 0, r.h / 2);
            if !nxt {
                draw::rect_r(s, ui::mix(self.b.accent, p.border, 0.6), *r, 1, r.h / 2);
            }
            let txt = self.fmt_short(m100);
            let img = self.fit(&txt, &[&self.f_tiny, &self.f_micro], r.w - 4, fg);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
    }

    fn draw_hint(&self, s: &mut Surface) {
        let (x0, w) = self.panel_inner;
        let text = if let Some(a) = self.auto.as_ref() {
            tf("chip.auto_left", &[("n", &a.left.to_string())])
        } else if self.state == State::Play {
            t("lmi.hint_play")
        } else {
            t("lmi.hint_idle")
        };
        let lines = Self::wrap(&text, &self.f_micro, w);
        let mut y = self.hint_y;
        for line in lines.iter().take(2) {
            let img = self.f_micro.render(line, true, ui::pal().text_faint);
            s.bl(&img, (x0, y));
            y += self.f_micro.get_height() + 1;
        }
    }

    // ----- Auto-Fenster (ueber der Seitenleiste) -----------------------------------

    fn draw_auto(&self, s: &mut Surface) {
        let pal = ui::pal();
        let p = self.panel_r;
        ui::draw_panel_ex(s, p, None, None, None, true, Some(self.b.accent));
        let (x0, w) = self.panel_inner;
        let hot = self.mouse;
        let title = self.fit(&t("chip.auto_title"), &[&self.f_big, &self.f_btn], self.r_auto_close.x - x0 - 4, pal.text);
        s.blr(&title, title.get_rect().midleft_at((x0, self.r_auto_close.centery())));
        self.btn_s(s, self.r_auto_close, "", true, self.r_auto_close.hit(hot), false, false);
        let rc = self.r_auto_close;
        d::icon(s, "close", rc.inflate(py::fdiv32(-rc.w, 4), py::fdiv32(-rc.h, 4)), pal.text);
        let labels = [t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
        let values = [self.auto_rounds, self.auto_win, self.auto_loss];
        for (row, (ly, rects)) in self.auto_rows.iter().enumerate() {
            let img = self.fit(&labels[row], &[&self.f_tiny, &self.f_micro], w, pal.text_dim);
            s.bl(&img, (x0, *ly));
            let opts: &[i64] = if row == 0 { &ROUND_OPTS } else { &LIMIT_OPTS };
            for (j, r) in rects.iter().enumerate() {
                let v = opts[j];
                let lab = if row == 0 {
                    v.to_string()
                } else if v == 0 {
                    t("chip.auto_off")
                } else {
                    cd::fmt_amount(v)
                };
                self.btn_s(s, *r, &lab, true, r.hit(hot), v == values[row], true);
            }
        }
        // Markierungen + Gewinn bei Erfolg
        let mut y = self.auto_info_y;
        let n = self.marks_now().len() as i32;
        let g = ml::gems(self.grid, self.mines);
        let img = self.fit(&tf("lmi.marked", &[("n", &n.to_string())]), &[&self.f_btn, &self.f_tiny], w, if n != 0 { pal.text } else { pal.text_dim });
        s.bl(&img, (x0, y));
        y += img.h() + 1;
        let (info, col) = if 0 < n && n <= g {
            let x: String = self.fmt(ml::mult100(self.grid, self.mines, n)).chars().skip(1).collect();
            let pp = self.fmt_pct(ml::all_safe_percent(self.grid, self.mines, n));
            (tf("lmi.marked_info", &[("x", &x), ("p", &pp)]), cd::COL_GOLD.c())
        } else if n > g {
            (tf("lmi.auto_too_many", &[("n", &g.to_string())]), pal.red)
        } else {
            (t("lmi.hint_auto"), pal.text_dim)
        };
        let room = self.r_auto_start.y - y - 4;
        let fh = self.f_tiny.get_height();
        let nmax = 1.max(py::fdiv32(room, 1.max(fh))) as usize;
        for line in Self::wrap(&info, &self.f_tiny, w).iter().take(nmax) {
            let img = self.f_tiny.render(line, true, col);
            s.bl(&img, (x0, y));
            y += fh;
        }
        let mh = self.f_micro.get_height();
        if 0 < n && n <= g && y + mh < self.r_auto_start.y - 4 {
            for line in Self::wrap(&t("lmi.hint_auto"), &self.f_micro, w) {
                if y + mh >= self.r_auto_start.y - 4 {
                    break;
                }
                let img = self.f_micro.render(&line, true, pal.text_faint);
                s.bl(&img, (x0, y));
                y += mh;
            }
        }
        // freier Platz: das Maskottchen schaut zu
        let free = self.r_auto_start.y - y - 2 * self.ppad;
        let ms = self.r_mascot.w;
        if free >= ms + 4 {
            let mw = ms + (self.cart_w as f64 * 1.0) as i32;
            let rr = Rect::new(x0 + py::fdiv32(w - mw, 2), y + self.ppad + py::fdiv32(free - ms, 2), ms, ms);
            if rr.w != 0 && rr.h != 0 {
                self.draw_mascot(s, Some(rr));
            } else {
                self.draw_mascot(s, None);
            }
        }
        self.btn_s(s, self.r_auto_clear, &t("lmi.clear_marks"), n > 0, self.r_auto_clear.hit(hot), false, false);
        self.btn(s, self.r_auto_start, &t("chip.auto_start"), true, 0 < n && n <= g, self.r_auto_start.hit(hot), false, false, None, false);
    }

    // ----- Kopfzeile, Werkzeuge, Verlauf ---------------------------------------------

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        s.bl(&self.hud_bg, (0, 0));
        draw::line(s, ui::mix(cd::COL_GOLD, self.b.accent, 0.35), (0, self.hud_h - 1), (w, self.hud_h - 1), 1);
        let cy = self.hud_h / 2;
        let pad = self.margin;
        let coin_r = 6.max((self.hud_h as f64 * 0.26) as i32);
        let coin = cd::symbol_surface("coin", coin_r * 2 + 4, false);
        s.blr(&coin, coin.get_rect().midleft_at((pad - 2, cy)));
        let left = self.f_big.render(&format!("{}: {}", t("cas.chips"), self.group(py::round(self.shown_chips))), true, p.gold);
        s.blr(&left, left.get_rect().midleft_at((pad + coin_r * 2 + 6, cy)));
        let right_txt = format!(
            "{}: {}   \u{00B7}   {}",
            t("cas.record"),
            self.group(self.b.score),
            tf("cas.credits", &[("n", &lamabank::refills().to_string())])
        );
        let mut right = self.f_small.render(&right_txt, true, p.text_dim);
        if right.w() as f64 > w as f64 * 0.42 {
            right = self.f_tiny.render(&right_txt, true, p.text_dim);
        }
        let rr = right.get_rect().midright_at((w - pad, cy));
        s.blr(&right, rr);
        let Some(round) = self.round.as_ref() else {
            return;
        };
        let bet = round.bet;
        let k = self.kk();
        let mut mid_txt = format!("{}: {}", t("cas.bet"), self.group(bet));
        if k > 0 {
            let m100 = ml::mult100(self.grid, self.mines, k);
            mid_txt += &format!("   \u{00B7}   {} = {}", self.fmt(m100), self.group(self.win_at(bet, k)));
        }
        let x0 = pad + coin_r * 2 + 6 + left.w() + pad;
        let x1 = rr.x - pad;
        let mut mid = self.f_small.render(&mid_txt, true, p.text);
        if mid.w() > x1 - x0 {
            mid = self.f_tiny.render(&mid_txt, true, p.text);
        }
        if mid.w() <= x1 - x0 {
            s.blr(&mid, mid.get_rect().center_at((py::fdiv32(x0 + x1, 2), cy)));
        }
    }

    fn tool_active(&self, name: &str) -> bool {
        match name {
            "turbo" => self.turbo,
            "auto" => self.window == Some(Win::Auto) || self.auto.is_some(),
            _ => self.window.map(|w| w.name()) == Some(name),
        }
    }

    fn draw_tools(&self, s: &mut Surface) {
        let p = ui::pal();
        let hot = self.mouse;
        for (name, r) in &self.r_tools {
            let r = *r;
            let active = self.tool_active(name);
            let h = r.hit(hot);
            let rad = 5.max(r.h / 4);
            let fill = if active || h { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, r, 0, rad);
            let mut edge = if active {
                self.b.accent
            } else if h {
                p.border_light
            } else {
                p.border
            };
            if *name == "auto" && self.auto.is_some() {
                edge = ui::mix(self.b.accent, (255, 255, 255), 0.5 * (0.5 + 0.5 * (self.time * 5.0).sin()));
            }
            draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
            let col = if active { self.b.accent } else { p.text };
            let ir = Rect::new(r.x, r.y, r.h, r.h).inflate(py::fdiv32(-r.h, 3), py::fdiv32(-r.h, 3));
            d::icon(s, name, ir, col);
            if self.tool_labels {
                let img = self.f_tiny.render(&tool_label(name), true, if active || h { p.text } else { p.text_dim });
                s.blr(&img, img.get_rect().midleft_at((r.x + r.h - 2.max(r.h / 8), r.centery())));
            }
        }
    }

    fn draw_tooltip(&self, s: &mut Surface) {
        let p = ui::pal();
        for (name, r) in &self.r_tools {
            if r.hit(self.mouse) {
                let txt = format!("{} ({})", tool_label(name), tool_key(name));
                let img = self.f_tiny.render(&txt, true, p.text);
                let bx = img.get_rect().inflate(12, 6).topleft_at((r.x, r.bottom() + 4));
                draw::rect_r(s, (12, 10, 22), bx, 0, bx.h / 2);
                draw::rect_r(s, p.border_light, bx, 1, bx.h / 2);
                s.blr(&img, img.get_rect().center_at(bx.center()));
            }
        }
    }

    /// Letzte Runden als Schilder oben rechts (neueste rechts).
    fn draw_history(&self, s: &mut Surface) {
        let mut x = self.b.width - self.margin;
        let y = self.r_tools[0].1.y;
        let h = self.tool_h;
        let limit = self.tools_right + self.margin;
        for &m100 in self.hist.iter().rev() {
            let txt = if m100 > 0 { self.fmt_short(m100) } else { "\u{00D7}0".to_string() };
            let img = self.f_tiny.render(&txt, true, rgb(240, 240, 245));
            let mut bx = img.get_rect();
            bx.w += (12.0 * self.k) as i32 + 4;
            bx.h = (h as f64 * 0.78) as i32;
            let bx = bx.topright_at((x, y + py::fdiv32(h - bx.h, 2)));
            if bx.x < limit {
                break;
            }
            let col = if m100 >= 200 {
                (34, 132, 100)
            } else if m100 > 100 {
                (62, 96, 82)
            } else if m100 > 0 {
                (70, 72, 92)
            } else {
                (128, 36, 46)
            };
            draw::rect_r(s, col, bx, 0, bx.h / 2);
            s.blr(&img, img.get_rect().center_at(bx.center()));
            x = bx.x - 4;
        }
    }

    // ----- Bedienleiste -----------------------------------------------------------

    fn draw_strip(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.strip_bg, (self.strip.x, self.strip.y));
        draw::line(s, ui::mix(cd::COL_GOLD, self.b.accent, 0.35), (self.strip.x, self.strip.y), (self.strip.right(), self.strip.y), 2);
        let idle = self.state == State::Idle && self.auto.is_none();
        let bet = self.round_bet();
        let hot = self.mouse;
        self.btn_s(s, self.r_minus, "\u{2013}", idle && self.bet > ml::BETS[0], self.r_minus.hit(hot), false, false);
        self.btn_s(s, self.r_plus, "+", idle && self.bet < ml::BETS[ml::BETS.len() - 1], self.r_plus.hit(hot), false, false);
        draw::rect_r(s, (16, 12, 26), self.r_bet, 0, 6.max(self.r_bet.h / 4));
        draw::rect_r(s, p.border, self.r_bet, 1, 6.max(self.r_bet.h / 4));
        let lbl = self.f_tiny.render(&t("cas.bet"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midtop_at((self.r_bet.centerx(), self.r_bet.y + 2)));
        let col = if idle && bet > lamabank::balance() { p.red } else { p.gold };
        let val = self.fit(&self.group(bet), &[&self.f_btn, &self.f_tiny], self.r_bet.w - 4, col);
        s.blr(&val, val.get_rect().midbottom_at((self.r_bet.centerx(), self.r_bet.bottom() - 3)));
        let play = self.state == State::Play;
        if let Some(a) = self.auto.as_ref() {
            // Mitte: Fortschritt, rechts: STOPP
            let txt = tf("chip.auto_left", &[("n", &a.left.to_string())]);
            let img = self.fit(&txt, &[&self.f_btn, &self.f_tiny], self.r_cash.x - self.r_plus.right() - 2 * self.margin, self.b.accent);
            s.blr(&img, img.get_rect().center_at((py::fdiv32(self.r_plus.right() + self.r_cash.x, 2), self.r_go.centery())));
            self.btn(s, self.r_go, &t("chip.auto_stop"), true, true, self.r_go.hit(hot), false, false, None, true);
        } else {
            let ready = idle || (matches!(self.state, State::Boom | State::Cashed) && self.result_visible() && self.res_t >= READ_T);
            self.btn(s, self.r_go, &t("lmi.play"), true, ready && !self.broke && self.window.is_none(), self.r_go.hit(hot), false, false, None, false);
            if play {
                let on = self.window.is_none();
                let lab = if self.random_label { t("lmi.random") } else { String::new() };
                self.btn(s, self.r_random, &lab, false, on, self.r_random.hit(hot), false, false, Some("dice"), false);
            }
        }
        let mut cash = t("lmi.cashout");
        let can = play && self.kk() > 0 && self.auto.is_none();
        if can {
            if let Some(r) = self.round.as_ref() {
                cash += &format!(" {}", self.group(self.win_at(r.bet, self.kk())));
            }
        }
        self.btn_s(s, self.r_cash, &cash, can, self.r_cash.hit(hot), can, false);
    }

    // ----- Effekte, Ergebnis, Hinweise ---------------------------------------------

    fn draw_fx(&self, s: &mut Surface) {
        for c in &self.coins_fx {
            let w = 2.max((c.r * 2.0 * c.spin.cos().abs()) as i32);
            let h = (c.r * 2.0) as i32;
            let rect = Rect::new(0, 0, w, h).center_at((c.x as i32, c.y as i32));
            draw::ellipse(s, cd::COL_GOLD_D, rect.move_(1, 1), 0);
            draw::ellipse(s, (246, 196, 60), rect, 0);
            if w > 6 {
                draw::ellipse(s, (255, 236, 150), rect.inflate(py::fdiv32(-w, 2), py::fdiv32(-h, 2)), 0);
            }
        }
        for p in &self.popups {
            let f = p.t / p.dur;
            let mut img = self.f_big.render(&p.text, true, p.color);
            if f > 0.6 {
                img = img.copy();
                img.alpha_set((255.0 * (1.0 - (f - 0.6) / 0.4)) as i32);
            }
            let y = p.y as f64 - 40.0 * self.k * (1.0 - (1.0 - f) * (1.0 - f));
            s.blr(&img, img.get_rect().center_at((p.x, y as i32)));
        }
    }

    fn draw_result(&self, s: &mut Surface) {
        let Some((title, sub, col)) = self.result.as_ref() else {
            return;
        };
        let br = self.board_r;
        let k = self.k;
        let maxw = br.w - (24.0 * k) as i32;
        let all = [&self.f_huge, &self.f_mult, &self.f_big, &self.f_btn];
        let fonts: &[&Font] = if self.f_huge.width(title) as f64 > br.w as f64 * 0.62 { &all[1..] } else { &all[..] };
        let timg = self.fit(title, fonts, maxw, *col);
        let simg = self.fit(sub, &[&self.f_small, &self.f_tiny, &self.f_micro], maxw, ui::pal().text);
        let pw = (br.w - 8).min(timg.w().max(simg.w()) + (40.0 * k) as i32);
        let ph = timg.h() + simg.h() + (30.0 * k) as i32;
        let mut panel = Rect::new(0, 0, pw, ph).center_at(br.center());
        let f = 1f64.min(self.res_t / 0.18);
        if f < 1.0 {
            panel = panel.inflate(-((pw as f64 * 0.15 * (1.0 - f)) as i32), -((ph as f64 * 0.15 * (1.0 - f)) as i32));
        }
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(*col));
        s.blr(&timg, timg.get_rect().midtop_at((panel.centerx(), panel.y + (10.0 * k) as i32)));
        s.blr(&simg, simg.get_rect().midbottom_at((panel.centerx(), panel.bottom() - (10.0 * k) as i32)));
    }

    fn draw_banner(&self, s: &mut Surface) {
        let Some((text, left)) = self.banner.as_ref() else {
            return;
        };
        let br = self.board_r;
        let pulse = 0.5 + 0.5 * (self.time * 8.0).sin();
        let col = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.4 * pulse);
        let fonts = [&self.f_huge, &self.f_mult, &self.f_big];
        let mut font = fonts[fonts.len() - 1];
        for f in fonts {
            if f.width(text) <= br.w - 16 {
                font = f;
                break;
            }
        }
        let mut img = self.fit(text, &[font], br.w - 16, col);
        let mut sh = self.fit(text, &[font], br.w - 16, (40, 20, 0));
        let y = br.y + (br.h as f64 * 0.2) as i32;
        if *left < 0.4 {
            let a = (255.0 * left / 0.4) as i32;
            img = img.copy();
            img.alpha_set(a);
            sh = sh.copy();
            sh.alpha_set(a);
        }
        s.blr(&sh, sh.get_rect().center_at((br.centerx() + 3, y + 3)));
        s.blr(&img, img.get_rect().center_at((br.centerx(), y)));
    }

    fn draw_toast(&self, s: &mut Surface) {
        let Some((text, left)) = self.toast.as_ref() else {
            return;
        };
        let mut img = self.fit(text, &[&self.f_small, &self.f_tiny], self.b.width - 40, ui::pal().text);
        let mut bx = img.get_rect().inflate(24, 12).midtop_at((self.board_r.centerx(), self.board_r.y + self.margin));
        if bx.x < 4 {
            bx.x = 4;
        }
        let mut panel = pg::surface_alpha(bx.w, bx.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(12, 8, 20, 225), pr, 0, bx.h / 2);
        draw::rect_r(&mut panel, pg::alpha(ui::mix(cd::COL_GOLD, self.b.accent, 0.4), 255), pr, 1, bx.h / 2);
        if *left < 0.3 {
            let a = (255.0 * left / 0.3) as i32;
            panel.alpha_set(a);
            img = img.copy();
            img.alpha_set(a);
        }
        s.blr(&panel, bx);
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    fn draw_broke(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let pw = (self.b.width - 40).min((480.0 * 0.8f64.max(self.k)) as i32);
        let ph = (200.0 * 0.75f64.max(self.k)) as i32;
        let panel = Rect::new(cx - pw / 2, cy - ph / 2, pw, ph);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(p.red));
        let head = self.fit(&t("cas.broke"), &[&self.f_huge, &self.f_big], pw - 16, p.red);
        s.blr(&head, head.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * 0.28)));
        let lines = [
            (t("cas.broke_sub"), p.text_dim),
            (tf("cas.broke_restart", &[("n", &lamabank::START_CHIPS.to_string())]), p.text),
        ];
        for (i, (txt, col)) in lines.iter().enumerate() {
            let img = self.fit(txt, &[&self.f_small, &self.f_tiny, &self.f_micro], pw - 16, *col);
            s.blr(&img, img.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * (0.58 + 0.2 * i as f64))));
        }
    }

    // ----- Statistik- und Fairness-Fenster ---------------------------------------------

    fn window_rect(&self) -> (Rect, Rect) {
        let (w0, h0, k, m) = (self.b.width, self.b.height, self.k, self.margin);
        let (w, h) = if self.window == Some(Win::Fair) {
            ((w0 - 2 * m).min((720.0 * k) as i32), (h0 - 2 * m).min((540.0 * k) as i32))
        } else {
            ((w0 - 2 * m).min((640.0 * k) as i32), (h0 - 2 * m).min((470.0 * k) as i32))
        };
        let rect = Rect::new(0, 0, w, h).center_at((w0 / 2, h0 / 2));
        let cb = 20.max((28.0 * k) as i32);
        let close = Rect::new(rect.right() - self.ppad - cb, rect.y + self.ppad, cb, cb);
        (rect, close)
    }

    fn draw_window(&mut self, s: &mut Surface) {
        let pal = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (rect, close) = self.window_rect();
        ui::draw_panel_ex(s, rect, None, None, None, true, Some(self.b.accent));
        let pad = self.ppad;
        let icon = Rect::new(rect.x + pad, close.y, close.h, close.h);
        let wname = self.window.map(|w| w.name()).unwrap_or("stats");
        d::icon(s, wname, icon.inflate(py::fdiv32(-icon.w, 6), py::fdiv32(-icon.h, 6)), self.b.accent);
        let mut title = if self.window == Some(Win::Stats) { t("chip.stats") } else { t("chip.fair_title") };
        title += "  \u{00B7}  Lama Mines";
        let img = self.fit(&title, &[&self.f_big, &self.f_btn], close.x - icon.right() - 2 * pad, pal.text);
        s.blr(&img, img.get_rect().midleft_at((icon.right() + pad / 2, close.centery())));
        self.btn_s(s, close, "", true, close.hit(self.mouse), false, false);
        d::icon(s, "close", close.inflate(py::fdiv32(-close.w, 4), py::fdiv32(-close.h, 4)), pal.text);
        let body = Rect::new(rect.x + pad, close.bottom() + pad, rect.w - 2 * pad, rect.bottom() - close.bottom() - 2 * pad);
        if self.window == Some(Win::Stats) {
            self.draw_stats(s, body);
        } else {
            self.draw_fair(s, body);
        }
    }

    fn draw_stats(&self, s: &mut Surface, body: Rect) {
        let p = ui::pal();
        let st = &self.stats;
        let k = self.k;
        let sep = self.sep();
        let mut rtp = "\u{2013}".to_string();
        if st.wagered > 0 {
            let p10 = py::fdiv(st.paid * 2000 + st.wagered, st.wagered * 2);
            rtp = format!("{} %", ml::fmt_percent(p10, &sep));
        }
        let net = st.net;
        let tiles: Vec<(String, String, Color)> = vec![
            (t("chip.st_rounds"), self.group(st.rounds), p.text),
            (t("chip.st_wins"), self.group(st.wins), p.green),
            (t("chip.st_losses"), self.group(st.losses), p.red),
            (t("chip.st_rtp"), rtp, p.text),
            (t("chip.st_wagered"), self.group(st.wagered), p.text),
            (t("chip.st_paid"), self.group(st.paid), p.text),
            (t("chip.st_net"), format!("{}{}", if net > 0 { "+" } else { "" }, self.group(net)), if net >= 0 { p.green } else { p.red }),
            (
                t("chip.st_best_win"),
                format!("{}{}", if st.best_win > 0 { "+" } else { "" }, self.group(st.best_win)),
                cd::COL_GOLD.c(),
            ),
            (
                t("chip.st_best_mult"),
                if st.best_mult > 0 { self.fmt(st.best_mult) } else { "\u{2013}".into() },
                cd::COL_GOLD.c(),
            ),
            (t("lmi.st_gems"), self.group(self.xint("gems")), self.b.accent),
            (t("lmi.st_booms"), self.group(self.xint("booms")), p.red),
            (t("lmi.st_clears"), self.group(self.xint("clears")), cd::COL_GOLD.c()),
        ];
        let cols = 4;
        let gap = 4.max((8.0 * k) as i32);
        let tw = py::fdiv32(body.w - (cols - 1) * gap, cols);
        let th = self.f_tiny.get_height() + self.f_big.get_height() + 8.max((14.0 * k) as i32);
        for (i, (lab, val, col)) in tiles.iter().enumerate() {
            let i = i as i32;
            let r = Rect::new(body.x + (i % cols) * (tw + gap), body.y + (i / cols) * (th + gap), tw, th);
            draw::rect_r(s, ui::mix(p.panel, (0, 0, 0), 0.25), r, 0, 4.max((8.0 * k) as i32));
            draw::rect_r(s, p.border, r, 1, 4.max((8.0 * k) as i32));
            let limg = self.fit(lab, &[&self.f_tiny, &self.f_micro], tw - 8, p.text_dim);
            s.bl(&limg, (r.x + 5, r.y + 3.max((5.0 * k) as i32)));
            let vimg = self.fit(val, &[&self.f_big, &self.f_btn, &self.f_tiny], tw - 8, *col);
            s.blr(&vimg, vimg.get_rect().bottomleft_at((r.x + 5, r.bottom() - 3.max((5.0 * k) as i32))));
        }
        let mut y = body.y + 3 * (th + gap) + gap;
        let curve = &st.curve;
        let lab = if !curve.is_empty() { tf("chip.st_curve", &[("n", &curve.len().to_string())]) } else { t("chip.st_empty") };
        let img = self.fit(&lab, &[&self.f_tiny, &self.f_micro], body.w, p.text_dim);
        s.bl(&img, (body.x, y));
        y += img.h() + 4 + pgz_engine::charts::fnt(10, false).get_height(); // Platz fuer die obere Achsen-Zahl
        let chart = Rect::new(body.x, y, body.w, body.bottom() - y);
        if curve.len() >= 2 && chart.h > 20 {
            let mut vals: Vec<i64> = Vec::new();
            if curve.len() < chipstats::CURVE {
                vals.push(0);
            }
            vals.extend(curve.iter().copied());
            let fv: Vec<f64> = vals.iter().map(|v| *v as f64).collect();
            let fmt = |v: f64| self.group(v as i64);
            pgz_engine::charts::line(s, chart, &fv, Some(self.b.accent), &[], Some(&fmt), None);
            // Null-Linie
            let lo = *vals.iter().min().unwrap_or(&0);
            let hi = *vals.iter().max().unwrap_or(&0);
            if lo < 0 && 0 < hi {
                let zy = (chart.bottom() as f64 - (0 - lo) as f64 / (hi - lo) as f64 * chart.h as f64) as i32;
                draw::line(s, ui::mix(p.red, p.panel, 0.4), (chart.x, zy), (chart.right(), zy), 1);
            }
        } else if chart.h > 20 {
            let img = d::miner_llama(16.max((chart.h - 8).min((90.0 * k) as i32)), "stand", self.b.accent, false);
            s.blr(&img, img.get_rect().center_at(chart.center()));
        }
    }

    fn draw_fair(&mut self, s: &mut Surface, body: Rect) {
        let p = ui::pal();
        let k = self.k;
        let gap = 4.max((7.0 * k) as i32);
        let mut y = body.y;
        for line in Self::wrap(&t("chip.fair_info"), &self.f_micro, body.w).iter().take(3) {
            let img = self.f_micro.render(line, true, p.text_dim);
            s.bl(&img, (body.x, y));
            y += self.f_micro.get_height();
        }
        y += gap;
        // Naechster Pruefcode (+ laufende Runde)
        let mut rows: Vec<(String, String, Color)> = Vec::new();
        if let Some(r) = self.round.as_ref() {
            rows.push((t("lmi.fair_current"), r.h.clone(), p.text));
        }
        rows.push((t("chip.fair_next"), fairness::commit(&self.fair.server), self.b.accent));
        for (lab, hx, col) in &rows {
            let img = self.fit(lab, &[&self.f_tiny, &self.f_micro], body.w, p.text_dim);
            s.bl(&img, (body.x, y));
            y += img.h();
            y = self.draw_hex(s, hx, body.x, y, body.w, *col);
            y += 2;
        }
        // Client-Seed + Knopf
        let bh = 22.max((28.0 * k) as i32);
        let lab = format!("{}: {}   \u{00B7}   {} {}", t("chip.fair_client"), self.fair.client, t("chip.fair_nonce"), self.fair.nonce);
        let mut bw = self.f_btn.width(&t("chip.fair_new_client")) + (20.0 * k) as i32;
        bw = bw.min(py::fdiv32(body.w, 2));
        self.r_fair_new = Rect::new(body.right() - bw, y, bw, bh);
        let img = self.fit(&lab, &[&self.f_tiny, &self.f_micro], body.w - bw - gap, p.text);
        s.blr(&img, img.get_rect().midleft_at((body.x, y + bh / 2)));
        self.btn_s(s, self.r_fair_new, &t("chip.fair_new_client"), self.round.is_none(), self.r_fair_new.hit(self.mouse), false, false);
        y += bh + gap;
        draw::line(s, p.border, (body.x, y), (body.right(), y), 1);
        y += gap;
        // Liste links, Einzelheiten rechts
        let area = Rect::new(body.x, y, body.w, body.bottom() - y - self.f_micro.get_height() - 2);
        let note = self.fit(&t("chip.fair_note"), &[&self.f_micro], body.w, p.text_faint);
        s.bl(&note, (body.x, body.bottom() - note.h()));
        self.r_fair_rows = Vec::new();
        self.r_fair_verify = r0();
        if self.fairlog.is_empty() {
            let img = self.fit(&t("chip.fair_empty"), &[&self.f_small, &self.f_tiny], area.w, p.text_dim);
            s.blr(&img, img.get_rect().center_at(area.center()));
            return;
        }
        let lw = ((area.w as f64 * 0.40) as i32).max(py::fdiv32(area.w, 2).min((220.0 * k) as i32));
        let lst = Rect::new(area.x, area.y, lw, area.h);
        let det = Rect::new(lst.right() + gap, area.y, area.w - lw - gap, area.h);
        let rh = (self.f_tiny.get_height() + 6).max((24.0 * k) as i32);
        let nvis = 1.max(py::fdiv32(lst.h, rh)) as i64;
        let order: Vec<usize> = (0..self.fairlog.len()).rev().collect(); // neueste oben
        let n_order = order.len() as i64;
        self.fair_scroll = 0.max(self.fair_scroll.min(n_order - nvis));
        let start = (self.fair_scroll as usize).min(order.len());
        let end = (start + nvis as usize).min(order.len());
        let mut rows_out = Vec::new();
        for (row, &idx) in order[start..end].iter().enumerate() {
            let e = &self.fairlog[idx];
            let r = Rect::new(lst.x, lst.y + row as i32 * rh, lst.w, rh - 2);
            let sel = Some(idx) == self.fair_sel;
            let fill = if sel || r.hit(self.mouse) { p.btn_sel } else { ui::mix(p.panel, (0, 0, 0), 0.2) };
            draw::rect_r(s, fill, r, 0, 3.max(rh / 4));
            if sel {
                draw::rect_r(s, self.b.accent, r, 1, 3.max(rh / 4));
            }
            let mv = e.o.as_object().and_then(|o| o.get("m"));
            let res = match py_int(mv) {
                Some(m) => {
                    if m != 0 {
                        self.fmt_short(m)
                    } else {
                        "\u{00D7}0".to_string()
                    }
                }
                None => "?".to_string(),
            };
            let num = self.f_tiny.render(&format!("#{}", e.n), true, p.text_dim);
            s.blr(&num, num.get_rect().midleft_at((r.x + 5, r.centery())));
            let rimg = self.f_tiny.render(&res, true, if truthy(mv) { p.green } else { p.red });
            s.blr(&rimg, rimg.get_rect().midright_at((r.right() - 5, r.centery())));
            let hx0 = r.x + 5 + num.w().max(self.f_tiny.width("#0000")) + 6;
            let hx1 = r.right() - 5 - rimg.w().max(self.f_tiny.width("\u{00D7}00,00")) - 6;
            if hx1 - hx0 > 20 {
                let himg = self.fit(&fairness::short(&e.h, 10), &[&self.f_mono], hx1 - hx0, p.text_faint);
                s.blr(&himg, himg.get_rect().midleft_at((hx0, r.centery())));
            }
            if let Some(chk) = self.fair_checks.get(&e.n) {
                let ir = Rect::new(r.right() - rimg.w() - 10 - rh / 2, r.y + rh / 4, rh / 2, rh / 2);
                d::icon(s, if chk.0 { "check" } else { "cross" }, ir, if chk.0 { p.green } else { p.red });
            }
            rows_out.push((idx, r));
        }
        self.r_fair_rows = rows_out;
        if n_order > nvis {
            // Bildlauf-Leiste
            let sb = Rect::new(lst.right() - 3, lst.y, 3, lst.h);
            draw::rect(s, p.border, sb, 0);
            let hh = 8.max(py::fdiv(sb.h as i64 * nvis, n_order) as i32);
            let hy = sb.y + py::fdiv((sb.h - hh) as i64 * self.fair_scroll, 1.max(n_order - nvis)) as i32;
            draw::rect(s, self.b.accent, (sb.x, hy, 3, hh), 0);
        }
        let Some(sel) = self.fair_sel else {
            return;
        };
        if sel >= self.fairlog.len() {
            return;
        }
        let e = self.fairlog[sel].clone();
        self.draw_fair_detail(s, det, &e);
    }

    /// Hex-Text in Monospace, bei Bedarf auf mehrere Zeilen umbrochen.
    fn draw_hex(&self, s: &mut Surface, hx: &str, x: i32, mut y: i32, w: i32, col: Color) -> i32 {
        let cw = 1.max(self.f_mono.width("0"));
        let per = 8.max(py::fdiv32(w, cw)) as usize;
        let chars: Vec<char> = hx.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let part: String = chars[i..(i + per).min(chars.len())].iter().collect();
            let img = self.f_mono.render(&part, true, col);
            s.bl(&img, (x, y));
            y += self.f_mono.get_height();
            i += per;
        }
        y
    }

    fn draw_fair_detail(&mut self, s: &mut Surface, det: Rect, e: &fairness::LogEntry) {
        let p = ui::pal();
        let k = self.k;
        let (x, mut y, w) = (det.x, det.y, det.w);
        let o = e.o.as_object().cloned().unwrap_or_default();
        let mut head = format!("#{}", e.n);
        let og = py_int(o.get("grid")).filter(|g| ml::GRIDS.iter().any(|x| *x as i64 == *g));
        if let (Some(g), Some(m)) = (og, py_int(o.get("mines"))) {
            head += &format!("  \u{00B7}  {}", tf("lmi.fair_round", &[("g", &g.to_string()), ("m", &m.to_string())]));
        }
        let img = self.fit(&head, &[&self.f_btn, &self.f_tiny], w, p.text);
        s.bl(&img, (x, y));
        y += img.h() + 2;
        for (lab, val) in [(t("chip.fair_hash"), &e.h), (t("chip.fair_server"), &e.s)] {
            let img = self.fit(&lab, &[&self.f_micro], w, p.text_dim);
            s.bl(&img, (x, y));
            y += img.h();
            y = self.draw_hex(s, val, x, y, w, p.text);
        }
        let img = self.fit(&format!("{}: {}", t("chip.fair_client"), e.c), &[&self.f_tiny, &self.f_micro], w, p.text);
        s.bl(&img, (x, y + 1));
        y += img.h() + 4;
        let bh = 22.max((28.0 * k) as i32);
        let bw = w.min(self.f_btn.width(&t("chip.fair_verify")) + (24.0 * k) as i32);
        self.r_fair_verify = Rect::new(x, y, bw, bh);
        self.btn(s, self.r_fair_verify, &t("chip.fair_verify"), true, true, self.r_fair_verify.hit(self.mouse), false, false, None, false);
        if let Some((ok, field)) = self.fair_checks.get(&e.n).cloned() {
            let ir = Rect::new(self.r_fair_verify.right() + 6, y + bh / 6, bh * 2 / 3, bh * 2 / 3);
            d::icon(s, if ok { "check" } else { "cross" }, ir, if ok { p.green } else { p.red });
            let img = self.fit(
                &if ok { t("chip.fair_ok") } else { t("chip.fair_bad") },
                &[&self.f_btn, &self.f_tiny],
                10.max(det.right() - ir.right() - 6),
                if ok { p.green } else { p.red },
            );
            s.blr(&img, img.get_rect().midleft_at((ir.right() + 4, y + bh / 2)));
            y += bh + 6;
            if let Some(f) = field {
                self.draw_mini(s, Rect::new(x, y, w, det.bottom() - y), &o, &f);
            }
        }
    }

    /// Mini-Vorschau des nachgerechneten Feldes: aufgedeckte Plaetze hell, der
    /// Rest abgedunkelt, die getroffene Mine rot.
    fn draw_mini(&self, s: &mut Surface, area: Rect, o: &Map<String, Value>, field: &[i32]) {
        let Some(g) = py_int(o.get("grid")) else {
            return;
        };
        let g = g as i32;
        let side = area.w.min(area.h);
        if side < g * 6 {
            return;
        }
        let gap = 1.max(py::fdiv32(side, 80));
        let cs = py::fdiv32(side - gap * (g - 1), g);
        let opened: Vec<Value> = match o.get("open") {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let mut mines = vec![false; (g * g) as usize];
        for &v in field {
            mines[v as usize] = true;
        }
        let hit = if o.get("m").and_then(num) == Some(0.0) && !opened.is_empty() { opened.last().and_then(num) } else { None };
        for i in 0..g * g {
            let x = area.x + (i % g) * (cs + gap);
            let y = area.y + (i / g) * (cs + gap);
            let kind = if hit == Some(i as f64) {
                "boom"
            } else if mines[i as usize] {
                "mine"
            } else {
                "gem"
            };
            let not_open = !opened.iter().any(|v| num(v) == Some(i as f64));
            s.bl(&d::face(cs, kind, not_open), (x, y));
        }
    }
}

impl Game for LamaMinesGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    /// Esc schliesst ein offenes Fenster bzw. stoppt den Auto-Modus.
    fn wants_escape(&self) -> bool {
        self.window.is_some() || self.auto.is_some()
    }

    fn reset(&mut self) {
        self.b.game_over = false;
        lamabank::load();
        self.rng = py::Random::unseeded();
        self.grid = match self.opt_int("grid") {
            Some(g) if ml::GRIDS.iter().any(|x| *x as i64 == g) => g as i32,
            _ => ml::DEFAULT_GRID,
        };
        let mines = self.opt_int("mines").unwrap_or(ml::DEFAULT_MINES as i64);
        self.mines = ml::clamp_mines(self.grid, mines.clamp(-1_000_000, 1_000_000) as i32);
        self.bet = match self.opt_int("bet") {
            Some(b) if ml::BETS.contains(&b) => b,
            _ => ml::BETS[0],
        };
        self.turbo = matches!(self.b.opt_value(KEY, "turbo"), Some(Value::Bool(true)));
        self.auto_rounds = match self.opt_int("auto_rounds") {
            Some(v) if ROUND_OPTS.contains(&v) => v,
            _ => 10,
        };
        self.auto_win = match self.opt_int("auto_win") {
            Some(v) if LIMIT_OPTS.contains(&v) => v,
            _ => 0,
        };
        self.auto_loss = match self.opt_int("auto_loss") {
            Some(v) if LIMIT_OPTS.contains(&v) => v,
            _ => 0,
        };
        let extra = match lamabank::get_extra(KEY) {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        self.hist = ml::clean_history(extra.get("hist"));
        {
            let rng = &mut self.rng;
            self.fair = fairness::clean_state(extra.get("fair"), &mut |k| rng.getrandbits(k));
        }
        self.fairlog = fairness::clean_log(extra.get("fairlog"));
        self.stats = chipstats::clean(extra.get("stats"));
        self.marks = ml::clean_marks(extra.get("auto_marks"));
        self.carry = ml::clean_carry(extra.get("carry"));
        self.round = ml::clean_round(extra.get("round"));
        // Spielfeld (laufende bzw. letzte Runde)
        self.field = Vec::new();
        self.is_mine = Vec::new();
        self.cells = Vec::new();
        self.hit = None;
        self.state = State::Idle;
        self.result = None;
        self.res_t = 0.0;
        self.res_delay = 0.0;
        self.boom_t = None;
        self.win = 0;
        self.lost = 0;
        self.last_mult = 0;
        self.pending = 0;
        self.shown_chips = lamabank::balance() as f64;
        self.particles.clear();
        self.coins_fx.clear();
        self.popups.clear();
        self.flashes.clear();
        self.toast = None;
        self.banner = None;
        self.shake = 0.0;
        self.time = 0.0;
        self.mouse = (-1, -1);
        self.broke = false;
        self.window = None;
        self.auto = None;
        self.cursor = 0;
        self.kb = false;
        self.mood = (Mood::Idle, 0.0);
        self.sooty = false;
        self.cart_fill = 0;
        self.fair_sel = None;
        self.fair_scroll = 0;
        self.fair_checks.clear();
        self.r_fair_rows = Vec::new();
        self.r_fair_new = r0();
        self.r_fair_verify = r0();
        self.cash_info = CashInfo { clear: false, maxed: false, m100: 0, shown: true };
        self.tick_cd = 0.0;
        self.lang = i18n::get_language();
        self.layout();
        if let Some(r) = self.round.clone() {
            // Unterbrochene Runde genau dort fortsetzen (Feld aus dem Seed)
            self.grid = r.grid;
            self.mines = r.mines;
            self.new_board(&r);
            for (j, &i) in r.open.iter().enumerate() {
                let c = &mut self.cells[i as usize];
                c.face = Face::Gem;
                c.flip = 1.0;
                c.pop = POP_T;
                c.k = j as i32 + 1;
            }
            self.cart_fill = self.cart_level();
            self.state = State::Play;
            self.layout();
            self.say(t("lmi.resumed"), 1.8);
        } else {
            self.blank_board();
            self.clamp_bet();
        }
        self.cursor = (self.grid * self.grid) / 2;
        self.b.score = lamabank::score_for(KEY);
        self.check_broke();
    }

    fn on_surface_changed(&mut self) {
        d::clear_cache();
        cd::clear_cache();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        if matches!(kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            if kind == Kind::MouseMove && self.board_r.hit(ev.pos) {
                self.kb = false;
            }
            self.mouse = ev.pos;
        }
        if self.broke {
            if kind == Kind::MouseDown || (kind == Kind::KeyDown && Self::is_action(&ev.key)) {
                self.take_credit();
            }
            return;
        }
        if kind == Kind::KeyDown {
            let key = ev.key.clone();
            self.on_key(&key, ev.repeat);
        } else if kind == Kind::Wheel {
            if self.window == Some(Win::Fair) {
                self.fair_scroll = 0.max(self.fair_scroll - if ev.delta > 0 { 1 } else { -1 });
            } else if self.window.is_none() && self.state == State::Idle {
                self.set_bet(if ev.delta > 0 { 1 } else { -1 });
            }
        } else if kind == Kind::MouseDown && ev.button == 1 {
            self.on_click(ev.pos);
        }
    }

    fn update(&mut self, dt: f64) {
        self.time += dt;
        if self.lang != i18n::get_language() {
            self.lang = i18n::get_language();
            self.layout();
        }
        let target = 0.max(lamabank::balance() - self.pending) as f64;
        let diff = target - self.shown_chips;
        if diff.abs() < 0.5 {
            self.shown_chips = target;
        } else {
            let step = (diff.abs() * 1f64.min(dt * 4.5)).max(60.0 * dt);
            self.shown_chips += diff.abs().min(step).copysign(diff);
            if diff > 0.0 {
                self.tick_cd -= dt;
                if self.tick_cd <= 0.0 {
                    self.tick_cd = 0.07;
                    let freq = 1500.0 + self.rng.random() * 300.0;
                    audio::tone(freq, 0.025, Some(&self.b.settings), "square", 0.08);
                }
            }
        }
        if let Some(toast) = self.toast.as_mut() {
            toast.1 -= dt;
            if toast.1 <= 0.0 {
                self.toast = None;
            }
        }
        if let Some(b) = self.banner.as_mut() {
            b.1 -= dt;
            if b.1 <= 0.0 {
                self.banner = None;
            }
        }
        self.shake = 0f64.max(self.shake - dt);
        self.mood.1 += dt;
        self.update_cells(dt);
        self.update_fx(dt);
        if self.state == State::Boom {
            match self.boom_t {
                None => {
                    if self.hit.is_some_and(|h| self.cells[h].flip >= 1.0) {
                        self.explode();
                    }
                }
                Some(bt) => self.boom_t = Some(bt + dt),
            }
        }
        if matches!(self.state, State::Boom | State::Cashed) {
            if self.res_delay > 0.0 {
                self.res_delay -= dt;
                if self.res_delay <= 0.0 && self.state == State::Cashed {
                    self.celebrate();
                }
            } else {
                self.res_t += dt;
                let limit = if self.auto.is_some() { AUTO_RESULT_T * self.tf() } else { RESULT_T };
                if self.res_t >= limit {
                    self.go_idle();
                    let tf = self.tf();
                    if let Some(a) = self.auto.as_mut() {
                        a.t = AUTO_PAUSE_T * tf;
                    }
                }
            }
        }
        if self.auto.is_some() && matches!(self.state, State::Idle | State::Play) {
            self.auto_tick(dt);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        s.bl(&d::background(w, h), (0, 0));
        self.draw_board(&mut s);
        self.draw_panel(&mut s);
        self.draw_hud(&mut s);
        self.draw_tools(&mut s);
        self.draw_history(&mut s);
        self.draw_strip(&mut s);
        self.draw_fx(&mut s);
        if self.result.is_some() && self.result_visible() {
            self.draw_result(&mut s);
        }
        if self.banner.is_some() {
            self.draw_banner(&mut s);
        }
        if matches!(self.window, Some(Win::Stats) | Some(Win::Fair)) {
            self.draw_window(&mut s);
        }
        if self.toast.is_some() {
            self.draw_toast(&mut s);
        }
        self.draw_tooltip(&mut s);
        if self.broke {
            self.draw_broke(&mut s);
        }
    }
}

/// Eintrag fuer `all_games()`.
pub const INFO: GameInfo = GameInfo {
    key: "lamamines",
    class_name: "LamaMinesGame",
    supports_multiplayer: false,
    modes: &[],
    make: LamaMinesGame::new,
};
