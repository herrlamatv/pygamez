/*
 * flappy.js - Flappy Bird - Klon mit vielen Extras (Port von games/flappy.py)
 * ===========================================================================
 * - Schwerkraft-Physik: Leertaste / Pfeil hoch / W / Mausklick lässt den Vogel
 *   flattern; der Vogel neigt sich je nach Steig-/Sinktempo (Rotation).
 * - Endlose Röhrenpaare mit Lücke; +1 Punkt pro passierter Röhre.
 * - MÜNZEN (Bonuspunkte) und SCHILD-Power-up (überlebt eine Kollision) erscheinen
 *   gelegentlich in den Lücken.
 * - TAG/NACHT-THEMEN: Der Himmel wechselt mit steigender Punktzahl (Farbpaletten),
 *   mit driftenden Wolken (Parallax) und scrollendem Boden.
 * - Schwierigkeit (Leicht/Normal/Schwer): Lückengröße, Tempo, Röhrenabstand;
 *   die Lücke wird mit steigender Punktzahl etwas enger.
 * - MEDAILLEN nach dem Game Over (Bronze/Silber/Gold/Platin) je nach Punktzahl.
 * - READY-Screen (Vogel wippt), Crash-Animation mit Kamera-Shake, Partikel.
 *
 * Steuerung: Leertaste / Pfeil hoch / W / Klick = Flattern.  Enter = neu, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const GRAV = 1550.0;
  const FLAP_V = -430.0;
  const BIRD_X_FRAC = 0.30;

  const PIPE_W_FRAC = 0.13; // Röhrenbreite als Anteil der Breite
  const CAP_H_FRAC = 0.028; // Höhe der Röhren-"Kappe"

  // Schwierigkeit: Lücke (px-Anteil Höhe), Tempo (px/s), Röhrenabstand (px)
  const DIFFS = [
    { key: "easy", gap: 0.30, speed: 150.0, spacing: 0.62 },
    { key: "normal", gap: 0.25, speed: 190.0, spacing: 0.55 },
    { key: "hard", gap: 0.21, speed: 230.0, spacing: 0.48 },
  ];

  // Tag/Nacht-Paletten: (sky_top, sky_bottom, pipe, pipe_dark, ground, ground_dark)
  // Identitätsfarben des Spielfelds - bewusst NICHT aus dem UI-Theme.
  const THEMES = [
    [[120, 200, 245], [200, 235, 250], [110, 205, 90], [70, 150, 60], [222, 200, 120], [180, 155, 90]],
    [[250, 190, 120], [255, 225, 170], [230, 160, 70], [180, 120, 50], [210, 180, 120], [170, 140, 90]],
    [[40, 45, 90], [80, 90, 150], [90, 120, 200], [60, 80, 150], [60, 60, 100], [40, 40, 75]],
    [[150, 120, 200], [220, 190, 235], [200, 120, 190], [150, 80, 140], [200, 170, 200], [160, 130, 165]],
  ];

  const MEDALS = [
    [70, "platin", [225, 235, 245]], [45, "gold", [245, 205, 70]],
    [25, "silber", [200, 205, 215]], [10, "bronze", [205, 140, 85]],
  ];

  // Weitere Identitätsfarben (Spielobjekte, nicht Theme-abhängig)
  const COL_COIN = [250, 205, 70]; // Münzen-Gold
  const COL_SHIELD = [120, 220, 255]; // Schild-Blau

  const SETUP = "setup", READY = "ready", PLAY = "play", DYING = "dying", GAMEOVER = "gameover";

  const circleRect = (cx, cy, r, rx, ry, rw, rh) => {
    const nx = Math.max(rx, Math.min(cx, rx + rw));
    const ny = Math.max(ry, Math.min(cy, ry + rh));
    return (cx - nx) ** 2 + (cy - ny) ** 2 < r * r;
  };

  class FlappyGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.diff = Math.max(0, Math.min(2, parseInt(this.opts.difficulty, 10) || 0));
      this.best = this.highscore;
      this.animT = 0.0;
      this.skyCache = null;
      this.applyLayout();
      this.clouds = [];
      for (let i = 0; i < 5; i++) this.clouds.push(this.newCloud(PG.rand.uniform(0, this.width)));
      this.newGame();
      this.state = SETUP;
    }

    makeFonts() {
      // Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schrift)
      const h = this.height;
      this.font = ui.font(Math.max(18, Math.min(26, Math.floor(h / 26))));
      this.bigFont = ui.font(Math.max(32, Math.min(52, Math.floor(h / 12))), true);
      this.small = ui.font(Math.max(14, Math.min(20, Math.floor(h / 32))));
      this.tiny = ui.font(Math.max(12, Math.min(16, Math.floor(h / 42))));
      this.huge = ui.font(Math.max(30, Math.floor(h / 9)), true);
    }

    applyLayout() {
      // Layout-Größen aus width/height berechnen
      this.makeFonts();
      this.groundH = Math.floor(this.height * 0.12);
      this.birdX = this.width * BIRD_X_FRAC;
      this.birdR = Math.max(9, Math.floor(this.height * 0.028));
      this.pipeW = Math.floor(this.width * PIPE_W_FRAC);
      this.buildSetupLayout();
    }

    newCloud(x) {
      return {
        x: x == null ? this.width : x,
        y: PG.rand.uniform(this.height * 0.08, this.height * 0.5),
        s: PG.rand.uniform(0.6, 1.4),
        v: PG.rand.uniform(12, 30),
      };
    }

    newGame() {
      this.score = 0;
      this.coins = 0;
      this.gameOver = false;
      this._resultReported = false;
      this.birdY = this.height * 0.45;
      this.birdV = 0.0;
      this.birdA = 0.0;
      this.pipes = [];
      this.pickups = []; // {x, y, kind('coin'/'shield'), taken}
      this.particles = [];
      this.shield = false;
      this.shake = 0.0;
      this.groundX = 0.0;
      this.dist = 0.0;
      this.wingT = 0.0;
      this.newBest = false;
      this.overT = 0.0;
      this.state = READY;
    }

    get theme() {
      return THEMES[Math.floor(this.score / 10) % THEMES.length];
    }

    diffCfg() {
      return DIFFS[this.diff];
    }

    gap() {
      const base = this.diffCfg().gap * this.height;
      return Math.max(this.birdR * 4.5, base - this.score * 1.2);
    }

    speed() {
      return this.diffCfg().speed + this.score * 1.5;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(440, this.width - 60);
      const ph = Math.max(56, Math.min(72, Math.floor(this.height * 0.14)));
      const y0 = Math.max(140, Math.floor(this.height * 0.34));
      this.diffPanel = new PG.Rect(cx - Math.floor(bw / 2), y0, bw, ph);
      this.diffLeft = new PG.Rect(this.diffPanel.left, y0, 44, ph);
      this.diffRight = new PG.Rect(this.diffPanel.right - 44, y0, 44, ph);
      this.startRect = new PG.Rect(cx - 95, this.diffPanel.bottom + 26, 190, 52);
    }

    cycleDiff(step) {
      this.diff = PG.mod(this.diff + step, DIFFS.length);
      this.opts.difficulty = this.diff;
      this.saveSettings();
      this.playSound("click");
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        if (["Left", "a", "A"].includes(ev.key)) this.cycleDiff(-1);
        else if (["Right", "d", "D"].includes(ev.key)) this.cycleDiff(+1);
        else if (ev.key === "Return" || ev.key === "space") {
          this.newGame();
          this.playSound("click");
        }
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        if (this.diffLeft.collidepoint(p)) this.cycleDiff(-1);
        else if (this.diffRight.collidepoint(p) || this.diffPanel.collidepoint(p)) this.cycleDiff(+1);
        else if (this.startRect.collidepoint(p)) {
          this.newGame();
          this.playSound("click");
        }
      }
    }

    // ===================================================== Eingabe
    isFlap(key) {
      return this.isAction(key, "up") || ["Up", "space", "w", "W"].includes(key);
    }

    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === GAMEOVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.newGame();
            this.playSound("click");
          } else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ui.now() - this.overT > 0.5) {
          // Klick startet neu - aber erst nach kurzer Sperre, damit
          // hektisches Weiterklicken den Ergebnis-Screen nicht überspringt.
          // (Echtzeit statt anim_t, da update() im Game Over ruht.)
          this.newGame();
          this.playSound("click");
        }
        return;
      }
      const flap = ev.kind === "mousedown" || (ev.kind === "keydown" && this.isFlap(ev.key));
      if (flap) {
        if (this.state === READY) this.state = PLAY;
        if (this.state === PLAY) this.flap();
      }
    }

    flap() {
      this.birdV = FLAP_V;
      this.wingT = 0.0;
      this.playSound("move");
      for (let i = 0; i < 4; i++) {
        this.particles.push([this.birdX - this.birdR, this.birdY, PG.rand.uniform(-60, -20), PG.rand.uniform(-20, 40), 0.35, [255, 255, 255]]);
      }
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.animT += dt;
      this.wingT += dt;
      this.updateClouds(dt);
      this.updateParticles(dt);
      if (this.shake > 0) this.shake = Math.max(0.0, this.shake - dt * 2.0);
      if (this.state === SETUP || this.state === GAMEOVER) return;
      if (this.state === READY) {
        this.birdY = this.height * 0.45 + Math.sin(this.animT * 3) * 12;
        return;
      }
      if (this.state === DYING) {
        this.birdV += GRAV * dt;
        this.birdY += this.birdV * dt;
        this.birdA = Math.max(-1.4, this.birdA - dt * 4);
        if (this.birdY >= this.height - this.groundH - this.birdR) {
          this.birdY = this.height - this.groundH - this.birdR;
          this.state = GAMEOVER;
          this.finish();
        }
        return;
      }

      // PLAY
      this.dist += this.speed() * dt;
      this.groundX = PG.mod(this.groundX - this.speed() * dt, this.width);
      this.birdV += GRAV * dt;
      this.birdY += this.birdV * dt;
      this.birdA = Math.max(-1.3, Math.min(1.4, this.birdV / 520.0));
      if (this.birdY < this.birdR) {
        // Decke: abprallen
        this.birdY = this.birdR;
        this.birdV = Math.max(this.birdV, 40.0);
      }
      this.spawnPipes();
      this.movePipes(dt);
      this.collisions();
    }

    spawnPipes() {
      const spacing = this.diffCfg().spacing * this.width;
      if (!this.pipes.length || this.pipes[this.pipes.length - 1].x < this.width - spacing) {
        const gap = this.gap();
        const top = this.height - this.groundH;
        const margin = this.height * 0.09;
        const gy = PG.rand.uniform(margin + gap / 2, top - margin - gap / 2);
        const pipe = { x: this.width + this.pipeW, gy, gap, passed: false };
        this.pipes.push(pipe);
        // gelegentlich Münze oder Schild in die Lücke
        const n = this.pipes.length;
        if (n % 3 === 0) {
          this.pickups.push({ x: pipe.x + this.pipeW / 2, y: gy, kind: "coin", taken: false });
        } else if (n % 7 === 0 && !this.shield) {
          this.pickups.push({ x: pipe.x + this.pipeW / 2, y: gy, kind: "shield", taken: false });
        }
      }
    }

    movePipes(dt) {
      const v = this.speed() * dt;
      for (const p of this.pipes) {
        p.x -= v;
        if (!p.passed && p.x + this.pipeW / 2 < this.birdX) {
          p.passed = true;
          this.score += 1;
          this.playSound("point");
        }
      }
      this.pipes = this.pipes.filter((p) => p.x + this.pipeW > -4);
      for (const pk of this.pickups) pk.x -= v;
      this.pickups = this.pickups.filter((pk) => pk.x > -20 && !pk.taken);
    }

    collisions() {
      const bx = this.birdX, by = this.birdY, r = this.birdR;
      // Boden
      if (by + r >= this.height - this.groundH) {
        this.hit();
        return;
      }
      // Röhren (Rects wie pygame.Rect: ganzzahlig)
      for (const p of this.pipes) {
        const x = Math.trunc(p.x);
        const topH = Math.trunc(p.gy - p.gap / 2);
        const botY = Math.trunc(p.gy + p.gap / 2);
        if (circleRect(bx, by, r, x, 0, this.pipeW, topH) || circleRect(bx, by, r, x, botY, this.pipeW, this.height)) {
          this.hit();
          return;
        }
      }
      // Pickups
      for (const pk of this.pickups) {
        if (!pk.taken && Math.hypot(bx - pk.x, by - pk.y) < r + 12) {
          pk.taken = true;
          if (pk.kind === "coin") {
            this.coins += 1;
            this.score += 2;
            this.playSound("eat");
          } else {
            this.shield = true;
            this.playSound("powerup");
          }
          this.sparkle(pk.x, pk.y, COL_COIN);
        }
      }
    }

    hit() {
      if (this.shield) {
        this.shield = false;
        this.birdV = FLAP_V * 0.8;
        this.shake = 0.4;
        this.playSound("hit");
        this.sparkle(this.birdX, this.birdY, COL_SHIELD);
        return;
      }
      this.state = DYING;
      this.birdV = FLAP_V * 0.5;
      this.shake = 0.7;
      this.playSound("explode");
      this.rumble(250);
      this.sparkle(this.birdX, this.birdY, [255, 200, 90], 16);
    }

    finish() {
      this.gameOver = true;
      this.newBest = this.score > this.best;
      this.best = Math.max(this.best, this.score);
      this.overT = ui.now();
      this.playSound("gameover");
    }

    medal() {
      for (const [thr, key, col] of MEDALS) {
        if (this.score >= thr) return [key, col];
      }
      return [null, null];
    }

    // ----- Effekte -------------------------------------------------------
    sparkle(x, y, col, n = 10) {
      for (let i = 0; i < n; i++) {
        const a = PG.rand.uniform(0, PG.TAU);
        const sp = PG.rand.uniform(40, 180);
        this.particles.push([x, y, Math.cos(a) * sp, Math.sin(a) * sp, PG.rand.uniform(0.25, 0.55), col]);
      }
    }

    updateParticles(dt) {
      const rest = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] += 300 * dt;
        p[4] -= dt;
        if (p[4] > 0) rest.push(p);
      }
      this.particles = rest;
    }

    updateClouds(dt) {
      const f = [SETUP, READY, GAMEOVER].includes(this.state) ? 0.4 : 1.0;
      for (const c of this.clouds) {
        c.x -= c.v * c.s * f * dt;
        if (c.x < -80) {
          Object.assign(c, this.newCloud());
          c.x = this.width + 60;
        }
      }
    }

    // ----- Theme-UI-Helfer ----------------------------------------------
    panel(ctx, rect, alpha = 235, border = 2) {
      // Halbtransparentes Theme-Panel mit Akzent-Rahmen
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], alpha], rect, 0, 12);
      draw.rect(ctx, this.accent, rect, border, 12);
    }

    chip(ctx, text, font, center, color) {
      // Kleine Info-Plakette im Theme-Stil (über dem Spielfeld lesbar)
      const [w, h] = font.size(text);
      const box = new PG.Rect(0, 0, w, h);
      box.center = center;
      box.inflateIp(24, 12);
      this.panel(ctx, box, 200, 1);
      ui.text(ctx, text, box.centerx, box.centery, font, color || ui.TEXT, "center");
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      let ox = 0, oy = 0;
      if (this.shake > 0) {
        const amp = 8 * this.shake;
        ox = PG.rand.uniform(-amp, amp);
        oy = PG.rand.uniform(-amp, amp);
      }
      this.drawSky(ctx);
      this.drawClouds(ctx, ox * 0.3, oy * 0.3);
      for (const p of this.pipes) this.drawPipe(ctx, p, ox, oy);
      for (const pk of this.pickups) this.drawPickup(ctx, pk, ox, oy);
      this.drawGround(ctx, ox);
      for (const p of this.particles) {
        const a = Math.max(0, Math.min(255, Math.floor((255 * p[4]) / 0.55)));
        draw.circle(ctx, [p[5][0], p[5][1], p[5][2], a], [p[0] + ox, p[1] + oy], 2);
      }
      this.drawBird(ctx, ox, oy);
      this.drawHud(ctx);
    }

    drawSky(ctx) {
      const th = this.theme;
      const key = th[0].join() + "|" + th[1].join();
      if (!this.skyCache || this.skyCache.key !== key) {
        // Verlauf einmal in eine Offscreen-Fläche rendern
        const c = ui.makeCanvas(4, this.height);
        const g = c.getContext("2d");
        const lg = g.createLinearGradient(0, 0, 0, this.height);
        lg.addColorStop(0, ui.col(th[0]));
        lg.addColorStop(1, ui.col(th[1]));
        g.fillStyle = lg;
        g.fillRect(0, 0, 4, this.height);
        this.skyCache = { key, canvas: c };
      }
      ctx.drawImage(this.skyCache.canvas, 0, 0, this.width, this.height);
    }

    drawClouds(ctx, ox, oy) {
      for (const c of this.clouds) {
        const x = c.x + ox, y = c.y + oy, sc = c.s;
        for (const [dx, dy, rr] of [[0, 0, 26], [22, 6, 20], [-22, 6, 20], [0, 10, 22]]) {
          draw.circle(ctx, [255, 255, 255, 150], [x + dx * sc, y + dy * sc], Math.floor(rr * sc));
        }
      }
    }

    drawPipe(ctx, p, ox, oy) {
      const pipe = this.theme[2], dark = this.theme[3];
      const w = this.pipeW;
      const cap = Math.floor(this.height * CAP_H_FRAC) + 4;
      const x = p.x + ox;
      const topH = p.gy - p.gap / 2;
      const botY = p.gy + p.gap / 2;
      // oberes Rohr
      draw.rect(ctx, pipe, [x, oy, w, topH - cap]);
      draw.rect(ctx, pipe, [x - 4, topH - cap + oy, w + 8, cap], 0, 4);
      draw.rect(ctx, dark, [x + w - 8, oy, 6, topH]);
      // unteres Rohr
      const gh = this.height - this.groundH - botY;
      draw.rect(ctx, pipe, [x, botY + cap + oy, w, gh]);
      draw.rect(ctx, pipe, [x - 4, botY + oy, w + 8, cap], 0, 4);
      draw.rect(ctx, dark, [x + w - 8, botY + oy, 6, gh + cap]);
      draw.rect(ctx, [255, 255, 255], [x + 4, oy, 4, topH - cap]);
    }

    drawPickup(ctx, pk, ox, oy) {
      const x = Math.floor(pk.x + ox), y = Math.floor(pk.y + oy);
      if (pk.kind === "coin") {
        const w = Math.trunc(10 + 5 * Math.sin(this.animT * 6));
        draw.ellipse(ctx, COL_COIN, [x - w, y - 12, 2 * w, 24]);
        draw.ellipse(ctx, [255, 240, 160], [x - w + 2, y - 9, 2 * w - 4, 18], 2);
      } else {
        draw.circle(ctx, [COL_SHIELD[0], COL_SHIELD[1], COL_SHIELD[2], 90], [x, y], 17);
        draw.circle(ctx, COL_SHIELD, [x, y], 17, 2);
        ui.text(ctx, "S", x, y, this.tiny, [240, 250, 255], "center");
      }
    }

    drawGround(ctx, ox) {
      const gcol = this.theme[4], gdark = this.theme[5];
      const gy = this.height - this.groundH;
      draw.rect(ctx, gcol, [0, gy, this.width, this.groundH]);
      draw.rect(ctx, gdark, [0, gy, this.width, 5]);
      const step = 26;
      const off = PG.mod(Math.floor(this.groundX), step);
      for (let x = -step; x < this.width + step; x += step) {
        draw.line(ctx, gdark, [x - off, gy + 8], [x - off + 12, this.height], 2);
      }
    }

    drawBird(ctx, ox, oy) {
      const r = this.birdR;
      const wing = this.state === PLAY || this.state === READY ? Math.sin(Math.min(this.wingT, 0.3) * 30) * r * 0.5 : 0;
      ctx.save();
      ctx.translate(this.birdX + ox, this.birdY + oy);
      // rotozoom(-deg(a)) dreht bei a > 0 im Uhrzeigersinn = Canvas-rotate(+a)
      ctx.rotate(this.birdA);
      this.drawBirdBody(ctx, r, wing);
      ctx.restore();
      if (this.shield) {
        const c = [this.birdX + ox, this.birdY + oy];
        const rr = Math.floor(r * 1.9);
        draw.circle(ctx, [COL_SHIELD[0], COL_SHIELD[1], COL_SHIELD[2], 70], c, rr);
        draw.circle(ctx, [150, 235, 255], c, rr, 2);
      }
    }

    drawBirdBody(ctx, r, wing) {
      // Vogel um (0, 0) zeichnen (entspricht _bird_surface)
      const body = [250, 205, 60];
      draw.polygon(ctx, [235, 180, 50], [[-r, 0], [-r * 1.5, -r * 0.5], [-r * 1.5, r * 0.5]]);
      draw.ellipse(ctx, body, [-r, -r * 0.85, r * 2, r * 1.7]);
      draw.ellipse(ctx, [255, 240, 200], [-r * 0.4, -r * 0.1, r * 1.1, r * 0.9]);
      draw.ellipse(ctx, [230, 180, 60], [-r * 0.35, -r * 0.15 + wing, r * 0.9, r * 0.6]);
      draw.circle(ctx, [255, 255, 255], [r * 0.45, -r * 0.32], Math.floor(r * 0.32));
      draw.circle(ctx, [25, 25, 35], [r * 0.55, -r * 0.32], Math.floor(r * 0.15));
      draw.polygon(ctx, [250, 150, 40], [[r * 0.75, -r * 0.1], [r * 1.35, 0], [r * 0.75, r * 0.25]]);
    }

    // ----- HUD / Overlays -----------------------------------------------
    drawHud(ctx) {
      if (this.state === PLAY || this.state === DYING) {
        // Punktzahl mit Kontur - bleibt auf jedem Himmel lesbar
        const s = String(this.score);
        const cx = Math.floor(this.width / 2), cy = Math.floor(this.height * 0.14);
        for (const [dx, dy] of [[-2, 0], [2, 0], [0, -2], [0, 2]]) {
          ui.text(ctx, s, cx + dx, cy + dy, this.huge, [30, 30, 40], "center");
        }
        ui.text(ctx, s, cx, cy, this.huge, [245, 245, 250], "center");
      }
      if (this.state === READY) {
        this.chip(ctx, t("fb.best", { hs: this.best }), this.small, [Math.floor(this.width / 2), Math.floor(this.height * 0.1)], ui.TEXT_DIM);
        const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
        this.chip(ctx, t("fb.tap_hint"), this.small, [Math.floor(this.width / 2), Math.floor(this.height * 0.38)], hintCol);
      }
      if (this.state === GAMEOVER) this.drawGameover(ctx);
    }

    drawGameover(ctx) {
      draw.rect(ctx, [10, 12, 20, 140], [0, 0, this.width, this.height]);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("common.game_over"), cx, Math.floor(this.height * 0.18), this.huge, this.accent, "center");

      const lh = this.small.height + 8;
      const pw = Math.min(360, this.width - 40);
      const ph = lh * 3 + 40;
      const panel = new PG.Rect(cx - Math.floor(pw / 2), Math.floor(this.height * 0.32), pw, ph);
      this.panel(ctx, panel);
      const x = panel.x + 20;
      let y = panel.y + 18;
      ui.text(ctx, t("common.points", { score: this.score }), x, y, this.small, ui.TEXT);
      y += lh;
      ui.text(ctx, t("fb.coins", { n: this.coins }), x, y, this.small, ui.GOLD);
      y += lh;
      const bestCol = this.newBest ? ui.GOLD : ui.TEXT_DIM;
      ui.text(ctx, t("fb.best", { hs: this.best }), x, y, this.small, bestCol);

      // Medaille rechts im Panel (Identitätsfarben)
      const [key, col] = this.medal();
      if (key) {
        const mr = Math.max(24, Math.min(36, Math.floor(panel.h / 4)));
        const mcx = panel.right - mr - 24, mcy = panel.centery - 8;
        draw.circle(ctx, col, [mcx, mcy], mr);
        draw.circle(ctx, col.map((c) => Math.floor(c * 0.7)), [mcx, mcy], mr, 3);
        this.star(ctx, mcx, mcy, Math.floor(mr * 0.55), [255, 255, 255]);
        ui.text(ctx, t("fb.medal." + key), mcx, mcy + mr + 12, this.tiny, col, "center");
      }

      if (this.newBest) {
        const recCol = ui.mix(ui.GOLD, ui.TEXT, ui.pulse(3.0, 0.0, 0.5));
        ui.text(ctx, t("trex.new_record"), cx, panel.top - 18, this.small, recCol, "center");
      }

      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
      ui.text(ctx, t("fb.restart_hint"), cx, panel.bottom + 26, this.small, hintCol, "center");
    }

    star(ctx, cx, cy, r, col) {
      const pts = [];
      for (let i = 0; i < 10; i++) {
        const rr = i % 2 === 0 ? r : r * 0.45;
        const a = -Math.PI / 2 + (i * Math.PI) / 5;
        pts.push([cx + Math.cos(a) * rr, cy + Math.sin(a) * rr]);
      }
      draw.polygon(ctx, col, pts);
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      this.drawClouds(ctx, 0, 0);
      ui.drawTitle(ctx, this.width, "FLAPPY BIRD", { subtitle: t("snake.singleplayer"), accent: this.accent });

      const d = DIFFS[this.diff];
      const dp = this.diffPanel;
      this.panel(ctx, dp);
      ui.text(ctx, t("fb.difficulty") + ":  " + t("fb.diff." + d.key), dp.centerx, dp.top + Math.floor(dp.h * 0.36), this.font, ui.TEXT, "center");
      ui.text(ctx, t("fb.diff_note"), dp.centerx, dp.top + Math.floor(dp.h * 0.72), this.tiny, ui.TEXT_DIM, "center");
      const arrCol = ui.mix(this.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3));
      for (const [rect, sym] of [[this.diffLeft, "<"], [this.diffRight, ">"]]) {
        ui.text(ctx, sym, rect.centerx, rect.centery, this.bigFont, arrCol, "center");
      }

      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: this.accent });

      ui.text(ctx, t("fb.controls_hint"), Math.floor(this.width / 2), this.startRect.bottom + 24, this.tiny, ui.GREEN, "center");
      ui.drawFooter(ctx, this.width, this.height, t("fb.setup_hint"));
    }
  }

  PG.register(FlappyGame, {
    id: "FlappyGame",
    key: "flappy",
    name: "Flappy Bird",
    settingsKey: "flappy",
    defaults: { difficulty: 1 },
  });
})();
