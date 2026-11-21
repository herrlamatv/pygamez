/*
 * lamaplinko_logic.js - Regeln von Lama Plinko (1:1 aus games/lamaplinko_logic.py)
 * ================================================================================
 * Ein Wollknaeuel faellt durch rows Reihen von Stiften (je 50 % links/rechts);
 * das Fach ist die Zahl der Rechts-Spruenge, Wahrscheinlichkeit
 * C(rows, i) / 2^rows. Die 27 Tabellen (Multiplikatoren x100) stehen fest im
 * Code (erzeugt mit devtools/gen_plinko_tables.py): symmetrisch, nach aussen
 * nicht fallend, Quote 96,9-97,1 %. Der Pfad kommt aus dem Fairness-Zufall
 * (PG.fairness.roundRand(KEY, ...)): rows Wuerfe random() < 0.5 = rechts.
 * tests/audit_chiplogic.py vergleicht beide Fassungen
 * (ueber web/tools/lamaplinko_logic_dump.js). Braucht seedrand.js + fairness.js.
 */
(function () {
  "use strict";

  const KEY = "lamaplinko";
  const ROWS = [8, 9, 10, 11, 12, 13, 14, 15, 16];
  const DEFAULT_ROWS = 12;
  const RISKS = ["low", "medium", "high"];
  const DEFAULT_RISK = "medium";
  const BETS = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000];
  const MAX_BALLS = 50;
  const HISTORY = 12;
  const EDGE_ROWS = 12;
  const RAIN_BALLS = 25;
  const MAX_HITS = 9007199254740991; // 2^53 - 1

  // Multiplikator x100 je Fach 0 .. rows (erzeugt mit devtools/gen_plinko_tables.py)
  const TABLES = {
    low: {
      8: [530, 200, 120, 80, 70, 80, 120, 200, 530],
      9: [590, 260, 130, 100, 70, 70, 100, 130, 260, 590],
      10: [640, 300, 160, 110, 80, 70, 80, 110, 160, 300, 640],
      11: [770, 370, 200, 130, 90, 70, 70, 90, 130, 200, 370, 770],
      12: [880, 440, 250, 140, 100, 80, 70, 80, 100, 140, 250, 440, 880],
      13: [950, 480, 290, 160, 120, 90, 70, 70, 90, 120, 160, 290, 480, 950],
      14: [1200, 640, 340, 200, 120, 100, 80, 70, 80, 100, 120, 200, 340, 640, 1200],
      15: [1400, 700, 410, 230, 160, 120, 80, 70, 70, 80, 120, 160, 230, 410, 700, 1400],
      16: [1600, 860, 490, 280, 180, 130, 100, 70, 70, 70, 100, 130, 180, 280, 490, 860, 1600],
    },
    medium: {
      8: [1300, 340, 130, 60, 40, 60, 130, 340, 1300],
      9: [1800, 470, 160, 80, 50, 50, 80, 160, 470, 1800],
      10: [2100, 670, 250, 100, 60, 40, 60, 100, 250, 670, 2100],
      11: [3000, 880, 340, 140, 80, 40, 40, 80, 140, 340, 880, 3000],
      12: [3700, 1200, 460, 180, 90, 60, 40, 60, 90, 180, 460, 1200, 3700],
      13: [5000, 1600, 620, 250, 130, 70, 40, 40, 70, 130, 250, 620, 1600, 5000],
      14: [6300, 2200, 820, 340, 160, 90, 50, 40, 50, 90, 160, 340, 820, 2200, 6300],
      15: [8400, 3000, 1100, 460, 200, 110, 70, 40, 40, 70, 110, 200, 460, 1100, 3000, 8400],
      16: [11000, 3800, 1400, 620, 280, 140, 80, 50, 40, 50, 80, 140, 280, 620, 1400, 3800, 11000],
    },
    high: {
      8: [3200, 470, 110, 30, 20, 30, 110, 470, 3200],
      9: [4700, 770, 180, 50, 20, 20, 50, 180, 770, 4700],
      10: [7100, 1200, 270, 80, 30, 20, 30, 80, 270, 1200, 7100],
      11: [12000, 2000, 420, 120, 40, 20, 20, 40, 120, 420, 2000, 12000],
      12: [18000, 2900, 660, 180, 60, 30, 20, 30, 60, 180, 660, 2900, 18000],
      13: [27000, 5000, 1000, 270, 90, 40, 20, 20, 40, 90, 270, 1000, 5000, 27000],
      14: [40000, 7500, 1600, 410, 130, 50, 30, 20, 30, 50, 130, 410, 1600, 7500, 40000],
      15: [66000, 12000, 2500, 620, 190, 70, 40, 20, 20, 40, 70, 190, 620, 2500, 12000, 66000],
      16: [100000, 19000, 3800, 970, 280, 90, 50, 30, 20, 30, 50, 90, 280, 970, 3800, 19000, 100000],
    },
  };

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);

  /** Multiplikatoren x100 aller Faecher 0 .. rows. */
  function table(rows, risk) {
    return TABLES[risk][rows];
  }
  /** Multiplikator x100 eines Fachs. */
  function mult100(rows, risk, slot) {
    return TABLES[risk][rows][slot];
  }
  /** Auszahlung einer Kugel (abgerundet auf ganze Chips). */
  function payout(bet, rows, risk, slot) {
    return Math.floor((bet * mult100(rows, risk, slot)) / 100);
  }
  /** [Auszahlung, neuer Uebertrag]: bet * m100 + carry in Hundertsteln, ganze Chips
   *  werden ausgezahlt, der Rest (0..99) bleibt fuer die naechste Kugel. */
  function payoutCarry(bet, rows, risk, slot, carry) {
    const total = bet * mult100(rows, risk, slot) + carry;
    return [Math.floor(total / 100), total % 100];
  }
  /** Gespeicherter Uebertrag: ganze Zahl 0..99, sonst 0. */
  function cleanCarry(raw) {
    return isInt(raw) && raw >= 0 && raw <= 99 ? raw : 0;
  }
  /** Aeusserstes Fach? */
  function isEdge(rows, slot) {
    return slot === 0 || slot === rows;
  }
  /** Pfad einer Kugel: je Reihe 1 = rechts (random() < 0.5), sonst 0. */
  function dropPath(rand, rows) {
    const out = [];
    for (let i = 0; i < rows; i++) out.push(rand.random() < 0.5 ? 1 : 0);
    return out;
  }
  /** Fach = Zahl der Rechts-Spruenge. */
  function slotOf(path) {
    let s = 0;
    for (const v of path) s += v;
    return s;
  }
  /** Pfad einer Kugel aus ihren Fairness-Werten (zum Nachrechnen). */
  function pathFromSeed(rows, server, client, nonce) {
    return dropPath(PG.fairness.roundRand(KEY, server, client, nonce), rows);
  }
  /** Binomialkoeffizient C(n, k) (exakt, n <= 16). */
  function comb(n, k) {
    if (k < 0 || k > n) return 0;
    k = Math.min(k, n - k);
    let r = 1;
    for (let i = 1; i <= k; i++) r = (r * (n - k + i)) / i;
    return r;
  }
  /** Wahrscheinlichkeit eines Fachs als Bruch [Zaehler, Nenner]. */
  function prob(rows, slot) {
    return [comb(rows, slot), Math.pow(2, rows)];
  }
  /** Auszahlungsquote einer Tabelle (Gleitkomma = float(Fraction) in Python). */
  function expectedReturn(rows, risk) {
    let num = 0;
    TABLES[risk][rows].forEach((m, i) => { num += comb(rows, i) * m; });
    return num / (Math.pow(2, rows) * 100);
  }
  /** Wahrscheinlichkeit in Prozent als Text: ab 1 % zwei Stellen, darunter vier. */
  function fmtProb(rows, slot, sep = ".") {
    const num = comb(rows, slot);
    const den = Math.pow(2, rows);
    if (num * 100 >= den) {
      const v = Math.floor((num * 20000 + den) / (den * 2));
      return Math.floor(v / 100) + sep + String(v % 100).padStart(2, "0");
    }
    const v = Math.floor((num * 2000000 + den) / (den * 2));
    return Math.floor(v / 10000) + sep + String(v % 10000).padStart(4, "0");
  }
  /** Multiplikator x100 mit zwei Stellen: 20 -> "0.20". */
  function fmtMult(m100, sep = ".") {
    return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
  }
  /** Multiplikator x100 kurz: 20 -> "0.2", 1100 -> "11", 125 -> "1.25". */
  function fmtShort(m100, sep = ".") {
    if (m100 % 100 === 0) return String(Math.floor(m100 / 100));
    if (m100 % 10 === 0) return Math.floor(m100 / 100) + sep + Math.floor((m100 % 100) / 10);
    return fmtMult(m100, sep);
  }
  /** Leere Trefferzaehler: {"8": [0 x 9], ..., "16": [0 x 17]}. */
  function blankHits() {
    const out = {};
    for (const r of ROWS) out[String(r)] = new Array(r + 1).fill(0);
    return out;
  }
  /** Trefferzaehler pruefen (immer alle neun Schluessel; falsche Laenge -> Nullen). */
  function cleanHits(raw) {
    const out = blankHits();
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return out;
    for (const r of ROWS) {
      const v = raw[String(r)];
      if (Array.isArray(v) && v.length === r + 1) out[String(r)] = v.map((x) => (isInt(x) && x >= 0 && x <= MAX_HITS ? x : 0));
    }
    return out;
  }
  /** Einen Treffer zaehlen. */
  function addHit(hits, rows, slot) {
    hits[String(rows)][slot] += 1;
    return hits;
  }
  /** Verlauf der letzten Kugeln: Multiplikatoren x100, letzte HISTORY. */
  function cleanHistory(raw) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((v) => isInt(v) && v >= 0 && v <= 1000000).slice(-HISTORY);
  }

  PG.lamaPlinkoLogic = {
    KEY, ROWS, DEFAULT_ROWS, RISKS, DEFAULT_RISK, BETS, MAX_BALLS, HISTORY, EDGE_ROWS, RAIN_BALLS, TABLES,
    table, mult100, payout, payoutCarry, cleanCarry, isEdge, dropPath, slotOf, pathFromSeed, comb, prob, expectedReturn,
    fmtProb, fmtMult, fmtShort, blankHits, cleanHits, addHit, cleanHistory,
  };
})();
