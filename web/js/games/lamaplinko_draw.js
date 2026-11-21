/*
 * lamaplinko_draw.js - Zeichen-Bausteine von Lama Plinko (Port von games/lamaplinko_draw.py)
 * ==========================================================================================
 * Alles aus Canvas-Pfaden, teure Teile werden einmal je Größe in Offscreen-Canvases
 * gerendert (x pixelScale) und gecacht - auch mit 50 Kugeln im Flug kostet ein Bild
 * nur ein paar hundert drawImage-Aufrufe:
 *
 * - background   : dunkler Spielhallen-Verlauf mit orangem Lichtschein hinter dem Brett
 *                  und feinem Punktraster,
 * - boardPlate   : halbdurchsichtige Trapez-Platte unter den Stiften,
 * - pegLayer     : alle Stifte eines Bretts auf einer Fläche (je Reihenzahl und Größe),
 *                  pegGlow = Leuchten eines getroffenen Stifts,
 * - yarnBall     : die Kugel - ein goldenes Wollknäuel in 12 Drehstufen;
 *                  trailDot = Leuchtpunkt der Spur,
 * - slotColor    : Farbe eines Fachs nach Multiplikator (Mitte kühl, außen heiß),
 * - slotSurface  : ein Fach mit Verlauf, Kante und Beschriftung, pill = Schild,
 * - funnel       : Einwurf-Trichter unter dem Lama,
 * - drawLlama    : das Lama oben am Brett (aus lamacrossing_draw.js), um die Hufe
 *                  gedreht (Nicken),
 * - icon         : Piktogramme der Knöpfe (Auto, Statistik, Fairness, Turbo, Schließen,
 *                  Pfeile, Haken, Kreuz),
 * - pathPreview  : Mini-Vorschau eines Kugelwegs (Fairness-Fenster).
 *
 * Alle Formen hängen nur von Größen und ganzzahligen Werten ab (kein Zufall, keine Zeit).
 * Farb-Verläufe rechnen wie im Original mit Ganzzahlen (lerpI).
 *
 * pygame zeichnet auf SRCALPHA-Flächen mit "Alpha ersetzen"; wo das Original
 * halbdurchsichtige Formen übereinander zeichnet, wird hier entweder ein
 * stufiger Radialverlauf benutzt (nestedCircles: gleiche Farbe, ineinander) oder
 * die Form erst ausgestanzt und dann neu gezeichnet (replace) - gleiches Ergebnis.
 */
