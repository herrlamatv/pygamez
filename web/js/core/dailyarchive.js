/*
 * dailyarchive.js - Tages-Archiv (Port von dailyarchive.py)
 * ============================================================================
 * Für die Tagesmodi von Crossy Road (Tagesstrecke), Sudoku (Tagesrätsel) und
 * Wordle (Tageswort). Der Tages-Seed hängt nur vom Datum ab
 * (PG.seedrand.dailySeed(spiel, datum) - in Python, JS und Rust bitgleich),
 * jeder vergangene Tag lässt sich also exakt neu erzeugen. Nachspielbar sind
 * die letzten WINDOW Tage (heute eingeschlossen) und jeder ältere Tag, den der
 * Spieler schon gespielt hat.
 *
 * Gespeichert wird kompakt in der Section des Spiels als Liste von Zeilen,
 * deren erstes Feld die Tagesnummer ist (PG.seedrand.dayIndex, Tage seit
 * 2026-01-01), z.B. Crossy: "arch": [[329, 57, 3], [330, 112, 1], ...]
 * (Tag, Bestwert, Versuche). Die Liste bleibt nach Tag sortiert.
 *
 *   PG.dailyArchive.available(played)        wählbare Tage
 *   PG.dailyArchive.cleanRows/find/upsert    Archiv-Zeilen
 *   PG.dailyArchive.dayRowRects/drawDayRow   Datums-Leiste [◀][Heute · 26.11.][▶][Archiv]
 *   new PG.dailyArchive.Picker(...)          Auswahlfenster (Kalender)
 *
 * PG.seedrand (js/games/seedrand.js) wird erst beim Aufruf gebraucht.
 */
