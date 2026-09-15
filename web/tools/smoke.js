#!/usr/bin/env node
/*
 * smoke.js - startet tools/smoketest.html in headless Chrome und meldet Fehler.
 *
 *   node web/tools/smoke.js <GameId...|all> [--shot] [--idle] [--frames N]
 *                           [--mode M] [--keys "Right,Up@30"] [--lang en]
 *
 * --shot   speichert das letzte Bild nach web/tools/shots/<GameId>-<mode>.png
 * Exit-Code 1, wenn ein Spiel Fehler geworfen hat.
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
const ids = [];
for (let i = 0; i < args.length; i++) {
  const a = args[i];
  if (a === "--shot" || a === "--idle" || a === "--verbose") flags[a.slice(2)] = true;
  else if (a.startsWith("--")) flags[a.slice(2)] = args[++i];
  else ids.push(a);
}

let games = ids;
if (!games.length || games[0] === "all") {
  const src = fs.readFileSync(path.join(WEB, "js", "manifest.js"), "utf8");
  games = [...src.matchAll(/id:\s*"([A-Za-z0-9_]+)"/g)].map((m) => m[1]);
}

function chrome(extra, url, timeoutMs) {
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "pgz-smoke-"));
  try {
    return execFileSync(
      CHROME,
      [
        "--headless=new", "--disable-gpu", "--no-first-run", "--no-default-browser-check",
        "--allow-file-access-from-files", "--mute-audio", "--hide-scrollbars",
        `--user-data-dir=${profile}`, ...extra, url,
      ],
      { encoding: "utf8", timeout: timeoutMs, maxBuffer: 64 * 1024 * 1024, stdio: ["ignore", "pipe", "pipe"] }
    );
  } finally {
    try {
      fs.rmSync(profile, { recursive: true, force: true });
    } catch (e) {}
  }
}

function pageUrl(id) {
  const q = new URLSearchParams({ game: id });
  if (flags.idle) q.set("idle", "1");
  if (flags.frames) q.set("frames", flags.frames);
  if (flags.mode) q.set("mode", flags.mode);
  if (flags.keys) q.set("keys", flags.keys);
  if (flags.lang) q.set("lang", flags.lang);
  if (flags.seed) q.set("seed", flags.seed);
  const file = path.join(WEB, "tools", "smoketest.html").replace(/\\/g, "/");
  return "file:///" + file.replace(/^\/+/, "") + "?" + q.toString();
}

let failed = 0;
const budget = Number(flags.budget || 60000);
for (const id of games) {
  const url = pageUrl(id);
  let html = "";
  try {
    html = chrome([`--virtual-time-budget=${budget}`, "--dump-dom"], url, budget + 90000);
  } catch (e) {
    console.log(`✗ ${id}: Chrome-Aufruf fehlgeschlagen: ${e.message.split("\n")[0]}`);
    failed++;
    continue;
  }
  const m = /<pre id="result">([\s\S]*?)<\/pre>/.exec(html);
  let res = null;
  try {
    res = JSON.parse(m[1].replace(/&quot;/g, '"').replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&"));
  } catch (e) {
    console.log(`✗ ${id}: kein Ergebnis (Test lief nicht zu Ende) -> ${m ? m[1].slice(0, 200) : "?"}`);
    failed++;
    continue;
  }
  const ok = !res.errors.length;
  if (!ok) failed++;
  const modes = res.modes.map((mm) => `${mm.mode}: ${mm.frames}f, over=${mm.gameOvers}, max=${mm.maxScore}`).join(" | ");
  console.log(`${ok ? "✓" : "✗"} ${id} (${res.meta ? res.meta.name : "?"}) ${modes}`);
  for (const err of res.errors) {
    console.log(`    [${err.mode || ""} ${err.where} @${err.frame != null ? err.frame : "-"}] ${err.message}`);
    if (err.stack) console.log("      " + err.stack.split("\n").slice(0, 4).join("\n      "));
  }
  if (flags.shot) {
    const modesToShoot = flags.mode ? [flags.mode] : res.meta ? res.meta.modes : ["single"];
    const shotsDir = path.join(WEB, "tools", "shots");
    fs.mkdirSync(shotsDir, { recursive: true });
    for (const mode of modesToShoot) {
      const q = new URL(url);
      q.searchParams.set("mode", mode);
      const out = path.join(shotsDir, `${id}-${mode}.png`);
      try {
        chrome([`--virtual-time-budget=${budget}`, `--screenshot=${out}`, "--window-size=800,600"], q.toString(), budget + 90000);
        console.log(`    Screenshot: ${out}`);
      } catch (e) {
        console.log(`    Screenshot fehlgeschlagen: ${e.message.split("\n")[0]}`);
      }
    }
  }
}
process.exit(failed ? 1 : 0);
