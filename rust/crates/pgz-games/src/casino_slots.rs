//! casino_slots.rs
//! ===============
//! Lama-Slot des Casinos - uebertragen aus `games/casino_slots.py`.
//!
//! 5 Walzen x 3 Reihen, 10 Gewinnlinien, Einsatz je Linie 1/2/5/10. Lama =
//! Wild, Goldmuenze = Scatter (3+ = Scatter-Gewinn und 10 Freispiele mit
//! doppelten Gewinnen). Walzen laufen gleichmaessig, bremsen gestaffelt ab
//! und landen exakt auf ihrem Stopp; zwei Muenzen machen die restlichen
//! Walzen spannungsvoll laenger ("Anticipation").

use std::f64::consts::PI;

use pgz_core::lamabank;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

use crate::casino::CasinoGame;
use crate::casino_draw as d;
use crate::casino_logic::{self as l, SlotRes};

const FREE_KEY: &str = "slots_free";
const SPEED: f64 = 21.0;
const DECEL: f64 = 0.34;
const BOUNCE: f64 = 0.16;
const BIG_WIN: f64 = 15.0;
const MEGA_WIN: f64 = 50.0;
const BTN_KEYS: [&str; 4] = ["pay", "turbo", "auto", "spin"];

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SPhase {
    #[default]
    Idle,
    Spin,
    Show,
}

#[derive(Clone, Copy)]
struct Reel {
    p0: f64,
    v: f64,
    t_stop: f64,
    target: i64,
    landed: bool,
}

struct Banner {
    text: String,
    sub: Option<String>,
    color: (i32, i32, i32),
    t: f64,
    dur: f64,
    win: bool,
    amount: i64,
}

/// Zustand des Slots (Python: `s_*`-Felder).
#[derive(Default)]
pub(crate) struct Slot {
    bet: i64,
    turbo: bool,
    pub(crate) phase: SPhase,
    pos: [f64; l::REELS],
    reels: Vec<Reel>,
    t: f64,
    speed: f64,
    res: Option<SlotRes>,
    free_spin: bool,
    show_t: f64,
    win_shown: f64,
    auto: i64,
    auto_menu: bool,
    paytable: bool,
    banners: Vec<Banner>,
    anticip: Vec<usize>,
    hover_line: Option<usize>,
    pub(crate) free: i64,
    free_bet: i64,
    free_won: i64,
    fs_active: bool,
    cell: i32,
    gap: i32,
    tb: i32,
    cab: Rect,
    win: Rect,
    reel_rects: Vec<Rect>,
    info: Rect,
    tabs: Vec<((usize, usize), Rect)>,
    cab_surf: Option<Surface>,
    cab_off: i32,
    title_band: Rect,
    bulbs: Vec<(i32, i32)>,
    minus: Rect,
    bet_box: Rect,
    plus: Rect,
    btns: Vec<(&'static str, Rect)>,
    auto_opts: Vec<(i64, Rect)>,
    bg: Option<Surface>,
    pay_cache: Option<((i32, i32, i64, String), Surface)>,
}

/// Python `int(x)` fuer Zahlen aus gespeicherten Daten.
fn json_int(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Number(n)) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        Some(Value::Bool(b)) => *b as i64,
        _ => 0,
    }
}

/// Eine Gewinnanzeige: (Zellen, Linien, Text, Phasenzeit).
type Highlight = (Vec<(usize, usize)>, Vec<usize>, String, f64);

impl CasinoGame {
    // ===================================================== Aufbau

