//! poker.rs
//! ========
//! Poker - Texas Hold'em, 5 Card Draw und Video Poker (Jacks or Better).
//!
//! Uebertragen aus `games/poker.py`: Blinds/Ante, Setzrunden mit KI in drei
//! Staerken, Kartentausch, Showdown mit 5-aus-7-Bewertung, Video-Poker-
//! Auszahlungstabelle. Chips aus der Lama-Bank; waehrend einer Hand liegt der
//! Kontostand als Tischstapel im escrow.
//!
//! Handwerte sind in Python verschachtelte Tupel (z. B. `(FLUSH, (14, 9, ...))`);
//! hier flach als `Vec<i64>` - innerhalb einer Kategorie ist die Form gleich,
//! der lexikographische Vergleich also derselbe.

use std::collections::HashSet;

use crate::cards::{self as c, Card, CardRenderer};
use pgz_core::lamabank;
use pgz_engine::prelude::*;

const COL_FELT: Color = Color::rgb(20, 50, 39);
const COL_FELT_EDGE: Color = Color::rgb(13, 35, 27);
const SMALL_BLIND: i64 = 10;
const BIG_BLIND: i64 = 20;
const ANTE: i64 = 10;
const VIDEO_BETS: [i64; 3] = [10, 25, 50];

const HIGH: i64 = 0;
const PAIR: i64 = 1;
const TWO_PAIR: i64 = 2;
const TRIPS: i64 = 3;
const STRAIGHT: i64 = 4;
const FLUSH: i64 = 5;
const FULL_HOUSE: i64 = 6;
const QUADS: i64 = 7;
const STR_FLUSH: i64 = 8;

fn cat_key(c: i64) -> &'static str {
    ["high_card", "pair", "two_pair", "trips", "straight", "flush", "full_house", "quads", "straight_flush"][c as usize]
}

const VIDEO_PAYOUT: [(i64, &str, i64); 9] = [
    (STR_FLUSH, "royal", 250),
    (STR_FLUSH, "", 50),
    (QUADS, "", 25),
    (FULL_HOUSE, "", 9),
    (FLUSH, "", 6),
    (STRAIGHT, "", 4),
    (TRIPS, "", 3),
    (TWO_PAIR, "", 2),
    (PAIR, "jacks", 1),
];

fn val(card: &Card) -> i64 {
    if card.rank == 1 {
        14
    } else {
        card.rank as i64
    }
}

/// `eval5(cards)` - flaches Rangtupel (hoeher = besser).
fn eval5(cards: &[Card]) -> Vec<i64> {
    let mut vals: Vec<i64> = cards.iter().map(val).collect();
    vals.sort_by(|a, b| b.cmp(a));
    let mut counts: Vec<(i64, i64)> = Vec::new();
    for &v in &vals {
        match counts.iter_mut().find(|c| c.0 == v) {
            Some(c) => c.1 += 1,
            None => counts.push((v, 1)),
        }
    }
    let mut by_count = counts.clone();
    by_count.sort_by(|a, b| (b.1, b.0).cmp(&(a.1, a.0)));
    let is_flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let mut uniq = vals.clone();
    uniq.dedup();
    let mut straight_high = None;
    if uniq.len() == 5 {
        if uniq[0] - uniq[4] == 4 {
            straight_high = Some(uniq[0]);
        } else if uniq == [14, 5, 4, 3, 2] {
            straight_high = Some(5);
        }
    }
    if let (true, Some(sh)) = (is_flush, straight_high) {
        return vec![STR_FLUSH, sh];
    }
    if by_count[0].1 == 4 {
        let quad = by_count[0].0;
        let kicker = vals.iter().copied().filter(|v| *v != quad).max().unwrap_or(0);
        return vec![QUADS, quad, kicker];
    }
    if by_count[0].1 == 3 && by_count[1].1 == 2 {
        return vec![FULL_HOUSE, by_count[0].0, by_count[1].0];
    }
    if is_flush {
        let mut r = vec![FLUSH];
        r.extend(&vals);
        return r;
    }
    if let Some(sh) = straight_high {
        return vec![STRAIGHT, sh];
    }
    if by_count[0].1 == 3 {
        let mut r = vec![TRIPS, by_count[0].0];
        r.extend(vals.iter().filter(|v| **v != by_count[0].0));
        return r;
    }
    if by_count[0].1 == 2 && by_count[1].1 == 2 {
        let (hp, lp) = (by_count[0].0, by_count[1].0);
        let kicker = vals.iter().copied().filter(|v| *v != hp && *v != lp).max().unwrap_or(0);
        return vec![TWO_PAIR, hp, lp, kicker];
    }
    if by_count[0].1 == 2 {
        let mut r = vec![PAIR, by_count[0].0];
        r.extend(vals.iter().filter(|v| **v != by_count[0].0));
        return r;
    }
    let mut r = vec![HIGH];
    r.extend(&vals);
    r
}

