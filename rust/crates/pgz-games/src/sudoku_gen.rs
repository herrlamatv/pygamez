//! sudoku_gen.rs
//! =============
//! Deterministischer Sudoku-Generator + Loeser (uebertragen aus
//! `games/sudoku_gen.py`).
//!
//! Klassische Level: `random.Random(seed_for(D, N))` (CPython-Mersenne-
//! Twister, `py::Random`), Varianten und Tages-Sudoku: `seedrand` (mulberry32).
//! Killer-Level kommen vorab erzeugt aus `games/levels/sudoku-killer.json`.

use std::sync::OnceLock;

use pgz_core::seedrand;
use pgz_engine::py;
use serde_json::Value;

/// Level je Schwierigkeitsgrad.
pub const LEVELS: i64 = 100;
const CLUES: [usize; 4] = [42, 34, 29, 25];
const SEED_BASE: u64 = 987_654_321;
const ALL9: u32 = 0x1FF;

/// Varianten in Setup-Reihenfolge.
pub const VARIANTS: [&str; 4] = ["classic", "x", "killer", "mini"];
const DIG_RULES: [&str; 4] = ["singles", "singles", "unique", "hard"];
const DAILY_DIFF: [i64; 7] = [0, 1, 1, 2, 2, 3, 2];

fn variant_clues(variant: &str) -> [usize; 4] {
    match variant {
        "x" => [36, 29, 25, 21],
        "mini" => [20, 16, 13, 10],
        _ => [42, 34, 29, 25],
    }
}

struct Tables {
    row_of: [usize; 81],
    col_of: [usize; 81],
    box_of: [usize; 81],
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = Tables { row_of: [0; 81], col_of: [0; 81], box_of: [0; 81] };
        for i in 0..81 {
            t.row_of[i] = i / 9;
            t.col_of[i] = i % 9;
            t.box_of[i] = (i / 27) * 3 + (i % 9) / 3;
        }
        t
    })
}

/// `seed_for(diff, level)`
pub fn seed_for(diff: i64, level: i64) -> u64 {
    (SEED_BASE as i64 + diff * 1_000_000 + level) as u64
}

fn fill(rng: &mut py::Random) -> Vec<u8> {
    let t = tables();
    let mut board = vec![0u8; 81];
    let mut row = [ALL9; 9];
    let mut col = [ALL9; 9];
    let mut bx = [ALL9; 9];
    #[allow(clippy::too_many_arguments)]
    fn solve(i: usize, t: &Tables, board: &mut [u8], row: &mut [u32; 9], col: &mut [u32; 9], bx: &mut [u32; 9], rng: &mut py::Random) -> bool {
        if i == 81 {
            return true;
        }
        let (r, c, b) = (t.row_of[i], t.col_of[i], t.box_of[i]);
        let cand = row[r] & col[c] & bx[b];
        if cand == 0 {
            return false;
        }
        let mut digits: Vec<u8> = (1..=9u8).filter(|d| cand & (1 << (d - 1)) != 0).collect();
        rng.shuffle(&mut digits);
        for d in digits {
            let m = 1u32 << (d - 1);
            board[i] = d;
            row[r] ^= m;
            col[c] ^= m;
            bx[b] ^= m;
            if solve(i + 1, t, board, row, col, bx, rng) {
                return true;
            }
            board[i] = 0;
            row[r] |= m;
            col[c] |= m;
            bx[b] |= m;
        }
        false
    }
    solve(0, t, &mut board, &mut row, &mut col, &mut bx, rng);
    board
}

