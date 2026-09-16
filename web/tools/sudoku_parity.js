#!/usr/bin/env node
/*
 * sudoku_parity.js - erzeugt Sudoku-Rätsel mit web/js/games/sudoku_gen.js
 * =======================================================================
 * Für tests/audit_sudoku.py: gibt X-Sudoku-, Mini-6x6- und Tages-Rätsel als
 * JSON aus, damit der Test sie mit games/sudoku_gen.py vergleichen kann
 * (Desktop und Browser müssen bitgenau dieselben Rätsel liefern). Dazu die
 * Hashes der klassischen Level (dürfen sich nie ändern) und die eingebauten
 * Killer-Level aus sudoku_killer.js.
 *
 *   node web/tools/sudoku_parity.js '{"variants": [["x", 0, 1], ...], "dates": ["2026-09-16"]}'
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");
const crypto = require("crypto");

const JS = path.resolve(__dirname, "..", "js");
const store = {};
const ctx = { console, Math, Date, JSON };
ctx.window = ctx;
ctx.localStorage = {
  getItem: (k) => (k in store ? store[k] : null),
  setItem: (k, v) => (store[k] = String(v)),
  removeItem: (k) => delete store[k],
};
ctx.navigator = { language: "de" };
ctx.document = { createElement: () => ({ getContext: () => ({}) }), addEventListener() {} };
vm.createContext(ctx);
for (const f of ["core/util.js", "games/seedrand.js", "games/sudoku_gen.js", "games/sudoku_killer.js"]) {
  vm.runInContext(fs.readFileSync(path.join(JS, f), "utf8"), ctx, { filename: f });
}
const PG = ctx.PG;
const gen = PG.sudokuGen;

const req = JSON.parse(process.argv[2] || "{}");
const out = { variants: [], daily: [], classic: {}, killer: PG.sudokuKiller };
for (const [variant, diff, level] of req.variants || []) {
  const [p, s] = gen.generateVariant(variant, diff, level);
  out.variants.push({ variant, diff, level, puzzle: p.join(""), solution: s.join("") });
}
for (const date of req.dates || []) {
  const [p, s, d] = gen.generateDaily(date);
  out.daily.push({ date, diff: d, puzzle: p.join(""), solution: s.join("") });
}
for (const [d, lv] of req.classic || []) {
  const [p, s] = gen.generate(d, lv);
  out.classic[d + "," + lv] = crypto.createHash("sha256").update(p.join("") + "|" + s.join("")).digest("hex").slice(0, 16);
}
process.stdout.write(JSON.stringify(out));
