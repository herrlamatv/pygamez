//! chess_engine.rs
//! ===============
//! Regeln, Notation und KI fuer Schach - uebertragen aus
//! `games/chess_engine.py` (gleiche Zuggenerierungs-Reihenfolge, gleiche
//! Bewertung, gleiche Stufen: bei gleichem Knotenlimit derselbe Zug).
//!
//! 0x88-Brett (128 Felder, Index = Reihe * 16 + Linie). Figuren sind Zahlen:
//! Typ (1 = Bauer .. 6 = Koenig) plus 8 fuer Schwarz. Zuege sind
//! `von | nach << 8 | Umwandlung << 16 | Art << 20`; eine Rochade ist intern
//! "Koenig schlaegt eigenen Turm" (Chess960-tauglich).
//!
//! Die Suche ist in Python ein fortsetzbarer Generator, der nach
//! `slice_end` (perf_counter) abgibt. Hier rechnet [`Search::run`] am Stueck;
//! ein Abbruch von aussen geht ueber [`Search::stop_flag`] (das Spiel laesst
//! die Suche ausserhalb des Pruefstands in einem eigenen Thread laufen).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use pgz_engine::py;

/// Weiss.
pub const WHITE: i32 = 0;
/// Schwarz.
pub const BLACK: i32 = 1;
/// Bauer.
pub const PAWN: i32 = 1;
/// Springer.
pub const KNIGHT: i32 = 2;
/// Laeufer.
pub const BISHOP: i32 = 3;
/// Turm.
pub const ROOK: i32 = 4;
/// Dame.
pub const QUEEN: i32 = 5;
/// Koenig.
pub const KING: i32 = 6;

/// Normaler Zug.
pub const M_NORMAL: i32 = 0;
/// Bauern-Doppelschritt.
pub const M_DOUBLE: i32 = 1;
/// En passant.
pub const M_EP: i32 = 2;
/// Rochade.
pub const M_CASTLE: i32 = 3;

const KNIGHT_D: [i32; 8] = [33, 31, 18, 14, -14, -18, -31, -33];
const KING_D: [i32; 8] = [17, 16, 15, 1, -1, -15, -16, -17];
const BISHOP_D: [i32; 4] = [17, 15, -15, -17];
const ROOK_D: [i32; 4] = [16, 1, -1, -16];

/// Figurenbuchstaben.
pub const LETTERS: &[u8; 7] = b" PNBRQK";
/// Linien.
pub const FILES: &[u8; 8] = b"abcdefgh";

/// Mattwert.
pub const MATE: i32 = 30000;
/// Unendlich.
pub const INF: i32 = 32000;
/// Groesste Suchtiefe in Halbzuegen.
pub const MAX_PLY: i32 = 64;

/// Die 64 Felder in 0x88-Zaehlung (a1, b1, .. h8).
pub fn squares() -> impl Iterator<Item = i32> {
    (0..8).flat_map(|r| (0..8).map(move |f| r * 16 + f))
}

#[inline]
fn off(s: i32) -> bool {
    s & 0x88 != 0
}

/// `sq_name(sq)` - "e4".
pub fn sq_name(sq: i32) -> String {
    format!("{}{}", FILES[(sq & 7) as usize] as char, (sq >> 4) + 1)
}

/// `sq_parse(name)`
pub fn sq_parse(name: &str) -> Option<i32> {
    let b = name.as_bytes();
    if b.len() < 2 {
        return None;
    }
    let f = FILES.iter().position(|c| *c == b[0])? as i32;
    let r = (b[1] as char).to_digit(10)? as i32;
    Some((r - 1) * 16 + f)
}

/// `piece_char(p)` - Grossbuchstabe fuer Weiss.
pub fn piece_char(p: i32) -> char {
    let ch = LETTERS[(p & 7) as usize] as char;
    if p >> 3 != 0 {
        ch.to_ascii_lowercase()
    } else {
        ch
    }
}

// ----------------------------------------------------------------- Zobrist

struct Zobrist {
    piece: Vec<[u64; 128]>,
    side: u64,
    castle: [[u64; 8]; 4],
    ep: [u64; 8],
}

fn zobrist() -> &'static Zobrist {
    static Z: OnceLock<Zobrist> = OnceLock::new();
    Z.get_or_init(|| {
        // Feste 64-Bit-Zufallszahlen (xorshift64*) - reproduzierbar.
        let count = 16 * 128 + 1 + 32 + 8;
        let mut x: u64 = 0x9E3779B97F4A7C15;
        let mut zn = Vec::with_capacity(count);
        for _ in 0..count {
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            zn.push(x.wrapping_mul(0x2545F4914F6CDD1D));
        }
        let mut piece = vec![[0u64; 128]; 16];
        for (i, row) in piece.iter_mut().enumerate() {
            row.copy_from_slice(&zn[i * 128..(i + 1) * 128]);
        }
        let mut castle = [[0u64; 8]; 4];
        for (i, row) in castle.iter_mut().enumerate() {
            row.copy_from_slice(&zn[16 * 128 + 1 + i * 8..16 * 128 + 1 + (i + 1) * 8]);
        }
        let mut ep = [0u64; 8];
        ep.copy_from_slice(&zn[16 * 128 + 33..16 * 128 + 41]);
        Zobrist { piece, side: zn[16 * 128], castle, ep }
    })
}

fn castle_key(castle: &[i32; 4]) -> u64 {
    let z = zobrist();
    let mut h = 0;
    for (i, c) in castle.iter().enumerate() {
        if *c >= 0 {
            h ^= z.castle[i][(c & 7) as usize];
        }
    }
    h
}

// ------------------------------------------------------------ Bewertung

const MG_VALUE: [i32; 7] = [0, 82, 337, 365, 477, 1025, 0];
const EG_VALUE: [i32; 7] = [0, 94, 281, 297, 512, 936, 0];
const PHASE_INC: [i32; 7] = [0, 0, 1, 1, 2, 4, 0];

#[rustfmt::skip]
const MG_TABLES: [[i32; 64]; 6] = [
    [0, 0, 0, 0, 0, 0, 0, 0,
     98, 134, 61, 95, 68, 126, 34, -11,
     -6, 7, 26, 31, 65, 56, 25, -20,
     -14, 13, 6, 21, 23, 12, 17, -23,
     -27, -2, -5, 12, 17, 6, 10, -25,
     -26, -4, -4, -10, 3, 3, 33, -12,
     -35, -1, -20, -23, -15, 24, 38, -22,
     0, 0, 0, 0, 0, 0, 0, 0],
    [-167, -89, -34, -49, 61, -97, -15, -107,
     -73, -41, 72, 36, 23, 62, 7, -17,
     -47, 60, 37, 65, 84, 129, 73, 44,
     -9, 17, 19, 53, 37, 69, 18, 22,
     -13, 4, 16, 13, 28, 19, 21, -8,
     -23, -9, 12, 10, 19, 17, 25, -16,
     -29, -53, -12, -3, -1, 18, -14, -19,
     -105, -21, -58, -33, -17, -28, -19, -23],
    [-29, 4, -82, -37, -25, -42, 7, -8,
     -26, 16, -18, -13, 30, 59, 18, -47,
     -16, 37, 43, 40, 35, 50, 37, -2,
     -4, 5, 19, 50, 37, 37, 7, -2,
     -6, 13, 13, 26, 34, 12, 10, 4,
     0, 15, 15, 15, 14, 27, 18, 10,
     4, 15, 16, 0, 7, 21, 33, 1,
     -33, -3, -14, -21, -13, -12, -39, -21],
    [32, 42, 32, 51, 63, 9, 31, 43,
     27, 32, 58, 62, 80, 67, 26, 44,
     -5, 19, 26, 36, 17, 45, 61, 16,
     -24, -11, 7, 26, 24, 35, -8, -20,
     -36, -26, -12, -1, 9, -7, 6, -23,
     -45, -25, -16, -17, 3, 0, -5, -33,
     -44, -16, -20, -9, -1, 11, -6, -71,
     -19, -13, 1, 17, 16, 7, -37, -26],
    [-28, 0, 29, 12, 59, 44, 43, 45,
     -24, -39, -5, 1, -16, 57, 28, 54,
     -13, -17, 7, 8, 29, 56, 47, 57,
     -27, -27, -16, -16, -1, 17, -2, 1,
     -9, -26, -9, -10, -2, -4, 3, -3,
     -14, 2, -11, -2, -5, 2, 14, 5,
     -35, -8, 11, 2, 8, 15, -3, 1,
     -1, -18, -9, 10, -15, -25, -31, -50],
    [-65, 23, 16, -15, -56, -34, 2, 13,
     29, -1, -20, -7, -8, -4, -38, -29,
     -9, 24, 2, -16, -20, 6, 22, -22,
     -17, -20, -12, -27, -30, -25, -14, -36,
     -49, -1, -27, -39, -46, -44, -33, -51,
     -14, -14, -22, -46, -44, -30, -15, -27,
     1, 7, -8, -64, -43, -16, 9, 8,
     -15, 36, 12, -54, 8, -28, 24, 14],
];

