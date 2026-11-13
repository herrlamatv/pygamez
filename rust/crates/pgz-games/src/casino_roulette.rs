//! casino_roulette.rs
//! ==================
//! Roulette-Teil des Casinos - uebertragen aus `games/casino_roulette.py`.
//!
//! Ablauf: SETZEN -> DREHEN -> ERGEBNIS -> SETZEN ... Der Rotor dreht im
//! Uhrzeigersinn und laeuft aus; der Winkel der Kugel RELATIV zum Rotor
//! folgt einer kubischen Ease-out-Kurve, die exakt im gezogenen Fach endet.

use std::f64::consts::{PI, TAU};

use pgz_core::lamabank;
use pgz_engine::prelude::*;
use serde_json::{json, Value};

use crate::cards;
use crate::casino::CasinoGame;
use crate::casino_draw::{self as d, WheelArt};
use crate::casino_logic as l;

const LAND_T: f64 = 5.6;
const RESULT_T: f64 = 3.4;
const IDLE_OMEGA: f64 = 0.32;
const SPIN_OMEGA: f64 = 2.3;
const OMEGA_TAU: f64 = 2.8;
const BALL_TURNS: f64 = 6.0;
const HISTORY_KEY: &str = "roulette_history";
const BTN_KEYS: [&str; 4] = ["clear", "double", "rebet", "spin"];

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum RPhase {
    #[default]
    Bet,
    Spin,
    Result,
}

/// Zustand des Roulettes (Python: `r_*`-Felder).
#[derive(Default)]
pub(crate) struct Roul {
    chip: i64,
    pub(crate) phase: RPhase,
    pub(crate) bets: Vec<(String, i64)>,
    last: Vec<(String, i64)>,
    history: Vec<i64>,
    hover: Option<String>,
    rot: f64,
    omega: f64,
    ball: Option<(f64, f64)>,
    trail: Vec<(f64, f64)>,
    number: Option<i64>,
    winners: Vec<String>,
    payout: i64,
    stake: i64,
    t: f64,
    fast: bool,
    res_t: f64,
    msg: Option<(String, Color)>,
    tick_i: Option<i64>,
    tick_cd: f64,
    drops: Vec<(String, f64)>,
    phi0: f64,
    phi_end: f64,
    uw: f64,
    uh: f64,
    tx: f64,
    ty: f64,
    wheel_c: (i32, i32),
    wheel: Option<WheelArt>,
    panel: Rect,
    felt: Option<Surface>,
    chip_r: i32,
    table: Option<Surface>,
    table_off: (i32, i32),
    hl: Option<Surface>,
    win_hl: Option<Surface>,
    chip_rects: Vec<(i64, Rect)>,
    chip_rad: i32,
    btns: Vec<(&'static str, Rect)>,
}

fn get(bets: &[(String, i64)], key: &str) -> Option<i64> {
    bets.iter().find(|(k, _)| k == key).map(|e| e.1)
}

impl CasinoGame {
    // ===================================================== Aufbau

    pub(crate) fn r_init(&mut self) {
        let chip = self.opt("roulette_chip").and_then(|v| if v.is_i64() || v.is_u64() { v.as_i64() } else { None });
        self.r.chip = chip.filter(|c| l::CHIP_VALUES.contains(c)).unwrap_or(5);
        self.r.phase = RPhase::Bet;
        self.r.bets.clear();
        self.r.last.clear();
        let hist = lamabank::get_extra(HISTORY_KEY);
        self.r.history = match hist {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| match v {
                    Value::Number(n) if n.is_i64() || n.is_u64() => n.as_i64(),
                    Value::Bool(b) => Some(*b as i64),
                    _ => None,
                })
                .filter(|n| (0..=36).contains(n))
                .collect(),
            _ => Vec::new(),
        };
        self.r.history.truncate(l::HISTORY_LEN);
        self.r.hover = None;
        self.r.rot = self.rng.uniform(0.0, TAU);
        self.r.omega = IDLE_OMEGA;
        self.r.ball = None;
        self.r.trail.clear();
        self.r.number = None;
        self.r.winners.clear();
        self.r.payout = 0;
        self.r.stake = 0;
        self.r.t = 0.0;
        self.r.fast = false;
        self.r.res_t = 0.0;
        self.r.msg = None;
        self.r.tick_i = None;
        self.r.tick_cd = 0.0;
        self.r.drops.clear();
    }

    pub(crate) fn r_busy(&self) -> bool {
        self.r.phase == RPhase::Spin
    }

