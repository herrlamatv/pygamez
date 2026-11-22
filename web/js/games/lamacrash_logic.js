/*
 * lamacrash_logic.js - Regeln von Lama Crash (1:1 aus games/lamacrash_logic.py)
 * ==============================================================================
 * Alle Multiplikatoren sind GANZZAHLEN in Hundertsteln (m100: 235 = x2,35); bis
 * auf worldBlend (Zeichenhilfe) kommt keine Gleitkommazahl vor. Jeder
 * Zwischenwert bleibt unter 2^53 - JS-Zahlen rechnen hier also exakt (nie
 * << / | 0 / >> 0 auf grosse Werte: 97 * 2^32 steht als 97 * 4294967296 da).
 *
 * Platzpunkt aus dem Fairness-Zufall der Runde (PG.fairness.roundRand, mulberry32):
 *
 *   r        = rand.nextU32()                        // 0 .. 2^32-1
 *   crash100 = min(MAX, max(100, 97 * 2^32 // (2^32 - r)))
 *
 * Steigkurve (ein Takt = 1/60 s):
 *
 *   mu_0 = 1_000_000;  mu_(t+1) = mu_t + mu_t // 1000;  m100(t) = mu_t // 10_000
 *
 * Takt-Regel (tick): t += 1; m = multAt(t)
 *   1. jeder noch fliegende Teil mit Auto-Ziel a: a <= m und a <= crash100 -> genau a
 *   2. crash100 >= MAX und m >= MAX -> Mondlandung (alle fliegenden Teile bekommen MAX)
 *   3. sonst m > crash100 -> Platzen
 *
 * Gespeicherte Runde (Schluesselreihenfolge wie Python):
 *   {n, s, c, h, t, bets: [{b, a, hm, xm, p, man}, ...]}
 * tests/audit_chiplogic.py vergleicht beide Fassungen
 * (ueber web/tools/lamacrash_logic_dump.js). Braucht seedrand.js + fairness.js.
 */
