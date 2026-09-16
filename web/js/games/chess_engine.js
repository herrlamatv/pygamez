/*
 * chess_engine.js - Regeln, Notation und KI für Schach (Port von games/chess_engine.py)
 * ==================================================================================
 * 1:1-Übertragung der Python-Engine: 0x88-Brett, gleiche Zuggenerierungs-
 * Reihenfolge, Chess960-Rochade ("König schlägt eigenen Turm"), FEN/SAN/UCI/PGN,
 * dieselbe Bewertung (PeSTO + Mobilität, Bauernstruktur, Königssicherheit,
 * Läuferpaar, Türme auf offenen Linien, Mop-up) und dieselbe fortsetzbare Suche
 * (Generator: iterative Vertiefung, Transpositionstabelle, PVS, Null-Zug, LMR,
 * Killer/History, Ruhesuche). Gleiche Stufen-Parameter - bei gleichem
 * Knotenlimit finden Python und JavaScript denselben Zug.
 *
 * Zobrist-Schlüssel: dieselben 64-Bit-Zahlen wie in Python, hier als zwei
 * 32-Bit-Hälften; Tabellen-Schlüssel = obere 53 Bit (exakt als Zahl darstellbar).
 *
 * Läuft auch ohne Browser (Node): PG wird dann auf globalThis angelegt.
 */
