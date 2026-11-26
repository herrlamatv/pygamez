//! solitaire.rs
//! ============
//! Solitaer - fuenf Varianten (Klondike, Spider, FreeCell, Pyramid, TriPeaks).
//!
//! Uebertragen aus `games/solitaire.py`: Drag & Drop oder Klick-Klick,
//! Rechtsklick aufs Fundament, unbegrenztes Undo ueber Zug-Protokolle.

use std::collections::HashMap;

use crate::cards::{self as c, Card, CardRenderer};
use pgz_engine::prelude::*;
use serde_json::Value;

const COL_FELT: Color = Color::rgb(20, 44, 35);
const COL_FELT_EDGE: Color = Color::rgb(13, 31, 24);
const SLOT: Color = Color::rgb(92, 118, 104);
const DRAG_PX: i32 = 6;
const CLICK_S: f64 = 0.35;

#[derive(Clone, Copy, PartialEq)]
enum PKind {
    Stock,
    Waste,
    Foundation,
    Tableau,
    Cell,
    Grid,
    Removed,
}

#[derive(Clone)]
struct Pile {
    kind: PKind,
    cards: Vec<Card>,
    x: i32,
    y: i32,
    dy_down: i32,
    dy_up: i32,
    row: i32,
    col: f64,
}

impl Pile {
    fn new(kind: PKind) -> Pile {
        Pile { kind, cards: Vec::new(), x: 0, y: 0, dy_down: 0, dy_up: 0, row: 0, col: 0.0 }
    }

    fn rects(&self, w: i32, h: i32) -> Vec<Rect> {
        c::fan_rects(self.x as f64, self.y as f64, &self.cards, w, h, self.dy_down as f64, self.dy_up as f64)
    }

    fn top(&self) -> Option<&Card> {
        self.cards.last()
    }
}

#[derive(Default, Clone)]
struct Extra {
    recycle: Option<usize>,
    mv: bool,
    sequence: bool,
    deal: bool,
    combo_was: Option<i64>,
}

#[derive(Default, Clone)]
struct Record {
    ops: Vec<(usize, usize, usize)>,
    flips: Vec<(usize, usize)>,
    score: i64,
    extra: Extra,
}

enum Click {
    Nothing,
    Selected,
    Rec(Record),
}

#[derive(Clone, Copy, PartialEq)]
enum VKind {
    Klondike,
    Spider,
    FreeCell,
    Pyramid,
    TriPeaks,
}

const VKEYS: [&str; 5] = ["klondike", "spider", "freecell", "pyramid", "tripeaks"];

/// TriPeaks: (Reihe, Spalten-Versatz in Kartenbreiten) fuer 28 Karten.
fn tri_layout() -> Vec<(i32, f64)> {
    let mut v: Vec<(i32, f64)> = [1.5, 4.5, 7.5].iter().map(|c| (0, *c)).collect();
    v.extend([1.0, 2.0, 4.0, 5.0, 7.0, 8.0].iter().map(|c| (1, *c)));
    v.extend((0..9).map(|c| (2, c as f64 + 0.5)));
    v.extend((0..10).map(|c| (3, c as f64)));
    v
}

struct Drag {
    src: usize,
    ci: usize,
    cards: Vec<Card>,
    pos: (i32, i32),
    start: (i32, i32),
    t: f64,
    moved: bool,
    off: (i32, i32),
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
}

/// `SolitaireGame`
pub struct SolitaireGame {
    b: GameBase,
    vk: VKind,
    // Variantenzustand
    draw3: bool,
    recycles: i64,
    sequences: i64,
    v_moves: i64,
    children: HashMap<usize, (usize, usize)>,
    redeals: i64,
    combo: i64,
    // Huelle
    small: Font,
    tiny: Font,
    mono: Font,
    huge: Font,
    renderer: CardRenderer,
    hud_h: i32,
    felt: Surface,
    hud_bg: Surface,
    overlay: Surface,
    piles: Vec<Pile>,
    sel: Option<(usize, usize)>,
    drag: Option<Drag>,
    undo_stack: Vec<Record>,
    elapsed: f64,
    moves: i64,
    won: bool,
    win_bonus: i64,
    msg: Option<String>,
    msg_t: f64,
    option_rects: Vec<Rect>,
    start_rect: Rect,
    state: State,
    cw: i32,
    ch: i32,
}

