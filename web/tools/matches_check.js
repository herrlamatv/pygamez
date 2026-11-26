#!/usr/bin/env node
/*
 * matches_check.js - Prüfstand für Match-Historie und Tages-Archiv (headless, Node)
 * ================================================================================
 * Lädt die echten Dateien aus web/js/ mit einer kleinen DOM-/Canvas-Attrappe
 * in einen vm-Kontext und prüft - als Gegenstück zu tests/audit_matches.py
 * (Teil 1-3) und tests/audit_dailyarchive.py (Teil 1-5):
 *
 * Daten     (1) PG.matches: neueste zuerst, Summen, Kappe CAP, Sieg/Niederlage/
 *               Remis, "Ersetzen", Info bereinigen, Modi, Speicher-Rundlauf
 *               (kompakt, ohne Gleitkommazahlen), kaputte Einträge, Profile.
 * Hooks     (2) App.matchSave (aus app.js herausgelöst): Latch, Rekord-Flag,
 *               Abbruch-Regel, matchManual, Ergebnis, Ersetzen; matchActive.
 *           (3) Spiele mit eigenen Matches: Blackjack, Video-Poker, Roulette,
 *               Slots, Labyrinth, Geometry Dash.
 * Archiv    (4) dailyarchive.js: dateOf, verfügbare Tage, Zeilen, Texte, Picker.
 *           (5) Crossy Road, Sudoku, Wordle: Archiv-Tage wie in Python.
 * Anzeige   (6) MatchScreen + Datums-Leisten/Picker zeichnen in 14 Sprachen
 *               fehlerfrei, im Bild und ohne Überlappung; Filter/Sortierung.
 *
 *   node web/tools/matches_check.js
 *
 * Exit-Code 1, wenn eine Prüfung fehlschlägt.
 */
"use strict";

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const JS = path.resolve(__dirname, "..", "js");
const FAILS = [];

function check(ok, label, detail) {
  console.log("  " + (ok ? "OK  " : "FAIL") + " " + label + (!ok && detail !== undefined ? "  -> " + JSON.stringify(detail) : ""));
  if (!ok) FAILS.push(label);
  return ok;
}

// ------------------------------------------------------------ DOM-Attrappe
/** Objekt, das jeden Zugriff/Aufruf schluckt (für DOM-Elemente). */
function anything() {
  const fn = function () {};
  const store = {};
  return new Proxy(fn, {
    get(_t, key) {
      if (key in store) return store[key];
      if (key === Symbol.toPrimitive) return () => 0;
      if (key === Symbol.iterator) return function* () {};
      if (key === "then") return undefined;
      if (key === "length") return 0;
      if (key === "getContext") return () => makeCtx();
      return anything();
    },
    set(_t, key, value) {
      store[key] = value;
      return true;
    },
    apply() {
      return anything();
    },
    construct() {
      return anything();
    },
  });
}

/** 2D-Context: misst Text grob (0,55 x Schriftgröße je Zeichen), zeichnet nichts. */
function makeCtx() {
  const state = { font: "10px sans-serif" };
  const noop = () => {};
  const grad = { addColorStop: noop };
  return new Proxy(state, {
    get(t, key) {
      if (key === "measureText") {
        return (s) => {
          const m = /(\d+(?:\.\d+)?)px/.exec(t.font || "");
          const px = m ? Number(m[1]) : 10;
          return { width: Array.from(String(s)).length * px * 0.55 };
        };
      }
      if (key === "createLinearGradient" || key === "createRadialGradient" || key === "createPattern" || key === "createConicGradient") return () => grad;
      if (key === "getImageData" || key === "createImageData") {
        return (a, b, c, d) => {
          const w = Math.max(1, Math.floor(c || a || 1)), h = Math.max(1, Math.floor(d || b || 1));
          return { width: w, height: h, data: new Uint8ClampedArray(w * h * 4) };
        };
      }
      if (key === "getTransform") return () => ({ a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 });
      if (key === "isPointInPath") return () => false;
      if (key === "canvas") return { width: 800, height: 600 };
      if (key in t) return t[key];
      return noop;
    },
    set(t, key, value) {
      t[key] = value;
      return true;
    },
  });
}

const lsData = {};
const localStorage = {
  getItem: (k) => (Object.prototype.hasOwnProperty.call(lsData, k) ? lsData[k] : null),
  setItem: (k, v) => {
    lsData[k] = String(v);
  },
  removeItem: (k) => {
    delete lsData[k];
  },
  key: (i) => Object.keys(lsData)[i] || null,
  get length() {
    return Object.keys(lsData).length;
  },
};

const listeners = {};
const ctx = {
  console, Math, Date, JSON, Map, Set, Array, Object, Number, String, Boolean, RegExp, Error, Symbol, Promise,
  Uint8Array, Uint8ClampedArray, Float32Array, Int32Array, Uint32Array, Proxy, Reflect, Intl, isFinite, isNaN,
  parseInt, parseFloat, performance, setTimeout, clearTimeout, queueMicrotask,
  localStorage,
  navigator: { language: "de", languages: ["de"] },
  location: { search: "", href: "file:///matches_check" },
  document: anything(),
  Image: function () {
    return anything();
  },
  Option: function () {
    return anything();
  },
  ResizeObserver: function () {
    return anything();
  },
  requestAnimationFrame: () => 0,
  addEventListener(name, fn) {
    (listeners[name] = listeners[name] || []).push(fn);
  },
  removeEventListener() {},
  devicePixelRatio: 1,
};
ctx.window = ctx;
ctx.globalThis = ctx;
vm.createContext(ctx);