(function () {
  "use strict";

  const root = typeof window !== "undefined" ? window : globalThis;
  const PG = root.PG || (root.PG = {});
  const now = () => (typeof performance !== "undefined" ? performance.now() : Date.now()) / 1000;

  const WHITE = 0, BLACK = 1;
  const PAWN = 1, KNIGHT = 2, BISHOP = 3, ROOK = 4, QUEEN = 5, KING = 6;
  const M_NORMAL = 0, M_DOUBLE = 1, M_EP = 2, M_CASTLE = 3;

  const SQUARES = [];
  for (let r = 0; r < 8; r++) for (let f = 0; f < 8; f++) SQUARES.push(r * 16 + f);
  const KNIGHT_D = [33, 31, 18, 14, -14, -18, -31, -33];
  const KING_D = [17, 16, 15, 1, -1, -15, -16, -17];
  const BISHOP_D = [17, 15, -15, -17];
  const ROOK_D = [16, 1, -1, -16];

  const LETTERS = " PNBRQK";
  const FILES = "abcdefgh";
  const MATE = 30000, INF = 32000, MAX_PLY = 64;

  const sqName = (sq) => FILES[sq & 7] + String((sq >> 4) + 1);
  const sqParse = (name) => (Number(name[1]) - 1) * 16 + FILES.indexOf(name[0]);
  const pieceChar = (p) => {
    const ch = LETTERS[p & 7];
    return p >> 3 ? ch.toLowerCase() : ch;
  };

  // ----------------------------------------------------------------- Zobrist
  /** Dieselben 64-Bit-Zahlen wie Python (xorshift64*), als [hoch, tief]. */
  function zobristNumbers(count) {
    let x = 0x9E3779B97F4A7C15n;
    const mask = (1n << 64n) - 1n;
    const out = [];
    for (let i = 0; i < count; i++) {
      x ^= x >> 12n;
      x ^= (x << 25n) & mask;
      x ^= x >> 27n;
      const v = (x * 0x2545F4914F6CDD1Dn) & mask;
      out.push([Number(v >> 32n) | 0, Number(v & 0xFFFFFFFFn) | 0]);
    }
    return out;
  }
  const ZN = zobristNumbers(16 * 128 + 1 + 32 + 8);
  const Z_PIECE_HI = [], Z_PIECE_LO = [];
  for (let i = 0; i < 16; i++) {
    Z_PIECE_HI.push(new Int32Array(128));
    Z_PIECE_LO.push(new Int32Array(128));
    for (let s = 0; s < 128; s++) {
      Z_PIECE_HI[i][s] = ZN[i * 128 + s][0];
      Z_PIECE_LO[i][s] = ZN[i * 128 + s][1];
    }
  }
  const Z_SIDE = ZN[16 * 128];
  const Z_CASTLE = [];
  for (let i = 0; i < 4; i++) Z_CASTLE.push(ZN.slice(16 * 128 + 1 + i * 8, 16 * 128 + 1 + (i + 1) * 8));
  const Z_EP = ZN.slice(16 * 128 + 33, 16 * 128 + 41);

  function castleKey(castle) {
    let hi = 0, lo = 0;
    for (let i = 0; i < 4; i++) {
      if (castle[i] >= 0) {
        hi ^= Z_CASTLE[i][castle[i] & 7][0];
        lo ^= Z_CASTLE[i][castle[i] & 7][1];
      }
    }
    return [hi, lo];
  }
  const keyOf = (hi, lo) => (hi >>> 0) * 2097152 + (lo >>> 11);

  // ------------------------------------------------------------ Bewertung
  // PeSTO-Tabellen (Index 0 = a8 aus Sicht von Weiß), identisch zu Python.
  const MG_VALUE = [0, 82, 337, 365, 477, 1025, 0];
  const EG_VALUE = [0, 94, 281, 297, 512, 936, 0];
  const PHASE_INC = [0, 0, 1, 1, 2, 4, 0];
  const MG_T = {
    1: [0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20,
      -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
      -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0],
    2: [-167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84, 129, 73, 44,
      -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10, 19, 17, 25, -16,
      -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23],
    3: [-29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50, 37, -2,
      -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18, 10,
      4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21],
    4: [32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
      -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0, -5, -33,
      -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26],
    5: [-28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47, 57,
      -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2, 14, 5,
      -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50],
    6: [-65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6, 22, -22,
      -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14, -14, -22, -46, -44, -30, -15, -27,
      1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24, 14],
  };
  const EG_T = {
    1: [0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53, 82, 84,
      32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1, -8,
      13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0],
    2: [-58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10, 9, -1, -9, -19, -41,
      -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23, -3, -1, 15, 10, -3, -20, -22,
      -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15, -22, -18, -50, -64],
    3: [-14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6, 0, 4,
      -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7, -15,
      -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17],
    4: [13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3,
      4, 3, 13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16,
      -6, -6, 0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20],
    5: [-9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19, 9,
      3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17, 10, 5,
      -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41],
    6: [-74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20, 45, 44, 13,
      -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11, 21, 23, 16, 7, -9,
      -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43],
  };
  function buildPst(tables, values) {
    const out = [];
    for (let i = 0; i < 16; i++) out.push(new Int32Array(128));
    for (let typ = 1; typ <= 6; typ++) {
      for (const sq of SQUARES) {
        const r = sq >> 4, f = sq & 7;
        out[typ][sq] = values[typ] + tables[typ][(7 - r) * 8 + f];
        out[typ | 8][sq] = values[typ] + tables[typ][r * 8 + f];
      }
    }
    return out;
  }
  const PST_MG = buildPst(MG_T, MG_VALUE);
  const PST_EG = buildPst(EG_T, EG_VALUE);

  const BISHOP_PAIR = [25, 45];
  const DOUBLED = [8, 18];
  const ISOLATED = [10, 12];
  const PASSED_MG = [0, 5, 8, 15, 28, 50, 80, 0];
  const PASSED_EG = [0, 10, 15, 28, 50, 85, 130, 0];
  const ROOK_OPEN = [20, 10];
  const ROOK_HALF = [10, 5];
  const MOB_BASE = [0, 0, 4, 7, 7, 14, 0];
  const MOB_MG = [0, 0, 4, 3, 2, 1, 0];
  const MOB_EG = [0, 0, 4, 4, 4, 2, 0];
  const SHIELD_NEAR = 0, SHIELD_FAR = 10, SHIELD_NONE = 25, SHIELD_OPEN = 15;
  const TEMPO = 10;
  const ORDER_VALUE = [0, 1, 3, 3, 5, 9, 20];

  const pawnCache = new Map();

  // ================================================================ Position
  class Position {
    constructor() {
      this.b = new Int8Array(128);
      this.side = WHITE;
      this.castle = [-1, -1, -1, -1]; // Turmfeld je Recht: wK, wQ, bK, bQ
      this.ep = -1;
      this.half = 0;
      this.full = 1;
      this.kings = [-1, -1];
      this.h1 = 0; this.h2 = 0; // Zobrist hoch/tief
      this.p1 = 0; this.p2 = 0; // Bauern-Schlüssel
      this.mg = [0, 0];
      this.eg = [0, 0];
      this.phase = 0;
      this.counts = new Int32Array(16);
      this.stack = [];
      this.hist = [];
      this.chess960 = false;
    }

    get hash() { return keyOf(this.h1, this.h2); }

    copy() {
      const p = new Position();
      p.b = this.b.slice();
      p.side = this.side;
      p.castle = this.castle.slice();
      p.ep = this.ep;
      p.half = this.half;
      p.full = this.full;
      p.kings = this.kings.slice();
      p.h1 = this.h1; p.h2 = this.h2; p.p1 = this.p1; p.p2 = this.p2;
      p.mg = this.mg.slice();
      p.eg = this.eg.slice();
      p.phase = this.phase;
      p.counts = this.counts.slice();
      p.stack = [];
      p.hist = this.hist.slice();
      p.chess960 = this.chess960;
      return p;
    }

    recompute() {
      const b = this.b;
      this.mg = [0, 0];
      this.eg = [0, 0];
      this.phase = 0;
      this.counts = new Int32Array(16);
      let h1 = 0, h2 = 0, p1 = 0, p2 = 0;
      for (const sq of SQUARES) {
        const p = b[sq];
        if (!p) continue;
        const c = p >> 3;
        this.mg[c] += PST_MG[p][sq];
        this.eg[c] += PST_EG[p][sq];
        this.phase += PHASE_INC[p & 7];
        this.counts[p]++;
        h1 ^= Z_PIECE_HI[p][sq]; h2 ^= Z_PIECE_LO[p][sq];
        if ((p & 7) === PAWN) { p1 ^= Z_PIECE_HI[p][sq]; p2 ^= Z_PIECE_LO[p][sq]; }
        if ((p & 7) === KING) this.kings[c] = sq;
      }
      if (this.side === BLACK) { h1 ^= Z_SIDE[0]; h2 ^= Z_SIDE[1]; }
      const ck = castleKey(this.castle);
      h1 ^= ck[0]; h2 ^= ck[1];
      if (this.ep >= 0) { h1 ^= Z_EP[this.ep & 7][0]; h2 ^= Z_EP[this.ep & 7][1]; }
      this.h1 = h1; this.h2 = h2; this.p1 = p1; this.p2 = p2;
      this.hist = [keyOf(h1, h2)];
    }

    // --------------------------------------------------------- Angriffe
    attacked(sq, by) {
      const b = this.b;
      let kn, kg, bi, rk, qu, s;
      if (by === WHITE) {
        s = sq - 15; if (!(s & 0x88) && b[s] === 1) return true;
        s = sq - 17; if (!(s & 0x88) && b[s] === 1) return true;
        kn = 2; kg = 6; bi = 3; rk = 4; qu = 5;
      } else {
        s = sq + 15; if (!(s & 0x88) && b[s] === 9) return true;
        s = sq + 17; if (!(s & 0x88) && b[s] === 9) return true;
        kn = 10; kg = 14; bi = 11; rk = 12; qu = 13;
      }
      for (let i = 0; i < 8; i++) {
        s = sq + KNIGHT_D[i];
        if (!(s & 0x88) && b[s] === kn) return true;
      }
      for (let i = 0; i < 8; i++) {
        s = sq + KING_D[i];
        if (!(s & 0x88) && b[s] === kg) return true;
      }
      for (let i = 0; i < 4; i++) {
        const d = BISHOP_D[i];
        s = sq + d;
        while (!(s & 0x88)) {
          const q = b[s];
          if (q) {
            if (q === bi || q === qu) return true;
            break;
          }
          s += d;
        }
      }
      for (let i = 0; i < 4; i++) {
        const d = ROOK_D[i];
        s = sq + d;
        while (!(s & 0x88)) {
          const q = b[s];
          if (q) {
            if (q === rk || q === qu) return true;
            break;
          }
          s += d;
        }
      }
      return false;
    }

    inCheck(color) {
      const c = color == null ? this.side : color;
      return this.attacked(this.kings[c], 1 - c);
    }

    // --------------------------------------------------------- Zuggenerator
    /** Pseudolegale Züge. tactical: nur Schlagzüge + Damenumwandlung. */
    genMoves(tactical = false) {
      const b = this.b;
      const us = this.side, them = 1 - us;
      const moves = [];
      let fwd, startRank, promoRank;
      if (us === WHITE) { fwd = 16; startRank = 1; promoRank = 7; } else { fwd = -16; startRank = 6; promoRank = 0; }
      const ep = this.ep;
      for (let si = 0; si < 64; si++) {
        const sq = SQUARES[si];
        const p = b[sq];
        if (!p || (p >> 3) !== us) continue;
        const typ = p & 7;
        if (typ === PAWN) {
          const to = sq + fwd;
          const promo = (to >> 4) === promoRank;
          if (!b[to]) {
            if (promo) {
              moves.push(sq | (to << 8) | (QUEEN << 16));
              if (!tactical) {
                moves.push(sq | (to << 8) | (ROOK << 16));
                moves.push(sq | (to << 8) | (BISHOP << 16));
                moves.push(sq | (to << 8) | (KNIGHT << 16));
              }
            } else if (!tactical) {
              moves.push(sq | (to << 8));
              if ((sq >> 4) === startRank && !b[to + fwd]) moves.push(sq | ((to + fwd) << 8) | (M_DOUBLE << 20));
            }
          }
          for (const t of [to - 1, to + 1]) {
            if (t & 0x88) continue;
            const q = b[t];
            if (q && (q >> 3) === them) {
              if (promo) {
                moves.push(sq | (t << 8) | (QUEEN << 16));
                if (!tactical) {
                  moves.push(sq | (t << 8) | (ROOK << 16));
                  moves.push(sq | (t << 8) | (BISHOP << 16));
                  moves.push(sq | (t << 8) | (KNIGHT << 16));
                }
              } else {
                moves.push(sq | (t << 8));
              }
            } else if (t === ep) {
              moves.push(sq | (t << 8) | (M_EP << 20));
            }
          }
        } else if (typ === KNIGHT || typ === KING) {
          const dirs = typ === KNIGHT ? KNIGHT_D : KING_D;
          for (let i = 0; i < 8; i++) {
            const t = sq + dirs[i];
            if (t & 0x88) continue;
            const q = b[t];
            if (!q) {
              if (!tactical) moves.push(sq | (t << 8));
            } else if ((q >> 3) === them) {
              moves.push(sq | (t << 8));
            }
          }
          if (typ === KING && !tactical) this.genCastles(sq, moves);
        } else {
          const dirs = typ === BISHOP ? BISHOP_D : typ === ROOK ? ROOK_D : KING_D;
          for (let i = 0; i < dirs.length; i++) {
            const d = dirs[i];
            let t = sq + d;
            while (!(t & 0x88)) {
              const q = b[t];
              if (!q) {
                if (!tactical) moves.push(sq | (t << 8));
              } else {
                if ((q >> 3) === them) moves.push(sq | (t << 8));
                break;
              }
              t += d;
            }
          }
        }
      }
      return moves;
    }

    genCastles(k, moves) {
      const us = this.side;
      const c = this.castle;
      if (c[us * 2] < 0 && c[us * 2 + 1] < 0) return;
      const base = us === WHITE ? 0 : 112;
      if ((k >> 4) !== (base >> 4)) return;
      const them = 1 - us;
      if (this.attacked(k, them)) return;
      const b = this.b;
      const rook = ROOK | (us << 3);
      for (const i of [0, 1]) {
        const rsq = c[us * 2 + i];
        if (rsq < 0 || b[rsq] !== rook) continue;
        const kt = base + (i === 0 ? 6 : 2);
        const rt = base + (i === 0 ? 5 : 3);
        const lo = Math.min(k, kt, rsq, rt), hi = Math.max(k, kt, rsq, rt);
        let free = true;
        for (let s = lo; s <= hi; s++) {
          if (b[s] && s !== k && s !== rsq) { free = false; break; }
        }
        if (!free) continue;
        // König und Turm anheben: der Weg des Königs darf nicht angegriffen sein.
        const kp = b[k];
        b[k] = 0;
        b[rsq] = 0;
        let safe = true;
        const step = kt > k ? 1 : -1;
        let s = k;
        while (s !== kt) {
          s += step;
          if (this.attacked(s, them)) { safe = false; break; }
        }
        b[k] = kp;
        b[rsq] = rook;
        if (safe) moves.push(k | (rsq << 8) | (M_CASTLE << 20));
      }
    }

    legalMoves() {
      const us = this.side;
      const res = [];
      for (const m of this.genMoves()) {
        this.make(m);
        if (!this.attacked(this.kings[us], 1 - us)) res.push(m);
        this.unmake();
      }
      return res;
    }

    hasLegalMove() {
      const us = this.side;
      for (const m of this.genMoves()) {
        this.make(m);
        const ok = !this.attacked(this.kings[us], 1 - us);
        this.unmake();
        if (ok) return true;
      }
      return false;
    }

    // --------------------------------------------------------- Zug ausführen
    make(m) {
      const b = this.b;
      const frm = m & 255, to = (m >> 8) & 255, flag = m >> 20;
      const us = this.side, them = 1 - us;
      const p = b[frm];
      const typ = p & 7;
      const cap = flag === M_CASTLE ? 0 : b[to];
      this.stack.push([m, p, cap, this.castle, this.ep, this.half, this.h1, this.h2, this.p1, this.p2,
        this.mg[0], this.mg[1], this.eg[0], this.eg[1], this.phase]);
      let h1 = this.h1 ^ Z_SIDE[0], h2 = this.h2 ^ Z_SIDE[1];
      if (this.ep >= 0) { h1 ^= Z_EP[this.ep & 7][0]; h2 ^= Z_EP[this.ep & 7][1]; }
      this.ep = -1;
      let half = this.half + 1;
      const mg = this.mg, eg = this.eg;
      if (flag === M_CASTLE) {
        const base = frm & 0x70;
        let kt, rt;
        if (to > frm) { kt = base + 6; rt = base + 5; } else { kt = base + 2; rt = base + 3; }
        const rp = b[to];
        b[frm] = 0; b[to] = 0; b[kt] = p; b[rt] = rp;
        h1 ^= Z_PIECE_HI[p][frm] ^ Z_PIECE_HI[p][kt] ^ Z_PIECE_HI[rp][to] ^ Z_PIECE_HI[rp][rt];
        h2 ^= Z_PIECE_LO[p][frm] ^ Z_PIECE_LO[p][kt] ^ Z_PIECE_LO[rp][to] ^ Z_PIECE_LO[rp][rt];
        mg[us] += PST_MG[p][kt] - PST_MG[p][frm] + PST_MG[rp][rt] - PST_MG[rp][to];
        eg[us] += PST_EG[p][kt] - PST_EG[p][frm] + PST_EG[rp][rt] - PST_EG[rp][to];
        this.kings[us] = kt;
      } else {
        if (flag === M_EP) {
          const cs = us === WHITE ? to - 16 : to + 16;
          const cp = b[cs];
          b[cs] = 0;
          h1 ^= Z_PIECE_HI[cp][cs]; h2 ^= Z_PIECE_LO[cp][cs];
          this.p1 ^= Z_PIECE_HI[cp][cs]; this.p2 ^= Z_PIECE_LO[cp][cs];
          mg[them] -= PST_MG[cp][cs];
          eg[them] -= PST_EG[cp][cs];
          this.counts[cp]--;
          half = 0;
        } else if (cap) {
          h1 ^= Z_PIECE_HI[cap][to]; h2 ^= Z_PIECE_LO[cap][to];
          mg[them] -= PST_MG[cap][to];
          eg[them] -= PST_EG[cap][to];
          this.phase -= PHASE_INC[cap & 7];
          this.counts[cap]--;
          if ((cap & 7) === PAWN) { this.p1 ^= Z_PIECE_HI[cap][to]; this.p2 ^= Z_PIECE_LO[cap][to]; }
          half = 0;
        }
        b[frm] = 0;
        h1 ^= Z_PIECE_HI[p][frm]; h2 ^= Z_PIECE_LO[p][frm];
        mg[us] -= PST_MG[p][frm];
        eg[us] -= PST_EG[p][frm];
        const promo = (m >> 16) & 7;
        let np;
        if (promo) {
          np = promo | (us << 3);
          this.counts[p]--;
          this.counts[np]++;
          this.phase += PHASE_INC[promo];
          this.p1 ^= Z_PIECE_HI[p][frm]; this.p2 ^= Z_PIECE_LO[p][frm];
        } else {
          np = p;
          if (typ === PAWN) {
            this.p1 ^= Z_PIECE_HI[p][frm] ^ Z_PIECE_HI[p][to];
            this.p2 ^= Z_PIECE_LO[p][frm] ^ Z_PIECE_LO[p][to];
          }
        }
        b[to] = np;
        h1 ^= Z_PIECE_HI[np][to]; h2 ^= Z_PIECE_LO[np][to];
        mg[us] += PST_MG[np][to];
        eg[us] += PST_EG[np][to];
        if (typ === PAWN) {
          half = 0;
          if (flag === M_DOUBLE) {
            // En-passant-Feld nur setzen, wenn ein Gegnerbauer daneben steht.
            const epPawn = PAWN | (them << 3);
            if ((!((to - 1) & 0x88) && b[to - 1] === epPawn) || (!((to + 1) & 0x88) && b[to + 1] === epPawn)) {
              this.ep = (frm + to) >> 1;
              h1 ^= Z_EP[this.ep & 7][0]; h2 ^= Z_EP[this.ep & 7][1];
            }
          }
        } else if (typ === KING) {
          this.kings[us] = to;
        }
      }
      const c = this.castle;
      if (c[0] >= 0 || c[1] >= 0 || c[2] >= 0 || c[3] >= 0) {
        const nc = c.slice();
        if (typ === KING) nc[us * 2] = nc[us * 2 + 1] = -1;
        for (let i = 0; i < 4; i++) {
          if (nc[i] >= 0 && (nc[i] === frm || nc[i] === to)) nc[i] = -1;
        }
        if (nc[0] !== c[0] || nc[1] !== c[1] || nc[2] !== c[2] || nc[3] !== c[3]) {
          const k1 = castleKey(c), k2 = castleKey(nc);
          h1 ^= k1[0] ^ k2[0];
          h2 ^= k1[1] ^ k2[1];
          this.castle = nc;
        }
      }
      this.h1 = h1; this.h2 = h2;
      this.half = half;
      if (us === BLACK) this.full++;
      this.side = them;
      this.hist.push(keyOf(h1, h2));
    }

    unmake() {
      const st = this.stack.pop();
      this.hist.pop();
      const m = st[0], p = st[1], cap = st[2];
      const b = this.b;
      const them = this.side, us = 1 - them;
      this.side = us;
      if (us === BLACK) this.full--;
      const frm = m & 255, to = (m >> 8) & 255, flag = m >> 20;
      if (flag === M_CASTLE) {
        const base = frm & 0x70;
        let kt, rt;
        if (to > frm) { kt = base + 6; rt = base + 5; } else { kt = base + 2; rt = base + 3; }
        const rp = b[rt];
        b[kt] = 0; b[rt] = 0; b[frm] = p; b[to] = rp;
        this.kings[us] = frm;
      } else {
        const promo = (m >> 16) & 7;
        if (promo) {
          this.counts[promo | (us << 3)]--;
          this.counts[p]++;
        }
        b[to] = cap;
        b[frm] = p;
        if (cap) this.counts[cap]++;
        if (flag === M_EP) {
          const cp = PAWN | (them << 3);
          b[us === WHITE ? to - 16 : to + 16] = cp;
          this.counts[cp]++;
        }
        if ((p & 7) === KING) this.kings[us] = frm;
      }
      this.castle = st[3];
      this.ep = st[4];
      this.half = st[5];
      this.h1 = st[6]; this.h2 = st[7]; this.p1 = st[8]; this.p2 = st[9];
      this.mg[0] = st[10]; this.mg[1] = st[11]; this.eg[0] = st[12]; this.eg[1] = st[13];
      this.phase = st[14];
    }

    makeNull() {
      this.stack.push([0, 0, 0, this.castle, this.ep, this.half, this.h1, this.h2]);
      let h1 = this.h1 ^ Z_SIDE[0], h2 = this.h2 ^ Z_SIDE[1];
      if (this.ep >= 0) { h1 ^= Z_EP[this.ep & 7][0]; h2 ^= Z_EP[this.ep & 7][1]; }
      this.ep = -1;
      this.h1 = h1; this.h2 = h2;
      this.half = 0; // über einen Null-Zug hinweg keine Wiederholung
      this.side = 1 - this.side;
      this.hist.push(keyOf(h1, h2));
    }

    unmakeNull() {
      const st = this.stack.pop();
      this.hist.pop();
      this.side = 1 - this.side;
      this.ep = st[4];
      this.half = st[5];
      this.h1 = st[6]; this.h2 = st[7];
    }

    // --------------------------------------------------------- Zustände
    isCapture(m) {
      const flag = m >> 20;
      return flag === M_EP || (flag !== M_CASTLE && this.b[(m >> 8) & 255] !== 0);
    }

    repetitions() {
      const h = this.hist[this.hist.length - 1];
      const hist = this.hist;
      let n = 0;
      const stop = Math.max(0, hist.length - 1 - this.half);
      for (let i = hist.length - 1; i >= stop; i -= 2) if (hist[i] === h) n++;
      return n;
    }

    insufficientMaterial() {
      const c = this.counts;
      if (c[1] || c[9] || c[4] || c[12] || c[5] || c[13]) return false;
      const minors = c[2] + c[3] + c[10] + c[11];
      if (minors <= 1) return true;
      if (c[2] || c[10]) return false;
      const colors = new Set();
      for (const sq of SQUARES) if ((this.b[sq] & 7) === BISHOP) colors.add(((sq >> 4) + (sq & 7)) & 1);
      return colors.size === 1;
    }

    canMate(color) {
      const c = this.counts, o = color << 3;
      if (c[PAWN | o] || c[ROOK | o] || c[QUEEN | o]) return true;
      return c[KNIGHT | o] + c[BISHOP | o] >= 2;
    }

    nonPawnMaterial(color) {
      const c = this.counts, o = color << 3;
      return c[KNIGHT | o] + c[BISHOP | o] + c[ROOK | o] + c[QUEEN | o];
    }

    material(color) {
      const c = this.counts, o = color << 3;
      return c[PAWN | o] + 3 * c[KNIGHT | o] + 3 * c[BISHOP | o] + 5 * c[ROOK | o] + 9 * c[QUEEN | o];
    }

    status() {
      if (!this.hasLegalMove()) return this.inCheck() ? "checkmate" : "stalemate";
      if (this.insufficientMaterial()) return "material";
      if (this.half >= 100) return "fifty";
      if (this.repetitions() >= 3) return "threefold";
      return null;
    }

    // --------------------------------------------------------- Bewertung
    /** Statische Bewertung aus Sicht der Seite am Zug (Zentibauern). */
    evaluate() {
      const b = this.b, c = this.counts;
      let mg = this.mg[0] - this.mg[1];
      let eg = this.eg[0] - this.eg[1];
      if (c[3] >= 2) { mg += BISHOP_PAIR[0]; eg += BISHOP_PAIR[1]; }
      if (c[11] >= 2) { mg -= BISHOP_PAIR[0]; eg -= BISHOP_PAIR[1]; }
      const pk = keyOf(this.p1, this.p2);
      let pe = pawnCache.get(pk);
      if (pe === undefined) pe = this.pawnEval(pk);
      mg += pe[0];
      eg += pe[1];
      const wf = pe[2], bf = pe[3];
      for (let si = 0; si < 64; si++) {
        const sq = SQUARES[si];
        const p = b[sq];
        if (!p) continue;
        const typ = p & 7;
        if (typ === PAWN || typ === KING) continue;
        const col = p >> 3;
        let n = 0;
        if (typ === KNIGHT) {
          for (let i = 0; i < 8; i++) {
            const t = sq + KNIGHT_D[i];
            if (!(t & 0x88)) {
              const q = b[t];
              if (!q || (q >> 3) !== col) n++;
            }
          }
        } else {
          const dirs = typ === BISHOP ? BISHOP_D : typ === ROOK ? ROOK_D : KING_D;
          for (let i = 0; i < dirs.length; i++) {
            const d = dirs[i];
            let t = sq + d;
            while (!(t & 0x88)) {
              const q = b[t];
              if (q) {
                if ((q >> 3) !== col) n++;
                break;
              }
              n++;
              t += d;
            }
          }
        }
        n -= MOB_BASE[typ];
        let dm = n * MOB_MG[typ];
        let de = n * MOB_EG[typ];
        if (typ === ROOK) {
          const f = sq & 7;
          const own = col === WHITE ? wf[f] : bf[f];
          if (own === 0) {
            if ((col === WHITE ? bf[f] : wf[f]) === 0) { dm += ROOK_OPEN[0]; de += ROOK_OPEN[1]; } else { dm += ROOK_HALF[0]; de += ROOK_HALF[1]; }
          }
        }
        if (col === WHITE) { mg += dm; eg += de; } else { mg -= dm; eg -= de; }
      }
      mg -= this.shieldPenalty(WHITE, wf, bf);
      mg += this.shieldPenalty(BLACK, wf, bf);
      if (this.phase <= 8) {
        const mw = this.material(WHITE), mb = this.material(BLACK);
        if (mw >= mb + 4 && c[9] === 0) eg += mopUp(this.kings[WHITE], this.kings[BLACK]);
        else if (mb >= mw + 4 && c[1] === 0) eg -= mopUp(this.kings[BLACK], this.kings[WHITE]);
      }
      const phase = this.phase < 24 ? this.phase : 24;
      const score = Math.floor((mg * phase + eg * (24 - phase)) / 24);
      return (this.side === WHITE ? score : -score) + TEMPO;
    }

    shieldPenalty(color, wf, bf) {
      const k = this.kings[color];
      const r = k >> 4;
      const rel = color === WHITE ? r : 7 - r;
      if (rel > 1) return 0;
      const enemyQueen = this.counts[QUEEN | ((1 - color) << 3)];
      const b = this.b;
      const pawn = PAWN | (color << 3);
      const fwd = color === WHITE ? 16 : -16;
      const f = k & 7;
      let pen = 0;
      for (const ff of [f - 1, f, f + 1]) {
        if (ff < 0 || ff > 7) continue;
        const s1 = (k - f + ff) + fwd;
        if (b[s1] === pawn) { pen += SHIELD_NEAR; continue; }
        const s2 = s1 + fwd;
        if (!(s2 & 0x88) && b[s2] === pawn) { pen += SHIELD_FAR; continue; }
        pen += SHIELD_NONE;
        if (wf[ff] === 0 && bf[ff] === 0) pen += SHIELD_OPEN;
      }
      return enemyQueen ? pen : Math.floor(pen / 2);
    }

    pawnEval(pk) {
      const b = this.b;
      const wf = [0, 0, 0, 0, 0, 0, 0, 0], bf = [0, 0, 0, 0, 0, 0, 0, 0];
      const wmin = [8, 8, 8, 8, 8, 8, 8, 8], bmax = [-1, -1, -1, -1, -1, -1, -1, -1];
      const wps = [], bps = [];
      for (const sq of SQUARES) {
        const p = b[sq];
        if (p === 1) {
          const f = sq & 7;
          wf[f]++;
          if ((sq >> 4) < wmin[f]) wmin[f] = sq >> 4;
          wps.push(sq);
        } else if (p === 9) {
          const f = sq & 7;
          bf[f]++;
          if ((sq >> 4) > bmax[f]) bmax[f] = sq >> 4;
          bps.push(sq);
        }
      }
      let mg = 0, eg = 0;
      for (let f = 0; f < 8; f++) {
        if (wf[f] > 1) { mg -= DOUBLED[0] * (wf[f] - 1); eg -= DOUBLED[1] * (wf[f] - 1); }
        if (bf[f] > 1) { mg += DOUBLED[0] * (bf[f] - 1); eg += DOUBLED[1] * (bf[f] - 1); }
      }
      for (const sq of wps) {
        const f = sq & 7, r = sq >> 4;
        if ((f === 0 || wf[f - 1] === 0) && (f === 7 || wf[f + 1] === 0)) { mg -= ISOLATED[0]; eg -= ISOLATED[1]; }
        let passed = true;
        for (const ff of [f - 1, f, f + 1]) {
          if (ff >= 0 && ff <= 7 && bmax[ff] > r) { passed = false; break; }
        }
        if (passed) { mg += PASSED_MG[r]; eg += PASSED_EG[r]; }
      }
      for (const sq of bps) {
        const f = sq & 7, r = sq >> 4;
        if ((f === 0 || bf[f - 1] === 0) && (f === 7 || bf[f + 1] === 0)) { mg += ISOLATED[0]; eg += ISOLATED[1]; }
        let passed = true;
        for (const ff of [f - 1, f, f + 1]) {
          if (ff >= 0 && ff <= 7 && wmin[ff] < r) { passed = false; break; }
        }
        if (passed) { mg -= PASSED_MG[7 - r]; eg -= PASSED_EG[7 - r]; }
      }
      const entry = [mg, eg, wf, bf];
      if (pawnCache.size > 50000) pawnCache.clear();
      pawnCache.set(pk, entry);
      return entry;
    }

    // --------------------------------------------------------- Notation
    fen() {
      const rows = [];
      for (let r = 7; r >= 0; r--) {
        let row = "", empty = 0;
        for (let f = 0; f < 8; f++) {
          const p = this.b[r * 16 + f];
          if (!p) empty++;
          else {
            if (empty) { row += empty; empty = 0; }
            row += pieceChar(p);
          }
        }
        if (empty) row += empty;
        rows.push(row);
      }
      let cr = "";
      "KQkq".split("").forEach((ch, i) => {
        const rsq = this.castle[i];
        if (rsq < 0) return;
        const color = i >> 1;
        if (this.outermostRook(color, i % 2 === 0) === rsq) cr += ch;
        else cr += color === WHITE ? FILES[rsq & 7].toUpperCase() : FILES[rsq & 7];
      });
      return rows.join("/") + " " + (this.side === WHITE ? "w" : "b") + " " + (cr || "-") + " " +
        (this.ep >= 0 ? sqName(this.ep) : "-") + " " + this.half + " " + this.full;
    }

    outermostRook(color, kingside) {
      const base = color === WHITE ? 0 : 112;
      const kf = this.kings[color] & 7;
      const rook = ROOK | (color << 3);
      if (kingside) {
        for (let f = 7; f > kf; f--) if (this.b[base + f] === rook) return base + f;
      } else {
        for (let f = 0; f < kf; f++) if (this.b[base + f] === rook) return base + f;
      }
      return -1;
    }

    uci(m) {
      const frm = m & 255;
      let to = (m >> 8) & 255;
      if (m >> 20 === M_CASTLE && !this.chess960) to = (frm & 0x70) + (to > frm ? 6 : 2);
      const promo = (m >> 16) & 7;
      return sqName(frm) + sqName(to) + (promo ? LETTERS[promo].toLowerCase() : "");
    }

    parseUci(text) {
      text = String(text).trim().toLowerCase();
      for (const m of this.legalMoves()) {
        if (this.uci(m) === text) return m;
        if (m >> 20 === M_CASTLE) {
          const frm = m & 255, to = (m >> 8) & 255;
          const kt = (frm & 0x70) + (to > frm ? 6 : 2);
          if (text === sqName(frm) + sqName(to) || text === sqName(frm) + sqName(kt)) return m;
        }
      }
      return 0;
    }

    san(m, legal) {
      const b = this.b;
      const frm = m & 255, to = (m >> 8) & 255, flag = m >> 20;
      const p = b[frm];
      const typ = p & 7;
      let s;
      if (flag === M_CASTLE) {
        s = to > frm ? "O-O" : "O-O-O";
      } else {
        const capture = flag === M_EP || b[to] !== 0;
        const promo = (m >> 16) & 7;
        if (typ === PAWN) {
          s = (capture ? FILES[frm & 7] + "x" : "") + sqName(to);
          if (promo) s += "=" + LETTERS[promo];
        } else {
          if (!legal) legal = this.legalMoves();
          let sameFile = false, sameRank = false, ambiguous = false;
          for (const o of legal) {
            const of = o & 255;
            if (of === frm || ((o >> 8) & 255) !== to || o >> 20 === M_CASTLE) continue;
            if (b[of] !== p) continue;
            ambiguous = true;
            if ((of & 7) === (frm & 7)) sameFile = true;
            if ((of >> 4) === (frm >> 4)) sameRank = true;
          }
          let dis = "";
          if (ambiguous) {
            if (!sameFile) dis = FILES[frm & 7];
            else if (!sameRank) dis = String((frm >> 4) + 1);
            else dis = sqName(frm);
          }
          s = LETTERS[typ] + dis + (capture ? "x" : "") + sqName(to);
        }
      }
      this.make(m);
      if (this.inCheck()) s += this.hasLegalMove() ? "+" : "#";
      this.unmake();
      return s;
    }

    parseSan(text) {
      let t = String(text).trim().replace("0-0-0", "O-O-O").replace("0-0", "O-O");
      t = t.replace(/[+#!?]+$/, "");
      const legal = this.legalMoves();
      for (const m of legal) {
        if (this.san(m, legal).replace(/[+#]$/, "") === t) return m;
      }
      return 0;
    }
  }

  function mopUp(winK, loseK) {
    const lf = loseK & 7, lr = loseK >> 4;
    const center = Math.max(3 - lf, lf - 4) + Math.max(3 - lr, lr - 4);
    const dist = Math.abs((winK & 7) - lf) + Math.abs((winK >> 4) - lr);
    return 10 * center + 4 * (14 - dist);
  }

  // ============================================================ Startstellungen
  const START_FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
  const KRN = [[0, 1], [0, 2], [0, 3], [0, 4], [1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]];

  /** Grundreihe (a..h) der Chess960-Stellung Nr. n (0..959, 518 = normal). */
  function chess960Backrank(n) {
    n = ((n % 960) + 960) % 960;
    const rank = ["", "", "", "", "", "", "", ""];
    const b1 = n % 4; n = Math.floor(n / 4);
    rank[b1 * 2 + 1] = "B";
    const b2 = n % 4; n = Math.floor(n / 4);
    rank[b2 * 2] = "B";
    const q = n % 6; n = Math.floor(n / 6);
    let empty = [0, 1, 2, 3, 4, 5, 6, 7].filter((i) => !rank[i]);
    rank[empty[q]] = "Q";
    empty = [0, 1, 2, 3, 4, 5, 6, 7].filter((i) => !rank[i]);
    rank[empty[KRN[n][0]]] = "N";
    rank[empty[KRN[n][1]]] = "N";
    empty = [0, 1, 2, 3, 4, 5, 6, 7].filter((i) => !rank[i]);
    rank[empty[0]] = "R";
    rank[empty[1]] = "K";
    rank[empty[2]] = "R";
    return rank.join("");
  }

  function chess960Fen(n) {
    const back = chess960Backrank(n);
    return back.toLowerCase() + "/pppppppp/8/8/8/8/PPPPPPPP/" + back + " w KQkq - 0 1";
  }

  /** Position aus FEN (X-FEN/Shredder-FEN erlaubt). */
  function fromFen(fen, chess960) {
    const parts = String(fen).trim().split(/\s+/);
    const pos = new Position();
    const rows = parts[0].split("/");
    if (rows.length !== 8) throw new Error("FEN: 8 Reihen erwartet");
    rows.forEach((row, i) => {
      const r = 7 - i;
      let f = 0;
      for (const ch of row) {
        if (ch >= "0" && ch <= "9") f += Number(ch);
        else {
          const typ = LETTERS.indexOf(ch.toUpperCase());
          if (typ <= 0 || f > 7) throw new Error("FEN: ungültige Figur");
          pos.b[r * 16 + f] = typ | (ch === ch.toLowerCase() ? 8 : 0);
          f++;
        }
      }
      if (f !== 8) throw new Error("FEN: Reihe hat nicht 8 Felder");
    });
    pos.side = parts[1] === "b" ? BLACK : WHITE;
    for (const sq of SQUARES) {
      const p = pos.b[sq];
      if ((p & 7) === KING) pos.kings[p >> 3] = sq;
    }
    if (pos.kings[0] < 0 || pos.kings[1] < 0) throw new Error("FEN: König fehlt");
    const castle = [-1, -1, -1, -1];
    let std = true;
    for (const ch of parts[2] || "-") {
      if (ch === "-") continue;
      const color = ch === ch.toUpperCase() ? WHITE : BLACK;
      const base = color === WHITE ? 0 : 112;
      const kf = pos.kings[color] & 7;
      if (ch.toUpperCase() === "K") {
        const rsq = pos.outermostRook(color, true);
        if (rsq >= 0) castle[color * 2] = rsq;
      } else if (ch.toUpperCase() === "Q") {
        const rsq = pos.outermostRook(color, false);
        if (rsq >= 0) castle[color * 2 + 1] = rsq;
      } else if (FILES.includes(ch.toLowerCase())) {
        const f = FILES.indexOf(ch.toLowerCase());
        if (pos.b[base + f] === (ROOK | (color << 3))) castle[color * 2 + (f > kf ? 0 : 1)] = base + f;
        std = false;
      }
    }
    for (let i = 0; i < 4; i++) {
      const color = i >> 1;
      const base = color === WHITE ? 0 : 112;
      if (castle[i] >= 0 && (pos.kings[color] >> 4) !== (base >> 4)) castle[i] = -1;
      if (castle[i] >= 0 && (pos.kings[color] !== base + 4 || (castle[i] !== base && castle[i] !== base + 7))) std = false;
    }
    pos.castle = castle;
    if (parts[3] && parts[3] !== "-") {
      const ep = sqParse(parts[3]);
      const us = pos.side;
      const capPawn = PAWN | (us << 3);
      let ok = false;
      for (const d of us === WHITE ? [-15, -17] : [15, 17]) {
        const s = ep + d;
        if (!(s & 0x88) && pos.b[s] === capPawn) ok = true;
      }
      pos.ep = ok ? ep : -1;
    }
    pos.half = parts[4] ? Number(parts[4]) : 0;
    pos.full = parts[5] ? Number(parts[5]) : 1;
    pos.chess960 = chess960 == null ? !std : !!chess960;
    pos.recompute();
    return pos;
  }

  function perft(pos, depth) {
    if (depth === 0) return 1;
    const us = pos.side;
    let n = 0;
    for (const m of pos.genMoves()) {
      pos.make(m);
      if (!pos.attacked(pos.kings[us], 1 - us)) n += depth === 1 ? 1 : perft(pos, depth - 1);
      pos.unmake();
    }
    return n;
  }

  // ================================================================== PGN
  function pgnResultCode(winner) {
    if (winner === "*") return "*";
    if (winner === null || winner === undefined) return "1/2-1/2";
    return winner === WHITE ? "1-0" : "0-1";
  }

  function pgnText(tags, sans, result, startSide = WHITE, startFull = 1) {
    const lines = tags.map(([k, v]) => "[" + k + ' "' + String(v).replace(/\\/g, "\\\\").replace(/"/g, '\\"') + '"]');
    const tokens = [];
    let full = startFull, side = startSide;
    sans.forEach((s, i) => {
      if (side === WHITE) tokens.push(full + ".");
      else if (i === 0) tokens.push(full + "...");
      tokens.push(s);
      if (side === BLACK) full++;
      side = 1 - side;
    });
    tokens.push(result);
    const body = [];
    let line = "";
    for (const tok of tokens) {
      if (line && line.length + 1 + tok.length > 79) {
        body.push(line);
        line = tok;
      } else line = line ? line + " " + tok : tok;
    }
    if (line) body.push(line);
    return lines.join("\n") + "\n\n" + body.join("\n") + "\n";
  }

  // =========================================================== Eröffnungsbuch
  const BOOK_LINES = [
    "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6",
    "e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 d2d4 e4d6 b5c6 d7c6 d4e5 d6f5",
    "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d3 d7d6 e1g1 e8g8",
    "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d3 f8e7 e1g1 e8g8 f1e1 d7d6",
    "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5 d8e7",
    "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4 d6d5 f1d3 b8c6",
    "e2e4 e7e5 b1c3 g8f6 g1f3 b8c6 f1b5 f8b4 e1g1 e8g8",
    "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1e3 e7e5 d4b3 c8e6",
    "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g8f6 b1c3 e7e5 d4b5 d7d6",
    "e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6 f1d3 g8f6 e1g1 d8c7",
    "e2e4 c7c5 g1f3 d7d6 f1b5 c8d7 b5d7 d8d7 e1g1 b8c6 c2c3 g8f6",
    "e2e4 c7c5 b1c3 b8c6 g2g3 g7g6 f1g2 f8g7 d2d3 d7d6",
    "e2e4 c7c5 c2c3 g8f6 e4e5 f6d5 d2d4 c5d4 g1f3 b8c6",
    "e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 c1g5 f8e7 e4e5 f6d7 g5e7 d8e7",
    "e2e4 e7e6 d2d4 d7d5 e4e5 c7c5 c2c3 b8c6 g1f3 d8b6 a2a3 c5c4",
    "e2e4 e7e6 d2d4 d7d5 b1d2 g8f6 e4e5 f6d7 f1d3 c7c5 c2c3 b8c6",
    "e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6",
    "e2e4 c7c6 d2d4 d7d5 e4e5 c8f5 g1f3 e7e6 f1e2 c6c5 e1g1 b8c6",
    "e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 g1f3 f8g7 f1e2 e8g8 e1g1 c7c6",
    "e2e4 d7d5 e4d5 d8d5 b1c3 d5a5 d2d4 g8f6 g1f3 c8f5 f1c4 e7e6",
    "e2e4 g8f6 e4e5 f6d5 d2d4 d7d6 g1f3 c8g4 f1e2 e7e6 e1g1 f8e7",
    "e2e4 g7g6 d2d4 f8g7 b1c3 d7d6 g1f3 g8f6 f1e2 e8g8 e1g1 c7c6",
    "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 f8e7 e2e3 e8g8 g1f3 b8d7",
    "d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 d5c4 a2a4 c8f5 e2e3 e7e6",
    "d2d4 d7d5 c2c4 d5c4 g1f3 g8f6 e2e3 e7e6 f1c4 c7c5 e1g1 a7a6",
    "d2d4 d7d5 c2c4 e7e6 g1f3 g8f6 g2g3 f8e7 f1g2 e8g8 e1g1 d5c4",
    "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5",
    "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3 e8g8 f1d3 d7d5 g1f3 c7c5",
    "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3 c8a6 b2b3 f8b4 c1d2 b4e7",
    "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5 e2e4 d5c3 b2c3 f8g7",
    "d2d4 g8f6 c2c4 c7c5 d4d5 e7e6 b1c3 e6d5 c4d5 d7d6 e2e4 g7g6",
    "d2d4 g8f6 c2c4 e7e6 g2g3 d7d5 f1g2 f8e7 g1f3 e8g8 e1g1 d5c4",
    "d2d4 d7d5 g1f3 g8f6 c1f4 e7e6 e2e3 c7c5 c2c3 b8c6 b1d2 f8d6",
    "d2d4 g8f6 g1f3 e7e6 c1g5 c7c5 e2e3 b7b6 b1d2 c8b7",
    "d2d4 f7f5 g2g3 g8f6 f1g2 g7g6 g1f3 f8g7 e1g1 e8g8 c2c4 d7d6",
    "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 g2g3 d7d5 c4d5 f6d5 f1g2 d5b6",
    "c2c4 g8f6 b1c3 e7e6 g1f3 d7d5 d2d4 f8e7 c1f4 e8g8 e2e3 c7c5",
    "c2c4 c7c5 g1f3 b8c6 b1c3 g7g6 g2g3 f8g7 f1g2 g8f6 e1g1 e8g8",
    "g1f3 d7d5 g2g3 g8f6 f1g2 e7e6 e1g1 f8e7 d2d3 e8g8 b1d2 c7c5",
    "g1f3 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 d2d4 e8g8 f1e2 e7e5",
  ];

  function bookMoves(historyUci) {
    const n = historyUci.length;
    const out = [];
    for (const line of BOOK_LINES) {
      const seq = line.split(" ");
      if (seq.length > n && historyUci.every((u, i) => seq[i] === u)) out.push(seq[n]);
    }
    return out;
  }

  // ================================================================== KI
  const LEVELS = [
    { depth: 1, time: 0.05, nodes: 2500, quiesce: false, random: 1.0, spread: 0, book: false },
    { depth: 1, time: 0.15, nodes: 2500, quiesce: false, random: 0.3, spread: 120, book: false },
    { depth: 2, time: 0.25, nodes: 6000, quiesce: true, random: 0.1, spread: 50, book: true },
    { depth: 3, time: 0.4, nodes: 15000, quiesce: true, random: 0.03, spread: 20, book: true },
    { depth: 5, time: 0.8, nodes: 40000, quiesce: true, random: 0.0, spread: 0, book: true },
    { depth: 32, time: 1.4, nodes: 120000, quiesce: true, random: 0.0, spread: 0, book: true },
  ];
  const HINT_LEVEL = { depth: 32, time: 0.9, nodes: 60000, quiesce: true, random: 0.0, spread: 0, book: false };
  const TT_MAX = 200000;

  /** Stabile absteigende Sortierung von Indizes nach Wertung (wie Pythons sorted(..., reverse=True)). */
  function orderIdx(scores) {
    const idx = scores.map((_, i) => i);
    idx.sort((a, b) => scores[b] - scores[a] || a - b);
    return idx;
  }

  /**
   * Fortsetzbare Suche:
   *   const s = new Search(pos, LEVELS[5]); const gen = s.run();
   *   s.sliceEnd = now + 0.007; gen.next();  // jedes Frame, bis done
   *   s.bestMove
   */
  class Search {
    constructor(pos, level, opts = {}) {
      this.pos = pos.copy();
      this.level = level;
      this.maxDepth = opts.maxDepth || level.depth;
      this.nodeLimit = opts.nodeLimit != null ? opts.nodeLimit : level.nodes;
      this.quiesce = level.quiesce;
      this.spread = level.spread;
      this.tt = opts.tt || new Map();
      this.rng = opts.rng || Math.random;
      this.nodes = 0;
      this.sliceEnd = Infinity;
      this.stop = false;
      this.done = false;
      this.bestMove = 0;
      this.bestScore = 0;
      this.depthDone = 0;
      this.rootScores = new Map();
      this.killers = [];
      for (let i = 0; i < MAX_PLY + 2; i++) this.killers.push([0, 0]);
      this.history = new Float64Array(16 * 128);
      this.checkEvery = 255;
    }

    *run() {
      const pos = this.pos;
      const moves = pos.legalMoves();
      if (!moves.length) {
        this.done = true;
        return;
      }
      this.bestMove = moves[0];
      if (moves.length === 1 && !this.spread) {
        this.done = true;
        return;
      }
      const scores = moves.map((m) => this.orderScore(m, 0, 0));
      this.rootMoves = orderIdx(scores).map((i) => moves[i]);
      for (let depth = 1; depth <= this.maxDepth; depth++) {
        const score = yield* this.root(depth);
        if (this.stop) break;
        this.depthDone = depth;
        this.bestScore = score;
        if (Math.abs(score) >= MATE - MAX_PLY && depth >= 2 && !this.spread) break;
      }
      if (this.spread && this.rootScores.size) {
        let best = -Infinity;
        for (const v of this.rootScores.values()) best = Math.max(best, v);
        const good = this.rootMoves.filter((m) => (this.rootScores.has(m) ? this.rootScores.get(m) : -INF) >= best - this.spread);
        if (good.length) this.bestMove = good[Math.floor(this.rng() * good.length)];
      }
      this.done = true;
    }

    *root(depth) {
      const pos = this.pos;
      const exact = this.spread > 0;
      let alpha = -INF;
      const beta = INF;
      let best = -INF, bestMove = 0;
      const scored = new Map();
      for (let i = 0; i < this.rootMoves.length; i++) {
        const m = this.rootMoves[i];
        pos.make(m);
        let score;
        if (i === 0 || exact) {
          score = -(yield* this.negamax(depth - 1, -beta, exact ? INF : -alpha, 1, true));
        } else {
          score = -(yield* this.negamax(depth - 1, -alpha - 1, -alpha, 1, true));
          if (score > alpha && !this.stop) score = -(yield* this.negamax(depth - 1, -beta, -alpha, 1, true));
        }
        pos.unmake();
        if (this.stop) break;
        scored.set(m, score);
        if (score > best) {
          best = score;
          bestMove = m;
          if (!exact) {
            this.bestMove = m;
            this.bestScore = score;
          }
          if (score > alpha && !exact) alpha = score;
        }
      }
      if (!this.stop || exact) for (const [m, s] of scored) this.rootScores.set(m, s);
      if (bestMove) {
        const keys = this.rootMoves.map((m) => (scored.has(m) ? scored.get(m) : -INF - 1));
        this.rootMoves = orderIdx(keys).map((i) => this.rootMoves[i]);
        if (exact && !this.stop) {
          this.bestMove = bestMove;
          this.bestScore = best;
        }
      }
      return best;
    }

    orderScore(m, ttMove, ply) {
      if (m === ttMove) return 4000000;
      const b = this.pos.b;
      const frm = m & 255, to = (m >> 8) & 255, flag = m >> 20, promo = (m >> 16) & 7;
      if (flag === M_EP) return 2000000 + 10 - 1;
      if (flag !== M_CASTLE) {
        const cap = b[to];
        if (cap) return 2000000 + ORDER_VALUE[cap & 7] * 10 - ORDER_VALUE[b[frm] & 7] + promo;
      }
      if (promo) return 1900000 + promo;
      const k = this.killers[ply];
      if (m === k[0]) return 1800000;
      if (m === k[1]) return 1700000;
      return this.history[b[frm] * 128 + to];
    }

    *negamax(depth, alpha, beta, ply, canNull) {
      this.nodes++;
      if (this.nodes >= this.nodeLimit) this.stop = true;
      else if (!(this.nodes & this.checkEvery) && now() > this.sliceEnd) yield;
      if (this.stop) return 0;
      const pos = this.pos;
      if (pos.half >= 100 || pos.repetitions() >= 2 || pos.insufficientMaterial()) return 0;
      if (alpha < -MATE + ply) alpha = -MATE + ply;
      if (beta > MATE - ply - 1) beta = MATE - ply - 1;
      if (alpha >= beta) return alpha;
      const us = pos.side, them = 1 - us;
      const inCheck = pos.attacked(pos.kings[us], them);
      if (inCheck) depth++;
      if (depth <= 0 || ply >= MAX_PLY) {
        if (this.quiesce && ply < MAX_PLY) return yield* this.quiesceSearch(alpha, beta, ply);
        return pos.evaluate();
      }
      const pv = beta - alpha > 1;
      const key = pos.hist[pos.hist.length - 1];
      const entry = this.tt.get(key);
      let ttMove = 0;
      if (entry !== undefined) {
        ttMove = entry[3];
        if (!pv && entry[0] >= depth) {
          let es = entry[2];
          if (es > MATE - MAX_PLY) es -= ply;
          else if (es < -MATE + MAX_PLY) es += ply;
          const ef = entry[1];
          if (ef === 0 || (ef === 1 && es >= beta) || (ef === 2 && es <= alpha)) return es;
        }
      }
      let stat = 0;
      if (!pv && !inCheck) {
        stat = pos.evaluate();
        if (depth <= 3 && Math.abs(beta) < MATE - MAX_PLY && stat - 90 * depth >= beta) return stat;
        if (canNull && depth >= 3 && stat >= beta && pos.nonPawnMaterial(us) > 0) {
          const r = depth < 6 ? 2 : 3;
          pos.makeNull();
          const score = -(yield* this.negamax(depth - 1 - r, -beta, -beta + 1, ply + 1, false));
          pos.unmakeNull();
          if (this.stop) return 0;
          if (score >= beta) return score >= MATE - MAX_PLY ? beta : score;
        }
      }
      const moves = pos.genMoves();
      const scores = moves.map((m) => this.orderScore(m, ttMove, ply));
      const order = orderIdx(scores);
      const b = pos.b;
      let best = -INF, bestMove = 0, legal = 0;
      const origAlpha = alpha;
      const futile = !pv && !inCheck && depth <= 2 && Math.abs(alpha) < MATE - MAX_PLY && stat + 120 * depth <= alpha;
      for (let oi = 0; oi < order.length; oi++) {
        const i = order[oi];
        const m = moves[i];
        const quiet = scores[i] < 1700000;
        if (futile && quiet && legal > 0) continue;
        pos.make(m);
        if (pos.attacked(pos.kings[us], them)) {
          pos.unmake();
          continue;
        }
        legal++;
        let score;
        if (legal === 1) {
          score = -(yield* this.negamax(depth - 1, -beta, -alpha, ply + 1, true));
        } else {
          let red = 0;
          if (depth >= 3 && quiet && legal > 3 && !inCheck) red = legal < 10 || depth < 6 ? 1 : 2;
          score = -(yield* this.negamax(depth - 1 - red, -alpha - 1, -alpha, ply + 1, true));
          if (score > alpha && !this.stop && (red || score < beta)) {
            score = -(yield* this.negamax(depth - 1, -beta, -alpha, ply + 1, true));
          }
        }
        pos.unmake();
        if (this.stop) return 0;
        if (score > best) {
          best = score;
          bestMove = m;
          if (score > alpha) {
            alpha = score;
            if (score >= beta) {
              if (quiet) {
                const k = this.killers[ply];
                if (k[0] !== m) {
                  k[1] = k[0];
                  k[0] = m;
                }
                const hi = b[m & 255] * 128 + ((m >> 8) & 255);
                this.history[hi] += depth * depth;
                if (this.history[hi] > 1000000) {
                  for (let j = 0; j < this.history.length; j++) this.history[j] = Math.floor(this.history[j] / 2);
                }
              }
              break;
            }
          }
        }
      }
      if (legal === 0) return inCheck ? -MATE + ply : 0;
      const flag = best >= beta ? 1 : best > origAlpha ? 0 : 2;
      let store = best;
      if (store > MATE - MAX_PLY) store += ply;
      else if (store < -MATE + MAX_PLY) store -= ply;
      if (this.tt.size >= TT_MAX) this.tt.clear();
      this.tt.set(key, [depth, flag, store, bestMove]);
      return best;
    }

    *quiesceSearch(alpha, beta, ply) {
      this.nodes++;
      if (this.nodes >= this.nodeLimit) this.stop = true;
      else if (!(this.nodes & this.checkEvery) && now() > this.sliceEnd) yield;
      if (this.stop) return 0;
      const pos = this.pos;
      const stand = pos.evaluate();
      if (ply >= MAX_PLY) return stand;
      if (stand >= beta) return stand;
      if (stand > alpha) alpha = stand;
      const us = pos.side, them = 1 - us;
      const b = pos.b;
      const moves = pos.genMoves(true);
      const scores = moves.map((m) => this.orderScore(m, 0, ply));
      const order = orderIdx(scores);
      for (let oi = 0; oi < order.length; oi++) {
        const m = moves[order[oi]];
        const to = (m >> 8) & 255;
        if (!((m >> 16) & 7)) {
          const cap = b[to];
          const gain = cap ? MG_VALUE[cap & 7] : MG_VALUE[PAWN];
          if (stand + gain + 200 < alpha) continue; // Delta-Schnitt
        }
        pos.make(m);
        if (pos.attacked(pos.kings[us], them)) {
          pos.unmake();
          continue;
        }
        const score = -(yield* this.quiesceSearch(-beta, -alpha, ply + 1));
        pos.unmake();
        if (this.stop) return 0;
        if (score >= beta) return score;
        if (score > alpha) alpha = score;
      }
      return alpha;
    }
  }

  function runToEnd(search) {
    const gen = search.run();
    while (!gen.next().done) { /* rechnen */ }
    return search.bestMove;
  }

  /** Anfänger-Stufe: zufälliger Zug, Schlagzüge bevorzugt. */
  function pickWeakMove(pos, rng = Math.random) {
    const moves = pos.legalMoves();
    if (!moves.length) return 0;
    const caps = moves.filter((m) => pos.isCapture(m));
    if (caps.length && rng() < 0.6) return caps[Math.floor(rng() * caps.length)];
    return moves[Math.floor(rng() * moves.length)];
  }

  function givesMate(pos, m) {
    pos.make(m);
    const mate = pos.inCheck() && !pos.hasLegalMove();
    pos.unmake();
    return mate;
  }

  function forcedMate(pos, n) {
    const us = pos.side;
    const cands = [];
    for (const m of pos.genMoves()) {
      pos.make(m);
      if (pos.attacked(pos.kings[us], 1 - us)) {
        pos.unmake();
        continue;
      }
      const check = pos.inCheck();
      pos.unmake();
      if (n === 1 && !check) continue;
      cands.push([check ? 0 : pos.isCapture(m) ? 1 : 2, cands.length, m]);
    }
    cands.sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    for (const [, , m] of cands) {
      pos.make(m);
      const replies = pos.legalMoves();
      if (!replies.length) {
        const mate = pos.inCheck();
        pos.unmake();
        if (mate) return true;
        continue;
      }
      if (n === 1) {
        pos.unmake();
        continue;
      }
      let ok = true;
      for (const r of replies) {
        pos.make(r);
        const sub = forcedMate(pos, n - 1);
        pos.unmake();
        if (!sub) {
          ok = false;
          break;
        }
      }
      pos.unmake();
      if (ok) return true;
    }
    return false;
  }

  PG.chessEngine = {
    WHITE, BLACK, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, M_NORMAL, M_DOUBLE, M_EP, M_CASTLE,
    SQUARES, FILES, LETTERS, MATE, INF, ORDER_VALUE, START_FEN, BOOK_LINES, LEVELS, HINT_LEVEL,
    Position, Search, fromFen, perft, chess960Backrank, chess960Fen, pgnText, pgnResultCode, bookMoves,
    runToEnd, pickWeakMove, givesMate, forcedMate, sqName, sqParse, now,
  };
})();
