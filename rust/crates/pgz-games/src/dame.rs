//! dame.rs
//! =======
//! Dame / Checkers - drei Regelwerke, 1 Spieler gegen KI oder 2 Spieler.
//!
//! Uebertragen aus `games/dame.py`: Deutsche Dame (8x8, fliegende Dame),
//! Internationale Dame (10x10, Maximum-Regel), Checkers (8x8, Maenner
//! schlagen nur vorwaerts). Schlagzwang, Mehrfachschlag Schritt fuer Schritt,
//! KI als Minimax mit Alpha-Beta in drei Staerken.

use pgz_engine::prelude::*;

const EMPTY: u8 = 0;
const P0_MAN: u8 = 1;
const P0_KING: u8 = 2;
const P1_MAN: u8 = 3;
const P1_KING: u8 = 4;
const DIRS4: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

const COL_LIGHT: Color = Color::rgb(222, 196, 150);
const COL_DARK: Color = Color::rgb(120, 84, 52);
const COL_PLATE: Color = Color::rgb(40, 28, 20);
const COL_P0: Color = Color::rgb(236, 230, 214);
const COL_P0_HI: Color = Color::rgb(255, 255, 250);
const COL_P1: Color = Color::rgb(188, 60, 60);
const COL_P1_HI: Color = Color::rgb(232, 120, 120);
const COL_KING: Color = Color::rgb(245, 205, 90);
const COL_HINT: Color = Color::rgb(250, 236, 150);
const COL_CAP: Color = Color::rgb(232, 90, 80);

const VARIANTS: [&str; 3] = ["german", "international", "checkers"];
const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const BIG: i64 = 1_000_000_000_000_000_000;

type Pos = (i32, i32);

fn owner(code: u8) -> Option<usize> {
    match code {
        EMPTY => None,
        c if c <= P0_KING => Some(0),
        _ => Some(1),
    }
}

fn is_king(code: u8) -> bool {
    code == P0_KING || code == P1_KING
}

fn make_code(side: usize, king: bool) -> u8 {
    match (side, king) {
        (0, true) => P0_KING,
        (0, false) => P0_MAN,
        (_, true) => P1_KING,
        _ => P1_MAN,
    }
}

#[derive(Clone, Copy)]
struct Flags {
    size: i32,
    men_back: bool,
    flying: bool,
    maximum: bool,
}

fn variant_flags(v: &str) -> Flags {
    match v {
        "german" => Flags { size: 8, men_back: true, flying: true, maximum: false },
        "international" => Flags { size: 10, men_back: true, flying: true, maximum: true },
        _ => Flags { size: 8, men_back: false, flying: false, maximum: false },
    }
}

#[derive(Clone, PartialEq)]
struct Board {
    n: i32,
    v: Vec<u8>,
}

impl Board {
    fn get(&self, r: i32, c: i32) -> u8 {
        self.v[(r * self.n + c) as usize]
    }
    fn set(&mut self, r: i32, c: i32, x: u8) {
        self.v[(r * self.n + c) as usize] = x;
    }
}

fn initial_board(f: &Flags) -> Board {
    let n = f.size;
    let rows = if n == 8 { 3 } else { 4 };
    let mut b = Board { n, v: vec![EMPTY; (n * n) as usize] };
    for r in 0..n {
        for c in 0..n {
            if (r + c) % 2 != 1 {
                continue;
            }
            if r < rows {
                b.set(r, c, P1_MAN);
            } else if r >= n - rows {
                b.set(r, c, P0_MAN);
            }
        }
    }
    b
}

fn inside(r: i32, c: i32, n: i32) -> bool {
    0 <= r && r < n && 0 <= c && c < n
}

fn back_rank(row: i32, side: usize, n: i32) -> bool {
    if side == 0 {
        row == 0
    } else {
        row == n - 1
    }
}

