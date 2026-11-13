//! casino.rs
//! =========
//! Casino - europaeisches Roulette und der Lama-Slot, gespielt mit den
//! Lama-Chips der gemeinsamen Lama-Bank (geteilt mit Blackjack und Poker).
//! Uebertragen aus `games/casino.py`; der Roulette-Teil steht in
//! [`crate::casino_roulette`], der Slot in [`crate::casino_slots`].
//!
//! Geld-Fluss: Einsatz wird beim Drehen abgebucht, das vorher gezogene
//! Ergebnis sofort gutgeschrieben - angezeigt wird es erst, wenn Kugel bzw.
//! Walzen stehen (`pending`). Pleite = Bank-Kredit auf 1000.

use pgz_core::{i18n, lamabank};
use pgz_engine::prelude::*;
use serde_json::Value;

use crate::casino_draw as d;
use crate::casino_roulette::Roul;
use crate::casino_slots::Slot;

const MODES: [&str; 2] = ["roulette", "slots"];

pub(crate) struct CoinFx {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    spin: f64,
    vs: f64,
    r: f64,
}

pub(crate) struct Popup {
    x: f64,
    y: f64,
    text: String,
    color: Color,
    t: f64,
    dur: f64,
}

/// `CasinoGame`
pub struct CasinoGame {
    pub(crate) b: GameBase,
    pub(crate) rng: py::Random,
    pub(crate) pending: i64,
    pub(crate) shown_chips: f64,
    pub(crate) mouse: (i32, i32),
    pub(crate) coins_fx: Vec<CoinFx>,
    pub(crate) popups: Vec<Popup>,
    pub(crate) toast: Option<(String, f64)>,
    pub(crate) broke: bool,
    tick_cd: f64,
    pub(crate) lang: String,
    pub(crate) k: f64,
    pub(crate) f_tiny: Font,
    pub(crate) f_small: Font,
    pub(crate) f_btn: Font,
    pub(crate) f_big: Font,
    pub(crate) f_huge: Font,
    pub(crate) margin: i32,
    pub(crate) hud_h: i32,
    pub(crate) strip_h: i32,
    pub(crate) strip: Rect,
    hud_bg: Surface,
    strip_bg: Surface,
    pub(crate) overlay: Surface,
    pub(crate) r: Roul,
    pub(crate) s: Slot,
}

