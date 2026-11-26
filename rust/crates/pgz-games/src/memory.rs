//! memory.rs
//! =========
//! Memory - Paare finden, allein oder im Duell.
//!
//! Uebertragen aus `games/memory.py`: drei Brettgroessen (gespeichert),
//! 24 gezeichnete Motive (8 Formen x 3 Farben), Flip-Animation,
//! Punkte aus Zuegen und Zeit, Duell mit Zugwechsel.

use pgz_engine::prelude::*;

const COL_CARD_BACK: Color = Color::rgb(56, 64, 92);
const COL_CARD_BACK_D: Color = Color::rgb(40, 46, 66);
const COL_CARD_FACE: Color = Color::rgb(232, 234, 240);
const COL_CARD_DONE: Color = Color::rgb(58, 66, 88);
const COL_CARD_EDGE: Color = Color::rgb(74, 84, 116);
const COL_P1: Color = Color::rgb(88, 156, 255);
const COL_P2: Color = Color::rgb(245, 205, 100);
const SIZES: [(&str, usize, usize, i64); 3] = [("4x4", 4, 4, 1000), ("6x6", 6, 6, 2500), ("8x6", 8, 6, 4000)];
const MOTIF_COLORS: [Color; 3] = [Color::rgb(225, 95, 95), Color::rgb(88, 156, 255), Color::rgb(245, 205, 100)];
const N_SHAPES: usize = 8;
const FLIP_TIME: f64 = 0.25;
const RESOLVE_TIME: f64 = 0.8;

fn pts_star(cx: f64, cy: f64, r: f64) -> Vec<(f64, f64)> {
    (0..10)
        .map(|i| {
            let rr = if i % 2 == 0 { r } else { r * 0.45 };
            let a = -py::PI / 2.0 + i as f64 * py::PI / 5.0;
            (cx + rr * a.cos(), cy + rr * a.sin())
        })
        .collect()
}

