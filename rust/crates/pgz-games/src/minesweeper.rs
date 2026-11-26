//! minesweeper.rs
//! ==============
//! Minesweeper - der Klassiker, mausgesteuert.
//!
//! Uebertragen aus `games/minesweeper.py`: drei Stufen (gespeichert), sicherer
//! erster Klick, Flaggen mit optionalem Fragezeichen, Chording, Smiley-HUD
//! mit LED-Anzeigen, Bestzeit je Stufe, Konfetti beim Sieg.

use pgz_engine::prelude::*;
use serde_json::{Map, Value};

const COL_TILE: Color = Color::rgb(52, 60, 82);
const COL_TILE_HOVER: Color = Color::rgb(64, 74, 100);
const COL_BEVEL_L: Color = Color::rgb(88, 99, 128);
const COL_BEVEL_D: Color = Color::rgb(30, 34, 48);
const COL_OPEN: Color = Color::rgb(33, 38, 54);
const COL_OPEN_LINE: Color = Color::rgb(24, 28, 40);
const COL_BOOM: Color = Color::rgb(168, 66, 66);
const COL_LED: Color = Color::rgb(240, 90, 90);
const COL_FLAG: Color = Color::rgb(235, 90, 90);

const NUMBER_COLORS: [Color; 8] = [
    Color::rgb(110, 160, 255),
    Color::rgb(110, 220, 130),
    Color::rgb(250, 110, 110),
    Color::rgb(180, 140, 255),
    Color::rgb(230, 160, 90),
    Color::rgb(110, 220, 220),
    Color::rgb(235, 235, 235),
    Color::rgb(170, 170, 185),
];

const PRESETS: [(&str, i32, i32, usize, i64); 3] =
    [("beginner", 9, 9, 10, 150), ("advanced", 16, 16, 40, 500), ("expert", 30, 16, 99, 1200)];

const HUD_H: i32 = 52;
const SURPRISE_T: f64 = 0.22;

type Cell = (i32, i32);

/// `MinesweeperGame`
pub struct MinesweeperGame {
    b: GameBase,
    preset_index: usize,
    qmarks: bool,
    best: Map<String, Value>,
    small: Font,
    tiny: Font,
    led: Font,
    num_font: Font,
    anim_t: f64,
    setup: bool,
    cols: i32,
    rows: i32,
    n_mines: usize,
    base_points: i64,
    mines: Vec<bool>,
    numbers: Vec<i32>,
    revealed: Vec<bool>,
    n_revealed: usize,
    flags: Vec<u8>,
    flag_used: bool,
    first_click: bool,
    exploded: Option<Cell>,
    won: bool,
    elapsed: f64,
    running: bool,
    hover: Option<Cell>,
    surprise: f64,
    particles: Vec<[f64; 5]>,
    particle_cols: Vec<Color>,
    cell: i32,
    bx: i32,
    by: i32,
    face_rect: Rect,
    preset_rects: Vec<Rect>,
    qmark_rect: Rect,
    start_rect: Rect,
}

