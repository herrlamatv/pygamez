/*
 * lamacrossing_draw.js - Zeichen-Bausteine von Lama Crossing (Port von games/lamacrossing_draw.py)
 * ================================================================================================
 * Lama (stehen / springen / fallen, Schal in der Akzentfarbe), Hintergrund und
 * Ferne je Welt (Hängebrücke, Fluss, Eissee), Ufer, Brückenseile, Kacheln
 * (Planke / Stein / Eisscholle), Risslinien und die Welt-Piktogramme.
 * Alle Formen hängen nur von Größe, Index und Zeit ab - kein Zufall.
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const TAU = Math.PI * 2;

  const WOOL = [250, 244, 230];
  const WOOL_D = [214, 200, 178];
  const DARK = [60, 44, 40];
  const HOOF = [82, 60, 52];
  const PINK = [240, 150, 160];

  const cache = new Map();
  const scale = () => (PG.app && PG.app.pixelScale) || 1;

  function cached(key, build) {
    const k = key + "|" + scale();
    let c = cache.get(k);
    if (!c) {
      if (cache.size > 120) cache.clear();
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

  function gradient(g, x, y, w, h, top, bottom) {
    const gr = g.createLinearGradient(0, y, 0, y + h);
    gr.addColorStop(0, ui.col(top));
    gr.addColorStop(1, ui.col(bottom));
    g.fillStyle = gr;
    g.fillRect(x, y, w, h);
  }

  // ============================================================ Lama
  function leg(g, x0, y0, x1, y1, w, col) {
    draw.line(g, col, [x0, y0], [x1, y1], w);
    draw.circle(g, HOOF, [x1, y1], w * 0.62);
  }

  function drawLlama(g, n, pose, scarf) {
    const lw = Math.max(2, n * 0.055);
    const hip = [n * 0.34, n * 0.62], sh = [n * 0.62, n * 0.62];
    let legs;
    if (pose === "hop") {
      legs = [[[-0.02, 0], [-0.12, 0.22]], [[0.04, 0], [-0.04, 0.24]], [[-0.02, 0], [0.1, 0.2]], [[0.04, 0], [0.16, 0.18]]];
    } else if (pose === "fall") {
      legs = [[[-0.02, 0], [-0.14, -0.1]], [[0.04, 0], [-0.08, 0.16]], [[-0.02, 0], [0.14, -0.08]], [[0.04, 0], [0.12, 0.16]]];
    } else {
      legs = [[[-0.02, 0], [-0.03, 0.31]], [[0.04, 0], [0.05, 0.31]], [[-0.02, 0], [-0.01, 0.31]], [[0.04, 0], [0.06, 0.31]]];
    }
    const legAt = (i, col) => {
      const base = i < 2 ? hip : sh;
      const [[ax, ay], [bx, by]] = legs[i];
      leg(g, base[0] + ax * n, base[1] + ay * n, base[0] + bx * n, base[1] + by * n, lw, col);
    };
    legAt(1, WOOL_D);
    legAt(3, WOOL_D);
    draw.circle(g, WOOL_D, [n * 0.215, n * 0.47], n * 0.055);
    draw.ellipse(g, WOOL_D, [n * 0.22, n * 0.44, n * 0.54, n * 0.27]);
    draw.ellipse(g, WOOL, [n * 0.22, n * 0.42, n * 0.54, n * 0.27]);
    for (let i = 0; i < 6; i++) draw.circle(g, WOOL, [n * (0.27 + i * 0.085), n * 0.44], n * 0.055);
    draw.rect(g, WOOL, [n * 0.6, n * 0.17, n * 0.12, n * 0.38], 0, n * 0.05);
    draw.polygon(g, WOOL, [[n * 0.63, n * 0.12], [n * 0.655, n * 0.01], [n * 0.69, n * 0.11]]);
    draw.polygon(g, PINK, [[n * 0.645, n * 0.11], [n * 0.66, n * 0.04], [n * 0.68, n * 0.11]]);
    draw.ellipse(g, WOOL, [n * 0.59, n * 0.09, n * 0.24, n * 0.17]);
    draw.ellipse(g, WOOL, [n * 0.72, n * 0.14, n * 0.17, n * 0.12]);
    for (let i = 0; i < 3; i++) draw.circle(g, WOOL, [n * (0.62 + i * 0.045), n * 0.1], n * 0.035);
    const eye = [n * 0.765, n * 0.155];
    if (pose === "fall") {
      const d = n * 0.022, w = Math.max(1, lw / 2);
      draw.line(g, DARK, [eye[0] - d, eye[1] - d], [eye[0] + d, eye[1] + d], w);
      draw.line(g, DARK, [eye[0] - d, eye[1] + d], [eye[0] + d, eye[1] - d], w);
      draw.circle(g, DARK, [n * 0.86, n * 0.225], n * 0.022);
    } else {
      draw.circle(g, DARK, eye, n * 0.024);
      draw.circle(g, [255, 255, 255], [eye[0] - n * 0.007, eye[1] - n * 0.008], n * 0.008);
      draw.arc(g, DARK, [n * 0.8, n * 0.19, n * 0.07, n * 0.05], Math.PI * 1.15, Math.PI * 1.85, Math.max(1, lw / 3));
    }
    draw.circle(g, PINK, [n * 0.79, n * 0.205], n * 0.022);
    draw.circle(g, DARK, [n * 0.88, n * 0.18], n * 0.012);
    legAt(0, WOOL);
    legAt(2, WOOL);
    const darkScarf = ui.mix(scarf, [0, 0, 0], 0.35);
    draw.rect(g, scarf, [n * 0.585, n * 0.33, n * 0.15, n * 0.07], 0, n * 0.03);
    let end = [[n * 0.61, n * 0.37], [n * 0.66, n * 0.38], [n * 0.56, n * 0.52], [n * 0.5, n * 0.49]];
    if (pose === "hop") end = [[n * 0.61, n * 0.36], [n * 0.66, n * 0.37], [n * 0.48, n * 0.42], [n * 0.46, n * 0.37]];
    draw.polygon(g, darkScarf, end);
    draw.line(g, darkScarf, [n * 0.6, n * 0.355], [n * 0.73, n * 0.355], Math.max(1, lw / 3));
  }

  function llamaCanvas(size, pose, scarf) {
    return cached("llama|" + size + "|" + pose + "|" + scarf.join(","), () => {
      const [c, g] = canvas(size, size);
      drawLlama(g, size, pose, scarf);
      return c;
    });
  }

  // ============================================================ Welten
  const SKY = {
    bridge: [[58, 82, 140], [238, 170, 120]],
    river: [[120, 186, 236], [222, 238, 250]],
    ice: [[150, 182, 214], [232, 240, 248]],
  };
  const FAR = {
    bridge: [[176, 104, 84], [132, 74, 66]],
    river: [[70, 128, 84], [46, 98, 66]],
    ice: [[214, 228, 242], [168, 192, 220]],
  };

  function ridge(x, w, h, seed) {
    const a = (TAU * x) / w;
    return h * (0.5 + 0.22 * Math.sin(a * 2 + seed) + 0.16 * Math.sin(a * 5 + seed * 2.3) + 0.08 * Math.sin(a * 11 + seed * 0.7));
  }

  function background(world, w, h, pathY) {
    return cached("bg|" + world + "|" + w + "|" + h + "|" + pathY, () => {
      const [c, g] = canvas(w, h);
      const [top, bot] = SKY[world];
      gradient(g, 0, 0, w, pathY, top, bot);
      if (world === "bridge") {
        const sun = [w * 0.78, pathY * 0.42];
        for (let i = 5; i > 0; i--) draw.circle(g, [255, 214, 150, 26], sun, pathY * (0.06 + 0.035 * i));
        draw.circle(g, [255, 226, 170], sun, pathY * 0.07);
        gradient(g, 0, pathY, w, h - pathY, [96, 54, 48], [22, 14, 22]);
        for (let i = 0; i < 7; i++) {
          const x = w * (0.07 + 0.145 * i) + Math.sin(i * 2.3) * w * 0.03;
          const top2 = pathY + (h - pathY) * (0.3 + 0.3 * Math.abs(Math.sin(i * 1.7)));
          const bw = w * (0.035 + 0.015 * Math.abs(Math.cos(i * 1.3)));
          draw.polygon(g, [66, 38, 40], [[x - bw, h], [x - bw * 0.45, top2 + bw * 0.4], [x, top2], [x + bw * 0.5, top2 + bw * 0.5], [x + bw, h]]);
          draw.line(g, [112, 66, 58], [x - bw * 0.45, top2 + bw * 0.4], [x, top2], 2);
        }
      } else if (world === "river") {
        gradient(g, 0, pathY - h * 0.1, w, h - pathY + h * 0.1, [78, 156, 200], [26, 78, 124]);
      } else {
        gradient(g, 0, pathY - h * 0.08, w, h - pathY + h * 0.08, [34, 78, 108], [10, 30, 52]);
      }
      return c;
    });
  }

  function farLayer(world, w, h, pathY) {
    return cached("far|" + world + "|" + w + "|" + h + "|" + pathY, () => {
      const [c, g] = canvas(w, pathY);
      const [c1, c2] = FAR[world];
      const base = pathY - (world === "river" ? Math.floor(h * 0.1) : world === "ice" ? Math.floor(h * 0.08) : 0);
      [[c1, 0.42, 1.0], [c2, 0.26, 2.6]].forEach(([col, amp, seed], layer) => {
        let pts = [[0, base]];
        for (let i = 0; i <= w; i += 4) pts.push([i, base - ridge(i, w, pathY * amp, seed)]);
        pts.push([w, base]);
        if (world === "ice" && layer === 0) {
          draw.polygon(g, [252, 253, 255], pts);
          pts = pts.map(([x, y]) => [x, y + (x > 0 && x < w ? 5 : 0)]);
        }
        draw.polygon(g, col, pts);
      });
      if (world === "river") draw.rect(g, [96, 160, 92], [0, base - 4, w, 6]);
      return c;
    });
  }

  // ============================================================ Ufer + Seile
  function drawBank(g, world, xEdge, side, y, h, k, w) {
    let x0, x1;
    if (side < 0) {
      if (xEdge < 0) return;
      x0 = -10;
      x1 = xEdge;
    } else {
      if (xEdge > w) return;
      x0 = xEdge;
      x1 = w + 10;
    }
    const lip = xEdge;
    if (world === "bridge") {
      const pts = side < 0
        ? [[x0, y], [lip, y], [lip - 18 * k, h], [x0, h]]
        : [[lip, y], [x1, y], [x1, h], [lip + 18 * k, h]];
      draw.polygon(g, [150, 92, 66], pts);
      for (let i = 1; i < 6; i++) {
        const yy = y + ((h - y) * i) / 6, off = (18 * k * i) / 6;
        const a = side < 0 ? [x0, yy] : [lip + off, yy];
        const b = side < 0 ? [lip - off, yy] : [x1, yy];
        draw.line(g, [104, 62, 48], a, b, Math.max(1, 2 * k));
      }
      draw.rect(g, [112, 168, 84], [x0, y - 6 * k, x1 - x0, 8 * k]);
    } else if (world === "river") {
      const sand = 22 * k;
      draw.rect(g, [88, 156, 74], [x0, y - 8 * k, x1 - x0, h]);
      for (let i = 1; i < 5; i++) {
        const yy = y + ((h - y) * i) / 5;
        draw.line(g, [78, 142, 66], [x0, yy], [x1, yy], Math.max(1, 2 * k));
      }
      const sandPts = side < 0
        ? [[lip - sand, y - 8 * k], [lip, y - 4 * k], [lip + 6 * k, h], [lip - sand, h]]
        : [[lip + sand, y - 8 * k], [lip, y - 4 * k], [lip - 6 * k, h], [lip + sand, h]];
      draw.polygon(g, [222, 200, 146], sandPts);
      draw.line(g, [250, 252, 255], [lip, y - 4 * k], [lip - side * 6 * k, h], Math.max(1, 3 * k));
      draw.rect(g, [110, 184, 92], [x0, y - 10 * k, x1 - x0, 4 * k]);
    } else {
      draw.rect(g, [236, 244, 250], [x0, y - 4 * k, x1 - x0, h]);
      draw.rect(g, [196, 214, 232], [x0, y + 10 * k, x1 - x0, 4 * k]);
      draw.line(g, [255, 255, 255], [x0, y - 4 * k], [x1, y - 4 * k], Math.max(1, 2 * k));
    }
  }

  function drawRopes(g, points, k, w) {
    const rope = [120, 82, 48];
    const lw = Math.max(1, 2 * k);
    const rail = 46 * k;
    for (let i = 0; i < points.length - 1; i++) {
      const [xa, ya] = points[i], [xb, yb] = points[i + 1];
      if (xb < -40 || xa > w + 40) continue;
      const mx = (xa + xb) / 2, my = (ya + yb) / 2 + 5 * k;
      draw.lines(g, rope, false, [[xa, ya], [mx, my], [xb, yb]], lw);
      draw.lines(g, rope, false, [[xa, ya - rail], [mx, my - rail + 2 * k], [xb, yb - rail]], lw);
      draw.line(g, rope, [xb, yb], [xb, yb - rail], Math.max(1, lw - 1));
    }
  }

  // ============================================================ Kacheln
  function drawTile(g, world, cx, y, tw, k, t, idx, alpha = 1) {
    if (alpha <= 0) return;
    if (alpha < 1) {
      g.save();
      g.globalAlpha *= alpha;
      drawTile(g, world, cx, y, tw, k, t, idx, 1);
      g.restore();
      return;
    }
    if (world === "bridge") {
      const th = Math.max(5, 13 * k);
      const r = new PG.Rect(cx - tw / 2, y, tw, th);
      const rad = Math.max(1, 3 * k);
      draw.rect(g, [96, 60, 34], [r.x, r.y + Math.max(1, 3 * k), r.w, r.h], 0, rad);
      draw.rect(g, [168, 112, 66], r, 0, rad);
      for (const j of [0.3, 0.68]) {
        const yy = r.y + r.h * j;
        draw.line(g, [132, 84, 48], [r.x + 4, yy], [r.right - 4 - 6 * j * k, yy], 1);
      }
      for (const xx of [r.x + 5 * k, r.right - 5 * k]) draw.circle(g, [70, 58, 54], [xx, r.y + r.h / 2], Math.max(1, 1.6 * k));
    } else if (world === "river") {
      const bob = Math.sin(t * 2.2 + idx * 1.3) * 1.5 * k;
      const h = tw * 0.4;
      const r = new PG.Rect(cx - tw / 2, y - h * 0.22 + bob, tw, h);
      const ring = new PG.Rect(cx - tw * 0.625, y + h * 0.42 + bob - h * 0.31, tw * 1.25, h * 0.62);
      draw.ellipse(g, [170, 214, 236], ring, Math.max(1, 2 * k));
      draw.ellipse(g, [82, 88, 96], [r.x, r.y + 4 * k, r.w, r.h]);
      draw.ellipse(g, [128, 134, 142], r);
      const tw2 = tw - tw * 0.28, th2 = h - h * 0.45;
      draw.ellipse(g, [170, 176, 184], [r.centerx - tw2 / 2 - tw * 0.05, r.centery - th2 / 2 - h * 0.12, tw2, th2]);
      draw.ellipse(g, [96, 140, 92], [r.x + tw * 0.62, r.y + h * 0.1, tw * 0.16, h * 0.2]);
    } else {
      const h = tw * 0.34;
      const side = Math.max(3, 7 * k);
      const pts = [];
      for (let j = 0; j < 7; j++) {
        const a = (TAU * j) / 7 + idx * 0.7;
        const rad = 0.5 + 0.06 * Math.sin(idx * 2.1 + j * 1.7);
        pts.push([cx + Math.cos(a) * tw * 0.8 * rad * 1.1, y + h * 0.5 + Math.sin(a) * h * rad]);
      }
      draw.polygon(g, [24, 52, 76], pts.map(([x, yy]) => [x, yy + side + 3 * k]));
      draw.polygon(g, [164, 202, 226], pts.map(([x, yy]) => [x, yy + side]));
      draw.polygon(g, [228, 242, 250], pts);
      draw.line(g, [255, 255, 255], [cx - tw * 0.22, y + h * 0.28], [cx - tw * 0.02, y + h * 0.18], Math.max(1, 2 * k));
    }
  }

  function crackLines(cx, y, tw, idx, f) {
    const lines = [];
    for (let j = 0; j < 4; j++) {
      const a = (TAU * j) / 4 + idx * 0.9 + 0.4;
      const pts = [[cx, y + tw * 0.15]];
      for (let step = 1; step < 5; step++) {
        if (step / 4 > f) break;
        const r = tw * 0.16 * step;
        const wob = Math.sin(idx * 3.1 + j * 2.0 + step * 1.9) * 0.45;
        pts.push([cx + Math.cos(a + wob) * r, y + tw * 0.15 + Math.sin(a + wob) * r * 0.42]);
      }
      if (pts.length > 1) lines.push(pts);
    }
    return lines;
  }

  function worldIcon(g, world, r, col) {
    const cx = r.centerx, cy = r.centery, u = r.w / 10;
    const lw = Math.max(1, u * 0.8);
    if (world === "bridge") {
      draw.lines(g, col, false, [[r.x + u, cy - u * 2], [cx, cy - u * 0.8], [r.right - u, cy - u * 2]], lw);
      draw.lines(g, col, false, [[r.x + u, cy + u], [cx, cy + u * 2.2], [r.right - u, cy + u]], lw);
      for (let i = -2; i <= 2; i++) {
        const x = cx + i * u * 1.6, dy = u * (2.2 - 0.3 * i * i) * 0.5;
        draw.line(g, col, [x, cy + u + dy - u * 0.6], [x, cy + u + dy + u * 0.4], lw);
      }
    } else if (world === "river") {
      for (let i = 0; i < 3; i++) {
        const yy = cy + u * (1.0 + i * 1.3);
        draw.lines(g, col, false, [[r.x + u, yy], [cx - u * 1.5, yy - u * 0.6], [cx + u * 1.5, yy + u * 0.3], [r.right - u, yy - u * 0.4]], Math.max(1, lw - 1));
      }
      draw.ellipse(g, col, [cx - u * 2.4, cy - u * 2.2, u * 4.8, u * 2.6]);
    } else {
      draw.polygon(g, col, [[r.x + u, cy], [cx - u * 1.5, cy - u * 2.0], [cx + u * 2.5, cy - u * 1.6], [r.right - u, cy + u * 0.4],
        [cx + u * 0.5, cy + u * 1.8], [cx - u * 2.6, cy + u * 1.4]], lw);
      draw.lines(g, col, false, [[cx - u * 0.5, cy - u * 1.6], [cx + u * 0.2, cy - u * 0.2], [cx - u * 0.6, cy + u * 0.6], [cx + u * 0.4, cy + u * 1.6]],
        Math.max(1, lw - 1));
    }
  }

  PG.lamaCrossingDraw = {
    llamaCanvas, background, farLayer, drawBank, drawRopes, drawTile, crackLines, worldIcon,
    clearCache: () => cache.clear(),
  };
})();
