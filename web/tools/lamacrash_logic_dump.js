#!/usr/bin/env node
/*
 * lamacrash_logic_dump.js - gibt die Lama-Crash-Regeln der Web-Version als JSON aus.
 *
 *   node web/tools/lamacrash_logic_dump.js            # Konstanten, ganze Kurve, feste Werte
 *   node web/tools/lamacrash_logic_dump.js - < f.json # dazu Faelle aus der Eingabe
 *
 * Wird von tests/audit_chiplogic.py aufgerufen, um Python
 * (games/lamacrash_logic.py) und JavaScript (web/js/games/lamacrash_logic.js)
 * zu vergleichen. Faelle (alle optional):
 *   tick_at [m100], u32 [r], seeds [[s, c, n]], win_count [t100], chance [t100],
 *   pop_tick [crash100], sims [{pairs, crash, acts, carry0}] (ganze Runden mit
 *   Hand-Ausstiegen wie devtools/export_chiplogic_ref.py), cash [[rnd, i, half, carry]],
 *   rounds [Rohdaten], x_clean [Rohdaten], x_record [[[rnd, crash, end], ...]],
 *   history [Rohdaten], carry_clean [Rohdaten], target_ok [Rohdaten], fmt [m100],
 *   chart [tick], ylines [y_max], xlines [x_max], blend [Takt], world_of [m100],
 *   step [[t100, Richtung]], bucket [crash100], close [[rnd, crash]], check [[o, crash]],
 *   expected [t100], payout [[[stake, m100], ...]].
 */
"use strict";
const fs = require("fs");
const path = require("path");
global.window = global;
global.PG = {};
for (const f of ["seedrand.js", "fairness.js", "lamacrash_logic.js", "lamacrash_music.js"]) require(path.join(__dirname, "..", "js", "games", f));
const L = global.PG.lamaCrashLogic;
const MU = global.PG.lamaCrashMusic; // compose/squeak/stageOf brauchen kein Audio

let cases = {};
const arg = process.argv[2];
if (arg === "-") cases = JSON.parse(fs.readFileSync(0, "utf8") || "{}");
else if (arg) cases = JSON.parse(fs.readFileSync(arg, "utf8"));

const clone = (v) => JSON.parse(JSON.stringify(v));

const curve = [];
const mu = [];
for (let tk = 0; tk <= L.MOON_TICK; tk++) {
  curve.push(L.multAt(tk));
  mu.push(L.muAt(tk));
}

const out = {
  consts: {
    key: L.KEY, house: L.HOUSE, max: L.MAX_MULT100, min_cash: L.MIN_CASH, bets: L.BETS, max_bets: L.MAX_BETS,
    history: L.HISTORY, tps: L.TPS, mu0: L.MU0, growth: L.GROWTH, mu_unit: L.MU_UNIT, moon_tick: L.MOON_TICK,
    speed_turbo: L.SPEED_TURBO, speed_ff: L.SPEED_FF, targets: L.TARGETS, default_target: L.DEFAULT_TARGET,
    worlds: L.WORLDS, world_at: L.WORLD_AT, blend_ticks: L.BLEND_TICKS, brave: L.BRAVE_MULT, space: L.SPACE_MULT,
    close_min: L.CLOSE_MIN, close_pct: L.CLOSE_PCT, buckets: L.BUCKETS, chart_min_ticks: L.CHART_MIN_TICKS,
    y_steps: L.Y_STEPS, x_steps: L.X_STEPS, two32: L.TWO32,
  },
  curve,
  mu,
  clamp: [-5, -1, 0, 0.9, 1.5, 11519.99, 11520, 11521, 1e9].map((v) => [L.multAt(v), L.muAt(v)]),
  world_ticks: L.worldTicks(),
  bucket_prob: L.BUCKETS.map((_, i) => L.bucketProbFrac(i)),
  bucket_prob_f: L.BUCKETS.map((_, i) => L.bucketProb(i)),
  blank_x: L.blankX(),
  half: L.BETS.concat([1, 3, 7]).map((b) => [L.halfStake(b), L.restStake(b)]),
  new_round: L.newRound({ n: 5, s: "ab".repeat(16), c: "kaugummi", h: "0".repeat(64) }, [[10, 0], [20, 150]]),
};

