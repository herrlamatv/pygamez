/*
 * reversi.js - Reversi (Othello) gegen die KI (Port von games/reversi.py)
 * =======================================================================
 * - Klassisches 8x8-Brett mit der Standard-Startstellung (vier Steine im Zentrum).
 * - Ein Zug ist nur erlaubt, wenn er mindestens eine gegnerische Kette in gerader
 *   Linie einschließt; alle eingeschlossenen Steine werden umgedreht.
 * - Hat ein Spieler keinen gültigen Zug, wird automatisch gepasst; hat KEIN
 *   Spieler einen Zug, endet die Partie. Sieger = mehr Steine auf dem Brett.
 * - KI mit drei Stärken (easy/medium/hard) über Negamax mit Alpha-Beta-Schnitt
 *   und positionsgewichteter Bewertung (Ecken hoch, X-/C-Felder negativ) plus
 *   Mobilität; easy patzt absichtlich.
 * - Punkte (Highscore) = kumulierte Siege gegen die KI in einer Sitzung.
 * - Web-Version: nur Einzelspieler. Die KI-Suche wird über mehrere Frames
 *   verteilt (ein Wurzelzug nach dem anderen), damit nichts einfriert.
 *
 * Steuerung: Maus (Feld anklicken) oder Pfeile/WASD bewegen den Auswahlrahmen,
 * Leertaste/Enter setzt den Stein. Nach Rundenende: Enter = neue Runde,
 * S = Auswahl.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Untertitel ohne "oder zu zweit" (Web-Version = nur gegen die KI)
  PG.addStrings({
    de: {
      "web.reversi.subtitle": "Steine einschließen und umdrehen - gegen die KI",
      "web.reversi.pass_ai": "KI muss passen",
      "web.reversi.pass_you": "Du musst passen",
    },
    en: {
      "web.reversi.subtitle": "Enclose and flip discs - versus the AI",
      "web.reversi.pass_ai": "AI has to pass",
      "web.reversi.pass_you": "You have to pass",
    },
  });

  // Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme):
  // das grüne Brett und die Schwarz/Weiß-Steine SIND Reversi.
  const COL_BOARD = [28, 92, 58];
  const COL_BOARD_DARK = [22, 74, 46];
  const COL_GRID = [16, 54, 34];
  const COL_PLATE = [18, 40, 28];
  const COL_P1 = [30, 33, 42]; // Schwarz (Spieler 0)
  const COL_P1_HI = [78, 84, 100];
  const COL_P2 = [238, 240, 246]; // Weiß (Spieler 1)
  const COL_P2_HI = [255, 255, 255];
  const COL_HINT = [90, 200, 150];

  const DIFFS = ["easy", "medium", "hard"];
  const DEPTHS = [1, 3, 4];

  const N = 8;
  const DIRS = [[-1, -1], [-1, 0], [-1, 1], [0, -1], [0, 1], [1, -1], [1, 0], [1, 1]];

  // Positionsgewichte für die KI-Bewertung (Ecken sehr wertvoll, Nachbarfelder
  // der Ecken gefährlich). Symmetrisches 8x8-Raster.
  const WEIGHTS = [
    [120, -20, 20, 5, 5, 20, -20, 120],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [20, -5, 15, 3, 3, 15, -5, 20],
    [5, -5, 3, 3, 3, 3, -5, 5],
    [5, -5, 3, 3, 3, 3, -5, 5],
    [20, -5, 15, 3, 3, 15, -5, 20],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [120, -20, 20, 5, 5, 20, -20, 120],
  ];

  const SETUP = "setup", PLAY = "play", OVER = "over";

  // Zeitbudget der KI-Suche pro Frame (ms)
  const AI_BUDGET_MS = 12;

  /** Liste der Steine, die ein Zug (r,c) für Spielerwert pv umdreht (leer=illegal). */
  function flipsFor(board, r, c, pv) {
    if (board[r][c] !== 0) return [];
    const opp = 3 - pv;
    const flips = [];
    for (const [dr, dc] of DIRS) {
      const line = [];
      let rr = r + dr, cc = c + dc;
      while (rr >= 0 && rr < N && cc >= 0 && cc < N && board[rr][cc] === opp) {
        line.push([rr, cc]);
        rr += dr;
        cc += dc;
      }
      if (line.length && rr >= 0 && rr < N && cc >= 0 && cc < N && board[rr][cc] === pv) {
        for (const p of line) flips.push(p);
      }
    }
    return flips;
  }

  /** Map "r,c" -> {r, c, flips} aller gültigen Züge für Spielerwert pv (Einfügereihenfolge wie Python). */
  function legalMoves(board, pv) {
    const moves = new Map();
    for (let r = 0; r < N; r++) {
      for (let c = 0; c < N; c++) {
        if (board[r][c] === 0) {
          const f = flipsFor(board, r, c, pv);
          if (f.length) moves.set(r + "," + c, { r, c, flips: f });
        }
      }
    }
    return moves;
  }

  /** Setzt den Stein und dreht die eingeschlossenen Steine um (in-place). */
  function apply(board, r, c, pv, flips) {
    board[r][c] = pv;
    for (const [fr, fc] of flips) board[fr][fc] = pv;
  }

  /** [Anzahl Spieler0-Steine, Anzahl Spieler1-Steine]. */
  function count(board) {
    let a = 0, b = 0;
    for (const row of board) {
      for (const v of row) {
        if (v === 1) a++;
        else if (v === 2) b++;
      }
    }
    return [a, b];
  }

  const copyBoard = (board) => board.map((row) => row.slice());

  class ReversiGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.diff = PG.clamp(Math.trunc(Number(this.opts.difficulty) || 0), 0, 2);

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
      this.cell = Math.floor(Math.min((this.width - 40) / N, (this.height - this.hudH - 24) / N));
      this.bw = N * this.cell;
      this.bh = N * this.cell;
      this.bx = Math.floor((this.width - this.bw) / 2);
      this.by = this.hudH + Math.max(8, Math.floor((this.height - this.hudH - this.bh) / 2));
      this.r = Math.floor(this.cell * 0.42);
    }

    newRound() {
      this.board = Array.from({ length: N }, () => new Array(N).fill(0));
      const mid = N / 2;
      this.board[mid - 1][mid - 1] = 2;
      this.board[mid][mid] = 2;
      this.board[mid - 1][mid] = 1;
      this.board[mid][mid - 1] = 1;
      this.player = this.starter; // 0 = Schwarz beginnt
      this.cursor = [mid, mid];
      this.moves = legalMoves(this.board, this.player + 1);
      this.winner = null;
      this.msg = null;
      this.msgT = 0;
      this.aiDelay = 0;
      this.aiJob = null;
      this.lastMove = null;
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
          this.diff = Number(k) - 1;
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

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.restart();
          } else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown") {
          this.restart();
        }
        return;
      }
      if (this.state !== PLAY) return;
      // Nur Spieler 0 (Schwarz) ist steuerbar.
      if (this.player === 1) return;
      if (ev.kind === "mousemove") {
        const rc = this.cellAt(ev.pos);
        if (rc) this.cursor = rc;
      } else if (ev.kind === "mousedown") {
        const rc = this.cellAt(ev.pos);
        if (rc) {
          this.cursor = rc;
          this.tryPlace(rc[0], rc[1]);
        }
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.isAction(k, "up") || k === "Up") {
          this.cursor[0] = PG.mod(this.cursor[0] - 1, N);
          this.playSound("move");
        } else if (this.isAction(k, "down") || k === "Down") {
          this.cursor[0] = PG.mod(this.cursor[0] + 1, N);
          this.playSound("move");
        } else if (this.isAction(k, "left") || k === "Left") {
          this.cursor[1] = PG.mod(this.cursor[1] - 1, N);
          this.playSound("move");
        } else if (this.isAction(k, "right") || k === "Right") {
          this.cursor[1] = PG.mod(this.cursor[1] + 1, N);
          this.playSound("move");
        } else if (this.isAction(k, "action") || k === "space" || k === "Return") {
          this.tryPlace(this.cursor[0], this.cursor[1]);
        }
      }
    }

    cellAt(pos) {
      const c = Math.floor((pos[0] - this.bx) / this.cell);
      const r = Math.floor((pos[1] - this.by) / this.cell);
      if (r >= 0 && r < N && c >= 0 && c < N) return [r, c];
      return null;
    }

    restart() {
      this.starter = 1 - this.starter;
      this.gameOver = false;
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    tryPlace(r, c) {
      const mv = this.moves.get(r + "," + c);
      if (!mv) {
        this.msg = t("rev.illegal");
        this.msgT = 1.0;
        this.playSound("click");
        return;
      }
      apply(this.board, r, c, this.player + 1, mv.flips);
      this.lastMove = [r, c];
      this.playSound("lock");
      this.advanceTurn();
    }

    /** Wechselt den Spieler; behandelt Passen und Spielende. */
    advanceTurn() {
      this.aiJob = null;
      const other = 1 - this.player;
      const otherMoves = legalMoves(this.board, other + 1);
      if (otherMoves.size) {
        this.player = other;
        this.moves = otherMoves;
        this.aiDelay = 0.35;
        return;
      }
      // Gegner muss passen - hat der aktuelle Spieler noch Züge?
      const selfMoves = legalMoves(this.board, this.player + 1);
      if (selfMoves.size) {
        this.moves = selfMoves;
        // Spieler 1 = KI, Spieler 0 = Mensch
        this.msg = other === 1 ? t("web.reversi.pass_ai") : t("web.reversi.pass_you");
        this.msgT = 1.4;
        this.playSound("select");
        this.aiDelay = 0.35;
        return;
      }
      // Keiner kann ziehen -> Partie zu Ende.
      this.ende();
    }

    ende() {
      const [a, b] = count(this.board);
      if (a > b) this.winner = 0;
      else if (b > a) this.winner = 1;
      else this.winner = null;
      this.state = OVER;
      if (this.winner !== null) {
        this.wins[this.winner] += 1;
        if (this.winner === 0) {
          this.score = this.wins[0];
          this.playSound("win");
          this.reportResult(true);
        } else {
          this.playSound("gameover");
          this.reportResult(false);
        }
      } else {
        this.playSound("select");
      }
      this.gameOver = true; // die App speichert den Score einmalig
    }

    // ===================================================== Spiellogik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state === PLAY && this.player === 1) {
        this.aiDelay -= dt;
        if (this.aiDelay <= 0) this.aiPlay();
      }
    }

    // ===================================================== KI
    aiPlay() {
      if (!this.aiJob) this.aiJob = this.aiStart();
      const move = this.aiStep(this.aiJob);
      if (move === undefined) return; // Suche läuft noch (nächster Frame)
      this.aiJob = null;
      if (move === null) {
        this.advanceTurn();
        return;
      }
      const [r, c] = move;
      const mv = this.moves.get(r + "," + c);
      const flips = mv ? mv.flips : flipsFor(this.board, r, c, 2);
      apply(this.board, r, c, 2, flips);
      this.lastMove = [r, c];
      this.playSound("lock");
      this.advanceTurn();
    }

    /** Beginnt die Zugwahl (Spielerwert 2) je nach Stärke. */
    aiStart() {
      const moves = [...this.moves.values()].map((m) => [m.r, m.c]);
      const job = { moves, i: 0, bestVal: -1e18, best: [], result: undefined };
      if (!moves.length) {
        job.result = null;
        return job;
      }
      // easy: meist zufällig, gelegentlich gierig.
      if (this.diff === 0 && PG.rand.random() < 0.6) {
        job.result = PG.rand.choice(moves);
        return job;
      }
      job.depth = DEPTHS[this.diff];
      return job;
    }

    /** Rechnet Wurzelzüge, bis das Frame-Budget verbraucht ist. undefined = noch nicht fertig. */
    aiStep(job) {
      if (job.result !== undefined) return job.result;
      const t0 = performance.now();
      while (job.i < job.moves.length) {
        const [r, c] = job.moves[job.i++];
        const nb = copyBoard(this.board);
        apply(nb, r, c, 2, flipsFor(nb, r, c, 2));
        const val = this.negamax(nb, job.depth - 1, -1e18, 1e18, 1); // 1 = Mensch am Zug
        if (val > job.bestVal) {
          job.bestVal = val;
          job.best = [[r, c]];
        } else if (val === job.bestVal) {
          job.best.push([r, c]);
        }
        if (performance.now() - t0 > AI_BUDGET_MS && job.i < job.moves.length) return undefined;
      }
      // medium: mit kleiner Wahrscheinlichkeit nicht optimal.
      if (this.diff === 1 && job.moves.length > 1 && PG.rand.random() < 0.25) {
        job.result = PG.rand.choice(job.moves);
      } else {
        job.result = PG.rand.choice(job.best);
      }
      return job.result;
    }

    /** turn: 0 = KI (Wert 2) am Zug, 1 = Mensch (Wert 1). Bewertung aus KI-Sicht. */
    negamax(board, depth, alpha, beta, turn) {
      if (depth === 0) return this.evaluate(board);
      const pv = turn === 0 ? 2 : 1;
      const moves = legalMoves(board, pv);
      if (!moves.size) {
        // Passen - kann der andere ziehen?
        if (!legalMoves(board, 3 - pv).size) return this.terminal(board);
        return this.negamax(board, depth - 1, alpha, beta, 1 - turn);
      }
      // Zugreihenfolge: hoch bewertete Felder zuerst (Alpha-Beta profitiert).
      const ordered = [...moves.values()].sort((m1, m2) => WEIGHTS[m2.r][m2.c] - WEIGHTS[m1.r][m1.c]);
      if (turn === 0) {
        // maximierend (KI)
        let best = -1e18;
        for (const m of ordered) {
          const nb = copyBoard(board);
          apply(nb, m.r, m.c, pv, m.flips);
          best = Math.max(best, this.negamax(nb, depth - 1, alpha, beta, 1));
          alpha = Math.max(alpha, best);
          if (alpha >= beta) break;
        }
        return best;
      }
      // minimierend (Mensch)
      let best = 1e18;
      for (const m of ordered) {
        const nb = copyBoard(board);
        apply(nb, m.r, m.c, pv, m.flips);
        best = Math.min(best, this.negamax(nb, depth - 1, alpha, beta, 0));
        beta = Math.min(beta, best);
        if (alpha >= beta) break;
      }
      return best;
    }

    /** Endstellungs-Bewertung: klarer Sieg/Verlust dominiert. */
    terminal(board) {
      const [a, b] = count(board); // a = Mensch(1), b = KI(2)
      if (b > a) return 100000 + (b - a);
      if (a > b) return -100000 - (a - b);
      return 0;
    }

    /** Positionsgewichtung + Mobilität, aus Sicht der KI (Wert 2). */
    evaluate(board) {
      let score = 0;
      for (let r = 0; r < N; r++) {
        for (let c = 0; c < N; c++) {
          const v = board[r][c];
          if (v === 2) score += WEIGHTS[r][c];
          else if (v === 1) score -= WEIGHTS[r][c];
        }
      }
      const myMob = legalMoves(board, 2).size;
      const opMob = legalMoves(board, 1).size;
      if (myMob + opMob) score += Math.trunc(((8 * (myMob - opMob)) / (myMob + opMob + 1)) * 10);
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

    discHi(p) {
      return p === 0 ? COL_P1_HI : COL_P2_HI;
    }

    drawBoard(ctx) {
      // Rahmenplatte
      draw.rect(ctx, COL_PLATE, [this.bx - 8, this.by - 8, this.bw + 16, this.bh + 16], 0, 10);

      // Felder (Schachbrett-Grün) + Rasterlinien
      for (let r = 0; r < N; r++) {
        for (let c = 0; c < N; c++) {
          const base = (r + c) % 2 === 0 ? COL_BOARD : COL_BOARD_DARK;
          draw.rect(ctx, base, [this.bx + c * this.cell, this.by + r * this.cell, this.cell, this.cell]);
        }
      }
      for (let i = 0; i <= N; i++) {
        const gx = this.bx + i * this.cell + 0.5;
        const gy = this.by + i * this.cell + 0.5;
        draw.line(ctx, COL_GRID, [gx, this.by], [gx, this.by + this.bh]);
        draw.line(ctx, COL_GRID, [this.bx, gy], [this.bx + this.bw, gy]);
      }

      const humanTurn = this.state === PLAY && this.player === 0;
      const half = Math.floor(this.cell / 2);

      // Zughinweise (kleine Punkte) für den steuerbaren Spieler
      if (humanTurn) {
        for (const m of this.moves.values()) {
          draw.circle(ctx, COL_HINT, [this.bx + m.c * this.cell + half, this.by + m.r * this.cell + half], Math.max(3, Math.floor(this.r / 5)));
        }
      }

      // Steine
      for (let r = 0; r < N; r++) {
        for (let c = 0; c < N; c++) {
          const v = this.board[r][c];
          if (v === 0) continue;
          const cx = this.bx + c * this.cell + half;
          const cy = this.by + r * this.cell + half;
          draw.circle(ctx, [8, 14, 10], [cx, cy + 2], this.r);
          draw.circle(ctx, this.discColor(v - 1), [cx, cy], this.r);
          const o = Math.floor(this.r / 3);
          draw.circle(ctx, this.discHi(v - 1), [cx - o, cy - o], Math.max(2, Math.floor(this.r / 4)));
        }
      }

      // Letzter Zug markieren
      if (this.lastMove) {
        const [r, c] = this.lastMove;
        draw.rect(ctx, this.accent, [this.bx + c * this.cell, this.by + r * this.cell, this.cell, this.cell], 2);
      }

      // Auswahlrahmen (Tastatur/Maus)
      if (humanTurn) {
        const [r, c] = this.cursor;
        const x = this.bx + c * this.cell;
        const y = this.by + r * this.cell;
        const k = 0.5 + (0.5 * Math.abs((ui.ticks() % 900) - 450)) / 450;
        const g = Math.floor(120 + 120 * k);
        draw.rect(ctx, [g, g, g], [x + 1, y + 1, this.cell - 2, this.cell - 2], 2);
      }
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = Math.floor(this.hudH / 2);
      const [a, b] = count(this.board);
      // Stein-Zähler links (Schwarz) und rechts (Weiß)
      draw.circle(ctx, COL_P1, [18, cy], 9);
      draw.circle(ctx, COL_P1_HI, [15, cy - 3], 3);
      draw.circle(ctx, ui.BORDER_LIGHT, [18, cy], 9, 1);
      ui.text(ctx, String(a), 32, cy, this.small, ui.TEXT, "midleft");
      draw.circle(ctx, COL_P2, [this.width - 18, cy], 9);
      draw.circle(ctx, ui.BORDER_LIGHT, [this.width - 18, cy], 9, 1);
      ui.text(ctx, String(b), this.width - 32, cy, this.small, ui.TEXT, "midright");

      if (this.state === PLAY) {
        const mid = this.player === 1 ? t("rev.ai_thinks") : t("rev.your_turn");
        ui.text(ctx, mid, this.width / 2, cy, this.small, this.accent, "center");
      }
      if (this.msg) {
        ui.text(ctx, this.msg, this.width / 2, this.hudH + 12, this.tiny, ui.GOLD, "center");
      }
    }

    drawOver(ctx) {
      const hh = this.huge.height;
      const sh = this.small.height;
      const th = this.tiny.height;
      const bandH = hh + sh + th + 40;
      const y = Math.floor(this.height / 2 - bandH / 2);
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], [0, y, this.width, bandH]);
      draw.line(ctx, this.accent, [0, y], [this.width, y], 2);
      draw.line(ctx, this.accent, [0, y + bandH - 1], [this.width, y + bandH - 1], 2);
      const cx = this.width / 2;
      const [a, b] = count(this.board);
      let yy = y + 12;
      if (this.winner === null) {
        ui.text(ctx, t("common.draw"), cx, yy, this.huge, ui.TEXT_DIM, "midtop");
      } else {
        const key = this.winner === 0 ? "rev.win_you" : "rev.win_ai";
        ui.text(ctx, t(key), cx, yy, this.huge, this.winner === 0 ? this.accent : ui.RED, "midtop");
      }
      yy += hh + 8;
      ui.text(ctx, `${a} : ${b}`, cx, yy, this.small, ui.TEXT, "midtop");
      yy += sh + 6;
      ui.text(ctx, t("rev.new_round"), cx, yy, this.tiny, ui.TEXT_DIM, "midtop");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, PG.gameName("ReversiGame").toUpperCase(), cx, Math.floor(this.height * 0.14), this.huge, this.accent, "center");
      ui.text(ctx, t("web.reversi.subtitle"), cx, Math.floor(this.height * 0.21), this.small, ui.TEXT_DIM, "center");
      this.diffRects.forEach((rc, i) => {
        const on = i === this.diff;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 10);
        draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 10);
        ui.text(ctx, t("rev.diff." + DIFFS[i]), rc.x + 18, rc.centery, this.font, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
      });
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("rev.setup_hint"), cx, this.height - 16, this.tiny, ui.TEXT_FAINT, "center");
    }
  }

  PG.register(ReversiGame, {
    id: "ReversiGame",
    key: "reversi",
    name: "Reversi",
    settingsKey: "reversi",
    defaults: { difficulty: 1 },
  });
})();
