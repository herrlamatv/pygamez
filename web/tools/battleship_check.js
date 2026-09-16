#!/usr/bin/env node
/*
 * battleship_check.js - prüft Regeln und KI des Web-Ports ohne Browser.
 *
 *   node web/tools/battleship_check.js [--games 200]
 *
 * Lädt web/js/games/battleship.js mit einem minimalen PG-Ersatz in eine
 * Node-VM und prüft dasselbe wie tests/audit_battleship.py für die Logik:
 * Platzierungsregeln (inkl. Berühr-Regel), Zufallsaufstellungen, Schüsse,
 * und dass jede KI-Stärke jede Flotte in <= 100 Schüssen ohne doppelten
 * Schuss versenkt - im Mittel schwer < mittel < leicht (wie in Python).
 * Exit-Code 1 bei Fehlern.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const args = process.argv.slice(2);
const GAMES = Number((args[args.indexOf("--games") + 1] || "").match(/^\d+$/) ? args[args.indexOf("--games") + 1] : 200);

// ----- minimaler PG-Ersatz (nur was die Logik braucht)
const PG = {
  TAU: Math.PI * 2,
  mod: (a, b) => ((a % b) + b) % b,
  rand: {
    random: Math.random,
    uniform: (a, b) => a + Math.random() * (b - a),
    randint: (a, b) => a + Math.floor(Math.random() * (b - a + 1)),
    choice: (arr) => arr[Math.floor(Math.random() * arr.length)],
  },
  addStrings() {},
  register() {},
  Game: class {},
};
const src = fs.readFileSync(path.join(__dirname, "..", "js", "games", "battleship.js"), "utf8");
vm.runInNewContext(src, { PG, console, Math, Set, Map, Uint8Array, Float64Array, Array, Object, Number, String });
const { Sea, ShotAI, N } = PG.battleship;

let fails = 0;
function check(ok, label, detail) {
  console.log(`  ${ok ? "OK  " : "FAIL"} ${label}${!ok && detail ? "  -> " + detail : ""}`);
  if (!ok) fails++;
}

console.log("\nRegeln");
const sea = new Sea();
check(!sea.canPlace(0, 0, 6, true) && !sea.canPlace(0, 6, 0, false), "Flugzeugträger ragt nicht über den Rand");
sea.place(0, 4, 2, true);
check(!sea.canPlace(1, 2, 4, false), "Überlappung wird abgelehnt");
check(sea.canPlace(1, 5, 2, true, true) && !sea.canPlace(1, 5, 2, true, false), "Berührung an der Seite nur mit touch=true");
check(sea.canPlace(4, 5, 7, true, true) && !sea.canPlace(4, 5, 7, true, false), "Berührung über Eck nur mit touch=true");
check(!sea.canPlace(4, 4, 7, true, false) && sea.canPlace(4, 4, 8, true, false), "ein Feld Abstand erlaubt (touch=false)");
let bad = 0;
for (const touch of [true, false]) {
  for (let i = 0; i < 150; i++) {
    const s = new Sea();
    if (!s.randomize(touch) || !s.complete()) bad++;
    for (const sh of s.ships) if (!s.canPlace(sh.idx, sh.r, sh.c, sh.horiz, touch, sh)) bad++;
  }
}
check(bad === 0, "300 Zufallsaufstellungen sind legal", bad + " kaputt");
const s2 = new Sea();
for (let i = 0; i < 5; i++) s2.place(i, i * 2, 0, true);
check(s2.fire(1, 0)[0] === "miss" && s2.fire(8, 0)[0] === "hit" && s2.fire(8, 0)[0] === null && s2.fire(8, 1)[0] === "sunk", "Wasser / Treffer / doppelt / versenkt");
const [grid, sunk, rem] = s2.knowledge();
check(grid[10] === 1 && sunk.length === 1 && sunk[0].join() === "80,81" && rem.length === 4, "knowledge(): nur öffentliches Wissen");

console.log(`\nKI (${GAMES} Partien je Stärke)`);
for (const touch of [true, false]) {
  const avgs = [];
  for (const [level, name] of [[0, "leicht"], [1, "mittel"], [2, "schwer"]]) {
    let max = 0, sum = 0, dup = 0;
    for (let g = 0; g < GAMES; g++) {
      const s = new Sea();
      s.randomize(touch);
      const ai = new ShotAI(level, touch);
      let shots = 0;
      while (!s.allSunk() && shots < 200) {
        const [r, c] = ai.choose(s);
        if (s.fire(r, c)[0] === null) dup++;
        shots++;
      }
      max = Math.max(max, shots);
      sum += shots;
    }
    avgs.push(sum / GAMES);
    check(max <= 100 && dup === 0, `${name} (Berühren ${touch ? "an" : "aus"}): <= 100 Schüsse, kein Feld doppelt`, `max=${max} doppelt=${dup}`);
    console.log(`         Durchschnitt ${(sum / GAMES).toFixed(1)}  (max ${max})`);
  }
  check(avgs[2] + 5 < avgs[1] && avgs[1] < avgs[0] - 5, `Berühren ${touch ? "an" : "aus"}: schwer < mittel < leicht (${avgs.map((a) => a.toFixed(1)).join(" / ")})`);
}

console.log(fails ? `\n${fails} FEHLER` : "\nALLE PRÜFUNGEN BESTANDEN");
process.exit(fails ? 1 : 0);