impl MinesweeperGame {
    /// `MinesweeperGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = MinesweeperGame {
            b,
            preset_index: 0,
            qmarks: false,
            best: Map::new(),
            small: f.clone(),
            tiny: f.clone(),
            led: f.clone(),
            num_font: f,
            anim_t: 0.0,
            setup: true,
            cols: 9,
            rows: 9,
            n_mines: 10,
            base_points: 150,
            mines: Vec::new(),
            numbers: Vec::new(),
            revealed: Vec::new(),
            n_revealed: 0,
            flags: Vec::new(),
            flag_used: false,
            first_click: true,
            exploded: None,
            won: false,
            elapsed: 0.0,
            running: false,
            hover: None,
            surprise: 0.0,
            particles: Vec::new(),
            particle_cols: Vec::new(),
            cell: 10,
            bx: 0,
            by: 0,
            face_rect: z,
            preset_rects: Vec::new(),
            qmark_rect: z,
            start_rect: z,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.b.font = ui::f(16.max(h / 26));
        self.b.big_font = ui::fb(30.max(h / 11));
        self.small = ui::f(13.max(h / 32));
        self.tiny = ui::f(11.max(h / 40));
        self.led = ui::font(18.max(26.min(h / 24)), true, true);
    }

    fn idx(&self, c: Cell) -> usize {
        (c.1 * self.cols + c.0) as usize
    }

    fn start_board(&mut self) {
        let (_, cols, rows, n_mines, base) = PRESETS[self.preset_index];
        self.cols = cols;
        self.rows = rows;
        self.n_mines = n_mines;
        self.base_points = base;
        let n = (cols * rows) as usize;
        self.mines = vec![false; n];
        self.numbers = vec![0; n];
        self.revealed = vec![false; n];
        self.n_revealed = 0;
        self.flags = vec![0; n];
        self.flag_used = false;
        self.first_click = true;
        self.exploded = None;
        self.won = false;
        self.b.game_over = false;
        self.b.score = 0;
        self.elapsed = 0.0;
        self.running = false;
        self.hover = None;
        self.surprise = 0.0;
        self.particles.clear();
        self.particle_cols.clear();
        self.layout_board();
    }

    fn layout_board(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let aw = w - 24;
        let ah = h - HUD_H - 24;
        self.cell = 10.max(py::fdiv32(aw, self.cols).min(py::fdiv32(ah, self.rows)));
        let (bw, bh) = (self.cell * self.cols, self.cell * self.rows);
        self.bx = py::fdiv32(w - bw, 2);
        self.by = HUD_H + 12 + py::fdiv32(ah - bh, 2);
        self.num_font = ui::fb(11.max((self.cell as f64 * 0.55) as i32));
        self.face_rect = Rect::new(w / 2 - 20, 8, 40, 40);
    }

    fn neighbors(&self, c: Cell) -> Vec<Cell> {
        let mut out = Vec::with_capacity(8);
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let (nx, ny) = (c.0 + dx, c.1 + dy);
                if 0 <= nx && nx < self.cols && 0 <= ny && ny < self.rows {
                    out.push((nx, ny));
                }
            }
        }
        out
    }

    fn cell_at(&self, pos: (i32, i32)) -> Option<Cell> {
        let x = py::fdiv32(pos.0 - self.bx, self.cell);
        let y = py::fdiv32(pos.1 - self.by, self.cell);
        if 0 <= x && x < self.cols && 0 <= y && y < self.rows {
            Some((x, y))
        } else {
            None
        }
    }

    fn place_mines(&mut self, safe: Cell) {
        let mut tabu = self.neighbors(safe);
        tabu.push(safe);
        let mut alle = Vec::new();
        for x in 0..self.cols {
            for y in 0..self.rows {
                if !tabu.contains(&(x, y)) {
                    alle.push((x, y));
                }
            }
        }
        let k = self.n_mines.min(alle.len());
        let gewaehlt = py::sample(&alle, k);
        self.mines.iter_mut().for_each(|m| *m = false);
        for c in gewaehlt {
            let i = self.idx(c);
            self.mines[i] = true;
        }
        for x in 0..self.cols {
            for y in 0..self.rows {
                let i = self.idx((x, y));
                if self.mines[i] {
                    continue;
                }
                self.numbers[i] = self.neighbors((x, y)).iter().filter(|nb| self.mines[self.idx(**nb)]).count() as i32;
            }
        }
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 440.min(self.b.width - 60);
        let (bh, gap) = (46, 10);
        let y0 = 124.max((self.b.height as f64 * 0.24) as i32);
        self.preset_rects = (0..PRESETS.len() as i32).map(|i| Rect::new(cx - py::fdiv32(bw, 2), y0 + i * (bh + gap), bw, bh)).collect();
        let y1 = y0 + PRESETS.len() as i32 * (bh + gap) + 4;
        self.qmark_rect = Rect::new(cx - py::fdiv32(bw, 2), y1, bw, 40);
        self.start_rect = Rect::new(cx - 95, y1 + 52, 190, 50);
    }

    fn select_preset(&mut self, i: usize) {
        self.preset_index = i;
        self.b.set_opt("minesweeper", "preset", PRESETS[i].0.into());
        self.b.play_sound("click");
    }

    fn toggle_qmarks(&mut self) {
        self.qmarks = !self.qmarks;
        self.b.set_opt("minesweeper", "qmarks", self.qmarks.into());
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.start_board();
        self.setup = false;
        self.b.play_sound("click");
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" => self.select_preset(k.parse::<usize>().unwrap_or(1) - 1),
                "Up" | "w" | "W" => self.select_preset((self.preset_index + PRESETS.len() - 1) % PRESETS.len()),
                "Down" | "s" | "S" => self.select_preset((self.preset_index + 1) % PRESETS.len()),
                "q" | "Q" | "f" | "F" => self.toggle_qmarks(),
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            let p = ev.pos;
            if let Some(i) = self.preset_rects.iter().position(|r| r.hit(p)) {
                self.select_preset(i);
                return;
            }
            if self.qmark_rect.hit(p) {
                self.toggle_qmarks();
            } else if self.start_rect.hit(p) {
                self.start_play();
            }
        }
    }

    fn toggle_flag(&mut self, c: Cell) {
        let i = self.idx(c);
        if self.revealed[i] || self.b.game_over {
            return;
        }
        match self.flags[i] {
            0 => {
                self.flags[i] = 1;
                self.flag_used = true;
            }
            1 if self.qmarks => self.flags[i] = 2,
            _ => self.flags[i] = 0,
        }
        self.b.play_sound("move");
    }

    fn click_cell(&mut self, c: Cell) {
        self.surprise = SURPRISE_T;
        let i = self.idx(c);
        if self.revealed[i] {
            self.chord(c);
            return;
        }
        if self.flags[i] == 1 {
            return;
        }
        self.reveal(c);
    }

    fn chord(&mut self, c: Cell) {
        let zahl = self.numbers[self.idx(c)];
        if zahl <= 0 {
            return;
        }
        let nbs = self.neighbors(c);
        let flaggen = nbs.iter().filter(|nb| self.flags[self.idx(**nb)] == 1).count() as i32;
        if flaggen != zahl {
            return;
        }
        for nb in nbs {
            let j = self.idx(nb);
            if !self.revealed[j] && self.flags[j] != 1 {
                self.reveal(nb);
                if self.b.game_over {
                    return;
                }
            }
        }
    }

    fn reveal(&mut self, c: Cell) {
        if self.first_click {
            self.place_mines(c);
            self.first_click = false;
            self.running = true;
        }
        if self.mines[self.idx(c)] {
            self.lose(c);
            return;
        }
        let mut stapel = vec![c];
        let mut neu = 0;
        while let Some(cc) = stapel.pop() {
            let i = self.idx(cc);
            if self.revealed[i] || self.mines[i] {
                continue;
            }
            self.revealed[i] = true;
            self.n_revealed += 1;
            self.flags[i] = 0;
            neu += 1;
            if self.numbers[i] == 0 {
                for nb in self.neighbors(cc) {
                    if !self.revealed[self.idx(nb)] {
                        stapel.push(nb);
                    }
                }
            }
        }
        if neu > 0 {
            self.b.play_sound(if neu == 1 { "click" } else { "merge" });
        }
        self.check_win();
    }

    fn lose(&mut self, c: Cell) {
        self.exploded = Some(c);
        let i = self.idx(c);
        if !self.revealed[i] {
            self.revealed[i] = true;
            self.n_revealed += 1;
        }
        self.b.game_over = true;
        self.won = false;
        self.b.score = 0;
        self.b.report_result(false);
        self.b.play_sound("explode");
        self.b.play_sound("gameover");
        self.b.rumble(250);
    }

    fn check_win(&mut self) {
        if self.n_revealed != (self.cols * self.rows) as usize - self.n_mines {
            return;
        }
        self.b.game_over = true;
        self.won = true;
        if !self.flag_used {
            self.b.ach_event("mine_noflag", None);
        }
        for i in 0..self.mines.len() {
            if self.mines[i] {
                self.flags[i] = 1;
            }
        }
        let sekunden = self.elapsed as i64;
        self.b.score = 10.max(self.base_points - sekunden);
        let key = PRESETS[self.preset_index].0;
        let alt = self.best.get(key).and_then(|v| v.as_f64()).map(|v| v as i64);
        if alt.is_none_or(|a| sekunden < a) {
            self.best.insert(key.into(), sekunden.into());
            self.b.set_opt("minesweeper", "best", Value::Object(self.best.clone()));
        }
        self.b.report_result(true);
        self.b.ach_event("mine_win", None);
        self.confetti();
        self.b.play_sound("win");
        self.b.rumble(200);
    }

    fn confetti(&mut self) {
        let w = self.b.width as f64;
        for _ in 0..90 {
            let x = py::uniform(0.0, w);
            let farbe = py::choice(&NUMBER_COLORS);
            let y = py::uniform(-80.0, 0.0);
            let vx = py::uniform(-30.0, 30.0);
            let vy = py::uniform(60.0, 190.0);
            let life = py::uniform(1.5, 3.2);
            self.particles.push([x, y, vx, vy, life]);
            self.particle_cols.push(farbe);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let hud = Rect::new(10, 6, self.b.width - 20, HUD_H - 4);
        draw::rect_r(s, p.panel, hud, 0, 10);
        draw::rect_r(s, p.border, hud, 1, 10);
        let flaggen = self.flags.iter().filter(|&&f| f == 1).count() as i64;
        let wert = (-99i64).max(self.n_mines as i64 - flaggen);
        let led_l = self.draw_led(s, &format!("{wert:03}"), 24, None);
        let sek = self.elapsed as i64;
        self.draw_led(s, &format!("{:03}", sek.min(999)), 0, Some(self.b.width - 24));
        let name = self.tiny.render(&t(&format!("mines.preset.{}", PRESETS[self.preset_index].0)), true, p.text_dim);
        s.bl(&name, (led_l.right() + 12, HUD_H / 2 - name.h() / 2 + 2));
        self.draw_face(s);
    }

    fn draw_led(&self, s: &mut Surface, text: &str, links: i32, rechts: Option<i32>) -> Rect {
        let img = self.led.render(text, true, COL_LED);
        let (w, h) = (img.w() + 14, img.h() + 8);
        let x = rechts.map_or(links, |r| r - w);
        let y = py::fdiv32(HUD_H - h, 2) + 2;
        let rect = Rect::new(x, y, w, h);
        draw::rect_r(s, rgb(16, 12, 14), rect, 0, 6);
        draw::rect_r(s, rgb(60, 40, 44), rect, 1, 6);
        s.bl(&img, (x + 7, y + h / 2 - img.h() / 2));
        rect
    }

    fn draw_face(&self, s: &mut Surface) {
        let p = ui::pal();
        let r = self.face_rect;
        draw::rect_r(s, p.btn, r, 0, 8);
        draw::rect_r(s, COL_BEVEL_L, r, 2, 8);
        let (cx, cy) = r.center();
        draw::circle(s, rgb(245, 205, 90), (cx, cy), 14, 0);
        draw::circle(s, rgb(120, 95, 30), (cx, cy), 14, 2);
        let braun = rgb(60, 40, 20);
        if self.b.game_over && !self.won {
            for ex in [cx - 6, cx + 6] {
                draw::line(s, braun, (ex - 3, cy - 8), (ex + 3, cy - 2), 2);
                draw::line(s, braun, (ex + 3, cy - 8), (ex - 3, cy - 2), 2);
            }
            draw::line(s, braun, (cx - 6, cy + 7), (cx + 6, cy + 7), 2);
        } else if self.b.game_over && self.won {
            let dunkel = rgb(30, 30, 40);
            draw::rect_r(s, dunkel, (cx - 10, cy - 8, 8, 6), 0, 2);
            draw::rect_r(s, dunkel, (cx + 2, cy - 8, 8, 6), 0, 2);
            draw::line(s, dunkel, (cx - 2, cy - 6), (cx + 2, cy - 6), 2);
            draw::arc(s, braun, (cx - 7, cy - 2, 14, 11), py::PI, py::TAU, 2);
        } else if self.surprise > 0.0 {
            draw::circle(s, braun, (cx - 5, cy - 5), 2, 0);
            draw::circle(s, braun, (cx + 5, cy - 5), 2, 0);
            draw::circle(s, braun, (cx, cy + 6), 4, 2);
        } else {
            draw::circle(s, braun, (cx - 5, cy - 5), 2, 0);
            draw::circle(s, braun, (cx + 5, cy - 5), 2, 0);
            draw::arc(s, braun, (cx - 7, cy - 3, 14, 12), py::PI, py::TAU, 2);
        }
    }

    fn draw_board(&self, s: &mut Surface) {
        let p = ui::pal();
        let brett = Rect::new(self.bx - 8, self.by - 8, self.cols * self.cell + 16, self.rows * self.cell + 16);
        draw::rect_r(s, p.panel, brett, 0, 10);
        draw::rect_r(s, p.border, brett, 1, 10);
        for x in 0..self.cols {
            for y in 0..self.rows {
                self.draw_cell(s, (x, y));
            }
        }
    }

    fn draw_cell(&self, s: &mut Surface, cell: Cell) {
        let p = ui::pal();
        let c = self.cell;
        let (rx, ry) = (self.bx + cell.0 * c, self.by + cell.1 * c);
        let rect = Rect::new(rx, ry, c, c);
        let i = self.idx(cell);
        let offen = self.revealed[i];
        let zeige_mine = self.b.game_over && self.mines[i];
        if offen || (zeige_mine && self.flags[i] != 1) {
            let grund = if Some(cell) == self.exploded { COL_BOOM } else { COL_OPEN };
            draw::rect(s, grund, rect, 0);
            draw::rect(s, COL_OPEN_LINE, rect, 1);
            if self.mines[i] {
                self.draw_mine(s, rect);
            } else {
                let zahl = self.numbers[i];
                if zahl > 0 {
                    let img = self.num_font.render(&zahl.to_string(), true, NUMBER_COLORS[(zahl - 1) as usize]);
                    s.blr(&img, img.get_rect().center_at(rect.center()));
                }
            }
        } else {
            let grund = if Some(cell) == self.hover && !self.b.game_over { COL_TILE_HOVER } else { COL_TILE };
            draw::rect(s, grund, rect, 0);
            draw::line(s, COL_BEVEL_L, (rx, ry), (rx + c - 1, ry), 2);
            draw::line(s, COL_BEVEL_L, (rx, ry), (rx, ry + c - 1), 2);
            draw::line(s, COL_BEVEL_D, (rx + 1, ry + c - 1), (rx + c - 1, ry + c - 1), 2);
            draw::line(s, COL_BEVEL_D, (rx + c - 1, ry + 1), (rx + c - 1, ry + c - 1), 2);
            match self.flags[i] {
                1 => {
                    self.draw_flag(s, rect);
                    if self.b.game_over && !self.mines[i] {
                        let rot = rgb(250, 90, 90);
                        draw::line(s, rot, rect.topleft(), rect.bottomright(), 2);
                        draw::line(s, rot, rect.topright(), rect.bottomleft(), 2);
                    }
                }
                2 => {
                    let img = self.num_font.render("?", true, p.text_dim);
                    s.blr(&img, img.get_rect().center_at(rect.center()));
                }
                _ => {}
            }
        }
    }

    fn draw_mine(&self, s: &mut Surface, rect: Rect) {
        let (cx, cy) = rect.center();
        let r = 3.max(self.cell / 4);
        let dunkel = rgb(20, 22, 30);
        for winkel in (0..360).step_by(45) {
            let a = py::radians(winkel as f64);
            let ende = (cx as f64 + a.cos() * (r + 3) as f64, cy as f64 + a.sin() * (r + 3) as f64);
            draw::line(s, dunkel, (cx, cy), ende, 2);
        }
        draw::circle(s, dunkel, (cx, cy), r, 0);
        draw::circle(s, rgb(90, 95, 110), (cx - r / 3, cy - r / 3), 1.max(r / 4), 0);
    }

    fn draw_flag(&self, s: &mut Surface, rect: Rect) {
        let (cx, cy) = rect.center();
        let (cx, cy) = (cx as f64, cy as f64);
        let h = self.cell as f64 * 0.6;
        let x = cx - 1.0;
        let hell = rgb(200, 205, 215);
        draw::line(s, hell, (x, cy - h / 2.0), (x, cy + h / 2.0), 2);
        draw::polygon(s, COL_FLAG, &[(x, cy - h / 2.0), (x + h * 0.55, cy - h * 0.28), (x, cy - h * 0.06)], 0);
        draw::line(s, hell, (x - h * 0.25, cy + h / 2.0), (x + h * 0.3, cy + h / 2.0), 2);
    }

    fn draw_result(&self, s: &mut Surface) {
        let p = ui::pal();
        let (text, farbe) = if self.won {
            (tf("mines.win", &[("t", &format!("{}s", self.elapsed as i64))]), p.green)
        } else {
            (t("mines.lose"), p.red)
        };
        let img = self.b.font.render(&text, true, farbe);
        let hint_col = ui::mix(p.text_dim, p.text, ui::pulse(2.0, 0.2, 1.0));
        let hint = self.small.render(&t("common.enter_restart"), true, hint_col);
        let w = img.w().max(hint.w()) + 36;
        let h = img.h() + hint.h() + 22;
        let r = Rect::new(self.b.width / 2 - w / 2, self.by - 10, w, h);
        let mut panel = pg::surface_alpha(w, h);
        draw::rect_r(&mut panel, pg::alpha(p.panel, 235), (0, 0, w, h), 0, 10);
        draw::rect_r(&mut panel, farbe, (0, 0, w, h), 2, 10);
        s.bl(&panel, (r.x, r.y));
        s.blr(&img, img.get_rect().midtop_at((r.centerx(), r.y + 8)));
        s.blr(&hint, hint.get_rect().midtop_at((r.centerx(), r.y + 8 + img.h() + 4)));
    }

    fn draw_setup(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        ui::draw_title_ex(&mut s, w, "MINESWEEPER", Some(&t("mines.subtitle")), 52, None, None, Some(self.b.accent));
        for (i, (key, cols, rows, minen, _pts)) in PRESETS.iter().enumerate() {
            let r = self.preset_rects[i];
            let an = i == self.preset_index;
            draw::rect_r(&mut s, if an { p.btn_sel } else { p.btn }, r, 0, 8);
            draw::rect_r(&mut s, if an { self.b.accent } else { p.border }, r, if an { 2 } else { 1 }, 8);
            let lab = self.b.font.render(&t(&format!("mines.preset.{key}")), true, p.text);
            s.bl(&lab, (r.x + 16, r.y + 5));
            let det = self.tiny.render(&format!("{cols}x{rows}   {minen} {}", t("mines.mines")), true, p.text_dim);
            s.bl(&det, (r.x + 16, r.bottom() - det.h() - 5));
            let best = self.best.get(*key).and_then(|v| v.as_f64());
            let btxt = match best {
                Some(b) => tf("mines.best", &[("t", &format!("{}s", b as i64))]),
                None => t("mines.best_none"),
            };
            let img = self.small.render(&btxt, true, if best.is_some() { p.gold } else { p.text_dim });
            s.bl(&img, (r.right() - img.w() - 16, r.centery() - img.h() / 2));
        }
        let r = self.qmark_rect;
        draw::rect_r(&mut s, if self.qmarks { p.btn_sel } else { p.btn }, r, 0, 8);
        draw::rect_r(&mut s, p.border, r, 1, 8);
        let lab = self.b.font.render(&t("mines.qmarks"), true, p.text);
        s.bl(&lab, (r.x + 16, r.centery() - lab.h() / 2));
        let wert = if self.qmarks { t("common.on") } else { t("common.off") };
        let img = self.b.font.render(&format!("< {wert} >"), true, if self.qmarks { self.b.accent } else { p.text_dim });
        s.bl(&img, (r.right() - img.w() - 16, r.centery() - img.h() / 2));
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(p.green));
        let h2 = self.tiny.render(&t("mines.hint"), true, p.green);
        s.blr(&h2, h2.get_rect().center_at((w / 2, h - 48)));
        ui::draw_footer(&mut s, w, h, &t("mines.setup_hint"));
    }
}

