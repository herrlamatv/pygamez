/**
 * charts.js - kleine, theme-treue Diagramme für die Statistik-Screens
 * (Port von charts.py auf die Canvas).
 *
 * Alle Funktionen zeichnen in ein PG.Rect und geben die Trefferflächen
 * `[[rect, daten], ...]` zurück - daraus baut der Screen Hover und Tooltips.
 *
 *   PG.charts.bars()          Balken (senkrecht oder waagerecht)
 *   PG.charts.line()          Linienverlauf mit Rekord-Markern
 *   PG.charts.calendarHeat()  Spieltage-Karte (7 Reihen x Wochen)
 *   PG.charts.donut()         Ringdiagramm
 *   PG.charts.clockRing()     Tageszeit-Profil (24 Speichen)
 *   PG.charts.tooltip()       Hinweiskasten an der Maus
 */
(function () {
  "use strict";

  const FALLBACK_WEEKDAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
  const FALLBACK_MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

  function nameList(key, expected, fallback) {
    const parts = String(PG.t(key)).split(",").map((s) => s.trim()).filter(Boolean);
    return parts.length < expected ? fallback.slice() : parts.slice(0, expected);
  }

  function gridColor() {
    return PG.ui.mix(PG.ui.PANEL, PG.ui.BORDER_LIGHT, 0.6);
  }

  function pad(n) {
    return String(n).padStart(2, "0");
  }

  const charts = {
    weekdayNames() {
      return nameList("cal.weekdays_short", 7, FALLBACK_WEEKDAYS);
    },

    monthNames() {
      return nameList("cal.months_short", 12, FALLBACK_MONTHS);
    },

    /** Balkendiagramm; opts: {labels, colors, color, horizontal, fmt, hover}. */
    bars(ctx, rect, values, opts) {
      const ui = PG.ui;
      const draw = PG.draw;
      const o = opts || {};
      const hits = [];
      if (!values || !values.length) return hits;
      const base = o.color || ui.ACCENT;
      const top = Math.max(o.maxValue != null ? o.maxValue : Math.max(...values), 1e-6);
      const n = values.length;
      const fnt = ui.font(o.labelSize || 10);
      const valFnt = ui.font(o.labelSize || 10, true);
      const labels = o.labels || null;

      if (o.horizontal) {
        let labelW = 0;
        if (labels) {
          labelW = Math.min(rect.w / 3, Math.max(...labels.map((s) => fnt.width(String(s)))) + 8);
        }
        const gap = 4;
        const rowH = Math.max(10, (rect.h - gap * (n - 1)) / n);
        for (let i = 0; i < n; i++) {
          const y = rect.y + i * (rowH + gap);
          const col = o.colors ? o.colors[i] : base;
          if (labels && i < labels.length) {
            ui.text(ctx, String(labels[i]), rect.x, y + rowH / 2, fnt,
              i === o.hover ? ui.TEXT : ui.TEXT_DIM, "midleft");
          }
          const bx = rect.x + labelW;
          const bw = Math.max(0, rect.right - bx);
          const track = new PG.Rect(bx, y, bw, rowH);
          draw.rect(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.4), track, 0, rowH / 2);
          const fill = bw * (values[i] / top);
          if (fill > 1) draw.rect(ctx, col, [bx, y, fill, rowH], 0, rowH / 2);
          if (o.fmt) ui.text(ctx, o.fmt(values[i]), rect.right - 6, y + rowH / 2, valFnt, ui.TEXT_DIM, "midright");
          hits.push([track, i]);
        }
        return hits;
      }

      const gap = Math.max(2, Math.min(8, rect.w / (n * 5)));
      const barW = Math.max(3, (rect.w - gap * (n - 1)) / n);
      const labelH = labels ? fnt.height + 2 : 0;
      const topPad = valFnt.height + 2;
      const plotH = Math.max(8, rect.h - labelH - topPad);
      for (let i = 0; i < n; i++) {
        const x = rect.x + i * (barW + gap);
        const h = plotH * (values[i] / top);
        const y = rect.y + topPad + (plotH - h);
        let col = o.colors ? o.colors[i] : base;
        if (i === o.hover) col = ui.mix(col, [255, 255, 255], 0.35);
        const cell = new PG.Rect(x, rect.y + topPad, barW, plotH);
        draw.rect(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), cell, 0, 3);
        if (h > 1) draw.rect(ctx, col, [x, y, barW, h], 0, 3);
        if (labels && i < labels.length) {
          ui.text(ctx, String(labels[i]), x + barW / 2, rect.bottom - labelH + 1, fnt,
            i === o.hover ? ui.TEXT : ui.TEXT_FAINT, "midtop");
        }
        if (o.fmt && (i === o.hover || n <= 12)) {
          ui.text(ctx, o.fmt(values[i]), x + barW / 2, y - 1, valFnt, ui.TEXT_DIM, "midbottom");
        }
        hits.push([cell, i]);
      }
      return hits;
    },

    /** Linienverlauf; opts: {color, marks, fmt, hover, fill}. */
    line(ctx, rect, values, opts) {
      const ui = PG.ui;
      const draw = PG.draw;
      const o = opts || {};
      const hits = [];
      if (!values || !values.length) return hits;
      const col = o.color || ui.ACCENT;
      const marks = new Set(o.marks || []);
      let lo = Math.min(...values);
      let hi = Math.max(...values);
      if (hi <= lo) hi = lo + 1;
      const n = values.length;
      const fnt = ui.font(10);
      for (const frac of [0, 0.5, 1]) {
        const y = Math.round(rect.bottom - frac * rect.h);
        draw.line(ctx, gridColor(), [rect.x, y], [rect.right, y]);
        if (o.fmt) ui.text(ctx, o.fmt(lo + (hi - lo) * frac), rect.x + 2, y - 1, fnt, ui.TEXT_FAINT, "bottomleft");
      }
      const step = n > 1 ? rect.w / (n - 1) : 0;
      const points = values.map((v, i) => [
        Math.round(rect.x + (n > 1 ? i * step : rect.w / 2)),
        Math.round(rect.bottom - ((v - lo) / (hi - lo)) * rect.h),
      ]);
      if (o.fill !== false && points.length > 1) {
        const poly = points.concat([[points[points.length - 1][0], rect.bottom], [points[0][0], rect.bottom]]);
        draw.polygon(ctx, ui.col(col, 46), poly);
      }
      if (points.length > 1) draw.lines(ctx, col, false, points, 2);
      points.forEach(([x, y], i) => {
        if (marks.has(i)) {
          draw.circle(ctx, ui.GOLD, [x, y], 4);
          draw.circle(ctx, ui.mix(ui.GOLD, [255, 255, 255], 0.5), [x, y], 2);
        } else if (n <= 60) {
          draw.circle(ctx, col, [x, y], 2);
        }
        if (i === o.hover) draw.circle(ctx, ui.TEXT, [x, y], 5, 1);
        hits.push([new PG.Rect(x - Math.max(2, step / 2), rect.y, Math.max(4, step || 4), rect.h), i]);
      });
      return hits;
    },

    /** Spieltage-Karte. days = {"YYYY-MM-DD": sekunden}. */
    calendarHeat(ctx, rect, days, opts) {
      const ui = PG.ui;
      const draw = PG.draw;
      const o = opts || {};
      const hits = [];
      const col = o.color || ui.ACCENT;
      const fnt = ui.font(9);
      const labelH = o.monthLabels === false ? 0 : fnt.height + 2;
      const dayLabelW = o.monthLabels === false ? 0 : fnt.width("Mo") + 4;
      const grid = new PG.Rect(rect.x + dayLabelW, rect.y + labelH, rect.w - dayLabelW, rect.h - labelH);
      const cell = Math.max(6, Math.min(16, Math.floor(grid.h / 7) - 2));
      const gap = Math.max(1, Math.floor(cell / 6));
      const perWeek = cell + gap;
      const fit = Math.max(1, Math.floor((grid.w + gap) / perWeek));
      const weeks = Math.min(o.weeks || fit, fit);

      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const monday = new Date(today.getTime() - ((today.getDay() + 6) % 7) * 86400000);
      const values = Object.values(days || {});
      const top = Math.max(values.length ? Math.max(...values) : 1, 1);
      const seenMonths = new Set();

      for (let w = 0; w < weeks; w++) {
        const weekStart = new Date(monday.getTime() - (weeks - 1 - w) * 7 * 86400000);
        const x = grid.x + w * perWeek;
        for (let d = 0; d < 7; d++) {
          const date = new Date(weekStart.getTime() + d * 86400000);
          if (date > today) continue;
          const key = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
          const value = (days && days[key]) || 0;
          const y = grid.y + d * perWeek;
          const box = new PG.Rect(x, y, cell, cell);
          if (value > 0) {
            const frac = Math.min(1, Math.pow(value / top, 0.45));
            draw.rect(ctx, ui.mix(ui.mix(ui.PANEL, col, 0.25), col, frac), box, 0, 2);
          } else {
            draw.rect(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), box, 0, 2);
          }
          if (date.getTime() === today.getTime()) draw.rect(ctx, ui.GOLD, box, 1, 2);
          if (o.hover && o.hover === key) draw.rect(ctx, ui.TEXT, box, 1, 2);
          hits.push([box, [key, value]]);
        }
        if (labelH && !seenMonths.has(weekStart.getMonth()) && (w === 0 || weekStart.getDate() <= 7)) {
          seenMonths.add(weekStart.getMonth());
          ui.text(ctx, this.monthNames()[weekStart.getMonth()], x, rect.y, fnt, ui.TEXT_FAINT);
        }
      }
      if (labelH) {
        const short = this.weekdayNames();
        for (const d of [0, 2, 4]) {
          ui.text(ctx, short[d], rect.x, grid.y + d * perWeek - 1, fnt, ui.TEXT_FAINT);
        }
      }
      return hits;
    },

    heatLegend(ctx, rect, opts) {
      const ui = PG.ui;
      const o = opts || {};
      const col = o.color || ui.ACCENT;
      const fnt = ui.font(9);
      let x = rect.x;
      if (o.less) {
        const r = ui.text(ctx, o.less, x, rect.centery, fnt, ui.TEXT_FAINT, "midleft");
        x += r.w + 6;
      }
      const size = Math.min(10, rect.h);
      for (const frac of [0, 0.33, 0.66, 1]) {
        const box = [x, rect.centery - size / 2, size, size];
        const fill = frac <= 0 ? ui.mix(ui.PANEL, ui.BG_TOP, 0.35) : ui.mix(ui.mix(ui.PANEL, col, 0.25), col, frac);
        PG.draw.rect(ctx, fill, box, 0, 2);
        x += size + 3;
      }
      if (o.more) ui.text(ctx, o.more, x + 3, rect.centery, fnt, ui.TEXT_FAINT, "midleft");
    },

    /** Ringdiagramm; slices = [[wert, farbe], ...]. */
    donut(ctx, center, radius, slices, opts) {
      const ui = PG.ui;
      const draw = PG.draw;
      const o = opts || {};
      const thickness = o.thickness || Math.max(6, radius / 3);
      const total = slices.reduce((s, [v]) => s + Math.max(0, v), 0);
      const box = [center[0] - radius, center[1] - radius, radius * 2, radius * 2];
      draw.circle(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), center, radius, thickness);
      if (total > 0) {
        let start = -Math.PI / 2;
        for (const [value, col] of slices) {
          if (value <= 0) continue;
          const end = start + PG.TAU * (value / total);
          draw.arc(ctx, col, box, start, end, thickness);
          start = end;
        }
      }
      if (o.centerText) {
        const f = ui.font(Math.max(12, Math.round(radius / 3)), true);
        ui.text(ctx, o.centerText, center[0], center[1] - (o.centerSub ? 3 : 0), f, ui.TEXT, "center");
        if (o.centerSub) {
          ui.text(ctx, o.centerSub, center[0], center[1] + f.height / 2 + 4,
            ui.font(Math.max(9, Math.round(radius / 5))), ui.TEXT_DIM, "center");
        }
      }
    },

    /** Tageszeit-Profil: 24 Speichen. */
    clockRing(ctx, center, radius, hours, opts) {
      const ui = PG.ui;
      const draw = PG.draw;
      const o = opts || {};
      const col = o.color || ui.ACCENT;
      const top = hours.length ? Math.max(...hours) : 0;
      const hits = [];
      const inner = Math.max(8, radius / 3);
      draw.circle(ctx, gridColor(), center, radius, 1);
      draw.circle(ctx, gridColor(), center, inner, 1);
      const fnt = ui.font(9);
      for (let h = 0; h < 24; h++) {
        const ang = -Math.PI / 2 + (h * Math.PI) / 12;
        const frac = top > 0 ? hours[h] / top : 0;
        const len = inner + (radius - inner) * frac;
        const spoke = h === o.hover ? ui.mix(col, [255, 255, 255], 0.4) : col;
        if (frac > 0) {
          draw.lines(ctx, spoke, false, [
            [center[0] + inner * Math.cos(ang), center[1] + inner * Math.sin(ang)],
            [center[0] + len * Math.cos(ang), center[1] + len * Math.sin(ang)],
          ], 5);
        }
        const box = new PG.Rect(0, 0, 14, 14);
        box.center = [center[0] + (radius - 6) * Math.cos(ang), center[1] + (radius - 6) * Math.sin(ang)];
        hits.push([box, h]);
      }
      for (const h of [0, 6, 12, 18]) {
        const ang = -Math.PI / 2 + (h * Math.PI) / 12;
        ui.text(ctx, String(h), center[0] + (radius + 9) * Math.cos(ang),
          center[1] + (radius + 9) * Math.sin(ang), fnt, ui.TEXT_FAINT, "center");
      }
      return hits;
    },

    /** Kleiner Hinweiskasten neben der Maus (erste Zeile fett). */
    tooltip(ctx, pos, lines, widthLimit) {
      const ui = PG.ui;
      const rows = (lines || []).filter(Boolean);
      if (!rows.length) return;
      const head = ui.font(11, true);
      const body = ui.font(10);
      const widths = rows.map((s, i) => (i === 0 ? head : body).width(s));
      const w = Math.max(...widths) + 16;
      const h = head.height + (rows.length - 1) * body.height + 12;
      const limit = widthLimit || PG.W;
      const x = Math.max(4, Math.min(pos[0] + 14, limit - w - 4));
      const y = Math.max(4, pos[1] - h - 10);
      const rect = new PG.Rect(x, y, w, h);
      ui.drawPanel(ctx, rect, { color: ui.PANEL_LIGHT, radius: 8, shadow: false });
      let ty = rect.y + 6;
      rows.forEach((text, i) => {
        const f = i === 0 ? head : body;
        ui.text(ctx, text, rect.x + 8, ty, f, i === 0 ? ui.TEXT : ui.TEXT_DIM);
        ty += f.height;
      });
    },
  };

  PG.charts = charts;
})();
