/*
 * tanks.js - Panzer-Duell gegen die KI (Port von games/tanks.py)
 * ================================================================
 * - Panzer drehen und fahren (WASD + Leertaste ODER Pfeile + Enter - beide
 *   Belegungen steuern den eigenen Panzer); Schüsse prallen EINMAL von Wänden
 *   ab (Ricochet) und verschwinden beim zweiten Wandkontakt. Auch der eigene
 *   Schuss ist gefährlich (kurze Schonfrist nach dem Abfeuern).
 * - Runden: Wer zuerst 5 Runden gewinnt, gewinnt das Match.
 * - 4 Arenen (offen/Kreuz/Säulen/Labyrinth), wählbar oder zufällige Rotation.
 * - Power-Ups: Schnellfeuer, Schild (1 Treffer), Dreifach-Schuss.
 * - KI mit drei Stärken: easy wandert und streut, medium verfolgt und weicht
 *   aus, hard nutzt Vorhalt UND Ricochet-Schüsse über Wandspiegelung.
 * - Punkte: Matchsieg gegen die KI = (Stufe+1)*100 + (5-KI-Runden)*20,
 *   kumulativ über Rematches.
 * Web-Version: nur Einzelspieler (der 2-Spieler-Modus entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  PG.addStrings({
    de: { "web.tanks.controls_hint": "WASD oder Pfeile = fahren   -   Leertaste oder Enter = Feuer" },
    en: { "web.tanks.controls_hint": "WASD or arrows = drive   -   Space or Enter = fire" },
  });

  // Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme):
  // Arena-Boden/-Wände und die beiden Panzerfarben.
  const COL_FLOOR = [24, 28, 38];
  const COL_WALL = [72, 80, 100];
  const COL_WALL_EDGE = [100, 110, 134];
  const COL_P1 = [120, 180, 90]; // Grün-oliv
  const COL_P2 = [200, 120, 70]; // Rost
  const COL_BULLET = [240, 240, 250];
  const POWER_COLS = { rapid: [245, 205, 90], shield: [110, 190, 255], triple: [230, 120, 200] };

  const DIFFS = ["easy", "medium", "hard"];
  const ARENAS = ["random", "open", "cross", "pillars", "maze"];
  const WALL_PRESETS = {
    open: [],
    cross: [[0.35, 0.475, 0.3, 0.05], [0.475, 0.35, 0.05, 0.3]],
    pillars: [[0.18, 0.18, 0.14, 0.14], [0.68, 0.18, 0.14, 0.14], [0.18, 0.68, 0.14, 0.14], [0.68, 0.68, 0.14, 0.14]],
    maze: [[0.0, 0.32, 0.42, 0.045], [0.58, 0.32, 0.42, 0.045], [0.3, 0.62, 0.045, 0.38], [0.655, 0.0, 0.045, 0.38]],
  };

  const ROUNDS_TO_WIN = 5;
  const BULLET_SPEED = 320.0;
  const BULLET_LIFE = 6.0;
  const MAX_BULLETS = 4;
  const COOLDOWN = 0.5;
  const GRACE = 0.15; // Eigentreffer-Schonfrist nach dem Abfeuern

  const SETUP = "setup", COUNTDOWN = "countdown", PLAY = "play", ROUND_END = "round_end", MATCH_END = "match_end";

  /** Winkel auf -PI..PI normieren (wie (a + pi) % tau - pi in Python). */
  const wrapAngle = (a) => PG.mod(a + Math.PI, PG.TAU) - Math.PI;

  /** pygame.Rect.clipline-Ersatz: schneidet die Strecke a->b das Rechteck? (Liang-Barsky) */
  function clipsRect(r, a, b) {
    const x0 = a[0], y0 = a[1];
    const dx = b[0] - x0, dy = b[1] - y0;
    let t0 = 0, t1 = 1;
    const p = [-dx, dx, -dy, dy];
    const q = [x0 - r.left, r.right - x0, y0 - r.top, r.bottom - y0];
    for (let i = 0; i < 4; i++) {
      if (p[i] === 0) {
        if (q[i] < 0) return false;
      } else {
        const tt = q[i] / p[i];
        if (p[i] < 0) {
          if (tt > t1) return false;
          if (tt > t0) t0 = tt;
        } else {
          if (tt < t0) return false;
          if (tt < t1) t1 = tt;
        }
      }
    }
    return t0 <= t1;
  }

  class TankDuelGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const tk = this.opts;
      this.diff = PG.clamp(Math.trunc(Number(tk.difficulty)) || 0, 0, 2);
      const ar = Math.trunc(Number(tk.arena));
      this.arenaSel = PG.clamp(isNaN(ar) ? -1 : ar, -1, 3);

      this.makeFonts();
      this.rounds = [0, 0];
      this.arenaCycle = [];
      this.arenaCache = null;
      this.aiAim = null;
      this.aiWp = null;
      this.buildSetupLayout();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größen aus der Fensterhöhe abgeleitet. */
    makeFonts() {
      this.small = ui.font(Math.max(13, Math.min(22, Math.floor(this.height / 30))));
      this.tiny = ui.font(Math.max(11, Math.min(18, Math.floor(this.height / 38))));
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(380, this.width - 60);
      const y0 = Math.floor(this.height * 0.3);
      this.setupRows = ["diff", "arena"];
      this.rowRects = this.setupRows.map((_, i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 58, bw, 46));
      this.startRect = new PG.Rect(cx - 95, y0 + this.setupRows.length * 58 + 16, 190, 46);
      this.selRow = 0;
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    adjustRow(row, direction) {
      if (this.setupRows[row] === "diff") {
        this.diff = PG.mod(this.diff + direction, 3);
        this.saveSetting("difficulty", this.diff);
      } else {
        this.arenaSel = PG.mod(this.arenaSel + 1 + direction, 5) - 1;
        this.saveSetting("arena", this.arenaSel);
      }
      this.playSound("select");
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Up" || k === "w" || k === "W") {
          this.selRow = PG.mod(this.selRow - 1, this.setupRows.length);
          this.playSound("move");
        } else if (k === "Down" || k === "s" || k === "S") {
          this.selRow = PG.mod(this.selRow + 1, this.setupRows.length);
          this.playSound("move");
        } else if (k === "Left" || k === "a" || k === "A") {
          this.adjustRow(this.selRow, -1);
        } else if (k === "Right" || k === "d" || k === "D") {
          this.adjustRow(this.selRow, +1);
        } else if (k === "Return" || k === "space") {
          this.startMatch();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.rowRects.length; i++) {
          if (this.rowRects[i].collidepoint(ev.pos)) {
            this.selRow = i;
            this.adjustRow(i, +1);
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startMatch();
      }
    }

    // ===================================================== Match / Runde
    startMatch() {
      this.rounds = [0, 0];
      this.gameOver = false;
      this.arenaCycle = PG.rand.shuffle(["open", "cross", "pillars", "maze"]);
      this.startRound();
      this.playSound("click");
    }

    layoutArena() {
      this.hudH = Math.max(34, Math.floor(this.height * 0.075));
      this.arena = new PG.Rect(8, this.hudH, this.width - 16, this.height - this.hudH - 8);
      this.sc = Math.min(this.width / 640.0, this.height / 480.0);
      this.innerWalls = [];
      for (const [fx, fy, fw, fh] of WALL_PRESETS[this.arenaKey]) {
        this.innerWalls.push(new PG.Rect(
          this.arena.x + Math.trunc(fx * this.arena.w),
          this.arena.y + Math.trunc(fy * this.arena.h),
          Math.max(14, Math.trunc(fw * this.arena.w)),
          Math.max(14, Math.trunc(fh * this.arena.h))));
      }
    }

    newTank(x, y, angDeg) {
      return { x, y, ang: PG.radians(angDeg), cooldown: 0, shield: false, rapid: 0, triple: 0, alive: true, keys: new Set() };
    }

    startRound() {
      if (this.arenaSel >= 0) this.arenaKey = ARENAS[this.arenaSel + 1];
      else this.arenaKey = this.arenaCycle[(this.rounds[0] + this.rounds[1]) % 4];
      this.layoutArena();
      const insetX = Math.trunc(this.arena.w * 0.15);
      const insetY = Math.trunc(this.arena.h * 0.15);
      this.tanks = [
        this.newTank(this.arena.x + insetX, this.arena.y + insetY, 45),
        this.newTank(this.arena.right - insetX, this.arena.bottom - insetY, 225),
      ];
      this.bullets = []; // {x,y,vx,vy,owner,bounces,age}
      this.powerups = []; // {x,y,kind,age}
      this.powerTimer = PG.rand.uniform(8, 14);
      this.countT = 2.4;
      this.bannerT = 0;
      this.roundWinner = null;
      this.aiTimer = 0;
      this.aiMove = [0, 0]; // [throttle, turn]
      this.state = COUNTDOWN;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === MATCH_END) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.gameOver = false;
            this.startMatch();
          } else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown") {
          this.gameOver = false;
          this.startMatch();
        }
        return;
      }
      if (ev.kind === "keydown") this.setKey(ev.key, true, ev.repeat);
      else if (ev.kind === "keyup") this.setKey(ev.key, false, false);
    }

    /**
     * Gehaltene Aktionen puffern; Feuern sofort auslösen.
     * BEIDE Tastenbelegungen (P1 und P2) steuern den eigenen Panzer.
     */
    setKey(key, down, repeat) {
      const tank = this.tanks[0];
      for (const p of ["p1", "p2"]) {
        for (const act of ["up", "down", "left", "right"]) {
          if (this.keyFor(p, act) === key) {
            if (down) tank.keys.add(act);
            else tank.keys.delete(act);
          }
        }
        // Tasten-Wiederholung des Browsers löst keinen Dauerfeuer-Schuss aus
        if (this.keyFor(p, "action") === key && down && !repeat && this.state === PLAY) {
          this.fire(tank, 0);
        }
      }
    }

    // ===================================================== Physik
    moveTank(tk, dt, throttle, turn) {
      tk.ang += PG.radians(180) * turn * dt;
      const speed = (throttle > 0 ? 140 : 100) * this.sc * throttle;
      if (speed) {
        tk.x += Math.cos(tk.ang) * speed * dt;
        tk.y += Math.sin(tk.ang) * speed * dt;
      }
      this.collideWalls(tk);
    }

    tankR() {
      return 13 * this.sc;
    }

    collideWalls(tk) {
      const r = this.tankR();
      tk.x = Math.max(this.arena.x + r, Math.min(this.arena.right - r, tk.x));
      tk.y = Math.max(this.arena.y + r, Math.min(this.arena.bottom - r, tk.y));
      for (const w of this.innerWalls) {
        const nearestX = Math.max(w.left, Math.min(w.right, tk.x));
        const nearestY = Math.max(w.top, Math.min(w.bottom, tk.y));
        const dx = tk.x - nearestX, dy = tk.y - nearestY;
        const d2 = dx * dx + dy * dy;
        if (d2 < r * r) {
          const d = Math.sqrt(d2) || 0.001;
          const push = r - d;
          tk.x += (dx / d) * push;
          tk.y += (dy / d) * push;
        }
      }
    }

    fire(tk, owner) {
      if (tk.cooldown > 0 || !tk.alive) return;
      const live = this.bullets.filter((b) => b.owner === owner).length;
      if (live >= MAX_BULLETS) return;
      tk.cooldown = tk.rapid > 0 ? 0.2 : COOLDOWN;
      let angles = [0.0];
      if (tk.triple > 0) {
        angles = [-PG.radians(12), 0.0, PG.radians(12)];
        tk.triple -= 1;
      }
      const r = this.tankR();
      for (const da of angles) {
        const a = tk.ang + da;
        this.bullets.push({
          x: tk.x + Math.cos(a) * (r + 6),
          y: tk.y + Math.sin(a) * (r + 6),
          vx: Math.cos(a) * BULLET_SPEED * this.sc,
          vy: Math.sin(a) * BULLET_SPEED * this.sc,
          owner, bounces: 0, age: 0,
        });
      }
      this.playSound("shoot");
    }

    updateBullets(dt) {
      const alive = [];
      for (const b of this.bullets) {
        if (this.state !== PLAY) break; // Runde ist gerade zu Ende gegangen
        b.age += dt;
        if (b.age > BULLET_LIFE) continue;
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        // Außenrand
        let bounced = false;
        if (b.x < this.arena.x || b.x > this.arena.right) {
          b.vx = -b.vx;
          b.x = Math.max(this.arena.x, Math.min(this.arena.right, b.x));
          bounced = true;
        }
        if (b.y < this.arena.y || b.y > this.arena.bottom) {
          b.vy = -b.vy;
          b.y = Math.max(this.arena.y, Math.min(this.arena.bottom, b.y));
          bounced = true;
        }
        // Innenwände: Achse mit kleinerer Eindringtiefe spiegeln
        if (!bounced) {
          for (const w of this.innerWalls) {
            if (w.collidepoint(b.x, b.y)) {
              const penX = Math.min(b.x - w.left, w.right - b.x);
              const penY = Math.min(b.y - w.top, w.bottom - b.y);
              if (penX < penY) {
                b.vx = -b.vx;
                b.x += (b.vx < 0 ? -1 : 1) * (penX + 1);
              } else {
                b.vy = -b.vy;
                b.y += (b.vy < 0 ? -1 : 1) * (penY + 1);
              }
              bounced = true;
              break;
            }
          }
        }
        if (bounced) {
          b.bounces += 1;
          if (b.bounces >= 2) continue;
          this.playSound("bounce");
        }
        // Treffer?
        let hit = false;
        for (let i = 0; i < this.tanks.length; i++) {
          const tk = this.tanks[i];
          if (!tk.alive) continue;
          if (i === b.owner && b.age < GRACE) continue;
          if (Math.hypot(b.x - tk.x, b.y - tk.y) < this.tankR() + 3.5) {
            hit = true;
            if (tk.shield) {
              tk.shield = false;
              this.playSound("bounce");
            } else {
              tk.alive = false;
              this.roundOver(1 - i);
            }
            break;
          }
        }
        if (!hit) alive.push(b);
      }
      if (this.state === PLAY) this.bullets = alive;
    }

    updatePowerups(dt) {
      for (const p of this.powerups) p.age += dt;
      this.powerups = this.powerups.filter((p) => p.age < 12.0);
      this.powerTimer -= dt;
      if (this.powerTimer <= 0 && this.powerups.length < 2) {
        this.powerTimer = PG.rand.uniform(8, 14);
        for (let n = 0; n < 30; n++) {
          const x = PG.rand.uniform(this.arena.x + 60, this.arena.right - 60);
          const y = PG.rand.uniform(this.arena.y + 60, this.arena.bottom - 60);
          if (this.innerWalls.some((w) => w.inflate(120, 120).collidepoint(x, y))) continue;
          if (this.tanks.some((tk) => Math.hypot(x - tk.x, y - tk.y) < 60)) continue;
          this.powerups.push({ x, y, age: 0, kind: PG.rand.choice(["rapid", "shield", "triple"]) });
          break;
        }
      }
      // Aufnehmen
      for (const p of this.powerups.slice()) {
        for (const tk of this.tanks) {
          if (!tk.alive) continue;
          if (Math.hypot(p.x - tk.x, p.y - tk.y) < this.tankR() + 12) {
            if (p.kind === "rapid") tk.rapid = 8.0;
            else if (p.kind === "shield") tk.shield = true;
            else tk.triple = 5;
            this.powerups.splice(this.powerups.indexOf(p), 1);
            this.playSound("powerup");
            break;
          }
        }
      }
    }

    // ===================================================== KI
    /** Sichtlinie: Segment a->b kreuzt keine Innenwand. */
    clear(a, b) {
      for (const w of this.innerWalls) {
        if (clipsRect(w, a, b)) return false;
      }
      return true;
    }

    aiUpdate(dt) {
      const ai = this.tanks[1];
      const pl = this.tanks[0];
      if (!ai.alive || !pl.alive) return;
      const intervals = [0.6, 0.35, 0.2];
      const errors = [14.0, 7.0, 3.0];
      this.aiTimer -= dt;
      if (this.aiTimer <= 0) {
        this.aiTimer = intervals[this.diff];
        this.aiThink(ai, pl, errors[this.diff]);
      }
      const [throttle, turn] = this.aiMove;
      this.moveTank(ai, dt, throttle, turn);
      // Feuern, wenn grob ausgerichtet
      const want = this.aiAim;
      if (want != null) {
        const diffA = wrapAngle(want - ai.ang);
        if (Math.abs(diffA) < PG.radians(8)) this.fire(ai, 1);
      }
    }

    aiThink(ai, pl, errDeg) {
      const aPos = [ai.x, ai.y];
      const pPos = [pl.x, pl.y];
      const dist = Math.hypot(pPos[0] - aPos[0], pPos[1] - aPos[1]);
      let aim = null;
      if (this.clear(aPos, pPos)) {
        let [tx, ty] = pPos;
        if (this.diff === 2) {
          // Vorhalt auf hard: grobe Zielbewegung aus gehaltenen Tasten ableiten
          const spd = pl.keys.has("up") ? 140 * this.sc : 0;
          tx += (Math.cos(pl.ang) * spd * dist) / (BULLET_SPEED * this.sc);
          ty += (Math.sin(pl.ang) * spd * dist) / (BULLET_SPEED * this.sc);
        }
        aim = Math.atan2(ty - aPos[1], tx - aPos[0]);
      } else if (this.diff === 2) {
        aim = this.ricochetAim(aPos, pPos);
      }
      if (aim != null) aim += PG.radians(PG.rand.uniform(-errDeg, errDeg));
      this.aiAim = aim;

      // Bewegung
      let dodge = null;
      if (this.diff >= 1) {
        for (const b of this.bullets) {
          if (b.owner === 1) continue;
          const d = Math.hypot(b.x - aPos[0], b.y - aPos[1]);
          if (d < 120 * this.sc) {
            dodge = Math.atan2(aPos[1] - b.y, aPos[0] - b.x) + Math.PI / 2;
            break;
          }
        }
      }
      let targetAng, throttle;
      if (dodge != null) {
        targetAng = dodge;
        throttle = 1.0;
      } else if (this.diff === 0) {
        let wp = this.aiWp;
        if (wp == null || Math.hypot(wp[0] - aPos[0], wp[1] - aPos[1]) < 40) {
          wp = [PG.rand.uniform(this.arena.x + 40, this.arena.right - 40), PG.rand.uniform(this.arena.y + 40, this.arena.bottom - 40)];
          this.aiWp = wp;
        }
        targetAng = Math.atan2(wp[1] - aPos[1], wp[0] - aPos[0]);
        throttle = 1.0;
      } else if (this.diff === 1) {
        targetAng = Math.atan2(pPos[1] - aPos[1], pPos[0] - aPos[0]);
        throttle = dist > 350 * this.sc ? 1.0 : dist < 200 * this.sc ? -1.0 : 0.0;
      } else {
        targetAng = Math.atan2(pPos[1] - aPos[1], pPos[0] - aPos[0]) + Math.PI / 2; // Umkreisen
        throttle = 1.0;
      }
      // Richtung Ziel ausrichten: wenn Zielwinkel (fürs Schießen) existiert,
      // hat der Vorrang beim Drehen.
      const steerTo = this.aiAim != null ? this.aiAim : targetAng;
      const diffA = wrapAngle(steerTo - ai.ang);
      const turn = PG.clamp(diffA * 3.0, -1.0, 1.0);
      this.aiMove = [throttle, turn];
    }

    /** Ricochet über Spiegelung des Ziels an Wand-Ebenen (auch Rand). */
    ricochetAim(a, p) {
      let planes = [["x", this.arena.x], ["x", this.arena.right], ["y", this.arena.y], ["y", this.arena.bottom]];
      for (const w of this.innerWalls) {
        planes.push(["x", w.left], ["x", w.right], ["y", w.top], ["y", w.bottom]);
      }
      PG.rand.shuffle(planes);
      for (const [axis, v] of planes) {
        const mirror = axis === "x" ? [2 * v - p[0], p[1]] : [p[0], 2 * v - p[1]];
        // Reflexionspunkt auf der Ebene
        const dx = mirror[0] - a[0];
        const dy = mirror[1] - a[1];
        let tt;
        if (axis === "x") {
          if (dx === 0) continue;
          tt = (v - a[0]) / dx;
        } else {
          if (dy === 0) continue;
          tt = (v - a[1]) / dy;
        }
        if (!(tt > 0.05 && tt < 0.95)) continue;
        const hit = [a[0] + dx * tt, a[1] + dy * tt];
        if (this.clear(a, hit) && this.clear(hit, p)) return Math.atan2(dy, dx);
      }
      return null;
    }

    // ===================================================== Runden-Ende
    roundOver(winner) {
      this.rounds[winner] += 1;
      this.roundWinner = winner;
      this.playSound("explode");
      if (this.rounds[winner] >= ROUNDS_TO_WIN) {
        this.matchOver(winner);
      } else {
        this.bannerT = 1.8;
        this.state = ROUND_END;
        this.playSound("point");
      }
    }

    matchOver(winner) {
      this.state = MATCH_END;
      if (winner === 0) {
        this.score += (this.diff + 1) * 100 + (ROUNDS_TO_WIN - this.rounds[1]) * 20;
        this.playSound("win");
        this.reportResult(true);
      } else {
        this.playSound("gameover");
        this.reportResult(false);
      }
      this.gameOver = true;
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === COUNTDOWN) {
        this.countT -= dt;
        if (this.countT <= 0) {
          this.state = PLAY;
          this.playSound("level");
        }
        return;
      }
      if (this.state === ROUND_END) {
        this.bannerT -= dt;
        if (this.bannerT <= 0) this.startRound();
        return;
      }
      if (this.state !== PLAY) return;

      for (const tk of this.tanks) {
        tk.cooldown = Math.max(0, tk.cooldown - dt);
        tk.rapid = Math.max(0, tk.rapid - dt);
      }
      // Spieler-Steuerung
      const tk = this.tanks[0];
      const throttle = (tk.keys.has("up") ? 1 : 0) - (tk.keys.has("down") ? 1 : 0);
      const turn = (tk.keys.has("right") ? 1 : 0) - (tk.keys.has("left") ? 1 : 0);
      this.moveTank(tk, dt, throttle, turn);
      this.aiUpdate(dt);
      // Panzer-Panzer sanft auseinanderdrücken
      const [a, b] = this.tanks;
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      const minD = 2 * this.tankR();
      if (d > 0 && d < minD) {
        const push = (minD - d) / 2;
        const nx = (a.x - b.x) / d, ny = (a.y - b.y) / d;
        a.x += nx * push;
        a.y += ny * push;
        b.x -= nx * push;
        b.y -= ny * push;
      }
      this.updateBullets(dt);
      if (this.state === PLAY) this.updatePowerups(dt);
    }

    // ===================================================== Zeichnen
    /** Boden + Wände der aktuellen Arena als gecachte Offscreen-Canvas. */
    arenaLayer() {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = this.arenaKey + "@" + ps;
      if (this.arenaCache && this.arenaCache.key === key) return this.arenaCache.canvas;
      const c = ui.makeCanvas(this.width * ps, this.height * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      draw.rect(g, COL_FLOOR, this.arena);
      draw.rect(g, COL_WALL_EDGE, this.arena, 3);
      for (const w of this.innerWalls) {
        draw.rect(g, COL_WALL, w);
        draw.rect(g, COL_WALL_EDGE, w, 2);
      }
      this.arenaCache = { key, canvas: c };
      return c;
    }

    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      ctx.drawImage(this.arenaLayer(), 0, 0, this.width, this.height);
      for (const p of this.powerups) this.drawPowerup(ctx, p);
      const br = Math.max(2, Math.trunc(3.5 * this.sc));
      for (const b of this.bullets) draw.circle(ctx, COL_BULLET, [Math.trunc(b.x), Math.trunc(b.y)], br);
      this.tanks.forEach((tk, i) => {
        if (tk.alive) this.drawTank(ctx, tk, i === 0 ? COL_P1 : COL_P2);
      });
      this.drawHud(ctx);
      if (this.state === COUNTDOWN) {
        const n = Math.max(1, Math.ceil(this.countT / 0.8));
        ui.text(ctx, String(n), Math.floor(this.width / 2), Math.floor(this.height / 2), this.huge, this.accent, "center");
      } else if (this.state === ROUND_END) {
        this.drawBanner(ctx, t("tank.round_win", { n: this.roundWinner + 1 }));
      } else if (this.state === MATCH_END) {
        this.drawMatchEnd(ctx);
      }
    }

    drawTank(ctx, tk, col) {
      const r = this.tankR();
      const { x, y, ang: a } = tk;
      // Körper (gedrehtes Rechteck über Polygon)
      const c = Math.cos(a), sn = Math.sin(a);
      const pts = [[-r, -r * 0.75], [r, -r * 0.75], [r, r * 0.75], [-r, r * 0.75]].map(([px, py]) => [x + px * c - py * sn, y + px * sn + py * c]);
      draw.polygon(ctx, col, pts);
      draw.polygon(ctx, col.map((v) => Math.trunc(v * 0.6)), pts, 2);
      // Ketten-Andeutung
      draw.circle(ctx, col.map((v) => Math.trunc(v * 0.7)), [x, y], Math.trunc(r * 0.55));
      // Rohr
      draw.line(ctx, [230, 232, 240], [x, y], [x + c * r * 1.5, y + sn * r * 1.5], Math.max(3, Math.trunc(4 * this.sc)));
      // Schild-Ring
      if (tk.shield) draw.circle(ctx, POWER_COLS.shield, [x, y], Math.trunc(r * 1.5), 2);
      if (tk.rapid > 0) draw.circle(ctx, POWER_COLS.rapid, [x, y], Math.trunc(r * 1.3), 1);
    }

    drawPowerup(ctx, p) {
      const col = POWER_COLS[p.kind];
      const x = Math.trunc(p.x), y = Math.trunc(p.y);
      const blink = p.age > 9.0 && Math.trunc(p.age * 6) % 2 === 0;
      if (blink) return;
      draw.circle(ctx, col, [x, y], 11);
      draw.circle(ctx, [20, 24, 34], [x, y], 11, 2);
      const sym = { rapid: ">>", shield: "O", triple: "3x" }[p.kind];
      ui.text(ctx, sym, x, y, this.tiny, [20, 24, 34], "center");
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = Math.floor(this.hudH / 2);
      const pr = Math.max(3, Math.floor(this.hudH / 9)); // Runden-Punkte ("Pips") als Kreise
      const gap = 2 * pr + 6;
      for (const [i, col] of [[0, COL_P1], [1, COL_P2]]) {
        const name = i === 0 ? t("common.player1") : t("common.ai");
        const nw = this.small.width(name);
        let x0, dir;
        if (i === 0) {
          ui.text(ctx, name, 12, cy, this.small, col, "midleft");
          x0 = 12 + nw + 12 + pr;
          dir = 1;
        } else {
          ui.text(ctx, name, this.width - 12, cy, this.small, col, "midright");
          x0 = this.width - 12 - nw - 12 - pr;
          dir = -1;
        }
        for (let k = 0; k < ROUNDS_TO_WIN; k++) {
          const x = x0 + dir * k * gap;
          if (k < this.rounds[i]) draw.circle(ctx, col, [x, cy], pr);
          else draw.circle(ctx, ui.BORDER_LIGHT, [x, cy], pr, 1);
        }
      }
      ui.text(ctx, t("tank.first_to", { n: ROUNDS_TO_WIN }), Math.floor(this.width / 2), cy, this.small, ui.TEXT_DIM, "center");
    }

    drawBanner(ctx, text) {
      const bandH = this.huge.height + 28;
      const y = Math.floor(this.height / 2) - Math.floor(bandH / 2);
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 225], [0, y, this.width, bandH]);
      draw.line(ctx, this.accent, [0, y], [this.width, y], 2);
      draw.line(ctx, this.accent, [0, y + bandH - 1], [this.width, y + bandH - 1], 2);
      ui.text(ctx, text, Math.floor(this.width / 2), y + Math.floor(bandH / 2), this.huge, this.accent, "center");
    }

    drawMatchEnd(ctx) {
      ctx.fillStyle = "rgba(8,10,16,0.588)";
      ctx.fillRect(0, 0, this.width, this.height);
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      const winner = this.rounds[0] >= ROUNDS_TO_WIN ? 0 : 1;
      const key = winner === 0 ? "tank.match_win" : "common.game_over";
      const rows = [
        [t(key), this.huge, winner === 0 ? COL_P1 : ui.RED],
        [this.rounds[0] + " : " + this.rounds[1], this.font, ui.TEXT],
        [t("common.points", { score: this.score }), this.small, ui.TEXT_DIM],
        [t("tank.rematch"), this.small, ui.TEXT_DIM],
      ];
      const gap = 10;
      const total = rows.reduce((s, r) => s + r[1].height, 0) + gap * (rows.length - 1);
      const pw = Math.min(this.width - 30, Math.max(340, Math.max(...rows.map((r) => r[1].width(r[0]))) + 64));
      const panel = new PG.Rect(0, 0, pw, total + 48);
      panel.center = [cx, cy];
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], panel);
      draw.rect(ctx, this.accent, panel, 2, 14);
      let yy = panel.y + 24;
      for (const [txt, f, col] of rows) {
        ui.text(ctx, txt, cx, yy, f, col, "midtop");
        yy += f.height + gap;
      }
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, PG.gameName("TankDuelGame").toUpperCase(), cx, Math.floor(this.height * 0.13), this.huge, this.accent, "center");
      ui.text(ctx, t("tank.subtitle"), cx, Math.floor(this.height * 0.2), this.small, ui.TEXT_DIM, "center");
      this.rowRects.forEach((r, i) => {
        const on = i === this.selRow;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 10);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 10);
        let label, value;
        if (this.setupRows[i] === "diff") {
          label = t("tank.difficulty");
          value = t("tank.diff." + DIFFS[this.diff]);
        } else {
          label = t("tank.arena");
          value = t("tank.arena." + ARENAS[this.arenaSel + 1]);
        }
        ui.text(ctx, label, r.x + 16, r.centery, this.small, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
        ui.text(ctx, "< " + value + " >", r.right - 16, r.centery, this.small, this.accent, "midright");
      });
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("tank.setup_hint"), cx, this.height - 30, this.tiny, ui.TEXT_FAINT, "center");
      ui.text(ctx, t("web.tanks.controls_hint"), cx, this.height - 12, this.tiny, ui.mix(this.accent, ui.TEXT, 0.45), "center");
    }
  }

  PG.register(TankDuelGame, {
    id: "TankDuelGame",
    key: "tanks",
    name: { default: "Tank Duel", de: "Panzer-Duell", fr: "Duel de chars", es: "Duelo de tanques", pt: "Duelo de tanques" },
    settingsKey: "tanks",
    defaults: { difficulty: 1, arena: -1 },
  });
})();
