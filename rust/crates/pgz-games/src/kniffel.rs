//! kniffel.rs
//! ==========
//! Kniffel (Yahtzee) - Highscore-Jagd allein oder Hotseat zu zweit.
//!
//! Uebertragen aus `games/kniffel.py`: 13 Kategorien, Oberer-Block-Bonus,
//! Halten zwischen den Wuerfen, Wurfanimation (die beim Zeichnen wuerfelt -
//! wie in Python, damit die Zufallsfolge gleich bleibt).

use pgz_engine::prelude::*;

const COL_PANEL: Color = Color::rgb(28, 28, 38);
const COL_PANEL_HI: Color = Color::rgb(40, 40, 54);
const COL_TEXT: Color = Color::rgb(228, 230, 238);
const COL_DIM: Color = Color::rgb(150, 154, 170);
const COL_FAINT: Color = Color::rgb(110, 114, 130);
const COL_ACCENT: Color = Color::rgb(232, 176, 75);
const COL_GOOD: Color = Color::rgb(110, 205, 140);
const COL_P1: Color = Color::rgb(232, 176, 75);
const COL_P2: Color = Color::rgb(110, 170, 235);
const COL_DIE: Color = Color::rgb(240, 242, 248);
const COL_DIE_HELD: Color = Color::rgb(250, 226, 160);
const COL_PIP: Color = Color::rgb(36, 38, 48);
const COL_BTN: Color = Color::rgb(44, 44, 58);
const COL_BTN_ON: Color = Color::rgb(92, 70, 34);
const COL_BTN_BORDER: Color = Color::rgb(78, 78, 100);

/// Kategorien in Anzeigereihenfolge (`CATS`).
const CATS: [&str; 13] = [
    "ones",
    "twos",
    "threes",
    "fours",
    "fives",
    "sixes",
    "three_kind",
    "four_kind",
    "full_house",
    "small_straight",
    "large_straight",
    "kniffel",
    "chance",
];
const UPPER_BONUS: i64 = 35;
const UPPER_TARGET: i64 = 63;

fn upper_face(key: &str) -> Option<i64> {
    match key {
        "ones" => Some(1),
        "twos" => Some(2),
        "threes" => Some(3),
        "fours" => Some(4),
        "fives" => Some(5),
        "sixes" => Some(6),
        _ => None,
    }
}

/// `kniffel.score_category(key, dice)`
pub fn score_category(key: &str, dice: &[i64; 5]) -> i64 {
    let counts: Vec<i64> = (1..=6).map(|v| dice.iter().filter(|d| **d == v).count() as i64).collect();
    let total: i64 = dice.iter().sum();
    let max_c = *counts.iter().max().unwrap_or(&0);
    let mut faces: Vec<i64> = dice.to_vec();
    faces.sort_unstable();
    faces.dedup();
    if let Some(face) = upper_face(key) {
        return counts[(face - 1) as usize] * face;
    }
    match key {
        "three_kind" => {
            if max_c >= 3 {
                total
            } else {
                0
            }
        }
        "four_kind" => {
            if max_c >= 4 {
                total
            } else {
                0
            }
        }
        "full_house" => {
            let mut nz: Vec<i64> = counts.iter().copied().filter(|c| *c != 0).collect();
            nz.sort_unstable();
            if nz == [2, 3] {
                25
            } else {
                0
            }
        }
        "small_straight" => {
            let runs: [[i64; 4]; 3] = [[1, 2, 3, 4], [2, 3, 4, 5], [3, 4, 5, 6]];
            if runs.iter().any(|run| run.iter().all(|v| faces.contains(v))) {
                30
            } else {
                0
            }
        }
        "large_straight" => {
            if faces == [1, 2, 3, 4, 5] || faces == [2, 3, 4, 5, 6] {
                40
            } else {
                0
            }
        }
        "kniffel" => {
            if max_c >= 5 {
                50
            } else {
                0
            }
        }
        "chance" => total,
        _ => 0,
    }
}

type Card = [Option<i64>; 13];

fn upper_sum(card: &Card) -> i64 {
    (0..6).filter_map(|i| card[i]).sum()
}

fn grand_total(card: &Card) -> i64 {
    let up = upper_sum(card);
    let bonus = if up >= UPPER_TARGET { UPPER_BONUS } else { 0 };
    let low: i64 = (6..13).filter_map(|i| card[i]).sum();
    up + bonus + low
}

