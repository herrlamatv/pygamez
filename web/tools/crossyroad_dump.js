#!/usr/bin/env node
/*
 * crossyroad_dump.js - gibt Reihen der Crossy-Road-Welt als JSON aus (ohne Browser).
 *
 *   node web/tools/crossyroad_dump.js <seed> <anzahl>          -> Reihen 0..anzahl-1
 *   node web/tools/crossyroad_dump.js daily <YYYY-MM-DD> <n>    -> Tagesstrecke eines Datums
 *   node web/tools/crossyroad_dump.js hash <r> <c>              -> decoHash
 *
 * Der Python-Audit (tests/audit_crossyroad.py) vergleicht die Ausgabe mit
 * games/crossyroad_world.py: beide müssen Reihe für Reihe identisch sein.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const GAMES = path.resolve(__dirname, "..", "js", "games");
const ctx = { window: { PG: {} }, console };
vm.createContext(ctx);
for (const f of ["seedrand.js", "crossyroad_world.js"]) {
  vm.runInContext(fs.readFileSync(path.join(GAMES, f), "utf8"), ctx, { filename: f });
}
const PG = ctx.window.PG;
const cw = PG.crossyWorld;

const args = process.argv.slice(2);
let out;
if (args[0] === "daily") {
  const seed = PG.seedrand.dailySeed("crossy", args[1]);
  out = { seed, rows: new cw.World(seed).export(Number(args[2] || 50)) };
} else if (args[0] === "hash") {
  out = cw.decoHash(Number(args[1]), Number(args[2]));
} else {
  const seed = Number(args[0] || 1);
  out = { seed, rows: new cw.World(seed).export(Number(args[1] || 50)) };
}
process.stdout.write(JSON.stringify(out));