#[rustfmt::skip]
const EG_TABLES: [[i32; 64]; 6] = [
    [0, 0, 0, 0, 0, 0, 0, 0,
     178, 173, 158, 134, 147, 132, 165, 187,
     94, 100, 85, 67, 56, 53, 82, 84,
     32, 24, 13, 5, -2, 4, 17, 17,
     13, 9, -3, -7, -7, -8, 3, -1,
     4, 7, -6, 1, 0, -5, -1, -8,
     13, 8, 8, 10, 13, 0, 2, -7,
     0, 0, 0, 0, 0, 0, 0, 0],
    [-58, -38, -13, -28, -31, -27, -63, -99,
     -25, -8, -25, -2, -9, -25, -24, -52,
     -24, -20, 10, 9, -1, -9, -19, -41,
     -17, 3, 22, 22, 22, 11, 8, -18,
     -18, -6, 16, 25, 16, 17, 4, -18,
     -23, -3, -1, 15, 10, -3, -20, -22,
     -42, -20, -10, -5, -2, -20, -23, -44,
     -29, -51, -23, -15, -22, -18, -50, -64],
    [-14, -21, -11, -8, -7, -9, -17, -24,
     -8, -4, 7, -12, -3, -13, -4, -14,
     2, -8, 0, -1, -2, 6, 0, 4,
     -3, 9, 12, 9, 14, 10, 3, 2,
     -6, 3, 13, 19, 7, 10, -3, -9,
     -12, -3, 8, 10, 13, 3, -7, -15,
     -14, -18, -7, -1, 4, -9, -15, -27,
     -23, -9, -23, -5, -9, -16, -5, -17],
    [13, 10, 18, 15, 12, 12, 8, 5,
     11, 13, 13, 11, -3, 3, 8, 3,
     7, 7, 7, 5, 4, -3, -5, -3,
     4, 3, 13, 1, 2, 1, -1, 2,
     3, 5, 8, 4, -5, -6, -8, -11,
     -4, 0, -5, -1, -7, -12, -8, -16,
     -6, -6, 0, 2, -9, -9, -11, -3,
     -9, 2, 3, -1, -5, -13, 4, -20],
    [-9, 22, 22, 27, 27, 19, 10, 20,
     -17, 20, 32, 41, 58, 25, 30, 0,
     -20, 6, 9, 49, 47, 35, 19, 9,
     3, 22, 24, 45, 57, 40, 57, 36,
     -18, 28, 19, 47, 31, 34, 39, 23,
     -16, -27, 15, 6, 9, 17, 10, 5,
     -22, -23, -30, -16, -16, -23, -36, -32,
     -33, -28, -22, -43, -5, -32, -20, -41],
    [-74, -35, -18, -18, -11, 15, 4, -17,
     -12, 17, 14, 17, 17, 38, 23, 11,
     10, 17, 23, 15, 20, 45, 44, 13,
     -8, 22, 24, 27, 26, 33, 26, 3,
     -18, -4, 21, 24, 27, 23, 9, -11,
     -19, -3, 11, 21, 23, 16, 7, -9,
     -27, -11, 4, 13, 14, 4, -5, -17,
     -53, -34, -21, -11, -28, -14, -24, -43],
];

struct Pst {
    mg: Vec<[i32; 128]>,
    eg: Vec<[i32; 128]>,
}

fn build_pst(tables: &[[i32; 64]; 6], values: &[i32; 7]) -> Vec<[i32; 128]> {
    let mut out = vec![[0i32; 128]; 16];
    for typ in 1..7usize {
        for sq in squares() {
            let (r, f) = (sq >> 4, sq & 7);
            out[typ][sq as usize] = values[typ] + tables[typ - 1][((7 - r) * 8 + f) as usize];
            out[typ | 8][sq as usize] = values[typ] + tables[typ - 1][(r * 8 + f) as usize];
        }
    }
    out
}

fn pst() -> &'static Pst {
    static P: OnceLock<Pst> = OnceLock::new();
    P.get_or_init(|| Pst { mg: build_pst(&MG_TABLES, &MG_VALUE), eg: build_pst(&EG_TABLES, &EG_VALUE) })
}

const BISHOP_PAIR: (i32, i32) = (25, 45);
const DOUBLED: (i32, i32) = (8, 18);
const ISOLATED: (i32, i32) = (10, 12);
const PASSED_MG: [i32; 8] = [0, 5, 8, 15, 28, 50, 80, 0];
const PASSED_EG: [i32; 8] = [0, 10, 15, 28, 50, 85, 130, 0];
const ROOK_OPEN: (i32, i32) = (20, 10);
const ROOK_HALF: (i32, i32) = (10, 5);
const MOB_BASE: [i32; 7] = [0, 0, 4, 7, 7, 14, 0];
const MOB_MG: [i32; 7] = [0, 0, 4, 3, 2, 1, 0];
const MOB_EG: [i32; 7] = [0, 0, 4, 4, 4, 2, 0];
const SHIELD_NEAR: i32 = 0;
const SHIELD_FAR: i32 = 10;
const SHIELD_NONE: i32 = 25;
const SHIELD_OPEN: i32 = 15;
const TEMPO: i32 = 10;
/// Grobe Figurenwerte (Zugsortierung, Materialbilanz).
pub const ORDER_VALUE: [i32; 7] = [0, 1, 3, 3, 5, 9, 20];

/// Bauernstruktur-Eintrag: (mg, eg, Bauern je Linie weiss/schwarz).
type PawnEntry = (i32, i32, [i32; 8], [i32; 8]);

thread_local! {
    static PAWN_CACHE: RefCell<HashMap<u64, PawnEntry>> = RefCell::new(HashMap::new());
}

// ================================================================ Position

#[derive(Clone, Copy)]
struct Undo {
    m: i32,
    p: i32,
    cap: i32,
    castle: [i32; 4],
    ep: i32,
    half: i32,
    hash: u64,
    phash: u64,
    mg: [i32; 2],
    eg: [i32; 2],
    phase: i32,
}

/// Eine Schachstellung mit Zug ausfuehren/zuruecknehmen.
#[derive(Clone)]
pub struct Position {
    /// Brett (0x88).
    pub b: [i32; 128],
    /// Seite am Zug.
    pub side: i32,
    /// Turmfeld je Rochaderecht (wK, wQ, bK, bQ) oder -1.
    pub castle: [i32; 4],
    /// En-passant-Feld oder -1.
    pub ep: i32,
    /// Halbzuege seit Schlag/Bauernzug.
    pub half: i32,
    /// Zugnummer.
    pub full: i32,
    /// Koenigsfelder.
    pub kings: [i32; 2],
    /// Zobrist-Schluessel.
    pub hash: u64,
    phash: u64,
    mg: [i32; 2],
    eg: [i32; 2],
    phase: i32,
    counts: [i32; 16],
    stack: Vec<Undo>,
    hist: Vec<u64>,
    /// Chess960-Notation (Rochade als Koenig-schlaegt-Turm).
    pub chess960: bool,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            b: [0; 128],
            side: WHITE,
            castle: [-1; 4],
            ep: -1,
            half: 0,
            full: 1,
            kings: [-1, -1],
            hash: 0,
            phash: 0,
            mg: [0, 0],
            eg: [0, 0],
            phase: 0,
            counts: [0; 16],
            stack: Vec::new(),
            hist: Vec::new(),
            chess960: false,
        }
    }
}

impl Position {
    /// `copy()` - ohne Zugstapel (wie Python).
    pub fn copy(&self) -> Position {
        let mut p = self.clone();
        p.stack = Vec::new();
        p
    }

    fn recompute(&mut self) {
        let z = zobrist();
        let t = pst();
        self.mg = [0, 0];
        self.eg = [0, 0];
        self.phase = 0;
        self.counts = [0; 16];
        let mut h = 0u64;
        let mut ph = 0u64;
        for sq in squares() {
            let p = self.b[sq as usize];
            if p == 0 {
                continue;
            }
            let c = (p >> 3) as usize;
            self.mg[c] += t.mg[p as usize][sq as usize];
            self.eg[c] += t.eg[p as usize][sq as usize];
            self.phase += PHASE_INC[(p & 7) as usize];
            self.counts[p as usize] += 1;
            h ^= z.piece[p as usize][sq as usize];
            if p & 7 == PAWN {
                ph ^= z.piece[p as usize][sq as usize];
            }
            if p & 7 == KING {
                self.kings[c] = sq;
            }
        }
        if self.side == BLACK {
            h ^= z.side;
        }
        h ^= castle_key(&self.castle);
        if self.ep >= 0 {
            h ^= z.ep[(self.ep & 7) as usize];
        }
        self.hash = h;
        self.phash = ph;
        self.hist = vec![h];
    }

    // --------------------------------------------------------- Angriffe

    /// `attacked(sq, by)`
    pub fn attacked(&self, sq: i32, by: i32) -> bool {
        let b = &self.b;
        let (kn, kg, bi, rk, qu);
        if by == WHITE {
            let s = sq - 15;
            if !off(s) && b[s as usize] == 1 {
                return true;
            }
            let s = sq - 17;
            if !off(s) && b[s as usize] == 1 {
                return true;
            }
            (kn, kg, bi, rk, qu) = (2, 6, 3, 4, 5);
        } else {
            let s = sq + 15;
            if !off(s) && b[s as usize] == 9 {
                return true;
            }
            let s = sq + 17;
            if !off(s) && b[s as usize] == 9 {
                return true;
            }
            (kn, kg, bi, rk, qu) = (10, 14, 11, 12, 13);
        }
        for d in KNIGHT_D {
            let s = sq + d;
            if !off(s) && b[s as usize] == kn {
                return true;
            }
        }
        for d in KING_D {
            let s = sq + d;
            if !off(s) && b[s as usize] == kg {
                return true;
            }
        }
        for d in BISHOP_D {
            let mut s = sq + d;
            while !off(s) {
                let q = b[s as usize];
                if q != 0 {
                    if q == bi || q == qu {
                        return true;
                    }
                    break;
                }
                s += d;
            }
        }
        for d in ROOK_D {
            let mut s = sq + d;
            while !off(s) {
                let q = b[s as usize];
                if q != 0 {
                    if q == rk || q == qu {
                        return true;
                    }
                    break;
                }
                s += d;
            }
        }
        false
    }

    /// `in_check(color=None)`
    pub fn in_check(&self) -> bool {
        self.in_check_of(self.side)
    }

    /// `in_check(color)`
    pub fn in_check_of(&self, c: i32) -> bool {
        self.attacked(self.kings[c as usize], 1 - c)
    }

    // --------------------------------------------------------- Zuggenerator

