/*
 * tetris_core.js - Regelwerk + KI von Tetris (1:1-Port von games/tetris_core.py
 * und games/tetris_ai.py)
 * ==============================================================================
 * Ohne Grafik und ohne DOM - läuft auch in Node (web/tools/tetris_replay.js
 * vergleicht es mit der Python-Fassung Zug für Zug).
 *
 * - Feld 10 x 20 plus 20 verborgene Pufferzeilen (Zeile 0 = oberste Pufferzeile),
 *   jede Zeile als 10-Bit-Maske, Farben parallel in `cells`.
 * - 7-Bag über PG.seedrand -> dieselbe Steinfolge wie am PC.
 * - SRS mit echten Kicks, Lock Delay 0,5 s / 15 Resets, T-Spin (voll/Mini),
 *   Back-to-Back, Combo, Perfect Clear, Guideline-Gravitation und Angriff.
 * - KI: El-Tetris-Gewichte, drei Stärken, auf Mittel/Schwer Tetris-Aufbau.
 *
 *   const b = new PG.tetrisCore.Board(seed);
 *   b.move(-1); b.rotate(1); b.hardDrop(); b.tick(dt, soft);
 *   PG.tetrisAI.plan(b, level, rng)  -> ["hold", "cw", "left", ..., "drop"]
 */
