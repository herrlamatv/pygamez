/*
 * geodash_extra_ui.js - Reiter "Weitere" (Port von games/geodash_extra.py)
 * ========================================================================
 * Die Level-Bibliothek: 80 Level in zehn Kategorien, erzeugt und vom Solver
 * bewiesen von devtools/build_geodash_extra.py.
 *
 * Geladen wird in zwei Stufen: geodash_extra_index.js (window.PG.gdExtraIndex,
 * nur Name/Kategorie/Länge - kommt mit dem Spiel) und geodash_extra.js
 * (window.PG.gdExtra, die Objekte aller Level) - letzteres erst, wenn man den
 * Reiter öffnet. Bis es da ist, zeigt die Liste schon alles an; gespielt wird,
 * sobald die Volldaten stehen.
 *
 * Fortschritt: Bestwert, Münzen und Versuche je Level unter "x:<id>" (siehe
 * GeometryDashGame.bestKey). Sterne und Erfolge bleiben der Kampagne.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const core = PG.gdCore;
  const gdraw = PG.gdDraw;
  const ugc = PG.ugc;
  const GAME = "geodash";

  const CATS = ["all", "starter", "rhythm", "coins", "ship", "ufo", "ball", "gravity", "speed", "wave", "demon"];
  const BUTTONS = ["play", "template", "random"];
  const DIFF_KEYS = ["easy", "normal", "hard", "harder", "insane", "demon"];

  const R = (x, y, w, h) => new PG.Rect(Math.trunc(x), Math.trunc(y), Math.trunc(w), Math.trunc(h));
  const diffKey = (d) => DIFF_KEYS[Math.max(0, Math.min(DIFF_KEYS.length - 1, Math.trunc(d) || 0))];
  const coinsOf = (mask) => [1, 2, 4].filter((b) => mask & b).length;

  function fit(font, text, w) {
    if (font.width(text) <= w) return text;
    let s = text;
    while (s.length > 1 && font.width(s + "...") > w) s = s.slice(0, -1);
    return s + "...";
  }

  function btn(ctx, rc, text, fnt, on = false, enabled = true) {
    draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
    draw.rect(ctx, on ? ui.ACCENT : ui.BORDER, rc, on ? 2 : 1, 7);
    ui.text(ctx, fit(fnt, text, rc.w - 8), rc.centerx, rc.centery, fnt, enabled || on ? ui.TEXT : ui.TEXT_FAINT, "center");
  }

  // ---------------------------------------------------------------- Daten
  // Pfad der Volldaten - aus dem Pfad DIESER Datei abgeleitet, damit er
  // sowohl aus index.html als auch aus tools/smoketest.html stimmt.
  const FULL_SRC = (function () {
    const src = (document.currentScript && document.currentScript.src) || "";
    return src ? src.replace(/geodash_extra_ui\.js(\?.*)?$/, "geodash_extra.js")
               : "js/games/geodash_extra.js";
  })();

  let loading = false;
  const waiting = [];

  /** Volldaten (Objekte) nachladen; 'done' läuft, sobald sie stehen. */
  function loadFull(done) {
    if (PG.gdExtra) return done && done();
    if (done) waiting.push(done);
    if (loading) return;
    loading = true;
    PG.gdExtraReady = () => {
      loading = false;
      while (waiting.length) waiting.shift()();
    };
    const s = document.createElement("script");
    s.src = FULL_SRC;
    s.onerror = () => {
      console.warn("[PyGameZ] Level-Bibliothek fehlt:", s.src);
      loading = false;
      PG.gdExtra = PG.gdExtra || {};
      while (waiting.length) waiting.shift()();
    };
    document.head.appendChild(s);
  }

  /** Ein Level der Bibliothek mit allen Objekten (oder null). */
  function fullLevel(id) {
    const d = PG.gdExtra && PG.gdExtra[id];
    return d ? core.normalizeLevel(d) : null;
  }

  // ===========================================================================
  //  Die Liste
  // ===========================================================================
  class ExtraList {
    constructor(game) {
      this.game = game;
      this.levels = [];
      this.view = [];
      this.cat = "all";
      this.query = new ui.TextInput("", 20, null, t("gd.more.search"));
      this.searchFocus = false;
      this.sel = 0;
      this.first = 0;
      this.toast = "";
      this.toastT = 0;
      this.hay = [];
      this.pending = null;          // id, das gespielt werden soll, sobald geladen
      this.reload();
      this.layout();
      const cat = game.opts.last_cat;
      if (CATS.includes(cat)) this.cat = cat;
      this.filter();
      loadFull(() => this.afterLoad());
    }

    // ----- Daten --------------------------------------------------------
    reload() {
      if (!this.levels.length) {
        this.levels = (PG.gdExtraIndex || []).map((d) => ({
          id: d.id, name: d.name, category: d.cat, difficulty: d.difficulty,
          stars: d.stars, speed: d.speed, length: d.length, bg: d.bg, ground: d.ground,
        }));
        // Suchtext je Level einmal vorbereiten (Filtern kostet dann nichts).
        this.hay = this.levels.map((d) => (d.name + " " + d.id + " " + d.category).toLowerCase());
      }
      this.filter();
    }

    /** Volldaten sind da: ein wartendes Level starten. */
    afterLoad() {
      const id = this.pending;
      this.pending = null;
      if (id) {
        const d = fullLevel(id);
        if (d) this.game.startLevel(d, "extra");
      }
    }

    filter() {
      const q = this.query.text.trim().toLowerCase();
      this.view = [];
      for (let i = 0; i < this.levels.length; i++) {
        if (this.cat !== "all" && this.levels[i].category !== this.cat) continue;
        if (q && !this.hay[i].includes(q)) continue;
        this.view.push(i);
      }
      this.sel = Math.max(0, Math.min(this.sel, Math.max(0, this.view.length - 1)));
      this.clamp();
    }

    selected() {
      return this.view.length ? this.levels[this.view[this.sel]] : null;
    }

    hasNext() {
      return this.view.length > 0 && this.sel < this.view.length - 1;
    }

    playNext() {
      if (this.hasNext()) {
        this.sel++;
        this.clamp();
        this.action("play");
      }
    }

    /** Wie viele Level der Bibliothek sind schon geschafft? */
    doneCount() {
      const best = this.game.best;
      let n = 0;
      for (const d of this.levels) if ((best["x:" + d.id] || [0, 0])[0] >= 100) n++;
      return n;
    }

    // ----- Layout -------------------------------------------------------
    layout() {
      const g = this.game, w = g.width, h = g.height;
      this.fnt = ui.font(Math.max(13, Math.floor(h / 30)));
      this.tiny = ui.font(Math.max(10, Math.floor(h / 40)));
      const fh = this.fnt.getHeight(), th = this.tiny.getHeight();
      this.twoLines = h >= 400;
      this.rowH = this.twoLines ? fh + th + 10 : Math.max(26, fh + 12);

      // Kategorie-Leiste (eine oder zwei Zeilen, je nach Breite)
      const cols = w >= 720 ? 6 : 4;
      const ch = Math.max(20, Math.min(28, Math.floor(h / 18)), th + 8);
      const gap = 5;
      const cw = (w - 24 - gap * (cols - 1)) / cols;
      const rowsC = Math.ceil(CATS.length / cols);
      const top = g.tabBottom + 6;
      this.catRects = {};
      CATS.forEach((key, i) => {
        const r = Math.floor(i / cols), c = i % cols;
        this.catRects[key] = R(12 + c * (cw + gap), top + r * (ch + gap), cw, ch);
      });
      const catBottom = top + rowsC * (ch + gap);

      // Suchfeld
      const ff = Math.max(22, Math.min(30, Math.floor(h / 15)), th + 10);
      this.searchRect = R(12, catBottom, w - 24, ff);

      // Knopfleiste unten
      const bh = Math.max(24, Math.min(34, Math.floor(h / 13)), th + 12);
      const bw = (w - 24 - gap * (BUTTONS.length - 1)) / BUTTONS.length;
      const bottom = h - 12;
      this.btnRects = {};
      BUTTONS.forEach((key, i) => (this.btnRects[key] = R(12 + i * (bw + gap), bottom - bh, bw, bh)));
      this.listTop = this.searchRect.bottom + 6;
      this.listBottom = bottom - bh - th - 8;
      this.rowsVisible = Math.max(1, Math.floor((this.listBottom - this.listTop - 4) / this.rowH));
      this.listW = w - 30;
      this.clamp();
    }

    clamp() {
      const n = this.view.length, vis = this.rowsVisible || 1;
      this.first = Math.max(0, Math.min(this.first, Math.max(0, n - vis)));
      if (this.sel < this.first) this.first = this.sel;
      else if (this.sel >= this.first + vis) this.first = this.sel - vis + 1;
    }

    setToast(text) {
      this.toast = text;
      this.toastT = 2.8;
    }

    update(dt) {
      if (this.toastT > 0) {
        this.toastT -= dt;
        if (this.toastT <= 0) this.toast = "";
      }
    }

    // ----- Eingabe ------------------------------------------------------
    handle(ev) {
      const g = this.game;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.searchFocus) {
          if (k === "Return" || k === "KP_Enter") {
            this.searchFocus = false;
            return;
          }
          if (k === "Escape") {
            this.query.setText("");
            this.searchFocus = false;
            this.filter();
            return;
          }
          if (this.query.handle(ev)) {
            this.filter();
            return;
          }
        }
        if (k === "Up" || k === "Left" || g.isAction(k, "up")) this.move(-1);
        else if (k === "Down" || k === "Right" || g.isAction(k, "down")) this.move(1);
        else if (k === "Prior") this.move(-this.rowsVisible);
        else if (k === "Next") this.move(this.rowsVisible);
        else if (k === "Return" || k === "space" || k === "KP_Enter") this.action("play");
        else if (k === "t" || k === "T") this.action("template");
        else if (k === "r" || k === "R") this.action("random");
        else if (k === "slash" || k === "f" || k === "F") this.searchFocus = true;
        else if (k === "c" || k === "C") this.stepCat(1);
        return;
      }
      if (ev.kind === "wheel") {
        this.first = Math.max(0, Math.min(Math.max(0, this.view.length - this.rowsVisible), this.first - ev.delta));
        return;
      }
      if (ev.kind === "mousedown") {
        const p = ev.pos;
        for (const [key, rc] of Object.entries(this.catRects)) if (rc.collidepoint(p)) return this.setCat(key);
        if (this.searchRect.collidepoint(p)) {
          this.searchFocus = true;
          return;
        }
        this.searchFocus = false;
        for (const [key, rc] of Object.entries(this.btnRects)) if (rc.collidepoint(p)) return this.action(key);
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.view.length) break;
          if (this.rowRect(i).collidepoint(p)) {
            if (this.sel === idx) return this.action("play");
            this.sel = idx;
            this.game.playSound("move");
            return;
          }
        }
      }
    }

    rowRect(i) {
      return R(12, this.listTop + i * this.rowH, this.listW, this.rowH - 3);
    }

    move(d) {
      if (!this.view.length) return;
      this.sel = Math.max(0, Math.min(this.view.length - 1, this.sel + d));
      this.clamp();
      this.game.playSound("move");
    }

    setCat(key) {
      if (!CATS.includes(key) || key === this.cat) return;
      this.cat = key;
      this.sel = 0;
      this.first = 0;
      this.filter();
      this.game.saveSetting("last_cat", key);
      this.game.playSound("click");
    }

    stepCat(d) {
      this.setCat(CATS[PG.mod(CATS.indexOf(this.cat) + d, CATS.length)]);
    }

    action(key) {
      const g = this.game;
      if (key === "random") {
        if (this.view.length) {
          this.sel = Math.floor(Math.random() * this.view.length);
          this.clamp();
          this.action("play");
        }
        return;
      }
      const d = this.selected();
      if (!d) return;
      if (key === "play") {
        g.playSound("click");
        const full = fullLevel(d.id);
        if (full) g.startLevel(full, "extra");
        else {
          // Volldaten sind noch unterwegs - das Level startet, sobald sie da sind.
          this.pending = d.id;
          this.setToast(t("gd.more.loading"));
          loadFull(() => this.afterLoad());
        }
      } else if (key === "template") this.copyAsTemplate(d);
    }

    /** Ein Level in die eigenen Level kopieren - dort baut man es um. */
    copyAsTemplate(d) {
      const g = this.game;
      if (ugc.isFull(GAME)) return this.setToast(t("gd.ugc.err.full", { max: ugc.maxItems(GAME) }));
      const full = fullLevel(d.id);
      if (!full) {
        this.setToast(t("gd.more.loading"));
        loadFull(() => {});
        return;
      }
      const m = PG.gdEdit.newLevel(d.name || "", "");
      m.id = ugc.uniqueId(ugc.slug(d.id || "level"), GAME);
      m.objects = (full.objects || []).map((o) => o.slice());
      for (const key of ["speed", "mode", "bg", "ground", "music", "length"]) if (key in full) m[key] = full[key];
      m.from = d.id || "";
      const [ok] = ugc.saveMap(m, GAME);
      if (!ok) return this.setToast(t("gd.ugc.err.full", { max: ugc.maxItems(GAME) }));
      g.playSound("click");
      if (g.lists) g.lists.reload();
      g.ugcEdit(m);
    }

    // ----- Zeichnen -----------------------------------------------------
    draw(ctx) {
      const g = this.game;
      for (const [key, rc] of Object.entries(this.catRects)) btn(ctx, rc, t("gd.more.cat." + key), this.tiny, key === this.cat);
      this.query.draw(ctx, this.searchRect, this.fnt, this.searchFocus);
      if (!this.view.length) this.drawEmpty(ctx);
      else {
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.view.length) break;
          this.drawRow(ctx, this.levels[this.view[idx]], idx, this.rowRect(i));
        }
        if (this.view.length > this.rowsVisible) this.drawScrollbar(ctx);
      }
      const line = t("gd.more.count", { n: this.view.length, max: this.levels.length }) + "   ·   " + t("gd.more.done", { n: this.doneCount(), max: this.levels.length });
      ui.text(ctx, line, 12, this.listBottom + 2, this.tiny, ui.TEXT_DIM);
      const has = !!this.selected();
      for (const [key, rc] of Object.entries(this.btnRects)) btn(ctx, rc, t("gd.more.btn_" + key), this.tiny, false, has);
      if (this.toast) {
        const w = this.tiny.width(this.toast) + 26, h = this.tiny.getHeight() + 12;
        const r = R((g.width >> 1) - (w >> 1), this.listBottom - h - 2, w, h);
        ui.drawPanel(ctx, r, { radius: h >> 1, shadow: false });
        draw.rect(ctx, ui.ACCENT2, r, 1, h >> 1);
        ui.text(ctx, this.toast, r.centerx, r.centery, this.tiny, ui.TEXT, "center");
      }
    }

    drawEmpty(ctx) {
      const g = this.game;
      const box = R(24, this.listTop + 10, g.width - 48, Math.max(50, this.listBottom - this.listTop - 24));
      ui.drawPanel(ctx, box, { shadow: false });
      const txt = t("gd.more.empty");
      const f = this.fnt.width(txt) > box.w - 16 ? this.tiny : this.fnt;
      ui.text(ctx, txt, box.centerx, box.centery, f, ui.TEXT_DIM, "center");
    }

    drawRow(ctx, d, idx, rc) {
      const g = this.game;
      const sel = idx === this.sel;
      draw.rect(ctx, sel ? ui.PANEL_LIGHT : ui.PANEL, rc, 0, 7);
      draw.rect(ctx, sel ? g.accent : ui.BORDER, rc, sel ? 2 : 1, 7);
      const pad = 10;
      const diff = Math.trunc(d.difficulty) || 0;
      const col = gdraw.DIFF_COLORS[Math.max(0, Math.min(gdraw.DIFF_COLORS.length - 1, diff))];
      draw.rect(ctx, col, R(rc.x + 5, rc.y + 5, 5, rc.h - 10), 0, 2);
      const x = rc.x + pad + 6;
      ui.text(ctx, d.name || "", x, rc.y + 3, this.fnt, ui.TEXT);
      const key = "x:" + d.id;
      const best = (g.best[key] || [0, 0])[0];
      const mask = g.coins[key] || 0;
      const sub = [t("gd.more.cat." + (d.category || "all")), t("gd.diff." + diffKey(diff)),
        t("gd.ugc.length", { n: Math.trunc(d.length) || 0 })].join("  ·  ")
        + "  ·  " + t("gd.coins_of", { n: coinsOf(mask & 7), max: 3 });
      if (this.twoLines) {
        ui.text(ctx, fit(this.tiny, sub, rc.w - 90), x, rc.bottom - this.tiny.getHeight() - 3, this.tiny, ui.TEXT_DIM);
      }
      ui.text(ctx, best + "%", rc.right - pad, rc.centery, this.fnt, best >= 100 ? gdraw.COL_P1 : ui.TEXT_DIM, "midright");
    }

    drawScrollbar(ctx) {
      const track = R(this.listW + 14, this.listTop, 4, this.rowsVisible * this.rowH);
      draw.rect(ctx, ui.PANEL, track, 0, 2);
      const h = Math.max(24, Math.trunc(track.h * (this.rowsVisible / this.view.length)));
      const pos = this.first / Math.max(1, this.view.length - this.rowsVisible);
      const y = track.y + Math.trunc((track.h - h) * Math.min(1, pos));
      draw.rect(ctx, ui.BORDER_LIGHT, R(track.x, y, 4, h), 0, 2);
    }
  }

  PG.gdExtraUI = { CATS, BUTTONS, ExtraList, loadFull, fullLevel };
})();
