//! mastermind.rs
//! =============
//! Mastermind - knacke den geheimen Farbcode.
//!
//! Uebertragen aus `games/mastermind.py`: drei Modi, Standard-Rueckmeldung
//! (schwarz/weiss mit Haeufigkeiten), Endlos-Serie, Palette mit Loeschen/OK.

use pgz_engine::prelude::*;

const COL_SLOT: Color = Color::rgb(44, 50, 70);
const COL_SLOT_BORDER: Color = Color::rgb(70, 78, 104);
const COL_PEG_BLACK: Color = Color::rgb(26, 28, 36);
const COL_PEG_WHITE: Color = Color::rgb(236, 238, 245);
const PALETTE: [Color; 8] = [
    Color::rgb(228, 72, 72),
    Color::rgb(240, 150, 52),
    Color::rgb(240, 214, 74),
    Color::rgb(96, 200, 112),
    Color::rgb(66, 202, 210),
    Color::rgb(82, 132, 240),
    Color::rgb(172, 112, 236),
    Color::rgb(240, 120, 190),
];

fn mode_cfg(mode: &str) -> (usize, usize, usize) {
    match mode {
        "easy" => (4, 6, 12),
        "hard" => (5, 8, 10),
        _ => (4, 6, 10),
    }
}

fn diff_bonus(mode: &str) -> i64 {
    match mode {
        "easy" => 0,
        "hard" => 25,
        _ => 10,
    }
}

/// `mastermind.feedback(guess, secret)` - `(schwarz, weiss)`.
pub fn feedback(guess: &[usize], secret: &[usize]) -> (usize, usize) {
    let black = guess.iter().zip(secret).filter(|(g, s)| g == s).count();
    let mut total = 0;
    let mut gesehen: Vec<usize> = Vec::new();
    for c in guess {
        if gesehen.contains(c) {
            continue;
        }
        gesehen.push(*c);
        let sc = secret.iter().filter(|x| *x == c).count();
        let gc = guess.iter().filter(|x| *x == c).count();
        total += sc.min(gc);
    }
    (black, total - black)
}

#[derive(PartialEq, Clone, Copy)]
enum Zustand {
    Play,
    Solved,
    Over,
}

/// `MastermindGame`
pub struct MastermindGame {
    b: GameBase,
    pins: usize,
    colors: usize,
    max_rows: usize,
    solved_count: i64,
    secret: Vec<usize>,
    rows_data: Vec<(Vec<usize>, (usize, usize))>,
    current: Vec<Option<usize>>,
    row: usize,
    state: Zustand,
    last_points: i64,
    pal_h: i32,
    pal_top: i32,
    del_rect: Rect,
    check_rect: Rect,
    swatch_rects: Vec<Rect>,
    peg: i32,
    gap: i32,
    pitch: f64,
    grid_top: i32,
    fb_w: i32,
    grid_x: i32,
    fb_x: i32,
    hud: Font,
    small: Font,
    tiny: Font,
    huge: Font,
}

