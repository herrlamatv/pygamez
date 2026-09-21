/**
 * progressview.js - der Erfolge-&-Statistik-Screen der Web-Version
 * (Port von progress.py).
 *
 * Fünf Reiter: Erfolge, Statistiken, Kalender, Verlauf, Profile - mit
 * Seltenheitsstufen, Erfolgspunkten, Level, Spieltage-Karte, Wochentags- und
 * Tageszeit-Diagrammen, Ereignis-Verlauf samt Zeitstempeln, Geräte-
 * Bestenliste und dem "PyGameZ-Rückblick". Export/Import schreiben bzw. lesen
 * dieselbe `.pgzlamastats`-Datei wie die Desktop-Version.
 */
(function () {
  "use strict";

  const { ui, t } = PG;
  const W = PG.W;
  const H = PG.H;

  const ROW_ACH = 46;
  const ROW_GAME = 30;
  const ROW_HEAD = 30;
  const ROW_EVENT = 28;
  const GAP = 6;

  const TABS = ["achievements", "stats", "calendar", "timeline", "profiles"];
  const CATS = ["general", "milestone", "master", "regular", "special"];
  const FILTERS = ["all", "unlocked", "locked"];

  function pad(n) {
    return String(n).padStart(2, "0");
  }

  function prettyDate(day) {
    const text = String(day || "").slice(0, 10);
    const date = new Date(text + "T00:00:00");
    if (isNaN(date)) return text;
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const diff = Math.round((today - date) / 86400000);
    if (diff === 0) return t("cal.today");
    if (diff === 1) return t("cal.yesterday");
    return t("cal.date", { d: date.getDate(), m: date.getMonth() + 1, y: date.getFullYear() });
  }

  function gameEntry(key) {
    const entries = (PG.app && PG.app.entries) || [];
    return entries.find((e) => e.meta && e.meta.key === key) || null;
  }

  function gameName(key) {
    const e = gameEntry(key);
    return e ? PG.gameName(e) : key;
  }

  function gameColor(key) {
    const e = gameEntry(key);
    return e ? ui.gameColor(e.id) : ui.ACCENT;
  }

  class ProgressScreen {
    constructor(app, onClose, tab) {
      this.app = app;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.onClose = onClose || (() => app.backToMenu());
      this.name = t("progress.name");
      this.tab = TABS.includes(tab) ? tab : "achievements";
      this.filter = "all";
      this.scroll = 0;
      this.tabHover = null;
      this.hoverRow = null;
      this.mouse = [0, 0];
      this.hits = {};
      this.tip = null;
      this.buttons = [];
      this.seriesGame = null;
      this.flash = "";
      this.flashT = 0;
      this.build();
    }

    // ----- Aufbau ---------------------------------------------------------
    setTab(tab) {
      if (!TABS.includes(tab) || tab === this.tab) return;
      this.tab = tab;
      this.scroll = 0;
      this.hoverRow = null;
      this.build();
      PG.audio.play("move");
      if (tab === "calendar" || tab === "timeline") PG.ach.event("statistician");
      if (tab === "calendar") PG.ach.event("wrapped");
    }

    build() {
      this.left = Math.max(28, W / 2 - 330);
      this.right = W - this.left;
      this.top = 132;
      this.bottomY = H - 52;
      this.buttons = [];

      this.tabFont = ui.font(13, true);
      this.tabRects = [];
      let tx = this.left;
      for (const key of TABS) {
        const tw = this.tabFont.width(t("progress.tab_" + key)) + 20;
        this.tabRects.push([new PG.Rect(tx, 48, tw, 26), key]);
        tx += tw + 6;
      }

      this.unlocked = PG.ach.unlocked();
      this.totals = PG.ach.totals();
      this.scores = PG.highscore.all();
      [this.nUnlocked, this.nTotal] = PG.ach.counts();

      this.rows = [];
      let y = 0;
      if (this.tab === "achievements") y = this.buildAchievements(y);
      else if (this.tab === "stats") y = this.buildStats(y);
      else if (this.tab === "calendar") y = this.buildCalendar(y);
      else if (this.tab === "timeline") y = this.buildTimeline(y);
      else y = this.buildProfiles(y);
      this.contentH = y;
      this.clampScroll();
    }

    buildAchievements(y) {
      this.rows.push([y, 26, "filter", null]);
      y += 26 + GAP;
      const byCat = {};
      for (const c of CATS) byCat[c] = [];
      for (const d of PG.ach.defs()) {
        const ts = this.unlocked[d.id];
        if (this.filter === "unlocked" && !ts) continue;
        if (this.filter === "locked" && ts) continue;
        byCat[d.cat].push(d);
      }
      for (const cat of CATS) {
        if (!byCat[cat].length) continue;
        const got = byCat[cat].filter((d) => this.unlocked[d.id]).length;
        this.rows.push([y, ROW_HEAD, "head", `${t("progress.cat_" + cat)}   ${got}/${byCat[cat].length}`]);
        y += ROW_HEAD + GAP;
        for (const d of byCat[cat]) {
          const ts = this.unlocked[d.id];
          const prog = ts ? null : PG.ach.progressOf(d, this.totals, this.scores);
          this.rows.push([y, ROW_ACH, "ach", [d, ts, prog]]);
          y += ROW_ACH + GAP;
        }
        y += 6;
      }
      if (this.rows.length <= 1) {
        this.rows.push([y, ROW_HEAD, "note", t("progress.filter_empty")]);
        y += ROW_HEAD;
      }
      return y;
    }

    buildStats(y) {
      this.cardCols = this.right - this.left >= 560 ? 3 : 2;
      this.cardH = 64;
      const cardRows = Math.ceil(8 / this.cardCols);
      const cardsH = cardRows * (this.cardH + 10) + 30;
      this.rows.push([y, cardsH, "cards", null]);
      y += cardsH + 14;

      const entries = [];
      for (const e of this.app.entries) {
        const st = PG.stats.get(e.meta.key);
        if (st.plays > 0 || st.time > 0) entries.push([e, st]);
      }
      entries.sort((a, b) => b[1].time - a[1].time);
      this.entries = entries;

      if (entries.length) {
        const top = entries.slice(0, 10);
        this.rows.push([y, ROW_HEAD, "head", t("progress.top_games")]);
        y += ROW_HEAD;
        const h = 22 * top.length + 10;
        this.rows.push([y, h, "chartTop", top]);
        y += h + 14;

        this.rows.push([y, ROW_HEAD, "head", t("progress.balance")]);
        y += ROW_HEAD;
        this.rows.push([y, 150, "chartSplit", null]);
        y += 150 + 14;

        this.rows.push([y, ROW_HEAD, "tableHead", null]);
        y += ROW_HEAD + 2;
        for (const [e, st] of entries) {
          this.rows.push([y, ROW_GAME, "game", [e, st]]);
          y += ROW_GAME + 2;
        }
        const rest = this.app.entries.length - entries.length;
        if (rest > 0) {
          y += 6;
          this.rows.push([y, ROW_HEAD, "note", t("progress.never_played", { n: rest })]);
          y += ROW_HEAD;
        }
      } else {
        this.rows.push([y, ROW_HEAD, "note", t("progress.no_stats")]);
        y += ROW_HEAD;
      }
      return y;
    }

    buildCalendar(y) {
      this.days = PG.stats.daysMap();
      this.dayTime = {};
      for (const d of Object.keys(this.days)) this.dayTime[d] = this.days[d].t || 0;
      this.streak = PG.stats.streak();
      this.facts = this.buildFacts();

      const h = 24 + 20 * this.facts.length;
      this.rows.push([y, h, "wrapped", null]);
      y += h + 14;
      this.rows.push([y, ROW_HEAD, "head", t("progress.calendar_head")]);
      y += ROW_HEAD;
      this.rows.push([y, 132, "heat", null]);
      y += 132 + 10;
      this.rows.push([y, 40, "streaks", null]);
      y += 40 + 14;
      this.rows.push([y, ROW_HEAD, "head", t("progress.weekday_head")]);
      y += ROW_HEAD;
      this.rows.push([y, 110, "weekdays", null]);
      y += 110 + 14;
      this.rows.push([y, ROW_HEAD, "head", t("progress.clock_head")]);
      y += ROW_HEAD;
      this.rows.push([y, 190, "clock", null]);
      y += 190 + 10;
      return y;
    }

    buildFacts() {
      const facts = [];
      const days = this.days;
      const keys = Object.keys(days);
      if (!keys.length) return facts;
      let bestDay = keys[0];
      for (const d of keys) if ((days[d].t || 0) > (days[bestDay].t || 0)) bestDay = d;
      if (days[bestDay].t > 0) {
        facts.push(t("progress.fact_best_day", { day: prettyDate(bestDay), time: PG.stats.formatTime(days[bestDay].t) }));
      }
      const hours = PG.stats.hoursList();
      if (hours.some((v) => v > 0)) {
        const peak = hours.indexOf(Math.max(...hours));
        facts.push(t("progress.fact_peak_hour", { h: peak, h2: (peak + 1) % 24 }));
      }
      if (this.totals.longest) {
        facts.push(t("progress.fact_longest", { time: PG.stats.formatTime(this.totals.longest) }));
      }
      if (this.totals.favorite) {
        facts.push(t("progress.fact_favorite", {
          game: gameName(this.totals.favorite),
          time: PG.stats.formatTime(PG.stats.get(this.totals.favorite).time),
        }));
      }
      const rare = PG.ach.rarestUnlocked(1);
      if (rare.length) {
        facts.push(t("progress.fact_rarest", {
          name: PG.ach.displayName(rare[0][0]), rarity: PG.ach.rarityName(rare[0][0]),
        }));
      }
      if (this.totals.days) {
        facts.push(t("progress.fact_days", {
          n: this.totals.days,
          avg: PG.stats.formatTime(this.totals.time / Math.max(1, this.totals.days)),
        }));
      }
      return facts.slice(0, 6);
    }

    buildTimeline(y) {
      const played = PG.stats.playedGames();
      if (!played.includes(this.seriesGame)) this.seriesGame = played[0] || null;
      this.playedSeries = played.slice(0, 8);
      if (this.seriesGame) {
        this.rows.push([y, 26, "seriesPick", null]);
        y += 26 + 6;
        this.rows.push([y, 140, "chartSeries", null]);
        y += 140 + 14;
      }
      this.rows.push([y, ROW_HEAD, "head", t("progress.timeline_head")]);
      y += ROW_HEAD + 2;
      const events = PG.stats.events({ limit: 400 });
      if (!events.length) {
        this.rows.push([y, ROW_HEAD, "note", t("progress.no_events")]);
        return y + ROW_HEAD;
      }
      let lastDay = null;
      for (const ev of events) {
        const day = String(ev.t || "").slice(0, 10);
        if (day !== lastDay) {
          lastDay = day;
          this.rows.push([y, 24, "dayHead", prettyDate(day)]);
          y += 24;
        }
        this.rows.push([y, ROW_EVENT, "event", ev]);
        y += ROW_EVENT + 2;
      }
      return y;
    }

    buildProfiles(y) {
      this.plist = PG.profiles.all();
      this.pstats = {};
      for (const p of this.plist) this.pstats[p.id] = PG.stats.totals(p.id);

      this.rows.push([y, ROW_HEAD, "head", t("progress.profiles_head")]);
      y += ROW_HEAD + 2;
      for (const p of this.plist) {
        this.rows.push([y, 54, "profile", p]);
        y += 54 + GAP;
      }
      y += 8;

      const board = this.buildBoard();
      if (board.length) {
        this.rows.push([y, ROW_HEAD, "head", t("progress.board_head")]);
        y += ROW_HEAD + 2;
        for (const entry of board) {
          this.rows.push([y, ROW_GAME, "board", entry]);
          y += ROW_GAME + 2;
        }
        y += 8;
      }
      const duels = this.buildDuels();
      if (duels.length) {
        this.rows.push([y, ROW_HEAD, "head", t("progress.duel_head")]);
        y += ROW_HEAD + 2;
        for (const entry of duels) {
          this.rows.push([y, ROW_GAME, "duel", entry]);
          y += ROW_GAME + 2;
        }
      }
      return y;
    }

    buildBoard() {
      const perProfile = {};
      for (const p of this.plist) {
        let raw = null;
        try {
          const key = PG.store.prefix + (p.id === PG.store.profile ? "" : p.id + ".") + "highscores";
          raw = JSON.parse(window.localStorage.getItem(key) || "{}");
        } catch (e) {
          raw = {};
        }
        perProfile[p.id] = raw && typeof raw === "object" ? raw : {};
      }
      const out = [];
      for (const e of this.app.entries) {
        const key = e.meta.key;
        const values = [];
        for (const p of this.plist) {
          const score = Number(perProfile[p.id][key]) || 0;
          if (score > 0) values.push([p.id, score]);
        }
        if (values.length > 1) {
          values.sort((a, b) => b[1] - a[1]);
          out.push([key, values]);
        }
      }
      return out;
    }

    buildDuels() {
      const me = PG.profiles.activeId();
      const table = PG.stats.duelTable(me);
      const out = [];
      for (const other of Object.keys(table)) {
        for (const key of Object.keys(table[other])) {
          out.push([key, me, other, table[other][key].w, table[other][key].l]);
        }
      }
      out.sort((a, b) => b[3] + b[4] - (a[3] + a[4]));
      return out;
    }

    viewport() {
      return this.bottomY - this.top;
    }

    clampScroll() {
      this.scroll = Math.max(0, Math.min(this.scroll, Math.max(0, this.contentH - this.viewport())));
    }

    // ----- Eingabe --------------------------------------------------------
    handleEvent(e) {
      if (e.kind === "keydown") {
        const k = e.key;
        if (k === "Escape") this.onClose();
        else if (k === "Tab") this.setTab(TABS[PG.mod(TABS.indexOf(this.tab) + 1, TABS.length)]);
        else if (k === "Up" || k === "w") {
          this.scroll -= 40;
          this.clampScroll();
        } else if (k === "Down" || k === "s") {
          this.scroll += 40;
          this.clampScroll();
        } else if (k === "Prior") {
          this.scroll -= this.viewport();
          this.clampScroll();
        } else if (k === "Next") {
          this.scroll += this.viewport();
          this.clampScroll();
        } else if (k === "Left" || k === "a") {
          this.setTab(TABS[PG.mod(TABS.indexOf(this.tab) - 1, TABS.length)]);
        } else if (k === "Right" || k === "d") {
          this.setTab(TABS[PG.mod(TABS.indexOf(this.tab) + 1, TABS.length)]);
        } else if (k === "e" || k === "E") this.exportStats();
        else if (k === "i" || k === "I") this.importStats();
      } else if (e.kind === "wheel") {
        this.scroll -= e.delta * 48;
        this.clampScroll();
      } else if (e.kind === "mousemove") {
        this.mouse = e.pos;
        this.tabHover = null;
        for (const [r, key] of this.tabRects) if (r.collidepoint(e.pos)) this.tabHover = key;
        this.hoverRow = this.rowAt(e.pos);
      } else if (e.kind === "mousedown" && e.button === 1) {
        for (const [r, key] of this.tabRects) {
          if (r.collidepoint(e.pos)) {
            this.setTab(key);
            return;
          }
        }
        for (const [r, cb] of this.buttons) {
          if (r.collidepoint(e.pos)) {
            PG.audio.play("click");
            cb();
            return;
          }
        }
        const hit = this.rowAt(e.pos);
        if (hit != null) {
          const entry = this.rowGame(this.rows[hit]);
          if (entry) {
            PG.audio.play("click");
            this.app.openPreGame(entry);
          }
        }
      }
    }

    rowAt(pos) {
      const [x, y] = pos;
      if (x < this.left || x > this.right || y < this.top || y > this.bottomY) return null;
      const cy = y - this.top + this.scroll;
      for (let i = 0; i < this.rows.length; i++) {
        const [ry, rh, kind] = this.rows[i];
        if (cy >= ry && cy <= ry + rh && ["ach", "game", "board"].includes(kind)) return i;
      }
      return null;
    }

    rowGame(row) {
      const [, , kind, data] = row;
      if (kind === "game") return data[0];
      if (kind === "board") return gameEntry(data[0]);
      if (kind === "ach") return data[0].game ? gameEntry(data[0].game) : null;
      return null;
    }

    setFilter(key) {
      this.filter = key;
      this.scroll = 0;
      this.build();
    }

    pickSeries(key) {
      this.seriesGame = key;
      this.build();
    }

    exportStats() {
      const name = "pygamez-stats" + PG.stats.EXT;
      PG.downloadText(name, PG.stats.exportText(), "application/json");
      this.setFlash(t("progress.export_ok"));
    }

    importStats() {
      PG.pickTextFile(PG.stats.EXT + ",.json,application/json", (text) => {
        const [ok, reason] = PG.stats.importText(text, { target: PG.profiles.activeId() });
        this.setFlash(ok ? t("progress.import_ok") : t("progress.import_err", { reason }));
        if (ok) {
          PG.ach.backfillSilent();
          this.build();
        }
      });
    }

    setFlash(text) {
      this.flash = text;
      this.flashT = performance.now();
    }

    update() {}

    // ----- Zeichnen -------------------------------------------------------
    draw(ctx) {
      this.tip = null;
      this.buttons = [];
      ui.drawBackground(ctx, W, H, false);
      ui.drawTitle(ctx, W, t("progress.name"), { y: 40, big: ui.font(30, true), accent: ui.GOLD });

      for (const [r, key] of this.tabRects) {
        const active = key === this.tab;
        const hover = key === this.tabHover;
        PG.draw.rect(ctx, active || hover ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 13);
        PG.draw.rect(ctx, active ? ui.GOLD : ui.BORDER, r, 1, 13);
        ui.text(ctx, t("progress.tab_" + key), r.centerx, r.centery, this.tabFont,
          active ? ui.TEXT : ui.TEXT_DIM, "center");
      }

      this.drawLevelHead(ctx);

      ctx.save();
      ctx.beginPath();
      ctx.rect(0, this.top, W, this.bottomY - this.top);
      ctx.clip();
      for (let i = 0; i < this.rows.length; i++) {
        const [ry, rh, kind, data] = this.rows[i];
        const y = this.top + ry - this.scroll;
        if (y + rh < this.top - 120 || y > this.bottomY + 20) continue;
        const fn = this["draw_" + kind];
        if (fn) fn.call(this, ctx, y, rh, data, i);
      }
      ctx.restore();

      const vp = this.viewport();
      if (this.contentH > vp) {
        PG.draw.rect(ctx, ui.PANEL, [W - 10, this.top, 4, vp], 0, 2);
        const th = Math.max(24, (vp * vp) / this.contentH);
        const ty = this.top + ((vp - th) * this.scroll) / Math.max(1, this.contentH - vp);
        PG.draw.rect(ctx, ui.BORDER_LIGHT, [W - 10, ty, 4, th], 0, 2);
      }

      if (this.tip) PG.charts.tooltip(ctx, this.mouse, this.tip, W);
      let hint = t("progress.hint");
      if (this.flash && performance.now() - this.flashT < 4000) hint = this.flash;
      ui.drawFooter(ctx, W, H, hint);
    }

    drawLevelHead(ctx) {
      const [, points] = PG.stats.levelOf();
      const [lvl, inLevel, need] = PG.ach.levelProgress(points);
      const color = PG.ach.rankColor(lvl);
      const y = 84;
      PG.ach.drawBadge(ctx, [this.left + 16, y + 12], 15, "bolt", color, false);
      const nameFont = ui.font(15, true);
      ui.text(ctx, `${t("level.short")} ${lvl}  ·  ${PG.ach.rankTitle(lvl)}`, this.left + 38, y, nameFont, ui.TEXT);
      ui.text(ctx, `${t("progress.points", { n: points })}   ·   ${t("progress.next_level", { n: Math.max(0, need - inLevel), m: lvl + 1 })}`,
        this.left + 38, y + nameFont.height + 1, ui.font(11), ui.TEXT_DIM);

      const barW = Math.min(190, W / 4);
      const bx = this.right - barW;
      PG.draw.rect(ctx, ui.PANEL_LIGHT, [bx, y + 4, barW, 7], 0, 4);
      const fw = barW * Math.min(1, inLevel / Math.max(1, need));
      if (fw > 0) PG.draw.rect(ctx, color, [bx, y + 4, fw, 7], 0, 4);
      PG.draw.rect(ctx, ui.BORDER, [bx, y + 4, barW, 7], 1, 4);
      ui.text(ctx, t("progress.unlocked_of", { n: this.nUnlocked, m: this.nTotal }),
        this.right, y + 15, ui.font(11, true), ui.GOLD, "topright");
    }

    // -- Zeilen ------------------------------------------------------------
    draw_head(ctx, y, rh, data) {
      const img = ui.text(ctx, data, this.left, y + 8, ui.font(13, true), ui.ACCENT);
      PG.draw.line(ctx, ui.BORDER, [this.left + img.w + 12, y + 16], [this.right, y + 16]);
    }

    draw_dayHead(ctx, y, rh, data) {
      ui.text(ctx, data, this.left, y + 6, ui.font(11, true), ui.TEXT_FAINT);
    }

    draw_note(ctx, y, rh, data) {
      ui.text(ctx, data, this.left, y + 8, ui.font(13), ui.TEXT_FAINT);
    }

    draw_filter(ctx, y) {
      const fnt = ui.font(11, true);
      let x = this.left;
      for (const key of FILTERS) {
        const label = t("progress.filter_" + key);
        const w = fnt.width(label) + 18;
        const r = new PG.Rect(x, y, w, 22);
        const active = this.filter === key;
        PG.draw.rect(ctx, active ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 11);
        PG.draw.rect(ctx, active ? ui.ACCENT : ui.BORDER, r, 1, 11);
        ui.text(ctx, label, r.centerx, r.centery, fnt, active ? ui.TEXT : ui.TEXT_DIM, "center");
        this.buttons.push([r, () => this.setFilter(key)]);
        x += w + 6;
      }
      const counts = PG.ach.rarityCounts();
      x = this.right;
      for (const rarity of PG.ach.RARITY_ORDER.slice().reverse()) {
        const [got, total] = counts[rarity] || [0, 0];
        if (!total) continue;
        const label = `${got}/${total}`;
        const f = ui.font(10, true);
        x -= f.width(label) + 14;
        PG.draw.circle(ctx, PG.ach.rarityColor(rarity), [x - 6, y + 11], 4);
        ui.text(ctx, label, x, y + 11, f, PG.ach.rarityColor(rarity), "midleft");
        const box = new PG.Rect(x - 12, y, f.width(label) + 14, 22);
        if (box.collidepoint(this.mouse)) {
          this.tip = [PG.ach.rarityName(rarity), t("progress.rarity_points", { n: PG.ach.RARITY[rarity].points })];
        }
      }
    }

    draw_ach(ctx, y, rh, data, i) {
      const [d, ts, prog] = data;
      const unlocked = !!ts;
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      const hover = i === this.hoverRow;
      ui.drawPanel(ctx, row, { color: hover && d.game ? ui.PANEL_LIGHT : ui.PANEL, shadow: false, radius: 10 });

      const rarityCol = PG.ach.rarityColor(d);
      if (unlocked) PG.draw.rect(ctx, rarityCol, [row.x, row.y + 6, 3, rh - 12], 0, 2);
      PG.ach.drawBadge(ctx, [row.x + 26, row.centery], 15, d.icon, unlocked ? rarityCol : ui.BORDER_LIGHT, !unlocked);

      const nameFont = ui.font(15, true);
      ui.text(ctx, PG.ach.displayName(d), row.x + 50, row.y + 5, nameFont, unlocked ? ui.TEXT : ui.TEXT_DIM);
      ui.text(ctx, PG.ach.displayDesc(d), row.x + 50, row.y + 8 + nameFont.height, ui.font(12),
        unlocked ? ui.TEXT_DIM : ui.TEXT_FAINT);
      ui.text(ctx, "+" + PG.ach.pointsOf(d), row.right - 14, row.y + 6, ui.font(11, true),
        unlocked ? rarityCol : ui.TEXT_FAINT, "topright");

      if (unlocked) {
        ui.text(ctx, String(ts).slice(0, 10), row.right - 14, row.bottom - 6, ui.font(11), ui.TEXT_FAINT, "bottomright");
      } else if (prog) {
        const [cur, target] = prog;
        const bw = 86;
        const bx = row.right - 14 - bw;
        const byy = row.bottom - 14;
        const label = d.stat === "time"
          ? `${PG.stats.formatTime(cur)} / ${PG.stats.formatTime(target)}`
          : `${cur} / ${target}`;
        ui.text(ctx, label, row.right - 14, byy - 8, ui.font(11), ui.TEXT_FAINT, "midright");
        PG.draw.rect(ctx, ui.BTN, [bx, byy, bw, 5], 0, 3);
        const fw = (bw * cur) / Math.max(1, target);
        if (fw > 0) PG.draw.rect(ctx, ui.ACCENT_SOFT, [bx, byy, fw, 5], 0, 3);
      }
      if (hover && row.collidepoint(this.mouse)) {
        this.tip = [PG.ach.displayName(d), `${PG.ach.rarityName(d)}  ·  +${PG.ach.pointsOf(d)}`];
      }
    }

    draw_cards(ctx, y) {
      const tot = this.totals;
      const cards = [
        ["clock", PG.stats.formatTime(tot.time), t("progress.total_time")],
        ["flag", String(tot.plays), t("progress.total_plays")],
        ["medal", String(tot.wins), t("progress.total_wins")],
        ["trophy", String(tot.records), t("progress.total_records")],
        ["gem", `${tot.distinct} / ${this.app.entries.length}`, t("progress.distinct")],
        ["star", `${this.nUnlocked} / ${this.nTotal}`, t("progress.achievements")],
        ["calendar", String(tot.days || 0), t("progress.total_days")],
        ["bolt", String(tot.points || 0), t("progress.total_points")],
      ];
      const cols = this.cardCols;
      const gap = 10;
      const cw = (this.right - this.left - (cols - 1) * gap) / cols;
      cards.forEach(([icon, value, label], i) => {
        const rect = new PG.Rect(this.left + (i % cols) * (cw + gap), y + Math.floor(i / cols) * (this.cardH + gap), cw, this.cardH);
        ui.drawPanel(ctx, rect, { shadow: false, radius: 10 });
        PG.ach.drawBadge(ctx, [rect.x + 24, rect.centery], 13, icon, ui.ACCENT, false);
        const valFont = ui.font(19, true);
        ui.text(ctx, value, rect.x + 46, rect.y + 10, valFont, ui.TEXT);
        ui.text(ctx, label, rect.x + 46, rect.y + 14 + valFont.height, ui.font(11), ui.TEXT_DIM);
      });
      const rows = Math.ceil(cards.length / cols);
      const fy = y + rows * (this.cardH + gap) + 4;
      if (tot.favorite) {
        const f1 = ui.text(ctx, t("progress.favorite") + ":", this.left, fy, ui.font(13), ui.TEXT_DIM);
        ui.text(ctx, ` ${gameName(tot.favorite)} (${PG.stats.formatTime(PG.stats.get(tot.favorite).time)})`,
          this.left + f1.w, fy, ui.font(13, true), gameColor(tot.favorite));
      }
    }

    draw_chartTop(ctx, y, rh, data) {
      const rect = new PG.Rect(this.left, y + 4, this.right - this.left, rh - 8);
      const values = data.map(([, st]) => st.time);
      const hits = PG.charts.bars(ctx, rect, values, {
        labels: data.map(([e]) => PG.gameName(e)),
        colors: data.map(([e]) => ui.gameColor(e.id)),
        horizontal: true,
        fmt: (v) => PG.stats.formatTime(v),
        hover: this.hits.topHover,
      });
      this.track(hits, "top", (idx) => [
        PG.gameName(data[idx][0]),
        t("progress.col_time") + ": " + PG.stats.formatTime(values[idx]),
        t("progress.col_plays") + ": " + data[idx][1].plays,
      ]);
    }

    draw_chartSplit(ctx, y, rh) {
      const tot = this.totals;
      const wins = tot.wins;
      const losses = tot.losses;
      const cx = this.left + 80;
      const cy = y + rh / 2;
      const ratio = wins + losses ? (100 * wins) / (wins + losses) : 0;
      PG.charts.donut(ctx, [cx, cy], 60, [[wins, ui.GREEN], [losses, ui.mix(ui.PANEL_LIGHT, ui.RED, 0.8)]], {
        centerText: Math.round(ratio) + "%",
        centerSub: t("progress.win_ratio"),
      });
      const fnt = ui.font(12);
      const tx = cx + 84;
      let ry = y + 14;
      const rows = [
        [t("progress.total_wins"), String(wins)],
        [t("progress.total_losses"), String(losses)],
        [t("progress.total_records"), String(tot.records)],
        [t("progress.longest_session"), PG.stats.formatTime(tot.longest || 0)],
        [t("progress.duels"), String(tot.duels || 0)],
      ];
      for (const [label, value] of rows) {
        ui.text(ctx, label, tx, ry, fnt, ui.TEXT_DIM);
        ui.text(ctx, value, this.right - 8, ry, ui.font(12, true), ui.TEXT, "topright");
        ry += 24;
      }
    }

    tableColumns() {
      const w = this.right - this.left;
      const colW = Math.max(64, Math.min(92, w / 7));
      return [[0, 1, 2, 3].map((i) => this.right - 10 - i * colW), colW];
    }

    draw_tableHead(ctx, y) {
      const [xs] = this.tableColumns();
      const fnt = ui.font(11, true);
      ui.text(ctx, t("progress.col_game"), this.left + 8, y + 10, fnt, ui.TEXT_FAINT);
      ["col_best", "col_wins", "col_time", "col_plays"].forEach((key, i) => {
        ui.text(ctx, t("progress." + key), xs[i], y + 10, fnt, ui.TEXT_FAINT, "topright");
      });
      PG.draw.line(ctx, ui.BORDER, [this.left, y + 26], [this.right, y + 26]);
    }

    draw_game(ctx, y, rh, data, i) {
      const [e, st] = data;
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      if (i === this.hoverRow) ui.drawPanel(ctx, row, { color: ui.PANEL_LIGHT, shadow: false, radius: 8 });
      PG.draw.circle(ctx, ui.gameColor(e.id), [row.x + 12, row.centery], 4);
      ui.text(ctx, PG.gameName(e), row.x + 24, row.centery, ui.font(13, true), ui.TEXT, "midleft");
      const [xs] = this.tableColumns();
      const values = [
        String(this.scores[e.meta.key] || 0),
        st.wins || st.losses ? String(st.wins) : "-",
        PG.stats.formatTime(st.time),
        String(st.plays),
      ];
      values.forEach((val, idx) => ui.text(ctx, val, xs[idx], row.centery, ui.font(12), ui.TEXT_DIM, "midright"));
    }

    draw_wrapped(ctx, y, rh) {
      const rect = new PG.Rect(this.left, y, this.right - this.left, rh);
      ui.drawPanel(ctx, rect, { color: ui.PANEL_LIGHT, radius: 12, accentTop: ui.GOLD });
      const headFont = ui.font(14, true);
      ui.text(ctx, t("progress.wrapped_head"), rect.x + 14, rect.y + 6, headFont, ui.GOLD);
      let ty = rect.y + 8 + headFont.height;
      for (const fact of this.facts) {
        PG.draw.circle(ctx, ui.ACCENT, [rect.x + 20, ty + 8], 3);
        ui.text(ctx, fact, rect.x + 30, ty, ui.font(12), ui.TEXT_DIM);
        ty += 20;
      }
      const fnt = ui.font(11, true);
      let x = rect.right - 10;
      for (const [label, cb] of [[t("progress.import"), () => this.importStats()],
        [t("progress.export"), () => this.exportStats()]]) {
        const w = fnt.width(label) + 20;
        const r = new PG.Rect(x - w, rect.y + 8, w, 22);
        PG.draw.rect(ctx, ui.PANEL, r, 0, 11);
        PG.draw.rect(ctx, ui.BORDER_LIGHT, r, 1, 11);
        ui.text(ctx, label, r.centerx, r.centery, fnt, ui.TEXT_DIM, "center");
        this.buttons.push([r, cb]);
        x -= w + 6;
      }
    }

    draw_heat(ctx, y, rh) {
      const rect = new PG.Rect(this.left, y, this.right - this.left, rh - 20);
      const hits = PG.charts.calendarHeat(ctx, rect, this.dayTime, { color: ui.ACCENT, hover: this.hits.heatHover });
      this.hits.heatHover = null;
      for (const [box, [day, value]] of hits) {
        if (box.collidepoint(this.mouse)) {
          this.hits.heatHover = day;
          const entry = this.days[day] || {};
          this.tip = [prettyDate(day), t("progress.tip_time", { time: PG.stats.formatTime(value) }),
            t("progress.tip_plays", { n: entry.p || 0 })];
          break;
        }
      }
      PG.charts.heatLegend(ctx, new PG.Rect(rect.x, rect.bottom + 4, rect.w, 14),
        { color: ui.ACCENT, less: t("progress.less"), more: t("progress.more") });
    }

    draw_streaks(ctx, y, rh) {
      const cards = [
        [String(this.streak.cur), t("progress.streak_cur"), "flame"],
        [String(this.streak.best), t("progress.streak_best"), "flame"],
        [String(this.totals.days || 0), t("progress.total_days"), "calendar"],
      ];
      const gap = 10;
      const cw = (this.right - this.left - gap * (cards.length - 1)) / cards.length;
      cards.forEach(([value, label, icon], i) => {
        const rect = new PG.Rect(this.left + i * (cw + gap), y, cw, rh);
        ui.drawPanel(ctx, rect, { shadow: false, radius: 10 });
        PG.ach.drawBadge(ctx, [rect.x + 20, rect.centery], 11, icon, i === 1 ? ui.GOLD : ui.ACCENT, false);
        const f = ui.font(15, true);
        ui.text(ctx, value, rect.x + 38, rect.centery - f.height + 2, f, ui.TEXT);
        ui.text(ctx, label, rect.x + 38, rect.centery + 2, ui.font(10), ui.TEXT_DIM);
      });
    }

    draw_weekdays(ctx, y, rh) {
      const rect = new PG.Rect(this.left, y, this.right - this.left, rh);
      const values = PG.stats.weekdayList();
      const labels = PG.charts.weekdayNames();
      const hits = PG.charts.bars(ctx, rect, values, {
        labels, color: ui.ACCENT, fmt: (v) => PG.stats.formatTime(v), hover: this.hits.wdHover,
      });
      this.track(hits, "wd", (idx) => [labels[idx] || "", PG.stats.formatTime(values[idx])]);
    }

    draw_clock(ctx, y, rh) {
      const hours = PG.stats.hoursList();
      const cx = this.left + (this.right - this.left) / 2;
      const cy = y + rh / 2;
      const hits = PG.charts.clockRing(ctx, [cx, cy], Math.min(80, rh / 2 - 14), hours,
        { color: ui.ACCENT, hover: this.hits.clockHover });
      this.track(hits, "clock", (idx) => [
        t("progress.tip_hour", { h: idx, h2: (idx + 1) % 24 }), PG.stats.formatTime(hours[idx]),
      ]);
      if (hours.some((v) => v > 0)) {
        const peak = hours.indexOf(Math.max(...hours));
        ui.text(ctx, t("progress.fact_peak_hour", { h: peak, h2: (peak + 1) % 24 }),
          cx, y + rh - 16, ui.font(11), ui.TEXT_DIM, "midtop");
      }
    }

    draw_seriesPick(ctx, y) {
      const fnt = ui.font(11, true);
      const label = ui.text(ctx, t("progress.pick_game") + ":", this.left, y + 5, ui.font(11), ui.TEXT_FAINT);
      let x = this.left + label.w + 8;
      for (const key of this.playedSeries) {
        const e = gameEntry(key);
        if (!e) continue;
        const name = PG.gameName(e);
        const w = fnt.width(name) + 16;
        if (x + w > this.right) break;
        const r = new PG.Rect(x, y, w, 22);
        const active = key === this.seriesGame;
        PG.draw.rect(ctx, active ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 11);
        PG.draw.rect(ctx, active ? ui.gameColor(e.id) : ui.BORDER, r, 1, 11);
        ui.text(ctx, name, r.centerx, r.centery, fnt, active ? ui.TEXT : ui.TEXT_DIM, "center");
        this.buttons.push([r, () => this.pickSeries(key)]);
        x += w + 6;
      }
    }

    draw_chartSeries(ctx, y, rh) {
      const key = this.seriesGame;
      if (!key) return;
      const series = PG.stats.seriesOf(key);
      if (!series.length) return;
      const rect = new PG.Rect(this.left, y + 4, this.right - this.left, rh - 8);
      const values = series.map((p) => p[1]);
      const marks = series.map((p, i) => (p[2] ? i : -1)).filter((i) => i >= 0);
      const hits = PG.charts.line(ctx, rect, values, {
        color: gameColor(key), marks, fmt: (v) => String(Math.round(v)), hover: this.hits.seriesHover,
      });
      this.track(hits, "series", (idx) => [
        String(series[idx][1]),
        prettyDate(series[idx][0].slice(0, 10)) + " " + series[idx][0].slice(11, 16),
        series[idx][2] ? t("progress.new_record") : "",
      ]);
    }

    draw_event(ctx, y, rh, ev) {
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      const stamp = String(ev.t || "").slice(11, 16);
      let icon = "star";
      let color = ui.ACCENT;
      let text = "";
      let extra = "";
      if (ev.k === "ach") {
        const d = PG.ach.byId(ev.id);
        if (d) {
          icon = d.icon;
          color = PG.ach.rarityColor(d);
          text = t("progress.ev_ach", { name: PG.ach.displayName(d) });
          extra = "+" + PG.ach.pointsOf(d);
        } else text = t("progress.ev_ach", { name: ev.id });
      } else if (ev.k === "rec") {
        icon = "trophy";
        color = ui.GOLD;
        text = t("progress.ev_rec", { game: gameName(ev.g), n: ev.v });
      } else if (ev.k === "lvl") {
        icon = "bolt";
        color = PG.ach.rankColor(ev.v || 1);
        text = t("progress.ev_lvl", { n: ev.v, rank: PG.ach.rankTitle(ev.v || 1) });
      } else if (ev.k === "duel") {
        icon = "swords";
        color = ui.ACCENT2;
        text = t("progress.ev_duel", {
          game: gameName(ev.g), name: PG.profiles.nameOf(ev.o),
          result: ev.w ? t("progress.ev_won") : t("progress.ev_lost"),
        });
      }
      PG.ach.drawBadge(ctx, [row.x + 14, row.centery], 10, icon, color, false);
      ui.text(ctx, text, row.x + 32, row.centery, ui.font(12), ui.TEXT_DIM, "midleft");
      if (extra) ui.text(ctx, extra, row.right - 54, row.centery, ui.font(11, true), color, "midright");
      ui.text(ctx, stamp, row.right - 8, row.centery, ui.font(11), ui.TEXT_FAINT, "midright");
    }

    draw_profile(ctx, y, rh, p) {
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      const active = p.id === PG.profiles.activeId();
      ui.drawPanel(ctx, row, { color: active ? ui.PANEL_LIGHT : ui.PANEL, shadow: false, radius: 10 });
      const color = ui.hexToRgb(PG.profiles.colorHex(p.id));
      PG.ach.drawBadge(ctx, [row.x + 28, row.centery], 17, PG.profiles.iconOf(p.id), color, false);
      const nameFont = ui.font(15, true);
      ui.text(ctx, PG.profiles.nameOf(p.id), row.x + 54, row.y + 8, nameFont, ui.TEXT);
      const tot = this.pstats[p.id] || {};
      const level = tot.level || 1;
      ui.text(ctx, `${t("level.short")} ${level} · ${PG.ach.rankTitle(level)} · ${t("progress.points", { n: tot.points || 0 })} · ${PG.stats.formatTime(tot.time || 0)}`,
        row.x + 54, row.y + 10 + nameFont.height, ui.font(11), ui.TEXT_DIM);
      if (active) ui.text(ctx, t("profile.active"), row.right - 12, row.y + 8, ui.font(10, true), ui.GOLD, "topright");
    }

    draw_board(ctx, y, rh, data, i) {
      const [key, values] = data;
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      if (i === this.hoverRow) ui.drawPanel(ctx, row, { color: ui.PANEL_LIGHT, shadow: false, radius: 8 });
      PG.draw.circle(ctx, gameColor(key), [row.x + 12, row.centery], 4);
      ui.text(ctx, gameName(key), row.x + 24, row.centery, ui.font(13, true), ui.TEXT, "midleft");
      let x = row.x + 190;
      values.slice(0, 3).forEach(([pid, score], rank) => {
        const col = ui.hexToRgb(PG.profiles.colorHex(pid));
        const label = `${PG.profiles.nameOf(pid)} ${score}`;
        const f = ui.font(11, rank === 0);
        PG.draw.circle(ctx, col, [x, row.centery], 3);
        ui.text(ctx, label, x + 8, row.centery, f, rank === 0 ? ui.GOLD : ui.TEXT_DIM, "midleft");
        x += f.width(label) + 24;
      });
    }

    draw_duel(ctx, y, rh, data) {
      const [key, me, other, wins, losses] = data;
      const row = new PG.Rect(this.left, y, this.right - this.left, rh);
      PG.draw.circle(ctx, gameColor(key), [row.x + 12, row.centery], 4);
      ui.text(ctx, gameName(key), row.x + 24, row.centery, ui.font(13, true), ui.TEXT, "midleft");
      const label = `${PG.profiles.nameOf(me)}  ${wins} : ${losses}  ${PG.profiles.nameOf(other)}`;
      ui.text(ctx, label, row.right - 10, row.centery, ui.font(12, true),
        wins > losses ? ui.GOLD : ui.TEXT_DIM, "midright");
    }

    track(hits, name, tipFn) {
      this.hits[name] = hits;
      for (const [rect, idx] of hits) {
        if (rect.collidepoint(this.mouse)) {
          this.hits[name + "Hover"] = idx;
          const lines = tipFn(idx);
          if (lines) this.tip = lines;
          return;
        }
      }
      this.hits[name + "Hover"] = null;
    }
  }

  PG.ProgressScreen = ProgressScreen;
})();