    /// `gen_moves(tactical)` - pseudolegale Zuege.
    pub fn gen_moves(&mut self, tactical: bool) -> Vec<i32> {
        let us = self.side;
        let them = 1 - us;
        let mut moves = Vec::with_capacity(48);
        let (fwd, start_rank, promo_rank) = if us == WHITE { (16, 1, 7) } else { (-16, 6, 0) };
        let ep = self.ep;
        for sq in squares() {
            let p = self.b[sq as usize];
            if p == 0 || (p >> 3) != us {
                continue;
            }
            let typ = p & 7;
            if typ == PAWN {
                let to = sq + fwd;
                let promo = (to >> 4) == promo_rank;
                if self.b[to as usize] == 0 {
                    if promo {
                        moves.push(sq | to << 8 | QUEEN << 16);
                        if !tactical {
                            moves.push(sq | to << 8 | ROOK << 16);
                            moves.push(sq | to << 8 | BISHOP << 16);
                            moves.push(sq | to << 8 | KNIGHT << 16);
                        }
                    } else if !tactical {
                        moves.push(sq | to << 8);
                        if (sq >> 4) == start_rank && self.b[(to + fwd) as usize] == 0 {
                            moves.push(sq | (to + fwd) << 8 | M_DOUBLE << 20);
                        }
                    }
                }
                for t in [to - 1, to + 1] {
                    if off(t) {
                        continue;
                    }
                    let q = self.b[t as usize];
                    if q != 0 && (q >> 3) == them {
                        if promo {
                            moves.push(sq | t << 8 | QUEEN << 16);
                            if !tactical {
                                moves.push(sq | t << 8 | ROOK << 16);
                                moves.push(sq | t << 8 | BISHOP << 16);
                                moves.push(sq | t << 8 | KNIGHT << 16);
                            }
                        } else {
                            moves.push(sq | t << 8);
                        }
                    } else if t == ep {
                        moves.push(sq | t << 8 | M_EP << 20);
                    }
                }
            } else if typ == KNIGHT || typ == KING {
                let dirs: &[i32] = if typ == KNIGHT { &KNIGHT_D } else { &KING_D };
                for &d in dirs {
                    let t = sq + d;
                    if off(t) {
                        continue;
                    }
                    let q = self.b[t as usize];
                    if q == 0 {
                        if !tactical {
                            moves.push(sq | t << 8);
                        }
                    } else if (q >> 3) == them {
                        moves.push(sq | t << 8);
                    }
                }
                if typ == KING && !tactical {
                    self.gen_castles(sq, &mut moves);
                }
            } else {
                let dirs: &[i32] = if typ == BISHOP {
                    &BISHOP_D
                } else if typ == ROOK {
                    &ROOK_D
                } else {
                    &KING_D
                };
                for &d in dirs {
                    let mut t = sq + d;
                    while !off(t) {
                        let q = self.b[t as usize];
                        if q == 0 {
                            if !tactical {
                                moves.push(sq | t << 8);
                            }
                        } else {
                            if (q >> 3) == them {
                                moves.push(sq | t << 8);
                            }
                            break;
                        }
                        t += d;
                    }
                }
            }
        }
        moves
    }

    fn gen_castles(&mut self, k: i32, moves: &mut Vec<i32>) {
        let us = self.side;
        let c = self.castle;
        if c[(us * 2) as usize] < 0 && c[(us * 2 + 1) as usize] < 0 {
            return;
        }
        let base = if us == WHITE { 0 } else { 112 };
        if (k >> 4) != (base >> 4) {
            return;
        }
        let them = 1 - us;
        if self.attacked(k, them) {
            return;
        }
        let rook = ROOK | (us << 3);
        for i in 0..2 {
            let rsq = c[(us * 2 + i) as usize];
            if rsq < 0 || self.b[rsq as usize] != rook {
                continue;
            }
            let kt = base + if i == 0 { 6 } else { 2 };
            let rt = base + if i == 0 { 5 } else { 3 };
            let lo = k.min(kt).min(rsq).min(rt);
            let hi = k.max(kt).max(rsq).max(rt);
            let free = (lo..=hi).all(|s| self.b[s as usize] == 0 || s == k || s == rsq);
            if !free {
                continue;
            }
            let kp = self.b[k as usize];
            self.b[k as usize] = 0;
            self.b[rsq as usize] = 0;
            let mut safe = true;
            let step = if kt > k { 1 } else { -1 };
            let mut s = k;
            while s != kt {
                s += step;
                if self.attacked(s, them) {
                    safe = false;
                    break;
                }
            }
            self.b[k as usize] = kp;
            self.b[rsq as usize] = rook;
            if safe {
                moves.push(k | rsq << 8 | M_CASTLE << 20);
            }
        }
    }

    /// `legal_moves()`
    pub fn legal_moves(&mut self) -> Vec<i32> {
        let us = self.side;
        let mut res = Vec::new();
        for m in self.gen_moves(false) {
            self.make(m);
            if !self.attacked(self.kings[us as usize], 1 - us) {
                res.push(m);
            }
            self.unmake();
        }
        res
    }

    /// `has_legal_move()`
    pub fn has_legal_move(&mut self) -> bool {
        let us = self.side;
        for m in self.gen_moves(false) {
            self.make(m);
            let ok = !self.attacked(self.kings[us as usize], 1 - us);
            self.unmake();
            if ok {
                return true;
            }
        }
        false
    }

    /// `is_legal(m)`
    pub fn is_legal(&mut self, m: i32) -> bool {
        let us = self.side;
        self.make(m);
        let ok = !self.attacked(self.kings[us as usize], 1 - us);
        self.unmake();
        ok
    }

    // --------------------------------------------------------- Zug ausfuehren

    /// `make(m)`
    pub fn make(&mut self, m: i32) {
        let z = zobrist();
        let t = pst();
        let frm = m & 255;
        let to = (m >> 8) & 255;
        let flag = m >> 20;
        let us = self.side;
        let them = 1 - us;
        let p = self.b[frm as usize];
        let typ = p & 7;
        let cap = if flag == M_CASTLE { 0 } else { self.b[to as usize] };
        self.stack.push(Undo {
            m,
            p,
            cap,
            castle: self.castle,
            ep: self.ep,
            half: self.half,
            hash: self.hash,
            phash: self.phash,
            mg: self.mg,
            eg: self.eg,
            phase: self.phase,
        });
        let mut h = self.hash ^ z.side;
        if self.ep >= 0 {
            h ^= z.ep[(self.ep & 7) as usize];
        }
        self.ep = -1;
        let mut half = self.half + 1;
        let (usu, themu) = (us as usize, them as usize);
        let (fu, tu) = (frm as usize, to as usize);
        let pu = p as usize;
        if flag == M_CASTLE {
            let base = frm & 0x70;
            let (kt, rt) = if to > frm { (base + 6, base + 5) } else { (base + 2, base + 3) };
            let rp = self.b[tu];
            let rpu = rp as usize;
            self.b[fu] = 0;
            self.b[tu] = 0;
            self.b[kt as usize] = p;
            self.b[rt as usize] = rp;
            h ^= z.piece[pu][fu] ^ z.piece[pu][kt as usize] ^ z.piece[rpu][tu] ^ z.piece[rpu][rt as usize];
            self.mg[usu] += t.mg[pu][kt as usize] - t.mg[pu][fu] + t.mg[rpu][rt as usize] - t.mg[rpu][tu];
            self.eg[usu] += t.eg[pu][kt as usize] - t.eg[pu][fu] + t.eg[rpu][rt as usize] - t.eg[rpu][tu];
            self.kings[usu] = kt;
        } else {
            if flag == M_EP {
                let cs = if us == WHITE { to - 16 } else { to + 16 };
                let cp = self.b[cs as usize];
                let (cpu, csu) = (cp as usize, cs as usize);
                self.b[csu] = 0;
                h ^= z.piece[cpu][csu];
                self.phash ^= z.piece[cpu][csu];
                self.mg[themu] -= t.mg[cpu][csu];
                self.eg[themu] -= t.eg[cpu][csu];
                self.counts[cpu] -= 1;
                half = 0;
            } else if cap != 0 {
                let cu = cap as usize;
                h ^= z.piece[cu][tu];
                self.mg[themu] -= t.mg[cu][tu];
                self.eg[themu] -= t.eg[cu][tu];
                self.phase -= PHASE_INC[(cap & 7) as usize];
                self.counts[cu] -= 1;
                if cap & 7 == PAWN {
                    self.phash ^= z.piece[cu][tu];
                }
                half = 0;
            }
            self.b[fu] = 0;
            h ^= z.piece[pu][fu];
            self.mg[usu] -= t.mg[pu][fu];
            self.eg[usu] -= t.eg[pu][fu];
            let promo = (m >> 16) & 7;
            let np_;
            if promo != 0 {
                np_ = promo | (us << 3);
                self.counts[pu] -= 1;
                self.counts[np_ as usize] += 1;
                self.phase += PHASE_INC[promo as usize];
                self.phash ^= z.piece[pu][fu];
            } else {
                np_ = p;
                if typ == PAWN {
                    self.phash ^= z.piece[pu][fu] ^ z.piece[pu][tu];
                }
            }
            let npu = np_ as usize;
            self.b[tu] = np_;
            h ^= z.piece[npu][tu];
            self.mg[usu] += t.mg[npu][tu];
            self.eg[usu] += t.eg[npu][tu];
            if typ == PAWN {
                half = 0;
                if flag == M_DOUBLE {
                    let ep_pawn = PAWN | (them << 3);
                    if (!off(to - 1) && self.b[(to - 1) as usize] == ep_pawn) || (!off(to + 1) && self.b[(to + 1) as usize] == ep_pawn) {
                        self.ep = (frm + to) >> 1;
                        h ^= z.ep[(self.ep & 7) as usize];
                    }
                }
            } else if typ == KING {
                self.kings[usu] = to;
            }
        }
        let c = self.castle;
        if c != [-1; 4] {
            let mut nc = c;
            if typ == KING {
                nc[usu * 2] = -1;
                nc[usu * 2 + 1] = -1;
            }
            for v in nc.iter_mut() {
                if *v >= 0 && (*v == frm || *v == to) {
                    *v = -1;
                }
            }
            if nc != c {
                h ^= castle_key(&c) ^ castle_key(&nc);
                self.castle = nc;
            }
        }
        self.hash = h;
        self.half = half;
        if us == BLACK {
            self.full += 1;
        }
        self.side = them;
        self.hist.push(h);
    }

