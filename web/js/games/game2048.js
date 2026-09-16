/*
 * game2048.js - 2048, das Zahlen-Schiebespiel (Port von games/game2048.py)
 * =========================================================================
 * Setup-Screen: Brettgröße 3x3 bis 8x8, Modus Klassisch (Ziel 2048, danach
 * "Weiterspielen?") · Zeitangriff (3 Minuten, die Uhr startet mit dem ersten
 * Zug) · Endlos, Rückgängig aus / 3 pro Partie / unbegrenzt, Bestwerte je
 * Größe/Modus und "Fortsetzen", wenn eine gespeicherte Partie existiert.
 *
 * Spiel: Pfeile/WASD oder Wischen (Maus/Touch) schieben alle Kacheln; gehaltene
 * Tasten wiederholen nicht. Kacheln gleiten, verschmelzen mit Pop, neue wachsen
 * hinein; Eingaben während Animationen werden gepuffert. U/Rücktaste =
 * Rückgängig, R/N = neue Partie, Tab = Setup.
 *
 * Wer Rückgängig benutzt, spielt "unterstützt": kein Highscore, keine
 * Statistik-Siege, keine Kachel-Erfolge. Highscore "2048" nur aus 4x4
 * Klassisch ohne Rückgängig - sonst bleibt this.score 0; die Bestwerte stehen
 * in PG.store "mem.g2048" (gleiches Format wie die mem.json-Section "g2048").
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ----- Regeln --------------------------------------------------------------
  const SIZES = [3, 4, 5, 6, 7, 8];
  const MODE_KEYS = ["classic", "time", "endless"];
  const UNDO_KEYS = ["off", "limited", "unlimited"];
  const UNDO_LIMIT = 3;
  const TIME_LIMIT = 180.0;
  const WIN_TILE = 2048;
  const SPAWN_FOUR = 0.1;
  const HS_SIZE = 4, HS_MODE = "classic";
  const ACH_MAX_SIZE = 4;

  // ----- Speicher --------------------------------------------------------------
  const STORE_KEY = "mem.g2048"; // entspricht store.load_section("g2048")
  const STACK_MAX = 256;
  const SAVE_STACK = 3;
  const SAVE_EVERY = 2.5;
  const VALID_KEYS = new Set();
  for (const n of SIZES) for (const m of MODE_KEYS) VALID_KEYS.add(n + "-" + m);

  // ----- Animation (Sekunden) --------------------------------------------------
  const SLIDE_T = 0.12, SLIDE_FAST = 0.07, POP_T = 0.18, SPAWN_T = 0.16, NUDGE_T = 0.16;
  const OVER_DELAY = 0.6, FLASH_T = 1.3, MSG_T = 1.8, POPUP_T = 0.8;
  const QUEUE_MAX = 4, MAX_PARTICLES = 260;

  const SETUP = "setup", PLAY = "play", WIN = "win";

  const DIRS = {
    L: ["Left", "left", [-1, 0]],
    R: ["Right", "right", [1, 0]],
    U: ["Up", "up", [0, -1]],
    D: ["Down", "down", [0, 1]],
  };

  // ----- Identitätsfarben (fest, unabhängig vom Theme) --------------------------
  const COL_BOARD = [40, 44, 58];
  const COL_EMPTY = [55, 60, 78];
  const COL_TEXT_DARK = [104, 94, 84];
  const COL_TEXT_LIGHT = [250, 247, 242];
  const COL_TILE_BEYOND = [44, 46, 60];
  const COL_TEXT_BEYOND = [255, 214, 102];
  const TILE_COLORS = {
    2: [238, 228, 218], 4: [237, 224, 200], 8: [242, 177, 121], 16: [245, 149, 99],
    32: [246, 124, 95], 64: [246, 94, 59], 128: [237, 207, 114], 256: [237, 204, 97],
    512: [237, 200, 80], 1024: [237, 197, 63], 2048: [237, 194, 46],
    4096: [92, 196, 146], 8192: [52, 168, 196], 16384: [82, 122, 226],
    32768: [138, 96, 222], 65536: [190, 80, 196], 131072: [228, 70, 118],
  };
  const DIGIT_SCALE = { 1: 0.52, 2: 0.5, 3: 0.42, 4: 0.34, 5: 0.28, 6: 0.24 };

  // ===========================================================================
  //  Reine Spiellogik (1:1 wie in game2048.py)
  // ===========================================================================
  const emptyGrid = (n) => Array.from({ length: n }, () => new Array(n).fill(0));
  const copyGrid = (g) => g.map((row) => row.slice());

  /** Schiebt eine Linie zum Rand bei Index 0; jede Kachel verschmilzt höchstens einmal. */
  function slideLine(vals) {
    const out = new Array(vals.length).fill(0);
    const wege = [];
    let punkte = 0, pos = -1, frei = false;
    for (let i = 0; i < vals.length; i++) {
      const v = vals[i];
      if (!v) continue;
      if (frei && out[pos] === v) {
        out[pos] = v * 2;
        punkte += v * 2;
        frei = false;
        wege.push([i, pos, true]);
      } else {
        pos++;
        out[pos] = v;
        frei = true;
        wege.push([i, pos, false]);
      }
    }
    return [out, punkte, wege];
  }

  function lineCells(n, dir, k) {
    const out = [];
    if (dir === "L") for (let c = 0; c < n; c++) out.push([k, c]);
    else if (dir === "R") for (let c = n - 1; c >= 0; c--) out.push([k, c]);
    else if (dir === "U") for (let r = 0; r < n; r++) out.push([r, k]);
    else for (let r = n - 1; r >= 0; r--) out.push([r, k]);
    return out;
  }

  function planMove(grid, dir) {
    const n = grid.length;
    const next = emptyGrid(n);
    const slides = [], merges = [];
    let gained = 0, moved = false;
    for (let k = 0; k < n; k++) {
      const cells = lineCells(n, dir, k);
      const vals = cells.map(([r, c]) => grid[r][c]);
      const [out, pts, wege] = slideLine(vals);
      gained += pts;
      cells.forEach(([r, c], i) => {
        next[r][c] = out[i];
        if (out[i] !== grid[r][c]) moved = true;
      });
      for (const [von, nach, merged] of wege) {
        const [fr, fc] = cells[von];
        const [tr, tc] = cells[nach];
        slides.push([fr, fc, tr, tc, vals[von]]);
        if (merged) merges.push([tr, tc, out[nach]]);
      }
    }
    return { grid: next, gained, slides, merges, moved };
  }

  function spawnTile(grid) {
    const free = [];
    grid.forEach((row, r) => row.forEach((v, c) => { if (!v) free.push([r, c]); }));
    if (!free.length) return null;
    const [r, c] = PG.rand.choice(free);
    const v = PG.rand.random() < SPAWN_FOUR ? 4 : 2;
    grid[r][c] = v;
    return [r, c, v];
  }

  function hasMoves(grid) {
    const n = grid.length;
    for (let r = 0; r < n; r++) {
      for (let c = 0; c < n; c++) {
        const v = grid[r][c];
        if (!v) return true;
        if (c + 1 < n && v === grid[r][c + 1]) return true;
        if (r + 1 < n && v === grid[r + 1][c]) return true;
      }
    }
    return false;
  }

  const maxTile = (grid) => grid.reduce((m, row) => Math.max(m, ...row), 0);

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);

  function validGrid(grid, n) {
    if (!Array.isArray(grid) || grid.length !== n) return false;
    for (const row of grid) {
      if (!Array.isArray(row) || row.length !== n) return false;
      for (const v of row) {
        if (!isInt(v)) return false;
        if (v && (v < 2 || v > 2 ** 40 || !Number.isInteger(Math.log2(v)))) return false;
      }
    }
    return true;
  }

  const clampInt = (v, lo, hi, def) => (isInt(v) ? Math.max(lo, Math.min(hi, v)) : def);
  const clampNum = (v, lo, hi, def) => (typeof v === "number" && isFinite(v) ? Math.max(lo, Math.min(hi, v)) : def);

  function cleanSlides(raw, n) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((sl) => Array.isArray(sl) && sl.length === 5 && sl.every(isInt) && sl.slice(0, 4).every((x) => x >= 0 && x < n) && sl[4] >= 2).map((sl) => sl.slice());
  }

  function cleanSpawn(raw, n) {
    if (Array.isArray(raw) && raw.length === 3 && raw.every(isInt) && raw[0] >= 0 && raw[0] < n && raw[1] >= 0 && raw[1] < n && (raw[2] === 2 || raw[2] === 4)) return raw.slice();
    return null;
  }

  function cleanSave(sv, n) {
    if (!sv || typeof sv !== "object" || !validGrid(sv.grid, n)) return null;
    const out = {
      grid: copyGrid(sv.grid),
      points: clampInt(sv.points, 0, 1e12, 0),
      moves: clampInt(sv.moves, 0, 1e9, 0),
      undo_used: clampInt(sv.undo_used, 0, 1e9, 0),
      assisted: sv.assisted === true,
      reached: sv.reached === true,
      continued: sv.continued === true,
      peak: clampInt(sv.peak, 0, 2 ** 40, 0),
      time_left: clampNum(sv.time_left, 0, TIME_LIMIT, TIME_LIMIT),
      clock_on: sv.clock_on === true,
      elapsed: clampNum(sv.elapsed, 0, 1e7, 0),
    };
    out.peak = Math.max(out.peak, maxTile(out.grid));
    const stack = [];
    for (const e of (Array.isArray(sv.stack) ? sv.stack : []).slice(-SAVE_STACK)) {
      if (e && typeof e === "object" && validGrid(e.grid, n)) {
        stack.push({
          grid: copyGrid(e.grid),
          points: clampInt(e.points, 0, 1e12, 0),
          moves: clampInt(e.moves, 0, 1e9, 0),
          reached: e.reached === true,
          slides: cleanSlides(e.slides, n),
          spawn: cleanSpawn(e.spawn, n),
        });
      }
    }
    out.stack = stack;
    return out;
  }

  const tileColor = (v) => TILE_COLORS[v] || COL_TILE_BEYOND;
  const tileTextColor = (v) => (v <= 4 ? COL_TEXT_DARK : TILE_COLORS[v] ? COL_TEXT_LIGHT : COL_TEXT_BEYOND);
  function fmtTime(sec) {
    sec = Math.max(0, Math.ceil(sec - 1e-6));
    return Math.floor(sec / 60) + ":" + String(sec % 60).padStart(2, "0");
  }
  const easeOutCubic = (p) => 1 - (1 - p) ** 3;
  function easeOutBack(p) {
    const c1 = 1.70158, q = p - 1;
    return 1 + (c1 + 1) * q ** 3 + c1 * q ** 2;
  }
  const shade = (c, f) => [Math.floor(c[0] * f), Math.floor(c[1] * f), Math.floor(c[2] * f)];

  // ===========================================================================
  //  Das Spiel
  // ===========================================================================
  class Game2048 extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const o = this.opts;
      this.size = SIZES.includes(o.size) ? o.size : 4;
      this.gmode = MODE_KEYS.includes(o.mode) ? o.mode : "classic";
      this.undoRule = UNDO_KEYS.includes(o.undo) ? o.undo : "limited";

      this.data = this.loadData();
      this.animT = 0;
      this.hover = null;
      this.drag = null;
      this.fitCache = new Map();
      this.tileCache = new Map();
      this.boardKey = null;
      this.boardCache = null;
      this.setupFocus = 0;
      this.setupAct = 0;
      this.blankGame();
      this.makeFonts();
      this.layout();
      this.buildSetupLayout();
      this.state = SETUP;
    }

    blankGame() {
      this.grid = emptyGrid(this.size);
      this.points = 0;
      this.moves = 0;
      this.undoUsed = 0;
      this.assisted = false;
      this.reached = false;
      this.continued = false;
      this.peak = 0;
      this.maxTile = 0;
      this.timeLeft = TIME_LIMIT;
      this.clockOn = false;
      this.elapsed = 0;
      this.stack = [];
      this.queue = [];
      this.slide = null;
      this.pops = new Map();
      this.spawns = new Map();
      this.particles = [];
      this.rings = [];
      this.popups = [];
      this.flash = null;
      this.msg = null;
      this.nudge = null;
      this.overT = null;
      this.overReason = "stuck";
      this.newBest = false;
      this.winPending = false;
      this.panelT0 = -1e9;
      this.bestStart = { best: 0, best_assisted: 0 };
      this.dirty = false;
      this.saveCd = SAVE_EVERY;
      this.off = [0, 0];
    }

    makeFonts() {
      const h = this.height;
      this.tiny = ui.font(Math.max(11, Math.floor(h / 40)));
      this.small = ui.font(Math.max(13, Math.floor(h / 32)));
      this.labPx = Math.max(10, Math.floor(h / 44));
      this.bigPx = Math.max(20, Math.floor(h / 15));
      this.midPx = Math.max(15, Math.floor(h / 24));
      this.rowPx = Math.max(12, Math.floor(h / 34));
      this.titlePx = Math.max(24, Math.floor(h / 13));
      this.fLabel = ui.font(this.labPx);
      this.fBig = ui.font(this.bigPx, true);
      this.fMid = ui.font(this.midPx, true);
      this.fRow = ui.font(this.rowPx);
      this.fRowB = ui.font(this.rowPx, true);
      this.fTitle = ui.font(this.titlePx, true);
      this.fSetupTitle = ui.font(Math.max(28, Math.floor(h / 12)), true);
    }

    // ----- Hilfen ----------------------------------------------------------
    key() {
      return this.size + "-" + this.gmode;
    }
    counts() {
      return this.size === HS_SIZE && this.gmode === HS_MODE && !this.assisted;
    }
    get showHighscoreBanner() {
      return this.counts();
    }
    syncScore() {
      this.score = this.counts() ? this.points : 0;
    }
    undoLeft() {
      return this.undoRule === "limited" ? Math.max(0, UNDO_LIMIT - this.undoUsed) : 0;
    }
    canUndo() {
      if (this.undoRule === "off" || !this.stack.length) return false;
      if (this.gameOver && this.overReason === "time") return false; // abgelaufene Zeit holt kein Undo zurück
      return this.undoRule === "unlimited" || this.undoLeft() > 0;
    }
    free(key) {
      return [key, key.toLowerCase(), key.toUpperCase()].every((k) => this.keyIsFree(k));
    }
    toneFor(value) {
      const step = Math.max(0, Math.floor(Math.log2(Math.max(2, value))) - 6);
      this.tone(330 * 2 ** ((step * 2) / 12), 0.11, "triangle", 0.22);
    }

    // ===================================================== Persistenz
    loadData() {
      const raw = PG.store.get(STORE_KEY, {}) || {};
      const data = { best: {}, best_assisted: {}, best_tile: {}, top_tile: 0, saves: {} };
      for (const name of ["best", "best_assisted", "best_tile"]) {
        const src = raw[name];
        if (src && typeof src === "object") {
          for (const [k, v] of Object.entries(src)) {
            if (VALID_KEYS.has(k) && clampInt(v, 1, 1e12, 0) > 0) data[name][k] = v;
          }
        }
      }
      data.top_tile = clampInt(raw.top_tile, 0, 2 ** 40, 0);
      if (raw.saves && typeof raw.saves === "object") {
        for (const [k, sv] of Object.entries(raw.saves)) {
          if (!VALID_KEYS.has(k)) continue;
          const clean = cleanSave(sv, Number(k.split("-")[0]));
          if (clean) data.saves[k] = clean;
        }
      }
      return data;
    }
    persist() {
      PG.store.set(STORE_KEY, this.data);
    }
    snapshot() {
      return {
        grid: copyGrid(this.grid), points: this.points, moves: this.moves,
        undo_used: this.undoUsed, assisted: this.assisted, reached: this.reached,
        continued: this.continued, peak: this.peak,
        time_left: Math.round(this.timeLeft * 100) / 100, clock_on: this.clockOn,
        elapsed: Math.round(this.elapsed * 10) / 10,
        stack: this.stack.slice(-SAVE_STACK).map((e) => ({
          grid: copyGrid(e.grid), points: e.points, moves: e.moves, reached: e.reached,
          slides: e.slides.map((s) => s.slice()), spawn: e.spawn ? e.spawn.slice() : null,
        })),
      };
    }
    storeGame() {
      if ((this.state === PLAY || this.state === WIN) && !this.gameOver && this.moves > 0) {
        this.data.saves[this.key()] = this.snapshot();
      }
    }
    noteBest() {
      const key = this.key();
      const book = this.data[this.assisted ? "best_assisted" : "best"];
      if (this.points > (book[key] || 0)) book[key] = this.points;
      if (this.maxTile > (this.data.best_tile[key] || 0)) this.data.best_tile[key] = this.maxTile;
      if (this.maxTile > this.data.top_tile) this.data.top_tile = this.maxTile;
    }
    /** Zählende Partie wird ohne Game Over verworfen -> Punktestand trotzdem sichern. */
    bankHighscore() {
      if (this.score > 0) PG.highscore.update(this.highscoreKey, this.score);
    }
    destroy() {
      this.storeGame();
      this.persist();
      this.dirty = false;
    }

    // ===================================================== Partie starten
    start(resume) {
      const key = this.key();
      const sv = resume ? this.data.saves[key] : null;
      if (!sv && !this.gameOver) this.bankHighscore();
      this.blankGame();
      this.layout();
      this.newRoundResult();
      this.gameOver = false;
      this.state = PLAY;
      if (sv) {
        this.grid = copyGrid(sv.grid);
        this.points = sv.points;
        this.moves = sv.moves;
        this.undoUsed = sv.undo_used;
        this.assisted = sv.assisted;
        this.reached = sv.reached;
        this.continued = sv.continued;
        this.peak = sv.peak;
        this.timeLeft = sv.time_left;
        this.clockOn = sv.clock_on;
        this.elapsed = sv.elapsed;
        this.stack = this.undoRule === "off" ? [] : sv.stack.map((e) => Object.assign({}, e));
        for (let r = 0; r < this.size; r++) {
          for (let c = 0; c < this.size; c++) if (this.grid[r][c]) this.spawns.set(r * 16 + c, -0.02 * (r + c));
        }
      } else {
        if (this.data.saves[key]) {
          delete this.data.saves[key];
          this.persist();
        }
        for (let i = 0; i < 2; i++) {
          const sp = spawnTile(this.grid);
          if (sp) this.spawns.set(sp[0] * 16 + sp[1], 0);
        }
      }
      this.maxTile = maxTile(this.grid);
      this.peak = Math.max(this.peak, this.maxTile);
      this.bestStart = { best: this.data.best[key] || 0, best_assisted: this.data.best_assisted[key] || 0 };
      this.syncScore();
      if (this.reached && this.gmode === "classic" && !this.continued) {
        this.state = WIN;
        this.panelT0 = ui.now();
      } else if (this.gmode === "time" && !this.clockOn) {
        this.message(t("g2048.clock_hint"));
      }
      this.playSound("click");
    }

    toSetup() {
      this.storeGame();
      this.persist();
      this.dirty = false;
      this.queue = [];
      this.slide = null;
      this.gameOver = false;
      this.setupAct = 0;
      this.state = SETUP;
      this.playSound("click");
    }

    // ===================================================== Layout (Spiel)
    layout() {
      const w = this.width, h = this.height, n = this.size;
      const m = Math.max(10, Math.floor(h / 40));
      const sideW = Math.floor(Math.max(150, Math.min(340, w * 0.27)));
      const bar = this.gmode === "time" ? Math.max(6, Math.floor(h / 64)) : 0;
      const barGap = bar ? Math.max(5, Math.floor(h / 90)) : 0;
      const areaW = w - sideW - 3 * m;
      const areaH = h - 2 * m - bar - barGap;
      let board = Math.max(60, Math.min(areaW, areaH));
      const gap = Math.max(3, Math.floor((board * (n <= 4 ? 0.13 : 0.1)) / (n + 1)));
      const cell = Math.max(8, Math.floor((board - gap * (n + 1)) / n));
      board = cell * n + gap * (n + 1);
      const bx = m + Math.floor((areaW - board) / 2);
      const by = m + Math.floor((areaH - board) / 2);
      this.gap = gap;
      this.cell = cell;
      this.boardRect = new PG.Rect(bx, by, board, board);
      this.timeRect = new PG.Rect(bx, by + board + barGap, board, bar);
      this.sideRect = new PG.Rect(w - m - sideW, by, sideW, board + bar + barGap);
      this.tileCache.clear();
      this.layoutSide();
    }

    layoutSide() {
      const sd = this.sideRect, h = this.height;
      const pad = Math.max(6, Math.floor(h / 64));
      const gap = Math.max(5, Math.floor(h / 90));
      this.pad = pad;
      const lh = this.fLabel.height;
      let y = sd.y;
      this.tagRect = new PG.Rect(sd.x, y, sd.w, lh);
      y += lh + gap;
      let ch = lh + this.fBig.height + 2 * pad + 2;
      this.scoreRect = new PG.Rect(sd.x, y, sd.w, ch);
      y += ch + gap;
      ch = lh + this.fMid.height + 2 * pad + 2;
      this.bestRect = new PG.Rect(sd.x, y, sd.w, ch);
      y += ch + gap;
      this.rowH = this.fRow.height + Math.max(4, Math.floor(h / 90));
      this.infoRect = new PG.Rect(sd.x, y, sd.w, 4 * this.rowH + 2 * pad);
      y = this.infoRect.bottom + gap;
      const bh = Math.max(26, Math.floor(h / 17));
      const stackedH = 3 * bh + 2 * gap;
      if (sd.bottom - y >= stackedH) {
        const top = sd.bottom - stackedH;
        this.btnRects = [0, 1, 2].map((i) => new PG.Rect(sd.x, top + i * (bh + gap), sd.w, bh));
        this.btnStacked = true;
      } else {
        const bw = Math.floor((sd.w - 2 * gap) / 3);
        this.btnRects = [0, 1, 2].map((i) => new PG.Rect(sd.x + i * (bw + gap), sd.bottom - bh, bw, bh));
        this.btnStacked = false;
      }
      this.hintArea = new PG.Rect(sd.x, y, sd.w, Math.max(0, this.btnRects[0].top - gap - y));
    }

    cellPos(r, c) {
      const br = this.boardRect;
      return [br.x + this.off[0] + this.gap + c * (this.cell + this.gap), br.y + this.off[1] + this.gap + r * (this.cell + this.gap)];
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const w = this.width, h = this.height, cx = Math.floor(w / 2);
      const th = this.fSetupTitle.height, sh = this.small.height, lh = this.tiny.height;
      this.titleY = Math.max(30, Math.floor(h * 0.085));
      const bottom = h - 36 - Math.max(4, Math.floor(h / 90));
      const g = Math.max(5, Math.floor(h / 72));
      const bw = Math.min(620, w - 40);
      const infoPad = Math.max(5, Math.floor(h / 90));
      const actMin = sh + lh + 18; // Platz für Unterzeile + v4.2-Linie
      let top, infoH, avail, rowH, actH, used, withSub;
      for (withSub of [true, false]) {
        if (withSub) top = this.titleY + Math.max(Math.floor(th / 2) + 36, 42) + Math.floor(sh / 2) + Math.max(10, Math.floor(h / 40));
        else top = this.titleY + Math.floor(th / 2) + 14 + Math.max(6, Math.floor(h / 60));
        infoH = 3 * lh + 2 * infoPad + 4;
        avail = bottom - top;
        const fixed = 3 * (lh + 3) + 4 * g + infoH;
        rowH = Math.floor((avail - fixed - actMin) / 3);
        rowH = Math.max(22, Math.min(rowH, Math.max(36, Math.floor(h / 15))));
        actH = Math.max(actMin, rowH + 6);
        used = fixed + 3 * rowH + actH;
        if (used <= avail) break;
      }
      this.setupSub = withSub;
      const extra = Math.max(0, avail - used);
      const vg = g + Math.min(Math.floor(extra / 8), Math.floor(h / 28));
      used += 4 * (vg - g);
      let y = top + Math.max(0, Math.floor((avail - used) / 2));
      const row = (yy, n) => {
        const cw = (bw - g * (n - 1)) / n;
        const out = [];
        for (let i = 0; i < n; i++) out.push(new PG.Rect(Math.floor(cx - bw / 2 + i * (cw + g)), yy, Math.floor(cw), rowH));
        return out;
      };
      this.sizeRects = row(y + lh + 3, SIZES.length);
      y += lh + 3 + rowH + vg;
      this.modeRects = row(y + lh + 3, MODE_KEYS.length);
      y += lh + 3 + rowH + vg;
      this.undoRects = row(y + lh + 3, UNDO_KEYS.length);
      y += lh + 3 + rowH + vg;
      this.infoRectSetup = new PG.Rect(cx - Math.floor(bw / 2), y, bw, infoH);
      this.infoPad = infoPad;
      y += infoH + vg;
      const half = Math.floor((bw - g) / 2);
      this.contRect = new PG.Rect(cx - Math.floor(bw / 2), y, half, actH);
      this.newRect = new PG.Rect(cx - Math.floor(bw / 2) + half + g, y, half, actH);
      const sw = Math.min(bw, Math.max(200, Math.floor(w / 3)));
      this.startRect = new PG.Rect(cx - Math.floor(sw / 2), y, sw, actH);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }
    saved() {
      return this.data.saves[this.key()] || null;
    }
    setSize(n) {
      if (n === this.size) return;
      this.size = n;
      this.saveSetting("size", n);
      this.setupAct = 0;
      this.layout();
      this.playSound("click");
    }
    setMode(m) {
      if (m === this.gmode) return;
      this.gmode = m;
      this.saveSetting("mode", m);
      this.setupAct = 0;
      this.layout();
      this.playSound("click");
    }
    setUndo(u) {
      if (u === this.undoRule) return;
      this.undoRule = u;
      this.saveSetting("undo", u);
      this.playSound("select");
    }
    setupStep(step) {
      const wrap = (i, n) => ((i % n) + n) % n;
      if (this.setupFocus === 0) this.setSize(SIZES[wrap(SIZES.indexOf(this.size) + step, SIZES.length)]);
      else if (this.setupFocus === 1) this.setMode(MODE_KEYS[wrap(MODE_KEYS.indexOf(this.gmode) + step, MODE_KEYS.length)]);
      else if (this.setupFocus === 2) this.setUndo(UNDO_KEYS[wrap(UNDO_KEYS.indexOf(this.undoRule) + step, UNDO_KEYS.length)]);
      else if (this.saved()) {
        this.setupAct = 1 - this.setupAct;
        this.playSound("move");
      }
    }

    handleSetup(ev) {
      if (ev.kind === "mousemove") {
        this.hover = ev.pos;
        return;
      }
      if (ev.kind === "mousedown" && ev.button === 1) {
        const p = ev.pos;
        const groups = [[this.sizeRects, 0, SIZES, (v) => this.setSize(v)], [this.modeRects, 1, MODE_KEYS, (v) => this.setMode(v)], [this.undoRects, 2, UNDO_KEYS, (v) => this.setUndo(v)]];
        for (const [rects, focus, keys, setter] of groups) {
          for (let i = 0; i < rects.length; i++) {
            if (rects[i].collidepoint(p)) {
              this.setupFocus = focus;
              setter(keys[i]);
              return;
            }
          }
        }
        if (this.saved()) {
          if (this.contRect.collidepoint(p)) {
            this.setupFocus = 3;
            this.setupAct = 0;
            this.start(true);
          } else if (this.newRect.collidepoint(p)) {
            this.setupFocus = 3;
            this.setupAct = 1;
            this.start(false);
          }
        } else if (this.startRect.collidepoint(p)) {
          this.start(false);
        }
        return;
      }
      if (ev.kind !== "keydown") return;
      const k = ev.key;
      if (["3", "4", "5", "6", "7", "8"].includes(k)) {
        this.setupFocus = 0;
        this.setSize(Number(k));
      } else if (k === "Up" || k === "w" || k === "W") {
        this.setupFocus = (this.setupFocus + 3) % 4;
        this.playSound("move");
      } else if (k === "Down" || k === "s" || k === "S") {
        this.setupFocus = (this.setupFocus + 1) % 4;
        this.playSound("move");
      } else if (k === "Left" || k === "a" || k === "A") this.setupStep(-1);
      else if (k === "Right" || k === "d" || k === "D") this.setupStep(1);
      else if (k === "m" || k === "M") {
        this.setupFocus = 1;
        this.setupStep(1);
      } else if (k === "u" || k === "U") {
        this.setupFocus = 2;
        this.setupStep(1);
      } else if (k === "Return" || k === "space" || k === "KP_Enter") {
        this.start(!!this.saved() && this.setupAct === 0);
      }
    }

    // ===================================================== Eingabe (Spiel)
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (ev.kind === "mousemove") {
        this.hover = ev.pos;
        return;
      }
      if (ev.kind === "mousedown") {
        if (ev.button !== 1) return;
        this.drag = null;
        if (this.state === WIN || this.gameOver) {
          this.panelClick(ev.pos);
          return;
        }
        const acts = ["undo", "new", "setup"];
        for (let i = 0; i < 3; i++) {
          if (this.btnRects[i].collidepoint(ev.pos)) {
            this.button(acts[i]);
            return;
          }
        }
        this.drag = ev.pos;
        return;
      }
      if (ev.kind === "mouseup") {
        const start = this.drag;
        this.drag = null;
        if (!start || ev.button !== 1 || !ev.pos) return;
        const d = this.swipeDir(start, ev.pos);
        if (d) this.input(d);
        return;
      }
      if (ev.kind !== "keydown" || ev.repeat) return;

      const key = ev.key;
      const undoKey = ["u", "U", "BackSpace"].includes(key) && this.free(key);
      const newKey = ["r", "R", "n", "N"].includes(key) && this.free(key);
      const setupKey = key === "Tab" || ((key === "s" || key === "S") && this.free(key));
      const enter = key === "Return" || key === "space" || key === "KP_Enter";

      if (this.gameOver) {
        if (enter || newKey) this.start(false);
        else if (undoKey) this.input("undo");
        else if (setupKey) this.toSetup();
        return;
      }
      if (this.state === WIN) {
        if (enter) this.panelAction("continue");
        else if (newKey) this.start(false);
        else if (undoKey) this.input("undo");
        else if (setupKey) this.toSetup();
        return;
      }
      for (const [d, [arrow, action]] of Object.entries(DIRS)) {
        if (key === arrow || this.isAction(key, action)) {
          this.input(d);
          return;
        }
      }
      if (undoKey) this.input("undo");
      else if (newKey) this.start(false);
      else if (setupKey) this.toSetup();
    }

    swipeDir(a, b) {
      const dx = b[0] - a[0], dy = b[1] - a[1];
      const need = Math.max(18, this.cell * 0.3);
      if (Math.max(Math.abs(dx), Math.abs(dy)) < need) return null;
      if (Math.abs(dx) > Math.abs(dy) * 1.2) return dx > 0 ? "R" : "L";
      if (Math.abs(dy) > Math.abs(dx) * 1.2) return dy > 0 ? "D" : "U";
      return null;
    }

    button(act) {
      if (act === "undo") this.input("undo");
      else if (act === "new") this.start(false);
      else this.toSetup();
    }

    input(what) {
      if (what === "undo") {
        if (this.slide) {
          if (this.queue.length < QUEUE_MAX) this.queue.push(what);
          return;
        }
        this.undo();
        return;
      }
      if (this.state !== PLAY || this.gameOver || this.overT !== null) return;
      if (this.slide) {
        if (this.queue.length < QUEUE_MAX) this.queue.push(what);
        return;
      }
      this.doMove(what);
    }

    // ===================================================== Züge
    doMove(d) {
      const plan = planMove(this.grid, d);
      if (!plan.moved) {
        const vec = DIRS[d][2];
        this.nudge = [vec[0], vec[1], 0];
        return false;
      }
      const entry = { grid: copyGrid(this.grid), points: this.points, moves: this.moves, reached: this.reached, slides: plan.slides, spawn: null };
      this.grid = plan.grid;
      this.points += plan.gained;
      this.moves++;
      const spawn = spawnTile(this.grid);
      entry.spawn = spawn;
      if (this.undoRule !== "off") {
        this.stack.push(entry);
        const cap = this.undoRule === "unlimited" ? STACK_MAX : UNDO_LIMIT;
        if (this.stack.length > cap) this.stack.shift();
      }
      if (this.gmode === "time") this.clockOn = true;
      this.pops.clear();
      this.spawns.clear();
      this.slide = { t: 0, dur: this.queue.length ? SLIDE_FAST : SLIDE_T, slides: plan.slides, merges: plan.merges, spawn, vanish: null, undo: false, flash: null };
      this.afterMove(plan);
      return true;
    }

    afterMove(plan) {
      if (plan.gained) {
        this.popups.push([plan.gained, 0]);
        if (this.popups.length > 3) this.popups.splice(0, this.popups.length - 3);
      }
      const top = plan.merges.reduce((m, x) => Math.max(m, x[2]), 0);
      if (top) {
        this.playSound("merge");
        if (top >= 128) this.toneFor(top);
      } else this.playSound("move");

      this.maxTile = maxTile(this.grid);
      const newPeak = this.maxTile > this.peak;
      this.peak = Math.max(this.peak, this.maxTile);

      if (!this.reached && this.maxTile >= WIN_TILE) {
        this.reached = true;
        if (!this.assisted) {
          if (this.gmode !== "time") this.reportResult(true);
          if (this.size <= ACH_MAX_SIZE) this.achEvent("tile_2048");
        }
        if (this.gmode === "classic" && !this.continued) this.winPending = true;
      }
      if (this.maxTile >= 4096 && !this.assisted && this.size <= ACH_MAX_SIZE) this.achEvent("tile_4096");
      if (newPeak && this.maxTile >= WIN_TILE && !this.winPending) this.slide.flash = this.maxTile;

      this.noteBest();
      this.syncScore();
      this.dirty = true;
      if (!hasMoves(this.grid)) {
        this.overT = OVER_DELAY;
        this.queue = [];
      }
    }

    undo() {
      if (this.undoRule === "off") {
        this.message(t("g2048.undo_disabled"));
        this.playSound("hit");
        return false;
      }
      if (!this.stack.length) {
        this.playSound("hit");
        return false;
      }
      if (this.undoRule === "limited" && this.undoLeft() <= 0) {
        this.message(t("g2048.undo_empty"));
        this.playSound("hit");
        return false;
      }
      if (this.gameOver && this.overReason === "time") return false;
      const e = this.stack.pop();
      this.undoUsed++;
      this.assisted = true;
      this.grid = copyGrid(e.grid);
      this.points = e.points;
      this.moves = e.moves;
      this.reached = e.reached;
      this.maxTile = maxTile(this.grid);
      if (this.gameOver) this.resumeFromGameOver(); // dieselbe Partie, kein Neustart
      this.overT = null;
      this.winPending = false;
      this.state = PLAY;
      this.pops.clear();
      this.spawns.clear();
      if (e.slides.length) {
        const rev = e.slides.map(([fr, fc, tr, tc, v]) => [tr, tc, fr, fc, v]);
        this.slide = { t: 0, dur: SLIDE_T, slides: rev, merges: [], spawn: null, vanish: e.spawn, undo: true, flash: null };
      } else {
        for (let r = 0; r < this.size; r++) for (let c = 0; c < this.size; c++) if (this.grid[r][c]) this.spawns.set(r * 16 + c, 0);
      }
      this.noteBest();
      this.syncScore();
      this.dirty = true;
      this.playSound("select");
      return true;
    }

    endSlide() {
      const sl = this.slide;
      this.slide = null;
      if (!sl.undo) {
        for (const [r, c, v] of sl.merges) {
          this.pops.set(r * 16 + c, 0);
          if (v >= 128) {
            const [x, y] = this.cellPos(r, c);
            const cx = x + this.cell / 2, cy = y + this.cell / 2;
            this.burst(cx, cy, tileColor(v), Math.min(22, 4 + Math.floor(Math.log2(v))), this.cell * 2.6);
            if (v >= WIN_TILE) this.rings.push([cx, cy, 0, tileColor(v)]);
          }
        }
        if (sl.spawn) this.spawns.set(sl.spawn[0] * 16 + sl.spawn[1], 0);
        if (sl.flash) {
          this.flash = [String(sl.flash), 0, tileColor(sl.flash)];
          this.playSound("powerup");
        }
      }
      if (this.winPending) {
        this.winPending = false;
        this.state = WIN;
        this.queue = [];
        this.panelT0 = ui.now();
        const br = this.boardRect;
        for (let i = 0; i < 3; i++) {
          this.burst(br.centerx + PG.rand.uniform(-br.w * 0.3, br.w * 0.3), br.centery + PG.rand.uniform(-br.h * 0.3, br.h * 0.3), tileColor(WIN_TILE), 18, this.cell * 3.2);
        }
        this.playSound("win");
        this.rumble(180);
      }
    }

    finish(timeout) {
      this.overT = null;
      this.queue = [];
      this.slide = null;
      this.pops.clear();
      this.spawns.clear();
      this.overReason = timeout ? "time" : "stuck";
      this.state = PLAY;
      this.winPending = false;
      this.noteBest();
      const book = this.assisted ? "best_assisted" : "best";
      this.newBest = this.points > Math.max(0, this.bestStart[book] || 0);
      if (!this.assisted && this.gmode === "classic" && !this.reached) this.reportResult(false);
      delete this.data.saves[this.key()];
      this.persist();
      this.dirty = false;
      this.panelT0 = ui.now();
      this.syncScore();
      this.gameOver = true;
      this.playSound(timeout ? "level" : "gameover");
      this.rumble(220);
    }

    panelAction(act) {
      if (act === "continue") {
        this.continued = true;
        this.state = PLAY;
        this.dirty = true;
        this.playSound("click");
      } else if (act === "new") this.start(false);
      else if (act === "undo") this.input("undo");
      else if (act === "setup") this.toSetup();
    }

    message(text) {
      this.msg = [text, 0];
    }

    burst(x, y, color, n, speed) {
      const light = ui.mix(color, [255, 255, 255], 0.35);
      const size = Math.max(1.5, this.cell / 34);
      for (let i = 0; i < n && this.particles.length < MAX_PARTICLES; i++) {
        const a = PG.rand.uniform(0, PG.TAU);
        const sp = speed * PG.rand.uniform(0.35, 1);
        this.particles.push([x, y, Math.cos(a) * sp, Math.sin(a) * sp, 0, PG.rand.uniform(0.35, 0.7), PG.rand.random() < 0.5 ? light : color, size * PG.rand.uniform(0.7, 1.6)]);
      }
    }

    // ===================================================== Update
    update(dt) {
      this.animT += dt;
      if (this.state === SETUP) return;

      if (this.state === PLAY && this.overT === null) {
        if (this.moves > 0) this.elapsed += dt;
        if (this.gmode === "time" && this.clockOn) {
          this.timeLeft -= dt;
          if (this.timeLeft <= 0) {
            this.timeLeft = 0;
            this.finish(true);
            return;
          }
        }
      }

      if (this.slide) {
        this.slide.t += dt;
        if (this.slide.t >= this.slide.dur) this.endSlide();
      }
      while (!this.slide && this.queue.length && this.state === PLAY && this.overT === null && !this.gameOver) {
        const nxt = this.queue.shift();
        if (nxt === "undo") this.undo();
        else this.doMove(nxt);
      }

      for (const [map, dur] of [[this.pops, POP_T], [this.spawns, SPAWN_T]]) {
        for (const [k, v] of map) {
          if (v + dt >= dur) map.delete(k);
          else map.set(k, v + dt);
        }
      }
      if (this.particles.length) {
        const grav = this.cell * 3;
        this.particles = this.particles.filter((p) => {
          p[4] += dt;
          if (p[4] >= p[5]) return false;
          p[0] += p[2] * dt;
          p[1] += p[3] * dt;
          p[2] *= 0.92;
          p[3] = p[3] * 0.92 + grav * dt;
          return true;
        });
      }
      const age = (item, idx, life) => (item[idx] += dt) < life;
      this.rings = this.rings.filter((r) => age(r, 2, 0.55));
      this.popups = this.popups.filter((p) => age(p, 1, POPUP_T));
      if (this.flash && !age(this.flash, 1, FLASH_T)) this.flash = null;
      if (this.msg && !age(this.msg, 1, MSG_T)) this.msg = null;
      if (this.nudge && !age(this.nudge, 2, NUDGE_T)) this.nudge = null;

      if (this.overT !== null && !this.slide && this.state === PLAY) {
        this.overT -= dt;
        if (this.overT <= 0) {
          this.finish(false);
          return;
        }
      }

      this.saveCd -= dt;
      if (this.dirty && this.saveCd <= 0) {
        this.storeGame();
        this.persist();
        this.dirty = false;
        this.saveCd = SAVE_EVERY;
      }
    }

    // ===================================================== Zeichen-Hilfen
    /** Größte Schrift <= px, in der text in maxW passt (gecacht). */
    fitFont(text, px, maxW, bold = false, minPx = 8) {
      const key = text + "|" + px + "|" + Math.floor(maxW) + "|" + (bold ? 1 : 0);
      let f = this.fitCache.get(key);
      if (!f) {
        let size = Math.floor(px);
        f = ui.font(size, bold);
        while (size > minPx && f.width(text) > maxW) {
          size--;
          f = ui.font(size, bold);
        }
        if (this.fitCache.size > 500) this.fitCache.clear();
        this.fitCache.set(key, f);
      }
      return f;
    }

    tileCanvas(v) {
      const scale = (PG.app && PG.app.pixelScale) || 1;
      const key = v + "|" + this.cell + "|" + scale;
      let entry = this.tileCache.get(key);
      if (entry) return entry;
      const cell = this.cell;
      const col = tileColor(v);
      const rad = Math.max(3, Math.floor(cell / 10));
      const glow = v >= 128;
      const pad = glow ? Math.max(2, Math.floor(cell / 7)) : 0;
      const size = cell + 2 * pad;
      const cv = ui.makeCanvas(size * scale, size * scale);
      const c = cv.getContext("2d");
      c.scale(scale, scale);
      if (glow) {
        const strength = Math.min(1, (Math.log2(v) - 6) / 6);
        const gcol = TILE_COLORS[v] ? col : COL_TEXT_BEYOND;
        c.save();
        c.shadowColor = ui.col(gcol, (50 + 130 * strength) / 255);
        c.shadowBlur = pad * 1.2;
        draw.rect(c, gcol, [pad, pad, cell, cell], 0, rad);
        c.restore();
      }
      const depth = Math.max(1, Math.floor(cell / 28));
      draw.rect(c, shade(col, 0.78), [pad, pad + depth, cell, cell - depth], 0, rad);
      draw.rect(c, col, [pad, pad, cell, cell - depth], 0, rad);
      if (!TILE_COLORS[v]) draw.rect(c, COL_TEXT_BEYOND, [pad, pad, cell, cell - depth], Math.max(1, Math.floor(cell / 30)), rad);
      const text = String(v);
      const px = Math.max(7, Math.floor(cell * (DIGIT_SCALE[text.length] || 0.2)));
      const f = this.fitFont(text, px, cell * 0.84, true, 6);
      ui.text(c, text, pad + cell / 2, pad + (cell - depth) / 2, f, tileTextColor(v), "center");
      entry = { cv, pad, size };
      if (this.tileCache.size > 120) this.tileCache.clear();
      this.tileCache.set(key, entry);
      return entry;
    }

    drawTile(ctx, v, x, y, scale = 1) {
      if (scale <= 0.03) return;
      const e = this.tileCanvas(v);
      const sz = e.size * scale;
      ctx.drawImage(e.cv, x + this.cell / 2 - sz / 2, y + this.cell / 2 - sz / 2, sz, sz);
    }

    boardCanvas() {
      const br = this.boardRect;
      const scale = (PG.app && PG.app.pixelScale) || 1;
      const style = ui.fx("style");
      const key = [br.w, this.size, this.cell, this.gap, style, scale].join("|");
      if (this.boardKey !== key) {
        const pad = style === "v1" ? 0 : Math.max(4, Math.floor(br.w / 40));
        const size = br.w + 2 * pad;
        const cv = ui.makeCanvas(size * scale, size * scale);
        const c = cv.getContext("2d");
        c.scale(scale, scale);
        const rad = Math.max(6, Math.floor(br.w / 45));
        if (pad) {
          c.save();
          c.shadowColor = "rgba(0,0,0,0.45)";
          c.shadowBlur = pad * 2;
          c.shadowOffsetY = pad / 2;
          draw.rect(c, COL_BOARD, [pad, pad, br.w, br.h], 0, rad);
          c.restore();
        }
        draw.rect(c, COL_BOARD, [pad, pad, br.w, br.h], 0, rad);
        const crad = Math.max(3, Math.floor(this.cell / 10));
        for (let r = 0; r < this.size; r++) {
          for (let cc = 0; cc < this.size; cc++) {
            draw.rect(c, COL_EMPTY, [pad + this.gap + cc * (this.cell + this.gap), pad + this.gap + r * (this.cell + this.gap), this.cell, this.cell], 0, crad);
          }
        }
        this.boardCache = { cv, pad, size };
        this.boardKey = key;
      }
      return this.boardCache;
    }

    icon(ctx, kind, cx, cy, size, color) {
      const w = Math.max(2, Math.floor(size / 8));
      if (kind === "undo") {
        const r = size * 0.36;
        const rect = new PG.Rect(0, 0, 2 * r, 2 * r);
        rect.center = [cx, cy + size * 0.06];
        draw.arc(ctx, color, rect, -Math.PI / 3, Math.PI, w);
        const a = size * 0.2, tx = rect.left + w / 2, ty = rect.centery;
        draw.polygon(ctx, color, [[tx - a, ty - a * 0.2], [tx + a, ty - a * 0.2], [tx, ty + a]]);
      } else if (kind === "new") {
        const ln = Math.floor(size * 0.34);
        draw.rect(ctx, color, [cx - ln, cy - w / 2, 2 * ln, w], 0, w / 2);
        draw.rect(ctx, color, [cx - w / 2, cy - ln, w, 2 * ln], 0, w / 2);
      } else {
        const ln = Math.floor(size * 0.36);
        for (const [i, knob] of [[-1, 0.35], [0, -0.3], [1, 0.1]]) {
          const yy = cy + i * Math.floor(size * 0.26);
          draw.line(ctx, color, [cx - ln, yy], [cx + ln, yy], Math.max(1, Math.floor(w / 2)));
          draw.circle(ctx, color, [cx + knob * ln, yy], Math.max(2, w));
        }
      }
    }

    infinity(ctx, right, cy, h, color) {
      const r = Math.max(3, Math.floor(h * 0.26));
      const w = Math.max(1, Math.floor(h / 9));
      draw.circle(ctx, color, [right - r, cy], r, w);
      draw.circle(ctx, color, [right - 3 * r + w, cy], r, w);
      return 4 * r;
    }

    // ===================================================== Zeichnen (Spiel)
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawBoard(ctx);
      this.drawSide(ctx);
      if (this.gmode === "time") this.drawTimebar(ctx);
      this.drawEffects(ctx);
      if (this.state === WIN) this.drawPanel(ctx, true);
      else if (this.gameOver) this.drawPanel(ctx, false);
    }

    drawBoard(ctx) {
      this.off = [0, 0];
      if (this.nudge) {
        const [dx, dy, tt] = this.nudge;
        const amp = Math.max(3, this.cell * 0.07) * Math.sin(Math.PI * Math.min(1, tt / NUDGE_T));
        this.off = [Math.round(dx * amp), Math.round(dy * amp)];
      }
      const b = this.boardCanvas();
      ctx.drawImage(b.cv, this.boardRect.x - b.pad + this.off[0], this.boardRect.y - b.pad + this.off[1], b.size, b.size);

      const sl = this.slide;
      if (sl) {
        const p = easeOutCubic(Math.min(1, sl.t / sl.dur));
        if (sl.vanish) {
          const [x, y] = this.cellPos(sl.vanish[0], sl.vanish[1]);
          this.drawTile(ctx, sl.vanish[2], x, y, 1 - p);
        }
        for (const [fr, fc, tr, tc, v] of sl.slides) {
          const [x0, y0] = this.cellPos(fr, fc);
          const [x1, y1] = this.cellPos(tr, tc);
          this.drawTile(ctx, v, x0 + (x1 - x0) * p, y0 + (y1 - y0) * p);
        }
        this.off = [0, 0];
        return;
      }
      const popping = [];
      for (let r = 0; r < this.size; r++) {
        for (let c = 0; c < this.size; c++) {
          const v = this.grid[r][c];
          if (!v) continue;
          const [x, y] = this.cellPos(r, c);
          const k = r * 16 + c;
          if (this.spawns.has(k)) {
            this.drawTile(ctx, v, x, y, easeOutBack(Math.max(0, Math.min(1, this.spawns.get(k) / SPAWN_T))));
            continue;
          }
          if (this.pops.has(k)) {
            popping.push([v, x, y, 1 + 0.2 * Math.sin(Math.PI * Math.min(1, this.pops.get(k) / POP_T))]);
            continue;
          }
          this.drawTile(ctx, v, x, y);
        }
      }
      for (const [v, x, y, s] of popping) this.drawTile(ctx, v, x, y, s);
      this.off = [0, 0];
    }

    card(ctx, rect, label, value, px, color) {
      ui.drawPanel(ctx, rect, { radius: Math.max(6, Math.floor(rect.h / 5)), shadow: false });
      const pad = this.pad;
      ui.text(ctx, label, rect.x + pad, rect.y + pad, this.fitFont(label, this.labPx, rect.w - 2 * pad), ui.TEXT_DIM);
      const lh = this.fLabel.height;
      const vf = this.fitFont(value, px, rect.w - 2 * pad, true);
      const freeH = rect.h - 2 * pad - lh;
      ui.text(ctx, value, rect.x + pad, rect.y + pad + lh + Math.max(2, Math.floor((freeH - vf.height) / 2) + 2), vf, color);
    }

    bestValue() {
      return this.data[this.assisted ? "best_assisted" : "best"][this.key()] || 0;
    }

    drawSide(ctx) {
      const sd = this.sideRect;
      let tag = this.size + "×" + this.size + " · " + t("g2048.mode." + this.gmode);
      const tagCol = ui.mix(this.accent, ui.TEXT, 0.25);
      if (this.assisted) {
        // "mit Rückgängig" ausgeschrieben, wenn es passt - sonst nur das Symbol
        const full = tag + " · " + t("g2048.assisted");
        if (this.fLabel.width(full) <= sd.w - 4) tag = full;
        else {
          const lh = this.fLabel.height;
          const r = ui.text(ctx, tag, sd.x + 2, this.tagRect.centery, this.fitFont(tag, this.labPx, sd.w - lh - 10), tagCol, "midleft");
          this.icon(ctx, "undo", sd.x + 2 + r.w + 4 + lh / 2, this.tagRect.centery, lh, ui.TEXT_DIM);
          tag = null;
        }
      }
      if (tag !== null) ui.text(ctx, tag, sd.x + 2, this.tagRect.centery, this.fitFont(tag, this.labPx, sd.w - 4), tagCol, "midleft");

      this.card(ctx, this.scoreRect, t("g2048.hud.score"), String(this.points), this.bigPx, ui.TEXT);
      this.card(ctx, this.bestRect, t("g2048.hud.best"), String(this.bestValue()), this.midPx, ui.GOLD);

      for (const [gained, tt] of this.popups) {
        const p = tt / POPUP_T;
        const r = this.scoreRect;
        const y = r.bottom - this.pad - this.fRowB.height - Math.floor(p * r.h * 0.55);
        ui.text(ctx, "+" + gained, r.right - this.pad, y, this.fRowB, ui.GREEN, "topright", (1 - p) ** 0.8);
      }

      const ir = this.infoRect, pad = this.pad;
      ui.drawPanel(ctx, ir, { radius: Math.max(6, Math.floor(this.rowH / 2)), shadow: false });
      const rows = [["moves", t("g2048.hud.moves")], ["max", t("g2048.hud.max")], ["undo", t("g2048.hud.undo")], ["time", t("g2048.hud.time")]];
      const fh = this.fRow.height;
      rows.forEach(([kind, label], i) => {
        const cy = ir.y + pad + i * this.rowH + Math.floor(this.rowH / 2);
        const right = ir.right - pad;
        let vw;
        if (kind === "moves") vw = this.value(ctx, String(this.moves), right, cy, ui.TEXT);
        else if (kind === "max") {
          vw = this.value(ctx, String(this.maxTile), right, cy, ui.TEXT);
          const sq = Math.max(6, Math.floor(fh * 0.62));
          const rc = [right - vw - sq - 5, cy - sq / 2, sq, sq];
          draw.rect(ctx, tileColor(Math.max(2, this.maxTile)), rc, 0, Math.max(2, Math.floor(sq / 4)));
          if (this.maxTile && !TILE_COLORS[this.maxTile]) draw.rect(ctx, COL_TEXT_BEYOND, rc, 1, Math.max(2, Math.floor(sq / 4)));
          vw += sq + 5;
        } else if (kind === "undo") {
          if (this.undoRule === "off") vw = this.value(ctx, "–", right, cy, ui.TEXT_FAINT);
          else if (this.undoRule === "limited") {
            const left = this.undoLeft();
            vw = this.value(ctx, left + "/" + UNDO_LIMIT, right, cy, left ? ui.TEXT : ui.TEXT_FAINT);
          } else vw = this.infinity(ctx, right, cy, fh, ui.TEXT);
        } else if (this.gmode === "time") {
          let col = ui.TEXT;
          if (this.timeLeft < 15 && this.clockOn && !this.gameOver) col = ui.mix(ui.RED, ui.TEXT, 0.4 * ui.pulse(8, 0, 1));
          vw = this.value(ctx, fmtTime(this.timeLeft), right, cy, col);
        } else vw = this.value(ctx, fmtTime(this.elapsed), right, cy, ui.TEXT_DIM);
        ui.text(ctx, label, ir.x + pad, cy, this.fitFont(label, this.rowPx, ir.w - 2 * pad - vw - 6), ui.TEXT_DIM, "midleft");
      });

      const specs = [["undo", t("g2048.btn.undo"), "U"], ["new", t("g2048.btn.new"), "R"], ["setup", t("g2048.btn.setup"), "Tab"]];
      const blocked = this.state === WIN || this.gameOver;
      specs.forEach(([kind, label, cap], i) => {
        const rc = this.btnRects[i];
        const hov = !blocked && this.hover && rc.collidepoint(this.hover);
        ui.drawButton(ctx, rc, "", this.fRow, !!hov, { accent: this.accent });
        const enabled = kind !== "undo" || this.canUndo();
        const col = enabled ? ui.TEXT : ui.TEXT_FAINT;
        const isz = Math.max(12, Math.floor(rc.h * 0.56));
        if (this.btnStacked) {
          const ix = rc.x + pad + Math.floor(isz / 2) + 2;
          this.icon(ctx, kind, ix, rc.centery, isz, col);
          const capW = this.fLabel.width(cap) + 8;
          const avail = rc.w - (ix + Math.floor(isz / 2) + 8 - rc.x) - pad - capW;
          if (avail > 20) {
            ui.text(ctx, label, ix + Math.floor(isz / 2) + 8, rc.centery, this.fitFont(label, this.rowPx, avail), col, "midleft");
            ui.text(ctx, cap, rc.right - pad, rc.centery, this.fLabel, ui.TEXT_FAINT, "midright");
          }
        } else this.icon(ctx, kind, rc.centerx, rc.centery, isz, col);
      });

      const area = this.hintArea;
      if (area.h > 0) {
        const parts = t("g2048.play_hint").split("·").map((p) => p.trim());
        const lines = [];
        let cur = "";
        const fnt = this.fLabel;
        for (const part of parts) {
          const trial = cur ? cur + " · " + part : part;
          if (fnt.width(trial) <= area.w) cur = trial;
          else {
            if (cur) lines.push(cur);
            cur = part;
          }
        }
        if (cur) lines.push(cur);
        const lh = fnt.height + 2;
        if (lines.length && lines.length * lh <= area.h && lines.every((ln) => fnt.width(ln) <= area.w)) {
          let y = area.centery - Math.floor((lines.length * lh) / 2);
          for (const ln of lines) {
            ui.text(ctx, ln, area.centerx, y, fnt, ui.TEXT_FAINT, "midtop");
            y += lh;
          }
        }
      }
    }

    value(ctx, text, right, cy, color) {
      return ui.text(ctx, text, right, cy, this.fRowB, color, "midright").w;
    }

    drawTimebar(ctx) {
      const r = this.timeRect;
      if (r.h <= 0) return;
      const rad = Math.floor(r.h / 2);
      draw.rect(ctx, COL_EMPTY, r, 0, rad);
      const frac = Math.max(0, Math.min(1, this.timeLeft / TIME_LIMIT));
      let col = frac > 0.5 ? ui.GREEN : frac > 0.2 ? ui.GOLD : ui.RED;
      if (this.timeLeft < 15 && this.clockOn && !this.gameOver) col = ui.mix(ui.RED, [255, 255, 255], 0.35 * ui.pulse(8, 0, 1));
      const fw = Math.floor(r.w * frac);
      if (fw > 0) draw.rect(ctx, col, [r.x, r.y, Math.max(fw, r.h), r.h], 0, rad);
    }

    drawEffects(ctx) {
      for (const [x, y, , , a, life, col, size] of this.particles) {
        const k = 1 - a / life;
        draw.circle(ctx, col, [x, y], Math.max(1, size * (0.4 + 0.6 * k)));
      }
      const br = this.boardRect;
      if (this.rings.length) {
        // Druckwelle ab 2048 - bleibt auf dem Brett
        ctx.save();
        ctx.beginPath();
        ctx.rect(br.x, br.y, br.w, br.h);
        ctx.clip();
        for (const [cx, cy, tt, col] of this.rings) {
          const p = tt / 0.55;
          const wid = Math.max(1, Math.floor(Math.max(2, Math.floor(this.cell / 14)) * (1 - p)) + 1);
          draw.circle(ctx, ui.mix(ui.mix(col, [255, 255, 255], 0.3), COL_BOARD, p), [cx, cy], this.cell * (0.55 + 0.45 * easeOutCubic(p)), wid);
        }
        ctx.restore();
      }
      if (this.flash) {
        const [text, tt, col] = this.flash;
        const fnt = ui.font(Math.max(30, Math.floor(br.w * (text.length <= 4 ? 0.2 : 0.15))), true);
        const scale = easeOutBack(Math.min(1, tt / 0.22));
        const alpha = tt < FLASH_T - 0.4 ? 1 : Math.max(0, (FLASH_T - tt) / 0.4);
        const rise = Math.max(0, tt - (FLASH_T - 0.4)) * br.h * 0.15;
        ctx.save();
        ctx.translate(br.centerx, br.centery - rise);
        ctx.scale(Math.max(0.01, scale), Math.max(0.01, scale));
        ui.text(ctx, text, 3, 4, fnt, [0, 0, 0], "center", alpha * 0.55);
        ui.text(ctx, text, 0, 0, fnt, col, "center", alpha);
        ctx.restore();
      }
      if (this.msg) {
        const [text, tt] = this.msg;
        const fnt = this.fitFont(text, this.rowPx, br.w - 40);
        const tw = fnt.width(text);
        const drop = Math.floor(Math.min(1, tt / 0.18) * 6);
        const pill = new PG.Rect(br.centerx - tw / 2 - 14, br.y + Math.max(10, Math.floor(br.h / 14)) - 6 + drop - 6, tw + 28, fnt.height + 12);
        ui.drawPanel(ctx, pill, { radius: Math.floor(pill.h / 2), shadow: false });
        ui.text(ctx, text, pill.centerx, pill.centery, fnt, ui.TEXT, "center");
      }
    }

    // ----- Sieg-/Endstand-Panel -------------------------------------------
    panelGeom(win) {
      const br = this.boardRect, h = this.height;
      const pad = Math.max(10, Math.floor(h / 42));
      const gap = Math.max(4, Math.floor(h / 110));
      const bh = Math.max(26, Math.floor(h / 17));
      const rowh = this.fRow.height;
      let heights, acts;
      if (win) {
        heights = [this.fTitle.height, rowh, rowh];
        acts = ["continue", "new"];
      } else {
        heights = [this.fTitle.height, this.fBig.height, rowh, rowh];
        acts = ["new"].concat(this.canUndo() ? ["undo"] : [], ["setup"]);
      }
      const ph = 2 * pad + heights.reduce((a, b) => a + b, 0) + gap * heights.length + bh + gap + this.fLabel.height;
      const pw = Math.min(br.w - 2 * Math.max(6, Math.floor(br.w / 30)), Math.max(Math.floor(br.w * 0.86), 200));
      const rect = new PG.Rect(0, 0, pw, ph);
      rect.center = br.center;
      const k = acts.length;
      const bw = Math.floor((pw - 2 * pad - (k - 1) * gap) / k);
      const by = rect.bottom - pad - this.fLabel.height - gap - bh;
      const btns = acts.map((a, i) => [new PG.Rect(rect.x + pad + i * (bw + gap), by, bw, bh), a]);
      return { rect, heights, btns, pad, gap };
    }

    panelClick(pos) {
      for (const [rc, act] of this.panelGeom(this.state === WIN).btns) {
        if (rc.collidepoint(pos)) {
          this.panelAction(act);
          return true;
        }
      }
      return false;
    }

    drawPanel(ctx, win) {
      const br = this.boardRect;
      const k = easeOutCubic(Math.max(0, Math.min(1, (ui.now() - this.panelT0) / 0.3)));
      draw.rect(ctx, [12, 14, 22, Math.round(165 * k)], br, 0, Math.max(6, Math.floor(br.w / 45)));
      const g = this.panelGeom(win);
      const lift = Math.floor((1 - k) * 18);
      const rect = g.rect.move(0, lift);
      const frame = win || this.overReason === "time" ? ui.GOLD : ui.RED;
      const radius = Math.max(10, Math.floor(this.height / 40));
      ui.drawPanel(ctx, rect, { radius, accentTop: this.accent });
      draw.rect(ctx, frame, rect, 2, radius);
      const inner = rect.w - 2 * g.pad;
      let y = rect.y + g.pad;
      const key = this.key();
      let lines;
      if (win) {
        lines = [
          [t("g2048.reached"), this.titlePx, true, ui.GOLD],
          [t("g2048.keep_going"), this.rowPx, false, ui.TEXT],
          [t("g2048.stats", { score: this.points, moves: this.moves }), this.rowPx, false, ui.TEXT_DIM],
        ];
      } else {
        const title = this.overReason === "time" ? t("g2048.time_up") : t("common.game_over");
        const book = this.data[this.assisted ? "best_assisted" : "best"];
        const bestLine = this.newBest
          ? [t("g2048.new_best"), this.rowPx, true, ui.mix(ui.GOLD, [255, 255, 255], 0.35 * ui.pulse(4, 0, 1))]
          : [t("g2048.best_is", { score: book[key] || 0 }), this.rowPx, false, ui.TEXT_DIM];
        lines = [
          [title, this.titlePx, true, frame],
          [String(this.points), this.bigPx, true, this.newBest ? ui.GOLD : ui.TEXT],
          bestLine,
          [t("g2048.over_stats", { moves: this.moves, tile: this.maxTile }), this.rowPx, false, ui.TEXT_DIM],
        ];
      }
      lines.forEach(([text, px, bold, col], i) => {
        const hh = g.heights[i];
        ui.text(ctx, text, rect.centerx, y + hh / 2, this.fitFont(text, px, inner, bold), col, "center");
        y += hh + g.gap;
      });
      const labels = {
        continue: t("g2048.btn.continue"),
        new: t(g.btns.length > 2 ? "g2048.btn.new" : "g2048.btn.new_game"),
        undo: t("g2048.btn.undo"),
        setup: t("g2048.btn.setup"),
      };
      g.btns.forEach(([rc0, act], i) => {
        const rc = rc0.move(0, lift);
        const hov = this.hover && rc.collidepoint(this.hover);
        const label = labels[act];
        ui.drawButton(ctx, rc, label, this.fitFont(label, this.rowPx, rc.w - 12), i === 0 || !!hov, { accent: this.accent });
      });
      const hint = win ? t("g2048.win_hint") : t("g2048.over_hint");
      ui.text(ctx, hint, rect.centerx, rect.bottom - g.pad + 2, this.fitFont(hint, this.labPx, inner), ui.mix(ui.TEXT_FAINT, ui.TEXT_DIM, ui.pulse(2, 0, 1)), "midbottom");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      const w = this.width, h = this.height, cx = Math.floor(w / 2);
      ui.drawTitle(ctx, w, "2048", { subtitle: this.setupSub ? t("g2048.subtitle") : null, y: this.titleY, big: this.fSetupTitle, small: this.small, accent: this.accent });
      const groups = [
        [this.sizeRects, "g2048.lbl_size", SIZES.map((n) => n + "×" + n), SIZES.map((n) => n === this.size)],
        [this.modeRects, "g2048.lbl_mode", MODE_KEYS.map((m) => t("g2048.mode." + m)), MODE_KEYS.map((m) => m === this.gmode)],
        [this.undoRects, "g2048.lbl_undo", UNDO_KEYS.map((u) => t("g2048.undo." + u)), UNDO_KEYS.map((u) => u === this.undoRule)],
      ];
      const btnPx = Math.max(12, Math.min(Math.floor(h / 30), Math.floor(this.sizeRects[0].h * 0.5)));
      groups.forEach(([rects, lbl, labels, sel], gi) => {
        ui.text(ctx, t(lbl), cx, rects[0].top - 3, this.tiny, this.setupFocus === gi ? this.accent : ui.TEXT_DIM, "midbottom");
        rects.forEach((rc, i) => ui.drawButton(ctx, rc, labels[i], this.fitFont(labels[i], btnPx, rc.w - 14), sel[i], { accent: this.accent }));
      });

      const ir = this.infoRectSetup;
      ui.drawPanel(ctx, ir, { radius: Math.max(8, Math.floor(ir.h / 5)), shadow: false });
      const key = this.key();
      const best = this.data.best[key] || 0, bestA = this.data.best_assisted[key] || 0, tile = this.data.best_tile[key] || 0;
      let line2, col2;
      if (best || bestA || tile) {
        line2 = t("g2048.best_line", { score: best, tile });
        if (bestA > best) line2 += "  ·  " + t("g2048.best_undo", { score: bestA });
        col2 = ui.GOLD;
      } else {
        line2 = t("g2048.no_best");
        col2 = ui.TEXT_FAINT;
      }
      const hs = this.size === HS_SIZE && this.gmode === HS_MODE;
      const lines = [
        [t("g2048.mode_desc." + this.gmode), ui.TEXT_DIM],
        [line2, col2],
        [hs ? t("g2048.hs_yes") : t("g2048.hs_no"), hs ? ui.mix(ui.GREEN, ui.TEXT, 0.25) : ui.TEXT_FAINT],
      ];
      const lh = this.tiny.height;
      let y = ir.y + this.infoPad;
      for (const [text, col] of lines) {
        ui.text(ctx, text, cx, y + lh / 2, this.fitFont(text, Math.max(11, Math.floor(h / 40)), ir.w - 2 * this.infoPad), col, "center");
        y += lh + 2;
      }

      const sv = this.saved();
      const actPx = Math.max(13, Math.floor(h / 30));
      if (sv) {
        const sub = t("g2048.stats", { score: sv.points, moves: sv.moves });
        [[this.contRect, t("g2048.btn.resume")], [this.newRect, t("g2048.btn.new_game")]].forEach(([rc, label], i) => {
          ui.drawButton(ctx, rc, label, this.fitFont(label, actPx, rc.w - 20, i === 0), this.setupAct === i, {
            accent: i === 0 ? ui.GREEN : this.accent,
            sub: i === 0 ? sub : null,
            subFont: this.fitFont(sub, Math.max(11, Math.floor(h / 42)), rc.w - 20),
          });
        });
      } else {
        ui.drawButton(ctx, this.startRect, t("common.start"), this.fitFont(t("common.start"), actPx, this.startRect.w - 20, true), true, { accent: ui.GREEN });
      }
      const hint = t("g2048.setup_hint");
      ui.drawFooter(ctx, w, h, hint, this.fitFont(hint, 14, w - 24));
    }
  }

  // Für Tests/Konsole: reine Logik zugänglich machen.
  PG.g2048 = { slideLine, planMove, hasMoves, cleanSave };

  PG.register(Game2048, {
    id: "Game2048",
    key: "2048",
    name: "2048",
    settingsKey: "g2048",
    defaults: { size: 4, mode: "classic", undo: "limited" },
  });
})();
