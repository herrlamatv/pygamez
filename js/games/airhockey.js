/*
 * airhockey.js - Air Hockey gegen die KI (Port von games/airhockey.py)
 * =====================================================================
 * - Echte 2D-Physik: runde Schläger und Puck mit Impulsübertragung - der Puck
 *   übernimmt die Schlägergeschwindigkeit beim Treffer, Banden mit Restitution,
 *   leichte Eisreibung, Tempolimit gegen Tunneln (2 Physik-Unterschritte).
 * - Tore als Öffnungen in den Seitenwänden; nach jedem Tor kurze Pause mit
 *   Tor-Einblendung und Anspiel beim Gegentor-Nehmer.
 * - MAUSSTEUERUNG: der eigene Schläger folgt der Maus (sobald man sie bewegt);
 *   jede Richtungstaste schaltet zurück auf Tastensteuerung (WASD UND Pfeile).
 * - KI mit drei Schwierigkeitsgraden (Leicht/Mittel/Schwer): verteidigt die
 *   eigene Toröffnung, greift an, wenn der Puck in ihrer Hälfte liegt, und
 *   umkurvt den Puck, um kein Eigentor zu schieben.
 * - POWER-UPS (im Setup abschaltbar): gehören dem Spieler, der den Puck
 *   zuletzt berührt hat, wenn der Puck sie einsammelt:
 *     * XL  - größerer eigener Schläger (8 s)
 *     * TOR - gegnerisches Tor schrumpft (8 s)
 *     * >>  - schnellerer eigener Schläger (8 s)
 * - Setup-Screen: Schwierigkeit, Tore bis zum Sieg (3/5/7/10), Power-Ups an/aus
 *   (gespeichert im Einstellungs-Abschnitt "airhockey").
 * - Highscore = eigene Tore einer Partie.
 * Web-Version: nur Einzelspieler (der Mehrspieler-Modus entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Identitätsfarben des Tisches und der Spieler - bewusst NICHT aus dem UI-Theme.
  const COL_TABLE = [21, 27, 44];
  const COL_TABLE_DARK = [10, 13, 22]; // Tor-Maul (dunkle Öffnung)
  const COL_TABLE_LINE = [46, 58, 88];
  const COL_BORDER = [90, 110, 150];
  const COL_P1 = [120, 230, 160];
  const COL_P2 = [140, 195, 255];
  const COL_PUCK = [245, 205, 90];

  const PUCK_R = 12;
  const MALLET_R = 26;
  const MALLET_SPEED = 420; // Pixel/s (Basis, Spieler)
  const PUCK_MAX_SPEED = 980;
  const WALL_REST = 0.92; // Energieerhalt an den Banden
  const FRICTION = 0.28; // "Eisreibung" pro Sekunde
  const GOAL_H_FRAC = 0.34; // Torhöhe als Anteil der Feldhöhe

  const POWERUP_EVERY = 9.0; // Sekunden bis zum nächsten Power-Up
  const POWERUP_DUR = 8.0; // Wirkdauer eines Effekts
  const POWERUP_R = 15; // Radius des Symbols auf dem Feld
  const FREEZE_AFTER_GOAL = 1.4; // Pause nach einem Tor (Sekunden)

  const GOALS_CHOICES = [3, 5, 7, 10];

  // KI-Parameter je Schwierigkeit: Tempo, Zielfehler (Pixel), Verteidigungslinie
  const AI_LEVELS = [
    { key: "easy", speed: 270, noise: 42, defend: 0.8 },
    { key: "medium", speed: 350, noise: 18, defend: 0.82 },
    { key: "hard", speed: 440, noise: 6, defend: 0.84 },
  ];

  // Power-Up-Typen: [Schlüssel, Kurzsymbol, Farbe]
  const POWERUPS = [
    ["big", "XL", [120, 230, 160]],
    ["shrink", "TOR", [240, 150, 90]],
    ["fast", ">>", [140, 195, 255]],
  ];

  const SETUP = "setup", PLAY = "play";
  const DIRS = ["up", "down", "left", "right"];

  const other = (side) => (side === "p1" ? "p2" : "p1");

  /** Ein Schläger: Position, geglättete Geschwindigkeit (für den Impuls). */
  function mallet(x, y) {
    return { x, y, vx: 0, vy: 0 };
  }

  class AirHockeyGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.winner = null;

      const ah = this.opts;
      this.diff = PG.clamp(Math.trunc(Number(ah.difficulty)) || 0, 0, 2);
      const goals = Math.trunc(Number(ah.goals));
      this.winGoals = GOALS_CHOICES.includes(goals) ? goals : 5;
      this.powerupsOn = ah.powerups !== false;

      this.animT = 0;
      this.tableCache = null;

      this.applyGeometry();
      this.state = SETUP;
      this.newMatch();
    }

    /** Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schrift). */
    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, Math.min(26, Math.floor(h / 26))));
      this.bigFont = ui.font(Math.max(30, Math.min(54, Math.floor(h / 10))), true);
      this.small = ui.font(Math.max(13, Math.min(20, Math.floor(h / 32))));
      this.tiny = ui.font(Math.max(11, Math.min(16, Math.floor(h / 42))));
    }

    /** Feldgeometrie, Schriften und Setup-Layout aus width/height ableiten. */
    applyGeometry() {
      this.makeFonts();
      this.fx0 = 14;
      this.fy0 = 14;
      this.fx1 = this.width - 14;
      this.fy1 = this.height - 14;
      this.cx = Math.floor(this.width / 2);
      this.cy = Math.floor(this.height / 2);
      this.buildSetupLayout();
    }

    newMatch() {
      this.goals = { p1: 0, p2: 0 };
      this.score = 0;
      this.gameOver = false;
      this.winner = null;
      this.effects = { p1: {}, p2: {} }; // side -> {effekt: restzeit}
      this.powerup = null; // [x, y, typ_index] oder null
      this.powerupTimer = POWERUP_EVERY;
      this.lastTouch = null;
      this.freeze = 0;
      this.goalFlash = 0; // Tor-Einblendung (Restzeit)
      this.goalFlashSide = null; // wer hat getroffen
      this.particles = [];
      this.trail = []; // letzte Puck-Positionen
      this.overT = 0;

      const qh = this.height / 2;
      this.mallets = {
        p1: mallet(this.fx0 + (this.cx - this.fx0) * 0.35, qh),
        p2: mallet(this.fx1 - (this.fx1 - this.cx) * 0.35, qh),
      };
      // Gedrückte Richtungs-Aktionen je Tastenbelegung
      this.pressed = { p1: new Set(), p2: new Set() };
      // Maussteuerung des eigenen Schlägers; wird bei Mausbewegung aktiv
      this.mouseMode = false;
      this.mousePos = [this.mallets.p1.x, this.mallets.p1.y];

      this.centerPuck(null);
    }

    /** Puck platzieren: Mitte oder Anspiel in der Hälfte von 'side'. */
    centerPuck(side) {
      if (side == null) {
        this.puckX = this.cx;
        this.puckY = this.cy;
      } else {
        const off = (this.cx - this.fx0) * 0.5;
        this.puckX = side === "p1" ? this.cx - off : this.cx + off;
        this.puckY = this.cy;
      }
      this.puckVx = 0;
      this.puckVy = 0;
      this.trail = [];
    }

    // ----- Effekt-Helfer --------------------------------------------------
    malletR(side) {
      return MALLET_R * ("big" in this.effects[side] ? 1.35 : 1.0);
    }

    malletSpeed(side) {
      return MALLET_SPEED * ("fast" in this.effects[side] ? 1.45 : 1.0);
    }

    /** Höhe der Toröffnung von 'side' (schrumpft durch gegnerisches TOR-Up). */
    goalH(side) {
      let h = (this.fy1 - this.fy0) * GOAL_H_FRAC;
      if ("shrink" in this.effects[other(side)]) h *= 0.6;
      return h;
    }

    goalRange(side) {
      const h = this.goalH(side);
      return [this.cy - h / 2, this.cy + h / 2];
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(440, this.width - 60);
      const ph = Math.max(52, Math.min(64, Math.floor(this.height * 0.12)));
      const y0 = Math.max(108, Math.floor(this.height * 0.24));
      this.diffPanel = new PG.Rect(cx - Math.floor(bw / 2), y0, bw, ph);
      this.diffLeft = new PG.Rect(this.diffPanel.left, y0, 42, ph);
      this.diffRight = new PG.Rect(this.diffPanel.right - 42, y0, 42, ph);
      const bh = Math.max(38, Math.min(46, Math.floor(this.height * 0.085)));
      const gap = 10;
      const y = this.diffPanel.bottom + 14;
      this.goalsRect = new PG.Rect(cx - Math.floor(bw / 2), y, bw, bh);
      this.powerRect = new PG.Rect(cx - Math.floor(bw / 2), y + bh + gap, bw, bh);
      this.startRect = new PG.Rect(cx - 95, y + 2 * (bh + gap) + 8, 190, 50);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    cycleDiff(step) {
      this.diff = PG.mod(this.diff + step, AI_LEVELS.length);
      this.saveSetting("difficulty", this.diff);
      this.playSound("click");
    }

    cycleGoals() {
      const i = GOALS_CHOICES.indexOf(this.winGoals);
      this.winGoals = GOALS_CHOICES[(i + 1) % GOALS_CHOICES.length];
      this.saveSetting("goals", this.winGoals);
      this.playSound("click");
    }

    togglePowerups() {
      this.powerupsOn = !this.powerupsOn;
      this.saveSetting("powerups", this.powerupsOn);
      this.playSound("select");
    }

    startPlay() {
      this.newMatch();
      this.state = PLAY;
      this.playSound("click");
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || k === "a" || k === "A") this.cycleDiff(-1);
        else if (k === "Right" || k === "d" || k === "D") this.cycleDiff(+1);
        else if (k === "t" || k === "T") this.cycleGoals();
        else if (k === "p" || k === "P") this.togglePowerups();
        else if (k === "Return" || k === "space") this.startPlay();
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        if (this.diffLeft.collidepoint(p)) this.cycleDiff(-1);
        else if (this.diffRight.collidepoint(p) || this.diffPanel.collidepoint(p)) this.cycleDiff(+1);
        else if (this.goalsRect.collidepoint(p)) this.cycleGoals();
        else if (this.powerRect.collidepoint(p)) this.togglePowerups();
        else if (this.startRect.collidepoint(p)) this.startPlay();
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetupEvent(ev);
        return;
      }

      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.startPlay();
          else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.gameOver = false;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ui.now() - this.overT > 0.5) {
          // Klick startet die Revanche - mit kurzer Sperre nach Spielende
          this.startPlay();
        }
        return;
      }

      if (ev.kind === "mousemove") {
        // Maus übernimmt den eigenen Schläger
        this.mouseMode = true;
        this.mousePos = ev.pos;
        return;
      }

      if (ev.kind === "keyup") {
        for (const scheme of ["p1", "p2"]) {
          for (const act of DIRS) {
            if (this.isAction(ev.key, act, scheme)) this.pressed[scheme].delete(act);
          }
        }
        return;
      }

      if (ev.kind !== "keydown") return;

      let gedrueckt = false;
      for (const scheme of ["p1", "p2"]) {
        for (const act of DIRS) {
          if (this.isAction(ev.key, act, scheme)) {
            this.pressed[scheme].add(act);
            gedrueckt = true;
          }
        }
      }
      if (gedrueckt) this.mouseMode = false; // Taste gedrückt -> zurück zu Tasten
    }

    keyDir(scheme) {
      const p = this.pressed[scheme];
      const dx = (p.has("right") ? 1 : 0) - (p.has("left") ? 1 : 0);
      const dy = (p.has("down") ? 1 : 0) - (p.has("up") ? 1 : 0);
      return [dx, dy];
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.animT += dt;
      this.updateParticles(dt);
      if (this.state !== PLAY || this.gameOver) return;

      // Effekt-Timer
      for (const side of ["p1", "p2"]) {
        for (const k of Object.keys(this.effects[side])) {
          this.effects[side][k] -= dt;
          if (this.effects[side][k] <= 0) delete this.effects[side][k];
        }
      }

      if (this.goalFlash > 0) this.goalFlash -= dt;

      this.moveMallets(dt);

      // Nach einem Tor: kurze Pause, der Puck liegt still
      if (this.freeze > 0) {
        this.freeze -= dt;
        return;
      }

      // Power-Ups
      if (this.powerupsOn) this.updatePowerups(dt);

      // Puck in 2 Unterschritten bewegen (gegen Tunneln bei hohem Tempo)
      for (let i = 0; i < 2; i++) {
        this.movePuck(dt / 2);
        if (this.freeze > 0 || this.gameOver) break; // Tor gefallen
      }

      // Leuchtspur
      this.trail.push([this.puckX, this.puckY]);
      if (this.trail.length > 10) this.trail.shift();
    }

    moveMallets(dt) {
      // --- Spieler (links) ---
      const m1 = this.mallets.p1;
      if (this.mouseMode) {
        this.mouseToMallet(m1, dt);
      } else {
        // beide Belegungen steuern den eigenen Schläger
        const d1 = this.keyDir("p1"), d2 = this.keyDir("p2");
        const dx = PG.clamp(d1[0] + d2[0], -1, 1);
        const dy = PG.clamp(d1[1] + d2[1], -1, 1);
        this.keysToMallet(m1, dx, dy, this.malletSpeed("p1"), dt);
      }
      this.clampMallet(m1, "p1");

      // --- KI (rechts) ---
      const m2 = this.mallets.p2;
      this.aiMove(m2, dt);
      this.clampMallet(m2, "p2");
    }

    keysToMallet(m, dx, dy, speed, dt) {
      const ln = Math.hypot(dx, dy);
      let tvx = 0, tvy = 0;
      if (ln > 0) {
        tvx = (dx / ln) * speed;
        tvy = (dy / ln) * speed;
      }
      // weiches Beschleunigen/Bremsen -> Geschwindigkeit stimmt für den Impuls
      const k = Math.min(1.0, dt * 12.0);
      m.vx += (tvx - m.vx) * k;
      m.vy += (tvy - m.vy) * k;
      m.x += m.vx * dt;
      m.y += m.vy * dt;
    }

    mouseToMallet(m, dt) {
      const [tx, ty] = this.mousePos;
      const k = Math.min(1.0, dt * 16.0);
      const nx = m.x + (tx - m.x) * k;
      const ny = m.y + (ty - m.y) * k;
      if (dt > 0) {
        m.vx = (nx - m.x) / dt;
        m.vy = (ny - m.y) / dt;
        // Maus-Sprünge deckeln, sonst bekommt der Puck absurde Impulse
        const sp = Math.hypot(m.vx, m.vy);
        if (sp > 1300) {
          m.vx *= 1300 / sp;
          m.vy *= 1300 / sp;
        }
      }
      m.x = nx;
      m.y = ny;
    }

    clampMallet(m, side) {
      const r = this.malletR(side);
      let x0, x1;
      if (side === "p1") {
        x0 = this.fx0 + r;
        x1 = this.cx - r;
      } else {
        x0 = this.cx + r;
        x1 = this.fx1 - r;
      }
      const nx = Math.max(x0, Math.min(x1, m.x));
      const ny = Math.max(this.fy0 + r, Math.min(this.fy1 - r, m.y));
      if (nx !== m.x) m.vx = 0;
      if (ny !== m.y) m.vy = 0;
      m.x = nx;
      m.y = ny;
    }

    // ----- KI ---------------------------------------------------------------
    aiMove(m, dt) {
      const lvl = AI_LEVELS[this.diff];
      const speed = lvl.speed * ("fast" in this.effects.p2 ? 1.45 : 1.0);
      const r = this.malletR("p2");
      const feldW = this.fx1 - this.fx0;
      let tx, ty;

      if (this.puckX > this.cx && this.freeze <= 0) {
        // Angriff: hinter den Puck stellen und Richtung Spieler-Tor schieben
        const zx = this.fx0, zy = this.cy; // gegnerisches Tor
        const dx = zx - this.puckX;
        const dy = zy - this.puckY;
        const ln = Math.hypot(dx, dy) || 1.0;
        tx = this.puckX - (dx / ln) * (r * 0.5);
        ty = this.puckY - (dy / ln) * (r * 0.5);
        if (this.puckX > m.x + 4) {
          // Puck liegt hinter dem Schläger -> außen herum, kein Eigentor
          tx = Math.min(this.fx1 - r, this.puckX + r + PUCK_R + 8);
          ty = this.puckY + (this.puckY < m.y ? r + PUCK_R + 10 : -(r + PUCK_R + 10));
        }
      } else {
        // Verteidigung: auf der Linie vor dem eigenen Tor dem Puck folgen
        tx = this.fx0 + feldW * lvl.defend;
        const [gy0, gy1] = this.goalRange("p2");
        ty = Math.max(gy0 + 10, Math.min(gy1 - 10, this.puckY));
      }

      tx += PG.rand.uniform(-lvl.noise, lvl.noise);
      ty += PG.rand.uniform(-lvl.noise, lvl.noise);

      const dx = tx - m.x, dy = ty - m.y;
      const ln = Math.hypot(dx, dy);
      if (ln > 4) {
        m.vx = (dx / ln) * speed;
        m.vy = (dy / ln) * speed;
        m.x += m.vx * dt;
        m.y += m.vy * dt;
      } else {
        m.vx = m.vy = 0;
      }
    }

    // ----- Puck-Physik --------------------------------------------------------
    movePuck(dt) {
      // Reibung
      const f = Math.max(0.0, 1.0 - FRICTION * dt);
      this.puckVx *= f;
      this.puckVy *= f;
      this.puckX += this.puckVx * dt;
      this.puckY += this.puckVy * dt;

      const r = PUCK_R;
      // Obere/untere Bande
      if (this.puckY - r < this.fy0) {
        this.puckY = this.fy0 + r;
        this.puckVy = Math.abs(this.puckVy) * WALL_REST;
        this.wallSound();
      } else if (this.puckY + r > this.fy1) {
        this.puckY = this.fy1 - r;
        this.puckVy = -Math.abs(this.puckVy) * WALL_REST;
        this.wallSound();
      }

      // Linke Wand / linkes Tor (dort punktet die KI)
      let [gy0, gy1] = this.goalRange("p1");
      if (this.puckX - r < this.fx0) {
        if (gy0 < this.puckY && this.puckY < gy1) {
          if (this.puckX + r < this.fx0) {
            this.goalScored("p2");
            return;
          }
        } else {
          this.puckX = this.fx0 + r;
          this.puckVx = Math.abs(this.puckVx) * WALL_REST;
          this.wallSound();
        }
      }
      // Rechte Wand / rechtes Tor (dort punktet der Spieler)
      [gy0, gy1] = this.goalRange("p2");
      if (this.puckX + r > this.fx1) {
        if (gy0 < this.puckY && this.puckY < gy1) {
          if (this.puckX - r > this.fx1) {
            this.goalScored("p1");
            return;
          }
        } else {
          this.puckX = this.fx1 - r;
          this.puckVx = -Math.abs(this.puckVx) * WALL_REST;
          this.wallSound();
        }
      }

      // Schläger-Kollisionen
      this.collideMallet("p1");
      this.collideMallet("p2");

      // Tempolimit
      const sp = Math.hypot(this.puckVx, this.puckVy);
      if (sp > PUCK_MAX_SPEED) {
        this.puckVx *= PUCK_MAX_SPEED / sp;
        this.puckVy *= PUCK_MAX_SPEED / sp;
      }
    }

    wallSound() {
      if (Math.hypot(this.puckVx, this.puckVy) > 120) this.playSound("bounce");
    }

    collideMallet(side) {
      const m = this.mallets[side];
      const r = this.malletR(side) + PUCK_R;
      let dx = this.puckX - m.x;
      let dy = this.puckY - m.y;
      let dist = Math.hypot(dx, dy);
      if (dist >= r) return;
      if (dist < 1e-6) {
        const ang = PG.rand.uniform(0, PG.TAU);
        dx = Math.cos(ang);
        dy = Math.sin(ang);
        dist = 1.0;
      }
      const nx = dx / dist, ny = dy / dist;
      // Puck aus dem Schläger schieben
      this.puckX = m.x + nx * (r + 0.5);
      this.puckY = m.y + ny * (r + 0.5);
      // Relativgeschwindigkeit am Kontakt reflektieren (Schläger = schwer)
      let rvx = this.puckVx - m.vx;
      let rvy = this.puckVy - m.vy;
      const vn = rvx * nx + rvy * ny;
      if (vn < 0) {
        rvx -= 1.9 * vn * nx; // Restitution ~0.9
        rvy -= 1.9 * vn * ny;
      }
      this.puckVx = m.vx + rvx;
      this.puckVy = m.vy + rvy;
      this.lastTouch = side;
      const wucht = Math.hypot(this.puckVx, this.puckVy);
      if (wucht > 520) {
        this.spawnParticles(this.puckX, this.puckY, side === "p1" ? COL_P1 : COL_P2, 6);
        this.playSound("hit");
        this.rumble(60);
      } else {
        this.playSound("bounce");
      }
    }

    // ----- Tore / Power-Ups -----------------------------------------------
    goalScored(by) {
      this.goals[by] += 1;
      if (by === "p1") this.score = this.goals.p1;
      this.goalFlash = 1.0;
      this.goalFlashSide = by;
      const gx = by === "p1" ? this.fx1 : this.fx0;
      this.spawnParticles(gx, this.puckY, by === "p1" ? COL_P1 : COL_P2, 22);
      this.playSound("point");
      this.rumble(150);

      if (this.goals[by] >= this.winGoals) {
        this.gameOver = true;
        this.winner = by;
        this.overT = ui.now();
        this.playSound(by === "p1" ? "win" : "gameover");
        this.reportResult(by === "p1");
        this.rumble(250);
        return;
      }
      // Anspiel beim Spieler, der das Tor kassiert hat
      this.freeze = FREEZE_AFTER_GOAL;
      this.centerPuck(other(by));
      this.lastTouch = null;
    }

    updatePowerups(dt) {
      if (this.powerup == null) {
        this.powerupTimer -= dt;
        if (this.powerupTimer <= 0) {
          const x = PG.rand.uniform(this.cx - 150, this.cx + 150);
          const y = PG.rand.uniform(this.fy0 + 70, this.fy1 - 70);
          this.powerup = [x, y, PG.rand.randrange(POWERUPS.length)];
          this.powerupTimer = POWERUP_EVERY;
        }
        return;
      }
      const [px, py, idx] = this.powerup;
      if (Math.hypot(this.puckX - px, this.puckY - py) < PUCK_R + POWERUP_R) {
        if (this.lastTouch != null) {
          const key = POWERUPS[idx][0];
          this.effects[this.lastTouch][key] = POWERUP_DUR;
          this.spawnParticles(px, py, POWERUPS[idx][2], 12);
          this.playSound("powerup");
        }
        this.powerup = null;
      }
    }

    // ----- Partikel ---------------------------------------------------------
    spawnParticles(x, y, color, n) {
      for (let i = 0; i < n; i++) {
        const ang = PG.rand.uniform(0, PG.TAU);
        const spd = PG.rand.uniform(40, 240);
        this.particles.push([x, y, Math.cos(ang) * spd, Math.sin(ang) * spd, PG.rand.uniform(0.25, 0.6), color]);
      }
    }

    updateParticles(dt) {
      const rest = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[2] *= 0.98;
        p[3] *= 0.98;
        p[4] -= dt;
        if (p[4] > 0) rest.push(p);
      }
      this.particles = rest;
    }

    // ----- Theme-UI-Helfer ----------------------------------------------
    /** Halbtransparentes Theme-Panel mit Akzent-Rahmen. */
    panel(ctx, rect, alpha = 235, border = 2, borderCol = null) {
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], alpha], rect, 0, 12);
      draw.rect(ctx, borderCol || this.accent, rect, border, 12);
    }

    /** Statische Tischfläche (Filz, Linien, Bande) als gecachte Offscreen-Canvas. */
    tableLayer() {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      if (this.tableCache && this.tableCache.ps === ps) return this.tableCache.canvas;
      const W = this.width, H = this.height;
      const c = ui.makeCanvas(W * ps, H * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      const fw = this.fx1 - this.fx0, fh = this.fy1 - this.fy0;
      draw.rect(g, COL_TABLE, [this.fx0, this.fy0, fw, fh], 0, 18);
      // Mittellinie (gestrichelt) + Mittelkreis + Anspielpunkt
      for (let y = this.fy0 + 6; y < this.fy1 - 6; y += 22) {
        draw.rect(g, COL_TABLE_LINE, [this.cx - 2, y, 4, 12]);
      }
      draw.circle(g, COL_TABLE_LINE, [this.cx, this.cy], 52, 3);
      draw.circle(g, COL_TABLE_LINE, [this.cx, this.cy], 5);
      // Bande
      draw.rect(g, COL_BORDER, [this.fx0, this.fy0, fw, fh], 3, 18);
      this.tableCache = { ps, canvas: c };
      return c;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      this.drawTable(ctx);
      this.drawPowerup(ctx);
      this.drawTrail(ctx);
      this.drawPuck(ctx);
      this.drawMallet(ctx, "p1");
      this.drawMallet(ctx, "p2");
      for (const p of this.particles) {
        const a = PG.clamp(Math.floor(255 * (p[4] / 0.6)), 0, 255);
        draw.circle(ctx, [p[5][0], p[5][1], p[5][2], a], [p[0], p[1]], 3);
      }
      this.drawHud(ctx);
      if (this.gameOver) this.drawGameOver(ctx);
    }

    drawTable(ctx) {
      ctx.drawImage(this.tableLayer(), 0, 0, this.width, this.height);
      // Tor-Mäuler: Öffnung dunkel + pulsierender Glow in Spielerfarbe
      const pulse = 2 + Math.trunc(2 * Math.sin(this.animT * 4));
      for (const [side, farbe, x] of [["p1", COL_P1, this.fx0], ["p2", COL_P2, this.fx1]]) {
        const [gy0, gy1] = this.goalRange(side);
        draw.rect(ctx, COL_TABLE_DARK, [x - 5, gy0, 10, gy1 - gy0]);
        draw.rect(ctx, farbe, [x - 3, gy0, 6, gy1 - gy0], 0, 3);
        draw.rect(ctx, [farbe[0], farbe[1], farbe[2], 60 + pulse * 10], [x - 8, gy0 - 10, 16, Math.trunc(gy1 - gy0) + 20], 0, 8);
      }
    }

    drawTrail(ctx) {
      const n = this.trail.length;
      this.trail.forEach(([x, y], i) => {
        const a = Math.trunc((70 * (i + 1)) / Math.max(1, n));
        const r = Math.max(2, Math.trunc((PUCK_R * (i + 1)) / Math.max(1, n)) - 2);
        draw.circle(ctx, [COL_PUCK[0], COL_PUCK[1], COL_PUCK[2], a], [x, y], r);
      });
    }

    drawPuck(ctx) {
      const x = Math.trunc(this.puckX), y = Math.trunc(this.puckY);
      draw.circle(ctx, COL_PUCK, [x, y], PUCK_R);
      draw.circle(ctx, [255, 240, 190], [x - 3, y - 3], 4);
      draw.circle(ctx, [120, 95, 30], [x, y], PUCK_R, 2);
    }

    drawMallet(ctx, side) {
      const m = this.mallets[side];
      const r = Math.trunc(this.malletR(side));
      const farbe = side === "p1" ? COL_P1 : COL_P2;
      const dunkel = farbe.map((c) => c >> 1);
      const c = [Math.trunc(m.x), Math.trunc(m.y)];
      draw.circle(ctx, dunkel, c, r);
      draw.circle(ctx, farbe, c, r, 4);
      draw.circle(ctx, farbe, c, Math.max(4, Math.floor(r / 3)));
    }

    drawPowerup(ctx) {
      if (this.powerup == null) return;
      const [x, y, idx] = this.powerup;
      const [, label, farbe] = POWERUPS[idx];
      const r = POWERUP_R + Math.trunc(2 * Math.sin(this.animT * 6));
      draw.circle(ctx, [farbe[0], farbe[1], farbe[2], 50], [x, y], r * 2);
      draw.circle(ctx, [25, 30, 45], [Math.trunc(x), Math.trunc(y)], r);
      draw.circle(ctx, farbe, [Math.trunc(x), Math.trunc(y)], r, 2);
      ui.text(ctx, label, Math.trunc(x), Math.trunc(y), this.tiny, farbe, "center");
    }

    drawHud(ctx) {
      const y0 = this.fy0 + 8;
      ui.text(ctx, String(this.goals.p1), this.cx - 70, y0, this.bigFont, COL_P1, "midtop");
      ui.text(ctx, String(this.goals.p2), this.cx + 70, y0, this.bigFont, COL_P2, "midtop");
      const ly = y0 + this.bigFont.height + 2;
      ui.text(ctx, t("ah.you"), this.cx - 70, ly, this.small, COL_P1, "midtop");
      ui.text(ctx, t("ah.ai"), this.cx + 70, ly, this.small, COL_P2, "midtop");
      ui.text(ctx, t("ah.first_to", { n: this.winGoals }), this.cx, ly + this.small.height + 4, this.tiny, ui.TEXT_DIM, "midtop");

      // Aktive Effekte als kleine Anzeigen unter dem Spielstand
      for (const [side, basex, richtung] of [["p1", this.fx0 + 12, 1], ["p2", this.fx1 - 12, -1]]) {
        let y = this.fy0 + 8;
        for (const key of Object.keys(this.effects[side]).sort()) {
          const restzeit = this.effects[side][key];
          for (const [pk, label, farbe] of POWERUPS) {
            if (pk !== key) continue;
            const txt = label + " " + String(Math.round(restzeit)).padStart(2, " ") + "s";
            ui.text(ctx, txt, basex, y, this.tiny, farbe, richtung === 1 ? "topleft" : "topright");
            y += 16;
          }
        }
      }

      // Tor-Einblendung
      if (this.goalFlash > 0 && !this.gameOver) {
        const farbe = this.goalFlashSide === "p1" ? COL_P1 : COL_P2;
        const a = PG.clamp(this.goalFlash, 0, 1);
        ui.text(ctx, t("ah.goal"), this.cx, this.cy - 60, this.bigFont, farbe, "center", a);
      }

      // Steuerungs-Hinweis (nur die ersten Sekunden)
      if (this.animT < 6) {
        ui.text(ctx, t("ah.mouse_hint"), this.cx, this.fy1 - 6, this.tiny, ui.TEXT_DIM, "midbottom");
      }
    }

    drawGameOver(ctx) {
      ctx.fillStyle = "rgba(8,10,16,0.588)";
      ctx.fillRect(0, 0, this.width, this.height);
      const gewonnen = this.winner === "p1";
      const text = gewonnen ? t("ah.win") : t("ah.lose");
      const farbe = gewonnen ? COL_P1 : ui.RED;
      this.drawCenterText(ctx, text, this.bigFont, farbe, -40);
      this.drawCenterText(ctx, this.goals.p1 + " : " + this.goals.p2, this.font, ui.TEXT, 4);
      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
      this.drawCenterText(ctx, t("ah.restart_hint"), this.font, hintCol, 44);
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      ui.drawTitle(ctx, this.width, "AIR HOCKEY", { subtitle: t("snake.singleplayer"), accent: this.accent });

      // Schwierigkeit
      const lvl = AI_LEVELS[this.diff];
      const dp = this.diffPanel;
      this.panel(ctx, dp);
      ui.text(ctx, t("ah.difficulty") + ":  " + t("ah.diff." + lvl.key), dp.centerx, dp.top + Math.trunc(dp.h * 0.34), this.font, ui.TEXT, "center");
      ui.text(ctx, t("ah.diff_note"), dp.centerx, dp.top + Math.trunc(dp.h * 0.72), this.tiny, ui.TEXT_DIM, "center");
      const arrCol = ui.mix(this.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3));
      for (const [r, sym] of [[this.diffLeft, "<"], [this.diffRight, ">"]]) {
        ui.text(ctx, sym, r.centerx, r.centery, this.bigFont, arrCol, "center");
      }

      this.drawRow(ctx, this.goalsRect, t("ah.goals"), String(this.winGoals), true);
      this.drawRow(ctx, this.powerRect, t("ah.powerups"), this.powerupsOn ? t("common.on") : t("common.off"), this.powerupsOn);

      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: this.accent });

      ui.text(ctx, t("ah.mouse_hint"), Math.floor(this.width / 2), this.startRect.bottom + 22, this.tiny, ui.GREEN, "center");
      ui.drawFooter(ctx, this.width, this.height, t("ah.setup_hint"));
    }

    drawRow(ctx, rect, label, wert, an) {
      draw.rect(ctx, an ? ui.BTN_SEL : ui.BTN, rect, 0, 8);
      draw.rect(ctx, an ? ui.BORDER_LIGHT : ui.BORDER, rect, 1, 8);
      ui.text(ctx, label, rect.x + 16, rect.centery, this.font, ui.TEXT, "midleft");
      ui.text(ctx, "< " + wert + " >", rect.right - 16, rect.centery, this.font, an ? this.accent : ui.TEXT_DIM, "midright");
    }
  }

  PG.register(AirHockeyGame, {
    id: "AirHockeyGame",
    key: "airhockey",
    name: "Air Hockey",
    settingsKey: "airhockey",
    defaults: { difficulty: 1, goals: 5, powerups: true },
  });
})();