    pub(crate) fn r_layout(&mut self) {
        let (w, h, k, m) = (self.b.width, self.b.height, self.k, self.margin);
        let top = self.hud_h + m;
        let bottom = self.strip.y - m;
        let uw = (w - 2 * m) as f64 / l::TABLE_W;
        let avail = bottom - top;
        let uh = (uw * 0.82).min(avail as f64 * 0.56 / l::TABLE_H);
        self.r.uw = uw;
        self.r.uh = uh;
        let th = uh * l::TABLE_H;
        self.r.tx = m as f64;
        self.r.ty = bottom as f64 - th;
        let region_h = self.r.ty - m as f64 - top as f64;
        let wd = 60f64.max(region_h.min(w as f64 * 0.40)) as i32;
        self.r.wheel_c = (m + wd / 2 + (6.0 * k) as i32, (top as f64 + region_h / 2.0) as i32);
        self.r.wheel = Some(WheelArt::new(wd));
        let px = self.r.wheel_c.0 + wd / 2 + 2 * m;
        self.r.panel = Rect::new(px, top, w - m - px, region_h as i32);
        self.r.felt = Some(cards::make_felt(w, h, d::COL_FELT.c(), d::COL_FELT_D.c()));
        self.r.chip_r = 6.max((uw.min(uh) * 0.34) as i32);
        self.r_build_table();
        self.r_build_strip();
        let mut hl = pg::surface_alpha(1.max(uw as i32), 1.max(uh as i32));
        hl.fill_c(rgba(255, 255, 255, 60));
        self.r.hl = Some(hl);
        let mut win_hl = pg::surface_alpha(1.max(uw as i32), 1.max(uh as i32));
        win_hl.fill_c(rgba(255, 214, 90, 80));
        self.r.win_hl = Some(win_hl);
    }

    fn r_build_strip(&mut self) {
        let (m, k) = (self.margin, self.k);
        let (sy, sh) = (self.strip.y, self.strip_h);
        let cr = 11.max((sh as f64 * 0.30) as i32);
        let gap = 4.max((8.0 * k) as i32);
        self.r.chip_rects.clear();
        let mut x = m + cr;
        for v in l::CHIP_VALUES {
            self.r.chip_rects.push((v, Rect::new(x - cr, sy + sh / 2 - cr, 2 * cr, 2 * cr)));
            x += 2 * cr + gap;
        }
        self.r.chip_rad = cr;
        let left = x - gap + m;
        let pad = (22.0 * 0.7f64.max(k)) as i32;
        let mut widths: Vec<i32> = BTN_KEYS.iter().map(|key| self.f_btn.size(&Self::r_btn_label(key)).0 + pad).collect();
        widths[3] = widths[3].max((110.0 * k) as i32);
        let bgap = 4.max((8.0 * k) as i32);
        let room = self.b.width - m - left - bgap * (BTN_KEYS.len() as i32 - 1);
        let total: i32 = widths.iter().sum();
        if total > room {
            let fr = room as f64 / total as f64;
            widths = widths.iter().map(|w| (*w as f64 * fr) as i32).collect();
        }
        let bh = 30.max((sh as f64 * 0.62) as i32);
        self.r.btns.clear();
        let mut x = self.b.width - m;
        for (key, w) in BTN_KEYS.iter().zip(widths.iter()).rev() {
            self.r.btns.push((key, Rect::new(x - w, sy + py::fdiv32(sh - bh, 2), *w, bh)));
            x -= w + bgap;
        }
    }

    fn r_btn_label(key: &str) -> String {
        match key {
            "clear" => t("cas.r.clear"),
            "double" => t("cas.r.double"),
            "rebet" => t("cas.r.rebet"),
            _ => t("cas.spin"),
        }
    }

