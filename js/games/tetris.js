/*
 * tetris.js - Tetris (Port von games/tetris.py)
 * ==============================================
 * - Steuerung: links/rechts verschieben, hoch = drehen, runter = Soft-Drop,
 *   Aktion (Leertaste/Enter) = Hard-Drop. WASD und Pfeile gehen beide.
 * - Volle Reihen lösen sich auf und geben Punkte; alle 10 Reihen steigt das Level.
 * - Highscore wird gespeichert.
 * - Web-Version: nur Einzelspieler (der Versus-Modus entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const COLS = 10;
  const ROWS = 20;

  const SHAPES = {
    I: [[[0, 1], [1, 1], [2, 1], [3, 1]],
        [[2, 0], [2, 1], [2, 2], [2, 3]]],
    O: [[[1, 0], [2, 0], [1, 1], [2, 1]]],
    T: [[[1, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [2, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [1, 2]],
        [[1, 0], [0, 1], [1, 1], [1, 2]]],
    S: [[[1, 0], [2, 0], [0, 1], [1, 1]],
        [[1, 0], [1, 1], [2, 1], [2, 2]]],
    Z: [[[0, 0], [1, 0], [1, 1], [2, 1]],
        [[2, 0], [1, 1], [2, 1], [1, 2]]],
    J: [[[0, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [2, 0], [1, 1], [1, 2]],
        [[0, 1], [1, 1], [2, 1], [2, 2]],
        [[1, 0], [1, 1], [0, 2], [1, 2]]],
    L: [[[2, 0], [0, 1], [1, 1], [2, 1]],
        [[1, 0], [1, 1], [1, 2], [2, 2]],
        [[0, 1], [1, 1], [2, 1], [0, 2]],
        [[0, 0], [1, 0], [1, 1], [1, 2]]],
  };

  // Identitätsfarben der Steine - bewusst NICHT aus dem UI-Theme.
  const COLORS = {
    I: [80, 210, 220], O: [240, 220, 90], T: [190, 110, 220],
    S: [110, 220, 120], Z: [235, 100, 100], J: [100, 130, 230],
    L: [240, 160, 80],
  };

  const LINE_POINTS = { 1: 40, 2: 100, 3: 300, 4: 1200 };

  /** Ein einzelnes Tetris-Spielfeld mit eigenem Stein, Punkten und Level. */
  class Board {
    constructor(game) {
      this.game = game; // für Sound/Haptik-Rückrufe
      this.grid = Array.from({ length: ROWS }, () => new Array(COLS).fill(null));
      this.level = 1;
      this.lines = 0;
      this.score = 0;
      this.dead = false;
      this.fallTimer = 0;
      this.bag = [];
      this.nextKind = this.bagNext();
      this.spawn();
    }

    // ----- Steine ------------------------------------------------------------
    bagNext() {
      if (!this.bag.length) {
        this.bag = Object.keys(SHAPES);
        PG.rand.shuffle(this.bag);
      }
      return this.bag.pop();
    }

    spawn() {
      this.kind = this.nextKind;
      this.nextKind = this.bagNext();
      this.rot = 0;
      this.px = Math.floor(COLS / 2) - 2;
      this.py = this.kind === "I" ? -1 : 0;
      if (this.collision(this.px, this.py, this.rot)) this.dead = true;
    }

    cells(px, py, rot) {
      const form = SHAPES[this.kind];
      return form[PG.mod(rot, form.length)].map(([cx, cy]) => [px + cx, py + cy]);
    }

    collision(px, py, rot) {
      for (const [x, y] of this.cells(px, py, rot)) {
        if (x < 0 || x >= COLS || y >= ROWS) return true;
        if (y >= 0 && this.grid[y][x] !== null) return true;
      }
      return false;
    }

    curCells() {
      return this.cells(this.px, this.py, this.rot);
    }

    ghostCells() {
      let gy = this.py;
      while (!this.collision(this.px, gy + 1, this.rot)) gy++;
      return this.cells(this.px, gy, this.rot);
    }

    // ----- Eingaben ----------------------------------------------------------
    move(dx) {
      if (!this.dead && !this.collision(this.px + dx, this.py, this.rot)) {
        this.px += dx;
        this.game.playSound("move");
      }
    }

    rotate() {
      if (this.dead) return;
      const neu = this.rot + 1;
      for (const dx of [0, -1, 1, -2, 2]) {
        if (!this.collision(this.px + dx, this.py, neu)) {
          this.px += dx;
          this.rot = neu;
          this.game.playSound("rotate");
          return;
        }
      }
    }

    soft() {
      if (this.dead) return;
      if (!this.collision(this.px, this.py + 1, this.rot)) {
        this.py += 1;
        this.score += 1;
      } else {
        this.lock();
      }
    }

    hard() {
      if (this.dead) return;
      while (!this.collision(this.px, this.py + 1, this.rot)) {
        this.py += 1;
        this.score += 2;
      }
      this.lock();
    }

    // ----- Logik -------------------------------------------------------------
    update(dt) {
      if (this.dead) return;
      const interval = Math.max(0.05, 0.55 - (this.level - 1) * 0.045);
      this.fallTimer += dt;
      if (this.fallTimer < interval) return;
      this.fallTimer = 0;
      if (!this.collision(this.px, this.py + 1, this.rot)) this.py += 1;
      else this.lock();
    }

    lock() {
      const farbe = COLORS[this.kind];
      for (const [x, y] of this.curCells()) {
        if (y >= 0) this.grid[y][x] = farbe;
      }
      this.game.playSound("lock");
      this.clearLines();
      this.spawn();
    }

    clearLines() {
      const behalten = this.grid.filter((row) => row.some((c) => c === null));
      const entfernt = ROWS - behalten.length;
      if (entfernt) {
        const leer = Array.from({ length: entfernt }, () => new Array(COLS).fill(null));
        this.grid = leer.concat(behalten);
        this.lines += entfernt;
        const punkte = LINE_POINTS[entfernt] || 0;
        this.score += punkte * this.level;
        this.level = 1 + Math.floor(this.lines / 10);
        if (entfernt === 4) this.game.achEvent("tetris_four");
        this.game.playSound("line");
        this.game.rumble(120);
      }
    }
  }

  class TetrisGame extends PG.Game {
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.boardCache = null;
      this.boards = [new Board(this)];
      this.layout();
    }

    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, Math.min(26, Math.floor(h / 26))));
      this.bigFont = ui.font(Math.max(30, Math.min(52, Math.floor(h / 12))), true);
      this.small = ui.font(Math.max(13, Math.min(18, Math.floor(h / 36))));
    }

    /** Zellgröße und Feld-Position aus width/height berechnen. */
    layout() {
      this.makeFonts();
      this.cell = Math.min(Math.floor((this.height - 40) / ROWS), Math.floor((this.width - 200) / COLS));
      const oy = Math.floor((this.height - this.cell * ROWS) / 2);
      this.layoutPos = [[24, oy]];
      this.boardCache = null;
    }

    // ----- Eingabe -------------------------------------------------------------
    handleEvent(ev) {
      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.reset();
        return;
      }
      this.input(this.boards[0], ev.key);
    }

    input(board, key) {
      if (this.isAction(key, "left")) board.move(-1);
      else if (this.isAction(key, "right")) board.move(1);
      else if (this.isAction(key, "up")) board.rotate();
      else if (this.isAction(key, "down")) board.soft();
      else if (this.isAction(key, "action")) board.hard();
    }

    // ----- Logik ---------------------------------------------------------------
    update(dt) {
      if (this.gameOver) return;
      for (const b of this.boards) b.update(dt);

      this.score = Math.max(...this.boards.map((b) => b.score));

      if (this.boards.some((b) => b.dead)) {
        this.gameOver = true;
        this.playSound("gameover");
        this.rumble(200);
      }
    }

    // ----- Theme-UI-Helfer -------------------------------------------------------
    /** Gecachter Feld-Hintergrund: dunkle Panel-Fläche + Gitterlinien. */
    boardBg() {
      const bw = this.cell * COLS, bh = this.cell * ROWS;
      const ps = Math.max(1, (PG.app && PG.app.pixelScale) || 1);
      const key = [bw, bh, ps, ui.PANEL.join(","), ui.BORDER.join(",")].join("|");
      if (!this.boardCache || this.boardCache.key !== key) {
        const c = ui.makeCanvas((bw + 1) * ps, (bh + 1) * ps);
        const g = c.getContext("2d");
        g.scale(ps, ps);
        g.fillStyle = ui.col(ui.PANEL.map((v) => Math.floor(v * 0.72)));
        g.fillRect(0, 0, bw + 1, bh + 1);
        for (let cx = 0; cx <= COLS; cx++) {
          const x = cx * this.cell;
          draw.line(g, ui.PANEL, [x + 0.5, 0], [x + 0.5, bh + 1]);
        }
        for (let cy = 0; cy <= ROWS; cy++) {
          const y = cy * this.cell;
          draw.line(g, ui.PANEL, [0, y + 0.5], [bw + 1, y + 0.5]);
        }
        this.boardCache = { key, canvas: c, w: bw + 1, h: bh + 1 };
      }
      return this.boardCache;
    }

    // ----- Zeichnen ------------------------------------------------------------
    drawCell(ctx, ox, oy, cx, cy, farbe) {
      draw.rect(ctx, farbe, [ox + cx * this.cell + 1, oy + cy * this.cell + 1, this.cell - 2, this.cell - 2], 0, 3);
    }

    drawBoard(ctx, b, ox, oy) {
      const bw = this.cell * COLS;
      const bh = this.cell * ROWS;

      const bg = this.boardBg();
      ctx.drawImage(bg.canvas, ox, oy, bg.w, bg.h);

      for (let cy = 0; cy < ROWS; cy++) {
        for (let cx = 0; cx < COLS; cx++) {
          if (b.grid[cy][cx] !== null) this.drawCell(ctx, ox, oy, cx, cy, b.grid[cy][cx]);
        }
      }

      if (!b.dead) {
        // Geist in der Stein-Farbe, stark Richtung Feld-Hintergrund gemischt
        const ghost = COLORS[b.kind].map((c, i) => Math.floor(c * 0.3 + ui.PANEL[i] * 0.7));
        for (const [x, y] of b.ghostCells()) if (y >= 0) this.drawCell(ctx, ox, oy, x, y, ghost);
        const farbe = COLORS[b.kind];
        for (const [x, y] of b.curCells()) if (y >= 0) this.drawCell(ctx, ox, oy, x, y, farbe);
      }

      draw.rect(ctx, ui.BORDER_LIGHT, [ox, oy, bw, bh], 2);
    }

    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);

      this.boards.forEach((b, i) => {
        const [ox, oy] = this.layoutPos[i];
        this.drawBoard(ctx, b, ox, oy);
        this.drawInfo(ctx, b, ox + this.cell * COLS + 24, oy);
      });

      if (this.gameOver) {
        ctx.fillStyle = "rgba(8,10,16,0.588)";
        ctx.fillRect(0, 0, this.width, this.height);
        this.drawCenterText(ctx, t("common.game_over"), this.bigFont, ui.RED, -20);
        const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
        this.drawCenterText(ctx, t("common.enter_restart"), this.font, hintCol, 30);
      }
    }

    /** Seitenleiste mit Punkten, Level, Reihen und Vorschau. */
    drawInfo(ctx, b, infoX, oy) {
      const lh = this.font.height + 6;
      const prevH = this.cell * 2;
      const contentH = 14 + lh + this.bigFont.height + 16 + 2 * lh + 14 + lh + prevH + 18;
      const pw = Math.max(120, this.width - infoX + 2);
      const panel = new PG.Rect(infoX - 14, oy, Math.min(pw, this.width - infoX + 2), contentH);

      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 225], panel, 0, 12);
      draw.rect(ctx, ui.BORDER, panel, 1, 12);

      const x = infoX;
      let y = oy + 14;
      ui.text(ctx, t("tetris.points"), x, y, this.font, ui.TEXT_DIM);
      y += lh;
      ui.text(ctx, String(b.score), x, y, this.bigFont, this.accent);
      y += this.bigFont.height + 16;
      ui.text(ctx, t("tetris.level", { level: b.level }), x, y, this.font, ui.TEXT);
      y += lh;
      ui.text(ctx, t("tetris.rows", { lines: b.lines }), x, y, this.font, ui.TEXT);
      y += lh + 14;
      ui.text(ctx, t("tetris.next"), x, y, this.font, ui.TEXT_DIM);
      y += lh;
      for (const [cx, cy] of SHAPES[b.nextKind][0]) {
        draw.rect(ctx, COLORS[b.nextKind], [x + cx * this.cell + 1, y + cy * this.cell + 1, this.cell - 2, this.cell - 2], 0, 3);
      }
    }
  }

  PG.register(TetrisGame, {
    id: "TetrisGame",
    key: "tetris",
    name: "Tetris",
  });
})();
