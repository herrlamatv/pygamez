//! maze_gen.rs
//! ===========
//! Seed-basierte Labyrinth-Erzeugung (aus `games/maze_gen.py`).
//!
//! - `level_params(level)`: Groesse/Orbs/Par-Zeit fuer Level 1-50.
//! - `generate(cells, rng)`: Recursive Backtracker -> Tile-Grid (1 = Wand).
//! - `far_exit(grid)`: BFS von (1,1) -> fernstes Tile + Distanzkarte in
//!   Einfuegereihenfolge (wie das Python-Dict).
//! - `place_orbs(...)`: Orbs im 30-90 %-Distanzband.

use pgz_engine::py::Random;

/// Anzahl Level.
pub const LEVELS: i32 = 50;

/// `level_params(level)` -> (cells, orbs, par)
pub fn level_params(level: i32) -> (usize, usize, f64) {
    let level = level.clamp(1, LEVELS);
    let cells = 5 + (level - 1) * 15 / (LEVELS - 1);
    let orbs = 3 + cells / 3;
    let par = (cells * cells) as f64 * 1.1;
    (cells as usize, orbs as usize, par)
}

/// `generate(cells, rng)` - `grid[y][x]`, 1 = Wand, 0 = frei.
pub fn generate(cells: usize, rng: &mut Random) -> Vec<Vec<u8>> {
    let n = 2 * cells + 1;
    let mut grid = vec![vec![1u8; n]; n];
    let mut visited = vec![vec![false; cells]; cells];
    let mut stack: Vec<(i32, i32)> = vec![(0, 0)];
    visited[0][0] = true;
    grid[1][1] = 0;
    let c = cells as i32;
    while let Some(&(cx, cy)) = stack.last() {
        let mut neighbors = Vec::with_capacity(4);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (cx + dx, cy + dy);
            if 0 <= nx && nx < c && 0 <= ny && ny < c && !visited[ny as usize][nx as usize] {
                neighbors.push((nx, ny));
            }
        }
        if neighbors.is_empty() {
            stack.pop();
            continue;
        }
        let (nx, ny) = rng.choice(&neighbors);
        visited[ny as usize][nx as usize] = true;
        grid[(cy * 2 + 1 + (ny - cy)) as usize][(cx * 2 + 1 + (nx - cx)) as usize] = 0;
        grid[(ny * 2 + 1) as usize][(nx * 2 + 1) as usize] = 0;
        stack.push((nx, ny));
    }
    grid
}

/// Distanzkarte in Einfuegereihenfolge (`dict` der Python-Fassung).
pub struct DistMap {
    /// `(tile, distanz)` in der Reihenfolge, in der die BFS sie fand.
    pub order: Vec<((i32, i32), i32)>,
}

/// `far_exit(grid)` -> (Ausgang, Distanzkarte)
pub fn far_exit(grid: &[Vec<u8>]) -> ((i32, i32), DistMap) {
    let n = grid.len() as i32;
    let mut seen = vec![vec![-1i32; n as usize]; n as usize];
    let mut order = vec![((1, 1), 0)];
    seen[1][1] = 0;
    let mut q = std::collections::VecDeque::from([(1i32, 1i32)]);
    let (mut far, mut far_d) = ((1, 1), 0);
    while let Some((x, y)) = q.pop_front() {
        let d = seen[y as usize][x as usize];
        if d > far_d {
            far = (x, y);
            far_d = d;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if 0 <= nx && nx < n && 0 <= ny && ny < n && grid[ny as usize][nx as usize] == 0 && seen[ny as usize][nx as usize] < 0 {
                seen[ny as usize][nx as usize] = d + 1;
                order.push(((nx, ny), d + 1));
                q.push_back((nx, ny));
            }
        }
    }
    (far, DistMap { order })
}

/// `place_orbs(grid, dist, exit_pos, n_orbs, rng)`
pub fn place_orbs(dist: &DistMap, exit_pos: (i32, i32), n_orbs: usize, rng: &mut Random) -> Vec<(i32, i32)> {
    let mut max_d = dist.order.iter().map(|e| e.1).max().unwrap_or(0);
    if max_d == 0 {
        max_d = 1;
    }
    let (lo, hi) = (0.3 * max_d as f64, 0.9 * max_d as f64);
    let mut pool: Vec<(i32, i32)> = dist
        .order
        .iter()
        .filter(|(p, d)| lo <= *d as f64 && *d as f64 <= hi && *p != exit_pos && *p != (1, 1))
        .map(|e| e.0)
        .collect();
    rng.shuffle(&mut pool);
    let mut orbs: Vec<(i32, i32)> = Vec::new();
    for &p in &pool {
        if orbs.len() >= n_orbs {
            break;
        }
        if orbs.iter().all(|o| (p.0 - o.0).abs() + (p.1 - o.1).abs() >= 4) {
            orbs.push(p);
        }
    }
    for &p in &pool {
        if orbs.len() >= n_orbs {
            break;
        }
        if !orbs.contains(&p) {
            orbs.push(p);
        }
    }
    orbs
}