impl MastermindGame {
    /// `MastermindGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = MastermindGame {
            b,
            pins: 4,
            colors: 6,
            max_rows: 10,
            solved_count: 0,
            secret: Vec::new(),
            rows_data: Vec::new(),
            current: Vec::new(),
            row: 0,
            state: Zustand::Play,
            last_points: 0,
            pal_h: 0,
            pal_top: 0,
            del_rect: z,
            check_rect: z,
            swatch_rects: Vec::new(),
            peg: 0,
            gap: 0,
            pitch: 0.0,
            grid_top: 0,
            fb_w: 0,
            grid_x: 0,
            fb_x: 0,
            hud: f.clone(),
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        self.hud = ui::fb(20);
        self.small = ui::f(15);
        self.tiny = ui::fb(13);
        self.huge = ui::fb(28.max(self.b.height / 12));
    }

    fn new_code(&mut self) {
        self.secret = (0..self.pins).map(|_| py::randrange0(self.colors as i64) as usize).collect();
        self.rows_data.clear();
        self.current = vec![None; self.pins];
        self.row = 0;
        self.state = Zustand::Play;
        self.last_points = 0;
    }

    fn layout(&mut self) {
        let pins = self.pins as f64;
        let gap = 6;
        self.pal_h = 46.max(self.b.height / 10);
        self.pal_top = self.b.height - self.pal_h - 10;
        self.build_palette();
        let top = 64;
        let bottom = self.pal_top - 12;
        let pitch = (bottom - top) as f64 / self.max_rows as f64;
        let peg = (pitch - gap as f64).min((self.b.width - 150) as f64 / pins - gap as f64) as i32;
        self.peg = 16.max(peg.min(46));
        self.gap = gap;
        self.pitch = pitch;
        self.grid_top = top;
        let grid_w = self.pins as i32 * (self.peg + gap) - gap;
        self.fb_w = 26.max(self.peg);
        let total_w = grid_w + 22 + self.fb_w;
        self.grid_x = py::fdiv32(self.b.width - total_w, 2);
        self.fb_x = self.grid_x + grid_w + 22;
    }

    fn build_palette(&mut self) {
        let gap = 8;
        let c = self.colors as i32;
        let pw = ((self.b.width - 40) as f64 / (c + 3) as f64).min(52.0) as i32;
        let pw = 28.max(pw);
        let special = (pw as f64 * 1.4) as i32;
        let row_w = c * pw + (c - 1) * gap + 2 * (special + gap);
        let mut x = py::fdiv32(self.b.width - row_w, 2);
        let y = self.pal_top + py::fdiv32(self.pal_h - pw, 2);
        let h = (self.pal_h - 8).min(pw);
        self.del_rect = Rect::new(x, y, special, h);
        x += special + gap;
        self.swatch_rects.clear();
        for _ in 0..c {
            self.swatch_rects.push(Rect::new(x, y, pw, h));
            x += pw + gap;
        }
        self.check_rect = Rect::new(x, y, special, h);
    }

    fn row_peg_rect(&self, row: usize, i: usize) -> Rect {
        let x = self.grid_x + i as i32 * (self.peg + self.gap);
        let y = (self.grid_top as f64 + row as f64 * self.pitch) as i32;
        Rect::new(x, y, self.peg, self.peg)
    }

    fn click(&mut self, pos: (i32, i32)) {
        if self.check_rect.hit(pos) {
            self.submit();
            return;
        }
        if self.del_rect.hit(pos) {
            self.delete();
            return;
        }
        for i in 0..self.pins {
            if self.current[i].is_some() && self.row_peg_rect(self.row, i).hit(pos) {
                self.current[i] = None;
                self.b.play_sound("move");
                return;
            }
        }
        if let Some(ci) = self.swatch_rects.iter().position(|rc| rc.hit(pos)) {
            self.place(ci);
        }
    }

    fn place(&mut self, color_idx: usize) {
        if let Some(slot) = self.current.iter_mut().find(|c| c.is_none()) {
            *slot = Some(color_idx);
        }
        self.b.play_sound("click");
    }

    fn delete(&mut self) {
        for i in (0..self.pins).rev() {
            if self.current[i].is_some() {
                self.current[i] = None;
                self.b.play_sound("move");
                return;
            }
        }
    }

    fn submit(&mut self) {
        if self.current.iter().any(Option::is_none) {
            self.b.play_sound("click");
            return;
        }
        let guess: Vec<usize> = self.current.iter().map(|c| c.unwrap_or(0)).collect();
        let (black, white) = feedback(&guess, &self.secret);
        self.rows_data.push((guess, (black, white)));
        self.current = vec![None; self.pins];
        self.row += 1;
        if black == self.pins {
            let used = self.row as i64;
            let pts = 20 + (self.max_rows as i64 - used) * 8 + diff_bonus(&self.b.mode);
            self.last_points = pts;
            self.b.score += pts;
            self.solved_count += 1;
            self.state = Zustand::Solved;
            if used <= 3 {
                self.b.ach_event("master_three", None);
            }
            self.b.report_result(true);
            self.b.play_sound("win");
        } else if self.row >= self.max_rows {
            self.state = Zustand::Over;
            self.b.game_over = true;
            if self.solved_count == 0 {
                self.b.report_result(false);
            }
            self.b.play_sound("gameover");
        } else {
            self.b.play_sound("move");
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let img = self.hud.render(&t("mm.title"), true, self.b.accent);
        s.blr(&img, img.get_rect().midleft_at((20, 30)));
        let img = self.small.render(&tf("mm.score", &[("n", &self.b.score.to_string())]), true, p.gold);
        s.blr(&img, img.get_rect().center_at((w / 2, 22)));
        let a = (self.row + 1).min(self.max_rows).to_string();
        let img = self.small.render(&tf("mm.rows", &[("a", &a), ("b", &self.max_rows.to_string())]), true, p.text_dim);
        s.blr(&img, img.get_rect().midright_at((w - 20, 22)));
        let img = self.small.render(&tf("mm.solved_n", &[("n", &self.solved_count.to_string())]), true, p.text_dim);
        s.blr(&img, img.get_rect().midright_at((w - 20, 44)));
    }

    fn draw_rows(&self, s: &mut Surface) {
        for r in 0..self.max_rows {
            let active = r == self.row && self.state == Zustand::Play;
            for i in 0..self.pins {
                let rc = self.row_peg_rect(r, i);
                let val = if r < self.rows_data.len() {
                    Some(self.rows_data[r].0[i])
                } else if active {
                    self.current[i]
                } else {
                    None
                };
                self.draw_peg(s, rc, val, active && val.is_none());
            }
            if r < self.rows_data.len() {
                self.draw_feedback(s, r, self.rows_data[r].1);
            }
        }
    }

    fn draw_peg(&self, s: &mut Surface, rc: Rect, val: Option<usize>, highlight: bool) {
        let (cx, cy) = rc.center();
        let rad = rc.w / 2;
        match val {
            None => {
                draw::circle(s, COL_SLOT, (cx, cy), rad, 0);
                draw::circle(s, if highlight { self.b.accent } else { COL_SLOT_BORDER }, (cx, cy), rad, 2);
            }
            Some(v) => {
                let col = PALETTE[v];
                draw::circle(s, col, (cx, cy), rad, 0);
                draw::circle(s, mix(col, Color::rgb(255, 255, 255), 0.35), (cx, cy), rad, 2);
                draw::circle(s, mix(col, Color::rgb(255, 255, 255), 0.5), (cx - rad / 3, cy - rad / 3), 2.max(rad / 5), 0);
            }
        }
    }

    fn draw_feedback(&self, s: &mut Surface, r: usize, fb: (usize, usize)) {
        let (black, white) = fb;
        let mut pegs = vec![COL_PEG_BLACK; black];
        pegs.extend(vec![COL_PEG_WHITE; white]);
        let rest = self.pins.saturating_sub(pegs.len());
        pegs.extend(vec![Color::rgb(58, 64, 84); rest]);
        let cols = if self.pins <= 4 { 2 } else { 3 };
        let sr = 3.max(self.peg / 6);
        let step = sr * 2 + 3;
        let y0 = (self.grid_top as f64 + r as f64 * self.pitch) as i32 + self.peg / 2 - step;
        for (k, pc) in pegs.iter().enumerate() {
            let (gx, gy) = ((k % cols) as i32, (k / cols) as i32);
            let x = self.fb_x + gx * step + sr;
            let y = y0 + gy * step;
            draw::circle(s, *pc, (x, y), sr, 0);
            if *pc == COL_PEG_WHITE {
                draw::circle(s, Color::rgb(120, 122, 130), (x, y), sr, 1);
            }
        }
    }

    fn draw_palette(&self, s: &mut Surface) {
        for (ci, rc) in self.swatch_rects.iter().enumerate() {
            let col = PALETTE[ci];
            draw::rect_r(s, col, *rc, 0, 8);
            draw::rect_r(s, mix(col, Color::rgb(255, 255, 255), 0.3), *rc, 2, 8);
            let img = self.tiny.render(&(ci + 1).to_string(), true, Color::rgb(20, 22, 30));
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.bottom() - 9)));
        }
        for (rc, label) in [(self.del_rect, "<"), (self.check_rect, "OK")] {
            draw::rect_r(s, Color::rgb(60, 66, 90), rc, 0, 8);
            draw::rect_r(s, ui::border_light(), rc, 2, 8);
            let img = self.tiny.render(label, true, ui::text());
            s.blr(&img, img.get_rect().center_at(rc.center()));
        }
    }

    fn draw_banner(&self, s: &mut Surface, title: &str, color: Color, sub: &str, reveal: bool) {
        let w = (self.b.width - 40).min(480);
        let h = if reveal { 150 } else { 108 };
        let rc = Rect::new((self.b.width - w) / 2, (self.b.height - h) / 2, w, h);
        let mut panel = pg::surface_alpha(w, h);
        panel.fill_c(Color::rgba(16, 18, 24, 238));
        s.bl(&panel, (rc.x, rc.y));
        draw::rect_r(s, color, rc, 2, 14);
        let img = self.huge.render(title, true, color);
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 36)));
        if reveal {
            let img = self.small.render(&t("mm.code_was"), true, ui::text_dim());
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 70)));
            let rad = 12;
            let tot = self.pins as i32 * (rad * 2 + 8) - 8;
            let mut x = rc.centerx() - tot / 2 + rad;
            for val in &self.secret {
                draw::circle(s, PALETTE[*val], (x, rc.y + 98), rad, 0);
                draw::circle(s, mix(PALETTE[*val], Color::rgb(255, 255, 255), 0.35), (x, rc.y + 98), rad, 2);
                x += rad * 2 + 8;
            }
            let img = self.small.render(sub, true, ui::text_dim());
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 128)));
        } else {
            let img = self.small.render(sub, true, ui::text_dim());
            s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 78)));
        }
    }
}

