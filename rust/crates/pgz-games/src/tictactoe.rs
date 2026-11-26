//! tictactoe.rs
//! ============
//! Tic-Tac-Toe mit Setup-Menue, Brettgroessen 3x3 bis 9x9 und KI-Stufen.
//!
//! Uebertragen aus `games/tictactoe.py`: allgemeines m,n,k-Spiel (K = 3, 4
//! oder 5 in einer Reihe), KI Easy (zufaellig), Medium (gewinnt/blockt,
//! sonst Heuristik) und Hard (Minimax bzw. tiefenbegrenzte Alpha-Beta-Suche).
//! Die Kandidatenmenge der KI ist ein Python-`set` - darum `py::IntSet`,
//! damit bei Gleichstand dasselbe Feld gewinnt.

use pgz_engine::prelude::*;
use pgz_engine::py::IntSet;

const COL_LINE: Color = Color::rgb(90, 95, 120);
const COL_X: Color = Color::rgb(90, 180, 255);
const COL_O: Color = Color::rgb(255, 140, 90);
const COL_WIN: Color = Color::rgb(240, 240, 120);
const COL_WIN_BG: Color = Color::rgb(60, 70, 40);

const EMPTY: u8 = 0;
const HUMAN: u8 = 1;
const AI: u8 = 2;

const DIFF_ORDER: [&str; 3] = ["Easy", "Medium", "Hard"];
const SIZES: [i32; 7] = [3, 4, 5, 6, 7, 8, 9];
const WIN_SCORE: i64 = 10_000_000;
const LINE_WEIGHTS: [i64; 6] = [0, 1, 12, 120, 1200, 12000];
const BIG: i64 = 1_000_000_000_000_000_000;

