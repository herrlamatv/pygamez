/*
 * geodash.js - Geometry Dash (Port von games/geodash.py)
 * =====================================================
 * Rhythmus-Plattformer mit 8 eingebauten Leveln (Leicht bis Dämon),
 * Übungsmodus, prozeduralem Soundtrack und Level-Editor (geodash_edit.js).
 *
 * Die Physik steckt in geodash_core.js (bitgleich mit Python): feste
 * 240-Hz-Schritte, Eingaben mit Zeitstempel (performance.now) werden dem
 * Schritt zugeordnet, in dem sie passiert sind. Tod und Neustart laufen
 * intern (nicht über gameOver), damit update() weiterläuft.
 *
 * Punkte (Highscore) = Sterne gesamt (Level-Sterne + Münzen der eingebauten
 * Level) - wächst nur. Fortschritt in PG.store "geodash":
 *   {best: {id: [normal, übung]}, coins: {id: maske}, attempts: {id: n}, jumps}
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const core = PG.gdCore;
  const gdraw = PG.gdDraw;
  const music = PG.gdMusic;

  const SELECT = "select", PLAY = "play", COMPLETE = "complete", EDIT = "edit";
  const TABS = ["play", "levels"];
  const DIFF_KEYS = ["easy", "normal", "hard", "harder", "insane", "demon"];
  const DEATH_PAUSE = 0.85;
  const AUTO_CHECK_STEPS = 2 * core.HZ;
  const VIEW_ROWS = 12.5;

  const R = (x, y, w, h) => new PG.Rect(Math.trunc(x), Math.trunc(y), Math.trunc(w), Math.trunc(h));
  const coinsOf = (mask) => [1, 2, 4].filter((b) => mask & b).length;

  function fit(font, text, w) {
    if (font.width(text) <= w) return text;
    let s = text;
    while (s.length > 2 && font.width(s + "...") > w) s = s.slice(0, -1);
    return s + "...";
  }

  function builtinLevels() {
    const raw = PG.gdLevels || [];
    const out = raw.filter((d) => d && d.id).map((d) => core.normalizeLevel(d));
    if (out.length) return out;
    return [core.normalizeLevel({ id: "lama-launch", name: "Lama Launch", difficulty: 0, stars: 1, length: 110, objects: [0, 1, 2, 3, 4, 5, 6, 7].map((i) => ["spike", 20 + 10 * i, 0, 0]) })];
  }

  class GeometryDashGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    init() {
      this.renderer = new gdraw.Renderer();
      this.thumbs = new Map();
    }

    reset() {
      this.score = 0;
      this.gameOver = false;
      this.levels = builtinLevels();
      this.sel = Math.max(0, Math.min(this.levels.length - 1, Math.trunc(Number(this.opts.last_level) || 0)));
      this.practice = this.mode === "practice";
      this.musicOn = !!this.opts.music;
      this.showBar = !!this.opts.progress_bar;
      this.autoCp = !!this.opts.auto_checkpoints;
      this.setupTab = "play";
      this.lists = null;
      this.editor = null;
      this.test = null;
      this.t = 0;
      this.cardAnim = 0;
      this.cardDir = 1;
      this.loadProgress();
      this.buildFonts();
      this.layout();
      this.state = SELECT;
      this.cur = null;
      this.lv = null;
      this.run = null;
      this.voices = null;
      this.voicesKey = "";
      this.particles = [];
      this.musicT = 0;
    }

    destroy() {
      PG.audio.stopMusic(this);
      this.saveProgress();
    }

    get wantsEscape() {
      return this.state === EDIT || this.state === COMPLETE || !!this.test || (this.state === SELECT && this.setupTab === "levels");
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    buildFonts() {
      const h = this.height;
      this.fHuge = ui.font(Math.max(24, Math.floor(h / 13)), true);
      this.fMid = ui.font(Math.max(16, Math.floor(h / 24)), true);
      this.fSmall = ui.font(Math.max(14, Math.floor(h / 32)));
      this.fTiny = ui.font(Math.max(11, Math.floor(h / 42)));
      this.fAttempt = ui.font(Math.max(18, Math.floor(h / 17)), true);
      this.fBtn = ui.font(Math.max(15, Math.floor(h / 30)), true); // START
    }

    // ===================================================== Fortschritt
    loadProgress() {
      const data = PG.store.get("geodash", {}) || {};
      const num = (v, lo, hi) => Math.max(lo, Math.min(hi, Math.trunc(Number(v) || 0)));
      this.best = {};
      for (const [k, v] of Object.entries(data.best || {})) {
        if (Array.isArray(v) && v.length >= 2) this.best[k] = [num(v[0], 0, 100), num(v[1], 0, 100)];
      }
      this.coins = {};
      for (const [k, v] of Object.entries(data.coins || {})) this.coins[k] = num(v, 0, 7) & 7;
      this.attempts = {};
      for (const [k, v] of Object.entries(data.attempts || {})) this.attempts[k] = num(v, 0, 1e9);
      this.jumps = num(data.jumps, 0, 1e12);
      this.dirty = false;
      this.score = this.totalStars();
    }

    saveProgress() {
      if (!this.dirty) return;
      PG.store.set("geodash", { best: this.best, coins: this.coins, attempts: this.attempts, jumps: this.jumps });
      this.dirty = false;
    }

    totalStars() {
      let n = 0;
      for (const d of this.levels) {
        if ((this.best[d.id] || [0, 0])[0] >= 100) n += Math.trunc(Number(d.stars) || 0);
        n += coinsOf(this.coins[d.id] || 0);
      }
      return n;
    }

    maxStars() {
      return this.levels.reduce((s, d) => s + Math.trunc(Number(d.stars) || 0) + 3, 0);
    }

    bestKey() {
      if (!this.cur) return "";
      return this.curKind === "main" ? this.cur.id : "ugc:" + (this.cur.id || "");
    }

    // ===================================================== Level starten
    startLevel(level, kind = "main", startBlock = null, practice = null) {
      this.cur = core.normalizeLevel(level);
      this.curKind = kind;
      this.lv = core.compile(this.cur);
      gdraw.blockMasks(this.lv);
      this.startBlock = startBlock;
      if (practice !== null) this.practice = practice;
      this.checkpoints = [];
      this.sessionAttempts = 0;
      this.sessionJumps = 0;
      this.sessionTime = 0;
      this.bpm = music.styleBpm(this.cur);
      this.state = PLAY;
      this.newAttempt(false);
      this.playSound("select");
    }

    newAttempt(fromCheckpoint) {
      if (fromCheckpoint && this.checkpoints.length) this.run = core.copyState(this.checkpoints[this.checkpoints.length - 1]);
      else this.run = core.newState(this.lv, this.startBlock);
      this.sessionAttempts++;
      if (this.curKind !== "test") {
        const key = this.bestKey();
        this.attempts[key] = (this.attempts[key] || 0) + 1;
        this.dirty = true;
      }
      this.acc = 0;
      this.queue = [];
      this.sources = new Set();
      this.lastUpdate = performance.now();
      this.deadT = -1;
      this.lastCpStep = this.run.step;
      this.angle = 0;
      this.trail = [];
      const [px, py] = this.playerBlocks();
      this.camY = this.camTarget(py);
      this.attemptX = px + 2.5;
      this.attemptY = py + 4.0;
      this.newBest = null;
      this.shock = null;
      if (!(fromCheckpoint && this.practice && PG.audio.musicPlaying(this))) this.startMusic();
    }

    startMusic() {
      PG.audio.stopMusic(this);
      this.musicT = 0;
      if (!this.musicOn || !this.cur || this.cur.music === "none") return;
      const key = [this.cur.music, this.bpm, this.cur.id || ""].join("|");
      if (this.voicesKey !== key) {
        try {
          this.voices = music.buildVoices(this.cur.music, this.bpm, this.cur.id || "");
        } catch (e) {
          this.voices = null;
        }
        this.voicesKey = key;
      }
      if (this.voices) PG.audio.music(this.voices, this, 1);
    }

    playerBlocks() {
      return [this.run.x / core.B, this.run.y / core.B];
    }

    // ===================================================== Eingabe
    isJumpKey(k) {
      return ["space", "Up", "w", "W", "Return", "KP_Enter"].includes(k) || this.isAction(k, "up") || this.isAction(k, "action");
    }

    handleEvent(ev) {
      if (this.state === EDIT && this.editor) return this.editor.handle(ev);
      if (this.state === SELECT) return this.handleSelect(ev);
      if (this.state === COMPLETE) return this.handleComplete(ev);
      if (this.state !== PLAY) return;
      const now = performance.now();
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (ev.repeat) return;
        if (k === "Escape" && this.test) return this.backToEditor();
        if (this.isJumpKey(k)) {
          this.queue.push([now, true, "k:" + String(k).toLowerCase()]);
          return;
        }
        const low = String(k).toLowerCase();
        if (low === "z" && this.keyIsFree(k) && this.practice) this.placeCheckpoint();
        else if (low === "x" && this.keyIsFree(k) && this.practice) this.removeCheckpoint();
        else if (low === "p" && this.keyIsFree(k)) this.togglePracticeLive();
        else if (low === "r" && this.keyIsFree(k)) this.restartNow();
        else if ((k === "BackSpace" || low === "q") && this.keyIsFree(k)) {
          if (this.test) this.backToEditor();
          else this.leaveToSelect();
        }
      } else if (ev.kind === "keyup") {
        if (this.isJumpKey(ev.key)) this.queue.push([now, false, "k:" + String(ev.key).toLowerCase()]);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.queue.push([now, true, "mouse"]);
      } else if (ev.kind === "mouseup" && ev.button === 1) {
        this.queue.push([now, false, "mouse"]);
      }
    }

    togglePracticeLive() {
      if (this.test) this.test.verify = false;
      this.practice = !this.practice;
      this.checkpoints = [];
      this.playSound("select");
      if (this.practice) {
        if (this.run && !this.run.dead && !this.run.won && (this.run.ground || this.run.mode !== core.CUBE)) this.checkpoints.push(core.copyState(this.run));
      } else this.newAttempt(false);
    }

    placeCheckpoint() {
      const s = this.run;
      if (!s || s.dead || s.won) return;
      this.checkpoints.push(core.copyState(s));
      if (this.checkpoints.length > 200) this.checkpoints.shift();
      this.lastCpStep = s.step;
      this.playSound("powerup");
    }

    removeCheckpoint() {
      if (this.checkpoints.length) {
        this.checkpoints.pop();
        this.playSound("click");
      }
    }

    restartNow() {
      if (!this.run) return;
      this.recordBest(core.progress(this.run, this.lv));
      this.newAttempt(this.practice);
    }

    leaveToSelect() {
      PG.audio.stopMusic(this);
      this.saveProgress();
      this.state = SELECT;
      this.cur = null;
      this.run = null;
      this.score = this.totalStars();
      this.playSound("click");
    }

    // ===================================================== Update
    update(dt) {
      this.t += dt;
      this.updateParticles(dt);
      if (this.state === EDIT) {
        if (this.editor) this.editor.update(dt);
        return;
      }
      if (this.state === SELECT) {
        this.cardAnim = Math.max(0, this.cardAnim - dt * 4);
        if (this.setupTab === "levels" && this.lists) this.lists.update(dt);
        return;
      }
      if (this.state === COMPLETE) {
        this.completeT += dt;
        this.musicT += dt;
        return;
      }
      if (this.state !== PLAY || !this.run) return;
      this.musicT += dt;
      const now = performance.now();
      this.lastUpdate = now;
      const s = this.run;
      if (s.dead) {
        this.queue = [];
        this.deadT += dt;
        if (this.deadT >= DEATH_PAUSE) this.newAttempt(this.practice);
        return;
      }
      this.acc += dt;
      const n = Math.floor(this.acc * core.HZ + 1e-9);
      if (n <= 0) return;
      this.acc -= n / core.HZ;
      // Die n Schritte decken (in ms) t0 .. jetzt minus Akkumulator-Rest ab;
      // Schritt i beginnt bei t0 + i/240 s. Jede Eingabe landet im Schritt, in
      // dem sie passiert ist - was danach kam, wartet auf den nächsten Frame.
      const t0 = now - Math.max(0, this.acc) * 1000 - (n / core.HZ) * 1000;
      const events = this.queue.sort((a, b) => a[0] - b[0]);
      this.queue = [];
      let ei = 0;
      for (let i = 0; i < n; i++) {
        let pressed = false;
        while (ei < events.length) {
          const [te, down, src] = events[ei];
          if (Math.floor(((te - t0) / 1000) * core.HZ) > i) break;
          ei++;
          if (down) {
            if (!this.sources.has(src)) {
              this.sources.add(src);
              pressed = true;
              this.jumps++;
              this.sessionJumps++;
              this.dirty = true;
            }
          } else this.sources.delete(src);
        }
        const held = this.sources.size > 0;
        core.step(s, this.lv, held || pressed, pressed);
        if (s.dead || s.won) break;
        if (this.practice && this.autoCp && s.step - this.lastCpStep >= AUTO_CHECK_STEPS && (s.ground || s.mode !== core.CUBE)) {
          this.checkpoints.push(core.copyState(s));
          this.lastCpStep = s.step;
        }
      }
      if (s.dead || s.won) {
        // übrig gebliebene Ereignisse (Loslassen nach dem Tod) nicht verlieren
        for (const [, down, src] of events.slice(ei)) {
          if (down) this.sources.add(src);
          else this.sources.delete(src);
        }
      } else this.queue = events.slice(ei).concat(this.queue);
      this.runTime = (this.runTime || 0) + dt;
      this.sessionTime += dt;
      this.updateVisuals(dt);
      if (s.dead) this.onDeath();
      else if (s.won) this.onWin();
    }

    updateVisuals(dt) {
      const s = this.run;
      const [px, py] = this.playerBlocks();
      const held = this.sources.size > 0;
      const g = s.grav;
      const bps = (core.SPEEDS[s.speed] * core.HZ) / core.B;
      if (s.mode === core.CUBE) {
        if (s.ground) {
          const target = Math.round(this.angle / 90) * 90;
          this.angle += (target - this.angle) * Math.min(1, dt * 22);
        } else this.angle += 420 * dt * g;
      } else if (s.mode === core.BALL) this.angle += (bps / 0.5) * 57.3 * dt * g;
      else if (s.mode === core.SHIP) {
        const vy = (s.vy * core.HZ) / core.B;
        const want = ((Math.atan2(vy, bps) * 180) / Math.PI) * 0.8;
        this.angle += (want - this.angle) * Math.min(1, dt * 14);
      } else if (s.mode === core.WAVE) this.angle = (held ? 45 : -45) * g;
      else this.angle *= Math.max(0, 1 - dt * 10);
      if (s.mode !== core.CUBE) {
        this.trail.push([px, py]);
        const limit = s.mode === core.WAVE ? 60 : 14;
        if (this.trail.length > limit) this.trail.shift();
      } else this.trail = [];
      const ty = this.camTarget(py);
      this.camY += (ty - this.camY) * Math.min(1, dt * (s.ceil ? 5 : 3.5));
    }

    ts() {
      return Math.max(8, Math.trunc(this.height / VIEW_ROWS));
    }

    camTarget(py) {
      const viewH = this.height / this.ts();
      const s = this.run;
      if (s && s.ceil > 0) return s.ceil / core.B / 2 - viewH / 2;
      const base = -viewH * 0.18;
      if (s && s.grav < 0) return Math.max(base, py - viewH * 0.62);
      return Math.max(base, py - viewH * 0.45);
    }

    // ----- Tod und Ziel ------------------------------------------------------
    onDeath() {
      this.deadT = 0;
      const [px, py] = this.playerBlocks();
      this.burst(px + 0.5, py + 0.5, gdraw.COL_P1, 34);
      this.burst(px + 0.5, py + 0.5, gdraw.COL_P2, 18);
      this.shock = [px + 0.5, py + 0.5, this.t];
      this.playSound("explode");
      this.rumble(160);
      const pct = core.progress(this.run, this.lv);
      if (this.recordBest(pct)) this.newBest = [pct, this.t];
      if (!this.practice) PG.audio.stopMusic(this);
    }

    recordBest(pct) {
      if (this.curKind === "test" || this.startBlock) return false;
      const key = this.bestKey();
      if (!this.best[key]) this.best[key] = [0, 0];
      const idx = this.practice ? 1 : 0;
      if (pct > this.best[key][idx]) {
        this.best[key][idx] = pct;
        this.dirty = true;
        this.saveProgress();
        return true;
      }
      return false;
    }

    onWin() {
      const s = this.run;
      this.state = COMPLETE;
      this.completeT = 0;
      this.playSound("win");
      this.rumble(220);
      const [px, py] = this.playerBlocks();
      for (const c of [gdraw.COL_P1, gdraw.COL_P2, gdraw.COL_COIN]) this.burst(px + 0.5, py + 0.5, c, 26);
      ui.spawnConfetti(this.width, this.height);
      const info = {
        attempts: this.sessionAttempts, jumps: this.sessionJumps, time: this.sessionTime, coins: s.coins,
        coinCount: this.lv.coinCount, stars: 0, newCoins: 0, practice: this.practice, verified: false,
      };
      const newRecord = this.recordBest(100);
      if (this.curKind === "main" && !this.practice) {
        const lid = this.cur.id;
        const oldMask = this.coins[lid] || 0;
        const newMask = oldMask | s.coins;
        info.newCoins = coinsOf(newMask) - coinsOf(oldMask);
        if (newRecord) {
          info.stars = Math.trunc(Number(this.cur.stars) || 0);
          this.achEvent("gd_first");
          if (Math.trunc(Number(this.cur.difficulty) || 0) >= 5) this.achEvent("gd_demon");
        }
        if (newMask !== oldMask) {
          this.coins[lid] = newMask;
          this.dirty = true;
        }
        if (newMask === 7) this.achEvent("gd_coins");
        this.score = this.totalStars();
      } else if (this.curKind === "test" && this.test && this.test.verify && !this.practice && !this.startBlock) {
        info.verified = true;
        if (this.editor) this.editor.markVerified();
      }
      this.saveProgress();
      this.completeInfo = info;
      PG.audio.setMusicVolume(0.45);
    }

    burst(xb, yb, col, n) {
      for (let i = 0; i < n; i++) {
        const a = Math.random() * PG.TAU;
        const sp = 2 + Math.random() * 7;
        this.particles.push([xb, yb, Math.cos(a) * sp, Math.sin(a) * sp, 0.4 + Math.random() * 0.5, col, 0.08 + Math.random() * 0.14]);
      }
    }

    updateParticles(dt) {
      if (!this.particles.length) return;
      const keep = [];
      for (const p of this.particles) {
        p[4] -= dt;
        if (p[4] <= 0) continue;
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] -= 14 * dt;
        p[2] *= 1 - 1.5 * dt;
        keep.push(p);
      }
      this.particles = keep.slice(-400);
    }

    // ===================================================== Level-Auswahl
    layout() {
      const w = this.width, h = this.height, cx = w >> 1;
      this.renderer.resize(w, h, this.ts());
      // Höhen wachsen mit der Schrift
      const sh = this.fSmall.getHeight(), th = this.fTiny.getHeight();
      const tabH = Math.max(22, Math.min(30, Math.floor(h / 15)), sh + 10);
      const tabW = Math.min(Math.max(150, Math.floor(w / 6)), (w - 40) >> 1);
      const tabY = Math.trunc(h * 0.2);
      this.tabRects = [R(cx - tabW - 4, tabY, tabW, tabH), R(cx + 4, tabY, tabW, tabH)];
      this.tabBottom = tabY + tabH;
      const foot = th * 2 + 14;
      const bh = Math.max(24, Math.min(40, Math.floor(h / 14)), sh + 14);
      const gap = Math.max(6, Math.floor(h / 60));
      const optH = Math.max(22, Math.min(34, Math.floor(h / 16)), th + 12);
      const cardTop = this.tabBottom + gap + 2;
      const cardBottom = h - foot - optH - bh - 3 * gap - Math.max(8, Math.floor(h / 50));
      const arrow = Math.max(28, Math.min(52, Math.floor(w / 14)));
      const cardW = Math.min(w - 2 * arrow - 44, Math.max(620, Math.trunc(w * 0.6)));
      this.cardRect = R(cx - cardW / 2, cardTop, cardW, Math.max(90, cardBottom - cardTop));
      const cr = this.cardRect;
      this.arrowRects = [R(cr.x - arrow - 10, cr.centery - arrow / 2, arrow, arrow), R(cr.right + 10, cr.centery - arrow / 2, arrow, arrow)];
      this.dotsY = cr.bottom + Math.max(6, Math.floor(h / 70));
      let y = this.dotsY + Math.max(6, Math.floor(h / 70));
      const bw = Math.min(w - 40, Math.max(380, Math.trunc(w * 0.62)));
      const x0 = cx - (bw >> 1);
      const modeW = Math.trunc(bw * 0.29);
      this.modeRects = [R(x0, y, modeW, bh), R(x0 + modeW + gap, y, modeW, bh)];
      const sx = x0 + 2 * modeW + 3 * gap;
      this.startRect = R(sx, y, x0 + bw - sx, bh);
      y += bh + gap;
      const ow = Math.floor((bw - 2 * gap) / 3);
      this.optRects = {};
      ["music", "bar", "auto"].forEach((key, i) => (this.optRects[key] = R(x0 + i * (ow + gap), y, ow, optH)));
    }

    setTab(tab) {
      if (!TABS.includes(tab) || tab === this.setupTab) return;
      this.setupTab = tab;
      if (tab === "levels") {
        if (!this.lists) this.lists = new PG.gdEdit.LevelList(this);
        else this.lists.reload();
      }
      this.playSound("click");
    }

    handleSelect(ev) {
      if (ev.kind === "mousedown") {
        for (let i = 0; i < this.tabRects.length; i++) {
          if (this.tabRects[i].collidepoint(ev.pos)) return this.setTab(TABS[i]);
        }
      } else if (ev.kind === "keydown" && (ev.key === "Tab" || ev.key === "ISO_Left_Tab")) {
        return this.setTab(this.setupTab === "play" ? "levels" : "play");
      }
      if (this.setupTab === "levels") {
        if (ev.kind === "keydown" && ev.key === "Escape") return this.setTab("play");
        if (!this.lists) this.lists = new PG.gdEdit.LevelList(this);
        return this.lists.handle(ev);
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || this.isAction(k, "left")) this.stepLevel(-1);
        else if (k === "Right" || this.isAction(k, "right")) this.stepLevel(1);
        else if (/^[1-9]$/.test(k)) {
          const i = Number(k) - 1;
          if (i < this.levels.length) this.stepLevel(i - this.sel);
        } else if (k === "Return" || k === "space" || k === "KP_Enter") this.playSelected();
        else if (k === "p" || k === "P") {
          this.practice = !this.practice;
          this.playSound("select");
        } else if (k === "m" || k === "M") this.toggleOption("music");
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const p = ev.pos;
        if (this.arrowRects[0].collidepoint(p)) this.stepLevel(-1);
        else if (this.arrowRects[1].collidepoint(p)) this.stepLevel(1);
        else if (this.cardRect.collidepoint(p) || this.startRect.collidepoint(p)) this.playSelected();
        else if (this.modeRects[0].collidepoint(p)) {
          if (this.practice) {
            this.practice = false;
            this.playSound("select");
          }
        } else if (this.modeRects[1].collidepoint(p)) {
          if (!this.practice) {
            this.practice = true;
            this.playSound("select");
          }
        } else {
          for (const [key, rc] of Object.entries(this.optRects)) if (rc.collidepoint(p)) return this.toggleOption(key);
          this.dotRects().forEach((rc, i) => {
            if (rc.collidepoint(p)) this.stepLevel(i - this.sel);
          });
        }
      } else if (ev.kind === "wheel") this.stepLevel(ev.delta > 0 ? -1 : 1);
    }

    toggleOption(key) {
      if (key === "music") {
        this.musicOn = !this.musicOn;
        this.saveSetting("music", this.musicOn);
        if (!this.musicOn) PG.audio.stopMusic(this);
      } else if (key === "bar") {
        this.showBar = !this.showBar;
        this.saveSetting("progress_bar", this.showBar);
      } else {
        this.autoCp = !this.autoCp;
        this.saveSetting("auto_checkpoints", this.autoCp);
      }
      this.playSound("select");
    }

    stepLevel(d) {
      if (!d) return;
      this.sel = PG.mod(this.sel + d, this.levels.length);
      this.cardAnim = 1;
      this.cardDir = d > 0 ? 1 : -1;
      this.saveSetting("last_level", this.sel);
      this.playSound("move");
    }

    playSelected() {
      this.test = null;
      this.startLevel(this.levels[this.sel], "main");
    }

    dotRects() {
      const n = this.levels.length;
      const r = Math.max(4, Math.floor(this.height / 110));
      const gap = r * 4;
      const x0 = (this.width >> 1) - Math.floor(((n - 1) * gap) / 2);
      return Array.from({ length: n }, (_, i) => R(x0 + i * gap - r - 2, this.dotsY - r - 2, 2 * r + 4, 2 * r + 4));
    }

    // ===================================================== Ergebnis
    completeButtons() {
      const w = this.width, h = this.height;
      const pw = Math.min(w - 30, Math.max(460, Math.trunc(w * 0.56)));
      const sh = this.fSmall.getHeight();
      const bh = Math.max(26, Math.min(38, Math.floor(h / 15)), sh + 12);
      // Höhe aus dem Inhalt: Titel, Name, 3 Werte, Münzen, Extra-Zeile, Knöpfe
      const need = 12 + this.fHuge.getHeight() + 4 + sh + 8 + 3 * (sh + 2) + 6 + 2 * Math.max(8, Math.floor(h / 40)) + 8 + sh + 30 + bh + 14;
      const ph = Math.min(h - 30, Math.max(220, need));
      const panel = R((w - pw) / 2, (h - ph) / 2, pw, ph);
      let keys = ["again", "select"];
      if (this.curKind === "main" && this.sel < this.levels.length - 1) keys.push("next");
      if (this.curKind === "test") keys = ["again", "editor"];
      const gap = 8;
      const bw = Math.floor((pw - 30 - gap * (keys.length - 1)) / keys.length);
      const y = panel.bottom - bh - 14;
      return [panel, keys.map((k, i) => [k, R(panel.x + 15 + i * (bw + gap), y, bw, bh)])];
    }

    handleComplete(ev) {
      if (this.completeT < 0.35) return;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["Return", "space", "r", "R", "KP_Enter"].includes(k)) this.completeAction("again");
        else if (k === "Escape" || k === "BackSpace") this.completeAction(this.curKind === "test" ? "editor" : "select");
        else if (k === "n" || k === "N" || k === "Right") this.completeAction("next");
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (const [key, rc] of this.completeButtons()[1]) if (rc.collidepoint(ev.pos)) return this.completeAction(key);
      }
    }

    completeAction(key) {
      PG.audio.setMusicVolume(1);
      if (key === "again") this.startLevel(this.cur, this.curKind, this.startBlock);
      else if (key === "next") {
        if (this.curKind === "main" && this.sel < this.levels.length - 1) {
          this.sel++;
          this.saveSetting("last_level", this.sel);
          this.startLevel(this.levels[this.sel], "main");
        }
      } else if (key === "editor") this.backToEditor();
      else {
        if (this.curKind === "ugc") {
          this.setupTab = "levels";
          if (this.lists) this.lists.reload();
        }
        this.leaveToSelect();
      }
    }

    // ===================================================== Eigene Level
    get gridSnap() {
      return this.opts.grid !== false;
    }

    setGridSnap(on) {
      this.saveSetting("grid", !!on);
    }

    ugcNewLevel() {
      this.ugcEdit(PG.gdEdit.newLevel());
    }

    ugcEdit(m) {
      PG.audio.stopMusic(this);
      this.editor = new PG.gdEdit.LevelEditor(this, m);
      this.state = EDIT;
      this.playSound("click");
    }

    ugcCloseEditor() {
      this.editor = null;
      this.test = null;
      this.state = SELECT;
      this.setupTab = "levels";
      if (!this.lists) this.lists = new PG.gdEdit.LevelList(this);
      this.lists.reload();
      this.playSound("click");
    }

    ugcPlay(m) {
      this.test = null;
      this.startLevel(m, "ugc");
    }

    ugcTest(level, startBlock) {
      this.test = { verify: !startBlock };
      this.startLevel(level, "test", startBlock || null, false);
    }

    backToEditor() {
      PG.audio.stopMusic(this);
      PG.audio.setMusicVolume(1);
      this.test = null;
      this.run = null;
      if (!this.editor) return this.ugcCloseEditor();
      this.state = EDIT;
      this.playSound("click");
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === EDIT || this.state === SELECT) ui.drawBackground(ctx, this.width, this.height);
      if (this.state === EDIT && this.editor) return this.editor.draw(ctx);
      if (this.state === SELECT) return this.drawSelect(ctx);
      this.drawWorld(ctx);
      this.drawHud(ctx);
      if (this.state === COMPLETE) this.drawComplete(ctx);
    }

    drawWorld(ctx) {
      const run = this.run;
      const r = this.renderer;
      r.resize(this.width, this.height, this.ts());
      const ts = r.ts;
      const px = run.x / core.B, py = run.y / core.B;
      const camX = px - (this.width / ts) * 0.3;
      const camY = this.camY;
      const lv = this.lv;
      const pulse = this.bpm ? music.beatPulse(this.musicT, this.bpm) : 0;
      const ground = gdraw.colorAt(lv, px, 1);
      r.drawBackdrop(ctx, gdraw.colorAt(lv, px, 0), camX, camY, pulse);
      r.drawObjects(ctx, lv, camX, camY, this.t, pulse, run.used, false);
      r.drawGround(ctx, lv, ground, camX, camY, pulse);
      if (run.ceil > 0) r.drawCeiling(ctx, ground, run.ceil / core.B, camY, pulse);
      const n = this.curKind === "test" ? this.sessionAttempts : this.attempts[this.bestKey()] || 1;
      const ax = r.sx(this.attemptX, camX), ay = r.sy(this.attemptY, camY);
      if (ax > -this.width && ax < this.width + 50) {
        const label = t("gd.attempt", { n });
        ui.text(ctx, label, ax + 3, ay + 3, this.fAttempt, [0, 0, 0]);
        ui.text(ctx, label, ax, ay, this.fAttempt, [255, 255, 255]);
      }
      if (this.practice) {
        for (const cp of this.checkpoints.slice(-30)) {
          const cx = r.sx(cp.x / core.B + 0.5, camX), cy = r.sy(cp.y / core.B + 0.5, camY);
          if (cx > -ts && cx < this.width + ts) {
            const d = ts * 0.32;
            const pts = [[cx, cy - d], [cx + d * 0.7, cy], [cx, cy + d], [cx - d * 0.7, cy]];
            draw.polygon(ctx, gdraw.COL_CHECK, pts);
            draw.polygon(ctx, [10, 40, 20], pts, Math.max(1, Math.trunc(ts / 16)));
          }
        }
      }
      for (const p of this.particles) {
        const x = r.sx(p[0], camX), y = r.sy(p[1], camY);
        const size = Math.max(2, Math.trunc(ts * p[6] * Math.min(1, p[4] * 2)));
        draw.rect(ctx, p[5], [x - size / 2, y - size / 2, size, size]);
      }
      if (this.shock) {
        const age = this.t - this.shock[2];
        if (age < 0.45) {
          const f = age / 0.45;
          const rad = ts * (0.4 + 2.6 * f);
          draw.circle(ctx, [255, 255, 255, Math.trunc(220 * (1 - f))], [r.sx(this.shock[0], camX), r.sy(this.shock[1], camY)], rad, Math.max(2, ts * 0.18 * (1 - f)));
        }
      }
      if (!run.dead) r.drawPlayer(ctx, px, py, run.mode, run.grav, this.angle, camX, camY, this.t, this.trail);
    }

    drawHud(ctx) {
      const w = this.width, h = this.height, run = this.run;
      const pct = run ? core.progress(run, this.lv) : 0;
      if (this.showBar) {
        const bw = Math.trunc(w * 0.42), bh = Math.max(8, Math.floor(h / 64));
        const x = (w >> 1) - (bw >> 1), y = Math.max(6, Math.floor(h / 60));
        draw.rect(ctx, [0, 0, 0], [x - 2, y - 2, bw + 4, bh + 4], 0, bh);
        draw.rect(ctx, [40, 40, 60], [x, y, bw, bh], 0, bh);
        const fill = Math.trunc((bw * pct) / 100);
        if (fill > 0) draw.rect(ctx, gdraw.COL_P1, [x, y, fill, bh], 0, bh);
        ui.text(ctx, pct + "%", x + bw + 11, y + (bh >> 1) + 1, this.fSmall, [0, 0, 0], "midleft");
        ui.text(ctx, pct + "%", x + bw + 10, y + (bh >> 1), this.fSmall, [255, 255, 255], "midleft");
      }
      if (run && this.lv.coinCount) {
        const rr = Math.max(6, Math.floor(h / 50));
        for (let i = 0; i < this.lv.coinCount; i++) {
          const cx = w - 14 - rr - i * (2 * rr + 6);
          if (run.coins & (1 << (this.lv.coinCount - 1 - i))) gdraw.drawCoin(ctx, cx, 14 + rr, rr, 0);
          else draw.circle(ctx, [0, 0, 0], [cx, 14 + rr], rr, 2);
        }
      }
      const lines = [];
      if (this.practice) lines.push([t("gd.practice_hud"), gdraw.COL_CHECK]);
      if (this.curKind === "test") lines.push([t("gd.test_hud"), [255, 220, 120]]);
      let yb = h - 8;
      for (const [text, col] of lines.reverse()) {
        const tw = this.fTiny.width(text), th = this.fTiny.getHeight();
        draw.rect(ctx, [0, 0, 0, 140], [(w - tw) / 2 - 8, yb - th - 3, tw + 16, th + 6]);
        ui.text(ctx, text, w >> 1, yb, this.fTiny, col, "midbottom");
        yb -= th + 14;
      }
      if (this.newBest && this.state === PLAY) {
        const age = this.t - this.newBest[1];
        if (age < 1.6) {
          const sc = 1 + 0.25 * Math.max(0, 0.25 - age) * 4;
          const f = ui.font(Math.trunc(Math.max(18, Math.floor(h / 14)) * sc), true);
          const txt = t("gd.new_best", { pct: this.newBest[0] });
          ui.text(ctx, txt, (w >> 1) + 3, Math.trunc(h * 0.32) + 3, f, [0, 0, 0], "center");
          ui.text(ctx, txt, w >> 1, Math.trunc(h * 0.32), f, gdraw.COL_P1, "center");
        }
      }
    }

    drawComplete(ctx) {
      const w = this.width, h = this.height, info = this.completeInfo;
      const a = Math.min(1, this.completeT * 3);
      draw.rect(ctx, [0, 0, 0, Math.trunc(140 * a)], [0, 0, w, h]);
      const [panel0, rects] = this.completeButtons();
      const off = Math.trunc((1 - a) * 40);
      const panel = panel0.move(0, off);
      ui.drawPanel(ctx, panel, { accentTop: this.accent });
      const cx = panel.centerx;
      const key = info.practice ? "gd.complete_practice" : "gd.complete";
      const tf = this.fHuge.width(t(key)) > panel.w - 20 ? this.fMid : this.fHuge;
      let y = panel.y + 12;
      ui.text(ctx, t(key), cx, y, tf, this.accent, "midtop");
      y += tf.getHeight() + 4;
      ui.text(ctx, this.cur.name || "", cx, y, this.fSmall, ui.TEXT, "midtop");
      y += this.fSmall.getHeight() + 8;
      const mins = Math.floor(info.time / 60), secs = Math.floor(info.time) % 60;
      const rows = [[t("gd.stat_attempts"), String(info.attempts)], [t("gd.stat_jumps"), String(info.jumps)], [t("gd.stat_time"), mins + ":" + String(secs).padStart(2, "0")]];
      const lw = panel.w - 60;
      for (const [label, val] of rows) {
        ui.text(ctx, label, panel.x + 30, y, this.fSmall, ui.TEXT_DIM);
        ui.text(ctx, val, panel.x + 30 + lw, y, this.fSmall, ui.TEXT, "topright");
        y += this.fSmall.getHeight() + 2;
      }
      y += 6;
      const rr = Math.max(8, Math.floor(h / 40));
      if (info.coinCount) {
        const tw = info.coinCount * (2 * rr + 10);
        for (let i = 0; i < info.coinCount; i++) {
          const ccx = cx - (tw >> 1) + rr + i * (2 * rr + 10);
          if (info.coins & (1 << i)) gdraw.drawCoin(ctx, ccx, y + rr, rr, this.t);
          else draw.circle(ctx, ui.TEXT_FAINT, [ccx, y + rr], rr, 2);
        }
        y += 2 * rr + 8;
      }
      const extra = [];
      if (info.stars) extra.push(t("gd.stars_won", { n: info.stars }));
      if (info.newCoins) extra.push(t("gd.coins_won", { n: info.newCoins }));
      if (info.verified) extra.push(t("gd.verified_now"));
      if (info.practice) extra.push(t("gd.practice_note"));
      if (extra.length) {
        const txt = extra.join("  ·  ");
        const f = this.fSmall.width(txt) > panel.w - 20 ? this.fTiny : this.fSmall;
        ui.text(ctx, txt, cx, y, f, ui.GOLD, "midtop");
      }
      const labels = { again: t("gd.btn_again"), select: t("gd.btn_levels"), next: t("gd.btn_next"), editor: t("gd.btn_editor") };
      for (const [k, rc] of rects) this.btn(ctx, rc.move(0, off), labels[k], k === "again");
    }

    drawSelect(ctx) {
      const w = this.width, h = this.height, cx = w >> 1;
      ui.text(ctx, t("gd.title"), cx, Math.trunc(h * 0.085), this.fHuge, this.accent, "center");
      ui.text(ctx, t(this.setupTab === "play" ? "gd.subtitle" : "gd.ugc.subtitle"), cx, Math.trunc(h * 0.155), this.fSmall, ui.TEXT_DIM, "center");
      this.tabRects.forEach((rc, i) => this.btn(ctx, rc, t("gd.tab_" + TABS[i]), this.setupTab === TABS[i]));
      if (this.setupTab === "levels") {
        if (!this.lists) this.lists = new PG.gdEdit.LevelList(this);
        return this.lists.draw(ctx);
      }
      this.drawCard(ctx);
      this.arrowRects.forEach((rc, i) => {
        draw.rect(ctx, ui.BTN, rc, 0, rc.w >> 2);
        draw.rect(ctx, ui.BORDER, rc, 1, rc.w >> 2);
        const d = i === 0 ? -1 : 1, m = rc.w >> 2;
        draw.polygon(ctx, this.accent, [[rc.centerx - d * m * 0.6, rc.centery - m], [rc.centerx + d * m * 0.8, rc.centery], [rc.centerx - d * m * 0.6, rc.centery + m]]);
      });
      this.dotRects().forEach((rc, i) => {
        const on = i === this.sel;
        draw.circle(ctx, on ? this.accent : ui.BORDER_LIGHT, [rc.centerx, rc.centery], on ? rc.w / 2 - 2 : rc.w / 2 - 4);
      });
      this.btn(ctx, this.modeRects[0], t("gd.mode.normal"), !this.practice);
      this.btn(ctx, this.modeRects[1], t("gd.mode.practice"), this.practice);
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      const sf = this.fBtn.width(t("common.start")) > this.startRect.w - 10 ? this.fSmall : this.fBtn;
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, sf, ui.TEXT, "center");
      for (const [key, on] of [["music", this.musicOn], ["bar", this.showBar], ["auto", this.autoCp]]) {
        this.btn(ctx, this.optRects[key], t("gd.opt_" + key) + ": " + (on ? t("common.on") : t("common.off")), on, true);
      }
      const line = t("gd.total_stars", { n: this.totalStars(), max: this.maxStars() });
      const yl = h - 6 - this.fTiny.getHeight() - (this.fTiny.getHeight() >> 1) - 2;
      const r = Math.max(5, (this.fTiny.getHeight() >> 1) - 1);
      const tw = this.fTiny.width(line) + 2 * r + 6;
      gdraw.drawStar(ctx, cx - tw / 2 + r, yl, r, ui.GOLD);
      ui.text(ctx, line, cx - tw / 2 + 2 * r + 6, yl, this.fTiny, ui.GOLD, "midleft");
      const hintFont = this.fTiny.width(t("gd.select_hint")) > w - 16 ? ui.font(Math.max(10, Math.floor(h / 48))) : this.fTiny;
      ui.text(ctx, t("gd.select_hint"), cx, h - 4, hintFont, ui.TEXT_DIM, "midbottom");
    }

    cardParts(rect) {
      const pad = rect.x + 8 + Math.max(6, Math.floor(rect.w / 60)) + 12;
      const tinyH = this.fTiny.getHeight();
      const frBase = Math.max(14, Math.min(Math.floor(rect.h / 7), Math.floor(rect.w / 14), 44));
      const headH = Math.max(this.fMid.getHeight() + this.fSmall.getHeight() + tinyH + 10, 2 * frBase + tinyH + 6);
      const fr = Math.max(12, Math.min(Math.floor(rect.h / 7), Math.floor(rect.w / 14), 44, Math.floor((headH - tinyH - 6) / 2)));
      const barH = Math.max(10, Math.min(20, Math.floor(rect.h / 12)));
      const side = rect.w >= 420;
      const block = tinyH + 2 + barH;
      const barsTop = rect.bottom - 12 - (side ? block : 2 * block + 6);
      const bwAll = rect.right - 14 - pad;
      const bars = [0, 1].map((i) => {
        if (side) {
          const bw = Math.floor((bwAll - 16) / 2);
          return R(pad + i * (bw + 16), barsTop + tinyH + 2, bw, barH);
        }
        return R(pad, barsTop + i * (block + 6) + tinyH + 2, bwAll, barH);
      });
      const headTop = rect.y + 12;
      const thumbTop = headTop + headH + 8;
      return { pad, fr, headTop, headH, bars, thumb: R(pad, thumbTop, bwAll, barsTop - 8 - thumbTop), right: rect.right - 14 };
    }

    drawCard(ctx) {
      const d = this.levels[this.sel];
      const lid = d.id;
      const rect = this.cardRect.move(Math.trunc(this.cardAnim * 30 * this.cardDir), 0);
      const diff = Math.trunc(Number(d.difficulty) || 0);
      const col = gdraw.DIFF_COLORS[diff];
      ui.drawPanel(ctx, rect, { accentTop: col });
      const stripW = Math.max(6, Math.floor(rect.w / 60));
      draw.rect(ctx, d.bg || [40, 110, 255], [rect.x + 8, rect.y + 8, stripW, rect.h - 16], 0, stripW >> 1);
      const P = this.cardParts(rect);
      const { pad, fr } = P;
      const top = P.headTop;
      const faceX = pad + fr, faceY = top + fr;
      gdraw.drawFace(ctx, faceX, faceY, fr, diff);
      const diffText = t("gd.diff." + DIFF_KEYS[diff]);
      const df = this.fTiny.width(diffText) > 2 * fr + 30 ? ui.font(Math.max(9, Math.floor(this.height / 52))) : this.fTiny;
      ui.text(ctx, diffText, faceX, faceY + fr + 3, df, col, "midtop");
      const tx = pad + 2 * fr + Math.max(18, (df.width(diffText) >> 1) - fr + 10);
      const nameFont = this.fMid.width(d.name || "") > P.right - tx ? this.fSmall : this.fMid;
      ui.text(ctx, d.name || "", tx, top - 2, nameFont, ui.TEXT);
      const ny = top - 2 + nameFont.getHeight() + 3;
      const sr = Math.max(6, (this.fSmall.getHeight() >> 1) - 1);
      const done = (this.best[lid] || [0, 0])[0] >= 100;
      gdraw.drawStar(ctx, tx + sr, ny + sr, sr, done ? ui.GOLD : ui.TEXT_FAINT);
      const starTxt = String(Math.trunc(Number(d.stars) || 0));
      ui.text(ctx, starTxt, tx + 2 * sr + 5, ny + sr, this.fSmall, done ? ui.GOLD : ui.TEXT_DIM, "midleft");
      const mask = this.coins[lid] || 0;
      const cx0 = tx + 2 * sr + 5 + this.fSmall.width(starTxt) + 16;
      for (let i = 0; i < 3; i++) {
        const ccx = cx0 + sr + i * (2 * sr + 6);
        if (mask & (1 << i)) gdraw.drawCoin(ctx, ccx, ny + sr, sr, this.t + i);
        else draw.circle(ctx, ui.TEXT_FAINT, [ccx, ny + sr], sr, 2);
      }
      ui.text(ctx, t("gd.attempts_total", { n: this.attempts[lid] || 0 }), tx, ny + 2 * sr + 5, this.fTiny, ui.TEXT_FAINT);
      const th = P.thumb;
      if (th.h >= 26 && th.w >= 60) {
        ctx.drawImage(this.thumb(d, th.w, th.h), th.x, th.y, th.w, th.h);
        draw.rect(ctx, ui.BORDER, th, 1, 6);
      }
      const best = this.best[lid] || [0, 0];
      [["gd.mode.normal", gdraw.COL_P1], ["gd.mode.practice", gdraw.COL_CHECK]].forEach(([key, bcol], i) => {
        const bar = P.bars[i];
        ui.text(ctx, t("gd.best_label", { mode: t(key) }), bar.x, bar.y - 2, this.fTiny, ui.TEXT_DIM, "bottomleft");
        ui.text(ctx, best[i] + "%", bar.right, bar.y - 2, this.fTiny, ui.TEXT, "bottomright");
        draw.rect(ctx, ui.BTN, bar, 0, bar.h >> 1);
        const fill = Math.trunc((bar.w * best[i]) / 100);
        if (fill > 0) draw.rect(ctx, bcol, [bar.x, bar.y, Math.max(bar.h, fill), bar.h], 0, bar.h >> 1);
      });
    }

    thumb(d, w, h) {
      const key = d.id + "|" + w + "|" + h;
      let c = this.thumbs.get(key);
      if (c) return c;
      const lv = core.compile(d);
      gdraw.blockMasks(lv);
      const ren = new gdraw.Renderer();
      ren.resize(w, h, Math.max(8, Math.trunc(h / 7.5)));
      const camX = 12, camY = -1.2;
      c = gdraw.offscreen(w, h, (x) => {
        PG.ui.roundPath(x, 0, 0, w, h, 6);
        x.clip();
        ren.drawBackdrop(x, gdraw.colorAt(lv, camX, 0), camX, camY, 0);
        ren.drawObjects(x, lv, camX, camY, 0.2, 0, null, false);
        ren.drawGround(x, lv, gdraw.colorAt(lv, camX, 1), camX, camY, 0);
        ren.drawPlayer(x, camX + 3, 0, core.CUBE, 1, 0, camX, camY, 0);
      });
      this.thumbs.set(key, c);
      return c;
    }

    btn(ctx, rc, text, on, small) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      const col = on ? ui.TEXT : ui.TEXT_DIM;
      let f = small ? this.fTiny : this.fSmall;
      if (f.width(text) > rc.w - 12) f = this.fTiny;
      ui.text(ctx, fit(f, text, rc.w - 8), rc.centerx, rc.centery, f, col, "center");
    }
  }

  PG.register(GeometryDashGame, {
    id: "GeometryDashGame",
    key: "geodash",
    name: "Geometry Dash",
    modes: [["normal", "gd.mode.normal"], ["practice", "gd.mode.practice"]],
    settingsKey: "geodash",
    defaults: { music: true, auto_checkpoints: true, progress_bar: true, grid: true, last_level: 0 },
    wantsRightClick: true,
  });
})();
