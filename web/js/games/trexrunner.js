/*
 * trexrunner.js - T-Rex Runner (Port von games/trexrunner.py)
 * ============================================================
 * Der endlose Wüstenlauf (Hommage an das Chrome-Dino-Spiel).
 *
 * - Ein T-Rex läuft von links, du springst über Kakteen und duckst dich unter
 *   Flugsauriern hindurch. Das Tempo steigt kontinuierlich mit der Strecke.
 * - Variable Sprunghöhe: je länger du die Sprungtaste hältst, desto höher
 *   springt der Dino (kürzer Antippen = kleiner Hopser). In der Luft nach unten
 *   = schneller Fall.
 * - Flugsaurier tauchen auf drei Höhen auf: hoch = ducken, tief/mittig = springen.
 * - Tag/Nacht-Wechsel mit Sternenhimmel und Mond, Parallax-Wolken, scrollender
 *   Boden mit Bodenwellen. Alle 100 Punkte ein kurzer Ton + Blinken.
 * - Optionen (bleiben erhalten, settings.trex): Schwierigkeit (chill/normal/
 *   hardcore), Figur/Skin und Tag/Nacht an/aus - im Startbildschirm umschaltbar.
 *
 * Steuerung: Leertaste/Pfeil-hoch/Aktion = Springen (halten für höher),
 * Pfeil-runter = Ducken/schneller fallen. Nach Game Over: Enter/Leertaste = neu.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ---- Farbpaletten (Tag / Nacht) - werden für weiche Übergänge interpoliert.
  const DAY = {
    sky: [244, 246, 250], ground: [84, 84, 92], line: [120, 120, 130], obj: [74, 78, 90],
    cloud: [210, 214, 224], star: [244, 246, 250], text: [60, 64, 78], dim: [120, 126, 140],
  };
  const NIGHT = {
    sky: [24, 26, 42], ground: [150, 154, 168], line: [110, 116, 140], obj: [198, 202, 216],
    cloud: [70, 76, 104], star: [250, 250, 210], text: [226, 230, 242], dim: [150, 156, 178],
  };

  // Skins = Körperfarbe des Dinos (Tag / Nacht wird davon leicht abgeleitet).
  const SKINS = [[94, 106, 122], [86, 200, 130], [240, 150, 70], [110, 160, 240]];

  const DIFFS = ["chill", "normal", "hardcore"];
  // [Start-Tempo, Beschleunigung px/s^2, Vogel-ab-Score] - jeweils Design-Einheiten.
  const DIFF_PARAMS = {
    chill: [300, 5, 250],
    normal: [380, 8, 150],
    hardcore: [470, 12, 60],
  };

  const READY = "ready", RUN = "run", OVER = "over";

  const BIRD_HIGH = "bird_high", BIRD_MID = "bird_mid", BIRD_LOW = "bird_low";

  const I = Math.trunc; // int() wie in Python (Richtung 0)
  const pad5 = (n) => String(n).padStart(5, "0");

  class TRexRunnerGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      this.score = 0;

      const tr = this.opts;
      this.diff = DIFFS.includes(tr.difficulty) ? tr.difficulty : "normal";
      this.skin = Math.max(0, Math.min(SKINS.length - 1, Math.trunc(Number(tr.skin)) || 0));
      this.nightOn = !!tr.night;

      this.makeFonts();
      this.best = this.highscore;

      this.layout();
      this.newRun();
      this.state = READY;
    }

    /**
     * Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schriftart).
     * Der Punktestand nutzt eine Monospace-Schrift, damit die Ziffern im
     * Chrome-Stil ("HI 00512  00047") nicht wackeln.
     */
    makeFonts() {
      const h = this.height;
      this.big = ui.font(Math.max(26, Math.floor(h / 12)), true);
      this.mid = ui.font(Math.max(16, Math.floor(h / 24)), true);
      this.small = ui.font(Math.max(12, Math.floor(h / 32)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 36)));
      this.mono = ui.font(Math.max(14, Math.floor(h / 26)), true, true);
    }

    layout() {
      this.scale = this.height / 480;
      this.gy = I(this.height * 0.8); // Bodenlinie (y)
      this.dinoX = I(this.width * 0.16);
      this.dinoW = I(44 * this.scale);
      this.dinoH = I(48 * this.scale);
      this.duckW = I(60 * this.scale);
      this.duckH = I(28 * this.scale);
    }

    newRun() {
      const [base, accel] = DIFF_PARAMS[this.diff];
      this.speed = base * this.scale;
      this.baseSpeed = base * this.scale;
      this.accel = accel * this.scale;
      this.maxSpeed = 900 * this.scale;
      this.dist = 0;
      this.score = 0;
      this.nextMilestone = 100;

      // Dino-Zustand
      this.dy = 0; // vertikaler Versatz über dem Boden (positiv = oben)
      this.vy = 0;
      this.onGround = true;
      this.ducking = false;
      this.jumpHeld = false;
      this.duckHeld = false;
      this.runFrame = 0;
      this.deadT = 0;
      this.blink = 0;

      this.obstacles = []; // {x, w, h, bird, oy, flap}
      this.spawnGap = this.randGap() * 0.6;
      const R = PG.rand;
      this.clouds = [];
      for (let i = 0; i < 3; i++) {
        this.clouds.push([R.uniform(0, this.width), R.uniform(this.height * 0.12, this.height * 0.42), R.uniform(0.25, 0.5)]);
      }
      this.stars = [];
      for (let i = 0; i < 46; i++) this.stars.push([R.uniform(0, this.width), R.uniform(0, this.gy * 0.7), R.choice([1, 1, 2])]);
      this.groundBumps = [];
      for (let i = 0; i < 24; i++) this.groundBumps.push(R.uniform(0, this.width));
      // Tag/Nacht: 0 = Tag, 1 = Nacht (weich interpoliert)
      this.night = 0;
      this.nightTarget = 0;
      this.nextFlip = 300;
    }

    /** Zufälliger horizontaler Abstand zum nächsten Hindernis (px). */
    randGap() {
      // Mindestabstand skaliert mit dem Tempo (schnell = mehr Platz zum Reagieren).
      const base = PG.rand.uniform(320, 560) * this.scale;
      return base * Math.sqrt(this.speed / this.baseSpeed);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ===================================================== Eingabe
    isJumpKey(k) {
      return k === "space" || k === "Up" || k === "w" || k === "W" || this.isAction(k, "action") || this.isAction(k, "up");
    }

    isDuckKey(k) {
      return k === "Down" || k === "s" || k === "S" || this.isAction(k, "down");
    }

    handleEvent(ev) {
      if (this.state === READY) {
        this.handleReady(ev);
        return;
      }
      if (this.state === OVER) {
        if ((ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space" || ev.key === "Up")) || ev.kind === "mousedown") {
          this.newRun();
          this.state = RUN;
          // Web: gameOver zurücksetzen, sonst ruft die App update() nicht mehr auf
          this.gameOver = false;
          this.playSound("select");
        }
        return;
      }

      // ---- RUN
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.isJumpKey(k)) {
          this.jumpHeld = true;
          this.tryJump();
        } else if (this.isDuckKey(k)) {
          this.duckHeld = true;
        }
      } else if (ev.kind === "keyup") {
        // WICHTIG: dieselben Tasten prüfen wie beim KEYDOWN (inkl. der frei
        // belegbaren Aktionstasten) - sonst bleibt der Sprung "hängen".
        const k = ev.key;
        if (this.isJumpKey(k)) this.jumpHeld = false;
        else if (this.isDuckKey(k)) this.duckHeld = false;
      } else if (ev.kind === "mousedown") {
        this.jumpHeld = true;
        this.tryJump();
      } else if (ev.kind === "mouseup") {
        this.jumpHeld = false;
      }
    }

    handleReady(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diff = DIFFS[Number(k) - 1];
          this.saveSetting("difficulty", this.diff);
          this.newRun();
          this.playSound("click");
        } else if (k === "n" || k === "N") {
          this.nightOn = !this.nightOn;
          this.saveSetting("night", this.nightOn);
          this.playSound("click");
        } else if (k === "c" || k === "C") {
          this.skin = (this.skin + 1) % SKINS.length;
          this.saveSetting("skin", this.skin);
          this.playSound("click");
        } else if (k === "space" || k === "Up" || k === "Return" || k === "w" || k === "W" || this.isAction(k, "action")) {
          this.state = RUN;
          this.jumpHeld = true;
          this.tryJump();
          this.playSound("select");
        }
      } else if (ev.kind === "mousedown") {
        this.state = RUN;
        this.jumpHeld = true;
        this.tryJump();
        this.playSound("select");
      }
    }

    tryJump() {
      if (this.onGround && this.state === RUN) {
        this.vy = -620 * this.scale;
        this.onGround = false;
        this.ducking = false;
        this.playSound("bounce");
      }
    }

    // ===================================================== Update
    update(dt) {
      dt = Math.min(dt, 0.05);
      if (this.blink > 0) this.blink -= dt;
      for (const c of this.clouds) {
        // Wolken driften immer
        c[0] -= this.baseSpeed * c[2] * dt * (this.state !== RUN ? 0.4 : 1.0);
        if (c[0] < -60 * this.scale) {
          c[0] = this.width + PG.rand.uniform(20, 120) * this.scale;
          c[1] = PG.rand.uniform(this.height * 0.12, this.height * 0.42);
        }
      }

      if (this.state === OVER) {
        this.deadT += dt;
        this.updateNight(dt);
        return;
      }
      if (this.state !== RUN) {
        this.runFrame += dt * 6;
        this.updateNight(dt);
        return;
      }

      // Tempo + Strecke
      this.speed = Math.min(this.maxSpeed, this.speed + this.accel * dt);
      this.dist += this.speed * dt;
      const old = this.score;
      this.score = Math.floor(this.dist / (10 * this.scale));
      if (Math.floor(this.score / 100) > Math.floor(old / 100) && this.score >= this.nextMilestone) {
        this.nextMilestone += 100;
        this.blink = 0.6;
        this.playSound("point");
      }

      this.updateDino(dt);
      this.updateObstacles(dt);
      this.updateNight(dt);
      this.runFrame += dt * (6 + this.speed / (120 * this.scale));
    }

    updateDino(dt) {
      if (this.onGround) {
        this.ducking = this.duckHeld;
      } else {
        // Schwerkraft; beim Halten der Sprungtaste im Aufstieg schwächer
        let g = 1800 * this.scale;
        if (this.jumpHeld && this.vy < 0) g = 1150 * this.scale;
        this.vy += g * dt;
        if (this.duckHeld) this.vy += 2600 * this.scale * dt; // schneller Fall
        this.dy -= this.vy * dt;
        if (this.dy <= 0) {
          this.dy = 0;
          this.vy = 0;
          this.onGround = true;
          this.ducking = this.duckHeld;
        }
      }
    }

    dinoRect() {
      let w, h;
      if (this.ducking && this.onGround) {
        w = this.duckW;
        h = this.duckH;
      } else {
        w = this.dinoW;
        h = this.dinoH;
      }
      return new PG.Rect(this.dinoX, this.gy - I(this.dy) - h, w, h);
    }

    updateObstacles(dt) {
      for (const o of this.obstacles) {
        o.x -= this.speed * dt;
        if (o.bird) o.flap += dt * 8;
      }
      this.obstacles = this.obstacles.filter((o) => o.x + o.w > -4);

      // Nachschub, sobald genug Platz hinter dem letzten Hindernis ist.
      this.spawnGap -= this.speed * dt;
      if (this.spawnGap <= 0) {
        this.spawn();
        this.spawnGap = this.randGap();
      }

      // Kollision
      const dr = this.dinoRect().inflate(I(-8 * this.scale), I(-8 * this.scale));
      for (const o of this.obstacles) {
        const r = new PG.Rect(I(o.x), I(o.oy), I(o.w), I(o.h));
        r.inflateIp(I(-5 * this.scale), I(-6 * this.scale));
        if (dr.colliderect(r)) {
          this.die();
          return;
        }
      }
    }

    spawn() {
      const birdAt = DIFF_PARAMS[this.diff][2];
      const allowBird = this.score >= birdAt;
      const R = PG.rand;
      if (allowBird && R.random() < 0.28) {
        const kind = R.choice([BIRD_HIGH, BIRD_MID, BIRD_LOW]);
        const w = I(46 * this.scale);
        const h = I(30 * this.scale);
        const off = { bird_high: 92, bird_mid: 58, bird_low: 26 }[kind];
        const oy = this.gy - I(off * this.scale) - h;
        this.obstacles.push({ x: this.width + 20 * this.scale, w, h, bird: true, oy, flap: 0, kind });
      } else {
        const variant = R.random();
        let w, h, spikes;
        if (variant < 0.4) {
          w = I(18 * this.scale);
          h = I(36 * this.scale);
          spikes = 1;
        } else if (variant < 0.72) {
          w = I(26 * this.scale);
          h = I(50 * this.scale);
          spikes = 1;
        } else {
          // Gruppe
          spikes = R.choice([2, 3]);
          w = I((14 * spikes + 6) * this.scale);
          h = I(40 * this.scale);
        }
        const oy = this.gy - h;
        this.obstacles.push({ x: this.width + 20 * this.scale, w, h, bird: false, oy, spikes });
      }
    }

    updateNight(dt) {
      if (!this.nightOn) {
        this.night += (0 - this.night) * Math.min(1, dt * 3);
        return;
      }
      if (this.state === RUN && this.score >= this.nextFlip) {
        this.nightTarget = 1 - this.nightTarget;
        this.nextFlip += 250;
      }
      this.night += (this.nightTarget - this.night) * Math.min(1, dt * 1.4);
    }

    die() {
      this.state = OVER;
      this.gameOver = true; // die App sichert den Highscore
      this.deadT = 0;
      this.ducking = false;
      if (this.score > this.best) this.best = this.score;
      this.playSound("gameover");
    }

    // ===================================================== Zeichnen
    pal(key) {
      const d = DAY[key], n = NIGHT[key];
      const f = this.night;
      return [I(d[0] + (n[0] - d[0]) * f), I(d[1] + (n[1] - d[1]) * f), I(d[2] + (n[2] - d[2]) * f)];
    }

    draw(ctx) {
      draw.rect(ctx, this.pal("sky"), [0, 0, this.width, this.height]);
      this.drawSky(ctx);
      this.drawGround(ctx);
      this.drawObstacles(ctx);
      this.drawDino(ctx);
      this.drawHud(ctx);
      if (this.state === READY) this.drawReady(ctx);
      else if (this.state === OVER) this.drawOver(ctx);
    }

    drawSky(ctx) {
      // Sonne (Tag) / Mond (Nacht)
      const cx = I(this.width * 0.82);
      const cy = I(this.height * 0.2);
      const r = I(24 * this.scale);
      if (this.night > 0.35) {
        ctx.fillStyle = ui.col(this.pal("star"));
        for (const [sx, sy, sr] of this.stars) ctx.fillRect(I(sx), I(sy), sr, sr);
        draw.circle(ctx, [232, 234, 220], [cx, cy], r);
        draw.circle(ctx, this.pal("sky"), [cx + I(9 * this.scale), cy - I(6 * this.scale)], r);
      } else {
        draw.circle(ctx, [250, 214, 120], [cx, cy], r);
        draw.circle(ctx, [250, 224, 150], [cx, cy], I(r * 1.5), 2);
      }
      // Wolken
      const col = this.pal("cloud");
      for (const [x, y] of this.clouds) this.drawCloud(ctx, I(x), I(y), col);
    }

    drawCloud(ctx, x, y, col) {
      const u = I(9 * this.scale);
      draw.ellipse(ctx, col, [x, y, u * 5, u * 2]);
      draw.ellipse(ctx, col, [x + u, y - u, u * 3, u * 2]);
    }

    drawGround(ctx) {
      const gy = this.gy;
      draw.line(ctx, this.pal("line"), [0, gy], [this.width, gy], Math.max(2, I(2 * this.scale)));
      // Bodenwellen / Kieselsteine, scrollend
      const off = this.state !== READY ? PG.mod(this.dist * 0.5, this.width) : 0;
      const col = this.pal("line");
      const by = gy + I(6 * this.scale);
      const lw = Math.max(1, I(this.scale));
      for (const bx of this.groundBumps) {
        const x = I(PG.mod(bx - off, this.width));
        draw.line(ctx, col, [x, by], [x + I(10 * this.scale), by], lw);
      }
    }

    drawDino(ctx) {
      const r = this.dinoRect();
      let body = SKINS[this.skin];
      if (this.night > 0.5) {
        // nachts leicht aufhellen
        body = body.map((c) => Math.min(255, I(c + 60 * (this.night - 0.5) * 2)));
      }
      const dark = body.map((c) => I(c * 0.6));
      const u = this.scale;
      if (this.ducking && this.onGround) {
        // Geduckt: langgestreckter Körper
        draw.rect(ctx, body, r, 0, I(6 * u));
        const eye = [r.right - I(8 * u), r.y + I(7 * u)];
        draw.circle(ctx, [250, 250, 250], eye, Math.max(2, I(3 * u)));
        draw.circle(ctx, [20, 20, 20], eye, Math.max(1, I(1.5 * u)));
        // Beinchen
        const f = I(this.runFrame) % 2;
        for (let i = 0; i < 2; i++) {
          const lx = r.x + I((10 + i * 22) * u);
          const ly = r.bottom;
          const dh = I(((i + f) % 2 ? 6 : 3) * u);
          draw.rect(ctx, dark, [lx, ly, I(5 * u), dh]);
        }
        return;
      }
      // Stehend/Springend: Körper + Kopf + Schwanz
      const head = new PG.Rect(r.right - I(20 * u), r.y, I(20 * u), I(18 * u));
      draw.rect(ctx, body, [r.x + I(6 * u), r.y + I(10 * u), r.w - I(10 * u), r.h - I(18 * u)], 0, I(5 * u));
      draw.rect(ctx, body, head, 0, I(4 * u));
      // Schwanz
      draw.polygon(ctx, body, [
        [r.x + I(6 * u), r.y + I(14 * u)],
        [r.x - I(8 * u), r.y + I(8 * u)],
        [r.x + I(6 * u), r.y + I(24 * u)],
      ]);
      // Auge
      const eye = [head.right - I(6 * u), head.y + I(6 * u)];
      draw.circle(ctx, [250, 250, 250], eye, Math.max(2, I(3 * u)));
      draw.circle(ctx, [20, 20, 20], eye, Math.max(1, I(1.5 * u)));
      // Beine (laufen nur am Boden animiert)
      const f = this.onGround && this.state === RUN ? I(this.runFrame) % 2 : 0;
      for (let i = 0; i < 2; i++) {
        const lx = r.x + I((10 + i * 14) * u);
        const ly = r.bottom - I(10 * u);
        const dh = I(((i + f) % 2 ? 10 : 5) * u);
        draw.rect(ctx, dark, [lx, ly, I(6 * u), dh]);
      }
    }

    drawObstacles(ctx) {
      const col = this.pal("obj");
      const u = this.scale;
      for (const o of this.obstacles) {
        const x = I(o.x), oy = I(o.oy), w = I(o.w), h = I(o.h);
        if (o.bird) {
          const cy = oy + Math.floor(h / 2);
          draw.ellipse(ctx, col, [x + I(8 * u), cy - I(5 * u), I(28 * u), I(12 * u)]);
          // Kopf + Schnabel
          draw.circle(ctx, col, [x + w - I(4 * u), cy], I(6 * u));
          draw.polygon(ctx, col, [[x + w, cy - I(2 * u)], [x + w + I(8 * u), cy], [x + w, cy + I(2 * u)]]);
          // Flügel: auf/ab
          const up = PG.mod(o.flap, 2) < 1;
          const tipY = up ? cy - I(16 * u) : cy + I(16 * u);
          draw.polygon(ctx, col, [[x + I(10 * u), cy], [x + I(24 * u), cy], [x + I(14 * u), tipY]]);
        } else {
          const spikes = o.spikes || 1;
          const cw = Math.floor(w / spikes);
          for (let i = 0; i < spikes; i++) {
            const bx = x + i * cw;
            const mx = bx + Math.floor(cw / 2);
            draw.rect(ctx, col, [mx - I(3 * u), oy, I(6 * u), h]); // Stamm
            // Arme
            const ay = oy + Math.floor(h / 3);
            draw.rect(ctx, col, [mx - I(9 * u), ay, I(6 * u), I(3 * u)]);
            draw.rect(ctx, col, [mx - I(9 * u), ay - I(8 * u), I(3 * u), I(10 * u)]);
            draw.rect(ctx, col, [mx + I(3 * u), ay + I(4 * u), I(6 * u), I(3 * u)]);
            draw.rect(ctx, col, [mx + I(6 * u), ay - I(4 * u), I(3 * u), I(10 * u)]);
          }
        }
      }
    }

    drawHud(ctx) {
      // Score rechts oben, Chrome-Stil: "HI 00512  00047"
      const cur = pad5(this.score);
      const hi = pad5(this.best);
      const blinkOn = this.blink > 0 && I(this.blink * 8) % 2 === 0;
      const txt = "HI " + hi;
      ui.text(ctx, txt, this.width - this.mono.width(txt) - I(90 * this.scale), 12, this.mono, this.pal("dim"));
      if (!blinkOn) ui.text(ctx, cur, this.width - this.mono.width(cur) - 14, 12, this.mono, this.pal("text"));
    }

    drawReady(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, t("trex.title"), cx, I(this.height * 0.24), this.big, this.pal("text"), "center");
      if (Math.floor(ui.ticks() / 500) % 2 === 0) {
        ui.text(ctx, t("trex.press_start"), cx, I(this.height * 0.36), this.mid, this.pal("text"), "center");
      }
      ui.text(ctx, t("trex.controls"), cx, I(this.height * 0.44), this.small, this.pal("dim"), "center");
      // Optionszeile
      const onoff = this.nightOn ? t("common.on") : t("common.off");
      const opts = [
        "[1-3] " + t("trex.difficulty") + ": " + t("trex.diff." + this.diff),
        "[N] " + t("trex.night") + ": " + onoff,
        "[C] " + t("trex.skin") + ": " + (this.skin + 1),
      ].join("  ·  ");
      ui.text(ctx, opts, cx, I(this.height * 0.52), this.tiny, this.pal("dim"), "center");
    }

    drawOver(ctx) {
      const cx = this.width / 2;
      const cy = I(this.height * 0.34);
      ui.text(ctx, t("common.game_over"), cx, cy, this.big, this.pal("text"), "center");
      if (this.score >= this.best && this.score > 0) {
        ui.text(ctx, t("trex.new_record"), cx, cy + I(30 * this.scale), this.small, ui.GOLD, "center");
      }
      if (I(this.deadT * 2) % 2 === 0) {
        ui.text(ctx, t("common.enter_restart"), cx, cy + I(58 * this.scale), this.small, this.pal("dim"), "center");
      }
    }
  }

  PG.register(TRexRunnerGame, {
    id: "TRexRunnerGame",
    key: "trex",
    name: "T-Rex",
    settingsKey: "trex",
    defaults: { difficulty: "normal", skin: 1, night: true },
  });
})();
