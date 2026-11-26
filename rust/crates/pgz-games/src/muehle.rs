//! muehle.rs
//! =========
//! Muehle (Nine Men's Morris) - 1 Spieler gegen KI oder 2 Spieler lokal.
//!
//! Uebertragen aus `games/muehle.py`: Setzen, Ziehen, Springen (abschaltbar),
//! Muehle -> Stein entfernen (bevorzugt ausserhalb gegnerischer Muehlen),
//! KI als Negamax mit Alpha-Beta, Knoten- und Zeitbudget. Die Zeit kommt aus
//! `py::time_time` - im Pruefstand die feste Bild-Uhr wie in Python.

use pgz_engine::prelude::*;

const COL_PLATE: Color = Color::rgb(24, 38, 31);
const COL_LINE: Color = Color::rgb(120, 168, 140);
const COL_SPOT: Color = Color::rgb(46, 74, 60);
const COL_P1: Color = Color::rgb(238, 240, 246);
const COL_P1_HI: Color = Color::rgb(255, 255, 255);
const COL_P2: Color = Color::rgb(44, 52, 66);
const COL_P2_HI: Color = Color::rgb(96, 108, 130);
const COL_SEL: Color = Color::rgb(246, 214, 92);
const COL_HINT: Color = Color::rgb(120, 210, 150);
const COL_MILL: Color = Color::rgb(240, 180, 80);
const COL_REMOVE: Color = Color::rgb(224, 96, 96);

const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const DEPTHS: [i32; 3] = [1, 2, 4];
const TIME_BUDGET: [f64; 3] = [0.2, 0.5, 1.0];
const NODE_BUDGET: u64 = 300_000;
const INF: i64 = 1_000_000_000;

const POS: [(i32, i32); 24] = [
    (0, 0),
    (3, 0),
    (6, 0),
    (1, 1),
    (3, 1),
    (5, 1),
    (2, 2),
    (3, 2),
    (4, 2),
    (0, 3),
    (1, 3),
    (2, 3),
    (4, 3),
    (5, 3),
    (6, 3),
    (2, 4),
    (3, 4),
    (4, 4),
    (1, 5),
    (3, 5),
    (5, 5),
    (0, 6),
    (3, 6),
    (6, 6),
];

const ADJ: [&[usize]; 24] = [
    &[1, 9],
    &[0, 2, 4],
    &[1, 14],
    &[4, 10],
    &[1, 3, 5, 7],
    &[4, 13],
    &[7, 11],
    &[4, 6, 8],
    &[7, 12],
    &[0, 10, 21],
    &[3, 9, 11, 18],
    &[6, 10, 15],
    &[8, 13, 17],
    &[5, 12, 14, 20],
    &[2, 13, 23],
    &[11, 16],
    &[15, 17, 19],
    &[12, 16],
    &[10, 19],
    &[16, 18, 20, 22],
    &[13, 19],
    &[9, 22],
    &[19, 21, 23],
    &[14, 22],
];

const MILLS: [[usize; 3]; 16] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [9, 10, 11],
    [12, 13, 14],
    [15, 16, 17],
    [18, 19, 20],
    [21, 22, 23],
    [0, 9, 21],
    [3, 10, 18],
    [6, 11, 15],
    [1, 4, 7],
    [16, 19, 22],
    [8, 12, 17],
    [5, 13, 20],
    [2, 14, 23],
];

type Board = [u8; 24];

fn mills_at(p: usize) -> impl Iterator<Item = &'static [usize; 3]> {
    MILLS.iter().filter(move |m| m.contains(&p))
}

fn forms_mill(board: &Board, point: usize, val: u8) -> bool {
    mills_at(point).any(|m| m.iter().all(|&q| board[q] == val))
}

fn double_mill(board: &Board, val: u8) -> bool {
    (0..24).any(|p| board[p] == val && mills_at(p).filter(|m| m.iter().all(|&q| board[q] == val)).count() >= 2)
}

fn removable(board: &Board, opp: u8) -> Vec<usize> {
    let mut in_mill = [false; 24];
    for m in &MILLS {
        if m.iter().all(|&q| board[q] == opp) {
            for &q in m {
                in_mill[q] = true;
            }
        }
    }
    let opp_pts: Vec<usize> = (0..24).filter(|&p| board[p] == opp).collect();
    let free: Vec<usize> = opp_pts.iter().copied().filter(|p| !in_mill[*p]).collect();
    if free.is_empty() {
        opp_pts
    } else {
        free
    }
}

