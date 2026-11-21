/*
 * lamamines_logic.js - Regeln von Lama Mines (1:1 aus games/lamamines_logic.py)
 * ==============================================================================
 * N = grid x grid Plaetze, M Minen, der Rest Edelsteine. Multiplikator (x100)
 * nach k Edelsteinen - ganzzahlig, schrittweise, in Python, JS und Rust bitgleich:
 *
 *   raw(j) = HOUSE * C(N, M) // C(N - j, M)
 *   m_0 = 100,  m_j = max(raw(j), m_(j-1) + 1)  (j = 1 .. k)
 *   m100(k) = min(MAX_MULT100, m_k)
 *
 * Die Untergrenze m_(j-1) + 1 macht jeden Edelstein mindestens x0.01 wertvoller
 * (sonst braechte z.B. 6x6 mit 1 Mine beim ersten Edelstein nur x0.99).
 *
 * Groesster Wert C(49, 24) * 97 < 2^53 - exakt mit JS-Zahlen. Das Minenfeld
 * entsteht aus dem Fairness-Zufall (PG.fairness.roundRand(KEY, ...)): Liste
 * 0 .. N-1 mischen, die ersten M Plaetze sind Minen. Platz = Zeile * grid + Spalte.
 * tests/audit_chiplogic.py vergleicht beide Fassungen
 * (ueber web/tools/lamamines_logic_dump.js). Braucht seedrand.js + fairness.js.
 */
