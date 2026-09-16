/*
 * tetris.js - Tetris (Port von games/tetris.py)
 * ==============================================
 * Moderne Guideline-Regeln aus tetris_core.js (identisch zur PC-Version):
 * SRS mit Kicks, Hold, 5er-Vorschau, Lock Delay, T-Spins, Back-to-Back,
 * Combos, Perfect Clear, Müllzeilen.
 *
 * Modi: Solo (Setup: Marathon = Highscore, Sprint 40 Zeilen = Bestzeit,
 * Ultra 2 Minuten = Bestwert) und Versus KI (drei Stärken).
 * Web-Version: nur Einzelspieler - "2 Spieler" entfällt.
 *
 * Steuerung: links/rechts (eigenes DAS/ARR), hoch = rechts drehen,
 * runter = Soft Drop, Aktion = Hard Drop; C/Shift = Halten,
 * Z/Y/Strg rechts = links drehen, X = rechts drehen (nur wenn frei).
 * Sprint/Ultra: R = sofort neu. Nach dem Ende: R = nochmal, S = Setup.
 * Bestwerte in PG.store "tetris" (wie die mem.json-Section am PC).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const core = PG.tetrisCore;
  const AI = PG.tetrisAI;
  const { COLS, VISIBLE, BUFFER, ROWS } = core;

  const COLORS = {
    I: [80, 210, 220], O: [240, 220, 90], T: [190, 110, 220],
    S: [110, 220, 120], Z: [235, 100, 100], J: [100, 130, 230],
    L: [240, 160, 80], G: [128, 134, 150], X: [96, 100, 112],
  };
  const PIECE_COLORS = ["I", "O", "T", "S", "Z", "J", "L"].map((k) => COLORS[k]);
  const COL_P1 = COLORS.S;
  const COL_P2 = COLORS.I;
  const COL_B2B = [255, 206, 92];
  const COL_PC = [255, 236, 150];

  const SETUP = "setup", COUNT = "count", PLAY = "play", FINISH = "finish", OVER = "over";
  const VARIANTS = ["marathon", "sprint", "ultra"];
  const SPRINT_LINES = 40;
  const ULTRA_TIME = 120.0;
  const SPRINT_ACH_TIME = 120.0;
  const COUNT_TIME = 1.25;
  const GO_TIME = 0.7;
  const FINISH_TIME = { topout: 1.5, sprint: 1.4, ultra: 1.4, ko: 2.0 };
  const OVER_LOCK = 0.5;
  const VS_LEVEL_EVERY = 40.0;
  const VS_LEVEL_MAX = 12;
  const CLEAR_ANIM = 0.2;
  const DAS_RANGE = [50, 400, 10];
  const ARR_RANGE = [0, 200, 5];
  const STORE_KEY = "tetris";

  const KEYS_SOLO = {
    hold: ["c", "C", "Shift_L", "Shift_R"],
    ccw: ["z", "Z", "y", "Y", "Control_R"],
    cw: ["x", "X"],
  };
  const RESTART_KEYS = ["r", "R"];
  const SETUP_KEYS = ["s", "S"];

  const rgba = (c, a) => [c[0], c[1], c[2], Math.max(0, Math.min(255, Math.round(a)))];
  const easeOut = (p) => 1 - Math.pow(1 - Math.max(0, Math.min(1, p)), 3);
  const nowSec = () => performance.now() / 1000;

  /** Sekunden als "m:ss.cc" (bzw. "m:ss"). */
  function fmtTime(sec, cents = true) {
    sec = Math.max(0, sec);
    const m = Math.floor(sec / 60);
    const s = sec - 60 * m;
    if (cents) {
      const cs = Math.floor(s * 100) / 100;
      return m + ":" + (cs < 10 ? "0" : "") + cs.toFixed(2);
    }
    return m + ":" + String(Math.floor(s)).padStart(2, "0");
  }

  /** Eingabe-Zustand: gehaltene Tasten + eigenes DAS/ARR. */
  class Pad {
    constructor() {
      this.left = new Set();
      this.right = new Set();
      this.down = new Set();
      this.dir = 0;
      this.heldMs = 0;
      this.auto = 0;
    }
    press(d, key) {
      (d < 0 ? this.left : this.right).add(key);
      this.dir = d;
      this.heldMs = 0;
      this.auto = 0;
    }
    release(key) {
      this.down.delete(key);
      for (const [d, keys] of [[-1, this.left], [1, this.right]]) {
        if (keys.has(key)) {
          keys.delete(key);
          if (!keys.size && this.dir === d) {
            const other = d < 0 ? this.right : this.left;
            this.dir = other.size ? -d : 0;
            this.heldMs = 0;
            this.auto = 0;
          }
        }
      }
    }
    clear() {
      this.left.clear();
      this.right.clear();
      this.down.clear();
      this.dir = 0;
    }
  }

  /** Rein optische Effekte eines Spielfelds. */
  function newFx() {
    return {
      shake: 0, clearT: 9, clearRows: [], shift: new Map(), flashCells: [], flashT: 9,
      trail: null, trailT: 9, levelT: 9, riseT: 9, riseN: 0, cancelT: 9, koT: -1,
    };
  }

  /** Steuert ein Feld über PG.tetrisAI: denken, dann Eingabe für Eingabe. */
  class AIPlayer {
    constructor(level, seed) {
      this.level = Math.max(0, Math.min(2, level));
      this.cfg = AI.LEVELS[this.level];
      this.rng = new PG.seedrand.Rand(seed ^ 0x2545f491);
      this.piece = -1;
      this.acts = [];
      this.timer = 0;
      this.replanned = false;
    }
    uniform(a, b) {
      return a + (b - a) * this.rng.random();
    }
    update(dt, board) {
      if (!board.active) return;
      if (board.pieceId !== this.piece) {
        this.piece = board.pieceId;
        this.acts = AI.plan(board, this.level, this.rng);
        this.timer = this.cfg.think * this.uniform(0.7, 1.3);
        this.replanned = false;
      }
      this.timer -= dt;
      let guard = 0;
      while (this.timer <= 0 && this.acts.length && board.active && guard < 16) {
        guard++;
        const act = this.acts.shift();
        const ok = AI.apply(board, act);
        if (act === "drop") {
          this.acts = [];
          break;
        }
        if (act === "hold") this.piece = board.pieceId;
        else if (!ok && !this.replanned) {
          this.acts = AI.plan(board, this.level, this.rng, false);
          this.replanned = true;
        }
        this.timer += this.cfg.step * this.uniform(0.8, 1.2);
      }
    }
  }

  class TetrisGame extends PG.Game {
    init() {
      this.sprites = new Map();
      this.caches = new Map();
    }

    reset() {
      this.score = 0;
      this.gameOver = false;
      if (this.mode !== "solo" && this.mode !== "versus_ai") this.mode = "solo";
      this.versus = this.mode !== "solo";
      this.loadOptions();
      this.makeFonts();
      this.animT = 0;
      this.wins = [0, 0];
      this.best = this.loadBest();
      this.particles = [];
      this.callouts = [];
      this.missiles = [];
      this.boards = [];
      this.pads = [];
      this.fx = [];
      this.ai = null;
      this.result = null;
      this.elapsed = 0;
      this.goT = 0;
      this.overAt = 0;
      this.setupFocus = 0;
      this.overRects = {};
      this.state = SETUP;
      this.deco = Array.from({ length: 9 }, () => this.newDeco(true));
      this.layout();
      this.buildSetupLayout();
    }

    get showHighscoreBanner() {
      return this.mode === "solo" && this.variant === "marathon";
    }

    loadOptions() {
      const o = this.opts;
      const num = (key, lo, hi, def) => {
        const v = o[key];
        return Number.isInteger(v) ? Math.max(lo, Math.min(hi, v)) : def;
      };
      this.variant = VARIANTS.includes(o.solo) ? o.solo : "marathon";
      this.startLevel = num("start_level", 1, 15, 1);
      this.aiLevel = num("ai_level", 0, 2, 1);
      this.das = num("das", DAS_RANGE[0], DAS_RANGE[1], 170);
      this.arr = num("arr", ARR_RANGE[0], ARR_RANGE[1], 50);
      this.ghost = typeof o.ghost === "boolean" ? o.ghost : true;
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    loadBest() {
      const data = PG.store.get(STORE_KEY, {}) || {};
      const out = { sprint: null, sprint_pps: 0, ultra: 0, ultra_lines: 0, vs: [[0, 0], [0, 0], [0, 0]] };
      if (typeof data.sprint === "number" && data.sprint > 0) out.sprint = data.sprint;
      if (typeof data.sprint_pps === "number") out.sprint_pps = data.sprint_pps;
      for (const key of ["ultra", "ultra_lines"]) if (Number.isInteger(data[key]) && data[key] > 0) out[key] = data[key];
      if (Array.isArray(data.vs) && data.vs.length === 3) {
        data.vs.forEach((pair, i) => {
          if (Array.isArray(pair) && pair.length === 2 && pair.every((n) => Number.isInteger(n) && n >= 0)) out.vs[i] = [pair[0], pair[1]];
        });
      }
      return out;
    }

    saveBest() {
      const data = { ultra: this.best.ultra, ultra_lines: this.best.ultra_lines, vs: this.best.vs };
      if (this.best.sprint !== null) {
        data.sprint = Math.round(this.best.sprint * 1000) / 1000;
        data.sprint_pps = Math.round(this.best.sprint_pps * 100) / 100;
      }
      PG.store.set(STORE_KEY, data);
    }

    makeFonts() {
      const h = this.height;
      this.huge = ui.font(Math.max(26, Math.min(64, Math.floor(h / 11))), true);
      this.big = ui.font(Math.max(20, Math.min(44, Math.floor(h / 16))), true);
      this.small = ui.font(Math.max(13, Math.min(22, Math.floor(h / 30))));
      this.smallB = ui.font(Math.max(13, Math.min(22, Math.floor(h / 30))), true);
      this.tiny = ui.font(Math.max(11, Math.min(18, Math.floor(h / 38))));
    }

    // ===================================================== Layout
    layout() {
      const W = this.width, H = this.height;
      this.sides = [];
      if (!this.versus) {
        const m = Math.max(8, Math.floor(H / 28));
        const c = Math.max(8, Math.min(Math.floor((H - 2 * m) / VISIBLE), Math.floor((W - 32) / 22)));
        const fw = COLS * c, fh = VISIBLE * c;
        const fx = Math.floor((W - fw) / 2), fy = Math.floor((H - fh) / 2);
        const gap = Math.max(8, Math.floor(c / 2));
        const sideW = Math.max(3 * c, Math.min(Math.floor(6.8 * c), fx - gap - 10));
        const lx = fx - gap - sideW, rx = fx + fw + gap;
        const hold = new PG.Rect(lx, fy, sideW, Math.floor(4.0 * c));
        const stats = new PG.Rect(lx, hold.bottom + gap, sideW, fy + fh - hold.bottom - gap);
        const nxt = new PG.Rect(rx, fy, sideW, Math.floor(14.2 * c));
        this.sides.push({ field: new PG.Rect(fx, fy, fw, fh), cell: c, hold, next: nxt, stats, bar: null,
          mini: Math.max(5, Math.floor(c * 0.78)), mini2: Math.max(4, Math.floor(c * 0.62)), pad: Math.max(6, Math.floor(c / 3)) });
      } else {
        const top = Math.max(24, Math.floor(H / 14));
        const bottom = Math.max(20, Math.floor(H / 20));
        const c = Math.max(6, Math.min(Math.floor((H - top - bottom - 6) / VISIBLE), Math.floor((W - 16) / 35.6)));
        const fw = COLS * c, fh = VISIBLE * c;
        const side = Math.max(16, Math.floor(2.95 * c));
        const barw = Math.max(4, Math.floor(0.42 * c));
        const gap = Math.max(3, Math.floor(c / 4));
        const sideW = side + gap + barw + fw + gap + side;
        const center = Math.max(10, c);
        const x0 = Math.floor((W - (2 * sideW + center)) / 2);
        const fy = top + Math.floor((H - top - bottom - fh) / 2);
        const mini = Math.max(4, Math.floor(c * 0.6));
        for (let i = 0; i < 2; i++) {
          const sx = x0 + i * (sideW + center);
          const hold = new PG.Rect(sx, fy, side, Math.floor(3.3 * c));
          const bar = new PG.Rect(sx + side + gap, fy, barw, fh);
          const field = new PG.Rect(bar.right, fy, fw, fh);
          const nxt = new PG.Rect(field.right + gap, fy, side, Math.floor(12.4 * c));
          this.sides.push({ field, cell: c, hold, next: nxt, stats: null, bar,
            mini, mini2: Math.max(4, Math.floor(c * 0.5)), pad: Math.max(4, Math.floor(c / 4)) });
        }
      }
      const c = this.sides[0].cell;
      const compact = this.versus;
      this.fLabel = ui.font(Math.max(10, Math.min(17, Math.floor(c * (compact ? 0.56 : 0.62)))), true);
      this.fValue = ui.font(Math.max(12, Math.min(30, Math.floor(c * (compact ? 0.8 : 0.95)))), true);
      this.fHero = ui.font(Math.max(16, Math.min(46, Math.floor(c * 1.35))), true);
      this.fCall = ui.font(Math.max(12, Math.min(34, Math.floor(c * (compact ? 0.78 : 0.92)))), true);
      this.fCallS = ui.font(Math.max(10, Math.min(24, Math.floor(c * 0.62))), true);
    }

    // ===================================================== Setup
    setupItems() {
      const items = this.mode === "solo" ? ["variant", "level"] : ["ai"];
      return items.concat(["ghost", "das", "arr", "start"]);
    }

    buildSetupLayout() {
      const W = this.width, H = this.height;
      const cx = Math.floor(W / 2);
      const bw = Math.min(Math.max(500, Math.floor(W * 0.64)), W - 40);
      const tinyH = this.tiny.height;
      const labH = tinyH + 3;
      const btnH = Math.max(28, Math.min(48, Math.floor(H / 13)));
      const lineH = tinyH + 5;
      this.setupTitleY = Math.floor(H * 0.095);
      this.setupSubY = Math.floor(H * 0.165);
      const top = Math.floor(H * 0.215);
      const bottom = H - (2 * tinyH + 20);
      const blocks = [["choice", labH + btnH + 2 * lineH + 2], ["opts", labH + btnH + lineH + 2], ["start", btnH + 4]];
      const total = blocks.reduce((a, b) => a + b[1], 0);
      const sp = Math.max(4, Math.floor((bottom - top - total) / (blocks.length + 1)));
      let y = top + sp;
      const gap = 8;
      this.setupRects = {};
      for (const [kind, h] of blocks) {
        if (kind === "choice") {
          const w = Math.floor((bw - gap * 2) / 3);
          this.setupRects.choice = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2) + i * (w + gap), y + labH, w, btnH));
          this.setupDescY = y + labH + btnH + 3;
        } else if (kind === "opts") {
          const names = (this.mode === "solo" ? ["level"] : []).concat(["ghost", "das", "arr"]);
          const w = Math.floor((bw - gap * (names.length - 1)) / names.length);
          names.forEach((name, i) => {
            this.setupRects[name] = new PG.Rect(cx - Math.floor(bw / 2) + i * (w + gap), y + labH, w, btnH);
          });
          this.setupHelpY = y + labH + btnH + 3;
        } else {
          const sw = Math.min(bw, Math.max(180, Math.floor(W / 3)));
          this.setupRects.start = new PG.Rect(cx - Math.floor(sw / 2), y, sw, btnH);
        }
        y += h + sp;
      }
      this.setupFooterY = [H - 2 * tinyH - 12, H - tinyH - 6];
    }

    setupChange(item, d) {
      if (item === "variant") {
        this.variant = VARIANTS[PG.mod(VARIANTS.indexOf(this.variant) + d, 3)];
        this.saveSetting("solo", this.variant);
      } else if (item === "ai") {
        this.aiLevel = PG.mod(this.aiLevel + d, 3);
        this.saveSetting("ai_level", this.aiLevel);
      } else if (item === "level") {
        if (this.variant !== "marathon") return;
        this.startLevel = Math.max(1, Math.min(15, this.startLevel + d));
        this.saveSetting("start_level", this.startLevel);
      } else if (item === "ghost") {
        this.ghost = !this.ghost;
        this.saveSetting("ghost", this.ghost);
      } else if (item === "das") {
        this.das = Math.max(DAS_RANGE[0], Math.min(DAS_RANGE[1], this.das + d * DAS_RANGE[2]));
        this.saveSetting("das", this.das);
      } else if (item === "arr") {
        this.arr = Math.max(ARR_RANGE[0], Math.min(ARR_RANGE[1], this.arr + d * ARR_RANGE[2]));
        this.saveSetting("arr", this.arr);
      } else return;
      this.playSound("move");
    }

    handleSetup(ev) {
      const items = this.setupItems();
      if (ev.kind === "keydown") {
        const k = ev.key;
        this.setupFocus = Math.max(0, Math.min(items.length - 1, this.setupFocus));
        const item = items[this.setupFocus];
        if (["Up", "w", "W"].includes(k)) {
          this.setupFocus = PG.mod(this.setupFocus - 1, items.length);
          this.playSound("click");
        } else if (["Down", "s", "S", "Tab"].includes(k)) {
          this.setupFocus = PG.mod(this.setupFocus + 1, items.length);
          this.playSound("click");
        } else if (["Left", "a", "A", "minus", "KP_Subtract"].includes(k)) {
          this.setupChange(item, -1);
        } else if (["Right", "d", "D", "plus", "KP_Add"].includes(k)) {
          this.setupChange(item, 1);
        } else if (["1", "2", "3"].includes(k)) {
          if (this.mode === "solo") {
            this.variant = VARIANTS[Number(k) - 1];
            this.saveSetting("solo", this.variant);
          } else {
            this.aiLevel = Number(k) - 1;
            this.saveSetting("ai_level", this.aiLevel);
          }
          this.playSound("click");
        } else if (k === "g" || k === "G") {
          this.setupChange("ghost", 1);
        } else if (["Return", "KP_Enter", "space"].includes(k) && !ev.repeat) {
          if (k === "space" && item === "ghost") this.setupChange(item, 1);
          else this.start();
        }
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const hit = this.setupHit(ev.pos);
        if (!hit) return;
        const [item, value] = hit;
        if (items.includes(item)) this.setupFocus = items.indexOf(item);
        if (item === "start") this.start();
        else if (item === "variant") {
          this.variant = VARIANTS[value];
          this.saveSetting("solo", this.variant);
          this.playSound("click");
        } else if (item === "ai") {
          this.aiLevel = value;
          this.saveSetting("ai_level", this.aiLevel);
          this.playSound("click");
        } else this.setupChange(item, value);
      }
    }

    setupHit(pos) {
      const r = this.setupRects;
      for (let i = 0; i < r.choice.length; i++) {
        if (r.choice[i].collidepoint(pos)) return [this.mode === "solo" ? "variant" : "ai", i];
      }
      for (const name of ["level", "das", "arr"]) {
        if (r[name] && r[name].collidepoint(pos)) return [name, pos[0] < r[name].centerx ? -1 : 1];
      }
      if (r.ghost.collidepoint(pos)) return ["ghost", 1];
      if (r.start.collidepoint(pos)) return ["start", 0];
      return null;
    }

    // ===================================================== Partie
    start() {
      const seed = Math.floor(Math.random() * 4294967296) >>> 0;
      if (this.versus) {
        this.boards = [new core.Board(seed, 1, true), new core.Board(seed, 1, true)];
      } else if (this.variant === "marathon") {
        this.boards = [new core.Board(seed, this.startLevel)];
      } else {
        this.boards = [new core.Board(seed, 1, true)];
      }
      this.pads = this.boards.map(() => new Pad());
      this.fx = this.boards.map(() => newFx());
      this.ai = this.versus ? new AIPlayer(this.aiLevel, seed) : null;
      this.particles = [];
      this.callouts = [];
      this.missiles = [];
      this.elapsed = 0;
      this.countT = COUNT_TIME;
      this.goT = 0;
      this.result = null;
      this.score = 0;
      this.gameOver = false;
      this.state = COUNT;
      this.layout();
      this.playSound("select");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        this.handleOver(ev);
        return;
      }
      if (ev.kind === "keyup") {
        for (const pad of this.pads) pad.release(ev.key);
        return;
      }
      if (ev.kind !== "keydown" || ev.repeat) return;
      if (this.state !== COUNT && this.state !== PLAY) return;
      const key = ev.key;
      if (this.mode === "solo" && this.variant !== "marathon" && RESTART_KEYS.includes(key) && this.keyIsFree(key)) {
        this.start();
        return;
      }
      this.playKey(key, this.state === PLAY);
    }

    playKey(key, live) {
      const board = this.boards[0];
      const pad = this.pads[0];
      if (this.isAction(key, "left") || this.isAction(key, "right")) {
        const d = this.isAction(key, "left") ? -1 : 1;
        pad.press(d, key);
        if (live && board.move(d)) this.playSound("move");
      } else if (this.isAction(key, "down")) {
        pad.down.add(key);
        if (live) board.softStep();
      } else if (this.isAction(key, "up")) {
        if (live && board.rotate(1)) this.playSound("rotate");
      } else if (this.isAction(key, "action")) {
        if (live) this.hardDrop(0);
      } else if (KEYS_SOLO.hold.includes(key) && this.keyIsFree(key)) {
        if (live && board.hold()) this.playSound("select");
      } else if (KEYS_SOLO.ccw.includes(key) && this.keyIsFree(key)) {
        if (live && board.rotate(-1)) this.playSound("rotate");
      } else if (KEYS_SOLO.cw.includes(key) && this.keyIsFree(key)) {
        if (live && board.rotate(1)) this.playSound("rotate");
      }
    }

    hardDrop(slot) {
      const b = this.boards[slot];
      if (!b.active) return;
      const f = this.fx[slot];
      const gy = b.ghostY();
      if (gy > b.y) {
        const cols = new Map();
        for (const [x, y] of b.cellsOf()) {
          const prev = cols.get(x);
          cols.set(x, [prev ? Math.min(prev[0], y) : y, y + gy - b.y]);
        }
        f.trail = [cols, b.kind];
        f.trailT = 0;
      }
      f.shake = Math.max(f.shake, 0.12);
      b.hardDrop();
    }

    handleOver(ev) {
      if (nowSec() - this.overAt < OVER_LOCK) return;
      if (ev.kind === "keydown" && !ev.repeat) {
        if (RESTART_KEYS.includes(ev.key)) this.start();
        else if (SETUP_KEYS.includes(ev.key)) this.toSetup();
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (const [name, r] of Object.entries(this.overRects)) {
          if (r.collidepoint(ev.pos)) {
            if (name === "again") this.start();
            else this.toSetup();
            return;
          }
        }
      }
    }

    toSetup() {
      this.gameOver = false;
      this.state = SETUP;
      this.score = 0;
      this.boards = [];
      this.particles = [];
      this.callouts = [];
      this.missiles = [];
      this.layout();
      this.buildSetupLayout();
      this.playSound("click");
    }

    // ===================================================== Logik
    update(dt) {
      this.animT += dt;
      this.updateEffects(dt);
      if (this.state === SETUP) {
        this.updateDeco(dt);
        return;
      }
      if (this.state === COUNT) {
        this.countT -= dt;
        if (this.countT <= 0) {
          this.state = PLAY;
          this.goT = GO_TIME;
          this.playSound("powerup");
        }
        return;
      }
      if (this.state === FINISH) {
        this.finishT += dt;
        for (const f of this.fx) if (f.koT >= 0) f.koT += dt;
        if (this.finishT >= this.finishDur) {
          this.state = OVER;
          this.gameOver = true;
          this.overAt = nowSec();
        }
        return;
      }
      if (this.state !== PLAY) return;

      this.elapsed += dt;
      this.goT = Math.max(0, this.goT - dt);
      if (this.versus) {
        const lvl = Math.min(VS_LEVEL_MAX, 1 + Math.floor(this.elapsed / VS_LEVEL_EVERY));
        if (lvl !== this.boards[0].level) {
          this.boards.forEach((b, i) => {
            b.level = lvl;
            this.callout(i, [[t("tetris.call.level", { n: lvl }), ui.GOLD]]);
            this.fx[i].levelT = 0;
          });
          this.playSound("level");
        }
      }
      this.boards.forEach((b, i) => {
        const pad = this.pads[i];
        if (i === 0) this.dasStep(pad, b, dt);
        b.tick(dt, i === 0 && pad.down.size > 0);
      });
      if (this.ai) this.ai.update(dt, this.boards[1]);
      for (let i = 0; i < this.boards.length; i++) this.processEvents(i);
      this.checkEnd();
      if (this.mode === "solo" && this.variant === "marathon") this.score = this.boards[0].score;
    }

    dasStep(pad, board, dt) {
      if (pad.dir === 0) return;
      pad.heldMs += dt * 1000;
      if (pad.heldMs < this.das || !board.active) return;
      let moved = false;
      if (this.arr <= 0) {
        while (board.move(pad.dir)) moved = true;
      } else {
        const target = 1 + Math.floor((pad.heldMs - this.das) / this.arr);
        const n = Math.min(target - pad.auto, COLS);
        pad.auto = target;
        for (let i = 0; i < n; i++) {
          if (!board.move(pad.dir)) break;
          moved = true;
        }
      }
      if (moved) this.playSound("move");
    }

    processEvents(i) {
      const b = this.boards[i];
      const f = this.fx[i];
      for (const [kind, data] of b.events) {
        if (kind === "lock") this.onLock(i, data);
        else if (kind === "level") {
          f.levelT = 0;
          this.callout(i, [[t("tetris.call.level", { n: data }), ui.GOLD]]);
          this.playSound("level");
        } else if (kind === "garbage") {
          f.riseT = 0;
          f.riseN = data;
          f.shake = Math.max(f.shake, 0.2);
          if (i === 0) {
            this.playSound("hit");
            this.rumble(140);
          }
        }
      }
      b.events.length = 0;
    }

    onLock(i, info) {
      const f = this.fx[i];
      const human = i === 0;
      const side = this.sides[i];
      const c = side.cell;
      const field = side.field;
      f.flashCells = info.cells.slice();
      f.flashT = 0;
      const lines = info.lines;
      if (human || lines) this.playSound(lines ? "line" : "lock");
      if (lines) {
        f.clearT = 0;
        f.clearRows = info.rows.map((r, k) => [r, info.cleared[k]]);
        const full = new Set(info.rows);
        f.shift = new Map();
        let j = 0;
        for (let o = 0; o < ROWS; o++) {
          if (full.has(o)) continue;
          const k = lines + j - o;
          if (k > 0 && lines + j >= BUFFER) f.shift.set(lines + j, k);
          j++;
        }
        for (const [r, row] of f.clearRows) {
          if (r < BUFFER) continue;
          const y = field.y + (r - BUFFER) * c + c / 2;
          row.forEach((kind, x) => {
            const col = COLORS[kind || "G"] || COLORS.G;
            for (let n = 0; n < (human ? 2 : 1); n++) this.particle(field.x + x * c + c / 2, y, col, c);
          });
        }
        if (lines === 4 || info.spin) {
          f.shake = Math.max(f.shake, 0.18);
          if (human) this.rumble(160);
        }
      }

      const rows = [];
      const spin = info.spin;
      if (spin) {
        let word = t(spin === "mini" ? "tetris.call.tspin_mini" : "tetris.call.tspin");
        if (lines) word = t("tetris.call.spin_lines", { spin: word, lines: t("tetris.call.lines" + lines) });
        rows.push([word, COLORS.T]);
      } else if (lines === 4) rows.push([t("tetris.call.tetris"), COLORS.I]);
      if (info.b2b) rows.push([t("tetris.call.b2b"), COL_B2B]);
      if (info.combo >= 1) rows.push([t("tetris.call.combo", { n: info.combo }), ui.mix(COLORS.L, [255, 255, 255], 0.15)]);
      if (info.pc) rows.push([t("tetris.call.pc"), COL_PC]);
      if (rows.length) {
        const small = [];
        if (!this.versus && info.points && this.variant !== "sprint") small.push("+" + info.points);
        else if (this.versus && info.sent) small.push("+" + info.sent);
        this.callout(i, rows, small);
      }
      if (spin && human) this.tone(spin === "full" ? 880 : 660, 0.12, "square", 0.3);
      if (info.pc) {
        this.playSound("win");
        for (let n = 0; n < 40; n++) {
          this.particle(field.centerx + PG.rand.uniform(-field.w / 2, field.w / 2),
            field.bottom - PG.rand.uniform(0, field.h * 0.3), PG.rand.choice([COL_PC, COLORS.I, COLORS.T]), c);
        }
      } else if (lines === 4 && human) this.playSound("powerup");

      if (human) {
        if (lines === 4) this.achEvent("tetris_four");
        if (spin === "full" && lines === 2) this.achEvent("tetris_tspin");
        if (info.pc) this.achEvent("tetris_pc");
      }

      if (this.versus) {
        if (info.cancelled) f.cancelT = 0;
        if (info.sent) {
          const other = 1 - i;
          this.boards[other].receive(info.sent);
          const src = this.sides[i].field;
          const dst = this.sides[other].bar;
          this.missiles.push({ x0: src.centerx, y0: src.y + src.h * 0.4, x1: dst.centerx, y1: dst.bottom - c,
            t: 0, dur: 0.38, n: info.sent, color: i === 0 ? COL_P1 : COL_P2 });
        }
      }
    }

    checkEnd() {
      if (!this.versus) {
        const b = this.boards[0];
        if (b.dead) this.finish("topout");
        else if (this.variant === "sprint" && b.lines >= SPRINT_LINES) this.finish("sprint");
        else if (this.variant === "ultra" && this.elapsed >= ULTRA_TIME) {
          this.elapsed = ULTRA_TIME;
          this.finish("ultra");
        }
        return;
      }
      const dead = this.boards.map((b, i) => (b.dead ? i : -1)).filter((i) => i >= 0);
      if (dead.length) this.finish("ko", dead.length === 2 ? null : 1 - dead[0]);
    }

    statsOf(i) {
      const b = this.boards[i];
      const el = Math.max(0.001, this.elapsed);
      return { lines: b.lines, score: b.score, level: b.level, pieces: b.pieces, pps: b.pieces / el,
        attack: b.attackSent, tetrises: b.tetrises, tspins: b.tspins };
    }

    finish(reason, winner = null) {
      this.state = FINISH;
      this.finishT = 0;
      this.finishDur = FINISH_TIME[reason];
      for (const b of this.boards) b.active = false;
      const res = { reason, winner, time: this.elapsed, stats: this.boards.map((_, i) => this.statsOf(i)),
        newBest: false, prevBest: null };
      const b0 = this.boards[0];
      if (!this.versus) {
        if (reason === "topout") {
          this.fx[0].koT = 0;
          this.playSound("gameover");
          this.rumble(220);
          if (this.variant === "marathon") this.score = b0.score;
        } else if (reason === "sprint") {
          res.prevBest = this.best.sprint;
          if (this.best.sprint === null || this.elapsed < this.best.sprint) {
            this.best.sprint = this.elapsed;
            this.best.sprint_pps = res.stats[0].pps;
            res.newBest = true;
            this.saveBest();
          }
          if (this.elapsed < SPRINT_ACH_TIME) this.achEvent("tetris_sprint");
          this.playSound("win");
          this.confetti(0);
        } else {
          res.prevBest = this.best.ultra || null;
          if (b0.score > this.best.ultra) {
            this.best.ultra = b0.score;
            this.best.ultra_lines = b0.lines;
            res.newBest = true;
            this.saveBest();
          }
          this.playSound("win");
          this.confetti(0);
        }
      } else {
        if (winner !== null) this.wins[winner]++;
        this.boards.forEach((b, i) => {
          if (!b.dead) return;
          this.fx[i].koT = 0;
          const field = this.sides[i].field;
          for (let n = 0; n < 50; n++) {
            this.particle(PG.rand.uniform(field.left, field.right), PG.rand.uniform(field.centery, field.bottom),
              PG.rand.choice(PIECE_COLORS), this.sides[i].cell, 1.8);
          }
        });
        this.playSound("explode");
        this.rumble(260);
        if (winner !== null) {
          this.best.vs[this.aiLevel][winner === 0 ? 0 : 1]++;
          this.saveBest();
          this.reportResult(winner === 0);
          this.confetti(winner);
        }
      }
      this.result = res;
    }

    // ===================================================== Effekte
    particle(x, y, color, c, power = 1) {
      if (this.particles.length > 420) return;
      const sp = c * 9 * power;
      const a = PG.rand.uniform(0, PG.TAU);
      const v = PG.rand.uniform(0.25, 1) * sp;
      const life = PG.rand.uniform(0.45, 0.9);
      this.particles.push([x, y, Math.cos(a) * v, Math.sin(a) * v - sp * 0.5, life, life, color,
        Math.max(2, Math.floor(c * PG.rand.uniform(0.14, 0.3)))]);
    }

    confetti(i) {
      const field = this.sides[i].field;
      for (let n = 0; n < 60; n++) {
        this.particle(PG.rand.uniform(field.left, field.right), field.y + PG.rand.uniform(0, field.h * 0.5),
          PG.rand.choice(PIECE_COLORS), this.sides[i].cell, 1.3);
      }
    }

    callout(i, rows, small = []) {
      const lines = rows.map(([text, color]) => [text, color, this.fCall]);
      for (const text of small) lines.push([text, ui.TEXT, this.fCallS]);
      this.callouts = this.callouts.filter((cl) => cl.side !== i);
      this.callouts.push({ side: i, lines, t: 0, dur: 1.25 });
    }

    updateEffects(dt) {
      for (const f of this.fx) {
        f.shake = Math.max(0, f.shake - dt);
        f.clearT += dt;
        f.flashT += dt;
        f.trailT += dt;
        f.levelT += dt;
        f.riseT += dt;
        f.cancelT += dt;
      }
      const alive = [];
      for (const p of this.particles) {
        p[4] -= dt;
        if (p[4] <= 0) continue;
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] += 1400 * dt * (p[7] / 6);
        alive.push(p);
      }
      this.particles = alive;
      for (const cl of this.callouts) cl.t += dt;
      this.callouts = this.callouts.filter((cl) => cl.t < cl.dur);
      const keep = [];
      for (const m of this.missiles) {
        m.t += dt;
        if (m.t < m.dur) keep.push(m);
        else for (let n = 0; n < 8; n++) this.particle(m.x1, m.y1, m.color, 12, 0.8);
      }
      this.missiles = keep;
    }

    newDeco(anywhere = false) {
      return [PG.rand.choice(core.KINDS.split("")), Math.random(),
        anywhere ? Math.random() : PG.rand.uniform(-0.3, -0.1), PG.rand.uniform(0.02, 0.06)];
    }

    updateDeco(dt) {
      for (const d of this.deco) d[2] += d[3] * dt;
      this.deco = this.deco.map((d) => (d[2] < 1.15 ? d : this.newDeco()));
    }

    // ===================================================== Zeichen-Helfer
    pixelScale() {
      return Math.max(1, (PG.app && PG.app.pixelScale) || 1);
    }

    /** Gecachter Block (Identitätsfarbe) als Offscreen-Canvas in Zellgröße c. */
    sprite(kind, c, style = "block") {
      const ps = this.pixelScale();
      const flat = ui.fx("style") === "v1";
      const key = kind + "|" + c + "|" + style + "|" + flat + "|" + ps;
      let cv = this.sprites.get(key);
      if (cv) return cv;
      if (this.sprites.size > 400) this.sprites.clear();
      cv = ui.makeCanvas(Math.ceil(c * ps), Math.ceil(c * ps));
      const g = cv.getContext("2d");
      g.scale(ps, ps);
      const base = COLORS[kind] || COLORS.G;
      const s = c - 1;
      if (style === "ghost") {
        draw.rect(g, rgba(base, 50), [1, 1, s - 1, s - 1], 0, Math.max(1, Math.floor(c / 7)));
        draw.rect(g, rgba(base, 190), [1, 1, s - 1, s - 1], Math.max(1, Math.floor(c / 12)), Math.max(1, Math.floor(c / 7)));
      } else if (flat || c < 9) {
        draw.rect(g, base, [0, 0, s, s], 0, Math.floor(c / 9));
        draw.rect(g, base.map((v) => Math.floor(v * 0.7)), [0, 0, s, s], 1, Math.floor(c / 9));
      } else {
        const bev = Math.max(2, Math.floor(c / 6));
        const light = ui.mix(base, [255, 255, 255], 0.42);
        const dark = base.map((v) => Math.floor(v * 0.58));
        const face = ui.mix(base, [255, 255, 255], 0.06);
        draw.rect(g, dark, [0, 0, s, s], 0, Math.max(1, Math.floor(c / 10)));
        draw.polygon(g, light, [[0, 0], [s - 1, 0], [s - bev, bev], [bev, bev], [bev, s - bev], [0, s - 1]]);
        draw.rect(g, face, [bev, bev, s - 2 * bev, s - 2 * bev]);
        draw.rect(g, [255, 255, 255, 80], [bev + 1, bev + 1, Math.max(1, Math.floor((s - 2 * bev) / 2)), Math.max(1, Math.floor(bev / 2) + 1)]);
      }
      cv.logical = c;
      this.sprites.set(key, cv);
      return cv;
    }

    blitSprite(ctx, cv, x, y) {
      ctx.drawImage(cv, x, y, cv.logical, cv.logical);
    }

    /** Stein in Spawn-Lage, zentriert um (cx, cy). */
    drawPiece(ctx, kind, mc, cx, cy, alpha = 1) {
      const sh = core.SHAPES[kind][0];
      const w = (sh.maxx - sh.minx + 1) * mc;
      const h = (sh.maxy - sh.miny + 1) * mc;
      const x0 = Math.round(cx - w / 2), y0 = Math.round(cy - h / 2);
      const spr = this.sprite(alpha >= 1 ? kind : "X", mc);
      ctx.save();
      ctx.globalAlpha = alpha;
      for (const [x, y] of sh.cells) this.blitSprite(ctx, spr, x0 + (x - sh.minx) * mc, y0 + (y - sh.miny) * mc);
      ctx.restore();
    }

    fieldBg(w, h, c) {
      const ps = this.pixelScale();
      const key = [w, h, c, ps, ui.PANEL.join(","), ui.BORDER.join(","), ui.BG_BOTTOM.join(",")].join("|");
      let cached = this.caches.get("fieldbg");
      if (!cached || cached.key !== key) {
        const cv = ui.makeCanvas(w * ps, h * ps);
        const g = cv.getContext("2d");
        g.scale(ps, ps);
        const base = ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.35);
        const grad = g.createLinearGradient(0, 0, 0, h);
        grad.addColorStop(0, ui.col(base, 238 / 255));
        grad.addColorStop(1, ui.col(ui.mix(base, ui.PANEL, 0.5), 238 / 255));
        g.fillStyle = grad;
        g.fillRect(0, 0, w, h);
        const grid = rgba(ui.mix(ui.PANEL, ui.BORDER, 0.45), 110);
        for (let x = 1; x < COLS; x++) draw.line(g, grid, [x * c + 0.5, 0], [x * c + 0.5, h]);
        for (let y = 1; y < VISIBLE; y++) draw.line(g, grid, [0, y * c + 0.5], [w, y * c + 0.5]);
        cached = { key, cv, w, h };
        this.caches.set("fieldbg", cached);
      }
      return cached;
    }

    panel(ctx, r, alpha = 205) {
      const rad = Math.max(4, Math.min(12, Math.floor(r.h / 6)));
      draw.rect(ctx, rgba(ui.PANEL, alpha), r, 0, rad);
      draw.rect(ctx, ui.BORDER, r, 1, rad);
    }

    /** Text, der notfalls kleiner gesetzt wird, damit er in maxW passt. */
    fitText(ctx, text, x, y, font, color, maxW, anchor = "topleft", alpha) {
      let f = font;
      const w = font.width(text);
      if (w > maxW && maxW > 10) f = ui.font(Math.max(7, Math.floor((font.px * maxW) / w)), font.bold);
      return ui.text(ctx, text, x, y, f, color, anchor, alpha);
    }

    label(ctx, text, rect, y) {
      let f = this.fLabel;
      if (f.width(text) > rect.w - 6) {
        f = ui.font(Math.max(9, Math.floor(this.fLabel.height * 0.72)), true);
        if (f.width(text) > rect.w - 6) return y;
      }
      ui.text(ctx, text, rect.centerx, y, f, ui.TEXT_DIM, "midtop");
      return y + f.height;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Die Pause frisst im Browser die keyups -> gehaltene Tasten lösen.
      if (this.paused) for (const pad of this.pads) pad.clear();
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        this.drawParticles(ctx);
        return;
      }
      for (let i = 0; i < this.boards.length; i++) this.drawSide(ctx, i);
      if (this.versus) this.drawVsHud(ctx);
      this.drawMissiles(ctx);
      this.drawParticles(ctx);
      this.drawCallouts(ctx);
      if (this.state === COUNT) this.drawCountdown(ctx);
      else if (this.state === PLAY && this.goT > 0) this.drawGo(ctx);
      else if (this.state === FINISH || this.state === OVER) this.drawFinishMarks(ctx);
      if (this.state === OVER) this.drawResults(ctx);
    }

    drawSide(ctx, i) {
      const L = this.sides[i];
      const f = this.fx[i];
      let field = L.field;
      if (f.shake > 0) {
        const amp = Math.max(1, Math.floor(L.cell / 7));
        field = field.move(0, Math.round(amp * Math.sin(f.shake * 70) * f.shake * 6));
      }
      this.drawField(ctx, i, field, L.cell);
      this.drawHold(ctx, i, L);
      this.drawNext(ctx, i, L);
      if (L.bar) this.drawBar(ctx, i, L);
      if (L.stats) this.drawStats(ctx, L);
    }

    drawField(ctx, i, rect, c) {
      const b = this.boards[i];
      const f = this.fx[i];
      const bg = this.fieldBg(rect.w, rect.h, c);
      ctx.drawImage(bg.cv, rect.x, rect.y, bg.w, bg.h);
      const grayRows = f.koT >= 0 ? Math.ceil(Math.min(1, f.koT / 0.8) * VISIBLE) : 0;
      ctx.save();
      ctx.beginPath();
      ctx.rect(rect.x, rect.y, rect.w, rect.h);
      ctx.clip();
      const clearing = f.clearT < CLEAR_ANIM && f.shift.size;
      const e = clearing ? easeOut(f.clearT / CLEAR_ANIM) : 1;
      const rise = !clearing && f.riseT < 0.14 && f.riseN ? Math.round(f.riseN * c * (1 - easeOut(f.riseT / 0.14))) : 0;
      for (let y = BUFFER; y < ROWS; y++) {
        if (!b.rows[y]) continue;
        const gray = ROWS - y <= grayRows;
        const dy = (clearing ? -Math.round((f.shift.get(y) || 0) * c * (1 - e)) : 0) + rise;
        const py = rect.y + (y - BUFFER) * c + dy;
        const row = b.cells[y];
        for (let x = 0; x < COLS; x++) {
          if (row[x] !== null) this.blitSprite(ctx, this.sprite(gray ? "X" : row[x], c), rect.x + x * c, py);
        }
      }

      if (f.clearT < 0.32) {
        const p = f.clearT / 0.32;
        for (const [r] of f.clearRows) {
          if (r < BUFFER) continue;
          const w = Math.floor(rect.w * (0.55 + 0.45 * easeOut(p * 1.6)));
          draw.rect(ctx, [255, 255, 255, 230 * (1 - p)], [rect.centerx - w / 2, rect.y + (r - BUFFER) * c, w, c]);
        }
      }

      if (f.trail && f.trailT < 0.2) {
        const [cols, kind] = f.trail;
        const col = COLORS[kind] || COLORS.G;
        const a = 1 - f.trailT / 0.2;
        for (const [x, [top, bot]] of cols) {
          const y0 = Math.max(top, BUFFER) - BUFFER;
          const y1 = bot - BUFFER;
          if (y1 <= y0) continue;
          const gy0 = rect.y + y0 * c, gy1 = rect.y + y1 * c;
          const grad = ctx.createLinearGradient(0, gy0, 0, gy1);
          grad.addColorStop(0, ui.col(col, 0));
          grad.addColorStop(1, ui.col(col, 0.43 * a));
          ctx.fillStyle = grad;
          ctx.fillRect(rect.x + x * c + c / 5, gy0, c - (2 * c) / 5, gy1 - gy0);
        }
      }

      if (b.active) {
        if (this.ghost) {
          const gy = b.ghostY();
          if (gy !== b.y) {
            const spr = this.sprite(b.kind, c, "ghost");
            for (const [x, y] of b.cellsOf(b.kind, b.rot, b.x, gy)) {
              if (y >= BUFFER) this.blitSprite(ctx, spr, rect.x + x * c, rect.y + (y - BUFFER) * c);
            }
          }
        }
        const spr = this.sprite(b.kind, c);
        const dim = b.grounded() ? Math.min(1, b.lockTimer / core.LOCK_DELAY) : 0;
        for (const [x, y] of b.cellsOf()) {
          if (y < BUFFER - 2) continue;
          const px = rect.x + x * c, py = rect.y + (y - BUFFER) * c;
          this.blitSprite(ctx, spr, px, py);
          if (dim > 0.05) draw.rect(ctx, [255, 255, 255, 70 * dim], [px, py, c - 1, c - 1]);
        }
      }
      if (f.flashT < 0.14) {
        const a = 150 * (1 - f.flashT / 0.14);
        for (const [x, y] of f.flashCells) {
          if (y >= BUFFER) draw.rect(ctx, [255, 255, 255, a], [rect.x + x * c, rect.y + (y - BUFFER) * c, c - 1, c - 1], 0, Math.max(1, Math.floor(c / 8)));
        }
      }
      ctx.restore();

      let border = ui.BORDER_LIGHT;
      let width = 2;
      const height = b.stackHeight();
      if (!b.dead && (height >= 17 || height + b.pendingLines() >= 19)) {
        border = ui.mix(ui.BORDER_LIGHT, ui.RED, ui.pulse(7, 0.35, 1));
        width = 3;
      }
      if (f.levelT < 0.8) {
        border = ui.mix(ui.GOLD, border, f.levelT / 0.8);
        width = 3;
      }
      draw.rect(ctx, border, rect.inflate(4, 4), width, Math.max(3, Math.floor(c / 5)));
    }

    drawHold(ctx, i, L) {
      const b = this.boards[i];
      const r = L.hold;
      this.panel(ctx, r);
      const y = this.label(ctx, t("tetris.hud.hold"), r, r.y + Math.floor(L.pad / 2) + 1);
      if (b.holdKind) {
        const areaTop = y + 2;
        this.drawPiece(ctx, b.holdKind, L.mini, r.centerx, areaTop + (r.bottom - areaTop) / 2, b.holdUsed ? 0.47 : 1);
      }
    }

    drawNext(ctx, i, L) {
      const b = this.boards[i];
      const r = L.next;
      this.panel(ctx, r);
      const y = this.label(ctx, t("tetris.hud.next"), r, r.y + Math.floor(L.pad / 2) + 1) + Math.floor(L.pad / 2);
      const kinds = b.queue.peek(5);
      const slot1 = Math.floor(L.mini * 2.9);
      const rest = (r.bottom - y - slot1 - 4) / 4;
      kinds.forEach((kind, k) => {
        const mc = k === 0 ? L.mini : L.mini2;
        const cy = k === 0 ? y + slot1 / 2 : y + slot1 + Math.floor(rest * (k - 0.5));
        this.drawPiece(ctx, kind, mc, r.centerx, cy);
      });
    }

    drawBar(ctx, i, L) {
      const b = this.boards[i];
      const f = this.fx[i];
      const r = L.bar;
      const c = L.cell;
      draw.rect(ctx, ui.PANEL, r, 0, 2);
      let y = r.bottom;
      for (const [n, , age] of b.pending) {
        const h = Math.min(n * c, y - r.y);
        if (h <= 0) break;
        const ready = age >= core.GARBAGE_DELAY;
        const col = ready ? ui.mix(ui.RED, [255, 255, 255], 0.25 * ui.pulse(9, 0, 1)) : ui.GOLD;
        draw.rect(ctx, col, [r.x, y - h, r.w, h - 1], 0, 2);
        y -= h;
      }
      if (f.cancelT < 0.3) draw.rect(ctx, [255, 255, 255, 120 * (1 - f.cancelT / 0.3)], [r.x - 2, r.y, r.w + 4, r.h]);
      draw.rect(ctx, ui.BORDER, r.inflate(2, 2), 1, 2);
    }

    statRows() {
      const b = this.boards[0];
      const el = this.elapsed;
      const pps = (el > 0.5 ? b.pieces / el : 0).toFixed(2);
      if (this.variant === "sprint") {
        const left = Math.max(0, SPRINT_LINES - b.lines);
        return [[t("tetris.hud.time"), fmtTime(el), true, null],
          [t("tetris.hud.lines"), Math.min(b.lines, SPRINT_LINES) + " / " + SPRINT_LINES, false, left <= 5 ? ui.GREEN : null],
          [t("tetris.hud.pps"), pps, false, null],
          [t("tetris.hud.level"), String(b.level), false, null]];
      }
      if (this.variant === "ultra") {
        const rest = Math.max(0, ULTRA_TIME - el);
        return [[t("tetris.hud.left"), fmtTime(rest), true, rest < 10 ? ui.RED : null],
          [t("tetris.hud.score"), String(b.score), false, null],
          [t("tetris.hud.lines"), String(b.lines), false, null],
          [t("tetris.hud.pps"), pps, false, null]];
      }
      return [[t("tetris.hud.score"), String(b.score), true, null],
        [t("tetris.hud.level"), String(b.level), false, null],
        [t("tetris.hud.lines"), String(b.lines), false, null],
        [t("tetris.hud.time"), fmtTime(el, false), false, null],
        [t("tetris.hud.pps"), pps, false, null]];
    }

    drawStats(ctx, L) {
      const r = L.stats;
      const pad = L.pad;
      const labH = this.fLabel.height;
      let y = r.y + pad;
      const used = [];
      for (const [label, value, hero, col] of this.statRows()) {
        const fnt = hero ? this.fHero : this.fValue;
        const h = labH + fnt.height + pad;
        if (y + h > r.bottom) break;
        used.push([label, value, fnt, col, y]);
        y += h;
      }
      this.panel(ctx, new PG.Rect(r.x, r.y, r.w, y - r.y));
      for (const [label, value, fnt, col, yy] of used) {
        this.fitText(ctx, label, r.centerx, yy, this.fLabel, ui.TEXT_DIM, r.w - 12, "midtop");
        this.fitText(ctx, value, r.centerx, yy + labH, fnt, col || (fnt === this.fHero ? this.accent : ui.TEXT), r.w - 10, "midtop");
      }
    }

    drawVsHud(ctx) {
      for (let i = 0; i < 2; i++) {
        const L = this.sides[i];
        const field = L.field;
        const name = i === 0 ? t("tetris.player.you") : t("tetris.player.ai", { level: t("tetris.ai." + this.aiLevel) });
        const maxW = L.next.right - L.hold.x;
        this.fitText(ctx, name + "   " + this.wins[i], field.centerx, field.y - 6, this.fValue, i === 0 ? COL_P1 : COL_P2, maxW, "midbottom");
        const b = this.boards[i];
        this.fitText(ctx, t("tetris.vs_line", { lines: b.lines, attack: b.attackSent }), field.centerx, field.bottom + 6,
          this.fLabel, ui.TEXT_DIM, maxW, "midtop");
      }
      const mid = (this.sides[0].next.right + this.sides[1].hold.x) / 2;
      ui.text(ctx, fmtTime(this.elapsed, false), mid, this.sides[0].field.y - 6, this.fLabel, ui.TEXT_DIM, "midbottom");
    }

    drawParticles(ctx) {
      for (const [x, y, , , life, maxl, col, size] of this.particles) {
        const sz = Math.max(1, Math.floor(size * (0.5 + 0.5 * (life / maxl))));
        draw.rect(ctx, col, [Math.floor(x - sz / 2), Math.floor(y - sz / 2), sz, sz]);
      }
    }

    drawMissiles(ctx) {
      for (const m of this.missiles) {
        const p = m.t / m.dur;
        for (let k = 0; k < 5; k++) {
          const q = Math.max(0, p - k * 0.05);
          const e = easeOut(q);
          const x = m.x0 + (m.x1 - m.x0) * e;
          const y = m.y0 + (m.y1 - m.y0) * e - Math.sin(q * Math.PI) * 40;
          const rad = Math.max(2, (6 + Math.min(6, m.n)) * (1 - k * 0.18));
          draw.circle(ctx, ui.mix(m.color, [255, 255, 255], k === 0 ? 0.5 : 0), [x, y], rad);
        }
      }
    }

    drawCallouts(ctx) {
      for (const cl of this.callouts) {
        const field = this.sides[cl.side].field;
        const p = cl.t / cl.dur;
        const alpha = Math.min(1, cl.t / 0.08) * Math.min(1, (1 - p) / 0.3);
        const total = cl.lines.reduce((a, l) => a + l[2].height, 0);
        let y = field.y + Math.floor(field.h * 0.3) - total / 2 - p * field.h * 0.05;
        draw.rect(ctx, [6, 8, 14, 120 * alpha], [field.x + 2, y - 4, field.w - 4, total + 8]);
        for (const [text, color, font] of cl.lines) {
          this.fitText(ctx, text, field.centerx, y, font, color, field.w - 8, "midtop", alpha);
          y += font.height;
        }
      }
    }

    centerOfPlay() {
      if (this.versus) return [this.width / 2, this.sides[0].field.centery];
      return this.sides[0].field.center;
    }

    drawCountdown(ctx) {
      const [cx, cy] = this.centerOfPlay();
      const p = 1 - this.countT / COUNT_TIME;
      const k = 1 + 0.06 * Math.sin(p * PG.TAU * 2);
      const text = t("tetris.ready");
      const w = this.big.width(text) * k, h = this.big.height * k;
      this.panel(ctx, new PG.Rect(cx - w / 2 - 15, cy - h / 2 - 7, w + 30, h + 14), 225);
      ctx.save();
      ctx.translate(cx, cy);
      ctx.scale(k, k);
      ui.text(ctx, text, 0, 0, this.big, ui.TEXT, "center");
      ctx.restore();
    }

    drawGo(ctx) {
      const [cx, cy] = this.centerOfPlay();
      const p = 1 - this.goT / GO_TIME;
      const k = 0.8 + 0.5 * easeOut(p);
      ctx.save();
      ctx.translate(cx, cy);
      ctx.scale(k, k);
      ui.text(ctx, t("tetris.go"), 0, 0, this.huge, ui.GREEN, "center", Math.min(1, (1 - p) * 2.5));
      ctx.restore();
    }

    drawFinishMarks(ctx) {
      const res = this.result;
      if (!res) return;
      const ft = this.state === FINISH ? this.finishT : 9;
      if (res.reason === "ko") {
        this.fx.forEach((f, i) => {
          const field = this.sides[i].field;
          if (f.koT >= 0) this.stamp(ctx, t("tetris.res.ko"), ui.RED, field, ft - 0.35);
          else if (res.winner === i) this.stamp(ctx, t("tetris.res.win"), ui.GOLD, field, ft - 0.7);
        });
      } else if ((res.reason === "sprint" || res.reason === "ultra") && this.state === FINISH) {
        const text = res.reason === "sprint" ? t("tetris.res.sprint", { n: SPRINT_LINES }) : t("tetris.res.time_up");
        this.stamp(ctx, text, ui.GOLD, this.sides[0].field, ft - 0.1);
      }
    }

    stamp(ctx, text, color, field, tt) {
      if (tt < 0) return;
      const k = 1 + 1.2 * (1 - easeOut(Math.min(1, tt / 0.25)));
      let f = this.fHero;
      const maxW = field.w * 0.9;
      if (f.width(text) > maxW) f = ui.font(Math.max(8, Math.floor((f.px * maxW) / f.width(text))), true);
      const w = f.width(text), h = f.height;
      if (tt >= 0.25) this.panel(ctx, new PG.Rect(field.centerx - w / 2 - 10, field.centery - h / 2 - 5, w + 20, h + 10), 215);
      ctx.save();
      ctx.translate(field.centerx, field.centery);
      ctx.scale(k, k);
      ui.text(ctx, text, 0, 0, f, color, "center", Math.min(1, tt / 0.15));
      ctx.restore();
    }

    // ----- Ergebnis ------------------------------------------------------------
    resultContent() {
      const res = this.result;
      const st = res.stats;
      if (!this.versus) {
        const s0 = st[0];
        const rows = [];
        let badge = null, title, color;
        if (res.reason === "sprint") {
          title = t("tetris.res.sprint", { n: SPRINT_LINES });
          color = ui.GOLD;
          rows.push([t("tetris.stat.time"), fmtTime(res.time), ui.TEXT]);
          if (res.newBest) badge = t("tetris.res.new_best_time");
          else if (res.prevBest !== null) rows.push([t("tetris.stat.best"), fmtTime(res.prevBest), ui.TEXT_DIM]);
        } else if (res.reason === "ultra") {
          title = t("tetris.res.time_up");
          color = ui.GOLD;
          rows.push([t("tetris.stat.score"), String(s0.score), ui.TEXT]);
          if (res.newBest) badge = t("tetris.res.new_best");
          else if (res.prevBest) rows.push([t("tetris.stat.best"), String(res.prevBest), ui.TEXT_DIM]);
        } else {
          title = t("common.game_over");
          color = ui.RED;
          if (this.variant !== "sprint") rows.push([t("tetris.stat.score"), String(s0.score), ui.TEXT]);
          rows.push([t("tetris.stat.time"), fmtTime(res.time), ui.TEXT]);
        }
        rows.push([t("tetris.stat.lines"), String(s0.lines), ui.TEXT]);
        if (this.variant === "marathon") rows.push([t("tetris.stat.level"), String(s0.level), ui.TEXT]);
        rows.push([t("tetris.stat.pps"), s0.pps.toFixed(2), ui.TEXT]);
        return { title, color, rows, badge, table: null };
      }
      const w = res.winner;
      let title, color;
      if (w === null) {
        title = t("common.draw");
        color = ui.TEXT_DIM;
      } else {
        title = w === 0 ? t("tetris.res.you_win") : t("tetris.res.ai_wins");
        color = w === 0 ? COL_P1 : COL_P2;
      }
      const table = [
        [t("tetris.stat.lines"), String(st[0].lines), String(st[1].lines)],
        [t("tetris.stat.attack"), String(st[0].attack), String(st[1].attack)],
        [t("tetris.stat.pps"), st[0].pps.toFixed(2), st[1].pps.toFixed(2)],
        [t("tetris.stat.tetrises"), String(st[0].tetrises), String(st[1].tetrises)],
        [t("tetris.stat.tspins"), String(st[0].tspins), String(st[1].tspins)],
      ];
      return { title, color, rows: [], badge: this.wins[0] + " : " + this.wins[1], table };
    }

    resultsLayout() {
      const content = this.resultContent();
      const W = this.width, H = this.height;
      const rowH = this.small.height + 4;
      const btnH = Math.max(28, Math.min(42, Math.floor(H / 14)));
      const nRows = content.table ? content.table.length + 1 : content.rows.length;
      const h = 18 + this.big.height + 8 + (content.badge ? this.smallB.height + 6 : 0) + nRows * rowH + 12 + btnH + 8 + this.tiny.height + 14;
      const pw = Math.min(W - 30, Math.max(300, Math.floor(W * 0.46)));
      const reserve = this.showHighscoreBanner ? 46 : 0;
      const panel = new PG.Rect(0, 0, pw, h);
      panel.center = [Math.floor(W / 2), Math.floor((H - reserve) / 2)];
      if (panel.top < 6) panel.top = 6;
      const bw = Math.floor((pw - 3 * 14) / 2);
      const by = panel.bottom - 14 - this.tiny.height - 8 - btnH;
      const again = new PG.Rect(panel.x + 14, by, bw, btnH);
      const setup = new PG.Rect(again.right + 14, by, bw, btnH);
      return Object.assign(content, { panel, again, setup, rowH });
    }

    drawResults(ctx) {
      draw.rect(ctx, [8, 10, 16, 140], [0, 0, this.width, this.height]);
      const lay = this.resultsLayout();
      const panel = lay.panel;
      ui.drawPanel(ctx, panel);
      draw.rect(ctx, this.accent, [panel.x + 14, panel.y, panel.w - 28, 2]);
      const cx = panel.centerx;
      let y = panel.y + 18;
      this.fitText(ctx, lay.title, cx, y, this.big, lay.color, this.width - 70, "midtop");
      y += this.big.height + 8;
      if (lay.badge) {
        const col = ui.mix(ui.GOLD, [255, 255, 255], ui.pulse(4, 0, 0.5));
        this.fitText(ctx, lay.badge, cx, y, this.smallB, col, panel.w - 24, "midtop");
        y += this.smallB.height + 6;
      }
      const left = panel.x + 24;
      const right = panel.right - 24;
      if (!lay.table) {
        for (const [label, value, col] of lay.rows) {
          ui.text(ctx, label, left, y, this.small, ui.TEXT_DIM);
          ui.text(ctx, value, right, y, this.smallB, col, "topright");
          y += lay.rowH;
        }
      } else {
        const head = [[t("tetris.player.you"), COL_P1], [t("common.ai"), COL_P2]];
        let colw = 0;
        for (const [h] of head) colw = Math.max(colw, this.smallB.width(h));
        for (const [, a, b] of lay.table) colw = Math.max(colw, this.smallB.width(a), this.smallB.width(b));
        colw += 22;
        const c2 = right, c1 = c2 - colw;
        head.forEach(([text, col], k) => ui.text(ctx, text, k === 0 ? c1 : c2, y, this.smallB, col, "topright"));
        y += lay.rowH;
        for (const [label, v1, v2] of lay.table) {
          this.fitText(ctx, label, left, y, this.small, ui.TEXT_DIM, c1 - left - this.smallB.width(v1) - 10);
          ui.text(ctx, v1, c1, y, this.smallB, ui.TEXT, "topright");
          ui.text(ctx, v2, c2, y, this.smallB, ui.TEXT, "topright");
          y += lay.rowH;
        }
      }
      const ready = nowSec() - this.overAt >= OVER_LOCK;
      this.overRects = { again: lay.again, setup: lay.setup };
      ui.drawButton(ctx, lay.again, t(this.versus ? "tetris.res.rematch" : "tetris.res.again"), this.small, ready, { accent: this.accent });
      ui.drawButton(ctx, lay.setup, t("tetris.res.setup"), this.small, false, { accent: this.accent });
      this.fitText(ctx, t("tetris.res.hint"), cx, panel.bottom - 10, this.tiny, ui.TEXT_FAINT, panel.w - 20, "midbottom");
    }

    // ----- Setup zeichnen --------------------------------------------------------
    drawSetup(ctx) {
      const W = this.width, H = this.height;
      const cx = W / 2;
      const dc = Math.max(10, Math.floor(H / 20));
      for (const [kind, fx, fy] of this.deco) this.drawPiece(ctx, kind, dc, fx * (W - 3 * dc) + 2 * dc, fy * H + dc, 0.13);

      ui.gradText(ctx, "TETRIS", cx, this.setupTitleY, this.huge, [250, 250, 255], ui.mix(this.accent, [255, 255, 255], 0.2), "center");
      this.fitText(ctx, t("tetris.subtitle." + this.mode), cx, this.setupSubY, this.small, ui.TEXT_DIM, W - 30, "center");

      const items = this.setupItems();
      const focus = items[Math.max(0, Math.min(items.length - 1, this.setupFocus))];
      const rects = this.setupRects;
      const lbl = (text, x, y, w, active) => this.fitText(ctx, text, x, y, this.tiny, active ? ui.TEXT : ui.TEXT_DIM, w, "midbottom");

      const chs = rects.choice;
      const full = chs[0].union(chs[2]);
      let names, cur, desc, best;
      if (this.mode === "solo") {
        names = VARIANTS.map((v) => t("tetris.variant." + v));
        cur = VARIANTS.indexOf(this.variant);
        lbl(t("tetris.lbl.variant"), full.centerx, chs[0].top - 3, full.w, focus === "variant");
        desc = t("tetris.variant_desc." + this.variant);
        best = this.bestText();
      } else {
        names = [0, 1, 2].map((k) => t("tetris.ai." + k));
        cur = this.aiLevel;
        lbl(t("tetris.lbl.ai"), full.centerx, chs[0].top - 3, full.w, focus === "ai");
        desc = t("tetris.ai_desc." + this.aiLevel);
        const wl = this.best.vs[this.aiLevel];
        best = t("tetris.vs_record", { w: wl[0], l: wl[1] });
      }
      chs.forEach((r, k) => this.optButton(ctx, r, names[k], k === cur, (focus === "variant" || focus === "ai") && k === cur));
      this.fitText(ctx, desc, cx, this.setupDescY, this.tiny, ui.TEXT, full.w, "midtop");
      this.fitText(ctx, best, cx, this.setupDescY + this.tiny.height + 3, this.tiny, ui.GOLD, full.w, "midtop");

      const optNames = ["level", "ghost", "das", "arr"].filter((n) => rects[n]);
      for (const name of optNames) {
        const r = rects[name];
        const active = focus === name;
        lbl(t("tetris.lbl." + name), r.centerx, r.top - 3, r.w + 6, active);
        if (name === "ghost") {
          this.optButton(ctx, r, this.ghost ? t("common.on") : t("common.off"), this.ghost, active);
        } else {
          const enabled = !(name === "level" && this.variant !== "marathon");
          let val;
          if (name === "level") val = enabled ? String(this.startLevel) : "1";
          else val = t("tetris.ms", { n: name === "das" ? this.das : this.arr });
          this.stepper(ctx, r, val, active, enabled);
        }
      }
      const helpKey = { level: "tetris.help.level", ghost: "tetris.help.ghost", das: "tetris.help.das", arr: "tetris.help.arr" }[focus] || "tetris.help.das";
      const optFull = rects[optNames[0]].union(rects[optNames[optNames.length - 1]]);
      this.fitText(ctx, t(helpKey), cx, this.setupHelpY, this.tiny, ui.TEXT_FAINT, optFull.w, "midtop");

      const start = rects.start;
      if (focus === "start") {
        draw.rect(ctx, ui.mix(this.accent, [255, 255, 255], ui.pulse(3, 0, 0.5)), start.inflate(8, 8), 2, 12);
      }
      ui.drawButton(ctx, start, t("common.start"), this.smallB, true, { accent: this.accent });

      const [y1, y2] = this.setupFooterY;
      this.fitText(ctx, t("tetris.setup_hint"), cx, y1, this.tiny, ui.TEXT_FAINT, W - 20, "midtop");
      this.fitText(ctx, t("tetris.keys_solo"), cx, y2, this.tiny, ui.mix(this.accent, ui.TEXT, 0.45), W - 20, "midtop");
    }

    bestText() {
      if (this.variant === "sprint") {
        return this.best.sprint === null ? t("tetris.best_none") : t("tetris.best_time", { time: fmtTime(this.best.sprint) });
      }
      if (this.variant === "ultra") {
        return this.best.ultra ? t("tetris.best_score", { score: this.best.ultra }) : t("tetris.best_none");
      }
      const hs = this.highscore;
      return hs ? t("tetris.best_score", { score: hs }) : t("tetris.best_none");
    }

    optButton(ctx, r, text, on, focus) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 8);
      draw.rect(ctx, on || focus ? this.accent : ui.BORDER, r, focus ? 2 : 1, 8);
      if (focus) draw.rect(ctx, ui.mix(this.accent, [255, 255, 255], ui.pulse(3, 0, 0.4)), r.inflate(4, 4), 1, 10);
      this.fitText(ctx, text, r.centerx, r.centery, on ? this.smallB : this.small, on ? ui.TEXT : ui.TEXT_DIM, r.w - 10, "center");
    }

    stepper(ctx, r, text, focus, enabled) {
      draw.rect(ctx, ui.BTN, r, 0, 8);
      draw.rect(ctx, focus ? this.accent : ui.BORDER, r, focus ? 2 : 1, 8);
      const aw = Math.max(3, Math.floor(r.h / 8));
      const arrow = focus && enabled ? this.accent : ui.TEXT_DIM;
      for (const sx of [-1, 1]) {
        const ax = r.centerx + sx * (r.w / 2 - aw - 5);
        const tip = ax + (sx * aw) / 2, back = ax - (sx * aw) / 2;
        draw.polygon(ctx, arrow, [[tip, r.centery], [back, r.centery - aw], [back, r.centery + aw]]);
      }
      this.fitText(ctx, text, r.centerx, r.centery, focus ? this.smallB : this.small, enabled ? ui.TEXT : ui.TEXT_FAINT, r.w - 4 * aw - 22, "center");
    }
  }

  PG.register(TetrisGame, {
    id: "TetrisGame",
    key: "tetris",
    name: "Tetris",
    modes: [["solo", "tetris.mode.solo"], ["versus_ai", "tetris.mode.versus_ai"]],
    settingsKey: "tetris",
    defaults: { das: 170, arr: 50, ghost: true, solo: "marathon", ai_level: 1, start_level: 1 },
  });
})();
