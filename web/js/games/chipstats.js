/*
 * chipstats.js - Statistik der Chip-Spiele (1:1 aus chipstats.py)
 * ================================================================
 * Je Profil im lamabank-Extra des Spiels unter "stats":
 *   {rounds, wins, losses, wagered, paid, net, best_win, best_mult,
 *    curve: [Bilanz nach jeder Runde], x: {...}}
 * x gehoert dem Spiel (z.B. Plinko-Treffer je Reihenzahl). Auszahlung ueber
 * dem Einsatz = Gewinn, darunter = Verlust, genau der Einsatz = keins von beiden.
 */
(function () {
  "use strict";

  const CURVE = 100;
  const MAX_INT = 9007199254740991; // 2^53 - 1
  const COUNTERS = ["rounds", "wins", "losses", "wagered", "paid", "net", "best_win", "best_mult"];

  const isInt = (v) => typeof v === "number" && Number.isInteger(v) && Math.abs(v) <= MAX_INT;
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);

  /** Leere Statistik. */
  function blank() {
    return { rounds: 0, wins: 0, losses: 0, wagered: 0, paid: 0, net: 0, best_win: 0, best_mult: 0, curve: [], x: {} };
  }

  /** Nur JSON-Grundtypen, keine Gleitkommazahlen, Tiefe hoechstens 4. */
  function plain(v, depth = 0) {
    if (v === null || typeof v === "boolean" || typeof v === "string") return true;
    if (typeof v === "number") return isInt(v);
    if (depth >= 4) return false;
    if (Array.isArray(v)) return v.every((x) => plain(x, depth + 1));
    if (isDict(v)) return Object.keys(v).every((k) => plain(v[k], depth + 1));
    return false;
  }

  /** Gespeicherte Statistik pruefen (Zaehler int >= 0, net = paid - wagered,
   *  curve = letzte CURVE ganze Zahlen, x = dict aus JSON-Grundtypen). */
  function clean(raw) {
    const st = blank();
    if (!isDict(raw)) return st;
    for (const k of COUNTERS) {
      const v = raw[k];
      if (k !== "net" && isInt(v) && v >= 0) st[k] = v;
    }
    st.net = st.paid - st.wagered;
    if (Array.isArray(raw.curve)) st.curve = raw.curve.filter((v) => isInt(v)).slice(-CURVE);
    if (isDict(raw.x)) {
      for (const k of Object.keys(raw.x)) if (plain(raw.x[k], 1)) st.x[k] = raw.x[k];
    }
    return st;
  }

  /** Eine Runde (bzw. Kugel) eintragen: Einsatz, Auszahlung, Multiplikator x100. */
  function record(st, wager, paid, mult100) {
    st.rounds += 1;
    if (paid > wager) st.wins += 1;
    else if (paid < wager) st.losses += 1;
    st.wagered += wager;
    st.paid += paid;
    st.net += paid - wager;
    st.best_win = Math.max(st.best_win, paid - wager);
    st.best_mult = Math.max(st.best_mult, mult100);
    st.curve.push(st.net);
    if (st.curve.length > CURVE) st.curve.splice(0, st.curve.length - CURVE);
    return st;
  }

  /** Tatsaechliche Auszahlungsquote (paid / wagered) oder null ohne Einsatz. */
  function rtp(st) {
    return st.wagered > 0 ? st.paid / st.wagered : null;
  }
  /** Anteil gewonnener Runden oder null ohne Runde. */
  function winRate(st) {
    return st.rounds > 0 ? st.wins / st.rounds : null;
  }

  PG.chipstats = { CURVE, MAX_INT, blank, clean, record, rtp, winRate };
})();
