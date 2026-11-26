/*
 * matchview.js - der Matches-Screen eines Spiels (Port von matchview.py)
 * ============================================================================
 * Geöffnet über den Knopf "Matches" im Vorspiel-Screen (app.js PreGameScreen):
 *
 * - Reiter "Letzte Matches": Verlaufskurve der letzten 30 Ergebnisse (Rekorde
 *   markiert, Tooltip beim Überfahren) und darunter die letzten 10 Matches.
 * - Reiter "Alle Matches"  : alle gespeicherten Matches (PG.matches.CAP je
 *   Spiel), Filter nach Modus (Taste M), Sortierung neueste/beste (Taste O),
 *   Bildlauf mit Pfeilen/Mausrad/Bild auf/ab.
 *
 * Über beiden Reitern stehen die Summen (Matches, Siege, Bestwert, Schnitt,
 * Spielzeit) - sie laufen unbegrenzt weiter, auch wenn alte Einträge aus der
 * Liste fallen. Daten: matches.js.
 */
(function () {
  "use strict";

  const PG = window.PG;
  const ui = PG.ui;
  const draw = PG.draw;
  const t = PG.t;

  const ROW_H = 48;
  const TABS = ["recent", "all"];
  const CHART_N = 30;
  const RECENT_N = 10;

  /** Python-round (Hälfte zur geraden Zahl) -> int. */
  function pyRound(v) {
    const f = Math.floor(v);
    const diff = v - f;
    if (diff > 0.5) return f + 1;
    if (diff < 0.5) return f;
    return f % 2 === 0 ? f : f + 1;
  }

  function pad2(n) {
    return String(n).padStart(2, "0");
  }

  /** Anzeigename eines Modus (aus den Modi des Spiels, sonst Einzel/Mehr). */
  function modeLabel(entry, mode) {
    for (const [key, label] of (entry && entry.meta && entry.meta.modes) || []) {
      if (key === mode) return PG.hasT(label) ? t(label) : PG.pick(label);
    }
    if (mode === "single") return t("pregame.single");
    if (mode === "multi") return t("pregame.multi");
    return mode || "-";
  }

  /** Ganzzahl mit Tausenderpunkten (sprachneutral, wie im HUD). */
  function fmtNum(value) {
    value = Math.trunc(Number(value) || 0);
    const sign = value < 0 ? "-" : "";
    const txt = String(Math.abs(value)).replace(/\B(?=(\d{3})+(?!\d))/g, ".");
    return sign + txt;
  }

  /** Punkte eines Eintrags - Chip-Spiele mit Vorzeichen (Gewinn/Verlust). */
  function fmtScore(e) {
    const score = e[2];
    const info = e[6] || {};
    if ("bet" in info) return (score > 0 ? "+" : "") + fmtNum(score);
    return fmtNum(score);
  }

  /** Kürzt einen Text mit '…', bis er in 'room' Pixel passt. */
  function fitText(fnt, text, room) {
    if (fnt.width(text) <= room) return text;
    while (text && fnt.width(text + "…") > room) text = text.slice(0, -1);
    return text ? text.trimEnd() + "…" : "";
  }

  /** Wert wie Pythons str() (bool -> True/False). */
  function pyStr(v) {
    if (v === true) return "True";
    if (v === false) return "False";
    return String(v);
  }

  /** Detail-Texte eines Eintrags (Reihenfolge wie PG.matches.INFO_KEYS). */
  function infoParts(entry, info) {
    const out = [];
    if (!info) return out;
    const cls = entry && entry.cls;
    const custom = cls && typeof cls.matchText === "function" ? cls.matchText.bind(cls) : null;
    for (const [key, labelKey] of PG.matches.INFO_KEYS) {
      if (!(key in info)) continue;
      const value = info[key];
      let text = custom ? custom(key, value) : null;
      if (text == null) {
        if (key === "day") text = t(labelKey, { v: PG.seedrand.dateOf(value) });
        else if (key === "x" || key === "cr") {
          const v = Math.max(0, Math.trunc(Number(value) || 0));
          text = t(labelKey, { v: `${Math.floor(v / 100)}.${pad2(v % 100)}` });
        } else if (key === "tm") text = t(labelKey, { v: PG.matches.formatDuration(value) });
        else if (key === "bet" || key === "win" || key === "c") text = t(labelKey, { v: fmtNum(value) });
        else text = t(labelKey, { v: pyStr(value) });
      }
      if (text) out.push(text);
    }
    return out;
  }

  /** pygame-Mitte (ganzzahlig wie Rect.center). */
  function cxOf(r) {
    return r.x + Math.floor(r.w / 2);
  }
  function cyOf(r) {
    return r.y + Math.floor(r.h / 2);
  }

  /** Letzte Matches / Alle Matches eines Spiels. */
  class MatchScreen {
    constructor(app, entry, onClose, tab = "recent") {
      this.app = app;
      this.entry = entry;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.key = entry.meta.key;
      this.accent = ui.gameColor(entry.id);
      this.onClose = onClose || null;
      this.tab = TABS.indexOf(tab) >= 0 ? tab : "recent";
      this.filter = null; // null = alle Modi, sonst Modus-Schlüssel
      this.sortBest = false;
      this.first = 0;
      this.sel = 0;
      this.hover = null; // [art, wert] unter der Maus
      this.chartHits = [];
      this.chartChron = [];
      this.mouse = null;
      this.width = PG.W;
      this.height = PG.H;
      this.name = t("match.name");
      this.load();
      this.build();
    }

    update() {}

    get wantsEscape() {
      return true;
    }

    isAction(key, action) {
      return ["p1", "p2"].some((p) => (PG.CONTROLS[p] || {})[action] === key);
    }

    // ----- Daten & Layout -----------------------------------------------
    load() {
      this.all = PG.matches.entries(this.key);
      this.tot = PG.matches.totals(this.key);
      this.modes = PG.matches.modesOf(this.key);
      if (this.modes.indexOf(this.filter) < 0) this.filter = null;
      this.applyView();
    }

    applyView() {
      let items = this.all;
      if (this.filter !== null) items = items.filter((e) => e[1] === this.filter);
      if (this.sortBest) {
        // Stabil: bei gleichen Punkten bleibt das neuere vorn.
        items = items.slice().sort((a, b) => b[2] - a[2]);
      }
      this.items = items;
      this.sel = Math.min(this.sel, Math.max(0, this.items.length - 1));
      this.first = Math.min(this.first, Math.max(0, this.items.length - 1));
    }

    build() {
      const W = this.width, H = this.height;
      this.left = Math.max(24, Math.floor(W / 2) - 340);
      this.right = W - this.left;
      this.bottom = H - 40;
      // Reiter mittig unter dem Titel
      this.tabFont = ui.font(14, true);
      const labels = TABS.map((k) => t("match.tab_" + k));
      const widths = labels.map((lb) => this.tabFont.width(lb) + 30);
      const total = widths.reduce((a, b) => a + b, 0) + 8 * (widths.length - 1);
      let x = Math.floor(W / 2) - Math.floor(total / 2);
      this.tabRects = [];
      TABS.forEach((k, i) => {
        this.tabRects.push([new PG.Rect(x, 92, widths[i], 26), k, labels[i]]);
        x += widths[i] + 8;
      });
      // Summen-Leiste
      this.sumRect = new PG.Rect(this.left, 126, this.right - this.left, 44);
      const top = this.sumRect.bottom + 10;
      if (this.tab === "recent") {
        const ch = Math.max(64, Math.min(130, Math.floor((this.bottom - top) / 3)));
        this.chartRect = new PG.Rect(this.left, top, this.right - this.left, ch);
        this.top = this.chartRect.bottom + 10;
        this.filterRects = [];
        this.sortRect = null;
      } else {
        this.chartRect = null;
        // Filter-Chips (Alle + jeder Modus) links, Sortierung rechts
        const fnt = ui.font(12, true);
        this.chipFont = fnt;
        const sortLb = this.sortBest ? t("match.sort_best") : t("match.sort_new");
        const sw = fnt.width(sortLb) + 24;
        this.sortRect = new PG.Rect(this.right - sw, top, sw, 24);
        this.filterRects = [];
        let fx = this.left;
        const limit = this.sortRect.x - 8;
        for (const mode of [null].concat(this.modes)) {
          const lb = mode === null ? t("match.filter_all") : modeLabel(this.entry, mode);
          const w = fnt.width(lb) + 24;
          if (fx + w > limit) break;
          this.filterRects.push([new PG.Rect(fx, top, w, 24), mode, lb]);
          fx += w + 6;
        }
        this.headY = top + 24 + 14;
        this.top = top + 24 + 28;
      }
      this.rowsVisible = Math.max(1, Math.floor((this.bottom - this.top) / ROW_H));
      this.rowRects = [];
      for (let i = 0; i < this.rowsVisible; i++) {
        this.rowRects.push(new PG.Rect(this.left, this.top + i * ROW_H, this.right - this.left, ROW_H - 6));
      }
    }

    /** Einträge, die im aktiven Reiter als Zeilen erscheinen. */
    rows() {
      if (this.tab === "recent") return this.all.slice(0, RECENT_N);
      return this.items;
    }

    // ----- Eingabe ------------------------------------------------------
    close() {
      if (this.onClose) this.onClose();
      else this.app.backToMenu();
    }

    switchTab(tab) {
      if (tab === this.tab) return;
      this.tab = tab;
      this.first = 0;
      this.sel = 0;
      this.build();
      PG.audio.play("move");
    }

    cycleFilter() {
      const options = [null].concat(this.modes);
      const i = options.indexOf(this.filter) >= 0 ? options.indexOf(this.filter) : 0;
      this.filter = options[(i + 1) % options.length];
      this.first = this.sel = 0;
      this.applyView();
      PG.audio.play("move");
    }

    toggleSort() {
      this.sortBest = !this.sortBest;
      this.first = this.sel = 0;
      this.applyView();
      this.build();
      PG.audio.play("move");
    }

    move(d) {
      const rows = this.rows();
      if (!rows.length) return;
      this.sel = Math.max(0, Math.min(rows.length - 1, this.sel + d));
      if (this.sel < this.first) this.first = this.sel;
      else if (this.sel >= this.first + this.rowsVisible) this.first = this.sel - this.rowsVisible + 1;
      PG.audio.play("move");
    }

    handleEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Escape") this.close();
        else if (k === "Tab" || k === "ISO_Left_Tab" || k === "Left" || k === "Right"
                 || this.isAction(k, "left") || this.isAction(k, "right")) {
          this.switchTab(this.tab === "recent" ? "all" : "recent");
        } else if (k === "Up" || this.isAction(k, "up")) this.move(-1);
        else if (k === "Down" || this.isAction(k, "down")) this.move(1);
        else if (k === "Prior") this.move(-this.rowsVisible);
        else if (k === "Next") this.move(this.rowsVisible);
        else if (k === "Home") this.move(-this.rows().length);
        else if (k === "End") this.move(this.rows().length);
        else if ((k === "m" || k === "M") && this.tab === "all") this.cycleFilter();
        else if ((k === "o" || k === "O") && this.tab === "all") this.toggleSort();
      } else if (ev.kind === "mousemove") {
        this.mouse = ev.pos;
        this.hover = null;
        for (const [r, k] of this.tabRects) if (r.collidepoint(ev.pos)) this.hover = ["tab", k];
        for (const [r, mode] of this.filterRects) if (r.collidepoint(ev.pos)) this.hover = ["filter", mode];
        if (this.sortRect !== null && this.sortRect.collidepoint(ev.pos)) this.hover = ["sort", null];
        const rows = this.rows();
        this.rowRects.forEach((r, i) => {
          if (r.collidepoint(ev.pos) && this.first + i < rows.length) this.sel = this.first + i;
        });
        if (this.app && this.app.canvas) this.app.canvas.style.cursor = this.hover ? "pointer" : "";
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (const [r, k] of this.tabRects) {
          if (r.collidepoint(ev.pos)) {
            this.switchTab(k);
            return;
          }
        }
        for (const [r, mode] of this.filterRects) {
          if (r.collidepoint(ev.pos)) {
            if (mode !== this.filter) {
              this.filter = mode;
              this.first = this.sel = 0;
              this.applyView();
              PG.audio.play("move");
            }
            return;
          }
        }
        if (this.sortRect !== null && this.sortRect.collidepoint(ev.pos)) this.toggleSort();
      } else if (ev.kind === "wheel") {
        this.move(ev.delta > 0 ? -1 : 1);
      }
    }

    isHover(kind, value) {
      return this.hover !== null && this.hover[0] === kind && this.hover[1] === value;
    }

    // ----- Zeichnen -----------------------------------------------------
    draw(ctx) {
      const W = this.width, H = this.height;
      ui.drawBackground(ctx, W, H, false);
      ui.drawTitle(ctx, W, PG.gameName(this.entry), {
        subtitle: t("match.subtitle"), y: 26, big: ui.font(30, true), accent: this.accent,
      });
      for (const [r, k, lb] of this.tabRects) {
        const active = k === this.tab;
        const hover = this.isHover("tab", k);
        draw.rect(ctx, active || hover ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 13);
        draw.rect(ctx, active ? this.accent : ui.BORDER, r, 1, 13);
        ui.text(ctx, lb, cxOf(r), cyOf(r), this.tabFont, active ? ui.TEXT : ui.TEXT_DIM, "center");
      }
      this.drawSums(ctx);
      if (this.tab === "recent") this.drawChart(ctx);
      else this.drawFilters(ctx);
      const rows = this.rows();
      if (!rows.length) this.drawEmpty(ctx);
      else {
        for (let i = 0; i < this.rowRects.length; i++) {
          const idx = this.first + i;
          if (idx >= rows.length) break;
          this.drawRow(ctx, this.rowRects[i], rows[idx], idx === this.sel);
        }
        if (rows.length > this.rowsVisible) this.drawScrollbar(ctx, rows.length);
      }
      if (this.tab === "recent" && this.chartHits.length && this.mouse) this.drawChartTip(ctx);
      ui.drawFooter(ctx, W, H, t("match.hint_" + this.tab));
    }

    drawSums(ctx) {
      const tot = this.tot;
      const cells = [[t("match.sum_n"), fmtNum(tot.n)]];
      const decided = tot.w + tot.l + tot.d;
      if (decided) {
        const pct = pyRound((100.0 * tot.w) / decided);
        cells.push([t("match.sum_w"), `${fmtNum(tot.w)} (${pct} %)`]);
      }
      if (tot.n) {
        cells.push([t("match.sum_best"), fmtNum(tot.best || 0)]);
        cells.push([t("match.sum_avg"), fmtNum(pyRound(tot.sum / tot.n))]);
        cells.push([t("match.sum_time"), PG.matches.formatDuration(tot.time)]);
      }
      const r = this.sumRect;
      const gap = 8;
      const cw = Math.floor((r.w - gap * (cells.length - 1)) / cells.length);
      const labF = ui.font(11);
      const valF = ui.font(16, true);
      cells.forEach(([lab, val], i) => {
        const cell = new PG.Rect(r.x + i * (cw + gap), r.y, cw, r.h);
        ui.drawPanel(ctx, cell, { color: ui.PANEL, radius: 10, shadow: false });
        ui.text(ctx, lab, cxOf(cell), cell.y + 5, labF, ui.TEXT_FAINT, "midtop");
        ui.text(ctx, val, cxOf(cell), cell.bottom - 5, valF, i === 0 ? this.accent : ui.TEXT, "midbottom");
      });
    }

    drawChart(ctx) {
      const r = this.chartRect;
      ui.drawPanel(ctx, r, { color: ui.PANEL, radius: 10, shadow: false });
      const chron = this.all.slice(0, CHART_N).reverse();
      ui.text(ctx, t("match.chart", { n: chron.length }), r.x + 10, r.y + 6, ui.font(11, true), ui.TEXT_FAINT);
      // Index unter der Maus aus den Treffern des letzten Bilds - vor dem Leeren
      const hover = this.chartIndex();
      this.chartHits = [];
      if (chron.length < 2) {
        ui.text(ctx, t("match.chart_few"), cxOf(r), cyOf(r), ui.font(12), ui.TEXT_FAINT, "center");
        return;
      }
      const inner = new PG.Rect(r.x + 10, r.y + 34, r.w - 20, r.h - 42);
      const marks = [];
      chron.forEach((e, i) => {
        if (e[5] & PG.matches.F_RECORD) marks.push(i);
      });
      this.chartChron = chron;
      this.chartHits = PG.charts.line(ctx, inner, chron.map((e) => e[2]), {
        color: this.accent, marks, fmt: (v) => fmtNum(pyRound(v)), hover,
      });
    }

    /** Index des Kurvenpunkts unter der Maus (aus den Treffern des letzten Bilds). */
    chartIndex() {
      if (!this.mouse) return null;
      for (const [rect, idx] of this.chartHits) if (rect.collidepoint(this.mouse)) return idx;
      return null;
    }

    drawChartTip(ctx) {
      const idx = this.chartIndex();
      if (idx === null || idx >= this.chartChron.length) return;
      const e = this.chartChron[idx];
      PG.charts.tooltip(ctx, this.mouse, [fmtScore(e), e[0], modeLabel(this.entry, e[1])]);
    }

    drawFilters(ctx) {
      for (const [r, mode, lb] of this.filterRects) {
        const active = mode === this.filter;
        const hover = this.isHover("filter", mode);
        draw.rect(ctx, active || hover ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 12);
        draw.rect(ctx, active ? this.accent : ui.BORDER, r, 1, 12);
        ui.text(ctx, lb, cxOf(r), cyOf(r), this.chipFont, active ? ui.TEXT : ui.TEXT_DIM, "center");
      }
      const r = this.sortRect;
      const hover = this.isHover("sort", null);
      draw.rect(ctx, hover ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 12);
      draw.rect(ctx, hover ? ui.ACCENT2 : ui.BORDER, r, 1, 12);
      const lb = this.sortBest ? t("match.sort_best") : t("match.sort_new");
      ui.text(ctx, lb, cxOf(r), cyOf(r), this.chipFont, ui.TEXT, "center");
      ui.text(ctx, t("match.stored", { n: fmtNum(this.tot.stored), total: fmtNum(this.tot.n), max: PG.matches.CAP }),
        this.left, this.headY, ui.font(12, true), ui.TEXT_DIM, "midleft");
    }

    badge(ctx, cx, cy, result, record) {
      const M = PG.matches;
      const col = result === M.WIN ? ui.GREEN : result === M.LOSS ? ui.RED : result === M.DRAW ? ui.GOLD
        : ui.mix(ui.BTN, this.accent, 0.35);
      draw.circle(ctx, col, [cx, cy], 13);
      const dark = ui.mix(col, [0, 0, 0], 0.55);
      if (result === M.WIN) {
        draw.lines(ctx, dark, false, [[cx - 6, cy], [cx - 2, cy + 5], [cx + 6, cy - 5]], 3);
      } else if (result === M.LOSS) {
        draw.line(ctx, dark, [cx - 5, cy - 5], [cx + 5, cy + 5], 3);
        draw.line(ctx, dark, [cx - 5, cy + 5], [cx + 5, cy - 5], 3);
      } else if (result === M.DRAW) {
        draw.line(ctx, dark, [cx - 5, cy - 3], [cx + 5, cy - 3], 3);
        draw.line(ctx, dark, [cx - 5, cy + 3], [cx + 5, cy + 3], 3);
      } else {
        draw.circle(ctx, ui.TEXT_DIM, [cx, cy], 3);
      }
      if (record) draw.circle(ctx, ui.GOLD, [cx, cy], 16, 2);
    }

    drawRow(ctx, rect, e, selected) {
      const M = PG.matches;
      const [stamp, mode, , result, dur, flags, info] = e;
      const col = selected ? ui.PANEL_LIGHT : ui.PANEL;
      ui.drawPanel(ctx, rect, { color: col, radius: 10, shadow: false });
      if (selected) draw.rect(ctx, this.accent, rect, 1, 10);
      draw.rect(ctx, this.accent, [rect.x, rect.y + 8, 3, rect.h - 16], 0, 2);
      this.badge(ctx, rect.x + 28, cyOf(rect), result, flags & M.F_RECORD);

      const tf = ui.font(15, true);
      const title = fitText(tf, `${fmtScore(e)}  ·  ${modeLabel(this.entry, mode)}`, rect.w - 52 - 90);
      ui.text(ctx, title, rect.x + 52, rect.y + 5, tf, selected ? ui.TEXT : ui.TEXT_DIM);

      let parts = [];
      if (result !== null && result !== undefined) {
        parts.push(t({ [M.WIN]: "match.win", [M.LOSS]: "match.loss", [M.DRAW]: "match.draw" }[result]));
      }
      if (flags & M.F_RECORD) parts.push(t("match.record"));
      if (flags & M.F_ABORT) parts.push(t("match.aborted"));
      if (flags & M.F_ARCHIVE) parts.push(t("match.archive"));
      else if (flags & M.F_DAILY) parts.push(t("match.daily"));
      parts = parts.concat(infoParts(this.entry, info));
      const fnt = ui.font(11);
      const sub = fitText(fnt, parts.join("  ·  "), rect.w - 52 - 150);
      ui.text(ctx, sub, rect.x + 52, rect.y + 25, fnt, flags & M.F_RECORD ? ui.GOLD : ui.TEXT_FAINT);

      ui.text(ctx, M.formatDuration(dur), rect.right - 14, rect.y + 6, ui.font(13, true), this.accent, "topright");
      ui.text(ctx, stamp, rect.right - 14, rect.y + 25, ui.font(11), ui.TEXT_FAINT, "topright");
    }

    drawEmpty(ctx) {
      const cx = Math.floor(this.width / 2);
      const cy = Math.floor((this.top + this.bottom) / 2);
      ui.text(ctx, t("match.empty", { game: PG.gameName(this.entry) }), cx, cy - 12, ui.font(17, true), ui.TEXT_DIM, "center");
      ui.text(ctx, t("match.empty_hint"), cx, cy + 14, ui.font(13), ui.TEXT_FAINT, "center");
    }

    drawScrollbar(ctx, n) {
      const top = this.top, bottom = this.top + this.rowsVisible * ROW_H;
      const track = new PG.Rect(this.right + 6, top, 4, bottom - top);
      draw.rect(ctx, ui.PANEL, track, 0, 2);
      const frac = this.rowsVisible / n;
      const h = Math.max(24, Math.trunc(track.h * frac));
      const pos = this.first / Math.max(1, n - this.rowsVisible);
      const y = top + Math.trunc((track.h - h) * Math.min(1.0, pos));
      draw.rect(ctx, ui.BORDER_LIGHT, [track.x, y, 4, h], 0, 2);
    }
  }

  PG.MatchScreen = MatchScreen;
  PG.matchView = { modeLabel, fmtNum, fmtScore, fitText, infoParts, pyRound, ROW_H, TABS, CHART_N, RECENT_N };
})();
