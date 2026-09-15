/*
 * chess.js - Schach gegen die KI (Port von games/chess.py)
 * ========================================================
 * - Vollständige Regeln: alle Figurenzüge, Rochade (kurz/lang, mit korrekten
 *   Bedingungen inkl. Zug durch/ins Schach), En Passant, Bauernumwandlung mit
 *   Auswahl (Dame/Turm/Läufer/Springer), Schach, Schachmatt und Patt sowie Remis
 *   durch 50-Züge-Regel, dreifache Stellungswiederholung und ungenügendes Material.
 * - KI über Negamax mit Alpha-Beta-Schnitt, Zugsortierung (Schlagzüge zuerst),
 *   Figur- und Feldwert-Tabellen und optionaler Ruhesuche (Quiescence).
 *   Sechs Stufen von Anfänger (zufällig) bis Meister; niedrige Stufen patzen
 *   absichtlich. Zeit- und Knotenbudget deckeln die Suche; zusätzlich wird sie
 *   im Browser Wurzelzug für Wurzelzug über mehrere Frames verteilt, damit die
 *   Oberfläche nie einfriert.
 * - Punkte (Highscore) = Siege gegen die KI in einer Sitzung.
 *
 * Web-Version: nur Einzelspieler (kein 2-Spieler-Modus), keine Replays.
 *
 * Steuerung: Maus (Figur anklicken, dann Zielfeld) oder Pfeile/WASD bewegen den
 * Auswahlrahmen, Leertaste/Enter wählt/zieht. Nach Rundenende: Enter = neue
 * Runde, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  PG.addStrings({
    de: { "web.chess.subtitle": "Klassisches Schach gegen die KI" },
    en: { "web.chess.subtitle": "Classic chess against the AI" },
  });

  // ------------------------------------------------- Brett-Identitätsfarben
  const COL_LIGHT = [232, 219, 196];     // helle Felder
  const COL_DARK = [129, 100, 74];       // dunkle Felder
  const COL_PLATE = [40, 34, 30];        // Brettrahmen
  const COL_SEL = [246, 214, 92];
  const COL_MOVE = [110, 200, 130];
  const COL_LAST = [120, 160, 240];
  const COL_CHECK = [224, 84, 84];
  const COL_WHITE = [244, 244, 248];     // weiße Figuren
  const COL_BLACK = [34, 32, 38];        // schwarze Figuren
  const COL_OUTLINE = [16, 14, 18];

  // ----------------------------------------------------------------- Regeln
  const SETUP = "setup", PLAY = "play", OVER = "over";

  const DIFFS = ["lvl0", "lvl1", "lvl2", "lvl3", "lvl4", "lvl5"];
  const DIFF_DEPTH = [0, 1, 2, 2, 3, 3];
  const DIFF_QUIES = [false, false, false, true, true, true];
  const DIFF_RAND = [1.0, 0.55, 0.25, 0.12, 0.04, 0.0];
  // Harte Zeitobergrenze pro KI-Zug (Sekunden reine Rechenzeit), damit die
  // Oberfläche nie einfriert - die Suche bricht danach mit der bisherigen
  // Bewertung ab.
  const TIME_BUDGET = [0.0, 0.25, 0.4, 0.6, 0.9, 1.1];
  const NODE_BUDGET = 400000;
  // Browser: maximale Rechenzeit pro Frame (ms), bevor der nächste Wurzelzug
  // auf den nächsten Frame verschoben wird.
  const FRAME_SLICE_MS = 12;

  const KNIGHT_OFF = [[-2, -1], [-2, 1], [-1, -2], [-1, 2], [1, -2], [1, 2], [2, -1], [2, 1]];
  const KING_OFF = [[-1, -1], [-1, 0], [-1, 1], [0, -1], [0, 1], [1, -1], [1, 0], [1, 1]];
  const BISHOP_DIR = [[-1, -1], [-1, 1], [1, -1], [1, 1]];
  const ROOK_DIR = [[-1, 0], [1, 0], [0, -1], [0, 1]];
  const QUEEN_DIR = BISHOP_DIR.concat(ROOK_DIR);

  const VALUES = { P: 100, N: 320, B: 330, R: 500, Q: 900, K: 20000 };

  // Feldwert-Tabellen (aus Sicht von Weiß, Index 0 = a8 = Zeile r0). Für
  // schwarze Figuren wird vertikal gespiegelt (Zeile 7-r).
  const PST = {
    P: [0, 0, 0, 0, 0, 0, 0, 0,
      50, 50, 50, 50, 50, 50, 50, 50,
      10, 10, 20, 30, 30, 20, 10, 10,
      5, 5, 10, 25, 25, 10, 5, 5,
      0, 0, 0, 20, 20, 0, 0, 0,
      5, -5, -10, 0, 0, -10, -5, 5,
      5, 10, 10, -20, -20, 10, 10, 5,
      0, 0, 0, 0, 0, 0, 0, 0],
    N: [-50, -40, -30, -30, -30, -30, -40, -50,
      -40, -20, 0, 0, 0, 0, -20, -40,
      -30, 0, 10, 15, 15, 10, 0, -30,
      -30, 5, 15, 20, 20, 15, 5, -30,
      -30, 0, 15, 20, 20, 15, 0, -30,
      -30, 5, 10, 15, 15, 10, 5, -30,
      -40, -20, 0, 5, 5, 0, -20, -40,
      -50, -40, -30, -30, -30, -30, -40, -50],
    B: [-20, -10, -10, -10, -10, -10, -10, -20,
      -10, 0, 0, 0, 0, 0, 0, -10,
      -10, 0, 5, 10, 10, 5, 0, -10,
      -10, 5, 5, 10, 10, 5, 5, -10,
      -10, 0, 10, 10, 10, 10, 0, -10,
      -10, 10, 10, 10, 10, 10, 10, -10,
      -10, 5, 0, 0, 0, 0, 5, -10,
      -20, -10, -10, -10, -10, -10, -10, -20],
    R: [0, 0, 0, 0, 0, 0, 0, 0,
      5, 10, 10, 10, 10, 10, 10, 5,
      -5, 0, 0, 0, 0, 0, 0, -5,
      -5, 0, 0, 0, 0, 0, 0, -5,
      -5, 0, 0, 0, 0, 0, 0, -5,
      -5, 0, 0, 0, 0, 0, 0, -5,
      -5, 0, 0, 0, 0, 0, 0, -5,
      0, 0, 0, 5, 5, 0, 0, 0],
    Q: [-20, -10, -10, -5, -5, -10, -10, -20,
      -10, 0, 0, 0, 0, 0, 0, -10,
      -10, 0, 5, 5, 5, 5, 0, -10,
      -5, 0, 5, 5, 5, 5, 0, -5,
      0, 0, 5, 5, 5, 5, 0, -5,
      -10, 5, 5, 5, 5, 5, 0, -10,
      -10, 0, 5, 0, 0, 0, 0, -10,
      -20, -10, -10, -5, -5, -10, -10, -20],
    K: [-30, -40, -40, -50, -50, -40, -40, -30,
      -30, -40, -40, -50, -50, -40, -40, -30,
      -30, -40, -40, -50, -50, -40, -40, -30,
      -30, -40, -40, -50, -50, -40, -40, -30,
      -20, -30, -30, -40, -40, -30, -30, -20,
      -10, -20, -20, -20, -20, -20, -20, -10,
      20, 20, 0, 0, 0, 0, 20, 20,
      20, 30, 10, 0, 0, 10, 30, 20],
  };

  // U+FE0E erzwingt Text- statt Emoji-Darstellung (sonst wird ♟ bunt)
  const GLYPH = { K: "♚︎", Q: "♛︎", R: "♜︎", B: "♝︎", N: "♞︎", P: "♟︎" };
  const PIECE_FAMILY = '"Segoe UI Symbol", "DejaVu Sans", "Arial Unicode MS", "Noto Sans Symbols 2", FreeSerif, serif';

  // =================================================================== Regel-Helfer
  // Brett = flaches Array mit 64 Einträgen (Index r*8+c) aus "wK"/"bP"/... oder null.
  // Züge = [fr, fc, tr, tc, promo, flag] wie die Python-Tupel.
  const inB = (r, c) => r >= 0 && r < 8 && c >= 0 && c < 8;

  /** Standard-Grundstellung. r0 = Zeile 8 (Schwarz), r7 = Zeile 1 (Weiß). */
  function startBoard() {
    const back = "RNBQKBNR";
    const board = new Array(64).fill(null);
    for (let c = 0; c < 8; c++) {
      board[c] = "b" + back[c];
      board[8 + c] = "bP";
      board[48 + c] = "wP";
      board[56 + c] = "w" + back[c];
    }
    return board;
  }

  /** true, wenn Feld (r,c) von einer Figur der Farbe 'by' angegriffen wird. */
  function attacked(board, r, c, by) {
    const pr = by === "w" ? r + 1 : r - 1;
    const pawn = by + "P";
    if (pr >= 0 && pr < 8) {
      if (c > 0 && board[pr * 8 + c - 1] === pawn) return true;
      if (c < 7 && board[pr * 8 + c + 1] === pawn) return true;
    }
    const kn = by + "N";
    for (const [dr, dc] of KNIGHT_OFF) {
      const rr = r + dr, cc = c + dc;
      if (inB(rr, cc) && board[rr * 8 + cc] === kn) return true;
    }
    const kg = by + "K";
    for (const [dr, dc] of KING_OFF) {
      const rr = r + dr, cc = c + dc;
      if (inB(rr, cc) && board[rr * 8 + cc] === kg) return true;
    }
    for (const [dr, dc] of BISHOP_DIR) {
      let rr = r + dr, cc = c + dc;
      while (inB(rr, cc)) {
        const p = board[rr * 8 + cc];
        if (p) {
          if (p[0] === by && (p[1] === "B" || p[1] === "Q")) return true;
          break;
        }
        rr += dr;
        cc += dc;
      }
    }
    for (const [dr, dc] of ROOK_DIR) {
      let rr = r + dr, cc = c + dc;
      while (inB(rr, cc)) {
        const p = board[rr * 8 + cc];
        if (p) {
          if (p[0] === by && (p[1] === "R" || p[1] === "Q")) return true;
          break;
        }
        rr += dr;
        cc += dc;
      }
    }
    return false;
  }

  function kingSq(board, color) {
    const i = board.indexOf(color + "K");
    return i < 0 ? null : [(i / 8) | 0, i % 8];
  }

  function inCheck(board, color) {
    const ks = kingSq(board, color);
    if (!ks) return false;
    return attacked(board, ks[0], ks[1], color === "w" ? "b" : "w");
  }

  /** Pseudolegale Züge (ohne Fesselungsprüfung). Rochade wird schon hier
   *  korrekt auf 'nicht durch/ins Schach' geprüft. */
  function genPseudo(board, color, castling, ep) {
    const moves = [];
    const opp = color === "w" ? "b" : "w";
    const fwd = color === "w" ? -1 : 1;
    const startRow = color === "w" ? 6 : 1;
    const promoRow = color === "w" ? 0 : 7;
    for (let r = 0; r < 8; r++) {
      for (let c = 0; c < 8; c++) {
        const p = board[r * 8 + c];
        if (!p || p[0] !== color) continue;
        const typ = p[1];
        if (typ === "P") {
          // Ein Feld vor
          const nr = r + fwd;
          if (nr >= 0 && nr < 8 && board[nr * 8 + c] === null) {
            if (nr === promoRow) moves.push([r, c, nr, c, "Q", "promo"]);
            else moves.push([r, c, nr, c, null, null]);
            // Zwei Felder von der Grundreihe
            if (r === startRow && board[(r + 2 * fwd) * 8 + c] === null) {
              moves.push([r, c, r + 2 * fwd, c, null, "2step"]);
            }
          }
          // Schlagen (inkl. En Passant)
          for (const dc of [-1, 1]) {
            const cc = c + dc;
            if (!inB(nr, cc)) continue;
            const tgt = board[nr * 8 + cc];
            if (tgt && tgt[0] === opp) {
              if (nr === promoRow) moves.push([r, c, nr, cc, "Q", "promo"]);
              else moves.push([r, c, nr, cc, null, null]);
            } else if (ep !== null && ep[0] === nr && ep[1] === cc) {
              moves.push([r, c, nr, cc, null, "ep"]);
            }
          }
        } else if (typ === "N" || typ === "K") {
          for (const [dr, dc] of typ === "N" ? KNIGHT_OFF : KING_OFF) {
            const rr = r + dr, cc = c + dc;
            if (inB(rr, cc)) {
              const tgt = board[rr * 8 + cc];
              if (tgt === null || tgt[0] === opp) moves.push([r, c, rr, cc, null, null]);
            }
          }
          // Rochade
          if (typ === "K") genCastle(board, color, castling, r, c, moves);
        } else {
          const dirs = typ === "B" ? BISHOP_DIR : typ === "R" ? ROOK_DIR : QUEEN_DIR;
          for (const [dr, dc] of dirs) {
            let rr = r + dr, cc = c + dc;
            while (inB(rr, cc)) {
              const tgt = board[rr * 8 + cc];
              if (tgt === null) {
                moves.push([r, c, rr, cc, null, null]);
              } else {
                if (tgt[0] === opp) moves.push([r, c, rr, cc, null, null]);
                break;
              }
              rr += dr;
              cc += dc;
            }
          }
        }
      }
    }
    return moves;
  }

  function genCastle(board, color, castling, r, c, moves) {
    const opp = color === "w" ? "b" : "w";
    const row = color === "w" ? 7 : 0;
    if (r !== row || c !== 4) return;
    if (attacked(board, row, 4, opp)) return; // aus dem Schach darf man nicht rochieren
    const b = row * 8;
    if (castling.has(color + "K") && board[b + 5] === null && board[b + 6] === null &&
        board[b + 7] === color + "R" &&
        !attacked(board, row, 5, opp) && !attacked(board, row, 6, opp)) {
      moves.push([row, 4, row, 6, null, "castleK"]);
    }
    if (castling.has(color + "Q") && board[b + 1] === null && board[b + 2] === null &&
        board[b + 3] === null && board[b] === color + "R" &&
        !attacked(board, row, 3, opp) && !attacked(board, row, 2, opp)) {
      moves.push([row, 4, row, 2, null, "castleQ"]);
    }
  }

  /** Führt Zug aus und liefert [neues_board, neue_rochaderechte, neues_ep]. */
  function applyMove(board, mv, color, castling, ep) {
    const nb = board.slice();
    const [fr, fc, tr, tc, promo, flag] = mv;
    const piece = nb[fr * 8 + fc];
    nb[tr * 8 + tc] = piece;
    nb[fr * 8 + fc] = null;
    let newEp = null;
    if (flag === "2step") {
      newEp = [(fr + tr) >> 1, fc];
    } else if (flag === "ep") {
      nb[fr * 8 + tc] = null; // geschlagener Bauer steht neben dem Läufer
    } else if (flag === "promo") {
      nb[tr * 8 + tc] = color + (promo || "Q");
    } else if (flag === "castleK") {
      nb[fr * 8 + 5] = nb[fr * 8 + 7];
      nb[fr * 8 + 7] = null;
    } else if (flag === "castleQ") {
      nb[fr * 8 + 3] = nb[fr * 8];
      nb[fr * 8] = null;
    }
    // Rochaderechte robust aus dem Brett ableiten (König-/Turmzug/-schlag).
    let ncr = castling;
    if (castling.size) {
      ncr = new Set(castling);
      if (nb[60] !== "wK") { ncr.delete("wK"); ncr.delete("wQ"); }
      if (nb[4] !== "bK") { ncr.delete("bK"); ncr.delete("bQ"); }
      if (nb[63] !== "wR") ncr.delete("wK");
      if (nb[56] !== "wR") ncr.delete("wQ");
      if (nb[7] !== "bR") ncr.delete("bK");
      if (nb[0] !== "bR") ncr.delete("bQ");
      if (ncr.size === castling.size) ncr = castling; // unverändert -> teilen
    }
    return [nb, ncr, newEp];
  }

  /** Alle legalen Züge (pseudolegal ohne Selbst-Schach). */
  function legalMoves(board, color, castling, ep) {
    const res = [];
    for (const mv of genPseudo(board, color, castling, ep)) {
      const nb = applyMove(board, mv, color, castling, ep)[0];
      if (!inCheck(nb, color)) res.push(mv);
    }
    return res;
  }

  /** Material + Feldwerte, positiv = gut für Weiß (Zentipawns). */
  function evaluate(board) {
    let score = 0;
    for (let i = 0; i < 64; i++) {
      const p = board[i];
      if (!p) continue;
      const typ = p[1];
      if (p[0] === "w") {
        score += VALUES[typ] + PST[typ][i];
      } else {
        const r = (i / 8) | 0, c = i % 8;
        score -= VALUES[typ] + PST[typ][(7 - r) * 8 + c];
      }
    }
    return score;
  }

  /** Remis durch ungenügendes Material (nur Könige, K+L, K+S). */
  function insufficient(board) {
    let minors = 0;
    for (let i = 0; i < 64; i++) {
      const p = board[i];
      if (!p) continue;
      const t2 = p[1];
      if (t2 === "P" || t2 === "R" || t2 === "Q") return false;
      if (t2 === "B" || t2 === "N") minors++;
    }
    return minors <= 1;
  }

  function posKey(board, color, castling, ep) {
    const rows = board.map((p) => p || ".").join("");
    return rows + " " + color + " " + [...castling].sort().join("") + " " + (ep ? ep.join(",") : "-");
  }

  /** Schlagzüge zuerst (MVV-LVA), das beschleunigt Alpha-Beta stark. */
  function orderMoves(board, moves) {
    const scored = moves.map((m, i) => {
      const tgt = board[m[2] * 8 + m[3]];
      let s = 0;
      if (tgt !== null) s = 10 * VALUES[tgt[1]] - VALUES[board[m[0] * 8 + m[1]][1]];
      if (m[5] === "promo") s += 800;
      return [-s, i, m];
    });
    scored.sort((a, b) => a[0] - b[0] || a[1] - b[1]); // stabil wie Pythons sorted
    return scored.map((x) => x[2]);
  }

  const now = () => (typeof performance !== "undefined" ? performance.now() : Date.now());

  // =================================================================== Suche
  /** Suchkontext mit Knoten- und Zeitbudget (Negamax + Alpha-Beta). */
  class Searcher {
    constructor() {
      this.nodes = 0;
      this.deadline = 0;
      this.timedOut = false;
    }

    outOfTime() {
      if (this.timedOut) return true;
      if (now() > this.deadline) this.timedOut = true;
      return this.timedOut;
    }

    search(board, color, depth, alpha, beta, castling, ep, quies) {
      this.nodes++;
      if (this.nodes > NODE_BUDGET || ((this.nodes & 1023) === 0 && this.outOfTime()) || this.timedOut) {
        const rel = evaluate(board);
        return color === "w" ? rel : -rel;
      }
      let moves = legalMoves(board, color, castling, ep);
      if (!moves.length) {
        if (inCheck(board, color)) return -30000 + (5 - depth); // Matt: je schneller, desto besser
        return 0; // Patt
      }
      if (depth <= 0) {
        if (quies) return this.quiesce(board, color, alpha, beta, castling, ep, 4);
        const rel = evaluate(board);
        return color === "w" ? rel : -rel;
      }
      moves = orderMoves(board, moves);
      let best = -1e9;
      const opp = color === "w" ? "b" : "w";
      for (const mv of moves) {
        const [nb, nc, nep] = applyMove(board, mv, color, castling, ep);
        const val = -this.search(nb, opp, depth - 1, -beta, -alpha, nc, nep, quies);
        if (val > best) best = val;
        if (best > alpha) alpha = best;
        if (alpha >= beta) break;
      }
      return best;
    }

    quiesce(board, color, alpha, beta, castling, ep, ply) {
      this.nodes++;
      const rel = evaluate(board);
      const stand = color === "w" ? rel : -rel;
      if (ply <= 0 || this.nodes > NODE_BUDGET || ((this.nodes & 63) === 0 && this.outOfTime()) || this.timedOut) return stand;
      if (stand >= beta) return beta;
      if (stand > alpha) alpha = stand;
      const opp = color === "w" ? "b" : "w";
      const caps = legalMoves(board, color, castling, ep).filter((m) => board[m[2] * 8 + m[3]] !== null || m[5] === "ep");
      for (const mv of orderMoves(board, caps)) {
        const [nb, nc, nep] = applyMove(board, mv, color, castling, ep);
        const val = -this.quiesce(nb, opp, -beta, -alpha, nc, nep, ply - 1);
        if (val >= beta) return beta;
        if (val > alpha) alpha = val;
      }
      return alpha;
    }
  }

  class ChessGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.diff = Math.max(0, Math.min(5, parseInt(this.opts.difficulty, 10) || 0));
      if (this.opts.difficulty == null) this.diff = 2;
      this.humanColor = this.opts.color === "black" ? "b" : "w";

      this.makeFonts();
      this.wins = [0, 0]; // [Weiß, Schwarz]
      this.buildSetupLayout();
      this.newRound();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größe abhängig von der Fensterhöhe. */
    makeFonts() {
      this.small = ui.font(Math.max(14, Math.floor(this.height / 34)));
      this.tiny = ui.font(Math.max(12, Math.floor(this.height / 44)));
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 12)), true);
    }

    makePieceFont() {
      this.piecePx = Math.floor(this.cell * 0.74);
      this.pieceCss = this.piecePx + "px " + PIECE_FAMILY;
    }

    layout() {
      this.hudH = 44;
      this.cell = Math.floor(Math.min((this.width - 40) / 8, (this.height - this.hudH - 24) / 8));
      this.bw = this.bh = 8 * this.cell;
      this.bx = Math.floor((this.width - this.bw) / 2);
      this.by = this.hudH + Math.max(8, Math.floor((this.height - this.hudH - this.bh) / 2));
    }

    newRound() {
      this.board = startBoard();
      this.castling = new Set(["wK", "wQ", "bK", "bQ"]);
      this.ep = null;
      this.turn = "w";
      this.halfmove = 0;
      this.posCounts = new Map();
      this.sel = null;
      this.targets = new Map(); // "r,c" -> Zug
      this.cursor = [6, 4];
      this.lastMove = null;
      this.winner = null; // 0=Weiß, 1=Schwarz, null=Remis/laufend
      this.resultKey = null;
      this.promoMove = null;
      this.aiDelay = 0.6;
      this.aiJob = null;
      // Jede Partie meldet ihr Ergebnis einzeln an die Statistik (wie Python)
      this._resultReported = false;
      this.layout();
      this.makePieceFont();
      this.refreshLegal();
    }

    refreshLegal() {
      this.legal = legalMoves(this.board, this.turn, this.castling, this.ep);
      this.legalByFrom = new Map();
      for (const mv of this.legal) {
        const k = mv[0] * 8 + mv[1];
        if (!this.legalByFrom.has(k)) this.legalByFrom.set(k, []);
        this.legalByFrom.get(k).push(mv);
      }
      this.check = inCheck(this.board, this.turn);
      const key = posKey(this.board, this.turn, this.castling, this.ep);
      const n = (this.posCounts.get(key) || 0) + 1;
      this.posCounts.set(key, n);
      this.threefold = n >= 3;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const y0 = Math.floor(this.height * 0.30);
      const bw = Math.min(360, this.width - 60);
      // Sechs Schwierigkeits-Buttons in einer Reihe
      const gap = 8, n = 6;
      const cellw = (bw - gap * (n - 1)) / n;
      this.diffRects = [];
      for (let i = 0; i < n; i++) {
        this.diffRects.push(new PG.Rect(Math.floor(cx - bw / 2 + i * (cellw + gap)), y0, Math.floor(cellw), 46));
      }
      const y1 = y0 + 84;
      const cw = Math.min(150, Math.floor((bw - gap) / 2));
      this.colorRects = [
        new PG.Rect(cx - cw - gap / 2, y1, cw, 42),
        new PG.Rect(cx + gap / 2, y1, cw, 42),
      ];
      this.startRect = new PG.Rect(cx - 95, y1 + 60, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4", "5", "6"].includes(k)) {
          this.diff = Number(k) - 1;
          this.saveSetting("difficulty", this.diff);
          this.playSound("click");
        } else if (["Left", "a", "A", "Up", "w", "W"].includes(k)) {
          this.diff = PG.mod(this.diff - 1, 6);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (["Right", "d", "D", "Down", "s", "S"].includes(k)) {
          this.diff = PG.mod(this.diff + 1, 6);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "c" || k === "C") {
          this.humanColor = this.humanColor === "w" ? "b" : "w";
          this.saveSetting("color", this.humanColor === "b" ? "black" : "white");
          this.playSound("select");
        } else if (k === "Return" || k === "space") {
          this.startPlay();
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
        for (let i = 0; i < this.colorRects.length; i++) {
          if (this.colorRects[i].collidepoint(ev.pos)) {
            this.humanColor = i === 0 ? "w" : "b";
            this.saveSetting("color", i === 0 ? "white" : "black");
            this.playSound("select");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    startPlay() {
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
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
      if (this.promoMove !== null) {
        this.handlePromo(ev);
        return;
      }
      if (!this.humanTurn()) return;
      if (ev.kind === "mousemove") {
        const rc = this.cellAt(ev.pos);
        // Cursor speichert ANZEIGE-Koordinaten (wichtig bei gedrehtem Brett,
        // wenn der Mensch Schwarz spielt).
        if (rc) this.cursor = this.boardToDisp(rc[0], rc[1]);
      } else if (ev.kind === "mousedown") {
        const rc = this.cellAt(ev.pos);
        if (rc) {
          this.cursor = this.boardToDisp(rc[0], rc[1]);
          this.clickCell(rc[0], rc[1]);
        }
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.isAction(k, "up")) {
          this.cursor[0] = Math.max(0, this.cursor[0] - 1);
          this.playSound("move");
        } else if (this.isAction(k, "down")) {
          this.cursor[0] = Math.min(7, this.cursor[0] + 1);
          this.playSound("move");
        } else if (this.isAction(k, "left")) {
          this.cursor[1] = Math.max(0, this.cursor[1] - 1);
          this.playSound("move");
        } else if (this.isAction(k, "right")) {
          this.cursor[1] = Math.min(7, this.cursor[1] + 1);
          this.playSound("move");
        } else if (this.isAction(k, "action")) {
          const d = this.dispToBoard(this.cursor[0], this.cursor[1]);
          this.clickCell(d[0], d[1]);
        }
      }
    }

    humanTurn() {
      return this.turn === this.humanColor;
    }

    clickCell(r, c) {
      // Auswahl eines eigenen Steins
      const p = this.board[r * 8 + c];
      const key = r + "," + c;
      if (this.sel !== null && this.targets.has(key)) {
        const mv = this.targets.get(key);
        if (mv[5] === "promo") {
          this.promoMove = [mv[0], mv[1], mv[2], mv[3]];
          this.playSound("select");
          return;
        }
        this.commit(mv);
        return;
      }
      if (p && p[0] === this.turn) {
        this.sel = [r, c];
        this.targets = new Map();
        for (const m of this.legalByFrom.get(r * 8 + c) || []) this.targets.set(m[2] + "," + m[3], m);
        this.playSound("click");
      } else {
        this.sel = null;
        this.targets = new Map();
      }
    }

    handlePromo(ev) {
      const order = ["Q", "R", "B", "N"];
      if (ev.kind === "mousedown") {
        const rects = this.promoRects();
        for (let i = 0; i < rects.length; i++) {
          if (rects[i].collidepoint(ev.pos)) {
            this.finishPromo(order[i]);
            return;
          }
        }
      } else if (ev.kind === "keydown" && ev.key) {
        const m = { q: "Q", r: "R", b: "B", n: "N" };
        const k = ev.key.toLowerCase();
        if (m[k]) this.finishPromo(m[k]);
      }
    }

    finishPromo(piece) {
      const [fr, fc, tr, tc] = this.promoMove;
      this.promoMove = null;
      this.commit([fr, fc, tr, tc, piece, "promo"]);
    }

    cellAt(pos) {
      const c = Math.floor((pos[0] - this.bx) / this.cell);
      const r = Math.floor((pos[1] - this.by) / this.cell);
      if (r >= 0 && r < 8 && c >= 0 && c < 8) return this.dispToBoard(r, c);
      return null;
    }

    // Menschliche Seite immer unten anzeigen.
    flip() {
      return this.humanColor === "b";
    }

    dispToBoard(dr, dc) {
      return this.flip() ? [7 - dr, 7 - dc] : [dr, dc];
    }

    boardToDisp(r, c) {
      return this.flip() ? [7 - r, 7 - c] : [r, c];
    }

    restart() {
      this.gameOver = false;
      // Farben wechseln nach jeder Partie
      this.humanColor = this.humanColor === "w" ? "b" : "w";
      this.saveSetting("color", this.humanColor === "b" ? "black" : "white");
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Zug ausführen
    commit(mv) {
      const piece = this.board[mv[0] * 8 + mv[1]];
      const capture = this.board[mv[2] * 8 + mv[3]] !== null || mv[5] === "ep";
      [this.board, this.castling, this.ep] = applyMove(this.board, mv, this.turn, this.castling, this.ep);
      if (piece[1] === "P" || capture) this.halfmove = 0;
      else this.halfmove += 1;
      this.lastMove = [mv[0], mv[1], mv[2], mv[3]];
      this.sel = null;
      this.targets = new Map();
      this.turn = this.turn === "w" ? "b" : "w";
      this.refreshLegal();
      this.playSound(capture ? "lock" : "move");
      this.checkEnd();
    }

    checkEnd() {
      if (!this.legal.length) {
        if (this.check) {
          // Der Spieler am Zug ist matt -> der andere gewinnt.
          this.winner = this.turn === "w" ? 1 : 0;
          this.resultKey = "checkmate";
        } else {
          this.winner = null;
          this.resultKey = "stalemate";
        }
        this.end();
        return;
      }
      if (this.halfmove >= 100) {
        this.winner = null;
        this.resultKey = "fifty";
        this.end();
        return;
      }
      if (this.threefold) {
        this.winner = null;
        this.resultKey = "threefold";
        this.end();
        return;
      }
      if (insufficient(this.board)) {
        this.winner = null;
        this.resultKey = "material";
        this.end();
        return;
      }
      if (this.turn !== this.humanColor) this.aiDelay = 0.45;
    }

    end() {
      this.state = OVER;
      this.aiJob = null;
      if (this.winner !== null) {
        this.wins[this.winner] += 1;
        const humanIdx = this.humanColor === "w" ? 0 : 1;
        if (this.winner === humanIdx) {
          this.score = this.wins[humanIdx];
          this.playSound("win");
          this.reportResult(true);
          this.achEvent("chess_win");
        } else {
          this.playSound("gameover");
          this.reportResult(false);
        }
      } else {
        this.playSound("select");
      }
      this.gameOver = true;
    }

    // ===================================================== KI
    update(dt) {
      if (this.state === PLAY && this.promoMove === null && this.turn !== this.humanColor) {
        if (this.aiJob) {
          this.stepAi();
          return;
        }
        this.aiDelay -= dt;
        if (this.aiDelay <= 0) this.aiPlay();
      }
    }

    aiPlay() {
      const moves = this.legal.slice();
      if (!moves.length) return;
      if (this.diff === 0 || PG.rand.random() < DIFF_RAND[this.diff]) {
        // Schwache Stufen: bevorzugt einfache Schlagzüge, sonst zufällig.
        const caps = moves.filter((m) => this.board[m[2] * 8 + m[3]] !== null);
        if (caps.length && PG.rand.random() < 0.6) this.commit(PG.rand.choice(caps));
        else this.commit(PG.rand.choice(moves));
        return;
      }
      // Suche starten - sie läuft in stepAi() Wurzelzug für Wurzelzug über
      // mehrere Frames, bis alle Züge bewertet sind oder das Budget alle ist.
      const s = new Searcher();
      this.aiJob = {
        s,
        depth: DIFF_DEPTH[this.diff],
        quies: DIFF_QUIES[this.diff],
        moves: orderMoves(this.board, moves),
        idx: 0,
        bestVal: -1e9,
        best: [],
        budget: TIME_BUDGET[this.diff] * 1000,
        spent: 0,
        turn: this.turn,
      };
      this.stepAi();
    }

    stepAi() {
      const job = this.aiJob;
      if (job.turn !== this.turn) {
        this.aiJob = null;
        return;
      }
      const frameStart = now();
      const opp = this.turn === "w" ? "b" : "w";
      let done = false;
      while (job.idx < job.moves.length) {
        const mv = job.moves[job.idx++];
        const t0 = now();
        // Das Zeitbudget zählt reine Rechenzeit (über alle Frames summiert).
        job.s.deadline = t0 + Math.max(0, job.budget - job.spent);
        const [nb, nc, nep] = applyMove(this.board, mv, this.turn, this.castling, this.ep);
        const val = -job.s.search(nb, opp, job.depth - 1, -1e9, 1e9, nc, nep, job.quies);
        const t1 = now();
        job.spent += t1 - t0;
        if (job.best.length && job.spent > job.budget) {
          done = true; // Zeitbudget erschöpft - bisher bester Zug zählt
          break;
        }
        if (val > job.bestVal) {
          job.bestVal = val;
          job.best = [mv];
        } else if (val === job.bestVal) {
          job.best.push(mv);
        }
        if (t1 - frameStart > FRAME_SLICE_MS) break; // Rest im nächsten Frame
      }
      if (job.idx >= job.moves.length) done = true;
      if (!done) return;
      this.aiJob = null;
      const pick = job.best.length ? PG.rand.choice(job.best) : PG.rand.choice(job.moves);
      this.commit(pick);
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
      if (this.promoMove !== null) this.drawPromo(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    sqRect(r, c) {
      const [dr, dc] = this.boardToDisp(r, c);
      return new PG.Rect(this.bx + dc * this.cell, this.by + dr * this.cell, this.cell, this.cell);
    }

    drawBoard(ctx) {
      const cell = this.cell;
      const plate = new PG.Rect(this.bx - 7, this.by - 7, this.bw + 14, this.bh + 14);
      draw.rect(ctx, COL_PLATE, plate, 0, 8);
      draw.rect(ctx, ui.mix(COL_PLATE, this.accent, 0.45), plate, 1, 8);
      for (let r = 0; r < 8; r++) {
        for (let c = 0; c < 8; c++) {
          draw.rect(ctx, (r + c) % 2 === 0 ? COL_LIGHT : COL_DARK, this.sqRect(r, c));
        }
      }
      // Letzter Zug (sanft pulsierend)
      if (this.lastMove) {
        const alpha = Math.floor(60 + 45 * ui.pulse(1.6));
        const lm = this.lastMove;
        for (const [r, c] of [[lm[0], lm[1]], [lm[2], lm[3]]]) {
          draw.rect(ctx, [COL_LAST[0], COL_LAST[1], COL_LAST[2], alpha], this.sqRect(r, c));
        }
      }
      // König im Schach markieren
      if (this.check && this.state === PLAY) {
        const ks = kingSq(this.board, this.turn);
        if (ks) draw.rect(ctx, [COL_CHECK[0], COL_CHECK[1], COL_CHECK[2], 110], this.sqRect(ks[0], ks[1]));
      }
      // Auswahl + Zughinweise
      const human = this.state === PLAY && this.humanTurn();
      if (this.sel !== null && human) {
        draw.rect(ctx, COL_SEL, this.sqRect(this.sel[0], this.sel[1]), 3);
        for (const mv of this.targets.values()) {
          const rect = this.sqRect(mv[2], mv[3]);
          const center = [rect.centerx, rect.centery];
          if (this.board[mv[2] * 8 + mv[3]] !== null || mv[5] === "ep") {
            draw.circle(ctx, COL_MOVE, center, Math.floor(cell / 2) - 3, 3);
          } else {
            draw.circle(ctx, COL_MOVE, center, Math.max(4, Math.floor(cell / 7)));
          }
        }
      }
      // Figuren
      for (let r = 0; r < 8; r++) {
        for (let c = 0; c < 8; c++) {
          const p = this.board[r * 8 + c];
          if (p) this.drawPiece(ctx, p, this.sqRect(r, c));
        }
      }
      // Cursor (Tastatur)
      if (human) {
        const [dr, dc] = this.cursor;
        const k = ui.pulse(2.2, 0.0, 1.0);
        const g = Math.floor(120 + 120 * k);
        draw.rect(ctx, [g, g, g], [this.bx + dc * cell + 1, this.by + dr * cell + 1, cell - 2, cell - 2], 2);
      }
    }

    drawPiece(ctx, piece, rect) {
      const glyph = GLYPH[piece[1]];
      const main = piece[0] === "w" ? COL_WHITE : COL_BLACK;
      const cx = rect.centerx, cy = rect.centery + this.piecePx * 0.04;
      ctx.save();
      ctx.font = this.pieceCss;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      // Outline für Lesbarkeit
      ctx.fillStyle = ui.col(COL_OUTLINE);
      for (const [ox, oy] of [[-2, 0], [2, 0], [0, -2], [0, 2], [-1, -1], [1, 1]]) {
        ctx.fillText(glyph, cx + ox, cy + oy);
      }
      ctx.fillStyle = ui.col(main);
      ctx.fillText(glyph, cx, cy);
      ctx.restore();
    }

    drawHud(ctx) {
      const panel = new PG.Rect(8, 6, this.width - 16, this.hudH - 10);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      const cy = panel.centery;
      // Siegzähler mit Farbpunkt (links Weiß, rechts Schwarz)
      draw.circle(ctx, COL_WHITE, [panel.x + 16, cy], 7);
      draw.circle(ctx, ui.BORDER_LIGHT, [panel.x + 16, cy], 7, 1);
      ui.text(ctx, String(this.wins[0]), panel.x + 30, cy, this.small, ui.TEXT, "midleft");
      draw.circle(ctx, COL_BLACK, [panel.right - 16, cy], 7);
      draw.circle(ctx, ui.BORDER_LIGHT, [panel.right - 16, cy], 7, 1);
      ui.text(ctx, String(this.wins[1]), panel.right - 30, cy, this.small, ui.TEXT, "midright");
      if (this.state === PLAY) {
        let warn = false;
        let mid;
        if (this.promoMove !== null) {
          mid = t("chess.promote");
        } else if (this.turn !== this.humanColor) {
          mid = t("chess.ai_thinks");
        } else {
          mid = this.check ? t("chess.check") : t("chess.your_turn");
          warn = this.check;
        }
        ui.text(ctx, mid, this.width / 2, cy, this.small, warn ? ui.RED : this.accent, "center");
      }
    }

    promoRects() {
      const n = 4;
      const w = Math.min(70, Math.floor((this.width - 40) / n));
      const total = w * n + 12 * (n - 1);
      const x0 = Math.floor((this.width - total) / 2);
      const y = Math.floor(this.height / 2 - w / 2);
      const out = [];
      for (let i = 0; i < n; i++) out.push(new PG.Rect(x0 + i * (w + 12), y, w, w));
      return out;
    }

    drawPromo(ctx) {
      draw.rect(ctx, [10, 8, 12, 180], [0, 0, this.width, this.height]);
      const rects = this.promoRects();
      ui.text(ctx, t("chess.promote"), this.width / 2, rects[0].y - 26, this.small, ui.TEXT, "center");
      ["Q", "R", "B", "N"].forEach((ch, i) => {
        const rc = rects[i];
        draw.rect(ctx, ui.BTN_SEL, rc, 0, 8);
        draw.rect(ctx, this.accent, rc, 2, 8);
        this.drawPiece(ctx, this.turn + ch, rc);
      });
    }

    drawOver(ctx) {
      const cx = this.width / 2;
      let head, headCol;
      if (this.winner === null) {
        head = t("common.draw");
        headCol = ui.TEXT_DIM;
      } else {
        const humanIdx = this.humanColor === "w" ? 0 : 1;
        const won = this.winner === humanIdx;
        head = won ? t("chess.win_you") : t("chess.win_ai");
        headCol = won ? this.accent : ui.TEXT_DIM;
      }
      const sub = t("chess.reason." + (this.resultKey || ""));
      const hint = t("chess.new_round");
      const w = Math.min(this.width - 24, Math.max(this.huge.width(head), this.small.width(sub), this.tiny.width(hint)) + 64);
      const panel = new PG.Rect(cx - w / 2, this.height / 2 - 56, w, 112);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      ui.text(ctx, head, cx, panel.y + 34, this.huge, headCol, "center");
      ui.text(ctx, sub, cx, panel.y + 68, this.small, ui.TEXT, "center");
      ui.text(ctx, hint, cx, panel.y + 94, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = this.width / 2;
      ui.drawTitle(ctx, this.width, t("chess.title"), {
        subtitle: t("web.chess.subtitle"), y: Math.floor(this.height * 0.14),
        big: this.huge, small: this.small, accent: this.accent,
      });
      // Schwierigkeits-Buttons (1-6)
      this.diffRects.forEach((rc, i) => {
        ui.drawButton(ctx, rc, String(i + 1), this.font, i === this.diff, { accent: this.accent });
      });
      ui.text(ctx, t("chess.diff." + DIFFS[this.diff]), cx, this.diffRects[0].bottom + 18, this.small, ui.TEXT, "center");
      // Farbwahl
      const labels = [t("chess.white"), t("chess.black")];
      this.colorRects.forEach((rc, i) => {
        const on = this.humanColor === (i === 0 ? "w" : "b");
        ui.drawButton(ctx, rc, labels[i], this.small, on, { accent: this.accent });
      });
      // Start
      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: this.accent });
      ui.drawFooter(ctx, this.width, this.height, t("chess.setup_hint"), this.tiny);
    }
  }

  // Regel-Engine für Tests zugänglich machen
  PG.chessEngine = { startBoard, legalMoves, applyMove, inCheck, evaluate, insufficient, orderMoves, Searcher };

  PG.register(ChessGame, {
    id: "ChessGame",
    key: "chess",
    name: { default: "Chess", de: "Schach", fr: "Échecs", es: "Ajedrez", pt: "Xadrez" },
    settingsKey: "chess",
    defaults: { difficulty: 2, color: "white" },
  });
})();
