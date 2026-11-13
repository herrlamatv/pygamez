//! cards.rs
//! ========
//! Gemeinsames Spielkarten-Werkzeug (aus `games/cards.py`).
//!
//! Alles mit Zeichen-Primitiven, keine Bilddateien: `Card`, `make_deck`,
//! `CardRenderer` (Vorder-/Rueckseiten mit Zwischenspeicher), `make_felt`
//! (Filz-Tisch), `draw_slot`, `fan_rects`, `hit_index`.
//!
//! Python unterscheidet Karten ueber `id(card)`; hier traegt jede Karte eine
//! eindeutige `id` aus einem Zaehler.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use pgz_engine::prelude::*;

/// Pik
pub const SUIT_SPADE: u8 = 0;
/// Herz
pub const SUIT_HEART: u8 = 1;
/// Karo
pub const SUIT_DIAMOND: u8 = 2;
/// Kreuz
pub const SUIT_CLUB: u8 = 3;

/// Rote Farbe der Herz-/Karo-Zeichen.
pub const COL_RED: Color = Color::rgb(198, 58, 62);
/// Schwarze Farbe der Pik-/Kreuz-Zeichen.
pub const COL_BLACK: Color = Color::rgb(36, 40, 52);
/// Kartenflaeche.
pub const COL_FACE: Color = Color::rgb(240, 242, 247);
/// Kartenrand.
pub const COL_FACE_EDGE: Color = Color::rgb(150, 158, 178);

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// `rank_label(rank)`
pub fn rank_label(rank: u8) -> String {
    match rank {
        1 => "A".into(),
        11 => "J".into(),
        12 => "Q".into(),
        13 => "K".into(),
        r => r.to_string(),
    }
}

/// `is_red(suit)`
pub fn is_red(suit: u8) -> bool {
    suit == SUIT_HEART || suit == SUIT_DIAMOND
}

/// `Card` - Rang 1..13, Farbe 0..3, aufgedeckt?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Card {
    /// Rang 1 (Ass) bis 13 (Koenig).
    pub rank: u8,
    /// Farbe 0..3.
    pub suit: u8,
    /// Aufgedeckt?
    pub face_up: bool,
    /// Ersatz fuer Pythons `id(card)`.
    pub id: u64,
}

impl Card {
    /// `Card(rank, suit, face_up)`
    pub fn new(rank: u8, suit: u8, face_up: bool) -> Card {
        Card { rank, suit, face_up, id: NEXT_ID.fetch_add(1, Ordering::Relaxed) }
    }

    /// `card.red`
    pub fn red(&self) -> bool {
        is_red(self.suit)
    }
}

/// `make_deck(suits, copies)`
pub fn make_deck(suits: &[u8], copies: usize) -> Vec<Card> {
    let mut deck = Vec::with_capacity(suits.len() * 13 * copies);
    for _ in 0..copies {
        for &suit in suits {
            for rank in 1..=13 {
                deck.push(Card::new(rank, suit, false));
            }
        }
    }
    deck
}

/// Farbsymbol mit Radius r um (cx, cy).
pub fn draw_suit(surf: &mut Surface, suit: u8, cx: i32, cy: i32, r: i32, col: Color) {
    let r = 2.max(r);
    let fd = py::fdiv32;
    match suit {
        SUIT_HEART => {
            let rr = 2.max((r as f64 * 0.55) as i32);
            draw::circle(surf, col, (cx - rr + 1, cy - fd(rr, 2)), rr, 0);
            draw::circle(surf, col, (cx + rr - 1, cy - fd(rr, 2)), rr, 0);
            draw::polygon(
                surf,
                col,
                &[(cx - 2 * rr + 1, cy - fd(rr, 6)), (cx + 2 * rr - 1, cy - fd(rr, 6)), (cx, cy + (rr as f64 * 1.6) as i32)],
                0,
            );
        }
        SUIT_DIAMOND => {
            let k = (r as f64 * 0.7) as i32;
            draw::polygon(surf, col, &[(cx, cy - r), (cx + k, cy), (cx, cy + r), (cx - k, cy)], 0);
        }
        SUIT_SPADE => {
            let rr = 2.max((r as f64 * 0.55) as i32);
            draw::circle(surf, col, (cx - rr + 1, cy + fd(rr, 2)), rr, 0);
            draw::circle(surf, col, (cx + rr - 1, cy + fd(rr, 2)), rr, 0);
            draw::polygon(
                surf,
                col,
                &[(cx - 2 * rr + 1, cy + fd(rr, 6)), (cx + 2 * rr - 1, cy + fd(rr, 6)), (cx, cy - (rr as f64 * 1.6) as i32)],
                0,
            );
            let fuss = cy + (rr as f64 * 1.7) as i32;
            draw::polygon(surf, col, &[(cx - fd(rr, 2), fuss), (cx + fd(rr, 2), fuss), (cx, cy + fd(rr, 2))], 0);
        }
        _ => {
            let rr = 2.max((r as f64 * 0.45) as i32);
            draw::circle(surf, col, (cx, cy - rr), rr, 0);
            draw::circle(surf, col, (cx - rr, cy + fd(rr, 2)), rr, 0);
            draw::circle(surf, col, (cx + rr, cy + fd(rr, 2)), rr, 0);
            let fuss = cy + (rr as f64 * 2.1) as i32;
            draw::polygon(surf, col, &[(cx - fd(rr, 2), fuss), (cx + fd(rr, 2), fuss), (cx, cy)], 0);
        }
    }
}