/// `_count_solutions(board, limit=2)` - klassisches 9x9 mit MRV.
fn count_solutions9(board: &[u8], limit: usize) -> usize {
    let t = tables();
    let mut board = board.to_vec();
    let mut row = [ALL9; 9];
    let mut col = [ALL9; 9];
    let mut bx = [ALL9; 9];
    let mut empty = Vec::new();
    for i in 0..81 {
        let d = board[i];
        if d != 0 {
            let m = 1u32 << (d - 1);
            let (r, c, b) = (t.row_of[i], t.col_of[i], t.box_of[i]);
            if row[r] & m == 0 || col[c] & m == 0 || bx[b] & m == 0 {
                return 0;
            }
            row[r] ^= m;
            col[c] ^= m;
            bx[b] ^= m;
        } else {
            empty.push(i);
        }
    }
    struct S<'a> {
        t: &'a Tables,
        board: Vec<u8>,
        row: [u32; 9],
        col: [u32; 9],
        bx: [u32; 9],
        empty: Vec<usize>,
        count: usize,
        limit: usize,
    }
    fn solve(s: &mut S) {
        let mut best_i: i64 = -1;
        let mut best_cand = 0u32;
        let mut best_n = 10u32;
        for k in 0..s.empty.len() {
            let i = s.empty[k];
            if s.board[i] != 0 {
                continue;
            }
            let cand = s.row[s.t.row_of[i]] & s.col[s.t.col_of[i]] & s.bx[s.t.box_of[i]];
            let n = cand.count_ones();
            if n == 0 {
                return;
            }
            if n < best_n {
                best_i = i as i64;
                best_cand = cand;
                best_n = n;
                if n == 1 {
                    break;
                }
            }
        }
        if best_i < 0 {
            s.count += 1;
            return;
        }
        let bi = best_i as usize;
        let (r, c, b) = (s.t.row_of[bi], s.t.col_of[bi], s.t.box_of[bi]);
        let mut cand = best_cand;
        while cand != 0 {
            let m = cand & cand.wrapping_neg();
            cand ^= m;
            s.board[bi] = (32 - m.leading_zeros()) as u8;
            s.row[r] ^= m;
            s.col[c] ^= m;
            s.bx[b] ^= m;
            solve(s);
            s.board[bi] = 0;
            s.row[r] |= m;
            s.col[c] |= m;
            s.bx[b] |= m;
            if s.count >= s.limit {
                return;
            }
        }
    }
    let mut s = S { t, board: std::mem::take(&mut board), row, col, bx, empty, count: 0, limit };
    solve(&mut s);
    s.count
}

fn dig(solution: &[u8], target: usize, rng: &mut py::Random) -> Vec<u8> {
    let mut puzzle = solution.to_vec();
    let mut order: Vec<usize> = (0..81).collect();
    rng.shuffle(&mut order);
    let mut clues = 81;
    for i in order {
        if clues <= target {
            break;
        }
        let saved = puzzle[i];
        puzzle[i] = 0;
        if count_solutions9(&puzzle, 2) == 1 {
            clues -= 1;
        } else {
            puzzle[i] = saved;
        }
    }
    puzzle
}

/// `generate(diff, level)` - (puzzle, solution) der klassischen Level.
pub fn generate(diff: i64, level: i64) -> (Vec<u8>, Vec<u8>) {
    let diff = diff.clamp(0, 3);
    let mut rng = py::Random::new(seed_for(diff, level));
    let solution = fill(&mut rng);
    let puzzle = dig(&solution, CLUES[diff as usize], &mut rng);
    (puzzle, solution)
}

// =========================================================================
//  Varianten
// =========================================================================

/// Geometrie eines Sudoku-Typs.
pub struct Layout {
    /// Kantenlaenge (9 oder 6).
    pub n: usize,
    /// Zellen (n*n).
    pub size: usize,
    /// Blockhoehe.
    pub box_h: usize,
    /// Blockbreite.
    pub box_w: usize,
    /// X-Sudoku?
    pub diagonals: bool,
    /// Bitmaske aller Ziffern.
    pub all: u32,
    /// Alle Einheiten.
    pub units: Vec<Vec<usize>>,
    /// Einheiten je Zelle.
    pub units_of: Vec<Vec<usize>>,
    /// Nachbarn je Zelle (sortiert).
    pub peers: Vec<Vec<usize>>,
}

impl Layout {
    fn new(n: usize, box_h: usize, box_w: usize, diagonals: bool) -> Layout {
        let size = n * n;
        let mut units: Vec<Vec<usize>> = (0..n).map(|r| (0..n).map(|c| r * n + c).collect()).collect();
        units.extend((0..n).map(|c| (0..n).map(|r| r * n + c).collect::<Vec<_>>()));
        for br in 0..n / box_h {
            for bc in 0..n / box_w {
                let mut u = Vec::new();
                for r in 0..box_h {
                    for c in 0..box_w {
                        u.push((br * box_h + r) * n + bc * box_w + c);
                    }
                }
                units.push(u);
            }
        }
        if diagonals {
            units.push((0..n).map(|i| i * n + i).collect());
            units.push((0..n).map(|i| i * n + (n - 1 - i)).collect());
        }
        let units_of: Vec<Vec<usize>> = (0..size).map(|i| (0..units.len()).filter(|&k| units[k].contains(&i)).collect()).collect();
        let peers: Vec<Vec<usize>> = (0..size)
            .map(|i| {
                let mut p: Vec<usize> = units_of[i].iter().flat_map(|&k| units[k].iter().copied()).filter(|&j| j != i).collect();
                p.sort_unstable();
                p.dedup();
                p
            })
            .collect();
        Layout { n, size, box_h, box_w, diagonals, all: (1u32 << n) - 1, units, units_of, peers }
    }