    fn r_build_table(&mut self) {
        let (uw, uh) = (self.r.uw, self.r.uh);
        let (tw, th) = ((uw * l::TABLE_W) as i32 + 1, (uh * l::TABLE_H) as i32 + 1);
        let mut surf = pg::surface_alpha(tw + 8, th + 8);
        let (ox, oy) = (4, 4);
        let (oxf, oyf) = (ox as f64, oy as f64);
        let felt = ui::mix(d::COL_FELT, (255, 255, 255), 0.06);
        let line: (i32, i32, i32) = (236, 230, 206);
        let lw = 1.max((uw.min(uh) / 22.0) as i32);
        draw::rect_r(&mut surf, rgba(0, 0, 0, 70), (ox + 2, oy + 3, tw, th), 0, 6);
        draw::rect_r(&mut surf, felt, (ox, oy, tw, th), 0, 6);
        let cell = |ux: f64, uy: f64, cw: f64, ch: f64| -> Rect {
            Rect::new(
                ox + py::round(ux * uw) as i32,
                oy + py::round(uy * uh) as i32,
                py::round((ux + cw) * uw) as i32 - py::round(ux * uw) as i32,
                py::round((uy + ch) * uh) as i32 - py::round(uy * uh) as i32,
            )
        };
        let num_font = ui::font(9.max((uh * 0.40) as i32), true, false);
        let lbl_font = ui::font(9.max((uh * 0.34).min(uw * 0.30) as i32), true, false);
        let zero = [(oxf + uw, oyf), (oxf + uw * 0.42, oyf), (oxf + 2.0, oyf + 1.5 * uh), (oxf + uw * 0.42, oyf + 3.0 * uh), (oxf + uw, oyf + 3.0 * uh)];
        draw::polygon(&mut surf, d::COL_GREEN, &zero, 0);
        draw::polygon(&mut surf, line, &zero, lw);
        let img = num_font.render("0", true, (255, 255, 255));
        surf.blr(&img, img.get_rect().center_at((oxf + uw * 0.62, oyf + 1.5 * uh)));
        for col in 0..12 {
            for row in 0..3 {
                let n = l::cell_number(col, row);
                let r = cell(1.0 + col as f64, row as f64, 1.0, 1.0);
                let pill = r.inflate(-(4.max((uw * 0.16) as i32)), -(4.max((uh * 0.20) as i32)));
                draw::rect_r(&mut surf, d::number_color(n), pill, 0, 3.max(py::fdiv32(pill.h, 3)));
                let img = num_font.render(&n.to_string(), true, (255, 255, 255));
                surf.blr(&img, img.get_rect().center_at(r.center()));
                draw::rect(&mut surf, line, r, lw);
            }
        }
        for row in 0..3 {
            let r = cell(13.0, row as f64, 1.0, 1.0);
            draw::rect(&mut surf, line, r, lw);
            let img = lbl_font.render("2:1", true, (255, 255, 255));
            surf.blr(&img, img.get_rect().center_at(r.center()));
        }
        for dz in 0..3 {
            let r = cell(1.0 + 4.0 * dz as f64, 3.0, 4.0, l::DOZEN_H);
            draw::rect(&mut surf, line, r, lw);
            let img = lbl_font.render(&format!("{}\u{2013}{}", 12 * dz + 1, 12 * dz + 12), true, (255, 255, 255));
            surf.blr(&img, img.get_rect().center_at(r.center()));
        }
        for (i, key) in l::OUTSIDE_KEYS.iter().enumerate() {
            let r = cell(1.0 + 2.0 * i as f64, 3.0 + l::DOZEN_H, 2.0, l::OUTSIDE_H);
            draw::rect(&mut surf, line, r, lw);
            if *key == "red" || *key == "black" {
                let (dw, dh) = (r.w as f64 * 0.26, r.h as f64 * 0.34);
                let (cx, cy) = (r.centerx() as f64, r.centery() as f64);
                let pts = [(cx, cy - dh), (cx + dw, cy), (cx, cy + dh), (cx - dw, cy)];
                draw::polygon(&mut surf, if *key == "red" { d::COL_RED } else { d::COL_BLACK }, &pts, 0);
                draw::polygon(&mut surf, line, &pts, lw);
                continue;
            }
            let txt = match *key {
                "low" => "1\u{2013}18".to_string(),
                "high" => "19\u{2013}36".to_string(),
                "even" => t("cas.r.even"),
                _ => t("cas.r.odd"),
            };
            let mut img = lbl_font.render(&txt, true, (255, 255, 255));
            if img.w() > r.w - 6 {
                let fnt = ui::font(8.max((lbl_font.get_height() as f64 * 0.72) as i32), true, false);
                img = fnt.render(&txt, true, (255, 255, 255));
            }
            surf.blr(&img, img.get_rect().center_at(r.center()));
        }
        draw::rect_r(&mut surf, d::COL_GOLD, (ox - 1, oy - 1, tw + 2, th + 2), 2.max(lw + 1), 6);
        self.r.table = Some(surf);
        self.r.table_off = (ox, oy);
    }

    // ===================================================== Hilfen

    fn r_total(&self) -> i64 {
        self.r.bets.iter().map(|e| e.1).sum()
    }

    pub(crate) fn reserved(&self) -> i64 {
        if self.roulette() && self.r.phase == RPhase::Bet {
            return self.r_total();
        }
        0
    }

    fn r_to_px(&self, ux: f64, uy: f64) -> (f64, f64) {
        (self.r.tx + ux * self.r.uw, self.r.ty + uy * self.r.uh)
    }

    fn r_key_at(&self, pos: (i32, i32)) -> Option<String> {
        let ux = (pos.0 as f64 - self.r.tx) / self.r.uw;
        let uy = (pos.1 as f64 - self.r.ty) / self.r.uh;
        l::hit_test(ux, uy)
    }

    fn r_bet_name(key: &str) -> String {
        let kind = l::bet_kind(key);
        if ["red", "black", "even", "odd"].contains(&kind) {
            return t(&format!("cas.r.{kind}"));
        }
        if kind == "low" {
            return t("cas.r.low") + " 1\u{2013}18";
        }
        if kind == "high" {
            return t("cas.r.high") + " 19\u{2013}36";
        }
        let arg = key.split_once(':').map(|e| e.1).unwrap_or("");
        if kind == "column" {
            return format!("{} {}", t("cas.r.column"), 3 - arg.parse::<i64>().unwrap_or(0));
        }
        if kind == "dozen" {
            let dz = arg.parse::<i64>().unwrap_or(0);
            return format!("{} {}\u{2013}{}", t("cas.r.dozen"), 12 * dz + 1, 12 * dz + 12);
        }
        let sep = if kind == "cheval" { "/" } else { "-" };
        let nums: Vec<String> = l::numbers(key).iter().map(|n| n.to_string()).collect();
        format!("{} {}", t(&format!("cas.r.{kind}")), nums.join(sep))
    }

