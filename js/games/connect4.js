/*
 * connect4.js - Vier gewinnt (Connect Four) (Port von games/connect4.py)
 * ======================================================================
 * - Klassisches 7x6-Brett; Stein fällt animiert in die gewählte Spalte.
 * - KI mit drei Stärken (easy/medium/hard) über Minimax mit Alpha-Beta-
 *   Schnitt und Fenster-Bewertung; easy patzt absichtlich.
 * - Punkte (Highscore) = kumulierte Siege gegen die KI in einer Sitzung
 *   (tictactoe-Konvention).
 * - Web-Version: nur Einzelspieler (der 2-Spieler-Modus entfällt).
 *
 * Steuerung: Maus (Spalte anklicken) oder Links/Rechts + Runter/Leertaste/
 * Enter; 1-7 wählt die Spalte direkt. Nach Rundenende: Enter = neue Runde,
 * S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme):
  // blaue Brettplatte mit roten/gelben Steinen IST Vier gewinnt.
  const COL_PLATE = [38, 58, 140];
  const COL_PLATE_EDGE = [26, 40, 100];
  const COL_EMPTY = [18, 22, 36];
  const COL_P1 = [230, 90, 80]; // Rot
  const COL_P2 = [245, 205, 90]; // Gelb

  const DIFFS = ["easy", "medium", "hard"];
  const DEPTHS = [2, 4, 5];

  const COLS = 7, ROWS = 6;
  const ORDER = [3, 2, 4, 1, 5, 0, 6]; // Spaltenreihenfolge für Alpha-Beta

  const SETUP = "setup", PLAY = "play", ANIM = "anim", OVER = "over";

  // Web-eigener Untertitel (der Python-Text erwähnt den entfallenen 2-Spieler-Modus)
  PG.addStrings({
    de: { "web.connect4.subtitle": "Vier in einer Reihe - gegen die KI" },
    en: { "web.connect4.subtitle": "Four in a row - against the AI" },
  });

  // Alle 69 Viererfenster einmalig vorberechnen (Liste von [r,c]-Paaren)
  const WINDOWS = [];
  for (let r = 0; r < ROWS; r++) for (let c = 0; c < COLS - 3; c++) WINDOWS.push([0, 1, 2, 3].map((i) => [r, c + i]));
  for (let c = 0; c < COLS; c++) for (let r = 0; r < ROWS - 3; r++) WINDOWS.push([0, 1, 2, 3].map((i) => [r + i, c]));
  for (let r = 0; r < ROWS - 3; r++) {
    for (let c = 0; c < COLS - 3; c++) {
      WINDOWS.push([0, 1, 2, 3].map((i) => [r + i, c + i]));
      WINDOWS.push([0, 1, 2, 3].map((i) => [r + 3 - i, c + i]));
    }
  }

  class ConnectFourGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.diff = Math.max(0, Math.min(2, parseInt(this.opts.difficulty, 10) || 0));

      this.makeFonts();
      this.wins = [0, 0];
      this.starter = 0;
      this.buildSetupLayout();
      this.newRound();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größen aus der Fensterhöhe abgeleitet. */
    makeFonts() {
      this.small = ui.font(Math.max(13, Math.min(22, Math.floor(this.height / 30))));
      this.tiny = ui.font(Math.max(11, Math.min(18, Math.floor(this.height / 38))));
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    layout() {
      this.hudH = Math.max(40, Math.floor(this.height * 0.085));
      this.cell = Math.floor(Math.min((this.width - 40) / 7, (this.height - this.hudH - 20) / 7));
      this.bw = 7 * this.cell;
      this.bh = 6 * this.cell;
      this.bx = Math.floor((this.width - this.bw) / 2);
      this.by = this.hudH + this.cell;
      this.r = Math.floor(this.cell * 0.42);
    }

    newRound() {
      this.board = Array.from({ length: ROWS }, () => new Array(COLS).fill(0)); // [row][col], 0 oben
      this.player = this.starter; // 0 = Rot (Du), 1 = Gelb (KI)
      this.hover = 3;
      this.anim = null; // {col, row, y, vy, player}
      this.winCells = null;
      this.winner = null;
      this.msg = null;
      this.msgT = 0;
      this.aiDelay = 0;
      this._resultReported = false; // jede Runde ist eine eigene Partie (Statistik)
      this.layout();
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(360, this.width - 60);
      const y0 = Math.floor(this.height * 0.32);
      this.diffRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 58, bw, 48));
      this.startRect = new PG.Rect(cx - 95, y0 + 3 * 58 + 14, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diff = parseInt(k, 10) - 1;
          this.saveSetting("difficulty", this.diff);
          this.playSound("click");
        } else if (k === "Up" || k === "w" || k === "W") {
          this.diff = PG.mod(this.diff - 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "Down" || k === "s" || k === "S") {
          this.diff = PG.mod(this.diff + 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.newRound();
          this.state = PLAY;
          this.playSound("click");
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diff = i;
            this.saveSetting("difficulty", i);
            this.playSound("click");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) {
          this.newRound();
          this.state = PLAY;
          this.playSound("click");
        }
      }
    }

    // ===================================================== Brett-Logik
    /** Unterste freie Reihe der Spalte (oder null). */
    dropRow(board, col) {
      for (let row = ROWS - 1; row >= 0; row--) if (board[row][col] === 0) return row;
      return null;
    }

    /** Gewinnzellen ab dem letzten Zug (oder null). */
    checkWin(board, row, col) {
      const p = board[row][col];
      for (const [dr, dc] of [[0, 1], [1, 0], [1, 1], [1, -1]]) {
        const cells = [[row, col]];
        for (const sgn of [1, -1]) {
          let rr = row + dr * sgn, cc = col + dc * sgn;
          while (rr >= 0 && rr < ROWS && cc >= 0 && cc < COLS && board[rr][cc] === p) {
            cells.push([rr, cc]);
            rr += dr * sgn;
            cc += dc * sgn;
          }
        }
        if (cells.length >= 4) return cells;
      }
      return null;
    }

    /** Schnelle Variante für die KI-Suche (nur ja/nein, ohne Zell-Listen). */
    wins4(board, row, col) {
      const p = board[row][col];
      const dirs = DIRS;
      for (let d = 0; d < 4; d++) {
        const dr = dirs[d][0], dc = dirs[d][1];
        let n = 1;
        let rr = row + dr, cc = col + dc;
        while (rr >= 0 && rr < ROWS && cc >= 0 && cc < COLS && board[rr][cc] === p) { n++; rr += dr; cc += dc; }
        rr = row - dr; cc = col - dc;
        while (rr >= 0 && rr < ROWS && cc >= 0 && cc < COLS && board[rr][cc] === p) { n++; rr -= dr; cc -= dc; }
        if (n >= 4) return true;
      }
      return false;
    }

    isFull(board) {
      for (let c = 0; c < COLS; c++) if (board[0][c] === 0) return false;
      return true;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.starter = 1 - this.starter;
            this.gameOver = false;
            this.newRound();
            this.state = PLAY;
            this.playSound("click");
          } else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown") {
          this.starter = 1 - this.starter;
          this.gameOver = false;
          this.newRound();
          this.state = PLAY;
          this.playSound("click");
        }
        return;
      }
      if (this.state !== PLAY) return;
      // Nur Spieler 0 (Rot) ist steuerbar - Gelb ist die KI.
      if (this.player === 1) return;
      if (ev.kind === "mousemove") {
        const col = Math.floor((ev.pos[0] - this.bx) / this.cell);
        if (col >= 0 && col < COLS) this.hover = col;
      } else if (ev.kind === "mousedown") {
        const col = Math.floor((ev.pos[0] - this.bx) / this.cell);
        if (col >= 0 && col < COLS) this.tryDrop(col);
      } else if (ev.kind === "keydown") {
        const k = ev.key || "";
        if (this.isAction(k, "left") || k === "Left") {
          this.hover = PG.mod(this.hover - 1, COLS);
          this.playSound("move");
        } else if (this.isAction(k, "right") || k === "Right") {
          this.hover = PG.mod(this.hover + 1, COLS);
          this.playSound("move");
        } else if (this.isAction(k, "down") || k === "Down" || k === "space" || k === "Return") {
          this.tryDrop(this.hover);
        } else if (k.length === 1 && "1234567".includes(k)) {
          this.tryDrop(parseInt(k, 10) - 1);
        } else if (k.startsWith("KP_") && k.length === 4 && "1234567".includes(k[3])) {
          this.tryDrop(parseInt(k[3], 10) - 1);
        }
      }
    }

    tryDrop(col) {
      const row = this.dropRow(this.board, col);
      if (row === null) {
        this.msg = t("c4.col_full");
        this.msgT = 1.2;
        this.playSound("click");
        return;
      }
      this.hover = col;
      this.anim = { col, row, y: this.by - this.cell, vy: 0, player: this.player };
      this.state = ANIM;
      this.playSound("move");
    }

    // ===================================================== Spiellogik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }

      if (this.state === ANIM && this.anim) {
        const a = this.anim;
        a.vy += 2600 * dt;
        a.y += a.vy * dt;
        const target = this.by + a.row * this.cell;
        if (a.y >= target) {
          this.board[a.row][a.col] = a.player + 1;
          this.playSound("lock");
          this.anim = null;
          const cells = this.checkWin(this.board, a.row, a.col);
          if (cells) this.ende(cells, a.player);
          else if (this.isFull(this.board)) this.ende(null, null);
          else {
            this.player = 1 - this.player;
            this.state = PLAY;
            this.aiDelay = 0.35;
          }
        }
        return;
      }

      if (this.state === PLAY && this.player === 1) {
        this.aiDelay -= dt;
        // Die Suche (max. Tiefe 5, Alpha-Beta) braucht nur wenige ms -
        // sie läuft daher in einem Frame, nach der "KI denkt..."-Pause.
        if (this.aiDelay <= 0) this.tryDrop(this.aiMove());
      }
    }

    ende(cells, winner) {
      this.winCells = cells;
      this.winner = winner;
      this.state = OVER;
      if (winner !== null) {
        this.wins[winner] += 1;
        if (winner === 0) {
          this.score = this.wins[0];
          this.playSound("win");
          this.reportResult(true);
        } else {
          this.playSound("gameover");
          this.reportResult(false);
        }
      }
      this.gameOver = true; // die App speichert den Score einmalig
    }

    // ===================================================== KI
    validCols(board) {
      return ORDER.filter((c) => board[0][c] === 0);
    }

    aiMove() {
      const board = this.board;
      const valid = this.validCols(board);
      if (!valid.length) return 3;
      // 1) Sofortiger Sieg
      for (const c of valid) {
        const r = this.dropRow(board, c);
        board[r][c] = 2;
        const won = this.wins4(board, r, c);
        board[r][c] = 0;
        if (won) return c;
      }
      // 2) Sofortigen Verlust blocken (easy patzt zu 40 %)
      for (const c of valid) {
        const r = this.dropRow(board, c);
        board[r][c] = 1;
        const lose = this.wins4(board, r, c);
        board[r][c] = 0;
        if (lose && !(this.diff === 0 && PG.rand.random() < 0.4)) return c;
      }
      // 3) Minimax mit Alpha-Beta
      const depth = DEPTHS[this.diff];
      const scores = [];
      for (const c of valid) {
        const r = this.dropRow(board, c);
        board[r][c] = 2;
        if (this.wins4(board, r, c)) {
          board[r][c] = 0;
          return c;
        }
        const val = this.minimax(board, depth - 1, -1e9, 1e9, false);
        board[r][c] = 0;
        scores.push([val, c]);
      }
      // wie Python sort(reverse=True): nach Wert, bei Gleichstand höhere Spalte zuerst
      scores.sort((a, b) => b[0] - a[0] || b[1] - a[1]);
      if (this.diff === 0 && scores.length > 1 && PG.rand.random() < 0.3) return scores[1][1];
      return scores[0][1];
    }

    minimax(board, depth, alpha, beta, maximizing) {
      const valid = this.validCols(board);
      if (depth === 0 || !valid.length) return this.evaluate(board);
      if (maximizing) {
        let best = -1e9;
        for (const c of valid) {
          const r = this.dropRow(board, c);
          board[r][c] = 2;
          const val = this.wins4(board, r, c) ? 100000 + depth : this.minimax(board, depth - 1, alpha, beta, false);
          board[r][c] = 0;
          best = Math.max(best, val);
          alpha = Math.max(alpha, best);
          if (alpha >= beta) break;
        }
        return best;
      }
      let best = 1e9;
      for (const c of valid) {
        const r = this.dropRow(board, c);
        board[r][c] = 1;
        const val = this.wins4(board, r, c) ? -100000 - depth : this.minimax(board, depth - 1, alpha, beta, true);
        board[r][c] = 0;
        best = Math.min(best, val);
        beta = Math.min(beta, best);
        if (alpha >= beta) break;
      }
      return best;
    }

    /** Fenster-Bewertung aller 69 Viererfenster + Zentrums-Bonus. */
    evaluate(board) {
      let score = 0;
      for (let r = 0; r < ROWS; r++) if (board[r][3] === 2) score += 6;
      for (let w = 0; w < WINDOWS.length; w++) {
        const win = WINDOWS[w];
        let ai = 0, hu = 0;
        for (let i = 0; i < 4; i++) {
          const v = board[win[i][0]][win[i][1]];
          if (v === 2) ai++;
          else if (v === 1) hu++;
        }
        if (hu === 0 && (ai === 2 || ai === 3)) score += ai === 2 ? 10 : 120;
        else if (ai === 0 && (hu === 2 || hu === 3)) score -= Math.trunc((hu === 2 ? 10 : 120) * 1.2);
      }
      return score;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawHud(ctx);
      this.drawBoard(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    discColor(p) {
      return p === 0 ? COL_P1 : COL_P2;
    }

    drawBoard(ctx) {
      const half = Math.floor(this.cell / 2);
      // Hover-Stein über dem Brett
      if (this.state === PLAY && this.player === 0) {
        const hx = this.bx + this.hover * this.cell + half;
        const hy = this.by - half;
        draw.circle(ctx, this.discColor(this.player), [hx, hy], this.r);
        draw.circle(ctx, COL_PLATE_EDGE, [hx, hy], this.r, 2);
      }

      // Brettplatte
      const plate = [this.bx - 8, this.by - 8, this.bw + 16, this.bh + 16];
      draw.rect(ctx, COL_PLATE, plate, 0, 12);
      draw.rect(ctx, COL_PLATE_EDGE, plate, 3, 12);

      // Zellen (leer oder Stein)
      for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) {
          const cx = this.bx + c * this.cell + half;
          const cy = this.by + r * this.cell + half;
          const v = this.board[r][c];
          draw.circle(ctx, v === 0 ? COL_EMPTY : this.discColor(v - 1), [cx, cy], this.r);
          draw.circle(ctx, COL_PLATE_EDGE, [cx, cy], this.r, 2);
        }
      }

      // Fallender Stein (aufs Brett geclippt, damit er "hineinfällt")
      if (this.anim) {
        const a = this.anim;
        ctx.save();
        ctx.beginPath();
        ctx.rect(this.bx, this.by - this.cell, this.bw, this.bh + this.cell);
        ctx.clip();
        const cx = this.bx + a.col * this.cell + half;
        const cy = Math.floor(a.y) + half;
        draw.circle(ctx, this.discColor(a.player), [cx, cy], this.r);
        ctx.restore();
      }

      // Sieg-Linie pulsierend hervorheben
      if (this.winCells) {
        const k = 0.5 + 0.5 * Math.abs((ui.ticks() % 1000) - 500) / 500;
        for (const [r, c] of this.winCells) {
          const cx = this.bx + c * this.cell + half;
          const cy = this.by + r * this.cell + half;
          draw.circle(ctx, [255, 255, 255], [cx, cy], this.r + 2, Math.max(2, Math.floor(4 * k)));
        }
      }
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = Math.floor(this.hudH / 2);
      ui.text(ctx, `${t("c4.score_sp")}: ${this.wins[0]}`, 12, cy, this.small, COL_P1, "midleft");
      ui.text(ctx, `${this.wins[1]} :${t("common.ai")}`, this.width - 12, cy, this.small, COL_P2, "midright");

      if (this.state === PLAY || this.state === ANIM) {
        const mid = this.player === 1 ? t("c4.ai_thinks") : t("c4.turn", { name: t("common.player1") });
        ui.text(ctx, mid, this.width / 2, cy, this.small, this.discColor(this.player), "center");
      }
      if (this.msg) ui.text(ctx, this.msg, this.width / 2, this.hudH + 12, this.tiny, ui.GOLD, "center");
    }

    drawOver(ctx) {
      const hh = this.huge.height;
      const sh = this.small.height;
      const bandH = hh + sh + 36;
      const y = Math.floor(this.height / 2 - bandH / 2);
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], [0, y, this.width, bandH]);
      draw.line(ctx, this.accent, [0, y], [this.width, y], 2);
      draw.line(ctx, this.accent, [0, y + bandH - 1], [this.width, y + bandH - 1], 2);
      const cx = this.width / 2;
      let head, col;
      if (this.winner === null) {
        head = t("common.draw");
        col = ui.TEXT_DIM;
      } else {
        head = t(this.winner === 0 ? "c4.win_you" : "c4.win_ai");
        col = this.discColor(this.winner);
      }
      ui.text(ctx, head, cx, y + 12, this.huge, col, "midtop");
      ui.text(ctx, t("c4.new_round"), cx, y + 12 + hh + 10, this.small, ui.TEXT_DIM, "midtop");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, PG.gameName(this.meta.id).toUpperCase(), cx, Math.floor(this.height * 0.14), this.huge, this.accent, "center");
      ui.text(ctx, t("web.connect4.subtitle"), cx, Math.floor(this.height * 0.21), this.small, ui.TEXT_DIM, "center");
      for (let i = 0; i < this.diffRects.length; i++) {
        const r = this.diffRects[i];
        const on = i === this.diff;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 10);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 10);
        ui.text(ctx, t("c4.diff." + DIFFS[i]), r.x + 18, r.centery, this.font, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
      }
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("c4.setup_hint"), cx, this.height - 16, this.tiny, ui.TEXT_FAINT, "center");
    }
  }

  const DIRS = [[0, 1], [1, 0], [1, 1], [1, -1]];

  PG.register(ConnectFourGame, {
    id: "ConnectFourGame",
    key: "connect4",
    name: { default: "Connect Four", de: "Vier gewinnt", fr: "Puissance 4", es: "Cuatro en línea", pt: "Quatro em linha" },
    settingsKey: "connect4",
    defaults: { difficulty: 1 },
  });
})();