/// `CardRenderer` - rendert und speichert Kartenflaechen.
pub struct CardRenderer {
    /// Rueckseiten-Farbe.
    pub accent: Color,
    faces: RefCell<HashMap<(u8, u8, i32, i32), Surface>>,
    backs: RefCell<HashMap<(i32, i32), Surface>>,
}

impl CardRenderer {
    /// `CardRenderer(accent)`
    pub fn new(accent: Color) -> CardRenderer {
        CardRenderer { accent, faces: RefCell::new(HashMap::new()), backs: RefCell::new(HashMap::new()) }
    }

    /// `clear()`
    pub fn clear(&self) {
        self.faces.borrow_mut().clear();
        self.backs.borrow_mut().clear();
    }

    /// `get(card, w, h)`
    pub fn get(&self, card: &Card, w: i32, h: i32) -> Surface {
        if !card.face_up {
            return self.back(w, h);
        }
        let key = (card.rank, card.suit, w, h);
        if let Some(s) = self.faces.borrow().get(&key) {
            return s.clone();
        }
        let s = Self::face(card.rank, card.suit, w, h);
        self.faces.borrow_mut().insert(key, s.clone());
        s
    }

    /// `back(w, h)`
    pub fn back(&self, w: i32, h: i32) -> Surface {
        if let Some(s) = self.backs.borrow().get(&(w, h)) {
            return s.clone();
        }
        let mut surf = pg::surface_alpha(w, h);
        let rad = 3.max(w / 8);
        let base = ui::mix(self.accent, (16, 20, 28), 0.30);
        let dark = ui::mix(base, (0, 0, 0), 0.35);
        draw::rect_r(&mut surf, COL_FACE, (0, 0, w, h), 0, rad);
        let inner = Rect::new(2, 2, w - 4, h - 4);
        let irad = 2.max(rad - 2);
        draw::rect_r(&mut surf, base, inner, 0, irad);
        let pat = inner.inflate(-4, -4);
        let step = 6.max(w / 5);
        surf.set_clip(Some(pat));
        let mut yy = pat.y;
        while yy < pat.bottom() {
            let mut xx = pat.x;
            while xx < pat.right() {
                draw::line(&mut surf, dark, (xx, yy + step / 2), (xx + step / 2, yy), 1);
                draw::line(&mut surf, dark, (xx + step / 2, yy), (xx + step, yy + step / 2), 1);
                xx += step;
            }
            yy += step;
        }
        surf.set_clip(None);
        draw::rect_r(&mut surf, dark, inner, 1, irad);
        draw::rect_r(&mut surf, COL_FACE_EDGE, (0, 0, w, h), 1, rad);
        self.backs.borrow_mut().insert((w, h), surf.clone());
        surf
    }

