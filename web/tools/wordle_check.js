#!/usr/bin/env node
/*
 * wordle_check.js - rechnet Wordle-Logik der Web-Version in Node nach, damit
 * tests/audit_wordle.py sie mit games/wordle.py vergleichen kann.
 *
 *   node web/tools/wordle_check.js anfragen.json
 *
 * anfragen.json:
 *   {
 *     "daily":    [["de", 5, "2026-09-16"], ...],   // Tageswort je Sprache/Länge/Datum
 *     "counts":   [["de", 5], ...],                  // [Lösungswörter, Rateworte]
 *     "evaluate": [["GUESS", "ANSWER"], ...],
 *     "hard":     [["GUESS", [["WORT", ["correct", ...]], ...]], ...],
 *     "share":    [[mode, lang, length, boards, hard, colorblind, date], ...],
 *     "record":   [[mode, won, tries, day|null], ...]  // nacheinander in EINE Statistik
 *   }
 *
 * Ausgabe (stdout): dasselbe Objekt mit den Ergebnissen der Web-Fassung.
 * Geladen werden die echten Dateien aus web/js/ (seedrand.js, wordle_words.js,
 * wordle_words/<code><n>.js, wordle.js) mit einer kleinen PG-Attrappe.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const JS = path.resolve(__dirname, "..", "js");

global.window = global;
global.document = { currentScript: null };
global.PG = {
  Game: class {},
  register() {},
  addStrings() {},
  ui: {},
  draw: {},
  t: (key) => key,
  mod: (a, b) => ((a % b) + b) % b,
};

function run(rel) {
  const file = path.join(JS, rel);
  vm.runInThisContext(fs.readFileSync(file, "utf8"), { filename: file });
}

run("games/seedrand.js");
run("games/wordle_words.js");
run("games/wordle.js");

const loaded = new Set();
function words(lang, length) {
  const key = lang + length;
  if (!loaded.has(key)) {
    run("games/wordle_words/" + lang + (length === 5 ? "" : String(length)) + ".js");
    loaded.add(key);
  }
  return PG.wordleWords;
}

const req = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const out = {};
const W = PG.wordle;

if (req.daily) {
  out.daily = req.daily.map(([lang, length, date]) => W.dailyAnswer(words(lang, length).wordsFor(lang, length), lang, length, date));
}
if (req.counts) {
  out.counts = req.counts.map(([lang, length]) => {
    const ww = words(lang, length);
    return [ww.wordsFor(lang, length).length, ww.allowedFor(lang, length).size];
  });
}
if (req.evaluate) out.evaluate = req.evaluate.map(([g, a]) => W.evaluate(g, a));
if (req.hard) out.hard = req.hard.map(([g, history]) => W.hardProblem(g, history));
if (req.share) out.share = req.share.map((args) => W.shareText(...args));
if (req.record) {
  const data = { stats: {}, daily: {}, best: {} };
  out.record = req.record.map(([mode, won, tries, day]) => W.recordGame(data, mode, "de", 5, won, tries, day));
}
process.stdout.write(JSON.stringify(out));