    pub(crate) fn s_init(&mut self) {
        let bet = self.opt("line_bet").and_then(|v| if v.is_i64() || v.is_u64() { v.as_i64() } else { None });
        self.s.bet = bet.filter(|b| l::LINE_BETS.contains(b)).unwrap_or(1);
        self.s.turbo = match self.opt("turbo") {
            Some(Value::Bool(b)) => b,
            Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0) != 0.0,
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
            _ => false,
        };
        self.s.phase = SPhase::Idle;
        let stops = l::random_stops(&mut self.rng);
        for (r, st) in stops.iter().enumerate() {
            self.s.pos[r] = *st as f64;
        }
        self.s.reels.clear();
        self.s.t = 0.0;
        self.s.speed = 1.0;
        self.s.res = None;
        self.s.free_spin = false;
        self.s.show_t = 0.0;
        self.s.win_shown = 0.0;
        self.s.auto = 0;
        self.s.auto_menu = false;
        self.s.paytable = false;
        self.s.banners.clear();
        self.s.anticip.clear();
        self.s.hover_line = None;
        let free = match lamabank::get_extra(FREE_KEY) {
            Some(Value::Object(o)) => o,
            _ => Map::new(),
        };
        self.s.free = 0.max(json_int(free.get("left")));
        let fb = match free.get("bet") {
            Some(v) => v.as_i64().filter(|_| v.is_i64() || v.is_u64()),
            None => Some(self.s.bet),
        };
        self.s.free_bet = fb.filter(|b| l::LINE_BETS.contains(b)).unwrap_or(self.s.bet);
        self.s.free_won = 0.max(json_int(free.get("won")));
        self.s.fs_active = self.s.free > 0;
        if self.s.free > 0 {
            let text = tf("cas.s.free_left", &[("n", &self.s.free.to_string())]);
            self.s_banner(text, None, (255, 150, 200), 2.2, false, 0);
        }
    }

    pub(crate) fn s_busy(&self) -> bool {
        self.s.phase == SPhase::Spin
    }

    pub(crate) fn s_layout(&mut self) {
        let (w, h, k, m) = (self.b.width, self.b.height, self.k, self.margin);
        let top = self.hud_h + m;
        let bottom = self.strip.y - m;
        let tb = 24.max((44.0 * k) as i32);
        let ib = 26.max((40.0 * k) as i32);
        let tab = 16.max((26.0 * k) as i32);
        let pad = 6.max((12.0 * k) as i32);
        let gap = 3.max((6.0 * k) as i32);
        let avail_h = bottom - top;
        let cell = ((avail_h - tb - ib - pad) as f64 / 3.0).min((w - 2 * m - 2 * tab - 2 * pad - 4 * gap) as f64 / 5.0);
        let cell = 24f64.max(cell) as i32;
        let reels_w = 5 * cell + 4 * gap;
        let cab_w = reels_w + 2 * tab + 2 * pad;
        let cab_h = tb + 3 * cell + ib + pad;
        let cab = Rect::new(0, 0, cab_w, cab_h).center_at((w / 2, (top as f64 + avail_h as f64 / 2.0) as i32));
        self.s.cell = cell;
        self.s.gap = gap;
        self.s.tb = tb;
        self.s.cab = cab;
        self.s.win = Rect::new(cab.x + pad + tab, cab.y + tb, reels_w, 3 * cell);
        self.s.reel_rects = (0..l::REELS as i32).map(|r| Rect::new(self.s.win.x + r * (cell + gap), self.s.win.y, cell, 3 * cell)).collect();
        let fpad = gap + 3;
        self.s.info = Rect::new(cab.x + pad, self.s.win.bottom() + fpad, cab_w - 2 * pad, ib - fpad);
        self.s_build_tabs(tab);
        self.s_build_cabinet();
        self.s_build_strip();
        self.s.bg = Some(self.s_make_bg(w, h));
        self.s.pay_cache = None;
    }

    fn s_build_tabs(&mut self, tab: i32) {
        let cell = self.s.cell;
        self.s.tabs.clear();
        for side in 0..2usize {
            let mut per_row: [Vec<usize>; 3] = [Vec::new(), Vec::new(), Vec::new()];
            for (i, line) in l::LINES.iter().enumerate() {
                per_row[line[if side == 0 { 0 } else { l::REELS - 1 }]].push(i);
            }
            for (row, lines) in per_row.iter().enumerate() {
                let n = lines.len() as i32;
                let th = ((cell as f64 / 4.4) as i32).min((tab as f64 * 0.9) as i32);
                let total = n * th + (n - 1) * 2;
                let y0 = self.s.win.y + row as i32 * cell + py::fdiv32(cell - total, 2);
                let x = if side == 0 { self.s.win.x - tab - 1 } else { self.s.win.right() + 1 };
                for (j, i) in lines.iter().enumerate() {
                    self.s.tabs.push(((side, *i), Rect::new(x, y0 + j as i32 * (th + 2), tab, th)));
                }
            }
        }
    }

    fn tab_rect(&self, side: usize, i: usize) -> Rect {
        self.s.tabs.iter().find(|(k, _)| *k == (side, i)).map(|e| e.1).unwrap_or(Rect::new(0, 0, 0, 0))
    }

    fn s_make_bg(&self, w: i32, h: i32) -> Surface {
        let mut surf = pg::surface(w, h);
        let (top, bottom) = ((46, 16, 58), (12, 6, 20));
        for y in 0..h {
            let fr = y as f64 / 1.max(h - 1) as f64;
            surf.fill_r(ui::mix(top, bottom, fr), (0, y, w, 1));
        }
        let mut glow = pg::surface_alpha(w, h);
        let step = 18.max((34.0 * self.k) as i32);
        let mut x = -h;
        while x < w {
            draw::line(&mut glow, rgba(255, 255, 255, 7), (x, 0), (x + h, h), 2);
            x += step;
        }
        let (wf, hf) = (w as f64, h as f64);
        draw::ellipse(&mut glow, rgba(255, 120, 190, 22), (wf * 0.1, hf * 0.12, wf * 0.8, hf * 0.7), 0);
        surf.bl(&glow, (0, 0));
        surf
    }

    fn s_build_cabinet(&mut self) {
        let cab = self.s.cab;
        let k = self.k;
        let mut surf = pg::surface_alpha(cab.w + 16, cab.h + 16);
        let o = 8;
        let body = Rect::new(o, o, cab.w, cab.h);
        let rad = 10.max((22.0 * k) as i32);
        draw::rect_r(&mut surf, rgba(0, 0, 0, 110), body.move_(3, 6), 0, rad);
        let mut grad = pg::surface_alpha(body.w, body.h);
        for y in 0..body.h {
            let fr = y as f64 / 1.max(body.h - 1) as f64;
            grad.fill_r(ui::mix((92, 30, 96), (38, 12, 48), fr), (0, y, body.w, 1));
        }
        let mut mask = pg::surface_alpha(body.w, body.h);
        let mr = mask.get_rect();
        draw::rect_r(&mut mask, rgba(255, 255, 255, 255), mr, 0, rad);
        grad.bl_flags(&mask, (0, 0), BLEND_RGBA_MULT);
        surf.blr(&grad, body);
        draw::rect_r(&mut surf, d::COL_GOLD_D, body, 3.max((5.0 * k) as i32), rad);
        draw::rect_r(&mut surf, d::COL_GOLD, body.inflate(-4, -4), 1.max((2.0 * k) as i32), rad);
        let win = self.s.win.move_(o - cab.x, o - cab.y);
        let frame = win.inflate(self.s.gap * 2 + 6, self.s.gap * 2 + 6);
        draw::rect_r(&mut surf, (20, 8, 24), frame, 0, 6.max((10.0 * k) as i32));
        draw::rect_r(&mut surf, d::COL_GOLD, frame, 2.max((3.0 * k) as i32), 6.max((10.0 * k) as i32));
        for r in 0..l::REELS {
            let rr = self.s.reel_rects[r].move_(o - cab.x, o - cab.y);
            for y in 0..rr.h {
                let fr = (y as f64 / 1.max(rr.h - 1) as f64 - 0.5).abs() * 2.0;
                surf.fill_r(ui::mix((252, 248, 238), (206, 196, 180), py::pow(fr, 2.0)), (rr.x, rr.y + y, rr.w, 1));
            }
        }
        let band = Rect::new(body.x + (30.0 * k) as i32, body.y + 3.max((6.0 * k) as i32), body.w - (60.0 * k) as i32, self.s.tb - 6.max((12.0 * k) as i32));
        draw::rect_r(&mut surf, (26, 8, 30), band, 0, band.h / 2);
        draw::rect_r(&mut surf, d::COL_GOLD_D, band, 1.max((2.0 * k) as i32), band.h / 2);
        self.s.title_band = band.move_(cab.x - o, cab.y - o);
        self.s.cab_surf = Some(surf);
        self.s.cab_off = o;
        self.s.bulbs.clear();
        let n = 8.max(band.w / 12.max((22.0 * k) as i32));
        let tb = self.s.title_band;
        for i in 0..n {
            let x = tb.x + ((i as f64 + 0.5) * tb.w as f64 / n as f64) as i32;
            self.s.bulbs.push((x, tb.y - 2.max((3.0 * k) as i32)));
            self.s.bulbs.push((x, tb.bottom() + 2.max((3.0 * k) as i32)));
        }
    }

    fn s_build_strip(&mut self) {
        let (m, k) = (self.margin, self.k);
        let (sy, sh) = (self.strip.y, self.strip_h);
        let bh = 30.max((sh as f64 * 0.62) as i32);
        let by = sy + py::fdiv32(sh - bh, 2);
        let pad = (22.0 * 0.7f64.max(k)) as i32;
        let sq = bh;
        let lbl_w = self.f_tiny.size(&t("cas.s.line_bet")).0.max(self.f_btn.size("10 (= 100)").0) + (10.0 * k) as i32;
        self.s.minus = Rect::new(m, by, sq, bh);
        self.s.bet_box = Rect::new(self.s.minus.right() + 4, by, lbl_w, bh);
        self.s.plus = Rect::new(self.s.bet_box.right() + 4, by, sq, bh);
        let left = self.s.plus.right() + m;
        let labels = [t("cas.s.paytable"), t("cas.s.turbo"), t("cas.s.auto"), t("cas.spin")];
        let mut widths: Vec<i32> = labels.iter().map(|lb| self.f_btn.size(lb).0 + pad).collect();
        widths[2] = widths[2].max(self.f_btn.size(&(t("cas.s.auto") + " 25")).0 + pad);
        widths[3] = widths[3].max(self.f_btn.size(&t("cas.s.stop")).0 + pad).max((110.0 * k) as i32);
        let bgap = 4.max((8.0 * k) as i32);
        let room = self.b.width - m - left - bgap * (BTN_KEYS.len() as i32 - 1);
        let total: i32 = widths.iter().sum();
        if total > room {
            let fr = room as f64 / total as f64;
            widths = widths.iter().map(|w| (*w as f64 * fr) as i32).collect();
        }
        self.s.btns.clear();
        let mut x = self.b.width - m;
        for (key, w) in BTN_KEYS.iter().zip(widths.iter()).rev() {
            self.s.btns.push((key, Rect::new(x - w, by, *w, bh)));
            x -= w + bgap;
        }
        let a = self.s_btn("auto");
        let mh = (bh as f64 * 0.9) as i32;
        self.s.auto_opts = l::AUTO_COUNTS.iter().enumerate().map(|(i, n)| (*n, Rect::new(a.x, self.strip.y - (i as i32 + 1) * (mh + 4), a.w, mh))).collect();
    }

    fn s_btn(&self, key: &str) -> Rect {
        self.s.btns.iter().find(|e| e.0 == key).map(|e| e.1).unwrap_or(Rect::new(0, 0, 0, 0))
    }

    // ===================================================== Hilfen

    fn s_line_bet(&self) -> i64 {
        if self.s.free > 0 || self.s.free_spin {
            self.s.free_bet
        } else {
            self.s.bet
        }
    }

    fn s_total_bet(&self) -> i64 {
        self.s_line_bet() * l::LINES.len() as i64
    }

    pub(crate) fn s_hud_text(&self) -> String {
        if self.s.free > 0 || self.s.free_spin {
            return tf("cas.s.free_left", &[("n", &self.s.free.to_string())]);
        }
        format!("{}: {}", t("cas.bet"), self.s_total_bet())
    }

    fn s_banner(&mut self, text: String, sub: Option<String>, color: (i32, i32, i32), dur: f64, win: bool, amount: i64) {
        let dur = dur * if self.s.turbo { 0.6 } else { 1.0 };
        self.s.banners.push(Banner { text, sub, color, t: 0.0, dur, win, amount });
    }

    fn free_value(left: i64, bet: i64, won: i64) -> Value {
        json!({"left": left, "bet": bet, "won": won})
    }

    fn s_save_free(&mut self) {
        if self.s.free > 0 || self.s.fs_active {
            lamabank::set_extra(FREE_KEY, Self::free_value(self.s.free, self.s.free_bet, self.s.free_won));
        } else {
            lamabank::set_extra(FREE_KEY, json!({}));
        }
    }

    fn s_set_bet(&mut self, direction: i64) {
        if self.s.phase == SPhase::Spin || self.s.free > 0 || self.s.fs_active {
            return;
        }
        let i = l::LINE_BETS.iter().position(|b| *b == self.s.bet).unwrap_or(0) as i64;
        let j = 0.max((l::LINE_BETS.len() as i64 - 1).min(i + direction));
        if j != i {
            self.s.bet = l::LINE_BETS[j as usize];
            let v = self.s.bet;
            self.set_opt("line_bet", json!(v));
            self.b.play_sound("select");
        }
    }

    fn s_toggle_turbo(&mut self) {
        self.s.turbo = !self.s.turbo;
        let v = self.s.turbo;
        self.set_opt("turbo", json!(v));
        self.b.play_sound("click");
    }

    fn s_start_auto(&mut self, n: i64) {
        self.s.auto_menu = false;
        self.s.auto = n;
        self.b.play_sound("select");
        if self.s.phase != SPhase::Spin && self.s.banners.is_empty() {
            self.s_spin();
        }
    }

    // ===================================================== Drehen

    fn s_spin(&mut self) {
        if self.s.phase == SPhase::Spin {
            self.s.speed = 3.2;
            return;
        }
        if let Some(b) = self.s.banners.first_mut() {
            b.t = b.t.max(b.dur - 0.25);
            return;
        }
        let free = self.s.free > 0;
        let bet;
        if free {
            self.s.free -= 1;
            bet = self.s.free_bet;
        } else {
            bet = self.s.bet;
            if !lamabank::debit(bet * l::LINES.len() as i64, "casino") {
                self.s.auto = 0;
                self.say(t("cas.not_enough"));
                self.b.play_sound("hit");
                self.check_broke();
                return;
            }
            if self.s.auto > 0 {
                self.s.auto -= 1;
            }
        }
        let stops = l::random_stops(&mut self.rng);
        let res = l::evaluate(&l::window(&stops), bet, free);
        lamabank::credit(res.total, "casino", true);
        self.pending += res.total;
        self.s.free_spin = free;
        if res.free_spins != 0 {
            if !self.s.fs_active && !free {
                self.s.free_won = 0;
            }
            self.s.free += res.free_spins;
            self.s.free_bet = bet;
            self.s.fs_active = true;
        }
        if free || res.free_spins != 0 {
            let won = self.s.free_won + if free { res.total } else { 0 };
            lamabank::set_extra(FREE_KEY, Self::free_value(self.s.free, self.s.free_bet, won));
        }
        self.s.res = Some(res);
        self.s_schedule(&stops);
        self.s.phase = SPhase::Spin;
        self.s.t = 0.0;
        self.s.speed = 1.0;
        self.s.auto_menu = false;
        self.b.play_sound("rotate");
    }

    fn s_schedule(&mut self, stops: &[i64; l::REELS]) {
        let turbo = self.s.turbo;
        let base = if turbo { 0.34 } else { 0.78 };
        let gap = if turbo { 0.10 } else { 0.24 };
        let extra = if turbo { 0.45 } else { 1.05 };
        let speed = SPEED * if turbo { 1.5 } else { 1.0 };
        let win = l::window(stops);
        self.s.reels.clear();
        self.s.anticip.clear();
        let mut delay = 0.0;
        let mut coins = 0;
        for r in 0..l::REELS {
            if r >= 2 && coins >= 2 {
                delay += extra;
                self.s.anticip.push(r);
            }
            let t_stop = base + r as f64 * gap + delay;
            let n = l::STRIPS[r].len() as f64;
            let p0 = self.s.pos[r];
            let nominal = speed * (t_stop - DECEL / 2.0);
            let need = py::fpmod(p0 - nominal - stops[r] as f64, n);
            let dist = nominal + need;
            let v = dist / (t_stop - DECEL / 2.0);
            self.s.reels.push(Reel { p0, v, t_stop, target: stops[r], landed: false });
            coins += win[r].iter().filter(|s| **s == l::SCATTER).count();
        }
    }

    fn s_reel_pos(reel: &Reel, t: f64) -> (f64, f64) {
        let (tt, v) = (reel.t_stop, reel.v);
        let n = l::STRIPS[0].len() as i64;
        if t <= tt - DECEL {
            return (reel.p0 - v * t, v);
        }
        if t < tt {
            let u = (t - (tt - DECEL)) / DECEL;
            let dd = v * (tt - DECEL) + v * DECEL * (u - u * u / 2.0);
            return (reel.p0 - dd, v * (1.0 - u));
        }
        let b = (t - tt) / BOUNCE;
        if b < 1.0 {
            return (reel.target as f64 - 0.14 * (PI * b).sin() * (1.0 - b), 0.0);
        }
        (py::pmod(reel.target, n) as f64, 0.0)
    }

    fn s_reveal(&mut self) {
        let Some(res) = self.s.res.clone() else {
            return;
        };
        self.pending = 0.max(self.pending - res.total);
        self.s.phase = if res.total > 0 { SPhase::Show } else { SPhase::Idle };
        self.s.show_t = 0.0;
        self.s.win_shown = 0.0;
        if self.s.free_spin {
            self.s.free_won += res.total;
        }
        self.b.score = lamabank::score_for("casino");
        let total_bet = self.s_line_bet() * l::LINES.len() as i64;
        let mult = res.total as f64 / 1.max(total_bet) as f64;
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        if res.jackpot {
            self.s_banner(t("cas.s.jackpot"), None, (255, 214, 90), 3.2, true, res.total);
            self.b.ach_event("slots_jackpot", None);
            ui::spawn_confetti(w, h, 140);
            self.coin_shower(90);
            self.b.play_sound("level");
            self.b.rumble(300);
        } else if mult >= MEGA_WIN {
            self.s_banner(t("cas.s.mega_win"), None, (255, 150, 220), 2.8, true, res.total);
            ui::spawn_confetti(w, h, 110);
            self.coin_shower(70);
            self.b.play_sound("win");
            self.b.rumble(220);
        } else if mult >= BIG_WIN {
            self.s_banner(t("cas.s.big_win"), None, (255, 214, 90), 2.4, true, res.total);
            ui::spawn_confetti(w, h, 90);
            self.coin_shower(40);
            self.b.play_sound("win");
        } else if res.total > 0 {
            self.b.play_sound("point");
        }
        if res.free_spins != 0 {
            let text = tf("cas.s.free_won", &[("n", &res.free_spins.to_string())]);
            self.s_banner(text, Some(t("cas.s.free_info")), (255, 150, 200), 2.4, false, 0);
            self.b.play_sound("powerup");
        }
        if self.s.free_spin && self.s.free == 0 {
            self.s_save_free();
        }
    }

    fn s_end_free(&mut self) {
        self.s.fs_active = false;
        let won = self.s.free_won;
        self.s_banner(t("cas.s.free_end"), Some(format!("+{won}")), (255, 214, 90), 2.6, true, won);
        if won as f64 >= BIG_WIN * self.s.free_bet as f64 * l::LINES.len() as f64 {
            ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 90);
            self.coin_shower(40);
        }
        self.b.play_sound("level");
        self.s.free_won = 0;
        self.s_save_free();
    }

    // ===================================================== Eingabe

    pub(crate) fn s_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        if kind == Kind::MouseMove {
            self.s.hover_line = None;
            for ((_, i), rect) in &self.s.tabs {
                if rect.hit(ev.pos) {
                    self.s.hover_line = Some(*i);
                }
            }
            return;
        }
        let key = ev.key.as_str();
        if self.s.paytable {
            if kind == Kind::MouseDown || (kind == Kind::KeyDown && ["i", "I", "p", "P", "Return", "space", "KP_Enter"].contains(&key)) {
                self.s.paytable = false;
                self.b.play_sound("click");
            }
            return;
        }
        if kind == Kind::KeyDown {
            if ["Return", "space", "KP_Enter"].contains(&key) {
                if self.s.auto > 0 && self.s.phase != SPhase::Spin {
                    self.s.auto = 0;
                } else {
                    self.s_spin();
                }
            } else if ["plus", "KP_Add", "Up", "equal"].contains(&key) {
                self.s_set_bet(1);
            } else if ["minus", "KP_Subtract", "Down"].contains(&key) {
                self.s_set_bet(-1);
            } else if key == "t" || key == "T" {
                self.s_toggle_turbo();
            } else if key == "a" || key == "A" {
                if self.s.auto != 0 {
                    self.s.auto = 0;
                } else {
                    self.s_start_auto(l::AUTO_COUNTS[if key == "a" { 0 } else { 1 }]);
                }
            } else if ["i", "I", "p", "P"].contains(&key) {
                self.s.paytable = true;
                self.s.auto_menu = false;
                self.b.play_sound("click");
            }
            return;
        }
        if kind == Kind::Wheel {
            self.s_set_bet(if ev.delta > 0 { 1 } else { -1 });
            return;
        }
        if kind != Kind::MouseDown || ev.button != 1 {
            return;
        }
        let pos = ev.pos;
        if self.s.auto_menu {
            if let Some(&(n, _)) = self.s.auto_opts.iter().find(|(_, r)| r.hit(pos)) {
                self.s_start_auto(n);
                return;
            }
            self.s.auto_menu = false;
            return;
        }
        if self.s.minus.hit(pos) {
            self.s_set_bet(-1);
        } else if self.s.plus.hit(pos) {
            self.s_set_bet(1);
        } else if self.s_btn("pay").hit(pos) {
            self.s.paytable = true;
            self.b.play_sound("click");
        } else if self.s_btn("turbo").hit(pos) {
            self.s_toggle_turbo();
        } else if self.s_btn("auto").hit(pos) {
            if self.s.auto != 0 {
                self.s.auto = 0;
                self.b.play_sound("move");
            } else {
                self.s.auto_menu = true;
                self.b.play_sound("click");
            }
        } else if self.s_btn("spin").hit(pos) || self.s.win.hit(pos) {
            if self.s.auto > 0 && self.s.phase != SPhase::Spin {
                self.s.auto = 0;
            } else {
                self.s_spin();
            }
        }
    }

    // ===================================================== Update

    pub(crate) fn s_update(&mut self, dt: f64) {
        if let Some(b) = self.s.banners.first_mut() {
            b.t += dt;
            if b.t >= b.dur {
                self.s.banners.remove(0);
            }
        }
        if self.s.phase == SPhase::Spin {
            self.s.t += dt * self.s.speed;
            let mut last = 0.0f64;
            for r in 0..self.s.reels.len() {
                let reel = self.s.reels[r];
                let (pos, _) = Self::s_reel_pos(&reel, self.s.t);
                self.s.pos[r] = pos;
                if !reel.landed && self.s.t >= reel.t_stop {
                    self.s.reels[r].landed = true;
                    self.b.play_sound("lock");
                    if r + 1 < l::REELS && self.s.anticip.contains(&(r + 1)) {
                        audio::tone(420.0 + 90.0 * r as f64, 0.22, Some(&self.b.settings), "sine", 0.18);
                    }
                }
                last = last.max(reel.t_stop);
            }
            if self.s.t >= last + BOUNCE {
                for r in 0..self.s.reels.len() {
                    self.s.pos[r] = self.s.reels[r].target as f64;
                }
                self.s_reveal();
            }
            return;
        }
        self.s.show_t += dt;
        if let Some(res) = self.s.res.as_ref() {
            if res.total > 0 {
                let target = res.total as f64;
                let step = (target * dt / if self.s.turbo { 0.5 } else { 1.1 }).max(30.0 * dt);
                self.s.win_shown = target.min(self.s.win_shown + step);
            }
        }
        if self.broke || self.s.paytable || !self.s.banners.is_empty() {
            return;
        }
        let won = self.s.res.as_ref().is_some_and(|r| r.total > 0);
        let wait = (if won { 1.7 } else { 0.55 }) * if self.s.turbo { 0.5 } else { 1.0 };
        if self.s.fs_active && self.s.free == 0 && self.s.show_t >= wait {
            self.s_end_free();
            return;
        }
        if self.s.show_t < wait {
            return;
        }
        if self.s.free > 0 {
            self.s_spin();
        } else if self.s.auto > 0 {
            self.s_spin();
            if self.s.phase != SPhase::Spin {
                self.s.auto = 0;
            }
        } else {
            self.check_broke_idle();
        }
    }

    fn check_broke_idle(&mut self) {
        if !self.broke && lamabank::is_broke("casino", Some("slots")) && self.s.free == 0 && self.pending == 0 {
            self.check_broke();
        }
    }

    // ===================================================== Zeichnen

    fn s_highlights(&self) -> Option<Highlight> {
        let res = self.s.res.as_ref()?;
        if self.s.phase != SPhase::Show {
            return None;
        }
        // ("line", i, sym, n, amount) bzw. ("scatter", anzahl, gewinn)
        let mut entries: Vec<Option<(usize, &str, i64, i64)>> = res.lines.iter().map(|e| Some(*e)).collect();
        if res.scatter_win > 0 {
            entries.push(None);
        }
        if entries.is_empty() {
            return None;
        }
        let first = if !self.s.turbo { 1.0 } else { 0.6 };
        let step = if !self.s.turbo { 1.0 } else { 0.6 };
        let tt = self.s.show_t;
        let shown: Vec<Option<(usize, &str, i64, i64)>>;
        let phase_t;
        let text;
        if tt < first || entries.len() == 1 {
            shown = entries.clone();
            phase_t = tt;
            text = format!("{}: {}", t("cas.win"), self.s.win_shown as i64);
        } else {
            let idx = py::pmod(((tt - first) / step) as i64, entries.len() as i64) as usize;
            shown = vec![entries[idx]];
            phase_t = py::fpmod(tt - first, step);
            text = match entries[idx] {
                Some((i, sym, n, amount)) => tf(
                    "cas.s.line_win",
                    &[("line", &(i + 1).to_string()), ("n", &n.to_string()), ("sym", &t(&format!("cas.sym.{sym}"))), ("win", &amount.to_string())],
                ),
                None => tf("cas.s.scatter_win", &[("n", &res.scatter.to_string()), ("win", &res.scatter_win.to_string())]),
            };
        }
        let mut cells: Vec<(usize, usize)> = Vec::new();
        let mut lines = Vec::new();
        for e in shown {
            match e {
                Some((i, _sym, n, _amount)) => {
                    lines.push(i);
                    for r in 0..n as usize {
                        let c = (r, l::LINES[i][r]);
                        if !cells.contains(&c) {
                            cells.push(c);
                        }
                    }
                }
                None => {
                    for c in &res.coins {
                        if !cells.contains(c) {
                            cells.push(*c);
                        }
                    }
                }
            }
        }
        Some((cells, lines, text, phase_t))
    }

    pub(crate) fn s_draw(&mut self, s: &mut Surface) {
        if let Some(bg) = self.s.bg.as_ref() {
            s.bl(bg, (0, 0));
        }
        let cab = self.s.cab;
        let o = self.s.cab_off;
        if let Some(cs) = self.s.cab_surf.as_ref() {
            s.bl(cs, (cab.x - o, cab.y - o));
        }
        self.s_draw_title(s);
        let hl = self.s_highlights();
        let cells = hl.as_ref().map(|h| h.0.clone());
        let phase_t = hl.as_ref().map(|h| h.3).unwrap_or(0.0);
        let pop_scale = if phase_t < 0.35 { 1.0 + 0.13 * (PI * 1f64.min(phase_t / 0.35)).sin() } else { 1.0 };
        for r in 0..self.s.reel_rects.len() {
            let rect = self.s.reel_rects[r];
            let mut speed = 0.0;
            if self.s.phase == SPhase::Spin && r < self.s.reels.len() {
                speed = Self::s_reel_pos(&self.s.reels[r], self.s.t).1;
            }
            let (dim, pop): (Option<Vec<usize>>, Option<Vec<(usize, f64)>>) = match cells.as_ref() {
                Some(c) => (
                    Some((0..l::ROWS).filter(|row| !c.contains(&(r, *row))).collect()),
                    Some((0..l::ROWS).filter(|row| c.contains(&(r, *row))).map(|row| (row, pop_scale)).collect()),
                ),
                None => (None, None),
            };
            d::draw_reel(s, rect, &l::STRIPS[r], self.s.pos[r], self.s.cell, speed * self.s.speed, dim.as_deref(), pop.as_deref());
            if self.s.phase == SPhase::Spin && self.s.anticip.contains(&r) && !self.s.reels[r].landed {
                let glow = ui::mix(d::COL_GOLD, (255, 255, 255), ui::pulse(9.0, 0.0, 1.0));
                draw::rect_r(s, glow, rect.inflate(6, 6), 2.max((3.0 * self.k) as i32), 6);
            }
        }
        self.s_draw_tabs(s, hl.as_ref());
        if let Some(h) = hl.as_ref() {
            self.s_draw_win(s, h);
        } else if let Some(i) = self.s.hover_line {
            if self.s.phase != SPhase::Spin {
                self.s_draw_line_path(s, i, 2.0);
            }
        }
        self.s_draw_info(s, hl.as_ref());
        self.s_draw_strip(s);
        if !self.s.banners.is_empty() {
            self.s_draw_banner(s, 0);
        }
        if self.s.paytable {
            self.s_draw_paytable(s);
        }
    }

    fn s_draw_title(&self, s: &mut Surface) {
        let band = self.s.title_band;
        let tick = (pg::ticks() as f64 / if self.s.fs_active { 90.0 } else { 160.0 }) as i64;
        for (i, &(x, y)) in self.s.bulbs.iter().enumerate() {
            let on = (i as i64 / 2 + tick) % 3 == 0;
            let col = if on { (255, 236, 150) } else { (120, 80, 40) };
            let r = 2.max((3.0 * self.k) as i32);
            if on {
                draw::circle(s, (255, 200, 90), (x, y), r + 2, 0);
            }
            draw::circle(s, col, (x, y), r, 0);
        }
        let (title, top, bot) = if self.s.fs_active {
            (format!("{}  {}", t("cas.s.free_title"), self.s.free), rgb(255, 190, 230), rgb(230, 80, 150))
        } else {
            (t("cas.mode.slots").to_uppercase(), rgb(255, 240, 170), rgb(226, 150, 40))
        };
        let fnt = ui::font(13.max((band.h as f64 * 0.72) as i32), true, false);
        let mut img = ui::grad_text(&fnt, &title, Some(top), Some(bot));
        if img.w() > band.w - 10 {
            let fnt = ui::font(11.max((band.h as f64 * 0.52) as i32), true, false);
            img = ui::grad_text(&fnt, &title, Some(top), Some(bot));
        }
        s.blr(&img, img.get_rect().center_at(band.center()));
    }

    fn s_draw_tabs(&self, s: &mut Surface, hl: Option<&Highlight>) {
        let active: Vec<usize> = hl.map(|h| h.1.clone()).unwrap_or_default();
        let fnt = ui::font(8.max((self.tab_rect(0, 0).h as f64 * 0.8) as i32), true, false);
        for &((_side, i), rect) in &self.s.tabs {
            let col = d::LINE_COLS[i];
            let on = active.contains(&i) || Some(i) == self.s.hover_line;
            let fill = if on { col.c() } else { ui::mix(col, (30, 10, 30), 0.55) };
            draw::rect_r(s, fill, rect, 0, 2.max(rect.h / 3));
            let img = fnt.render(&(i + 1).to_string(), true, if on { (20, 10, 20) } else { (240, 230, 240) });
            s.blr(&img, img.get_rect().center_at(rect.center()));
        }
    }

    fn s_line_points(&self, i: usize) -> Vec<(i32, i32)> {
        let cell = self.s.cell;
        let a = self.tab_rect(0, i);
        let mut pts = vec![(a.right(), a.centery())];
        for r in 0..l::REELS {
            let rect = self.s.reel_rects[r];
            pts.push((rect.centerx(), rect.y + l::LINES[i][r] as i32 * cell + cell / 2));
        }
        let b = self.tab_rect(1, i);
        pts.push((b.x, b.centery()));
        pts
    }

    fn s_draw_line_path(&self, s: &mut Surface, i: usize, width: f64) {
        let pts = self.s_line_points(i);
        let w = 2.max((width * self.k * 1.5) as i32);
        draw::lines(s, (20, 8, 24), false, &pts, w + 4);
        draw::lines(s, d::LINE_COLS[i], false, &pts, w);
    }

    fn s_draw_win(&self, s: &mut Surface, hl: &Highlight) {
        let (cells, lines, _, _) = hl;
        for i in lines {
            self.s_draw_line_path(s, *i, 3.0);
        }
        let col = if lines.len() == 1 { d::LINE_COLS[lines[0]] } else { d::COL_GOLD };
        let pulse = ui::pulse(7.0, 0.0, 1.0);
        for &(r, row) in cells {
            let rect = Rect::new(self.s.reel_rects[r].x, self.s.win.y + row as i32 * self.s.cell, self.s.cell, self.s.cell).inflate(-2, -2);
            draw::rect_r(s, ui::mix(col, (255, 255, 255), pulse * 0.5), rect, 2.max((3.0 * self.k) as i32), 4.max((8.0 * self.k) as i32));
        }
    }

    fn s_draw_info(&self, s: &mut Surface, hl: Option<&Highlight>) {
        let p = ui::pal();
        let r = self.s.info;
        let (text, col): (String, Color) = if let Some(h) = hl {
            (h.2.clone(), p.gold)
        } else if self.s.phase == SPhase::Spin {
            (
                if self.s.fs_active { format!("{}  +{}", t("cas.s.free_title"), self.s.free_won) } else { t("cas.s.good_luck") },
                p.text_dim,
            )
        } else if self.s.fs_active {
            (tf("cas.s.free_total", &[("n", &self.s.free_won.to_string())]), rgb(255, 190, 230))
        } else if self.s.auto != 0 {
            (tf("cas.s.auto_left", &[("n", &self.s.auto.to_string())]), p.text_dim)
        } else if self.s.res.as_ref().is_some_and(|r| r.total == 0) {
            (t("cas.no_win"), p.text_dim)
        } else {
            (t("cas.s.hint"), p.text_dim)
        };
        let fnt = if hl.is_some() { &self.f_big } else { &self.f_small };
        let mut img = fnt.render(&text, true, col);
        if img.w() > r.w - 8 {
            img = self.f_small.render(&text, true, col);
        }
        if img.w() > r.w - 8 {
            img = self.f_tiny.render(&text, true, col);
        }
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    fn s_draw_strip(&self, s: &mut Surface) {
        let p = ui::pal();
        self.draw_strip_bg(s);
        let locked = self.s.phase == SPhase::Spin || self.s.free > 0 || self.s.fs_active;
        let bet = self.s_line_bet();
        self.btn(s, self.s.minus, "\u{2013}", false, !locked && bet > l::LINE_BETS[0], self.hot(self.s.minus), false);
        self.btn(s, self.s.plus, "+", false, !locked && bet < l::LINE_BETS[3], self.hot(self.s.plus), false);
        let bx = self.s.bet_box;
        let lbl = self.f_tiny.render(&t("cas.s.line_bet"), true, p.text_dim);
        let val = self.f_btn.render(&format!("{} (= {})", bet, bet * l::LINES.len() as i64), true, p.gold);
        let total_h = lbl.h() + val.h();
        let y0 = bx.centery() - total_h / 2;
        s.blr(&lbl, lbl.get_rect().midtop_at((bx.centerx(), y0)));
        s.blr(&val, val.get_rect().midtop_at((bx.centerx(), y0 + lbl.h())));
        let spinning = self.s.phase == SPhase::Spin;
        let auto_lbl = t("cas.s.auto") + &if self.s.auto != 0 { format!(" {}", self.s.auto) } else { String::new() };
        let spin_lbl = if self.s.auto != 0 && !spinning { t("cas.s.stop") } else { t("cas.spin") };
        let (rp, rt, ra, rs) = (self.s_btn("pay"), self.s_btn("turbo"), self.s_btn("auto"), self.s_btn("spin"));
        self.btn(s, rp, &t("cas.s.paytable"), false, true, self.hot(rp), false);
        self.btn(s, rt, &t("cas.s.turbo"), false, true, self.hot(rt), self.s.turbo);
        self.btn(s, ra, &auto_lbl, false, true, self.hot(ra), self.s.auto != 0);
        self.btn(s, rs, &spin_lbl, true, self.s.banners.is_empty() || spinning, self.hot(rs), false);
        if self.s.auto_menu {
            for &(n, rect) in &self.s.auto_opts {
                self.btn(s, rect, &format!("{} {}", t("cas.s.auto"), n), false, true, self.hot(rect), false);
            }
        }
    }

    fn s_draw_banner(&self, s: &mut Surface, idx: usize) {
        let b = &self.s.banners[idx];
        let cab = self.s.cab;
        let fr = b.t / 0.01f64.max(b.dur);
        let appear = 1f64.min(b.t / 0.25);
        let fade = if fr < 0.85 { 1.0 } else { 0f64.max((1.0 - fr) / 0.15) };
        let h = (cab.h as f64 * 0.42) as i32;
        let mut band = pg::surface_alpha(cab.w, h);
        band.fill_c(rgba(12, 4, 18, (210.0 * appear * fade) as i32));
        let lc = rgba(b.color.0, b.color.1, b.color.2, (255.0 * fade) as i32);
        draw::line(&mut band, lc, (0, 0), (cab.w, 0), 2);
        draw::line(&mut band, lc, (0, h - 2), (cab.w, h - 2), 2);
        let y = cab.centery() - h / 2;
        s.bl(&band, (cab.x, y));
        let scale = 0.6 + 0.4 * (1.0 - py::pow(1.0 - appear, 3.0)) + 0.04 * (b.t * 8.0).sin();
        let size = 16.max((self.f_huge.get_height() as f64 * scale) as i32);
        let fnt = ui::font(size, true, false);
        let mut img = ui::grad_text(&fnt, &b.text, Some(rgb(255, 255, 255)), Some(b.color.c()));
        if img.w() > cab.w - 20 {
            let fnt = ui::font(14.max((size as f64 * (cab.w - 20) as f64 / img.w() as f64) as i32), true, false);
            img = ui::grad_text(&fnt, &b.text, Some(rgb(255, 255, 255)), Some(b.color.c()));
        }
        let mut img = img.copy();
        img.set_alpha(Some((255.0 * fade) as i32 as u8));
        let mut sub = b.sub.clone();
        if b.win && b.amount != 0 {
            let shown = (b.amount as f64 * 1f64.min(b.t / (b.dur * 0.6))) as i64;
            sub = Some(format!("+{shown}"));
        }
        match sub.filter(|x| !x.is_empty()) {
            Some(sub) => {
                let mut simg = self.f_big.render(&sub, true, (255, 244, 220));
                if simg.w() > cab.w - 20 {
                    simg = self.f_small.render(&sub, true, (255, 244, 220));
                }
                let mut simg = simg.copy();
                simg.set_alpha(Some((255.0 * fade) as i32 as u8));
                s.blr(&img, img.get_rect().center_at((cab.centerx(), cab.centery() - simg.h() / 2)));
                s.blr(&simg, simg.get_rect().midtop_at((cab.centerx(), cab.centery() + img.h() / 2 - simg.h() / 2)));
            }
            None => {
                s.blr(&img, img.get_rect().center_at(cab.center()));
            }
        }
    }

    fn s_draw_paytable(&mut self, s: &mut Surface) {
        let key = (self.b.width, self.b.height, self.s_line_bet(), self.lang.clone());
        let fresh = self.s.pay_cache.as_ref().is_some_and(|(k, _)| *k == key);
        if !fresh {
            let surf = self.s_build_paytable();
            self.s.pay_cache = Some((key, surf));
        }
        let Some((_, surf)) = self.s.pay_cache.as_ref() else {
            return;
        };
        s.bl(&self.overlay, (0, 0));
        s.blr(surf, surf.get_rect().center_at((self.b.width / 2, (self.hud_h + self.strip.y) / 2)));
    }

    fn s_build_paytable(&self) -> Surface {
        let p = ui::pal();
        let k = self.k;
        let w = (self.b.width - 2 * self.margin).min((640.0 * 0.75f64.max(k)) as i32);
        let h = self.strip.y - self.hud_h - 2 * self.margin;
        let mut surf = pg::surface_alpha(w, h);
        let sr = surf.get_rect();
        ui::draw_panel_ex(&mut surf, sr, None, None, None, false, Some(d::COL_GOLD.c()));
        let bet = self.s_line_bet();
        let pad = 6.max((12.0 * k) as i32);
        let title = self.f_big.render(&t("cas.s.paytable_title"), true, d::COL_GOLD);
        surf.blr(&title, title.get_rect().midtop_at((w / 2, pad)));
        let y = pad + title.h() + 2.max((4.0 * k) as i32);
        let foot_lines = [t("cas.s.wild_info"), t("cas.s.scatter_info"), tf("cas.s.rtp", &[("p", l::RTP_TEXT)])];
        let foot_h = foot_lines.len() as i32 * (self.f_tiny.get_height() + 1);
        let rows = l::SYMBOLS;
        let cols = if w > 440 { 2 } else { 1 };
        let per_col = (rows.len() as i32 + cols - 1) / cols;
        let row_h = ((h - y - foot_h - pad * 2) as f64 / per_col as f64) as i32;
        let icon = 12.max((row_h - 4).min((46.0 * k) as i32));
        let col_w = py::fdiv32(w - 2 * pad, cols);
        let fnt = if row_h >= 22 { &self.f_small } else { &self.f_tiny };
        for (idx, sym) in rows.iter().enumerate() {
            let c = idx as i32 / per_col;
            let rr = idx as i32 % per_col;
            let x = pad + c * col_w;
            let yy = y + rr * row_h;
            let img = d::symbol_surface(sym, icon, false);
            surf.bl(&img, (x, yy + py::fdiv32(row_h - icon, 2)));
            let pays: Vec<i64> = if *sym == l::SCATTER {
                [3, 4, 5].iter().map(|n| l::scatter_pays(*n) * bet * l::LINES.len() as i64).collect()
            } else {
                l::pays(sym).unwrap_or([0; 3]).iter().map(|pp| pp * bet).collect()
            };
            let txt = [3, 4, 5].iter().zip(pays.iter()).map(|(n, pp)| format!("{n}\u{00D7} {pp}")).collect::<Vec<_>>().join("   ");
            let mut timg = fnt.render(&txt, true, p.text);
            if timg.w() > col_w - icon - 12 {
                timg = self.f_tiny.render(&txt, true, p.text);
            }
            surf.blr(&timg, timg.get_rect().midleft_at((x + icon + 8, yy + row_h / 2)));
        }
        let mut fy = h - pad - foot_h;
        for line in &foot_lines {
            let mut img = self.f_tiny.render(line, true, p.text_dim);
            if img.w() > w - 2 * pad {
                img = ui::font(9.max(self.f_tiny.get_height() - 3), false, false).render(line, true, p.text_dim);
            }
            surf.blr(&img, img.get_rect().midtop_at((w / 2, fy)));
            fy += self.f_tiny.get_height() + 1;
        }
        surf
    }
}