(function () {
  "use strict";

  const PG = window.PG;

  // Tage im Kalender (heute eingeschlossen)
  const WINDOW = 30;
  // Höchstens so viele Tage merkt sich ein Archiv (gut 5 Jahre - Schutz gegen
  // beschädigte Daten, im Alltag nie erreicht).
  const MAX_DAYS = 2000;

  function isInt(v) {
    return typeof v === "number" && Number.isInteger(v);
  }

  function pad2(n) {
    return String(n).padStart(2, "0");
  }

  // ------------------------------------------------------------ Tage & Daten
  function todayIndex() {
    return PG.seedrand.dayIndex();
  }

  /**
   * Tagesnummer eines Datumstexts "YYYY-MM-DD" oder -1, wenn er ungültig ist
   * (wie seedrand.day_index + "except ValueError: idx = -1"). Leerer Text =
   * heute (day_index(None)).
   */
  function safeDayIndex(date) {
    if (!date) return todayIndex();
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(date));
    if (!m) return -1;
    const y = Number(m[1]), mo = Number(m[2]), d = Number(m[3]);
    const dt = new Date(Date.UTC(y, mo - 1, d));
    if (dt.getUTCFullYear() !== y || dt.getUTCMonth() !== mo - 1 || dt.getUTCDate() !== d) return -1;
    return PG.seedrand.dayIndex(String(date));
  }

  /** Die letzten WINDOW Tage als Tagesnummern, älteste zuerst. */
  function windowDays(today) {
    today = today == null ? todayIndex() : today;
    const out = [];
    for (let d = today - WINDOW + 1; d <= today; d++) out.push(d);
    return out;
  }

  /** Alle wählbaren Tage (Fenster + ältere gespielte), älteste zuerst. */
  function available(played, today) {
    today = today == null ? todayIndex() : today;
    const days = new Set(windowDays(today));
    for (const d of played || []) {
      if (isInt(d) && d >= 0 && d <= today) days.add(d);
    }
    return Array.from(days).sort((a, b) => a - b);
  }

  /**
   * Archiv-Zeilen prüfen: [tag, ...] mit genau 'width' Feldern, sortiert.
   * Ungültige Zeilen fallen weg, doppelte Tage: die letzte gewinnt. Tage in der
   * Zukunft (Uhr verstellt) bleiben erhalten, werden aber nicht angeboten.
   */
  function cleanRows(raw, width) {
    if (!Array.isArray(raw)) return [];
    const rows = new Map();
    for (const row of raw) {
      if (!Array.isArray(row) || row.length !== width) continue;
      const day = row[0];
      if (!isInt(day) || day < 0) continue;
      rows.set(day, row.slice());
    }
    const out = Array.from(rows.keys()).sort((a, b) => a - b).map((d) => rows.get(d));
    return out.slice(-MAX_DAYS);
  }

  /** Zeile eines Tages oder null. */
  function find(rows, day) {
    for (const row of rows) if (row[0] === day) return row;
    return null;
  }

  /** Zeile eintragen/ersetzen (Liste bleibt nach Tag sortiert). */
  function upsert(rows, row) {
    const day = row[0];
    for (let i = 0; i < rows.length; i++) {
      const old = rows[i];
      if (old[0] === day) {
        rows[i] = row;
        return rows;
      }
      if (old[0] > day) {
        rows.splice(i, 0, row);
        return rows;
      }
    }
    rows.push(row);
    if (rows.length > MAX_DAYS) rows.splice(0, rows.length - MAX_DAYS);
    return rows;
  }

  function dateStr(day) {
    return PG.seedrand.dateOf(day);
  }

  /** {y, m, d, wd} eines Tages (wd: Montag = 0 wie date.weekday()). */
  function partsOf(day) {
    const dt = new Date(Date.UTC(2026, 0, 1) + day * 86400000);
    return { y: dt.getUTCFullYear(), m: dt.getUTCMonth() + 1, d: dt.getUTCDate(), wd: (dt.getUTCDay() + 6) % 7 };
  }

  /** Kürzt einen Text mit '…', bis er in 'room' Pixel passt. */
  function fitText(fnt, text, room) {
    if (fnt.width(text) <= room) return text;
    while (text && fnt.width(text + "…") > room) text = text.slice(0, -1);
    return text ? text.trimEnd() + "…" : "";
  }

  /** '23.11.25' (kurze Leiste der älteren Tage). */
  function chipDate(day) {
    const p = partsOf(day);
    return `${pad2(p.d)}.${pad2(p.m)}.${pad2(p.y % 100)}`;
  }

  /** '26.11.' (für Chips und HUD). */
  function shortDate(day) {
    const p = partsOf(day);
    return `${pad2(p.d)}.${pad2(p.m)}.`;
  }

  /** 'Mi, 26.11.2026' (Wochentag aus den Sprachdateien). */
  function longDate(day) {
    const p = partsOf(day);
    const wd = PG.charts.weekdayNames()[p.wd];
    return `${wd}, ${pad2(p.d)}.${pad2(p.m)}.${String(p.y).padStart(4, "0")}`;
  }

  /** 'Heute · 26.11.' / 'Gestern · 25.11.' / 'Mo, 23.11.2026'. */
  function dayLabel(day, today) {
    today = today == null ? todayIndex() : today;
    if (day === today) return PG.t("arch.today", { date: shortDate(day) });
    if (day === today - 1) return PG.t("arch.yesterday", { date: shortDate(day) });
    return longDate(day);
  }

  /** Seed des Tages als 8 Hex-Zeichen (Anzeige). */
  function seedHex(game, day) {
    return (PG.seedrand.dailySeed(game, dateStr(day)) >>> 0).toString(16).toUpperCase().padStart(8, "0");
  }

  // ------------------------------------------- Datums-Leiste im Setup
  //  [◀] [ Heute · 26.11. ] [▶] [Archiv]

  /**
   * Rechtecke [zurück, Datum, vor, Archiv] einer Datums-Leiste. Schmale
   * Leisten (unter 300 px) bekommen einen quadratischen Archiv-Knopf (nur
   * Symbol), damit das Datum lesbar bleibt.
   */
  function dayRowRects(x, y, w, h) {
    const gap = 6;
    const arrow = Math.min(h, Math.max(18, Math.floor(w / 12)));
    const arch = w < 300 ? h : Math.max(90, Math.floor(w / 4));
    const mid = w - 2 * arrow - arch - 3 * gap;
    const rPrev = new PG.Rect(x, y, arrow, h);
    const rDate = new PG.Rect(rPrev.right + gap, y, mid, h);
    const rNext = new PG.Rect(rDate.right + gap, y, arrow, h);
    const rArch = new PG.Rect(rNext.right + gap, y, x + w - (rNext.right + gap), h);
    return [rPrev, rDate, rNext, rArch];
  }

  /** pygame-Mitte (ganzzahlig wie Rect.center). */
  function cxOf(r) {
    return r.x + Math.floor(r.w / 2);
  }
  function cyOf(r) {
    return r.y + Math.floor(r.h / 2);
  }

  function drawTriangle(ctx, rect, direction, color) {
    const cx = cxOf(rect), cy = cyOf(rect);
    const k = Math.max(4, Math.floor(rect.h / 5));
    const h2 = Math.floor(k / 2);
    let pts;
    if (direction < 0) pts = [[cx + h2, cy - k], [cx + h2, cy + k], [cx - k, cy]];
    else pts = [[cx - h2, cy - k], [cx - h2, cy + k], [cx + k, cy]];
    PG.draw.polygon(ctx, color, pts);
  }

  /**
   * Zeichnet die Datums-Leiste (Pfeile ausgegraut, wenn es nicht weitergeht).
   * Passt "Gestern · 25.11." bzw. "Mo, 23.11.2026" nicht, steht nur das kurze
   * Datum da.
   */
  function drawDayRow(ctx, rects, day, accent, canPrev, canNext, fnt, archived = false) {
    const ui = PG.ui, draw = PG.draw;
    const [rPrev, rDate, rNext, rArch] = rects;
    let label = dayLabel(day);
    if (fnt.width(label) > rDate.w - 10) label = shortDate(day);
    for (const [rc, d, on] of [[rPrev, -1, canPrev], [rNext, 1, canNext]]) {
      draw.rect(ctx, ui.BTN, rc, 0, 8);
      draw.rect(ctx, ui.BORDER, rc, 1, 8);
      drawTriangle(ctx, rc, d, on ? ui.TEXT : ui.TEXT_FAINT);
    }
    draw.rect(ctx, archived ? ui.BTN_SEL : ui.BTN, rDate, 0, 8);
    draw.rect(ctx, archived ? ui.GOLD : accent, rDate, 1, 8);
    ui.text(ctx, fitText(fnt, label, rDate.w - 10), cxOf(rDate), cyOf(rDate), fnt,
      archived ? ui.GOLD : ui.TEXT, "center");
    draw.rect(ctx, ui.BTN, rArch, 0, 8);
    draw.rect(ctx, accent, rArch, 1, 8);
    // Kalender-Symbol + Text (bei schmalem Knopf nur das Symbol)
    const side = Math.max(10, Math.floor(rArch.h / 2));
    const ic = new PG.Rect(0, 0, side, side);
    const lab = PG.t("arch.button");
    const labW = fnt.width(lab);
    let total = ic.w + 6 + labW;
    const showLab = total <= rArch.w - 10;
    if (!showLab) total = ic.w;
    ic.x = cxOf(rArch) - Math.floor(total / 2);
    ic.y = cyOf(rArch) - Math.floor(ic.h / 2);
    draw.rect(ctx, ui.TEXT_DIM, ic, 1, 2);
    draw.line(ctx, ui.TEXT_DIM, [ic.x, ic.y + Math.floor(ic.h / 3)], [ic.right - 1, ic.y + Math.floor(ic.h / 3)]);
    if (showLab) ui.text(ctx, lab, ic.right + 6, cyOf(rArch), fnt, ui.TEXT, "midleft");
  }

  // ------------------------------------------------------------ Auswahlfenster
  /**
   * Kalender der letzten WINDOW Tage + ältere gespielte Tage.
   * info: Map/Objekt {tag: [zustand, wert_text]} - zustand ist "solved"
   * (gelöst/geschafft), "played" (gespielt, nicht gelöst), "started"
   * (angefangen) oder fehlt (offen). handleEvent liefert ["pick", tag],
   * ["close", null] oder null.
   */
  class Picker {
    constructor(gameKey, accent, sel, info, today) {
      this.gameKey = gameKey;
      this.accent = accent;
      this.today = today == null ? todayIndex() : today;
      this.info = new Map();
      if (info instanceof Map) for (const [k, v] of info) this.info.set(Number(k), v);
      else for (const k of Object.keys(info || {})) this.info.set(Number(k), info[k]);
      this.days = available(Array.from(this.info.keys()), this.today);
      this.window = windowDays(this.today);
      this.older = this.days.filter((d) => d < this.window[0]);
      this.sel = this.days.indexOf(sel) >= 0 ? sel : this.today;
      this.hover = null;
      this.rect = null;
    }

    // ----- Layout -------------------------------------------------------
    layout(W, H) {
      const pw = Math.min(640, W - 32);
      const ph = Math.min(H - 32, 470);
      this.rect = new PG.Rect(Math.floor(W / 2) - Math.floor(pw / 2), Math.floor(H / 2) - Math.floor(ph / 2), pw, ph);
      const r = this.rect;
      const top = r.y + 64;
      const olderH = this.older.length ? 54 : 0;
      const foot = 54;
      const gridH = r.bottom - foot - olderH - top - 22;
      // Wochen-Raster: Montag der Woche des ersten Fenstertags
      const start = this.window[0] - partsOf(this.window[0]).wd;
      const weeks = Math.floor((this.today - start) / 7) + 1;
      const cw = Math.floor((r.w - 32 - 6 * 6) / 7);
      const ch = Math.max(26, Math.min(54, Math.floor((gridH - (weeks - 1) * 5) / Math.max(1, weeks))));
      const gx = r.x + Math.floor((r.w - (7 * cw + 6 * 6)) / 2);
      this.headY = top;
      this.cells = [];
      const inWindow = new Set(this.window);
      for (let wi = 0; wi < weeks; wi++) {
        for (let di = 0; di < 7; di++) {
          const day = start + wi * 7 + di;
          const rc = new PG.Rect(gx + di * (cw + 6), top + 18 + wi * (ch + 5), cw, ch);
          this.cells.push([rc, inWindow.has(day) ? day : null]);
        }
      }
      this.gridX = gx;
      this.cellW = cw;
      // Leiste der älteren gespielten Tage (neueste zuerst)
      this.chips = [];
      if (this.older.length) {
        const y = top + 18 + weeks * (ch + 5) + 22;
        let x = r.x + 16;
        const fnt = PG.ui.font(11, true);
        for (const day of this.older.slice().reverse()) {
          const w = fnt.width(chipDate(day)) + 18;
          if (x + w > r.right - 16) break;
          this.chips.push([new PG.Rect(x, y, w, 24), day]);
          x += w + 6;
        }
        this.olderY = y - 14;
      }
    }

    // ----- Eingabe ------------------------------------------------------
    step(d) {
      const i = this.days.indexOf(this.sel);
      this.sel = this.days[Math.max(0, Math.min(this.days.length - 1, i + d))];
    }

    handleEvent(ev, game) {
      const act = (k, a) => !!(game && game.isAction(k, a));
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Escape") return ["close", null];
        if (k === "Return" || k === "space" || k === "KP_Enter") return ["pick", this.sel];
        if (k === "Left" || act(k, "left")) this.step(-1);
        else if (k === "Right" || act(k, "right")) this.step(1);
        else if (k === "Up" || act(k, "up")) {
          if (this.days.indexOf(this.sel - 7) >= 0) this.sel -= 7;
          else this.step(-1);
        } else if (k === "Down" || act(k, "down")) {
          if (this.days.indexOf(this.sel + 7) >= 0) this.sel += 7;
          else this.step(1);
        } else if (k === "Home") this.sel = this.days[0];
        else if (k === "End") this.sel = this.today;
        return null;
      }
      if (ev.kind === "mousemove") {
        this.hover = this.hit(ev.pos);
        if (this.hover !== null) this.sel = this.hover;
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const day = this.hit(ev.pos);
        if (day !== null) return ["pick", day];
        if (this.rect !== null && !this.rect.collidepoint(ev.pos)) return ["close", null];
      }
      return null;
    }

    hit(pos) {
      if (!this.cells) return null;
      for (const [rc, day] of this.cells) if (day !== null && rc.collidepoint(pos)) return day;
      for (const [rc, day] of this.chips) if (rc.collidepoint(pos)) return day;
      return null;
    }

    // ----- Zeichnen -----------------------------------------------------
    stateColor(state) {
      const ui = PG.ui;
      return { solved: ui.GREEN, played: ui.GOLD, started: this.accent }[state] || ui.TEXT_FAINT;
    }

    infoOf(day) {
      return this.info.get(day) || [null, ""];
    }

    draw(ctx, W = PG.W, H = PG.H) {
      const ui = PG.ui, draw = PG.draw, t = PG.t;
      if (this.rect === null) this.layout(W, H);
      draw.rect(ctx, [0, 0, 0, 150], [0, 0, W, H]);
      const r = this.rect;
      ui.drawPanel(ctx, r, { radius: 14 });
      draw.rect(ctx, this.accent, r, 1, 14);
      ui.text(ctx, t("arch.title"), cxOf(r), r.y + 12, ui.font(20, true), ui.TEXT, "midtop");
      ui.text(ctx, t("arch.sub", { n: WINDOW }), cxOf(r), r.y + 40, ui.font(12), ui.TEXT_DIM, "midtop");
      // Wochentage
      const names = PG.charts.weekdayNames();
      const hf = ui.font(11, true);
      for (let i = 0; i < 7; i++) {
        const cx = this.gridX + i * (this.cellW + 6) + Math.floor(this.cellW / 2);
        ui.text(ctx, names[i], cx, this.headY, hf, ui.TEXT_FAINT, "midtop");
      }
      const nf = ui.font(14, true);
      const vf = ui.font(10);
      for (const [rc, day] of this.cells) {
        if (day === null) continue;
        const [state, value] = this.infoOf(day);
        const sel = day === this.sel;
        draw.rect(ctx, sel ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
        const border = sel ? this.accent : day === this.today ? ui.GOLD : ui.BORDER;
        draw.rect(ctx, border, rc, sel || day === this.today ? 2 : 1, 7);
        ui.text(ctx, String(partsOf(day).d), rc.x + 6, rc.y + 3, nf, state ? ui.TEXT : ui.TEXT_DIM);
        if (state) draw.circle(ctx, this.stateColor(state), [rc.right - 8, rc.y + 9], 4);
        if (value && rc.h >= 34) {
          ui.text(ctx, fitText(vf, String(value), rc.w - 8), rc.x + 5, rc.bottom - 3, vf,
            this.stateColor(state), "bottomleft");
        }
      }
      if (this.older.length) {
        ui.text(ctx, t("arch.older", { n: this.older.length }), r.x + 16, this.olderY, ui.font(11, true), ui.TEXT_FAINT);
        const cf = ui.font(11, true);
        for (const [rc, day] of this.chips) {
          const [state] = this.infoOf(day);
          const sel = day === this.sel;
          draw.rect(ctx, sel ? ui.BTN_SEL : ui.BTN, rc, 0, 12);
          draw.rect(ctx, sel ? this.accent : this.stateColor(state), rc, 1, 12);
          ui.text(ctx, chipDate(day), cxOf(rc), cyOf(rc), cf, ui.TEXT, "center");
        }
      }
      // Detailzeile des gewählten Tags + Hinweis
      const [state, value] = this.infoOf(this.sel);
      const parts = [dayLabel(this.sel, this.today), t("arch.seed", { s: seedHex(this.gameKey, this.sel) })];
      parts.push(t("arch.state." + (state || "open")));
      if (value) parts.push(String(value));
      const df = ui.font(13, true);
      ui.text(ctx, fitText(df, parts.join("  ·  "), r.w - 24), cxOf(r), r.bottom - 30, df,
        state ? this.stateColor(state) : ui.TEXT, "midbottom");
      ui.text(ctx, t("arch.hint"), cxOf(r), r.bottom - 10, ui.font(11), ui.TEXT_FAINT, "midbottom");
    }
  }

  PG.dailyArchive = {
    WINDOW, MAX_DAYS,
    todayIndex, safeDayIndex, windowDays, available, cleanRows, find, upsert,
    dateStr, fitText, chipDate, shortDate, longDate, dayLabel, seedHex,
    dayRowRects, drawTriangle, drawDayRow, Picker,
  };
})();