/// Die acht Motiv-Formen (`SHAPES`), gezeichnet in `rect`.
fn draw_shape(i: usize, s: &mut Surface, rect: Rect, col: Color, face: Color) {
    let (cx, cy) = rect.center();
    let (cxf, cyf) = (cx as f64, cy as f64);
    let (w, h) = (rect.w as f64, rect.h as f64);
    match i {
        0 => {
            draw::polygon(s, col, &pts_star(cxf, cyf, w * 0.45), 0);
        }
        1 => {
            let r = rect.w / 4;
            let (cx, cy) = (rect.centerx(), rect.centery() - r / 3);
            draw::circle(s, col, (cx - r + 1, cy), r, 0);
            draw::circle(s, col, (cx + r - 1, cy), r, 0);
            draw::polygon(s, col, &[(cx - 2 * r + 1, cy + r / 3), (cx + 2 * r - 1, cy + r / 3), (cx, cy + 2 * r)], 0);
        }
        2 => {
            let r = (w * 0.4) as i32;
            draw::circle(s, col, rect.center(), r, 0);
            draw::circle(s, face, (rect.centerx() + r / 2, rect.centery() - r / 3), r, 0);
        }
        3 => {
            let (dw, dh) = (w * 0.38, h * 0.46);
            draw::polygon(s, col, &[(cxf, cyf - dh), (cxf + dw, cyf), (cxf, cyf + dh), (cxf - dw, cyf)], 0);
        }
        4 => {
            let r = w * 0.42;
            draw::polygon(s, col, &[(cxf, cyf - r), (cxf + r, cyf + r * 0.7), (cxf - r, cyf + r * 0.7)], 0);
        }
        5 => {
            draw::circle(s, col, rect.center(), (w * 0.4) as i32, 3.max(rect.w / 8));
        }
        6 => {
            let a = (w * 0.42) as i32;
            let b = 3.max(rect.w / 7);
            draw::rect_r(s, col, (cx - b, cy - a, 2 * b, 2 * a), 0, b);
            draw::rect_r(s, col, (cx - a, cy - b, 2 * a, 2 * b), 0, b);
        }
        _ => {
            let (bw, bh) = (w * 0.32, h * 0.45);
            draw::polygon(
                s,
                col,
                &[
                    (cxf + bw * 0.4, cyf - bh),
                    (cxf - bw, cyf + bh * 0.15),
                    (cxf - bw * 0.05, cyf + bh * 0.15),
                    (cxf - bw * 0.4, cyf + bh),
                    (cxf + bw, cyf - bh * 0.15),
                    (cxf + bw * 0.05, cyf - bh * 0.15),
                ],
                0,
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum KState {
    Down,
    Up,
    Done,
}

#[derive(Clone, Copy)]
struct Card {
    motif: (usize, usize),
    state: KState,
    p: f64,
    target: f64,
}

/// `MemoryGame`
pub struct MemoryGame {
    b: GameBase,
    size_idx: usize,
    setup: bool,
    size_rects: Vec<Rect>,
    start_rect: Rect,
    cols: usize,
    rows_n: usize,
    base: i64,
    cards: Vec<Card>,
    first: Option<usize>,
    resolve_t: f64,
    resolve_pair: Option<(usize, usize)>,
    moves: i64,
    elapsed: f64,
    cursor: usize,
    turn: usize,
    pairs: [i64; 2],
    found: usize,
    hud_h: i32,
    ch: i32,
    cw: i32,
    bx: i32,
    by: i32,
    gap: i32,
    small: Font,
    tiny: Font,
    huge: Font,
}

impl MemoryGame {
    /// `MemoryGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = MemoryGame {
            b,
            size_idx: 1,
            setup: true,
            size_rects: Vec::new(),
            start_rect: z,
            cols: 6,
            rows_n: 6,
            base: 2500,
            cards: Vec::new(),
            first: None,
            resolve_t: 0.0,
            resolve_pair: None,
            moves: 0,
            elapsed: 0.0,
            cursor: 0,
            turn: 0,
            pairs: [0, 0],
            found: 0,
            hud_h: 0,
            ch: 0,
            cw: 0,
            bx: 0,
            by: 0,
            gap: 0,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 30));
        self.tiny = ui::f(11.max(h / 36));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 360.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.32) as i32;
        self.size_rects = (0..3).map(|i| Rect::new(cx - bw / 2, y0 + i * 58, bw, 48)).collect();
        self.start_rect = Rect::new(cx - 95, y0 + 3 * 58 + 14, 190, 46);
    }

    fn save_size(&mut self) {
        self.b.set_opt("memory", "size", SIZES[self.size_idx].0.into());
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "1" | "2" | "3" => {
                    self.size_idx = ev.key.parse::<usize>().unwrap_or(2) - 1;
                    self.save_size();
                    self.b.play_sound("click");
                }
                "Up" | "w" | "W" => {
                    self.size_idx = py::pmod(self.size_idx as i64 - 1, 3) as usize;
                    self.save_size();
                    self.b.play_sound("move");
                }
                "Down" | "s" | "S" => {
                    self.size_idx = (self.size_idx + 1) % 3;
                    self.save_size();
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.new_game(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.size_rects.iter().position(|r| r.hit(ev.pos)) {
                self.size_idx = i;
                self.save_size();
                self.b.play_sound("click");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.new_game();
            }
        }
    }

    fn new_game(&mut self) {
        self.b.new_round_result();
        self.b.score = 0;
        self.b.game_over = false;
        let (_k, cols, rows, base) = SIZES[self.size_idx];
        self.cols = cols;
        self.rows_n = rows;
        self.base = base;
        let n_pairs = cols * rows / 2;
        let mut catalog: Vec<(usize, usize)> = Vec::new();
        for sh in 0..N_SHAPES {
            for co in 0..MOTIF_COLORS.len() {
                catalog.push((sh, co));
            }
        }
        let mut rng = py::Random::unseeded();
        rng.shuffle(&mut catalog);
        let mut motifs: Vec<(usize, usize)> = catalog[..n_pairs].to_vec();
        motifs.extend_from_slice(&catalog[..n_pairs]);
        rng.shuffle(&mut motifs);
        self.cards = motifs.into_iter().map(|m| Card { motif: m, state: KState::Down, p: 0.0, target: 0.0 }).collect();
        self.first = None;
        self.resolve_t = 0.0;
        self.resolve_pair = None;
        self.moves = 0;
        self.elapsed = 0.0;
        self.cursor = 0;
        self.turn = 0;
        self.pairs = [0, 0];
        self.found = 0;
        self.layout_board();
        self.setup = false;
        self.b.play_sound("click");
    }

    fn layout_board(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 40.max((h as f64 * 0.09) as i32);
        let m = 8.max(w / 100);
        let (cols, rows) = (self.cols as i32, self.rows_n as i32);
        let slot_w = py::fdiv32(w - m * (cols + 1), cols);
        let slot_h = py::fdiv32(h - self.hud_h - m * (rows + 1), rows);
        self.ch = slot_h.min((slot_w as f64 * 4.0 / 3.0) as i32);
        self.cw = (self.ch as f64 * 0.75) as i32;
        let total_w = cols * self.cw + (cols - 1) * m;
        let total_h = rows * self.ch + (rows - 1) * m;
        self.bx = py::fdiv32(w - total_w, 2);
        self.by = self.hud_h + py::fdiv32(h - self.hud_h - total_h, 2);
        self.gap = m;
    }

    fn card_rect(&self, i: usize) -> Rect {
        let (r, c) = ((i / self.cols) as i32, (i % self.cols) as i32);
        Rect::new(self.bx + c * (self.cw + self.gap), self.by + r * (self.ch + self.gap), self.cw, self.ch)
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let (r, c) = ((self.cursor / self.cols) as i32, (self.cursor % self.cols) as i32);
        let c = (c + dx).clamp(0, self.cols as i32 - 1);
        let r = (r + dy).clamp(0, self.rows_n as i32 - 1);
        let new = (r * self.cols as i32 + c) as usize;
        if new != self.cursor {
            self.cursor = new;
            self.b.play_sound("move");
        }
    }

    fn flip(&mut self, i: usize) {
        if self.resolve_t > 0.0 || self.cards[i].state != KState::Down {
            return;
        }
        self.cards[i].state = KState::Up;
        self.cards[i].target = 1.0;
        self.b.play_sound("click");
        let Some(first) = self.first else {
            self.first = Some(i);
            return;
        };
        self.moves += 1;
        if self.cards[first].motif == self.cards[i].motif {
            self.cards[first].state = KState::Done;
            self.cards[i].state = KState::Done;
            let who = if self.b.multiplayer { self.turn } else { 0 };
            self.pairs[who] += 1;
            self.found += 1;
            self.first = None;
            self.b.play_sound("merge");
            if self.found >= self.cards.len() / 2 {
                self.finish();
            }
        } else {
            self.resolve_t = RESOLVE_TIME;
            self.resolve_pair = Some((first, i));
            self.first = None;
        }
    }

    fn finish(&mut self) {
        if self.b.multiplayer {
            let winner = self.pairs[0].max(self.pairs[1]);
            self.b.score = winner * 100;
            if self.pairs[0] != self.pairs[1] {
                self.b.report_duel(if self.pairs[0] > self.pairs[1] { 0 } else { 1 });
            }
        } else {
            self.b.score = 100.max(self.base - 15 * self.moves - 2 * self.elapsed as i64);
            if self.moves as usize == self.cards.len() / 2 {
                self.b.ach_event("memory_perfect", None);
            }
        }
        if self.size_idx == 0 && self.elapsed < 30.0 {
            self.b.ach_event("memory_swift", None);
        }
        self.b.game_over = true;
        self.b.play_sound("win");
        self.b.rumble(200);
    }

    fn fmt_time(&self) -> String {
        let sec = self.elapsed as i64;
        format!("{:02}:{:02}", sec / 60, sec % 60)
    }

    fn draw_card(&self, s: &mut Surface, i: usize, card: &Card) {
        let rect = self.card_rect(i);
        let wf = (py::PI * card.p).cos().abs();
        let w = 3.max((rect.w as f64 * wf) as i32);
        let r = Rect::new(rect.centerx() - w / 2, rect.y, w, rect.h);
        let front = card.p > 0.5;
        let rad = 4.max(rect.w / 8);
        if front {
            let face_col = if card.state == KState::Done { COL_CARD_DONE } else { COL_CARD_FACE };
            draw::rect_r(s, face_col, r, 0, rad);
            draw::rect_r(s, COL_CARD_EDGE, r, 2, rad);
            if w as f64 > rect.w as f64 * 0.5 {
                let (shape_i, col_i) = card.motif;
                let mut col = MOTIF_COLORS[col_i];
                if card.state == KState::Done {
                    col = pg::scaled(col, 0.55);
                }
                let inner = rect.inflate(py::fdiv32(-rect.w, 4), py::fdiv32(-rect.h, 3));
                draw_shape(shape_i, s, inner, col, face_col);
                if card.state == KState::Done {
                    let (bx, by) = (rect.right() - 12, rect.bottom() - 10);
                    draw::lines(s, ui::green(), false, &[(bx - 6, by - 3), (bx - 3, by), (bx + 3, by - 7)], 2);
                }
            }
        } else {
            draw::rect_r(s, COL_CARD_BACK, r, 0, rad);
            draw::rect_r(s, COL_CARD_BACK_D, r, 2, rad);
            if w as f64 > rect.w as f64 * 0.5 {
                let q = self.small.render("?", true, self.b.accent);
                s.blr(&q, q.get_rect().center_at(rect.center()));
            }
        }
        if i == self.cursor && !self.b.game_over {
            draw::rect_r(s, self.b.accent, rect.inflate(6, 6), 2, rad + 3);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        if self.b.multiplayer {
            let n1: String = self.b.pname(0).chars().take(10).collect();
            let n2: String = self.b.pname(1).chars().take(10).collect();
            let p1 = self.small.render(
                &format!("{n1}  {}", tf("mem.player_pairs", &[("n", &self.pairs[0].to_string())])),
                true,
                if self.turn == 0 { COL_P1 } else { p.text_dim },
            );
            s.blr(&p1, p1.get_rect().midleft_at((12, cy)));
            let p2 = self.small.render(
                &format!("{}  {n2}", tf("mem.player_pairs", &[("n", &self.pairs[1].to_string())])),
                true,
                if self.turn == 1 { COL_P2 } else { p.text_dim },
            );
            s.blr(&p2, p2.get_rect().midright_at((w - 12, cy)));
            let who = if self.turn == 0 { n1 } else { n2 };
            let mid = self.small.render(&tf("mem.turn", &[("name", &who)]), true, self.b.accent);
            s.blr(&mid, mid.get_rect().center_at((w / 2, cy)));
        } else {
            let mv = self.small.render(&tf("mem.moves", &[("n", &self.moves.to_string())]), true, p.text);
            s.blr(&mv, mv.get_rect().midleft_at((12, cy)));
            let tm = self.small.render(&tf("mem.time", &[("t", &self.fmt_time())]), true, self.b.accent);
            s.blr(&tm, tm.get_rect().center_at((w / 2, cy)));
            let left = self.cards.len() / 2 - self.found;
            let pr = self.small.render(&tf("mem.pairs", &[("n", &left.to_string())]), true, p.text_dim);
            s.blr(&pr, pr.get_rect().midright_at((w - 12, cy)));
        }
    }

    fn draw_result(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(Color::rgba(8, 10, 16, 185));
        s.bl(&ov, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let (head, sub) = if self.b.multiplayer {
            let head = if self.pairs[0] == self.pairs[1] {
                self.huge.render(&t("common.draw"), true, p.text_dim)
            } else {
                let n = if self.pairs[0] > self.pairs[1] { 1 } else { 2 };
                let name: String = self.b.pname(n - 1).chars().take(10).collect();
                self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, if n == 1 { COL_P1 } else { COL_P2 })
            };
            (head, self.b.font.render(&format!("{} : {}", self.pairs[0], self.pairs[1]), true, p.text))
        } else {
            (
                self.huge.render(&tf("mem.win", &[("t", &self.fmt_time()), ("m", &self.moves.to_string())]), true, p.green),
                self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text),
            )
        };
        let pw = (w - 40).min(460).max(head.w() + 40);
        let panel = Rect::new(cx - pw / 2, cy - 90, pw, 160);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, self.b.accent, panel, 2, 14);
        s.blr(&head, head.get_rect().center_at((cx, cy - 50)));
        s.blr(&sub, sub.get_rect().center_at((cx, cy + 4)));
        let hint = self.small.render(&t("mem.retry"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 40)));
    }

    fn draw_setup(&self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(&mut s, w, h);
        let title = self.huge.render("MEMORY", true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((w / 2, (h as f64 * 0.14) as i32)));
        let sub = self.small.render(&t("mem.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((w / 2, (h as f64 * 0.21) as i32)));
        for (i, r) in self.size_rects.iter().enumerate() {
            let on = i == self.size_idx;
            draw::rect_r(&mut s, if on { p.btn_sel } else { p.btn }, *r, 0, 10);
            draw::rect_r(&mut s, if on { self.b.accent } else { p.border }, *r, if on { 2 } else { 1 }, 10);
            let (_k, cols, rows, _base) = SIZES[i];
            let name = self.b.font.render(&format!("{cols} x {rows}"), true, if on { p.text } else { p.text_dim });
            s.blr(&name, name.get_rect().midleft_at((r.x + 18, r.centery())));
            let pairs = self.tiny.render(&tf("mem.pairs", &[("n", &(cols * rows / 2).to_string())]), true, p.text_dim);
            s.blr(&pairs, pairs.get_rect().midright_at((r.right() - 18, r.centery())));
        }
        draw::rect_r(&mut s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(&mut s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("mem.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((w / 2, h - 30)));
        let ctrl = self.tiny.render(&t("mem.hint"), true, p.text_faint);
        s.blr(&ctrl, ctrl.get_rect().center_at((w / 2, h - 12)));
    }
}

impl Game for MemoryGame {
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
        let key = self.b.opt_str("memory", "size", "6x6");
        self.size_idx = SIZES.iter().position(|s| s.0 == key).unwrap_or(1);
        self.build_fonts();
        self.build_setup_layout();
        self.setup = true;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.build_setup_layout();
        if !self.setup {
            self.layout_board();
        }
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.setup {
            self.handle_setup(ev);
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown {
                if ev.key_in(&["Return", "space"]) {
                    self.new_game();
                } else if ev.key_in(&["s", "S"]) {
                    self.setup = true;
                    self.b.play_sound("click");
                }
            }
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if k == "r" || k == "R" {
                self.new_game();
            } else if (k == "s" || k == "S") && !self.b.is_action(k, "down", None) {
                self.setup = true;
                self.b.play_sound("click");
            } else if self.b.is_action(k, "up", None) || k == "Up" {
                self.move_cursor(0, -1);
            } else if self.b.is_action(k, "down", None) || k == "Down" {
                self.move_cursor(0, 1);
            } else if self.b.is_action(k, "left", None) || k == "Left" {
                self.move_cursor(-1, 0);
            } else if self.b.is_action(k, "right", None) || k == "Right" {
                self.move_cursor(1, 0);
            } else if k == "space" || k == "Return" {
                self.flip(self.cursor);
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = (0..self.cards.len()).find(|i| self.card_rect(*i).hit(ev.pos)) {
                self.cursor = i;
                self.flip(i);
            }
        }
    }

    fn update(&mut self, dt: f64) {
        if self.setup || self.b.game_over {
            return;
        }
        self.elapsed += dt;
        for card in self.cards.iter_mut() {
            if card.p < card.target {
                card.p = card.target.min(card.p + dt / FLIP_TIME);
            } else if card.p > card.target {
                card.p = card.target.max(card.p - dt / FLIP_TIME);
            }
        }
        if self.resolve_t > 0.0 {
            self.resolve_t -= dt;
            if self.resolve_t <= 0.0 {
                if let Some((a, b)) = self.resolve_pair.take() {
                    for i in [a, b] {
                        if self.cards[i].state == KState::Up {
                            self.cards[i].state = KState::Down;
                            self.cards[i].target = 0.0;
                        }
                    }
                    if self.b.multiplayer {
                        self.turn = 1 - self.turn;
                    }
                    self.b.play_sound("move");
                }
            }
        }
    }

    fn draw(&mut self) {
        if self.setup {
            self.draw_setup();
            return;
        }
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        for (i, card) in self.cards.iter().enumerate() {
            self.draw_card(&mut s, i, card);
        }
        self.draw_hud(&mut s);
        if self.b.game_over {
            self.draw_result(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "memory",
    class_name: "MemoryGame",
    supports_multiplayer: true,
    modes: &[],
    make: MemoryGame::new,
};
