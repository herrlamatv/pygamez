/*
 * casino_logic.js - Spielregeln des Casinos (Port von games/casino_logic.py)
 * ===========================================================================
 * Reine Logik ohne Zeichnen: europäisches Roulette (Wetten, Treffererkennung
 * auf dem Setztisch, Auszahlung) und der Lama-Slot (Walzenstreifen,
 * Gewinnlinien, Auswertung, exakte Auszahlungsquote). Zahlen und Tabellen sind
 * 1:1 identisch mit Python - tests/audit_casino.py vergleicht beide.
 */
(function () {
  "use strict";

  // ============================================================ ROULETTE
  const WHEEL_ORDER = [0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8,
    23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12, 35, 3, 26];
  const RED_NUMBERS = new Set([1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36]);
  const CHIP_VALUES = [1, 5, 25, 100, 500];
  const HISTORY_LEN = 12;
  const TABLE_W = 14.0;
  const DOZEN_H = 0.8;
  const OUTSIDE_H = 0.8;
  const TABLE_H = 3.0 + DOZEN_H + OUTSIDE_H;
  const EDGE = 0.26;
  const OUTSIDE_KEYS = ["low", "even", "red", "black", "odd", "high"];
  const PAYOUT = { plein: 35, cheval: 17, trans: 11, carre: 8, sixain: 5, column: 2, dozen: 2,
    red: 1, black: 1, even: 1, odd: 1, low: 1, high: 1 };

  function colorOf(n) {
    if (n === 0) return "green";
    return RED_NUMBERS.has(n) ? "red" : "black";
  }
  const numCol = (n) => Math.floor((n - 1) / 3);
  const numRow = (n) => 2 - ((n - 1) % 3);
  const cellNumber = (col, row) => 3 * col + 3 - row;

  function makeKey(kind, numbers) {
    if (numbers === undefined || numbers === null) return kind;
    if (kind === "column" || kind === "dozen") return kind + ":" + Math.trunc(numbers);
    return kind + ":" + [...numbers].sort((a, b) => a - b).join("-");
  }
  const betKind = (key) => key.split(":")[0];

  const numCache = new Map();
  /** Alle Zahlen einer Wette (Array, aufsteigend). */
  function betNumbers(key) {
    let out = numCache.get(key);
    if (out) return out;
    const i = key.indexOf(":");
    const kind = i < 0 ? key : key.slice(0, i);
    const arg = i < 0 ? "" : key.slice(i + 1);
    out = [];
    if (kind === "column") {
      const r = Number(arg);
      for (let c = 0; c < 12; c++) out.push(cellNumber(c, r));
    } else if (kind === "dozen") {
      const d = Number(arg);
      for (let n = 1 + 12 * d; n < 13 + 12 * d; n++) out.push(n);
    } else if (kind === "red" || kind === "black") {
      for (let n = 1; n <= 36; n++) if (RED_NUMBERS.has(n) === (kind === "red")) out.push(n);
    } else if (kind === "even" || kind === "odd") {
      for (let n = 1; n <= 36; n++) if ((n % 2 === 0) === (kind === "even")) out.push(n);
    } else if (kind === "low") {
      for (let n = 1; n <= 18; n++) out.push(n);
    } else if (kind === "high") {
      for (let n = 19; n <= 36; n++) out.push(n);
    } else {
      out = arg.split("-").map(Number);
    }
    out.sort((a, b) => a - b);
    numCache.set(key, out);
    return out;
  }

  const payoutMultiplier = (key) => PAYOUT[betKind(key)];

  /** Abrechnung: [auszahlung (Einsatz + Gewinn), gewinner-schlüssel]. */
  function settle(bets, number) {
    let total = 0;
    const winners = [];
    for (const key of Object.keys(bets)) {
      const amount = bets[key];
      if (amount > 0 && betNumbers(key).includes(number)) {
        total += amount * (payoutMultiplier(key) + 1);
        winners.push(key);
      }
    }
    return [total, winners];
  }

  const street = (col) => makeKey("trans", [3 * col + 1, 3 * col + 2, 3 * col + 3]);

  /** Welche Wette liegt an Tisch-Position (ux, uy)? Schlüssel oder null. */
  function hitTest(ux, uy) {
    if (!(ux >= 0 && ux < TABLE_W && uy >= 0 && uy < TABLE_H)) return null;
    if (uy < 3.0) {
      if (ux >= 13.0) return makeKey("column", Math.floor(uy));
      if (ux < 1.0 - EDGE) return makeKey("plein", [0]);
      let col = Math.min(11, Math.max(0, Math.floor(ux - 1.0)));
      const row = Math.min(2, Math.floor(uy));
      let fx = ux - 1.0 - col;
      const fy = uy - row;
      if (ux < 1.0) {
        col = 0;
        fx = 0.0;
      }
      const left = fx < EDGE, right = fx > 1.0 - EDGE;
      const top = fy < EDGE, bottom = fy > 1.0 - EDGE;
      const n = cellNumber(col, row);
      const vedge = left || right, hedge = top || bottom;
      if (vedge && hedge) {
        const c1 = left ? col - 1 : col, c2 = left ? col : col + 1;
        let rows = null;
        if (top && row > 0) rows = [row - 1, row];
        else if (bottom && row < 2) rows = [row, row + 1];
        if (rows) {
          if (c1 < 0) return makeKey("trans", [0, cellNumber(0, rows[0]), cellNumber(0, rows[1])]);
          if (c2 > 11) return makeKey("cheval", [cellNumber(col, rows[0]), cellNumber(col, rows[1])]);
          const nums = [];
          for (const c of [c1, c2]) for (const r of rows) nums.push(cellNumber(c, r));
          return makeKey("carre", nums);
        }
        if (c1 < 0) return makeKey("carre", [0, 1, 2, 3]);
        if (c2 > 11) return street(col);
        const six = [];
        for (let k = 3 * c1 + 1; k < 3 * c2 + 4; k++) six.push(k);
        return makeKey("sixain", six);
      }
      if (vedge) {
        if (left && col === 0) return makeKey("cheval", [0, n]);
        if (right && col === 11) return makeKey("plein", [n]);
        return makeKey("cheval", [n, cellNumber(left ? col - 1 : col + 1, row)]);
      }
      if (hedge) {
        if (top && row > 0) return makeKey("cheval", [n, cellNumber(col, row - 1)]);
        if (bottom && row < 2) return makeKey("cheval", [n, cellNumber(col, row + 1)]);
        return street(col);
      }
      return makeKey("plein", [n]);
    }
    if (!(ux >= 1.0 && ux < 13.0)) return null;
    if (uy < 3.0 + DOZEN_H) return makeKey("dozen", Math.min(2, Math.floor((ux - 1.0) / 4.0)));
    return OUTSIDE_KEYS[Math.min(5, Math.floor((ux - 1.0) / 2.0))];
  }

  /** Mittelpunkt des Chip-Stapels einer Wette in Tisch-Einheiten. */
  function anchor(key) {
    const kind = betKind(key);
    if (kind === "column") return [13.5, Number(key.split(":")[1]) + 0.5];
    if (kind === "dozen") return [1.0 + 4.0 * Number(key.split(":")[1]) + 2.0, 3.0 + DOZEN_H / 2];
    if (OUTSIDE_KEYS.includes(kind)) return [1.0 + 2.0 * OUTSIDE_KEYS.indexOf(kind) + 1.0, 3.0 + DOZEN_H + OUTSIDE_H / 2];
    const nums = betNumbers(key);
    if (kind === "plein") {
      const n = nums[0];
      if (n === 0) return [0.5, 1.5];
      return [numCol(n) + 1.5, numRow(n) + 0.5];
    }
    if (nums.includes(0)) {
      const rest = nums.slice(1);
      if (nums.length === 4) return [1.0, 3.0];
      const rows = rest.map(numRow);
      if (rest.length === 1) return [1.0, rows[0] + 0.5];
      return [1.0, Math.max(...rows)];
    }
    const cols = [...new Set(nums.map(numCol))].sort((a, b) => a - b);
    const rows = [...new Set(nums.map(numRow))].sort((a, b) => a - b);
    const x = cols.length === 1 ? cols[0] + 1.5 : cols[1] + 1.0;
    if (kind === "trans" || kind === "sixain") return [x, 3.0];
    const y = rows.length === 1 ? rows[0] + 0.5 : rows[1];
    return [x, y];
  }

  // ============================================================ LAMA-SLOT
  const REELS = 5;
  const ROWS = 3;
  const WILD = "lama";
  const SCATTER = "coin";
  const SYMBOLS = ["lama", "coin", "seven", "gem", "bell", "clover", "grape", "lemon", "cherry"];
  const LINE_BETS = [1, 2, 5, 10];
  const AUTO_COUNTS = [10, 25];
  const FREE_SPINS = 10;
  const FREE_MULT = 2;
  const PAYS = {
    lama: [20, 100, 750],
    seven: [8, 30, 120],
    gem: [6, 20, 80],
    bell: [4, 12, 50],
    clover: [3, 10, 30],
    grape: [2, 5, 20],
    lemon: [1, 4, 15],
    cherry: [1, 4, 12],
  };
  const RTP_TEXT = "96.1";
  const SCATTER_PAYS = { 3: 2, 4: 10, 5: 50 };
  const LINES = [
    [1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0],
    [2, 2, 2, 2, 2],
    [0, 1, 2, 1, 0],
    [2, 1, 0, 1, 2],
    [0, 0, 1, 2, 2],
    [2, 2, 1, 0, 0],
    [1, 0, 0, 0, 1],
    [1, 2, 2, 2, 1],
    [1, 0, 1, 2, 1],
  ];
  const STRIPS = [
    ["cherry", "lama", "grape", "bell", "lemon", "gem", "cherry", "clover",
      "seven", "grape", "lemon", "lama", "cherry", "bell", "grape", "coin",
      "lemon", "clover", "gem", "cherry", "lama", "grape", "bell", "lemon",
      "seven", "clover", "cherry", "gem", "grape", "lama", "lemon", "bell"],
    ["lemon", "lama", "cherry", "gem", "grape", "bell", "lama", "clover",
      "cherry", "seven", "lemon", "grape", "lama", "bell", "cherry", "clover",
      "coin", "grape", "gem", "lemon", "lama", "cherry", "bell", "seven",
      "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell"],
    ["grape", "cherry", "lama", "bell", "lemon", "gem", "lama", "grape",
      "clover", "seven", "cherry", "lemon", "lama", "bell", "grape", "gem",
      "coin", "cherry", "clover", "lemon", "lama", "grape", "bell", "seven",
      "cherry", "clover", "lemon", "gem", "lama", "grape", "cherry", "bell"],
    ["cherry", "grape", "lama", "gem", "lemon", "bell", "lama", "cherry",
      "clover", "seven", "grape", "lemon", "lama", "bell", "cherry", "gem",
      "coin", "grape", "clover", "lemon", "lama", "cherry", "bell", "seven",
      "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell"],
    ["lemon", "cherry", "lama", "grape", "bell", "gem", "lemon", "clover",
      "seven", "cherry", "grape", "lama", "lemon", "bell", "cherry", "coin",
      "grape", "clover", "gem", "lemon", "lama", "cherry", "bell", "grape",
      "seven", "clover", "lemon", "gem", "cherry", "lama", "grape", "bell"],
  ];

  function window_(stops) {
    return stops.map((stop, r) => {
      const strip = STRIPS[r];
      const out = [];
      for (let row = 0; row < ROWS; row++) out.push(strip[(stop + row) % strip.length]);
      return out;
    });
  }

  function randomStops(rng) {
    return STRIPS.map((s) => Math.floor(rng.random() * s.length));
  }

  /** [symbol, anzahl, multiplikator] einer Linie (wie Python line_result). */
  function lineResult(symbols) {
    let wildRun = 0;
    while (wildRun < REELS && symbols[wildRun] === WILD) wildRun++;
    let best = [WILD, wildRun, wildRun >= 3 ? PAYS[WILD][wildRun - 3] : 0];
    const base = symbols.find((s) => s !== WILD);
    if (base === undefined || base === SCATTER) return best[2] > 0 ? best : [null, 0, 0];
    let n = 0;
    for (const s of symbols) {
      if (s === base || s === WILD) n++;
      else break;
    }
    const mult = n >= 3 ? PAYS[base][n - 3] : 0;
    if (mult > best[2]) best = [base, n, mult];
    return best[2] > 0 ? best : [null, 0, 0];
  }

  function evaluate(win, lineBet, free = false) {
    const mult = free ? FREE_MULT : 1;
    const lines = [];
    let total = 0;
    let jackpot = false;
    LINES.forEach((line, i) => {
      const [sym, n, m] = lineResult(line.map((row, r) => win[r][row]));
      if (m > 0) {
        const amount = m * lineBet * mult;
        lines.push([i, sym, n, amount]);
        total += amount;
        if (sym === WILD && n === REELS) jackpot = true;
      }
    });
    const coins = [];
    for (let r = 0; r < REELS; r++) for (let row = 0; row < ROWS; row++) if (win[r][row] === SCATTER) coins.push([r, row]);
    const scatterWin = (SCATTER_PAYS[coins.length] || 0) * lineBet * LINES.length * mult;
    total += scatterWin;
    return { lines, scatter: coins.length, coins, scatter_win: scatterWin, total, jackpot,
      free_spins: coins.length >= 3 ? FREE_SPINS : 0 };
  }

  /** Auszahlungsquote exakt aus den Walzenstreifen (siehe Python exact_rtp). */
  function exactRtp() {
    const probs = STRIPS.map((strip) => {
      const counts = {};
      for (const s of strip) counts[s] = (counts[s] || 0) + 1;
      return Object.keys(counts).sort().map((s) => [s, counts[s] / strip.length]);
    });
    let line = 0;
    const combo = new Array(REELS);
    const rec = (r, p) => {
      if (r === REELS) {
        const m = lineResult(combo)[2];
        if (m) line += p * m;
        return;
      }
      for (const [s, q] of probs[r]) {
        combo[r] = s;
        rec(r + 1, p * q);
      }
    };
    rec(0, 1);
    let dist = [1];
    for (const strip of STRIPS) {
      const n = strip.length;
      const per = new Array(ROWS + 1).fill(0);
      for (let stop = 0; stop < n; stop++) {
        let k = 0;
        for (let row = 0; row < ROWS; row++) if (strip[(stop + row) % n] === SCATTER) k++;
        per[k] += 1 / n;
      }
      const next = new Array(dist.length + ROWS).fill(0);
      dist.forEach((pa, a) => per.forEach((pb, b) => (next[a + b] += pa * pb)));
      dist = next;
    }
    let scatter = 0, trigger = 0;
    dist.forEach((p, k) => {
      scatter += p * (SCATTER_PAYS[k] || 0);
      if (k >= 3) trigger += p;
    });
    const base = line + scatter;
    const expFree = FREE_SPINS / (1 - FREE_SPINS * trigger);
    return { rtp: base + trigger * expFree * FREE_MULT * base, line, scatter, trigger, free_spins: expFree };
  }

  PG.casinoLogic = {
    WHEEL_ORDER, RED_NUMBERS, CHIP_VALUES, HISTORY_LEN, TABLE_W, TABLE_H, DOZEN_H, OUTSIDE_H, EDGE,
    OUTSIDE_KEYS, PAYOUT, colorOf, numCol, numRow, cellNumber, makeKey, betKind, betNumbers,
    payoutMultiplier, settle, hitTest, anchor,
    REELS, ROWS, WILD, SCATTER, SYMBOLS, LINE_BETS, AUTO_COUNTS, FREE_SPINS, FREE_MULT, PAYS, RTP_TEXT,
    SCATTER_PAYS, LINES, STRIPS, window: window_, randomStops, lineResult, evaluate, exactRtp,
  };
})();
