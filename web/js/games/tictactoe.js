/*
 * tictactoe.js - Tic-Tac-Toe mit Setup-Menü, Brettgrößen und KI (Port von games/tictactoe.py)
 * ===========================================================================================
 * - Setup-Screen: Schwierigkeit (Easy/Medium/Hard) und Brettgröße 3x3 .. 9x9.
 * - Allgemeines m,n,k-Spiel: gewonnen hat, wer K Steine in einer Reihe hat.
 *   Gewinnlänge K: 3x3 -> 3, 4x4 -> 4, ab 5x5 -> 5.
 * - KI:
 *     Easy   - zufällige Züge.
 *     Medium - gewinnt/blockt sofort, sonst heuristisch bester Zug.
 *     Hard   - 3x3: volle Minimax-Suche (unschlagbar);
 *              größere Bretter: tiefenbegrenzte Alpha-Beta-Suche mit Heuristik.
 * - Web-Version: nur 1 Spieler gegen die KI (der 2-Spieler-Modus entfällt).
 * - Spieler = X, KI = O, Klick-Steuerung.
 * - "Score"/Highscore = Anzahl gewonnener Runden (Siege).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const Rect = PG.Rect;
  const fl = Math.floor;

  // Identitätsfarben des Bretts (bewusst fest, unabhängig vom Theme).
  const COL_LINE = [90, 95, 120]; // Gitterlinien
  const COL_X = [90, 180, 255]; // Spieler X (blau)
  const COL_O = [255, 140, 90]; // Spieler O (orange)
  const COL_WIN = [240, 240, 120]; // Gewinnlinie
  const COL_WIN_BG = [60, 70, 40]; // Hinterlegung der Gewinnzellen

  const HUMAN = "X";
  const AI = "O";
  const DRAW = "Unentschieden";

  const SETUP = "setup", PLAY = "play", OVER = "over";

  const DIFF_ORDER = ["Easy", "Medium", "Hard"];
  const SIZES = [3, 4, 5, 6, 7, 8, 9];

  const WIN_SCORE = 10000000;
  // Bewertungsgewichte nach Anzahl eigener Steine in einem freien K-Fenster
  const LINE_WEIGHTS = [0, 1, 12, 120, 1200, 12000];

  /** Benötigte Anzahl Steine in einer Reihe je Brettgröße. */
  function winLength(n) {
    return n <= 4 ? n : 5;
  }

  /** Lokalisierter Anzeigename einer Schwierigkeit. */
  function diffLabel(name) {
    return t("ttt.diff." + name.toLowerCase());
  }

  class TicTacToeGame extends PG.Game {
    reset() {
      this.score = 0;
      this.winsX = 0;
      this.winsO = 0;
      this.gameOver = false;
      this.state = SETUP;

      this.diffName = "Hard";
      this.size = 3;

      this.makeFonts();
      this.buildSetupLayout();
    }

    // ===== Layout / Theme ===============================================
    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, fl(h / 24)));
      this.bigFont = ui.font(Math.max(30, fl(h / 11)), true);
      this.small = ui.font(Math.max(13, fl(h / 32)));
      this.mid = ui.font(Math.max(15, fl(h / 27)), true);
    }

    // ===== Setup-Screen =================================================
    buildSetupLayout() {
      const cx = fl(this.width / 2);
      const h = this.height;
      const bh = Math.max(40, Math.min(52, fl(h / 10)));

      this.diffY = Math.max(120, Math.trunc(h * 0.25));
      this.diffRects = {};
      DIFF_ORDER.forEach((name, i) => {
        this.diffRects[name] = new Rect(cx - 165 + i * 112, this.diffY, 100, bh);
      });

      this.sizeY = this.diffY + bh + Math.max(60, Math.trunc(h * 0.12));
      this.sizeRects = {};
      const bw = 52, gap = 6;
      const start = cx - fl((SIZES.length * (bw + gap) - gap) / 2);
      SIZES.forEach((n, i) => {
        this.sizeRects[n] = new Rect(start + i * (bw + gap), this.sizeY, bw, bh);
      });

      this.startRect = new Rect(cx - 90, this.sizeY + bh + 46, 180, 50);
    }

    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      ui.drawTitle(ctx, this.width, t("ttt.setup_title"), { accent: this.accent });

      ui.text(ctx, t("ttt.difficulty"), fl(this.width / 2) - 165, this.diffY - 8, this.mid, ui.TEXT_DIM, "bottomleft");
      for (const name of DIFF_ORDER) {
        ui.drawButton(ctx, this.diffRects[name], diffLabel(name), this.mid, name === this.diffName, { accent: this.accent });
      }

      ui.text(ctx, t("ttt.field"), fl(this.width / 2) - 165, this.sizeY - 8, this.mid, ui.TEXT_DIM, "bottomleft");
      for (const n of SIZES) {
        ui.drawButton(ctx, this.sizeRects[n], `${n}x${n}`, this.mid, n === this.size, { accent: this.accent });
      }

      ui.text(ctx, t("ttt.goal", { k: winLength(this.size) }), fl(this.width / 2), this.startRect.y - 22, this.small, ui.TEXT_DIM, "center");

      ui.drawButton(ctx, this.startRect, t("common.start"), this.mid, true, { accent: ui.GREEN });

      ui.drawFooter(ctx, this.width, this.height, t("ttt.setup_hint"));
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        if (["1", "2", "3"].includes(ev.key)) {
          this.diffName = DIFF_ORDER[Number(ev.key) - 1];
          this.playSound("select");
        } else if (ev.key === "Return" || ev.key === "space") {
          this.startRun();
        }
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        for (const name of DIFF_ORDER) {
          if (this.diffRects[name].collidepoint(p)) {
            this.diffName = name;
            this.playSound("select");
          }
        }
        for (const n of SIZES) {
          if (this.sizeRects[n].collidepoint(p)) {
            this.size = n;
            this.playSound("select");
          }
        }
        if (this.startRect.collidepoint(p)) this.startRun();
      }
    }

    // ===== Runde vorbereiten ============================================
    startRun() {
      this.n = this.size;
      this.k = winLength(this.n);
      this.maxDepth = { 3: 9, 4: 4, 5: 3 }[this.n] || 2; // Suchtiefe für Hard
      this.precomputeWindows();
      this.neueRunde();
      this.state = PLAY;
      this.playSound("click");
    }

    neueRunde() {
      const n = this.n;
      this.board = new Array(n * n).fill("");
      this.current = HUMAN;
      this.winner = null; // "X" | "O" | DRAW | null
      this.winCells = null;
      this.aiTimer = 0;
      this.gameOver = false;
      this.layoutBoard();
    }

    /** Brett-Geometrie (zentriertes Quadrat) aus width/height ableiten. */
    layoutBoard() {
      const n = this.n;
      this.boardSize = Math.min(this.width, this.height) - 90;
      this.cell = fl(this.boardSize / n);
      this.boardSize = this.cell * n; // exakt durch n teilbar
      this.ox = fl((this.width - this.boardSize) / 2);
      this.oy = fl((this.height - this.boardSize) / 2) + 10;
    }

    /** Alle K-Fenster (Reihen/Spalten/Diagonalen) für die Heuristik. */
    precomputeWindows() {
      const n = this.n, k = this.k;
      this.windows = [];
      const mk = (f) => Array.from({ length: k }, (_, i) => f(i));
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          if (c + k <= n) this.windows.push(mk((i) => r * n + (c + i))); // horizontal
          if (r + k <= n) this.windows.push(mk((i) => (r + i) * n + c)); // vertikal
          if (r + k <= n && c + k <= n) this.windows.push(mk((i) => (r + i) * n + (c + i))); // diagonal rechts unten
          if (r + k <= n && c - k + 1 >= 0) this.windows.push(mk((i) => (r + i) * n + (c - i))); // diagonal links unten
        }
      }
    }

    // ===== Eingabe ======================================================
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetupEvent(ev);
        return;
      }

      if (this.state === OVER) {
        if (ev.kind === "keydown" && (ev.key === "s" || ev.key === "S")) {
          this.state = SETUP;
          // Web: gameOver zurücksetzen, sonst liegt das Highscore-Banner über dem Setup.
          this.gameOver = false;
          this.playSound("click");
        } else if (ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"))) {
          this.neueRunde();
          this.state = PLAY;
        }
        return;
      }

      // PLAY: Klick setzt einen Stein
      if (ev.kind !== "mousedown") return;
      if (this.current === HUMAN) {
        const idx = this.feldAusPos(ev.pos);
        if (idx !== null && this.board[idx] === "") {
          this.applyMove(idx, HUMAN);
          if (this.state === PLAY) {
            this.current = AI;
            this.aiTimer = 0.25;
          }
        }
      }
    }

    feldAusPos(pos) {
      const [x, y] = pos;
      if (!(x >= this.ox && x < this.ox + this.boardSize && y >= this.oy && y < this.oy + this.boardSize)) return null;
      const col = fl((x - this.ox) / this.cell);
      const row = fl((y - this.oy) / this.cell);
      return row * this.n + col;
    }

    // ===== Spiellogik ===================================================
    update(dt) {
      if (this.state !== PLAY || this.current !== AI) return;
      // kleine Denkpause, damit der KI-Zug sichtbar ist
      this.aiTimer -= dt;
      if (this.aiTimer > 0) return;
      const zug = this.aiDecide();
      if (zug !== null) {
        this.applyMove(zug, AI);
        if (this.state === PLAY) this.current = HUMAN;
      }
    }

    applyMove(idx, sym) {
      this.board[idx] = sym;
      this.playSound("click");
      const cells = this.winningCells(idx, sym);
      if (cells) {
        this.winner = sym;
        this.winCells = cells;
        if (sym === HUMAN) {
          this.winsX++;
          this.score = this.winsX;
        } else {
          this.winsO++;
        }
        this.ende();
      } else if (!this.board.includes("")) {
        this.winner = DRAW;
        this.ende();
      }
    }

    ende() {
      this.state = OVER;
      this.gameOver = true; // damit die App den Highscore speichert
      if (this.winner === HUMAN || this.winner === AI) this.reportResult(this.winner === HUMAN);
      if (this.winner === DRAW) this.playSound("point");
      else if (this.winner === AI) this.playSound("gameover");
      else this.playSound("win");
    }

    /** Liefert die Gewinnzellen, falls 'sym' durch Zug auf idx gewinnt. */
    winningCells(idx, sym) {
      const n = this.n, k = this.k;
      const r = fl(idx / n), c = idx % n;
      for (const [dr, dc] of [[0, 1], [1, 0], [1, 1], [1, -1]]) {
        const cells = [idx];
        // in eine Richtung
        let rr = r + dr, cc = c + dc;
        while (rr >= 0 && rr < n && cc >= 0 && cc < n && this.board[rr * n + cc] === sym) {
          cells.push(rr * n + cc);
          rr += dr;
          cc += dc;
        }
        // in die Gegenrichtung
        rr = r - dr;
        cc = c - dc;
        while (rr >= 0 && rr < n && cc >= 0 && cc < n && this.board[rr * n + cc] === sym) {
          cells.push(rr * n + cc);
          rr -= dr;
          cc -= dc;
        }
        if (cells.length >= k) return cells;
      }
      return null;
    }

    wonAt(idx, sym) {
      return this.winningCells(idx, sym) !== null;
    }

    // ----- KI -----------------------------------------------------------
    empties() {
      const out = [];
      for (let i = 0; i < this.board.length; i++) if (this.board[i] === "") out.push(i);
      return out;
    }

    /** Sinnvolle Zugfelder: bei kleinen Brettern alle, sonst Nachbarn. */
    candidates() {
      const n = this.n;
      const empties = this.empties();
      if (n <= 3) return empties;
      const belegt = [];
      for (let i = 0; i < this.board.length; i++) if (this.board[i] !== "") belegt.push(i);
      if (!belegt.length) return [fl((n * n) / 2)]; // Mitte
      const nah = new Set();
      for (const i of belegt) {
        const r = fl(i / n), c = i % n;
        for (let dr = -1; dr <= 1; dr++) {
          for (let dc = -1; dc <= 1; dc++) {
            const rr = r + dr, cc = c + dc;
            if (rr >= 0 && rr < n && cc >= 0 && cc < n) {
              const j = rr * n + cc;
              if (this.board[j] === "") nah.add(j);
            }
          }
        }
      }
      return nah.size ? Array.from(nah) : empties;
    }

    /** Feld, auf dem 'sym' sofort gewinnen würde (oder null). */
    findWinning(sym) {
      for (const i of this.empties()) {
        this.board[i] = sym;
        const gewinnt = this.wonAt(i, sym);
        this.board[i] = "";
        if (gewinnt) return i;
      }
      return null;
    }

    aiDecide() {
      const empties = this.empties();
      if (!empties.length) return null;
      // Erstes Feld: Mitte (gut und schnell)
      if (empties.length === this.n * this.n) return fl((this.n * this.n) / 2);

      // Immer sofort gewinnen, wenn möglich
      const gewinn = this.findWinning(AI);
      if (gewinn !== null) return gewinn;

      if (this.diffName === "Easy") return PG.rand.choice(empties);

      // Gegnerischen Sofortgewinn blocken
      const block = this.findWinning(HUMAN);
      if (block !== null) return block;

      if (this.diffName === "Medium") return this.greedy();
      return this.searchRoot();
    }

    /** Heuristik: Summe der Fensterwerte (AI positiv, HUMAN negativ). */
    evaluate() {
      let score = 0;
      const b = this.board;
      const maxW = LINE_WEIGHTS.length - 1;
      for (const win of this.windows) {
        let ai = 0, hu = 0;
        for (const i of win) {
          const v = b[i];
          if (v === AI) ai++;
          else if (v === HUMAN) hu++;
        }
        if (ai && hu) continue; // gemischt -> wertlos
        if (ai) score += LINE_WEIGHTS[Math.min(ai, maxW)];
        else if (hu) score -= LINE_WEIGHTS[Math.min(hu, maxW)];
      }
      return score;
    }

    /** Setzt den Stein dorthin, wo die Stellung am besten bewertet wird. */
    greedy() {
      let best = -1e18, besteIdx = null;
      for (const idx of this.candidates()) {
        this.board[idx] = AI;
        const val = this.evaluate();
        this.board[idx] = "";
        if (val > best) {
          best = val;
          besteIdx = idx;
        }
      }
      return besteIdx !== null ? besteIdx : PG.rand.choice(this.empties());
    }

    searchRoot() {
      let best = -1e18, besteIdx = null;
      let alpha = -1e18;
      const beta = 1e18;
      for (const idx of this.candidates()) {
        this.board[idx] = AI;
        const val = this.wonAt(idx, AI) ? WIN_SCORE : this.search(this.maxDepth - 1, alpha, beta, false);
        this.board[idx] = "";
        if (val > best) {
          best = val;
          besteIdx = idx;
        }
        alpha = Math.max(alpha, best);
      }
      return besteIdx !== null ? besteIdx : PG.rand.choice(this.empties());
    }

    /** Alpha-Beta-Suche; bewertet die Stellung aus AI-Sicht. */
    search(depth, alpha, beta, maximizing) {
      if (depth === 0 || !this.board.includes("")) return this.evaluate();
      const cand = this.candidates();
      if (!cand.length) return this.evaluate();

      if (maximizing) {
        // AI am Zug
        let best = -1e18;
        for (const idx of cand) {
          this.board[idx] = AI;
          const val = this.wonAt(idx, AI) ? WIN_SCORE - (this.maxDepth - depth) : this.search(depth - 1, alpha, beta, false);
          this.board[idx] = "";
          best = Math.max(best, val);
          alpha = Math.max(alpha, best);
          if (beta <= alpha) break;
        }
        return best;
      }
      // HUMAN am Zug
      let best = 1e18;
      for (const idx of cand) {
        this.board[idx] = HUMAN;
        const val = this.wonAt(idx, HUMAN) ? -WIN_SCORE + (this.maxDepth - depth) : this.search(depth - 1, alpha, beta, true);
        this.board[idx] = "";
        best = Math.min(best, val);
        beta = Math.min(beta, best);
        if (beta <= alpha) break;
      }
      return best;
    }

    // ===== Zeichnen =====================================================
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      ui.drawBackground(ctx, this.width, this.height);
      const n = this.n, cell = this.cell;
      const lw = Math.max(2, fl(cell / 20)); // Linienbreite passend zur Zellgröße

      // Gewinnzellen hinterlegen (unter dem Gitter)
      if (this.winCells) {
        for (const idx of this.winCells) {
          const r = fl(idx / n), c = idx % n;
          draw.rect(ctx, COL_WIN_BG, [this.ox + c * cell, this.oy + r * cell, cell, cell]);
        }
      }

      // Gitter
      for (let i = 1; i < n; i++) {
        const x = this.ox + i * cell;
        const y = this.oy + i * cell;
        draw.line(ctx, COL_LINE, [x, this.oy], [x, this.oy + this.boardSize], lw);
        draw.line(ctx, COL_LINE, [this.ox, y], [this.ox + this.boardSize, y], lw);
      }
      draw.rect(ctx, COL_LINE, [this.ox, this.oy, this.boardSize, this.boardSize], lw);

      // Symbole
      const rad = Math.trunc(cell * 0.3);
      const mw = Math.max(3, fl(cell / 12));
      for (let i = 0; i < this.board.length; i++) {
        const sym = this.board[i];
        if (!sym) continue;
        const r = fl(i / n), c = i % n;
        const mx = this.ox + c * cell + fl(cell / 2);
        const my = this.oy + r * cell + fl(cell / 2);
        if (sym === HUMAN) {
          draw.line(ctx, COL_X, [mx - rad, my - rad], [mx + rad, my + rad], mw);
          draw.line(ctx, COL_X, [mx + rad, my - rad], [mx - rad, my + rad], mw);
        } else {
          draw.circle(ctx, COL_O, [mx, my], rad, mw);
        }
      }

      // Gewinnlinie
      if (this.winCells && this.winCells.length >= 2) {
        const a = this.winCells[0], z = this.winCells[this.winCells.length - 1];
        const p1 = [this.ox + (a % n) * cell + fl(cell / 2), this.oy + fl(a / n) * cell + fl(cell / 2)];
        const p2 = [this.ox + (z % n) * cell + fl(cell / 2), this.oy + fl(z / n) * cell + fl(cell / 2)];
        draw.line(ctx, COL_WIN, p1, p2, Math.max(4, fl(cell / 10)));
      }

      // Kopfzeile
      const kopf = t("ttt.header_sp", { diff: diffLabel(this.diffName), n, k: this.k });
      ui.text(ctx, kopf, 10, 8, this.small, ui.TEXT_DIM);
      ui.text(ctx, t("ttt.score_sp", { wins: this.winsX }), this.width - 10, 8, this.small, ui.TEXT, "topright");
      if (this.state === PLAY) {
        const zug = this.current === HUMAN ? t("ttt.turn_you") : t("ttt.turn_ai");
        ui.text(ctx, zug, 10, this.height - this.small.height - 8, this.small, ui.TEXT);
      }

      if (this.state === OVER) this.drawOver(ctx);
    }

    drawOver(ctx) {
      let msg, farbe;
      if (this.winner === DRAW) {
        msg = t("common.draw");
        farbe = ui.GOLD;
      } else if (this.winner === HUMAN) {
        msg = t("ttt.win_you");
        farbe = ui.GREEN;
      } else {
        msg = t("ttt.win_ai");
        farbe = ui.RED;
      }

      // Kompaktes, transluzentes Panel statt Vollbild-Abdunklung.
      const zeilen = [
        [msg, this.bigFont, farbe],
        [t("ttt.new_round"), this.font, null], // null -> pulsierend
        [t("ttt.settings"), this.small, ui.TEXT_DIM],
      ];
      const bw = Math.max(...zeilen.map(([txt, f]) => f.width(txt))) + 80;
      const bh = zeilen.reduce((s, [, f]) => s + f.height, 0) + 66;
      const rect = new Rect(0, 0, bw, bh);
      rect.center = [fl(this.width / 2), fl(this.height / 2)];

      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], rect, 0, 16);
      draw.rect(ctx, farbe, rect, 2, 16);

      let y = rect.y + 20;
      for (const [txt, f, c] of zeilen) {
        const col = c === null ? ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0)) : c;
        ui.text(ctx, txt, rect.centerx, y, f, col, "midtop");
        y += f.height + 11;
      }
    }
  }

  PG.register(TicTacToeGame, {
    id: "TicTacToeGame",
    key: "tictactoe",
    name: "Tic-Tac-Toe",
  });
})();
