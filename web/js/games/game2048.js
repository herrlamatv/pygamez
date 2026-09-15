/*
 * game2048.js - 2048, das Zahlen-Schiebespiel (Port von games/game2048.py)
 * =========================================================================
 * - Pfeiltasten/WASD schieben alle Kacheln; gleiche Zahlen verschmelzen.
 * - Nach jedem gültigen Zug erscheint eine neue Kachel (2 oder 4).
 * - Ziel: die 2048er-Kachel. Danach darf man weiterspielen.
 * - Kein Zug mehr möglich -> Game Over.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const SIZE = 4;

  const COL_BOARD = [40, 44, 58];
  const COL_EMPTY = [55, 60, 78];
  const COL_TILE_TEXT = [235, 235, 240];
  const COL_TILE_TEXT_DARK = [60, 55, 45];
  const COL_TILE_BIG = [60, 58, 50];

  const TILE_COLORS = {
    2: [238, 228, 218], 4: [237, 224, 200], 8: [242, 177, 121], 16: [245, 149, 99],
    32: [246, 124, 95], 64: [246, 94, 59], 128: [237, 207, 114], 256: [237, 204, 97],
    512: [237, 200, 80], 1024: [237, 197, 63], 2048: [237, 194, 46],
  };

  class Game2048 extends PG.Game {
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.won = false;
      this.buildLayout();
      this.grid = Array.from({ length: SIZE }, () => new Array(SIZE).fill(0));
      this.addTile();
      this.addTile();
    }

    // ----- Layout ----------------------------------------------------------
    buildLayout() {
      this.boardPx = Math.min(this.width, this.height) - 60;
      this.pad = 10;
      this.cell = Math.floor((this.boardPx - this.pad * (SIZE + 1)) / SIZE);
      this.ox = Math.floor((this.width - this.boardPx) / 2);
      this.oy = Math.floor((this.height - this.boardPx) / 2) + 10;
      const h = this.height;
      this.font = ui.font(Math.max(16, Math.floor(h / 24)));
      this.bigFont = ui.font(Math.max(32, Math.floor(h / 10)), true);
      this.tileFonts = [
        ui.font(Math.max(18, this.cell * 0.4), true),
        ui.font(Math.max(15, this.cell * 0.32), true),
        ui.font(Math.max(12, this.cell * 0.26), true),
      ];
    }

    addTile() {
      const free = [];
      for (let r = 0; r < SIZE; r++) for (let c = 0; c < SIZE; c++) if (this.grid[r][c] === 0) free.push([r, c]);
      if (!free.length) return;
      const [r, c] = PG.rand.choice(free);
      this.grid[r][c] = PG.rand.random() < 0.1 ? 4 : 2;
    }

    // ----- Eingabe ---------------------------------------------------------
    handleEvent(ev) {
      if (ev.kind !== "keydown") return;
      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.reset();
        return;
      }
      let dir = null;
      if (this.isAction(ev.key, "left")) dir = "L";
      else if (this.isAction(ev.key, "right")) dir = "R";
      else if (this.isAction(ev.key, "up")) dir = "U";
      else if (this.isAction(ev.key, "down")) dir = "D";
      if (!dir) return;

      const before = this.score;
      const had2048 = this.won;
      if (this.move(dir)) {
        this.playSound(this.score > before ? "merge" : "move");
        if (this.won && !had2048) {
          this.reportResult(true);
          this.achEvent("tile_2048");
          this.playSound("win");
          this.rumble(150);
        }
        this.addTile();
        if (!this.movesAvailable()) {
          this.gameOver = true;
          this.playSound("gameover");
        }
      }
    }

    update(dt) {
      // rundenbasiert - keine zeitabhängige Logik
    }

    // ----- Schiebe-/Verschmelz-Logik ------------------------------------------
    compress(row) {
      const nums = row.filter((z) => z !== 0);
      const out = [];
      let i = 0;
      while (i < nums.length) {
        if (i + 1 < nums.length && nums[i] === nums[i + 1]) {
          const merged = nums[i] * 2;
          out.push(merged);
          this.score += merged;
          if (merged === 2048) this.won = true;
          i += 2;
        } else {
          out.push(nums[i]);
          i += 1;
        }
      }
      while (out.length < SIZE) out.push(0);
      return out;
    }

    move(dir) {
      const old = JSON.stringify(this.grid);
      const col = (c) => this.grid.map((row) => row[c]);
      let next;
      if (dir === "L") next = this.grid.map((row) => this.compress(row));
      else if (dir === "R") next = this.grid.map((row) => this.compress(row.slice().reverse()).reverse());
      else {
        const cols = [];
        for (let c = 0; c < SIZE; c++) {
          cols.push(dir === "U" ? this.compress(col(c)) : this.compress(col(c).reverse()).reverse());
        }
        next = Array.from({ length: SIZE }, (_, r) => cols.map((cc) => cc[r]));
      }
      this.grid = next;
      return JSON.stringify(next) !== old;
    }

    movesAvailable() {
      for (let r = 0; r < SIZE; r++) {
        for (let c = 0; c < SIZE; c++) {
          const v = this.grid[r][c];
          if (v === 0) return true;
          if (c + 1 < SIZE && v === this.grid[r][c + 1]) return true;
          if (r + 1 < SIZE && v === this.grid[r + 1][c]) return true;
        }
      }
      return false;
    }

    // ----- Zeichnen -----------------------------------------------------------
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);

      ui.text(ctx, t("common.points", { score: this.score }), 12, 8, this.font, ui.TEXT);
      if (this.won && !this.gameOver) {
        ui.text(ctx, t("g2048.reached"), this.width - 12, 8, this.font, ui.GOLD, "topright");
      }

      draw.rect(ctx, COL_BOARD, [this.ox, this.oy, this.boardPx, this.boardPx], 0, 8);
      for (let r = 0; r < SIZE; r++) {
        for (let c = 0; c < SIZE; c++) {
          const x = this.ox + this.pad + c * (this.cell + this.pad);
          const y = this.oy + this.pad + r * (this.cell + this.pad);
          const v = this.grid[r][c];
          const color = v ? TILE_COLORS[v] || COL_TILE_BIG : COL_EMPTY;
          draw.rect(ctx, color, [x, y, this.cell, this.cell], 0, 6);
          if (v) {
            const tcol = v <= 4 ? COL_TILE_TEXT_DARK : COL_TILE_TEXT;
            const f = this.tileFonts[v < 100 ? 0 : v < 1000 ? 1 : 2];
            ui.text(ctx, String(v), x + this.cell / 2, y + this.cell / 2, f, tcol, "center");
          }
        }
      }

      if (this.gameOver) {
        ui.drawOverlayBox(ctx, this.width, this.height, t("common.game_over"), ui.RED, t("common.enter_restart"), { big: this.bigFont, small: this.font });
      }
    }
  }

  PG.register(Game2048, {
    id: "Game2048",
    key: "2048",
    name: "2048",
  });
})();
