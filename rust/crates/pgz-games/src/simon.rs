//! simon.rs
//! ========
//! Simon / Senso - das klassische Merkspiel mit leuchtenden Feldern.
//!
//! Uebertragen aus `games/simon.py`: fuenf Modi (Klassisch, Speed, Reverse,
//! Duell, Gemischt), drei Ton-Arten, 4/6/9 Felder, Bestwerte je Modus im
//! Abschnitt `simon` des Profils.

use pgz_engine::prelude::*;
use pgz_core::store;
use serde_json::{Map, Value};

const PAD_COLORS: [Color; 9] = [
    Color::rgb(46, 204, 113),
    Color::rgb(231, 76, 60),
    Color::rgb(241, 196, 15),
    Color::rgb(52, 152, 219),
    Color::rgb(155, 89, 182),
    Color::rgb(230, 126, 34),
    Color::rgb(26, 188, 156),
    Color::rgb(233, 96, 160),
    Color::rgb(149, 200, 60),
];
const FREQS: [f64; 9] = [261.63, 293.66, 329.63, 392.00, 440.00, 523.25, 587.33, 659.25, 783.99];

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

const MODES: [&str; 5] = ["classic", "speed", "reverse", "duel", "mixed"];
const AUDIO_MODES: [&str; 3] = ["off", "on", "mixed"];
const PAD_OPTS: [i64; 3] = [4, 6, 9];
const SUBS: [&str; 3] = ["classic", "speed", "reverse"];

fn grid(pads: i64) -> (i32, i32) {
    match pads {
        6 => (3, 2),
        9 => (3, 3),
        _ => (2, 2),
    }
}

fn dim(c: Color, f: f64) -> Color {
    Color::rgb(
        (c.r as f64 * f) as i32 as u8,
        (c.g as f64 * f) as i32 as u8,
        (c.b as f64 * f) as i32 as u8,
    )
}

fn lit(c: Color) -> Color {
    Color::rgb(
        255.min(c.r as i32 + 90) as u8,
        255.min(c.g as i32 + 90) as u8,
        255.min(c.b as i32 + 90) as u8,
    )
}

/// `SimonGame`
pub struct SimonGame {
    b: GameBase,
    smode: &'static str,
    audio_mode: &'static str,
    pads: i64,
    small: Font,
    tiny: Font,
    huge: Font,
    best: Map<String, Value>,
    state: State,
    seq: Vec<usize>,
    phase: &'static str,
    lit_pad: Option<usize>,
    lit_t: f64,
    msg_t: f64,
    winner: usize,
    active: &'static str,
    lit_dur: f64,
    gap_dur: f64,
    round_sound: bool,
    expected: Vec<usize>,
    show_i: usize,
    show_state: &'static str,
    show_t: f64,
    input_i: usize,
    turn_player: usize,
    adding: bool,
    hud_h: i32,
    pad_rects: Vec<Rect>,
    glow: Surface,
    mode_rects: Vec<Rect>,
    audio_rects: Vec<Rect>,
    pad_rects_setup: Vec<Rect>,
    start_rect: Rect,
}

