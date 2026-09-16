#!/usr/bin/env node
/*
 * build-wordlists.js - erzeugt web/js/games/wordle_words/<code>.js aus den
 * Wortlisten in woordlistz/ der Desktop-Version.
 *
 *   node web/tools/build-wordlists.js
 *
 * Die Web-Version kann (wegen file://) keine Textdateien per fetch laden,
 * deshalb wird je Sprache eine kleine Skriptdatei geschrieben, die sich bei
 * PG.wordleWords anmeldet. Wordle lädt davon nur die Datei der aktuell
 * eingestellten Sprache nach - alle 14 auf einmal wären mehrere Megabyte.
 *
 * Die Wörter stehen ohne Trennzeichen hintereinander (jedes Wort genau 5
 * Zeichen) - das spart gegenüber einem JSON-Array rund 40 % Dateigröße.
 */
"use strict";

const fs = require("fs");
const path = require("path");

const WEB = path.resolve(__dirname, "..");
const ROOT = path.resolve(WEB, "..");
const LISTS = path.join(ROOT, "woordlistz");
const OUT = path.join(WEB, "js", "games", "wordle_words");
const CODES = ["de", "en", "fr", "es", "pt", "pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr"];

/** Liest eine Wortliste und prüft sie (genau 5 Großbuchstaben A-Z). */
function readList(code, name) {
  const file = path.join(LISTS, code, name + ".txt");
  if (!fs.existsSync(file)) throw new Error(`${file} fehlt - erst woordlistz/build_wordlists.py laufen lassen`);
  const words = fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .map((w) => w.trim().toUpperCase())
    .filter((w) => /^[A-Z]{5}$/.test(w));
  return [...new Set(words)].sort();
}

fs.mkdirSync(OUT, { recursive: true });
let total = 0;
for (const code of CODES) {
  const answers = readList(code, "answers");
  const allowed = readList(code, "allowed");
  const text =
    `// Automatisch aus woordlistz/${code}/ erzeugt (PyGameZ Web) - nicht von Hand ändern.\n` +
    `// ${answers.length} Lösungswörter, ${allowed.length} erlaubte Rateworte.\n` +
    `PG.wordleWords.add(${JSON.stringify(code)}, ${JSON.stringify(answers.join(""))}, ${JSON.stringify(allowed.join(""))});\n`;
  const file = path.join(OUT, code + ".js");
  fs.writeFileSync(file, text, "utf8");
  total += text.length;
  console.log(`js/games/wordle_words/${code}.js: ${answers.length} Lösungswörter, ${allowed.length} Rateworte (${Math.round(text.length / 1024)} KB)`);
}
console.log(`${CODES.length} Sprachen, zusammen ${Math.round(total / 1024)} KB (es wird immer nur eine davon geladen).`);