    /// `unmake()`
    pub fn unmake(&mut self) {
        let Some(u) = self.stack.pop() else {
            return;
        };
        self.hist.pop();
        let them = self.side;
        let us = 1 - them;
        self.side = us;
        if us == BLACK {
            self.full -= 1;
        }
        let m = u.m;
        let frm = m & 255;
        let to = (m >> 8) & 255;
        let flag = m >> 20;
        if flag == M_CASTLE {
            let base = frm & 0x70;
            let (kt, rt) = if to > frm { (base + 6, base + 5) } else { (base + 2, base + 3) };
            let rp = self.b[rt as usize];
            self.b[kt as usize] = 0;
            self.b[rt as usize] = 0;
            self.b[frm as usize] = u.p;
            self.b[to as usize] = rp;
            self.kings[us as usize] = frm;
        } else {
            let promo = (m >> 16) & 7;
            if promo != 0 {
                self.counts[(promo | (us << 3)) as usize] -= 1;
                self.counts[u.p as usize] += 1;
            }
            self.b[to as usize] = u.cap;
            self.b[frm as usize] = u.p;
            if u.cap != 0 {
                self.counts[u.cap as usize] += 1;
            }
            if flag == M_EP {
                let cp = PAWN | (them << 3);
                self.b[(if us == WHITE { to - 16 } else { to + 16 }) as usize] = cp;
                self.counts[cp as usize] += 1;
            }
            if u.p & 7 == KING {
                self.kings[us as usize] = frm;
            }
        }
        self.castle = u.castle;
        self.ep = u.ep;
        self.half = u.half;
        self.hash = u.hash;
        self.phash = u.phash;
        self.mg = u.mg;
        self.eg = u.eg;
        self.phase = u.phase;
    }

    /// `make_null()`
    pub fn make_null(&mut self) {
        let z = zobrist();
        self.stack.push(Undo {
            m: 0,
            p: 0,
            cap: 0,
            castle: self.castle,
            ep: self.ep,
            half: self.half,
            hash: self.hash,
            phash: self.phash,
            mg: self.mg,
            eg: self.eg,
            phase: self.phase,
        });
        let mut h = self.hash ^ z.side;
        if self.ep >= 0 {
            h ^= z.ep[(self.ep & 7) as usize];
        }
        self.ep = -1;
        self.hash = h;
        self.half = 0;
        self.side = 1 - self.side;
        self.hist.push(h);
    }

    /// `unmake_null()`
    pub fn unmake_null(&mut self) {
        let Some(u) = self.stack.pop() else {
            return;
        };
        self.hist.pop();
        self.side = 1 - self.side;
        self.ep = u.ep;
        self.half = u.half;
        self.hash = u.hash;
    }

    /// Anzahl der Zuege im Rueckgabe-Stapel.
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    // --------------------------------------------------------- Zustaende

    /// `is_capture(m)`
    pub fn is_capture(&self, m: i32) -> bool {
        let flag = m >> 20;
        flag == M_EP || (flag != M_CASTLE && self.b[((m >> 8) & 255) as usize] != 0)
    }

    /// `repetitions()`
    pub fn repetitions(&self) -> i32 {
        let h = self.hash;
        let hist = &self.hist;
        let mut n = 0;
        let mut i = hist.len() as i64 - 1;
        let stop = 0.max(hist.len() as i64 - 1 - self.half as i64);
        while i >= stop {
            if hist[i as usize] == h {
                n += 1;
            }
            i -= 2;
        }
        n
    }

    /// `insufficient_material()`
    pub fn insufficient_material(&self) -> bool {
        let c = &self.counts;
        if c[1] != 0 || c[9] != 0 || c[4] != 0 || c[12] != 0 || c[5] != 0 || c[13] != 0 {
            return false;
        }
        let minors = c[2] + c[3] + c[10] + c[11];
        if minors <= 1 {
            return true;
        }
        if c[2] != 0 || c[10] != 0 {
            return false;
        }
        let mut colors = [false; 2];
        for sq in squares() {
            if self.b[sq as usize] & 7 == BISHOP {
                colors[(((sq >> 4) + (sq & 7)) & 1) as usize] = true;
            }
        }
        colors.iter().filter(|c| **c).count() == 1
    }

    /// `can_mate(color)`
    pub fn can_mate(&self, color: i32) -> bool {
        let c = &self.counts;
        let o = color << 3;
        if c[(PAWN | o) as usize] != 0 || c[(ROOK | o) as usize] != 0 || c[(QUEEN | o) as usize] != 0 {
            return true;
        }
        c[(KNIGHT | o) as usize] + c[(BISHOP | o) as usize] >= 2
    }

    /// `non_pawn_material(color)`
    pub fn non_pawn_material(&self, color: i32) -> i32 {
        let c = &self.counts;
        let o = color << 3;
        c[(KNIGHT | o) as usize] + c[(BISHOP | o) as usize] + c[(ROOK | o) as usize] + c[(QUEEN | o) as usize]
    }

    /// `material(color)`
    pub fn material(&self, color: i32) -> i32 {
        let c = &self.counts;
        let o = color << 3;
        c[(PAWN | o) as usize] + 3 * c[(KNIGHT | o) as usize] + 3 * c[(BISHOP | o) as usize] + 5 * c[(ROOK | o) as usize] + 9 * c[(QUEEN | o) as usize]
    }