#[derive(PartialEq, Clone, Copy)]
enum Zustand {
    Ready,
    RollAnim,
    Rolled,
    Over,
}

/// `KniffelGame`
pub struct KniffelGame {
    b: GameBase,
    nplayers: usize,
    cards: Vec<Card>,
    player: usize,
    dice: [i64; 5],
    held: [bool; 5],
    rolls_left: i64,
    rolled_once: bool,
    anim_t: f64,
    final_: [i64; 5],
    winner: Option<usize>,
    sel: usize,
    state: Zustand,
    hud_h: i32,
    dice_h: i32,
    dice_y: i32,
    row_h: i32,
    card_top: i32,
    name_x: i32,
    val_x: Vec<i32>,
    col_w: i32,
    row_rects: Vec<Rect>,
    die: i32,
    die_rects: Vec<Rect>,
    roll_rect: Rect,
    small: Font,
    tiny: Font,
    row: Font,
    big: Font,
    huge: Font,
}

impl KniffelGame {
    /// `KniffelGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let mut g = KniffelGame {
            b,
            nplayers: 1,
            cards: Vec::new(),
            player: 0,
            dice: [1; 5],
            held: [false; 5],
            rolls_left: 3,
            rolled_once: false,
            anim_t: 0.0,
            final_: [1; 5],
            winner: None,
            sel: 0,
            state: Zustand::Ready,
            hud_h: 0,
            dice_h: 0,
            dice_y: 0,
            row_h: 0,
            card_top: 0,
            name_x: 0,
            val_x: Vec::new(),
            col_w: 0,
            row_rects: Vec::new(),
            die: 0,
            die_rects: Vec::new(),
            roll_rect: Rect::new(0, 0, 0, 0),
            small: f.clone(),
            tiny: f.clone(),
            row: f.clone(),
            big: f.clone(),
            huge: f,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        self.small = ui::f(15);
        self.tiny = ui::f(12);
        self.row = ui::fm(14);
        self.big = ui::fb(20);
        self.huge = ui::fb(26.max(self.b.height / 12));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 42;
        self.dice_h = 92;
        self.dice_y = h - self.dice_h;
        let top = self.hud_h + 6;
        let bottom = self.dice_y - 6;
        let n_rows = CATS.len() as i32 + 2;
        self.row_h = 16.max(py::fdiv32(bottom - top, n_rows));
        self.card_top = top;
        self.name_x = 16;
        let col_w = 62;
        self.val_x = if self.nplayers == 1 { vec![w - 20 - col_w] } else { vec![w - 20 - 2 * col_w - 10, w - 20 - col_w] };
        self.col_w = col_w;
        self.row_rects = (0..CATS.len() as i32)
            .map(|i| Rect::new(self.name_x - 6, self.card_top + i * self.row_h, w - self.name_x - 8, self.row_h))
            .collect();
        self.die = ((self.dice_h - 40) as f64).min((w - 200) as f64 / 5.0) as i32;
        self.die = 34.max(self.die);
        let gap = 12;
        let total_w = 5 * self.die + 4 * gap;
        let x0 = 14.max(py::fdiv32(w - 160 - total_w, 2));
        self.die_rects = (0..5).map(|i| Rect::new(x0 + i * (self.die + gap), self.dice_y + (self.dice_h - self.die) / 2, self.die, self.die)).collect();
        let bw = 130;
        self.roll_rect = Rect::new(w - bw - 16, self.dice_y + (self.dice_h - 44) / 2, bw, 44);
    }

    fn start_turn(&mut self) {
        self.rolls_left = 3;
        self.held = [false; 5];
        self.rolled_once = false;
        self.state = Zustand::Ready;
        if let Some(i) = self.cards[self.player].iter().position(Option::is_none) {
            self.sel = i;
        }
    }

    fn handle_key(&mut self, k: &str) {
        match k {
            "space" => self.roll(),
            "1" | "2" | "3" | "4" | "5" => {
                let i = k.parse::<usize>().unwrap_or(1) - 1;
                self.toggle_hold(i);
            }
            "Up" | "w" | "W" => {
                self.sel = py::pmod(self.sel as i64 - 1, CATS.len() as i64) as usize;
                self.b.play_sound("move");
            }
            "Down" | "s" | "S" => {
                self.sel = (self.sel + 1) % CATS.len();
                self.b.play_sound("move");
            }
            "Return" => self.book(self.sel),
            _ => {}
        }
    }

    fn handle_click(&mut self, pos: (i32, i32)) {
        if self.roll_rect.hit(pos) {
            self.roll();
            return;
        }
        if let Some(i) = self.die_rects.iter().position(|rc| rc.hit(pos)) {
            self.toggle_hold(i);
            return;
        }
        if let Some(i) = self.row_rects.iter().position(|rc| rc.hit(pos)) {
            self.sel = i;
            self.book(i);
        }
    }

    fn toggle_hold(&mut self, i: usize) {
        if !self.rolled_once || self.state == Zustand::RollAnim {
            return;
        }
        self.held[i] = !self.held[i];
        self.b.play_sound("click");
    }

    fn roll(&mut self) {
        if self.state == Zustand::RollAnim || self.rolls_left <= 0 {
            return;
        }
        for i in 0..5 {
            self.final_[i] = if self.rolled_once && self.held[i] { self.dice[i] } else { py::randint(1, 6) };
        }
        self.state = Zustand::RollAnim;
        self.anim_t = 0.5;
        self.b.play_sound("move");
    }

    fn book(&mut self, cat_i: usize) {
        if !self.rolled_once || self.state == Zustand::RollAnim {
            return;
        }
        let key = CATS[cat_i];
        if self.cards[self.player][cat_i].is_some() {
            self.b.play_sound("click");
            return;
        }
        let v = score_category(key, &self.dice);
        self.cards[self.player][cat_i] = Some(v);
        if key == "kniffel" && v > 0 {
            self.b.ach_event("kniffel_five", None);
        }
        self.b.play_sound(if v > 0 { "point" } else { "select" });
        if !self.b.multiplayer {
            self.b.score = grand_total(&self.cards[self.player]);
        }
        self.next_turn();
    }

    fn next_turn(&mut self) {
        if self.cards.iter().all(|c| c.iter().all(Option::is_some)) {
            self.ende();
            return;
        }
        if self.b.multiplayer {
            self.player = 1 - self.player;
        }
        self.start_turn();
    }

    fn ende(&mut self) {
        self.state = Zustand::Over;
        self.b.game_over = true;
        let best;
        if self.b.multiplayer {
            let t0 = grand_total(&self.cards[0]);
            let t1 = grand_total(&self.cards[1]);
            self.winner = if t0 > t1 {
                Some(0)
            } else if t1 > t0 {
                Some(1)
            } else {
                None
            };
            if let Some(w) = self.winner {
                self.b.report_duel(w);
            }
            best = t0.max(t1);
            self.b.play_sound(if self.winner.is_some() { "win" } else { "select" });
        } else {
            self.b.score = grand_total(&self.cards[0]);
            best = self.b.score;
            self.b.play_sound("win");
        }
        self.b.ach_event("kniffel_grand", Some(best));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let w = self.b.width;
        draw::rect(s, COL_PANEL, (0, 0, w, self.hud_h), 0);
        draw::line(s, COL_BTN_BORDER, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let img = if self.b.multiplayer {
            let who = self.b.pname(self.player);
            let col = if self.player == 0 { COL_P1 } else { COL_P2 };
            self.big.render(&tf("kn.turn", &[("name", &who)]), true, col)
        } else {
            self.big.render(&tf("kn.total_now", &[("n", &grand_total(&self.cards[0]).to_string())]), true, COL_ACCENT)
        };
        s.blr(&img, img.get_rect().midleft_at((14, cy)));
        if self.state != Zustand::Over {
            let rt = tf("kn.rolls_left", &[("n", &self.rolls_left.max(0).to_string())]);
            let img = self.small.render(&rt, true, COL_DIM);
            s.blr(&img, img.get_rect().midright_at((w - 14, cy)));
        }
    }

    fn draw_card(&self, s: &mut Surface) {
        if self.b.multiplayer {
            for p in 0..2 {
                let name: String = self.b.pname(p).chars().take(9).collect();
                let lbl = self.tiny.render(&name, true, if p == 0 { COL_P1 } else { COL_P2 });
                s.blr(&lbl, lbl.get_rect().center_at((self.val_x[p] + self.col_w / 2, self.card_top - 1)));
            }
        }
        for (i, key) in CATS.iter().enumerate() {
            let y = self.card_top + i as i32 * self.row_h;
            let rc = self.row_rects[i];
            let selected = i == self.sel && self.state != Zustand::Over;
            if selected {
                draw::rect_r(s, COL_PANEL_HI, rc, 0, 5);
                draw::rect(s, COL_ACCENT, (rc.x, rc.y + 2, 3, rc.h - 4), 0);
            }
            let name = self.row.render(&t(&format!("kn.{key}")), true, if selected { COL_TEXT } else { COL_DIM });
            s.blr(&name, name.get_rect().midleft_at((self.name_x, y + self.row_h / 2)));
            for p in 0..self.nplayers {
                let cx = self.val_x[p] + self.col_w / 2;
                let img = if let Some(v) = self.cards[p][i] {
                    self.row.render(&v.to_string(), true, COL_TEXT)
                } else if p == self.player && self.rolled_once && self.state != Zustand::Over {
                    let val = score_category(key, &self.dice);
                    self.row.render(&val.to_string(), true, if val > 0 { COL_GOOD } else { COL_FAINT })
                } else {
                    self.row.render("-", true, COL_FAINT)
                };
                s.blr(&img, img.get_rect().center_at((cx, y + self.row_h / 2)));
            }
        }
        let mut y = self.card_top + CATS.len() as i32 * self.row_h;
        let lbl = self.row.render(&t("kn.upper_bonus"), true, COL_DIM);
        s.blr(&lbl, lbl.get_rect().midleft_at((self.name_x, y + self.row_h / 2)));
        for p in 0..self.nplayers {
            let up = upper_sum(&self.cards[p]);
            let bonus = if up >= UPPER_TARGET { UPPER_BONUS } else { 0 };
            let cx = self.val_x[p] + self.col_w / 2;
            let txt = format!("{up}/{UPPER_TARGET}{}", if bonus != 0 { format!(" +{bonus}") } else { String::new() });
            let img = self.tiny.render(&txt, true, if bonus != 0 { COL_GOOD } else { COL_FAINT });
            s.blr(&img, img.get_rect().center_at((cx, y + self.row_h / 2)));
        }
        y += self.row_h;
        draw::line(s, COL_BTN_BORDER, (self.name_x - 6, y), (self.b.width - 8, y), 1);
        let lbl = self.big.render(&t("kn.total"), true, COL_TEXT);
        s.blr(&lbl, lbl.get_rect().midleft_at((self.name_x, y + self.row_h / 2 + 2)));
        for p in 0..self.nplayers {
            let cx = self.val_x[p] + self.col_w / 2;
            let col = if self.b.multiplayer && p == 0 {
                COL_P1
            } else if self.b.multiplayer {
                COL_P2
            } else {
                COL_ACCENT
            };
            let img = self.big.render(&grand_total(&self.cards[p]).to_string(), true, col);
            s.blr(&img, img.get_rect().center_at((cx, y + self.row_h / 2 + 2)));
        }
    }

    fn draw_dice(&self, s: &mut Surface) {
        let w = self.b.width;
        draw::rect(s, COL_PANEL, (0, self.dice_y, w, self.dice_h), 0);
        draw::line(s, COL_BTN_BORDER, (0, self.dice_y), (w, self.dice_y), 1);
        let show_anim = self.state == Zustand::RollAnim;
        for (i, rc) in self.die_rects.iter().enumerate() {
            let val = if !self.rolled_once && self.state == Zustand::Ready {
                0
            } else if show_anim && !(self.rolled_once && self.held[i]) {
                py::randint(1, 6)
            } else {
                self.dice[i]
            };
            let held = self.held[i] && self.rolled_once;
            self.draw_die(s, *rc, val, held, i);
        }
        let can_roll = self.rolls_left > 0 && matches!(self.state, Zustand::Ready | Zustand::Rolled);
        draw::rect_r(s, if can_roll { COL_BTN_ON } else { COL_BTN }, self.roll_rect, 0, 10);
        draw::rect_r(s, if can_roll { COL_ACCENT } else { COL_BTN_BORDER }, self.roll_rect, if can_roll { 2 } else { 1 }, 10);
        let label = if self.state != Zustand::Ready || self.rolled_once { t("kn.roll") } else { t("kn.roll_start") };
        let img = self.small.render(&label, true, if can_roll { COL_TEXT } else { COL_FAINT });
        s.blr(&img, img.get_rect().center_at(self.roll_rect.center()));
    }

    fn draw_die(&self, s: &mut Surface, rc: Rect, val: i64, held: bool, idx: usize) {
        let base = if held { COL_DIE_HELD } else { COL_DIE };
        draw::rect_r(s, Color::rgb(10, 12, 18), rc.move_(0, 3), 0, 8);
        draw::rect_r(s, base, rc, 0, 8);
        if held {
            draw::rect_r(s, COL_ACCENT, rc, 3, 8);
        } else {
            draw::rect_r(s, Color::rgb(200, 202, 210), rc, 1, 8);
        }
        let num = self.tiny.render(&(idx + 1).to_string(), true, Color::rgb(170, 172, 180));
        s.bl(&num, (rc.x + 4, rc.y + 2));
        if val < 1 {
            return;
        }
        let r = 3.max(self.die / 10);
        let (cx, cy) = rc.center();
        let off = self.die / 4;
        let (l, c, rr) = (cx - off, cx, cx + off);
        let (tt, m, bb) = (cy - off, cy, cy + off);
        let pips: Vec<(i32, i32)> = match val {
            1 => vec![(c, m)],
            2 => vec![(l, tt), (rr, bb)],
            3 => vec![(l, tt), (c, m), (rr, bb)],
            4 => vec![(l, tt), (rr, tt), (l, bb), (rr, bb)],
            5 => vec![(l, tt), (rr, tt), (c, m), (l, bb), (rr, bb)],
            6 => vec![(l, tt), (rr, tt), (l, m), (rr, m), (l, bb), (rr, bb)],
            _ => vec![],
        };
        for p in pips {
            draw::circle(s, COL_PIP, p, r, 0);
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(Color::rgba(10, 12, 18, 180));
        s.bl(&ov, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        if self.b.multiplayer {
            let head = match self.winner {
                None => self.huge.render(&t("common.draw"), true, COL_DIM),
                Some(wn) => self.huge.render(
                    &tf("common.player_wins_name", &[("name", &self.b.pname(wn))]),
                    true,
                    if wn == 0 { COL_P1 } else { COL_P2 },
                ),
            };
            s.blr(&head, head.get_rect().center_at((cx, cy - 40)));
            let sub = self.small.render(&format!("{} : {}", grand_total(&self.cards[0]), grand_total(&self.cards[1])), true, COL_TEXT);
            s.blr(&sub, sub.get_rect().center_at((cx, cy)));
        } else {
            let head = self.huge.render(&t("kn.finished"), true, COL_ACCENT);
            s.blr(&head, head.get_rect().center_at((cx, cy - 40)));
            let sub = self.big.render(&tf("kn.your_score", &[("n", &grand_total(&self.cards[0]).to_string())]), true, COL_TEXT);
            s.blr(&sub, sub.get_rect().center_at((cx, cy + 2)));
        }
        let hint = self.tiny.render(&t("common.enter_restart"), true, COL_DIM);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 44)));
    }
}