fn count(board: &Board, val: u8) -> usize {
    board.iter().filter(|&&v| v == val).count()
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Mv {
    place: bool,
    a: usize,
    b: usize,
    rp: Option<usize>,
}

fn gen_moves(board: &mut Board, placed: &[u32; 2], player: usize, flying: bool) -> Vec<Mv> {
    let val = player as u8 + 1;
    let opp = if val == 1 { 2 } else { 1 };
    let mut moves = Vec::new();
    if placed[player] < 9 {
        for p in 0..24 {
            if board[p] == 0 {
                board[p] = val;
                if forms_mill(board, p, val) {
                    for rp in removable(board, opp) {
                        moves.push(Mv { place: true, a: p, b: 0, rp: Some(rp) });
                    }
                } else {
                    moves.push(Mv { place: true, a: p, b: 0, rp: None });
                }
                board[p] = 0;
            }
        }
    } else {
        let fly = flying && count(board, val) == 3;
        for p in 0..24 {
            if board[p] != val {
                continue;
            }
            let dests: Vec<usize> = if fly { (0..24).collect() } else { ADJ[p].to_vec() };
            for d in dests {
                if board[d] == 0 {
                    board[p] = 0;
                    board[d] = val;
                    if forms_mill(board, d, val) {
                        for rp in removable(board, opp) {
                            moves.push(Mv { place: false, a: p, b: d, rp: Some(rp) });
                        }
                    } else {
                        moves.push(Mv { place: false, a: p, b: d, rp: None });
                    }
                    board[d] = 0;
                    board[p] = val;
                }
            }
        }
    }
    moves
}

fn apply_move(board: &Board, placed: &[u32; 2], player: usize, mv: &Mv) -> (Board, [u32; 2]) {
    let val = player as u8 + 1;
    let mut nb = *board;
    let mut npl = *placed;
    if mv.place {
        nb[mv.a] = val;
        npl[player] += 1;
    } else {
        nb[mv.a] = 0;
        nb[mv.b] = val;
    }
    if let Some(rp) = mv.rp {
        nb[rp] = 0;
    }
    (nb, npl)
}

fn has_moves(board: &Board, placed: &[u32; 2], player: usize, flying: bool) -> bool {
    if placed[player] < 9 {
        return true;
    }
    let val = player as u8 + 1;
    if flying && count(board, val) == 3 {
        return true;
    }
    (0..24).any(|p| board[p] == val && ADJ[p].iter().any(|&d| board[d] == 0))
}

fn eval_side(board: &Board, val: u8, placed: &[u32; 2], player: usize, flying: bool) -> i64 {
    let (mut mills, mut twos) = (0, 0);
    for m in &MILLS {
        let cnt = m.iter().filter(|&&q| board[q] == val).count();
        let empty = m.iter().filter(|&&q| board[q] == 0).count();
        if cnt == 3 {
            mills += 1;
        } else if cnt == 2 && empty == 1 {
            twos += 1;
        }
    }
    let mut mob = 0;
    let on = count(board, val) as i64;
    if placed[player] >= 9 {
        if flying && on == 3 {
            mob = 10;
        } else {
            for p in 0..24 {
                if board[p] == val {
                    mob += ADJ[p].iter().filter(|&&d| board[d] == 0).count() as i64;
                }
            }
        }
    }
    on * 9 + mills * 6 + twos * 2 + mob
}

fn evaluate(board: &Board, placed: &[u32; 2], player: usize, flying: bool) -> i64 {
    let opp = 1 - player;
    eval_side(board, player as u8 + 1, placed, player, flying) - eval_side(board, opp as u8 + 1, placed, opp, flying)
}

fn order(moves: &[Mv]) -> Vec<Mv> {
    let mut v = moves.to_vec();
    v.sort_by_key(|m| if m.rp.is_some() { 0 } else { 1 });
    v
}

struct Suche {
    nodes: u64,
    deadline: f64,
    flying: bool,
}

impl Suche {
    fn search(&mut self, board: &Board, placed: &[u32; 2], player: usize, depth: i32, mut alpha: i64, beta: i64) -> i64 {
        self.nodes += 1;
        if self.nodes > NODE_BUDGET || ((self.nodes & 1023) == 0 && py::time_time() > self.deadline) {
            return evaluate(board, placed, player, self.flying);
        }
        if placed[0] >= 9 && placed[1] >= 9 && count(board, player as u8 + 1) < 3 {
            return -20000 + (5 - depth) as i64;
        }
        let mut b = *board;
        let moves = gen_moves(&mut b, placed, player, self.flying);
        if moves.is_empty() {
            return -20000 + (5 - depth) as i64;
        }
        if depth <= 0 {
            return evaluate(board, placed, player, self.flying);
        }
        let mut best = -INF;
        for mv in order(&moves) {
            let (nb, npl) = apply_move(board, placed, player, &mv);
            let val = -self.search(&nb, &npl, 1 - player, depth - 1, -beta, -alpha);
            if val > best {
                best = val;
            }
            if best > alpha {
                alpha = best;
            }
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

/// `MuehleGame`
pub struct MuehleGame {
    b: GameBase,
    diff: usize,
    flying: bool,
    small: Font,
    tiny: Font,
    huge: Font,
    wins: [i64; 2],
    starter: usize,
    diff_rects: Vec<Rect>,
    fly_rect: Rect,
    start_rect: Rect,
    board: Board,
    placed: [u32; 2],
    player: usize,
    remove_mode: bool,
    removable: Vec<usize>,
    sel: Option<usize>,
    targets: Vec<usize>,
    last_spot: Option<usize>,
    mill_spots: Vec<[usize; 3]>,
    mill_t: f64,
    winner: usize,
    msg_t: f64,
    ai_delay: f64,
    state: State,
    hud_h: i32,
    pr: i32,
    pts: Vec<(i32, i32)>,
    plate_rect: Rect,
}

impl MuehleGame {
    /// `MuehleGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = MuehleGame {
            b,
            diff: 1,
            flying: true,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            wins: [0, 0],
            starter: 0,
            diff_rects: Vec::new(),
            fly_rect: z,
            start_rect: z,
            board: [0; 24],
            placed: [0, 0],
            player: 0,
            remove_mode: false,
            removable: Vec::new(),
            sel: None,
            targets: Vec::new(),
            last_spot: None,
            mill_spots: Vec::new(),
            mill_t: 0.0,
            winner: 0,
            msg_t: 0.0,
            ai_delay: 0.6,
            state: State::Setup,
            hud_h: 46,
            pr: 9,
            pts: Vec::new(),
            plate_rect: z,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 34));
        self.tiny = ui::f(12.max(h / 44));
        self.huge = ui::fb(26.max(h / 12));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 46;
        let size = (w - 60).min(h - self.hud_h - 40);
        let bx = py::fdiv32(w - size, 2);
        let by = self.hud_h + 10.max(py::fdiv32(h - self.hud_h - size, 2));
        let step = size as f64 / 6.0;
        self.pr = 9.max((step * 0.28) as i32);
        self.pts = POS.iter().map(|(c, r)| ((bx as f64 + *c as f64 * step) as i32, (by as f64 + *r as f64 * step) as i32)).collect();
        let pad = 10.max(((step * 0.5) as i32).min(bx - 8).min(by - self.hud_h - 8));
        self.plate_rect = Rect::new(bx - pad, by - pad, size + 2 * pad, size + 2 * pad);
    }

    fn new_round(&mut self) {
        self.b.new_round_result();
        self.board = [0; 24];
        self.placed = [0, 0];
        self.player = self.starter;
        self.remove_mode = false;
        self.removable.clear();
        self.sel = None;
        self.targets.clear();
        self.last_spot = None;
        self.mill_spots.clear();
        self.mill_t = 0.0;
        self.winner = 0;
        self.msg_t = 0.0;
        self.ai_delay = 0.6;
        self.layout();
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 360.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.30) as i32;
        let l = cx - py::fdiv32(bw, 2);
        self.diff_rects = (0..3).map(|i| Rect::new(l, y0 + i * 52, bw, 44)).collect();
        self.fly_rect = Rect::new(l, y0 + 3 * 52 + 8, bw, 44);
        self.start_rect = Rect::new(cx - 95, y0 + 4 * 52 + 22, 190, 46);
    }

    fn save_diff(&mut self) {
        self.b.set_opt("muehle", "difficulty", (self.diff as i64).into());
    }

    fn toggle_flying(&mut self) {
        self.flying = !self.flying;
        self.b.set_opt("muehle", "flying", self.flying.into());
        self.b.play_sound("select");
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
                "f" | "F" => self.toggle_flying(),
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                self.diff = i;
                self.save_diff();
                self.b.play_sound("click");
                return;
            }
            if self.fly_rect.hit(ev.pos) {
                self.toggle_flying();
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn start_play(&mut self) {
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn human_turn(&self) -> bool {
        self.b.multiplayer || self.player == 0
    }

    fn spot_at(&self, pos: (i32, i32)) -> Option<usize> {
        let r = (self.pr + 6) as i64;
        self.pts.iter().position(|&(x, y)| {
            let (dx, dy) = ((pos.0 - x) as i64, (pos.1 - y) as i64);
            dx * dx + dy * dy <= r * r
        })
    }

    fn click_spot(&mut self, p: usize) {
        let val = self.player as u8 + 1;
        let opp = if val == 1 { 2 } else { 1 };
        if self.remove_mode {
            if self.board[p] == opp && self.removable.contains(&p) {
                self.board[p] = 0;
                self.last_spot = Some(p);
                self.b.play_sound("hit");
                self.remove_mode = false;
                self.removable.clear();
                self.end_turn();
            } else {
                self.b.play_sound("click");
            }
            return;
        }
        if self.placed[self.player] < 9 {
            if self.board[p] == 0 {
                self.board[p] = val;
                self.placed[self.player] += 1;
                self.last_spot = Some(p);
                self.after_action(p, val, opp);
            } else {
                self.b.play_sound("click");
            }
            return;
        }
        if self.board[p] == val {
            self.sel = Some(p);
            self.targets = self.move_targets(p, val);
            self.b.play_sound("click");
        } else if let Some(s) = self.sel.filter(|_| self.targets.contains(&p)) {
            self.board[s] = 0;
            self.board[p] = val;
            self.last_spot = Some(p);
            self.sel = None;
            self.targets.clear();
            self.after_action(p, val, opp);
        } else {
            self.sel = None;
            self.targets.clear();
        }
    }

    fn move_targets(&self, p: usize, val: u8) -> Vec<usize> {
        if self.flying && count(&self.board, val) == 3 {
            return (0..24).filter(|&d| self.board[d] == 0).collect();
        }
        ADJ[p].iter().copied().filter(|&d| self.board[d] == 0).collect()
    }

    fn after_action(&mut self, p: usize, val: u8, opp: u8) {
        if forms_mill(&self.board, p, val) {
            self.mill_spots = mills_at(p).filter(|m| m.iter().all(|&q| self.board[q] == val)).copied().collect();
            self.mill_t = 0.0;
            if double_mill(&self.board, val) {
                self.b.ach_event("muehle_double", None);
            }
            self.removable = removable(&self.board, opp);
            if !self.removable.is_empty() {
                self.remove_mode = true;
                self.b.play_sound("point");
                return;
            }
        }
        self.b.play_sound("lock");
        self.end_turn();
    }

    fn end_turn(&mut self) {
        self.mill_spots.clear();
        self.player = 1 - self.player;
        self.sel = None;
        self.targets.clear();
        self.check_state();
        if self.state == State::Play && !self.b.multiplayer && self.player == 1 {
            self.ai_delay = 0.45;
        }
    }

    fn check_state(&mut self) {
        if self.placed[0] >= 9 && self.placed[1] >= 9 {
            let val = self.player as u8 + 1;
            if count(&self.board, val) < 3 || !has_moves(&self.board, &self.placed, self.player, self.flying) {
                self.winner = 1 - self.player;
                self.end();
            }
        }
    }

    fn end(&mut self) {
        self.state = State::Over;
        self.wins[self.winner] += 1;
        if !self.b.multiplayer {
            if self.winner == 0 {
                self.b.score = self.wins[0];
                self.b.play_sound("win");
                self.b.report_result(true);
            } else {
                self.b.play_sound("gameover");
                self.b.report_result(false);
            }
        } else {
            self.b.report_duel(self.winner);
            self.b.play_sound("win");
        }
        self.b.game_over = true;
    }

    fn restart(&mut self) {
        self.starter = 1 - self.starter;
        self.b.game_over = false;
        self.new_round();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn ai_play(&mut self) {
        let Some(mv) = self.pick_ai_move() else {
            self.check_state();
            return;
        };
        let val = 2;
        if mv.place {
            self.board[mv.a] = val;
            self.placed[1] += 1;
            self.last_spot = Some(mv.a);
        } else {
            self.board[mv.a] = 0;
            self.board[mv.b] = val;
            self.last_spot = Some(mv.b);
        }
        let spot = if mv.place { mv.a } else { mv.b };
        if let Some(rp) = mv.rp {
            let mills: Vec<[usize; 3]> = mills_at(spot).filter(|m| m.iter().all(|&q| self.board[q] == val)).copied().collect();
            self.board[rp] = 0;
            self.b.play_sound("hit");
            self.end_turn();
            self.mill_spots = mills;
            self.mill_t = 1.2;
        } else {
            self.b.play_sound("lock");
            self.end_turn();
        }
    }

    fn pick_ai_move(&mut self) -> Option<Mv> {
        let mut b = self.board;
        let moves = gen_moves(&mut b, &self.placed, 1, self.flying);
        if moves.is_empty() {
            return None;
        }
        let schwelle = match self.diff {
            0 => 0.55,
            1 => 0.18,
            _ => 0.0,
        };
        if py::random() < schwelle {
            let caps: Vec<Mv> = moves.iter().copied().filter(|m| m.rp.is_some()).collect();
            if !caps.is_empty() && py::random() < 0.7 {
                return Some(py::choice(&caps));
            }
            return Some(py::choice(&moves));
        }
        let depth = DEPTHS[self.diff];
        let mut suche = Suche { nodes: 0, deadline: py::time_time() + TIME_BUDGET[self.diff], flying: self.flying };
        let mut best_val = -INF;
        let mut best: Vec<Mv> = Vec::new();
        for mv in order(&moves) {
            let (nb, npl) = apply_move(&self.board, &self.placed, 1, &mv);
            let val = -suche.search(&nb, &npl, 0, depth - 1, -INF, INF);
            if !best.is_empty() && py::time_time() > suche.deadline {
                break;
            }
            if val > best_val {
                best_val = val;
                best = vec![mv];
            } else if val == best_val {
                best.push(mv);
            }
        }
        Some(if best.is_empty() { py::choice(&moves) } else { py::choice(&best) })
    }

    fn draw_board(&self, s: &mut Surface) {
        draw::rect_r(s, COL_PLATE, self.plate_rect, 0, 12);
        draw::rect_r(s, ui::mix(COL_PLATE, self.b.accent, 0.45), self.plate_rect, 1, 12);
        for i in 0..24 {
            for &j in ADJ[i] {
                if j > i {
                    draw::line(s, COL_LINE, self.pts[i], self.pts[j], 3);
                }
            }
        }
        for m in &self.mill_spots {
            draw::line(s, COL_MILL, self.pts[m[0]], self.pts[m[2]], 5);
        }
        let human = self.state == State::Play && self.human_turn();
        for i in 0..24 {
            let (x, y) = self.pts[i];
            let v = self.board[i];
            if v == 0 {
                draw::circle(s, COL_SPOT, (x, y), 4.max(self.pr / 2), 0);
            } else {
                let base = if v == 1 { COL_P1 } else { COL_P2 };
                let hi = if v == 1 { COL_P1_HI } else { COL_P2_HI };
                draw::circle(s, rgb(10, 16, 12), (x, y + 2), self.pr, 0);
                draw::circle(s, base, (x, y), self.pr, 0);
                draw::circle(s, hi, (x - self.pr / 3, y - self.pr / 3), 2.max(self.pr / 4), 0);
            }
        }
        if human && !self.remove_mode && self.placed[self.player] < 9 {
            for i in 0..24 {
                if self.board[i] == 0 {
                    draw::circle(s, COL_HINT, self.pts[i], 3.max(self.pr / 4), 0);
                }
            }
        }
        if human {
            if let Some(sel) = self.sel {
                draw::circle(s, COL_SEL, self.pts[sel], self.pr + 3, 3);
                for &d in &self.targets {
                    draw::circle(s, COL_HINT, self.pts[d], 4.max(self.pr / 3), 0);
                }
            }
        }
        if human && self.remove_mode {
            let k = ui::pulse(2.8, 0.0, 1.0);
            for &i in &self.removable {
                draw::circle(s, COL_REMOVE, self.pts[i], self.pr + 2 + (3.0 * k) as i32, 3);
            }
        }
        if let Some(ls) = self.last_spot.filter(|&l| self.board[l] != 0) {
            let col = ui::mix(COL_SPOT, self.b.accent, 0.55 + 0.35 * ui::pulse(1.6, 0.0, 1.0));
            draw::circle(s, col, self.pts[ls], self.pr + 4, 2);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let panel = Rect::new(8, 6, self.b.width - 16, self.hud_h - 10);
        ui::draw_panel_ex(s, panel, None, None, None, false, Some(self.b.accent));
        let cy = panel.centery();
        for (idx, col) in [COL_P1, COL_P2].into_iter().enumerate() {
            let on = count(&self.board, idx as u8 + 1);
            let left = 9 - self.placed[idx];
            let txt = if left > 0 { format!("{on} (+{left})") } else { on.to_string() };
            let img = self.small.render(&txt, true, p.text);
            if idx == 0 {
                draw::circle(s, col, (panel.x + 16, cy), 8, 0);
                draw::circle(s, p.border_light, (panel.x + 16, cy), 8, 1);
                s.blr(&img, img.get_rect().midleft_at((panel.x + 30, cy)));
            } else {
                draw::circle(s, col, (panel.right() - 16, cy), 8, 0);
                draw::circle(s, p.border_light, (panel.right() - 16, cy), 8, 1);
                s.blr(&img, img.get_rect().midright_at((panel.right() - 30, cy)));
            }
        }
        if self.state == State::Play {
            let mp = self.b.multiplayer;
            let who = || if self.player == 0 { t("common.player1") } else { t("common.player2") };
            let mid = if self.remove_mode && self.human_turn() {
                t("mill.remove")
            } else if !mp && self.player == 1 {
                t("mill.ai_thinks")
            } else if self.placed[self.player] < 9 {
                if mp {
                    tf("mill.place_turn", &[("name", &who())])
                } else {
                    t("mill.place_you")
                }
            } else if mp {
                tf("mill.move_turn", &[("name", &who())])
            } else {
                t("mill.move_you")
            };
            let img = self.small.render(&mid, true, self.b.accent);
            s.blr(&img, img.get_rect().center_at((self.b.width / 2, cy)));
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let head = if self.b.multiplayer {
            self.huge.render(&tf("common.player_wins", &[("n", &(self.winner + 1).to_string())]), true, self.b.accent)
        } else {
            let won = self.winner == 0;
            self.huge.render(&if won { t("mill.win_you") } else { t("mill.win_ai") }, true, if won { self.b.accent } else { p.text_dim })
        };
        let hint_key = if self.b.multiplayer { "common.enter_restart" } else { "mill.new_round" };
        let hint = self.tiny.render(&t(hint_key), true, p.text_dim);
        let w = (self.b.width - 24).min(head.w().max(hint.w()) + 64);
        let panel = Rect::new(cx - py::fdiv32(w, 2), self.b.height / 2 - 48, w, 96);
        ui::draw_panel_ex(s, panel, None, None, None, false, Some(self.b.accent));
        s.blr(&head, head.get_rect().center_at((cx, panel.y + 36)));
        s.blr(&hint, hint.get_rect().center_at((cx, panel.y + 74)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_title_ex(
            s,
            w,
            &t("mill.title"),
            Some(&t("mill.subtitle")),
            (h as f64 * 0.14) as i32,
            Some(&self.huge),
            Some(&self.small),
            Some(self.b.accent),
        );
        for (i, rc) in self.diff_rects.iter().enumerate() {
            ui::draw_button_ex(s, *rc, &t(&format!("mill.diff.{}", DIFFS[i])), &self.b.font, i == self.diff, None, None, Some(self.b.accent));
        }
        let state = if self.flying { t("common.on") } else { t("common.off") };
        ui::draw_button_ex(s, self.fly_rect, &format!("{}: {state}", t("mill.flying")), &self.b.font, self.flying, None, None, Some(self.b.accent));
        ui::draw_button_ex(s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(self.b.accent));
        ui::draw_footer_ex(s, w, h, &t("mill.setup_hint"), Some(&self.tiny));
    }
}

impl Game for MuehleGame {
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
        self.diff = self.b.opt_i64("muehle", "difficulty", 1).clamp(0, 2) as usize;
        self.flying = self.b.opt_bool("muehle", "flying", true);
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
        if !self.human_turn() {
            return;
        }
        if ev.kind == Kind::MouseDown {
            if let Some(p) = self.spot_at(ev.pos) {
                self.click_spot(p);
            }
        } else if ev.kind == Kind::KeyDown && ev.key == "Escape" {
            self.sel = None;
            self.targets.clear();
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
        }
        if self.mill_t > 0.0 {
            self.mill_t -= dt;
            if self.mill_t <= 0.0 {
                self.mill_t = 0.0;
                if !self.remove_mode {
                    self.mill_spots.clear();
                }
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
    key: "muehle",
    class_name: "MuehleGame",
    supports_multiplayer: true,
    modes: &[],
    make: MuehleGame::new,
};
