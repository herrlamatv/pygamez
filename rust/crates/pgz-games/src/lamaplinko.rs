//! lamaplinko.rs
//! =============
//! Lama Plinko - Kugel-Spiel mit Lama-Chips, uebertragen aus
//! `games/lamaplinko.py`. Das Lama oben am Brett laesst goldene Wollknaeuel
//! durch 8 bis 16 Reihen Stifte fallen; an jedem Stift springt die Kugel
//! nach links oder rechts (je 50 %), das Fach unten bestimmt den
//! Multiplikator. Regeln: [`crate::lamaplinko_logic`], Zeichnen:
//! [`crate::lamaplinko_draw`], Fairness/Statistik: `pgz_core::fairness`,
//! `pgz_core::chipstats`.
//!
//! * 8-16 Reihen (Pfeile links/rechts) und Risiko Niedrig/Mittel/Hoch (1-3),
//!   beides gesperrt, solange Kugeln fliegen.
//! * Einsatz je Kugel 1 ... 1000; Leertaste/Klick = werfen, gehalten =
//!   Dauerfeuer (Takt = Flugzeit / 30), hoechstens 50 Kugeln; Turbo (T).
//! * Jede Kugel ist eine eigene Fairness-Runde (Fenster F, Nachrechnen mit
//!   Mini-Vorschau), Statistik (S) mit Treffer-Verteilung, Auto-Modus (A).
//!
//! Buchung wie in Python: der Weg kommt beim Abwurf aus dem Fairness-Zufall
//! und wird sofort mit `lamabank::bet_and_pay` gebucht (ein Speichern samt
//! Verlauf, Log, Statistik, Uebertrag); gezeigt wird erst bei der Landung.
//! Erfolge fallen bei der Landung, `on_exit` holt sie fuer fliegende Kugeln
//! nach. Alle Tasten roh (ohne p1/p2-Belegung).
//!
//! Abweichung (bekannt, wie bei allen Spielen): Rust prueft settings.json
//! nicht gegen `settings.SCHEMAS`; das Spiel bildet die Regeln fuer seine
//! eigenen Optionen aber nach (gleicher Wert im Spiel wie Python).

use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_core::chipstats::{self, ChipStats};
use pgz_core::fairness::{self, LogEntry};
use pgz_core::{i18n, lamabank};
use pgz_engine::charts;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::casino_draw as cd;
use crate::lamaplinko_draw as d;
use crate::lamaplinko_logic as l;
use d::{ri, Rgb};

const KEY: &str = l::KEY;

/// vom Maul des Lamas bis zum ersten Stift
const DROP_T: f64 = 0.30;
/// Sprunghoehe ueber dem Stift (Anteil am Reihenabstand)
const BOUNCE: f64 = 0.24;
/// Leuchtdauer eines getroffenen Stifts
const FLASH_T: f64 = 0.32;
/// Fach wippt nach einem Treffer
const BUMP_T: f64 = 0.32;
/// Lama nickt beim Abwurf
const NOD_T: f64 = 0.30;
/// gehaltene Taste: erst danach Dauerfeuer
const HOLD_DELAY: f64 = 0.32;
/// Dauerfeuer: so viele Kugeln sind dann gleichzeitig unterwegs
const HOLD_BALLS: f64 = 30.0;
/// Abstand im Auto-Modus (Turbo: halb so lang)
const AUTO_RATE: f64 = 0.24;
/// Spurpunkte je Kugel
const TRAIL: i32 = 6;
/// Flugzeit zwischen zwei Spurpunkten
const TRAIL_DT: f64 = 0.024;
/// kleinster Abstand zweier Stift-Klicks (Ton)
const TICK_GAP: f64 = 0.045;
/// ab x10: Muenzregen
const BIG_MULT: i64 = 1000;
/// ab x100: grosser Gewinn (Banner, Konfetti)
const HUGE_MULT: i64 = 10000;
const AUTO_BALLS: [i64; 7] = [10, 25, 50, 100, 250, 500, 1000];
const AUTO_LIMITS: [i64; 7] = [0, 100, 500, 1000, 5000, 10000, 50000];
const DOT: &str = "   \u{00B7}   ";

/// Pythons `max(a, b)` fuer Kommazahlen (bei Gleichstand das erste).
fn pmax(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// Pythons `min(a, b)` fuer Kommazahlen (bei Gleichstand das erste).
fn pmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// `seg_time(rows)` - Flugzeit von Stiftreihe zu Stiftreihe.
fn seg_time(rows: i32) -> f64 {
    0.165 - (rows - 8) as f64 * 0.005
}

/// `flight_time(rows)` - gesamte Flugzeit einer Kugel (ohne Turbo).
fn flight_time(rows: i32) -> f64 {
    DROP_T + rows as f64 * seg_time(rows)
}

/// `fire_rate(rows)` - Abstand zweier Kugeln im Dauerfeuer (ohne Turbo).
fn fire_rate(rows: i32) -> f64 {
    flight_time(rows) / HOLD_BALLS
}

/// Ganze Zahl wie Python sie vergleicht (`int`, auch `bool` als 0/1).
fn int_like(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Bool(b)) => Some(*b as i64),
        Some(Value::Number(n)) if !n.is_f64() => n.as_i64(),
        _ => None,
    }
}

/// Pythons `==` fuer JSON-Werte (Zahlen und bool numerisch, dicts ohne
/// Reihenfolge).
fn py_eq(a: &Value, b: &Value) -> bool {
    let num = |v: &Value| -> Option<f64> {
        match v {
            Value::Bool(x) => Some(*x as i64 as f64),
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    };
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| py_eq(p, q)),
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| py_eq(v, w)))
        }
        (Value::Number(x), Value::Number(y)) if !x.is_f64() && !y.is_f64() => x.as_i64() == y.as_i64(),
        _ => match (num(a), num(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        },
    }
}

