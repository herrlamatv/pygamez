//! lamacrash.rs
//! ============
//! Lama Crash - Mut-Spiel mit Lama-Chips, uebertragen aus
//! `games/lamacrash.py`. Ein Lama (bzw. zwei - je Einsatz eins) steigt im
//! Ballon auf, der Multiplikator waechst mit der Zeit. Wer aussteigt,
//! bekommt Einsatz x Multiplikator; platzt der Ballon vorher, ist der noch
//! fliegende Einsatz weg. Nach dem Aussteigen fliegt der leere Ballon
//! weiter, bis er platzt (Vorspulen moeglich).
//!
//! Regeln und Zahlen: [`crate::lamacrash_logic`], Bilder:
//! [`crate::lamacrash_draw`] (Ballon, Korb, Fallschirm ...) und
//! [`crate::lamacrash_world`] (Hoehenwelten), Musik:
//! [`crate::lamacrash_music`], Fairness/Statistik: `pgz_core::fairness`,
//! `pgz_core::chipstats`.
//!
//! * Zwei Einsaetze (Karte 2 per Schalter/Z), je mit eigenem Auto-Ziel;
//!   jeder Einsatz darf einmal halbiert werden.
//! * Hoehenwelten Wiese -> Wolken -> Gewitter -> Nordlicht -> Weltall ->
//!   Mondlandung bei x100.000.
//! * Fairness (F), Statistik (S), Auto-Modus (A), Turbo (T), Musik (M).
//!
//! Buchung wie in Python: SPIELEN bucht den Einsatz und speichert die Runde
//! in einem Zug (`lamabank::bet_and_pay`); jeder Ausstieg schreibt den
//! Gewinn gut und speichert die Runde. Wer mitten im Flug geht, macht beim
//! naechsten Start am selben Takt weiter. Der Platzpunkt (`self.crash`)
//! wird nur in `advance` (ueber `l::tick`) und beim Rundenabschluss gelesen.
//! Alle Tasten roh (ohne p1/p2-Belegung).
//!
//! Einstellungen: Rust prueft settings.json nicht gegen `settings.SCHEMAS`;
//! das Spiel bildet die Regeln fuer seinen eigenen Abschnitt nach (gleicher
//! Wert im Spiel wie Python) und ersetzt ungueltige Werte beim Start im
//! Speicher wie Python beim Laden - beim naechsten Speichern steht dann auch
//! in der Datei dasselbe.

use std::collections::HashMap;
use std::f64::consts::TAU;

use pgz_core::chipstats::{self, ChipStats};
use pgz_core::fairness::{self, LogEntry};
use pgz_core::{i18n, lamabank};
use pgz_engine::charts;
use pgz_engine::prelude::*;
use serde_json::{Map, Value};

use crate::casino_draw as cd;
use crate::lamacrash_draw as d;
use crate::lamacrash_logic as l;
use crate::lamacrash_music as mu;
use crate::lamacrash_world as w;

const KEY: &str = l::KEY;

/// 3-2-1 vor dem Start
const COUNT_T: f64 = 1.35;
/// Countdown beim Fortsetzen einer Runde
const RESUME_T: f64 = 1.8;
/// Ergebnis nach dem Platzen, danach Ruhezustand
const END_T: f64 = 2.8;
/// Ergebnis fruehestens danach wegdrueckbar
const READ_T: f64 = 0.35;
/// Auto: Ergebnis-Anzeige
const AUTO_END_T: f64 = 1.0;
/// Auto: Pause vor der naechsten Runde
const AUTO_PAUSE_T: f64 = 0.35;
/// Turbo/Auto: Countdown und Pausen x 0,4
const TURBO_F: f64 = 0.4;
/// Kamera-Wackeln nach dem Platzen
const SHAKE_T: f64 = 0.6;
/// ab x3: Muenzregen
const BIG_MULT: i64 = 300;
/// ab x10: Konfetti
const PARTY_MULT: i64 = 1000;
/// ab x100: Gewinn-Banner
const BANNER_MULT: i64 = 10000;

const ROUND_OPTS: [i64; 6] = [5, 10, 25, 50, 100, 250];
const LIMIT_OPTS: [i64; 7] = [0, 100, 500, 1000, 5000, 10000, 50000];
const TOOLS: [&str; 5] = ["auto", "stats", "fair", "turbo", "music"];
/// Halstuch-/Kartenfarbe je Einsatz
const CARD_COL: [(i32, i32, i32); 2] = [(255, 196, 72), (96, 198, 255)];
/// Fallschirm je Einsatz
const CHUTE_COL: [(i32, i32, i32); 2] = [(255, 140, 60), (70, 150, 255)];
/// beim Erreichen dieser Multiplikatoren ein kurzer Ton (steigend)
const TONE_AT: [i64; 9] = [200, 300, 500, 1000, 2000, 5000, 10000, 100000, 1000000];

/// Schluessel von `settings.SCHEMAS["lamacrash"]`.
const OPT_KEYS: [&str; 12] = [
    "bet1",
    "bet2",
    "two",
    "auto1",
    "target1",
    "auto2",
    "target2",
    "turbo",
    "music",
    "auto_rounds",
    "auto_win",
    "auto_loss",
];

const ELL: &str = "\u{2026}";
const DOT: &str = "  \u{00B7}  ";

fn tool_label(name: &str) -> &'static str {
    match name {
        "auto" => "chip.auto",
        "stats" => "chip.stats",
        "fair" => "chip.fair",
        "turbo" => "chip.turbo",
        _ => "lcx.music",
    }
}

fn tool_key(name: &str) -> &'static str {
    match name {
        "auto" => "A",
        "stats" => "S",
        "fair" => "F",
        "turbo" => "T",
        _ => "M",
    }
}

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

