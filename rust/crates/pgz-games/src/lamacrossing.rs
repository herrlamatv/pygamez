//! lamacrossing.rs
//! ===============
//! Lama Crossing - Mut-Spiel mit Lama-Chips, uebertragen aus
//! `games/lamacrossing.py` (2.0). Das Lama ueberquert Spur fuer Spur eine
//! Strasse, eine Haengebruecke, einen Fluss oder einen Eissee; jede geschaffte
//! Spur hebt den Multiplikator. Auszahlen nach jeder Spur, am anderen Ufer
//! automatisch. Regeln: [`crate::lamacrossing_logic`], Zeichnen:
//! [`crate::lamacrossing_draw`].
//!
//! Zwei Gefahren: Hindernisse (Timing - wer losgeht, waehrend auf der Zielspur
//! eins kommt, liegt platt da; die Spur, auf der das Lama steht, ist
//! geschuetzt) und der Zufall je Spur (Strasse: ein Raser, sonst bricht die
//! Planke, rutscht der Stein weg bzw. bricht das Eis).
//!
//! Konto: gemeinsame Lama-Bank (mit Blackjack, Poker, Casino). Einsatz bei LOS
//! abbuchen; eine geschaffte Spur wird beim Landen gespeichert, ein Zufalls-
//! Fehltritt sofort - wer mitten in der Ueberquerung geht, macht beim
//! naechsten Start auf der letzten sicheren Spur weiter.

use std::f64::consts::{PI, TAU};

use pgz_core::{i18n, lamabank};
use pgz_engine::prelude::*;
use serde_json::{Map, Value};

use crate::casino_draw as cd;
use crate::lamacrossing_draw as d;
use crate::lamacrossing_logic as l;

const KEY: &str = "lamacrossing";
const HOP_T: f64 = 0.36;
const FALL_T: f64 = 1.25;
const RESULT_T: f64 = 3.2;
/// Hindernisse auf einer gesperrten Spur blenden aus.
const FADE_T: f64 = 0.25;