(function () {
  "use strict";

  const KEY = "lamamines";
  const GRIDS = [4, 5, 6, 7];
  const DEFAULT_GRID = 5;
  const DEFAULT_MINES = 3;
  const BETS = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
  const HOUSE = 97;
  const MAX_MULT100 = 10000000;
  const HISTORY = 12;
  const MINE_PRESETS = {
    4: [1, 2, 3, 5, 8, 12],
    5: [1, 3, 5, 10, 15, 24],
    6: [1, 3, 6, 12, 20, 35],
    7: [1, 3, 7, 14, 24, 48],
  };
  // Erfolge: "Mutig" ab x10, "Legende" ab x1000 ausgezahlt
  const BRAVE_MULT = 1000;
  const LEGEND_MULT = 100000;

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);

  /** Ganzzahlige Division a // b (a, b >= 0, beide < 2^53) - exakt. */
  function idiv(a, b) {
    let q = Math.floor(a / b);
    while (q * b > a) q -= 1;
    while ((q + 1) * b <= a) q += 1;
    return q;
  }

  /** Anzahl der Plaetze. */
  function cells(grid) {
    return grid * grid;
  }
  /** Hoechste Minenzahl (mindestens ein Edelstein). */
  function maxMines(grid) {
    return cells(grid) - 1;
  }
  /** Anzahl der Edelsteine. */
  function gems(grid, mines) {
    return cells(grid) - mines;
  }
  /** Minenzahl auf 1 .. N-1 begrenzen. */
  function clampMines(grid, mines) {
    return Math.max(1, Math.min(maxMines(grid), Math.trunc(mines)));
  }
  /** Binomialkoeffizient C(n, k) - schrittweise wie in Python (jeder Zwischenwert < 2^53). */
  function comb(n, k) {
    if (k < 0 || k > n) return 0;
    k = Math.min(k, n - k);
    let r = 1;
    for (let i = 1; i <= k; i++) r = idiv(r * (n - k + i), i);
    return r;
  }
  /** Reine Formel HOUSE * C(N, M) // C(N - j, M) (ohne Untergrenze und Deckel). */
  function rawMult100(grid, mines, j) {
    const n = cells(grid);
    return idiv(HOUSE * comb(n, mines), comb(n - j, mines));
  }
  /** Multiplikator x100 nach k Edelsteinen (k <= 0 -> 100): m_j = max(raw(j), m_(j-1) + 1), gedeckelt. */
  function mult100(grid, mines, k) {
    if (k <= 0) return 100;
    const n = cells(grid);
    k = Math.min(k, n - mines);
    const top = HOUSE * comb(n, mines);
    let m = 100;
    for (let j = 1; j <= k; j++) {
      m = Math.max(idiv(top, comb(n - j, mines)), m + 1);
      if (m >= MAX_MULT100) return MAX_MULT100;
    }
    return m;
  }
  /** Auszahlung nach k Edelsteinen (abgerundet auf ganze Chips). */
  function payout(bet, grid, mines, k) {
    return idiv(bet * mult100(grid, mines, k), 100);
  }
  /** [Auszahlung, neuer Uebertrag] mit Rest-Hundertstel-Uebertrag (wie Plinko). */
  function payoutCarry(bet, grid, mines, k, carry) {
    const total = bet * mult100(grid, mines, k) + carry;
    return [idiv(total, 100), total % 100];
  }
  /** Gespeicherter Uebertrag: ganze Zahl 0..99, sonst 0. */
  function cleanCarry(raw) {
    return isInt(raw) && raw >= 0 && raw <= 99 ? raw : 0;
  }
  /** Multiplikator x100 nach dem naechsten Edelstein oder null (alle gefunden). */
  function nextMult100(grid, mines, k) {
    if (k >= gems(grid, mines)) return null;
    return mult100(grid, mines, k + 1);
  }
  /** Chance auf ein sicheres naechstes Feld: [N-M-k, N-k]. */
  function safeChance(grid, mines, k) {
    const n = cells(grid);
    return [Math.max(0, n - mines - k), Math.max(1, n - k)];
  }
  /** Chance in ZEHNTEL-Prozent, kaufmaennisch gerundet (845 = 84,5 %). */
  function safePercent(grid, mines, k) {
    const [num, den] = safeChance(grid, mines, k);
    return idiv(num * 2000 + den, den * 2);
  }
  /** Chance, dass k bestimmte Felder ALLE sicher sind, in Zehntel-Prozent (gerundet).
   *  C(N-k, M) * 2000 passt nicht in 2^53 - deshalb BigInt. k <= 0 -> 1000, k > N-M -> 0. */
  function allSafePercent(grid, mines, k) {
    const n = cells(grid);
    if (k <= 0) return 1000;
    if (k > n - mines) return 0;
    const num = BigInt(comb(n - k, mines));
    const den = BigInt(comb(n, mines));
    return Number((num * BigInt(2000) + den) / (den * BigInt(2)));
  }
  /** Zehntel-Prozent als Text: 845 -> "84.5". */
  function fmtPercent(p10, sep = ".") {
    return Math.floor(p10 / 10) + sep + (p10 % 10);
  }
  /** Hoechstgewinn erreicht -> automatisch auszahlen. */
  function isMax(grid, mines, k) {
    return mult100(grid, mines, k) >= MAX_MULT100;
  }
  /** Alle Edelsteine gefunden -> automatisch auszahlen. */
  function isClear(grid, mines, k) {
    return k >= gems(grid, mines);
  }
  /** Die naechsten count Stufen: [[Schritt, Multiplikator x100], ...]. */
  function ladder(grid, mines, k, count = 5) {
    const out = [];
    for (let j = k + 1; j <= Math.min(gems(grid, mines), k + count); j++) out.push([j, mult100(grid, mines, j)]);
    return out;
  }
  /** Minenplaetze (sortiert): Liste 0..N-1 mischen, die ersten M sind Minen. */
  function makeField(rand, grid, mines) {
    const order = [];
    for (let i = 0; i < cells(grid); i++) order.push(i);
    rand.shuffle(order);
    return order.slice(0, mines).sort((a, b) => a - b);
  }
  /** Minenplaetze einer Runde aus ihren Fairness-Werten (zum Nachrechnen). */
  function fieldFromSeed(grid, mines, server, client, nonce) {
    return makeField(PG.fairness.roundRand(KEY, server, client, nonce), grid, mines);
  }
  /** Multiplikator x100 als Text: 235 -> "2.35". */
  function fmtMult(m100, sep = ".") {
    return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
  }
  /** Erwartungswert (Anteil am Einsatz, Gleitkomma) der Strategie "nach k auszahlen". */
  function expectedReturn(grid, mines, k) {
    const n = cells(grid);
    k = Math.max(0, Math.min(k, n - mines));
    return (comb(n - k, mines) / comb(n, mines)) * mult100(grid, mines, k) / 100;
  }

  /** Gespeicherte, noch laufende Runde pruefen (siehe clean_round in Python) oder null. */
  function cleanRound(raw) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
    const { bet, grid, mines, n, s, c, h } = raw;
    const opened = raw.open;
    const F = PG.fairness;
    if (!isInt(bet) || !isInt(grid) || !isInt(mines) || bet <= 0 || !GRIDS.includes(grid)) return null;
    if (mines < 1 || mines > maxMines(grid)) return null;
    if (!(F.validNonce(n) && F.validServer(s) && F.validClient(c) && F.validHash(h) && F.commit(s) === h)) return null;
    if (!Array.isArray(opened) || opened.length >= gems(grid, mines)) return null;
    if (!opened.every((v) => isInt(v) && v >= 0 && v < cells(grid)) || new Set(opened).size !== opened.length) return null;
    const field = new Set(fieldFromSeed(grid, mines, s, c, n));
    if (opened.some((v) => field.has(v))) return null;
    return { bet, grid, mines, n, s, c, h, open: opened.slice() };
  }
  /** Verlauf: Liste von Multiplikatoren x100 (0 = verloren), letzte HISTORY. */
  function cleanHistory(raw) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((v) => isInt(v) && v >= 0 && v <= MAX_MULT100).slice(-HISTORY);
  }
  /** Auto-Markierungen je Feldgroesse: immer {"4": [...], ..., "7": [...]}. */
  function cleanMarks(raw) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) raw = {};
    const out = {};
    for (const grid of GRIDS) {
      const v = raw[String(grid)];
      const marks = [];
      if (Array.isArray(v)) {
        for (const c of v) if (isInt(c) && c >= 0 && c < cells(grid) && !marks.includes(c)) marks.push(c);
      }
      out[String(grid)] = marks.sort((a, b) => a - b).slice(0, maxMines(grid));
    }
    return out;
  }

  PG.lamaMinesLogic = {
    KEY, GRIDS, DEFAULT_GRID, DEFAULT_MINES, BETS, HOUSE, MAX_MULT100, HISTORY, MINE_PRESETS, BRAVE_MULT, LEGEND_MULT,
    cells, maxMines, gems, clampMines, comb, rawMult100, mult100, payout, payoutCarry, cleanCarry, nextMult100, safeChance, safePercent, allSafePercent, fmtPercent,
    isMax, isClear, ladder, makeField, fieldFromSeed, fmtMult, expectedReturn, cleanRound, cleanHistory, cleanMarks,
  };
})();