    /// `status()` - None oder "checkmate"/"stalemate"/"fifty"/"threefold"/"material".
    pub fn status(&mut self) -> Option<&'static str> {
        if !self.has_legal_move() {
            return Some(if self.in_check() { "checkmate" } else { "stalemate" });
        }
        if self.insufficient_material() {
            return Some("material");
        }
        if self.half >= 100 {
            return Some("fifty");
        }
        if self.repetitions() >= 3 {
            return Some("threefold");
        }
        None
    }

    // --------------------------------------------------------- Bewertung

    /// `evaluate()` - aus Sicht der Seite am Zug (Zentibauern).
    pub fn evaluate(&self) -> i32 {
        let b = &self.b;
        let c = &self.counts;
        let mut mg = self.mg[0] - self.mg[1];
        let mut eg = self.eg[0] - self.eg[1];
        if c[3] >= 2 {
            mg += BISHOP_PAIR.0;
            eg += BISHOP_PAIR.1;
        }
        if c[11] >= 2 {
            mg -= BISHOP_PAIR.0;
            eg -= BISHOP_PAIR.1;
        }
        let pe = PAWN_CACHE.with(|pc| pc.borrow().get(&self.phash).copied()).unwrap_or_else(|| self.pawn_eval());
        mg += pe.0;
        eg += pe.1;
        let (wf, bf) = (pe.2, pe.3);
        for sq in squares() {
            let p = b[sq as usize];
            if p == 0 {
                continue;
            }
            let typ = p & 7;
            if typ == PAWN || typ == KING {
                continue;
            }
            let col = p >> 3;
            let mut n = 0;
            if typ == KNIGHT {
                for d in KNIGHT_D {
                    let t = sq + d;
                    if !off(t) {
                        let q = b[t as usize];
                        if q == 0 || (q >> 3) != col {
                            n += 1;
                        }
                    }
                }
            } else {
                let dirs: &[i32] = if typ == BISHOP {
                    &BISHOP_D
                } else if typ == ROOK {
                    &ROOK_D
                } else {
                    &KING_D
                };
                for &d in dirs {
                    let mut t = sq + d;
                    while !off(t) {
                        let q = b[t as usize];
                        if q != 0 {
                            if (q >> 3) != col {
                                n += 1;
                            }
                            break;
                        }
                        n += 1;
                        t += d;
                    }
                }
            }
            n -= MOB_BASE[typ as usize];
            let mut dm = n * MOB_MG[typ as usize];
            let mut de = n * MOB_EG[typ as usize];
            if typ == ROOK {
                let f = (sq & 7) as usize;
                let own = if col == WHITE { wf[f] } else { bf[f] };
                if own == 0 {
                    if (if col == WHITE { bf[f] } else { wf[f] }) == 0 {
                        dm += ROOK_OPEN.0;
                        de += ROOK_OPEN.1;
                    } else {
                        dm += ROOK_HALF.0;
                        de += ROOK_HALF.1;
                    }
                }
            }
            if col == WHITE {
                mg += dm;
                eg += de;
            } else {
                mg -= dm;
                eg -= de;
            }
        }
        mg -= self.shield_penalty(WHITE, &wf, &bf);
        mg += self.shield_penalty(BLACK, &wf, &bf);
        if self.phase <= 8 {
            let mw = self.material(WHITE);
            let mb = self.material(BLACK);
            if mw >= mb + 4 && c[9] == 0 {
                eg += Self::mop_up(self.kings[0], self.kings[1]);
            } else if mb >= mw + 4 && c[1] == 0 {
                eg -= Self::mop_up(self.kings[1], self.kings[0]);
            }
        }
        let phase = self.phase.min(24);
        let score = py::fdiv((mg * phase + eg * (24 - phase)) as i64, 24) as i32;
        (if self.side == WHITE { score } else { -score }) + TEMPO
    }

    fn mop_up(win_k: i32, lose_k: i32) -> i32 {
        let (lf, lr) = (lose_k & 7, lose_k >> 4);
        let center = (3 - lf).max(lf - 4) + (3 - lr).max(lr - 4);
        let dist = ((win_k & 7) - lf).abs() + ((win_k >> 4) - lr).abs();
        10 * center + 4 * (14 - dist)
    }

    fn shield_penalty(&self, color: i32, wf: &[i32; 8], bf: &[i32; 8]) -> i32 {
        let k = self.kings[color as usize];
        let r = k >> 4;
        let rel = if color == WHITE { r } else { 7 - r };
        if rel > 1 {
            return 0;
        }
        let enemy_queen = self.counts[(QUEEN | ((1 - color) << 3)) as usize];
        let pawn = PAWN | (color << 3);
        let fwd = if color == WHITE { 16 } else { -16 };
        let f = k & 7;
        let mut pen = 0;
        for ff in [f - 1, f, f + 1] {
            if !(0..=7).contains(&ff) {
                continue;
            }
            let s1 = (k - f + ff) + fwd;
            if self.b[s1 as usize] == pawn {
                pen += SHIELD_NEAR;
                continue;
            }
            let s2 = s1 + fwd;
            if !off(s2) && self.b[s2 as usize] == pawn {
                pen += SHIELD_FAR;
                continue;
            }
            pen += SHIELD_NONE;
            if wf[ff as usize] == 0 && bf[ff as usize] == 0 {
                pen += SHIELD_OPEN;
            }
        }
        if enemy_queen != 0 {
            pen
        } else {
            pen / 2
        }
    }

    fn pawn_eval(&self) -> PawnEntry {
        let b = &self.b;
        let mut wf = [0i32; 8];
        let mut bf = [0i32; 8];
        let mut wmin = [8i32; 8];
        let mut bmax = [-1i32; 8];
        let mut wps = Vec::new();
        let mut bps = Vec::new();
        for sq in squares() {
            let p = b[sq as usize];
            if p == 1 {
                let f = (sq & 7) as usize;
                wf[f] += 1;
                if (sq >> 4) < wmin[f] {
                    wmin[f] = sq >> 4;
                }
                wps.push(sq);
            } else if p == 9 {
                let f = (sq & 7) as usize;
                bf[f] += 1;
                if (sq >> 4) > bmax[f] {
                    bmax[f] = sq >> 4;
                }
                bps.push(sq);
            }
        }
        let (mut mg, mut eg) = (0, 0);
        for f in 0..8 {
            if wf[f] > 1 {
                mg -= DOUBLED.0 * (wf[f] - 1);
                eg -= DOUBLED.1 * (wf[f] - 1);
            }
            if bf[f] > 1 {
                mg += DOUBLED.0 * (bf[f] - 1);
                eg += DOUBLED.1 * (bf[f] - 1);
            }
        }
        for sq in wps {
            let (f, r) = (sq & 7, sq >> 4);
            if (f == 0 || wf[(f - 1) as usize] == 0) && (f == 7 || wf[(f + 1) as usize] == 0) {
                mg -= ISOLATED.0;
                eg -= ISOLATED.1;
            }
            let passed = ![f - 1, f, f + 1].iter().any(|ff| (0..=7).contains(ff) && bmax[*ff as usize] > r);
            if passed {
                mg += PASSED_MG[r as usize];
                eg += PASSED_EG[r as usize];
            }
        }
        for sq in bps {
            let (f, r) = (sq & 7, sq >> 4);
            if (f == 0 || bf[(f - 1) as usize] == 0) && (f == 7 || bf[(f + 1) as usize] == 0) {
                mg += ISOLATED.0;
                eg += ISOLATED.1;
            }
            let passed = ![f - 1, f, f + 1].iter().any(|ff| (0..=7).contains(ff) && wmin[*ff as usize] < r);
            if passed {
                mg -= PASSED_MG[(7 - r) as usize];
                eg -= PASSED_EG[(7 - r) as usize];
            }
        }
        let entry = (mg, eg, wf, bf);
        PAWN_CACHE.with(|pc| {
            let mut pc = pc.borrow_mut();
            if pc.len() > 50000 {
                pc.clear();
            }
            pc.insert(self.phash, entry);
        });
        entry
    }

    // --------------------------------------------------------- Notation

    /// `fen()`
    pub fn fen(&self) -> String {
        let mut rows = Vec::new();
        for r in (0..8).rev() {
            let mut row = String::new();
            let mut empty = 0;
            for f in 0..8 {
                let p = self.b[(r * 16 + f) as usize];
                if p == 0 {
                    empty += 1;
                } else {
                    if empty != 0 {
                        row += &empty.to_string();
                        empty = 0;
                    }
                    row.push(piece_char(p));
                }
            }
            if empty != 0 {
                row += &empty.to_string();
            }
            rows.push(row);
        }
        let mut cr = String::new();
        for (i, ch) in "KQkq".chars().enumerate() {
            let rsq = self.castle[i];
            if rsq < 0 {
                continue;
            }
            let color = (i / 2) as i32;
            if self.outermost_rook(color, i % 2 == 0) == rsq {
                cr.push(ch);
            } else {
                let letter = FILES[(rsq & 7) as usize] as char;
                cr.push(if color == WHITE { letter.to_ascii_uppercase() } else { letter });
            }
        }
        format!(
            "{} {} {} {} {} {}",
            rows.join("/"),
            if self.side == WHITE { "w" } else { "b" },
            if cr.is_empty() { "-".to_string() } else { cr },
            if self.ep >= 0 { sq_name(self.ep) } else { "-".into() },
            self.half,
            self.full
        )
    }

    fn outermost_rook(&self, color: i32, kingside: bool) -> i32 {
        let base = if color == WHITE { 0 } else { 112 };
        let k = self.kings[color as usize];
        let rook = ROOK | (color << 3);
        let files: Vec<i32> = if kingside { ((k & 7) + 1..8).rev().collect() } else { (0..(k & 7)).collect() };
        for f in files {
            if self.b[(base + f) as usize] == rook {
                return base + f;
            }
        }
        -1
    }

    /// `uci(m)`
    pub fn uci(&self, m: i32) -> String {
        let frm = m & 255;
        let mut to = (m >> 8) & 255;
        if m >> 20 == M_CASTLE && !self.chess960 {
            to = (frm & 0x70) + if to > frm { 6 } else { 2 };
        }
        let promo = (m >> 16) & 7;
        let mut s = sq_name(frm) + &sq_name(to);
        if promo != 0 {
            s.push((LETTERS[promo as usize] as char).to_ascii_lowercase());
        }
        s
    }

    /// `parse_uci(text)` - Zug oder 0.
    pub fn parse_uci(&mut self, text: &str) -> i32 {
        let text = text.trim().to_lowercase();
        for m in self.legal_moves() {
            if self.uci(m) == text {
                return m;
            }
            if m >> 20 == M_CASTLE {
                let frm = m & 255;
                let to = (m >> 8) & 255;
                let kt = (frm & 0x70) + if to > frm { 6 } else { 2 };
                if text == sq_name(frm) + &sq_name(to) || text == sq_name(frm) + &sq_name(kt) {
                    return m;
                }
            }
        }
        0
    }

    /// `san(m, legal)` - Standard Algebraic Notation (inkl. +/#).
    pub fn san(&mut self, m: i32, legal: Option<&[i32]>) -> String {
        let frm = m & 255;
        let to = (m >> 8) & 255;
        let flag = m >> 20;
        let p = self.b[frm as usize];
        let typ = p & 7;
        let mut s;
        if flag == M_CASTLE {
            s = if to > frm { "O-O".to_string() } else { "O-O-O".to_string() };
        } else {
            let capture = flag == M_EP || self.b[to as usize] != 0;
            let promo = (m >> 16) & 7;
            if typ == PAWN {
                s = if capture { format!("{}x", FILES[(frm & 7) as usize] as char) } else { String::new() } + &sq_name(to);
                if promo != 0 {
                    s.push('=');
                    s.push(LETTERS[promo as usize] as char);
                }
            } else {
                let owned;
                let legal = match legal {
                    Some(l) => l,
                    None => {
                        owned = self.legal_moves();
                        &owned
                    }
                };
                let (mut same_file, mut same_rank, mut ambiguous) = (false, false, false);
                for &o in legal {
                    let of = o & 255;
                    if of == frm || (o >> 8) & 255 != to || o >> 20 == M_CASTLE {
                        continue;
                    }
                    if self.b[of as usize] != p {
                        continue;
                    }
                    ambiguous = true;
                    if (of & 7) == (frm & 7) {
                        same_file = true;
                    }
                    if (of >> 4) == (frm >> 4) {
                        same_rank = true;
                    }
                }
                let mut dis = String::new();
                if ambiguous {
                    if !same_file {
                        dis.push(FILES[(frm & 7) as usize] as char);
                    } else if !same_rank {
                        dis = ((frm >> 4) + 1).to_string();
                    } else {
                        dis = sq_name(frm);
                    }
                }
                s = format!("{}{}{}{}", LETTERS[typ as usize] as char, dis, if capture { "x" } else { "" }, sq_name(to));
            }
        }
        self.make(m);
        if self.in_check() {
            s += if self.has_legal_move() { "+" } else { "#" };
        }
        self.unmake();
        s
    }

    /// `parse_san(text)` - Zug oder 0.
    pub fn parse_san(&mut self, text: &str) -> i32 {
        let t = text.trim().replace("0-0-0", "O-O-O").replace("0-0", "O-O");
        let t = t.trim_end_matches(['+', '#', '!', '?']);
        let legal = self.legal_moves();
        for &m in &legal {
            if self.san(m, Some(&legal)).trim_end_matches(['+', '#']) == t {
                return m;
            }
        }
        0
    }
}

// ============================================================ Startstellungen

/// Grundstellung.
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KRN: [(usize, usize); 10] = [(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)];