    /// Liegt Zelle i auf einer Diagonalen (nur X-Sudoku)?
    pub fn on_diagonal(&self, i: usize) -> bool {
        let (r, c) = (i / self.n, i % self.n);
        self.diagonals && (r == c || r + c == self.n - 1)
    }
}

/// `layout_for(variant)` (gecacht).
pub fn layout_for(variant: &str) -> &'static Layout {
    static MINI: OnceLock<Layout> = OnceLock::new();
    static X: OnceLock<Layout> = OnceLock::new();
    static CLASSIC: OnceLock<Layout> = OnceLock::new();
    match variant {
        "mini" => MINI.get_or_init(|| Layout::new(6, 2, 3, false)),
        "x" => X.get_or_init(|| Layout::new(9, 3, 3, true)),
        _ => CLASSIC.get_or_init(|| Layout::new(9, 3, 3, false)),
    }
}

fn fill_grid(lay: &Layout, rng: &mut seedrand::Rand) -> Vec<u8> {
    struct S<'a> {
        lay: &'a Layout,
        board: Vec<u8>,
        umask: Vec<u32>,
    }
    fn solve(s: &mut S, left: usize, rng: &mut seedrand::Rand) -> bool {
        if left == 0 {
            return true;
        }
        let all = s.lay.all;
        let (mut best_i, mut best_c, mut best_n): (i64, u32, u32) = (-1, 0, 99);
        for i in 0..s.lay.size {
            if s.board[i] != 0 {
                continue;
            }
            let mut c = all;
            for &u in &s.lay.units_of[i] {
                c &= s.umask[u];
            }
            let k = c.count_ones();
            if k < best_n {
                best_i = i as i64;
                best_c = c;
                best_n = k;
                if k <= 1 {
                    break;
                }
            }
        }
        if best_c == 0 {
            return false;
        }
        let bi = best_i as usize;
        let mut digits: Vec<u8> = (1..=s.lay.n as u8).filter(|d| (best_c >> (d - 1)) & 1 != 0).collect();
        rng.shuffle(&mut digits);
        for d in digits {
            let m = 1u32 << (d - 1);
            s.board[bi] = d;
            for k in 0..s.lay.units_of[bi].len() {
                let u = s.lay.units_of[bi][k];
                s.umask[u] ^= m;
            }
            if solve(s, left - 1, rng) {
                return true;
            }
            for k in 0..s.lay.units_of[bi].len() {
                let u = s.lay.units_of[bi][k];
                s.umask[u] ^= m;
            }
            s.board[bi] = 0;
        }
        false
    }
    let mut s = S { lay, board: vec![0; lay.size], umask: vec![lay.all; lay.units.len()] };
    solve(&mut s, lay.size, rng);
    s.board
}

/// `count_solutions(lay, puzzle, limit)` (ohne Kaefige).
pub fn count_solutions(lay: &Layout, puzzle: &[u8], limit: usize) -> usize {
    let mut board = puzzle.to_vec();
    let mut umask = vec![lay.all; lay.units.len()];
    for i in 0..lay.size {
        let d = board[i];
        if d != 0 {
            let m = 1u32 << (d - 1);
            for &u in &lay.units_of[i] {
                if umask[u] & m == 0 {
                    return 0;
                }
                umask[u] ^= m;
            }
        }
    }
    let empty: Vec<usize> = (0..lay.size).filter(|&i| board[i] == 0).collect();
    struct S<'a> {
        lay: &'a Layout,
        board: &'a mut Vec<u8>,
        umask: Vec<u32>,
        empty: Vec<usize>,
        count: usize,
        limit: usize,
    }
    fn solve(s: &mut S) {
        let (mut best_i, mut best_c, mut best_n): (i64, u32, u32) = (-1, 0, 99);
        for k in 0..s.empty.len() {
            let i = s.empty[k];
            if s.board[i] != 0 {
                continue;
            }
            let mut c = s.lay.all;
            for &u in &s.lay.units_of[i] {
                c &= s.umask[u];
            }
            if c == 0 {
                return;
            }
            let n = c.count_ones();
            if n < best_n {
                best_i = i as i64;
                best_c = c;
                best_n = n;
                if n == 1 {
                    break;
                }
            }
        }
        if best_i < 0 {
            s.count += 1;
            return;
        }
        let bi = best_i as usize;
        let mut c = best_c;
        while c != 0 {
            let m = c & c.wrapping_neg();
            c ^= m;
            s.board[bi] = (32 - m.leading_zeros()) as u8;
            for k in 0..s.lay.units_of[bi].len() {
                let u = s.lay.units_of[bi][k];
                s.umask[u] ^= m;
            }
            solve(s);
            s.board[bi] = 0;
            for k in 0..s.lay.units_of[bi].len() {
                let u = s.lay.units_of[bi][k];
                s.umask[u] ^= m;
            }
            if s.count >= s.limit {
                return;
            }
        }
    }
    let mut s = S { lay, board: &mut board, umask, empty, count: 0, limit };
    solve(&mut s);
    s.count
}