    fn face(rank: u8, suit: u8, w: i32, h: i32) -> Surface {
        let mut surf = pg::surface_alpha(w, h);
        let rad = 3.max(w / 8);
        draw::rect_r(&mut surf, COL_FACE, (0, 0, w, h), 0, rad);
        draw::rect_r(&mut surf, COL_FACE_EDGE, (0, 0, w, h), 1, rad);
        let col = if is_red(suit) { COL_RED } else { COL_BLACK };
        let fnt = ui::fb(10.max(h / 5));
        let label = rank_label(rank);
        let img = fnt.render(&label, true, col);
        let rand = 2.max(w / 12);
        surf.bl(&img, (rand, 2));
        let pr = 2.max(w / 10);
        draw_suit(&mut surf, suit, rand + img.w() / 2, 4 + img.h() + pr, pr, col);
        let img2 = transform::rotate(&img, 180.0);
        surf.bl(&img2, (w - img2.w() - rand, h - img2.h() - 2));
        let big = 4.max(w / 4);
        if rank > 10 || rank == 1 {
            let bfnt = ui::fb(14.max(h / 3));
            let bimg = bfnt.render(&label, true, col);
            surf.blr(&bimg, bimg.get_rect().center_at((w / 2, h / 2 - big / 2)));
            draw_suit(&mut surf, suit, w / 2, h / 2 + big, big / 2 + 2, col);
        } else {
            draw_suit(&mut surf, suit, w / 2, h / 2, big, col);
        }
        surf
    }
}

/// `make_felt(w, h, base, edge)` - Filz-Hintergrund einmal rendern.
pub fn make_felt(w: i32, h: i32, base: Color, edge: Color) -> Surface {
    let mut surf = pg::surface(1.max(w), 1.max(h));
    let top = ui::mix(base, (255, 255, 255), 0.10);
    for y in 0..h {
        let f = y as f64 / 1.max(h - 1) as f64;
        surf.fill_r(ui::mix(top, edge, f * 0.85), (0, y, w, 1));
    }
    let mut glow = pg::surface_alpha(w, h);
    let (gw, gh) = ((w as f64 * 0.86) as i32, (h as f64 * 0.62) as i32);
    draw::ellipse(&mut glow, rgba(255, 255, 255, 14), (w / 2 - gw / 2, (h as f64 * 0.14) as i32, gw, gh), 0);
    draw::ellipse(
        &mut glow,
        rgba(255, 255, 255, 10),
        (w / 2 - (gw as f64 * 0.42) as i32, (h as f64 * 0.20) as i32, (gw as f64 * 0.84) as i32, (gh as f64 * 0.8) as i32),
        0,
    );
    surf.bl(&glow, (0, 0));
    draw::rect(&mut surf, edge, (0, 0, w, h), 6);
    draw::rect(&mut surf, ui::mix(edge, base, 0.5), (6, 6, w - 12, h - 12), 2);
    draw::rect(&mut surf, ui::mix(base, (255, 255, 255), 0.06), (8, 8, w - 16, h - 16), 1);
    surf
}

/// `draw_slot(surf, rect, color)` - gestrichelte Umrandung einer leeren Ablage.
pub fn draw_slot(surf: &mut Surface, r: Rect, color: Color) {
    let rad = 3.max(r.w / 8);
    let step = 7;
    let halb = rgb((color.r as f64 * 0.5) as i32, (color.g as f64 * 0.5) as i32, (color.b as f64 * 0.5) as i32);
    draw::rect_r(surf, halb, r, 1, rad);
    let mut x = r.x + rad;
    while x < r.right() - rad {
        let x2 = (x + step).min(r.right() - rad);
        draw::line(surf, color, (x, r.y), (x2, r.y), 1);
        draw::line(surf, color, (x, r.bottom() - 1), (x2, r.bottom() - 1), 1);
        x += step * 2;
    }
    let mut y = r.y + rad;
    while y < r.bottom() - rad {
        let y2 = (y + step).min(r.bottom() - rad);
        draw::line(surf, color, (r.x, y), (r.x, y2), 1);
        draw::line(surf, color, (r.right() - 1, y), (r.right() - 1, y2), 1);
        y += step * 2;
    }
}

/// `fan_rects(x, y, cards, w, h, dy_down, dy_up)`
pub fn fan_rects(x: f64, y: f64, cards: &[Card], w: i32, h: i32, dy_down: f64, dy_up: f64) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(cards.len());
    let mut yy = y;
    for card in cards {
        rects.push(Rect::new(x as i32, yy as i32, w, h));
        yy += if card.face_up { dy_up } else { dy_down };
    }
    rects
}

/// `hit_index(rects, pos)` - oberste getroffene Karte.
pub fn hit_index(rects: &[Rect], pos: (i32, i32)) -> Option<usize> {
    (0..rects.len()).rev().find(|&i| rects[i].hit(pos))
}
