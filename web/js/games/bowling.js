/*
 * bowling.js - Bowling (Port von games/bowling.py, Einzelspieler)
 * ================================================================
 * Zehn Frames mit vollständiger Strike-/Spare-Wertung, echter Pin-Physik und
 * perspektivischer Bahnansicht.
 *
 * Ein Wurf läuft in vier kurzen Schritten, die je mit der Aktionstaste
 * (Leertaste/Enter) oder einem Klick festgelegt werden:
 *
 *     1. Position : Standpunkt an der Foullinie
 *     2. Ziel     : Wurfwinkel
 *     3. Effet    : Drall nach links/rechts (der Hook greift erst hinten,
 *                   wenn das Öl auf der Bahn ausläuft)
 *     4. Kraft    : Wurfgeschwindigkeit
 *
 * Jeder Regler pendelt von allein - wer lieber genau zielt, stellt ihn mit den
 * Links-/Rechts-Tasten selbst ein (das Pendeln hält dann an).
 *
 * Die zehn Pins werden als Kreise mit Masse simuliert. Auf *Pro* pendeln die
 * Regler schneller und die Bahn streut minimal stärker.
 *
 * Punkte (Highscore) = Spielsumme nach offiziellen Regeln (maximal 300).
 * (Replays und der 2-Spieler-Modus der Desktop-Version entfallen.)
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ------------------------------------------------- Identitätsfarben (Bahn)
  const COL_LANE = [206, 164, 104];
  const COL_BOARD = [196, 152, 96];
  const COL_GUTTER = [46, 50, 62];
  const COL_GUTTER_D = [32, 35, 45];
  const COL_FOUL = [208, 72, 72];
  const COL_ARROW = [150, 108, 62];
  const COL_DECK = [232, 228, 220];
  const COL_BACK = [28, 32, 46];
  const COL_PIN = [246, 246, 242];
  const COL_PIN_D = [198, 198, 192];
  const COL_PIN_RED = [216, 60, 60];
  const COL_BALL = [58, 74, 176];
  const COL_BALL_HI = [120, 140, 236];
  const COL_MARK = [250, 224, 120];

  // --------------------------------------------------------- Bahn / Physik
  // Alle Maße in Zoll - so stimmen die Verhältnisse mit echten Bahnen überein.
  const LW = 41.5; // Bahnbreite
  const GUT = 9.0; // Rinne je Seite
  const PIN_Y = 720.0; // Kopfpin, gemessen ab Foullinie
  const ROW_DY = 10.392; // Reihenabstand (12 * cos 30°)
  const PIN_DX = 6.0; // halber Abstand benachbarter Pins
  const BALL_R = 4.25;
  const PIN_R = 2.4;
  const PIN_M = 3.5;
  const BALL_M = 16.0;
  const OIL_END = 430.0; // ab hier greift der Hook
  const HOOK = 300.0; // seitliche Beschleunigung bei vollem Effet
  const PIN_FRIC = 3.4;
  const BALL_FRIC = 0.1;
  const REST = 0.55;
  const DOWN_DIST = 1.6; // so weit verschoben gilt ein Pin als gefallen
  const MAX_ROLL_TIME = 7.0;
  const PIT_Y = 810.0;

  const SETUP = "setup", PLAY = "play", OVER = "over";
  const DIFFS = ["easy", "normal", "pro"];
  const STEPS = ["pos", "aim", "spin", "power"];
  // Pendelgeschwindigkeit der vier Regler je Schwierigkeit
  const SWING = { easy: 0.72, normal: 1.05, pro: 1.45 };
  const SCATTER = { easy: 0.2, normal: 0.45, pro: 0.8 };

  // Standard-Aufstellung: [Pin-Nummer, x, y] - Kopfpin zuerst
  const PIN_SETUP = [
    [1, 0.0, PIN_Y],
    [2, -PIN_DX, PIN_Y + ROW_DY], [3, PIN_DX, PIN_Y + ROW_DY],
    [4, -2 * PIN_DX, PIN_Y + 2 * ROW_DY], [5, 0.0, PIN_Y + 2 * ROW_DY], [6, 2 * PIN_DX, PIN_Y + 2 * ROW_DY],
    [7, -3 * PIN_DX, PIN_Y + 3 * ROW_DY], [8, -PIN_DX, PIN_Y + 3 * ROW_DY],
    [9, PIN_DX, PIN_Y + 3 * ROW_DY], [10, 3 * PIN_DX, PIN_Y + 3 * ROW_DY],
  ];

  function makePin(num, x, y) {
    return { num, x, y, hx: x, hy: y, vx: 0, vy: 0, down: false, spin: 0 };
  }
  const pinMoved = (p) => Math.hypot(p.x - p.hx, p.y - p.hy);

  /** Offizielle Wertung; liefert je Frame die laufende Summe (null = offen). */
  function scoreFrames(rolls) {
    const out = [];
    let total = 0;
    let i = 0;
    for (let f = 0; f < 10; f++) {
      if (i >= rolls.length) {
        out.push(null);
        continue;
      }
      if (rolls[i] === 10) { // Strike
        if (i + 2 < rolls.length) {
          total += 10 + rolls[i + 1] + rolls[i + 2];
          out.push(total);
        } else out.push(null);
        i += 1;
      } else if (i + 1 < rolls.length) {
        if (rolls[i] + rolls[i + 1] === 10) { // Spare
          if (i + 2 < rolls.length) {
            total += 10 + rolls[i + 2];
            out.push(total);
          } else out.push(null);
        } else {
          total += rolls[i] + rolls[i + 1];
          out.push(total);
        }
        i += 2;
      } else {
        out.push(null);
        i += 1;
      }
    }
    return out;
  }

  /** Aktuelle Gesamtsumme (letzte gewertete Frame-Summe). */
  function totalScore(rolls) {
    const vals = scoreFrames(rolls).filter((v) => v !== null);
    return vals.length ? vals[vals.length - 1] : 0;
  }

  class BowlingGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.diff = this.opts.difficulty;
      if (!DIFFS.includes(this.diff)) this.diff = "normal";
      this.guide = !!this.opts.guide;
      this.msg = null;
      this.msgT = 0;
      this.buildFonts();
      this.layout();
      this.buildSetupLayout();
      this.best = this.loadBest();
      this.newGame();
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, Math.floor(h / 32)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 42)));
      this.cardFont = ui.font(Math.max(10, Math.floor(h / 44)), false, true);
      this.huge = ui.font(Math.max(24, Math.floor(h / 13)), true);
    }

    /** Perspektive: Kamera hinter der Foullinie, Fluchtpunkt über der Bahn. */
    layout() {
      this.hudH = 40;
      this.cardH = Math.max(34, this.cardFont.height * 2 + 12);
      const top = this.hudH + this.cardH + 6;
      const nearY = this.height - 12;
      const farY = top + 18;
      this.cx = this.width / 2;
      this.depth = 340.0;
      // Fluchtpunkt aus Nah-/Fernkante ableiten
      const k = (PIT_Y + this.depth) / this.depth;
      this.hor = (k * farY - nearY) / (k - 1.0);
      this.ky = (nearY - this.hor) * this.depth;
      // Querskalierung: die Bahn samt Rinnen füllt vorn ~86 % der Breite
      const half = LW / 2 + GUT;
      this.kx = (this.width * 0.43 * this.depth) / half;
    }

    // ------------------------------------------------------- Speicherstand
    loadBest() {
      const data = PG.store.get("bowling", {});
      const best = data && typeof data === "object" ? data.best : null;
      const out = {};
      if (best && typeof best === "object") {
        for (const k in best) {
          const v = parseInt(best[k], 10);
          if (!isNaN(v)) out[String(k)] = v;
        }
      }
      return out;
    }

    saveBest(score) {
      if (score > (this.best[this.diff] || 0)) {
        this.best[this.diff] = Math.floor(score);
        PG.store.set("bowling", { best: this.best });
      }
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ------------------------------------------------------- Partie
    newGame() {
      this.rolls = [];
      this.frame = 0;
      this.strikeRun = 0;
      this.score = 0;
      this.gameOver = false;
      this.msg = null;
      this.msgT = 0;
      this.rack(true);
      this.newDelivery();
    }

    rack(full = true) {
      // Frisch aufgestellt? Nur dann kann der nächste Ball ein Strike sein.
      this.rackFresh = full;
      if (full) {
        this.pins = PIN_SETUP.map(([n, x, y]) => makePin(n, x, y));
      } else {
        this.pins = this.pins.filter((p) => !p.down);
        for (const p of this.pins) {
          p.x = p.hx;
          p.y = p.hy;
          p.vx = p.vy = 0;
          p.spin = 0;
        }
      }
    }

    newDelivery() {
      this.step = 0;
      this.tOsc = 0;
      this.manual = false;
      this.pos = 0; // -1 .. 1 (Standpunkt quer zur Bahn)
      this.aim = 0; // -1 .. 1 (Winkel)
      this.spin = 0; // -1 .. 1 (Effet)
      this.power = 0.5; // 0 .. 1
      this.ball = null;
      this.rollT = 0;
      this.pinsBefore = this.pins.filter((p) => !p.down).length;
      this.resultKey = null;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(370, this.width - 50);
      const y0 = Math.floor(this.height * 0.3);
      const gap = 8;
      const row = (y, n) => {
        const cw = (bw - gap * (n - 1)) / n;
        const out = [];
        for (let i = 0; i < n; i++) out.push(new PG.Rect(Math.floor(cx - bw / 2 + i * (cw + gap)), y, Math.floor(cw), 42));
        return out;
      };
      this.diffRects = row(y0, 3);
      this.guideRects = row(y0 + 88, 2);
      this.startRect = new PG.Rect(cx - 95, y0 + 156, 190, 46);
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diff = DIFFS[Number(k) - 1];
          this.saveSetting("difficulty", this.diff);
          this.playSound("click");
        } else if (k === "g" || k === "G") {
          this.toggleGuide();
        } else if (k === "Return" || k === "space") {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diff = DIFFS[i];
            this.saveSetting("difficulty", this.diff);
            this.playSound("click");
            return;
          }
        }
        for (let i = 0; i < this.guideRects.length; i++) {
          if (this.guideRects[i].collidepoint(ev.pos)) {
            if (this.guide !== (i === 0)) this.toggleGuide();
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    toggleGuide() {
      this.guide = !this.guide;
      this.saveSetting("guide", this.guide);
      this.playSound("select");
    }

    startPlay() {
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (ev.kind === "keydown" && (ev.key === "g" || ev.key === "G")) {
        this.toggleGuide();
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
      if (this.state !== PLAY || this.ball !== null) return;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || this.isAction(k, "left")) this.adjust(-0.06);
        else if (k === "Right" || this.isAction(k, "right")) this.adjust(0.06);
        else if (k === "space" || k === "Return" || this.isAction(k, "action")) this.confirmStep();
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.confirmStep();
      }
    }

    /** Regler von Hand verstellen (stoppt das Pendeln). */
    adjust(d) {
      this.manual = true;
      const name = STEPS[this.step];
      const lo = name === "power" ? 0 : -1;
      this[name] = Math.max(lo, Math.min(1, this[name] + d));
      this.playSound("move");
    }

    confirmStep() {
      this.playSound("click");
      this.step += 1;
      this.manual = false;
      this.tOsc = 0;
      if (this.step >= STEPS.length) this.throwBall();
    }

    throwBall() {
      const scatter = SCATTER[this.diff];
      const x0 = this.pos * (LW / 2 - BALL_R - 1.0);
      const ang = PG.radians(this.aim * 4.2 + PG.rand.uniform(-scatter, scatter) * 0.6);
      const speed = 240.0 + 210.0 * this.power;
      this.ball = {
        x: x0, y: 0,
        vx: Math.sin(ang) * speed,
        vy: Math.cos(ang) * speed,
        spin: this.spin + PG.rand.uniform(-scatter, scatter) * 0.1,
        gutter: false, roll: 0,
      };
      this.rollT = 0;
      this.playSound("shoot");
      this.rumble(80);
    }

    // ===================================================== Update / Physik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state !== PLAY) return;
      if (this.ball === null) {
        if (!this.manual) {
          this.tOsc += dt * SWING[this.diff];
          const name = STEPS[this.step];
          const w = Math.sin(this.tOsc * 2.4);
          this[name] = name === "power" ? Math.abs(w) : w;
        }
        return;
      }
      this.rollT += dt;
      this.stepBall(dt);
      this.stepPins(dt);
      if (this.rollDone()) this.finishRoll();
    }

    stepBall(dt) {
      const b = this.ball;
      const speed = Math.hypot(b.vx, b.vy) + 1.0;
      const steps = Math.max(2, Math.min(24, Math.floor((speed * dt) / PIN_R) + 2));
      const h = dt / steps;
      for (let s = 0; s < steps; s++) {
        if (!b.gutter && b.y > OIL_END) {
          // Der Hook greift, sobald das Öl ausläuft
          const grip = Math.min(1, (b.y - OIL_END) / 180.0);
          b.vx += b.spin * HOOK * grip * h;
        }
        const f = Math.max(0, 1 - BALL_FRIC * h);
        b.vx *= f;
        b.vy *= f;
        b.x += b.vx * h;
        b.y += b.vy * h;
        b.roll += (b.vy * h) / 24.0;
        const limit = LW / 2 - BALL_R;
        if (!b.gutter && Math.abs(b.x) > limit) {
          b.gutter = true;
          b.vx = 0;
          b.spin = 0;
          b.x = Math.sign(b.x) * (LW / 2 + GUT / 2 - BALL_R * 0.4);
          this.playSound("hit");
        }
        if (!b.gutter) for (const p of this.pins) this.hitPin(b, p);
      }
    }

    hitPin(b, p) {
      const dx = p.x - b.x, dy = p.y - b.y;
      const d = Math.hypot(dx, dy);
      const rad = BALL_R + PIN_R;
      if (d >= rad || d < 1e-9) return;
      const ux = dx / d, uy = dy / d;
      const overlap = rad - d;
      p.x += ux * overlap;
      p.y += uy * overlap;
      const rvx = p.vx - b.vx, rvy = p.vy - b.vy;
      const vn = rvx * ux + rvy * uy;
      if (vn > 0) return;
      const j = (-(1 + REST) * vn) / (1 / PIN_M + 1 / BALL_M);
      p.vx += (j * ux) / PIN_M;
      p.vy += (j * uy) / PIN_M;
      b.vx -= (j * ux) / BALL_M;
      b.vy -= (j * uy) / BALL_M;
      p.spin += PG.rand.uniform(-3, 3);
      if (!p.down) this.playSound("lock");
    }

    stepPins(dt) {
      const steps = 3;
      const h = dt / steps;
      const pins = this.pins;
      for (let s = 0; s < steps; s++) {
        for (const p of pins) {
          if (p.vx === 0 && p.vy === 0) continue;
          const f = Math.max(0, 1 - PIN_FRIC * h);
          p.vx *= f;
          p.vy *= f;
          p.x += p.vx * h;
          p.y += p.vy * h;
          if (Math.abs(p.vx) + Math.abs(p.vy) < 1.2) p.vx = p.vy = 0;
        }
        for (let i = 0; i < pins.length; i++) {
          for (let k = i + 1; k < pins.length; k++) pinPin(pins[i], pins[k]);
        }
        for (const p of pins) if (!p.down && pinMoved(p) > DOWN_DIST) p.down = true;
      }
    }

    rollDone() {
      const b = this.ball;
      if (b.y > PIT_Y) return this.pins.every((p) => p.vx === 0 && p.vy === 0);
      return this.rollT > MAX_ROLL_TIME;
    }

    // ===================================================== Wurf auswerten
    finishRoll() {
      const knocked = this.pins.filter((p) => p.down).length;
      const standing = this.pinsBefore - knocked;
      this.rolls.push(knocked);
      const rolls = this.rolls;
      const frame = this.frame;
      // Strike nur mit dem ersten Ball auf eine frische Aufstellung - wer erst
      // nichts trifft und dann alle 10 abräumt, hat einen Spare (war auch im
      // Python-Original als Strike gezählt, inkl. Turkey-Erfolg).
      const strike = knocked === 10 && this.pinsBefore === 10 && this.rackFresh;
      const spare = !strike && standing === 0;

      if (strike) {
        this.strikeRun += 1;
        this.resultKey = "bowl.strike";
        this.playSound("win");
        if (this.strikeRun >= 3) this.achEvent("bowl_turkey");
      } else if (spare) {
        this.strikeRun = 0;
        this.resultKey = "bowl.spare";
        this.playSound("level");
      } else {
        this.strikeRun = 0;
        this.resultKey = knocked ? "bowl.pins" : "bowl.miss";
        this.playSound(knocked ? "point" : "hit");
      }
      this.say(t(this.resultKey, { n: knocked }), 1.8);

      const [done, refill] = this.frameState(frame, rolls, strike, spare);
      this.ball = null;
      if (done) {
        this.frame += 1;
        if (this.frame >= 10) {
          this.endGame();
          return;
        }
        this.rack(true);
        this.newDelivery();
      } else {
        this.rack(refill);
        this.newDelivery();
      }
      this.syncScore();
    }

    /** [frame_fertig, neue_volle_aufstellung] nach dem aktuellen Wurf. */
    frameState(frame, rolls, strike, spare) {
      const inFrame = rollsInFrame(rolls, frame);
      if (frame < 9) {
        if (strike) return [true, true];
        return [inFrame >= 2, false];
      }
      // Zehntes Frame: Bonuswürfe nach Strike/Spare
      if (inFrame === 1) return [false, strike];
      if (inFrame === 2) {
        const first = rolls[rolls.length - 2], second = rolls[rolls.length - 1];
        if (first === 10) {
          // Nach dem Strike gibt es zwei Bonuswürfe; neu aufgestellt
          // wird nur, wenn auch der zweite Ball alle Pins geräumt hat.
          return [false, second === 10];
        }
        if (first + second === 10) return [false, true];
        return [true, true];
      }
      return [true, true];
    }

    endGame() {
      const final = totalScore(this.rolls);
      this.score = final;
      this.saveBest(final);
      if (final >= 200) this.achEvent("bowl_200", final);
      this.reportResult(final >= 100);
      this.state = OVER;
      this.gameOver = true;
      this.playSound("gameover");
    }

    restart() {
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
    }

    /**
     * Name des aktiven Reglers. Nach dem letzten Wurf einer Partie steht
     * step auf STEPS.length - der Wert wird deshalb gedeckelt.
     */
    stepName() {
      return STEPS[Math.min(this.step, STEPS.length - 1)];
    }

    syncScore() {
      this.score = totalScore(this.rolls);
    }

    say(text, secs) {
      this.msg = text;
      this.msgT = secs;
    }

    // ===================================================== Projektion
    project(x, y, z = 0) {
      const inv = 1.0 / (y + this.depth);
      const s = this.kx * inv;
      return [this.cx + x * s, this.hor + this.ky * inv - z * s, s];
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawLane(ctx);
      this.drawPins(ctx);
      if (this.ball !== null) this.drawBall(ctx);
      else if (this.state === PLAY) this.drawControls(ctx);
      this.drawHud(ctx);
      this.drawCard(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawLane(ctx) {
      // Rückwand
      const far = this.project(0, PIT_Y);
      draw.rect(ctx, COL_BACK, [0, this.hudH + this.cardH + 4, this.width, far[1] - this.hudH - this.cardH]);
      // Rinnen + Bahn als Trapeze
      const quad = (x0, x1, col) => {
        const a = this.project(x0, 0), b = this.project(x1, 0);
        const c = this.project(x1, PIT_Y), d = this.project(x0, PIT_Y);
        draw.polygon(ctx, col, [[a[0], a[1]], [b[0], b[1]], [c[0], c[1]], [d[0], d[1]]]);
      };
      quad(-LW / 2 - GUT, -LW / 2, COL_GUTTER);
      quad(LW / 2, LW / 2 + GUT, COL_GUTTER);
      quad(-LW / 2, LW / 2, COL_LANE);
      // Bretter
      for (let i = 1; i < 10; i++) {
        const x = -LW / 2 + (LW * i) / 10;
        const a = this.project(x, 0), b = this.project(x, PIT_Y);
        draw.line(ctx, COL_BOARD, [a[0], a[1]], [b[0], b[1]], 1);
      }
      // Pin-Deck
      {
        const a = this.project(-LW / 2, PIN_Y - 26), b = this.project(LW / 2, PIN_Y - 26);
        const c = this.project(LW / 2, PIT_Y), d = this.project(-LW / 2, PIT_Y);
        draw.polygon(ctx, COL_DECK, [[a[0], a[1]], [b[0], b[1]], [c[0], c[1]], [d[0], d[1]]]);
      }
      // Pfeile
      for (let i = -3; i < 4; i++) {
        const x = i * 5.0;
        const y = 180.0 + Math.abs(i) * 22.0;
        const p0 = this.project(x, y), p1 = this.project(x - 1.6, y - 16), p2 = this.project(x + 1.6, y - 16);
        draw.polygon(ctx, COL_ARROW, [[p0[0], p0[1]], [p1[0], p1[1]], [p2[0], p2[1]]]);
      }
      // Foullinie
      const fa = this.project(-LW / 2 - GUT, 0), fb = this.project(LW / 2 + GUT, 0);
      draw.line(ctx, COL_FOUL, [fa[0], fa[1]], [fb[0], fb[1]], 3);
      draw.line(ctx, COL_GUTTER_D, [fa[0], fa[1] + 3], [fb[0], fb[1] + 3], 1);
    }

    drawPins(ctx) {
      const sorted = this.pins.slice().sort((a, b) => b.y - a.y);
      for (const p of sorted) {
        if (p.down && p.vx === 0 && p.vy === 0 && pinMoved(p) > 40) continue; // aus dem Bild gerutscht
        const [px, py, sc] = this.project(p.x, p.y);
        const r = Math.max(2, Math.floor(PIN_R * sc));
        if (p.down) {
          // liegender Pin: flache Ellipse in Fallrichtung
          const w = Math.max(3, Math.floor(PIN_R * 2.6 * sc));
          const h = Math.max(2, Math.floor(PIN_R * 1.3 * sc));
          draw.ellipse(ctx, COL_PIN_D, [px - Math.floor(w / 2), py - Math.floor(h / 2), w, h]);
          draw.ellipse(ctx, COL_PIN_RED, [px - Math.floor(w / 2), py - Math.floor(h / 2), w, Math.max(1, Math.floor(h / 3))]);
          continue;
        }
        const body = [
          [px - r, py], [px - r * 0.75, py - r * 2.2],
          [px - r * 0.42, py - r * 3.4], [px - r * 0.62, py - r * 4.6],
          [px, py - r * 5.6], [px + r * 0.62, py - r * 4.6],
          [px + r * 0.42, py - r * 3.4], [px + r * 0.75, py - r * 2.2],
          [px + r, py],
        ];
        draw.polygon(ctx, COL_PIN, body);
        draw.polygon(ctx, COL_PIN_D, body, 1);
        if (r >= 3) draw.line(ctx, COL_PIN_RED, [px - r * 0.55, py - r * 3.0], [px + r * 0.55, py - r * 3.0], Math.max(1, Math.floor(r / 2)));
      }
    }

    drawBall(ctx) {
      const b = this.ball;
      const [px, py, sc] = this.project(b.x, b.y, BALL_R);
      const r = Math.max(3, Math.floor(BALL_R * sc));
      const sh = this.project(b.x, b.y);
      draw.ellipse(ctx, [120, 96, 60], [sh[0] - r, sh[1] - r * 0.4, r * 2, r * 0.8]);
      draw.circle(ctx, COL_BALL, [Math.floor(px), Math.floor(py)], r);
      draw.circle(ctx, COL_BALL_HI, [Math.floor(px - r * 0.35), Math.floor(py - r * 0.35)], Math.max(1, Math.floor(r * 0.32)));
      // Fingerlöcher rotieren mit dem Ball
      if (r >= 6) {
        for (let k = 0; k < 3; k++) {
          const a = b.roll + k * 2.1;
          const hx = px + Math.cos(a) * r * 0.45;
          const hy = py + Math.sin(a) * r * 0.45 * 0.6;
          draw.circle(ctx, [18, 22, 60], [Math.floor(hx), Math.floor(hy)], Math.max(1, Math.floor(r * 0.14)));
        }
      }
    }

    /** Zielhilfe und der aktive Regler unten am Bild. */
    drawControls(ctx) {
      const x0 = this.pos * (LW / 2 - BALL_R - 1.0);
      const p0 = this.project(x0, 0);
      // Ball an der Foullinie
      const r = Math.max(3, Math.floor(BALL_R * p0[2]));
      draw.circle(ctx, COL_BALL, [Math.floor(p0[0]), Math.floor(p0[1] - r)], r);
      if (this.guide && this.step >= 1) {
        const ang = PG.radians(this.aim * 4.2);
        const pts = [];
        let x = x0, y = 0;
        let vx = Math.sin(ang);
        const vy = Math.cos(ang);
        const spin = this.step >= 2 ? this.spin : 0;
        const step = PIN_Y / 46.0;
        for (let i = 0; i < 46; i++) {
          if (y > OIL_END) vx += spin * 0.055 * Math.min(1, (y - OIL_END) / 180.0);
          x += vx * step;
          y += vy * step;
          const pr = this.project(Math.max(-LW, Math.min(LW, x)), y);
          pts.push([pr[0], pr[1]]);
        }
        for (let i = 0; i < pts.length - 1; i += 2) draw.line(ctx, COL_MARK, pts[i], pts[i + 1], 2);
      }
      // Reglerbalken
      const name = this.stepName();
      const val = this[name];
      const bw = Math.min(240, this.width - 60);
      const bx = Math.floor(this.width / 2) - Math.floor(bw / 2);
      const by = this.height - 26;
      draw.rect(ctx, ui.PANEL, [bx - 6, by - 16, bw + 12, 34], 0, 8);
      draw.rect(ctx, ui.BTN, [bx, by, bw, 10], 0, 5);
      if (name === "power") {
        draw.rect(ctx, this.accent, [bx, by, Math.floor(bw * val), 10], 0, 5);
      } else {
        const mid = bx + Math.floor(bw / 2);
        draw.line(ctx, ui.BORDER, [mid + 0.5, by - 3], [mid + 0.5, by + 13], 1);
        const mx = mid + Math.trunc((val * bw) / 2);
        draw.rect(ctx, this.accent, [mx - 4, by - 3, 8, 16], 0, 3);
      }
      ui.text(ctx, t("bowl.step." + name), Math.floor(this.width / 2), by - 4, this.tiny, ui.TEXT, "midbottom");
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH + 0.5], [this.width, this.hudH + 0.5]);
      const cy = Math.floor(this.hudH / 2);
      ui.text(ctx, t("bowl.frame", { n: Math.min(10, this.frame + 1) }), 12, cy, this.tiny, ui.TEXT_DIM, "midleft");
      let mid, col;
      if (this.msg) {
        mid = this.msg;
        col = ui.GOLD;
      } else {
        mid = this.ball === null ? t("bowl.step." + this.stepName()) : t("bowl.rolling");
        col = ui.TEXT_DIM;
      }
      ui.text(ctx, mid, this.width / 2, cy, this.small, col, "center");
      ui.text(ctx, String(totalScore(this.rolls)), this.width - 12, cy, this.small, this.accent, "midright");
    }

    /** Scorecard: zehn Frames mit Würfen und laufender Summe. */
    drawCard(ctx) {
      const y = this.hudH + 2;
      const h = this.cardH - 4;
      draw.rect(ctx, ui.PANEL, [4, y, this.width - 8, h], 0, 6);
      draw.rect(ctx, ui.BORDER, [4, y, this.width - 8, h], 1, 6);
      const rolls = this.rolls;
      const sums = scoreFrames(rolls);
      const cells = frameCells(rolls);
      const fw = (this.width - 16) / 10;
      for (let f = 0; f < 10; f++) {
        const fx = 8 + f * fw;
        if (f) draw.line(ctx, ui.BORDER, [fx, y + 2], [fx, y + h - 2], 1);
        const active = f === this.frame;
        const marks = cells[f].join(" ");
        if (marks) ui.text(ctx, marks, fx + fw / 2, y + 3, this.cardFont, active ? ui.TEXT : ui.TEXT_DIM, "midtop");
        const val = sums[f];
        if (val !== null) ui.text(ctx, String(val), fx + fw / 2, y + h - 3, this.cardFont, active ? this.accent : ui.TEXT_DIM, "midbottom");
      }
    }

    drawOver(ctx) {
      const h = 118;
      const y = Math.floor(this.height / 2) - Math.floor(h / 2);
      draw.rect(ctx, [14, 12, 10, 214], [0, y, this.width, h]);
      draw.line(ctx, this.accent, [0, y + 0.5], [this.width, y + 0.5]);
      draw.line(ctx, this.accent, [0, y + h - 0.5], [this.width, y + h - 0.5]);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("bowl.game_done"), cx, y + 32, this.huge, this.accent, "center");
      ui.text(ctx, t("bowl.final", { n: totalScore(this.rolls) }), cx, y + 66, this.small, ui.TEXT, "center");
      const best = this.best[this.diff];
      if (best) ui.text(ctx, t("bowl.best", { n: best }), cx, y + 88, this.tiny, ui.GOLD, "center");
      ui.text(ctx, t("bowl.new_round"), cx, y + 106, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("bowl.title"), cx, Math.floor(this.height * 0.13), this.huge, this.accent, "center");
      ui.text(ctx, t("bowl.subtitle"), cx, Math.floor(this.height * 0.2), this.small, ui.TEXT_DIM, "center");
      const label = (rects, txt) => ui.text(ctx, txt, cx, rects[0].top - 4, this.tiny, ui.TEXT_DIM, "midbottom");

      label(this.diffRects, t("bowl.lbl_diff"));
      this.diffRects.forEach((rc, i) => this.btn(ctx, rc, t("bowl.diff." + DIFFS[i]), this.diff === DIFFS[i]));
      label(this.guideRects, t("bowl.lbl_guide"));
      this.guideRects.forEach((rc, i) => this.btn(ctx, rc, i === 0 ? t("common.on") : t("common.off"), this.guide === (i === 0)));
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      const best = this.best[this.diff];
      if (best) ui.text(ctx, t("bowl.best", { n: best }), cx, this.startRect.bottom + 22, this.tiny, ui.GOLD, "center");
      ui.text(ctx, t("bowl.setup_hint"), cx, this.height - 14, this.tiny, ui.TEXT_DIM, "center");
    }

    btn(ctx, rc, text, on) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      ui.text(ctx, text, rc.centerx, rc.centery, this.small, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }
  }

  /** Stoß zweier Pins (gleiche Masse). */
  function pinPin(a, b) {
    const dx = b.x - a.x, dy = b.y - a.y;
    const d = Math.hypot(dx, dy);
    const rad = PIN_R * 2;
    if (d >= rad || d < 1e-9) return;
    const ux = dx / d, uy = dy / d;
    const push = (rad - d) / 2;
    a.x -= ux * push;
    a.y -= uy * push;
    b.x += ux * push;
    b.y += uy * push;
    const vn = (b.vx - a.vx) * ux + (b.vy - a.vy) * uy;
    if (vn > 0) return;
    const j = (-(1 + REST) * vn) / (2 / PIN_M);
    a.vx -= (j * ux) / PIN_M;
    a.vy -= (j * uy) / PIN_M;
    b.vx += (j * ux) / PIN_M;
    b.vy += (j * uy) / PIN_M;
  }

  /** Zählt die Würfe, die im angegebenen Frame liegen. */
  function rollsInFrame(rolls, frame) {
    let i = 0;
    for (let f = 0; f < 10; f++) {
      const start = i;
      if (i >= rolls.length) return 0;
      if (f === 9) return rolls.length - start;
      if (rolls[i] === 10) i += 1;
      else i += 2;
      if (f === frame) return Math.min(rolls.length, i) - start;
    }
    return 0;
  }

  /** Wurf-Symbole je Frame: X, /, -, Zahl. */
  function frameCells(rolls) {
    const cells = [];
    for (let f = 0; f < 10; f++) cells.push([]);
    let i = 0;
    for (let f = 0; f < 10; f++) {
      if (i >= rolls.length) break;
      if (f < 9) {
        if (rolls[i] === 10) {
          cells[f] = ["X"];
          i += 1;
          continue;
        }
        const a = rolls[i];
        cells[f] = [a === 0 ? "-" : String(a)];
        i += 1;
        if (i < rolls.length) {
          const b = rolls[i];
          cells[f].push(a + b === 10 ? "/" : b === 0 ? "-" : String(b));
          i += 1;
        }
      } else {
        // 10. Frame: "X" nur auf frisch aufgestellte Pins, "/" nur für den
        // zweiten Ball auf dieselbe Aufstellung (vorher wurde z.B. 5,5,5 als
        // "5 / /" und 0,10 als "- X" angezeigt).
        let prev = null; // erster Ball auf die aktuelle, angeworfene Aufstellung
        while (i < rolls.length && cells[f].length < 3) {
          const v = rolls[i];
          if (prev === null) {
            if (v === 10) cells[f].push("X");
            else {
              cells[f].push(v === 0 ? "-" : String(v));
              prev = v;
            }
          } else {
            cells[f].push(prev + v === 10 ? "/" : v === 0 ? "-" : String(v));
            prev = null;
          }
          i += 1;
        }
      }
    }
    return cells;
  }

  PG.register(BowlingGame, {
    id: "BowlingGame",
    key: "bowling",
    name: "Bowling",
    settingsKey: "bowling",
    defaults: { difficulty: "normal", guide: true },
  });
})();
