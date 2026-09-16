/*
 * crossyroad_world.js - Welt von Crossy Road ohne Grafik (Port von games/crossyroad_world.py)
 * ==========================================================================================
 * Reihen-Generator (seedrand, verbraucht die Zufallszahlen in exakt derselben
 * Reihenfolge wie die Python-Fassung -> Tagesstrecke am PC und im Browser
 * identisch), Bewegungsformeln für Fahrzeuge/Stämme/Züge, Regeln und
 * Figuren-Katalog. Kein DOM - läuft auch in Node (web/tools/crossyroad_dump.js).
 *
 * Wichtig für die Gleichheit mit Python: ganzzahlige Gewichte, Math.floor nur
 * auf positiven Zahlen, Modulo mit Python-Semantik (pymod).
 */
(function () {
  "use strict";

  const PG = window.PG;

  // ----- Spielfeld -------------------------------------------------------------
  const COLS = 9; // begehbare Spalten 0..8
  const START_COL = 4;
  const MARGIN = 8; // Welt links/rechts außerhalb (Fahrzeuge fahren ein/aus)
  const LOOP = COLS + 2 * MARGIN;
  const START_ROWS = 3; // Reihen 0..2: sichere Startwiese
  const DIFF_ROWS = 160.0;

  const GRASS = "grass", ROAD = "road", RIVER = "river", RAIL = "rail";
  const KINDS = [GRASS, ROAD, RIVER, RAIL];
  const LEVEL = { grass: 0.12, road: 0.0, rail: 0.04, river: -0.12 };

  const CAR_LEN = 1.4;
  const TRUCK_LEN = 2.5;
  const N_CAR_COLORS = 6;
  const TRAIN_CAR = 3.0;
  const TRAIN_SPEED = 24.0;
  const TRAIN_WARN = 1.0;
  const BIG_COIN = 5;

  // ----- Figur & Regeln ----------------------------------------------------------
  const HOP_T = 0.13;
  const HOP_Z = 0.42;
  const CAR_HALF = 0.28;
  const LOG_GRACE = 0.3;
  const EAGLE_ROWS = 3.2;
  const CREEP_LAG = 1.0;
  const CREEP_BASE = 0.45;
  const CREEP_GROW = 0.0016;
  const CREEP_MAX = 0.8;
  const NIGHT_FROM = 50;
  const NIGHT_CYCLE = 90;

  const DIRS = { up: [0, 1], down: [0, -1], left: [-1, 0], right: [1, 0] };

  const CHARACTERS = [
    ["chicken", 0], ["frog", 25], ["pig", 40], ["penguin", 60], ["cat", 80],
    ["fox", 100], ["llama", 125], ["robot", 150], ["ghost", 200], ["unicorn", 250],
  ];
  const CHAR_IDS = CHARACTERS.map((c) => c[0]);
  const PRICES = {};
  for (const [id, p] of CHARACTERS) PRICES[id] = p;
  const HOVER = { ghost: true };

  /** Python-Modulo für Gleitkommazahlen (Ergebnis mit dem Vorzeichen von b). */
  function pymod(a, b) {
    let m = a % b;
    if (m !== 0 && (m < 0) !== (b < 0)) m += b;
    return m === 0 ? 0 : m;
  }

  function difficulty(r) {
    return Math.min(1.0, r / DIFF_ROWS);
  }

  /** Seitliche Hülle: alle Spalten, die von entry aus über erlaubte Nachbarn erreichbar sind. */
  function spread(entry, ok) {
    const seen = new Array(COLS).fill(false);
    const stack = [];
    for (const c of entry) {
      if (!seen[c]) {
        seen[c] = true;
        stack.push(c);
      }
    }
    while (stack.length) {
      const c = stack.pop();
      for (const n of [c - 1, c + 1]) {
        if (n >= 0 && n < COLS && !seen[n] && ok(n)) {
          seen[n] = true;
          stack.push(n);
        }
      }
    }
    const out = [];
    for (let c = 0; c < COLS; c++) if (seen[c]) out.push(c);
    return out;
  }

  function range(n) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(i);
    return out;
  }

  function blank(r, kind) {
    return { r, kind, trees: null, pads: null, dir: 0, speed: 0.0, objs: [], period: 0.0, phase: 0.0, cars: 0, coin: null, reach: range(COLS) };
  }

  /** Kleiner 32-Bit-Hash für reine Deko (identisch zu deco_hash in Python). */
  function decoHash(r, c) {
    let h = (Math.imul(r, 374761393) + Math.imul(c, 668265263)) >>> 0;
    h = Math.imul((h ^ (h >>> 13)) >>> 0, 1274126177) >>> 0;
    return (h ^ (h >>> 16)) >>> 0;
  }

  /** Reihen hinter dem Start (r < 0): Wiese, weiter hinten eine Baumwand. */
  function backRow(r) {
    const row = blank(r, GRASS);
    const trees = new Array(COLS).fill(0);
    for (let c = 0; c < COLS; c++) {
      const h = decoHash(r, c);
      const wall = r === -3 || (r === -2 && [0, 1, 7, 8].includes(c)) || (r === -1 && (c === 0 || c === 8));
      if (wall || (r < -3 && h % 100 < 55)) trees[c] = 1 + ((h >>> 8) % 3);
    }
    row.trees = trees;
    row.reach = range(COLS).filter((c) => trees[c] === 0);
    return row;
  }

  class World {
    constructor(seed) {
      this.seed = seed >>> 0;
      this.rng = new PG.seedrand.Rand(this.seed);
      this.rows = new Map();
      this.nextRow = 0;
      this.prevReach = [START_COL];
      this.lastKind = GRASS;
      this.lastDir = 1;
      this.lastLog = false;
    }

    row(r) {
      if (r < 0) {
        let got = this.rows.get(r);
        if (!got) {
          got = backRow(r);
          this.rows.set(r, got);
        }
        return got;
      }
      while (r >= this.nextRow) this.zone();
      return this.rows.get(r);
    }

    kind(r) {
      return this.row(r).kind;
    }

    /** Anzahl Gleise direkt hintereinander, die in Reihe r enden. */
    railRun(r) {
      let n = 0;
      while (r >= 0 && this.row(r).kind === RAIL) {
        n++;
        r--;
      }
      return n;
    }

    // ------------------------------------------------------------ Generator
    push(row) {
      this.rows.set(row.r, row);
      this.prevReach = row.reach;
      this.nextRow = row.r + 1;
      if (row.dir) this.lastDir = row.dir;
      this.lastLog = row.kind === RIVER && row.pads === null;
    }

    zone() {
      const rng = this.rng;
      const r0 = this.nextRow;
      if (r0 === 0) {
        for (let r = 0; r < START_ROWS; r++) this.grass(r, true);
        this.lastKind = GRASS;
        return;
      }
      const d = difficulty(r0);
      let w;
      if (this.lastKind === GRASS) {
        w = [0, 46 + Math.floor(10 * d), 30 + Math.floor(8 * d), 12 + Math.floor(10 * d)];
      } else {
        w = [58, 20 + Math.floor(10 * d), 14 + Math.floor(8 * d), 6 + Math.floor(8 * d)];
        w[KINDS.indexOf(this.lastKind)] = 0;
      }
      if (r0 < 10) w[3] = 0;
      const kind = KINDS[rng.weighted(w)];
      const early = r0 < 14;
      let n;
      if (kind === GRASS) n = rng.randint(1, 3 - Math.floor(d * 1.5));
      else if (kind === ROAD) n = rng.randint(1, early ? 2 : 2 + Math.floor(3 * d));
      else if (kind === RIVER) n = rng.randint(1, early ? 2 : 2 + Math.floor(2 * d));
      else n = 1 + rng.weighted([10, 6, Math.floor(6 * d), Math.floor(4 * d), Math.floor(4 * d)]);
      for (let i = 0; i < n; i++) {
        const r = this.nextRow;
        if (kind === GRASS) this.grass(r, false);
        else if (kind === ROAD) this.road(r, i);
        else if (kind === RIVER) this.river(r);
        else this.rail(r);
      }
      this.lastKind = kind;
    }

    coin(row, p) {
      const rng = this.rng;
      if (p > 0 && rng.random() < p) {
        const c = rng.choice(row.reach);
        const big = row.r >= 20 && rng.random() < 0.1;
        row.coin = [c, big ? BIG_COIN : 1];
      }
    }

    grass(r, start) {
      const rng = this.rng;
      const row = blank(r, GRASS);
      const trees = new Array(COLS).fill(0);
      if (r > 0) {
        const dens = start ? 0.12 : 0.12 + 0.18 * difficulty(r);
        for (let c = 0; c < COLS; c++) {
          if (rng.random() < dens) trees[c] = rng.randint(1, 4);
        }
        if (start) trees[START_COL] = 0;
      }
      let entry = this.prevReach.filter((c) => trees[c] === 0);
      if (!entry.length) {
        const c = rng.choice(this.prevReach);
        trees[c] = 0;
        entry = [c];
      }
      row.trees = trees;
      row.reach = spread(entry, (c) => trees[c] === 0);
      this.coin(row, r > 0 ? 0.3 : 0.0);
      this.push(row);
    }

    road(r, i) {
      const rng = this.rng;
      const d = difficulty(r);
      const row = blank(r, ROAD);
      let direction;
      if (i > 0 && rng.random() < 0.7) direction = -this.lastDir;
      else direction = rng.random() < 0.5 ? 1 : -1;
      row.dir = direction;
      row.speed = (1.3 + rng.random() * 1.4) * (1.0 + 0.9 * d);
      const ln = rng.random() < 0.28 ? TRUCK_LEN : CAR_LEN;
      const objs = [];
      const gapMin = 2.0 + 1.2 * (1.0 - d);
      const x0 = rng.random() * 2.0;
      let x = x0;
      for (;;) {
        objs.push([x, ln, rng.randint(0, N_CAR_COLORS - 1)]);
        x += ln + gapMin + rng.random() * (3.4 - 1.4 * d);
        if (x + ln + gapMin > LOOP + x0) break;
      }
      row.objs = objs;
      this.coin(row, 0.1);
      this.push(row);
    }

    river(r) {
      const rng = this.rng;
      const d = difficulty(r);
      const row = blank(r, RIVER);
      if (rng.random() < 0.2) {
        const pads = [];
        for (let c = 0; c < COLS; c++) pads.push(rng.random() < 0.45 ? 1 : 0);
        let entry = this.prevReach.filter((c) => pads[c]);
        if (!entry.length) {
          const c = rng.choice(this.prevReach);
          pads[c] = 1;
          entry = [c];
        }
        row.pads = pads;
        row.reach = spread(entry, (c) => pads[c] === 1);
        this.coin(row, 0.15);
      } else {
        let direction;
        if (this.lastLog && rng.random() < 0.8) direction = -this.lastDir;
        else direction = rng.random() < 0.5 ? 1 : -1;
        row.dir = direction;
        row.speed = (0.8 + rng.random() * 0.9) * (1.0 + 0.5 * d);
        const lmax = d < 0.6 ? 4 : 3;
        const objs = [];
        const x0 = rng.random() * 2.0;
        let x = x0;
        for (;;) {
          let ln = rng.randint(2, lmax);
          const room = LOOP + x0 - 1.0 - x;
          if (room < 2.0) break;
          ln = Math.min(ln, Math.floor(room));
          objs.push([x, ln, 0]);
          x += ln + 1.0 + rng.random() * (1.2 + 1.6 * d);
        }
        row.objs = objs;
      }
      this.push(row);
    }

    rail(r) {
      const rng = this.rng;
      const d = difficulty(r);
      const row = blank(r, RAIL);
      row.dir = rng.random() < 0.5 ? 1 : -1;
      const period = 4.6 + rng.random() * 4.0 - 1.2 * d;
      row.period = period;
      row.phase = rng.random() * period;
      row.cars = rng.randint(2, 4);
      this.coin(row, 0.08);
      this.push(row);
    }

    /** Die ersten n Reihen (für den Vergleich mit Python). */
    export(n) {
      const out = [];
      for (let r = 0; r < n; r++) {
        const row = this.row(r);
        out.push({ r: row.r, kind: row.kind, trees: row.trees, pads: row.pads, dir: row.dir, speed: row.speed, objs: row.objs, period: row.period, phase: row.phase, cars: row.cars, coin: row.coin, reach: row.reach });
      }
      return out;
    }
  }

  // ----- Bewegung --------------------------------------------------------------
  function objX(row, obj, t) {
    return pymod(obj[0] + row.dir * row.speed * t, LOOP) - MARGIN;
  }

  /** [warnen, lokSpitze oder null, zuglänge] eines Gleises zur Zeit t. */
  function trainState(row, t) {
    const length = (row.cars + 1) * TRAIN_CAR;
    const u = pymod(t + row.phase, row.period);
    if (u < TRAIN_WARN) return [true, null, length];
    const s = u - TRAIN_WARN;
    if (s * TRAIN_SPEED > LOOP + length) return [false, null, length];
    const dist = s * TRAIN_SPEED;
    const head = row.dir > 0 ? -MARGIN + dist : COLS + MARGIN - dist;
    return [true, head, length];
  }

  function trainSpan(row, t) {
    const [, head, length] = trainState(row, t);
    if (head === null) return null;
    return row.dir > 0 ? [head - length, head] : [head, head + length];
  }

  function vehicleAt(row, cx, t, half = CAR_HALF) {
    for (let i = 0; i < row.objs.length; i++) {
      const o = row.objs[i];
      const x = objX(row, o, t);
      if (x + 0.08 < cx + half && cx - half < x + o[1] - 0.08) return i;
    }
    return -1;
  }

  function trainHits(row, cx, t, half = CAR_HALF) {
    const span = trainSpan(row, t);
    return span !== null && span[0] < cx + half && cx - half < span[1];
  }

  /** [index, segment] des Stamms unter der Figurmitte cx, sonst null. */
  function logAt(row, cx, t, grace = LOG_GRACE) {
    for (let i = 0; i < row.objs.length; i++) {
      const o = row.objs[i];
      const x = objX(row, o, t);
      if (x - grace <= cx && cx <= x + o[1] + grace) {
        let seg = cx > x ? Math.floor(cx - x) : 0;
        seg = Math.max(0, Math.min(Math.floor(o[1]) - 1, seg));
        return [i, seg];
      }
    }
    return null;
  }

  function creepRate(best) {
    return Math.min(CREEP_MAX, CREEP_BASE + CREEP_GROW * Math.max(0, best));
  }

  function nightFactor(rowf) {
    if (rowf < NIGHT_FROM) return 0.0;
    const u = pymod(rowf - NIGHT_FROM, NIGHT_CYCLE);
    let v;
    if (u < 12) v = u / 12.0;
    else if (u < 50) v = 1.0;
    else if (u < 62) v = 1.0 - (u - 50) / 12.0;
    else v = 0.0;
    return v * v * (3 - 2 * v);
  }

  // ----- Modelle (Daten aus crossyroad_models.js) ---------------------------------
  function mirrorX(boxes, length) {
    return boxes.map((b) => [length - b[3], b[1], b[2], length - b[0], b[4], b[5], b[6]]);
  }

  function rotate(boxes, facing) {
    if (facing === "up") return boxes.slice();
    return boxes.map(([x0, y0, z0, x1, y1, z1, col]) => {
      if (facing === "down") return [1 - x1, 1 - y1, z0, 1 - x0, 1 - y0, z1, col];
      if (facing === "right") return [y0, 1 - x1, z0, y1, 1 - x0, z1, col];
      return [1 - y1, x0, z0, 1 - y0, x1, z1, col];
    });
  }

  PG.crossyWorld = {
    COLS, START_COL, MARGIN, LOOP, START_ROWS, GRASS, ROAD, RIVER, RAIL, KINDS, LEVEL,
    CAR_LEN, TRUCK_LEN, N_CAR_COLORS, TRAIN_CAR, TRAIN_SPEED, TRAIN_WARN, BIG_COIN,
    HOP_T, HOP_Z, CAR_HALF, LOG_GRACE, EAGLE_ROWS, CREEP_LAG, CREEP_BASE, CREEP_GROW, CREEP_MAX,
    NIGHT_FROM, NIGHT_CYCLE, DIRS, CHARACTERS, CHAR_IDS, PRICES, HOVER,
    pymod, difficulty, decoHash, backRow, World, objX, trainState, trainSpan, vehicleAt,
    trainHits, logAt, creepRate, nightFactor, mirrorX, rotate,
  };
})();
