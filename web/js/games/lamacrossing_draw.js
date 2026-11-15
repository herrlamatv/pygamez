/*
 * lamacrossing_draw.js - Zeichen-Bausteine von Lama Crossing (Port von games/lamacrossing_draw.py)
 * ================================================================================================
 * - llamaCanvas   : das Lama von der Seite (stehen / springen / fallen), Schal in der
 *                   Akzentfarbe, gecacht je Größe und Pose,
 * - flatLlama     : das von einem Hindernis plattgedrückte Lama (gecacht),
 * - background    : Himmel + Ferne je Welt bzw. Asphalt von oben (Straße), gecacht,
 * - farLayer      : kachelbare Silhouetten-Ebene für den Parallax-Effekt (nicht Straße),
 * - drawTile      : Planke, Stein, Eisscholle bzw. Zebrastreifen eines Schritts,
 * - drawBank      : Start- und Zielufer bzw. Gehweg mit Bordstein, Pflaster, Hecke, Rasen,
 * - drawLanes     : gestrichelte Fahrbahnmarkierungen der Straße,
 * - drawRopes     : Seile der Hängebrücke,
 * - obstacleLen   : senkrechte Länge eines Hindernisses (Treffer-Test),
 * - drawObstacle  : Autos + Lieferwagen von oben (2-fach Geometrie, gecacht je Größe/Art),
 *                   Felsbrocken (Brücke), Treibholz (Fluss), Eisblock (Eissee),
 * - drawBarrier   : rot-weiße Absperrung über einer geschafften Spur,
 * - drawTireMarks : Reifenspuren über dem plattgedrückten Lama,
 * - worldIcon     : Piktogramm je Welt für die Welt-Knöpfe.
 * Alle Formen hängen nur von Größe, Index und Zeit ab - kein Zufall.
 *
 * pygame zeichnet auf SRCALPHA-Flächen mit "Alpha ersetzen"; wo das Python-Original
 * mehrere halbdurchsichtige Formen ineinander zeichnet (Schatten, Lichthof, Abdruck),
 * werden die Alphawerte hier so umgerechnet, dass dasselbe Ergebnis entsteht.
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

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

  // Wiederverwendete Zwischenfläche für halbdurchsichtige Teile (wie pygame
  // layer + set_alpha: erst ganz zeichnen, dann als Ganzes ausblenden).
  let scratch = null, scratchG = null;
  function layer(w, h) {
    const s = scale();
    const pw = Math.max(1, Math.ceil(w * s)), ph = Math.max(1, Math.ceil(h * s));
    if (!scratch || scratch.width < pw || scratch.height < ph) {
      scratch = ui.makeCanvas(Math.max(pw, scratch ? scratch.width : 0), Math.max(ph, scratch ? scratch.height : 0));
      scratchG = scratch.getContext("2d");
    }
    scratchG.setTransform(1, 0, 0, 1, 0, 0);
    scratchG.clearRect(0, 0, pw, ph);
    scratchG.setTransform(s, 0, 0, s, 0, 0);
    return { c: scratch, g: scratchG, pw, ph, w: pw / s, h: ph / s };
  }
  function blitLayer(ctx, l, x, y, alpha) {
    ctx.save();
    ctx.globalAlpha *= alpha / 255;
    ctx.drawImage(l.c, 0, 0, l.pw, l.ph, x, y, l.w, l.h);
    ctx.restore();
  }

  /** Alphawerte (0..255), die ineinander gezeichnet wie "ersetzen" wirken. */
  function nestedAlphas(list) {
    const out = [];
    let prev = 0;
    for (const a of list) {
      const f = a / 255;
      out.push(255 * (prev >= 1 ? 1 : 1 - (1 - f) / (1 - prev)));
      prev = f;
    }
    return out;
  }

  function gradient(g, x, y, w, h, top, bottom) {
    const gr = g.createLinearGradient(0, y, 0, y + h);
    gr.addColorStop(0, ui.col(top));
    gr.addColorStop(1, ui.col(bottom));
    g.fillStyle = gr;
    g.fillRect(x, y, w, h);
  }

  function fill(g, col, x, y, w, h) {
    if (w <= 0 || h <= 0) return;
    g.fillStyle = ui.col(col);
    g.fillRect(x, y, w, h);
  }

  /** wie pygame _rrect: abgerundetes Rechteck mit ganzzahligen Werten. */
  function rrect(g, col, x, y, w, h, r) {
    draw.rect(g, col, [trunc(x), trunc(y), Math.max(1, trunc(w)), Math.max(1, trunc(h))], 0, Math.max(0, trunc(r)));
  }

  /** pygame Rect.inflate (C-Ganzzahldivision). */
  function inflate(r, dx, dy) {
    return [r[0] - trunc(dx / 2), r[1] - trunc(dy / 2), r[2] + dx, r[3] + dy];
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

  /**
   * Plattgedrücktes Lama (Pose "fall" gestaucht), gecacht. Fläche
   * int(size*1.6) x max(4, int(size*0.34)) mit dunklem Abdruck darunter.
   */
  function flatLlama(size, scarf) {
    return cached("flat|" + size + "|" + scarf.join(","), () => {
      const fw = Math.max(4, trunc(size * 1.6)), fh = Math.max(4, trunc(size * 0.34));
      const [c, g] = canvas(fw, fh);
      const [a1, a2] = nestedAlphas([50, 110]);
      draw.ellipse(g, [24, 20, 26, a1], [trunc(fw * 0.13), 0, trunc(fw * 0.82), fh]);
      draw.ellipse(g, [24, 20, 26, a2], [trunc(fw * 0.17), trunc(fh * 0.12), trunc(fw * 0.74), Math.max(1, trunc(fh * 0.8))]);
      const sq = llamaCanvas(size, "fall", scarf);
      g.drawImage(sq, 0, 0, fw, fh);
      const dark = ui.mix(scarf, [0, 0, 0], 0.35);
      draw.polygon(g, dark, [[fw * 0.57, fh * 0.42], [fw * 0.66, fh * 0.42], [fw * 0.56, fh * 0.98], [fw * 0.47, fh * 0.9]]);
      draw.polygon(g, scarf, [[fw * 0.58, fh * 0.42], [fw * 0.645, fh * 0.42], [fw * 0.555, fh * 0.86], [fw * 0.495, fh * 0.82]]);
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

  const ASPHALT = [[52, 54, 62], [66, 68, 76], [44, 46, 54]]; // oben, Pfad, unten
  const ASPHALT_GRAIN = [[80, 82, 90], [38, 40, 46], [58, 60, 68]];
  const LANE = [230, 230, 222];
  const ZEBRA = [238, 238, 230];
  const ZEBRA_D = [196, 196, 190];
  const CURB = [208, 208, 202], CURB_L = [236, 236, 230], CURB_D = [160, 160, 156];
  const GUTTER = [34, 35, 41];
  const PAVE = [184, 176, 164], PAVE_2 = [172, 164, 152], PAVE_D = [150, 142, 132];
  const HEDGE = [58, 128, 62], HEDGE_D = [38, 92, 46], HEDGE_L = [92, 162, 82];
  const GRASS = [104, 170, 88], GRASS_D = [94, 158, 80];

  /** Ganzzahl-Streuwert 0..2^32-1 (deterministisch, wie _hash in Python). */
  function hash(i) {
    let v = (Math.imul(i, 2654435761 | 0) + 40503) >>> 0;
    v = Math.imul(v ^ (v >>> 15), 2246822519 | 0) >>> 0;
    return (v ^ (v >>> 13)) >>> 0;
  }

  /** Straße von oben: dunkler Asphalt mit leichtem Verlauf und feiner Körnung. */
  function roadBg(g, w, h, pathY) {
    const [top, mid, bot] = ASPHALT;
    const py = Math.max(1, Math.min(h - 1, trunc(pathY)));
    gradient(g, 0, 0, w, py, top, mid);
    gradient(g, 0, py, w, h - py, mid, bot);
    const dot = w >= 1200 ? 2 : 1;
    const n = fdiv(w * h, 650);
    for (let i = 0; i < n; i++) {
      const v = hash(i);
      g.fillStyle = ui.col(ASPHALT_GRAIN[(v >>> 24) % 3]);
      g.fillRect(v % w, Math.floor(v / w) % h, dot, dot);
    }
  }

  function background(world, w, h, pathY) {
    return cached("bg|" + world + "|" + w + "|" + h + "|" + pathY, () => {
      const [c, g] = canvas(w, h);
      if (world === "road") {
        roadBg(g, w, h, pathY);
        return c;
      }
      const [top, bot] = SKY[world];
      gradient(g, 0, 0, w, pathY, top, bot);
      if (world === "bridge") {
        const sun = [trunc(w * 0.78), trunc(pathY * 0.42)];
        for (let i = 5; i > 0; i--) draw.circle(g, [255, 214, 150, 26], sun, trunc(pathY * (0.06 + 0.035 * i)));
        draw.circle(g, [255, 226, 170], sun, trunc(pathY * 0.07));
        gradient(g, 0, pathY, w, h - pathY, [96, 54, 48], [22, 14, 22]);
        for (let i = 0; i < 7; i++) {
          const x = w * (0.07 + 0.145 * i) + Math.sin(i * 2.3) * w * 0.03;
          const top2 = pathY + (h - pathY) * (0.3 + 0.3 * Math.abs(Math.sin(i * 1.7)));
          const bw = w * (0.035 + 0.015 * Math.abs(Math.cos(i * 1.3)));
          draw.polygon(g, [66, 38, 40], [[x - bw, h], [x - bw * 0.45, top2 + bw * 0.4], [x, top2], [x + bw * 0.5, top2 + bw * 0.5], [x + bw, h]]);
          draw.line(g, [112, 66, 58], [x - bw * 0.45, top2 + bw * 0.4], [x, top2], 2);
        }
      } else if (world === "river") {
        gradient(g, 0, pathY - trunc(h * 0.1), w, h - pathY + trunc(h * 0.1), [78, 156, 200], [26, 78, 124]);
      } else {
        gradient(g, 0, pathY - trunc(h * 0.08), w, h - pathY + trunc(h * 0.08), [34, 78, 108], [10, 30, 52]);
      }
      return c;
    });
  }

  /** Kachelbare Ferne (Tafelberge / Wald / Schneegipfel), Breite = w; Straße: leer. */
  function farLayer(world, w, h, pathY) {
    return cached("far|" + world + "|" + w + "|" + h + "|" + pathY, () => {
      const [c, g] = canvas(w, pathY);
      if (!FAR[world]) return c;
      const [c1, c2] = FAR[world];
      const base = pathY - (world === "river" ? trunc(h * 0.1) : world === "ice" ? trunc(h * 0.08) : 0);
      [[c1, 0.42, 1.0], [c2, 0.26, 2.6]].forEach(([col, amp, seed], lay) => {
        let pts = [[0, base]];
        for (let i = 0; i <= w; i += 4) pts.push([i, base - ridge(i, w, pathY * amp, seed)]);
        pts.push([w, base]);
        if (world === "ice" && lay === 0) {
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
  /** Senkrechter Streifen zwischen x=a und x=b (Reihenfolge egal). */
  function band(g, a, b, y0, hgt, col, w) {
    const xa = Math.max(Math.min(a, b), -10), xb = Math.min(Math.max(a, b), w + 10);
    if (xb > xa && hgt > 0) fill(g, col, trunc(xa), trunc(y0), trunc(xb - xa), trunc(hgt));
  }

  /**
   * Gehweg der Straße: Bordstein an lip, Pflaster, Hecke, Rasen nach außen.
   * side=-1: alles links von lip, +1: alles rechts davon. Zur Fahrbahn hin
   * kommen nur Rinne und durchgezogene Randlinie dazu.
   */
  function roadBank(g, lip, side, y, h, k, top, w) {
    top = trunc(top);
    h = trunc(h);
    y = trunc(y);
    const hh = h - top;
    if (hh <= 0) return;
    const sw = trunc(Math.max(44, trunc(118 * k)) * 1.15); // Gehweg inkl. Bordstein
    const cw = Math.max(2, trunc(4 * k)); // Bordstein
    const hw = Math.max(6, trunc(18 * k)); // Hecke
    const ps = Math.max(8, trunc(22 * k)); // Pflasterstein
    const jw = Math.max(1, trunc(1.2 * k)); // Fugen
    const far = side < 0 ? -10 : w + 10;
    const up = Math.max(0, y - top); // Raster hängt am Pfad (y)
    const span = h - Math.min(y, top); // Höhe ab der ersten Rasterzeile
    // Rasen mit Mähstreifen und ein paar Blümchen
    const g0 = lip + side * (sw + hw);
    band(g, g0, far, top, hh, GRASS, w);
    const bw = Math.max(6, trunc(30 * k));
    const fs = Math.max(10, trunc(64 * k));
    let m0 = -(fdiv(up, fs) + 1);
    const rows = fdiv(span, fs) + 3;
    const fr = Math.max(1, trunc(2 * k));
    const near = side < 0 ? w + 10 : -10;
    const j0 = Math.max(0, fdiv(trunc((near - g0) * side), bw) - 1); // Streifen vor dem Bild überspringen
    const jEnd = j0 + fdiv(w + 20, bw) + 4;
    for (let j = j0; j < jEnd; j++) {
      const a = g0 + side * j * bw;
      if ((a - far) * side > 0) break;
      if (j % 2 === 1) band(g, a, a + side * bw, top, hh, GRASS_D, w);
      for (let i = 0; i < rows; i++) {
        if ((j * 5 + i * 3) % 7 !== 0) continue;
        const fx = trunc(a + side * bw * 0.5 + (((j * 13 + i * 7) % 9) - 4) * k);
        const fy = trunc(y + (m0 + i) * fs + (((j * 7 + i * 11) % 11) - 5) * 2 * k);
        if (fy - fr * 2 < top || fy + fr * 2 > h) continue;
        const petal = (j + i) % 2 === 0 ? [250, 250, 240] : [246, 190, 210];
        for (const [dx, dy] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) draw.circle(g, petal, [fx + dx * fr, fy + dy * fr], fr);
        draw.circle(g, [246, 196, 64], [fx, fy], Math.max(1, fr - 1));
      }
    }
    // Hecke: Band + runde Büsche
    band(g, lip + side * sw, g0, top, hh, HEDGE_D, w);
    const step = Math.max(4, trunc(hw * 0.75));
    const rad = hw * 0.56;
    const hx = lip + side * (sw + hw * 0.5);
    m0 = -(fdiv(up, step) + 1);
    const nh = fdiv(span, step) + 3;
    for (let i = 0; i < nh; i++) {
      const cy = y + (m0 + i) * step;
      const cx = hx + (i % 2 === 0 ? hw * 0.08 : -hw * 0.08);
      if (cy - rad < top || cy + rad > h || cx < -hw || cx > w + hw) continue;
      draw.circle(g, HEDGE, [trunc(cx), trunc(cy)], trunc(rad));
      draw.circle(g, HEDGE_L, [trunc(cx - rad * 0.28), trunc(cy - rad * 0.3)], Math.max(1, trunc(rad * 0.4)));
    }
    // Gehweg: versetzte Pflastersteine
    band(g, lip + side * cw, lip + side * sw, top, hh, PAVE, w);
    const nc = Math.max(2, trunc((sw - cw) / ps + 0.5));
    const colw = (sw - cw) / nc;
    m0 = -(fdiv(up, ps) + 2);
    const np = fdiv(span, ps) + 4;
    for (let j = 0; j < nc; j++) {
      const xa = lip + side * (cw + j * colw);
      const xb = lip + side * (cw + (j + 1) * colw);
      const xl = trunc(Math.min(xa, xb)), xr = trunc(Math.max(xa, xb));
      if (xr < -10 || xl > w + 10) continue;
      const off = j % 2 === 1 ? fdiv(ps, 2) : 0;
      for (let i = 0; i < np; i++) {
        const yy = y + (m0 + i) * ps + off;
        if (yy >= h) break;
        if (yy + ps <= top) continue;
        if ((j * 5 + i * 3) % 7 === 0) {
          const y0 = Math.max(top, yy);
          fill(g, PAVE_2, xl, y0, xr - xl, Math.min(h, yy + ps) - y0);
        }
        if (yy >= top) fill(g, PAVE_D, xl, yy, xr - xl, jw);
      }
      if (j > 0) band(g, xa, xa + side * jw, top, hh, PAVE_D, w);
    }
    // Bordstein mit Fugen, Rinne und Randlinie auf der Fahrbahn
    band(g, lip, lip + side * cw, top, hh, CURB, w);
    band(g, lip, lip + side * Math.max(1, fdiv(cw, 3)), top, hh, CURB_L, w);
    m0 = -(fdiv(up, ps * 3) + 1);
    const nk = fdiv(span, ps * 3) + 3;
    for (let i = 0; i < nk; i++) {
      const yy = y + (m0 + i) * ps * 3;
      if (top <= yy && yy < h) band(g, lip, lip + side * cw, yy, 1, CURB_D, w);
    }
    band(g, lip, lip - side * Math.max(1, trunc(2 * k)), top, hh, GUTTER, w);
    const ex = lip - side * trunc(9 * k);
    band(g, ex, ex - side * Math.max(2, trunc(3 * k)), top, hh, LANE, w);
  }

  /**
   * Ufer: side=-1 links (Fels/Wiese bis xEdge), +1 rechts (ab xEdge); w = Bildbreite.
   * top: Oberkante (nur Straße; die übrigen Welten beginnen kurz über y).
   */
  function drawBank(g, world, xEdge, side, y, h, k, w, top = 0) {
    if (world === "road") {
      // Rinne und Randlinie reichen bis ~12k auf die Fahrbahn
      const m = trunc(14 * k) + 4;
      if ((side < 0 && xEdge < -m) || (side >= 0 && xEdge > w + m)) return;
      roadBank(g, trunc(xEdge), side < 0 ? -1 : 1, y, h, k, top, w);
      return;
    }
    let x0, x1;
    if (side < 0) {
      if (xEdge < 0) return;
      x0 = -10;
      x1 = trunc(xEdge);
    } else {
      if (xEdge > w) return;
      x0 = trunc(xEdge);
      x1 = w + 10;
    }
    const lip = trunc(xEdge);
    if (world === "bridge") {
      const pts = side < 0
        ? [[x0, y], [lip, y], [lip - trunc(18 * k), h], [x0, h]]
        : [[lip, y], [x1, y], [x1, h], [lip + trunc(18 * k), h]];
      draw.polygon(g, [150, 92, 66], pts);
      for (let i = 1; i < 6; i++) {
        const yy = y + trunc(((h - y) * i) / 6), off = trunc((18 * k * i) / 6);
        const a = side < 0 ? [x0, yy] : [lip + off, yy];
        const b = side < 0 ? [lip - off, yy] : [x1, yy];
        draw.line(g, [104, 62, 48], a, b, Math.max(1, trunc(2 * k)));
      }
      draw.rect(g, [112, 168, 84], [x0, y - trunc(6 * k), x1 - x0, trunc(8 * k)]);
    } else if (world === "river") {
      const sand = trunc(22 * k);
      draw.rect(g, [88, 156, 74], [x0, y - trunc(8 * k), x1 - x0, h]);
      for (let i = 1; i < 5; i++) {
        const yy = y + trunc(((h - y) * i) / 5);
        draw.line(g, [78, 142, 66], [x0, yy], [x1, yy], Math.max(1, trunc(2 * k)));
      }
      const sandPts = side < 0
        ? [[lip - sand, y - trunc(8 * k)], [lip, y - trunc(4 * k)], [lip + trunc(6 * k), h], [lip - sand, h]]
        : [[lip + sand, y - trunc(8 * k)], [lip, y - trunc(4 * k)], [lip - trunc(6 * k), h], [lip + sand, h]];
      draw.polygon(g, [222, 200, 146], sandPts);
      draw.line(g, [250, 252, 255], [lip, y - trunc(4 * k)], [lip + side * -trunc(6 * k), h], Math.max(1, trunc(3 * k)));
      draw.rect(g, [110, 184, 92], [x0, y - trunc(10 * k), x1 - x0, trunc(4 * k)]);
    } else {
      draw.rect(g, [236, 244, 250], [x0, y - trunc(4 * k), x1 - x0, h]);
      draw.rect(g, [196, 214, 232], [x0, y + trunc(10 * k), x1 - x0, trunc(4 * k)]);
      draw.line(g, [255, 255, 255], [x0, y - trunc(4 * k)], [x1, y - trunc(4 * k)], Math.max(1, trunc(2 * k)));
    }
  }

  function drawRopes(g, points, k, w) {
    const rope = [120, 82, 48];
    const lw = Math.max(1, trunc(2 * k));
    const rail = trunc(46 * k);
    for (let i = 0; i < points.length - 1; i++) {
      const [xa, ya] = points[i], [xb, yb] = points[i + 1];
      if (xb < -40 || xa > w + 40) continue;
      const mx = (xa + xb) / 2, my = (ya + yb) / 2 + 5 * k;
      draw.lines(g, rope, false, [[xa, ya], [mx, my], [xb, yb]], lw);
      draw.lines(g, rope, false, [[xa, ya - rail], [mx, my - rail + 2 * k], [xb, yb - rail]], lw);
      draw.line(g, rope, [xb, yb], [xb, yb - rail], Math.max(1, lw - 1));
    }
  }

  /** Gestrichelte Spur-Trennlinien der Straße (statisch, t ungenutzt); xs = Bild-x der Linien. */
  function drawLanes(g, xs, top, bottom, k, t, w) {
    const lw = Math.max(2, trunc(3 * k));
    const dash = Math.max(6, trunc(26 * k));
    const gap = Math.max(4, trunc(18 * k));
    top = trunc(top);
    bottom = trunc(bottom);
    g.fillStyle = ui.col(LANE);
    for (const x of xs) {
      if (x < -lw || x > w + lw) continue;
      const xi = trunc(x) - fdiv(lw, 2);
      for (let yy = top; yy < bottom; yy += dash + gap) g.fillRect(xi, yy, lw, Math.min(dash, bottom - yy));
    }
  }

  // ============================================================ Kacheln
  /**
   * Ein Schritt: Planke / Stein / Scholle, Oberkante bei y, Mitte cx (alpha 0..255).
   * Straße: Zebrastreifen-Stück von y - tw*0.38 bis y + tw*0.17 (Füße bei y).
   */
  function drawTile(g, world, cx, y, tw, k, t, idx, alpha = 255) {
    if (alpha <= 0) return;
    if (alpha < 255) {
      const oy = world === "road" ? trunc(tw * 0.45) : trunc(tw * 0.2);
      const lw = trunc(tw * 1.4) + 8, lh = trunc(tw * 0.9) + 8;
      const l = layer(lw, lh);
      drawTile(l.g, world, lw / 2, oy, tw, k, t, idx);
      blitLayer(g, l, trunc(cx - lw / 2), trunc(y - oy), alpha);
      return;
    }
    if (world === "road") {
      // Zebrastreifen: weiße Balken parallel zum Verkehr (senkrecht)
      const nb = 4;
      const u = tw / (2 * nb - 1);
      const zh = Math.max(4, trunc(tw * 0.55));
      const zy = trunc(y - tw * 0.38);
      const bw = Math.max(2, trunc(u));
      const rad = Math.max(1, trunc(u * 0.25));
      for (let j = 0; j < nb; j++) {
        const bx = trunc(cx - tw / 2 + j * 2 * u);
        draw.rect(g, ZEBRA_D, [bx, zy, bw, zh], 0, rad);
        draw.rect(g, ZEBRA, [bx, zy, bw, zh - Math.max(1, trunc(2 * k))], 0, rad);
      }
      return;
    }
    if (world === "bridge") {
      const th = Math.max(5, trunc(13 * k));
      const rw = trunc(tw);
      const r = new PG.Rect(trunc(cx) - fdiv(rw, 2), trunc(y), rw, th);
      const rad = Math.max(1, trunc(3 * k));
      draw.rect(g, [96, 60, 34], [r.x, r.y + Math.max(1, trunc(3 * k)), r.w, r.h], 0, rad);
      draw.rect(g, [168, 112, 66], r, 0, rad);
      for (const j of [0.3, 0.68]) {
        const yy = r.y + trunc(r.h * j);
        draw.line(g, [132, 84, 48], [r.x + 4, yy], [r.right - 4 - trunc(6 * j * k), yy], 1);
      }
      for (const xx of [r.x + trunc(5 * k), r.right - trunc(5 * k)]) {
        draw.circle(g, [70, 58, 54], [xx, r.y + fdiv(r.h, 2)], Math.max(1, trunc(1.6 * k)));
      }
    } else if (world === "river") {
      const bob = Math.sin(t * 2.2 + idx * 1.3) * 1.5 * k;
      const h = trunc(tw * 0.4);
      const rw = trunc(tw);
      const r = [trunc(cx) - fdiv(rw, 2), trunc(y - h * 0.22 + bob), rw, h];
      const gw = trunc(tw * 1.25), gh = trunc(h * 0.62);
      const ring = [trunc(cx) - fdiv(gw, 2), trunc(y + h * 0.42 + bob) - fdiv(gh, 2), gw, gh];
      draw.ellipse(g, [170, 214, 236], ring, Math.max(1, trunc(2 * k)));
      draw.ellipse(g, [82, 88, 96], [r[0], r[1] + trunc(4 * k), r[2], r[3]]);
      draw.ellipse(g, [128, 134, 142], r);
      const top = inflate(r, -trunc(tw * 0.28), -trunc(h * 0.45));
      top[0] -= trunc(tw * 0.05);
      top[1] -= trunc(h * 0.12);
      draw.ellipse(g, [170, 176, 184], top);
      draw.ellipse(g, [96, 140, 92], [r[0] + trunc(tw * 0.62), r[1] + trunc(h * 0.1), trunc(tw * 0.16), trunc(h * 0.2)]);
    } else {
      const h = trunc(tw * 0.34);
      const side = Math.max(3, trunc(7 * k));
      const pts = [];
      for (let j = 0; j < 7; j++) {
        const a = (TAU * j) / 7 + idx * 0.7;
        const rad = 0.5 + 0.06 * Math.sin(idx * 2.1 + j * 1.7);
        pts.push([cx + Math.cos(a) * tw * 0.8 * rad * 1.1, y + h * 0.5 + Math.sin(a) * h * rad]);
      }
      draw.polygon(g, [24, 52, 76], pts.map(([x, yy]) => [x, yy + side + 3 * k]));
      draw.polygon(g, [164, 202, 226], pts.map(([x, yy]) => [x, yy + side]));
      draw.polygon(g, [228, 242, 250], pts);
      draw.line(g, [255, 255, 255], [cx - tw * 0.22, y + h * 0.28], [cx - tw * 0.02, y + h * 0.18], Math.max(1, trunc(2 * k)));
    }
  }

  /**
   * Kachel als eigene Fläche wie das "layer" in _draw_broken_tile:
   * Größe int(tw*1.4)+8 x int(tw*0.9)+8, Kachel bei (Breite/2, int(tw*0.2)). Gecacht.
   */
  function tileCanvas(world, tw, k, t, idx) {
    const build = () => {
      const lw = trunc(tw * 1.4) + 8, lh = trunc(tw * 0.9) + 8;
      const [c, g] = canvas(lw, lh);
      drawTile(g, world, lw / 2, trunc(tw * 0.2), tw, k, t, idx);
      return c;
    };
    if (world === "river") return build(); // wippt mit der Zeit
    return cached("tile|" + world + "|" + tw + "|" + k + "|" + idx, build);
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

  // ============================================================ Hindernisse
  const CAR_COLS = [[216, 66, 58], [60, 124, 214], [242, 190, 50]];
  const TRUCK_COL = [36, 150, 140];
  const GLASS = [56, 72, 98];
  const GLASS_L = [150, 186, 214];
  const TIRE = [32, 32, 38];
  const HEAD = [255, 248, 206];
  const TAIL = [196, 26, 40];
  const SHADOW_A = nestedAlphas([28, 52, 78]);

  /** Senkrechte Länge eines Hindernisses in Pixeln (für Treffer-Test/Halten). */
  function obstacleLen(world, kind, size) {
    if (world === "road") return trunc(size * (kind === 3 ? 1.45 : 0.86));
    if (world === "bridge") return trunc(size * 0.42);
    if (world === "river") return trunc(size * 0.3);
    return trunc(size * 0.48);
  }

  /** Weicher Schatten nach rechts unten (3 Stufen). */
  function carShadow(b, x0, y0, w, ln, r) {
    const sh = Math.max(2, trunc(w * 0.08));
    [[2, 0], [1, 1], [0, 2]].forEach(([i, ai]) => {
      const g = sh * i * 0.5;
      rrect(b, [0, 0, 0, SHADOW_A[ai]], x0 + sh - g, y0 + sh * 1.4 - g, w + 2 * g, ln + 2 * g, r + g);
    });
  }

  /** Auto von oben, Front unten (fährt nach unten); w/ln in Feld-Pixeln. */
  function drawCar(b, cx, cy, w, ln, kind) {
    const col = CAR_COLS[kind % 3];
    const dark = ui.mix(col, [0, 0, 0], 0.3);
    const light = ui.mix(col, [255, 255, 255], 0.32);
    const roof = ui.mix(col, [255, 255, 255], 0.14);
    const x0 = cx - w / 2, y0 = cy - ln / 2;
    const v = (u) => y0 + ln * u;
    carShadow(b, x0, y0, w, ln, w * 0.3);
    // Räder (schauen seitlich etwas heraus)
    const rw = w * 0.14, rh = ln * 0.17;
    for (const u of [0.19, 0.79]) {
      for (const sx of [-1, 1]) rrect(b, TIRE, cx + sx * (w / 2 - rw * 0.3) - rw / 2, v(u) - rh / 2, rw, rh, rw * 0.35);
    }
    // Karosserie: dunkle Flanken, helle Oberseite
    rrect(b, dark, x0, y0, w, ln, w * 0.3);
    rrect(b, col, x0 + w * 0.07, y0 + ln * 0.012, w * 0.86, ln * 0.976, w * 0.26);
    // Glanz auf Motorhaube und Kofferraum
    rrect(b, light, x0 + w * 0.15, v(0.76), w * 0.07, ln * 0.17, w * 0.035);
    rrect(b, light, x0 + w * 0.15, v(0.05), w * 0.07, ln * 0.1, w * 0.035);
    // Heckscheibe, Dach, Seitenfenster, Frontscheibe
    draw.polygon(b, GLASS, [[cx - w * 0.27, v(0.19)], [cx + w * 0.27, v(0.19)], [cx + w * 0.34, v(0.295)], [cx - w * 0.34, v(0.295)]]);
    for (const sx of [-1, 1]) {
      draw.polygon(b, GLASS, [[cx + sx * w * 0.345, v(0.3)], [cx + sx * w * 0.405, v(0.32)], [cx + sx * w * 0.405, v(0.57)], [cx + sx * w * 0.345, v(0.585)]]);
    }
    rrect(b, roof, cx - w * 0.34, v(0.285), w * 0.68, ln * 0.31, w * 0.1);
    draw.polygon(b, GLASS, [[cx - w * 0.34, v(0.585)], [cx + w * 0.34, v(0.585)], [cx + w * 0.4, v(0.715)], [cx - w * 0.4, v(0.715)]]);
    const lw = Math.max(1, trunc(w * 0.05));
    draw.line(b, GLASS_L, [cx - w * 0.24, v(0.7)], [cx - w * 0.06, v(0.6)], lw);
    draw.line(b, GLASS_L, [cx - w * 0.04, v(0.7)], [cx + w * 0.04, v(0.645)], lw);
    draw.line(b, GLASS_L, [cx + w * 0.1, v(0.2)], [cx + w * 0.2, v(0.28)], lw);
    if (kind % 3 === 0) {
      // Rennstreifen über Motorhaube, Dach und Kofferraum
      for (const [u0, u1] of [[0.015, 0.185], [0.3, 0.575], [0.725, 0.985]]) {
        for (const sx of [-1, 1]) rrect(b, [248, 246, 240], cx + sx * w * 0.075 - w * 0.035, v(u0), w * 0.07, ln * (u1 - u0), 0);
      }
    } else if (kind % 3 === 1) {
      // Schiebedach
      rrect(b, GLASS, cx - w * 0.17, v(0.35), w * 0.34, ln * 0.13, w * 0.06);
      draw.line(b, GLASS_L, [cx - w * 0.09, v(0.465)], [cx + w * 0.02, v(0.37)], lw);
    } else {
      // Dachträger
      for (const u of [0.36, 0.51]) rrect(b, [66, 66, 74], cx - w * 0.31, v(u), w * 0.62, ln * 0.025, w * 0.02);
      for (const sx of [-1, 1]) rrect(b, [66, 66, 74], cx + sx * w * 0.24 - w * 0.025, v(0.34), w * 0.05, ln * 0.2, 0);
    }
    // Außenspiegel
    for (const sx of [-1, 1]) {
      draw.ellipse(b, dark, [trunc(cx + sx * w * 0.5 - w * 0.07), trunc(v(0.585)), trunc(w * 0.14), Math.max(2, trunc(ln * 0.045))]);
    }
    // Scheinwerfer vorn (unten), Rückleuchten hinten (oben)
    for (const sx of [-1, 1]) {
      rrect(b, HEAD, cx + sx * w * 0.27 - w * 0.1, v(0.962), w * 0.2, ln * 0.026, w * 0.04);
      rrect(b, TAIL, cx + sx * w * 0.28 - w * 0.08, v(0.01), w * 0.16, ln * 0.024, w * 0.03);
    }
  }

  /** Lieferwagen von oben: Kofferaufbau hinten, Fahrerhaus vorn (unten). */
  function drawTruck(b, cx, cy, w, ln) {
    const col = TRUCK_COL;
    const dark = ui.mix(col, [0, 0, 0], 0.3);
    const roof = ui.mix(col, [255, 255, 255], 0.16);
    const box = [238, 236, 228], boxD = [190, 188, 180], boxR = [216, 214, 206];
    const x0 = cx - w / 2, y0 = cy - ln / 2;
    const v = (u) => y0 + ln * u;
    carShadow(b, x0, y0, w, ln, w * 0.12);
    const rw = w * 0.13, rh = ln * 0.09;
    for (const u of [0.12, 0.23, 0.86]) {
      for (const sx of [-1, 1]) rrect(b, TIRE, cx + sx * (w / 2 - rw * 0.3) - rw / 2, v(u) - rh / 2, rw, rh, rw * 0.35);
    }
    // Kofferaufbau mit Rippen und Zierstreifen
    rrect(b, boxD, x0, y0, w, ln * 0.71, w * 0.08);
    rrect(b, box, x0 + w * 0.05, y0 + ln * 0.006, w * 0.9, ln * 0.7, w * 0.06);
    const lw = Math.max(1, trunc(w * 0.025));
    for (let i = 1; i < 11; i++) {
      const yy = v(0.065 * i);
      draw.line(b, boxR, [x0 + w * 0.1, yy], [x0 + w * 0.9, yy], lw);
    }
    for (const sx of [-1, 1]) rrect(b, col, cx + sx * w * 0.36 - w * 0.04, v(0.03), w * 0.08, ln * 0.65, 0);
    draw.line(b, boxD, [cx, v(0.006)], [cx, v(0.05)], lw);
    for (const sx of [-1, 1]) rrect(b, TAIL, cx + sx * w * 0.36 - w * 0.07, v(0.004), w * 0.14, ln * 0.016, w * 0.02);
    // Kupplung + Fahrerhaus
    rrect(b, [54, 54, 62], cx - w * 0.24, v(0.7), w * 0.48, ln * 0.05, 0);
    rrect(b, dark, x0 + w * 0.03, v(0.735), w * 0.94, ln * 0.265, w * 0.24);
    rrect(b, col, x0 + w * 0.09, v(0.742), w * 0.82, ln * 0.252, w * 0.2);
    rrect(b, roof, cx - w * 0.33, v(0.752), w * 0.66, ln * 0.12, w * 0.08);
    for (const sx of [-1, 1]) draw.circle(b, [255, 184, 56], [trunc(cx + sx * w * 0.16), trunc(v(0.77))], Math.max(1, trunc(w * 0.04)));
    draw.polygon(b, GLASS, [[cx - w * 0.33, v(0.876)], [cx + w * 0.33, v(0.876)], [cx + w * 0.38, v(0.935)], [cx - w * 0.38, v(0.935)]]);
    draw.line(b, GLASS_L, [cx - w * 0.22, v(0.93)], [cx - w * 0.1, v(0.88)], Math.max(1, trunc(w * 0.04)));
    for (const sx of [-1, 1]) {
      draw.ellipse(b, dark, [trunc(cx + sx * w * 0.52 - w * 0.08), trunc(v(0.86)), trunc(w * 0.16), Math.max(2, trunc(ln * 0.025))]);
      rrect(b, HEAD, cx + sx * w * 0.28 - w * 0.09, v(0.976), w * 0.18, ln * 0.016, w * 0.03);
    }
  }

  /** Auto/Lieferwagen als gecachte Fläche (Geometrie in 2-facher Größe wie in Python). */
  function carSprite(size, kind) {
    return cached("car|" + size + "|" + kind, () => {
      const truck = kind === 3;
      const bw = Math.max(4, trunc(size * (truck ? 0.58 : 0.52)));
      const ln = Math.max(4, obstacleLen("road", kind, size));
      const m = Math.max(3, trunc(size * 0.1));
      const fw = bw + 2 * m, fh = ln + 2 * m;
      const [c, g] = canvas(fw, fh);
      g.scale(0.5, 0.5);
      if (truck) drawTruck(g, fw, fh, bw * 2, ln * 2);
      else drawCar(g, fw, fh, bw * 2, ln * 2, kind);
      return c;
    });
  }

  /** Fallender Felsbrocken (Brücke), Durchmesser = obstacleLen. */
  function boulder(s, x, y, size, kind) {
    const r = obstacleLen("bridge", kind, size) / 2;
    const ring = (sc, dx, dy) => {
      const pts = [];
      for (let j = 0; j < 9; j++) {
        const a = (TAU * j) / 9 + kind * 0.6;
        const rad = r * sc * (0.86 + 0.14 * Math.sin(j * 2.4 + kind * 1.9));
        pts.push([x + dx + Math.cos(a) * rad * 1.08, y + dy + Math.sin(a) * rad]);
      }
      return pts;
    };
    draw.polygon(s, [74, 66, 68], ring(1.0, 0, 0));
    draw.polygon(s, [132, 124, 120], ring(0.84, -r * 0.08, -r * 0.1));
    draw.polygon(s, [168, 160, 152], ring(0.42, -r * 0.28, -r * 0.32));
    const lw = Math.max(1, trunc(r * 0.09));
    draw.lines(s, [90, 82, 82], false, [[x + r * 0.12, y - r * 0.18], [x + r * 0.3, y + r * 0.08], [x + r * 0.2, y + r * 0.4]], lw);
    draw.line(s, [90, 82, 82], [x - r * 0.42, y + r * 0.22], [x - r * 0.12, y + r * 0.34], lw);
  }

  /** Treibholz (Fluss): liegender Stamm, Stirnseite rechts, Gischt unten. */
  function log(s, x, y, size, kind) {
    const lw = Math.max(6, trunc(size * 0.8));
    const lh = Math.max(3, obstacleLen("river", kind, size));
    const x0 = trunc(x - lw / 2), y0 = trunc(y - lh / 2);
    const bark = [128, 84, 50], barkD = [88, 56, 36], barkL = [160, 112, 70];
    const wood = [228, 188, 130], ringC = [180, 132, 84];
    // Stamm (unten dunkler = rund)
    draw.rect(s, barkD, [x0, y0, lw, lh], 0, fdiv(lh, 2));
    draw.rect(s, bark, [x0, y0, lw, Math.max(2, trunc(lh * 0.7))], 0, fdiv(lh, 2));
    const t = Math.max(1, trunc(lh * 0.1));
    draw.line(s, barkL, [x0 + lh * 0.5, y0 + lh * 0.22], [x0 + lw - lh * 0.9, y0 + lh * 0.22], t);
    // Rindenrillen
    for (let j = 0; j < 3; j++) {
      const yy = y0 + lh * (0.4 + 0.18 * j);
      const a = x0 + lh * (0.45 + 0.9 * ((j * 2 + kind) % 3));
      const b = a + lw * (0.3 + 0.06 * ((kind + j) % 2));
      draw.line(s, barkD, [a, yy], [Math.min(b, x0 + lw - lh * 0.8), yy], t);
    }
    // Aststummel oben
    const sx = x0 + lw * (0.3 + 0.12 * (kind % 3));
    draw.ellipse(s, barkD, [trunc(sx - lh * 0.16), trunc(y0 - lh * 0.12), Math.max(2, trunc(lh * 0.32)), Math.max(2, trunc(lh * 0.3))]);
    draw.ellipse(s, wood, [trunc(sx - lh * 0.09), trunc(y0 - lh * 0.07), Math.max(1, trunc(lh * 0.18)), Math.max(1, trunc(lh * 0.16))]);
    // Stirnseite mit Jahresringen
    const ew = Math.max(2, trunc(lh * 0.62));
    const er = [x0 + lw - ew, y0, ew, lh];
    draw.ellipse(s, barkD, er);
    draw.ellipse(s, wood, inflate(er, -Math.max(2, trunc(lh * 0.16)), -Math.max(2, trunc(lh * 0.16))));
    draw.ellipse(s, ringC, inflate(er, -trunc(ew * 0.45), -trunc(lh * 0.45)), Math.max(1, trunc(lh * 0.06)));
    draw.circle(s, ringC, [er[0] + fdiv(ew, 2), er[1] + fdiv(lh, 2)], Math.max(1, trunc(lh * 0.06)));
    // kleine Gischt an der Wasserlinie (flache Schaumkronen + Bugwelle)
    const foam = [240, 248, 252];
    for (let j = 0; j < 5; j++) {
      const fw = lw * (0.16 + 0.04 * ((j + kind) % 2));
      const fx = x0 + lw * (0.02 + 0.2 * j);
      const fh = Math.max(2, trunc(lh * 0.22));
      const fy = y0 + lh * (0.9 + 0.05 * Math.sin(j * 1.7 + kind));
      draw.ellipse(s, foam, [trunc(fx), trunc(fy), Math.max(2, trunc(fw)), fh]);
    }
    for (const sxx of [-1, 1]) {
      const ex = sxx < 0 ? x0 - lh * 0.1 : x0 + lw - lh * 0.12;
      draw.ellipse(s, foam, [trunc(ex), trunc(y0 + lh * 0.7), Math.max(2, trunc(lh * 0.26)), Math.max(2, trunc(lh * 0.34))]);
    }
  }

  /** Flächen, Kanten und Glanz des Eisblocks. */
  function iceFaces(s, x0, y0, x1, y1, th, sd, bw, bh, kind) {
    const edge = [92, 150, 194];
    const front = [[x0, y0 + th], [x1 - sd, y0 + th], [x1 - sd, y1], [x0, y1]];
    const top = [[x0, y0 + th], [x0 + sd, y0], [x1, y0], [x1 - sd, y0 + th]];
    const side = [[x1 - sd, y0 + th], [x1, y0], [x1, y1 - th], [x1 - sd, y1]];
    draw.polygon(s, [178, 222, 244], front);
    draw.polygon(s, [128, 182, 220], side);
    draw.polygon(s, [240, 250, 255], top);
    const lw = Math.max(1, trunc(bw * 0.025));
    for (const pts of [front, top, side]) draw.polygon(s, edge, pts, lw);
    // Glanz + Luftbläschen + Riss
    const fw = bw - sd;
    draw.line(s, [255, 255, 255], [x0 + fw * 0.14, y1 - (bh - th) * 0.25], [x0 + fw * 0.38, y0 + th + (bh - th) * 0.22], Math.max(1, trunc(bw * 0.05)));
    draw.line(s, [255, 255, 255], [x0 + fw * 0.34, y1 - (bh - th) * 0.2], [x0 + fw * 0.46, y0 + th + (bh - th) * 0.45], Math.max(1, trunc(bw * 0.03)));
    for (let j = 0; j < 2; j++) {
      draw.circle(s, [220, 242, 252], [trunc(x0 + fw * (0.62 + 0.16 * j)), trunc(y0 + th + (bh - th) * (0.35 + 0.25 * j))], Math.max(1, trunc(bw * 0.03)));
    }
    if (kind % 2 === 1) {
      draw.lines(s, edge, false, [[x0 + fw * 0.7, y0 + th], [x0 + fw * 0.62, y0 + th + (bh - th) * 0.4], [x0 + fw * 0.72, y1 - (bh - th) * 0.15]], lw);
    }
    draw.line(s, [255, 255, 255], [x0 + sd * 0.6, y0 + th * 0.5], [x0 + sd + bw * 0.25, y0 + th * 0.5], Math.max(1, trunc(bw * 0.03)));
  }

  /** Rutschender Eisblock (Eissee): Quader mit Deck- und Seitenfläche. */
  function iceBlock(s, x, y, size, kind) {
    const bh = Math.max(4, obstacleLen("ice", kind, size));
    const bw = Math.max(4, trunc(size * 0.62));
    const x0 = trunc(x - bw / 2), y0 = trunc(y - bh / 2);
    const x1 = x0 + bw, y1 = y0 + bh;
    const th = trunc(bh * (0.36 + 0.04 * (kind % 2))); // Höhe der Deckfläche
    const sd = Math.max(2, trunc(bw * 0.16)); // Versatz der Seitenfläche
    const snow = [236, 246, 252];
    const sp = Math.max(2, trunc(bh * 0.16));
    // Schneestaub seitlich hinter dem Block
    for (const [fx, fy] of [[-0.12, -0.2], [1.0, -0.4]]) {
      const rw = Math.max(2, trunc(sp * 1.6)), rh = Math.max(1, trunc(sp * 0.9));
      draw.ellipse(s, snow, [trunc(x0 + bw * fx - rw / 2), trunc(y1 + bh * fy - rh / 2), rw, rh]);
    }
    iceFaces(s, x0, y0, x1, y1, th, sd, bw, bh, kind);
    // vorn aufgeschobener Schneewulst + ein paar Flocken
    draw.ellipse(s, snow, [x0 - trunc(bw * 0.06), y1 - trunc(sp * 0.7), bw - sd + trunc(bw * 0.12), Math.max(2, trunc(sp * 1.1))]);
    [[-0.17, -0.04], [-0.1, -0.34], [1.06, -0.18]].forEach(([fx, fy], j) => {
      draw.circle(s, snow, [trunc(x0 + bw * fx), trunc(y1 + bh * fy)], Math.max(1, trunc(sp * (0.3 + 0.1 * ((j + kind) % 2)))));
    });
  }

  /**
   * Hindernis mittig bei (x, y); size = fertige Pixelgröße, kind 0..3, alpha 0..255.
   * Straße: Auto (0-2 rot/blau/gelb) bzw. Lieferwagen (3) von oben, Front unten;
   * Brücke: Felsbrocken; Fluss: Treibholz; Eissee: Eisblock.
   */
  function drawObstacle(s, world, x, y, size, kind, alpha = 255) {
    size = trunc(size);
    if (alpha <= 0 || size <= 0) return;
    kind = kind % 4;
    if (world === "road") {
      const img = carSprite(size, kind);
      const dx = trunc(x) - fdiv(img.lw, 2), dy = trunc(y) - fdiv(img.lh, 2);
      if (alpha < 255) {
        s.save();
        s.globalAlpha *= alpha / 255;
        s.drawImage(img, dx, dy, img.lw, img.lh);
        s.restore();
      } else {
        s.drawImage(img, dx, dy, img.lw, img.lh);
      }
      return;
    }
    if (alpha < 255) {
      const lw = size + 8, lh = trunc(size * 0.8) + 8;
      const l = layer(lw, lh);
      drawObstacle(l.g, world, fdiv(lw, 2), fdiv(lh, 2), size, kind);
      blitLayer(s, l, trunc(x) - fdiv(lw, 2), trunc(y) - fdiv(lh, 2), alpha);
      return;
    }
    if (world === "bridge") boulder(s, trunc(x), trunc(y), size, kind);
    else if (world === "river") log(s, trunc(x), trunc(y), size, kind);
    else iceBlock(s, trunc(x), trunc(y), size, kind);
  }

  /** Weicher Lichthof für das Warnlicht (3 Ringe). */
  function glowCanvas(r) {
    return cached("glow|" + r, () => {
      const [c, g] = canvas(r * 6, r * 6);
      const al = nestedAlphas([40, 70, 100]);
      [[3, 0], [2, 1], [1, 2]].forEach(([i, ai]) => {
        draw.circle(g, [255, 190, 70, al[ai]], [r * 3, r * 3], trunc(r * (1 + i * 0.6)));
      });
      return c;
    });
  }

  /** Rot-weiße Absperrung (Balken der Breite w mittig bei x, Höhe ~8k bei y). */
  function drawBarrier(s, x, y, w, k) {
    w = Math.max(8, trunc(w));
    const bh = Math.max(4, trunc(8 * k));
    const pw = Math.max(2, trunc(4 * k));
    const ph = Math.max(4, trunc(12 * k));
    const x0 = trunc(x) - fdiv(w, 2), y0 = trunc(y) - fdiv(bh, 2);
    const post = [84, 86, 96], postD = [48, 50, 58];
    // Pfosten mit Füßen
    const foot = Math.max(2, trunc(3 * k));
    for (const px of [x0 + pw, x0 + w - 2 * pw]) {
      fill(s, post, px, y0, pw, fdiv(bh, 2) + ph);
      fill(s, postD, px - pw, y0 + fdiv(bh, 2) + ph - foot, pw * 3, foot);
    }
    const bar = cached("barrier|" + w + "|" + bh, () => {
      const [c, g] = canvas(w, bh);
      fill(g, [244, 244, 238], 0, 0, w, bh);
      const st = Math.max(3, trunc(bh * 1.1));
      for (let i = -1; i < fdiv(w, st) + 2; i++) {
        if (i % 2 === 0) continue;
        const sx = i * st;
        draw.polygon(g, [214, 44, 46], [[sx, 0], [sx + st, 0], [sx + st - bh, bh], [sx - bh, bh]]);
      }
      draw.rect(g, [120, 28, 34], [0, 0, w, bh], 1);
      return c;
    });
    s.drawImage(bar, x0, y0, bar.lw, bar.lh);
    // kleines Warnlicht links oben
    const r = Math.max(2, trunc(4 * k));
    const lx = x0 + pw + fdiv(pw, 2), ly = y0 - r;
    const glow = glowCanvas(r);
    s.drawImage(glow, lx - fdiv(glow.lw, 2), ly - fdiv(glow.lh, 2), glow.lw, glow.lh);
    fill(s, postD, lx - fdiv(r, 2), ly, Math.max(1, r), r + 1);
    draw.circle(s, [255, 168, 40], [lx, ly], r);
    draw.circle(s, [255, 236, 170], [lx - fdiv(r, 3), ly - fdiv(r, 3)], Math.max(1, fdiv(r, 2)));
  }

  /** Zwei dunkle, halbdurchsichtige Reifenspuren mittig bei (x, y). */
  function drawTireMarks(s, x, y, w, k) {
    const sw = Math.max(2, trunc(5 * k));
    const ln = Math.max(4, trunc(w * 1.2));
    const gp = Math.max(sw + 1, trunc(w * 0.32));
    const tread = Math.max(2, trunc(3 * k));
    const lay = cached("tire|" + ln + "|" + gp + "|" + sw + "|" + tread, () => {
      const [c, g] = canvas(gp + sw, ln);
      const half = ln / 2;
      for (let yy = 0; yy < ln; yy++) {
        const f = 1.0 - Math.abs(yy + 0.5 - half) / half; // 0 an den Enden, 1 Mitte
        let a = trunc(30 + 110 * Math.min(1.0, f * 2.5));
        if (fdiv(yy, tread) % 2 === 1) a = fdiv(a * 6, 10); // Profilkerben
        for (const xx of [0, gp]) fill(g, [22, 20, 26, a], xx, yy, sw, 1);
      }
      return c;
    });
    s.drawImage(lay, trunc(x) - fdiv(lay.lw, 2), trunc(y) - fdiv(ln, 2), lay.lw, lay.lh);
  }

  /** Piktogramm der Welt (für die Welt-Knöpfe). */
  function worldIcon(g, world, r, col) {
    const cx = r.centerx, cy = r.centery, u = r.w / 10;
    const lw = Math.max(1, trunc(u * 0.8));
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
    } else if (world === "road") {
      // Straße in die Ferne: zulaufende Ränder, kürzer werdende Mittelstriche
      for (const sx of [-1, 1]) draw.line(g, col, [cx + sx * u * 4.4, r.bottom - u * 0.6], [cx + sx * u * 1.2, r.y + u * 1.0], lw);
      draw.line(g, col, [cx, r.bottom - u * 0.6], [cx, r.bottom - u * 2.8], lw + 1);
      draw.line(g, col, [cx, cy + u * 0.6], [cx, cy - u * 0.9], lw);
      draw.line(g, col, [cx, r.y + u * 2.6], [cx, r.y + u * 1.6], Math.max(1, lw - 1));
    } else {
      draw.polygon(g, col, [[r.x + u, cy], [cx - u * 1.5, cy - u * 2.0], [cx + u * 2.5, cy - u * 1.6], [r.right - u, cy + u * 0.4],
        [cx + u * 0.5, cy + u * 1.8], [cx - u * 2.6, cy + u * 1.4]], lw);
      draw.lines(g, col, false, [[cx - u * 0.5, cy - u * 1.6], [cx + u * 0.2, cy - u * 0.2], [cx - u * 0.6, cy + u * 0.6], [cx + u * 0.4, cy + u * 1.6]],
        Math.max(1, lw - 1));
    }
  }

  PG.lamaCrossingDraw = {
    llamaCanvas, flatLlama, background, farLayer, drawBank, drawRopes, drawLanes, drawTile, tileCanvas, crackLines,
    obstacleLen, drawObstacle, drawBarrier, drawTireMarks, worldIcon,
    clearCache: () => cache.clear(),
  };
})();
