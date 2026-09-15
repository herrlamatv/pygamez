/*
 * frogger.js - Frogger, Klassiker mit Extras (Port von games/frogger.py)
 * =======================================================================
 * Aufbau (13 logische Reihen von oben nach unten):
 * - Reihe 0     : Ufer mit 5 Ziel-Buchten (alle füllen = Level geschafft)
 * - Reihen 1-5  : Fluss - Stämme und Schildkröten tragen den Frosch;
 *                 Schildkröten tauchen ab höheren Leveln periodisch ab
 * - Reihe 6     : Mittelstreifen (sicher)
 * - Reihen 7-11 : Straße - Autos und Laster in wechselnden Richtungen
 * - Reihe 12    : Startstreifen (sicher)
 *
 * Extras: Bonus-Fliege (+200), Krokodile in Buchten, Zeitlimit je Frosch,
 * 3 Schwierigkeitsgrade, Level werden schneller. 3 Leben, Extraleben
 * einmalig bei 10 000 Punkten.
 *
 * Steuerung: Pfeile/WASD = hüpfen, R = neu, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const R = PG.rand;

  // ----- Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme)
  const COL_GRASS = [36, 66, 46];       // Ufer
  const COL_GRASS_DARK = [28, 52, 38];  // Mittel-/Startstreifen
  const COL_ROAD = [30, 33, 44];
  const COL_ROAD_LINE = [70, 76, 96];
  const COL_RIVER = [24, 38, 66];
  const COL_BAY = [16, 24, 38];         // leere Bucht
  const COL_BAY_DONE = [56, 120, 80];   // gefüllte Bucht
  const COL_FROG = [108, 205, 109];     // Froschkörper (helles Grün)
  const COL_FROG_DARK = [58, 130, 78];
  const COL_CAR = [[225, 95, 95], [245, 205, 100], [150, 160, 235], [240, 240, 250]];
  const COL_TRUCK = [150, 158, 178];
  const COL_LOG = [128, 92, 60];
  const COL_LOG_DARK = [96, 68, 44];
  const COL_TURTLE = [90, 160, 120];
  const COL_TURTLE_DARK = [60, 110, 84];
  const COL_CROC = [70, 130, 70];
  const COL_FLY = [245, 205, 100];

  // Schwierigkeit: Tempo-Faktor, Lücken (Zellen), Zeit je Frosch (s),
  // Krokodile ab Level, tauchende Schildkröten ab Level.
  const DIFFS = [
    { key: "easy", speed: 0.8, gap: 3.5, timer: 45, crocLv: 4, diveLv: 3 },
    { key: "normal", speed: 1.0, gap: 2.75, timer: 35, crocLv: 2, diveLv: 2 },
    { key: "hard", speed: 1.25, gap: 2.0, timer: 25, crocLv: 1, diveLv: 1 },
  ];
  const DIFF_KEYS = DIFFS.map((d) => d.key);

  // Lanes: (Art, Basistempo in Zellen/s, Breite in Zellen). Vorzeichen = Richtung.
  const ROAD_LANES = [["car", 1.6, 1], ["truck", -1.1, 2], ["car", 1.9, 1],
    ["car", -2.4, 1], ["truck", 1.3, 2]];     // Reihen 7..11
  const RIVER_LANES = [["log", 1.2, 3], ["turtle", -1.5, 3], ["log", 1.9, 4],
    ["turtle", -1.2, 2], ["log", 1.6, 2]];    // Reihen 1..5

  const EXTRA_LIFE_AT = 10000;
  const FLY_TIME = 4.0;
  const CROC_TIME = 5.0;
  const DIVE_CYCLE = 10.0; // 6s oben, 1s blinken, 2s unten, 1s auftauchen

  const SETUP = "setup", PLAY = "play";

  class FroggerGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const key = this.opts.difficulty;
      this.diffIdx = DIFF_KEYS.includes(key) ? DIFF_KEYS.indexOf(key) : 1;

      this.makeFonts();
      this.animT = 0.0;
      this.lanes = [];
      this.lastDraw = null;

      this.layout();
      this.buildSetupLayout();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größen aus der Fensterhöhe abgeleitet. */
    makeFonts() {
      this.fSmall = ui.font(Math.max(13, Math.min(22, Math.floor(this.height / 30))));
      this.fTiny = ui.font(Math.max(11, Math.min(18, Math.floor(this.height / 38))));
      this.fHuge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    diff() {
      return DIFFS[this.diffIdx];
    }

    /** Spielfeld-Maße aus der Fläche ableiten. */
    layout() {
      this.hudH = Math.max(30, Math.floor(this.height * 0.08));
      this.barH = Math.max(8, Math.floor(this.height / 48));
      this.cell = Math.max(16, Math.floor((this.height - this.hudH - this.barH) / 13));
      this.fieldTop = this.hudH;
    }

    rowY(row) {
      return this.fieldTop + row * this.cell;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(420, this.width - 60);
      const y0 = Math.floor(this.height * 0.3);
      this.diffRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 62, bw, 52));
      this.startRect = new PG.Rect(cx - 95, y0 + 3 * 62 + 14, 190, 46);
    }

    saveDiff() {
      this.opts.difficulty = DIFF_KEYS[this.diffIdx];
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diffIdx = Number(k) - 1;
          this.saveDiff();
          this.playSound("click");
        } else if (k === "Up" || k === "w" || k === "W") {
          this.diffIdx = PG.mod(this.diffIdx - 1, 3);
          this.saveDiff();
          this.playSound("move");
        } else if (k === "Down" || k === "s" || k === "S") {
          this.diffIdx = PG.mod(this.diffIdx + 1, 3);
          this.saveDiff();
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.newGame();
        }
      } else if (ev.kind === "mousedown" && ev.pos) {
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diffIdx = i;
            this.saveDiff();
            this.playSound("click");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.newGame();
      }
    }

    // ===================================================== Spielaufbau
    newGame() {
      this.score = 0;
      this.gameOver = false;
      this.lives = 3;
      this.level = 1;
      this.extraLifeGiven = false;
      this.baysDone = [false, false, false, false, false];
      this.fly = null;   // {bay, t}
      this.croc = null;
      this.flyTimer = R.uniform(10, 16);
      this.deathT = 0.0;
      this.clearT = 0.0;
      this.buildLanes();
      this.respawn(true);
      this.state = PLAY;
      this.playSound("click");
    }

    speedMult() {
      return Math.min(2.5, this.diff().speed * (1 + 0.12 * (this.level - 1)));
    }

    /** Erzeugt alle Fahrspuren/Flussbahnen mit gleichmäßig verteilten
     *  Objekten. Objekte wandern und wrappen über eine gemeinsame Spannweite. */
    buildLanes() {
      this.lanes = [];
      const d = this.diff();
      const mult = this.speedMult();

      const addLane = (row, kind, baseSpeed, wCells) => {
        const speed = baseSpeed * mult * this.cell;
        const ew = wCells * this.cell;
        const gap = d.gap * this.cell * R.uniform(0.8, 1.3);
        const margin = ew + this.cell;
        const n = Math.max(2, Math.floor((this.width + 2 * margin) / (ew + gap)) + 1);
        const span = n * (ew + gap);
        const ents = [];
        for (let i = 0; i < n; i++) {
          ents.push({ x: i * (ew + gap) - margin, w: ew, color: R.choice(COL_CAR), dive: false, phase: R.uniform(0, DIVE_CYCLE) });
        }
        this.lanes.push({ row, kind, speed, ents, span, margin });
      };

      ROAD_LANES.forEach(([kind, sp, w], i) => addLane(7 + i, kind, sp, w));
      RIVER_LANES.forEach(([kind, sp, w], i) => addLane(1 + i, kind, sp, w));

      // Tauchende Schildkröten: ab Schwellen-Level jede 2. Gruppe.
      if (this.level >= d.diveLv) {
        for (const lane of this.lanes) {
          if (lane.kind === "turtle") lane.ents.forEach((e, j) => (e.dive = j % 2 === 0));
        }
      }
    }

    respawn(full = false) {
      this.frogRow = 12;
      this.frogX = this.width / 2;
      this.bestRow = 12;
      this.timeLeft = this.diff().timer;
      if (full) this.animT = 0.0;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.newGame();
          else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.playSound("click");
          }
        }
        return;
      }
      if (ev.kind !== "keydown") return;
      if (ev.key === "r" || ev.key === "R") {
        this.newGame();
        return;
      }
      if ((ev.key === "s" || ev.key === "S") && !this.isAction(ev.key, "down")) {
        // S gehört standardmäßig zu "runter" (WASD) -> nur als Setup-
        // Taste werten, wenn es nicht als Aktion belegt ist.
        this.state = SETUP;
        this.playSound("click");
        return;
      }
      if (this.deathT > 0 || this.clearT > 0) return;
      if (this.isAction(ev.key, "up") || ev.key === "Up") this.hop(0, -1);
      else if (this.isAction(ev.key, "down") || ev.key === "Down") this.hop(0, 1);
      else if (this.isAction(ev.key, "left") || ev.key === "Left") this.hop(-1, 0);
      else if (this.isAction(ev.key, "right") || ev.key === "Right") this.hop(1, 0);
    }

    hop(dx, dy) {
      const nx = this.frogX + dx * this.cell;
      const nrow = this.frogRow + dy;
      if (!(nrow >= 0 && nrow <= 12)) return;
      const half = this.cell * 0.5;
      if (!(nx >= half && nx <= this.width - half)) return;
      this.frogX = nx;
      this.frogRow = nrow;
      this.playSound("move");
      // Punkte für jede neu erreichte (höhere) Reihe in diesem Leben.
      if (nrow < this.bestRow) {
        this.score += 10 * (this.bestRow - nrow);
        this.bestRow = nrow;
      }
      if (nrow === 0) this.tryHome();
    }

    // ===================================================== Spiellogik
    /** Objekte bewegen und über die gemeinsame Spannweite wrappen. */
    moveLanes(dt) {
      for (const lane of this.lanes) {
        for (const e of lane.ents) {
          e.x += lane.speed * dt;
          if (lane.speed > 0 && e.x > this.width + lane.margin) e.x -= lane.span;
          else if (lane.speed < 0 && e.x + e.w < -lane.margin) e.x += lane.span;
        }
      }
    }

    update(dt) {
      this.animT += dt;
      if (this.state !== PLAY) return;

      // Objekte bewegen (auch während Todes-/Clear-Anims, wirkt lebendiger)
      this.moveLanes(dt);

      if (this.gameOver) return;

      if (this.clearT > 0) {
        this.clearT -= dt;
        if (this.clearT <= 0) this.nextLevel();
        return;
      }
      if (this.deathT > 0) {
        this.deathT -= dt;
        if (this.deathT <= 0) this.afterDeath();
        return;
      }

      // Extraleben
      if (!this.extraLifeGiven && this.score >= EXTRA_LIFE_AT) {
        this.extraLifeGiven = true;
        this.lives += 1;
        this.playSound("powerup");
      }

      // Fliege / Krokodil in den Buchten
      this.updateBayExtras(dt);

      // Zeitlimit
      this.timeLeft -= dt;
      if (this.timeLeft <= 0) {
        this.die();
        return;
      }

      // Fluss: mitfahren oder ertrinken
      if (this.frogRow >= 1 && this.frogRow <= 5) {
        const carrier = this.carrierAt(this.frogRow, this.frogX);
        if (carrier === null) {
          this.die();
          return;
        }
        this.frogX += carrier[0].speed * dt;
        const half = this.cell * 0.5;
        if (this.frogX < half || this.frogX > this.width - half) {
          this.die();
          return;
        }
      }

      // Straße: Kollision
      if (this.frogRow >= 7 && this.frogRow <= 11) {
        if (this.vehicleHit(this.frogRow, this.frogX)) this.die();
      }
    }

    laneForRow(row) {
      return this.lanes.find((l) => l.row === row) || null;
    }

    /** 'up' / 'blink' / 'under' / 'rise' je nach Tauch-Zyklus. */
    turtleState(ent) {
      if (!ent.dive) return "up";
      const tt = PG.mod(this.animT + ent.phase, DIVE_CYCLE);
      if (tt < 6.0) return "up";
      if (tt < 7.0) return "blink";
      if (tt < 9.0) return "under";
      return "rise";
    }

    /** Stamm/aufgetauchte Schildkröte unter Position x (oder null). */
    carrierAt(row, x) {
      const lane = this.laneForRow(row);
      if (lane === null) return null;
      for (const e of lane.ents) {
        if (e.x <= x && x <= e.x + e.w) {
          if (lane.kind === "turtle" && this.turtleState(e) === "under") return null;
          return [lane, e];
        }
      }
      return null;
    }

    vehicleHit(row, x) {
      const lane = this.laneForRow(row);
      if (lane === null) return false;
      const half = this.cell * 0.3; // Frosch-Hitbox etwas kleiner als die Zelle
      return lane.ents.some((e) => e.x < x + half && x - half < e.x + e.w);
    }

    /** Bucht, deren Zentrum nah genug an x liegt (oder null). */
    bayIndexAt(x) {
      for (let i = 0; i < 5; i++) {
        const cx = ((i + 0.5) * this.width) / 5;
        if (Math.abs(x - cx) < 0.5 * this.cell) return i;
      }
      return null;
    }

    tryHome() {
      const bay = this.bayIndexAt(this.frogX);
      if (bay === null || this.baysDone[bay]) {
        this.die();
        return;
      }
      if (this.croc !== null && this.croc.bay === bay) {
        this.die();
        return;
      }
      // Geschafft!
      this.baysDone[bay] = true;
      const bonus = 50 + 10 * Math.trunc(Math.max(0, this.timeLeft));
      this.score += bonus;
      if (this.fly !== null && this.fly.bay === bay) {
        this.score += 200;
        this.fly = null;
        this.playSound("powerup");
      }
      this.playSound("point");
      if (this.baysDone.every(Boolean)) {
        this.score += 1000;
        this.clearT = 1.5;
        this.achEvent("frog_home");
        this.playSound("level");
      } else {
        this.respawn();
      }
    }

    nextLevel() {
      this.level += 1;
      this.baysDone = [false, false, false, false, false];
      this.fly = null;
      this.croc = null;
      this.buildLanes();
      this.respawn();
    }

    updateBayExtras(dt) {
      const d = this.diff();
      if (this.fly !== null) {
        this.fly.t -= dt;
        if (this.fly.t <= 0) this.fly = null;
      }
      if (this.croc !== null) {
        this.croc.t -= dt;
        if (this.croc.t <= 0) this.croc = null;
      }
      this.flyTimer -= dt;
      if (this.flyTimer <= 0) {
        this.flyTimer = R.uniform(10, 16);
        const free = [0, 1, 2, 3, 4].filter((i) => !this.baysDone[i] && (this.croc === null || this.croc.bay !== i) && (this.fly === null || this.fly.bay !== i));
        if (free.length) {
          // Ab dem Krokodil-Level teilen sich Fliege und Krokodil den Takt.
          if (this.level >= d.crocLv && this.croc === null && R.random() < 0.45) {
            this.croc = { bay: R.choice(free), t: CROC_TIME };
          } else if (this.fly === null) {
            this.fly = { bay: R.choice(free), t: FLY_TIME };
          }
        }
      }
    }

    die() {
      this.deathT = 1.0;
      this.lives -= 1;
      this.playSound("hit");
      this.rumble(160);
    }

    afterDeath() {
      if (this.lives <= 0) {
        this.gameOver = true;
        this.playSound("gameover");
        this.rumble(250);
      } else {
        this.respawn();
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Nach dem Ende läuft nur noch der Verkehr als Kulisse weiter - die App
      // ruft update() bei Game Over nicht mehr auf, daher hier weiterbewegen.
      const now = ui.now();
      if (this.state === PLAY && this.gameOver && this.lastDraw !== null && !this.paused) {
        this.moveLanes(Math.min(0.1, Math.max(0, now - this.lastDraw)));
      }
      this.lastDraw = now;

      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      const s = ctx;
      ui.drawBackground(s, this.width, this.height);
      this.drawField(s);
      this.drawEntities(s);
      this.drawBayExtras(s);
      if (this.deathT > 0) this.drawDeath(s);
      else if (this.clearT <= 0) this.drawFrog(s, this.frogX, this.rowY(this.frogRow));
      this.drawHud(s);
      if (this.clearT > 0) this.drawBanner(s, t("frog.level_clear", { n: this.level }));
      if (this.gameOver) this.drawGameover(s);
    }

    drawField(s) {
      const c = this.cell;
      const w = this.width;
      // Ufer mit Buchten (Reihe 0)
      const y0 = this.rowY(0);
      draw.rect(s, COL_GRASS, [0, y0, w, c]);
      for (let i = 0; i < 5; i++) {
        const cx = ((i + 0.5) * w) / 5;
        const bay = [Math.floor(cx - c * 0.6), y0 + 2, Math.floor(c * 1.2), c - 4];
        draw.rect(s, this.baysDone[i] ? COL_BAY_DONE : COL_BAY, bay, 0, 6);
        if (this.baysDone[i]) this.drawFrog(s, cx, y0, true);
      }
      // Fluss (1-5)
      draw.rect(s, COL_RIVER, [0, this.rowY(1), w, 5 * c]);
      // Mittelstreifen (6) / Start (12)
      draw.rect(s, COL_GRASS_DARK, [0, this.rowY(6), w, c]);
      draw.rect(s, COL_GRASS_DARK, [0, this.rowY(12), w, c]);
      // Straße (7-11) mit Mittellinien
      draw.rect(s, COL_ROAD, [0, this.rowY(7), w, 5 * c]);
      for (let r = 8; r < 12; r++) {
        const y = this.rowY(r);
        for (let x = 0; x < w; x += c) {
          draw.rect(s, COL_ROAD_LINE, [x + Math.floor(c / 4), y - 1, Math.floor(c / 2), 2]);
        }
      }
    }

    drawEntities(s) {
      const c = this.cell;
      for (const lane of this.lanes) {
        const y = this.rowY(lane.row);
        for (const e of lane.ents) {
          const r = new PG.Rect(Math.floor(e.x), y + 3, Math.floor(e.w), c - 6);
          if (lane.kind === "car" || lane.kind === "truck") {
            const col = lane.kind === "truck" ? COL_TRUCK : e.color;
            draw.rect(s, col, r, 0, 5);
            draw.rect(s, col.map((v) => Math.floor(v * 0.6)), r, 2, 5);
            // Fenster in Fahrtrichtung
            const wx = lane.speed > 0 ? r.right - Math.floor(c / 3) : r.x + 4;
            draw.rect(s, [200, 225, 245], [wx, r.y + 3, Math.floor(c / 4), r.h - 6], 0, 3);
          } else if (lane.kind === "log") {
            const rad = Math.floor(c / 3);
            draw.rect(s, COL_LOG, r, 0, rad);
            draw.rect(s, COL_LOG_DARK, r, 2, rad);
            for (let lx = r.x + Math.floor(c / 2); lx < r.right - Math.floor(c / 3); lx += c) {
              draw.line(s, COL_LOG_DARK, [lx + 0.5, r.y + 4], [lx + 0.5, r.bottom - 4]);
            }
          } else {
            // turtle
            const st = this.turtleState(e);
            if (st === "under") continue;
            let col = COL_TURTLE;
            if (st === "blink" && Math.floor(this.animT * 8) % 2 === 0) col = COL_TURTLE_DARK;
            if (st === "rise") col = COL_TURTLE_DARK;
            const n = Math.max(1, Math.floor(Math.floor(e.w) / c));
            const rr = Math.floor(c / 2) - 3;
            for (let i = 0; i < n; i++) {
              const cxx = Math.floor(e.x + (i + 0.5) * c);
              draw.circle(s, col, [cxx, y + Math.floor(c / 2)], rr);
              draw.circle(s, COL_TURTLE_DARK, [cxx, y + Math.floor(c / 2)], rr, 2);
            }
          }
        }
      }
    }

    drawBayExtras(s) {
      const c = this.cell;
      const y0 = this.rowY(0);
      if (this.fly !== null) {
        const cx = Math.floor(((this.fly.bay + 0.5) * this.width) / 5);
        const fy = y0 + Math.floor(c / 2);
        draw.circle(s, COL_FLY, [cx, fy], Math.max(3, Math.floor(c / 6)));
        const off = Math.max(2, Math.floor(c / 8));
        for (const sx of [-off, off]) draw.circle(s, [255, 240, 200], [cx + sx, fy - off], Math.max(2, Math.floor(c / 9)), 1);
      }
      if (this.croc !== null) {
        const cx = Math.floor(((this.croc.bay + 0.5) * this.width) / 5);
        const r = new PG.Rect(cx - Math.floor(c * 0.55), y0 + Math.floor(c / 4), Math.floor(c * 1.1), Math.floor(c / 2));
        draw.rect(s, COL_CROC, r, 0, 4);
        // Aufgerissenes Maul + Auge
        draw.polygon(s, [240, 240, 250], [[r.x + 4, r.centery - 2], [r.x + Math.floor(c / 3), r.y + 2], [r.x + Math.floor(c / 3), r.bottom - 2]]);
        draw.circle(s, [20, 24, 34], [r.right - Math.floor(c / 4), r.y + 4], 2);
      }
    }

    drawFrog(s, x, y, small = false) {
      const c = this.cell;
      const r = Math.floor(c / 2) - (small ? 4 : 2);
      const cx = Math.floor(x), cy = Math.floor(y + Math.floor(c / 2));
      // Beine
      if (!small) {
        for (const sx of [-1, 1]) draw.line(s, COL_FROG_DARK, [cx + sx * r, cy], [cx + sx * (r + 3), cy + r], 3);
      }
      draw.ellipse(s, COL_FROG, [cx - r, cy - r, 2 * r, 2 * r]);
      draw.ellipse(s, COL_FROG_DARK, [cx - r, cy - r, 2 * r, 2 * r], 2);
      // Augen
      const er = Math.max(2, Math.floor(r / 3));
      for (const sx of [-1, 1]) {
        const ex = cx + Math.floor((sx * r) / 2), ey = cy - r + Math.floor(er / 2);
        draw.circle(s, [240, 240, 250], [ex, ey], er);
        draw.circle(s, [20, 24, 34], [ex, ey], Math.max(1, Math.floor(er / 2)));
      }
    }

    /** Splat/Platsch-Animation an der Todesposition. */
    drawDeath(s) {
      const c = this.cell;
      const cx = Math.floor(this.frogX);
      const cy = Math.floor(this.rowY(this.frogRow) + Math.floor(c / 2));
      const f = 1.0 - this.deathT; // 0..1
      const n = 8;
      for (let i = 0; i < n; i++) {
        const a = (i / n) * PG.TAU;
        const d = f * c * 0.9;
        draw.circle(s, COL_FROG_DARK, [Math.floor(cx + Math.cos(a) * d), Math.floor(cy + Math.sin(a) * d)], Math.max(1, Math.floor(4 * (1 - f))));
      }
      ui.text(s, "+", cx, cy, this.fSmall, ui.RED, "center");
    }

    drawHud(s) {
      draw.rect(s, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(s, ui.BORDER, [0, this.hudH + 0.5], [this.width, this.hudH + 0.5]);
      const cy = Math.floor(this.hudH / 2);
      ui.text(s, t("common.points", { score: this.score }), 12, cy, this.fSmall, ui.TEXT, "midleft");
      ui.text(s, t("frog.level", { n: this.level }), Math.floor(this.width / 2), cy, this.fSmall, this.accent, "center");
      // Leben als Mini-Frösche rechts
      for (let i = 0; i < this.lives; i++) {
        const fx = this.width - 20 - i * (Math.floor(this.cell / 2) + 8);
        this.drawFrog(s, fx, cy - Math.floor(this.cell / 2), true);
      }

      // Zeitbalken unten
      const frac = Math.max(0.0, this.timeLeft / this.diff().timer);
      draw.rect(s, frac > 0.25 ? ui.GREEN : ui.RED, [0, this.height - this.barH, Math.floor(this.width * frac), this.barH]);
    }

    drawBanner(s, text) {
      const tw = this.fHuge.width(text), th = this.fHuge.height;
      const bg = new PG.Rect(0, 0, tw, th);
      bg.center = [Math.floor(this.width / 2), Math.floor(this.height / 2)];
      const box = bg.inflate(48, 26);
      draw.rect(s, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 225], box);
      draw.rect(s, this.accent, box, 2, 12);
      ui.text(s, text, box.centerx, box.centery, this.fHuge, this.accent, "center");
    }

    drawGameover(s) {
      draw.rect(s, [8, 10, 16, 150], [0, 0, this.width, this.height]);
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      const rows = [
        [t("common.game_over"), this.fHuge, ui.RED],
        [t("common.points", { score: this.score }), this.font, ui.TEXT],
        [t("frog.retry"), this.fSmall, ui.TEXT_DIM],
      ];
      const gap = 10;
      const total = rows.reduce((acc, [, f]) => acc + f.height, 0) + gap * (rows.length - 1);
      const pw = Math.min(this.width - 30, Math.max(340, Math.max(...rows.map(([txt, f]) => f.width(txt))) + 64));
      const panel = new PG.Rect(0, 0, pw, total + 48);
      panel.center = [cx, cy];
      draw.rect(s, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], panel);
      draw.rect(s, this.accent, panel, 2, 14);
      let yy = panel.y + 24;
      for (const [txt, f, col] of rows) {
        ui.text(s, txt, cx, yy, f, col, "midtop");
        yy += f.height + gap;
      }
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(s) {
      ui.drawBackground(s, this.width, this.height);
      const cx = Math.floor(this.width / 2);
      ui.text(s, "FROGGER", cx, Math.floor(this.height * 0.14), this.fHuge, this.accent, "center");
      ui.text(s, t("frog.subtitle"), cx, Math.floor(this.height * 0.2), this.fSmall, ui.TEXT_DIM, "center");
      this.diffRects.forEach((r, i) => {
        const on = i === this.diffIdx;
        draw.rect(s, on ? ui.BTN_SEL : ui.BTN, r, 0, 10);
        draw.rect(s, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 10);
        ui.text(s, t("frog.diff." + DIFF_KEYS[i]), r.x + 18, r.y + 18, this.font, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
        ui.text(s, t("frog.diff_desc." + DIFF_KEYS[i]), r.x + 18, r.bottom - 15, this.fTiny, ui.TEXT_DIM, "midleft");
      });
      draw.rect(s, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(s, this.accent, this.startRect, 2, 10);
      ui.text(s, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(s, t("frog.setup_hint"), cx, this.height - 30, this.fTiny, ui.TEXT_FAINT, "center");
      ui.text(s, t("frog.hint"), cx, this.height - 12, this.fTiny, ui.mix(this.accent, ui.TEXT, 0.45), "center");
    }
  }

  PG.register(FroggerGame, {
    id: "FroggerGame",
    key: "frogger",
    name: "Frogger",
    settingsKey: "frogger",
    defaults: { difficulty: "normal" },
  });
})();
