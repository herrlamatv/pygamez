/**
 * stats.js - Statistik der Web-Version (Port von stats.py).
 *
 * Gleiches Datenmodell wie die Desktop-Datei `stats.pgzlamastats`: je Profil
 * Summen pro Spiel, Spieltage, Tageszeit-Profil, Punkteverlauf, Ereignis-
 * Verlauf mit Zeitstempeln, Duell-Bilanzen, Serien, Erfolgspunkte und Level.
 * Gespeichert wird im localStorage (Schlüssel `stats` des aktiven Profils);
 * Export/Import erzeugen bzw. lesen dieselbe `.pgzlamastats`-Datei wie die
 * Desktop-Version - Statistiken lassen sich also hin und her tragen.
 *
 * Der alte Web-Stand ({played, wins, losses, time, records} plus `_events`)
 * wird beim ersten Laden automatisch übernommen.
 */
(function () {
  "use strict";

  const FORMAT = "pygamez.stats";
  const VERSION = 1;
  const EXT = ".pgzlamastats";
  const FLUSH_MS = 5000;

  const MAX_EVENTS = 4000;
  const MAX_SERIES = 250;
  const MAX_DAYS = 1100;

  const FIELDS = { plays: 0, time: 0, wins: 0, losses: 0, records: 0,
    best: 0, best_at: "", first: "", last: "", longest: 0 };

  function pad(n) {
    return String(n).padStart(2, "0");
  }

  function nowStr() {
    const d = new Date();
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  function todayStr() {
    const d = new Date();
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  }

  function num(v, def) {
    const x = Number(v);
    return Number.isFinite(x) && x >= 0 ? x : (def || 0);
  }

  function blankGame() {
    return Object.assign({}, FIELDS);
  }

  function blankProfile() {
    return { games: {}, days: {}, hours: new Array(24).fill(0), series: {},
      events: [], duel: {},
      streak: { cur: 0, best: 0, first_day: "", last_day: "" },
      points: 0, level: 1, longest: 0 };
  }

  function cleanGame(raw) {
    const g = blankGame();
    if (!raw || typeof raw !== "object") return g;
    for (const f of Object.keys(FIELDS)) {
      const def = FIELDS[f];
      g[f] = typeof def === "string" ? String(raw[f] == null ? def : raw[f]) : num(raw[f], def);
    }
    return g;
  }

  function cleanProfile(raw) {
    const p = blankProfile();
    if (!raw || typeof raw !== "object") return p;
    // Alter Web-Stand: flaches {key: {played, wins, ...}} plus _events.
    const legacyKeys = Object.keys(raw).filter((k) => k !== "_events" && raw[k] && typeof raw[k] === "object" && "played" in raw[k]);
    if (legacyKeys.length) {
      for (const k of legacyKeys) {
        const old = raw[k];
        const g = blankGame();
        g.plays = num(old.played);
        g.time = num(old.time);
        g.wins = num(old.wins);
        g.losses = num(old.losses);
        g.records = num(old.records);
        p.games[k] = g;
      }
      if (raw._events && typeof raw._events === "object") p.legacyEvents = Object.assign({}, raw._events);
      return p;
    }
    if (raw.games && typeof raw.games === "object") {
      for (const k of Object.keys(raw.games)) p.games[k] = cleanGame(raw.games[k]);
    }
    if (raw.days && typeof raw.days === "object") {
      for (const day of Object.keys(raw.days)) {
        const e = raw.days[day] || {};
        const slot = { t: num(e.t), p: num(e.p), w: num(e.w), a: num(e.a), g: {} };
        if (e.g && typeof e.g === "object") {
          for (const k of Object.keys(e.g)) slot.g[k] = num(e.g[k]);
        }
        p.days[String(day).slice(0, 10)] = slot;
      }
    }
    if (Array.isArray(raw.hours)) {
      for (let i = 0; i < 24 && i < raw.hours.length; i++) p.hours[i] = num(raw.hours[i]);
    }
    if (raw.series && typeof raw.series === "object") {
      for (const k of Object.keys(raw.series)) {
        const list = Array.isArray(raw.series[k]) ? raw.series[k] : [];
        const clean = [];
        for (const item of list) {
          if (Array.isArray(item) && item.length >= 2 && typeof item[0] === "string") {
            clean.push([item[0].slice(0, 16), Math.round(num(item[1])), item[2] ? 1 : 0]);
          }
        }
        if (clean.length) p.series[k] = clean.slice(-MAX_SERIES);
      }
    }
    if (Array.isArray(raw.events)) {
      p.events = raw.events.filter((e) => e && typeof e.t === "string" && typeof e.k === "string").slice(-MAX_EVENTS);
    }
    if (raw.duel && typeof raw.duel === "object") {
      for (const other of Object.keys(raw.duel)) {
        const games = raw.duel[other] || {};
        const slot = {};
        for (const k of Object.keys(games)) {
          slot[k] = { w: num(games[k].w), l: num(games[k].l) };
        }
        if (Object.keys(slot).length) p.duel[other] = slot;
      }
    }
    if (raw.streak && typeof raw.streak === "object") {
      p.streak = { cur: num(raw.streak.cur), best: num(raw.streak.best),
        first_day: String(raw.streak.first_day || "").slice(0, 10),
        last_day: String(raw.streak.last_day || "").slice(0, 10) };
    }
    p.points = Math.round(num(raw.points));
    p.level = Math.max(1, Math.round(num(raw.level, 1)));
    p.longest = num(raw.longest);
    if (raw.legacyEvents && typeof raw.legacyEvents === "object") p.legacyEvents = Object.assign({}, raw.legacyEvents);
    return p;
  }

  function rawKeyFor(pid) {
    return PG.store.prefix + pid + ".stats";
  }

  function readFor(pid) {
    if (!pid || pid === PG.store.profile) return PG.store.get("stats", null);
    try {
      const raw = window.localStorage.getItem(rawKeyFor(pid));
      return raw == null ? null : JSON.parse(raw);
    } catch (e) {
      return null;
    }
  }

  function writeFor(pid, data) {
    if (!pid || pid === PG.store.profile) {
      PG.store.set("stats", data);
      return;
    }
    try {
      window.localStorage.setItem(rawKeyFor(pid), JSON.stringify(data));
    } catch (e) {
      /* Speicher voll - dann eben ohne */
    }
  }

  function dayEntry(prof, day) {
    let slot = prof.days[day];
    if (!slot) {
      slot = { t: 0, p: 0, w: 0, a: 0, g: {} };
      prof.days[day] = slot;
      const keys = Object.keys(prof.days);
      if (keys.length > MAX_DAYS) {
        keys.sort();
        for (const old of keys.slice(0, keys.length - MAX_DAYS)) delete prof.days[old];
      }
    }
    return slot;
  }

  function recalcStreak(prof) {
    const days = Object.keys(prof.days).filter((d) => d.length === 10).sort();
    if (!days.length) {
      prof.streak = { cur: 0, best: 0, first_day: "", last_day: "" };
      return;
    }
    const DAY = 86400000;
    let best = 1;
    let cur = 1;
    let prev = null;
    for (const day of days) {
      const date = new Date(day + "T00:00:00");
      if (prev) cur = Math.round((date - prev) / DAY) === 1 ? cur + 1 : 1;
      prev = date;
      if (cur > best) best = cur;
    }
    const today = new Date(todayStr() + "T00:00:00");
    if (prev && Math.round((today - prev) / DAY) > 1) cur = 0;
    prof.streak = { cur, best, first_day: days[0], last_day: days[days.length - 1] };
  }

  function addEvent(prof, ev) {
    prof.events.push(ev);
    if (prof.events.length > MAX_EVENTS) prof.events.splice(0, prof.events.length - MAX_EVENTS);
  }

  const stats = {
    EXT,
    FORMAT,
    _cur: null,
    _curId: "",
    _dirty: false,
    _last: 0,
    _dayKey: "",
    _sessKey: null,
    _sessTime: 0,

    /** Profil-Datensatz (Standard: aktives Profil). */
    prof(pid) {
      const id = pid || PG.store.profile || "p1";
      if (!pid || id === PG.store.profile) {
        if (!this._cur || this._curId !== id) {
          this._cur = cleanProfile(readFor(id));
          this._curId = id;
          this._dayKey = "";
        }
        return this._cur;
      }
      return cleanProfile(readFor(id));
    },

    resetCache() {
      this.endSession();
      this._cur = null;
      this._curId = "";
      this._dayKey = "";
      this._sessKey = null;
      this._sessTime = 0;
    },

    today() {
      const prof = this.prof();
      const day = todayStr();
      if (this._dayKey !== day) this._dayKey = day;
      return dayEntry(prof, day);
    },

    entry(key) {
      const prof = this.prof();
      if (!prof.games[key]) prof.games[key] = blankGame();
      return prof.games[key];
    },

    // ----- Ereignisse ------------------------------------------------------
    gameStarted(key) {
      this.endSession();
      const e = this.entry(key);
      e.plays++;
      e.last = nowStr();
      if (!e.first) e.first = e.last;
      this.today().p++;
      recalcStreak(this.prof());
      this._dirty = true;
      this.flush();
    },

    addPlaytime(key, dt) {
      if (!(dt > 0)) return;
      if (this._sessKey !== key) {
        this.endSession();
        this._sessKey = key;
      }
      this._sessTime += dt;
      this.entry(key).time += dt;
      const slot = this.today();
      slot.t += dt;
      slot.g[key] = (slot.g[key] || 0) + dt;
      this.prof().hours[new Date().getHours()] += dt;
      this._dirty = true;
    },

    endSession() {
      if (this._sessKey == null || this._sessTime <= 0) {
        this._sessKey = null;
        this._sessTime = 0;
        return 0;
      }
      const prof = this.prof();
      const e = prof.games[this._sessKey];
      if (e && this._sessTime > e.longest) e.longest = this._sessTime;
      if (this._sessTime > prof.longest) prof.longest = this._sessTime;
      const len = this._sessTime;
      this._sessKey = null;
      this._sessTime = 0;
      this._dirty = true;
      return len;
    },

    sessionTime() {
      return this._sessTime;
    },

    recordResult(key, won) {
      const e = this.entry(key);
      if (won) {
        e.wins++;
        this.today().w++;
      } else e.losses++;
      this._dirty = true;
      this.flush();
    },

    recordBroken(key) {
      this.entry(key).records++;
      this._dirty = true;
    },

    scoreResult(key, score, record) {
      score = Math.round(Number(score) || 0);
      const prof = this.prof();
      const e = this.entry(key);
      const st = nowStr();
      if (score > e.best) {
        e.best = score;
        e.best_at = st;
      }
      if (score > 0 || record) {
        const series = prof.series[key] || (prof.series[key] = []);
        series.push([st, score, record ? 1 : 0]);
        if (series.length > MAX_SERIES) {
          const keep = series.filter((p) => p[2]);
          const rest = series.filter((p) => !p[2]).slice(-(MAX_SERIES - keep.length));
          prof.series[key] = keep.concat(rest).sort((a, b) => (a[0] < b[0] ? -1 : 1));
        }
      }
      if (record) addEvent(prof, { t: st, k: "rec", g: key, v: score });
      this._dirty = true;
    },

    recordDuel(key, opponent, won, pid) {
      const prof = this.prof(pid);
      const slot = prof.duel[opponent] || (prof.duel[opponent] = {});
      const wl = slot[key] || (slot[key] = { w: 0, l: 0 });
      wl[won ? "w" : "l"]++;
      addEvent(prof, { t: nowStr(), k: "duel", g: key, o: String(opponent), w: won ? 1 : 0 });
      if (!pid || pid === PG.store.profile) {
        this._dirty = true;
        this.flush();
      } else writeFor(pid, prof);
    },

    duelTotal(pid) {
      const prof = this.prof(pid);
      let n = 0;
      for (const other of Object.keys(prof.duel)) {
        for (const k of Object.keys(prof.duel[other])) {
          n += prof.duel[other][k].w + prof.duel[other][k].l;
        }
      }
      return n;
    },

    /** Erfolgs-Ereignis der Spiele (game.js achEvent) -> Erfolgs-Logik. */
    event(id, value) {
      if (PG.ach) PG.ach.event(id, value);
    },

    logAch(id, points, stampStr) {
      const prof = this.prof();
      const st = stampStr || nowStr();
      addEvent(prof, { t: st, k: "ach", id: String(id), p: Math.round(points) || 0 });
      dayEntry(prof, st.slice(0, 10)).a++;
      this._dirty = true;
    },

    logLevel(level, stampStr) {
      const prof = this.prof();
      addEvent(prof, { t: stampStr || nowStr(), k: "lvl", v: Math.round(level) });
      this._dirty = true;
      this.flush();
    },

    setPoints(points, pid) {
      const prof = this.prof(pid);
      prof.points = Math.max(0, Math.round(points) || 0);
      prof.level = PG.ach.levelFor(prof.points);
      if (!pid || pid === PG.store.profile) this._dirty = true;
      else writeFor(pid, prof);
    },

    pointsOf(pid) {
      return this.prof(pid).points;
    },

    levelOf(pid) {
      const prof = this.prof(pid);
      return [prof.level, prof.points];
    },

    // ----- Abfragen --------------------------------------------------------
    get(key, pid) {
      const prof = this.prof(pid);
      return Object.assign(blankGame(), prof.games[key] || {});
    },

    allStats(pid) {
      const prof = this.prof(pid);
      const out = {};
      for (const k of Object.keys(prof.games)) out[k] = Object.assign({}, prof.games[k]);
      return out;
    },

    totals(pid) {
      const prof = this.prof(pid);
      const t = { plays: 0, time: 0, wins: 0, losses: 0, records: 0, distinct: 0,
        win_games: 0, favorite: null };
      let bestTime = 0;
      for (const key of Object.keys(prof.games)) {
        const e = prof.games[key];
        t.plays += e.plays;
        t.time += e.time;
        t.wins += e.wins;
        t.losses += e.losses;
        t.records += e.records;
        if (e.plays > 0 || e.time > 0) t.distinct++;
        if (e.wins > 0) t.win_games++;
        if (e.time > bestTime) {
          bestTime = e.time;
          t.favorite = key;
        }
      }
      t.days = Object.keys(prof.days).filter((d) => prof.days[d].t > 0 || prof.days[d].p > 0).length;
      t.streak = prof.streak.cur;
      t.best_streak = prof.streak.best;
      t.duels = this.duelTotal(pid);
      t.points = prof.points;
      t.level = prof.level;
      t.longest = prof.longest;
      return t;
    },

    daysMap(pid) {
      const prof = this.prof(pid);
      const out = {};
      for (const d of Object.keys(prof.days)) out[d] = Object.assign({}, prof.days[d]);
      return out;
    },

    hoursList(pid) {
      return this.prof(pid).hours.slice();
    },

    weekdayList(pid) {
      const prof = this.prof(pid);
      const out = new Array(7).fill(0);
      for (const day of Object.keys(prof.days)) {
        const date = new Date(day + "T00:00:00");
        if (isNaN(date)) continue;
        out[(date.getDay() + 6) % 7] += prof.days[day].t;
      }
      return out;
    },

    events(opts) {
      const o = opts || {};
      let items = this.prof(o.pid).events.slice();
      if (o.kinds) items = items.filter((e) => o.kinds.indexOf(e.k) >= 0);
      items.reverse();
      return o.limit ? items.slice(0, o.limit) : items;
    },

    seriesOf(key, pid) {
      const prof = this.prof(pid);
      return (prof.series[key] || []).map((p) => p.slice());
    },

    playedGames(pid) {
      const prof = this.prof(pid);
      return Object.keys(prof.series).sort((a, b) => prof.series[b].length - prof.series[a].length);
    },

    streak(pid) {
      return Object.assign({}, this.prof(pid).streak);
    },

    duelTable(pid) {
      const prof = this.prof(pid);
      const out = {};
      for (const other of Object.keys(prof.duel)) {
        out[other] = {};
        for (const k of Object.keys(prof.duel[other])) out[other][k] = Object.assign({}, prof.duel[other][k]);
      }
      return out;
    },

    /** Legacy-Erfolgs-Ereignisse aus dem alten Web-Stand (einmalig). */
    legacyEvents() {
      return this.prof().legacyEvents || null;
    },

    clearLegacyEvents() {
      const prof = this.prof();
      if (prof.legacyEvents) {
        delete prof.legacyEvents;
        this._dirty = true;
      }
    },

    dropProfile(pid) {
      try {
        window.localStorage.removeItem(rawKeyFor(pid));
      } catch (e) {
        /* nichts zu tun */
      }
      if (pid === this._curId) this.resetCache();
    },

    formatTime(seconds) {
      const s = Math.max(0, Math.floor(seconds || 0));
      const h = Math.floor(s / 3600);
      const m = Math.floor((s % 3600) / 60);
      const r = s % 60;
      if (h) return `${h}h ${pad(m)}m`;
      if (m) return `${m}m ${pad(r)}s`;
      return `${r}s`;
    },

    // ----- Speichern -------------------------------------------------------
    maybeFlush() {
      if (this._dirty && performance.now() - this._last > FLUSH_MS) this.flush();
    },

    flush() {
      if (!this._dirty || !this._cur) return;
      this._last = performance.now();
      this._dirty = false;
      writeFor(this._curId, this._cur);
    },

    // ----- Teilen ----------------------------------------------------------
    /** Baut den Inhalt einer .pgzlamastats-Datei (alle Profile des Geräts). */
    exportText() {
      this.flush();
      const doc = { format: FORMAT, v: VERSION, app: "PyGameZ", updated: nowStr(), profiles: {} };
      for (const p of PG.profiles.all()) {
        const raw = readFor(p.id);
        if (raw) doc.profiles[p.id] = cleanProfile(raw);
      }
      return JSON.stringify(doc, null, 2);
    },

    /** Liest eine .pgzlamastats-Datei. Gibt [ok, grund] zurück. */
    importText(text, opts) {
      const o = opts || {};
      let raw;
      try {
        raw = JSON.parse(text);
      } catch (e) {
        return [false, "io"];
      }
      if (!raw || raw.format !== FORMAT || !raw.profiles || typeof raw.profiles !== "object") {
        return [false, "format"];
      }
      const merge = o.merge !== false;
      for (const pid of Object.keys(raw.profiles)) {
        const src = cleanProfile(raw.profiles[pid]);
        const dest = o.target || pid;
        const known = PG.profiles.all().some((p) => p.id === dest);
        if (!known && !o.target) continue; // fremdes Profil: nur mit Ziel übernehmen
        const cur = merge ? cleanProfile(readFor(dest)) : blankProfile();
        mergeProfile(cur, src);
        if (dest === PG.store.profile) {
          this._cur = cur;
          this._curId = dest;
          this._dirty = true;
          this.flush();
        } else writeFor(dest, cur);
      }
      return [true, ""];
    },
  };

  function mergeProfile(dest, src) {
    for (const key of Object.keys(src.games)) {
      const e = src.games[key];
      const slot = dest.games[key] || (dest.games[key] = blankGame());
      slot.plays += e.plays;
      slot.wins += e.wins;
      slot.losses += e.losses;
      slot.records += e.records;
      slot.time += e.time;
      slot.longest = Math.max(slot.longest, e.longest);
      if (e.best > slot.best) {
        slot.best = e.best;
        slot.best_at = e.best_at;
      }
      if (e.first && (!slot.first || e.first < slot.first)) slot.first = e.first;
      if (e.last > slot.last) slot.last = e.last;
    }
    for (const day of Object.keys(src.days)) {
      const e = src.days[day];
      const slot = dayEntry(dest, day);
      slot.t += e.t;
      slot.p += e.p;
      slot.w += e.w;
      slot.a += e.a;
      for (const k of Object.keys(e.g)) slot.g[k] = (slot.g[k] || 0) + e.g[k];
    }
    for (let i = 0; i < 24; i++) dest.hours[i] += src.hours[i];
    for (const key of Object.keys(src.series)) {
      const merged = (dest.series[key] || []).concat(src.series[key]).sort((a, b) => (a[0] < b[0] ? -1 : 1));
      dest.series[key] = merged.slice(-MAX_SERIES);
    }
    dest.events = dest.events.concat(src.events).sort((a, b) => (a.t < b.t ? -1 : 1)).slice(-MAX_EVENTS);
    for (const other of Object.keys(src.duel)) {
      const slot = dest.duel[other] || (dest.duel[other] = {});
      for (const k of Object.keys(src.duel[other])) {
        const cur = slot[k] || (slot[k] = { w: 0, l: 0 });
        cur.w += src.duel[other][k].w;
        cur.l += src.duel[other][k].l;
      }
    }
    dest.points = Math.max(dest.points, src.points);
    dest.longest = Math.max(dest.longest, src.longest);
    recalcStreak(dest);
    dest.level = PG.ach.levelFor(dest.points);
  }

  PG.stats = stats;
  if (PG.profiles) PG.profiles.onSwitch(() => stats.resetCache());
})();
