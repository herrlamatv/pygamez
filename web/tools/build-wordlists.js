#!/usr/bin/env node
/*
 * build-wordlists.js - erzeugt web/js/games/wordle_words/<code><länge>.js aus
 * den Wortlisten in woordlistz/ der Desktop-Version.
 *
 *   node web/tools/build-wordlists.js
 *
 * Die Web-Version kann (wegen file://) keine Textdateien per fetch laden,
 * deshalb wird je Sprache UND Wortlänge eine kleine Skriptdatei geschrieben,
 * die sich bei PG.wordleWords anmeldet. Wordle lädt davon nur die Datei der
 * gerade gespielten Sprache/Länge nach - alle 56 auf einmal wären viele
 * Megabyte.
 *
 * Dateinamen wie in woordlistz/: 5 Buchstaben behalten den alten Namen
 * (de.js aus answers.txt/allowed.txt), die anderen Längen bekommen die Zahl
 * angehängt (de4.js, de6.js, de7.js aus answers4.txt/allowed4.txt ...).
 *
 * Die Wörter stehen ohne Trennzeichen hintereinander (jedes Wort genau N
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
const LENGTHS = [4, 5, 6, 7];

/** Dateiname ohne Endung: 5 Buchstaben ohne Zahl, sonst mit. */
function suffix(length) {
  return length === 5 ? "" : String(length);
}

/** Liest eine Wortliste und prüft sie (genau 'length' Großbuchstaben A-Z). */
function readList(code, name, length) {
  const file = path.join(LISTS, code, name + ".txt");
  if (!fs.existsSync(file)) throw new Error(`${file} fehlt - erst woordlistz/build_wordlists.py laufen lassen`);
  const re = new RegExp(`^[A-Z]{${length}}$`);
  const words = fs
    .readFileSync(file, "utf8")
    .split(/\r?\n/)
    .map((w) => w.trim().toUpperCase())
    .filter((w) => re.test(w));
  return [...new Set(words)].sort();
}

fs.mkdirSync(OUT, { recursive: true });
let total = 0;
let biggest = 0;
for (const code of CODES) {
  const sizes = [];
  for (const length of LENGTHS) {
    const s = suffix(length);
    const answers = readList(code, "answers" + s, length);
    const allowed = readList(code, "allowed" + s, length);
    // Lösungswörter sind immer auch erlaubt - sie stehen nur einmal in der
    // Datei (add() fügt sie den Rateworten wieder hinzu).
    const inAnswers = new Set(answers);
    const extra = allowed.filter((w) => !inAnswers.has(w));
    const text =
      `// Automatisch aus woordlistz/${code}/ erzeugt (PyGameZ Web) - nicht von Hand ändern.\n` +
      `// ${length} Buchstaben: ${answers.length} Lösungswörter, ${allowed.length} erlaubte Rateworte.\n` +
      `PG.wordleWords.add(${JSON.stringify(code)}, ${JSON.stringify(answers.join(""))}, ` +
      `${JSON.stringify(extra.join(""))}, ${length});\n`;
    fs.writeFileSync(path.join(OUT, code + s + ".js"), text, "utf8");
    total += text.length;
    biggest = Math.max(biggest, text.length);
    sizes.push(`${length}: ${answers.length}/${allowed.length} (${Math.round(text.length / 1024)} KB)`);
  }
  console.log(`js/games/wordle_words/${code}*.js  ${sizes.join("  ")}`);
}
console.log(
  `${CODES.length} Sprachen x ${LENGTHS.length} Längen, zusammen ${Math.round(total / 1024)} KB ` +
    `(größte Datei ${Math.round(biggest / 1024)} KB - geladen wird immer nur eine).`
);
