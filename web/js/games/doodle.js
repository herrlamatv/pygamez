/*
 * doodle.js - Doodle Jump - Klon mit vielen Extras (Port von games/doodle.py)
 * ===========================================================================
 * - Der Doodler springt automatisch, sobald er auf einer Plattform landet;
 *   gesteuert wird nur links/rechts (mit Trägheit), die Ränder sind offen
 *   (Wrap-around). Die Kamera scrollt mit dem Aufstieg nach oben.
 * - PLATTFORM-TYPEN: grün normal, blau beweglich, braun zerbricht,
 *   weiß verschwindet nach einem Sprung.
 * - SPRUNGFEDERN geben einen Superhüpfer; der PROPELLER-HUT trägt einen kurz
 *   automatisch nach oben (und macht unverwundbar).
 * - MONSTER: Berührung ist tödlich - aber man kann sie mit Pfeil hoch / Leertaste
 *   von unten ABSCHIESSEN (Extrapunkte).
 * - Schwierigkeit (Leicht/Normal/Schwer): Plattform-Abstand und Anteil an
 *   beweglichen/zerbrechlichen Plattformen sowie Monstern; wird mit der Höhe härter.
 * - Punkte = erreichte Höhe. Partikel, Feder-Animation, Highscore.
 *
 * Steuerung: links/rechts (A/D oder Pfeile) = bewegen, Pfeil hoch / Leertaste = schießen.
 * Enter = neu, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const Rect = PG.Rect;
  const fl = Math.floor;
  const rand = PG.rand;

  const GRAV = 1350.0;
  const JUMP_V = -720.0; // normaler Absprung
  const SPRING_V = -1250.0; // Sprungfeder
  const PROP_V = -560.0; // Propeller-Steiggeschwindigkeit
  const MOVE_ACC = 1500.0;
  const MOVE_MAX = 430.0;
  const MOVE_FRICTION = 0.86;
  const SHOOT_CD = 0.28;
  const PROP_TIME = 2.6;

  // Schwierigkeit: Basis-Plattformabstand (px), Anteil beweglich/zerbrechlich, Monster-Chance
  const DIFFS = [
    { key: "easy", gap: 62, move: 0.12, brittle: 0.1, monster: 0.05 },
    { key: "normal", gap: 74, move: 0.2, brittle: 0.18, monster: 0.09 },
    { key: "hard", gap: 86, move: 0.28, brittle: 0.24, monster: 0.14 },
  ];

  // Identitätsfarben des Spielfelds (Papier-Look) - bewusst NICHT aus dem UI-Theme.
  const COL_BG_TOP = [222, 236, 250];
  const COL_BG_BOT = [238, 246, 252];
  const COL_GRID = [220, 228, 240];
  const COL_DOODLE = [120, 210, 90];
  const COL_DOODLE_DARK = [70, 150, 55];

  const PLAT_COLORS = {
    normal: [110, 205, 90], move: [90, 160, 235],
    brittle: [190, 130, 80], vanish: [235, 235, 240],
  };
  const MONSTER_COLS = [[230, 110, 130], [150, 120, 220], [235, 150, 80]];

  const SETUP = "setup", PLAY = "play", GAMEOVER = "gameover";

  const dark = (col) => col.map((c) => Math.trunc(c * 0.7));

  function makePlat(x, y, w, kind) {
    return { x, y, w, kind, vx: 0, spring: false, prop: false, dead: false, used: false };
  }

  class DoodleGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.diff = Math.max(0, Math.min(2, Math.trunc(Number(this.opts.difficulty))));
      if (!Number.isFinite(this.diff)) this.diff = 1;

      this.best = this.highscore;
      this.animT = 0;
      this.lastDrawT = null;

      this.bgCache = null;
      this.applyLayout();

      this.newGame();
      this.state = SETUP;
    }

    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(18, Math.min(26, fl(h / 26))));
      this.bigFont = ui.font(Math.max(32, Math.min(52, fl(h / 12))), true);
      this.small = ui.font(Math.max(14, Math.min(20, fl(h / 32))));
      this.tiny = ui.font(Math.max(12, Math.min(16, fl(h / 42))));
      this.huge = ui.font(Math.max(26, fl(h / 11)), true);
    }

    /** Layout-Größen aus width/height berechnen. */
    applyLayout() {
      this.makeFonts();
      this.platW = Math.max(46, Math.trunc(this.width * 0.16));
      this.dr = Math.max(12, Math.trunc(this.height * 0.03));
      this.buildSetupLayout();
    }

    newGame() {
      this.score = 0;
      this.gameOver = false;
      this.camY = 0;
      this.x = this.width / 2;
      this.y = this.height * 0.7;
      this.pyPrev = this.y;
      this.vx = 0;
      this.vy = JUMP_V;
      this.face = 1;
      this.shootCd = 0;
      this.shootPose = 0;
      this.propT = 0;
      this.startY = this.y;
      this.maxRise = 0;
      this.press = new Set();
      this.platforms = [];
      this.monsters = []; // {x, y, vx, hp, kind, bob}
      this.bullets = []; // {x, y, vy}
      this.particles = [];
      this.newBest = false;
      this.overT = 0;
      this.prevKind = "";
      // Startplattform + erste Plattformen aufbauen
      this.platforms.push(makePlat(this.width / 2 - this.platW / 2, this.height * 0.75, this.platW, "normal"));
      this.genY = this.height * 0.75;
      this.fillPlatforms();
      this.state = PLAY;
    }

    difficultyScale() {
      // steigt mit der Höhe (0..1): Plattformen weiter auseinander, mehr Hazards
      return Math.min(1.0, this.score / 4000.0);
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = fl(this.width / 2);
      const bw = Math.min(440, this.width - 60);
      const ph = Math.max(56, Math.min(72, Math.trunc(this.height * 0.14)));
      const y0 = Math.max(140, Math.trunc(this.height * 0.34));
      this.diffPanel = new Rect(cx - fl(bw / 2), y0, bw, ph);
      this.diffLeft = new Rect(this.diffPanel.left, y0, 44, ph);
      this.diffRight = new Rect(this.diffPanel.right - 44, y0, 44, ph);
      this.startRect = new Rect(cx - 95, this.diffPanel.bottom + 26, 190, 52);
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
    isLeft(key) {
      return this.isAction(key, "left") || key === "Left";
    }

    isRight(key) {
      return this.isAction(key, "right") || key === "Right";
    }

    isShoot(key) {
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
            // Web: gameOver zurücksetzen, sonst liegt das Highscore-Banner über dem Setup.
            this.gameOver = false;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ui.now() - this.overT > 0.5) {
          // Klick startet neu - mit kurzer Sperre gegen Durchklicken
          this.newGame();
          this.playSound("click");
        }
        return;
      }
      if (ev.kind === "keydown") {
        if (this.isLeft(ev.key)) this.press.add("l");
        else if (this.isRight(ev.key)) this.press.add("r");
        else if (this.isShoot(ev.key)) this.shoot();
      } else if (ev.kind === "keyup") {
        if (this.isLeft(ev.key)) this.press.delete("l");
        else if (this.isRight(ev.key)) this.press.delete("r");
      }
    }

    shoot() {
      if (this.shootCd > 0) return;
      this.shootCd = SHOOT_CD;
      this.shootPose = 0.25;
      this.bullets.push({ x: this.x, y: this.y - this.dr, vy: -820.0 });
      this.playSound("shoot");
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.animT += dt;
      this.updateParticles(dt);
      if (this.state !== PLAY) return;
      this.shootCd = Math.max(0, this.shootCd - dt);
      this.shootPose = Math.max(0, this.shootPose - dt);

      // Horizontale Steuerung
      if (this.press.has("l")) {
        this.vx -= MOVE_ACC * dt;
        this.face = -1;
      }
      if (this.press.has("r")) {
        this.vx += MOVE_ACC * dt;
        this.face = 1;
      }
      if (!this.press.size) this.vx *= MOVE_FRICTION;
      this.vx = Math.max(-MOVE_MAX, Math.min(MOVE_MAX, this.vx));
      this.x += this.vx * dt;
      if (this.x < 0) this.x += this.width; // Wrap-around
      else if (this.x > this.width) this.x -= this.width;

      // Propeller trägt nach oben
      this.pyPrev = this.y; // für die "swept"-Kollision merken
      if (this.propT > 0) {
        this.propT -= dt;
        this.vy = PROP_V;
      } else {
        this.vy += GRAV * dt;
      }
      this.y += this.vy * dt;

      if (this.vy > 0) this.platformCollisions(); // nur beim Fallen auf Plattformen prüfen

      this.updatePlatforms(dt);
      this.updateMonsters(dt);
      if (this.state !== PLAY) return;
      this.updateBullets(dt);
      this.cameraAndScore();
      this.fillPlatforms();
      this.cull();

      // Tod: unter den Bildschirm gefallen
      if (this.y - this.camY > this.height + this.dr) this.die();
    }

    /**
     * Swept-Kollision: prüft, ob die Füße die Plattform-Oberkante seit dem
     * letzten Frame ÜBERQUERT haben - so tunnelt der Doodler auch bei hohem
     * Tempo (Feder!) nicht hindurch.
     */
    platformCollisions() {
      const prevFeet = this.pyPrev + this.dr;
      const feet = this.y + this.dr;
      for (const p of this.platforms) {
        if (p.dead) continue;
        if (!(p.x <= this.x && this.x <= p.x + p.w)) continue;
        // Oberkante der Plattform lag zwischen altem und neuem Fußpunkt?
        if (!(prevFeet <= p.y + 6 && feet >= p.y - 2)) continue;
        if (p.kind === "brittle") {
          // zerbricht -> kein Absprung, weiterfallen
          p.dead = true;
          this.sparkle(this.x, p.y, PLAT_COLORS.brittle);
          this.playSound("hit");
          continue;
        }
        this.y = p.y - this.dr; // auf die Plattform setzen (kein Re-Trigger)
        if (p.prop) {
          this.propT = PROP_TIME;
          p.prop = false;
          this.playSound("powerup");
          this.sparkle(this.x, p.y, [120, 200, 255]);
        } else if (p.spring) {
          this.vy = SPRING_V;
          this.playSound("powerup");
          this.sparkle(this.x, p.y, [255, 220, 120]);
        } else {
          this.vy = JUMP_V;
          this.playSound("bounce");
        }
        if (p.kind === "vanish" && !p.used) {
          p.used = true;
          p.dead = true;
        }
        break;
      }
    }

    updatePlatforms(dt) {
      for (const p of this.platforms) {
        if (p.kind === "move" && !p.dead) {
          p.x += p.vx * dt;
          if (p.x < 0 || p.x + p.w > this.width) {
            p.vx = -p.vx;
            p.x = Math.max(0, Math.min(this.width - p.w, p.x));
          }
        }
      }
    }

    updateMonsters(dt) {
      for (const m of this.monsters) {
        m.x += m.vx * dt;
        if (m.x < 20 || m.x > this.width - 20) m.vx = -m.vx;
        // Kollision mit Doodler
        const nah = Math.hypot(this.x - m.x, this.y - m.y) < this.dr + 16;
        if (this.propT <= 0 && nah) {
          this.die();
          return;
        } else if (this.propT > 0 && nah) {
          m.hp = 0;
          this.score += 200;
          this.sparkle(m.x, m.y, [255, 120, 120]);
          this.playSound("explode");
        }
      }
      this.monsters = this.monsters.filter((m) => m.hp > 0);
    }

    updateBullets(dt) {
      for (const b of this.bullets) {
        b.y += b.vy * dt;
        for (const m of this.monsters) {
          if (m.hp > 0 && Math.hypot(b.x - m.x, b.y - m.y) < 18) {
            m.hp = 0;
            b.y = -99999;
            this.score += 200;
            this.sparkle(m.x, m.y, [255, 120, 120]);
            this.playSound("explode");
          }
        }
      }
      this.bullets = this.bullets.filter((b) => b.y - this.camY > -30);
      this.monsters = this.monsters.filter((m) => m.hp > 0);
    }

    cameraAndScore() {
      const thresh = this.camY + this.height * 0.42;
      if (this.y < thresh) this.camY = this.y - this.height * 0.42;
      const rise = this.startY - (this.camY + this.height * 0.42);
      this.maxRise = Math.max(this.maxRise, rise);
      this.score = Math.max(this.score, Math.trunc(this.maxRise / 4));
    }

    /**
     * Erzeugt Plattformen nach oben, bis genug über der Kamera liegen.
     *
     * WICHTIG (Erreichbarkeit): Das "Gerüst" besteht nur aus bespringbaren
     * Plattformen (normal/beweglich/verschwindend) mit Abständen unterhalb
     * der maximalen Sprunghöhe. Zerbrechliche Plattformen ERSETZEN keinen
     * Gerüst-Schritt, sondern werden zusätzlich dazwischen gestreut.
     */
    fillPlatforms() {
      const d = DIFFS[this.diff];
      const scale = this.difficultyScale();
      const gap = d.gap + scale * 40;
      while (this.genY > this.camY - this.height * 0.3) {
        const prevY = this.genY;
        this.genY -= rand.uniform(gap * 0.7, gap * 1.15);
        const x = rand.uniform(0, this.width - this.platW);
        let kind = this.pickKind(d, scale);
        // Nie zwei verschwindende Plattformen direkt hintereinander.
        if (kind === "vanish" && this.prevKind === "vanish") kind = "normal";
        this.prevKind = kind;
        const p = makePlat(x, this.genY, this.platW, kind);
        if (kind === "move") p.vx = rand.choice([-1, 1]) * rand.uniform(60, 110);
        // Feder / Propeller auf stabilen Plattformen
        if (kind === "normal" || kind === "move") {
          const rr = rand.random();
          if (rr < 0.06) p.prop = true;
          else if (rr < 0.16) p.spring = true;
        }
        this.platforms.push(p);
        // Zerbrechliche Plattform als zusätzliche Falle ZWISCHEN den
        // Gerüst-Plattformen (nie als Ersatz eines Schritts).
        const pbrit = d.brittle + scale * 0.1;
        if (prevY - this.genY > 40 && rand.random() < pbrit) {
          this.platforms.push(makePlat(rand.uniform(0, this.width - this.platW), rand.uniform(this.genY + 14, prevY - 14), this.platW, "brittle"));
        }
        // Monster gelegentlich zwischen die Plattformen
        if (rand.random() < d.monster * (0.5 + scale)) {
          this.monsters.push({
            x: rand.uniform(40, this.width - 40),
            y: this.genY - rand.uniform(20, 40),
            vx: rand.choice([-1, 1]) * rand.uniform(20, 55),
            hp: 1,
            kind: rand.randint(0, 2),
            bob: rand.uniform(0, PG.TAU),
          });
        }
      }
    }

    /** Wählt die Art einer GERÜST-Plattform - alle sind bespringbar. */
    pickKind(d, scale) {
      const r = rand.random();
      const pmove = d.move + scale * 0.12;
      const pvan = 0.06 + scale * 0.06;
      if (r < pvan) return "vanish";
      if (r < pvan + pmove) return "move";
      return "normal";
    }

    cull() {
      const limit = this.camY + this.height + 60;
      this.platforms = this.platforms.filter((p) => p.y < limit && !(p.dead && p.y > this.camY + this.height));
      this.monsters = this.monsters.filter((m) => m.y < limit);
    }

    die() {
      if (this.state !== PLAY) return;
      this.state = GAMEOVER;
      this.gameOver = true;
      this.newBest = this.score > this.best;
      this.best = Math.max(this.best, this.score);
      this.overT = ui.now();
      this.playSound("gameover");
      this.rumble(220);
      this.sparkle(this.x, this.y, COL_DOODLE, 16);
    }

    // ----- Effekte -------------------------------------------------------
    sparkle(x, y, col, n = 9) {
      for (let i = 0; i < n; i++) {
        const a = rand.uniform(0, PG.TAU);
        const sp = rand.uniform(40, 160);
        this.particles.push([x, y, Math.cos(a) * sp, Math.sin(a) * sp, rand.uniform(0.25, 0.5), col]);
      }
    }

    updateParticles(dt) {
      const rest = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] += 260 * dt;
        p[4] -= dt;
        if (p[4] > 0) rest.push(p);
      }
      this.particles = rest;
    }

    /** Halbtransparentes Theme-Panel mit Akzent-Rahmen. */
    panel(ctx, rect, alpha = 235, border = 2) {
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], alpha], rect, 0, 12);
      draw.rect(ctx, this.accent, rect, border, 12);
    }

    // ===================================================== Zeichnen
    sy(wy) {
      return Math.trunc(wy - this.camY);
    }

    draw(ctx) {
      // Web: update() läuft bei gameOver nicht -> Partikel hier weiterbewegen.
      const now = ui.now();
      if (this.gameOver && this.lastDrawT !== null) {
        const dt = Math.min(0.1, now - this.lastDrawT);
        this.animT += dt;
        this.updateParticles(dt);
      }
      this.lastDrawT = now;

      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawBg(ctx);
      for (const p of this.platforms) if (!p.dead) this.drawPlat(ctx, p);
      for (const m of this.monsters) this.drawMonster(ctx, m);
      for (const b of this.bullets) {
        draw.circle(ctx, [240, 250, 255], [Math.trunc(b.x), this.sy(b.y)], 5);
        draw.circle(ctx, [120, 180, 230], [Math.trunc(b.x), this.sy(b.y)], 5, 1);
      }
      for (const p of this.particles) {
        const a = Math.max(0, Math.min(255, Math.trunc((255 * p[4]) / 0.5)));
        draw.circle(ctx, [p[5][0], p[5][1], p[5][2], a], [p[0], this.sy(p[1])], 2);
      }
      this.drawDoodler(ctx);
      this.drawHud(ctx);
    }

    drawBg(ctx) {
      const ps = Math.max(1, (PG.app && PG.app.pixelScale) || 1);
      const key = this.width + "x" + this.height + "@" + ps;
      if (!this.bgCache || this.bgCache[0] !== key) {
        const c = ui.makeCanvas(this.width * ps, this.height * ps);
        const g = c.getContext("2d");
        const lg = g.createLinearGradient(0, 0, 0, c.height);
        lg.addColorStop(0, ui.col(COL_BG_TOP));
        lg.addColorStop(1, ui.col(COL_BG_BOT));
        g.fillStyle = lg;
        g.fillRect(0, 0, c.width, c.height);
        this.bgCache = [key, c];
      }
      ctx.drawImage(this.bgCache[1], 0, 0, this.width, this.height);
      // feines Karo-Muster (scrollt mit)
      const step = 40;
      const off = PG.mod(Math.trunc(-this.camY), step);
      for (let y = -step; y < this.height + step; y += step) draw.line(ctx, COL_GRID, [0, y + off + 0.5], [this.width, y + off + 0.5], 1);
      for (let x = 0; x < this.width; x += step) draw.line(ctx, COL_GRID, [x + 0.5, 0], [x + 0.5, this.height], 1);
    }

    drawPlat(ctx, p) {
      const sy = this.sy(p.y);
      const col = PLAT_COLORS[p.kind];
      const rect = [Math.trunc(p.x), sy, Math.trunc(p.w), 14];
      draw.rect(ctx, col, rect, 0, 6);
      draw.rect(ctx, dark(col), rect, 2, 6);
      if (p.kind === "brittle") draw.line(ctx, [120, 80, 50], [p.x + p.w * 0.4, sy], [p.x + p.w * 0.5, sy + 14], 2);
      if (p.spring) {
        const sx = Math.trunc(p.x + p.w / 2);
        draw.rect(ctx, [200, 200, 210], [sx - 5, sy - 10, 10, 10]);
        draw.line(ctx, [120, 120, 130], [sx - 4, sy - 8], [sx + 4, sy - 4], 2);
        draw.line(ctx, [120, 120, 130], [sx - 4, sy - 4], [sx + 4, sy - 8], 2);
      }
      if (p.prop) {
        const sx = Math.trunc(p.x + p.w / 2);
        draw.ellipse(ctx, [90, 150, 230], [sx - 14, sy - 8, 28, 6]);
        draw.circle(ctx, [60, 90, 160], [sx, sy - 5], 3);
      }
    }

    drawMonster(ctx, m) {
      const x = Math.trunc(m.x);
      const y = this.sy(m.y) + Math.trunc(Math.sin(this.animT * 4 + m.bob) * 3);
      const col = MONSTER_COLS[m.kind % 3];
      draw.circle(ctx, col, [x, y], 16);
      draw.circle(ctx, dark(col), [x, y], 16, 2);
      for (const sx of [-6, 6]) {
        draw.circle(ctx, [255, 255, 255], [x + sx, y - 4], 5);
        draw.circle(ctx, [30, 30, 40], [x + sx, y - 4], 2);
      }
      draw.arc(ctx, [60, 30, 40], [x - 7, y + 2, 14, 8], Math.PI, PG.TAU, 2);
    }

    drawDoodler(ctx) {
      const x = Math.trunc(this.x);
      const y = this.sy(this.y);
      const r = this.dr;
      const aimUp = this.shootPose > 0;
      // Körper
      draw.ellipse(ctx, COL_DOODLE, [x - r, y - r * 0.9, 2 * r, 1.8 * r]);
      draw.ellipse(ctx, COL_DOODLE_DARK, [x - r, y - r * 0.9, 2 * r, 1.8 * r], 2);
      // Beine
      draw.line(ctx, COL_DOODLE_DARK, [x - r * 0.4, y + r * 0.7], [x - r * 0.6, y + r * 1.2], 3);
      draw.line(ctx, COL_DOODLE_DARK, [x + r * 0.4, y + r * 0.7], [x + r * 0.6, y + r * 1.2], 3);
      // Nase/Schnauze (zeigt in Blickrichtung, beim Schießen nach oben)
      if (aimUp) {
        draw.ellipse(ctx, [90, 170, 70], [x - 6, y - r * 1.4, 12, 12]);
      } else {
        const nx = x + this.face * r * 0.8;
        draw.ellipse(ctx, [90, 170, 70], [this.face > 0 ? nx - 6 : nx - 8, y - 4, 14, 12]);
      }
      // Augen
      for (const sx of [-1, 1]) {
        const ex = x + sx * r * 0.4;
        draw.circle(ctx, [255, 255, 255], [Math.trunc(ex), Math.trunc(y - r * 0.4)], 5);
        draw.circle(ctx, [30, 30, 40], [Math.trunc(ex + this.face), Math.trunc(y - r * 0.4)], 2);
      }
      // Propeller-Hut
      if (this.propT > 0) {
        draw.rect(ctx, [80, 110, 180], [x - 4, y - r * 1.7, 8, 8]);
        const w = Math.trunc(16 + 6 * Math.sin(this.animT * 40));
        draw.ellipse(ctx, [120, 160, 230], [x - w, y - r * 1.8, 2 * w, 6]);
      }
      // Wrap-Kopie am Rand
      if (x < r) this.ghostDoodle(ctx, x + this.width, y, r);
      else if (x > this.width - r) this.ghostDoodle(ctx, x - this.width, y, r);
    }

    ghostDoodle(ctx, x, y, r) {
      draw.ellipse(ctx, COL_DOODLE, [Math.trunc(x - r), Math.trunc(y - r * 0.9), Math.trunc(2 * r), Math.trunc(1.8 * r)]);
    }

    // ----- HUD / Overlays -----------------------------------------------
    drawHud(ctx) {
      // Punkte-Plakette links, Bestwert rechts - Theme-Panels über dem Papier
      const sc = String(this.score);
      const box = new Rect(8, 8, this.small.width(sc) + 24, this.small.height + 10);
      this.panel(ctx, box, 210, 1);
      ui.text(ctx, sc, box.centerx, box.centery, this.small, ui.TEXT, "center");
      const bestTxt = t("dj.best", { hs: this.best });
      const bw = this.tiny.width(bestTxt) + 20;
      const bbox = new Rect(this.width - 8 - bw, 8, bw, this.tiny.height + 10);
      this.panel(ctx, bbox, 210, 1);
      ui.text(ctx, bestTxt, bbox.centerx, bbox.centery, this.tiny, ui.TEXT_DIM, "center");
      if (this.state === GAMEOVER) this.drawGameover(ctx);
    }

    drawGameover(ctx) {
      draw.rect(ctx, [10, 12, 20, 140], [0, 0, this.width, this.height]);
      const cx = fl(this.width / 2), cy = fl(this.height / 2);
      ui.text(ctx, t("common.game_over"), cx, cy - Math.trunc(this.height * 0.16), this.huge, ui.RED, "center");

      const pw = Math.min(340, this.width - 40);
      const ph = this.font.height + this.small.height + 46;
      const panel = new Rect(cx - fl(pw / 2), cy - fl(ph / 2) + 10, pw, ph);
      this.panel(ctx, panel);
      ui.text(ctx, t("common.points", { score: this.score }), cx, panel.y + 14, this.font, ui.TEXT, "midtop");
      ui.text(ctx, t("dj.best", { hs: this.best }), cx, panel.bottom - 14, this.small, this.newBest ? ui.GOLD : ui.TEXT_DIM, "midbottom");

      if (this.newBest) {
        const recCol = ui.mix(ui.GOLD, ui.TEXT, ui.pulse(3.0, 0.0, 0.5));
        ui.text(ctx, t("trex.new_record"), cx, panel.top - 18, this.small, recCol, "center");
      }

      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
      ui.text(ctx, t("dj.restart_hint"), cx, panel.bottom + 28, this.small, hintCol, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      ui.drawTitle(ctx, this.width, "DOODLE JUMP", { subtitle: t("snake.singleplayer"), accent: this.accent });

      const d = DIFFS[this.diff];
      const dp = this.diffPanel;
      this.panel(ctx, dp);
      ui.text(ctx, t("dj.difficulty") + ":  " + t("dj.diff." + d.key), dp.centerx, dp.top + Math.trunc(dp.h * 0.36), this.font, ui.TEXT, "center");
      ui.text(ctx, t("dj.diff_note"), dp.centerx, dp.top + Math.trunc(dp.h * 0.72), this.tiny, ui.TEXT_DIM, "center");
      const arrCol = ui.mix(this.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3));
      for (const [rect, sym] of [[this.diffLeft, "<"], [this.diffRight, ">"]]) {
        ui.text(ctx, sym, rect.centerx, rect.centery, this.bigFont, arrCol, "center");
      }

      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: this.accent });

      ui.text(ctx, t("dj.controls_hint"), fl(this.width / 2), this.startRect.bottom + 24, this.tiny, ui.GREEN, "center");
      ui.drawFooter(ctx, this.width, this.height, t("dj.setup_hint"));
    }
  }

  PG.register(DoodleGame, {
    id: "DoodleGame",
    key: "doodle",
    name: "Doodle Jump",
    settingsKey: "doodle",
    defaults: { difficulty: 1 },
  });
})();