impl SimonGame {
    /// `SimonGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = SimonGame {
            b,
            smode: "classic",
            audio_mode: "on",
            pads: 4,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            best: Map::new(),
            state: State::Setup,
            seq: Vec::new(),
            phase: "input",
            lit_pad: None,
            lit_t: 0.0,
            msg_t: 0.0,
            winner: 0,
            active: "classic",
            lit_dur: 0.42,
            gap_dur: 0.18,
            round_sound: true,
            expected: Vec::new(),
            show_i: 0,
            show_state: "pre",
            show_t: 0.0,
            input_i: 0,
            turn_player: 0,
            adding: true,
            hud_h: 46,
            pad_rects: Vec::new(),
            glow: pg::surface_alpha(1, 1),
            mode_rects: Vec::new(),
            audio_rects: Vec::new(),
            pad_rects_setup: Vec::new(),
            start_rect: z,
        };
        g.reset();
        Box::new(g)
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 32));
        self.tiny = ui::f(11.max(h / 40));
        self.huge = ui::fb(28.max(h / 10));
    }

    fn load_best() -> Map<String, Value> {
        let data = store::load_section("simon");
        match data.get("best") {
            Some(Value::Object(m)) => m.clone(),
            _ => Map::new(),
        }
    }

    fn save_best(&self) {
        let mut sec = Map::new();
        sec.insert("best".into(), Value::Object(self.best.clone()));
        store::save_section("simon", sec);
    }

    fn best_of(&self, mode: &str) -> i64 {
        self.best.get(mode).and_then(|v| v.as_f64()).map(|v| v as i64).unwrap_or(0)
    }

    fn layout(&mut self) {
        self.hud_h = 46;
        let (cols, rows) = grid(self.pads);
        let (w, h) = (self.b.width, self.b.height);
        let area = (w - 60).min(h - self.hud_h - 50);
        let pad_gap = 8.max(py::fdiv32(area, 40));
        let mx = cols.max(rows);
        let cell = (area - pad_gap * (mx - 1)) as f64 / mx as f64;
        let gw = cols as f64 * cell + (cols - 1) as f64 * pad_gap as f64;
        let gh = rows as f64 * cell + (rows - 1) as f64 * pad_gap as f64;
        let grid_x = py::ffdiv(w as f64 - gw, 2.0);
        let grid_y = self.hud_h as f64 + 12.0f64.max(py::ffdiv(h as f64 - self.hud_h as f64 - gh, 2.0));
        self.pad_rects = (0..self.pads as i32)
            .map(|i| {
                let (r, c) = (i / cols, i % cols);
                let x = grid_x + c as f64 * (cell + pad_gap as f64);
                let y = grid_y + r as f64 * (cell + pad_gap as f64);
                Rect::new(x as i32, y as i32, cell as i32, cell as i32)
            })
            .collect();
        self.glow = pg::surface_alpha(cell as i32, cell as i32);
        self.glow.fill_c(Color::rgba(255, 255, 255, 60));
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 380.min(self.b.width - 50);
        let y0 = (self.b.height as f64 * 0.26) as i32;
        let gap = 8;
        let row = |y: i32, n: i32| -> Vec<Rect> {
            let cw = (bw - gap * (n - 1)) as f64 / n as f64;
            (0..n)
                .map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap as f64)) as i32, y, cw as i32, 40))
                .collect()
        };
        self.mode_rects = row(y0, 5);
        self.audio_rects = row(y0 + 82, 3);
        self.pad_rects_setup = row(y0 + 150, 3);
        self.start_rect = Rect::new(cx - 95, y0 + 210, 190, 46);
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("simon", key, value);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" | "4" | "5" => {
                    self.smode = MODES[k.parse::<usize>().unwrap_or(1) - 1];
                    self.b.mode = self.smode.to_string(); // Python: self.mode = MODES[...]
                    self.save_setting("mode", self.smode.into());
                    self.b.play_sound("click");
                }
                "a" | "A" => {
                    let i = (AUDIO_MODES.iter().position(|m| *m == self.audio_mode).unwrap_or(0) + 1) % 3;
                    self.audio_mode = AUDIO_MODES[i];
                    self.save_setting("audio", self.audio_mode.into());
                    self.b.play_sound("select");
                }
                "p" | "P" => {
                    let i = (PAD_OPTS.iter().position(|p| *p == self.pads).unwrap_or(0) + 1) % 3;
                    self.pads = PAD_OPTS[i];
                    self.save_setting("pads", self.pads.into());
                    self.layout();
                    self.b.play_sound("select");
                }
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.mode_rects.iter().position(|r| r.hit(ev.pos)) {
                self.smode = MODES[i];
                self.b.mode = self.smode.to_string();
                self.save_setting("mode", self.smode.into());
                self.b.play_sound("click");
                return;
            }
            if let Some(i) = self.audio_rects.iter().position(|r| r.hit(ev.pos)) {
                self.audio_mode = AUDIO_MODES[i];
                self.save_setting("audio", self.audio_mode.into());
                self.b.play_sound("select");
                return;
            }
            if let Some(i) = self.pad_rects_setup.iter().position(|r| r.hit(ev.pos)) {
                self.pads = PAD_OPTS[i];
                self.save_setting("pads", self.pads.into());
                self.layout();
                self.b.play_sound("select");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn start_play(&mut self) {
        self.layout();
        self.state = State::Play;
        self.b.game_over = false;
        self.b.new_round_result();
        self.b.score = 0;
        self.lit_pad = None;
        self.lit_t = 0.0;
        self.msg_t = 0.0;
        if self.smode == "duel" {
            self.begin_duel();
        } else {
            self.seq = vec![py::randrange0(self.pads) as usize];
            self.begin_round();
        }
        self.b.play_sound("click");
    }

    fn begin_round(&mut self) {
        self.active = if self.smode == "mixed" { py::choice(&SUBS) } else { self.smode };
        let rnd = self.seq.len() as f64;
        if self.active == "speed" {
            self.lit_dur = 0.14f64.max(0.44 - 0.02 * rnd);
            self.gap_dur = 0.06f64.max(0.18 - 0.008 * rnd);
        } else {
            self.lit_dur = 0.42;
            self.gap_dur = 0.18;
        }
        self.round_sound = self.audio_mode != "mixed" || py::random() < 0.5;
        self.expected = if self.active == "reverse" {
            self.seq.iter().rev().copied().collect()
        } else {
            self.seq.clone()
        };
        self.phase = "show";
        self.show_i = 0;
        self.show_state = "pre";
        self.show_t = 0.55;
        self.input_i = 0;
        self.lit_pad = None;
    }

    fn update_show(&mut self, dt: f64) {
        self.show_t -= dt;
        if self.show_t > 0.0 {
            return;
        }
        if self.show_state == "pre" {
            self.light_show(0);
        } else if self.show_state == "on" {
            self.lit_pad = None;
            self.show_state = "off";
            self.show_t = self.gap_dur;
            self.show_i += 1;
        } else if self.show_i >= self.seq.len() {
            self.phase = "input";
            self.input_i = 0;
            self.lit_pad = None;
        } else {
            self.light_show(self.show_i);
        }
    }

    fn light_show(&mut self, i: usize) {
        self.lit_pad = Some(self.seq[i]);
        self.show_state = "on";
        self.show_t = self.lit_dur;
        self.tone(self.seq[i]);
    }

    fn begin_duel(&mut self) {
        self.seq.clear();
        self.turn_player = 0;
        self.input_i = 0;
        self.adding = true;
        self.phase = "input";
        self.round_sound = true;
    }

    fn tone(&self, i: usize) {
        if self.audio_mode == "off" {
            return;
        }
        if self.audio_mode == "mixed" && !self.round_sound {
            return;
        }
        audio::tone(FREQS[i % FREQS.len()], 0.22, Some(&self.b.settings), "sine", 0.35);
    }

    fn single_press(&mut self, pad: usize) {
        self.lit_pad = Some(pad);
        self.lit_t = 0.2;
        self.tone(pad);
        if pad == self.expected[self.input_i] {
            self.input_i += 1;
            if self.input_i >= self.expected.len() {
                self.b.score = self.seq.len() as i64;
                self.b.ach_event("simon_20", Some(self.b.score));
                self.b.play_sound("point");
                self.seq.push(py::randrange0(self.pads) as usize);
                self.begin_round();
            }
        } else {
            self.fail_single();
        }
    }

    fn fail_single(&mut self) {
        self.b.play_sound("gameover");
        let key = self.smode;
        if self.best_of(key) < self.b.score {
            self.best.insert(key.into(), self.b.score.into());
            self.save_best();
        }
        self.state = State::Over;
        self.b.game_over = true;
    }

    fn duel_press(&mut self, pad: usize) {
        self.lit_pad = Some(pad);
        self.lit_t = 0.2;
        self.tone(pad);
        if self.adding {
            self.seq.push(pad);
            self.adding = false;
            self.input_i = 0;
            self.turn_player = 1 - self.turn_player;
            self.b.play_sound("select");
            return;
        }
        if pad == self.seq[self.input_i] {
            self.input_i += 1;
            if self.input_i >= self.seq.len() {
                self.adding = true;
                self.b.play_sound("point");
            }
        } else {
            self.winner = 1 - self.turn_player;
            self.b.report_duel(self.winner);
            self.b.play_sound("gameover");
            self.state = State::Over;
            self.b.game_over = true;
        }
    }

    fn draw_pads(&self, s: &mut Surface) {
        for (i, rc) in self.pad_rects.iter().enumerate() {
            let col = PAD_COLORS[i];
            let on = self.lit_pad == Some(i);
            let fill = if on { lit(col) } else { dim(col, 0.5) };
            draw::rect_r(s, fill, *rc, 0, 14);
            draw::rect_r(s, dim(col, 0.8), *rc, 3, 14);
            if on {
                s.bl(&self.glow, (rc.x, rc.y));
            }
            let num = self.small.render(&(i + 1).to_string(), true, if on { Color::rgb(255, 255, 255) } else { dim(col, 0.85) });
            s.blr(&num, num.get_rect().center_at(rc.center()));
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let mut mid;
        if self.smode == "duel" {
            let n0: String = self.b.pname(0).chars().take(8).collect();
            let left = self.small.render(&n0, true, if self.turn_player == 0 { self.b.accent } else { p.text_dim });
            s.blr(&left, left.get_rect().midleft_at((14, cy)));
            let n1: String = self.b.pname(1).chars().take(8).collect();
            let right = self.small.render(&n1, true, if self.turn_player == 1 { self.b.accent } else { p.text_dim });
            s.blr(&right, right.get_rect().midright_at((w - 14, cy)));
            mid = if self.adding { t("simon.add") } else { t("simon.repeat") };
            mid += &format!("  ({})", self.seq.len());
        } else {
            let sc = self.small.render(&tf("simon.round", &[("n", &self.seq.len().to_string())]), true, p.text);
            s.blr(&sc, sc.get_rect().midleft_at((14, cy)));
            let best = self.best_of(self.smode);
            let bimg = self.small.render(&tf("simon.best", &[("n", &best.to_string())]), true, p.text_dim);
            s.blr(&bimg, bimg.get_rect().midright_at((w - 14, cy)));
            mid = if self.phase == "show" { t("simon.watch") } else { t("simon.your_input") };
            if self.smode == "mixed" {
                mid += "  \u{b7}  ";
                mid += &t(&format!("simon.mode.{}", self.active));
            }
        }
        let img = self.small.render(&mid, true, self.b.accent);
        s.blr(&img, img.get_rect().center_at((w / 2, cy)));
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let mut ov = pg::surface_alpha(w, 108);
        ov.fill_c(Color::rgba(10, 8, 16, 210));
        let y = self.b.height / 2 - 54;
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 1);
        draw::line(s, self.b.accent, (0, y + 107), (w, y + 107), 1);
        let cx = w / 2;
        let (head, sub) = if self.smode == "duel" {
            let name: String = self.b.pname(self.winner).chars().take(10).collect();
            (
                self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, self.b.accent),
                self.small.render(&tf("simon.duel_len", &[("n", &self.seq.len().to_string())]), true, p.text),
            )
        } else {
            (
                self.huge.render(&tf("simon.score", &[("n", &self.b.score.to_string())]), true, self.b.accent),
                self.small.render(&tf("simon.best", &[("n", &self.best_of(self.smode).to_string())]), true, p.text),
            )
        };
        s.blr(&head, head.get_rect().center_at((cx, y + 34)));
        s.blr(&sub, sub.get_rect().center_at((cx, y + 68)));
        let hint = self.tiny.render(&t("simon.new_round"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + 92)));
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool, fnt: &Font) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 8);
        let im = fnt.render(text, true, if on { p.text } else { p.text_dim });
        s.blr(&im, im.get_rect().center_at(rc.center()));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let title = self.huge.render(&t("simon.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.12) as i32)));
        let sub = self.small.render(&t("simon.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.185) as i32)));
        let label = |s: &mut Surface, y: i32, txt: &str| {
            let im = self.tiny.render(txt, true, p.text_dim);
            s.blr(&im, im.get_rect().midbottom_at((cx, y - 4)));
        };
        label(s, self.mode_rects[0].y, &t("simon.lbl_mode"));
        for (i, rc) in self.mode_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("simon.mode.{}", MODES[i])), self.smode == MODES[i], &self.tiny);
        }
        label(s, self.audio_rects[0].y, &t("simon.lbl_audio"));
        for (i, rc) in self.audio_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("simon.audio.{}", AUDIO_MODES[i])), self.audio_mode == AUDIO_MODES[i], &self.small);
        }
        label(s, self.pad_rects_setup[0].y, &t("simon.lbl_pads"));
        for (i, rc) in self.pad_rects_setup.iter().enumerate() {
            self.btn(s, *rc, &PAD_OPTS[i].to_string(), self.pads == PAD_OPTS[i], &self.small);
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("simon.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 14)));
    }
}

