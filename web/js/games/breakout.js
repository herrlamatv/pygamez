/*
 * breakout.js - Breakout / Brick-Breaker Deluxe (Port von games/breakout.py)
 * ==========================================================================
 * - Steinsorten: Normal (1-3 Treffer), Stahl (unzerstörbar), Bombe
 *   (explodiert und reißt Nachbarn mit), Gold (viele Extrapunkte).
 * - Power-ups: Multi/Spread, Laser, Feuerball, Klebrig, Schild, Münze,
 *   breit/schmal, schnell/langsam, Extraleben.
 * - Combo-System mit steigendem Punkte-Multiplikator.
 * - Partikel, Ball-Spuren, Screen-Shake, Punkte-Popups.
 * - 25 Level mit Mustern (Herz, Wellen, Ringe, Kreuz, Rahmen, Zufall ...)
 *
 * Steuerung
 * ---------
 * - Setup: 1/2/3 = Schwierigkeit, Pfeil links/rechts = Ballfarbe,
 *          Hoch/Runter = Startlevel, M = Aufbau, Enter/Leertaste = Start
 *          (alles auch per Mausklick).
 * - Spiel: Maus oder Pfeil links/rechts bewegt den Schläger,
 *          Leertaste/Klick startet bzw. löst den haftenden Ball (und feuert Laser),
 *          P = Pause (Esc übernimmt die App).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const Rect = PG.Rect;

  // ---------------------------------------------------------------- Farben
  // Steinfarbe nach Resthärte (Treffer bis zur Zerstörung)
  const STR_COLORS = { 1: [120, 205, 120], 2: [235, 185, 80], 3: [230, 95, 95] };

  // Auswählbare Ballfarben (Name, RGB)
  const BALL_COLORS = [
    ["Gelb", [255, 230, 120]],
    ["Weiß", [240, 240, 240]],
    ["Cyan", [110, 230, 230]],
    ["Pink", [255, 120, 200]],
    ["Grün", [120, 240, 140]],
    ["Orange", [255, 160, 70]],
    ["Lila", [185, 130, 255]],
  ];

  // Schwierigkeitsgrade
  const DIFFICULTIES = {
    Easy: { lives: 5, paddle: 140, ball_speed: 300, drop: 0.42, bad: 0.12, hard_bonus: 0 },
    Medium: { lives: 3, paddle: 110, ball_speed: 365, drop: 0.32, bad: 0.28, hard_bonus: 0 },
    Hard: { lives: 2, paddle: 92, ball_speed: 440, drop: 0.26, bad: 0.42, hard_bonus: 1 },
  };
  const DIFF_ORDER = ["Easy", "Medium", "Hard"];

  // Level-Definitionen (tag / pat / rows / cols / base / drop / spec)
  const L = (tag, pat, rows, cols, base, drop, spec) => ({ tag, pat, rows, cols, base, drop, spec });
  const LEVEL_DEFS = [
    L("Normal", "full", 3, 11, 1, 1.0, 0.05),
    L("Normal", "checker", 4, 11, 1, 1.0, 0.06),
    L("Spass", "full", 6, 16, 1, 1.7, 0.10),
    L("Normal", "pyramid", 6, 11, 1, 1.0, 0.08),
    L("Schwer", "border", 6, 13, 2, 0.8, 0.12),
    L("Spass", "full", 8, 20, 1, 2.0, 0.14),
    L("Normal", "columns", 6, 11, 1, 1.0, 0.08),
    L("Schwer", "checker", 6, 11, 2, 0.8, 0.12),
    L("Spass", "waves", 8, 18, 1, 1.8, 0.12),
    L("Normal", "rows", 7, 11, 1, 1.0, 0.08),
    L("Schwer", "cross", 8, 13, 2, 0.8, 0.15),
    L("Spass", "heart", 8, 15, 1, 2.2, 0.10),
    L("Normal", "pyramid", 8, 13, 1, 1.0, 0.10),
    L("Schwer", "rings", 8, 15, 2, 0.7, 0.18),
    L("Spass", "checker", 8, 18, 1, 2.0, 0.12),
    L("Normal", "diamond", 8, 13, 2, 1.0, 0.12),
    L("Schwer", "border", 8, 15, 3, 0.6, 0.20),
    L("Spass", "random", 8, 20, 1, 2.4, 0.16),
    L("Schwer", "waves", 8, 15, 3, 0.6, 0.20),
    L("Normal", "heart", 9, 17, 2, 1.0, 0.12),
    L("Schwer", "cross", 9, 15, 3, 0.6, 0.22),
    L("Spass", "full", 9, 22, 1, 2.6, 0.18),
    L("Schwer", "rings", 9, 17, 3, 0.6, 0.24),
    L("Schwer", "full", 9, 13, 3, 0.55, 0.26),
    L("Schwer", "full", 9, 11, 3, 0.5, 0.30), // Finale
  ];
  const NUM_LEVELS = LEVEL_DEFS.length;

  const PADDLE_H = 16;
  const BALL_R = 8;
  const BALL_MIN_SPEED = 200;
  const BALL_MAX_SPEED = 760;
  const PADDLE_MIN = 60;
  const PADDLE_MAX = 220;
  const MAX_BALLS = 20;
  const POWERUP_FALL = 165; // Fallgeschwindigkeit der Power-ups (px/s)
  const LASER_SPEED = 620;
  const LASER_INTERVAL = 0.28; // Sekunden zwischen zwei Laser-Salven

  // Dauer der zeitlich begrenzten Effekte (Sekunden)
  const FX_DUR = { laser: 9.0, fire: 7.0, sticky: 11.0, shield: 10.0 };

  // Spielzustände
  const SETUP = "setup", PLAY = "play", PAUSE = "pause", OVER = "over";

  // pygame.Rect schneidet Koordinaten auf int ab
  const irect = (x, y, w, h) => new Rect(Math.trunc(x), Math.trunc(y), Math.trunc(w), Math.trunc(h));

  // ==================================================================== Ball
  /** Einzelner Ball mit Position, Geschwindigkeit und Spur. */
  class Ball {
    constructor(x, y, vx, vy) {
      this.x = x; this.y = y; this.vx = vx; this.vy = vy;
      this.trail = []; // letzte Positionen für die Ball-Spur
      this.stuck = false; // haftet gerade am Schläger (Klebrig)?
      this.stuckOff = 0.0; // Abstand zur Schlägermitte beim Haften
    }
    speed() {
      return Math.hypot(this.vx, this.vy);
    }
    scaleSpeed(faktor) {
      const s = this.speed() || 1.0;
      const neu = Math.max(BALL_MIN_SPEED, Math.min(BALL_MAX_SPEED, s * faktor));
      this.vx *= neu / s;
      this.vy *= neu / s;
    }
  }

  // ================================================================= PowerUp
  // Anzeige je Typ: (Beschriftung, Farbe, gut?)
  const PU_INFO = {
    multi: ["x2", [110, 230, 230], true],
    spread: ["x3", [110, 170, 255], true],
    speed: [">>", [255, 150, 70], false],
    slow: ["<<", [185, 130, 255], true],
    wide: ["W", [120, 240, 140], true],
    shrink: ["S", [230, 95, 95], false],
    life: ["+", [255, 120, 160], true],
    laser: ["L", [255, 90, 90], true],
    fire: ["F", [255, 170, 60], true],
    sticky: ["G", [150, 255, 150], true],
    shield: ["U", [120, 200, 255], true],
    coin: ["$", [255, 220, 90], true],
  };

  /** Herabfallendes Power-up. */
  class PowerUp {
    constructor(x, y, kind) {
      this.kind = kind;
      this.y = y; // exakte Fallposition (Rects sind int)
      this.rect = new Rect(Math.trunc(x) - 17, Math.trunc(y) - 11, 34, 22);
      this.phase = PG.rand.uniform(0, PG.TAU);
    }
  }

  // ================================================================== Brick
  const NORMAL = "normal", STEEL = "steel", BOMB = "bomb", GOLD = "gold";

  /** Ein einzelner Stein mit Sorte und Resthärte. */
  class Brick {
    constructor(rect, strength, kind = NORMAL) {
      this.rect = rect;
      this.strength = strength;
      this.kind = kind;
    }
    get breakable() {
      return this.kind !== STEEL;
    }
  }

  // =============================================================== Hauptspiel
  class BreakoutGame extends PG.Game {
    // ----- Setup / Zustand ---------------------------------------------
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.state = SETUP;

      // Setup-Auswahl
      this.diffName = "Medium";
      this.colorIndex = 0;
      this.startLevelIndex = 0;
      this.levelMode = "Standard"; // "Standard" (gemischt) oder "Voll"

      this.makeFonts();

      // Visuelle Extras
      this.particles = [];
      this.floaters = [];
      this.lasers = [];
      this.shake = 0.0;
      this.animT = 0.0;

      // Platzhalter, damit draw() vor dem ersten Start sicher ist
      this.bricks = [];
      this.balls = [];
      this.powerups = [];
      this.fx = { laser: 0.0, fire: 0.0, sticky: 0.0, shield: 0.0 };

      this.buildBg();
      this.buildSetupLayout();
    }

    /** Schriftgrößen aus der Fensterhöhe ableiten. */
    makeFonts() {
      const h = this.height;
      this.tiny = ui.font(Math.max(11, Math.floor(h / 36)));
      this.small = ui.font(Math.max(13, Math.floor(h / 30)));
      this.mid = ui.font(Math.max(16, Math.floor(h / 24)), true);
    }

    // ----- Hintergrund (Theme-Verlauf + eigene Funkel-Sterne) -----------
    buildBg() {
      const w = this.width, h = this.height;
      this.stars = [];
      for (let i = 0; i < 70; i++) {
        this.stars.push([PG.rand.randint(0, w), PG.rand.randint(0, h), PG.rand.uniform(0.4, 1.0), PG.rand.uniform(0.5, 1.6)]);
      }
    }

    blitBg(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      for (const [x, y, b, r] of this.stars) {
        const tw = 0.6 + 0.4 * Math.sin(this.animT * 1.5 + x);
        const col = ui.mix(ui.BG_TOP, ui.TEXT, 0.2 + 0.5 * b * tw);
        draw.circle(ctx, col, [x, y], Math.max(1, Math.trunc(r)));
      }
    }

    /** Theme-Farbe für den Level-Typ (zur Zeichenzeit ausgewertet). */
    tagColor(tag) {
      return { Normal: ui.TEXT_DIM, Schwer: ui.RED, Spass: ui.GREEN, Voll: ui.ACCENT }[tag] || ui.TEXT_DIM;
    }

    // ===== Setup-Screen =================================================
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      this.diffRects = {};
      DIFF_ORDER.forEach((name, i) => {
        this.diffRects[name] = new Rect(cx - 165 + i * 112, 132, 102, 50);
      });

      this.colorRects = [];
      const total = BALL_COLORS.length;
      const sw = 44;
      const start = cx - Math.floor((total * (sw + 6)) / 2);
      for (let i = 0; i < total; i++) this.colorRects.push(new Rect(start + i * (sw + 6), 262, sw, 44));

      this.startRect = new Rect(cx - 95, 352, 190, 52);
      this.modeRect = new Rect(cx - 165, 196, 330, 34);
      this.levelUpRect = new Rect(this.width - 56, 18, 38, 26);
      this.levelDownRect = new Rect(this.width - 56, 50, 38, 26);
    }

    drawSetup(ctx) {
      this.blitBg(ctx);
      const cx = Math.floor(this.width / 2);

      // Titel mit leichtem Glanz in der Akzentfarbe
      ui.text(ctx, "BREAKOUT", cx + 2, 58, this.bigFont, this.accent, "center");
      ui.text(ctx, "BREAKOUT", cx, 56, this.bigFont, ui.TEXT, "center");
      ui.text(ctx, t("bo.deluxe"), cx, 86, this.small, this.accent, "center");

      // Level-Wahl (oben rechts)
      const ld = this.levelDef(this.startLevelIndex);
      ui.text(ctx, t("bo.startlevel"), this.width - 170, 12, this.small, ui.TEXT_DIM);
      ui.text(ctx, `${String(this.startLevelIndex + 1).padStart(2, " ")}/${NUM_LEVELS}`, this.width - 170, 36, this.mid, ui.TEXT);
      ui.text(ctx, t("bo.tag." + ld.tag), this.width - 170, 62, this.small, this.tagColor(ld.tag));
      for (const [r, sym] of [[this.levelUpRect, "+"], [this.levelDownRect, "-"]]) {
        this.panel(ctx, r, ui.BTN);
        ui.text(ctx, sym, r.centerx, r.centery, this.mid, ui.TEXT, "center");
      }

      ui.text(ctx, t("bo.difficulty"), cx - 165, 104, this.mid, ui.TEXT_DIM);
      for (const name of DIFF_ORDER) {
        const r = this.diffRects[name];
        const aktiv = name === this.diffName;
        this.panel(ctx, r, aktiv ? ui.BTN_SEL : ui.BTN, aktiv ? this.accent : ui.BORDER);
        ui.text(ctx, t("bo.diff." + name.toLowerCase()), r.centerx, r.centery, this.mid, ui.TEXT, "center");
      }

      // Aufbau-Umschalter
      const voll = this.levelMode === "Voll";
      this.panel(ctx, this.modeRect, voll ? ui.BTN_SEL : ui.BTN);
      ui.text(ctx, voll ? t("bo.build_full") : t("bo.build_std"), this.modeRect.centerx, this.modeRect.centery, this.small, ui.TEXT, "center");

      ui.text(ctx, t("bo.ballcolor"), cx - 165, 234, this.mid, ui.TEXT_DIM);
      this.colorRects.forEach((r, i) => {
        draw.rect(ctx, BALL_COLORS[i][1], r, 0, 8);
        if (i === this.colorIndex) draw.rect(ctx, ui.TEXT, r.inflate(8, 8), 3, 10);
      });

      // Start-Knopf (pulsierend)
      const col = ui.mix(ui.GREEN, [255, 255, 255], ui.pulse(4, 0.0, 0.22));
      draw.rect(ctx, col, this.startRect, 0, 12);
      draw.rect(ctx, ui.TEXT, this.startRect, 2, 12);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.mid, ui.TEXT, "center");

      ui.text(ctx, t("bo.setup_hint"), cx, 428, this.tiny, ui.TEXT_DIM, "center");
      ui.text(ctx, t("bo.legend"), cx, 448, this.tiny, ui.TEXT_FAINT, "center");
    }

    panel(ctx, rect, fill, border) {
      draw.rect(ctx, fill, rect, 0, 8);
      draw.rect(ctx, border || ui.BORDER, rect, 1, 8);
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diffName = DIFF_ORDER[Number(k) - 1];
        } else if (k === "Left" || k === "a") {
          this.colorIndex = PG.mod(this.colorIndex - 1, BALL_COLORS.length);
        } else if (k === "Right" || k === "d") {
          this.colorIndex = PG.mod(this.colorIndex + 1, BALL_COLORS.length);
        } else if (k === "Up" || k === "w") {
          this.startLevelIndex = Math.min(NUM_LEVELS - 1, this.startLevelIndex + 1);
        } else if (k === "Down" || k === "s") {
          this.startLevelIndex = Math.max(0, this.startLevelIndex - 1);
        } else if (k === "m" || k === "M") {
          this.toggleMode();
        } else if (k === "Return" || k === "space") {
          this.playSound("select");
          this.startRun();
        }
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        for (const name of DIFF_ORDER) {
          if (this.diffRects[name].collidepoint(p)) {
            this.diffName = name;
            this.playSound("click");
          }
        }
        this.colorRects.forEach((r, i) => {
          if (r.collidepoint(p)) {
            this.colorIndex = i;
            this.playSound("click");
          }
        });
        if (this.levelUpRect.collidepoint(p)) {
          this.startLevelIndex = Math.min(NUM_LEVELS - 1, this.startLevelIndex + 1);
        } else if (this.levelDownRect.collidepoint(p)) {
          this.startLevelIndex = Math.max(0, this.startLevelIndex - 1);
        }
        if (this.modeRect.collidepoint(p)) this.toggleMode();
        if (this.startRect.collidepoint(p)) {
          this.playSound("select");
          this.startRun();
        }
      }
    }

    toggleMode() {
      this.levelMode = this.levelMode === "Standard" ? "Voll" : "Standard";
      this.playSound("click");
    }

    levelDef(index) {
      if (this.levelMode === "Voll") {
        const base = Math.min(3, 1 + Math.floor((index + 1) / 7));
        return { tag: "Voll", pat: "full", rows: 9, cols: 18, base, drop: 1.4, spec: 0.12 };
      }
      return LEVEL_DEFS[index];
    }

    // ===== Spiel starten / Level laden ==================================
    startRun() {
      this.cfg = DIFFICULTIES[this.diffName];
      this.ballColor = BALL_COLORS[this.colorIndex][1];
      this.score = 0;
      this.lives = this.cfg.lives;
      this.levelIndex = this.startLevelIndex;
      this.paddleW = this.cfg.paddle;
      this.won = false;
      this.gameOver = false;
      this.combo = 0;
      this.mult = 1;
      this.state = PLAY;
      this.loadLevel();
    }

    loadLevel() {
      this.levelDefCur = this.levelDef(this.levelIndex);
      this.bricks = this.makeLevel(this.levelDefCur);
      this.powerups = [];
      this.particles = [];
      this.lasers = [];
      this.floaters = [];
      this.fx = { laser: 0.0, fire: 0.0, sticky: 0.0, shield: 0.0 };
      this.laserCd = 0.0;
      this.introTimer = 2.2;
      this.combo = 0;
      this.mult = 1;
      this.resetPaddleBall();
    }

    resetPaddleBall() {
      this.paddleX = this.width / 2 - this.paddleW / 2;
      this.paddleY = this.height - 36;
      this.moveDir = 0;
      const spd = this.cfg.ball_speed;
      this.balls = [new Ball(this.width / 2, this.paddleY - BALL_R - 1, spd * 0.5, -spd)];
      this.ballHaengt = true;
      this.combo = 0;
      this.mult = 1;
    }

    /** Erzeugt die Steine für ein Level anhand seiner Definition 'd'. */
    makeLevel(d) {
      const { rows, cols, pat } = d;
      const base = Math.max(1, Math.min(3, d.base + this.cfg.hard_bonus));
      const spec = d.spec || 0.0;

      const gap = 4;
      const top = 62;
      const bw = Math.floor((this.width - (cols + 1) * gap) / cols);
      const bh = 22;

      const bricks = [];
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          if (!BreakoutGame.brickDa(pat, r, c, rows, cols)) continue;
          const extra = r >= rows - 2 && base >= 2 ? 1 : 0;
          let strength = Math.min(3, base + extra);
          const rect = new Rect(gap + c * (bw + gap), top + r * (bh + gap), bw, bh);

          let kind = NORMAL;
          if (PG.rand.random() < spec) {
            const roll = PG.rand.random();
            if (roll < 0.30) kind = STEEL;
            else if (roll < 0.65) kind = BOMB;
            else {
              kind = GOLD;
              strength = 1;
            }
          }
          bricks.push(new Brick(rect, strength, kind));
        }
      }
      return bricks;
    }

    /** Entscheidet, ob an (r,c) ein Stein steht (Muster). */
    static brickDa(pat, r, c, rows, cols) {
      const mitte = Math.floor(cols / 2);
      const rh = Math.floor(rows / 2);
      switch (pat) {
        case "full": return true;
        case "checker": return (r + c) % 2 === 0;
        case "pyramid": return Math.abs(c - mitte) <= r;
        case "columns": return c % 2 === 0;
        case "rows": return r % 2 === 0;
        case "diamond": return Math.abs(c - mitte) + Math.abs(r - rh) <= Math.max(mitte, rh) - 1;
        case "border": return r === 0 || r === rows - 1 || c === 0 || c === cols - 1;
        case "cross": return Math.abs(c - mitte) <= 1 || Math.abs(r - rh) <= 1;
        case "columns3": return c % 3 !== 1;
        case "waves": {
          // zwei sinusförmige Bänder
          const band = (rows - 1) / 2.0 + (rows / 3.0) * Math.sin(c * 0.9);
          return Math.abs(r - band) < 1.2;
        }
        case "rings": return (Math.abs(c - mitte) + Math.abs(r - rh)) % 3 !== 1;
        case "random": return PG.rand.random() < 0.72;
        case "heart": {
          // normierte Koordinaten, klassische Herz-Ungleichung
          const x = (c - (cols - 1) / 2.0) / (cols / 2.6);
          const y = ((rows - 1) / 2.0 - r) / (rows / 2.4) + 0.35;
          const q = x * x + y * y - 1;
          return q * q * q - x * x * (y * y * y) <= 0;
        }
        default: return true;
      }
    }

    // ===== Eingabe ======================================================
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetupEvent(ev);
        return;
      }

      if (this.state === OVER) {
        if ((ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space")) || ev.kind === "mousedown") {
          this.state = SETUP;
          this.gameOver = false;
        }
        return;
      }

      if (this.state === PAUSE) {
        if (ev.kind === "keydown" && ["p", "P", "Escape", "space"].includes(ev.key)) this.state = PLAY;
        return;
      }

      // PLAY
      if (ev.kind === "keydown") {
        if (ev.key === "Left" || ev.key === "a") this.moveDir = -1;
        else if (ev.key === "Right" || ev.key === "d") this.moveDir = 1;
        else if (ev.key === "p" || ev.key === "P" || ev.key === "Escape") {
          this.state = PAUSE;
          this.playSound("click");
        } else if (ev.key === "space") this.launchOrFire();
      } else if (ev.kind === "keyup") {
        // Taste losgelassen -> Schläger stoppt (sonst rutscht er bis an den Rand)
        if ((ev.key === "Left" || ev.key === "a") && this.moveDir === -1) this.moveDir = 0;
        else if ((ev.key === "Right" || ev.key === "d") && this.moveDir === 1) this.moveDir = 0;
      } else if (ev.kind === "mousemove") {
        this.paddleX = ev.pos[0] - this.paddleW / 2;
        this.moveDir = 0;
      } else if (ev.kind === "mousedown") {
        this.launchOrFire();
      }
    }

    /** Leertaste/Klick: haftenden Ball lösen und ggf. Laser feuern. */
    launchOrFire() {
      let released = false;
      if (this.ballHaengt) {
        this.ballHaengt = false;
        released = true;
      }
      for (const b of this.balls) {
        if (b.stuck) {
          b.stuck = false;
          released = true;
        }
      }
      if (!released && this.fx.laser > 0) this.fireLaser();
    }

    // ===== Logik ========================================================
    update(dt) {
      this.animT += dt;
      if (this.state !== PLAY) {
        this.updateParticles(dt); // Effekte laufen auch im Pause-Screen sanft aus
        return;
      }

      if (this.introTimer > 0) this.introTimer = Math.max(0.0, this.introTimer - dt);

      // Effekt-Timer herunterzählen
      for (const k in this.fx) if (this.fx[k] > 0) this.fx[k] = Math.max(0.0, this.fx[k] - dt);

      // Schläger bewegen (Tastatur)
      if (this.moveDir) this.paddleX += this.moveDir * 560 * dt;
      this.paddleX = Math.max(0, Math.min(this.width - this.paddleW, this.paddleX));
      const paddleRect = irect(this.paddleX, this.paddleY, this.paddleW, PADDLE_H);

      // Haftende / hängende Bälle folgen dem Schläger
      for (const b of this.balls) {
        if (this.ballHaengt || b.stuck) {
          const off = this.ballHaengt ? 0 : b.stuckOff;
          b.x = this.paddleX + this.paddleW / 2 + off;
          b.y = this.paddleY - BALL_R - 1;
        }
      }
      if (!this.ballHaengt) this.updateBalls(dt, paddleRect);

      // Laser automatisch nachladen/feuern
      if (this.fx.laser > 0) {
        this.laserCd -= dt;
        if (this.laserCd <= 0) {
          this.fireLaser();
          this.laserCd = LASER_INTERVAL;
        }
      }
      this.updateLasers(dt);
      this.updatePowerups(dt, paddleRect);
      this.updateParticles(dt);
      this.updateFloaters(dt);
      if (this.shake > 0) this.shake = Math.max(0.0, this.shake - 55 * dt);

      // Alle Bälle verloren?
      if (!this.balls.length) {
        this.lives -= 1;
        this.playSound("hit");
        this.shake = 10;
        if (this.lives <= 0) this.ende(false);
        else this.resetPaddleBall();
        return;
      }

      // Level geschafft? (Stahl zählt nicht)
      if (!this.bricks.some((b) => b.breakable)) {
        if (this.levelIndex + 1 < NUM_LEVELS) {
          this.levelIndex += 1;
          this.playSound("point");
          this.loadLevel();
        } else {
          this.ende(true);
        }
      }
    }

    updateBalls(dt, paddleRect) {
      const ueberlebende = [];
      const fire = this.fx.fire > 0;
      const shield = this.fx.shield > 0;
      for (const b of this.balls) {
        if (b.stuck) {
          ueberlebende.push(b);
          continue;
        }

        // Spur mitschreiben
        b.trail.push([b.x, b.y]);
        if (b.trail.length > 7) b.trail.shift();

        b.x += b.vx * dt;
        b.y += b.vy * dt;

        // Wände
        if (b.x - BALL_R <= 0) {
          b.x = BALL_R;
          b.vx = Math.abs(b.vx);
        } else if (b.x + BALL_R >= this.width) {
          b.x = this.width - BALL_R;
          b.vx = -Math.abs(b.vx);
        }
        if (b.y - BALL_R <= 0) {
          b.y = BALL_R;
          b.vy = Math.abs(b.vy);
        }

        // Schild (Auffangnetz) am unteren Rand
        if (shield && b.vy > 0 && b.y + BALL_R >= this.height - 6) {
          b.y = this.height - 6 - BALL_R;
          b.vy = -Math.abs(b.vy);
          this.playSound("bounce");
          this.spawnParticles(b.x, this.height - 6, [120, 200, 255], 6);
        }

        // Unten raus -> dieser Ball ist weg
        if (b.y - BALL_R > this.height) continue;

        const ballRect = irect(b.x - BALL_R, b.y - BALL_R, BALL_R * 2, BALL_R * 2);

        // Schläger
        if (ballRect.colliderect(paddleRect) && b.vy > 0) {
          let treff = (b.x - (this.paddleX + this.paddleW / 2)) / (this.paddleW / 2);
          treff = Math.max(-1, Math.min(1, treff));
          const speed = b.speed();
          const winkel = treff * (Math.PI / 3);
          b.vx = speed * Math.sin(winkel);
          b.vy = -Math.abs(speed * Math.cos(winkel));
          b.y = this.paddleY - BALL_R - 1;
          this.playSound("bounce");
          this.combo = 0; // Combo endet beim Schläger
          this.mult = 1;
          if (this.fx.sticky > 0) {
            b.stuck = true;
            b.stuckOff = treff * (this.paddleW / 2 - BALL_R);
          }
        }

        // Steine
        this.ballBricks(b, ballRect, fire);
        ueberlebende.push(b);
      }
      this.balls = ueberlebende;
    }

    ballBricks(b, ballRect, fire) {
      // Kopie durchlaufen: hitBrick/explode entfernen Steine aus der Liste.
      for (const brick of this.bricks.slice()) {
        if (!this.bricks.includes(brick)) continue; // durch Kettenexplosion schon weg
        if (!ballRect.colliderect(brick.rect)) continue;
        const rect = brick.rect;

        // Stahl: immer abprallen (auch beim Feuerball), nie zerstören
        if (brick.kind === STEEL) {
          if (!BreakoutGame.bounce(b, ballRect, rect)) continue; // fliegt schon hinaus
          this.playSound("bounce");
          return;
        }

        if (fire) {
          // Feuerball durchschlägt Steine ohne abzuprallen
          this.hitBrick(brick, true);
          continue;
        }

        // Überlappung vom letzten Treffer (Ball fliegt schon weg) = kein neuer Treffer
        if (!BreakoutGame.bounce(b, ballRect, rect)) continue;
        this.hitBrick(brick, false);
        return;
      }
    }

    /**
     * Abprall am Stein. Das Original kehrt nur die Richtung um (vx = -vx) und
     * wählt die Achse allein nach der kleineren Eindringtiefe. Streift der Ball
     * eine Kante, kippt die Richtung so Frame für Frame hin und her: er tunnelt
     * durch Stahl, trifft harte Steine mehrfach oder bleibt dauerhaft in einem
     * Stahlstein hängen. Deshalb: Achse und Richtung nach der Seite wählen, von
     * der der Ball kam (Position vor diesem Frame = letzter Spur-Punkt), und die
     * Geschwindigkeit immer VOM Stein WEG zeigen lassen.
     * Rückgabe: true, wenn der Ball wirklich abgeprallt ist (false = er verlässt
     * den Stein ohnehin schon, die Überlappung stammt noch vom letzten Frame).
     */
    static bounce(b, ballRect, rect) {
      const penL = ballRect.right - rect.left, penR = rect.right - ballRect.left;
      const penT = ballRect.bottom - rect.top, penB = rect.bottom - ballRect.top;
      let axisX = Math.min(penL, penR) < Math.min(penT, penB);
      let dirX = penL < penR ? -1 : 1;
      let dirY = penT < penB ? -1 : 1;
      const prev = b.trail.length ? b.trail[b.trail.length - 1] : null;
      if (prev) {
        const pr = irect(prev[0] - BALL_R, prev[1] - BALL_R, BALL_R * 2, BALL_R * 2);
        const overX = pr.x < rect.right && rect.x < pr.right;
        const overY = pr.y < rect.bottom && rect.y < pr.bottom;
        if (!overX) dirX = pr.x < rect.x ? -1 : 1;
        if (!overY) dirY = pr.y < rect.y ? -1 : 1;
        if (overX && !overY) axisX = false; // kam von oben/unten
        else if (overY && !overX) axisX = true; // kam von der Seite
      }
      // nie entlang einer Achse ohne Tempo "abprallen"
      if (axisX && b.vx === 0) axisX = false;
      else if (!axisX && b.vy === 0) axisX = true;
      if (axisX) {
        const vx = dirX * Math.abs(b.vx);
        const hit = vx !== b.vx;
        b.vx = vx;
        return hit;
      }
      const vy = dirY * Math.abs(b.vy);
      const hit = vy !== b.vy;
      b.vy = vy;
      return hit;
    }

    /** Fügt einem Stein Schaden zu; zerstört ihn ggf. inkl. Effekten. */
    hitBrick(brick, full) {
      const idx = this.bricks.indexOf(brick);
      if (idx < 0) return;
      const dmg = full ? brick.strength : 1;
      brick.strength -= dmg;
      if (brick.strength > 0) {
        this.score += 5;
        this.playSound("eat");
        return;
      }
      // WICHTIG: erst entfernen, dann poppen - sonst erwischt die
      // Bomben-Kettenexplosion denselben Stein ein zweites Mal.
      this.bricks.splice(idx, 1);
      this.popBrick(brick, true);
    }

    /** Stein zerstören: Punkte, Combo, Partikel, evtl. Kettenexplosion. */
    popBrick(brick, chain) {
      this.combo += 1;
      this.mult = Math.min(8, 1 + Math.floor(this.combo / 4));

      let base, col;
      if (brick.kind === GOLD) {
        base = 60;
        col = [255, 215, 90];
      } else if (brick.kind === BOMB) {
        base = 25;
        col = [255, 130, 70];
      } else {
        base = 10;
        col = STR_COLORS[Math.max(1, brick.strength + 1)] || [200, 200, 200];
      }

      const pts = base * this.mult;
      this.score += pts;
      const [bx, by] = brick.rect.center;
      this.addFloater(bx, by, `+${pts}`, col);
      this.spawnParticles(bx, by, col, 10);
      this.playSound("explode");
      this.maybeDrop([bx, by]);

      if (brick.kind === BOMB && chain) {
        this.playSound("hit");
        this.shake = Math.max(this.shake, 12);
        this.explode([bx, by], 70);
      }
    }

    /** Reißt alle Steine im Umkreis mit (Bomben-Kettenreaktion). */
    explode(center, radius) {
      const [cx, cy] = center;
      this.spawnParticles(cx, cy, [255, 150, 60], 22, 260);
      const opfer = this.bricks.filter((b) => b.breakable && Math.hypot(b.rect.centerx - cx, b.rect.centery - cy) <= radius);
      for (const b of opfer) {
        const i = this.bricks.indexOf(b);
        if (i >= 0) {
          this.bricks.splice(i, 1);
          this.popBrick(b, b.kind === BOMB);
        }
      }
    }

    /** Feuert zwei Laserschüsse von den Schlägerenden nach oben. */
    fireLaser() {
      const y = this.paddleY - 4;
      this.lasers.push([this.paddleX + 8, y]);
      this.lasers.push([this.paddleX + this.paddleW - 8, y]);
      this.playSound("shoot");
    }

    updateLasers(dt) {
      const bleibt = [];
      for (const shot of this.lasers) {
        shot[1] -= LASER_SPEED * dt;
        if (shot[1] < 0) continue;
        const r = new Rect(Math.trunc(shot[0]) - 2, Math.trunc(shot[1]) - 10, 4, 12);
        let getroffen = false;
        for (const brick of this.bricks) {
          if (r.colliderect(brick.rect)) {
            // Stahl schluckt den Schuss
            if (brick.kind !== STEEL) this.hitBrick(brick, false);
            getroffen = true;
            break;
          }
        }
        if (!getroffen) bleibt.push(shot);
      }
      this.lasers = bleibt;
    }

    maybeDrop(pos) {
      const chance = Math.min(0.95, this.cfg.drop * this.levelDefCur.drop);
      if (PG.rand.random() > chance) return;
      const gut = ["multi", "spread", "wide", "life", "slow", "laser", "fire", "sticky", "shield", "coin"];
      const schlecht = ["shrink", "speed"];
      const kind = PG.rand.random() < this.cfg.bad ? PG.rand.choice(schlecht) : PG.rand.choice(gut);
      this.powerups.push(new PowerUp(pos[0], pos[1], kind));
    }

    updatePowerups(dt, paddleRect) {
      const bleibt = [];
      for (const p of this.powerups) {
        // Fallposition als float führen
        p.y += POWERUP_FALL * dt;
        p.rect.centery = Math.trunc(p.y);
        if (p.rect.colliderect(paddleRect)) this.activate(p.kind);
        else if (p.rect.top <= this.height) bleibt.push(p);
      }
      this.powerups = bleibt;
    }

    activate(kind) {
      const [label, col, gut] = PU_INFO[kind];
      this.addFloater(this.paddleX + this.paddleW / 2, this.paddleY - 18, label, col);
      this.playSound(gut ? "point" : "hit");

      if (kind === "life") {
        this.lives += 1;
      } else if (kind === "wide") {
        this.paddleW = Math.min(PADDLE_MAX, this.paddleW + 30);
      } else if (kind === "shrink") {
        this.paddleW = Math.max(PADDLE_MIN, this.paddleW - 26);
      } else if (kind === "speed") {
        for (const b of this.balls) b.scaleSpeed(1.25);
      } else if (kind === "slow") {
        for (const b of this.balls) b.scaleSpeed(0.80);
      } else if (kind === "coin") {
        this.score += 250;
      } else if (kind === "laser" || kind === "fire" || kind === "sticky" || kind === "shield") {
        this.fx[kind] = FX_DUR[kind];
        if (kind === "laser") this.laserCd = 0.0;
      } else if (kind === "multi") {
        const neu = [];
        for (const b of this.balls) {
          if (b.stuck) continue;
          if (this.balls.length + neu.length < MAX_BALLS) neu.push(new Ball(b.x, b.y, -b.vx, b.vy));
        }
        this.balls.push(...neu);
      } else if (kind === "spread") {
        let aktive = this.balls.filter((b) => !b.stuck);
        if (!aktive.length) aktive = this.balls;
        if (aktive.length) {
          const quelle = aktive[0];
          const spd = quelle.speed() || this.cfg.ball_speed;
          for (const da of [-0.5, 0.5]) {
            if (this.balls.length < MAX_BALLS) {
              this.balls.push(new Ball(quelle.x, quelle.y, spd * Math.sin(da), -Math.abs(spd * Math.cos(da))));
            }
          }
        }
      }
    }

    // ----- Partikel / Popups -------------------------------------------
    spawnParticles(x, y, color, n, spread = 180) {
      for (let i = 0; i < n; i++) {
        const ang = PG.rand.uniform(0, PG.TAU);
        const spd = PG.rand.uniform(40, spread);
        this.particles.push([x, y, Math.cos(ang) * spd, Math.sin(ang) * spd, PG.rand.uniform(0.3, 0.7), color, PG.rand.uniform(2, 4)]);
      }
    }

    updateParticles(dt) {
      const bleibt = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] += 260 * dt; // Schwerkraft
        p[4] -= dt;
        if (p[4] > 0) bleibt.push(p);
      }
      this.particles = bleibt;
    }

    addFloater(x, y, text, color) {
      this.floaters.push([x, y, text, color, 0.9]);
    }

    updateFloaters(dt) {
      const bleibt = [];
      for (const f of this.floaters) {
        f[1] -= 34 * dt;
        f[4] -= dt;
        if (f[4] > 0) bleibt.push(f);
      }
      this.floaters = bleibt;
    }

    ende(gewonnen) {
      this.won = gewonnen;
      this.state = OVER;
      this.gameOver = true;
      if (gewonnen) {
        this.reportResult(true);
        this.achEvent("breakout_clear");
      }
      this.playSound(gewonnen ? "win" : "gameover");
      this.rumble(220);
    }

    // ===== Zeichnen =====================================================
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      // Szene mit Screen-Shake
      ctx.save();
      // (im Endstand ruht die Szene - update() läuft bei gameOver nicht mehr)
      if (this.shake > 0.5 && this.state !== OVER) {
        const dx = PG.rand.uniform(-this.shake, this.shake);
        const dy = PG.rand.uniform(-this.shake, this.shake);
        draw.rect(ctx, ui.BG_BOTTOM, [0, 0, this.width, this.height]);
        ctx.translate(dx, dy);
      }
      this.drawScene(ctx);
      ctx.restore();

      if (this.state === PAUSE) this.drawPause(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawScene(ctx) {
      this.blitBg(ctx);

      // Schild-Netz
      if ((this.state === PLAY || this.state === PAUSE) && this.fx.shield > 0) {
        const a = 120 + Math.trunc(80 * Math.sin(this.animT * 6));
        draw.rect(ctx, [110, 190, 255, Math.max(60, a)], [0, this.height - 6, this.width, 6]);
      }

      // Steine
      for (const brick of this.bricks) this.drawBrick(ctx, brick);

      // Partikel
      for (const p of this.particles) {
        const a = Math.max(0, Math.min(255, Math.trunc(255 * (p[4] / 0.7))));
        const rad = Math.trunc(p[6]);
        draw.circle(ctx, [p[5][0], p[5][1], p[5][2], a], [p[0] - p[6] + rad, p[1] - p[6] + rad], rad);
      }

      // Ball-Spuren
      for (const b of this.balls) {
        const n = b.trail.length;
        b.trail.forEach(([tx, ty], j) => {
          const a = Math.trunc((90 * (j + 1)) / n);
          draw.circle(ctx, [this.ballColor[0], this.ballColor[1], this.ballColor[2], a], [Math.trunc(tx), Math.trunc(ty)], Math.max(2, Math.trunc((BALL_R * (j + 1)) / n)));
        });
      }

      // Laser
      for (const shot of this.lasers) {
        draw.rect(ctx, [255, 90, 90], [Math.trunc(shot[0]) - 2, Math.trunc(shot[1]) - 10, 4, 12], 0, 2);
      }

      // Schläger
      this.drawPaddle(ctx);

      // Bälle (Feuerball andersfarbig)
      const fire = this.fx.fire > 0;
      for (const b of this.balls) {
        const col = fire ? [255, 150, 40] : this.ballColor;
        if (fire) draw.circle(ctx, [255, 120, 30, 90], [b.x, b.y], BALL_R * 2);
        draw.circle(ctx, col, [Math.trunc(b.x), Math.trunc(b.y)], BALL_R);
        draw.circle(ctx, [255, 255, 255], [Math.trunc(b.x - 2), Math.trunc(b.y - 2)], 2);
      }

      // Power-ups
      for (const p of this.powerups) this.drawPowerup(ctx, p);

      // Punkte-Popups
      for (const f of this.floaters) {
        const a = Math.max(0, Math.min(255, Math.trunc(255 * (f[4] / 0.9))));
        ui.text(ctx, f[2], Math.trunc(f[0]), Math.trunc(f[1]), this.small, f[3], "center", a / 255);
      }

      this.drawHud(ctx);

      if (this.ballHaengt && this.introTimer <= 0) {
        ui.text(ctx, t("bo.start_ball"), Math.floor(this.width / 2), Math.floor(this.height / 2) + 60, this.font, ui.TEXT, "center");
      }

      // Level-Intro-Banner
      if (this.introTimer > 0) this.drawIntro(ctx);
    }

    drawPaddle(ctx) {
      const rect = irect(this.paddleX, this.paddleY, this.paddleW, PADDLE_H);
      let base = ui.TEXT;
      if (this.fx.sticky > 0) base = [150, 240, 160];
      draw.rect(ctx, base, rect, 0, 8);
      draw.rect(ctx, this.accent, rect, 2, 8);
      // Laser-Kanonen
      if (this.fx.laser > 0) {
        for (const gx of [rect.left + 8, rect.right - 8]) {
          draw.rect(ctx, [255, 90, 90], [gx - 3, rect.top - 8, 6, 8], 0, 2);
        }
      }
    }

    drawBrick(ctx, brick) {
      const rect = brick.rect;
      if (brick.kind === STEEL) {
        draw.rect(ctx, [120, 128, 140], rect, 0, 4);
        draw.rect(ctx, [170, 178, 190], rect, 2, 4);
        for (const [bx, by] of [[rect.left + 5, rect.top + 5], [rect.right - 5, rect.top + 5], [rect.left + 5, rect.bottom - 5], [rect.right - 5, rect.bottom - 5]]) {
          draw.circle(ctx, [80, 86, 96], [bx, by], 2);
        }
        return;
      }
      if (brick.kind === GOLD) {
        draw.rect(ctx, [245, 200, 70], rect, 0, 4);
        draw.rect(ctx, [255, 240, 160], rect, 2, 4);
        ui.text(ctx, "$", rect.centerx, rect.centery, this.small, [120, 80, 10], "center");
        return;
      }
      if (brick.kind === BOMB) {
        draw.rect(ctx, [70, 40, 40], rect, 0, 4);
        draw.rect(ctx, [230, 90, 60], rect, 2, 4);
        draw.circle(ctx, [230, 90, 60], rect.center, 5);
        return;
      }
      const col = STR_COLORS[brick.strength] || [200, 200, 200];
      draw.rect(ctx, col, rect, 0, 4);
      // oberer Glanzstreifen
      draw.rect(ctx, [255, 255, 255, 55], [rect.left + 2, rect.top + 2, rect.w - 4, 4]);
      if (brick.strength > 1) {
        ui.text(ctx, String(brick.strength), rect.centerx, rect.centery, this.small, [20, 20, 20], "center");
      }
    }

    drawPowerup(ctx, p) {
      const [label, farbe, gut] = PU_INFO[p.kind];
      const bob = Math.trunc(2 * Math.sin(this.animT * 6 + p.phase));
      const r = p.rect.move(0, bob);
      draw.rect(ctx, [farbe[0], farbe[1], farbe[2], 70], [r.left - 4, r.top - 4, r.w + 8, r.h + 8], 0, 8);
      draw.rect(ctx, farbe, r, 0, 6);
      draw.rect(ctx, gut ? [20, 20, 30] : [30, 10, 10], r, 2, 6);
      ui.text(ctx, label, r.centerx, r.centery, this.small, [20, 20, 20], "center");
    }

    drawHud(ctx) {
      // Halbtransparenter Panel-Streifen oben (im Theme-Ton)
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 150], [0, 0, this.width, 34]);
      draw.line(ctx, ui.BORDER, [0, 33.5], [this.width, 33.5]);

      ui.text(ctx, t("common.points", { score: this.score }), 10, 6, this.font, ui.TEXT);
      const tag = this.levelDefCur.tag;
      ui.text(ctx, t("bo.hud", { diff: t("bo.diff." + this.diffName.toLowerCase()), level: this.levelIndex + 1, total: NUM_LEVELS, tag: t("bo.tag." + tag) }),
        Math.floor(this.width / 2), 8, this.small, this.tagColor(tag), "midtop");

      // Leben als Herzen
      const heart = [230, 80, 110];
      for (let i = 0; i < Math.min(this.lives, 8); i++) {
        const hx = this.width - 20 - i * 20;
        draw.circle(ctx, heart, [hx - 3, 14], 4);
        draw.circle(ctx, heart, [hx + 3, 14], 4);
        draw.polygon(ctx, heart, [[hx - 6, 15], [hx + 6, 15], [hx, 23]]);
      }
      if (this.lives > 8) ui.text(ctx, `x${this.lives}`, this.width - 20 - 8 * 20 - 24, 8, this.tiny, ui.TEXT);

      // Combo-Anzeige
      if (this.combo > 1) {
        ui.text(ctx, t("bo.combo", { mult: this.mult }), Math.floor(this.width / 2), 38, this.mid, ui.GOLD, "midtop");
      }

      // Aktive Effekte mit Restzeit-Balken (unten links)
      const y = this.height - 22;
      const order = [["laser", "L", [255, 90, 90]], ["fire", "F", [255, 170, 60]], ["sticky", "G", [150, 255, 150]], ["shield", "U", [120, 200, 255]]];
      let x = 8;
      for (const [key, lab, col] of order) {
        if (this.fx[key] <= 0) continue;
        const frac = this.fx[key] / FX_DUR[key];
        draw.rect(ctx, ui.PANEL, [x, y, 54, 14], 0, 4);
        draw.rect(ctx, col, [x, y, Math.trunc(54 * frac), 14], 0, 4);
        ui.text(ctx, lab, x + 4, y, this.tiny, [10, 10, 10]);
        x += 60;
      }
    }

    drawIntro(ctx) {
      const a = this.introTimer < 0.4 ? Math.min(1.0, this.introTimer / 0.4) : 1.0;
      const alpha = Math.trunc(220 * Math.min(1.0, a));
      const top = Math.floor(this.height / 2) - 45;
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], Math.trunc(alpha * 0.7)], [0, top, this.width, 90]);
      const ac = [this.accent[0], this.accent[1], this.accent[2], alpha];
      draw.rect(ctx, ac, [0, top, this.width, 2]);
      draw.rect(ctx, ac, [0, top + 88, this.width, 2]);
      const tag = this.levelDefCur.tag;
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      ui.text(ctx, t("bo.level_intro", { level: this.levelIndex + 1 }), cx, cy - 10, this.bigFont, ui.TEXT, "center", alpha / 255);
      ui.text(ctx, `[${t("bo.tag." + tag)}]`, cx, cy + 26, this.mid, this.tagColor(tag), "center", alpha / 255);
    }

    drawPause(ctx) {
      draw.rect(ctx, [0, 0, 0, 170], [0, 0, this.width, this.height]);
      this.drawCenterText(ctx, t("app.pause"), this.bigFont, ui.TEXT, -10);
      this.drawCenterText(ctx, t("bo.pause_resume"), this.font, ui.TEXT_DIM, 40);
    }

    drawOver(ctx) {
      draw.rect(ctx, [0, 0, 0, 170], [0, 0, this.width, this.height]);
      if (this.won) this.drawCenterText(ctx, t("bo.cleared"), this.bigFont, ui.GREEN, -30);
      else this.drawCenterText(ctx, t("common.game_over"), this.bigFont, ui.RED, -30);
      this.drawCenterText(ctx, t("common.points", { score: this.score }), this.font, ui.TEXT, 18);
      this.drawCenterText(ctx, t("bo.back_to_select"), this.font, ui.TEXT_DIM, 52);
    }
  }

  PG.register(BreakoutGame, {
    id: "BreakoutGame",
    key: "breakout",
    name: "Breakout",
  });
})();
