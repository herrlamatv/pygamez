/*
 * bubbleshooter.js - Bubble Shooter (Port von games/bubbleshooter.py)
 * ====================================================================
 * Bubble Shooter (Puzzle Bobble) - schieße Kugeln nach oben und bilde Gruppen aus
 * mindestens drei gleichen Farben, damit sie platzen.
 *
 * - Das Feld ist ein versetztes Wabenraster (gerade Reihen bündig links, ungerade
 *   um einen halben Kugelradius nach rechts versetzt). Jede Kugel hat sechs
 *   Nachbarn.
 * - Die Kanone unten zielt zur Maus (oder mit Pfeil links/rechts); Klick bzw.
 *   Leertaste schießt. Die Kugel prallt an den Seitenwänden ab und rastet beim
 *   Treffer in die nächste freie Wabe ein.
 * - Nach dem Einrasten platzt eine gleichfarbige Gruppe ab drei Kugeln; danach
 *   fallen alle Kugeln, die dadurch den Halt zur Decke verlieren (Bonuspunkte).
 * - Drei Modi: Leicht (4 Farben, keine neuen Reihen), Klassik (5 Farben, ab und zu
 *   rückt oben eine Reihe nach) und Schwer (6 Farben, schneller).
 *
 * Game Over, sobald eine Kugel zu tief einrastet. Punkte zählt die App als Highscore.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const BUBBLE_COLORS = [
    [230, 72, 72], [240, 205, 70], [96, 200, 112],
    [80, 140, 240], [175, 110, 235], [240, 150, 52],
  ];

  // (Farbanzahl, Startreihen, Schuss-Abstand zwischen neuen Reihen; 0 = nie).
  const MODE_CFG = { easy: [4, 4, 0], classic: [5, 5, 8], hard: [6, 6, 6] };

  const COLS = 12;
  const SPEED = 820.0;

  const PLAY = "play", OVER = "over";

  class BubbleShooterGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      [this.ncolors, this.startRows, this.dropEvery] = MODE_CFG[this.mode] || MODE_CFG.classic;
      this.score = 0;
      this.gameOver = false;
      this.state = PLAY;
      this.makeFonts();
      this.layout();
      this.newBoard();
    }

    makeFonts() {
      this.hud = ui.font(20, true);
      this.small = ui.font(15);
      this.huge = ui.font(Math.max(30, Math.floor(this.height / 11)), true);
    }

    layout() {
      const margin = 14;
      this.top = 54;
      const playW = this.width - 2 * margin;
      this.r = Math.min(28, playW / (2 * COLS + 1));
      this.rowH = this.r * Math.sqrt(3);
      const boardW = 2 * this.r * COLS + this.r;
      this.bx = (this.width - boardW) / 2.0;
      this.left = this.bx;
      this.right = this.bx + boardW;
      this.cannon = [this.width / 2.0, this.height - 30.0];
      this.deathY = this.cannon[1] - 2 * this.r - 26;
    }

    newBoard() {
      this.grid = []; // Liste von Reihen; je Reihe COLS Zellen
      this.dropOffset = 0;
      for (let i = 0; i < this.startRows; i++) this.grid.push(this.randomRow());
      this.shots = 0;
      this.shot = null; // fliegende Kugel oder null
      this.aim = -Math.PI / 2; // zeigt nach oben
      this.cur = this.pickColor();
      this.nxt = this.pickColor();
      this.state = PLAY;
    }

    randomRow() {
      const row = [];
      for (let c = 0; c < COLS; c++) row.push(PG.rand.randrange(this.ncolors));
      return row;
    }

    pickColor() {
      const present = new Set();
      for (const row of this.grid) for (const c of row) if (c !== null) present.add(c);
      if (present.size) return PG.rand.choice([...present]);
      return PG.rand.randrange(this.ncolors);
    }

    // ===================================================== Rasterhelfer
    parity(row) {
      return PG.mod(row + this.dropOffset, 2);
    }

    center(row, col) {
      const x = this.bx + this.r + 2 * this.r * col + (this.parity(row) ? this.r : 0);
      const y = this.top + this.r + row * this.rowH;
      return [x, y];
    }

    filled(row, col) {
      return row >= 0 && row < this.grid.length && col >= 0 && col < COLS && this.grid[row][col] !== null;
    }

    neighbors(row, col) {
      const offs = this.parity(row) === 0
        ? [[0, -1], [0, 1], [-1, -1], [-1, 0], [1, -1], [1, 0]]
        : [[0, -1], [0, 1], [-1, 0], [-1, 1], [1, 0], [1, 1]];
      const out = [];
      for (const [dr, dc] of offs) {
        const rr = row + dr, cc = col + dc;
        if (rr >= 0 && cc >= 0 && cc < COLS) out.push([rr, cc]);
      }
      return out;
    }

    cellFromPoint(x, y) {
      let row = Math.round((y - this.top - this.r) / this.rowH);
      row = Math.max(0, row);
      const p = PG.mod(row + this.dropOffset, 2);
      let col = Math.round((x - this.bx - this.r - (p ? this.r : 0)) / (2 * this.r));
      col = Math.max(0, Math.min(COLS - 1, col));
      return [row, col];
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === OVER) {
        if (this.isContinue(ev)) {
          this.gameOver = false;
          this.reset();
        }
        return;
      }
      if (ev.kind === "mousemove") {
        this.aimAt(ev.pos);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.aimAt(ev.pos);
        this.shoot();
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "space" || k === "Up" || k === "w" || k === "W") {
          this.shoot();
        } else if (k === "Left" || k === "a" || k === "A") {
          this.aim = Math.max(-Math.PI + 0.28, this.aim - 0.08);
        } else if (k === "Right" || k === "d" || k === "D") {
          this.aim = Math.min(-0.28, this.aim + 0.08);
        }
      }
    }

    isContinue(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    aimAt(pos) {
      const dx = pos[0] - this.cannon[0];
      let dy = pos[1] - this.cannon[1];
      if (dy > -1) dy = -1;
      const ang = Math.atan2(dy, dx);
      this.aim = Math.max(-Math.PI + 0.28, Math.min(-0.28, ang));
    }

    shoot() {
      if (this.shot !== null || this.state !== PLAY) return;
      this.shot = {
        x: this.cannon[0], y: this.cannon[1],
        vx: Math.cos(this.aim) * SPEED, vy: Math.sin(this.aim) * SPEED, c: this.cur,
      };
      this.playSound("move");
    }

    // ===================================================== Update
    update(dt) {
      if (this.state !== PLAY || this.shot === null) return;
      dt = Math.min(dt, 0.05);
      // Substeps gegen Durchtunneln
      const steps = Math.max(1, Math.floor((SPEED * dt) / (this.r * 0.5)) + 1);
      const sdt = dt / steps;
      for (let i = 0; i < steps; i++) {
        if (this.advance(sdt)) break;
      }
    }

    hitsBubble(x, y) {
      const hit = 2 * this.r * 0.86;
      const h2 = hit * hit;
      for (let row = 0; row < this.grid.length; row++) {
        for (let col = 0; col < COLS; col++) {
          if (this.grid[row][col] === null) continue;
          const [cx, cy] = this.center(row, col);
          if ((x - cx) ** 2 + (y - cy) ** 2 <= h2) return true;
        }
      }
      return false;
    }

    advance(dt) {
      const sh = this.shot;
      sh.x += sh.vx * dt;
      sh.y += sh.vy * dt;
      // Wandreflexion
      if (sh.x <= this.left + this.r) {
        sh.x = this.left + this.r;
        sh.vx = Math.abs(sh.vx);
      } else if (sh.x >= this.right - this.r) {
        sh.x = this.right - this.r;
        sh.vx = -Math.abs(sh.vx);
      }
      // Decke
      if (sh.y <= this.top + this.r) {
        this.place(sh.x, sh.y, sh.c);
        return true;
      }
      // Kollision mit vorhandenen Kugeln
      if (this.hitsBubble(sh.x, sh.y)) {
        this.place(sh.x, sh.y, sh.c);
        return true;
      }
      return false;
    }

    place(x, y, color) {
      this.shot = null;
      this.shots += 1;
      const cell = this.snapCell(x, y);
      if (cell === null) return;
      const [row, col] = cell;
      while (this.grid.length <= row) this.grid.push(new Array(COLS).fill(null));
      this.grid[row][col] = color;
      this.playSound("click");

      const cluster = this.sameColorCluster(row, col);
      if (cluster.length >= 3) {
        for (const [r, c] of cluster) this.grid[r][c] = null;
        this.score += cluster.length * 10;
        this.playSound("win");
        const dropped = this.dropFloating();
        if (dropped) this.score += dropped * 20;
      }
      // Feld leer? Bonus + Nachfüllen
      if (!this.grid.some((rr) => rr.some((c) => c !== null))) {
        this.score += 500;
        this.playSound("win");
        this.newRowsTop(this.startRows);
      }

      // Neue Reihe von oben (Modus-abhängig)
      if (this.dropEvery && this.shots % this.dropEvery === 0) this.newRowsTop(1);

      this.cur = this.nxt;
      this.nxt = this.pickColor();
      this.checkOver();
    }

    snapCell(x, y) {
      const [r0, c0] = this.cellFromPoint(x, y);
      let best = null, bestd = 1e18;
      for (let rr = Math.max(0, r0 - 1); rr < r0 + 3; rr++) {
        for (let cc = 0; cc < COLS; cc++) {
          if (this.filled(rr, cc)) continue;
          const attached = rr === 0 || this.neighbors(rr, cc).some(([nr, nc]) => this.filled(nr, nc));
          if (!attached) continue;
          const [cx, cy] = this.center(rr, cc);
          const d = (x - cx) ** 2 + (y - cy) ** 2;
          if (d < bestd) {
            best = [rr, cc];
            bestd = d;
          }
        }
      }
      if (best === null) best = [r0, c0]; // Notfall: irgendeine freie Wabe
      return best;
    }

    sameColorCluster(row, col) {
      const color = this.grid[row][col];
      const seen = new Set([row * COLS + col]);
      const out = [[row, col]];
      const stack = [[row, col]];
      while (stack.length) {
        const [r, c] = stack.pop();
        for (const [nr, nc] of this.neighbors(r, c)) {
          const k = nr * COLS + nc;
          if (!seen.has(k) && this.filled(nr, nc) && this.grid[nr][nc] === color) {
            seen.add(k);
            out.push([nr, nc]);
            stack.push([nr, nc]);
          }
        }
      }
      return out;
    }

    /** Entfernt alle Kugeln ohne Verbindung zur obersten Reihe (Reihe 0). */
    dropFloating() {
      const anchored = new Set();
      const stack = [];
      for (let c = 0; c < COLS; c++) {
        if (this.filled(0, c)) {
          anchored.add(c);
          stack.push([0, c]);
        }
      }
      while (stack.length) {
        const [r, c] = stack.pop();
        for (const [nr, nc] of this.neighbors(r, c)) {
          const k = nr * COLS + nc;
          if (!anchored.has(k) && this.filled(nr, nc)) {
            anchored.add(k);
            stack.push([nr, nc]);
          }
        }
      }
      let dropped = 0;
      for (let r = 0; r < this.grid.length; r++) {
        for (let c = 0; c < COLS; c++) {
          if (this.filled(r, c) && !anchored.has(r * COLS + c)) {
            this.grid[r][c] = null;
            dropped += 1;
          }
        }
      }
      return dropped;
    }

    newRowsTop(n) {
      for (let i = 0; i < n; i++) {
        this.grid.unshift(this.randomRow());
        this.dropOffset += 1;
      }
    }

    checkOver() {
      for (let r = 0; r < this.grid.length; r++) {
        for (let c = 0; c < COLS; c++) {
          if (this.grid[r][c] !== null) {
            const cy = this.center(r, c)[1];
            if (cy + this.r >= this.deathY) {
              this.state = OVER;
              this.gameOver = true; // App speichert Highscore
              this.playSound("gameover");
              return;
            }
          }
        }
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawDeathLine(ctx);
      this.drawGrid(ctx);
      if (this.state === PLAY) this.drawAim(ctx);
      this.drawCannon(ctx);
      this.drawHud(ctx);
      if (this.state === OVER) this.drawBanner(ctx);
    }

    bubble(ctx, x, y, color, radius) {
      const rad = Math.floor(radius || this.r);
      x = Math.floor(x);
      y = Math.floor(y);
      draw.circle(ctx, color, [x, y], rad);
      draw.circle(ctx, ui.mix(color, [255, 255, 255], 0.35), [x, y], rad, 2);
      draw.circle(ctx, ui.mix(color, [255, 255, 255], 0.55), [x - Math.floor(rad / 3), y - Math.floor(rad / 3)], Math.max(2, Math.floor(rad / 4)));
    }

    drawGrid(ctx) {
      for (let r = 0; r < this.grid.length; r++) {
        for (let c = 0; c < COLS; c++) {
          const col = this.grid[r][c];
          if (col === null) continue;
          const [cx, cy] = this.center(r, c);
          this.bubble(ctx, cx, cy, BUBBLE_COLORS[col]);
        }
      }
    }

    drawDeathLine(ctx) {
      const y = Math.floor(this.deathY);
      for (let x = Math.floor(this.left); x < Math.floor(this.right); x += 16) {
        draw.line(ctx, [120, 60, 70], [x, y], [x + 8, y], 2);
      }
    }

    drawAim(ctx) {
      const col = ui.mix(this.accent, [255, 255, 255], 0.2);
      this.traceAim().forEach(([px, py], i) => {
        if (i % 2 === 0) draw.circle(ctx, col, [Math.floor(px), Math.floor(py)], 3);
      });
    }

    traceAim() {
      let [x, y] = this.cannon;
      let vx = Math.cos(this.aim), vy = Math.sin(this.aim);
      const pts = [];
      const step = this.r * 0.7;
      for (let i = 0; i < 140; i++) {
        x += vx * step;
        y += vy * step;
        if (x <= this.left + this.r) {
          x = this.left + this.r;
          vx = Math.abs(vx);
        } else if (x >= this.right - this.r) {
          x = this.right - this.r;
          vx = -Math.abs(vx);
        }
        if (y <= this.top + this.r) break;
        const stop = this.hitsBubble(x, y);
        pts.push([x, y]);
        if (stop) break;
      }
      return pts;
    }

    drawCannon(ctx) {
      const [cx, cy] = this.cannon;
      const rr = Math.floor(this.r) + 6;
      draw.circle(ctx, [40, 46, 66], [Math.floor(cx), Math.floor(cy)], rr);
      draw.circle(ctx, this.accent, [Math.floor(cx), Math.floor(cy)], rr, 2);
      if (this.shot === null && this.state === PLAY) this.bubble(ctx, cx, cy, BUBBLE_COLORS[this.cur]);
      if (this.shot !== null) this.bubble(ctx, this.shot.x, this.shot.y, BUBBLE_COLORS[this.shot.c]);
      // Nächste Kugel als Vorschau links neben der Kanone
      this.bubble(ctx, this.left + this.r, cy, BUBBLE_COLORS[this.nxt], this.r * 0.7);
    }

    drawHud(ctx) {
      ui.text(ctx, t("bub.title"), 20, 28, this.hud, this.accent, "midleft");
      ui.text(ctx, t("bub.score", { n: this.score }), this.width - 20, 28, this.small, ui.GOLD, "midright");
    }

    drawBanner(ctx) {
      const w = Math.min(this.width - 40, 460);
      const h = 108;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), Math.floor((this.height - h) / 2), w, h);
      draw.rect(ctx, [16, 18, 24, 238], rc);
      draw.rect(ctx, [228, 96, 96], rc, 2, 14);
      ui.text(ctx, t("bub.gameover"), rc.centerx, rc.y + 40, this.huge, [228, 96, 96], "center");
      ui.text(ctx, t("bub.score", { n: this.score }) + "   ·   " + t("common.enter_restart"), rc.centerx, rc.y + 78, this.small, ui.TEXT_DIM, "center");
    }
  }

  PG.register(BubbleShooterGame, {
    id: "BubbleShooterGame",
    key: "bubble",
    name: "Bubble Shooter",
    modes: [["easy", "bub.mode.easy"], ["classic", "bub.mode.classic"], ["hard", "bub.mode.hard"]],
  });
})();
