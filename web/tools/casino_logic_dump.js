#!/usr/bin/env node
/*
 * casino_logic_dump.js - gibt die Casino-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/casino_logic_dump.js
 *
 * Wird von tests/audit_casino.py aufgerufen, um Python (games/casino_logic.py)
 * und JavaScript (web/js/games/casino_logic.js) zu vergleichen: Tabellen,
 * Treffererkennung auf einem feinen Raster, Auswertung fester Walzenbilder und
 * die exakte Auszahlungsquote.
 */
"use strict";
const path = require("path");
global.PG = {};
require(path.join(__dirname, "..", "js", "games", "casino_logic.js"));
const L = global.PG.casinoLogic;

const grid = [];
const STEP = 0.05;
for (let x = 0.013; x < L.TABLE_W; x += STEP) {
  for (let y = 0.011; y < L.TABLE_H; y += STEP) grid.push(L.hitTest(x, y));
}
const keys = [...new Set(grid.filter(Boolean))].sort();
const anchors = {};
for (const k of keys) anchors[k] = L.anchor(k);

const evals = [];
for (let i = 0; i < 400; i++) {
  const stops = L.STRIPS.map((s, r) => (i * (7 + r * 3) + r * 11) % s.length);
  const res = L.evaluate(L.window(stops), 2, i % 5 === 0);
  evals.push([stops, res.total, res.lines.length, res.scatter, res.jackpot]);
}

process.stdout.write(JSON.stringify({
  wheel: L.WHEEL_ORDER, red: [...L.RED_NUMBERS].sort((a, b) => a - b), pays: L.PAYS,
  scatter_pays: L.SCATTER_PAYS, lines: L.LINES, strips: L.STRIPS, rtp_text: L.RTP_TEXT,
  free_spins: L.FREE_SPINS, free_mult: L.FREE_MULT, line_bets: L.LINE_BETS,
  grid_step: STEP, grid, anchors, evals, rtp: L.exactRtp(),
}));