/// `chess960_backrank(n)` - Grundreihe a..h.
pub fn chess960_backrank(n: i64) -> String {
    let mut n = py::pmod(n, 960);
    let mut rank = [' '; 8];
    let b1 = n % 4;
    n /= 4;
    rank[(b1 * 2 + 1) as usize] = 'B';
    let b2 = n % 4;
    n /= 4;
    rank[(b2 * 2) as usize] = 'B';
    let q = n % 6;
    n /= 6;
    let empty: Vec<usize> = (0..8).filter(|i| rank[*i] == ' ').collect();
    rank[empty[q as usize]] = 'Q';
    let empty: Vec<usize> = (0..8).filter(|i| rank[*i] == ' ').collect();
    let (k1, k2) = KRN[n as usize];
    rank[empty[k1]] = 'N';
    rank[empty[k2]] = 'N';
    let empty: Vec<usize> = (0..8).filter(|i| rank[*i] == ' ').collect();
    for (i, ch) in empty.iter().zip("RKR".chars()) {
        rank[*i] = ch;
    }
    rank.iter().collect()
}

/// `chess960_fen(n)`
pub fn chess960_fen(n: i64) -> String {
    let back = chess960_backrank(n);
    format!("{}/pppppppp/8/8/8/8/PPPPPPPP/{} w KQkq - 0 1", back.to_lowercase(), back)
}

/// `from_fen(fen, chess960)` - Fehlertext bei ungueltiger FEN.
pub fn from_fen(fen: &str, chess960: Option<bool>) -> Result<Position, &'static str> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    let mut pos = Position::default();
    let rows: Vec<&str> = parts.first().ok_or("FEN: leer")?.split('/').collect();
    if rows.len() != 8 {
        return Err("FEN: 8 Reihen erwartet");
    }
    for (i, row) in rows.iter().enumerate() {
        let r = 7 - i as i32;
        let mut f = 0;
        for ch in row.chars() {
            if let Some(d) = ch.to_digit(10) {
                f += d as i32;
            } else {
                let typ = LETTERS.iter().position(|c| *c as char == ch.to_ascii_uppercase()).map(|x| x as i32).unwrap_or(-1);
                if typ <= 0 || f > 7 {
                    return Err("FEN: ungueltige Figur");
                }
                pos.b[(r * 16 + f) as usize] = typ | if ch.is_lowercase() { 8 } else { 0 };
                f += 1;
            }
        }
        if f != 8 {
            return Err("FEN: Reihe hat nicht 8 Felder");
        }
    }
    pos.side = if parts.len() > 1 && parts[1] == "b" { BLACK } else { WHITE };
    for sq in squares() {
        let p = pos.b[sq as usize];
        if p & 7 == KING {
            pos.kings[(p >> 3) as usize] = sq;
        }
    }
    if pos.kings[0] < 0 || pos.kings[1] < 0 {
        return Err("FEN: Koenig fehlt");
    }
    let mut castle = [-1i32; 4];
    let mut std = true;
    let cpart = if parts.len() > 2 { parts[2] } else { "-" };
    for ch in cpart.chars() {
        if ch == '-' {
            continue;
        }
        let color = if ch.is_uppercase() { WHITE } else { BLACK };
        let base = if color == WHITE { 0 } else { 112 };
        let kf = pos.kings[color as usize] & 7;
        let up = ch.to_ascii_uppercase();
        if up == 'K' {
            let rsq = pos.outermost_rook(color, true);
            if rsq >= 0 {
                castle[(color * 2) as usize] = rsq;
            }
        } else if up == 'Q' {
            let rsq = pos.outermost_rook(color, false);
            if rsq >= 0 {
                castle[(color * 2 + 1) as usize] = rsq;
            }
        } else if let Some(f) = FILES.iter().position(|c| *c as char == ch.to_ascii_lowercase()) {
            let f = f as i32;
            if pos.b[(base + f) as usize] == ROOK | (color << 3) {
                castle[(color * 2 + if f > kf { 0 } else { 1 }) as usize] = base + f;
            }
            std = false;
        }
    }
    for (i, c) in castle.iter_mut().enumerate() {
        let color = (i / 2) as i32;
        let base = if color == WHITE { 0 } else { 112 };
        if *c >= 0 && (pos.kings[color as usize] >> 4) != (base >> 4) {
            *c = -1;
        }
        if *c >= 0 && (pos.kings[color as usize] != base + 4 || !(*c == base || *c == base + 7)) {
            std = false;
        }
    }
    pos.castle = castle;
    if parts.len() > 3 && parts[3] != "-" {
        let ep = sq_parse(parts[3]).ok_or("FEN: ep")?;
        let us = pos.side;
        let cap_pawn = PAWN | (us << 3);
        let ds: [i32; 2] = if us == WHITE { [-15, -17] } else { [15, 17] };
        let ok = ds.iter().any(|d| {
            let s = ep + d;
            !off(s) && pos.b[s as usize] == cap_pawn
        });
        pos.ep = if ok { ep } else { -1 };
    }
    pos.half = if parts.len() > 4 { parts[4].parse().map_err(|_| "FEN: half")? } else { 0 };
    pos.full = if parts.len() > 5 { parts[5].parse().map_err(|_| "FEN: full")? } else { 1 };
    pos.chess960 = chess960.unwrap_or(!std);
    pos.recompute();
    Ok(pos)
}

/// `perft(pos, depth)`
pub fn perft(pos: &mut Position, depth: i32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let us = pos.side;
    let mut n = 0;
    for m in pos.gen_moves(false) {
        pos.make(m);
        if !pos.attacked(pos.kings[us as usize], 1 - us) {
            n += if depth == 1 { 1 } else { perft(pos, depth - 1) };
        }
        pos.unmake();
    }
    n
}

// ================================================================== PGN

/// `pgn_result_code(winner)` - `Some(Some(c))` Sieger, `Some(None)` Remis, `None` laeuft.
pub fn pgn_result_code(winner: Option<Option<i32>>) -> &'static str {
    match winner {
        None => "*",
        Some(None) => "1/2-1/2",
        Some(Some(c)) => {
            if c == WHITE {
                "1-0"
            } else {
                "0-1"
            }
        }
    }
}

/// `pgn_text(tags, sans, result)`
pub fn pgn_text(tags: &[(String, String)], sans: &[String], result: &str, start_side: i32, start_full: i32) -> String {
    let lines: Vec<String> = tags.iter().map(|(k, v)| format!("[{} \"{}\"]", k, v.replace('\\', "\\\\").replace('"', "\\\""))).collect();
    let mut tokens = Vec::new();
    let mut full = start_full;
    let mut side = start_side;
    for (i, s) in sans.iter().enumerate() {
        if side == WHITE {
            tokens.push(format!("{full}."));
        } else if i == 0 {
            tokens.push(format!("{full}..."));
        }
        tokens.push(s.clone());
        if side == BLACK {
            full += 1;
        }
        side = 1 - side;
    }
    tokens.push(result.to_string());
    let mut body = Vec::new();
    let mut line = String::new();
    for tok in tokens {
        if !line.is_empty() && line.chars().count() + 1 + tok.chars().count() > 79 {
            body.push(std::mem::take(&mut line));
            line = tok;
        } else if line.is_empty() {
            line = tok;
        } else {
            line = line + " " + &tok;
        }
    }
    if !line.is_empty() {
        body.push(line);
    }
    lines.join("\n") + "\n\n" + &body.join("\n") + "\n"
}

// =========================================================== Eroeffnungsbuch

const BOOK_LINES: [&str; 40] = [
    "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6",
    "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 d2d4 e4d6 b5c6 d7c6 d4e5 d6f5",
    "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d3 d7d6 e1g1 e8g8",
    "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d3 f8e7 e1g1 e8g8 f1e1 d7d6",
    "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5 d8e7",
    "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4 d6d5 f1d3 b8c6",
    "e2e4 e7e5 b1c3 g8f6 g1f3 b8c6 f1b5 f8b4 e1g1 e8g8",
    "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3 e7e5 d4b3 c8e6",
    "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e5 d4b5 d7d6",
    "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6 f1d3 g8f6 e1g1 d8c7",
    "e2e4 c7c5 g1f3 d7d6 f1b5 c8d7 b5d7 d8d7 e1g1 b8c6 c2c3 g8f6",
    "e2e4 c7c5 b1c3 b8c6 g2g3 g7g6 f1g2 f8g7 d2d3 d7d6",
    "e2e4 c7c5 c2c3 g8f6 e4e5 f6d5 d2d4 c5d4 g1f3 b8c6",
    "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 c1g5 f8e7 e4e5 f6d7 g5e7 d8e7",
    "e2e4 e7e6 d2d4 d7d5 e4e5 c7c5 c2c3 b8c6 g1f3 d8b6 a2a3 c5c4",
    "e2e4 e7e6 d2d4 d7d5 b1d2 g8f6 e4e5 f6d7 f1d3 c7c5 c2c3 b8c6",
    "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6",
    "e2e4 c7c6 d2d4 d7d5 e4e5 c8f5 g1f3 e7e6 f1e2 c6c5 e1g1 b8c6",
    "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 g1f3 f8g7 f1e2 e8g8 e1g1 c7c6",
    "e2e4 d7d5 e4d5 d8d5 b1c3 d5a5 d2d4 g8f6 g1f3 c8f5 f1c4 e7e6",
    "e2e4 g8f6 e4e5 f6d5 d2d4 d7d6 g1f3 c8g4 f1e2 e7e6 e1g1 f8e7",
    "e2e4 g7g6 d2d4 f8g7 b1c3 d7d6 g1f3 g8f6 f1e2 e8g8 e1g1 c7c6",
    "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 f8e7 e2e3 e8g8 g1f3 b8d7",
    "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 d5c4 a2a4 c8f5 e2e3 e7e6",
    "d2d4 d7d5 c2c4 d5c4 g1f3 g8f6 e2e3 e7e6 f1c4 c7c5 e1g1 a7a6",
    "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 f8e7 f1g2 e8g8 e1g1 d5c4",
    "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5",
    "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3 e8g8 f1d3 d7d5 g1f3 c7c5",
    "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3 c8a6 b2b3 f8b4 c1d2 b4e7",
    "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5 e2e4 d5c3 b2c3 f8g7",
    "d2d4 g8f6 c2c4 c7c5 d4d5 e7e6 b1c3 e6d5 c4d5 d7d6 e2e4 g7g6",
    "d2d4 g8f6 c2c4 e7e6 g2g3 d7d5 f1g2 f8e7 g1f3 e8g8 e1g1 d5c4",
    "d2d4 d7d5 g1f3 g8f6 c1f4 e7e6 e2e3 c7c5 c2c3 b8c6 b1d2 f8d6",
    "d2d4 g8f6 g1f3 e7e6 c1g5 c7c5 e2e3 b7b6 b1d2 c8b7",
    "d2d4 f7f5 g2g3 g8f6 f1g2 g7g6 g1f3 f8g7 e1g1 e8g8 c2c4 d7d6",
    "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 g2g3 d7d5 c4d5 f6d5 f1g2 d5b6",
    "c2c4 g8f6 b1c3 e7e6 g1f3 d7d5 d2d4 f8e7 c1f4 e8g8 e2e3 c7c5",
    "c2c4 c7c5 g1f3 b8c6 b1c3 g7g6 g2g3 f8g7 f1g2 g8f6 e1g1 e8g8",
    "g1f3 d7d5 g2g3 g8f6 f1g2 e7e6 e1g1 f8e7 d2d3 e8g8 b1d2 c7c5",
    "g1f3 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 d2d4 e8g8 f1e2 e7e5",
];

