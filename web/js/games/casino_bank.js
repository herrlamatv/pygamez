/*
 * casino_bank.js - Lama-Bank: gemeinsames Chip-Konto (Port von lamabank.py)
 * ==========================================================================
 * Blackjack, Poker und Casino setzen dieselben Lama-Chips. Gespeichert wird im
 * localStorage unter "pygamez.mem.casino" im gleichen Format wie der
 * mem.json-Abschnitt "casino" der Desktop-Version:
 *
 *   {chips, ledger: {blackjack, poker, casino}, peak: {...}, escrow, refills,
 *    migrated, extra}
 *
 * - Einsatz sofort abbuchen + speichern (Hand verlassen = Einsatz weg).
 * - Poker-Tischstapel liegt als escrow; load() erstattet ihn beim nächsten Start.
 * - Je Spiel eigene Bilanz; Highscore = Höchststand von 1000 + Bilanz (bzw. der
 *   übernommene alte Bestwert). Bank-Kredite zählen nicht.
 * - Pleite = weniger als der Mindesteinsatz -> Kredit, Konto wieder 1000.
 * - Einmalige Migration aus "mem.blackjack"/"mem.poker" (Chips = Maximum,
 *   Bestwert = alter best bzw. Highscore). Die Highscores werden nur gelesen.
 */
(function () {
  "use strict";

  const KEY = "mem.casino";
  const START_CHIPS = 1000;
  const GAMES = ["blackjack", "poker", "casino"];
  const MIN_BET = { blackjack: 10, poker: 20, "poker:video": 10, casino: 1, "casino:slots": 10 };

  let data = null;

  function toInt(v, def, lo) {
    if (typeof v !== "number" || !Number.isFinite(v)) return def;
    const n = Math.trunc(v);
    return lo === undefined ? n : Math.max(lo, n);
  }

  function blank() {
    const d = { chips: START_CHIPS, ledger: {}, peak: {}, escrow: 0, refills: 0, migrated: false, extra: {} };
    for (const g of GAMES) {
      d.ledger[g] = 0;
      d.peak[g] = START_CHIPS;
    }
    return d;
  }

  function clean(raw) {
    const out = blank();
    if (!raw || typeof raw !== "object") return out;
    out.chips = toInt(raw.chips, START_CHIPS, 0);
    for (const [field, def] of [["ledger", 0], ["peak", START_CHIPS]]) {
      const src = raw[field] && typeof raw[field] === "object" ? raw[field] : {};
      for (const g of GAMES) out[field][g] = toInt(src[g], def);
    }
    out.escrow = toInt(raw.escrow, 0, 0);
    out.refills = toInt(raw.refills, 0, 0);
    out.migrated = raw.migrated === true;
    if (raw.extra && typeof raw.extra === "object") out.extra = raw.extra;
    return out;
  }

  function migrate() {
    const d = blank();
    const chips = [];
    for (const g of ["blackjack", "poker"]) {
      const old = PG.store.get("mem." + g, null) || {};
      const c = toInt(old.chips, null, 0);
      if (c !== null) chips.push(c);
      const hs = PG.highscore ? PG.highscore.get(g) || 0 : 0;
      d.peak[g] = Math.max(START_CHIPS, toInt(old.best, 0), toInt(hs, 0));
    }
    d.peak.casino = Math.max(START_CHIPS, toInt(PG.highscore ? PG.highscore.get("casino") || 0 : 0, 0));
    d.chips = chips.length ? Math.max(...chips) : START_CHIPS;
    d.migrated = true;
    return d;
  }

  function save() {
    if (data) PG.store.set(KEY, data);
  }

  const bank = {
    START_CHIPS,
    MIN_BET,

    /** Konto frisch laden (Migration + escrow-Erstattung). */
    load() {
      const raw = PG.store.get(KEY, null);
      let changed = false;
      if (raw && raw.migrated === true) {
        data = clean(raw);
        changed = JSON.stringify(data) !== JSON.stringify(raw);
      } else {
        data = migrate();
        changed = true;
      }
      if (data.escrow > 0) {
        data.chips += data.escrow;
        data.escrow = 0;
        changed = true;
      }
      if (changed) save();
      return data;
    },
    _d() {
      return data || this.load();
    },
    balance() {
      return this._d().chips;
    },
    escrow() {
      return this._d().escrow;
    },
    ledger(game) {
      return this._d().ledger[game] || 0;
    },
    valueFor(game) {
      return START_CHIPS + this.ledger(game);
    },
    scoreFor(game) {
      const d = this._d();
      return Math.max(d.peak[game] || START_CHIPS, this.valueFor(game));
    },
    refills() {
      return this._d().refills;
    },
    minBet(game, mode) {
      if (mode != null && MIN_BET[game + ":" + mode] !== undefined) return MIN_BET[game + ":" + mode];
      return MIN_BET[game] !== undefined ? MIN_BET[game] : 1;
    },
    isBroke(game, mode) {
      return this.balance() < this.minBet(game, mode);
    },
    getExtra(key, def) {
      const v = this._d().extra[key];
      return v === undefined ? def : v;
    },
    /** Einsatz abbuchen + speichern. false = nicht gedeckt. */
    debit(n, game) {
      const d = this._d();
      n = Math.trunc(n);
      if (n <= 0) return true;
      if (n > d.chips) return false;
      d.chips -= n;
      d.ledger[game] = (d.ledger[game] || 0) - n;
      save();
      return true;
    },
    /** Gewinn gutschreiben (Bilanz + Höchststand). */
    credit(n, game, doSave = true) {
      const d = this._d();
      n = Math.trunc(n);
      if (n <= 0) return;
      d.chips += n;
      d.ledger[game] = (d.ledger[game] || 0) + n;
      d.peak[game] = Math.max(d.peak[game] || START_CHIPS, START_CHIPS + d.ledger[game]);
      if (doSave) save();
    },
    /** Bank-Kredit bei Pleite: Konto zurück auf 1000. */
    refillIfBroke(game, mode) {
      const d = this._d();
      if (d.chips >= this.minBet(game, mode)) return false;
      d.chips = START_CHIPS;
      d.refills += 1;
      save();
      return true;
    },
    setEscrow(n) {
      const d = this._d();
      n = Math.max(0, Math.min(Math.trunc(n), d.chips));
      d.chips -= n;
      d.escrow += n;
      save();
      return n;
    },
    payFromEscrow(n, game = "poker") {
      const d = this._d();
      n = Math.max(0, Math.min(Math.trunc(n), d.escrow));
      if (n <= 0) return 0;
      d.escrow -= n;
      d.ledger[game] = (d.ledger[game] || 0) - n;
      save();
      return n;
    },
    clearEscrow(doSave = true) {
      const d = this._d();
      if (d.escrow <= 0) return 0;
      const n = d.escrow;
      d.chips += n;
      d.escrow = 0;
      if (doSave) save();
      return n;
    },
    settleEscrow(win, game = "poker") {
      this.clearEscrow(false);
      if (win > 0) this.credit(win, game, false);
      save();
    },
    setExtra(key, value) {
      this._d().extra[key] = value;
      save();
    },
  };

  PG.lamabank = bank;
})();