fn single_jumps(b: &Board, r: i32, c: i32, side: usize, king: bool, captured: &[Pos], f: &Flags) -> Vec<(Pos, Pos)> {
    let n = f.size;
    let opp = 1 - side;
    let mut out = Vec::new();
    let stepwise = |dirs: &[(i32, i32)], out: &mut Vec<(Pos, Pos)>| {
        for &(dr, dc) in dirs {
            let (mr, mc, lr, lc) = (r + dr, c + dc, r + 2 * dr, c + 2 * dc);
            if inside(lr, lc, n) && owner(b.get(mr, mc)) == Some(opp) && !captured.contains(&(mr, mc)) && b.get(lr, lc) == EMPTY {
                out.push(((mr, mc), (lr, lc)));
            }
        }
    };
    if !king {
        let dirs: &[(i32, i32)] = if f.men_back {
            &DIRS4
        } else if side == 0 {
            &[(-1, -1), (-1, 1)]
        } else {
            &[(1, -1), (1, 1)]
        };
        stepwise(dirs, &mut out);
        return out;
    }
    if f.flying {
        for (dr, dc) in DIRS4 {
            let mut i = 1;
            while inside(r + dr * i, c + dc * i, n) && b.get(r + dr * i, c + dc * i) == EMPTY {
                i += 1;
            }
            let (er, ec) = (r + dr * i, c + dc * i);
            if inside(er, ec, n) && owner(b.get(er, ec)) == Some(opp) && !captured.contains(&(er, ec)) {
                let mut j = i + 1;
                while inside(r + dr * j, c + dc * j, n) && b.get(r + dr * j, c + dc * j) == EMPTY {
                    out.push(((er, ec), (r + dr * j, c + dc * j)));
                    j += 1;
                }
            }
        }
    } else {
        stepwise(&DIRS4, &mut out);
    }
    out
}

#[derive(Clone, PartialEq)]
struct Move {
    path: Vec<Pos>,
    caps: Vec<Pos>,
}

impl Move {
    fn start(&self) -> Pos {
        self.path[0]
    }
    fn end(&self) -> Pos {
        self.path[self.path.len() - 1]
    }
}

fn capture_sequences(board: &Board, r: i32, c: i32, f: &Flags) -> Vec<(Vec<Pos>, Vec<Pos>)> {
    let code = board.get(r, c);
    let side = owner(code).unwrap_or(0);
    let king0 = is_king(code);
    let n = f.size;
    let mut results = Vec::new();
    #[allow(clippy::too_many_arguments)]
    fn rec(
        b: &Board,
        cr: i32,
        cc: i32,
        cur_king: bool,
        captured: &[Pos],
        path: &[Pos],
        side: usize,
        n: i32,
        f: &Flags,
        results: &mut Vec<(Vec<Pos>, Vec<Pos>)>,
    ) -> bool {
        let jumps = single_jumps(b, cr, cc, side, cur_king, captured, f);
        if jumps.is_empty() {
            return false;
        }
        for (cap, (lr, lc)) in jumps {
            let mut nb = b.clone();
            nb.set(cr, cc, EMPTY);
            let promoted = !cur_king && back_rank(lr, side, n);
            let nk = cur_king || promoted;
            nb.set(lr, lc, make_code(side, nk));
            let mut ncap = captured.to_vec();
            if !ncap.contains(&cap) {
                ncap.push(cap);
            }
            let mut npath = path.to_vec();
            npath.push((lr, lc));
            if promoted || !rec(&nb, lr, lc, nk, &ncap, &npath, side, n, f, results) {
                results.push((npath, ncap));
            }
        }
        true
    }
    rec(board, r, c, king0, &[], &[(r, c)], side, n, f, &mut results);
    results
}

fn simple_moves(board: &Board, r: i32, c: i32, f: &Flags) -> Vec<(Pos, Pos)> {
    let code = board.get(r, c);
    let side = owner(code).unwrap_or(0);
    let n = f.size;
    let mut out = Vec::new();
    if !is_king(code) {
        let fdir = if side == 0 { -1 } else { 1 };
        for dc in [-1, 1] {
            let (lr, lc) = (r + fdir, c + dc);
            if inside(lr, lc, n) && board.get(lr, lc) == EMPTY {
                out.push(((r, c), (lr, lc)));
            }
        }
    } else if f.flying {
        for (dr, dc) in DIRS4 {
            let mut i = 1;
            while inside(r + dr * i, c + dc * i, n) && board.get(r + dr * i, c + dc * i) == EMPTY {
                out.push(((r, c), (r + dr * i, c + dc * i)));
                i += 1;
            }
        }
    } else {
        for (dr, dc) in DIRS4 {
            let (lr, lc) = (r + dr, c + dc);
            if inside(lr, lc, n) && board.get(lr, lc) == EMPTY {
                out.push(((r, c), (lr, lc)));
            }
        }
    }
    out
}

