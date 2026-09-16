#!/usr/bin/env node
/*
 * tetris_replay.js - spielt eine Eingabefolge mit web/js/games/tetris_core.js nach
 * ===============================================================================
 * Gegenstück zu tests/audit_tetris.py (audit_web): Der Test schickt per stdin
 *   {"seed": n, "script": [["left"], ["tick", 0.2], ["garbage", 3], ...], "ai_script": n}
 * und vergleicht die Ausgabe mit tetris_core.py - Verlauf, Endfeld und eine
 * komplette KI-Partie müssen bitgenau übereinstimmen.
 *
 *   echo '{"seed":1,"script":[["drop"]],"ai_script":10}' | node web/tools/tetris_replay.js
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const GAMES = path.resolve(__dirname, "..", "js", "games");
const sandbox = { window: { PG: {} }, console };
vm.createContext(sandbox);
for (const file of ["seedrand.js", "tetris_core.js"]) {
  vm.runInContext(fs.readFileSync(path.join(GAMES, file), "utf8"), sandbox, { filename: file });
}
const PG = sandbox.window.PG;
const core = PG.tetrisCore;
const ai = PG.tetrisAI;

const input = JSON.parse(fs.readFileSync(0, "utf8"));

const b = new core.Board(input.seed);
const trace = [];
for (const step of input.script) {
  if (b.dead) break;
  const name = step[0];
  if (name === "tick") b.tick(step[1]);
  else if (name === "garbage") b.receive(step[1]);
  else if (name === "soft") b.softStep();
  else if (name === "ai") for (const act of ai.plan(b, 2, null)) ai.apply(b, act);
  else ai.apply(b, name);
  trace.push([b.score, b.lines, b.kind || "-", b.rot, b.x, b.y, b.pendingLines(), b.dead ? 1 : 0]);
}

const b2 = new core.Board(input.seed + 1);
const rng2 = new PG.seedrand.Rand(5);
for (let i = 0; i < input.ai_script; i++) {
  if (b2.dead) break;
  for (const act of ai.plan(b2, 2, null)) ai.apply(b2, act);
  if (rng2.random() < 0.2) {
    b2.receive(rng2.randint(1, 3));
    b2.tick(0.6);
  }
}

process.stdout.write(JSON.stringify({
  trace, rows: b.rows, score: b.score, lines: b.lines,
  ai_rows: b2.rows, ai_lines: b2.lines, ai_score: b2.score, ai_pieces: b2.pieces,
}));
