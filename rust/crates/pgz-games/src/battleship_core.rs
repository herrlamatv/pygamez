//! battleship_core.rs
//! ==================
//! Regeln und KI fuer Schiffe versenken - uebertragen aus
//! `games/battleship_core.py`.
//!
//! Brett 10x10, Flotte 5/4/3/3/2. Felder intern als Index 0..99, Lagen der
//! Schiffe als Bitmasken (100 Bit passen in ein `u128`). Die KI in drei
//! Staerken sieht nur, was auch ein Mensch weiss.

use std::collections::HashSet;
use std::sync::OnceLock;

use pgz_engine::py;

/// Kantenlaenge des Bretts.
pub const N: i32 = 10;
/// Schiffslaengen der Flotte.
pub const FLEET: [i32; 5] = [5, 4, 3, 3, 2];
/// Namen der Schiffe (i18n `bs.ship.<key>`).
pub const SHIP_KEYS: [&str; 5] = ["carrier", "battleship", "cruiser", "submarine", "destroyer"];

/// Feld unbeschossen.
pub const UNKNOWN: u8 = 0;
/// Wasser.
pub const MISS: u8 = 1;
/// Treffer.
pub const HIT: u8 = 2;

/// KI-Staerken.
pub const EASY: i64 = 0;
/// Mittel.
pub const MEDIUM: i64 = 1;
/// Schwer.
pub const HARD: i64 = 2;

/// `cells_of(r, c, size, horiz)`
pub fn cells_of(r: i32, c: i32, size: i32, horiz: bool) -> Vec<(i32, i32)> {
    if horiz {
        (0..size).map(|i| (r, c + i)).collect()
    } else {
        (0..size).map(|i| (r + i, c)).collect()
    }
}

/// `in_board(r, c)`
pub fn in_board(r: i32, c: i32) -> bool {
    (0..N).contains(&r) && (0..N).contains(&c)
}

fn neighbors8(r: i32, c: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for dr in [-1, 0, 1] {
        for dc in [-1, 0, 1] {
            if (dr != 0 || dc != 0) && in_board(r + dr, c + dc) {
                out.push((r + dr, c + dc));
            }
        }
    }
    out
}

/// Eine Lage: (Maske, Feld-Indizes, Randmaske).
pub type Placement = (u128, Vec<usize>, u128);

struct Tables {
    /// Lagen je Laenge 2..=5 (Index = Laenge).
    placements: Vec<Vec<Placement>>,
    halo8: Vec<u128>,
    diag: Vec<u128>,
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut placements = vec![Vec::new(); 6];
        let mut sizes: Vec<i32> = FLEET.to_vec();
        sizes.sort_unstable();
        sizes.dedup();
        for size in sizes {
            let mut lst = Vec::new();
            for horiz in [true, false] {
                let rows = if horiz { N } else { N - size + 1 };
                let cols = if horiz { N - size + 1 } else { N };
                for r in 0..rows {
                    for c in 0..cols {
                        let cells = cells_of(r, c, size, horiz);
                        let mut mask: u128 = 0;
                        let mut idx = Vec::new();
                        for &(rr, cc) in &cells {
                            mask |= 1u128 << (rr * N + cc);
                            idx.push((rr * N + cc) as usize);
                        }
                        let mut halo: u128 = 0;
                        for &(rr, cc) in &cells {
                            for (nr, nc) in neighbors8(rr, cc) {
                                halo |= 1u128 << (nr * N + nc);
                            }
                        }
                        halo &= !mask;
                        lst.push((mask, idx, halo));
                    }
                }
            }
            placements[size as usize] = lst;
        }
        let mut halo8 = vec![0u128; (N * N) as usize];
        let mut diag = vec![0u128; (N * N) as usize];
        for r in 0..N {
            for c in 0..N {
                for (nr, nc) in neighbors8(r, c) {
                    halo8[(r * N + c) as usize] |= 1u128 << (nr * N + nc);
                    if nr != r && nc != c {
                        diag[(r * N + c) as usize] |= 1u128 << (nr * N + nc);
                    }
                }
            }
        }
        Tables { placements, halo8, diag }
    })
}

