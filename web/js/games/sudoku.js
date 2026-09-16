/*
 * sudoku.js - Sudoku in vier Varianten + Tages-Sudoku (Port von games/sudoku.py + sudoku_draw.py)
 * ===============================================================================================
 * Varianten (Setup-Screen)
 * - Klassisch : 9x9 - die 400 bekannten Level (alter Generator, unverändert).
 * - X-Sudoku  : zusätzlich enthalten beide Diagonalen jede Ziffer einmal.
 * - Killer    : Käfige mit Summen, vorab erzeugt (sudoku_killer.js).
 * - Mini 6x6  : 2x3-Blöcke, Ziffern 1-6.
 * - Tages-Sudoku: ein klassisches Rätsel pro Tag (seedrand, am PC identisch),
 *   Stufe je Wochentag, eigene Serie.
 *
 * Speicher (Abschnitt "sudoku"): solved (klassisch, wie bisher), best
 * {variante: {stufe: {level: [sterne, sekunden]}}}, saves (angefangene
 * Rätsel), last (Levelcursor der neuen Varianten), daily (Serie).
 *
 * Sterne: 1 = gelöst, 2 = fehlerfrei ohne Tipps, 3 = zusätzlich unter Zielzeit.
 *
 * Spielmodi: classic (x2,0) / notes (x1,5, + Notizen, Kandidaten) /
 * comfort (x1,0, + Fehler rot, Konflikte, Einrasten, Käfigsummen) /
 * assist (x0,7, + 3 Tipps). Fortgesetzt in anderem Modus: kleinster
 * Multiplikator gilt.
 *
 * Steuerung: Pfeile/WASD = Zelle, 1-9 = Ziffer (Ziffer zuerst: wählen, dann
 * Klick/Leertaste), 0/Backspace/Entf/Rechtsklick = radieren, N = Notizen,
 * C = Kandidaten, U/Z = rückgängig, Y = wiederholen, M = Farbmarker,
 * H = Tipp, R = neu, Q = Levelwahl.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const gen = PG.sudokuGen;
  const Rect = PG.Rect;
  const fl = Math.floor;

  const COL_USER = [110, 165, 255]; // eigene Ziffern (klassische Sudoku-Optik)
  const MARK_COLORS = [[232, 92, 92], [240, 150, 60], [228, 204, 70], [96, 196, 118], [84, 160, 240], [176, 116, 226]];

  const VARIANTS = gen.VARIANTS;
  const DIFFICULTIES = [["easy", 1000], ["normal", 2000], ["hard", 3500], ["expert", 5000]];
  const BASE_POINTS = {
    classic: [1000, 2000, 3500, 5000],
    x: [1200, 2400, 4000, 5600],
    killer: [1500, 3000, 5000, 7000],
    mini: [500, 900, 1400, 2000],
  };
  const TARGET_TIME = {
    classic: [300, 540, 840, 1200],
    x: [360, 600, 900, 1320],
    killer: [600, 960, 1440, 1920],
    mini: [90, 150, 240, 360],
  };
  const MODE_MULT = { classic: 2.0, notes: 1.5, comfort: 1.0, assist: 0.7 };

  const MAX_HINTS = 3;
  const HINT_COST = 200;
  const ERR_COST = 150;
  const TIME_COST = 2;
  const FAIL_LIMIT = 3;
  const MARK_COUNT = 6;
  const MAX_SAVES = 40;
  const SAVE_EVERY = 4.0;
  const MAX_UNDO = 500;

  const PAD_TIPS = { erase: "sud.tip.erase", undo: "sud.tip.undo", redo: "sud.tip.redo", note: "sud.tip.note", cands: "sud.tip.cands", hint: "sud.tip.hint" };
  const PAD_KEYS = { erase: "0", undo: "U", redo: "Y", note: "N", cands: "C", hint: "H" };

  const SETUP = "setup", GENERATING = "generating", PLAY = "play";
  const MOVE = {
    Up: [-1, 0], w: [-1, 0], W: [-1, 0], Down: [1, 0], s: [1, 0], S: [1, 0],
    Left: [0, -1], a: [0, -1], A: [0, -1], Right: [0, 1], d: [0, 1], D: [0, 1],
  };

  function fmtTime(sec) {
    sec = Math.max(0, Math.trunc(sec));
    const p2 = (n) => String(n).padStart(2, "0");
    if (sec >= 3600) return fl(sec / 3600) + ":" + p2(fl(sec / 60) % 60) + ":" + p2(sec % 60);
    return p2(fl(sec / 60)) + ":" + p2(sec % 60);
  }

  function starsFor(variant, diff, secs, errors, hints) {
    const clean = errors === 0 && hints === 0;
    const fast = secs <= TARGET_TIME[variant][Math.max(0, Math.min(3, diff))];
    return 1 + (clean ? 1 : 0) + (clean && fast ? 1 : 0);
  }

  function starPoints(cx, cy, r) {
    const pts = [];
    for (let k = 0; k < 10; k++) {
      const rad = k % 2 === 0 ? r : r * 0.45;
      const a = -Math.PI / 2 + (k * Math.PI) / 5;
      pts.push([cx + Math.cos(a) * rad, cy + Math.sin(a) * rad]);
    }
    return pts;
  }
  function drawStar(ctx, cx, cy, r, color, filled = true) {
    draw.polygon(ctx, color, starPoints(cx, cy, r), filled ? 0 : Math.max(1, fl(r / 5)));
  }
  function drawCheck(ctx, cx, cy, size, color, width = 2) {
    draw.lines(ctx, color, false, [[cx - size, cy], [cx - size * 0.35, cy + size * 0.6], [cx + size, cy - size * 0.7]], width);
  }
  const isInt = (v) => Number.isInteger(v);
  const clampLvl = (n) => Math.max(1, Math.min(100, n));

  class SudokuGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.won = false;

      const o = this.opts;
      this.diff = Math.max(0, Math.min(3, Math.trunc(Number(o.difficulty) || 0)));
      this.failLimit = o.fail_limit !== false;
      this.lastLevel = Object.assign({}, o.last_level && typeof o.last_level === "object" ? o.last_level : {});
      this.variant = VARIANTS.includes(o.variant) ? o.variant : "classic";
      this.inputMode = o.input === "digit" ? "digit" : "cell";
      this.colorsOn = o.colors === true;

      this.canNotes = ["notes", "comfort", "assist"].includes(this.mode);
      this.canCheck = ["comfort", "assist"].includes(this.mode);
      this.canHint = this.mode === "assist";

      this.daily = false;
      this.dailyDate = PG.seedrand.todayStr();
      this.playVariant = this.variant;
      this.playDiff = this.diff;
      this.level = 1;
      this.lay = gen.layoutFor("classic");
      this.fresh = false;
      this.hover = null;
      this.msg = null;
      this.msgT = 0;
      this.msgCol = null;
      this.genDrawn = false;
      this.cagePath = null;

      this.buildFonts();
      this.loadProgress();
      this.cursor = this.getLast(this.variant, this.diff);
      this.buildSetupLayout();
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, fl(h / 30)));
      this.tiny = ui.font(Math.max(11, fl(h / 36)));
      this.huge = ui.font(Math.max(26, fl(h / 11)), true);
      this.titleFont = ui.font(Math.max(24, fl(h / 13)), true);
      this.clockFont = ui.font(Math.max(18, fl(h / 22)), false, true);
    }

    destroy() {
      if (this.state === PLAY && !this.gameOver) this.writeSave();
    }

    // ===================================================== Speicher
    loadProgress() {
      const data = PG.store.get("sudoku", {}) || {};
      const known = ["solved", "best", "saves", "last", "daily"];
      this.progExtra = {};
      for (const k in data) if (!known.includes(k)) this.progExtra[k] = data[k];

      const solved = data.solved && typeof data.solved === "object" ? data.solved : {};
      this.solved = {};
      for (const k of ["0", "1", "2", "3"]) {
        const lst = solved[k];
        this.solved[k] = Array.isArray(lst)
          ? Array.from(new Set(lst.filter((v) => isInt(v) && v >= 1 && v <= gen.LEVELS))).sort((a, b) => a - b)
          : [];
      }

      const rawBest = data.best && typeof data.best === "object" ? data.best : {};
      this.best = {};
      for (const v of VARIANTS) {
        const vb = rawBest[v] && typeof rawBest[v] === "object" ? rawBest[v] : {};
        const outV = {};
        for (const d of ["0", "1", "2", "3"]) {
          const db = vb[d] && typeof vb[d] === "object" ? vb[d] : {};
          const out = {};
          for (const key in db) {
            const lvl = Number(key), rec = db[key];
            if (!Array.isArray(rec) || !isInt(lvl) || lvl < 1 || lvl > gen.LEVELS) continue;
            const stars = Math.trunc(Number(rec[0])), secs = Math.trunc(Number(rec[1]));
            if (!Number.isFinite(stars) || !Number.isFinite(secs)) continue;
            out[String(lvl)] = [Math.max(1, Math.min(3, stars)), Math.max(0, secs)];
          }
          if (Object.keys(out).length) outV[d] = out;
        }
        this.best[v] = outV;
      }

      const saves = data.saves && typeof data.saves === "object" ? data.saves : {};
      this.saves = {};
      for (const k in saves) if (saves[k] && typeof saves[k] === "object" && !Array.isArray(saves[k])) this.saves[k] = saves[k];

      const rawLast = data.last && typeof data.last === "object" ? data.last : {};
      this.last = {};
      for (const v of VARIANTS) {
        const lv = rawLast[v] && typeof rawLast[v] === "object" ? rawLast[v] : {};
        this.last[v] = {};
        for (const d in lv) if (["0", "1", "2", "3"].includes(d) && isInt(lv[d])) this.last[v][d] = clampLvl(lv[d]);
      }

      const dl = data.daily && typeof data.daily === "object" ? data.daily : {};
      this.dailyState = {};
      if (typeof dl.date === "string" && dl.date.length === 10) this.dailyState.date = dl.date;
      for (const k of ["streak", "best", "time", "stars", "count"]) if (isInt(dl[k])) this.dailyState[k] = Math.max(0, dl[k]);
    }

    saveProgress() {
      const data = Object.assign({}, this.progExtra, {
        solved: this.solved, best: this.best, saves: this.saves, last: this.last, daily: this.dailyState,
      });
      PG.store.set("sudoku", data);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    getLast(variant, diff) {
      const n = variant === "classic" ? this.lastLevel[String(diff)] : (this.last[variant] || {})[String(diff)];
      return isInt(n) ? clampLvl(n) : 1;
    }

    setLast(variant, diff, level) {
      if (variant === "classic") {
        this.lastLevel[String(diff)] = level;
        this.saveSetting("last_level", Object.assign({}, this.lastLevel));
      } else {
        (this.last[variant] = this.last[variant] || {})[String(diff)] = level;
      }
    }

    record(variant, diff, level) {
      const rec = ((this.best[variant] || {})[String(diff)] || {})[String(level)];
      if (rec) return [rec[0], rec[1]];
      if (variant === "classic" && (this.solved[String(diff)] || []).includes(level)) return [1, null];
      return [0, null];
    }

    stageSummary(variant, diff) {
      let n = 0, stars = 0;
      for (let lvl = 1; lvl <= gen.LEVELS; lvl++) {
        const [s] = this.record(variant, diff, lvl);
        if (s) {
          n++;
          stars += s;
        }
      }
      return [n, stars];
    }

    threeStarCount() {
      let n = 0;
      for (const v in this.best) for (const d in this.best[v]) for (const l in this.best[v][d]) if (this.best[v][d][l][0] >= 3) n++;
      return n;
    }

    saveKey() {
      return this.daily ? "daily:" + this.dailyDate : this.playVariant + ":" + this.playDiff + ":" + this.level;
    }

    levelSave(variant, diff, level) {
      return this.saves[variant + ":" + diff + ":" + level] || null;
    }

    dailyStreak() {
      const last = this.dailyState.date;
      if (!last) return 0;
      const gap = PG.seedrand.dayIndex(this.dailyDate) - PG.seedrand.dayIndex(last);
      return gap === 0 || gap === 1 ? this.dailyState.streak || 0 : 0;
    }

    dailyDoneToday() {
      return this.dailyState.date === this.dailyDate;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const W = this.width, H = this.height;
      const cx = fl(W / 2);
      const m = Math.max(10, fl(W / 48));
      const gap = Math.max(6, fl(H / 90));
      this.suTitleY = Math.max(20, fl(H * 0.06));
      this.suSubY = this.suTitleY + fl(this.titleFont.height / 2) + fl(this.tiny.height / 2) + 1;
      const rowW = W < 900 ? W - 2 * m : fl(W * 0.8);
      const left = cx - fl(rowW / 2);
      const bh = Math.max(26, Math.min(44, fl(H / 15)));
      const y0 = this.suSubY + fl(this.tiny.height / 2) + gap + 2;
      const row = (y, n) => {
        const cw = (rowW - gap * (n - 1)) / n;
        const out = [];
        for (let i = 0; i < n; i++) out.push(new Rect(Math.trunc(left + i * (cw + gap)), y, Math.trunc(cw), bh));
        return out;
      };
      this.varRects = row(y0, 4);
      this.diffRects = row(y0 + bh + gap, 4);
      let bodyTop = y0 + 2 * bh + 3 * gap;
      const foot = this.tiny.height + 10;
      const progH = this.small.height + 6;
      const availH = H - foot - bodyTop - progH;
      const grid = Math.min(availH, Math.trunc(rowW * 0.57));
      const cell = Math.max(18, fl(grid / 10));
      bodyTop += Math.max(0, fl((availH - 10 * cell) / 3));
      this.lvCell = cell;
      this.lvX = left;
      this.lvY = bodyTop;
      this.lvFont = ui.font(Math.max(10, fl((cell * 2) / 5)), false, true);
      this.progY = bodyTop + 10 * cell + fl(progH / 2) + 1;

      const px = left + 10 * cell + 2 * gap;
      const pw = left + rowW - px;
      this.panelRect = new Rect(px, bodyTop, pw, 10 * cell);
      const pad = Math.max(6, gap);
      const ix = px + pad, iw = pw - 2 * pad;
      const dh = this.small.height + 2 * this.tiny.height + 12;
      this.dailyRect = new Rect(ix, bodyTop + pad, iw, dh);
      const th = Math.max(22, Math.min(34, fl(H / 22)));
      const ty = this.dailyRect.bottom + gap;
      this.limitRect = new Rect(ix, ty, iw, th);
      this.inputRect = new Rect(ix, ty + th + 4, iw, th);
      this.colorsRect = new Rect(ix, ty + 2 * (th + 4), iw, th);
      const infoTop = this.colorsRect.bottom + gap;
      this.infoRect = new Rect(ix, infoTop, iw, Math.max(0, this.panelRect.bottom - pad - infoTop));
      const need = th + 4 + this.small.height + 2 * this.tiny.height + 8;
      if (this.infoRect.h >= need) {
        const bh2 = Math.max(th, Math.min(40, fl(H / 18)));
        this.startRect = new Rect(ix, this.infoRect.bottom - bh2, iw, bh2);
        this.infoRect = new Rect(ix, infoTop, iw, this.infoRect.h - bh2 - 6);
      } else {
        this.startRect = null;
      }
    }

    levelAt(pos) {
      const c = fl((pos[0] - this.lvX) / this.lvCell);
      const r = fl((pos[1] - this.lvY) / this.lvCell);
      if (c >= 0 && c < 10 && r >= 0 && r < 10) return r * 10 + c + 1;
      return null;
    }

    selectVariant(v) {
      if (!VARIANTS.includes(v)) return;
      this.variant = v;
      this.saveSetting("variant", v);
      this.cursor = this.getLast(v, this.diff);
      this.playSound("click");
    }

    selectDifficulty(i) {
      this.diff = Math.max(0, Math.min(3, i));
      this.saveSetting("difficulty", this.diff);
      this.cursor = this.getLast(this.variant, this.diff);
      this.playSound("click");
    }

    toggleFailLimit() {
      this.failLimit = !this.failLimit;
      this.saveSetting("fail_limit", this.failLimit);
      this.playSound("select");
    }

    toggleInput() {
      this.inputMode = this.inputMode === "digit" ? "cell" : "digit";
      this.saveSetting("input", this.inputMode);
      this.playSound("select");
    }

    toggleColors() {
      this.colorsOn = !this.colorsOn;
      this.saveSetting("colors", this.colorsOn);
      this.playSound("select");
    }

    setupHit(pos) {
      for (let i = 0; i < 4; i++) if (this.varRects[i].collidepoint(pos)) return ["variant", i];
      for (let i = 0; i < 4; i++) if (this.diffRects[i].collidepoint(pos)) return ["diff", i];
      const items = [["daily", this.dailyRect], ["limit", this.limitRect], ["input", this.inputRect], ["colors", this.colorsRect], ["start", this.startRect]];
      for (const [key, r] of items) if (r && r.collidepoint(pos)) return [key, 0];
      return null;
    }

    isHover(key, i = 0) {
      return !!this.hover && Array.isArray(this.hover) && this.hover[0] === key && this.hover[1] === i;
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4"].includes(k)) this.selectDifficulty(Number(k) - 1);
        else if ((k === "v" || k === "V") && this.keyIsFree(k)) {
          const step = k === "V" ? -1 : 1;
          this.selectVariant(VARIANTS[PG.mod(VARIANTS.indexOf(this.variant) + step, VARIANTS.length)]);
        } else if ((k === "f" || k === "F") && this.keyIsFree(k)) this.toggleFailLimit();
        else if ((k === "i" || k === "I") && this.keyIsFree(k)) this.toggleInput();
        else if ((k === "m" || k === "M") && this.keyIsFree(k)) this.toggleColors();
        else if ((k === "t" || k === "T") && this.keyIsFree(k)) this.startDaily();
        else if (["Left", "a", "A"].includes(k)) {
          this.cursor = PG.mod(this.cursor - 2, 100) + 1;
          this.playSound("move");
        } else if (["Right", "d", "D"].includes(k)) {
          this.cursor = PG.mod(this.cursor, 100) + 1;
          this.playSound("move");
        } else if (["Up", "w", "W"].includes(k)) {
          this.cursor = PG.mod(this.cursor - 11, 100) + 1;
          this.playSound("move");
        } else if (["Down", "s", "S"].includes(k)) {
          this.cursor = PG.mod(this.cursor + 9, 100) + 1;
          this.playSound("move");
        } else if (k === "Return" || k === "space" || k === "KP_Enter") {
          this.startLevel(this.cursor);
        }
      } else if (ev.kind === "mousemove") {
        const lv = this.levelAt(ev.pos);
        if (lv !== null) this.cursor = lv;
        this.hover = this.setupHit(ev.pos);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const hit = this.setupHit(ev.pos);
        if (hit) {
          const [kind, i] = hit;
          if (kind === "variant") this.selectVariant(VARIANTS[i]);
          else if (kind === "diff") this.selectDifficulty(i);
          else if (kind === "daily") this.startDaily();
          else if (kind === "limit") this.toggleFailLimit();
          else if (kind === "input") this.toggleInput();
          else if (kind === "colors") this.toggleColors();
          else if (kind === "start") this.startLevel(this.cursor);
          return;
        }
        const lv = this.levelAt(ev.pos);
        if (lv !== null) this.startLevel(lv);
      }
    }

    // ===================================================== Level starten
    beginGenerate() {
      this.gameOver = false;
      this.won = false;
      this.score = 0;
      this.reveal = false;
      this.hover = null;
      this.genDrawn = false;
      this.state = GENERATING;
      this.playSound("click");
    }

    startLevel(n, fresh = false) {
      if (this.variant === "killer" && !gen.killerLevel(this.diff, 1)) {
        this.say(t("sud.killer_missing"), ui.RED);
        this.playSound("hit");
        return;
      }
      this.daily = false;
      this.playVariant = this.variant;
      this.playDiff = this.diff;
      this.level = clampLvl(Math.trunc(n));
      this.cursor = this.level;
      this.setLast(this.variant, this.diff, this.level);
      this.fresh = fresh;
      this.beginGenerate();
    }

    startDaily(fresh = false) {
      this.daily = true;
      this.dailyDate = PG.seedrand.todayStr();
      this.playVariant = "classic";
      this.playDiff = gen.dailyDiff(this.dailyDate);
      this.level = 0;
      this.fresh = fresh;
      this.beginGenerate();
    }

    restart() {
      if (this.daily) this.startDaily(true);
      else {
        this.variant = this.playVariant;
        this.diff = this.playDiff;
        this.startLevel(this.level, true);
      }
    }

    backToSetup() {
      if (this.state === PLAY && !this.gameOver) this.writeSave();
      this.state = SETUP;
      this.gameOver = false;
      this.hover = null;
      this.msg = null;
      this.dailyDate = PG.seedrand.todayStr();
      if (!this.daily) {
        this.variant = this.playVariant;
        this.diff = this.playDiff;
        this.cursor = this.level;
      }
      this.playSound("click");
    }

    doGenerate() {
      const v = this.playVariant;
      let puzzle, solution, cages = [];
      if (this.daily) [puzzle, solution, this.playDiff] = gen.generateDaily(this.dailyDate);
      else if (v === "classic") [puzzle, solution] = gen.generate(this.playDiff, this.level);
      else if (v === "killer") {
        const data = gen.killerLevel(this.playDiff, this.level);
        if (!data) {
          this.state = SETUP;
          this.say(t("sud.killer_missing"), ui.RED);
          return;
        }
        [puzzle, solution, cages] = data;
      } else [puzzle, solution] = gen.generateVariant(v, this.playDiff, this.level);
      const lay = (this.lay = gen.layoutFor(this.daily ? "classic" : v));
      this.N = lay.n;
      this.cells = lay.size;
      this.puzzle = puzzle.slice();
      this.solution = solution.slice();
      this.cages = cages.map(([total, cells]) => [total, cells.slice()]);
      this.cageOf = new Array(this.cells).fill(-1);
      this.cages.forEach(([, cells], ci) => cells.forEach((i) => (this.cageOf[i] = ci)));
      this.peers = [];
      for (let i = 0; i < this.cells; i++) {
        const ps = new Set(lay.peers[i]);
        if (this.cageOf[i] >= 0) {
          for (const j of this.cages[this.cageOf[i]][1]) ps.add(j);
          ps.delete(i);
        }
        this.peers.push(Array.from(ps).sort((a, b) => a - b));
      }
      this.groups = lay.units.map((u) => u.slice()).concat(this.cages.map(([, c]) => c));
      this.groupsOf = Array.from({ length: this.cells }, () => []);
      this.groups.forEach((cells, g) => cells.forEach((i) => this.groupsOf[i].push(g)));

      this.board = puzzle.slice();
      this.given = puzzle.map((x) => x !== 0);
      this.locked = this.given.slice();
      this.notes = new Array(this.cells).fill(0);
      this.marks = new Array(this.cells).fill(0);
      this.wrong = new Set();
      this.hinted = new Set();
      const firstFree = this.given.indexOf(false);
      this.sel = firstFree >= 0 ? firstFree : 0;
      this.errors = 0;
      this.hintsUsed = 0;
      this.elapsed = 0;
      this.mult = MODE_MULT[this.mode] || 1.0;
      this.noteMode = false;
      this.brush = 0;
      this.reveal = false;
      this.msg = null;
      this.msgT = 0;
      this.undoStack = [];
      this.redoStack = [];
      this.step = null;
      this.dirty = false;
      this.saveT = 0;
      this.conflicts = new Set();
      this.badCages = new Set();
      this.fxPop = new Map();
      this.fxShake = new Map();
      this.fxUnits = [];
      this.winTicks = 0;
      this.starSnd = new Set();
      this.resultStars = 0;
      this.newRecord = false;

      if (this.fresh) {
        if (this.saves[this.saveKey()]) {
          delete this.saves[this.saveKey()];
          this.saveProgress();
        }
      } else if (this.restoreSave()) {
        this.say(t("sud.resumed"), this.accent);
      }
      this.fresh = false;
      this.updateConflicts();
      this.cagePath = null;
      this.buildPlayLayout();
      this.state = PLAY;
    }

    // ----- Spielstand ----------------------------------------------------
    hasProgress() {
      return !!(this.errors || this.hintsUsed || this.notes.some((x) => x) || this.marks.some((x) => x) || this.board.some((b, i) => b !== this.puzzle[i]));
    }

    writeSave() {
      this.dirty = false;
      this.saveT = 0;
      const key = this.saveKey();
      if (!this.hasProgress()) {
        if (this.saves[key]) {
          delete this.saves[key];
          this.saveProgress();
        }
        return;
      }
      let todo = 0, filled = 0;
      for (let i = 0; i < this.cells; i++) {
        if (!this.puzzle[i]) todo++;
        if (!this.given[i] && this.board[i]) filled++;
      }
      const locked = [];
      for (let i = 0; i < this.cells; i++) if (this.locked[i] && !this.given[i]) locked.push(i);
      this.saves[key] = {
        b: this.board.join(""),
        n: this.notes.slice(),
        k: this.marks.join(""),
        l: locked,
        h: Array.from(this.hinted).sort((a, b) => a - b),
        e: this.errors,
        hu: this.hintsUsed,
        t: Math.round(this.elapsed * 10) / 10,
        m: this.mult,
        p: Math.trunc((100 * filled) / (todo || 1)),
        ts: Math.trunc(Date.now() / 1000),
      };
      const keys = Object.keys(this.saves);
      if (keys.length > MAX_SAVES) {
        const old = keys.filter((k) => k !== key).sort((a, b) => (this.saves[a].ts || 0) - (this.saves[b].ts || 0));
        for (const k of old.slice(0, keys.length - MAX_SAVES)) delete this.saves[k];
      }
      this.saveProgress();
    }

    restoreSave() {
      const raw = this.saves[this.saveKey()];
      if (!raw || typeof raw !== "object" || typeof raw.b !== "string") return false;
      const n = this.N, size = this.cells;
      const board = raw.b.split("").map(Number);
      if (board.length !== size || board.some((v) => !isInt(v) || v < 0 || v > n)) return false;
      for (let i = 0; i < size; i++) if (this.given[i] && board[i] !== this.puzzle[i]) return false;
      let notes = Array.isArray(raw.n) && raw.n.length === size ? raw.n.map((x) => (Math.trunc(Number(x)) || 0) & this.lay.all) : new Array(size).fill(0);
      let marks = typeof raw.k === "string" ? raw.k.split("").map(Number) : [];
      if (marks.length !== size || marks.some((v) => !isInt(v) || v < 0 || v > MARK_COUNT)) marks = new Array(size).fill(0);
      const locked = Array.isArray(raw.l) ? raw.l.filter((i) => isInt(i) && i >= 0 && i < size) : [];
      const hinted = Array.isArray(raw.h) ? raw.h.filter((i) => isInt(i) && i >= 0 && i < size) : [];
      const errors = Math.max(0, Math.trunc(Number(raw.e) || 0));
      const hints = Math.max(0, Math.min(MAX_HINTS, Math.trunc(Number(raw.hu) || 0)));
      const elapsed = Math.max(0, Number(raw.t) || 0);
      const mult = Number(raw.m);
      if (this.failLimit && errors >= FAIL_LIMIT) return false;
      this.board = board;
      notes = notes.map((x, i) => (board[i] ? 0 : x));
      this.notes = notes;
      this.marks = marks;
      this.wrong = new Set();
      for (let i = 0; i < size; i++) if (board[i] && board[i] !== this.solution[i]) this.wrong.add(i);
      for (const i of locked) if (board[i] && board[i] === this.solution[i]) this.locked[i] = true;
      this.hinted = new Set(hinted.filter((i) => this.locked[i] && !this.given[i]));
      this.errors = errors;
      this.hintsUsed = hints;
      this.elapsed = elapsed;
      if (mult > 0 && mult <= 2) this.mult = Math.min(this.mult, mult);
      const free = this.board.indexOf(0);
      if (free >= 0) this.sel = free;
      return true;
    }

    // ===================================================== Spiel-Layout
    buildPlayLayout() {
      const W = this.width, H = this.height, N = this.lay.n;
      this.hudH = Math.max(40, fl(H / 12));
      const foot = this.tiny.height + 12;
      const padW = Math.max(128, fl(W / 5));
      const size = Math.min(H - this.hudH - foot - 12, W - padW - 44);
      this.cell = Math.max(20, fl(size / N));
      const bs = this.cell * N;
      this.bs = bs;
      this.bx = Math.max(12, fl((W - padW - 24 - bs) / 2));
      this.by = this.hudH + Math.max(6, fl((H - this.hudH - foot - bs) / 2));

      this.numFont = ui.font(Math.max(14, fl((this.cell * 3) / 5)), true, true);
      this.noteFont = this.cages.length ? ui.font(Math.max(7, fl((this.cell * 2) / 9)), false, true) : ui.font(Math.max(8, fl((this.cell * 2) / 7)), false, true);
      this.sumFont = ui.font(Math.max(8, fl(this.cell / 4)), true);
      this.padFont = ui.font(Math.max(14, Math.min(fl((this.cell * 3) / 5), 44)), true, true);

      const px = this.bx + bs + 24;
      const pb = Math.max(24, Math.min(fl((W - px - 12 - 8) / 3), this.cell + 8));
      const fw = 3 * pb + 8;
      const fh = Math.max(22, fl((pb * 2) / 3));
      const rows = fl((N + 2) / 3);
      const funcRows = [["erase", "undo", "redo"]];
      const second = [["note", this.canNotes], ["cands", this.canNotes], ["hint", this.canHint]].filter((x) => x[1]).map((x) => x[0]);
      if (second.length) funcRows.push(second);
      const sh = this.colorsOn ? Math.max(14, fl((fh * 2) / 3)) : 0;
      const capH = 2 * this.tiny.height + 6;
      const total = rows * (pb + 4) + 4 + funcRows.length * (fh + 4) + (sh ? sh + 6 : 0) + capH;
      const py = this.by + Math.max(0, fl((bs - total) / 2));
      this.padRects = {};
      for (let d = 1; d <= N; d++) {
        const r = fl((d - 1) / 3), c = (d - 1) % 3;
        this.padRects[String(d)] = new Rect(px + c * (pb + 4), py + r * (pb + 4), pb, pb);
      }
      let y = py + rows * (pb + 4) + 4;
      for (const keys of funcRows) {
        keys.forEach((key, c) => (this.padRects[key] = new Rect(px + c * (pb + 4), y, pb, fh)));
        y += fh + 4;
      }
      if (sh) {
        y += 2;
        const sw = fl((fw - (MARK_COUNT - 1) * 3) / MARK_COUNT);
        for (let k = 0; k < MARK_COUNT; k++) this.padRects["mark" + (k + 1)] = new Rect(px + k * (sw + 3), y, sw, sh);
        y += sh + 4;
      }
      this.padX = px;
      this.padW = fw;
      this.captionY = y + 2;
      this.captionBottom = Math.min(H - foot, y + 2 + capH);
    }

    cellAt(pos) {
      const c = fl((pos[0] - this.bx) / this.cell);
      const r = fl((pos[1] - this.by) / this.cell);
      if (c >= 0 && c < this.N && r >= 0 && r < this.N) return r * this.N + c;
      return null;
    }

    padAt(pos) {
      for (const key in this.padRects) if (this.padRects[key].collidepoint(pos)) return key;
      return null;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state !== PLAY) return;
      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space" || ev.key === "KP_Enter") {
            if (this.won) this.nextLevel();
            else this.restart();
          } else if (["s", "S", "q", "Q"].includes(ev.key)) {
            this.backToSetup();
          } else if (ev.key === "a" || ev.key === "A") {
            this.reveal = !this.reveal;
            this.playSound("select");
          }
        } else if (ev.kind === "mousedown" && this.reveal) {
          this.reveal = false;
        }
        return;
      }

      if (ev.kind === "keydown") {
        this.handlePlayKey(ev.key);
      } else if (ev.kind === "mousemove") {
        this.hover = this.padAt(ev.pos);
      } else if (ev.kind === "mousedown") {
        const cell = this.cellAt(ev.pos);
        if (ev.button === 3) {
          if (cell !== null) {
            this.sel = cell;
            this.erase(cell);
          }
          return;
        }
        if (cell !== null) {
          this.sel = cell;
          if (this.inputMode === "digit" && this.brush) this.applyBrush(cell);
          else this.playSound("move");
          return;
        }
        const pad = this.padAt(ev.pos);
        if (pad === null) return;
        if (/^[1-9]$/.test(pad)) {
          const d = Number(pad);
          if (this.inputMode === "digit") this.setBrush(this.brush === d ? 0 : d);
          else this.enterDigit(this.sel, d);
        } else if (pad.startsWith("mark")) {
          this.setMark(this.sel, Number(pad.slice(4)));
        } else {
          this.padAction(pad);
        }
      }
    }

    padAction(key) {
      if (key === "erase") this.erase(this.sel);
      else if (key === "undo") this.undo();
      else if (key === "redo") this.redo();
      else if (key === "note") this.toggleNoteMode();
      else if (key === "cands") this.autoCandidates();
      else if (key === "hint") this.useHint();
    }

    handlePlayKey(k) {
      const n = this.N;
      if (k in MOVE) {
        const [dr, dc] = MOVE[k];
        const r = fl(this.sel / n), c = this.sel % n;
        this.sel = PG.mod(r + dr, n) * n + PG.mod(c + dc, n);
        this.playSound("move");
        return;
      }
      let digit = null;
      if (/^[0-9]$/.test(k)) digit = Number(k);
      else if (/^KP_[0-9]$/.test(k)) digit = Number(k.slice(3));
      if (digit !== null && digit >= 1 && digit <= n) {
        if (this.inputMode === "digit") this.setBrush(this.brush === digit ? 0 : digit);
        else this.enterDigit(this.sel, digit);
      } else if (digit === 0 || k === "BackSpace" || k === "Delete") {
        this.erase(this.sel);
      } else if (k === "space" || k === "Return" || k === "KP_Enter") {
        if (this.inputMode === "digit" && this.brush) this.applyBrush(this.sel);
      } else if (k === "n" || k === "N") {
        this.toggleNoteMode();
      } else if ((k === "c" || k === "C") && this.keyIsFree(k)) {
        this.autoCandidates();
      } else if (["u", "U", "z", "Z"].includes(k) && this.keyIsFree(k)) {
        this.undo();
      } else if ((k === "y" || k === "Y") && this.keyIsFree(k)) {
        this.redo();
      } else if ((k === "m" || k === "M") && this.keyIsFree(k)) {
        if (this.colorsOn) this.setMark(this.sel, (this.marks[this.sel] + 1) % (MARK_COUNT + 1), false);
      } else if (k === "h" || k === "H") {
        this.useHint();
      } else if (k === "r" || k === "R") {
        this.restart();
      } else if (k === "q" || k === "Q") {
        this.backToSetup();
      }
    }

    setBrush(d) {
      this.brush = d;
      this.playSound(d ? "select" : "move");
    }

    applyBrush(idx) {
      const d = this.brush;
      if (!d || this.locked[idx]) {
        this.playSound("move");
        return;
      }
      if (!this.noteMode && this.board[idx] === d) this.erase(idx);
      else this.enterDigit(idx, d);
    }

    // ===================================================== Rückgängig/Wiederholen
    cellState(i) {
      return [this.board[i], this.notes[i], this.marks[i]];
    }

    begin(primary) {
      this.step = { cell: primary, before: new Map() };
    }

    snap(cells) {
      for (const i of cells) if (!this.step.before.has(i)) this.step.before.set(i, this.cellState(i));
    }

    commit() {
      const step = this.step;
      this.step = null;
      const changes = [];
      for (const [i, before] of step.before) {
        const after = this.cellState(i);
        if (after[0] !== before[0] || after[1] !== before[1] || after[2] !== before[2]) changes.push([i, before, after]);
      }
      if (changes.length) {
        this.undoStack.push([step.cell, changes]);
        if (this.undoStack.length > MAX_UNDO) this.undoStack.shift();
        this.redoStack = [];
        this.dirty = true;
      }
    }

    applyState(i, st) {
      [this.board[i], this.notes[i], this.marks[i]] = st;
      if (this.board[i] && this.board[i] !== this.solution[i]) this.wrong.add(i);
      else this.wrong.delete(i);
    }

    undo() {
      if (this.gameOver) return;
      while (this.undoStack.length) {
        const [primary, changes] = this.undoStack.pop();
        if (primary >= 0 && this.locked[primary]) continue;
        for (const [i, before] of changes) if (!this.locked[i]) this.applyState(i, before);
        this.redoStack.push([primary, changes]);
        if (primary >= 0) this.sel = primary;
        this.updateConflicts();
        this.dirty = true;
        this.playSound("rotate");
        return;
      }
      this.playSound("move");
    }

    redo() {
      if (this.gameOver) return;
      while (this.redoStack.length) {
        const [primary, changes] = this.redoStack.pop();
        if (primary >= 0 && this.locked[primary]) continue;
        for (const [i, , after] of changes) {
          if (this.locked[i]) continue;
          this.applyState(i, after);
          if (this.canCheck && this.board[i] && this.board[i] === this.solution[i]) this.locked[i] = true;
        }
        this.undoStack.push([primary, changes]);
        if (primary >= 0) this.sel = primary;
        this.updateConflicts();
        this.dirty = true;
        this.playSound("rotate");
        this.checkWin();
        return;
      }
      this.playSound("move");
    }

    // ===================================================== Spiellogik
    toggleNoteMode() {
      if (!this.canNotes) return;
      this.noteMode = !this.noteMode;
      this.playSound("select");
    }

    enterDigit(idx, d) {
      if (this.locked[idx] || d < 1 || d > this.N) return;
      if (this.noteMode) {
        this.toggleNote(idx, d);
        return;
      }
      if (this.board[idx] === d) return;
      this.begin(idx);
      this.snap([idx].concat(this.peers[idx]));
      this.board[idx] = d;
      this.notes[idx] = 0;
      if (this.canNotes) this.pruneNotes(idx, d);
      const now = ui.ticks();
      if (d === this.solution[idx]) {
        this.wrong.delete(idx);
        if (this.canCheck) this.locked[idx] = true;
        this.commit();
        this.fxPop.set(idx, now);
        this.updateConflicts();
        this.playSound(this.checkUnits(idx, now) ? "line" : "select");
        this.checkWin();
      } else {
        this.wrong.add(idx);
        this.errors++;
        this.commit();
        this.fxShake.set(idx, now);
        this.playSound("hit");
        this.rumble(120);
        this.updateConflicts();
        if (this.failLimit && this.errors >= FAIL_LIMIT) this.lose();
        else this.checkWin();
      }
    }

    toggleNote(idx, d) {
      if (this.board[idx] || !this.canNotes) return;
      this.begin(idx);
      this.snap([idx]);
      this.notes[idx] ^= 1 << (d - 1);
      this.commit();
      this.playSound("move");
    }

    erase(idx) {
      if (this.locked[idx]) return;
      if (this.board[idx] || this.notes[idx] || this.marks[idx]) {
        this.begin(idx);
        this.snap([idx]);
        if (this.board[idx] || this.notes[idx]) {
          this.board[idx] = 0;
          this.notes[idx] = 0;
        } else {
          this.marks[idx] = 0;
        }
        this.wrong.delete(idx);
        this.commit();
        this.updateConflicts();
        this.playSound("move");
      }
    }

    setMark(idx, color, toggle = true) {
      if (!this.colorsOn || color < 0 || color > MARK_COUNT) return;
      if (toggle && this.marks[idx] === color) color = 0;
      this.begin(idx);
      this.snap([idx]);
      this.marks[idx] = color;
      this.commit();
      this.playSound("select");
    }

    pruneNotes(idx, d) {
      const m = ~(1 << (d - 1));
      for (const j of this.peers[idx]) this.notes[j] &= m;
    }

    candidates(i) {
      let used = 0;
      for (const j of this.peers[i]) if (this.board[j]) used |= 1 << (this.board[j] - 1);
      return this.lay.all & ~used;
    }

    autoCandidates() {
      if (!this.canNotes || this.gameOver) return;
      this.begin(-1);
      for (let i = 0; i < this.cells; i++) {
        if (this.board[i] || this.locked[i]) continue;
        this.snap([i]);
        this.notes[i] = this.candidates(i);
      }
      this.commit();
      this.say(t("sud.cands_done"), this.accent);
      this.playSound("powerup");
    }

    useHint() {
      if (!this.canHint || this.hintsUsed >= MAX_HINTS || this.gameOver) return;
      let idx = this.sel;
      if (this.locked[idx] || (this.board[idx] && !this.wrong.has(idx))) {
        const empties = [];
        for (let i = 0; i < this.cells; i++) if (!this.locked[i] && this.board[i] !== this.solution[i]) empties.push(i);
        if (!empties.length) return;
        idx = PG.rand.choice(empties);
      }
      const d = this.solution[idx];
      this.board[idx] = d;
      this.notes[idx] = 0;
      this.wrong.delete(idx);
      this.locked[idx] = true;
      this.hinted.add(idx);
      this.hintsUsed++;
      this.pruneNotes(idx, d);
      this.sel = idx;
      const now = ui.ticks();
      this.fxPop.set(idx, now);
      this.dirty = true;
      this.playSound("powerup");
      this.updateConflicts();
      this.checkUnits(idx, now);
      this.checkWin();
    }

    checkUnits(idx, now) {
      let done = false;
      for (const g of this.groupsOf[idx]) {
        const cells = this.groups[g];
        if (cells.every((i) => this.board[i] === this.solution[i])) {
          this.fxUnits.push([cells, idx, now]);
          done = true;
        }
      }
      if (this.fxUnits.length > 12) this.fxUnits = this.fxUnits.slice(-12);
      return done;
    }

    updateConflicts() {
      if (!this.canCheck) return;
      const bad = new Set();
      for (let i = 0; i < this.cells; i++) {
        const v = this.board[i];
        if (v && this.peers[i].some((j) => this.board[j] === v)) bad.add(i);
      }
      const badCages = new Set();
      this.cages.forEach(([total, cells], ci) => {
        const vals = cells.map((i) => this.board[i]).filter((x) => x);
        const s = vals.reduce((a, b) => a + b, 0);
        if (new Set(vals).size !== vals.length || s > total || (vals.length === cells.length && s !== total) || (vals.length < cells.length && s >= total)) badCages.add(ci);
      });
      this.conflicts = bad;
      this.badCages = badCages;
    }

    checkWin() {
      if (this.board.includes(0)) return;
      if (this.board.every((v, i) => v === this.solution[i])) this.win();
      else if (!this.canCheck) this.say(t("sud.full_wrong"), ui.RED);
    }

    say(text, col = null, secs = 2.5) {
      this.msg = text;
      this.msgCol = col;
      this.msgT = secs;
    }

    win() {
      this.won = true;
      this.gameOver = true;
      const v = this.playVariant, d = this.playDiff;
      const secs = Math.trunc(this.elapsed);
      const raw = BASE_POINTS[v][d] - TIME_COST * secs - ERR_COST * this.errors - HINT_COST * this.hintsUsed;
      this.score = Math.trunc(Math.max(50, raw) * this.mult);
      const stars = starsFor(v, d, secs, this.errors, this.hintsUsed);
      this.resultStars = stars;
      this.resultTime = secs;
      this.newRecord = false;
      this.bestTime = secs;
      if (this.daily) {
        const st = this.dailyState;
        if (st.date === this.dailyDate) {
          const old = st.time !== undefined ? st.time : secs + 1;
          this.newRecord = secs < old;
          st.time = Math.min(old, secs);
          st.stars = Math.max(st.stars || 0, stars);
        } else {
          const streak = this.dailyStreak() ? this.dailyStreak() + 1 : 1;
          Object.assign(st, { date: this.dailyDate, streak, best: Math.max(st.best || 0, streak), time: secs, stars, count: (st.count || 0) + 1 });
        }
        this.bestTime = st.time;
      } else {
        const bv = (this.best[v] = this.best[v] || {});
        const levels = (bv[String(d)] = bv[String(d)] || {});
        const old = levels[String(this.level)];
        if (old) {
          this.newRecord = secs < old[1];
          levels[String(this.level)] = [Math.max(old[0], stars), Math.min(old[1], secs)];
        } else {
          levels[String(this.level)] = [stars, secs];
        }
        this.bestTime = levels[String(this.level)][1];
        if (v === "classic" && !this.solved[String(d)].includes(this.level)) {
          this.solved[String(d)] = this.solved[String(d)].concat([this.level]).sort((a, b) => a - b);
        }
      }
      delete this.saves[this.saveKey()];
      this.saveProgress();
      this.reportResult(true);
      if (this.errors === 0) this.achEvent("sudoku_clean");
      if (v === "killer" && !this.daily) this.achEvent("sudoku_killer");
      this.achEvent("sudoku_stars", this.threeStarCount());
      this.winTicks = ui.ticks();
      this.starSnd = new Set();
      this.playSound("win");
      this.rumble(200);
      if (stars === 3) ui.spawnConfetti(this.width, this.height, 70);
    }

    lose() {
      this.won = false;
      this.gameOver = true;
      this.score = 0;
      if (this.saves[this.saveKey()]) {
        delete this.saves[this.saveKey()];
        this.saveProgress();
      }
      this.reportResult(false);
      this.winTicks = ui.ticks();
      this.playSound("gameover");
      this.rumble(250);
    }

    nextLevel() {
      if (this.daily || this.level >= gen.LEVELS) {
        this.backToSetup();
        return;
      }
      this.variant = this.playVariant;
      this.diff = this.playDiff;
      this.startLevel(this.level + 1);
    }

    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state === GENERATING) {
        if (this.genDrawn) this.doGenerate();
        return;
      }
      if (this.state !== PLAY || this.gameOver) return;
      this.elapsed += dt;
      if (this.dirty) {
        this.saveT += dt;
        if (this.saveT >= SAVE_EVERY) this.writeSave();
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) this.drawSetup(ctx);
      else if (this.state === GENERATING) this.drawGenerating(ctx);
      else {
        this.drawHud(ctx);
        this.drawBoard(ctx);
        this.drawPad(ctx);
        if (this.gameOver && !this.reveal) this.drawResult(ctx);
      }
    }

    drawGenerating(ctx) {
      ui.text(ctx, t("sud.generating"), fl(this.width / 2), fl(this.height / 2), this.font, ui.TEXT, "center");
      this.genDrawn = true;
    }

    /** Text in die erste passende Schrift (sonst gekürzt) -> [text, font]. */
    fit(text, width, fonts) {
      for (const f of fonts) if (f.width(text) <= width) return [text, f];
      const f = fonts[fonts.length - 1];
      while (text.length > 1 && f.width(text + "...") > width) text = text.slice(0, -1);
      return [text.trimEnd() + "...", f];
    }

    fitText(ctx, text, x, y, width, fonts, color, anchor) {
      const [s, f] = this.fit(text, width, fonts);
      return ui.text(ctx, s, x, y, f, color, anchor);
    }

    diffName(d) {
      return t("sud.diff." + DIFFICULTIES[d][0]);
    }

    drawFooter(ctx, text, foot) {
      const cx = fl(this.width / 2), cy = this.height - fl(foot / 2);
      if (this.msg) {
        const col = this.msgCol || ui.RED;
        const [s, f] = this.fit(this.msg, this.width - 40, [this.small, this.tiny]);
        const w = f.width(s) + 20, h = f.height + 4;
        const bg = new Rect(cx - fl(w / 2), Math.min(cy - fl(h / 2), this.height - 1 - h), w, h);
        draw.rect(ctx, ui.PANEL, bg, 0, fl(h / 2));
        draw.rect(ctx, col, bg, 1, fl(h / 2));
        ui.text(ctx, s, bg.centerx, bg.centery, f, col, "center");
        return;
      }
      this.fitText(ctx, text, cx, cy, this.width - 12, [this.tiny], ui.TEXT_DIM, "center");
    }

    // ----- Setup ----------------------------------------------------------
    drawSetup(ctx) {
      const W = this.width, cx = fl(W / 2);
      ui.text(ctx, "SUDOKU", cx, this.suTitleY, this.titleFont, this.accent, "center");
      const modeLbl = this.mode in MODE_MULT ? t("sud.mode." + this.mode) : this.mode;
      this.fitText(ctx, modeLbl, cx, this.suSubY, W - 24, [this.tiny], ui.TEXT_DIM, "center");

      this.varRects.forEach((r, i) => {
        const v = VARIANTS[i];
        const on = v === this.variant;
        ui.drawButton(ctx, r, "", this.small, on, { accent: this.accent });
        this.drawVariantLabel(ctx, r, v, on);
      });
      this.diffRects.forEach((r, i) => {
        const label = this.diffName(i);
        const f = this.small.width(label) <= r.w - 10 ? this.small : this.tiny;
        ui.drawButton(ctx, r, label, f, i === this.diff, { accent: this.accent });
      });

      this.drawLevelGrid(ctx);

      const [n, stars] = this.stageSummary(this.variant, this.diff);
      const txt = t("sud.progress", { n }) + "   ·   " + stars + "/300";
      const rc = this.fitText(ctx, txt, this.lvX + 5 * this.lvCell - 10, this.progY, 10 * this.lvCell - 20, [this.small, this.tiny], ui.TEXT_DIM, "center");
      drawStar(ctx, rc.right + 10, rc.centery, Math.max(5, fl(rc.h / 3)), ui.GOLD);

      ui.drawPanel(ctx, this.panelRect);
      this.drawDailyCard(ctx);
      this.drawToggle(ctx, this.limitRect, "limit", t("sud.fail_limit"), null, this.failLimit);
      this.drawToggle(ctx, this.inputRect, "input", t("sud.opt_input"), t("sud.input." + this.inputMode), true);
      this.drawToggle(ctx, this.colorsRect, "colors", t("sud.opt_colors"), null, this.colorsOn);
      this.drawLevelInfo(ctx);
      if (this.startRect) {
        const save = this.levelSave(this.variant, this.diff, this.cursor);
        const label = save ? t("sud.resume") : t("sud.start", { n: this.cursor });
        ui.drawButton(ctx, this.startRect, label, this.small, this.isHover("start"), { accent: this.accent });
      }
      this.drawFooter(ctx, t("sud.setup_hint"), this.tiny.height + 10);
    }

    drawVariantLabel(ctx, r, v, on) {
      const label = t("sud.variant." + v);
      const col = on ? ui.TEXT : ui.TEXT_DIM;
      const f = this.small.width(label) > r.w - 10 ? this.tiny : this.small;
      const icon = Math.max(10, Math.min(r.h - 12, 22));
      const total = icon + 7 + f.width(label);
      if (total <= r.w - 12) {
        const x = r.centerx - fl(total / 2);
        this.drawVariantIcon(ctx, new Rect(x, r.centery - fl(icon / 2), icon, icon), v, on ? this.accent : ui.TEXT_FAINT);
        ui.text(ctx, label, x + icon + 7, r.centery, f, col, "midleft");
      } else {
        ui.text(ctx, label, r.centerx, r.centery, f, col, "center");
      }
    }

    drawVariantIcon(ctx, r, v, col) {
      const n = v === "mini" ? 2 : 3;
      draw.rect(ctx, col, r, 1, 2);
      for (let k = 1; k < n; k++) {
        const x = r.x + fl((r.w * k) / n) + 0.5, y = r.y + fl((r.h * k) / n) + 0.5;
        draw.line(ctx, col, [x, r.y], [x, r.bottom], 1);
        draw.line(ctx, col, [r.x, y], [r.right, y], 1);
      }
      if (v === "x") {
        draw.line(ctx, col, [r.x, r.y], [r.right, r.bottom], 2);
        draw.line(ctx, col, [r.right, r.y], [r.x, r.bottom], 2);
      } else if (v === "killer") {
        const inner = new Rect(r.x + fl(r.w / 6), r.y + fl(r.h / 6), r.w - 2 * fl(r.w / 6), r.h - 2 * fl(r.h / 6));
        ctx.save();
        ctx.setLineDash([2, 2]);
        draw.rect(ctx, col, inner, 1);
        ctx.restore();
      }
    }

    drawLevelGrid(ctx) {
      const v = this.variant, d = this.diff, cell = this.lvCell;
      const solvedBg = ui.mix(ui.PANEL, ui.GREEN, 0.2);
      const goldBg = ui.mix(ui.PANEL, ui.GOLD, 0.22);
      const big = cell >= 30;
      for (let n = 1; n <= 100; n++) {
        const i = n - 1;
        const x = this.lvX + (i % 10) * cell, y = this.lvY + fl(i / 10) * cell;
        const rc = new Rect(x + 1, y + 1, cell - 2, cell - 2);
        const [stars] = this.record(v, d, n);
        const save = this.levelSave(v, d, n);
        const bg = stars === 3 ? goldBg : stars ? solvedBg : ui.BTN;
        draw.rect(ctx, bg, rc, 0, 4);
        if (n === this.cursor) draw.rect(ctx, this.accent, rc, 2, 4);
        const col = stars === 3 ? ui.GOLD : stars ? ui.GREEN : ui.TEXT_DIM;
        const cy = rc.centery - (big && stars ? fl(cell / 7) : 0);
        ui.text(ctx, String(n), rc.centerx, cy, this.lvFont, col, "center");
        if (stars) {
          if (big) {
            const sr = Math.max(3, fl(cell / 10));
            for (let k = 0; k < 3; k++) {
              drawStar(ctx, rc.centerx + (k - 1) * (sr * 2 + 2), rc.bottom - sr - 3, sr, k < stars ? ui.GOLD : ui.mix(bg, ui.TEXT_FAINT, 0.5));
            }
          } else {
            for (let k = 0; k < stars; k++) draw.circle(ctx, ui.GOLD, [rc.centerx + (k - 1) * 4, rc.bottom - 3], 1);
          }
        }
        if (save) {
          const tri = Math.max(5, fl(cell / 4));
          draw.polygon(ctx, this.accent, [[rc.right - tri, rc.top], [rc.right, rc.top], [rc.right, rc.top + tri]]);
          const p = isInt(save.p) ? save.p : 0;
          const bw = fl(((rc.w - 6) * Math.max(0, Math.min(100, p))) / 100);
          if (bw > 0 && !stars) draw.rect(ctx, this.accent, [rc.x + 3, rc.bottom - 4, bw, 2]);
        }
      }
    }

    drawDailyCard(ctx) {
      const r = this.dailyRect;
      const hov = this.isHover("daily");
      const done = this.dailyDoneToday();
      draw.rect(ctx, ui.mix(ui.PANEL_LIGHT, this.accent, 0.1 + (hov ? 0.1 : 0)), r, 0, 8);
      const glow = hov ? this.accent : ui.mix(ui.BORDER, this.accent, 0.35 + 0.35 * ui.pulse(2.0, 0, 1) * (done ? 0 : 1));
      draw.rect(ctx, glow, r, hov ? 2 : 1, 8);

      const ih = Math.min(r.h - 10, 34);
      const cal = new Rect(r.x + 7, r.centery - fl(ih / 2), ih, ih);
      draw.rect(ctx, ui.mix(ui.PANEL, ui.TEXT, 0.08), cal, 0, 4);
      draw.rect(ctx, this.accent, [cal.x, cal.y, cal.w, Math.max(4, fl(ih / 4))], 0, [4, 4, 0, 0]);
      ui.text(ctx, String(Number(this.dailyDate.slice(8, 10))), cal.centerx, cal.centery + fl(ih / 8), this.tiny, ui.TEXT, "center");

      const tx = cal.right + 8;
      const tw = r.right - 8 - tx;
      const y1 = r.y + 5;
      const l1 = this.fitText(ctx, t("sud.daily"), tx, y1, tw - 18, [this.small, this.tiny], ui.TEXT, "topleft");
      const streak = this.dailyStreak() ? t("sud.daily_streak", { n: this.dailyStreak() }) : null;
      const save = this.saves["daily:" + this.dailyDate];
      let head, options;
      if (done) {
        head = [fmtTime(this.dailyState.time || 0)];
        drawCheck(ctx, r.right - 14, y1 + fl(l1.h / 2), 5, ui.GREEN, 2);
        options = [head.concat([streak]), head];
      } else {
        head = [t("sud.wd." + gen.weekday(this.dailyDate)), this.diffName(gen.dailyDiff(this.dailyDate))];
        if (save && isInt(save.p)) head.push(save.p + " %");
        options = [head.concat([streak]), head.slice(1).concat([streak]), head.slice(1)];
      }
      let text = "";
      for (const parts of options) {
        text = parts.filter((p) => p).join("  ·  ");
        if (this.tiny.width(text) <= tw) break;
      }
      this.fitText(ctx, text, tx, r.bottom - 5, tw, [this.tiny], done ? ui.GREEN : ui.TEXT_DIM, "bottomleft");
    }

    drawToggle(ctx, r, key, label, value, on) {
      const hov = this.isHover(key);
      draw.rect(ctx, hov ? ui.BTN_SEL : ui.BTN, r, 0, 6);
      draw.rect(ctx, hov ? this.accent : ui.BORDER, r, 1, 6);
      let room;
      if (value === null) {
        const sw = Math.max(24, fl((r.h * 3) / 2));
        const sh = Math.max(12, r.h - 10);
        const pill = new Rect(r.right - 8 - sw, r.centery - fl(sh / 2), sw, sh);
        draw.rect(ctx, on ? ui.GREEN : ui.mix(ui.BTN, ui.TEXT_FAINT, 0.5), pill, 0, fl(sh / 2));
        const kx = on ? pill.right - fl(sh / 2) : pill.x + fl(sh / 2);
        draw.circle(ctx, ui.TEXT, [kx, pill.centery], fl(sh / 2) - 2);
        room = pill.x - 8 - (r.x + 8);
        this.fitText(ctx, label, r.x + 8, r.centery, room, [this.tiny], on ? ui.TEXT : ui.TEXT_DIM, "midleft");
      } else {
        const vr = ui.text(ctx, value, r.right - 8, r.centery, this.tiny, this.accent, "midright");
        room = r.w - 24 - vr.w;
        this.fitText(ctx, label, r.x + 8, r.centery, room, [this.tiny], ui.TEXT, "midleft");
      }
    }

    drawLevelInfo(ctx) {
      const r = this.infoRect;
      if (r.h < this.tiny.height) return;
      const v = this.variant, d = this.diff, n = this.cursor;
      const [stars, secs] = this.record(v, d, n);
      const save = this.levelSave(v, d, n);
      let y = r.y;
      const head = ui.text(ctx, t("sud.level", { n }), r.x, y, this.small, ui.TEXT, "topleft");
      const sr = Math.max(5, fl(head.h / 3));
      for (let k = 0; k < 3; k++) {
        drawStar(ctx, r.right - sr - k * (2 * sr + 3), y + fl(head.h / 2), sr, 3 - k <= stars ? ui.GOLD : ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.6));
      }
      y += head.h + 2;
      const lines = [];
      if (secs !== null) lines.push([t("sud.info_best", { t: fmtTime(secs) }), ui.GREEN]);
      else if (stars) lines.push([t("sud.info_solved"), ui.GREEN]);
      else lines.push([t("sud.info_unsolved"), ui.TEXT_DIM]);
      if (save) lines.push([t("sud.info_started", { p: isInt(save.p) ? save.p : 0 }), this.accent]);
      lines.push([t("sud.info_target", { t: fmtTime(TARGET_TIME[v][d]) }), ui.TEXT_DIM]);
      const th = this.tiny.height;
      for (const [txt, col] of lines) {
        for (const line of ui.wrap(txt, this.tiny, r.w).slice(0, 2)) {
          if (y + th > r.bottom) return;
          this.fitText(ctx, line, r.x, y, r.w, [this.tiny], col, "topleft");
          y += th;
        }
        y += 1;
      }
      y += 4;
      for (const line of ui.wrap(t("sud.vdesc." + v), this.tiny, r.w)) {
        if (y + th > r.bottom) return;
        ui.text(ctx, line, r.x, y, this.tiny, ui.TEXT_FAINT, "topleft");
        y += th;
      }
    }

    // ----- HUD ------------------------------------------------------------
    drawHud(ctx) {
      const W = this.width;
      draw.rect(ctx, ui.PANEL, [0, 0, W, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH + 0.5], [W, this.hudH + 0.5]);
      const cy = fl(this.hudH / 2);
      let crect;
      if (this.gameOver && this.reveal) crect = ui.text(ctx, t("sud.hide_solution"), fl(W / 2), cy, this.small, this.accent, "center");
      else crect = ui.text(ctx, fmtTime(this.elapsed), fl(W / 2), cy, this.clockFont, this.accent, "center");
      const side = crect.left - 20;

      let top, bottom;
      if (this.daily) {
        top = t("sud.daily");
        bottom = t("sud.wd." + gen.weekday(this.dailyDate)) + "  ·  " + this.diffName(this.playDiff);
      } else {
        top = t("sud.variant." + this.playVariant);
        bottom = t("sud.level", { n: this.level }) + "  ·  " + this.diffName(this.playDiff);
      }
      this.fitText(ctx, top, 12, cy + 1, side, [this.small, this.tiny], this.accent, "bottomleft");
      this.fitText(ctx, bottom, 12, cy + 1, side, [this.tiny], ui.TEXT, "topleft");

      const errTxt = this.failLimit ? t("sud.errors", { n: this.errors, m: FAIL_LIMIT }) : t("sud.errors_free", { n: this.errors });
      this.fitText(ctx, errTxt, W - 12, cy + 1, side, [this.small, this.tiny], this.errors ? ui.RED : ui.TEXT_DIM, "bottomright");
      const right = [];
      if (this.canHint) right.push([t("sud.hints", { n: MAX_HINTS - this.hintsUsed }), ui.TEXT_DIM]);
      if (this.noteMode) right.push([t("sud.pad_note").toUpperCase(), this.accent]);
      if (this.inputMode === "digit" && this.brush) right.push([t("sud.brush", { d: this.brush }), this.accent]);
      let x = W - 12;
      for (const [txt, col] of right.reverse()) {
        const w = this.tiny.width(txt);
        if (x - w < crect.right + 20) break;
        ui.text(ctx, txt, x, cy + 1, this.tiny, col, "topright");
        x -= w + 12;
      }
    }

    // ----- Brett ------------------------------------------------------------
    buildCagePath() {
      const N = this.N, cell = this.cell;
      const path = new Path2D();
      const ins = Math.max(3, fl(cell / 9));
      const dash = Math.max(3, fl(cell / 8));
      const period = dash + Math.max(2, fl((dash * 3) / 4));
      const cageOf = this.cageOf;
      const labels = new Map();
      this.sumPos = [];
      this.cages.forEach(([total, cells]) => {
        const first = Math.min(...cells);
        const r = fl(first / N), c = first % N;
        const x = c * cell + 2, y = r * cell + 1;
        this.sumPos.push([x, y]);
        labels.set(first, [x - 1, y, this.sumFont.width(String(total)) + 3, this.sumFont.height]);
      });
      const same = (r, c, k) => r >= 0 && r < N && c >= 0 && c < N && cageOf[r * N + c] === k;
      let lbl = null;
      const inLabel = (x0, y0, x1, y1) => lbl && x1 >= lbl[0] && x0 <= lbl[0] + lbl[2] && y1 >= lbl[1] && y0 <= lbl[1] + lbl[3];
      const hline = (y, xa, xb) => {
        let x = xa;
        while (x < xb) {
          const ph = PG.mod(x, period);
          if (ph < dash) {
            const e = Math.min(xb, x + dash - ph);
            if (!inLabel(x, y, e, y)) {
              path.moveTo(x, y + 0.5);
              path.lineTo(e + 1, y + 0.5);
            }
            x = e;
          } else x += period - ph;
        }
      };
      const vline = (x, ya, yb) => {
        let y = ya;
        while (y < yb) {
          const ph = PG.mod(y, period);
          if (ph < dash) {
            const e = Math.min(yb, y + dash - ph);
            if (!inLabel(x, y, x, e)) {
              path.moveTo(x + 0.5, y);
              path.lineTo(x + 0.5, e + 1);
            }
            y = e;
          } else y += period - ph;
        }
      };
      for (let i = 0; i < N * N; i++) {
        const r = fl(i / N), c = i % N, k = cageOf[i];
        lbl = labels.get(i) || null;
        const x0 = c * cell, y0 = r * cell, x1 = x0 + cell - 1, y1 = y0 + cell - 1;
        const L = same(r, c - 1, k), R = same(r, c + 1, k), U = same(r - 1, c, k), D = same(r + 1, c, k);
        if (!U) hline(y0 + ins, L ? x0 : x0 + ins, R ? x1 + 1 : x1 - ins);
        if (!D) hline(y1 - ins, L ? x0 : x0 + ins, R ? x1 + 1 : x1 - ins);
        if (!L) vline(x0 + ins, U ? y0 : y0 + ins, D ? y1 + 1 : y1 - ins);
        if (!R) vline(x1 - ins, U ? y0 : y0 + ins, D ? y1 + 1 : y1 - ins);
        for (const [dr, dc] of [[-1, -1], [-1, 1], [1, -1], [1, 1]]) {
          if (same(r + dr, c, k) && same(r, c + dc, k) && !same(r + dr, c + dc, k)) {
            const px = dc < 0 ? x0 + ins : x1 - ins;
            const py = dr < 0 ? y0 + ins : y1 - ins;
            if (dr < 0) vline(px, y0, py);
            else vline(px, py, y1 + 1);
            if (dc < 0) hline(py, x0, px);
            else hline(py, px, x1 + 1);
          }
        }
      }
      this.cagePath = path;
      this.cageCell = cell;
    }

    drawBoard(ctx) {
      const N = this.N, cell = this.cell, bx = this.bx, by = this.by, lay = this.lay;
      const now = ui.ticks();
      const sel = this.sel;
      const selPeers = new Set(this.peers[sel]);
      const selCage = this.cageOf[sel];
      const sameVal = this.inputMode === "digit" && this.brush ? this.brush : this.board[sel];

      const cCell = ui.PANEL, cPeer = ui.PANEL_LIGHT;
      const cSel = ui.mix(ui.PANEL_LIGHT, this.accent, 0.35);
      const cSame = ui.mix(ui.PANEL_LIGHT, this.accent, 0.18);
      const cBad = ui.mix(ui.PANEL, ui.RED, 0.35);
      const cDiag = ui.mix(ui.PANEL, this.accent, 0.08);
      const cCage = ui.mix(ui.PANEL, this.accent, 0.13);
      const glow = ui.mix(this.accent, [255, 255, 255], 0.45);
      const showSolution = this.gameOver && this.reveal;
      const badCells = new Set(this.conflicts);
      if (this.canCheck) for (const ci of this.badCages) for (const i of this.cages[ci][1]) badCells.add(i);

      const flash = new Map();
      this.fxUnits = this.fxUnits.filter(([cells, origin, t0]) => {
        const ageAll = now - t0;
        if (ageAll > 900) return false;
        const orr = fl(origin / N), occ = origin % N;
        for (const i of cells) {
          const age = ageAll - 45 * (Math.abs(fl(i / N) - orr) + Math.abs((i % N) - occ));
          if (age >= 0 && age < 420) {
            const a = Math.sin((Math.PI * age) / 420) * 0.6;
            if (a > (flash.get(i) || 0)) flash.set(i, a);
          }
        }
        return true;
      });
      const winAge = this.gameOver && this.won ? now - this.winTicks : -1;

      for (let i = 0; i < N * N; i++) {
        const r = fl(i / N), c = i % N;
        let bg;
        if (i === sel) bg = cSel;
        else if (this.canCheck && badCells.has(i)) bg = cBad;
        else if (this.canCheck && sameVal && this.board[i] === sameVal) bg = cSame;
        else if (selCage >= 0 && this.cageOf[i] === selCage) bg = cCage;
        else if (selPeers.has(i)) bg = cPeer;
        else if (lay.diagonals && lay.onDiagonal(i)) bg = cDiag;
        else bg = cCell;
        if (this.marks[i]) bg = ui.mix(bg, MARK_COLORS[this.marks[i] - 1], 0.42);
        const a = flash.get(i);
        if (a) bg = ui.mix(bg, glow, a);
        if (winAge >= 0) {
          const dist = Math.abs(r - (N - 1) / 2) + Math.abs(c - (N - 1) / 2);
          const age = winAge - 60 * dist;
          if (age >= 0 && age < 520) bg = ui.mix(bg, ui.GREEN, Math.sin((Math.PI * age) / 520) * 0.55);
        }
        draw.rect(ctx, bg, [bx + c * cell, by + r * cell, cell, cell]);
      }

      if (lay.diagonals) {
        const bs = cell * N;
        const col = ui.col(ui.mix(ui.PANEL_LIGHT, this.accent, 0.55), 90 / 255);
        draw.line(ctx, col, [bx, by], [bx + bs, by + bs], Math.max(1, fl(cell / 14)));
        draw.line(ctx, col, [bx + bs, by], [bx, by + bs], Math.max(1, fl(cell / 14)));
      }
      if (this.cages.length) {
        if (!this.cagePath || this.cageCell !== cell) this.buildCagePath();
        ctx.save();
        ctx.translate(bx, by);
        ctx.strokeStyle = ui.col(ui.mix(ui.TEXT_DIM, ui.TEXT, 0.2), 215 / 255);
        ctx.lineWidth = 1;
        ctx.stroke(this.cagePath);
        ctx.restore();
        this.cages.forEach(([total], ci) => {
          const col = this.canCheck && this.badCages.has(ci) ? ui.RED : ci === selCage ? this.accent : ui.TEXT_DIM;
          const [sx, sy] = this.sumPos[ci];
          ui.text(ctx, String(total), bx + sx, by + sy, this.sumFont, col, "topleft");
        });
      }

      const ins = this.cages.length ? Math.max(3, fl(cell / 9)) + 2 : 0;
      const noteTop = this.cages.length ? Math.max(ins, this.sumFont.height - 1) : 0;
      const rows = fl((N + 2) / 3);
      const cw = fl((cell - 2 * ins) / 3);
      const ch = fl((cell - noteTop - ins) / rows);
      for (let i = 0; i < N * N; i++) {
        const r = fl(i / N), c = i % N;
        const x = bx + c * cell, y = by + r * cell;
        if (showSolution && this.board[i] !== this.solution[i]) {
          ui.text(ctx, String(this.solution[i]), x + fl(cell / 2), y + fl(cell / 2), this.numFont, this.accent, "center");
          continue;
        }
        const v = this.board[i];
        if (v) {
          let col;
          if (this.given[i]) col = ui.TEXT;
          else if (this.canCheck && this.wrong.has(i)) col = ui.RED;
          else if (this.locked[i]) col = ui.GREEN;
          else col = COL_USER;
          let cxm = x + fl(cell / 2);
          const cym = y + fl(cell / 2) + (this.cages.length ? fl(noteTop / 3) : 0);
          let k = 1;
          const t0 = this.fxPop.get(i);
          if (t0 !== undefined) {
            const age = now - t0;
            if (age < 240) k = 1 + 0.4 * Math.sin((Math.PI * age) / 240);
            else this.fxPop.delete(i);
          }
          const t1 = this.fxShake.get(i);
          if (t1 !== undefined) {
            const age = now - t1;
            if (age < 380) cxm += Math.trunc(Math.sin(age / 22) * cell * 0.09 * (1 - age / 380));
            else this.fxShake.delete(i);
          }
          if (k !== 1) {
            ctx.save();
            ctx.translate(cxm, cym);
            ctx.scale(k, k);
            ui.text(ctx, String(v), 0, 0, this.numFont, col, "center");
            ctx.restore();
          } else {
            ui.text(ctx, String(v), cxm, cym, this.numFont, col, "center");
          }
        } else if (this.notes[i]) {
          const nm = this.notes[i];
          for (let d = 1; d <= N; d++) {
            if (!((nm >> (d - 1)) & 1)) continue;
            const col = this.canCheck && d === sameVal ? this.accent : ui.TEXT_FAINT;
            const nx = x + ins + ((d - 1) % 3) * cw + fl(cw / 2);
            const ny = y + noteTop + fl((d - 1) / 3) * ch + fl(ch / 2);
            ui.text(ctx, String(d), nx, ny, this.noteFont, col, "center");
          }
        }
      }

      const bs = cell * N;
      for (let k = 0; k <= N; k++) {
        const thick = k % lay.boxW === 0;
        const x = bx + k * cell + (thick ? 0 : 0.5);
        draw.line(ctx, thick ? ui.BORDER_LIGHT : ui.BORDER, [x, by], [x, by + bs], thick ? 2 : 1);
      }
      for (let k = 0; k <= N; k++) {
        const thick = k % lay.boxH === 0;
        const y = by + k * cell + (thick ? 0 : 0.5);
        draw.line(ctx, thick ? ui.BORDER_LIGHT : ui.BORDER, [bx, y], [bx + bs, y], thick ? 2 : 1);
      }
      if (!this.gameOver) {
        draw.rect(ctx, this.accent, [bx + (sel % N) * cell, by + fl(sel / N) * cell, cell + 1, cell + 1], 2);
      }
    }

    // ----- Ziffernfeld ------------------------------------------------------
    drawPad(ctx) {
      const N = this.N;
      const counts = new Array(N + 1).fill(0);
      for (const v of this.board) counts[v]++;
      const th = this.tiny.height;
      for (let d = 1; d <= N; d++) {
        const r = this.padRects[String(d)];
        const left = N - counts[d];
        const done = left <= 0;
        const on = this.inputMode === "digit" && this.brush === d;
        const hov = this.hover === String(d);
        draw.rect(ctx, on ? ui.BTN_SEL : hov ? ui.mix(ui.BTN, ui.PANEL_LIGHT, 0.6) : ui.BTN, r, 0, 6);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 6);
        ui.text(ctx, String(d), r.centerx, r.centery - fl(th / 3), this.padFont, done ? ui.TEXT_FAINT : on ? this.accent : ui.TEXT, "center");
        if (done) drawCheck(ctx, r.centerx, r.bottom - fl(th / 2) - 2, Math.max(3, fl(th / 3)), ui.GREEN, 2);
        else ui.text(ctx, String(left), r.centerx, r.bottom - fl(th / 2) - 1, this.tiny, ui.TEXT_DIM, "center");
      }

      for (const key of ["erase", "undo", "redo", "note", "cands", "hint"]) {
        const r = this.padRects[key];
        if (!r) continue;
        const hov = this.hover === key;
        const active = key === "note" && this.noteMode;
        let enabled = true;
        if (key === "undo") enabled = this.undoStack.length > 0;
        else if (key === "redo") enabled = this.redoStack.length > 0;
        else if (key === "hint") enabled = this.hintsUsed < MAX_HINTS;
        draw.rect(ctx, active ? ui.BTN_SEL : hov ? ui.mix(ui.BTN, ui.PANEL_LIGHT, 0.6) : ui.BTN, r, 0, 6);
        draw.rect(ctx, active || hov ? this.accent : ui.BORDER, r, active ? 2 : 1, 6);
        this.drawPadIcon(ctx, key, r, active ? this.accent : enabled ? ui.TEXT : ui.TEXT_FAINT);
        let letter = PAD_KEYS[key];
        if (key === "hint") letter += " " + (MAX_HINTS - this.hintsUsed);
        if (r.h >= 26) ui.text(ctx, letter, r.right - 3, r.bottom, this.tiny, ui.TEXT_FAINT, "bottomright");
      }

      for (let k = 0; k < MARK_COUNT; k++) {
        const r = this.padRects["mark" + (k + 1)];
        if (!r) continue;
        draw.rect(ctx, MARK_COLORS[k], r, 0, 4);
        if (this.marks[this.sel] === k + 1) draw.rect(ctx, ui.TEXT, r.inflate(2, 2), 2, 5);
      }

      let tip = null;
      if (this.hover && PAD_TIPS[this.hover]) tip = t(PAD_TIPS[this.hover]);
      else if (this.hover && this.hover.startsWith("mark")) tip = t("sud.tip.mark");
      else if (this.hover && /^[1-9]$/.test(this.hover)) tip = this.inputMode === "digit" ? t("sud.tip.digit_first") : t("sud.tip.digit");
      else if (this.inputMode === "digit" && !this.brush) tip = t("sud.tip.digit_first");
      if (tip) {
        let y = this.captionY;
        for (const line of ui.wrap(tip, this.tiny, this.padW).slice(0, 2)) {
          if (y + th > this.captionBottom) break;
          ui.text(ctx, line, this.padX, y, this.tiny, ui.TEXT_DIM, "topleft");
          y += th;
        }
      }
      this.drawFooter(ctx, t(this.inputMode === "digit" ? "sud.hint_digit" : "sud.hint"), this.tiny.height + 12);
    }

    drawPadIcon(ctx, key, r, col) {
      const cx = r.centerx, cy = r.centery - (r.h >= 26 ? 2 : 0);
      const z = Math.max(5, fl((Math.min(r.w, r.h) * 3) / 10));
      const w = Math.max(2, fl(z / 4));
      if (key === "erase") {
        const lw = Math.max(1, fl(w / 2) + 1);
        draw.polygon(ctx, col, [[cx - z, cy], [cx - fl(z / 2), cy - fl((z * 2) / 3)], [cx + z, cy - fl((z * 2) / 3)], [cx + z, cy + fl((z * 2) / 3)], [cx - fl(z / 2), cy + fl((z * 2) / 3)]], lw);
        const k = fl(z / 3);
        draw.line(ctx, col, [cx - k + 2, cy - k], [cx + k + 2, cy + k], lw);
        draw.line(ctx, col, [cx - k + 2, cy + k], [cx + k + 2, cy - k], lw);
      } else if (key === "undo" || key === "redo") {
        const rr = new Rect(cx - z, cy - fl((z * 2) / 3), 2 * z, 2 * z);
        ctx.save();
        ctx.strokeStyle = ui.col(col);
        ctx.lineWidth = Math.max(2, w);
        ctx.beginPath();
        ctx.arc(rr.centerx, rr.centery, z - Math.max(2, w) / 2, Math.PI, 2 * Math.PI);
        ctx.stroke();
        ctx.restore();
        const tipX = key === "undo" ? rr.left + Math.max(1, fl(w / 2)) : rr.right - Math.max(1, fl(w / 2));
        const tipY = rr.centery + 2;
        draw.polygon(ctx, col, [[tipX - fl(z / 2) - 1, tipY - 2], [tipX + fl(z / 2) + 1, tipY - 2], [tipX, tipY + fl(z / 2) + 1]]);
      } else if (key === "note") {
        const a = [cx - z, cy + z];
        draw.line(ctx, col, [a[0] + fl(z / 3), a[1] - fl(z / 3)], [cx + fl((z * 2) / 3), cy - fl((z * 2) / 3)], Math.max(3, w + 1));
        draw.polygon(ctx, col, [a, [a[0] + fl(z / 2), a[1] - fl(z / 6)], [a[0] + fl(z / 6), a[1] - fl(z / 2)]]);
      } else if (key === "cands") {
        const step = Math.max(3, fl((z * 2) / 3));
        for (let rr = 0; rr < 3; rr++) {
          for (let cc = 0; cc < 3; cc++) {
            draw.circle(ctx, col, [cx + (cc - 1) * step, cy + (rr - 1) * step], Math.max(1, fl(z / 6)) + ((rr + cc) % 2 === 0 ? 1 : 0));
          }
        }
      } else if (key === "hint") {
        const rad = Math.max(3, fl((z * 2) / 3));
        draw.circle(ctx, col, [cx, cy - fl(z / 4)], rad, Math.max(1, fl(w / 2) + 1));
        draw.rect(ctx, col, [cx - fl(rad / 2), cy - fl(z / 4) + rad, rad, Math.max(2, fl(z / 3))]);
      }
    }

    // ----- Ergebnis ---------------------------------------------------------
    drawResult(ctx) {
      const W = this.width, H = this.height;
      draw.rect(ctx, [8, 10, 16, 175], [0, 0, W, H]);
      const cx = fl(W / 2), cy = fl(H / 2);
      const age = ui.ticks() - this.winTicks;
      const sh = this.small.height;
      let head, headCol, lines, starH;
      if (this.won) {
        head = t("sud.win", { t: fmtTime(this.resultTime) });
        headCol = ui.GREEN;
        lines = [[t("common.points", { score: this.score }), ui.TEXT]];
        if (this.daily) lines.push([t("sud.daily_solved", { n: this.dailyStreak() }), ui.GOLD]);
        if (this.newRecord) lines.push([t("sud.new_best", { t: fmtTime(this.bestTime) }), ui.GOLD]);
        else lines.push([t("sud.info_best", { t: fmtTime(this.bestTime) }), ui.TEXT_DIM]);
        lines.push([this.daily ? t("sud.next_daily") : t("sud.next"), ui.TEXT_DIM]);
        lines.push([t("sud.show_solution"), ui.TEXT_DIM]);
        starH = Math.max(26, fl(H / 14));
      } else {
        head = t("sud.lose");
        headCol = ui.RED;
        lines = [[t("sud.retry"), ui.TEXT_DIM], [t("sud.show_solution"), ui.TEXT_DIM]];
        starH = 0;
      }
      const [headTxt, headFont] = this.fit(head, W - 40, [this.huge, this.titleFont, this.small]);
      const fitted = lines.map(([txt, col]) => this.fit(txt, W - 60, [this.small, this.tiny]).concat([col]));
      let pw = Math.max(Math.min(W - 40, 460), headFont.width(headTxt) + 40, Math.max(...fitted.map(([s, f]) => f.width(s))) + 40);
      pw = Math.min(pw, W - 16);
      const ph = 20 + headFont.height + (starH ? starH + 10 : 4) + fitted.length * (sh + 6) + 14;
      const k = Math.min(1, Math.max(0, age / 260));
      const panel = new Rect(cx - fl(pw / 2), cy - fl(ph / 2) + Math.trunc((1 - k) * 24), pw, ph);
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, this.accent, panel, 2, 14);
      let y = panel.y + 14;
      ui.text(ctx, headTxt, cx, y, headFont, headCol, "midtop");
      y += headFont.height + 6;
      if (starH) {
        const r = fl(starH / 2);
        for (let n = 0; n < 3; n++) {
          const sx = cx + (n - 1) * (starH + 12), sy = y + r;
          const t0 = 350 + n * 280;
          if (n < this.resultStars && age >= t0) {
            const p = Math.min(1, (age - t0) / 220);
            const scale = p < 1 ? 1 + 0.5 * Math.sin(Math.PI * p) : 1;
            drawStar(ctx, sx, sy, r * scale, ui.GOLD);
            if (!this.starSnd.has(n)) {
              this.starSnd.add(n);
              this.playSound("point");
            }
          } else {
            drawStar(ctx, sx, sy, r, ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.7));
            drawStar(ctx, sx, sy, r, ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.9), false);
          }
        }
        y += starH + 10;
      }
      for (const [s, f, col] of fitted) {
        ui.text(ctx, s, cx, y, f, col, "midtop");
        y += sh + 6;
      }
    }
  }

  PG.register(SudokuGame, {
    id: "SudokuGame",
    key: "sudoku",
    name: "Sudoku",
    modes: [
      ["classic", "sud.mode.classic"],
      ["notes", "sud.mode.notes"],
      ["comfort", "sud.mode.comfort"],
      ["assist", "sud.mode.assist"],
    ],
    settingsKey: "sudoku",
    defaults: { difficulty: 0, fail_limit: true, last_level: {}, variant: "classic", input: "cell", colors: false },
    wantsRightClick: true, // Rechtsklick = radieren
  });
})();
