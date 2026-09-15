/*
 * pinball.js - Flipperautomat (Port von games/pinball.py)
 * ========================================================
 * Drei Tische, Multiball, Drop-Targets, Rollover-Bahnen, Bonus-Multiplikator,
 * Ball-Save, Nudge und Tilt.
 *
 * Tische (im Setup wählbar, wird gespeichert):
 *   - Classic : klassisches Layout, drei Pop-Bumper, ein Target-Bank links.
 *   - Space   : vier Bumper im Karo, zwei Target-Banks, hohe Punktwerte.
 *   - Lama    : offener Tisch mit sechs Targets im Bogen und mittigem Saucer.
 *
 * Ablauf: Ball in der Schussbahn mit gedrückter Aktionstaste (Leertaste/Enter)
 * aufladen und loslassen. Flipper links/rechts über die Links-/Rechts-Tasten
 * (zusätzlich Shift). Mit der Hoch-Taste lässt sich der Tisch anstoßen (Nudge) -
 * dreimal zu schnell hintereinander und der Automat geht auf TILT: die Flipper
 * sind bis zum Ballverlust tot.
 *
 * Punkte: Bumper, Slingshots, Targets und Bahnen zählen mit dem aktuellen
 * Multiplikator (bis x5). Drei gefangene Bälle im Saucer starten den
 * Multiball samt Jackpot. Der Highscore ist die Punktzahl einer Partie.
 * Web-Version: nur Einzelspieler (der 2-Spieler-Wechsel entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ------------------------------------------------- Identitätsfarben (Tisch)
  const COL_TABLE = [24, 26, 40];
  const COL_TABLE_D = [18, 19, 30];
  const COL_WALL = [128, 136, 168];
  const COL_WALL_HI = [176, 186, 220];
  const COL_BALL = [226, 230, 240];
  const COL_BALL_D = [140, 146, 164];
  const COL_FLIP = [238, 196, 74];
  const COL_FLIP_D = [176, 138, 40];
  const COL_BUMP = [86, 196, 236];
  const COL_BUMP_HI = [206, 244, 255];
  const COL_SLING = [236, 108, 96];
  const COL_TARGET = [150, 236, 140];
  const COL_TARGET_OFF = [58, 82, 60];
  const COL_LANE = [208, 150, 240];
  const COL_SAUCER = [250, 214, 120];
  const COL_PLUNGER = [222, 96, 96];

  // ------------------------------------------------------------ Tisch / Physik
  const TW = 100.0, TH = 170.0; // Tischmaße in Tisch-Einheiten
  const BR = 2.2; // Ballradius
  const WALL_R = 1.0; // halbe Dicke einer Wandstrecke
  const FLIP_R = 1.6; // halbe Dicke eines Flippers
  const GRAV = 118.0; // Schwerkraft (Einheiten/s²)
  const DRAG = 0.14; // Rollwiderstand je Sekunde
  const WALL_E = 0.52; // Restitution der Banden
  const SLING_KICK = 118.0; // Zusatzschub der Slingshots
  const BUMP_KICK = 96.0; // Zusatzschub der Pop-Bumper
  const MAX_SPEED = 320.0;
  const FLIP_SPEED = 17.0; // Winkelgeschwindigkeit der Flipper (rad/s)
  const FLIP_LEN = 19.0;
  const DRAIN_Y = 162.0; // darunter ist der Ball verloren
  const LANE_X = 91.0; // Mitte der Schussbahn
  const BALL_SAVE = 6.0; // Sekunden Ball-Save nach dem Abschuss
  const TILT_LIMIT = 3; // so viele Nudges in Folge -> TILT
  const LANE_LETTERS = "LAMA";

  const SETUP = "setup", PLAY = "play", OVER = "over";
  const TABLES = ["classic", "space", "lama"];
  const BALL_COUNTS = [3, 5];

  function makeBall(x, y, vx = 0, vy = 0) {
    // held > 0: liegt im Saucer und wartet; st = Sekunden ohne nennenswerte
    // Bewegung, gemessen ab (sx, sy)
    return { x, y, vx, vy, held: 0, st: 0, sx: x, sy: y };
  }

  /** Kreisbogen als Streckenzug (Winkel in Grad). */
  function arc(cx, cy, r, a0, a1, n) {
    const pts = [];
    for (let i = 0; i <= n; i++) {
      const a = PG.radians(a0 + ((a1 - a0) * i) / n);
      pts.push([cx + Math.cos(a) * r, cy + Math.sin(a) * r]);
    }
    const out = [];
    for (let i = 0; i < pts.length - 1; i++) out.push([pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1]]);
    return out;
  }

  /** Banden, die jeder Tisch teilt: Bogen, Seiten, Schussbahn, Trichter. */
  function baseWalls() {
    return arc(50, 52, 46, 180, 360, 22).concat([
      [4, 52, 4, 110], // linke Wand
      [4, 110, 24, 140], // linker Trichter (endet am Flipper-Drehpunkt)
      [86, 52, 86, 110], // rechte Wand (Trennwand zur Schussbahn)
      [86, 110, 76, 140], // rechter Trichter (endet am Drehpunkt)
      [86, 110, 86, 152], // Schussbahn links
      [96, 40, 96, 152], // Schussbahn rechts
      [86, 152, 96, 152], // Boden der Schussbahn
    ]);
  }

  // Einbahn-Klappe am Ausgang der Schussbahn (wirkt nur vom Spielfeld aus)
  const GATE = [86.0, 52.0, 77.0, 43.0];

  function table(d) {
    return {
      walls: baseWalls().concat(d.extra_walls || []),
      bumpers: d.bumpers, slings: d.slings, targets: d.targets, lanes: d.lanes, saucer: d.saucer,
    };
  }

  const TABLE_DATA = {
    // Klassiker: drei Bumper im Dreieck, ein Target-Bank links
    classic: table({
      bumpers: [[38, 62, 6], [64, 62, 6], [51, 42, 6]],
      slings: [[24, 116, 38, 134], [76, 116, 62, 134]],
      targets: [[12, 66, 7, 6], [12, 76, 7, 6], [12, 86, 7, 6], [12, 96, 7, 6]],
      lanes: [[32, 26, 3], [44, 22, 3], [57, 22, 3], [69, 26, 3]],
      saucer: [50, 88, 4.2],
      extra_walls: [[24, 104, 34, 96], [76, 104, 66, 96]],
    }),
    // Weltraum: vier Bumper im Karo, zwei Banks
    space: table({
      bumpers: [[50, 38, 5.5], [34, 58, 5.5], [66, 58, 5.5], [50, 76, 5.5]],
      slings: [[24, 116, 38, 134], [76, 116, 62, 134]],
      targets: [[11, 62, 7, 6], [11, 72, 7, 6], [11, 82, 7, 6], [79, 62, 7, 6], [79, 72, 7, 6], [79, 82, 7, 6]],
      lanes: [[30, 28, 3], [43, 22, 3], [58, 22, 3], [71, 28, 3]],
      saucer: [50, 100, 4.2],
      extra_walls: [[20, 96, 32, 104], [80, 96, 68, 104]],
    }),
    // Lama: offener Tisch, Targets im Bogen, Saucer in der Mitte
    lama: table({
      bumpers: [[30, 50, 6.5], [70, 50, 6.5]],
      slings: [[24, 116, 38, 134], [76, 116, 62, 134], [34, 92, 44, 104], [66, 92, 56, 104]],
      targets: [[24, 30, 7, 6], [36, 24, 7, 6], [48, 22, 7, 6], [60, 24, 7, 6], [72, 30, 7, 6], [46, 62, 8, 6]],
      lanes: [[16, 60, 3], [16, 72, 3], [84, 60, 3], [84, 72, 3]],
      saucer: [50, 84, 4.6],
      extra_walls: [[16, 96, 26, 104], [84, 96, 74, 104]],
    }),
  };

  function closestOnSeg(x1, y1, x2, y2, px, py) {
    const dx = x2 - x1, dy = y2 - y1;
    const l2 = dx * dx + dy * dy;
    if (l2 < 1e-9) return [x1, y1];
    const f = Math.max(0.0, Math.min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2));
    return [x1 + f * dx, y1 + f * dy];
  }

  /** Dicke Linie mit runden Enden (Banden/Flipper ohne Lücken an den Knicken). */
  function thickLine(ctx, color, a, b, width) {
    ctx.beginPath();
    ctx.moveTo(a[0], a[1]);
    ctx.lineTo(b[0], b[1]);
    ctx.strokeStyle = ui.col(color);
    ctx.lineWidth = width;
    ctx.lineCap = "round";
    ctx.stroke();
    ctx.lineCap = "butt";
  }

  class PinballGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const gs = this.opts;
      this.tableKey = TABLES.includes(gs.table) ? gs.table : "classic";
      this.ballCount = Math.trunc(Number(gs.balls));
      if (!BALL_COUNTS.includes(this.ballCount)) this.ballCount = 3;

      this.buildFonts();
      this.layout();
      this.buildSetupLayout();
      this.tableCache = null;
      this.best = this.loadBest();
      this.newGame();
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, Math.floor(h / 32)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 42)));
      this.num = ui.font(Math.max(16, Math.floor(h / 26)), true);
      this.huge = ui.font(Math.max(24, Math.floor(h / 13)), true);
    }

    layout() {
      this.hudH = 44;
      const availH = this.height - this.hudH - 10;
      this.scale = Math.max(1.1, Math.min((this.width - 150) / TW, availH / TH));
      this.ox = this.width / 2.0 - (TW * this.scale) / 2.0;
      this.oy = this.hudH + (availH - TH * this.scale) / 2.0 + 5;
      this.sideX = Math.trunc(this.ox + TW * this.scale) + 8;
    }

    // ------------------------------------------------------- Speicherstand
    loadBest() {
      const data = PG.store.get("pinball", {});
      const best = data && typeof data === "object" ? data.best : null;
      const out = {};
      if (best && typeof best === "object") {
        for (const k in best) {
          const v = Math.trunc(Number(best[k]));
          if (isFinite(v)) out[String(k)] = v;
        }
      }
      return out;
    }

    saveBest(score) {
      if (score > (this.best[this.tableKey] || 0)) {
        this.best[this.tableKey] = Math.trunc(score);
        PG.store.set("pinball", { best: this.best });
      }
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ------------------------------------------------------- Partie / Ball
    newGame() {
      this.table = TABLE_DATA[this.tableKey];
      this.scores = [0];
      this.ballsLeft = [this.ballCount];
      this.player = 0;
      this.score = 0;
      this.gameOver = false;
      this.msg = null;
      this.msgT = 0.0;
      this.newBall();
    }

    newBall() {
      this.balls = [makeBall(LANE_X, 146.0)];
      this.phase = "launch";
      this.plunger = 0.0;
      this.charging = false;
      this.flip = { l: 0.0, r: 0.0 }; // 0 = Ruhe, 1 = oben
      this.flipUp = { l: false, r: false };
      this.omega = { l: 0.0, r: 0.0 };
      this.mult = 1;
      this.locks = 0;
      this.multiball = false;
      this.jackpot = false;
      this.saveT = 0.0;
      this.tilt = false;
      this.nudges = 0;
      this.nudgeT = 0.0;
      this.shake = 0.0;
      this.targetsHit = this.table.targets.map(() => false);
      this.lanesHit = this.table.lanes.map(() => false);
      this.bankClears = 0;
      this.flash = {};
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(370, this.width - 50);
      const y0 = Math.floor(this.height * 0.3);
      const gap = 8;
      const row = (y, n) => {
        const cw = (bw - gap * (n - 1)) / n;
        return Array.from({ length: n }, (_, i) => new PG.Rect(Math.trunc(cx - bw / 2 + i * (cw + gap)), y, Math.trunc(cw), 42));
      };
      this.tableRects = row(y0, 3);
      this.ballRects = row(y0 + 88, 2);
      this.startRect = new PG.Rect(cx - 95, y0 + 156, 190, 46);
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.tableKey = TABLES[Number(k) - 1];
          this.saveSetting("table", this.tableKey);
          this.playSound("click");
        } else if (k === "b" || k === "B") {
          this.cycleBalls();
        } else if (k === "Return" || k === "space") {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.tableRects.length; i++) {
          if (this.tableRects[i].collidepoint(ev.pos)) {
            this.tableKey = TABLES[i];
            this.saveSetting("table", this.tableKey);
            this.playSound("click");
            return;
          }
        }
        for (let i = 0; i < this.ballRects.length; i++) {
          if (this.ballRects[i].collidepoint(ev.pos)) {
            this.ballCount = BALL_COUNTS[i];
            this.saveSetting("balls", this.ballCount);
            this.playSound("select");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    cycleBalls() {
      const i = (BALL_COUNTS.indexOf(this.ballCount) + 1) % BALL_COUNTS.length;
      this.ballCount = BALL_COUNTS[i];
      this.saveSetting("balls", this.ballCount);
      this.playSound("select");
    }

    startPlay() {
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Eingabe
    isLeft(k) {
      return k === "Shift_L" || this.isAction(k, "left") || k === "Left";
    }
    isRight(k) {
      return k === "Shift_R" || this.isAction(k, "right") || k === "Right";
    }
    isLaunch(k) {
      return this.isAction(k, "action") || k === "space" || k === "Return";
    }

    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.restart();
          else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.gameOver = false;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ev.button === 1) {
          this.restart();
        }
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.isLeft(k)) this.flipUp.l = true;
        if (this.isRight(k)) this.flipUp.r = true;
        // gehaltene Taste (Browser-Wiederholung) stößt nicht erneut an
        if ((this.isAction(k, "up") || k === "Up") && !ev.repeat) this.nudge();
        if (this.isLaunch(k) && this.phase === "launch") this.charging = true;
      } else if (ev.kind === "keyup") {
        const k = ev.key;
        if (this.isLeft(k)) this.flipUp.l = false;
        if (this.isRight(k)) this.flipUp.r = false;
        if (this.isLaunch(k) && this.charging) this.launch();
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        if (this.phase === "launch") {
          this.charging = true;
        } else if (ev.pos[0] < Math.floor(this.width / 2)) {
          // Maus: linke Hälfte = linker Flipper, rechte Hälfte = rechter
          this.flipUp.l = true;
        } else {
          this.flipUp.r = true;
        }
      } else if (ev.kind === "mouseup" && ev.button === 1) {
        if (this.charging) this.launch();
        this.flipUp.l = this.flipUp.r = false;
      }
    }

    launch() {
      this.charging = false;
      if (this.phase !== "launch" || !this.balls.length) return;
      const b = this.balls[0];
      b.vy = -(95.0 + 175.0 * Math.max(0.1, this.plunger));
      b.vx = -PG.rand.uniform(0.0, 4.0);
      this.phase = "play";
      this.saveT = BALL_SAVE;
      this.plunger = 0.0;
      this.playSound("shoot");
      this.rumble(70);
    }

    nudge() {
      if (this.tilt || this.phase !== "play") return;
      this.nudges += 1;
      this.nudgeT = 4.0;
      this.shake = 1.0;
      for (const b of this.balls) {
        b.vx += PG.rand.uniform(-26.0, 26.0);
        b.vy -= 22.0;
      }
      this.playSound("hit");
      if (this.nudges > TILT_LIMIT) {
        this.tilt = true;
        this.say(t("pin.tilt"), 2.6);
        this.playSound("gameover");
      }
    }

    // ===================================================== Update / Physik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      for (const key of Object.keys(this.flash)) {
        this.flash[key] -= dt;
        if (this.flash[key] <= 0) delete this.flash[key];
      }
      if (this.shake > 0) this.shake = Math.max(0.0, this.shake - dt * 3.2);
      if (this.nudgeT > 0) {
        this.nudgeT -= dt;
        if (this.nudgeT <= 0) this.nudges = 0;
      }
      if (this.state !== PLAY) return;

      // Flipper weich zur Zielstellung fahren
      for (const side of ["l", "r"]) {
        const cur = this.flip[side];
        const up = this.flipUp[side] && !this.tilt;
        const target = up ? 1.0 : 0.0;
        const step = (FLIP_SPEED * dt) / 2.6;
        const nw = target > cur ? Math.min(target, cur + step) : Math.max(target, cur - step);
        this.omega[side] = ((nw - cur) / Math.max(dt, 1e-4)) * 0.95;
        this.flip[side] = nw;
      }

      if (this.phase === "launch") {
        if (this.charging) this.plunger = Math.min(1.0, this.plunger + dt * 0.85);
        return;
      }

      if (this.saveT > 0) this.saveT = Math.max(0.0, this.saveT - dt);
      for (const b of this.balls.slice()) {
        if (b.held > 0) {
          b.held -= dt;
          if (b.held <= 0) this.eject(b);
          continue;
        }
        this.stepBall(b, dt);
        this.unstick(b, dt);
      }
      if (!this.balls.length) {
        this.ballLost();
      } else if (this.backInLane()) {
        // Zu schwach abgeschossen: der Ball rollt in die Schussbahn zurück -
        // dann darf noch einmal geladen werden (wie am Automaten).
        const b = this.balls[0];
        b.x = LANE_X;
        b.y = 146.0;
        b.vx = b.vy = 0.0;
        this.phase = "launch";
        this.plunger = 0.0;
        this.charging = false;
      }
    }

    /**
     * Sicherheitsnetz: ein liegen gebliebener Ball bekommt einen Stups.
     * Bewegt sich die Kugel drei Sekunden lang kaum, stupst der Automat sie
     * an - so hängt keine Partie fest.
     */
    unstick(b, dt) {
      if (Math.hypot(b.x - b.sx, b.y - b.sy) > 1.0) {
        b.sx = b.x;
        b.sy = b.y;
        b.st = 0.0;
        return;
      }
      b.st += dt;
      if (b.st > 3.0) {
        b.vx += PG.rand.uniform(-45.0, 45.0);
        b.vy -= 25.0;
        b.sx = b.x;
        b.sy = b.y;
        b.st = 0.0;
      }
    }

    /** true, wenn der einzige Ball unten in der Schussbahn liegen bleibt. */
    backInLane() {
      if (this.phase !== "play" || this.balls.length !== 1) return false;
      const b = this.balls[0];
      return b.x > 86.0 && b.y > 138.0 && Math.abs(b.vy) < 26.0 && Math.abs(b.vx) < 26.0;
    }

    stepBall(b, dt) {
      const speed = Math.hypot(b.vx, b.vy);
      const steps = Math.max(2, Math.min(26, Math.trunc((speed * dt) / BR) + 2));
      const h = dt / steps;
      for (let s = 0; s < steps; s++) {
        b.vy += GRAV * h;
        const f = Math.max(0.0, 1.0 - DRAG * h);
        b.vx *= f;
        b.vy *= f;
        const sp = Math.hypot(b.vx, b.vy);
        if (sp > MAX_SPEED) {
          b.vx *= MAX_SPEED / sp;
          b.vy *= MAX_SPEED / sp;
        }
        b.x += b.vx * h;
        b.y += b.vy * h;
        this.collideWalls(b);
        this.collideFlippers(b);
        this.collideBumpers(b);
        this.collideSlings(b);
        this.collideTargets(b);
        this.checkLanes(b);
        if (this.checkSaucer(b)) return;
        if (b.y > DRAIN_Y) {
          this.drain(b);
          return;
        }
      }
    }

    // ------------------------------------------------------- Kollisionen
    /** Kreis gegen Strecke; true bei Treffer. */
    hitSeg(b, seg, radius, rest, kick = 0.0, vel = null) {
      const [px, py] = closestOnSeg(seg[0], seg[1], seg[2], seg[3], b.x, b.y);
      let dx = b.x - px, dy = b.y - py;
      let d = Math.hypot(dx, dy);
      const rad = BR + radius;
      if (d >= rad) return false;
      if (d < 1e-9) {
        dx = 0.0; dy = -1.0; d = 1.0;
      }
      const ux = dx / d, uy = dy / d;
      b.x = px + ux * rad;
      b.y = py + uy * rad;
      const vx0 = vel ? vel[0] : 0, vy0 = vel ? vel[1] : 0;
      let rvx = b.vx - vx0, rvy = b.vy - vy0;
      const dot = rvx * ux + rvy * uy;
      if (dot < 0) {
        rvx -= (1 + rest) * dot * ux;
        rvy -= (1 + rest) * dot * uy;
      }
      b.vx = rvx + vx0 + ux * kick;
      b.vy = rvy + vy0 + uy * kick;
      return true;
    }

    collideWalls(b) {
      for (const seg of this.table.walls) {
        if (this.hitSeg(b, seg, WALL_R, WALL_E)) {
          if (Math.abs(b.vx) + Math.abs(b.vy) > 90) this.playSound("bounce");
        }
      }
      if (b.x < 84.0) this.hitSeg(b, GATE, WALL_R, WALL_E); // Einbahn-Klappe nur vom Feld aus
    }

    /** Aktuelle Strecke (Drehpunkt -> Spitze) eines Flippers. */
    flipperSeg(side) {
      let pivot, a;
      if (side === "l") {
        pivot = [24.0, 140.0];
        const rest = PG.radians(26), up = PG.radians(-30);
        a = rest + (up - rest) * this.flip.l;
      } else {
        pivot = [76.0, 140.0];
        const rest = PG.radians(154), up = PG.radians(210);
        a = rest + (up - rest) * this.flip.r;
      }
      return [pivot[0], pivot[1], pivot[0] + Math.cos(a) * FLIP_LEN, pivot[1] + Math.sin(a) * FLIP_LEN];
    }

    collideFlippers(b) {
      for (const side of ["l", "r"]) {
        const seg = this.flipperSeg(side);
        const [px, py] = closestOnSeg(seg[0], seg[1], seg[2], seg[3], b.x, b.y);
        // Umfangsgeschwindigkeit am Kontaktpunkt (Drehrichtung beachten)
        const sign = side === "l" ? -1.0 : 1.0;
        const rx = px - seg[0], ry = py - seg[1];
        const w = this.omega[side] * sign * 3.1;
        if (this.hitSeg(b, seg, FLIP_R, 0.45, 0.0, [-ry * w, rx * w])) this.playSound("bounce");
      }
    }

    collideBumpers(b) {
      this.table.bumpers.forEach(([x, y, r], i) => {
        const dx = b.x - x, dy = b.y - y;
        const d = Math.hypot(dx, dy);
        const rad = r + BR;
        if (d >= rad || d < 1e-9) return;
        const ux = dx / d, uy = dy / d;
        b.x = x + ux * rad;
        b.y = y + uy * rad;
        const dot = b.vx * ux + b.vy * uy;
        if (dot < 0) {
          b.vx -= 1.5 * dot * ux;
          b.vy -= 1.5 * dot * uy;
        }
        b.vx += ux * BUMP_KICK;
        b.vy += uy * BUMP_KICK;
        this.flash["b" + i] = 0.22;
        this.award(this.jackpot ? 2500 : 100, this.jackpot ? t("pin.jackpot") : null);
        this.playSound(this.jackpot ? "point" : "bounce");
      });
    }

    collideSlings(b) {
      this.table.slings.forEach((seg, i) => {
        if (this.hitSeg(b, seg, WALL_R * 1.6, 0.35, SLING_KICK)) {
          this.flash["s" + i] = 0.18;
          this.award(50);
          this.playSound("bounce");
        }
      });
    }

    collideTargets(b) {
      const tg = this.table.targets;
      for (let i = 0; i < tg.length; i++) {
        if (this.targetsHit[i]) continue;
        const [x, y, w, h] = tg[i];
        const nx = Math.max(x, Math.min(b.x, x + w));
        const ny = Math.max(y, Math.min(b.y, y + h));
        const dx = b.x - nx, dy = b.y - ny;
        if (dx * dx + dy * dy >= BR * BR) continue;
        const d = Math.hypot(dx, dy) || 1.0;
        const ux = dx / d, uy = dy / d;
        b.x = nx + ux * BR;
        b.y = ny + uy * BR;
        const dot = b.vx * ux + b.vy * uy;
        if (dot < 0) {
          b.vx -= 1.4 * dot * ux;
          b.vy -= 1.4 * dot * uy;
        }
        this.targetsHit[i] = true;
        this.award(250);
        this.playSound("lock");
        if (this.targetsHit.every(Boolean)) {
          this.targetsHit = this.targetsHit.map(() => false);
          this.bankClears += 1;
          this.mult = Math.min(5, this.mult + 1);
          this.award(2500, t("pin.bank", { n: this.mult }));
          this.playSound("powerup");
        }
      }
    }

    checkLanes(b) {
      const ln = this.table.lanes;
      for (let i = 0; i < ln.length; i++) {
        if (this.lanesHit[i]) continue;
        const [x, y, r] = ln[i];
        if ((b.x - x) ** 2 + (b.y - y) ** 2 < (r + BR) ** 2) {
          this.lanesHit[i] = true;
          this.award(200);
          this.playSound("eat");
          if (this.lanesHit.every(Boolean)) {
            this.lanesHit = this.lanesHit.map(() => false);
            this.mult = Math.min(5, this.mult + 1);
            this.award(1000, t("pin.lanes", { n: this.mult }));
            this.playSound("level");
          }
        }
      }
    }

    checkSaucer(b) {
      const [x, y, r] = this.table.saucer;
      if ((b.x - x) ** 2 + (b.y - y) ** 2 > r * r) return false;
      if (Math.hypot(b.vx, b.vy) > 110) return false;
      b.x = x;
      b.y = y;
      b.vx = b.vy = 0.0;
      b.held = 1.1;
      if (this.multiball) {
        this.award(5000, t("pin.jackpot"));
        this.playSound("win");
        return true;
      }
      this.locks += 1;
      if (this.locks >= 3) {
        this.startMultiball(b);
      } else {
        this.award(1000, t("pin.lock", { n: this.locks }));
        this.playSound("powerup");
      }
      return true;
    }

    startMultiball(b) {
      this.multiball = true;
      this.jackpot = true;
      this.locks = 0;
      b.held = 1.4;
      this.award(5000, t("pin.multiball"));
      this.achEvent("pin_multiball");
      this.playSound("win");
    }

    /** Wirft den Ball aus dem Saucer - im Multiball gleich mit Nachschub. */
    eject(b) {
      b.vx = PG.rand.uniform(-40.0, 40.0);
      b.vy = -150.0;
      b.held = 0.0;
      if (this.multiball && this.balls.length < 3) {
        const n = 3 - this.balls.length;
        for (let i = 0; i < n; i++) {
          this.balls.push(makeBall(b.x + PG.rand.uniform(-4, 4), b.y, PG.rand.uniform(-60, 60), -130.0));
        }
        this.saveT = Math.max(this.saveT, 4.0);
      }
      this.playSound("shoot");
    }

    // ------------------------------------------------------- Ballverlust
    drain(b) {
      const i = this.balls.indexOf(b);
      if (i >= 0) this.balls.splice(i, 1);
      if (this.balls.length) {
        if (this.balls.length === 1) {
          // Multiball vorbei
          this.multiball = false;
          this.jackpot = false;
        }
        this.playSound("hit");
        return;
      }
      if (this.saveT > 0 && !this.tilt) {
        this.balls = [makeBall(LANE_X, 146.0)];
        this.phase = "launch";
        this.plunger = 0.0;
        this.say(t("pin.saved"), 1.8);
        this.playSound("powerup");
        return;
      }
      this.playSound("gameover");
    }

    /** Wird gerufen, wenn kein Ball mehr im Spiel ist. */
    ballLost() {
      const bonus = 250 * this.mult;
      this.scores[this.player] += bonus;
      this.syncScore();
      this.ballsLeft[this.player] -= 1;
      if (Math.max(...this.ballsLeft) <= 0) {
        this.endGame();
        return;
      }
      this.newBall();
      this.say(t("pin.bonus", { n: bonus }), 2.2);
    }

    endGame() {
      this.saveBest(this.scores[0]);
      if (this.scores[0] >= 50000) this.achEvent("pin_high", this.scores[0]);
      this.reportResult(this.scores[0] >= (this.best[this.tableKey] || 0));
      this.syncScore();
      this.state = OVER;
      this.gameOver = true;
      this.playSound("gameover");
    }

    restart() {
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
    }

    // ------------------------------------------------------- Hilfsfunktionen
    award(points, message = null) {
      this.scores[this.player] += points * this.mult;
      this.syncScore();
      if (message) this.say(message, 1.8);
    }

    syncScore() {
      this.score = this.scores[0];
    }

    say(text, secs) {
      this.msg = text;
      this.msgT = secs;
    }

    // ===================================================== Zeichnen
    shakeX() {
      return this.shake > 0 ? Math.sin(ui.ticks() / 22.0) * 4 * this.shake : 0.0;
    }

    /** Tisch-Koordinaten -> Bildschirm (ohne Wackeln; das verschiebt draw()). */
    project(x, y) {
      return [this.ox + x * this.scale, this.oy + y * this.scale];
    }

    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      ctx.save();
      ctx.translate(this.shakeX(), 0);
      this.drawTable(ctx);
      ctx.restore();
      this.drawHud(ctx);
      this.drawSide(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    /** Statischer Tisch (Brett, Lichtkegel, Banden, Klappe, Saucer) - gecacht. */
    tableLayer() {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = this.tableKey + "@" + ps;
      if (this.tableCache && this.tableCache.key === key) return this.tableCache.canvas;
      const c = ui.makeCanvas(this.width * ps, this.height * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      const tb = TABLE_DATA[this.tableKey];
      const board = new PG.Rect(Math.trunc(this.ox), Math.trunc(this.oy), Math.trunc(TW * this.scale), Math.trunc(TH * this.scale));
      draw.rect(g, COL_TABLE, board, 0, 8);
      draw.rect(g, COL_TABLE_D, board, 2, 8);
      // dezenter Lichtkegel oben
      draw.ellipse(g, [70, 90, 150, 46], [board.x, board.y, board.w, board.h >> 1]);

      const ww = Math.max(2, Math.trunc(WALL_R * 2 * this.scale));
      for (const seg of tb.walls) {
        thickLine(g, COL_WALL, this.project(seg[0], seg[1]), this.project(seg[2], seg[3]), ww);
      }
      for (const seg of tb.walls) {
        draw.line(g, COL_WALL_HI, this.project(seg[0], seg[1]), this.project(seg[2], seg[3]), 1);
      }
      draw.line(g, COL_WALL_HI, this.project(GATE[0], GATE[1]), this.project(GATE[2], GATE[3]), 1);

      // Saucer
      const [sx, sy, sr] = tb.saucer;
      const [px, py] = this.project(sx, sy);
      const rr = Math.max(3, Math.trunc(sr * this.scale));
      draw.circle(g, [12, 14, 22], [px, py], rr);
      draw.circle(g, COL_SAUCER, [px, py], rr, 2);
      this.tableCache = { key, canvas: c };
      return c;
    }

    drawTable(ctx) {
      ctx.drawImage(this.tableLayer(), 0, 0, this.width, this.height);

      // Rollover-Bahnen mit Buchstaben
      this.table.lanes.forEach(([x, y, r], i) => {
        const [px, py] = this.project(x, y);
        const rr = Math.max(3, Math.trunc(r * this.scale));
        const on = this.lanesHit[i];
        draw.circle(ctx, on ? COL_LANE : [58, 44, 72], [px, py], rr);
        draw.circle(ctx, COL_LANE, [px, py], rr, 1);
        const ch = LANE_LETTERS[i % LANE_LETTERS.length];
        ui.text(ctx, ch, px, py, this.tiny, on ? [20, 16, 28] : COL_LANE, "center");
      });

      // Drop-Targets
      this.table.targets.forEach((r, i) => {
        const [x, y] = this.project(r[0], r[1]);
        const rc = [x, y, Math.max(2, Math.trunc(r[2] * this.scale)), Math.max(2, Math.trunc(r[3] * this.scale))];
        const col = this.targetsHit[i] ? COL_TARGET_OFF : COL_TARGET;
        draw.rect(ctx, col, rc, 0, 2);
        draw.rect(ctx, ui.mix(col, [255, 255, 255], 0.3), rc, 1, 2);
      });

      // Slingshots
      const sw = Math.max(3, Math.trunc(WALL_R * 3.4 * this.scale));
      this.table.slings.forEach((seg, i) => {
        const hot = "s" + i in this.flash;
        thickLine(ctx, hot ? [255, 236, 200] : COL_SLING, this.project(seg[0], seg[1]), this.project(seg[2], seg[3]), sw);
      });

      // Pop-Bumper
      this.table.bumpers.forEach(([x, y, r], i) => {
        const [px, py] = this.project(x, y);
        const rr = Math.max(4, Math.trunc(r * this.scale));
        const hot = "b" + i in this.flash;
        draw.circle(ctx, COL_BUMP, [px, py], rr);
        draw.circle(ctx, hot ? [255, 255, 255] : COL_BUMP_HI, [px, py], Math.max(2, Math.trunc(rr * 0.55)));
        draw.circle(ctx, COL_BUMP_HI, [px, py], rr, 2);
      });

      this.drawFlippers(ctx);
      this.drawPlunger(ctx);
      for (const b of this.balls) this.drawBall(ctx, b);
    }

    drawFlippers(ctx) {
      for (const side of ["l", "r"]) {
        const seg = this.flipperSeg(side);
        const a = this.project(seg[0], seg[1]), b = this.project(seg[2], seg[3]);
        const w = Math.max(4, Math.trunc(FLIP_R * 2 * this.scale));
        const col = this.tilt ? COL_FLIP_D : COL_FLIP;
        thickLine(ctx, col, a, b, w);
        draw.circle(ctx, COL_FLIP_D, a, Math.max(2, w >> 1));
        draw.circle(ctx, ui.mix(col, [255, 255, 255], 0.4), b, Math.max(2, Math.floor(w / 3)));
      }
    }

    drawPlunger(ctx) {
      if (this.phase !== "launch") return;
      const [x0, y0] = this.project(LANE_X - 3.4, 150.0);
      const [x1] = this.project(LANE_X + 3.4, 150.0);
      const h = Math.max(6, Math.trunc(16 * this.scale * (0.25 + this.plunger)));
      draw.rect(ctx, COL_PLUNGER, [Math.trunc(x0), Math.trunc(y0), Math.trunc(x1 - x0), h], 0, 3);
    }

    drawBall(ctx, b) {
      const [px, py] = this.project(b.x, b.y);
      const r = Math.max(2, Math.trunc(BR * this.scale));
      draw.circle(ctx, [8, 10, 16], [px + 2, py + 2], r);
      draw.circle(ctx, COL_BALL, [px, py], r);
      draw.circle(ctx, COL_BALL_D, [px, py], r, 1);
      if (r >= 4) draw.circle(ctx, [255, 255, 255], [px - Math.floor(r / 3), py - Math.floor(r / 3)], Math.max(1, Math.floor(r / 3)));
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = this.hudH >> 1;
      // Punktzahl ohne Tausendertrennung (schreibt sich je nach Sprache anders)
      ui.text(ctx, String(this.scores[this.player]), 12, cy, this.num, this.accent, "midleft");
      let mid, col;
      if (this.msg) [mid, col] = [this.msg, ui.GOLD];
      else if (this.tilt) [mid, col] = [t("pin.tilt"), ui.RED];
      else if (this.phase === "launch") [mid, col] = [t("pin.launch"), ui.TEXT_DIM];
      else [mid, col] = [t("pin.mult", { n: this.mult }), ui.TEXT_DIM];
      ui.text(ctx, mid, this.width >> 1, cy, this.small, col, "center");
      ui.text(ctx, t("pin.ball", { n: this.ballsLeft[this.player] }), this.width - 12, cy, this.tiny, ui.TEXT_DIM, "midright");
    }

    /** Schmale Info-Spalte rechts: Multiplikator, Ball-Save, Locks. */
    drawSide(ctx) {
      const x = this.sideX;
      const w = this.width - x - 6;
      if (w < 70) return;
      const y = this.hudH + 10;
      ui.drawPanel(ctx, [x, y, w, 116], { radius: 8, shadow: false });
      let cy = y + 10;
      const rows = [
        [t("pin.lbl_mult"), "x" + this.mult, this.accent],
        [t("pin.lbl_lock"), this.locks + "/3", COL_SAUCER],
        [t("pin.lbl_bank"), String(this.bankClears), COL_TARGET],
      ];
      if (this.saveT > 0) rows.push([t("pin.lbl_save"), Math.round(this.saveT) + "s", ui.GREEN]);
      else if (this.multiball) rows.push([t("pin.lbl_mode"), t("pin.multiball_short"), ui.GOLD]);
      for (const [label, value, col] of rows) {
        ui.text(ctx, label, x + 8, cy, this.tiny, ui.TEXT_DIM);
        ui.text(ctx, value, x + w - 8, cy, this.tiny, col, "topright");
        cy += this.tiny.height + 8;
      }
    }

    drawOver(ctx) {
      const h = 118;
      const y = (this.height >> 1) - (h >> 1);
      draw.rect(ctx, [10, 12, 20, 214], [0, y, this.width, h]);
      draw.line(ctx, this.accent, [0, y + 0.5], [this.width, y + 0.5]);
      draw.line(ctx, this.accent, [0, y + h - 0.5], [this.width, y + h - 0.5]);
      const cx = this.width >> 1;
      ui.text(ctx, t("common.game_over"), cx, y + 32, this.huge, this.accent, "center");
      ui.text(ctx, t("common.points", { score: this.scores[0] }), cx, y + 66, this.small, ui.TEXT, "center");
      const best = this.best[this.tableKey];
      if (best) ui.text(ctx, t("pin.best", { n: best }), cx, y + 88, this.tiny, ui.GOLD, "center");
      ui.text(ctx, t("pin.new_round"), cx, y + 106, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = this.width >> 1;
      ui.text(ctx, t("pin.title"), cx, Math.floor(this.height * 0.13), this.huge, this.accent, "center");
      ui.text(ctx, t("pin.subtitle"), cx, Math.floor(this.height * 0.2), this.small, ui.TEXT_DIM, "center");

      const label = (rects, txt) => ui.text(ctx, txt, cx, rects[0].top - 4, this.tiny, ui.TEXT_DIM, "midbottom");

      label(this.tableRects, t("pin.lbl_table"));
      this.tableRects.forEach((rc, i) => this.btn(ctx, rc, t("pin.table." + TABLES[i]), this.tableKey === TABLES[i]));
      label(this.ballRects, t("pin.lbl_balls"));
      this.ballRects.forEach((rc, i) => this.btn(ctx, rc, String(BALL_COUNTS[i]), this.ballCount === BALL_COUNTS[i]));
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      const best = this.best[this.tableKey];
      if (best) ui.text(ctx, t("pin.best", { n: best }), cx, this.startRect.bottom + 22, this.tiny, ui.GOLD, "center");
      ui.text(ctx, t("pin.setup_hint"), cx, this.height - 14, this.tiny, ui.TEXT_DIM, "center");
    }

    btn(ctx, rc, text, on) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      ui.text(ctx, text, rc.centerx, rc.centery, this.small, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }
  }

  PG.register(PinballGame, {
    id: "PinballGame",
    key: "pinball",
    name: "Pinball",
    settingsKey: "pinball",
    defaults: { table: "classic", balls: 3 },
  });
})();
