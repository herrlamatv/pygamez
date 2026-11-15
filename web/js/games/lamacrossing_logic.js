/*
 * lamacrossing_logic.js - Regeln von Lama Crossing (1:1 aus games/lamacrossing_logic.py)
 * =======================================================================================
 * Das Lama überquert Schritt für Schritt eine Schlucht, einen Fluss oder einen
 * Eissee. Jeder Schritt hält mit Wahrscheinlichkeit 1 - p und hebt den
 * Multiplikator; ein Fehltritt kostet den Einsatz.
 *
 * Multiplikatoren stehen als Ganzzahlen in Hundertsteln fest in den Tabellen
 * (m_k = floor(0,97 / (1 - p)^k * 100)) - in Python, JS und Rust bitgleich.
 * Jede Strategie "bei Schritt k auszahlen" bringt so ~97 % des Einsatzes zurück.
 * tests/audit_lamacrossing.py vergleicht beide Fassungen
 * (über web/tools/lamacrossing_logic_dump.js).
 */
(function () {
  "use strict";

  const DIFFS = ["easy", "medium", "hard", "extreme"];
  const WORLDS = ["bridge", "river", "ice"];
  const RISK = { easy: [1, 12], medium: [1, 7], hard: [1, 4], extreme: [2, 5] };
  const TABLES = {
    easy: [105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300, 327, 357, 390, 425, 464, 506, 552],
    medium: [113, 132, 154, 179, 209, 244, 285, 332, 388, 453, 528, 616, 719, 839, 979, 1142],
    hard: [129, 172, 229, 306, 408, 545, 726, 968, 1291, 1722, 2296, 3062, 4082],
    extreme: [161, 269, 449, 748, 1247, 2079, 3465, 5775, 9625, 16042],
  };
  const BETS = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
  const BRAVE_MULT = 1000;
  const HISTORY = 12;

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);

  function steps(diff) {
    return TABLES[diff].length;
  }
  function risk(diff) {
    const [num, den] = RISK[diff];
    return num / den;
  }
  function mult(diff, k) {
    if (k <= 0) return 100;
    const table = TABLES[diff];
    return table[Math.min(k, table.length) - 1];
  }
  function payout(bet, diff, k) {
    return Math.floor((bet * mult(diff, k)) / 100);
  }
  /** Ein Schritt: true = hält, false = Fehltritt (genau ein random()-Aufruf). */
  function roll(rng, diff) {
    return !(rng.random() < risk(diff));
  }
  /** Nach dem Auszahlen bei Schritt k: wo wäre es schiefgegangen? (null = nirgends) */
  function nextBreak(rng, diff, k) {
    for (let i = k + 1; i <= steps(diff); i++) {
      if (!roll(rng, diff)) return i;
    }
    return null;
  }
  function riskPercent(diff) {
    const [num, den] = RISK[diff];
    return Math.floor((num * 200 + den) / (den * 2));
  }
  function fmtMult(m100, sep = ".") {
    return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
  }
  function cleanRound(raw) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
    const { bet, diff, step } = raw;
    if (!DIFFS.includes(diff) || !isInt(bet) || !isInt(step)) return null;
    if (bet <= 0 || step < 1 || step >= steps(diff)) return null;
    return { bet, diff, step };
  }
  function cleanHistory(raw) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((v) => isInt(v) && v >= 0).slice(-HISTORY);
  }

  PG.lamaCrossingLogic = {
    DIFFS, WORLDS, RISK, TABLES, BETS, BRAVE_MULT, HISTORY,
    steps, risk, mult, payout, roll, nextBreak, riskPercent, fmtMult, cleanRound, cleanHistory,
  };
})();