(function () {
  "use strict";

  const KEY = "lamacrash";

  // Auszahlungsquote in Prozent
  const HOUSE = 97;
  // Hoechster Multiplikator x100 (x100.000 = Mondlandung)
  const MAX_MULT100 = 10000000;
  // Kleinster Ausstieg (x1,01) - auch kleinstes Auto-Ziel
  const MIN_CASH = 101;
  // Einsatzstufen je Einsatz (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
  const BETS = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
  // hoechstens so viele Einsaetze in einer Runde
  const MAX_BETS = 2;
  // Laenge des Verlaufs (Platzpunkte)
  const HISTORY = 12;

  // Steigkurve
  const TPS = 60; // Takte je Sekunde (Normal-Tempo)
  const MU0 = 1000000; // mu bei x1,00
  const GROWTH = 1000; // je Takt waechst mu um mu // GROWTH (~0,1 %)
  const MU_UNIT = 10000; // m100 = mu // MU_UNIT
  const MOON_TICK = 11520; // erster Takt mit m100 >= MAX_MULT100
  const SPEED_TURBO = 3; // Takte je 1/60 s im Turbo
  const SPEED_FF = 16; // Takte je 1/60 s beim Vorspulen (alle ausgestiegen)

  // Auto-Ziele (Leiter fuer Pfeil links/rechts und das Mausrad)
  const TARGETS = [101, 102, 105, 110, 115, 120, 125, 130, 140, 150, 160, 175, 200, 225, 250,
    300, 350, 400, 500, 600, 750, 1000, 1250, 1500, 2000, 2500, 3000, 4000,
    5000, 7500, 10000, 15000, 20000, 25000, 50000, 75000, 100000, 250000,
    500000, 1000000, 2500000, 5000000, 10000000];
  const DEFAULT_TARGET = 200;

  // Hoehenwelten: Wiese, Wolken, Gewitter, Nordlicht, Weltall, Mond
  const WORLDS = ["meadow", "clouds", "storm", "aurora", "space", "moon"];
  const WORLD_AT = [100, 150, 300, 1000, 10000, MAX_MULT100];
  // Ueberblendung zur naechsten Welt: so viele Takte vor ihrer Schwelle
  const BLEND_TICKS = 90;

  // Erfolge: "Polarlicht-Pilot" ab x10, "Weltraumlama" ab x1000 ausgestiegen;
  // "Haarscharf": von Hand ab x2 ausgestiegen und hoechstens 5 % unter dem Platzpunkt
  const BRAVE_MULT = 1000;
  const SPACE_MULT = 100000;
  const CLOSE_MIN = 200;
  const CLOSE_PCT = 95;

  // Statistik: Verteilung der Platzpunkte (Untergrenzen der Faecher)
  const BUCKETS = [100, 150, 200, 500, 1000, 10000, 100000];

  // Kurve: sichtbare Zeit mindestens 8 s
  const CHART_MIN_TICKS = 8 * TPS;
  const Y_STEPS = [10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000, 10000, 20000,
    25000, 50000, 100000, 200000, 250000, 500000, 1000000, 2000000, 2500000, 5000000];
  const X_STEPS = [1, 2, 5, 10, 15, 20, 30, 60, 120]; // Sekunden

  const TWO32 = 4294967296;
  const HOUSE_TWO32 = HOUSE * 4294967296; // 416.611.827.712 < 2^53
  const MAX_INT = 9007199254740991; // 2^53 - 1 (= fairness.MAX_NONCE)

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);
  /** Python int(): Richtung 0 abschneiden. */
  const pyInt = (v) => Math.trunc(v);

  /** Ganzzahlige Division a // b fuer a >= 0, b > 0 (beide < 2^53) - exakt. */
  function idiv(a, b) {
    let q = Math.floor(a / b);
    while (q * b > a) q -= 1;
    while ((q + 1) * b <= a) q += 1;
    return q;
  }

  // ===================================================== Steigkurve
  const MU = []; // mu je Takt 0 .. MOON_TICK
  const M = []; // m100 je Takt (gedeckelt bei MAX_MULT100)

  /** Baut die Kurve einmal auf (11521 Werte, schrittweise wie oben). */
  function table() {
    if (!MU.length) {
      let mu = MU0;
      for (let i = 0; i <= MOON_TICK; i++) {
        MU.push(mu);
        M.push(Math.min(MAX_MULT100, idiv(mu, MU_UNIT)));
        mu += idiv(mu, GROWTH);
      }
    }
    return M;
  }
  /** Takt auf 0 .. MOON_TICK begrenzt (wie max(0, min(MOON_TICK, int(tick)))). */
  function clampTick(tick) {
    return Math.max(0, Math.min(MOON_TICK, pyInt(tick)));
  }
  /** mu zum Takt (ungedeckelt; Takt auf 0 .. MOON_TICK begrenzt). */
  function muAt(tick) {
    table();
    return MU[clampTick(tick)];
  }
  /** Multiplikator x100 zum Takt (100 bei Takt 0, hoechstens MAX_MULT100). */
  function multAt(tick) {
    return table()[clampTick(tick)];
  }
  /** Erster Takt mit multAt(t) >= m100 (<= 100 -> 0, > MAX -> MOON_TICK) - wie bisect_left. */
  function tickAt(m100) {
    const m = table();
    let lo = 0;
    let hi = m.length;
    while (lo < hi) {
      const mid = (lo + hi) >>> 1; // klein (< 11521): Bit-Operator unbedenklich
      if (m[mid] < m100) lo = mid + 1;
      else hi = mid;
    }
    return Math.min(MOON_TICK, lo);
  }

  // ===================================================== Platzpunkt
  /** Platzpunkt x100 aus einer 32-Bit-Zufallszahl r (0 .. 2^32-1). */
  function crashFromU32(r) {
    const v = idiv(HOUSE_TWO32, TWO32 - r);
    return Math.min(MAX_MULT100, Math.max(100, v));
  }
  /** Platzpunkt einer Runde aus ihren Fairness-Werten (zum Nachrechnen). */
  function crashPoint(server, client, nonce) {
    return crashFromU32(PG.fairness.roundRand(KEY, server, client, nonce).nextU32());
  }
  /** Takt, an dem die Runde endet: Mond -> MOON_TICK, sonst der erste Takt mit
   *  multAt(t) > crash100. Nur fuer Tests und cleanRound - das Spiel liest
   *  crash100 ausschliesslich in tick und beim Rundenabschluss. */
  function popTick(crash100) {
    if (crash100 >= MAX_MULT100) return MOON_TICK;
    return tickAt(crash100 + 1);
  }
  /** Anzahl der 2^32 Zufallswerte, bei denen crash100 >= t100 (101 <= t100). */
  function winCount(t100) {
    t100 = Math.max(MIN_CASH, Math.min(MAX_MULT100, pyInt(t100)));
    return idiv(HOUSE_TWO32, t100);
  }
  /** Erwartete Auszahlung beim Ausstieg bei t100 als Bruch [Zaehler, Nenner] (ungekuerzt). */
  function expectedReturnFrac(t100) {
    t100 = Math.max(MIN_CASH, Math.min(MAX_MULT100, pyInt(t100)));
    return [t100 * winCount(t100), 100 * TWO32];
  }
  /** Erwartete Auszahlung (Anteil am Einsatz) = float(Fraction) in Python. */
  function expectedReturn(t100) {
    const [num, den] = expectedReturnFrac(t100);
    return num / den;
  }
  /** Chance, t100 zu erreichen, in Prozent als Text (kaufmaennisch gerundet):
   *  ab 1 % zwei Nachkommastellen ("48.50"), darunter vier ("0.0010"). */
  function fmtChance(t100, sep = ".") {
    const num = winCount(t100);
    const den = TWO32;
    if (num * 100 >= den) {
      const v = idiv(num * 20000 + den, den * 2);
      return idiv(v, 100) + sep + String(v % 100).padStart(2, "0");
    }
    const v = idiv(num * 2000000 + den, den * 2);
    return idiv(v, 10000) + sep + String(v % 10000).padStart(4, "0");
  }

  // ===================================================== Auszahlung
  /** [Auszahlung, neuer Uebertrag]: stake * m100 + carry in Hundertsteln, ganze
   *  Chips werden ausgezahlt, der Rest (0..99) bleibt fuer die naechste Auszahlung. */
  function payoutCarry(stake, m100, carry) {
    const total = stake * m100 + carry;
    return [idiv(total, 100), total % 100];
  }
  /** Gespeicherter Uebertrag: ganze Zahl 0..99, sonst 0. */
  function cleanCarry(raw) {
    return isInt(raw) && raw >= 0 && raw <= 99 ? raw : 0;
  }
  /** Anteil, der beim Halbieren aussteigt. */
  function halfStake(b) {
    return idiv(b, 2);
  }
  /** Anteil, der nach dem Halbieren weiterfliegt. */
  function restStake(b) {
    return b - idiv(b, 2);
  }
  /** Noch fliegender Einsatz eines Teil-Einsatzes (0 = ganz ausgestiegen). */
  function riding(bet) {
    if (bet.xm) return 0;
    return bet.hm ? restStake(bet.b) : bet.b;
  }
  /** Fliegt in der Runde noch Geld mit? */
  function flying(rnd) {
    return rnd.bets.some((b) => riding(b) > 0);
  }
  /** Gesamteinsatz der Runde. */
  function stakeTotal(rnd) {
    let total = 0;
    for (const b of rnd.bets) total += b.b;
    return total;
  }
  /** Bisher ausgezahlt (alle Teile). */
  function paidTotal(rnd) {
    let total = 0;
    for (const b of rnd.bets) total += b.p;
    return total;
  }
  /** Gueltiges Auto-Ziel? (ganze Zahl MIN_CASH .. MAX_MULT100) */
  function targetOk(t100) {
    return isInt(t100) && t100 >= MIN_CASH && t100 <= MAX_MULT100;
  }
  /** Naechstes Auto-Ziel der Leiter in Richtung direction (+1/-1). */
  function stepTarget(t100, direction) {
    t100 = Math.max(MIN_CASH, Math.min(MAX_MULT100, pyInt(t100)));
    if (direction > 0) {
      for (const v of TARGETS) if (v > t100) return v;
      return TARGETS[TARGETS.length - 1];
    }
    for (let i = TARGETS.length - 1; i >= 0; i--) if (TARGETS[i] < t100) return TARGETS[i];
    return TARGETS[0];
  }
  /** Teil-Einsatz zu Rundenbeginn (a = Auto-Ziel oder 0). */
  function newBet(b, a) {
    return { b, a, hm: 0, xm: 0, p: 0, man: 0 };
  }
  /** Neue Runde aus dem Fairness-Eintrag (fairness.begin) und den Einsaetzen
   *  pairs = [[Einsatz, Auto-Ziel oder 0], ...] (1 .. MAX_BETS). */
  function newRound(entry, pairs) {
    return { n: entry.n, s: entry.s, c: entry.c, h: entry.h, t: 0, bets: pairs.map(([b, a]) => newBet(b, a)) };
  }
  /** Den noch fliegenden Teil eines Einsatzes bei m100 auszahlen -> [Gewinn, Uebertrag]. */
  function pay(bet, m100, carry) {
    const res = payoutCarry(riding(bet), m100, carry);
    bet.xm = m100;
    bet.p += res[0];
    return res;
  }
  /** Einen Takt weiter (Takt-Regel oben). Liefert [Ereignisse, Uebertrag, Ende]:
   *  Ereignisse = [[Index, "auto"|"moon", m100, Gewinn], ...] in Auszahlungs-
   *  reihenfolge, Ende = null (fliegt weiter) / "pop" / "moon". */
  function tick(rnd, crash100, carry) {
    rnd.t += 1;
    const m = multAt(rnd.t);
    const events = [];
    rnd.bets.forEach((bet, i) => {
      const a = bet.a;
      if (a && riding(bet) > 0 && a <= m && a <= crash100) {
        const [win, c] = pay(bet, a, carry);
        carry = c;
        events.push([i, "auto", a, win]);
      }
    });
    if (crash100 >= MAX_MULT100 && m >= MAX_MULT100) {
      rnd.bets.forEach((bet, i) => {
        if (riding(bet) > 0) {
          const [win, c] = pay(bet, MAX_MULT100, carry);
          carry = c;
          events.push([i, "moon", MAX_MULT100, win]);
        }
      });
      return [events, carry, "moon"];
    }
    if (m > crash100) return [events, carry, "pop"];
    return [events, carry, null];
  }
  /** Von Hand aussteigen (ganz oder Haelfte) beim aktuellen Takt.
   *  Liefert [Gewinn, m100, neuer Uebertrag] oder null (nicht erlaubt: unter
   *  MIN_CASH, nichts mehr im Flug, Haelfte schon genommen). */
  function cashOut(rnd, i, half, carry) {
    if (!(i >= 0 && i < rnd.bets.length)) return null;
    const bet = rnd.bets[i];
    const m = multAt(rnd.t);
    if (m < MIN_CASH || riding(bet) <= 0) return null;
    let res;
    if (half) {
      if (bet.hm) return null;
      res = payoutCarry(halfStake(bet.b), m, carry);
      bet.hm = m;
      bet.man |= 1;
    } else {
      res = payoutCarry(riding(bet), m, carry);
      bet.xm = m;
      bet.man |= 2;
    }
    bet.p += res[0];
    return [res[0], m, res[1]];
  }
  /** Hoechster Ausstieg der Runde (m100, 0 = keiner). */
  function bestCash(rnd) {
    let best = 0;
    for (const b of rnd.bets) best = Math.max(best, b.hm, b.xm);
    return best;
  }
  /** "Haarscharf": ein Ausstieg VON HAND ab CLOSE_MIN und hoechstens 5 % unter dem
   *  Platzpunkt (ganzzahlig: m * 100 >= CLOSE_PCT * crash100). */
  function closeCall(rnd, crash100) {
    for (const b of rnd.bets) {
      for (const [bit, m] of [[1, b.hm], [2, b.xm]]) {
        if ((b.man & bit) && m >= CLOSE_MIN && m * 100 >= CLOSE_PCT * crash100) return true;
      }
    }
    return false;
  }
  /** Ergebnis fuer das Fairness-Log (nur ganze Zahlen, Tiefe 3). */
  function outcome(rnd, crash100) {
    return { x: crash100, t: rnd.t, b: rnd.bets.map((b) => [b.b, b.hm, b.xm, b.p]) };
  }
  /** Passt ein Log-Ergebnis zum nachgerechneten Platzpunkt? (Platzpunkt gleich,
   *  jeder Ausstieg 0 oder MIN_CASH .. crash100) */
  function checkOutcome(o, crash100) {
    if (!isDict(o) || o.x !== crash100) return false;
    const bets = o.b;
    if (!Array.isArray(bets) || !(bets.length >= 1 && bets.length <= MAX_BETS)) return false;
    for (const row of bets) {
      if (!Array.isArray(row) || row.length !== 4 || !row.every((v) => isInt(v))) return false;
      for (const m of [row[1], row[2]]) {
        if (m && !(m >= MIN_CASH && m <= crash100)) return false;
      }
    }
    return true;
  }

  // ===================================================== Hoehenwelten / Kurve
  /** Welt (0 = Wiese .. 5 = Mond) zum Multiplikator. */
  function worldOf(m100) {
    let w = 0;
    WORLD_AT.forEach((lo, i) => {
      if (m100 >= lo) w = i;
    });
    return w;
  }
  const WORLD_T = [];
  /** Takt, an dem jede Welt beginnt (Wiese = 0) - einmal berechnet. */
  function worldTicks() {
    if (!WORLD_T.length) for (const v of WORLD_AT) WORLD_T.push(tickAt(v));
    return WORLD_T.slice();
  }
  /** [Welt, Anteil 0..255 der naechsten Welt] zum (gebrochenen) Takt - die
   *  Ueberblendung beginnt BLEND_TICKS vor der naechsten Schwelle. Nur fuers
   *  Zeichnen; die Mond-Welt (5) erreicht die Kurve nie von selbst. */
  function worldBlend(tickF) {
    const starts = worldTicks();
    let w = 0;
    for (let i = 1; i < 5; i++) if (tickF >= starts[i]) w = i;
    if (w >= 4) return [4, 0];
    const nxt = starts[w + 1];
    const left = nxt - tickF;
    if (left >= BLEND_TICKS) return [w, 0];
    return [w, Math.max(0, Math.min(255, Math.trunc(((BLEND_TICKS - left) * 255) / BLEND_TICKS)))];
  }
  /** Achsen der Kurve (wachsen mit): [x_max in Takten, y_max in m100]. */
  function chartSpan(tick) {
    const xMax = Math.max(CHART_MIN_TICKS, tick + Math.floor(tick / 5));
    const m = multAt(tick);
    const yMax = Math.max(200, m + Math.floor((m - 100) / 4));
    return [xMax, yMax];
  }
  /** Waagerechte Hilfslinien (m100-Werte ueber x1,00, hoechstens 4). */
  function chartYlines(yMax) {
    const span = Math.max(1, yMax - 100);
    let step = Y_STEPS[Y_STEPS.length - 1];
    for (const s of Y_STEPS) {
      if (Math.floor(span / s) <= 4) {
        step = s;
        break;
      }
    }
    const first = (Math.floor(100 / step) + 1) * step;
    const out = [];
    for (let v = first; v <= yMax && out.length < 4; v += step) out.push(v);
    return out;
  }
  /** Senkrechte Hilfslinien als Takte (ganze Sekunden, hoechstens 5). */
  function chartXlines(xMax) {
    let step = X_STEPS[X_STEPS.length - 1];
    for (const s of X_STEPS) {
      if (Math.floor(xMax / (s * TPS)) <= 5) {
        step = s;
        break;
      }
    }
    const out = [];
    const n = Math.floor(xMax / (step * TPS));
    for (let v = 1; v <= n && out.length < 5; v++) out.push(v * step * TPS);
    return out;
  }

  // ===================================================== Statistik
  /** Fach der Verteilung (0 .. BUCKETS.length-1). */
  function bucketOf(crash100) {
    let b = 0;
    BUCKETS.forEach((lo, i) => {
      if (crash100 >= lo) b = i;
    });
    return b;
  }
  /** Erwartete Wahrscheinlichkeit eines Fachs als Bruch [Zaehler, 2^32] (exakt, ungekuerzt). */
  function bucketProbFrac(i) {
    const lo = BUCKETS[i];
    const pLo = lo <= 100 ? TWO32 : winCount(lo);
    if (i + 1 >= BUCKETS.length) return [pLo, TWO32];
    return [pLo - winCount(BUCKETS[i + 1]), TWO32];
  }
  /** Erwartete Wahrscheinlichkeit eines Fachs (= float(Fraction) in Python). */
  function bucketProb(i) {
    const [num, den] = bucketProbFrac(i);
    return num / den;
  }

  const X_COUNTERS = ["cash", "half", "auto", "inst", "moon", "close", "top"];

  /** Leere Crash-Zaehler fuer stats.x. */
  function blankX() {
    const out = { hits: new Array(BUCKETS.length).fill(0) };
    for (const k of X_COUNTERS) out[k] = 0;
    return out;
  }
  /** Crash-Zaehler pruefen: hits = Liste mit BUCKETS.length Zahlen >= 0, sonst Nullen;
   *  jeder Zaehler ganze Zahl >= 0, sonst 0. */
  function cleanX(raw) {
    const out = blankX();
    if (!isDict(raw)) return out;
    const hits = raw.hits;
    if (Array.isArray(hits) && hits.length === BUCKETS.length) {
      out.hits = hits.map((v) => (isInt(v) && v >= 0 && v <= MAX_INT ? v : 0));
    }
    for (const k of X_COUNTERS) {
      const v = raw[k];
      if (isInt(v) && v >= 0 && v <= MAX_INT) out[k] = v;
    }
    return out;
  }
  /** Eine fertige Runde in die Crash-Zaehler eintragen. */
  function recordX(x, rnd, crash100, end) {
    x.hits[bucketOf(crash100)] += 1;
    for (const b of rnd.bets) {
      if (b.hm) {
        x.cash += 1;
        x.half += 1;
      }
      if (b.xm) {
        x.cash += 1;
        if (!(b.man & 2)) x.auto += 1;
      }
    }
    if (crash100 < MIN_CASH) x.inst += 1;
    if (end === "moon") x.moon += 1;
    if (closeCall(rnd, crash100)) x.close += 1;
    x.top = Math.max(x.top, crash100);
    return x;
  }

  // ===================================================== Pruefen / Laden
  /** Verlauf der letzten Runden: Liste von Platzpunkten x100. */
  function cleanHistory(raw) {
    if (!Array.isArray(raw)) return [];
    return raw.filter((v) => isInt(v) && v >= 100 && v <= MAX_MULT100).slice(-HISTORY);
  }
  function cleanBet(raw, m, crash100) {
    if (!isDict(raw)) return null;
    const { b, a, hm, xm, p, man } = raw;
    if (![b, a, hm, xm, p, man].every((v) => isInt(v))) return null;
    if (!BETS.includes(b) || !(a === 0 || targetOk(a)) || !(man >= 0 && man <= 3) || p < 0) return null;
    for (const v of [hm, xm]) {
      if (v && !(v >= MIN_CASH && v <= Math.min(m, crash100))) return null;
    }
    if (hm && xm && xm < hm) return null;
    if (((man & 1) && !hm) || ((man & 2) && !xm)) return null;
    if (p > idiv(b * MAX_MULT100, 100) + 1) return null;
    const bet = { b, a, hm, xm, p, man };
    // ein schon erreichtes Auto-Ziel muss ausgezahlt sein
    if (a && riding(bet) > 0 && a <= m && a <= crash100) return null;
    return bet;
  }
  /** Gespeicherte, noch offene Runde pruefen (aus lamabank-Extra "round") oder null.
   *  Gueltig: Fairness-Werte mit h = sha256(s), Takt 0 .. MOON_TICK vor dem
   *  Rundenende (Platzpunkt aus den Seeds neu berechnet), 1 .. MAX_BETS Einsaetze
   *  mit Ausstiegen hoechstens min(multAt(t), crash100). Eine Runde ohne
   *  fliegende Teile ist gueltig (das Spiel schliesst sie beim Laden ab). */
  function cleanRound(raw) {
    if (!isDict(raw)) return null;
    const { n, s, c, h, t } = raw;
    const F = PG.fairness;
    if (!(F.validNonce(n) && F.validServer(s) && F.validClient(c) && F.validHash(h) && F.commit(s) === h)) return null;
    if (!isInt(t) || !(t >= 0 && t <= MOON_TICK)) return null;
    const crash100 = crashPoint(s, c, n);
    const m = multAt(t);
    if (crash100 >= MAX_MULT100) {
      if (t >= MOON_TICK) return null;
    } else if (m > crash100) {
      return null;
    }
    const rawBets = raw.bets;
    if (!Array.isArray(rawBets) || !(rawBets.length >= 1 && rawBets.length <= MAX_BETS)) return null;
    const bets = [];
    for (const rb of rawBets) {
      const bet = cleanBet(rb, m, crash100);
      if (bet === null) return null;
      bets.push(bet);
    }
    return { n, s, c, h, t, bets };
  }

  // ===================================================== Formatierung
  /** Multiplikator x100 als Text mit zwei Stellen: 235 -> "2.35". */
  function fmtMult(m100, sep = ".") {
    return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
  }
  /** Kurzform fuer Pillen/Leiste: x1,23 · x12,3 · x123 · x12,3k · x100k (ohne x). */
  function fmtShort(m100, sep = ".") {
    if (m100 < 1000) return Math.floor(m100 / 100) + sep + String(m100 % 100).padStart(2, "0");
    if (m100 < 10000) return Math.floor(m100 / 100) + sep + (Math.floor(m100 / 10) % 10);
    if (m100 < 100000) return String(Math.floor(m100 / 100));
    if (m100 < 1000000) return Math.floor(m100 / 100000) + sep + (Math.floor(m100 / 10000) % 10) + "k";
    return Math.floor(m100 / 100000) + "k";
  }

  PG.lamaCrashLogic = {
    KEY, HOUSE, MAX_MULT100, MIN_CASH, BETS, MAX_BETS, HISTORY, TPS, MU0, GROWTH, MU_UNIT, MOON_TICK,
    SPEED_TURBO, SPEED_FF, TARGETS, DEFAULT_TARGET, WORLDS, WORLD_AT, BLEND_TICKS, BRAVE_MULT, SPACE_MULT,
    CLOSE_MIN, CLOSE_PCT, BUCKETS, CHART_MIN_TICKS, Y_STEPS, X_STEPS, TWO32, X_COUNTERS,
    isInt, table, muAt, multAt, tickAt, crashFromU32, crashPoint, popTick, winCount, expectedReturn,
    expectedReturnFrac, fmtChance, payoutCarry, cleanCarry, halfStake, restStake, riding, flying,
    stakeTotal, paidTotal, targetOk, stepTarget, newBet, newRound, tick, cashOut, bestCash, closeCall,
    outcome, checkOutcome, worldOf, worldTicks, worldBlend, chartSpan, chartYlines, chartXlines,
    bucketOf, bucketProb, bucketProbFrac, blankX, cleanX, recordX, cleanHistory, cleanRound,
    fmtMult, fmtShort,
  };
})();