impl Game for KniffelGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.new_round_result();
        self.b.score = 0;
        self.b.game_over = false;
        self.nplayers = if self.b.multiplayer { 2 } else { 1 };
        self.cards = vec![[None; 13]; self.nplayers];
        self.player = 0;
        self.dice = [1; 5];
        self.held = [false; 5];
        self.rolls_left = 3;
        self.rolled_once = false;
        self.anim_t = 0.0;
        self.final_ = self.dice;
        self.winner = None;
        self.sel = 0;
        self.make_fonts();
        self.layout();
        self.start_turn();
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == Zustand::Over {
            if ev.kind == Kind::MouseDown || ev.is_any(&["Return", "space"]) {
                self.b.game_over = false;
                self.reset();
            }
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.clone();
            self.handle_key(&k);
        } else if ev.kind == Kind::MouseDown {
            self.handle_click(ev.pos);
        }
    }

    fn update(&mut self, dt: f64) {
        if self.state == Zustand::RollAnim {
            self.anim_t -= dt;
            if self.anim_t <= 0.0 {
                self.dice = self.final_;
                self.rolls_left -= 1;
                self.rolled_once = true;
                self.state = Zustand::Rolled;
                self.b.play_sound("lock");
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background_ex(&mut s, self.b.width, self.b.height, false, Some(true));
        self.draw_hud(&mut s);
        self.draw_card(&mut s);
        self.draw_dice(&mut s);
        if self.state == Zustand::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "kniffel",
    class_name: "KniffelGame",
    supports_multiplayer: true,
    modes: &[],
    make: KniffelGame::new,
};