fn legal_moves(board: &Board, side: usize, f: &Flags) -> (Vec<Move>, bool) {
    let n = f.size;
    let mut caps = Vec::new();
    for r in 0..n {
        for c in 0..n {
            if owner(board.get(r, c)) == Some(side) {
                for (path, capset) in capture_sequences(board, r, c, f) {
                    caps.push(Move { path, caps: capset });
                }
            }
        }
    }
    if !caps.is_empty() {
        if f.maximum {
            let m = caps.iter().map(|x| x.caps.len()).max().unwrap_or(0);
            caps.retain(|x| x.caps.len() == m);
        }
        return (caps, true);
    }
    let mut simples = Vec::new();
    for r in 0..n {
        for c in 0..n {
            if owner(board.get(r, c)) == Some(side) {
                for (st, en) in simple_moves(board, r, c, f) {
                    simples.push(Move { path: vec![st, en], caps: Vec::new() });
                }
            }
        }
    }
    (simples, false)
}

fn apply_move(board: &Board, mv: &Move, f: &Flags) -> Board {
    let mut b = board.clone();
    let (sr, sc) = mv.start();
    let (er, ec) = mv.end();
    let code = b.get(sr, sc);
    let side = owner(code).unwrap_or(0);
    let mut king = is_king(code);
    b.set(sr, sc, EMPTY);
    for &(cr, cc) in &mv.caps {
        b.set(cr, cc, EMPTY);
    }
    if !king && back_rank(er, side, f.size) {
        king = true;
    }
    b.set(er, ec, make_code(side, king));
    b
}

