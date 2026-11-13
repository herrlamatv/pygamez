//! tetris_ai.rs
//! ============
//! Heuristik-KI fuer Tetris (El-Tetris-Gewichte) - uebertragen aus
//! `games/tetris_ai.py`.

use std::collections::HashSet;

use pgz_engine::py;

use crate::tetris_core::{self as core, Board};

const W_LAND: f64 = -4.500158825082766;
const W_ERODE: f64 = 3.4181268101392694;
const W_ROWT: f64 = -3.2178882868487753;
const W_COLT: f64 = -9.348695305445199;
const W_HOLES: f64 = -7.899265427351652;
const W_WELLS: f64 = -3.3855972247263626;
const SAFE_HEIGHT: i32 = 9;
const TETRIS_BONUS: f64 = 30.0;
const SMALL_CLEAR_MALUS: f64 = 14.0;
const WELL_COL_MALUS: f64 = 4.0;

/// Staerke: Denkzeit, Zeit je Eingabe, Fehlerquote, Auswahlbreite, Hold, Angriff.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    /// Denkzeit nach dem Erscheinen (s).
    pub think: f64,
    /// Zeit je Eingabe (s).
    pub step: f64,
    /// Fehlerquote.
    pub error: f64,
    /// Aus wie vielen der besten Lagen ein Fehler waehlt.
    pub top: f64,
    /// Hold erlaubt.
    pub hold: bool,
    /// Angriffsspiel.
    pub attack: bool,
}

/// `LEVELS`
pub const LEVELS: [Level; 3] = [
    Level { think: 0.55, step: 0.11, error: 0.22, top: 4.0, hold: false, attack: false },
    Level { think: 0.24, step: 0.06, error: 0.05, top: 3.0, hold: true, attack: true },
    Level { think: 0.07, step: 0.03, error: 0.0, top: 1.0, hold: true, attack: true },
];

const WALLS: u32 = 1 | (1 << (core::COLS + 1));
const ROW_PAIRS: u32 = (1 << (core::COLS + 1)) - 1;
const LEFT_WALL: u32 = 1;
const RIGHT_WALL: u32 = 1 << (core::COLS - 1);

/// `evaluate(rows, lines, eroded, land_h, attack)`
pub fn evaluate(rows: &[u32], lines: i64, eroded: i64, land_h: f64, attack: bool) -> f64 {
    let h = core::ROWS as usize;
    let full = core::FULL;
    let mut t = 0;
    while t < h && rows[t] == 0 {
        t += 1;
    }
    let mut rowt = 2 * t as i64;
    let (mut colt, mut holes, mut wells) = (0i64, 0i64, 0i64);
    let mut cover = 0u32;
    let mut prev = 0u32;
    let atk = attack && (h - t) as i32 <= SAFE_HEIGHT;
    let well_cols = if atk { full & !RIGHT_WALL } else { full };
    let wall_bit = if atk { RIGHT_WALL } else { 0 };
    let mut col9 = 0;
    for i in t..h {
        let r = rows[i];
        let w = ((r | wall_bit) << 1) | WALLS;
        rowt += ((w ^ (w >> 1)) & ROW_PAIRS).count_ones() as i64;
        colt += (r ^ prev).count_ones() as i64;
        holes += (cover & !r & full).count_ones() as i64;
        cover |= r;
        prev = r;
        let mut wm = !r & ((r << 1) | LEFT_WALL) & ((r >> 1) | RIGHT_WALL) & well_cols;
        while wm != 0 {
            let b = wm & wm.wrapping_neg();
            wm ^= b;
            let mut k = i;
            while k < h && rows[k] & b == 0 {
                wells += 1;
                k += 1;
            }
        }
        if r & RIGHT_WALL != 0 {
            col9 += 1;
        }
    }
    colt += (prev ^ full).count_ones() as i64;
    let mut score = W_LAND * land_h + W_ERODE * eroded as f64 + W_ROWT * rowt as f64 + W_COLT * colt as f64 + W_HOLES * holes as f64 + W_WELLS * wells as f64;
    if atk {
        if lines == 4 {
            score += TETRIS_BONUS;
        } else if lines != 0 {
            score -= SMALL_CLEAR_MALUS * (4 - lines) as f64 / 3.0;
        }
        score -= WELL_COL_MALUS * col9 as f64;
    }
    score
}

fn place(rows: &[u32], kind: char, rot: i32, x: i32, y: i32) -> (Vec<u32>, i64, i64) {
    let sh = core::shape(kind, rot);
    let mut new = rows.to_vec();
    for &(dy, m) in &sh.rows {
        new[(y + dy) as usize] |= if x >= 0 { m << x } else { m >> -x };
    }
    let (mut lines, mut eroded) = (0i64, 0i64);
    for &(dy, m) in &sh.rows {
        if new[(y + dy) as usize] == core::FULL {
            lines += 1;
            eroded += m.count_ones() as i64;
        }
    }
    if lines != 0 {
        let keep: Vec<u32> = new.iter().copied().filter(|r| *r != core::FULL).collect();
        let mut nn = vec![0u32; lines as usize];
        nn.extend(keep);
        new = nn;
    }
    (new, lines, lines * eroded)
}

fn tops(rows: &[u32]) -> Vec<i32> {
    let mut tops = vec![core::ROWS; core::COLS as usize];
    let mut seen = 0u32;
    for (i, &r) in rows.iter().enumerate() {
        let new = r & !seen;
        if new != 0 {
            for c in 0..core::COLS {
                if (new >> c) & 1 != 0 {
                    tops[c as usize] = i as i32;
                }
            }
            seen |= r;
            if seen == core::FULL {
                break;
            }
        }
    }
    tops
}

