/*
 * battleship.js - Battleship / Schiffe versenken (Port von games/battleship.py
 * und games/battleship_core.py)
 * ============================================================================
 * - 10x10-Bretter, Flotte aus Flugzeugträger (5), Schlachtschiff (4), Kreuzer (3),
 *   U-Boot (3) und Zerstörer (2).
 * - Setup: KI-Stärke (Leicht/Mittel/Schwer) und drei Regel-Schalter, alle sofort
 *   gespeichert: Schiffe dürfen sich berühren · Salven-Modus (Schüsse pro Zug =
 *   eigene Schiffe) · nach einem Treffer nochmal schießen.
 * - Aufstellen: Drag & Drop aus dem Dock (oder anklicken und ablegen), R/Rechts-
 *   klick dreht, "Zufällig" und "Alle entfernen"; Vorschau grün/rot.
 * - Gefecht: großes Zielbrett mit Radar-Sweep, rechts das eigene Brett und die
 *   Flottenübersicht; Granaten fliegen ein, Wasser spritzt, Treffer brennen,
 *   versenkte Schiffe werden mit Umriss enthüllt.
 * - KI: leicht (zufällig), mittel (Jagen/Zielen), schwer (Wahrscheinlichkeits-
 *   karte über alle legalen Lagen der Restschiffe + Parität).
 * - Punkte (Highscore) = Siege gegen die KI in einer Sitzung (connect4-Konvention).
 * - Web-Version: nur gegen die KI (der 2-Spieler-Modus mit Übergabe entfällt).
 *
 * Steuerung: Maus oder Pfeile/WASD + Leertaste/Enter. Nach Rundenende:
 * Enter = neue Runde, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // =================================================================== Regeln
  const N = 10;
  const FLEET = [5, 4, 3, 3, 2];
  const SHIP_KEYS = ["carrier", "battleship", "cruiser", "submarine", "destroyer"];
  const UNKNOWN = 0, MISS = 1, HIT = 2;
  const EASY = 0, MEDIUM = 1, HARD = 2;

  function cellsOf(r, c, size, horiz) {
    const out = [];
    for (let i = 0; i < size; i++) out.push(horiz ? [r, c + i] : [r + i, c]);
    return out;
  }
  const inBoard = (r, c) => r >= 0 && r < N && c >= 0 && c < N;

  function neighbors8(r, c) {
    const out = [];
    for (let dr = -1; dr <= 1; dr++)
      for (let dc = -1; dc <= 1; dc++) if ((dr || dc) && inBoard(r + dr, c + dc)) out.push([r + dr, c + dc]);
    return out;
  }

  // Alle Lagen je Schiffslänge: {cells: [idx], halo: [idx]} (halo = 8er-Rand ohne Schiff)
  const PLACEMENTS = {};
  for (const size of [...new Set(FLEET)].sort((a, b) => a - b)) {
    const lst = [];
    for (const horiz of [true, false]) {
      for (let r = 0; r < (horiz ? N : N - size + 1); r++) {
        for (let c = 0; c < (horiz ? N - size + 1 : N); c++) {
          const cells = cellsOf(r, c, size, horiz).map(([rr, cc]) => rr * N + cc);
          const set = new Set(cells);
          const halo = new Set();
          for (const i of cells) for (const [nr, nc] of neighbors8(Math.floor(i / N), i % N)) if (!set.has(nr * N + nc)) halo.add(nr * N + nc);
          lst.push({ cells, halo: [...halo] });
        }
      }
    }
    PLACEMENTS[size] = lst;
  }
  const HALO8 = [], DIAG = [];
  for (let i = 0; i < N * N; i++) {
    const r = Math.floor(i / N), c = i % N;
    HALO8.push(neighbors8(r, c).map(([a, b]) => a * N + b));
    DIAG.push(neighbors8(r, c).filter(([a, b]) => a !== r && b !== c).map(([a, b]) => a * N + b));
  }

  class Ship {
    constructor(idx, r, c, horiz) {
      this.idx = idx;
      this.size = FLEET[idx];
      this.r = r;
      this.c = c;
      this.horiz = horiz;
      this.hits = new Set(); // Feld-Indizes
    }
    get cells() {
      return cellsOf(this.r, this.c, this.size, this.horiz);
    }
    get sunk() {
      return this.hits.size >= this.size;
    }
    has(r, c) {
      return this.horiz ? r === this.r && c >= this.c && c < this.c + this.size : c === this.c && r >= this.r && r < this.r + this.size;
    }
  }

  /** Ein Spielbrett: eigene Schiffe + alle Schüsse, die darauf fielen. */
  class Sea {
    constructor() {
      this.ships = [];
      this.shots = Array.from({ length: N }, () => new Array(N).fill(UNKNOWN));
    }
    shipAt(r, c) {
      for (const s of this.ships) if (s.has(r, c)) return s;
      return null;
    }
    shipByIdx(idx) {
      for (const s of this.ships) if (s.idx === idx) return s;
      return null;
    }
    /** Darf Schiff idx ab Feld (r, c) so liegen? ignore = gerade gehaltenes Schiff. */
    canPlace(idx, r, c, horiz, touch = true, ignore = null) {
      const cells = cellsOf(r, c, FLEET[idx], horiz);
      if (!cells.every(([rr, cc]) => inBoard(rr, cc))) return false;
      const mine = new Set(cells.map(([rr, cc]) => rr * N + cc));
      for (const s of this.ships) {
        if (s === ignore || s.idx === idx) continue;
        for (const [rr, cc] of s.cells) {
          if (mine.has(rr * N + cc)) return false;
          if (!touch) for (const [nr, nc] of neighbors8(rr, cc)) if (mine.has(nr * N + nc)) return false;
        }
      }
      return true;
    }
    place(idx, r, c, horiz) {
      const old = this.shipByIdx(idx);
      if (old) this.ships.splice(this.ships.indexOf(old), 1);
      const ship = new Ship(idx, r, c, horiz);
      this.ships.push(ship);
      return ship;
    }
    remove(ship) {
      const i = this.ships.indexOf(ship);
      if (i >= 0) this.ships.splice(i, 1);
    }
    clear() {
      this.ships = [];
    }
    complete() {
      return this.ships.length === FLEET.length;
    }
    randomize(touch = true) {
      const order = [0, 1, 2, 3, 4].sort((a, b) => FLEET[b] - FLEET[a]);
      for (let attempt = 0; attempt < 200; attempt++) {
        this.ships = [];
        let ok = true;
        for (const idx of order) {
          const spots = [];
          for (const h of [true, false])
            for (let r = 0; r < N; r++) for (let c = 0; c < N; c++) if (this.canPlace(idx, r, c, h, touch)) spots.push([r, c, h]);
          if (!spots.length) {
            ok = false;
            break;
          }
          const [r, c, h] = PG.rand.choice(spots);
          this.place(idx, r, c, h);
        }
        if (ok) return true;
      }
      return false;
    }
    /** Schuss auf (r, c): [ergebnis, schiff] mit "miss"|"hit"|"sunk" oder [null, null]. */
    fire(r, c) {
      if (!inBoard(r, c) || this.shots[r][c] !== UNKNOWN) return [null, null];
      const ship = this.shipAt(r, c);
      if (!ship) {
        this.shots[r][c] = MISS;
        return ["miss", null];
      }
      this.shots[r][c] = HIT;
      ship.hits.add(r * N + c);
      return [ship.sunk ? "sunk" : "hit", ship];
    }
    allSunk() {
      return this.ships.length > 0 && this.ships.every((s) => s.sunk);
    }
    shipsLeft() {
      return this.ships.filter((s) => !s.sunk).length;
    }
    untouched() {
      let n = 0;
      for (const row of this.shots) for (const v of row) if (v === UNKNOWN) n++;
      return n;
    }
    /** Öffentliches Wissen: grid (100 Werte), sunk (Feld-Indizes je versenktem Schiff), remaining (Längen). */
    knowledge() {
      const grid = [];
      for (let i = 0; i < N * N; i++) grid.push(this.shots[Math.floor(i / N)][i % N]);
      const sunk = this.ships.filter((s) => s.sunk).map((s) => s.cells.map(([r, c]) => r * N + c));
      const remaining = this.ships.filter((s) => !s.sunk).map((s) => s.size);
      return [grid, sunk, remaining];
    }
  }

  /** Wählt Schüsse auf ein gegnerisches Brett (nur über Sea.knowledge()). */
  class ShotAI {
    constructor(level, touch) {
      this.level = level;
      this.touch = touch;
      this.parity = PG.rand.randint(0, 5); // Versatz des Paritätsgitters
    }
    choose(sea) {
      const [grid, sunk, remaining] = sea.knowledge();
      const free = [];
      for (let i = 0; i < N * N; i++) if (grid[i] === UNKNOWN) free.push(i);
      if (!free.length) return null;
      let i;
      if (this.level === HARD) i = this.hard(grid, sunk, remaining, free);
      else if (this.level === MEDIUM) i = this.medium(grid, sunk, free);
      else i = this.easy(grid, sunk, free);
      return [Math.floor(i / N), i % N];
    }
    /** [miss, offene Treffer, versenkt] als Uint8Array(100). */
    masks(grid, sunk) {
      const miss = new Uint8Array(N * N), open = new Uint8Array(N * N), sunkM = new Uint8Array(N * N);
      for (const cells of sunk) for (const i of cells) sunkM[i] = 1;
      for (let i = 0; i < N * N; i++) {
        if (grid[i] === MISS) miss[i] = 1;
        else if (grid[i] === HIT && !sunkM[i]) open[i] = 1;
      }
      return [miss, open, sunkM];
    }
    /** Felder, die nach der Berühr-Regel sicher leer sind. */
    knownEmpty(sunkM, open) {
      const empty = new Uint8Array(N * N);
      if (this.touch) return empty;
      for (let i = 0; i < N * N; i++) {
        if (sunkM[i]) for (const j of HALO8[i]) empty[j] = 1;
        if (open[i]) for (const j of DIAG[i]) empty[j] = 1;
      }
      return empty;
    }
    neighbors4(open, freeSet) {
      const out = [];
      for (let i = 0; i < N * N; i++) {
        if (!open[i]) continue;
        const r = Math.floor(i / N), c = i % N;
        for (const [dr, dc] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) {
          const rr = r + dr, cc = c + dc;
          if (inBoard(rr, cc) && freeSet.has(rr * N + cc)) out.push(rr * N + cc);
        }
      }
      return out;
    }
    // ----- Leicht: zufällig, setzt nur ab und zu nach
    easy(grid, sunk, free) {
      const [, open] = this.masks(grid, sunk);
      if (open.includes(1) && PG.rand.random() < 0.35) {
        const near = this.neighbors4(open, new Set(free));
        if (near.length) return PG.rand.choice(near);
      }
      return PG.rand.choice(free);
    }
    // ----- Mittel: jagen und zielen
    medium(grid, sunk, free) {
      const [, open, sunkM] = this.masks(grid, sunk);
      const empty = this.knownEmpty(sunkM, open);
      let freeSet = new Set(free.filter((i) => !empty[i]));
      if (!freeSet.size) freeSet = new Set(free);
      if (open.includes(1)) {
        const ends = [];
        for (let i = 0; i < N * N; i++) {
          if (!open[i]) continue;
          const r = Math.floor(i / N), c = i % N;
          for (const [dr, dc] of [[0, 1], [1, 0]]) {
            if (!inBoard(r + dr, c + dc) || !open[(r + dr) * N + c + dc]) continue;
            // Linie in beide Richtungen bis zum Ende verfolgen
            for (const sgn of [1, -1]) {
              let rr = r, cc = c;
              while (inBoard(rr, cc) && open[rr * N + cc]) {
                rr += dr * sgn;
                cc += dc * sgn;
              }
              if (inBoard(rr, cc) && freeSet.has(rr * N + cc)) ends.push(rr * N + cc);
            }
          }
        }
        if (ends.length) return PG.rand.choice(ends);
        const near = this.neighbors4(open, freeSet);
        if (near.length) return PG.rand.choice(near);
      }
      return PG.rand.choice([...freeSet].sort((a, b) => a - b));
    }
    // ----- Schwer: Wahrscheinlichkeitskarte
    density(grid, sunk, remaining) {
      const [miss, open, sunkM] = this.masks(grid, sunk);
      const empty = this.knownEmpty(sunkM, open);
      const dens = new Float64Array(N * N);
      const target = open.includes(1);
      for (const size of remaining) {
        for (const p of PLACEMENTS[size]) {
          let bad = false, k = 0;
          for (const i of p.cells) {
            if (miss[i] || sunkM[i] || empty[i]) {
              bad = true;
              break;
            }
            if (open[i]) k++;
          }
          if (bad) continue;
          if (!this.touch && p.halo.some((i) => open[i] || sunkM[i])) continue;
          let w = 1;
          if (target) {
            if (!k) continue;
            w = Math.pow(8, k);
          }
          for (const i of p.cells) dens[i] += w;
        }
      }
      return [dens, target];
    }
    hard(grid, sunk, remaining, free) {
      let [dens, target] = this.density(grid, sunk, remaining);
      let cand = free.filter((i) => dens[i] > 0);
      if (!cand.length && target) {
        [dens, target] = this.density(grid.map((v) => (v === HIT ? MISS : v)), sunk, remaining);
        cand = free.filter((i) => dens[i] > 0);
      }
      if (!cand.length) return PG.rand.choice(free);
      if (!target && remaining.length) {
        const step = Math.min(...remaining);
        const par = cand.filter((i) => (Math.floor(i / N) + (i % N) + this.parity) % step === 0);
        if (par.length) cand = par;
      }
      let best = -1;
      for (const i of cand) best = Math.max(best, dens[i]);
      return PG.rand.choice(cand.filter((i) => dens[i] === best));
    }
  }

  // Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme)
  const COL_SEA_TOP = [22, 70, 116];
  const COL_SEA_BOT = [10, 38, 72];
  const COL_RADAR_TOP = [14, 54, 78];
  const COL_RADAR_BOT = [6, 28, 46];
  const COL_GRID = [70, 128, 176];
  const COL_GRID_RADAR = [48, 128, 120];
  const COL_WAVE = [150, 205, 240];
  const COL_RING = [90, 200, 160];
  const COL_HULL = [118, 128, 144];
  const COL_HULL_DARK = [44, 50, 62];
  const COL_DECK = [150, 160, 174];
  const COL_BRIDGE = [196, 202, 212];
  const COL_TURRET = [82, 90, 104];
  const COL_SUB = [70, 82, 96];
  const COL_FLAME = [255, 120, 40];
  const COL_FLAME_IN = [255, 220, 110];
  const COL_SCORCH = [40, 18, 14];
  const COL_MISS = [215, 234, 248];
  const COL_VALID = [80, 220, 120];
  const COL_INVALID = [240, 70, 70];
  const COL_AIM = [255, 214, 110];
  const COL_AI_AIM = [255, 96, 80];
  const COL_P1 = [120, 200, 255];
  const COL_P2 = [255, 150, 110];

  const DIFFS = ["easy", "medium", "hard"];
  const RULES = ["touch", "salvo", "extra_shot"];
  const RULE_TEXT = { touch: "bs.rule.touch", salvo: "bs.rule.salvo", extra_shot: "bs.rule.extra" };
  const BUTTONS = ["rotate", "random", "clear", "ready"];
  const LETTERS = "ABCDEFGHIJ";
  const SETUP = "setup", PLACE = "place", PLAY = "play", OVER = "over";
  const MAX_PARTICLES = 420;

  const smooth = (x) => {
    x = Math.max(0, Math.min(1, x));
    return x * x * (3 - 2 * x);
  };

  // Web-eigener Untertitel (der Python-Text erwähnt den entfallenen 2-Spieler-Modus)
  PG.addStrings({
    de: { "web.battleship.subtitle": "Versenke die feindliche Flotte - gegen die KI" },
    en: { "web.battleship.subtitle": "Sink the enemy fleet - against the AI" },
    fr: { "web.battleship.subtitle": "Coule la flotte ennemie - contre l'IA" },
    es: { "web.battleship.subtitle": "Hunde la flota enemiga - contra la IA" },
    pt: { "web.battleship.subtitle": "Afunda a frota inimiga - contra a IA" },
    pl: { "web.battleship.subtitle": "Zatop wrogą flotę - przeciw AI" },
    tr: { "web.battleship.subtitle": "Düşman filosunu batır - YZ'ye karşı" },
    da: { "web.battleship.subtitle": "Sænk fjendens flåde - mod AI'en" },
    no: { "web.battleship.subtitle": "Senk fiendens flåte - mot AI-en" },
    sv: { "web.battleship.subtitle": "Sänk fiendens flotta - mot AI:n" },
    fi: { "web.battleship.subtitle": "Upota vihollisen laivasto - tekoälyä vastaan" },
    cs: { "web.battleship.subtitle": "Potop nepřátelskou flotilu - proti AI" },
    sl: { "web.battleship.subtitle": "Potopi sovražno floto - proti AI" },
    hr: { "web.battleship.subtitle": "Potopi neprijateljsku flotu - protiv AI-ja" },
  });

  class BattleshipGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.diff = Math.max(0, Math.min(2, parseInt(this.opts.difficulty, 10) || 0));
      this.rules = {};
      for (const k of RULES) this.rules[k] = !!this.opts[k];
      this.cache = new Map();
      this.makeFonts();
      this.wins = [0, 0];
      this.starter = 0;
      this.lastLayout = null;
      this.setupSel = 0;
      this.mouse = [-1, -1];
      this.lastDraw = null;
      this.buildSetupLayout();
      this.newRound();
      this.state = SETUP;
    }

    makeFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(13, Math.min(22, Math.floor(h / 30))));
      this.tiny = ui.font(Math.max(11, Math.min(18, Math.floor(h / 38))));
      this.huge = ui.font(Math.max(26, Math.floor(h / 11)), true);
      this.bold = ui.font(Math.max(15, Math.min(30, Math.floor(h / 24))), true);
    }

    newRound() {
      this.seas = [new Sea(), new Sea()];
      this.stats = [0, 1].map(() => ({ shots: 0, hits: 0, turns: 0 }));
      this.turn = this.starter;
      this.shotsLeft = 0;
      this.turnReport = { shots: 0, hits: 0 };
      this.held = null;
      this.placeHoriz = true;
      this.kcursor = [4, 4];
      this.aim = [4, 4];
      this.kbMode = false;
      this.flight = null;
      this.wait = 0;
      this.after = null; // "next" | "end" | "win"
      this.ai = new ShotAI(this.diff, this.rules.touch);
      this.aiTarget = null;
      this.aiTimer = 0;
      this.aiThink = 1;
      this.aiFrom = [4.5, 4.5];
      this.aiPos = [4.5, 4.5];
      this.reveals = [];
      this.shakeT0 = -9;
      this.shakeAmp = 0;
      this.winner = null;
      this.msg = null;
      this.msgCol = ui.TEXT;
      this.msgT = 0;
      this.overT0 = 0;
      this.particles = [];
      this.layout();
    }

    // ===================================================== Layout
    layout() {
      const w = this.width, h = this.height;
      this.hudH = Math.max(40, Math.floor(h * 0.085));
      const m = Math.max(8, Math.floor(w / 64));
      const gap = Math.max(12, Math.floor(w / 48));
      this.statusH = this.small.height + 12;
      const top = this.hudH + m;
      let cb = Math.floor((h - top - this.statusH - m) / 10.6);
      cb = Math.min(cb, Math.floor(((w - 2 * m - gap) * 0.63) / 10.6));
      this.cb = Math.max(10, cb);
      this.lab = Math.max(12, Math.floor(this.cb * 0.6));
      const colw = w - 2 * m - gap - (this.lab + 10 * this.cb);
      this.cs = Math.max(8, Math.floor(Math.min(this.cb * 0.62, colw / 10)));
      const total = this.lab + 10 * this.cb + gap + Math.max(10 * this.cs, colw);
      const x0 = Math.max(m, Math.floor((w - total) / 2));
      this.bx = x0 + this.lab;
      this.by = top + this.lab;
      this.colX = this.bx + 10 * this.cb + gap;
      this.colW = w - m - this.colX;
      this.sx = this.colX + Math.floor((this.colW - 10 * this.cs) / 2);
      this.sy = this.by;
      const boardBottom = this.by + 10 * this.cb;
      this.statusY = boardBottom + Math.floor(this.statusH / 2) + 2;
      this.labelFont = ui.font(Math.max(10, Math.min(24, Math.floor(this.cb * 0.42))), true);

      const capY = this.sy + 10 * this.cs + 4;
      this.captionY = capY + Math.floor(this.tiny.height / 2);
      const fy = capY + this.tiny.height + Math.max(6, Math.floor(h / 80));
      const fh = Math.max(40, boardBottom + this.statusH - 6 - fy);
      const half = Math.floor((this.colW - 10) / 2);
      this.fleetRects = [new PG.Rect(this.colX, fy, half, fh), new PG.Rect(this.colX + half + 10, fy, half, fh)];

      const titleH = this.small.height + 8;
      this.btnH = Math.max(22, Math.min(40, Math.floor(h * 0.058)));
      const bgap = Math.max(4, Math.floor(this.btnH / 6));
      const readyH = this.btnH + Math.max(4, Math.floor(this.btnH / 4));
      const buttonsH = 3 * this.btnH + 3 * bgap + readyH;
      const dockTop = this.by - this.lab + titleH;
      const dockBottom = boardBottom - buttonsH - bgap;
      const rowH = Math.max(12, Math.floor((dockBottom - dockTop) / FLEET.length));
      this.dockTitleY = this.by - this.lab + Math.floor(titleH / 2);
      this.dockRects = FLEET.map((_, i) => new PG.Rect(this.colX, dockTop + i * rowH, this.colW, rowH));
      this.dockCell = Math.max(6, Math.floor(Math.min(this.cb * 0.7, rowH * 0.62, (this.colW - 30) / 9.5)));
      let y = dockTop + FLEET.length * rowH + bgap;
      this.btnRects = {};
      for (const key of ["rotate", "random", "clear"]) {
        this.btnRects[key] = new PG.Rect(this.colX, y, this.colW, this.btnH);
        y += this.btnH + bgap;
      }
      this.btnRects.ready = new PG.Rect(this.colX, y, this.colW, readyH);
    }

    /** [x, y, cell] des Bretts von Spieler side (0 = du, 1 = KI). */
    boardGeom(side) {
      if (this.state === PLACE) return [this.bx, this.by, this.cb];
      return side === 0 ? [this.sx, this.sy, this.cs] : [this.bx, this.by, this.cb];
    }

    static cellAt(pos, x, y, cell) {
      if (!pos) return null;
      const c = Math.floor((pos[0] - x) / cell);
      const r = Math.floor((pos[1] - y) / cell);
      return r >= 0 && r < N && c >= 0 && c < N ? [r, c] : null;
    }

    // ===================================================== Setup-Screen
    setupItems() {
      return ["diff", ...RULES, "start"];
    }

    buildSetupLayout() {
      const w = this.width, h = this.height;
      const cx = Math.floor(w / 2);
      const bw = Math.min(Math.max(360, Math.floor(w * 0.7)), w - 40);
      const th = this.tiny.height;
      const bh = Math.max(26, Math.min(46, Math.floor(h * 0.075)));
      const gap = Math.max(5, Math.floor(bh / 6));
      const lab = th + 6;
      const block = lab + bh + 2 * gap + lab + 3 * bh + 2 * gap + 3 * gap + bh + 6;
      const top = Math.floor(h * 0.25);
      const bottom = h - 26;
      let y = top + Math.max(0, Math.floor((bottom - top - block) / 2));
      y = Math.min(y, top + Math.floor(h * 0.05));
      y += lab;
      const cw = (bw - 2 * gap) / 3;
      this.diffRects = [0, 1, 2].map((i) => new PG.Rect(Math.floor(cx - bw / 2 + i * (cw + gap)), y, Math.floor(cw), bh));
      this.diffLabelY = y - 4;
      y += bh + 2 * gap;
      y += lab;
      this.rulesLabelY = y - 4;
      this.ruleRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2), y + i * (bh + gap), bw, bh));
      y += 3 * bh + 2 * gap + 3 * gap;
      const sw = Math.max(190, Math.floor(bw * 0.42));
      this.startRect = new PG.Rect(cx - Math.floor(sw / 2), y, sw, bh + 6);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    setDiff(d) {
      this.diff = PG.mod(d, 3);
      this.saveSetting("difficulty", this.diff);
      this.playSound("click");
    }

    toggleRule(key) {
      this.rules[key] = !this.rules[key];
      this.saveSetting(key, this.rules[key]);
      this.playSound("select");
    }

    setupItemRects(item) {
      if (item === "diff") return this.diffRects;
      if (RULES.includes(item)) return [this.ruleRects[RULES.indexOf(item)]];
      return [this.startRect];
    }

    handleSetup(ev) {
      const items = this.setupItems();
      this.setupSel = Math.max(0, Math.min(items.length - 1, this.setupSel));
      if (ev.kind === "keydown") {
        const k = ev.key || "";
        const cur = items[this.setupSel];
        if (k === "1" || k === "2" || k === "3") this.setDiff(parseInt(k, 10) - 1);
        else if (k === "Up" || this.isAction(k, "up")) {
          this.setupSel = PG.mod(this.setupSel - 1, items.length);
          this.playSound("move");
        } else if (k === "Down" || this.isAction(k, "down")) {
          this.setupSel = PG.mod(this.setupSel + 1, items.length);
          this.playSound("move");
        } else if (k === "Left" || k === "Right" || this.isAction(k, "left") || this.isAction(k, "right")) {
          const step = k === "Left" || this.isAction(k, "left") ? -1 : 1;
          if (cur === "diff") this.setDiff(this.diff + step);
          else if (RULES.includes(cur)) this.toggleRule(cur);
        } else if (k === "Return" || k === "KP_Enter") this.startRound();
        else if (k === "space") {
          if (cur === "diff") this.setDiff(this.diff + 1);
          else if (RULES.includes(cur)) this.toggleRule(cur);
          else this.startRound();
        }
      } else if (ev.kind === "mousemove") {
        items.forEach((it, i) => {
          if (this.setupItemRects(it).some((r) => r.collidepoint(ev.pos))) this.setupSel = i;
        });
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (let i = 0; i < 3; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.setupSel = 0;
            this.setDiff(i);
            return;
          }
        }
        for (let i = 0; i < 3; i++) {
          if (this.ruleRects[i].collidepoint(ev.pos)) {
            this.setupSel = items.indexOf(RULES[i]);
            this.toggleRule(RULES[i]);
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startRound();
      }
    }

    // ===================================================== Rundenablauf
    startRound() {
      this.gameOver = false;
      this.newRoundResult();
      this.newRound();
      // Die letzte Aufstellung als Vorschlag wieder auslegen (soweit erlaubt)
      for (const [idx, r, c, horiz] of this.lastLayout || []) {
        if (this.seas[0].canPlace(idx, r, c, horiz, this.rules.touch)) this.seas[0].place(idx, r, c, horiz);
      }
      this.state = PLACE;
      this.playSound("click");
    }

    restart() {
      this.starter = 1 - this.starter;
      this.startRound();
    }

    ready() {
      const sea = this.seas[0];
      if (this.held) this.dropHeld();
      if (!sea.complete()) {
        this.say(t("bs.place_all"), ui.GOLD, 1.6);
        this.playSound("hit");
        return;
      }
      this.lastLayout = sea.ships.map((s) => [s.idx, s.r, s.c, s.horiz]);
      this.playSound("level");
      this.seas[1].randomize(this.rules.touch);
      this.beginTurn(this.starter);
    }

    beginTurn(p) {
      this.turn = p;
      this.state = PLAY;
      this.shotsLeft = this.rules.salvo ? this.seas[p].shipsLeft() : 1;
      this.stats[p].turns += 1;
      this.turnReport = { shots: 0, hits: 0 };
      this.flight = null;
      this.after = null;
      this.wait = 0;
      this.aiTarget = null;
      if (p === 0) this.tone(1320, 0.07, "sine", 0.18); // leises Radar-Ping
    }

    say(text, color, secs = 1.4) {
      this.msg = text;
      this.msgCol = color;
      this.msgT = secs;
    }

    /** Feuert für den Spieler am Zug auf (r, c). true = Schuss unterwegs. */
    fire(r, c) {
      if (this.state !== PLAY || this.flight) return false;
      if ((this.after !== null && this.after !== "next") || this.shotsLeft <= 0) return false;
      const target = this.seas[1 - this.turn];
      if (target.shots[r][c] !== UNKNOWN) {
        if (this.turn === 0) {
          this.say(t("bs.already"), ui.TEXT_DIM, 1.2);
          this.playSound("click");
        }
        return false;
      }
      this.after = null;
      this.wait = 0;
      this.shotsLeft -= 1;
      this.flight = { r, c, t: 0, shooter: this.turn, dur: this.turn === 1 ? 0.46 : 0.36 };
      this.playSound("shoot");
      return true;
    }

    impact() {
      const f = this.flight;
      this.flight = null;
      const shooter = f.shooter, side = 1 - shooter;
      const target = this.seas[side];
      const [res, ship] = target.fire(f.r, f.c);
      if (res === null) {
        this.after = "next";
        this.wait = 0.1;
        return;
      }
      const st = this.stats[shooter];
      st.shots += 1;
      this.turnReport.shots += 1;
      const [x, y, cell] = this.boardGeom(side);
      const px = x + f.c * cell + cell / 2, py = y + f.r * cell + cell / 2;
      const mine = side === 0; // traf es MEINE Flotte?
      let pause;
      if (res === "miss") {
        this.fxSplash(px, py, cell);
        this.tone(180, 0.22, "noise", 0.32);
        this.playSound("bounce");
        this.say(t("bs.miss"), COL_MISS, 1.0);
        pause = 0.55;
      } else {
        st.hits += 1;
        this.turnReport.hits += 1;
        this.fxExplosion(px, py, cell, res === "sunk");
        this.playSound("explode");
        this.shakeT0 = ui.now();
        this.shakeAmp = Math.max(2, cell * (res === "sunk" ? 0.16 : 0.09));
        if (mine) this.rumble(res === "sunk" ? 260 : 140);
        if (this.rules.extra_shot) this.shotsLeft += 1;
        if (res === "sunk") {
          this.reveals.push({ side, ship, t0: ui.now() });
          for (const [rr, cc] of ship.cells) this.fxExplosion(x + cc * cell + cell / 2, y + rr * cell + cell / 2, cell, false, true);
          this.playSound("line");
          this.say(t("bs.sunk", { ship: t("bs.ship." + SHIP_KEYS[ship.idx]) }), mine ? COL_P2 : ui.GOLD, 2.2);
          pause = 1.15;
        } else {
          this.say(t(this.rules.extra_shot ? "bs.hit_again" : "bs.hit"), COL_FLAME, 1.3);
          pause = 0.7;
        }
      }
      if (target.allSunk()) {
        this.after = "win";
        this.wait = 1.7;
      } else if (this.shotsLeft <= 0 || target.untouched() === 0) {
        this.after = "end";
        this.wait = Math.max(pause, 0.95);
      } else {
        this.after = "next";
        this.wait = pause;
      }
    }

    continueTurn() {
      const after = this.after;
      this.after = null;
      if (after === "win") this.finish(this.turn);
      else if (after === "end") this.beginTurn(1 - this.turn);
    }

    finish(winner) {
      this.winner = winner;
      this.state = OVER;
      this.overT0 = ui.now();
      this.wins[winner] += 1;
      if (winner === 0) {
        this.score = this.wins[0];
        this.playSound("win");
        this.reportResult(true);
        if (!this.seas[0].ships.some((s) => s.sunk)) this.achEvent("bs_flawless");
        if (this.diff === HARD) this.achEvent("bs_hard");
      } else {
        this.playSound("gameover");
        this.reportResult(false);
      }
      this.gameOver = true; // die App speichert den Score einmalig
    }

    // ===================================================== Spiellogik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state !== PLAY) return;
      if (this.flight) {
        this.flight.t += dt;
        if (this.flight.t >= this.flight.dur) this.impact();
        return;
      }
      if (this.after !== null) {
        this.wait -= dt;
        if (this.wait <= 0) this.continueTurn();
        return;
      }
      if (this.turn === 1) this.aiStep(dt);
    }

    aiStep(dt) {
      if (!this.aiTarget) {
        const pick = this.ai.choose(this.seas[0]);
        if (!pick) {
          this.after = "end";
          this.wait = 0.2;
          return;
        }
        this.aiTarget = pick;
        const first = this.turnReport.shots === 0;
        this.aiThink = (first ? 0.8 : 0.42) + PG.rand.uniform(0, 0.25);
        this.aiTimer = this.aiThink;
        this.aiFrom = this.aiPos.slice();
      }
      this.aiTimer -= dt;
      const k = smooth(1 - Math.max(0, this.aiTimer) / this.aiThink);
      const [r, c] = this.aiTarget;
      this.aiPos = [this.aiFrom[0] + (c + 0.5 - this.aiFrom[0]) * k, this.aiFrom[1] + (r + 0.5 - this.aiFrom[1]) * k];
      if (this.aiTimer <= 0) {
        this.aiPos = [c + 0.5, r + 0.5];
        this.aiTarget = null;
        this.fire(r, c);
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if ((ev.kind === "mousemove" || ev.kind === "mousedown" || ev.kind === "mouseup") && ev.pos) this.mouse = ev.pos;
      if (this.state === SETUP) this.handleSetup(ev);
      else if (this.state === PLACE) this.handlePlace(ev);
      else if (this.state === PLAY) this.handlePlay(ev);
      else if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "KP_Enter" || ev.key === "space") this.restart();
          else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.setupSel = 0;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ev.button === 1 && ui.now() - this.overT0 > 0.8) this.restart();
      }
    }

    handlePlay(ev) {
      if (this.turn === 1) return;
      const aim = this.aim;
      if (ev.kind === "mousemove") {
        const rc = BattleshipGame.cellAt(ev.pos, this.bx, this.by, this.cb);
        if (rc) {
          aim[0] = rc[0];
          aim[1] = rc[1];
          this.kbMode = false;
        }
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const rc = BattleshipGame.cellAt(ev.pos, this.bx, this.by, this.cb);
        if (rc) {
          aim[0] = rc[0];
          aim[1] = rc[1];
          this.kbMode = false;
          this.fire(rc[0], rc[1]);
        }
      } else if (ev.kind === "keydown") {
        const k = ev.key || "";
        let moved = true;
        if (k === "Up" || this.isAction(k, "up")) aim[0] = PG.mod(aim[0] - 1, N);
        else if (k === "Down" || this.isAction(k, "down")) aim[0] = PG.mod(aim[0] + 1, N);
        else if (k === "Left" || this.isAction(k, "left")) aim[1] = PG.mod(aim[1] - 1, N);
        else if (k === "Right" || this.isAction(k, "right")) aim[1] = PG.mod(aim[1] + 1, N);
        else moved = false;
        if (moved) {
          this.kbMode = true;
          this.playSound("move");
        } else if (k === "space" || k === "Return" || k === "KP_Enter" || this.isAction(k, "action")) {
          this.kbMode = true;
          this.fire(aim[0], aim[1]);
        }
      }
    }

    // ----- Aufstellen ---------------------------------------------------
    pick(idx, grab = 0, viaMouse = true) {
      const sea = this.seas[0];
      const ship = sea.shipByIdx(idx);
      let origin = null, horiz = this.placeHoriz;
      if (ship) {
        origin = [ship.r, ship.c, ship.horiz];
        horiz = ship.horiz;
        sea.remove(ship);
      }
      this.held = { idx, horiz, grab: Math.max(0, Math.min(FLEET[idx] - 1, grab)), origin, mouse: viaMouse, press: this.mouse.slice(), moved: false };
      this.playSound("select");
    }

    /** [r, c] des ersten Felds der Vorschau oder null (Maus neben dem Brett). */
    heldAnchor() {
      const hd = this.held;
      if (!hd) return null;
      const size = FLEET[hd.idx];
      let r, c;
      if (hd.mouse) {
        const rc = BattleshipGame.cellAt(this.mouse, this.bx, this.by, this.cb);
        if (!rc) return null;
        [r, c] = rc;
      } else [r, c] = this.kcursor;
      if (hd.horiz) c -= hd.grab;
      else r -= hd.grab;
      if (!hd.mouse) {
        // Tastatur: Schiff immer ganz im Brett halten
        if (hd.horiz) c = Math.max(0, Math.min(N - size, c));
        else r = Math.max(0, Math.min(N - size, r));
      }
      return [r, c];
    }

    heldValid() {
      const a = this.heldAnchor();
      return !!a && this.seas[0].canPlace(this.held.idx, a[0], a[1], this.held.horiz, this.rules.touch);
    }

    putHeld() {
      const a = this.heldAnchor();
      if (!a) return false;
      const hd = this.held;
      if (!this.seas[0].canPlace(hd.idx, a[0], a[1], hd.horiz, this.rules.touch)) {
        this.say(t("bs.place_invalid"), COL_INVALID, 1.2);
        this.playSound("hit");
        return false;
      }
      this.seas[0].place(hd.idx, a[0], a[1], hd.horiz);
      this.held = null;
      this.playSound("lock");
      return true;
    }

    /** Bricht das Halten ab: zurück an den alten Platz oder ins Dock. */
    dropHeld() {
      const hd = this.held;
      this.held = null;
      if (hd && hd.origin) {
        const [r, c, horiz] = hd.origin;
        if (this.seas[0].canPlace(hd.idx, r, c, horiz, this.rules.touch)) this.seas[0].place(hd.idx, r, c, horiz);
      }
    }

    rotateHeld() {
      this.held.horiz = !this.held.horiz;
      this.placeHoriz = this.held.horiz;
      this.playSound("rotate");
    }

    /** Dreht ein liegendes Schiff um das Feld pivot (wenn es passt). */
    rotatePlaced(ship, pivot) {
      const sea = this.seas[0];
      const k = ship.horiz ? pivot[1] - ship.c : pivot[0] - ship.r;
      const horiz = !ship.horiz;
      const r = horiz ? pivot[0] : pivot[0] - k;
      const c = horiz ? pivot[1] - k : pivot[1];
      const tries = [[r, c]];
      for (const d of [-1, 1, -2, 2, -3, 3, -4, 4]) tries.push(horiz ? [r, c + d] : [r + d, c]);
      for (const [rr, cc] of tries) {
        if (sea.canPlace(ship.idx, rr, cc, horiz, this.rules.touch, ship)) {
          sea.place(ship.idx, rr, cc, horiz);
          this.playSound("rotate");
          return true;
        }
      }
      this.say(t("bs.place_invalid"), COL_INVALID, 1.2);
      this.playSound("hit");
      return false;
    }

    pressButton(key) {
      const sea = this.seas[0];
      if (key === "rotate") {
        if (this.held) this.rotateHeld();
        else {
          const rc = this.kbMode ? this.kcursor.slice() : BattleshipGame.cellAt(this.mouse, this.bx, this.by, this.cb);
          const ship = rc ? sea.shipAt(rc[0], rc[1]) : null;
          if (ship) this.rotatePlaced(ship, rc);
          else {
            this.placeHoriz = !this.placeHoriz;
            this.playSound("rotate");
          }
        }
      } else if (key === "random") {
        this.held = null;
        sea.clear();
        sea.randomize(this.rules.touch);
        this.playSound("merge");
      } else if (key === "clear") {
        this.held = null;
        sea.clear();
        this.playSound("click");
      } else if (key === "ready") this.ready();
    }

    nextUnplaced() {
      for (let idx = 0; idx < FLEET.length; idx++) if (!this.seas[0].shipByIdx(idx)) return idx;
      return null;
    }

    handlePlace(ev) {
      const sea = this.seas[0];
      const onBoard = (p) => BattleshipGame.cellAt(p, this.bx, this.by, this.cb);
      if (ev.kind === "mousemove") {
        const hd = this.held;
        if (hd) {
          if (!hd.mouse && onBoard(ev.pos)) hd.mouse = true;
          if (hd.press && Math.abs(ev.pos[0] - hd.press[0]) + Math.abs(ev.pos[1] - hd.press[1]) > 6) hd.moved = true;
        }
        if (onBoard(ev.pos)) this.kbMode = false;
        return;
      }
      if (ev.kind === "mousedown") {
        const pos = ev.pos;
        const rc = onBoard(pos);
        if (ev.button === 3) {
          if (this.held) this.rotateHeld();
          else if (rc && sea.shipAt(rc[0], rc[1])) this.rotatePlaced(sea.shipAt(rc[0], rc[1]), rc);
          else {
            this.placeHoriz = !this.placeHoriz;
            this.playSound("rotate");
          }
          return;
        }
        if (ev.button !== 1) return;
        this.kbMode = false;
        if (this.held) {
          // Tragen per Klick: auf dem Brett ablegen, sonst zurück
          if (rc) {
            this.held.mouse = true;
            this.putHeld();
            return;
          }
          for (const key of BUTTONS) {
            if (this.btnRects[key].collidepoint(pos)) {
              this.dropHeld();
              this.pressButton(key);
              return;
            }
          }
          this.dropHeld();
          this.playSound("click");
          return;
        }
        for (const key of BUTTONS) {
          if (this.btnRects[key].collidepoint(pos)) {
            this.pressButton(key);
            return;
          }
        }
        if (rc) {
          const ship = sea.shipAt(rc[0], rc[1]);
          if (ship) this.pick(ship.idx, ship.horiz ? rc[1] - ship.c : rc[0] - ship.r);
          return;
        }
        for (let idx = 0; idx < this.dockRects.length; idx++) {
          const drc = this.dockRects[idx];
          if (drc.collidepoint(pos)) {
            const wasPlaced = !!sea.shipByIdx(idx);
            this.pick(idx, Math.floor((pos[0] - (drc.x + 8)) / Math.max(1, this.dockCell)));
            if (!wasPlaced) this.held.horiz = true;
            return;
          }
        }
        return;
      }
      if (ev.kind === "mouseup") {
        const hd = this.held;
        if (ev.button === 3 || !hd || !hd.moved || !hd.mouse) return;
        if (onBoard(ev.pos) && this.heldValid()) this.putHeld();
        else {
          if (onBoard(ev.pos)) {
            this.say(t("bs.place_invalid"), COL_INVALID, 1.2);
            this.playSound("hit");
          }
          this.dropHeld();
        }
        return;
      }
      if (ev.kind !== "keydown") return;
      const k = ev.key || "";
      let dr = 0, dc = 0;
      if (k === "Up" || this.isAction(k, "up")) dr = -1;
      else if (k === "Down" || this.isAction(k, "down")) dr = 1;
      else if (k === "Left" || this.isAction(k, "left")) dc = -1;
      else if (k === "Right" || this.isAction(k, "right")) dc = 1;
      if (dr || dc) {
        if (!this.kbMode && !this.held) {
          const rc = onBoard(this.mouse);
          if (rc) this.kcursor = rc;
        }
        this.kbMode = true;
        if (this.held && this.held.mouse) {
          this.held.mouse = false;
          const rc = onBoard(this.mouse);
          if (rc) this.kcursor = rc;
        }
        this.kcursor[0] = PG.mod(this.kcursor[0] + dr, N);
        this.kcursor[1] = PG.mod(this.kcursor[1] + dc, N);
        this.playSound("move");
        return;
      }
      if ((k === "r" || k === "R") && this.keyIsFree(k)) return this.pressButton("rotate");
      if ((k === "x" || k === "X") && this.keyIsFree(k)) return this.pressButton("random");
      if (((k === "c" || k === "C") && this.keyIsFree(k)) || k === "Delete" || k === "BackSpace") return this.pressButton("clear");
      const isEnter = k === "Return" || k === "KP_Enter";
      if (k === "space" || isEnter || this.isAction(k, "action")) {
        this.kbMode = true;
        if (this.held) {
          this.held.mouse = false;
          this.putHeld();
          return;
        }
        const ship = sea.shipAt(this.kcursor[0], this.kcursor[1]);
        const grabOf = (s) => (s.horiz ? this.kcursor[1] - s.c : this.kcursor[0] - s.r);
        if (isEnter && sea.complete()) this.ready();
        else if (ship && !isEnter) this.pick(ship.idx, grabOf(ship), false);
        else {
          const idx = this.nextUnplaced();
          if (idx !== null) this.pick(idx, 0, false);
          else if (ship) this.pick(ship.idx, grabOf(ship), false);
        }
      }
    }

    // ===================================================== Effekte
    spawn(p) {
      if (this.particles.length >= MAX_PARTICLES) return;
      this.particles.push(Object.assign({ age: 0, g: 0, vx: 0, vy: 0 }, p));
    }

    fxSplash(x, y, cell) {
      const k = cell / 40;
      this.spawn({ kind: "flash", x, y, life: 0.16, r: cell * 0.45, col: [210, 235, 255] });
      this.spawn({ kind: "ring", x, y, life: 0.7, r0: cell * 0.12, r1: cell * 0.62 });
      this.spawn({ kind: "ring", x, y, life: 0.95, r0: cell * 0.05, r1: cell * 0.9 });
      for (let i = 0; i < 16; i++) {
        const ang = PG.rand.uniform(Math.PI * 1.05, Math.PI * 1.95);
        const sp = PG.rand.uniform(70, 230) * k;
        this.spawn({ kind: "drop", x, y, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp * 1.4, g: 620 * k, life: PG.rand.uniform(0.45, 0.8), size: PG.rand.uniform(1.2, 2.8) * Math.max(1, k) });
      }
    }

    fxExplosion(x, y, cell, big = false, quiet = false) {
      const k = cell / 40;
      const n = quiet ? 10 : big ? 34 : 22;
      this.spawn({ kind: "flash", x, y, life: quiet ? 0.16 : 0.22, r: cell * (big ? 1.1 : 0.8), col: [255, 190, 90] });
      for (let i = 0; i < n; i++) {
        const ang = PG.rand.uniform(0, PG.TAU);
        const sp = PG.rand.uniform(80, big ? 360 : 280) * k;
        this.spawn({ kind: "spark", x, y, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp - 60 * k, g: 520 * k, life: PG.rand.uniform(0.3, 0.7), size: PG.rand.uniform(1.5, 3.2) * Math.max(1, k), col: PG.rand.choice([[255, 230, 120], [255, 160, 60], [255, 110, 40]]) });
      }
      for (let i = 0; i < (quiet ? 3 : 6); i++) {
        this.spawn({ kind: "smoke", x: x + PG.rand.uniform(-0.2, 0.2) * cell, y: y + PG.rand.uniform(-0.1, 0.2) * cell, vx: PG.rand.uniform(-12, 12) * k, vy: PG.rand.uniform(-40, -18) * k, life: PG.rand.uniform(0.9, 1.6), r0: cell * 0.16, r1: cell * 0.55 });
      }
    }

    /** Gecachte Offscreen-Canvas (scharf über pixelScale). */
    sprite(key, w, h, paint) {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const full = key + "|" + ps;
      let c = this.cache.get(full);
      if (!c) {
        if (this.cache.size > 600) this.cache.clear();
        c = ui.makeCanvas(Math.ceil(w * ps), Math.ceil(h * ps));
        const g = c.getContext("2d");
        g.scale(ps, ps);
        paint(g);
        c.lw = w;
        c.lh = h;
        this.cache.set(full, c);
      }
      return c;
    }

    glow(radius, color) {
      radius = Math.max(4, Math.floor(radius / 3) * 3);
      return this.sprite("glow|" + radius + "|" + color.join(","), radius * 2, radius * 2, (g) => {
        const grad = g.createRadialGradient(radius, radius, 0, radius, radius, radius);
        grad.addColorStop(0, ui.col(color, 0.82));
        grad.addColorStop(0.45, ui.col(color, 0.35));
        grad.addColorStop(1, ui.col(color, 0));
        g.fillStyle = grad;
        g.fillRect(0, 0, radius * 2, radius * 2);
      });
    }

    puff(radius) {
      radius = Math.max(3, Math.floor(radius / 2) * 2);
      return this.sprite("puff|" + radius, radius * 2, radius * 2, (g) => {
        const grad = g.createRadialGradient(radius, radius, 0, radius, radius, radius);
        grad.addColorStop(0, "rgba(70,72,78,0.62)");
        grad.addColorStop(0.7, "rgba(70,72,78,0.3)");
        grad.addColorStop(1, "rgba(70,72,78,0)");
        g.fillStyle = grad;
        g.fillRect(0, 0, radius * 2, radius * 2);
      });
    }

    blit(ctx, c, x, y, alpha) {
      if (alpha !== undefined && alpha < 1) {
        ctx.save();
        ctx.globalAlpha = Math.max(0, alpha);
        ctx.drawImage(c, x, y, c.lw, c.lh);
        ctx.restore();
      } else ctx.drawImage(c, x, y, c.lw, c.lh);
    }

    stepParticles(ctx, dt) {
      const alive = [];
      for (const p of this.particles) {
        p.age += dt;
        if (p.age >= p.life) continue;
        const f = p.age / p.life;
        p.vy += p.g * dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        if (p.kind === "spark") {
          const sz = Math.max(1, Math.floor(p.size * (1 - f * 0.6)));
          draw.rect(ctx, ui.mix(p.col, [120, 30, 10], f), [Math.floor(p.x), Math.floor(p.y), sz, sz]);
        } else if (p.kind === "drop") {
          draw.circle(ctx, ui.mix([240, 250, 255], COL_SEA_TOP, f * 0.9), [p.x, p.y], Math.max(1, p.size));
        } else if (p.kind === "ring") {
          const rr = p.r0 + (p.r1 - p.r0) * Math.sqrt(f);
          draw.circle(ctx, ui.mix([220, 240, 255], COL_SEA_TOP, f), [p.x, p.y], Math.max(2, rr), Math.max(1, Math.floor(3 * (1 - f)) + 1));
        } else if (p.kind === "smoke") {
          const img = this.puff(p.r0 + (p.r1 - p.r0) * f);
          this.blit(ctx, img, p.x - img.lw / 2, p.y - img.lh / 2, 1 - f);
        } else if (p.kind === "flash") {
          const img = this.glow(p.r * (0.6 + 0.6 * f), p.col);
          this.blit(ctx, img, p.x - img.lw / 2, p.y - img.lh / 2, 1 - f);
        }
        alive.push(p);
      }
      this.particles = alive;
    }

    // ===================================================== Sprites
    shipSprite(idx, cell, horiz, style = "normal") {
      const size = FLEET[idx];
      const L = size * cell;
      return this.sprite(["ship", idx, cell, horiz, style].join("|"), horiz ? L : cell, horiz ? cell : L, (g) => {
        if (!horiz) {
          g.translate(cell, 0);
          g.rotate(Math.PI / 2);
        }
        this.renderShip(g, idx, cell);
        if (style === "sunk") {
          g.globalCompositeOperation = "source-atop";
          g.fillStyle = "rgba(60,26,18,0.55)";
          g.fillRect(0, 0, L, cell);
          for (let i = 0; i < size; i++) {
            const cx = Math.floor((i + 0.5) * cell) + Math.floor(cell * 0.1 * (((i * 7) % 3) - 1));
            draw.circle(g, [24, 12, 10, 150], [cx, Math.floor(cell / 2)], Math.max(2, Math.floor(cell * 0.2)));
          }
          g.globalCompositeOperation = "source-over";
        }
      });
    }

    renderShip(g, idx, cell) {
      const size = FLEET[idx];
      const L = size * cell, H = cell;
      const p = Math.max(2, Math.floor(cell * 0.14));
      const top = p, bot = H - p, mid = H / 2, hullH = bot - top;
      const x0 = p, x1 = L - Math.max(1, Math.floor(p / 2));
      const lw = Math.max(1, Math.floor(cell / 18));
      const key = SHIP_KEYS[idx];
      if (key === "submarine") {
        const body = new PG.Rect(x0, Math.floor(top + hullH * 0.14), x1 - x0, Math.max(3, Math.floor(hullH * 0.72)));
        draw.ellipse(g, COL_SUB, body);
        draw.ellipse(g, ui.mix(COL_SUB, [255, 255, 255], 0.22), [body.x + body.w * 0.08, body.y + Math.max(1, body.h / 6), body.w * 0.84, body.h * 0.45]);
        draw.ellipse(g, COL_HULL_DARK, body, lw);
        const tw = Math.max(3, Math.floor(cell * 0.62)), th = Math.max(3, Math.floor(hullH * 0.5));
        const tower = [Math.floor(L * 0.4 - tw / 2), Math.floor(mid - th / 2), tw, th];
        draw.rect(g, COL_DECK, tower, 0, Math.max(1, Math.floor(th / 2)));
        draw.rect(g, COL_HULL_DARK, tower, lw, Math.max(1, Math.floor(th / 2)));
        draw.line(g, COL_HULL_DARK, [tower[0] + tw / 2, Math.floor(mid)], [Math.min(x1 - p, tower[0] + tw + Math.floor(cell * 0.35)), Math.floor(mid)], lw);
        return;
      }
      const bow = Math.min(cell * 0.95, L * 0.32);
      const pts = [[x0 + p, top], [x1 - bow, top], [x1, mid], [x1 - bow, bot], [x0 + p, bot], [x0, bot - p * 0.7], [x0, top + p * 0.7]];
      draw.polygon(g, COL_HULL, pts);
      const d = Math.max(1, Math.floor(hullH * 0.2));
      const deck = [[x0 + p + d * 0.5, top + d], [x1 - bow + d * 0.2, top + d], [x1 - d * 1.6, mid], [x1 - bow + d * 0.2, bot - d], [x0 + p + d * 0.5, bot - d], [x0 + d, bot - p * 0.7 - d * 0.3], [x0 + d, top + p * 0.7 + d * 0.3]];
      draw.polygon(g, COL_DECK, deck);
      draw.polygon(g, COL_HULL_DARK, pts, lw);
      if (key === "carrier") {
        // Flugdeck mit Mittellinie und Insel an der Seite
        const dash = Math.max(2, Math.floor(cell / 4));
        const yy = Math.floor(mid);
        for (let xx = Math.floor(x0 + p + d); xx < x1 - bow; xx += dash * 2) {
          draw.line(g, [235, 238, 240], [xx, yy], [Math.min(xx + dash, Math.floor(x1 - bow)), yy], Math.max(1, Math.floor(cell / 20)));
        }
        const iw = Math.max(3, Math.floor(cell * 0.7)), ih = Math.max(2, Math.floor(hullH * 0.3));
        const island = [Math.floor(L * 0.58), Math.floor(top + 1), iw, ih];
        draw.rect(g, COL_BRIDGE, island, 0, Math.max(1, Math.floor(ih / 3)));
        draw.rect(g, COL_HULL_DARK, island, lw, Math.max(1, Math.floor(ih / 3)));
        return;
      }
      const bw = Math.max(3, Math.floor(cell * 0.62)), bh = Math.max(3, Math.floor(hullH * 0.52));
      const bridge = new PG.Rect(Math.floor(L * 0.44 - bw / 2), Math.floor(mid - bh / 2), bw, bh);
      draw.rect(g, COL_BRIDGE, bridge, 0, Math.max(1, Math.floor(bh / 4)));
      draw.rect(g, COL_HULL_DARK, bridge, lw, Math.max(1, Math.floor(bh / 4)));
      if (cell >= 20) {
        const wx = bridge.right - Math.max(2, Math.floor(bw / 4));
        draw.line(g, [60, 90, 120], [wx, bridge.y + 2], [wx, bridge.bottom - 3], Math.max(1, Math.floor(cell / 16)));
      }
      const turrets = { battleship: [0.18, 0.7, 0.84], cruiser: [0.2, 0.76], destroyer: [0.74] }[key] || [];
      const tr = Math.max(2, Math.floor(hullH * 0.22));
      for (const fx of turrets) {
        let tx = Math.floor(L * fx);
        if (tx + tr >= x1 - bow * 0.35) tx = Math.floor(x1 - bow * 0.35 - tr - 1);
        draw.line(g, COL_HULL_DARK, [tx, Math.floor(mid)], [tx + Math.floor(tr * 1.9), Math.floor(mid)], Math.max(1, Math.floor(tr / 2)));
        draw.circle(g, COL_TURRET, [tx, Math.floor(mid)], tr);
        draw.circle(g, COL_HULL_DARK, [tx, Math.floor(mid)], tr, Math.max(1, lw));
      }
    }

    /** Gecachte Brettfläche: Wasserverlauf, Schachbrett-Schimmer, Raster (+ Radarringe). */
    boardBase(cell, kind) {
      const size = cell * N;
      return this.sprite("base|" + cell + "|" + kind, size, size, (g) => {
        const radar = kind === "target";
        const top = radar ? COL_RADAR_TOP : COL_SEA_TOP, bot = radar ? COL_RADAR_BOT : COL_SEA_BOT;
        const grad = g.createLinearGradient(0, 0, 0, size);
        grad.addColorStop(0, ui.col(top));
        grad.addColorStop(1, ui.col(bot));
        g.fillStyle = grad;
        g.fillRect(0, 0, size, size);
        g.save();
        g.globalCompositeOperation = "lighter";
        g.fillStyle = radar ? "rgb(3,9,8)" : "rgb(5,8,10)";
        for (let r = 0; r < N; r++) for (let c = 0; c < N; c++) if ((r + c) % 2) g.fillRect(c * cell, r * cell, cell, cell);
        g.restore();
        if (radar) {
          const ring = ui.mix(COL_RADAR_TOP, COL_RING, 0.28);
          for (const i of [1, 2, 3, 4]) draw.circle(g, ring, [size / 2, size / 2], Math.floor(size * 0.125 * i), 1);
          const cross = ui.mix(COL_RADAR_TOP, COL_RING, 0.2);
          draw.line(g, cross, [size / 2, 0], [size / 2, size]);
          draw.line(g, cross, [0, size / 2], [size, size / 2]);
        }
        const gcol = ui.mix(bot, radar ? COL_GRID_RADAR : COL_GRID, 0.55);
        for (let i = 0; i <= N; i++) {
          draw.line(g, gcol, [i * cell, 0], [i * cell, size]);
          draw.line(g, gcol, [0, i * cell], [size, i * cell]);
        }
        draw.rect(g, ui.mix(bot, radar ? COL_GRID_RADAR : COL_GRID, 0.9), [0, 0, size, size], Math.max(1, Math.floor(cell / 20)));
      });
    }

    waveGlyph(cell, lvl) {
      const gw = Math.max(6, Math.floor(cell * 0.5)), gh = Math.max(3, Math.floor(cell * 0.18));
      const alpha = [26, 52, 80, 110][lvl];
      return this.sprite("wave|" + cell + "|" + lvl, gw, gh, (g) => {
        const pts = [];
        const step = Math.max(1, Math.floor(gw / 10));
        for (let i = 0; i <= gw; i += step) pts.push([i, gh / 2 + Math.sin((i / gw) * PG.TAU) * gh * 0.38]);
        draw.lines(g, [COL_WAVE[0], COL_WAVE[1], COL_WAVE[2], alpha], false, pts, Math.max(1, Math.floor(cell / 22)));
      });
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      const now = ui.now();
      let dt = this.lastDraw === null ? 0 : Math.max(0, Math.min(0.05, now - this.lastDraw));
      if (this.paused) dt = 0;
      this.lastDraw = now;
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx, now);
        return;
      }
      this.drawHud(ctx);
      if (this.state === PLACE) this.drawPlace(ctx, now);
      else this.drawBattle(ctx, now);
      this.stepParticles(ctx, dt);
      if (this.state === OVER) this.drawOver(ctx, now);
    }

    /** Text höchstens maxw breit: erst kleinere Schriften, dann kürzen -> [font, text]. */
    fit(font, text, maxw) {
      const chain = [font];
      if (font === this.huge) chain.push(this.bold);
      if (font === this.huge || font === this.bold) chain.push(this.small);
      if (font !== this.tiny) chain.push(this.tiny);
      for (const f of chain) if (f.width(text) <= maxw) return [f, text];
      let kurz = text;
      while (kurz.length > 2 && this.tiny.width(kurz + "…") > maxw) kurz = kurz.slice(0, -1);
      return [this.tiny, kurz + "…"];
    }

    fitText(ctx, font, text, maxw, x, y, color, anchor) {
      const [f, s] = this.fit(font, text, maxw);
      return ui.text(ctx, s, x, y, f, color, anchor);
    }

    // ----- Brett ---------------------------------------------------------
    drawWaves(ctx, x, y, cell, now, skip) {
      const g0 = this.waveGlyph(cell, 0);
      const gw = g0.lw, gh = g0.lh;
      for (let r = 0; r < N; r++) {
        for (let c = 0; c < N; c++) {
          if (skip && skip[r][c]) continue;
          const ph = r * 1.37 + c * 2.11;
          const a = Math.sin(now * 1.25 + ph);
          if (a < -0.35) continue;
          const lvl = Math.min(3, Math.floor(((a + 0.35) / 1.35) * 4));
          const dx = Math.cos(now * 0.9 + ph) * cell * 0.12;
          const yy = y + r * cell + cell * (0.3 + (0.35 * ((r * 3 + c * 5) % 3)) / 2) - gh / 2;
          this.blit(ctx, this.waveGlyph(cell, lvl), x + c * cell + (cell - gw) / 2 + dx, yy);
        }
      }
    }

    drawLabels(ctx, x, y, cell) {
      for (let i = 0; i < N; i++) {
        ui.text(ctx, LETTERS[i], x + i * cell + cell / 2, y - this.lab / 2, this.labelFont, ui.TEXT_DIM, "center");
        ui.text(ctx, String(i + 1), x - this.lab / 2, y + i * cell + cell / 2, this.labelFont, ui.TEXT_DIM, "center");
      }
    }

    drawMiss(ctx, x, y, cell, now, seed) {
      const cx = x + cell / 2, cy = y + cell / 2;
      const fr = (now * 0.55 + seed * 0.137) % 1;
      const rr = Math.floor(cell * (0.16 + 0.26 * fr));
      if (rr > 2) draw.circle(ctx, ui.mix(COL_MISS, COL_SEA_BOT, 0.35 + 0.6 * fr), [cx, cy], rr, Math.max(1, Math.floor(cell / 26)));
      draw.circle(ctx, ui.mix(COL_MISS, COL_SEA_BOT, 0.25), [cx, cy], Math.max(2, Math.floor(cell * 0.2)), Math.max(1, Math.floor(cell / 16)));
      draw.circle(ctx, COL_MISS, [cx, cy], Math.max(1, Math.floor(cell * 0.08)));
    }

    drawFire(ctx, x, y, cell, now, seed, strong) {
      const cx = x + cell / 2, cy = y + cell / 2;
      const gl = this.glow(cell * (strong ? 0.55 : 0.4), [255, 120, 40]);
      this.blit(ctx, gl, cx - gl.lw / 2, cy - gl.lh / 2, ((strong ? 150 : 90) + 50 * Math.sin(now * 11 + seed)) / 255);
      draw.circle(ctx, COL_SCORCH, [cx, cy + cell * 0.06], Math.max(2, Math.floor(cell * 0.22)));
      const base = cy + cell * 0.22;
      const scale = strong ? 1 : 0.6;
      for (let i = 0; i < 3; i++) {
        const ph = now * 9 + seed * 1.7 + i * 2.1;
        const fh = cell * (0.34 + 0.1 * Math.sin(ph)) * scale * (i !== 1 ? 0.8 : 1);
        const fx = cx + (i - 1) * cell * 0.13;
        const sway = Math.sin(ph * 1.3) * cell * 0.05;
        const w = cell * 0.1 * scale;
        draw.polygon(ctx, COL_FLAME, [[fx - w, base], [fx + sway, base - fh], [fx + w, base]]);
        draw.polygon(ctx, COL_FLAME_IN, [[fx - w * 0.45, base], [fx + sway * 0.6, base - fh * 0.55], [fx + w * 0.45, base]]);
      }
    }

    drawSea(ctx, side, x, y, cell, kind, now) {
      const sea = this.seas[side];
      this.blit(ctx, this.boardBase(cell, kind), x, y);
      const occupied = Array.from({ length: N }, () => new Array(N).fill(false));
      const shown = [];
      const revealAll = this.state === OVER;
      for (const ship of sea.ships) {
        if (kind === "own" || ship.sunk || revealAll) {
          shown.push(ship);
          for (const [r, c] of ship.cells) occupied[r][c] = true;
        }
      }
      for (let r = 0; r < N; r++) for (let c = 0; c < N; c++) if (sea.shots[r][c] !== UNKNOWN) occupied[r][c] = true;
      this.drawWaves(ctx, x, y, cell, now, occupied);
      for (const ship of shown) {
        if (ship.sunk) this.blit(ctx, this.shipSprite(ship.idx, cell, ship.horiz, "sunk"), x + ship.c * cell, y + ship.r * cell);
        else this.blit(ctx, this.shipSprite(ship.idx, cell, ship.horiz), x + ship.c * cell, y + ship.r * cell, kind === "own" ? 1 : 150 / 255);
      }
      const sunkCells = new Set();
      for (const sh of sea.ships) if (sh.sunk) for (const [r, c] of sh.cells) sunkCells.add(r * N + c);
      for (let r = 0; r < N; r++) {
        for (let c = 0; c < N; c++) {
          const v = sea.shots[r][c];
          if (v === MISS) this.drawMiss(ctx, x + c * cell, y + r * cell, cell, now, r * 10 + c);
          else if (v === HIT) this.drawFire(ctx, x + c * cell, y + r * cell, cell, now, r * 10 + c, !sunkCells.has(r * N + c));
        }
      }
      if (kind === "target") this.drawRadar(ctx, x, y, cell * N, now);
      // Versenkt-Enthüllung: Umriss pulsiert auf
      for (const rv of this.reveals) {
        if (rv.side !== side) continue;
        const k = (now - rv.t0) / 1.4;
        if (k < 0 || k >= 1) continue;
        const sh = rv.ship;
        const w = (sh.horiz ? sh.size : 1) * cell, h = (sh.horiz ? 1 : sh.size) * cell;
        const grow = Math.floor(cell * 0.35 * (1 - k));
        const rect = new PG.Rect(x + sh.c * cell, y + sh.r * cell, w, h).inflate(grow, grow);
        draw.rect(ctx, ui.mix([255, 255, 255], ui.GOLD, k), rect, Math.max(2, Math.floor(cell * 0.09 * (1 - k)) + 1), Math.max(2, Math.floor(cell / 5)));
      }
    }

    /** "VERSENKT!" steigt über dem gerade enthüllten Schiff auf und verblasst. */
    drawSunkBanners(ctx, now, ox, oy) {
      for (const rv of this.reveals) {
        const k = (now - rv.t0) / 1.6;
        if (k < 0 || k >= 1) continue;
        const [x, y, cell] = this.boardGeom(rv.side);
        const sh = rv.ship;
        const cx = x + ox + (sh.c + (sh.horiz ? sh.size / 2 : 0.5)) * cell;
        const cy = y + oy + (sh.r + (sh.horiz ? 0.5 : sh.size / 2)) * cell;
        const mine = rv.side === 0;
        const base = mine ? this.small : this.bold;
        const pop = 1 + 0.25 * Math.max(0, 1 - k * 6);
        const font = pop > 1.01 ? ui.font(base.px * pop, true) : base;
        const txt = t("bs.sunk_banner");
        const half = font.width(txt) / 2 + 4;
        const bx = Math.max(half, Math.min(this.width - half, cx));
        const yy = cy - cell * (0.2 + 0.9 * smooth(k));
        ctx.save();
        ctx.globalAlpha = Math.min(1, (1 - k) / 0.35);
        ui.text(ctx, txt, bx + 2, yy + 2, font, [8, 12, 20], "center");
        ui.text(ctx, txt, bx, yy, font, mine ? COL_P2 : ui.GOLD, "center");
        ctx.restore();
      }
    }

    drawRadar(ctx, x, y, size, now) {
      const ang = now * 1.5;
      const cx = x + size / 2, cy = y + size / 2;
      const R = size * 0.72, steps = 12, span = 1.05;
      ctx.save();
      ctx.beginPath();
      ctx.rect(x, y, size, size);
      ctx.clip();
      ctx.globalCompositeOperation = "lighter";
      for (let i = 0; i < steps; i++) {
        const a0 = ang - (span * (i + 1)) / steps, a1 = ang - (span * i) / steps;
        const k = Math.pow(1 - i / steps, 2);
        draw.polygon(ctx, [Math.floor(6 * k), Math.floor(62 * k), Math.floor(44 * k)], [[cx, cy], [cx + Math.cos(a0) * R, cy + Math.sin(a0) * R], [cx + Math.cos(a1) * R, cy + Math.sin(a1) * R]]);
      }
      draw.line(ctx, [40, 150, 105], [cx, cy], [cx + Math.cos(ang) * R, cy + Math.sin(ang) * R], Math.max(2, Math.floor(size / 220)));
      ctx.restore();
    }

    drawBrackets(ctx, rect, color, width) {
      const L = Math.max(4, Math.floor(rect.w / 3));
      const x0 = rect.left, y0 = rect.top, x1 = rect.right - 1, y1 = rect.bottom - 1;
      for (const [ax, ay, dx, dy] of [[x0, y0, 1, 1], [x1, y0, -1, 1], [x0, y1, 1, -1], [x1, y1, -1, -1]]) {
        draw.line(ctx, color, [ax, ay], [ax + dx * L, ay], width);
        draw.line(ctx, color, [ax, ay], [ax, ay + dy * L], width);
      }
    }

    // ----- Gefecht -------------------------------------------------------
    shakeOffset(now) {
      const k = (now - this.shakeT0) / 0.32;
      if (k < 0 || k >= 1 || this.paused) return [0, 0];
      const a = this.shakeAmp * (1 - k);
      return [Math.trunc(Math.sin(now * 90) * a), Math.trunc(Math.cos(now * 77) * a)];
    }

    drawBattle(ctx, now) {
      const [ox, oy] = this.shakeOffset(now);
      const bx = this.bx + ox, by = this.by + oy, sx = this.sx + ox, sy = this.sy + oy;
      const cb = this.cb, cs = this.cs;
      this.drawLabels(ctx, this.bx, this.by, cb);
      this.drawSea(ctx, 1, bx, by, cb, "target", now);
      this.drawSea(ctx, 0, sx, sy, cs, "own", now);

      if (this.state === PLAY && this.turn === 0 && !this.flight) {
        const [ar, ac] = this.aim;
        const rect = new PG.Rect(bx + ac * cb, by + ar * cb, cb, cb);
        const free = this.seas[1].shots[ar][ac] === UNKNOWN;
        const pul = 0.5 + 0.5 * Math.sin(now * 6);
        if (free) draw.rect(ctx, [COL_AIM[0], COL_AIM[1], COL_AIM[2], Math.floor(24 + 22 * pul)], rect);
        const inset = Math.max(2, Math.floor(cb / 10));
        this.drawBrackets(ctx, rect.inflate(-inset, -inset), free ? ui.mix(COL_AIM, [255, 255, 255], pul * 0.4) : ui.TEXT_FAINT, Math.max(2, Math.floor(cb / 16)));
      }
      if (this.state === PLAY && this.turn === 1) {
        const [px, py] = this.aiPos;
        const rect = new PG.Rect(Math.floor(sx + (px - 0.5) * cs), Math.floor(sy + (py - 0.5) * cs), cs, cs);
        this.drawBrackets(ctx, rect, COL_AI_AIM, Math.max(1, Math.floor(cs / 12)));
        draw.circle(ctx, COL_AI_AIM, [rect.centerx, rect.centery], Math.max(2, Math.floor(cs / 3)), 1);
      }
      if (this.flight) this.drawFlight(ctx);
      this.drawSunkBanners(ctx, now, ox, oy);

      this.fitText(ctx, this.tiny, t("bs.your_waters"), this.colW, this.sx + 5 * cs, this.captionY, ui.TEXT_DIM, "center");
      this.drawFleet(ctx, this.fleetRects[0], this.seas[1], true);
      this.drawFleet(ctx, this.fleetRects[1], this.seas[0], false);
      this.drawStatus(ctx);
    }

    drawFlight(ctx) {
      const f = this.flight;
      const side = 1 - f.shooter;
      const [x, y, cell] = this.boardGeom(side);
      const tx = x + f.c * cell + cell / 2, ty = y + f.r * cell + cell / 2;
      const [ox, oy, ocell] = this.boardGeom(f.shooter);
      const sx = ox + ocell * 5, sy = oy + ocell * 5;
      const k = Math.min(1, f.t / f.dur);
      const dist = Math.hypot(tx - sx, ty - sy);
      for (let j = 6; j >= 0; j--) {
        const kk = Math.max(0, k - j * 0.035);
        const e = kk * kk * (3 - 2 * kk) * 0.35 + kk * 0.65;
        const px = sx + (tx - sx) * e;
        const py = sy + (ty - sy) * e - Math.sin(Math.PI * kk) * dist * 0.28;
        if (j === 0) {
          const g = this.glow(cell * 0.35, [255, 200, 120]);
          this.blit(ctx, g, px - g.lw / 2, py - g.lh / 2);
        }
        draw.circle(ctx, ui.mix([255, 240, 190], [255, 120, 50], j / 6), [px, py], Math.max(2, Math.floor(cell * 0.16 * (1 - j / 8))));
      }
      draw.circle(ctx, side === 0 ? COL_AI_AIM : COL_AIM, [tx, ty], Math.max(3, Math.floor(cell * 0.42 * (1 - k * 0.6))), 1);
    }

    pname(p) {
      return p === 0 ? t("bs.you") : t("common.ai");
    }

    drawFleet(ctx, rect, sea, enemy) {
      const title = t(enemy ? "bs.fleet_enemy" : "bs.fleet_own");
      const headFont = this.fit(this.tiny, `${title}  ${sea.shipsLeft()}/${FLEET.length}`, rect.w);
      ui.text(ctx, headFont[1], rect.centerx, rect.top, headFont[0], enemy ? COL_P2 : COL_P1, "midtop");
      const y0 = rect.top + headFont[0].height + 4;
      const rowH = Math.max(6, Math.floor((rect.bottom - y0) / FLEET.length));
      const u = Math.max(4, Math.floor(Math.min((rect.w - 6) / 5, rowH * 0.78)));
      for (let idx = 0; idx < FLEET.length; idx++) {
        const ship = sea.shipByIdx(idx);
        const sw = FLEET[idx] * u;
        const xx = rect.centerx - Math.floor((5 * u) / 2);
        const yy = y0 + idx * rowH + Math.floor((rowH - u) / 2);
        const sunk = !!ship && ship.sunk;
        this.blit(ctx, this.shipSprite(idx, u, true, sunk ? "sunk" : "normal"), xx, yy, sunk ? 170 / 255 : 1);
        if (sunk) draw.line(ctx, COL_INVALID, [xx - 2, yy + Math.floor(u / 2)], [xx + sw + 2, yy + Math.floor(u / 2)], Math.max(2, Math.floor(u / 7)));
        else if (!enemy && ship) {
          for (const i of ship.hits) {
            const r = Math.floor(i / N), c = i % N;
            const off = ship.horiz ? c - ship.c : r - ship.r;
            draw.circle(ctx, COL_FLAME, [xx + Math.floor((off + 0.5) * u), yy + Math.floor(u / 2)], Math.max(2, Math.floor(u / 4)));
          }
        }
      }
    }

    drawStatus(ctx) {
      const y = this.statusY, x0 = this.bx, right = this.bx + 10 * this.cb;
      let pipW = 0;
      const human = this.state === PLAY && this.turn === 0;
      if (this.state === PLAY && (this.rules.salvo || this.shotsLeft > 1)) {
        const r = Math.max(3, Math.floor(this.small.height / 5));
        const n = Math.max(0, this.shotsLeft);
        pipW = n * (2 * r + 4);
        const col = human ? COL_AIM : COL_AI_AIM;
        for (let i = 0; i < n; i++) {
          const cx = right - r - i * (2 * r + 4);
          draw.circle(ctx, col, [cx, y], r);
          draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.5), [cx, y], r, 1);
        }
        const lbl = t("bs.shots_left", { n });
        if (right - pipW - 8 - this.tiny.width(lbl) > x0 + 120) {
          ui.text(ctx, lbl, right - pipW - 6, y, this.tiny, ui.TEXT_DIM, "midright");
          pipW += this.tiny.width(lbl) + 10;
        }
      }
      let text = "", col = ui.TEXT_DIM;
      if (this.msg) [text, col] = [this.msg, this.msgCol];
      else if (human) [text, col] = [t("bs.fire_hint"), ui.TEXT_FAINT];
      if (text) this.fitText(ctx, this.small, text, right - x0 - pipW - 8, x0, y, col, "midleft");
    }

    // ----- Aufstellen ----------------------------------------------------
    drawPlace(ctx, now) {
      const sea = this.seas[0];
      const x = this.bx, y = this.by, cell = this.cb;
      this.drawLabels(ctx, x, y, cell);
      this.blit(ctx, this.boardBase(cell, "own"), x, y);
      const occ = Array.from({ length: N }, () => new Array(N).fill(false));
      for (const ship of sea.ships) for (const [r, c] of ship.cells) occ[r][c] = true;
      this.drawWaves(ctx, x, y, cell, now, occ);
      if (!this.rules.touch) {
        // Sperrzone um liegende Schiffe sichtbar machen
        const seen = new Set();
        for (const ship of sea.ships) {
          for (const [r, c] of ship.cells) {
            for (let dr = -1; dr <= 1; dr++) {
              for (let dc = -1; dc <= 1; dc++) {
                const rr = r + dr, cc = c + dc;
                if (inBoard(rr, cc) && !occ[rr][cc] && !seen.has(rr * N + cc)) {
                  seen.add(rr * N + cc);
                  draw.rect(ctx, [255, 255, 255, 16], [x + cc * cell, y + rr * cell, cell, cell]);
                }
              }
            }
          }
        }
      }
      for (const ship of sea.ships) this.blit(ctx, this.shipSprite(ship.idx, cell, ship.horiz), x + ship.c * cell, y + ship.r * cell);

      const hd = this.held;
      if (hd) {
        const size = FLEET[hd.idx];
        const anchor = this.heldAnchor();
        const img = this.shipSprite(hd.idx, cell, hd.horiz);
        if (anchor) {
          const valid = this.heldValid();
          const tint = valid ? [COL_VALID[0], COL_VALID[1], COL_VALID[2], 70] : [COL_INVALID[0], COL_INVALID[1], COL_INVALID[2], 90];
          for (const [r, c] of cellsOf(anchor[0], anchor[1], size, hd.horiz)) if (inBoard(r, c)) draw.rect(ctx, tint, [x + c * cell, y + r * cell, cell, cell]);
          const boardRect = new PG.Rect(x, y, cell * N, cell * N);
          ctx.save();
          ctx.beginPath();
          ctx.rect(x, y, cell * N, cell * N);
          ctx.clip();
          this.blit(ctx, img, x + anchor[1] * cell, y + anchor[0] * cell, valid ? 235 / 255 : 150 / 255);
          ctx.restore();
          // Rahmen auf das Brett begrenzen (wie pygame Rect.clip)
          const rx0 = Math.max(boardRect.x, x + anchor[1] * cell), ry0 = Math.max(boardRect.y, y + anchor[0] * cell);
          const rx1 = Math.min(boardRect.right, x + (anchor[1] + (hd.horiz ? size : 1)) * cell);
          const ry1 = Math.min(boardRect.bottom, y + (anchor[0] + (hd.horiz ? 1 : size)) * cell);
          const rect = new PG.Rect(rx0, ry0, Math.max(0, rx1 - rx0), Math.max(0, ry1 - ry0));
          draw.rect(ctx, valid ? COL_VALID : COL_INVALID, rect, Math.max(2, Math.floor(cell / 14)), Math.max(2, Math.floor(cell / 6)));
        } else {
          const [mx, my] = this.mouse;
          const off = (hd.grab + 0.5) * cell;
          this.blit(ctx, img, mx - (hd.horiz ? off : cell / 2), my - (hd.horiz ? cell / 2 : off), 190 / 255);
        }
      } else if (this.kbMode) {
        const [r, c] = this.kcursor;
        this.drawBrackets(ctx, new PG.Rect(x + c * cell, y + r * cell, cell, cell), COL_AIM, Math.max(2, Math.floor(cell / 16)));
      }

      // Rechte Spalte: Dock + Knöpfe
      this.fitText(ctx, this.small, `${t("bs.fleet_own")}  ${sea.ships.length}/${FLEET.length}`, this.colW, this.colX + this.colW / 2, this.dockTitleY, COL_P1, "center");
      const dc = this.dockCell;
      for (let idx = 0; idx < this.dockRects.length; idx++) {
        const rc = this.dockRects[idx];
        const placed = !!sea.shipByIdx(idx);
        const holding = !!hd && hd.idx === idx;
        const hover = rc.collidepoint(this.mouse) && !hd;
        const inset = Math.max(2, Math.floor(rc.h / 8));
        const inner = new PG.Rect(rc.x, rc.y + Math.floor(inset / 2), rc.w, rc.h - inset);
        const rad = Math.max(4, Math.floor(inner.h / 4));
        draw.rect(ctx, hover || holding ? ui.BTN_SEL : ui.BTN, inner, 0, rad);
        draw.rect(ctx, holding ? this.accent : ui.BORDER, inner, holding ? 2 : 1, rad);
        this.blit(ctx, this.shipSprite(idx, dc, true), rc.x + 8, rc.centery - Math.floor(dc / 2), placed || holding ? 80 / 255 : 1);
        if (placed) {
          // Haken mitten auf der verblassten Silhouette
          const ck = Math.max(4, Math.min(Math.floor(rc.h / 4), Math.floor(dc / 2)));
          const cxk = rc.x + 8 + Math.floor((FLEET[idx] * dc) / 2), cyk = rc.centery;
          draw.lines(ctx, COL_VALID, false, [[cxk - ck, cyk], [cxk - Math.floor(ck / 3), cyk + Math.floor((ck * 2) / 3)], [cxk + ck, cyk - Math.floor((ck * 2) / 3)]], Math.max(2, Math.floor(ck / 3)));
        }
        const tx = rc.x + 14 + 5 * dc;
        const room = rc.right - tx - 8;
        let label = t("bs.ship." + SHIP_KEYS[idx]);
        if (this.tiny.width(label) > room) label = "×" + FLEET[idx];
        if (room > 16) this.fitText(ctx, this.tiny, label, room, tx, rc.centery, placed ? ui.TEXT_FAINT : ui.TEXT, "midleft");
      }
      const labels = { rotate: t("bs.btn_rotate"), random: t("bs.btn_random"), clear: t("bs.btn_clear"), ready: t("bs.btn_ready") };
      const keys = { rotate: "R", random: "X", clear: "C", ready: "Enter" };
      for (const key of BUTTONS) {
        const rc = this.btnRects[key];
        const hover = rc.collidepoint(this.mouse);
        let col;
        if (key === "ready") {
          const on = sea.complete();
          draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 10);
          draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 10);
          if (on && hover) draw.rect(ctx, [255, 255, 255], rc.inflate(-4, -4), 1, 9);
          col = on ? ui.TEXT : ui.TEXT_FAINT;
        } else {
          draw.rect(ctx, hover ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
          draw.rect(ctx, hover ? ui.BORDER_LIGHT : ui.BORDER, rc, 1, 8);
          col = hover ? ui.TEXT : ui.TEXT_DIM;
        }
        const room = rc.w - this.tiny.width(keys[key]) - 24;
        this.fitText(ctx, this.small, labels[key], room, rc.x + 8 + room / 2, rc.centery, col, "center");
        ui.text(ctx, keys[key], rc.right - 8, rc.centery, this.tiny, ui.TEXT_FAINT, "midright");
      }

      // Statuszeile
      let text, col;
      if (this.msg) [text, col] = [this.msg, this.msgCol];
      else if (hd) [text, col] = [t("bs.holding", { ship: t("bs.ship." + SHIP_KEYS[hd.idx]) }), ui.TEXT_DIM];
      else {
        const hover = this.dockRects.findIndex((rc) => rc.collidepoint(this.mouse));
        if (hover >= 0) [text, col] = [`${t("bs.ship." + SHIP_KEYS[hover])} (${FLEET[hover]})`, ui.TEXT_DIM];
        else [text, col] = [t("bs.place_hint"), ui.TEXT_FAINT];
      }
      this.fitText(ctx, this.small, text, this.colX + this.colW - x, x, this.statusY, col, "midleft");
    }

    // ----- HUD / Rundenende ----------------------------------------------
    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = Math.floor(this.hudH / 2);
      const left = `${this.pname(0)}: ${this.wins[0]}`, right = `${this.wins[1]} :${this.pname(1)}`;
      ui.text(ctx, left, 12, cy, this.small, COL_P1, "midleft");
      ui.text(ctx, right, this.width - 12, cy, this.small, COL_P2, "midright");
      let mid = "", col = ui.TEXT;
      if (this.state === PLACE) [mid, col] = [t("bs.place_title"), COL_P1];
      else if (this.state === PLAY) [mid, col] = [t(this.turn === 1 ? "bs.ai_thinks" : "bs.your_turn"), this.turn === 0 ? COL_P1 : COL_P2];
      if (mid) {
        const room = this.width - 2 * Math.max(this.small.width(left), this.small.width(right)) - 48;
        this.fitText(ctx, this.small, mid, room, this.width / 2, cy, col, "center");
      }
    }

    overRows() {
      const acc = (p) => Math.round((100 * this.stats[p].hits) / Math.max(1, this.stats[p].shots)) + "%";
      return [
        [t("bs.stat_shots"), String(this.stats[0].shots), String(this.stats[1].shots)],
        [t("bs.stat_hits"), String(this.stats[0].hits), String(this.stats[1].hits)],
        [t("bs.stat_acc"), acc(0), acc(1)],
        [t("bs.stat_turns"), String(this.stats[0].turns), String(this.stats[1].turns)],
        [t("bs.stat_left"), `${this.seas[0].shipsLeft()}/${FLEET.length}`, `${this.seas[1].shipsLeft()}/${FLEET.length}`],
      ];
    }

    overPanelRect() {
      const w = this.width, h = this.height;
      const rh = this.small.height + 4;
      const ph = 14 + this.huge.height + 10 + rh * 6 + 10 + this.tiny.height + 16;
      const pw = Math.min(w - 32, Math.max(340, Math.floor(w * 0.56)));
      const top = this.hudH + 6, bottom = h - 58;
      const py = top + Math.max(0, Math.floor((bottom - top - ph) / 2));
      return new PG.Rect(Math.floor((w - pw) / 2), py, pw, ph);
    }

    drawOver(ctx, now) {
      const k = smooth((now - this.overT0) / 0.35);
      const rect = this.overPanelRect().move(0, Math.floor((1 - k) * 18));
      // Bretter leicht abdunkeln, Panel deckend unterlegen (Glas-Themes)
      draw.rect(ctx, [4, 8, 16, Math.floor(110 * k)], [0, this.hudH + 1, this.width, this.height - this.hudH - 1]);
      draw.rect(ctx, ui.PANEL, rect, 0, 12);
      ui.drawPanel(ctx, rect, { accentTop: this.accent });
      const cx = rect.centerx;
      let y = rect.top + 14;
      const [head, hcol] = this.winner === 0 ? [t("bs.win_you"), ui.GOLD] : [t("bs.win_ai"), COL_P2];
      this.fitText(ctx, this.huge, head, rect.w - 24, cx, y, hcol, "midtop");
      y += this.huge.height + 10;
      const rh = this.small.height + 4;
      const c1 = rect.left + 18, c3 = rect.right - 18;
      const colw = Math.max(this.small.width(this.pname(0)), this.small.width(this.pname(1)), this.small.width("100%")) + 12;
      const c2 = c3 - colw;
      ui.text(ctx, this.pname(0), c2, y + rh / 2, this.small, COL_P1, "midright");
      ui.text(ctx, this.pname(1), c3, y + rh / 2, this.small, COL_P2, "midright");
      draw.line(ctx, ui.BORDER, [c1, y + rh], [c3, y + rh]);
      y += rh + 2;
      for (const [label, a, b] of this.overRows()) {
        this.fitText(ctx, this.small, label, c2 - colw - c1 - 6, c1, y + rh / 2, ui.TEXT_DIM, "midleft");
        ui.text(ctx, a, c2, y + rh / 2, this.small, ui.TEXT, "midright");
        ui.text(ctx, b, c3, y + rh / 2, this.small, ui.TEXT, "midright");
        y += rh;
      }
      y += 8;
      this.fitText(ctx, this.tiny, t("bs.new_round"), rect.w - 20, cx, y, ui.TEXT_DIM, "midtop");
    }

    // ----- Setup zeichnen ------------------------------------------------
    drawSetup(ctx) {
      const w = this.width, h = this.height, cx = w / 2;
      const titleY = Math.floor(h * 0.11);
      const [tf, tt] = this.fit(this.huge, PG.gameName(this.meta.id).toUpperCase(), w - 30);
      ui.text(ctx, tt, cx, titleY, tf, this.accent, "center");
      this.fitText(ctx, this.small, t("web.battleship.subtitle"), w - 30, cx, titleY + Math.floor(tf.height / 2) + Math.floor(this.small.height / 2) + 6, ui.TEXT_DIM, "center");
      const items = this.setupItems();
      const sel = items[Math.max(0, Math.min(items.length - 1, this.setupSel))];
      ui.text(ctx, t("bs.lbl_diff"), cx, this.diffLabelY, this.tiny, ui.TEXT_DIM, "midbottom");
      for (let i = 0; i < 3; i++) {
        const rc = this.diffRects[i];
        const on = i === this.diff;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 9);
        draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 9);
        if (sel === "diff" && on) draw.rect(ctx, ui.TEXT, rc.inflate(4, 4), 1, 11);
        this.fitText(ctx, this.small, t("bs.diff." + DIFFS[i]), rc.w - 10, rc.centerx, rc.centery, on ? ui.TEXT : ui.TEXT_DIM, "center");
      }
      ui.text(ctx, t("bs.lbl_rules"), cx, this.rulesLabelY, this.tiny, ui.TEXT_DIM, "midbottom");
      const onTxt = t("common.on"), offTxt = t("common.off");
      for (let i = 0; i < 3; i++) {
        const rc = this.ruleRects[i];
        const key = RULES[i];
        const on = this.rules[key];
        const focus = sel === key;
        draw.rect(ctx, focus ? ui.BTN_SEL : ui.BTN, rc, 0, 9);
        draw.rect(ctx, focus ? this.accent : ui.BORDER, rc, focus ? 2 : 1, 9);
        const isz = Math.max(10, Math.floor(rc.h * 0.5));
        this.drawRuleIcon(ctx, key, rc.x + 10 + Math.floor(isz / 2), rc.centery, isz);
        const ph = Math.max(16, Math.floor(rc.h * 0.62));
        const pw = Math.max(this.tiny.width(onTxt), this.tiny.width(offTxt)) + ph + 16;
        const pill = new PG.Rect(rc.right - 10 - pw, rc.centery - Math.floor(ph / 2), pw, ph);
        draw.rect(ctx, on ? ui.mix(ui.BTN, ui.GREEN, 0.75) : ui.PANEL, pill, 0, Math.floor(ph / 2));
        draw.rect(ctx, on ? ui.GREEN : ui.BORDER_LIGHT, pill, 1, Math.floor(ph / 2));
        const kx = on ? pill.right - Math.floor(ph / 2) : pill.left + Math.floor(ph / 2);
        draw.circle(ctx, on ? ui.TEXT : ui.TEXT_DIM, [kx, pill.centery], Math.floor(ph / 2) - 3);
        if (on) ui.text(ctx, onTxt, pill.left + 8, pill.centery, this.tiny, [20, 30, 24], "midleft");
        else ui.text(ctx, offTxt, pill.right - 8, pill.centery, this.tiny, ui.TEXT_DIM, "midright");
        const room = pill.left - (rc.x + 20 + isz) - 8;
        this.fitText(ctx, this.small, t(RULE_TEXT[key]), room, rc.x + 20 + isz, rc.centery, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
      }
      const focus = sel === "start";
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, focus ? ui.TEXT : this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      this.fitText(ctx, this.tiny, t("bs.setup_hint"), w - 20, cx, h - 13, ui.TEXT_FAINT, "center");
    }

    drawRuleIcon(ctx, key, cx, cy, size) {
      const col = ui.TEXT_DIM;
      const u = Math.max(3, Math.floor(size / 4));
      if (key === "touch") {
        // zwei Schiffe dicht an dicht
        draw.rect(ctx, col, [cx - Math.floor(size / 2), cy - u - 1, size, u], 0, Math.floor(u / 2));
        draw.rect(ctx, col, [cx - Math.floor(size / 2), cy + 1, size - u, u], 0, Math.floor(u / 2));
      } else if (key === "salvo") {
        for (const i of [-1, 0, 1]) draw.circle(ctx, col, [cx + i * (u + 2), cy + PG.mod(i, 2) * 2 - 1], Math.max(2, Math.floor(u / 2) + 1));
      } else {
        draw.circle(ctx, col, [cx, cy], Math.floor(size / 2), Math.max(1, Math.floor(size / 10)));
        draw.line(ctx, col, [cx - u, cy], [cx + u, cy], Math.max(1, Math.floor(size / 8)));
        draw.line(ctx, col, [cx, cy - u], [cx, cy + u], Math.max(1, Math.floor(size / 8)));
      }
    }
  }

  PG.register(BattleshipGame, {
    id: "BattleshipGame",
    key: "battleship",
    name: { default: "Battleship", de: "Schiffe versenken" },
    settingsKey: "battleship",
    defaults: { difficulty: 1, touch: true, salvo: false, extra_shot: false },
    wantsRightClick: true,
  });
  // Für Tests (Node/Smoke): Regeln + KI ohne Zeichenebene erreichbar
  PG.battleship = { Sea, ShotAI, FLEET, N };
})();
