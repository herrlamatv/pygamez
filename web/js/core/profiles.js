/**
 * profiles.js - Spielerprofile für die Web-Version (Port von profiles.py).
 *
 * Mehrere Spieler an einem Gerät: jedes Profil hat eigene Highscores,
 * Erfolge, Statistiken, Chips und Spielstände. Technisch bekommt jedes Profil
 * im localStorage ein eigenes Prefix (`pygamez.<id>.<schlüssel>`) - die
 * Umleitung macht PG.store selbst, die Spiele merken davon nichts.
 *
 * Gerätweit bleiben nur `lang`, `settings` und `profiles` (die Verwaltung).
 * Das entspricht der Desktop-Version, wo alles unter `players/<id>` in der
 * mem.json liegt und nur Sprache und settings.json geteilt werden.
 *
 *   PG.profiles.init()        einmal beim Start (legt Profil 1 an, migriert)
 *   PG.profiles.activeId()    aktive Profil-id
 *   PG.profiles.all()         [{id, name, icon, color, created, seen}, ...]
 *   PG.profiles.switchTo(id)  Profil wechseln (leert alle Zwischenspeicher)
 */
(function () {
  "use strict";

  const KEY = "profiles";
  const FIRST = "p1";
  const MAX = 8;

  // Symbole und Farben wie in profiles.py (gleiche Reihenfolge!).
  const ICONS = ["llama", "star", "crown", "gem", "heart", "rocket",
    "moon", "flame", "dice", "target", "medal", "check"];
  const COLORS = ["#6ecd8c", "#589cff", "#f0a05a", "#b07fe8", "#5ad4d4", "#f5cd64",
    "#e05a7d", "#4fd17a", "#c86ad8", "#e2725b", "#35e2ff", "#b9f23a"];
  const MAX_NAME = 14;
  const GUEST = "guest";

  function stamp() {
    const d = new Date();
    const p = (n) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
  }

  function blankMeta(name) {
    const now = stamp();
    return { name: name || "", icon: "llama", color: 0, created: now, seen: now };
  }

  /** Rückrufe, die nach einem Profilwechsel die Zwischenspeicher leeren. */
  const switchHooks = [];

  const profiles = {
    ICONS,
    COLORS,
    MAX_NAME,
    MAX,
    GUEST,

    /** Liest die Verwaltung (immer mit mindestens einem Profil). */
    load() {
      let raw = PG.store.get(KEY, null);
      if (!raw || typeof raw !== "object" || !raw.list || !Object.keys(raw.list).length) {
        raw = { active: FIRST, order: [FIRST], list: { [FIRST]: blankMeta("") } };
      }
      const list = {};
      for (const id of Object.keys(raw.list)) {
        const m = raw.list[id] || {};
        const clean = blankMeta(typeof m.name === "string" ? m.name.slice(0, MAX_NAME) : "");
        if (typeof m.icon === "string" && ICONS.indexOf(m.icon) >= 0) clean.icon = m.icon;
        if (Number.isInteger(m.color)) clean.color = Math.max(0, Math.min(COLORS.length - 1, m.color));
        if (typeof m.created === "string") clean.created = m.created;
        if (typeof m.seen === "string") clean.seen = m.seen;
        list[id] = clean;
      }
      const order = (Array.isArray(raw.order) ? raw.order : []).filter((id) => list[id]);
      for (const id of Object.keys(list)) if (order.indexOf(id) < 0) order.push(id);
      const active = list[raw.active] ? raw.active : order[0];
      return { active, order, list };
    },

    save(data) {
      PG.store.set(KEY, data);
      PG.store.profile = data.active;
    },

    /**
     * Einmal beim Start: Profil-Prefix setzen und - beim ersten Mal nach dem
     * Update - den vorhandenen Spielstand ins erste Profil umziehen.
     */
    init() {
      const fresh = PG.store.get(KEY, null) == null;
      const data = this.load();
      PG.store.profile = data.active;
      if (fresh) {
        this.migrate(data.active);
        this.save(data);
      } else {
        this.save(data);
      }
      return data.active;
    },

    /** Verschiebt alte `pygamez.X`-Schlüssel nach `pygamez.<pid>.X`. */
    migrate(pid) {
      let ls;
      try {
        ls = window.localStorage;
      } catch (e) {
        return;
      }
      const pre = PG.store.prefix;
      const move = [];
      for (let i = 0; i < ls.length; i++) {
        const full = ls.key(i);
        if (!full || full.indexOf(pre) !== 0) continue;
        const name = full.slice(pre.length);
        const base = name.split(".")[0];
        if (PG.store.shared.indexOf(base) >= 0) continue;
        // Schon umgezogene Schlüssel (pygamez.p1.xyz) nicht erneut anfassen.
        if (/^p\d+\./.test(name)) continue;
        move.push(name);
      }
      for (const name of move) {
        try {
          const value = ls.getItem(pre + name);
          ls.setItem(pre + pid + "." + name, value);
          ls.removeItem(pre + name);
        } catch (e) {
          /* Speicher voll: dann bleibt der alte Schlüssel eben liegen */
        }
      }
    },

    onSwitch(fn) {
      if (switchHooks.indexOf(fn) < 0) switchHooks.push(fn);
    },

    fireSwitch() {
      for (const fn of switchHooks.slice()) {
        try {
          fn();
        } catch (e) {
          console.error(e);
        }
      }
    },

    // ----- Abfragen -------------------------------------------------------
    activeId() {
      return this.load().active;
    },

    all() {
      const d = this.load();
      return d.order.map((id) => Object.assign({ id }, d.list[id]));
    },

    count() {
      return this.load().order.length;
    },

    meta(pid) {
      const d = this.load();
      return Object.assign({}, d.list[pid || d.active] || blankMeta(""));
    },

    nameOf(pid) {
      if (pid === GUEST) return PG.t("profile.guest");
      const d = this.load();
      const id = pid || d.active;
      const m = d.list[id];
      const name = m && m.name ? m.name.trim() : "";
      if (name) return name;
      const n = Math.max(1, d.order.indexOf(id) + 1);
      return PG.t("profile.player_n", { n });
    },

    iconOf(pid) {
      if (pid === GUEST) return "person";
      return this.meta(pid).icon || "llama";
    },

    colorHex(pid) {
      if (pid === GUEST) return "#9aa6bf";
      return COLORS[this.meta(pid).color % COLORS.length];
    },

    levelOf(pid) {
      return PG.stats.levelOf(pid || this.activeId());
    },

    /** "" = Name in Ordnung, sonst ein Fehlercode (empty/long/dirty/dup/full). */
    checkName(name, pid) {
      const text = (name || "").trim();
      if (!text) return "empty";
      if (text.length > MAX_NAME) return "long";
      if (PG.swear && !PG.swear.allClean(text)) return "dirty";
      const lower = text.toLowerCase();
      for (const p of this.all()) {
        if (p.id === pid) continue;
        if ((p.name || "").trim().toLowerCase() === lower) return "dup";
      }
      return "";
    },

    // ----- Ändern ---------------------------------------------------------
    create(name, icon, color) {
      const err = this.checkName(name);
      if (err) return [null, err];
      const d = this.load();
      if (d.order.length >= MAX) return [null, "full"];
      let i = 1;
      while (d.list["p" + i]) i++;
      const pid = "p" + i;
      const meta = blankMeta(name.trim());
      if (icon) meta.icon = icon;
      if (Number.isInteger(color)) meta.color = color;
      d.list[pid] = meta;
      d.order.push(pid);
      this.save(d);
      PG.ach.event("profile_second", d.order.length);
      PG.ach.event("profile_family", d.order.length);
      return [pid, ""];
    },

    rename(pid, name) {
      const err = this.checkName(name, pid);
      if (err) return err;
      const d = this.load();
      if (!d.list[pid]) return "empty";
      d.list[pid].name = name.trim();
      this.save(d);
      return "";
    },

    setLook(pid, icon, color) {
      const d = this.load();
      if (!d.list[pid]) return;
      if (icon) d.list[pid].icon = icon;
      if (Number.isInteger(color)) d.list[pid].color = color;
      this.save(d);
    },

    /** Löscht ein Profil samt allen seinen Daten (das letzte bleibt). */
    del(pid) {
      const d = this.load();
      if (!d.list[pid] || d.order.length <= 1) return false;
      PG.stats.flush();
      PG.stats.dropProfile(pid);
      try {
        const ls = window.localStorage;
        const pre = PG.store.prefix + pid + ".";
        const kill = [];
        for (let i = 0; i < ls.length; i++) {
          const k = ls.key(i);
          if (k && k.indexOf(pre) === 0) kill.push(k);
        }
        for (const k of kill) ls.removeItem(k);
      } catch (e) {
        /* nichts zu löschen */
      }
      delete d.list[pid];
      d.order = d.order.filter((id) => id !== pid);
      if (d.active === pid) d.active = d.order[0];
      this.save(d);
      this.fireSwitch();
      return true;
    },

    switchTo(pid) {
      const d = this.load();
      if (!d.list[pid] || d.active === pid) return false;
      PG.stats.flush();
      d.active = pid;
      d.list[pid].seen = stamp();
      this.save(d);
      this.fireSwitch();
      return true;
    },

    touch(pid) {
      const d = this.load();
      const id = pid || d.active;
      if (d.list[id]) {
        d.list[id].seen = stamp();
        this.save(d);
      }
    },
  };

  PG.profiles = profiles;
})();
