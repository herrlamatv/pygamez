//! slidepuzzle.rs
//! ==============
//! Schiebepuzzle (15-Puzzle) in drei Groessen.
//!
//! Uebertragen aus `games/slidepuzzle.py`: Mischen durch gueltige Zuege
//! (immer loesbar), Linienschieben per Klick, Punkte aus Groesse, Zuegen und
//! Zeit, Aufleuchten des Zielfelds.

use pgz_engine::prelude::*;

const TIME_PEN: i64 = 5;

fn base(n: usize) -> i64 {
    match n {
        3 => 2000,
        4 => 6000,
        5 => 12000,
        _ => 6000,
    }
}

fn move_pen(n: usize) -> i64 {
    match n {
        3 => 8,
        4 => 6,
        5 => 4,
        _ => 6,
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Zustand {
    Play,
    Solved,
}

/// `SlidingPuzzleGame`
pub struct SlidingPuzzleGame {
    b: GameBase,
    n: usize,
    state: Zustand,
    moves: i64,
    elapsed: f64,
    flash: Vec<(usize, f64)>,
    solved: Vec<usize>,
    board: Vec<usize>,
    blank: usize,
    board_px: i32,
    gap: i32,
    tile: i32,
    ox: i32,
    oy: i32,
    tile_font: Font,
    hud: Font,
    small: Font,
    huge: Font,
}

impl SlidingPuzzleGame {
    /// `SlidingPuzzleGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = SlidingPuzzleGame {
            b,
            n: 4,
            state: Zustand::Play,
            moves: 0,
            elapsed: 0.0,
            flash: Vec::new(),
            solved: Vec::new(),
            board: Vec::new(),
            blank: 0,
            board_px: 0,
            gap: 0,
            tile: 0,
            ox: 0,
            oy: 0,
            tile_font: f.clone(),
            hud: f.clone(),
            small: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.tile_font = ui::fb(22.max(h / 12));
        self.hud = ui::fb(20);
        self.small = ui::f(15);
        self.huge = ui::fb(30.max(h / 11));
    }

    fn new_board(&mut self) {
        let n = self.n;
        self.solved = (1..n * n).chain(std::iter::once(0)).collect();
        self.board = self.solved.clone();
        self.blank = n * n - 1;
        self.scramble();
        self.moves = 0;
        self.elapsed = 0.0;
        self.state = Zustand::Play;
        self.flash.clear();
    }

    fn scramble(&mut self) {
        let n = self.n;
        let mut last: i64 = -1;
        for _ in 0..n * n * 60 {
            let nb: Vec<usize> = self.neighbors(self.blank).into_iter().filter(|i| *i as i64 != last).collect();
            let pick = py::choice(&nb);
            last = self.blank as i64;
            self.board.swap(self.blank, pick);
            self.blank = pick;
        }
        if self.board == self.solved {
            self.scramble();
        }
    }

    fn neighbors(&self, idx: usize) -> Vec<usize> {
        let n = self.n;
        let (r, c) = (idx / n, idx % n);
        let mut out = Vec::new();
        if r > 0 {
            out.push(idx - n);
        }
        if r < n - 1 {
            out.push(idx + n);
        }
        if c > 0 {
            out.push(idx - 1);
        }
        if c < n - 1 {
            out.push(idx + 1);
        }
        out
    }

    fn layout(&mut self) {
        let n = self.n as i32;
        let top = 74;
        let bottom = self.b.height - 40;
        let avail = (self.b.width - 48).min(bottom - top);
        self.board_px = 120.max(avail);
        self.gap = 3.max(self.board_px / (n * 22));
        self.tile = (self.board_px - (n + 1) * self.gap) / n;
        self.board_px = n * self.tile + (n + 1) * self.gap;
        self.ox = (self.b.width - self.board_px) / 2;
        self.oy = top + 0.max((bottom - top - self.board_px) / 2);
    }

    fn tile_rect(&self, idx: usize) -> Rect {
        let (r, c) = ((idx / self.n) as i32, (idx % self.n) as i32);
        let x = self.ox + self.gap + c * (self.tile + self.gap);
        let y = self.oy + self.gap + r * (self.tile + self.gap);
        Rect::new(x, y, self.tile, self.tile)
    }

    fn click(&mut self, pos: (i32, i32)) {
        for idx in 0..self.n * self.n {
            if idx != self.blank && self.tile_rect(idx).hit(pos) {
                self.slide_line(idx);
                return;
            }
        }
    }

    fn slide_from(&mut self, idx: i64) {
        let n = self.n as i64;
        if 0 <= idx && idx < n * n && self.neighbors(self.blank).contains(&(idx as usize)) {
            self.swap(idx as usize);
        }
    }

    fn slide_line(&mut self, idx: usize) {
        let n = self.n;
        let (br, bc) = (self.blank / n, self.blank % n);
        let (cr, cc) = (idx / n, idx % n);
        if cr == br {
            while self.blank != idx {
                let ziel = if cc > bc { self.blank + 1 } else { self.blank - 1 };
                self.swap(ziel);
            }
        } else if cc == bc {
            while self.blank != idx {
                let ziel = if cr > br { self.blank + n } else { self.blank - n };
                self.swap(ziel);
            }
        } else {
            self.b.play_sound("click");
        }
    }

    fn swap(&mut self, idx: usize) {
        self.board.swap(self.blank, idx);
        let blank = self.blank;
        match self.flash.iter_mut().find(|(i, _)| *i == blank) {
            Some(e) => e.1 = 0.18,
            None => self.flash.push((blank, 0.18)),
        }
        self.blank = idx;
        self.moves += 1;
        self.b.play_sound("move");
        if self.board == self.solved {
            self.win();
        }
    }

    fn win(&mut self) {
        let n = self.n;
        let pts = 50.max(base(n) - self.moves * move_pen(n) - (self.elapsed as i64) * TIME_PEN);
        self.b.score = pts;
        self.state = Zustand::Solved;
        self.b.game_over = true;
        self.b.report_result(true);
        if n == 4 && self.moves < 100 {
            self.b.ach_event("slide_swift", None);
        }
        self.b.play_sound("win");
    }

    fn fmt_time(&self) -> String {
        let sec = self.elapsed as i64;
        format!("{}:{:02}", sec / 60, sec % 60)
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let img = self.hud.render(&t("slide.title"), true, self.b.accent);
        s.blr(&img, img.get_rect().midleft_at((20, 30)));
        let img = self.small.render(&tf("slide.moves", &[("n", &self.moves.to_string())]), true, p.text);
        s.blr(&img, img.get_rect().midright_at((self.b.width - 20, 22)));
        let img = self.small.render(&tf("slide.time", &[("t", &self.fmt_time())]), true, p.text_dim);
        s.blr(&img, img.get_rect().midright_at((self.b.width - 20, 44)));
        let img = self.small.render(&t("slide.hint"), true, p.text_faint);
        s.blr(&img, img.get_rect().center_at((self.b.width / 2, 52)));
    }

    fn draw_board(&self, s: &mut Surface) {
        let pad = self.gap;
        let panel = Rect::new(self.ox - pad, self.oy - pad, self.board_px + 2 * pad, self.board_px + 2 * pad);
        draw::rect_r(s, Color::rgb(24, 28, 42), panel, 0, 12);
        draw::rect_r(s, mix(self.b.accent, Color::rgb(20, 24, 38), 0.6), panel, 2, 12);
        for idx in 0..self.n * self.n {
            let val = self.board[idx];
            if val == 0 {
                continue;
            }
            let rc = self.tile_rect(idx);
            let mut base = mix(self.b.accent, Color::rgb(235, 240, 250), 0.12);
            if let Some((_, f)) = self.flash.iter().find(|(i, _)| *i == idx) {
                base = mix(base, Color::rgb(255, 255, 255), (f / 0.18).min(1.0));
            }
            let correct = val == self.solved[idx];
            let fill = if correct { base } else { mix(base, Color::rgb(40, 46, 66), 0.42) };
            draw::rect_r(s, fill, rc, 0, 8);
            draw::rect_r(s, mix(fill, Color::rgb(255, 255, 255), 0.18), rc, 2, 8);
            let col = if correct { Color::rgb(20, 24, 34) } else { ui::text() };
            let img = self.tile_font.render(&val.to_string(), true, col);
            s.blr(&img, img.get_rect().center_at(rc.center()));
        }
    }

    fn draw_banner(&self, s: &mut Surface) {
        let w = (self.b.width - 40).min(460);
        let h = 118;
        let rc = Rect::new((self.b.width - w) / 2, (self.b.height - h) / 2, w, h);
        let mut panel = pg::surface_alpha(w, h);
        panel.fill_c(Color::rgba(16, 18, 24, 236));
        s.bl(&panel, (rc.x, rc.y));
        draw::rect_r(s, self.b.accent, rc, 2, 14);
        let img = self.huge.render(&t("slide.solved"), true, self.b.accent);
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 38)));
        let img = self.hud.render(&tf("slide.score", &[("n", &self.b.score.to_string())]), true, ui::gold());
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 74)));
        let img = self.small.render(&t("common.enter_restart"), true, ui::text_dim());
        s.blr(&img, img.get_rect().center_at((rc.centerx(), rc.y + 100)));
    }
}

