/*
 * geodash_draw.js - Zeichnen für Geometry Dash (Port von games/geodash_draw.py)
 * ============================================================================
 * Spiel UND Level-Editor zeichnen mit denselben Funktionen. Die Welt wird in
 * Blöcken gerechnet: ts = Kachelgröße (logische Pixel), camX/camY = Welt-
 * Koordinate der linken unteren Ecke. Welt-y zeigt nach oben.
 *
 * Teure Teile (Block-Kacheln je Kanten-Maske, Stacheln je Drehung, Leucht-
 * scheiben, Spieler-Figuren) werden als Offscreen-Canvas in der echten
 * Pixeldichte (PG.app.pixelScale) gecacht und pro Frame nur geblittet.
 */
(function () {
  "use strict";

  const { ui, draw } = PG;
  const core = PG.gdCore;

  const COL_YELLOW = [255, 214, 40];
  const COL_PINK = [255, 90, 210];
  const COL_BLUE = [70, 200, 255];
  const COL_PAD = [COL_YELLOW, COL_PINK, COL_BLUE];
  const COL_MODE = [[90, 255, 110], [255, 110, 220], [255, 120, 50], [255, 190, 40], [70, 170, 255]];
  const COL_GRAV = { 1: [70, 200, 255], "-1": [255, 225, 60] };
  const COL_SPEED = [[255, 170, 60], [90, 200, 255], [90, 255, 140], [255, 90, 200]];
  const COL_COIN = [255, 205, 70];
  const COL_CHECK = [90, 255, 140];
  const COL_P1 = [185, 242, 58];
  const COL_P2 = [58, 214, 242];
  const DIFF_COLORS = [[90, 200, 255], [110, 230, 90], [255, 200, 60], [255, 120, 60], [255, 70, 150], [220, 40, 50]];
  const DARK = [14, 14, 24];

  function mix(c1, c2, f) {
    f = Math.max(0, Math.min(1, f));
    return [Math.trunc(c1[0] + (c2[0] - c1[0]) * f), Math.trunc(c1[1] + (c2[1] - c1[1]) * f), Math.trunc(c1[2] + (c2[2] - c1[2]) * f)];
  }

  const ps = () => (PG.app && PG.app.pixelScale) || 1;

  /** Hintergrund- (which=0) bzw. Bodenfarbe (1) an Welt-x - Farb-Trigger der Reihe nach. */
  function colorAt(lv, xb, which) {
    const d = lv.data;
    let col = (which === 0 ? d.bg : d.ground).slice();
    for (const i of lv.triggers) {
      const tx = lv.ox[i];
      if (tx > xb) break;
      const p = lv.par[i];
      if (p[0] !== which) continue;
      const f = p[4] > 0 ? (xb - tx) / Math.max(1, p[4]) : 1;
      col = mix(col, [p[1], p[2], p[3]], f);
    }
    return col;
  }

  /** Offscreen-Canvas in Pixeldichte; fn(ctx) zeichnet in logischen Einheiten. */
  function offscreen(w, h, fn) {
    const s = ps();
    const c = ui.makeCanvas(Math.max(1, Math.ceil(w * s)), Math.max(1, Math.ceil(h * s)));
    const x = c.getContext("2d");
    x.scale(s, s);
    fn(x);
    return c;
  }

  function glowCanvas(radius, color, strength) {
    const size = Math.max(4, Math.round(radius * 2));
    return offscreen(size, size, (x) => {
      const g = x.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, radius);
      g.addColorStop(0, `rgba(${color[0]},${color[1]},${color[2]},${0.36 * strength})`);
      g.addColorStop(0.45, `rgba(${color[0]},${color[1]},${color[2]},${0.14 * strength})`);
      g.addColorStop(1, `rgba(${color[0]},${color[1]},${color[2]},0)`);
      x.fillStyle = g;
      x.fillRect(0, 0, size, size);
    });
  }

  function blockMasks(lv) {
    if (lv.cacheMasks) return lv.cacheMasks;
    const cells = new Set();
    for (let i = 0; i < lv.n; i++) if (core.KIND_NAMES[lv.kind[i]] === "block") cells.add(lv.ox[i] + "," + lv.oy[i]);
    const masks = new Map();
    for (let i = 0; i < lv.n; i++) {
      if (core.KIND_NAMES[lv.kind[i]] !== "block") continue;
      const x = lv.ox[i], y = lv.oy[i];
      let m = 0;
      if (!cells.has(x + "," + (y + 1))) m |= 1;
      if (!cells.has(x + 1 + "," + y)) m |= 2;
      if (!cells.has(x + "," + (y - 1)) && y > 0) m |= 4;
      if (!cells.has(x - 1 + "," + y)) m |= 8;
      masks.set(i, m);
    }
    lv.cacheMasks = masks;
    return masks;
  }

  class Renderer {
    constructor() {
      this.ts = 0;
      this.w = 0;
      this.h = 0;
      this.cache = new Map();
    }

    resize(w, h, ts) {
      ts = Math.max(8, Math.trunc(ts));
      if (w === this.w && h === this.h && ts === this.ts) return;
      this.w = w;
      this.h = h;
      this.ts = ts;
      this.cache.clear();
    }

    sx(xb, camX) {
      return (xb - camX) * this.ts;
    }

    sy(yb, camY) {
      return this.h - (yb - camY) * this.ts;
    }

    cached(key, fn) {
      let c = this.cache.get(key);
      if (!c) {
        c = fn();
        this.cache.set(key, c);
      }
      return c;
    }

    // ----- Kacheln -------------------------------------------------------
    blockTile(mask, half) {
      return this.cached("blk" + mask + "|" + half, () => {
        const ts = this.ts;
        return offscreen(ts, ts, (x) => {
          let r = [0, 0, ts, ts];
          if (half === 1) r = [0, ts / 2, ts, ts / 2];
          else if (half === 2) r = [0, 0, ts / 2, ts];
          else if (half === 3) r = [0, 0, ts, ts / 2];
          else if (half === 4) r = [ts / 2, 0, ts / 2, ts];
          const g = x.createLinearGradient(0, r[1], 0, r[1] + r[3]);
          g.addColorStop(0, "rgba(34,36,58,0.94)");
          g.addColorStop(1, "rgba(8,8,16,0.94)");
          x.fillStyle = g;
          x.fillRect(r[0], r[1], r[2], r[3]);
          const lw = Math.max(1, Math.trunc(ts / 14));
          x.fillStyle = "rgb(250,252,255)";
          if (half) {
            x.strokeStyle = "rgb(250,252,255)";
            x.lineWidth = lw;
            x.strokeRect(r[0] + lw / 2, r[1] + lw / 2, r[2] - lw, r[3] - lw);
          } else {
            if (mask & 1) x.fillRect(0, 0, ts, lw);
            if (mask & 2) x.fillRect(ts - lw, 0, lw, ts);
            if (mask & 4) x.fillRect(0, ts - lw, ts, lw);
            if (mask & 8) x.fillRect(0, 0, lw, ts);
            const inset = Math.max(3, Math.trunc(ts / 4));
            x.strokeStyle = "rgba(255,255,255,0.1)";
            x.lineWidth = Math.max(1, lw / 2);
            x.strokeRect(inset, inset, ts - 2 * inset, ts - 2 * inset);
          }
        });
      });
    }

    spikeTile(rot, small) {
      return this.cached("spk" + rot + "|" + small, () => {
        const ts = this.ts;
        return offscreen(ts, ts, (x) => {
          x.translate(ts / 2, ts / 2);
          x.rotate((rot * Math.PI) / 2);
          x.translate(-ts / 2, -ts / 2);
          const top = small ? ts * 0.5 : ts * 0.06;
          const pts = [[ts * 0.06, ts - 1], [ts * 0.94, ts - 1], [ts * 0.5, top]];
          draw.polygon(x, [12, 12, 22], pts);
          draw.polygon(x, [60, 64, 90], [[ts * 0.5, top + (ts - top) * 0.28], [ts * 0.34, ts - 3], [ts * 0.5, ts - 3]]);
          draw.polygon(x, [250, 252, 255], pts, Math.max(1, Math.trunc(ts / 16)));
        });
      });
    }

    glow(key, radius, color, strength = 1) {
      return this.cached("glow" + key + "|" + Math.trunc(radius) + "|" + color.join(",") + "|" + strength, () => glowCanvas(radius, color, strength));
    }

    // ----- Hintergrund, Boden, Decke ----------------------------------------
    drawBackdrop(ctx, bg, camX, camY, pulse = 0) {
      const col = mix(bg, [255, 255, 255], 0.06 * pulse);
      draw.rect(ctx, mix(col, [0, 0, 0], 0.25), [0, 0, this.w, this.h]);
      const ts = this.ts;
      [[0.1, 4.6, 0.045], [0.25, 2.2, 0.06]].forEach(([speed, size, alpha], layer) => {
        const px = size * ts;
        const span = px * 2.6;
        const off = PG.mod(camX * speed * ts, span);
        const n = Math.trunc(this.w / span) + 2;
        for (let i = -1; i < n; i++) {
          const x = Math.trunc(i * span - off);
          const k = Math.floor((camX * speed * ts) / span) + i;
          const yb = PG.mod(k * 37 + layer * 11, 7) / 7;
          const y = Math.trunc(this.h * (0.08 + 0.5 * yb) + camY * ts * speed);
          const c = mix(bg, [255, 255, 255], alpha + 0.03 * pulse);
          draw.rect(ctx, c, [x, y, Math.trunc(px), Math.trunc(px)], layer ? Math.max(1, Math.trunc(ts / 12)) : 0, Math.max(2, Math.trunc(ts / 6)));
        }
      });
      const shade = this.cached("shade", () => {
        const c = ui.makeCanvas(1, 64);
        const x = c.getContext("2d");
        const g = x.createLinearGradient(0, 0, 0, 64);
        g.addColorStop(0, "rgba(0,0,20,0.47)");
        g.addColorStop(1, "rgba(0,0,20,0)");
        x.fillStyle = g;
        x.fillRect(0, 0, 1, 64);
        return c;
      });
      ctx.drawImage(shade, 0, 0, this.w, this.h);
    }

    drawGround(ctx, lv, ground, camX, camY, pulse = 0) {
      const ts = this.ts;
      const y0 = Math.round(this.sy(0, camY));
      if (y0 >= this.h) return;
      const dark = mix(ground, [0, 0, 0], 0.35);
      draw.rect(ctx, dark, [0, y0, this.w, this.h - y0]);
      const band = Math.max(2, Math.trunc(ts / 2));
      ctx.save();
      const g = ctx.createLinearGradient(0, y0, 0, y0 + band);
      g.addColorStop(0, ui.col(ground));
      g.addColorStop(1, ui.col(dark));
      ctx.fillStyle = g;
      ctx.fillRect(0, y0, this.w, band);
      ctx.restore();
      const tile = 4 * ts;
      const off = Math.trunc(PG.mod(camX * ts, tile));
      const fug = mix(dark, [255, 255, 255], 0.08);
      for (let i = -1; i < Math.trunc(this.w / tile) + 2; i++) {
        const x = i * tile - off;
        draw.line(ctx, fug, [x, y0 + band], [x, this.h]);
      }
      if (lv && lv.pits.size) {
        const c0 = Math.trunc(camX) - 1, c1 = Math.trunc(camX + this.w / ts) + 1;
        for (let c = c0; c <= c1; c++) {
          if (!lv.pits.has(c)) continue;
          const x = Math.round(this.sx(c, camX));
          draw.rect(ctx, [4, 2, 8], [x, y0, ts + 1, this.h - y0]);
          ctx.save();
          const pg = ctx.createLinearGradient(0, this.h - ts / 3, 0, this.h);
          pg.addColorStop(0, "rgba(255,60,60,0)");
          pg.addColorStop(1, "rgba(255,60,60,0.35)");
          ctx.fillStyle = pg;
          ctx.fillRect(x, this.h - ts / 3, ts, ts / 3);
          ctx.restore();
        }
      }
      const glow = mix(ground, [255, 255, 255], 0.55 + 0.35 * pulse);
      const lw = Math.max(2, Math.trunc(ts / 16));
      draw.rect(ctx, glow, [0, y0 - (lw >> 1), this.w, lw]);
    }

    drawCeiling(ctx, ground, ceilB, camY, pulse = 0) {
      if (ceilB <= 0) return;
      const y1 = Math.round(this.sy(ceilB, camY));
      if (y1 <= 0) return;
      draw.rect(ctx, mix(ground, [0, 0, 0], 0.35), [0, 0, this.w, y1]);
      const lw = Math.max(2, Math.trunc(this.ts / 16));
      draw.rect(ctx, mix(ground, [255, 255, 255], 0.55 + 0.35 * pulse), [0, y1 - (lw >> 1), this.w, lw]);
    }

    // ----- Objekte ------------------------------------------------------------
    drawObjects(ctx, lv, camX, camY, t, pulse, used, editor) {
      const ts = this.ts;
      used = used || new Set();
      const c0 = Math.floor(camX) - 2, c1 = Math.trunc(camX + this.w / ts) + 2;
      if (!lv.cacheCols) {
        const cols = new Map();
        for (let i = 0; i < lv.n; i++) {
          if (!cols.has(lv.ox[i])) cols.set(lv.ox[i], []);
          cols.get(lv.ox[i]).push(i);
        }
        lv.cacheCols = cols;
      }
      const masks = blockMasks(lv);
      const glows = [], portals = [];
      for (let c = c0; c <= c1; c++) {
        for (const i of lv.cacheCols.get(c) || []) {
          const name = core.KIND_NAMES[lv.kind[i]];
          const x = Math.round(this.sx(lv.ox[i], camX));
          const y = Math.round(this.sy(lv.oy[i] + 1, camY));
          if (y > this.h + 3 * ts || y < -4 * ts) continue;
          const cls = lv.cls[i];
          if (name === "block") ctx.drawImage(this.blockTile(masks.has(i) ? masks.get(i) : 15, 0), x, y, ts, ts);
          else if (name === "half") ctx.drawImage(this.blockTile(15, lv.rot[i] + 1), x, y, ts, ts);
          else if (cls === core.C_HAZARD) ctx.drawImage(this.spikeTile(lv.rot[i], name === "spike_s"), x, y, ts, ts);
          else if (cls === core.C_PAD) this.pad(ctx, x, y, lv.val[i], lv.rot[i], t, used.has(i));
          else if (cls === core.C_ORB) glows.push([i, x, y]);
          else if (cls === core.C_MODE || cls === core.C_GRAV || cls === core.C_SPEED) portals.push([i, x, y]);
          else if (cls === core.C_COIN) {
            if (used.has(i)) continue;
            const g = this.glow("coin", ts * 0.9, COL_COIN, 0.8);
            const gw = g.width / ps();
            ctx.drawImage(g, x + ts / 2 - gw / 2, y + ts / 2 - gw / 2, gw, gw);
            drawCoin(ctx, x + ts / 2, y + ts / 2, ts * 0.36, t);
          } else if (editor && cls === core.C_TRIGGER) this.triggerIcon(ctx, x, y, lv.par[i]);
          else if (editor && cls === core.C_PIT) {
            const y0 = Math.round(this.sy(0, camY));
            draw.rect(ctx, [255, 70, 70], [x + 2, y0 + 2, ts - 4, Math.max(3, Math.trunc(ts / 6))]);
          }
        }
      }
      for (const [i, x, y] of glows) this.orb(ctx, x, y, lv.val[i], t, pulse, used.has(i));
      for (const [i, x, y] of portals) this.portal(ctx, x, y, lv.cls[i], lv.val[i], t);
    }

    pad(ctx, x, y, which, rot, t, used) {
      const ts = this.ts;
      const col = COL_PAD[which];
      const g = this.glow("pad", ts * 0.7, col, 0.9);
      const gw = g.width / ps();
      const cy = rot === 0 ? y + ts - ts / 8 : y + ts / 8;
      ctx.drawImage(g, x + ts / 2 - gw / 2, cy - gw / 2, gw, gw);
      const w = Math.trunc(ts * 0.8), h = Math.max(3, Math.trunc(ts / 5));
      const rx = x + (ts - w) / 2, ry = rot === 0 ? y + ts - h : y;
      draw.ellipse(ctx, used ? mix(col, [255, 255, 255], 0.5) : col, [rx, ry - h / 2, w, h * 2]);
      draw.rect(ctx, [12, 12, 22], [rx, rot === 0 ? y + ts - 2 : y, w, 2]);
      for (let k = 0; k < 3; k++) {
        const ph = PG.mod(t * 1.6 + k / 3, 1);
        const px = x + ts * (0.25 + 0.25 * k);
        const dy = ph * ts * 0.9;
        const py = rot === 0 ? y + ts - h - dy : y + h + dy;
        draw.circle(ctx, mix(col, [255, 255, 255], 0.4), [px, py], Math.max(1, ts * 0.05 * (1 - ph)));
      }
    }

    orb(ctx, x, y, which, t, pulse, used) {
      const ts = this.ts;
      const col = COL_PAD[which];
      const cx = x + ts / 2, cy = y + ts / 2;
      const g = this.glow("orb", ts * 0.95, col, 1);
      const gw = g.width / ps();
      ctx.drawImage(g, cx - gw / 2, cy - gw / 2, gw, gw);
      const r = Math.trunc(ts * 0.34);
      draw.circle(ctx, mix(col, [255, 255, 255], 0.25), [cx, cy], r);
      draw.circle(ctx, [255, 255, 255], [cx, cy], r, Math.max(1, Math.trunc(ts / 14)));
      draw.circle(ctx, mix(col, [255, 255, 255], 0.7), [cx - r / 3, cy - r / 3], Math.max(1, r / 3));
      const ring = used ? r + ts * 0.4 : r + ts * (0.12 + 0.18 * pulse);
      draw.circle(ctx, col, [cx, cy], ring, Math.max(1, Math.trunc(ts / 18)));
    }

    portal(ctx, x, y, cls, val, t) {
      const ts = this.ts;
      const cx = x + ts / 2, cy = y + ts / 2;
      if (cls === core.C_SPEED) {
        const col = COL_SPEED[val];
        const g = this.glow("spd", ts * 1.2, col, 0.7);
        const gw = g.width / ps();
        ctx.drawImage(g, cx - gw / 2, cy - gw / 2, gw, gw);
        const n = val + 1, wv = ts * 0.34;
        for (let i = 0; i < n; i++) {
          const ox = cx + (i - (n - 1) / 2) * wv * 0.8;
          const pts = [[ox - wv * 0.5, cy - ts * 0.55], [ox + wv * 0.4, cy], [ox - wv * 0.5, cy + ts * 0.55], [ox - wv * 0.05, cy]];
          draw.polygon(ctx, col, pts);
          draw.polygon(ctx, [255, 255, 255], pts, Math.max(1, Math.trunc(ts / 20)));
        }
        return;
      }
      const col = cls === core.C_MODE ? COL_MODE[val] : COL_GRAV[val];
      const g = this.glow("prt", ts * 1.6, col, 0.8);
      const gw = g.width / ps();
      ctx.drawImage(g, cx - gw / 2, cy - gw / 2, gw, gw);
      const rw = ts * 0.9, rh = ts * 3;
      const rect = [cx - rw / 2, cy - rh / 2, rw, rh];
      const wob = 0.08 * Math.sin(t * 4);
      const iw = rw - ts * (0.35 + wob), ih = rh - ts * 0.5;
      draw.ellipse(ctx, [10, 10, 20], rect);
      draw.ellipse(ctx, mix(col, [0, 0, 0], 0.45), [cx - iw / 2, cy - ih / 2, iw, ih]);
      draw.ellipse(ctx, col, rect, Math.max(2, Math.trunc(ts / 8)));
      const d = Math.max(2, Math.trunc(ts / 5));
      draw.ellipse(ctx, [255, 255, 255], [rect[0] + d / 2, rect[1] + d / 2, rw - d, rh - d], Math.max(1, Math.trunc(ts / 20)));
      const r = Math.max(3, Math.trunc(ts * 0.18));
      if (cls === core.C_GRAV) {
        const dd = val > 0 ? -1 : 1;
        draw.polygon(ctx, [255, 255, 255], [[cx - r, cy - dd * r * 0.4], [cx + r, cy - dd * r * 0.4], [cx, cy + dd * r]]);
      } else this.modeGlyph(ctx, cx, cy, r, val);
    }

    modeGlyph(ctx, cx, cy, r, mode) {
      const white = [255, 255, 255];
      const lw = Math.max(1, Math.trunc(r / 3));
      if (mode === core.CUBE) draw.rect(ctx, white, [cx - r, cy - r, 2 * r, 2 * r], lw);
      else if (mode === core.SHIP) draw.polygon(ctx, white, [[cx - r, cy + r * 0.6], [cx + r, cy + r * 0.2], [cx - r * 0.4, cy - r * 0.8]], lw);
      else if (mode === core.BALL) draw.circle(ctx, white, [cx, cy], r, lw);
      else if (mode === core.UFO) {
        draw.ellipse(ctx, white, [cx - r, cy - r * 0.2, 2 * r, r], lw);
        draw.arc(ctx, white, [cx - r * 0.5, cy - r * 0.8, r, r * 1.2], 0, Math.PI, lw);
      } else draw.lines(ctx, white, false, [[cx - r, cy + r * 0.5], [cx - r * 0.3, cy - r * 0.5], [cx + r * 0.3, cy + r * 0.5], [cx + r, cy - r * 0.5]], lw);
    }

    triggerIcon(ctx, x, y, par) {
      const ts = this.ts;
      const r = [x + ts / 6, y + ts / 6, ts - ts / 3, ts - ts / 3];
      draw.rect(ctx, [par[1], par[2], par[3]], r, 0, Math.max(2, Math.trunc(ts / 6)));
      draw.rect(ctx, [255, 255, 255], r, Math.max(1, Math.trunc(ts / 16)), Math.max(2, Math.trunc(ts / 6)));
      ui.text(ctx, par[0] === 0 ? "BG" : "G", x + ts / 2, y + ts / 2, ui.font(Math.max(9, Math.trunc(ts / 3)), true), [255, 255, 255], "center");
    }

    // ----- Spieler -----------------------------------------------------------
    drawPlayer(ctx, x, y, mode, grav, angle, camX, camY, t, trail, alpha) {
      const ts = this.ts;
      const cx = this.sx(x + 0.5, camX), cy = this.sy(y + 0.5, camY);
      if (trail && trail.length > 1) {
        const pts = trail.map(([px, py]) => [this.sx(px + 0.5, camX), this.sy(py + 0.5, camY)]);
        if (mode === core.WAVE) {
          const w = Math.max(2, Math.trunc(ts / 6));
          draw.lines(ctx, mix(COL_P2, [255, 255, 255], 0.2), false, pts, w + 4);
          draw.lines(ctx, [255, 255, 255], false, pts, Math.max(1, w >> 1));
        } else {
          for (let i = 0; i < pts.length - 1; i++) {
            const f = i / pts.length;
            draw.circle(ctx, mix(COL_P2, [255, 255, 255], f), pts[i], Math.max(1, ts * 0.14 * f));
          }
        }
      }
      ctx.save();
      if (alpha != null) ctx.globalAlpha = alpha;
      ctx.translate(cx, cy);
      if (mode === core.CUBE || mode === core.BALL) ctx.rotate((angle * Math.PI) / 180);
      else if (mode === core.SHIP || mode === core.WAVE) ctx.rotate((-angle * Math.PI) / 180);
      if ((mode === core.SHIP || mode === core.UFO) && grav < 0) ctx.scale(1, -1);
      const lw = Math.max(1, Math.trunc(ts / 14));
      if (mode === core.CUBE) this.cubeFace(ctx, 0, 0, ts * 0.96);
      else if (mode === core.BALL) {
        const r = ts * 0.48;
        draw.circle(ctx, DARK, [0, 0], r);
        draw.circle(ctx, COL_P1, [0, 0], r - lw);
        for (let k = 0; k < 4; k++) {
          const a = (k * Math.PI) / 2;
          draw.line(ctx, DARK, [0, 0], [Math.cos(a) * r, Math.sin(a) * r], lw + 1);
        }
        draw.circle(ctx, COL_P2, [0, 0], r * 0.45);
        draw.circle(ctx, DARK, [0, 0], r * 0.45, lw);
        draw.circle(ctx, DARK, [0, 0], r, lw);
      } else if (mode === core.SHIP) {
        this.cubeFace(ctx, ts * 0.02, -ts * 0.22, ts * 0.5);
        const hull = [[-ts * 0.55, -ts * 0.02], [ts * 0.62, ts * 0.12], [ts * 0.3, ts * 0.42], [-ts * 0.5, ts * 0.42]];
        draw.polygon(ctx, COL_P2, hull);
        draw.polygon(ctx, DARK, hull, lw + 1);
        draw.line(ctx, COL_P1, [-ts * 0.4, ts * 0.2], [ts * 0.35, ts * 0.24], lw + 1);
      } else if (mode === core.UFO) {
        const dw = ts * 0.7, dh = ts * 0.62;
        draw.ellipse(ctx, [200, 240, 255, 90], [-dw / 2, ts * 0.08 - dh, dw, dh]);
        this.cubeFace(ctx, 0, -ts * 0.14, ts * 0.42);
        draw.ellipse(ctx, DARK, [-dw / 2, ts * 0.08 - dh, dw, dh], lw);
        const sw = ts * 1.1, sh = ts * 0.36;
        draw.ellipse(ctx, COL_P2, [-sw / 2, ts * 0.18 - sh / 2, sw, sh]);
        draw.ellipse(ctx, DARK, [-sw / 2, ts * 0.18 - sh / 2, sw, sh], lw + 1);
        for (const k of [-1, 0, 1]) draw.circle(ctx, COL_P1, [k * ts * 0.28, ts * 0.18], Math.max(1, ts / 16));
      } else {
        const pts = [[ts * 0.42, 0], [-ts * 0.34, -ts * 0.3], [-ts * 0.16, 0], [-ts * 0.34, ts * 0.3]];
        draw.polygon(ctx, COL_P1, pts);
        draw.polygon(ctx, DARK, pts, lw + 1);
        draw.circle(ctx, COL_P2, [-ts * 0.02, 0], Math.max(2, ts / 9));
      }
      ctx.restore();
    }

    cubeFace(ctx, cx, cy, size) {
      const lw = Math.max(1, Math.trunc(size / 14));
      const r = [cx - size / 2, cy - size / 2, size, size];
      draw.rect(ctx, COL_P1, r);
      const inner = [r[0] + size / 6, r[1] + size / 6, size - size / 3, size - size / 3];
      draw.rect(ctx, COL_P2, inner);
      draw.rect(ctx, DARK, inner, lw);
      const eye = Math.max(2, size / 7);
      draw.rect(ctx, DARK, [inner[0] + inner[2] / 4 - eye / 2, inner[1] + inner[3] / 3 - eye / 2, eye, eye]);
      draw.rect(ctx, DARK, [inner[0] + inner[2] - inner[2] / 4 - eye / 2, inner[1] + inner[3] / 3 - eye / 2, eye, eye]);
      draw.rect(ctx, DARK, [inner[0] + inner[2] / 4, inner[1] + inner[3] - inner[3] / 3, inner[2] / 2, Math.max(1, lw)]);
      const ear = Math.max(2, size / 6);
      draw.rect(ctx, COL_P2, [r[0] + size / 8, r[1] + 2, ear, ear]);
      draw.rect(ctx, COL_P2, [r[0] + size - size / 8 - ear, r[1] + 2, ear, ear]);
      draw.rect(ctx, DARK, r, lw + 1);
    }
  }

  function drawCoin(ctx, cx, cy, r, t, dim) {
    const squash = Math.abs(Math.cos(t * 2.4));
    const w = Math.max(2, 2 * r * (0.35 + 0.65 * squash)), h = Math.max(2, 2 * r);
    const base = dim ? [120, 110, 90] : COL_COIN;
    draw.ellipse(ctx, mix(base, [0, 0, 0], 0.35), [cx - w / 2 + Math.max(1, r / 6), cy - h / 2, w, h]);
    draw.ellipse(ctx, base, [cx - w / 2, cy - h / 2, w, h]);
    const iw = w - Math.max(2, w / 3), ih = h - Math.max(2, h / 3);
    draw.ellipse(ctx, mix(base, [255, 255, 255], 0.45), [cx - iw / 2, cy - ih / 2, iw, ih], Math.max(1, r / 5));
    draw.ellipse(ctx, [60, 40, 10], [cx - w / 2, cy - h / 2, w, h], Math.max(1, r / 6));
  }

  function drawStar(ctx, cx, cy, r, col) {
    const pts = [];
    for (let k = 0; k < 10; k++) {
      const a = -Math.PI / 2 + (k * Math.PI) / 5;
      const rr = k % 2 === 0 ? r : r * 0.45;
      pts.push([cx + Math.cos(a) * rr, cy + Math.sin(a) * rr]);
    }
    draw.polygon(ctx, col, pts);
    draw.polygon(ctx, [20, 20, 30], pts, Math.max(1, r / 6));
  }

  /** Schwierigkeits-Gesicht (0 leicht .. 5 Dämon). */
  function drawFace(ctx, cx, cy, r, diff) {
    const col = DIFF_COLORS[Math.max(0, Math.min(5, diff))];
    const dark = [18, 14, 24];
    const lw = Math.max(1, Math.trunc(r / 8));
    if (diff >= 5) {
      for (const sgn of [-1, 1]) {
        const horn = [[cx + sgn * r * 0.55, cy - r * 0.55], [cx + sgn * r * 1.05, cy - r * 1.25], [cx + sgn * r * 0.2, cy - r * 0.85]];
        draw.polygon(ctx, [240, 60, 40], horn);
        draw.polygon(ctx, dark, horn, lw);
      }
    }
    draw.circle(ctx, col, [cx, cy], r);
    draw.circle(ctx, mix(col, [255, 255, 255], 0.35), [cx - r / 3, cy - r / 3], Math.max(1, r / 4));
    draw.circle(ctx, dark, [cx, cy], r, lw + 1);
    const ey = cy - r / 5, ex = r * 0.38, er = Math.max(1, r / 6);
    if (diff <= 1) {
      draw.circle(ctx, dark, [cx - ex, ey], er);
      draw.circle(ctx, dark, [cx + ex, ey], er);
      draw.arc(ctx, dark, [cx - r / 2, cy - r / 4, r, r * 0.8], Math.PI * 1.1, Math.PI * 1.9, lw + 1);
    } else if (diff === 2) {
      draw.circle(ctx, dark, [cx - ex, ey], er);
      draw.circle(ctx, dark, [cx + ex, ey], er);
      draw.line(ctx, dark, [cx - r / 3, cy + r / 3], [cx + r / 3, cy + r / 3], lw + 1);
    } else if (diff === 3) {
      for (const sgn of [-1, 1]) {
        draw.line(ctx, dark, [cx + sgn * ex - er * 2, ey + er * 2 * sgn], [cx + sgn * ex + er * 2, ey - er * 2 * sgn], lw + 1);
        draw.circle(ctx, dark, [cx + sgn * ex, ey + er], er);
      }
      draw.arc(ctx, dark, [cx - r / 3, cy + r / 5, r * 0.66, r * 0.6], 0.15, Math.PI - 0.15, lw + 1);
    } else {
      for (const sgn of [-1, 1]) {
        draw.polygon(ctx, diff === 5 ? [255, 255, 255] : dark, [[cx + sgn * ex - er * 2, ey - er], [cx + sgn * ex + er * 2, ey - er], [cx + sgn * (ex - er * 2 * sgn), ey + er * 2]]);
        draw.line(ctx, dark, [cx + sgn * (ex + er * 3), ey - er * 3], [cx + sgn * (ex - er * 2), ey - er], lw + 1);
      }
      const mouth = [cx - r / 2, cy + r / 5, r, Math.max(3, r / 3)];
      draw.rect(ctx, dark, mouth, 0, Math.max(1, r / 8));
      for (let k = 1; k < 4; k++) {
        const x = mouth[0] + (mouth[2] * k) / 4;
        draw.line(ctx, [255, 255, 255], [x, mouth[1]], [x, mouth[1] + mouth[3] - 1], Math.max(1, lw / 2));
      }
    }
  }

  PG.gdDraw = {
    COL_YELLOW, COL_PINK, COL_BLUE, COL_PAD, COL_MODE, COL_GRAV, COL_SPEED, COL_COIN, COL_CHECK, COL_P1, COL_P2,
    DIFF_COLORS, mix, colorAt, blockMasks, Renderer, drawCoin, drawStar, drawFace, offscreen,
  };
})();