fn count(board: &Board) -> (i64, i64) {
    let (mut a, mut b) = (0, 0);
    for &v in &board.v {
        match owner(v) {
            Some(0) => a += 1,
            Some(_) => b += 1,
            None => {}
        }
    }
    (a, b)
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

/// `DameGame`
pub struct DameGame {
    b: GameBase,
    variant: &'static str,
    diff: usize,
    small: Font,
    tiny: Font,
    huge: Font,
    wins: [i64; 2],
    starter: usize,
    var_rects: Vec<Rect>,
    diff_rects: Vec<Rect>,
    start_rect: Rect,
    flags: Flags,
    depths: [i32; 3],
    board: Board,
    player: usize,
    cursor: [i32; 2],
    moves: Vec<Move>,
    forced: bool,
    sel: Option<Pos>,
    partial: Vec<Pos>,
    step_options: Vec<Pos>,
    last_move: Option<Move>,
    winner: usize,
    ai_delay: f64,
    msg_t: f64,
    state: State,
    hud_h: i32,
    cell: i32,
    bw: i32,
    bh: i32,
    bx: i32,
    by: i32,
    pr: i32,
}

impl DameGame {
    /// `DameGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let flags = variant_flags("german");
        let mut g = DameGame {
            b,
            variant: "german",
            diff: 1,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            wins: [0, 0],
            starter: 0,
            var_rects: Vec::new(),
            diff_rects: Vec::new(),
            start_rect: z,
            flags,
            depths: [1, 3, 4],
            board: initial_board(&flags),
            player: 0,
            cursor: [0, 0],
            moves: Vec::new(),
            forced: false,
            sel: None,
            partial: Vec::new(),
            step_options: Vec::new(),
            last_move: None,
            winner: 0,
            ai_delay: 0.0,
            msg_t: 0.0,
            state: State::Setup,
            hud_h: 46,
            cell: 1,
            bw: 0,
            bh: 0,
            bx: 0,
            by: 0,
            pr: 0,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 32));
        self.tiny = ui::f(12.max(h / 40));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn layout(&mut self) {
        let n = self.flags.size;
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 46;
        self.cell = ((w - 40) as f64 / n as f64).min((h - self.hud_h - 24) as f64 / n as f64) as i32;
        self.bw = n * self.cell;
        self.bh = n * self.cell;
        self.bx = py::fdiv32(w - self.bw, 2);
        self.by = self.hud_h + 8.max(py::fdiv32(h - self.hud_h - self.bh, 2));
        self.pr = (self.cell as f64 * 0.38) as i32;
    }

    fn new_round(&mut self) {
        self.flags = variant_flags(self.variant);
        let n = self.flags.size;
        self.depths = if n == 8 { [1, 3, 4] } else { [1, 2, 3] };
        self.board = initial_board(&self.flags);
        self.player = self.starter;
        let h = n / 2;
        self.cursor = [h, if h % 2 == 0 { h - 1 } else { h }];
        let (m, f) = legal_moves(&self.board, self.player, &self.flags);
        self.moves = m;
        self.forced = f;
        self.sel = None;
        self.partial.clear();
        self.step_options.clear();
        self.last_move = None;
        self.winner = 0;
        self.ai_delay = 0.0;
        self.msg_t = 0.0;
        self.layout();
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 380.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.26) as i32;
        let l = cx - py::fdiv32(bw, 2);
        self.var_rects = (0..3).map(|i| Rect::new(l, y0 + i * 50, bw, 42)).collect();
        let y1 = y0 + 3 * 50 + 18;
        let dw = 120.min(py::fdiv32(bw - 20, 3));
        let gap = py::fdiv32(bw - 3 * dw, 2);
        self.diff_rects = (0..3).map(|i| Rect::new(l + i * (dw + gap), y1, dw, 40)).collect();
        self.start_rect = Rect::new(cx - 95, y1 + 60, 190, 46);
    }

    fn save_diff(&mut self) {
        self.b.set_opt("dame", "difficulty", (self.diff as i64).into());
    }

    fn pick_variant(&mut self, i: usize) {
        self.variant = VARIANTS[i];
        self.b.set_opt("dame", "variant", self.variant.into());
        self.b.play_sound("click");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let mp = self.b.multiplayer;
        let vi = VARIANTS.iter().position(|v| *v == self.variant).unwrap_or(0);
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" => self.pick_variant(k.parse::<usize>().unwrap_or(1) - 1),
                "Up" | "w" | "W" => self.pick_variant((vi + 2) % 3),
                "Down" | "s" | "S" => self.pick_variant((vi + 1) % 3),
                "Left" | "a" | "A" if !mp => {
                    self.diff = (self.diff + 2) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Right" | "d" | "D" if !mp => {
                    self.diff = (self.diff + 1) % 3;
                    self.save_diff();
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.start_game(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.var_rects.iter().position(|r| r.hit(ev.pos)) {
                self.pick_variant(i);
                return;
            }
            if !mp {
                if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                    self.diff = i;
                    self.save_diff();
                    self.b.play_sound("click");
                    return;
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.start_game();
            }
        }
    }

    fn start_game(&mut self) {
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("select");
    }

    fn cell_at(&self, pos: (i32, i32)) -> Option<Pos> {
        let c = py::fdiv32(pos.0 - self.bx, self.cell);
        let r = py::fdiv32(pos.1 - self.by, self.cell);
        if inside(r, c, self.flags.size) {
            Some((r, c))
        } else {
            None
        }
    }

    fn is_start_square(&self, rc: Pos) -> bool {
        self.moves.iter().any(|m| m.start() == rc)
    }

    fn start_squares(&self) -> Vec<Pos> {
        let mut v: Vec<Pos> = Vec::new();
        for m in &self.moves {
            if !v.contains(&m.start()) {
                v.push(m.start());
            }
        }
        v
    }

    fn choose(&mut self, rc: Pos) {
        let Some(sel) = self.sel else {
            if self.is_start_square(rc) {
                self.sel = Some(rc);
                self.partial = vec![rc];
                self.recompute_options();
                self.b.play_sound("select");
            } else {
                self.b.play_sound("click");
            }
            return;
        };
        let depth = self.partial.len();
        let cand: Vec<Move> = self
            .moves
            .iter()
            .filter(|m| m.start() == sel && m.path.len() > depth && m.path[..depth] == self.partial[..] && m.path[depth] == rc)
            .cloned()
            .collect();
        if cand.is_empty() {
            if rc == sel && depth == 1 {
                self.sel = None;
                self.partial.clear();
                self.step_options.clear();
                self.b.play_sound("click");
            } else if self.is_start_square(rc) {
                self.sel = Some(rc);
                self.partial = vec![rc];
                self.recompute_options();
                self.b.play_sound("select");
            } else {
                self.b.play_sound("click");
            }
            return;
        }
        self.partial.push(rc);
        if let Some(mv) = cand.iter().find(|m| m.path.len() == self.partial.len()).cloned() {
            self.do_move(mv);
        } else {
            self.recompute_options();
            self.b.play_sound("lock");
        }
    }

    fn recompute_options(&mut self) {
        let depth = self.partial.len();
        let mut opts = Vec::new();
        for m in &self.moves {
            if Some(m.start()) == self.sel && m.path.len() > depth && m.path[..depth] == self.partial[..] && !opts.contains(&m.path[depth]) {
                opts.push(m.path[depth]);
            }
        }
        self.step_options = opts;
    }

    fn is_promotion(&self, mv: &Move) -> bool {
        let (sr, sc) = mv.start();
        let code = self.board.get(sr, sc);
        if is_king(code) {
            return false;
        }
        back_rank(mv.end().0, owner(code).unwrap_or(0), self.flags.size)
    }

    fn do_move(&mut self, mv: Move) {
        let promoted = self.is_promotion(&mv);
        self.board = apply_move(&self.board, &mv, &self.flags);
        self.sel = None;
        self.partial.clear();
        self.step_options.clear();
        if !mv.caps.is_empty() {
            self.b.play_sound("lock");
            if mv.caps.len() >= 3 {
                self.b.ach_event("dame_multi", None);
            }
        } else {
            self.b.play_sound("move");
        }
        if promoted {
            self.b.play_sound("powerup");
        }
        self.last_move = Some(mv);
        self.advance_turn();
    }

    fn advance_turn(&mut self) {
        let other = 1 - self.player;
        let (moves, forced) = legal_moves(&self.board, other, &self.flags);
        if moves.is_empty() {
            self.ende(self.player);
            return;
        }
        self.player = other;
        self.moves = moves;
        self.forced = forced;
        self.sel = None;
        self.partial.clear();
        self.step_options.clear();
        if !self.b.multiplayer && self.player == 1 {
            self.ai_delay = 0.45;
        }
    }

    fn restart(&mut self) {
        self.starter = 1 - self.starter;
        self.b.game_over = false;
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn ende(&mut self, winner: usize) {
        self.winner = winner;
        self.wins[winner] += 1;
        self.state = State::Over;
        if !self.b.multiplayer {
            if winner == 0 {
                self.b.score = self.wins[0];
                self.b.play_sound("win");
                self.b.report_result(true);
            } else {
                self.b.play_sound("gameover");
                self.b.report_result(false);
            }
        } else {
            self.b.report_duel(winner);
            self.b.play_sound("win");
        }
        self.b.game_over = true;
    }

    fn ai_play(&mut self) {
        let Some(mv) = self.ai_choose() else {
            self.ende(0);
            return;
        };
        let promoted = self.is_promotion(&mv);
        self.board = apply_move(&self.board, &mv, &self.flags);
        if !mv.caps.is_empty() {
            self.b.play_sound("lock");
        } else {
            self.b.play_sound("move");
        }
        if promoted {
            self.b.play_sound("powerup");
        }
        self.last_move = Some(mv);
        self.advance_turn();
    }

    fn ai_choose(&self) -> Option<Move> {
        if self.moves.is_empty() {
            return None;
        }
        if self.diff == 0 && py::random() < 0.55 {
            return Some(py::choice(&self.moves));
        }
        let depth = self.depths[self.diff];
        let mut best = -BIG;
        let mut best_moves: Vec<Move> = Vec::new();
        for m in &self.moves {
            let nb = apply_move(&self.board, m, &self.flags);
            let val = self.search(&nb, 0, depth - 1, -BIG, BIG);
            if val > best {
                best = val;
                best_moves = vec![m.clone()];
            } else if val == best {
                best_moves.push(m.clone());
            }
        }
        if self.diff == 1 && self.moves.len() > 1 && py::random() < 0.2 {
            return Some(py::choice(&self.moves));
        }
        Some(py::choice(&best_moves))
    }

    fn search(&self, board: &Board, side: usize, depth: i32, mut alpha: i64, mut beta: i64) -> i64 {
        let (moves, _) = legal_moves(board, side, &self.flags);
        if moves.is_empty() {
            return if side == 1 { -1_000_000_000 } else { 1_000_000_000 };
        }
        if depth <= 0 {
            return self.evaluate(board);
        }
        if side == 1 {
            let mut best = -BIG;
            for m in &moves {
                let nb = apply_move(board, m, &self.flags);
                best = best.max(self.search(&nb, 0, depth - 1, alpha, beta));
                alpha = alpha.max(best);
                if alpha >= beta {
                    break;
                }
            }
            return best;
        }
        let mut best = BIG;
        for m in &moves {
            let nb = apply_move(board, m, &self.flags);
            best = best.min(self.search(&nb, 1, depth - 1, alpha, beta));
            beta = beta.min(best);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    fn evaluate(&self, board: &Board) -> i64 {
        let n = self.flags.size;
        let mut score = 0i64;
        for r in 0..n {
            for c in 0..n {
                let code = board.get(r, c);
                if code == EMPTY {
                    continue;
                }
                let o = owner(code).unwrap_or(0);
                let mut val: i64 = if is_king(code) { 300 } else { 100 };
                if !is_king(code) {
                    val += if o == 1 { (r * 4) as i64 } else { ((n - 1 - r) * 4) as i64 };
                }
                if c == 0 || c == n - 1 {
                    val += 6;
                }
                score += if o == 1 { val } else { -val };
            }
        }
        score
    }

    fn center_of(&self, r: i32, c: i32) -> (i32, i32) {
        (self.bx + c * self.cell + self.cell / 2, self.by + r * self.cell + self.cell / 2)
    }

    fn draw_board(&self, s: &mut Surface) {
        let n = self.flags.size;
        let cell = self.cell;
        let plate = Rect::new(self.bx - 8, self.by - 8, self.bw + 16, self.bh + 16);
        draw::rect_r(s, COL_PLATE, plate, 0, 10);
        draw::rect_r(s, ui::mix(COL_PLATE, self.b.accent, 0.45), plate, 1, 10);
        for r in 0..n {
            for c in 0..n {
                let dark = (r + c) % 2 == 1;
                draw::rect(s, if dark { COL_DARK } else { COL_LIGHT }, (self.bx + c * cell, self.by + r * cell, cell, cell), 0);
            }
        }
        let human_turn = self.state == State::Play && (self.b.multiplayer || self.player == 0);
        if let Some(lm) = &self.last_move {
            for &(cr, cc) in &lm.caps {
                let (x, y) = (self.bx + cc * cell, self.by + cr * cell);
                draw::line(s, COL_CAP, (x + 6, y + 6), (x + cell - 6, y + cell - 6), 2);
                draw::line(s, COL_CAP, (x + cell - 6, y + 6), (x + 6, y + cell - 6), 2);
            }
            let last_col = ui::mix(COL_DARK, self.b.accent, 0.55 + 0.35 * ui::pulse(1.6, 0.0, 1.0));
            for pos in [lm.start(), lm.end()] {
                draw::rect(s, last_col, (self.bx + pos.1 * cell, self.by + pos.0 * cell, cell, cell), 2);
            }
        }
        if human_turn && self.sel.is_none() {
            for (sr, sc) in self.start_squares() {
                let col = if self.forced { COL_CAP } else { COL_HINT };
                draw::circle(s, col, self.center_of(sr, sc), self.pr + 3, 2);
            }
        }
        if human_turn && self.sel.is_some() {
            let cur = self.partial[self.partial.len() - 1];
            draw::rect(s, COL_HINT, (self.bx + cur.1 * cell, self.by + cur.0 * cell, cell, cell), 3);
            for &(dr, dc) in &self.step_options {
                draw::circle(s, COL_HINT, self.center_of(dr, dc), 4.max(self.pr / 3), 0);
            }
        }
        for r in 0..n {
            for c in 0..n {
                let v = self.board.get(r, c);
                if v != EMPTY {
                    self.draw_piece(s, r, c, v);
                }
            }
        }
        if human_turn {
            let [r, c] = self.cursor;
            let k = ui::pulse(2.2, 0.0, 1.0);
            let g = (150.0 + 100.0 * k) as i32;
            draw::rect(s, rgb(g, g, g), (self.bx + c * cell + 1, self.by + r * cell + 1, cell - 2, cell - 2), 2);
        }
    }

    fn draw_piece(&self, s: &mut Surface, r: i32, c: i32, v: u8) {
        let (cx, cy) = self.center_of(r, c);
        let o = owner(v).unwrap_or(0);
        let base = if o == 0 { COL_P0 } else { COL_P1 };
        let hi = if o == 0 { COL_P0_HI } else { COL_P1_HI };
        let dark = rgb((base.r as f64 * 0.55) as i32, (base.g as f64 * 0.55) as i32, (base.b as f64 * 0.55) as i32);
        let pr = self.pr;
        draw::circle(s, rgb(10, 8, 6), (cx, cy + 2), pr, 0);
        draw::circle(s, dark, (cx, cy), pr, 0);
        draw::circle(s, base, (cx, cy), (pr as f64 * 0.86) as i32, 0);
        draw::circle(s, hi, (cx - pr / 3, cy - pr / 3), 2.max(pr / 5), 0);
        if is_king(v) {
            draw::circle(s, COL_KING, (cx, cy), (pr as f64 * 0.5) as i32, 2);
            let kr = (pr as f64 * 0.34) as i32;
            let pts = [
                (cx - kr, cy + kr / 2),
                (cx - kr, cy - kr / 2),
                (cx - kr / 2, cy),
                (cx, cy - kr),
                (cx + kr / 2, cy),
                (cx + kr, cy - kr / 2),
                (cx + kr, cy + kr / 2),
            ];
            draw::polygon(s, COL_KING, &pts, 0);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let panel = Rect::new(8, 6, self.b.width - 16, self.hud_h - 10);
        ui::draw_panel_ex(s, panel, None, None, None, false, Some(self.b.accent));
        let cy = panel.centery();
        let (a, b) = count(&self.board);
        draw::circle(s, COL_P0, (panel.x + 18, cy), 9, 0);
        draw::circle(s, p.border_light, (panel.x + 18, cy), 9, 1);
        let img = self.small.render(&a.to_string(), true, p.text);
        s.blr(&img, img.get_rect().midleft_at((panel.x + 34, cy)));
        draw::circle(s, COL_P1, (panel.right() - 18, cy), 9, 0);
        draw::circle(s, p.border_light, (panel.right() - 18, cy), 9, 1);
        let img = self.small.render(&b.to_string(), true, p.text);
        s.blr(&img, img.get_rect().midright_at((panel.right() - 34, cy)));
        if self.state == State::Play {
            let mp = self.b.multiplayer;
            let mid = if !mp && self.player == 1 {
                t("dame.ai_thinks")
            } else if !mp {
                t("dame.your_turn")
            } else {
                let name: String = self.b.pname(self.player).chars().take(8).collect();
                tf("dame.turn", &[("name", &name)])
            };
            let img = self.small.render(&mid, true, self.b.accent);
            s.blr(&img, img.get_rect().center_at((self.b.width / 2, cy)));
            if self.forced {
                let fi = self.tiny.render(&t("dame.must_capture"), true, p.red);
                s.blr(&fi, fi.get_rect().center_at((self.b.width / 2, self.hud_h + 11)));
            }
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let head = if self.b.multiplayer {
            let name: String = self.b.pname(self.winner).chars().take(10).collect();
            self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, if self.winner == 0 { COL_P0 } else { COL_P1 })
        } else {
            let key = if self.winner == 0 { "dame.win_you" } else { "dame.win_ai" };
            self.huge.render(&t(key), true, if self.winner == 0 { self.b.accent } else { p.text_dim })
        };
        let hint = self.tiny.render(&t("dame.new_round"), true, p.text_dim);
        let w = (self.b.width - 24).min(head.w().max(hint.w()) + 64);
        let panel = Rect::new(cx - py::fdiv32(w, 2), self.b.height / 2 - 48, w, 96);
        ui::draw_panel_ex(s, panel, None, None, None, false, Some(self.b.accent));
        s.blr(&head, head.get_rect().center_at((cx, panel.y + 36)));
        s.blr(&hint, hint.get_rect().center_at((cx, panel.y + 74)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        ui::draw_title_ex(
            s,
            w,
            &t("dame.title"),
            Some(&t("dame.variant")),
            (h as f64 * 0.13) as i32,
            Some(&self.huge),
            Some(&self.small),
            Some(self.b.accent),
        );
        for (i, rc) in self.var_rects.iter().enumerate() {
            ui::draw_button_ex(s, *rc, &t(&format!("dame.var.{}", VARIANTS[i])), &self.small, VARIANTS[i] == self.variant, None, None, Some(self.b.accent));
        }
        if !self.b.multiplayer {
            let dl = self.tiny.render(&t("dame.difficulty"), true, p.text_dim);
            s.blr(&dl, dl.get_rect().midbottom_at((cx, self.diff_rects[0].y - 4)));
            for (i, rc) in self.diff_rects.iter().enumerate() {
                ui::draw_button_ex(s, *rc, &t(&format!("dame.diff.{}", DIFFS[i])), &self.tiny, i == self.diff, None, None, Some(self.b.accent));
            }
        }
        ui::draw_button_ex(s, self.start_rect, &t("common.start"), &self.small, true, None, None, Some(self.b.accent));
        ui::draw_footer_ex(s, w, h, &t("dame.setup_hint"), Some(&self.tiny));
    }
}

impl Game for DameGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let v = self.b.opt_str("dame", "variant", "german");
        self.variant = VARIANTS.iter().copied().find(|x| *x == v).unwrap_or("german");
        self.diff = self.b.opt_i64("dame", "difficulty", 1).clamp(0, 2) as usize;
        self.make_fonts();
        self.wins = [0, 0];
        self.starter = 0;
        self.build_setup_layout();
        self.new_round();
        self.state = State::Setup;
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
                    } else if ev.key_in(&["s", "S"]) {
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
                if let Some(rc) = self.cell_at(ev.pos) {
                    self.cursor = [rc.0, rc.1];
                }
            }
            Kind::MouseDown => {
                if let Some(rc) = self.cell_at(ev.pos) {
                    self.cursor = [rc.0, rc.1];
                    self.choose(rc);
                }
            }
            Kind::KeyDown => {
                let n = self.flags.size;
                let k = ev.key.as_str();
                if self.b.is_action(k, "up", None) || k == "Up" {
                    self.cursor[0] = py::pmod32(self.cursor[0] - 1, n);
                } else if self.b.is_action(k, "down", None) || k == "Down" {
                    self.cursor[0] = py::pmod32(self.cursor[0] + 1, n);
                } else if self.b.is_action(k, "left", None) || k == "Left" {
                    self.cursor[1] = py::pmod32(self.cursor[1] - 1, n);
                } else if self.b.is_action(k, "right", None) || k == "Right" {
                    self.cursor[1] = py::pmod32(self.cursor[1] + 1, n);
                } else if self.b.is_action(k, "action", None) || matches!(k, "space" | "Return") {
                    self.choose((self.cursor[0], self.cursor[1]));
                    return;
                } else {
                    return;
                }
                self.b.play_sound("move");
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
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
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
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
    key: "dame",
    class_name: "DameGame",
    supports_multiplayer: true,
    modes: &[],
    make: DameGame::new,
};