/// `best_hand(cards)` - beste Bewertung aus 5..7 Karten.
fn best_hand(cards: &[Card]) -> Vec<i64> {
    if cards.len() <= 5 {
        return eval5(cards);
    }
    let n = cards.len();
    let mut best: Option<Vec<i64>> = None;
    let mut idx = [0usize, 1, 2, 3, 4];
    loop {
        let combo: Vec<Card> = idx.iter().map(|&i| cards[i]).collect();
        let r = eval5(&combo);
        if best.as_ref().is_none_or(|b| r > *b) {
            best = Some(r);
        }
        // naechste Kombination in itertools-Reihenfolge
        let mut i = 5;
        loop {
            if i == 0 {
                return best.unwrap_or_default();
            }
            i -= 1;
            if idx[i] != i + n - 5 {
                break;
            }
            if i == 0 {
                return best.unwrap_or_default();
            }
        }
        idx[i] += 1;
        for j in i + 1..5 {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

fn is_royal(rt: &[i64]) -> bool {
    rt[0] == STR_FLUSH && rt[1] == 14
}

fn category_key(rt: &[i64]) -> &'static str {
    if is_royal(rt) {
        "royal"
    } else {
        cat_key(rt[0])
    }
}

fn hand_strength(cards: &[Card]) -> f64 {
    let rt = best_hand(cards);
    let top = rt[1];
    1.0f64.min(rt[0] as f64 / 8.0 * 0.82 + (top as f64 / 14.0) * 0.18)
}

fn preflop_strength(hole: &[Card]) -> f64 {
    let (a, b) = (hole[0], hole[1]);
    let (va, vb) = (val(&a), val(&b));
    let (hi, lo) = (va.max(vb), va.min(vb));
    let mut score = hi as f64 / 14.0 * 0.5;
    if va == vb {
        score = 0.5 + (va as f64 / 14.0) * 0.5;
    } else {
        if a.suit == b.suit {
            score += 0.12;
        }
        let gap = hi - lo;
        if gap == 1 {
            score += 0.08;
        } else if gap == 2 {
            score += 0.04;
        }
        score += lo as f64 / 14.0 * 0.15;
    }
    0f64.max(1f64.min(score))
}

#[derive(Clone)]
struct Player {
    name: String,
    stack: i64,
    hole: Vec<Card>,
    folded: bool,
    all_in: bool,
    round_bet: i64,
    is_human: bool,
    result: Option<Vec<i64>>,
}

impl Player {
    fn new(name: String, stack: i64, is_human: bool) -> Player {
        Player { name, stack, hole: Vec::new(), folded: false, all_in: false, round_bet: 0, is_human, result: None }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    PreHand,
    BetVideo,
    Acting,
    DrawSelect,
    VideoHold,
    Showdown,
    Broke,
}

const ACTIONS: [&str; 4] = ["fold", "call", "raise", "allin"];

/// `PokerGame`
pub struct PokerGame {
    b: GameBase,
    chips: i64,
    best: i64,
    n_opponents: usize,
    ai_level: usize,
    small: Font,
    tiny: Font,
    big: Font,
    huge: Font,
    renderer: CardRenderer,
    deck: Vec<Card>,
    community: Vec<Card>,
    pot: i64,
    players: Vec<Player>,
    dealer: usize,
    turn: usize,
    current_bet: i64,
    min_raise: i64,
    to_act: HashSet<usize>,
    street: &'static str,
    msg: String,
    act_delay: f64,
    reveal: bool,
    holds: HashSet<usize>,
    video_bet: i64,
    win_amount: i64,
    phase: Phase,
    ch: i32,
    cw: i32,
    strip: Rect,
    felt: Surface,
    action_rects: Vec<Rect>,
    deal_rect: Rect,
    chip_rects: Vec<Rect>,
}

impl PokerGame {
    /// `PokerGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let accent = b.accent;
        let z = Rect::new(0, 0, 0, 0);
        let mut g = PokerGame {
            b,
            chips: 0,
            best: 0,
            n_opponents: 2,
            ai_level: 1,
            small: f.clone(),
            tiny: f.clone(),
            big: f.clone(),
            huge: f,
            renderer: CardRenderer::new(accent),
            deck: Vec::new(),
            community: Vec::new(),
            pot: 0,
            players: Vec::new(),
            dealer: 0,
            turn: 0,
            current_bet: 0,
            min_raise: BIG_BLIND,
            to_act: HashSet::new(),
            street: "preflop",
            msg: String::new(),
            act_delay: 0.0,
            reveal: false,
            holds: HashSet::new(),
            video_bet: VIDEO_BETS[0],
            win_amount: 0,
            phase: Phase::PreHand,
            ch: 0,
            cw: 0,
            strip: z,
            felt: pg::surface(1, 1),
            action_rects: Vec::new(),
            deal_rect: z,
            chip_rects: Vec::new(),
        };
        g.reset();
        Box::new(g)
    }

    fn mode(&self) -> &str {
        self.b.mode.as_str()
    }

    fn make_fonts(&mut self) {
        self.small = ui::f(15);
        self.tiny = ui::f(13);
        self.big = ui::fb(22);
        self.huge = ui::fb(24.max(self.b.height / 12));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.ch = (h as f64 * 0.19) as i32;
        self.cw = (self.ch as f64 * 0.72) as i32;
        self.strip = Rect::new(0, h - 66, w, 66);
        self.felt = c::make_felt(w, h, COL_FELT, COL_FELT_EDGE);
        let bw = (w as f64 * 0.17) as i32;
        let gap = py::fdiv32(w - 4 * bw, 5);
        self.action_rects = (0..4).map(|i| Rect::new(gap + i * (bw + gap), self.strip.y + 12, bw, 42)).collect();
        self.deal_rect = Rect::new(w / 2 - 90, self.strip.y + 12, 180, 42);
        self.chip_rects = (0..VIDEO_BETS.len() as i32).map(|i| Rect::new(24 + i * 54, self.strip.y + 12, 42, 42)).collect();
    }

    fn sync_chips(&mut self) {
        self.chips = lamabank::balance();
        self.best = lamabank::score_for("poker");
        self.b.score = self.best;
        self.b.ach_event("poker_rich", Some(lamabank::value_for("poker")));
    }

    fn fresh_deck(&mut self) {
        self.deck = c::make_deck(&[0, 1, 2, 3], 1);
        let mut rng = py::Random::unseeded();
        rng.shuffle(&mut self.deck);
    }

    fn draw_card(&mut self, face_up: bool) -> Card {
        let mut card = self.deck.pop().expect("deck is never empty here");
        card.face_up = face_up;
        card
    }

    fn broke(&self) -> bool {
        lamabank::is_broke("poker", Some(self.mode()))
    }

    fn start_hand(&mut self) {
        lamabank::clear_escrow(true);
        self.chips = lamabank::balance();
        if self.broke() {
            self.phase = Phase::Broke;
            self.b.play_sound("gameover");
            return;
        }
        self.fresh_deck();
        self.community.clear();
        self.pot = 0;
        self.reveal = false;
        self.holds.clear();
        self.msg.clear();
        self.win_amount = 0;
        let mut names = vec![t("poker.you")];
        for i in 0..self.n_opponents {
            names.push(tf("poker.cpu", &[("n", &(i + 1).to_string())]));
        }
        let mut stacks = vec![self.chips];
        for _ in 0..self.n_opponents {
            stacks.push((BIG_BLIND * 20).max(self.chips));
        }
        self.players = (0..=self.n_opponents).map(|i| Player::new(names[i].clone(), stacks[i], i == 0)).collect();
        lamabank::set_escrow(self.chips);
        match self.mode() {
            "holdem" => self.start_holdem(),
            "draw" => self.start_draw(),
            _ => {}
        }
    }

    fn all_not_allin(&self) -> HashSet<usize> {
        (0..self.players.len()).filter(|&i| !self.players[i].all_in).collect()
    }

    fn start_holdem(&mut self) {
        let n = self.players.len();
        self.dealer %= n;
        for _ in 0..2 {
            for i in 0..n {
                let up = self.players[i].is_human;
                let card = self.draw_card(up);
                self.players[i].hole.push(card);
            }
        }
        let (mut sb, mut bb) = ((self.dealer + 1) % n, (self.dealer + 2) % n);
        if n == 2 {
            sb = self.dealer;
            bb = (self.dealer + 1) % n;
        }
        self.post(sb, SMALL_BLIND);
        self.post(bb, BIG_BLIND);
        self.current_bet = BIG_BLIND;
        self.min_raise = BIG_BLIND;
        self.street = "preflop";
        self.to_act = self.all_not_allin();
        self.turn = (bb + 1) % n;
        self.phase = Phase::Acting;
        self.msg = t("poker.preflop");
        self.b.play_sound("move");
        self.begin_turn();
    }

    fn post(&mut self, pi: usize, amount: i64) {
        let p = &mut self.players[pi];
        let amount = amount.min(p.stack);
        if p.is_human && amount > 0 {
            lamabank::pay_from_escrow(amount, "poker");
        }
        p.stack -= amount;
        p.round_bet += amount;
        self.pot += amount;
        if p.stack == 0 {
            p.all_in = true;
        }
    }

    fn start_draw(&mut self) {
        for i in 0..self.players.len() {
            self.post(i, ANTE);
        }
        for _ in 0..5 {
            for i in 0..self.players.len() {
                let up = self.players[i].is_human;
                let card = self.draw_card(up);
                self.players[i].hole.push(card);
            }
        }
        self.current_bet = 0;
        self.min_raise = BIG_BLIND;
        self.street = "draw1";
        self.to_act = self.all_not_allin();
        self.turn = (self.dealer + 1) % self.players.len();
        self.phase = Phase::Acting;
        self.msg = t("poker.round1");
        self.b.play_sound("move");
        self.begin_turn();
    }

    fn contenders(&self) -> usize {
        self.players.iter().filter(|p| !p.folded).count()
    }

    fn begin_turn(&mut self) {
        if self.contenders() <= 1 {
            self.end_hand();
            return;
        }
        if self.to_act.is_empty() {
            self.end_round();
            return;
        }
        if !self.to_act.contains(&self.turn) {
            self.advance_turn();
            return;
        }
        self.act_delay = if self.players[self.turn].is_human { 0.0 } else { 0.7 };
    }

    fn advance_turn(&mut self) {
        let n = self.players.len();
        if self.to_act.is_empty() {
            self.end_round();
            return;
        }
        for i in 1..=n {
            let cand = (self.turn + i) % n;
            if self.to_act.contains(&cand) {
                self.turn = cand;
                self.begin_turn();
                return;
            }
        }
        self.end_round();
    }

    fn to_call(&self, pi: usize) -> i64 {
        0.max(self.current_bet - self.players[pi].round_bet)
    }

    fn others_to_act(&self) -> HashSet<usize> {
        (0..self.players.len()).filter(|&i| !self.players[i].folded && !self.players[i].all_in && i != self.turn).collect()
    }

    fn do_action(&mut self, action: &str, raise_to: Option<i64>) {
        let ti = self.turn;
        match action {
            "fold" => {
                self.players[ti].folded = true;
                self.to_act.remove(&ti);
                self.b.play_sound("select");
            }
            "check" => {
                self.to_act.remove(&ti);
                self.b.play_sound("click");
            }
            "call" => {
                let need = self.to_call(ti).min(self.players[ti].stack);
                self.post(ti, need);
                self.to_act.remove(&ti);
                self.b.play_sound("point");
            }
            "raise" => {
                let p = &self.players[ti];
                let target = raise_to.unwrap_or(self.current_bet + self.min_raise).min(p.round_bet + p.stack);
                let rb = p.round_bet;
                self.post(ti, target - rb);
                self.min_raise = self.min_raise.max(target - self.current_bet);
                self.current_bet = self.current_bet.max(self.players[ti].round_bet);
                self.to_act = self.others_to_act();
                self.b.play_sound("point");
            }
            _ => {
                let st = self.players[ti].stack;
                self.post(ti, st);
                let rb = self.players[ti].round_bet;
                if rb > self.current_bet {
                    self.min_raise = self.min_raise.max(rb - self.current_bet);
                    self.current_bet = rb;
                    self.to_act = self.others_to_act();
                } else {
                    self.to_act.remove(&ti);
                }
                self.b.play_sound("point");
            }
        }
        self.advance_turn();
    }

    fn end_round(&mut self) {
        for p in self.players.iter_mut() {
            p.round_bet = 0;
        }
        self.current_bet = 0;
        self.min_raise = BIG_BLIND;
        if self.contenders() <= 1 {
            self.end_hand();
            return;
        }
        if self.mode() == "holdem" {
            self.next_street();
        } else if self.street == "draw1" {
            self.enter_draw_phase();
        } else {
            self.showdown();
        }
    }

    fn next_street(&mut self) {
        let actionable = self.players.iter().filter(|p| !p.folded && !p.all_in).count();
        let fast = actionable <= 1;
        match self.street {
            "preflop" => {
                self.community = (0..3).map(|_| self.draw_card(true)).collect();
                self.street = "flop";
                self.msg = t("poker.flop");
            }
            "flop" => {
                let c = self.draw_card(true);
                self.community.push(c);
                self.street = "turn";
                self.msg = t("poker.turn");
            }
            "turn" => {
                let c = self.draw_card(true);
                self.community.push(c);
                self.street = "river";
                self.msg = t("poker.river");
            }
            _ => {
                self.showdown();
                return;
            }
        }
        self.b.play_sound("move");
        if fast {
            if self.street != "river" {
                self.next_street();
            } else {
                self.showdown();
            }
            return;
        }
        self.to_act = (0..self.players.len()).filter(|&i| !self.players[i].folded && !self.players[i].all_in).collect();
        self.turn = self.dealer;
        self.advance_turn();
    }

    fn enter_draw_phase(&mut self) {
        self.phase = Phase::DrawSelect;
        self.holds.clear();
        self.msg = t("poker.draw_hint");
        self.act_delay = 0.0;
    }

    fn do_human_draw(&mut self) {
        let hole = self.players[0].hole.clone();
        let mut new_hole = Vec::new();
        for (i, card) in hole.into_iter().enumerate() {
            if self.holds.contains(&i) {
                new_hole.push(card);
            } else {
                let nc = self.draw_card(true);
                new_hole.push(nc);
            }
        }
        self.players[0].hole = new_hole;
        self.holds.clear();
        self.b.play_sound("rotate");
        for pi in 1..self.players.len() {
            if !self.players[pi].folded {
                self.ai_draw(pi);
            }
        }
        self.current_bet = 0;
        self.min_raise = BIG_BLIND;
        self.street = "draw2";
        self.to_act = (0..self.players.len()).filter(|&i| !self.players[i].folded && !self.players[i].all_in).collect();
        self.turn = (self.dealer + 1) % self.players.len();
        self.phase = Phase::Acting;
        self.msg = t("poker.round2");
        self.begin_turn();
    }

    fn ai_draw(&mut self, pi: usize) {
        let hole = self.players[pi].hole.clone();
        let vals: Vec<i64> = hole.iter().map(val).collect();
        let count_v = |v: i64| vals.iter().filter(|x| **x == v).count();
        let mut suits: Vec<(u8, usize)> = Vec::new();
        for c in &hole {
            match suits.iter_mut().find(|s| s.0 == c.suit) {
                Some(s) => s.1 += 1,
                None => suits.push((c.suit, 1)),
            }
        }
        let mut keep: HashSet<usize> = (0..hole.len()).filter(|&i| count_v(val(&hole[i])) >= 2).collect();
        // most_common(1): erstes Element mit der hoechsten Anzahl
        let fs = suits.iter().fold(None::<(u8, usize)>, |acc, s| match acc {
            Some(a) if a.1 >= s.1 => Some(a),
            _ => Some(*s),
        });
        if let Some(fs) = fs {
            if fs.1 >= 4 {
                keep = (0..hole.len()).filter(|&i| hole[i].suit == fs.0).collect();
            }
        }
        if keep.is_empty() {
            keep = (0..hole.len()).filter(|&i| val(&hole[i]) >= 12).collect();
        }
        let mut new = Vec::new();
        for (i, c) in hole.into_iter().enumerate() {
            if keep.contains(&i) {
                new.push(c);
            } else {
                let nc = self.draw_card(false);
                new.push(nc);
            }
        }
        self.players[pi].hole = new;
    }

    fn ai_action(&mut self, pi: usize) {
        let to_call = self.to_call(pi);
        let p = self.players[pi].clone();
        let strength = if self.mode() == "holdem" {
            if self.street == "preflop" && self.community.is_empty() {
                preflop_strength(&p.hole)
            } else {
                let mut all = p.hole.clone();
                all.extend(&self.community);
                hand_strength(&all)
            }
        } else {
            hand_strength(&p.hole)
        };
        let aggro = [0.05, 0.14, 0.22][self.ai_level];
        let bluff = [0.03, 0.06, 0.10][self.ai_level];
        let r = py::random();
        if to_call == 0 {
            if strength > 0.6 || r < aggro {
                self.ai_raise(pi);
            } else {
                self.do_action("check", None);
            }
            return;
        }
        let pot_odds = to_call as f64 / (self.pot as f64 + to_call as f64 + 1e-9);
        if strength < 0.28 && r > bluff {
            if to_call as f64 > p.stack as f64 * 0.12 {
                self.do_action("fold", None);
            } else {
                self.do_action("call", None);
            }
            return;
        }
        if strength > 0.72 && r < 0.5 + aggro {
            self.ai_raise(pi);
            return;
        }
        if strength < pot_odds - 0.05 && r > bluff {
            self.do_action("fold", None);
            return;
        }
        self.do_action("call", None);
    }

    fn ai_raise(&mut self, pi: usize) {
        let pot_part = (self.pot as f64 * 0.6) as i64;
        let raise_to = self.current_bet + self.min_raise.max(if pot_part != 0 { pot_part } else { BIG_BLIND });
        let p = &self.players[pi];
        if raise_to >= p.round_bet + p.stack {
            self.do_action("allin", None);
        } else {
            self.do_action("raise", Some(raise_to));
        }
    }

    fn showdown(&mut self) {
        self.reveal = true;
        for p in self.players.iter_mut() {
            for c in p.hole.iter_mut() {
                c.face_up = true;
            }
        }
        let holdem = self.mode() == "holdem";
        let mut ranked: Vec<(Vec<i64>, usize)> = Vec::new();
        for i in 0..self.players.len() {
            if self.players[i].folded {
                continue;
            }
            let mut cards = self.players[i].hole.clone();
            if holdem {
                cards.extend(&self.community);
            }
            let rt = best_hand(&cards);
            self.players[i].result = Some(rt.clone());
            ranked.push((rt, i));
        }
        let best_rt = ranked.iter().map(|r| r.0.clone()).max().unwrap_or_default();
        let winners: Vec<usize> = ranked.iter().filter(|r| r.0 == best_rt).map(|r| r.1).collect();
        self.award(&winners, Some(best_rt));
        self.phase = Phase::Showdown;
    }

    fn end_hand(&mut self) {
        let winners: Vec<usize> = (0..self.players.len()).filter(|&i| !self.players[i].folded).collect();
        self.award(&winners, None);
        self.phase = Phase::Showdown;
    }

    fn award(&mut self, winners: &[usize], rank: Option<Vec<i64>>) {
        let share = py::fdiv(self.pot, winners.len() as i64);
        let rem = self.pot - share * winners.len() as i64;
        let mut won = 0;
        for (i, &w) in winners.iter().enumerate() {
            let add = share + if i == 0 { rem } else { 0 };
            self.players[w].stack += add;
            if self.players[w].is_human {
                won = add;
            }
        }
        lamabank::settle_escrow(won, "poker");
        let me_wins = winners.contains(&0);
        self.win_amount = if me_wins { self.pot } else { 0 };
        if me_wins {
            self.msg = match &rank {
                Some(rt) => tf("poker.you_win_hand", &[("hand", &t(&format!("poker.hand.{}", category_key(rt))))]),
                None => t("poker.you_win"),
            };
            self.b.play_sound("win");
        } else {
            self.msg = t("poker.you_lose");
            self.b.play_sound("gameover");
        }
        self.reveal = true;
        self.dealer = (self.dealer + 1) % 1.max(self.players.len());
        self.sync_chips();
    }

    fn video_deal(&mut self) {
        if !lamabank::debit(self.video_bet, "poker") {
            if lamabank::is_broke("poker", Some("video")) {
                self.phase = Phase::Broke;
                self.b.play_sound("gameover");
            }
            return;
        }
        self.chips = lamabank::balance();
        self.fresh_deck();
        let mut me = Player::new(t("poker.you"), self.chips, true);
        me.hole = (0..5).map(|_| self.draw_card(true)).collect();
        self.players = vec![me];
        self.holds.clear();
        self.phase = Phase::VideoHold;
        self.msg = t("poker.hold_hint");
        self.win_amount = 0;
        self.b.play_sound("move");
    }

    fn video_draw(&mut self) {
        let hole = self.players[0].hole.clone();
        let mut new = Vec::new();
        for (i, c) in hole.into_iter().enumerate() {
            if self.holds.contains(&i) {
                new.push(c);
            } else {
                let nc = self.draw_card(true);
                new.push(nc);
            }
        }
        self.players[0].hole = new;
        let rt = best_hand(&self.players[0].hole);
        let mult = video_payout(&rt);
        self.win_amount = self.video_bet * mult;
        lamabank::credit(self.win_amount, "poker", true);
        self.sync_chips();
        if mult > 0 {
            self.msg = tf("poker.video_win", &[("hand", &t(&format!("poker.hand.{}", category_key(&rt)))), ("mult", &mult.to_string())]);
            self.b.play_sound(if mult >= 6 { "win" } else { "point" });
        } else {
            self.msg = t("poker.video_none");
            self.b.play_sound("gameover");
        }
        self.phase = Phase::Showdown;
    }

    fn is_confirm(ev: &InputEvent) -> bool {
        ev.kind == Kind::MouseDown || (ev.kind == Kind::KeyDown && ev.key_in(&["Return", "space"]))
    }

    fn next_after_showdown(&mut self) {
        for p in self.players.iter_mut() {
            p.hole.clear();
            p.folded = false;
            p.all_in = false;
            p.round_bet = 0;
            p.result = None;
        }
        self.chips = lamabank::balance();
        self.phase = if self.broke() {
            Phase::Broke
        } else if self.mode() == "video" {
            Phase::BetVideo
        } else {
            Phase::PreHand
        };
        if self.phase == Phase::Broke {
            self.b.play_sound("gameover");
        }
    }

    fn handle_video_bet(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if matches!(k, "1" | "2" | "3") {
                self.video_bet = VIDEO_BETS[k.parse::<usize>().unwrap_or(1) - 1];
                self.b.play_sound("click");
            } else if matches!(k, "Return" | "space") {
                self.video_deal();
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.chip_rects.iter().position(|r| r.hit(ev.pos)) {
                self.video_bet = VIDEO_BETS[i];
                self.b.play_sound("click");
                return;
            }
            if self.deal_rect.hit(ev.pos) {
                self.video_deal();
            }
        }
    }

    fn toggle_hold(&mut self, i: usize) {
        if !self.holds.remove(&i) {
            self.holds.insert(i);
        }
    }

    fn handle_holds(&mut self, ev: &InputEvent, video: bool) {
        let n = self.players[0].hole.len();
        let mut confirm = false;
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if matches!(k, "1" | "2" | "3" | "4" | "5") {
                let i = k.parse::<usize>().unwrap_or(1) - 1;
                if i < n {
                    self.toggle_hold(i);
                    self.b.play_sound("click");
                }
            } else if matches!(k, "Return" | "space") {
                confirm = true;
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.hole_rects(0, false).iter().position(|r| r.hit(ev.pos)) {
                self.toggle_hold(i);
                self.b.play_sound("click");
                return;
            }
            if self.deal_rect.hit(ev.pos) {
                confirm = true;
            }
        }
        if confirm {
            if video {
                self.video_draw();
            } else {
                self.do_human_draw();
            }
        }
    }

    fn human_raise(&mut self) {
        let pot_part = (self.pot as f64 * 0.5) as i64;
        let raise_to = self.current_bet + self.min_raise.max(if pot_part != 0 { pot_part } else { BIG_BLIND });
        let p = &self.players[self.turn];
        if raise_to >= p.round_bet + p.stack {
            self.do_action("allin", None);
        } else {
            self.do_action("raise", Some(raise_to));
        }
    }

    fn handle_acting(&mut self, ev: &InputEvent) {
        if !self.players[self.turn].is_human {
            return;
        }
        let to_call = self.to_call(self.turn);
        let call = if to_call == 0 { "check" } else { "call" };
        let act = if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                "f" | "F" => Some("fold"),
                "c" | "C" => Some(call),
                "r" | "R" => Some("raise"),
                "a" | "A" => Some("allin"),
                _ => None,
            }
        } else if ev.kind == Kind::MouseDown {
            self.action_rects.iter().position(|r| r.hit(ev.pos)).map(|i| match ACTIONS[i] {
                "call" => call,
                x => x,
            })
        } else {
            None
        };
        match act {
            Some("raise") => self.human_raise(),
            Some(a) => self.do_action(a, None),
            None => {}
        }
    }

    // ------------------------------------------------- Zeichnen

    fn hud_chips(&self) -> i64 {
        if self.mode() != "video" && !self.players.is_empty() && matches!(self.phase, Phase::Acting | Phase::DrawSelect) {
            return self.players[0].stack;
        }
        self.chips
    }

    fn draw_topbar(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let label = format!("{}: {}", t("poker.chips"), self.hud_chips());
        let mut img = self.big.render(&label, true, p.gold);
        if img.w() > w / 2 - 90 {
            img = self.small.render(&label, true, p.gold);
        }
        s.bl(&img, (16, 12));
        let img = self.small.render(&format!("{}: {}", t("poker.best"), self.best), true, p.text_dim);
        s.bl(&img, (16, 40));
        let mi = self.small.render(&t(&format!("poker.mode.{}", self.mode())), true, self.b.accent);
        s.blr(&mi, mi.get_rect().topright_at((w - 16, 14)));
        if self.pot != 0 && !matches!(self.phase, Phase::BetVideo | Phase::PreHand) {
            let pi = self.big.render(&format!("{}: {}", t("poker.pot"), self.pot), true, p.gold);
            s.blr(&pi, pi.get_rect().center_at((w / 2, 24)));
        }
    }

    fn hole_rects(&self, pi: usize, center: bool) -> Vec<Rect> {
        let n = self.players.get(pi).map(|p| p.hole.len()).unwrap_or(0) as i32;
        if n == 0 {
            return Vec::new();
        }
        let dx = (self.cw as f64 * 1.12) as i32;
        let total = self.cw + (n - 1) * dx;
        let cx = self.b.width / 2;
        let y = if center || self.mode() == "video" { (self.b.height as f64 * 0.62) as i32 } else { (self.b.height as f64 * 0.60) as i32 };
        let x0 = cx - py::fdiv32(total, 2);
        (0..n).map(|i| Rect::new(x0 + i * dx, y, self.cw, self.ch)).collect()
    }

    fn draw_table(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let opp = &self.players[1..];
        for (i, pl) in opp.iter().enumerate() {
            let cx = (w as f64 * (i + 1) as f64 / (opp.len() + 1) as f64) as i32;
            self.draw_opponent(s, pl, cx, (h as f64 * 0.20) as i32, i + 1 == self.turn && self.phase == Phase::Acting);
        }
        if !self.community.is_empty() {
            self.draw_community(s);
        }
        let me = &self.players[0];
        let highlight = self.phase == Phase::DrawSelect;
        for (i, rect) in self.hole_rects(0, false).iter().enumerate() {
            s.bl(&self.renderer.get(&me.hole[i], self.cw, self.ch), (rect.x, rect.y));
            if highlight && self.holds.contains(&i) {
                draw::rect_r(s, p.gold, *rect, 3, 6);
                let lbl = self.tiny.render(&t("poker.keep"), true, p.gold);
                s.blr(&lbl, lbl.get_rect().midtop_at((rect.centerx(), rect.bottom() + 2)));
            }
        }
        let mut me_label = t("poker.you");
        if me.folded {
            me_label += &format!("  ({})", t("poker.folded"));
        }
        let li = self.small.render(&me_label, true, if self.turn == 0 && self.phase == Phase::Acting { self.b.accent } else { p.text_dim });
        s.blr(&li, li.get_rect().midtop_at((w / 2, (h as f64 * 0.60) as i32 - 22)));
        if self.phase == Phase::Acting && self.players[self.turn].is_human {
            self.draw_actions(s);
        } else if self.phase == Phase::DrawSelect {
            self.draw_confirm(s, &t("poker.draw_btn"));
        } else if self.phase == Phase::Showdown {
            self.draw_confirm(s, &t("poker.next_hand"));
        }
        if !self.msg.is_empty() {
            self.draw_msg(s);
        }
    }

    fn draw_opponent(&self, s: &mut Surface, pl: &Player, cx: i32, cy: i32, active: bool) {
        let p = ui::pal();
        let w = (self.cw as f64 * 0.62) as i32;
        let h = (self.ch as f64 * 0.62) as i32;
        let dx = (w as f64 * 0.5) as i32;
        let x0 = cx - py::fdiv32(w + dx, 2);
        for (i, card) in pl.hole.iter().enumerate() {
            let img = if card.face_up || self.reveal { self.renderer.get(card, w, h) } else { self.renderer.back(w, h) };
            s.bl(&img, (x0 + i as i32 * dx, cy));
        }
        let mut name = pl.name.clone();
        if pl.folded {
            name += &format!(" ({})", t("poker.folded"));
        }
        let col = if active {
            self.b.accent
        } else if !pl.folded {
            p.text_dim
        } else {
            p.red
        };
        let ni = self.tiny.render(&name, true, col);
        s.blr(&ni, ni.get_rect().midtop_at((cx, cy - 16)));
        let si = self.tiny.render(&pl.stack.to_string(), true, p.gold);
        s.blr(&si, si.get_rect().midtop_at((cx, cy + h + 2)));
        if pl.round_bet != 0 {
            let bi = self.tiny.render(&format!("+{}", pl.round_bet), true, p.gold);
            s.blr(&bi, bi.get_rect().midtop_at((cx, cy + h + 18)));
        }
        if self.phase == Phase::Showdown && !pl.folded {
            if let Some(rt) = &pl.result {
                let hi = self.tiny.render(&t(&format!("poker.hand.{}", category_key(rt))), true, p.text);
                s.blr(&hi, hi.get_rect().midtop_at((cx, cy + h + 18)));
            }
        }
    }

    fn draw_community(&self, s: &mut Surface) {
        let n = self.community.len() as i32;
        let (w, h) = (self.cw, self.ch);
        let dx = (w as f64 * 1.12) as i32;
        let total = w + (n - 1) * dx;
        let x0 = self.b.width / 2 - py::fdiv32(total, 2);
        let y = (self.b.height as f64 * 0.38) as i32;
        for (i, card) in self.community.iter().enumerate() {
            s.bl(&self.renderer.get(card, w, h), (x0 + i as i32 * dx, y));
        }
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
        if primary {
            let border = ui::mix(self.b.accent, (255, 255, 255), 0.25 * ui::pulse(2.2, 0.0, 1.0));
            draw::rect_r(s, border, rect, 2, 10);
        } else {
            draw::rect_r(s, p.border_light, rect, 1, 10);
        }
        let img = self.small.render(label, true, if on { p.text } else { p.text_dim });
        s.blr(&img, img.get_rect().center_at(rect.center()));
    }

    fn draw_actions(&self, s: &mut Surface) {
        self.draw_strip(s);
        let pl = &self.players[self.turn];
        let to_call = self.to_call(self.turn);
        let call_lbl = if to_call == 0 { t("poker.check") } else { tf("poker.call_n", &[("n", &to_call.min(pl.stack).to_string())]) };
        let labels = [t("poker.fold"), call_lbl, t("poker.raise"), t("poker.allin")];
        let keys = ["F", "C", "R", "A"];
        for (i, r) in self.action_rects.iter().enumerate() {
            self.draw_btn(s, *r, &format!("{} ({})", labels[i], keys[i]), false, true);
        }
    }

    fn draw_confirm(&self, s: &mut Surface, label: &str) {
        self.draw_strip(s);
        self.draw_btn(s, self.deal_rect, label, true, true);
    }

    fn draw_msg(&self, s: &mut Surface) {
        let p = ui::pal();
        let img = self.small.render(&self.msg, true, p.gold);
        s.blr(&img, img.get_rect().center_at((self.b.width / 2, self.strip.y - 16)));
    }

    fn draw_video(&self, s: &mut Surface) {
        let p = ui::pal();
        let me = &self.players[0];
        for (i, rect) in self.hole_rects(0, true).iter().enumerate() {
            s.bl(&self.renderer.get(&me.hole[i], self.cw, self.ch), (rect.x, rect.y));
            if self.phase == Phase::VideoHold && self.holds.contains(&i) {
                draw::rect_r(s, p.gold, *rect, 3, 6);
                let lbl = self.small.render(&t("poker.held"), true, p.gold);
                s.blr(&lbl, lbl.get_rect().midbottom_at((rect.centerx(), rect.y - 3)));
            }
            let num = self.tiny.render(&(i + 1).to_string(), true, p.text_dim);
            s.blr(&num, num.get_rect().midtop_at((rect.centerx(), rect.bottom() + 3)));
        }
        self.draw_paytable(s);
        if self.phase == Phase::VideoHold {
            self.draw_confirm(s, &t("poker.draw_btn"));
        } else if self.phase == Phase::Showdown {
            self.draw_confirm(s, &t("poker.deal_btn"));
        }
        if !self.msg.is_empty() {
            self.draw_msg(s);
        }
    }

    fn draw_paytable(&self, s: &mut Surface) {
        let p = ui::pal();
        let rows = [
            ("royal", 250),
            ("straight_flush", 50),
            ("quads", 25),
            ("full_house", 9),
            ("flush", 6),
            ("straight", 4),
            ("trips", 3),
            ("two_pair", 2),
            ("jacks", 1),
        ];
        let x = 16;
        let mut y = (self.b.height as f64 * 0.16) as i32;
        let title = self.tiny.render(&t("poker.paytable"), true, self.b.accent);
        s.bl(&title, (x, y - 18));
        for (name, mult) in rows {
            let key = if name != "jacks" { format!("poker.hand.{name}") } else { "poker.jacks".to_string() };
            let lbl = self.tiny.render(&t(&key), true, p.text_dim);
            s.bl(&lbl, (x, y));
            let mi = self.tiny.render(&format!("x{mult}"), true, p.gold);
            s.bl(&mi, (x + 150, y));
            y += 16;
        }
    }

    fn draw_video_bet(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let title = self.huge.render(&t("poker.mode.video"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (self.b.height as f64 * 0.22) as i32)));
        self.draw_paytable(s);
        self.draw_strip(s);
        for (i, r) in self.chip_rects.iter().enumerate() {
            let v = VIDEO_BETS[i];
            let on = v == self.video_bet;
            draw::circle(s, if on { p.gold } else { p.btn_sel }, r.center(), 21, 0);
            draw::circle(s, p.text, r.center(), 21, 2);
            let img = self.tiny.render(&v.to_string(), true, if on { rgb(20, 24, 30) } else { p.text });
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let mid = self.big.render(&format!("{}: {}", t("poker.bet"), self.video_bet), true, p.text);
        s.blr(&mid, mid.get_rect().center_at((cx, self.strip.centery())));
        self.draw_btn(s, self.deal_rect, &t("poker.deal_btn"), true, true);
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
        let head = self.huge.render(&t("poker.broke"), true, p.red);
        s.blr(&head, head.get_rect().center_at((cx, cy - 42)));
        let best = self.small.render(&format!("{}: {}", t("poker.best"), self.best), true, p.text_dim);
        s.blr(&best, best.get_rect().center_at((cx, cy + 2)));
        let sub = self.small.render(&tf("poker.broke_restart", &[("n", &lamabank::START_CHIPS.to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, cy + 32)));
        let hint = self.tiny.render(&t("common.enter_restart"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 60)));
    }

    fn draw_prehand(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let h = self.b.height;
        let title = self.huge.render(&t(&format!("poker.mode.{}", self.mode())), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.28) as i32)));
        let mut info = if self.mode() == "holdem" {
            vec![tf("poker.blinds", &[("sb", &SMALL_BLIND.to_string()), ("bb", &BIG_BLIND.to_string())])]
        } else {
            vec![tf("poker.ante", &[("n", &ANTE.to_string())])]
        };
        if self.mode() == "holdem" {
            info.push(tf("poker.opponents", &[("n", &self.n_opponents.to_string())]));
        }
        for (i, line) in info.iter().enumerate() {
            let im = self.small.render(line, true, p.text_dim);
            s.blr(&im, im.get_rect().center_at((cx, (h as f64 * 0.40) as i32 + i as i32 * 22)));
        }
        self.draw_btn(s, self.deal_rect, &t("poker.deal_btn"), true, true);
    }
}

