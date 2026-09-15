/*
 * pacman.js - Pac-Man, möglichst originalgetreuer Klon (Port von games/pacman.py)
 * ================================================================================
 * - Klassisches 28x31-Labyrinth mit Pillen, 4 Power-Pillen, Tunnel-Warp und
 *   Geisterhaus in der Mitte.
 * - Vier Geister mit den ORIGINAL-Verhaltensweisen (Ziel-Kachel-KI):
 *     Blinky jagt direkt, Pinky zielt 4 Kacheln voraus, Inky nutzt den
 *     Blinky-Vektor, Clyde jagt aus der Ferne und weicht aus der Nähe aus.
 * - Scatter/Chase-Phasen; bei jedem Moduswechsel drehen die Geister um.
 * - Power-Pille -> Frightened (200/400/800/1600), Augen kehren heim.
 * - Früchte, 3 Leben, Extraleben bei 10.000, Level, Death-Animation.
 * - Setup: Schwierigkeit (Normal/Schwer/Extrem).
 *
 * Steuerung: Pfeile oder WASD.  Enter = nach Game Over neu,  S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ------------------------------------------------------------------ Labyrinth
  // '#' Wand, '.' Pille, 'o' Power-Pille, ' ' frei, '=' Geisterhaus-Tür,
  // 't' Tunnel (frei, kein Punkt).  28 Spalten x 31 Zeilen.
  const MAZE = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.#####.##.#####.######",
    "######.#####.##.#####.######",
    "######.##..........##.######",
    "######.##.###==###.##.######",
    "######.##.#      #.##.######",
    "tttttt....#      #....tttttt",
    "######.##.#      #.##.######",
    "######.##.########.##.######",
    "######.##..........##.######",
    "######.##.########.##.######",
    "######.##.########.##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......  .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
  ];
  const COLS = 28, ROWS = 31;

  // Kacheln, die beim Aufbau von Pillen befreit werden (Geister-Startbereich).
  const CLEAR_TILES = new Set(["13,11", "14,11"]);

  // Startpositionen
  const PAC_START = [13, 23];
  const DOOR_EXIT = [13, 11];              // Kachel direkt über der Tür (Aus-/Eingang)
  const GHOST_HOMES = { blinky: [13, 11], pinky: [13, 14], inky: [11, 14], clyde: [16, 14] };
  const GHOST_SCATTER = { blinky: [25, 0], pinky: [2, 0], inky: [27, 30], clyde: [0, 30] };
  const GHOST_COLORS = { blinky: [255, 70, 70], pinky: [255, 165, 220], inky: [70, 220, 240], clyde: [255, 175, 70] };
  const GHOST_RELEASE = { blinky: -1.0, pinky: 2.0, inky: 7.0, clyde: 13.0 };

  // Richtungen + klassische Vorrang-Reihenfolge (hoch, links, runter, rechts)
  const UP = [0, -1], LEFT = [-1, 0], DOWN = [0, 1], RIGHT = [1, 0];
  const ORDER = [UP, LEFT, DOWN, RIGHT];

  // Scatter/Chase-Zeitplan (Sekunden, Modus)
  const SCHEDULE = [[7, "scatter"], [20, "chase"], [7, "scatter"], [20, "chase"],
    [5, "scatter"], [20, "chase"], [5, "scatter"], [1e9, "chase"]];

  // Tempo in Kacheln/Sekunde
  const SPD_PAC = 8.4;
  const SPD_GHOST = 7.4;
  const SPD_FRIGHT = 4.8;
  const SPD_EYES = 16.0;
  const SPD_HOUSE = 3.6;
  const SPD_TUNNEL = 4.4;

  const FRUIT_TABLE = [
    ["KIRSCHE", 100, [235, 60, 60]],
    ["ERDBEERE", 300, [235, 80, 110]],
    ["ORANGE", 500, [245, 160, 50]],
    ["APFEL", 700, [220, 50, 50]],
    ["MELONE", 1000, [120, 220, 120]],
    ["GALAXIAN", 2000, [120, 180, 255]],
    ["GLOCKE", 3000, [245, 220, 80]],
    ["SCHLÜSSEL", 5000, [200, 210, 230]],
  ];

  const DIFFS = [
    { key: "normal", gspeed: 1.0, fright: 6.0 },
    { key: "hard", gspeed: 1.07, fright: 4.5 },
    { key: "extreme", gspeed: 1.13, fright: 3.5 },
  ];

  // Identitätsfarben (Labyrinth-Blau, Pillen, Pac-Gelb, Frightened-Blau) -
  // bewusst NICHT ans Theme gekoppelt.
  const COL_BG = [0, 0, 0];
  const COL_WALL = [36, 46, 190];
  const COL_WALL_HI = [80, 110, 255];
  const COL_DOOR = [255, 180, 210];
  const COL_PILL = [250, 220, 170];
  const COL_POWER = [255, 200, 120];
  const COL_PAC = [255, 235, 50];
  const COL_FRIGHT = [36, 40, 210];
  const COL_FRIGHT_END = [245, 245, 255];

  const SETUP = "setup", READY = "ready", PLAY = "play", DYING = "dying", LEVELCLEAR = "levelclear", GAMEOVER = "gameover";

  const add = (a, b) => [a[0] + b[0], a[1] + b[1]];
  const opp = (d) => [-d[0], -d[1]];
  const eq = (a, b) => a[0] === b[0] && a[1] === b[1];
  const dist2 = (a, b) => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2;

  /** Gemeinsamer Zustand von Pac-Man und Geistern (Kachel + Bruchteil). */
  function makeMover(tile, dir) {
    return { tile: tile.slice(), dir, frac: 0.0 };
  }

  function makeGhost(name) {
    const g = makeMover(GHOST_HOMES[name], LEFT);
    g.name = name;
    g.color = GHOST_COLORS[name];
    g.home = GHOST_HOMES[name];
    g.scatter = GHOST_SCATTER[name];
    g.releaseAt = GHOST_RELEASE[name];
    g.state = name === "blinky" ? "maze" : "house";
    g.frightened = false;
    g.bobT = PG.rand.uniform(0, PG.TAU);
    return g;
  }

  class PacmanGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.diff = Math.max(0, Math.min(2, parseInt(this.opts.difficulty, 10) || 0));

      this.mazeCache = {};          // (Flash, Pixel-Skala) -> Labyrinth-Canvas
      this.layout();
      this.hs = this.highscore;
      this.animT = 0.0;
      this.popups = [];
      this.fruitHistory = [];

      this.buildSetupLayout();
      this.newGame();
      this.state = SETUP;
    }

    /** Zellgröße/Ursprung an die Fläche anpassen (zentriert). */
    layout() {
      const cell = Math.min(Math.floor(this.width / COLS), Math.floor(this.height / (ROWS + 5)));
      this.CELL = Math.max(6, cell);
      this.mazeW = COLS * this.CELL;
      this.mazeH = ROWS * this.CELL;
      this.ox = Math.floor((this.width - this.mazeW) / 2);
      const top = Math.floor(2.2 * this.CELL);
      this.oy = top + Math.max(0, Math.floor((this.height - top - this.mazeH - 2 * this.CELL) / 2));

      // Themen-Schriften, Größen aus der Zellgröße abgeleitet;
      // Punktestände in Monospace, damit die Ziffern nicht "zappeln".
      const c = this.CELL;
      this.fFont = ui.font(Math.max(11, Math.floor(c * 1.15)), true);
      this.fDigit = ui.font(Math.max(11, Math.floor(c * 1.15)), true, true);
      this.fSmall = ui.font(Math.max(10, Math.floor(c * 0.95)));
      this.fTiny = ui.font(Math.max(9, Math.floor(c * 0.8)));
      this.fBig = ui.font(Math.max(20, Math.floor(c * 2.0)), true);
    }

    // ----- Level / Positionen -------------------------------------------
    newGame() {
      this.level = 1;
      this.score = 0;
      this.lives = 3;
      this.extraAwarded = false;
      this.gameOver = false;
      this.fruitHistory = [];
      this.buildLevel();
      this.resetPositions(2.2);
    }

    /** Baut Wände + Pillen für das aktuelle Level neu auf. */
    buildLevel() {
      this.grid = MAZE.map((row) => row.split(""));
      this.pellets = Array.from({ length: ROWS }, () => new Array(COLS).fill(0));
      let total = 0;
      for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) {
          const ch = this.grid[r][c];
          if (CLEAR_TILES.has(c + "," + r)) continue;
          if (ch === ".") {
            this.pellets[r][c] = 1;
            total++;
          } else if (ch === "o") {
            this.pellets[r][c] = 2;
            total++;
          }
        }
      }
      this.dotsTotal = total;
      this.dotsEaten = 0;
      this.fruit = null;
      this.fruitThresholds = [70, 170];
    }

    resetPositions(readyTime) {
      this.pac = makeMover(PAC_START, LEFT);
      this.pac.want = LEFT;
      this.pac.moving = false;
      this.pac.mouth = 0.2;
      this.ghosts = ["blinky", "pinky", "inky", "clyde"].map(makeGhost);
      this.playTimer = 0.0;
      this.phase = 0;
      this.modeTimer = SCHEDULE[0][0];
      this.frightTimer = 0.0;
      this.ghostCombo = 0;
      this.freeze = 0.0;
      this.state = READY;
      this.readyTimer = readyTime;
      this.dyingTimer = 0.0;
    }

    get globalMode() {
      return SCHEDULE[this.phase][1];
    }

    frightTime() {
      return Math.max(1.0, DIFFS[this.diff].fright - (this.level - 1) * 0.4);
    }

    levelFactor() {
      return Math.min(1.28, 1.0 + (this.level - 1) * 0.035);
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(420, this.width - 60);
      const top = Math.max(150, Math.floor(this.height * 0.30));
      this.diffPanel = new PG.Rect(cx - Math.floor(bw / 2), top, bw, 60);
      this.diffLeft = new PG.Rect(this.diffPanel.left, top, 42, 60);
      this.diffRight = new PG.Rect(this.diffPanel.right - 42, top, 42, 60);
      this.startRect = new PG.Rect(cx - 95, top + 88, 190, 52);
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
      } else if (ev.kind === "mousedown" && ev.pos) {
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
    dirFromKey(key) {
      if (this.isAction(key, "up") || key === "Up") return UP;
      if (this.isAction(key, "down") || key === "Down") return DOWN;
      if (this.isAction(key, "left") || key === "Left") return LEFT;
      if (this.isAction(key, "right") || key === "Right") return RIGHT;
      return null;
    }

    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (ev.kind !== "keydown") return;
      if (this.state === GAMEOVER) {
        if (ev.key === "Return" || ev.key === "space") this.newGame();
        else if (ev.key === "s" || ev.key === "S") {
          // Web: Game-Over-Flag zurücknehmen, sonst liegt das Highscore-Banner der App über dem Setup
          this.gameOver = false;
          this.state = SETUP;
          this.playSound("click");
        }
        return;
      }
      const d = this.dirFromKey(ev.key);
      if (d && (this.state === READY || this.state === PLAY)) {
        this.pac.want = d;
        if (eq(d, opp(this.pac.dir))) this.reverse(this.pac); // sofortiges Umkehren fühlt sich besser an
      }
    }

    // ===================================================== Wände / Wege
    wrapt(tile) {
      return [PG.mod(tile[0], COLS), tile[1]];
    }

    pacOpen(tile) {
      const [c, r] = tile;
      if (r < 0 || r >= ROWS) return false;
      const ch = this.grid[r][PG.mod(c, COLS)];
      return ch !== "#" && ch !== "=";
    }

    ghostOpen(g, tile) {
      const [c, r] = tile;
      if (r < 0 || r >= ROWS) return false;
      const ch = this.grid[r][PG.mod(c, COLS)];
      if (ch === "#") return false;
      if (ch === "=") return g.state === "leaving" || g.state === "eyes" || g.state === "entering";
      return true;
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.animT += dt;
      this.updatePopups(dt);
      if (this.state === SETUP || this.state === GAMEOVER) return;
      if (this.state === READY) {
        this.readyTimer -= dt;
        this.animPac(dt, false);
        if (this.readyTimer <= 0) this.state = PLAY;
        return;
      }
      if (this.state === LEVELCLEAR) {
        this.levelclearTimer -= dt;
        if (this.levelclearTimer <= 0) {
          this.level += 1;
          this.buildLevel();
          this.resetPositions(1.6);
        }
        return;
      }
      if (this.state === DYING) {
        this.dyingTimer -= dt;
        if (this.dyingTimer <= 0) this.afterDeath();
        return;
      }
      if (this.freeze > 0) {
        this.freeze -= dt;
        return;
      }

      this.playTimer += dt;
      this.updateModes(dt);
      this.releaseGhosts();

      this.movePac(dt);
      this.animPac(dt, this.pac.moving);
      for (const g of this.ghosts) {
        if (g.state === "house") g.bobT += dt * 3.0;
        else this.moveGhost(g, dt);
      }

      this.collisions();
      this.updateFruit(dt);
      this.checkExtraLife();
      if (this.dotsEaten >= this.dotsTotal) {
        this.state = LEVELCLEAR;
        this.levelclearTimer = 1.8;
        this.achEvent("pacman_clear");
        this.playSound("win");
      }
    }

    updateModes(dt) {
      if (this.frightTimer > 0) {
        this.frightTimer -= dt;
        if (this.frightTimer <= 0) {
          for (const g of this.ghosts) if (g.state === "maze") g.frightened = false;
        }
        return;
      }
      this.modeTimer -= dt;
      if (this.modeTimer <= 0 && this.phase < SCHEDULE.length - 1) {
        this.phase += 1;
        this.modeTimer = SCHEDULE[this.phase][0];
        for (const g of this.ghosts) {
          // Moduswechsel -> Geister drehen um
          if (g.state === "maze" && !g.frightened) this.reverse(g);
        }
      }
    }

    releaseGhosts() {
      for (const g of this.ghosts) {
        if (g.state === "house" && this.playTimer >= g.releaseAt) {
          g.state = "leaving";
          g.tile = g.home.slice();
          g.frac = 0.0;
          g.dir = UP;
        }
      }
    }

    // ----- Bewegung ------------------------------------------------------
    /** Kehrt die Bewegungsrichtung um (auch mitten in einer Kachel). */
    reverse(e) {
      if (e.frac > 1e-6) {
        e.tile = this.wrapt(add(e.tile, e.dir));
        e.frac = 1.0 - e.frac;
      }
      e.dir = opp(e.dir);
    }

    pacSpeed() {
      return SPD_PAC * this.levelFactor();
    }

    movePac(dt) {
      const p = this.pac;
      let dist = this.pacSpeed() * dt;
      while (dist > 1e-9) {
        if (p.frac <= 1e-9) {
          if (p.want && this.pacOpen(add(p.tile, p.want))) p.dir = p.want;
          if (!this.pacOpen(add(p.tile, p.dir))) {
            p.moving = false;
            break;
          }
          p.moving = true;
        }
        const step = Math.min(dist, 1.0 - p.frac);
        p.frac += step;
        dist -= step;
        if (p.frac >= 1.0 - 1e-9) {
          p.tile = this.wrapt(add(p.tile, p.dir));
          p.frac = 0.0;
          this.pacEat(p.tile);
        }
      }
    }

    pacEat(tile) {
      const [c, r] = tile;
      const v = this.pellets[r][c];
      if (v === 1) {
        this.pellets[r][c] = 0;
        this.score += 10;
        this.dotsEaten += 1;
        if (this.dotsEaten % 2 === 0) this.playSound("eat");
      } else if (v === 2) {
        this.pellets[r][c] = 0;
        this.score += 50;
        this.dotsEaten += 1;
        this.frighten();
        this.playSound("powerup");
      }
      if (this.fruit && (eq(tile, this.fruit.a) || eq(tile, this.fruit.b))) this.eatFruit();
    }

    animPac(dt, forceMove) {
      if (forceMove) this.pac.mouth = Math.abs(Math.sin(this.animT * 11.0)) * 0.92;
      else this.pac.mouth = 0.18;
    }

    ghostSpeed(g) {
      if (g.state === "eyes" || g.state === "entering") return SPD_EYES;
      if (g.state === "house" || g.state === "leaving") return SPD_HOUSE;
      if (g.frightened) return SPD_FRIGHT;
      const [c, r] = g.tile;
      if (this.grid[r][PG.mod(c, COLS)] === "t") return SPD_TUNNEL * this.levelFactor();
      return SPD_GHOST * this.levelFactor() * DIFFS[this.diff].gspeed;
    }

    moveGhost(g, dt) {
      let dist = this.ghostSpeed(g) * dt;
      while (dist > 1e-9) {
        if (g.frac <= 1e-9) this.ghostDecide(g);
        const step = Math.min(dist, 1.0 - g.frac);
        g.frac += step;
        dist -= step;
        if (g.frac >= 1.0 - 1e-9) {
          g.tile = this.wrapt(add(g.tile, g.dir));
          g.frac = 0.0;
          this.ghostArrive(g);
        }
      }
    }

    ghostDecide(g) {
      const back = opp(g.dir);
      if (g.frightened && g.state === "maze") {
        const opts = ORDER.filter((d) => !eq(d, back) && this.ghostOpen(g, add(g.tile, d)));
        g.dir = opts.length ? PG.rand.choice(opts) : back;
        return;
      }
      const target = this.ghostTarget(g);
      let best = null, bd = 1e18;
      for (const d of ORDER) {
        if (eq(d, back)) continue;
        const nt = add(g.tile, d);
        if (!this.ghostOpen(g, nt)) continue;
        const dd = dist2(this.wrapt(nt), target);
        if (dd < bd) {
          bd = dd;
          best = d;
        }
      }
      g.dir = best !== null ? best : back;
    }

    ghostTarget(g) {
      if (g.state === "leaving" || g.state === "eyes") return DOOR_EXIT;
      if (g.state === "entering") return g.home;
      if (this.globalMode === "scatter") return g.scatter;
      return this.chaseTarget(g);
    }

    chaseTarget(g) {
      const pac = this.pac;
      if (g.name === "blinky") return pac.tile;
      if (g.name === "pinky") return add(pac.tile, [pac.dir[0] * 4, pac.dir[1] * 4]);
      if (g.name === "inky") {
        const p2 = add(pac.tile, [pac.dir[0] * 2, pac.dir[1] * 2]);
        const bl = this.ghosts[0].tile;
        return [2 * p2[0] - bl[0], 2 * p2[1] - bl[1]];
      }
      // clyde: aus der Ferne jagen, aus der Nähe ausweichen
      if (dist2(g.tile, pac.tile) >= 64) return pac.tile;
      return g.scatter;
    }

    ghostArrive(g) {
      const atDoor = g.tile[1] === 11 && (g.tile[0] === 13 || g.tile[0] === 14);
      if (g.state === "leaving" && atDoor) {
        g.state = "maze";
        g.frightened = this.frightTimer > 0;
      } else if (g.state === "eyes" && atDoor) {
        g.state = "entering";
      } else if (g.state === "entering" && eq(g.tile, g.home)) {
        g.state = "house";
        g.dir = UP;
        g.releaseAt = this.playTimer + 1.0;
      }
    }

    frighten() {
      this.frightTimer = this.frightTime();
      this.ghostCombo = 0;
      for (const g of this.ghosts) {
        if (g.state === "maze") {
          g.frightened = true;
          this.reverse(g);
        }
      }
    }

    // ----- Kollisionen / Tod --------------------------------------------
    pix(e, bob = 0.0) {
      const [c, r] = e.tile;
      const x = this.ox + (c + 0.5 + e.dir[0] * e.frac) * this.CELL;
      const y = this.oy + (r + 0.5 + e.dir[1] * e.frac + bob) * this.CELL;
      return [x, y];
    }

    collisions() {
      const [px, py] = this.pix(this.pac);
      for (const g of this.ghosts) {
        if (g.state !== "maze") continue;
        const [gx, gy] = this.pix(g);
        if (Math.hypot(px - gx, py - gy) < 0.55 * this.CELL) {
          if (g.frightened) this.eatGhost(g, gx, gy);
          else {
            this.pacDie();
            return;
          }
        }
      }
    }

    eatGhost(g, gx, gy) {
      const pts = 200 * 2 ** this.ghostCombo;
      this.ghostCombo = Math.min(this.ghostCombo + 1, 3);
      this.score += pts;
      g.state = "eyes";
      g.frightened = false;
      this.popup(gx, gy, String(pts), [120, 230, 255]);
      this.freeze = 0.5;
      this.playSound("point");
      this.rumble(80);
    }

    pacDie() {
      this.state = DYING;
      this.dyingTimer = 1.7;
      this.playSound("explode");
      this.rumble(300);
    }

    afterDeath() {
      this.lives -= 1;
      if (this.lives <= 0) {
        this.state = GAMEOVER;
        this.gameOver = true;
        this.hs = Math.max(this.hs, this.score);
        this.playSound("gameover");
      } else {
        this.resetPositions(1.8);
      }
    }

    checkExtraLife() {
      if (!this.extraAwarded && this.score >= 10000) {
        this.extraAwarded = true;
        this.lives += 1;
        this.playSound("level");
      }
    }

    // ----- Früchte -------------------------------------------------------
    updateFruit(dt) {
      if (this.fruit !== null) {
        this.fruit.t -= dt;
        if (this.fruit.t <= 0) this.fruit = null;
      } else if (this.fruitThresholds.length && this.dotsEaten >= this.fruitThresholds[0]) {
        this.fruitThresholds.shift();
        const [name, pts, col] = FRUIT_TABLE[Math.min(this.level - 1, FRUIT_TABLE.length - 1)];
        this.fruit = { name, pts, col, t: 9.5, a: [13, 17], b: [14, 17] };
      }
    }

    eatFruit() {
      const f = this.fruit;
      this.score += f.pts;
      const gx = this.ox + 13.5 * this.CELL;
      const gy = this.oy + 17.5 * this.CELL;
      this.popup(gx, gy, String(f.pts), f.col);
      this.fruitHistory.push(f.col);
      this.fruit = null;
      this.playSound("point");
    }

    // ----- Popups --------------------------------------------------------
    popup(x, y, text, col) {
      this.popups.push({ x, y, text, col, t: 1.4, t0: 1.4 });
    }

    updatePopups(dt) {
      for (const p of this.popups) {
        p.t -= dt;
        p.y -= dt * 12;
      }
      this.popups = this.popups.filter((p) => p.t > 0);
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      draw.rect(ctx, COL_BG, [0, 0, this.width, this.height]);
      const flash = this.state === LEVELCLEAR && Math.floor(this.animT * 6) % 2 === 0;
      this.drawMaze(ctx, flash);
      if (!flash) this.drawPellets(ctx);
      this.drawFruit(ctx);
      if (this.state !== DYING) for (const g of this.ghosts) this.drawGhost(ctx, g);
      this.drawPac(ctx);
      for (const p of this.popups) {
        const a = Math.max(0, Math.min(1, p.t / p.t0));
        ui.text(ctx, p.text, Math.floor(p.x), Math.floor(p.y), this.fTiny, p.col, "center", a);
      }
      this.drawHud(ctx);
      this.drawOverlays(ctx);
    }

    // ----- Labyrinth -----------------------------------------------------
    isWall(col, row) {
      if (row < 0 || row >= ROWS || col < 0 || col >= COLS) return true;
      return this.grid[row][col] === "#";
    }

    /** Labyrinth ist statisch -> einmal je Farbe in eine Offscreen-Canvas zeichnen. */
    drawMaze(ctx, flash) {
      const ps = Math.max(1, (PG.app && PG.app.pixelScale) || 1);
      const key = (flash ? "hi" : "lo") + "@" + ps.toFixed(2);
      let cv = this.mazeCache[key];
      if (!cv) {
        if (Object.keys(this.mazeCache).length > 4) this.mazeCache = {};
        cv = ui.makeCanvas(this.width * ps, this.height * ps);
        const g = cv.getContext("2d");
        g.scale(ps, ps);
        this.renderMaze(g, flash);
        this.mazeCache[key] = cv;
      }
      ctx.drawImage(cv, 0, 0, this.width, this.height);
    }

    /** Neon-Umriss-Stil: Linien entlang der Korridorränder (wie im Original). */
    renderMaze(s, flash) {
      const c = this.CELL;
      const colLine = flash ? COL_WALL_HI : COL_WALL;
      const w = Math.max(2, Math.floor(c * 0.16));
      const m = c * 0.28; // Abstand der Linie zur Kachelkante
      for (let r = 0; r < ROWS; r++) {
        for (let cc = 0; cc < COLS; cc++) {
          const rx = this.ox + cc * c, ry = this.oy + r * c;
          if (this.grid[r][cc] !== "#") {
            if (this.grid[r][cc] === "=") draw.rect(s, COL_DOOR, [rx, ry + Math.floor(c / 2) - 2, c, 4]);
            continue;
          }
          const top = !this.isWall(cc, r - 1);
          const bot = !this.isWall(cc, r + 1);
          const lft = !this.isWall(cc - 1, r);
          const rgt = !this.isWall(cc + 1, r);
          if (top) draw.line(s, colLine, [rx, ry + m], [rx + c, ry + m], w);
          if (bot) draw.line(s, colLine, [rx, ry + c - m], [rx + c, ry + c - m], w);
          if (lft) draw.line(s, colLine, [rx + m, ry], [rx + m, ry + c], w);
          if (rgt) draw.line(s, colLine, [rx + c - m, ry], [rx + c - m, ry + c], w);
          // Innenecken abrunden: kleine Verbindungspunkte an konvexen Ecken
          const hr = Math.floor(w / 2);
          if (top && lft) draw.circle(s, colLine, [rx + m, ry + m], hr);
          if (top && rgt) draw.circle(s, colLine, [rx + c - m, ry + m], hr);
          if (bot && lft) draw.circle(s, colLine, [rx + m, ry + c - m], hr);
          if (bot && rgt) draw.circle(s, colLine, [rx + c - m, ry + c - m], hr);
        }
      }
    }

    drawPellets(s) {
      const c = this.CELL;
      const pr = Math.max(1, Math.floor(c * 0.12));
      const powerOn = Math.floor(this.animT * 6) % 2 === 0;
      const half = Math.floor(c / 2);
      s.fillStyle = ui.col(COL_PILL);
      s.beginPath();
      for (let r = 0; r < ROWS; r++) {
        for (let col = 0; col < COLS; col++) {
          if (this.pellets[r][col] === 1) {
            const cx = this.ox + col * c + half, cy = this.oy + r * c + half;
            s.moveTo(cx + pr, cy);
            s.arc(cx, cy, pr, 0, PG.TAU);
          }
        }
      }
      s.fill();
      if (!powerOn) return;
      for (let r = 0; r < ROWS; r++) {
        for (let col = 0; col < COLS; col++) {
          if (this.pellets[r][col] === 2) {
            draw.circle(s, COL_POWER, [this.ox + col * c + half, this.oy + r * c + half], Math.max(3, Math.floor(c * 0.34)));
          }
        }
      }
    }

    drawFruit(s) {
      if (this.fruit === null) return;
      const cx = this.ox + 13.5 * this.CELL;
      const cy = this.oy + 17.5 * this.CELL;
      this.fruitIcon(s, cx, cy, this.CELL * 0.7, this.fruit.col);
    }

    fruitIcon(s, cx, cy, r, col) {
      draw.circle(s, col, [cx, cy + r * 0.15], Math.floor(r * 0.6));
      draw.line(s, [90, 200, 90], [cx, cy - r * 0.4], [cx + r * 0.35, cy - r * 0.7], 2);
      draw.circle(s, [255, 255, 255], [cx - r * 0.2, cy], Math.max(1, Math.floor(r * 0.12)));
    }

    // ----- Pac-Man -------------------------------------------------------
    drawPac(s) {
      const r = this.CELL * 0.46;
      const [px, py] = this.pix(this.pac);
      let mouth;
      if (this.state === DYING) {
        const prog = 1.0 - this.dyingTimer / 1.7;
        mouth = 0.15 + prog * (Math.PI - 0.15);
        if (prog >= 0.98) return;
      } else {
        mouth = this.pac.mouth;
      }
      const ang = Math.atan2(this.pac.dir[1], this.pac.dir[0]);
      draw.circle(s, COL_PAC, [px, py], r);
      if (mouth > 0.03) {
        const pts = [[px, py]];
        const steps = 12;
        for (let i = 0; i <= steps; i++) {
          const a = ang - mouth + (2 * mouth * i) / steps;
          pts.push([px + Math.cos(a) * r * 1.25, py + Math.sin(a) * r * 1.25]);
        }
        draw.polygon(s, COL_BG, pts);
      }
    }

    // ----- Geister -------------------------------------------------------
    drawGhost(s, g) {
      const r = this.CELL * 0.46;
      let bob = 0.0;
      if (g.state === "house") bob = 0.12 * Math.sin(g.bobT);
      const [px, py] = this.pix(g, bob);
      const eyesOnly = g.state === "eyes" || g.state === "entering";

      if (!eyesOnly) {
        let col = g.color;
        if (g.frightened) {
          const ending = this.frightTimer < Math.min(2.2, this.frightTime() * 0.4);
          col = ending && Math.floor(this.animT * 8) % 2 === 0 ? COL_FRIGHT_END : COL_FRIGHT;
        }
        this.ghostBody(s, px, py, r, col);
      }

      if (g.frightened && !eyesOnly) this.frightFace(s, px, py, r);
      else this.ghostEyes(s, px, py, r, g.dir);
    }

    ghostBody(s, cx, cy, r, col) {
      draw.circle(s, col, [cx, cy - r * 0.12], r);
      draw.rect(s, col, [cx - r, cy - r * 0.12, 2 * r, r * 1.05]);
      const footY = cy - r * 0.12 + r * 1.05;
      const n = 4;
      const fw = (2 * r) / n;
      for (let i = 0; i < n; i++) {
        const x0 = cx - r + i * fw;
        draw.polygon(s, COL_BG, [[x0, footY + 0.5], [x0 + fw / 2, footY - r * 0.42], [x0 + fw, footY + 0.5]]);
      }
    }

    ghostEyes(s, cx, cy, r, d) {
      for (const sx of [-1, 1]) {
        const ex = cx + sx * r * 0.42;
        const ey = cy - r * 0.18;
        draw.circle(s, [255, 255, 255], [ex, ey], r * 0.34);
        draw.circle(s, [40, 50, 190], [ex + d[0] * r * 0.16, ey + d[1] * r * 0.16], r * 0.17);
      }
    }

    frightFace(s, cx, cy, r) {
      for (const sx of [-1, 1]) {
        draw.circle(s, [255, 220, 230], [cx + sx * r * 0.38, cy - r * 0.12], Math.max(1, r * 0.14));
      }
      const y = cy + r * 0.4;
      const pts = [];
      for (let i = 0; i < 7; i++) {
        const x = cx - r * 0.6 + (r * 1.2 * i) / 6;
        pts.push([x, y + (i % 2 ? r * 0.14 : -r * 0.14)]);
      }
      draw.lines(s, [255, 220, 230], false, pts, 2);
    }

    // ----- HUD / Overlays -----------------------------------------------
    /** Halbtransparentes Themen-Panel mit Akzent-Rahmen. */
    blitPanel(s, rect, border = 2) {
      const c = ui.PANEL;
      draw.rect(s, [c[0], c[1], c[2], 222], rect, 0, 10);
      draw.rect(s, this.accent, rect, border, 10);
    }

    drawHud(s) {
      this.hs = Math.max(this.hs, this.score);
      const labH = this.fTiny.height;
      const scoreH = this.fDigit.height;

      // Themen-Leiste über dem Labyrinth, genau so hoch wie der Inhalt
      const bar = new PG.Rect(6, 2, this.width - 12, labH + scoreH + 8);
      this.blitPanel(s, bar, 1);
      ui.text(s, t("pac.1up"), 14, 5, this.fTiny, this.accent);
      ui.text(s, String(this.score), 14, 5 + labH, this.fDigit, ui.TEXT);
      const cx = Math.floor(this.width / 2);
      ui.text(s, t("pac.high"), cx, 5, this.fTiny, this.accent, "midtop");
      ui.text(s, String(this.hs), cx, 5 + labH, this.fDigit, ui.TEXT, "midtop");
      ui.text(s, t("pac.level", { n: this.level }), this.width - 14, 7, this.fSmall, ui.TEXT_DIM, "topright");

      // Leben (unten links) + gesammelte Früchte (unten rechts)
      const y = this.height - Math.floor(this.CELL * 1.2);
      for (let i = 0; i < Math.max(0, this.lives - 1); i++) {
        this.lifeIcon(s, 16 + i * Math.floor(this.CELL * 1.4), y, this.CELL * 0.5);
      }
      this.fruitHistory.slice(-6).forEach((col, i) => {
        this.fruitIcon(s, this.width - 16 - i * Math.floor(this.CELL * 1.3), y, this.CELL * 0.6, col);
      });
    }

    lifeIcon(s, x, y, r) {
      draw.circle(s, COL_PAC, [x, y], r);
      const pts = [[x, y]];
      for (let i = 0; i < 7; i++) {
        const a = Math.PI - 0.5 + (1.0 * i) / 6;
        pts.push([x + Math.cos(a) * r * 1.3, y + Math.sin(a) * r * 1.3]);
      }
      draw.polygon(s, COL_BG, pts);
    }

    drawOverlays(s) {
      const cx = this.ox + Math.floor(this.mazeW / 2);
      const cy = this.oy + Math.floor(17.2 * this.CELL);
      if (this.state === READY) {
        const label = t("pac.ready");
        const tw = this.fFont.width(label), th = this.fFont.height;
        const box = new PG.Rect(0, 0, tw, th);
        box.center = [cx, cy];
        this.blitPanel(s, box.inflate(Math.floor(this.CELL * 1.8), Math.floor(this.CELL * 0.9)));
        ui.text(s, label, cx, cy, this.fFont, this.accent, "center");
      } else if (this.state === GAMEOVER) {
        const title = t("common.game_over");
        const hint = t("pac.restart_hint");
        const pad = Math.floor(this.CELL * 0.8);
        const bw = Math.max(this.fBig.width(title), this.fSmall.width(hint)) + 2 * pad + this.CELL;
        const top = cy - Math.floor(this.fBig.height / 2) - pad;
        const bottom = cy + Math.floor(this.CELL * 2.2) + Math.floor(this.fSmall.height / 2) + pad;
        const box = new PG.Rect(0, 0, bw, bottom - top);
        box.midtop = [cx, top];
        this.blitPanel(s, box);
        ui.text(s, title, cx, cy, this.fBig, ui.RED, "center");
        ui.text(s, hint, cx, cy + Math.floor(this.CELL * 2.2), this.fSmall, ui.TEXT, "center");
      }
    }

    // ----- Setup zeichnen -----------------------------------------------
    /** Setup-Screen im Theme-Stil (Palette wird zur Zeichenzeit gelesen). */
    drawSetup(s) {
      ui.drawBackground(s, this.width, this.height);
      ui.drawTitle(s, this.width, "PAC-MAN", { subtitle: t("snake.singleplayer"), accent: this.accent });

      const d = DIFFS[this.diff];
      draw.rect(s, ui.PANEL, this.diffPanel, 0, 10);
      draw.rect(s, this.accent, this.diffPanel, 2, 10);
      ui.text(s, t("pac.difficulty") + ":  " + t("pac.diff." + d.key), this.diffPanel.centerx, this.diffPanel.top + 22, this.fFont, ui.TEXT, "center");
      ui.text(s, t("pac.diff_note"), this.diffPanel.centerx, this.diffPanel.top + 44, this.fTiny, ui.TEXT_DIM, "center");
      for (const [rect, sym] of [[this.diffLeft, "<"], [this.diffRight, ">"]]) {
        ui.text(s, sym, rect.centerx, rect.centery, this.fBig, this.accent, "center");
      }

      ui.drawButton(s, this.startRect, t("common.start"), this.fFont, true, { accent: ui.GREEN });

      ui.text(s, t("pac.setup_hint"), Math.floor(this.width / 2), this.height - 38, this.fSmall, ui.TEXT_DIM, "center");
      ui.text(s, t("pac.controls_hint"), Math.floor(this.width / 2), this.height - 16, this.fTiny, ui.GREEN, "center");
    }
  }

  PG.register(PacmanGame, {
    id: "PacmanGame",
    key: "pacman",
    name: "Pac-Man",
    settingsKey: "pacman",
    defaults: { difficulty: 0 },
  });
})();