function run(rel) {
  const file = path.join(JS, rel);
  vm.runInContext(fs.readFileSync(file, "utf8"), ctx, { filename: rel });
}

// Sprachen (alle 14) + Kern + die geprüften Spiele (Manifest-Dateien)
const LANGS = ["de", "en", "fr", "es", "pt", "pl", "tr", "da", "no", "sv", "fi", "cs", "sl", "hr"];
for (const code of LANGS) run("lang/" + code + ".js");
for (const f of ["util.js", "achdefs.js"]) run("core/" + f);
run("games/ugc.js");
for (const f of ["profiles.js", "stats.js", "ach.js", "charts.js", "ui.js", "audio.js", "game.js", "replay.js",
  "matches.js", "dailyarchive.js", "matchview.js"]) run("core/" + f);
run("manifest.js");
const PG = ctx.PG;
PG.app = { pixelScale: 1, cssScale: 1, canvas: anything(), resetTransform() {} };
PG.lang = "de";
PG.profiles.init();

const WANT = ["CrossyRoadGame", "SudokuGame", "WordleGame", "BlackjackGame", "PokerGame", "CasinoGame",
  "LabyrinthGame", "GeometryDashGame", "LamaCrossingGame", "LamaPlinkoGame", "SnakeGame", "Game2048", "ConnectFourGame"];
