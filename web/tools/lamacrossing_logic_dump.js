#!/usr/bin/env node
/*
 * lamacrossing_logic_dump.js - gibt die Lama-Crossing-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/lamacrossing_logic_dump.js
 *
 * Wird von tests/audit_lamacrossing.py aufgerufen, um Python
 * (games/lamacrossing_logic.py) und JavaScript (web/js/games/lamacrossing_logic.js)
 * zu vergleichen: Tabellen, Auszahlungen, Ziehungen und Anzeige-Texte.
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
const rollsIn = [0, 0.05, 0.0833, 0.08333333333333333, 0.0834, 0.142857, 0.1428571428571428, 0.15,
  0.2499, 0.25, 0.3999, 0.4, 0.99];
const rolls = [];
for (const d of L.DIFFS) for (const v of rollsIn) rolls.push(L.roll({ random: () => v }, d));
const fmt = [];
for (const m of [100, 105, 1142, 16042]) for (const s of [".", ","]) fmt.push(L.fmtMult(m, s));

process.stdout.write(JSON.stringify({
  diffs: L.DIFFS, worlds: L.WORLDS, tables: L.TABLES, risk: L.RISK, bets: L.BETS,
  brave: L.BRAVE_MULT, history: L.HISTORY, payout, rolls_in: rollsIn, rolls, fmt,
  pct: L.DIFFS.map((d) => L.riskPercent(d)),
}));