/// Iterationsreihenfolge von `set(liste)` in CPython (`py::IntSet`):
/// `for i in hits` laeuft dort in Hash-Tabellen-Folge, nicht aufsteigend.
pub fn py_int_set_order(items: &[usize]) -> Vec<usize> {
    py::IntSet::from_iter_py(items.iter().map(|i| *i as i64)).iter().map(|i| i as usize).collect()
}

// ===================================================================== Brett

/// Ein Schiff der Flotte.
#[derive(Clone, Debug)]
pub struct Ship {
    /// Position in FLEET (bestimmt Name/Laenge).
    pub idx: usize,
    /// Laenge.
    pub size: i32,
    /// Erstes Feld: Zeile.
    pub r: i32,
    /// Erstes Feld: Spalte.
    pub c: i32,
    /// Waagerecht?
    pub horiz: bool,
    /// Getroffene Felder (Menge).
    pub hits: Vec<(i32, i32)>,
}

impl Ship {
    fn new(idx: usize, r: i32, c: i32, horiz: bool) -> Ship {
        Ship { idx, size: FLEET[idx], r, c, horiz, hits: Vec::new() }
    }

    /// `cells`
    pub fn cells(&self) -> Vec<(i32, i32)> {
        cells_of(self.r, self.c, self.size, self.horiz)
    }

    /// `sunk`
    pub fn sunk(&self) -> bool {
        self.hits.len() as i32 >= self.size
    }

    fn covers(&self, r: i32, c: i32) -> bool {
        if self.horiz {
            r == self.r && c >= self.c && c < self.c + self.size
        } else {
            c == self.c && r >= self.r && r < self.r + self.size
        }
    }
}

/// Ergebnis eines Schusses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shot {
    /// Wasser.
    Miss,
    /// Treffer.
    Hit,
    /// Versenkt.
    Sunk,
}

/// Ein Spielbrett: eigene Schiffe + alle Schuesse darauf.
#[derive(Clone, Debug)]
pub struct Sea {
    /// Die Schiffe (Reihenfolge wie in Python).
    pub ships: Vec<Ship>,
    /// Schuesse je Feld.
    pub shots: [[u8; 10]; 10],
}

impl Default for Sea {
    fn default() -> Self {
        Self::new()
    }
}

impl Sea {
    /// `Sea()`
    pub fn new() -> Sea {
        Sea { ships: Vec::new(), shots: [[UNKNOWN; 10]; 10] }
    }

    /// `ship_at(r, c)` - Index in `ships`.
    pub fn ship_at(&self, r: i32, c: i32) -> Option<usize> {
        self.ships.iter().position(|s| s.covers(r, c))
    }

    /// `ship_by_idx(idx)` - Index in `ships`.
    pub fn ship_by_idx(&self, idx: usize) -> Option<usize> {
        self.ships.iter().position(|s| s.idx == idx)
    }

