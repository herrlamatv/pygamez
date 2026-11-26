/*
 * matches.js - Match-Historie (Port von matches.py)
 * ============================================================================
 * "Letzte Matches" und "Alle Matches" je Spiel. Gespeichert wird über PG.store
 * unter dem Schlüssel "matches" - also automatisch je Profil
 * (pygamez.<profil>.matches). Inhalt (kompaktes JSON)::
 *
 *   {"format": "pygamez.matches", "v": 1, "app": "PyGameZ",
 *    "updated": "2026-11-26 14:03",
 *    "games": {
 *      "crossy": {"n": 57, "w": 0, "l": 0, "d": 0, "best": 212, "sum": 3120,
 *                 "time": 4210, "first": "2026-11-20 18:02",
 *                 "list": [["2026-11-26 14:03", "daily", 57, null, 41, 4,
 *                           {"day": 329, "c": 3, "ch": "chicken"}], ...]}}}
 *
 * Die Einträge je Spiel sind exakt die der Desktop-Datei matches.pgzlamamatches
 * (dort liegen die Spiele unter "profiles" -> <profil>):
 *
 *   [zeit "YYYY-MM-DD HH:MM", modus, punkte, ergebnis, dauer_s, flags, info]
 *
 * ergebnis: 1 Sieg, 0 Niederlage, 2 Remis, null unbekannt; flags: F_*;
 * info: kleines Objekt (nur Ganzzahlen/Text/bool) oder null.
 * Die Liste je Spiel ist auf CAP Einträge gedeckelt, die Summen laufen weiter.
 *
 * Aufgerufen wird record() zentral aus app.js (Game Over / Verlassen) bzw. aus
 * Game.logMatch() für Spiele, deren Runden ohne Game Over enden (Chip-Spiele,
 * Geometry Dash, Labyrinth). Geschrieben wird gedrosselt (maybeFlush) und
 * sofort beim Verlassen/Profilwechsel.
 */