fn win_length(n: i32) -> i32 {
    if n <= 4 {
        n
    } else {
        5
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

#[derive(Clone, Copy, PartialEq)]
enum Winner {
    Sym(u8),
    Draw,
}

/// `TicTacToeGame`
pub struct TicTacToeGame {
    b: GameBase,
    wins_x: i64,
    wins_o: i64,
    streak: i64,
    state: State,
    diff_name: &'static str,
    size: i32,
    small: Font,
    mid: Font,
    diff_y: i32,
    diff_rects: Vec<(&'static str, Rect)>,
    size_y: i32,
    size_rects: Vec<(i32, Rect)>,
    start_rect: Rect,
    n: i32,
    k: i32,
    max_depth: i32,
    windows: Vec<Vec<usize>>,
    board: Vec<u8>,
    current: u8,
    winner: Option<Winner>,
    win_cells: Option<Vec<usize>>,
    ai_timer: f64,
    board_size: i32,
    cell: i32,
    ox: i32,
    oy: i32,
}

impl TicTacToeGame {
    /// `TicTacToeGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = TicTacToeGame {
            b,
            wins_x: 0,
            wins_o: 0,
            streak: 0,
            state: State::Setup,
            diff_name: "Hard",
            size: 3,
            small: f.clone(),
            mid: f,
            diff_y: 0,
            diff_rects: Vec::new(),
            size_y: 0,
            size_rects: Vec::new(),
            start_rect: z,
            n: 3,
            k: 3,
            max_depth: 9,
            windows: Vec::new(),
            board: Vec::new(),
            current: HUMAN,
            winner: None,
            win_cells: None,
            ai_timer: 0.0,
            board_size: 0,
            cell: 1,
            ox: 0,
            oy: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.b.font = ui::f(16.max(h / 24));
        self.b.big_font = ui::fb(30.max(h / 11));
        self.small = ui::f(13.max(h / 32));
        self.mid = ui::fb(15.max(h / 27));
    }

    fn diff_label(name: &str) -> String {
        t(&format!("ttt.diff.{}", name.to_lowercase()))
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let h = self.b.height;
        let bh = 40.max(52.min(h / 10));
        self.diff_y = 120.max((h as f64 * 0.25) as i32);
        self.diff_rects = DIFF_ORDER
            .iter()
            .enumerate()
            .map(|(i, name)| (*name, Rect::new(cx - 165 + i as i32 * 112, self.diff_y, 100, bh)))
            .collect();
        self.size_y = self.diff_y + bh + 60.max((h as f64 * 0.12) as i32);
        let (bw, gap) = (52, 6);
        let start = cx - py::fdiv32(SIZES.len() as i32 * (bw + gap) - gap, 2);
        self.size_rects = SIZES
            .iter()
            .enumerate()
            .map(|(i, n)| (*n, Rect::new(start + i as i32 * (bw + gap), self.size_y, bw, bh)))
            .collect();
        self.start_rect = Rect::new(cx - 90, self.size_y + bh + 46, 180, 50);
    }

    fn draw_setup(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let mp = self.b.multiplayer;
        let titel = if mp { t("ttt.setup_title_mp") } else { t("ttt.setup_title") };
        ui::draw_title_ex(&mut s, w, &titel, None, 52, None, None, Some(self.b.accent));
        let label_col = if mp { p.text_faint } else { p.text_dim };
        let img = self.mid.render(&t("ttt.difficulty"), true, label_col);
        s.bl(&img, (w / 2 - 165, self.diff_y - img.h() - 8));
        for (name, r) in &self.diff_rects {
            if mp {
                draw::rect_r(&mut s, p.btn, *r, 0, 8);
                draw::rect_r(&mut s, p.border, *r, 1, 8);
                let lab = self.mid.render(&Self::diff_label(name), true, p.text_faint);
                s.blr(&lab, lab.get_rect().center_at(r.center()));
            } else {
                ui::draw_button_ex(&mut s, *r, &Self::diff_label(name), &self.mid, *name == self.diff_name, None, None, Some(self.b.accent));
            }
        }
        let img = self.mid.render(&t("ttt.field"), true, p.text_dim);
        s.bl(&img, (w / 2 - 165, self.size_y - img.h() - 8));
        for (n, r) in &self.size_rects {
            ui::draw_button_ex(&mut s, *r, &format!("{n}x{n}"), &self.mid, *n == self.size, None, None, Some(self.b.accent));
        }
        let info = self.small.render(&tf("ttt.goal", &[("k", &win_length(self.size).to_string())]), true, p.text_dim);
        s.blr(&info, info.get_rect().center_at((w / 2, self.start_rect.y - 22)));
        ui::draw_button_ex(&mut s, self.start_rect, &t("common.start"), &self.mid, true, None, None, Some(p.green));
        ui::draw_footer(&mut s, w, h, &t("ttt.setup_hint"));
    }

    fn handle_setup_event(&mut self, ev: &InputEvent) {
        let mp = self.b.multiplayer;
        if ev.kind == Kind::KeyDown {
            if ev.key_in(&["1", "2", "3"]) && !mp {
                self.diff_name = DIFF_ORDER[ev.key.parse::<usize>().unwrap_or(3) - 1];
                self.b.play_sound("select");
            } else if ev.key_in(&["Return", "space"]) {
                self.start_run();
            }
        } else if ev.kind == Kind::MouseDown {
            let pos = ev.pos;
            for i in 0..self.diff_rects.len() {
                let (name, r) = self.diff_rects[i];
                if r.hit(pos) && !mp {
                    self.diff_name = name;
                    self.b.play_sound("select");
                }
            }
            for i in 0..self.size_rects.len() {
                let (n, r) = self.size_rects[i];
                if r.hit(pos) {
                    self.size = n;
                    self.b.play_sound("select");
                }
            }
            if self.start_rect.hit(pos) {
                self.start_run();
            }
        }
    }

    fn start_run(&mut self) {
        self.streak = 0;
        self.n = self.size;
        self.k = win_length(self.n);
        self.max_depth = match self.n {
            3 => 9,
            4 => 4,
            5 => 3,
            _ => 2,
        };
        self.precompute_windows();
        self.neue_runde();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn neue_runde(&mut self) {
        let n = self.n as usize;
        self.board = vec![EMPTY; n * n];
        self.current = HUMAN;
        self.winner = None;
        self.win_cells = None;
        self.ai_timer = 0.0;
        self.b.game_over = false;
        self.b.new_round_result();
        self.layout_board();
    }

    fn layout_board(&mut self) {
        let n = self.n;
        let (w, h) = (self.b.width, self.b.height);
        self.board_size = w.min(h) - 90;
        self.cell = py::fdiv32(self.board_size, n);
        self.board_size = self.cell * n;
        self.ox = py::fdiv32(w - self.board_size, 2);
        self.oy = py::fdiv32(h - self.board_size, 2) + 10;
    }

    fn precompute_windows(&mut self) {
        let (n, k) = (self.n, self.k);
        let idx = |r: i32, c: i32| (r * n + c) as usize;
        self.windows.clear();
        for r in 0..n {
            for c in 0..n {
                if c + k <= n {
                    self.windows.push((0..k).map(|i| idx(r, c + i)).collect());
                }
                if r + k <= n {
                    self.windows.push((0..k).map(|i| idx(r + i, c)).collect());
                }
                if r + k <= n && c + k <= n {
                    self.windows.push((0..k).map(|i| idx(r + i, c + i)).collect());
                }
                if r + k <= n && c - k + 1 >= 0 {
                    self.windows.push((0..k).map(|i| idx(r + i, c - i)).collect());
                }
            }
        }
    }

    fn feld_aus_pos(&self, pos: (i32, i32)) -> Option<usize> {
        let (x, y) = pos;
        if !(self.ox <= x && x < self.ox + self.board_size && self.oy <= y && y < self.oy + self.board_size) {
            return None;
        }
        let col = py::fdiv32(x - self.ox, self.cell);
        let row = py::fdiv32(y - self.oy, self.cell);
        Some((row * self.n + col) as usize)
    }

    fn apply_move(&mut self, idx: usize, sym: u8) {
        self.board[idx] = sym;
        self.b.play_sound("click");
        if let Some(cells) = self.winning_cells(idx, sym) {
            self.winner = Some(Winner::Sym(sym));
            self.win_cells = Some(cells);
            if sym == HUMAN {
                self.wins_x += 1;
            } else {
                self.wins_o += 1;
            }
            if !self.b.multiplayer && sym == HUMAN {
                self.b.score = self.wins_x;
            }
            self.ende();
        } else if !self.board.contains(&EMPTY) {
            self.winner = Some(Winner::Draw);
            self.ende();
        }
    }

    fn ende(&mut self) {
        self.state = State::Over;
        self.b.game_over = true;
        let mp = self.b.multiplayer;
        let sym = match self.winner {
            Some(Winner::Sym(s)) => Some(s),
            _ => None,
        };
        if !mp {
            if let Some(s) = sym {
                self.b.report_result(s == HUMAN);
            }
            self.streak = if sym == Some(HUMAN) { self.streak + 1 } else { 0 };
            self.b.ach_event("ttt_unbeaten", Some(self.streak));
        } else if let Some(s) = sym {
            self.b.report_duel(if s == HUMAN { 0 } else { 1 });
        }
        if self.winner == Some(Winner::Draw) {
            self.b.play_sound("point");
        } else if !mp && sym == Some(AI) {
            self.b.play_sound("gameover");
        } else {
            self.b.play_sound("win");
        }
    }

    fn winning_cells(&self, idx: usize, sym: u8) -> Option<Vec<usize>> {
        let (n, k) = (self.n, self.k);
        let (r, c) = (idx as i32 / n, idx as i32 % n);
        for (dr, dc) in [(0, 1), (1, 0), (1, 1), (1, -1)] {
            let mut cells = vec![idx];
            let (mut rr, mut cc) = (r + dr, c + dc);
            while 0 <= rr && rr < n && 0 <= cc && cc < n && self.board[(rr * n + cc) as usize] == sym {
                cells.push((rr * n + cc) as usize);
                rr += dr;
                cc += dc;
            }
            let (mut rr, mut cc) = (r - dr, c - dc);
            while 0 <= rr && rr < n && 0 <= cc && cc < n && self.board[(rr * n + cc) as usize] == sym {
                cells.push((rr * n + cc) as usize);
                rr -= dr;
                cc -= dc;
            }
            if cells.len() as i32 >= k {
                return Some(cells);
            }
        }
        None
    }

    fn won_at(&self, idx: usize, sym: u8) -> bool {
        self.winning_cells(idx, sym).is_some()
    }

    fn empties(&self) -> Vec<usize> {
        (0..self.board.len()).filter(|&i| self.board[i] == EMPTY).collect()
    }

    fn candidates(&self) -> Vec<usize> {
        let n = self.n;
        let empties = self.empties();
        if n <= 3 {
            return empties;
        }
        let belegt: Vec<usize> = (0..self.board.len()).filter(|&i| self.board[i] != EMPTY).collect();
        if belegt.is_empty() {
            return vec![(n * n / 2) as usize];
        }
        let mut nah = IntSet::new();
        for i in belegt {
            let (r, c) = (i as i32 / n, i as i32 % n);
            for dr in [-1, 0, 1] {
                for dc in [-1, 0, 1] {
                    let (rr, cc) = (r + dr, c + dc);
                    if 0 <= rr && rr < n && 0 <= cc && cc < n {
                        let j = rr * n + cc;
                        if self.board[j as usize] == EMPTY {
                            nah.add(j as i64);
                        }
                    }
                }
            }
        }
        if nah.is_empty() {
            empties
        } else {
            nah.iter().map(|v| v as usize).collect()
        }
    }

    fn find_winning(&mut self, sym: u8) -> Option<usize> {
        for i in self.empties() {
            self.board[i] = sym;
            let gewinnt = self.won_at(i, sym);
            self.board[i] = EMPTY;
            if gewinnt {
                return Some(i);
            }
        }
        None
    }

    fn ai_decide(&mut self) -> Option<usize> {
        let empties = self.empties();
        if empties.is_empty() {
            return None;
        }
        let nn = (self.n * self.n) as usize;
        if empties.len() == nn {
            return Some(nn / 2);
        }
        if let Some(g) = self.find_winning(AI) {
            return Some(g);
        }
        if self.diff_name == "Easy" {
            return Some(py::choice(&empties));
        }
        if let Some(bl) = self.find_winning(HUMAN) {
            return Some(bl);
        }
        if self.diff_name == "Medium" {
            return Some(self.greedy());
        }
        Some(self.search_root())
    }

    fn evaluate(&self) -> i64 {
        let mut score = 0;
        for win in &self.windows {
            let (mut ai, mut hu) = (0usize, 0usize);
            for &i in win {
                match self.board[i] {
                    AI => ai += 1,
                    HUMAN => hu += 1,
                    _ => {}
                }
            }
            if ai > 0 && hu > 0 {
                continue;
            }
            if ai > 0 {
                score += LINE_WEIGHTS[ai.min(LINE_WEIGHTS.len() - 1)];
            } else if hu > 0 {
                score -= LINE_WEIGHTS[hu.min(LINE_WEIGHTS.len() - 1)];
            }
        }
        score
    }

    fn greedy(&mut self) -> usize {
        let (mut best, mut beste) = (-BIG, None);
        for idx in self.candidates() {
            self.board[idx] = AI;
            let val = self.evaluate();
            self.board[idx] = EMPTY;
            if val > best {
                best = val;
                beste = Some(idx);
            }
        }
        beste.unwrap_or_else(|| py::choice(&self.empties()))
    }

    fn search_root(&mut self) -> usize {
        let (mut best, mut beste) = (-BIG, None);
        let (mut alpha, beta) = (-BIG, BIG);
        for idx in self.candidates() {
            self.board[idx] = AI;
            let val = if self.won_at(idx, AI) { WIN_SCORE } else { self.search(self.max_depth - 1, alpha, beta, false) };
            self.board[idx] = EMPTY;
            if val > best {
                best = val;
                beste = Some(idx);
            }
            alpha = alpha.max(best);
        }
        beste.unwrap_or_else(|| py::choice(&self.empties()))
    }

    fn search(&mut self, depth: i32, mut alpha: i64, mut beta: i64, maximizing: bool) -> i64 {
        if depth == 0 || !self.board.contains(&EMPTY) {
            return self.evaluate();
        }
        let cand = self.candidates();
        if cand.is_empty() {
            return self.evaluate();
        }
        let tiefe = (self.max_depth - depth) as i64;
        if maximizing {
            let mut best = -BIG;
            for idx in cand {
                self.board[idx] = AI;
                let val = if self.won_at(idx, AI) { WIN_SCORE - tiefe } else { self.search(depth - 1, alpha, beta, false) };
                self.board[idx] = EMPTY;
                best = best.max(val);
                alpha = alpha.max(best);
                if beta <= alpha {
                    break;
                }
            }
            best
        } else {
            let mut best = BIG;
            for idx in cand {
                self.board[idx] = HUMAN;
                let val = if self.won_at(idx, HUMAN) { -WIN_SCORE + tiefe } else { self.search(depth - 1, alpha, beta, true) };
                self.board[idx] = EMPTY;
                best = best.min(val);
                beta = beta.min(best);
                if beta <= alpha {
                    break;
                }
            }
            best
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let mp = self.b.multiplayer;
        let (msg, farbe) = match self.winner {
            Some(Winner::Draw) | None => (t("common.draw"), p.gold),
            Some(Winner::Sym(sym)) if mp => {
                let i = if sym == HUMAN { 0 } else { 1 };
                let name: String = self.b.pname(i).chars().take(10).collect();
                (tf("common.player_wins_name", &[("name", &name)]), if i == 0 { COL_X } else { COL_O })
            }
            Some(Winner::Sym(HUMAN)) => (t("ttt.win_you"), p.green),
            Some(Winner::Sym(_)) => (t("ttt.win_ai"), p.red),
        };
        let zeilen: [(String, &Font, Option<Color>); 3] = [
            (msg, &self.b.big_font, Some(farbe)),
            (t("ttt.new_round"), &self.b.font, None),
            (t("ttt.settings"), &self.small, Some(p.text_dim)),
        ];
        let bw = zeilen.iter().map(|(txt, f, _)| f.size(txt).0).max().unwrap_or(0) + 80;
        let bh = zeilen.iter().map(|(_, f, _)| f.get_height()).sum::<i32>() + 66;
        let rect = Rect::new(0, 0, bw, bh).center_at((self.b.width / 2, self.b.height / 2));
        let mut panel = pg::surface_alpha(bw, bh);
        draw::rect_r(&mut panel, pg::alpha(p.panel, 235), (0, 0, bw, bh), 0, 16);
        draw::rect_r(&mut panel, farbe, (0, 0, bw, bh), 2, 16);
        s.bl(&panel, (rect.x, rect.y));
        let mut y = rect.y + 20;
        for (txt, f, col) in &zeilen {
            let col = col.unwrap_or_else(|| ui::mix(p.text_dim, p.text, ui::pulse(2.0, 0.2, 1.0)));
            let img = f.render(txt, true, col);
            s.blr(&img, img.get_rect().midtop_at((rect.centerx(), y)));
            y += f.get_height() + 11;
        }
    }
}

impl Game for TicTacToeGame {
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
        self.wins_x = 0;
        self.wins_o = 0;
        self.streak = 0;
        self.b.game_over = false;
        self.state = State::Setup;
        self.diff_name = "Hard";
        self.size = 3;
        self.make_fonts();
        self.build_setup_layout();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.build_setup_layout();
        if self.state != State::Setup {
            self.layout_board();
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup_event(ev);
                return;
            }
            State::Over => {
                if ev.kind == Kind::KeyDown && ev.key_in(&["s", "S"]) {
                    self.state = State::Setup;
                    self.b.play_sound("click");
                } else if ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space"])) {
                    self.neue_runde();
                    self.state = State::Play;
                }
                return;
            }
            State::Play => {}
        }
        if ev.kind != Kind::MouseDown {
            return;
        }
        if self.b.multiplayer {
            if let Some(idx) = self.feld_aus_pos(ev.pos) {
                if self.board[idx] == EMPTY {
                    let sym = self.current;
                    self.apply_move(idx, sym);
                    if self.state == State::Play {
                        self.current = if sym == HUMAN { AI } else { HUMAN };
                    }
                }
            }
        } else if self.current == HUMAN {
            if let Some(idx) = self.feld_aus_pos(ev.pos) {
                if self.board[idx] == EMPTY {
                    self.apply_move(idx, HUMAN);
                    if self.state == State::Play {
                        self.current = AI;
                        self.ai_timer = 0.25;
                    }
                }
            }
        }
    }

    fn update(&mut self, dt: f64) {
        if self.b.multiplayer || self.state != State::Play || self.current != AI {
            return;
        }
        self.ai_timer -= dt;
        if self.ai_timer > 0.0 {
            return;
        }
        if let Some(zug) = self.ai_decide() {
            self.apply_move(zug, AI);
            if self.state == State::Play {
                self.current = HUMAN;
            }
        }
    }

    fn draw(&mut self) {
        if self.state == State::Setup {
            self.draw_setup();
            return;
        }
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let (n, cell) = (self.n, self.cell);
        let lw = 2.max(cell / 20);
        let mitte = |idx: usize| -> (i32, i32) {
            let (r, c) = (idx as i32 / n, idx as i32 % n);
            (self.ox + c * cell + cell / 2, self.oy + r * cell + cell / 2)
        };
        if let Some(cells) = &self.win_cells {
            for &idx in cells {
                let (r, c) = (idx as i32 / n, idx as i32 % n);
                draw::rect(&mut s, COL_WIN_BG, (self.ox + c * cell, self.oy + r * cell, cell, cell), 0);
            }
        }
        for i in 1..n {
            let x = self.ox + i * cell;
            let y = self.oy + i * cell;
            draw::line(&mut s, COL_LINE, (x, self.oy), (x, self.oy + self.board_size), lw);
            draw::line(&mut s, COL_LINE, (self.ox, y), (self.ox + self.board_size, y), lw);
        }
        draw::rect(&mut s, COL_LINE, (self.ox, self.oy, self.board_size, self.board_size), lw);
        let rad = (cell as f64 * 0.30) as i32;
        let mw = 3.max(cell / 12);
        for (i, &sym) in self.board.iter().enumerate() {
            if sym == EMPTY {
                continue;
            }
            let (mx, my) = mitte(i);
            if sym == HUMAN {
                draw::line(&mut s, COL_X, (mx - rad, my - rad), (mx + rad, my + rad), mw);
                draw::line(&mut s, COL_X, (mx + rad, my - rad), (mx - rad, my + rad), mw);
            } else {
                draw::circle(&mut s, COL_O, (mx, my), rad, mw);
            }
        }
        if let Some(cells) = &self.win_cells {
            if cells.len() >= 2 {
                let p1 = mitte(cells[0]);
                let p2 = mitte(cells[cells.len() - 1]);
                draw::line(&mut s, COL_WIN, p1, p2, 4.max(cell / 10));
            }
        }
        let mp = self.b.multiplayer;
        let (kopf, stand) = if mp {
            (
                tf("ttt.header_mp", &[("n", &n.to_string()), ("k", &self.k.to_string())]),
                self.small.render(&tf("ttt.score_mp", &[("x", &self.wins_x.to_string()), ("o", &self.wins_o.to_string())]), true, p.text),
            )
        } else {
            (
                tf("ttt.header_sp", &[("diff", &Self::diff_label(self.diff_name)), ("n", &n.to_string()), ("k", &self.k.to_string())]),
                self.small.render(&tf("ttt.score_sp", &[("wins", &self.wins_x.to_string())]), true, p.text),
            )
        };
        let kimg = self.small.render(&kopf, true, p.text_dim);
        s.bl(&kimg, (10, 8));
        s.bl(&stand, (w - stand.w() - 10, 8));
        if self.state == State::Play {
            let zug = if mp {
                tf("common.player_turn_name", &[("name", &self.b.pname(if self.current == HUMAN { 0 } else { 1 }))])
            } else if self.current == HUMAN {
                t("ttt.turn_you")
            } else {
                t("ttt.turn_ai")
            };
            let zimg = self.small.render(&zug, true, p.text);
            s.bl(&zimg, (10, h - self.small.get_height() - 8));
        }
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "tictactoe",
    class_name: "TicTacToeGame",
    supports_multiplayer: true,
    modes: &[],
    make: TicTacToeGame::new,
};