const loaded = new Set();
for (const m of ctx.PG_MANIFEST) {
  if (!WANT.includes(m.id)) continue;
  for (const f of m.files) {
    if (loaded.has(f)) continue;
    loaded.add(f);
    run(f.replace(/^js\//, ""));
  }
}
run("games/wordle_words/de.js");

const M = PG.matches;
const DA = PG.dailyArchive;
const SR = PG.seedrand;
const TODAY = SR.dayIndex();
const t = PG.t;
const byId = (id) => PG.gameById[id];

function quiet(g) {
  g.playSound = () => {};
  g.rumble = () => {};
  g.tone = () => {};
  g.events = [];
  g.achEvent = (id) => g.events.push(id);
  return g;
}

function fresh() {
  M._doc = null;
  M._pid = "";
  M._dirty = false;
  PG.store.remove("matches");
}

function floatsIn(v) {
  if (typeof v === "number") return !Number.isInteger(v);
  if (Array.isArray(v)) return v.some(floatsIn);
  if (v && typeof v === "object") return Object.values(v).some(floatsIn);
  return false;
}

function overlap(rects) {
  for (let i = 0; i < rects.length; i++) for (let j = i + 1; j < rects.length; j++) if (rects[i].colliderect(rects[j])) return true;
  return false;
}

const key = (k) => ({ kind: "keydown", key: k, char: k.length === 1 ? k : "", pos: null, button: 1, delta: 0, repeat: false });
const click = (pos) => ({ kind: "mousedown", key: null, char: "", pos, button: 1, delta: 0, repeat: false });
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);

// =================================================================== (1) Daten
function auditData() {
  console.log("\nDaten (matches.js)");
  fresh();
  M.record("snake", "single", 10, null, 30);
  M.record("snake", "single", 50, null, 12, M.F_RECORD);
  M.record("snake", "single", 20, null, 7);
  let e = M.entries("snake");
  check(same(e.map((x) => x[2]), [20, 50, 10]), "neueste zuerst", e.map((x) => x[2]));
  let tot = M.totals("snake");
  check(tot.n === 3 && tot.best === 50 && tot.sum === 80 && tot.time === 49 && tot.stored === 3, "Summen (n, best, sum, time, stored)", tot);
  check(e[0].length === 7 && e[0][0].length === 16, "Eintrag = 7 Felder, Zeit YYYY-MM-DD HH:MM", e[0]);
  M.record("chess", "single", 0, M.WIN, 100);
  M.record("chess", "single", 0, M.LOSS, 100);
  M.record("chess", "single", 0, M.DRAW, 100);
  M.record("chess", "single", 0, true, 100);
  tot = M.totals("chess");
  check(tot.w === 1 && tot.l === 1 && tot.d === 1 && tot.n === 4, "Sieg/Niederlage/Remis gezählt", tot);
  check(M.entries("chess")[0][3] === null, "bool als Ergebnis wird verworfen");
  for (let i = 0; i < M.CAP + 15; i++) M.record("tetris", "solo", i, null, 1);
  tot = M.totals("tetris");
  e = M.entries("tetris");
  check(e.length === M.CAP && tot.n === M.CAP + 15 && e[0][2] === M.CAP + 14 && e[e.length - 1][2] === 15,
    "Kappe " + M.CAP + " je Spiel, Summen laufen weiter", [e.length, tot.n]);
  check(M.recent("tetris", 10)[0][2] === M.CAP + 14 && M.recent("tetris").length === 10, "recent() = letzte 10");
  M.record("game2048", "single", 100, null, 20);
  M.record("game2048", "single", 300, null, 15, 0, null, { replace: true });
  e = M.entries("game2048");
  tot = M.totals("game2048");
  check(e.length === 1 && e[0][2] === 300 && e[0][4] === 35 && tot.n === 1 && tot.sum === 300, "replace: altes Match raus, Spielzeit addiert", [e, tot]);
  M.record("crossy", "daily", 5, null, 3, M.F_DAILY, { c: 2, ch: "fox", x: 1.5, zu_langer_schluessel: 1, ok: true });
  const info = M.entries("crossy")[0][6];
  check(same(info, { c: 2, ch: "fox", ok: true }), "Info: nur int/str/bool, kurze Schlüssel", info);
  M.record("crossy", "endless", 7, null, 3);
  check(same(M.modesOf("crossy"), ["daily", "endless"]), "modesOf in Reihenfolge der Sichtung");
  M.flush();
  const rawText = localStorage.getItem(PG.store.key("matches"));
  const raw = JSON.parse(rawText);
  check(rawText.indexOf("\n") < 0 && raw.format === M.FORMAT && raw.v === 1 && raw.games && raw.games.snake,
    "Speicher kompakt in einer Zeile, Kopf format/v, Spiele je Profil", rawText.slice(0, 80));
  check(!floatsIn(raw), "keine Gleitkommazahlen im Speicher");
  const before = {};
  for (const k of ["snake", "chess", "tetris", "crossy"]) before[k] = M.entries(k);
  M._doc = null;
  const after = {};
  for (const k of ["snake", "chess", "tetris", "crossy"]) after[k] = M.entries(k);
  check(same(before, after), "Rundlauf: Laden == Speichern");
  raw.games.snake.list.push(["x"]);
  raw.games.snake.list.push(["2026-01-01 10:00", 3, "viel", 7, -5, 999, [1]]);
  raw.games.snake.n = "drei";
  localStorage.setItem(PG.store.key("matches"), JSON.stringify(raw));
  M._doc = null;
  e = M.entries("snake");
  check(e.length === 4 && same(e[0], ["2026-01-01 10:00", "", 0, null, 0, 255, null]) && M.totals("snake").n === 0,
    "kaputte Einträge bereinigt", e.slice(0, 2));
  // Profile
  fresh();
  M.record("snake", "single", 9, null, 1);
  M.record("snake", "single", 4, null, 1, 0, null, { pid: "p_other" });
  check(M.totals("snake").n === 1 && M.totals("snake", "p_other").n === 1, "Profile getrennt");
  M.dropProfile("p_other");
  check(M.totals("snake", "p_other").n === 0, "dropProfile löscht die Historie");
  check(M.formatDuration(3725) === "1:02:05" && M.formatDuration(65) === "1:05", "formatDuration");
  // Profilwechsel: Offenes wird ins alte Profil geschrieben
  fresh();
  M.record("snake", "single", 77, null, 1);
  const old = PG.store.profile;
  PG.store.profile = "p_switch";
  PG.profiles.fireSwitch();
  check(M.entries("snake").length === 0, "Profilwechsel: neues Profil hat eigene (leere) Historie");
  PG.store.profile = old;
  PG.profiles.fireSwitch();
  check(M.entries("snake").length === 1 && M.entries("snake")[0][2] === 77, "Profilwechsel: Offenes blieb beim alten Profil");
}

// ============================================================ (2) app-Hook
function appMatchSave() {
  const src = fs.readFileSync(path.join(JS, "core", "app.js"), "utf8");
  const start = src.indexOf("    matchSave(g, record = false, leaving = false) {");
  const end = src.indexOf("\n    }\n", start);
  if (start < 0 || end < 0) throw new Error("matchSave nicht in app.js gefunden");
  const body = src.slice(start, end + 6).trim();
  return vm.runInContext("(function " + body + ")", ctx);
}

function auditHook() {
  console.log("\nApp.matchSave (app.js)");
  const hook = appMatchSave();
  const app = {};
  fresh();
  const Snake = byId("SnakeGame").cls;
  let g = quiet(new Snake(800, 600, "single"));
  g.score = 42;
  g.gameOver = true;
  g._matchTime = 33.9;
  hook.call(app, g, true, false);
  hook.call(app, g, true, true);
  let e = M.entries("snake");
  check(e.length === 1 && e[0][2] === 42 && e[0][4] === 33 && (e[0][5] & M.F_RECORD), "Game Over -> genau ein Eintrag mit Rekord-Flag, Dauer abgerundet", e);
  g = quiet(new Snake(800, 600, "single"));
  g.score = 0;
  g._matchTime = 2.0;
  hook.call(app, g, false, true);
  check(M.entries("snake").length === 1, "kurz verlassen ohne Punkte -> kein Eintrag");
  g._matchTime = M.MIN_ABORT_SECONDS + 0.5;
  hook.call(app, g, false, true);
  e = M.entries("snake");
  check(e.length === 2 && (e[0][5] & M.F_ABORT), "lange gespielt und verlassen -> abgebrochen", e[0]);
  check(!M._dirty, "Verlassen schreibt sofort");
  g = quiet(new Snake(800, 600, "single"));
  g.score = 99;
  hook.call(app, g, false, false);
  check(M.entries("snake").length === 2, "laufendes Spiel ohne Verlassen -> kein Eintrag");
  const Connect = byId("ConnectFourGame").cls;
  g = quiet(new Connect(800, 600, "single"));
  g.reportResult(true);
  g.gameOver = true;
  hook.call(app, g, false, false);
  check(M.entries("connect4")[0][3] === M.WIN, "Ergebnis aus reportResult");
  const BJ = byId("BlackjackGame").cls;
  g = quiet(new BJ(800, 600, "single"));
  g.score = 1500;
  g.gameOver = true;
  hook.call(app, g, false, true);
  check(M.entries("blackjack").length === 0, "matchManual-Spiele: app schreibt nichts");
  const G2048 = byId("Game2048").cls;
  g = quiet(new G2048(800, 600, "single"));
  g.score = 100;
  g.gameOver = true;
  hook.call(app, g, false, false);
  g.resumeFromGameOver();
  g.score = 180;
  g.gameOver = true;
  hook.call(app, g, false, false);
  e = M.entries(g.highscoreKey);
  check(e.length === 1 && e[0][2] === 180, "Rückgängig nach Game Over ersetzt das Match", e);
  let bad = null;
  for (const entry of PG.games) {
    const inst = Object.create(entry.cls.prototype);
    inst.state = "setup";
    const desc = Object.getOwnPropertyDescriptor(entry.cls.prototype, "matchActive");
    if (!desc && inst.matchActive) bad = entry.id;
  }
  check(bad === null, "Setup-Zustand zählt nicht als Match-Dauer", bad);
}

// ======================================================== (3) eigene Matches
function auditManual() {
  console.log("\nSpiele mit eigenen Matches (Game.logMatch)");
  const manual = new Set(PG.games.filter((e) => Object.create(e.cls.prototype).matchManual).map((e) => e.meta.key));
  const want = ["blackjack", "poker", "casino", "lamacrossing", "lamaplinko", "geodash", "maze"];
  check(want.every((k) => manual.has(k)), "matchManual: geladene Chip-Spiele + Geodash + Labyrinth", want.filter((k) => !manual.has(k)));
  fresh();
  const C = PG.cards;
  const BJ = byId("BlackjackGame").cls;
  let g = quiet(new BJ(800, 600, "single"));
  g.hands = [[new C.Card(10, "H"), new C.Card(9, "S")], [new C.Card(10, "D"), new C.Card(5, "C")]];
  g.handBets = [20, 20];
  g.dealer = [new C.Card(10, "C"), new C.Card(8, "H")];
  g._matchTime = 12.4;
  g._settle(null);
  let e = M.entries("blackjack");
  check(e.length === 1 && e[0][2] === 0 && e[0][3] === M.DRAW && same(e[0][6], { bet: 40, win: 40 }) && e[0][4] === 12,
    "Blackjack: Hand = Match (Einsatz/Auszahlung, Remis bei +-0)", e);
  const PK = byId("PokerGame").cls;
  g = quiet(new PK(800, 600, "video"));
  g.videoBet = 10;
  g._videoDeal();
  g.holds = new Set([0, 1, 2, 3, 4]);
  g._videoDraw();
  e = M.entries("poker");
  check(e.length === 1 && e[0][6].bet === 10 && e[0][1] === "video", "Video-Poker: Ziehung = Match", e);
  const CA = byId("CasinoGame").cls;
  g = quiet(new CA(800, 600, "roulette"));
  if (!Object.keys(g.rBets || {}).length) g.rBets = { red: 10 };
  try {
    g._rSpin();
  } catch (x) {
    check(false, "Roulette-Dreh", String(x));
  }
  e = M.entries("casino");
  check(e.length === 1 && e[0][1] === "roulette" && "num" in (e[0][6] || {}), "Roulette: Wurf = Match mit gezogener Zahl", e);
  g = quiet(new CA(800, 600, "slots"));
  const n0 = M.entries("casino").length;
  try {
    g._sSpin();
  } catch (x) {
    check(false, "Slots-Dreh", String(x));
  }
  e = M.entries("casino");
  check(e.length === n0 + 1 && e[0][1] === "slots", "Slots: Dreh = Match", e.slice(0, 1));
  const MZ = byId("LabyrinthGame").cls;
  g = quiet(new MZ(800, 600, "single"));
  g.startLevel(2);
  g.elapsed = 21.7;
  g.finish();
  e = M.entries("maze");
  check(e.length === 1 && e[0][3] === M.WIN && e[0][6].lv === 2 && e[0][6].tm === 21, "Labyrinth: gelöstes Level = Match", e);
  const GD = byId("GeometryDashGame").cls;
  g = quiet(new GD(800, 600, "normal"));
  g.startMusic = () => {};
  try {
    g.startLevel(g.levels[0], "main");
    check(g.matchActive === true, "Geometry Dash: im Level zählt die Match-Dauer");
    g._gdBestPct = 37;
    g.leaveToSelect();
    e = M.entries("geodash");
    check(e.length === 1 && e[0][2] === 37 && e[0][3] === M.LOSS && e[0][6].g >= 1, "Geometry Dash: Level verlassen = Match mit bestem Fortschritt", e);
    g.leaveToSelect();
    check(M.entries("geodash").length === 1 && g.matchActive === false, "Geometry Dash: nur ein Eintrag je Besuch");
  } catch (x) {
    check(false, "Geometry Dash Level-Besuch", String(x && x.stack));
  }
  const parts = PG.matchView.infoParts(byId("LamaCrossingGame"), { lv: "hard", x: 1525, bet: 1500 });
  check(parts[0].endsWith("1.500") && parts[1].includes("15.25") && parts.includes(t("lcr.diff.hard")),
    "Info-Formatierung (Tausenderpunkt, Multiplikator, matchText)", parts);
  const pl = PG.matchView.infoParts(byId("LamaPlinkoGame"), { lv: "high", rw: 12 });
  check(pl.includes(t("lpl.risk.high")), "Plinko: matchText für das Risiko", pl);
}

// ==================================================== (4) dailyarchive.js
function auditCore() {
  console.log("\nBaustein (dailyarchive.js)");
  let ok = true;
  for (let i = -400; i < 3000; i += 7) if (SR.dayIndex(SR.dateOf(i)) !== i) ok = false;
  check(ok && SR.dateOf(0) === "2026-01-01" && SR.dateOf(365) === "2027-01-01", "dateOf = Umkehrung von dayIndex");
  const days = DA.available([TODAY - 100, TODAY - 5, TODAY + 3, -1, true], TODAY);
  check(days[0] === TODAY - 100 && days[days.length - 1] === TODAY && days.length === DA.WINDOW + 1 && !days.includes(TODAY + 3),
    "verfügbar: 30 Tage + ältere gespielte, keine Zukunft", days.slice(0, 3));
  const rows = DA.cleanRows([[5, 1, 1], [3, 2, 2], [5, 9, 9], ["x", 1, 1], [7, 1], [-2, 1, 1], [true, 1, 1]], 3);
  check(same(rows, [[3, 2, 2], [5, 9, 9]]), "cleanRows: sortiert, letzte gewinnt, Müll raus", rows);
  DA.upsert(rows, [4, 0, 0]);
  DA.upsert(rows, [5, 1, 1]);
  DA.upsert(rows, [9, 3, 3]);
  check(same(rows.map((r) => r[0]), [3, 4, 5, 9]) && same(DA.find(rows, 5), [5, 1, 1]) && DA.find(rows, 6) === null, "upsert/find", rows);
  check(DA.shortDate(SR.dayIndex("2026-11-26")) === "26.11." && DA.chipDate(SR.dayIndex("2025-12-31")) === "31.12.25"
    && DA.longDate(SR.dayIndex("2026-11-26")).endsWith("26.11.2026") && DA.longDate(SR.dayIndex("2026-11-26")).startsWith(t("cal.weekdays_short").split(",")[3]),
  "Datums-Texte (Do, 26.11.2026)", DA.longDate(SR.dayIndex("2026-11-26")));
  check(DA.dayLabel(TODAY) === t("arch.today", { date: DA.shortDate(TODAY) }) && DA.dayLabel(TODAY - 1) === t("arch.yesterday", { date: DA.shortDate(TODAY - 1) }), "Heute/Gestern");
  check(DA.seedHex("crossy", TODAY) === SR.dailySeed("crossy").toString(16).toUpperCase().padStart(8, "0"), "Seed-Anzeige = dailySeed");
  check(DA.safeDayIndex("2026-13-01") === -1 && DA.safeDayIndex("kaputt") === -1 && DA.safeDayIndex("2026-01-02") === 1 && DA.safeDayIndex("") === TODAY,
    "ungültige Datumstexte -> -1 (wie ValueError), leer = heute");
  const p = new DA.Picker("crossy", [200, 200, 0], TODAY, { [TODAY - 60]: ["solved", "x"], [TODAY - 2]: ["played", ""] });
  p.layout(640, 480);
  p.handleEvent(key("Left"));
  check(p.sel === TODAY - 1, "Picker: Links = Vortag");
  p.handleEvent(key("Up"));
  check(p.sel === TODAY - 8, "Picker: Hoch = eine Woche zurück");
  p.handleEvent(key("Home"));
  check(p.sel === TODAY - 60, "Picker: Pos1 = ältester Tag (gespielter Alt-Tag)");
  p.handleEvent(key("End"));
  check(p.sel === TODAY, "Picker: Ende = heute");
  check(same(p.handleEvent(key("Return")), ["pick", TODAY]), "Picker: Enter wählt");
  check(same(p.handleEvent(key("Escape")), ["close", null]), "Picker: Esc schließt");
  const cell = p.cells.find(([, d]) => d === TODAY - 3)[0];
  check(same(p.handleEvent(click(cell.center)), ["pick", TODAY - 3]), "Picker: Klick auf einen Tag");
  const chip = p.chips[0];
  check(chip[1] === TODAY - 60 && same(p.handleEvent(click(chip[0].center)), ["pick", TODAY - 60]), "Picker: Klick auf älteren gespielten Tag");
  check(same(p.handleEvent(click([2, 2])), ["close", null]), "Picker: Klick daneben schließt");
  check(p.cells.every(([, d]) => d === null || (d >= TODAY - 29 && d <= TODAY)) && p.cells.filter(([, d]) => d !== null).length === DA.WINDOW,
    "Kalender = genau 30 Tage");
}

// ===================================================== (5) Spiele: Archiv
function auditCrossy() {
  console.log("\nCrossy Road (Tagesstrecke)");
  const CR = byId("CrossyRoadGame").cls;
  PG.store.set("crossy", { coins: 3, daily: { date: SR.dateOf(TODAY - 1), best: 8 } });
  let g = quiet(new CR(800, 600, "daily"));
  check(same(g.data.arch, [[TODAY - 1, 8, 1]]), "Altbestand (gestern) wandert ins Archiv", g.data.arch);
  check(g.day === TODAY && g.dailyBest() === 0 && !g.isArchive(), "Start: heute, Bestwert 0");
  g.stepDay(1);
  check(g.day === TODAY, "Bild ab über heute hinaus: bleibt heute");
  g.handleEvent(key("Prior"));
  check(g.day === TODAY - 1 && g.isArchive() && g.dailyBest() === 8, "Bild auf: gestern, Bestwert 8");
  g.newRun();
  const W2 = new PG.crossyWorld.World(SR.dailySeed("crossy", SR.dateOf(TODAY - 1)));
  check(g.world.seed === undefined || g.world.seed === W2.seed, "Archiv-Lauf nutzt den Seed des Datums");
  let sameRows = true;
  if (typeof g.world.row === "function") {
    for (let r = 1; r < 60; r++) if (!same(g.world.row(r), W2.row(r))) sameRows = false;
  }
  check(sameRows, "Strecke identisch zur Tagesstrecke dieses Datums");
  g.best = 15;
  const flags = g.matchFlags();
  g.finishRun();
  let d = PG.store.get("crossy");
  check(same(DA.find(d.arch, TODAY - 1), [TODAY - 1, 15, 2]) && g.record && same(d.daily, { date: SR.dateOf(TODAY - 1), best: 8 }),
    "Archiv-Tag: Bestwert/Versuche im Archiv, 'daily' unberührt", [d.arch, d.daily]);
  check(flags === (M.F_DAILY | M.F_ARCHIVE) && g.matchInfo().day === TODAY - 1, "Match: Flags Tagesmodus+Archiv, Info mit Tag");
  g = quiet(new CR(800, 600, "daily"));
  g.newRun();
  g.best = 21;
  g.finishRun();
  d = PG.store.get("crossy");
  check(same(d.daily, { date: SR.todayStr(), best: 21 }) && same(DA.find(d.arch, TODAY), [TODAY, 21, 1]), "heute: 'daily' + Archiv", d);
  check(g.matchFlags() === M.F_DAILY, "heute: nur Flag Tagesmodus");
  check(same(Object.keys(g.matchInfo()), ["day", "c", "ch"]), "Info-Reihenfolge day, c, ch wie Python", g.matchInfo());
  g.toSetup("play");
  g.openPicker();
  check(g.wantsEscape && g.picker.info.get(TODAY - 1)[0] === "played", "Archiv-Fenster: Esc gehört dem Spiel");
  g.handleEvent(key("Escape"));
  check(g.picker === null && g.state === "setup", "Esc schließt nur das Fenster");
  g.openPicker();
  g.picker.sel = TODAY - 1;
  g.handleEvent(key("Return"));
  check(g.picker === null && g.day === TODAY - 1, "Enter im Fenster wählt den Tag");
  const en = quiet(new CR(800, 600, "endless"));
  check(en.matchFlags() === 0 && !("day" in en.matchInfo()), "Endlos: keine Tages-Flags");
  PG.store.set("crossy", { coins: 0, daily: { date: "2026-01-02", best: 99 }, arch: [] });
  check(same(quiet(new CR(800, 600, "daily")).data.arch, [[1, 99, 1]]), "alter Tagesbestwert (02.01.) -> Archiv [[1, 99, 1]]");
}

function auditSudoku() {
  console.log("\nSudoku (Tagesrätsel)");
  const SD = byId("SudokuGame").cls;
  const gen = PG.sudokuGen;
  const yday = SR.dateOf(TODAY - 1);
  PG.store.set("sudoku", { daily: { date: yday, streak: 4, best: 6, time: 300, stars: 2, count: 9 } });
  const g = quiet(new SD(800, 600, "classic"));
  check(same(g.arch, [[TODAY - 1, 300, 2]]), "Altbestand wandert ins Archiv", g.arch);
  check(g.dailyStreak() === 4, "Serie (gestern gelöst) läuft");
  g.stepDay(-3);
  check(g.archDay === TODAY - 3 && g.isArchive(), "Bild auf x3: Archiv-Tag");
  g.startDaily();
  g.doGenerate();
  const [, solution, diff] = gen.generateDaily(SR.dateOf(TODAY - 3));
  check(g.dailyDate === SR.dateOf(TODAY - 3) && g.playDiff === diff, "Archiv-Rätsel = Tagesrätsel dieses Datums (Stufe aus dem Wochentag)");
  check(same(g.solution, solution), "Lösung identisch");
  g.elapsed = 412.0;
  g.errors = 0;
  g.hintsUsed = 0;
  g.win();
  let d = PG.store.get("sudoku");
  check(d.daily.streak === 4 && d.daily.date === yday && d.daily.count === 9, "Archiv-Tag ändert Serie und Zähler nicht", d.daily);
  const row = DA.find(d.arch, TODAY - 3);
  check(row !== null && row[1] === 412 && row[2] >= 1 && row[2] <= 3, "Archiv-Zeile: Zeit + Sterne", row);
  check(g.matchFlags() === (M.F_DAILY | M.F_ARCHIVE) && g.matchInfo().tm === 412 && g.matchInfo().day === TODAY - 3, "Match: Archiv-Flags, Zeit, Tag");
  g.startDaily();
  g.doGenerate();
  g.elapsed = 500.0;
  g.win();
  check(DA.find(PG.store.get("sudoku").arch, TODAY - 3)[1] === 412 && !g.newRecord, "langsamer gelöst: Bestzeit bleibt");
  g.backToSetup();
  g.setArchDay(TODAY);
  g.startDaily();
  g.doGenerate();
  g.elapsed = 200.0;
  g.win();
  d = PG.store.get("sudoku");
  check(d.daily.date === SR.todayStr() && d.daily.streak === 5 && DA.find(d.arch, TODAY)[1] === 200, "heute: Serie +1 und Archiv-Zeile", d.daily);
  g.backToSetup();
  g.openPicker();
  check(g.wantsEscape && g.picker.info.get(TODAY)[0] === "solved" && g.picker.info.get(TODAY - 3)[0] === "solved", "Archiv-Fenster zeigt gelöste Tage");
  g.handleEvent(key("Escape"));
  check(g.picker === null && g.state === "setup", "Esc schließt nur das Fenster");
  const c = quiet(new SD(800, 600, "classic"));
  c.startLevel(7);
  c.doGenerate();
  check(same(c.matchInfo(), { vr: "classic", lv: 7 }) && c.matchFlags() === 0, "Level-Rätsel: Info Variante + Level", c.matchInfo());
}

function auditWordle() {
  console.log("\nWordle (Tageswort)");
  const WD = byId("WordleGame").cls;
  const W = PG.wordle;
  PG.store.set("mem.wordle", {});
  let g = quiet(new WD(800, 600, "daily"));
  g.start();
  const todayAnswer = g.boards[0].answer;
  check(todayAnswer === W.dailyAnswer(g.words, g.lang, g.length, SR.todayStr()), "heute: Tageswort");
  const wrong = g.words.find((w) => w !== todayAnswer);
  g.current = wrong;
  g.submit();
  g.finishReveal();
  let data = W.loadData();
  let rows = W.archRows(data, g.dailySlot());
  check(same(DA.find(rows, TODAY), [TODAY, todayAnswer, wrong, 2]), "heute: Stand ins Archiv gespiegelt", rows);
  const stBefore = JSON.stringify(data.stats);
  const dailyBefore = JSON.stringify(data.daily);
  g.toSetup();
  g.archDay = TODAY - 5;
  check(g.isArchive(), "Archiv-Tag gewählt");
  g.start();
  const ans = g.boards[0].answer;
  check(ans === W.dailyAnswer(g.words, g.lang, g.length, SR.dateOf(TODAY - 5)), "Archiv-Wort = Tageswort dieses Datums");
  const other = g.words.find((w) => w !== ans);
  g.current = other;
  g.submit();
  g.finishReveal();
  const g2 = quiet(new WD(800, 600, "daily"));
  g2.archDay = TODAY - 5;
  g2.start();
  check(same(g2.guesses, [other]), "Archiv-Fortschritt wird fortgesetzt", g2.guesses);
  g2.current = ans;
  g2.submit();
  g2.finishReveal();
  data = W.loadData();
  check(JSON.stringify(data.stats) === stBefore, "Archiv-Sieg ändert die Statistik nicht");
  check(JSON.stringify(data.daily) === dailyBefore, "heutiger Stand bleibt unberührt");
  const row = DA.find(W.archRows(data, g2.dailySlot()), TODAY - 5);
  check(same(row, [TODAY - 5, ans, other + ans, 1]), "Archiv-Zeile: Antwort, Versuche, gelöst", row);
  check(!g2.events.includes("wordle_daily7"), "kein Serien-Erfolg für Archiv-Tage");
  check(g2.matchFlags() === (M.F_DAILY | M.F_ARCHIVE) && g2.matchInfo().g === 2 && g2.matchScore() > 0 && g2.matchInfo().day === TODAY - 5,
    "Match: Archiv-Flags, 2 Versuche, Punkte", [g2.matchInfo(), g2.matchScore()]);
  check(same(Object.keys(g2.matchInfo()), ["day", "len", "g"]), "Info-Reihenfolge day, len, g wie Python", g2.matchInfo());
  g2.donePrimary();
  check(g2.state === "setup", "Archiv-Tag fertig: Enter führt ins Setup");
  data.arch[g2.dailySlot()] = [[TODAY - 7, "ZZZZZ", "", 2]];
  W.saveData(data);
  const g3 = quiet(new WD(800, 600, "daily"));
  g3.archDay = TODAY - 7;
  g3.start();
  g3.current = "ZZZZZ";
  g3.submit();
  check(g3.reveal !== null, "gespeicherte Antwort gilt, auch wenn sie nicht in der Liste steht");
  g3.toSetup();
  g3.archDay = TODAY;
  g3.openPicker();
  const st = (g3.picker.info.get(TODAY) || [""])[0];
  check(["started", "played", "solved"].includes(st) && g3.wantsEscape, "Archiv-Fenster: heute");
  // Altbestand: heutiger/letzter Stand wandert ins Archiv
  PG.store.set("mem.wordle", { daily: { "de:5": { date: SR.dateOf(TODAY - 2), answer: "LAMAS", guesses: ["HALLO", "LAMAS"], done: true } } });
  data = W.loadData();
  check(same(data.arch["de:5"], [[TODAY - 2, "LAMAS", "HALLOLAMAS", 1]]), "Altbestand (daily) wandert ins Archiv", data.arch);
  const en = quiet(new WD(800, 600, "endless"));
  check(en.matchFlags() === 0 && same(en.matchInfo(), { len: en.length, wd: 0 }), "Endlos: Info Wortlänge + Wörter", en.matchInfo());
}

// ====================================================== (6) Anzeige
function auditScreens() {
  console.log("\nAnzeige (MatchScreen, Datums-Leisten, Archiv-Fenster)");
  fresh();
  const keys = M.INFO_KEYS.map(([k]) => k);
  const sample = { day: 330, bet: 20, win: 45, num: 17, cr: 245, mi: 3, op: 5, rw: 12, x: 225, c: 4, ch: "fox", vr: "x", lv: 7, ln: 9,
    st: 3, tm: 125, g: 4, wd: 6, len: 5, vs: "Lama" };
  check(same(keys.slice().sort(), Object.keys(sample).sort()), "Beispiel deckt alle Info-Schlüssel ab");
  for (let n = 0; n < 40; n++) {
    for (const k of ["crossy", "blackjack", "sudoku", "wordle", "lamacrossing"]) {
      M.record(k, ["crossy", "wordle"].includes(k) ? (n % 2 ? "daily" : "endless") : "single",
        ((n * 37) % 500) - (k === "blackjack" ? 100 : 0), [null, 1, 0, 2][n % 4], n * 13, n % 16, Object.assign({}, sample));
    }
  }
  const badTxt = [];
  for (const code of LANGS) {
    const table = (ctx.PG_LANG || {})[code] || {};
    for (const k of keys) if (String(table["match.info." + k] || "").indexOf("{v}") < 0) badTxt.push([code, k]);
  }
  const errors = [], layout = [];
  const fake = { canvas: anything(), backToMenu() {} };
  const dc = makeCtx();
  const W = PG.W, H = PG.H;
  for (const code of LANGS) {
    PG.lang = code;
    for (const id of WANT) {
      const entry = byId(id);
      for (const tab of ["recent", "all"]) {
        let scr;
        try {
          scr = new PG.MatchScreen(fake, entry, null, tab);
          scr.draw(dc);
        } catch (x) {
          errors.push([code, id, tab, String(x && x.stack).slice(0, 300)]);
          continue;
        }
        const rects = scr.tabRects.map((r) => r[0]).concat([scr.sumRect]);
        if (scr.chartRect) rects.push(scr.chartRect);
        for (const [r] of scr.filterRects) rects.push(r);
        if (scr.sortRect) rects.push(scr.sortRect);
        const rows = scr.rowRects.slice(0, Math.max(1, Math.min(scr.rows().length, scr.rowsVisible)));
        const all = rects.concat(rows);
        if (all.some((r) => r.left < 0 || r.right > W || r.top < 0 || r.bottom > H - 30) || overlap(all)) layout.push([code, id, tab]);
        if (code === "de") {
          for (const k of ["m", "o", "Down", "Next", "End", "Home", "Tab"]) scr.handleEvent(key(k));
          scr.handleEvent({ kind: "mousemove", pos: [400, 250] });
          scr.draw(dc);
        }
      }
    }
    // Datums-Leisten + Archiv-Fenster
    try {
      const c = quiet(new (byId("CrossyRoadGame").cls)(W, H, "daily"));
      c.day = TODAY - 2;
      c.draw(dc);
      const rr = c.dayRects;
      if (rr.some((r) => r.left < 0 || r.right > W || r.bottom > H - 34) || overlap(rr.concat(c.nightRects, c.shadowRects, [c.startRect]))) layout.push(["crossy", code]);
      c.openPicker();
      c.draw(dc);
      const s = quiet(new (byId("SudokuGame").cls)(W, H, "classic"));
      s.setArchDay(TODAY - 2);
      s.draw(dc);
      const sr = s.dayrowRects;
      if (sr.some((r) => !s.panelRect.contains(r)) || overlap(sr.concat([s.dailyRect, s.limitRect]))) layout.push(["sudoku", code]);
      s.openPicker();
      s.draw(dc);
      const q = quiet(new (byId("WordleGame").cls)(W, H, "daily"));
      q.archDay = TODAY - 2;
      q.draw(dc);
      const qr = q.suDay;
      if (qr.some((r) => r.left < 0 || r.bottom > H - 20) || overlap(qr.concat([q.suCb, q.suStart]))) layout.push(["wordle", code]);
      q.openPicker();
      q.draw(dc);
      const p = q.picker;
      if (p.rect.left < 0 || p.rect.right > W || p.rect.top < 0 || p.rect.bottom > H) layout.push(["picker", code]);
      const cells = p.cells.filter(([, d]) => d !== null).map(([r]) => r).concat(p.chips.map(([r]) => r));
      if (cells.some((r) => !p.rect.contains(r))) layout.push(["picker-zellen", code]);
    } catch (x) {
      errors.push([code, "archiv", String(x && x.stack).slice(0, 300)]);
    }
  }
  PG.lang = "de";
  check(!badTxt.length, "Info-Texte vorhanden", badTxt.slice(0, 5));
  check(!errors.length, "zeichnet fehlerfrei: " + WANT.length + " Spiele x 2 Reiter x 14 Sprachen + Datums-Leisten/Archiv-Fenster", errors.slice(0, 3));
  check(!layout.length, "Reiter/Summen/Kurve/Zeilen bzw. Leisten im Bild, ohne Überlappung", layout.slice(0, 5));
  const scr = new PG.MatchScreen(fake, byId("CrossyRoadGame"), null, "all");
  const nAll = scr.items.length;
  scr.handleEvent(key("m"));
  check(scr.filter !== null && scr.items.every((e) => e[1] === scr.filter) && scr.items.length < nAll, "Filter M: nur ein Modus");
  scr.handleEvent(key("o"));
  const vals = scr.items.map((e) => e[2]);
  check(scr.sortBest && same(vals, vals.slice().sort((a, b) => b - a)), "Sortierung O: beste zuerst");
  scr.handleEvent(key("End"));
  check(scr.sel === scr.items.length - 1 && scr.first + scr.rowsVisible > scr.sel, "Ende: letzte Zeile sichtbar");
  scr.handleEvent(key("Tab"));
  check(scr.tab === "recent", "Tab wechselt den Reiter");
  let closed = false;
  const s2 = new PG.MatchScreen(fake, byId("CrossyRoadGame"), () => (closed = true));
  s2.handleEvent(key("Escape"));
  check(closed, "Esc schließt (zurück zum Vorspiel-Screen)");
  check(PG.matchView.fmtNum(-1234567) === "-1.234.567" && PG.matchView.pyRound(2.5) === 2 && PG.matchView.pyRound(3.5) === 4,
    "Zahlen: Tausenderpunkte, Python-Runden");
  const src = fs.readFileSync(path.join(JS, "core", "matches.js"), "utf8");
  check(src.split("\n").filter((l) => l.trim() === "const CAP = 250;").length === 1, "matches.js: 'const CAP = 250;' als eigene Zeile (pygamez.com-Patch)");
}

const T0 = Date.now();
const parts = [auditData, auditHook, auditManual, auditCore, auditCrossy, auditSudoku, auditWordle, auditScreens];
for (const part of parts) {
  try {
    part();
  } catch (x) {
    check(false, part.name + " lief nicht durch", String(x && x.stack));
  }
}
console.log();
if (FAILS.length) {
  console.log(FAILS.length + " FEHLER: " + JSON.stringify(FAILS) + "  (" + ((Date.now() - T0) / 1000).toFixed(1) + " s)");
  process.exit(1);
}
console.log("ALLE PRÜFUNGEN BESTANDEN  (" + ((Date.now() - T0) / 1000).toFixed(1) + " s)");