(function () {
  "use strict";

  const PG = window.PG;

  const FORMAT = "pygamez.matches";
  const VERSION = 1;
  const KEY = "matches";

  // Gespeicherte Matches je Spiel (wie die Desktop-Fassung; pygamez.com setzt
  // hier per Patch eine eigene, im Dashboard einstellbare Zahl ein).
  const CAP = 250;

  // Ergebnis eines Matches
  const WIN = 1, LOSS = 0, DRAW = 2;

  // Flags (Bitmaske)
  const F_RECORD = 1; // neuer Highscore
  const F_ABORT = 2; // verlassen, bevor die Runde zu Ende war
  const F_DAILY = 4; // Tagesmodus (Tagesstrecke, Tagesrätsel, Tageswort)
  const F_ARCHIVE = 8; // vergangener Tag aus dem Tages-Archiv

  // Bekannte Info-Schlüssel (Anzeige-Reihenfolge) -> Text-Schlüssel
  const INFO_KEYS = [
    ["day", "match.info.day"], // Tag (dayIndex) des Tagesmodus
    ["bet", "match.info.bet"], // Einsatz (Chips)
    ["win", "match.info.win"], // Auszahlung (Chips)
    ["num", "match.info.num"], // gezogene Zahl (Roulette)
    ["cr", "match.info.cr"], // Absturzpunkt x100 (Lama Crash)
    ["mi", "match.info.mi"], // Minen (Lama Mines)
    ["op", "match.info.op"], // aufgedeckte Felder (Lama Mines)
    ["rw", "match.info.rw"], // Reihen (Lama Plinko)
    ["x", "match.info.x"], // Multiplikator x100
    ["c", "match.info.c"], // gesammelte Münzen
    ["ch", "match.info.ch"], // Figur
    ["vr", "match.info.vr"], // Variante (Sudoku)
    ["lv", "match.info.lv"], // Level / Bahn / Stufe / Schwierigkeit
    ["ln", "match.info.ln"], // erreichte Spur (Lama Crossing)
    ["st", "match.info.st"], // Sterne
    ["tm", "match.info.tm"], // Zeit in Sekunden (Rätsel)
    ["g", "match.info.g"], // Versuche (Wordle)
    ["wd", "match.info.wd"], // gelöste Wörter (Wordle Endlos)
    ["len", "match.info.len"], // Wortlänge (Wordle)
    ["vs", "match.info.vs"], // Gegner (Profilname / KI)
  ];

  // Schreib-Drosselung (Plinko & Co. können viele Runden je Sekunde liefern).
  const FLUSH_MS = 5000;

  // Kurze Runden, die man verlässt, ohne etwas erreicht zu haben, zählen nicht.
  const MIN_ABORT_SECONDS = 5;

  const TOTALS = [["n", 0], ["w", 0], ["l", 0], ["d", 0], ["best", null],
    ["sum", 0], ["time", 0], ["first", ""]];
  const MAX_INT = Number.MAX_SAFE_INTEGER; // 2**53 - 1 wie in Python

  function pad(n) {
    return String(n).padStart(2, "0");
  }

  function nowStr() {
    const d = new Date();
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  /** Text auf n Zeichen kürzen (Codepunkte wie Python-Slicing). */
  function cut(s, n) {
    const chars = Array.from(s);
    return chars.length > n ? chars.slice(0, n).join("") : s;
  }

  // ---------------------------------------------------------------- Bereinigen
  function isInt(v) {
    return typeof v === "number" && Number.isInteger(v);
  }

  function clampInt(v, lo = -MAX_INT, hi = MAX_INT, def = 0) {
    if (!isInt(v)) return def;
    return Math.max(lo, Math.min(hi, v));
  }

  /** Nur flache Werte (int, str bis 40 Zeichen, bool) - sonst null. */
  function cleanInfo(raw) {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
    const out = {};
    let any = false;
    for (const key of Object.keys(raw)) {
      if (Array.from(key).length > 8) continue;
      const value = raw[key];
      if (typeof value === "boolean") out[key] = value;
      else if (isInt(value)) out[key] = clampInt(value);
      else if (typeof value === "string") out[key] = cut(value, 40);
      else continue;
      any = true;
    }
    return any ? out : null;
  }

  function cleanEntry(raw) {
    if (!Array.isArray(raw) || raw.length < 6) return null;
    const [stamp, mode, score, result0, dur, flags] = raw;
    const info = raw.length > 6 ? raw[6] : null;
    if (typeof stamp !== "string") return null;
    const result = result0 === WIN || result0 === LOSS || result0 === DRAW ? result0 : null;
    return [cut(stamp, 16), typeof mode === "string" ? cut(mode, 24) : "",
      clampInt(score), result, clampInt(dur, 0),
      clampInt(flags, 0, 255), cleanInfo(info)];
  }

  function blankGame() {
    const g = {};
    for (const [k, d] of TOTALS) g[k] = d;
    g.list = [];
    return g;
  }

  function cleanGame(raw) {
    const game = blankGame();
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) return game;
    for (const [key, def] of TOTALS) {
      const value = key in raw ? raw[key] : def;
      if (key === "first") game[key] = typeof value === "string" ? cut(value, 16) : "";
      else if (key === "best") game[key] = isInt(value) ? clampInt(value) : null;
      else game[key] = clampInt(value, 0);
    }
    if (Array.isArray(raw.list)) {
      for (const item of raw.list) {
        const entry = cleanEntry(item);
        if (entry !== null) game.list.push(entry);
      }
      game.list = game.list.slice(-CAP);
    }
    return game;
  }

  function blankDoc() {
    return { format: FORMAT, v: VERSION, app: "PyGameZ", updated: nowStr(), games: {} };
  }

  function rawKeyFor(pid) {
    return PG.store.prefix + (pid ? pid + "." : "") + KEY;
  }

  function readFor(pid) {
    if (!pid || pid === PG.store.profile) return PG.store.get(KEY, null);
    try {
      const raw = window.localStorage.getItem(rawKeyFor(pid));
      return raw == null ? null : JSON.parse(raw);
    } catch (e) {
      return null;
    }
  }

  function writeFor(pid, data) {
    if (!pid || pid === PG.store.profile) {
      PG.store.set(KEY, data);
      return;
    }
    try {
      window.localStorage.setItem(rawKeyFor(pid), JSON.stringify(data));
    } catch (e) {
      /* Speicher voll - dann eben ohne */
    }
  }

  function parseDoc(raw) {
    const doc = blankDoc();
    if (raw && typeof raw === "object" && raw.format === FORMAT && raw.games
        && typeof raw.games === "object" && !Array.isArray(raw.games)) {
      for (const k of Object.keys(raw.games)) doc.games[String(k)] = cleanGame(raw.games[k]);
    }
    return doc;
  }

  // ------------------------------------------------------------------ Modul
  const matches = {
    FORMAT, VERSION, CAP, WIN, LOSS, DRAW,
    F_RECORD, F_ABORT, F_DAILY, F_ARCHIVE,
    INFO_KEYS, MIN_ABORT_SECONDS, FLUSH_MS,
    _doc: null,
    _pid: "",
    _dirty: false,
    _last: -Infinity,

    /** Historie des aktiven Profils (einmalig geladen; fehlt sie, ist sie leer). */
    _load() {
      const pid = PG.store.profile || "";
      if (this._doc && this._pid === pid) return this._doc;
      if (this._doc && this._dirty) this.flush();
      this._doc = parseDoc(readFor(pid));
      this._pid = pid;
      this._dirty = false;
      return this._doc;
    },

    /** Daten eines anderen Profils (nur lesen). */
    _docFor(pid) {
      if (!pid || pid === (PG.store.profile || "")) return this._load();
      return parseDoc(readFor(pid));
    },

    _game(key, pid, create = true) {
      if (pid && pid !== (PG.store.profile || "")) {
        // Fremdes Profil: lesen, ändern und sofort zurückschreiben.
        const doc = parseDoc(readFor(pid));
        let game = doc.games[key];
        if (!game && create) game = doc.games[key] = blankGame();
        return [game || null, doc, pid];
      }
      const doc = this._load();
      let game = doc.games[key];
      if (!game && create) game = doc.games[key] = blankGame();
      return [game || null, doc, null];
    },

    // ----- Schreiben --------------------------------------------------------
    _serialize(doc) {
      const out = { format: FORMAT, v: VERSION, app: "PyGameZ", updated: nowStr(), games: {} };
      for (const key of Object.keys(doc.games)) {
        const game = doc.games[key];
        if (game && game.n) out.games[key] = game;
      }
      return out;
    },

    /** Schreibt ungespeicherte Matches sofort. */
    flush() {
      if (!this._dirty || !this._doc) return;
      writeFor(this._pid, this._serialize(this._doc));
      this._dirty = false;
      this._last = performance.now();
    },

    /** Gedrosselt schreiben (höchstens alle FLUSH_MS Millisekunden). */
    maybeFlush() {
      if (this._dirty && performance.now() - this._last >= FLUSH_MS) this.flush();
    },

    /** Profilwechsel: Offenes wegschreiben und den Zwischenspeicher leeren. */
    resetCache() {
      this.flush();
      this._doc = null;
      this._pid = "";
      this._dirty = false;
    },

    // ----- Aufzeichnen ------------------------------------------------------
    /**
     * Hält ein Match fest (neueste zuletzt). opts: {pid, replace}.
     * replace = true ersetzt das zuletzt gespeicherte Match dieses Spiels
     * (Rückgängig nach Game Over: dieselbe Partie lief weiter).
     * Gibt den gespeicherten Eintrag zurück.
     */
    record(key, mode = "", score = 0, result = null, duration = 0, flags = 0, info = null, opts) {
      const o = opts || {};
      const [game, doc, foreign] = this._game(String(key), o.pid);
      const sc = isInt(score) ? score : Math.trunc(Number(score) || 0);
      const dur = Math.trunc(Math.max(0, Number(duration) || 0));
      const entry = cleanEntry([nowStr(), mode || "", sc, result, dur, flags, info]);
      if (o.replace && game.list.length) {
        // Dieselbe Partie lief weiter: altes Match raus, Spielzeit addieren.
        const old = game.list.pop();
        subTotals(game, old);
        entry[4] = clampInt(entry[4] + old[4], 0);
      }
      addTotals(game, entry);
      game.list.push(entry);
      if (game.list.length > CAP) game.list.splice(0, game.list.length - CAP);
      if (foreign) writeFor(foreign, this._serialize(doc));
      else this._dirty = true;
      return entry;
    },

    // ----- Abfragen (für den Matches-Screen) ---------------------------------
    /** Alle gespeicherten Matches eines Spiels, NEUESTE ZUERST (Kopien). */
    entries(key, pid) {
      const game = this._docFor(pid).games[String(key)];
      if (!game) return [];
      return game.list.slice().reverse().map((e) => e.slice());
    },

    /** Die letzten n Matches, neueste zuerst. */
    recent(key, n = 10, pid) {
      return this.entries(key, pid).slice(0, n);
    },

    /** Summen eines Spiels: n, w, l, d, best, sum, time, first, stored. */
    totals(key, pid) {
      const game = this._docFor(pid).games[String(key)];
      const out = {};
      for (const [k, d] of TOTALS) out[k] = d;
      out.stored = 0;
      if (game) {
        for (const [k] of TOTALS) out[k] = game[k];
        out.stored = game.list.length;
      }
      return out;
    },

    /** Modi, die in den gespeicherten Matches vorkommen (erste Sichtung zuerst). */
    modesOf(key, pid) {
      const seen = [];
      const game = this._docFor(pid).games[String(key)];
      for (const e of (game && game.list) || []) if (seen.indexOf(e[1]) < 0) seen.push(e[1]);
      return seen;
    },

    /** Historie eines gelöschten Profils entfernen. */
    dropProfile(pid) {
      if (this._doc && this._pid === pid) {
        this._doc = null;
        this._pid = "";
        this._dirty = false;
      }
      try {
        window.localStorage.removeItem(rawKeyFor(pid));
      } catch (e) {
        /* nichts zu löschen */
      }
    },

    /** Historie eines Spiels leeren. */
    clearGame(key) {
      const doc = this._load();
      if (doc.games[String(key)] !== undefined) {
        delete doc.games[String(key)];
        this._dirty = true;
        this.flush();
      }
    },

    /** Sekunden -> "1:05:09" / "3:07" / "0:42". */
    formatDuration(seconds) {
      const s = Math.max(0, Math.trunc(Number(seconds) || 0));
      const h = Math.floor(s / 3600);
      const m = Math.floor((s % 3600) / 60);
      const r = s % 60;
      if (h) return `${h}:${pad(m)}:${pad(r)}`;
      return `${m}:${pad(r)}`;
    },

    // Für Prüfstände (matches_check.js): Bereinigung von außen aufrufbar.
    _cleanEntry: cleanEntry,
    _cleanGame: cleanGame,
  };

  function addTotals(game, entry) {
    const [, , score, result, dur] = entry;
    game.n = Math.min(MAX_INT, game.n + 1);
    if (result === WIN) game.w += 1;
    else if (result === LOSS) game.l += 1;
    else if (result === DRAW) game.d += 1;
    if (game.best === null || score > game.best) game.best = score;
    game.sum = clampInt(game.sum + score);
    game.time = clampInt(game.time + dur, 0);
    if (!game.first) game.first = entry[0];
  }

  /** Nimmt ein Match aus den Summen (der Bestwert bleibt - er war erreicht). */
  function subTotals(game, entry) {
    const [, , score, result, dur] = entry;
    game.n = Math.max(0, game.n - 1);
    if (result === WIN) game.w = Math.max(0, game.w - 1);
    else if (result === LOSS) game.l = Math.max(0, game.l - 1);
    else if (result === DRAW) game.d = Math.max(0, game.d - 1);
    game.sum = clampInt(game.sum - score);
    game.time = Math.max(0, game.time - dur);
  }

  PG.matches = matches;
  if (PG.profiles) PG.profiles.onSwitch(() => matches.resetCache());
})();