/// `solve_singles(lay, puzzle)` - Loesung oder `None`.
pub fn solve_singles(lay: &Layout, puzzle: &[u8]) -> Option<Vec<u8>> {
    let all = lay.all;
    let mut board = puzzle.to_vec();
    let mut cand = vec![0u32; lay.size];
    for i in 0..lay.size {
        let d = board[i];
        if d != 0 {
            if lay.peers[i].iter().any(|&j| board[j] == d) {
                return None;
            }
            continue;
        }
        let mut used = 0u32;
        for &j in &lay.peers[i] {
            if board[j] != 0 {
                used |= 1 << (board[j] - 1);
            }
        }
        cand[i] = all & !used;
    }
    let mut left = board.iter().filter(|v| **v == 0).count();
    while left > 0 {
        let mut placed = 0;
        for i in 0..lay.size {
            if board[i] != 0 {
                continue;
            }
            let c = cand[i];
            if c == 0 {
                return None;
            }
            if c & (c - 1) == 0 {
                board[i] = (32 - c.leading_zeros()) as u8;
                cand[i] = 0;
                for &j in &lay.peers[i] {
                    cand[j] &= !c;
                }
                placed += 1;
            }
        }
        for cells in &lay.units {
            let (mut once, mut twice, mut have) = (0u32, 0u32, 0u32);
            for &i in cells {
                if board[i] != 0 {
                    have |= 1 << (board[i] - 1);
                } else {
                    let c = cand[i];
                    twice |= once & c;
                    once |= c;
                }
            }
            if all & !have & !once != 0 {
                return None;
            }
            let mut single = once & !twice & !have;
            while single != 0 {
                let m = single & single.wrapping_neg();
                single ^= m;
                for &i in cells {
                    if board[i] == 0 && cand[i] & m != 0 {
                        board[i] = (32 - m.leading_zeros()) as u8;
                        cand[i] = 0;
                        for &j in &lay.peers[i] {
                            cand[j] &= !m;
                        }
                        placed += 1;
                        break;
                    }
                }
            }
        }
        if placed == 0 {
            return None;
        }
        left -= placed.min(left);
    }
    Some(board)
}

fn dig_variant(lay: &Layout, solution: &[u8], target: usize, rule: &str, rng: &mut seedrand::Rand) -> Vec<u8> {
    let mut puzzle = solution.to_vec();
    let mut order: Vec<usize> = (0..lay.size).collect();
    rng.shuffle(&mut order);
    let mut clues = lay.size;
    let mut easy = true;
    for i in order {
        if clues <= target {
            if rule != "hard" {
                break;
            }
            if easy {
                easy = solve_singles(lay, &puzzle).is_some();
            }
            if !easy {
                break;
            }
        }
        let saved = puzzle[i];
        puzzle[i] = 0;
        let ok = if rule == "singles" { solve_singles(lay, &puzzle).is_some() } else { count_solutions(lay, &puzzle, 2) == 1 };
        if ok {
            clues -= 1;
        } else {
            puzzle[i] = saved;
        }
    }
    puzzle
}

/// `variant_seed(variant, diff, level)`
pub fn variant_seed(variant: &str, diff: i64, level: i64) -> u32 {
    seedrand::seed_from(&["sudoku", variant, &diff.to_string(), &level.to_string()])
}

