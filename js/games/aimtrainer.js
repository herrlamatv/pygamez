/*
 * aimtrainer.js - Aim Trainer, chilliges 3D-Zielschießen (Port von games/aimtrainer.py)
 * =====================================================================================
 * Zielen (FPS-Look wie Minecraft/Fortnite): Das Fadenkreuz sitzt fest in der
 * Bildmitte, die Maus steuert die Kamera DIREKT - jede Bewegung dreht den
 * Blick sofort um den Bewegungsbetrag (Pointer-Lock: die App liefert relative
 * Bewegung als "mouserel"; Esc/Pause gibt die Maus frei). Yaw ist unbegrenzt
 * (360 Grad), Pitch klemmt bei +-60 Grad. Linksklick schießt exakt durch
 * die Bildmitte.
 *
 * Modi (Auswahl im Vorspiel-Screen):
 * - precision : 60 s, immer 3 statische Kugeln - Abschuss spawnt sofort neu;
 *               Genauigkeits-Bonus am Ende.
 * - reflex    : 30 Ziele einzeln, wachsen und schrumpfen (2.0 -> 1.2 s);
 *               Reaktionszeit-Statistik (Durchschnitt/Bestwert).
 * - moving    : 60 s, Ziele schweben auf Bahnen (Strafe/Orbit/Bob);
 *               Combo-Multiplikator bis x4, Fehlschuss resettet.
 * - chill     : endlos, ohne Timer und ohne Fehlschlag-Strafe; [E] beendet
 *               die Sitzung, sonst wird der Score beim Menü-Rückweg gespeichert.
 *
 * Themes (Setup, gespeichert): space (Sternenkugel + schwarzes Loch mit hellem
 * Ring im Gargantua-Stil + Planet), neon (Synthwave-Grid + Sonne) und range
 * (Schießstand-Halle). Nur Optik - das Gameplay ist identisch.
 *
 * Die 3D-Technik (Kamera-Basis, Projektion, Near-Clip, Painter-Sortierung,
 * Nebel, Billboards) ist reine Software auf dem 2D-Canvas, ohne Assets.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ----- Kamera / Projektion ---------------------------------------------------
  const CAM_POS = [0.0, 1.6, 0.0];
  const NEAR = 0.12;
  const FOV_MUL = 1.04; // _f = height * FOV_MUL  (~65 Grad horizontal)

  const DEG_PER_PX = 0.12; // Grad Drehung je Maus-Pixel bei sens = 1.0
  const PITCH_CLAMP = PG.radians(60.0);

  const COOLDOWN = 0.18; // s zwischen zwei Schüssen
  const FORGIVE_REL = 1.15; // Treffer-Vergebung (relativ + absolut)
  const FORGIVE_ABS = 0.004;

  const SETUP = "setup", PLAY = "play";
  const TAU = Math.PI * 2;

  // ----- Modi ------------------------------------------------------------------
  const MODE_CFG = {
    precision: { duration: 60.0, targets: null, simul: 3, radius: 0.45, paths: false, speed: 1.0, yaw: 70, pmin: -10, pmax: 35, dmin: 8, dmax: 20, respawn: [0.0, 0.0], relative: false },
    reflex: { duration: null, targets: 30, simul: 1, radius: 0.5, paths: false, speed: 1.0, yaw: 60, pmin: -15, pmax: 30, dmin: 6, dmax: 14, respawn: [0.4, 0.9], relative: true },
    moving: { duration: 60.0, targets: null, simul: 2, radius: 0.5, paths: true, speed: 1.0, yaw: 70, pmin: -5, pmax: 30, dmin: 7, dmax: 16, respawn: [0.3, 0.3], relative: false },
    chill: { duration: null, targets: null, simul: 3, radius: 0.55, paths: true, speed: 0.6, yaw: 90, pmin: -10, pmax: 35, dmin: 6, dmax: 18, respawn: [0.3, 0.3], relative: false },
  };

  // ----- Themes ----------------------------------------------------------------
  const THEMES = {
    space: {
      sky: [[4, 5, 12], [10, 8, 24]],
      fog: [10, 10, 22], fog_start: 26.0, fog_end: 60.0,
      body: [120, 200, 255], ring: [225, 240, 255], dot: [255, 255, 255],
      glow: [90, 160, 255], accent: [240, 245, 255],
    },
    neon: {
      sky: [[18, 8, 31], [58, 22, 80]],
      fog: [58, 22, 80], fog_start: 10.0, fog_end: 30.0,
      body: [0, 240, 200], ring: [255, 255, 255], dot: [20, 20, 40],
      glow: [0, 255, 220], accent: [255, 230, 120],
    },
    range: {
      sky: [[30, 32, 36], [30, 32, 36]],
      fog: [30, 32, 36], fog_start: 14.0, fog_end: 40.0,
      body: [230, 70, 70], ring: [240, 240, 240], dot: [230, 70, 70],
      glow: null, accent: [250, 250, 250],
    },
  };
  const THEME_KEYS = ["space", "neon", "range"];

  /** Kleiner Überschwinger am Ende (wie snake.py) - für den Target-Spawn. */
  function easeOutBack(x) {
    const c1 = 1.70158, c3 = 2.70158;
    x = Math.max(0.0, Math.min(1.0, x));
    return 1 + c3 * Math.pow(x - 1, 3) + c1 * Math.pow(x - 1, 2);
  }

  function dirFrom(yaw, pitch) {
    const cp = Math.cos(pitch);
    return [Math.sin(yaw) * cp, Math.sin(pitch), Math.cos(yaw) * cp];
  }

  /** Python-str(float): 1.0 -> "1.0", 1.25 -> "1.25" */
  function pyFloat(v) {
    return Number.isInteger(v) ? v.toFixed(1) : String(v);
  }

  /** Python-round(x, 1) für die Regler-Werte */
  function round1(v) {
    return Math.round(v * 10) / 10;
  }

  const rnd = PG.rand;

  class AimTrainerGame extends PG.Game {
    // Pointer-Lock nur während eines laufenden Durchgangs
    get captureMouse() {
      return !!this._capture;
    }

    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.cfg = MODE_CFG[this.mode] || MODE_CFG.precision;
      const [theme, sens, blur] = this.aimSettings();
      this.themeKey = theme;
      this.sens = sens;
      this.blur = blur; // Motion-Blur-Stärke (0.0 = aus .. 0.8)
      this._prevFrame = null; // letztes Bild für die Blur-Mischung

      this.makeFonts();

      this.yaw = 0.0;
      this.pitch = 0.0;
      this._kick = 0.0;
      this.animT = 0.0;
      this._goLast = null;
      this._capture = false; // App: Pointer-Lock + mouserel

      // Render-Caches (auflösungs-/themenabhängig)
      this._skyCache = null;
      this._bhCache = null;
      this._sunCache = null;
      this._stars = AimTrainerGame.makeStars(220);

      this._toast = null;
      this._toastT = 0.0;
      this.buildSetupLayout();
      this.state = SETUP;
    }

    makeFonts() {
      /* Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Schrift). */
      const h = this.height;
      this._small = ui.font(Math.max(13, Math.floor(h / 30)));
      this._tiny = ui.font(Math.max(11, Math.floor(h / 36)));
      this._big = ui.font(Math.max(16, Math.floor(h / 21)), true);
      this._huge = ui.font(Math.max(26, Math.floor(h / 11)), true);
    }

    /** Vollbild-Abdunklung */
    dim(ctx, rgb, alpha) {
      ctx.fillStyle = ui.col(rgb, alpha / 255);
      ctx.fillRect(0, 0, this.width, this.height);
    }

    aimSettings() {
      const aim = this.opts || {};
      let theme = aim.theme;
      if (!THEMES[theme]) theme = "space";
      let sens = Number(aim.sens);
      sens = isFinite(sens) ? Math.max(0.5, Math.min(2.0, sens)) : 1.0;
      let blur = Number(aim.blur);
      blur = isFinite(blur) ? Math.max(0.0, Math.min(0.8, blur)) : 0.0;
      return [theme, sens, blur];
    }

    saveAim(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    theme() {
      return THEMES[this.themeKey];
    }

    static makeStars(n) {
      const r = new PG.Random(4021);
      const stars = [];
      for (let i = 0; i < n; i++) {
        const z = r.uniform(-1.0, 1.0);
        const az = r.uniform(0, TAU);
        const xy = Math.sqrt(Math.max(0.0, 1.0 - z * z));
        stars.push([[Math.cos(az) * xy, z, Math.sin(az) * xy], r.choice([1, 1, 2]), r.uniform(0, TAU)]);
      }
      return stars;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(120, Math.floor((this.width - 80) / 3) - 10);
      const total = 3 * bw + 2 * 12;
      const y0 = Math.floor(this.height * 0.28);
      this.themeRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(total / 2) + i * (bw + 12), y0, bw, 44));
      // Regler-Reihen: Beschriftung steht LINKS daneben (spart Höhe).
      const rx = cx + 24;
      const sy = y0 + 58;
      this.sensMinus = new PG.Rect(rx - 60, sy, 44, 40);
      this.sensPlus = new PG.Rect(rx + 116, sy, 44, 40);
      this.sensBox = new PG.Rect(rx - 8, sy, 116, 40);
      const by = sy + 52;
      this.blurMinus = new PG.Rect(rx - 60, by, 44, 40);
      this.blurPlus = new PG.Rect(rx + 116, by, 44, 40);
      this.blurBox = new PG.Rect(rx - 8, by, 116, 40);
      this.startRect = new PG.Rect(cx - 95, by + 54, 190, 46);
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") this.setTheme(Number(k) - 1);
        else if (k === "Left" || k === "a" || k === "A") this.setTheme(PG.mod(THEME_KEYS.indexOf(this.themeKey) - 1, 3));
        else if (k === "Right" || k === "d" || k === "D") this.setTheme(PG.mod(THEME_KEYS.indexOf(this.themeKey) + 1, 3));
        else if (k === "plus" || k === "equal" || k === "KP_Add") this.changeSens(+0.1);
        else if (k === "minus" || k === "KP_Subtract") this.changeSens(-0.1);
        else if (k === "b" || k === "B") this.cycleBlur();
        else if (k === "Return" || k === "space") this.startRun();
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.themeRects.length; i++) {
          if (this.themeRects[i].collidepoint(ev.pos)) {
            this.setTheme(i);
            return;
          }
        }
        if (this.sensMinus.collidepoint(ev.pos)) this.changeSens(-0.1);
        else if (this.sensPlus.collidepoint(ev.pos)) this.changeSens(+0.1);
        else if (this.blurMinus.collidepoint(ev.pos)) this.changeBlur(-0.1);
        else if (this.blurPlus.collidepoint(ev.pos)) this.changeBlur(+0.1);
        else if (this.startRect.collidepoint(ev.pos)) this.startRun();
      }
    }

    setTheme(idx) {
      this.themeKey = THEME_KEYS[PG.mod(idx, 3)];
      this.saveAim("theme", this.themeKey);
      this._skyCache = null;
      this.playSound("click");
    }

    changeSens(delta) {
      this.sens = round1(Math.max(0.5, Math.min(2.0, this.sens + delta)));
      this.saveAim("sens", this.sens);
      this._toast = t("aim.sens_toast", { v: this.sens.toFixed(1) });
      this._toastT = 1.0;
      this.playSound("select");
    }

    changeBlur(delta) {
      this.blur = round1(Math.max(0.0, Math.min(0.8, this.blur + delta)));
      this.saveAim("blur", this.blur);
      if (this.blur <= 0) this._prevFrame = null;
      this.playSound("select");
    }

    cycleBlur() {
      /* [B] im Setup: Blur in 0.1er-Schritten durchschalten (0.8 -> aus). */
      this.blur = this.blur >= 0.79 ? 0.0 : round1(this.blur + 0.1);
      this.saveAim("blur", this.blur);
      if (this.blur <= 0) this._prevFrame = null;
      this.playSound("select");
    }

    blurLabel() {
      return this.blur <= 0 ? t("common.off") : Math.round(this.blur * 100) + "%";
    }

    applyBlur(ctx) {
      /* Motion Blur: das vorige Bild mit Blur-abhängiger Deckkraft über das
         neue mischen (exponentieller Trail). Läuft VOR Fadenkreuz/HUD, damit
         die scharf bleiben. */
      if (this.blur <= 0.01) {
        this._prevFrame = null;
        return;
      }
      const src = ctx.canvas;
      if (!src) return;
      const prev = this._prevFrame;
      if (!prev || prev.width !== src.width || prev.height !== src.height) {
        this._prevFrame = ui.makeCanvas(src.width, src.height);
        this.copyFrame(ctx);
        return;
      }
      ctx.save();
      ctx.globalAlpha = Math.floor(this.blur * 230) / 255;
      ctx.drawImage(prev, 0, 0, this.width, this.height);
      ctx.restore();
      this.copyFrame(ctx);
    }

    copyFrame(ctx) {
      const g = this._prevFrame.getContext("2d");
      g.save();
      g.globalCompositeOperation = "copy";
      g.drawImage(ctx.canvas, 0, 0);
      g.restore();
    }

    // ===================================================== Lauf starten/beenden
    startRun() {
      this.score = 0;
      this.gameOver = false;
      this.playT = 0.0;
      this.timeLeft = this.cfg.duration;
      this.targets = [];
      this._pending = []; // Respawn-Timer
      this.spawned = 0; // (reflex) bisher erschienene Ziele
      this.hits = 0;
      this.shots = 0;
      this.misses = 0;
      this.combo = 0;
      this.maxCombo = 0;
      this.lastHitT = 0.0;
      this.reactions = [];
      this.lastReaction = null;
      this.lastReactionT = -9.0;
      this.cooldown = 0.0;
      this._flash = 0.0;
      this._tracer = 0.0;
      this._hitmark = 0.0;
      this._toast = null;
      this._toastT = 0.0;
      this.particles = []; // [pos, vel, age, life, col]
      this.popups = []; // [text, worldPos, age]
      this.accBonus = 0;
      this._goLast = null;
      for (let i = 0; i < this.cfg.simul; i++) this.spawnTarget();
      this.state = PLAY;
      this._capture = true;
      this.playSound("level");
    }

    finishRun() {
      if (this.mode === "precision" && this.shots > 0) {
        const bonus = Math.floor(Math.pow(this.hits / this.shots, 2) * 1500);
        this.accBonus = bonus;
        this.score += bonus;
        if (bonus) this.playSound("point");
      } else {
        this.accBonus = 0;
      }
      // 100% Genauigkeit bei einer echten Sitzung (min. 20 Schüsse).
      if (this.shots >= 20 && this.hits === this.shots) this.achEvent("aim_perfect");
      this.gameOver = true;
      this._capture = false;
      this.playSound("win");
    }

    // ===================================================== Targets
    spawnTarget() {
      const cfg = this.cfg;
      if (cfg.targets !== null && this.spawned >= cfg.targets) return;
      const baseYaw = cfg.relative ? this.yaw : 0.0;
      let pos = null;
      let yw = 0;
      for (let n = 0; n < 20; n++) {
        yw = baseYaw + PG.radians(rnd.uniform(-cfg.yaw, cfg.yaw));
        const pt = PG.radians(rnd.uniform(cfg.pmin, cfg.pmax));
        const dist = rnd.uniform(cfg.dmin, cfg.dmax);
        const d = dirFrom(yw, pt);
        pos = [CAM_POS[0] + d[0] * dist, CAM_POS[1] + d[1] * dist, CAM_POS[2] + d[2] * dist];
        if (this.targets.every((o) => this.angleBetween(pos, o) > PG.radians(15))) break;
      }
      let path = null;
      if (cfg.paths) {
        const sp = cfg.speed;
        const r = rnd.random();
        const kind = r < 0.45 ? "strafe" : r < 0.75 ? "orbit" : "bob";
        if (kind === "strafe") {
          const amp = rnd.uniform(2.5, 5.0);
          path = { kind, amp, om: (rnd.uniform(1.5, 3.5) / amp) * sp, ax: [Math.cos(yw), 0.0, -Math.sin(yw)] };
        } else if (kind === "orbit") {
          path = { kind, rad: rnd.uniform(1.5, 3.0), om: rnd.uniform(0.5, 1.2) * sp };
        } else {
          path = { kind, amp: rnd.uniform(0.8, 1.6), om: rnd.uniform(1.0, 2.0) * sp, drift: rnd.uniform(-0.4, 0.4) * sp };
        }
      }
      let life = null;
      if (this.mode === "reflex") {
        const i = Math.min(this.spawned, 29);
        life = 2.0 - (0.8 * i) / 29.0;
      }
      this.targets.push({ base: pos, radius: cfg.radius, state: "grow", age: 0.0, life, path, phase: rnd.uniform(0, TAU), spawnT: this.playT });
      this.spawned += 1;
    }

    angleBetween(posA, targetB) {
      const pb = this.targetPos(targetB);
      const va = AimTrainerGame.normDir(posA);
      const vb = AimTrainerGame.normDir(pb);
      const d = Math.max(-1.0, Math.min(1.0, va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]));
      return Math.acos(d);
    }

    static normDir(p) {
      const d = [p[0] - CAM_POS[0], p[1] - CAM_POS[1], p[2] - CAM_POS[2]];
      const n = Math.sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) || 1.0;
      return [d[0] / n, d[1] / n, d[2] / n];
    }

    targetPos(tg) {
      const b = tg.base;
      const p = tg.path;
      if (!p) return b;
      const tt = this.playT + tg.phase;
      if (p.kind === "strafe") {
        const off = p.amp * Math.sin(p.om * tt);
        const ax = p.ax;
        return [b[0] + ax[0] * off, b[1], b[2] + ax[2] * off];
      }
      if (p.kind === "orbit") {
        return [b[0] + p.rad * Math.cos(p.om * tt), b[1], b[2] + p.rad * Math.sin(p.om * tt)];
      }
      // bob: vertikale Sinuswelle + begrenzte horizontale Drift
      const dx = Math.max(-2.5, Math.min(2.5, p.drift * (PG.mod(tt, 12.0) - 6.0)));
      return [b[0] + dx, b[1] + p.amp * Math.sin(p.om * tt), b[2]];
    }

    targetScale(tg) {
      if (tg.state === "grow") return easeOutBack(tg.age / 0.25);
      if (tg.state === "shrink") return Math.max(0.0, 1.0 - tg.age / 0.18);
      if (tg.life !== null) {
        // reflex: Ende-Schrumpfen
        const frac = tg.age / tg.life;
        if (frac > 0.8) return Math.max(0.3, 1.0 - ((frac - 0.8) / 0.2) * 0.7);
      }
      return 1.0;
    }

    updateTargets(dt) {
      const cfg = this.cfg;
      const alive = [];
      for (const tg of this.targets) {
        tg.age += dt;
        if (tg.state === "grow" && tg.age >= 0.25) {
          tg.state = "alive";
          tg.age = 0.25;
        }
        if (tg.state === "shrink") {
          if (tg.age >= 0.18) continue;
        } else if (tg.life !== null && tg.age >= tg.life) {
          // Reflex: abgelaufen = Fehlschuss
          this.misses += 1;
          this.playSound("move");
          this._pending.push(rnd.uniform(cfg.respawn[0], cfg.respawn[1]));
          continue;
        }
        alive.push(tg);
      }
      this.targets = alive;

      const rest = [];
      for (let timer of this._pending) {
        timer -= dt;
        if (timer <= 0) this.spawnTarget();
        else rest.push(timer);
      }
      this._pending = rest;

      // Lauf-Ende (reflex): alle Ziele verbraucht und keins mehr aktiv
      if (this.mode === "reflex" && this.spawned >= cfg.targets && !this.targets.length && !this._pending.length && !this.gameOver) {
        this.finishRun();
      }
    }

    // ===================================================== Kamera
    applyLook(rel) {
      /* Direkte 1:1-Maussteuerung (FPS-Look): Delta-Pixel -> Drehung. */
      const k = PG.radians(DEG_PER_PX) * this.sens;
      this.yaw = PG.mod(this.yaw + rel[0] * k, TAU);
      this.pitch = Math.max(-PITCH_CLAMP, Math.min(PITCH_CLAMP, this.pitch - rel[1] * k));
    }

    forward(extraPitch = 0.0) {
      return dirFrom(this.yaw, this.pitch + extraPitch);
    }

    viewBasis() {
      const f = this.forward(PG.radians(0.6) * this._kick);
      const rl = Math.hypot(f[0], f[2]) || 1.0;
      const r = [f[2] / rl, 0.0, -f[0] / rl]; // echter Rechts-Vektor
      const u = [
        // up = f x r (zeigt nach oben)
        f[1] * r[2] - f[2] * r[1],
        f[2] * r[0] - f[0] * r[2],
        f[0] * r[1] - f[1] * r[0],
      ];
      return [r, u, f];
    }

    toCam(p) {
      const [r, u, f] = this._basis;
      const dx = p[0] - CAM_POS[0];
      const dy = p[1] - CAM_POS[1];
      const dz = p[2] - CAM_POS[2];
      return [dx * r[0] + dy * r[1] + dz * r[2], dx * u[0] + dy * u[1] + dz * u[2], dx * f[0] + dy * f[1] + dz * f[2]];
    }

    proj(c) {
      const k = this._f / c[2];
      return [this._scx + c[0] * k, this._scy - c[1] * k];
    }

    static clipNear(pts) {
      const out = [];
      const n = pts.length;
      for (let i = 0; i < n; i++) {
        const a = pts[i], b = pts[(i + 1) % n];
        const da = a[2] - NEAR, db = b[2] - NEAR;
        if (da >= 0) out.push(a);
        if (da >= 0 !== db >= 0) {
          const tt = da / (da - db);
          out.push([a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR]);
        }
      }
      return out;
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

    fogColor(col, depth) {
      const th = this.theme();
      if (depth <= th.fog_start) return col;
      const f = Math.min(1.0, (depth - th.fog_start) / (th.fog_end - th.fog_start));
      const fog = th.fog;
      return [Math.trunc(col[0] + (fog[0] - col[0]) * f), Math.trunc(col[1] + (fog[1] - col[1]) * f), Math.trunc(col[2] + (fog[2] - col[2]) * f)];
    }

    bill(p) {
      /* Billboard-Projektion: [sx, sy, k, z] oder null hinter der Kamera. */
      const c = this.toCam(p);
      if (c[2] < NEAR) return null;
      const k = this._f / c[2];
      return [this._scx + c[0] * k, this._scy - c[1] * k, k, c[2]];
    }

    // ===================================================== Schießen
    shoot() {
      if (this.cooldown > 0 || this.gameOver) return;
      this.cooldown = COOLDOWN;
      this.shots += 1;
      this._flash = 0.06;
      this._tracer = 0.08;
      this._kick = 1.0;
      this.playSound("shoot");

      const f = this.forward(); // echter Ray ohne Recoil-Kick
      let best = null;
      for (const tg of this.targets) {
        if (tg.state === "shrink") continue;
        if (tg.state === "grow" && this.targetScale(tg) < 0.5) continue;
        const pos = this.targetPos(tg);
        const d = AimTrainerGame.normDir(pos);
        const dist = Math.hypot(pos[0] - CAM_POS[0], pos[1] - CAM_POS[1], pos[2] - CAM_POS[2]);
        const ang = Math.acos(Math.max(-1.0, Math.min(1.0, f[0] * d[0] + f[1] * d[1] + f[2] * d[2])));
        const ar = Math.atan((tg.radius * this.targetScale(tg)) / dist);
        if (ang <= ar * FORGIVE_REL + FORGIVE_ABS) {
          if (best === null || ang < best[0]) best = [ang, tg, pos, dist];
        }
      }
      if (best !== null) this.registerHit(best[1], best[2], best[3]);
      else this.registerMiss();
    }

    registerHit(tg, pos, dist) {
      this.hits += 1;
      this.combo += 1;
      this.maxCombo = Math.max(this.maxCombo, this.combo);
      this.lastHitT = this.playT;
      this._hitmark = 0.1;
      const mult = 1.0 + 0.25 * Math.min(this.combo, 12);
      let pts;
      if (this.mode === "precision") {
        pts = 100 + Math.trunc((dist - 8) * 6);
      } else if (this.mode === "reflex") {
        const reaction = Math.trunc((this.playT - tg.spawnT) * 1000);
        this.reactions.push(reaction);
        this.lastReaction = reaction;
        this.lastReactionT = this.playT;
        pts = Math.max(50, 1000 - reaction);
      } else if (this.mode === "moving") {
        pts = Math.trunc(80 * mult);
        if (this.combo === 5 || this.combo === 10) this.playSound("point");
        else if (this.combo === 12) this.playSound("level");
      } else {
        pts = Math.trunc(50 * mult);
      }
      this.score += pts;
      this.playSound("hit");
      this.rumble(60);

      tg.state = "shrink";
      tg.age = 0.0;
      this._pending.push(rnd.uniform(this.cfg.respawn[0], this.cfg.respawn[1]));
      // Treffer-Partikel + Punkte-Popup
      const th = this.theme();
      for (let i = 0; i < 10; i++) {
        const a = rnd.uniform(0, TAU);
        const b = rnd.uniform(-1, 1);
        const sp = rnd.uniform(2.0, 5.0);
        const v = [Math.cos(a) * sp, b * sp, Math.sin(a) * sp];
        this.particles.push([pos.slice(), v, 0.0, rnd.uniform(0.3, 0.55), th.body]);
      }
      this.popups.push(["+" + pts, pos, 0.0]);
    }

    registerMiss() {
      this.misses += 1;
      if (this.mode === "moving") this.combo = 0;
    }

    // ===================================================== Eingabe / Update
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.gameOver) {
        if (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space")) this.reset();
        return;
      }
      if (ev.kind === "mouserel") {
        this.applyLook(ev.rel);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.shoot();
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "plus" || k === "equal" || k === "KP_Add") this.changeSens(+0.1);
        else if (k === "minus" || k === "KP_Subtract") this.changeSens(-0.1);
        else if ((k === "e" || k === "E") && this.mode === "chill") this.finishRun();
      }
    }

    update(dt) {
      this.animT += dt;
      if (this.state !== PLAY || this.gameOver) return;
      this.playT += dt;
      this.cooldown = Math.max(0.0, this.cooldown - dt);
      this._flash = Math.max(0.0, this._flash - dt);
      this._tracer = Math.max(0.0, this._tracer - dt);
      this._hitmark = Math.max(0.0, this._hitmark - dt);
      this._kick -= this._kick * Math.min(1.0, dt * 9.0);
      if (this._toastT > 0) this._toastT -= dt;

      this.updateTargets(dt);

      // Chill: Combo verfällt nach 5 s ohne Treffer
      if (this.mode === "chill" && this.combo > 0 && this.playT - this.lastHitT > 5.0) this.combo = 0;

      // Partikel / Popups
      const alive = [];
      for (const p of this.particles) {
        p[2] += dt;
        if (p[2] < p[3]) {
          p[0][0] += p[1][0] * dt;
          p[0][1] += p[1][1] * dt;
          p[0][2] += p[1][2] * dt;
          alive.push(p);
        }
      }
      this.particles = alive;
      this.popups = this.popups.filter((pp) => pp[2] + dt < 0.9).map((pp) => [pp[0], pp[1], pp[2] + dt]);

      if (this.timeLeft !== null) {
        this.timeLeft -= dt;
        if (this.timeLeft <= 0) {
          this.timeLeft = 0.0;
          this.finishRun();
        }
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      this._scx = this.width / 2;
      this._scy = this.height / 2;
      this._f = this.height * FOV_MUL;

      if (this.state === SETUP) {
        // update() läuft im Setup normal -> animT kommt von dort
        this.drawSetup(ctx);
        return;
      }

      if (this.gameOver) {
        // update() steht still -> Hintergrund über Echtzeit weiterdrehen
        const now = ui.now();
        if (this._goLast !== null) {
          const gdt = Math.min(0.05, now - this._goLast);
          this.animT += gdt;
          this.yaw = PG.mod(this.yaw + PG.radians(8) * gdt, TAU);
        }
        this._goLast = now;
      }

      this._basis = this.viewBasis();
      this.drawBackground(ctx);
      this.drawTargets(ctx);
      this.drawParticles(ctx);
      this.drawPopups(ctx);
      this.drawTracerFlash(ctx);
      this.applyBlur(ctx); // Motion Blur nur auf die Szene
      this.drawCrosshair(ctx);
      this.drawHud(ctx);
      if (this.gameOver) this.drawResult(ctx);
    }

    // ----- Hintergrund / Themes ------------------------------------------------

    sky(ctx) {
      const key = this.themeKey;
      if (this._skyCache && this._skyCache[0] === key) return this._skyCache[1];
      const [top, bottom] = this.theme().sky;
      const g = ctx.createLinearGradient(0, 0, 0, this.height);
      g.addColorStop(0, ui.col(top));
      g.addColorStop(1, ui.col(bottom));
      this._skyCache = [key, g];
      return g;
    }

    drawBackground(ctx) {
      ctx.fillStyle = this.sky(ctx);
      ctx.fillRect(0, 0, this.width, this.height);
      if (this.themeKey === "space") {
        this.drawStars(ctx, 220, [200, 210, 235]);
        this.drawPlanet(ctx);
        this.drawBlackhole(ctx);
      } else if (this.themeKey === "neon") {
        this.drawStars(ctx, 120, [255, 200, 240]);
        this.drawNeonSun(ctx);
        this.drawNeonGrid(ctx);
      } else {
        this.drawRangeHall(ctx);
      }
    }

    drawStars(ctx, count, tint) {
      const [r, u, f] = this._basis;
      const w = this.width, h = this.height;
      for (let i = 0; i < count; i++) {
        const [d, size, ph] = this._stars[i];
        const cz = d[0] * f[0] + d[1] * f[1] + d[2] * f[2];
        if (cz < 0.05) continue;
        const cx = d[0] * r[0] + d[1] * r[1] + d[2] * r[2];
        const cy = d[0] * u[0] + d[1] * u[1] + d[2] * u[2];
        const k = this._f / cz;
        const sx = this._scx + cx * k;
        const sy = this._scy - cy * k;
        if (sx >= 0 && sx < w && sy >= 0 && sy < h) {
          const b = 0.55 + 0.45 * Math.sin(this.animT * 1.7 + ph);
          ctx.fillStyle = ui.col([tint[0] * b * 0.6 + 60, tint[1] * b * 0.6 + 60, tint[2] * b * 0.6 + 60]);
          ctx.fillRect(Math.trunc(sx), Math.trunc(sy), size, size);
        }
      }
    }

    blackholeSprite() {
      const R = Math.max(24, Math.trunc((this._f * 3.2) / 45.0));
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      if (this._bhCache && this._bhCache[0] === R && this._bhCache[3] === ps) return [this._bhCache[1], this._bhCache[2]];
      const w = Math.trunc(R * 6.8), h = Math.trunc(R * 3.4);
      const cx = Math.floor(w / 2), cy = Math.floor(h / 2);
      const surf = ui.makeCanvas(w * ps, h * ps);
      const g = surf.getContext("2d");
      g.scale(ps, ps);
      // Formen ERSETZEN die Pixel (wie pygame.draw auf SRCALPHA-Surfaces)
      const put = (path, rgba) => {
        g.save();
        g.globalCompositeOperation = "destination-out";
        g.fillStyle = "#000";
        path();
        g.fill();
        g.restore();
        if (rgba[3] > 0) {
          g.fillStyle = ui.col(rgba);
          path();
          g.fill();
        }
      };
      const circ = (x, y, r) => () => {
        g.beginPath();
        g.arc(x, y, r, 0, TAU);
      };
      const ell = (rw, rh) => () => {
        g.beginPath();
        g.ellipse(cx, cy, rw / 2, rh / 2, 0, 0, TAU);
      };
      // dezenter warmer Glow um den Kern
      put(circ(cx, cy, Math.trunc(R * 1.6)), [255, 240, 220, 22]);
      put(circ(cx, cy, Math.trunc(R * 1.3)), [255, 240, 220, 36]);
      // horizontale Akkretions-Scheibe (der "Saturn-Ring")
      for (const [grow, alpha] of [[1.15, 60], [1.0, 190]]) {
        put(ell(Math.trunc(R * 3.4 * 2 * grow), Math.trunc(R * 0.44 * 2 * grow)), [255, 236, 210, alpha]);
      }
      // Loch in der Scheibe, damit der Ring als Ring lesbar bleibt
      put(ell(Math.trunc(R * 1.7 * 2), Math.trunc(R * 0.3 * 2)), [0, 0, 0, 0]);
      // schwarzer Kern (Ereignishorizont)
      put(circ(cx, cy, R), [2, 2, 4, 255]);
      // Schimmer-Sprite: NUR der Ring + Photonenring (für additives Pulsen)
      const shimmer = ui.makeCanvas(w * ps, h * ps);
      const sg = shimmer.getContext("2d");
      sg.scale(ps, ps);
      // heller Linsen-Ring - leicht gekippt, VOR dem Kern (Gargantua-Look)
      const lw = Math.max(2, Math.floor(R / 22));
      sg.save();
      sg.translate(cx, cy);
      sg.rotate(PG.radians(-18));
      sg.strokeStyle = "rgb(255,244,224)";
      sg.lineWidth = lw;
      sg.beginPath();
      sg.ellipse(0, 0, R * 3.0 - lw / 2, R * 0.9 - lw / 2, 0, 0, TAU);
      sg.stroke();
      sg.restore();
      const pw = Math.max(1, Math.floor(R / 30));
      sg.strokeStyle = ui.col([255, 244, 224, 220]);
      sg.lineWidth = pw;
      sg.beginPath();
      sg.arc(cx, cy, Math.trunc(R * 1.06) - pw / 2, 0, TAU);
      sg.stroke();
      g.save();
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.drawImage(shimmer, 0, 0);
      g.restore();
      this._bhCache = [R, surf, shimmer, ps];
      return [surf, shimmer];
    }

    drawBlackhole(ctx) {
      const d = dirFrom(PG.radians(40), PG.radians(8));
      const pos = [CAM_POS[0] + d[0] * 45, CAM_POS[1] + d[1] * 45, CAM_POS[2] + d[2] * 45];
      const b = this.bill(pos);
      if (b === null) return;
      const [sprite, shimmer] = this.blackholeSprite();
      const R = this._bhCache[0];
      const w = Math.trunc(R * 6.8), h = Math.trunc(R * 3.4);
      const x = Math.trunc(b[0]) - Math.floor(w / 2), y = Math.trunc(b[1]) - Math.floor(h / 2);
      if (x + w < 0 || x > this.width || y + h < 0 || y > this.height) return;
      ctx.drawImage(sprite, x, y, w, h);
      // Nur der Ring schimmert (additiv, pulsierend)
      ctx.save();
      ctx.globalCompositeOperation = "lighter";
      ctx.globalAlpha = Math.max(0, Math.trunc(35 + 30 * Math.sin(this.animT * 0.8))) / 255;
      ctx.drawImage(shimmer, x, y, w, h);
      ctx.restore();
    }

    drawPlanet(ctx) {
      const d = dirFrom(PG.radians(-70), PG.radians(4));
      const pos = [CAM_POS[0] + d[0] * 40, CAM_POS[1] + d[1] * 40, CAM_POS[2] + d[2] * 40];
      const b = this.bill(pos);
      if (b === null) return;
      const r = Math.max(8, Math.trunc((this._f * 0.6) / 40.0));
      const x = Math.trunc(b[0]), y = Math.trunc(b[1]);
      draw.circle(ctx, [30, 26, 40], [x, y], r);
      draw.circle(ctx, [180, 150, 220], [x - Math.floor(r / 4), y - Math.floor(r / 4)], r, 2);
      draw.circle(ctx, [52, 44, 66], [x + Math.floor(r / 5), y + Math.floor(r / 5)], Math.trunc(r * 0.82));
    }

    drawNeonSun(ctx) {
      const d = dirFrom(0.0, PG.radians(10));
      const pos = [CAM_POS[0] + d[0] * 50, CAM_POS[1] + d[1] * 50, CAM_POS[2] + d[2] * 50];
      const b = this.bill(pos);
      if (b === null) return;
      const r = Math.max(30, Math.trunc((this._f * 7.0) / 50.0));
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      if (this._sunCache === null || this._sunCache[0] !== r || this._sunCache[2] !== ps) {
        const surf = ui.makeCanvas(2 * r * ps, 2 * r * ps);
        const g = surf.getContext("2d");
        g.scale(ps, ps);
        const lg = g.createLinearGradient(0, 0, 0, 2 * r);
        lg.addColorStop(0, "rgb(255,120,180)");
        lg.addColorStop(1, "rgb(255,190,90)");
        g.fillStyle = lg;
        g.beginPath();
        g.arc(r, r, r, 0, TAU);
        g.fill();
        // klassische Streifen in der unteren Hälfte
        for (let i = 0; i < 4; i++) {
          const yy = r + Math.trunc(r * (0.15 + i * 0.22));
          const hh = Math.max(2, Math.trunc(r * 0.06 + i * 2));
          g.clearRect(0, yy, 2 * r, hh);
        }
        this._sunCache = [r, surf, ps];
      }
      ctx.drawImage(this._sunCache[1], Math.trunc(b[0]) - r, Math.trunc(b[1]) - r, 2 * r, 2 * r);
    }

    drawNeonGrid(ctx) {
      const colMain = [255, 60, 200];
      const colHi = [255, 120, 220];
      let i = 0;
      for (let gz = -40; gz < 42; gz += 2, i++) {
        const seg = AimTrainerGame.clipSeg(this.toCam([-40.0, 0.0, gz]), this.toCam([40.0, 0.0, gz]));
        if (seg) this.gridLine(ctx, seg, i % 5 === 0 ? colHi : colMain);
      }
      i = 0;
      for (let gx = -40; gx < 42; gx += 2, i++) {
        const seg = AimTrainerGame.clipSeg(this.toCam([gx, 0.0, -40.0]), this.toCam([gx, 0.0, 40.0]));
        if (seg) this.gridLine(ctx, seg, i % 5 === 0 ? colHi : colMain);
      }
    }

    gridLine(ctx, seg, col) {
      const [a, b] = seg;
      const depth = (a[2] + b[2]) / 2;
      if (depth > this.theme().fog_end) return;
      const pa = this.proj(a), pb = this.proj(b);
      const w = this.width;
      if ((pa[0] < -w && pb[0] < -w) || (pa[0] > 2 * w && pb[0] > 2 * w)) return;
      draw.line(ctx, this.fogColor(col, depth), pa, pb, 1);
    }

    drawRangeHall(ctx) {
      const items = [];
      const L = 22.0, HT = 6.0;

      const add = (pts, col) => {
        let cam = pts.map((p) => this.toCam(p));
        if (cam.every((c) => c[2] <= NEAR)) return;
        cam = AimTrainerGame.clipNear(cam);
        if (cam.length < 3) return;
        let depth = 0;
        for (const c of cam) depth += c[2];
        depth /= cam.length;
        const proj = cam.map((c) => this.proj(c));
        items.push([depth, proj, this.fogColor(col, depth)]);
      };

      // Boden (8 Quads) + Decke (4)
      for (let i = 0; i < 4; i++) {
        const x0 = -L + i * (L / 2);
        add([[x0, 0, -L], [x0 + L / 2, 0, -L], [x0 + L / 2, 0, 0], [x0, 0, 0]], [40, 42, 46]);
        add([[x0, 0, 0], [x0 + L / 2, 0, 0], [x0 + L / 2, 0, L], [x0, 0, L]], [43, 45, 49]);
        add([[x0, HT, -L], [x0 + L / 2, HT, -L], [x0 + L / 2, HT, L], [x0, HT, L]], [34, 36, 42]);
      }
      // Wände (je 4 Streifen)
      for (let i = 0; i < 4; i++) {
        const z0 = -L + i * (L / 2);
        const shade = i % 2 === 0 ? 0.9 : 0.75;
        const col = [52, 54, 60].map((c) => Math.trunc(c * shade));
        add([[-L, 0, z0], [-L, 0, z0 + L / 2], [-L, HT, z0 + L / 2], [-L, HT, z0]], col);
        add([[L, 0, z0], [L, 0, z0 + L / 2], [L, HT, z0 + L / 2], [L, HT, z0]], col);
        const x0 = -L + i * (L / 2);
        add([[x0, 0, -L], [x0 + L / 2, 0, -L], [x0 + L / 2, HT, -L], [x0, HT, -L]], col);
        add([[x0, 0, L], [x0 + L / 2, 0, L], [x0 + L / 2, HT, L], [x0, HT, L]], col);
      }
      items.sort((p, q) => q[0] - p[0]);
      for (const [, pts, col] of items) {
        draw.polygon(ctx, col, pts);
        // Nahtlose Kanten (pygame füllt pixelgenau, Canvas antialiast)
        draw.polygon(ctx, col, pts, 1);
      }
      // Bahnlinien auf dem Boden
      for (let gx = -10; gx < 11; gx += 5) {
        const seg = AimTrainerGame.clipSeg(this.toCam([gx / 2, 0.01, -L]), this.toCam([gx / 2, 0.01, L]));
        if (seg) this.gridLine(ctx, seg, [66, 70, 78]);
      }
      // Lampen
      for (const lx of [-8.0, 0.0, 8.0]) {
        for (const lz of [-16.0, -6.0, 6.0, 16.0]) {
          const b = this.bill([lx, HT - 0.1, lz]);
          if (b === null) continue;
          const wpx = Math.max(2, Math.trunc(b[2] * 0.5));
          draw.rect(ctx, [235, 235, 215], [Math.trunc(b[0]) - wpx, Math.trunc(b[1]) - Math.floor(wpx / 3), 2 * wpx, Math.max(2, Math.floor(wpx / 2))]);
        }
      }
    }

    // ----- Targets & Effekte -----------------------------------------------------

    drawTargets(ctx) {
      const th = this.theme();
      const drawlist = [];
      for (const tg of this.targets) {
        const pos = this.targetPos(tg);
        const b = this.bill(pos);
        if (b === null) continue;
        const scale = this.targetScale(tg);
        const pulse = 1.0 + 0.04 * Math.sin(this.animT * 2 + tg.phase);
        const rpx = b[2] * tg.radius * scale * pulse;
        if (rpx < 1) continue;
        drawlist.push([b[3], b[0], b[1], rpx]);
      }
      drawlist.sort((p, q) => q[0] - p[0]);
      for (const [depth, sx, sy, rpx] of drawlist) {
        const x = Math.trunc(sx), y = Math.trunc(sy), r = Math.trunc(rpx);
        if (x < -r || x > this.width + r || y < -r || y > this.height + r) continue;
        if (th.glow) {
          // additiver Glow (BLEND_RGBA_ADD)
          ctx.save();
          ctx.globalCompositeOperation = "lighter";
          draw.circle(ctx, th.glow, [x, y], Math.trunc(r * 1.5));
          ctx.restore();
        }
        const body = this.fogColor(th.body, depth);
        const ring = this.fogColor(th.ring, depth);
        const dot = this.fogColor(th.dot, depth);
        draw.circle(ctx, body, [x, y], r);
        if (r >= 5) draw.circle(ctx, ring, [x, y], Math.trunc(r * 0.65), Math.max(2, Math.floor(r / 5)));
        draw.circle(ctx, dot, [x, y], Math.max(1, Math.trunc(r * 0.28)));
      }
    }

    drawParticles(ctx) {
      for (const [pos, , age, life, col] of this.particles) {
        const b = this.bill(pos);
        if (b === null) continue;
        const f = 1.0 - age / life;
        const r = Math.max(1, Math.trunc(b[2] * 0.06 * f));
        draw.circle(ctx, this.fogColor(col, b[3]), [Math.trunc(b[0]), Math.trunc(b[1])], r);
      }
    }

    drawPopups(ctx) {
      for (const [txt, pos, age] of this.popups) {
        const b = this.bill(pos);
        if (b === null) continue;
        ui.text(ctx, txt, Math.trunc(b[0]), Math.trunc(b[1] - age * 46), this._small, this.theme().accent, "center", Math.max(0, 1.0 - age / 0.9));
      }
    }

    drawTracerFlash(ctx) {
      const gx = Math.trunc(this.width * 0.54), gy = Math.trunc(this.height * 0.98);
      const cx = Math.trunc(this._scx), cy = Math.trunc(this._scy);
      const ac = this.theme().accent;
      if (this._tracer > 0) {
        const f = this._tracer / 0.08;
        const dim = ac.map((c) => Math.trunc(c * 0.45 * f));
        const bright = ac.map((c) => Math.trunc(c * f));
        draw.line(ctx, dim, [gx, gy], [cx, cy], 3);
        draw.line(ctx, bright, [gx, gy], [cx, cy], 1);
      }
      if (this._flash > 0) {
        const r = 12;
        draw.circle(ctx, [255, 240, 200], [gx, gy - 4], r / 2);
        for (const a of [0, 90, 45, 135]) {
          const rad = PG.radians(a);
          draw.line(ctx, [255, 220, 160], [gx - Math.cos(rad) * r, gy - 4 - Math.sin(rad) * r], [gx + Math.cos(rad) * r, gy - 4 + Math.sin(rad) * r], 2);
        }
      }
    }

    drawCrosshair(ctx) {
      const cx = Math.trunc(this._scx), cy = Math.trunc(this._scy);
      let col = this.theme().accent;
      if (this._hitmark > 0) col = [255, 255, 255];
      const ext = Math.trunc(4 * this._kick);
      const gap = 5, arm = 9 + ext;
      for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        draw.line(ctx, col, [cx + dx * gap, cy + dy * gap], [cx + dx * (gap + arm), cy + dy * (gap + arm)], 2);
      }
      draw.circle(ctx, col, [cx, cy], 2);
      draw.circle(ctx, [col[0], col[1], col[2], 120], [cx, cy], 14, 1);
    }

    // ----- HUD / Ergebnis ---------------------------------------------------------

    fmtMmss(sec) {
      sec = Math.trunc(sec);
      return String(Math.floor(sec / 60)).padStart(2, "0") + ":" + String(sec % 60).padStart(2, "0");
    }

    drawHud(ctx) {
      const col = this.theme().accent;
      ui.text(ctx, t("common.points", { score: this.score }), 14, 10, this._big, col);
      const acc = this.shots ? Math.trunc((this.hits / this.shots) * 100) : 100;

      let right = [];
      if (this.mode === "precision") {
        right = [Math.max(0.0, this.timeLeft).toFixed(1), `${this.hits}/${this.shots} · ${acc}%`];
      } else if (this.mode === "reflex") {
        right = [t("aim.targets", { n: Math.min(this.spawned, 30), m: 30 })];
        if (this.reactions.length) {
          const avg = Math.floor(this.reactions.reduce((a, b) => a + b, 0) / this.reactions.length);
          right.push(t("aim.reaction_avg", { ms: avg }));
        }
      } else if (this.mode === "moving") {
        const mult = 1.0 + 0.25 * Math.min(this.combo, 12);
        right = [Math.max(0.0, this.timeLeft).toFixed(1), t("aim.combo", { m: mult.toFixed(2) }), `${this.hits}/${this.shots}`];
      } else {
        right = [t("aim.hits", { n: this.hits }), t("aim.session_time", { t: this.fmtMmss(this.playT) })];
        if (this.combo > 1) {
          const mult = 1.0 + 0.25 * Math.min(this.combo, 12);
          right.push(t("aim.combo", { m: mult.toFixed(2) }));
        }
      }
      let y = 12;
      for (const line of right) {
        ui.text(ctx, line, this.width - 14, y, this._small, col, "topright");
        y += 22;
      }

      // Reflex: letzte Reaktionszeit kurz einblenden
      if (this.mode === "reflex" && this.lastReaction !== null && this.playT - this.lastReactionT < 0.8) {
        const ms = this.lastReaction;
        const c = ms < 400 ? ui.GREEN : ms < 700 ? ui.TEXT : ui.GOLD;
        ui.text(ctx, `${ms} ms`, Math.floor(this.width / 2), Math.trunc(this.height * 0.72), this._big, c, "center");
      }

      if (this.mode === "chill" && !this.gameOver) {
        ui.text(ctx, t("aim.end_hint"), Math.floor(this.width / 2), this.height - 6, this._tiny, col, "midbottom");
      }

      if (this._toast && this._toastT > 0) {
        ui.text(ctx, this._toast, Math.floor(this.width / 2), 12, this._small, ui.TEXT, "midtop");
      }
    }

    drawResult(ctx) {
      this.dim(ctx, [0, 0, 0], 140);
      const cx = Math.floor(this.width / 2);
      const cy = Math.floor(this.height / 2) - 40;
      const acc = this.shots ? Math.trunc((this.hits / this.shots) * 100) : 100;

      const headTxt = t("aim.result_title");
      let lines = [t("common.points", { score: this.score })];
      if (this.mode === "precision") {
        lines = lines.concat([
          `${t("aim.hits", { n: this.hits })}   ·   ${t("aim.misses", { n: this.misses })}`,
          t("aim.accuracy", { p: acc }),
          t("aim.bonus_accuracy", { n: this.accBonus || 0 }),
        ]);
      } else if (this.mode === "reflex") {
        const avg = this.reactions.length ? Math.floor(this.reactions.reduce((a, b) => a + b, 0) / this.reactions.length) : 0;
        const best = this.reactions.length ? Math.min(...this.reactions) : 0;
        lines = lines.concat([`${t("aim.hits", { n: this.hits })} / 30`, t("aim.reaction_avg", { ms: avg }), t("aim.reaction_best", { ms: best })]);
      } else if (this.mode === "moving") {
        lines = lines.concat([t("aim.accuracy", { p: acc }), t("aim.max_combo", { m: pyFloat(1.0 + 0.25 * Math.min(this.maxCombo, 12)) })]);
      } else {
        lines = lines.concat([
          t("aim.hits", { n: this.hits }),
          t("aim.max_combo", { m: pyFloat(1.0 + 0.25 * Math.min(this.maxCombo, 12)) }),
          t("aim.session_time", { t: this.fmtMmss(this.playT) }),
        ]);
      }
      const hintTxt = t("common.enter_restart");

      // Panel hinter dem Ergebnis (dynamische ui-Palette)
      const top = cy - 70 - Math.floor(this._huge.height / 2) - 22;
      const bottom = cy - 24 + 32 * lines.length + 10 + Math.floor(this._small.height / 2) + 22;
      const pw = Math.min(this.width - 40, Math.max(400, this._huge.width(headTxt) + 80, this._small.width(hintTxt) + 60, Math.max(...lines.map((l) => this.font.width(l))) + 60));
      const panel = new PG.Rect(cx - Math.floor(pw / 2), top, pw, bottom - top);
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, ui.BORDER_LIGHT, panel, 1, 14);

      ui.text(ctx, headTxt, cx, cy - 70, this._huge, this.theme().accent, "center");
      let y = cy - 24;
      for (const line of lines) {
        ui.text(ctx, line, cx, y, this.font, ui.TEXT, "center");
        y += 32;
      }
      ui.text(ctx, hintTxt, cx, y + 10, this._small, ui.TEXT_DIM, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------------------
    drawSetup(ctx) {
      // Live-Vorschau des gewählten Themes mit langsamer Auto-Drehung
      const oldYaw = this.yaw, oldPitch = this.pitch;
      this.yaw = PG.mod(this.animT * PG.radians(6), TAU);
      this.pitch = PG.radians(4);
      this._basis = this.viewBasis();
      this.drawBackground(ctx);
      this.applyBlur(ctx); // Live-Vorschau der Blur-Einstellung
      this.yaw = oldYaw;
      this.pitch = oldPitch;

      this.dim(ctx, [5, 6, 12], 120);

      const cx = Math.floor(this.width / 2);
      const accent = this.theme().accent;
      ui.text(ctx, "AIM TRAINER", cx, Math.trunc(this.height * 0.13), this._huge, accent, "center");
      const modeLbl = MODE_CFG[this.mode] ? t("aim.mode." + this.mode) : this.mode;
      ui.text(ctx, modeLbl + "   -   " + t("aim.subtitle"), cx, Math.trunc(this.height * 0.2), this._small, ui.TEXT_DIM, "center");

      ui.text(ctx, t("aim.setup_theme"), cx, this.themeRects[0].y - 8, this._small, ui.TEXT_DIM, "midbottom");
      this.themeRects.forEach((r, i) => {
        const on = THEME_KEYS[i] === this.themeKey;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 8);
        draw.rect(ctx, on ? accent : ui.BORDER, r, on ? 2 : 1, 8);
        ui.text(ctx, t("aim.theme." + THEME_KEYS[i]), r.centerx, r.centery, this._small, on ? ui.TEXT : ui.TEXT_DIM, "center");
      });

      const rows = [
        ["aim.setup_sens", this.sensMinus, this.sensPlus, this.sensBox, this.sens.toFixed(1)],
        ["aim.setup_blur", this.blurMinus, this.blurPlus, this.blurBox, this.blurLabel()],
      ];
      for (const [labelKey, minus, plus, box, value] of rows) {
        ui.text(ctx, t(labelKey), minus.x - 16, minus.centery, this._small, ui.TEXT_DIM, "midright");
        for (const [r, sym] of [[minus, "-"], [plus, "+"]]) {
          draw.rect(ctx, ui.BTN, r, 0, 8);
          draw.rect(ctx, ui.BORDER, r, 1, 8);
          ui.text(ctx, sym, r.centerx, r.centery, this._big, ui.TEXT, "center");
        }
        ui.text(ctx, value, box.centerx, box.centery, this._big, accent, "center");
      }

      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");

      // Hinweis ggf. umbrechen (Web-Fläche ist schmaler als typische Fenster)
      const hint = t("aim.setup_hint");
      const hl = ui.wrap(hint, this._tiny, this.width - 40);
      hl.forEach((line, i) => {
        ui.text(ctx, line, cx, this.height - 14 - (hl.length - 1 - i) * this._tiny.height, this._tiny, ui.TEXT_FAINT, "center");
      });
    }
  }

  PG.register(AimTrainerGame, {
    id: "AimTrainerGame",
    key: "aim",
    name: "Aim Trainer",
    modes: [
      ["precision", "aim.mode.precision"],
      ["reflex", "aim.mode.reflex"],
      ["moving", "aim.mode.moving"],
      ["chill", "aim.mode.chill"],
    ],
    settingsKey: "aim",
    defaults: { theme: "space", sens: 1.0, blur: 0.0 },
  });
})();
