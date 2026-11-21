/*
 * fairness.js - Nachpruefbarer Zufall der Chip-Spiele (1:1 aus fairness.py)
 * ==========================================================================
 * Vor jeder Runde steht ein geheimer Server-Seed fest (32 Hex-Zeichen),
 * angezeigt wird nur sein Pruefcode sha256Hex(server). Dazu kommen der
 * Client-Seed des Spielers und die Nonce (Rundennummer). Der Rundenzufall ist
 * new PG.seedrand.Rand(PG.seedrand.seedFrom(spiel, server, client, nonce)) -
 * in Python, JS und Rust bitgleich. Nach der Runde wird der Server-Seed
 * gezeigt; wer nachrechnet, sieht, dass nichts nachtraeglich geaendert wurde.
 *
 * SHA-256 ist hier selbst geschrieben (synchron, ohne crypto.subtle - laeuft
 * auch ueber file://). Neue Seeds kommen aus crypto.getRandomValues (Rueckfall
 * Math.random); fuer Tests kann jede Funktion, die Seeds zieht, als letztes
 * Argument eine Funktion bekommen, die vorzeichenlose 32-Bit-Zahlen liefert.
 * tests/audit_chiplogic.py vergleicht beide Fassungen (ueber
 * web/tools/fairness_dump.js).
 *
 *   const entry = PG.fairness.begin(state);   // {n, s, c, h} dieser Runde
 *   PG.fairness.advance(state);               // naechster Pruefcode steht fest
 *   const rand = PG.fairness.entryRand("lamamines", entry);
 */
