/*
 * tunnelracer.js - Tunnel Racer (Port von games/tunnelracer.py)
 * ==============================================================
 * Neon-3D-Röhrenflug (Einzelspieler).
 *
 * - Zwei Modi (Vorspiel-Screen): ENDLOS (immer schneller, Highscore) und
 *   LEVEL (30 feste, seed-generierte Strecken mit Ziel; Fortschritt wird
 *   gespeichert und in der Levelauswahl abgehakt).
 * - Das Schiff fliegt durch eine kurvige Quadrat-Röhre; Hindernissen
 *   ausweichen (Balken, Blöcke, Ring-Blenden zum Durchfädeln), Münzen sammeln.
 * - Steuerung: Tasten (Standard, Pfeile/WASD in beide Achsen) oder Maus
 *   (Pointer-Lock, relative Bewegung) - im Setup umschaltbar, gespeichert.
 * - Motion Blur (0-80 %) einstellbar und gespeichert.
 *
 * 3D-Technik wie im Original (Kamera-Basis, Projektion, Near-Clip, Nebel) -
 * gezeichnet mit Canvas-Linien/-Polygonen. Der Motion-Blur arbeitet auf einer
 * Offscreen-Canvas (Szene + Vorbild), HUD/Overlays liegen darüber.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const NEAR = 0.12;
  const FOV_MUL = 1.04;
  const H = 3.0; // halbe Röhrenbreite
  const SHIP_AHEAD = 4.5; // Schiff fliegt so weit vor der Kamera
  const SHIP_W = 0.45, SHIP_H = 0.35;
  const VIEW_DIST = 120.0;
  const LEVELS = 30;

  // 3D-Welt-Palette (bewusst fest - unabhängig vom UI-Theme);
  // Akzent-/Schiffsfarbe kommt dynamisch aus this.accent (= Sidebar-Farbe).
  const COL_BG_TOP = [8, 6, 24];
  const COL_BG_BOT = [24, 10, 48];
  const COL_RING = [90, 240, 255];
  const COL_RING_HI = [255, 80, 220];
  const COL_RAIL = [150, 90, 255];
  const COL_OBST = [200, 40, 120];
  const COL_OBST_EDGE = [255, 120, 190];
  const COL_COIN = [255, 210, 80];

  const SETUP = "setup", READY = "ready", PLAY = "play", CRASH = "crash", FINISH = "finish";
  const STORE_KEY = "mem.tunnel"; // entspricht store.load_section("tunnel")

  /** Streckendaten für Level i (1-basiert). */
  function levelDef(i) {
    return {
      length: 900 + i * 140,
      base: Math.min(55.0, 24.0 + i * 1.1),
      gap: Math.max(11.0, 24.0 - i * 0.45),
      ring_p: Math.min(0.35, 0.08 + i * 0.01),
      amp: Math.min(2.4, 0.5 + i * 0.07),
    };
  }

  /** Nebel-Farbe als CSS-String (auf 1/64 quantisiert und gecacht). */
  const fogCache = new Map();
  function fogCss(col, depth) {
    const f = Math.min(1.0, Math.max(0.0, depth / VIEW_DIST));
    const q = Math.round(f * 64);
    const key = col[0] * 65536 + col[1] * 256 + col[2] + "|" + q;
    let s = fogCache.get(key);
    if (s === undefined) {
      const ff = q / 64;
      s =
        "rgb(" +
        Math.trunc(col[0] + (COL_BG_BOT[0] - col[0]) * ff) + "," +
        Math.trunc(col[1] + (COL_BG_BOT[1] - col[1]) * ff) + "," +
        Math.trunc(col[2] + (COL_BG_BOT[2] - col[2]) * ff) + ")";
      fogCache.set(key, s);
    }
    return s;
  }

  class TunnelRacerGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const tn = this.opts;
      this.control = tn.control === "mouse" ? "mouse" : "keys";
      const b = Number(tn.blur);
      this.blur = isFinite(b) ? Math.max(0.0, Math.min(0.8, b)) : 0.35;
      // Maus-Richtung: Standard normal (Maus rechts = rechts, hoch = hoch);
      // invertiert (Taste I) kehrt beide Achsen um (klassischer Flug-Stil).
      this.invert = !!tn.mouse_invert;
      const lv = parseInt(tn.last_level, 10);
      this.cursor = isFinite(lv) ? Math.max(1, Math.min(LEVELS, lv)) : 1;

      this.makeFonts();
      this._hasPrev = false; // Motion-Blur-Vorbild (Canvas bleibt erhalten)
      this._capture = false;
      this.loadSolved();
      this.level = 1;
      this.coins = 0;
      this.z = 0;
      this.flashT = 0;
      this.buildSetupLayout();
      this.state = SETUP;
    }

    get captureMouse() {
      return this._capture;
    }

    makeFonts() {
      const h = this.height;
      this._small = ui.font(Math.max(13, Math.floor(h / 30)));
      this._tiny = ui.font(Math.max(11, Math.floor(h / 36)));
      this._big = ui.font(Math.max(16, Math.floor(h / 21)), true);
      this._huge = ui.font(Math.max(26, Math.floor(h / 11)), true);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    loadSolved() {
      const data = PG.store.get(STORE_KEY, {});
      const lst = data && Array.isArray(data.solved) ? data.solved : [];
      const set = new Set();
      for (const v of lst) if (Number.isInteger(v) && v >= 1 && v <= LEVELS) set.add(v);
      this.solved = [...set].sort((a, c) => a - c);
    }

    markSolved(n) {
      if (!this.solved.includes(n)) {
        this.solved = this.solved.concat([n]).sort((a, c) => a - c);
        PG.store.set(STORE_KEY, { solved: this.solved });
      }
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const y0 = Math.floor(this.height * 0.24);
      const rx = cx + 24;
      this.ctrlRect = new PG.Rect(rx - 60, y0, 220, 40);
      this.blurMinus = new PG.Rect(rx - 60, y0 + 52, 44, 40);
      this.blurPlus = new PG.Rect(rx + 116, y0 + 52, 44, 40);
      this.blurBox = new PG.Rect(rx - 8, y0 + 52, 116, 40);
      if (this.mode === "levels") {
        const top = y0 + 108;
        const availH = this.height - top - 96;
        const cell = Math.max(20, Math.min(Math.floor((this.width - 80) / 10), Math.floor(availH / 3)));
        this.lvCell = cell;
        this.lvX = cx - cell * 5;
        this.lvY = top;
        this.lvFont = ui.font(Math.max(10, Math.floor((cell * 2) / 5)));
        this.startRect = new PG.Rect(cx - 95, top + 3 * cell + 14, 190, 46);
      } else {
        this.startRect = new PG.Rect(cx - 95, y0 + 124, 190, 46);
      }
    }

    levelAt(pos) {
      const c = Math.floor((pos[0] - this.lvX) / this.lvCell);
      const r = Math.floor((pos[1] - this.lvY) / this.lvCell);
      if (c >= 0 && c < 10 && r >= 0 && r < 3) return r * 10 + c + 1;
      return null;
    }

    cycleBlur() {
      this.blur = this.blur >= 0.79 ? 0.0 : Math.round((this.blur + 0.1) * 10) / 10;
      this.saveSetting("blur", this.blur);
    }

    handleSetup(ev) {
      const lv = this.mode === "levels";
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "c" || k === "C") this.toggleControl();
        else if (k === "i" || k === "I") this.toggleInvert();
        else if (k === "b" || k === "B") {
          this.cycleBlur();
          this.playSound("select");
        } else if (lv && (k === "Left" || k === "a" || k === "A")) {
          this.cursor = PG.mod(this.cursor - 2, LEVELS) + 1;
          this.playSound("move");
        } else if (lv && (k === "Right" || k === "d" || k === "D")) {
          this.cursor = PG.mod(this.cursor, LEVELS) + 1;
          this.playSound("move");
        } else if (lv && (k === "Up" || k === "w" || k === "W")) {
          this.cursor = PG.mod(this.cursor - 11, LEVELS) + 1;
          this.playSound("move");
        } else if (lv && (k === "Down" || k === "s" || k === "S")) {
          this.cursor = PG.mod(this.cursor + 9, LEVELS) + 1;
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.startRun(lv ? this.cursor : null);
        }
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        if (this.ctrlRect.collidepoint(p)) {
          this.toggleControl();
          return;
        }
        if (this.blurMinus.collidepoint(p)) {
          this.blur = Math.round(Math.max(0.0, this.blur - 0.1) * 10) / 10;
          this.saveSetting("blur", this.blur);
          this.playSound("select");
          return;
        }
        if (this.blurPlus.collidepoint(p)) {
          this.blur = Math.round(Math.min(0.8, this.blur + 0.1) * 10) / 10;
          this.saveSetting("blur", this.blur);
          this.playSound("select");
          return;
        }
        if (lv) {
          const n = this.levelAt(p);
          if (n !== null) {
            this.startRun(n);
            return;
          }
        }
        if (this.startRect.collidepoint(p)) this.startRun(lv ? this.cursor : null);
      }
    }

    toggleInvert() {
      this.invert = !this.invert;
      this.saveSetting("mouse_invert", this.invert);
      this.playSound("select");
    }

    toggleControl() {
      this.control = this.control === "keys" ? "mouse" : "keys";
      this.saveSetting("control", this.control);
      this.playSound("select");
    }

    // ===================================================== Lauf starten
    startRun(level) {
      this.level = level || 1;
      if (this.mode === "levels") {
        this.cursor = this.level;
        this.saveSetting("last_level", this.level);
        const d = levelDef(this.level);
        this.length = d.length;
        this.baseSpeed = d.base;
        this.gap = d.gap;
        this.ringP = d.ring_p;
        this.amp = d.amp;
        this.rng = new PG.Random(7700 + this.level);
      } else {
        this.length = null;
        this.baseSpeed = 26.0;
        this.gap = 26.0;
        this.ringP = 0.1;
        this.amp = 0.0; // rampt hoch
        this.rng = new PG.Random();
      }

      this.z = 0.0;
      this.px = 0.0;
      this.py = 0.0;
      this.vx = 0.0;
      this.vy = 0.0;
      this.camX = 0.0;
      this.camY = 0.0;
      this.coins = 0;
      this.runScore = 0;
      this.elapsed = 0.0;
      this.obstacles = []; // {z, cx, cy, hw, hh} im Querschnitt
      this.coinList = []; // {z, cx, cy, taken}
      this.nextSpawn = 40.0;
      this.readyT = 1.2;
      this.flashT = 0.0;
      this._hasPrev = false;
      this.keys = new Set();
      this.state = READY;
      this._capture = false;
      this.playSound("level");
    }

    // ===================================================== Strecke
    ampNow() {
      if (this.mode === "levels") return this.amp;
      return Math.min(2.2, (this.z / 1500.0) * 2.2);
    }

    center(z) {
      const a1 = this.ampNow();
      const a2 = 0.4 * a1;
      return [a1 * Math.sin(z * 0.021) + a2 * Math.sin(z * 0.043 + 1.7), 0.5 * a1 * Math.sin(z * 0.017 + 0.9)];
    }

    gapNow() {
      if (this.mode === "levels") return this.gap;
      return 26.0 - (26.0 - 11.0) * Math.min(1.0, this.z / 6000.0);
    }

    /** Hindernisse/Münzen bis z+160 erzeugen, Altes hinter uns löschen. */
    genAhead() {
      const rng = this.rng;
      const horizon = this.z + 160.0;
      while (this.nextSpawn < horizon) {
        if (this.length !== null && this.nextSpawn > this.length - 30) break;
        const zs = this.nextSpawn;
        const kindRoll = rng.random();
        const ringP = this.mode === "levels" ? this.ringP : Math.min(0.35, 0.08 + this.z / 20000.0);
        let freeCell = null;
        if (kindRoll < ringP) {
          // Ring mit Loch zum Durchfädeln
          const hx = rng.uniform(-H + 1.0, H - 1.0);
          const hy = rng.uniform(-H + 1.0, H - 1.0);
          const hole = 1.4;
          // vier Rechtecke um das Loch
          this.obstacles.push(
            { z: zs, cx: 0, cy: (hy - hole / 2 - H) / 2 + 0, hw: H, hh: (hy - hole / 2 + H) / 2 },
            { z: zs, cx: 0, cy: (hy + hole / 2 + H) / 2, hw: H, hh: (H - hy - hole / 2) / 2 },
            { z: zs, cx: (hx - hole / 2 - H) / 2, cy: hy, hw: (hx - hole / 2 + H) / 2, hh: hole / 2 },
            { z: zs, cx: (hx + hole / 2 + H) / 2, cy: hy, hw: (H - hx - hole / 2) / 2, hh: hole / 2 }
          );
          freeCell = [hx, hy];
        } else if (kindRoll < ringP + 0.3) {
          // horizontale Barriere oben oder unten
          const top = rng.random() < 0.5;
          const cy = top ? -H + 0.45 * H : H - 0.45 * H;
          this.obstacles.push({ z: zs, cx: 0.0, cy, hw: H, hh: 0.45 * H });
          freeCell = [0.0, top ? H * 0.5 : -H * 0.5];
        } else if (kindRoll < ringP + 0.55) {
          // vertikale Halbwand links oder rechts
          const left = rng.random() < 0.5;
          const cx = left ? -H / 2 : H / 2;
          this.obstacles.push({ z: zs, cx, cy: 0.0, hw: H / 2, hh: H });
          freeCell = [left ? H / 2 : -H / 2, 0.0];
        } else {
          // Block in einer von 9 Zellen
          const gx = rng.randint(-1, 1);
          const gy = rng.randint(-1, 1);
          this.obstacles.push({ z: zs, cx: gx * 2.0, cy: gy * 2.0, hw: 0.6, hh: 0.6 });
          freeCell = [(PG.mod(gx + 2, 3) - 1) * 2.0, gy * 2.0];
        }
        // Münzreihe zwischen den Hindernissen
        if (rng.random() < 0.6 && freeCell !== null) {
          const n = rng.randint(3, 5);
          for (let i = 0; i < n; i++) {
            this.coinList.push({ z: zs + 6 + i * 4.0, cx: freeCell[0], cy: freeCell[1], taken: false });
          }
        }
        this.nextSpawn += this.gapNow() * rng.uniform(0.8, 1.2);
      }
      this.obstacles = this.obstacles.filter((o) => o.z > this.z - 10);
      this.coinList = this.coinList.filter((c) => c.z > this.z - 10);
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === CRASH) {
        if (ev.kind === "keydown") {
          const k = ev.key;
          if (this.mode === "endless") {
            if (k === "Return" || k === "space") {
              this.gameOver = false;
              this.reset();
            }
          } else if (k === "r" || k === "R" || k === "Return" || k === "space") {
            this.startRun(this.level);
          } else if (k === "s" || k === "S") {
            this.state = SETUP;
            this._capture = false;
            this.buildSetupLayout();
          }
        }
        return;
      }
      if (this.state === FINISH) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.startRun(Math.min(LEVELS, this.level + 1));
          else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.buildSetupLayout();
          }
        }
        return;
      }
      const ACTS = ["up", "down", "left", "right"];
      const CAP = { up: "Up", down: "Down", left: "Left", right: "Right" };
      if (ev.kind === "keydown") {
        const k = ev.key;
        for (const act of ACTS) if (this.isAction(k, act) || k === CAP[act]) this.keys.add(act);
        if (k === "b" || k === "B") this.cycleBlur();
        if (k === "i" || k === "I") this.toggleInvert();
      } else if (ev.kind === "keyup") {
        const k = ev.key;
        for (const act of ACTS) if (this.isAction(k, act) || k === CAP[act]) this.keys.delete(act);
      } else if (ev.kind === "mouserel" && this.control === "mouse") {
        // normal: Maus rechts = Schiff rechts, Maus hoch = Schiff hoch.
        const hx = this.invert ? -1.0 : 1.0;
        const hy = this.invert ? 1.0 : -1.0;
        this.px += ev.rel[0] * 0.011 * H * hx;
        this.py += ev.rel[1] * 0.011 * H * hy;
      }
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === READY) {
        this.readyT -= dt;
        if (this.readyT <= 0) {
          this.state = PLAY;
          this._capture = this.control === "mouse";
        }
        return;
      }
      if (this.state !== PLAY || this.gameOver) return;
      this.elapsed += dt;

      // Tempo
      let v;
      if (this.mode === "endless") {
        v = Math.min(75.0, 26.0 + 10.0 * (this.z / 1000.0));
      } else {
        const progress = this.z / Math.max(1.0, this.length);
        v = this.baseSpeed * (1.0 + 0.15 * progress);
      }
      const zPrev = this.z;
      this.z += v * dt;

      // Steuerung
      if (this.control === "keys") {
        const ax = ((this.keys.has("right") ? 1 : 0) - (this.keys.has("left") ? 1 : 0)) * 14.0;
        const ay = ((this.keys.has("down") ? 1 : 0) - (this.keys.has("up") ? 1 : 0)) * 14.0;
        this.vx += ax * dt;
        this.vy += ay * dt;
        this.vx -= this.vx * Math.min(1.0, 6.0 * dt);
        this.vy -= this.vy * Math.min(1.0, 6.0 * dt);
        this.vx = Math.max(-7.0, Math.min(7.0, this.vx));
        this.vy = Math.max(-7.0, Math.min(7.0, this.vy));
        this.px += this.vx * dt;
        this.py += this.vy * dt;
      }
      this.px = Math.max(-(H - SHIP_W), Math.min(H - SHIP_W, this.px));
      this.py = Math.max(-(H - SHIP_H), Math.min(H - SHIP_H, this.py));

      // Kamera weich hinter dem Schiff
      const k = Math.min(1.0, 6.0 * dt);
      this.camX += (this.px * 0.55 - this.camX) * k;
      this.camY += (this.py * 0.55 - this.camY) * k;

      this.genAhead();

      // Kollisionen: über die gesamte z-Strecke des Frames prüfen, damit
      // bei hohem Tempo/niedriger FPS nichts übersprungen wird (Sweep).
      const zLo = zPrev + SHIP_AHEAD - 0.8;
      const zHi = this.z + SHIP_AHEAD + 0.8;
      for (const o of this.obstacles) {
        if (zLo < o.z && o.z < zHi) {
          if (Math.abs(this.px - o.cx) < SHIP_W + o.hw && Math.abs(this.py - o.cy) < SHIP_H + o.hh) {
            this.crash();
            return;
          }
        }
      }
      for (const c of this.coinList) {
        if (!c.taken && zLo < c.z && c.z < zHi) {
          if (Math.hypot(this.px - c.cx, this.py - c.cy) < 0.75) {
            c.taken = true;
            this.coins += 1;
            this.playSound("point");
          }
        }
      }

      // Punkte / Ziel
      if (this.mode === "endless") this.score = Math.trunc(this.z * 10 + this.coins * 50);
      else if (this.z >= this.length) this.finish();

      if (this.flashT > 0) this.flashT -= dt;
    }

    crash() {
      this.playSound("explode");
      this.rumble(220);
      this.flashT = 0.15;
      this._capture = false;
      this.state = CRASH;
      if (this.mode === "endless") this.gameOver = true; // Engine speichert den Score
    }

    finish() {
      this._capture = false;
      const par = (this.length / this.baseSpeed) * 1.25;
      this.timeBonus = Math.max(0, Math.trunc((par - this.elapsed) * 20));
      this.coinBonus = this.coins * 50;
      this.runScore = 1000 + this.coinBonus + this.timeBonus;
      this.score += this.runScore;
      this.markSolved(this.level);
      this.state = FINISH;
      this.playSound("win");
    }

    // ===================================================== 3D-Helfer
    tangent(z) {
      const c0 = this.center(z);
      const c1 = this.center(z + 8.0);
      return [Math.atan2(c1[0] - c0[0], 8.0), Math.atan2(c1[1] - c0[1], 8.0)];
    }

    basisAt(z) {
      const [yaw, pitch] = this.tangent(z);
      const cp = Math.cos(pitch);
      const f = [Math.sin(yaw) * cp, Math.sin(pitch), Math.cos(yaw) * cp];
      const rl = Math.hypot(f[0], f[2]) || 1.0;
      const r = [f[2] / rl, 0.0, -f[0] / rl];
      const u = [f[1] * r[2] - f[2] * r[1], f[2] * r[0] - f[0] * r[2], f[0] * r[1] - f[1] * r[0]];
      return [r, u, f];
    }

    /**
     * Punkt im Röhren-Querschnitt bei z (Offset ox/oy) direkt in Kamera-
     * koordinaten (world + to_cam in einem Schritt). Vereinfachung wie im
     * Original: Querschnitt bleibt achsenparallel (kein Roll).
     */
    camPt(z, ox, oy, cxy) {
      const c = cxy || this.center(z);
      const [r, u, f] = this._basis;
      const dx = c[0] + ox - this.cam[0];
      const dy = c[1] + oy - this.cam[1];
      const dz = z - this.cam[2];
      return [dx * r[0] + dy * r[1] + dz * r[2], dx * u[0] + dy * u[1] + dz * u[2], dx * f[0] + dy * f[1] + dz * f[2]];
    }

    proj(c) {
      const k = this._f / c[2];
      return [this._scx + c[0] * k, this._scy - c[1] * k];
    }

    static clipSeg(a, b) {
      const da = a[2] - NEAR, db = b[2] - NEAR;
      if (da < 0 && db < 0) return null;
      if (da < 0 || db < 0) {
        const tt = da / (da - db);
        const m = [a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR];
        return da < 0 ? [m, b] : [a, m];
      }
      return [a, b];
    }

    // ===================================================== Zeichnen
    sceneCanvas() {
      const ps = Math.max(1, Math.min(2, (PG.app && PG.app.pixelScale) || 1));
      const w = Math.round(this.width * ps), h = Math.round(this.height * ps);
      if (!this._scene || this._scene.width !== w || this._scene.height !== h) {
        this._scene = ui.makeCanvas(w, h);
        this._sceneCtx = this._scene.getContext("2d");
        this._prev = ui.makeCanvas(w, h);
        this._prevCtx = this._prev.getContext("2d");
        this._hasPrev = false;
        this._sky = null;
      }
      this._sceneCtx.setTransform(ps, 0, 0, ps, 0, 0);
      return this._sceneCtx;
    }

    draw(ctx) {
      this._scx = this.width / 2;
      this._scy = this.height / 2;
      this._f = this.height * FOV_MUL;

      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      const [cx, cy] = this.center(this.z);
      this.cam = [cx + this.camX, cy + this.camY, this.z];
      this._basis = this.basisAt(this.z);

      // Welt in die Szene-Canvas (für den Motion-Blur), sonst direkt
      const useBlur = this.blur > 0.01;
      const sctx = useBlur ? this.sceneCanvas() : ctx;
      this.drawSky(sctx);
      this.drawTunnel(sctx);
      this.drawObstacles(sctx);
      this.drawCoins(sctx);
      this.drawShip(sctx);
      if (useBlur) this.applyBlur(ctx);
      else this._hasPrev = false;

      if (this.flashT > 0) {
        ctx.fillStyle = ui.col([255, 255, 255], Math.trunc((200 * this.flashT) / 0.15) / 255);
        ctx.fillRect(0, 0, this.width, this.height);
      }
      this.drawHud(ctx);
      if (this.state === READY) {
        ui.text(ctx, t("tun.start_hint"), Math.floor(this.width / 2), Math.floor(this.height * 0.42), this._huge, this.accent, "center");
      } else if (this.state === CRASH) {
        this.drawCrash(ctx);
      } else if (this.state === FINISH) {
        this.drawFinish(ctx);
      }
    }

    drawSky(c) {
      if (!this._skyGrad || this._skyCtx !== c) {
        const g = c.createLinearGradient(0, 0, 0, this.height);
        g.addColorStop(0, ui.col(COL_BG_TOP));
        g.addColorStop(1, ui.col(COL_BG_BOT));
        this._skyGrad = g;
        this._skyCtx = c;
      }
      c.fillStyle = this._skyGrad;
      c.fillRect(0, 0, this.width, this.height);
    }

    ringPts(z) {
      const cxy = this.center(z);
      return [this.camPt(z, -H, -H, cxy), this.camPt(z, H, -H, cxy), this.camPt(z, H, H, cxy), this.camPt(z, -H, H, cxy)];
    }

    seg(c, a, b, col, width) {
      const s = TunnelRacerGame.clipSeg(a, b);
      if (!s) return;
      const depth = (s[0][2] + s[1][2]) / 2;
      const pa = this.proj(s[0]), pb = this.proj(s[1]);
      c.strokeStyle = fogCss(col, depth);
      c.lineWidth = width;
      c.beginPath();
      c.moveTo(pa[0], pa[1]);
      c.lineTo(pb[0], pb[1]);
      c.stroke();
    }

    drawTunnel(c) {
      const z0 = Math.floor(this.z / 6.0) * 6.0;
      const rings = [];
      const count = Math.trunc(VIEW_DIST / 6) + 1;
      for (let i = 0; i < count; i++) {
        const z = z0 + i * 6.0;
        rings.push([z, this.ringPts(z)]);
      }
      c.lineCap = "butt";
      // Ecken-Rails (verbinden aufeinanderfolgende Ringe)
      for (let i = 0; i + 1 < rings.length; i++) {
        const a = rings[i][1], b = rings[i + 1][1];
        for (let k = 0; k < 4; k++) this.seg(c, a[k], b[k], COL_RAIL, 1);
      }
      // Ringe
      for (const [z, pts] of rings) {
        const hi = PG.mod(Math.trunc(z / 6.0), 4) === 0;
        const col = hi ? COL_RING_HI : COL_RING;
        for (let k = 0; k < 4; k++) this.seg(c, pts[k], pts[(k + 1) % 4], col, hi ? 2 : 1);
      }
    }

    drawObstacles(c) {
      const items = [];
      for (const o of this.obstacles) {
        const rel = o.z - this.z;
        if (rel < NEAR || rel > VIEW_DIST) continue;
        const cxy = this.center(o.z);
        const corners = [
          this.camPt(o.z, o.cx - o.hw, o.cy - o.hh, cxy),
          this.camPt(o.z, o.cx + o.hw, o.cy - o.hh, cxy),
          this.camPt(o.z, o.cx + o.hw, o.cy + o.hh, cxy),
          this.camPt(o.z, o.cx - o.hw, o.cy + o.hh, cxy),
        ];
        if (corners.every((q) => q[2] <= NEAR)) continue;
        const depth = (corners[0][2] + corners[1][2] + corners[2][2] + corners[3][2]) / 4;
        items.push([depth, corners.map((q) => this.proj(q))]);
      }
      items.sort((a, b) => b[0] - a[0]);
      c.lineJoin = "round";
      for (const [depth, pts] of items) {
        c.beginPath();
        c.moveTo(pts[0][0], pts[0][1]);
        for (let i = 1; i < 4; i++) c.lineTo(pts[i][0], pts[i][1]);
        c.closePath();
        c.fillStyle = fogCss(COL_OBST, depth);
        c.fill();
        c.strokeStyle = fogCss(COL_OBST_EDGE, depth);
        c.lineWidth = 2;
        c.stroke();
      }
    }

    drawCoins(c) {
      const tick = ui.now();
      for (const coin of this.coinList) {
        if (coin.taken) continue;
        const rel = coin.z - this.z;
        if (rel < NEAR || rel > VIEW_DIST) continue;
        const cam = this.camPt(coin.z, coin.cx, coin.cy);
        if (cam[2] < NEAR) continue;
        const k = this._f / cam[2];
        const x = this._scx + cam[0] * k;
        const y = this._scy - cam[1] * k;
        const r = Math.max(2, Math.trunc(k * 0.35 * (1.0 + 0.15 * Math.sin(tick * 4))));
        c.beginPath();
        c.arc(x, y, r, 0, PG.TAU);
        c.fillStyle = fogCss(COL_COIN, cam[2]);
        c.fill();
        const lw = Math.max(1, Math.floor(r / 4));
        c.beginPath();
        c.arc(x, y, Math.max(0.1, r - lw / 2), 0, PG.TAU);
        c.strokeStyle = fogCss([255, 240, 180], cam[2]);
        c.lineWidth = lw;
        c.stroke();
      }
    }

    drawShip(c) {
      const z = this.z + SHIP_AHEAD;
      const cams = [
        this.camPt(z, this.px, this.py - SHIP_H), // Nase oben
        this.camPt(z - 0.9, this.px - SHIP_W, this.py + SHIP_H),
        this.camPt(z - 0.5, this.px, this.py + SHIP_H * 0.4),
        this.camPt(z - 0.9, this.px + SHIP_W, this.py + SHIP_H),
      ];
      if (cams.some((q) => q[2] <= NEAR)) return;
      const pts = cams.map((q) => this.proj(q));
      draw.polygon(c, this.accent, pts);
      draw.polygon(c, [240, 250, 255], pts, 2);
    }

    /** Motion Blur: Vorbild mit Alpha über die Szene, Ergebnis wird neues Vorbild. */
    applyBlur(ctx) {
      const sc = this._sceneCtx;
      if (this._hasPrev) {
        sc.save();
        sc.setTransform(1, 0, 0, 1, 0, 0);
        sc.globalAlpha = Math.trunc(this.blur * 230) / 255;
        sc.drawImage(this._prev, 0, 0);
        sc.restore();
      }
      const pc = this._prevCtx;
      pc.globalCompositeOperation = "copy";
      pc.drawImage(this._scene, 0, 0);
      pc.globalCompositeOperation = "source-over";
      this._hasPrev = true;
      ctx.drawImage(this._scene, 0, 0, this.width, this.height);
    }

    drawHud(ctx) {
      ui.text(ctx, t("common.points", { score: this.score }), 14, 10, this._big, this.accent);
      const right = [t("tun.coins", { n: this.coins })];
      if (this.mode === "endless") {
        const v = Math.min(75.0, 26.0 + 10.0 * (this.z / 1000.0));
        right.push(t("tun.distance", { n: Math.trunc(this.z) }));
        right.push(t("tun.speed", { v: Math.trunc(v * 3.6) }));
      } else {
        right.unshift(t("tun.level", { n: this.level }));
        const pct = Math.trunc((100 * this.z) / Math.max(1, this.length));
        right.push(Math.min(100, pct) + "%");
      }
      let y = 12;
      for (const line of right) {
        ui.text(ctx, line, this.width - 14, y, this._small, ui.TEXT_DIM, "topright");
        y += 22;
      }
      // Fortschrittsbalken (Level-Modus)
      if (this.mode === "levels" && this.state === PLAY) {
        const frac = Math.min(1.0, this.z / Math.max(1, this.length));
        draw.rect(ctx, ui.PANEL_LIGHT, [0, this.height - 6, this.width, 6]);
        draw.rect(ctx, this.accent, [0, this.height - 6, Math.trunc(this.width * frac), 6]);
      }
    }

    drawCrash(ctx) {
      ctx.fillStyle = ui.col([10, 6, 20], 175 / 255);
      ctx.fillRect(0, 0, this.width, this.height);
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      const head = t("tun.crash");
      const sc = t("common.points", { score: this.score });
      const hint = t(this.mode === "endless" ? "common.enter_restart" : "tun.retry");

      // Panel hinter dem Ergebnis (dynamische ui-Palette)
      const top = cy - 50 - Math.floor(this._huge.height / 2) - 22;
      const bottom = cy + 40 + Math.floor(this._small.height / 2) + 22;
      const pw = Math.min(this.width - 40, Math.max(380, this._huge.width(head) + 80, this._small.width(hint) + 60, this.font.width(sc) + 60));
      const panel = [cx - Math.floor(pw / 2), top, pw, bottom - top];
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, ui.BORDER_LIGHT, panel, 1, 14);

      ui.text(ctx, head, cx, cy - 50, this._huge, ui.RED, "center");
      ui.text(ctx, sc, cx, cy + 2, this.font, ui.TEXT, "center");
      ui.text(ctx, hint, cx, cy + 40, this._small, ui.TEXT_DIM, "center");
    }

    drawFinish(ctx) {
      ctx.fillStyle = ui.col([6, 12, 24], 185 / 255);
      ctx.fillRect(0, 0, this.width, this.height);
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      const head = t("tun.level_done", { n: this.level });
      const lines = [
        t("tun.base", { n: 1000 }),
        t("tun.coin_bonus", { n: this.coinBonus }),
        t("tun.time_bonus", { n: this.timeBonus }),
        t("common.points", { score: this.score }),
      ];
      const hint = t("tun.next");

      const top = cy - 70 - Math.floor(this._huge.height / 2) - 22;
      const bottom = cy - 20 + 30 * lines.length + 12 + Math.floor(this._small.height / 2) + 22;
      const pw = Math.min(
        this.width - 40,
        Math.max(400, this._huge.width(head) + 80, this._small.width(hint) + 60, Math.max(...lines.map((l) => this.font.width(l))) + 60)
      );
      const panel = [cx - Math.floor(pw / 2), top, pw, bottom - top];
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, ui.BORDER_LIGHT, panel, 1, 14);

      ui.text(ctx, head, cx, cy - 70, this._huge, this.accent, "center");
      let y = cy - 20;
      for (const line of lines) {
        ui.text(ctx, line, cx, y, this.font, ui.TEXT, "center");
        y += 30;
      }
      ui.text(ctx, hint, cx, y + 12, this._small, ui.TEXT_DIM, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, "TUNNEL RACER", cx, Math.floor(this.height * 0.1), this._huge, this.accent, "center");
      ui.text(ctx, t("tun.mode." + this.mode) + "   -   " + t("tun.subtitle"), cx, Math.floor(this.height * 0.17), this._small, ui.TEXT_DIM, "center");

      const cr = this.ctrlRect;
      ui.text(ctx, t("tun.control"), cr.x - 16, cr.centery, this._small, ui.TEXT_DIM, "midright");
      draw.rect(ctx, ui.BTN_SEL, cr, 0, 8);
      draw.rect(ctx, ui.BORDER, cr, 1, 8);
      ui.text(ctx, t("tun.control." + this.control) + "  [C]", cr.centerx, cr.centery, this._small, ui.TEXT, "center");

      ui.text(ctx, t("tun.blur"), this.blurMinus.x - 16, this.blurMinus.centery, this._small, ui.TEXT_DIM, "midright");
      for (const [r, sym] of [[this.blurMinus, "-"], [this.blurPlus, "+"]]) {
        draw.rect(ctx, ui.BTN, r, 0, 8);
        draw.rect(ctx, ui.BORDER, r, 1, 8);
        ui.text(ctx, sym, r.centerx, r.centery, this._big, ui.TEXT, "center");
      }
      const blurLbl = this.blur <= 0 ? t("common.off") : Math.trunc(Math.round(this.blur * 1000) / 10) + "%";
      ui.text(ctx, blurLbl, this.blurBox.centerx, this.blurBox.centery, this._big, this.accent, "center");

      // Maus-Richtung (nur bei Maussteuerung relevant; Taste I schaltet um)
      const mdir = this.invert ? t("common.dir_inverted") : t("common.dir_normal");
      ui.text(ctx, t("tun.mousedir", { dir: mdir }), cx, this.height - 18, this._small, ui.TEXT_DIM, "center");

      if (this.mode === "levels") {
        const doneFill = ui.mix(ui.BTN, this.accent, 0.25);
        const doneText = ui.mix(this.accent, ui.TEXT, 0.35);
        for (let n = 1; n <= LEVELS; n++) {
          const i = n - 1;
          const x = this.lvX + (i % 10) * this.lvCell;
          const y = this.lvY + Math.floor(i / 10) * this.lvCell;
          const cell = new PG.Rect(x + 1, y + 1, this.lvCell - 2, this.lvCell - 2);
          const done = this.solved.includes(n);
          draw.rect(ctx, done ? doneFill : ui.BTN, cell, 0, 4);
          if (n === this.cursor) draw.rect(ctx, this.accent, cell, 2, 4);
          ui.text(ctx, String(n), cell.centerx, cell.centery, this.lvFont, done ? doneText : ui.TEXT_DIM, "center");
        }
        ui.text(ctx, t("tun.progress", { n: this.solved.length, m: LEVELS }), cx, this.lvY + 3 * this.lvCell + 40, this._small, ui.TEXT_DIM, "center");
      }

      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
    }
  }

  PG.register(TunnelRacerGame, {
    id: "TunnelRacerGame",
    key: "tunnel",
    name: "Tunnel Racer",
    modes: [["endless", "tun.mode.endless"], ["levels", "tun.mode.levels"]],
    settingsKey: "tunnel",
    defaults: { control: "keys", blur: 0.35, last_level: 1, mouse_invert: false },
  });
})();