    fn r_number_text(n: i64) -> String {
        if n == 0 {
            return t("cas.r.zero");
        }
        let parts = [
            t(&format!("cas.r.{}", l::color_of(n))),
            t(if n % 2 == 0 { "cas.r.even" } else { "cas.r.odd" }),
            t(if n <= 18 { "cas.r.low" } else { "cas.r.high" }),
        ];
        parts.join(" \u{00B7} ")
    }

    pub(crate) fn r_hud_text(&self) -> String {
        let total = if self.r.phase == RPhase::Bet { self.r_total() } else { self.r.stake };
        format!("{}: {}", t("cas.bet"), total)
    }

    // ===================================================== Aktionen

    fn r_select_chip(&mut self, value: i64) {
        if value != self.r.chip {
            self.r.chip = value;
            self.set_opt("roulette_chip", json!(value));
            self.b.play_sound("select");
        }
    }

    fn r_available(&self) -> i64 {
        lamabank::balance() - self.r_total()
    }

    fn set_drop(&mut self, key: &str) {
        match self.r.drops.iter_mut().find(|e| e.0 == key) {
            Some(e) => e.1 = 0.0,
            None => self.r.drops.push((key.to_string(), 0.0)),
        }
    }

    fn r_place(&mut self, key: &str) {
        let avail = self.r_available();
        if avail <= 0 {
            self.say(t("cas.not_enough"));
            self.b.play_sound("hit");
            return;
        }
        let amount = self.r.chip.min(avail);
        match self.r.bets.iter_mut().find(|e| e.0 == key) {
            Some(e) => e.1 += amount,
            None => self.r.bets.push((key.to_string(), amount)),
        }
        self.set_drop(key);
        self.b.play_sound("click");
    }

    fn r_remove(&mut self, key: &str) {
        let Some(i) = self.r.bets.iter().position(|e| e.0 == key) else {
            return;
        };
        let left = self.r.bets[i].1 - self.r.chip;
        if left > 0 {
            self.r.bets[i].1 = left;
        } else {
            self.r.bets.remove(i);
        }
        self.b.play_sound("move");
    }

    fn r_clear(&mut self) {
        if !self.r.bets.is_empty() {
            self.r.bets.clear();
            self.b.play_sound("move");
        }
    }

    fn r_rebet(&mut self) {
        if self.r.last.is_empty() {
            return;
        }
        let need: i64 = self.r.last.iter().map(|e| e.1).sum();
        if need > lamabank::balance() {
            self.say(t("cas.not_enough"));
            self.b.play_sound("hit");
            return;
        }
        self.r.bets = self.r.last.clone();
        let keys: Vec<String> = self.r.bets.iter().map(|e| e.0.clone()).collect();
        for key in keys {
            self.set_drop(&key);
        }
        self.b.play_sound("click");
    }

    fn r_double(&mut self) {
        if self.r.bets.is_empty() {
            return;
        }
        if self.r_total() * 2 > lamabank::balance() {
            self.say(t("cas.not_enough"));
            self.b.play_sound("hit");
            return;
        }
        for e in self.r.bets.iter_mut() {
            e.1 *= 2;
        }
        let keys: Vec<String> = self.r.bets.iter().map(|e| e.0.clone()).collect();
        for key in keys {
            self.set_drop(&key);
        }
        self.b.play_sound("merge");
    }

    fn r_spin(&mut self) {
        let stake = self.r_total();
        if stake <= 0 {
            self.say(t("cas.r.place_first"));
            return;
        }
        if !lamabank::debit(stake, "casino") {
            self.say(t("cas.not_enough"));
            self.b.play_sound("hit");
            return;
        }
        self.r.stake = stake;
        self.r.last = self.r.bets.clone();
        let number = self.rng.randrange(0, 37);
        self.r.number = Some(number);
        let (payout, winners) = l::settle(&self.r.bets, number);
        self.r.payout = payout;
        self.r.winners = winners;
        lamabank::credit(payout, "casino", true);
        self.pending += payout;
        self.r.omega = SPIN_OMEGA;
        let beta0 = self.rng.uniform(0.0, TAU);
        self.r.phi0 = beta0 - self.r.rot;
        let target = self.r.wheel.as_ref().map(|w| w.pocket_angle(number)).unwrap_or(0.0);
        let dd = py::fpmod(self.r.phi0 - target, TAU);
        self.r.phi_end = self.r.phi0 - dd - BALL_TURNS * TAU;
        self.r.t = 0.0;
        self.r.fast = false;
        self.r.trail.clear();
        self.r.tick_i = None;
        self.r.phase = RPhase::Spin;
        self.r.msg = Some((t("cas.r.no_more"), ui::pal().text_dim));
        self.b.play_sound("shoot");
    }

