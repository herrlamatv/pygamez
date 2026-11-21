/*
 * lamamines_draw.js - Zeichen-Bausteine von Lama Mines (Port von games/lamamines_draw.py)
 * ======================================================================================
 * - background   : Höhlenwand (Verlauf, Gesteinsbrocken, Körnung, leuchtende
 *                  Kristalladern, Vignette), gecacht je Auflösung,
 * - board        : Holzrahmen mit Eisenwinkeln und Nieten um die dunkle Grube,
 * - tileHidden   : ungeöffnetes Feld = Steinquader mit Fase und Sprenkeln
 *                  (4 Varianten, normal / Maus darüber),
 * - tileOpen     : ausgehobenes Feld (Grube mit Innenschatten),
 * - tileBoom     : Feld der getroffenen Mine (rot glühend, Brandfleck),
 * - dim          : Abdunklung für das nach der Runde aufgedeckte Feld,
 * - gem          : geschliffener Edelstein (Brillant, 4 Farben),
 * - mine         : Seemine mit Hörnern und Signallicht (an/aus),
 * - face         : fertige Vorderseite eines aufgedeckten Feldes,
 * - glow         : Lichthof auf Schwarz - mit drawGlow additiv ("lighter") gezeichnet,
 * - minerLlama   : das Maskottchen - Lama aus lamacrossing_draw.js mit Grubenhelm,
 *                  Stirnlampe, auf Wunsch rußig,
 * - drawCart     : Lore auf Schienen, die sich mit Edelsteinen füllt,
 * - drawPickaxe  : Spitzhacke, drawMark : Auto-Markierung, sparkle : Funkeln,
 * - icon         : Piktogramme der Knöpfe.
 *
 * Wie im Original werden Edelstein, Mine und Maskottchen in doppelter Größe
 * gezeichnet (hier: Koordinaten x2 mit Maßstab 0,5 - die Kantenglättung der
 * Canvas ersetzt smoothscale). Alle Formen hängen nur von Größe, Index und Zeit
 * ab (kein Zufall). pygame zeichnet auf SRCALPHA-Flächen mit "Alpha ersetzen";
 * wo das Original das ausnutzt (Lichthof der Kristalle, rotes Licht der Mine),
 * wird die Fläche erst ausgestanzt und dann halbdurchsichtig gefüllt.
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

  // Farben -------------------------------------------------------------------
  const GEM_COLS = [
    [30, 212, 174], // Türkis (Spielfarbe) - der Edelstein im Feld
    [236, 72, 104], // Rubin
    [84, 140, 248], // Saphir
    [250, 196, 64], // Topas
  ];
  const STONE = [66, 70, 98];
  const STONE_HOT = [86, 92, 128];
  const STONE_D = [32, 34, 52];
  const STONE_L = [112, 118, 156];
  const PIT = [17, 18, 30];
  const PIT_D = [9, 9, 17];
  const PIT_L = [44, 48, 72];
  const WOOD = [112, 72, 42];
  const WOOD_D = [66, 40, 24];
  const WOOD_L = [146, 98, 58];
  const IRON = [104, 108, 124];
  const IRON_D = [60, 62, 74];
  const IRON_L = [168, 172, 188];
  const HELMET = [248, 192, 44];
  const MINE_BODY = [48, 46, 58];
  const MINE_HORN = [92, 90, 104];
  const MINE_RED = [255, 64, 70];
  const MINE_RED_OFF = [120, 32, 40];

  const cache = new Map();
  const scale = () => (PG.app && PG.app.pixelScale) || 1;

  function cached(key, build) {
    const k = key + "|" + scale();
    let c = cache.get(k);
    if (!c) {
      if (cache.size > 400) cache.clear();
      c = build();
      cache.set(k, c);
    }
    return c;
  }
  function clearCache() {
    cache.clear();
  }

  /** Canvas w x h (logisch) in Bildschirmschärfe; ss = Zeichenmaßstab (2 = Koordinaten doppelt). */
  function canvas(w, h, ss = 1) {
    const s = scale();
    const c = ui.makeCanvas(Math.max(1, Math.ceil(w * s)), Math.max(1, Math.ceil(h * s)));
    const g = c.getContext("2d");
    g.scale(s / ss, s / ss);
    c.lw = w;
    c.lh = h;
    return [c, g];
  }

  /** Ganzzahl-Streuwert 0..2^32-1 (deterministisch, wie _hash in Python). */
  function hash(i) {
    let v = (Math.imul(i, 2654435761 | 0) + 40503) >>> 0;
    v = Math.imul((v ^ (v >>> 15)) >>> 0, 2246822519 | 0) >>> 0;
    return (v ^ (v >>> 13)) >>> 0;
  }

  /** f > 0 Richtung Weiß, f < 0 Richtung Schwarz. */
  function shade(col, f) {
    return ui.mix(col, f > 0 ? [255, 255, 255] : [0, 0, 0], Math.abs(f));
  }

  function fill(g, col, x, y, w, h) {
    if (w <= 0 || h <= 0) return;
    g.fillStyle = ui.col(col);
    g.fillRect(x, y, w, h);
  }

  function gradient(g, x, y, w, h, top, bottom) {
    const hh = Math.max(1, h - 1);
    for (let yy = 0; yy < h; yy++) fill(g, ui.mix(top, bottom, yy / hh), x, y + yy, w, 1);
  }

  /** pygame "Alpha ersetzen": Fläche erst ausstanzen, dann (halbdurchsichtig) füllen. */
  function replaceCircle(g, col, c, r) {
    g.save();
    g.globalCompositeOperation = "destination-out";
    draw.circle(g, [0, 0, 0], c, r);
    g.restore();
    draw.circle(g, col, c, r);
  }

  /** pygame Rect.inflate (C-Ganzzahldivision). */
  function inflate(r, dx, dy) {
    return [r[0] - trunc(dx / 2), r[1] - trunc(dy / 2), r[2] + dx, r[3] + dy];
  }

  /** Bild mittig bei (cx, cy) wie blit(img, img.get_rect(center=...)). */
  function blitCenter(ctx, img, cx, cy, w, h) {
    w = w === undefined ? img.lw : w;
    h = h === undefined ? img.lh : h;
    ctx.drawImage(img, trunc(cx) - fdiv(w, 2), trunc(cy) - fdiv(h, 2), w, h);
  }

  // ============================================================= Hintergrund
  /** Ein schlanker Kristall (Sechseck-Prisma) mit Glanzkante. */
  function crystal(s, x, y, h, ang, col) {
    const ca = Math.cos(ang), sa = Math.sin(ang);
    const w = h * 0.26;
    const p = (u, v) => [x + ca * u * h - sa * v * w, y + sa * u * h + ca * v * w];
    draw.polygon(s, shade(col, -0.45), [p(0, -0.8), p(0.78, -1.0), p(1.0, 0), p(0.78, 1.0), p(0, 0.8)]);
    draw.polygon(s, col, [p(0, -0.8), p(0.78, -1.0), p(1.0, 0), p(0.05, 0)]);
    draw.line(s, shade(col, 0.55), p(0.08, -0.35), p(0.8, -0.55), Math.max(1, trunc(h * 0.05)));
  }

  /** Höhlenwand: dunkler Verlauf, Gesteinsbrocken, Körnung und Kristalle. */
  function background(w, h) {
    return cached("bg|" + w + "|" + h, () => {
      const k = Math.min(w / 800, h / 600);
      const [c, s] = canvas(w, h);
      gradient(s, 0, 0, w, h, [16, 15, 28], [30, 22, 36]);
      // große, weiche Gesteinsbrocken (etwas heller als die Wand)
      for (let i = 0; i < 22; i++) {
        const v = hash(i * 3 + 1);
        const x = v % w;
        const y = (v >>> 11) % h;
        const r = trunc((24 + ((v >>> 5) % 70)) * Math.max(0.6, k));
        const f = ((v >>> 3) % 100) / 100;
        const col = ui.mix([24, 22, 38], [34, 30, 48], f);
        const rect = [x - r, y - trunc(r * 0.6), 2 * r, trunc(r * 1.2)];
        draw.ellipse(s, col, rect);
        draw.arc(s, ui.mix(col, [60, 54, 80], 0.6), rect, Math.PI * 0.25, Math.PI * 0.85, Math.max(1, trunc(2 * k)));
      }
      // feine Körnung
      const grain = [[40, 36, 56], [12, 11, 20], [28, 26, 42]];
      const ng = fdiv(w * h, 420);
      for (let i = 0; i < ng; i++) {
        const v = hash(5000 + i);
        fill(s, grain[(v >>> 24) % 3], v % w, Math.floor(v / w) % h, 1, 1);
      }
      // Kristalladern mit Lichthof
      const spots = [[0.03, 0.3, 0], [0.97, 0.22, 1], [0.02, 0.78, 2], [0.98, 0.7, 0], [0.5, 0.985, 1], [0.3, 0.99, 2], [0.74, 0.99, 0]];
      const cols = [[30, 200, 168], [150, 96, 230], [70, 150, 240]];
      spots.forEach(([fx, fy, ci], i) => {
        const x = trunc(w * fx), y = trunc(h * fy);
        const col = cols[ci];
        const r = trunc(46 * Math.max(0.6, k));
        // Lichthof: drei Kreise, jeder ersetzt den Alphawert des größeren
        const [gc, gg] = canvas(r * 2, r * 2);
        for (const [j, a] of [[3, 14], [2, 18], [1, 24]]) replaceCircle(gg, [col[0], col[1], col[2], a], [r, r], trunc((r * j) / 3));
        s.drawImage(gc, x - r, y - r, gc.lw, gc.lh);
        const base = -Math.PI / 2 + (fx > 0.5 ? 0.5 : -0.5) * (fy < 0.9 ? 1 : 0.4);
        for (let j = 0; j < 3; j++) {
          const hh = (18 + 9 * ((i + j) % 3)) * Math.max(0.6, k);
          crystal(s, x + (j - 1) * 7 * k, y, hh, base + (j - 1) * 0.42, col);
        }
      });
      // Vignette: 10 Ringe von außen nach innen, jeder etwas heller (Alpha 80 -> 0)
      const steps = 10;
      const band = Math.max(1, trunc(Math.min(w, h) * 0.012));
      for (let i = 0; i < steps; i++) {
        const a = fdiv(80 * (steps - i) * (steps - i), steps * steps);
        if (a > 0) draw.rect(s, [0, 0, 0, a], [i * band, i * band, w - 2 * i * band, h - 2 * i * band], band);
      }
      return c;
    });
  }

  // ============================================================= Spielbrett
  /** Holzrahmen (Breite pad) mit Eisenwinkeln um die dunkle Grube. */
  function board(side, pad) {
    return cached("board|" + side + "|" + pad, () => {
      const [c, s] = canvas(side, side);
      const rad = Math.max(4, trunc(side * 0.035));
      draw.rect(s, WOOD_D, [0, 0, side, side], 0, rad);
      draw.rect(s, WOOD, [1, 1, side - 2, side - 3], 0, rad);
      // Maserung: Bretter oben/unten waagerecht, links/rechts senkrecht
      const lw = Math.max(1, fdiv(side, 260));
      for (let j = 0; j < 3; j++) {
        const f = (j + 1) / 4;
        const y0 = trunc(pad * f);
        draw.line(s, WOOD_D, [pad, y0], [side - pad, y0 + (j - 1)], lw);
        draw.line(s, WOOD_D, [pad, side - y0], [side - pad, side - y0 - (j - 1)], lw);
        const x0 = trunc(pad * f);
        draw.line(s, WOOD_D, [x0, pad], [x0 + (j - 1), side - pad], lw);
        draw.line(s, WOOD_D, [side - x0, pad], [side - x0 - (j - 1), side - pad], lw);
      }
      for (let i = 0; i < 6; i++) {
        const v = hash(i * 11 + 3);
        const x = pad + (v % Math.max(1, side - 2 * pad));
        const kw = Math.max(3, fdiv(pad, 2)), kh = Math.max(2, fdiv(pad, 4));
        const cy = i % 2 === 0 ? fdiv(pad, 2) : side - fdiv(pad, 2);
        draw.ellipse(s, WOOD_D, [x - fdiv(kw, 2), cy - fdiv(kh, 2), kw, kh], Math.max(1, lw));
      }
      draw.line(s, WOOD_L, [rad, 1], [side - rad, 1], Math.max(1, lw));
      // Fugen an den Ecken (Gehrung)
      for (const [[ax, ay], [bx, by]] of [[[0, 0], [pad, pad]], [[side, 0], [side - pad, pad]],
        [[0, side], [pad, side - pad]], [[side, side], [side - pad, side - pad]]]) {
        draw.line(s, WOOD_D, [ax, ay], [bx, by], Math.max(1, lw));
      }
      // Grube mit Innenschatten
      const inner = [pad, pad, side - 2 * pad, side - 2 * pad];
      const irad = Math.max(3, fdiv(rad, 2));
      draw.rect(s, PIT_D, inner, 0, irad);
      const in2 = inflate(inner, -2, -2);
      in2[1] += Math.max(1, fdiv(pad, 5));
      draw.rect(s, [14, 14, 24], in2, 0, irad);
      draw.rect(s, [40, 26, 16], inflate(inner, 2, 2), 1, irad);
      // Eisenwinkel mit Nieten
      const a = trunc(pad * 2.1);
      const t = Math.max(3, trunc(pad * 0.62));
      const nr = Math.max(1, trunc(pad * 0.14));
      const br = Math.max(1, fdiv(t, 3));
      for (const [cx, cy, sx, sy] of [[0, 0, 1, 1], [side, 0, -1, 1], [0, side, 1, -1], [side, side, -1, -1]]) {
        const x0 = sx > 0 ? cx : cx - a;
        const y0 = sy > 0 ? cy : cy - t;
        draw.rect(s, IRON_D, [x0, y0 + sy, a, t], 0, br);
        draw.rect(s, IRON, [x0, y0, a, t], 0, br);
        const x1 = sx > 0 ? cx : cx - t;
        const y1 = sy > 0 ? cy : cy - a;
        draw.rect(s, IRON_D, [x1 + sx, y1, t, a], 0, br);
        draw.rect(s, IRON, [x1, y1, t, a], 0, br);
        for (const [ox, oy] of [[fdiv(t, 2), fdiv(t, 2)], [a - fdiv(t, 2), fdiv(t, 2)], [fdiv(t, 2), a - fdiv(t, 2)]]) {
          const px = cx + sx * ox, py = cy + sy * oy;
          draw.circle(s, IRON_D, [px, py + 1], nr + 1);
          draw.circle(s, IRON_L, [px, py], nr);
        }
      }
      return c;
    });
  }

  // ============================================================= Felder
  const tileRad = (size) => Math.max(2, trunc(size * 0.16));

  /** Steinquader size x size; v 0..3 = Sprenkel-Variante. */
  function tileHidden(size, v, hot = false) {
    const variant = ((v % 4) + 4) % 4;
    return cached("hid|" + size + "|" + variant + "|" + (hot ? 1 : 0), () => {
      const [c, s] = canvas(size, size);
      const rad = tileRad(size);
      const bev = Math.max(2, trunc(size * 0.09));
      const body = hot ? STONE_HOT : STONE;
      draw.rect(s, STONE_D, [0, 0, size, size], 0, rad);
      draw.rect(s, body, [0, 0, size, size - bev], 0, rad);
      const face = { x: bev, y: bev, w: size - 2 * bev, h: size - 3 * bev };
      draw.rect(s, shade(body, 0.1), [face.x, face.y, face.w, face.h], 0, Math.max(1, rad - fdiv(bev, 2)));
      draw.line(s, hot ? shade(STONE_L, 0.2) : STONE_L, [rad, Math.max(1, fdiv(bev, 2))],
        [size - rad - 1, Math.max(1, fdiv(bev, 2))], Math.max(1, fdiv(bev, 2)));
      // Sprenkel und ein feiner Riss je Variante (variant = var % 4 wie der Cache-Schlüssel)
      for (let j = 0; j < 5; j++) {
        const hv = hash(v * 31 + j * 7 + 1);
        const x = face.x + 2 + (hv % Math.max(1, face.w - 4));
        const y = face.y + 2 + ((hv >>> 9) % Math.max(1, face.h - 4));
        const r = Math.max(1, trunc(size * (0.022 + 0.012 * ((hv >>> 4) % 2))));
        draw.circle(s, j % 2 === 0 ? shade(body, -0.25) : shade(body, 0.22), [x, y], r);
      }
      if (v % 2 === 1) {
        // feiner Haarriss von der oberen Kante schräg nach unten
        const hv = hash(v * 97 + 5);
        const x0 = face.x + face.w * (0.55 + (hv % 25) / 100);
        draw.lines(s, shade(body, -0.16), false, [[x0, face.y], [x0 - face.w * 0.06, face.y + face.h * 0.16],
          [x0 + face.w * 0.02, face.y + face.h * 0.27], [x0 - face.w * 0.08, face.y + face.h * 0.42]], 1);
      }
      return c;
    });
  }

  /** Ausgehobenes Feld: dunkle Grube mit Innenschatten oben und Lichtkante unten. */
  function tileOpen(size) {
    return cached("open|" + size, () => {
      const [c, s] = canvas(size, size);
      const rad = tileRad(size);
      const bev = Math.max(2, trunc(size * 0.09));
      draw.rect(s, PIT_L, [0, 0, size, size], 0, rad);
      draw.rect(s, PIT_D, [0, 0, size, size - Math.max(1, fdiv(bev, 2))], 0, rad);
      const b3 = Math.max(1, fdiv(bev, 3));
      draw.rect(s, PIT, [b3, bev, size - 2 * b3, size - bev - Math.max(1, fdiv(bev, 2)) - 1], 0, Math.max(1, rad - 1));
      return c;
    });
  }

  /** Feld der getroffenen Mine: rot glühend mit dunklem Brandfleck. */
  function tileBoom(size) {
    return cached("boom|" + size, () => {
      const [c, s] = canvas(size, size);
      const rad = tileRad(size);
      draw.rect(s, [110, 20, 28], [0, 0, size, size], 0, rad);
      draw.rect(s, [176, 38, 44], [1, 1, size - 2, size - 3], 0, rad);
      const cc = fdiv(size, 2);
      [[214, 70, 50], [240, 120, 60], [250, 170, 90]].forEach((col, i) => {
        draw.circle(s, col, [cc, cc], trunc(size * (0.44 - 0.11 * i)));
      });
      const bw = trunc(size * 0.64), bh = trunc(size * 0.5);
      draw.ellipse(s, [70, 22, 20], [cc - fdiv(bw, 2), trunc(size * 0.58) - fdiv(bh, 2), bw, bh]);
      for (let j = 0; j < 6; j++) {
        const a = (TAU * j) / 6 + 0.3;
        const x1 = cc + Math.cos(a) * size * 0.46;
        const y1 = cc + Math.sin(a) * size * 0.4;
        draw.line(s, [90, 26, 22], [cc, trunc(size * 0.58)], [x1, y1], Math.max(1, fdiv(size, 30)));
      }
      return c;
    });
  }

  /** Halbdurchsichtige Abdunklung eines Feldes (abgerundet). */
  function dim(size, alpha = 150) {
    return cached("dim|" + size + "|" + alpha, () => {
      const [c, s] = canvas(size, size);
      draw.rect(s, [6, 6, 14, alpha], [0, 0, size, size], 0, tileRad(size));
      return c;
    });
  }

  // ============================================================= Edelstein + Mine
  function drawGem(s, n, col) {
    const TL = [n * 0.3, n * 0.17], TR = [n * 0.7, n * 0.17];
    const L = [n * 0.09, n * 0.39], R = [n * 0.91, n * 0.39], B = [n * 0.5, n * 0.9];
    const T = [n * 0.5, n * 0.17];
    const G1 = [n * 0.35, n * 0.39], G2 = [n * 0.65, n * 0.39];
    const outline = [TL, TR, R, B, L];
    // weicher Schatten
    draw.ellipse(s, [0, 0, 0, 70], [n * 0.22, n * 0.84, n * 0.56, n * 0.1]);
    draw.polygon(s, shade(col, -0.55), outline.map(([x, y]) => [x + n * 0.012, y + n * 0.02]));
    draw.polygon(s, col, outline);
    draw.polygon(s, shade(col, 0.5), [TL, T, G1, L]);
    draw.polygon(s, shade(col, 0.22), [T, TR, R, G2]);
    draw.polygon(s, shade(col, 0.7), [T, G2, G1]);
    draw.polygon(s, shade(col, -0.05), [L, G1, B]);
    draw.polygon(s, shade(col, 0.28), [G1, G2, B]);
    draw.polygon(s, shade(col, -0.38), [G2, R, B]);
    const edge = shade(col, 0.82);
    const lw = Math.max(1, fdiv(n, 70));
    draw.line(s, edge, L, R, lw);
    draw.lines(s, edge, true, outline, lw);
    draw.line(s, shade(col, 0.6), G1, B, lw);
    draw.line(s, shade(col, 0.6), G2, B, lw);
    // Glanzpunkte
    draw.circle(s, [255, 255, 255], [n * 0.33, n * 0.25], n * 0.035);
    draw.polygon(s, [255, 255, 255], [[n * 0.4, n * 0.47], [n * 0.46, n * 0.47], [n * 0.5, n * 0.7]]);
  }

  /** Geschliffener Edelstein size x size (gecacht). */
  function gem(size, colIdx = 0) {
    const ci = colIdx % GEM_COLS.length;
    return cached("gem|" + size + "|" + ci, () => {
      const [c, g] = canvas(size, size, 2);
      drawGem(g, size * 2, GEM_COLS[ci]);
      return c;
    });
  }

  function drawMine(s, n, lit) {
    const cx = n * 0.5, cy = n * 0.54;
    const R = n * 0.25;
    draw.ellipse(s, [0, 0, 0, 80], [n * 0.2, n * 0.84, n * 0.6, n * 0.1]);
    const lw = Math.max(2, trunc(n * 0.075));
    for (let i = 0; i < 8; i++) {
      const a = (TAU * i) / 8 + Math.PI / 8;
      const bx = cx + Math.cos(a) * R * 0.8, by = cy + Math.sin(a) * R * 0.8;
      const tx = cx + Math.cos(a) * R * 1.42, ty = cy + Math.sin(a) * R * 1.42;
      draw.line(s, shade(MINE_HORN, -0.3), [bx, by], [tx, ty], lw);
      draw.circle(s, MINE_HORN, [tx, ty], n * 0.058);
      draw.circle(s, shade(MINE_HORN, 0.45), [tx - n * 0.015, ty - n * 0.015], n * 0.022);
    }
    draw.circle(s, shade(MINE_BODY, -0.45), [cx, cy + n * 0.02], R);
    draw.circle(s, MINE_BODY, [cx, cy], R);
    draw.ellipse(s, shade(MINE_BODY, -0.25), [cx - R, cy - R * 0.18, 2 * R, R * 0.36], Math.max(1, trunc(n * 0.02)));
    draw.ellipse(s, shade(MINE_BODY, 0.38), [cx - R * 0.7, cy - R * 0.78, R * 0.75, R * 0.55]);
    draw.circle(s, [230, 230, 240], [cx - R * 0.42, cy - R * 0.55], n * 0.028);
    const red = lit ? MINE_RED : MINE_RED_OFF;
    if (lit) replaceCircle(s, [255, 120, 110, 90], [cx, cy - R * 0.05], n * 0.13);
    draw.circle(s, shade(red, -0.4), [cx, cy - R * 0.05], n * 0.075);
    draw.circle(s, red, [cx, cy - R * 0.05], n * 0.058);
    draw.circle(s, shade(red, 0.6), [cx - n * 0.018, cy - R * 0.05 - n * 0.02], n * 0.018);
  }

  /** Seemine size x size mit Signallicht (lit = leuchtet). */
  function mine(size, lit = true) {
    return cached("mine|" + size + "|" + (lit ? 1 : 0), () => {
      const [c, g] = canvas(size, size, 2);
      drawMine(g, size * 2, lit);
      return c;
    });
  }

  /**
   * Fertige Vorderseite eines aufgedeckten Feldes: "gem" (Edelstein in der Grube),
   * "mine" (Mine, Licht aus) oder "boom" (getroffene Mine auf glühendem Grund).
   */
  function face(size, kind, dimmed = false) {
    return cached("face|" + size + "|" + kind + "|" + (dimmed ? 1 : 0), () => {
      const [c, s] = canvas(size, size);
      const base = kind === "boom" ? tileBoom(size) : tileOpen(size);
      s.drawImage(base, 0, 0, size, size);
      const g = Math.max(4, trunc(size * 0.8));
      const img = kind === "gem" ? gem(g) : mine(g, kind === "boom");
      blitCenter(s, img, fdiv(size, 2), fdiv(size, 2));
      if (dimmed) s.drawImage(dim(size), 0, 0, size, size);
      return c;
    });
  }

  /** Lichthof mit Radius r auf schwarzem Grund (Größe 2r+2) - additiv zeichnen. */
  function glow(r, col) {
    return cached("glow|" + r + "|" + col[0] + "," + col[1] + "," + col[2], () => {
      const [c, s] = canvas(2 * r + 2, 2 * r + 2);
      fill(s, [0, 0, 0], 0, 0, 2 * r + 2, 2 * r + 2);
      const steps = 8;
      for (let i = 0; i < steps; i++) {
        const f = (i + 1) / steps; // 1/8 .. 1 (außen -> innen)
        const rr = Math.max(1, trunc(r * (1 - i / steps)));
        const a = f * f;
        draw.circle(s, [trunc(col[0] * a), trunc(col[1] * a), trunc(col[2] * a)], [r + 1, r + 1], rr);
      }
      return c;
    });
  }
  /** Lichthof additiv zeichnen (wie blit mit BLEND_RGB_ADD), linke obere Ecke x/y. */
  function drawGlow(ctx, r, col, x, y) {
    if (col[0] <= 0 && col[1] <= 0 && col[2] <= 0) return;
    const g = glow(r, col);
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.drawImage(g, trunc(x), trunc(y), g.lw, g.lh);
    ctx.restore();
  }

  /** Vierzackiges Funkeln (Mittelpunkt x/y, Zacken-Länge r). */
  function sparkle(s, x, y, r, col) {
    if (r < 1) return;
    const q = r * 0.28;
    draw.polygon(s, col, [[x, y - r], [x + q, y - q], [x + r, y], [x + q, y + q], [x, y + r], [x - q, y + q], [x - r, y], [x - q, y - q]]);
    draw.circle(s, [255, 255, 255], [trunc(x), trunc(y)], Math.max(1, trunc(r * 0.22)));
  }

  /** Auto-Markierung: Rahmen in der Spielfarbe + kleine Raute in der Mitte. */
  function drawMark(s, rect, col, t, idx) {
    const r = rect;
    const rad = tileRad(r.w);
    const lw = Math.max(2, fdiv(r.w, 18));
    const gl = 0.5 + 0.5 * Math.sin(t * 3 + idx * 0.7);
    const edge = ui.mix(col, [255, 255, 255], 0.25 * gl);
    draw.rect(s, edge, inflate([r.x, r.y, r.w, r.h], -lw, -lw), lw, rad);
    const cx = r.x + fdiv(r.w, 2), cy = r.y + fdiv(r.h, 2) - fdiv(r.h, 20);
    const d = Math.max(3, fdiv(r.w, 7));
    const pts = [[cx, cy - d], [cx + d, cy], [cx, cy + d], [cx - d, cy]];
    draw.polygon(s, shade(col, -0.5), pts.map(([x, y]) => [x + 1, y + 2]));
    draw.polygon(s, edge, pts);
    draw.polygon(s, shade(col, 0.7), [[cx, cy - d], [cx + fdiv(d, 2), cy - fdiv(d, 2)], [cx, cy], [cx - fdiv(d, 2), cy - fdiv(d, 2)]]);
  }

  // ============================================================= Maskottchen
  /** Grubenhelm mit Stirnlampe auf dem Lama-Kopf (Koordinaten wie drawLlama). */
  function helmet(s, n) {
    const cx = n * 0.705, by = n * 0.132;
    const rx = n * 0.128, ry = n * 0.118;
    const dome = [];
    for (let i = 0; i < 17; i++) dome.push([cx + rx * Math.cos((Math.PI * i) / 16), by - ry * Math.sin((Math.PI * i) / 16)]);
    draw.polygon(s, shade(HELMET, -0.35), dome.map(([x, y]) => [x, y + n * 0.012]));
    draw.polygon(s, HELMET, dome);
    const hi = [];
    for (let i = 0; i < 9; i++) {
      const a = Math.PI * (0.45 + (0.5 * i) / 8);
      hi.push([cx - rx * 0.15 + rx * 0.62 * Math.cos(a), by - ry * 0.18 - ry * 0.7 * Math.sin(a)]);
    }
    draw.lines(s, shade(HELMET, 0.55), false, hi, Math.max(1, trunc(n * 0.02)));
    // Mittelgrat
    draw.polygon(s, shade(HELMET, -0.15), [[cx - n * 0.012, by - ry * 0.98], [cx + n * 0.012, by - ry * 0.98],
      [cx + n * 0.016, by], [cx - n * 0.016, by]]);
    // Krempe
    draw.rect(s, shade(HELMET, -0.3), [cx - rx * 1.16, by - n * 0.006, rx * 2.38, n * 0.034], 0, trunc(n * 0.016));
    draw.rect(s, shade(HELMET, -0.05), [cx - rx * 1.16, by - n * 0.01, rx * 2.38, n * 0.024], 0, trunc(n * 0.012));
    // Stirnlampe
    const lx = cx + rx * 0.8, ly = by - ry * 0.5;
    draw.rect(s, [64, 64, 76], [lx - n * 0.04, ly - n * 0.034, n * 0.062, n * 0.068], 0, trunc(n * 0.014));
    draw.circle(s, [255, 236, 160], [lx + n * 0.02, ly], n * 0.03);
    draw.circle(s, [255, 255, 230], [lx + n * 0.024, ly - n * 0.006], n * 0.013);
  }

  /** Rußflecken nach der Explosion (deckende Grautöne). */
  function soot(s, n) {
    for (const [x, y, r] of [[0.33, 0.5, 0.05], [0.47, 0.46, 0.038], [0.58, 0.55, 0.03], [0.66, 0.3, 0.03], [0.8, 0.22, 0.024], [0.26, 0.56, 0.026]]) {
      draw.circle(s, [126, 118, 114], [n * x, n * y], n * r);
      draw.circle(s, [98, 92, 92], [n * x + n * r * 0.2, n * y + n * r * 0.2], n * r * 0.55);
    }
  }

  /** Maskottchen size x size: Lama mit Grubenhelm (pose: stand/hop/fall). */
  function minerLlama(size, pose, scarf, sooty = false) {
    return cached("miner|" + size + "|" + pose + "|" + scarf.join(",") + "|" + (sooty ? 1 : 0), () => {
      const [c, g] = canvas(size, size, 2);
      const n = size * 2;
      PG.lamaCrossingDraw.drawLlama(g, n, pose, scarf);
      if (sooty) soot(g, n);
      helmet(g, n);
      return c;
    });
  }

  /** Spitzhacke, Stiel-Fuß bei (x, y), Höhe h (lehnt nach rechts). */
  function drawPickaxe(s, x, y, h, k) {
    const lw = Math.max(2, trunc(h * 0.07));
    const top = [x + h * 0.32, y - h];
    draw.line(s, [88, 56, 30], [x, y], top, lw + 1);
    draw.line(s, [150, 104, 60], [x + 1, y - 1], [top[0] + 1, top[1] + 1], Math.max(1, fdiv(lw, 2)));
    // Kopf: gebogenes Eisen quer zum Stiel
    const pts = [];
    for (let i = 0; i < 9; i++) {
      const u = -1 + (2 * i) / 8;
      pts.push([top[0] + u * h * 0.34, top[1] + u * u * h * 0.12 - h * 0.02]);
    }
    draw.lines(s, IRON_D, false, pts.map(([px, py]) => [px, py + 2]), lw + 1);
    draw.lines(s, IRON_L, false, pts, lw);
  }

  /** Lore (Unterkante der Räder bei y, linke Kante x, Breite w); fill = Edelsteine (0..7). */
  function drawCart(s, x, y, w, k, fillN, gemSize) {
    const h = w * 0.46;
    const top = y - w * 0.16 - h;
    // Schienen
    draw.line(s, [70, 64, 70], [x - w * 0.15, y], [x + w * 1.15, y], Math.max(2, trunc(w * 0.035)));
    for (let i = 0; i < 5; i++) {
      const sx = x - w * 0.1 + i * w * 0.31;
      draw.line(s, [86, 58, 36], [sx, y + 1], [sx + w * 0.12, y + 1], Math.max(2, trunc(w * 0.05)));
    }
    // Edelstein-Haufen (ragt über den Rand; fy in Edelstein-Größen über der Kante)
    const spots = [[0.3, 0.1, 1], [0.52, 0.06, 0], [0.74, 0.1, 2], [0.41, -0.34, 3], [0.63, -0.32, 0], [0.52, -0.7, 1], [0.18, -0.08, 2]];
    for (let i = 0; i < Math.min(fillN, spots.length); i++) {
      const [fx, fy, ci] = spots[i];
      blitCenter(s, gem(gemSize, ci), trunc(x + w * fx), trunc(top + gemSize * fy));
    }
    // Wagenkasten (Trapez) mit Rand und Nieten
    const body = [[x, top], [x + w, top], [x + w * 0.88, top + h], [x + w * 0.12, top + h]];
    draw.polygon(s, IRON_D, body.map(([px, py]) => [px, py + 2]));
    draw.polygon(s, IRON, body);
    draw.polygon(s, shade(IRON, -0.18), [[x + w * 0.05, top + h * 0.55], [x + w * 0.95, top + h * 0.55], [x + w * 0.88, top + h], [x + w * 0.12, top + h]]);
    draw.rect(s, IRON_L, [trunc(x - w * 0.03), trunc(top - w * 0.02), trunc(w * 1.06), Math.max(2, trunc(w * 0.07))], 0, Math.max(1, trunc(w * 0.03)));
    for (let i = 0; i < 4; i++) {
      const rx = x + w * (0.2 + 0.2 * i);
      draw.circle(s, IRON_L, [trunc(rx), trunc(top + h * 0.32)], Math.max(1, trunc(w * 0.025)));
    }
    // Räder
    for (const fx of [0.27, 0.73]) {
      const cx = trunc(x + w * fx), cy = trunc(y - w * 0.12);
      const rr = Math.max(3, trunc(w * 0.12));
      draw.circle(s, [30, 30, 38], [cx, cy], rr);
      draw.circle(s, [120, 120, 134], [cx, cy], Math.max(1, rr - Math.max(1, fdiv(rr, 3))), Math.max(1, fdiv(rr, 4)));
      draw.circle(s, [170, 170, 184], [cx, cy], Math.max(1, fdiv(rr, 3)));
    }
  }

  // ============================================================= Piktogramme
  /** Piktogramm in rect (Linienstärke wächst mit der Größe). */
  function icon(s, name, rect, col) {
    const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
    const cx = r.x + fdiv(r.w, 2), cy = r.y + fdiv(r.h, 2);
    const right = r.x + r.w, bottom = r.y + r.h;
    const u = r.w / 10;
    const lw = Math.max(1, trunc(u * 0.9));
    if (name === "auto") {
      // Kreispfeil (Wiederholen) mit Play-Dreieck in der Mitte
      const rad = u * 3.8;
      const bw = trunc(rad * 2);
      draw.arc(s, col, [cx - fdiv(bw, 2), cy - fdiv(bw, 2), bw, bw], Math.PI * 0.35, Math.PI * 1.95, Math.max(1, lw));
      const a = Math.PI * 0.35;
      const ex = cx + Math.cos(a) * rad, ey = cy - Math.sin(a) * rad;
      draw.polygon(s, col, [[ex + u * 1.6, ey - u * 0.2], [ex - u * 0.6, ey - u * 1.3], [ex - u * 0.2, ey + u * 1.2]]);
      draw.polygon(s, col, [[cx - u * 1.0, cy - u * 1.5], [cx + u * 1.6, cy], [cx - u * 1.0, cy + u * 1.5]]);
    } else if (name === "stats") {
      const base = bottom - u * 1.6;
      [3.0, 5.2, 4.0, 6.8].forEach((hh, i) => {
        const x = r.x + u * (1.4 + i * 2.0);
        draw.rect(s, col, [trunc(x), trunc(base - hh * u), Math.max(1, trunc(u * 1.3)), trunc(hh * u)]);
      });
      draw.line(s, col, [r.x + u, base], [right - u, base], Math.max(1, lw > 1 ? lw - 1 : 1));
    } else if (name === "fair") {
      const pts = [[cx, r.y + u * 1.0], [cx + u * 3.6, r.y + u * 2.2], [cx + u * 3.2, cy + u * 1.4],
        [cx, bottom - u * 0.8], [cx - u * 3.2, cy + u * 1.4], [cx - u * 3.6, r.y + u * 2.2]];
      draw.polygon(s, col, pts, Math.max(1, lw));
      draw.lines(s, col, false, [[cx - u * 1.6, cy], [cx - u * 0.3, cy + u * 1.4], [cx + u * 1.8, cy - u * 1.4]], Math.max(1, lw));
    } else if (name === "turbo") {
      draw.polygon(s, col, [[cx + u * 1.0, r.y + u * 0.6], [cx - u * 2.6, cy + u * 0.6], [cx - u * 0.2, cy + u * 0.6],
        [cx - u * 1.0, bottom - u * 0.6], [cx + u * 2.6, cy - u * 0.8], [cx + u * 0.2, cy - u * 0.8]]);
    } else if (name === "dice") {
      const bw = trunc(u * 7.2);
      draw.rect(s, col, [cx - fdiv(bw, 2), cy - fdiv(bw, 2), bw, bw], Math.max(1, lw), Math.max(2, trunc(u * 1.4)));
      const pr = Math.max(1, trunc(u * 0.75));
      for (const [dx, dy] of [[-1, -1], [1, 1], [0, 0], [1, -1], [-1, 1]]) {
        draw.circle(s, col, [trunc(cx + dx * u * 1.9), trunc(cy + dy * u * 1.9)], pr);
      }
    } else if (name === "close") {
      const d = u * 2.6;
      draw.line(s, col, [cx - d, cy - d], [cx + d, cy + d], Math.max(2, lw));
      draw.line(s, col, [cx - d, cy + d], [cx + d, cy - d], Math.max(2, lw));
    } else if (name === "check") {
      draw.lines(s, col, false, [[cx - u * 3.0, cy], [cx - u * 0.8, cy + u * 2.4], [cx + u * 3.2, cy - u * 2.6]], Math.max(2, trunc(lw * 1.4)));
    } else if (name === "cross") {
      const d = u * 2.6;
      draw.line(s, col, [cx - d, cy - d], [cx + d, cy + d], Math.max(2, trunc(lw * 1.4)));
      draw.line(s, col, [cx - d, cy + d], [cx + d, cy - d], Math.max(2, trunc(lw * 1.4)));
    } else if (name === "left" || name === "right") {
      const sx = name === "left" ? -1 : 1;
      draw.polygon(s, col, [[cx + sx * u * 2.0, cy], [cx - sx * u * 1.4, cy - u * 2.6], [cx - sx * u * 1.4, cy + u * 2.6]]);
    } else if (name === "gem") {
      blitCenter(s, gem(Math.max(4, trunc(r.w * 0.9))), cx, cy);
    } else if (name === "mine") {
      blitCenter(s, mine(Math.max(4, trunc(r.w * 0.9))), cx, cy);
    }
  }

  PG.lamaMinesDraw = {
    GEM_COLS, hash, shade, background, board, tileHidden, tileOpen, tileBoom, dim, gem, mine, face,
    glow, drawGlow, sparkle, drawMark, minerLlama, drawPickaxe, drawCart, icon, blitCenter, clearCache,
  };
})();