impl Game for SimonGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, State::Setup)
    }
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let m = self.b.opt_str("simon", "mode", "classic");
        self.smode = MODES.iter().copied().find(|x| *x == m).unwrap_or("classic");
        // Python ueberschreibt self.mode mit dem Simon-Modus (Match-Historie)
        self.b.mode = self.smode.to_string();
        let a = self.b.opt_str("simon", "audio", "on");
        self.audio_mode = AUDIO_MODES.iter().copied().find(|x| *x == a).unwrap_or("on");
        let p = self.b.opt_i64("simon", "pads", 4);
        self.pads = if PAD_OPTS.contains(&p) { p } else { 4 };
        self.build_fonts();
        self.best = Self::load_best();
        self.build_setup_layout();
        self.layout();
        self.state = State::Setup;
        self.seq.clear();
        self.phase = "input";
        self.lit_pad = None;
        self.lit_t = 0.0;
        self.msg_t = 0.0;
        self.winner = 0;
        self.active = if self.smode != "mixed" { self.smode } else { "classic" };
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.build_setup_layout();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup(ev);
                return;
            }
            State::Over => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "space"]) {
                        self.start_play();
                    } else if ev.key_in(&["s", "S"]) {
                        self.state = State::Setup;
                        self.b.game_over = false;
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown {
                    self.start_play();
                }
                return;
            }
            State::Play => {}
        }
        if self.smode != "duel" && self.phase != "input" {
            return;
        }
        let mut pad = None;
        if ev.kind == Kind::MouseDown {
            pad = self.pad_rects.iter().position(|r| r.hit(ev.pos));
        } else if ev.kind == Kind::KeyDown && "123456789".contains(ev.key.as_str()) {
            // Python: `event.key in "123456789"` und dann int(...)
            if let Ok(n) = ev.key.parse::<i64>() {
                let idx = n - 1;
                if idx < self.pads && idx >= 0 {
                    pad = Some(idx as usize);
                }
            }
        }
        let Some(pad) = pad else {
            return;
        };
        if self.smode == "duel" {
            self.duel_press(pad);
        } else {
            self.single_press(pad);
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
        }
        if self.lit_t > 0.0 {
            self.lit_t -= dt;
            if self.lit_t <= 0.0 && (self.smode == "duel" || self.phase == "input") {
                self.lit_pad = None;
            }
        }
        if self.state == State::Play && self.smode != "duel" && self.phase == "show" {
            self.update_show(dt);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        self.draw_hud(&mut s);
        self.draw_pads(&mut s);
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "simon",
    class_name: "SimonGame",
    supports_multiplayer: true,
    modes: &[],
    make: SimonGame::new,
};