/// Eine Endlage: (Bewertung, Drehfolge, dx, Drehung, x, y).
pub type Cand = (f64, Vec<i32>, i32, i32, i32, i32);

/// `candidates(rows, kind, rot0, x0, y0, attack)`
pub fn candidates(rows: &[u32], kind: char, rot0: i32, x0: i32, y0: i32, attack: bool) -> Vec<Cand> {
    let mut out: Vec<Cand> = Vec::new();
    let tp = tops(rows);
    let seqs: Vec<Vec<i32>> = if kind == 'O' { vec![vec![]] } else { vec![vec![], vec![1], vec![1, 1], vec![-1]] };
    let mut seen: HashSet<Vec<(i32, i32)>> = HashSet::new();
    for seq in seqs {
        let (mut r, mut x, mut y) = (rot0, x0, y0);
        let mut ok = true;
        for &d in &seq {
            match core::try_rotate(rows, kind, r, x, y, d) {
                Some((nr, nx, ny, _k)) => {
                    r = nr;
                    x = nx;
                    y = ny;
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            continue;
        }
        let sh = core::shape(kind, r);
        let mut xl = x;
        while !core::collides(rows, kind, r, xl - 1, y) {
            xl -= 1;
        }
        let mut xr = x;
        while !core::collides(rows, kind, r, xr + 1, y) {
            xr += 1;
        }
        for tx in xl..=xr {
            let mut ly: Option<i32> = None;
            for &(cx, by) in &sh.bottom {
                let top = tp[(tx + cx) as usize];
                if top <= y + by {
                    ly = Some(core::drop_y(rows, kind, r, tx, y));
                    break;
                }
                let cand = top - 1 - by;
                ly = Some(match ly {
                    None => cand,
                    Some(l) => {
                        if cand < l {
                            cand
                        } else {
                            l
                        }
                    }
                });
            }
            let ly = ly.unwrap_or(y);
            let mut key: Vec<(i32, i32)> = sh.cells.iter().map(|&(cx, cy)| (tx + cx, ly + cy)).collect();
            key.sort_unstable();
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            let (new, lines, eroded) = place(rows, kind, r, tx, ly);
            let land_h = core::ROWS as f64 - (ly as f64 + (sh.miny + sh.maxy) as f64 / 2.0);
            let score = evaluate(&new, lines, eroded, land_h, attack);
            out.push((score, seq.clone(), tx - x, r, tx, ly));
        }
    }
    out.sort_by(|a, b| (-a.0).partial_cmp(&(-b.0)).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn spawn_pos(rows: &[u32], kind: char) -> Option<(i32, i32)> {
    let (x, mut y) = (3, core::BUFFER - 2);
    if core::collides(rows, kind, 0, x, y) {
        return None;
    }
    if !core::collides(rows, kind, 0, x, y + 1) {
        y += 1;
    }
    Some((x, y))
}

/// Eine Eingabe der KI.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    /// Halten.
    Hold,
    /// Rechts drehen.
    Cw,
    /// Links drehen.
    Ccw,
    /// Schieben.
    Left,
    /// Schieben.
    Right,
    /// Hard Drop.
    Drop,
}

/// `plan(board, level, rng, allow_hold)`
pub fn plan(board: &mut Board, level: usize, rng: Option<&mut py::Random>, allow_hold: bool) -> Vec<Act> {
    if !board.active {
        return Vec::new();
    }
    let cfg = LEVELS[level.min(2)];
    let attack = cfg.attack;
    let rows = board.rows.clone();
    let mut best = candidates(&rows, board.kind, board.rot, board.x, board.y, attack);
    let mut use_hold = false;
    if allow_hold && cfg.hold && !board.hold_used {
        let alt = match board.hold_kind {
            Some(k) => k,
            None => board.queue.peek(1)[0],
        };
        let pos = if alt != board.kind { spawn_pos(&rows, alt) } else { None };
        if let Some(pos) = pos {
            let alt_c = candidates(&rows, alt, 0, pos.0, pos.1, attack);
            if !alt_c.is_empty() && (best.is_empty() || alt_c[0].0 > best[0].0 + 1.0) {
                best = alt_c;
                use_hold = true;
            }
        }
    }
    if best.is_empty() {
        return vec![Act::Drop];
    }
    let mut pick = best[0].clone();
    if let Some(rng) = rng {
        if cfg.error > 0.0 && rng.random() < cfg.error {
            let i = ((best.len() - 1) as i64).min((rng.random() * cfg.top) as i64) as usize;
            pick = best[i].clone();
        }
    }
    let (_s, seq, dx, _r, _x, _y) = pick;
    let mut acts = if use_hold { vec![Act::Hold] } else { Vec::new() };
    acts.extend(seq.iter().map(|d| if *d > 0 { Act::Cw } else { Act::Ccw }));
    for _ in 0..dx.abs() {
        acts.push(if dx < 0 { Act::Left } else { Act::Right });
    }
    acts.push(Act::Drop);
    acts
}

/// `apply(board, act)`
pub fn apply(board: &mut Board, act: Act) -> bool {
    match act {
        Act::Hold => board.hold(),
        Act::Cw => board.rotate(1),
        Act::Ccw => board.rotate(-1),
        Act::Left => board.mv(-1),
        Act::Right => board.mv(1),
        Act::Drop => board.hard_drop().is_some(),
    }
}
