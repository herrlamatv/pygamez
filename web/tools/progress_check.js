#!/usr/bin/env node
/*
 * progress_check.js - Prüfstand für Erfolge, Statistik und Profile (headless Chrome)
 * =================================================================================
 * Startet tools/progresstest.html: dort werden ein alter Speicherstand
 * migriert, Profile angelegt und gewechselt, Erfolge samt Punkten und Level
 * freigeschaltet, die Statistik-Datei (.pgzlamastats) exportiert und wieder
 * eingelesen und alle fünf Reiter des Fortschritt-Screens in mehreren
 * Sprachen gezeichnet.
 *
 *   node web/tools/progress_check.js [--budget 120000]
 *
 * Exit-Code 1, wenn eine Prüfung fehlschlägt. Gegenstück zu
 * tests/progress_audit.py der Desktop-Version.
 */
"use strict";

const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const WEB = path.resolve(__dirname, "..");
const CHROME_CANDIDATES = [
  process.env.CHROME,
  "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
  path.join(process.env.LOCALAPPDATA || "", "Google\\Chrome\\Application\\chrome.exe"),
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
].filter(Boolean);
const CHROME = CHROME_CANDIDATES.find((p) => fs.existsSync(p));
if (!CHROME) {
  console.error("Kein Chrome/Edge gefunden (Umgebungsvariable CHROME setzen).");
  process.exit(2);
}

const args = process.argv.slice(2);
const flags = {};
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a.startsWith("--")) flags[a.slice(2)] = args[++i];
}
const budget = Number(flags.budget || 120000);

const file = path.join(WEB, "tools", "progresstest.html").replace(/\\/g, "/");
const url = "file:///" + file.replace(/^\/+/, "");
const profile = fs.mkdtempSync(path.join(os.tmpdir(), "pgz-progress-"));
let html = "";
try {
  html = execFileSync(
    CHROME,
    [
      "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
      "--allow-file-access-from-files", "--mute-audio", "--hide-scrollbars",
      `--user-data-dir=${profile}`, `--virtual-time-budget=${budget}`, "--dump-dom", url,
    ],
    { encoding: "utf8", timeout: budget + 120000, maxBuffer: 64 * 1024 * 1024, stdio: ["ignore", "pipe", "pipe"] }
  );
} catch (e) {
  console.error("Chrome-Aufruf fehlgeschlagen:", e.message.split("\n")[0]);
  process.exit(2);
} finally {
  try {
    fs.rmSync(profile, { recursive: true, force: true });
  } catch (e) {}
}

const m = /<pre id="result">([\s\S]*?)<\/pre>/.exec(html);
let res = null;
try {
  res = JSON.parse(m[1].replace(/&quot;/g, '"').replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&"));
} catch (e) {
  console.error("Kein Ergebnis (Test lief nicht zu Ende):", m ? m[1].slice(0, 400) : "?");
  process.exit(2);
}

let failed = 0;
for (const c of res.checks) {
  if (!c.ok) failed++;
  console.log(`  ${c.ok ? "OK  " : "FAIL"} ${c.label}${c.ok || !c.detail ? "" : "  -> " + c.detail.split("\n")[0]}`);
}
for (const e of res.errors) {
  failed++;
  console.log("  FAIL Laufzeitfehler: " + String(e).split("\n").slice(0, 3).join(" | "));
}
console.log("");
console.log(failed ? `FEHLGESCHLAGEN: ${failed} von ${res.checks.length}` : `Alle ${res.checks.length} Prüfungen bestanden.`);
process.exit(failed ? 1 : 0);