/// `str(v)` wie Python (fuer Platzhalter in `t(...)`).
fn py_str(v: Option<&Value>) -> String {
    fn repr(v: &Value) -> String {
        match v {
            Value::String(s) => format!("'{}'", s),
            other => py_str(Some(other)),
        }
    }
    match v {
        None | Some(Value::Null) => "None".into(),
        Some(Value::Bool(true)) => "True".into(),
        Some(Value::Bool(false)) => "False".into(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => format!("[{}]", a.iter().map(repr).collect::<Vec<_>>().join(", ")),
        Some(Value::Object(o)) => {
            format!("{{{}}}", o.iter().map(|(k, v)| format!("'{}': {}", k, repr(v))).collect::<Vec<_>>().join(", "))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Auto,
    Stats,
    Fair,
}

/// Klickflaechen der Fenster (`lambda` in Python).
#[derive(Clone, Copy)]
enum Act {
    AutoSet(usize, i64),
    AutoTurbo,
    AutoStart,
    StatsMinus,
    StatsPlus,
    NewClient,
    FairSel(i32),
    Verify,
}

/// Eine fliegende Kugel (`dict(rows, risk, bet, win, m, slot, path, pre,
/// jit, hb, spin, t, hit, log, msg)`).
struct Ball {
    rows: i32,
    risk: &'static str,
    bet: i64,
    win: i64,
    m: i64,
    slot: i32,
    path: Vec<i32>,
    pre: Vec<i32>,
    jit: Vec<f64>,
    hb: Vec<f64>,
    spin: i64,
    t: f64,
    hit: i32,
    log: LogEntry,
    msg: Option<&'static str>,
}

struct Auto {
    left: i64,
    net: i64,
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    r: f64,
    col: Rgb,
    t: f64,
    life: f64,
    g: f64,
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
    x: f64,
    y: f64,
    text: String,
    color: Color,
    t: f64,
    dur: f64,
    big: bool,
}

struct Banner {
    title: String,
    sub: String,
    t: f64,
    dur: f64,
}

/// Geometrie des Fairness-Fensters (`_fair_geom`).
struct FairGeom {
    body: Rect,
    lh: i32,
    y: i32,
    r_new: Rect,
    hash_y: i32,
    ly: i32,
    lw: i32,
    step: i32,
    vis: i32,
    detail: Rect,
    preview: Rect,
    r_ver: Rect,
    bottom: i32,
}

/// `_fair_rects()`: Geometrie, sichtbare Zeilen `(rect, index)`, Klickflaechen.
type FairRects = (FairGeom, Vec<(Rect, i32)>, Vec<(Rect, Act)>);

/// Geometrie des Auto-Fensters (`_auto_rects`).
struct AutoGeom {
    rows: Vec<(i32, Vec<(Rect, i64)>)>,
    turbo: Rect,
    sum_y: i32,
    start: Rect,
    hits: Vec<(Rect, Act)>,
}

/// `LamaPlinkoGame`
pub struct LamaPlinkoGame {
    b: GameBase,
    rng: py::Random,
    rows: i32,
    risk: &'static str,
    bet: i64,
    turbo: bool,
    auto_balls: i64,
    auto_win: i64,
    auto_loss: i64,
    hist: Vec<i64>,
    fair: fairness::State,
    fairlog: Vec<LogEntry>,
    stats: ChipStats,
    hits: l::Hits,
    carry: i64,
    hist_shown: Vec<i64>,
    fairlog_shown: Vec<LogEntry>,
    stats_shown: ChipStats,
    hits_shown: l::Hits,
    balls: Vec<Ball>,
    pending: i64,
    shown_chips: f64,
    rec_shown: i64,
    shown_ledger: i64,
    sess_balls: i64,
    sess_net: i64,
    sess_best: i64,
    auto: Option<Auto>,
    panel: Option<Panel>,
    auto_focus: i32,
    stats_rows: i32,
    stats_hover: Option<usize>,
    fair_sel: i32,
    fair_scroll: i32,
    fair_checks: HashMap<i64, (bool, bool, Vec<i32>)>,
    hold: bool,
    hold_t: f64,
    fire_cd: f64,
    auto_cd: f64,
    tick_cd: f64,
    nod: f64,
    particles: Vec<Particle>,
    coins_fx: Vec<CoinFx>,
    popups: Vec<Popup>,
    banner: Option<Banner>,
    toast: Option<(String, f64)>,
    shake: f64,
    hist_anim: f64,
    time: f64,
    mouse: (i32, i32),
    broke: bool,
    chip_tick_cd: f64,
    tick_n: i64,
    lang: String,
    peg_flash: Vec<Vec<f64>>,
    slot_bump: Vec<f64>,
    // ----- Layout
    k: f64,
    f_tiny: Font,
    f_small: Font,
    f_btn: Font,
    f_big: Font,
    f_huge: Font,
    margin: i32,
    hud_h: i32,
    strip_h: i32,
    strip: Rect,
    region: Rect,
    hud_bg: Surface,
    strip_bg: Surface,
    overlay: Surface,
    dim: Surface,
    left: Rect,
    right: Rect,
    r_risks: Vec<Rect>,
    lbl_rows_y: i32,
    r_rows_m: Rect,
    r_rows_p: Rect,
    r_rows_v: Rect,
    info_y: i32,
    pill_h: i32,
    r_minus: Rect,
    r_bet: Rect,
    r_plus: Rect,
    r_drop: Rect,
    r_icons: Vec<Rect>,
    cx: f64,
    llama_n: i32,
    foot_y: i32,
    top: i32,
    gap: f64,
    peg_r: i32,
    ball_r: i32,
    slot_w: i32,
    slot_h: i32,
    slot_top: i32,
    slot_font: i32,
    foot_x: f64,
    mouth_r: i32,
    mouth: (f64, f64),
    board_box: Rect,
    plate_pts: [(f64, f64); 4],
    pegs_local: Vec<(i32, i32)>,
    slot_rects: Vec<Rect>,
    pan_head: i32,
    pan: Rect,
    r_close: Rect,
}

impl LamaPlinkoGame {
    /// `LamaPlinkoGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let r0 = Rect::new(0, 0, 0, 0);
        let mut g = LamaPlinkoGame {
            b,
            rng: py::Random::new(0),
            rows: l::DEFAULT_ROWS,
            risk: l::DEFAULT_RISK,
            bet: 10,
            turbo: false,
            auto_balls: 25,
            auto_win: 0,
            auto_loss: 0,
            hist: Vec::new(),
            fair: fairness::State { server: String::new(), client: String::new(), nonce: 0 },
            fairlog: Vec::new(),
            stats: chipstats::blank(),
            hits: l::blank_hits(),
            carry: 0,
            hist_shown: Vec::new(),
            fairlog_shown: Vec::new(),
            stats_shown: chipstats::blank(),
            hits_shown: l::blank_hits(),
            balls: Vec::new(),
            pending: 0,
            shown_chips: 0.0,
            rec_shown: 0,
            shown_ledger: 0,
            sess_balls: 0,
            sess_net: 0,
            sess_best: 0,
            auto: None,
            panel: None,
            auto_focus: 0,
            stats_rows: l::DEFAULT_ROWS,
            stats_hover: None,
            fair_sel: 0,
            fair_scroll: 0,
            fair_checks: HashMap::new(),
            hold: false,
            hold_t: 0.0,
            fire_cd: 0.0,
            auto_cd: 0.0,
            tick_cd: 0.0,
            nod: 0.0,
            particles: Vec::new(),
            coins_fx: Vec::new(),
            popups: Vec::new(),
            banner: None,
            toast: None,
            shake: 0.0,
            hist_anim: 1.0,
            time: 0.0,
            mouse: (-1, -1),
            broke: false,
            chip_tick_cd: 0.0,
            tick_n: 0,
            lang: String::new(),
            peg_flash: Vec::new(),
            slot_bump: Vec::new(),
            k: 1.0,
            f_tiny: f.clone(),
            f_small: f.clone(),
            f_btn: f.clone(),
            f_big: f.clone(),
            f_huge: f,
            margin: 0,
            hud_h: 0,
            strip_h: 0,
            strip: r0,
            region: r0,
            hud_bg: pg::surface_alpha(1, 1),
            strip_bg: pg::surface_alpha(1, 1),
            overlay: pg::surface_alpha(1, 1),
            dim: pg::surface_alpha(1, 1),
            left: r0,
            right: r0,
            r_risks: Vec::new(),
            lbl_rows_y: 0,
            r_rows_m: r0,
            r_rows_p: r0,
            r_rows_v: r0,
            info_y: 0,
            pill_h: 0,
            r_minus: r0,
            r_bet: r0,
            r_plus: r0,
            r_drop: r0,
            r_icons: Vec::new(),
            cx: 0.0,
            llama_n: 0,
            foot_y: 0,
            top: 0,
            gap: 8.0,
            peg_r: 2,
            ball_r: 3,
            slot_w: 6,
            slot_h: 10,
            slot_top: 0,
            slot_font: 8,
            foot_x: 0.0,
            mouth_r: 3,
            mouth: (0.0, 0.0),
            board_box: r0,
            plate_pts: [(0.0, 0.0); 4],
            pegs_local: Vec::new(),
            slot_rects: Vec::new(),
            pan_head: 30,
            pan: r0,
            r_close: r0,
        };
        g.reset();
        Box::new(g)
    }

    // ===================================================== Einstellungen

    /// Ganzzahl-Option wie `settings.SCHEMAS` ("choice": genau ein int aus
    /// der Liste, sonst die Vorgabe aus `settings.DEFAULTS`).
    fn opt_choice(&self, key: &str, choices: &[i64], default: i64) -> i64 {
        match self.b.opt_value(KEY, key) {
            Some(Value::Number(n)) if !n.is_f64() => match n.as_i64() {
                Some(v) if choices.contains(&v) => v,
                _ => default,
            },
            _ => default,
        }
    }

    /// `_set_opt(key, value)` - nur bei Aenderung speichern.
    fn set_opt(&mut self, key: &str, value: Value) {
        let alt = self.b.opt_value(KEY, key);
        if !alt.as_ref().is_some_and(|a| py_eq(a, &value)) {
            self.b.set_opt(KEY, key, value);
        }
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.k = pmin(w as f64 / 800.0, h as f64 / 600.0);
        let k = self.k;
        self.f_tiny = ui::font(11.max((13.0 * k) as i32), false, false);
        self.f_small = ui::font(12.max((15.0 * k) as i32), false, false);
        self.f_btn = ui::font(12.max((15.0 * k) as i32), true, false);
        self.f_big = ui::font(15.max((22.0 * k) as i32), true, false);
        self.f_huge = ui::font(24.max((46.0 * k) as i32), true, false);
        let _f_mono = ui::font(10.max((13.0 * k) as i32), false, true);
        self.margin = 6.max((12.0 * k) as i32);
        let m = self.margin;
        self.hud_h = 28.max((42.0 * k) as i32);
        self.strip_h = 46.max((64.0 * k) as i32);
        self.strip = Rect::new(0, h - self.strip_h, w, self.strip_h);
        self.region = Rect::new(0, self.hud_h, w, self.strip.y - self.hud_h);
        self.hud_bg = pg::surface_alpha(w, self.hud_h);
        for y in 0..self.hud_h {
            let a = (210.0 - (60 * y) as f64 / 1.max(self.hud_h - 1) as f64) as i32;
            self.hud_bg.fill_r(rgba(10, 8, 16, a), (0, y, w, 1));
        }
        self.strip_bg = pg::surface_alpha(self.strip.w, self.strip.h);
        for y in 0..self.strip_h {
            let a = (170.0 + (60 * y) as f64 / 1.max(self.strip_h - 1) as f64) as i32;
            self.strip_bg.fill_r(rgba(10, 8, 16, a), (0, y, w, 1));
        }
        self.overlay = pg::surface_alpha(w, h);
        self.overlay.fill_c(rgba(6, 4, 12, 200));
        self.dim = pg::surface_alpha(w, h);
        self.dim.fill_c(rgba(6, 4, 12, 150));
        // --- Seitenleisten: links Risiko + Reihen, rechts Verlauf
        let side = 92.max((150.0 * k) as i32);
        let top = self.region.y + m;
        self.left = Rect::new(m, top, side, self.region.h - 2 * m);
        self.right = Rect::new(w - m - side, top, side, self.region.h - 2 * m);
        let pad = 5.max((9.0 * k) as i32);
        let bh = 22.max((30.0 * k) as i32);
        let x = self.left.x + pad;
        let bw = self.left.w - 2 * pad;
        let mut y = self.left.y + pad + self.f_tiny.get_height() + 3;
        self.r_risks.clear();
        for _ in 0..3 {
            self.r_risks.push(Rect::new(x, y, bw, bh));
            y += bh + 4;
        }
        self.lbl_rows_y = y + pad / 2;
        let y = self.lbl_rows_y + self.f_tiny.get_height() + 3;
        self.r_rows_m = Rect::new(x, y, bh, bh);
        self.r_rows_p = Rect::new(x + bw - bh, y, bh, bh);
        self.r_rows_v = Rect::new(self.r_rows_m.right() + 3, y, self.r_rows_p.x - self.r_rows_m.right() - 6, bh);
        self.info_y = y + bh + pad;
        self.pill_h = 16.max((22.0 * k) as i32);
        // --- Bedienleiste: Einsatz links, Fenster-Knoepfe Mitte, WERFEN rechts
        let sh = self.strip_h;
        let bh = 30.max((sh as f64 * 0.62) as i32);
        let by = self.strip.y + py::fdiv32(sh - bh, 2);
        let lbl_w = self.f_tiny.width(&t("cas.bet")).max(self.f_btn.width("1000")) + (14.0 * k) as i32;
        self.r_minus = Rect::new(m, by, bh, bh);
        self.r_bet = Rect::new(self.r_minus.right() + 4, by, lbl_w, bh);
        self.r_plus = Rect::new(self.r_bet.right() + 4, by, bh, bh);
        let pad = (22.0 * pmax(0.7, k)) as i32;
        let mut drop_w = self.f_btn.width(&t("lpl.drop")).max(self.f_btn.width(&t("chip.auto_stop"))) + pad;
        drop_w = drop_w.max((130.0 * k) as i32);
        self.r_drop = Rect::new(w - m - drop_w, by, drop_w, bh);
        let gap = 4.max((7.0 * k) as i32);
        let n_icons = 4;
        let room = self.r_drop.x - self.r_plus.right() - 2 * m;
        let ib = bh.min(py::fdiv32(room - (n_icons - 1) * gap, n_icons));
        let x0 = self.r_plus.right() + m + py::fdiv32(room - (n_icons * ib + (n_icons - 1) * gap), 2);
        let iy = by + py::fdiv32(bh - ib, 2);
        self.r_icons = (0..n_icons).map(|i| Rect::new(x0 + i * (ib + gap), iy, ib, ib)).collect();
        self.board_layout();
        self.panel_layout();
    }

    /// Brett-Geometrie fuer die aktuelle Reihenzahl (Stifte, Faecher, Lama).
    fn board_layout(&mut self) {
        let (k, m) = (self.k, self.margin);
        let n = self.rows;
        let nf = n as f64;
        let bx0 = self.left.right() + m;
        let bx1 = self.right.x - m;
        self.cx = (bx0 + bx1) as f64 / 2.0;
        self.llama_n = 32.max((62.0 * k) as i32);
        self.foot_y = self.region.y + (self.llama_n as f64 * 0.95) as i32 + 2;
        self.top = self.foot_y + 8.max((14.0 * k) as i32);
        let avail_h = self.region.bottom() - m - 3 - self.top;
        let gap = pmin((bx1 - bx0 - 10) as f64 / (nf + 2.6), avail_h as f64 / (nf + 0.36));
        let gap = pmax(8.0, gap);
        self.gap = gap;
        self.peg_r = 2.max((gap * 0.12) as i32);
        self.ball_r = 3.max((gap * 0.28) as i32);
        self.slot_w = 6.max((gap - pmax(2.0, gap * 0.1)) as i32);
        self.slot_h = 10.max((gap * 0.78) as i32);
        self.slot_top = (self.top as f64 + (n - 1) as f64 * gap + gap * 0.55) as i32;
        self.slot_font = 8.max((gap * 0.44) as i32);
        // Lama: Maul knapp links ueber dem Einwurf
        let lnf = self.llama_n as f64;
        self.foot_x = self.cx - d::LLAMA_MOUTH.0 * lnf + d::LLAMA_PIVOT.0 * lnf - gap * 0.35;
        let (mx, my) = d::mouth_pos(self.llama_n, 0, self.foot_x, self.foot_y as f64);
        // kleines Knaeuel im Maul; beim Abwurf waechst es auf volle Groesse
        self.mouth_r = 3.max(self.ball_r.min((lnf * 0.085) as i32));
        self.mouth = (mx + self.mouth_r as f64 * 0.55, my + self.mouth_r as f64 * 0.2);
        // Brett-Umriss (Trapez) + Stifte relativ zur Brett-Box
        let half_top = gap * 1.6;
        let half_bot = (n + 2) as f64 / 2.0 * gap + gap * 0.25;
        let y0 = self.top as f64 - gap * 0.55;
        let y1 = self.slot_top + self.slot_h + 6;
        self.board_box = Rect::new(
            (self.cx - half_bot) as i32 - 4,
            y0 as i32 - 4,
            (2.0 * half_bot) as i32 + 8,
            (y1 as f64 - y0) as i32 + 8,
        );
        let bb = self.board_box;
        let (bx, by) = (bb.x as f64, bb.y as f64);
        self.plate_pts = [
            (self.cx - half_top - bx, y0 - by),
            (self.cx + half_top - bx, y0 - by),
            (self.cx + half_bot - bx, y1 as f64 - by),
            (self.cx - half_bot - bx, y1 as f64 - by),
        ];
        let mut pegs = Vec::new();
        for r in 0..n {
            for j in 0..r + 3 {
                let (x, y) = self.peg_xy(r, j);
                pegs.push((ri(x) - bb.x, ri(y) - bb.y));
            }
        }
        self.pegs_local = pegs;
        self.slot_rects = (0..=n).map(|i| self.slot_rect(i)).collect();
    }

    /// Fenster (Auto, Statistik, Fairness): mittig.
    fn panel_layout(&mut self) {
        let (w, h, k, m) = (self.b.width, self.b.height, self.k, self.margin);
        self.pan_head = 30.max((42.0 * k) as i32);
        let pw = (w - 2 * m).min(((660.0 * k) as i32).max(456));
        let mut ph = (h - 2 * m).min(((480.0 * k) as i32).max(340));
        if self.panel == Some(Panel::Auto) {
            ph = ph.min(self.pan_head + m / 2 + self.auto_content_h() + m);
        }
        self.pan = Rect::new(py::fdiv32(w - pw, 2), py::fdiv32(h - ph, 2), pw, ph);
        let cb = self.pan_head - 10;
        self.r_close = Rect::new(self.pan.right() - 6 - cb, self.pan.y + 5, cb, cb);
    }

    fn new_board_fx(&mut self) {
        self.peg_flash = (0..self.rows).map(|r| vec![0.0; (r + 3) as usize]).collect();
        self.slot_bump = vec![0.0; (self.rows + 1) as usize];
    }

    // ===================================================== Geometrie

    /// Mittelpunkt von Stift j in Reihe r (Reihe r hat r+3 Stifte).
    fn peg_xy(&self, r: i32, j: i32) -> (f64, f64) {
        (self.cx + (j as f64 - (r + 2) as f64 / 2.0) * self.gap, self.top as f64 + r as f64 * self.gap)
    }

    fn slot_x(&self, i: i32) -> f64 {
        self.cx + (i as f64 - self.rows as f64 / 2.0) * self.gap
    }

    fn slot_rect(&self, i: i32) -> Rect {
        Rect::new(0, 0, self.slot_w, self.slot_h).midtop_at((ri(self.slot_x(i)), self.slot_top))
    }

    /// Aufprallpunkt der Kugel ueber dem Stift (r, k+1) - mit Zittern.
    fn contact(&self, r: i32, k: i32, jit: f64) -> (f64, f64) {
        let x = self.cx + (k as f64 - r as f64 / 2.0 + jit) * self.gap;
        let y = self.top as f64 + r as f64 * self.gap - (self.peg_r + self.ball_r) as f64 * 0.92;
        (x, y)
    }

    fn landing(&self, k: i32, jit: f64) -> (f64, f64) {
        (self.slot_x(k) + jit * self.gap * 0.6, self.slot_top as f64 + self.ball_r as f64 * 0.15)
    }

    /// Radius einer Kugel nach der Flugzeit t (waechst aus dem Maul heraus).
    fn ball_radius(&self, t: f64) -> i32 {
        let f = t / (DROP_T * 0.7);
        if f >= 1.0 {
            return self.ball_r;
        }
        self.mouth_r + ((self.ball_r - self.mouth_r) as f64 * pmax(0.0, f)) as i32
    }

    /// Position einer Kugel nach der Flugzeit t (reine Funktion von t).
    fn ball_pos(&self, b: &Ball, t: f64) -> (f64, f64) {
        let n = b.rows;
        if t <= 0.0 {
            return self.mouth;
        }
        if t < DROP_T {
            let p = t / DROP_T;
            let (x1, y1) = self.contact(0, 0, b.jit[0]);
            let (x0, y0) = self.mouth;
            return (x0 + (x1 - x0) * p, y0 + (y1 - y0) * p * p);
        }
        let st = seg_time(n);
        let u = (t - DROP_T) / st;
        let seg = (n - 1).min(u as i32);
        let p = pmin(1.0, u - seg as f64);
        let su = seg as usize;
        let ka = b.pre[su];
        let (xa, ya) = self.contact(seg, ka, b.jit[su]);
        let kb = ka + b.path[su];
        let (xb, yb) = if seg < n - 1 { self.contact(seg + 1, kb, b.jit[su + 1]) } else { self.landing(kb, b.jit[n as usize]) };
        let hgt = BOUNCE * self.gap * b.hb[su];
        let dy = yb - ya;
        let a = 2.0 * hgt + 2.0 * (hgt * hgt + hgt * dy).sqrt();
        let bq = dy + a;
        (xa + (xb - xa) * p, ya - a * p + bq * p * p)
    }

    // ===================================================== Konto

    /// `stats` mit den Trefferzaehlern unter `x.hits`.
    fn stats_value(&self) -> Value {
        let mut v = self.stats.to_value();
        v["x"]["hits"] = l::hits_to_value(&self.hits);
        v
    }

    /// `_extra()`
    fn extra_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("hist".into(), Value::from(self.hist.clone()));
        m.insert("fair".into(), self.fair.to_value());
        m.insert("fairlog".into(), fairness::log_to_value(&self.fairlog));
        m.insert("stats".into(), self.stats_value());
        m.insert("carry".into(), Value::from(self.carry));
        Value::Object(m)
    }

    /// Angezeigtes Guthaben (ohne die Gewinne noch fliegender Kugeln).
    fn avail(&self) -> i64 {
        0.max(lamabank::balance() - self.pending)
    }

    fn clamp_bet(&mut self) {
        let chips = self.avail();
        if self.bet > chips {
            let ok: Vec<i64> = l::BETS.iter().copied().filter(|b| *b <= chips).collect();
            self.bet = ok.last().copied().unwrap_or(l::BETS[0]);
        }
    }

    fn check_broke(&mut self) {
        let was = self.broke;
        self.broke = self.balls.is_empty() && lamabank::is_broke(KEY, None);
        if self.broke && !was {
            self.b.play_sound("gameover");
            self.panel = None;
            self.auto = None;
        }
    }

    fn take_credit(&mut self) {
        if lamabank::refill_if_broke(KEY, None) {
            self.b.play_sound("powerup");
            ui::spawn_burst((self.b.width / 2) as f64, (self.b.height / 2) as f64, Some(ui::pal().gold), 26, 260.0);
        }
        self.broke = false;
        self.shown_chips = lamabank::balance() as f64;
        self.clamp_bet();
    }

    fn say(&mut self, text: String, secs: f64) {
        self.toast = Some((text, secs));
    }

    fn sep(&self) -> String {
        t("lpl.decimal")
    }

    fn fmt(&self, m100: i64) -> String {
        format!("\u{00D7}{}", l::fmt_short(m100, &self.sep()))
    }

    fn rtp_text(&self) -> String {
        // Fraction(expected_return): (num * 20000 + den) // (2 * den) - ungekuerzt gleich
        let num: i64 =
            l::table(self.rows, self.risk).iter().enumerate().map(|(i, m)| l::comb(self.rows as i64, i as i64) * m).sum();
        let den: i64 = (1i64 << self.rows) * 100;
        let v = (num * 20000 + den) / (2 * den);
        format!("{}{}{:02}", v / 100, self.sep(), v % 100)
    }

    fn risk_name(&self, risk: &str) -> String {
        t(&format!("lpl.risk.{risk}"))
    }

    // ===================================================== Abwurf

    fn speed(&self) -> f64 {
        if self.turbo {
            2.0
        } else {
            1.0
        }
    }

    /// `_drop(quiet)` - eine Kugel werfen: Ergebnis ziehen, buchen, Flug starten.
    fn drop_ball(&mut self, quiet: bool) -> bool {
        if self.broke {
            return false;
        }
        if self.balls.len() >= l::MAX_BALLS {
            if !quiet {
                self.say(tf("lpl.max_balls", &[("n", &l::MAX_BALLS.to_string())]), 1.2);
            }
            return false;
        }
        let bet = self.bet;
        if bet > self.avail() {
            if !quiet {
                self.say(t("cas.not_enough"), 1.8);
                self.b.play_sound("hit");
            }
            return false;
        }
        let (rows, risk) = (self.rows, self.risk);
        let entry = fairness::begin(&self.fair);
        {
            let rng = &mut self.rng;
            fairness::advance(&mut self.fair, &mut |k| rng.getrandbits(k));
        }
        let path = l::drop_path(&mut fairness::entry_rand(KEY, &entry), rows);
        let slot = l::slot_of(&path);
        let m100 = l::mult100(rows, risk, slot);
        // Rest-Hundertstel-Uebertrag: x0,4 bei Einsatz 1 zahlt sonst nie etwas
        let (win, carry) = l::payout_carry(bet, rows, risk, slot, self.carry);
        self.carry = carry;
        let outcome = json!({"rows": rows, "risk": risk, "slot": slot, "m": m100, "bet": bet, "win": win});
        fairness::record(&mut self.fairlog, &entry, &outcome);
        self.stats.record(bet, win, m100);
        l::add_hit(&mut self.hits, rows, slot);
        self.hist.push(m100);
        if self.hist.len() > l::HISTORY {
            let cut = self.hist.len() - l::HISTORY;
            self.hist.drain(..cut);
        }
        if !lamabank::bet_and_pay(bet, win, KEY, Some((KEY, self.extra_value()))) {
            return false;
        }
        self.b.score = lamabank::score_for(KEY);
        self.pending += win;
        let mut pre = vec![0];
        for dd in &path {
            pre.push(pre[pre.len() - 1] + dd);
        }
        // kosmetisches Zittern (nur Anzeige - das Ergebnis steht schon fest)
        let jit: Vec<f64> = (0..rows + 1).map(|_| self.rng.uniform(-0.06, 0.06)).collect();
        let hb: Vec<f64> = (0..rows).map(|_| self.rng.uniform(0.82, 1.18)).collect();
        let spin = self.rng.randrange0(12);
        self.balls.push(Ball {
            rows,
            risk,
            bet,
            win,
            m: m100,
            slot,
            path,
            pre,
            jit,
            hb,
            spin,
            t: 0.0,
            hit: 0,
            log: fairness::make_entry(&entry, &outcome),
            msg: None,
        });
        self.nod = NOD_T;
        self.b.play_sound("move");
        if let Some(a) = self.auto.as_mut() {
            a.net += win - bet;
        }
        if self.balls.len() >= l::RAIN_BALLS {
            self.b.ach_event("lpl_rain", None);
        }
        true
    }

    fn start_hold(&mut self) {
        if self.auto.is_some() {
            self.stop_auto(None);
            return;
        }
        self.hold = true;
        self.hold_t = 0.0;
        self.fire_cd = 0.0;
        self.drop_ball(false);
    }

    // ===================================================== Auto-Modus

    fn start_auto(&mut self) {
        if self.broke || self.auto.is_some() {
            return;
        }
        if self.bet > self.avail() {
            self.say(t("cas.not_enough"), 1.8);
            self.b.play_sound("hit");
            return;
        }
        self.auto = Some(Auto { left: self.auto_balls, net: 0 });
        self.auto_cd = 0.0;
        self.panel = None;
        self.hold = false;
        self.b.play_sound("select");
    }

    /// Auto beenden; msg_key wird gezeigt, sobald die letzte Kugel gelandet ist.
    fn stop_auto(&mut self, msg_key: Option<&'static str>) {
        if self.auto.is_none() {
            return;
        }
        self.auto = None;
        let Some(key) = msg_key else {
            self.b.play_sound("click");
            return;
        };
        if let Some(last) = self.balls.last_mut() {
            last.msg = Some(key);
        } else {
            self.say(t(key), 2.4);
        }
    }

    fn auto_step(&mut self) {
        let left = self.auto.as_ref().map(|a| a.left).unwrap_or(0);
        if left <= 0 {
            self.stop_auto(Some("chip.auto_done"));
            return;
        }
        if self.bet > self.avail() {
            if !self.balls.is_empty() {
                return; // erst landen lassen - vielleicht reicht es dann
            }
            self.stop_auto(Some("chip.auto_no_chips"));
            self.clamp_bet();
            return;
        }
        if self.balls.len() >= l::MAX_BALLS {
            return;
        }
        if !self.drop_ball(true) {
            return;
        }
        let (left, net) = match self.auto.as_mut() {
            Some(a) => {
                a.left -= 1;
                (a.left, a.net)
            }
            None => return,
        };
        if self.auto_win != 0 && net >= self.auto_win {
            self.stop_auto(Some("chip.auto_hit_win"));
        } else if self.auto_loss != 0 && -net >= self.auto_loss {
            self.stop_auto(Some("chip.auto_hit_loss"));
        } else if left <= 0 {
            self.stop_auto(Some("chip.auto_done"));
        }
    }

    // ===================================================== Landung

    fn land(&mut self, b: &Ball) {
        let (slot, m100, win) = (b.slot, b.m, b.win);
        self.pending = 0.max(self.pending - win);
        self.shown_ledger += win - b.bet;
        self.rec_shown = self.b.score.min(self.rec_shown.max(lamabank::START_CHIPS + self.shown_ledger));
        self.hist_shown.push(m100);
        if self.hist_shown.len() > l::HISTORY {
            let cut = self.hist_shown.len() - l::HISTORY;
            self.hist_shown.drain(..cut);
        }
        self.hist_anim = 0.0;
        self.sess_balls += 1;
        self.sess_net += win - b.bet;
        self.sess_best = self.sess_best.max(m100);
        self.stats_shown.record(b.bet, win, m100);
        l::add_hit(&mut self.hits_shown, b.rows, slot);
        self.fairlog_shown.push(b.log.clone());
        if self.fairlog_shown.len() > fairness::LOG {
            let cut = self.fairlog_shown.len() - fairness::LOG;
            self.fairlog_shown.drain(..cut);
        }
        if 0 <= slot && (slot as usize) < self.slot_bump.len() {
            self.slot_bump[slot as usize] = BUMP_T;
        }
        let r = if (slot as usize) < self.slot_rects.len() { self.slot_rects[slot as usize] } else { self.slot_rect(slot) };
        let col = d::slot_color(m100);
        self.spawn(r.centerx() as f64, r.y as f64, (6 + 14.min(m100 / 200)) as usize, col, 160.0, true, 2.6, 0.5, 520.0);
        let big = m100 >= BIG_MULT;
        let pcol = if m100 >= 100 { col.c() } else { ui::pal().text_dim };
        let txt = self.fmt(m100);
        self.popup(r.centerx() as f64, r.y as f64 - 6.0 * self.k, txt, pcol, 1.0 + 0.4 * big as i32 as f64, big);
        self.land_sound(m100);
        if m100 >= HUGE_MULT {
            let title = if b.rows == 16 && b.risk == "high" && l::is_edge(16, slot) {
                t("cas.s.jackpot")
            } else if m100 >= 5 * HUGE_MULT {
                t("cas.s.mega_win")
            } else {
                t("cas.s.big_win")
            };
            let sub = format!("{}{}+{}", self.fmt(m100), DOT, win);
            self.banner = Some(Banner { title, sub, t: 0.0, dur: 2.6 });
            ui::spawn_confetti(self.b.width as f64, self.b.height as f64, if m100 >= 5 * HUGE_MULT { 120 } else { 80 });
            self.coin_shower(if m100 >= 5 * HUGE_MULT { 90 } else { 50 });
            self.shake = 0.35;
            self.b.rumble(240);
        } else if big {
            self.coin_shower(18);
            self.b.rumble(90);
        }
        self.ball_achievements(b);
        if let Some(msg) = b.msg {
            self.say(t(msg), 2.4);
        }
    }

    fn ball_achievements(&self, b: &Ball) {
        if !l::is_edge(b.rows, b.slot) {
            return;
        }
        if b.rows >= l::EDGE_ROWS {
            self.b.ach_event("lpl_edge", None);
        }
        if b.rows == 16 && b.risk == "high" {
            self.b.ach_event("lpl_jackpot", None);
        }
    }

    fn flush_flying(&mut self) {
        let balls = std::mem::take(&mut self.balls);
        for b in &balls {
            self.ball_achievements(b);
        }
        self.pending = 0;
    }

    fn land_sound(&self, m100: i64) {
        let st = Some(&self.b.settings);
        if m100 >= HUGE_MULT {
            self.b.play_sound("level");
        } else if m100 >= BIG_MULT {
            self.b.play_sound("win");
        } else if m100 >= 200 {
            audio::tone(if m100 >= 500 { 784.0 } else { 587.0 }, 0.09, st, "square", 0.12);
        } else if m100 >= 100 {
            audio::tone(440.0, 0.07, st, "square", 0.10);
        } else {
            audio::tone(262.0, 0.06, st, "sine", 0.10);
        }
    }

    // ===================================================== Einstellungen (Spiel)

    fn set_bet(&mut self, direction: i32) {
        if self.auto.is_some() {
            return;
        }
        let i = l::BETS.iter().position(|b| *b == self.bet).unwrap_or(0) as i32;
        let j = 0.max((l::BETS.len() as i32 - 1).min(i + direction));
        if j != i {
            self.bet = l::BETS[j as usize];
            self.set_opt("bet", Value::from(self.bet));
            self.b.play_sound("select");
            if self.bet > self.avail() {
                self.say(t("cas.not_enough"), 1.2);
            }
        }
    }

    fn locked(&mut self) -> bool {
        if !self.balls.is_empty() || self.auto.is_some() {
            self.say(t("lpl.locked"), 1.4);
            self.b.play_sound("hit");
            return true;
        }
        false
    }

    fn set_rows(&mut self, rows: i32) {
        let rows = l::ROWS[0].max(l::ROWS[l::ROWS.len() - 1].min(rows));
        if rows == self.rows || self.locked() {
            return;
        }
        self.rows = rows;
        self.set_opt("rows", Value::from(rows));
        self.board_layout();
        self.new_board_fx();
        self.b.play_sound("click");
    }

    fn set_risk(&mut self, i: usize) {
        let risk = l::RISKS[i];
        if risk == self.risk || self.locked() {
            return;
        }
        self.risk = risk;
        self.set_opt("risk", Value::from(risk));
        self.b.play_sound("click");
    }

    fn toggle_turbo(&mut self) {
        self.turbo = !self.turbo;
        self.set_opt("turbo", Value::from(self.turbo));
        let msg = if self.turbo { t("lpl.turbo_on") } else { t("lpl.turbo_off") };
        self.say(msg, 1.2);
        self.b.play_sound("select");
    }

    fn open(&mut self, name: Panel) {
        if self.panel == Some(name) {
            self.panel = None;
            self.b.play_sound("click");
            return;
        }
        self.panel = Some(name);
        self.hold = false;
        self.panel_layout();
        if name == Panel::Stats {
            self.stats_rows = self.rows;
        } else if name == Panel::Fair {
            self.fair_sel = 0;
            self.fair_scroll = 0;
        }
        self.b.play_sound("select");
    }

    fn new_client(&mut self) {
        if self.locked() {
            return;
        }
        {
            let rng = &mut self.rng;
            fairness::new_client(&mut self.fair, &mut |k| rng.getrandbits(k));
        }
        lamabank::set_extra(KEY, self.extra_value());
        let msg = tf("lpl.client_new", &[("c", &self.fair.client)]);
        self.say(msg, 1.8);
        self.b.play_sound("select");
    }

    /// Nachrechnen: Pruefcode + Ergebnis aus Seed, Client-Seed und Nr. neu.
    fn verify(&mut self, e: &LogEntry) {
        let empty = Map::new();
        let o = e.o.as_object().unwrap_or(&empty);
        let rows = int_like(o.get("rows")).filter(|r| l::ROWS.contains(&(*r as i32)) && *r >= 8 && *r <= 16).map(|r| r as i32);
        let ok_hash = e.verify();
        let path = match rows {
            Some(r) => l::path_from_seed(r, &e.s, &e.c, e.n),
            None => Vec::new(),
        };
        let ok_res = match rows {
            Some(r) => {
                let slot = l::slot_of(&path);
                let risk = o.get("risk").and_then(|v| v.as_str()).filter(|s| l::RISKS.contains(s));
                int_like(o.get("slot")) == Some(slot as i64)
                    && risk.is_some_and(|rk| int_like(o.get("m")) == Some(l::mult100(r, rk, slot)))
            }
            None => false,
        };
        self.fair_checks.insert(e.n, (ok_hash, ok_res, path));
        self.b.play_sound(if ok_hash && ok_res { "point" } else { "hit" });
    }

    // ===================================================== Effekte

    #[allow(clippy::too_many_arguments)]
    fn spawn(&mut self, x: f64, y: f64, n: usize, col: Rgb, speed: f64, up: bool, r: f64, life: f64, grav: f64) {
        let k = self.k;
        for _ in 0..n {
            let a = if up { self.rng.uniform(PI * 1.1, PI * 1.9) } else { self.rng.uniform(0.0, TAU) };
            let sp = speed * self.rng.uniform(0.35, 1.0) * k;
            let pr = r * k * self.rng.uniform(0.6, 1.2);
            let pl = life * self.rng.uniform(0.7, 1.1);
            self.particles.push(Particle { x, y, vx: a.cos() * sp, vy: a.sin() * sp, r: pr, col, t: 0.0, life: pl, g: grav * k });
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
            let x = self.rng.uniform(w * 0.1, w * 0.9);
            let y = self.rng.uniform(-h * 0.4, -10.0);
            let vx = self.rng.uniform(-60.0, 60.0) * k;
            let vy = self.rng.uniform(-80.0, 120.0) * k;
            let spin = self.rng.uniform(0.0, TAU);
            let vs = self.rng.uniform(6.0, 14.0);
            let r = self.rng.uniform(7.0, 12.0) * pmax(0.7, k);
            self.coins_fx.push(CoinFx { x, y, vx, vy, spin, vs, r });
        }
        if self.coins_fx.len() > 160 {
            let cut = self.coins_fx.len() - 160;
            self.coins_fx.drain(..cut);
        }
    }

    fn popup(&mut self, x: f64, y: f64, text: String, color: Color, dur: f64, big: bool) {
        self.popups.push(Popup { x, y, text, color, t: 0.0, dur, big });
        if self.popups.len() > 14 {
            let cut = self.popups.len() - 14;
            self.popups.drain(..cut);
        }
    }

    // ===================================================== Eingabe (Fenster)

    fn panel_event(&mut self, ev: &InputEvent) {
        let Some(name) = self.panel else {
            return;
        };
        let kind = ev.kind;
        if kind == Kind::KeyDown {
            let key = ev.key.as_str();
            let letter: [&str; 2] = match name {
                Panel::Auto => ["a", "A"],
                Panel::Stats => ["s", "S"],
                Panel::Fair => ["f", "F"],
            };
            if key == "Escape" || (letter.contains(&key) && !ev.repeat) {
                self.panel = None;
                self.b.play_sound("click");
                return;
            }
            match name {
                Panel::Auto => self.auto_key(key, ev.repeat),
                Panel::Stats => {
                    if key == "Left" {
                        self.stats_rows = l::ROWS[0].max(self.stats_rows - 1);
                    } else if key == "Right" {
                        self.stats_rows = l::ROWS[l::ROWS.len() - 1].min(self.stats_rows + 1);
                    }
                }
                Panel::Fair => self.fair_key(key),
            }
            return;
        }
        if kind == Kind::Wheel && name == Panel::Fair {
            self.fair_move(if ev.delta > 0 { -1 } else { 1 });
            return;
        }
        if kind != Kind::MouseDown || ev.button != 1 {
            return;
        }
        let pos = ev.pos;
        if self.r_close.hit(pos) || !self.pan.hit(pos) {
            self.panel = None;
            self.b.play_sound("click");
            return;
        }
        for (rect, act) in self.panel_hits() {
            if rect.hit(pos) {
                self.run_act(act);
                return;
            }
        }
    }

    fn run_act(&mut self, act: Act) {
        match act {
            Act::AutoSet(ri_, v) => {
                self.auto_focus = ri_ as i32;
                self.auto_set(ri_, v);
            }
            Act::AutoTurbo => {
                self.auto_focus = 3;
                self.toggle_turbo();
            }
            Act::AutoStart => {
                if self.auto.is_some() {
                    self.stop_auto(None);
                } else {
                    self.start_auto();
                }
            }
            Act::StatsMinus => self.stats_rows = l::ROWS[0].max(self.stats_rows - 1),
            Act::StatsPlus => self.stats_rows = l::ROWS[l::ROWS.len() - 1].min(self.stats_rows + 1),
            Act::NewClient => self.new_client(),
            Act::FairSel(i) => {
                self.fair_sel = i;
                self.b.play_sound("move");
            }
            Act::Verify => {
                if let Some(e) = self.fair_entry() {
                    self.verify(&e);
                }
            }
        }
    }

    fn auto_key(&mut self, key: &str, repeat: bool) {
        let rows: [&[i64]; 3] = [&AUTO_BALLS, &AUTO_LIMITS, &AUTO_LIMITS];
        let lr = key == "Left" || key == "Right";
        if key == "Up" {
            self.auto_focus = py::pmod32(self.auto_focus - 1, 4);
        } else if key == "Down" {
            self.auto_focus = py::pmod32(self.auto_focus + 1, 4);
        } else if lr && self.auto_focus < 3 && self.auto.is_none() {
            let dd = if key == "Left" { -1 } else { 1 };
            let opts = rows[self.auto_focus as usize];
            let cur = [self.auto_balls, self.auto_win, self.auto_loss][self.auto_focus as usize];
            let idx = opts.iter().position(|v| *v == cur).unwrap_or(0) as i32;
            let j = 0.max((opts.len() as i32 - 1).min(idx + dd));
            self.auto_set(self.auto_focus as usize, opts[j as usize]);
        } else if (lr && self.auto_focus == 3) || ((key == "t" || key == "T") && !repeat) {
            // Python: zwei Zweige (Pfeil auf der Turbo-Zeile, Taste T) mit gleicher Aktion
            self.toggle_turbo();
        } else if ["Return", "KP_Enter", "space"].contains(&key) && !repeat {
            if self.auto.is_some() {
                self.stop_auto(None);
            } else {
                self.start_auto();
            }
        }
    }

    fn auto_set(&mut self, row: usize, value: i64) {
        if self.auto.is_some() {
            return;
        }
        if row == 0 {
            self.auto_balls = value;
            self.set_opt("auto_balls", Value::from(value));
        } else if row == 1 {
            self.auto_win = value;
            self.set_opt("auto_win", Value::from(value));
        } else {
            self.auto_loss = value;
            self.set_opt("auto_loss", Value::from(value));
        }
        self.b.play_sound("click");
    }

    fn fair_key(&mut self, key: &str) {
        if key == "Up" || key == "Left" {
            self.fair_move(-1);
        } else if key == "Down" || key == "Right" {
            self.fair_move(1);
        } else if ["Return", "KP_Enter", "space"].contains(&key) {
            if let Some(e) = self.fair_entry() {
                self.verify(&e);
            }
        } else if key == "c" || key == "C" {
            self.new_client();
        }
    }

    /// `_fair_entries()` - neueste zuerst.
    fn fair_entries(&self) -> Vec<LogEntry> {
        self.fairlog_shown.iter().rev().cloned().collect()
    }

    fn fair_entry(&mut self) -> Option<LogEntry> {
        let es = self.fair_entries();
        if es.is_empty() {
            return None;
        }
        self.fair_sel = 0.max((es.len() as i32 - 1).min(self.fair_sel));
        Some(es[self.fair_sel as usize].clone())
    }

    fn fair_move(&mut self, dd: i32) {
        let n = self.fairlog_shown.len() as i32;
        if n == 0 {
            return;
        }
        self.fair_sel = 0.max((n - 1).min(self.fair_sel + dd));
        let vis = self.fair_geom().vis;
        if self.fair_sel < self.fair_scroll {
            self.fair_scroll = self.fair_sel;
        } else if self.fair_sel >= self.fair_scroll + vis {
            self.fair_scroll = self.fair_sel - vis + 1;
        }
        self.b.play_sound("move");
    }

    // ===================================================== Update

    fn update_balls(&mut self, dt: f64) {
        if self.balls.is_empty() {
            return;
        }
        let sp = self.speed();
        for bi in 0..self.balls.len() {
            self.balls[bi].t += dt * sp;
            let n = self.balls[bi].rows;
            let st = seg_time(n);
            while self.balls[bi].hit < n && self.balls[bi].t >= DROP_T + self.balls[bi].hit as f64 * st {
                let r = self.balls[bi].hit;
                let j = self.balls[bi].pre[r as usize] + 1;
                if (r as usize) < self.peg_flash.len() && (j as usize) < self.peg_flash[r as usize].len() {
                    self.peg_flash[r as usize][j as usize] = FLASH_T;
                }
                if self.tick_cd <= 0.0 {
                    self.tick_cd = TICK_GAP;
                    audio::tone(880.0 + r as f64 * 40.0, 0.025, Some(&self.b.settings), "sine", 0.05);
                }
                self.balls[bi].hit += 1;
            }
        }
        if self.balls.iter().any(|b| b.t >= flight_time(b.rows)) {
            let (landed, rest): (Vec<Ball>, Vec<Ball>) =
                std::mem::take(&mut self.balls).into_iter().partition(|b| b.t >= flight_time(b.rows));
            self.balls = rest;
            for b in &landed {
                self.land(b);
            }
            if self.balls.is_empty() {
                if self.auto.is_none() {
                    self.clamp_bet(); // (im Auto bleibt der Einsatz fest)
                }
                self.check_broke();
            }
        }
    }

    fn update_fx(&mut self, dt: f64) {
        for p in self.particles.iter_mut() {
            p.t += dt;
            p.vy += p.g * dt;
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
    }

    // ===================================================== Zeichnen: Brett

    fn draw_board(&self, s: &mut Surface, ox: i32) {
        let bb = self.board_box;
        let plate = d::board_plate(&self.plate_pts, bb.w, bb.h, self.b.accent);
        s.bl(&plate, (bb.x + ox, bb.y));
        s.bl(&d::peg_layer(bb.w, bb.h, &self.pegs_local, self.peg_r), (bb.x + ox, bb.y));
        for (r, row) in self.peg_flash.iter().enumerate() {
            for (j, &f) in row.iter().enumerate() {
                if f > 0.0 {
                    let level = 1.max(8.min((8.0 * f / FLASH_T + 0.999) as i32));
                    let g = d::peg_glow(self.peg_r, level);
                    let (x, y) = self.peg_xy(r as i32, j as i32);
                    s.blr(&g, g.get_rect().center_at((ri(x) + ox, ri(y))));
                }
            }
        }
        let hot = self.hover_slot();
        let table = l::table(self.rows, self.risk);
        let bump_px = self.gap * 0.28;
        let sep = self.sep();
        for (i, rect) in self.slot_rects.iter().enumerate() {
            let m100 = table[i];
            let img = d::slot_surface(rect.w, rect.h, m100, &l::fmt_short(m100, &sep), self.slot_font, hot == Some(i));
            let mut dy = 0;
            if self.slot_bump[i] > 0.0 {
                let p = 1.0 - self.slot_bump[i] / BUMP_T;
                dy = (bump_px * (PI * p).sin() * (1.0 - p * 0.5)) as i32;
            }
            s.bl(&img, (rect.x + ox, rect.y + dy));
        }
    }

    fn draw_llama(&self, s: &mut Surface, ox: i32) {
        let n = self.llama_n;
        let k = self.k;
        let accent = self.b.accent;
        // Sims unter dem Lama (endet knapp vor dem Einwurf)
        let ledge_r = (self.cx - self.gap * 0.55) as i32 + ox;
        let ledge = Rect::new(0, 0, (n as f64 * 1.05) as i32, 4.max((7.0 * k) as i32)).topright_at((ledge_r, self.foot_y));
        draw::rect_r(s, ui::mix(accent, (0, 0, 0), 0.55), ledge.move_(0, 2), 0, ledge.h / 2);
        draw::rect_r(s, ui::mix(accent, (40, 20, 10), 0.35), ledge, 0, ledge.h / 2);
        draw::line(s, ui::mix(accent, (255, 255, 255), 0.4), (ledge.x + 3, ledge.y), (ledge.right() - 3, ledge.y), 1);
        // Trichter ueber dem ersten Stift
        let fw = (self.ball_r * 3 + 6).max((self.gap * 1.35) as i32);
        let fbot = (self.top as f64 - self.gap * 0.42) as i32;
        let fh = 8.max(((self.gap * 1.0) as i32).min(fbot - (self.mouth.1 + self.ball_r as f64 + 2.0) as i32));
        let fun = d::funnel(fw, fh, accent);
        s.bl(&fun, ((self.cx - fw as f64 / 2.0) as i32 + ox, fbot - fh));
        let mut deg = 0;
        if self.nod > 0.0 {
            let p = 1.0 - self.nod / NOD_T;
            deg = -ri(10.0 * (PI * p).sin());
        }
        let img = d::llama_bow(n, deg, accent);
        let (x, y) = d::llama_blit_pos(&img, n, deg, self.foot_x + ox as f64, self.foot_y as f64);
        s.bl(&img, (x, y));
        // naechstes Wollknaeuel im Maul (wenn ein Wurf moeglich ist)
        let ready = !self.broke && self.nod <= 0.0 && self.balls.len() < l::MAX_BALLS && self.bet <= self.avail();
        if ready {
            let (mx, my) = self.mouth;
            let ball = d::yarn_ball(self.mouth_r, 0);
            s.bl(&ball, (mx as i32 - self.mouth_r - 1 + ox, my as i32 - self.mouth_r - 1));
        }
    }

    fn draw_balls(&self, s: &mut Surface, ox: i32) {
        let r = self.ball_r;
        let mut sprite_cache: HashMap<i64, Surface> = HashMap::new();
        for b in &self.balls {
            let t0 = b.t;
            for i in (1..=TRAIL).rev() {
                let tt = t0 - i as f64 * TRAIL_DT;
                if tt <= 0.0 {
                    continue;
                }
                let (x, y) = self.ball_pos(b, tt);
                let a = 150 * (TRAIL + 1 - i) / (TRAIL + 1);
                let rr = 1.max(self.ball_radius(tt) * (TRAIL + 2 - i) / (TRAIL + 2));
                let dot = d::trail_dot(rr, a);
                s.bl(&dot, (x as i32 - rr - 1 + ox, y as i32 - rr - 1));
            }
        }
        for b in &self.balls {
            let (x, y) = self.ball_pos(b, b.t);
            let step = py::pmod(b.spin + (b.t * 16.0) as i64, 12);
            let br = self.ball_radius(b.t);
            if br != r {
                s.bl(&d::yarn_ball(br, step), (x as i32 - br - 1 + ox, y as i32 - br - 1));
                continue;
            }
            let img = sprite_cache.entry(step).or_insert_with(|| d::yarn_ball(r, step));
            s.bl(img, (x as i32 - r - 1 + ox, y as i32 - r - 1));
        }
    }

    fn draw_particles(&self, s: &mut Surface, ox: i32) {
        for p in &self.particles {
            let rr = p.r * (1.0 - p.t / p.life);
            if rr >= 0.5 {
                draw::circle(s, p.col, ((p.x + ox as f64) as i32, p.y as i32), 1.max(rr as i32), 0);
            }
        }
    }

    // ===================================================== Zeichnen: Seitenleisten

    fn side_accent(&self) -> Color {
        ui::mix(self.b.accent, cd::COL_GOLD, 0.3)
    }

    fn draw_left(&self, s: &mut Surface) {
        let p = ui::pal();
        let k = self.k;
        let mut bx = self.left;
        bx.h = self.info_y - bx.y + 2 * self.f_tiny.get_height() + (14.0 * k) as i32;
        ui::draw_panel_ex(s, bx, None, None, None, true, Some(self.side_accent()));
        let pad = 5.max((9.0 * k) as i32);
        let locked = !self.balls.is_empty() || self.auto.is_some();
        let lbl = self.f_tiny.render(&t("lpl.risk"), true, p.text_dim);
        s.bl(&lbl, (bx.x + pad, bx.y + pad - 2));
        for (i, r) in self.r_risks.iter().enumerate() {
            let risk = l::RISKS[i];
            let label = format!("{}  {}", i + 1, self.risk_name(risk));
            self.btn(s, *r, &label, false, !locked || risk == self.risk, r.hit(self.mouse) && !locked, risk == self.risk);
        }
        let lbl = self.f_tiny.render(&t("lpl.rows"), true, p.text_dim);
        s.bl(&lbl, (bx.x + pad, self.lbl_rows_y - 2));
        self.icon_btn(s, self.r_rows_m, "left", !locked && self.rows > l::ROWS[0], false, false, None);
        self.icon_btn(s, self.r_rows_p, "right", !locked && self.rows < l::ROWS[l::ROWS.len() - 1], false, false, None);
        let rv = self.r_rows_v;
        draw::rect_r(s, (16, 12, 26), rv, 0, 6.max(rv.h / 4));
        draw::rect_r(s, p.border, rv, 1, 6.max(rv.h / 4));
        let val = self.f_btn.render(&self.rows.to_string(), true, if !locked { p.gold } else { p.text_dim });
        s.blr(&val, val.get_rect().center_at(rv.center()));
        let top = l::table(self.rows, self.risk).iter().copied().max().unwrap_or(0);
        let mut y = self.info_y;
        let lines = [
            (tf("lpl.max", &[("m", &l::fmt_short(top, &self.sep()))]), cd::COL_GOLD.c()),
            (tf("lpl.rtp", &[("p", &self.rtp_text())]), p.text_dim),
        ];
        for (txt, col) in lines.iter() {
            let mut img = self.f_tiny.render(txt, true, *col);
            if img.w() > bx.w - 2 * pad {
                img = d::fit_font(txt, bx.w - 2 * pad, self.f_tiny.get_height(), false).render(txt, true, *col);
            }
            s.bl(&img, (bx.x + pad, y));
            y += self.f_tiny.get_height() + 2;
        }
        self.draw_session(s, bx.bottom() + self.margin);
    }

    /// Kasten "Sitzung" unter den Einstellungen (nur wenn Platz ist).
    fn draw_session(&self, s: &mut Surface, top: i32) {
        let p = ui::pal();
        let k = self.k;
        let pad = 5.max((9.0 * k) as i32);
        let lh = self.f_tiny.get_height();
        let bx = Rect::new(self.left.x, top, self.left.w, pad * 2 + lh + 3 * (lh + 3) + 2);
        if bx.bottom() > self.left.bottom() {
            return;
        }
        ui::draw_panel_ex(s, bx, None, None, None, true, Some(self.side_accent()));
        s.bl(&self.f_tiny.render(&t("lpl.session"), true, p.text_dim), (bx.x + pad, bx.y + pad - 2));
        let net = self.sess_net;
        let rows = [
            (t("chip.auto_balls"), self.sess_balls.to_string(), p.text),
            (
                t("chip.st_net"),
                format!("{}{}", if net > 0 { "+" } else { "" }, net),
                if net > 0 {
                    p.green
                } else if net < 0 {
                    p.red
                } else {
                    p.text
                },
            ),
            (
                t("lpl.best"),
                if self.sess_best != 0 { self.fmt(self.sess_best) } else { "\u{2013}".to_string() },
                cd::COL_GOLD.c(),
            ),
        ];
        let mut y = bx.y + pad + lh + 2;
        let w = bx.w - 2 * pad;
        for (lbl, val, col) in rows.iter() {
            let mut vi = self.f_btn.render(val, true, *col);
            if vi.w() > w / 2 {
                vi = d::fit_font(val, w / 2, self.f_tiny.get_height(), true).render(val, true, *col);
            }
            let li = d::fit_font(lbl, w - vi.w() - 6, lh, false).render(lbl, true, p.text_dim);
            s.blr(&li, li.get_rect().midleft_at((bx.x + pad, y + (lh + 3) / 2)));
            s.blr(&vi, vi.get_rect().midright_at((bx.right() - pad, y + (lh + 3) / 2)));
            y += lh + 3;
        }
    }

    fn draw_right(&self, s: &mut Surface) {
        let p = ui::pal();
        let k = self.k;
        let pad = 5.max((9.0 * k) as i32);
        let ph = self.pill_h;
        let n_vis = l::HISTORY as i32;
        let head = self.f_tiny.get_height() + pad;
        let mut bx = self.right;
        bx.h = self.right.h.min(head + pad + n_vis * (ph + 3));
        ui::draw_panel_ex(s, bx, None, None, None, true, Some(self.side_accent()));
        let lbl = self.f_tiny.render(&t("chip.history"), true, p.text_dim);
        s.bl(&lbl, (bx.x + pad, bx.y + pad - 2));
        let y0 = bx.y + head + 2;
        let w = bx.w - 2 * pad;
        let slide = (1.0 - self.hist_anim) * (ph + 3) as f64;
        let clip = s.get_clip();
        s.set_clip(Some(Rect::new(bx.x, y0 - 2, bx.w, bx.bottom() - y0)));
        let ghost = ui::mix(p.panel, p.bg_top, 0.45);
        for i in self.hist_shown.len() as i32..n_vis {
            let y = y0 + i * (ph + 3);
            if y > bx.bottom() - ph {
                break;
            }
            draw::rect_r(s, ghost, (bx.x + pad, y, w, ph - 2), 0, ph / 2);
        }
        for (i, &m100) in self.hist_shown.iter().rev().enumerate() {
            let y = (y0 as f64 + (i as i32 * (ph + 3)) as f64 - slide) as i32;
            if y > bx.bottom() - ph {
                break;
            }
            let mut img = d::pill(w, ph, m100, &self.fmt(m100), 9.max((ph as f64 * 0.62) as i32));
            if i == 0 && self.hist_anim < 1.0 {
                img = img.copy();
                img.alpha_set((255.0 * self.hist_anim) as i32);
            }
            s.bl(&img, (bx.x + pad, y));
        }
        s.set_clip(Some(clip));
    }

    // ===================================================== Zeichnen: HUD + Leiste

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
        let chips = (self.shown_chips + 0.5).floor() as i64;
        let left = self.f_big.render(&format!("{}: {}", t("cas.chips"), chips), true, p.gold);
        s.blr(&left, left.get_rect().midleft_at((pad + coin_r * 2 + 6, cy)));
        let right_txt = format!(
            "{}: {}{}{}",
            t("cas.record"),
            self.rec_shown,
            DOT,
            tf("cas.credits", &[("n", &lamabank::refills().to_string())])
        );
        let mut right = self.f_small.render(&right_txt, true, p.text_dim);
        if right.w() as f64 > w as f64 * 0.42 {
            right = self.f_tiny.render(&right_txt, true, p.text_dim);
        }
        let rr = right.get_rect().midright_at((w - pad, cy));
        s.blr(&right, rr);
        let mut parts: Vec<String> = Vec::new();
        if !self.balls.is_empty() {
            parts.push(tf("lpl.in_flight", &[("n", &self.balls.len().to_string())]));
        }
        if let Some(a) = self.auto.as_ref() {
            parts.push(tf("chip.auto_left", &[("n", &a.left.to_string())]));
        }
        if self.turbo {
            parts.push(t("chip.turbo"));
        }
        if parts.is_empty() {
            return;
        }
        let mid_txt = parts.join(DOT);
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

    fn draw_strip(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.strip_bg, (self.strip.x, self.strip.y));
        draw::line(s, ui::mix(cd::COL_GOLD, self.b.accent, 0.35), (self.strip.x, self.strip.y), (self.strip.right(), self.strip.y), 2);
        let hot = self.mouse;
        let free = self.auto.is_none();
        self.btn(s, self.r_minus, "\u{2013}", false, free && self.bet > l::BETS[0], self.r_minus.hit(hot), false);
        self.btn(s, self.r_plus, "+", false, free && self.bet < l::BETS[l::BETS.len() - 1], self.r_plus.hit(hot), false);
        let rb = self.r_bet;
        draw::rect_r(s, (16, 12, 26), rb, 0, 6.max(rb.h / 4));
        draw::rect_r(s, p.border, rb, 1, 6.max(rb.h / 4));
        let lbl = self.f_tiny.render(&t("cas.bet"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midtop_at((rb.centerx(), rb.y + 2)));
        let col = if self.bet > self.avail() { p.red } else { p.gold };
        let val = self.f_btn.render(&self.bet.to_string(), true, col);
        s.blr(&val, val.get_rect().midbottom_at((rb.centerx(), rb.bottom() - 3)));
        let names = ["auto", "stats", "fair", "turbo"];
        let letters = ["A", "S", "F", "T"];
        for ((name, letter), r) in names.iter().zip(letters.iter()).zip(self.r_icons.iter()) {
            let active = (*name == "turbo" && self.turbo) || (*name == "auto" && self.auto.is_some());
            self.icon_btn(s, *r, name, true, r.hit(hot), active, Some(letter));
        }
        if self.auto.is_some() {
            self.btn(s, self.r_drop, &t("chip.auto_stop"), true, true, self.r_drop.hit(hot), false);
        } else {
            let ok = !self.broke && self.bet <= self.avail() && self.balls.len() < l::MAX_BALLS;
            self.btn(s, self.r_drop, &t("lpl.drop"), true, ok, self.r_drop.hit(hot) || self.hold, false);
        }
    }

    fn pulse(&self, speed: f64) -> f64 {
        0.5 + 0.5 * (self.time * speed + 0.0).sin()
    }

    /// `_btn(s, rect, label, primary, on, hot, active)`
    #[allow(clippy::too_many_arguments)]
    fn btn(&self, s: &mut Surface, r: Rect, label: &str, primary: bool, on: bool, hot: bool, active: bool) {
        let p = ui::pal();
        let rad = 6.max(r.h / 4);
        let col = if primary && on {
            let base = ui::mix(self.b.accent, (0, 0, 0), 0.18);
            let fill = ui::mix(base, (255, 255, 255), if hot { 0.12 } else { 0.0 });
            draw::rect_r(s, ui::mix(base, (0, 0, 0), 0.4), r.move_(0, 2), 0, rad);
            draw::rect_r(s, fill, r, 0, rad);
            let border = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.35 * self.pulse(2.4));
            draw::rect_r(s, border, r, 2, rad);
            rgb(255, 255, 255)
        } else {
            let fill = if on && (hot || active) { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, r, 0, rad);
            let edge = if active {
                cd::COL_GOLD.c()
            } else if on {
                p.border_light
            } else {
                p.border
            };
            draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
            if on {
                p.text
            } else {
                p.text_faint
            }
        };
        let mut img = self.f_btn.render(label, true, col);
        if img.w() > r.w - 8 {
            img = self.f_tiny.render(label, true, col);
        }
        if img.w() > r.w - 4 {
            img = d::fit_font(label, r.w - 4, self.f_tiny.get_height(), false).render(label, true, col);
        }
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    /// `_icon_btn(s, rect, name, on, hot, active, letter)`
    #[allow(clippy::too_many_arguments)]
    fn icon_btn(&self, s: &mut Surface, r: Rect, name: &str, on: bool, hot: bool, active: bool, letter: Option<&str>) {
        let p = ui::pal();
        let rad = 5.max(r.h / 4);
        let fill = if on && (hot || active) { p.btn_sel } else { p.btn };
        draw::rect_r(s, fill, r, 0, rad);
        let edge = if active {
            cd::COL_GOLD.c()
        } else if on {
            p.border_light
        } else {
            p.border
        };
        draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
        let col = if on {
            if active {
                cd::COL_GOLD.c()
            } else {
                p.text
            }
        } else {
            p.text_faint
        };
        let size = 8.max((r.h as f64 * if letter.is_some() { 0.52 } else { 0.5 }) as i32);
        let img = d::icon(name, size, col);
        let dy = if letter.is_some() { r.h / 14 } else { 0 };
        s.blr(&img, img.get_rect().center_at((r.centerx(), r.centery() - dy)));
        if let Some(lt) = letter {
            if r.h >= 26 {
                let f = ui::font(8.max((r.h as f64 * 0.26) as i32), true, false);
                let li = f.render(lt, true, p.text_dim);
                s.blr(&li, li.get_rect().bottomright_at((r.right() - 3, r.bottom() - 1)));
            }
        }
    }

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
        for pp in &self.popups {
            let f = pp.t / pp.dur;
            let fnt = if pp.big { &self.f_big } else { &self.f_btn };
            let mut img = fnt.render(&pp.text, true, pp.color);
            let mut sh = fnt.render(&pp.text, true, (10, 6, 16));
            if f > 0.6 {
                let a = (255.0 * (1.0 - (f - 0.6) / 0.4)) as i32;
                img = img.copy();
                img.alpha_set(a);
                sh = sh.copy();
                sh.alpha_set(a);
            }
            let y = pp.y - (if pp.big { 46.0 } else { 30.0 }) * self.k * (1.0 - py::pow(1.0 - f, 2.0));
            let mut r = img.get_rect().midbottom_at((pp.x as i32, y as i32));
            r.clamp_ip(Rect::new(2, 0, self.b.width - 4, self.b.height));
            s.blr(&sh, r.move_(1, 1));
            s.blr(&img, r);
        }
    }

    fn draw_banner(&self, s: &mut Surface) {
        let Some(b) = self.banner.as_ref() else {
            return;
        };
        let p = ui::pal();
        let f = b.t / b.dur;
        let k = self.k;
        let w = self.b.width;
        let grow = pmin(1.0, b.t / 0.18);
        let mut timg = self.f_huge.render(&b.title, true, cd::COL_GOLD);
        if timg.w() > w - 40 {
            timg = self.f_big.render(&b.title, true, cd::COL_GOLD);
        }
        let mut simg = self.f_big.render(&b.sub, true, p.text);
        if simg.w() > w - 40 {
            simg = self.f_small.render(&b.sub, true, p.text);
        }
        let pw = (w - 20).min(timg.w().max(simg.w()) + (48.0 * k) as i32);
        let ph = timg.h() + simg.h() + (30.0 * k) as i32;
        let panel = Rect::new(0, 0, (pw as f64 * (0.7 + 0.3 * grow)) as i32, (ph as f64 * (0.7 + 0.3 * grow)) as i32)
            .center_at((self.cx as i32, self.top + ((self.slot_top - self.top) as f64 * 0.38) as i32));
        let (sw, shh) = s.get_size();
        let mut layer = pg::surface_alpha(sw as i32, shh as i32);
        ui::draw_panel_ex(&mut layer, panel, None, None, None, true, Some(cd::COL_GOLD.c()));
        if grow >= 1.0 {
            layer.blr(&timg, timg.get_rect().midtop_at((panel.centerx(), panel.y + (10.0 * k) as i32)));
            layer.blr(&simg, simg.get_rect().midbottom_at((panel.centerx(), panel.bottom() - (10.0 * k) as i32)));
        }
        if f > 0.8 {
            layer.alpha_set((255.0 * (1.0 - (f - 0.8) / 0.2)) as i32);
        }
        s.bl(&layer, (0, 0));
    }

    fn draw_toast(&self, s: &mut Surface) {
        let Some((text, left)) = self.toast.as_ref() else {
            return;
        };
        let p = ui::pal();
        let mut img = self.f_small.render(text, true, p.text);
        if img.w() > self.b.width - 40 {
            img = self.f_tiny.render(text, true, p.text);
        }
        let mut bx = img.get_rect();
        bx.inflate_ip(24, 12);
        bx = bx.midtop_at((self.cx as i32, (self.top as f64 + (self.slot_top - self.top) as f64 * 0.16) as i32));
        bx.clamp_ip(Rect::new(4, 0, self.b.width - 8, self.b.height));
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

    fn hover_slot(&self) -> Option<usize> {
        if self.panel.is_some() {
            return None;
        }
        self.slot_rects.iter().position(|r| r.inflate(2, 6).hit(self.mouse))
    }

    fn draw_hover(&self, s: &mut Surface) {
        if let Some(i) = self.hover_slot() {
            let sep = self.sep();
            let m100 = l::mult100(self.rows, self.risk, i as i32);
            let lines = vec![
                format!("{}{}{}", self.fmt(m100), DOT, tf("lpl.slot", &[("n", &i.to_string())])),
                tf("lpl.tip_chance", &[("p", &l::fmt_prob(self.rows, i as i32, &sep))]),
                tf("lpl.tip_win", &[("b", &self.bet.to_string()), ("n", &l::fmt_short(self.bet * m100, &sep))]),
            ];
            charts::tooltip(s, self.mouse, &lines, None);
            return;
        }
        let tips = [
            (self.r_icons[0], format!("{}  (A)", t("chip.auto_title"))),
            (self.r_icons[1], format!("{}  (S)", t("chip.stats"))),
            (self.r_icons[2], format!("{}  (F)", t("chip.fair_title"))),
            (self.r_icons[3], format!("{}  (T)", t("chip.turbo"))),
        ];
        for (r, txt) in tips.iter() {
            if r.hit(self.mouse) {
                charts::tooltip(s, self.mouse, std::slice::from_ref(txt), None);
                return;
            }
        }
    }

    fn draw_broke(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let pw = (self.b.width - 40).min((480.0 * pmax(0.8, self.k)) as i32);
        let ph = (200.0 * pmax(0.75, self.k)) as i32;
        let panel = Rect::new(cx - pw / 2, cy - ph / 2, pw, ph);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(p.red));
        let head = self.f_huge.render(&t("cas.broke"), true, p.red);
        s.blr(&head, head.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * 0.28)));
        let lines = [
            (t("cas.broke_sub"), p.text_dim),
            (tf("cas.broke_restart", &[("n", &lamabank::START_CHIPS.to_string())]), p.text),
        ];
        for (i, (txt, col)) in lines.iter().enumerate() {
            let mut img = self.f_small.render(txt, true, *col);
            if img.w() > pw - 16 {
                img = self.f_tiny.render(txt, true, *col);
            }
            s.blr(&img, img.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * (0.58 + 0.2 * i as f64))));
        }
    }

    // ===================================================== Fenster

    fn panel_frame(&self, s: &mut Surface, icon_name: &str, title: &str) -> Rect {
        let p = ui::pal();
        s.bl(&self.dim, (0, 0));
        ui::draw_panel_ex(s, self.pan, None, None, None, true, Some(self.b.accent));
        let k = self.k;
        let pn = self.pan;
        let hy = pn.y + self.pan_head / 2;
        let ic = d::icon(icon_name, 12.max((self.pan_head as f64 * 0.5) as i32), cd::COL_GOLD);
        s.blr(&ic, ic.get_rect().midleft_at((pn.x + self.margin, hy)));
        let timg = self.f_big.render(title, true, p.text);
        s.blr(&timg, timg.get_rect().midleft_at((pn.x + self.margin + ic.w() + (8.0 * k) as i32, hy)));
        self.icon_btn(s, self.r_close, "close", true, self.r_close.hit(self.mouse), false, None);
        draw::line(s, p.border, (pn.x + 8, pn.y + self.pan_head), (pn.right() - 8, pn.y + self.pan_head), 1);
        self.panel_body()
    }

    fn panel_hits(&self) -> Vec<(Rect, Act)> {
        match self.panel {
            Some(Panel::Auto) => self.auto_rects().hits,
            Some(Panel::Stats) => {
                let body = self.panel_body();
                let (r_m, r_p) = self.stats_row_btns(body);
                vec![(r_m, Act::StatsMinus), (r_p, Act::StatsPlus)]
            }
            Some(Panel::Fair) => self.fair_rects().2,
            None => Vec::new(),
        }
    }

    fn panel_body(&self) -> Rect {
        let pn = self.pan;
        Rect::new(
            pn.x + self.margin,
            pn.y + self.pan_head + self.margin / 2,
            pn.w - 2 * self.margin,
            pn.h - self.pan_head - self.margin,
        )
    }

    // ----- Auto

    /// `_auto_metrics()`: (lh, bh, gap, row_gap, sep, sum_h, start_h, start_gap)
    fn auto_metrics(&self) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
        let k = self.k;
        (
            self.f_tiny.get_height(),
            22.max((30.0 * k) as i32),
            3.max((5.0 * k) as i32),
            6.max((12.0 * k) as i32),
            8.max((14.0 * k) as i32),
            self.f_small.get_height(),
            30.max((42.0 * k) as i32),
            6.max((10.0 * k) as i32),
        )
    }

    fn auto_content_h(&self) -> i32 {
        let (lh, bh, _gap, row_gap, sep, sum_h, start_h, start_gap) = self.auto_metrics();
        3 * (lh + 3 + bh + row_gap) + bh + sep + sum_h + start_gap + start_h
    }

    fn auto_rects(&self) -> AutoGeom {
        let body = self.panel_body();
        let (lh, bh, gap, row_gap, sep, sum_h, start_h, start_gap) = self.auto_metrics();
        let mut rows = Vec::new();
        let mut hits = Vec::new();
        let mut y = body.y;
        let lists: [&[i64]; 3] = [&AUTO_BALLS, &AUTO_LIMITS, &AUTO_LIMITS];
        for (ri_, opts) in lists.iter().enumerate() {
            y += lh + 3;
            let n = opts.len() as i32;
            let bw = py::fdiv32(body.w - (n - 1) * gap, n);
            let mut rects = Vec::new();
            for (j, &v) in opts.iter().enumerate() {
                let r = Rect::new(body.x + j as i32 * (bw + gap), y, bw, bh);
                rects.push((r, v));
                hits.push((r, Act::AutoSet(ri_, v)));
            }
            rows.push((y - lh - 3, rects));
            y += bh + row_gap;
        }
        let tw = body.w.min(((body.w as f64 * 0.36) as i32).max(self.f_btn.width(&t("chip.turbo")) + 3 * bh + 10));
        let turbo = Rect::new(body.x, y, tw, bh);
        hits.push((turbo, Act::AutoTurbo));
        y += bh + sep;
        let sum_y = y;
        y += sum_h + start_gap;
        let start = Rect::new(body.x + body.w / 4, (body.bottom() - start_h).min(y), body.w / 2, start_h);
        hits.push((start, Act::AutoStart));
        AutoGeom { rows, turbo, sum_y, start, hits }
    }

    fn draw_auto(&self, s: &mut Surface) {
        let p = ui::pal();
        self.panel_frame(s, "auto", &t("chip.auto_title"));
        let g = self.auto_rects();
        let labels = [t("chip.auto_balls"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
        let cur = [self.auto_balls, self.auto_win, self.auto_loss];
        let running = self.auto.is_some();
        for (ri_, (ly, rects)) in g.rows.iter().enumerate() {
            let col = if self.auto_focus == ri_ as i32 { p.text } else { p.text_dim };
            let img = self.f_tiny.render(&labels[ri_], true, col);
            s.bl(&img, (rects[0].0.x, *ly));
            for (r, v) in rects.iter() {
                let txt = if ri_ == 0 || *v != 0 { v.to_string() } else { t("chip.auto_off") };
                self.btn(s, *r, &txt, false, !running || *v == cur[ri_], r.hit(self.mouse) && !running, *v == cur[ri_]);
            }
        }
        self.toggle_btn(s, g.turbo, &t("chip.turbo"), self.turbo, self.auto_focus == 3);
        let total = self.auto_balls * self.bet;
        let txt = tf(
            "lpl.auto_sum",
            &[("n", &self.auto_balls.to_string()), ("b", &self.bet.to_string()), ("t", &total.to_string())],
        );
        let col = if self.bet > self.avail() { p.red } else { p.text_dim };
        let mut img = self.f_small.render(&txt, true, col);
        if img.w() > self.pan.w - 2 * self.margin {
            img = self.f_tiny.render(&txt, true, col);
        }
        s.blr(&img, img.get_rect().midtop_at((self.pan.centerx(), g.sum_y)));
        if let Some(a) = self.auto.as_ref() {
            let lbl = format!("{}  \u{00B7}  {}", t("chip.auto_stop"), tf("chip.auto_left", &[("n", &a.left.to_string())]));
            self.btn(s, g.start, &lbl, true, true, g.start.hit(self.mouse), false);
        } else {
            self.btn(s, g.start, &t("chip.auto_start"), true, !self.broke, g.start.hit(self.mouse), false);
        }
    }

    fn toggle_btn(&self, s: &mut Surface, r: Rect, label: &str, on: bool, focus: bool) {
        let p = ui::pal();
        let hot = r.hit(self.mouse);
        draw::rect_r(s, if hot || focus { p.btn_sel } else { p.btn }, r, 0, 6.max(r.h / 4));
        draw::rect_r(s, if on { cd::COL_GOLD.c() } else { p.border_light }, r, if on { 2 } else { 1 }, 6.max(r.h / 4));
        let sw = Rect::new(0, 0, (r.h as f64 * 1.5) as i32, (r.h as f64 * 0.56) as i32).midleft_at((r.x + 8, r.centery()));
        draw::rect_r(s, if on { cd::COL_GOLD } else { (60, 60, 76) }, sw, 0, sw.h / 2);
        let knob = sw.h / 2 - 2;
        let kx = if on { sw.right() - knob - 3 } else { sw.x + knob + 3 };
        draw::circle(s, (255, 255, 255), (kx, sw.centery()), knob, 0);
        let ic = d::icon("turbo", 8.max((r.h as f64 * 0.5) as i32), if on { cd::COL_GOLD.c() } else { p.text_dim });
        s.blr(&ic, ic.get_rect().midleft_at((sw.right() + 6, r.centery())));
        let img = self.f_btn.render(label, true, p.text);
        s.blr(&img, img.get_rect().midleft_at((sw.right() + 10 + ic.w(), r.centery())));
    }

    // ----- Statistik

    fn stats_row_btns(&self, body: Rect) -> (Rect, Rect) {
        let bh = 18.max((24.0 * self.k) as i32);
        let y = self.stats_dist_y(body);
        let r_p = Rect::new(body.right() - bh, y, bh, bh);
        let r_m = Rect::new(r_p.x - bh - 4, y, bh, bh);
        (r_m, r_p)
    }

    fn stats_dist_y(&self, body: Rect) -> i32 {
        let tile_h = 34.max((46.0 * self.k) as i32);
        body.y + 3 * (tile_h + 4) + 6.max((10.0 * self.k) as i32)
    }

    fn pct(&self, num: i64, den: i64) -> String {
        let v = py::fdiv(num * 20000 + den, 2 * den);
        format!("{}{}{:02}", py::fdiv(v, 100), self.sep(), py::pmod(v, 100))
    }

    fn draw_stats(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let body = self.panel_frame(s, "stats", &t("chip.stats"));
        let k = self.k;
        let sep = self.sep();
        let st = self.stats_shown.clone();
        if st.rounds <= 0 {
            let img = self.f_small.render(&t("chip.st_empty"), true, p.text_dim);
            s.blr(&img, img.get_rect().center_at(body.center()));
            return;
        }
        let rtp_txt = if st.rtp().is_none() { "\u{2013}".to_string() } else { self.pct(st.paid, st.wagered) };
        let tiles = [
            (t("chip.st_rounds"), st.rounds.to_string(), p.text),
            (t("chip.st_wins"), st.wins.to_string(), p.green),
            (t("chip.st_losses"), st.losses.to_string(), p.red),
            (t("chip.st_wagered"), st.wagered.to_string(), p.text),
            (t("chip.st_paid"), st.paid.to_string(), p.text),
            (
                t("chip.st_net"),
                format!("{}{}", if st.net > 0 { "+" } else { "" }, st.net),
                if st.net >= 0 { p.green } else { p.red },
            ),
            (t("chip.st_best_win"), format!("+{}", st.best_win), cd::COL_GOLD.c()),
            (t("chip.st_best_mult"), format!("\u{00D7}{}", l::fmt_short(st.best_mult, &sep)), cd::COL_GOLD.c()),
            (t("chip.st_rtp"), format!("{} %", rtp_txt), p.text),
        ];
        let tile_h = 34.max((46.0 * k) as i32);
        let half = body.w / 2 - 4;
        let tw = py::fdiv32(half - 2 * 4, 3);
        for (i, (lbl, val, col)) in tiles.iter().enumerate() {
            let i = i as i32;
            let r = Rect::new(body.x + (i % 3) * (tw + 4), body.y + (i / 3) * (tile_h + 4), tw, tile_h);
            draw::rect_r(s, ui::mix(p.panel, p.bg_top, 0.35), r, 0, 4.max(r.h / 5));
            let li = d::fit_font(lbl, r.w - 6, self.f_tiny.get_height(), false).render(lbl, true, p.text_dim);
            s.blr(&li, li.get_rect().midtop_at((r.centerx(), r.y + 3)));
            let vi = d::fit_font(val, r.w - 6, self.f_btn.get_height(), true).render(val, true, *col);
            s.blr(&vi, vi.get_rect().midbottom_at((r.centerx(), r.bottom() - 3)));
        }
        // Bilanzkurve rechts
        let cr = Rect::new(body.x + half + 8, body.y, body.w - half - 8, 3 * (tile_h + 4) - 4);
        draw::rect_r(s, ui::mix(p.panel, p.bg_top, 0.35), cr, 0, 4.max((6.0 * k) as i32));
        let cap = tf("chip.st_curve", &[("n", &st.curve.len().to_string())]);
        let ci = d::fit_font(&cap, cr.w - 8, self.f_tiny.get_height(), false).render(&cap, true, p.text_dim);
        s.bl(&ci, (cr.x + 6, cr.y + 3));
        let plot = Rect::new(cr.x + 6, cr.y + ci.h() + 8, cr.w - 12, cr.h - ci.h() - 14);
        if st.curve.len() >= 2 {
            let col = if st.curve[st.curve.len() - 1] >= 0 { p.green } else { p.red };
            let mut vals: Vec<f64> = vec![0.0];
            vals.extend(st.curve.iter().map(|v| *v as f64));
            let fmt = |v: f64| ((v + 0.5).floor() as i64).to_string();
            charts::line(s, plot, &vals, Some(col), &[], Some(&fmt), None);
        }
        // Treffer-Verteilung (gewaehlte Reihenzahl) gegen die erwartete Binomialkurve
        let n = self.stats_rows;
        let cap = tf("lpl.st_dist", &[("n", &n.to_string())]);
        let (r_m, r_p) = self.stats_row_btns(body);
        self.icon_btn(s, r_m, "left", n > l::ROWS[0], r_m.hit(self.mouse), false, None);
        self.icon_btn(s, r_p, "right", n < l::ROWS[l::ROWS.len() - 1], r_p.hit(self.mouse), false, None);
        let cap_img = d::fit_font(&cap, r_m.x - body.x - 8, self.f_small.get_height(), true).render(&cap, true, p.text);
        s.blr(&cap_img, cap_img.get_rect().midleft_at((body.x, r_m.centery())));
        let hits = self.hits_shown[(n - 8) as usize].clone();
        let mut total: i64 = 0;
        for v in &hits {
            total += v;
        }
        let mut area = Rect::new(body.x, r_m.bottom() + 6, body.w, body.bottom() - r_m.bottom() - 6);
        let leg = t("lpl.st_legend");
        let li = d::fit_font(&leg, area.w, self.f_tiny.get_height(), false).render(&leg, true, p.text_faint);
        s.blr(&li, li.get_rect().bottomleft_at((area.x, area.bottom())));
        area.h -= li.h() + 4;
        if total <= 0 {
            let img = self.f_small.render(&tf("lpl.st_none", &[("n", &n.to_string())]), true, p.text_dim);
            s.blr(&img, img.get_rect().center_at(area.center()));
            return;
        }
        let pow2 = (1i64 << n) as f64;
        let exp: Vec<f64> = (0..=n).map(|i| (total * l::comb(n as i64, i as i64)) as f64 / pow2).collect();
        let max_hits = hits.iter().copied().max().unwrap_or(0);
        let max_exp = exp.iter().copied().fold(f64::NEG_INFINITY, pmax);
        let top = pmax(max_hits as f64, max_exp);
        let table = l::table(n, self.risk);
        let cols: Vec<Color> = table.iter().map(|m| ui::mix(d::slot_color(*m), p.panel, 0.25)).collect();
        let mut hover = None;
        let lbls: Option<Vec<String>> =
            if area.w as f64 / (n + 1) as f64 >= 16.0 { Some((0..=n).map(|i| i.to_string()).collect()) } else { None };
        let vals: Vec<f64> = hits.iter().map(|v| *v as f64).collect();
        let label_size = 9.max((10.0 * k) as i32);
        let hits_r = charts::bars_ex(
            s,
            area,
            &vals,
            lbls.as_deref(),
            Some(&cols),
            None,
            Some(top),
            false,
            None,
            self.stats_hover,
            label_size,
        );
        // erwartete Kurve: Punkte auf Hoehe des Erwartungswerts ueber den Balkenmitten
        let fnt_h = if lbls.is_some() { ui::font(label_size, false, false).get_height() + 2 } else { 0 };
        let val_h = ui::font(label_size, true, false).get_height() + 2;
        let plot_h = 8.max(area.h - fnt_h - val_h);
        let mut pts: Vec<(i32, i32)> = Vec::new();
        for (cell, i) in hits_r.iter() {
            let y = (area.y + val_h + plot_h) as f64 - plot_h as f64 * (exp[*i] / top);
            pts.push((cell.centerx(), y as i32));
            if cell.hit(self.mouse) {
                hover = Some(*i);
            }
        }
        if pts.len() >= 2 {
            draw::lines(s, cd::COL_GOLD, false, &pts, 2);
        }
        for pt in &pts {
            draw::circle(s, cd::COL_GOLD, *pt, 3, 0);
            draw::circle(s, (40, 26, 8), *pt, 1, 0);
        }
        self.stats_hover = hover;
        if let Some(hv) = hover {
            let e = exp[hv];
            let ev = (e * 10.0 + 0.5) as i64;
            let lines = vec![
                format!("{}{}{}", tf("lpl.slot", &[("n", &hv.to_string())]), DOT, self.fmt(table[hv])),
                tf("lpl.st_bar", &[("n", &hits[hv].to_string()), ("e", &format!("{}{}{}", ev / 10, sep, ev % 10))]),
            ];
            charts::tooltip(s, self.mouse, &lines, None);
        }
    }

    // ----- Fairness

    fn fair_geom(&self) -> FairGeom {
        let body = self.panel_body();
        let k = self.k;
        let lh = self.f_tiny.get_height();
        let note_h = lh + 2;
        let bh = 22.max((28.0 * k) as i32);
        let y = body.y + 2 * lh + 4;
        let nc_w = (body.w / 2).min(self.f_btn.width(&t("chip.fair_new_client")) + 20);
        let r_new = Rect::new(body.right() - nc_w, y + lh - bh / 2, nc_w, bh);
        let hash_y = y + 2 * lh + 2;
        let ly = hash_y + lh + 6.max((8.0 * k) as i32);
        let bottom = body.bottom() - note_h;
        let lw = ((body.w as f64 * 0.34) as i32).max(118);
        let step = self.pill_h + 4;
        let vis = 1.max(py::fdiv32(bottom - ly, step));
        let detail = Rect::new(body.x + lw + self.margin, ly, body.w - lw - self.margin, bottom - ly);
        let pv = 24.max(((detail.w as f64 * 0.36) as i32).min(((detail.h - bh - 8) as f64 * 0.62) as i32));
        let preview = Rect::new(detail.right() - pv - 4, detail.y + 4, pv, (pv as f64 * 1.05) as i32);
        let ver_w = (detail.w - 8).min((self.f_btn.width(&t("chip.fair_verify")) + 28).max((detail.w as f64 * 0.4) as i32));
        let r_ver = Rect::new(detail.x + 4, detail.bottom() - bh - 4, ver_w, bh);
        FairGeom { body, lh, y, r_new, hash_y, ly, lw, step, vis, detail, preview, r_ver, bottom }
    }

    fn fair_rects(&self) -> FairRects {
        let g = self.fair_geom();
        let mut rows = Vec::new();
        let mut hits = vec![(g.r_new, Act::NewClient)];
        let n = self.fairlog_shown.len() as i32;
        let mut i = self.fair_scroll;
        while i < n.min(self.fair_scroll + g.vis) {
            let r = Rect::new(g.body.x, g.ly + (i - self.fair_scroll) * g.step, g.lw, self.pill_h);
            rows.push((r, i));
            hits.push((r, Act::FairSel(i)));
            i += 1;
        }
        if n > 0 {
            hits.push((g.r_ver, Act::Verify));
        }
        (g, rows, hits)
    }

    fn draw_fair(&mut self, s: &mut Surface) {
        let p = ui::pal();
        self.panel_frame(s, "fair", &t("chip.fair_title"));
        let (g, rows, _) = self.fair_rects();
        let (body, lh, detail) = (g.body, g.lh, g.detail);
        let k = self.k;
        let sep = self.sep();
        // Erklaerung (hoechstens zwei Zeilen)
        let mut y = body.y;
        for line in self.wrap(&t("chip.fair_info"), &self.f_tiny, body.w, 2) {
            s.bl(&self.f_tiny.render(&line, true, p.text_dim), (body.x, y));
            y += lh;
        }
        // Naechster Pruefcode, Client-Seed, Knopf "Neuer Client-Seed"
        let y = g.y;
        let r_new = g.r_new;
        let left_w = r_new.x - body.x - 8;
        let nxt = format!("{}:", t("chip.fair_next"));
        let img = d::fit_font(&nxt, left_w, lh, false).render(&nxt, true, p.text);
        s.bl(&img, (body.x, y));
        let ci_txt = format!(
            "{}: {}{}{} {}",
            t("chip.fair_client"),
            self.fair.client,
            DOT,
            t("chip.fair_nonce"),
            self.fair.nonce
        );
        let ci = d::fit_font(&ci_txt, left_w, lh, false).render(&ci_txt, true, p.text_dim);
        s.bl(&ci, (body.x, y + lh));
        self.btn(s, r_new, &t("chip.fair_new_client"), false, self.balls.is_empty() && self.auto.is_none(), r_new.hit(self.mouse), false);
        let hi = self.mono_fit(&fairness::commit(&self.fair.server), body.w, cd::COL_GOLD.c(), None);
        s.bl(&hi, (body.x, g.hash_y));
        let ly = g.ly - 3.max((4.0 * k) as i32);
        draw::line(s, p.border, (body.x, ly), (body.right(), ly), 1);
        let note_txt = t("chip.fair_note");
        let note = d::fit_font(&note_txt, body.w, lh, false).render(&note_txt, true, p.text_faint);
        s.blr(&note, note.get_rect().midbottom_at((body.centerx(), body.bottom())));
        let es = self.fair_entries();
        if es.is_empty() {
            let img = self.f_small.render(&t("chip.fair_empty"), true, p.text_dim);
            s.blr(&img, img.get_rect().center_at((body.centerx(), py::fdiv32(g.ly + g.bottom, 2))));
            return;
        }
        let Some(sel) = self.fair_entry() else {
            return;
        };
        let empty = Map::new();
        // Liste der letzten Kugeln (neueste oben)
        for (r, i) in rows.iter() {
            let e = &es[*i as usize];
            let o = e.o.as_object().unwrap_or(&empty);
            let on = *i == self.fair_sel;
            let hot = r.hit(self.mouse);
            draw::rect_r(s, if on || hot { p.btn_sel } else { ui::mix(p.panel, p.bg_top, 0.35) }, *r, 0, r.h / 2);
            if on {
                draw::rect_r(s, cd::COL_GOLD, *r, 1, r.h / 2);
            }
            let ni = self.f_tiny.render(&format!("#{}", e.n), true, if on { p.text } else { p.text_dim });
            s.blr(&ni, ni.get_rect().midleft_at((r.x + 8, r.centery())));
            let m = match o.get("m") {
                Some(Value::Number(nm)) if !nm.is_f64() => nm.as_i64().unwrap_or(0),
                _ => 0,
            };
            let pw = 30.max(r.w * 2 / 5);
            let pill = d::pill(pw, 10.max(r.h - 4), m, &self.fmt(m), 8.max((r.h as f64 * 0.55) as i32));
            s.blr(&pill, pill.get_rect().midright_at((r.right() - 3, r.centery() + 1)));
            if let Some(chk) = self.fair_checks.get(&e.n) {
                let good = chk.0 && chk.1;
                let ic = d::icon(if good { "check" } else { "cross" }, 8.max(r.h - 6), if good { p.green } else { p.red });
                s.blr(&ic, ic.get_rect().midright_at((r.right() - pw - 6, r.centery())));
            }
        }
        if es.len() > rows.len() && !rows.is_empty() {
            let more = self.f_tiny.render(
                &format!("{}-{} / {}", self.fair_scroll + 1, self.fair_scroll + rows.len() as i32, es.len()),
                true,
                p.text_faint,
            );
            let last = rows[rows.len() - 1].0;
            if last.bottom() + more.h() <= g.bottom {
                s.blr(&more, more.get_rect().midtop_at((last.centerx(), last.bottom() + 1)));
            }
        }
        // Details des gewaehlten Eintrags
        draw::rect_r(s, ui::mix(p.panel, p.bg_top, 0.35), detail, 0, 4.max((6.0 * k) as i32));
        let pv = g.preview;
        let o = sel.o.as_object().unwrap_or(&empty);
        // Multiplikator robust lesen (int ohne bool, sonst 0) - wie Python `om`
        let om = match o.get("m") {
            Some(Value::Number(nm)) if !nm.is_f64() => nm.as_i64().unwrap_or(0),
            _ => 0,
        };
        let xx = detail.x + 6;
        let tw = pv.x - xx - 6;
        let mut y = detail.y + 4;
        let head = format!("{} {}", t("chip.fair_nonce"), sel.n);
        let hi = self.f_btn.render(&head, true, p.text);
        s.bl(&hi, (xx, y));
        y += hi.h() + 1;
        let o_rows = int_like(o.get("rows")).filter(|r| (8..=16).contains(r));
        let o_risk = o.get("risk").and_then(|v| v.as_str()).filter(|r| l::RISKS.contains(r));
        if let (Some(rows_v), Some(risk_v)) = (o_rows, o_risk) {
            let m_txt = l::fmt_short(om, &sep);
            let res = tf(
                "lpl.fair_result",
                &[
                    ("rows", &rows_v.to_string()),
                    ("risk", &self.risk_name(risk_v)),
                    ("slot", &py_str(o.get("slot"))),
                    ("m", &m_txt),
                ],
            );
            let rimg = d::fit_font(&res, tw, self.f_tiny.get_height() + 1, true).render(&res, true, cd::COL_GOLD);
            s.bl(&rimg, (xx, y));
            y += rimg.h() + 3;
        }
        let blocks: [(String, Vec<String>); 3] = [
            (t("chip.fair_hash"), vec![sel.h.chars().take(32).collect(), sel.h.chars().skip(32).collect()]),
            (t("chip.fair_server"), vec![sel.s.clone()]),
            (t("chip.fair_client"), vec![sel.c.clone()]),
        ];
        for (lbl, vals) in blocks.iter() {
            let lt = format!("{}:", lbl);
            let li = d::fit_font(&lt, tw, lh, false).render(&lt, true, p.text_dim);
            s.bl(&li, (xx, y));
            y += lh;
            for v in vals {
                let vi = self.mono_fit(v, tw, p.text, Some(32));
                s.bl(&vi, (xx, y));
                y += vi.h();
            }
            y += 2;
        }
        // Vorschau-Rahmen rechts oben (nach dem Nachrechnen mit dem Weg der Kugel)
        draw::rect_r(s, ui::mix(p.panel, p.bg_top, 0.6), pv, 0, 3.max((5.0 * k) as i32));
        let chk = self.fair_checks.get(&sel.n).cloned();
        let r_ver = g.r_ver;
        self.btn(s, r_ver, &t("chip.fair_verify"), chk.is_none(), true, r_ver.hit(self.mouse), false);
        let Some((ok_hash, ok_res, path)) = chk else {
            let ic = d::icon("fair", 12.max(pv.w / 3), p.text_faint);
            s.blr(&ic, ic.get_rect().center_at(pv.center()));
            return;
        };
        let good = ok_hash && ok_res;
        let col = if good { p.green } else { p.red };
        if !path.is_empty() {
            let prow = int_like(o.get("rows")).map(|r| r as i32).unwrap_or(path.len() as i32);
            let img = d::path_preview(prow, &path, pv.w - 4, pv.h - 4, Some(om));
            s.bl(&img, (pv.x + 2, pv.y + 2));
        }
        // Ergebnis der Pruefung unter der Vorschau
        let y2 = pv.bottom() + 4;
        let ic = d::icon(if good { "check" } else { "cross" }, lh + 4, col);
        let word = if good { t("chip.fair_ok") } else { t("chip.fair_bad") };
        let wi = d::fit_font(&word, pv.w - ic.w() - 4, self.f_btn.get_height(), true).render(&word, true, col);
        s.bl(&ic, (pv.x, y2));
        s.blr(&wi, wi.get_rect().midleft_at((pv.x + ic.w() + 4, y2 + ic.h() / 2)));
        if !path.is_empty() {
            let calc = tf("lpl.fair_recalc", &[("slot", &l::slot_of(&path).to_string())]);
            let cimg = d::fit_font(&calc, pv.w + 4, lh, false).render(&calc, true, p.text_dim);
            if y2 + ic.h() + cimg.h() <= r_ver.y + r_ver.h {
                s.bl(&cimg, (pv.x - 2, y2 + ic.h() + 1));
            }
        }
    }

    /// Hex-Werte in Monospace: groesste Schrift, in der ref Zeichen (sonst der
    /// Text selbst) in max_w passen; notfalls gekuerzt mit "…".
    fn mono_fit(&self, text: &str, max_w: i32, color: Color, refn: Option<usize>) -> Surface {
        let max_w = 20.max(max_w);
        let probe = match refn {
            Some(n) if n > 0 => "0".repeat(n),
            _ => text.to_string(),
        };
        let mut f = ui::font(9, false, true);
        let mut sz = 10.max((14.0 * self.k) as i32);
        while sz > 8 {
            f = ui::font(sz, false, true);
            if f.width(&probe) <= max_w {
                break;
            }
            sz -= 1;
        }
        let mut cut: Vec<char> = text.chars().collect();
        while cut.len() > 4 && f.width(&cut.iter().collect::<String>()) > max_w {
            cut.truncate(cut.len() - 2);
            cut.push('\u{2026}');
        }
        f.render(&cut.iter().collect::<String>(), true, color)
    }

    fn wrap(&self, text: &str, fnt: &Font, w: i32, max_lines: usize) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        for wd in text.split_whitespace() {
            let test = format!("{} {}", cur, wd).trim().to_string();
            if fnt.width(&test) <= w || cur.is_empty() {
                cur = test;
            } else {
                lines.push(std::mem::replace(&mut cur, wd.to_string()));
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        if lines.len() > max_lines {
            lines.truncate(max_lines);
            let mut last: Vec<char> = lines[max_lines - 1].chars().collect();
            while last.len() > 3 && fnt.width(&format!("{}\u{2026}", last.iter().collect::<String>())) > w {
                last.pop();
            }
            lines[max_lines - 1] = format!("{}\u{2026}", last.iter().collect::<String>());
        }
        lines
    }
}

impl Game for LamaPlinkoGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        if !self.balls.is_empty() {
            self.flush_flying();
        }
        self.b.game_over = false;
        lamabank::load();
        self.rng = py::Random::unseeded();
        // Optionen wie settings.SCHEMAS (+ Rueckfall des Spiels)
        self.rows = match self.b.opt_value(KEY, "rows") {
            Some(Value::Number(n)) if !n.is_f64() => n.as_i64().map(|v| v.clamp(8, 16) as i32).unwrap_or(l::DEFAULT_ROWS),
            _ => l::DEFAULT_ROWS,
        };
        self.risk = match self.b.opt_value(KEY, "risk") {
            Some(Value::String(r)) => l::RISKS.iter().copied().find(|x| *x == r).unwrap_or(l::DEFAULT_RISK),
            _ => l::DEFAULT_RISK,
        };
        self.bet = self.opt_choice("bet", &l::BETS, 10);
        self.turbo = matches!(self.b.opt_value(KEY, "turbo"), Some(Value::Bool(true)));
        self.auto_balls = self.opt_choice("auto_balls", &AUTO_BALLS, 25);
        self.auto_win = self.opt_choice("auto_win", &AUTO_LIMITS, 0);
        self.auto_loss = self.opt_choice("auto_loss", &AUTO_LIMITS, 0);
        let extra = match lamabank::get_extra(KEY) {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        self.hist = l::clean_history(extra.get("hist"));
        {
            let rng = &mut self.rng;
            self.fair = fairness::clean_state(extra.get("fair"), &mut |k| rng.getrandbits(k));
        }
        self.fairlog = fairness::clean_log(extra.get("fairlog"));
        self.stats = chipstats::clean(extra.get("stats"));
        self.carry = l::clean_carry(extra.get("carry"));
        self.hits = l::clean_hits(self.stats.x.get("hits"));
        self.stats.x.insert("hits".into(), l::hits_to_value(&self.hits));
        let gleich = extra.get("fair").is_some_and(|v| py_eq(v, &self.fair.to_value()));
        if !gleich {
            // neue Seeds sofort festschreiben: ein gezeigter Pruefcode gilt auch nach einem Neustart
            lamabank::set_extra(KEY, self.extra_value());
        }
        // Angezeigter Stand (bei der Landung nachgefuehrt) - gespeichert wird beim Abwurf
        self.hist_shown = self.hist.clone();
        self.fairlog_shown = self.fairlog.clone();
        self.stats_shown = self.stats.clone();
        self.hits_shown = self.hits.clone();
        self.balls.clear();
        self.pending = 0;
        self.shown_chips = lamabank::balance() as f64;
        self.b.score = lamabank::score_for(KEY);
        self.rec_shown = self.b.score;
        self.shown_ledger = lamabank::ledger(KEY);
        self.sess_balls = 0;
        self.sess_net = 0;
        self.sess_best = 0;
        self.auto = None;
        self.panel = None;
        self.auto_focus = 0;
        self.stats_rows = self.rows;
        self.stats_hover = None;
        self.fair_sel = 0;
        self.fair_scroll = 0;
        self.fair_checks.clear();
        self.hold = false;
        self.hold_t = 0.0;
        self.fire_cd = 0.0;
        self.auto_cd = 0.0;
        self.tick_cd = 0.0;
        self.nod = 0.0;
        self.particles.clear();
        self.coins_fx.clear();
        self.popups.clear();
        self.banner = None;
        self.toast = None;
        self.shake = 0.0;
        self.hist_anim = 1.0;
        self.time = 0.0;
        self.mouse = (-1, -1);
        self.broke = false;
        self.chip_tick_cd = 0.0;
        self.tick_n = 0;
        self.lang = i18n::get_language();
        self.layout();
        self.new_board_fx();
        self.check_broke();
        if !self.broke {
            self.say(t("lpl.hint"), 3.6);
        }
    }

    fn on_surface_changed(&mut self) {
        d::clear_cache();
        self.layout();
    }

    fn on_exit(&mut self) {
        // Verlassen: Erfolge noch fliegender Kugeln nachholen (gebucht ist schon alles)
        self.flush_flying();
        self.b.score = lamabank::score_for(KEY);
    }

    fn wants_escape(&self) -> bool {
        self.panel.is_some() || self.auto.is_some()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        let drop_key = ["space", "Return", "KP_Enter"].contains(&ev.key.as_str());
        if matches!(kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            self.mouse = ev.pos;
        }
        if kind == Kind::KeyUp && drop_key {
            self.hold = false;
            return;
        }
        if kind == Kind::MouseUp {
            self.hold = false;
            return;
        }
        if self.broke {
            if kind == Kind::MouseDown || (kind == Kind::KeyDown && drop_key) {
                self.take_credit();
            }
            return;
        }
        if self.panel.is_some() {
            self.panel_event(ev);
            return;
        }
        if kind == Kind::KeyDown {
            let key = ev.key.as_str();
            if drop_key {
                if ev.repeat {
                    if self.auto.is_none() && !self.hold {
                        self.hold = true;
                        self.hold_t = HOLD_DELAY;
                    }
                } else {
                    self.start_hold();
                }
            } else if key == "Escape" {
                self.stop_auto(None);
            } else if key == "Left" {
                self.set_rows(self.rows - 1);
            } else if key == "Right" {
                self.set_rows(self.rows + 1);
            } else if ["plus", "KP_Add", "Up", "equal"].contains(&key) {
                self.set_bet(1);
            } else if ["minus", "KP_Subtract", "Down"].contains(&key) {
                self.set_bet(-1);
            } else if ["1", "2", "3", "KP_1", "KP_2", "KP_3"].contains(&key) {
                let i = key.chars().last().and_then(|c| c.to_digit(10)).unwrap_or(1) as usize - 1;
                self.set_risk(i);
            } else if ev.repeat {
                return;
            } else if key == "a" || key == "A" {
                self.open(Panel::Auto);
            } else if key == "s" || key == "S" {
                self.open(Panel::Stats);
            } else if key == "f" || key == "F" {
                self.open(Panel::Fair);
            } else if key == "t" || key == "T" {
                self.toggle_turbo();
            }
            return;
        }
        if kind == Kind::Wheel {
            self.set_bet(if ev.delta > 0 { 1 } else { -1 });
            return;
        }
        if kind != Kind::MouseDown || ev.button != 1 {
            return;
        }
        let pos = ev.pos;
        if self.r_drop.hit(pos) {
            self.start_hold();
        } else if self.r_minus.hit(pos) {
            self.set_bet(-1);
        } else if self.r_plus.hit(pos) {
            self.set_bet(1);
        } else if self.r_rows_m.hit(pos) {
            self.set_rows(self.rows - 1);
        } else if self.r_rows_p.hit(pos) {
            self.set_rows(self.rows + 1);
        } else {
            for i in 0..self.r_risks.len() {
                if self.r_risks[i].hit(pos) {
                    self.set_risk(i);
                    return;
                }
            }
            let names = [Some(Panel::Auto), Some(Panel::Stats), Some(Panel::Fair), None];
            for (name, r) in names.iter().zip(self.r_icons.clone().iter()) {
                if r.hit(pos) {
                    match name {
                        None => self.toggle_turbo(),
                        Some(pn) => self.open(*pn),
                    }
                    return;
                }
            }
            if self.board_box.hit(pos) && pos.1 < self.slot_top {
                self.start_hold(); // Klick aufs Brett wirft ebenfalls
            }
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
            let step = pmax(diff.abs() * pmin(1.0, dt * 6.0), 60.0 * dt);
            self.shown_chips += pmin(diff.abs(), step).copysign(diff);
            if diff > 0.0 {
                self.chip_tick_cd -= dt;
                if self.chip_tick_cd <= 0.0 {
                    self.chip_tick_cd = 0.07;
                    self.tick_n += 1;
                    let freq = 1500.0 + py::pmod(self.tick_n * 113, 300) as f64;
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
        self.shake = pmax(0.0, self.shake - dt);
        self.nod = pmax(0.0, self.nod - dt);
        self.tick_cd = pmax(0.0, self.tick_cd - dt);
        self.hist_anim = pmin(1.0, self.hist_anim + dt / 0.22);
        if let Some(b) = self.banner.as_mut() {
            b.t += dt;
            if b.t >= b.dur {
                self.banner = None;
            }
        }
        for row in self.peg_flash.iter_mut() {
            for v in row.iter_mut() {
                if *v > 0.0 {
                    *v = pmax(0.0, *v - dt);
                }
            }
        }
        for v in self.slot_bump.iter_mut() {
            if *v > 0.0 {
                *v = pmax(0.0, *v - dt);
            }
        }
        self.update_fx(dt);
        self.update_balls(dt);
        // Dauerfeuer (gehaltene Leertaste bzw. Maus auf WERFEN)
        if self.hold && self.auto.is_none() && self.panel.is_none() {
            self.hold_t += dt;
            if self.hold_t >= HOLD_DELAY {
                // Rest aufheben statt neu setzen: der Takt stimmt bei jeder Bildrate
                self.fire_cd -= dt;
                let mut n = 0;
                while self.fire_cd <= 0.0 && n < 3 {
                    self.fire_cd += fire_rate(self.rows) / self.speed();
                    self.drop_ball(true);
                    n += 1;
                }
                if self.fire_cd <= 0.0 {
                    self.fire_cd = 0.0;
                }
            }
        }
        if self.auto.is_some() {
            self.auto_cd -= dt;
            if self.auto_cd <= 0.0 {
                self.auto_cd = AUTO_RATE / self.speed();
                self.auto_step();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        let glow_c = (self.cx as i32, self.region.y + py::fdiv32(self.region.h * 11, 20));
        let bg = d::background(w, h, glow_c, (w.max(h) as f64 * 0.42) as i32, self.b.accent);
        s.bl(&bg, (0, 0));
        let mut ox = 0;
        if self.shake > 0.0 {
            ox = ((self.time * 70.0).sin() * 6.0 * self.k * self.shake / 0.35) as i32;
        }
        self.draw_board(&mut s, ox);
        self.draw_llama(&mut s, ox);
        self.draw_balls(&mut s, ox);
        self.draw_particles(&mut s, ox);
        self.draw_left(&mut s);
        self.draw_right(&mut s);
        self.draw_hud(&mut s);
        self.draw_strip(&mut s);
        self.draw_fx(&mut s);
        if self.banner.is_some() {
            self.draw_banner(&mut s);
        }
        if self.panel.is_none() && !self.broke {
            self.draw_hover(&mut s);
        }
        match self.panel {
            Some(Panel::Auto) => self.draw_auto(&mut s),
            Some(Panel::Stats) => self.draw_stats(&mut s),
            Some(Panel::Fair) => self.draw_fair(&mut s),
            None => {}
        }
        // Hinweis UEBER den Fenstern (sonst bliebe z.B. "Neuer Client-Seed" unsichtbar)
        if self.toast.is_some() {
            self.draw_toast(&mut s);
        }
        if self.broke {
            self.draw_broke(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "lamaplinko",
    class_name: "LamaPlinkoGame",
    supports_multiplayer: false,
    modes: &[],
    make: LamaPlinkoGame::new,
};
