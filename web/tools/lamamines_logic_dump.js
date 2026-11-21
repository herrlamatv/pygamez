#!/usr/bin/env node
/*
 * lamamines_logic_dump.js - gibt die Lama-Mines-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/lamamines_logic_dump.js            # Tabellen + feste Werte
 *   node web/tools/lamamines_logic_dump.js - < f.json # dazu Faelle aus der Eingabe
 *
 * Wird von tests/audit_chiplogic.py aufgerufen, um Python
 * (games/lamamines_logic.py) und JavaScript (web/js/games/lamamines_logic.js)
 * zu vergleichen: alle Multiplikatoren, Auszahlungen, Chancen, Texte, Felder
 * aus Seeds und die clean_*-Pruefungen. Faelle (optional): fields
 * [[grid, mines, s, c, n]], make [[seed, grid, mines]], rounds [Rohdaten],
 * history [Rohdaten], marks [Rohdaten].
 */
"use strict";
const fs = require("fs");
const path = require("path");
global.window = global;
global.PG = {};
for (const f of ["seedrand.js", "fairness.js", "lamamines_logic.js"]) require(path.join(__dirname, "..", "js", "games", f));
const L = global.PG.lamaMinesLogic;

let cases = {};
const arg = process.argv[2];
if (arg === "-") cases = JSON.parse(fs.readFileSync(0, "utf8") || "{}");
else if (arg) cases = JSON.parse(fs.readFileSync(arg, "utf8"));

const mult = {}, safe = {}, next = {}, expected = {}, payout = {}, ladder = {}, allsafe = {};
for (const g of L.GRIDS) {
  for (let m = 1; m <= L.maxMines(g); m++) {
    const key = g + ":" + m;
    mult[key] = [];
    safe[key] = [];
    allsafe[key] = [];
    next[key] = [];
    expected[key] = [];
    payout[key] = [];
    for (let k = 0; k <= L.gems(g, m) + 1; k++) {
      mult[key].push(L.mult100(g, m, k));
      safe[key].push(L.safePercent(g, m, k));
      allsafe[key].push(L.allSafePercent(g, m, k));
      next[key].push(L.nextMult100(g, m, k));
      expected[key].push(L.expectedReturn(g, m, k));
      payout[key].push([10, 55, 10000].map((b) => L.payout(b, g, m, k)));
    }
    ladder[key] = [0, 1, L.gems(g, m) - 2, L.gems(g, m)].map((k) => L.ladder(g, m, k));
  }
}
const fmt = [];
for (const v of [100, 105, 1142, 16042, 10000000]) for (const s of [".", ","]) fmt.push(L.fmtMult(v, s));
const fmtpct = [];
for (const v of [0, 5, 845, 880, 1000]) for (const s of [".", ","]) fmtpct.push(L.fmtPercent(v, s));
const combs = [];
for (let n = 0; n <= 49; n++) for (let k = -1; k <= n + 1; k++) combs.push(L.comb(n, k));

const out = {
  key: L.KEY, grids: L.GRIDS, default_grid: L.DEFAULT_GRID, default_mines: L.DEFAULT_MINES, bets: L.BETS,
  house: L.HOUSE, max_mult100: L.MAX_MULT100, history: L.HISTORY, presets: L.MINE_PRESETS,
  brave: L.BRAVE_MULT, legend: L.LEGEND_MULT,
  mult, safe, allsafe, next, expected, payout, ladder, fmt, fmtpct, combs,
  clamp: [[4, 0], [4, 20], [7, 48], [7, 49], [5, 3.7]].map(([g, m]) => L.clampMines(g, m)),
  chance: [[5, 3, 0], [5, 3, 22], [4, 15, 1]].map(([g, m, k]) => L.safeChance(g, m, k)),
};
if (cases.fields) out.fields = cases.fields.map(([g, m, s, c, n]) => L.fieldFromSeed(g, m, s, c, n));
if (cases.make) out.make = cases.make.map(([seed, g, m]) => L.makeField(new PG.seedrand.Rand(seed), g, m));
if (cases.rounds) out.rounds = cases.rounds.map((r) => L.cleanRound(r));
if (cases.history) out.clean_history = cases.history.map((r) => L.cleanHistory(r));
if (cases.carry) {
  // Auszahlungen [[bet, grid, mines, k], ...] mit Uebertrag ab 0
  let carry = 0;
  out.carry = cases.carry.map(([b, g, m, k]) => {
    const [win, c] = L.payoutCarry(b, g, m, k, carry);
    carry = c;
    return [win, c];
  });
  out.carry_clean = [42, 100, -1, true, "5", null].map((v) => L.cleanCarry(v));
}
if (cases.marks) out.marks = cases.marks.map((r) => L.cleanMarks(r));
process.stdout.write(JSON.stringify(out));
