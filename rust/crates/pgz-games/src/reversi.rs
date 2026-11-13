//! reversi.rs
//! ==========
//! Reversi (Othello) - 1 Spieler gegen KI oder 2 Spieler lokal.
//!
//! Uebertragen aus `games/reversi.py`: Standard-Startstellung, Passen und
//! Spielende, KI in drei Staerken (Negamax mit Alpha-Beta, Positionsgewichte
//! plus Mobilitaet; easy/medium patzen absichtlich).

use pgz_engine::prelude::*;

const COL_BOARD: Color = Color::rgb(28, 92, 58);
const COL_BOARD_DARK: Color = Color::rgb(22, 74, 46);
const COL_GRID: Color = Color::rgb(16, 54, 34);
const COL_PLATE: Color = Color::rgb(18, 40, 28);
const COL_P1: Color = Color::rgb(30, 33, 42);
const COL_P1_HI: Color = Color::rgb(78, 84, 100);
const COL_P2: Color = Color::rgb(238, 240, 246);
const COL_P2_HI: Color = Color::rgb(255, 255, 255);
const COL_HINT: Color = Color::rgb(90, 200, 150);

const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const DEPTHS: [i32; 3] = [1, 3, 4];
const N: usize = 8;
const DIRS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
const WEIGHTS: [[i64; 8]; 8] = [
    [120, -20, 20, 5, 5, 20, -20, 120],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [20, -5, 15, 3, 3, 15, -5, 20],
    [5, -5, 3, 3, 3, 3, -5, 5],
    [5, -5, 3, 3, 3, 3, -5, 5],
    [20, -5, 15, 3, 3, 15, -5, 20],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [120, -20, 20, 5, 5, 20, -20, 120],
];
const BIG: i64 = 1_000_000_000_000_000_000;

type Board = [[u8; N]; N];
type Pos = (usize, usize);
type Moves = Vec<(Pos, Vec<Pos>)>;

fn flips_for(board: &Board, r: usize, c: usize, pv: u8) -> Vec<Pos> {
    if board[r][c] != 0 {
        return Vec::new();
    }
    let opp = 3 - pv;
    let mut flips = Vec::new();
    let inside = |rr: i32, cc: i32| (0..N as i32).contains(&rr) && (0..N as i32).contains(&cc);
    for (dr, dc) in DIRS {
        let mut line = Vec::new();
        let (mut rr, mut cc) = (r as i32 + dr, c as i32 + dc);
        while inside(rr, cc) && board[rr as usize][cc as usize] == opp {
            line.push((rr as usize, cc as usize));
            rr += dr;
            cc += dc;
        }
        if !line.is_empty() && inside(rr, cc) && board[rr as usize][cc as usize] == pv {
            flips.extend(line);
        }
    }
    flips
}

fn legal_moves(board: &Board, pv: u8) -> Moves {
    let mut moves = Vec::new();
    for r in 0..N {
        for c in 0..N {
            if board[r][c] == 0 {
                let f = flips_for(board, r, c, pv);
                if !f.is_empty() {
                    moves.push(((r, c), f));
                }
            }
        }
    }
    moves
}

fn get_flips(moves: &Moves, p: Pos) -> Option<&Vec<Pos>> {
    moves.iter().find(|m| m.0 == p).map(|m| &m.1)
}

fn apply(board: &mut Board, r: usize, c: usize, pv: u8, flips: &[Pos]) {
    board[r][c] = pv;
    for &(fr, fc) in flips {
        board[fr][fc] = pv;
    }
}

fn count(board: &Board) -> (i64, i64) {
    let (mut a, mut b) = (0, 0);
    for row in board {
        for &v in row {
            if v == 1 {
                a += 1;
            } else if v == 2 {
                b += 1;
            }
        }
    }
    (a, b)
}

fn terminal(board: &Board) -> i64 {
    let (a, b) = count(board);
    if b > a {
        return 100000 + (b - a);
    }
    if a > b {
        return -100000 - (a - b);
    }
    0
}

fn evaluate(board: &Board) -> i64 {
    let mut score = 0;
    for r in 0..N {
        for c in 0..N {
            match board[r][c] {
                2 => score += WEIGHTS[r][c],
                1 => score -= WEIGHTS[r][c],
                _ => {}
            }
        }
    }
    let my = legal_moves(board, 2).len() as i64;
    let op = legal_moves(board, 1).len() as i64;
    if my + op != 0 {
        score += ((8 * (my - op)) as f64 / (my + op + 1) as f64 * 10.0) as i64;
    }
    score
}

