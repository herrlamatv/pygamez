//! connect4.rs
//! ===========
//! Vier gewinnt - 1 Spieler gegen KI oder 2 Spieler lokal.
//!
//! Uebertragen aus `games/connect4.py`: 7x6-Brett mit fallendem Stein,
//! KI in drei Staerken (Minimax mit Alpha-Beta und Fenster-Bewertung,
//! "easy" patzt absichtlich), Siege der Sitzung als Punkte.

use pgz_engine::prelude::*;

const COL_PLATE: Color = Color::rgb(38, 58, 140);
const COL_PLATE_EDGE: Color = Color::rgb(26, 40, 100);
const COL_EMPTY: Color = Color::rgb(18, 22, 36);
const COL_P1: Color = Color::rgb(230, 90, 80);
const COL_P2: Color = Color::rgb(245, 205, 90);

const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const DEPTHS: [i32; 3] = [2, 4, 5];

const COLS: usize = 7;
const ROWS: usize = 6;
const ORDER: [usize; 7] = [3, 2, 4, 1, 5, 0, 6];
const INF: i64 = 1_000_000_000;

type Board = [[i32; COLS]; ROWS];

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Anim,
    Over,
}

#[derive(Clone, Copy)]
struct Anim {
    col: usize,
    row: usize,
    y: f64,
    vy: f64,
    player: usize,
}

fn drop_row(board: &Board, col: usize) -> Option<usize> {
    (0..ROWS).rev().find(|&row| board[row][col] == 0)
}

fn check_win(board: &Board, row: usize, col: usize) -> Option<Vec<(usize, usize)>> {
    let p = board[row][col];
    for (dr, dc) in [(0i32, 1i32), (1, 0), (1, 1), (1, -1)] {
        let mut cells = vec![(row, col)];
        for sgn in [1i32, -1] {
            let (mut rr, mut cc) = (row as i32 + dr * sgn, col as i32 + dc * sgn);
            while (0..ROWS as i32).contains(&rr) && (0..COLS as i32).contains(&cc) && board[rr as usize][cc as usize] == p {
                cells.push((rr as usize, cc as usize));
                rr += dr * sgn;
                cc += dc * sgn;
            }
        }
        if cells.len() >= 4 {
            return Some(cells);
        }
    }
    None
}

fn is_full(board: &Board) -> bool {
    (0..COLS).all(|c| board[0][c] != 0)
}

fn valid_cols(board: &Board) -> Vec<usize> {
    ORDER.iter().copied().filter(|&c| board[0][c] == 0).collect()
}

fn evaluate(board: &Board) -> i64 {
    let mut score = 0i64;
    for row in board.iter() {
        if row[3] == 2 {
            score += 6;
        }
    }
    let mut windows: Vec<[i32; 4]> = Vec::with_capacity(69);
    for r in 0..ROWS {
        for c in 0..COLS - 3 {
            windows.push([0, 1, 2, 3].map(|i| board[r][c + i]));
        }
    }
    for c in 0..COLS {
        for r in 0..ROWS - 3 {
            windows.push([0, 1, 2, 3].map(|i| board[r + i][c]));
        }
    }
    for r in 0..ROWS - 3 {
        for c in 0..COLS - 3 {
            windows.push([0, 1, 2, 3].map(|i| board[r + i][c + i]));
            windows.push([0, 1, 2, 3].map(|i| board[r + 3 - i][c + i]));
        }
    }
    let vals = |n: usize| -> Option<i64> {
        match n {
            2 => Some(10),
            3 => Some(120),
            _ => None,
        }
    };
    for w in &windows {
        let ai = w.iter().filter(|&&v| v == 2).count();
        let hu = w.iter().filter(|&&v| v == 1).count();
        if let (0, Some(v)) = (hu, vals(ai)) {
            score += v;
        } else if let (0, Some(v)) = (ai, vals(hu)) {
            score -= (v as f64 * 1.2) as i64;
        }
    }
    score
}

