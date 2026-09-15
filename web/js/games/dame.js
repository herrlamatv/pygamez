/*
 * dame.js - Dame / Checkers (Port von games/dame.py)
 * ===================================================
 * Drei Regelwerke, 1 Spieler gegen KI (Web-Version: nur Einzelspieler).
 *
 * Wählbare Varianten (im Setup-Screen):
 * - Deutsche Dame (8x8): 12 Steine je Seite; Männer ziehen 1 diagonal vorwärts,
 *   SCHLAGEN aber vor- UND rückwärts; die Dame FLIEGT beliebig weit diagonal.
 *   Schlagzwang, Mehrfachschlag - das Maximum muss aber NICHT genommen werden.
 * - Internationale Dame (10x10): 20 Steine je Seite; Männer schlagen vor/rückwärts,
 *   fliegende Damen. Schlagzwang MIT Maximum-Regel (längste Schlagfolge ist Pflicht).
 * - Checkers 8x8 (englisch): Männer schlagen NUR vorwärts, die Dame (King) zieht
 *   nur EIN Feld (nicht fliegend). Schlagzwang, kein Maximum.
 *
 * Gemeinsam: Steine stehen auf den dunklen Feldern. Wer keinen Zug mehr hat (kein
 * Stein oder eingeschlossen), verliert. Der Mehrfachschlag muss vollständig
 * ausgeführt werden (Schlagzwang). Ein Mann, der beim Schlagen die letzte Reihe
 * erreicht, wird zur Dame und die Schlagfolge endet dort.
 *
 * KI: Minimax + Alpha-Beta (Material + Vormarsch), drei Stärken. Die Suche wird
 * Wurzelzug für Wurzelzug über mehrere Frames verteilt, damit die Oberfläche
 * nicht einfriert ("KI denkt ..." bleibt sichtbar).
 * Punkte (Highscore) = Siege gegen die KI in dieser Sitzung.
 *
 * Steuerung: Stein anklicken, dann Zielfeld(er). Mehrfachsprung Schritt für Schritt
 * anklicken. Tastatur: Pfeile/WASD bewegen den Rahmen, Leertaste/Enter wählt.
 * Nach Rundenende: Enter = neue Runde, S = Setup (Variantenwahl).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ---- Feldkodierung
  const EMPTY = 0;
  const P0_MAN = 1, P0_KING = 2; // Spieler 0 (unten, hell) - zieht nach oben (row -)
  const P1_MAN = 3, P1_KING = 4; // Spieler 1 (oben, dunkel) - zieht nach unten (row +)

  const DIRS4 = [[-1, -1], [-1, 1], [1, -1], [1, 1]];

  // ---- Brett-Identitätsfarben (generische UI-Farben liefert die ui-Palette)
  const COL_LIGHT = [222, 196, 150]; // helle (unbespielte) Felder
  const COL_DARK = [120, 84, 52]; // dunkle (bespielte) Felder
  const COL_PLATE = [40, 28, 20]; // Brettrahmen
  const COL_P0 = [236, 230, 214]; // Steine Spieler 0 (creme)
  const COL_P0_HI = [255, 255, 250];
  const COL_P1 = [188, 60, 60]; // Steine Spieler 1 (rot)
  const COL_P1_HI = [232, 120, 120];
  const COL_KING = [245, 205, 90]; // Krone/Ring der Dame
  const COL_HINT = [250, 236, 150]; // Auswahl-/Zughinweise auf dem Holzbrett
  const COL_CAP = [232, 90, 80]; // Schlagzwang-/Schlag-Markierungen

  const VARIANTS = ["german", "international", "checkers"];
  const DIFFS = ["easy", "medium", "hard"];

  const SETUP = "setup", PLAY = "play", OVER = "over";

  // Zeitbudget der KI-Suche pro Frame (ms)
  const AI_BUDGET_MS = 12;

  function owner(code) {
    if (code === EMPTY) return null;
    return code <= P0_KING ? 0 : 1;
  }

  function isKing(code) {
    return code === P0_KING || code === P1_KING;
  }

  function makeCode(side, king) {
    if (side === 0) return king ? P0_KING : P0_MAN;
    return king ? P1_KING : P1_MAN;
  }

  /** Regel-Flags je Variante. */
  function variantFlags(variant) {
    if (variant === "german") return { size: 8, menBack: true, flying: true, maximum: false };
    if (variant === "international") return { size: 10, menBack: true, flying: true, maximum: true };
    // checkers (englisch)
    return { size: 8, menBack: false, flying: false, maximum: false };
  }

  function initialBoard(flags) {
    const size = flags.size;
    const rows = size === 8 ? 3 : 4;
    const b = [];
    for (let r = 0; r < size; r++) {
      b.push(new Array(size).fill(EMPTY));
      for (let c = 0; c < size; c++) {
        if ((r + c) % 2 !== 1) continue; // nur dunkle Felder
        if (r < rows) b[r][c] = P1_MAN; // oben = Spieler 1
        else if (r >= size - rows) b[r][c] = P0_MAN; // unten = Spieler 0
      }
    }
    return b;
  }

  function inside(r, c, size) {
    return r >= 0 && r < size && c >= 0 && c < size;
  }

  function backRank(row, side, size) {
    return side === 0 ? row === 0 : row === size - 1;
  }

  function copyBoard(b) {
    return b.map((row) => row.slice());
  }

  function hasPos(list, r, c) {
    for (const p of list) if (p[0] === r && p[1] === c) return true;
    return false;
  }

  // ---------------------------------------------------------------------------
  //  Zug- und Schlaggenerierung
  // ---------------------------------------------------------------------------

  /** Ein-Schritt-Schläge von (r,c) auf Brett b. Liefert [[capPos, landPos]]. */
  function singleJumps(b, r, c, side, king, captured, flags) {
    const size = flags.size;
    const opp = 1 - side;
    const out = [];
    if (!king) {
      const dirs = flags.menBack ? DIRS4 : side === 0 ? [[-1, -1], [-1, 1]] : [[1, -1], [1, 1]];
      for (const [dr, dc] of dirs) {
        const mr = r + dr, mc = c + dc, lr = r + 2 * dr, lc = c + 2 * dc;
        if (inside(lr, lc, size) && owner(b[mr][mc]) === opp && !hasPos(captured, mr, mc) && b[lr][lc] === EMPTY) {
          out.push([[mr, mc], [lr, lc]]);
        }
      }
      return out;
    }
    // Dame
    if (flags.flying) {
      for (const [dr, dc] of DIRS4) {
        let i = 1;
        while (inside(r + dr * i, c + dc * i, size) && b[r + dr * i][c + dc * i] === EMPTY) i++;
        const er = r + dr * i, ec = c + dc * i;
        if (inside(er, ec, size) && owner(b[er][ec]) === opp && !hasPos(captured, er, ec)) {
          let j = i + 1;
          while (inside(r + dr * j, c + dc * j, size) && b[r + dr * j][c + dc * j] === EMPTY) {
            out.push([[er, ec], [r + dr * j, c + dc * j]]);
            j++;
          }
        }
      }
    } else {
      for (const [dr, dc] of DIRS4) {
        const mr = r + dr, mc = c + dc, lr = r + 2 * dr, lc = c + 2 * dc;
        if (inside(lr, lc, size) && owner(b[mr][mc]) === opp && !hasPos(captured, mr, mc) && b[lr][lc] === EMPTY) {
          out.push([[mr, mc], [lr, lc]]);
        }
      }
    }
    return out;
  }

  /** Alle vollständigen Schlagfolgen des Steins auf (r,c). Liefert [[path, caps]]. */
  function captureSequences(board, r, c, flags) {
    const code = board[r][c];
    const side = owner(code);
    const king0 = isKing(code);
    const size = flags.size;
    const results = [];

    const rec = (b, cr, cc, curKing, captured, path) => {
      const jumps = singleJumps(b, cr, cc, side, curKing, captured, flags);
      if (!jumps.length) return false;
      for (const [cap, [lr, lc]] of jumps) {
        const nb = copyBoard(b);
        nb[cr][cc] = EMPTY;
        const promoted = !curKing && backRank(lr, side, size);
        const nk = curKing || promoted;
        nb[lr][lc] = makeCode(side, nk);
        const ncap = captured.concat([cap]);
        const npath = path.concat([[lr, lc]]);
        if (promoted) results.push([npath, ncap]); // Mann wird Dame -> Ende
        else if (!rec(nb, lr, lc, nk, ncap, npath)) results.push([npath, ncap]); // keine Fortsetzung
      }
      return true;
    };

    rec(copyBoard(board), r, c, king0, [], [[r, c]]);
    return results;
  }

  function simpleMoves(board, r, c, flags) {
    const code = board[r][c];
    const side = owner(code);
    const king = isKing(code);
    const size = flags.size;
    const out = [];
    if (!king) {
      const fdir = side === 0 ? -1 : 1;
      for (const dc of [-1, 1]) {
        const lr = r + fdir, lc = c + dc;
        if (inside(lr, lc, size) && board[lr][lc] === EMPTY) out.push([[r, c], [lr, lc]]);
      }
    } else if (flags.flying) {
      for (const [dr, dc] of DIRS4) {
        let i = 1;
        while (inside(r + dr * i, c + dc * i, size) && board[r + dr * i][c + dc * i] === EMPTY) {
          out.push([[r, c], [r + dr * i, c + dc * i]]);
          i++;
        }
      }
    } else {
      for (const [dr, dc] of DIRS4) {
        const lr = r + dr, lc = c + dc;
        if (inside(lr, lc, size) && board[lr][lc] === EMPTY) out.push([[r, c], [lr, lc]]);
      }
    }
    return out;
  }

  /**
   * [Liste Züge, forced] - bei Schlagzwang nur Schlagzüge.
   * Ein Zug: {path: [[r,c],...], caps: [[r,c],...], start, end}.
   */
  function legalMoves(board, side, flags) {
    let caps = [];
    const size = flags.size;
    for (let r = 0; r < size; r++) {
      for (let c = 0; c < size; c++) {
        if (owner(board[r][c]) === side) {
          for (const [path, capset] of captureSequences(board, r, c, flags)) {
            caps.push({ path, caps: capset, start: path[0], end: path[path.length - 1] });
          }
        }
      }
    }
    if (caps.length) {
      if (flags.maximum) {
        let m = 0;
        for (const x of caps) m = Math.max(m, x.caps.length);
        caps = caps.filter((x) => x.caps.length === m);
      }
      return [caps, true];
    }
    const simples = [];
    for (let r = 0; r < size; r++) {
      for (let c = 0; c < size; c++) {
        if (owner(board[r][c]) === side) {
          for (const [st, en] of simpleMoves(board, r, c, flags)) {
            simples.push({ path: [st, en], caps: [], start: st, end: en });
          }
        }
      }
    }
    return [simples, false];
  }

  function applyMove(board, move, flags) {
    const b = copyBoard(board);
    const [sr, sc] = move.start;
    const [er, ec] = move.end;
    const code = b[sr][sc];
    const side = owner(code);
    let king = isKing(code);
    b[sr][sc] = EMPTY;
    for (const [cr, cc] of move.caps) b[cr][cc] = EMPTY;
    if (!king && backRank(er, side, flags.size)) king = true;
    b[er][ec] = makeCode(side, king);
    return b;
  }

  function countPieces(board) {
    let a = 0, bc = 0;
    for (const row of board) {
      for (const v of row) {
        const o = owner(v);
        if (o === 0) a++;
        else if (o === 1) bc++;
      }
    }
    return [a, bc];
  }

  const samePos = (p, q) => p[0] === q[0] && p[1] === q[1];

  class DameGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      this.variant = this.opts.variant;
      if (!VARIANTS.includes(this.variant)) this.variant = "german";
      this.diff = Math.max(0, Math.min(2, parseInt(this.opts.difficulty, 10) || 0));

      this.makeFonts();
      this.wins = [0, 0];
      this.starter = 0;
      this.buildSetupLayout();
      this.newRound();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größe abhängig von der Fensterhöhe. */
    makeFonts() {
      this.small = ui.font(Math.max(14, Math.floor(this.height / 32)));
      this.tiny = ui.font(Math.max(12, Math.floor(this.height / 40)));
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    layout() {
      const n = this.flags.size;
      this.hudH = 46;
      this.cell = Math.floor(Math.min((this.width - 40) / n, (this.height - this.hudH - 24) / n));
      this.bw = n * this.cell;
      this.bh = n * this.cell;
      this.bx = Math.floor((this.width - this.bw) / 2);
      this.by = this.hudH + Math.max(8, Math.floor((this.height - this.hudH - this.bh) / 2));
      this.pr = Math.floor(this.cell * 0.38);
    }

    newRound() {
      this.flags = variantFlags(this.variant);
      const n = this.flags.size;
      this.depths = n === 8 ? [1, 3, 4] : [1, 2, 3];
      this.board = initialBoard(this.flags);
      this.player = this.starter;
      const h = Math.floor(n / 2);
      this.cursor = [h, h % 2 === 0 ? h - 1 : h];
      [this.moves, this.forced] = legalMoves(this.board, this.player, this.flags);
      this.sel = null;
      this.partial = [];
      this.stepOptions = [];
      this.lastMove = null;
      this.winner = null;
      this.aiDelay = 0.0;
      this.aiJob = null;
      this.layout();
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(380, this.width - 60);
      const y0 = Math.floor(this.height * 0.26);
      this.varRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 50, bw, 42));
      const y1 = y0 + 3 * 50 + 18;
      const dw = Math.min(120, Math.floor((bw - 20) / 3));
      const gap = Math.floor((bw - 3 * dw) / 2);
      this.diffRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2) + i * (dw + gap), y1, dw, 40));
      this.startRect = new PG.Rect(cx - 95, y1 + 60, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") this.pickVariant(Number(k) - 1);
        else if (k === "Up" || k === "w" || k === "W") this.pickVariant(PG.mod(VARIANTS.indexOf(this.variant) - 1, 3));
        else if (k === "Down" || k === "s" || k === "S") this.pickVariant(PG.mod(VARIANTS.indexOf(this.variant) + 1, 3));
        else if (k === "Left" || k === "a" || k === "A") {
          this.diff = PG.mod(this.diff - 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "Right" || k === "d" || k === "D") {
          this.diff = PG.mod(this.diff + 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.startGame();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.varRects.length; i++) {
          if (this.varRects[i].collidepoint(ev.pos)) {
            this.pickVariant(i);
            return;
          }
        }
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diff = i;
            this.saveSetting("difficulty", i);
            this.playSound("click");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startGame();
      }
    }

    pickVariant(i) {
      this.variant = VARIANTS[i];
      this.saveSetting("variant", this.variant);
      this.playSound("click");
    }

    startGame() {
      this.newRound();
      this.state = PLAY;
      this.playSound("select");
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
      if (this.player === 1) return; // KI ist am Zug
      if (ev.kind === "mousemove") {
        const rc = this.cellAt(ev.pos);
        if (rc) this.cursor = rc;
      } else if (ev.kind === "mousedown") {
        const rc = this.cellAt(ev.pos);
        if (rc) {
          this.cursor = rc.slice();
          this.choose(rc);
        }
      } else if (ev.kind === "keydown") {
        const n = this.flags.size;
        const k = ev.key;
        if (this.isAction(k, "up") || k === "Up") this.cursor[0] = PG.mod(this.cursor[0] - 1, n);
        else if (this.isAction(k, "down") || k === "Down") this.cursor[0] = PG.mod(this.cursor[0] + 1, n);
        else if (this.isAction(k, "left") || k === "Left") this.cursor[1] = PG.mod(this.cursor[1] - 1, n);
        else if (this.isAction(k, "right") || k === "Right") this.cursor[1] = PG.mod(this.cursor[1] + 1, n);
        else if (this.isAction(k, "action") || k === "space" || k === "Return") {
          this.choose([this.cursor[0], this.cursor[1]]);
          return;
        } else return;
        this.playSound("move");
      }
    }

    cellAt(pos) {
      const c = Math.floor((pos[0] - this.bx) / this.cell);
      const r = Math.floor((pos[1] - this.by) / this.cell);
      if (inside(r, c, this.flags.size)) return [r, c];
      return null;
    }

    isStartSquare(rc) {
      return this.moves.some((m) => samePos(m.start, rc));
    }

    startSquares() {
      const out = [];
      for (const m of this.moves) if (!hasPos(out, m.start[0], m.start[1])) out.push(m.start);
      return out;
    }

    /** true, wenn m.path mit dem bisherigen Teilpfad beginnt. */
    pathMatches(m, depth) {
      if (m.path.length <= depth) return false;
      for (let i = 0; i < depth; i++) if (!samePos(m.path[i], this.partial[i])) return false;
      return true;
    }

    /** Zentraler Auswahl-Handler für Maus und Tastatur (Mehrfachsprung). */
    choose(rc) {
      if (this.sel === null) {
        if (this.isStartSquare(rc)) {
          this.sel = rc;
          this.partial = [rc];
          this.recomputeOptions();
          this.playSound("select");
        } else {
          this.playSound("click");
        }
        return;
      }
      // Es ist ein Stein gewählt -> nächster Schritt?
      const depth = this.partial.length;
      const cand = this.moves.filter((m) => samePos(m.start, this.sel) && this.pathMatches(m, depth) && samePos(m.path[depth], rc));
      if (!cand.length) {
        // Umwahl oder Abwahl
        if (samePos(rc, this.sel) && depth === 1) {
          this.sel = null;
          this.partial = [];
          this.stepOptions = [];
          this.playSound("click");
        } else if (this.isStartSquare(rc)) {
          this.sel = rc;
          this.partial = [rc];
          this.recomputeOptions();
          this.playSound("select");
        } else {
          this.playSound("click");
        }
        return;
      }
      this.partial.push(rc);
      const complete = cand.filter((m) => m.path.length === this.partial.length);
      if (complete.length) {
        this.doMove(complete[0]);
      } else {
        this.recomputeOptions();
        this.playSound("lock");
      }
    }

    recomputeOptions() {
      const depth = this.partial.length;
      this.stepOptions = [];
      for (const m of this.moves) {
        if (samePos(m.start, this.sel) && this.pathMatches(m, depth)) {
          const p = m.path[depth];
          if (!hasPos(this.stepOptions, p[0], p[1])) this.stepOptions.push(p);
        }
      }
    }

    doMove(move) {
      const promoted = this.isPromotion(move);
      this.board = applyMove(this.board, move, this.flags);
      this.lastMove = move;
      this.sel = null;
      this.partial = [];
      this.stepOptions = [];
      this.playSound(move.caps.length ? "lock" : "move");
      if (promoted) this.playSound("powerup");
      this.advanceTurn();
    }

    isPromotion(move) {
      const [sr, sc] = move.start;
      const code = this.board[sr][sc];
      if (isKing(code)) return false;
      return backRank(move.end[0], owner(code), this.flags.size);
    }

    advanceTurn() {
      const other = 1 - this.player;
      const [moves, forced] = legalMoves(this.board, other, this.flags);
      if (!moves.length) {
        this.ende(this.player); // Gegner kann nicht ziehen
        return;
      }
      this.player = other;
      this.moves = moves;
      this.forced = forced;
      this.sel = null;
      this.partial = [];
      this.stepOptions = [];
      if (this.player === 1) this.aiDelay = 0.45;
    }

    restart() {
      this.starter = 1 - this.starter;
      this.gameOver = false;
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    ende(winner) {
      this.winner = winner;
      this.wins[winner] += 1;
      this.state = OVER;
      this.aiJob = null;
      if (winner === 0) {
        this.score = this.wins[0];
        this.playSound("win");
        this.reportResult(true);
      } else {
        this.playSound("gameover");
        this.reportResult(false);
      }
      this.gameOver = true; // die App sichert den Score
    }

    // ===================================================== KI
    update(dt) {
      if (this.state === PLAY && this.player === 1) {
        this.aiDelay -= dt;
        if (this.aiDelay <= 0) this.aiStep();
      }
    }

    /** Ein Frame KI-Arbeit: Suche fortsetzen und ggf. den Zug ausführen. */
    aiStep() {
      if (!this.aiJob) {
        const moves = this.moves;
        if (!moves.length) {
          this.ende(0);
          return;
        }
        if (this.diff === 0 && PG.rand.random() < 0.55) {
          this.aiPlay(PG.rand.choice(moves));
          return;
        }
        this.aiJob = { idx: 0, best: -1e18, bestMoves: [], depth: this.depths[this.diff] };
      }
      const job = this.aiJob;
      const moves = this.moves;
      const t0 = performance.now();
      // Wurzelzüge nacheinander bewerten, bis das Frame-Budget aufgebraucht ist
      while (job.idx < moves.length) {
        const m = moves[job.idx++];
        const nb = applyMove(this.board, m, this.flags);
        const val = this.search(nb, 0, job.depth - 1, -1e18, 1e18);
        if (val > job.best) {
          job.best = val;
          job.bestMoves = [m];
        } else if (val === job.best) {
          job.bestMoves.push(m);
        }
        if (performance.now() - t0 > AI_BUDGET_MS) break;
      }
      if (job.idx < moves.length) return; // nächster Frame rechnet weiter
      this.aiJob = null;
      let move;
      if (this.diff === 1 && moves.length > 1 && PG.rand.random() < 0.2) move = PG.rand.choice(moves);
      else move = PG.rand.choice(job.bestMoves);
      this.aiPlay(move);
    }

    aiPlay(move) {
      const promoted = this.isPromotion(move);
      this.board = applyMove(this.board, move, this.flags);
      this.lastMove = move;
      this.playSound(move.caps.length ? "lock" : "move");
      if (promoted) this.playSound("powerup");
      this.advanceTurn();
    }

    /** Minimax aus Sicht der KI (Spieler 1 maximiert). */
    search(board, side, depth, alpha, beta) {
      const [moves] = legalMoves(board, side, this.flags);
      if (!moves.length) return side === 1 ? -1e9 : 1e9; // wer nicht ziehen kann, verliert
      if (depth <= 0) return this.evaluate(board);
      if (side === 1) {
        let best = -1e18;
        for (const m of moves) {
          const nb = applyMove(board, m, this.flags);
          best = Math.max(best, this.search(nb, 0, depth - 1, alpha, beta));
          alpha = Math.max(alpha, best);
          if (alpha >= beta) break;
        }
        return best;
      }
      let best = 1e18;
      for (const m of moves) {
        const nb = applyMove(board, m, this.flags);
        best = Math.min(best, this.search(nb, 1, depth - 1, alpha, beta));
        beta = Math.min(beta, best);
        if (alpha >= beta) break;
      }
      return best;
    }

    evaluate(board) {
      const size = this.flags.size;
      let score = 0;
      for (let r = 0; r < size; r++) {
        for (let c = 0; c < size; c++) {
          const code = board[r][c];
          if (code === EMPTY) continue;
          const o = owner(code);
          let val = isKing(code) ? 300 : 100;
          if (!isKing(code)) val += o === 1 ? r * 4 : (size - 1 - r) * 4;
          if (c === 0 || c === size - 1) val += 6;
          score += o === 1 ? val : -val;
        }
      }
      return score;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawHud(ctx);
      this.drawBoard(ctx);
      // "Schlagzwang!" erst NACH dem Brett - sonst verdeckt ihn die Brettplatte
      if (this.state === PLAY && this.forced) this.drawNote(ctx, t("dame.must_capture"), ui.RED);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawBoard(ctx) {
      const n = this.flags.size;
      const cell = this.cell;
      const plate = [this.bx - 8, this.by - 8, this.bw + 16, this.bh + 16];
      draw.rect(ctx, COL_PLATE, plate, 0, 10);
      draw.rect(ctx, ui.mix(COL_PLATE, this.accent, 0.45), plate, 1, 10);
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          const dark = (r + c) % 2 === 1;
          draw.rect(ctx, dark ? COL_DARK : COL_LIGHT, [this.bx + c * cell, this.by + r * cell, cell, cell]);
        }
      }

      const humanTurn = this.state === PLAY && this.player === 0;

      // Geschlagene Steine des letzten Zuges dezent markieren (schon entfernt)
      if (this.lastMove) {
        for (const [cr, cc] of this.lastMove.caps) {
          const x = this.bx + cc * cell;
          const y = this.by + cr * cell;
          draw.line(ctx, COL_CAP, [x + 6, y + 6], [x + cell - 6, y + cell - 6], 2);
          draw.line(ctx, COL_CAP, [x + cell - 6, y + 6], [x + 6, y + cell - 6], 2);
        }
        const lastCol = ui.mix(COL_DARK, this.accent, 0.55 + 0.35 * ui.pulse(1.6, 0.0, 1.0));
        for (const pos of [this.lastMove.start, this.lastMove.end]) {
          draw.rect(ctx, lastCol, [this.bx + pos[1] * cell, this.by + pos[0] * cell, cell, cell], 2);
        }
      }

      // wählbare Steine (Schlagzwang: nur diese) hervorheben
      if (humanTurn && this.sel === null) {
        const col = this.forced ? COL_CAP : COL_HINT;
        for (const [sr, sc] of this.startSquares()) {
          const cx = this.bx + sc * cell + Math.floor(cell / 2);
          const cy = this.by + sr * cell + Math.floor(cell / 2);
          draw.circle(ctx, col, [cx, cy], this.pr + 3, 2);
        }
      }

      // gewählter Stein + mögliche Schritte
      if (humanTurn && this.sel !== null) {
        const cur = this.partial[this.partial.length - 1];
        draw.rect(ctx, COL_HINT, [this.bx + cur[1] * cell, this.by + cur[0] * cell, cell, cell], 3);
        for (const [dr, dc] of this.stepOptions) {
          const cx = this.bx + dc * cell + Math.floor(cell / 2);
          const cy = this.by + dr * cell + Math.floor(cell / 2);
          draw.circle(ctx, COL_HINT, [cx, cy], Math.max(4, Math.floor(this.pr / 3)));
        }
      }

      // Steine
      for (let r = 0; r < n; r++) {
        for (let c = 0; c < n; c++) {
          const v = this.board[r][c];
          if (v !== EMPTY) this.drawPiece(ctx, r, c, v);
        }
      }

      // Cursor
      if (humanTurn) {
        const [r, c] = this.cursor;
        const k = ui.pulse(2.2, 0.0, 1.0);
        const g = Math.floor(150 + 100 * k);
        draw.rect(ctx, [g, g, g], [this.bx + c * cell + 1, this.by + r * cell + 1, cell - 2, cell - 2], 2);
      }
    }

    drawPiece(ctx, r, c, v) {
      const pr = this.pr;
      const cx = this.bx + c * this.cell + Math.floor(this.cell / 2);
      const cy = this.by + r * this.cell + Math.floor(this.cell / 2);
      const o = owner(v);
      const base = o === 0 ? COL_P0 : COL_P1;
      const hi = o === 0 ? COL_P0_HI : COL_P1_HI;
      const dark = base.map((x) => Math.floor(x * 0.55));
      draw.circle(ctx, [10, 8, 6], [cx, cy + 2], pr);
      draw.circle(ctx, dark, [cx, cy], pr);
      draw.circle(ctx, base, [cx, cy], Math.floor(pr * 0.86));
      draw.circle(ctx, hi, [cx - Math.floor(pr / 3), cy - Math.floor(pr / 3)], Math.max(2, Math.floor(pr / 5)));
      if (isKing(v)) {
        draw.circle(ctx, COL_KING, [cx, cy], Math.floor(pr * 0.5), 2);
        // kleine Krone
        const kr = Math.floor(pr * 0.34);
        const h = Math.floor(kr / 2);
        const pts = [[cx - kr, cy + h], [cx - kr, cy - h], [cx - h, cy], [cx, cy - kr], [cx + h, cy], [cx + kr, cy - h], [cx + kr, cy + h]];
        draw.polygon(ctx, COL_KING, pts);
      }
    }

    drawHud(ctx) {
      const panel = new PG.Rect(8, 6, this.width - 16, this.hudH - 10);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      const cy = panel.centery;
      const [a, b] = countPieces(this.board);
      draw.circle(ctx, COL_P0, [panel.x + 18, cy], 9);
      draw.circle(ctx, ui.BORDER_LIGHT, [panel.x + 18, cy], 9, 1);
      ui.text(ctx, String(a), panel.x + 34, cy, this.small, ui.TEXT, "midleft");
      draw.circle(ctx, COL_P1, [panel.right - 18, cy], 9);
      draw.circle(ctx, ui.BORDER_LIGHT, [panel.right - 18, cy], 9, 1);
      ui.text(ctx, String(b), panel.right - 34, cy, this.small, ui.TEXT, "midright");

      if (this.state === PLAY) {
        const mid = this.player === 1 ? t("dame.ai_thinks") : t("dame.your_turn");
        ui.text(ctx, mid, this.width / 2, cy, this.small, this.accent, "center");
      }
    }

    /**
     * Hinweiszeile direkt unter dem HUD. Liegt über dem oberen Brettrand, daher
     * nach dem Brett zeichnen und mit kleinem Hintergrund lesbar halten (in
     * Python wurde sie vor dem Brett gezeichnet und war praktisch unsichtbar).
     */
    drawNote(ctx, text, color) {
      const w = this.tiny.width(text) + 16;
      const h = this.tiny.height + 4;
      const cx = Math.floor(this.width / 2), cy = this.hudH + 11;
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 225], [Math.floor(cx - w / 2), Math.floor(cy - h / 2), w, h], 0, 6);
      ui.text(ctx, text, cx, cy, this.tiny, color, "center");
    }

    drawOver(ctx) {
      const cx = Math.floor(this.width / 2);
      const head = t(this.winner === 0 ? "dame.win_you" : "dame.win_ai");
      const headCol = this.winner === 0 ? this.accent : ui.TEXT_DIM;
      const hint = t("dame.new_round");
      const w = Math.min(this.width - 24, Math.max(this.huge.width(head), this.tiny.width(hint)) + 64);
      const panel = new PG.Rect(cx - Math.floor(w / 2), Math.floor(this.height / 2) - 48, w, 96);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      ui.text(ctx, head, cx, panel.y + 36, this.huge, headCol, "center");
      ui.text(ctx, hint, cx, panel.y + 74, this.tiny, ui.TEXT_DIM, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      const cx = Math.floor(this.width / 2);
      ui.drawTitle(ctx, this.width, t("dame.title"), {
        subtitle: t("dame.variant"),
        y: Math.floor(this.height * 0.13),
        big: this.huge,
        small: this.small,
        accent: this.accent,
      });
      this.varRects.forEach((rc, i) => {
        ui.drawButton(ctx, rc, t("dame.var." + VARIANTS[i]), this.small, VARIANTS[i] === this.variant, { accent: this.accent });
      });
      ui.text(ctx, t("dame.difficulty"), cx, this.diffRects[0].y - 4, this.tiny, ui.TEXT_DIM, "midbottom");
      this.diffRects.forEach((rc, i) => {
        ui.drawButton(ctx, rc, t("dame.diff." + DIFFS[i]), this.tiny, i === this.diff, { accent: this.accent });
      });
      ui.drawButton(ctx, this.startRect, t("common.start"), this.small, true, { accent: this.accent });
      ui.drawFooter(ctx, this.width, this.height, t("dame.setup_hint"), this.tiny);
    }
  }

  PG.register(DameGame, {
    id: "DameGame",
    key: "dame",
    name: { default: "Checkers", de: "Dame", fr: "Jeu de dames", es: "Damas", pt: "Damas" },
    settingsKey: "dame",
    defaults: { variant: "german", difficulty: 1 },
  });
})();
