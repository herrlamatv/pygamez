#!/usr/bin/env node
/*
 * build-i18n.js - erzeugt web/js/lang/<code>.js und web/js/wiki/<code>.js
 * aus lang/*.json und lamawiki/*.json der Desktop-Version.
 *
 *   node web/tools/build-i18n.js
 *
 * Die Web-Version kann (wegen file://) keine JSON-Dateien per fetch laden,
 * deshalb landen die Daten als Skripte in window.PG_LANG / window.PG_WIKI.
 */
"use strict";

const fs = require("fs");
const path = require("path");

const WEB = path.resolve(__dirname, "..");
const ROOT = path.resolve(WEB, "..");
const CODES = ["de", "en", "fr", "es", "pt", "pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr"];

function findJson(base, code) {
  for (const p of [path.join(base, code + ".json"), path.join(base, "lang.expansion", code + ".json")]) {
    if (fs.existsSync(p)) return JSON.parse(fs.readFileSync(p, "utf8").replace(/^﻿/, ""));
  }
  throw new Error(`${code}.json nicht gefunden in ${base}`);
}

function sortKeys(obj) {
  const out = {};
  for (const k of Object.keys(obj).sort()) out[k] = obj[k];
  return out;
}

for (const [dir, global, src, pretty] of [
  ["lang", "PG_LANG", "lang", true],
  ["wiki", "PG_WIKI", "lamawiki", false],
]) {
  fs.mkdirSync(path.join(WEB, "js", dir), { recursive: true });
  for (const code of CODES) {
    const data = findJson(path.join(ROOT, src), code);
    const body = pretty ? JSON.stringify(sortKeys(data), null, 1) : JSON.stringify(data);
    const text =
      `// Automatisch aus ${src}/${code}.json erzeugt (PyGameZ Web).\n` +
      `window.${global} = window.${global} || {};\nwindow.${global}[${JSON.stringify(code)}] = ` +
      body +
      ";\n";
    fs.writeFileSync(path.join(WEB, "js", dir, code + ".js"), text, "utf8");
  }
  console.log(`js/${dir}: ${CODES.length} Sprachen geschrieben`);
}