impl Game for MastermindGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        let (p, c, r) = mode_cfg(&self.b.mode);
        self.pins = p;
        self.colors = c;
        self.max_rows = r;
        self.b.score = 0;
        self.b.game_over = false;
        self.solved_count = 0;
        self.make_fonts();
        self.new_code();
        self.layout();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if matches!(self.state, Zustand::Solved | Zustand::Over) {
            if ev.kind == Kind::MouseDown || ev.is_any(&["Return", "space"]) {
                if self.state == Zustand::Over {
                    self.b.game_over = false;
                    self.reset();
                } else {
                    self.new_code();
                    self.b.play_sound("click");
                }
            }
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if k == "BackSpace" {
                self.delete();
            } else if k == "Return" {
                self.submit();
            } else if !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(n) = k.parse::<usize>() {
                    if 1 <= n && n <= self.colors {
                        self.place(n - 1);
                    }
                }
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            self.click(ev.pos);
        }
    }

    fn update(&mut self, _dt: f64) {}

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        self.draw_hud(&mut s);
        self.draw_rows(&mut s);
        self.draw_palette(&mut s);
        if self.state == Zustand::Solved {
            self.draw_banner(&mut s, &tf("mm.solved", &[("n", &self.last_points.to_string())]), self.b.accent, &t("mm.next"), false);
        } else if self.state == Zustand::Over {
            self.draw_banner(&mut s, &t("mm.gameover"), Color::rgb(228, 96, 96), &t("common.enter_restart"), true);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "mastermind",
    class_name: "MastermindGame",
    supports_multiplayer: false,
    modes: &[("easy", "mm.mode.easy"), ("classic", "mm.mode.classic"), ("hard", "mm.mode.hard")],
    make: MastermindGame::new,
};