/// `isinstance(v, int) and not isinstance(v, bool)` fuer JSON-Werte.
fn py_int(v: Option<&Value>) -> Option<i64> {
    match v {
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

/// `_is_action(key)` - Leertaste/Enter (alle Tasten roh, wie Lama Mines).
fn is_action(key: &str) -> bool {
    key == "space" || key == "Return" || key == "KP_Enter"
}

/// `key.lower() if len(key) == 1 else key`
fn low_key(key: &str) -> String {
    if key.chars().count() == 1 {
        key.to_lowercase()
    } else {
        key.to_string()
    }
}

/// `_wrap(text, font, maxw)` - Woerter an Leerzeichen (wie `str.split(" ")`).
fn wrap(text: &str, font: &Font, maxw: i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for wd in text.split(' ') {
        let test = if !cur.is_empty() { format!("{} {}", cur, wd) } else { wd.to_string() };
        if font.width(&test) <= maxw || cur.is_empty() {
            cur = test;
        } else {
            lines.push(std::mem::replace(&mut cur, wd.to_string()));
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// `_fit(text, fonts, maxw, col)` - Text in der ersten Schrift, die passt,
/// sonst gekuerzt mit "…".
fn fit(text: &str, fonts: &[&Font], maxw: i32, col: impl Col) -> Surface {
    for f in fonts {
        if f.width(text) <= maxw {
            return f.render(text, true, col);
        }
    }
    let f = fonts[fonts.len() - 1];
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() && f.width(&format!("{}{}", chars.iter().collect::<String>(), ELL)) > maxw {
        chars.pop();
    }
    f.render(&format!("{}{}", chars.iter().collect::<String>(), ELL), true, col)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum St {
    Idle,
    Count,
    Fly,
    Coast,
    End,
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

/// Eine Einsatz-Karte der Seitenleiste (`self.cards[i]`).
#[derive(Clone, Copy)]
struct Card {
    rect: Rect,
    title_y: i32,
    title_h: i32,
    sw: Rect,
    minus: Rect,
    bet: Rect,
    plus: Rect,
    auto: Rect,
    tl: Rect,
    tv: Rect,
    tr: Rect,
    cash: Rect,
    halfc: Rect,
    row_a: Rect,
    row_b: Rect,
}

/// Letzte fertige Runde (`self.last`).
#[derive(Clone)]
struct Last {
    rnd: l::Round,
    crash: i64,
    end: &'static str,
    stake: i64,
    paid: i64,
}

/// Lama bzw. Geldsack am Fallschirm.
struct Chute {
    llama: bool,
    slot: usize,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    t: f64,
    n: i32,
}

/// Purzelndes Lama nach dem Platzen.
struct Faller {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    t: f64,
    slot: usize,
}

/// Gummifetzen.
struct Shred {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    t: f64,
    life: f64,
    i: usize,
    w: i32,
}

/// Knall-Stern.
struct PopFx {
    x: i32,
    y: i32,
    t: f64,
    r: i32,
}

/// Fallender Korb nach dem Platzen.
struct BasketFx {
    x: f64,
    y: f64,
    vy: f64,
    t: f64,
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    r: f64,
    col: Color,
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
}

/// `[Titel, Untertitel, Restzeit, Dauer]`
struct Banner {
    title: String,
    sub: String,
    left: f64,
    dur: f64,
}

/// Laufender Auto-Modus (`{"left", "net", "t"}`).
struct AutoRun {
    left: i64,
    net: i64,
    t: f64,
}

/// `LamaCrashGame`
pub struct LamaCrashGame {
    b: GameBase,
    seed_rng: py::Random,
    rng: py::Random,
    bets: [i64; 2],
    autos: [bool; 2],
    targets: [i64; 2],
    two: bool,
    turbo: bool,
    music_on: bool,
    auto_rounds: i64,
    auto_win: i64,
    auto_loss: i64,
    hist: Vec<i64>,
    fair: fairness::State,
    fairlog: Vec<LogEntry>,
    stats: ChipStats,
    xs: l::XStats,
    carry: i64,
    round: Option<l::Round>,
    crash: Option<i64>,
    state: St,
    sel: usize,
    acc: f64,
    count_t: f64,
    count_len: f64,
    ff: bool,
    end: Option<&'static str>,
    end_t: f64,
    last: Option<Last>,
    tone_i: usize,
    world_seen: usize,
    world_label: Option<(String, f64)>,
    squeak_t: f64,
    flame_t: f64,
    env_rect: Rect,
    chutes: Vec<Chute>,
    fallers: Vec<Faller>,
    shreds: Vec<Shred>,
    pop_fx: Option<PopFx>,
    basket_fx: Option<BasketFx>,
    particles: Vec<Particle>,
    coins_fx: Vec<CoinFx>,
    popups: Vec<Popup>,
    toast: Option<(String, f64)>,
    banner: Option<Banner>,
    shake: f64,
    time: f64,
    mouse: (i32, i32),
    broke: bool,
    window: Option<Win>,
    auto: Option<AutoRun>,
    auto_focus: i32,
    fair_sel: Option<usize>,
    fair_scroll: i64,
    fair_checks: HashMap<i64, (bool, i64, Vec<i64>)>,
    r_fair_rows: Vec<(usize, Rect)>,
    r_fair_new: Rect,
    r_fair_verify: Rect,
    stats_hover: Option<usize>,
    mus_stage: Option<usize>,
    mus_t: f64,
    mus_vol: Option<f64>,
    mus_fail: Option<usize>,
    pending: i64,
    shown_chips: f64,
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
    fs_micro: i32,
    fs_tiny: i32,
    margin: i32,
    hud_h: i32,
    strip_h: i32,
    tool_h: i32,
    strip: Rect,
    stage: Rect,
    panel_r: Rect,
    hud_bg: Surface,
    strip_bg: Surface,
    overlay: Surface,
    bg: Surface,
    tool_labels: bool,
    r_tools: Vec<(&'static str, Rect)>,
    tools_right: i32,
    r_total: Rect,
    r_half: Rect,
    r_go: Rect,
    half_label: bool,
    status_x: (i32, i32),
    ppad: i32,
    panel_inner: (i32, i32),
    cards: Vec<Card>,
    info_y: i32,
    hint_y: i32,
    rig_base: i32,
    plot: Rect,
    auto_r: Rect,
    r_auto_close: Rect,
    auto_rows: Vec<(i32, Vec<Rect>)>,
    r_auto_turbo: Rect,
    auto_sum_y: i32,
    r_auto_start: Rect,
}

impl LamaCrashGame {
    /// `LamaCrashGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let r0 = Rect::new(0, 0, 0, 0);
        let mut g = LamaCrashGame {
            b,
            seed_rng: py::Random::new(0),
            rng: py::Random::new(0),
            bets: [l::BETS[0], l::BETS[0]],
            autos: [false, true],
            targets: [l::DEFAULT_TARGET, 1000],
            two: false,
            turbo: false,
            music_on: true,
            auto_rounds: 10,
            auto_win: 0,
            auto_loss: 0,
            hist: Vec::new(),
            fair: fairness::State { server: String::new(), client: String::new(), nonce: 0 },
            fairlog: Vec::new(),
            stats: chipstats::blank(),
            xs: l::blank_x(),
            carry: 0,
            round: None,
            crash: None,
            state: St::Idle,
            sel: 0,
            acc: 0.0,
            count_t: 0.0,
            count_len: COUNT_T,
            ff: false,
            end: None,
            end_t: 0.0,
            last: None,
            tone_i: 0,
            world_seen: 0,
            world_label: None,
            squeak_t: 0.0,
            flame_t: 0.0,
            env_rect: r0,
            chutes: Vec::new(),
            fallers: Vec::new(),
            shreds: Vec::new(),
            pop_fx: None,
            basket_fx: None,
            particles: Vec::new(),
            coins_fx: Vec::new(),
            popups: Vec::new(),
            toast: None,
            banner: None,
            shake: 0.0,
            time: 0.0,
            mouse: (-1, -1),
            broke: false,
            window: None,
            auto: None,
            auto_focus: 0,
            fair_sel: None,
            fair_scroll: 0,
            fair_checks: HashMap::new(),
            r_fair_rows: Vec::new(),
            r_fair_new: r0,
            r_fair_verify: r0,
            stats_hover: None,
            mus_stage: None,
            mus_t: 0.0,
            mus_vol: None,
            mus_fail: None,
            pending: 0,
            shown_chips: 0.0,
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
            fs_micro: 9,
            fs_tiny: 11,
            margin: 6,
            hud_h: 28,
            strip_h: 46,
            tool_h: 26,
            strip: r0,
            stage: r0,
            panel_r: r0,
            hud_bg: pg::surface_alpha(1, 1),
            strip_bg: pg::surface_alpha(1, 1),
            overlay: pg::surface_alpha(1, 1),
            bg: pg::surface(1, 1),
            tool_labels: false,
            r_tools: Vec::new(),
            tools_right: 0,
            r_total: r0,
            r_half: r0,
            r_go: r0,
            half_label: true,
            status_x: (0, 0),
            ppad: 5,
            panel_inner: (0, 0),
            cards: Vec::new(),
            info_y: 0,
            hint_y: 0,
            rig_base: 30,
            plot: r0,
            auto_r: r0,
            r_auto_close: r0,
            auto_rows: Vec::new(),
            r_auto_turbo: r0,
            auto_sum_y: 0,
            r_auto_start: r0,
        };
        g.reset();
        Box::new(g)
    }

    // ===================================================== Layout

    fn layout(&mut self) {
        let (ww, h) = (self.b.width, self.b.height);
        self.k = pmin(ww as f64 / 800.0, h as f64 / 600.0);
        let k = self.k;
        self.f_micro = ui::font(10.max((11.0 * k) as i32), false, false);
        self.f_tiny = ui::font(11.max((13.0 * k) as i32), false, false);
        self.f_small = ui::font(12.max((15.0 * k) as i32), false, false);
        self.f_btn = ui::font(12.max((15.0 * k) as i32), true, false);
        self.f_big = ui::font(15.max((22.0 * k) as i32), true, false);
        self.f_mult = ui::font(20.max((40.0 * k) as i32), true, false);
        self.f_huge = ui::font(26.max((58.0 * k) as i32), true, false);
        self.f_mono = ui::font(10.max((12.0 * k) as i32), false, true);
        // Schriftgroessen fuer gecachte Zeichen-Bausteine
        self.fs_micro = 9.max((11.0 * k) as i32);
        self.fs_tiny = 11.max((13.0 * k) as i32);
        self.margin = 6.max((12.0 * k) as i32);
        let m = self.margin;
        self.hud_h = 28.max((42.0 * k) as i32);
        self.strip_h = 46.max((64.0 * k) as i32);
        self.tool_h = 26.max((34.0 * k) as i32);
        self.strip = Rect::new(0, h - self.strip_h, ww, self.strip_h);
        let tool_y = self.hud_h + m;
        let area_y = tool_y + self.tool_h + m;
        let area_h = self.strip.y - m - area_y;
        let panel_w = 184.max(((ww as f64 * 0.38) as i32).min((300.0 * k) as i32));
        self.stage = Rect::new(m, area_y, ww - panel_w - 3 * m, area_h);
        self.panel_r = Rect::new(self.stage.right() + m, area_y, panel_w, area_h);
        self.hud_bg = pg::surface_alpha(ww, self.hud_h);
        for y in 0..self.hud_h {
            let a = (210.0 - (60 * y) as f64 / 1.max(self.hud_h - 1) as f64) as i32;
            self.hud_bg.fill_r(rgba(10, 8, 16, a), (0, y, ww, 1));
        }
        self.strip_bg = pg::surface_alpha(self.strip.w, self.strip.h);
        for y in 0..self.strip_h {
            let a = (170.0 + (60 * y) as f64 / 1.max(self.strip_h - 1) as f64) as i32;
            self.strip_bg.fill_r(rgba(10, 8, 16, a), (0, y, ww, 1));
        }
        self.overlay = pg::surface_alpha(ww, h);
        self.overlay.fill_c(rgba(6, 4, 12, 200));
        self.bg = pg::surface(ww, h);
        for y in 0..h {
            self.bg.fill_r(ui::mix((22, 16, 34), (8, 6, 14), y as f64 / 1.max(h - 1) as f64), (0, y, ww, 1));
        }
        self.layout_tools(tool_y);
        self.layout_strip();
        self.layout_cards();
        self.layout_stage();
        self.layout_auto();
    }

    /// Werkzeug-Knoepfe oben links (mit Beschriftung, wenn Platz ist).
    fn layout_tools(&mut self, y: i32) {
        let m = self.margin;
        let h = self.tool_h;
        let pad = 6.max((10.0 * self.k) as i32);
        let gap = 3.max((5.0 * self.k) as i32);
        let widths: Vec<i32> = TOOLS.iter().map(|name| h + pad + self.f_tiny.width(&t(tool_label(name)))).collect();
        let mut total = 0;
        for wd in &widths {
            total += wd + gap;
        }
        self.tool_labels = total <= (self.stage.w as f64 * 0.72) as i32;
        let mut x = m;
        self.r_tools.clear();
        for (i, name) in TOOLS.iter().enumerate() {
            let wd = if self.tool_labels { widths[i] } else { h };
            self.r_tools.push((name, Rect::new(x, y, wd, h)));
            x += wd + gap;
        }
        self.tools_right = x;
    }

    /// Bedienleiste: Gesamteinsatz, Haelfte-Knopf, Status, Hauptknopf.
    fn layout_strip(&mut self) {
        let (m, k, ww) = (self.margin, self.k, self.b.width);
        let sh = self.strip_h;
        let bh = 30.max((sh as f64 * 0.62) as i32);
        let by = self.strip.y + py::fdiv32(sh - bh, 2);
        let tw = self.f_tiny.width(&t("lcx.total")).max(self.f_btn.width("20.000")) + (16.0 * k) as i32;
        self.r_total = Rect::new(m, by, tw, bh);
        let hw = bh.max(bh + self.f_btn.width(&t("lcx.half_all")) + (14.0 * k) as i32);
        self.r_half = Rect::new(self.r_total.right() + 4.max((8.0 * k) as i32), by, hw, bh);
        let pad = (22.0 * pmax(0.7, k)) as i32;
        let labels = [t("lcx.play"), t("lcx.cash_all") + " 9.999.999", t("lcx.ff"), t("chip.auto_stop")];
        let mut go_w = 0;
        for lab in labels.iter() {
            go_w = go_w.max(self.f_btn.width(lab) + pad);
        }
        go_w = go_w.max((130.0 * k) as i32).min((ww as f64 * 0.45) as i32);
        self.r_go = Rect::new(ww - m - go_w, by, go_w, bh);
        let room = self.r_go.x - m - self.r_half.right() - m;
        self.half_label = room >= 0;
        if room < 0 {
            // zu eng: Haelfte-Knopf nur mit Symbol
            self.r_half.w = bh;
        }
        self.status_x = (self.r_half.right() + m, self.r_go.x - m);
    }

    /// Seitenleiste: zwei Einsatz-Karten, Info-Kasten, Hinweis.
    fn layout_cards(&mut self) {
        let p = self.panel_r;
        let k = self.k;
        self.ppad = 5.max((8.0 * k) as i32);
        let pad = self.ppad;
        let (x0, wd) = (p.x + pad, p.w - 2 * pad);
        self.panel_inner = (x0, wd);
        let rh = 22.max((28.0 * k) as i32);
        let gap = 4.max((6.0 * k) as i32);
        let cpad = 4.max((6.0 * k) as i32);
        let title_h = self.f_tiny.get_height() + 2.max((4.0 * k) as i32);
        let card_h = 2 * cpad + title_h + 2 * rh + gap;
        self.cards.clear();
        let mut y = p.y + pad;
        for _ in 0..2 {
            let c = Rect::new(x0, y, wd, card_h);
            let iw = wd - 2 * cpad;
            let ix = c.x + cpad;
            let ty = c.y + cpad;
            let ya = ty + title_h;
            let yb = ya + rh + gap;
            let sw_w = ((rh as f64 * 1.6) as i32).max(30);
            let vw = iw - 2 * rh - 8;
            let tv = iw - 3 * rh - 12;
            let cash_w = iw - rh - 4;
            self.cards.push(Card {
                rect: c,
                title_y: ty,
                title_h,
                sw: Rect::new(c.right() - cpad - sw_w, ty, sw_w, title_h - 2),
                minus: Rect::new(ix, ya, rh, rh),
                bet: Rect::new(ix + rh + 4, ya, vw, rh),
                plus: Rect::new(ix + rh + 4 + vw + 4, ya, rh, rh),
                auto: Rect::new(ix, yb, rh, rh),
                tl: Rect::new(ix + rh + 4, yb, rh, rh),
                tv: Rect::new(ix + 2 * rh + 8, yb, tv, rh),
                tr: Rect::new(ix + 2 * rh + 8 + tv + 4, yb, rh, rh),
                cash: Rect::new(ix, yb, cash_w, rh),
                halfc: Rect::new(ix + cash_w + 4, yb, rh, rh),
                row_a: Rect::new(ix, ya, iw, rh),
                row_b: Rect::new(ix, yb, iw, rh),
            });
            y = c.bottom() + gap;
        }
        self.info_y = y;
        let hint_h = 2 * self.f_micro.get_height() + 2;
        self.hint_y = p.bottom() - pad - hint_h;
        if self.hint_y - self.info_y < 3 * (self.f_tiny.get_height() + 2) {
            self.hint_y = p.bottom() - pad; // zu eng: kein Hinweis
        }
    }

    /// Buehne: Zeichenflaeche der Kurve (Platz fuer den Ballon ueber der Spitze).
    fn layout_stage(&mut self) {
        let st = self.stage;
        self.rig_base = 30.max((st.h as f64 * 0.19) as i32);
        let base = self.rig_base;
        let (bw, bh) = d::balloon_size(base, d::INFL_MAX);
        let basket_h = py::fdiv32(d::basket_w(base) * 2, 3);
        // Start auf dem Startplatz der Wiese (lamacrash_world: Mitte bei 15 % der
        // Breite, Oberkante max(16, 5 % der Hoehe) ueber dem Buehnenrand)
        let left = (py::fdiv32(bw, 2) + 4).max(self.f_micro.width("\u{00D7}1.000") + 10).max(py::fdiv32(st.w * 15, 100));
        let right = (py::fdiv32(bw, 2) + 4).max((10.0 * self.k) as i32);
        let top = (bh as f64 * 0.92) as i32;
        let bottom = basket_h + 16.max(py::fdiv32(st.h * 5, 100));
        self.plot = Rect::new(st.x + left, st.y + top, 20.max(st.w - left - right), 20.max(st.h - top - bottom));
    }

    /// Auto-Fenster ueber der Buehne.
    fn layout_auto(&mut self) {
        let r = self.stage.inflate(-self.margin, -self.margin);
        self.auto_r = r;
        let k = self.k;
        let pad = 6.max((10.0 * k) as i32);
        let cb = 18.max((24.0 * k) as i32);
        self.r_auto_close = Rect::new(r.right() - pad - cb, r.y + pad, cb, cb);
        let (x0, wd) = (r.x + pad, r.w - 2 * pad);
        let mut y = r.y + pad + cb.max(self.f_btn.get_height()) + 3.max((6.0 * k) as i32);
        let lab = self.f_tiny.get_height();
        let rh = 20.max((26.0 * k) as i32);
        let gap = 3.max((5.0 * k) as i32);
        let mut rows = Vec::new();
        let lists: [&[i64]; 3] = [&ROUND_OPTS, &LIMIT_OPTS, &LIMIT_OPTS];
        for opts in lists.iter() {
            let ly = y;
            y += lab + 2;
            let n = opts.len() as i32;
            let mut first = 0;
            if opts[0] == 0 {
                first = 0.max(
                    self.f_tiny.width(&t("chip.auto_off")).min(self.f_micro.width(&t("chip.auto_off")) + 4) + 8,
                );
            }
            let bw = py::fdiv32(wd - first - (n - 1) * 3, if first == 0 { n } else { n - 1 });
            let mut rects = Vec::new();
            let mut x = x0;
            for i in 0..n {
                let rw = if i == 0 && first != 0 { first } else { bw };
                rects.push(Rect::new(x, y, rw, rh));
                x += rw + 3;
            }
            rows.push((ly, rects));
            y += rh + gap;
        }
        self.auto_rows = rows;
        let tw = wd.min(((wd as f64 * 0.5) as i32).max(self.f_btn.width(&t("chip.turbo")) + 3 * rh + 10));
        self.r_auto_turbo = Rect::new(x0, y, tw, rh);
        y += rh + gap;
        self.auto_sum_y = y;
        let bh = 26.max((34.0 * k) as i32);
        let sw = wd.min((220.0 * k) as i32);
        self.r_auto_start = Rect::new(r.centerx() - py::fdiv32(sw, 2), r.bottom() - pad - bh, sw, bh);
    }

    // ===================================================== Einstellungen

    /// Vorgabe aus `settings.DEFAULTS["lamacrash"]`.
    fn opt_default(key: &str) -> Value {
        match key {
            "bet1" | "bet2" => Value::from(10),
            "two" | "auto1" | "turbo" => Value::Bool(false),
            "auto2" | "music" => Value::Bool(true),
            "target1" => Value::from(200),
            "target2" => Value::from(1000),
            "auto_rounds" => Value::from(10),
            _ => Value::from(0),
        }
    }

    /// Wert wie Python ihn sieht: `settings._apply_schemas` laesst nur
    /// gueltige Werte durch (sonst die Vorgabe), int-Bereiche werden begrenzt.
    fn opt_py(&self, key: &str) -> Value {
        let default = Self::opt_default(key);
        let Some(v) = self.b.opt_value(KEY, key) else {
            return default;
        };
        let ok = match key {
            "bet1" | "bet2" => py_int(Some(&v)).is_some_and(|x| l::BETS.contains(&x)).then(|| v.clone()),
            "two" | "auto1" | "auto2" | "turbo" | "music" => v.is_boolean().then(|| v.clone()),
            "target1" | "target2" => py_int(Some(&v)).map(|x| Value::from(x.clamp(101, 10_000_000))),
            "auto_rounds" => py_int(Some(&v)).is_some_and(|x| ROUND_OPTS.contains(&x)).then(|| v.clone()),
            _ => py_int(Some(&v)).is_some_and(|x| LIMIT_OPTS.contains(&x)).then(|| v.clone()),
        };
        ok.unwrap_or(default)
    }

    /// Wie `settings._apply_schemas` beim Laden: ungueltige Werte des eigenen
    /// Abschnitts im Speicher durch das ersetzen, was Python sieht (Vorgabe
    /// bzw. begrenzte Zahl) - Python schreibt beim naechsten Speichern genau
    /// diese Werte in die Datei. Gespeichert wird hier nichts.
    fn normalize_opts(&mut self) {
        for key in OPT_KEYS {
            let Some(raw) = self.b.opt_value(KEY, key) else {
                continue;
            };
            let v = self.opt_py(key);
            if raw != v {
                self.b.settings.set_game(KEY, key, v.clone());
                shell::edit_settings(|s| s.set_game(KEY, key, v));
            }
        }
    }

    /// `_opt(key, default) is True`
    fn opt_true(&self, key: &str) -> bool {
        matches!(self.opt_py(key), Value::Bool(true))
    }

    /// `_choice_opt(key, opts, default)`
    fn choice_opt(&self, key: &str, opts: &[i64], default: i64) -> i64 {
        match py_int(Some(&self.opt_py(key))) {
            Some(v) if opts.contains(&v) => v,
            _ => default,
        }
    }

    /// `_target_opt(key, default)`
    fn target_opt(&self, key: &str, default: i64) -> i64 {
        match py_int(Some(&self.opt_py(key))) {
            Some(v) if l::target_ok(v) => v,
            _ => default,
        }
    }

    /// `_set_opt(key, value)` - nur bei Aenderung speichern.
    fn set_opt(&mut self, key: &str, value: Value) {
        if !py_eq(&self.opt_py(key), &value) {
            self.b.set_opt(KEY, key, value);
        }
    }

    // ===================================================== Konto / Speichern

    /// `_extra()`
    fn extra_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("round".into(), self.round.as_ref().map(|r| r.to_value()).unwrap_or(Value::Null));
        m.insert("hist".into(), Value::from(self.hist.clone()));
        m.insert("fair".into(), self.fair.to_value());
        m.insert("fairlog".into(), fairness::log_to_value(&self.fairlog));
        m.insert("stats".into(), self.stats.to_value());
        m.insert("carry".into(), Value::from(self.carry));
        Value::Object(m)
    }

    fn save_extra(&self) {
        lamabank::set_extra(KEY, self.extra_value());
    }

    /// Indizes der aktiven Einsatz-Karten (Karte 1 immer, Karte 2 per Schalter).
    fn active(&self) -> Vec<usize> {
        if self.two {
            vec![0, 1]
        } else {
            vec![0]
        }
    }

    fn total(&self) -> i64 {
        let mut total = 0;
        for i in self.active() {
            total += self.bets[i];
        }
        total
    }

    /// Einsaetze auf das Bezahlbare senken - zuerst Karte 2, dann Karte 1.
    fn clamp_bets(&mut self) {
        let chips = lamabank::balance();
        let order: Vec<usize> = if self.two { vec![1, 0] } else { vec![0] };
        for i in order {
            let others = self.total() - self.bets[i];
            while self.bets[i] > l::BETS[0] && others + self.bets[i] > chips {
                let j = l::BETS.iter().position(|v| *v == self.bets[i]).unwrap_or(1);
                self.bets[i] = l::BETS[j - 1];
            }
        }
    }

    fn check_broke(&mut self) {
        self.broke =
            self.state == St::Idle && self.round.is_none() && self.auto.is_none() && lamabank::is_broke(KEY, None);
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

    fn say(&mut self, text: String, secs: f64) {
        self.toast = Some((text, secs));
    }

    /// Zeitfaktor fuer Countdown und Pausen (Turbo/Auto schneller).
    fn tf(&self) -> f64 {
        if self.turbo || self.auto.is_some() {
            TURBO_F
        } else {
            1.0
        }
    }

    // ===================================================== Formatierung

    fn sep(&self) -> String {
        t("lcx.decimal")
    }

    /// Ganze Zahl mit Tausenderpunkten (bzw. -kommas, je Sprache).
    fn group(&self, n: i64) -> String {
        let sep = if self.sep() == "," { "." } else { "," };
        let mut s = n.unsigned_abs().to_string();
        let mut out = String::new();
        while s.len() > 3 {
            out = format!("{}{}{}", sep, &s[s.len() - 3..], out);
            s.truncate(s.len() - 3);
        }
        format!("{}{}{}", if n < 0 { "-" } else { "" }, s, out)
    }

    /// Multiplikator x100 als Text: x1.234,56 (ab x10.000 ohne Nachkommastellen).
    fn fmt(&self, m100: i64) -> String {
        if m100 >= 1000000 {
            return format!("\u{00D7}{}", self.group(py::fdiv(m100, 100)));
        }
        format!("\u{00D7}{}{}{:02}", self.group(py::fdiv(m100, 100)), self.sep(), py::pmod(m100, 100))
    }

    fn fmt_short(&self, m100: i64) -> String {
        format!("\u{00D7}{}", l::fmt_short(m100, &self.sep()))
    }

    /// Farbe des grossen Multiplikators: weiss -> gold -> orange -> rot -> violett.
    fn mult_col(m100: i64) -> Color {
        let stops: [(i64, (i32, i32, i32)); 5] = [
            (100, (240, 244, 255)),
            (200, (255, 226, 120)),
            (1000, (255, 168, 60)),
            (10000, (255, 84, 96)),
            (100000, (214, 120, 255)),
        ];
        if m100 <= stops[0].0 {
            return stops[0].1.c();
        }
        for wn in stops.windows(2) {
            let ((a, ca), (b, cb)) = (wn[0], wn[1]);
            if m100 <= b {
                return ui::mix(ca, cb, (m100 - a) as f64 / (b - a) as f64);
            }
        }
        stops[4].1.c()
    }

    // ===================================================== Kurve / Hoehe

    /// Hoehe als gebrochener Takt (fuer Bild und Welt; Geld zaehlt nur ganze Takte).
    fn alt(&self) -> f64 {
        if matches!(self.state, St::Fly | St::Coast) {
            if let Some(r) = self.round.as_ref() {
                return r.t as f64 + self.acc;
            }
        }
        if self.state == St::Count {
            if let Some(r) = self.round.as_ref() {
                return r.t as f64;
            }
        }
        if self.state == St::End {
            if let Some(last) = self.last.as_ref() {
                return last.rnd.t as f64;
            }
        }
        0.0
    }

    /// Multiplikator x100 zu einem gebrochenen Takt (nur fuers Zeichnen).
    fn m_at_f(tf: f64) -> f64 {
        let i = tf as i64;
        let a = l::mult_at(i);
        let b = l::mult_at(i + 1);
        a as f64 + (b - a) as f64 * (tf - i as f64)
    }

    /// Angezeigter Multiplikator (ganzer Takt; nach dem Platzen der Platzpunkt).
    fn cur_mult(&self) -> i64 {
        if matches!(self.state, St::Fly | St::Coast | St::Count) {
            if let Some(r) = self.round.as_ref() {
                return l::mult_at(r.t);
            }
        }
        if self.state == St::End {
            if let Some(last) = self.last.as_ref() {
                return if last.end == "pop" { last.crash } else { l::MAX_MULT100 };
            }
        }
        100
    }

    fn tone_index(m100: i64) -> usize {
        let mut i = 0;
        while i < TONE_AT.len() && m100 >= TONE_AT[i] {
            i += 1;
        }
        i
    }

    // ===================================================== Ablauf

    /// SPIELEN: Einsaetze abbuchen, Runde speichern, Countdown starten.
    fn play(&mut self) -> bool {
        if self.state == St::End && self.end_t >= READ_T {
            self.to_idle();
        }
        if self.state != St::Idle || self.broke || self.round.is_some() {
            return false;
        }
        let forced = self.auto.is_some();
        let mut pairs = Vec::new();
        for i in self.active() {
            let a = if self.autos[i] || forced { self.targets[i] } else { 0 };
            pairs.push((self.bets[i], a));
        }
        let total = self.total();
        if total > lamabank::balance() {
            self.say(t("cas.not_enough"), 1.8);
            self.b.play_sound("hit");
            return false;
        }
        let entry = fairness::begin(&self.fair);
        {
            let rng = &mut self.seed_rng;
            fairness::advance(&mut self.fair, &mut |k| rng.getrandbits(k));
        }
        self.round = Some(l::new_round(&entry, &pairs));
        if !lamabank::bet_and_pay(total, 0, KEY, Some((KEY, self.extra_value()))) {
            self.round = None;
            return false;
        }
        self.crash = Some(l::crash_point(&entry.s, &entry.c, entry.n));
        self.state = St::Count;
        self.count_t = COUNT_T * self.tf();
        self.count_len = self.count_t;
        self.acc = 0.0;
        self.ff = false;
        self.tone_i = Self::tone_index(100);
        self.world_seen = 0;
        self.world_label = None;
        self.squeak_t = 0.0;
        self.last = None;
        self.end = None;
        self.chutes.clear();
        self.fallers.clear();
        self.shreds.clear();
        self.pop_fx = None;
        self.basket_fx = None;
        self.window = None;
        self.b.score = lamabank::score_for(KEY);
        self.b.play_sound("select");
        true
    }

    /// Countdown vorbei: Brenner-Stoss, der Ballon hebt ab.
    fn launch(&mut self) {
        self.state = St::Fly;
        self.acc = 0.0;
        self.flame_t = 0.9;
        audio::tone(150.0, 0.45, Some(&self.b.settings), "noise", 0.16);
        audio::tone(1320.0, 0.1, Some(&self.b.settings), "sine", 0.25);
    }

    /// Takte abarbeiten (Takt-Regel in l::tick) - je Takt Auszahlungen und Ende.
    fn advance(&mut self, dt: f64) {
        let speed = if self.state == St::Coast && (self.ff || self.auto.is_some()) {
            l::SPEED_FF
        } else if self.turbo {
            l::SPEED_TURBO
        } else {
            1
        };
        let dt = pmin(dt, 0.1); // ein Ruckler wird nicht nachgeholt (wie main.py)
        self.acc += dt * 60.0 * speed as f64;
        let n = (self.acc + 1e-9) as i64;
        self.acc = pmax(0.0, self.acc - n as f64);
        for _ in 0..n {
            let crash = self.crash.unwrap_or(100);
            let Some(rnd) = self.round.as_mut() else {
                return;
            };
            let (events, carry, end) = l::tick(rnd, crash, self.carry);
            self.carry = carry;
            for ev in events.iter() {
                self.paid(ev.0, ev.1, ev.2, ev.3);
            }
            if let Some(end) = end {
                self.finish(end, false);
                return;
            }
            if !events.is_empty() {
                self.after_pay();
            }
        }
    }

    /// Landeplatz des Korbs auf dem Mond (Korbrand-Mitte).
    fn moon_rig(&self) -> (i32, i32) {
        (self.stage.centerx(), self.stage.bottom() - (self.stage.h as f64 * 0.16) as i32)
    }

    /// Korbrand-Mitte des Ballons zur aktuellen Hoehe (Bildschirm).
    fn rig_pos(&self) -> (f64, f64) {
        let tf = self.alt();
        if tf <= 0.0 {
            return (self.plot.x as f64, self.plot.bottom() as f64);
        }
        let (x_max, y_max) = l::chart_span(tf as i64);
        self.pt(tf, Self::m_at_f(tf), x_max, y_max)
    }

    fn pt(&self, tf: f64, m100: f64, x_max: i64, y_max: i64) -> (f64, f64) {
        let p = self.plot;
        let x = p.x as f64 + p.w as f64 * pmin(1.0, tf / 1.max(x_max) as f64);
        let y = p.bottom() as f64 - p.h as f64 * (m100 - 100.0) / 1.max(y_max - 100) as f64;
        (x, y)
    }

    fn n_riders(&self) -> usize {
        if let Some(r) = self.round.as_ref() {
            return r.bets.len();
        }
        if let Some(last) = self.last.as_ref() {
            if self.state == St::End {
                return last.rnd.bets.len();
            }
        }
        self.active().len()
    }

    /// Platz eines Lamas im Korb (links/rechts) relativ zur Korbmitte.
    fn slot_x(&self, i: usize) -> i32 {
        if self.n_riders() < 2 {
            return 0;
        }
        let bw = d::basket_w(self.rig_base);
        if i == 0 {
            py::fdiv32(-bw, 4)
        } else {
            py::fdiv32(bw, 4)
        }
    }

    /// Auto-Ziel bzw. Mondlandung: gutschreiben (gespeichert wird danach einmal).
    fn paid(&mut self, i: usize, kind: &str, m100: i64, win: i64) {
        lamabank::credit(win, KEY, false);
        if kind == "auto" {
            self.jump(i, true, win);
            self.celebrate(m100);
        } else {
            let (x, y) = self.moon_rig();
            let px = (x + self.slot_x(i)) as f64;
            let py_ = y as f64 - self.rig_base as f64 * 0.9 - (i as i32 * self.f_big.get_height()) as f64;
            let txt = format!("+{}", self.group(win));
            self.popup(px, py_, txt, cd::COL_GOLD.c(), 2.4);
        }
        self.cash_ach(m100);
    }

    /// Nach Auszahlungen im Flug: einmal speichern, ggf. in den Leerflug.
    fn after_pay(&mut self) {
        self.save_extra();
        self.b.score = lamabank::score_for(KEY);
        if self.state == St::Fly && !self.round.as_ref().is_some_and(l::flying) {
            self.state = St::Coast;
            self.ff = false;
        }
    }

    /// Erfolge beim Aussteigen sofort (verraet nichts, geht beim Verlassen nicht verloren).
    fn cash_ach(&self, m100: i64) {
        if m100 >= l::BRAVE_MULT {
            self.b.ach_event("lcx_brave", None);
        }
        if m100 >= l::SPACE_MULT {
            self.b.ach_event("lcx_space", None);
        }
    }

    /// Lama springt ab (bzw. Geldsack fliegt) - mit Fallschirm, Popup, Ton.
    fn jump(&mut self, i: usize, llama: bool, win: i64) {
        let (x, y) = self.rig_pos();
        let n = d::llama_size(self.rig_base);
        let sx = x + self.slot_x(i) as f64;
        let size = if llama { n } else { 8.max(py::fdiv32(n * 2, 3)) };
        self.chutes.push(Chute {
            llama,
            slot: i,
            x: sx,
            y: y + n as f64 * 0.15,
            vx: -((40 + 30 * i as i64) as f64) * self.k,
            vy: -150.0 * self.k,
            t: 0.0,
            n: size,
        });
        let txt = format!("+{}", self.group(win));
        self.popup(sx, y - n as f64, txt, cd::COL_GOLD.c(), 1.4);
        self.b.play_sound(if llama { "win" } else { "merge" });
        audio::tone(660.0, 0.07, Some(&self.b.settings), "sine", 0.22);
    }

    /// Von Hand aussteigen: parts = [(Einsatz-Index, Haelfte?)] - ein Speichern.
    fn cash_parts(&mut self, parts: &[(usize, bool)]) -> bool {
        if self.state != St::Fly {
            return false;
        }
        let Some(t_now) = self.round.as_ref().map(|r| r.t) else {
            return false;
        };
        if l::mult_at(t_now) < l::MIN_CASH {
            self.say(t("lcx.need_cash"), 1.2);
            return false;
        }
        let mut done = false;
        let mut best = 0;
        for &(i, half) in parts {
            let carry = self.carry;
            let Some(rnd) = self.round.as_mut() else {
                break;
            };
            let Some((win, m100, carry)) = l::cash_out(rnd, i, half, carry) else {
                continue;
            };
            self.carry = carry;
            lamabank::credit(win, KEY, false);
            self.jump(i, !half, win);
            self.cash_ach(m100);
            best = best.max(m100);
            done = true;
        }
        if done {
            self.after_pay();
            self.celebrate(best);
        }
        done
    }

    fn cash_all(&mut self) {
        let Some(rnd) = self.round.as_ref() else {
            return;
        };
        let parts: Vec<(usize, bool)> =
            rnd.bets.iter().enumerate().filter(|(_, b)| l::riding(b) > 0).map(|(i, _)| (i, false)).collect();
        if !parts.is_empty() && !self.cash_parts(&parts) {
            self.b.play_sound("hit");
        }
    }

    fn cash_one(&mut self, i: usize, half: bool) {
        let Some(rnd) = self.round.as_ref() else {
            return;
        };
        if i >= rnd.bets.len() {
            return;
        }
        let b = rnd.bets[i].clone();
        if l::riding(&b) <= 0 {
            return;
        }
        if half && b.hm != 0 {
            self.say(t("lcx.half_done"), 1.2);
            self.b.play_sound("hit");
            return;
        }
        if !self.cash_parts(&[(i, half)]) {
            self.b.play_sound("hit");
        }
    }

    fn half_all(&mut self) {
        let Some(rnd) = self.round.as_ref() else {
            return;
        };
        let parts: Vec<(usize, bool)> = rnd
            .bets
            .iter()
            .enumerate()
            .filter(|(_, b)| l::riding(b) > 0 && b.hm == 0)
            .map(|(i, _)| (i, true))
            .collect();
        if parts.is_empty() {
            self.say(t("lcx.half_done"), 1.2);
            self.b.play_sound("hit");
            return;
        }
        if !self.cash_parts(&parts) {
            self.b.play_sound("hit");
        }
    }

    /// Rundenende verbuchen: Zaehler, Statistik, Verlauf, Fairness-Log - einmal.
    fn finish(&mut self, end: &'static str, quiet: bool) {
        let Some(rnd) = self.round.clone() else {
            return;
        };
        let crash = self.crash.unwrap_or(100);
        l::record_x(&mut self.xs, &rnd, crash, end);
        self.stats.x = self.xs.to_map();
        let (stake, paid) = (l::stake_total(&rnd), l::paid_total(&rnd));
        chipstats::record(&mut self.stats, stake, paid, l::best_cash(&rnd));
        let net = paid - stake;
        let mut info = info_map(vec![("bet", stake.into()), ("win", paid.into()), ("cr", crash.into())]);
        if l::best_cash(&rnd) != 0 {
            info.insert("x".into(), l::best_cash(&rnd).into());
        }
        self.b.log_match(net, chip_result(net), Some(info), None, 0);
        self.hist.push(crash);
        if self.hist.len() > l::HISTORY {
            let cut = self.hist.len() - l::HISTORY;
            self.hist.drain(..cut);
        }
        fairness::record(&mut self.fairlog, &rnd.entry(), &l::outcome(&rnd, crash));
        let close = l::close_call(&rnd, crash);
        let lost: Vec<usize> = if end == "pop" {
            rnd.bets.iter().enumerate().filter(|(_, b)| l::riding(b) > 0).map(|(i, _)| i).collect()
        } else {
            Vec::new()
        };
        if let Some(a) = self.auto.as_mut() {
            a.net += paid - stake;
        }
        self.round = None;
        self.crash = None;
        self.last = Some(Last { rnd, crash, end, stake, paid });
        self.fair_sel = None;
        self.save_extra();
        self.b.score = lamabank::score_for(KEY);
        if close {
            self.b.ach_event("lcx_close", None);
        }
        if quiet {
            self.state = St::Idle;
            return;
        }
        self.state = St::End;
        self.end = Some(end);
        self.end_t = 0.0;
        if end == "pop" {
            self.pop(&lost);
        } else {
            self.moon();
        }
        if close {
            self.say(t("lcx.close"), 2.0);
        }
    }

    /// Runde ohne fliegendes Geld sofort abschliessen (Verlassen im Leerflug, Laden).
    fn finish_now(&mut self) {
        let crash = self.crash.unwrap_or(100);
        let Some(rnd) = self.round.as_mut() else {
            return;
        };
        let end = if crash >= l::MAX_MULT100 { "moon" } else { "pop" };
        rnd.t = l::pop_tick(crash);
        self.finish(end, true);
    }

    /// Der Ballon platzt: Knall, Fetzen, Wackeln, purzelnde Lamas, fallender Korb.
    fn pop(&mut self, lost: &[usize]) {
        let e = if self.env_rect.w > 0 { self.env_rect } else { Rect::new(self.plot.centerx(), self.plot.y, 40, 40) };
        self.pop_fx = Some(PopFx {
            x: e.centerx(),
            y: e.centery(),
            t: 0.0,
            r: 12.max(py::fdiv32(e.w, 2) + (10.0 * self.k) as i32),
        });
        self.shake = SHAKE_T;
        let pieces = d::shreds(8.max(e.w), self.b.accent);
        for j in 0..18usize {
            let a = self.rng.uniform(0.0, TAU);
            let sp = self.rng.uniform(120.0, 420.0) * self.k;
            let vx = a.cos() * sp;
            let vy = a.sin() * sp - 80.0 * self.k;
            let life = self.rng.uniform(0.9, 1.6);
            self.shreds.push(Shred {
                x: e.centerx() as f64,
                y: e.centery() as f64,
                vx,
                vy,
                t: 0.0,
                life,
                i: j % pieces.len().max(1),
                w: 8.max(e.w),
            });
        }
        let (x, y) = self.rig_pos();
        let n = d::llama_size(self.rig_base);
        for &i in lost {
            self.fallers.push(Faller {
                x: x + self.slot_x(i) as f64,
                y: y + n as f64 * 0.15,
                vx: (if i == 0 { -60.0 } else { 60.0 }) * self.k,
                vy: -120.0 * self.k,
                t: 0.0,
                slot: i,
            });
        }
        self.basket_fx = Some(BasketFx { x, y, vy: -40.0 * self.k, t: 0.0 });
        self.b.play_sound("explode");
        self.b.rumble(320);
        if !lost.is_empty() {
            audio::tone(220.0, 0.35, Some(&self.b.settings), "saw", 0.18);
        }
    }

    /// Mondlandung: Hoechstgewinn - Fahne, Konfetti, Muenzregen.
    fn moon(&mut self) {
        self.banner = Some(Banner { title: t("lcx.moon"), sub: t("lcx.moon_sub"), left: 3.4, dur: 3.4 });
        ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 150);
        self.coin_shower(90);
        self.b.play_sound("level");
        self.b.rumble(300);
    }

    /// Jubel im Moment des Aussteigens (haengt nur vom Ausstieg ab, verraet
    /// nichts): Muenzregen ab x3, Konfetti ab x10, Gewinn-Banner ab x100.
    fn celebrate(&mut self, m100: i64) {
        if m100 >= BANNER_MULT {
            self.banner = Some(Banner { title: t("cas.s.big_win"), sub: self.fmt(m100), left: 2.6, dur: 2.6 });
            self.b.play_sound("level");
        }
        if m100 >= PARTY_MULT {
            ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 110);
            self.coin_shower(60);
            self.b.rumble(200);
        } else if m100 >= BIG_MULT {
            self.coin_shower(28);
        }
    }

    /// Ergebnis schliessen: Ruhezustand (Korb wieder auf dem Startplatz).
    fn to_idle(&mut self) {
        self.state = St::Idle;
        self.end_t = 0.0;
        self.pop_fx = None;
        self.basket_fx = None;
        self.fallers.clear();
        self.clamp_bets();
        self.check_broke();
    }

    // ----- Einstellungen aendern

    /// Wer im Ergebnis etwas verstellt, schliesst es (Karten zeigen wieder die Einstellungen).
    fn leave_result(&mut self) {
        if self.state == St::End && self.round.is_none() {
            self.to_idle();
        }
    }

    fn can_edit(&self) -> bool {
        matches!(self.state, St::Idle | St::End) && self.round.is_none() && self.auto.is_none()
    }

    fn set_bet(&mut self, i: usize, direction: i64) {
        if !self.can_edit() || !self.active().contains(&i) {
            return;
        }
        self.leave_result();
        let j = l::BETS.iter().position(|v| *v == self.bets[i]).unwrap_or(0) as i64;
        let k = 0.max((l::BETS.len() as i64 - 1).min(j + direction));
        if k != j {
            self.bets[i] = l::BETS[k as usize];
            self.set_opt(&format!("bet{}", i + 1), Value::from(self.bets[i]));
            self.b.play_sound("select");
            if self.total() > lamabank::balance() {
                self.say(t("cas.not_enough"), 1.2);
            }
        }
    }

    fn set_target(&mut self, i: usize, direction: i64) {
        if !self.can_edit() || !self.active().contains(&i) {
            return;
        }
        self.leave_result();
        let v = l::step_target(self.targets[i], direction);
        if v != self.targets[i] {
            self.targets[i] = v;
            self.set_opt(&format!("target{}", i + 1), Value::from(v));
            self.b.play_sound("click");
        }
        if !self.autos[i] {
            self.autos[i] = true;
            self.set_opt(&format!("auto{}", i + 1), Value::Bool(true));
        }
    }

    fn toggle_target(&mut self, i: usize) {
        if !self.can_edit() || !self.active().contains(&i) {
            return;
        }
        self.leave_result();
        self.autos[i] = !self.autos[i];
        let v = self.autos[i];
        self.set_opt(&format!("auto{}", i + 1), Value::Bool(v));
        self.b.play_sound("click");
    }

    fn toggle_two(&mut self) {
        if !self.can_edit() {
            self.say(t("lcx.locked"), 1.2);
            return;
        }
        self.leave_result();
        self.two = !self.two;
        let v = self.two;
        self.set_opt("two", Value::Bool(v));
        if !self.two && self.sel == 1 {
            self.sel = 0;
        }
        self.clamp_bets();
        self.say(if self.two { t("lcx.two_on") } else { t("lcx.two_off") }, 1.0);
        self.b.play_sound("click");
    }

    fn select(&mut self, i: usize) {
        if i == 1 && !self.two {
            return;
        }
        self.leave_result();
        if self.sel != i {
            self.sel = i;
            self.b.play_sound("rotate");
        }
    }

    fn toggle_turbo(&mut self) {
        self.turbo = !self.turbo;
        let v = self.turbo;
        self.set_opt("turbo", Value::Bool(v));
        self.say(if self.turbo { t("lcx.turbo_on") } else { t("lcx.turbo_off") }, 1.0);
        self.b.play_sound("click");
    }

    fn toggle_music(&mut self) {
        self.music_on = !self.music_on;
        let v = self.music_on;
        self.set_opt("music", Value::Bool(v));
        if !self.music_on {
            audio::music_stop(Some(self.ident()));
            self.mus_stage = None;
        }
        self.say(if self.music_on { t("lcx.music_on") } else { t("lcx.music_off") }, 1.0);
        self.b.play_sound("click");
    }

    fn open_window(&mut self, name: Win) {
        if self.window == Some(name) {
            self.window = None;
            self.b.play_sound("click");
            return;
        }
        if name == Win::Auto && self.auto.is_some() {
            self.stop_auto(Some(t("chip.auto_done")));
            return;
        }
        if self.round.as_ref().is_some_and(l::flying) {
            self.say(t("lcx.locked_fly"), 1.2);
            self.b.play_sound("hit");
            return;
        }
        if name == Win::Auto {
            if self.state == St::End && self.end_t >= READ_T {
                self.to_idle();
            }
            if self.state != St::Idle || self.broke || self.round.is_some() {
                self.say(t("lcx.locked"), 1.2);
                return;
            }
        }
        self.window = Some(name);
        if name == Win::Fair {
            self.fair_scroll = 0;
            self.fair_sel = if self.fairlog.is_empty() { None } else { Some(self.fairlog.len() - 1) };
        }
        self.b.play_sound("click");
    }

    // ----- Auto-Modus

    fn start_auto(&mut self) {
        if self.round.is_some() || !matches!(self.state, St::Idle | St::End) {
            return;
        }
        if self.total() > lamabank::balance() {
            self.say(t("cas.not_enough"), 1.8);
            self.b.play_sound("hit");
            return;
        }
        let mut forced = false;
        for i in self.active() {
            if !self.autos[i] {
                self.autos[i] = true;
                self.set_opt(&format!("auto{}", i + 1), Value::Bool(true));
                forced = true;
            }
        }
        self.window = None;
        if self.state == St::End {
            self.to_idle();
        }
        self.auto = Some(AutoRun { left: self.auto_rounds, net: 0, t: 0.0 });
        if forced {
            self.say(t("lcx.auto_forced"), 1.8);
        }
        self.b.play_sound("powerup");
    }

    /// Auto beenden. Eine laufende Runde bleibt offen und fliegt weiter.
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
        if self.state == St::Idle {
            self.check_broke();
        }
    }

    fn auto_tick(&mut self, dt: f64) {
        let (left, net) = {
            let Some(a) = self.auto.as_mut() else {
                return;
            };
            a.t -= dt;
            if a.t > 0.0 {
                return;
            }
            (a.left, a.net)
        };
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
        if self.total() > lamabank::balance() {
            self.stop_auto(Some(t("chip.auto_no_chips")));
            return;
        }
        if self.play() {
            if let Some(a) = self.auto.as_mut() {
                a.left -= 1;
            }
        } else {
            self.stop_auto(Some(t("chip.auto_no_chips")));
        }
    }

    // ===================================================== Effekte

    #[allow(dead_code, clippy::too_many_arguments)]
    fn spawn(&mut self, x: f64, y: f64, n: usize, col: Color, speed: f64, r: f64, life: f64, grav: f64) {
        for _ in 0..n {
            let a = self.rng.uniform(0.0, TAU);
            let sp = speed * self.rng.uniform(0.3, 1.0) * self.k;
            let rr = r * self.k * self.rng.uniform(0.6, 1.25);
            let lf = life * self.rng.uniform(0.7, 1.15);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                r: rr,
                col,
                t: 0.0,
                life: lf,
                g: grav * self.k,
            });
        }
        if self.particles.len() > 260 {
            let cut = self.particles.len() - 260;
            self.particles.drain(..cut);
        }
    }

    fn coin_shower(&mut self, n: usize) {
        let ww = self.b.width as f64;
        let hh = self.b.height as f64;
        for _ in 0..n {
            let x = self.rng.uniform(ww * 0.05, ww * 0.95);
            let y = self.rng.uniform(-hh * 0.4, -10.0);
            let vx = self.rng.uniform(-60.0, 60.0) * self.k;
            let vy = self.rng.uniform(-80.0, 120.0) * self.k;
            let spin = self.rng.uniform(0.0, TAU);
            let vs = self.rng.uniform(6.0, 14.0);
            let r = self.rng.uniform(7.0, 12.0) * pmax(0.7, self.k);
            self.coins_fx.push(CoinFx { x, y, vx, vy, spin, vs, r });
        }
        if self.coins_fx.len() > 180 {
            let cut = self.coins_fx.len() - 180;
            self.coins_fx.drain(..cut);
        }
    }

    fn popup(&mut self, x: f64, y: f64, text: String, color: Color, dur: f64) {
        self.popups.push(Popup { x, y, text, color, t: 0.0, dur });
    }

    // ===================================================== Eingabe

    fn on_key(&mut self, key: &str, repeat: bool) {
        let low = low_key(key);
        if key == "Escape" {
            if self.window.is_some() {
                self.window = None;
                self.b.play_sound("click");
            } else if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
            }
            return;
        }
        if key.chars().count() == 1 && ["a", "s", "f", "t", "m"].contains(&low.as_str()) {
            if repeat {
                return;
            }
            match low.as_str() {
                "t" => self.toggle_turbo(),
                "m" => self.toggle_music(),
                "a" => self.open_window(Win::Auto),
                "s" => self.open_window(Win::Stats),
                _ => self.open_window(Win::Fair),
            }
            return;
        }
        if self.window == Some(Win::Auto) {
            self.auto_key(key, repeat);
            return;
        }
        if self.window == Some(Win::Fair) {
            self.fair_key(key, repeat);
            return;
        }
        if self.window.is_some() {
            if is_action(key) && !repeat {
                self.window = None;
            }
            return;
        }
        let st = self.state;
        if st == St::Fly {
            if repeat {
                return;
            }
            if is_action(key) || low == "c" {
                self.cash_all();
            } else if ["1", "2", "KP_1", "KP_2"].contains(&key) {
                self.cash_one(if key.ends_with('1') { 0 } else { 1 }, false);
            } else if low == "h" {
                self.half_all();
            } else if low == "q" || low == "w" {
                self.cash_one(if low == "q" { 0 } else { 1 }, true);
            }
            return;
        }
        if st == St::Coast {
            if (is_action(key) || low == "c") && !repeat && !self.ff {
                self.ff = true;
                self.b.play_sound("click");
            }
            return;
        }
        if st == St::Count {
            return;
        }
        if self.auto.is_some() {
            // Auto laeuft (Ruhe/Ergebnis): Leertaste/Enter = STOPP
            if is_action(key) && !repeat {
                self.stop_auto(Some(t("chip.auto_done")));
            }
            return;
        }
        if st == St::End && (self.end_t < READ_T || repeat) && is_action(key) {
            return;
        }
        if is_action(key) {
            if !repeat {
                self.play();
            }
        } else if ["1", "2", "KP_1", "KP_2"].contains(&key) {
            self.select(if key.ends_with('1') { 0 } else { 1 });
        } else if key == "Tab" {
            if self.two {
                self.select(1 - self.sel);
            }
        } else if ["plus", "KP_Add", "Up", "equal"].contains(&key) {
            self.set_bet(self.sel, 1);
        } else if ["minus", "KP_Subtract", "Down"].contains(&key) {
            self.set_bet(self.sel, -1);
        } else if key == "Left" || key == "Right" {
            self.set_target(self.sel, if key == "Right" { 1 } else { -1 });
        } else if low == "x" && !repeat {
            self.toggle_target(self.sel);
        } else if low == "z" && !repeat {
            self.toggle_two();
        }
    }

    /// Tasten im Auto-Fenster: hoch/runter Zeile, links/rechts Wert,
    /// Enter/Leertaste = START/STOPP.
    fn auto_key(&mut self, key: &str, repeat: bool) {
        if key == "Up" || key == "Down" {
            self.auto_focus = py::pmod32(self.auto_focus + if key == "Down" { 1 } else { -1 }, 4);
            self.b.play_sound("rotate");
        } else if key == "Left" || key == "Right" {
            let dd: i64 = if key == "Right" { 1 } else { -1 };
            if self.auto_focus == 3 {
                self.toggle_turbo();
                return;
            }
            let row = self.auto_focus as usize;
            let opts: &[i64] = if row == 0 { &ROUND_OPTS } else { &LIMIT_OPTS };
            let cur = [self.auto_rounds, self.auto_win, self.auto_loss][row];
            let idx = opts.iter().position(|v| *v == cur).unwrap_or(0) as i64;
            let j = 0.max((opts.len() as i64 - 1).min(idx + dd));
            self.auto_set(row, opts[j as usize]);
        } else if is_action(key) && !repeat {
            if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
            } else {
                self.start_auto();
            }
        }
    }

    fn auto_set(&mut self, row: usize, value: i64) {
        if self.auto.is_some() {
            return;
        }
        let key = ["auto_rounds", "auto_win", "auto_loss"][row];
        let cur = [self.auto_rounds, self.auto_win, self.auto_loss][row];
        if cur != value {
            match row {
                0 => self.auto_rounds = value,
                1 => self.auto_win = value,
                _ => self.auto_loss = value,
            }
            self.set_opt(key, Value::from(value));
            self.b.play_sound("click");
        }
    }

    /// Neueste oben.
    fn fair_order(&self) -> Vec<usize> {
        (0..self.fairlog.len()).rev().collect()
    }

    /// Fairness-Fenster: hoch/runter waehlt eine Runde, Enter rechnet nach,
    /// C = neuer Client-Seed.
    fn fair_key(&mut self, key: &str, repeat: bool) {
        let low = low_key(key);
        let order = self.fair_order();
        if (key == "Up" || key == "Down") && !order.is_empty() {
            let pos = self.fair_sel.and_then(|s| order.iter().position(|v| *v == s)).unwrap_or(0) as i64;
            let pos = 0.max((order.len() as i64 - 1).min(pos + if key == "Down" { 1 } else { -1 }));
            self.fair_sel = Some(order[pos as usize]);
            self.b.play_sound("click");
        } else if (key == "Return" || key == "KP_Enter") && !repeat {
            if let Some(sel) = self.fair_sel {
                self.verify(sel);
            }
        } else if low == "c" && !repeat {
            self.new_client();
        } else if key == "space" && !repeat {
            self.window = None;
        }
    }

    fn new_client(&mut self) {
        if self.round.is_some() {
            self.say(t("lcx.locked"), 1.4);
            self.b.play_sound("hit");
            return;
        }
        {
            let rng = &mut self.seed_rng;
            fairness::new_client(&mut self.fair, &mut |k| rng.getrandbits(k));
        }
        self.save_extra();
        self.b.play_sound("select");
    }

    fn on_wheel(&mut self, delta: i32) {
        let dd: i64 = if delta > 0 { 1 } else { -1 };
        if self.window == Some(Win::Fair) {
            self.fair_scroll = 0.max(self.fair_scroll - dd);
            return;
        }
        if self.window.is_some() || !self.can_edit() {
            return;
        }
        for i in 0..self.cards.len() {
            let c = self.cards[i];
            if c.rect.hit(self.mouse) && self.active().contains(&i) {
                self.select(i);
                if c.row_b.hit(self.mouse) {
                    self.set_target(i, dd);
                } else {
                    self.set_bet(i, dd);
                }
                return;
            }
        }
        self.set_bet(self.sel, dd);
    }

    fn on_click(&mut self, pos: (i32, i32)) {
        for (name, r) in self.r_tools.clone() {
            if r.hit(pos) {
                match name {
                    "turbo" => self.toggle_turbo(),
                    "music" => self.toggle_music(),
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
        let st = self.state;
        if self.r_go.hit(pos) {
            if st == St::Fly {
                self.cash_all();
            } else if st == St::Coast {
                if !self.ff {
                    self.ff = true;
                    self.b.play_sound("click");
                }
            } else if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
            } else if matches!(st, St::Idle | St::End) {
                if st == St::End && self.end_t < READ_T {
                    return;
                }
                self.play();
            }
            return;
        }
        if self.r_half.hit(pos) {
            if st == St::Fly {
                self.half_all();
            }
            return;
        }
        for i in 0..self.cards.len() {
            let c = self.cards[i];
            if !c.rect.hit(pos) {
                continue;
            }
            if st == St::Fly {
                if let Some(n) = self.round.as_ref().map(|r| r.bets.len()) {
                    if i < n {
                        if c.cash.hit(pos) {
                            self.cash_one(i, false);
                        } else if c.halfc.hit(pos) {
                            self.cash_one(i, true);
                        }
                    }
                    return;
                }
            }
            if !self.can_edit() {
                if self.auto.is_some() {
                    self.say(t("lcx.locked"), 1.2);
                }
                return;
            }
            if i == 1 && (c.sw.hit(pos) || !self.two) {
                self.toggle_two();
                if self.two {
                    self.sel = 1;
                }
                return;
            }
            self.select(i);
            if c.minus.hit(pos) {
                self.set_bet(i, -1);
            } else if c.plus.hit(pos) {
                self.set_bet(i, 1);
            } else if c.auto.hit(pos) {
                self.toggle_target(i);
            } else if c.tl.hit(pos) {
                self.set_target(i, -1);
            } else if c.tr.hit(pos) {
                self.set_target(i, 1);
            }
            return;
        }
        if self.stage.hit(pos) && st == St::Idle && self.auto.is_none() {
            self.say(t("lcx.press_play"), 1.4);
        }
    }

    fn auto_click(&mut self, pos: (i32, i32)) {
        if self.r_auto_close.hit(pos) {
            self.window = None;
            self.b.play_sound("click");
            return;
        }
        let rows = self.auto_rows.clone();
        for (row, (_ly, rects)) in rows.iter().enumerate() {
            let opts: &[i64] = if row == 0 { &ROUND_OPTS } else { &LIMIT_OPTS };
            for (j, r) in rects.iter().enumerate() {
                if r.hit(pos) {
                    self.auto_focus = row as i32;
                    self.auto_set(row, opts[j]);
                    return;
                }
            }
        }
        if self.r_auto_turbo.hit(pos) {
            self.auto_focus = 3;
            self.toggle_turbo();
        } else if self.r_auto_start.hit(pos) {
            if self.auto.is_some() {
                self.stop_auto(Some(t("chip.auto_done")));
            } else {
                self.start_auto();
            }
        } else if !self.auto_r.hit(pos) && !self.panel_r.hit(pos) {
            self.window = None;
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
            self.new_client();
            return;
        }
        if self.r_fair_verify.hit(pos) {
            if let Some(sel) = self.fair_sel {
                self.verify(sel);
                return;
            }
        }
        for (idx, r) in self.r_fair_rows.clone() {
            if r.hit(pos) {
                self.fair_sel = Some(idx);
                self.b.play_sound("click");
                return;
            }
        }
    }

    /// Nachrechnen: Pruefcode = sha256(Seed), Platzpunkt aus den Seeds,
    /// Ausstiege passen.
    fn verify(&mut self, idx: usize) {
        if idx >= self.fairlog.len() {
            return;
        }
        let e = self.fairlog[idx].clone();
        let mut ok = e.verify();
        let crash = l::crash_point(&e.s, &e.c, e.n);
        ok = ok && l::check_outcome(Some(&e.o), crash);
        let mut marks: Vec<i64> = Vec::new();
        if let Some(Value::Array(rows)) = e.o.as_object().and_then(|o| o.get("b")) {
            for row in rows {
                if let Value::Array(rv) = row {
                    if rv.len() == 4 {
                        for m in [&rv[1], &rv[2]] {
                            if let Some(v) = py_int(Some(m)) {
                                if v > 0 {
                                    marks.push(v);
                                }
                            }
                        }
                    }
                }
            }
        }
        marks.sort();
        self.fair_checks.insert(e.n, (ok, crash, marks));
        self.b.play_sound(if ok { "point" } else { "hit" });
    }

    // ===================================================== Update

    /// Toene im Flug - nur vom aktuellen Multiplikator abhaengig (verraten nichts).
    fn update_sounds(&mut self, dt: f64) {
        if !matches!(self.state, St::Fly | St::Coast) {
            return;
        }
        let Some(t_now) = self.round.as_ref().map(|r| r.t) else {
            return;
        };
        let m = l::mult_at(t_now);
        let wn = 4.min(l::world_of(m));
        if wn > self.world_seen {
            self.world_seen = wn;
            self.world_label = Some((w::name_key(wn as i32), 2.4));
            audio::tone(392.0 * py::pow(2.0, (wn * 2) as f64 / 12.0), 0.22, Some(&self.b.settings), "triangle", 0.18);
        }
        while self.tone_i < TONE_AT.len() && m >= TONE_AT[self.tone_i] {
            audio::tone(
                523.25 * py::pow(2.0, (self.tone_i * 2) as f64 / 12.0),
                0.12,
                Some(&self.b.settings),
                "sine",
                0.22,
            );
            self.tone_i += 1;
        }
        if let Some((freq, gap)) = mu::squeak(m) {
            self.squeak_t -= dt;
            if self.squeak_t <= 0.0 {
                audio::tone(freq, 0.09, Some(&self.b.settings), "triangle", 0.10);
                self.squeak_t = gap;
            }
        }
    }

    fn update_fx(&mut self, dt: f64) {
        let k = self.k;
        for p in self.particles.iter_mut() {
            p.t += dt;
            p.vy += p.g * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.t < p.life);
        let hh = self.b.height as f64;
        for c in self.coins_fx.iter_mut() {
            c.vy += 900.0 * k * dt;
            c.x += c.vx * dt;
            c.y += c.vy * dt;
            c.spin += c.vs * dt;
        }
        self.coins_fx.retain(|c| c.y < hh + 30.0);
        for p in self.popups.iter_mut() {
            p.t += dt;
        }
        self.popups.retain(|p| p.t < p.dur);
        let bottom = self.stage.bottom() as f64 + 60.0 * k;
        for c in self.chutes.iter_mut() {
            c.t += dt;
            if c.t < 0.35 {
                c.vy += 520.0 * k * dt;
            } else {
                c.vy += (60.0 * k - c.vy) * pmin(1.0, dt * 3.0);
                c.vx *= pmax(0.0, 1.0 - dt * 0.8);
            }
            c.x += (c.vx + (c.t * 2.2 + c.slot as f64).sin() * 18.0 * k) * dt;
            c.y += c.vy * dt;
        }
        self.chutes.retain(|c| c.y < bottom && c.t < 9.0);
        for f in self.fallers.iter_mut() {
            f.t += dt;
            f.vy += 900.0 * k * dt;
            f.x += f.vx * dt;
            f.y += f.vy * dt;
        }
        self.fallers.retain(|f| f.y < bottom);
        for p in self.shreds.iter_mut() {
            p.t += dt;
            p.vy += 420.0 * k * dt;
            p.vx *= pmax(0.0, 1.0 - dt * 1.2);
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.shreds.retain(|p| p.t < p.life);
        if let Some(f) = self.pop_fx.as_mut() {
            f.t += dt;
            if f.t > 0.6 {
                self.pop_fx = None;
            }
        }
        if let Some(b) = self.basket_fx.as_mut() {
            b.t += dt;
            b.vy += 900.0 * k * dt;
            b.y += b.vy * dt;
            if b.y > bottom {
                self.basket_fx = None;
            }
        }
    }

    /// Spannungsmusik: Stufe je Hoehenwelt, Wechsel nur auf dem Schlag.
    fn update_music(&mut self, dt: f64) {
        self.mus_t += dt;
        if !self.music_on {
            return;
        }
        let st = self.state;
        let (mut want, mut vol): (Option<usize>, f64) = (Some(0), 0.45);
        if st == St::End {
            if self.end == Some("moon") {
                want = Some(5);
                vol = 1.0;
            } else {
                want = None;
                vol = 0.0;
            }
        } else if matches!(st, St::Fly | St::Coast) && self.round.is_some() {
            let t_now = self.round.as_ref().map(|r| r.t).unwrap_or(0);
            want = Some(mu::stage_of(l::mult_at(t_now)));
            vol = if st == St::Fly { 1.0 } else { 0.75 };
        } else if st == St::Count {
            want = Some(0);
            vol = 0.8;
        }
        let Some(want) = want else {
            if self.mus_stage.is_some() {
                audio::music_stop(Some(self.ident()));
                self.mus_stage = None;
            }
            return;
        };
        let mut guard = 0;
        while !mu::ready(want) && guard < mu::STAGES {
            if !mu::prerender_step() {
                break;
            }
            guard += 1;
        }
        if mu::ready(want) {
            let playing = audio::music_playing(Some(self.ident()));
            let mut switch = self.mus_stage.is_none() || !playing;
            if !switch && Some(want) != self.mus_stage {
                let beat = mu::beat_seconds(self.mus_stage.unwrap_or(0));
                switch = (self.mus_t / beat) as i64 != ((self.mus_t - dt) / beat) as i64;
            }
            if switch && (Some(want) != self.mus_stage || !playing) && Some(want) != self.mus_fail {
                let ok = match mu::voices(want) {
                    Some(v) => audio::music_play(&v, self.ident(), Some(&self.b.settings), vol),
                    None => false,
                };
                if ok {
                    self.mus_stage = Some(want);
                    self.mus_t = 0.0;
                    self.mus_vol = Some(vol);
                    self.mus_fail = None;
                } else {
                    self.mus_fail = Some(want); // kein Mixer: nicht jedes Bild neu versuchen
                }
            }
        }
        if self.mus_stage.is_some() && Some(vol) != self.mus_vol {
            audio::music_set_volume(vol);
            self.mus_vol = Some(vol);
        }
        if st != St::Fly {
            mu::prerender_step(); // restliche Stufen in ruhigen Bildern vorbereiten
        }
    }

    // ===================================================== Zeichnen

    fn shake_off(&self) -> (i32, i32) {
        if self.shake <= 0.0 {
            return (0, 0);
        }
        let a = 8.0 * self.k * self.shake / SHAKE_T;
        (((self.time * 71.0).sin() * a) as i32, ((self.time * 53.0).cos() * a * 0.7) as i32)
    }

    /// Plaetze im Korb: [(Halstuch, Pose) oder None] - je Einsatz ein Lama.
    fn riders(&self) -> Vec<Option<(Color, &'static str)>> {
        let mut out = Vec::new();
        if let Some(r) = self.round.as_ref() {
            for (i, b) in r.bets.iter().enumerate() {
                out.push(if l::riding(b) > 0 { Some((CARD_COL[i].c(), "stand")) } else { None });
            }
        } else if self.state == St::End && self.last.is_some() {
            let last = self.last.as_ref().unwrap();
            for (i, b) in last.rnd.bets.iter().enumerate() {
                let keep = last.end == "moon" && (l::riding(b) > 0 || b.xm == l::MAX_MULT100);
                out.push(if keep { Some((CARD_COL[i].c(), "hop")) } else { None });
            }
        } else {
            for i in self.active() {
                out.push(Some((CARD_COL[i].c(), "stand")));
            }
        }
        out
    }

    fn flame(&self) -> i32 {
        if self.state == St::Count || self.flame_t > 0.0 {
            return py::pmod((self.time * 14.0) as i64, 6) as i32;
        }
        if matches!(self.state, St::Fly | St::Coast) && py::fpmod(self.time, 3.1) < 0.45 {
            return py::pmod((self.time * 14.0) as i64, 6) as i32;
        }
        -1
    }

    fn draw_stage(&mut self, s: &mut Surface) {
        let st = self.stage;
        let (ox, oy) = self.shake_off();
        let clip = s.get_clip();
        s.set_clip(Some(st));
        let alt = self.alt();
        let moon = self.state == St::End && self.end == Some("moon");
        let (world, share) = if moon { (5, 0) } else { l::world_blend(alt) };
        let (world, share) = (world as i32, share as i32);
        w::draw(s, st, world, share, alt, self.time);
        if !moon {
            let (x_max, y_max) = l::chart_span(alt as i64);
            let ylines: Vec<(i32, String)> = l::chart_ylines(y_max)
                .into_iter()
                .map(|v| (self.pt(0.0, v as f64, x_max, y_max).1 as i32 + oy, self.fmt_short(v)))
                .collect();
            let xlines: Vec<(i32, String)> = l::chart_xlines(x_max)
                .into_iter()
                .map(|v| (self.pt(v as f64, 100.0, x_max, y_max).0 as i32 + ox, format!("{}s", py::fdiv(v, l::TPS))))
                .collect();
            d::draw_grid(s, self.plot.move_(ox, oy), &ylines, &xlines, self.fs_micro);
            if alt > 0.0 {
                let mut pts = Vec::new();
                let n = 48;
                for j in 0..=n {
                    let tt = alt * j as f64 / n as f64;
                    let (x, y) = self.pt(tt, Self::m_at_f(tt), x_max, y_max);
                    pts.push((x + ox as f64, y + oy as f64));
                }
                let col = if self.state == St::End { ui::pal().red } else { ui::mix(self.b.accent, (255, 255, 255), 0.15) };
                d::draw_curve(s, &pts, col, (self.plot.bottom() + oy) as f64, 2.max((3.0 * self.k) as i32));
            }
        }
        self.draw_rig(s, ox, oy, moon);
        self.draw_flyers(s, ox, oy);
        self.draw_center(s);
        s.set_clip(Some(clip));
        draw::rect_r(s, ui::mix(self.b.accent, ui::pal().border, 0.55), st, 1, 4.max((8.0 * self.k) as i32));
    }

    fn draw_rig(&mut self, s: &mut Surface, ox: i32, oy: i32, moon: bool) {
        if self.state == St::End && self.end == Some("pop") {
            return;
        }
        let n = self.n_riders();
        let mut riders = self.riders();
        riders.truncate(n);
        if moon {
            let (cx, rim_y) = self.moon_rig();
            let fl = d::flag(12.max((self.rig_base as f64 * 0.7) as i32), CARD_COL[0]);
            let bw = d::basket_w(self.rig_base);
            s.blr(
                &fl,
                fl.get_rect().bottomleft_at((cx + py::fdiv32(bw, 2) + (6.0 * self.k) as i32, rim_y + py::fdiv32(bw * 2, 3))),
            );
            self.env_rect =
                d::draw_rig(s, cx as f64, rim_y as f64, self.rig_base, d::INFL_MAX, self.b.accent, &riders, -1);
            return;
        }
        let (x, y) = self.rig_pos();
        let m = self.cur_mult();
        let level = d::infl_level(m);
        let mut sway = 0.0;
        if matches!(self.state, St::Fly | St::Coast) {
            sway = (self.time * 1.7).sin() * self.k * (1.5 + level as f64 * 0.35);
        }
        let flame = self.flame();
        self.env_rect = d::draw_rig(
            s,
            x + ox as f64 + sway,
            y + oy as f64,
            self.rig_base,
            level,
            self.b.accent,
            &riders,
            flame,
        );
    }

    fn draw_flyers(&self, s: &mut Surface, ox: i32, oy: i32) {
        let (oxf, oyf) = (ox as f64, oy as f64);
        for c in &self.chutes {
            let open_f = pmax(0.0, pmin(1.0, (c.t - 0.25) / 0.4));
            if c.llama {
                d::draw_jumper(s, c.x + oxf, c.y + oyf, c.n, CARD_COL[c.slot], CHUTE_COL[c.slot], open_f);
            } else {
                d::draw_sack(s, c.x + oxf, c.y + oyf, c.n, CHUTE_COL[c.slot], open_f);
            }
        }
        let n = d::llama_size(self.rig_base);
        for f in &self.fallers {
            d::draw_faller(s, f.x + oxf, f.y + oyf, n, CARD_COL[f.slot], py::pmod((f.t * 10.0) as i64, 8) as i32);
        }
        if let Some(b) = self.basket_fx.as_ref() {
            let bw = d::basket_w(self.rig_base);
            let x = b.x as i32 - py::fdiv32(bw, 2) + ox;
            let y = b.y as i32 + oy;
            s.bl(&d::basket(bw, false), (x, y));
            s.bl(&d::basket(bw, true), (x, y));
        }
        for p in &self.shreds {
            let pieces = d::shreds(p.w, self.b.accent);
            let img = &pieces[p.i % pieces.len()];
            s.blr(img, img.get_rect().center_at((p.x as i32 + ox, p.y as i32 + oy)));
        }
        if let Some(f) = self.pop_fx.as_ref() {
            let img = d::pop_star(f.r, 5.min((f.t / 0.1) as i32));
            s.blr(&img, img.get_rect().center_at((f.x + ox, f.y + oy)));
        }
        for p in &self.particles {
            let fr = p.t / p.life;
            let r = p.r * (1.0 - fr);
            if r >= 0.5 {
                draw::circle(s, p.col, (p.x as i32, p.y as i32), 1.max(r as i32), 0);
            }
        }
    }

    /// Grosser Multiplikator, Countdown bzw. Ergebnis mitten auf der Buehne.
    fn draw_center(&self, s: &mut Surface) {
        let p = ui::pal();
        let st = self.stage;
        let cx = st.centerx();
        let cy = st.y + (st.h as f64 * 0.30) as i32;
        let state = self.state;
        if state == St::Count {
            let seg = self.count_len / 3.0;
            let num = 1.max(3.min((self.count_t / seg) as i32 + 1));
            let ph = py::fpmod(self.count_t, seg) / seg;
            let img = self.f_huge.render(&num.to_string(), true, cd::COL_GOLD);
            let sc = 1.0 + 0.35 * ph;
            let img = transform::smoothscale(
                &img,
                1.max((img.w() as f64 * sc) as i32),
                1.max((img.h() as f64 * sc) as i32),
            );
            s.blr(&img, img.get_rect().center_at((cx, cy)));
            return;
        }
        if matches!(state, St::Fly | St::Coast) {
            let m = self.cur_mult();
            let txt = self.fmt(m);
            let col = Self::mult_col(m);
            let fonts = [&self.f_huge, &self.f_mult, &self.f_big];
            let img = fit(&txt, &fonts, st.w - 16, col);
            let sh = fit(&txt, &fonts, st.w - 16, (12, 8, 20));
            s.blr(&sh, sh.get_rect().center_at((cx + 2, cy + 3)));
            s.blr(&img, img.get_rect().center_at((cx, cy)));
            let y = cy + img.h() / 2 + 2;
            let (sub, scol) = match (state, self.round.as_ref()) {
                (St::Fly, Some(rnd)) => {
                    let mut now = 0;
                    for b in &rnd.bets {
                        now += py::fdiv(l::riding(b) * m, 100);
                    }
                    (tf("lcx.win_now", &[("n", &self.group(now))]), p.green)
                }
                _ => (t("lcx.coast"), p.text_dim),
            };
            let simg = fit(&sub, &[&self.f_btn, &self.f_tiny], st.w - 16, scol);
            s.blr(&simg, simg.get_rect().midtop_at((cx, y)));
            if let Some((key_, left)) = self.world_label.as_ref() {
                let left = *left;
                let wimg = fit(&t(key_), &[&self.f_big, &self.f_btn, &self.f_tiny], st.w - 24, (255, 255, 255));
                let mut bx = wimg.get_rect().inflate((28.0 * self.k) as i32, (10.0 * self.k) as i32);
                bx.set_midtop((cx, st.y + self.margin));
                let mut panel = pg::surface_alpha(bx.w, bx.h);
                let pr = panel.get_rect();
                draw::rect_r(&mut panel, rgba(10, 8, 18, 170), pr, 0, bx.h / 2);
                draw::rect_r(&mut panel, pg::alpha(self.b.accent, 220), pr, 1, bx.h / 2);
                panel.blr(&wimg, wimg.get_rect().center_at((bx.w / 2, bx.h / 2)));
                let a = pmin(pmin(1.0, (2.4 - left) / 0.25), left / 0.5);
                panel.alpha_set((255.0 * pmax(0.0, a)) as i32);
                s.blr(&panel, bx);
            }
            return;
        }
        if state == St::End {
            if let Some(last) = self.last.as_ref() {
                let f = pmin(1.0, self.end_t / 0.18);
                if last.end != "pop" {
                    return; // Mondlandung: Banner
                }
                let title = t("lcx.popped");
                let val = self.fmt(last.crash);
                let col = p.red;
                let timg = fit(&title, &[&self.f_big, &self.f_btn], st.w - 16, col);
                let mut vimg = fit(&val, &[&self.f_huge, &self.f_mult, &self.f_big], st.w - 16, col);
                if f < 1.0 {
                    let sc = 0.7 + 0.3 * f;
                    vimg = transform::smoothscale(
                        &vimg,
                        1.max((vimg.w() as f64 * sc) as i32),
                        1.max((vimg.h() as f64 * sc) as i32),
                    );
                }
                s.blr(&timg, timg.get_rect().midbottom_at((cx, cy - vimg.h() / 2)));
                s.blr(&vimg, vimg.get_rect().center_at((cx, cy)));
                let res = last.paid - last.stake;
                let sub = tf("lcx.round_net", &[("n", &format!("{}{}", if res > 0 { "+" } else { "" }, self.group(res)))]);
                let simg = fit(&sub, &[&self.f_btn, &self.f_tiny], st.w - 16, if res >= 0 { p.green } else { p.text_dim });
                s.blr(&simg, simg.get_rect().midtop_at((cx, cy + vimg.h() / 2 + 2)));
                return;
            }
        }
        if state == St::Idle {
            if let Some(last) = self.last.as_ref() {
                let x = if last.end == "pop" { last.crash } else { l::MAX_MULT100 };
                let txt = tf("lcx.last_crash", &[("x", &self.fmt(x))]);
                let img = fit(&txt, &[&self.f_small, &self.f_tiny], st.w - 16, p.text_dim);
                s.blr(&img, img.get_rect().midtop_at((cx, st.y + self.margin)));
            }
        }
    }

    // ----- Seitenleiste

    fn draw_panel(&self, s: &mut Surface) {
        ui::draw_panel_ex(s, self.panel_r, None, None, None, true, Some(self.b.accent));
        for i in 0..2 {
            self.draw_card(s, i);
        }
        self.draw_info(s);
        self.draw_hint(s);
    }

    /// Teil-Einsatz der laufenden bzw. letzten Runde fuer Karte i (oder None).
    fn card_bet(&self, i: usize) -> Option<l::Bet> {
        if let Some(r) = self.round.as_ref() {
            return r.bets.get(i).cloned();
        }
        if let Some(last) = self.last.as_ref() {
            if self.state == St::End || self.auto.is_some() {
                return last.rnd.bets.get(i).cloned();
            }
        }
        None
    }

    fn draw_card(&self, s: &mut Surface, i: usize) {
        let p = ui::pal();
        let c = self.cards[i];
        let r = c.rect;
        let k = self.k;
        let col = CARD_COL[i].c();
        let hot = self.mouse;
        let editing = self.state == St::Idle && self.round.is_none() && self.auto.is_none();
        let bet = self.card_bet(i);
        let mut active = if editing || bet.is_none() { self.active().contains(&i) } else { true };
        if !editing && bet.is_none() {
            active = false;
        }
        let rad = 5.max((8.0 * k) as i32);
        draw::rect_r(s, ui::mix(p.panel, (0, 0, 0), 0.28), r, 0, rad);
        let edge = if active && (!editing || self.sel == i) {
            col
        } else if active {
            ui::mix(col, p.border, 0.6)
        } else {
            p.border
        };
        draw::rect_r(s, edge, r, if editing && self.sel == i && active { 2 } else { 1 }, rad);
        // Titelzeile: Farbpunkt, "Einsatz n", rechts Schalter bzw. Zustand
        let (ty, th) = (c.title_y, c.title_h);
        let dot_r = 3.max(th / 4);
        draw::circle(s, if active { col } else { p.text_faint }, (r.x + 6.max((9.0 * k) as i32), ty + th / 2), dot_r, 0);
        let mut title = tf("lcx.bet_n", &[("n", &(i + 1).to_string())]);
        if let Some(b) = bet.as_ref() {
            title += &format!("{}{}", DOT, self.group(b.b));
        }
        let right_w = if i == 1 && editing { c.sw.w + 6 } else { 0 };
        let img = fit(
            &title,
            &[&self.f_tiny, &self.f_micro],
            r.w - right_w - (24.0 * k) as i32,
            if active { p.text } else { p.text_dim },
        );
        s.blr(&img, img.get_rect().midleft_at((r.x + 6.max((9.0 * k) as i32) + dot_r + 5, ty + th / 2)));
        if i == 1 && editing {
            self.switch(s, c.sw, self.two, c.sw.hit(hot));
        } else if let Some(b) = bet.as_ref() {
            let (badge, bcol) = self.bet_badge(b);
            let bimg = fit(&badge, &[&self.f_micro], r.w / 2, bcol);
            s.blr(&bimg, bimg.get_rect().midright_at((r.right() - 5.max((8.0 * k) as i32), ty + th / 2)));
        }
        if editing {
            if !active {
                let msg = t("lcx.second_hint");
                let img = fit(&msg, &[&self.f_tiny, &self.f_micro], r.w - 12, p.text_faint);
                s.blr(&img, img.get_rect().center_at((r.centerx(), py::fdiv32(c.row_a.y + c.row_b.bottom(), 2))));
                return;
            }
            let on_m = self.bets[i] > l::BETS[0];
            let on_p = self.bets[i] < l::BETS[l::BETS.len() - 1];
            self.btn(s, c.minus, "\u{2013}", false, on_m, c.minus.hit(hot), false, false, None, false);
            self.btn(s, c.plus, "+", false, on_p, c.plus.hit(hot), false, false, None, false);
            let vcol = if self.total() > lamabank::balance() { p.red } else { p.gold };
            self.value_box(s, c.bet, &self.group(self.bets[i]), vcol);
            self.check(s, c.auto, self.autos[i], c.auto.hit(hot));
            let on = self.autos[i];
            self.btn(
                s,
                c.tl,
                "",
                false,
                on && self.targets[i] > l::TARGETS[0],
                c.tl.hit(hot),
                false,
                false,
                Some("left"),
                false,
            );
            self.btn(
                s,
                c.tr,
                "",
                false,
                on && self.targets[i] < l::TARGETS[l::TARGETS.len() - 1],
                c.tr.hit(hot),
                false,
                false,
                Some("right"),
                false,
            );
            let txt = if on { self.fmt(self.targets[i]) } else { t("lcx.target_off") };
            self.value_box(s, c.tv, &txt, if on { col } else { p.text_faint });
            return;
        }
        let Some(bet) = bet else {
            let img = fit("\u{2013}", &[&self.f_btn], r.w - 12, p.text_faint);
            s.blr(&img, img.get_rect().center_at((r.centerx(), py::fdiv32(c.row_a.y + c.row_b.bottom(), 2))));
            return;
        };
        // Runde laeuft bzw. ist gerade vorbei
        let (ra, rb) = (c.row_a, c.row_b);
        let m = self.cur_mult();
        let riding = l::riding(&bet);
        let mut mark: Option<&str> = None;
        let (left, lcol) = if riding > 0 && self.state != St::End {
            (tf("lcx.win_now", &[("n", &self.group(py::fdiv(riding * m, 100)))]), p.green)
        } else if bet.xm != 0 {
            mark = Some("check");
            (self.fmt(bet.xm), cd::COL_GOLD.c())
        } else if self.state == St::End {
            mark = Some("cross");
            (t("lcx.lost"), p.red)
        } else {
            (ELL.to_string(), p.text_dim)
        };
        let right = if bet.p > 0 {
            format!("+{}", self.group(bet.p))
        } else if bet.a != 0 {
            tf("lcx.target_at", &[("x", &self.fmt(bet.a))])
        } else {
            String::new()
        };
        let rimg = fit(
            &right,
            &[&self.f_tiny, &self.f_micro],
            ra.w / 2,
            if bet.p > 0 { cd::COL_GOLD.c() } else { p.text_dim },
        );
        let mut lx = ra.x;
        if let Some(mk) = mark {
            let isz = 8.max((ra.h as f64 * 0.62) as i32);
            s.bl(&d::icon(mk, isz, lcol), (lx, ra.centery() - isz / 2));
            lx += isz + 4;
        }
        let limg = fit(&left, &[&self.f_btn, &self.f_tiny, &self.f_micro], ra.right() - lx - rimg.w() - 6, lcol);
        s.blr(&limg, limg.get_rect().midleft_at((lx, ra.centery())));
        s.blr(&rimg, rimg.get_rect().midright_at((ra.right(), ra.centery())));
        if self.state == St::Fly && riding > 0 {
            let can = m >= l::MIN_CASH;
            let lab = format!("{} {}", t("lcx.cash"), self.group(py::fdiv(riding * m, 100)));
            self.btn(s, c.cash, &lab, true, can, c.cash.hit(hot), false, false, None, false);
            self.btn(s, c.halfc, "", false, can && bet.hm == 0, c.halfc.hit(hot), false, false, Some("half"), false);
        } else {
            let mut info = String::new();
            if bet.hm != 0 {
                info = tf("lcx.half_at", &[("x", &self.fmt(bet.hm))]);
            } else if self.state == St::Count {
                info = t("lcx.ready");
            }
            let img = fit(&info, &[&self.f_tiny, &self.f_micro], rb.w, p.text_dim);
            s.blr(&img, img.get_rect().midleft_at((rb.x, rb.centery())));
        }
    }

    fn bet_badge(&self, bet: &l::Bet) -> (String, Color) {
        let p = ui::pal();
        if l::riding(bet) > 0 && self.state != St::End {
            return (t("lcx.riding"), p.green);
        }
        if bet.xm != 0 {
            return (t("lcx.out"), cd::COL_GOLD.c());
        }
        (t("lcx.lost"), p.red)
    }

    fn value_box(&self, s: &mut Surface, r: Rect, text: &str, col: Color) {
        let p = ui::pal();
        let rad = 4.max(r.h / 4);
        draw::rect_r(s, (16, 12, 26), r, 0, rad);
        draw::rect_r(s, p.border, r, 1, rad);
        let img = fit(text, &[&self.f_btn, &self.f_tiny, &self.f_micro], r.w - 6, col);
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn switch(&self, s: &mut Surface, r: Rect, on: bool, hot: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { cd::COL_GOLD } else { (60, 60, 76) }, r, 0, r.h / 2);
        if hot {
            draw::rect_r(s, p.border_light, r, 1, r.h / 2);
        }
        let kr = 2.max(r.h / 2 - 2);
        let kx = if on { r.right() - kr - 3 } else { r.x + kr + 3 };
        draw::circle(s, (255, 255, 255), (kx, r.centery()), kr, 0);
    }

    fn check(&self, s: &mut Surface, r: Rect, on: bool, hot: bool) {
        let p = ui::pal();
        let rad = 4.max(r.h / 4);
        draw::rect_r(s, if hot { p.btn_sel } else { p.btn }, r, 0, rad);
        draw::rect_r(s, if on { cd::COL_GOLD.c() } else { p.border_light }, r, if on { 2 } else { 1 }, rad);
        if on {
            let img = d::icon("check", 8.max((r.h as f64 * 0.62) as i32), cd::COL_GOLD);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
    }

    /// Zeilen des Info-Kastens: [(Beschriftung, Wert, Farbe)].
    fn info_rows(&self) -> Vec<(String, String, Color)> {
        let p = ui::pal();
        if self.state == St::End {
            if let Some(last) = self.last.as_ref() {
                let net = last.paid - last.stake;
                let x = if last.end == "pop" { last.crash } else { l::MAX_MULT100 };
                return vec![
                    (t("lcx.crash_at"), self.fmt(x), if last.end == "pop" { p.red } else { cd::COL_GOLD.c() }),
                    (
                        t("chip.st_paid"),
                        self.group(last.paid),
                        if last.paid != 0 { cd::COL_GOLD.c() } else { p.text_dim },
                    ),
                    (
                        t("chip.st_net"),
                        format!("{}{}", if net > 0 { "+" } else { "" }, self.group(net)),
                        if net >= 0 { p.green } else { p.red },
                    ),
                ];
            }
        }
        if let Some(rnd) = self.round.as_ref() {
            if matches!(self.state, St::Fly | St::Coast | St::Count) {
                let m = self.cur_mult();
                let secs = py::fdiv(rnd.t, l::TPS);
                return vec![
                    (t("lcx.world"), t(&w::name_key(4.min(l::world_of(m)) as i32)), self.b.accent),
                    (t("lcx.flight"), format!("{}:{:02}", py::fdiv(secs, 60), py::pmod(secs, 60)), p.text),
                    (t("chip.st_paid"), self.group(l::paid_total(rnd)), cd::COL_GOLD.c()),
                ];
            }
        }
        let mut rows = vec![(
            t("lcx.total"),
            self.group(self.total()),
            if self.total() > lamabank::balance() { p.red } else { p.text },
        )];
        for i in self.active() {
            if self.autos[i] {
                rows.push((
                    tf("lcx.chance", &[("x", &self.fmt(self.targets[i]))]),
                    format!("{} %", l::fmt_chance(self.targets[i], &self.sep())),
                    CARD_COL[i].c(),
                ));
            }
        }
        rows.push((t("chip.max_win"), self.fmt(l::MAX_MULT100), cd::COL_GOLD.c()));
        rows
    }

    fn draw_info(&self, s: &mut Surface) {
        let p = ui::pal();
        let (x0, wd) = self.panel_inner;
        let mut y = self.info_y;
        let mut rows = self.info_rows();
        let avail = self.hint_y - 4.max((6.0 * self.k) as i32) - y;
        let min_lh = self.f_tiny.get_height() + 2;
        let max_lh = (self.f_btn.get_height() as f64 * 1.6) as i32;
        while !rows.is_empty() && py::fdiv32(avail, rows.len() as i32) < min_lh {
            rows.pop();
        }
        if rows.is_empty() {
            return;
        }
        let lh = max_lh.min(py::fdiv32(avail, rows.len() as i32));
        for (lab, val, col) in rows.iter() {
            let vimg = fit(val, &[&self.f_btn, &self.f_tiny], wd / 2, *col);
            let limg = fit(lab, &[&self.f_small, &self.f_tiny, &self.f_micro], wd - vimg.w() - 8, p.text_dim);
            s.blr(&limg, limg.get_rect().midleft_at((x0, y + lh / 2)));
            s.blr(&vimg, vimg.get_rect().midright_at((x0 + wd, y + lh / 2)));
            y += lh;
        }
    }

    fn draw_hint(&self, s: &mut Surface) {
        if self.hint_y >= self.panel_r.bottom() - self.ppad {
            return;
        }
        let p = ui::pal();
        let (x0, wd) = self.panel_inner;
        let text = if let Some(a) = self.auto.as_ref() {
            tf("chip.auto_left", &[("n", &a.left.to_string())])
        } else if self.state == St::Fly {
            t("lcx.hint_fly")
        } else if self.state == St::Coast {
            t("lcx.hint_coast")
        } else {
            t("lcx.hint_idle")
        };
        let mut y = self.hint_y;
        for line in wrap(&text, &self.f_micro, wd).iter().take(2) {
            let img = self.f_micro.render(line, true, p.text_faint);
            s.bl(&img, (x0, y));
            y += self.f_micro.get_height() + 1;
        }
    }

    // ----- Kopfzeile, Werkzeuge, Verlauf

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let ww = self.b.width;
        s.bl(&self.hud_bg, (0, 0));
        draw::line(s, ui::mix(cd::COL_GOLD, self.b.accent, 0.35), (0, self.hud_h - 1), (ww, self.hud_h - 1), 1);
        let cy = self.hud_h / 2;
        let pad = self.margin;
        let coin_r = 6.max((self.hud_h as f64 * 0.26) as i32);
        let coin = cd::symbol_surface("coin", coin_r * 2 + 4, false);
        s.blr(&coin, coin.get_rect().midleft_at((pad - 2, cy)));
        let left = self.f_big.render(
            &format!("{}: {}", t("cas.chips"), self.group(py::round(self.shown_chips))),
            true,
            p.gold,
        );
        s.blr(&left, left.get_rect().midleft_at((pad + coin_r * 2 + 6, cy)));
        let right_txt = format!(
            "{}: {}   \u{00B7}   {}",
            t("cas.record"),
            self.group(self.b.score),
            tf("cas.credits", &[("n", &lamabank::refills().to_string())])
        );
        let mut right = self.f_small.render(&right_txt, true, p.text_dim);
        if right.w() as f64 > ww as f64 * 0.42 {
            right = self.f_tiny.render(&right_txt, true, p.text_dim);
        }
        let rr = right.get_rect().midright_at((ww - pad, cy));
        s.blr(&right, rr);
        let Some(rnd) = self.round.as_ref() else {
            return;
        };
        let mid_txt = format!("{}: {}", t("cas.bet"), self.group(l::stake_total(rnd)));
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
            "music" => self.music_on,
            "auto" => self.window == Some(Win::Auto) || self.auto.is_some(),
            "stats" => self.window == Some(Win::Stats),
            _ => self.window == Some(Win::Fair),
        }
    }

    fn draw_tools(&self, s: &mut Surface) {
        let p = ui::pal();
        let hot = self.mouse;
        for (name, r) in self.r_tools.iter() {
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
            let icon = if *name != "music" || self.music_on { name } else { "mute" };
            let img = d::icon(icon, 8.max((r.h as f64 * 0.6) as i32), col);
            s.blr(&img, img.get_rect().center_at((r.x + r.h / 2, r.centery())));
            if self.tool_labels {
                let lab = self.f_tiny.render(&t(tool_label(name)), true, if active || h { p.text } else { p.text_dim });
                s.blr(&lab, lab.get_rect().midleft_at((r.x + r.h - 2.max(r.h / 8), r.centery())));
            }
        }
    }

    fn draw_tooltip(&self, s: &mut Surface) {
        if self.window.is_some() {
            return;
        }
        for (name, r) in self.r_tools.iter() {
            if r.hit(self.mouse) {
                let line = format!("{} ({})", t(tool_label(name)), tool_key(name));
                charts::tooltip(s, self.mouse, &[line], None);
                return;
            }
        }
        if self.r_half.hit(self.mouse) && self.state == St::Fly {
            charts::tooltip(s, self.mouse, &[format!("{} (H)", t("lcx.half_tip"))], None);
        }
    }

    /// Letzte Platzpunkte als Pillen oben rechts (neueste rechts).
    fn draw_history(&self, s: &mut Surface) {
        let mut x = self.b.width - self.margin;
        let y = self.r_tools[0].1.y;
        let h = self.tool_h;
        let ph = (h as f64 * 0.78) as i32;
        let limit = self.tools_right + self.margin;
        for &m100 in self.hist.iter().rev() {
            let label = self.fmt_short(m100);
            let wd = self.f_tiny.width(&label) + (12.0 * self.k) as i32 + 6;
            if x - wd < limit {
                break;
            }
            let img = d::pill(wd, ph, m100, &label, self.fs_tiny);
            s.bl(&img, (x - wd, y + py::fdiv32(h - ph, 2)));
            x -= wd + 4;
        }
    }

    // ----- Bedienleiste

    fn draw_strip(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.strip_bg, (self.strip.x, self.strip.y));
        draw::line(
            s,
            ui::mix(cd::COL_GOLD, self.b.accent, 0.35),
            (self.strip.x, self.strip.y),
            (self.strip.right(), self.strip.y),
            2,
        );
        let hot = self.mouse;
        let st = self.state;
        // Gesamteinsatz
        let r = self.r_total;
        draw::rect_r(s, (16, 12, 26), r, 0, 6.max(r.h / 4));
        draw::rect_r(s, p.border, r, 1, 6.max(r.h / 4));
        let lbl = fit(&t("lcx.total"), &[&self.f_tiny, &self.f_micro], r.w - 4, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midtop_at((r.centerx(), r.y + 2)));
        let total = match self.round.as_ref() {
            Some(rnd) => l::stake_total(rnd),
            None => self.total(),
        };
        let col = if self.round.is_none() && total > lamabank::balance() { p.red } else { p.gold };
        let val = fit(&self.group(total), &[&self.f_btn, &self.f_tiny], r.w - 4, col);
        s.blr(&val, val.get_rect().midbottom_at((r.centerx(), r.bottom() - 3)));
        // Haelfte
        let can_half = st == St::Fly
            && self.round.as_ref().is_some_and(|rnd| {
                l::mult_at(rnd.t) >= l::MIN_CASH && rnd.bets.iter().any(|b| l::riding(b) > 0 && b.hm == 0)
            });
        let half_lab = if self.half_label { t("lcx.half_all") } else { String::new() };
        self.btn(s, self.r_half, &half_lab, false, can_half, self.r_half.hit(hot), false, false, Some("half"), false);
        // Status in der Mitte
        let (x0, x1) = self.status_x;
        let (status, scol) = if let Some(a) = self.auto.as_ref() {
            (tf("chip.auto_left", &[("n", &a.left.to_string())]), self.b.accent)
        } else if st == St::Count {
            (t("lcx.ready"), p.text_dim)
        } else if st == St::Coast {
            (t("lcx.coast"), p.text_dim)
        } else {
            (String::new(), p.text_dim)
        };
        if !status.is_empty() && x1 - x0 > 30 {
            let img = fit(&status, &[&self.f_btn, &self.f_tiny, &self.f_micro], x1 - x0, scol);
            s.blr(&img, img.get_rect().center_at((py::fdiv32(x0 + x1, 2), self.r_go.centery())));
        }
        // Hauptknopf
        let go = self.r_go;
        match (st, self.round.as_ref()) {
            (St::Fly, Some(rnd)) => {
                let m = l::mult_at(rnd.t);
                let mut now = 0;
                for b in &rnd.bets {
                    now += py::fdiv(l::riding(b) * m, 100);
                }
                let lab = format!("{} {}", t("lcx.cash_all"), self.group(now));
                self.btn(s, go, &lab, true, m >= l::MIN_CASH, go.hit(hot), false, false, None, false);
            }
            _ if st == St::Coast => {
                self.btn(
                    s,
                    go,
                    &t("lcx.ff"),
                    !self.ff && self.auto.is_none(),
                    true,
                    go.hit(hot),
                    false,
                    false,
                    Some("ff"),
                    false,
                );
            }
            _ if self.auto.is_some() => {
                self.btn(s, go, &t("chip.auto_stop"), true, true, go.hit(hot), false, false, None, true);
            }
            _ => {
                let ready = st == St::Idle || (st == St::End && self.end_t >= READ_T);
                self.btn(
                    s,
                    go,
                    &t("lcx.play"),
                    true,
                    ready && !self.broke && self.window.is_none() && self.round.is_none(),
                    go.hit(hot),
                    false,
                    false,
                    Some("play"),
                    false,
                );
            }
        }
    }

    /// `_btn(s, rect, label, primary, on, hot, active, small, icon, danger)`
    #[allow(clippy::too_many_arguments)]
    fn btn(
        &self,
        s: &mut Surface,
        r: Rect,
        label: &str,
        primary: bool,
        on: bool,
        hot: bool,
        active: bool,
        small: bool,
        icon: Option<&str>,
        danger: bool,
    ) {
        let p = ui::pal();
        let rad = 5.max(r.h / 4);
        let col = if primary && on {
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
        let tx = r.centerx();
        if let Some(icon) = icon {
            let isz = 8.max((r.h as f64 * 0.56) as i32);
            let img = d::icon(icon, isz, col);
            if !label.is_empty() {
                let limg = if small {
                    fit(label, &[&self.f_tiny, &self.f_micro], 10.max(r.w - isz - 14), col)
                } else {
                    fit(label, &[&self.f_btn, &self.f_tiny, &self.f_micro], 10.max(r.w - isz - 14), col)
                };
                let total = isz + 5 + limg.w();
                let ix = r.centerx() - total / 2;
                s.blr(&img, img.get_rect().midleft_at((ix, r.centery())));
                s.blr(&limg, limg.get_rect().midleft_at((ix + isz + 5, r.centery())));
            } else {
                s.blr(&img, img.get_rect().center_at(r.center()));
            }
            return;
        }
        if !label.is_empty() {
            let img = if small {
                fit(label, &[&self.f_tiny, &self.f_micro], r.w - 6, col)
            } else {
                fit(label, &[&self.f_btn, &self.f_tiny, &self.f_micro], r.w - 6, col)
            };
            s.blr(&img, img.get_rect().center_at((tx, r.centery())));
        }
    }

    // ----- Effekte, Hinweise

    fn draw_fx(&self, s: &mut Surface) {
        for c in &self.coins_fx {
            let wd = 2.max((c.r * 2.0 * c.spin.cos().abs()) as i32);
            let h = (c.r * 2.0) as i32;
            let rect = Rect::new(0, 0, wd, h).center_at((c.x as i32, c.y as i32));
            draw::ellipse(s, cd::COL_GOLD_D, rect.move_(1, 1), 0);
            draw::ellipse(s, (246, 196, 60), rect, 0);
            if wd > 6 {
                draw::ellipse(s, (255, 236, 150), rect.inflate(py::fdiv32(-wd, 2), py::fdiv32(-h, 2)), 0);
            }
        }
        for pp in &self.popups {
            let f = pp.t / pp.dur;
            let mut img = self.f_big.render(&pp.text, true, pp.color);
            let mut sh = self.f_big.render(&pp.text, true, (10, 6, 16));
            if f > 0.6 {
                let a = (255.0 * (1.0 - (f - 0.6) / 0.4)) as i32;
                img = img.copy();
                img.alpha_set(a);
                sh = sh.copy();
                sh.alpha_set(a);
            }
            let y = pp.y - 40.0 * self.k * (1.0 - (1.0 - f) * (1.0 - f));
            let mut r = img.get_rect().center_at((pp.x as i32, y as i32));
            r.clamp_ip(self.stage);
            s.blr(&sh, r.move_(1, 1));
            s.blr(&img, r);
        }
    }

    fn draw_banner(&self, s: &mut Surface) {
        let Some(bn) = self.banner.as_ref() else {
            return;
        };
        let p = ui::pal();
        let st = self.stage;
        let k = self.k;
        let grow = pmin(1.0, (bn.dur - bn.left) / 0.18);
        let pulse = 0.5 + 0.5 * (self.time * 8.0).sin();
        let col = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.4 * pulse);
        let timg = fit(&bn.title, &[&self.f_huge, &self.f_mult, &self.f_big], st.w - 24, col);
        let simg = fit(&bn.sub, &[&self.f_big, &self.f_btn, &self.f_tiny], st.w - 24, p.text);
        let pw = (st.w - 8).min(timg.w().max(simg.w()) + (40.0 * k) as i32);
        let ph = timg.h() + simg.h() + (26.0 * k) as i32;
        let panel = Rect::new(0, 0, (pw as f64 * (0.7 + 0.3 * grow)) as i32, (ph as f64 * (0.7 + 0.3 * grow)) as i32)
            .center_at((st.centerx(), st.y + (st.h as f64 * 0.32) as i32));
        let (sw, shh) = s.get_size();
        let mut layer = pg::surface_alpha(sw as i32, shh as i32);
        ui::draw_panel_ex(&mut layer, panel, None, None, None, true, Some(cd::COL_GOLD.c()));
        if grow >= 1.0 {
            layer.blr(&timg, timg.get_rect().midtop_at((panel.centerx(), panel.y + (8.0 * k) as i32)));
            layer.blr(&simg, simg.get_rect().midbottom_at((panel.centerx(), panel.bottom() - (8.0 * k) as i32)));
        }
        if bn.left < 0.4 {
            layer.alpha_set((255.0 * bn.left / 0.4) as i32);
        }
        s.bl(&layer, (0, 0));
    }

    fn draw_toast(&self, s: &mut Surface) {
        let Some((text, left)) = self.toast.as_ref() else {
            return;
        };
        let p = ui::pal();
        let mut img = fit(text, &[&self.f_small, &self.f_tiny], self.b.width - 40, p.text);
        let mut bx = img.get_rect();
        bx.inflate_ip(24, 12);
        bx.set_midbottom((self.stage.centerx(), self.stage.bottom() - self.margin));
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

    fn draw_broke(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let pw = (self.b.width - 40).min((480.0 * pmax(0.8, self.k)) as i32);
        let ph = (200.0 * pmax(0.75, self.k)) as i32;
        let panel = Rect::new(cx - pw / 2, cy - ph / 2, pw, ph);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(p.red));
        let head = fit(&t("cas.broke"), &[&self.f_huge, &self.f_big], pw - 16, p.red);
        s.blr(&head, head.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * 0.28)));
        let lines = [
            (t("cas.broke_sub"), p.text_dim),
            (tf("cas.broke_restart", &[("n", &lamabank::START_CHIPS.to_string())]), p.text),
        ];
        for (i, (txt, col)) in lines.iter().enumerate() {
            let img = fit(txt, &[&self.f_small, &self.f_tiny, &self.f_micro], pw - 16, *col);
            s.blr(&img, img.get_rect().center_at((cx as f64, panel.y as f64 + ph as f64 * (0.58 + 0.2 * i as f64))));
        }
    }

    // ----- Auto-Fenster (ueber der Buehne)

    fn draw_auto(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.auto_r;
        ui::draw_panel_ex(s, r, None, None, None, true, Some(self.b.accent));
        let k = self.k;
        let pad = 6.max((10.0 * k) as i32);
        let (x0, wd) = (r.x + pad, r.w - 2 * pad);
        let hot = self.mouse;
        let icon = d::icon("auto", 10.max(self.r_auto_close.h - 4), self.b.accent);
        s.blr(&icon, icon.get_rect().midleft_at((x0, self.r_auto_close.centery())));
        let title = fit(
            &t("chip.auto_title"),
            &[&self.f_big, &self.f_btn],
            self.r_auto_close.x - x0 - icon.w() - 10,
            p.text,
        );
        s.blr(&title, title.get_rect().midleft_at((x0 + icon.w() + 6, self.r_auto_close.centery())));
        self.btn(s, self.r_auto_close, "", false, true, self.r_auto_close.hit(hot), false, false, Some("close"), false);
        let labels = [t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
        let values = [self.auto_rounds, self.auto_win, self.auto_loss];
        let running = self.auto.is_some();
        for (row, (ly, rects)) in self.auto_rows.iter().enumerate() {
            let img = fit(
                &labels[row],
                &[&self.f_tiny, &self.f_micro],
                wd,
                if self.auto_focus == row as i32 { p.text } else { p.text_dim },
            );
            s.bl(&img, (x0, *ly));
            let opts: &[i64] = if row == 0 { &ROUND_OPTS } else { &LIMIT_OPTS };
            for (j, rr) in rects.iter().enumerate() {
                let v = opts[j];
                let lab = if row == 0 {
                    v.to_string()
                } else if v == 0 {
                    t("chip.auto_off")
                } else {
                    cd::fmt_amount(v)
                };
                self.btn(
                    s,
                    *rr,
                    &lab,
                    false,
                    !running || v == values[row],
                    rr.hit(hot) && !running,
                    v == values[row],
                    true,
                    None,
                    false,
                );
            }
        }
        let tr = self.r_auto_turbo;
        self.btn(
            s,
            tr,
            &t("chip.turbo"),
            false,
            true,
            tr.hit(hot) || self.auto_focus == 3,
            self.turbo,
            false,
            Some("turbo"),
            false,
        );
        let parts: Vec<String> = self.active().iter().map(|&i| self.fmt(self.targets[i])).collect();
        let txt = tf(
            "lcx.auto_sum",
            &[("n", &self.auto_rounds.to_string()), ("b", &self.group(self.total())), ("x", &parts.join(" / "))],
        );
        let mut y = self.auto_sum_y;
        let scol = if self.total() > lamabank::balance() { p.red } else { p.text_dim };
        for line in wrap(&txt, &self.f_tiny, wd).iter().take(2) {
            let img = self.f_tiny.render(line, true, scol);
            s.bl(&img, (x0, y));
            y += self.f_tiny.get_height();
        }
        let note = t("lcx.auto_note");
        for line in wrap(&note, &self.f_micro, wd).iter() {
            if y + self.f_micro.get_height() >= self.r_auto_start.y - 4 {
                break;
            }
            let img = self.f_micro.render(line, true, p.text_faint);
            s.bl(&img, (x0, y));
            y += self.f_micro.get_height();
        }
        if running {
            self.btn(
                s,
                self.r_auto_start,
                &t("chip.auto_stop"),
                true,
                true,
                self.r_auto_start.hit(hot),
                false,
                false,
                None,
                true,
            );
        } else {
            self.btn(
                s,
                self.r_auto_start,
                &t("chip.auto_start"),
                true,
                !self.broke,
                self.r_auto_start.hit(hot),
                false,
                false,
                None,
                false,
            );
        }
    }

    // ----- Statistik- und Fairness-Fenster

    fn window_rect(&self) -> (Rect, Rect) {
        let (ww, hh, k, m) = (self.b.width, self.b.height, self.k, self.margin);
        let (wd, h) = if self.window == Some(Win::Fair) {
            ((ww - 2 * m).min((720.0 * k) as i32), (hh - 2 * m).min((540.0 * k) as i32))
        } else {
            ((ww - 2 * m).min((680.0 * k) as i32), (hh - 2 * m).min((500.0 * k) as i32))
        };
        let rect = Rect::new(0, 0, wd, h).center_at((ww / 2, hh / 2));
        let cb = 20.max((28.0 * k) as i32);
        let close = Rect::new(rect.right() - self.ppad - cb, rect.y + self.ppad, cb, cb);
        (rect, close)
    }

    fn draw_window(&mut self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (rect, close) = self.window_rect();
        ui::draw_panel_ex(s, rect, None, None, None, true, Some(self.b.accent));
        let pad = self.ppad;
        let win = self.window.unwrap_or(Win::Stats);
        let icon = d::icon(win.name(), 10.max(close.h - 6), self.b.accent);
        s.blr(&icon, icon.get_rect().midleft_at((rect.x + pad, close.centery())));
        let mut title = if win == Win::Stats { t("chip.stats") } else { t("chip.fair_title") };
        title += "  \u{00B7}  Lama Crash";
        let img = fit(&title, &[&self.f_big, &self.f_btn], close.x - rect.x - icon.w() - 3 * pad, p.text);
        s.blr(&img, img.get_rect().midleft_at((rect.x + pad + icon.w() + pad / 2, close.centery())));
        self.btn(s, close, "", false, true, close.hit(self.mouse), false, false, Some("close"), false);
        let body = Rect::new(
            rect.x + pad,
            close.bottom() + pad,
            rect.w - 2 * pad,
            rect.bottom() - close.bottom() - 2 * pad,
        );
        if win == Win::Stats {
            self.draw_stats(s, body);
        } else {
            self.draw_fair(s, body);
        }
    }

    fn xint(&self, name: &str) -> i64 {
        // stats["x"] ist nach clean_x immer gueltig (ganze Zahlen >= 0)
        self.xs.counter(name)
    }

    fn draw_stats(&mut self, s: &mut Surface, body: Rect) {
        let p = ui::pal();
        let k = self.k;
        let st = self.stats.clone();
        let mut rtp = "\u{2013}".to_string();
        if st.wagered > 0 {
            let pv = py::fdiv(st.paid * 20000 + st.wagered, st.wagered * 2);
            rtp = format!("{}{}{:02} %", self.group(py::fdiv(pv, 100)), self.sep(), py::pmod(pv, 100));
        }
        let net = st.net;
        let top_x = self.xint("top");
        let tiles: Vec<(String, String, Color)> = vec![
            (t("chip.st_rounds"), self.group(st.rounds), p.text),
            (t("chip.st_wins"), self.group(st.wins), p.green),
            (t("chip.st_losses"), self.group(st.losses), p.red),
            (t("chip.st_rtp"), rtp, p.text),
            (t("chip.st_wagered"), self.group(st.wagered), p.text),
            (t("chip.st_paid"), self.group(st.paid), p.text),
            (
                t("chip.st_net"),
                format!("{}{}", if net > 0 { "+" } else { "" }, self.group(net)),
                if net >= 0 { p.green } else { p.red },
            ),
            (
                t("chip.st_best_win"),
                format!("{}{}", if st.best_win > 0 { "+" } else { "" }, self.group(st.best_win)),
                cd::COL_GOLD.c(),
            ),
            (
                t("chip.st_best_mult"),
                if st.best_mult > 0 { self.fmt(st.best_mult) } else { "\u{2013}".to_string() },
                cd::COL_GOLD.c(),
            ),
            (t("lcx.st_top"), if top_x != 0 { self.fmt(top_x) } else { "\u{2013}".to_string() }, self.b.accent),
            (t("lcx.st_half"), self.group(self.xint("half")), p.text),
            (t("lcx.st_close"), self.group(self.xint("close")), cd::COL_GOLD.c()),
        ];
        let cols = 4;
        let gap = 4.max((8.0 * k) as i32);
        let tw = py::fdiv32(body.w - (cols - 1) * gap, cols);
        let th = self.f_tiny.get_height() + self.f_big.get_height() + 8.max((12.0 * k) as i32);
        for (i, (lab, val, col)) in tiles.iter().enumerate() {
            let i = i as i32;
            let r = Rect::new(body.x + (i % cols) * (tw + gap), body.y + (i / cols) * (th + gap), tw, th);
            draw::rect_r(s, ui::mix(p.panel, (0, 0, 0), 0.25), r, 0, 4.max((8.0 * k) as i32));
            draw::rect_r(s, p.border, r, 1, 4.max((8.0 * k) as i32));
            let limg = fit(lab, &[&self.f_tiny, &self.f_micro], tw - 8, p.text_dim);
            s.bl(&limg, (r.x + 5, r.y + 3.max((4.0 * k) as i32)));
            let vimg = fit(val, &[&self.f_big, &self.f_btn, &self.f_tiny], tw - 8, *col);
            s.blr(&vimg, vimg.get_rect().bottomleft_at((r.x + 5, r.bottom() - 3.max((4.0 * k) as i32))));
        }
        let y = body.y + 3 * (th + gap) + gap;
        let half = py::fdiv32((body.w - gap) * 45, 100);
        let left = Rect::new(body.x, y, half, body.bottom() - y);
        let right = Rect::new(left.right() + gap, y, body.right() - left.right() - gap, body.bottom() - y);
        // Bilanzkurve
        let curve = st.curve.clone();
        let lab = if !curve.is_empty() {
            tf("chip.st_curve", &[("n", &curve.len().to_string())])
        } else {
            t("chip.st_empty")
        };
        let img = fit(&lab, &[&self.f_tiny, &self.f_micro], left.w, p.text_dim);
        s.bl(&img, (left.x, left.y));
        let cy = left.y + img.h() + 4 + charts::fnt(10, false).get_height();
        let chart = Rect::new(left.x, cy, left.w, left.bottom() - cy);
        if curve.len() >= 2 && chart.h > 20 {
            let mut vals: Vec<f64> = Vec::new();
            if curve.len() < chipstats::CURVE {
                vals.push(0.0);
            }
            vals.extend(curve.iter().map(|v| *v as f64));
            let fmtf = |v: f64| self.group(v as i64);
            charts::line(s, chart, &vals, Some(self.b.accent), &[], Some(&fmtf), None);
        }
        // Verteilung der Platzpunkte gegen die Erwartung
        let hits: Vec<i64> = self.xs.hits.clone();
        let mut total = 0;
        for v in &hits {
            total += v;
        }
        let lab = t("lcx.st_dist");
        let img = fit(&lab, &[&self.f_tiny, &self.f_micro], right.w, p.text_dim);
        s.bl(&img, (right.x, right.y));
        let mut area = Rect::new(right.x, right.y + img.h() + 4, right.w, right.h - img.h() - 4);
        let leg = fit(&t("lcx.st_legend"), &[&self.f_micro], area.w, p.text_faint);
        s.blr(&leg, leg.get_rect().bottomleft_at((area.x, area.bottom())));
        area.h -= leg.h() + 4;
        if total <= 0 || area.h < 24 {
            if area.h >= 14 {
                let img = fit(&t("lcx.st_none"), &[&self.f_small, &self.f_tiny], area.w, p.text_dim);
                s.blr(&img, img.get_rect().center_at(area.center()));
            }
            return;
        }
        let exp: Vec<f64> = (0..l::BUCKETS.len()).map(|i| total as f64 * l::bucket_prob_f(i)).collect();
        let max_hits = hits.iter().copied().max().unwrap_or(0) as f64;
        let mut max_exp = exp[0];
        for e in exp.iter().skip(1) {
            max_exp = pmax(max_exp, *e);
        }
        let top = pmax(max_hits, max_exp);
        let labels: Vec<String> = (0..l::BUCKETS.len()).map(|i| self.bucket_label(i)).collect();
        let cols_c: Vec<Color> = l::BUCKETS.iter().map(|lo| ui::mix(d::pill_color(*lo).0, p.panel, 0.2)).collect();
        let lsz = 9.max((10.0 * k) as i32);
        let vals: Vec<f64> = hits.iter().map(|v| *v as f64).collect();
        let hits_r = charts::bars_ex(
            s,
            area,
            &vals,
            Some(&labels),
            Some(&cols_c),
            None,
            Some(top),
            false,
            None,
            self.stats_hover,
            lsz,
        );
        let fnt_h = ui::font(lsz, false, false).get_height() + 2;
        let val_h = ui::font(lsz, true, false).get_height() + 2;
        let plot_h = 8.max(area.h - fnt_h - val_h);
        let mut pts: Vec<(i32, i32)> = Vec::new();
        let mut hover = None;
        for (cell, i) in hits_r.iter() {
            let yy = (area.y + val_h + plot_h) as f64 - plot_h as f64 * (exp[*i] / top);
            pts.push((cell.centerx(), yy as i32));
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
            let e = (exp[hv] * 10.0 + 0.5) as i64;
            let lines = vec![
                labels[hv].clone(),
                tf(
                    "lcx.st_bar",
                    &[
                        ("n", &hits[hv].to_string()),
                        ("e", &format!("{}{}{}", py::fdiv(e, 10), self.sep(), py::pmod(e, 10))),
                    ],
                ),
            ];
            charts::tooltip(s, self.mouse, &lines, None);
        }
    }

    fn bucket_label(&self, i: usize) -> String {
        let lo = l::BUCKETS[i];
        if i + 1 >= l::BUCKETS.len() {
            return format!("{}+", self.fmt_short(lo));
        }
        format!("{}\u{2013}{}", self.fmt_short(lo), l::fmt_short(l::BUCKETS[i + 1], &self.sep()))
    }

    fn draw_fair(&mut self, s: &mut Surface, body: Rect) {
        let p = ui::pal();
        let k = self.k;
        let gap = 4.max((7.0 * k) as i32);
        let mut y = body.y;
        for line in wrap(&t("lcx.fair_info"), &self.f_micro, body.w).iter().take(3) {
            let img = self.f_micro.render(line, true, p.text_dim);
            s.bl(&img, (body.x, y));
            y += self.f_micro.get_height();
        }
        y += gap;
        let mut rows: Vec<(String, String, Color)> = Vec::new();
        if let Some(rnd) = self.round.as_ref() {
            rows.push((t("lcx.fair_current"), rnd.h.clone(), p.text));
        }
        rows.push((t("chip.fair_next"), fairness::commit(&self.fair.server), self.b.accent));
        for (lab, hx, col) in rows.iter() {
            let img = fit(lab, &[&self.f_tiny, &self.f_micro], body.w, p.text_dim);
            s.bl(&img, (body.x, y));
            y += img.h();
            y = self.draw_hex(s, hx, body.x, y, body.w, *col);
            y += 2;
        }
        let bh = 22.max((28.0 * k) as i32);
        let lab = format!(
            "{}: {}   \u{00B7}   {} {}",
            t("chip.fair_client"),
            self.fair.client,
            t("chip.fair_nonce"),
            self.fair.nonce
        );
        let bw = (self.f_btn.width(&t("chip.fair_new_client")) + (20.0 * k) as i32).min(body.w / 2);
        self.r_fair_new = Rect::new(body.right() - bw, y, bw, bh);
        let img = fit(&lab, &[&self.f_tiny, &self.f_micro], body.w - bw - gap, p.text);
        s.blr(&img, img.get_rect().midleft_at((body.x, y + bh / 2)));
        let rn = self.r_fair_new;
        self.btn(s, rn, &t("chip.fair_new_client"), false, self.round.is_none(), rn.hit(self.mouse), false, false, None, false);
        y += bh + gap;
        draw::line(s, p.border, (body.x, y), (body.right(), y), 1);
        y += gap;
        let area = Rect::new(body.x, y, body.w, body.bottom() - y - self.f_micro.get_height() - 2);
        let note = fit(&t("chip.fair_note"), &[&self.f_micro], body.w, p.text_faint);
        s.bl(&note, (body.x, body.bottom() - note.h()));
        self.r_fair_rows.clear();
        self.r_fair_verify = Rect::new(0, 0, 0, 0);
        if self.fairlog.is_empty() {
            let img = fit(&t("chip.fair_empty"), &[&self.f_small, &self.f_tiny], area.w, p.text_dim);
            s.blr(&img, img.get_rect().center_at(area.center()));
            return;
        }
        let lw = ((area.w as f64 * 0.40) as i32).max((area.w / 2).min((220.0 * k) as i32));
        let lst = Rect::new(area.x, area.y, lw, area.h);
        let det = Rect::new(lst.right() + gap, area.y, area.w - lw - gap, area.h);
        let rh = (self.f_tiny.get_height() + 6).max((24.0 * k) as i32);
        let nvis = 1.max(py::fdiv32(lst.h, rh)) as i64;
        let order = self.fair_order();
        let n_order = order.len() as i64;
        self.fair_scroll = 0.max(self.fair_scroll.min(n_order - nvis));
        if let Some(pos) = self.fair_sel.and_then(|sel| order.iter().position(|v| *v == sel)) {
            let pos = pos as i64;
            if pos < self.fair_scroll {
                self.fair_scroll = pos;
            } else if pos >= self.fair_scroll + nvis {
                self.fair_scroll = pos - nvis + 1;
            }
        }
        let a0 = (self.fair_scroll.max(0) as usize).min(order.len());
        let a1 = ((self.fair_scroll + nvis).max(0) as usize).min(order.len());
        for (row, &idx) in order[a0..a1.max(a0)].iter().enumerate() {
            let e = self.fairlog[idx].clone();
            let r = Rect::new(lst.x, lst.y + row as i32 * rh, lst.w, rh - 2);
            let sel = Some(idx) == self.fair_sel;
            draw::rect_r(
                s,
                if sel || r.hit(self.mouse) { p.btn_sel } else { ui::mix(p.panel, (0, 0, 0), 0.2) },
                r,
                0,
                3.max(rh / 4),
            );
            if sel {
                draw::rect_r(s, self.b.accent, r, 1, 3.max(rh / 4));
            }
            let x = e.o.as_object().and_then(|o| py_int(o.get("x")));
            let res = match x {
                Some(v) if v >= 100 => self.fmt_short(v),
                _ => "?".to_string(),
            };
            let num = self.f_tiny.render(&format!("#{}", e.n), true, p.text_dim);
            s.blr(&num, num.get_rect().midleft_at((r.x + 5, r.centery())));
            let (bg, _fg) = d::pill_color(x.unwrap_or(100));
            let rimg = self.f_tiny.render(&res, true, ui::mix(bg, (255, 255, 255), 0.35));
            s.blr(&rimg, rimg.get_rect().midright_at((r.right() - 5, r.centery())));
            let hx0 = r.x + 5 + num.w().max(self.f_tiny.width("#0000")) + 6;
            let hx1 = r.right() - 5 - rimg.w().max(self.f_tiny.width("\u{00D7}00,00")) - 6;
            if hx1 - hx0 > 20 {
                let himg = fit(&fairness::short(&e.h, 10), &[&self.f_mono], hx1 - hx0, p.text_faint);
                s.blr(&himg, himg.get_rect().midleft_at((hx0, r.centery())));
            }
            if let Some(chk) = self.fair_checks.get(&e.n) {
                let isz = rh / 2;
                let img = d::icon(if chk.0 { "check" } else { "cross" }, isz, if chk.0 { p.green } else { p.red });
                s.bl(&img, (r.right() - rimg.w() - 10 - isz, r.y + py::fdiv32(r.h - isz, 2)));
            }
            self.r_fair_rows.push((idx, r));
        }
        if n_order > nvis {
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
    fn draw_hex(&self, s: &mut Surface, hx: &str, x: i32, mut y: i32, wd: i32, col: Color) -> i32 {
        let cw = 1.max(self.f_mono.width("0"));
        let per = 8.max(py::fdiv32(wd, cw)) as usize;
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

    fn draw_fair_detail(&mut self, s: &mut Surface, det: Rect, e: &LogEntry) {
        let p = ui::pal();
        let k = self.k;
        let (x, mut y, wd) = (det.x, det.y, det.w);
        let mut head = format!("#{}", e.n);
        if let Some(ox) = e.o.as_object().and_then(|o| py_int(o.get("x"))) {
            head += &format!("{}{}", DOT, tf("lcx.fair_round", &[("x", &self.fmt(ox))]));
        }
        let img = fit(&head, &[&self.f_btn, &self.f_tiny], wd, p.text);
        s.bl(&img, (x, y));
        y += img.h() + 2;
        for (lab, val) in [(t("chip.fair_hash"), e.h.clone()), (t("chip.fair_server"), e.s.clone())] {
            let img = fit(&lab, &[&self.f_micro], wd, p.text_dim);
            s.bl(&img, (x, y));
            y += img.h();
            y = self.draw_hex(s, &val, x, y, wd, p.text);
        }
        let img = fit(&format!("{}: {}", t("chip.fair_client"), e.c), &[&self.f_tiny, &self.f_micro], wd, p.text);
        s.bl(&img, (x, y + 1));
        y += img.h() + 4;
        let bh = 22.max((28.0 * k) as i32);
        let bw = wd.min(self.f_btn.width(&t("chip.fair_verify")) + (24.0 * k) as i32);
        self.r_fair_verify = Rect::new(x, y, bw, bh);
        let rv = self.r_fair_verify;
        self.btn(s, rv, &t("chip.fair_verify"), true, true, rv.hit(self.mouse), false, false, None, false);
        let Some((ok, crash, marks)) = self.fair_checks.get(&e.n).cloned() else {
            return;
        };
        let isz = py::fdiv32(bh * 2, 3);
        let img = d::icon(if ok { "check" } else { "cross" }, isz, if ok { p.green } else { p.red });
        s.bl(&img, (rv.right() + 6, y + py::fdiv32(bh - isz, 2)));
        let msg = if ok { t("chip.fair_ok") } else { t("chip.fair_bad") };
        let timg = fit(&msg, &[&self.f_btn, &self.f_tiny], 10.max(det.right() - rv.right() - isz - 14), if ok { p.green } else { p.red });
        s.blr(&timg, timg.get_rect().midleft_at((rv.right() + isz + 10, y + bh / 2)));
        y += bh + 6;
        let info = tf("lcx.fair_calc", &[("x", &self.fmt(crash))]);
        let img = fit(&info, &[&self.f_tiny, &self.f_micro], wd, p.text);
        s.bl(&img, (x, y));
        y += img.h() + 4;
        let ph = det.bottom() - y;
        if ph >= 24 && wd >= 40 {
            let img = d::curve_preview(wd, ph.min((wd as f64 * 0.6) as i32), crash, &marks);
            s.bl(&img, (x, y));
        }
    }
}

impl Game for LamaCrashGame {
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

    fn reset(&mut self) {
        self.b.game_over = false;
        lamabank::load();
        self.seed_rng = py::Random::unseeded(); // nur Fairness-Seeds
        self.rng = py::Random::unseeded(); // Optik (Partikel, Muenzen ...)
        self.normalize_opts();
        self.bets =[self.choice_opt("bet1", &l::BETS, l::BETS[0]), self.choice_opt("bet2", &l::BETS, l::BETS[0])];
        self.autos = [self.opt_true("auto1"), self.opt_true("auto2")];
        self.targets = [self.target_opt("target1", l::DEFAULT_TARGET), self.target_opt("target2", 1000)];
        self.two = self.opt_true("two");
        self.turbo = self.opt_true("turbo");
        self.music_on = self.opt_true("music");
        self.auto_rounds = self.choice_opt("auto_rounds", &ROUND_OPTS, 10);
        self.auto_win = self.choice_opt("auto_win", &LIMIT_OPTS, 0);
        self.auto_loss = self.choice_opt("auto_loss", &LIMIT_OPTS, 0);
        let extra = match lamabank::get_extra(KEY) {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        self.hist = l::clean_history(extra.get("hist"));
        {
            let rng = &mut self.seed_rng;
            self.fair = fairness::clean_state(extra.get("fair"), &mut |k| rng.getrandbits(k));
        }
        self.fairlog = fairness::clean_log(extra.get("fairlog"));
        self.stats = chipstats::clean(extra.get("stats"));
        self.xs = l::clean_x(Some(&Value::Object(self.stats.x.clone())));
        self.stats.x = self.xs.to_map();
        self.carry = l::clean_carry(extra.get("carry")); // Rest-Hundertstel der Auszahlungen
        self.round = l::clean_round(extra.get("round"));
        self.crash = None; // Platzpunkt der laufenden Runde (nur advance/finish!)
        self.state = St::Idle;
        self.sel = 0;
        self.acc = 0.0;
        self.count_t = 0.0;
        self.count_len = COUNT_T;
        self.ff = false;
        self.end = None;
        self.end_t = 0.0;
        self.last = None;
        self.tone_i = 0;
        self.world_seen = 0;
        self.world_label = None;
        self.squeak_t = 0.0;
        self.flame_t = 0.0;
        self.env_rect = Rect::new(0, 0, 0, 0);
        self.chutes.clear();
        self.fallers.clear();
        self.shreds.clear();
        self.pop_fx = None;
        self.basket_fx = None;
        self.particles.clear();
        self.coins_fx.clear();
        self.popups.clear();
        self.toast = None;
        self.banner = None;
        self.shake = 0.0;
        self.time = 0.0;
        self.mouse = (-1, -1);
        self.broke = false;
        self.window = None;
        self.auto = None;
        self.auto_focus = 0;
        self.fair_sel = None;
        self.fair_scroll = 0;
        self.fair_checks.clear();
        self.r_fair_rows.clear();
        self.r_fair_new = Rect::new(0, 0, 0, 0);
        self.r_fair_verify = Rect::new(0, 0, 0, 0);
        self.stats_hover = None;
        self.mus_stage = None;
        self.mus_t = 0.0;
        self.mus_vol = None;
        self.mus_fail = None;
        self.pending = 0;
        self.shown_chips = lamabank::balance() as f64;
        self.tick_cd = 0.0;
        self.lang = i18n::get_language();
        self.layout();
        let gleich = extra.get("fair").is_some_and(|v| py_eq(v, &self.fair.to_value()));
        if !gleich {
            // frisch gewuerfelte Seeds sofort sichern: der gezeigte Pruefcode bleibt gueltig
            self.save_extra();
        }
        if let Some(rnd) = self.round.as_ref() {
            self.crash = Some(l::crash_point(&rnd.s, &rnd.c, rnd.n));
            if l::flying(rnd) {
                // Unterbrochene Runde: Countdown, dann geht es am selben Takt weiter
                let m = l::mult_at(rnd.t);
                self.two = rnd.bets.len() > 1;
                self.state = St::Count;
                self.count_t = RESUME_T;
                self.count_len = RESUME_T;
                self.tone_i = Self::tone_index(m);
                self.world_seen = 4.min(l::world_of(m));
                self.say(t("lcx.resumed"), 2.4);
            } else {
                self.finish_now();
            }
        }
        if self.round.is_none() {
            self.clamp_bets();
        }
        self.b.score = lamabank::score_for(KEY);
        self.check_broke();
    }

    fn on_surface_changed(&mut self) {
        d::clear_cache();
        w::clear_cache();
        cd::clear_cache();
        self.layout();
    }

    /// Verlassen: fliegt noch Geld, wird der Takt gespeichert (Fortsetzen);
    /// sind alle ausgestiegen, wird die Runde sofort abgeschlossen.
    fn on_exit(&mut self) {
        audio::music_stop(Some(self.ident()));
        self.auto = None;
        if let Some(rnd) = self.round.as_ref() {
            if l::flying(rnd) {
                self.save_extra();
            } else {
                self.finish_now();
            }
        }
        self.b.score = lamabank::score_for(KEY);
    }

    /// Esc schliesst ein offenes Fenster bzw. stoppt den Auto-Modus.
    fn wants_escape(&self) -> bool {
        self.window.is_some() || self.auto.is_some()
    }

    /// Fokus weg, waehrend Geld fliegt -> Pause (der Ballon stiege sonst weiter).
    fn pause_on_blur(&self) -> bool {
        self.round.as_ref().is_some_and(|r| matches!(self.state, St::Count | St::Fly) && l::flying(r))
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        if matches!(kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            self.mouse = ev.pos;
        }
        if self.broke {
            if kind == Kind::MouseDown || (kind == Kind::KeyDown && is_action(&ev.key)) {
                self.take_credit();
            }
            return;
        }
        if kind == Kind::KeyDown {
            let key = ev.key.clone();
            self.on_key(&key, ev.repeat);
        } else if kind == Kind::Wheel {
            self.on_wheel(ev.delta);
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
        let target = 0.max(lamabank::balance() - self.pending);
        let diff = target as f64 - self.shown_chips;
        if diff.abs() < 0.5 {
            self.shown_chips = target as f64;
        } else {
            let step = pmax(diff.abs() * pmin(1.0, dt * 4.5), 60.0 * dt);
            self.shown_chips += pmin(diff.abs(), step).copysign(diff);
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
        if let Some(bn) = self.banner.as_mut() {
            bn.left -= dt;
            if bn.left <= 0.0 {
                self.banner = None;
            }
        }
        if let Some(wl) = self.world_label.as_mut() {
            wl.1 -= dt;
            if wl.1 <= 0.0 {
                self.world_label = None;
            }
        }
        self.shake = pmax(0.0, self.shake - dt);
        self.flame_t = pmax(0.0, self.flame_t - dt);
        let st = self.state;
        if st == St::Count {
            let seg = self.count_len / 3.0;
            let before = (self.count_t / seg) as i64;
            self.count_t -= dt;
            if self.count_t <= 0.0 {
                self.launch();
            } else if (self.count_t / seg) as i64 != before {
                audio::tone(880.0, 0.08, Some(&self.b.settings), "sine", 0.22);
            }
        } else if matches!(st, St::Fly | St::Coast) {
            self.advance(dt);
        } else if st == St::End {
            self.end_t += dt;
            let limit = if self.auto.is_some() { AUTO_END_T * self.tf() } else { END_T };
            if self.end_t >= limit {
                self.to_idle();
                let pause = AUTO_PAUSE_T * self.tf();
                if let Some(a) = self.auto.as_mut() {
                    a.t = pause;
                }
            }
        }
        if self.auto.is_some() && self.state == St::Idle {
            self.auto_tick(dt);
        }
        self.update_sounds(dt);
        self.update_fx(dt);
        self.update_music(dt);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        s.bl(&self.bg, (0, 0));
        self.draw_stage(&mut s);
        self.draw_panel(&mut s);
        self.draw_hud(&mut s);
        self.draw_tools(&mut s);
        self.draw_history(&mut s);
        self.draw_strip(&mut s);
        self.draw_fx(&mut s);
        if self.banner.is_some() {
            self.draw_banner(&mut s);
        }
        match self.window {
            Some(Win::Auto) => self.draw_auto(&mut s),
            Some(_) => self.draw_window(&mut s),
            None => {}
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

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "lamacrash",
    class_name: "LamaCrashGame",
    supports_multiplayer: false,
    modes: &[],
    make: LamaCrashGame::new,
};
