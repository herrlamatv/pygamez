/*
 * seedrand.js - Zufallsgenerator mit exakt denselben Zahlen wie seedrand.py
 * ==========================================================================
 * mulberry32 in reiner 32-Bit-Ganzzahl-Arithmetik (Math.imul, >>> 0), dazu
 * randint/shuffle/choice, die die Zufallszahlen in derselben Reihenfolge
 * verbrauchen wie die Python-Fassung. So sind Tagesstrecke, Tageswort,
 * Tages-Sudoku und die neuen Sudoku-Varianten am PC und im Browser identisch.
 *
 *   const rng = new PG.seedrand.Rand(PG.seedrand.seedFrom("crossy", 42));
 *   rng.randint(1, 6); rng.choice(arr); rng.shuffle(arr);
 *   PG.seedrand.dailySeed("wordle");
 */
(function () {
  "use strict";

  const PG = window.PG;

  class Rand {
    constructor(seed) {
      this.state = (seed || 0) >>> 0;
    }
    nextU32() {
      this.state = (this.state + 0x6d2b79f5) >>> 0;
      let t = this.state;
      t = Math.imul(t ^ (t >>> 15), t | 1) >>> 0;
      t = (((t + Math.imul(t ^ (t >>> 7), t | 61)) >>> 0) ^ t) >>> 0;
      return (t ^ (t >>> 14)) >>> 0;
    }
    random() {
      return this.nextU32() / 4294967296;
    }
    randint(a, b) {
      return a + Math.floor(this.random() * (b - a + 1));
    }
    uniform(a, b) {
      return a + (b - a) * this.random();
    }
    chance(p) {
      return this.random() < p;
    }
    choice(seq) {
      return seq[Math.floor(this.random() * seq.length)];
    }
    shuffle(items) {
      for (let i = items.length - 1; i > 0; i--) {
        const j = Math.floor(this.random() * (i + 1));
        const tmp = items[i];
        items[i] = items[j];
        items[j] = tmp;
      }
      return items;
    }
    weighted(weights) {
      let total = 0;
      for (const w of weights) total += w;
      const r = this.random() * total;
      let acc = 0;
      for (let i = 0; i < weights.length; i++) {
        acc += weights[i];
        if (r < acc) return i;
      }
      return weights.length - 1;
    }
  }

  /** 32-Bit-Seed aus Teilen (FNV-1a), wie seed_from() in seedrand.py. Nur ASCII. */
  function seedFrom(...parts) {
    let h = 0x811c9dc5;
    const s = parts.map((p) => (typeof p === "boolean" ? (p ? "True" : "False") : String(p))).join("|");
    for (let i = 0; i < s.length; i++) {
      h ^= s.charCodeAt(i) & 0xff;
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h >>> 0;
  }

  function pad2(n) {
    return (n < 10 ? "0" : "") + n;
  }

  /** Heutiges lokales Datum als "YYYY-MM-DD". */
  function todayStr(d) {
    d = d || new Date();
    return d.getFullYear() + "-" + pad2(d.getMonth() + 1) + "-" + pad2(d.getDate());
  }

  function dailySeed(game, date) {
    return seedFrom("daily", game, date || todayStr());
  }

  /** Tage seit 2026-01-01 (wie day_index() in seedrand.py). */
  function dayIndex(date) {
    const [y, m, d] = (date || todayStr()).split("-").map(Number);
    return Math.round((Date.UTC(y, m - 1, d) - Date.UTC(2026, 0, 1)) / 86400000);
  }

  PG.seedrand = { Rand, seedFrom, todayStr, dailySeed, dayIndex };
})();
