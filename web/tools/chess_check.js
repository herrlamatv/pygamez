#!/usr/bin/env node
/*
 * chess_check.js - prüft die Web-Schach-Engine (js/games/chess_engine.js) in Node.
 *
 *   node web/tools/chess_check.js                 # eingebaute Perft-Werte
 *   node web/tools/chess_check.js referenz.json   # Vergleich mit Python-Werten
 *
 * Die Referenzdatei schreibt tests/audit_chess.py: je Stellung Perft, statische
 * Bewertung, alle legalen Züge in SAN und das Ergebnis einer Suche mit festem
 * Knotenlimit (bester Zug, Wert, Knoten, Tiefe). JavaScript muss dieselben
 * Zahlen liefern - dann spielen Desktop und Browser gleich stark.
 * Exit-Code 1 bei Abweichungen.
 */
"use strict";

const fs = require("fs");
const path = require("path");

globalThis.PG = {};
require(path.join(__dirname, "..", "js", "games", "chess_engine.js"));
const E = globalThis.PG.chessEngine;

let fails = 0;
function check(ok, label, detail) {
  console.log((ok ? "  OK   " : "  FAIL ") + label + (!ok && detail ? "  -> " + detail : ""));
  if (!ok) fails++;
}

const refFile = process.argv[2];
if (!refFile) {
  const cases = [
    ["Grundstellung", E.START_FEN, [20, 400, 8902, 197281], false],
    ["Kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", [48, 2039, 97862], false],
    ["Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", [14, 191, 2812, 43238], false],
    ["Position 4", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", [6, 264, 9467], false],
    ["Position 5", "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", [44, 1486, 62379], false],
    ["Chess960 A", "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9", [21, 528, 12189, 326672], true],
    ["Chess960 B", "2nnrbkr/p1qppppp/8/1ppb4/6PP/3PP3/PPP2P2/BQNNRBKR w HEhe - 1 9", [21, 807, 18002, 667366], true],
  ];
  for (const [name, fen, exp, c960] of cases) {
    const pos = E.fromFen(fen, c960);
    const got = exp.map((_, d) => E.perft(pos, d + 1));
    check(JSON.stringify(got) === JSON.stringify(exp), name + ": Perft " + JSON.stringify(exp), JSON.stringify(got));
  }
  check(E.chess960Backrank(518) === "RNBQKBNR", "Chess960 Nr. 518 = Normalstellung");
  process.exit(fails ? 1 : 0);
}

const ref = JSON.parse(fs.readFileSync(refFile, "utf8"));
let n = 0;
for (const c of ref.cases) {
  const pos = E.fromFen(c.fen, c.c960);
  const tag = c.fen.split(" ").slice(0, 2).join(" ");
  const perft = c.perft.map((_, d) => E.perft(pos, d + 1));
  check(JSON.stringify(perft) === JSON.stringify(c.perft), "Perft " + tag, JSON.stringify(perft) + " != " + JSON.stringify(c.perft));
  check(pos.fen() === c.fen_out, "FEN " + tag, pos.fen() + " != " + c.fen_out);
  check(pos.evaluate() === c.eval, "Bewertung " + tag, pos.evaluate() + " != " + c.eval);
  const legal = pos.legalMoves();
  const sans = legal.map((m) => pos.san(m, legal));
  check(JSON.stringify(sans) === JSON.stringify(c.sans), "SAN aller Züge " + tag, sans.join(" ") + " != " + c.sans.join(" "));
  const ucis = legal.map((m) => pos.uci(m));
  check(JSON.stringify(ucis) === JSON.stringify(c.ucis), "UCI aller Züge " + tag);
  check(pos.status() === c.status, "Status " + tag, pos.status() + " != " + c.status);
  for (const s of c.searches) {
    const level = Object.assign({}, E.LEVELS[s.level]);
    if (s.spread != null) level.spread = s.spread;
    const search = new E.Search(pos, level, { nodeLimit: s.node_limit, maxDepth: s.max_depth });
    E.runToEnd(search);
    const got = { best: search.bestMove ? pos.uci(search.bestMove) : "", score: search.bestScore, nodes: search.nodes, depth: search.depthDone };
    const want = { best: s.best, score: s.score, nodes: s.nodes, depth: s.depth };
    check(JSON.stringify(got) === JSON.stringify(want), "Suche Stufe " + (s.level + 1) + " (" + s.node_limit + " Knoten) " + tag,
      JSON.stringify(got) + " != " + JSON.stringify(want));
  }
  n++;
}
console.log(n + " Stellungen verglichen, " + fails + " Abweichungen");
process.exit(fails ? 1 : 0);