fn minimax(board: &mut Board, depth: i32, mut alpha: i64, mut beta: i64, maximizing: bool) -> i64 {
    let valid = valid_cols(board);
    if depth == 0 || valid.is_empty() {
        return evaluate(board);
    }
    if maximizing {
        let mut best = -INF;
        for c in valid {
            let r = drop_row(board, c).unwrap();
            board[r][c] = 2;
            let val = if check_win(board, r, c).is_some() {
                100000 + depth as i64
            } else {
                minimax(board, depth - 1, alpha, beta, false)
            };
            board[r][c] = 0;
            best = best.max(val);
            alpha = alpha.max(best);
            if alpha >= beta {
                break;
            }
        }
        return best;
    }
    let mut best = INF;
    for c in valid {
        let r = drop_row(board, c).unwrap();
        board[r][c] = 1;
        let val = if check_win(board, r, c).is_some() {
            -100000 - depth as i64
        } else {
            minimax(board, depth - 1, alpha, beta, true)
        };
        board[r][c] = 0;
        best = best.min(val);
        beta = beta.min(best);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// `ConnectFourGame`
pub struct ConnectFourGame {
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
    hover: usize,
    anim: Option<Anim>,
    win_cells: Option<Vec<(usize, usize)>>,
    winner: Option<usize>,
    msg: Option<String>,
    msg_t: f64,
    ai_delay: f64,
    state: State,
    hud_h: i32,
    cell: i32,
    bw: i32,
    bh: i32,
    bx: i32,
    by: i32,
    r: i32,
}

impl ConnectFourGame {
    /// `ConnectFourGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = ConnectFourGame {
            b,
            diff: 1,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            wins: [0, 0],
            starter: 0,
            diff_rects: Vec::new(),
            start_rect: Rect::new(0, 0, 0, 0),
            board: [[0; COLS]; ROWS],
            player: 0,
            hover: 3,
            anim: None,
            win_cells: None,
            winner: None,
            msg: None,
            msg_t: 0.0,
            ai_delay: 0.0,
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
        self.cell = ((w - 40) as f64 / 7.0).min((h - self.hud_h - 20) as f64 / 7.0) as i32;
        self.bw = 7 * self.cell;
        self.bh = 6 * self.cell;
        self.bx = py::fdiv32(w - self.bw, 2);
        self.by = self.hud_h + self.cell;
        self.r = (self.cell as f64 * 0.42) as i32;
    }

    fn new_round(&mut self) {
        self.board = [[0; COLS]; ROWS];
        self.player = self.starter;
        self.hover = 3;
        self.anim = None;
        self.win_cells = None;
        self.winner = None;
        self.msg = None;
        self.msg_t = 0.0;
        self.ai_delay = 0.0;
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
        self.b.set_opt("connect4", "difficulty", (self.diff as i64).into());
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
                "Return" | "space" => {
                    self.new_round();
                    self.state = State::Play;
                    self.b.play_sound("click");
                }
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
                self.new_round();
                self.state = State::Play;
                self.b.play_sound("click");
            }
        }
    }

    fn try_drop(&mut self, col: usize) {
        let Some(row) = drop_row(&self.board, col) else {
            self.msg = Some(t("c4.col_full"));
            self.msg_t = 1.2;
            self.b.play_sound("click");
            return;
        };
        self.hover = col;
        self.anim = Some(Anim { col, row, y: (self.by - self.cell) as f64, vy: 0.0, player: self.player });
        self.state = State::Anim;
        self.b.play_sound("move");
    }

    fn ende(&mut self, cells: Option<Vec<(usize, usize)>>, winner: Option<usize>) {
        self.state = State::Over;
        if let Some(w) = winner {
            if let Some(cs) = &cells {
                let mp = self.b.multiplayer;
                let mut rs: Vec<usize> = cs.iter().map(|c| c.0).collect();
                let mut cl: Vec<usize> = cs.iter().map(|c| c.1).collect();
                rs.sort();
                rs.dedup();
                cl.sort();
                cl.dedup();
                if (mp || w == 0) && rs.len() > 1 && cl.len() > 1 {
                    self.b.ach_event("c4_diagonal", None);
                }
            }
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
                self.b.report_duel(w);
                self.b.play_sound("win");
            }
        }
        self.win_cells = cells;
        self.winner = winner;
        self.b.game_over = true;
    }

    fn ai_move(&mut self) -> usize {
        let mut board = self.board;
        let valid = valid_cols(&board);
        if valid.is_empty() {
            return 3;
        }
        for &c in &valid {
            let r = drop_row(&board, c).unwrap();
            board[r][c] = 2;
            let won = check_win(&board, r, c).is_some();
            board[r][c] = 0;
            if won {
                return c;
            }
        }
        for &c in &valid {
            let r = drop_row(&board, c).unwrap();
            board[r][c] = 1;
            let lose = check_win(&board, r, c).is_some();
            board[r][c] = 0;
            if lose && !(self.diff == 0 && py::random() < 0.4) {
                return c;
            }
        }
        let depth = DEPTHS[self.diff];
        let mut scores: Vec<(i64, usize)> = Vec::new();
        for &c in &valid {
            let r = drop_row(&board, c).unwrap();
            board[r][c] = 2;
            if check_win(&board, r, c).is_some() {
                return c;
            }
            let val = minimax(&mut board, depth - 1, -INF, INF, false);
            board[r][c] = 0;
            scores.push((val, c));
        }
        scores.sort_by(|a, b| b.cmp(a));
        if self.diff == 0 && scores.len() > 1 && py::random() < 0.3 {
            return scores[1].1;
        }
        scores[0].1
    }

    fn disc_color(p: usize) -> Color {
        if p == 0 {
            COL_P1
        } else {
            COL_P2
        }
    }

    fn draw_board(&self, s: &mut Surface) {
        let (cell, r) = (self.cell, self.r);
        if self.state == State::Play && (self.b.multiplayer || self.player == 0) {
            let hx = self.bx + self.hover as i32 * cell + cell / 2;
            let hy = self.by - cell / 2;
            draw::circle(s, Self::disc_color(self.player), (hx, hy), r, 0);
            draw::circle(s, COL_PLATE_EDGE, (hx, hy), r, 2);
        }
        let plate = Rect::new(self.bx - 8, self.by - 8, self.bw + 16, self.bh + 16);
        draw::rect_r(s, COL_PLATE, plate, 0, 12);
        draw::rect_r(s, COL_PLATE_EDGE, plate, 3, 12);
        for row in 0..ROWS {
            for c in 0..COLS {
                let cx = self.bx + c as i32 * cell + cell / 2;
                let cy = self.by + row as i32 * cell + cell / 2;
                let v = self.board[row][c];
                let col = if v == 0 { COL_EMPTY } else { Self::disc_color((v - 1) as usize) };
                draw::circle(s, col, (cx, cy), r, 0);
                draw::circle(s, COL_PLATE_EDGE, (cx, cy), r, 2);
            }
        }
        if let Some(a) = self.anim {
            let prev = s.get_clip();
            s.set_clip(Some(Rect::new(self.bx, self.by - cell, self.bw, self.bh + cell)));
            let cx = self.bx + a.col as i32 * cell + cell / 2;
            let cy = a.y as i32 + cell / 2;
            draw::circle(s, Self::disc_color(a.player), (cx, cy), r, 0);
            s.set_clip(Some(prev));
        }
        if let Some(cells) = &self.win_cells {
            if !cells.is_empty() {
                let k = 0.5 + 0.5 * ((pg::ticks() % 1000) as i64 - 500).abs() as f64 / 500.0;
                for &(rr, c) in cells {
                    let cx = self.bx + c as i32 * cell + cell / 2;
                    let cy = self.by + rr as i32 * cell + cell / 2;
                    draw::circle(s, Color::rgb(255, 255, 255), (cx, cy), r + 2, 2.max((4.0 * k) as i32));
                }
            }
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let mp = self.b.multiplayer;
        let n1: String = if mp { self.b.pname(0).chars().take(8).collect() } else { t("c4.score_sp") };
        let n2: String = if mp { self.b.pname(1).chars().take(8).collect() } else { t("common.ai") };
        let img = self.small.render(&format!("{n1}: {}", self.wins[0]), true, COL_P1);
        s.blr(&img, img.get_rect().midleft_at((12, cy)));
        let img = self.small.render(&format!("{} :{n2}", self.wins[1]), true, COL_P2);
        s.blr(&img, img.get_rect().midright_at((w - 12, cy)));
        if matches!(self.state, State::Play | State::Anim) {
            let mid = if !mp && self.player == 1 {
                t("c4.ai_thinks")
            } else {
                let who: String = self.b.pname(self.player).chars().take(8).collect();
                tf("c4.turn", &[("name", &who)])
            };
            let img = self.small.render(&mid, true, Self::disc_color(self.player));
            s.blr(&img, img.get_rect().center_at((w / 2, cy)));
        }
        if let Some(m) = &self.msg {
            if !m.is_empty() {
                let img = self.tiny.render(m, true, p.gold);
                s.blr(&img, img.get_rect().center_at((w / 2, self.hud_h + 12)));
            }
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let hh = self.huge.get_height();
        let sh = self.small.get_height();
        let band_h = hh + sh + 36;
        let y = self.b.height / 2 - band_h / 2;
        let mut ov = pg::surface_alpha(w, band_h);
        ov.fill_c(pg::alpha(p.panel, 235));
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 2);
        draw::line(s, self.b.accent, (0, y + band_h - 1), (w, y + band_h - 1), 2);
        let cx = w / 2;
        let head = match self.winner {
            None => self.huge.render(&t("common.draw"), true, p.text_dim),
            Some(wn) if self.b.multiplayer => {
                let name: String = self.b.pname(wn).chars().take(10).collect();
                self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, Self::disc_color(wn))
            }
            Some(wn) => {
                let key = if wn == 0 { "c4.win_you" } else { "c4.win_ai" };
                self.huge.render(&t(key), true, Self::disc_color(wn))
            }
        };
        s.blr(&head, head.get_rect().midtop_at((cx, y + 12)));
        let hint_key = if self.b.multiplayer { "common.enter_restart" } else { "c4.new_round" };
        let hint = self.small.render(&t(hint_key), true, p.text_dim);
        s.blr(&hint, hint.get_rect().midtop_at((cx, y + 12 + hh + 10)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let name = pgz_core::achievements::game_name(self.b.key).to_uppercase();
        let title = self.huge.render(&name, true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.14) as i32)));
        let sub = self.small.render(&t("c4.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.21) as i32)));
        for (i, r) in self.diff_rects.iter().enumerate() {
            let on = i == self.diff;
            draw::rect_r(s, if on { p.btn_sel } else { p.btn }, *r, 0, 10);
            draw::rect_r(s, if on { self.b.accent } else { p.border }, *r, if on { 2 } else { 1 }, 10);
            let lbl = self.b.font.render(&t(&format!("c4.diff.{}", DIFFS[i])), true, if on { p.text } else { p.text_dim });
            s.blr(&lbl, lbl.get_rect().midleft_at((r.x + 18, r.centery())));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("c4.setup_hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 16)));
    }

    fn restart_round(&mut self) {
        self.starter = 1 - self.starter;
        self.b.game_over = false;
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }
}

impl Game for ConnectFourGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        self.diff = self.b.opt_i64("connect4", "difficulty", 1).clamp(0, 2) as usize;
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
                        self.restart_round();
                    } else if ev.key_in(&["s", "S"]) && !self.b.multiplayer {
                        self.b.game_over = false;
                        self.state = State::Setup;
                        self.b.play_sound("click");
                    }
                } else if ev.kind == Kind::MouseDown {
                    self.restart_round();
                }
                return;
            }
            State::Anim => return,
            State::Play => {}
        }
        if !self.b.multiplayer && self.player == 1 {
            return;
        }
        match ev.kind {
            Kind::MouseMove => {
                let col = py::fdiv32(ev.pos.0 - self.bx, self.cell);
                if (0..COLS as i32).contains(&col) {
                    self.hover = col as usize;
                }
            }
            Kind::MouseDown => {
                let col = py::fdiv32(ev.pos.0 - self.bx, self.cell);
                if (0..COLS as i32).contains(&col) {
                    self.try_drop(col as usize);
                }
            }
            Kind::KeyDown => {
                let k = ev.key.as_str();
                if self.b.is_action(k, "left", None) || k == "Left" {
                    self.hover = (self.hover + COLS - 1) % COLS;
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "right", None) || k == "Right" {
                    self.hover = (self.hover + 1) % COLS;
                    self.b.play_sound("move");
                } else if self.b.is_action(k, "down", None) || matches!(k, "Down" | "space" | "Return") {
                    self.try_drop(self.hover);
                } else if "1234567".contains(k) {
                    if let Ok(n) = k.parse::<usize>() {
                        if (1..=COLS).contains(&n) {
                            self.try_drop(n - 1);
                        }
                    }
                } else if let Some(rest) = k.strip_prefix("KP_") {
                    if "1234567".contains(rest) {
                        if let Ok(n) = rest.parse::<usize>() {
                            if (1..=COLS).contains(&n) {
                                self.try_drop(n - 1);
                            }
                        }
                    }
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
        if self.state == State::Anim {
            if let Some(mut a) = self.anim {
                a.vy += 2600.0 * dt;
                a.y += a.vy * dt;
                self.anim = Some(a);
                let target = (self.by + a.row as i32 * self.cell) as f64;
                if a.y >= target {
                    self.board[a.row][a.col] = a.player as i32 + 1;
                    self.b.play_sound("lock");
                    self.anim = None;
                    if let Some(cells) = check_win(&self.board, a.row, a.col) {
                        self.ende(Some(cells), Some(a.player));
                    } else if is_full(&self.board) {
                        self.ende(None, None);
                    } else {
                        self.player = 1 - self.player;
                        self.state = State::Play;
                        self.ai_delay = 0.35;
                    }
                }
                return;
            }
        }
        if self.state == State::Play && !self.b.multiplayer && self.player == 1 {
            self.ai_delay -= dt;
            if self.ai_delay <= 0.0 {
                let c = self.ai_move();
                self.try_drop(c);
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
    key: "connect4",
    class_name: "ConnectFourGame",
    supports_multiplayer: true,
    modes: &[],
    make: ConnectFourGame::new,
};