/// Pythons `max(a, b)` fuer Gleitkommazahlen (bei Gleichstand das erste).
fn pmax(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// Pythons `min(a, b)` fuer Gleitkommazahlen (bei Gleichstand das erste).
fn pmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Idle,
    Hop,
    Wait,
    Fall,
    Cashed,
    Goal,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Outcome {
    None,
    Ok,
    Fail,
}

/// `fall_kind`: "break" = Zufall (Planke, Stein, Eis), "hit" = platt.
#[derive(Clone, Copy, PartialEq, Eq)]
enum FallKind {
    Break,
    Hit,
}

#[derive(Clone, Copy)]
struct Hop {
    a: i32,
    b: i32,
    t: f64,
}

/// Ein Hindernis (`dict(y, v, kind, stop, fade)`).
struct Obs {
    y: f64,
    v: f64,
    kind: i32,
    stop: Option<f64>,
    fade: Option<f64>,
}

/// Eine Spur (`dict(v, t, obs, closed)`): festes Tempo, Zeit bis zum
/// naechsten Hindernis.
struct Lane {
    v: f64,
    t: f64,
    obs: Vec<Obs>,
    closed: bool,
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
}

struct Ripple {
    x: f64,
    y: f64,
    t: f64,
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

/// `LamaCrossingGame`
pub struct LamaCrossingGame {
    b: GameBase,
    rng: py::Random,
    world: String,
    diff: String,
    bet: i64,
    hist: Vec<i64>,
    round: Option<l::Round>,
    step: i32,
    state: State,
    hop: Option<Hop>,
    outcome: Outcome,
    fall_kind: FallKind,
    broken: i32,
    lanes: Vec<Lane>,
    fall_t: f64,
    res_t: f64,
    result: Option<(String, String, Color)>,
    win: i64,
    lost: i64,
    pending: i64,
    shown_chips: f64,
    particles: Vec<Particle>,
    ripples: Vec<Ripple>,
    coins_fx: Vec<CoinFx>,
    popups: Vec<Popup>,
    toast: Option<(String, f64)>,
    shake: f64,
    time: f64,
    mouse: (i32, i32),
    broke: bool,
    tick_cd: f64,
    lang: String,
    cam: f64,
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
    scene: Rect,
    path_y: i32,
    gap: i32,
    tw: i32,
    llama_n: i32,
    hop_h: i32,
    sag: i32,
    hud_bg: Surface,
    strip_bg: Surface,
    overlay: Surface,
    r_minus: Rect,
    r_bet: Rect,
    r_plus: Rect,
    r_go: Rect,
    r_cash: Rect,
    r_diffs: Vec<Rect>,
    r_worlds: Vec<Rect>,
}

impl LamaCrossingGame {
    /// `LamaCrossingGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = LamaCrossingGame {
            b,
            rng: py::Random::new(0),
            world: "road".into(),
            diff: "medium".into(),
            bet: 10,
            hist: Vec::new(),
            round: None,
            step: 0,
            state: State::Idle,
            hop: None,
            outcome: Outcome::None,
            fall_kind: FallKind::Break,
            broken: 0,
            lanes: Vec::new(),
            fall_t: 0.0,
            res_t: 0.0,
            result: None,
            win: 0,
            lost: 0,
            pending: 0,
            shown_chips: 0.0,
            particles: Vec::new(),
            ripples: Vec::new(),
            coins_fx: Vec::new(),
            popups: Vec::new(),
            toast: None,
            shake: 0.0,
            time: 0.0,
            mouse: (-1, -1),
            broke: false,
            tick_cd: 0.0,
            lang: String::new(),
            cam: 0.0,
            k: 1.0,
            f_tiny: f.clone(),
            f_small: f.clone(),
            f_btn: f.clone(),
            f_big: f.clone(),
            f_huge: f,
            margin: 0,
            hud_h: 0,
            strip_h: 0,
            strip: Rect::new(0, 0, 0, 0),
            scene: Rect::new(0, 0, 0, 0),
            path_y: 0,
            gap: 0,
            tw: 0,
            llama_n: 0,
            hop_h: 0,
            sag: 0,
            hud_bg: pg::surface_alpha(1, 1),
            strip_bg: pg::surface_alpha(1, 1),
            overlay: pg::surface_alpha(1, 1),
            r_minus: Rect::new(0, 0, 0, 0),
            r_bet: Rect::new(0, 0, 0, 0),
            r_plus: Rect::new(0, 0, 0, 0),
            r_go: Rect::new(0, 0, 0, 0),
            r_cash: Rect::new(0, 0, 0, 0),
            r_diffs: Vec::new(),
            r_worlds: Vec::new(),
        };
        g.reset();
        Box::new(g)
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.k = (w as f64 / 800.0).min(h as f64 / 600.0);
        let k = self.k;
        self.f_tiny = ui::font(11.max((13.0 * k) as i32), false, false);
        self.f_small = ui::font(12.max((15.0 * k) as i32), false, false);
        self.f_btn = ui::font(12.max((15.0 * k) as i32), true, false);
        self.f_big = ui::font(15.max((22.0 * k) as i32), true, false);
        self.f_huge = ui::font(24.max((48.0 * k) as i32), true, false);
        self.margin = 6.max((12.0 * k) as i32);
        let m = self.margin;
        self.hud_h = 28.max((42.0 * k) as i32);
        self.strip_h = 46.max((64.0 * k) as i32);
        self.strip = Rect::new(0, h - self.strip_h, w, self.strip_h);
        self.scene = Rect::new(0, self.hud_h, w, self.strip.y - self.hud_h);
        self.path_y = self.scene.y + (self.scene.h as f64 * 0.56) as i32;
        self.gap = 44.max((118.0 * k) as i32);
        self.tw = (self.gap as f64 * 0.62) as i32;
        self.llama_n = 30.max((72.0 * k) as i32);
        self.hop_h = (44.0 * k) as i32;
        self.sag = (16.0 * k) as i32;
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
        // Bedienleiste: Einsatz links, LOS/WEITER + AUSZAHLEN rechts
        let sh = self.strip_h;
        let bh = 30.max((sh as f64 * 0.62) as i32);
        let by = self.strip.y + (sh - bh) / 2;
        let lbl_w = self.f_tiny.width(&t("cas.bet")).max(self.f_btn.width("10000")) + (14.0 * k) as i32;
        self.r_minus = Rect::new(m, by, bh, bh);
        self.r_bet = Rect::new(self.r_minus.right() + 4, by, lbl_w, bh);
        self.r_plus = Rect::new(self.r_bet.right() + 4, by, bh, bh);
        let pad = (22.0 * 0.7f64.max(k)) as i32;
        let mut go_w = self.f_btn.width(&t("lcr.go")).max(self.f_btn.width(&t("lcr.step"))) + pad;
        go_w = go_w.max((110.0 * k) as i32);
        let mut cash_w = self.f_btn.width(&format!("{} 999999999", t("lcr.cashout"))) + pad;
        let room = w - m - (self.r_plus.right() + m) - 4.max((8.0 * k) as i32);
        if go_w + cash_w > room {
            let f = room as f64 / (go_w + cash_w) as f64;
            go_w = (go_w as f64 * f) as i32;
            cash_w = (cash_w as f64 * f) as i32;
        }
        self.r_go = Rect::new(w - m - go_w, by, go_w, bh);
        self.r_cash = Rect::new(self.r_go.x - 4.max((8.0 * k) as i32) - cash_w, by, cash_w, bh);
        // Schwierigkeit (nur vor der Runde) direkt ueber der Leiste
        let bar_h = 26.max((36.0 * k) as i32);
        let bar_y = self.strip.y - m - bar_h;
        let bw = ((150.0 * k) as i32).min((w - 2 * m - 3 * 6) / 4);
        let x0 = (w - (4 * bw + 3 * 6)) / 2;
        self.r_diffs = (0..4).map(|i| Rect::new(x0 + i * (bw + 6), bar_y, bw, bar_h)).collect();
        // Welt-Knoepfe oben links
        let wb = 26.max((36.0 * k) as i32);
        self.r_worlds = (0..l::WORLDS.len() as i32).map(|i| Rect::new(m + i * (wb + 4), self.hud_h + m, wb, wb)).collect();
    }

    // ===================================================== Einstellungen

    fn set_opt(&mut self, key: &str, value: Value) {
        if self.b.opt_value(KEY, key).as_ref() != Some(&value) {
            self.b.set_opt(KEY, key, value);
        }
    }

    // ===================================================== Konto / Runde

    fn save_extra(&self) {
        let mut m = Map::new();
        m.insert("round".into(), self.round.as_ref().map(|r| r.to_value()).unwrap_or(Value::Null));
        m.insert("hist".into(), Value::from(self.hist.clone()));
        lamabank::set_extra(KEY, Value::Object(m));
    }

    /// Einsatz auf die hoechste bezahlbare Stufe senken (mind. Stufe 1).
    fn clamp_bet(&mut self) {
        let chips = lamabank::balance();
        if self.bet > chips {
            let ok: Vec<i64> = l::BETS.iter().copied().filter(|b| *b <= chips).collect();
            self.bet = ok.last().copied().unwrap_or(l::BETS[0]);
        }
    }

    fn check_broke(&mut self) {
        self.broke = self.state == State::Idle && lamabank::is_broke(KEY, None);
        if self.broke {
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

    fn fmt(&self, m100: i64) -> String {
        format!("\u{00D7}{}", l::fmt_mult(m100, &t("lcr.decimal")))
    }

    // ===================================================== Geometrie

    fn n(&self) -> i32 {
        l::steps(&self.diff)
    }

    /// Weltkoordinate x der Spur-Mitte (0 = Startufer, n+1 = Zielufer).
    fn tile_x(&self, i: i32) -> i32 {
        (self.b.width as f64 * 0.22) as i32 + i * self.gap
    }

    fn tile_y(&self, i: i32) -> i32 {
        let n = self.n();
        if self.world == "bridge" && 1 <= i && i <= n {
            return self.path_y + (self.sag as f64 * (PI * i as f64 / (n + 1) as f64).sin()) as i32;
        }
        self.path_y
    }

    /// (x, y) der Fuesse in Weltkoordinaten + Pose.
    fn llama_pos(&self) -> (f64, f64, &'static str) {
        if self.state == State::Hop {
            if let Some(hp) = self.hop {
                let p = 1f64.min(hp.t / HOP_T);
                let e = p * p * (3.0 - 2.0 * p);
                let x = self.tile_x(hp.a) as f64 + (self.tile_x(hp.b) - self.tile_x(hp.a)) as f64 * e;
                let y = self.tile_y(hp.a) as f64 + (self.tile_y(hp.b) - self.tile_y(hp.a)) as f64 * p;
                return (x, y - (PI * p).sin() * self.hop_h as f64, "hop");
            }
        }
        let x = self.tile_x(self.step) as f64;
        let mut y = self.tile_y(self.step) as f64;
        if self.state == State::Fall {
            if self.fall_kind == FallKind::Hit {
                return (x, y, "flat");
            }
            let f = self.fall_t;
            if self.world == "bridge" {
                return (x, y + 0.5 * 1500.0 * self.k * f * f, "fall");
            }
            return (x, y + 46.0 * self.k * 1f64.min(f * 1.6), "fall");
        }
        if matches!(self.state, State::Wait | State::Idle) && self.world == "bridge" && self.step > 0 {
            y += (self.time * 3.1).sin() * 1.2 * self.k;
        }
        (x, y, "stand")
    }

    fn cam_target(&self) -> f64 {
        let n = self.n();
        let lx = match (self.state, self.hop) {
            (State::Hop, Some(hp)) => self.tile_x(hp.b),
            _ => self.tile_x(self.step),
        };
        let hi = 0.max(self.tile_x(n + 1) + (self.gap as f64 * 1.3) as i32 - self.b.width);
        0f64.max((hi as f64).min(lx as f64 - self.b.width as f64 * 0.32))
    }

    // ===================================================== Verkehr

    /// Oberkante der Hindernis-Bahn: Horizont (Fluss/Eis) bzw. ueber der Szene.
    fn horizon(&self) -> i32 {
        if self.world == "river" {
            return self.path_y - (self.b.height as f64 * 0.10) as i32;
        }
        if self.world == "ice" {
            return self.path_y - (self.b.height as f64 * 0.08) as i32;
        }
        self.scene.y - (self.gap as f64 * 1.5) as i32
    }

    /// Groesse eines Hindernisses auf Hoehe y (Fluss/Eis perspektivisch).
    fn obs_size(&self, y: f64) -> i32 {
        if self.world == "river" || self.world == "ice" {
            let top = self.horizon();
            let f = pmax(0.0, pmin(1.0, (y - top as f64) / 1.max(self.b.height - top) as f64));
            return (self.gap as f64 * (0.45 + 0.80 * f)) as i32;
        }
        self.gap
    }

    /// Hoehe der Treffer-Mitte des Lamas.
    fn target_y(&self) -> f64 {
        self.path_y as f64 - self.llama_n as f64 * 0.35
    }

    /// Jede Spur 1..n: festes Tempo, Startzeit zufaellig versetzt.
    fn init_traffic(&mut self) {
        let (v_lo, v_hi, _gap_lo, gap_hi) = l::traffic(&self.diff);
        self.lanes = vec![Lane { v: 0.0, t: 0.0, obs: Vec::new(), closed: false }];
        for i in 1..=self.n() {
            let v = self.rng.uniform(v_lo, v_hi) * self.k;
            let tm = self.rng.uniform(0.0, gap_hi);
            let closed = i <= self.step && self.state != State::Idle;
            self.lanes.push(Lane { v, t: tm, obs: Vec::new(), closed });
        }
    }

    fn spawn_obstacle(&mut self, lane: usize, kind: i32, v: f64, y: Option<f64>) {
        let y = match y {
            Some(y) => y,
            None => {
                let top = self.horizon();
                let size = self.obs_size(top as f64);
                if self.world == "river" || self.world == "ice" {
                    top as f64
                } else {
                    top as f64 - d::obstacle_len(&self.world, kind, size) as f64 / 2.0
                }
            }
        };
        if let Some(ln) = self.lanes.get_mut(lane) {
            ln.obs.push(Obs { y, v, kind, stop: None, fade: None });
        }
    }

    fn update_traffic(&mut self, dt: f64) {
        let (_v_lo, _v_hi, gap_lo, gap_hi) = l::traffic(&self.diff);
        let f = l::speed_f(&self.world);
        let h = self.b.height as f64;
        for i in 1..self.lanes.len() {
            let obs = std::mem::take(&mut self.lanes[i].obs);
            let mut alive = Vec::with_capacity(obs.len());
            for mut o in obs {
                let mut ny = o.y + o.v * f * dt;
                if let Some(stop) = o.stop {
                    if ny >= stop {
                        ny = pmax(o.y, stop);
                    }
                }
                o.y = ny;
                if let Some(fd) = o.fade.as_mut() {
                    *fd -= dt;
                }
                let sichtbar = o.fade.is_none_or(|fd| fd > 0.0);
                if sichtbar && o.y - d::obstacle_len(&self.world, o.kind, self.obs_size(o.y)) as f64 / 2.0 <= h {
                    alive.push(o);
                }
            }
            self.lanes[i].obs = alive;
            self.lanes[i].t -= dt;
            if self.lanes[i].t <= 0.0 && !self.lanes[i].closed {
                let kind = self.rng.randrange0(4) as i32;
                self.lanes[i].t = self.rng.uniform(gap_lo, gap_hi);
                let v = self.lanes[i].v;
                self.spawn_obstacle(i, kind, v, None);
            }
        }
    }

    /// Steht auf Spur b gerade ein Hindernis dort, wo das Lama landet?
    fn hit(&self, b: i32) -> bool {
        if !(1 <= b && (b as usize) < self.lanes.len()) {
            return false;
        }
        let ty = self.target_y();
        let reach = self.llama_n as f64 * 0.3;
        for o in &self.lanes[b as usize].obs {
            if o.fade.is_some() {
                continue;
            }
            let olen = d::obstacle_len(&self.world, o.kind, self.obs_size(o.y));
            if l::hit_test(o.y, olen, ty, reach) {
                return true;
            }
        }
        false
    }

    /// Spur b sicher erreicht: keine neuen Hindernisse, die kommenden halten
    /// an (Strasse) bzw. blenden aus.
    fn close_lane(&mut self, b: i32) {
        if !(1 <= b && (b as usize) < self.lanes.len()) {
            return;
        }
        let ty = self.target_y();
        let road = self.world == "road";
        let (path_y, llama_n, gap, k) = (self.path_y, self.llama_n, self.gap, self.k);
        let lane = &mut self.lanes[b as usize];
        lane.closed = true;
        for o in lane.obs.iter_mut() {
            if o.y >= ty {
                continue; // schon vorbei
            }
            if road {
                let olen = d::obstacle_len("road", o.kind, gap);
                let stop = (path_y - llama_n) as f64 - olen as f64 / 2.0 - 6.0 * k;
                // davor anhalten bzw. sofort, wenn schon naeher
                o.stop = Some(pmax(stop, o.y));
            } else if o.fade.is_none() {
                o.fade = Some(FADE_T);
            }
        }
    }

    fn open_lanes(&mut self) {
        for lane in self.lanes.iter_mut().skip(1) {
            lane.closed = false;
            for o in lane.obs.iter_mut() {
                o.stop = None;
            }
        }
    }

    // ===================================================== Ablauf

    /// LOS (vor der Runde) bzw. WEITER (auf einer Spur).
    fn go(&mut self) {
        if self.state == State::Idle {
            if self.broke {
                return;
            }
            if self.bet > lamabank::balance() {
                self.say(t("cas.not_enough"), 1.8);
                self.b.play_sound("hit");
                return;
            }
            if !lamabank::debit(self.bet, KEY) {
                return;
            }
            self.round = Some(l::Round { bet: self.bet, diff: self.diff.clone(), step: 0 });
            self.step = 0;
            self.b.play_sound("select");
            self.do_step();
        } else if self.state == State::Wait {
            self.do_step();
        }
    }

    fn do_step(&mut self) {
        let k = self.step + 1;
        let bet = self.round.as_ref().map(|r| r.bet).unwrap_or(0);
        if l::roll(&mut self.rng, &self.diff) {
            self.outcome = Outcome::Ok;
        } else {
            // Zufalls-Fehltritt: sofort verbucht (Verlassen hilft nicht)
            self.push_hist(0);
            self.round = None;
            self.outcome = Outcome::Fail;
            self.broken = k;
            self.lost = bet;
            self.save_extra();
            self.log_cross(bet, 0, k, None);
            if self.world == "road" {
                // Raser: trifft genau bei der Landung
                let v = l::traffic(&self.diff).1 * l::RASER_F * self.k;
                let y = self.target_y() - v * HOP_T;
                self.spawn_obstacle(k as usize, 0, v, Some(y));
            }
        }
        self.hop = Some(Hop { a: self.step, b: k, t: 0.0 });
        self.state = State::Hop;
        self.b.play_sound("move");
    }

    fn push_hist(&mut self, v: i64) {
        self.hist.push(v);
        if self.hist.len() > l::HISTORY {
            let cut = self.hist.len() - l::HISTORY;
            self.hist.drain(..cut);
        }
    }

    fn land(&mut self) {
        let Some(hp) = self.hop else {
            return;
        };
        let b = hp.b;
        self.step = b;
        let (x, y) = (self.tile_x(b) as f64, self.tile_y(b) as f64);
        let n = self.n();
        let pal = ui::pal();
        if b > n {
            // am Zielufer angekommen
            self.hop = None;
            self.state = State::Goal;
            self.res_t = 0.0;
            self.pending = 0.max(self.pending - self.win);
            let mult = l::mult(&self.diff, n);
            self.result = Some((t("lcr.goal"), tf("lcr.cashed", &[("n", &self.win.to_string())]), pal.gold));
            ui::spawn_confetti(self.b.width as f64, self.b.height as f64, 120);
            self.coin_shower(70);
            let txt = format!("+{}", self.win);
            self.popup(x - self.cam, y - self.llama_n as f64 - 10.0 * self.k, txt, None, 1.4);
            self.b.play_sound("level");
            self.b.rumble(220);
            self.b.ach_event("lcr_other_side", None);
            if self.diff == "extreme" && mult >= l::EXTREME_MULT {
                self.b.ach_event("lcr_extreme", None);
            }
            if mult >= l::BRAVE_MULT {
                self.b.ach_event("lcr_brave", None);
            }
            return;
        }
        if self.outcome == Outcome::Fail {
            let kind = if self.world == "road" { FallKind::Hit } else { FallKind::Break };
            let title = t(&format!("lcr.fall.{}", self.world));
            self.fall(kind, title, x, y);
            return;
        }
        if self.hit(b) {
            // ins Hindernis gelaufen: Einsatz weg
            self.lost = self.round.as_ref().map(|r| r.bet).unwrap_or(0);
            self.push_hist(0);
            self.round = None;
            self.broken = b;
            self.save_extra();
            self.log_cross(self.lost, 0, b, None);
            let title = t(&format!("lcr.hit.{}", self.world));
            self.fall(FallKind::Hit, title, x, y);
            return;
        }
        self.close_lane(b);
        self.dust(x, y);
        self.b.score = lamabank::score_for(KEY);
        if b >= n {
            // letzte Spur geschafft: auszahlen, dann ans Zielufer huepfen
            let bet = self.round.as_ref().map(|r| r.bet).unwrap_or(0);
            self.win = l::payout(bet, &self.diff, n);
            lamabank::credit(self.win, KEY, true);
            self.pending += self.win;
            self.log_cross(bet, self.win, n, Some(l::mult(&self.diff, n)));
            self.push_hist(l::mult(&self.diff, n));
            self.round = None;
            self.save_extra();
            self.b.score = lamabank::score_for(KEY);
            self.hop = Some(Hop { a: b, b: b + 1, t: 0.0 });
            self.b.play_sound("point");
            return;
        }
        if let Some(r) = self.round.as_mut() {
            r.step = b;
        }
        self.save_extra();
        self.hop = None;
        self.state = State::Wait;
        self.b.play_sound("point");
        let txt = self.fmt(l::mult(&self.diff, b));
        self.popup(x - self.cam, y - self.llama_n as f64 - 10.0 * self.k, txt, Some(pal.green), 0.9);
    }

    /// `_log_cross(bet, win, lane, mult100)` - Match-Historie: eine
    /// Ueberquerung (verloren oder ausgezahlt).
    fn log_cross(&mut self, bet: i64, win: i64, lane: i32, mult100: Option<i64>) {
        let mut info = info_map(vec![
            ("bet", bet.into()),
            ("win", win.into()),
            ("ln", lane.into()),
            ("lv", self.diff.clone().into()),
        ]);
        if let Some(m) = mult100 {
            info.insert("x".into(), m.into());
        }
        let net = win - bet;
        self.b.log_match(net, chip_result(net), Some(info), None, 0);
    }

    /// Runde verloren: Zufall (break) oder Treffer (hit, platt).
    fn fall(&mut self, kind: FallKind, title: String, x: f64, y: f64) {
        self.hop = None;
        self.state = State::Fall;
        self.fall_kind = kind;
        self.fall_t = 0.0;
        self.res_t = 0.0;
        self.shake = 0.35;
        self.result = Some((title, tf("lcr.lost", &[("n", &self.lost.to_string())]), ui::pal().red));
        if kind == FallKind::Hit {
            self.spawn_hit(x, y);
        } else {
            self.spawn_fail(x, y);
        }
        self.b.play_sound("hit");
        self.b.rumble(260);
    }

    fn cash_out(&mut self) {
        if self.state != State::Wait || self.step < 1 {
            return;
        }
        let Some(bet) = self.round.as_ref().map(|r| r.bet) else {
            return;
        };
        let mult = l::mult(&self.diff, self.step);
        self.win = l::payout(bet, &self.diff, self.step);
        lamabank::credit(self.win, KEY, true);
        self.log_cross(bet, self.win, self.step, Some(mult));
        self.push_hist(mult);
        self.round = None;
        self.save_extra();
        self.b.score = lamabank::score_for(KEY);
        self.state = State::Cashed;
        self.res_t = 0.0;
        let sub = format!("{}   \u{00B7}   {}: {}", self.fmt(mult), t("cas.bet"), bet);
        self.result = Some((tf("lcr.cashed", &[("n", &self.win.to_string())]), sub, ui::pal().gold));
        let (x, y) = (self.tile_x(self.step) as f64, self.tile_y(self.step) as f64);
        let txt = format!("+{}", self.win);
        self.popup(x - self.cam, y - self.llama_n as f64 - 10.0 * self.k, txt, None, 1.4);
        if mult >= 300 {
            self.coin_shower(if mult < l::BRAVE_MULT { 30 } else { 60 });
        }
        self.b.play_sound("win");
        if mult >= l::BRAVE_MULT {
            self.b.ach_event("lcr_brave", None);
        }
        if self.diff == "extreme" && mult >= l::EXTREME_MULT {
            self.b.ach_event("lcr_extreme", None);
        }
    }

    fn back_to_idle(&mut self) {
        self.state = State::Idle;
        self.step = 0;
        self.hop = None;
        self.result = None;
        self.outcome = Outcome::None;
        self.broken = 0;
        self.ripples.clear();
        self.open_lanes();
        let (x, y) = (self.tile_x(0) as f64, self.tile_y(0) as f64);
        self.dust(x, y);
        self.clamp_bet();
        self.check_broke();
    }

    fn set_bet(&mut self, direction: i32) {
        if self.state != State::Idle {
            return;
        }
        let i = l::BETS.iter().position(|b| *b == self.bet).unwrap_or(0) as i32;
        let j = 0.max((l::BETS.len() as i32 - 1).min(i + direction));
        if j != i {
            self.bet = l::BETS[j as usize];
            self.set_opt("bet", Value::from(self.bet));
            self.b.play_sound("select");
            if self.bet > lamabank::balance() {
                self.say(t("cas.not_enough"), 1.2);
            }
        }
    }

    fn set_diff(&mut self, i: usize) {
        if self.state != State::Idle || l::DIFFS[i] == self.diff {
            return;
        }
        self.diff = l::DIFFS[i].to_string();
        self.set_opt("difficulty", Value::from(self.diff.clone()));
        self.init_traffic();
        self.cam = self.cam_target();
        self.b.play_sound("click");
    }

    fn set_world(&mut self, i: usize) {
        if self.state != State::Idle || l::WORLDS[i] == self.world {
            return;
        }
        self.world = l::WORLDS[i].to_string();
        self.set_opt("world", Value::from(self.world.clone()));
        self.particles.clear();
        self.init_traffic();
        self.b.play_sound("click");
    }

    // ===================================================== Effekte

    #[allow(clippy::too_many_arguments)]
    fn spawn(&mut self, x: f64, y: f64, n: usize, col: (i32, i32, i32), speed: f64, up: bool, r: f64, life: f64, grav: f64) {
        let k = self.k;
        for _ in 0..n {
            let a = if up { self.rng.uniform(PI * 1.05, PI * 1.95) } else { self.rng.uniform(0.0, TAU) };
            let sp = speed * self.rng.uniform(0.35, 1.0) * k;
            let pr = r * k * self.rng.uniform(0.6, 1.2);
            let pl = life * self.rng.uniform(0.7, 1.1);
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
            });
        }
        if self.particles.len() > 220 {
            let cut = self.particles.len() - 220;
            self.particles.drain(..cut);
        }
    }

    fn dust(&mut self, x: f64, y: f64) {
        let col = match self.world.as_str() {
            "road" => (180, 180, 186),
            "bridge" => (196, 160, 120),
            "river" => (200, 230, 246),
            _ => (240, 248, 255),
        };
        self.spawn(x, y, 8, col, 90.0, true, 3.0, 0.45, 260.0);
    }

    fn spawn_fail(&mut self, x: f64, y: f64) {
        let k = self.k;
        if self.world == "bridge" {
            self.spawn(x, y, 16, (150, 98, 58), 220.0, true, 3.2, 1.1, 900.0);
            self.spawn(x, y, 6, (196, 160, 120), 120.0, true, 4.0, 0.6, 300.0);
        } else if self.world == "river" {
            self.spawn(x, y + 10.0 * k, 22, (214, 238, 250), 260.0, true, 3.4, 0.8, 900.0);
            self.ripples.push(Ripple { x, y: y + 12.0 * k, t: 0.0 });
            self.ripples.push(Ripple { x, y: y + 12.0 * k, t: -0.25 });
        } else {
            self.spawn(x, y, 14, (232, 246, 255), 200.0, true, 3.0, 0.9, 800.0);
            self.spawn(x, y + 8.0 * k, 14, (150, 206, 236), 240.0, true, 3.0, 0.8, 900.0);
            self.ripples.push(Ripple { x, y: y + 14.0 * k, t: -0.3 });
        }
    }

    /// Treffer: Staub/Truemmer je Welt, Wolle fliegt.
    fn spawn_hit(&mut self, x: f64, y: f64) {
        let col = match self.world.as_str() {
            "road" => (120, 120, 128),
            "bridge" => (140, 130, 120),
            "river" => (214, 238, 250),
            _ => (232, 246, 255),
        };
        let yy = y - self.llama_n as f64 * 0.3;
        self.spawn(x, yy, 14, col, 240.0, false, 3.4, 0.8, 500.0);
        self.spawn(x, yy, 10, (250, 244, 230), 180.0, false, 3.0, 1.0, 200.0);
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
            let r = self.rng.uniform(7.0, 12.0) * 0.7f64.max(k);
            self.coins_fx.push(CoinFx { x, y, vx, vy, spin, vs, r });
        }
        if self.coins_fx.len() > 160 {
            let cut = self.coins_fx.len() - 160;
            self.coins_fx.drain(..cut);
        }
    }

    fn popup(&mut self, x: f64, y: f64, text: String, color: Option<Color>, dur: f64) {
        let color = color.unwrap_or(ui::pal().gold);
        self.popups.push(Popup { x, y, text, color, t: 0.0, dur });
    }

    fn next_tile_rect(&self) -> Rect {
        let i = self.step + 1;
        let x = self.tile_x(i) as f64 - self.cam;
        let y = self.tile_y(i) as f64;
        Rect::new(
            (x - self.gap as f64 * 0.45) as i32,
            (y - self.llama_n as f64) as i32,
            (self.gap as f64 * 0.9) as i32,
            (self.llama_n as f64 + self.tw as f64 * 0.6) as i32,
        )
    }

    fn update_fx(&mut self, dt: f64) {
        for p in self.particles.iter_mut() {
            p.t += dt;
            p.vy += p.g * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.t < p.life);
        for r in self.ripples.iter_mut() {
            r.t += dt;
        }
        self.ripples.retain(|r| r.t < 1.1);
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

    // ===================================================== Zeichnen: Welt

    fn draw_water(&self, s: &mut Surface, ox: f64) {
        let k = self.k;
        let w = self.b.width;
        if self.world == "river" {
            for i in 0..26 {
                let row = i % 6;
                let y = self.path_y - (self.b.height as f64 * 0.06) as i32 + row * (20.0 * k) as i32 + (10.0 * k) as i32;
                if y > self.b.height {
                    continue;
                }
                let sp = 28 + row * 9;
                let x = py::fpmod((i * 137) as f64 + self.time * sp as f64 * k + ox * 0.6, (w + 60) as f64) - 30.0;
                draw::line(s, (150, 204, 232), (x, y as f64), (x + 22.0 * k, y as f64), 1.max((2.0 * k) as i32));
            }
        } else if self.world == "ice" {
            self.draw_snow(s, 0);
            for i in 0..10 {
                let x = py::fpmod((i * 211) as f64 + ox * 0.9, (w + 80) as f64) - 40.0;
                let y = self.path_y + (40.0 * k) as i32 + (i % 3) * (26.0 * k) as i32;
                draw::line(s, (40, 92, 124), (x, y as f64), (x + 40.0 * k, y as f64), 1.max((2.0 * k) as i32));
            }
        } else if self.world == "bridge" {
            // ein paar Voegel ueber der Schlucht
            for i in 0..3 {
                let x = py::fpmod((i * 300) as f64 + self.time * 25.0 * k, (w + 100) as f64) - 50.0;
                let y = self.path_y as f64 * 0.30 + (i * 22) as f64 * k + (self.time * 2.0 + i as f64).sin() * 4.0 * k;
                let wing = (self.time * 9.0 + (i * 2) as f64).sin() * 3.0 * k;
                draw::lines(
                    s,
                    (70, 50, 60),
                    false,
                    &[(x - 7.0 * k, y - wing), (x, y + 2.0 * k), (x + 7.0 * k, y - wing)],
                    1.max((2.0 * k) as i32),
                );
            }
        }
    }

    fn draw_snow(&self, s: &mut Surface, layer: i32) {
        let k = self.k;
        let (w, h) = (self.b.width, self.b.height);
        let mut i = layer;
        while i < 60 {
            let fi = i as f64;
            let x = py::fpmod(fi * 97.3 + self.time * (10 + i % 5 * 4) as f64 * k + (self.time + fi).sin() * 8.0 * k, w as f64);
            let y = self.hud_h as f64 + py::fpmod(fi * 53.1 + self.time * (24 + i % 7 * 5) as f64 * k, (h - self.hud_h) as f64);
            draw::circle(s, (255, 255, 255), (x as i32, y as i32), 1.max(((1 + layer) as f64 * k) as i32), 0);
            i += 2;
        }
    }

    fn draw_world(&self, s: &mut Surface, ox: f64) {
        let k = self.k;
        let n = self.n();
        let h = self.b.height;
        let xl = self.tile_x(0) as f64 + self.gap as f64 * 0.42 + ox;
        let xr = self.tile_x(n + 1) as f64 - self.gap as f64 * 0.42 + ox;
        let broken_tile = self.state == State::Fall && self.fall_kind == FallKind::Break;
        if self.world == "road" {
            let xs: Vec<f64> = (2..=n)
                .map(|i| self.tile_x(i) as f64 - self.gap as f64 / 2.0 + ox)
                .filter(|x| -10.0 <= *x && *x <= (self.b.width + 10) as f64)
                .collect();
            d::draw_lanes(s, &xs, self.scene.y, h, k, self.time);
        }
        if self.world == "bridge" {
            let mut pts = vec![(xl, self.path_y as f64)];
            for i in 1..=n {
                if broken_tile && i == self.broken {
                    continue;
                }
                pts.push((self.tile_x(i) as f64 + ox, self.tile_y(i) as f64 + 4.0 * k));
            }
            pts.push((xr, self.path_y as f64));
            d::draw_ropes(s, &pts, k);
        }
        d::draw_bank(s, &self.world, xl, -1, self.path_y, h, k, 0);
        d::draw_bank(s, &self.world, xr, 1, self.path_y, h, k, 0);
        self.draw_goal_flag(s, ox);
        let cur = if matches!(self.state, State::Wait | State::Hop | State::Fall | State::Cashed | State::Goal) { self.step } else { 0 };
        for i in 1..=n {
            let cx = self.tile_x(i) as f64 + ox;
            if cx < -(self.gap as f64) || cx > (self.b.width + self.gap) as f64 {
                continue;
            }
            let y = self.tile_y(i) as f64;
            if broken_tile && i == self.broken {
                self.draw_broken_tile(s, cx, y, i);
            } else {
                if self.state == State::Wait && i == self.step + 1 {
                    let glow = Rect::new(0, 0, (self.tw as f64 * 1.3) as i32, (self.tw as f64 * 0.5) as i32)
                        .center_at((cx as i32, (y + self.tw as f64 * 0.12) as i32));
                    let col = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.3 * ui::pulse(3.0, 0.0, 1.0));
                    draw::ellipse(s, col, glow, 1.max((2.0 * k) as i32));
                }
                d::draw_tile(s, &self.world, cx, y, self.tw, k, self.time, i, 255);
            }
            self.draw_label(s, cx, y, i, cur);
        }
    }

    /// Hindernisse aller sichtbaren Spuren, auf der Strasse Baken auf
    /// gesperrten Spuren.
    fn draw_obstacles(&self, s: &mut Surface, ox: f64) {
        let top = self.horizon() as f64;
        let fade_in = 20.0 * self.k;
        let perspektive = self.world == "river" || self.world == "ice";
        for (i, lane) in self.lanes.iter().enumerate().skip(1) {
            let cx = self.tile_x(i as i32) as f64 + ox;
            if cx < -(self.gap as f64) || cx > (self.b.width + self.gap) as f64 {
                continue;
            }
            if self.world == "road" && lane.closed {
                let by = (self.path_y - self.llama_n) as f64 - 2.0 * self.k;
                d::draw_barrier(s, cx, by, (self.gap as f64 * 0.7) as i32, self.k);
            }
            for o in &lane.obs {
                let mut alpha = 255;
                if let Some(fd) = o.fade {
                    alpha = (255.0 * pmax(0.0, fd) / FADE_T) as i32;
                }
                if perspektive && o.y - top < fade_in {
                    alpha = alpha.min((255.0 * pmax(0.0, o.y - top) / fade_in) as i32);
                }
                if alpha > 0 {
                    d::draw_obstacle(s, &self.world, cx, o.y, self.obs_size(o.y), o.kind, alpha);
                }
            }
        }
    }

    /// Das ueberfahrene Lama: platt auf seiner Spur (auf der Strasse mit
    /// Reifenspuren).
    fn draw_flat(&self, s: &mut Surface, ox: f64) {
        let x = self.tile_x(self.step) as f64 + ox;
        let y = self.tile_y(self.step) as f64;
        let img = d::flat_llama(self.llama_n, self.b.accent);
        s.blr(&img, img.get_rect().midbottom_at((x as i32, (y + 2.0 * self.k) as i32)));
        if self.world == "road" {
            d::draw_tire_marks(s, x, y - img.h() as f64 / 2.0, self.gap as f64 * 0.5, self.k);
        }
    }

    fn draw_goal_flag(&self, s: &mut Surface, ox: f64) {
        let k = self.k;
        let n = self.n();
        let x = (self.tile_x(n + 1) + (self.gap as f64 * 0.55) as i32) as f64 + ox;
        if x < -40.0 || x > (self.b.width + 40) as f64 {
            return;
        }
        let y = self.path_y;
        let pole = (70.0 * k) as i32;
        draw::line(s, (90, 70, 60), (x, y as f64), (x, (y - pole) as f64), 2.max((3.0 * k) as i32));
        let wave = (self.time * 4.0).sin() * 3.0 * k;
        let yp = (y - pole) as f64;
        draw::polygon(s, self.b.accent, &[(x, yp), (x + 34.0 * k, yp + 8.0 * k + wave), (x, yp + 18.0 * k)], 0);
        let txt = self.fmt(l::mult(&self.diff, n));
        let img = self.f_small.render(&txt, true, cd::COL_GOLD.c());
        s.blr(&img, img.get_rect().midbottom_at((x as i32, (y as f64 - pole as f64 - 4.0 * k) as i32)));
    }

    fn draw_broken_tile(&self, s: &mut Surface, cx: f64, y: f64, i: i32) {
        let k = self.k;
        let f = self.fall_t;
        if self.world == "river" {
            let alpha = (255.0 * 0f64.max(1.0 - f / 0.8)) as i32;
            d::draw_tile(s, &self.world, cx, y + 26.0 * k * 1f64.min(f), self.tw, k, self.time, i, alpha);
            return;
        }
        if self.world == "ice" && f < 0.3 {
            d::draw_tile(s, &self.world, cx, y, self.tw, k, self.time, i, 255);
            for line in d::crack_lines(cx, y, self.tw, i, f / 0.3) {
                draw::lines(s, (40, 92, 124), false, &line, 1.max((2.0 * k) as i32));
            }
            return;
        }
        let g = f - if self.world == "ice" { 0.3 } else { 0.0 };
        let twf = self.tw as f64;
        let mut layer = pg::surface_alpha((twf * 1.4) as i32 + 8, (twf * 0.9) as i32 + 8);
        let lx = layer.w() as f64 / 2.0;
        d::draw_tile(&mut layer, &self.world, lx, (twf * 0.2) as i32 as f64, self.tw, k, self.time, i, 255);
        let half = layer.w() / 2;
        let drop = if self.world == "bridge" { 0.5 * 1100.0 * k * g * g } else { 30.0 * k * g };
        let alpha = (255.0 * 0f64.max(1.0 - g / 0.9)) as i32;
        for side in 0..2 {
            let mut piece = layer.subsurface(Rect::new(side * half, 0, half, layer.h())).copy();
            piece.set_alpha(Some(alpha as u8));
            let dx = (if side == 0 { -1.0 } else { 1.0 }) * (6.0 + 40.0 * g) * k;
            s.bl(&piece, (cx - lx + (side * half) as f64 + dx, y - (twf * 0.2) as i32 as f64 + drop));
        }
    }

    /// Multiplikator-Schild unter einer Spur.
    fn draw_label(&self, s: &mut Surface, cx: f64, y: f64, i: i32, cur: i32) {
        let txt = self.fmt(l::mult(&self.diff, i));
        let (fg, bg): (Color, Color) = if self.state == State::Fall && i == self.broken {
            (rgb(255, 255, 255), ui::pal().red)
        } else if i <= cur {
            (rgb(230, 255, 236), rgb(40, 120, 72))
        } else if i == cur + 1 {
            (rgb(30, 24, 10), cd::COL_GOLD.c())
        } else {
            (rgb(232, 232, 240), rgb(24, 22, 34))
        };
        let img = self.f_tiny.render(&txt, true, fg);
        let bx = img
            .get_rect()
            .inflate((10.0 * self.k) as i32 + 4, (4.0 * self.k) as i32 + 2)
            .midtop_at((cx as i32, (y + self.tw as f64 * 0.36 + 6.0 * self.k) as i32));
        draw::rect_r(s, bg, bx, 0, bx.h / 2);
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    fn draw_llama(&self, s: &mut Surface, ox: f64) {
        let (x, y, pose) = self.llama_pos();
        let n = self.llama_n;
        let nf = n as f64;
        let mut img = d::llama_sprite(n, pose, self.b.accent);
        let sx = x + ox;
        if self.state != State::Fall && pose != "hop" {
            let shadow = Rect::new(0, 0, (nf * 0.5) as i32, 3.max((nf * 0.08) as i32)).center_at(((sx + nf * 0.02) as i32, (y + 1.0) as i32));
            draw::ellipse(s, (0, 0, 0), shadow, 0);
        }
        if self.state == State::Fall {
            let f = self.fall_t;
            let fade = 1.0 - 1f64.min(0f64.max((f - if self.world == "bridge" { 0.5 } else { 0.15 }) / 0.5));
            if fade <= 0.0 {
                return;
            }
            img = img.copy();
            img.set_alpha(Some((255.0 * fade) as i32 as u8));
        }
        s.bl(&img, ((sx - nf / 2.0) as i32, (y - nf + nf * 0.06) as i32));
    }

    fn draw_particles(&self, s: &mut Surface, ox: f64) {
        for p in &self.particles {
            let r = p.r * (1.0 - p.t / p.life);
            if r >= 0.5 {
                draw::circle(s, p.col, ((p.x + ox) as i32, p.y as i32), 1.max(r as i32), 0);
            }
        }
        for rp in &self.ripples {
            if rp.t <= 0.0 {
                continue;
            }
            let w = ((20.0 + 140.0 * rp.t) * self.k) as i32;
            let rect = Rect::new(0, 0, w, 2.max(w / 4)).center_at(((rp.x + ox) as i32, rp.y as i32));
            draw::ellipse(s, (210, 236, 250), rect, 1.max((2.0 * self.k) as i32));
        }
    }

    // ===================================================== Zeichnen: Oberflaeche

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
        let left = self.f_big.render(&format!("{}: {}", t("cas.chips"), py::round(self.shown_chips)), true, p.gold);
        s.blr(&left, left.get_rect().midleft_at((pad + coin_r * 2 + 6, cy)));
        let right_txt = format!("{}: {}   \u{00B7}   {}", t("cas.record"), self.b.score, tf("cas.credits", &[("n", &lamabank::refills().to_string())]));
        let mut right = self.f_small.render(&right_txt, true, p.text_dim);
        if right.w() as f64 > w as f64 * 0.42 {
            right = self.f_tiny.render(&right_txt, true, p.text_dim);
        }
        let rr = right.get_rect().midright_at((w - pad, cy));
        s.blr(&right, rr);
        let bet = self.round_bet();
        let mut mid_txt = String::new();
        if self.round.is_some() {
            mid_txt = format!("{}: {}", t("cas.bet"), bet);
            if self.step > 0 {
                mid_txt += &format!(
                    "   \u{00B7}   {} = {}",
                    self.fmt(l::mult(&self.diff, self.step)),
                    l::payout(bet, &self.diff, self.step)
                );
            }
        }
        if mid_txt.is_empty() {
            return;
        }
        let x0 = pad + coin_r * 2 + 6 + left.w() + pad;
        let x1 = rr.x - pad;
        let mut mid = self.f_small.render(&mid_txt, true, p.text);
        if mid.w() > x1 - x0 {
            mid = self.f_tiny.render(&mid_txt, true, p.text);
        }
        if mid.w() <= x1 - x0 {
            s.blr(&mid, mid.get_rect().center_at(((x0 + x1) / 2, cy)));
        }
    }

    /// Letzte Runden als Schilder oben rechts (neueste rechts).
    fn draw_history(&self, s: &mut Surface) {
        let mut x = self.b.width - self.margin;
        let y = self.hud_h + self.margin;
        let last = self.r_worlds[self.r_worlds.len() - 1];
        let left_limit = last.right() + self.margin + self.f_small.width(&t(&format!("lcr.world.{}", self.world))) + self.margin;
        for &m100 in self.hist.iter().rev() {
            let txt = if m100 > 0 { self.fmt(m100) } else { "\u{00D7}0".to_string() };
            let img = self.f_tiny.render(&txt, true, rgb(240, 240, 245));
            let bx = img.get_rect().inflate((10.0 * self.k) as i32 + 4, (6.0 * self.k) as i32 + 2).topright_at((x, y));
            if bx.x < left_limit {
                break;
            }
            let col = if m100 >= 200 {
                (40, 120, 72)
            } else if m100 > 0 {
                (70, 96, 70)
            } else {
                (128, 36, 46)
            };
            draw::rect_r(s, col, bx, 0, bx.h / 2);
            s.blr(&img, img.get_rect().center_at(bx.center()));
            x = bx.x - 4;
        }
    }

    fn draw_worlds(&self, s: &mut Surface) {
        let p = ui::pal();
        let on = self.state == State::Idle;
        for (i, r) in self.r_worlds.iter().enumerate() {
            let w = l::WORLDS[i];
            let active = w == self.world;
            let hot = on && r.hit(self.mouse);
            let fill = if active || hot { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, *r, 0, 4.max(r.h / 4));
            let edge = if active { cd::COL_GOLD.c() } else if on { p.border_light } else { p.border };
            draw::rect_r(s, edge, *r, if active { 2 } else { 1 }, 4.max(r.h / 4));
            let icon_col = if on || active { p.text } else { p.text_faint };
            d::world_icon(s, w, r.inflate(py::fdiv32(-r.w, 5), py::fdiv32(-r.h, 5)), icon_col);
        }
        let name = self.f_small.render(&t(&format!("lcr.world.{}", self.world)), true, p.text);
        let last = self.r_worlds[self.r_worlds.len() - 1];
        s.blr(&name, name.get_rect().midleft_at((last.right() + self.margin, last.centery())));
    }

    fn draw_setup(&self, s: &mut Surface) {
        for (i, r) in self.r_diffs.iter().enumerate() {
            let label = format!("{}  {}", i + 1, t(&format!("lcr.diff.{}", l::DIFFS[i])));
            self.btn(s, *r, &label, false, true, r.hit(self.mouse), l::DIFFS[i] == self.diff);
        }
        let n = self.n();
        let info = tf(
            "lcr.info",
            &[
                ("n", &n.to_string()),
                ("p", &l::risk_percent(&self.diff).to_string()),
                ("m", &l::fmt_mult(l::mult(&self.diff, n), &t("lcr.decimal"))),
            ],
        );
        self.draw_hint(s, &info, Some(self.r_diffs[0].y - 4));
    }

    fn draw_hint(&self, s: &mut Surface, text: &str, bottom: Option<i32>) {
        let img = self.f_tiny.render(text, true, ui::pal().text);
        let bx = img
            .get_rect()
            .inflate(18, 8)
            .midbottom_at((self.b.width / 2, bottom.unwrap_or(self.strip.y - self.margin)));
        let mut panel = pg::surface_alpha(bx.w, bx.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(12, 8, 20, 190), pr, 0, bx.h / 2);
        s.blr(&panel, bx);
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    fn draw_strip(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.strip_bg, (self.strip.x, self.strip.y));
        draw::line(s, ui::mix(cd::COL_GOLD, self.b.accent, 0.35), (self.strip.x, self.strip.y), (self.strip.right(), self.strip.y), 2);
        let idle = self.state == State::Idle;
        let bet = self.round_bet();
        self.btn(s, self.r_minus, "\u{2013}", false, idle && self.bet > l::BETS[0], self.r_minus.hit(self.mouse), false);
        self.btn(s, self.r_plus, "+", false, idle && self.bet < l::BETS[l::BETS.len() - 1], self.r_plus.hit(self.mouse), false);
        draw::rect_r(s, (16, 12, 26), self.r_bet, 0, 6.max(self.r_bet.h / 4));
        draw::rect_r(s, p.border, self.r_bet, 1, 6.max(self.r_bet.h / 4));
        let lbl = self.f_tiny.render(&t("cas.bet"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midtop_at((self.r_bet.centerx(), self.r_bet.y + 2)));
        let col = if idle && bet > lamabank::balance() { p.red } else { p.gold };
        let val = self.f_btn.render(&bet.to_string(), true, col);
        s.blr(&val, val.get_rect().midbottom_at((self.r_bet.centerx(), self.r_bet.bottom() - 3)));
        let wait = self.state == State::Wait;
        let go_lbl = if idle { t("lcr.go") } else { t("lcr.step") };
        self.btn(s, self.r_go, &go_lbl, true, (idle && !self.broke) || wait, self.r_go.hit(self.mouse), false);
        let mut cash = t("lcr.cashout");
        if wait {
            if let Some(r) = self.round.as_ref() {
                cash += &format!(" {}", l::payout(r.bet, &self.diff, self.step));
            }
        }
        self.btn(s, self.r_cash, &cash, false, wait, self.r_cash.hit(self.mouse), wait);
    }

    #[allow(clippy::too_many_arguments)]
    fn btn(&self, s: &mut Surface, r: Rect, label: &str, primary: bool, on: bool, hot: bool, active: bool) {
        let p = ui::pal();
        let rad = 6.max(r.h / 4);
        let col = if primary && on {
            let base = ui::mix(self.b.accent, (0, 0, 0), 0.18);
            let fill = ui::mix(base, (255, 255, 255), if hot { 0.12 } else { 0.0 });
            draw::rect_r(s, ui::mix(base, (0, 0, 0), 0.4), r.move_(0, 2), 0, rad);
            draw::rect_r(s, fill, r, 0, rad);
            let border = ui::mix(cd::COL_GOLD, (255, 255, 255), 0.35 * ui::pulse(2.4, 0.0, 1.0));
            draw::rect_r(s, border, r, 2, rad);
            rgb(255, 255, 255)
        } else {
            let fill = if on && (hot || active) { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, r, 0, rad);
            let edge = if active { cd::COL_GOLD.c() } else if on { p.border_light } else { p.border };
            draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
            if on { p.text } else { p.text_faint }
        };
        let mut img = self.f_btn.render(label, true, col);
        if img.w() > r.w - 8 {
            img = self.f_tiny.render(label, true, col);
        }
        s.blr(&img, img.get_rect().center_at(r.center()));
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
        for p in &self.popups {
            let fr = p.t / p.dur;
            let mut img = self.f_big.render(&p.text, true, p.color);
            if fr > 0.6 {
                img = img.copy();
                img.set_alpha(Some((255.0 * (1.0 - (fr - 0.6) / 0.4)) as i32 as u8));
            }
            let y = p.y - 40.0 * self.k * (1.0 - py::pow(1.0 - fr, 2.0));
            s.blr(&img, img.get_rect().center_at((p.x as i32, y as i32)));
        }
    }

    fn draw_result(&self, s: &mut Surface) {
        let Some((title, sub, col)) = self.result.as_ref() else {
            return;
        };
        let p = ui::pal();
        let w = self.b.width;
        let k = self.k;
        let mut timg = self.f_huge.render(title, true, *col);
        if timg.w() > w - 40 {
            timg = self.f_big.render(title, true, *col);
        }
        let mut simg = self.f_small.render(sub, true, p.text);
        if simg.w() > w - 40 {
            simg = self.f_tiny.render(sub, true, p.text);
        }
        let pw = (w - 20).min(timg.w().max(simg.w()) + (48.0 * k) as i32);
        let ph = timg.h() + simg.h() + (34.0 * k) as i32;
        let panel = Rect::new(0, 0, pw, ph).center_at((w / 2, self.scene.y + (self.scene.h as f64 * 0.24) as i32 + ph / 2 - (10.0 * k) as i32));
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(*col));
        s.blr(&timg, timg.get_rect().midtop_at((panel.centerx(), panel.y + (12.0 * k) as i32)));
        s.blr(&simg, simg.get_rect().midbottom_at((panel.centerx(), panel.bottom() - (12.0 * k) as i32)));
    }

    fn draw_toast(&self, s: &mut Surface) {
        let Some((text, left)) = self.toast.as_ref() else {
            return;
        };
        let mut img = self.f_small.render(text, true, ui::pal().text);
        let bx = img
            .get_rect()
            .inflate(24, 12)
            .midtop_at((self.b.width / 2, self.hud_h + self.margin + self.r_worlds[0].h + self.margin));
        let mut panel = pg::surface_alpha(bx.w, bx.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(12, 8, 20, 225), pr, 0, bx.h / 2);
        draw::rect_r(&mut panel, pg::alpha(ui::mix(cd::COL_GOLD, self.b.accent, 0.4), 255), pr, 1, bx.h / 2);
        if *left < 0.3 {
            let a = (255.0 * left / 0.3) as i32 as u8;
            panel.set_alpha(Some(a));
            img = img.copy();
            img.set_alpha(Some(a));
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
}

impl Game for LamaCrossingGame {
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
        self.rng = py::Random::unseeded();
        let world = self.b.opt_value(KEY, "world");
        self.world = match world.as_ref().and_then(|v| v.as_str()) {
            Some(w) if l::WORLDS.contains(&w) => w.to_string(),
            _ => "road".into(),
        };
        let diff = self.b.opt_value(KEY, "difficulty");
        self.diff = match diff.as_ref().and_then(|v| v.as_str()) {
            Some(d) if l::DIFFS.contains(&d) => d.to_string(),
            _ => "medium".into(),
        };
        let bet = self.b.opt_value(KEY, "bet");
        self.bet = match bet.as_ref().and_then(|v| v.as_i64()) {
            Some(b) if l::BETS.contains(&b) && !matches!(bet, Some(Value::Bool(_))) => b,
            _ => l::BETS[0],
        };
        let extra = lamabank::get_extra(KEY);
        let extra = match extra {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        self.hist = l::clean_history(extra.get("hist"));
        self.round = l::clean_round(extra.get("round"));
        self.step = 0;
        self.state = State::Idle;
        self.hop = None;
        self.outcome = Outcome::None;
        self.fall_kind = FallKind::Break;
        self.broken = 0;
        self.fall_t = 0.0;
        self.res_t = 0.0;
        self.result = None;
        self.win = 0;
        self.lost = 0;
        self.pending = 0;
        self.shown_chips = lamabank::balance() as f64;
        self.particles.clear();
        self.ripples.clear();
        self.coins_fx.clear();
        self.popups.clear();
        self.toast = None;
        self.shake = 0.0;
        self.time = 0.0;
        self.mouse = (-1, -1);
        self.broke = false;
        self.tick_cd = 0.0;
        self.lang = i18n::get_language();
        self.layout();
        if let Some(r) = self.round.clone() {
            // Unterbrochene Ueberquerung genau dort fortsetzen
            self.diff = r.diff;
            self.step = r.step;
            self.state = State::Wait;
            self.say(t("lcr.resumed"), 1.8);
        } else {
            self.clamp_bet();
        }
        self.init_traffic();
        self.cam = self.cam_target();
        self.b.score = lamabank::score_for(KEY);
        self.check_broke();
    }

    fn on_surface_changed(&mut self) {
        d::clear_cache();
        self.layout();
        self.init_traffic();
        self.cam = self.cam_target();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let kind = ev.kind;
        if matches!(kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            self.mouse = ev.pos;
        }
        let confirm = kind == Kind::KeyDown && ["Return", "space", "KP_Enter"].contains(&ev.key.as_str());
        if self.broke {
            if kind == Kind::MouseDown || confirm {
                self.take_credit();
            }
            return;
        }
        if matches!(self.state, State::Fall | State::Cashed | State::Goal) {
            // Ergebnis wegklicken (fruehestens nach kurzer Zeit)
            if self.res_t > 0.6 && (kind == Kind::MouseDown || confirm) {
                self.back_to_idle();
            }
            return;
        }
        if kind == Kind::KeyDown {
            match ev.key.as_str() {
                "space" | "Right" => self.go(),
                "Return" | "KP_Enter" => {
                    if self.state == State::Wait {
                        self.cash_out();
                    } else {
                        self.go();
                    }
                }
                "c" | "C" => self.cash_out(),
                "plus" | "KP_Add" | "Up" | "equal" => self.set_bet(1),
                "minus" | "KP_Subtract" | "Down" => self.set_bet(-1),
                "1" | "2" | "3" | "4" => {
                    let i = ev.key.parse::<usize>().unwrap_or(1) - 1;
                    self.set_diff(i);
                }
                "w" | "W" => {
                    let i = l::WORLDS.iter().position(|w| *w == self.world).unwrap_or(0);
                    self.set_world((i + 1) % l::WORLDS.len());
                }
                _ => {}
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
        if self.r_minus.hit(pos) {
            self.set_bet(-1);
        } else if self.r_plus.hit(pos) {
            self.set_bet(1);
        } else if self.r_go.hit(pos) {
            self.go();
        } else if self.r_cash.hit(pos) {
            self.cash_out();
        } else if self.state == State::Idle {
            for i in 0..self.r_diffs.len() {
                if self.r_diffs[i].hit(pos) {
                    self.set_diff(i);
                    return;
                }
            }
            for i in 0..self.r_worlds.len() {
                if self.r_worlds[i].hit(pos) {
                    self.set_world(i);
                    return;
                }
            }
        } else if self.state == State::Wait && self.next_tile_rect().hit(pos) {
            self.go();
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
        self.shake = 0f64.max(self.shake - dt);
        self.update_fx(dt);
        self.update_traffic(dt);
        let tgt = self.cam_target();
        self.cam += (tgt - self.cam) * 1f64.min(dt * 6.0);
        if (tgt - self.cam).abs() < 0.5 {
            self.cam = tgt;
        }
        if self.state == State::Hop {
            let done = match self.hop.as_mut() {
                Some(hp) => {
                    hp.t += dt;
                    hp.t >= HOP_T
                }
                None => false,
            };
            if done {
                self.land();
            }
        } else if matches!(self.state, State::Fall | State::Cashed | State::Goal) {
            self.fall_t += dt;
            self.res_t += dt;
            if self.res_t >= RESULT_T + if self.state == State::Fall { FALL_T } else { 0.0 } {
                self.back_to_idle();
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        let bg = d::background(&self.world, w, h, self.path_y);
        s.bl(&bg, (0, 0));
        if self.world != "road" {
            let far = d::far_layer(&self.world, w, h, self.path_y);
            let off = py::pmod32(-((self.cam * 0.18) as i32), w);
            s.bl(&far, (off - w, 0));
            s.bl(&far, (off, 0));
        }
        let mut ox = -self.cam;
        if self.shake > 0.0 {
            ox += (self.time * 70.0).sin() * 6.0 * self.k * self.shake / 0.35;
        }
        self.draw_water(&mut s, ox);
        self.draw_world(&mut s, ox);
        let flat = self.state == State::Fall && self.fall_kind == FallKind::Hit;
        if flat {
            self.draw_flat(&mut s, ox);
        }
        self.draw_obstacles(&mut s, ox);
        if !flat {
            self.draw_llama(&mut s, ox);
        }
        self.draw_particles(&mut s, ox);
        if self.world == "ice" {
            self.draw_snow(&mut s, 1);
        }
        self.draw_hud(&mut s);
        self.draw_history(&mut s);
        self.draw_worlds(&mut s);
        if self.state == State::Idle {
            self.draw_setup(&mut s);
        } else if self.state == State::Wait {
            self.draw_hint(&mut s, &t("lcr.hint_wait"), None);
        }
        self.draw_strip(&mut s);
        self.draw_fx(&mut s);
        if self.result.is_some() && (self.state != State::Fall || self.fall_t > FALL_T * 0.55) {
            self.draw_result(&mut s);
        }
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
    key: "lamacrossing",
    class_name: "LamaCrossingGame",
    supports_multiplayer: false,
    modes: &[],
    make: LamaCrossingGame::new,
};