impl Game for MinesweeperGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !self.setup
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
        let key = self.b.opt_str("minesweeper", "preset", "beginner");
        self.preset_index = PRESETS.iter().position(|p| p.0 == key).unwrap_or(0);
        self.qmarks = self.b.opt_bool("minesweeper", "qmarks", false);
        self.best = match self.b.opt_value("minesweeper", "best") {
            Some(Value::Object(m)) => m,
            _ => Map::new(),
        };
        self.make_fonts();
        self.anim_t = 0.0;
        self.build_setup_layout();
        self.setup = true;
        self.start_board();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.build_setup_layout();
        self.layout_board();
        self.hover = None;
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.handle_setup_event(ev);
            return;
        }
        if ev.kind == Kind::MouseMove {
            self.hover = self.cell_at(ev.pos);
            return;
        }
        if ev.kind == Kind::MouseDown {
            if ev.button == 1 && self.face_rect.hit(ev.pos) {
                self.start_board();
                self.b.play_sound("click");
                return;
            }
            if self.b.game_over {
                return;
            }
            let Some(cell) = self.cell_at(ev.pos) else { return };
            if ev.button == 3 {
                self.toggle_flag(cell);
            } else {
                self.click_cell(cell);
            }
            return;
        }
        if ev.kind != Kind::KeyDown {
            return;
        }
        let k = ev.key.as_str();
        if self.b.game_over {
            if matches!(k, "Return" | "space" | "r" | "R") {
                self.start_board();
            } else if matches!(k, "s" | "S") {
                self.setup = true;
                self.b.play_sound("click");
            }
            return;
        }
        if matches!(k, "f" | "F") && self.hover.is_some() {
            let h = self.hover.unwrap();
            self.toggle_flag(h);
        } else if matches!(k, "r" | "R") {
            self.start_board();
            self.b.play_sound("click");
        } else if matches!(k, "s" | "S") {
            self.setup = true;
            self.b.play_sound("click");
        }
    }

    fn update(&mut self, dt: f64) {
        self.anim_t += dt;
        if self.surprise > 0.0 {
            self.surprise -= dt;
        }
        let hmax = (self.b.height + 10) as f64;
        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            p[0] += p[2] * dt;
            p[1] += p[3] * dt;
            p[3] += 60.0 * dt;
            p[4] -= dt;
            if p[4] > 0.0 && p[1] < hmax {
                i += 1;
            } else {
                self.particles.remove(i);
                self.particle_cols.remove(i);
            }
        }
        if !self.setup && self.running && !self.b.game_over {
            self.elapsed += dt;
        }
    }

    fn draw(&mut self) {
        if self.setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        self.draw_hud(&mut s);
        self.draw_board(&mut s);
        for (p, c) in self.particles.iter().zip(&self.particle_cols) {
            draw::rect(&mut s, *c, (p[0] as i32, p[1] as i32, 4, 6), 0);
        }
        if self.b.game_over {
            self.draw_result(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "minesweeper",
    class_name: "MinesweeperGame",
    supports_multiplayer: false,
    modes: &[],
    make: MinesweeperGame::new,
};