impl Game for SlidingPuzzleGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.n = match self.b.mode.as_str() {
            "3" => 3,
            "5" => 5,
            _ => 4,
        };
        self.b.score = 0;
        self.b.game_over = false;
        self.state = Zustand::Play;
        self.moves = 0;
        self.elapsed = 0.0;
        self.flash.clear();
        self.make_fonts();
        self.new_board();
        self.layout();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == Zustand::Solved {
            if ev.kind == Kind::MouseDown || ev.is_any(&["Return", "space"]) {
                self.b.game_over = false;
                self.new_board();
                self.b.play_sound("click");
            }
            return;
        }
        let n = self.n as i64;
        let blank = self.blank as i64;
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "Up" | "w" | "W" => self.slide_from(blank + n),
                "Down" | "s" | "S" => self.slide_from(blank - n),
                "Left" | "a" | "A" => self.slide_from(blank + 1),
                "Right" | "d" | "D" => self.slide_from(blank - 1),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            self.click(ev.pos);
        }
    }

    fn update(&mut self, dt: f64) {
        if self.state == Zustand::Play {
            self.elapsed += dt;
        }
        for e in self.flash.iter_mut() {
            e.1 -= dt;
        }
        self.flash.retain(|(_, f)| *f > 0.0);
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        self.draw_hud(&mut s);
        self.draw_board(&mut s);
        if self.state == Zustand::Solved {
            self.draw_banner(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "slide",
    class_name: "SlidingPuzzleGame",
    supports_multiplayer: false,
    modes: &[("3", "slide.mode.3"), ("4", "slide.mode.4"), ("5", "slide.mode.5")],
    make: SlidingPuzzleGame::new,
};
