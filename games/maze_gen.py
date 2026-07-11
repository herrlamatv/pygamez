# -*- coding: utf-8 -*-
"""
maze_gen.py
===========
Seed-basierte Labyrinth-Erzeugung für das Labyrinth-Spiel (pygame-frei).

- level_params(level): Größe/Orbs/Par-Zeit für Level 1-50.
- generate(cells, rng): Recursive Backtracker (iterativ) auf einem
  Zellenraster -> Tile-Grid (1 = Wand, 0 = frei), Kantenlänge 2*cells+1.
- far_exit(grid): BFS von (1,1) -> fernstes begehbares Tile als Ausgang,
  plus Distanzkarte für die Orb-Platzierung.
- place_orbs(grid, dist, exit_pos, n, rng): Orbs im 30-90 %-Distanzband,
  paarweise mindestens 4 Tiles auseinander, nie auf Start/Ausgang.
"""

from collections import deque

LEVELS = 50


def level_params(level):
    """Parameter für Level 1-50: Zellen 5 -> 20, Orbs, Par-Zeit (Sekunden)."""
    level = max(1, min(LEVELS, int(level)))
    cells = 5 + (level - 1) * 15 // (LEVELS - 1)      # 5..20
    orbs = 3 + cells // 3
    par = cells * cells * 1.1
    return dict(cells=cells, orbs=orbs, par=par)


def generate(cells, rng):
    """Perfektes Labyrinth (Recursive Backtracker, iterativ).

    Rückgabe: Liste von Listen, grid[y][x] mit 1 = Wand, 0 = frei;
    Kantenlänge 2*cells+1. Start der Erzeugung ist Zelle (0,0), also
    Tile (1,1).
    """
    n = 2 * cells + 1
    grid = [[1] * n for _ in range(n)]
    visited = [[False] * cells for _ in range(cells)]
    stack = [(0, 0)]
    visited[0][0] = True
    grid[1][1] = 0
    while stack:
        cx, cy = stack[-1]
        neighbors = []
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = cx + dx, cy + dy
            if 0 <= nx < cells and 0 <= ny < cells and not visited[ny][nx]:
                neighbors.append((nx, ny))
        if not neighbors:
            stack.pop()
            continue
        nx, ny = rng.choice(neighbors)
        visited[ny][nx] = True
        # Wand zwischen den Zellen und die Zielzelle öffnen
        grid[cy * 2 + 1 + (ny - cy)][cx * 2 + 1 + (nx - cx)] = 0
        grid[ny * 2 + 1][nx * 2 + 1] = 0
        stack.append((nx, ny))
    return grid


def far_exit(grid):
    """BFS von (1,1): (Ausgangs-Tile mit maximaler Distanz, Distanzkarte)."""
    n = len(grid)
    dist = {(1, 1): 0}
    q = deque([(1, 1)])
    far, far_d = (1, 1), 0
    while q:
        x, y = q.popleft()
        d = dist[(x, y)]
        if d > far_d:
            far, far_d = (x, y), d
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < n and 0 <= ny < n and grid[ny][nx] == 0 \
                    and (nx, ny) not in dist:
                dist[(nx, ny)] = d + 1
                q.append((nx, ny))
    return far, dist


def place_orbs(grid, dist, exit_pos, n_orbs, rng):
    """Orbs auf freie Tiles im 30-90 %-Distanzband verteilen."""
    max_d = max(dist.values()) or 1
    lo, hi = 0.3 * max_d, 0.9 * max_d
    pool = [p for p, d in dist.items()
            if lo <= d <= hi and p != exit_pos and p != (1, 1)]
    rng.shuffle(pool)
    orbs = []
    for p in pool:
        if len(orbs) >= n_orbs:
            break
        if all(abs(p[0] - o[0]) + abs(p[1] - o[1]) >= 4 for o in orbs):
            orbs.append(p)
    # Falls das Abstandskriterium zu streng war: auffüllen
    for p in pool:
        if len(orbs) >= n_orbs:
            break
        if p not in orbs:
            orbs.append(p)
    return orbs
