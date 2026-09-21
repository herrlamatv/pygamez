/**
 * ach.js - Erfolge, Seltenheitsstufen und Levelsystem der Web-Version
 * (Port von achievements.py + levels.py).
 *
 * Die Definitionen stehen in js/core/achdefs.js und werden von
 * devtools/export_achievements.py direkt aus achievements.py erzeugt - so
 * kennen Desktop und Browser dieselben Erfolge, Punkte und Stufen.
 *
 * Freigeschaltete Erfolge liegen (je Profil) im localStorage unter
 * `achievements` als {id: "YYYY-MM-DD HH:MM"}; Punkte, Level und der
 * Verlauf stehen in der Statistik (js/core/stats.js).
 *
 *   PG.ach.event("tile_2048")      Ereignis melden (aus den Spielen)
 *   PG.ach.checkStats()            zählbare Erfolge prüfen
 *   PG.ach.onHighscore(key, score) Punkte-Stufen prüfen
 *   PG.ach.drawToasts(ctx, dt)     Einblendung zeichnen (app.js, pro Frame)
 */
(function () {
  "use strict";

  const D = PG.achDefs;
  const SECTION = "achievements";
  const RARITY = D.rarity;
  const RARITY_ORDER = D.rarityOrder;

  let defs = null;
  let byId = null;
  let milestoneByGame = null;
  let unlockedCache = null;
  const toasts = [];

  function pad(n) {
    return String(n).padStart(2, "0");
  }

  function nowStr() {
    const d = new Date();
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }

  function hexRgb(hex) {
    const h = String(hex).replace("#", "");
    return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
  }

  /** Alle Spiele der Web-Version als {highscoreKey: manifest-eintrag}. */
  function gamesByKey() {
    const out = {};
    const entries = (PG.app && PG.app.entries) || [];
    for (const e of entries) {
      if (e.meta && e.meta.key) out[e.meta.key] = e;
    }
    return out;
  }

  function build() {
    if (defs) return defs;
    const games = gamesByKey();
    const list = [];
    const total = Object.keys(games).length || 46;
    for (const [id, icon, stat, target, rarity] of D.general) {
      list.push({ id, cat: "general", icon, stat, rarity,
        target: id === "explorer_all" ? total : target, game: null });
    }
    for (const key of Object.keys(games)) {
      if (D.milestones[key] != null) {
        list.push({ id: "score_" + key, cat: "milestone", icon: "star", stat: null,
          target: D.milestones[key], game: key, rarity: "rare" });
      }
      if (D.masters[key] != null) {
        list.push({ id: "master_" + key, cat: "master", icon: "crown", stat: null,
          target: D.masters[key], game: key, rarity: "epic" });
      }
    }
    for (const key of Object.keys(games)) {
      list.push({ id: "regular_" + key, cat: "regular", icon: "flag", stat: null,
        target: D.regularPlays, game: key, rarity: "common" });
    }
    for (const [id, icon, game, target, rarity] of D.specials) {
      list.push({ id, cat: "special", icon, stat: null, target, game, rarity });
    }
    defs = list;
    byId = {};
    milestoneByGame = {};
    for (const d of list) {
      byId[d.id] = d;
      if (d.cat === "milestone" || d.cat === "master") {
        (milestoneByGame[d.game] || (milestoneByGame[d.game] = [])).push(d);
      }
    }
    return defs;
  }

  function unlockedMap() {
    if (!unlockedCache) {
      const raw = PG.store.get(SECTION, {});
      unlockedCache = raw && typeof raw === "object" ? raw : {};
    }
    return unlockedCache;
  }

  function saveUnlocked() {
    PG.store.set(SECTION, unlockedMap());
  }

  // ---------------------------------------------------------------- Level
  function levelTotalFor(level) {
    const n = Math.max(1, level);
    return 4 * (n - 1) * (n + 3);
  }

  function levelCost(level) {
    return D.levelBase + D.levelStep * (Math.max(1, level) - 1);
  }

  function levelFor(points) {
    const p = Math.max(0, Math.round(points) || 0);
    let n = Math.max(1, Math.floor(-1 + Math.sqrt(4 + p / 4)));
    while (levelTotalFor(n + 1) <= p) n++;
    while (n > 1 && levelTotalFor(n) > p) n--;
    return n;
  }

  function rankOf(level) {
    let entry = D.ranks[0];
    for (const r of D.ranks) {
      if (level >= r[0]) entry = r;
      else break;
    }
    return entry;
  }

  const ach = {
    RARITY,
    RARITY_ORDER,

    defs() {
      return build().slice();
    },

    byId(id) {
      build();
      return byId[id] || null;
    },

    resetCache() {
      unlockedCache = null;
      toasts.length = 0;
    },

    unlocked() {
      return Object.assign({}, unlockedMap());
    },

    counts() {
      return [Object.keys(unlockedMap()).length, build().length];
    },

    pointsOf(d) {
      const def = typeof d === "string" ? this.byId(d) : d;
      if (!def) return 0;
      return (RARITY[def.rarity] || RARITY.common).points;
    },

    rarityColor(d) {
      const name = typeof d === "string" ? d : (d && d.rarity) || "common";
      return hexRgb((RARITY[name] || RARITY.common).color);
    },

    rarityName(d) {
      const name = typeof d === "string" ? d : (d && d.rarity) || "common";
      return PG.t("rarity." + name);
    },

    totalPoints() {
      build();
      let sum = 0;
      for (const id of Object.keys(unlockedMap())) if (byId[id]) sum += this.pointsOf(byId[id]);
      return sum;
    },

    maxPoints() {
      let sum = 0;
      for (const d of build()) sum += this.pointsOf(d);
      return sum;
    },

    rarityCounts() {
      const out = {};
      for (const r of RARITY_ORDER) out[r] = [0, 0];
      const un = unlockedMap();
      for (const d of build()) {
        const slot = out[d.rarity] || (out[d.rarity] = [0, 0]);
        slot[1]++;
        if (un[d.id]) slot[0]++;
      }
      return out;
    },

    gameName(key) {
      const e = gamesByKey()[key];
      return e ? PG.gameName(e) : key;
    },

    displayName(d) {
      if (d.cat === "milestone") return this.gameName(d.game);
      if (d.cat === "master") return PG.t("ach.master.name", { game: this.gameName(d.game) });
      if (d.cat === "regular") return PG.t("ach.regular.name", { game: this.gameName(d.game) });
      return PG.t("ach." + d.id + ".name");
    },

    displayDesc(d) {
      if (d.cat === "milestone" || d.cat === "master") return PG.t("ach.score.desc", { n: d.target });
      if (d.cat === "regular") return PG.t("ach.regular.desc", { n: d.target, game: this.gameName(d.game) });
      if (d.cat === "general" && d.stat) {
        const key = D.statDesc[d.stat];
        if (d.stat === "time") return PG.t(key, { t: Math.round(d.target / 3600) + "h" });
        return PG.t(key, { n: d.target });
      }
      return PG.t("ach." + d.id + ".desc");
    },

    progressOf(d, totals, scores) {
      if (d.cat === "general" && d.stat && d.target) {
        const t = totals || this.totals();
        return [Math.min(t[d.stat] || 0, d.target), d.target];
      }
      if (d.cat === "milestone" || d.cat === "master") {
        const s = scores || PG.highscore.all();
        return [Math.min(s[d.game] || 0, d.target), d.target];
      }
      if (d.cat === "regular") {
        return [Math.min(PG.stats.get(d.game).plays, d.target), d.target];
      }
      return null;
    },

    totals() {
      const t = PG.stats.totals();
      t.ach = Object.keys(unlockedMap()).length;
      return t;
    },

    // ----- Freischalten ---------------------------------------------------
    unlock(d, silent) {
      const un = unlockedMap();
      if (un[d.id]) return false;
      const stamp = nowStr();
      un[d.id] = stamp;
      saveUnlocked();

      const before = PG.stats.levelOf()[0];
      PG.stats.logAch(d.id, this.pointsOf(d), stamp);
      PG.stats.setPoints(this.totalPoints());
      const after = PG.stats.levelOf()[0];

      if (!silent) {
        toasts.push({ kind: "ach", def: d, t: 0, notes: [] });
        if (d.rarity === "legendary" || d.rarity === "mythic") {
          try {
            PG.ui.spawnConfetti(PG.W, PG.H, 60);
          } catch (e) {
            /* Konfetti ist Deko */
          }
        }
      }
      if (after > before) {
        PG.stats.logLevel(after, stamp);
        if (!silent) {
          toasts.push({ kind: "level", level: after, title: this.rankTitle(after),
            color: this.rankColor(after), t: 0, notes: [] });
        }
      }
      this.checkMeta(silent);
      return true;
    },

    unlockId(id, silent) {
      build();
      const d = byId[id];
      if (d && !unlockedMap()[id]) this.unlock(d, silent);
    },

    checkMeta(silent) {
      build();
      const un = unlockedMap();
      const seen = new Set();
      for (const id of Object.keys(un)) if (byId[id]) seen.add(byId[id].rarity);
      if (seen.has("legendary") || seen.has("mythic")) this.unlockId("first_legend", silent);
      let kinds = 0;
      for (const r of RARITY_ORDER) if (seen.has(r)) kinds++;
      if (kinds >= 4) this.unlockId("all_rarities", silent);
      const perGame = {};
      for (const d of defs) {
        if (!d.game) continue;
        const slot = perGame[d.game] || (perGame[d.game] = [0, 0]);
        slot[1]++;
        if (un[d.id]) slot[0]++;
      }
      for (const key of Object.keys(perGame)) {
        const [got, total] = perGame[key];
        if (total >= 3 && got === total) {
          this.unlockId("completionist", silent);
          break;
        }
      }
      const t = this.totals();
      for (const d of defs) {
        if (d.cat === "general" && d.stat === "ach" && d.target && t.ach >= d.target) {
          this.unlock(d, silent);
        }
      }
    },

    checkStats(silent) {
      const t = this.totals();
      for (const d of build()) {
        if (d.cat === "general" && d.stat && d.target) {
          if ((t[d.stat] || 0) >= d.target) this.unlock(d, silent);
        } else if (d.cat === "regular") {
          if (PG.stats.get(d.game).plays >= d.target) this.unlock(d, silent);
        }
      }
    },

    onHighscore(key, score, silent) {
      build();
      for (const d of milestoneByGame[key] || []) {
        if (score >= d.target) this.unlock(d, silent);
      }
    },

    onGameStarted(key) {
      const h = new Date().getHours();
      if (h < 5) this.event("night_owl");
      else if (h < 8) this.event("early_bird");
      this.checkWeekend();
      this.checkStats();
    },

    checkWeekend() {
      if (unlockedMap().weekend) return;
      const days = PG.stats.daysMap();
      for (const day of Object.keys(days)) {
        const date = new Date(day + "T00:00:00");
        if (isNaN(date) || date.getDay() !== 6) continue; // 6 = Samstag
        const next = new Date(date.getTime() + 86400000);
        const key = `${next.getFullYear()}-${pad(next.getMonth() + 1)}-${pad(next.getDate())}`;
        if (days[key]) {
          this.event("weekend");
          return;
        }
      }
    },

    onPlaytime(seconds) {
      if (seconds >= 3600) this.event("marathon", seconds);
    },

    event(id, value) {
      build();
      const d = byId[id];
      if (!d) return;
      if (d.target != null && d.stat == null && ["milestone", "master", "regular"].indexOf(d.cat) < 0) {
        if (value == null || value < d.target) return;
      }
      this.unlock(d, false);
    },

    /** Beim Start: Erreichtes still anrechnen. Gibt [neu, level, punkte]. */
    backfillSilent() {
      const before = Object.keys(unlockedMap()).length;
      try {
        const scores = PG.highscore.all();
        this.checkStats(true);
        for (const key of Object.keys(scores)) this.onHighscore(key, scores[key], true);
        // Erfolgs-Ereignisse aus dem alten Web-Stand (nur Zähler, ohne Zeit).
        const legacy = PG.stats.legacyEvents && PG.stats.legacyEvents();
        if (legacy) {
          build();
          for (const id of Object.keys(legacy)) {
            const d = byId[id];
            if (!d) continue;
            if (d.target != null && Number(legacy[id]) < d.target) continue;
            this.unlock(d, true);
          }
          PG.stats.clearLegacyEvents();
        }
        this.checkMeta(true);
        PG.stats.setPoints(this.totalPoints());
        PG.stats.flush();
      } catch (e) {
        console.error(e);
      }
      const [level, points] = PG.stats.levelOf();
      return [Object.keys(unlockedMap()).length - before, level, points];
    },

    rarestUnlocked(n) {
      build();
      const un = unlockedMap();
      const items = [];
      for (const id of Object.keys(un)) if (byId[id]) items.push([byId[id], un[id]]);
      items.sort((a, b) => {
        const ra = RARITY_ORDER.indexOf(b[0].rarity) - RARITY_ORDER.indexOf(a[0].rarity);
        return ra !== 0 ? ra : (a[1] < b[1] ? -1 : 1);
      });
      return items.slice(0, n || 3);
    },

    // ----- Level ----------------------------------------------------------
    levelFor,
    levelCost,
    levelTotalFor,

    levelProgress(points) {
      const p = Math.max(0, Math.round(points) || 0);
      const level = levelFor(p);
      return [level, p - levelTotalFor(level), levelCost(level)];
    },

    rankTitle(level) {
      return PG.t(rankOf(level)[1]);
    },

    rankColor(level) {
      return hexRgb(rankOf(level)[2]);
    },

    // ----- Abzeichen ------------------------------------------------------
    /** Rundes Erfolgs-Abzeichen mit Symbol (wie achievements.draw_badge). */
    drawBadge(ctx, center, radius, icon, color, locked) {
      const ui = PG.ui;
      const draw = PG.draw;
      let fill;
      let ring;
      let iconCol;
      if (locked) {
        fill = ui.mix(ui.PANEL, ui.PANEL_LIGHT, 0.5);
        ring = ui.BORDER_LIGHT;
        iconCol = ui.TEXT_FAINT;
        icon = "lock";
      } else {
        fill = ui.mix(ui.PANEL_LIGHT, color, 0.28);
        ring = color;
        iconCol = ui.mix(color, [255, 255, 255], 0.35);
      }
      draw.circle(ctx, fill, center, radius);
      draw.circle(ctx, ring, center, radius, 2);
      drawIcon(ctx, center[0], center[1], Math.max(4, radius * 0.62), icon, iconCol);
    },

    // ----- Toasts ---------------------------------------------------------
    drawToasts(ctx, dt) {
      if (!toasts.length) return;
      const ui = PG.ui;
      const t0 = toasts[0];
      t0.t += dt;
      const tt = t0.t;
      const level = t0.kind === "level";
      const notes = level
        ? [[0, 523], [0.12, 659], [0.24, 784], [0.36, 1047], [0.52, 1319]]
        : [[0, 660], [0.13, 880], [0.26, 1318]];
      notes.forEach(([at, freq], i) => {
        if (tt >= at && t0.notes.indexOf(i) < 0) {
          t0.notes.push(i);
          try {
            PG.audio.tone(freq, level ? 0.24 : 0.22, "square", 0.3);
          } catch (e) {
            /* ohne Ton weiter */
          }
        }
      });
      const IN = 0.35;
      const HOLD = 3.4 + (level ? 0.8 : 0);
      const OUT = 0.45;
      if (tt >= IN + HOLD + OUT) {
        toasts.shift();
        return;
      }

      let accent;
      let head;
      let name;
      let sub = "";
      let icon;
      if (level) {
        accent = t0.color;
        head = PG.t("level.up");
        name = PG.t("level.toast", { n: t0.level, rank: t0.title });
        icon = "bolt";
      } else {
        const d = t0.def;
        accent = this.rarityColor(d);
        head = PG.t("ach.unlocked");
        name = this.displayName(d);
        sub = `${this.rarityName(d)}  ·  +${this.pointsOf(d)}`;
        icon = d.icon;
      }
      const headFont = ui.font(12, true);
      const nameFont = ui.font(17, true);
      const subFont = ui.font(11, true);
      const br = 17;
      const pad2 = 14;
      const gap = 10;
      const textW = Math.max(headFont.width(head), nameFont.width(name), sub ? subFont.width(sub) : 0);
      const bw = Math.min(PG.W - 24, pad2 * 2 + br * 2 + gap + textW);
      const textH = headFont.height + nameFont.height + (sub ? subFont.height + 2 : 0);
      const bh = Math.max(br * 2 + 16, textH + 22);
      const ease = (x) => 1 - Math.pow(1 - Math.max(0, Math.min(1, x)), 3);
      let off = 0;
      if (tt < IN) off = (1 - ease(tt / IN)) * (bw + 24);
      else if (tt > IN + HOLD) off = ease((tt - IN - HOLD) / OUT) * (bw + 24);

      const rect = new PG.Rect(Math.round(PG.W - 12 - bw + off), 12, bw, bh);
      ui.drawPanel(ctx, rect, { color: ui.PANEL_LIGHT, radius: 12, accentTop: accent });
      const cx = rect.x + pad2 + br;
      const cy = rect.centery;
      PG.draw.circle(ctx, ui.col(accent, Math.round(50 * ui.pulse(3))), [cx, cy], br * 2);
      this.drawBadge(ctx, [cx, cy], br, icon, accent, false);
      const tx = rect.x + pad2 + br * 2 + gap;
      let ty = cy - textH / 2;
      ui.text(ctx, head, tx, ty, headFont, level ? accent : ui.GOLD);
      ty += headFont.height + 2;
      ui.text(ctx, name, tx, ty, nameFont, ui.TEXT);
      if (sub) ui.text(ctx, sub, tx, ty + nameFont.height, subFont, accent);
    },
  };

  /** Kleine, prozedural gezeichnete Symbole (wie achievements._draw_icon). */
  function drawIcon(ctx, cx, cy, r, icon, col) {
    const draw = PG.draw;
    const ui = PG.ui;
    if (icon === "star") {
      const pts = [];
      for (let i = 0; i < 10; i++) {
        const rad = i % 2 === 0 ? r : r * 0.45;
        const a = -Math.PI / 2 + (i * Math.PI) / 5;
        pts.push([cx + rad * Math.cos(a), cy + rad * Math.sin(a)]);
      }
      draw.polygon(ctx, col, pts);
    } else if (icon === "trophy") {
      const cw = r * 1.2;
      const chh = r * 0.9;
      draw.rect(ctx, col, [cx - cw / 2, cy - r + 1, cw, chh], 0, 3);
      for (const side of [-1, 1]) draw.circle(ctx, col, [cx + side * (cw / 2 + 2), cy - r + chh / 2], 3, 1);
      draw.rect(ctx, col, [cx - 2, cy - r + chh, 4, Math.max(2, r / 3)]);
      draw.rect(ctx, col, [cx - r / 2 - 1, cy + r - Math.max(3, r / 3), r + 2, Math.max(3, r / 3)], 0, 1);
    } else if (icon === "clock") {
      draw.circle(ctx, col, [cx, cy], r, 2);
      draw.lines(ctx, col, false, [[cx, cy], [cx, cy - r + 3]], 2);
      draw.lines(ctx, col, false, [[cx, cy], [cx + r - 4, cy + 2]], 2);
    } else if (icon === "medal") {
      draw.polygon(ctx, col, [[cx - r + 2, cy - r], [cx - 1, cy], [cx - r + 5, cy]]);
      draw.polygon(ctx, col, [[cx + r - 2, cy - r], [cx + 1, cy], [cx + r - 5, cy]]);
      draw.circle(ctx, col, [cx, cy + r / 3], Math.max(3, r * 0.55));
    } else if (icon === "flag") {
      draw.lines(ctx, col, false, [[cx - r + 3, cy - r], [cx - r + 3, cy + r]], 2);
      draw.polygon(ctx, col, [[cx - r + 4, cy - r], [cx + r - 1, cy - r / 2], [cx - r + 4, cy]]);
    } else if (icon === "crown") {
      const base = cy + r / 2;
      draw.polygon(ctx, col, [[cx - r, base], [cx - r, cy - r / 3], [cx - r / 2, cy],
        [cx, cy - r], [cx + r / 2, cy], [cx + r, cy - r / 3], [cx + r, base]]);
    } else if (icon === "gem") {
      draw.polygon(ctx, col, [[cx, cy - r], [cx + r, cy], [cx, cy + r], [cx - r, cy]]);
      draw.polygon(ctx, ui.mix(col, [255, 255, 255], 0.4),
        [[cx, cy - r], [cx + r / 2, cy - r / 2], [cx, cy], [cx - r / 2, cy - r / 2]]);
    } else if (icon === "heart") {
      draw.circle(ctx, col, [cx - r / 2, cy - r / 3], r / 2 + 1);
      draw.circle(ctx, col, [cx + r / 2, cy - r / 3], r / 2 + 1);
      draw.polygon(ctx, col, [[cx - r, cy - r / 4], [cx + r, cy - r / 4], [cx, cy + r]]);
    } else if (icon === "check") {
      draw.lines(ctx, col, false, [[cx - r + 2, cy], [cx - r / 4, cy + r - 3], [cx + r - 1, cy - r + 3]], 3);
    } else if (icon === "llama") {
      draw.rect(ctx, col, [cx - r / 3, cy - r / 4, Math.max(3, r / 2), r + r / 3], 0, 2);
      draw.rect(ctx, col, [cx - r, cy - r, r * 1.1, r * 0.8], 0, 3);
      for (const dx of [-r + 2, -r / 3]) {
        draw.polygon(ctx, col, [[cx + dx, cy - r], [cx + dx + 3, cy - r - r / 2], [cx + dx + 5, cy - r]]);
      }
      draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.6), [cx - r + r * 0.55 - 1, cy - r + r * 0.4], Math.max(1, r / 6));
    } else if (icon === "person") {
      draw.circle(ctx, col, [cx, cy - r / 2], Math.max(3, r / 2));
      draw.polygon(ctx, col, [[cx - r, cy + r], [cx - r + 2, cy + 1], [cx + r - 2, cy + 1], [cx + r, cy + r]]);
    } else if (icon === "flame") {
      draw.polygon(ctx, col, [[cx, cy - r], [cx + r, cy + r / 3], [cx + r / 2, cy + r],
        [cx - r / 2, cy + r], [cx - r, cy + r / 3]]);
      draw.polygon(ctx, ui.mix(col, [255, 255, 255], 0.45),
        [[cx, cy - r / 4], [cx + r / 2, cy + r / 2], [cx - r / 2, cy + r / 2]]);
    } else if (icon === "calendar") {
      draw.rect(ctx, col, [cx - r, cy - r + 2, 2 * r, 2 * r - 2], 2, 2);
      draw.lines(ctx, col, false, [[cx - r, cy - r + 7], [cx + r, cy - r + 7]], 2);
      for (let i = 0; i < 2; i++) {
        draw.lines(ctx, col, false, [[cx - r / 2 + i * r, cy - r], [cx - r / 2 + i * r, cy - r + 4]], 2);
      }
      draw.rect(ctx, col, [cx - 1, cy + 1, 3, 3]);
    } else if (icon === "dice") {
      draw.rect(ctx, col, [cx - r, cy - r, 2 * r, 2 * r], 2, 3);
      for (const [dx, dy] of [[-r / 2, -r / 2], [r / 2, r / 2], [0, 0]]) {
        draw.circle(ctx, col, [cx + dx, cy + dy], Math.max(1, r / 5));
      }
    } else if (icon === "bolt") {
      draw.polygon(ctx, col, [[cx + r / 3, cy - r], [cx - r, cy + 1], [cx, cy + 1],
        [cx - r / 3, cy + r], [cx + r, cy - 1], [cx, cy - 1]]);
    } else if (icon === "moon") {
      draw.circle(ctx, col, [cx, cy], r);
      draw.circle(ctx, ui.PANEL_LIGHT, [cx + r / 2, cy - r / 3], r);
    } else if (icon === "target") {
      draw.circle(ctx, col, [cx, cy], r, 2);
      draw.circle(ctx, col, [cx, cy], Math.max(2, r / 2), 2);
      draw.circle(ctx, col, [cx, cy], Math.max(1, r / 5));
    } else if (icon === "swords") {
      draw.lines(ctx, col, false, [[cx - r, cy + r], [cx + r, cy - r]], 2);
      draw.lines(ctx, col, false, [[cx + r, cy + r], [cx - r, cy - r]], 2);
      draw.circle(ctx, col, [cx - r, cy + r], Math.max(2, r / 4));
      draw.circle(ctx, col, [cx + r, cy + r], Math.max(2, r / 4));
    } else if (icon === "chart") {
      const base = cy + r;
      [0.45, 0.8, 0.6, 1].forEach((h, i) => {
        const bx = cx - r + (i * 2 * r) / 4 + 1;
        const bh = 2 * r * h * 0.75;
        draw.rect(ctx, col, [bx, base - bh, Math.max(2, r / 3), bh], 0, 1);
      });
    } else if (icon === "rocket") {
      draw.polygon(ctx, col, [[cx, cy - r], [cx + r / 2, cy], [cx + r / 2, cy + r / 2],
        [cx - r / 2, cy + r / 2], [cx - r / 2, cy]]);
      draw.polygon(ctx, col, [[cx - r / 2, cy], [cx - r, cy + r / 2], [cx - r / 2, cy + r / 2]]);
      draw.polygon(ctx, col, [[cx + r / 2, cy], [cx + r, cy + r / 2], [cx + r / 2, cy + r / 2]]);
      draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.5), [cx, cy - r / 4], Math.max(1, r / 4));
    } else {
      draw.rect(ctx, col, [cx - r + 2, cy - 1, 2 * r - 4, r + 1], 0, 2);
      draw.circle(ctx, col, [cx, cy - r / 3 - 1], r / 2 + 1, 2);
    }
  }

  ach.drawIcon = drawIcon;
  PG.ach = ach;
  if (PG.profiles) PG.profiles.onSwitch(() => ach.resetCache());
})();
