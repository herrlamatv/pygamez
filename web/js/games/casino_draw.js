/*
 * casino_draw.js - Zeichen-Bausteine des Casinos (Port von games/casino_draw.py)
 * ==============================================================================
 * Jetons, Roulette-Kessel (feste Schüssel + drehender Rotor), die Symbole des
 * Lama-Slots und das Zeichnen einer Walze. Alles aus Canvas-Pfaden, teure Teile
 * werden einmal je Größe in Offscreen-Canvases gerendert (x pixelScale).
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const L = PG.casinoLogic;
  const TAU = Math.PI * 2;

  const COL_RED = [196, 36, 48];
  const COL_BLACK = [26, 28, 34];
  const COL_GREEN = [22, 138, 76];
  const COL_GOLD = [232, 190, 92];
  const COL_GOLD_D = [150, 110, 40];
  const COL_WOOD = [92, 52, 28];
  const COL_WOOD_D = [58, 32, 18];
  const COL_FELT = [18, 92, 58];
  const COL_FELT_D = [10, 56, 36];
  const CHIP_COLS = {
    1: [[236, 236, 242], [40, 90, 190]],
    5: [[204, 46, 56], [255, 255, 255]],
    25: [[38, 150, 86], [255, 255, 255]],
    100: [[30, 32, 40], [255, 222, 120]],
    500: [[132, 70, 196], [255, 255, 255]],
  };
  const LINE_COLS = [[255, 214, 64], [84, 200, 255], [255, 110, 150], [130, 235, 120], [255, 150, 60],
    [190, 130, 255], [60, 230, 200], [255, 90, 90], [160, 200, 60], [240, 240, 255]];

  const cache = new Map();
  const scale = () => (PG.app && PG.app.pixelScale) || 1;

  function cached(key, build) {
    const k = key + "|" + scale();
    let c = cache.get(k);
    if (!c) {
      if (cache.size > 260) cache.clear();
      c = build();
      cache.set(k, c);
    }
    return c;
  }

  function canvas(w, h) {
    const s = scale();
    const c = ui.makeCanvas(Math.max(1, Math.ceil(w * s)), Math.max(1, Math.ceil(h * s)));
    const g = c.getContext("2d");
    g.scale(s, s);
    c.lw = w;
    c.lh = h;
    return [c, g];
  }

  function fmtAmount(n) {
    n = Math.trunc(n);
    if (n >= 10000) return Math.floor(n / 1000) + "k";
    if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, "") + "k";
    return String(n);
  }

  function numberColor(n) {
    const c = L.colorOf(n);
    return c === "green" ? COL_GREEN : c === "red" ? COL_RED : COL_BLACK;
  }

  function chipFor(amount) {
    let best = L.CHIP_VALUES[0];
    for (const v of L.CHIP_VALUES) if (amount >= v) best = v;
    return best;
  }

  // ========================================================== Jetons
  function chipCanvas(value, r, label) {
    label = label === undefined ? fmtAmount(value) : label;
    return cached("chip|" + value + "|" + r + "|" + label, () => {
      const [base, txt] = CHIP_COLS[value] || CHIP_COLS[chipFor(value)];
      const size = 2 * r + 4;
      const [c, g] = canvas(size, size);
      const cx = size / 2, cy = size / 2;
      draw.circle(g, ui.mix(base, [0, 0, 0], 0.35), [cx, cy + 1], r);
      draw.circle(g, base, [cx, cy], r);
      const stripe = value !== 1 ? [250, 250, 250] : [40, 90, 190];
      for (let i = 0; i < 6; i++) {
        const a = (i * TAU) / 6;
        const pts = [];
        for (const [da, rr] of [[-0.17, r * 0.99], [0.17, r * 0.99], [0.17, r * 0.74], [-0.17, r * 0.74]]) {
          pts.push([cx + Math.cos(a + da) * rr, cy + Math.sin(a + da) * rr]);
        }
        draw.polygon(g, stripe, pts);
      }
      draw.circle(g, ui.mix(base, [255, 255, 255], 0.12), [cx, cy], r * 0.66);
      draw.circle(g, ui.mix(stripe, base, 0.35), [cx, cy], r * 0.66, 1);
      if (label) {
        const f = ui.font(Math.max(8, Math.floor(r * (label.length <= 2 ? 0.78 : 0.62))), true);
        ui.text(g, label, cx, cy, f, txt, "center");
      }
      return c;
    });
  }

  function drawChip(ctx, value, r, x, y, label, alpha = 1) {
    const c = chipCanvas(value, r, label);
    if (alpha < 1) {
      ctx.save();
      ctx.globalAlpha = alpha;
    }
    ctx.drawImage(c, x - c.lw / 2, y - c.lh / 2, c.lw, c.lh);
    if (alpha < 1) ctx.restore();
  }

  function drawChipStack(ctx, center, amount, r, alpha = 1, lift = 0) {
    let rest = Math.trunc(amount);
    const chips = [];
    for (const v of [...L.CHIP_VALUES].reverse()) {
      while (rest >= v && chips.length < 5) {
        chips.push(v);
        rest -= v;
      }
    }
    if (!chips.length) chips.push(L.CHIP_VALUES[0]);
    chips.reverse();
    const step = Math.max(2, Math.floor(r / 5));
    chips.forEach((v, i) => {
      const top = i === chips.length - 1;
      drawChip(ctx, v, r, center[0], center[1] - lift - i * step, top ? fmtAmount(amount) : "", alpha);
    });
  }

  // ========================================================== Kessel
  class WheelArt {
    constructor(d) {
      this.d = d;
      this.R = d / 2;
      this.trackR = this.R * 0.855;
      this.pocketR = this.R * 0.555;
      this.rotorR = this.R * 0.78;
    }
    _bowl() {
      return cached("bowl|" + this.d, () => {
        const d = this.d, R = this.R;
        const [c, g] = canvas(d + 8, d + 10);
        const cx = d / 2, cy = d / 2;
        draw.circle(g, [0, 0, 0, 90], [cx + 3, cy + 5], R);
        const wood = g.createRadialGradient(cx, cy, R * 0.88, cx, cy, R);
        wood.addColorStop(0, ui.col(COL_WOOD));
        wood.addColorStop(1, ui.col(COL_WOOD_D));
        g.fillStyle = wood;
        g.beginPath();
        g.arc(cx, cy, R, 0, TAU);
        g.fill();
        draw.circle(g, COL_GOLD_D, [cx, cy], R * 0.905, 1);
        const track = g.createRadialGradient(cx, cy, R * 0.8, cx, cy, R * 0.9);
        track.addColorStop(0, ui.col([24, 18, 16]));
        track.addColorStop(1, ui.col([58, 44, 36]));
        g.fillStyle = track;
        g.beginPath();
        g.arc(cx, cy, R * 0.9, 0, TAU);
        g.fill();
        draw.circle(g, [90, 70, 56], [cx, cy], R * 0.8, 1);
        for (let i = 0; i < 8; i++) {
          const a = (i * TAU) / 8 + TAU / 16;
          const rr = R * 0.84;
          const px = cx + Math.sin(a) * rr, py = cy - Math.cos(a) * rr;
          const s = R * 0.035;
          draw.polygon(g, COL_GOLD, [
            [px + Math.sin(a) * s * 1.6, py - Math.cos(a) * s * 1.6],
            [px + Math.cos(a) * s, py + Math.sin(a) * s],
            [px - Math.sin(a) * s * 1.6, py + Math.cos(a) * s * 1.6],
            [px - Math.cos(a) * s, py - Math.sin(a) * s],
          ]);
        }
        g.strokeStyle = "rgba(255,240,220,0.24)";
        g.lineWidth = Math.max(2, R * 0.035);
        g.beginPath();
        g.arc(cx, cy, R * 0.955, Math.PI * 1.05, Math.PI * 1.45);
        g.stroke();
        return c;
      });
    }
    _rotor() {
      return cached("rotor|" + this.d, () => {
        const R = this.R;
        const size = Math.ceil(this.rotorR * 2) + 2;
        const [c, g] = canvas(size, size);
        const cx = size / 2, cy = size / 2;
        const n = L.WHEEL_ORDER.length;
        const seg = TAU / n;
        const ring = (a0, a1, r0, r1, col) => {
          // Canvas-Winkel: 0 = 3 Uhr; unsere Winkel: 0 = 12 Uhr, im Uhrzeigersinn
          g.beginPath();
          g.arc(cx, cy, r1, a0 - Math.PI / 2, a1 - Math.PI / 2);
          g.arc(cx, cy, r0, a1 - Math.PI / 2, a0 - Math.PI / 2, true);
          g.closePath();
          g.fillStyle = ui.col(col);
          g.fill();
        };
        const numFont = ui.font(Math.max(7, Math.floor(R * 0.085)), true);
        L.WHEEL_ORDER.forEach((num, i) => {
          const a0 = i * seg - seg / 2, a1 = i * seg + seg / 2;
          const col = numberColor(num);
          ring(a0, a1, R * 0.62, R * 0.78, col);
          ring(a0, a1, R * 0.5, R * 0.62, ui.mix(col, [0, 0, 0], 0.45));
          g.save();
          g.translate(cx + Math.sin(i * seg) * R * 0.7, cy - Math.cos(i * seg) * R * 0.7);
          g.rotate(i * seg);
          ui.text(g, String(num), 0, 0, numFont, [245, 240, 230], "center");
          g.restore();
        });
        g.strokeStyle = ui.col(COL_GOLD);
        g.lineWidth = 1;
        for (let i = 0; i < n; i++) {
          const a = i * seg - seg / 2;
          g.beginPath();
          g.moveTo(cx + Math.sin(a) * R * 0.5, cy - Math.cos(a) * R * 0.5);
          g.lineTo(cx + Math.sin(a) * R * 0.78, cy - Math.cos(a) * R * 0.78);
          g.stroke();
        }
        draw.circle(g, COL_GOLD, [cx, cy], R * 0.78, 2);
        draw.circle(g, COL_GOLD, [cx, cy], R * 0.62, 1);
        draw.circle(g, COL_GOLD_D, [cx, cy], R * 0.5, 2);
        const cone = g.createRadialGradient(cx, cy, R * 0.05, cx, cy, R * 0.49);
        cone.addColorStop(0, ui.col([196, 142, 80]));
        cone.addColorStop(1, ui.col([120, 72, 38]));
        g.fillStyle = cone;
        g.beginPath();
        g.arc(cx, cy, R * 0.49, 0, TAU);
        g.fill();
        for (let k = 0; k < 4; k++) {
          const a = (k * TAU) / 4;
          const p1 = [cx + Math.sin(a) * R * 0.34, cy - Math.cos(a) * R * 0.34];
          draw.line(g, COL_GOLD_D, [cx, cy], p1, Math.max(2, R * 0.05));
          draw.line(g, COL_GOLD, [cx, cy], p1, Math.max(1, R * 0.025));
          draw.circle(g, COL_GOLD, p1, Math.max(2, R * 0.035));
        }
        draw.circle(g, COL_GOLD_D, [cx, cy], R * 0.09);
        draw.circle(g, [255, 236, 170], [cx, cy], R * 0.06);
        return c;
      });
    }
    pocketAngle(number) {
      return (L.WHEEL_ORDER.indexOf(number) * TAU) / L.WHEEL_ORDER.length;
    }
    draw(ctx, center, rotation, ball, trail) {
      const [cx, cy] = center;
      const bowl = this._bowl();
      ctx.drawImage(bowl, cx - this.d / 2, cy - this.d / 2, bowl.lw, bowl.lh);
      const rotor = this._rotor();
      ctx.save();
      ctx.translate(cx, cy);
      ctx.rotate(rotation);
      ctx.drawImage(rotor, -rotor.lw / 2, -rotor.lh / 2, rotor.lw, rotor.lh);
      ctx.restore();
      if (!ball) return;
      const br = Math.max(3, this.R * 0.045);
      (trail || []).forEach(([a, r], i) => {
        draw.circle(ctx, [236, 238, 244, 40 + 30 * i], [cx + Math.sin(a) * r, cy - Math.cos(a) * r], br);
      });
      const [a, r] = ball;
      const bx = cx + Math.sin(a) * r, by = cy - Math.cos(a) * r;
      draw.circle(ctx, [0, 0, 0, 160], [bx + 1, by + 2], br);
      draw.circle(ctx, [236, 238, 244], [bx, by], br);
      draw.circle(ctx, [255, 255, 255], [bx - br * 0.35, by - br * 0.35], Math.max(1, br / 3));
    }
  }

  // ========================================================== Symbole
  const shade = (col, f) => ui.mix(col, f > 0 ? [255, 255, 255] : [0, 0, 0], Math.abs(f));
  const off = (pts, n, dx = 0.015, dy = 0.02) => pts.map(([x, y]) => [x + n * dx, y + n * dy]);

  function polyline(g, col, pts, width) {
    g.strokeStyle = ui.col(col);
    g.lineWidth = width;
    g.lineCap = "round";
    g.lineJoin = "round";
    g.beginPath();
    pts.forEach(([x, y], i) => (i ? g.lineTo(x, y) : g.moveTo(x, y)));
    g.stroke();
  }

  const SYM = {
    cherry(g, n) {
      const red = [214, 30, 52], stem = [70, 150, 60];
      polyline(g, stem, [[n * 0.33, n * 0.62], [n * 0.46, n * 0.34], [n * 0.58, n * 0.18]], Math.max(2, n / 28));
      polyline(g, stem, [[n * 0.68, n * 0.66], [n * 0.64, n * 0.38], [n * 0.58, n * 0.18]], Math.max(2, n / 28));
      draw.ellipse(g, [60, 170, 70], [n * 0.56, n * 0.12, n * 0.26, n * 0.13]);
      for (const [cx, cy] of [[0.32, 0.68], [0.66, 0.7]]) {
        const r = n * 0.17;
        draw.circle(g, shade(red, -0.35), [n * cx + n * 0.015, n * cy + n * 0.02], r);
        draw.circle(g, red, [n * cx, n * cy], r);
        draw.circle(g, shade(red, 0.55), [n * cx - r * 0.35, n * cy - r * 0.35], r * 0.28);
      }
    },
    lemon(g, n) {
      const y = [250, 214, 50];
      const pts = [[0.14, 0.52], [0.3, 0.3], [0.52, 0.24], [0.74, 0.32], [0.88, 0.5], [0.74, 0.7], [0.52, 0.77], [0.3, 0.7]]
        .map(([a, b]) => [n * a, n * b]);
      draw.polygon(g, shade(y, -0.3), off(pts, n));
      draw.polygon(g, y, pts);
      draw.ellipse(g, y, [n * 0.2, n * 0.26, n * 0.62, n * 0.5]);
      draw.ellipse(g, shade(y, 0.55), [n * 0.32, n * 0.34, n * 0.22, n * 0.1]);
      draw.circle(g, shade(y, -0.35), [n * 0.88, n * 0.5], n * 0.03);
    },
    grape(g, n) {
      const p = [136, 58, 176];
      draw.line(g, [90, 140, 60], [n * 0.5, n * 0.12], [n * 0.52, n * 0.3], Math.max(2, n / 26));
      draw.ellipse(g, [80, 170, 80], [n * 0.52, n * 0.1, n * 0.28, n * 0.15]);
      const r = n * 0.1;
      const berries = [[0.32, 0.3], [0.5, 0.3], [0.68, 0.3], [0.41, 0.47], [0.59, 0.47], [0.32, 0.63], [0.5, 0.63], [0.68, 0.63], [0.41, 0.79], [0.59, 0.79]];
      for (const [cx, cy] of berries) {
        draw.circle(g, shade(p, -0.35), [n * cx + n * 0.01, n * cy + n * 0.015], r);
        draw.circle(g, p, [n * cx, n * cy], r);
        draw.circle(g, shade(p, 0.5), [n * cx - r * 0.35, n * cy - r * 0.35], r * 0.28);
      }
    },
    clover(g, n) {
      const gr = [46, 176, 84];
      polyline(g, shade(gr, -0.3), [[n * 0.5, n * 0.52], [n * 0.56, n * 0.76], [n * 0.66, n * 0.88]], Math.max(3, n / 18));
      for (const pass of [0, 1]) {
        for (let a = 0; a < 4; a++) {
          const ang = (a * TAU) / 4 + TAU / 8;
          const cx = n * 0.5 + Math.cos(ang) * n * 0.17, cy = n * 0.46 + Math.sin(ang) * n * 0.17;
          for (const da of [-0.55, 0.55]) {
            const px = cx + Math.cos(ang + da) * n * 0.07, py = cy + Math.sin(ang + da) * n * 0.07;
            if (pass === 0) draw.circle(g, shade(gr, -0.25), [px + n * 0.01, py + n * 0.015], n * 0.12);
            else draw.circle(g, gr, [px, py], n * 0.12);
          }
        }
      }
      draw.circle(g, shade(gr, 0.35), [n * 0.5, n * 0.46], n * 0.05);
    },
    bell(g, n) {
      const gold = [246, 190, 50];
      const body = [[0.5, 0.14], [0.66, 0.22], [0.72, 0.44], [0.78, 0.64], [0.88, 0.74], [0.12, 0.74], [0.22, 0.64], [0.28, 0.44], [0.34, 0.22]]
        .map(([a, b]) => [n * a, n * b]);
      draw.polygon(g, shade(gold, -0.35), off(body, n));
      draw.polygon(g, gold, body);
      draw.circle(g, gold, [n * 0.5, n * 0.3], n * 0.17);
      draw.rect(g, shade(gold, -0.25), [n * 0.1, n * 0.72, n * 0.8, n * 0.07], 0, n * 0.03);
      draw.circle(g, shade(gold, -0.45), [n * 0.5, n * 0.84], n * 0.07);
      draw.circle(g, shade(gold, -0.2), [n * 0.5, n * 0.12], n * 0.05);
      draw.ellipse(g, shade(gold, 0.6), [n * 0.34, n * 0.24, n * 0.09, n * 0.3]);
    },
    gem(g, n) {
      const c = [70, 210, 240];
      const top = n * 0.26, mid = n * 0.42;
      const pts = [[n * 0.28, top], [n * 0.72, top], [n * 0.9, mid], [n * 0.5, n * 0.88], [n * 0.1, mid]];
      draw.polygon(g, shade(c, -0.45), off(pts, n));
      draw.polygon(g, c, pts);
      draw.polygon(g, shade(c, 0.45), [[n * 0.28, top], [n * 0.5, top], [n * 0.38, mid], [n * 0.1, mid]]);
      draw.polygon(g, shade(c, 0.2), [[n * 0.5, top], [n * 0.72, top], [n * 0.62, mid], [n * 0.38, mid]]);
      draw.polygon(g, shade(c, -0.2), [[n * 0.62, mid], [n * 0.9, mid], [n * 0.5, n * 0.88]]);
      draw.polygon(g, shade(c, 0.15), [[n * 0.38, mid], [n * 0.62, mid], [n * 0.5, n * 0.88]]);
      draw.polygon(g, [255, 255, 255], pts, Math.max(1, n / 60));
      draw.circle(g, [255, 255, 255], [n * 0.3, n * 0.31], n * 0.03);
    },
    seven(g, n) {
      const f = ui.font(Math.floor(n * 0.78), true);
      const o = Math.max(2, n / 40);
      for (const [dx, dy] of [[-o, 0], [o, 0], [0, -o], [0, o], [-o, -o], [o, o], [-o, o], [o, -o]]) {
        ui.text(g, "7", n * 0.5 + dx, n * 0.52 + dy, f, [255, 214, 90], "center");
      }
      ui.text(g, "7", n * 0.5 + o, n * 0.52 + o * 1.5, f, [120, 12, 20], "center");
      ui.gradText(g, "7", n * 0.5, n * 0.52, f, [255, 110, 110], [200, 20, 34], "center");
    },
    lama(g, n) {
      const wool = [250, 244, 230], shadow = [206, 190, 168], dark = [60, 44, 40], pink = [240, 150, 160];
      const glow = [255, 200, 80];
      for (let k = 0; k < 12; k++) {
        const a = (k * TAU) / 12;
        draw.polygon(g, ui.mix(glow, [255, 255, 255], 0.25), [
          [n * 0.5, n * 0.44],
          [n * 0.5 + Math.cos(a - 0.12) * n * 0.48, n * 0.44 + Math.sin(a - 0.12) * n * 0.48],
          [n * 0.5 + Math.cos(a + 0.12) * n * 0.48, n * 0.44 + Math.sin(a + 0.12) * n * 0.48],
        ]);
      }
      draw.circle(g, glow, [n * 0.5, n * 0.44], n * 0.32);
      const cx = n * 0.5, cy = n * 0.44;
      draw.rect(g, shadow, [cx - n * 0.13, cy + n * 0.12, n * 0.26, n * 0.3], 0, n * 0.05);
      llamaHead(g, cx + n * 0.012, cy + n * 0.018, n, shadow, null);
      llamaHead(g, cx, cy, n, wool, pink);
      for (const side of [-1, 1]) {
        const ex = cx + side * n * 0.09, ey = cy - n * 0.03;
        draw.circle(g, dark, [ex, ey], n * 0.035);
        draw.circle(g, [255, 255, 255], [ex - n * 0.01, ey - n * 0.012], n * 0.011);
        draw.circle(g, pink, [cx + side * n * 0.15, cy + n * 0.07], n * 0.03);
      }
      draw.circle(g, dark, [cx - n * 0.035, cy + n * 0.1], n * 0.013);
      draw.circle(g, dark, [cx + n * 0.035, cy + n * 0.1], n * 0.013);
      g.strokeStyle = ui.col(dark);
      g.lineWidth = Math.max(1, n / 60);
      g.beginPath();
      g.arc(cx, cy + n * 0.13, n * 0.04, 0.15 * Math.PI, 0.85 * Math.PI);
      g.stroke();
      const band = new PG.Rect(0, 0, n * 0.7, n * 0.2);
      band.center = [n * 0.5, n * 0.84];
      draw.rect(g, [150, 30, 90], [band.x, band.y + n / 60, band.w, band.h], 0, n * 0.06);
      draw.rect(g, [214, 51, 108], band, 0, n * 0.06);
      draw.rect(g, [255, 214, 90], band, Math.max(1, n / 60), n * 0.06);
      ui.text(g, "WILD", band.centerx, band.centery, ui.font(Math.floor(n * 0.15), true), [255, 244, 220], "center");
    },
    coin(g, n) {
      const gold = [246, 196, 60];
      const c = [n * 0.5, n * 0.5];
      draw.circle(g, shade(gold, -0.45), [c[0] + n * 0.02, c[1] + n * 0.03], n * 0.4);
      draw.circle(g, shade(gold, -0.15), c, n * 0.4);
      draw.circle(g, gold, c, n * 0.34);
      draw.circle(g, shade(gold, -0.25), c, n * 0.34, Math.max(2, n / 45));
      for (let k = 0; k < 24; k++) {
        const a = (k * TAU) / 24;
        draw.line(g, shade(gold, -0.35), [c[0] + Math.cos(a) * n * 0.37, c[1] + Math.sin(a) * n * 0.37],
          [c[0] + Math.cos(a) * n * 0.4, c[1] + Math.sin(a) * n * 0.4], Math.max(1, n / 80));
      }
      llamaHead(g, c[0] + n * 0.01, c[1] + n * 0.05, n * 0.72, shade(gold, -0.3), null);
      llamaHead(g, c[0], c[1] + n * 0.04, n * 0.72, shade(gold, 0.3), null);
      draw.circle(g, shade(gold, -0.5), [c[0] - n * 0.06, c[1] + n * 0.02], n * 0.02);
      draw.circle(g, shade(gold, -0.5), [c[0] + n * 0.06, c[1] + n * 0.02], n * 0.02);
      g.strokeStyle = "rgb(255,250,220)";
      g.lineWidth = Math.max(2, n / 40);
      g.beginPath();
      g.arc(c[0], c[1], n * 0.3, Math.PI * 1.05, Math.PI * 1.45);
      g.stroke();
      for (const [sx, sy, r] of [[0.8, 0.18, 0.06], [0.18, 0.8, 0.04]]) {
        const x = n * sx, y = n * sy, q = n * r;
        draw.polygon(g, [255, 255, 230], [[x, y - q], [x + q * 0.3, y - q * 0.3], [x + q, y], [x + q * 0.3, y + q * 0.3],
          [x, y + q], [x - q * 0.3, y + q * 0.3], [x - q, y], [x - q * 0.3, y - q * 0.3]]);
      }
    },
  };

  function llamaHead(g, cx, cy, n, wool, dark) {
    for (const side of [-1, 1]) {
      draw.polygon(g, wool, [[cx + side * n * 0.1, cy - n * 0.16], [cx + side * n * 0.2, cy - n * 0.44], [cx + side * n * 0.26, cy - n * 0.14]]);
      if (dark) {
        draw.polygon(g, dark, [[cx + side * n * 0.14, cy - n * 0.17], [cx + side * n * 0.2, cy - n * 0.36], [cx + side * n * 0.23, cy - n * 0.16]]);
      }
    }
    draw.ellipse(g, wool, [cx - n * 0.21, cy - n * 0.22, n * 0.42, n * 0.4]);
    for (let k = 0; k < 5; k++) draw.circle(g, wool, [cx - n * 0.14 + k * n * 0.07, cy - n * 0.22], n * 0.06);
    draw.ellipse(g, wool, [cx - n * 0.15, cy + n * 0.02, n * 0.3, n * 0.24]);
  }

  function symbolCanvas(sym, size, blur = false) {
    return cached("sym|" + sym + "|" + size + "|" + (blur ? 1 : 0), () => {
      const [c, g] = canvas(size, size);
      if (blur) {
        const sharp = symbolCanvas(sym, size);
        for (const [dy, a] of [[-size / 7, 0.28], [size / 7, 0.28], [0, 0.6]]) {
          g.globalAlpha = a;
          g.drawImage(sharp, 0, dy, size, size);
        }
        g.globalAlpha = 1;
        return c;
      }
      SYM[sym](g, size);
      return c;
    });
  }

  /** Walze zeichnen - pos = gebrochene Streifenposition der obersten Zeile. */
  function drawReel(ctx, rect, strip, pos, cell, speed, dimRows, pop) {
    const n = strip.length;
    const base = Math.floor(pos);
    const frac = pos - base;
    const blur = speed >= 8;
    const pad = Math.max(2, Math.floor(cell / 14));
    const size = cell - 2 * pad;
    ctx.save();
    ctx.beginPath();
    ctx.rect(rect.x, rect.y, rect.w, rect.h);
    ctx.clip();
    for (let i = -1; i < L.ROWS + 1; i++) {
      const sym = strip[PG.mod(base + i, n)];
      const y = rect.y + Math.round((i - frac) * cell);
      const row = frac < 1e-6 ? i : null;
      const sc = pop && row !== null && pop[row] ? pop[row] : 1;
      const img = symbolCanvas(sym, size, blur && sc === 1);
      const sz = size * sc;
      ctx.drawImage(img, rect.centerx - sz / 2, y + cell / 2 - sz / 2, sz, sz);
      if (dimRows && row !== null && dimRows.has(row)) draw.rect(ctx, [0, 0, 0, 120], [rect.x, y, rect.w, cell]);
    }
    const band = Math.max(4, rect.h / 4);
    const gTop = ctx.createLinearGradient(0, rect.y, 0, rect.y + band);
    gTop.addColorStop(0, "rgba(0,0,0,0.58)");
    gTop.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = gTop;
    ctx.fillRect(rect.x, rect.y, rect.w, band);
    const gBot = ctx.createLinearGradient(0, rect.bottom - band, 0, rect.bottom);
    gBot.addColorStop(0, "rgba(0,0,0,0)");
    gBot.addColorStop(1, "rgba(0,0,0,0.58)");
    ctx.fillStyle = gBot;
    ctx.fillRect(rect.x, rect.bottom - band, rect.w, band);
    ctx.restore();
  }

  PG.casinoDraw = {
    COL_RED, COL_BLACK, COL_GREEN, COL_GOLD, COL_GOLD_D, COL_FELT, COL_FELT_D, LINE_COLS,
    cached, canvas, fmtAmount, numberColor, chipCanvas, drawChip, drawChipStack, WheelArt,
    symbolCanvas, drawReel, clearCache: () => cache.clear(),
  };
})();
