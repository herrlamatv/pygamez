/*
 * lamacrash_world.js - Höhenwelten von Lama Crash (Port von games/lamacrash_world.py)
 * ==================================================================================
 * Hintergrund der Bühne, durch den der Ballon steigt (nur Canvas-Pfade, keine Bilder):
 *
 *   0 Wiese (ab x1) · 1 Wolken (ab x1,5) · 2 Gewitter (ab x3)
 *   3 Nordlicht (ab x10) · 4 Weltall (ab x100) · 5 Mond (nur die Landung)
 *
 * Schnittstelle wie in Python (camelCase), PG.lamaCrashWorld:
 *   clearCache(), sky(world, w, h), layer(world, depth, w, h), ground(w, h), groundH(h),
 *   risePx(alt, h), draw(s, rect, world, share, alt, t), moonScene(w, h), nameKey(world),
 *   PARALLAX, NAMES. Flächen sind Canvases mit c.lw / c.lh (logische Größe).
 *
 * Zufall nur über hash (deterministisch), keine Zeit außer t, nie der Platzpunkt.
 *
 * Speicher wie in Python: jede Weltfläche gehört zur Gruppe (Welt, w, h); draw verwirft in
 * jedem Bild alle Gruppen außer der aktuellen und der nächsten Welt (keep), und cached hält
 * höchstens zwei Gruppen. Ohne Überblendung baut draw die nächste Welt vorab - je Bild
 * höchstens eine Fläche. Für die Überblendung gibt es eine bühnengroße Zwischenfläche.
 *
 * pygame zeichnet Primitive auf SRCALPHA-Flächen mit "Alpha ersetzen" (auch die Schatten-
 * Ellipsen am Boden: dort scheint der Himmel durch) - hier mit Ausstanzen + Neuzeichnen
 * nachgebaut; Verläufe aus _grad werden zeilengenau wie in Python berechnet (ImageData).
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const LCD = () => PG.lamaCrossingDraw;
  const PI = Math.PI;
  const TAU = PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);
  const pymod = (a, b) => ((a % b) + b) % b;

  // Tempo der Ebenen (fern .. nah) relativ zur Steighöhe
  const PARALLAX = [0.22, 0.5, 1.0];
  const NAMES = ["meadow", "clouds", "storm", "aurora", "space", "moon"];

  const RISE = 22; // Steigtempo: h * RISE / 10000 Pixel je Takt
  const EASE = 60; // Anfahren: Tempo wächst in den ersten EASE Takten linear
  const GROUND = 30; // Bodenstreifen in % der Bühnenhöhe
  const PAD_X = 15; // Startplatz-Mitte in % der Bühnenbreite
  const PAD_W = 17; // Deckbreite in % der Bühnenhöhe
  const TOP = 35; // Ding 0 einer Höhenreihe steht bei Höhe 0 auf 35 % von h
  const SAT_P = 0.3;
  const PLANET_P = 0.16;
  const EARTH_P = 0.6;
  const FLASH = 1.9;
  const SHOOT = 2.6;

  const SKY = [
    [[0, [52, 128, 222]], [560, [110, 180, 242]], [1000, [196, 228, 250]]],
    [[0, [28, 88, 196]], [600, [84, 150, 232]], [1000, [162, 206, 244]]],
    [[0, [20, 22, 36]], [520, [42, 46, 64]], [1000, [70, 76, 94]]],
    [[0, [4, 5, 20]], [450, [10, 14, 46]], [850, [17, 19, 60]], [1000, [28, 22, 70]]],
    [[0, [0, 0, 3]], [700, [2, 3, 12]], [1000, [6, 8, 26]]],
    [[0, [0, 0, 2]], [1000, [8, 8, 18]]],
  ];

  // Wolken-Ebenen je "Welt,Tiefe": [Anzahl bei w = h, Breite ‰ von h min/max, Höhe % der
  // Breite, Palette (Licht, Körper, Schatten), Deckkraft, nur Seiten]
  const CLOUDS = {
    "0,1": [3, 200, 320, 44, [[255, 255, 255], [236, 243, 252], [188, 207, 234]], 225, false],
    "0,2": [2, 340, 480, 42, [[255, 255, 255], [240, 246, 254], [180, 201, 232]], 255, true],
    "1,0": [9, 110, 210, 44, [[234, 243, 254], [210, 227, 248], [174, 199, 236]], 205, false],
    "1,1": [5, 240, 380, 44, [[255, 255, 255], [230, 240, 252], [170, 195, 230]], 240, false],
    "1,2": [2, 520, 760, 42, [[255, 255, 255], [238, 245, 254], [164, 189, 226]], 255, true],
    "2,0": [4, 460, 760, 34, [[86, 92, 116], [62, 66, 88], [44, 46, 64]], 225, false],
    "2,1": [3, 340, 540, 38, [[98, 104, 130], [54, 58, 78], [34, 36, 50]], 240, false],
    "2,2": [2, 600, 900, 36, [[74, 78, 102], [40, 42, 58], [24, 25, 36]], 250, true],
  };
  // Sternen-Ebenen je "Welt,Tiefe": [Pixel je Stern, Deckkraft min/max, ‰ Glanz, ‰ Doppelpixel]
  const STARS = {
    "3,0": [700, 30, 140, 0, 40],
    "3,1": [5200, 90, 230, 60, 200],
    "3,2": [60000, 200, 255, 1000, 0],
    "4,0": [420, 40, 170, 0, 30],
    "4,1": [3400, 110, 255, 80, 250],
    "4,2": [24000, 210, 255, 1000, 0],
  };
  const STAR_COLS = [[238, 242, 255], [255, 238, 216], [206, 222, 255], [255, 250, 244]];
  // ferne Planeten: [Start-Takt ab Weltall-Beginn, x ‰ von w, Radius ‰ von h, Art]
  const PLANETS = [[520, 150, 58, 0], [2500, 860, 32, 1], [4500, 230, 44, 2]];
  // Polarlicht-Bögen je Art: [Unterkante % von h, Bogenhöhe ‰, Scheitel ‰ von w,
  // Welle 1: Höhe ‰ von h, Länge x w, Tempo; Welle 2 ebenso; Wandern x w/s]
  const ARCS = [[68, 100, 400, 40, 0.9, 0.55, 18, 0.33, 0.9, 0.012], [47, 70, 560, 30, 1.3, 0.4, 16, 0.4, 0.7, -0.008]];

  // ============================================================= Steigkurve
  const WORLD_AT = [100, 150, 300, 1000, 10000, 10000000];
  let fbTicks = null;
  /** Takt, an dem jede Welt beginnt (aus PG.lamaCrashLogic; sonst selbst gerechnet). */
  function worldTicks() {
    const L = PG.lamaCrashLogic;
    if (L && typeof L.worldTicks === "function") return L.worldTicks();
    if (!fbTicks) {
      const m = [];
      let mu = 1000000;
      for (let i = 0; i <= 11520; i++) {
        m.push(Math.min(10000000, Math.floor(mu / 10000)));
        mu += Math.floor(mu / 1000);
      }
      fbTicks = WORLD_AT.map((v) => {
        let lo = 0, hi = m.length;
        while (lo < hi) {
          const mid = (lo + hi) >> 1;
          if (m[mid] < v) lo = mid + 1;
          else hi = mid;
        }
        return Math.min(11520, lo);
      });
    }
    return fbTicks;
  }
  const moonTick = () => (PG.lamaCrashLogic && PG.lamaCrashLogic.MOON_TICK) || 11520;

  // ============================================================= Cache
  const scale = () => (PG.app && PG.app.pixelScale) || 1;
  const wcache = new Map(); // Weltflächen (Gruppe = Welt, w, h)
  const scache = new Map(); // kleine Teile (Vögel, Flugzeuge, Lichter)
  let groups = []; // zuletzt benutzte hinten
  const GROUPS = 2;
  const SMALL_MAX = 300;

  const grp = (world, w, h) => `${world}|${w}|${h}|${scale()}`;
  /** Schlüssel einer Weltfläche (Gruppe = (world, w, h)). */
  const wkey = (world, w, h, ...rest) => `${grp(world, w, h)}#${rest.join("|")}`;
  const groupOf = (key) => key.slice(0, key.indexOf("#"));

  function dropGroup(g) {
    for (const k of [...wcache.keys()]) if (groupOf(k) === g) wcache.delete(k);
  }
  /** Gecachte Weltfläche (höchstens GROUPS Gruppen, die am längsten unbenutzte fliegt). */
  function wcached(key, build) {
    const g = groupOf(key);
    if (!groups.length || groups[groups.length - 1] !== g) {
      const i = groups.indexOf(g);
      if (i >= 0) groups.splice(i, 1);
      else while (groups.length >= GROUPS) dropGroup(groups.shift());
      groups.push(g);
    }
    let c = wcache.get(key);
    if (c === undefined) {
      c = build();
      wcache.set(key, c);
    }
    return c;
  }
  /** Gecachtes kleines Teil (ab SMALL_MAX Einträgen wird gemeinsam verworfen). */
  function scached(key, build) {
    const k = key + "|" + scale();
    let c = scache.get(k);
    if (c === undefined) {
      if (scache.size >= SMALL_MAX) scache.clear();
      c = build();
      scache.set(k, c);
    }
    return c;
  }
  function clearCache() {
    wcache.clear();
    scache.clear();
    groups = [];
    buf = null;
    for (const k of Object.keys(scratches)) delete scratches[k];
  }

  const wclamp = (world) => Math.max(0, Math.min(5, trunc(world)));
  /** Text-Schlüssel des Weltnamens (lcx.world_<name>). */
  const nameKey = (world) => "lcx.world_" + NAMES[wclamp(world)];

  // ============================================================= Helfer
  /** Offscreen-Canvas w x h (logisch) in echter Pixeldichte; c.lw/c.lh = logische Größe. */
  function canvas(w, h) {
    const s = scale();
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    const c = ui.makeCanvas(Math.max(1, Math.ceil(w * s)), Math.max(1, Math.ceil(h * s)));
    const g = c.getContext("2d");
    g.scale(s, s);
    c.lw = w;
    c.lh = h;
    return [c, g];
  }
  // wiederverwendete Zwischenflächen (je Zweck eine), logisch w x h, leer
  const scratches = {};
  function scratch(name, w, h) {
    const s = scale();
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    const pw = Math.ceil(w * s), ph = Math.ceil(h * s);
    let sc = scratches[name];
    if (!sc || sc.c.width < pw || sc.c.height < ph) {
      const c = ui.makeCanvas(Math.max(pw, sc ? sc.c.width : 0), Math.max(ph, sc ? sc.c.height : 0));
      sc = scratches[name] = { c, g: c.getContext("2d") };
    }
    const g = sc.g;
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.globalAlpha = 1;
    g.globalCompositeOperation = "source-over";
    g.clearRect(0, 0, pw + 1, ph + 1);
    g.setTransform(s, 0, 0, s, 0, 0);
    sc.pw = pw;
    sc.ph = ph;
    sc.lw = pw / s;
    sc.lh = ph / s;
    return sc;
  }
  /** Fläche (Canvas mit lw/lh oder scratch) zeichnen, alpha 0..255 wie set_alpha. */
  function blit(s, img, x, y, alpha) {
    const src = img.c || img;
    const sw = img.c ? img.pw : src.width, sh = img.c ? img.ph : src.height;
    if (alpha != null && alpha < 255) {
      if (alpha <= 0) return;
      const a = s.globalAlpha;
      s.globalAlpha = a * (alpha / 255);
      s.drawImage(src, 0, 0, sw, sh, x, y, img.lw, img.lh);
      s.globalAlpha = a;
    } else s.drawImage(src, 0, 0, sw, sh, x, y, img.lw, img.lh);
  }
  /** Blit mit senkrechtem Umlauf (kachelbare Ebenen der Höhe H). */
  function wblit(dst, H, img, x, y, alpha) {
    blit(dst, img, x, y, alpha);
    if (y < 0) blit(dst, img, x, y + H, alpha);
    if (y + img.lh > H) blit(dst, img, x, y - H, alpha);
  }

  const css = (c) => ui.col(c);
  /** Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet), auch RGBA. */
  function lerp(c1, c2, num, den) {
    den = Math.max(1, den);
    const out = [];
    for (let i = 0; i < Math.min(c1.length, c2.length); i++) out.push(c1[i] + fdiv((c2[i] - c1[i]) * num, den));
    return out;
  }
  /** Ganzzahl-Streuwert 0..2^32-1 (deterministisch, wie _hash in Python). */
  function hash(i) {
    let v = (Math.imul(i, 2654435761 | 0) + 40503) >>> 0;
    v = Math.imul(v ^ (v >>> 15), 2246822519 | 0) >>> 0;
    return (v ^ (v >>> 13)) >>> 0;
  }
  /** v modulo p im Bereich 0 .. p, auch für negative v (wie Python-%). */
  const wrap = (v, p) => v - p * Math.floor(v / p);
  /** Weiche Wellenlinie -1..1 über die Breite w. */
  function wave(x, w, sd) {
    const a = (TAU * x) / Math.max(1, w);
    return 0.55 * Math.sin(a * 2 + sd[0]) + 0.3 * Math.sin(a * 5 + sd[1]) + 0.15 * Math.sin(a * 11 + sd[2]);
  }

  /** Zeilenfarben wie _grad (ganzzahlig gemischt, RGB oder RGBA). */
  function gradRows(hh, stops) {
    const den = Math.max(1, hh - 1);
    const out = new Array(Math.max(0, hh));
    let k = 0;
    for (let y = 0; y < hh; y++) {
      const p = fdiv(y * 1000, den);
      while (k < stops.length - 2 && p > stops[k + 1][0]) k++;
      const [p0, c0] = stops[k], [p1, c1] = stops[k + 1];
      out[y] = lerp(c0, c1, p - p0, p1 - p0);
    }
    return out;
  }
  /** Senkrechter Verlauf als 1 x hh Pixel-Canvas (gestreckt zeichnen). */
  function gradStrip(hh, stops) {
    hh = Math.max(1, trunc(hh));
    const rows = gradRows(hh, stops);
    const c = ui.makeCanvas(1, hh);
    const g = c.getContext("2d");
    const im = g.createImageData(1, hh);
    for (let y = 0; y < hh; y++) {
      const r = rows[y];
      im.data[y * 4] = r[0];
      im.data[y * 4 + 1] = r[1];
      im.data[y * 4 + 2] = r[2];
      im.data[y * 4 + 3] = r.length > 3 ? r[3] : 255;
    }
    g.putImageData(im, 0, 0);
    c.lw = 1;
    c.lh = hh;
    return c;
  }

  function path(g, pts, close = true) {
    g.beginPath();
    g.moveTo(pts[0][0], pts[0][1]);
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1]);
    if (close) g.closePath();
  }
  function poly(g, col, pts) {
    if (pts.length < 3) return;
    path(g, pts);
    g.fillStyle = css(col);
    g.fill();
  }
  function disc(g, col, cx, cy, r) {
    if (r <= 0) return;
    g.beginPath();
    g.arc(cx, cy, r, 0, TAU);
    g.fillStyle = css(col);
    g.fill();
  }
  /** pygame.draw.circle (Mitte = Pixelmitte). */
  const circle = (g, col, cx, cy, r) => disc(g, col, trunc(cx) + 0.5, trunc(cy) + 0.5, r);
  /** pygame.draw.ellipse mit Rect-Kürzung. */
  function ellipse(g, col, r, width = 0) {
    draw.ellipse(g, col, [trunc(r[0]), trunc(r[1]), trunc(r[2]), trunc(r[3])], width);
  }
  /** pygame.draw.rect mit Rect-Kürzung. */
  function rect(g, col, r, width = 0, radius = 0) {
    draw.rect(g, col, [trunc(r[0]), trunc(r[1]), trunc(r[2]), trunc(r[3])], width, Math.max(0, radius));
  }
  /** Mit Durchsichtig zeichnen (pygame (0,0,0,0)): paint(farbe) wird ausgestanzt. */
  function cut(g, paint) {
    g.save();
    g.globalCompositeOperation = "destination-out";
    paint([0, 0, 0, 255]);
    g.restore();
  }
  /** "Alpha ersetzen": Form ausstanzen, dann mit col neu zeichnen. */
  function replace(g, col, paint) {
    cut(g, paint);
    paint(col);
  }
  /** Surface.fill: Rechteck ersetzen. */
  function fillRep(g, col, x, y, w, h) {
    if (w <= 0 || h <= 0) return;
    g.clearRect(x, y, w, h);
    g.fillStyle = css(col);
    g.fillRect(x, y, w, h);
  }
  /**
   * Blit wie pygame-ce zwischen zwei SRCALPHA-Flächen (Gerätepixel): auf leere Pixel wird
   * kopiert, sonst RGB gemischt, als wäre das Ziel deckend (dC += ((sC - dC) * sA + sC) >> 8),
   * dA = sA + dA - sA * dA / 255. Nur wo das sichtbar anders wirkt als Canvas-"source-over"
   * (halbdurchsichtig auf halbdurchsichtig, z. B. Sonnenaufgang über dem Atmosphärensaum).
   */
  function pgBlit(dst, src, x, y) {
    const ps = scale();
    const x0 = Math.round(x * ps), y0 = Math.round(y * ps);
    const ax = Math.max(0, x0), ay = Math.max(0, y0);
    const bx = Math.min(dst.width, x0 + src.width), by = Math.min(dst.height, y0 + src.height);
    if (bx <= ax || by <= ay) return;
    const dg = dst.getContext("2d");
    const D = dg.getImageData(ax, ay, bx - ax, by - ay);
    const S = src.getContext("2d").getImageData(ax - x0, ay - y0, bx - ax, by - ay);
    const d = D.data, s = S.data;
    for (let i = 0; i < d.length; i += 4) {
      const sa = s[i + 3];
      if (!sa) continue;
      const da = d[i + 3];
      if (!da) {
        d[i] = s[i];
        d[i + 1] = s[i + 1];
        d[i + 2] = s[i + 2];
        d[i + 3] = sa;
        continue;
      }
      d[i] += ((s[i] - d[i]) * sa + s[i]) >> 8;
      d[i + 1] += ((s[i + 1] - d[i + 1]) * sa + s[i + 1]) >> 8;
      d[i + 2] += ((s[i + 2] - d[i + 2]) * sa + s[i + 2]) >> 8;
      d[i + 3] = sa + da - trunc((sa * da) / 255);
    }
    dg.putImageData(D, ax, ay);
  }

  /** Maske (BLEND_RGBA_MIN mit weißem Kreis): nur das Kreisinnere behalten. */
  function keepCircle(g, cx, cy, r) {
    g.save();
    g.globalCompositeOperation = "destination-in";
    disc(g, [0, 0, 0], cx, cy, r);
    g.restore();
  }

  /**
   * Weicher Lichthof (_glow_new) mit Mitte (cx, cy), Radius r: Deckkraft amax in der Mitte,
   * nach außen quadratisch auf 0 (Python: n x n Kreise, weich vergrößert).
   */
  function glowAt(g, cx, cy, r, col, amax) {
    r = Math.max(1, trunc(r));
    const n = Math.max(4, Math.min(40, fdiv(r, 2)));
    const gr = g.createRadialGradient(cx, cy, 0, cx, cy, r);
    const stop = (o, a) => gr.addColorStop(o, `rgba(${col[0]},${col[1]},${col[2]},${a / 255})`);
    stop(0, amax);
    for (let i = 1; i <= n; i++) {
      const q = n - i + 1;
      stop((i - 0.5) / n, fdiv(amax * q * q, n * n));
    }
    stop(1, fdiv(amax, n * n));
    g.fillStyle = gr;
    g.beginPath();
    g.arc(cx, cy, r, 0, TAU);
    g.fill();
  }
  /** _glow_new als Fläche 2r x 2r. */
  function glowCanvas(r, col, amax) {
    r = Math.max(1, trunc(r));
    const [c, g] = canvas(2 * r, 2 * r);
    glowAt(g, r, r, r, col, amax);
    return c;
  }

  /**
   * Kugel-Schatten (_shade) über die Fläche size x size: bis lit ‰ von rad um (lx, ly)
   * hell, dann quadratisch dunkler bis amax bei rad und darüber hinaus.
   */
  function shade(g, size, lx, ly, rad, col, amax, lit = 0, steps = 40) {
    const i0 = fdiv(steps * lit, 1000);
    const den = (steps - i0) * (steps - i0);
    const a = (i) => {
      const q = Math.max(0, i - i0);
      return fdiv(amax * q * q, den);
    };
    const rr = (i) => Math.max(1, fdiv(rad * i, steps));
    const R = rr(steps);
    const gr = g.createRadialGradient(lx, ly, 0, lx, ly, R);
    const stop = (o, al) => gr.addColorStop(Math.max(0, Math.min(1, o / R)), `rgba(${col[0]},${col[1]},${col[2]},${al / 255})`);
    stop(0, a(1));
    let prev = 0;
    for (let i = 1; i <= steps; i++) {
      stop((prev + rr(i)) / 2, a(i));
      prev = rr(i);
    }
    stop(R, amax);
    g.fillStyle = gr;
    g.fillRect(0, 0, size, size);
  }

  /** Glatte Scheibe (2r+2)^2, Mitte (r+1, r+1). */
  function discCanvas(r, col) {
    r = Math.max(1, trunc(r));
    return scached(`disc|${r}|${col}`, () => {
      const [c, g] = canvas(2 * r + 2, 2 * r + 2);
      disc(g, col, r + 1, r + 1, r);
      return c;
    });
  }
  /** Glanzstern (2r+1)^2: weicher Kern mit vier spitzen Strahlen. */
  function sparkleCanvas(r, col) {
    r = Math.max(1, trunc(r));
    return scached(`spark|${r}|${col}`, () => {
      const ss = 4;
      const n = (2 * r + 1) * ss;
      const [c, g] = canvas(2 * r + 1, 2 * r + 1);
      g.scale(1 / ss, 1 / ss);
      const cc = fdiv(n, 2);
      glowAt(g, cc, cc, Math.max(2, fdiv(r * ss * 3, 4)), col, 110);
      const R = r * ss, th = Math.max(1, fdiv(ss * 3, 4));
      replace(g, [col[0], col[1], col[2], 190], (cl) => {
        g.beginPath();
        g.moveTo(cc - R, cc); g.lineTo(cc, cc - th); g.lineTo(cc + R, cc); g.lineTo(cc, cc + th); g.closePath();
        g.moveTo(cc, cc - R); g.lineTo(cc + th, cc); g.lineTo(cc, cc + R); g.lineTo(cc - th, cc); g.closePath();
        g.fillStyle = css(cl);
        g.fill();
      });
      disc(g, [255, 255, 255, 255], cc, cc, Math.max(2, ss));
      return c;
    });
  }
  /** Positionslicht: Lichthof (2r)^2 mit hellem Kern. */
  function lightCanvas(r, col) {
    r = Math.max(2, trunc(r));
    return scached(`light|${r}|${col}`, () => {
      const [c, g] = canvas(2 * r, 2 * r);
      glowAt(g, r, r, r, col, 150);
      disc(g, [Math.min(255, col[0] + 60), Math.min(255, col[1] + 60), Math.min(255, col[2] + 60)], r, r, Math.max(1, fdiv(r, 4)));
      return c;
    });
  }

  /** Lama-Sprite m x m wie llama_sprite (2-fach gezeichnet), Kasten (x, y), flip = Spiegeln. */
  function llamaAt(g, x, y, m, pose, scarf, flip) {
    g.save();
    g.translate(x, y);
    if (flip) {
      g.translate(m, 0);
      g.scale(-1, 1);
    }
    g.scale(0.5, 0.5);
    LCD().drawLlama(g, m * 2, pose, scarf);
    g.restore();
  }

  // ============================================================= Steigen
  /** Steighöhe in Pixeln zur Höhe alt (gebrochener Takt), sanftes Anfahren. */
  function risePx(alt, h) {
    const a = Math.max(0.0, Number(alt) || 0);
    const d = a < EASE ? (a * a) / (2 * EASE) : a - EASE / 2;
    return (d * h * RISE) / 10000;
  }
  /** Höhe des Bodenstreifens (GROUND % der Bühnenhöhe). */
  const groundH = (h) => Math.max(8, fdiv(trunc(h) * GROUND, 100));
  /** Deck-Oberkante (Korbboden) im Bodenstreifen, von seiner Oberkante aus. */
  function padTop(h) {
    const gh = groundH(h);
    return Math.max(fdiv(gh, 3), gh - Math.max(16, fdiv(h * 5, 100)));
  }

  // ============================================================= Himmel
  function sun(g, w, h, world) {
    const cx = fdiv(w * 16, 100), cy = fdiv(h * 17, 100);
    const rg = Math.max(8, fdiv(h * 36, 100));
    glowAt(g, cx, cy, rg, [255, 244, 212], world === 0 ? 120 : 100);
    const r = Math.max(3, fdiv(h * 40, 1000));
    glowAt(g, cx, cy, r * 3, [255, 250, 232], 210);
    disc(g, [255, 252, 238], cx, cy, r);
  }

  /** Himmelsverlauf der Welt (deckend), Wiese/Wolken mit Sonne. */
  function sky(world, w, h) {
    world = wclamp(world);
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    return wcached(wkey(world, w, h, "sky"), () => {
      const [c, g] = canvas(w, h);
      g.drawImage(gradStrip(h, SKY[world]), 0, 0, w, h);
      if (world <= 1) sun(g, w, h, world);
      return c;
    });
  }

  // ============================================================= Wolken
  /** Haufenwolke cw x ch (2-fach überabgetastet) in die Zwischenfläche "cloud". */
  function cloud(cw, ch, seed, pal, storm) {
    const ss = 2;
    const W = Math.max(4, cw * ss), H = Math.max(4, ch * ss);
    const sc = scratch("cloud", cw, ch);
    const g = sc.g;
    g.scale(Math.max(1, cw) / W, Math.max(1, ch) / H);
    const [light, body, sh] = pal;
    const base = fdiv(H * 82, 100);
    const n = storm ? 7 + (hash(seed) % 5) : 4 + (hash(seed) % 4);
    const puffs = [];
    for (let k = 0; k < n; k++) {
      const v = hash(seed * 16 + k + 1);
      const u = fdiv((2 * k + 1) * 1000, 2 * n) + (v % 80) - 40;
      const bell = Math.max(0, 1000 - Math.abs(u - 500) * 2);
      const r = storm
        ? Math.max(2, fdiv(H * (150 + fdiv(bell * 170, 1000) + ((v >>> 9) % 70)), 1000))
        : Math.max(2, fdiv(H * (170 + fdiv(bell * 250, 1000) + ((v >>> 9) % 60)), 1000));
      const x = fdiv(W * 14, 100) + fdiv(fdiv(W * 72, 100) * u, 1000);
      puffs.push([x, base - fdiv(r * 45, 100), r]);
    }
    if (storm) {
      for (let j = 0; j < 2 + (hash(seed + 5) % 2); j++) {
        const v = hash(seed * 16 + 40 + j);
        const r = Math.max(2, fdiv(H * (200 + ((v >>> 9) % 80)), 1000));
        puffs.push([fdiv(W * (32 + (v % 36)), 100), Math.max(r + 1, base - fdiv(H * (40 + ((v >>> 20) % 12)), 100)), r]);
      }
    }
    const [lx, ly] = storm ? [0, -20] : [-10, -17];
    for (const [x, y, r] of puffs) circle(g, sh, x, y, r);
    const x0 = puffs[0][0], x1 = puffs[n - 1][0];
    const rb = Math.max(2, Math.min(puffs[0][2], puffs[n - 1][2]));
    draw.rect(g, sh, [x0 - fdiv(rb, 2), base - rb, x1 - x0 + rb, rb], 0, fdiv(rb, 2));
    for (const [x, y, r] of puffs) circle(g, light, x + fdiv(r * lx, 100), y + fdiv(r * ly, 100), fdiv(r * 84, 100));
    for (const [x, y, r] of puffs) circle(g, body, x + fdiv(r * (lx + 8), 100), y + fdiv(r * (ly + 9), 100), fdiv(r * 80, 100));
    g.clearRect(0, base, W, H - base);
    sc.lw = Math.max(1, cw);
    sc.lh = Math.max(1, ch);
    return sc;
  }

  /** Regenvorhang unter einer Gewitterwolke: schräge Schlieren, nach unten auslaufend. */
  function rainShaft(sw, sh, seed, col) {
    const W = Math.max(2, sw), H = Math.max(2, sh);
    const sc = scratch("shaft", W, H);
    const g = sc.g;
    const n = Math.max(2, fdiv(sw, 3));
    for (let k = 0; k < n; k++) {
      const v = hash(seed * 64 + k);
      const x = fdiv(sw * k, Math.max(1, fdiv(sw, 3))) + (v % 3);
      draw.line(g, [col[0], col[1], col[2], 40 + ((v >>> 8) % 90)], [x, 0], [x - fdiv(sh * 18, 100), sh], 1 + ((v >>> 16) % 2));
    }
    g.save();
    g.globalCompositeOperation = "destination-in";
    const gr = g.createLinearGradient(0, 0.5, 0, H - 0.5);
    gr.addColorStop(0, "rgba(255,255,255,1)");
    gr.addColorStop(0.5, `rgba(255,255,255,${150 / 255})`);
    gr.addColorStop(1, "rgba(255,255,255,0)");
    g.fillStyle = gr;
    g.fillRect(0, 0, W, H);
    g.restore();
    sc.lw = W;
    sc.lh = H;
    return sc;
  }

  /** Schichtwolken-Bänder quer über die ganze Breite (Gewitter, fern). */
  function stratusOn(s, seed, w, h, col, cnt, alpha) {
    const n = Math.max(1, cnt);
    for (let k = 0; k < n; k++) {
      const v = hash(seed + 71 * k + 3);
      const bh = Math.max(4, fdiv(h * (60 + (v % 50)), 1000));
      const yc = fdiv(k * h, n) + fdiv(((v >>> 8) % 1000) * h, n * 1000);
      const bw = w + bh * 4;
      const sc = scratch("band", bw, bh * 3);
      const m = Math.max(4, fdiv(w, Math.max(1, bh * 2)));
      for (let j = 0; j < m + 3; j++) {
        const u = hash(seed + 900 + k * 97 + j);
        const ew = fdiv(bh * (30 + (u % 30)), 10);
        const eh = fdiv(bh * (12 + ((u >>> 8) % 12)), 10);
        const ex = fdiv(bw * j, m + 2) - fdiv(ew, 2);
        ellipse(sc.g, col, [ex, fdiv(bh * 3, 2) - fdiv(eh, 2) + fdiv((((u >>> 16) % 5) - 2) * bh, 8), ew, eh]);
      }
      sc.lw = bw;
      sc.lh = bh * 3;
      wblit(s, h, sc, -bh * 2, yc - fdiv(bh * 3, 2), alpha);
    }
  }

  /** Federwolke ww x wh: dünne, schräg versetzte Schlieren (2-fach überabgetastet). */
  function wisp(ww, wh, seed, col) {
    const ss = 2;
    const W = Math.max(4, ww * ss), H = Math.max(4, wh * ss);
    const sc = scratch("wisp", ww, wh);
    const g = sc.g;
    g.scale(Math.max(1, ww) / W, Math.max(1, wh) / H);
    const n = 5 + (hash(seed) % 3);
    for (let k = 0; k < n; k++) {
      const v = hash(seed * 8 + k + 1);
      const lw = fdiv(W * (34 + (v % 30)), 100);
      const lh = Math.max(ss, fdiv(H * (16 + ((v >>> 8) % 14)), 100));
      const x = fdiv((W - lw) * k, n - 1);
      let y = fdiv((H - lh) * k, n - 1) + ((v >>> 16) % Math.max(1, fdiv(H, 6))) - fdiv(H, 12);
      y = Math.max(0, Math.min(H - lh, y));
      const a = 110 + ((v >>> 20) % 110);
      replace(g, [col[0], col[1], col[2], fdiv(a, 2)], (cl) => ellipse(g, cl, [x, y, lw, lh]));
      replace(g, [col[0], col[1], col[2], a], (cl) => ellipse(g, cl, [x + fdiv(lw, 6), y + fdiv(lh, 4), fdiv(lw * 2, 3), Math.max(1, fdiv(lh, 2))]));
    }
    sc.lw = Math.max(1, ww);
    sc.lh = Math.max(1, wh);
    return sc;
  }

  /** Wolken einer Ebene (goldener Schnitt bzw. Seiten); Gewitterwolken meist mit Regen. */
  function cloudsOn(s, seed, w, h, spec, storm) {
    const [cnt, wmin, wmax, hr, pal, alpha, sides] = spec;
    const n = Math.max(1, fdiv(cnt * w + fdiv(h, 2), h));
    for (let k = 0; k < n; k++) {
      const v = hash(seed + 31 * k);
      const cw = Math.max(8, fdiv(h * (wmin + (v % (wmax - wmin + 1))), 1000));
      const ch = Math.max(4, fdiv(cw * hr, 100));
      const y = fdiv(k * h, n) + fdiv(((v >>> 10) % 1000) * h, n * 1000);
      let fx;
      if (sides) {
        fx = ((v >>> 20) % 300) - 100;
        if (k % 2) fx = 1000 - fx;
      } else fx = fdiv(((k * 618 + ((v >>> 20) % 200)) % 1000) * 12, 10) - 100;
      const x = fdiv(w * fx, 1000) - fdiv(cw, 2);
      if (storm && (v >>> 28) % 3 !== 0) {
        // Regenvorhang unter der Wolke (hängt mit ihr in der Ebene)
        const sw = fdiv(cw * 55, 100), sh = fdiv(ch * 16, 10);
        wblit(s, h, rainShaft(sw, sh, v % 100000, pal[0]), x + fdiv(cw * 26, 100), y - fdiv(ch, 2) + fdiv(ch * 74, 100));
      }
      wblit(s, h, cloud(cw, ch, v % 100000, pal, storm), x, y - fdiv(ch, 2), alpha);
    }
  }

  /** Federwolken-Schlieren einer Ebene. */
  function wispsOn(s, seed, w, h, col, cnt, alpha) {
    const n = Math.max(1, fdiv(cnt * w + fdiv(h, 2), h));
    for (let k = 0; k < n; k++) {
      const v = hash(seed + 53 * k + 7);
      const ww = Math.max(12, fdiv(h * (420 + (v % 360)), 1000));
      const wh = Math.max(4, fdiv(ww * (7 + ((v >>> 10) % 5)), 100));
      const y = fdiv(k * h, n) + fdiv(((v >>> 14) % 1000) * h, n * 1000);
      const x = fdiv(w * ((k * 618 + ((v >>> 24) % 150)) % 1000), 1000);
      wblit(s, h, wisp(ww, wh, v % 100000, col), x - fdiv(ww, 2), y - fdiv(wh, 2), alpha);
    }
  }

  // ============================================================= Sterne
  /** n Sterne auf die Ebene (Deckkraft amin..amax); big ‰ Glanzsterne, two ‰ Doppelpixel. */
  function starsOn(s, seed, n, w, h, amin, amax, big, two) {
    const rs = Math.max(2, fdiv(h, 190));
    for (let i = 0; i < n; i++) {
      const v = hash(seed + 2 * i);
      const u = hash(seed + 2 * i + 1);
      const x = v % w, y = u % h;
      const col = STAR_COLS[(v >>> 20) % 4];
      const a = amin + ((u >>> 12) % (amax - amin + 1));
      const p = (u >>> 22) % 1000;
      if (p < big) {
        const spr = sparkleCanvas(rs + ((v >>> 26) % 2), col);
        wblit(s, h, spr, x - fdiv(spr.lw, 2), y - fdiv(spr.lh, 2), a);
      } else if (p < big + two) fillRep(s, [col[0], col[1], col[2], fdiv(a * 3, 4)], x, y, 2, 2);
      else fillRep(s, [col[0], col[1], col[2], a], x, y, 1, 1);
    }
  }

  /** Milchstraße (schräges Leuchtband mit Sternenstaub) und zwei Nebel. */
  function milky(s, seed, w, h) {
    const q = 4;
    const y0 = fdiv(h * 62, 100), y1 = fdiv(h * 36, 100), th = fdiv(h * 10, 100);
    const n = Math.max(6, fdiv(w, 36));
    const cols = [[150, 140, 230], [116, 158, 232], [190, 150, 222]];
    // Python zeichnet in Viertel-Auflösung und vergrößert weich - hier direkt
    for (let k = 0; k < n; k++) {
      const v = hash(seed + 7 * k);
      const x = fdiv(w * k, n - 1) + fdiv(((v % 41) - 20) * w, 1000);
      const yc = y0 + fdiv((y1 - y0) * x, w) + fdiv((((v >>> 8) % 61) - 30) * th, 100);
      const r = Math.max(2, fdiv(fdiv(th * [60, 80, 100, 120][(v >>> 16) % 4], 100), q));
      glowAt(s, fdiv(x, q) * q, fdiv(yc, q) * q, r * q, cols[(v >>> 24) % 3], 22 + ((v >>> 4) % 14));
    }
    const neb = [[fdiv(w * 22, 100), fdiv(h * 25, 100), [200, 90, 190], [120, 90, 220]],
      [fdiv(w * 80, 100), fdiv(h * 72, 100), [70, 160, 210], [90, 120, 220]]];
    neb.forEach(([cx, cy, col, col2], j) => {
      for (let k = 0; k < 16; k++) {
        const v = hash(seed + 900 + j * 50 + k);
        const r = Math.max(2, fdiv(fdiv(h * (60 + (v % 60)), 1000), q));
        const dx = fdiv(fdiv((((v >>> 8) % 240) - 120) * h, 1000), q);
        const dy = fdiv(fdiv((((v >>> 16) % 120) - 60) * h, 1000), q);
        glowAt(s, (fdiv(cx, q) + dx) * q, (fdiv(cy, q) + dy) * q, r * q, k % 3 ? col : col2, 9 + ((v >>> 24) % 8));
      }
    });
    const m = fdiv(w * th, 70);
    for (let i = 0; i < m; i++) {
      const v = hash(seed + 5000 + 2 * i);
      const u = hash(seed + 5001 + 2 * i);
      const x = v % w;
      const d = fdiv(((u % 1000) + ((u >>> 10) % 1000) - 1000) * th, 1000);
      const y = y0 + fdiv((y1 - y0) * x, w) + d;
      fillRep(s, [226, 230, 255, 40 + ((u >>> 20) % 130)], x, y, 1, 1);
    }
  }

  // ============================================================= Ebenen
  function buildLayer(world, depth, w, h) {
    const [c, g] = canvas(w, h);
    const seed = 1000 * (world * 3 + depth) + 17;
    if (world === 2 && depth === 0) stratusOn(g, seed + 300, w, h, [50, 54, 74], 3, 150);
    const spec = CLOUDS[`${world},${depth}`];
    if (spec) cloudsOn(g, seed, w, h, spec, world === 2);
    if (world === 0 && depth === 0) wispsOn(g, seed, w, h, [255, 255, 255], 3, 150);
    if (world === 4 && depth === 0) milky(g, seed + 400, w, h);
    if (world === 3 && depth === 2) wispsOn(g, seed, w, h, [120, 150, 210], 2, 56);
    const st = STARS[`${world},${depth}`];
    if (st) {
      const [per, amin, amax, big, two] = st;
      starsOn(g, seed + 100, Math.max(1, fdiv(w * h, per)), w, h, amin, amax, big, two);
    }
    return c;
  }

  /** Kachelbare Ebene der Welt (depth 0 fern .. 2 nah), w x h, durchsichtig. */
  function layer(world, depth, w, h) {
    world = wclamp(world);
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    depth = Math.max(0, Math.min(2, trunc(depth)));
    return wcached(wkey(world, w, h, "layer", depth), () => buildLayer(world, depth, w, h));
  }

  /** Kachelbare Ebene senkrecht um off nach unten verschoben. */
  function tile(s, r, img, off) {
    const lh = img.lh;
    const ps = img.height / img.lh;
    const y = pymod(trunc(off), lh);
    const sw = Math.min(img.width, Math.round(r.w * ps));
    if (y) s.drawImage(img, 0, (lh - y) * ps, sw, y * ps, r.x, r.y, r.w, y);
    s.drawImage(img, 0, 0, sw, (lh - y) * ps, r.x, r.y + y, r.w, lh - y);
  }

  // ============================================================= Boden
  /** Schatten-Ellipse wie pygame (ersetzt den Boden: dort scheint der Himmel durch). */
  const shadowEllipse = (s, col, r) => replace(s, col, (cl) => ellipse(s, cl, r));

  function barn(s, x0, yb, bw) {
    const bh = fdiv(bw * 50, 100);
    const rf = fdiv(bw * 42, 100);
    const yw = yb - bh, yk = yb - bh - fdiv(rf * 55, 100), yr = yb - bh - rf;
    const d = fdiv(bw * 20, 100), dy = fdiv(bw * 6, 100);
    const red = [190, 60, 50], redD = [158, 46, 40], redS = [138, 40, 36];
    const trim = [246, 242, 232], roof = [96, 70, 64], roofD = [70, 50, 48];
    const lw = Math.max(1, fdiv(bw, 55));
    const xk0 = x0 + fdiv(bw * 14, 100), xm = x0 + fdiv(bw, 2), xk1 = x0 + fdiv(bw * 86, 100);
    const gable = (x) => {
      if (x <= xk0) return yw + fdiv((yk - yw) * (x - x0), Math.max(1, xk0 - x0));
      if (x <= xm) return yk + fdiv((yr - yk) * (x - xk0), Math.max(1, xm - xk0));
      if (x <= xk1) return yr + fdiv((yk - yr) * (x - xm), Math.max(1, xk1 - xm));
      return yk + fdiv((yw - yk) * (x - xk1), Math.max(1, x0 + bw - xk1));
    };
    shadowEllipse(s, [40, 96, 40, 90], [x0 - fdiv(bw, 10), yb - fdiv(bw, 16), fdiv(bw * 13, 10) + d, fdiv(bw, 8)]);
    // Seitenwand und Dach nach hinten rechts
    poly(s, redS, [[x0 + bw, yb], [x0 + bw, yw], [x0 + bw + d, yw - dy], [x0 + bw + d, yb - dy]]);
    poly(s, roofD, [[xk1, yk], [x0 + bw, yw], [x0 + bw + d, yw - dy], [xk1 + d, yk - dy]]);
    poly(s, roof, [[xm, yr], [xk1, yk], [xk1 + d, yk - dy], [xm + d, yr - dy]]);
    const wx = x0 + bw + fdiv(d * 3, 10), wy = yw + fdiv(bh * 3, 10), ws = Math.max(2, fdiv(d * 4, 10));
    rect(s, [60, 36, 32], [wx, wy - fdiv(dy, 2), ws, ws]);
    rect(s, trim, [wx, wy - fdiv(dy, 2), ws, ws], lw);
    // Giebelwand
    poly(s, red, [[x0, yb], [x0, yw], [xk0, yk], [xm, yr], [xk1, yk], [x0 + bw, yw], [x0 + bw, yb]]);
    for (let k = 1; k < 12; k++) {
      const x = x0 + fdiv(bw * k, 12);
      draw.line(s, redD, [x, gable(x) + lw * 2], [x, yb], lw);
    }
    // Dachkante + Zierleiste, Ecken
    const edge = [[x0 - fdiv(bw * 5, 100), yw + lw * 3], [xk0 - lw, yk - lw * 2], [xm, yr - lw * 3], [xk1 + lw, yk - lw * 2],
      [x0 + fdiv(bw * 105, 100), yw + lw * 3]];
    draw.lines(s, roof, false, edge, Math.max(2, fdiv(bw, 14)));
    draw.lines(s, trim, false, [[x0, yw], [xk0, yk], [xm, yr], [xk1, yk], [x0 + bw, yw]], lw * 2);
    draw.line(s, trim, [x0 + lw, yw], [x0 + lw, yb], lw * 2);
    draw.line(s, trim, [x0 + bw - lw, yw], [x0 + bw - lw, yb], lw * 2);
    // Tor mit weißem X
    const dw = fdiv(bw * 40, 100), dh = fdiv(bh * 72, 100);
    const dx = x0 + fdiv(bw - dw, 2);
    rect(s, redD, [dx, yb - dh, dw, dh]);
    const h2 = fdiv(dw, 2);
    for (const [a, b] of [[[dx, yb - dh], [dx + h2, yb]], [[dx + h2, yb - dh], [dx, yb]], [[dx + h2, yb - dh], [dx + dw, yb]], [[dx + dw, yb - dh], [dx + h2, yb]]]) {
      draw.line(s, trim, a, b, lw * 2);
    }
    draw.line(s, trim, [dx + h2, yb - dh], [dx + h2, yb], lw * 2);
    rect(s, trim, [dx, yb - dh, dw, dh], lw * 2);
    // Heuluke mit Heu
    const hs = fdiv(bw * 22, 100);
    const hx = x0 + fdiv(bw - hs, 2), hy = yw - fdiv(rf * 40, 100);
    rect(s, [66, 38, 30], [hx, hy, hs, hs]);
    poly(s, [236, 200, 92], [[hx + lw, hy + hs], [hx + fdiv(hs, 5), hy + fdiv(hs * 55, 100)], [hx + fdiv(hs * 2, 5), hy + fdiv(hs * 75, 100)],
      [hx + fdiv(hs * 3, 5), hy + fdiv(hs * 50, 100)], [hx + fdiv(hs * 4, 5), hy + fdiv(hs * 72, 100)], [hx + hs - lw, hy + hs]]);
    rect(s, trim, [hx, hy, hs, hs], lw * 2);
    // Wetterfahne
    draw.line(s, [70, 64, 70], [xm, yr - lw * 3], [xm, yr - fdiv(rf * 40, 100)], lw);
    poly(s, [70, 64, 70], [[xm - fdiv(bw, 14), yr - fdiv(rf * 34, 100)], [xm + fdiv(bw, 10), yr - fdiv(rf * 36, 100)], [xm + fdiv(bw, 14), yr - fdiv(rf * 30, 100)]]);
  }

  function bale(s, cx, cy, r, ss) {
    shadowEllipse(s, [40, 96, 40, 90], [cx - fdiv(r * 11, 10), cy + fdiv(r * 6, 10), fdiv(r * 22, 10), fdiv(r, 2)]);
    circle(s, [204, 164, 74], cx, cy, r);
    circle(s, [234, 200, 106], cx - fdiv(r, 10), cy - fdiv(r, 10), fdiv(r * 86, 100));
    for (const k of [64, 42, 20]) {
      draw.circle(s, [198, 158, 70], [cx - fdiv(r, 10) + 0.5, cy - fdiv(r, 10) + 0.5], Math.max(1, fdiv(r * k, 100)), Math.max(1, ss));
    }
  }

  function tree(s, x, yb, size, ss) {
    const tw = Math.max(ss, fdiv(size, 9));
    shadowEllipse(s, [40, 96, 40, 90], [x - fdiv(size, 2), yb - fdiv(size, 18), size, fdiv(size, 9)]);
    rect(s, [112, 80, 54], [x - fdiv(tw, 2), yb - fdiv(size * 55, 100), tw, fdiv(size * 55, 100)]);
    const sh = [52, 118, 60], body = [72, 148, 70], light = [112, 182, 92];
    for (const [dx, dy, r, col] of [[-20, -60, 28, sh], [20, -58, 28, sh], [0, -80, 32, sh], [-14, -66, 24, body], [14, -64, 23, body],
      [-2, -84, 26, body], [-12, -88, 13, light], [-22, -70, 10, light]]) {
      circle(s, col, x + fdiv(size * dx, 100), yb + fdiv(size * dy, 100), Math.max(1, fdiv(size * r, 100)));
    }
  }

  function windsock(s, x, yb, hh, ss) {
    const top = yb - hh;
    draw.line(s, [196, 198, 206], [x, yb], [x, top], Math.max(ss, fdiv(hh, 45)));
    const ln = fdiv(hh * 38, 100);
    const h0 = fdiv(hh * 15, 100), h1 = fdiv(hh * 8, 100);
    for (let k = 0; k < 4; k++) {
      const xa = x + fdiv(ln * k, 4), xb = x + fdiv(ln * (k + 1), 4);
      const ha = h0 + fdiv((h1 - h0) * k, 4), hb = h0 + fdiv((h1 - h0) * (k + 1), 4);
      const da = fdiv(hh * 2 * k, 100), db = fdiv(hh * 2 * (k + 1), 100);
      const col = k % 2 === 0 ? [232, 52, 76] : [250, 248, 240];
      poly(s, col, [[xa, top + da - fdiv(ha, 2) + fdiv(h0, 2)], [xb, top + db - fdiv(hb, 2) + fdiv(h0, 2)],
        [xb, top + db + fdiv(hb, 2) + fdiv(h0, 2)], [xa, top + da + fdiv(ha, 2) + fdiv(h0, 2)]]);
    }
    circle(s, [230, 230, 236], x, top, Math.max(ss, fdiv(hh, 28)));
  }

  /** Startplatz: rundes Holzpodest mit Treppe, rotem Startkreis, Wimpeln und Fähnchen. */
  function pad(s, cx, yd, pw, G, ss) {
    const eb = Math.max(2, fdiv(pw * 11, 100));
    const th = Math.max(ss * 2, fdiv(G * 97, 100) - yd - eb);
    const yg = yd + th + eb;
    const wl = [220, 178, 120], wm = [174, 124, 74], wd = [136, 94, 56];
    const lw = Math.max(1, ss);
    shadowEllipse(s, [36, 86, 36, 110], [cx - fdiv(pw * 64, 100), yd + th - fdiv(eb, 2), fdiv(pw * 132, 100), eb * 2 + fdiv(G * 3, 100)]);
    // Treppe rechts (zwei Stufen)
    for (let k = 0; k < 2; k++) {
      const sx = cx + fdiv(pw * 38, 100) + fdiv(k * pw * 11, 100);
      const sy = yd + fdiv(th * (k + 1), 3);
      rect(s, wd, [sx, sy, fdiv(pw * 17, 100), yg - sy]);
      rect(s, wm, [sx, sy, fdiv(pw * 15, 100), yg - sy]);
      rect(s, wl, [sx, sy, fdiv(pw * 15, 100), Math.max(ss, fdiv(G * 15, 1000))]);
    }
    // Zylinder: Unterkante, Seitenfläche mit Brettern, Deck
    ellipse(s, wd, [cx - fdiv(pw, 2), yd + th - eb, pw, 2 * eb]);
    rect(s, wm, [cx - fdiv(pw, 2), yd, pw, th]);
    for (let k = 1; k < 8; k++) {
      const x = cx - fdiv(pw, 2) + fdiv(pw * k, 8);
      const q = ((x - cx) * 2) / pw;
      const e = trunc(eb * Math.sqrt(Math.max(0.0, 1.0 - q * q)));
      draw.line(s, wd, [x, yd + e], [x, yd + th + e], lw);
    }
    ellipse(s, wl, [cx - fdiv(pw, 2), yd - eb, pw, 2 * eb]);
    ellipse(s, wd, [cx - fdiv(pw, 2), yd - eb, pw, 2 * eb], lw);
    for (let k = 1; k < 5; k++) {
      const y = yd - eb + fdiv(2 * eb * k, 5);
      const q = (y - yd) / eb;
      const hw = trunc((pw / 2) * Math.sqrt(Math.max(0.0, 1.0 - q * q)));
      draw.line(s, [198, 154, 100], [cx - hw + ss * 2, y], [cx + hw - ss * 2, y], lw);
    }
    ellipse(s, [232, 52, 76], [cx - fdiv(pw * 30, 100), yd - fdiv(eb * 56, 100), fdiv(pw * 60, 100), fdiv(eb * 112, 100)], Math.max(ss, fdiv(pw, 36)));
    // Wimpelkette an der Vorderkante
    const cols = [[232, 52, 76], [250, 206, 70], [250, 248, 240], [70, 140, 220]];
    const tw = Math.max(ss * 2, fdiv(pw * 9, 100));
    for (let k = 0; k < 9; k++) {
      const x = cx - fdiv(pw * 44, 100) + fdiv(pw * 88 * k, 800);
      const q = ((x - cx) * 2) / pw;
      const y = yd + trunc(eb * Math.sqrt(Math.max(0.0, 1.0 - q * q))) + lw;
      poly(s, cols[k % 4], [[x - fdiv(tw, 2), y], [x + fdiv(tw, 2), y], [x, y + fdiv(tw * 11, 10)]]);
    }
    // zwei Fähnchen am hinteren Rand
    for (const [fx, col] of [[-44, [232, 52, 76]], [44, [250, 206, 70]]]) {
      const x = cx + fdiv(pw * fx, 100);
      const top = yd - fdiv(G * 30, 100);
      draw.line(s, [236, 236, 240], [x, yd - fdiv(eb, 3)], [x, top], Math.max(ss, fdiv(pw, 50)));
      const fw = fdiv(pw * 16, 100);
      poly(s, col, [[x, top], [x + fw, top + fdiv(fw * 3, 10)], [x, top + fdiv(fw * 6, 10)]]);
    }
  }

  /** Alles auf dem Bodenstreifen (W x G, ss-fach überabgetastet). */
  function meadow(s, W, G, ss, h) {
    const Y = (p) => fdiv(G * p, 1000);
    const step = Math.max(4, fdiv(W, 140));
    // Hügel: dunstig fern, grüner davor; Baumgruppen auf dem vorderen Kamm
    for (const [base, amp, col, sd] of [[440, 250, [158, 200, 192], [0.6, 1.9, 0.3]], [500, 160, [120, 182, 126], [2.2, 0.4, 1.3]]]) {
      const pts = [[0, G]];
      for (let x = 0; x < W + step; x += step) pts.push([x, Y(base) - trunc(Y(amp) * (0.5 + 0.5 * wave(x, W, sd)))]);
      pts.push([W + step, G]);
      poly(s, col, pts);
    }
    const nTrees = Math.max(4, fdiv(W, Math.max(1, Y(120))));
    for (let k = 0; k < nTrees; k++) {
      const v = hash(71000 + k);
      const x = fdiv(k * W, nTrees) + (v % Math.max(1, fdiv(W, nTrees)));
      const y = Y(500) - trunc(Y(160) * (0.5 + 0.5 * wave(x, W, [2.2, 0.4, 1.3]))) + Y(10);
      for (let j = 0; j < 1 + ((v >>> 8) % 3); j++) {
        const r = Math.max(ss, fdiv(G * (22 + ((v >>> (12 + j)) % 14)), 1000));
        const tx = x + fdiv(j * r * 15, 10);
        circle(s, [70, 136, 82], tx, y - r, r);
        circle(s, [92, 160, 96], tx - fdiv(r, 4), y - r - fdiv(r, 4), fdiv(r * 6, 10));
      }
    }
    // Wiese: Verlauf, oben leicht gewellt, mit hellem Saum
    const top = [];
    for (let x = 0; x < W + step; x += step) top.push([x, Y(505) - trunc(Y(26) * (0.5 + 0.5 * wave(x, W, [1.0, 2.6, 0.2])))]);
    const y0 = Y(470);
    s.save();
    path(s, [[0, G]].concat(top, [[W + step, G]]));
    s.clip();
    s.drawImage(gradStrip(G - y0, [[0, [146, 208, 104, 255]], [450, [110, 184, 82, 255]], [1000, [82, 154, 62, 255]]]), 0, y0, W, G - y0);
    s.restore();
    draw.lines(s, [176, 226, 128], false, top, Math.max(1, ss));
    // Grasbüschel und Blumen (vorn größer)
    const gy0 = Y(540);
    const n = fdiv(W * (G - gy0), ss * ss * 230);
    const flowers = [[250, 250, 240], [250, 220, 80], [240, 150, 180], [190, 160, 240]];
    for (let i = 0; i < n; i++) {
      const v = hash(72000 + 2 * i);
      const u = hash(72001 + 2 * i);
      const x = v % W;
      const y = gy0 + fdiv((u % 1000) * (G - gy0), 1000);
      const sz = ss * (2 + fdiv((y - gy0) * 5, Math.max(1, G - gy0)));
      if ((v >>> 20) % 7 === 0) circle(s, flowers[(u >>> 12) % 4], x, y - fdiv(sz, 2), Math.max(1, fdiv(sz, 3)));
      else {
        const col = (u >>> 14) & 1 ? [78, 146, 58] : [100, 170, 72];
        s.beginPath();
        for (const dx of [-fdiv(sz, 2), 0, fdiv(sz, 2)]) {
          s.moveTo(x + 0.5, y + 0.5);
          s.lineTo(x + dx + 0.5, y - sz + 0.5);
        }
        s.strokeStyle = css(col);
        s.lineWidth = Math.max(1, fdiv(ss, 2));
        s.lineCap = "butt";
        s.stroke();
      }
    }
    const pw = fdiv(h * PAD_W, 100) * ss;
    const cx = fdiv(fdiv(W, ss) * PAD_X, 100) * ss, yd = padTop(h) * ss;
    const free = cx + fdiv(W, 10); // bis hier bleibt es um den Startplatz frei
    const bw = Math.min(fdiv(W * 18, 100), fdiv(G * 62, 100));
    const bx = fdiv(W * 71, 100);
    // Trampelpfad vom Startplatz zum Stalltor
    const p0 = cx + fdiv(pw * 60, 100), p1 = bx + fdiv(bw, 2);
    const topPts = [], botPts = [];
    for (let k = 0; k < 9; k++) {
      const x = p0 + fdiv((p1 - p0) * k, 8);
      const y = Y(935) - fdiv((Y(935) - Y(872)) * k * k, 64);
      const half = Y(34) - fdiv((Y(34) - Y(14)) * k, 8);
      topPts.push([x, y - half]);
      botPts.push([x, y + half]);
    }
    poly(s, [190, 162, 112], topPts.concat(botPts.slice().reverse()));
    draw.lines(s, [210, 184, 134], false, topPts.map(([x, y]) => [x, y + ss]), Math.max(1, ss));
    // Zaun quer über die Weide, erst rechts vom Startplatz
    const rh = Math.max(ss, Y(20));
    for (const yr of [Y(588), Y(650)]) {
      s.fillStyle = css([190, 142, 92]);
      s.fillRect(free, yr, W - free, rh);
      s.fillStyle = css([140, 100, 62]);
      s.fillRect(free, yr + rh, W - free, Math.max(1, ss));
    }
    const sp = Math.max(8, Y(150));
    const pwid = Math.max(ss, Y(26));
    for (let x = free; x < W; x += sp) {
      rect(s, [160, 114, 72], [x, Y(552), pwid, Y(702) - Y(552)]);
      rect(s, [122, 86, 54], [x + fdiv(pwid * 6, 10), Y(552), pwid - fdiv(pwid * 6, 10), Y(702) - Y(552)]);
      poly(s, [176, 128, 82], [[x, Y(552)], [x + fdiv(pwid, 2), Y(534)], [x + pwid, Y(552)]]);
    }
    // Büsche, kleiner Baum in der Mitte, Windsack
    for (const [fx, fy, fr] of [[47, 880, 44], [51, 905, 32], [89, 935, 40], [64, 915, 28]]) {
      const ux = fdiv(W * fx, 100), uy = Y(fy), ur = Math.max(ss, fdiv(G * fr, 1000));
      circle(s, [58, 124, 62], ux, uy - ur, ur);
      circle(s, [84, 156, 74], ux - fdiv(ur, 4), uy - ur - fdiv(ur, 4), fdiv(ur * 7, 10));
    }
    tree(s, fdiv(W * 55, 100), Y(810), fdiv(G * 40, 100), ss);
    windsock(s, fdiv(W * 31, 100), Y(900), fdiv(G * 50, 100), ss);
    // Stall rechts (71 .. 90 %), Heuballen davor, großer Baum ganz rechts
    const r = Math.max(ss * 2, fdiv(G * 6, 100));
    bale(s, bx - fdiv(r * 16, 10), Y(900), r, ss);
    bale(s, bx - fdiv(r * 38, 10), Y(925), fdiv(r * 9, 10), ss);
    barn(s, bx, Y(875), bw);
    tree(s, fdiv(W * 95, 100), Y(860), fdiv(G * 60, 100), ss);
    // Startplatz und Lamas (sehen dem Start zu)
    pad(s, cx, yd, pw, G, ss);
    const nl = Math.max(8, fdiv(G * 24, 100));
    for (const [fx, fy, k, scarf, flip] of [[36, 925, 100, [60, 150, 220], true], [43, 960, 86, [250, 196, 60], true], [60, 945, 92, [150, 90, 200], false]]) {
      const m = Math.max(8, fdiv(nl * k, 100));
      const x = fdiv(W * fx, 100), y = Y(fy);
      shadowEllipse(s, [36, 86, 36, 100], [x - fdiv(m * 3, 10), y - fdiv(m, 16), fdiv(m * 6, 10), fdiv(m, 8)]);
      llamaAt(s, x - fdiv(m, 2), y - fdiv(m * 94, 100), m, "stand", scarf, flip);
    }
  }

  /**
   * Bodenstreifen w x groundH(h) (2-fach überabgetastet): Hügel mit Bäumen, Weide, Zaun,
   * Stall, Heuballen, Windsack, drei Lamas und links der Startplatz.
   */
  function ground(w, h) {
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    return wcached(wkey(0, w, h, "ground"), () => {
      const gh = groundH(h);
      const ss = 2;
      const [c, g] = canvas(w, gh);
      g.scale(1 / ss, 1 / ss);
      meadow(g, w * ss, gh * ss, ss, h);
      return c;
    });
  }

  // ============================================================= kleine Figuren
  /** Vogel n x n (Flügelschlag frame 0..3), Blick nach rechts (flip = links). */
  function bird(n, frame, col, flip) {
    frame = pymod(frame, 4);
    return scached(`bird|${n}|${frame}|${col}|${flip ? 1 : 0}`, () => {
      const ss = 4;
      const N = n * ss;
      const [c, g] = canvas(n, n);
      if (flip) {
        g.translate(n, 0);
        g.scale(-1, 1);
      }
      g.scale(1 / ss, 1 / ss);
      const tip = [0.1, 0.4, 0.74, 0.4][frame];
      const elb = [0.3, 0.44, 0.6, 0.46][frame];
      const cx = N * 0.52, cy = N * 0.54;
      const lw = Math.max(ss, fdiv(N, 9));
      draw.lines(g, col, false, [[N * 0.03, N * tip], [N * 0.28, N * elb], [cx, cy]], lw);
      draw.lines(g, col, false, [[cx, cy], [N * 0.76, N * elb], [N * 0.97, N * tip]], lw);
      ellipse(g, col, [N * 0.38, N * 0.48, N * 0.3, N * 0.13]);
      disc(g, col, N * 0.7, N * 0.52, N * 0.055);
      return c;
    });
  }

  /** Verkehrsflugzeug von der Seite, n lang (Blick nach rechts; flip = links). */
  function plane(n, flip) {
    return scached(`plane|${n}|${flip ? 1 : 0}`, () => {
      const ss = 3;
      const N = n * ss;
      const H = fdiv(N * 38, 100);
      const [c, g] = canvas(n, Math.max(2, fdiv(H, ss)));
      if (flip) {
        g.translate(n, 0);
        g.scale(-1, 1);
      }
      g.scale(n / N, Math.max(2, fdiv(H, ss)) / H);
      const body = [170, 176, 194], belly = [126, 132, 152], dark = [84, 90, 108];
      const fy = fdiv(H * 44, 100), fh = fdiv(H * 26, 100);
      const P = (a, b) => [fdiv(N * a, 100), fdiv(H * b, 100)];
      poly(g, dark, [P(1, 52), P(16, 52), P(19, 60), [0, fdiv(H * 62, 100)]]);
      poly(g, [184, 56, 74], [P(4, 52), [0, fdiv(H * 4, 100)], P(9, 4), P(25, 50)]);
      rect(g, body, [fdiv(N * 4, 100), fy, fdiv(N * 86, 100), fh], 0, fdiv(fh, 2));
      ellipse(g, body, [fdiv(N * 80, 100), fy, fdiv(N * 19, 100), fh]);
      rect(g, belly, [fdiv(N * 10, 100), fy + fdiv(fh * 62, 100), fdiv(N * 82, 100), fdiv(fh * 38, 100)], 0, fdiv(fh, 3));
      poly(g, [40, 52, 82], [[fdiv(N * 90, 100), fy + fdiv(fh * 22, 100)], [fdiv(N * 95, 100), fy + fdiv(fh * 30, 100)],
        [fdiv(N * 95, 100), fy + fdiv(fh * 44, 100)], [fdiv(N * 89, 100), fy + fdiv(fh * 44, 100)]]);
      for (let k = 0; k < 17; k++) {
        circle(g, [255, 226, 150], fdiv(N * (26 + fdiv(k * 37, 10)), 100), fy + fdiv(fh * 38, 100), Math.max(1, fdiv(ss * 2, 3)));
      }
      poly(g, dark, [[fdiv(N * 40, 100), fy + fdiv(fh * 70, 100)], [fdiv(N * 58, 100), fy + fdiv(fh * 70, 100)], [fdiv(N * 44, 100), H - 1], [fdiv(N * 36, 100), H - 1]]);
      ellipse(g, [70, 74, 92], [fdiv(N * 45, 100), fy + fh, fdiv(N * 13, 100), fdiv(H * 15, 100)]);
      return c;
    });
  }

  /** Schlanker Jet als dunkle Silhouette, n lang, Blick nach rechts. */
  function jet(n, flip) {
    return scached(`jet|${n}|${flip ? 1 : 0}`, () => {
      const ss = 3;
      const N = n * ss;
      const H = fdiv(N * 34, 100);
      const [c, g] = canvas(n, Math.max(2, fdiv(H, ss)));
      if (flip) {
        g.translate(n, 0);
        g.scale(-1, 1);
      }
      g.scale(n / N, Math.max(2, fdiv(H, ss)) / H);
      const col = [92, 102, 136], top = [176, 196, 224];
      const fy = fdiv(H * 46, 100), fh = fdiv(H * 18, 100);
      poly(g, col, [[fdiv(N * 4, 100), fdiv(H * 50, 100)], [0, fdiv(H * 6, 100)], [fdiv(N * 10, 100), fdiv(H * 6, 100)], [fdiv(N * 26, 100), fdiv(H * 48, 100)]]);
      rect(g, col, [fdiv(N * 4, 100), fy, fdiv(N * 80, 100), fh], 0, fdiv(fh, 2));
      poly(g, col, [[fdiv(N * 82, 100), fy], [N, fy + fdiv(fh * 60, 100)], [fdiv(N * 82, 100), fy + fh]]);
      poly(g, col, [[fdiv(N * 42, 100), fy + fdiv(fh * 60, 100)], [fdiv(N * 60, 100), fy + fdiv(fh * 60, 100)], [fdiv(N * 38, 100), H - 1], [fdiv(N * 30, 100), H - 1]]);
      draw.line(g, top, [fdiv(N * 10, 100), fy + fdiv(ss, 2)], [fdiv(N * 84, 100), fy + fdiv(ss, 2)], Math.max(1, fdiv(ss, 2)));
      return c;
    });
  }

  /** Kondensstreifen n x hgt: am Flugzeug schmal und hell, nach hinten breiter und blasser. */
  function contrail(n, hgt, flip) {
    return scached(`trail|${n}|${hgt}|${flip ? 1 : 0}`, () => {
      const ss = 2;
      const N = n * ss, Hh = Math.max(2, hgt * ss);
      // ganzzahlige Hilfsfläche (Segmente überlappen und ersetzen sich wie in pygame)
      const big = ui.makeCanvas(N, Hh);
      const bg = big.getContext("2d");
      const segs = 48;
      for (let k = 0; k < segs; k++) {
        const f = k / (segs - 1);
        const th = Math.max(1, trunc(Hh * (1.0 - 0.7 * f)));
        const a = trunc(8 + 190 * f * f);
        fillRep(bg, [206, 222, 246, a], fdiv(N * k, segs), fdiv(Hh - th, 2), fdiv(N, segs) + 1, th);
      }
      const [c, g] = canvas(n, Math.max(1, hgt));
      if (flip) {
        g.translate(n, 0);
        g.scale(-1, 1);
      }
      g.imageSmoothingQuality = "high";
      g.drawImage(big, 0, 0, n, Math.max(1, hgt));
      return c;
    });
  }

  /** Größe wie pygame.transform.rotate (Quadrat der Kante size, Winkel deg). */
  function rotSize(size, deg) {
    const a = (deg * PI) / 180;
    const ca = Math.cos(a), sa = Math.sin(a);
    const cx = ca * size, sy = sa * size;
    return trunc(Math.max(Math.abs(cx + sy), Math.abs(cx - sy)));
  }

  /** Satellit (goldener Körper, zwei Solarflügel, Schüssel), schräg gedreht. */
  function satellite(n) {
    return scached(`sat|${n}`, () => {
      const ss = 3;
      const N = n * ss;
      const R = rotSize(N, 24);
      const fin = Math.max(1, fdiv(R, ss));
      const [c, g] = canvas(fin, fin);
      g.translate(fin / 2, fin / 2);
      g.rotate((-24 * PI) / 180);
      g.scale(fin / R, fin / R);
      g.translate(-N / 2, -N / 2);
      const cc = fdiv(N, 2);
      const pw = fdiv(N * 30, 100), ph = fdiv(N * 13, 100);
      const bw = fdiv(N * 18, 100), bh = fdiv(N * 22, 100);
      const lw = Math.max(1, fdiv(ss, 2));
      draw.line(g, [150, 154, 166], [cc - fdiv(bw, 2) - fdiv(pw, 2), cc], [cc + fdiv(bw, 2) + fdiv(pw, 2), cc], Math.max(1, ss));
      for (const side of [-1, 1]) {
        const x0 = side > 0 ? cc + fdiv(bw, 2) + fdiv(N * 4, 100) : cc - fdiv(bw, 2) - fdiv(N * 4, 100) - pw;
        rect(g, [34, 58, 132], [x0, cc - fdiv(ph, 2), pw, ph]);
        for (let k = 1; k < 4; k++) {
          const xx = x0 + fdiv(pw * k, 4);
          draw.line(g, [88, 120, 200], [xx, cc - fdiv(ph, 2)], [xx, cc + fdiv(ph, 2)], lw);
        }
        draw.line(g, [88, 120, 200], [x0, cc], [x0 + pw, cc], lw);
        rect(g, [130, 150, 200], [x0, cc - fdiv(ph, 2), pw, ph], lw);
      }
      rect(g, [196, 156, 70], [cc - fdiv(bw, 2), cc - fdiv(bh, 2), bw, bh], 0, ss);
      rect(g, [238, 206, 118], [cc - fdiv(bw, 2), cc - fdiv(bh, 2), bw, fdiv(bh, 2)], 0, ss);
      draw.line(g, [200, 202, 210], [cc, cc - fdiv(bh, 2)], [cc, cc - bh], lw);
      ellipse(g, [226, 228, 236], [cc - fdiv(bw, 2), cc - bh - fdiv(bh, 3), bw, fdiv(bh, 2)]);
      return c;
    });
  }

  /** Blitz (Variante 0..3): gezackter Hauptast mit zwei Seitenästen, violetter Schein. */
  function bolt(w, h, v4) {
    v4 = pymod(v4, 4);
    return wcached(wkey(2, w, h, "bolt", v4), () => {
      const bw = Math.max(8, fdiv(h * 30, 100)), bh = Math.max(16, fdiv(h * 55, 100));
      const ss = 2;
      const BW = bw * ss, BH = bh * ss;
      const [c, g] = canvas(bw, bh);
      g.scale(1 / ss, 1 / ss);
      const pts = [[fdiv(BW, 2), 0]];
      let x = fdiv(BW, 2), y = 0, k = 0;
      while (y < fdiv(BH * 90, 100)) {
        const v = hash(21000 + v4 * 100 + k);
        y += fdiv(BH * (5 + (v % 6)), 100);
        x = Math.max(fdiv(BW, 6), Math.min(fdiv(BW * 5, 6), x + fdiv(BW * (((v >>> 8) % 25) - 12), 100)));
        pts.push([x, y]);
        k++;
      }
      const branches = [];
      [2, 4].forEach((start, j) => {
        if (start >= pts.length) return;
        let [bx, by] = pts[start];
        const side = hash(21500 + v4 * 10 + j) & 1 ? 1 : -1;
        const br = [[bx, by]];
        for (let m = 0; m < 4; m++) {
          const v = hash(21600 + v4 * 40 + j * 8 + m);
          bx = Math.max(0, Math.min(BW - 1, bx + fdiv(side * BW * (6 + (v % 8)), 100)));
          by += fdiv(BH * (4 + ((v >>> 8) % 4)), 100);
          br.push([bx, by]);
        }
        branches.push(br);
      });
      const gg = Math.max(2, fdiv(BH, 70));
      for (const [col, wid] of [[[150, 140, 255, 50], gg * 3], [[186, 178, 255, 110], fdiv(gg * 3, 2)], [[255, 255, 255, 255], Math.max(ss, fdiv(gg, 2))]]) {
        replace(g, col, (cl) => {
          draw.lines(g, cl, false, pts, wid);
          for (const br of branches) draw.lines(g, cl, false, br, Math.max(1, fdiv(wid * 2, 3)));
        });
      }
      return c;
    });
  }

  /** Lichtschein in der Wolke um den Blitz. */
  const boltGlow = (w, h) => wcached(wkey(2, w, h, "boltglow"), () => glowCanvas(Math.max(8, fdiv(h * 34, 100)), [160, 160, 236], 120));

  // ============================================================= Polarlicht
  /** [Strahlenhöhe, Schein darunter] eines Polarlicht-Vorhangs. */
  const ribbonH = (h, kind) => [Math.max(8, fdiv(h * (kind === 0 ? 44 : 38), 100)), Math.max(4, fdiv(h * 12, 100))];
  /** Streifenbreite, in der das Polarlicht gewellt gezeichnet wird. */
  const sliceW = (w) => (w >= 400 ? 8 : 4);

  /** Polarlicht-Vorhang (w + Streifen) x (rh + gh), waagerecht periodisch über w. */
  function ribbon(w, h, kind) {
    return wcached(wkey(3, w, h, "ribbon", kind), () => {
      const [rh, gh] = ribbonH(h, kind);
      const sl = sliceW(w);
      const [c, g] = canvas(w + sl, rh + gh);
      const gstops = kind === 0
        ? [[0, [120, 250, 176, 120]], [220, [90, 226, 160, 52]], [1000, [50, 160, 130, 0]]]
        : [[0, [226, 128, 230, 100]], [220, [180, 96, 214, 40]], [1000, [120, 60, 190, 0]]];
      const glow = gradStrip(gh, gstops);
      const stops = kind === 0
        ? [[0, [170, 80, 210, 0]], [380, [150, 110, 220, 40]], [700, [70, 230, 150, 140]], [880, [140, 255, 186, 225]], [950, [215, 255, 230, 240]], [1000, [130, 252, 180, 150]]]
        : [[0, [110, 50, 200, 0]], [450, [150, 70, 210, 50]], [800, [210, 100, 222, 135]], [930, [250, 160, 236, 200]], [1000, [232, 134, 232, 120]]];
      const prof = gradStrip(rh, stops);
      const rw = Math.max(2, fdiv(w, 200));
      const p = [kind * 1.7 + 0.3, kind * 0.9 + 2.1, kind * 2.3 + 1.1, kind * 1.3 + 0.7];
      for (let x = 0; x < w; x += rw) {
        const v = hash(51000 + kind * 4000 + fdiv(x, rw));
        const a = (TAU * x) / w;
        let i = 0.5 + 0.4 * Math.sin(a * 2 + p[0]) + 0.18 * Math.sin(a * 7 + p[1]) + 0.1 * Math.sin(a * 17 + p[2]);
        i = Math.max(0.0, Math.min(1.0, i)) * (0.74 + (v % 26) / 100);
        const hf = 0.56 + 0.3 * (0.5 + 0.5 * Math.sin(a * 5 + p[3])) + ((v >>> 8) % 14) / 100;
        const hh = Math.max(2, trunc(rh * Math.min(1.0, hf)));
        const jit = (v >>> 16) % Math.max(1, fdiv(h, 90));
        g.globalAlpha = trunc(255 * i) / 255;
        g.drawImage(glow, x, rh - jit, rw, gh);
        g.globalAlpha = Math.max(0, Math.min(255, trunc(12 + 243 * i))) / 255;
        g.drawImage(prof, x, rh - hh - jit, rw, hh);
      }
      g.globalAlpha = 1;
      // die ersten sl Spalten hinten anhängen (Umlauf)
      const ps = scale();
      const tmp = ui.makeCanvas(Math.ceil(sl * ps), c.height);
      tmp.getContext("2d").drawImage(c, 0, 0);
      g.save();
      g.setTransform(1, 0, 0, 1, 0, 0);
      g.clearRect(Math.round(w * ps), 0, c.width, c.height);
      g.drawImage(tmp, Math.round(w * ps), 0);
      g.restore();
      return c;
    });
  }

  /** Zwei wogende Polarlicht-Bögen (violett hinten, pulsierend; grün vorn), wandernd. */
  function aurora(s, r, t) {
    const w = r.w, h = r.h;
    const sl = sliceW(w);
    const ps = scale();
    for (const kind of [1, 0]) {
      const rib = ribbon(w, h, kind);
      const rh = ribbonH(h, kind)[0], th = rib.lh;
      const top = fdiv(rh * 11, 100);
      const [base, arc, peak, a1, l1, o1, a2, l2, o2, drift] = ARCS[kind];
      const al = s.globalAlpha;
      if (kind === 1) s.globalAlpha = al * (trunc(195 + 60 * Math.sin(t * 0.8 + 2.4)) / 255);
      const by = r.y + fdiv(h * base, 100) - rh + top;
      const dx = t * w * drift;
      const A1 = (h * a1) / 1000, A2 = (h * a2) / 1000, B = (h * arc) / 1000;
      const px = (w * peak) / 1000, span = w * 0.62;
      for (let x = 0; x < w; x += sl) {
        const u = (x - px) / span;
        const yo = B * u * u - B + A1 * Math.sin((TAU * x) / (w * l1) + t * o1) + A2 * Math.sin((TAU * x) / (w * l2) - t * o2 + kind);
        const sx = trunc(wrap(x + dx, w));
        s.drawImage(rib, sx * ps, top * ps, sl * ps, (th - top) * ps, r.x + x, by + trunc(yo), sl, th - top);
      }
      s.globalAlpha = al;
    }
  }

  // ============================================================= Weltall
  const moonR1 = (h) => Math.max(6, fdiv(h * 34, 100));

  /** Mondscheibe d x d: Meere, Krater, Kugelschatten (Licht oben links), 2-fach. */
  function moonDisc(g, x0, y0, d, seed) {
    const ss = 2;
    const D = d * ss;
    const C = fdiv(D, 2), R = fdiv(D, 2) - ss;
    const [c, b] = canvas(d, d);
    b.scale(1 / ss, 1 / ss);
    disc(b, [214, 214, 220], C + 0.5, C + 0.5, R);
    for (let k = 0; k < 7; k++) {
      const v = hash(seed + k);
      const ang = (v % 628) / 100;
      const rr = (R * ((v >>> 10) % 55)) / 100;
      const mx = C + Math.cos(ang) * rr, my = C + Math.sin(ang) * rr;
      for (let j = 0; j < 4; j++) {
        const u = hash(seed + 100 + k * 8 + j);
        disc(b, [170, 172, 188], mx + (R * ((u % 31) - 15)) / 100, my + (R * (((u >>> 8) % 31) - 15)) / 100, (R * (8 + ((u >>> 16) % 14))) / 100);
      }
    }
    for (let k = 0; k < 64; k++) {
      const v = hash(seed + 500 + k);
      const ang = (v % 628) / 100;
      const rr = R * 0.92 * Math.sqrt(((v >>> 10) % 1000) / 1000);
      const cr = (R * (1.5 + ((v >>> 20) % 5) + (k < 3 ? 8 : k < 10 ? 3 : 0))) / 100;
      const x = C + Math.cos(ang) * rr, y = C + Math.sin(ang) * rr;
      disc(b, [234, 234, 240], x - cr * 0.16, y - cr * 0.16, cr);
      disc(b, [158, 160, 176], x + cr * 0.06, y + cr * 0.06, cr * 0.86);
      disc(b, [196, 198, 208], x + cr * 0.24, y + cr * 0.24, cr * 0.6);
    }
    shade(b, D, trunc(C - R * 0.38), trunc(C - R * 0.38), trunc(R * 1.56), [16, 18, 38], 190, 200);
    keepCircle(b, C + 0.5, C + 0.5, R);
    g.drawImage(c, x0, y0, d, d);
  }

  /** Mond mit Lichthof in der größten Weltall-Größe (kleinere werden davon verkleinert). */
  function moonMaster(w, h) {
    return wcached(wkey(4, w, h, "moon"), () => {
      const R = moonR1(h);
      const M = fdiv(R * 27, 10);
      const [c, g] = canvas(M, M);
      glowAt(g, fdiv(M, 2), fdiv(M, 2), fdiv(M, 2), [176, 190, 236], 64);
      moonDisc(g, fdiv(M, 2) - R, fdiv(M, 2) - R, 2 * R, 61000);
      return c;
    });
  }

  /** [cx, cy, r] des Monds: winzig oben rechts zu Beginn des Weltalls, zum Mond-Takt groß. */
  function moonGeom(w, h, alt) {
    const t4 = worldTicks()[4], tm = moonTick();
    const f = Math.max(0.0, Math.min(1.0, (alt - t4) / Math.max(1, tm - t4)));
    const r0 = Math.max(3.0, h * 0.018), r1 = moonR1(h);
    const r = r0 * (r1 / r0) ** (f ** 1.4);
    const k = (r - r0) / Math.max(1.0, r1 - r0);
    return [w * (0.8 - 0.3 * k), h * (0.2 - 0.17 * k), r];
  }

  /** Weich verkleinern (halbieren, bis höchstens 2-fach übrig, dann hochwertig). */
  function shrink(src, size) {
    const ps = scale();
    const want = Math.max(1, Math.round(size * ps));
    let cur = src;
    while (cur.width > want * 2) {
      const nw = Math.max(want, Math.ceil(cur.width / 2));
      const nc = ui.makeCanvas(nw, nw);
      const ng = nc.getContext("2d");
      ng.imageSmoothingQuality = "high";
      ng.drawImage(cur, 0, 0, nw, nw);
      cur = nc;
    }
    const [c, g] = canvas(size, size);
    g.imageSmoothingQuality = "high";
    g.drawImage(cur, 0, 0, size, size);
    return c;
  }

  /** Wachsender Mond; nur die zuletzt gebrauchte Größe bleibt gecacht. */
  function moon(s, r, alt) {
    const w = r.w, h = r.h;
    const [cx, cy, rr] = moonGeom(w, h, alt);
    const R = moonR1(h);
    const master = moonMaster(w, h);
    const size = Math.max(4, fdiv(trunc((master.lw * rr) / R), 2) * 2);
    const key = wkey(4, w, h, "moon_now");
    let img = wcached(key, () => (size === master.lw ? master : shrink(master, size)));
    if (img.lw !== size) {
      img = size === master.lw ? master : shrink(master, size);
      wcache.set(key, img);
    }
    s.drawImage(img, r.x + trunc(cx) - fdiv(size, 2), r.y + trunc(cy) - fdiv(size, 2), size, size);
  }

  /** Erdkrümmung (w x 56 % h): Nachtseite mit Stadtlichtern, blauer Saum, Sonnenaufgang. */
  function earth(w, h) {
    return wcached(wkey(4, w, h, "earth"), () => {
      const eh = Math.max(8, fdiv(h * 56, 100));
      const [c, s] = canvas(w, eh);
      const g = Math.max(4, fdiv(h * 12, 100));
      const R = fdiv(Math.max(w, h) * 13, 10);
      const cx = fdiv(w, 2), cy = g + R;
      const limb = (x) => trunc(cy - Math.sqrt(Math.max(0, R * R - (x - cx) * (x - cx))));
      const band = gradStrip(g, [[0, [40, 90, 220, 0]], [640, [70, 140, 255, 56]], [1000, [156, 214, 255, 200]]]);
      for (let x = 0; x < w; x += 2) s.drawImage(band, x, limb(x) - g, 2, g);
      disc(s, [8, 16, 40], cx + 0.5, cy + 0.5, R);
      const sb = Math.max(3, fdiv(h * 9, 100));
      const inner = gradStrip(sb, [[0, [60, 124, 214, 255]], [300, [26, 56, 116, 200]], [1000, [8, 16, 40, 0]]]);
      for (let x = 0; x < w; x += 2) s.drawImage(inner, x, limb(x), 2, sb);
      const dot = h >= 600 ? 2 : 1;
      for (let k = 0; k < Math.max(6, fdiv(w, 40)); k++) {
        const v = hash(66000 + k);
        const x = v % w;
        const y = limb(x) + fdiv(sb * 6, 10) + ((v >>> 12) % Math.max(1, eh - limb(x) - fdiv(sb * 6, 10)));
        for (let j = 0; j < 4 + ((v >>> 24) % 10); j++) {
          const u = hash(66500 + k * 16 + j);
          const px = x + fdiv(((u % 41) - 20) * h, 1000);
          const py = y + fdiv(((((u >>> 8) % 21) - 10) * h), 1000);
          if (py > limb(px) + fdiv(sb, 2)) fillRep(s, [255, 206, 120, 120 + ((u >>> 16) % 120)], px, py, dot, dot);
        }
      }
      const pts = [];
      for (let x = 0; x < w + 1; x += Math.max(2, fdiv(w, 200))) pts.push([x, limb(x)]);
      draw.lines(s, [176, 224, 255], false, pts, Math.max(1, fdiv(h, 320)));
      // Sonnenaufgang: Lichthof + Glanzstern über dem halbdurchsichtigen Saum (pygame-Mischung)
      const sx = fdiv(w * 86, 100);
      const gr = Math.max(4, fdiv(h * 22, 100));
      pgBlit(c, glowCanvas(gr, [255, 196, 120], 150), sx - gr, limb(sx) - gr);
      const sp = sparkleCanvas(Math.max(3, fdiv(h, 60)), [255, 236, 200]);
      pgBlit(c, sp, sx - fdiv(sp.lw, 2), limb(sx) - fdiv(sp.lh, 2));
      return c;
    });
  }

  /** Erde zu Beginn des Weltalls: Kante bei 58 % von h, sinkt dann nach unten weg. */
  function earthDraw(s, r, alt) {
    const w = r.w, h = r.h;
    const t4 = worldTicks()[4];
    const drop = Math.max(-h * 0.08, (risePx(alt, h) - risePx(t4, h)) * EARTH_P);
    const y = r.y + fdiv(h * 58, 100) - Math.max(4, fdiv(h * 12, 100)) + trunc(drop);
    if (y < r.bottom) {
      const img = earth(w, h);
      s.drawImage(img, r.x, y, img.lw, img.lh);
    }
  }

  /** Ferner Planet (Mitte = Flächenmitte): 0 Ringplanet, 1 roter Planet, 2 Eisriese. */
  function planet(w, h, kind) {
    return wcached(wkey(4, w, h, "planet", kind), () => {
      const r = Math.max(3, fdiv(h * PLANETS[kind][2], 1000));
      const ss = 3;
      const R = r * ss;
      const size = kind === 0 ? R * 5 : R * 2 + 4 * ss;
      const C = fdiv(size, 2);
      // Scheibe in Überabtastung
      const [dc, d] = canvas(size / ss, size / ss);
      d.scale(1 / ss, 1 / ss);
      if (kind === 0) {
        const bands = [[228, 198, 144], [206, 166, 114], [238, 216, 170], [192, 150, 102], [224, 192, 138]];
        for (let k = 0; k < 11; k++) {
          d.fillStyle = css(bands[k % 5]);
          d.fillRect(0, C - R + fdiv(2 * R * k, 11), size, fdiv(2 * R, 11) + 1);
        }
      } else if (kind === 1) {
        d.fillStyle = css([198, 102, 70]);
        d.fillRect(0, 0, size, size);
        for (let k = 0; k < 6; k++) {
          const v = hash(67000 + k);
          circle(d, [154, 72, 52], C + fdiv(R * ((v % 120) - 60), 100), C + fdiv(R * (((v >>> 8) % 120) - 60), 100), fdiv(R * (12 + ((v >>> 16) % 20)), 100));
        }
        ellipse(d, [244, 236, 228], [C - fdiv(R * 45, 100), C - R - fdiv(R, 10), fdiv(R * 90, 100), fdiv(R * 36, 100)]);
      } else {
        d.fillStyle = css([128, 200, 220]);
        d.fillRect(0, 0, size, size);
        for (let k = 0; k < 5; k++) {
          d.fillStyle = css([112, 184, 210]);
          d.fillRect(0, C - R + fdiv(2 * R * (2 * k + 1), 10), size, Math.max(1, fdiv(R, 9)));
        }
      }
      shade(d, size, C - fdiv(R * 4, 10), C - fdiv(R * 4, 10), fdiv(R * 16, 10), [8, 8, 24], 220);
      keepCircle(d, C + 0.5, C + 0.5, R);
      const out = kind === 0 ? rotSize(size, 16) : size;
      const fin = Math.max(1, fdiv(out, ss));
      const [c, g] = canvas(fin, fin);
      g.translate(fin / 2, fin / 2);
      if (kind === 0) g.rotate((-16 * PI) / 180);
      g.scale(fin / out, fin / out);
      g.translate(-size / 2, -size / 2);
      const rings = (lower) => {
        g.save();
        g.beginPath();
        if (lower) g.rect(0, C, size, size - C);
        else g.rect(0, 0, size, C);
        g.clip();
        ellipse(g, [214, 194, 150, 210], [C - fdiv(R * 23, 10), C - fdiv(R * 55, 100), fdiv(R * 46, 10), fdiv(R * 110, 100)], Math.max(ss, fdiv(R, 5)));
        ellipse(g, [176, 156, 118, 160], [C - fdiv(R * 18, 10), C - fdiv(R * 42, 100), fdiv(R * 36, 10), fdiv(R * 84, 100)], Math.max(ss, fdiv(R, 9)));
        g.restore();
      };
      if (kind === 0) rings(false);
      g.drawImage(dc, 0, 0, size, size);
      if (kind === 0) rings(true);
      return c;
    });
  }

  /** Ferne Planeten ziehen ganz langsam von oben nach unten durchs Bild. */
  function planets(s, r, rise) {
    const w = r.w, h = r.h;
    const t4 = worldTicks()[4];
    PLANETS.forEach(([start, fx, fr], kind) => {
      const rr = Math.max(3, fdiv(h * fr, 1000));
      const cy = r.y - rr * 2 + (rise - risePx(t4 + start, h)) * PLANET_P;
      if (cy < r.y - rr * 3 || cy > r.bottom + rr * 3) return;
      const img = planet(w, h, kind);
      s.drawImage(img, r.x + fdiv(w * fx, 1000) - fdiv(img.lw, 2), trunc(cy) - fdiv(img.lh, 2), img.lw, img.lh);
    });
  }

  // ============================================================= belebte Details
  /** Plätze i >= 0 einer Höhenreihe im Abstand gap, die gerade sichtbar sein können. */
  function slots(off, h, gap) {
    const top = fdiv(h * TOP, 100) + off;
    const a = Math.max(0, trunc(Math.floor((top - 2 * h) / gap)));
    const b = trunc(Math.floor((top + h) / gap)) + 2;
    const out = [];
    for (let i = a; i < b; i++) out.push(i);
    return out;
  }
  /** x eines Dings, das waagerecht durchs Bild zieht (d = +1 nach rechts, -1 nach links). */
  function cross(r, v, d, sp, t, m) {
    const P = r.w + fdiv(r.w, 2) + 2 * m;
    const u = wrap(((v >>> 4) % P) + sp * t, P);
    return d > 0 ? r.x - m + u : r.right + m - u;
  }

  /** Vogelschwärme: Wiese kleine lose Grüppchen, Wolken Gänse im V. */
  function flocks(s, r, world, t, rise) {
    const w = r.w, h = r.h;
    const n = Math.max(5, fdiv(h * (world === 0 ? 18 : 24), 1000));
    const gap = fdiv(h * (world === 0 ? 50 : 56), 100);
    const off = rise * PARALLAX[1];
    const col = [46, 52, 74];
    const margin = n * 8;
    for (const i of slots(off, h, gap)) {
      const v = hash(9000 + world * 500 + i);
      if (v % 100 >= 80) continue;
      const y = r.y + trunc(fdiv(h * TOP, 100) + off - i * gap - ((v >>> 8) % Math.max(1, fdiv(gap, 3))));
      if (y < r.y - margin || y > r.bottom + margin) continue;
      const d = (v >>> 7) & 1 ? 1 : -1;
      const x = cross(r, v, d, (h * (34 + ((v >>> 9) % 30))) / 1000, t, margin);
      const nb = 3 + ((v >>> 20) % (world === 0 ? 3 : 5));
      const rate = world === 0 ? 7.0 : 4.2;
      for (let k = 0; k < nb; k++) {
        let bx, by;
        if (world === 0) {
          const u = hash(9400 + i * 8 + k);
          bx = x - fdiv(d * k * n * 13, 10) + fdiv(((u % 9) - 4) * n, 6);
          by = y + fdiv((((u >>> 8) % 13) - 6) * n, 6);
        } else {
          const rank = fdiv(k + 1, 2), side = k % 2 ? 1 : -1;
          bx = x - fdiv(d * rank * n * 14, 10);
          by = y + fdiv(side * rank * n * 6, 10);
        }
        const fr = pymod(trunc(t * rate + k * 1.37 + i), 4);
        const img = bird(n, fr, col, d < 0);
        s.drawImage(img, trunc(bx) - fdiv(n, 2), trunc(by) - fdiv(n, 2), n, n);
      }
    }
  }

  /** Blinklichter eines Flugzeugs (Positionslicht, rotes Beacon, weißer Doppel-Strobe). */
  function lights(s, x, y, iw, ih, d, t, ph, r, spots) {
    const at = (p) => {
      const px = d > 0 ? p[0] : 1000 - p[0];
      return [x + fdiv(iw * px, 1000), y + fdiv(ih * p[1], 1000)];
    };
    const nav = d > 0 ? [255, 70, 70] : [80, 255, 120];
    const q = wrap(t * 1.1 + ph, 1.0);
    const st = wrap(t * 0.8 + ph * 0.7, 1.0);
    const items = [[spots[2], nav, r]];
    if (q < 0.16) items.push([spots[0], [255, 60, 60], r + 1]);
    if (st < 0.05 || (st > 0.11 && st < 0.16)) items.push([spots[1], [255, 255, 255], r + 2]);
    for (const [p, col, rr] of items) {
      const [lx, ly] = at(p);
      const img = lightCanvas(rr, col);
      s.drawImage(img, trunc(lx) - Math.max(2, rr), trunc(ly) - Math.max(2, rr), img.lw, img.lh);
    }
  }

  /** Kleine Verkehrsflugzeuge mit leuchtender Fensterreihe und Blinklichtern. */
  function planes(s, r, t, rise) {
    const w = r.w, h = r.h;
    const n = Math.max(14, fdiv(h * 9, 100));
    const gap = fdiv(h * 60, 100);
    const off = rise * PARALLAX[1];
    const rr = Math.max(2, fdiv(n, 16));
    for (const i of slots(off, h, gap)) {
      const v = hash(12000 + i);
      if (v % 100 >= 72) continue;
      const y = r.y + trunc(fdiv(h * TOP, 100) + off - i * gap - ((v >>> 8) % Math.max(1, fdiv(gap, 3))));
      if (y < r.y - n || y > r.bottom + n) continue;
      const d = (v >>> 6) & 1 ? 1 : -1;
      const x = cross(r, v, d, (w * (55 + ((v >>> 16) % 35))) / 1000, t, n);
      const img = plane(n, d < 0);
      const ix = trunc(x) - fdiv(n, 2), iy = y - fdiv(img.lh, 2);
      s.drawImage(img, ix, iy, img.lw, img.lh);
      lights(s, ix, iy, n, img.lh, d, t, (v >>> 24) / 256, rr, [[520, 420], [20, 60], [370, 980]]);
    }
  }

  /** Jets mit Kondensstreifen unter dem Polarlicht. */
  function jets(s, r, t, rise) {
    const w = r.w, h = r.h;
    const n = Math.max(12, fdiv(h * 6, 100));
    const gap = fdiv(h * 65, 100);
    const off = rise * PARALLAX[1];
    const cl = Math.max(16, fdiv(w * 45, 100));
    const ch = Math.max(2, fdiv(h * 12, 1000));
    const rr = Math.max(2, fdiv(n, 14));
    for (const i of slots(off, h, gap)) {
      const v = hash(14000 + i);
      if (v % 100 >= 75) continue;
      const y = r.y + trunc(fdiv(h * TOP, 100) + off - i * gap - ((v >>> 8) % Math.max(1, fdiv(gap, 3))));
      if (y < r.y - n || y > r.bottom + n) continue;
      const d = (v >>> 6) & 1 ? 1 : -1;
      const x = cross(r, v, d, (w * (100 + ((v >>> 16) % 50))) / 1000, t, n + cl);
      const img = jet(n, d < 0);
      const ix = trunc(x) - fdiv(n, 2), iy = y - fdiv(img.lh, 2);
      const tx = d > 0 ? ix - cl + fdiv(n, 10) : ix + n - fdiv(n, 10);
      const tr = contrail(cl, ch, d < 0);
      s.drawImage(tr, tx, y + fdiv(img.lh, 10) - fdiv(ch, 2), tr.lw, tr.lh);
      s.drawImage(img, ix, iy, img.lw, img.lh);
      lights(s, ix, iy, n, img.lh, d, t, (v >>> 24) / 256, rr, [[540, 440], [30, 80], [340, 980]]);
    }
  }

  /** [Stärke 0..1, x ‰, y ‰, Variante] des Blitzes zur Zeit t oder null. */
  function flash(t) {
    const tt = Math.max(0.0, t);
    const k = Math.floor(tt / FLASH);
    const v = hash(20011 + (k % 1000000));
    if (v % 100 >= 60) return null;
    const d = tt - k * FLASH - (((v >>> 8) % 1000) * (FLASH - 0.5)) / 1000;
    if (d < 0 || d >= 0.45) return null;
    let f;
    if (d < 0.05) f = 1.0;
    else if (d < 0.09) f = 0.3;
    else if (d < 0.16) f = 0.9;
    else f = (0.9 * (0.45 - d)) / 0.29;
    return [f, 150 + ((v >>> 12) % 700), 40 + ((v >>> 22) % 260), (v >>> 4) % 4];
  }

  /** Blitz: hinten Schein + Blitzast in den Wolken, vorn heller Aufschein (additiv). */
  function lightning(s, r, t, front) {
    const fl = flash(t);
    if (!fl) return;
    const [f, fx, fy, v4] = fl;
    const w = r.w, h = r.h;
    if (front) {
      const g = trunc(34 * f);
      if (g > 0) {
        const op = s.globalCompositeOperation;
        s.globalCompositeOperation = "lighter";
        s.fillStyle = `rgb(${g},${g},${g + fdiv(g, 3)})`;
        s.fillRect(r.x, r.y, w, h);
        s.globalCompositeOperation = op;
      }
      return;
    }
    const x = r.x + fdiv(w * fx, 1000), y = r.y + fdiv(h * fy, 1000);
    const gl = boltGlow(w, h);
    blit(s, gl, x - fdiv(gl.lw, 2), y - fdiv(gl.lh, 2), trunc(255 * f));
    if (f > 0.5) {
      const b = bolt(w, h, v4);
      blit(s, b, x - fdiv(b.lw, 2), y, Math.min(255, trunc(300 * f)));
    }
  }

  /** Schräger Regen: nahe Tropfen länger, heller, schneller. */
  function rain(s, r, t, rise) {
    const w = r.w, h = r.h;
    const n = fdiv(w * h, 4200);
    const ln = Math.max(5, fdiv(h * 5, 100));
    const thick = h >= 600 ? 2 : 1;
    const nearP = new Path2D(), farP = new Path2D();
    for (let i = 0; i < n; i++) {
      const v = hash(30011 + i);
      const near = v % 3 === 0;
      const sp = ((h * (130 + ((v >>> 8) % 70))) / 100) * (near ? 1.4 : 1.0);
      const y = wrap((((v >>> 4) % 1000) * (h + ln)) / 1000 + sp * t + rise * 0.6, h + ln) - ln;
      const ll = fdiv(ln * (near ? 13 : 8), 10);
      const x = ((v >>> 14) % (w + fdiv(w, 3))) - fdiv(w, 8) - y * 0.22;
      const x0 = r.x + trunc(x), y0 = r.y + trunc(y);
      const p = near ? nearP : farP;
      const o = near && thick === 2 ? 1 : 0.5;
      p.moveTo(x0 + o, y0 + o);
      p.lineTo(x0 - fdiv(ll * 22, 100) + o, y0 + ll + o);
    }
    s.lineCap = "butt";
    s.strokeStyle = "rgb(136,146,176)";
    s.lineWidth = thick;
    s.stroke(nearP);
    s.strokeStyle = "rgb(92,100,128)";
    s.lineWidth = 1;
    s.stroke(farP);
  }

  /** Funkelnde Sterne (zur mittleren Ebene mitlaufend; Mond: nur am Himmel). */
  function twinkle(s, r, world, t, rise) {
    const w = r.w, h = r.h;
    const n = Math.max(6, fdiv(w * h, 24000));
    const off = rise * PARALLAX[1];
    const rr = Math.max(2, fdiv(h, 150));
    for (let i = 0; i < n; i++) {
      const v = hash(40000 + world * 1000 + i);
      const u = hash(40500 + world * 1000 + i);
      const x = v % w;
      let y;
      if (world === 5) {
        y = u % Math.max(1, fdiv(h * 50, 100));
        if (Math.abs(x - fdiv(w * 76, 100)) < fdiv(h * 16, 100) && Math.abs(y - fdiv(h * 22, 100)) < fdiv(h * 16, 100)) continue;
      } else y = pymod(trunc((u % h) + off), h);
      const b = 0.5 + 0.5 * Math.sin(t * (1.4 + ((v >>> 20) % 30) / 10) + ((v >>> 8) % 628) / 100);
      const al = trunc(255 * b * b);
      if (al < 12) continue;
      const spr = sparkleCanvas(rr + ((u >>> 24) % 2), STAR_COLS[(u >>> 12) % 4]);
      blit(s, spr, r.x + x - fdiv(spr.lw, 2), r.y + y - fdiv(spr.lh, 2), al);
    }
  }

  /** Sternschnuppe: in jedem SHOOT-Fenster mit 55 % eine, 0,75 s lang. */
  function shooting(s, r, t) {
    const tt = Math.max(0.0, t);
    const k = Math.floor(tt / SHOOT);
    const v = hash(41017 + (k % 1000000));
    if (v % 100 >= 55) return;
    const dur = 0.75;
    const d = tt - k * SHOOT - (((v >>> 8) % 1000) * (SHOOT - dur)) / 1000;
    if (!(d >= 0 && d < dur)) return;
    const p = d / dur;
    const w = r.w, h = r.h;
    const x0 = r.x + fdiv(w * (100 + ((v >>> 12) % 800)), 1000);
    const y0 = r.y + fdiv(h * (40 + ((v >>> 20) % 350)), 1000);
    const dx = (v >>> 3) & 1 ? 1 : -1;
    const ang = ((22 + ((v >>> 28) % 16)) * PI) / 180;
    const ux = dx * Math.cos(ang), uy = Math.sin(ang);
    const hx = x0 + ux * h * 0.45 * p, hy = y0 + uy * h * 0.45 * p;
    const tail = h * 0.22 * Math.min(1.0, p * 4);
    const env = Math.sin(PI * p);
    const bg = [4, 6, 20];
    const wid = h >= 400 ? 2 : 1;
    for (let j = 0; j < 5; j++) {
      const c = lerp(bg, [236, 242, 255], trunc(env * (1000 - j * 190)), 1000);
      const f0 = j / 5, f1 = (j + 1) / 5;
      draw.line(s, c, [trunc(hx - ux * tail * f0), trunc(hy - uy * tail * f0)], [trunc(hx - ux * tail * f1), trunc(hy - uy * tail * f1)], j < 3 ? wid : 1);
    }
    const hd = lightCanvas(Math.max(3, fdiv(h, 110)), [220, 230, 255]);
    blit(s, hd, trunc(hx) - fdiv(hd.lw, 2), trunc(hy) - fdiv(hd.lh, 2), trunc(255 * env));
  }

  /** Satelliten ziehen langsam schräg durchs Bild, mit rotem Blinklicht. */
  function satellites(s, r, t, rise) {
    const w = r.w, h = r.h;
    const n = Math.max(10, fdiv(h * 6, 100));
    const gap = fdiv(h * 70, 100);
    const off = rise * SAT_P;
    for (const i of slots(off, h, gap)) {
      const v = hash(43013 + i);
      if (v % 100 >= 80) continue;
      const d = (v >>> 6) & 1 ? 1 : -1;
      const x = trunc(cross(r, v, d, (w * (22 + ((v >>> 16) % 20))) / 1000, t, 2 * n));
      const slope = (((v >>> 24) % 40) - 20) / 100;
      const y = r.y + trunc(fdiv(h * TOP, 100) + off - i * gap - ((v >>> 8) % Math.max(1, fdiv(gap, 3))) + (x - (r.x + fdiv(w, 2))) * slope);
      if (y < r.y - n || y > r.bottom + n) continue;
      const img = satellite(n);
      s.drawImage(img, x - fdiv(img.lw, 2), y - fdiv(img.lh, 2), img.lw, img.lh);
      if (wrap(t + (v >>> 20) / 64, 1.3) < 0.14) {
        const lt = lightCanvas(Math.max(2, fdiv(n, 7)), [255, 70, 70]);
        s.drawImage(lt, x - fdiv(lt.lw, 2), y - fdiv(lt.lh, 2), lt.lw, lt.lh);
      }
    }
  }

  /** Belebtes je Welt; phase 0 = hinter der mittleren Ebene, 1 = hinter der nahen, 2 = vorn. */
  function details(s, r, world, phase, alt, t, rise) {
    if (world === 0 || world === 1) {
      if (phase === 1) flocks(s, r, world, t, rise);
    } else if (world === 2) {
      if (phase === 0) lightning(s, r, t, false);
      else if (phase === 1) planes(s, r, t, rise);
      else {
        rain(s, r, t, rise);
        lightning(s, r, t, true);
      }
    } else if (world === 3) {
      if (phase === 0) aurora(s, r, t);
      else if (phase === 1) jets(s, r, t, rise);
      else twinkle(s, r, 3, t, rise);
    } else if (world === 4 && phase === 2) {
      twinkle(s, r, 4, t, rise);
      shooting(s, r, t);
      planets(s, r, rise);
      moon(s, r, alt);
      earthDraw(s, r, alt);
      satellites(s, r, t, rise);
    }
  }

  // ============================================================= Mond-Landung
  /** Die Erde vom Mond aus: blaue Murmel mit Kontinenten, Wolken, Nachtseite, Saum. */
  function marble(g, cx, cy, r) {
    const ss = 3;
    const R = r * ss;
    const size = R * 3;
    const C = fdiv(size, 2);
    const fin = fdiv(size, ss);
    const [c, s] = canvas(fin, fin);
    s.scale(1 / ss, 1 / ss);
    glowAt(s, C, C, C, [110, 170, 255], 80);
    const [dc, d] = canvas(fin, fin);
    d.scale(1 / ss, 1 / ss);
    disc(d, [34, 92, 198], C + 0.5, C + 0.5, R);
    for (let k = 0; k < 5; k++) {
      const v = hash(64000 + k);
      const ang = (v % 628) / 100, rr = (R * (10 + ((v >>> 10) % 60))) / 100;
      const mx = C + Math.cos(ang) * rr, my = C + Math.sin(ang) * rr;
      for (let j = 0; j < 10; j++) {
        const u = hash(64100 + k * 16 + j);
        const col = (u >>> 24) % 4 === 0 ? [172, 152, 100] : [86, 152, 80];
        disc(d, col, mx + (R * ((u % 51) - 25)) / 100, my + (R * (((u >>> 8) % 41) - 20)) / 100, (R * (4 + ((u >>> 16) % 9))) / 100);
      }
    }
    for (let k = 0; k < 22; k++) {
      const v = hash(64500 + k);
      const ang = (v % 628) / 100, rr = (R * ((v >>> 10) % 92)) / 100;
      const cw = (R * (14 + ((v >>> 18) % 24))) / 100, ch = (R * (3 + ((v >>> 24) % 4))) / 100;
      const er = [C + Math.cos(ang) * rr - cw / 2, C + Math.sin(ang) * rr - ch / 2, cw, ch];
      replace(d, [246, 248, 255, 200], (cl) => ellipse(d, cl, er));
    }
    shade(d, size, trunc(C - R * 0.6), trunc(C - R * 0.25), trunc(R * 1.3), [4, 8, 26], 248, 560);
    keepCircle(d, C + 0.5, C + 0.5, R);
    s.save();
    s.setTransform(1, 0, 0, 1, 0, 0);
    s.drawImage(dc, 0, 0);
    s.restore();
    const arcPaint = (cl) => draw.arc(s, cl, [C - R, C - R, 2 * R, 2 * R], 1.2, 4.3, Math.max(1, ss));
    replace(s, [170, 214, 255, 190], arcPaint);
    g.drawImage(c, cx - fdiv(fin, 2), cy - fdiv(fin, 2), fin, fin);
  }

  /** Krater in Aufsicht schräg: heller Wall, dunkle Mulde, beschienene Innenwand rechts. */
  function crater(s, x, y, cw, ch) {
    ellipse(s, [172, 172, 180], [x - fdiv(cw, 2), y - fdiv(ch, 2) - Math.max(1, fdiv(ch, 8)), cw, ch]);
    ellipse(s, [84, 84, 94], [x - fdiv(cw * 42, 100), y - fdiv(ch * 36, 100), fdiv(cw * 84, 100), fdiv(ch * 76, 100)]);
    ellipse(s, [124, 124, 134], [x - fdiv(cw * 4, 100), y - fdiv(ch * 20, 100), fdiv(cw * 44, 100), fdiv(ch * 52, 100)]);
  }

  /** Mondlandung: Sterne, Erde oben rechts, ferne Berge, Kraterlandschaft mit Landeplatz. */
  function moonScene(w, h) {
    w = Math.max(1, trunc(w));
    h = Math.max(1, trunc(h));
    return wcached(wkey(5, w, h, "scene"), () => {
      const [c, s] = canvas(w, h);
      s.drawImage(gradStrip(h, SKY[5]), 0, 0, w, h);
      const bg = SKY[5][0][1];
      for (let i = 0; i < Math.max(1, fdiv(w * h, 1100)); i++) {
        const v = hash(81000 + 2 * i);
        const u = hash(81001 + 2 * i);
        const col = lerp(bg, STAR_COLS[(v >>> 20) % 4], 40 + ((u >>> 12) % 216), 255);
        const sz = 1 + ((u >>> 24) % 9 === 0 ? 1 : 0);
        s.fillStyle = css(col);
        s.fillRect(v % w, u % fdiv(h * 62, 100), sz, sz);
      }
      const re = Math.max(4, fdiv(h * 10, 100));
      marble(s, fdiv(w * 76, 100), fdiv(h * 22, 100), re);
      const yh = fdiv(h * 60, 100);
      // ferne, runde Mondberge in zwei Reihen (Licht von links: helle Kante)
      const step = Math.max(3, fdiv(w, 160));
      for (const [amp, base, col, rim, sd] of [[70, 10, [96, 96, 108], [140, 140, 152], [0.4, 2.0, 1.1]], [44, 22, [118, 118, 130], [170, 170, 180], [2.6, 0.8, 0.2]]]) {
        const pts = [[0, yh + fdiv(h, 20)]];
        for (let x = 0; x < w + step; x += step) {
          const q = 0.5 + 0.5 * wave(x, w, sd);
          pts.push([x, yh + fdiv(h * base, 1000) - trunc(((h * amp) / 1000) * q * q)]);
        }
        pts.push([w + step, yh + fdiv(h, 20)]);
        poly(s, col, pts);
        draw.lines(s, rim, false, pts.slice(1, -1), Math.max(1, fdiv(h, 320)));
      }
      // Boden: Verlauf, leicht gewölbt
      const RS = w * 2;
      const gh = h - yh + fdiv(h, 20);
      s.save();
      s.beginPath();
      s.arc(fdiv(w, 2) + 0.5, RS + fdiv(h, 20) + yh - fdiv(h, 20) + 0.5, RS, 0, TAU);
      s.clip();
      s.drawImage(gradStrip(gh, [[0, [160, 160, 168, 255]], [400, [134, 134, 144, 255]], [1000, [96, 96, 106, 255]]]), 0, yh - fdiv(h, 20), w, gh);
      s.restore();
      // dunklere Meere (flach in der Perspektive)
      for (let i = 0; i < 5; i++) {
        const v = hash(83500 + i);
        const z = 0.15 + (0.7 * (v % 1000)) / 1000;
        const mw = trunc(w * (0.25 + 0.3 * z)), mh = Math.max(2, trunc(h * 0.04 * (0.5 + z)));
        const mx = (v >>> 10) % w;
        const my = yh + trunc((h - yh) * z * z) + fdiv(h, 40);
        ellipse(s, [124, 124, 134], [mx - fdiv(mw, 2), my - fdiv(mh, 2), mw, mh]);
      }
      // Staub, Krater (hinten klein und flach, vorn groß), Steine
      const dust = [[150, 150, 158], [118, 118, 128], [170, 170, 178]];
      for (let i = 0; i < fdiv(w * (h - yh), 260); i++) {
        const v = hash(83000 + i);
        s.fillStyle = css(dust[(v >>> 24) % 3]);
        s.fillRect(v % w, yh + ((v >>> 12) % Math.max(1, h - yh)), 1, 1);
      }
      for (let i = 0; i < 26; i++) {
        const v = hash(84000 + i);
        const z = (v % 1000) / 1000;
        const y = yh + trunc((h - yh) * (0.06 + 0.9 * z * z));
        const x = fdiv(((v >>> 10) & 0xffff) * w, 65536);
        const cw = trunc(w * (0.03 + 0.13 * z) * (0.6 + ((v >>> 26) % 8) / 10));
        const ch = Math.max(2, trunc(cw * (0.16 + 0.22 * z)));
        if (Math.abs(x - fdiv(w, 2)) < fdiv(w * 16, 100) + fdiv(cw, 2) && y > fdiv(h * 70, 100)) continue;
        crater(s, x, y, cw, ch);
      }
      for (let i = 0; i < 40; i++) {
        const v = hash(85000 + i);
        const z = (v % 1000) / 1000;
        const y = yh + trunc((h - yh) * (0.05 + 0.92 * z));
        const x = (v >>> 10) % w;
        const rr = Math.max(1, trunc(h * (0.002 + 0.012 * z)));
        if (Math.abs(x - fdiv(w, 2)) < fdiv(w * 12, 100) && y > fdiv(h * 72, 100)) continue;
        ellipse(s, [66, 66, 76], [x - rr, y - fdiv(rr, 2), rr * 2, rr]);
        ellipse(s, [176, 176, 184], [x - fdiv(rr * 7, 10), y - fdiv(rr * 8, 10), fdiv(rr * 12, 10), fdiv(rr * 7, 10)]);
      }
      return c;
    });
  }

  // ============================================================= Zeichnen
  /** [Schlüssel, Bauen] aller Weltflächen von world - zum Vorab-Bauen. */
  function assets(world, w, h) {
    const out = [[wkey(world, w, h, "sky"), () => sky(world, w, h)]];
    for (let d = 0; d < 3; d++) out.push([wkey(world, w, h, "layer", d), () => layer(world, d, w, h)]);
    if (world === 0) out.push([wkey(0, w, h, "ground"), () => ground(w, h)]);
    else if (world === 2) {
      out.push([wkey(2, w, h, "boltglow"), () => boltGlow(w, h)]);
      for (let v = 0; v < 4; v++) out.push([wkey(2, w, h, "bolt", v), () => bolt(w, h, v)]);
    } else if (world === 3) {
      for (let k = 0; k < 2; k++) out.push([wkey(3, w, h, "ribbon", k), () => ribbon(w, h, k)]);
    } else if (world === 4) {
      out.push([wkey(4, w, h, "earth"), () => earth(w, h)]);
      out.push([wkey(4, w, h, "moon"), () => moonMaster(w, h)]);
      for (let k = 0; k < 3; k++) out.push([wkey(4, w, h, "planet", k), () => planet(w, h, k)]);
    }
    return out;
  }
  /** Eine noch fehlende Fläche von world bauen (höchstens eine je Bild). */
  function prebuild(world, w, h) {
    for (const [key, make] of assets(world, w, h)) {
      if (!wcache.has(key)) {
        make();
        return;
      }
    }
  }
  /** Weltflächen aller anderen Welten und Größen verwerfen - nur world und world + 1 bleiben. */
  function keep(world, w, h) {
    const a = grp(world, w, h), b = grp(world + 1, w, h);
    for (const g of groups.slice()) {
      if (g !== a && g !== b) {
        groups.splice(groups.indexOf(g), 1);
        dropGroup(g);
      }
    }
  }
  // bühnengroße Zwischenfläche für die Überblendung (eine, je Größe neu)
  let buf = null;
  function buffer(w, h) {
    const s = scale();
    if (!buf || buf.w !== w || buf.h !== h || buf.s !== s) {
      const [c, g] = canvas(w, h);
      buf = { c, g, w, h, s };
    }
    return buf;
  }

  /** Eine Welt deckend: Himmel, Ebenen fern -> nah mit Parallaxe, Details, Boden. */
  function scene(s, r, world, alt, t, rise, withGround) {
    const w = r.w, h = r.h;
    const sk = sky(world, w, h);
    s.drawImage(sk, r.x, r.y, sk.lw, sk.lh);
    for (let depth = 0; depth < 3; depth++) {
      tile(s, r, layer(world, depth, w, h), rise * PARALLAX[depth]);
      if (depth === 2 && withGround) {
        const gy = r.bottom - groundH(h) + trunc(rise);
        if (gy < r.bottom) {
          const gr = ground(w, h);
          s.drawImage(gr, r.x, gy, gr.lw, gr.lh);
        }
      }
      details(s, r, world, depth, alt, t, rise);
    }
  }

  /**
   * Ganze Bühne in rect: Welt mit Himmel, Ebenen (Parallaxe), Details und Boden; während der
   * Überblendung (share > 0) wird die nächste Welt genauso in eine Zwischenfläche gezeichnet
   * und mit share/255 darübergelegt. Ohne Überblendung wird die nächste Welt vorab gebaut.
   */
  function drawWorld(s, rect, world, share, alt, t) {
    const r0 = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
    const r = new PG.Rect(trunc(r0.x), trunc(r0.y), trunc(r0.w), trunc(r0.h));
    const w = r.w, h = r.h;
    if (w < 4 || h < 4) return;
    world = wclamp(world);
    t = Number(t) || 0;
    alt = Number(alt) || 0;
    keep(world, w, h);
    s.save();
    s.beginPath();
    s.rect(r.x, r.y, w, h);
    s.clip();
    if (world >= 5) {
      const m = moonScene(w, h);
      s.drawImage(m, r.x, r.y, m.lw, m.lh);
      twinkle(s, r, 5, t, 0.0);
    } else {
      share = world < 4 ? Math.max(0, Math.min(255, trunc(Number(share) || 0))) : 0;
      const rise = risePx(alt, h);
      scene(s, r, world, alt, t, rise, true);
      if (share) {
        const b = buffer(w, h);
        scene(b.g, new PG.Rect(0, 0, w, h), world + 1, alt, t, rise, false);
        const a = s.globalAlpha;
        s.globalAlpha = a * (share / 255);
        s.drawImage(b.c, r.x, r.y, w, h);
        s.globalAlpha = a;
      } else if (world < 4) prebuild(world + 1, w, h);
    }
    s.restore();
  }

  PG.lamaCrashWorld = {
    PARALLAX, NAMES, clearCache, sky, layer, ground, groundH, risePx, draw: drawWorld, moonScene, nameKey,
    // nur für Prüfstände (wie W._flash / W._hash in den Python-Vorschauen)
    _flash: flash, _hash: hash, _groups: () => groups.slice(), _cacheSize: () => wcache.size + scache.size,
    _moonMaster: moonMaster, _earth: earth, _planet: planet,
  };
})();