fn negamax(board: &Board, depth: i32, mut alpha: i64, mut beta: i64, turn: u8) -> i64 {
    let pv = if turn == 0 { 2 } else { 1 };
    let moves = legal_moves(board, pv);
    if depth == 0 {
        return evaluate(board);
    }
    if moves.is_empty() {
        if legal_moves(board, 3 - pv).is_empty() {
            return terminal(board);
        }
        return negamax(board, depth - 1, alpha, beta, 1 - turn);
    }
    let mut ordered: Vec<&(Pos, Vec<Pos>)> = moves.iter().collect();
    ordered.sort_by(|a, b| WEIGHTS[b.0 .0][b.0 .1].cmp(&WEIGHTS[a.0 .0][a.0 .1]));
    if turn == 0 {
        let mut best = -BIG;
        for ((r, c), fl) in ordered {
            let mut nb = *board;
            apply(&mut nb, *r, *c, pv, fl);
            best = best.max(negamax(&nb, depth - 1, alpha, beta, 1));
            alpha = alpha.max(best);
            if alpha >= beta {
                break;
            }
        }
        best
    } else {
        let mut best = BIG;
        for ((r, c), fl) in ordered {
            let mut nb = *board;
            apply(&mut nb, *r, *c, pv, fl);
            best = best.min(negamax(&nb, depth - 1, alpha, beta, 0));
            beta = beta.min(best);
            if alpha >= beta {
                break;
            }
        }
        best
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

/// `ReversiGame`
pub struct ReversiGame {
    b: GameBase,
    diff: usize,
    small: Font,
    tiny: Font,
    huge: Font,
    wins: [i64; 2],
    starter: usize,
    diff_rects: Vec<Rect>,
    start_rect: Rect,
    board: Board,
    player: usize,
    cursor: [i32; 2],
    moves: Moves,
    winner: Option<usize>,
    msg: Option<String>,
    msg_t: f64,
    ai_delay: f64,
    last_move: Option<Pos>,
    state: State,
    hud_h: i32,
    cell: i32,
    bw: i32,
    bh: i32,
    bx: i32,
    by: i32,
    r: i32,
}

impl ReversiGame {
    /// `ReversiGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = ReversiGame {
            b,
            diff: 1,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            wins: [0, 0],
            starter: 0,
            diff_rects: Vec::new(),
            start_rect: Rect::new(0, 0, 0, 0),
            board: [[0; N]; N],
            player: 0,
            cursor: [4, 4],
            moves: Vec::new(),
            winner: None,
            msg: None,
            msg_t: 0.0,
            ai_delay: 0.0,
            last_move: None,
            state: State::Setup,
            hud_h: 40,
            cell: 1,
            bw: 0,
            bh: 0,
            bx: 0,
            by: 0,
            r: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(13.max(22.min(h / 30)));
        self.tiny = ui::f(11.max(18.min(h / 38)));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 40.max((h as f64 * 0.085) as i32);
        self.cell = ((w - 40) as f64 / N as f64).min((h - self.hud_h - 24) as f64 / N as f64) as i32;
        self.bw = N as i32 * self.cell;
        self.bh = N as i32 * self.cell;
        self.bx = py::fdiv32(w - self.bw, 2);
        self.by = self.hud_h + 8.max(py::fdiv32(h - self.hud_h - self.bh, 2));
        self.r = (self.cell as f64 * 0.42) as i32;
    }

    fn new_round(&mut self) {
        self.board = [[0; N]; N];
        let mid = N / 2;
        self.board[mid - 1][mid - 1] = 2;
        self.board[mid][mid] = 2;
        self.board[mid - 1][mid] = 1;
        self.board[mid][mid - 1] = 1;
        self.player = self.starter;
        self.cursor = [mid as i32, mid as i32];
        self.moves = legal_moves(&self.board, self.player as u8 + 1);
        self.winner = None;
        self.msg = None;
        self.msg_t = 0.0;
        self.ai_delay = 0.0;
        self.last_move = None;
        self.layout();
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 360.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.32) as i32;
        self.diff_rects = (0..3).map(|i| Rect::new(cx - py::fdiv32(bw, 2), y0 + i * 58, bw, 48)).collect();
        self.start_rect = Rect::new(cx - 95, y0 + 3 * 58 + 14, 190, 46);
    }

    fn save_diff(&mut self) {
        self.b.set_opt("reversi", "difficulty", (self.diff as i64).into());
    }

    fn start_from_setup(&mut self) {
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" => {
                    self.diff = k.parse::<usize>().unwrap_or(2) - 1;
                    self.save_diff();
                    self.b.play_sound("click");
                }
                "Up" | "w" | "W" => {
                    self.diff = (self.diff + 2) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.diff = (self.diff + 1) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.start_from_setup(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                self.diff = i;
                self.save_diff();
                self.b.play_sound("click");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_from_setup();
            }
        }
    }

    fn cell_at(&self, pos: (i32, i32)) -> Option<Pos> {
        let c = py::fdiv32(pos.0 - self.bx, self.cell);
        let r = py::fdiv32(pos.1 - self.by, self.cell);
        if (0..N as i32).contains(&r) && (0..N as i32).contains(&c) {
            Some((r as usize, c as usize))
        } else {
            None
        }
    }

    fn restart(&mut self) {
        self.starter = 1 - self.starter;
        self.b.game_over = false;
        self.b.new_round_result();
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn try_place(&mut self, r: usize, c: usize) {
        let Some(flips) = get_flips(&self.moves, (r, c)).filter(|f| !f.is_empty()).cloned() else {
            self.msg = Some(t("rev.illegal"));
            self.msg_t = 1.0;
            self.b.play_sound("click");
            return;
        };
        apply(&mut self.board, r, c, self.player as u8 + 1, &flips);
        self.last_move = Some((r, c));
        self.b.play_sound("lock");
        self.advance_turn();
    }

    fn advance_turn(&mut self) {
        let other = 1 - self.player;
        let other_moves = legal_moves(&self.board, other as u8 + 1);
        if !other_moves.is_empty() {
            self.player = other;
            self.moves = other_moves;
            self.ai_delay = 0.35;
            return;
        }
        let self_moves = legal_moves(&self.board, self.player as u8 + 1);
        if !self_moves.is_empty() {
            self.moves = self_moves;
            let who = if other == 1 { self.b.pname(1) } else { self.b.pname(0) };
            self.msg = Some(tf("rev.pass", &[("name", &who)]));
            self.msg_t = 1.4;
            self.b.play_sound("select");
            self.ai_delay = 0.35;
            return;
        }
        self.ende();
    }

    fn ende(&mut self) {
        let (a, b) = count(&self.board);
        self.winner = if a > b {
            Some(0)
        } else if b > a {
            Some(1)
        } else {
            None
        };
        self.state = State::Over;
        if let Some(w) = self.winner {
            self.wins[w] += 1;
            if !self.b.multiplayer {
                if w == 0 {
                    self.b.score = self.wins[0];
                    self.b.play_sound("win");
                    self.b.report_result(true);
                } else {
                    self.b.play_sound("gameover");
                    self.b.report_result(false);
                }
            } else {
                self.b.play_sound("win");
                self.b.report_duel(w);
            }
            if a.min(b) == 0 && (self.b.multiplayer || w == 0) {
                self.b.ach_event("reversi_sweep", None);
            }
        } else {
            self.b.play_sound("select");
        }
        self.b.game_over = true;
    }

    fn ai_play(&mut self) {
        let Some((r, c)) = self.ai_move() else {
            self.advance_turn();
            return;
        };
        let flips = match get_flips(&self.moves, (r, c)) {
            Some(f) if !f.is_empty() => f.clone(),
            _ => flips_for(&self.board, r, c, 2),
        };
        apply(&mut self.board, r, c, 2, &flips);
        self.last_move = Some((r, c));
        self.b.play_sound("lock");
        self.advance_turn();
    }

    fn ai_move(&self) -> Option<Pos> {
        let moves: Vec<Pos> = self.moves.iter().map(|m| m.0).collect();
        if moves.is_empty() {
            return None;
        }
        if self.diff == 0 && py::random() < 0.6 {
            return Some(py::choice(&moves));
        }
        let depth = DEPTHS[self.diff];
        let mut best_val = -BIG;
        let mut best: Vec<Pos> = Vec::new();
        for &(r, c) in &moves {
            let mut nb = self.board;
            let fl = flips_for(&nb, r, c, 2);
            apply(&mut nb, r, c, 2, &fl);
            let val = negamax(&nb, depth - 1, -BIG, BIG, 1);
            if val > best_val {
                best_val = val;
                best = vec![(r, c)];
            } else if val == best_val {
                best.push((r, c));
            }
        }
        if self.diff == 1 && moves.len() > 1 && py::random() < 0.25 {
            return Some(py::choice(&moves));
        }
        Some(py::choice(&best))
    }

    fn disc_color(p: usize) -> Color {
        if p == 0 {
            COL_P1
        } else {
            COL_P2
        }
    }

    fn disc_hi(p: usize) -> Color {
        if p == 0 {
            COL_P1_HI
        } else {
            COL_P2_HI
        }
    }

    fn center_of(&self, r: usize, c: usize) -> (i32, i32) {
        (self.bx + c as i32 * self.cell + self.cell / 2, self.by + r as i32 * self.cell + self.cell / 2)
    }

    fn draw_board(&self, s: &mut Surface) {
        let plate = Rect::new(self.bx - 8, self.by - 8, self.bw + 16, self.bh + 16);
        draw::rect_r(s, COL_PLATE, plate, 0, 10);
        for r in 0..N {
            for c in 0..N {
                let x = self.bx + c as i32 * self.cell;
                let y = self.by + r as i32 * self.cell;
                let base = if (r + c) % 2 == 0 { COL_BOARD } else { COL_BOARD_DARK };
                draw::rect(s, base, (x, y, self.cell, self.cell), 0);
            }
        }
        for i in 0..=N as i32 {
            let gx = self.bx + i * self.cell;
            let gy = self.by + i * self.cell;
            draw::line(s, COL_GRID, (gx, self.by), (gx, self.by + self.bh), 1);
            draw::line(s, COL_GRID, (self.bx, gy), (self.bx + self.bw, gy), 1);
        }
        let human_turn = self.state == State::Play && (self.b.multiplayer || self.player == 0);
        if human_turn {
            for ((r, c), _) in &self.moves {
                draw::circle(s, COL_HINT, self.center_of(*r, *c), 3.max(self.r / 5), 0);
            }
        }
        for r in 0..N {
            for c in 0..N {
                let v = self.board[r][c];
                if v == 0 {
                    continue;
                }
                let (cx, cy) = self.center_of(r, c);
                draw::circle(s, rgb(8, 14, 10), (cx, cy + 2), self.r, 0);
                draw::circle(s, Self::disc_color(v as usize - 1), (cx, cy), self.r, 0);
                draw::circle(s, Self::disc_hi(v as usize - 1), (cx - self.r / 3, cy - self.r / 3), 2.max(self.r / 4), 0);
            }
        }
        if let Some((r, c)) = self.last_move {
            let x = self.bx + c as i32 * self.cell;
            let y = self.by + r as i32 * self.cell;
            draw::rect(s, self.b.accent, (x, y, self.cell, self.cell), 2);
        }
        if human_turn {
            let [r, c] = self.cursor;
            let x = self.bx + c * self.cell;
            let y = self.by + r * self.cell;
            let k = 0.5 + 0.5 * ((pg::ticks() % 900) as i64 - 450).abs() as f64 / 450.0;
            let g = (120.0 + 120.0 * k) as i32;
            draw::rect(s, rgb(g, g, g), (x + 1, y + 1, self.cell - 2, self.cell - 2), 2);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let (a, b) = count(&self.board);
        draw::circle(s, COL_P1, (18, cy), 9, 0);
        draw::circle(s, COL_P1_HI, (15, cy - 3), 3, 0);
        draw::circle(s, p.border_light, (18, cy), 9, 1);
        let img = self.small.render(&a.to_string(), true, p.text);
        s.blr(&img, img.get_rect().midleft_at((32, cy)));
        draw::circle(s, COL_P2, (w - 18, cy), 9, 0);
        draw::circle(s, p.border_light, (w - 18, cy), 9, 1);
        let img = self.small.render(&b.to_string(), true, p.text);
        s.blr(&img, img.get_rect().midright_at((w - 32, cy)));
        if self.state == State::Play {
            let mp = self.b.multiplayer;
            let mid = if !mp && self.player == 1 {
                t("rev.ai_thinks")
            } else if !mp {
                t("rev.your_turn")
            } else {
                let who = if self.player == 0 { self.b.pname(0) } else { self.b.pname(1) };
                tf("rev.turn", &[("name", &who)])
            };
            let img = self.small.render(&mid, true, self.b.accent);
            s.blr(&img, img.get_rect().center_at((w / 2, cy)));
        }
        if let Some(m) = self.msg.as_ref().filter(|m| !m.is_empty()) {
            let img = self.tiny.render(m, true, p.gold);
            s.blr(&img, img.get_rect().center_at((w / 2, self.hud_h + 12)));
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let hh = self.huge.get_height();
        let sh = self.small.get_height();
        let th = self.tiny.get_height();
        let band_h = hh + sh + th + 40;
        let y = self.b.height / 2 - band_h / 2;
        let mut ov = pg::surface_alpha(w, band_h);
        ov.fill_c(pg::alpha(p.panel, 235));
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 2);
        draw::line(s, self.b.accent, (0, y + band_h - 1), (w, y + band_h - 1), 2);
        let cx = w / 2;
        let (a, b) = count(&self.board);
        let mut yy = y + 12;
        match self.winner {
            None => {
                let head = self.huge.render(&t("common.draw"), true, p.text_dim);
                s.blr(&head, head.get_rect().midtop_at((cx, yy)));
            }
            Some(wn) if self.b.multiplayer => {
                let name: String = self.b.pname(wn).chars().take(10).collect();
                let head = self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, p.text);
                let ri = 8.max(hh / 3);
                let x0 = cx - py::fdiv32(head.w() + 2 * ri + 14, 2);
                let cyc = yy + hh / 2;
                draw::circle(s, Self::disc_color(wn), (x0 + ri, cyc), ri, 0);
                draw::circle(s, Self::disc_hi(wn), (x0 + ri - ri / 3, cyc - ri / 3), 2.max(ri / 4), 0);
                draw::circle(s, p.border_light, (x0 + ri, cyc), ri, 1);
                s.blr(&head, head.get_rect().midleft_at((x0 + 2 * ri + 14, cyc)));
            }
            Some(wn) => {
                let key = if wn == 0 { "rev.win_you" } else { "rev.win_ai" };
                let head = self.huge.render(&t(key), true, if wn == 0 { self.b.accent } else { p.red });
                s.blr(&head, head.get_rect().midtop_at((cx, yy)));
            }
        }
        yy += hh + 8;
        let sub = self.small.render(&format!("{a} : {b}"), true, p.text);
        s.blr(&sub, sub.get_rect().midtop_at((cx, yy)));
        yy += sh + 6;
        let hint_key = if self.b.multiplayer { "common.enter_restart" } else { "rev.new_round" };
        let hint = self.tiny.render(&t(hint_key), true, p.text_dim);
        s.blr(&hint, hint.get_rect().midtop_at((cx, yy)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let name = pgz_core::achievements::game_name(self.b.key).to_uppercase();
        let title = self.huge.render(&name, true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.14) as i32)));
        let sub = self.small.render(&t("rev.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.21) as i32)));
        for (i, rc) in self.diff_rects.iter().enumerate() {
            let on = i == self.diff;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, *rc, 0, 10);
            draw::rect_r(s, if on { self.b.accent } else { p.border }, *rc, if on { 2 } else { 1 }, 10);
            let lbl = self.b.font.render(&t(&format!("rev.diff.{}", DIFFS[i])), true, if on { p.text } else { p.text_dim });
            s.blr(&lbl, lbl.get_rect().midleft_at((rc.x + 18, rc.centery())));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("rev.setup_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 16)));
    }
}

impl Game for ReversiGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.diff = self.b.opt_i64("reversi", "difficulty", 1).clamp(0, 2) as usize;
        self.make_fonts();
        self.wins = [0, 0];
        self.starter = 0;
        self.build_setup_layout();
        self.new_round();
        self.state = if self.b.multiplayer { State::Play } else { State::Setup };
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
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
                        self.restart();
                    } else if ev.key_in(&["s", "S"]) && !self.b.multiplayer {
                        self.b.game_over = false;
                        self.state = State::Setup;
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown {
                    self.restart();
                }
                return;
            }
            State::Play => {}
        }
        if !self.b.multiplayer && self.player == 1 {
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                if let Some((r, c)) = self.cell_at(ev.pos) {
                    self.cursor = [r as i32, c as i32];
                }
            }
            Kind::MouseDown => {
                if let Some((r, c)) = self.cell_at(ev.pos) {
                    self.cursor = [r as i32, c as i32];
                    self.try_place(r, c);
                }
            }
            Kind::KeyDown => {
                let k = ev.key.as_str();
                let n = N as i32;
                if self.b.is_action(k, "up", None) || k == "Up" {
                    self.cursor[0] = py::pmod32(self.cursor[0] - 1, n);
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "down", None) || k == "Down" {
                    self.cursor[0] = py::pmod32(self.cursor[0] + 1, n);
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "left", None) || k == "Left" {
                    self.cursor[1] = py::pmod32(self.cursor[1] - 1, n);
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "right", None) || k == "Right" {
                    self.cursor[1] = py::pmod32(self.cursor[1] + 1, n);
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "action", None) || matches!(k, "space" | "Return") {
                    self.try_place(self.cursor[0] as usize, self.cursor[1] as usize);
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.state == State::Play && !self.b.multiplayer && self.player == 1 {
            self.ai_delay -= dt;
            if self.ai_delay <= 0.0 {
                self.ai_play();
            }
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
        self.draw_board(&mut s);
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "reversi",
    class_name: "ReversiGame",
    supports_multiplayer: true,
    modes: &[],
    make: ReversiGame::new,
};