/// `book_moves(history_uci)` - moegliche Buchzuege.
pub fn book_moves(history: &[String]) -> Vec<String> {
    let n = history.len();
    let mut out = Vec::new();
    for line in BOOK_LINES.iter() {
        let seq: Vec<&str> = line.split_whitespace().collect();
        if seq.len() > n && seq[..n].iter().zip(history.iter()).all(|(a, b)| *a == b) {
            out.push(seq[n].to_string());
        }
    }
    out
}

// ================================================================== KI

/// Eine KI-Stufe.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    /// Suchtiefe.
    pub depth: i32,
    /// Rechenzeit (s).
    pub time: f64,
    /// Knotenlimit.
    pub nodes: i64,
    /// Ruhesuche.
    pub quiesce: bool,
    /// Zufallszug-Wahrscheinlichkeit.
    pub random: f64,
    /// Streuung an der Wurzel (cp).
    pub spread: i32,
    /// Eroeffnungsbuch.
    pub book: bool,
}

/// `LEVELS` - Anfaenger .. Meister.
pub const LEVELS: [Level; 6] = [
    Level { depth: 1, time: 0.05, nodes: 2500, quiesce: false, random: 1.0, spread: 0, book: false },
    Level { depth: 1, time: 0.15, nodes: 2500, quiesce: false, random: 0.3, spread: 120, book: false },
    Level { depth: 2, time: 0.25, nodes: 6000, quiesce: true, random: 0.1, spread: 50, book: true },
    Level { depth: 3, time: 0.4, nodes: 15000, quiesce: true, random: 0.03, spread: 20, book: true },
    Level { depth: 5, time: 0.8, nodes: 40000, quiesce: true, random: 0.0, spread: 0, book: true },
    Level { depth: 32, time: 1.4, nodes: 120000, quiesce: true, random: 0.0, spread: 0, book: true },
];
/// `HINT_LEVEL`
pub const HINT_LEVEL: Level = Level { depth: 32, time: 0.9, nodes: 60000, quiesce: true, random: 0.0, spread: 0, book: false };
const TT_MAX: usize = 200000;

/// Transpositionstabelle: Schluessel -> (Tiefe, Art, Wert, Zug).
pub type Tt = HashMap<u64, (i32, i32, i32, i32)>;

/// Die Suche (Negamax mit Alpha-Beta, iterativer Vertiefung, TT, PVS,
/// Null-Zug, LMR, Killer/History, Ruhesuche).
pub struct Search {
    pos: Position,
    max_depth: i32,
    node_limit: i64,
    quiesce: bool,
    spread: i32,
    /// Transpositionstabelle (wird zwischen Suchen weitergereicht).
    pub tt: Tt,
    nodes: i64,
    stop: bool,
    /// Fertig?
    pub done: bool,
    /// Bester bisher gefundener Zug (0 = keiner).
    pub best_move: i32,
    /// Wert des besten Zugs.
    pub best_score: i32,
    /// Fertig durchsuchte Tiefe.
    pub depth_done: i32,
    root_scores: HashMap<i32, i32>,
    root_moves: Vec<i32>,
    killers: Vec<[i32; 2]>,
    history: Vec<i64>,
    /// Gleichwertige Wurzelzuege (Streuung) - die Wahl trifft
    /// [`Search::pick_spread`] mit dem globalen Zufall (wie Python am Ende
    /// von `run()`).
    pub good: Vec<i32>,
    /// Abbruch von aussen (anderer Thread).
    pub stop_flag: Arc<AtomicBool>,
}

