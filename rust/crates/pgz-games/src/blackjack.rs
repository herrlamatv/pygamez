//! blackjack.rs
//! ============
//! Blackjack gegen den Dealer - mit Chips, Double Down und Split.
//!
//! Uebertragen aus `games/blackjack.py`: 4-Deck-Schuh, Dealer steht auf
//! allen 17, Peek bei Ass/Zehn, Blackjack 3:2, Split genau einmal, Karten
//! fliegen vom Schuh, Hole-Card-Flip. Chips aus der Lama-Bank
//! (`pgz_core::lamabank`), geteilt mit Poker und Casino.

use std::collections::HashSet;

use crate::cards::{self as c, Card, CardRenderer};
use pgz_core::lamabank;
use pgz_engine::prelude::*;

const COL_FELT: Color = Color::rgb(23, 46, 37);
const COL_FELT_EDGE: Color = Color::rgb(15, 33, 26);
const BETS: [i64; 4] = [10, 25, 50, 100];
const CHIP_COLS: [Color; 4] = [Color::rgb(110, 160, 235), Color::rgb(110, 205, 140), Color::rgb(230, 120, 90), Color::rgb(40, 40, 48)];
const DEAL_T: f64 = 0.22;
const FLIP_T: f64 = 0.25;
const DEALER_T: f64 = 0.5;
const ACTIONS: [&str; 4] = ["hit", "stand", "double", "split"];

#[derive(Clone, Copy, PartialEq)]
enum State {
    Bet,
    Dealing,
    Player,
    Dealer,
    Payout,
    Broke,
}

#[derive(Clone, Copy)]
struct Tween {
    dealer: bool,
    idx: usize,
    id: u64,
    t: f64,
    delay: f64,
    landed: bool,
}

/// `hand_value(cards)` -> (Summe, soft?)
fn hand_value(cards: &[Card]) -> (i64, bool) {
    let mut total = 0;
    let mut aces = 0;
    for c in cards {
        let mut v = (c.rank as i64).min(10);
        if c.rank == 1 {
            aces += 1;
            v = 11;
        }
        total += v;
    }
    while total > 21 && aces > 0 {
        total -= 10;
        aces -= 1;
    }
    (total, aces > 0)
}

/// `BlackjackGame`
pub struct BlackjackGame {
    b: GameBase,
    chips: i64,
    best: i64,
    small: Font,
    tiny: Font,
    big: Font,
    huge: Font,
    renderer: CardRenderer,
    ch: i32,
    cw: i32,
    shoe_pos: (i32, i32),
    dealer_y: i32,
    player_y: i32,
    strip: Rect,
    felt: Surface,
    chip_rects: Vec<Rect>,
    deal_rect: Rect,
    clear_rect: Rect,
    action_rects: Vec<Rect>,
    shoe: Vec<Card>,
    bet: i64,
    hands: Vec<Vec<Card>>,
    hand_bets: Vec<i64>,
    hand_done: Vec<bool>,
    active: usize,
    split_aces: bool,
    dealer: Vec<Card>,
    hole_hidden: bool,
    results: Vec<&'static str>,
    tweens: Vec<Tween>,
    fly: HashSet<u64>,
    dealer_wait: f64,
    flip_t: f64,
    state: State,
}