(function () {
  "use strict";

  const PG = window.PG;

  const COLS = 10;
  const VISIBLE = 20;
  const BUFFER = 20;
  const ROWS = VISIBLE + BUFFER;
  const FULL = (1 << COLS) - 1;
  const KINDS = "IJLOSTZ";
  const SPAWN_X = 3;
  const SPAWN_Y = BUFFER - 2;

  const LOCK_DELAY = 0.5;
  const MAX_RESETS = 15;
  const SOFT_FACTOR = 20;
  const SOFT_MIN_RATE = 20.0;
  const GARBAGE_DELAY = 0.5;
  const GARBAGE_CAP = 8;

  const BASE = {
    I: [[0, 1], [1, 1], [2, 1], [3, 1]],
    J: [[0, 0], [0, 1], [1, 1], [2, 1]],
    L: [[2, 0], [0, 1], [1, 1], [2, 1]],
    O: [[1, 0], [2, 0], [1, 1], [2, 1]],
    S: [[1, 0], [2, 0], [0, 1], [1, 1]],
    T: [[1, 0], [0, 1], [1, 1], [2, 1]],
    Z: [[0, 0], [1, 0], [1, 1], [2, 1]],
  };

  /** Vorberechnete Daten eines Steins in einem Drehzustand (wie Shape in Python). */
  function makeShape(cells) {
    const sorted = cells.slice().sort((a, b) => a[1] - b[1] || a[0] - b[0]);
    const xs = cells.map((c) => c[0]);
    const ys = cells.map((c) => c[1]);
    const masks = new Map();
    for (const [cx, cy] of cells) masks.set(cy, (masks.get(cy) || 0) | (1 << cx));
    const rows = [...masks.entries()].sort((a, b) => a[0] - b[0]);
    const bottom = new Map();
    for (const [cx, cy] of cells) bottom.set(cx, Math.max(bottom.has(cx) ? bottom.get(cx) : -1, cy));
    return {
      cells: sorted,
      minx: Math.min(...xs), maxx: Math.max(...xs),
      miny: Math.min(...ys), maxy: Math.max(...ys),
      rows,
      bottom: [...bottom.entries()].sort((a, b) => a[0] - b[0]),
    };
  }

  const SHAPES = {};
  for (const kind of Object.keys(BASE)) {
    if (kind === "O") {
      const s = makeShape(BASE.O);
      SHAPES.O = [s, s, s, s];
      continue;
    }
    const n = kind === "I" ? 4 : 3;
    const states = [BASE[kind]];
    for (let i = 0; i < 3; i++) states.push(states[states.length - 1].map(([x, y]) => [n - 1 - y, x]));
    SHAPES[kind] = states.map(makeShape);
  }

  // SRS-Kicks (dy nach OBEN positiv, wie im Tetris-Wiki); Schlüssel "von>nach".
  const KICKS_JLSTZ = {
    "0>1": [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    "1>0": [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    "1>2": [[0, 0], [1, 0], [1, -1], [0, 2], [1, 2]],
    "2>1": [[0, 0], [-1, 0], [-1, 1], [0, -2], [-1, -2]],
    "2>3": [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
    "3>2": [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    "3>0": [[0, 0], [-1, 0], [-1, -1], [0, 2], [-1, 2]],
    "0>3": [[0, 0], [1, 0], [1, 1], [0, -2], [1, -2]],
  };
  const KICKS_I = {
    "0>1": [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    "1>0": [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    "1>2": [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
    "2>1": [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    "2>3": [[0, 0], [2, 0], [-1, 0], [2, 1], [-1, -2]],
    "3>2": [[0, 0], [-2, 0], [1, 0], [-2, -1], [1, 2]],
    "3>0": [[0, 0], [1, 0], [-2, 0], [1, -2], [-2, 1]],
    "0>3": [[0, 0], [-1, 0], [2, 0], [-1, 2], [2, -1]],
  };
  const T_FRONT = [[0, 1], [1, 2], [2, 3], [3, 0]];

  const SCORE_TABLE = {
    "|0": 0, "|1": 100, "|2": 300, "|3": 500, "|4": 800,
    "mini|0": 100, "mini|1": 200, "mini|2": 400,
    "full|0": 400, "full|1": 800, "full|2": 1200, "full|3": 1600,
  };
  const PC_POINTS = { 1: 800, 2: 1200, 3: 1800, 4: 2000 };
  const PC_POINTS_B2B_TETRIS = 3200;
  const COMBO_POINTS = 50;
  const ATTACK_TABLE = {
    "|0": 0, "|1": 0, "|2": 1, "|3": 2, "|4": 4,
    "mini|0": 0, "mini|1": 0, "mini|2": 1,
    "full|0": 0, "full|1": 2, "full|2": 4, "full|3": 6,
  };
  const COMBO_ATTACK = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5];
  const B2B_ATTACK = 1;
  const PC_ATTACK = 10;

  const POP = new Uint8Array(4096);
  for (let i = 1; i < 4096; i++) POP[i] = POP[i >> 1] + (i & 1);
  const popcount = (v) => POP[v & 4095];

  function gravityInterval(level) {
    const lv = Math.max(1, Math.min(Math.trunc(level), 20));
    const base = 0.8 - (lv - 1) * 0.007;
    let value = 1.0;
    for (let i = 0; i < lv - 1; i++) value *= base;
    return value;
  }

  function softInterval(level) {
    const g = gravityInterval(level);
    return Math.min(g / SOFT_FACTOR, 1.0 / SOFT_MIN_RATE);
  }

  function collides(rows, kind, rot, x, y) {
    const sh = SHAPES[kind][rot];
    if (x + sh.minx < 0 || x + sh.maxx >= COLS || y + sh.maxy >= ROWS || y + sh.miny < 0) return true;
    for (const [dy, m] of sh.rows) {
      if (rows[y + dy] & (x >= 0 ? m << x : m >> -x)) return true;
    }
    return false;
  }

  function tryRotate(rows, kind, rot, x, y, d) {
    const nrot = (((rot + d) % 4) + 4) % 4;
    if (kind === "O") return [nrot, x, y, 0];
    const table = kind === "I" ? KICKS_I : KICKS_JLSTZ;
    const kicks = table[rot + ">" + nrot];
    for (let i = 0; i < kicks.length; i++) {
      const nx = x + kicks[i][0];
      const ny = y - kicks[i][1];
      if (!collides(rows, kind, nrot, nx, ny)) return [nrot, nx, ny, i];
    }
    return null;
  }

  function dropY(rows, kind, rot, x, y) {
    while (!collides(rows, kind, rot, x, y + 1)) y++;
    return y;
  }

  class PieceQueue {
    constructor(seed) {
      this.rng = new PG.seedrand.Rand(seed);
      this.items = [];
    }
    fill(n) {
      while (this.items.length < n) {
        const bag = KINDS.split("");
        this.rng.shuffle(bag);
        this.items.push(...bag);
      }
    }
    pop() {
      this.fill(8);
      return this.items.shift();
    }
    peek(n) {
      this.fill(Math.max(n, 8));
      return this.items.slice(0, n);
    }
  }

  class Board {
    constructor(seed, level = 1, fixedLevel = false, garbageSeed = null) {
      this.rows = new Array(ROWS).fill(0);
      this.cells = Array.from({ length: ROWS }, () => new Array(COLS).fill(null));
      this.queue = new PieceQueue(seed);
      this.grng = new PG.seedrand.Rand(garbageSeed == null ? (seed ^ 0x5f3759df) >>> 0 : garbageSeed);
      this.holdKind = null;
      this.holdUsed = false;
      this.startLevel = Math.max(1, Math.trunc(level));
      this.level = this.startLevel;
      this.fixedLevel = fixedLevel;
      this.lines = 0;
      this.score = 0;
      this.pieces = 0;
      this.combo = -1;
      this.b2b = false;
      this.tetrises = 0;
      this.tspins = 0;
      this.attackSent = 0;
      this.garbageReceived = 0;
      this.pending = [];
      this.dead = false;
      this.deadReason = null;
      this.events = [];
      this.version = 0;
      this.pieceId = 0;
      this.active = false;
      this.kind = null;
      this.rot = this.x = this.y = 0;
      this.spawn();
    }

    // ----- Stein erzeugen / Zustand ------------------------------------------
    spawn(kind = null) {
      this.kind = kind != null ? kind : this.queue.pop();
      this.rot = 0;
      this.x = SPAWN_X;
      this.y = SPAWN_Y;
      this.pieceId++;
      this.lastRot = false;
      this.lastKick = -1;
      this.lockTimer = 0;
      this.lockResets = 0;
      this.fallAcc = 0;
      if (collides(this.rows, this.kind, 0, this.x, this.y)) {
        this.active = false;
        this.die("blockout");
        return false;
      }
      this.active = true;
      if (!collides(this.rows, this.kind, 0, this.x, this.y + 1)) this.y++;
      this.lowest = this.y;
      return true;
    }

    die(reason) {
      if (!this.dead) {
        this.dead = true;
        this.deadReason = reason;
        this.active = false;
        this.events.push(["dead", reason]);
      }
    }

    cellsOf(kind = this.kind, rot = this.rot, x = this.x, y = this.y) {
      return SHAPES[kind][rot].cells.map(([cx, cy]) => [x + cx, y + cy]);
    }
    ghostY() {
      return dropY(this.rows, this.kind, this.rot, this.x, this.y);
    }
    grounded() {
      return collides(this.rows, this.kind, this.rot, this.x, this.y + 1);
    }
    pendingLines() {
      let n = 0;
      for (const p of this.pending) n += p[0];
      return n;
    }
    stackHeight() {
      for (let i = 0; i < ROWS; i++) if (this.rows[i]) return ROWS - i;
      return 0;
    }

    // ----- Eingaben ----------------------------------------------------------
    move(dx) {
      if (!this.active) return false;
      if (collides(this.rows, this.kind, this.rot, this.x + dx, this.y)) return false;
      this.x += dx;
      this.lastRot = false;
      this.manipulated();
      return true;
    }
    rotate(d) {
      if (!this.active) return false;
      const res = tryRotate(this.rows, this.kind, this.rot, this.x, this.y, d);
      if (!res) return false;
      [this.rot, this.x, this.y, this.lastKick] = res;
      this.lastRot = true;
      this.manipulated();
      return true;
    }
    manipulated() {
      if (this.y > this.lowest) {
        this.lowest = this.y;
        this.lockResets = 0;
      }
      const onGround = this.grounded();
      if (onGround || this.lockTimer > 0) {
        if (this.lockResets < MAX_RESETS) {
          this.lockResets++;
          this.lockTimer = 0;
        } else if (onGround) {
          this.lockTimer = LOCK_DELAY;
        }
      }
    }
    softStep() {
      if (!this.active || this.grounded()) return false;
      this.y++;
      this.score += 1;
      this.lastRot = false;
      this.fallAcc = 0;
      if (this.y > this.lowest) {
        this.lowest = this.y;
        this.lockResets = 0;
      }
      return true;
    }
    hardDrop() {
      if (!this.active) return null;
      const gy = this.ghostY();
      const d = gy - this.y;
      if (d > 0) {
        this.y = gy;
        this.lastRot = false;
      }
      this.score += 2 * d;
      return this.lock(d);
    }
    hold() {
      if (!this.active || this.holdUsed) return false;
      const cur = this.kind;
      const swap = this.holdKind;
      this.holdKind = cur;
      this.spawn(swap);
      this.holdUsed = true;
      this.events.push(["hold", cur]);
      return true;
    }

    // ----- Zeit --------------------------------------------------------------
    tick(dt, soft = false) {
      for (const p of this.pending) p[2] += dt;
      if (!this.active) return;
      const interval = soft ? softInterval(this.level) : gravityInterval(this.level);
      if (!this.grounded()) {
        this.fallAcc += dt / interval;
        let n = Math.trunc(this.fallAcc);
        if (n > 0) {
          this.fallAcc -= n;
          let moved = 0;
          while (n > 0 && !this.grounded()) {
            this.y++;
            n--;
            moved++;
          }
          if (moved) {
            this.lastRot = false;
            if (soft) this.score += moved;
            if (this.y > this.lowest) {
              this.lowest = this.y;
              this.lockResets = 0;
            }
          }
        }
      }
      if (this.grounded()) {
        this.fallAcc = 0;
        this.lockTimer += dt;
        if (this.lockTimer >= LOCK_DELAY) this.lock(0);
      } else {
        this.lockTimer = 0;
      }
    }

    // ----- Einrasten & Wertung -----------------------------------------------
    occupied(x, y) {
      if (x < 0 || x >= COLS || y < 0 || y >= ROWS) return true;
      return ((this.rows[y] >> x) & 1) === 1;
    }
    spinType() {
      if (this.kind !== "T" || !this.lastRot) return null;
      const cx = this.x + 1, cy = this.y + 1;
      const occ = [this.occupied(cx - 1, cy - 1), this.occupied(cx + 1, cy - 1),
        this.occupied(cx + 1, cy + 1), this.occupied(cx - 1, cy + 1)];
      if (occ.filter(Boolean).length < 3) return null;
      const [a, b] = T_FRONT[this.rot];
      if ((occ[a] && occ[b]) || this.lastKick === 4) return "full";
      return "mini";
    }

    lock(drop = 0) {
      const kind = this.kind;
      const spin = this.spinType();
      const cells = this.cellsOf();
      for (const [px, py] of cells) {
        this.rows[py] |= 1 << px;
        this.cells[py][px] = kind;
      }
      this.version++;
      this.pieces++;
      this.holdUsed = false;
      this.active = false;
      if (cells.every(([, py]) => py < BUFFER)) {
        this.die("lockout");
        return null;
      }

      const full = [...new Set(cells.map(([, py]) => py))].filter((py) => this.rows[py] === FULL).sort((a, b) => a - b);
      const lines = full.length;
      const cleared = full.map((r) => this.cells[r].slice());
      if (lines) {
        const keepRows = [];
        const keepCells = [];
        for (let i = 0; i < ROWS; i++) {
          if (this.rows[i] !== FULL) {
            keepRows.push(this.rows[i]);
            keepCells.push(this.cells[i]);
          }
        }
        this.rows = new Array(lines).fill(0).concat(keepRows);
        this.cells = Array.from({ length: lines }, () => new Array(COLS).fill(null)).concat(keepCells);
      }
      const pc = lines > 0 && this.rows.every((r) => r === 0);

      const difficult = lines === 4 || (spin !== null && lines > 0);
      let b2bBonus = false;
      if (lines) {
        if (difficult) {
          b2bBonus = this.b2b;
          this.b2b = true;
        } else {
          this.b2b = false;
        }
        this.combo++;
      } else {
        this.combo = -1;
      }

      const key = (spin || "") + "|" + lines;
      const level = this.level;
      let base = SCORE_TABLE[key] || 0;
      if (b2bBonus) base = Math.floor((base * 3) / 2);
      let points = base * level;
      if (this.combo > 0) points += COMBO_POINTS * this.combo * level;
      if (pc) points += (b2bBonus && lines === 4 ? PC_POINTS_B2B_TETRIS : PC_POINTS[lines]) * level;
      this.score += points;

      let attack = ATTACK_TABLE[key] || 0;
      if (lines) {
        if (b2bBonus) attack += B2B_ATTACK;
        attack += COMBO_ATTACK[Math.min(this.combo, COMBO_ATTACK.length - 1)];
        if (pc) attack += PC_ATTACK;
      }
      let cancelled = 0;
      let sent = attack;
      while (sent > 0 && this.pending.length) {
        const take = Math.min(sent, this.pending[0][0]);
        this.pending[0][0] -= take;
        sent -= take;
        cancelled += take;
        if (this.pending[0][0] <= 0) this.pending.shift();
      }
      this.attackSent += sent;

      this.lines += lines;
      if (lines === 4) this.tetrises++;
      if (spin === "full" && lines) this.tspins++;
      let levelUp = false;
      if (!this.fixedLevel) {
        const newLevel = Math.max(this.startLevel, 1 + Math.floor(this.lines / 10));
        if (newLevel > this.level) {
          this.level = newLevel;
          levelUp = true;
        }
      }

      const info = { kind, lines, spin, b2b: b2bBonus, combo: this.combo, pc, points, attack,
        sent, cancelled, rows: full, cleared, cells, drop };
      this.events.push(["lock", info]);
      if (levelUp) this.events.push(["level", this.level]);
      if (!lines) this.raiseGarbage();
      if (!this.dead) this.spawn();
      return info;
    }

    // ----- Müll --------------------------------------------------------------
    receive(lines) {
      if (lines <= 0 || this.dead) return;
      this.pending.push([Math.trunc(lines), this.grng.randint(0, COLS - 1), 0.0]);
    }
    raiseGarbage() {
      let total = 0;
      let overflow = false;
      while (this.pending.length && total < GARBAGE_CAP) {
        const p = this.pending[0];
        if (p[2] < GARBAGE_DELAY) break;
        const n = Math.min(p[0], GARBAGE_CAP - total);
        p[0] -= n;
        const hole = p[1];
        if (p[0] <= 0) this.pending.shift();
        for (let i = 0; i < n; i++) {
          if (this.rows[0]) overflow = true;
          this.rows.shift();
          this.cells.shift();
          this.rows.push(FULL & ~(1 << hole));
          const row = new Array(COLS).fill("G");
          row[hole] = null;
          this.cells.push(row);
        }
        total += n;
      }
      if (total) {
        this.version++;
        this.garbageReceived += total;
        this.events.push(["garbage", total]);
        if (overflow) this.die("topout");
      }
    }
  }

  PG.tetrisCore = {
    COLS, VISIBLE, BUFFER, ROWS, FULL, KINDS, SPAWN_X, SPAWN_Y, LOCK_DELAY, MAX_RESETS,
    GARBAGE_DELAY, GARBAGE_CAP, SHAPES, KICKS_I, KICKS_JLSTZ, SCORE_TABLE, ATTACK_TABLE,
    COMBO_ATTACK, gravityInterval, softInterval, collides, tryRotate, dropY, PieceQueue, Board,
  };

  // =================================================================== KI
  const W_LAND = -4.500158825082766;
  const W_ERODE = 3.4181268101392694;
  const W_ROWT = -3.2178882868487753;
  const W_COLT = -9.348695305445199;
  const W_HOLES = -7.899265427351652;
  const W_WELLS = -3.3855972247263626;
  const SAFE_HEIGHT = 9;
  const TETRIS_BONUS = 30.0;
  const SMALL_CLEAR_MALUS = 14.0;
  const WELL_COL_MALUS = 4.0;
  const LEVELS = [
    { think: 0.55, step: 0.11, error: 0.22, top: 4, hold: false, attack: false },
    { think: 0.24, step: 0.06, error: 0.05, top: 3, hold: true, attack: true },
    { think: 0.07, step: 0.03, error: 0.0, top: 1, hold: true, attack: true },
  ];
  const WALLS = 1 | (1 << (COLS + 1));
  const ROW_PAIRS = (1 << (COLS + 1)) - 1;
  const LEFT_WALL = 1;
  const RIGHT_WALL = 1 << (COLS - 1);

  function evaluate(rows, lines, eroded, landH, attack) {
    const H = ROWS;
    let t = 0;
    while (t < H && !rows[t]) t++;
    let rowt = 2 * t;
    let colt = 0, holes = 0, wells = 0, cover = 0, prev = 0, col9 = 0;
    const atk = attack && H - t <= SAFE_HEIGHT;
    const wellCols = atk ? FULL & ~RIGHT_WALL : FULL;
    const wallBit = atk ? RIGHT_WALL : 0;
    for (let i = t; i < H; i++) {
      const r = rows[i];
      const w = ((r | wallBit) << 1) | WALLS;
      rowt += popcount((w ^ (w >> 1)) & ROW_PAIRS);
      colt += popcount(r ^ prev);
      holes += popcount(cover & ~r & FULL);
      cover |= r;
      prev = r;
      let wm = ~r & ((r << 1) | LEFT_WALL) & ((r >> 1) | RIGHT_WALL) & wellCols;
      while (wm) {
        const b = wm & -wm;
        wm ^= b;
        let k = i;
        while (k < H && !(rows[k] & b)) {
          wells++;
          k++;
        }
      }
      if (r & RIGHT_WALL) col9++;
    }
    colt += popcount(prev ^ FULL);
    let score = W_LAND * landH + W_ERODE * eroded + W_ROWT * rowt + W_COLT * colt + W_HOLES * holes + W_WELLS * wells;
    if (atk) {
      if (lines === 4) score += TETRIS_BONUS;
      else if (lines) score -= (SMALL_CLEAR_MALUS * (4 - lines)) / 3.0;
      score -= WELL_COL_MALUS * col9;
    }
    return score;
  }

  function place(rows, kind, rot, x, y) {
    const sh = SHAPES[kind][rot];
    let nrows = rows.slice();
    for (const [dy, m] of sh.rows) nrows[y + dy] |= x >= 0 ? m << x : m >> -x;
    let lines = 0, eroded = 0;
    for (const [dy, m] of sh.rows) {
      if (nrows[y + dy] === FULL) {
        lines++;
        eroded += popcount(m);
      }
    }
    if (lines) nrows = new Array(lines).fill(0).concat(nrows.filter((r) => r !== FULL));
    return [nrows, lines, lines * eroded];
  }

  function tops(rows) {
    const out = new Array(COLS).fill(ROWS);
    let seen = 0;
    for (let i = 0; i < ROWS; i++) {
      const r = rows[i];
      const fresh = r & ~seen;
      if (fresh) {
        for (let c = 0; c < COLS; c++) if ((fresh >> c) & 1) out[c] = i;
        seen |= r;
        if (seen === FULL) break;
      }
    }
    return out;
  }

  function candidates(rows, kind, rot0, x0, y0, attack = false) {
    const out = [];
    const tp = tops(rows);
    const seqs = kind === "O" ? [[]] : [[], [1], [1, 1], [-1]];
    const seen = new Set();
    for (const seq of seqs) {
      let r = rot0, x = x0, y = y0, ok = true;
      for (const d of seq) {
        const res = tryRotate(rows, kind, r, x, y, d);
        if (!res) {
          ok = false;
          break;
        }
        [r, x, y] = res;
      }
      if (!ok) continue;
      const sh = SHAPES[kind][r];
      let xl = x;
      while (!collides(rows, kind, r, xl - 1, y)) xl--;
      let xr = x;
      while (!collides(rows, kind, r, xr + 1, y)) xr++;
      for (let tx = xl; tx <= xr; tx++) {
        let ly = null;
        for (const [cx, by] of sh.bottom) {
          const top = tp[tx + cx];
          if (top <= y + by) {
            ly = dropY(rows, kind, r, tx, y);
            break;
          }
          const cand = top - 1 - by;
          ly = ly === null || cand < ly ? cand : ly;
        }
        const key = sh.cells.map(([cx, cy]) => (ly + cy) * COLS + tx + cx).sort((a, b) => a - b).join(",");
        if (seen.has(key)) continue;
        seen.add(key);
        const [nrows, lines, eroded] = place(rows, kind, r, tx, ly);
        const landH = ROWS - (ly + (sh.miny + sh.maxy) / 2.0);
        out.push([evaluate(nrows, lines, eroded, landH, attack), seq, tx - x, r, tx, ly]);
      }
    }
    out.sort((a, b) => b[0] - a[0]);
    return out;
  }

  function spawnPos(rows, kind) {
    const x = SPAWN_X;
    let y = SPAWN_Y;
    if (collides(rows, kind, 0, x, y)) return null;
    if (!collides(rows, kind, 0, x, y + 1)) y++;
    return [x, y];
  }

  /** Eingabefolge für den aktuellen Stein (endet immer mit "drop"). rng: {random()} oder null. */
  function plan(board, level = 1, rng = null, allowHold = true) {
    if (!board.active) return [];
    const cfg = LEVELS[Math.max(0, Math.min(2, level))];
    const attack = cfg.attack;
    const rows = board.rows;
    let best = candidates(rows, board.kind, board.rot, board.x, board.y, attack);
    let useHold = false;
    if (allowHold && cfg.hold && !board.holdUsed) {
      const alt = board.holdKind || board.queue.peek(1)[0];
      const pos = alt !== board.kind ? spawnPos(rows, alt) : null;
      if (pos) {
        const altC = candidates(rows, alt, 0, pos[0], pos[1], attack);
        if (altC.length && (!best.length || altC[0][0] > best[0][0] + 1.0)) {
          best = altC;
          useHold = true;
        }
      }
    }
    if (!best.length) return ["drop"];
    let pick = best[0];
    if (rng && cfg.error > 0 && rng.random() < cfg.error) {
      pick = best[Math.min(best.length - 1, Math.floor(rng.random() * cfg.top))];
    }
    const [, seq, dx] = pick;
    const acts = useHold ? ["hold"] : [];
    for (const d of seq) acts.push(d > 0 ? "cw" : "ccw");
    for (let i = 0; i < Math.abs(dx); i++) acts.push(dx < 0 ? "left" : "right");
    acts.push("drop");
    return acts;
  }

  function apply(board, act) {
    switch (act) {
      case "hold": return board.hold();
      case "cw": return board.rotate(1);
      case "ccw": return board.rotate(-1);
      case "left": return board.move(-1);
      case "right": return board.move(1);
      case "drop": return board.hardDrop() !== null;
      default: return false;
    }
  }

  PG.tetrisAI = { LEVELS, evaluate, candidates, plan, apply };
})();
