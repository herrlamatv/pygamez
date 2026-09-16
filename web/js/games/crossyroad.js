/*
 * crossyroad.js - Crossy Road (Port von games/crossyroad.py + crossyroad_draw.py)
 * ===============================================================================
 * Endlos über Wiesen, Straßen, Flüsse und Gleise hüpfen - im Voxel-Look.
 *
 * Welt/Regeln: crossyroad_world.js (bitgenau wie Python, gleiche Tagesstrecke),
 * Modelle: crossyroad_models.js (erzeugt aus Python). Hier: Voxel-Renderer
 * (Sprites und Boden-Streifen je Kachelgröße gecacht), Spielablauf, Kamera,
 * Adler, Tag/Nacht, HUD, Setup- und Figuren-Screen.
 *
 * Steuerung: Pfeile/WASD = hüpfen, Leertaste/Enter/Klick = vorwärts.
 * Speicher: PG.store "crossy" = {coins, unlocked, selected, daily: {date, best}}.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const W_ = PG.crossyWorld;
  const MODELS = PG.crossyModels;

  const SETUP = "setup", PLAY = "play";
  const TABS = ["play", "chars"];
  const LAND_T = 0.12;
  const BUMP_T = 0.14;
  const DEATH_DUR = { car: 1.25, train: 1.25, water: 1.25, edge: 1.25, eagle: 1.75 };
  const NIGHT_TINT = [62, 76, 140];
  const FOCUS_Y = 0.66;
  const GRID_COLS = 5;

  // ----- Voxel-Renderer ----------------------------------------------------------
  const MV = 7;
  const SHADOW_ALPHA = 64;
  const DIM = [30, 40, 66];
  const GRASS_A = [160, 222, 98], GRASS_B = [150, 212, 90], GRASS_SIDE = [112, 160, 64];
  const ROAD = [86, 90, 104], ROAD_SIDE = [62, 64, 76], ROAD_LINE = [226, 228, 222];
  const WATER_A = [96, 198, 248], WATER_B = [88, 190, 242], WAVE = [150, 224, 255];
  const RAIL_BED = [134, 122, 118], RAIL_SIDE = [104, 94, 92], SLEEPER = [116, 84, 62];
  const STEEL = [198, 202, 214], STEEL_SIDE = [124, 128, 142];

  function shade(col, f) {
    return [Math.min(255, Math.floor(col[0] * f)), Math.min(255, Math.floor(col[1] * f)), Math.min(255, Math.floor(col[2] * f))];
  }
  function dim(col, f = 0.4) {
    return [Math.floor(col[0] + (DIM[0] - col[0]) * f), Math.floor(col[1] + (DIM[1] - col[1]) * f), Math.floor(col[2] + (DIM[2] - col[2]) * f)];
  }
  function rgb(c) {
    return "rgb(" + c[0] + "," + c[1] + "," + c[2] + ")";
  }
  function pixelScale() {
    return Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
  }

  /** Quader-Liste zu einem Sprite-Schlüssel ("tree1", "car3<", "chicken:up", ...). */
  function modelFor(key) {
    if (key.indexOf(":") >= 0) {
      const [cid, facing] = key.split(":");
      return W_.rotate(MODELS.chars[cid], facing);
    }
    const flip = key.endsWith("<");
    const name = flip ? key.slice(0, -1) : key;
    let boxes = MODELS.models[name];
    if (flip) {
      let length = 1.0;
      if (name.startsWith("car")) length = W_.CAR_LEN;
      else if (name.startsWith("truck")) length = W_.TRUCK_LEN;
      else if (name === "engine" || name === "wagon") length = W_.TRAIN_CAR;
      boxes = W_.mirrorX(boxes, length);
    }
    return boxes;
  }

  /** Projektion + Caches für EINE Kachelgröße (wie crossyroad_draw.Voxel). */
  class Voxel {
    constructor(tw, shadows) {
      this.tw = Math.max(12, Math.floor(tw));
      this.th = Math.max(8, Math.round(this.tw * 0.78));
      this.sk = Math.max(2, Math.round(this.tw * 0.28));
      this.zh = Math.max(6, Math.round(this.tw * 0.66));
      this.shadows = shadows;
      this.ps = pixelScale();
      this.sprites = new Map();
      this.strips = new Map();
    }

    proj(x, y, z) {
      return [x * this.tw + y * this.sk, -y * this.th - z * this.zh];
    }

    depth(b) {
      const cx = (b[0] + b[3]) * 0.5, cy = (b[1] + b[4]) * 0.5, cz = (b[2] + b[5]) * 0.5;
      return (-cx * this.sk) / this.tw + cy - (cz * this.th) / this.zh;
    }

    rect(b) {
      let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
      for (const x of [b[0], b[3]]) {
        for (const y of [b[1], b[4]]) {
          for (const z of [b[2], b[5]]) {
            const p = this.proj(x, y, z);
            x0 = Math.min(x0, p[0]);
            y0 = Math.min(y0, p[1]);
            x1 = Math.max(x1, p[0]);
            y1 = Math.max(y1, p[1]);
          }
        }
      }
      return [x0, y0, x1, y1];
    }

    /** Zeichenreihenfolge hinten -> vorn (gleiches Verfahren wie Python). */
    order(boxes) {
      const n = boxes.length, eps = 1e-6;
      const rects = boxes.map((b) => this.rect(b));
      const depth = boxes.map((b) => this.depth(b));
      const deps = boxes.map(() => []);
      for (let i = 0; i < n; i++) {
        const ra = rects[i];
        for (let j = i + 1; j < n; j++) {
          const rb = rects[j];
          if (ra[2] <= rb[0] || rb[2] <= ra[0] || ra[3] <= rb[1] || rb[3] <= ra[1]) continue;
          const a = boxes[i], b = boxes[j];
          let first;
          if (a[1] >= b[4] - eps) first = i;
          else if (b[1] >= a[4] - eps) first = j;
          else if (a[3] <= b[0] + eps) first = i;
          else if (b[3] <= a[0] + eps) first = j;
          else if (a[5] <= b[2] + eps) first = i;
          else if (b[5] <= a[2] + eps) first = j;
          else first = depth[i] >= depth[j] ? i : j;
          deps[first === i ? j : i].push(first);
        }
      }
      const done = new Array(n).fill(false);
      const out = [];
      while (out.length < n) {
        let best = -1;
        for (let k = 0; k < n; k++) {
          if (done[k] || !deps[k].every((d) => done[d])) continue;
          if (best < 0 || depth[k] > depth[best]) best = k;
        }
        if (best < 0) {
          for (let k = 0; k < n; k++) if (!done[k] && (best < 0 || depth[k] > depth[best])) best = k;
        }
        done[best] = true;
        out.push(best);
      }
      return out;
    }

    /** Rendert Quader in eine Offscreen-Canvas: {canvas, ax, ay, w, h} (logische Pixel). */
    render(boxes, shadow) {
      const pts = [];
      for (const b of boxes) {
        const r = this.rect(b);
        pts.push([r[0], r[1]], [r[2], r[3]]);
      }
      const shadowPolys = [];
      if (shadow && this.shadows) {
        for (const b of boxes) {
          if (b[2] > 0.6) continue;
          const q = [this.proj(b[0] + 0.1, b[1] - 0.02, 0), this.proj(b[3] + 0.18, b[1] - 0.02, 0), this.proj(b[3] + 0.18, b[4] - 0.1, 0), this.proj(b[0] + 0.1, b[4] - 0.1, 0)];
          shadowPolys.push(q);
          pts.push(...q);
        }
      }
      let minx = Infinity, maxx = -Infinity, miny = Infinity, maxy = -Infinity;
      for (const p of pts) {
        minx = Math.min(minx, p[0]);
        maxx = Math.max(maxx, p[0]);
        miny = Math.min(miny, p[1]);
        maxy = Math.max(maxy, p[1]);
      }
      const ax = Math.floor(-minx) + 2, ay = Math.floor(-miny) + 2;
      const w = Math.floor(maxx - minx) + 5, h = Math.floor(maxy - miny) + 5;
      const ps = this.ps;
      const canvas = ui.makeCanvas(w * ps, h * ps);
      const c = canvas.getContext("2d");
      c.scale(ps, ps);
      if (shadowPolys.length) {
        const layer = ui.makeCanvas(w * ps, h * ps);
        const lc = layer.getContext("2d");
        lc.scale(ps, ps);
        lc.fillStyle = "#000";
        for (const q of shadowPolys) {
          lc.beginPath();
          q.forEach((p, i) => (i ? lc.lineTo(ax + p[0], ay + p[1]) : lc.moveTo(ax + p[0], ay + p[1])));
          lc.closePath();
          lc.fill();
        }
        c.save();
        c.setTransform(1, 0, 0, 1, 0, 0);
        c.globalAlpha = SHADOW_ALPHA / 255;
        c.drawImage(layer, 0, 0);
        c.restore();
      }
      c.lineJoin = "round";
      c.lineWidth = 0.6;
      for (const i of this.order(boxes)) this.box(c, boxes[i], ax, ay);
      return { canvas, ax, ay, w, h };
    }

    box(c, b, ax, ay) {
      const [x0, y0, z0, x1, y1, z1, col] = b;
      const p = (x, y, z) => {
        const q = this.proj(x, y, z);
        return [ax + q[0], ay + q[1]];
      };
      const face = (color, pts) => {
        const css = rgb(color);
        c.beginPath();
        c.moveTo(pts[0][0], pts[0][1]);
        for (let i = 1; i < pts.length; i++) c.lineTo(pts[i][0], pts[i][1]);
        c.closePath();
        c.fillStyle = css;
        c.strokeStyle = css;
        c.fill();
        c.stroke();
      };
      face(shade(col, 0.7), [p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1)]);
      face(shade(col, 0.86), [p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1)]);
      face(col, [p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)]);
    }

    sprite(key, shadow = true) {
      const ck = key + (shadow ? "" : "#");
      let got = this.sprites.get(ck);
      if (!got) {
        got = this.render(modelFor(key), shadow);
        this.sprites.set(ck, got);
      }
      return got;
    }

    /** Sprite so zeichnen, dass der Modell-Ursprung auf (x, y) liegt. */
    blit(ctx, key, x, y, shadow = true) {
      const s = this.sprite(key, shadow);
      ctx.drawImage(s.canvas, Math.round(x) - s.ax, Math.round(y) - s.ay, s.w, s.h);
    }

    /** Boden-Streifen einer Reihe (wie Voxel.strip in Python). */
    strip(kind, parity, lower, line) {
      const key = kind + "|" + parity + "|" + lower + "|" + line;
      let got = this.strips.get(key);
      if (got) return got;
      const lvl = W_.LEVEL[kind];
      const c0 = -MV, c1 = W_.COLS + MV;
      const left = c0 * this.tw - 2;
      const right = c1 * this.tw + this.sk + 2;
      // 1 px nach oben überstehen lassen: deckt Haarlinien zwischen den Reihen ab
      const top = Math.round(-this.th - lvl * this.zh) - 1;
      const bottom = Math.round(-(lower !== null ? lower : lvl) * this.zh);
      const ax = Math.floor(-left), ay = Math.floor(-top);
      const w = Math.floor(right - left), h = Math.floor(bottom - top) + 1;
      const ps = this.ps;
      const canvas = ui.makeCanvas(w * ps, h * ps);
      const c = canvas.getContext("2d");
      c.scale(ps, ps);
      c.lineJoin = "round";
      c.lineWidth = 0.6;
      const p = (x, y, z, up = 0) => {
        const q = this.proj(x, y, z);
        return [ax + q[0], ay + q[1] - up];
      };
      const poly = (col, pts) => {
        const css = rgb(col);
        c.beginPath();
        c.moveTo(pts[0][0], pts[0][1]);
        for (let i = 1; i < pts.length; i++) c.lineTo(pts[i][0], pts[i][1]);
        c.closePath();
        c.fillStyle = css;
        c.strokeStyle = css;
        c.fill();
        c.stroke();
      };
      const base = { grass: parity ? GRASS_A : GRASS_B, road: ROAD, river: parity ? WATER_A : WATER_B, rail: RAIL_BED }[kind];
      const side = { grass: GRASS_SIDE, road: ROAD_SIDE, river: WATER_B, rail: RAIL_SIDE }[kind];
      for (let cc = c0; cc < c1; cc++) {
        const out = cc < 0 || cc >= W_.COLS;
        const f = out ? 0.42 : 0.0;
        let col = out ? dim(base, f) : base;
        if (kind === "grass" && !out && PG.mod(cc + parity, 2) === 0) col = shade(col, 1.025);
        poly(col, [p(cc, 0, lvl), p(cc + 1, 0, lvl), p(cc + 1, 1, lvl, 1), p(cc, 1, lvl, 1)]);
        if (lower !== null) {
          const sc = out ? dim(side, f) : side;
          poly(sc, [p(cc, 0, lower), p(cc + 1, 0, lower), p(cc + 1, 0, lvl), p(cc, 0, lvl)]);
        }
        if (kind === "river") {
          const hh = W_.decoHash(parity * 7 + 3, cc);
          const wx = cc + (hh % 60) / 100.0;
          const wy = 0.2 + ((hh >>> 8) % 55) / 100.0;
          poly(out ? dim(WAVE, f) : WAVE, [p(wx, wy, lvl), p(wx + 0.3, wy, lvl), p(wx + 0.3, wy + 0.05, lvl), p(wx, wy + 0.05, lvl)]);
        } else if (kind === "road" && line) {
          poly(out ? dim(ROAD_LINE, f) : ROAD_LINE, [p(cc + 0.25, 0.93, lvl), p(cc + 0.75, 0.93, lvl), p(cc + 0.75, 1.0, lvl), p(cc + 0.25, 1.0, lvl)]);
        } else if (kind === "rail") {
          const sl = out ? dim(SLEEPER, f) : SLEEPER;
          for (const sx0 of [0.1, 0.6]) {
            poly(sl, [p(cc + sx0, 0.14, lvl), p(cc + sx0 + 0.28, 0.14, lvl), p(cc + sx0 + 0.28, 0.86, lvl), p(cc + sx0, 0.86, lvl)]);
          }
        }
      }
      if (kind === "rail") {
        for (const ry of [0.26, 0.66]) {
          const z1 = lvl + 0.05;
          poly(STEEL_SIDE, [p(c0, ry, lvl), p(c1, ry, lvl), p(c1, ry, z1), p(c0, ry, z1)]);
          poly(STEEL, [p(c0, ry, z1), p(c1, ry, z1), p(c1, ry + 0.07, z1), p(c0, ry + 0.07, z1)]);
        }
      }
      got = { canvas, ax, ay, w, h };
      this.strips.set(key, got);
      return got;
    }
  }

  // ----- Spiel ---------------------------------------------------------------------
  class CrossyRoadGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      if (this.mode !== "endless" && this.mode !== "daily") this.mode = "endless";
      this.shadows = this.opts.shadows !== false;
      this.daynight = this.opts.daynight !== false;
      this.data = this.load();
      this.anim = 0.0;
      this.state = SETUP;
      this.tab = "play";
      this.cursor = W_.CHAR_IDS.indexOf(this.data.selected);
      this.world = null;
      this.death = null;
      this.particles = [];
      this.ripples = [];
      this.lastDraw = null;
      this.runCoins = 0;
      this.best = 0;
      this.makeFonts();
      this.makeVoxels();
      this.layoutSetup();
    }

    makeFonts() {
      const h = this.height;
      this.fSmall = ui.font(Math.max(13, Math.min(22, Math.floor(h / 30))));
      this.fTiny = ui.font(Math.max(11, Math.min(18, Math.floor(h / 38))));
      this.fHuge = ui.font(Math.max(26, Math.floor(h / 11)), true);
      this.fNum = ui.font(Math.max(26, Math.floor(h / 10)), true);
      this.fMid = ui.font(Math.max(15, Math.min(26, Math.floor(h / 24))), true);
    }

    makeVoxels() {
      this.vox = new Voxel(Math.max(20, Math.floor(this.width / 14)), this.shadows);
      this.stageVox = null;
      this.cardVox = null;
      this.blockCache = null;
    }

    /** Pixelskala geändert (Fenstergröße)? Dann Sprites neu rendern. */
    checkScale() {
      if (this.vox.ps !== pixelScale()) this.makeVoxels();
    }

    get showHighscoreBanner() {
      return this.mode !== "daily";
    }

    get wantsEscape() {
      return this.state === SETUP && this.tab === "chars";
    }

    // ===================================================== Speicherstand
    load() {
      const raw = PG.store.get("crossy", {}) || {};
      let coins = Number.isInteger(raw.coins) ? raw.coins : 0;
      const had = Array.isArray(raw.unlocked) ? raw.unlocked : [];
      const unlocked = W_.CHAR_IDS.filter((c) => c === "chicken" || had.includes(c));
      let selected = raw.selected;
      if (!unlocked.includes(selected)) selected = "chicken";
      const d = raw.daily && typeof raw.daily === "object" ? raw.daily : {};
      const best = Number.isInteger(d.best) && d.best > 0 ? d.best : 0;
      return { coins: Math.max(0, coins), unlocked, selected, daily: { date: String(d.date || "").slice(0, 10), best } };
    }

    save() {
      PG.store.set("crossy", { coins: this.data.coins, unlocked: this.data.unlocked.slice(), selected: this.data.selected, daily: Object.assign({}, this.data.daily) });
    }

    dailyBest() {
      const d = this.data.daily;
      return d.date === PG.seedrand.todayStr() ? d.best : 0;
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    destroy() {
      if (this.state === PLAY && this.world && !this.gameOver && this.runCoins) this.save();
    }

    // ===================================================== Runde
    newRun() {
      const seed = this.mode === "daily" ? PG.seedrand.dailySeed("crossy") : Math.floor(Math.random() * 4294967296);
      this.world = new W_.World(seed);
      this.clock = 0.0;
      this.col = W_.START_COL;
      this.row = 0;
      this.facing = "up";
      this.hop = null;
      this.buffer = null;
      this.log = null;
      this.best = 0;
      this.score = 0;
      this.runCoins = 0;
      this.taken = new Set();
      this.started = false;
      this.creep = -W_.CREEP_LAG;
      this.cam = 0.0;
      this.camTarget = 0.0;
      this.death = null;
      this.particles = [];
      this.ripples = [];
      this.landT = 0.0;
      this.bumpT = 0.0;
      this.hops = 0;
      this.bellNext = 0.0;
      this.passes = new Set();
      this.record = false;
      this.gameOver = false;
      this.state = PLAY;
      this.lastDraw = null;
      this.hs = this.highscore;
      this.dayBest = this.dailyBest();
      this.playSound("click");
    }

    finishRun() {
      if (this.mode === "daily") {
        const today = PG.seedrand.todayStr();
        const d = this.data.daily;
        if (d.date !== today) {
          d.date = today;
          d.best = 0;
        }
        this.record = this.best > d.best;
        if (this.record) d.best = this.best;
      } else {
        this.record = this.best > this.hs;
      }
      this.save();
      this.gameOver = true;
      this.playSound("gameover");
      this.layoutOver();
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.gameOver) {
        this.handleOver(ev);
        return;
      }
      if (this.death) return;
      if (ev.kind === "keydown") {
        if (ev.repeat) return; // kein Dauerhüpfen durch Tastenwiederholung
        const k = ev.key;
        if (this.isAction(k, "up") || this.isAction(k, "action") || k === "Up") this.press("up");
        else if (this.isAction(k, "down") || k === "Down") this.press("down");
        else if (this.isAction(k, "left") || k === "Left") this.press("left");
        else if (this.isAction(k, "right") || k === "Right") this.press("right");
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.press("up");
      }
    }

    press(d) {
      if (this.hop) {
        this.buffer = d; // ein Hüpfer wird vorgemerkt
        return;
      }
      this.tryHop(d);
    }

    tryHop(d) {
      const [dc, dr] = W_.DIRS[d];
      this.facing = d;
      const fr = this.row, tr = fr + dr, fc = this.col;
      const cur = this.world.row(fr);
      const dst = this.world.row(tr);
      const onLog = this.log !== null;
      let drift = 0.0, tc;
      if (dr === 0) {
        tc = fc + dc;
        if (onLog) drift = cur.dir * cur.speed;
      } else if (dst.kind === W_.RIVER && dst.pads === null) {
        tc = fc;
      } else {
        tc = Math.floor(fc + 0.5);
      }
      if (this.blocked(dst, tc)) {
        this.bumpT = BUMP_T;
        this.tone(170, 0.05, "square", 0.1);
        return;
      }
      this.started = true;
      this.hop = { fc, fr, tc, tr, t: 0.0, drift, z0: this.standZ(cur), z1: this.standZ(dst) };
      this.log = null;
      this.hops += 1;
      this.tone(540 + 50 * (this.hops % 3), 0.045, "sine", 0.16);
    }

    blocked(row, tc) {
      if (row.kind === W_.RIVER && row.pads === null) return !(tc >= -0.5 && tc <= W_.COLS - 0.5);
      const c = Math.floor(tc + 0.5);
      if (c < 0 || c >= W_.COLS) return true;
      return !!(row.trees && row.trees[c]);
    }

    standZ(row) {
      const lvl = W_.LEVEL[row.kind];
      if (row.kind === W_.RIVER) return lvl + (row.pads !== null ? 0.05 : 0.26);
      return lvl;
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.anim += dt;
      if (this.state !== PLAY || !this.world) return;
      dt = Math.min(dt, 0.05);
      this.clock += dt;
      this.stepEffects(dt);
      if (this.death) {
        this.death.t += dt;
        if (this.death.t >= this.death.dur) this.finishRun();
        return;
      }
      this.landT = Math.max(0, this.landT - dt);
      this.bumpT = Math.max(0, this.bumpT - dt);
      if (this.hop) {
        this.hop.t += dt;
        if (this.hop.t >= W_.HOP_T) this.land();
      } else if (this.log) {
        const row = this.world.row(this.row);
        const [idx, seg] = this.log;
        this.col = W_.objX(row, row.objs[idx], this.clock) + seg;
        const cx = this.col + 0.5;
        if (cx < 0.0 || cx > W_.COLS) {
          this.die("edge");
          return;
        }
      }
      if (!this.death) this.collide();
      if (!this.death) this.camera(dt);
      this.sounds();
    }

    hopFrac() {
      return this.hop ? Math.min(1.0, this.hop.t / W_.HOP_T) : 1.0;
    }

    pos() {
      if (!this.hop) {
        let z = this.standZ(this.world.row(this.row));
        if (this.log) z += 0.012 * Math.sin(this.clock * 3.0 + this.log[0]);
        return [this.col, this.row, z];
      }
      const h = this.hop;
      const p = this.hopFrac();
      const x = h.fc + (h.tc - h.fc) * p + h.drift * h.t;
      const y = h.fr + (h.tr - h.fr) * p;
      const z = h.z0 + (h.z1 - h.z0) * p + W_.HOP_Z * Math.sin(Math.PI * p);
      return [x, y, z];
    }

    rowNow() {
      if (!this.hop) return this.row;
      return this.hopFrac() >= 0.5 ? this.hop.tr : this.hop.fr;
    }

    land() {
      const h = this.hop;
      this.hop = null;
      this.row = h.tr;
      this.col = h.tc + h.drift * W_.HOP_T;
      this.landT = LAND_T;
      const row = this.world.row(this.row);
      if (row.kind === W_.RIVER) {
        if (row.pads !== null) {
          const c = Math.floor(this.col + 0.5);
          if (c >= 0 && c < W_.COLS && row.pads[c]) {
            this.col = c;
            this.ripple(c + 0.5, this.row + 0.5, 0.5);
          } else {
            this.die("water");
            return;
          }
        } else {
          const hit = W_.logAt(row, this.col + 0.5, this.clock);
          if (hit === null) {
            this.die("water");
            return;
          }
          this.log = hit;
          this.col = W_.objX(row, row.objs[hit[0]], this.clock) + hit[1];
          this.ripple(this.col + 0.5, this.row + 0.5, 0.6);
        }
      } else {
        this.col = Math.floor(this.col + 0.5);
      }
      const coin = row.coin;
      if (coin && !this.taken.has(this.row) && coin[0] === Math.floor(this.col + 0.5)) {
        this.taken.add(this.row);
        this.runCoins += coin[1];
        this.data.coins += coin[1];
        this.sparkle(coin[0] + 0.5, this.row + 0.5, this.standZ(row) + 0.4, coin[1] > 1);
        this.playSound(coin[1] > 1 ? "powerup" : "eat");
      }
      if (this.row > this.best) {
        this.best = this.row;
        if (this.mode !== "daily") this.score = this.best;
        if (row.kind !== W_.RAIL && this.world.railRun(this.row - 1) >= 5) this.achEvent("crossy_train");
      }
      if (this.buffer !== null && !this.death) {
        const d = this.buffer;
        this.buffer = null;
        this.tryHop(d);
      }
    }

    collide() {
      const [x] = this.pos();
      const cx = x + 0.5;
      const rr = this.rowNow();
      const row = this.world.row(rr);
      if (row.kind === W_.ROAD) {
        const i = W_.vehicleAt(row, cx, this.clock);
        if (i >= 0) this.die("car");
      } else if (row.kind === W_.RAIL) {
        if (W_.trainHits(row, cx, this.clock)) this.die("train");
      }
    }

    camera(dt) {
      if (this.started) this.creep += W_.creepRate(this.best) * dt;
      this.creep = Math.max(this.creep, this.best - W_.CREEP_LAG);
      this.camTarget = Math.max(this.creep, this.best);
      this.cam += (this.camTarget - this.cam) * Math.min(1.0, dt * 5.0);
      if (this.started && this.camTarget - this.rowNow() > W_.EAGLE_ROWS) this.die("eagle");
    }

    danger() {
      if (!this.world || !this.started || this.death) return 0.0;
      const behind = this.camTarget - this.rowNow();
      return Math.max(0.0, Math.min(1.0, (behind - (W_.EAGLE_ROWS - 1.3)) / 1.3));
    }

    sounds() {
      let ring = false;
      for (let r = this.row - 3; r < this.row + 7; r++) {
        if (r < 0) continue;
        const row = this.world.row(r);
        if (row.kind !== W_.RAIL) continue;
        const [warn, head] = W_.trainState(row, this.clock);
        if (warn && head === null) ring = true;
        if (head !== null && Math.abs(r - this.row) <= 3) {
          const key = r + "|" + Math.floor((this.clock + row.phase) / row.period);
          if (!this.passes.has(key)) {
            this.passes.add(key);
            this.tone(90, 0.45, "noise", 0.22);
          }
        }
      }
      if (ring && this.clock >= this.bellNext) {
        this.bellNext = this.clock + 0.3;
        this.tone(1320, 0.07, "square", 0.1);
      }
    }

    die(cause) {
      const [x, y, z] = this.pos();
      this.death = { cause, t: 0.0, dur: DEATH_DUR[cause], x, y, z };
      this.hop = null;
      this.buffer = null;
      this.log = null;
      const col = MODELS.charColors[this.data.selected];
      if (cause === "car") {
        this.playSound("hit");
        this.tone(620, 0.22, "square", 0.12);
        this.burst(x + 0.5, y + 0.5, z + 0.3, col, 14, 2.2);
        this.rumble(180);
      } else if (cause === "train") {
        this.playSound("explode");
        const row = this.world.row(Math.round(y));
        this.burst(x + 0.5, y + 0.5, z + 0.4, col, 26, 4.0, row.dir * 6.0);
        this.rumble(260);
      } else if (cause === "water" || cause === "edge") {
        this.tone(160, 0.3, "noise", 0.3);
        this.tone(420, 0.12, "sine", 0.15);
        const wz = W_.LEVEL.river;
        this.death.z = wz;
        this.ripple(x + 0.5, y + 0.5, 1.0);
        this.ripple(x + 0.5, y + 0.5, 0.6);
        this.burst(x + 0.5, y + 0.5, wz + 0.1, [236, 250, 255], 22, 1.6, 0, 6.0);
        this.burst(x + 0.5, y + 0.5, wz + 0.1, [140, 214, 250], 12, 2.2, 0, 4.0);
        this.rumble(140);
      } else {
        this.playSound("shoot");
        this.rumble(220);
      }
    }

    // ----- Effekte ---------------------------------------------------------
    burst(x, y, z, col, n, speed, push = 0.0, up = 3.5) {
      const R = PG.rand;
      for (let i = 0; i < n; i++) {
        const a = R.uniform(0, PG.TAU);
        const sp = R.uniform(0.3, 1.0) * speed;
        this.particles.push([x, y, z, Math.cos(a) * sp + push * R.uniform(0.4, 1.0), Math.sin(a) * sp * 0.6, R.uniform(0.5, 1.0) * up, 0.0, R.uniform(0.6, 1.1), col, R.uniform(0.06, 0.12)]);
      }
    }

    sparkle(x, y, z, big) {
      const R = PG.rand;
      for (let i = 0; i < (big ? 16 : 9); i++) {
        const a = R.uniform(0, PG.TAU);
        const sp = R.uniform(0.6, 1.6);
        this.particles.push([x, y, z, Math.cos(a) * sp, Math.sin(a) * sp * 0.5, R.uniform(1.5, 3.0), 0.0, R.uniform(0.35, 0.6), [255, 222, 90], R.uniform(0.05, 0.09)]);
      }
    }

    ripple(x, y, strength) {
      this.ripples.push([x, y, 0.0, strength]);
    }

    stepEffects(dt) {
      const keep = [];
      for (const p of this.particles) {
        p[6] += dt;
        if (p[6] >= p[7]) continue;
        p[0] += p[3] * dt;
        p[1] += p[4] * dt;
        p[2] += p[5] * dt;
        p[5] -= 9.0 * dt;
        p[3] *= 1.0 - 1.5 * dt;
        if (p[2] < -0.2) {
          p[2] = -0.2;
          p[5] = 0.0;
        }
        keep.push(p);
      }
      this.particles = keep.slice(-240);
      this.ripples = this.ripples.filter((r) => r[2] + dt < 0.8);
      for (const r of this.ripples) r[2] += dt;
    }

    // ===================================================== Setup-Screen
    layoutSetup() {
      const W = this.width, H = this.height;
      const cx = Math.floor(W / 2);
      const tabH = Math.max(22, Math.min(32, Math.floor(H / 15)));
      const tabW = Math.min(170, Math.floor((W - 40) / 2));
      const tabY = Math.floor(H * 0.205);
      this.tabRects = [new PG.Rect(cx - tabW - 4, tabY, tabW, tabH), new PG.Rect(cx + 4, tabY, tabW, tabH)];
      const top = tabY + tabH + Math.max(10, Math.floor(H / 36));
      const bw = Math.min(Math.max(380, Math.floor(W * 0.62)), W - 40);
      const bh = Math.max(28, Math.min(46, Math.floor(H / 16)));
      this.bh = bh;
      const cardH = Math.max(96, Math.floor(H * 0.3));
      const lab = Math.max(20, this.fTiny.height + 10);
      const gap1 = lab + Math.max(0, Math.floor((H - 360) / 40));
      const gap2 = Math.max(12, Math.floor(H / 30));
      const used = cardH + gap1 + bh + gap2 + bh + 6;
      const free = H - 44 - top - used;
      const ptop = top + Math.max(0, Math.floor(free / 2));
      this.cardRect = new PG.Rect(cx - Math.floor(bw / 2), ptop, bw, cardH);
      let y = this.cardRect.bottom + gap1;
      const grp = (bw - 18) / 2.0;
      this.optW = Math.floor(grp);
      const pair = (x0) => {
        const w = (grp - 8) / 2.0;
        return [new PG.Rect(Math.floor(x0), y, Math.floor(w), bh), new PG.Rect(Math.floor(x0 + w + 8), y, Math.floor(w), bh)];
      };
      this.shadowRects = pair(cx - bw / 2);
      this.nightRects = pair(cx + bw / 2 - grp);
      y += bh + gap2;
      const sw = Math.max(190, Math.floor(bw * 0.42));
      this.startRect = new PG.Rect(cx - Math.floor(sw / 2), y, sw, bh + 6);
      // Figuren-Raster
      const gw = Math.min(W - 36, 760);
      const gap = Math.max(6, Math.floor(W / 100));
      const detailH = bh + 6;
      const bottom = H - 26 - Math.max(8, Math.floor(H / 40));
      const avail = bottom - top - detailH - gap * 2;
      const cwid = (gw - gap * (GRID_COLS - 1)) / GRID_COLS;
      const chei = Math.min(cwid * 1.12, (avail - gap) / 2.0);
      this.gridRects = W_.CHAR_IDS.map((_, i) => {
        const gx = i % GRID_COLS, gy = Math.floor(i / GRID_COLS);
        return new PG.Rect(Math.floor(cx - gw / 2 + gx * (cwid + gap)), Math.floor(top + gy * (chei + gap)), Math.floor(cwid), Math.floor(chei));
      });
      const dy = this.gridRects[this.gridRects.length - 1].bottom + gap * 2;
      this.detailRect = new PG.Rect(cx - Math.floor(gw / 2), dy, gw, detailH);
      this.buyRect = new PG.Rect(this.detailRect.right - Math.floor(gw * 0.36), dy + 3, Math.floor(gw * 0.36) - 3, detailH - 6);
    }

    handleSetup(ev) {
      if (ev.kind === "mousedown" && ev.pos) {
        for (let i = 0; i < this.tabRects.length; i++) {
          if (this.tabRects[i].collidepoint(ev.pos)) {
            this.setTab(TABS[i]);
            return;
          }
        }
      } else if (ev.kind === "keydown" && (ev.key === "Tab" || ev.key === "ISO_Left_Tab")) {
        this.setTab(this.tab === "play" ? "chars" : "play");
        return;
      }
      if (this.tab === "chars") {
        this.handleChars(ev);
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Return" || k === "space") this.newRun();
        else if (k === "Left" || this.isAction(k, "left")) this.cycleChar(-1);
        else if (k === "Right" || this.isAction(k, "right")) this.cycleChar(1);
        else if ((k === "h" || k === "H") && this.keyIsFree(k)) this.toggle("shadows");
        else if ((k === "n" || k === "N") && this.keyIsFree(k)) this.toggle("daynight");
      } else if (ev.kind === "mousedown" && ev.pos) {
        for (const [flag, rects] of [["shadows", this.shadowRects], ["daynight", this.nightRects]]) {
          for (let i = 0; i < rects.length; i++) {
            if (rects[i].collidepoint(ev.pos)) {
              if (this[flag] !== (i === 0)) this.toggle(flag);
              return;
            }
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.newRun();
        else if (this.cardRect.collidepoint(ev.pos)) this.setTab("chars");
      }
    }

    setTab(tab) {
      if (tab !== this.tab) {
        this.tab = tab;
        this.cursor = W_.CHAR_IDS.indexOf(this.data.selected);
        this.playSound("click");
      }
    }

    toggle(flag) {
      const val = !this[flag];
      this[flag] = val;
      this.saveSetting(flag, val);
      if (flag === "shadows") this.makeVoxels();
      this.playSound("click");
    }

    cycleChar(step) {
      const owned = W_.CHAR_IDS.filter((c) => this.data.unlocked.includes(c));
      const i = owned.indexOf(this.data.selected);
      this.data.selected = owned[PG.mod(i + step, owned.length)];
      this.save();
      this.playSound("select");
    }

    handleChars(ev) {
      const n = W_.CHAR_IDS.length;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Escape") this.setTab("play");
        else if (k === "Return" || k === "space") this.activateChar(this.cursor);
        else if (k === "Left" || this.isAction(k, "left")) {
          this.cursor = PG.mod(this.cursor - 1, n);
          this.playSound("move");
        } else if (k === "Right" || this.isAction(k, "right")) {
          this.cursor = PG.mod(this.cursor + 1, n);
          this.playSound("move");
        } else if (k === "Up" || this.isAction(k, "up")) {
          this.cursor = PG.mod(this.cursor - GRID_COLS, n);
          this.playSound("move");
        } else if (k === "Down" || this.isAction(k, "down")) {
          this.cursor = PG.mod(this.cursor + GRID_COLS, n);
          this.playSound("move");
        }
      } else if (ev.kind === "mousedown" && ev.pos) {
        for (let i = 0; i < this.gridRects.length; i++) {
          if (this.gridRects[i].collidepoint(ev.pos)) {
            this.cursor = i;
            if (this.data.unlocked.includes(W_.CHAR_IDS[i])) this.activateChar(i);
            else this.playSound("move");
            return;
          }
        }
        if (this.buyRect.collidepoint(ev.pos)) this.activateChar(this.cursor);
      }
    }

    activateChar(i) {
      const cid = W_.CHAR_IDS[i];
      if (this.data.unlocked.includes(cid)) {
        if (this.data.selected !== cid) {
          this.data.selected = cid;
          this.save();
        }
        this.playSound("select");
        return true;
      }
      const price = W_.PRICES[cid];
      if (this.data.coins < price) {
        this.playSound("hit");
        return false;
      }
      this.data.coins -= price;
      this.data.unlocked = W_.CHAR_IDS.filter((c) => this.data.unlocked.includes(c) || c === cid);
      this.data.selected = cid;
      this.save();
      this.playSound("win");
      const r = this.gridRects[i];
      ui.spawnBurst(r.centerx, r.centery, this.accent);
      this.achEvent("crossy_char");
      return true;
    }

    // ===================================================== Game Over
    layoutOver() {
      const W = this.width, H = this.height;
      const pw = Math.min(W - 30, Math.max(360, Math.floor(W * 0.56)));
      const bh = Math.max(28, Math.min(42, Math.floor(H / 17)));
      const rowsH = this.fHuge.height + this.fTiny.height + this.font.height + 2 * this.fSmall.height + bh + 5 * 8 + 44;
      const ph = rowsH + this.fTiny.height + 6;
      const top = Math.max(8, Math.floor((H - 56 - ph) / 2));
      this.overRect = new PG.Rect(Math.floor(W / 2) - Math.floor(pw / 2), top, pw, ph);
      const by = this.overRect.bottom - 22 - this.fTiny.height - bh;
      const gap = 8;
      const bw = (pw - 40 - 2 * gap) / 3.0;
      this.overBtns = ["again", "chars", "setup"].map((key, i) => [key, new PG.Rect(Math.floor(this.overRect.x + 20 + i * (bw + gap)), by, Math.floor(bw), bh)]);
    }

    handleOver(ev) {
      if (ev.kind === "keydown") {
        if (ev.repeat) return;
        const k = ev.key;
        if (k === "Return" || k === "space") this.newRun();
        else if (k === "f" || k === "F") this.toSetup("chars");
        else if (k === "s" || k === "S") this.toSetup("play");
      } else if (ev.kind === "mousedown" && ev.pos && this.overBtns) {
        for (const [key, rc] of this.overBtns) {
          if (rc.collidepoint(ev.pos)) {
            if (key === "again") this.newRun();
            else this.toSetup(key === "chars" ? "chars" : "play");
            return;
          }
        }
      }
    }

    toSetup(tab) {
      this.setupHs = null; // die App hat den Highscore inzwischen gesichert
      this.gameOver = false;
      this.score = 0;
      this.state = SETUP;
      this.tab = tab;
      this.cursor = W_.CHAR_IDS.indexOf(this.data.selected);
      this.playSound("click");
    }

    // ===================================================== Zeichnen: Welt
    draw(ctx) {
      this.checkScale();
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      // Nach dem Ende laufen Verkehr und Wasser als Kulisse weiter - die App
      // ruft update() bei Game Over nicht mehr auf, deshalb hier die Zeit.
      const now = ui.now();
      if (this.gameOver && this.lastDraw !== null && !this.paused) {
        const step = Math.max(0, Math.min(0.1, now - this.lastDraw));
        this.clock += step;
        this.anim += step;
      }
      this.lastDraw = now;
      this.drawWorld(ctx);
      this.drawEagle(ctx);
      if (this.daynight) this.drawNight(ctx);
      if (this.gameOver) this.drawOver(ctx); // das Panel zeigt alle Werte - kein HUD dahinter
      else this.drawHud(ctx);
    }

    setupProj() {
      const v = this.vox;
      const F = this.cam;
      this.ox = Math.round(this.width / 2 - (W_.COLS / 2) * v.tw - (F + 2.5) * v.sk);
      this.oy = Math.round(this.height * FOCUS_Y + (F + 0.5) * v.th);
    }

    scr(x, y, z) {
      const v = this.vox;
      return [this.ox + x * v.tw + y * v.sk, this.oy - y * v.th - z * v.zh];
    }

    blitAt(ctx, key, x, y, z) {
      const [px, py] = this.scr(x, y, z);
      this.vox.blit(ctx, key, px, py);
    }

    drawWorld(ctx) {
      const v = this.vox;
      const H = this.height;
      this.setupProj();
      const world = this.world;
      const rHi = Math.ceil(this.oy / v.th) + 1;
      const rLo = Math.floor((this.oy - H - 2.2 * v.zh) / v.th) - 1;
      const clock = this.clock;
      let pRow = null;
      if (!this.death) pRow = this.hop ? Math.min(this.hop.fr, this.hop.tr) : this.row;
      else if (["car", "water", "edge"].includes(this.death.cause)) pRow = Math.floor(this.death.y + 0.5);
      for (let r = rHi; r >= rLo; r--) {
        const row = world.row(r);
        const kind = row.kind;
        const lvl = W_.LEVEL[kind];
        const plvl = W_.LEVEL[world.row(r - 1).kind];
        const lower = plvl < lvl ? plvl : null;
        const line = kind === W_.ROAD && world.row(r + 1).kind === W_.ROAD;
        const st = v.strip(kind, r & 1, lower, line);
        const [sx, sy] = this.scr(0, r, 0);
        ctx.drawImage(st.canvas, Math.round(sx) - st.ax, Math.round(sy) - st.ay, st.w, st.h);
        const items = [];
        if (kind === W_.GRASS) {
          const trees = row.trees;
          for (let c = 0; c < W_.COLS; c++) if (trees[c]) items.push([1, c, "tree" + trees[c], c, lvl]);
          for (const c of [-4, -3, -2, -1, W_.COLS, W_.COLS + 1, W_.COLS + 2, W_.COLS + 3]) {
            const hh = W_.decoHash(r, c);
            if (hh % 100 < (c === -1 || c === W_.COLS ? 70 : 45)) items.push([1, c, "tree" + (1 + ((hh >>> 8) % 3)), c, lvl]);
          }
        } else if (kind === W_.ROAD) {
          for (const o of row.objs) {
            const x = W_.objX(row, o, clock);
            if (x > -7.0 && x < W_.COLS + 5.0) {
              const name = (o[1] > 2 ? "truck" : "car") + o[2];
              items.push([1, x, name + (row.dir > 0 ? "" : "<"), x, lvl]);
            }
          }
        } else if (kind === W_.RIVER) {
          if (row.pads !== null) {
            for (let c = 0; c < W_.COLS; c++) {
              if (row.pads[c]) {
                const bob = 0.012 * Math.sin(clock * 2.0 + c);
                items.push([0, c, "pad" + (W_.decoHash(r, c) % 3 === 0 ? 1 : 0), c, lvl + bob]);
              }
            }
          } else {
            row.objs.forEach((o, i) => {
              const x = W_.objX(row, o, clock);
              if (x > -7.0 && x < W_.COLS + 5.0) items.push([0, x, "log" + o[1], x, lvl + 0.012 * Math.sin(clock * 3.0 + i)]);
            });
          }
        } else {
          const [warn, head] = W_.trainState(row, clock);
          const lit = warn ? 1 + (Math.floor(clock * 6) % 2) : 0;
          items.push([1, -1.2, "signal" + lit, -1.2, lvl]);
          if (head !== null) {
            const n = row.cars + 1;
            for (let k = 0; k < n; k++) {
              let x, key;
              if (row.dir > 0) {
                x = head - W_.TRAIN_CAR * (k + 1);
                key = k === 0 ? "engine" : "wagon";
              } else {
                x = head + W_.TRAIN_CAR * k;
                key = k === 0 ? "engine<" : "wagon<";
              }
              if (x > -9.0 && x < W_.COLS + 6.0) items.push([1, x, key, x, lvl]);
            }
          }
        }
        const coin = row.coin;
        if (coin && !this.taken.has(r)) {
          const bob = 0.06 + 0.05 * Math.sin(this.anim * 3.0 + r);
          items.push([1, coin[0] - 0.01, coin[1] > 1 ? "bigcoin" : "coin", coin[0], lvl + bob]);
        }
        if (pRow === r) items.push([2, 0, null, 0, 0]);
        items.sort((a, b) => (a[0] === 2 ? 1 : a[0]) - (b[0] === 2 ? 1 : b[0]) || a[1] - b[1]);
        for (const [, , key, x, z] of items) {
          if (key === null) this.drawPlayer(ctx);
          else this.blitAt(ctx, key, x, r, z);
        }
        if (this.ripples.length) this.drawRipples(ctx, r);
      }
      this.drawParticles(ctx);
      this.drawDanger(ctx);
    }

    drawPlayer(ctx) {
      const cid = this.data.selected;
      if (this.death) {
        const d = this.death;
        if (d.cause === "car") {
          const q = Math.min(1.0, d.t / 0.08);
          this.playerSprite(ctx, cid, this.facing, d.x, d.y, d.z, 1.0 + 0.4 * q, 1.0 - 0.8 * q, false);
        } else if ((d.cause === "water" || d.cause === "edge") && d.t < 0.14) {
          const sink = d.t / 0.14;
          this.playerSprite(ctx, cid, this.facing, d.x, d.y, d.z - 0.5 * sink, 1.0, 1.0 - 0.6 * sink, false);
        }
        return;
      }
      const [x, y, z] = this.pos();
      let sx = 1.0, sy = 1.0;
      if (this.hop) {
        const k = Math.sin(Math.PI * this.hopFrac());
        sx = 1.0 - 0.1 * k;
        sy = 1.0 + 0.16 * k;
      } else if (this.landT > 0) {
        const q = this.landT / LAND_T;
        sx = 1.0 + 0.16 * q;
        sy = 1.0 - 0.24 * q;
      } else if (this.bumpT > 0) {
        const q = Math.sin((Math.PI * this.bumpT) / BUMP_T);
        sx = 1.0 + 0.08 * q;
        sy = 1.0 - 0.12 * q;
      } else {
        const k = 0.5 + 0.5 * Math.sin(this.anim * 4.0);
        sy = 1.0 - 0.03 * k;
        sx = 1.0 + 0.02 * k;
      }
      const ground = this.standZ(this.world.row(this.rowNow()));
      this.playerSprite(ctx, cid, this.facing, x, y, z, sx, sy, true, ground);
    }

    playerSprite(ctx, cid, facing, x, y, z, sx, sy, shadow = true, ground = null) {
      const v = this.vox;
      if (W_.HOVER[cid]) z += 0.14 + 0.05 * Math.sin(this.anim * 3.0);
      if (shadow && this.shadows) {
        const gz = ground === null ? z : ground;
        const lift = Math.max(0.0, z - gz);
        this.shadow(ctx, x + 0.5, y + 0.5, gz, Math.max(0.45, 1.0 - lift * 0.8));
      }
      const s = v.sprite(cid + ":" + facing, false);
      const fx = s.ax + 0.5 * v.tw + 0.5 * v.sk;
      const fy = s.ay - 0.5 * v.th;
      const [px, py] = this.scr(x + 0.5, y + 0.5, z);
      ctx.drawImage(s.canvas, px - fx * sx, py - fy * sy, s.w * sx, s.h * sy);
    }

    shadow(ctx, cx, cy, z, scale) {
      const v = this.vox;
      const [px, py] = this.scr(cx + 0.06, cy - 0.04, z);
      const w = v.tw * 0.7 * scale + 2, h = v.th * 0.6 * scale + 2;
      draw.ellipse(ctx, [0, 0, 0, 70], [px - w / 2, py - h / 2, w, h]);
    }

    drawRipples(ctx, r) {
      const v = this.vox;
      for (const [x, y, age, strength] of this.ripples) {
        if (Math.floor(y) !== r) continue;
        const f = age / 0.8;
        const rad = (0.2 + 0.75 * f) * strength;
        const [px, py] = this.scr(x, y, W_.LEVEL.river + 0.02);
        const w = rad * 2 * v.tw, h = rad * 2 * v.th * 0.9;
        if (w < 4 || h < 3) continue;
        draw.ellipse(ctx, ui.mix([255, 255, 255], [130, 206, 250], f), [px - w / 2, py - h / 2, w, h], Math.max(1, v.tw * 0.07 * (1 - f)));
      }
    }

    drawParticles(ctx) {
      const v = this.vox;
      for (const [x, y, z, , , , age, life, col, size] of this.particles) {
        const [px, py] = this.scr(x, y, z);
        const sz = Math.max(2, Math.floor(size * v.tw * (1.0 - (0.5 * age) / life)));
        ctx.fillStyle = rgb(col);
        ctx.fillRect(Math.floor(px) - sz / 2, Math.floor(py) - sz / 2, sz, sz);
        const b = Math.max(1, Math.floor(sz / 3));
        ctx.fillStyle = rgb(shade(col, 0.75));
        ctx.fillRect(Math.floor(px) - sz / 2, Math.floor(py) + sz / 2 - b, sz, b);
      }
    }

    drawDanger(ctx) {
      const k = this.danger();
      if (k <= 0) return;
      const W = this.width, H = this.height;
      const h = Math.floor(H / 3);
      const g = ctx.createLinearGradient(0, H - h, 0, H);
      const a = k * (0.6 + 0.4 * ui.pulse(6.0)) * (150 / 255);
      g.addColorStop(0, "rgba(220,30,40,0)");
      g.addColorStop(0.5, "rgba(220,30,40," + (a * 0.25).toFixed(3) + ")");
      g.addColorStop(1, "rgba(220,30,40," + a.toFixed(3) + ")");
      ctx.fillStyle = g;
      ctx.fillRect(0, H - h, W, h);
    }

    drawEagle(ctx) {
      const d = this.death;
      if (!d || d.cause !== "eagle") return;
      const v = this.vox;
      const W = this.width, H = this.height;
      const p = d.t / d.dur;
      const [px, py] = this.scr(d.x + 0.5, d.y + 0.5, d.z);
      const grab = 0.42;
      let ex, ey, carry, q;
      if (p < grab) {
        q = p / grab;
        q = q * q * (3 - 2 * q);
        ex = px + (1 - q) * W * 0.25;
        ey = -H * 0.35 + (py - v.zh * 0.6 + H * 0.35) * q;
        carry = false;
      } else {
        q = (p - grab) / (1 - grab);
        ex = px + q * W * 0.18;
        ey = py - v.zh * 0.6 - q * q * (py + H * 0.5);
        carry = true;
      }
      const cid = this.data.selected;
      if (!carry) {
        this.playerSprite(ctx, cid, this.facing, d.x, d.y, d.z, 1.0, 1.0);
        this.shadow(ctx, d.x + 0.5 + (1 - q) * 1.5, d.y + 0.5, d.z, 0.6 + q * 1.6);
      } else {
        const s = v.sprite(cid + ":" + this.facing, false);
        const fx = s.ax + 0.5 * v.tw + 0.5 * v.sk;
        const fy = s.ay - 0.5 * v.th;
        ctx.drawImage(s.canvas, ex - fx, ey + 1.25 * v.zh - fy, s.w, s.h);
      }
      const sy = 1.0 + 0.14 * Math.sin(this.anim * 16.0);
      const e = v.sprite("eagle_down", false);
      const [mx, my] = v.proj(0.0, 0.0, 0.3);
      ctx.drawImage(e.canvas, ex - (e.ax + mx), ey - (e.ay + my) * sy, e.w, e.h * sy);
    }

    // ----- Nacht -----------------------------------------------------------
    drawNight(ctx) {
      const n = W_.nightFactor(this.cam + 4.0);
      if (n <= 0.01) return;
      const W = this.width, H = this.height;
      const v = this.vox;
      const ps = v.ps;
      if (!this.nightCanvas || this.nightCanvas.width !== Math.round(W * ps)) {
        this.nightCanvas = ui.makeCanvas(W * ps, H * ps);
      }
      const oc = this.nightCanvas.getContext("2d");
      oc.setTransform(ps, 0, 0, ps, 0, 0);
      const tint = ui.mix([255, 255, 255], NIGHT_TINT, n);
      oc.fillStyle = rgb(tint);
      oc.fillRect(0, 0, W, H);
      const soft = ui.mix(tint, [255, 238, 200], 0.55);
      const bright = ui.mix(tint, [255, 246, 222], 0.92);
      const clock = this.clock;
      const rHi = Math.ceil(this.oy / v.th) + 1;
      const rLo = Math.floor((this.oy - H) / v.th) - 1;
      const poly = (col, pts) => draw.polygon(oc, col, pts.map((q) => this.scr(q[0], q[1], q[2])));
      for (let r = rHi; r >= rLo; r--) {
        const row = this.world.row(r);
        const lvl = W_.LEVEL[row.kind];
        const coin = row.coin;
        if (coin && !this.taken.has(r)) {
          const [px, py] = this.scr(coin[0] + 0.5, r + 0.5, lvl + 0.4);
          draw.circle(oc, ui.mix(tint, [255, 236, 160], 0.7), [px, py], v.tw * (coin[1] > 1 ? 0.5 : 0.36));
        }
        if (row.kind === W_.ROAD) {
          const d = row.dir;
          for (const o of row.objs) {
            const x = W_.objX(row, o, clock);
            if (!(x > -9.0 && x < W_.COLS + 7.0)) continue;
            const fx = d > 0 ? x + o[1] : x;
            poly(soft, [[fx, r + 0.2, lvl], [fx + 3.6 * d, r - 0.25, lvl], [fx + 3.6 * d, r + 1.25, lvl], [fx, r + 0.8, lvl]]);
            poly(bright, [[fx, r + 0.28, lvl], [fx + 2.4 * d, r + 0.05, lvl], [fx + 2.4 * d, r + 0.95, lvl], [fx, r + 0.72, lvl]]);
          }
        } else if (row.kind === W_.RAIL) {
          const [warn, head] = W_.trainState(row, clock);
          if (head !== null) {
            const d = row.dir;
            poly(soft, [[head, r + 0.2, lvl], [head + 6.0 * d, r - 0.4, lvl], [head + 6.0 * d, r + 1.4, lvl], [head, r + 0.8, lvl]]);
          }
          if (warn && Math.floor(clock * 6) % 2 === 0) {
            const [px, py] = this.scr(-1.0, r + 0.1, lvl + 1.1);
            draw.circle(oc, ui.mix(tint, [255, 150, 140], 0.8), [px, py], v.tw * 0.5);
          }
        }
      }
      if (!this.death || this.death.cause !== "eagle") {
        const [x, y, z] = this.death ? [this.death.x, this.death.y, this.death.z] : this.pos();
        const [px, py] = this.scr(x + 0.5, y + 0.5, z + 0.3);
        const w = v.tw * 1.9, h = v.th * 1.9;
        draw.ellipse(oc, soft, [px - w / 2, py - h / 2, w, h]);
      }
      ctx.save();
      ctx.globalCompositeOperation = "multiply";
      ctx.drawImage(this.nightCanvas, 0, 0, W, H);
      ctx.restore();
    }

    // ===================================================== HUD
    shadowText(ctx, font, txt, x, y, col, anchor = "topleft") {
      ui.text(ctx, txt, x + 1, y + 2, font, [20, 24, 36], anchor);
      ui.text(ctx, txt, x, y, font, col, anchor);
    }

    coinIcon(ctx, cx, cy, r) {
      draw.circle(ctx, [176, 120, 20], [cx + 1, cy + 2], r);
      draw.circle(ctx, [246, 190, 44], [cx, cy], r);
      draw.circle(ctx, [255, 226, 110], [cx, cy], Math.max(2, r - 3));
      draw.rect(ctx, [236, 170, 36], [cx - Math.max(1, Math.floor(r / 4)), cy - Math.floor(r / 2), Math.max(2, Math.floor(r / 2)), r]);
    }

    drawHud(ctx) {
      const W = this.width, H = this.height;
      const pad = Math.max(8, Math.floor(W / 60));
      const txt = String(this.best);
      this.shadowText(ctx, this.fNum, txt, pad + 1, pad + 1, [255, 255, 255]);
      const y = pad + this.fNum.height - 2;
      let best = this.mode === "daily" ? this.dayBest : this.hs;
      best = Math.max(best, this.best);
      const lab = this.mode === "daily" ? t("cr.hud_top_daily", { n: best }) : t("cr.hud_top", { n: best });
      this.shadowText(ctx, this.fSmall, lab, pad, y, [255, 236, 150]);
      // Münzen rechts oben
      const coins = String(this.data.coins);
      const cw = this.fMid.width(coins);
      const cy = pad + Math.floor(this.fMid.height / 2) + 2;
      const right = W - pad;
      this.shadowText(ctx, this.fMid, coins, right - cw, cy - this.fMid.height / 2, [255, 255, 255]);
      const r = Math.max(7, Math.floor(this.fMid.height / 2) - 1);
      this.coinIcon(ctx, right - cw - r - 8, cy, r);
      if (this.runCoins) {
        const plus = "+" + this.runCoins;
        this.shadowText(ctx, this.fSmall, plus, right - this.fSmall.width(plus), cy + r + 4, [255, 226, 110]);
      }
      if (this.mode === "daily") {
        const tag = t("cr.daily_tag", { date: PG.seedrand.todayStr() });
        const tw = this.fTiny.width(tag);
        const box = [W / 2 - tw / 2 - 9, pad - 4, tw + 18, this.fTiny.height + 8];
        draw.rect(ctx, [20, 24, 40, 150], box, 0, box[3] / 2);
        ui.text(ctx, tag, W / 2, pad, this.fTiny, [255, 255, 255], "midtop");
      }
      if (!this.started && !this.death && !this.gameOver) {
        const hint = t("cr.hint");
        const [f, s] = this.fit(this.fSmall, hint, W - 40);
        const tw = f.width(s);
        const bottom = H - pad - 6;
        const box = [W / 2 - tw / 2 - 12, bottom - f.height - 5, tw + 24, f.height + 10];
        draw.rect(ctx, [20, 24, 40, Math.floor(120 + 60 * ui.pulse(2.5))], box, 0, box[3] / 2);
        ui.text(ctx, s, W / 2, bottom, f, [255, 255, 255], "midbottom");
      }
    }

    /** [font, text] passend auf maxw: kleinere Schrift, zur Not gekürzt. */
    fit(font, txt, maxw) {
      if (font.width(txt) <= maxw) return [font, txt];
      for (const smaller of [this.fSmall, this.fTiny]) {
        if (smaller.height < font.height && smaller.width(txt) <= maxw) return [smaller, txt];
      }
      let kurz = txt;
      while (kurz.length > 2 && this.fTiny.width(kurz + "...") > maxw) kurz = kurz.slice(0, -1);
      return [this.fTiny, kurz + "..."];
    }

    fitText(ctx, font, txt, x, y, col, maxw, anchor) {
      const [f, s] = this.fit(font, txt, maxw);
      return ui.text(ctx, s, x, y, f, col, anchor);
    }

    btn(ctx, rc, text, on) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      this.fitText(ctx, this.fSmall, text, rc.centerx, rc.centery, on ? ui.TEXT : ui.TEXT_DIM, rc.w - 10, "center");
    }

    drawOver(ctx) {
      if (!this.overRect) this.layoutOver();
      const pr = this.overRect;
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 236], pr, 0, 14);
      draw.rect(ctx, this.accent, pr, 2, 14);
      const cx = pr.centerx;
      const maxw = pr.w - 28;
      let y = pr.y + 14;
      const cause = this.death ? this.death.cause : "car";
      for (const [f, txt, col] of [[this.fHuge, t("common.game_over"), ui.RED], [this.fTiny, t("cr.cause." + cause), ui.TEXT_DIM], [this.font, t("common.points", { score: this.best }), ui.TEXT]]) {
        const rc = this.fitText(ctx, f, txt, cx, y, col, maxw, "midtop");
        y += rc.h + 8;
      }
      let line = this.mode === "daily" ? t("cr.best_daily", { n: Math.max(this.best, this.dailyBest()) }) : t("cr.best", { n: Math.max(this.best, this.hs) });
      if (this.record && this.best > 0) line += "  ·  " + t("cr.new_record");
      let rc = this.fitText(ctx, this.fSmall, line, cx, y, ui.GOLD, maxw, "midtop");
      y += rc.h + 8;
      line = t("cr.coins_run", { n: this.runCoins, total: this.data.coins });
      const [f, s] = this.fit(this.fSmall, line, maxw);
      const rr = Math.max(6, Math.floor(f.height / 2) - 2);
      const tw = f.width(s);
      const tx = cx - tw / 2 + rr + 3;
      this.coinIcon(ctx, tx - rr - 6, y + f.height / 2, rr);
      ui.text(ctx, s, tx, y, f, ui.TEXT, "topleft");
      const chars = t("cr.tab_chars");
      const labels = { again: t("cr.btn_again"), chars: chars.charAt(0) + chars.slice(1).toLowerCase(), setup: t("cr.btn_setup") };
      for (const [key, r] of this.overBtns) this.btn(ctx, r, labels[key], key === "again");
      this.fitText(ctx, this.fTiny, t("cr.over_hint"), cx, pr.bottom - 10, ui.TEXT_DIM, maxw, "midbottom");
    }

    // ===================================================== Setup zeichnen
    drawSetup(ctx) {
      const W = this.width, H = this.height;
      ui.drawBackground(ctx, W, H);
      const cx = Math.floor(W / 2);
      ui.text(ctx, "CROSSY ROAD", cx, Math.floor(H * 0.1), this.fHuge, this.accent, "center");
      this.fitText(ctx, this.fSmall, t(this.tab === "play" ? "cr.subtitle" : "cr.chars_sub"), cx, Math.floor(H * 0.165), ui.TEXT_DIM, W - 30, "center");
      this.tabRects.forEach((rc, i) => this.btn(ctx, rc, t("cr.tab_" + TABS[i]), this.tab === TABS[i]));
      if (this.tab === "chars") this.drawChars(ctx);
      else this.drawPlayTab(ctx);
    }

    stage() {
      const tw = Math.max(24, Math.floor(this.cardRect.h * 0.42));
      if (!this.stageVox || this.stageVox.tw !== tw || this.stageVox.ps !== pixelScale()) {
        this.stageVox = new Voxel(tw, this.shadows);
        this.blockCache = null;
      }
      return this.stageVox;
    }

    drawPlayTab(ctx) {
      const W = this.width, H = this.height;
      const cx = Math.floor(W / 2);
      const card = this.cardRect;
      ui.drawPanel(ctx, card, { radius: 12 });
      const v = this.stage();
      if (!this.blockCache) {
        this.blockCache = v.render([[0.0, 0.0, -0.55, 1.0, 1.0, 0.0, [112, 160, 64]], [0.0, 0.0, 0.0, 1.0, 1.0, 0.16, GRASS_A]], false);
      }
      const b = this.blockCache;
      const stageW = Math.floor(v.tw + v.sk);
      const sx0 = card.x + 14 + Math.floor((card.h - 16 - stageW) / 2);
      const baseY = card.bottom - 10 - Math.floor(0.55 * v.zh);
      ctx.drawImage(b.canvas, sx0 - b.ax, baseY - b.ay, b.w, b.h);
      const faces = ["down", "right", "down", "left"];
      const ph = PG.mod(this.anim, 4.8) / 1.2;
      const facing = faces[Math.floor(ph)];
      const frac = ph - Math.floor(ph);
      let z = 0.16, sy = 1.0;
      if (frac > 0.72) {
        const k = Math.sin((Math.PI * (frac - 0.72)) / 0.28);
        z += 0.35 * k;
        sy = 1.0 + 0.12 * k;
      }
      const cid = this.data.selected;
      if (W_.HOVER[cid]) z += 0.12 + 0.05 * Math.sin(this.anim * 3.0);
      const s = v.sprite(cid + ":" + facing, false);
      const fx = s.ax + 0.5 * v.tw + 0.5 * v.sk;
      const fy = s.ay - 0.5 * v.th;
      const ppx = sx0 + 0.5 * v.tw + 0.5 * v.sk;
      const ppy = baseY - 0.5 * v.th - z * v.zh;
      ctx.drawImage(s.canvas, ppx - fx, ppy - fy * sy, s.w, s.h * sy);
      // Texte rechts neben der Bühne
      const tx = card.x + card.h + 6;
      const tw = card.right - 14 - tx;
      const lines = [[this.fMid, t(this.mode === "daily" ? "cr.mode.daily" : "cr.mode.endless"), this.accent]];
      lines.push([this.fTiny, this.mode === "daily" ? t("cr.mode_desc.daily", { date: PG.seedrand.todayStr() }) : t("cr.mode_desc.endless"), ui.TEXT_DIM]);
      if (this.mode === "daily") lines.push([this.fSmall, t("cr.best_daily", { n: this.dailyBest() }), ui.GOLD]);
      else {
        // Highscore nur einmal je Besuch aus dem Speicher lesen
        if (this.setupHs == null) this.setupHs = this.highscore;
        lines.push([this.fSmall, t("cr.best", { n: this.setupHs }), ui.GOLD]);
      }
      lines.push([this.fSmall, t("cr.char_line", { name: t("cr.char." + cid) }), ui.TEXT]);
      const fitted = lines.map(([f, txt, col]) => [...this.fit(f, txt, tw), col]);
      const total = fitted.reduce((a, [f]) => a + f.height, 0) + 6 * (fitted.length - 1) + this.fMid.height + 6;
      let y = card.y + Math.max(8, Math.floor((card.h - total) / 2));
      for (const [f, txt, col] of fitted) {
        ui.text(ctx, txt, tx, y, f, col, "topleft");
        y += f.height + 6;
      }
      const coins = String(this.data.coins);
      const r = Math.max(6, Math.floor(this.fMid.height / 2) - 2);
      this.coinIcon(ctx, tx + r, y + Math.floor(this.fMid.height / 2), r);
      ui.text(ctx, coins, tx + 2 * r + 8, y, this.fMid, ui.TEXT, "topleft");
      for (const [rects, key, on] of [[this.shadowRects, "cr.lbl_shadows", this.shadows], [this.nightRects, "cr.lbl_daynight", this.daynight]]) {
        const mid = Math.floor((rects[0].left + rects[1].right) / 2);
        this.fitText(ctx, this.fTiny, t(key), mid, rects[0].top - 4, ui.TEXT_DIM, this.optW, "midbottom");
        rects.forEach((rc, i) => this.btn(ctx, rc, i === 0 ? t("common.on") : t("common.off"), on === (i === 0)));
      }
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      this.fitText(ctx, this.fTiny, t("cr.setup_hint"), cx, H - 30, ui.TEXT_FAINT, W - 20, "center");
      this.fitText(ctx, this.fTiny, t("cr.hint"), cx, H - 12, ui.mix(this.accent, ui.TEXT, 0.45), W - 20, "center");
    }

    drawChars(ctx) {
      const W = this.width, H = this.height;
      const cx = Math.floor(W / 2);
      const r0 = this.gridRects[0];
      const tw = Math.max(16, Math.floor(Math.min(r0.w * 0.62, (r0.h - this.fTiny.height) * 0.5)));
      if (!this.cardVox || this.cardVox.tw !== tw || this.cardVox.ps !== pixelScale()) this.cardVox = new Voxel(tw, false);
      const v = this.cardVox;
      const coins = this.data.coins;
      this.gridRects.forEach((rc, i) => {
        const cid = W_.CHAR_IDS[i];
        const owned = this.data.unlocked.includes(cid);
        const chosen = cid === this.data.selected;
        const cur = i === this.cursor;
        draw.rect(ctx, cur ? ui.BTN_SEL : ui.BTN, rc, 0, 10);
        draw.rect(ctx, cur || chosen ? this.accent : ui.BORDER, rc, cur || chosen ? 2 : 1, 10);
        const [nf, ns] = this.fit(this.fTiny, t("cr.char." + cid), rc.w - 6);
        const ny = rc.bottom - nf.height - 4;
        const bob = cur ? Math.max(0.0, Math.sin(this.anim * 6.0)) * 0.18 : 0.0;
        const s = v.sprite(cid + ":down", false);
        const fx = s.ax + 0.5 * v.tw + 0.5 * v.sk;
        const fy = s.ay - 0.5 * v.th;
        const px = rc.centerx - 0.25 * v.sk;
        const roomTop = rc.y + 4;
        const feet = roomTop + Math.floor((ny - roomTop) / 2) + Math.floor(0.55 * v.zh);
        const py = Math.min(ny - Math.floor(0.3 * v.th), feet) - Math.floor(bob * v.zh);
        ctx.save();
        if (!owned) ctx.filter = "brightness(0.35) saturate(0.4)";
        ctx.drawImage(s.canvas, px - fx, py - fy, s.w, s.h);
        ctx.restore();
        ui.text(ctx, ns, rc.centerx, rc.bottom - 4, nf, owned ? ui.TEXT : ui.TEXT_DIM, "midbottom");
        if (chosen) {
          draw.circle(ctx, this.accent, [rc.right - 11, rc.y + 11], 6);
          draw.lines(ctx, ui.BG_TOP, false, [[rc.right - 14, rc.y + 11], [rc.right - 12, rc.y + 14], [rc.right - 8, rc.y + 8]], 2);
        } else if (!owned) {
          const price = String(W_.PRICES[cid]);
          const col = coins >= W_.PRICES[cid] ? ui.GOLD : ui.TEXT_FAINT;
          const pr = ui.text(ctx, price, rc.right - 6, rc.y + 5, this.fTiny, col, "topright");
          const rr = Math.max(4, Math.floor(this.fTiny.height / 2) - 2);
          this.coinIcon(ctx, pr.x - rr - 3, pr.centery, rr);
        }
      });
      const d = this.detailRect;
      ui.drawPanel(ctx, d, { radius: 10, shadow: false });
      const cid = W_.CHAR_IDS[this.cursor];
      const owned = this.data.unlocked.includes(cid);
      const coinTxt = String(coins);
      const r = Math.max(6, Math.floor(this.fMid.height / 2) - 2);
      this.coinIcon(ctx, d.x + 12 + r, d.centery, r);
      ui.text(ctx, coinTxt, d.x + 2 * r + 18, d.centery, this.fMid, ui.TEXT, "midleft");
      const nameX = d.x + 2 * r + 30 + this.fMid.width(coinTxt);
      this.fitText(ctx, this.font, t("cr.char." + cid), nameX, d.centery, ui.TEXT, this.buyRect.x - nameX - 8, "midleft");
      let label, on;
      if (cid === this.data.selected) [label, on] = [t("cr.chosen"), false];
      else if (owned) [label, on] = [t("cr.choose"), true];
      else [label, on] = [t("cr.buy", { n: W_.PRICES[cid] }), coins >= W_.PRICES[cid]];
      this.btn(ctx, this.buyRect, label, on);
      this.fitText(ctx, this.fTiny, t("cr.chars_hint"), cx, H - 13, ui.TEXT_FAINT, W - 20, "center");
    }
  }

  PG.register(CrossyRoadGame, {
    id: "CrossyRoadGame",
    key: "crossy",
    name: "Crossy Road",
    modes: [["endless", "cr.mode.endless"], ["daily", "cr.mode.daily"]],
    settingsKey: "crossy",
    defaults: { shadows: true, daynight: true },
  });
})();