impl BlackjackGame {
    /// `BlackjackGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let accent = b.accent;
        let z = Rect::new(0, 0, 0, 0);
        let mut g = BlackjackGame {
            b,
            chips: 0,
            best: 0,
            small: f.clone(),
            tiny: f.clone(),
            big: f.clone(),
            huge: f,
            renderer: CardRenderer::new(accent),
            ch: 0,
            cw: 0,
            shoe_pos: (0, 0),
            dealer_y: 0,
            player_y: 0,
            strip: z,
            felt: pg::surface(1, 1),
            chip_rects: Vec::new(),
            deal_rect: z,
            clear_rect: z,
            action_rects: Vec::new(),
            shoe: Vec::new(),
            bet: 0,
            hands: Vec::new(),
            hand_bets: Vec::new(),
            hand_done: Vec::new(),
            active: 0,
            split_aces: false,
            dealer: Vec::new(),
            hole_hidden: true,
            results: Vec::new(),
            tweens: Vec::new(),
            fly: HashSet::new(),
            dealer_wait: 0.0,
            flip_t: 0.0,
            state: State::Bet,
        };
        g.reset();
        Box::new(g)
    }

    fn make_fonts(&mut self) {
        self.small = ui::f(16);
        self.tiny = ui::f(13);
        self.big = ui::fb(22);
        self.huge = ui::fb(26.max(self.b.height / 11));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.ch = (h as f64 * 0.20) as i32;
        self.cw = (self.ch as f64 * 0.72) as i32;
        self.shoe_pos = (w - self.cw - 16, 12);
        self.dealer_y = (h as f64 * 0.20) as i32;
        self.player_y = (h as f64 * 0.58) as i32;
        self.strip = Rect::new(0, h - 72, w, 72);
        self.felt = c::make_felt(w, h, COL_FELT, COL_FELT_EDGE);
        self.chip_rects = (0..4).map(|i| Rect::new(24 + i * 56, self.strip.y + 14, 44, 44)).collect();
        let bw = (w as f64 * 0.14) as i32;
        self.deal_rect = Rect::new(w - bw - 16, self.strip.y + 14, bw, 44);
        self.clear_rect = Rect::new(w - 2 * bw - 28, self.strip.y + 14, bw, 44);
        let bw2 = (w as f64 * 0.16) as i32;
        let gap = py::fdiv32(w - 4 * bw2, 5);
        self.action_rects = (0..4).map(|i| Rect::new(gap + i * (bw2 + gap), self.strip.y + 14, bw2, 44)).collect();
    }

    fn sync_chips(&mut self) {
        self.chips = lamabank::balance();
        self.best = lamabank::score_for("blackjack");
        self.b.score = self.best;
    }

    fn ensure_shoe(&mut self) {
        if self.shoe.len() < 52 {
            self.shoe = c::make_deck(&[0, 1, 2, 3], 4);
            let mut rng = py::Random::unseeded();
            rng.shuffle(&mut self.shoe);
        }
    }

    fn draw_card(&mut self, face_up: bool) -> Card {
        let mut card = self.shoe.pop().expect("shoe is never empty here");
        card.face_up = face_up;
        card
    }

    fn hand_positions(&self, hand_i: usize, n: usize) -> Vec<(i32, i32)> {
        let cx = if self.hands.len() == 1 { self.b.width / 2 } else { (self.b.width as f64 * if hand_i == 0 { 0.35 } else { 0.65 }) as i32 };
        let dx = (self.cw as f64 * 0.55) as i32;
        let x0 = cx - py::fdiv32(self.cw + (n as i32 - 1) * dx, 2);
        (0..n as i32).map(|i| (x0 + i * dx, self.player_y)).collect()
    }

    fn dealer_positions(&self, n: usize) -> Vec<(i32, i32)> {
        let dx = (self.cw as f64 * 0.55) as i32;
        let x0 = self.b.width / 2 - py::fdiv32(self.cw + (n as i32 - 1) * dx, 2);
        (0..n as i32).map(|i| (x0 + i * dx, self.dealer_y)).collect()
    }

    fn start_deal(&mut self) {
        if self.bet < BETS[0] || !lamabank::debit(self.bet, "blackjack") {
            return;
        }
        self.sync_chips();
        self.ensure_shoe();
        self.hands = vec![Vec::new()];
        self.hand_bets = vec![self.bet];
        self.hand_done = vec![false];
        self.active = 0;
        self.split_aces = false;
        self.dealer.clear();
        self.hole_hidden = true;
        self.results.clear();
        self.tweens.clear();
        self.state = State::Dealing;
        for (to_dealer, up) in [(false, true), (true, true), (false, true), (true, false)] {
            let card = self.draw_card(up);
            let idx = if to_dealer {
                self.dealer.push(card);
                self.dealer.len() - 1
            } else {
                self.hands[0].push(card);
                self.hands[0].len() - 1
            };
            let delay = self.tweens.len() as f64 * DEAL_T;
            self.tweens.push(Tween { dealer: to_dealer, idx, id: card.id, t: 0.0, delay, landed: false });
        }
        self.b.play_sound("move");
    }

    fn after_deal(&mut self) {
        let (pv, _) = hand_value(&self.hands[0]);
        let (dv, _) = hand_value(&self.dealer);
        let player_bj = pv == 21;
        let up = self.dealer[0];
        if (up.rank == 1 || (up.rank as i64).min(10) == 10) && dv == 21 {
            self.reveal_hole();
            self.settle(Some(&[if player_bj { "push" } else { "lose" }]));
            return;
        }
        if player_bj {
            self.reveal_hole();
            self.b.ach_event("blackjack_two", None);
            self.settle(Some(&["blackjack"]));
            return;
        }
        self.state = State::Player;
        self.b.play_sound("select");
    }

    fn can_double(&self) -> bool {
        let h = &self.hands[self.active];
        h.len() == 2 && !self.split_aces && self.chips >= self.hand_bets[self.active]
    }

    fn can_split(&self) -> bool {
        if self.hands.len() != 1 {
            return false;
        }
        let h = &self.hands[0];
        h.len() == 2 && (h[0].rank as i64).min(10) == (h[1].rank as i64).min(10) && self.chips >= self.hand_bets[0]
    }

    fn hit(&mut self) {
        let card = self.draw_card(true);
        self.hands[self.active].push(card);
        self.b.play_sound("move");
        if hand_value(&self.hands[self.active]).0 >= 21 {
            self.next_hand();
        }
    }

    fn double(&mut self) {
        if !self.can_double() || !lamabank::debit(self.hand_bets[self.active], "blackjack") {
            return;
        }
        self.sync_chips();
        self.hand_bets[self.active] *= 2;
        let card = self.draw_card(true);
        self.hands[self.active].push(card);
        self.b.play_sound("move");
        self.next_hand();
    }

    fn split(&mut self) {
        if !self.can_split() || !lamabank::debit(self.hand_bets[0], "blackjack") {
            return;
        }
        self.sync_chips();
        let h = self.hands[0].clone();
        self.split_aces = h[0].rank == 1;
        self.hands = vec![vec![h[0]], vec![h[1]]];
        self.hand_bets = vec![self.hand_bets[0], self.hand_bets[0]];
        self.hand_done = vec![false, false];
        self.active = 0;
        let c0 = self.draw_card(true);
        self.hands[0].push(c0);
        let c1 = self.draw_card(true);
        self.hands[1].push(c1);
        self.b.play_sound("rotate");
        if self.split_aces {
            self.hand_done = vec![true, true];
            self.start_dealer();
        }
    }

    fn next_hand(&mut self) {
        self.hand_done[self.active] = true;
        if self.active + 1 < self.hands.len() {
            self.active += 1;
            if hand_value(&self.hands[self.active]).0 >= 21 {
                self.next_hand();
            }
            return;
        }
        if self.hands.iter().all(|h| hand_value(h).0 > 21) {
            self.reveal_hole();
            self.settle(None);
        } else {
            self.start_dealer();
        }
    }

    fn reveal_hole(&mut self) {
        self.hole_hidden = false;
        for card in self.dealer.iter_mut() {
            card.face_up = true;
        }
    }

    fn start_dealer(&mut self) {
        self.state = State::Dealer;
        self.reveal_hole();
        self.flip_t = FLIP_T;
        self.dealer_wait = DEALER_T;
        self.b.play_sound("rotate");
    }

    fn dealer_step(&mut self) {
        if hand_value(&self.dealer).0 < 17 {
            let card = self.draw_card(true);
            self.dealer.push(card);
            self.b.play_sound("move");
            self.dealer_wait = DEALER_T;
        } else {
            self.settle(None);
        }
    }

    fn settle(&mut self, forced: Option<&[&'static str]>) {
        self.reveal_hole();
        let (dv, _) = hand_value(&self.dealer);
        let dealer_bust = dv > 21;
        self.results.clear();
        let mut delta = 0;
        for (i, h) in self.hands.iter().enumerate() {
            let bet = self.hand_bets[i];
            let res = match forced {
                Some(f) => f[i.min(f.len() - 1)],
                None => {
                    let (pv, _) = hand_value(h);
                    if pv > 21 {
                        "bust"
                    } else if dealer_bust {
                        "dealer_bust"
                    } else if pv > dv {
                        "win"
                    } else if pv < dv {
                        "lose"
                    } else {
                        "push"
                    }
                }
            };
            match res {
                "blackjack" => delta += bet + py::fdiv(bet * 3, 2),
                "win" | "dealer_bust" => delta += bet * 2,
                "push" => delta += bet,
                _ => {}
            }
            self.results.push(res);
        }
        lamabank::credit(delta, "blackjack", true);
        self.sync_chips();
        self.state = State::Payout;
        let r = &self.results;
        if r.contains(&"blackjack") {
            self.b.play_sound("win");
        } else if r.iter().any(|x| *x == "win" || *x == "dealer_bust") {
            self.b.play_sound("point");
        } else if r.iter().all(|x| *x == "push") {
            self.b.play_sound("select");
        } else {
            self.b.play_sound("gameover");
        }
    }

    fn to_bet(&mut self) {
        self.bet = 0;
        self.sync_chips();
        if lamabank::is_broke("blackjack", None) {
            self.state = State::Broke;
            self.b.play_sound("gameover");
        } else {
            self.state = State::Bet;
        }
    }

    fn handle_bet(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" | "4" => self.add_chip(BETS[k.parse::<usize>().unwrap_or(1) - 1]),
                "BackSpace" => {
                    self.bet = 0;
                    self.b.play_sound("move");
                }
                "Return" | "space" => self.start_deal(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.chip_rects.iter().position(|r| r.hit(ev.pos)) {
                self.add_chip(BETS[i]);
                return;
            }
            if self.clear_rect.hit(ev.pos) {
                self.bet = 0;
                self.b.play_sound("move");
            } else if self.deal_rect.hit(ev.pos) {
                self.start_deal();
            }
        }
    }

    fn add_chip(&mut self, value: i64) {
        if self.bet + value <= self.chips {
            self.bet += value;
            self.b.play_sound("click");
        }
    }

    fn handle_player(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "h" | "H" => self.hit(),
                "s" | "S" => self.next_hand(),
                "d" | "D" => self.double(),
                "x" | "X" => self.split(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.action_rects.iter().position(|r| r.hit(ev.pos)) {
                match ACTIONS[i] {
                    "hit" => self.hit(),
                    "stand" => self.next_hand(),
                    "double" if self.can_double() => self.double(),
                    "split" if self.can_split() => self.split(),
                    _ => {}
                }
            }
        }
    }

    fn card_img(&self, card: &Card) -> Surface {
        self.renderer.get(card, self.cw, self.ch)
    }

    fn draw_flying(&self, s: &mut Surface) {
        for tw in &self.tweens {
            let p = (tw.t - tw.delay) / DEAL_T;
            if p <= 0.0 || p >= 1.0 {
                continue;
            }
            let e = 1.0 - py::pow(1.0 - p, 2.0);
            let (tx, ty, card) = if tw.dealer {
                let pos = self.dealer_positions(self.dealer.len())[tw.idx];
                (pos.0, pos.1, self.dealer[tw.idx])
            } else {
                let pos = self.hand_positions(0, self.hands[0].len())[tw.idx];
                (pos.0, pos.1, self.hands[0][tw.idx])
            };
            let (sx, sy) = self.shoe_pos;
            let img = if card.face_up { self.card_img(&card) } else { self.renderer.back(self.cw, self.ch) };
            let x = (sx as f64 + (tx - sx) as f64 * e) as i32;
            let y = (sy as f64 + (ty - sy) as f64 * e) as i32;
            s.bl(&img, (x, y));
        }
    }

    fn draw_dealer(&self, s: &mut Surface) {
        let p = ui::pal();
        let lbl = self.small.render(&t("bj.dealer"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midbottom_at((self.b.width / 2, self.dealer_y - 8)));
        let pos = self.dealer_positions(self.dealer.len());
        let flipping = self.state == State::Dealer && self.flip_t > 0.0;
        for (i, card) in self.dealer.iter().enumerate() {
            if self.fly.contains(&card.id) {
                continue;
            }
            let (x, y) = pos[i];
            if i == 1 && flipping {
                let f = self.flip_t / FLIP_T;
                let w = 2.max((self.cw as f64 * (2.0 * f - 1.0).abs()) as i32);
                let img = if f > 0.5 { self.renderer.back(self.cw, self.ch) } else { self.card_img(card) };
                let img = transform::scale(&img, w, self.ch);
                s.bl(&img, (x + py::fdiv32(self.cw - w, 2), y));
            } else if i == 1 && self.hole_hidden {
                s.bl(&self.renderer.back(self.cw, self.ch), (x, y));
            } else {
                s.bl(&self.card_img(card), (x, y));
            }
        }
        if !self.dealer.is_empty() && !self.hole_hidden && !flipping {
            let (dv, _) = hand_value(&self.dealer);
            let img = self.big.render(&dv.to_string(), true, if dv > 21 { p.red } else { p.text });
            s.bl(&img, (pos[pos.len() - 1].0 + self.cw + 12, self.dealer_y + 4));
        }
    }

    fn draw_player(&self, s: &mut Surface) {
        let p = ui::pal();
        for (hi, h) in self.hands.iter().enumerate() {
            let pos = self.hand_positions(hi, h.len());
            for (i, card) in h.iter().enumerate() {
                if self.fly.contains(&card.id) {
                    continue;
                }
                s.bl(&self.card_img(card), pos[i]);
            }
            if !h.is_empty() && self.state != State::Dealing {
                let (total, soft) = hand_value(h);
                let txt = format!("{total}{}", if soft && total <= 21 { "s" } else { "" });
                let col = if total > 21 {
                    p.red
                } else if total == 21 {
                    p.gold
                } else {
                    p.text
                };
                let img = self.big.render(&txt, true, col);
                s.bl(&img, (pos[pos.len() - 1].0 + self.cw + 12, self.player_y + 4));
                let bet = self.tiny.render(&format!("{}: {}", t("bj.bet"), self.hand_bets[hi]), true, p.text_dim);
                s.bl(&bet, (pos[0].0, self.player_y + self.ch + 6));
            }
            if self.hands.len() > 1 && hi == self.active && self.state == State::Player {
                let k = ((pg::ticks() % 800) as i64 - 400).abs() as f64 / 400.0;
                let ax = pos[0].0 - 18;
                let ay = self.player_y + self.ch / 2 + (6.0 * k) as i32;
                draw::polygon(s, p.gold, &[(ax, ay - 8), (ax + 12, ay), (ax, ay + 8)], 0);
            }
        }
    }

    fn draw_topbar(&self, s: &mut Surface) {
        let p = ui::pal();
        let img = self.big.render(&format!("{}: {}", t("bj.chips"), self.chips), true, p.gold);
        s.bl(&img, (16, 12));
        let img = self.small.render(&format!("{}: {}", t("bj.best"), self.best), true, p.text_dim);
        s.bl(&img, (16, 40));
    }

    fn draw_strip(&self, s: &mut Surface) {
        let mut bg = pg::surface_alpha(self.strip.w, self.strip.h);
        bg.fill_c(rgba(8, 22, 16, 224));
        s.bl(&bg, (self.strip.x, self.strip.y));
        draw::line(s, self.b.accent, self.strip.topleft(), self.strip.topright(), 2);
    }

    fn draw_btn(&self, s: &mut Surface, rect: Rect, label: &str, primary: bool, on: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rect, 0, 10);
        if primary && on {
            let border = ui::mix(self.b.accent, (255, 255, 255), 0.25 * ui::pulse(2.2, 0.0, 1.0));
            draw::rect_r(s, border, rect, 2, 10);
        } else {
            draw::rect_r(s, p.border_light, rect, 1, 10);
        }
        let img = self.small.render(label, true, if on { p.text } else { p.text_dim });
        s.blr(&img, img.get_rect().center_at(rect.center()));
    }

    fn draw_bet_ui(&self, s: &mut Surface) {
        let p = ui::pal();
        self.draw_strip(s);
        for (i, r) in self.chip_rects.iter().enumerate() {
            let val = BETS[i];
            draw::circle(s, CHIP_COLS[i], r.center(), 22, 0);
            draw::circle(s, p.text, r.center(), 22, 2);
            draw::circle(s, p.text, r.center(), 15, 1);
            let img = self.tiny.render(&val.to_string(), true, if val != 100 { rgb(20, 24, 30) } else { p.text });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let mid = self.big.render(&format!("{}: {}", t("bj.bet"), self.bet), true, p.text);
        s.blr(&mid, mid.get_rect().center_at((self.b.width / 2, self.strip.centery())));
        self.draw_btn(s, self.clear_rect, &t("bj.clear"), false, self.bet > 0);
        self.draw_btn(s, self.deal_rect, &t("bj.deal"), true, self.bet >= BETS[0]);
        if self.bet < BETS[0] {
            let hint = self.tiny.render(&tf("bj.min_bet", &[("n", &BETS[0].to_string())]), true, p.text_dim);
            s.blr(&hint, hint.get_rect().midbottom_at((self.b.width / 2, self.strip.y - 6)));
        }
    }

    fn draw_action_ui(&self, s: &mut Surface) {
        self.draw_strip(s);
        let avail = [true, true, self.can_double(), self.can_split()];
        let keys = ["H", "S", "D", "X"];
        for (i, r) in self.action_rects.iter().enumerate() {
            self.draw_btn(s, *r, &format!("{} ({})", t(&format!("bj.{}", ACTIONS[i])), keys[i]), false, avail[i]);
        }
    }

    fn draw_results(&self, s: &mut Surface) {
        let p = ui::pal();
        self.draw_strip(s);
        let texts: Vec<String> = self
            .results
            .iter()
            .enumerate()
            .map(|(i, res)| {
                let label = t(&format!("bj.{res}"));
                if self.results.len() > 1 {
                    format!("{} {}: {label}", t("bj.hand"), i + 1)
                } else {
                    label
                }
            })
            .collect();
        let img = self.big.render(&texts.join("   \u{b7}   "), true, p.gold);
        s.blr(&img, img.get_rect().center_at((self.b.width / 2, self.strip.centery() - 10)));
        let hint = self.tiny.render(&t("common.enter_restart"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((self.b.width / 2, self.strip.bottom() - 14)));
    }

    fn draw_broke(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(rgba(6, 14, 10, 200));
        s.bl(&ov, (0, 0));
        let (cx, cy) = (w / 2, h / 2);
        let pw = (w - 60).min(460);
        let panel = Rect::new(cx - py::fdiv32(pw, 2), cy - 92, pw, 184);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(p.red));
        let head = self.huge.render(&t("bj.broke"), true, p.red);
        s.blr(&head, head.get_rect().center_at((cx, cy - 42)));
        let best = self.small.render(&format!("{}: {}", t("bj.best"), self.best), true, p.text_dim);
        s.blr(&best, best.get_rect().center_at((cx, cy + 2)));
        let sub = self.small.render(&tf("bj.broke_restart", &[("n", &lamabank::START_CHIPS.to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, cy + 34)));
    }
}

impl Game for BlackjackGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.game_over = false;
        lamabank::load();
        self.sync_chips();
        self.make_fonts();
        self.renderer = CardRenderer::new(self.b.accent);
        self.layout();
        self.shoe.clear();
        self.bet = 0;
        self.hands.clear();
        self.hand_bets.clear();
        self.hand_done.clear();
        self.active = 0;
        self.split_aces = false;
        self.dealer.clear();
        self.hole_hidden = true;
        self.results.clear();
        self.tweens.clear();
        self.fly.clear();
        self.dealer_wait = 0.0;
        self.flip_t = 0.0;
        self.state = if lamabank::is_broke("blackjack", None) { State::Broke } else { State::Bet };
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.renderer.clear();
        self.layout();
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        let weiter = ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space"]));
        match self.state {
            State::Broke => {
                if weiter {
                    lamabank::refill_if_broke("blackjack", None);
                    self.to_bet();
                    self.b.play_sound("click");
                }
            }
            State::Bet => self.handle_bet(ev),
            State::Player => self.handle_player(ev),
            State::Payout => {
                if weiter {
                    self.to_bet();
                }
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        match self.state {
            State::Dealing => {
                let mut done = true;
                let mut klick = 0;
                for tw in self.tweens.iter_mut() {
                    tw.t += dt;
                    if tw.t >= tw.delay + DEAL_T {
                        if !tw.landed {
                            tw.landed = true;
                            klick += 1;
                        }
                    } else {
                        done = false;
                    }
                }
                for _ in 0..klick {
                    self.b.play_sound("click");
                }
                if done {
                    self.tweens.clear();
                    self.after_deal();
                }
            }
            State::Dealer => {
                if self.flip_t > 0.0 {
                    self.flip_t -= dt;
                    return;
                }
                self.dealer_wait -= dt;
                if self.dealer_wait <= 0.0 {
                    self.dealer_step();
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let p = ui::pal();
        let mut s = self.b.sf();
        s.bl(&self.felt, (0, 0));
        s.bl(&self.renderer.back(self.cw, self.ch), self.shoe_pos);
        let n = self.tiny.render(&self.shoe.len().to_string(), true, p.text_dim);
        s.bl(&n, (self.shoe_pos.0, self.shoe_pos.1 + self.ch + 4));
        self.fly = if self.state == State::Dealing {
            self.tweens.iter().filter(|tw| tw.t < tw.delay + DEAL_T).map(|tw| tw.id).collect()
        } else {
            HashSet::new()
        };
        self.draw_dealer(&mut s);
        self.draw_player(&mut s);
        if !self.fly.is_empty() {
            self.draw_flying(&mut s);
        }
        self.draw_topbar(&mut s);
        match self.state {
            State::Bet => self.draw_bet_ui(&mut s),
            State::Player => self.draw_action_ui(&mut s),
            State::Payout => self.draw_results(&mut s),
            State::Broke => self.draw_broke(&mut s),
            _ => {}
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "blackjack",
    class_name: "BlackjackGame",
    supports_multiplayer: false,
    modes: &[],
    make: BlackjackGame::new,
};
