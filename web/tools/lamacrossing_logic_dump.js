#!/usr/bin/env node
/*
 * lamacrossing_logic_dump.js - gibt die Lama-Crossing-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/lamacrossing_logic_dump.js
 *
 * Wird von tests/audit_lamacrossing.py aufgerufen, um Python
 * (games/lamacrossing_logic.py) und JavaScript (web/js/games/lamacrossing_logic.js)
 * zu vergleichen: Tabellen, Verkehr, Auszahlungen, Ziehungen, Treffertest und
 * Anzeige-Texte.
 */
"use strict";
const path = require("path");
global.PG = {};
require(path.join(__dirname, "..", "js", "games", "lamacrossing_logic.js"));
const L = global.PG.lamaCrossingLogic;

const payout = {};
for (const d of L.DIFFS) {
  payout[d] = [];
  for (let k = 0; k <= L.steps(d) + 1; k++) {
    const row = {};
    for (const bet of [10, 55, 10000]) row[bet] = L.payout(bet, d, k);
    payout[d].push(row);
  }
}
// Ziehungen genau an den Grenzen 1/12, 1/6, 3/10 und 1/2
const rollsIn = [0, 0.05, 0.0833, 0.08333333333333333, 0.0834, 0.1666, 0.16666666666666666, 0.1667,
  0.25, 0.2999, 0.3, 3 / 10, 0.3001, 0.4999, 0.5, 0.5001, 0.99];
const rolls = [];
for (const d of L.DIFFS) for (const v of rollsIn) rolls.push(L.roll({ random: () => v }, d));
// Treffertest: [obs_y, obs_len, target_y, reach] inkl. Grenzfälle (genau Kante = kein Treffer)
const hitsIn = [
  [100, 40, 120, 10], [100, 40, 131, 10], [100, 40, 130, 10], [100, 40, 129.999, 10],
  [140, 40, 120, 0.5], [0, 0, 0, 0], [-50.5, 101, 0, 0], [10, 51, 0, 0], [300.25, 61, 250, 21.6],
  [0.1 + 0.2, 0, 0.3, 0], [329.4, 101, 250.8, 21.6], [250.8, 40, 250.8, 0], [-200, 170, 330, 21.6],
  [180.5, 117, 250.8, 21.599999999999998],
];
const hits = hitsIn.map((a) => L.hitTest(a[0], a[1], a[2], a[3]));
const fmt = [];
for (const m of [100, 105, 1142, 16042]) for (const s of [".", ","]) fmt.push(L.fmtMult(m, s));

process.stdout.write(JSON.stringify({
  diffs: L.DIFFS, worlds: L.WORLDS, tables: L.TABLES, risk: L.RISK, bets: L.BETS,
  brave: L.BRAVE_MULT, extreme: L.EXTREME_MULT, history: L.HISTORY,
  traffic: L.TRAFFIC, speed_f: L.SPEED_F, raser_f: L.RASER_F,
  payout, rolls_in: rollsIn, rolls, hits_in: hitsIn, hits, fmt,
  pct: L.DIFFS.map((d) => L.riskPercent(d)),
}));