impl SolitaireGame {
    /// `SolitaireGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let accent = b.accent;
        let mut g = SolitaireGame {
            b,
            vk: VKind::Klondike,
            draw3: false,
            recycles: 0,
            sequences: 0,
            v_moves: 0,
            children: HashMap::new(),
            redeals: 2,
            combo: 0,
            small: f.clone(),
            tiny: f.clone(),
            mono: f.clone(),
            huge: f,
            renderer: CardRenderer::new(accent),
            hud_h: 34,
            felt: pg::surface(1, 1),
            hud_bg: pg::surface(1, 1),
            overlay: pg::surface(1, 1),
            piles: Vec::new(),
            sel: None,
            drag: None,
            undo_stack: Vec::new(),
            elapsed: 0.0,
            moves: 0,
            won: false,
            win_bonus: 0,
            msg: None,
            msg_t: 0.0,
            option_rects: Vec::new(),
            start_rect: Rect::new(0, 0, 0, 0),
            state: State::Setup,
            cw: 30,
            ch: 42,
        };
        g.reset();
        Box::new(g)
    }

    fn vkey(&self) -> &'static str {
        VKEYS[self.vk as usize]
    }

    // ----------------------------------------------- Einstellungen

    fn settings_get(&self, key: &str) -> Option<Value> {
        self.b.opt_value("solitaire", key)
    }

    fn truthy(v: &Option<Value>) -> bool {
        match v {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
            Some(Value::String(s)) => !s.is_empty(),
            Some(Value::Array(a)) => !a.is_empty(),
            Some(Value::Object(o)) => !o.is_empty(),
        }
    }

    fn settings_toggle(&mut self, key: &str) {
        let val = !Self::truthy(&self.settings_get(key));
        self.b.set_opt("solitaire", key, Value::Bool(val));
    }

    fn spider_suits_value(&self) -> Value {
        self.settings_get("spider_suits").unwrap_or(Value::from(1))
    }

    fn settings_cycle_suits(&mut self) {
        let values = [1i64, 2, 4];
        let cur = self.spider_suits_value();
        let curf = match &cur {
            Value::Number(n) => n.as_f64(),
            Value::Bool(b) => Some(*b as i64 as f64),
            _ => None,
        };
        let idx = curf.and_then(|f| values.iter().position(|v| *v as f64 == f)).unwrap_or(0);
        self.b.set_opt("solitaire", "spider_suits", Value::from(values[(idx + 1) % values.len()]));
    }

    fn setup_rows(&self) -> usize {
        match self.vk {
            VKind::Klondike | VKind::Spider => 1,
            _ => 0,
        }
    }

    fn setup_row_label(&self) -> String {
        match self.vk {
            VKind::Klondike => {
                let on = Self::truthy(&self.settings_get("draw3"));
                format!("{}:  {}", t("sol.draw3"), if on { t("common.on") } else { t("common.off") })
            }
            _ => {
                let v = self.spider_suits_value();
                let n = match &v {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => if *b { "True".into() } else { "False".into() },
                    Value::Number(n) => match n.as_i64() {
                        Some(i) => i.to_string(),
                        None => py::fstr(n.as_f64().unwrap_or(0.0)),
                    },
                    other => other.to_string(),
                };
                tf("sol.suits", &[("n", &n)])
            }
        }
    }

    fn setup_row_toggle(&mut self) {
        match self.vk {
            VKind::Klondike => self.settings_toggle("draw3"),
            VKind::Spider => self.settings_cycle_suits(),
            _ => {}
        }
    }

    // ----------------------------------------------- Aufbau

    fn make_fonts(&mut self) {
        self.small = ui::f(16);
        self.tiny = ui::f(13);
        self.mono = ui::font(15, false, true);
        self.huge = ui::fb(26.max(self.b.height / 11));
    }

    fn rebuild_static(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.felt = c::make_felt(w, h, COL_FELT, COL_FELT_EDGE);
        self.hud_bg = pg::surface_alpha(w, self.hud_h);
        self.hud_bg.fill_c(rgba(8, 22, 16, 224));
        self.overlay = pg::surface_alpha(w, h);
        self.overlay.fill_c(rgba(6, 14, 10, 190));
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 380.min(self.b.width - 60);
        let y0 = (self.b.height as f64 * 0.36) as i32;
        let rows = self.setup_rows() as i32;
        self.option_rects = (0..rows).map(|i| Rect::new(cx - bw / 2, y0 + i * 52, bw, 42)).collect();
        self.start_rect = Rect::new(cx - 95, y0 + rows * 52 + 16, 190, 46);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let rows = self.setup_rows();
        if ev.kind == Kind::KeyDown {
            if ev.key_in(&["Return", "space"]) {
                self.new_deal();
            } else if ev.key_in(&["Left", "Right", "a", "d", "A", "D"]) && rows > 0 {
                self.setup_row_toggle();
                self.b.play_sound("select");
                self.build_setup_layout();
            }
        } else if ev.kind == Kind::MouseDown {
            for i in 0..self.option_rects.len() {
                if self.option_rects[i].hit(ev.pos) && i < rows {
                    self.setup_row_toggle();
                    self.b.play_sound("select");
                    return;
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.new_deal();
            }
        }
    }

    // ----------------------------------------------- Neues Blatt

    fn new_deal(&mut self) {
        self.b.game_over = false;
        self.won = false;
        self.b.score = if self.vk == VKind::Spider { 500 } else { 0 };
        self.sel = None;
        self.drag = None;
        self.undo_stack.clear();
        self.elapsed = 0.0;
        self.moves = 0;
        let mut rng = py::Random::unseeded();
        self.deal(&mut rng);
        self.layout();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn deal(&mut self, rng: &mut py::Random) {
        match self.vk {
            VKind::Klondike => {
                self.piles = vec![Pile::new(PKind::Stock), Pile::new(PKind::Waste)];
                self.piles.extend((0..4).map(|_| Pile::new(PKind::Foundation)));
                self.piles.extend((0..7).map(|_| Pile::new(PKind::Tableau)));
                let mut deck = c::make_deck(&[0, 1, 2, 3], 1);
                rng.shuffle(&mut deck);
                for i in 0..7 {
                    for j in 0..=i {
                        let mut card = deck.pop().expect("deck");
                        card.face_up = j == i;
                        self.piles[6 + i].cards.push(card);
                    }
                }
                self.piles[0].cards = deck;
                self.draw3 = Self::truthy(&self.settings_get("draw3"));
                self.recycles = 0;
            }
            VKind::Spider => {
                self.piles = vec![Pile::new(PKind::Stock)];
                self.piles.extend((0..10).map(|_| Pile::new(PKind::Tableau)));
                self.piles.extend((0..8).map(|_| Pile::new(PKind::Foundation)));
                let suits_n = match self.spider_suits_value() {
                    Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.trunc() as i64)).unwrap_or(1),
                    Value::Bool(b) => b as i64,
                    Value::String(s) => s.trim().parse().unwrap_or(1),
                    _ => 1,
                };
                let mut deck = match suits_n {
                    1 => c::make_deck(&[0], 8),
                    2 => c::make_deck(&[0, 1], 4),
                    _ => c::make_deck(&[0, 1, 2, 3], 2),
                };
                rng.shuffle(&mut deck);
                for i in 0..10 {
                    let n = if i < 4 { 6 } else { 5 };
                    for j in 0..n {
                        let mut card = deck.pop().expect("deck");
                        card.face_up = j == n - 1;
                        self.piles[1 + i].cards.push(card);
                    }
                }
                self.piles[0].cards = deck;
                self.sequences = 0;
                self.v_moves = 0;
            }
            VKind::FreeCell => {
                self.piles = (0..4).map(|_| Pile::new(PKind::Cell)).collect();
                self.piles.extend((0..4).map(|_| Pile::new(PKind::Foundation)));
                self.piles.extend((0..8).map(|_| Pile::new(PKind::Tableau)));
                let mut deck = c::make_deck(&[0, 1, 2, 3], 1);
                rng.shuffle(&mut deck);
                for (i, mut card) in deck.into_iter().enumerate() {
                    card.face_up = true;
                    self.piles[8 + i % 8].cards.push(card);
                }
            }
            VKind::Pyramid => {
                self.piles = vec![Pile::new(PKind::Stock), Pile::new(PKind::Waste)];
                self.children.clear();
                for row in 0..7 {
                    for col in 0..=row {
                        let mut p = Pile::new(PKind::Grid);
                        p.row = row;
                        p.col = col as f64;
                        self.piles.push(p);
                    }
                }
                self.piles.push(Pile::new(PKind::Removed));
                let pidx = |r: usize, c: usize| 2 + r * (r + 1) / 2 + c;
                for row in 0..6 {
                    for col in 0..=row {
                        self.children.insert(pidx(row, col), (pidx(row + 1, col), pidx(row + 1, col + 1)));
                    }
                }
                let mut deck = c::make_deck(&[0, 1, 2, 3], 1);
                rng.shuffle(&mut deck);
                for i in 0..28 {
                    let mut card = deck.pop().expect("deck");
                    card.face_up = true;
                    self.piles[2 + i].cards.push(card);
                }
                self.piles[0].cards = deck;
                self.redeals = 2;
            }
            VKind::TriPeaks => {
                self.piles = vec![Pile::new(PKind::Stock), Pile::new(PKind::Waste)];
                let lay = tri_layout();
                for &(row, off) in &lay {
                    let mut p = Pile::new(PKind::Grid);
                    p.row = row;
                    p.col = off;
                    self.piles.push(p);
                }
                let mut deck = c::make_deck(&[0, 1, 2, 3], 1);
                rng.shuffle(&mut deck);
                for (i, l) in lay.iter().enumerate() {
                    let mut card = deck.pop().expect("deck");
                    card.face_up = l.0 == 3;
                    self.piles[2 + i].cards.push(card);
                }
                let mut first = deck.pop().expect("deck");
                first.face_up = true;
                self.piles[1].cards.push(first);
                self.piles[0].cards = deck;
                self.combo = 0;
            }
        }
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        match self.vk {
            VKind::Klondike => {
                self.cw = 30.max(w / 9);
                self.ch = (self.cw as f64 * 1.4) as i32;
                let m = 6.max(w / 80);
                let top = self.hud_h + m;
                self.piles[0].x = m;
                self.piles[0].y = top;
                self.piles[1].x = m + self.cw + m;
                self.piles[1].y = top;
                for i in 0..4 {
                    self.piles[2 + i].x = w - m - (4 - i as i32) * (self.cw + m) + m;
                    self.piles[2 + i].y = top;
                }
                let ty = top + self.ch + 2 * m;
                let span = 7 * self.cw + 6 * m;
                let x0 = py::fdiv32(w - span, 2);
                for i in 0..7 {
                    let p = &mut self.piles[6 + i];
                    p.x = x0 + i as i32 * (self.cw + m);
                    p.y = ty;
                    p.dy_down = (self.ch as f64 * 0.12) as i32;
                    p.dy_up = (self.ch as f64 * 0.28) as i32;
                }
            }
            VKind::Spider => {
                let m = 4.max(w / 120);
                self.cw = 26.max(py::fdiv32(w - 11 * m, 10));
                self.ch = (self.cw as f64 * 1.4) as i32;
                let top = self.hud_h + m;
                self.piles[0].x = m;
                self.piles[0].y = top;
                for i in 0..8 {
                    self.piles[11 + i].x = w - m - (8 - i as i32) * (self.cw / 2 + m) + m;
                    self.piles[11 + i].y = top;
                }
                let ty = top + self.ch + 2 * m;
                for i in 0..10 {
                    let p = &mut self.piles[1 + i];
                    p.x = m + i as i32 * (self.cw + m);
                    p.y = ty;
                    p.dy_down = (self.ch as f64 * 0.12) as i32;
                    p.dy_up = (self.ch as f64 * 0.24) as i32;
                }
            }
            VKind::FreeCell => {
                let m = 5.max(w / 100);
                self.cw = 28.max(py::fdiv32(w - 9 * m, 8) - 2);
                self.ch = (self.cw as f64 * 1.4) as i32;
                let top = self.hud_h + m;
                for i in 0..4 {
                    self.piles[i].x = m + i as i32 * (self.cw + m);
                    self.piles[i].y = top;
                }
                for i in 0..4 {
                    self.piles[4 + i].x = w - m - (4 - i as i32) * (self.cw + m) + m;
                    self.piles[4 + i].y = top;
                }
                let ty = top + self.ch + 2 * m;
                let span = 8 * self.cw + 7 * m;
                let x0 = py::fdiv32(w - span, 2);
                for i in 0..8 {
                    let p = &mut self.piles[8 + i];
                    p.x = x0 + i as i32 * (self.cw + m);
                    p.y = ty;
                    p.dy_down = (self.ch as f64 * 0.24) as i32;
                    p.dy_up = (self.ch as f64 * 0.24) as i32;
                }
            }
            VKind::Pyramid => {
                self.cw = 26.max((w as f64 / 9.5) as i32);
                self.ch = (self.cw as f64 * 1.4) as i32;
                let top = self.hud_h + 6;
                for i in 0..28 {
                    let (cw, ch) = (self.cw as f64, self.ch as f64);
                    let p = &mut self.piles[2 + i];
                    let (row, col) = (p.row as f64, p.col);
                    p.x = w / 2 + ((col - row / 2.0 - 0.5) * cw * 1.08) as i32 + self.cw / 12;
                    p.y = top + (row * ch * 0.45) as i32;
                }
                let m = 6.max(w / 60);
                self.piles[0].x = m;
                self.piles[0].y = h - self.ch - m;
                self.piles[1].x = m + self.cw + m;
                self.piles[1].y = h - self.ch - m;
                self.piles[30].x = -1000;
                self.piles[30].y = -1000;
            }
            VKind::TriPeaks => {
                self.cw = 26.max((w as f64 / 11.5) as i32);
                self.ch = (self.cw as f64 * 1.4) as i32;
                let top = self.hud_h + 6;
                let span = 10 * self.cw;
                let x0 = py::fdiv32(w - span, 2);
                for i in 0..28 {
                    let (cw, ch) = (self.cw as f64, self.ch as f64);
                    let p = &mut self.piles[2 + i];
                    p.x = x0 + (p.col * cw) as i32;
                    p.y = top + (p.row as f64 * ch * 0.5) as i32;
                }
                let m = 6.max(w / 60);
                self.piles[0].x = w / 2 - self.cw - m;
                self.piles[0].y = h - self.ch - m;
                self.piles[1].x = w / 2 + m;
                self.piles[1].y = h - self.ch - m;
            }
        }
    }

    // ----------------------------------------------- Regeln (Varianten)

    fn flip_top(&mut self, pi: usize, rec: &mut Record, score: i64) {
        let p = &mut self.piles[pi];
        if let Some(top) = p.cards.last_mut() {
            if !top.face_up {
                top.face_up = true;
                rec.flips.push((pi, p.cards.len() - 1));
                rec.score += score;
            }
        }
    }

    fn foundation_ok(card: &Card, pile: &Pile) -> bool {
        match pile.top() {
            Some(top) => card.suit == top.suit && card.rank == top.rank + 1,
            None => card.rank == 1,
        }
    }

    fn alt_desc_ok(card: &Card, pile: &Pile) -> bool {
        match pile.top() {
            Some(top) => top.face_up && card.red() != top.red() && card.rank + 1 == top.rank,
            None => true,
        }
    }

    fn freecell_limit(&self, target_empty: bool) -> usize {
        let free = (0..4).filter(|&i| self.piles[i].cards.is_empty()).count();
        let mut empty = (8..16).filter(|&i| self.piles[i].cards.is_empty()).count();
        if target_empty && empty > 0 {
            empty -= 1;
        }
        (free + 1) * (1usize << empty)
    }

    fn can_grab(&self, pi: usize, ci: usize) -> bool {
        let p = &self.piles[pi];
        match self.vk {
            VKind::Klondike => match p.kind {
                PKind::Waste | PKind::Foundation => ci == p.cards.len() - 1,
                PKind::Tableau => p.cards[ci].face_up,
                _ => false,
            },
            VKind::Spider => {
                if p.kind != PKind::Tableau {
                    return false;
                }
                let run = &p.cards[ci..];
                if run.is_empty() || !run[0].face_up {
                    return false;
                }
                run.windows(2).all(|w| w[1].face_up && w[1].suit == w[0].suit && w[1].rank + 1 == w[0].rank)
            }
            VKind::FreeCell => match p.kind {
                PKind::Cell => !p.cards.is_empty(),
                PKind::Foundation => false,
                _ => {
                    let run = &p.cards[ci..];
                    if !run.windows(2).all(|w| w[1].red() != w[0].red() && w[1].rank + 1 == w[0].rank) {
                        return false;
                    }
                    run.len() <= self.freecell_limit(false)
                }
            },
            _ => false,
        }
    }

    fn commit(&mut self, cards: Vec<Card>, src: usize, dst: usize, score: i64) -> Record {
        let n = cards.len();
        self.piles[dst].cards.extend(cards);
        Record { ops: vec![(src, dst, n)], score, ..Default::default() }
    }

    /// `try_drop` - bei Misserfolg kommen die Karten zurueck (`Err`).
    fn try_drop(&mut self, cards: Vec<Card>, src: usize, dst: usize) -> Result<Record, Vec<Card>> {
        let dk = self.piles[dst].kind;
        match self.vk {
            VKind::Klondike => {
                if dk == PKind::Foundation {
                    if cards.len() == 1 && Self::foundation_ok(&cards[0], &self.piles[dst]) {
                        let mut rec = self.commit(cards, src, dst, 10);
                        self.flip_top(src, &mut rec, 5);
                        return Ok(rec);
                    }
                } else if dk == PKind::Tableau {
                    let first = &cards[0];
                    let ok = if self.piles[dst].cards.is_empty() { first.rank == 13 } else { Self::alt_desc_ok(first, &self.piles[dst]) };
                    if ok {
                        let delta = if self.piles[src].kind == PKind::Foundation { -10 } else { 0 };
                        let mut rec = self.commit(cards, src, dst, delta);
                        self.flip_top(src, &mut rec, 5);
                        return Ok(rec);
                    }
                }
                Err(cards)
            }
            VKind::Spider => {
                if dk != PKind::Tableau {
                    return Err(cards);
                }
                if let Some(top) = self.piles[dst].top() {
                    if !(top.face_up && cards[0].rank + 1 == top.rank) {
                        return Err(cards);
                    }
                }
                self.v_moves += 1;
                let mut rec = self.commit(cards, src, dst, -1);
                rec.extra.mv = true;
                self.flip_top(src, &mut rec, 0);
                self.check_sequence(dst, &mut rec);
                Ok(rec)
            }
            VKind::FreeCell => {
                match dk {
                    PKind::Cell => {
                        if cards.len() == 1 && self.piles[dst].cards.is_empty() {
                            return Ok(self.commit(cards, src, dst, 0));
                        }
                    }
                    PKind::Foundation => {
                        if cards.len() == 1 && Self::foundation_ok(&cards[0], &self.piles[dst]) {
                            return Ok(self.commit(cards, src, dst, 10));
                        }
                    }
                    PKind::Tableau => {
                        if cards.len() > self.freecell_limit(self.piles[dst].cards.is_empty()) {
                            return Err(cards);
                        }
                        if self.piles[dst].cards.is_empty() || Self::alt_desc_ok(&cards[0], &self.piles[dst]) {
                            return Ok(self.commit(cards, src, dst, 0));
                        }
                    }
                    _ => {}
                }
                Err(cards)
            }
            _ => Err(cards),
        }
    }

    fn check_sequence(&mut self, pi: usize, rec: &mut Record) {
        let p = &self.piles[pi];
        if p.cards.len() < 13 {
            return;
        }
        let run = &p.cards[p.cards.len() - 13..];
        if run[0].rank != 13 || !run.iter().all(|c| c.face_up) {
            return;
        }
        if !run.windows(2).all(|w| w[1].suit == w[0].suit && w[1].rank + 1 == w[0].rank) {
            return;
        }
        let Some(target) = (11..19).find(|&i| self.piles[i].cards.is_empty()) else {
            return;
        };
        let n = self.piles[pi].cards.len();
        let moved: Vec<Card> = self.piles[pi].cards.drain(n - 13..).collect();
        self.piles[target].cards.extend(moved);
        rec.ops.push((pi, target, 13));
        rec.score += 100;
        self.sequences += 1;
        rec.extra.sequence = true;
        self.flip_top(pi, rec, 0);
        self.b.play_sound("line");
    }

    fn recycle_waste(&mut self) -> usize {
        let waste: Vec<Card> = std::mem::take(&mut self.piles[1].cards);
        let n = waste.len();
        for mut card in waste.into_iter().rev() {
            card.face_up = false;
            self.piles[0].cards.push(card);
        }
        n
    }

    fn on_stock(&mut self) -> Option<Record> {
        match self.vk {
            VKind::Klondike => {
                if !self.piles[0].cards.is_empty() {
                    let n = (if self.draw3 { 3 } else { 1 }).min(self.piles[0].cards.len());
                    let mut rec = Record::default();
                    for _ in 0..n {
                        let mut card = self.piles[0].cards.pop().expect("stock");
                        card.face_up = true;
                        self.piles[1].cards.push(card);
                    }
                    rec.ops.push((0, 1, n));
                    let wl = self.piles[1].cards.len();
                    rec.flips = (0..n).map(|i| (1, wl - 1 - i)).collect();
                    return Some(rec);
                }
                if !self.piles[1].cards.is_empty() {
                    let n = self.recycle_waste();
                    self.recycles += 1;
                    let mut rec = Record { score: -20, ..Default::default() };
                    rec.extra = Extra { recycle: Some(n), ..Default::default() };
                    return Some(rec);
                }
                None
            }
            VKind::Spider => {
                if self.piles[0].cards.is_empty() {
                    return None;
                }
                if (0..10).any(|i| self.piles[1 + i].cards.is_empty()) {
                    self.flash(t("sol.no_empty"));
                    return None;
                }
                let mut rec = Record::default();
                for i in 0..10 {
                    let mut card = self.piles[0].cards.pop().expect("stock");
                    card.face_up = true;
                    self.piles[1 + i].cards.push(card);
                    rec.ops.push((0, 1 + i, 1));
                    rec.flips.push((1 + i, self.piles[1 + i].cards.len() - 1));
                }
                rec.extra.deal = true;
                for i in 0..10 {
                    self.check_sequence(1 + i, &mut rec);
                }
                Some(rec)
            }
            VKind::Pyramid => {
                self.sel = None;
                if !self.piles[0].cards.is_empty() {
                    let mut card = self.piles[0].cards.pop().expect("stock");
                    card.face_up = true;
                    self.piles[1].cards.push(card);
                    let mut rec = Record::default();
                    rec.ops.push((0, 1, 1));
                    rec.flips.push((1, self.piles[1].cards.len() - 1));
                    return Some(rec);
                }
                if !self.piles[1].cards.is_empty() && self.redeals > 0 {
                    let n = self.recycle_waste();
                    self.redeals -= 1;
                    let rec = Record { extra: Extra { recycle: Some(n), ..Default::default() }, ..Default::default() };
                    return Some(rec);
                }
                None
            }
            VKind::TriPeaks => {
                if self.piles[0].cards.is_empty() {
                    return None;
                }
                let mut card = self.piles[0].cards.pop().expect("stock");
                card.face_up = true;
                self.piles[1].cards.push(card);
                let mut rec = Record::default();
                rec.ops.push((0, 1, 1));
                rec.flips.push((1, self.piles[1].cards.len() - 1));
                rec.extra.combo_was = Some(self.combo);
                self.combo = 0;
                Some(rec)
            }
            VKind::FreeCell => None,
        }
    }

    fn can_recycle(&self) -> bool {
        match self.vk {
            VKind::Klondike => !self.piles[1].cards.is_empty() && self.piles[0].cards.is_empty(),
            VKind::Pyramid => !self.piles[1].cards.is_empty() && self.piles[0].cards.is_empty() && self.redeals > 0,
            _ => false,
        }
    }

    fn on_right(&mut self, pi: usize) -> Option<Record> {
        let (kinds, fr): (&[PKind], std::ops::Range<usize>) = match self.vk {
            VKind::Klondike => (&[PKind::Waste, PKind::Tableau], 2..6),
            VKind::FreeCell => (&[PKind::Tableau, PKind::Cell], 4..8),
            _ => return None,
        };
        let p = &self.piles[pi];
        if !kinds.contains(&p.kind) || p.cards.is_empty() {
            return None;
        }
        let card = *p.top()?;
        if self.vk == VKind::Klondike && !card.face_up {
            return None;
        }
        for fi in fr {
            if Self::foundation_ok(&card, &self.piles[fi]) {
                self.piles[pi].cards.pop();
                return match self.try_drop(vec![card], pi, fi) {
                    Ok(r) => Some(r),
                    Err(_) => None,
                };
            }
        }
        None
    }

    fn exposed(&self, pi: usize) -> bool {
        let p = &self.piles[pi];
        if p.kind != PKind::Grid || p.cards.is_empty() {
            return false;
        }
        match self.children.get(&pi) {
            None => true,
            Some(&(a, b)) => self.piles[a].cards.is_empty() && self.piles[b].cards.is_empty(),
        }
    }

    fn covered_by(i: usize) -> Vec<usize> {
        let lay = tri_layout();
        let (row, off) = lay[i];
        lay.iter().enumerate().filter(|(_, (r2, o2))| *r2 == row + 1 && (o2 - off).abs() <= 0.51).map(|(j, _)| j).collect()
    }

    fn uncovered(&self, i: usize) -> bool {
        Self::covered_by(i).iter().all(|j| self.piles[2 + j].cards.is_empty())
    }

    fn reveal(&mut self, rec: &mut Record) {
        for i in 0..28 {
            let p = &self.piles[2 + i];
            if p.cards.last().is_some_and(|c| !c.face_up) && self.uncovered(i) {
                let p = &mut self.piles[2 + i];
                let n = p.cards.len();
                p.cards[n - 1].face_up = true;
                rec.flips.push((2 + i, n - 1));
            }
        }
    }

    fn on_click_card(&mut self, pi: usize, ci: usize) -> Click {
        match self.vk {
            VKind::Pyramid => {
                let p = &self.piles[pi];
                let clickable = (p.kind == PKind::Grid && self.exposed(pi)) || (p.kind == PKind::Waste && !p.cards.is_empty() && ci == p.cards.len() - 1);
                if !clickable {
                    return Click::Nothing;
                }
                let card = *p.top().expect("karte");
                if card.rank == 13 {
                    self.piles[pi].cards.pop();
                    self.piles[30].cards.push(card);
                    let mut rec = Record::default();
                    rec.ops.push((pi, 30, 1));
                    rec.score += 5;
                    self.b.play_sound("point");
                    return Click::Rec(rec);
                }
                let Some((spi, _)) = self.sel else {
                    self.sel = Some((pi, self.piles[pi].cards.len() - 1));
                    self.b.play_sound("select");
                    return Click::Selected;
                };
                self.sel = None;
                if spi == pi {
                    return Click::Nothing;
                }
                let Some(other) = self.piles[spi].top().copied() else {
                    return Click::Nothing;
                };
                if other.rank + card.rank == 13 {
                    let mut rec = Record::default();
                    for src in [spi, pi] {
                        let cc = self.piles[src].cards.pop().expect("karte");
                        self.piles[30].cards.push(cc);
                        rec.ops.push((src, 30, 1));
                        rec.score += 5;
                    }
                    self.b.play_sound("point");
                    return Click::Rec(rec);
                }
                Click::Nothing
            }
            VKind::TriPeaks => {
                let p = &self.piles[pi];
                if p.kind != PKind::Grid || p.cards.last().is_none_or(|c| !c.face_up) {
                    return Click::Nothing;
                }
                let i = pi - 2;
                if !self.uncovered(i) {
                    return Click::Nothing;
                }
                let top_rank = p.cards.last().map(|c| c.rank as i32).unwrap_or(0);
                let waste_rank = self.piles[1].top().map(|c| c.rank as i32).unwrap_or(0);
                let d = (top_rank - waste_rank).abs();
                if d != 1 && d != 12 {
                    return Click::Nothing;
                }
                let card = self.piles[pi].cards.pop().expect("karte");
                self.piles[1].cards.push(card);
                self.combo += 1;
                let mut rec = Record::default();
                rec.ops.push((pi, 1, 1));
                rec.score += 10 * self.combo;
                rec.extra.combo_was = Some(self.combo - 1);
                self.reveal(&mut rec);
                self.b.play_sound("point");
                Click::Rec(rec)
            }
            _ => Click::Nothing,
        }
    }

    fn undo_extra(&mut self, extra: &Extra) {
        match self.vk {
            VKind::Klondike | VKind::Pyramid => {
                if let Some(n) = extra.recycle {
                    let sl = self.piles[0].cards.len();
                    let moved: Vec<Card> = self.piles[0].cards.drain(sl - n..).collect();
                    for mut card in moved.into_iter().rev() {
                        card.face_up = true;
                        self.piles[1].cards.push(card);
                    }
                    if self.vk == VKind::Klondike {
                        self.recycles -= 1;
                    } else {
                        self.redeals += 1;
                    }
                }
            }
            VKind::Spider => {
                if extra.sequence {
                    self.sequences -= 1;
                }
                if extra.mv {
                    self.v_moves -= 1;
                }
            }
            VKind::TriPeaks => {
                if let Some(c) = extra.combo_was {
                    self.combo = c;
                }
            }
            VKind::FreeCell => {}
        }
    }

    fn is_won(&self) -> bool {
        match self.vk {
            VKind::Klondike => (2..6).all(|i| self.piles[i].cards.len() == 13),
            VKind::Spider => self.sequences >= 8,
            VKind::FreeCell => (4..8).all(|i| self.piles[i].cards.len() == 13),
            VKind::Pyramid | VKind::TriPeaks => (0..28).all(|i| self.piles[2 + i].cards.is_empty()),
        }
    }

    fn variant_win_bonus(&self) -> i64 {
        match self.vk {
            VKind::Klondike => 0.max(1000 - 2 * self.elapsed as i64),
            VKind::Spider => 0,
            VKind::FreeCell => 0.max(500 - self.elapsed as i64),
            _ => 500,
        }
    }

    fn hud_extra(&self) -> Vec<String> {
        match self.vk {
            VKind::Spider => vec![tf("sol.deals_left", &[("n", &(self.piles[0].cards.len() / 10).to_string())])],
            VKind::Pyramid => vec![tf("sol.redeals_left", &[("n", &self.redeals.to_string())])],
            VKind::TriPeaks => {
                let mut out = vec![tf("sol.deals_left", &[("n", &self.piles[0].cards.len().to_string())])];
                if self.combo > 1 {
                    out.push(tf("sol.combo", &[("n", &self.combo.to_string())]));
                }
                out
            }
            _ => Vec::new(),
        }
    }

    // ----------------------------------------------- Records / Undo

    fn apply_record(&mut self, rec: Option<Record>) -> bool {
        let Some(rec) = rec else {
            return false;
        };
        self.b.score += rec.score;
        self.moves += 1;
        self.undo_stack.push(rec);
        self.check_win();
        true
    }

    fn undo(&mut self) {
        if self.undo_stack.is_empty() || self.b.game_over {
            return;
        }
        self.cancel_drag();
        let rec = self.undo_stack.pop().expect("undo");
        for &(pi, ci) in &rec.flips {
            if ci < self.piles[pi].cards.len() {
                let card = &mut self.piles[pi].cards[ci];
                card.face_up = !card.face_up;
            }
        }
        for &(src, dst, n) in rec.ops.iter().rev() {
            let dl = self.piles[dst].cards.len();
            let moved: Vec<Card> = self.piles[dst].cards.drain(dl.saturating_sub(n)..).collect();
            self.piles[src].cards.extend(moved);
        }
        self.b.score -= rec.score;
        self.moves = 0.max(self.moves - 1);
        self.undo_extra(&rec.extra);
        self.sel = None;
        self.b.play_sound("rotate");
    }

    fn check_win(&mut self) {
        if self.is_won() {
            self.won = true;
            self.win_bonus = self.variant_win_bonus();
            self.b.score = 0.max(self.b.score + self.win_bonus);
            self.b.game_over = true;
            self.b.report_result(true);
            self.b.ach_event("solitaire_win", None);
            if self.elapsed < 180.0 {
                self.b.ach_event("solitaire_fast", None);
            }
            self.b.play_sound("win");
            self.b.rumble(220);
        }
    }

    fn flash(&mut self, text: String) {
        self.msg = Some(text);
        self.msg_t = 1.6;
    }

    // ----------------------------------------------- Treffer

    fn hit(&self, pos: (i32, i32)) -> Option<(usize, usize)> {
        let mut best = None;
        for (pi, p) in self.piles.iter().enumerate() {
            if p.kind == PKind::Removed {
                continue;
            }
            if let Some(ci) = c::hit_index(&p.rects(self.cw, self.ch), pos) {
                best = Some((pi, ci));
            }
        }
        best
    }

    fn pile_at(&self, pos: (i32, i32)) -> Option<usize> {
        let region_of = |p: &Pile, inflate: bool| {
            let rects = p.rects(self.cw, self.ch);
            let mut region = if rects.is_empty() {
                Rect::new(p.x, p.y, self.cw, self.ch)
            } else if rects.len() > 1 {
                rects[0].unionall(&rects[1..])
            } else {
                rects[0]
            };
            if inflate {
                region.h += self.ch / 2;
            }
            region
        };
        for inflate in [false, true] {
            for (pi, p) in self.piles.iter().enumerate() {
                if p.kind == PKind::Removed {
                    continue;
                }
                if region_of(p, inflate).hit(pos) {
                    return Some(pi);
                }
            }
        }
        None
    }

    // ----------------------------------------------- Eingabe

    fn on_down(&mut self, pos: (i32, i32)) {
        if self.drag.is_some() {
            self.on_up(pos);
            return;
        }
        let Some((pi, ci)) = self.hit(pos) else {
            if let Some(p0) = self.piles.first() {
                if p0.kind == PKind::Stock && p0.cards.is_empty() && Rect::new(p0.x, p0.y, self.cw, self.ch).hit(pos) {
                    self.sel = None;
                    let r = self.on_stock();
                    self.apply_record(r);
                    return;
                }
            }
            self.sel = None;
            return;
        };
        if self.piles[pi].kind == PKind::Stock {
            self.sel = None;
            let r = self.on_stock();
            self.apply_record(r);
            return;
        }
        match self.on_click_card(pi, ci) {
            Click::Rec(r) => {
                self.apply_record(Some(r));
                return;
            }
            Click::Selected => return,
            Click::Nothing => {}
        }
        if self.can_grab(pi, ci) {
            let p = &mut self.piles[pi];
            let cards: Vec<Card> = p.cards.drain(ci..).collect();
            let dy = if p.dy_up != 0 { p.dy_up } else { 1 };
            let off = (pos.0 - p.x, pos.1 - (p.y + ci as i32 * dy));
            self.drag = Some(Drag { src: pi, ci, cards, pos, start: pos, t: py::time_time(), moved: false, off });
            self.b.play_sound("select");
        } else {
            self.sel = None;
        }
    }

    fn on_up(&mut self, pos: (i32, i32)) {
        let Some(d) = self.drag.take() else {
            return;
        };
        let quick = !d.moved && py::time_time() - d.t < CLICK_S;
        if quick {
            let (src, ci) = (d.src, d.ci);
            self.piles[src].cards.extend(d.cards);
            match self.sel {
                Some(sel) if sel != (src, ci) => {
                    self.sel = None;
                    self.try_move(sel.0, sel.1, src);
                }
                _ => self.sel = Some((src, ci)),
            }
            return;
        }
        if let Some(dst) = self.pile_at(pos) {
            if dst != d.src {
                match self.try_drop(d.cards, d.src, dst) {
                    Ok(rec) => {
                        self.apply_record(Some(rec));
                        self.sel = None;
                        self.b.play_sound("move");
                        return;
                    }
                    Err(cards) => {
                        self.piles[d.src].cards.extend(cards);
                        return;
                    }
                }
            }
        }
        self.piles[d.src].cards.extend(d.cards);
    }

    fn try_move(&mut self, spi: usize, sci: usize, dst: usize) {
        if sci >= self.piles[spi].cards.len() || !self.can_grab(spi, sci) {
            return;
        }
        let cards: Vec<Card> = self.piles[spi].cards.drain(sci..).collect();
        match self.try_drop(cards, spi, dst) {
            Ok(rec) => {
                self.apply_record(Some(rec));
                self.b.play_sound("move");
            }
            Err(cards) => self.piles[spi].cards.extend(cards),
        }
    }

    fn cancel_drag(&mut self) {
        if let Some(d) = self.drag.take() {
            self.piles[d.src].cards.extend(d.cards);
        }
    }

    // ----------------------------------------------- Zeichnen

    fn fmt_time(&self) -> String {
        let sec = self.elapsed as i64;
        format!("{:02}:{:02}", sec / 60, sec % 60)
    }

    fn draw_piles(&self, s: &mut Surface) {
        let p_ = ui::pal();
        let (cw, ch) = (self.cw, self.ch);
        for (pi, p) in self.piles.iter().enumerate() {
            if p.kind == PKind::Removed {
                continue;
            }
            if p.cards.is_empty() {
                if p.kind != PKind::Grid {
                    c::draw_slot(s, Rect::new(p.x, p.y, cw, ch), SLOT);
                }
                if p.kind == PKind::Stock && self.can_recycle() {
                    self.draw_recycle(s, p, cw, ch);
                }
                continue;
            }
            match p.kind {
                PKind::Stock => {
                    let top = p.top().expect("karte");
                    let img = if top.face_up { self.renderer.get(top, cw, ch) } else { self.renderer.back(cw, ch) };
                    s.bl(&img, (p.x, p.y));
                    if p.cards.len() > 1 {
                        let n = self.tiny.render(&p.cards.len().to_string(), true, p_.text_dim);
                        s.bl(&n, (p.x + 2, p.y + ch + 2));
                    }
                }
                PKind::Waste | PKind::Foundation | PKind::Cell | PKind::Grid => {
                    s.bl(&self.renderer.get(p.top().expect("karte"), cw, ch), (p.x, p.y));
                }
                _ => {
                    for (card, r) in p.cards.iter().zip(p.rects(cw, ch)) {
                        s.bl(&self.renderer.get(card, cw, ch), (r.x, r.y));
                    }
                }
            }
            if let Some((spi, sci)) = self.sel {
                if spi == pi {
                    let rects = p.rects(cw, ch);
                    if !rects.is_empty() {
                        let ci = sci.min(rects.len() - 1);
                        let sel_r = if rects.len() > ci + 1 { rects[ci].unionall(&rects[ci..]) } else { rects[ci] };
                        draw::rect_r(s, p_.gold, sel_r.inflate(4, 4), 2, 6);
                    }
                }
            }
        }
    }

    fn draw_recycle(&self, s: &mut Surface, p: &Pile, cw: i32, ch: i32) {
        let (cx, cy) = (p.x + cw / 2, p.y + ch / 2);
        let r = 8.max(cw.min(ch) / 4);
        let col = self.b.accent;
        draw::arc(s, col, Rect::new(cx - r, cy - r, 2 * r, 2 * r), -0.35, 4.9, 2.max(r / 4));
        let tip = (cx + (r as f64 * 0.95) as i32, cy - (r as f64 * 0.33) as i32);
        draw::polygon(s, col, &[tip, (tip.0 - r / 2, tip.1 - r / 6), (tip.0 - r / 6, tip.1 + r / 2)], 0);
    }

    fn draw_drag(&self, s: &mut Surface) {
        let Some(d) = &self.drag else {
            return;
        };
        let x = d.pos.0 - d.off.0;
        let y = d.pos.1 - d.off.1;
        let p = &self.piles[d.src];
        let dy = if p.dy_up != 0 { p.dy_up } else { (self.ch as f64 * 0.28) as i32 };
        for (i, card) in d.cards.iter().enumerate() {
            s.bl(&self.renderer.get(card, self.cw, self.ch), (x, y + i as i32 * dy));
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        s.bl(&self.hud_bg, (0, 0));
        draw::line(s, self.b.accent, (0, self.hud_h), (w, self.hud_h), 2);
        let cy = self.hud_h / 2;
        let name = self.small.render(&t(&format!("sol.mode.{}", self.vkey())), true, self.b.accent);
        s.blr(&name, name.get_rect().midleft_at((12, cy)));
        let mid_txt = format!(
            "{}   ·   {}   ·   {}",
            tf("common.points", &[("score", &0.max(self.b.score).to_string())]),
            tf("sol.moves", &[("n", &self.moves.to_string())]),
            self.fmt_time()
        );
        let mid = self.mono.render(&mid_txt, true, p.text);
        s.blr(&mid, mid.get_rect().center_at((w / 2, cy)));
        let mut x = w - 12;
        for txt in self.hud_extra() {
            let img = self.small.render(&txt, true, p.text_dim);
            s.blr(&img, img.get_rect().midright_at((x, cy)));
            x -= img.w() + 16;
        }
        if let Some(m) = &self.msg {
            if !m.is_empty() {
                let img = self.small.render(m, true, p.gold);
                s.blr(&img, img.get_rect().center_at((w / 2, self.hud_h + 14)));
            }
        }
        let hint = self.tiny.render(&t("sol.hint"), true, p.text_faint);
        s.blr(&hint, hint.get_rect().midbottom_at((w / 2, self.b.height - 4)));
    }

    fn draw_result(&self, s: &mut Surface) {
        let p = ui::pal();
        s.bl(&self.overlay, (0, 0));
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let head = self.huge.render(&tf("sol.win", &[("bonus", &self.win_bonus.to_string())]), true, p.gold);
        let pw = (self.b.width - 40).min(380.max(head.w() + 70));
        let panel = Rect::new(cx - pw / 2, cy - 96, pw, 192);
        ui::draw_panel_ex(s, panel, None, None, None, true, Some(self.b.accent));
        s.blr(&head, head.get_rect().center_at((cx, cy - 46)));
        let sc = self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        s.blr(&sc, sc.get_rect().center_at((cx, cy + 10)));
        let hint = self.small.render(&t("sol.retry"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 52)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background_ex(s, w, h, false, None);
        let key = self.vkey();
        ui::draw_title_ex(s, w, &t(&format!("sol.mode.{key}")), Some(&t(&format!("sol.subtitle.{key}"))), (h as f64 * 0.14) as i32, Some(&self.huge), None, Some(self.b.accent));
        if self.setup_rows() > 0 {
            if let Some(r) = self.option_rects.first() {
                ui::draw_button_ex(s, *r, &self.setup_row_label(), &self.small, false, None, None, Some(self.b.accent));
            }
        }
        ui::draw_button_ex(s, self.start_rect, &t("common.start"), &self.b.font, true, None, None, Some(self.b.accent));
        ui::draw_footer_ex(s, w, h, &t("sol.setup_hint"), Some(&self.tiny));
    }
}

impl Game for SolitaireGame {
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
        self.vk = match self.b.mode.as_str() {
            "spider" => VKind::Spider,
            "freecell" => VKind::FreeCell,
            "pyramid" => VKind::Pyramid,
            "tripeaks" => VKind::TriPeaks,
            _ => VKind::Klondike,
        };
        self.make_fonts();
        self.renderer = CardRenderer::new(self.b.accent);
        self.hud_h = 34.max((self.b.height as f64 * 0.075) as i32);
        self.rebuild_static();
        self.piles.clear();
        self.sel = None;
        self.drag = None;
        self.undo_stack.clear();
        self.elapsed = 0.0;
        self.moves = 0;
        self.won = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.hud_h = 34.max((self.b.height as f64 * 0.075) as i32);
        self.renderer.clear();
        self.rebuild_static();
        self.build_setup_layout();
        if self.state == State::Play {
            self.layout();
        }
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.b.game_over {
            if ev.kind == Kind::KeyDown {
                if ev.key_in(&["Return", "space", "r", "R"]) {
                    self.new_deal();
                } else if ev.key_in(&["s", "S"]) {
                    self.state = State::Setup;
                    self.b.play_sound("click");
                }
            }
            return;
        }
        match ev.kind {
            Kind::KeyDown => match ev.key.as_str() {
                "u" | "U" => self.undo(),
                "r" | "R" => self.new_deal(),
                "s" | "S" => {
                    self.cancel_drag();
                    self.state = State::Setup;
                    self.b.play_sound("click");
                }
                "space" | "Return" => {
                    let r = self.on_stock();
                    self.apply_record(r);
                }
                _ => {}
            },
            Kind::MouseDown => {
                if ev.button == 3 {
                    let pi = match self.hit(ev.pos) {
                        Some((pi, _)) => Some(pi),
                        None => self.pile_at(ev.pos),
                    };
                    if let Some(pi) = pi {
                        let r = self.on_right(pi);
                        self.apply_record(r);
                    }
                    return;
                }
                self.on_down(ev.pos);
            }
            Kind::MouseMove => {
                if let Some(d) = self.drag.as_mut() {
                    d.pos = ev.pos;
                    let dx = ev.pos.0 - d.start.0;
                    let dy = ev.pos.1 - d.start.1;
                    if dx * dx + dy * dy > DRAG_PX * DRAG_PX {
                        d.moved = true;
                    }
                }
            }
            Kind::MouseUp => self.on_up(ev.pos),
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.state != State::Play || self.b.game_over {
            return;
        }
        self.elapsed += dt;
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.vk == VKind::Spider {
            self.b.score = 0.max(500 - self.v_moves + 100 * self.sequences);
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        s.bl(&self.felt, (0, 0));
        self.draw_piles(&mut s);
        if self.drag.is_some() {
            self.draw_drag(&mut s);
        }
        self.draw_hud(&mut s);
        if self.b.game_over {
            self.draw_result(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "solitaire",
    class_name: "SolitaireGame",
    supports_multiplayer: false,
    modes: &[
        ("klondike", "sol.mode.klondike"),
        ("spider", "sol.mode.spider"),
        ("freecell", "sol.mode.freecell"),
        ("pyramid", "sol.mode.pyramid"),
        ("tripeaks", "sol.mode.tripeaks"),
    ],
    make: SolitaireGame::new,
};