impl Search {
    /// `Search(pos, level, tt)`
    pub fn new(pos: &Position, level: &Level, tt: Tt) -> Search {
        Search {
            pos: pos.copy(),
            max_depth: level.depth,
            node_limit: level.nodes,
            quiesce: level.quiesce,
            spread: level.spread,
            tt,
            nodes: 0,
            stop: false,
            done: false,
            best_move: 0,
            best_score: 0,
            depth_done: 0,
            root_scores: HashMap::new(),
            root_moves: Vec::new(),
            killers: vec![[0, 0]; (MAX_PLY + 2) as usize],
            history: vec![0; 16 * 128],
            good: Vec::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// `run()` am Stueck. Danach `pick_spread()` aufrufen.
    pub fn run(&mut self) {
        let mut moves = self.pos.legal_moves();
        if moves.is_empty() {
            self.done = true;
            return;
        }
        self.best_move = moves[0];
        if moves.len() == 1 && self.spread == 0 {
            self.done = true;
            return;
        }
        let scores: Vec<i64> = moves.iter().map(|m| self.order_score(*m, 0, 0)).collect();
        let mut idx: Vec<usize> = (0..moves.len()).collect();
        idx.sort_by(|a, b| scores[*b].cmp(&scores[*a]));
        self.root_moves = idx.iter().map(|i| moves[*i]).collect();
        moves.clear();
        for depth in 1..=self.max_depth {
            let score = self.root(depth);
            if self.stop {
                break;
            }
            self.depth_done = depth;
            self.best_score = score;
            if score.abs() >= MATE - MAX_PLY && depth >= 2 && self.spread == 0 {
                break;
            }
        }
        if self.spread != 0 && !self.root_scores.is_empty() {
            let best = *self.root_scores.values().max().unwrap_or(&0);
            self.good = self.root_moves.iter().copied().filter(|m| *self.root_scores.get(m).unwrap_or(&-INF) >= best - self.spread).collect();
        }
        self.done = true;
    }

    /// Streuung: `best_move = rng.choice(good)` (globaler Zufall).
    pub fn pick_spread(&mut self) {
        if !self.good.is_empty() {
            self.best_move = py::choice(&self.good);
            self.good.clear();
        }
    }

    fn root(&mut self, depth: i32) -> i32 {
        let exact = self.spread > 0;
        let mut alpha = -INF;
        let beta = INF;
        let mut best = -INF;
        let mut best_move = 0;
        let mut scored: Vec<(i32, i32)> = Vec::new();
        let rm = self.root_moves.clone();
        for (i, &m) in rm.iter().enumerate() {
            self.pos.make(m);
            let score;
            if i == 0 || exact {
                score = -self.negamax(depth - 1, -beta, if !exact { -alpha } else { INF }, 1, true);
            } else {
                let mut sc = -self.negamax(depth - 1, -alpha - 1, -alpha, 1, true);
                if sc > alpha && !self.stop {
                    sc = -self.negamax(depth - 1, -beta, -alpha, 1, true);
                }
                score = sc;
            }
            self.pos.unmake();
            if self.stop {
                break;
            }
            scored.push((m, score));
            if score > best {
                best = score;
                best_move = m;
                if !exact {
                    self.best_move = m;
                    self.best_score = score;
                }
                if score > alpha && !exact {
                    alpha = score;
                }
            }
        }
        if !self.stop || exact {
            for (m, s) in &scored {
                self.root_scores.insert(*m, *s);
            }
        }
        if best_move != 0 {
            let get = |m: i32| scored.iter().find(|e| e.0 == m).map(|e| e.1).unwrap_or(-INF - 1);
            let keys: Vec<i32> = self.root_moves.iter().map(|m| get(*m)).collect();
            let mut order: Vec<usize> = (0..self.root_moves.len()).collect();
            order.sort_by(|a, b| keys[*b].cmp(&keys[*a]));
            self.root_moves = order.iter().map(|i| self.root_moves[*i]).collect();
            if exact && !self.stop {
                self.best_move = best_move;
                self.best_score = best;
            }
        }
        best
    }

    fn order_score(&self, m: i32, tt_move: i32, ply: i32) -> i64 {
        if m == tt_move {
            return 4000000;
        }
        let b = &self.pos.b;
        let frm = m & 255;
        let to = (m >> 8) & 255;
        let flag = m >> 20;
        let promo = (m >> 16) & 7;
        if flag == M_EP {
            return 2000000 + 10 - 1;
        }
        if flag != M_CASTLE {
            let cap = b[to as usize];
            if cap != 0 {
                return 2000000 + (ORDER_VALUE[(cap & 7) as usize] * 10 - ORDER_VALUE[(b[frm as usize] & 7) as usize] + promo) as i64;
            }
        }
        if promo != 0 {
            return 1900000 + promo as i64;
        }
        let k = self.killers[ply as usize];
        if m == k[0] {
            return 1800000;
        }
        if m == k[1] {
            return 1700000;
        }
        self.history[(b[frm as usize] * 128 + to) as usize]
    }

    fn tick(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes >= self.node_limit {
            self.stop = true;
        } else if self.nodes & 63 == 0 && self.stop_flag.load(Ordering::Relaxed) {
            self.stop = true;
        }
        self.stop
    }

    fn negamax(&mut self, mut depth: i32, mut alpha: i32, mut beta: i32, ply: i32, can_null: bool) -> i32 {
        if self.tick() {
            return 0;
        }
        if self.pos.half >= 100 || self.pos.repetitions() >= 2 || self.pos.insufficient_material() {
            return 0;
        }
        if alpha < -MATE + ply {
            alpha = -MATE + ply;
        }
        if beta > MATE - ply - 1 {
            beta = MATE - ply - 1;
        }
        if alpha >= beta {
            return alpha;
        }
        let us = self.pos.side;
        let them = 1 - us;
        let in_check = self.pos.attacked(self.pos.kings[us as usize], them);
        if in_check {
            depth += 1;
        }
        if depth <= 0 || ply >= MAX_PLY {
            if self.quiesce && ply < MAX_PLY {
                return self.quiesce_search(alpha, beta, ply);
            }
            return self.pos.evaluate();
        }
        let pv = beta - alpha > 1;
        let key = self.pos.hash;
        let mut tt_move = 0;
        if let Some(&(e_depth, e_flag, e_score, e_move)) = self.tt.get(&key) {
            tt_move = e_move;
            if !pv && e_depth >= depth {
                let mut e_score = e_score;
                if e_score > MATE - MAX_PLY {
                    e_score -= ply;
                } else if e_score < -MATE + MAX_PLY {
                    e_score += ply;
                }
                if e_flag == 0 || (e_flag == 1 && e_score >= beta) || (e_flag == 2 && e_score <= alpha) {
                    return e_score;
                }
            }
        }
        let mut stat = 0;
        if !pv && !in_check {
            stat = self.pos.evaluate();
            if depth <= 3 && beta.abs() < MATE - MAX_PLY && stat - 90 * depth >= beta {
                return stat;
            }
            if can_null && depth >= 3 && stat >= beta && self.pos.non_pawn_material(us) > 0 {
                let r = if depth < 6 { 2 } else { 3 };
                self.pos.make_null();
                let score = -self.negamax(depth - 1 - r, -beta, -beta + 1, ply + 1, false);
                self.pos.unmake_null();
                if self.stop {
                    return 0;
                }
                if score >= beta {
                    return if score >= MATE - MAX_PLY { beta } else { score };
                }
            }
        }
        let moves = self.pos.gen_moves(false);
        let scores: Vec<i64> = moves.iter().map(|m| self.order_score(*m, tt_move, ply)).collect();
        let mut order: Vec<usize> = (0..moves.len()).collect();
        order.sort_by(|a, b| scores[*b].cmp(&scores[*a]));
        let mut best = -INF;
        let mut best_move = 0;
        let mut legal = 0;
        let orig_alpha = alpha;
        let futile = !pv && !in_check && depth <= 2 && alpha.abs() < MATE - MAX_PLY && stat + 120 * depth <= alpha;
        for i in order {
            let m = moves[i];
            let quiet = scores[i] < 1700000;
            if futile && quiet && legal > 0 {
                continue;
            }
            self.pos.make(m);
            if self.pos.attacked(self.pos.kings[us as usize], them) {
                self.pos.unmake();
                continue;
            }
            legal += 1;
            let score;
            if legal == 1 {
                score = -self.negamax(depth - 1, -beta, -alpha, ply + 1, true);
            } else {
                let mut red = 0;
                if depth >= 3 && quiet && legal > 3 && !in_check {
                    red = if legal < 10 || depth < 6 { 1 } else { 2 };
                }
                let mut sc = -self.negamax(depth - 1 - red, -alpha - 1, -alpha, ply + 1, true);
                if sc > alpha && !self.stop && (red != 0 || sc < beta) {
                    sc = -self.negamax(depth - 1, -beta, -alpha, ply + 1, true);
                }
                score = sc;
            }
            self.pos.unmake();
            if self.stop {
                return 0;
            }
            if score > best {
                best = score;
                best_move = m;
                if score > alpha {
                    alpha = score;
                    if score >= beta {
                        if quiet {
                            let k = &mut self.killers[ply as usize];
                            if k[0] != m {
                                k[1] = k[0];
                                k[0] = m;
                            }
                            let hi = (self.pos.b[(m & 255) as usize] * 128 + ((m >> 8) & 255)) as usize;
                            self.history[hi] += (depth * depth) as i64;
                            if self.history[hi] > 1000000 {
                                for v in self.history.iter_mut() {
                                    *v /= 2;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        if legal == 0 {
            return if in_check { -MATE + ply } else { 0 };
        }
        let flag = if best >= beta {
            1
        } else if best > orig_alpha {
            0
        } else {
            2
        };
        let mut store = best;
        if store > MATE - MAX_PLY {
            store += ply;
        } else if store < -MATE + MAX_PLY {
            store -= ply;
        }
        if self.tt.len() >= TT_MAX {
            self.tt.clear();
        }
        self.tt.insert(key, (depth, flag, store, best_move));
        best
    }

    fn quiesce_search(&mut self, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        if self.tick() {
            return 0;
        }
        let stand = self.pos.evaluate();
        if ply >= MAX_PLY {
            return stand;
        }
        if stand >= beta {
            return stand;
        }
        if stand > alpha {
            alpha = stand;
        }
        let us = self.pos.side;
        let them = 1 - us;
        let moves = self.pos.gen_moves(true);
        let scores: Vec<i64> = moves.iter().map(|m| self.order_score(*m, 0, ply)).collect();
        let mut order: Vec<usize> = (0..moves.len()).collect();
        order.sort_by(|a, b| scores[*b].cmp(&scores[*a]));
        for i in order {
            let m = moves[i];
            let to = (m >> 8) & 255;
            if (m >> 16) & 7 == 0 {
                let cap = self.pos.b[to as usize];
                let gain = if cap != 0 { MG_VALUE[(cap & 7) as usize] } else { MG_VALUE[PAWN as usize] };
                if stand + gain + 200 < alpha {
                    continue;
                }
            }
            self.pos.make(m);
            if self.pos.attacked(self.pos.kings[us as usize], them) {
                self.pos.unmake();
                continue;
            }
            let score = -self.quiesce_search(-beta, -alpha, ply + 1);
            self.pos.unmake();
            if self.stop {
                return 0;
            }
            if score >= beta {
                return score;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }
}

/// `pick_weak_move(pos)` - zufaelliger Zug, Schlagzuege bevorzugt (globaler Zufall).
pub fn pick_weak_move(pos: &mut Position) -> i32 {
    let moves = pos.legal_moves();
    if moves.is_empty() {
        return 0;
    }
    let caps: Vec<i32> = moves.iter().copied().filter(|m| pos.is_capture(*m)).collect();
    if !caps.is_empty() && py::random() < 0.6 {
        return py::choice(&caps);
    }
    py::choice(&moves)
}

/// `gives_mate(pos, m)`
pub fn gives_mate(pos: &mut Position, m: i32) -> bool {
    pos.make(m);
    let mate = pos.in_check() && !pos.has_legal_move();
    pos.unmake();
    mate
}

/// `forced_mate(pos, n)` - erzwungenes Matt in hoechstens n Zuegen?
pub fn forced_mate(pos: &mut Position, n: i32) -> bool {
    let us = pos.side;
    let mut moves: Vec<(i32, i32)> = Vec::new();
    for m in pos.gen_moves(false) {
        pos.make(m);
        if pos.attacked(pos.kings[us as usize], 1 - us) {
            pos.unmake();
            continue;
        }
        let check = pos.in_check();
        pos.unmake();
        if n == 1 && !check {
            continue;
        }
        moves.push((if check { 0 } else if pos.is_capture(m) { 1 } else { 2 }, m));
    }
    moves.sort_by_key(|x| x.0);
    for (_, m) in moves {
        pos.make(m);
        let replies = pos.legal_moves();
        if replies.is_empty() {
            let mate = pos.in_check();
            pos.unmake();
            if mate {
                return true;
            }
            continue;
        }
        if n == 1 {
            pos.unmake();
            continue;
        }
        let mut ok = true;
        for r in replies {
            pos.make(r);
            let sub = forced_mate(pos, n - 1);
            pos.unmake();
            if !sub {
                ok = false;
                break;
            }
        }
        pos.unmake();
        if ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_grundstellung() {
        let mut p = from_fen(START_FEN, Some(false)).unwrap();
        assert_eq!(perft(&mut p, 1), 20);
        assert_eq!(perft(&mut p, 2), 400);
        assert_eq!(perft(&mut p, 3), 8902);
        assert_eq!(perft(&mut p, 4), 197281);
    }

    #[test]
    fn perft_kiwipete() {
        let mut p = from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", None).unwrap();
        assert_eq!(perft(&mut p, 1), 48);
        assert_eq!(perft(&mut p, 2), 2039);
        assert_eq!(perft(&mut p, 3), 97862);
    }

    #[test]
    fn suche_im_thread_abbrechen() {
        let pos = from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", None).unwrap();
        let mut level = LEVELS[5];
        level.nodes = 50_000_000;
        let s = Search::new(&pos, &level, Tt::new());
        let stop = s.stop_flag.clone();
        let h = std::thread::spawn(move || {
            let mut s = s;
            s.run();
            s
        });
        std::thread::sleep(std::time::Duration::from_millis(60));
        stop.store(true, Ordering::Relaxed);
        let s = h.join().unwrap();
        assert!(s.done);
        assert_ne!(s.best_move, 0);
        assert!(s.depth_done >= 1);
    }

    #[test]
    fn fen_und_san() {
        let mut p = from_fen(START_FEN, Some(false)).unwrap();
        assert_eq!(p.fen(), START_FEN);
        let m = p.parse_san("Nf3");
        assert_ne!(m, 0);
        assert_eq!(p.uci(m), "g1f3");
        assert_eq!(chess960_backrank(518), "RNBQKBNR");
    }
}