fn video_payout(rt: &[i64]) -> i64 {
    for (cat, special, mult) in VIDEO_PAYOUT {
        if rt[0] != cat {
            continue;
        }
        match special {
            "royal" => {
                if is_royal(rt) {
                    return mult;
                }
                continue;
            }
            "jacks" => {
                if rt[0] == PAIR && rt[1] >= 11 {
                    return mult;
                }
                continue;
            }
            _ => return mult,
        }
    }
    0
}

impl Game for PokerGame {
    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.game_over = false;
        if !matches!(self.b.mode.as_str(), "holdem" | "draw" | "video") {
            self.b.mode = "holdem".into();
        }
        lamabank::load();
        self.chips = lamabank::balance();
        self.best = lamabank::score_for("poker");
        self.b.score = self.best;
        self.n_opponents = self.b.opt_i64("poker", "opponents", 2).clamp(1, 3) as usize;
        self.ai_level = self.b.opt_i64("poker", "difficulty", 1).clamp(0, 2) as usize;
        match self.mode() {
            "draw" => self.n_opponents = 1,
            "video" => self.n_opponents = 0,
            _ => {}
        }
        self.make_fonts();
        self.renderer = CardRenderer::new(self.b.accent);
        self.deck.clear();
        self.community.clear();
        self.pot = 0;
        self.players.clear();
        self.dealer = 0;
        self.turn = 0;
        self.current_bet = 0;
        self.min_raise = BIG_BLIND;
        self.to_act.clear();
        self.street = "preflop";
        self.msg.clear();
        self.act_delay = 0.0;
        self.reveal = false;
        self.holds.clear();
        self.video_bet = VIDEO_BETS[0];
        self.win_amount = 0;
        self.layout();
        self.phase = if self.mode() == "video" { Phase::BetVideo } else { Phase::PreHand };
        if self.broke() {
            self.phase = Phase::Broke;
        }
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.renderer.clear();
        self.layout();
    }

    fn on_exit(&mut self) {
        lamabank::clear_escrow(true);
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.phase {
            Phase::Broke => {
                if Self::is_confirm(ev) {
                    let m = self.b.mode.clone();
                    lamabank::refill_if_broke("poker", Some(&m));
                    self.chips = lamabank::balance();
                    self.phase = if self.mode() == "video" { Phase::BetVideo } else { Phase::PreHand };
                    self.b.play_sound("click");
                }
            }
            Phase::PreHand => {
                if Self::is_confirm(ev) {
                    self.start_hand();
                }
            }
            Phase::BetVideo => self.handle_video_bet(ev),
            Phase::VideoHold => self.handle_holds(ev, true),
            Phase::DrawSelect => self.handle_holds(ev, false),
            Phase::Showdown => {
                if Self::is_confirm(ev) {
                    self.next_after_showdown();
                }
            }
            Phase::Acting => self.handle_acting(ev),
        }
    }

    fn update(&mut self, dt: f64) {
        if self.phase == Phase::Acting {
            let p = &self.players[self.turn];
            if !p.is_human && !p.folded && !p.all_in {
                self.act_delay -= dt;
                if self.act_delay <= 0.0 {
                    let ti = self.turn;
                    self.ai_action(ti);
                }
            }
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        s.bl(&self.felt, (0, 0));
        self.draw_topbar(&mut s);
        match self.phase {
            Phase::Broke => self.draw_broke(&mut s),
            Phase::BetVideo => self.draw_video_bet(&mut s),
            _ if self.phase == Phase::PreHand || self.players.is_empty() => self.draw_prehand(&mut s),
            _ if self.mode() == "video" => self.draw_video(&mut s),
            _ => self.draw_table(&mut s),
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "poker",
    class_name: "PokerGame",
    supports_multiplayer: false,
    modes: &[("holdem", "poker.mode.holdem"), ("draw", "poker.mode.draw"), ("video", "poker.mode.video")],
    make: PokerGame::new,
};