    fn r_ball_state(&self, u: f64) -> (f64, f64) {
        let Some(wa) = self.r.wheel.as_ref() else {
            return (0.0, 0.0);
        };
        let rr = wa.r;
        let e = py::pow(1.0 - u, 3.0);
        let mut phi = self.r.phi_end + (self.r.phi0 - self.r.phi_end) * e;
        if u > 0.62 {
            let w = (u - 0.62) / 0.38;
            phi += 0.16 * (u * 53.0).sin() * (1.0 - u) * w;
        }
        let rim = rr * 0.74;
        let r;
        if u < 0.52 {
            r = wa.track_r - rr * 0.004 * (u * 60.0).sin();
        } else if u < 0.80 {
            let v = (u - 0.52) / 0.28;
            let v = v * v * (3.0 - 2.0 * v);
            r = wa.track_r + (rim - wa.track_r) * v;
        } else {
            let v = (u - 0.80) / 0.20;
            let base = rim + (wa.pocket_r - rim) * (v * v * (3.0 - 2.0 * v));
            let hop = rr * 0.06 * (v * 3.2 * PI).sin().abs() * py::pow(1.0 - v, 1.5);
            r = base + hop;
        }
        (phi, r)
    }

    fn r_land(&mut self) {
        let n = self.r.number.unwrap_or(0);
        self.pending = 0.max(self.pending - self.r.payout);
        self.r.phase = RPhase::Result;
        self.r.res_t = 0.0;
        self.r.history.insert(0, n);
        self.r.history.truncate(l::HISTORY_LEN);
        lamabank::set_extra(HISTORY_KEY, json!(self.r.history));
        self.b.score = lamabank::score_for("casino");
        self.b.play_sound("lock");
        if self.r.payout > 0 {
            let plein = self.r.winners.iter().any(|k| l::bet_kind(k) == "plein");
            let big = plein || self.r.payout >= 15 * self.r.stake;
            self.r.msg = Some((format!("{}: +{}", t("cas.win"), self.r.payout), ui::pal().gold));
            self.b.play_sound(if big { "win" } else { "point" });
            for key in self.r.winners.clone() {
                let (ax, ay) = l::anchor(&key);
                let (ax, ay) = self.r_to_px(ax, ay);
                let amount = get(&self.r.bets, &key).unwrap_or(0) * (l::payout_multiplier(&key) + 1);
                let y = ay - self.r.chip_r as f64 * 1.5;
                self.popup(ax, y, format!("+{amount}"));
            }
            if plein {
                self.b.ach_event("roulette_plein", None);
            }
            if big {
                ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 90);
                self.coin_shower(if plein { 46 } else { 30 });
                self.b.rumble(160);
            }
        } else {
            self.r.msg = Some((t("cas.no_win"), ui::pal().text_dim));
            self.b.play_sound("select");
        }
    }

    fn r_new_round(&mut self) {
        self.r.phase = RPhase::Bet;
        self.r.bets.clear();
        self.r.drops.clear();
        self.r.msg = None;
        self.check_broke();
    }

    // ===================================================== Eingabe

    pub(crate) fn r_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        if kind == Kind::MouseMove {
            self.r.hover = if self.r.phase == RPhase::Bet { self.r_key_at(ev.pos) } else { None };
            return;
        }
        let enter = ["Return", "space", "KP_Enter"].contains(&ev.key.as_str());
        if self.r.phase == RPhase::Spin {
            if kind == Kind::MouseDown || (kind == Kind::KeyDown && enter) {
                self.r.fast = true;
            }
            return;
        }
        if self.r.phase == RPhase::Result {
            if kind == Kind::MouseDown && ev.button == 1 {
                let hit = self.r.btns.iter().find(|(_, r)| r.hit(ev.pos)).map(|e| e.0);
                self.r_new_round();
                if hit == Some("rebet") {
                    self.r_rebet();
                } else if hit == Some("spin") {
                    self.r_rebet();
                    if !self.r.bets.is_empty() && !self.broke {
                        self.r_spin();
                    }
                }
            } else if kind == Kind::KeyDown && (enter || ev.key == "r" || ev.key == "R") {
                self.r_new_round();
                if ev.key == "r" || ev.key == "R" {
                    self.r_rebet();
                }
            }
            return;
        }
        match kind {
            Kind::Wheel => {
                let i = l::CHIP_VALUES.iter().position(|v| *v == self.r.chip).unwrap_or(1) as i64;
                let i = 0.max((l::CHIP_VALUES.len() as i64 - 1).min(i + if ev.delta > 0 { 1 } else { -1 }));
                self.r_select_chip(l::CHIP_VALUES[i as usize]);
            }
            Kind::KeyDown => {
                let key = ev.key.as_str();
                if ["1", "2", "3", "4", "5"].contains(&key) {
                    let i = key.parse::<usize>().unwrap_or(1) - 1;
                    self.r_select_chip(l::CHIP_VALUES[i]);
                } else if enter {
                    self.r_spin();
                } else if ["BackSpace", "Delete", "c", "C"].contains(&key) {
                    self.r_clear();
                } else if key == "r" || key == "R" {
                    self.r_rebet();
                } else if key == "d" || key == "D" {
                    self.r_double();
                }
            }
            Kind::MouseDown => {
                let pos = ev.pos;
                if ev.button == 3 {
                    if let Some(key) = self.r_key_at(pos) {
                        self.r_remove(&key);
                    }
                    return;
                }
                if let Some(&(value, _)) = self.r.chip_rects.iter().find(|(_, r)| r.hit(pos)) {
                    self.r_select_chip(value);
                    return;
                }
                if let Some(&(name, _)) = self.r.btns.iter().find(|(_, r)| r.hit(pos)) {
                    match name {
                        "clear" => self.r_clear(),
                        "double" => self.r_double(),
                        "rebet" => self.r_rebet(),
                        _ => self.r_spin(),
                    }
                    return;
                }
                if let Some(key) = self.r_key_at(pos) {
                    self.r_place(&key);
                }
            }
            _ => {}
        }
    }

    // ===================================================== Update

    pub(crate) fn r_update(&mut self, dt: f64) {
        self.r.omega += (IDLE_OMEGA - self.r.omega) * (1.0 - (-dt / OMEGA_TAU).exp());
        for e in self.r.drops.iter_mut() {
            e.1 += dt * 5.0;
        }
        self.r.drops.retain(|e| e.1 < 1.0);
        if self.r.phase == RPhase::Spin {
            let step = dt * if self.r.fast { 3.5 } else { 1.0 };
            self.r.rot += self.r.omega * step;
            self.r.t += step;
            let u = 1f64.min(self.r.t / LAND_T);
            let (phi, rad) = self.r_ball_state(u);
            let ball = (self.r.rot + phi, rad);
            if u < 0.9 {
                self.r.trail.push(ball);
                if self.r.trail.len() > 4 {
                    let cut = self.r.trail.len() - 4;
                    self.r.trail.drain(..cut);
                }
            } else {
                self.r.trail.clear();
            }
            self.r.ball = Some(ball);
            if u > 0.55 {
                let idx = (phi / (TAU / 37.0)).floor() as i64;
                self.r.tick_cd -= dt;
                if self.r.tick_i.is_some() && Some(idx) != self.r.tick_i && self.r.tick_cd <= 0.0 {
                    self.r.tick_cd = 0.05 + 0.1 * u;
                    let freq = 1900.0 + 400.0 * self.rng.random();
                    audio::tone(freq, 0.015, Some(&self.b.settings), "square", 0.05);
                }
                self.r.tick_i = Some(idx);
            }
            if u >= 1.0 {
                self.r_land();
            }
        } else {
            self.r.rot += self.r.omega * dt;
            if let Some(n) = self.r.number {
                let pa = self.r.wheel.as_ref().map(|w| (w.pocket_angle(n), w.pocket_r)).unwrap_or((0.0, 0.0));
                self.r.ball = Some((self.r.rot + pa.0, pa.1));
            }
            if self.r.phase == RPhase::Result {
                self.r.res_t += dt;
                if self.r.res_t >= RESULT_T {
                    self.r_new_round();
                }
            } else if self.r.hover.is_none() && self.r.phase == RPhase::Bet {
                self.r.hover = self.r_key_at(self.mouse);
            }
        }
    }

    // ===================================================== Zeichnen

    pub(crate) fn r_draw(&mut self, s: &mut Surface) {
        if let Some(f) = self.r.felt.as_ref() {
            s.bl(f, (0, 0));
        }
        if let Some(w) = self.r.wheel.as_ref() {
            let trail: &[(f64, f64)] = if self.r.phase == RPhase::Spin { &self.r.trail } else { &[] };
            w.draw(s, self.r.wheel_c, self.r.rot, self.r.ball, trail);
        }
        self.r_draw_panel(s);
        self.r_draw_table(s);
        self.r_draw_strip(s);
    }

    fn r_cell_rect(&self, n: i64) -> Rect {
        let (uw, uh) = (self.r.uw, self.r.uh);
        if n == 0 {
            return Rect::new(self.r.tx as i32, self.r.ty as i32, uw as i32, (3.0 * uh) as i32);
        }
        let (x, y) = self.r_to_px((1 + l::num_col(n)) as f64, l::num_row(n) as f64);
        Rect::new(x as i32, y as i32, uw as i32, uh as i32)
    }

    fn r_key_rect(&self, key: &str) -> Option<Rect> {
        let kind = l::bet_kind(key);
        let arg = || key.split_once(':').map(|e| e.1).unwrap_or("0").parse::<i64>().unwrap_or(0) as f64;
        if kind == "column" {
            let (x, y) = self.r_to_px(13.0, arg());
            return Some(Rect::new(x as i32, y as i32, self.r.uw as i32, self.r.uh as i32));
        }
        if kind == "dozen" {
            let (x, y) = self.r_to_px(1.0 + 4.0 * arg(), 3.0);
            return Some(Rect::new(x as i32, y as i32, (self.r.uw * 4.0) as i32, (self.r.uh * l::DOZEN_H) as i32));
        }
        if let Some(i) = l::OUTSIDE_KEYS.iter().position(|k| *k == kind) {
            let (x, y) = self.r_to_px(1.0 + 2.0 * i as f64, 3.0 + l::DOZEN_H);
            return Some(Rect::new(x as i32, y as i32, (self.r.uw * 2.0) as i32, (self.r.uh * l::OUTSIDE_H) as i32));
        }
        None
    }

    fn r_highlight(&self, s: &mut Surface, key: &str, surf: &Surface) {
        if let Some(rect) = self.r_key_rect(key) {
            if (rect.w, rect.h) != (surf.w(), surf.h()) {
                let img = transform::scale(surf, rect.w, rect.h);
                s.blr(&img, rect);
            } else {
                s.blr(surf, rect);
            }
        }
        for n in l::numbers(key) {
            let r = self.r_cell_rect(n);
            if n == 0 {
                let img = transform::scale(surf, r.w, r.h);
                s.blr(&img, r);
            } else {
                s.blr(surf, r);
            }
        }
    }

    fn r_draw_table(&self, s: &mut Surface) {
        let (ox, oy) = self.r.table_off;
        if let Some(tb) = self.r.table.as_ref() {
            s.bl(tb, (self.r.tx as i32 - ox, self.r.ty as i32 - oy));
        }
        let cr = self.r.chip_r;
        if self.r.phase == RPhase::Bet {
            if let (Some(h), Some(hl)) = (self.r.hover.as_ref(), self.r.hl.as_ref()) {
                self.r_highlight(s, h, hl);
            }
        }
        if self.r.phase == RPhase::Result {
            if let Some(number) = self.r.number {
                if let Some(whl) = self.r.win_hl.as_ref() {
                    for key in &self.r.winners {
                        if self.r_key_rect(key).is_some() {
                            self.r_highlight(s, key, whl);
                        }
                    }
                }
                let r = self.r_cell_rect(number);
                let pulse = ui::pulse(6.0, 0.0, 1.0);
                draw::rect_r(s, ui::mix(d::COL_GOLD, (255, 255, 255), pulse), r.inflate(4, 4), 2.max((3.0 * self.k) as i32), 4);
                let (dx, dy) = (r.centerx(), r.centery());
                let dr = 5.max((self.r.uw.min(self.r.uh) * 0.22) as i32);
                draw::ellipse(s, (0, 0, 0), (dx - dr, dy - dr / 2 + 3, 2 * dr, dr), 0);
                let (dxf, dyf, drf) = (dx as f64, dy as f64, dr as f64);
                draw::rect(s, (230, 230, 236), (dxf - drf * 0.6, dyf - drf * 1.4, drf * 1.2, drf * 1.4), 0);
                draw::ellipse(s, (250, 250, 255), (dxf - drf * 0.6, dyf - drf * 1.7, drf * 1.2, drf * 0.6), 0);
            }
        }
        for (key, amount) in &self.r.bets {
            let (ax, ay) = l::anchor(key);
            let (ax, ay) = self.r_to_px(ax, ay);
            let mut alpha = 255;
            let mut lift = 0;
            if let Some(e) = self.r.drops.iter().find(|e| &e.0 == key) {
                let fr = e.1;
                lift = (py::pow(1.0 - fr, 2.0) * cr as f64 * 1.4) as i32;
            }
            let winner = self.r.winners.contains(key);
            if self.r.phase == RPhase::Result && !winner {
                alpha = 0.max((255.0 * (1.0 - self.r.res_t / 0.8)) as i32);
                if alpha <= 0 {
                    continue;
                }
            }
            d::draw_chip_stack(s, (ax, ay), *amount, cr, alpha, lift);
            if self.r.phase == RPhase::Result && winner {
                let ring = (cr as f64 + 3.0 + 3.0 * ui::pulse(5.0, 0.0, 1.0)) as i32;
                draw::circle(s, d::COL_GOLD, (ax as i32, ay as i32 - 2), ring, 2);
            }
        }
        if self.r.phase == RPhase::Bet {
            if let Some(h) = self.r.hover.as_ref() {
                if get(&self.r.bets, h).is_none() {
                    let (ax, ay) = l::anchor(h);
                    let (ax, ay) = self.r_to_px(ax, ay);
                    let img = d::chip_surface(self.r.chip, cr, None);
                    let mut ghost = img.copy();
                    ghost.set_alpha(Some(130));
                    s.blr(&ghost, ghost.get_rect().center_at((ax as i32, ay as i32)));
                }
            }
        }
    }

    fn r_draw_panel(&self, s: &mut Surface) {
        let pl = ui::pal();
        let p = self.r.panel;
        if p.w < 40 || p.h < 40 {
            return;
        }
        let k = self.k;
        let mut y = p.y;
        let title = self.f_tiny.render(&t("cas.r.history"), true, pl.text_dim);
        s.bl(&title, (p.x, y));
        y += title.h() + 2.max((4.0 * k) as i32);
        let gap = 2.max((4.0 * k) as i32);
        let mut per_row = 12;
        let mut size = ((p.w - gap * (per_row - 1)) as f64 / per_row as f64) as i32;
        if size < 18 {
            per_row = 6;
            size = ((p.w - gap * (per_row - 1)) as f64 / per_row as f64) as i32;
        }
        size = 12.max(size.min((34.0 * k) as i32));
        let hfont = ui::font(9.max((size as f64 * 0.52) as i32), true, false);
        for i in 0..l::HISTORY_LEN as i32 {
            let cx = p.x + (i % per_row) * (size + gap);
            let cy = y + (i / per_row) * (size + gap);
            let rect = Rect::new(cx, cy, size, size);
            if let Some(&n) = self.r.history.get(i as usize) {
                let col = d::number_color(n);
                draw::rect_r(s, col, rect, 0, 3.max(size / 4));
                if i == 0 {
                    draw::rect_r(s, d::COL_GOLD, rect, 2, 3.max(size / 4));
                }
                let img = hfont.render(&n.to_string(), true, (255, 255, 255));
                s.blr(&img, img.get_rect().center_at(rect.center()));
            } else {
                draw::rect_r(s, ui::mix(d::COL_FELT_D, (0, 0, 0), 0.3), rect, 1, 3.max(size / 4));
            }
        }
        let rows = (l::HISTORY_LEN as i32 + per_row - 1) / per_row;
        y += rows * (size + gap) + 6.max((12.0 * k) as i32);
        let show_n = if self.r.phase == RPhase::Result { self.r.number } else { None };
        let big_r = 14.max(((34.0 * k) as i32).min(py::fdiv32(p.bottom() - y, 3)));
        if let Some(n) = show_n {
            let c = (p.x + big_r, y + big_r);
            draw::circle(s, (0, 0, 0), (c.0 + 2, c.1 + 3), big_r, 0);
            draw::circle(s, d::number_color(n), c, big_r, 0);
            draw::circle(s, d::COL_GOLD, c, big_r, 2);
            let img = ui::font(12.max((big_r as f64 * 0.95) as i32), true, false).render(&n.to_string(), true, (255, 255, 255));
            s.blr(&img, img.get_rect().center_at(c));
            let txt = Self::r_number_text(n);
            let mut desc = self.f_small.render(&txt, true, pl.text);
            if desc.w() > p.w - 2 * big_r - 10 {
                desc = self.f_tiny.render(&txt, true, pl.text);
            }
            s.blr(&desc, desc.get_rect().midleft_at((p.x + 2 * big_r + 10, c.1)));
            y += 2 * big_r + 4.max((8.0 * k) as i32);
        }
        let mut msg = self.r.msg.clone();
        if msg.is_none() && self.r.phase == RPhase::Bet {
            msg = Some((t("cas.r.place_bets"), pl.text));
        }
        if let Some((text, col)) = msg {
            if y < p.bottom() {
                let mut img = self.f_big.render(&text, true, col);
                if img.w() > p.w {
                    img = self.f_small.render(&text, true, col);
                }
                s.bl(&img, (p.x, y));
                y += img.h() + 2.max((4.0 * k) as i32);
            }
        }
        let mut info = None;
        if self.r.phase == RPhase::Bet && self.r.hover.is_some() {
            let key = self.r.hover.clone().unwrap_or_default();
            let mut txt = format!("{} \u{00B7} {}:1", Self::r_bet_name(&key), l::payout_multiplier(&key));
            if let Some(v) = get(&self.r.bets, &key) {
                txt += &format!(" \u{00B7} {v}");
            }
            info = Some(txt);
        } else if self.r.phase == RPhase::Spin {
            info = Some(t("cas.r.skip_hint"));
        } else if self.r.phase == RPhase::Bet {
            info = Some(t("cas.r.place_hint"));
        }
        if let Some(info) = info.filter(|i| !i.is_empty()) {
            let mut img = self.f_tiny.render(&info, true, pl.text_dim);
            if img.w() > p.w {
                img = ui::font(9.max(self.f_tiny.get_height() - 3), false, false).render(&info, true, pl.text_dim);
            }
            let yy = y.max(p.bottom() - img.h());
            if yy + img.h() <= p.bottom() + 2 {
                s.bl(&img, (p.x, yy));
            }
        }
    }

    fn r_draw_strip(&self, s: &mut Surface) {
        self.draw_strip_bg(s);
        let cr = self.r.chip_rad;
        for &(value, rect) in &self.r.chip_rects {
            let sel = value == self.r.chip;
            let c = rect.center();
            let lift = if sel { (5.0 * self.k) as i32 } else { 0 };
            if sel {
                let glow = ui::mix(d::COL_GOLD, (255, 255, 255), ui::pulse(3.0, 0.0, 0.5));
                draw::circle(s, glow, (c.0, c.1 - lift), cr + 3, 2);
            }
            let img = d::chip_surface(value, cr - if sel { 0 } else { 2 }, None);
            s.blr(&img, img.get_rect().center_at((c.0, c.1 - lift)));
        }
        let betting = self.r.phase == RPhase::Bet;
        let result = self.r.phase == RPhase::Result;
        let has_bets = !self.r.bets.is_empty();
        let has_last = !self.r.last.is_empty();
        for &(key, rect) in &self.r.btns {
            let on = match key {
                "clear" | "double" => betting && has_bets,
                "rebet" => (betting || result) && has_last,
                _ => (betting && has_bets) || (result && has_last),
            };
            self.btn(s, rect, &Self::r_btn_label(key), key == "spin", on, self.hot(rect), false);
        }
    }
}