    /// `can_place(idx, r, c, horiz, touch)`
    pub fn can_place(&self, idx: usize, r: i32, c: i32, horiz: bool, touch: bool) -> bool {
        let cells = cells_of(r, c, FLEET[idx], horiz);
        if !cells.iter().all(|&(rr, cc)| in_board(rr, cc)) {
            return false;
        }
        let mine: HashSet<(i32, i32)> = cells.into_iter().collect();
        for s in &self.ships {
            if s.idx == idx {
                continue;
            }
            for (rr, cc) in s.cells() {
                if mine.contains(&(rr, cc)) {
                    return false;
                }
                if !touch {
                    for p in neighbors8(rr, cc) {
                        if mine.contains(&p) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// `place(idx, r, c, horiz)` - ersetzt eine fruehere Lage.
    pub fn place(&mut self, idx: usize, r: i32, c: i32, horiz: bool) {
        if let Some(i) = self.ship_by_idx(idx) {
            self.ships.remove(i);
        }
        self.ships.push(Ship::new(idx, r, c, horiz));
    }

    /// `remove(ship)`
    pub fn remove_idx(&mut self, idx: usize) {
        if let Some(i) = self.ship_by_idx(idx) {
            self.ships.remove(i);
        }
    }

    /// `clear()`
    pub fn clear(&mut self) {
        self.ships.clear();
    }

    /// `complete()`
    pub fn complete(&self) -> bool {
        self.ships.len() == FLEET.len()
    }

    /// `randomize(touch)` mit Pythons globalem `random`.
    pub fn randomize(&mut self, touch: bool) -> bool {
        let mut fixed: Vec<Ship> = Vec::new();
        let mut order: Vec<usize> = (0..FLEET.len()).collect();
        order.sort_by_key(|i| -FLEET[*i]);
        for _attempt in 0..200 {
            self.ships = fixed.clone();
            let mut ok = true;
            for &idx in &order {
                if self.ship_by_idx(idx).is_some() {
                    continue;
                }
                let mut spots = Vec::new();
                for h in [true, false] {
                    for r in 0..N {
                        for c in 0..N {
                            if self.can_place(idx, r, c, h, touch) {
                                spots.push((r, c, h));
                            }
                        }
                    }
                }
                if spots.is_empty() {
                    ok = false;
                    break;
                }
                let (r, c, h) = spots[py::choice_index(spots.len())];
                self.place(idx, r, c, h);
            }
            if ok {
                return true;
            }
            fixed.clear();
        }
        false
    }

    /// `fire(r, c)` - (Ergebnis, Index des Schiffs in `ships`).
    pub fn fire(&mut self, r: i32, c: i32) -> (Option<Shot>, Option<usize>) {
        if !in_board(r, c) || self.shots[r as usize][c as usize] != UNKNOWN {
            return (None, None);
        }
        let Some(si) = self.ship_at(r, c) else {
            self.shots[r as usize][c as usize] = MISS;
            return (Some(Shot::Miss), None);
        };
        self.shots[r as usize][c as usize] = HIT;
        let ship = &mut self.ships[si];
        if !ship.hits.contains(&(r, c)) {
            ship.hits.push((r, c));
        }
        (Some(if ship.sunk() { Shot::Sunk } else { Shot::Hit }), Some(si))
    }

    /// `all_sunk()`
    pub fn all_sunk(&self) -> bool {
        !self.ships.is_empty() && self.ships.iter().all(|s| s.sunk())
    }

    /// `ships_left()`
    pub fn ships_left(&self) -> i64 {
        self.ships.iter().filter(|s| !s.sunk()).count() as i64
    }

    /// `untouched()`
    pub fn untouched(&self) -> i64 {
        self.shots.iter().flatten().filter(|v| **v == UNKNOWN).count() as i64
    }

    /// `knowledge()` - (grid, sunk, remaining).
    pub fn knowledge(&self) -> (Vec<u8>, Vec<Vec<usize>>, Vec<i32>) {
        let grid: Vec<u8> = (0..(N * N) as usize).map(|i| self.shots[i / 10][i % 10]).collect();
        let sunk = self.ships.iter().filter(|s| s.sunk()).map(|s| s.cells().iter().map(|&(r, c)| (r * N + c) as usize).collect()).collect();
        let remaining = self.ships.iter().filter(|s| !s.sunk()).map(|s| s.size).collect();
        (grid, sunk, remaining)
    }
}

// ======================================================================= KI

fn bits(mut mask: u128) -> Vec<usize> {
    let mut out = Vec::new();
    while mask != 0 {
        out.push(mask.trailing_zeros() as usize);
        mask &= mask - 1;
    }
    out
}

/// `ShotAI` - waehlt Schuesse nur ueber `Sea::knowledge()`.
pub struct ShotAI {
    level: i64,
    touch: bool,
    rng: py::Random,
    parity: i64,
}

impl ShotAI {
    /// `ShotAI(level, touch)` mit eigenem `random.Random()`.
    pub fn new(level: i64, touch: bool) -> ShotAI {
        let mut rng = py::Random::unseeded();
        let parity = rng.randint(0, 5);
        ShotAI { level, touch, rng, parity }
    }

    /// KI mit vorgegebenem Generator (ohne Zufall zu ziehen).
    pub(crate) fn with_rng(level: i64, touch: bool, rng: py::Random, parity: i64) -> ShotAI {
        ShotAI { level, touch, rng, parity }
    }

    /// `choose(sea)` - (Zeile, Spalte) oder `None`.
    pub fn choose(&mut self, sea: &Sea) -> Option<(i32, i32)> {
        let (grid, sunk, remaining) = sea.knowledge();
        self.choose_from(&grid, &sunk, &remaining)
    }

    /// `choose_from(grid, sunk, remaining)`
    pub fn choose_from(&mut self, grid: &[u8], sunk: &[Vec<usize>], remaining: &[i32]) -> Option<(i32, i32)> {
        let free: Vec<usize> = (0..(N * N) as usize).filter(|i| grid[*i] == UNKNOWN).collect();
        if free.is_empty() {
            return None;
        }
        let i = if self.level == HARD {
            self.hard(grid, sunk, remaining, &free)
        } else if self.level == MEDIUM {
            self.medium(grid, sunk, &free)
        } else {
            self.easy(grid, sunk, &free)
        };
        Some(((i / 10) as i32, (i % 10) as i32))
    }

    fn masks(grid: &[u8], sunk: &[Vec<usize>]) -> (u128, u128, u128) {
        let (mut miss, mut hit, mut sunk_m) = (0u128, 0u128, 0u128);
        for (i, v) in grid.iter().enumerate() {
            if *v == MISS {
                miss |= 1u128 << i;
            } else if *v == HIT {
                hit |= 1u128 << i;
            }
        }
        for cells in sunk {
            for i in cells {
                sunk_m |= 1u128 << i;
            }
        }
        (miss, hit & !sunk_m, sunk_m)
    }

    fn known_empty(&self, sunk_m: u128, open_hits: u128) -> u128 {
        if self.touch {
            return 0;
        }
        let t = tables();
        let mut empty = 0u128;
        for i in bits(sunk_m) {
            empty |= t.halo8[i];
        }
        for i in bits(open_hits) {
            empty |= t.diag[i];
        }
        empty
    }

    fn neighbors4(open_hits: u128, free_set: &HashSet<usize>) -> Vec<usize> {
        let mut out = Vec::new();
        for i in bits(open_hits) {
            let (r, c) = ((i / 10) as i32, (i % 10) as i32);
            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let (rr, cc) = (r + dr, c + dc);
                if in_board(rr, cc) && free_set.contains(&((rr * N + cc) as usize)) {
                    out.push((rr * N + cc) as usize);
                }
            }
        }
        out
    }

    fn easy(&mut self, grid: &[u8], sunk: &[Vec<usize>], free: &[usize]) -> usize {
        let (_miss, open_hits, _s) = Self::masks(grid, sunk);
        if open_hits != 0 && self.rng.random() < 0.35 {
            let fs: HashSet<usize> = free.iter().copied().collect();
            let near = Self::neighbors4(open_hits, &fs);
            if !near.is_empty() {
                return self.rng.choice(&near);
            }
        }
        self.rng.choice(free)
    }

    fn medium(&mut self, grid: &[u8], sunk: &[Vec<usize>], free: &[usize]) -> usize {
        let (_miss, open_hits, sunk_m) = Self::masks(grid, sunk);
        let empty = self.known_empty(sunk_m, open_hits);
        let mut free_set: HashSet<usize> = free.iter().copied().filter(|i| (empty >> i) & 1 == 0).collect();
        if free_set.is_empty() {
            free_set = free.iter().copied().collect();
        }
        if open_hits != 0 {
            let hit_list = bits(open_hits);
            let hits: HashSet<usize> = hit_list.iter().copied().collect();
            let is_hit = |r: i32, c: i32| (r * N + c) >= 0 && hits.contains(&((r * N + c) as usize));
            let mut ends = Vec::new();
            for i in py_int_set_order(&hit_list) {
                let (r, c) = ((i / 10) as i32, (i % 10) as i32);
                for (dr, dc) in [(0, 1), (1, 0)] {
                    if !is_hit(r + dr, c + dc) || !in_board(r + dr, c + dc) {
                        continue;
                    }
                    for sgn in [1, -1] {
                        let (mut rr, mut cc) = (r, c);
                        while in_board(rr, cc) && is_hit(rr, cc) {
                            rr += dr * sgn;
                            cc += dc * sgn;
                        }
                        if in_board(rr, cc) && free_set.contains(&((rr * N + cc) as usize)) {
                            ends.push((rr * N + cc) as usize);
                        }
                    }
                }
            }
            if !ends.is_empty() {
                return self.rng.choice(&ends);
            }
            let near = Self::neighbors4(open_hits, &free_set);
            if !near.is_empty() {
                return self.rng.choice(&near);
            }
        }
        let mut sorted: Vec<usize> = free_set.into_iter().collect();
        sorted.sort_unstable();
        self.rng.choice(&sorted)
    }

    /// `density(grid, sunk, remaining)` - Gewichte je Feld.
    pub fn density(&self, grid: &[u8], sunk: &[Vec<usize>], remaining: &[i32]) -> (Vec<i64>, bool) {
        let (miss, open_hits, sunk_m) = Self::masks(grid, sunk);
        let blocked = miss | sunk_m | self.known_empty(sunk_m, open_hits);
        let touching = open_hits | sunk_m;
        let mut dens = vec![0i64; (N * N) as usize];
        let target = open_hits != 0;
        let t = tables();
        for &size in remaining {
            for (mask, cells, halo) in &t.placements[size as usize] {
                if mask & blocked != 0 {
                    continue;
                }
                if !self.touch && halo & touching != 0 {
                    continue;
                }
                let w = if target {
                    let k = (mask & open_hits).count_ones();
                    if k == 0 {
                        continue;
                    }
                    8i64.pow(k)
                } else {
                    1
                };
                for i in cells {
                    dens[*i] += w;
                }
            }
        }
        (dens, target)
    }

    fn hard(&mut self, grid: &[u8], sunk: &[Vec<usize>], remaining: &[i32], free: &[usize]) -> usize {
        let (mut dens, mut target) = self.density(grid, sunk, remaining);
        let mut cand: Vec<usize> = free.iter().copied().filter(|i| dens[*i] > 0).collect();
        if cand.is_empty() && target {
            let g2: Vec<u8> = grid.iter().map(|v| if *v == HIT { MISS } else { *v }).collect();
            let (d, tg) = self.density(&g2, sunk, remaining);
            dens = d;
            target = tg;
            cand = free.iter().copied().filter(|i| dens[*i] > 0).collect();
        }
        if cand.is_empty() {
            return self.rng.choice(free);
        }
        if !target && !remaining.is_empty() {
            let step = *remaining.iter().min().unwrap_or(&1) as i64;
            let par: Vec<usize> = cand.iter().copied().filter(|i| ((i / 10) as i64 + (i % 10) as i64 + self.parity) % step == 0).collect();
            if !par.is_empty() {
                cand = par;
            }
        }
        let best = cand.iter().map(|i| dens[*i]).max().unwrap_or(0);
        let top: Vec<usize> = cand.into_iter().filter(|i| dens[*i] == best).collect();
        self.rng.choice(&top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reihenfolgen mit Python 3.14 erzeugt: `list(set(x))`.
    #[test]
    fn set_reihenfolge_wie_cpython() {
        assert_eq!(py_int_set_order(&[7, 9]), vec![9, 7]);
        assert_eq!(py_int_set_order(&[5, 13]), vec![13, 5]);
        assert_eq!(py_int_set_order(&[1, 2, 3]), vec![1, 2, 3]);
    }
}