(function () {
  "use strict";

  const CLIENT_ALPHABET = "abcdefghijkmnpqrstuvwxyz23456789";
  const CLIENT_LEN = 8;
  const CLIENT_MAX = 16;
  const LOG = 12;
  const MAX_NONCE = 9007199254740991; // 2^53 - 1
  const HEX = "0123456789abcdef";

  // ---------------------------------------------------------------- SHA-256
  const K = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ];

  /** Text -> UTF-8-Bytes (ohne TextEncoder, laeuft ueberall). */
  function utf8(text) {
    const s = String(text);
    const out = [];
    for (let i = 0; i < s.length; i++) {
      let c = s.charCodeAt(i);
      if (c >= 0xd800 && c <= 0xdbff && i + 1 < s.length) {
        const d = s.charCodeAt(i + 1);
        if (d >= 0xdc00 && d <= 0xdfff) {
          c = 0x10000 + ((c - 0xd800) << 10) + (d - 0xdc00);
          i++;
        }
      }
      if (c < 0x80) out.push(c);
      else if (c < 0x800) out.push(0xc0 | (c >> 6), 0x80 | (c & 63));
      else if (c < 0x10000) out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
      else out.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 63), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
    }
    return out;
  }

  const rotr = (x, n) => (x >>> n) | (x << (32 - n));

  /** SHA-256 eines Textes (UTF-8) als 64 kleine Hex-Zeichen. */
  function sha256Hex(text) {
    const bytes = utf8(text);
    const len = bytes.length;
    const total = Math.ceil((len + 9) / 64) * 64;
    const buf = new Uint8Array(total);
    for (let i = 0; i < len; i++) buf[i] = bytes[i];
    buf[len] = 0x80;
    const bits = len * 8;
    const hi = Math.floor(bits / 4294967296);
    const lo = bits >>> 0;
    buf[total - 8] = (hi >>> 24) & 255;
    buf[total - 7] = (hi >>> 16) & 255;
    buf[total - 6] = (hi >>> 8) & 255;
    buf[total - 5] = hi & 255;
    buf[total - 4] = (lo >>> 24) & 255;
    buf[total - 3] = (lo >>> 16) & 255;
    buf[total - 2] = (lo >>> 8) & 255;
    buf[total - 1] = lo & 255;
    const h = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    const w = new Array(64);
    for (let off = 0; off < total; off += 64) {
      for (let i = 0; i < 16; i++) {
        const j = off + i * 4;
        w[i] = ((buf[j] << 24) | (buf[j + 1] << 16) | (buf[j + 2] << 8) | buf[j + 3]) >>> 0;
      }
      for (let i = 16; i < 64; i++) {
        const s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >>> 3);
        const s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >>> 10);
        w[i] = (w[i - 16] + s0 + w[i - 7] + s1) >>> 0;
      }
      let [a, b, c, d, e, f, g, hh] = h;
      for (let i = 0; i < 64; i++) {
        const S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
        const ch = (e & f) ^ (~e & g);
        const t1 = (hh + S1 + ch + K[i] + w[i]) >>> 0;
        const S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
        const maj = (a & b) ^ (a & c) ^ (b & c);
        const t2 = (S0 + maj) >>> 0;
        hh = g;
        g = f;
        f = e;
        e = (d + t1) >>> 0;
        d = c;
        c = b;
        b = a;
        a = (t1 + t2) >>> 0;
      }
      h[0] = (h[0] + a) >>> 0;
      h[1] = (h[1] + b) >>> 0;
      h[2] = (h[2] + c) >>> 0;
      h[3] = (h[3] + d) >>> 0;
      h[4] = (h[4] + e) >>> 0;
      h[5] = (h[5] + f) >>> 0;
      h[6] = (h[6] + g) >>> 0;
      h[7] = (h[7] + hh) >>> 0;
    }
    return h.map((v) => v.toString(16).padStart(8, "0")).join("");
  }

  /** Pruefcode eines Server-Seeds (wird VOR der Runde gezeigt). */
  function commit(server) {
    return sha256Hex(server);
  }

  // ---------------------------------------------------------------- Seeds
  /** Vorzeichenlose 32-Bit-Zufallszahl (crypto, Rueckfall Math.random). */
  function randomU32() {
    try {
      const c = typeof crypto !== "undefined" ? crypto : (typeof window !== "undefined" ? window.crypto : null);
      if (c && c.getRandomValues) {
        const a = new Uint32Array(1);
        c.getRandomValues(a);
        return a[0] >>> 0;
      }
    } catch (e) { /* Rueckfall unten */ }
    return Math.floor(Math.random() * 4294967296) >>> 0;
  }

  /** Neuer Server-Seed: 32 Hex-Zeichen (4 x 32 Bit). */
  function newServerSeed(next) {
    const src = next || randomU32;
    let s = "";
    for (let i = 0; i < 4; i++) s += (src() >>> 0).toString(16).padStart(8, "0");
    return s;
  }
  /** Neuer Client-Seed: 8 Zeichen aus CLIENT_ALPHABET (32 Zeichen = 5 Bit, ohne Schieflage). */
  function newClientSeed(next) {
    const src = next || randomU32;
    let s = "";
    for (let i = 0; i < CLIENT_LEN; i++) s += CLIENT_ALPHABET[src() & 31];
    return s;
  }

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);
  const isHex = (v, n) => typeof v === "string" && v.length === n && [...v].every((ch) => HEX.includes(ch));
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);

  /** Server-Seed gueltig? (genau 32 kleine Hex-Zeichen) */
  function validServer(v) {
    return isHex(v, 32);
  }
  /** Client-Seed gueltig? (1-16 Zeichen aus CLIENT_ALPHABET) */
  function validClient(v) {
    return typeof v === "string" && v.length >= 1 && v.length <= CLIENT_MAX
      && [...v].every((ch) => CLIENT_ALPHABET.includes(ch));
  }
  /** Nonce gueltig? (ganze Zahl 0 .. MAX_NONCE) */
  function validNonce(v) {
    return isInt(v) && v >= 0 && v <= MAX_NONCE;
  }
  /** Pruefcode gueltig? (genau 64 kleine Hex-Zeichen) */
  function validHash(v) {
    return isHex(v, 64);
  }

  /** Frischer Zustand: neuer Server- und Client-Seed, Nonce 0. */
  function blankState(next) {
    const server = newServerSeed(next);
    const client = newClientSeed(next);
    return { server, client, nonce: 0 };
  }
  /** Gespeicherten Zustand pruefen - Feld fuer Feld (ungueltig -> neu bzw. 0). */
  function cleanState(raw, next) {
    if (!isDict(raw)) raw = {};
    let server = raw.server;
    if (!validServer(server)) server = newServerSeed(next);
    let client = raw.client;
    if (!validClient(client)) client = newClientSeed(next);
    let nonce = raw.nonce;
    if (!validNonce(nonce)) nonce = 0;
    return { server, client, nonce };
  }

  /** Zufall einer Runde: seedrand.Rand aus (Spiel, Server, Client, Nonce). */
  function roundRand(game, server, client, nonce) {
    return new PG.seedrand.Rand(PG.seedrand.seedFrom(game, server, client, nonce));
  }
  /** Zufall der Runde zu einem Eintrag aus begin()/dem Fairness-Log. */
  function entryRand(game, entry) {
    return roundRand(game, entry.s, entry.c, entry.n);
  }
  /** Runde beginnt: Eintrag {n, s, c, h} mit den Werten dieser Runde ("s" erst danach zeigen). */
  function begin(state) {
    return { n: state.nonce, s: state.server, c: state.client, h: commit(state.server) };
  }
  /** Naechste Runde vorbereiten: Nonce + 1, neuer Server-Seed (Client bleibt). */
  function advance(state, next) {
    state.nonce = Math.min(MAX_NONCE, state.nonce + 1);
    state.server = newServerSeed(next);
    return state;
  }
  /** Neuer Client-Seed (Knopf im Fenster F; nicht waehrend einer Runde). */
  function newClient(state, next) {
    state.client = newClientSeed(next);
    return state;
  }

  /** Nur JSON-Grundtypen (null, bool, ganze Zahlen bis 2^53-1, Text, Listen,
   *  Objekte), keine Gleitkommazahlen, Tiefe hoechstens 4 (wie fairness._plain). */
  function plain(v, depth = 0) {
    if (v === null || typeof v === "boolean" || typeof v === "string") return true;
    if (typeof v === "number") return isInt(v) && Math.abs(v) <= MAX_NONCE;
    if (depth >= 4) return false;
    if (Array.isArray(v)) return v.every((x) => plain(x, depth + 1));
    if (isDict(v)) return Object.keys(v).every((k) => plain(v[k], depth + 1));
    return false;
  }

  /** Log-Eintrag {n, s, c, h, o} aus begin()-Eintrag + Ergebnis. */
  function makeEntry(entry, outcome) {
    return { n: entry.n, s: entry.s, c: entry.c, h: entry.h, o: outcome !== undefined && plain(outcome) ? outcome : null };
  }
  /** Fertige Runde ins Log (neueste zuletzt, hoechstens LOG Eintraege). */
  function record(log, entry, outcome) {
    log.push(makeEntry(entry, outcome));
    if (log.length > LOG) log.splice(0, log.length - LOG);
    return log;
  }
  /** Gespeichertes Fairness-Log pruefen: hoechstens LOG gueltige Eintraege. */
  function cleanLog(raw) {
    if (!Array.isArray(raw)) return [];
    const out = [];
    for (const e of raw) {
      if (!isDict(e)) continue;
      const { n, s, c, h } = e;
      if (!(validNonce(n) && validServer(s) && validClient(c) && validHash(h))) continue;
      const o = e.o;
      out.push({ n, s, c, h, o: o !== undefined && plain(o) ? o : null });
    }
    return out.slice(-LOG);
  }
  /** Stimmt der Pruefcode des Eintrags mit sha256(Server-Seed) ueberein? */
  function verify(entry) {
    if (!isDict(entry)) return false;
    return validServer(entry.s) && validHash(entry.h) && commit(entry.s) === entry.h;
  }
  /** Gekuerzter Pruefcode/Seed fuer die Anzeige: erste n Zeichen + "…". */
  function short(h, n = 12) {
    h = String(h);
    return h.length <= n ? h : h.slice(0, n) + "…";
  }

  PG.fairness = {
    CLIENT_ALPHABET, CLIENT_LEN, CLIENT_MAX, LOG, MAX_NONCE,
    sha256Hex, commit, randomU32, newServerSeed, newClientSeed,
    validServer, validClient, validNonce, validHash, blankState, cleanState,
    roundRand, entryRand, begin, advance, newClient, plain, makeEntry, record, cleanLog, verify, short,
  };
})();
