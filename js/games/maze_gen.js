/*
 * maze_gen.js - Seed-basierte Labyrinth-Erzeugung (Port von games/maze_gen.py)
 * =============================================================================
 * - levelParams(level): Größe/Orbs/Par-Zeit für Level 1-50.
 * - generate(cells, rng): Recursive Backtracker (iterativ) auf einem
 *   Zellenraster -> Tile-Grid (1 = Wand, 0 = frei), Kantenlänge 2*cells+1.
 * - farExit(grid): BFS von (1,1) -> fernstes begehbares Tile als Ausgang,
 *   plus Distanzkarte für die Orb-Platzierung.
 * - placeOrbs(grid, dist, exitPos, n, rng): Orbs im 30-90 %-Distanzband,
 *   paarweise mindestens 4 Tiles auseinander, nie auf Start/Ausgang.
 *
 * API: PG.mazeGen = { LEVELS, levelParams, generate, farExit, placeOrbs }
 * rng = PG.Random (Werte weichen von Python ab, die Logik ist identisch).
 */
(function () {
  "use strict";

  const LEVELS = 50;

  /** Parameter für Level 1-50: Zellen 5 -> 20, Orbs, Par-Zeit (Sekunden). */
  function levelParams(level) {
    level = Math.max(1, Math.min(LEVELS, Math.trunc(level)));
    const cells = 5 + Math.floor(((level - 1) * 15) / (LEVELS - 1)); // 5..20
    const orbs = 3 + Math.floor(cells / 3);
    const par = cells * cells * 1.1;
    return { cells, orbs, par };
  }

  /**
   * Perfektes Labyrinth (Recursive Backtracker, iterativ).
   * Rückgabe: grid[y][x] mit 1 = Wand, 0 = frei; Kantenlänge 2*cells+1.
   * Start der Erzeugung ist Zelle (0,0), also Tile (1,1).
   */
  function generate(cells, rng) {
    const n = 2 * cells + 1;
    const grid = Array.from({ length: n }, () => new Array(n).fill(1));
    const visited = Array.from({ length: cells }, () => new Array(cells).fill(false));
    const stack = [[0, 0]];
    visited[0][0] = true;
    grid[1][1] = 0;
    const DIRS = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    while (stack.length) {
      const [cx, cy] = stack[stack.length - 1];
      const neighbors = [];
      for (const [dx, dy] of DIRS) {
        const nx = cx + dx, ny = cy + dy;
        if (nx >= 0 && nx < cells && ny >= 0 && ny < cells && !visited[ny][nx]) neighbors.push([nx, ny]);
      }
      if (!neighbors.length) {
        stack.pop();
        continue;
      }
      const [nx, ny] = rng.choice(neighbors);
      visited[ny][nx] = true;
      // Wand zwischen den Zellen und die Zielzelle öffnen
      grid[cy * 2 + 1 + (ny - cy)][cx * 2 + 1 + (nx - cx)] = 0;
      grid[ny * 2 + 1][nx * 2 + 1] = 0;
      stack.push([nx, ny]);
    }
    return grid;
  }

  /**
   * BFS von (1,1): [Ausgangs-Tile mit maximaler Distanz, Distanzkarte].
   * Die Distanzkarte ist eine Map "x,y" -> {p: [x, y], d} in BFS-Reihenfolge.
   */
  function farExit(grid) {
    const n = grid.length;
    const dist = new Map();
    dist.set("1,1", { p: [1, 1], d: 0 });
    const q = [[1, 1]];
    let head = 0;
    let far = [1, 1], farD = 0;
    while (head < q.length) {
      const [x, y] = q[head++];
      const d = dist.get(x + "," + y).d;
      if (d > farD) {
        far = [x, y];
        farD = d;
      }
      for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const nx = x + dx, ny = y + dy;
        const key = nx + "," + ny;
        if (nx >= 0 && nx < n && ny >= 0 && ny < n && grid[ny][nx] === 0 && !dist.has(key)) {
          dist.set(key, { p: [nx, ny], d: d + 1 });
          q.push([nx, ny]);
        }
      }
    }
    return [far, dist];
  }

  /** Orbs auf freie Tiles im 30-90 %-Distanzband verteilen. */
  function placeOrbs(grid, dist, exitPos, nOrbs, rng) {
    let maxD = 0;
    for (const v of dist.values()) maxD = Math.max(maxD, v.d);
    maxD = maxD || 1;
    const lo = 0.3 * maxD, hi = 0.9 * maxD;
    const pool = [];
    for (const v of dist.values()) {
      const p = v.p;
      const isExit = p[0] === exitPos[0] && p[1] === exitPos[1];
      const isStart = p[0] === 1 && p[1] === 1;
      if (lo <= v.d && v.d <= hi && !isExit && !isStart) pool.push(p);
    }
    rng.shuffle(pool);
    const orbs = [];
    for (const p of pool) {
      if (orbs.length >= nOrbs) break;
      if (orbs.every((o) => Math.abs(p[0] - o[0]) + Math.abs(p[1] - o[1]) >= 4)) orbs.push(p);
    }
    // Falls das Abstandskriterium zu streng war: auffüllen
    for (const p of pool) {
      if (orbs.length >= nOrbs) break;
      if (!orbs.includes(p)) orbs.push(p);
    }
    return orbs;
  }

  PG.mazeGen = { LEVELS, levelParams, generate, farExit, placeOrbs };
})();
