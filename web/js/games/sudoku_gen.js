/*
 * sudoku_gen.js - Deterministischer Sudoku-Generator + Löser (Port von games/sudoku_gen.py)
 * =========================================================================================
 * Level-System
 * ------------
 * Es gibt LEVELS (100) Level je Schwierigkeitsgrad. Level N von Stufe D ist
 * IMMER dasselbe Puzzle: alle Zufallsentscheidungen (Lösungs-Aufbau und
 * Loch-Reihenfolge) kommen aus einem einzigen new PG.Random(seedFor(D, N)).
 * (Die Rätsel weichen von der Python-Version ab, sind aber je Level stabil
 * und eindeutig lösbar.)
 *
 * Ablauf von generate:
 * 1. fill : volle, gültige Lösung per randomisiertem Backtracking.
 * 2. dig  : Zellen in zufälliger Reihenfolge leeren; eine Zelle bleibt
 *           nur leer, wenn das Puzzle EINDEUTIG lösbar bleibt
 *           (countSolutions mit Abbruch bei 2 Lösungen).
 *
 * Alle Bretter sind flache Arrays mit 81 Einträgen (Index = zeile*9+spalte,
 * 0 = leer, 1..9 = Ziffer).
 */
(function () {
  "use strict";

  const LEVELS = 100;

  // Ziel-Anzahl an Vorgaben je Schwierigkeitsgrad (Leicht/Normal/Schwer/Experte).
  const CLUES = [42, 34, 29, 25];

  const SEED_BASE = 987654321;

  // ----- Index-Tabellen ------------------------------------------------------
  const ROW_OF = [];
  const COL_OF = [];
  const BOX_OF = [];
  for (let i = 0; i < 81; i++) {
    ROW_OF.push(Math.floor(i / 9));
    COL_OF.push(i % 9);
    BOX_OF.push(Math.floor(i / 27) * 3 + Math.floor((i % 9) / 3));
  }

  // 27 Einheiten (9 Zeilen, 9 Spalten, 9 Boxen) mit je 9 Zell-Indizes.
  const UNITS = [];
  for (let r = 0; r < 9; r++) UNITS.push(Array.from({ length: 9 }, (_, c) => r * 9 + c));
  for (let c = 0; c < 9; c++) UNITS.push(Array.from({ length: 9 }, (_, r) => r * 9 + c));
  for (let br = 0; br < 3; br++) {
    for (let bc = 0; bc < 3; bc++) {
      const u = [];
      for (let r = 0; r < 3; r++) for (let c = 0; c < 3; c++) u.push((br * 3 + r) * 9 + bc * 3 + c);
      UNITS.push(u);
    }
  }

  // Die 20 "Peers" jeder Zelle (gleiche Zeile/Spalte/Box, ohne sich selbst).
  const PEERS = [];
  for (let i = 0; i < 81; i++) {
    const s = new Set();
    for (const u of UNITS) if (u.includes(i)) for (const j of u) if (j !== i) s.add(j);
    PEERS.push(Array.from(s));
  }

  const ALL = 0x1ff; // Bitmaske: alle 9 Kandidaten (Bit d-1 = Ziffer d)

  // Anzahl gesetzter Bits je 9-Bit-Maske (ersetzt int.bit_count()).
  const POPCOUNT = new Uint8Array(512);
  for (let m = 1; m < 512; m++) POPCOUNT[m] = POPCOUNT[m >> 1] + (m & 1);

  /** Stabiler int-Seed für (Schwierigkeitsgrad, Level). */
  function seedFor(diff, level) {
    return SEED_BASE + Math.trunc(diff) * 1000000 + Math.trunc(level);
  }

  /** Volle Lösung per randomisiertem Backtracking (Bitmasken-Kandidaten). */
  function fill(rng) {
    const board = new Array(81).fill(0);
    const row = new Array(9).fill(ALL); // je Einheit: Bitmaske der noch freien Ziffern
    const col = new Array(9).fill(ALL);
    const box = new Array(9).fill(ALL);

    function solve(i) {
      if (i === 81) return true;
      const r = ROW_OF[i], c = COL_OF[i], b = BOX_OF[i];
      const cand = row[r] & col[c] & box[b];
      if (!cand) return false;
      const digits = [];
      for (let d = 1; d <= 9; d++) if (cand & (1 << (d - 1))) digits.push(d);
      rng.shuffle(digits);
      for (const d of digits) {
        const m = 1 << (d - 1);
        board[i] = d;
        row[r] ^= m;
        col[c] ^= m;
        box[b] ^= m;
        if (solve(i + 1)) return true;
        board[i] = 0;
        row[r] |= m;
        col[c] |= m;
        box[b] |= m;
      }
      return false;
    }

    solve(0);
    return board;
  }

  /** Zählt Lösungen (MRV-Heuristik, Abbruch bei limit). Hot Path! */
  function countSolutions(board, limit = 2) {
    board = board.slice();
    const row = new Array(9).fill(ALL);
    const col = new Array(9).fill(ALL);
    const box = new Array(9).fill(ALL);
    const empty = [];
    for (let i = 0; i < 81; i++) {
      const d = board[i];
      if (d) {
        const m = 1 << (d - 1);
        const r = ROW_OF[i], c = COL_OF[i], b = BOX_OF[i];
        if (!(row[r] & m && col[c] & m && box[b] & m)) return 0; // Vorgaben widersprechen sich
        row[r] ^= m;
        col[c] ^= m;
        box[b] ^= m;
      } else {
        empty.push(i);
      }
    }

    let count = 0;

    function solve() {
      // MRV: leere Zelle mit den wenigsten Kandidaten zuerst.
      let bestI = -1, bestCand = 0, bestN = 10;
      for (const i of empty) {
        if (board[i]) continue;
        const cand = row[ROW_OF[i]] & col[COL_OF[i]] & box[BOX_OF[i]];
        const n = POPCOUNT[cand];
        if (n === 0) return;
        if (n < bestN) {
          bestI = i;
          bestCand = cand;
          bestN = n;
          if (n === 1) break;
        }
      }
      if (bestI < 0) {
        // keine leere Zelle mehr -> Lösung gefunden
        count++;
        return;
      }
      const r = ROW_OF[bestI], c = COL_OF[bestI], b = BOX_OF[bestI];
      let cand = bestCand;
      while (cand) {
        const m = cand & -cand; // niedrigstes gesetztes Bit
        cand ^= m;
        board[bestI] = 32 - Math.clz32(m);
        row[r] ^= m;
        col[c] ^= m;
        box[b] ^= m;
        solve();
        board[bestI] = 0;
        row[r] |= m;
        col[c] |= m;
        box[b] |= m;
        if (count >= limit) return;
      }
    }

    solve();
    return count;
  }

  /**
   * Leert Zellen der Lösung, solange das Puzzle eindeutig lösbar bleibt.
   * Ein einziger, rng-geshuffelter Durchlauf über alle 81 Positionen deckelt
   * die Arbeit und ist je Seed deterministisch. Gestoppt wird, sobald nur
   * noch target Vorgaben da sind.
   */
  function dig(solution, target, rng) {
    const puzzle = solution.slice();
    const order = Array.from({ length: 81 }, (_, i) => i);
    rng.shuffle(order);
    let clues = 81;
    for (const i of order) {
      if (clues <= target) break;
      const saved = puzzle[i];
      puzzle[i] = 0;
      if (countSolutions(puzzle) === 1) clues--;
      else puzzle[i] = saved;
    }
    return puzzle;
  }

  /**
   * Erzeugt [puzzle, solution] für Stufe diff (0..3), Level level.
   * Deterministisch: gleicher Aufruf -> identisches Puzzle.
   */
  function generate(diff, level) {
    diff = Math.max(0, Math.min(CLUES.length - 1, Math.trunc(diff)));
    const rng = new PG.Random(seedFor(diff, level));
    const solution = fill(rng);
    const puzzle = dig(solution, CLUES[diff], rng);
    return [puzzle, solution];
  }

  // =========================================================================
  //  Varianten: allgemeine Einheiten-Listen + seedrand (Desktop == Browser)
  // =========================================================================
  // Alles ab hier ist bitgenau gleich zu games/sudoku_gen.py: gleicher
  // Zufallsgenerator (PG.seedrand), gleiche Reihenfolge der Zufallszahlen,
  // gleiche Grab-Entscheidungen -> X-Sudoku, Mini 6x6 und das Tages-Sudoku
  // sind am PC und im Browser dieselben Rätsel. (Die klassischen Level oben
  // bleiben beim alten Generator.)

  const VARIANTS = ["classic", "x", "killer", "mini"];

  // Vorgaben-Ziel je Stufe (Leicht/Normal/Schwer/Experte).
  const VARIANT_CLUES = { classic: [42, 34, 29, 25], x: [36, 29, 25, 21], mini: [20, 16, 13, 10] };

  // Grab-Regel je Stufe (siehe sudoku_gen.py DIG_RULES).
  const DIG_RULES = ["singles", "singles", "unique", "hard"];

  // Tages-Sudoku: Stufe je Wochentag (Montag = 0 ... Sonntag = 6).
  const DAILY_DIFF = [0, 1, 1, 2, 2, 3, 2];

  const bitLength = (m) => 32 - Math.clz32(m);

  /** Geometrie eines Sudoku-Typs: Größe, Blockform und alle Einheiten. */
  class Layout {
    constructor(n, boxH, boxW, diagonals) {
      this.n = n;
      this.size = n * n;
      this.boxH = boxH;
      this.boxW = boxW;
      this.diagonals = !!diagonals;
      this.all = (1 << n) - 1;
      const units = [];
      for (let r = 0; r < n; r++) units.push(Array.from({ length: n }, (_, c) => r * n + c));
      for (let c = 0; c < n; c++) units.push(Array.from({ length: n }, (_, r) => r * n + c));
      for (let br = 0; br < n / boxH; br++) {
        for (let bc = 0; bc < n / boxW; bc++) {
          const u = [];
          for (let r = 0; r < boxH; r++) for (let c = 0; c < boxW; c++) u.push((br * boxH + r) * n + bc * boxW + c);
          units.push(u);
        }
      }
      if (this.diagonals) {
        units.push(Array.from({ length: n }, (_, i) => i * n + i));
        units.push(Array.from({ length: n }, (_, i) => i * n + (n - 1 - i)));
      }
      this.units = units;
      this.boxOf = [];
      this.unitsOf = [];
      this.peers = [];
      for (let i = 0; i < this.size; i++) {
        this.boxOf.push(Math.floor(Math.floor(i / n) / boxH) * (n / boxW) + Math.floor((i % n) / boxW));
        const us = [];
        const ps = new Set();
        units.forEach((u, k) => {
          if (u.includes(i)) {
            us.push(k);
            for (const j of u) if (j !== i) ps.add(j);
          }
        });
        this.unitsOf.push(us);
        this.peers.push(Array.from(ps).sort((a, b) => a - b));
      }
    }
    /** Liegt Zelle i auf einer der beiden Diagonalen (nur X-Sudoku)? */
    onDiagonal(i) {
      const r = Math.floor(i / this.n), c = i % this.n;
      return this.diagonals && (r === c || r + c === this.n - 1);
    }
  }

  const LAYOUTS = {};
  function layoutFor(variant) {
    if (!LAYOUTS[variant]) {
      if (variant === "mini") LAYOUTS[variant] = new Layout(6, 2, 3, false);
      else if (variant === "x") LAYOUTS[variant] = new Layout(9, 3, 3, true);
      else LAYOUTS[variant] = new Layout(9, 3, 3, false);
    }
    return LAYOUTS[variant];
  }

  /** Volle Lösung per Backtracking (Zelle mit den wenigsten Kandidaten zuerst). */
  function fillGrid(lay, rng) {
    const n = lay.n, size = lay.size, ALL = lay.all;
    const board = new Array(size).fill(0);
    const umask = new Array(lay.units.length).fill(ALL);
    const uof = lay.unitsOf;

    function solve(left) {
      if (!left) return true;
      let bestI = -1, bestC = 0, bestN = 99;
      for (let i = 0; i < size; i++) {
        if (board[i]) continue;
        let c = ALL;
        for (const u of uof[i]) c &= umask[u];
        const k = POPCOUNT[c];
        if (k < bestN) {
          bestI = i;
          bestC = c;
          bestN = k;
          if (k <= 1) break;
        }
      }
      if (!bestC) return false;
      const digits = [];
      for (let d = 1; d <= n; d++) if ((bestC >> (d - 1)) & 1) digits.push(d);
      rng.shuffle(digits);
      const us = uof[bestI];
      for (const d of digits) {
        const m = 1 << (d - 1);
        board[bestI] = d;
        for (const u of us) umask[u] ^= m;
        if (solve(left - 1)) return true;
        for (const u of us) umask[u] ^= m;
        board[bestI] = 0;
      }
      return false;
    }

    solve(size);
    return board;
  }

  /** Zählt Lösungen (MRV, Abbruch bei limit) - Ergebnis = min(Anzahl, limit). */
  function countVariant(lay, puzzle, limit = 2) {
    const size = lay.size, ALL = lay.all;
    const board = puzzle.slice();
    const uof = lay.unitsOf;
    const umask = new Array(lay.units.length).fill(ALL);
    for (let i = 0; i < size; i++) {
      const d = board[i];
      if (!d) continue;
      const m = 1 << (d - 1);
      for (const u of uof[i]) {
        if (!(umask[u] & m)) return 0;
        umask[u] ^= m;
      }
    }
    const empty = [];
    for (let i = 0; i < size; i++) if (!board[i]) empty.push(i);
    let count = 0;

    function solve() {
      let bestI = -1, bestC = 0, bestN = 99;
      for (const i of empty) {
        if (board[i]) continue;
        let c = ALL;
        for (const u of uof[i]) c &= umask[u];
        if (!c) return;
        const k = POPCOUNT[c];
        if (k < bestN) {
          bestI = i;
          bestC = c;
          bestN = k;
          if (k === 1) break;
        }
      }
      if (bestI < 0) {
        count++;
        return;
      }
      const us = uof[bestI];
      let c = bestC;
      while (c) {
        const m = c & -c;
        c ^= m;
        board[bestI] = bitLength(m);
        for (const u of us) umask[u] ^= m;
        solve();
        board[bestI] = 0;
        for (const u of us) umask[u] ^= m;
        if (count >= limit) return;
      }
    }

    solve();
    return count;
  }

  /** Löst nur mit Singles (Naked + Hidden Single). Lösung oder null. */
  function solveSingles(lay, puzzle) {
    const size = lay.size, ALL = lay.all;
    const board = puzzle.slice();
    const peers = lay.peers;
    const cand = new Array(size).fill(0);
    for (let i = 0; i < size; i++) {
      const d = board[i];
      if (d) {
        for (const j of peers[i]) if (board[j] === d) return null;
        continue;
      }
      let used = 0;
      for (const j of peers[i]) if (board[j]) used |= 1 << (board[j] - 1);
      cand[i] = ALL & ~used;
    }
    let left = 0;
    for (const v of board) if (!v) left++;
    while (left) {
      let placed = 0;
      for (let i = 0; i < size; i++) {
        if (board[i]) continue;
        const c = cand[i];
        if (!c) return null;
        if (!(c & (c - 1))) {
          board[i] = bitLength(c);
          cand[i] = 0;
          for (const j of peers[i]) cand[j] &= ~c;
          placed++;
        }
      }
      for (const cells of lay.units) {
        let once = 0, twice = 0, have = 0;
        for (const i of cells) {
          if (board[i]) have |= 1 << (board[i] - 1);
          else {
            const c = cand[i];
            twice |= once & c;
            once |= c;
          }
        }
        if (ALL & ~have & ~once) return null;
        let single = once & ~twice & ~have;
        while (single) {
          const m = single & -single;
          single ^= m;
          for (const i of cells) {
            if (!board[i] && cand[i] & m) {
              board[i] = bitLength(m);
              cand[i] = 0;
              for (const j of peers[i]) cand[j] &= ~m;
              placed++;
              break;
            }
          }
        }
      }
      if (!placed) return null;
      left -= placed;
    }
    return board;
  }

  /** Leert Zellen in rng-Reihenfolge, solange das Rätsel (nach Regel) lösbar bleibt. */
  function digVariant(lay, solution, target, rule, rng) {
    const puzzle = solution.slice();
    const order = Array.from({ length: lay.size }, (_, i) => i);
    rng.shuffle(order);
    let clues = lay.size;
    let easy = true;
    for (const i of order) {
      if (clues <= target) {
        if (rule !== "hard") break;
        if (easy) easy = solveSingles(lay, puzzle) !== null;
        if (!easy) break;
      }
      const saved = puzzle[i];
      puzzle[i] = 0;
      const ok = rule === "singles" ? solveSingles(lay, puzzle) !== null : countVariant(lay, puzzle) === 1;
      if (ok) clues--;
      else puzzle[i] = saved;
    }
    return puzzle;
  }

  function variantSeed(variant, diff, level) {
    return PG.seedrand.seedFrom("sudoku", variant, Math.trunc(diff), Math.trunc(level));
  }

  /** [puzzle, solution] für X-Sudoku ("x") oder Mini 6x6 ("mini"). */
  function generateVariant(variant, diff, level) {
    const lay = layoutFor(variant);
    const clues = VARIANT_CLUES[variant] || VARIANT_CLUES.classic;
    diff = Math.max(0, Math.min(3, Math.trunc(diff)));
    const rng = new PG.seedrand.Rand(variantSeed(variant, diff, level));
    const solution = fillGrid(lay, rng);
    const puzzle = digVariant(lay, solution, clues[diff], DIG_RULES[diff], rng);
    return [puzzle, solution];
  }

  /** Stufe des Tages-Sudokus (Wochentag, Montag = 0). */
  function dailyDiff(date) {
    const [y, m, d] = (date || PG.seedrand.todayStr()).split("-").map(Number);
    const wd = (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
    return DAILY_DIFF[wd];
  }

  function weekday(date) {
    const [y, m, d] = date.split("-").map(Number);
    return (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
  }

  /** [puzzle, solution, diff] des Tages-Sudokus (klassisch 9x9). */
  function generateDaily(date) {
    date = date || PG.seedrand.todayStr();
    const diff = dailyDiff(date);
    const lay = layoutFor("classic");
    const rng = new PG.seedrand.Rand(PG.seedrand.dailySeed("sudoku", date));
    const solution = fillGrid(lay, rng);
    const puzzle = digVariant(lay, solution, VARIANT_CLUES.classic[diff], DIG_RULES[diff], rng);
    return [puzzle, solution, diff];
  }

  /** [puzzle, solution, cages] eines vorab erzeugten Killer-Levels (oder null). */
  function killerLevel(diff, level) {
    const data = PG.sudokuKiller;
    const list = data && data[String(Math.max(0, Math.min(3, Math.trunc(diff))))];
    if (!list || level < 1 || level > list.length) return null;
    const raw = list[level - 1];
    const sol = raw.s.split("").map(Number);
    const puz = raw.g.split("").map(Number);
    const cages = raw.c.map(([total, cells]) => [total, cells.slice()]);
    return [puz, sol, cages];
  }

  PG.sudokuGen = {
    LEVELS, CLUES, ROW_OF, COL_OF, BOX_OF, UNITS, PEERS, seedFor, fill, countSolutions, dig, generate,
    VARIANTS, VARIANT_CLUES, DIG_RULES, DAILY_DIFF, Layout, layoutFor, fillGrid, countVariant, solveSingles,
    digVariant, variantSeed, generateVariant, dailyDiff, weekday, generateDaily, killerLevel,
  };
})();