impl CasinoGame {
    /// `CasinoGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = CasinoGame {
            b,
            rng: py::Random::new(0),
            pending: 0,
            shown_chips: 0.0,
            mouse: (-1, -1),
            coins_fx: Vec::new(),
            popups: Vec::new(),
            toast: None,
            broke: false,
            tick_cd: 0.0,
            lang: String::new(),
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
            hud_bg: pg::surface_alpha(1, 1),
            strip_bg: pg::surface_alpha(1, 1),
            overlay: pg::surface_alpha(1, 1),
            r: Roul::default(),
            s: Slot::default(),
        };
        g.reset();
        Box::new(g)
    }

    pub(crate) fn roulette(&self) -> bool {
        self.b.mode == "roulette"
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.k = (w as f64 / 800.0).min(h as f64 / 600.0);
        let k = self.k;
        self.f_tiny = ui::font(11.max((13.0 * k) as i32), false, false);
        self.f_small = ui::font(12.max((15.0 * k) as i32), false, false);
        self.f_btn = ui::font(12.max((15.0 * k) as i32), true, false);
        self.f_big = ui::font(15.max((22.0 * k) as i32), true, false);
        self.f_huge = ui::font(26.max((58.0 * k) as i32), true, false);
        self.margin = 6.max((12.0 * k) as i32);
        self.hud_h = 28.max((42.0 * k) as i32);
        self.strip_h = 46.max((64.0 * k) as i32);
        self.strip = Rect::new(0, h - self.strip_h, w, self.strip_h);
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
        if self.roulette() {
            self.r_layout();
        } else {
            self.s_layout();
        }
    }

    // ===================================================== Einstellungen

    pub(crate) fn opt(&self, key: &str) -> Option<Value> {
        self.b.opt_value("casino", key)
    }

    pub(crate) fn set_opt(&mut self, key: &str, value: Value) {
        if self.opt(key).as_ref() != Some(&value) {
            self.b.set_opt("casino", key, value);
        }
    }

    // ===================================================== Konto

    fn busy(&self) -> bool {
        if self.roulette() {
            self.r_busy()
        } else {
            self.s_busy()
        }
    }

    /// Pleite = unter dem Mindesteinsatz des Modus (und nichts laeuft).
    pub(crate) fn check_broke(&mut self) {
        if self.busy() {
            self.broke = false;
            return;
        }
        let free = !self.roulette() && self.s.free > 0;
        self.broke = !free && lamabank::is_broke("casino", Some(&self.b.mode));
        if self.broke {
            self.b.play_sound("gameover");
        }
    }

    fn take_credit(&mut self) {
        if lamabank::refill_if_broke("casino", Some(&self.b.mode)) {
            self.b.play_sound("powerup");
            ui::spawn_burst((self.b.width / 2) as f64, (self.b.height / 2) as f64, Some(ui::pal().gold), 26, 260.0);
        }
        self.broke = false;
        self.shown_chips = lamabank::balance() as f64;
    }

    fn target_chips(&self) -> i64 {
        0.max(lamabank::balance() - self.pending - self.reserved())
    }

    pub(crate) fn say(&mut self, text: String) {
        self.toast = Some((text, 1.8));
    }

    // ===================================================== Effekte

    fn update_fx(&mut self, dt: f64) {
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

    /// Goldmuenzen-Regen (grosse Gewinne).
    pub(crate) fn coin_shower(&mut self, n: usize) {
        let w = self.b.width as f64;
        let h = self.b.height as f64;
        let k = self.k;
        for _ in 0..n {
            let x = self.rng.uniform(w * 0.1, w * 0.9);
            let y = self.rng.uniform(-h * 0.4, -10.0);
            let vx = self.rng.uniform(-60.0, 60.0) * k;
            let vy = self.rng.uniform(-80.0, 120.0) * k;
            let spin = self.rng.uniform(0.0, std::f64::consts::TAU);
            let vs = self.rng.uniform(6.0, 14.0);
            let r = self.rng.uniform(7.0, 12.0) * 0.7f64.max(k);
            self.coins_fx.push(CoinFx { x, y, vx, vy, spin, vs, r });
        }
        if self.coins_fx.len() > 160 {
            let cut = self.coins_fx.len() - 160;
            self.coins_fx.drain(..cut);
        }
    }

    pub(crate) fn popup(&mut self, x: f64, y: f64, text: String) {
        self.popups.push(Popup { x, y, text, color: ui::pal().gold, t: 0.0, dur: 1.4 });
    }

    // ===================================================== Zeichnen

    fn draw_hud(&mut self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        s.bl(&self.hud_bg, (0, 0));
        draw::line(s, ui::mix(d::COL_GOLD, self.b.accent, 0.35), (0, self.hud_h - 1), (w, self.hud_h - 1), 1);
        let cy = self.hud_h / 2;
        let pad = self.margin;
        let coin_r = 6.max((self.hud_h as f64 * 0.26) as i32);
        let coin = d::symbol_surface("coin", coin_r * 2 + 4, false);
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
        let mid_txt = if self.roulette() { self.r_hud_text() } else { self.s_hud_text() };
        if !mid_txt.is_empty() {
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
    }

    fn draw_fx(&self, s: &mut Surface) {
        for c in &self.coins_fx {
            let w = 2.max((c.r * 2.0 * c.spin.cos().abs()) as i32);
            let h = (c.r * 2.0) as i32;
            let rect = Rect::new(0, 0, w, h).center_at((c.x as i32, c.y as i32));
            draw::ellipse(s, d::COL_GOLD_D, rect.move_(1, 1), 0);
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

    fn draw_toast(&self, s: &mut Surface) {
        let Some((text, left)) = self.toast.as_ref() else {
            return;
        };
        let mut img = self.f_small.render(text, true, ui::pal().text);
        let mut bx = img.get_rect().inflate(24, 12);
        bx = bx.midbottom_at((self.b.width / 2, self.strip.y - self.margin));
        let mut panel = pg::surface_alpha(bx.w, bx.h);
        let pr = panel.get_rect();
        draw::rect_r(&mut panel, rgba(12, 8, 20, 225), pr, 0, bx.h / 2);
        draw::rect_r(&mut panel, pg::alpha(ui::mix(d::COL_GOLD, self.b.accent, 0.4), 255), pr, 1, bx.h / 2);
        if *left < 0.3 {
            let a = (255.0 * left / 0.3) as i32 as u8;
            panel.set_alpha(Some(a));
            img = img.copy();
            img.set_alpha(Some(a));
        }
        s.blr(&panel, bx);
        s.blr(&img, img.get_rect().center_at(bx.center()));
    }

    pub(crate) fn draw_strip_bg(&self, s: &mut Surface) {
        s.bl(&self.strip_bg, (self.strip.x, self.strip.y));
        draw::line(s, ui::mix(d::COL_GOLD, self.b.accent, 0.35), (self.strip.x, self.strip.y), (self.strip.right(), self.strip.y), 2);
    }

    /// Button im Theme-Look; primary = Akzentflaeche mit sanftem Puls.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn btn(&self, s: &mut Surface, r: Rect, label: &str, primary: bool, on: bool, hot: bool, active: bool) {
        let p = ui::pal();
        let rad = 6.max(r.h / 4);
        let col;
        if primary && on {
            let base = ui::mix(self.b.accent, (0, 0, 0), 0.18);
            let fill = ui::mix(base, (255, 255, 255), if hot { 0.12 } else { 0.0 });
            draw::rect_r(s, ui::mix(base, (0, 0, 0), 0.4), r.move_(0, 2), 0, rad);
            draw::rect_r(s, fill, r, 0, rad);
            let border = ui::mix(d::COL_GOLD, (255, 255, 255), 0.35 * ui::pulse(2.4, 0.0, 1.0));
            draw::rect_r(s, border, r, 2, rad);
            col = rgb(255, 255, 255);
        } else {
            let fill = if on && (hot || active) { p.btn_sel } else { p.btn };
            draw::rect_r(s, fill, r, 0, rad);
            let edge = if active { d::COL_GOLD.c() } else if on { p.border_light } else { p.border };
            draw::rect_r(s, edge, r, if active { 2 } else { 1 }, rad);
            col = if on { p.text } else { p.text_faint };
        }
        let mut img = self.f_btn.render(label, true, col);
        if img.w() > r.w - 8 {
            img = self.f_tiny.render(label, true, col);
        }
        s.blr(&img, img.get_rect().center_at(r.center()));
    }

    pub(crate) fn hot(&self, r: Rect) -> bool {
        r.hit(self.mouse)
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

impl Game for CasinoGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn reset(&mut self) {
        self.b.game_over = false;
        if !MODES.contains(&self.b.mode.as_str()) {
            self.b.mode = "roulette".into();
        }
        lamabank::load();
        self.rng = py::Random::unseeded();
        self.pending = 0;
        self.shown_chips = lamabank::balance() as f64;
        self.mouse = (-1, -1);
        self.coins_fx.clear();
        self.popups.clear();
        self.toast = None;
        self.broke = false;
        self.tick_cd = 0.0;
        self.lang = i18n::get_language();
        self.layout();
        if self.roulette() {
            self.r_init();
        } else {
            self.s_init();
        }
        self.b.score = lamabank::score_for("casino");
        self.check_broke();
    }

    fn on_surface_changed(&mut self) {
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if matches!(ev.kind, Kind::MouseMove | Kind::MouseDown | Kind::MouseUp) {
            self.mouse = ev.pos;
        }
        if self.broke {
            if ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && ["Return", "space", "KP_Enter"].contains(&ev.key.as_str())) {
                self.take_credit();
            }
            return;
        }
        if self.roulette() {
            self.r_event(ev);
        } else {
            self.s_event(ev);
        }
    }

    fn update(&mut self, dt: f64) {
        if self.lang != i18n::get_language() {
            self.lang = i18n::get_language();
            d::clear_cache();
            self.layout();
        }
        let target = self.target_chips() as f64;
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
        self.update_fx(dt);
        if self.roulette() {
            self.r_update(dt);
        } else {
            self.s_update(dt);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.roulette() {
            self.r_draw(&mut s);
        } else {
            self.s_draw(&mut s);
        }
        self.draw_hud(&mut s);
        self.draw_fx(&mut s);
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
    key: "casino",
    class_name: "CasinoGame",
    supports_multiplayer: false,
    modes: &[("roulette", "cas.mode.roulette"), ("slots", "cas.mode.slots")],
    make: CasinoGame::new,
};