/// `generate_variant(variant, diff, level)` - X-Sudoku oder Mini 6x6.
pub fn generate_variant(variant: &str, diff: i64, level: i64) -> (Vec<u8>, Vec<u8>) {
    let lay = layout_for(variant);
    let clues = variant_clues(variant);
    let diff = diff.clamp(0, 3);
    let mut rng = seedrand::Rand::new(variant_seed(variant, diff, level));
    let solution = fill_grid(lay, &mut rng);
    let puzzle = dig_variant(lay, &solution, clues[diff as usize], DIG_RULES[diff as usize], &mut rng);
    (puzzle, solution)
}

/// Wochentag (Montag = 0) eines Datums `YYYY-MM-DD`.
pub fn weekday(date: &str) -> usize {
    let mut p = date.split('-');
    let y: i64 = p.next().and_then(|s| s.parse().ok()).unwrap_or(2026);
    let m: i64 = p.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let d: i64 = p.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    // 1970-01-01 war ein Donnerstag (= 3)
    (seedrand::days_from_civil(y, m, d) + 3).rem_euclid(7) as usize
}

/// `daily_diff(date)`
pub fn daily_diff(date: &str) -> i64 {
    DAILY_DIFF[weekday(date)]
}

/// `generate_daily(date)` - (puzzle, solution, diff).
pub fn generate_daily(date: &str) -> (Vec<u8>, Vec<u8>, i64) {
    let diff = daily_diff(date);
    let lay = layout_for("classic");
    let mut rng = seedrand::Rand::new(seedrand::daily_seed("sudoku", Some(date)));
    let solution = fill_grid(lay, &mut rng);
    let puzzle = dig_variant(lay, &solution, variant_clues("classic")[diff as usize], DIG_RULES[diff as usize], &mut rng);
    (puzzle, solution, diff)
}

// ----- Killer-Sudoku: vorab erzeugte Level

/// Ein Killer-Level: (puzzle, solution, cages).
pub type KillerLevel = (Vec<u8>, Vec<u8>, Vec<(i64, Vec<usize>)>);

fn parse_killer_level(raw: &Value) -> Option<KillerLevel> {
    let digits = |key: &str| -> Option<Vec<u8>> { raw.get(key)?.as_str()?.chars().map(|ch| ch.to_digit(10).map(|d| d as u8)).collect() };
    let sol = digits("s")?;
    let puz = digits("g")?;
    let mut cages = Vec::new();
    for c in raw.get("c")?.as_array()? {
        let arr = c.as_array()?;
        let total = arr.first()?.as_i64()?;
        let cells: Option<Vec<usize>> = arr.get(1)?.as_array()?.iter().map(|v| v.as_u64().map(|u| u as usize)).collect();
        cages.push((total, cells?));
    }
    if sol.len() != 81 || puz.len() != 81 || cages.is_empty() {
        return None;
    }
    let mut all: Vec<usize> = cages.iter().flat_map(|(_, c)| c.iter().copied()).collect();
    all.sort_unstable();
    if all != (0..81).collect::<Vec<_>>() {
        return None;
    }
    if puz.iter().zip(&sol).any(|(p, s)| *p != 0 && p != s) {
        return None;
    }
    if cages.iter().any(|(total, cells)| cells.iter().map(|&i| sol[i] as i64).sum::<i64>() != *total) {
        return None;
    }
    Some((puz, sol, cages))
}

fn load_killer() -> &'static [Vec<KillerLevel>; 4] {
    static K: OnceLock<[Vec<KillerLevel>; 4]> = OnceLock::new();
    K.get_or_init(|| {
        let mut out: [Vec<KillerLevel>; 4] = Default::default();
        let text = include_str!("../../../../games/levels/sudoku-killer.json");
        if let Ok(data) = serde_json::from_str::<Value>(text) {
            for (d, slot) in out.iter_mut().enumerate() {
                if let Some(list) = data.get("levels").and_then(|l| l.get(d.to_string())).and_then(|l| l.as_array()) {
                    for raw in list {
                        if let Some(lvl) = parse_killer_level(raw) {
                            slot.push(lvl);
                        }
                    }
                }
            }
        }
        out
    })
}

/// `killer_level(diff, level)` - Level 1..100 oder `None`.
pub fn killer_level(diff: i64, level: i64) -> Option<&'static KillerLevel> {
    let levels = &load_killer()[diff.clamp(0, 3) as usize];
    if level >= 1 && level as usize <= levels.len() {
        Some(&levels[level as usize - 1])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn klassisch_ist_eindeutig() {
        let (p, s) = generate(0, 1);
        assert_eq!(count_solutions9(&p, 2), 1);
        assert!(p.iter().zip(&s).all(|(a, b)| *a == 0 || a == b));
    }
}