(function () {
  "use strict";
  const { ui, draw } = PG;
  const LCD = PG.lamaCrossingDraw;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

  const GOLD = [236, 184, 64];
  const GOLD_L = [255, 228, 140];
  const GOLD_D = [168, 112, 30];
  const GOLD_DD = [110, 68, 18];
  const PEG = [226, 230, 246];
  const PEG_D = [120, 124, 150];
  const WHITE = [255, 255, 255];
  const BLACK = [0, 0, 0];

  // Fach-Farben: [Multiplikator x100, Farbe] - dazwischen linear (ganzzahlig)
  const SLOT_STOPS = [
    [0, [40, 96, 196]],
    [20, [52, 120, 222]],
    [50, [40, 170, 206]],
    [100, [70, 196, 140]],
    [150, [150, 210, 84]],
    [200, [236, 206, 60]],
    [500, [248, 154, 44]],
    [2000, [240, 86, 52]],
    [10000, [226, 40, 86]],
    [100000, [200, 50, 226]],
  ];

  const LLAMA_PIVOT = [0.42, 0.94]; // Drehpunkt (Hufe) relativ zur Sprite-Größe
  const LLAMA_MOUTH = [0.895, 0.225]; // Schnauzenspitze relativ zur Sprite-Größe (Blick nach rechts)

  // ------------------------------------------------------------ Cache
  const cache = new Map();
  const scale = () => (PG.app && PG.app.pixelScale) || 1;

  function cached(key, build) {
    const k = key + "|" + scale();
    let c = cache.get(k);
    if (!c) {
      if (cache.size > 600) cache.clear();
      c = build();
      cache.set(k, c);
    }
    return c;
  }
  function clearCache() {
    cache.clear();
  }

  /** Offscreen-Canvas w x h (logisch) in echter Pixeldichte; c.lw/c.lh = logische Größe. */
  function canvas(w, h) {
    const s = scale();
    const c = ui.makeCanvas(Math.max(1, Math.ceil(w * s)), Math.max(1, Math.ceil(h * s)));
    const g = c.getContext("2d");
    g.scale(s, s);
    c.lw = w;
    c.lh = h;
    return [c, g];
  }

  /** Kaufmännisch runden (x.5 nach oben), in Python, JS und Rust gleich. */
  const ri = (v) => Math.floor(v + 0.5);

  /** Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet). */
  function lerpI(c1, c2, num, den) {
    den = Math.max(1, den);
    return [0, 1, 2].map((i) => c1[i] + fdiv((c2[i] - c1[i]) * num, den));
  }

  /** Senkrechter Farbverlauf Zeile für Zeile (ganzzahlig gemischt), alpha 0..255. */
  function gradientRows(g, x, y, w, h, top, bottom, alpha = 255) {
    const hh = Math.max(1, h - 1);
    for (let yy = 0; yy < h; yy++) {
      const c = lerpI(top, bottom, yy, hh);
      g.fillStyle = `rgba(${c[0]},${c[1]},${c[2]},${alpha / 255})`;
      g.fillRect(x, y + yy, w, 1);
    }
  }

  /**
   * Konzentrische Kreise gleicher Farbe, in pygame von außen nach innen auf eine
   * SRCALPHA-Fläche gezeichnet ("Alpha ersetzen"): ein Pixel im Abstand d hat das
   * Alpha des kleinsten Kreises mit Radius >= d. rings = [[radius, alpha], ...] in
   * Zeichenreihenfolge. Hier als EIN stufiger Radialverlauf - jedes Pixel wird nur
   * einmal überblendet (viele Überblendungen mit Alpha 1/255 verlieren in Chrome
   * pro Schritt eine Farbstufe und färben den Schein dunkelrot).
   */
  function nestedCircles(g, rgb, cx, cy, rings) {
    const byR = new Map();
    for (const [r, a] of rings) byR.set(r, a); // gleicher Radius: der später gezeichnete gilt
    const rs = [...byR.entries()].sort((p, q) => p[0] - q[0]);
    const R = rs[rs.length - 1][0];
    const gr = g.createRadialGradient(cx, cy, 0, cx, cy, R);
    let prev = 0;
    for (const [r, a] of rs) {
      const col = `rgba(${rgb[0]},${rgb[1]},${rgb[2]},${a / 255})`;
      gr.addColorStop(prev / R, col);
      gr.addColorStop(r / R, col);
      prev = r;
    }
    g.fillStyle = gr;
    g.beginPath();
    g.arc(cx, cy, R, 0, TAU);
    g.fill();
  }

  /** "Alpha ersetzen" wie pygame auf SRCALPHA: Form ausstanzen, dann neu zeichnen. */
  function replace(g, paint, col) {
    g.save();
    g.globalCompositeOperation = "destination-out";
    paint([0, 0, 0, 255]);
    g.restore();
    paint(col);
  }

  /** Linienzug ohne Pixelmitten-Versatz (für überabgetastete Formen). */
  function polyline(g, col, pts, width) {
    g.beginPath();
    g.moveTo(pts[0][0], pts[0][1]);
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1]);
    g.strokeStyle = ui.col(col);
    g.lineWidth = width;
    g.lineJoin = "round";
    g.lineCap = "butt";
    g.stroke();
  }

  function pathOf(g, pts) {
    g.beginPath();
    g.moveTo(pts[0][0], pts[0][1]);
    for (let i = 1; i < pts.length; i++) g.lineTo(pts[i][0], pts[i][1]);
    g.closePath();
  }

  // ============================================================ Hintergrund
  /** Spielhallen-Hintergrund: Verlauf, Lichtschein hinter dem Brett, Raster, Vignette. */
  function background(w, h, glowCenter, glowR, accent) {
    const gx = trunc(glowCenter[0]), gy = trunc(glowCenter[1]);
    return cached(`bg|${w}|${h}|${gx}|${gy}|${glowR}|${accent.join(",")}`, () => {
      const [c, g] = canvas(w, h);
      gradientRows(g, 0, 0, w, h, [26, 16, 38], [8, 6, 16]);
      // weicher Lichtschein: konzentrische Kreise von außen nach innen, jeder
      // ersetzt die Mitte mit etwas mehr Deckkraft (Kegel ohne sichtbare Ringe)
      const steps = 40;
      const rgb = [accent[0], fdiv(accent[1], 2) + 40, fdiv(accent[2], 3)];
      const rings = [];
      for (let i = steps; i >= 1; i--) rings.push([Math.max(2, fdiv(glowR * i, steps)), fdiv((steps - i + 1) * 58, steps)]);
      nestedCircles(g, rgb, gx + 0.5, gy + 0.5, rings);
      // feines Punktraster
      const step = Math.max(14, fdiv(Math.min(w, h), 26));
      const ds = w >= 1200 ? 2 : 1;
      g.fillStyle = "rgb(40,30,56)";
      for (let yy = fdiv(step, 2); yy < h; yy += step) {
        for (let xx = fdiv(step, 2) + (fdiv(yy, step) % 2) * fdiv(step, 2); xx < w; xx += step) g.fillRect(xx, yy, ds, ds);
      }
      // dunkle Ränder (Vignette): 1-px-Rahmen nach innen immer heller
      const band = Math.max(8, fdiv(Math.min(w, h), 7));
      for (let i = 0; i < band; i++) {
        const a = fdiv(fdiv((band - i) * 90, band), 3);
        const rw = w - 2 * i, rh = h - 2 * i;
        if (rw <= 0 || rh <= 0) break;
        g.fillStyle = `rgba(0,0,0,${a / 255})`;
        g.fillRect(i, i, rw, 1);
        if (rh > 1) g.fillRect(i, h - i - 1, rw, 1);
        if (rh > 2) {
          g.fillRect(i, i + 1, 1, rh - 2);
          if (rw > 1) g.fillRect(w - i - 1, i + 1, 1, rh - 2);
        }
      }
      return c;
    });
  }

  /** Trapez-Platte unter den Stiften (points = 4 Ecken relativ zur Fläche w x h). */
  function boardPlate(points, w, h, accent) {
    const pts = points.map(([x, y]) => [trunc(x), trunc(y)]);
    return cached(`plate|${pts.join(";")}|${w}|${h}|${accent.join(",")}`, () => {
      const [c, g] = canvas(w, h);
      // Verlauf (oben warm, unten dunkel), mit dem Trapez ausgestanzt
      const top = [fdiv(accent[0], 4) + 22, fdiv(accent[1], 6) + 12, fdiv(accent[2], 6) + 26];
      const bot = [fdiv(accent[0], 9) + 14, fdiv(accent[1], 12) + 8, fdiv(accent[2], 10) + 22];
      const ys = pts.map((p) => p[1]);
      const y0 = Math.min(...ys), y1 = Math.max(...ys);
      const hh = Math.max(1, y1 - y0);
      g.save();
      pathOf(g, pts);
      g.clip();
      for (let y = y0; y <= y1; y++) {
        const cc = lerpI(top, bot, y - y0, hh);
        g.fillStyle = `rgba(${cc[0]},${cc[1]},${cc[2]},${150 / 255})`;
        g.fillRect(0, y, w, 1);
      }
      g.restore();
      const edge = ui.mix(accent, WHITE, 0.25);
      replace(g, (col) => draw.polygon(g, col, pts, 2), [edge[0], edge[1], edge[2], 80]);
      // Lichtkante oben + feiner Schimmer an den Schrägen
      const lt = ui.mix(accent, WHITE, 0.55);
      replace(g, (col) => draw.line(g, col, pts[0], pts[1], 2), [lt[0], lt[1], lt[2], 140]);
      const sh = ui.mix(accent, WHITE, 0.4);
      const shc = [sh[0], sh[1], sh[2], 50];
      replace(g, (col) => draw.line(g, col, [pts[0][0] + 3, pts[0][1] + 2], [pts[3][0] + 3, pts[3][1] - 2], 1), shc);
      replace(g, (col) => draw.line(g, col, [pts[1][0] - 3, pts[1][1] + 2], [pts[2][0] - 3, pts[2][1] - 2], 1), shc);
      return c;
    });
  }

  // ============================================================ Stifte
  /** Ein Stift: Schatten, Körper, Glanzpunkt. Fläche (2r+4)². */
  function pegSprite(r) {
    return cached("peg|" + r, () => {
      const size = 2 * r + 4;
      const [c, g] = canvas(size, size);
      const cc = size / 2;
      const R = r * 4; // Maße des 4-fach-Originals
      draw.circle(g, [0, 0, 0, 110], [cc + 1, cc + 2], r);
      draw.circle(g, PEG_D, [cc, cc], r);
      draw.circle(g, PEG, [cc - 0.5, cc - 0.5], r - 1);
      const o = fdiv(R, 3) / 4;
      draw.circle(g, WHITE, [cc - o, cc - o], Math.max(4, fdiv(R, 3)) / 4);
      return c;
    });
  }

  /** Alle Stifte (Liste ganzzahliger Mittelpunkte) auf einer Fläche w x h. */
  function pegLayer(w, h, pegs, r) {
    return cached(`pegs|${w}|${h}|${r}|${pegs.join(";")}`, () => {
      const [c, g] = canvas(w, h);
      const spr = pegSprite(r);
      const off = fdiv(spr.lw, 2);
      for (const [x, y] of pegs) g.drawImage(spr, x - off, y - off, spr.lw, spr.lh);
      return c;
    });
  }

  /** Leuchten eines getroffenen Stifts; level 1..8 (8 = frisch getroffen). Fläche 2R+2. */
  function pegGlow(r, level) {
    return cached(`glow|${r}|${level}`, () => {
      const R = trunc(r * 3.2) + 2;
      const [c, g] = canvas(2 * R + 2, 2 * R + 2);
      const cc = R + 1 + 0.5; // pygame-Kreis um die Pixelmitte
      const rings = [];
      for (let i = 6; i >= 1; i--) rings.push([Math.max(1, fdiv(R * i, 6)), fdiv((7 - i) * 10 * level, 8)]);
      nestedCircles(g, [255, 214, 120], cc, cc, rings);
      replace(g, (col) => draw.circle(g, col, [cc, cc], r + 1), [255, 250, 220, 40 + 26 * level]);
      return c;
    });
  }

  // ============================================================ Kugel
  /** Goldenes Wollknäuel mit Radius r in Drehstufe step (0..11). Fläche (2r+2)². */
  function yarnBall(r, step) {
    step = ((step % 12) + 12) % 12;
    return cached(`yarn|${r}|${step}`, () => {
      const size = 2 * r + 2;
      const [cv, g0] = canvas(size, size);
      const ss = 4;
      const R = r * ss;
      const c = fdiv(size * ss, 2);
      const g = g0;
      g.save();
      g.scale(1 / ss, 1 / ss); // ab hier in den Maßen des 4-fach-Originals
      g.save();
      g.beginPath();
      g.arc(c, c, R, 0, TAU);
      g.clip(); // Maske: alles außer dem Rand liegt im Kreis
      draw.circle(g, GOLD_D, [c, c], R);
      draw.circle(g, GOLD, [c - fdiv(R, 8), c - fdiv(R, 8)], R - fdiv(R, 7));
      // Wollfäden: gebogene Linien quer über das Knäuel, gedreht je Stufe
      const ang = (step * Math.PI) / 6;
      let ca = Math.cos(ang), sa = Math.sin(ang);
      const lw = Math.max(ss, fdiv(R, 6));
      const threads = [[-0.52, GOLD_DD], [-0.12, GOLD_D], [0.3, GOLD_DD], [0.66, GOLD_D], [0.05, GOLD_L]];
      threads.forEach(([off, col], i) => {
        const half = Math.sqrt(Math.max(0, 1 - off * off)) * 0.98;
        const pts = [];
        for (let j = 0; j < 9; j++) {
          const u = -half + (2 * half * j) / 8;
          const v = off + 0.22 * (half > 0 ? 1 - (u / half) ** 2 : 0);
          const x = u * R, y = v * R;
          pts.push([c + x * ca - y * sa, c + x * sa + y * ca]);
        }
        polyline(g, col, pts, i < 4 ? lw : Math.max(ss, fdiv(lw * 2, 3)));
      });
      // zweite Fadenlage quer dazu
      const ang2 = ang + Math.PI / 2.4;
      ca = Math.cos(ang2);
      sa = Math.sin(ang2);
      for (const off of [-0.34, 0.4]) {
        const half = Math.sqrt(Math.max(0, 1 - off * off)) * 0.98;
        const pts = [];
        for (let j = 0; j < 9; j++) {
          const u = -half + (2 * half * j) / 8;
          const v = off - 0.18 * (1 - (u / half) ** 2);
          const x = u * R, y = v * R;
          pts.push([c + x * ca - y * sa, c + x * sa + y * ca]);
        }
        polyline(g, GOLD_D, pts, Math.max(ss, fdiv(lw * 3, 4)));
      }
      // Licht oben links, Schatten unten rechts (fest, dreht nicht mit): Sichel
      g.save();
      g.beginPath();
      g.arc(c + fdiv(R, 3), c + fdiv(R, 3), R, 0, TAU);
      g.clip();
      g.beginPath();
      g.rect(0, 0, size * ss, size * ss);
      g.arc(c - fdiv(R, 6), c - fdiv(R, 6), fdiv(R * 4, 5), 0, TAU);
      g.fillStyle = ui.col([90, 50, 10, 90]);
      g.fill("evenodd");
      g.restore();
      draw.circle(g, [255, 246, 210], [c - fdiv(R * 2, 5), c - fdiv(R * 2, 5)], Math.max(ss, fdiv(R, 5)));
      g.restore(); // Maske
      draw.circle(g, [70, 40, 10], [c, c], R, Math.max(ss, fdiv(R, 9)));
      g.restore();
      return cv;
    });
  }

  /** Leuchtpunkt der Kugelspur (alpha in Stufen von 16). Fläche 2R+2. */
  function trailDot(r, alpha) {
    alpha = Math.max(16, Math.min(255, fdiv(alpha, 16) * 16));
    return cached(`trail|${r}|${alpha}`, () => {
      const R = Math.max(1, r);
      const [c, g] = canvas(2 * R + 2, 2 * R + 2);
      const cc = R + 1 + 0.5;
      draw.circle(g, [255, 196, 80, fdiv(alpha, 2)], [cc, cc], R);
      replace(g, (col) => draw.circle(g, col, [cc, cc], Math.max(1, fdiv(R * 3, 5))), [255, 230, 150, alpha]);
      return c;
    });
  }

  // ============================================================ Fächer
  /** Farbe eines Fachs: kühl (unter x1) über Gelb und Orange bis Magenta. */
  function slotColor(m100) {
    m100 = Math.max(0, trunc(Number(m100) || 0));
    for (let i = 0; i + 1 < SLOT_STOPS.length; i++) {
      const [a, ca] = SLOT_STOPS[i], [b, cb] = SLOT_STOPS[i + 1];
      if (m100 <= b) return lerpI(ca, cb, m100 - a, b - a);
    }
    return SLOT_STOPS[SLOT_STOPS.length - 1][1].slice();
  }

  /** Dunkle Schrift auf hellen Fächern, sonst weiße. */
  function textDark(col) {
    return fdiv(col[0] * 299 + col[1] * 587 + col[2] * 114, 1000) > 150;
  }

  /** Größte Schrift (ab size abwärts), in der text in maxW passt. */
  function fitFont(text, maxW, size, bold = true, minSize = 7) {
    size = trunc(size);
    while (size > minSize && ui.font(size, bold).width(text) > maxW) size--;
    return ui.font(size, bold);
  }

  /** Ein Fach w x (h+3): Verlauf in der Multiplikator-Farbe, Kante, Text. */
  function slotSurface(w, h, m100, label, fontSize, hot = false) {
    return cached(`slot|${w}|${h}|${m100}|${label}|${fontSize}|${hot ? 1 : 0}`, () => {
      let col = slotColor(m100);
      if (hot) col = ui.mix(col, WHITE, 0.28);
      const [c, g] = canvas(w, h + 3);
      const rad = Math.max(2, fdiv(Math.min(w, h), 4));
      draw.rect(g, ui.mix(col, BLACK, 0.45), [0, 3, w, h], 0, rad);
      g.save();
      ui.roundPath(g, 0, 0, w, h, rad);
      g.clip();
      gradientRows(g, 0, 0, w, h, ui.mix(col, WHITE, 0.3), ui.mix(col, BLACK, 0.12));
      g.restore();
      draw.rect(g, ui.mix(col, WHITE, 0.55), [rad, 1, Math.max(1, w - 2 * rad), 1]);
      if (hot) draw.rect(g, WHITE, [0, 0, w, h], 1, rad);
      const fg = textDark(col) ? [34, 22, 12] : WHITE;
      const fnt = fitFont(label, w - 2, fontSize);
      const tw = fnt.width(label);
      if (tw > w - 1) {
        // winzige Fächer: Text weich auf die Fachbreite verkleinern
        const f = (w - 1) / tw;
        g.save();
        g.translate(fdiv(w, 2), fdiv(h, 2));
        g.scale(f, f);
        ui.text(g, label, 0, 0, fnt, fg, "center");
        g.restore();
      } else {
        ui.text(g, label, fdiv(w, 2), fdiv(h, 2), fnt, fg, "center");
      }
      return c;
    });
  }

  /** Schild im Verlauf (Farbe wie das getroffene Fach). */
  function pill(w, h, m100, label, fontSize) {
    return cached(`pill|${w}|${h}|${m100}|${label}|${fontSize}`, () => {
      const col = slotColor(m100);
      const [c, g] = canvas(w, h);
      const rad = fdiv(h, 2);
      draw.rect(g, ui.mix(col, BLACK, 0.4), [0, 1, w, h - 1], 0, rad);
      draw.rect(g, col, [0, 0, w, h - 2], 0, rad);
      draw.rect(g, ui.mix(col, WHITE, 0.45), [rad, 1, Math.max(1, w - h), 1]);
      const fg = textDark(col) ? [34, 22, 12] : WHITE;
      const fnt = fitFont(label, w - 6, fontSize);
      ui.text(g, label, fdiv(w, 2), fdiv(h - 2, 2), fnt, fg, "center");
      return c;
    });
  }

  // ============================================================ Trichter + Lama
  /** Einwurf-Trichter (w breit oben, h hoch), Spitze unten in der Mitte. */
  function funnel(w, h, accent) {
    return cached(`funnel|${w}|${h}|${accent.join(",")}`, () => {
      const [c, g] = canvas(w, h);
      const ss = 3;
      const W = w * ss, H = h * ss;
      const mid = fdiv(W, 2);
      const neck = Math.max(ss * 4, fdiv(W * 2, 9));
      const rimH = Math.max(ss * 2, fdiv(H, 6));
      const cone = fdiv(H * 13, 20);
      g.save();
      g.scale(1 / ss, 1 / ss); // Maße des 3-fach-Originals
      // Trichter-Körper: Verlauf in Akzent-Dunkel, mit der Form ausgestanzt
      const body = [[0, fdiv(rimH, 2)], [W, fdiv(rimH, 2)], [mid + neck, cone], [mid + neck, H],
        [mid - neck, H], [mid - neck, cone]];
      g.save();
      pathOf(g, body);
      g.clip();
      gradientRows(g, 0, 0, W, H, ui.mix(accent, [40, 20, 30], 0.35), ui.mix(accent, [10, 6, 16], 0.75));
      g.restore();
      // dunkle Öffnung + Glanzstreifen links
      const inner = [[fdiv(W, 7), rimH], [W - fdiv(W, 7), rimH], [mid + fdiv(neck * 2, 3), cone], [mid - fdiv(neck * 2, 3), cone]];
      replace(g, (col) => draw.polygon(g, col, inner), [18, 10, 24, 220]);
      const gl = ui.mix(accent, WHITE, 0.5);
      replace(g, (col) => draw.line(g, col, [fdiv(W, 9), rimH + ss], [mid - neck + ss, cone], Math.max(ss, fdiv(W, 30))),
        [gl[0], gl[1], gl[2], 160]);
      // Ring am Hals und goldener Rand oben
      draw.rect(g, GOLD_D, [mid - neck - ss, cone, 2 * neck + 2 * ss, Math.max(ss * 2, fdiv(H, 9))], 0, ss);
      draw.rect(g, GOLD_D, [0, 0, W, rimH], 0, fdiv(rimH, 2));
      draw.rect(g, GOLD, [ss, 0, W - 2 * ss, rimH - ss], 0, fdiv(rimH, 2));
      draw.line(g, GOLD_L, [rimH, ss], [W - rimH, ss], Math.max(1, ss));
      g.restore();
      return c;
    });
  }

  /**
   * Linke obere Ecke eines um deg gedrehten Sprites (Größe rw x rh), sodass die
   * Hufe (LLAMA_PIVOT) bei (footX, footY) stehen (wie llama_blit_pos).
   */
  function llamaBlitPos(rw, rh, n, deg, footX, footY) {
    const a = (deg * Math.PI) / 180;
    const vx = (LLAMA_PIVOT[0] - 0.5) * n;
    const vy = (LLAMA_PIVOT[1] - 0.5) * n;
    const rx = vx * Math.cos(a) + vy * Math.sin(a);
    const ry = -vx * Math.sin(a) + vy * Math.cos(a);
    return [ri(footX - rx - rw / 2), ri(footY - ry - rh / 2)];
  }

  /**
   * Lama (Größe n, Schal in der Akzentfarbe) um deg Grad gedreht zeichnen
   * (negativ = nach vorn gebeugt), die Hufe bleiben bei (footX, footY).
   * pygame dreht gegen den Uhrzeigersinn, die Canvas im Uhrzeigersinn.
   */
  function drawLlama(ctx, n, deg, scarf, footX, footY) {
    const img = LCD.llamaCanvas(n, "stand", scarf);
    if (!deg) {
      const [x, y] = llamaBlitPos(n, n, n, 0, footX, footY);
      ctx.drawImage(img, x, y, n, n);
      return;
    }
    ctx.save();
    ctx.translate(footX, footY);
    ctx.rotate((-deg * Math.PI) / 180);
    ctx.drawImage(img, -LLAMA_PIVOT[0] * n, -LLAMA_PIVOT[1] * n, n, n);
    ctx.restore();
  }

  /** Bildschirmposition des Mauls (dort startet die Kugel). */
  function mouthPos(n, deg, footX, footY) {
    const a = (deg * Math.PI) / 180;
    const vx = (LLAMA_MOUTH[0] - LLAMA_PIVOT[0]) * n;
    const vy = (LLAMA_MOUTH[1] - LLAMA_PIVOT[1]) * n;
    return [footX + vx * Math.cos(a) + vy * Math.sin(a), footY - vx * Math.sin(a) + vy * Math.cos(a)];
  }

  // ============================================================ Piktogramme
  /** Piktogramm size x size: auto, stats, fair, turbo, close, left, right, check, cross. */
  function icon(name, size, col) {
    return cached(`icon|${name}|${size}|${col.join(",")}`, () => {
      const [c, g] = canvas(size, size);
      const ss = 4;
      const n = size * ss;
      const lw = Math.max(ss, fdiv(n, 9));
      g.save();
      g.scale(1 / ss, 1 / ss);
      if (name === "auto") {
        // Kreispfeil mit Spitze + Play-Dreieck in der Mitte
        const cc = fdiv(n, 2), r = fdiv(n * 2, 5);
        draw.arc(g, col, [cc - r, cc - r, 2 * r, 2 * r], 0.55, 5.75, lw);
        const tip = [cc + r * Math.cos(0.55), cc - r * Math.sin(0.55)];
        draw.polygon(g, col, [[tip[0] - n * 0.16, tip[1] - n * 0.02], [tip[0] + n * 0.1, tip[1] - n * 0.06],
          [tip[0] - n * 0.02, tip[1] + n * 0.18]]);
        draw.polygon(g, col, [[n * 0.42, n * 0.34], [n * 0.66, n * 0.5], [n * 0.42, n * 0.66]]);
      } else if (name === "stats") {
        const bw = fdiv(n, 5);
        [0.38, 0.62, 0.86].forEach((hh, i) => {
          const x = n * 0.14 + i * (bw + n * 0.09);
          draw.rect(g, col, [trunc(x), trunc(n * (0.92 - hh)), bw, trunc(n * hh)], 0, Math.max(1, ss));
        });
      } else if (name === "fair") {
        const pts = [[n * 0.5, n * 0.06], [n * 0.88, n * 0.2], [n * 0.82, n * 0.6], [n * 0.5, n * 0.95],
          [n * 0.18, n * 0.6], [n * 0.12, n * 0.2]];
        draw.polygon(g, col, pts, lw);
        polyline(g, col, [[n * 0.32, n * 0.5], [n * 0.46, n * 0.64], [n * 0.7, n * 0.36]], lw);
      } else if (name === "turbo") {
        draw.polygon(g, col, [[n * 0.58, n * 0.04], [n * 0.2, n * 0.56], [n * 0.48, n * 0.56],
          [n * 0.38, n * 0.96], [n * 0.82, n * 0.4], [n * 0.53, n * 0.4]]);
      } else if (name === "close") {
        polyline(g, col, [[n * 0.22, n * 0.22], [n * 0.78, n * 0.78]], lw);
        polyline(g, col, [[n * 0.78, n * 0.22], [n * 0.22, n * 0.78]], lw);
      } else if (name === "left" || name === "right") {
        let pts = [[n * 0.66, n * 0.18], [n * 0.3, n * 0.5], [n * 0.66, n * 0.82]];
        if (name === "right") pts = pts.map(([x, y]) => [n - x, y]);
        draw.polygon(g, col, pts);
      } else if (name === "check") {
        polyline(g, col, [[n * 0.16, n * 0.52], [n * 0.4, n * 0.76], [n * 0.84, n * 0.26]], Math.max(ss, fdiv(n, 7)));
      } else if (name === "cross") {
        const w2 = Math.max(ss, fdiv(n, 7));
        polyline(g, col, [[n * 0.2, n * 0.2], [n * 0.8, n * 0.8]], w2);
        polyline(g, col, [[n * 0.8, n * 0.2], [n * 0.2, n * 0.8]], w2);
      }
      g.restore();
      return c;
    });
  }

  // ============================================================ Fairness-Vorschau
  /** Mini-Brett w x h mit Stiften und dem Weg der Kugel (path = Liste 0/1). */
  function pathPreview(rows, path, w, h, m100 = null) {
    const p = (path || []).map((b) => trunc(Number(b) || 0));
    return cached(`preview|${rows}|${p.join("")}|${w}|${h}|${m100}`, () => {
      const [c, g] = canvas(w, h);
      const ss = 2;
      const W = w * ss, H = h * ss;
      g.save();
      g.scale(1 / ss, 1 / ss);
      const n = Math.max(1, rows);
      const gap = Math.min(W / (n + 2.4), H / (n + 1.6));
      const cx = W / 2;
      const top = gap * 0.7;
      const dot = Math.max(1, trunc(gap * 0.12));
      for (let r = 0; r < n; r++) {
        for (let j = 0; j < r + 3; j++) {
          const x = cx + (j - (r + 2) / 2) * gap;
          draw.circle(g, [150, 156, 190], [trunc(x), trunc(top + r * gap)], dot);
        }
      }
      const pts = [[cx, top - gap * 0.6]];
      let k = 0;
      for (let r = 0; r < n; r++) {
        pts.push([cx + (k - r / 2) * gap, top + r * gap - gap * 0.3]);
        if (r < p.length) k += p[r];
      }
      const sy = top + n * gap - gap * 0.2;
      const sx = cx + (k - n / 2) * gap;
      pts.push([sx, sy]);
      const col = m100 != null ? slotColor(m100) : GOLD;
      const bw = Math.max(2, trunc(gap * 0.9)), bh = Math.max(2, trunc(gap * 0.7));
      const box = [trunc(sx) - fdiv(bw, 2), trunc(sy + gap * 0.2) - fdiv(bh, 2), bw, bh];
      draw.rect(g, col, box, 0, Math.max(1, fdiv(bh, 3)));
      polyline(g, GOLD_L, pts.map(([x, y]) => [trunc(x), trunc(y)]), Math.max(2, trunc(gap * 0.16)));
      draw.circle(g, GOLD, [trunc(sx), trunc(sy)], Math.max(2, trunc(gap * 0.28)));
      g.restore();
      return c;
    });
  }

  PG.lamaPlinkoDraw = {
    GOLD, GOLD_L, GOLD_D, GOLD_DD, PEG, PEG_D, SLOT_STOPS, LLAMA_PIVOT, LLAMA_MOUTH,
    background, boardPlate, pegSprite, pegLayer, pegGlow, yarnBall, trailDot, slotColor, textDark, fitFont,
    slotSurface, pill, funnel, llamaBlitPos, drawLlama, mouthPos, icon, pathPreview, clearCache,
  };
})();