if (cases.tick_at) out.tick_at = cases.tick_at.map((m) => L.tickAt(m));
if (cases.u32) out.u32 = cases.u32.map((r) => L.crashFromU32(r));
if (cases.seeds) out.seeds = cases.seeds.map(([s, c, n]) => L.crashPoint(s, c, n));
if (cases.win_count) out.win_count = cases.win_count.map((v) => L.winCount(v));
if (cases.chance) out.chance = cases.chance.map((v) => [L.fmtChance(v), L.fmtChance(v, ",")]);
if (cases.pop_tick) out.pop_tick = cases.pop_tick.map((c) => L.popTick(c));
if (cases.expected) out.expected = cases.expected.map((v) => [L.expectedReturnFrac(v), L.expectedReturn(v)]);
if (cases.payout) {
  // Auszahlungsfolgen [[stake, m100], ...] mit Uebertrag ab 0
  out.payout = cases.payout.map((seq) => {
    let carry = 0;
    return seq.map(([b, m]) => {
      const [win, c] = L.payoutCarry(b, m, carry);
      carry = c;
      return [win, c];
    });
  });
}
if (cases.sims) {
  // Ganze Runden: genau die Schleife aus devtools/export_chiplogic_ref.py (_crash_sims)
  out.sims = cases.sims.map((sim) => {
    const rnd = L.newRound({ n: 1, s: "0".repeat(32), c: "abc", h: "0".repeat(64) }, sim.pairs);
    let carry = sim.carry0;
    const log = [];
    const todo = sim.acts.map((a) => a.slice());
    let end = null;
    while (end === null) {
      while (todo.length && todo[0][0] === rnd.t) {
        const [, i, h] = todo.shift();
        const res = L.cashOut(rnd, i, h, carry);
        log.push(["cash", rnd.t, i, h].concat(res !== null ? res : [null, null, null]));
        if (res !== null) carry = res[2];
      }
      const [evs, c, e] = L.tick(rnd, sim.crash, carry);
      carry = c;
      end = e;
      for (const ev of evs) log.push(["ev", rnd.t, ev[0], ev[1], ev[2], ev[3]]);
      if (rnd.t > L.MOON_TICK + 2) break;
    }
    return {
      log, end, t: rnd.t, bets: rnd.bets, carry, outcome: L.outcome(rnd, sim.crash), best: L.bestCash(rnd),
      close: L.closeCall(rnd, sim.crash), flying: L.flying(rnd), stake: L.stakeTotal(rnd), paid: L.paidTotal(rnd),
      check: L.checkOutcome(L.outcome(rnd, sim.crash), sim.crash),
    };
  });
}
if (cases.cash) {
  out.cash = cases.cash.map(([rnd, i, half, carry]) => {
    const r = clone(rnd);
    const res = L.cashOut(r, i, half, carry);
    return [res, r, L.riding(r.bets[0]), L.flying(r)];
  });
}
if (cases.rounds) out.rounds = cases.rounds.map((v) => L.cleanRound(clone(v)));
if (cases.x_clean) out.x_clean = cases.x_clean.map((v) => L.cleanX(v));
if (cases.x_record) {
  out.x_record = cases.x_record.map((seq) => {
    const x = L.blankX();
    for (const [rnd, crash, end] of seq) L.recordX(x, rnd, crash, end);
    return x;
  });
}
if (cases.history) out.history = cases.history.map((v) => L.cleanHistory(v));
if (cases.carry_clean) out.carry_clean = cases.carry_clean.map((v) => L.cleanCarry(v));
if (cases.target_ok) out.target_ok = cases.target_ok.map((v) => L.targetOk(v));
if (cases.fmt) out.fmt = cases.fmt.map((v) => [L.fmtMult(v), L.fmtMult(v, ","), L.fmtShort(v), L.fmtShort(v, ",")]);
if (cases.chart) out.chart = cases.chart.map((tk) => L.chartSpan(tk));
if (cases.ylines) out.ylines = cases.ylines.map((v) => L.chartYlines(v));
if (cases.xlines) out.xlines = cases.xlines.map((v) => L.chartXlines(v));
if (cases.blend) out.blend = cases.blend.map((v) => L.worldBlend(v));
if (cases.world_of) out.world_of = cases.world_of.map((v) => L.worldOf(v));
if (cases.step) out.step = cases.step.map(([v, d]) => L.stepTarget(v, d));
if (cases.bucket) out.bucket = cases.bucket.map((v) => L.bucketOf(v));
if (cases.close) out.close = cases.close.map(([rnd, c]) => L.closeCall(rnd, c));
if (cases.check) out.check = cases.check.map(([o, c]) => L.checkOutcome(o, c));
// Musik (lamacrash_music.js): Noten aller Stufen, Tempi, Stufe/Quietschen je Multiplikator
const musM = cases.music_m || [0, 199, 200, 1000, 10000, 100000, 10000000];
out.music = {
  stages: MU.STAGES,
  bpm: [0, 1, 2, 3, 4, 5].map((s) => MU.bpm(s)),
  beat: [0, 1, 2, 3, 4, 5].map((s) => MU.beatSeconds(s)),
  compose: [0, 1, 2, 3, 4, 5].map((s) => MU.compose(s)),
  stage_of: musM.map((m) => MU.stageOf(m)),
  squeak: musM.map((m) => MU.squeak(m)),
};
process.stdout.write(JSON.stringify(out));
