#!/usr/bin/env node
/*
 * fairness_dump.js - gibt Fairness (SHA-256, Seeds, Log) und Chip-Statistik der
 * Web-Version als JSON aus.
 *
 *   node web/tools/fairness_dump.js            # nur feste Pruefwerte
 *   node web/tools/fairness_dump.js - < f.json # dazu Faelle aus der Eingabe
 *   node web/tools/fairness_dump.js f.json
 *
 * Wird von tests/audit_chiplogic.py aufgerufen, um Python (fairness.py,
 * chipstats.py) und JavaScript (web/js/games/fairness.js, chipstats.js) zu
 * vergleichen. Faelle (alle optional): sha [Texte], begin [Zustaende],
 * clean_state [Zustaende], logs [Logs], verify [Eintraege], rand
 * [[spiel, s, c, n, anzahl]], entries [[eintrag, ergebnis]], stats_clean
 * [Rohdaten], stats_record [[[einsatz, auszahlung, m100], ...]].
 */
"use strict";
const fs = require("fs");
const path = require("path");
global.window = global;
global.PG = {};
for (const f of ["seedrand.js", "fairness.js", "chipstats.js"]) require(path.join(__dirname, "..", "js", "games", f));
const F = global.PG.fairness;
const C = global.PG.chipstats;

let cases = {};
const arg = process.argv[2];
if (arg === "-") cases = JSON.parse(fs.readFileSync(0, "utf8") || "{}");
else if (arg) cases = JSON.parse(fs.readFileSync(arg, "utf8"));

// feste Seeds aus einem Zaehler (zum Pruefen von Form und Reihenfolge)
let counter = 0x12345678;
const next = () => (counter = (Math.imul(counter, 1664525) + 1013904223) >>> 0);
const st = F.blankState(next);
const e0 = F.begin(st);
F.advance(st, next);
F.newClient(st, next);

const out = {
  alphabet: F.CLIENT_ALPHABET, client_len: F.CLIENT_LEN, client_max: F.CLIENT_MAX, log: F.LOG,
  max_nonce: F.MAX_NONCE, curve: C.CURVE,
  vectors: ["", "abc", "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "a".repeat(1000),
    "x".repeat(55), "x".repeat(56), "x".repeat(63), "x".repeat(64), "Lama äöü € 🦙"]
    .map((t) => F.sha256Hex(t)),
  shape: {
    server_ok: F.validServer(st.server), client_ok: F.validClient(st.client), nonce: st.nonce,
    entry_ok: F.verify(e0), keys: Object.keys(e0), state_keys: Object.keys(st),
    clean_bad: Object.keys(F.cleanState({ server: "XYZ", client: "l0", nonce: -1 }, next)),
    clean_bad_nonce: F.cleanState({ server: "XYZ", client: "l0", nonce: -1 }, next).nonce,
    record_len: (() => { const log = []; for (let i = 0; i < 20; i++) F.record(log, e0, i); return [log.length, log[0].o]; })(),
    short: [F.short(e0.h), F.short("abc")],
    random_ok: F.validServer(F.newServerSeed()) && F.validClient(F.newClientSeed()),
  },
};
if (cases.sha) out.sha = cases.sha.map((t) => F.sha256Hex(t));
if (cases.begin) out.begin = cases.begin.map((s) => F.begin(s));
if (cases.clean_state) out.clean_state = cases.clean_state.map((s) => F.cleanState(s, next));
if (cases.logs) out.logs = cases.logs.map((l) => F.cleanLog(l));
if (cases.verify) out.verify = cases.verify.map((e) => F.verify(e));
if (cases.rand) {
  out.rand = cases.rand.map(([g, s, c, n, k]) => {
    const r = F.roundRand(g, s, c, n);
    const vals = [];
    for (let i = 0; i < k; i++) vals.push(r.nextU32());
    return vals;
  });
}
if (cases.entries) out.entries = cases.entries.map(([e, o]) => F.makeEntry(e, o));
if (cases.stats_clean) out.stats_clean = cases.stats_clean.map((r) => C.clean(r));
if (cases.stats_record) {
  out.stats_record = cases.stats_record.map((ops) => {
    const s = C.blank();
    for (const [w, p, m] of ops) C.record(s, w, p, m);
    return { st: s, rtp: C.rtp(s), win_rate: C.winRate(s) };
  });
}
process.stdout.write(JSON.stringify(out));
