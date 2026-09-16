#!/usr/bin/env node
/*
 * geodash_replay.js - spielt Geometry-Dash-Eingaben mit dem JS-Kern nach
 * ======================================================================
 * Beweist, dass web/js/games/geodash_core.js bitgenau dasselbe rechnet wie
 * games/geodash_core.py: Die Solver-Lösungen aus devtools/geodash_proofs.json
 * werden auf den Leveln aus games/levels/geodash.json abgespielt; ausgegeben
 * werden Endzustand und Zwischen-Hashes (alle 500 Schritte) als JSON. Der
 * Python-Test (tests/audit_geodash.py) vergleicht sie mit seinen eigenen.
 *
 *   node web/tools/geodash_replay.js                  # alle eingebauten Level
 *   node web/tools/geodash_replay.js --cases f.json   # eigene Fälle:
 *        [{"id": "...", "level": {...}, "toggles": [..], "every": 250,
 *          "start": null | Startblock}, ...]
 *
 * Exit-Code 1, wenn ein eingebautes Level mit seiner Lösung nicht ins Ziel
 * kommt (oder nicht alle Münzen einsammelt).
 */
"use strict";

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..", "..");
const core = require(path.join(ROOT, "web", "js", "games", "geodash_core.js"));

const args = process.argv.slice(2);
const casesIdx = args.indexOf("--cases");

function play(level, toggles, every, start) {
  const lv = core.compile(level);
  const st = start ? core.newState(lv, start) : null;
  const { state, trace } = core.runToggles(lv, toggles, null, every, st);
  return {
    won: state.won, dead: state.dead, coins: state.coins, coinCount: lv.coinCount,
    steps: state.step, hash: core.stateHash(state), state: core.stateStr(state), trace,
    progress: core.progress(state, lv), contentHash: core.contentHash(level),
  };
}

const out = {};
let failed = 0;
if (casesIdx >= 0) {
  const cases = JSON.parse(fs.readFileSync(args[casesIdx + 1], "utf8"));
  for (const c of cases) out[c.id] = play(c.level, c.toggles || [], c.every || 0, c.start || null);
} else {
  const levels = JSON.parse(fs.readFileSync(path.join(ROOT, "games", "levels", "geodash.json"), "utf8")).levels;
  const proofs = JSON.parse(fs.readFileSync(path.join(ROOT, "devtools", "geodash_proofs.json"), "utf8"));
  for (const level of levels) {
    const proof = proofs[level.id];
    if (!proof) {
      out[level.id] = { error: "keine Lösung" };
      failed++;
      continue;
    }
    const r = play(level, proof.toggles, 500, null);
    const all = (1 << r.coinCount) - 1;
    r.ok = !!r.won && !r.dead && r.coins === all && r.hash === proof.hash;
    if (!r.ok) failed++;
    out[level.id] = r;
  }
}
process.stdout.write(JSON.stringify(out));
process.exit(failed ? 1 : 0);
