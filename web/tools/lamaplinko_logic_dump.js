#!/usr/bin/env node
/*
 * lamaplinko_logic_dump.js - gibt die Lama-Plinko-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/lamaplinko_logic_dump.js            # Tabellen + feste Werte
 *   node web/tools/lamaplinko_logic_dump.js - < f.json # dazu Faelle aus der Eingabe
 *
 * Wird von tests/audit_chiplogic.py aufgerufen, um Python
 * (games/lamaplinko_logic.py) und JavaScript (web/js/games/lamaplinko_logic.js)
 * zu vergleichen: Tabellen, Quoten, Auszahlungen, Texte, Pfade aus Seeds und
 * die clean_*-Pruefungen. Faelle (optional): paths [[rows, s, c, n]], drops
 * [[seed, rows]], hits [Rohdaten], history [Rohdaten], carry [[[bet, rows, risk,
 * slot], ...]], carry_clean [Rohdaten].
 */
"use strict";
const fs = require("fs");
const path = require("path");
global.window = global;
global.PG = {};
for (const f of ["seedrand.js", "fairness.js", "lamaplinko_logic.js"]) require(path.join(__dirname, "..", "js", "games", f));
const L = global.PG.lamaPlinkoLogic;

let cases = {};
const arg = process.argv[2];
if (arg === "-") cases = JSON.parse(fs.readFileSync(0, "utf8") || "{}");
else if (arg) cases = JSON.parse(fs.readFileSync(arg, "utf8"));

const expected = {}, payout = {}, short = {}, probs = {}, fmtprob = {};
for (const risk of L.RISKS) {
  expected[risk] = [];
  payout[risk] = {};
  short[risk] = {};
  for (const r of L.ROWS) {
    expected[risk].push(L.expectedReturn(r, risk));
    payout[risk][r] = [];
    short[risk][r] = [];
    for (let s = 0; s <= r; s++) {
      payout[risk][r].push([1, 7, 1000].map((b) => L.payout(b, r, risk, s)));
      short[risk][r].push(L.fmtShort(L.mult100(r, risk, s), ","));
    }
  }
}
for (const r of L.ROWS) {
  probs[r] = [];
  fmtprob[r] = [];
  for (let s = 0; s <= r; s++) {
    probs[r].push(L.prob(r, s));
    fmtprob[r].push([L.fmtProb(r, s), L.fmtProb(r, s, ",")]);
  }
}
const fmt = [];
for (const v of [20, 100, 125, 1100, 100000]) for (const s of [".", ","]) fmt.push([L.fmtMult(v, s), L.fmtShort(v, s)]);

const out = {
  key: L.KEY, rows: L.ROWS, default_rows: L.DEFAULT_ROWS, risks: L.RISKS, default_risk: L.DEFAULT_RISK,
  bets: L.BETS, max_balls: L.MAX_BALLS, history: L.HISTORY, edge_rows: L.EDGE_ROWS, rain_balls: L.RAIN_BALLS,
  tables: L.TABLES, expected, payout, short, probs, fmtprob, fmt,
  edge: [[8, 0], [8, 8], [12, 6], [16, 16]].map(([r, s]) => L.isEdge(r, s)),
  slot_of: [[0, 1, 1, 0], [], [1, 1, 1]].map((p) => L.slotOf(p)),
  blank_hits: L.blankHits(),
  add_hit: L.addHit(L.addHit(L.blankHits(), 16, 0), 16, 0)["16"],
};
if (cases.carry) {
  // Kugelfolgen [[bet, rows, risk, slot], ...] mit Uebertrag ab 0
  out.carry = cases.carry.map((seq) => {
    let carry = 0;
    return seq.map(([b, r, k, s]) => {
      const [win, c] = L.payoutCarry(b, r, k, s, carry);
      carry = c;
      return [win, c];
    });
  });
}
if (cases.carry_clean) out.carry_clean = cases.carry_clean.map((v) => L.cleanCarry(v));
if (cases.paths) out.paths = cases.paths.map(([r, s, c, n]) => L.pathFromSeed(r, s, c, n));
if (cases.drops) out.drops = cases.drops.map(([seed, r]) => L.dropPath(new PG.seedrand.Rand(seed), r));
if (cases.hits) out.hits = cases.hits.map((h) => L.cleanHits(h));
if (cases.history) out.clean_history = cases.history.map((h) => L.cleanHistory(h));
process.stdout.write(JSON.stringify(out));
