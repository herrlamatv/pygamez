/*
 * lamacrash_draw.js - Zeichen-Bausteine von Lama Crash (Port von games/lamacrash_draw.py)
 * ======================================================================================
 * Nur Canvas-Pfade, keine Bilddateien. Alles Teure wird einmal je Größe in eine
 * Offscreen-Canvas (x PG.app.pixelScale) gezeichnet und gecacht; die draw*-Funktionen
 * setzen jedes Bild nur aus gecachten Teilen und billigen Primitiven zusammen.
 *
 * Schnittstelle wie in Python (camelCase), PG.lamaCrashDraw:
 *   clearCache(), inflLevel(m100), balloonSize(base, level) -> [w, h], balloon(base, level, stripe),
 *   basketW(base), basket(w, front), burnerFlame(w, step), parachute(w, col), moneySack(n, level),
 *   shreds(w, stripe) -> 8 Flächen, popStar(r, step), pillColor(m100) -> [bg, fg],
 *   pill(w, h, m100, label, fontSize), icon(name, size, col), drawGrid(s, rect, ylines, xlines, fontSize),
 *   drawCurve(s, pts, col, baseY, width), glow(r, col), curvePreview(w, h, crash100, marks),
 *   flag(n, col), llamaSize(base), drawRig(s, cx, rimY, base, level, stripe, riders, flame) -> PG.Rect,
 *   drawJumper(s, cx, cy, n, scarf, chute, openF), drawSack(s, cx, cy, n, chute, openF),
 *   drawFaller(s, cx, cy, n, scarf, step); Konstanten INFL_AT, INFL_MAX, RUBBER, ... wie in Python.
 *
 * Flächen sind Canvases in echter Pixeldichte, c.lw / c.lh = logische Größe (wie die übrigen
 * Web-Zeichenmodule): ctx.drawImage(c, x, y, c.lw, c.lh) bzw. blit(ctx, c, x, y[, alpha]).
 * glow() liefert den Lichthof auf DURCHSICHTIGEM Grund (Farbe x Deckkraft = Python-Wert auf
 * Schwarz): additiv zeichnen mit blitAdd(ctx, img, x, y) (= globalCompositeOperation "lighter",
 * wie BLEND_RGB_ADD); normal geblittet wirkt er als weicher Schein.
 *
 * pygame zeichnet Primitive auf SRCALPHA-Flächen mit "Alpha ersetzen". Wo das Ergebnis davon
 * abhängt, wird hier ausgestanzt und neu gezeichnet (replace), mit einem stufenlosen Verlauf
 * gearbeitet (gestufte Kreise/Ellipsen in Python) oder eine ganzzahlige Hilfsfläche in
 * Überabtastungs-Pixeln benutzt (Füllflächen). Formen wie im Original in den Maßen der
 * 2- bis 4-fachen Überabtastung (Kontext entsprechend skaliert, Canvas glättet die Kanten).
 * Unter aneinanderstoßende deckende Flächen (Bahnen) kommt eine Unterlage, damit die
 * Kantenglättung keine durchsichtigen Nähte lässt.
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const LCD = () => PG.lamaCrossingDraw;
  const PI = Math.PI;
  const TAU = PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);
  /** Kaufmännisch runden (x.5 nach oben), in Python, JS und Rust gleich. */
  const ri = (v) => Math.floor(v + 0.5);

  // Aufblas-Stufen: ab diesen Multiplikatoren (x100) wird der Ballon praller
  const INFL_AT = [100, 110, 125, 150, 200, 300, 500, 1000, 3000, 10000, 100000, 1000000];
  const INFL_MAX = INFL_AT.length - 1;

  const RUBBER = [232, 52, 76];
  const RUBBER_D = [150, 26, 46];
  const RUBBER_DD = [84, 14, 30];
  const RUBBER_L = [255, 226, 232];
  const WICKER = [176, 122, 62];
  const WICKER_D = [112, 72, 34];
  const WICKER_DD = [70, 42, 20];
  const WICKER_L = [222, 176, 108];
  const LEATHER = [98, 56, 34];
  const LEATHER_L = [160, 104, 64];
  const ROPE = [90, 70, 56];
  const METAL = [138, 144, 162];
  const METAL_L = [216, 222, 234];
  const METAL_D = [62, 66, 80];
  const GOLD = [232, 190, 92];
  const GOLD_L = [255, 236, 160];
  const GOLD_D = [160, 112, 34];
  const BURLAP = [196, 150, 88];
  const BURLAP_L = [230, 196, 134];
  const BURLAP_D = [128, 88, 46];
  const WHITE = [255, 255, 255];
  const BLACK = [0, 0, 0];

  const GORES = 8;
  const RIDER_RIM = 48;
  const CHUTE_HEM = 38;
  const FLAME_STEPS = [[100, 0, 100, 0], [93, -6, 97, 1], [97, 5, 103, -1], [89, -3, 95, 1], [100, 7, 99, -1], [95, -5, 104, 0]];
  const POP_SCALE = [52, 84, 100, 100, 97, 93];
  const POP_ALPHA = [255, 255, 255, 215, 140, 64];

  // ============================================================= Grundlagen
  const cache = new Map(); // große Teile: Ballon, Korb, Sprites
  const small = new Map(); // Texte, Pillen, Piktogramme (wechseln oft)
  const scale = () => (PG.app && PG.app.pixelScale) || 1;

  function cached(key, build, store) {
    store = store || cache;
    const k = key + "|" + scale();
    let c = store.get(k);
    if (c === undefined) {
      if (store.size > 400) store.clear();
      c = build();
      store.set(k, c);
    }
    return c;
  }
  function clearCache() {
    cache.clear();
    small.clear();
    work = null;
  }

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
  /** Wie canvas(), aber in den Maßen der ss-fachen Überabtastung (Python "big"). */
  function ssCanvas(w, h, ss) {
    const [c, g] = canvas(w, h);
    g.scale(1 / ss, 1 / ss);
    return [c, g];
  }
  /** Gleich große Fläche 1:1 (Gerätepixel) darüberlegen. */
  function blitRaw(g, src, alpha) {
    g.save();
    g.setTransform(1, 0, 0, 1, 0, 0);
    if (alpha != null) g.globalAlpha = alpha / 255;
    g.drawImage(src, 0, 0);
    g.restore();
  }
  /** Gecachte Fläche in logischer Größe zeichnen (alpha 0..255 wie set_alpha). */
  function blit(ctx, img, x, y, alpha) {
    if (alpha != null && alpha < 255) {
      if (alpha <= 0) return;
      const a = ctx.globalAlpha;
      ctx.globalAlpha = a * (alpha / 255);
      ctx.drawImage(img, x, y, img.lw, img.lh);
      ctx.globalAlpha = a;
    } else ctx.drawImage(img, x, y, img.lw, img.lh);
  }
  /** Additiv zeichnen (wie blit mit special_flags=BLEND_RGB_ADD). */
  function blitAdd(ctx, img, x, y) {
    const op = ctx.globalCompositeOperation;
    ctx.globalCompositeOperation = "lighter";
    ctx.drawImage(img, x, y, img.lw, img.lh);
    ctx.globalCompositeOperation = op;
  }

  const rgb = (c) => [c[0] | 0, c[1] | 0, c[2] | 0];
  const css = (c) => ui.col(c);
  const rgba = (c, a) => [c[0], c[1], c[2], Math.max(0, Math.min(255, trunc(a)))];
  /** Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet). */
  function lerpI(c1, c2, num, den) {
    den = Math.max(1, den);
    return [c1[0] + fdiv((c2[0] - c1[0]) * num, den), c1[1] + fdiv((c2[1] - c1[1]) * num, den),
      c1[2] + fdiv((c2[2] - c1[2]) * num, den)];
  }
  /** Ganzzahl-Streuwert 0..2^32-1 (deterministisch, wie _hash in Python). */
  function hash(i) {
    let v = (Math.imul(i, 2654435761 | 0) + 40503) >>> 0;
    v = Math.imul(v ^ (v >>> 15), 2246822519 | 0) >>> 0;
    return (v ^ (v >>> 13)) >>> 0;
  }

  // ------------------------------------------------------------ Pfade
  function path(g, pts, close = true) {
    g.beginPath();
    g.moveTo(pts[0][0], pts[0][1]);
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1]);
    if (close) g.closePath();
  }
  function fillPts(g, col, pts) {
    if (pts.length < 3) return;
    path(g, pts);
    g.fillStyle = css(col);
    g.fill();
  }
  /** Linienzug (Gelenke rund); round = runde Enden (wie _stroke mit Kreisen). */
  function strokePts(g, col, pts, lw, round = false, closed = false) {
    if (pts.length < 2) return;
    path(g, pts, closed);
    g.strokeStyle = css(col);
    g.lineWidth = lw;
    g.lineJoin = "round";
    g.lineCap = round ? "round" : "butt";
    g.stroke();
  }
  /** Mehrere Linienzüge gleicher Farbe als EIN Pfad (Überlappungen nicht doppelt). */
  function strokeMany(g, col, lines, lw) {
    g.beginPath();
    for (const pts of lines) {
      if (pts.length < 2) continue;
      g.moveTo(pts[0][0], pts[0][1]);
      for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1]);
    }
    g.strokeStyle = css(col);
    g.lineWidth = lw;
    g.lineJoin = "round";
    g.lineCap = "butt";
    g.stroke();
  }
  /**
   * Ein Abschnitt von pygame.draw.line(..., width > 1) als Fläche (wie draw_line_width in
   * pygame-ce): Endpunkte abgeschnitten (int), die Breite wird WAAGERECHT (steile Linie) bzw.
   * SENKRECHT (flache Linie) gemessen - schräge Linien sind also dünner (45°: Breite × 0,71)
   * und gerade abgeschnitten; gerade Breiten wachsen zur positiven Seite. Das Parallelogramm
   * läuft durch die Mitten der Bresenham-Spannen (Kanten geglättet statt treppig).
   */
  function pgSegPts(x1, y1, x2, y2, w) {
    x1 = trunc(x1);
    y1 = trunc(y1);
    x2 = trunc(x2);
    y2 = trunc(y2);
    const hw = fdiv(w, 2), lo = 1 - (w % 2) - hw;
    if (Math.abs(x1 - x2) <= Math.abs(y1 - y2)) {
      if (y1 > y2) [x1, y1, x2, y2] = [x2, y2, x1, y1];
      const sl = y2 > y1 ? (x2 - x1) / (y2 - y1) : 0;
      const a = x1 - 0.5 * sl + lo, b = x2 + 0.5 * sl + lo;
      return [[a, y1], [a + w, y1], [b + w, y2 + 1], [b, y2 + 1]];
    }
    if (x1 > x2) [x1, y1, x2, y2] = [x2, y2, x1, y1];
    const sl = (y2 - y1) / (x2 - x1);
    const a = y1 - 0.5 * sl + lo, b = y2 + 0.5 * sl + lo;
    return [[x1, a], [x1, a + w], [x2 + 1, b + w], [x2 + 1, b]];
  }
  /** Abschnitte von pygame.draw.lines (ohne Gelenk-Füllung) in den aktuellen Pfad -
   *  alle im selben Umlaufsinn, damit "nonzero" die Vereinigung füllt. */
  function pgSegsPath(g, pts, closed, w) {
    const seg = (p, q) => {
      const poly = pgSegPts(p[0], p[1], q[0], q[1], w);
      let area = 0;
      for (let i = 0; i < 4; i++) area += poly[i][0] * poly[(i + 1) % 4][1] - poly[(i + 1) % 4][0] * poly[i][1];
      if (area < 0) poly.reverse();
      g.moveTo(poly[0][0], poly[0][1]);
      for (let i = 1; i < 4; i++) g.lineTo(poly[i][0], poly[i][1]);
      g.closePath();
    };
    for (let i = 1; i < pts.length; i++) seg(pts[i - 1], pts[i]);
    if (closed && pts.length > 2) seg(pts[pts.length - 1], pts[0]);
  }
  /** pygame.draw.circle (gefüllt) in den aktuellen Pfad: Mitte und Radius abgeschnitten,
   *  Scheibe um die Pixel-ECKE (Mitte) - [c - r, c + r). */
  function pgDiscPath(g, c, r) {
    r = trunc(r);
    if (r < 1) return;
    const x = trunc(c[0]), y = trunc(c[1]);
    g.moveTo(x + r, y);
    g.arc(x, y, r, 0, TAU);
    g.closePath();
  }
  /** pygame.draw.lines(g, col, closed, pts, w) mit pygame-Geometrie (siehe pgSegPts);
   *  discR > 0: zusätzlich Kreise mit diesem Radius auf jedem Punkt (wie _stroke). */
  function pgLines(g, col, closed, pts, w, discR = 0) {
    w = trunc(w);
    if (!pts || pts.length < 2 || w < 1) return;
    if (w === 1) {
      // 1 px: Pixelmitte wie draw.lines (Kreise erst ab lw > 2)
      draw.lines(g, col, closed, pts.map(([x, y]) => [trunc(x), trunc(y)]), 1);
      return;
    }
    g.beginPath();
    pgSegsPath(g, pts, closed, w);
    if (discR > 0) for (const p of pts) pgDiscPath(g, p, discR);
    g.fillStyle = css(col);
    g.fill("nonzero");
  }
  function pgDisc(g, col, c, r) {
    g.beginPath();
    pgDiscPath(g, c, r);
    g.fillStyle = css(col);
    g.fill();
  }
  function pgLine(g, col, p1, p2, w) {
    pgLines(g, col, false, [p1, p2], w);
  }
  /** _stroke: dicke Linie (pygame.draw.lines) mit Kreisen auf den Punkten ab lw > 2. */
  function stroke(g, col, pts, lw) {
    lw = Math.max(1, trunc(lw));
    pgLines(g, col, false, pts, lw, lw > 2 ? lw / 2 : 0);
  }
  function disc(g, col, c, r) {
    if (r <= 0) return;
    g.beginPath();
    g.arc(c[0], c[1], r, 0, TAU);
    g.fillStyle = css(col);
    g.fill();
  }
  /** pygame.draw.ellipse mit Rect-Kürzung. */
  function ellipse(g, col, rect, width = 0) {
    draw.ellipse(g, col, [trunc(rect[0]), trunc(rect[1]), trunc(rect[2]), trunc(rect[3])], width);
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
  /** Maske (BLEND_RGBA_MIN mit weißem Polygon): nur das Innere von pts behalten. */
  function keepInside(g, pts) {
    g.save();
    g.globalCompositeOperation = "destination-in";
    fillPts(g, BLACK, pts);
    g.restore();
  }

  /**
   * Gestufter Radialverlauf (_radial): von außen (rOut, aOut) nach innen (rIn, aIn), jeder
   * kleinere Kreis ersetzt die Mitte - hier als stufenloser Verlauf durch die Stufenmitten.
   */
  function radial(g, col, c, rIn, rOut, aIn, aOut, steps, power = 1.0) {
    const rs = [], as = [];
    for (let i = 0; i <= steps; i++) {
      const f = i / steps;
      rs.push(Math.max(1, trunc(rIn + (rOut - rIn) * f)));
      as.push(trunc(aIn + (aOut - aIn) * f ** power));
    }
    const R = rs[steps];
    const gr = g.createRadialGradient(c[0], c[1], 0, c[0], c[1], R);
    const stop = (r, a) => gr.addColorStop(Math.max(0, Math.min(1, r / R)), `rgba(${col[0]},${col[1]},${col[2]},${a / 255})`);
    stop(0, as[0]);
    stop(rs[0], as[0]);
    for (let i = 1; i <= steps; i++) stop((rs[i - 1] + rs[i]) / 2, as[i]);
    stop(R, as[steps]);
    g.fillStyle = gr;
    g.beginPath();
    g.arc(c[0], c[1], R, 0, TAU);
    g.fill();
  }

  function ellipsePts(cx, cy, rx, ry, rot = 0, n = 24) {
    const ca = Math.cos(rot), sa = Math.sin(rot);
    const pts = [];
    for (let i = 0; i < n; i++) {
      const a = (TAU * i) / n;
      const x = rx * Math.cos(a), y = ry * Math.sin(a);
      pts.push([cx + x * ca - y * sa, cy + x * sa + y * ca]);
    }
    return pts;
  }

  /** Sichel entlang eines Kreisbogens, in der Mitte thick dick, an den Enden spitz. */
  function arcBand(cx, cy, rOut, thick, a0, a1, n = 14) {
    const outer = [], inner = [];
    for (let i = 0; i <= n; i++) {
      const f = i / n;
      const a = a0 + (a1 - a0) * f;
      const t = thick * Math.sin(PI * f);
      const ca = Math.cos(a), sa = Math.sin(a);
      outer.push([cx + rOut * ca, cy + rOut * sa]);
      inner.push([cx + (rOut - t) * ca, cy + (rOut - t) * sa]);
    }
    return outer.concat(inner.reverse());
  }

  function sparkle(g, col, x, y, r) {
    const q = r * 0.26;
    fillPts(g, col, [[x, y - r], [x + q, y - q], [x + r, y], [x + q, y + q], [x, y + r], [x - q, y + q], [x - r, y], [x - q, y - q]]);
  }

  /** Gezackter Comic-Stern: Zackenlängen über hash leicht unregelmäßig. */
  function burst(cx, cy, rOut, rIn, spikes, rot, seed, jit = 0.22) {
    const pts = [];
    for (let j = 0; j < spikes * 2; j++) {
      const a = rot + (PI * j) / spikes;
      const rr = j % 2 === 0 ? rOut * (1.0 - (jit * (hash(seed + j) % 100)) / 100.0)
        : rIn * (1.0 - (0.12 * (hash(seed + j * 7) % 100)) / 100.0);
      pts.push([cx + Math.cos(a) * rr, cy + Math.sin(a) * rr]);
    }
    return pts;
  }

  // ============================================================= Steigkurve (curvePreview)
  // aus PG.lamaCrashLogic; nur falls das fehlt (Vorschau ohne Spiel) eine eigene Tabelle
  let fbTable = null;
  function fbMult(t) {
    if (!fbTable) {
      fbTable = new Array(11521);
      let mu = 1000000;
      for (let i = 0; i <= 11520; i++) {
        fbTable[i] = Math.min(10000000, Math.floor(mu / 10000));
        mu += Math.floor(mu / 1000);
      }
    }
    return fbTable[Math.max(0, Math.min(11520, trunc(t)))];
  }
  function fbTick(m100) {
    fbMult(0);
    let lo = 0, hi = fbTable.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (fbTable[mid] < m100) lo = mid + 1;
      else hi = mid;
    }
    return Math.min(11520, lo);
  }
  function logic() {
    const L = PG.lamaCrashLogic || {};
    const has = (k) => typeof L[k] === "function";
    const MAX = L.MAX_MULT100 || 10000000;
    const multAt = has("multAt") ? L.multAt : fbMult;
    const tickAt = has("tickAt") ? L.tickAt : fbTick;
    return {
      MAX_MULT100: MAX,
      MIN_CASH: L.MIN_CASH || 101,
      TPS: L.TPS || 60,
      multAt,
      tickAt,
      popTick: has("popTick") ? L.popTick : (c) => (c >= MAX ? (L.MOON_TICK || 11520) : tickAt(c + 1)),
    };
  }

  // ============================================================= Größen
  function inflLevel(m100) {
    let lvl = 0;
    for (let i = 0; i < INFL_AT.length; i++) if (m100 >= INFL_AT[i]) lvl = i;
    return lvl;
  }
  /** Korbbreite zur Grundbreite des Ballons. */
  const basketW = (base) => Math.max(8, fdiv(trunc(base) * 9, 20));
  /** Größe der Lamas im Korb (Lama-Sprite n x n). */
  const llamaSize = (base) => Math.max(8, fdiv(basketW(base) * 3, 4));
  /** [w, h] der Ballon-Fläche. */
  function balloonSize(base, level) {
    base = trunc(base);
    level = Math.max(0, Math.min(INFL_MAX, trunc(level)));
    const w = Math.max(12, fdiv(base * (100 + 4 * level), 100));
    return [w, fdiv(w * (118 - level), 100) + fdiv(base * 9, 25)];
  }
  /** [Höhe, Randwulst, Tiefe der Vorderkante] des Korbs der Breite w. */
  function basketDims(w) {
    const h = Math.max(4, fdiv(w * 2, 3));
    const t = Math.max(2, fdiv(h * 14, 100));
    const e = Math.max(t * 2, fdiv(h * 30, 100));
    return [h, t, e];
  }
  /** Maße unter der Hülle (Pixel über dem Korbrand), wie _rig in Python. */
  function rigDims(base) {
    base = trunc(base);
    const bw = basketW(base);
    const n = llamaSize(base);
    const [, t, e] = basketDims(bw);
    const ears = fdiv(n * RIDER_RIM, 100) - (e - fdiv(t, 2));
    const frame = Math.max(3, ears + Math.max(2, fdiv(base, 30)));
    const burnerH = Math.max(3, fdiv(base, 20));
    const burnerHw = Math.max(2, fdiv(bw * 11, 100));
    const mouth = Math.max(frame + burnerH + 4, fdiv(base * 9, 25));
    const skirt = Math.max(2, fdiv(base * 7, 100));
    const neckHw = Math.max(2, fdiv(bw * 34, 100));
    const mouthHw = Math.max(3, fdiv(bw * 40, 100));
    const flameW = Math.max(4, fdiv(bw * 9, 25));
    return [mouth, skirt, neckHw, mouthHw, frame, burnerH, burnerHw, flameW];
  }
  /** y der Lama-Füße im Korb: Sprite-Zeile RIDER_RIM % liegt auf der Vorderkante. */
  function riderFoot(rimY, base) {
    const n = llamaSize(base);
    const [, t, e] = basketDims(basketW(base));
    return trunc(rimY) + e - fdiv(t, 2) + n - fdiv(n * RIDER_RIM, 100);
  }

  // ============================================================= Lama-Sprite
  /** Lama n x n wie llama_sprite in Python (2-fach gezeichnet, weich verkleinert). */
  function llamaSprite(n, pose, scarf) {
    scarf = rgb(scarf);
    return cached(`llama|${n}|${pose}|${scarf}`, () => {
      const [c, g] = canvas(n, n);
      g.scale(0.5, 0.5);
      LCD().drawLlama(g, n * 2, pose, scarf);
      return c;
    });
  }

  // ============================================================= Ballon
  function profile(R, L, m, level) {
    const f = Math.max(0, Math.min(INFL_MAX, level)) / INFL_MAX;
    const bulge = 0.92 * f ** 1.15;
    const p = 1.32 - 0.36 * f;
    const out = [];
    for (let i = 0; i < 18; i++) {
      const a = ((PI / 2) * i) / 18;
      out.push([-R * Math.cos(a), R * Math.sin(a)]);
    }
    for (let i = 0; i < 23; i++) {
      const u = i / 22;
      const g0 = Math.max(0, Math.cos((u * PI) / 2)) ** p;
      const g1 = Math.sqrt(Math.max(0, 1 - u * u));
      out.push([L * u, m + (R - m) * (g0 + (g1 - g0) * bulge)]);
    }
    return out;
  }
  function goreBounds() {
    const out = [-PI / 2];
    for (let k = 0; k < GORES; k++) out.push(-PI / 2 + ((k + 0.5) * PI) / GORES);
    out.push(PI / 2);
    return out;
  }
  /** Bahnfarbe mit Kontrast zu RUBBER (zu ähnliche Farben werden Creme-Rosa). */
  function stripeCol(stripe) {
    const c = rgb(stripe);
    if (Math.abs(c[0] - RUBBER[0]) + Math.abs(c[1] - RUBBER[1]) + Math.abs(c[2] - RUBBER[2]) < 150) return lerpI(c, [255, 246, 232], 82, 100);
    return c;
  }

  function balloon(base, level, stripe) {
    base = trunc(base);
    level = Math.max(0, Math.min(INFL_MAX, trunc(level)));
    const st = stripeCol(stripe);
    return cached(`balloon|${base}|${level}|${st}`, () => buildBalloon(base, level, st));
  }

  function buildBalloon(base, level, stripe) {
    const [w, h] = balloonSize(base, level);
    const [mouth, skirt, neckHw, mouthHw, frame, burnerH, burnerHw] = rigDims(base);
    const bw = basketW(base);
    const ss = 3;
    const W = w * ss, H = h * ss;
    const cx = W / 2;
    const R = ((w - 2) * ss) / 2;
    const yEq = ss + R;
    const yMouth = (h - mouth) * ss;
    const yNeck = yMouth - skirt * ss;
    const L = Math.max(R * 0.5, yNeck - yEq);
    const m = neckHw * ss;
    const mb = mouthHw * ss;
    const prof = profile(R, L, m, level);
    const right = prof.map(([dy, r]) => [cx + r, yEq + dy]);
    const left = prof.map(([dy, r]) => [cx - r, yEq + dy]);
    const sil = right.concat(left.slice().reverse());
    const lift = level * 2; // dünner Gummi -> heller
    const red = lerpI(RUBBER, RUBBER_L, lift, 100);
    const st = lerpI(stripe, WHITE, lift, 100);
    const outline = lerpI(RUBBER_DD, BLACK, 25, 100);
    const bounds = goreBounds();
    const [c, g] = ssCanvas(w, h, ss);

    // --- Hülle: Bahnen (Unterlage gegen Kantennähte)
    fillPts(g, red, sil);
    for (let j = 0; j < bounds.length - 1; j++) {
      const s0 = Math.sin(bounds[j]), s1 = Math.sin(bounds[j + 1]);
      const poly = prof.map(([dy, r]) => [cx + r * s0, yEq + dy]);
      for (let i = prof.length - 1; i >= 0; i--) poly.push([cx + prof[i][1] * s1, yEq + prof[i][0]]);
      fillPts(g, j % 2 === 0 ? red : st, poly);
    }
    // Nähte (bei praller Hülle glänzen sie gespannt)
    const sw = Math.max(2, fdiv(ss * 2, 3));
    const seams = [], shines = [];
    for (const b of bounds.slice(1, -1)) {
      const sb = Math.sin(b);
      const line = prof.slice(2).map(([dy, r]) => [cx + r * sb, yEq + dy]);
      seams.push(line);
      if (level >= 3) shines.push(line.slice(3).map(([x, y]) => [x - sw, y]));
    }
    strokeMany(g, [70, 10, 28, 84], seams, sw);
    if (shines.length) strokeMany(g, [255, 240, 244, Math.min(150, 14 + level * 12)], shines, Math.max(1, fdiv(sw, 2)));
    // Falten der noch schlaffen Hülle (nur Stufe 0..2)
    const folds = [];
    for (let j = 0; j < 3 - Math.min(3, level); j++) {
      const phi = [-0.42, 0.18, 0.62][j];
      const pts = [];
      prof.forEach(([dy, r], i) => {
        if (dy >= L * 0.66) {
          const wob = 0.05 * Math.sin(i * 1.7 + j);
          pts.push([cx + r * Math.sin(phi + wob), yEq + dy]);
        }
      });
      folds.push(pts);
    }
    if (folds.length) strokeMany(g, [60, 8, 24, 70], folds, ss);
    // Schatten: Licht von links oben, rechts unten dunkel
    radial(g, [58, 6, 24], [cx - R * 0.36, yEq - R * 0.42], R * 0.1, R * 2.4, 0, 196 - level * 6, 30, 1.45);
    // Gegenlicht am rechten/unteren Rand (Gummi spiegelt): Silhouette minus verschobene Silhouette
    g.save();
    path(g, sil);
    g.clip();
    g.beginPath();
    g.rect(-W, -H, W * 3, H * 3);
    const sh = sil.map(([x, y]) => [x - R * 0.075, y - R * 0.05]);
    g.moveTo(sh[0][0], sh[0][1]);
    for (let i = 1; i < sh.length; i++) g.lineTo(sh[i][0], sh[i][1]);
    g.closePath();
    g.fillStyle = css([255, 150, 160, 40 + level * 4]);
    g.fill("evenodd");
    g.restore();
    // Glanz: 10 ineinander liegende gedrehte Ellipsen (innen deckender) -> Verlauf
    const amax = 54 + level * 9;
    const rx = R * 0.52;
    g.save();
    g.translate(cx - R * 0.4, yEq - R * 0.46);
    g.rotate(-0.62);
    g.scale(1, 0.4 / 0.52);
    const gr = g.createRadialGradient(0, 0, 0, 0, 0, rx);
    const ga = (i) => `rgba(255,240,244,${trunc(amax * ((11 - i) / 10) ** 1.6) / 255})`;
    gr.addColorStop(0, ga(1));
    for (let i = 1; i <= 10; i++) gr.addColorStop((i - 0.5) / 10, ga(i));
    gr.addColorStop(1, ga(10));
    g.fillStyle = gr;
    g.beginPath();
    g.arc(0, 0, rx, 0, TAU);
    g.fill();
    g.restore();
    // scharfe Glanzformen darüber
    fillPts(g, [255, 255, 255, 168 + level * 7], arcBand(cx, yEq, R * 0.88, R * (0.085 + 0.005 * level), PI * 1.1, PI * (1.34 + 0.008 * level), 16));
    const pa = PI * (1.4 + 0.006 * level);
    const dot = [cx + R * 0.82 * Math.cos(pa), yEq + R * 0.82 * Math.sin(pa)];
    disc(g, [255, 255, 255, 190 + level * 5], dot, R * (0.03 + 0.0015 * level));
    if (level >= 4) {
      // gespannte Haut: Randlicht links unten am Bauch
      const k = level - 3;
      const sel = prof.filter(([dy]) => dy >= 0 && dy <= L * (0.4 + 0.03 * k));
      const rim = [], inner = [];
      sel.forEach(([dy, r], i) => {
        const t = Math.sin((PI * i) / Math.max(1, sel.length - 1));
        rim.push([cx - r * 0.955, yEq + dy]);
        inner.push([cx - r * (0.955 - (0.03 + 0.004 * k) * t), yEq + dy]);
      });
      if (rim.length >= 2) fillPts(g, [255, 236, 240, 50 + 14 * k], rim.concat(inner.reverse()));
    }
    if (level >= 6) {
      // Spannungs-Streifen neben dem Glanz
      for (let j = 0; j < 1 + fdiv(level - 6, 2); j++) {
        fillPts(g, [255, 255, 255, 70 + 10 * j], arcBand(cx, yEq, R * (0.74 - 0.07 * j), R * 0.022, PI * (1.17 + 0.02 * j), PI * (1.31 - 0.01 * j), 10));
      }
    }
    if (level >= 8) {
      // zweiter Lichtreflex rechts oben
      fillPts(g, [255, 250, 252, 60 + 12 * (level - 8)], arcBand(cx, yEq, R * 0.86, R * 0.05, PI * 1.66, PI * 1.8, 10));
    }
    if (level >= 10) sparkle(g, [255, 255, 255, 230], dot[0] + R * 0.09, dot[1] + R * 0.05, R * 0.085);
    keepInside(g, sil);
    const olw = Math.max(2, fdiv(ss * 4, 3));
    strokeMany(g, outline, [right, left], olw);

    // --- Schürze (Bahnen setzen sich dunkler fort) + Ring am Hals
    const yn = yNeck - ss;
    const skx = (y, sb) => cx + (m + ((mb - m) * (y - yn)) / Math.max(1.0, yMouth - yn)) * sb;
    const skirtCol = (j, mid) => lerpI(j % 2 === 0 ? red : st, [40, 8, 20], 28 + trunc(24 * (mid + 1)), 100);
    fillPts(g, skirtCol(1, 0), [[skx(yn, -1), yn], [skx(yMouth, -1), yMouth], [skx(yMouth, 1), yMouth], [skx(yn, 1), yn]]);
    for (let j = 0; j < bounds.length - 1; j++) {
      const s0 = Math.sin(bounds[j]), s1 = Math.sin(bounds[j + 1]);
      fillPts(g, skirtCol(j, (s0 + s1) / 2), [[skx(yn, s0), yn], [skx(yMouth, s0), yMouth], [skx(yMouth, s1), yMouth], [skx(yn, s1), yn]]);
    }
    draw.line(g, outline, [cx - m, yn], [cx - mb, yMouth], olw);
    draw.line(g, outline, [cx + m, yn], [cx + mb, yMouth], olw);
    const ring = Math.max(ss * 2, fdiv(skirt * ss, 3));
    rect(g, outline, [cx - m - ss, yn - fdiv(ring, 2), 2 * m + 2 * ss, ring], 0, fdiv(ring, 2));
    draw.line(g, lerpI(red, WHITE, 30, 100), [cx - m * 0.75, yn - fdiv(ring, 4)], [cx - m * 0.15, yn - fdiv(ring, 4)], Math.max(1, fdiv(ss, 2)));
    // Mundöffnung (dunkel) mit hellem Saum vorn
    const ry = Math.max(ss * 1.5, mb * 0.2);
    ellipse(g, [34, 10, 18], [cx - mb, yMouth - ry, 2 * mb, 2 * ry]);
    const hem = [];
    for (let i = 0; i < 13; i++) hem.push([cx + mb * Math.cos((PI * i) / 12), yMouth + ry * Math.sin((PI * i) / 12)]);
    stroke(g, lerpI(red, [40, 8, 20], 20, 100), hem, ss);

    // --- Seile, Brennerrahmen, Brenner
    const rlw = Math.max(ss, fdiv(ss * base, 100));
    const yb = H + ss;
    const cl = cx - (bw * ss) / 2, cr = cx + (bw * ss) / 2;
    const il = [cx - mb * 0.4, yMouth + ry * 0.7];
    const ir = [cx + mb * 0.4, yMouth + ry * 0.7];
    const ropeD = lerpI(ROPE, BLACK, 35, 100);
    draw.line(g, ropeD, il, [cl, yb], rlw);
    draw.line(g, ropeD, ir, [cr, yb], rlw);
    const yf = H - frame * ss;
    const tf = (yf - il[1]) / Math.max(1.0, yb - il[1]);
    const xl = il[0] + (cl - il[0]) * tf;
    const xr = ir[0] + (cr - ir[0]) * tf;
    const bar = Math.max(ss, fdiv(ss * base, 75));
    rect(g, METAL_D, [xl, yf - bar / 2, trunc(xr - xl), bar], 0, Math.max(1, fdiv(bar, 2)));
    draw.line(g, METAL, [xl + bar, yf - bar / 4], [xr - bar, yf - bar / 4], Math.max(1, fdiv(bar, 3)));
    const bhw = burnerHw * ss;
    const bt = yf - burnerH * ss;
    const bx = trunc(cx - bhw), by = trunc(bt), bwd = trunc(2 * bhw), bhh = trunc(yf - bt + fdiv(bar, 2));
    rect(g, METAL_D, [bx, by, bwd, bhh], 0, ss * 2);
    rect(g, METAL, [bx + ss, by + ss, fdiv(bwd * 11, 20), bhh - ss * 2], 0, ss);
    draw.line(g, METAL_L, [bx + ss * 2, by + ss * 2], [bx + ss * 2, by + bhh - ss * 2], Math.max(1, fdiv(ss * 2, 3)));
    for (const fy of [0.38, 0.68]) {
      const yy = by + bhh * fy;
      draw.line(g, METAL_D, [bx + ss, yy], [bx + bwd - ss, yy], Math.max(1, fdiv(ss * 2, 3)));
    }
    ellipse(g, [28, 28, 36], [bx + ss, by - ss, bwd - 2 * ss, ss * 2]);
    draw.line(g, ROPE, [cx - mb, yMouth], [cl, yb], rlw);
    draw.line(g, ROPE, [cx + mb, yMouth], [cr, yb], rlw);
    return c;
  }

  // ============================================================= Korb
  /** Korb w x (w*2//3): front=false Rückwand + Innenraum, front=true Vorderwand. */
  function basket(w, front) {
    w = Math.max(8, trunc(w));
    front = !!front;
    return cached(`basket|${w}|${front ? 1 : 0}`, () => {
      const [bh, t, e] = basketDims(w);
      const ss = 3;
      const [c, g] = ssCanvas(w, bh, ss);
      if (front) basketFront(g, w, bh, w * ss, bh * ss, t * ss, e * ss, ss);
      else basketBack(g, w * ss, bh * ss, t * ss, e * ss, ss);
      return c;
    });
  }

  function basketBack(g, W, H, T, E, ss) {
    const x0 = trunc(T * 0.5), x1 = trunc(W - T * 0.5);
    const y0 = trunc(T * 0.5), y1 = trunc(Math.min(H, E + T));
    if (y1 > y0 && x1 > x0) {
      const gr = g.createLinearGradient(0, y0, 0, y1);
      gr.addColorStop(0, css([100, 66, 36]));
      gr.addColorStop(1, css([44, 26, 12]));
      g.fillStyle = gr;
      g.fillRect(x0, y0, x1 - x0, y1 - y0);
    }
    const step = Math.max(ss * 3, T);
    const lw = Math.max(1, fdiv(ss, 2));
    for (let y = y0 + fdiv(step, 2); y < y1; y += step) {
      draw.line(g, [36, 22, 10], [x0, y], [x1, y], lw);
      const off = (fdiv(y, step) % 2) * step;
      for (let x = x0 + off; x < x1; x += step * 2) {
        draw.line(g, [128, 88, 50], [x + ss, y - fdiv(step, 2) + ss], [x + step - ss, y - fdiv(step, 2) + ss], lw);
      }
    }
    draw.rect(g, lerpI(LEATHER, BLACK, 28, 100), [0, 0, W, T], 0, fdiv(T, 2));
    draw.line(g, LEATHER, [fdiv(T, 2), fdiv(T, 3)], [W - fdiv(T, 2), fdiv(T, 3)], lw);
  }

  /** Schatten der Vorderwand als ganzzahlige Fläche W x H (Füllen ersetzt wie in pygame). */
  function basketShade(W, H, T, E, ss) {
    const c = ui.makeCanvas(W, H);
    const s = c.getContext("2d");
    const fillRep = (col, x, y, w, h) => {
      if (w <= 0 || h <= 0) return;
      s.clearRect(x, y, w, h);
      s.fillStyle = css(col);
      s.fillRect(x, y, w, h);
    };
    const bands = 12;
    for (let i = 0; i < bands; i++) {
      const x = fdiv(W * (40 + fdiv(60 * i, bands)), 100);
      fillRep([30, 14, 4, 8 + 6 * i], x, 0, W - x, H);
    }
    for (let i = 0; i < 6; i++) {
      const y = trunc(H - ((H - E) * (6 - i)) / 14);
      fillRep([30, 14, 4, 10 + 6 * i], 0, y, W, H - y);
    }
    fillRep([20, 10, 2, 90], 0, E + fdiv(T, 2), W, Math.max(ss, fdiv(T, 2)));
    return c;
  }

  function basketFront(g, w, bh, W, H, T, E, ss) {
    const inset = fdiv(W * 5, 100);
    const rad = Math.max(ss * 2, fdiv(H - E, 4));
    const wall = [[0, E], [W, E]];
    for (let i = 0; i < 7; i++) {
      const a = ((PI / 2) * i) / 6;
      wall.push([W - inset - rad + rad * Math.cos(a), H - rad + rad * Math.sin(a)]);
    }
    for (let i = 0; i < 7; i++) {
      const a = PI / 2 + ((PI / 2) * i) / 6;
      wall.push([inset + rad + rad * Math.cos(a), H - rad + rad * Math.sin(a)]);
    }
    const [lay, lg] = ssCanvas(w, bh, ss);
    lg.fillStyle = css(WICKER_DD);
    lg.fillRect(0, 0, W, H);
    // Geflecht: Reihen versetzter, gewölbter Stäbe
    const top = Math.min(E + fdiv(T, 2), H - ss * 2);
    const rows = Math.max(1, ri((H - top) / (ss * 5.5)));
    const rh = Math.max(ss, (H - top) / rows);
    const sw = rh * 1.9;
    const gap = Math.max(1, fdiv(ss * 2, 3));
    for (let r = 0; r < rows; r++) {
      const y = top + r * rh;
      let x = -sw + ((r % 2) * sw) / 2;
      while (x < W + sw) {
        ellipse(lg, WICKER, [x + gap, y + gap, sw - 2 * gap, rh - 2 * gap]);
        ellipse(lg, WICKER_L, [x + gap * 2, y + gap * 1.5, sw - 4 * gap, (rh - 2 * gap) * 0.45]);
        x += sw;
      }
    }
    // Schatten: rechts und unten dunkler, Schatten unter dem Wulst
    lg.save();
    lg.imageSmoothingQuality = "high";
    lg.drawImage(basketShade(W, H, T, E, ss), 0, 0, W, H);
    lg.restore();
    draw.line(lg, WICKER_L, [ss, E], [inset + ss, H - rad], Math.max(1, ss));
    rect(lg, WICKER_DD, [0, H - T * 0.8, W, trunc(T * 0.8) + 1]);
    draw.line(lg, WICKER_D, [0, trunc(H - T * 0.8)], [W, trunc(H - T * 0.8)], Math.max(1, fdiv(ss, 2)));
    keepInside(lg, wall);
    blitRaw(g, lay);
    draw.lines(g, WICKER_DD, false, wall.slice(1).concat([wall[0]]), Math.max(1, fdiv(ss * 2, 3)));
    // Eckpfosten (verbinden hinteren und vorderen Rand) mit Seil-Ringen
    const pw = trunc(T * 0.9);
    for (const x of [0, W - pw]) {
      draw.rect(g, LEATHER, [x, 0, pw, E + fdiv(T, 2)], 0, Math.max(1, fdiv(pw, 3)));
      draw.line(g, LEATHER_L, [x + fdiv(pw, 3), ss], [x + fdiv(pw, 3), E], Math.max(1, fdiv(ss, 2)));
      disc(g, METAL_D, [x + pw / 2, T * 0.45], T * 0.36);
      draw.circle(g, METAL_L, [x + pw / 2, T * 0.45], T * 0.36, Math.max(1, fdiv(ss * 2, 3)));
    }
    // vorderer Randwulst
    draw.rect(g, lerpI(LEATHER, BLACK, 30, 100), [0, E - fdiv(T, 2) + fdiv(ss, 2), W, T], 0, fdiv(T, 2));
    draw.rect(g, LEATHER, [0, E - fdiv(T, 2), W, T - fdiv(ss, 2)], 0, fdiv(T, 2));
    draw.line(g, LEATHER_L, [fdiv(T, 2), E - fdiv(T, 4)], [W - fdiv(T, 2), E - fdiv(T, 4)], Math.max(1, fdiv(ss * 2, 3)));
    // Nähte am Wulst
    const step = Math.max(ss * 4, T * 2);
    const seam = lerpI(LEATHER, BLACK, 40, 100);
    for (let x = T; x < W - T; x += step) {
      draw.line(g, seam, [x, E - fdiv(T, 2) + ss], [x - ss, E + fdiv(T, 2) - ss], Math.max(1, fdiv(ss, 2)));
    }
  }

  // ============================================================= Brenner
  function flamePts(cx, by, hw, fh, sway, n = 14) {
    const right = [], left = [];
    for (let i = 0; i <= n; i++) {
      const u = i / n;
      const wdt = hw * Math.sqrt(Math.min(1.0, u * 5.0 + 0.04)) * (1.0 - u) ** 1.15;
      const x = cx + sway * u * u;
      const y = by - fh * u;
      right.push([x + wdt, y]);
      left.push([x - wdt, y]);
    }
    return right.concat(left.reverse());
  }

  /** Düse + Flamme w x w*3//2 (6 Flacker-Stufen); Anker Unterkante Mitte. */
  function burnerFlame(w, step) {
    w = Math.max(4, trunc(w));
    step = ((trunc(step) % 6) + 6) % 6;
    return cached(`flame|${w}|${step}`, () => {
      const fhImg = fdiv(w * 3, 2);
      const ss = 4;
      const W = w * ss, H = fhImg * ss;
      const [c, g] = ssCanvas(w, fhImg, ss);
      const cx = W / 2;
      const [hp, swayP, wp, tongue] = FLAME_STEPS[step];
      const noz = Math.max(ss * 2, fdiv(H * 9, 100));
      const by = H - noz;
      const fh = ((by - ss) * hp) / 100.0;
      const hw = (W * 0.36 * wp) / 100.0;
      const sway = (W * swayP) / 100.0;
      const poly = (col, pts) => replace(g, col, (cc) => fillPts(g, cc, pts));
      poly([255, 140, 40, 64], flamePts(cx, by, hw * 1.32, fh, sway));
      if (tongue) poly([255, 150, 44, 220], flamePts(cx + tongue * hw * 0.55, by - fh * 0.1, hw * 0.34, fh * 0.42, tongue * W * 0.1, 10));
      poly([255, 118, 30, 238], flamePts(cx, by, hw, fh * 0.94, sway));
      poly([255, 192, 58, 250], flamePts(cx, by, hw * 0.7, fh * 0.74, sway * 0.8));
      fillPts(g, [255, 242, 176], flamePts(cx, by, hw * 0.42, fh * 0.5, sway * 0.5));
      poly([96, 162, 255, 236], flamePts(cx, by, hw * 0.32, fh * 0.22, 0, 8));
      fillPts(g, [220, 238, 255], flamePts(cx, by, hw * 0.14, fh * 0.12, 0, 6));
      // Düse
      const nw = W * 0.4;
      rect(g, METAL_D, [cx - nw / 2, by, nw, noz], 0, ss);
      rect(g, METAL, [cx - nw / 2 + ss, by + fdiv(ss, 2), nw * 0.45, noz - ss], 0, fdiv(ss, 2));
      ellipse(g, [24, 24, 30], [cx - nw / 2 + ss, by - ss, nw - 2 * ss, ss * 2]);
      return c;
    });
  }

  // ============================================================= Fallschirm
  /** Fallschirm w x w: Kuppel mit Bahnen in col und Weiß, Leinen laufen unten Mitte zusammen. */
  function parachute(w, col) {
    w = Math.max(6, trunc(w));
    col = rgb(col);
    return cached(`chute|${w}|${col}`, () => {
      const ss = 3;
      const N = w * ss;
      const [c, g] = ssCanvas(w, w, ss);
      const cx = N / 2;
      const rx = N * 0.47;
      const top = N * 0.05;
      const hem = (N * CHUTE_HEM) / 100;
      const ry = hem - top;
      const conv = [cx, N - ss * 0.5];
      const np = 6;
      const bounds = [];
      for (let k = 0; k <= np; k++) bounds.push(-PI / 2 + (k * PI) / np);
      const white = [250, 248, 242];
      const dark = lerpI(col, BLACK, 45, 100);
      const mer = (sb, th) => [cx + rx * sb * Math.sin(th), hem - ry * Math.cos(th)];
      const gores = [];
      const silTop = [];
      for (let j = 0; j < np; j++) {
        const s0 = Math.sin(bounds[j]), s1 = Math.sin(bounds[j + 1]);
        const x0 = cx + rx * s0, x1 = cx + rx * s1;
        const scal = rx * (s1 - s0) * 0.16;
        const pts = [];
        for (let i = 0; i <= 10; i++) pts.push(mer(s0, ((PI / 2) * i) / 10));
        const edge = [];
        for (let i = 1; i < 8; i++) edge.push([x0 + ((x1 - x0) * i) / 8, hem - scal * Math.sin((PI * i) / 8)]);
        pts.push(...edge);
        for (let i = 10; i >= 0; i--) pts.push(mer(s1, ((PI / 2) * i) / 10));
        const mid = (s0 + s1) / 2;
        gores.push([lerpI(j % 2 === 0 ? col : white, BLACK, trunc(8 + 16 * (mid + 1)), 100), pts]);
        silTop.push([x0, hem], ...edge);
      }
      silTop.push([cx + rx, hem]);
      // Unterlage (ganze Kuppel), dann die Bahnen
      const under = [];
      for (let i = 10; i >= 0; i--) under.push(mer(-1, ((PI / 2) * i) / 10));
      for (let i = 1; i <= 10; i++) under.push(mer(1, ((PI / 2) * i) / 10));
      for (let i = silTop.length - 2; i > 0; i--) under.push(silTop[i]);
      fillPts(g, lerpI(dark, BLACK, 20, 100), under);
      for (const [cc, pts] of gores) fillPts(g, cc, pts);
      // Glanz links oben, Bahnkanten
      const band = arcBand(0, 0, 0.84, 0.16, PI * 1.14, PI * 1.4, 12);
      fillPts(g, [255, 255, 255, 96], band.map(([x, y]) => [cx + rx * x, hem + ry * y]));
      const edges = [];
      for (const b of bounds.slice(1, -1)) {
        const sb = Math.sin(b);
        const pts = [];
        for (let i = 1; i <= 10; i++) pts.push(mer(sb, ((PI / 2) * i) / 10));
        edges.push(pts);
      }
      strokeMany(g, lerpI(dark, BLACK, 20, 100), edges, Math.max(1, fdiv(ss, 2)));
      const outline = [];
      for (let i = 10; i >= 0; i--) outline.push(mer(-1.0, ((PI / 2) * i) / 10));
      for (let i = 1; i <= 10; i++) outline.push(mer(1.0, ((PI / 2) * i) / 10));
      draw.lines(g, dark, false, outline, Math.max(2, ss));
      draw.lines(g, dark, false, silTop, Math.max(1, fdiv(ss * 2, 3)));
      // Leinen
      const llw = Math.max(2, fdiv(ss * w, 70));
      const lines = bounds.map((b) => [[cx + rx * Math.sin(b), hem], conv]);
      g.beginPath();
      for (const [a, b] of lines) {
        g.moveTo(a[0], a[1]);
        g.lineTo(b[0], b[1]);
      }
      g.strokeStyle = css([74, 70, 82]);
      g.lineWidth = llw;
      g.lineCap = "butt";
      g.stroke();
      disc(g, [74, 70, 82], conv, llw);
      return c;
    });
  }

  /** Schirm in Öffnungsstufe k (0..8): erst ein schmaler Schlauch, dann bläht er sich auf. */
  function chuteStage(cw, col, k) {
    col = rgb(col);
    if (k >= 8) return parachute(cw, col);
    return cached(`chute_k|${cw}|${col}|${k}`, () => {
      const full = parachute(cw, col);
      const hem = fdiv(cw * CHUTE_HEM, 100);
      const f = k / 8.0;
      const sw = Math.max(2, trunc(cw * (0.16 + 0.8 * f)));
      const sh = Math.max(2, trunc((hem + 1) * (0.82 + 0.18 * f)));
      const [c, g] = canvas(cw, cw);
      const x0 = fdiv(cw - sw, 2);
      const s = scale();
      g.save();
      g.imageSmoothingQuality = "high";
      g.drawImage(full, 0, 0, full.width, Math.min(full.height, Math.round((hem + 1) * s)), x0, hem + 1 - sh, sw, sh);
      g.restore();
      const conv = [cw / 2.0, cw - 1];
      g.beginPath();
      for (const fx of [0.06, 0.36, 0.64, 0.94]) {
        g.moveTo(x0 + sw * fx + 0.5, hem + 0.5);
        g.lineTo(conv[0] + 0.5, conv[1] + 0.5);
      }
      g.strokeStyle = css([74, 70, 82]);
      g.lineWidth = 1;
      g.stroke();
      return c;
    });
  }

  // ============================================================= Geldsack
  /** Geldsack n x n (Anker Mitte); level 0..2 = voller (2: Münzen schauen heraus). */
  function moneySack(n, level) {
    n = Math.max(6, trunc(n));
    level = Math.max(0, Math.min(2, trunc(level)));
    return cached(`sack|${n}|${level}`, () => {
      const ss = 3;
      const N = n * ss;
      const [c, g] = ssCanvas(n, n, ss);
      const cx = N / 2;
      const brx = N * (0.31 + 0.045 * level);
      const bry = N * (0.25 + 0.025 * level);
      const bcy = N * 0.95 - bry;
      const neckY = N * 0.34;
      const nhw = N * 0.085;
      const right = [];
      for (let i = 0; i < 10; i++) {
        const t = i / 10;
        right.push([cx + nhw + (brx - nhw) * Math.sin((t * PI) / 2) ** 1.3, neckY + (bcy - neckY) * t]);
      }
      for (let i = 0; i < 11; i++) {
        const a = ((PI / 2) * i) / 10;
        right.push([cx + brx * Math.cos(a), bcy + bry * Math.sin(a)]);
      }
      const body = right.concat(right.slice().reverse().map(([x, y]) => [2 * cx - x, y]));
      // Münzen oben (nur voller Sack), dahinter der Zipfel
      if (level >= 2) {
        for (const [dx, dy, rot] of [[-0.07, 0.15, -0.3], [0.06, 0.12, 0.25], [0.0, 0.08, 0.0]]) {
          const pts = ellipsePts(cx + N * dx, N * dy, N * 0.075, N * 0.05, rot, 16);
          fillPts(g, GOLD_D, pts.map(([x, y]) => [x, y + ss]));
          fillPts(g, GOLD, pts);
          draw.line(g, GOLD_L, [cx + N * (dx - 0.04), N * (dy - 0.015)], [cx + N * (dx + 0.02), N * (dy - 0.03)], Math.max(1, ss));
        }
      }
      const tuft = [[cx - nhw, neckY], [cx - N * 0.17, N * 0.13], [cx - N * 0.09, N * 0.19], [cx - N * 0.04, N * 0.1],
        [cx + N * 0.03, N * 0.18], [cx + N * 0.1, N * 0.11], [cx + N * 0.16, N * 0.17], [cx + nhw, neckY]];
      fillPts(g, BURLAP_D, tuft);
      fillPts(g, BURLAP, tuft.map(([x, y]) => [x + (cx - x) * 0.18, y + (neckY - y) * 0.15]));
      // Körper mit Schatten und Jute-Muster (eigene Fläche, mit dem Umriss ausgestanzt)
      const [lay, lg] = ssCanvas(n, n, ss);
      fillPts(lg, BURLAP, body);
      const step = Math.max(ss * 2, fdiv(N, 14));
      const tex1 = [], tex2 = [];
      for (let i = -N; i < N; i += step) {
        tex1.push([[i, 0], [i + N, N]]);
        tex2.push([[i + N, 0], [i, N]]);
      }
      strokeMany(lg, [110, 74, 38, 46], tex1, Math.max(1, fdiv(ss, 2)));
      strokeMany(lg, [250, 220, 170, 30], tex2, Math.max(1, fdiv(ss, 2)));
      const [shc, shg] = ssCanvas(n, n, ss);
      radial(shg, [80, 46, 16], [cx - N * 0.14, bcy - bry * 0.55], N * 0.08, N * 0.8, 0, 150, 16, 1.4);
      const band = arcBand(cx, bcy, brx * 0.86, brx * 0.14, PI * 1.05, PI * 1.35, 10);
      replace(shg, [255, 236, 196, 70], (cc) => fillPts(shg, cc, band));
      blitRaw(lg, shc);
      keepInside(lg, body);
      blitRaw(g, lay);
      draw.lines(g, lerpI(BURLAP_D, BLACK, 30, 100), true, body, Math.max(2, ss));
      // Kordel mit Schleife
      const cord = [124, 82, 46];
      rect(g, lerpI(cord, BLACK, 35, 100), [cx - nhw * 1.45, neckY - N * 0.035, trunc(nhw * 2.9), trunc(N * 0.07)], 0, trunc(N * 0.03));
      rect(g, cord, [cx - nhw * 1.35, neckY - N * 0.03, trunc(nhw * 2.7), trunc(N * 0.05)], 0, trunc(N * 0.025));
      const lw = Math.max(ss, fdiv(N, 30));
      ellipse(g, cord, [cx + nhw * 1.1, neckY - N * 0.09, N * 0.1, N * 0.08], lw);
      stroke(g, cord, [[cx + nhw * 1.2, neckY], [cx + nhw * 1.9, neckY + N * 0.07], [cx + nhw * 1.7, neckY + N * 0.12]], lw);
      // goldene Münze als Wappen
      const r = N * (0.13 + 0.012 * level);
      const mc = [cx, bcy - bry * 0.08];
      disc(g, lerpI(GOLD_D, BLACK, 25, 100), [mc[0], mc[1] + ss], r);
      disc(g, GOLD_D, mc, r);
      disc(g, GOLD, mc, r * 0.84);
      draw.circle(g, lerpI(GOLD, GOLD_D, 1, 3), mc, r * 0.64, Math.max(1, fdiv(ss, 2) + 1));
      const star = [];
      for (let j = 0; j < 10; j++) {
        const a = -PI / 2 + (PI * j) / 5;
        const rr = r * (j % 2 === 0 ? 0.46 : 0.19);
        star.push([mc[0] + Math.cos(a) * rr, mc[1] + Math.sin(a) * rr]);
      }
      fillPts(g, GOLD_D, star);
      fillPts(g, GOLD_L, arcBand(mc[0], mc[1], r * 0.8, r * 0.16, PI * 1.05, PI * 1.45, 8));
      if (level >= 2) sparkle(g, WHITE, cx + N * 0.2, N * 0.08, N * 0.07);
      return c;
    });
  }

  // ============================================================= Platzen
  /** Umriss eines Gummifetzens in einer Fläche S x S (Streuwert v). */
  function shredPoly(S, v, flap) {
    const c = S / 2.0;
    const rot = ((((v >>> 5) % 360) * PI) / 180);
    const ca = Math.cos(rot), sa = Math.sin(rot);
    let pts = [];
    if (flap) {
      const k = 11;
      for (let j = 0; j < k; j++) {
        const a = (TAU * j) / k;
        let rr = c * (0.62 + ((v >>> ((j * 3) % 27)) % 34) / 100.0);
        if (j % 4 === 1) rr *= 0.62; // Riss-Kerbe
        pts.push([Math.cos(a) * rr, Math.sin(a) * rr * 0.74]);
      }
    } else {
      const ln = S * (0.84 + (v % 12) / 100.0);
      const th = S * (0.2 + ((v >>> 4) % 12) / 100.0);
      const bend = (((v >>> 9) % 100) / 100.0) * 1.2 - 0.6;
      const top = [], bot = [];
      for (let i = 0; i < 9; i++) {
        const t = i / 8.0 - 0.5;
        const x = t * ln, y = bend * (t * t - 0.125) * ln;
        const d = 2.0 * bend * t;
        const nl = Math.hypot(1.0, d);
        const hw = (th * (0.62 + 0.38 * Math.cos(t * PI)) * (0.86 + 0.14 * ((v >>> (i + 2)) % 2))) / 2;
        top.push([x + (d / nl) * hw, y - hw / nl]);
        bot.push([x - (d / nl) * hw, y + hw / nl]);
      }
      // ausgefranste Enden (Zickzack)
      const tear = (a, b, sign, seed) => {
        const out = [];
        for (const j of [1, 2, 3]) {
          const f = j / 4.0;
          const jag = th * (j % 2 ? 0.3 : -0.12) * (0.7 + 0.3 * ((seed >>> j) % 2));
          out.push([a[0] + (b[0] - a[0]) * f + sign * jag, a[1] + (b[1] - a[1]) * f]);
        }
        return out;
      };
      pts = top.concat(tear(top[8], bot[8], 1, v >>> 3), bot.slice().reverse(), tear(bot[0], top[0], -1, v >>> 7));
    }
    return pts.map(([x, y]) => [c + x * ca - y * sa, c + x * sa + y * ca]);
  }

  /** 8 Gummifetzen (abwechselnd RUBBER / stripe wie die Hülle), Größe ~ w/5 .. w/3. */
  function shreds(w, stripe) {
    w = Math.max(8, trunc(w));
    const st = stripeCol(stripe);
    return cached(`shreds|${w}|${st}`, () => {
      const out = [];
      for (let i = 0; i < 8; i++) {
        const v = hash(i * 7 + 3);
        const sz = Math.max(5, fdiv(w * (20 + (v % 13)), 100));
        const ss = 3;
        const S = sz * ss;
        const [c, g] = ssCanvas(sz, sz, ss);
        const col = i % 2 === 0 ? RUBBER : st;
        const pts = shredPoly(S, v, i % 3 === 2);
        fillPts(g, lerpI(col, BLACK, 50, 100), pts);
        const cc = S / 2.0;
        const inner = pts.map(([x, y]) => [cc + (x - cc) * 0.86 - S * 0.02, cc + (y - cc) * 0.8 - S * 0.03]);
        fillPts(g, col, inner);
        // Glanz: helle Linie nahe der Oberkante
        const gl = inner.slice(1, fdiv(inner.length, 3)).map(([x, y]) => [cc + (x - cc) * 0.55 - S * 0.03, cc + (y - cc) * 0.45 - S * 0.06]);
        if (gl.length >= 2) stroke(g, lerpI(col, WHITE, 60, 100), gl, Math.max(ss, fdiv(S, 14)));
        out.push(c);
      }
      return out;
    });
  }

  /** Comic-Knall 2r x 2r (ohne Text); step 0..5: wächst, dann hohl und verblasst. */
  function popStar(r, step) {
    r = Math.max(4, trunc(r));
    step = ((trunc(step) % 6) + 6) % 6;
    return cached(`pop|${r}|${step}`, () => {
      const ss = 3;
      const S = 2 * r * ss;
      const [c, g] = ssCanvas(2 * r, 2 * r, ss);
      const cc = S / 2.0;
      const sc = (POP_SCALE[step] / 100.0) * 0.86;
      const a = POP_ALPHA[step];
      const rot = 0.12 * step;
      const R = cc * sc;
      // Stern deckend auf eigener Fläche, dann als Ganzes mit a (Alpha ersetzen in Python)
      const [sc2, sg] = ssCanvas(2 * r, 2 * r, ss);
      const orange = burst(cc, cc, R, R * 0.54, 14, rot, 11);
      fillPts(sg, [255, 112, 36], orange);
      fillPts(sg, [255, 212, 62], burst(cc, cc, R * 0.74, R * 0.42, 14, rot + 0.11, 23));
      fillPts(sg, [255, 252, 230], burst(cc, cc, R * 0.44, R * 0.32, 10, rot + 0.2, 37, 0.15));
      // dunkler Rand verblasst schneller als der Stern (sonst bleibt ein dunkler Ring)
      fillPts(g, [120, 28, 18, fdiv(a * a, 255)], burst(cc, cc, R * 1.06, R * 0.58, 14, rot, 11));
      cut(g, (col) => fillPts(g, col, orange));
      blitRaw(g, sc2, a);
      if (step >= 4) {
        // hohl werden: Mitte ausstanzen
        const hole = burst(cc, cc, R * (step === 4 ? 0.3 : 0.56), R * (step === 4 ? 0.24 : 0.44), 10, rot, 41, 0.1);
        cut(g, (col) => fillPts(g, col, hole));
      }
      if (step >= 1 && step <= 3) {
        // Speed-Linien außen herum
        const lw = Math.max(ss, fdiv(S, 60));
        const segs = [];
        for (let j = 0; j < 10; j++) {
          const ang = (TAU * j) / 10 + 0.3 + step * 0.05;
          const r0 = cc * (step < 3 ? 0.9 : 0.93);
          const r1 = cc * 0.99;
          segs.push([[cc + Math.cos(ang) * r0, cc + Math.sin(ang) * r0], [cc + Math.cos(ang) * r1, cc + Math.sin(ang) * r1]]);
        }
        strokeMany(g, [255, 236, 150, a], segs, lw);
      }
      return c;
    });
  }

  // ============================================================= Verlauf (Pillen)
  /** [Hintergrund, Schrift]: rot < x2, grün ab x2, gold ab x10, violett ab x100. */
  function pillColor(m100) {
    if (m100 >= 10000) return [[142, 70, 222], [255, 255, 255]];
    if (m100 >= 1000) return [[222, 168, 48], [46, 28, 6]];
    if (m100 >= 200) return [[36, 148, 100], [240, 255, 246]];
    return [[168, 44, 60], [255, 236, 238]];
  }
  /** Größte fette Schrift (ab size abwärts), in der text in maxW passt. */
  function fitFont(text, maxW, size, minSize = 7) {
    size = trunc(size);
    while (size > minSize && ui.font(size, true).width(text) > maxW) size--;
    return ui.font(size, true);
  }
  /** Verlaufs-Pille w x h: Verlauf, Unterkante, Glanzlinie, Beschriftung. */
  function pill(w, h, m100, label, fontSize) {
    w = Math.max(6, trunc(w));
    h = Math.max(6, trunc(h));
    m100 = trunc(m100);
    label = String(label);
    return cached(`pill|${w}|${h}|${m100}|${label}|${trunc(fontSize)}`, () => {
      const [bg, fg] = pillColor(m100);
      const ss = 2;
      const W = w * ss, H = h * ss;
      const [c, g] = canvas(w, h);
      g.save();
      g.scale(1 / ss, 1 / ss);
      const rad = fdiv(H, 2);
      draw.rect(g, lerpI(bg, BLACK, 50, 100), [0, ss, W, H - ss], 0, rad);
      const hb = H - ss;
      const gr = g.createLinearGradient(0, 0, 0, Math.max(1, hb - 1));
      gr.addColorStop(0, css(lerpI(bg, WHITE, 28, 100)));
      gr.addColorStop(1, css(lerpI(bg, BLACK, 10, 100)));
      g.fillStyle = gr;
      ui.roundPath(g, 0, 0, W, hb, rad);
      g.fill();
      draw.line(g, lerpI(bg, WHITE, 52, 100), [rad * 0.8, ss * 1.5], [W - rad * 0.8, ss * 1.5], ss);
      if (m100 >= 1000) draw.rect(g, lerpI(bg, WHITE, 60, 100), [0, 0, W, H - ss], Math.max(1, fdiv(ss, 2)), rad);
      g.restore();
      const fnt = fitFont(label, w - Math.max(4, fdiv(h, 2)), fontSize);
      ui.text(g, label, fdiv(w, 2), fdiv(h - 1, 2), fnt, fg, "center");
      return c;
    }, small);
  }

  // ============================================================= Piktogramme
  /** Notenkopf (schräge Ellipse) mit Hals bis stemTop; liefert x des Halses. */
  function note(g, col, hx, hy, n, stemTop, lw) {
    fillPts(g, col, ellipsePts(hx, hy, n * 0.145, n * 0.105, -0.42, 20));
    const sx = hx + n * 0.118;
    draw.line(g, col, [sx, hy - n * 0.02], [sx, stemTop], Math.max(1, trunc(lw * 0.8)));
    return sx;
  }
  /** Umriss eines kleinen Ballons (Pol oben bei top) + y und halbe Breite des Munds. */
  function miniBalloon(cx, top, R) {
    const yeq = top + R;
    const L = R * 1.15;
    const m = R * 0.26;
    const right = [];
    for (let i = 0; i < 9; i++) {
      const a = ((PI / 2) * i) / 8;
      right.push([cx + R * Math.sin(a), yeq - R * Math.cos(a)]);
    }
    for (let i = 1; i < 9; i++) {
      const u = i / 8;
      right.push([cx + m + (R - m) * Math.max(0, Math.cos((u * PI) / 2)) ** 1.2, yeq + L * u]);
    }
    return [right.concat(right.slice().reverse().map(([x, y]) => [2 * cx - x, y])), yeq + L, m];
  }
  function iconBalloon(g, col, cx, top, R, n, lw, ss) {
    const [sil, ym, m] = miniBalloon(cx, top, R);
    fillPts(g, col, sil);
    const band = arcBand(cx, top + R, R * 0.76, R * 0.22, PI * 1.1, PI * 1.4, 10);
    cut(g, (cc) => fillPts(g, cc, band));
    const bw = R * 0.7;
    const by = ym + R * 0.34;
    const rl = Math.max(ss, fdiv(lw, 2));
    draw.line(g, col, [cx - m, ym], [cx - bw / 2, by], rl);
    draw.line(g, col, [cx + m, ym], [cx + bw / 2, by], rl);
    rect(g, col, [cx - bw / 2, by, bw, R * 0.46], 0, Math.max(1, ss * 2));
    return sil;
  }

  const ICONS = ["auto", "stats", "fair", "turbo", "music", "mute", "close", "left", "right", "check", "cross", "half", "ff", "play", "balloon", "two"];

  /** Piktogramm size x size (4-fach überabgetastet). */
  function icon(name, size, col) {
    size = Math.max(4, trunc(size));
    col = rgb(col);
    return cached(`icon|${name}|${size}|${col}`, () => {
      const ss = 4;
      const n = size * ss;
      const [c, s] = ssCanvas(size, size, ss);
      const lw = Math.max(ss, fdiv(n, 9));
      const poly = (pts) => fillPts(s, col, pts);
      if (name === "auto") {
        // Kreispfeil mit Spitze + Play-Dreieck in der Mitte
        const cc = fdiv(n, 2), r = fdiv(n * 2, 5);
        draw.arc(s, col, [cc - r, cc - r, 2 * r, 2 * r], 0.55, 5.75, lw);
        const tip = [cc + r * Math.cos(0.55), cc - r * Math.sin(0.55)];
        poly([[tip[0] - n * 0.16, tip[1] - n * 0.02], [tip[0] + n * 0.1, tip[1] - n * 0.06], [tip[0] - n * 0.02, tip[1] + n * 0.18]]);
        poly([[n * 0.42, n * 0.34], [n * 0.66, n * 0.5], [n * 0.42, n * 0.66]]);
      } else if (name === "stats") {
        const bw = fdiv(n, 5);
        [0.38, 0.62, 0.86].forEach((hh, i) => {
          const x = n * 0.14 + i * (bw + n * 0.09);
          rect(s, col, [x, n * (0.92 - hh), bw, n * hh], 0, Math.max(1, ss));
        });
      } else if (name === "fair") {
        const pts = [[n * 0.5, n * 0.06], [n * 0.88, n * 0.2], [n * 0.82, n * 0.6], [n * 0.5, n * 0.95], [n * 0.18, n * 0.6], [n * 0.12, n * 0.2]];
        pgLines(s, col, true, pts, lw);
        pgLines(s, col, false, [[n * 0.32, n * 0.5], [n * 0.46, n * 0.64], [n * 0.7, n * 0.36]], lw);
      } else if (name === "turbo") {
        poly([[n * 0.58, n * 0.04], [n * 0.2, n * 0.56], [n * 0.48, n * 0.56], [n * 0.38, n * 0.96], [n * 0.82, n * 0.4], [n * 0.53, n * 0.4]]);
      } else if (name === "music") {
        // zwei Achtelnoten mit Balken
        const x1 = note(s, col, n * 0.3, n * 0.75, n, n * 0.22, lw);
        const x2 = note(s, col, n * 0.71, n * 0.66, n, n * 0.13, lw);
        const hl = Math.max(ss, trunc(lw * 0.8)) / 2;
        const bt = n * 0.13;
        poly([[x1 - hl, n * 0.22], [x2 + hl, n * 0.13], [x2 + hl, n * 0.13 + bt], [x1 - hl, n * 0.22 + bt]]);
      } else if (name === "mute") {
        // Achtelnote, schräg durchgestrichen
        const x1 = note(s, col, n * 0.4, n * 0.72, n, n * 0.15, lw);
        stroke(s, col, [[x1, n * 0.15], [x1 + n * 0.13, n * 0.25], [x1 + n * 0.19, n * 0.38], [x1 + n * 0.14, n * 0.5]], Math.max(ss, trunc(lw * 0.8)));
        const a = [n * 0.15, n * 0.13], b = [n * 0.86, n * 0.87];
        cut(s, (cc) => stroke(s, cc, [a, b], trunc(lw * 2.4)));
        stroke(s, col, [a, b], lw);
      } else if (name === "close") {
        pgLine(s, col, [n * 0.22, n * 0.22], [n * 0.78, n * 0.78], lw);
        pgLine(s, col, [n * 0.78, n * 0.22], [n * 0.22, n * 0.78], lw);
      } else if (name === "left" || name === "right") {
        let pts = [[n * 0.66, n * 0.18], [n * 0.3, n * 0.5], [n * 0.66, n * 0.82]];
        if (name === "right") pts = pts.map(([x, y]) => [n - x, y]);
        poly(pts);
      } else if (name === "check") {
        pgLines(s, col, false, [[n * 0.16, n * 0.52], [n * 0.4, n * 0.76], [n * 0.84, n * 0.26]], Math.max(ss, fdiv(n, 7)));
      } else if (name === "cross") {
        const w2 = Math.max(ss, fdiv(n, 7));
        pgLine(s, col, [n * 0.2, n * 0.2], [n * 0.8, n * 0.8], w2);
        pgLine(s, col, [n * 0.8, n * 0.2], [n * 0.2, n * 0.8], w2);
      } else if (name === "half") {
        // "½" aus Strichen: 1, Bruchstrich, 2
        const lw2 = Math.max(ss, trunc(lw * 0.9));
        stroke(s, col, [[n * 0.16, n * 0.22], [n * 0.3, n * 0.11], [n * 0.3, n * 0.5]], lw2);
        stroke(s, col, [[n * 0.74, n * 0.1], [n * 0.27, n * 0.9]], lw2);
        const c2x = n * 0.69, c2y = n * 0.63, r2 = n * 0.13;
        const pts = [];
        for (let i = 0; i < 11; i++) {
          const a = PI * (1.06 + (1.12 * i) / 10);
          pts.push([c2x + r2 * Math.cos(a), c2y + r2 * Math.sin(a)]);
        }
        pts.push([n * 0.55, n * 0.9], [n * 0.85, n * 0.9]);
        stroke(s, col, pts, lw2);
      } else if (name === "ff") {
        poly([[n * 0.1, n * 0.2], [n * 0.5, n * 0.5], [n * 0.1, n * 0.8]]);
        poly([[n * 0.48, n * 0.2], [n * 0.88, n * 0.5], [n * 0.48, n * 0.8]]);
      } else if (name === "play") {
        poly([[n * 0.26, n * 0.14], [n * 0.84, n * 0.5], [n * 0.26, n * 0.86]]);
      } else if (name === "balloon") {
        iconBalloon(s, col, n * 0.5, n * 0.05, n * 0.31, n, lw, ss);
      } else if (name === "two") {
        // zwei Ballone: hinten links kleiner, vorn rechts größer (mit Lücke)
        iconBalloon(s, col, n * 0.33, n * 0.04, n * 0.21, n, lw, ss);
        const sil = miniBalloon(n * 0.66, n * 0.16, n * 0.245)[0];
        cut(s, (cc) => {
          fillPts(s, cc, sil);
          pgLines(s, cc, true, sil, Math.max(ss, lw));
        });
        iconBalloon(s, col, n * 0.66, n * 0.16, n * 0.245, n, lw, ss);
      }
      return c;
    }, small);
  }

  // ============================================================= Kurve
  /** Beschriftung mit weichem Schatten (gecacht). */
  function label(text, size) {
    return cached(`label|${text}|${trunc(size)}`, () => {
      const f = ui.font(size, true);
      const [c, g] = canvas(f.width(text) + 1, f.height + 1);
      ui.text(g, text, 1, 1, f, [0, 0, 0], "topleft", 150 / 255);
      ui.text(g, text, 0, 0, f, [238, 240, 252]);
      return c;
    }, small);
  }
  /** Hilfslinie (1 px, gestrichelt, halbdurchsichtig) als gecachter Streifen. */
  function gridStrip(length, vertical) {
    length = Math.max(1, trunc(length));
    return cached(`gstrip|${length}|${vertical ? 1 : 0}`, () => {
      const [c, g] = vertical ? canvas(1, length) : canvas(length, 1);
      g.fillStyle = css([255, 255, 255, 44]);
      for (let i = 0; i < length; i += 8) {
        if (vertical) g.fillRect(0, i, 1, Math.min(5, length - i));
        else g.fillRect(i, 0, Math.min(5, length - i), 1);
      }
      return c;
    }, small);
  }
  /** Hilfslinien: waagerecht bei ylines (Text links über der Linie), senkrecht bei xlines. */
  function drawGrid(s, r, ylines, xlines, fontSize) {
    r = r instanceof PG.Rect ? r : new PG.Rect(r);
    if (r.w <= 0 || r.h <= 0) return;
    const pad = Math.max(2, fdiv(trunc(fontSize), 3));
    if (ylines && ylines.length) {
      const hl = gridStrip(r.w, false);
      for (const [yy, lab] of ylines) {
        const y = trunc(yy);
        blit(s, hl, r.x, y);
        if (lab) {
          const img = label(lab, fontSize);
          blit(s, img, r.x + pad, y - img.lh);
        }
      }
    }
    for (const [xx, lab] of xlines || []) {
      const x = trunc(xx);
      const img = lab ? label(lab, fontSize) : null;
      const bottom = r.bottom - (img ? img.lh + pad : 0);
      if (bottom > r.y) {
        const vs = gridStrip(r.h, true);
        const hh = bottom - r.y;
        s.drawImage(vs, 0, 0, vs.width, Math.max(1, Math.round((vs.height * hh) / r.h)), x, r.y, 1, hh);
      }
      if (img) blit(s, img, x - fdiv(img.lw, 2), r.bottom - img.lh - 1);
    }
  }

  // wiederverwendete Arbeitsfläche für die Kurvenfläche (wächst nur)
  let work = null;
  function workCanvas(w, h) {
    const s = scale();
    const pw = Math.ceil(w * s), ph = Math.ceil(h * s);
    if (!work || work.s !== s || work.c.width < pw || work.c.height < ph) {
      const ow = work && work.s === s ? work.c.width : 0, oh = work && work.s === s ? work.c.height : 0;
      const c = ui.makeCanvas(Math.ceil(Math.max(pw, ow) / 64) * 64, Math.ceil(Math.max(ph, oh) / 64) * 64);
      work = { c, g: c.getContext("2d"), s };
    }
    const g = work.g;
    g.setTransform(1, 0, 0, 1, 0, 0);
    g.globalCompositeOperation = "source-over";
    g.globalAlpha = 1;
    g.clearRect(0, 0, pw + 2, ph + 2);
    g.setTransform(s, 0, 0, s, 0, 0);
    return work;
  }

  /**
   * Multiplikator-Kurve: Verlaufsfläche bis baseY (oben dichter), breites Leuchten,
   * kräftige Linie mit Glanz und hellem Kopf (nicht gecacht).
   */
  function drawCurve(s, pts, col, baseY, width) {
    if (!pts || pts.length < 2) return;
    col = rgb(col);
    width = Math.max(1, trunc(width));
    const by = ri(baseY);
    const step = Math.max(1, fdiv(pts.length + 239, 240)); // höchstens ~240 Stützpunkte
    const sel = [];
    for (let i = 0; i < pts.length; i += step) sel.push(pts[i]);
    if ((pts.length - 1) % step) sel.push(pts[pts.length - 1]);
    const p = sel.map(([x, y]) => [ri(x), ri(y)]);
    const g0 = width * 2 + 3;
    let xmin = Infinity, xmax = -Infinity, ymin = Infinity, ymax = -Infinity;
    for (const [x, y] of p) {
      if (x < xmin) xmin = x;
      if (x > xmax) xmax = x;
      if (y < ymin) ymin = y;
      if (y > ymax) ymax = y;
    }
    const left = xmin - g0, right = xmax + g0 + 1;
    const top = Math.min(ymin, by) - g0, bottom = Math.max(ymax, by) + 1;
    const light = lerpI(col, WHITE, 40, 100);
    const o = 0.5; // pygame-Pixelmitte
    if (right > left && bottom > top) {
      const bw = right - left, bh = bottom - top;
      const wk = workCanvas(bw, bh);
      const g = wk.g;
      const loc = p.map(([x, y]) => [x - left + o, y - top + o]);
      if (by > ymin) {
        // Fläche bis zur Grundlinie, Deckkraft von oben (~70) nach unten (0)
        const y0 = Math.max(0, ymin - top);
        const span = Math.max(1, by - top - y0);
        const bands = 24;
        const gr = g.createLinearGradient(0, y0, 0, y0 + span);
        for (let k = 0; k < bands; k++) {
          const a = fdiv(70 * (bands - k) * (bands - k), bands * bands);
          gr.addColorStop((k + 0.5) / bands, `rgba(${col[0]},${col[1]},${col[2]},${a / 255})`);
        }
        g.fillStyle = gr;
        path(g, loc.concat([[loc[loc.length - 1][0], by - top + o], [loc[0][0], by - top + o]]));
        g.fill();
      }
      // Leuchten (ersetzt die Fläche darunter wie in pygame)
      const st = Math.max(1, fdiv(loc.length + 95, 96)); // höchstens ~96 Punkte
      const gl = [];
      for (let i = 0; i < loc.length; i += st) gl.push(loc[i]);
      if ((loc.length - 1) % st) gl.push(loc[loc.length - 1]);
      for (const [hw, a] of [[width * 2.0, 70], [width * 1.0 + 1, 120]]) {
        replace(g, [light[0], light[1], light[2], a], (cc) => strokePts(g, cc, gl, hw * 2, true));
      }
      s.drawImage(wk.c, 0, 0, Math.ceil(bw * wk.s), Math.ceil(bh * wk.s), left, top, Math.ceil(bw * wk.s) / wk.s, Math.ceil(bh * wk.s) / wk.s);
    }
    // Linie wie pygame.draw.lines (Kreise auf den Punkten ab Breite 3), Glanzlinie, heller Kopf
    pgLines(s, col, false, p, width, width >= 3 ? width / 2 : 0);
    if (width >= 3) pgLines(s, light, false, p.map(([x, y]) => [x, y - fdiv(width, 4)]), Math.max(1, fdiv(width, 3)));
    const end = p[p.length - 1];
    pgDisc(s, col, end, width * 0.95 + 1);
    pgDisc(s, lerpI(col, WHITE, 75, 100), end, Math.max(1.0, width * 0.5));
  }

  /**
   * Lichthof mit Radius r, Fläche (2r+2)^2: Farbe col mit Deckkraft f^2 (12 Ringe wie in
   * Python, dort auf Schwarz) - additiv zeichnen (blitAdd) ergibt dasselbe wie BLEND_RGB_ADD.
   */
  function glow(r, col) {
    r = Math.max(1, trunc(r));
    col = rgb(col);
    return cached(`glow|${r}|${col}`, () => {
      const [c, g] = canvas(2 * r + 2, 2 * r + 2);
      const steps = 12;
      const cc = r + 1.5;
      const gr = g.createRadialGradient(cc, cc, 0, cc, cc, r);
      const rs = [], as = [];
      for (let i = 0; i < steps; i++) {
        const f = (i + 1) / steps;
        rs.push(Math.max(1, trunc(r * (1.0 - i / steps))));
        as.push(f * f);
      }
      // Ring i reicht bis rs[i] (rs fällt), innen deckender
      const stop = (rr, a) => gr.addColorStop(Math.max(0, Math.min(1, rr / r)), `rgba(${col[0]},${col[1]},${col[2]},${a})`);
      stop(0, 1);
      for (let i = steps - 1; i >= 0; i--) {
        const inner = i < steps - 1 ? rs[i + 1] : 0;
        stop((inner + rs[i]) / 2, as[i]);
      }
      stop(r, as[0]);
      g.fillStyle = gr;
      g.beginPath();
      g.arc(cc, cc, r, 0, TAU);
      g.fill();
      return c;
    });
  }

  /** Mini-Kurve w x h fürs Fairness-Fenster (bis zum Platzpunkt, marks = Ausstiege m100). */
  function curvePreview(w, h, crash100, marks) {
    const L = logic();
    w = Math.max(16, trunc(w));
    h = Math.max(12, trunc(h));
    crash100 = Math.max(100, Math.min(L.MAX_MULT100, trunc(crash100)));
    marks = (marks || []).map((v) => trunc(v));
    return cached(`preview|${w}|${h}|${crash100}|${marks.join(",")}`, () => {
      const ss = 2;
      const W = w * ss, H = h * ss;
      const [c, g] = ssCanvas(w, h, ss);
      const pad = Math.max(ss * 2, fdiv(Math.min(W, H), 12));
      const x0 = pad, y0 = pad + ss, x1 = W - pad * 2, y1 = H - pad;
      const moon = crash100 >= L.MAX_MULT100;
      const T = Math.max(1, L.popTick(crash100));
      const xs = Math.max(T, L.TPS * 4);
      const top = Math.max(crash100, 150);
      const px = (t) => x0 + fdiv((x1 - x0) * t, xs);
      const py = (m) => y1 - fdiv((y1 - y0) * (Math.min(m, crash100) - 100), top - 100);
      const ax = [255, 255, 255, 70];
      draw.line(g, ax, [x0, y0 - fdiv(pad, 2)], [x0, y1], ss);
      draw.line(g, ax, [x0, y1], [W - fdiv(pad, 2), y1], ss);
      const yc = py(crash100);
      const dashes = [];
      for (let x = x0 + ss * 2; x < x1; x += ss * 6) dashes.push([[x, yc + 0.5], [Math.min(x1, x + ss * 3), yc + 0.5]]);
      strokeMany(g, [255, 255, 255, 46], dashes, Math.max(1, fdiv(ss, 2)));
      const k = Math.max(8, fdiv(W, 3 * ss));
      const pts = [];
      for (let i = 0; i <= k; i++) {
        const t = fdiv(T * i, k);
        pts.push([px(t), py(L.multAt(t))]);
      }
      const line = moon ? GOLD : [255, 96, 112];
      const area = pts.concat([[pts[pts.length - 1][0], y1], [pts[0][0], y1]]);
      replace(g, [line[0], line[1], line[2], 56], (cc) => fillPts(g, cc, area));
      pgLines(g, line, false, pts, Math.max(ss, fdiv(H, 40)));
      const rm = Math.max(ss * 2, fdiv(H, 22));
      for (const mv of marks) {
        if (mv >= L.MIN_CASH && mv <= crash100) {
          const cc = [px(L.tickAt(mv)), py(mv)];
          disc(g, [12, 52, 30], cc, rm + ss);
          disc(g, [64, 214, 122], cc, rm);
          disc(g, [214, 255, 230], [cc[0] - fdiv(rm, 3), cc[1] - fdiv(rm, 3)], Math.max(1, fdiv(rm, 3)));
        }
      }
      const [ex, ey] = pts[pts.length - 1];
      const rs = Math.max(ss * 4, fdiv(Math.min(W, H), 9));
      if (moon) {
        fillPts(g, GOLD_D, burst(ex, ey, rs * 1.1, rs * 0.5, 5, -PI / 2, 5, 0.0));
        fillPts(g, GOLD, burst(ex, ey, rs * 0.9, rs * 0.4, 5, -PI / 2, 5, 0.0));
      } else {
        fillPts(g, [120, 28, 18], burst(ex, ey, rs * 1.08, rs * 0.56, 9, 0.2, 3));
        fillPts(g, [255, 112, 36], burst(ex, ey, rs, rs * 0.52, 9, 0.2, 3));
        fillPts(g, [255, 226, 90], burst(ex, ey, rs * 0.62, rs * 0.36, 9, 0.35, 9));
        disc(g, [255, 252, 230], [ex, ey], rs * 0.26);
      }
      return c;
    }, small);
  }

  // ============================================================= Mondlandung
  /** Fahne n x n: Stange mit Querstab, Tuch in col mit Falten und weißem Lama, Staubhügel. */
  function flag(n, col) {
    n = Math.max(8, trunc(n));
    col = rgb(col);
    return cached(`flag|${n}|${col}`, () => {
      const ss = 3;
      const N = n * ss;
      const [c, g] = ssCanvas(n, n, ss);
      const px = N * 0.2;
      const pw = Math.max(ss, fdiv(N, 22));
      const foot = N * 0.94;
      ellipse(g, [92, 92, 104], [px - N * 0.18, foot - N * 0.05, N * 0.36, N * 0.1]);
      ellipse(g, [150, 150, 162], [px - N * 0.15, foot - N * 0.065, N * 0.28, N * 0.07]);
      rect(g, [112, 116, 130], [px - pw / 2, N * 0.06, pw, foot - N * 0.06]);
      rect(g, [232, 234, 242], [px - pw / 2, N * 0.06, Math.max(1, fdiv(pw, 3)), foot - N * 0.06]);
      const fx0 = px + pw / 2, fx1 = N * 0.93;
      const fy0 = N * 0.1;
      const fh = N * 0.4;
      const topE = [], botE = [];
      for (let i = 0; i < 17; i++) {
        const f = i / 16;
        const x = fx0 + (fx1 - fx0) * f;
        const wave = Math.sin(f * PI * 3.0) * N * 0.02 * f;
        topE.push([x, fy0 + ss + wave * 0.25]);
        botE.push([x, fy0 + fh + wave - N * 0.03 * f]);
      }
      const cloth = topE.concat(botE.slice().reverse());
      const [lay, lg] = ssCanvas(n, n, ss);
      fillPts(lg, col, cloth);
      // Falten: senkrechte Streifen hell/dunkel
      for (let x = trunc(fx0); x < trunc(fx1) + 1; x += ss) {
        const f = (x - fx0) / Math.max(1.0, fx1 - fx0);
        const a = trunc(46 * Math.cos(f * PI * 3.0) * Math.min(1.0, f * 4.0));
        lg.fillStyle = css(a > 0 ? [255, 255, 255, a] : [0, 0, 0, -a]);
        lg.fillRect(x, 0, ss, N);
      }
      // Abzeichen: dunklerer Kreis mit weißem Lama
      const bc = [fx0 + (fx1 - fx0) * 0.5, fy0 + fh * 0.5];
      const br = fh * 0.34;
      disc(lg, lerpI(col, BLACK, 30, 100), bc, br);
      const e = Math.max(8, trunc(br * 1.6));
      const [lam, lmg] = ssCanvas(n, n, ss);
      lmg.translate(trunc(bc[0] - e * 0.55), trunc(bc[1] - e * 0.52));
      LCD().drawLlama(lmg, e, "stand", WHITE);
      lmg.setTransform(1, 0, 0, 1, 0, 0);
      lmg.globalCompositeOperation = "source-in";
      lmg.fillStyle = "#fff";
      lmg.fillRect(0, 0, lam.width, lam.height);
      blitRaw(lg, lam);
      keepInside(lg, cloth);
      blitRaw(g, lay);
      draw.lines(g, lerpI(col, BLACK, 45, 100), false, botE, Math.max(1, fdiv(ss * 2, 3)));
      draw.line(g, [206, 208, 220], [px, fy0], [fx1 + ss, fy0], Math.max(ss, fdiv(pw * 2, 3)));
      disc(g, [244, 244, 250], [px, N * 0.06], pw * 0.9);
      return c;
    });
  }

  // ============================================================= Zusammengesetztes
  /**
   * Ganzer Ballon über dem Korbrand (cx, rimY): Hülle + Seile, Flamme, Rückwand, Lamas,
   * Vorderwand. riders = [[scarf, pose] | null, ...] (1 = mittig, 2 = bei -/+ basketW/4),
   * flame 0..5 oder -1. Gibt das Rechteck der Hülle (mit Schürze, ohne Seile) zurück.
   */
  function drawRig(s, cx, rimY, base, level, stripe, riders, flame) {
    cx = trunc(cx);
    rimY = trunc(rimY);
    base = trunc(base);
    const img = balloon(base, level, stripe);
    const w = img.lw, h = img.lh;
    const x0 = cx - fdiv(w, 2), y0 = rimY + 1 - h;
    s.drawImage(img, x0, y0, w, h);
    const [mouth, , , mouthHw, frame, burnerH, , flameW] = rigDims(base);
    if (flame != null && flame >= 0) {
      const gl = glow(Math.max(3, fdiv(mouthHw * 5, 4)), flame % 2 === 0 ? [84, 46, 12] : [70, 38, 10]);
      blitAdd(s, gl, cx - fdiv(gl.lw, 2), rimY - mouth - fdiv(gl.lh, 2));
      const fl = burnerFlame(flameW, flame);
      s.drawImage(fl, cx - fdiv(fl.lw, 2), rimY - frame - burnerH + 1 - fl.lh, fl.lw, fl.lh);
    }
    const bw = basketW(base);
    const bx = cx - fdiv(bw, 2);
    const back = basket(bw, false);
    s.drawImage(back, bx, rimY, back.lw, back.lh);
    const n = llamaSize(base);
    const foot = riderFoot(rimY, base);
    const list = riders || [];
    const single = list.length === 1;
    for (let i = 0; i < Math.min(2, list.length); i++) {
      const rider = list[i];
      if (!rider) continue;
      const [scarf, pose] = rider;
      const dx = single ? 0 : i === 0 ? -fdiv(bw, 4) : fdiv(bw, 4);
      const spr = llamaSprite(n, pose, scarf);
      s.drawImage(spr, cx + dx - fdiv(n, 2), foot - n, n, n);
    }
    const front = basket(bw, true);
    s.drawImage(front, bx, rimY, front.lw, front.lh);
    return new PG.Rect(x0, y0, w, h - mouth);
  }

  /** Öffnungsstufe 0..8 des Schirms. */
  const openStage = (f) => Math.max(0, Math.min(8, trunc(Number(f) * 8 + 1e-6)));

  /** Lama mit Gurt am Schirm (Stufe k) - [Fläche, lx, ly] (Lage der Sprite-Ecke). */
  function jumperImg(n, scarf, chute, k) {
    const cw = Math.max(6, fdiv(n * 3, 2));
    const W = Math.max(cw, n) + 2;
    const hx = fdiv(W, 2);
    const lx = hx - fdiv(n * 49, 100);
    const ly = cw - 1 - fdiv(n * 40, 100);
    const img = cached(`jumper|${n}|${scarf}|${chute}|${k}`, () => {
      const [c, g] = canvas(W, ly + n + 1);
      const can = chuteStage(cw, chute, k);
      g.drawImage(can, hx - fdiv(cw, 2), 0, can.lw, can.lh);
      g.drawImage(llamaSprite(n, k < 4 ? "hop" : "stand", scarf), lx, ly, n, n);
      const strap = Math.max(1, fdiv(n, 16));
      draw.line(g, [54, 48, 60], [hx, ly + fdiv(n * 40, 100)], [hx - fdiv(n * 3, 100), ly + fdiv(n * 68, 100)], strap);
      disc(g, METAL_L, [hx + 0.5, ly + fdiv(n * 41, 100) + 0.5], Math.max(1, strap));
      return c;
    });
    return [img, lx, ly];
  }

  /** Lama am Fallschirm; (cx, cy) = Mitte des Lama-Sprites, openF 0..1. */
  function drawJumper(s, cx, cy, n, scarf, chute, openF) {
    n = Math.max(4, trunc(n));
    const [img, lx, ly] = jumperImg(n, rgb(scarf), rgb(chute), openStage(openF));
    s.drawImage(img, trunc(cx) - lx - fdiv(n, 2), trunc(cy) - ly - fdiv(n, 2), img.lw, img.lh);
  }

  /** Geldsack am Mini-Schirm; (cx, cy) = Mitte des Sacks, openF 0..1. */
  function drawSack(s, cx, cy, n, chute, openF) {
    n = Math.max(6, trunc(n));
    chute = rgb(chute);
    const k = openStage(openF);
    const cw = Math.max(6, fdiv(n * 13, 10));
    const W = Math.max(cw, n) + 2;
    const hx = fdiv(W, 2);
    const hook = fdiv(n * 10, 100);
    const sx = hx - fdiv(n, 2), sy = cw - 1 - hook;
    const img = cached(`sackchute|${n}|${chute}|${k}`, () => {
      const [c, g] = canvas(W, sy + n + 1);
      const can = chuteStage(cw, chute, k);
      g.drawImage(can, hx - fdiv(cw, 2), 0, can.lw, can.lh);
      const sack = moneySack(n, 1);
      g.drawImage(sack, sx, sy, sack.lw, sack.lh);
      return c;
    });
    s.drawImage(img, trunc(cx) - sx - fdiv(n, 2), trunc(cy) - sy - fdiv(n, 2), img.lw, img.lh);
  }

  /** Purzelndes Lama (Pose "fall"), in 8 Stufen zu 45° gedreht, Mitte (cx, cy). */
  function drawFaller(s, cx, cy, n, scarf, step) {
    n = Math.max(4, trunc(n));
    step = ((trunc(step) % 8) + 8) % 8;
    scarf = rgb(scarf);
    const img = cached(`faller|${n}|${scarf}|${step}`, () => {
      // Flächengröße wie pygame.transform.rotate (2n x 2n gedreht), dann halbiert
      const N = n * 2;
      let rw = N, rh = N;
      if (step % 2) {
        const a = (45 * step * PI) / 180;
        const ca = Math.cos(a), sa = Math.sin(a);
        const cxx = ca * N, sxx = sa * N;
        rw = trunc(Math.max(Math.abs(cxx + sxx), Math.abs(cxx - sxx)));
        rh = rw;
      }
      const [c, g] = canvas(Math.max(1, fdiv(rw, 2)), Math.max(1, fdiv(rh, 2)));
      g.translate(rw / 4, rh / 4);
      g.rotate((step * PI) / 4);
      g.scale(0.5, 0.5);
      g.translate(-n, -n);
      LCD().drawLlama(g, N, "fall", scarf);
      return c;
    });
    s.drawImage(img, trunc(cx) - fdiv(img.lw, 2), trunc(cy) - fdiv(img.lh, 2), img.lw, img.lh);
  }

  PG.lamaCrashDraw = {
    INFL_AT, INFL_MAX, RUBBER, RUBBER_D, RUBBER_DD, RUBBER_L, WICKER, WICKER_D, WICKER_DD, WICKER_L,
    LEATHER, LEATHER_L, ROPE, METAL, METAL_L, METAL_D, GOLD, GOLD_L, GOLD_D, BURLAP, BURLAP_L, BURLAP_D,
    WHITE, BLACK, GORES, RIDER_RIM, CHUTE_HEM, FLAME_STEPS, POP_SCALE, POP_ALPHA, ICONS,
    clearCache, inflLevel, basketW, llamaSize, balloonSize, balloon, basket, burnerFlame, parachute,
    moneySack, shreds, popStar, pillColor, pill, icon, drawGrid, drawCurve, glow, curvePreview, flag,
    drawRig, drawJumper, drawSack, drawFaller, blit, blitAdd,
  };
})();
