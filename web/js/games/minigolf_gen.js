/*
 * minigolf_gen.js - Bahn-Generator für Minigolf (Port von games/minigolf_gen.py)
 * ==============================================================================
 * Aus einem Seed entstehen reproduzierbare Bahnen: Kurs 7, Bahn 3 sieht bei
 * jedem Start gleich aus - gespeichert werden muss dafür nichts. Die Tour
 * umfasst TOUR_COURSES Kurse zu je HOLES_PER_ROUND Bahnen; zusammen mit den
 * 18 handgebauten Bahnen aus minigolf.js ergibt das TOTAL_HOLES Bahnen.
 *
 * (Der Web-Zufallsgenerator ist ein anderer als Pythons random.Random - die
 * Tour-Bahnen sind deshalb deterministisch, aber nicht identisch mit der
 * Desktop-Version.)
 *
 * Passierbarkeit ist eingebaut, nicht erhofft. Jede Bahnfamilie legt zuerst
 * den Weg vom Abschlag zum Loch fest und baut die Hindernisse anschließend
 * darum herum:
 *  - Wandreihen lassen immer eine Lücke von mindestens MIN_PASS Einheiten.
 *  - Wasser liegt ausschließlich neben dem Weg, nie quer darüber.
 *  - Ein Wanderblock ist stets schmaler als seine Lücke.
 *  - Mühlenflügel sind kürzer als der halbe Korridor.
 *  - sanitize() entfernt zum Schluss alles, was auf Abschlag oder Loch liegt.
 *
 * Eine Bahn ist ein Objekt:
 *   {par, tee:[x,y], cup:[x,y], w, h, walls:[[x,y,w,h],...], sand, water, ...}
 */
(function () {
  "use strict";

  // ------------------------------------------------------------ Kursmaße
  // Einzige Definition der Platzmaße - minigolf.js importiert sie von hier.
  const CW = 100.0, CH = 160.0;   // Kursgröße in Bahn-Einheiten
  const BORDER = 3.0;              // Bandenbreite am Rand
  const BALL_R = 1.7;              // Ballradius

  // Innenfläche des Grüns (ohne Bande)
  const X0 = BORDER, X1 = CW - BORDER;
  const Y0 = BORDER, Y1 = CH - BORDER;

  const HOLES_PER_ROUND = 9;       // Bahnen je Runde
  const TOUR_COURSES = 38;         // Tour-Kurse (zusätzlich zu Classic + Pro)
  const HANDMADE_HOLES = 18;       // Classic + Pro in minigolf.js
  const TOTAL_HOLES = HANDMADE_HOLES + TOUR_COURSES * HOLES_PER_ROUND;

  const MIN_PASS = 12.0;           // schmalste Durchfahrt, die der Generator zulässt
  const SEED = 0x9e3779b1;         // Basis-Seed der Tour

  // Lückenbreite je Schwierigkeitsstufe (0 = leicht ... 3 = fies)
  const GAPS = [24.0, 21.0, 18.0, 15.0];

  // Alle Listen-Schlüssel einer Bahn, in der Reihenfolge von makeHole. Wer
  // über "alle Hindernisse" laufen will (Editor, Spiegeln, Speichern), nimmt
  // diese Liste - dann wird beim nächsten neuen Typ nichts vergessen.
  const HOLE_LISTS = ["walls", "sand", "water", "slopes", "bumpers", "movers", "mills",
    "tunnels", "ice", "boosters", "magnets", "gates", "sticky", "spinners", "jumps"];

  /**
   * Baut einen Bahn-Datensatz (gleiche Struktur wie die handgebauten Bahnen).
   *
   * Die sieben klassischen Typen:
   *   walls/sand/water : [x, y, w, h]
   *   slopes           : [x, y, w, h, ax, ay]          ax/ay = Beschleunigung
   *   bumpers          : [x, y, r]
   *   movers           : [x, y, w, h, dx, dy, speed]   pendelt zwischen den Enden
   *   mills            : [x, y, länge, arme, speed]   speed in rad/s
   * Dazu die acht Typen aus dem Bahn-Editor:
   *   tunnels  : [x1, y1, x2, y2, r]          Rohr-Paar
   *   ice      : [x, y, w, h]                 fast reibungsfrei
   *   boosters : [x, y, w, h, dx, dy, boost]  Einmal-Schub beim Betreten
   *   magnets  : [x, y, r, force]             zieht an (force > 0) / stößt ab
   *   gates    : [x, y, w, h, dx, dy]         Einbahn-Tor
   *   sticky   : [x, y, w, h]                 bremst extrem
   *   spinners : [x, y, r, speed]             Drehscheibe
   *   jumps    : [x, y, w, h, dx, dy, dist]   Sprungrampe
   *
   * opts: Objekt mit den Listen oben sowie w/h (Standard: CW x CH).
   */
  function makeHole(par, tee, cup, opts) {
    opts = opts || {};
    const hole = {
      par: Math.trunc(par), tee: [tee[0], tee[1]], cup: [cup[0], cup[1]],
      w: Number(opts.w == null ? CW : opts.w), h: Number(opts.h == null ? CH : opts.h),
    };
    for (const key of HOLE_LISTS) {
      hole[key] = (opts[key] || []).map((it) => it.map(Number));
    }
    return hole;
  }

  /**
   * Ergänzt fehlende Schlüssel einer Bahn (immer dasselbe Objekt zurück).
   *
   * Der Verträglichkeits-Riegel: ältere eigene Bahnen kennen die neuen
   * Hindernis-Typen und die Bahngröße noch nicht. Statt überall mit
   * "hole.x || []" zu hantieren, läuft jede Bahn einmal hier durch.
   */
  function normalize(hole) {
    if (!hole || typeof hole !== "object" || Array.isArray(hole)) {
      return makeHole(3, [CW / 2, CH - 18], [CW / 2, 22]);
    }
    if (hole.par == null) hole.par = 3;
    if (!Array.isArray(hole.tee)) hole.tee = [CW / 2, CH - 18];
    if (!Array.isArray(hole.cup)) hole.cup = [CW / 2, 22];
    for (const key of ["w", "h"]) {
      const v = parseFloat(hole[key]);
      hole[key] = isFinite(v) ? v : key === "w" ? CW : CH;
    }
    for (const key of HOLE_LISTS) {
      const items = hole[key];
      hole[key] = Array.isArray(items) ? items.filter(Array.isArray).map((it) => it.slice()) : [];
    }
    return hole;
  }

  /** Tiefe Kopie einer Bahn (Python: dict(h) + Listen-Kopien). */
  function cloneHole(hole) {
    return JSON.parse(JSON.stringify(hole));
  }

  // ---------------------------------------------------------------- Helfer

  function clampx(x, margin = 6.0) {
    return Math.max(X0 + margin, Math.min(X1 - margin, x));
  }

  function gapCenter(side, gap) {
    // Mitte der Lücke einer Wandreihe (side 0 = links, 1 = rechts).
    return side === 0 ? X0 + gap / 2 : X1 - gap / 2;
  }

  function row(side, gap, y, h = 7.0) {
    // Wandreihe über die volle Breite mit Lücke links oder rechts.
    const width = X1 - X0 - gap;
    return side === 0 ? [X0 + gap, y, width, h] : [X0, y, width, h];
  }

  function sandPatch(rng, x, y, w, h) {
    w = w != null ? w : rng.uniform(16, 26);
    h = h != null ? h : rng.uniform(12, 18);
    x = Math.max(X0 + 1, Math.min(X1 - 1 - w, x - w / 2));
    y = Math.max(Y0 + 1, Math.min(Y1 - 1 - h, y - h / 2));
    return [x, y, w, h];
  }

  function scatterSand(rng, hole, spots) {
    // Streut Sandflächen an vorgegebenen Stellen (Sand blockiert nie).
    for (const [x, y] of spots) {
      if (rng.random() < 0.65) hole.sand.push(sandPatch(rng, x, y));
    }
  }

  // Hindernisse, die den Ball wirklich aufhalten oder bestrafen - nur diese
  // dürfen nicht auf Abschlag oder Loch liegen.
  const BLOCKING_RECTS = ["walls", "water", "gates", "jumps"];

  /**
   * Entfernt Hindernisse auf Abschlag/Loch und klemmt beides ins Feld.
   * Es wird ausschließlich entfernt, nie hinzugefügt. Rechnet mit der
   * Bahngröße aus dem Datensatz (auch für Editor-Bahnen).
   */
  function sanitize(hole) {
    normalize(hole);
    const cw = hole.w, ch = hole.h;
    const x0 = BORDER, x1 = cw - BORDER, y0 = BORDER, y1 = ch - BORDER;
    const clamp = (v, lo, hi, margin) => Math.max(lo + margin, Math.min(hi - margin, v));
    hole.tee = [clamp(hole.tee[0], x0, x1, BALL_R + 3), clamp(hole.tee[1], y0, y1, BALL_R + 3)];
    hole.cup = [clamp(hole.cup[0], x0, x1, BALL_R + 5), clamp(hole.cup[1], y0, y1, BALL_R + 5)];
    const pts = [hole.tee, hole.cup];
    const clear = BALL_R + 3.0;

    const rectFree = (r) => {
      const [x, y, w, h] = r;
      return pts.every(([px, py]) => !(x - clear < px && px < x + w + clear && y - clear < py && py < y + h + clear));
    };
    const circleFree = (cx, cy, r) => pts.every(([px, py]) => Math.hypot(cx - px, cy - py) > r + clear);

    for (const key of BLOCKING_RECTS) hole[key] = hole[key].filter(rectFree);
    hole.bumpers = hole.bumpers.filter((b) => circleFree(b[0], b[1], b[2]));
    hole.mills = hole.mills.filter((m) => circleFree(m[0], m[1], m[2]));
    hole.spinners = hole.spinners.filter((s) => circleFree(s[0], s[1], s[2]));
    // Rohre dürfen nicht am Loch enden und nicht auf dem Abschlag liegen.
    hole.tunnels = hole.tunnels.filter((t) => circleFree(t[0], t[1], t[4]) && circleFree(t[2], t[3], t[4]));
    hole.movers = hole.movers.filter(([x, y, w, h, dx, dy]) => rectFree([x, y, w, h]) && rectFree([x + dx, y + dy, w, h]));
    return hole;
  }

  // ------------------------------------------------------------ Bahnfamilien
  //
  // Jede Familie bekommt (rng, tier, idx) und liefert eine fertige Bahn.
  // tier 0..3 = Schwierigkeitsstufe des Kurses, idx = Bahnnummer 0..8.

  function fStraight(rng, tier, idx) {
    // Gerade Bahn mit Trichter - der ruhige Auftakt jedes Kurses.
    const cx = rng.uniform(38, 62);
    const gap = GAPS[tier] + 4;
    const ymid = rng.uniform(66, 88);
    const half = (X1 - X0 - gap) / 2;
    const walls = [[X0, ymid, half, 8], [X1 - half, ymid, half, 8]];
    const hole = makeHole(tier < 2 ? 2 : 3, [cx, 142.0], [cx, rng.uniform(20, 30)], { walls });
    scatterSand(rng, hole, [[X0 + 14, ymid + 26], [X1 - 14, ymid + 26]]);
    if (tier >= 2 && rng.random() < 0.6) hole.bumpers.push([cx, ymid - 22, 6.0]);
    return sanitize(hole);
  }

  function fGates(rng, tier, idx) {
    // Wandreihen mit versetzten Lücken - der Zickzack-Klassiker.
    const n = Math.max(1, Math.min(4, 1 + tier + (idx >= 6 ? 1 : 0)));
    const gap = GAPS[tier];
    const ys = n === 1 ? [92.0] : Array.from({ length: n }, (_, i) => 126.0 - i * (92.0 / (n - 1)));
    const side = rng.randint(0, 1);
    const walls = [], sides = [];
    ys.forEach((y, i) => {
      const s = (side + i) % 2;
      sides.push(s);
      walls.push(row(s, gap, y));
    });
    const tee = [gapCenter(sides[0], gap), 146.0];
    const top = ys[ys.length - 1];
    let cupX;
    if (tier >= 2 && rng.random() < 0.5) cupX = gapCenter(1 - sides[sides.length - 1], gap); // Loch auf der Gegenseite
    else cupX = gapCenter(sides[sides.length - 1], gap);
    const hole = makeHole(Math.min(5, 2 + n), tee, [cupX, Math.max(16.0, top - 16.0)], { walls });
    ys.slice(0, -1).forEach((y, i) => scatterSand(rng, hole, [[gapCenter(1 - sides[i], gap), y - 14]]));
    return sanitize(hole);
  }

  function fDogleg(rng, tier, idx) {
    // Lange Wand quer im Feld - der Weg führt außen herum.
    let xw = rng.uniform(38, 58);
    const ytop = rng.uniform(32, 44);
    const ylen = rng.uniform(72, 96);
    const flip = rng.random() < 0.5;
    if (flip) xw = CW - xw - 9;
    const walls = [[xw, ytop, 9.0, ylen]];
    // Optionaler Riegel im unteren Feld, damit der Bogen größer wird
    if (tier >= 2) {
      const by = rng.uniform(96, 116);
      if (flip) walls.push([X0, by, rng.uniform(22, 30), 8.0]);
      else {
        const w = rng.uniform(22, 30);
        walls.push([X1 - w, by, w, 8.0]);
      }
    }
    const teeX = !flip ? xw / 2 : (xw + 9 + X1) / 2;
    const cupX = !flip ? (xw + 9 + X1) / 2 : xw / 2;
    const hole = makeHole(tier < 2 ? 3 : 4, [clampx(teeX), 142.0], [clampx(cupX), Math.max(16.0, ytop - 14.0)], { walls });
    scatterSand(rng, hole, [[xw + 4.5, ytop + ylen + 12]]);
    if (tier >= 1 && rng.random() < 0.5) {
      hole.bumpers.push([clampx(cupX + rng.choice([-18, 18])), ytop + 18, 6.0]);
    }
    return sanitize(hole);
  }

  function fBumpers(rng, tier, idx) {
    // Offenes Feld voller Gummipuffer - blockieren kann hier nichts.
    const cx = rng.uniform(40, 60);
    const k = 3 + tier;
    const bumpers = [];
    for (let i = 0; i < k * 4; i++) {
      if (bumpers.length >= k) break;
      const x = rng.uniform(X0 + 10, X1 - 10);
      const y = rng.uniform(40, 122);
      const r = rng.uniform(5, 7);
      if (bumpers.every((b) => Math.hypot(x - b[0], y - b[1]) > r + b[2] + 8)) bumpers.push([x, y, r]);
    }
    const hole = makeHole(3, [cx, 142.0], [clampx(cx + rng.uniform(-14, 14)), rng.uniform(18, 28)], { bumpers });
    scatterSand(rng, hole, [[cx, 118.0]]);
    return sanitize(hole);
  }

  function fWater(rng, tier, idx) {
    // Teiche links und rechts einer trockenen Gasse.
    const lane = GAPS[tier] + 8;
    const cx = rng.uniform(34, 66);
    const y0 = rng.uniform(48, 66);
    const h = rng.uniform(30, 46);
    const leftW = cx - lane / 2 - X0;
    const rightW = X1 - (cx + lane / 2);
    const water = [];
    if (leftW > 8) water.push([X0, y0, leftW, h]);
    if (rightW > 8) water.push([cx + lane / 2, y0, rightW, h]);
    let walls = [];
    if (tier >= 2) {
      // zusätzliche Engstelle über dem Wasser
      const gap = Math.max(MIN_PASS + 3, GAPS[tier]);
      const gx = clampx(cx + rng.uniform(-8, 8), gap / 2 + 2);
      walls = [[X0, y0 - 16, Math.max(2.0, gx - gap / 2 - X0), 7.0],
        [gx + gap / 2, y0 - 16, Math.max(2.0, X1 - gx - gap / 2), 7.0]];
    }
    const hole = makeHole(tier < 2 ? 3 : 4, [cx, 144.0], [clampx(cx + rng.uniform(-10, 10)), rng.uniform(18, 30)], { walls, water });
    scatterSand(rng, hole, [[cx, y0 + h + 16]]);
    return sanitize(hole);
  }

  function fRamp(rng, tier, idx) {
    // Steigung, die zurückschiebt - hier braucht es Kraft.
    const y0 = rng.uniform(40, 56);
    const h = rng.uniform(44, 60);
    const x0 = rng.uniform(16, 26);
    const w = rng.uniform(52, 66);
    const accel = 20.0 + tier * 5.0;
    const cx = clampx(x0 + w / 2 + rng.uniform(-10, 10));
    const hole = makeHole(tier < 2 ? 3 : 4, [cx, 146.0], [cx, rng.uniform(16, 26)], { slopes: [[x0, y0, w, h, 0.0, accel]] });
    scatterSand(rng, hole, [[x0 + 8, y0 + h + 14], [x0 + w - 8, y0 + h + 14]]);
    if (tier >= 2) hole.bumpers.push([clampx(cx + rng.choice([-24, 24])), y0 - 12, 6.0]);
    return sanitize(hole);
  }

  function fChicane(rng, tier, idx) {
    // Zwei stehende Wände - der Ball schlängelt sich durch.
    const xa = rng.uniform(26, 36);
    const xb = rng.uniform(60, 70);
    const ya = rng.uniform(34, 46), la = rng.uniform(52, 66);
    const yb = rng.uniform(84, 96), lb = rng.uniform(48, 62);
    const walls = [[xa, ya, 8.0, la], [xb, yb, 8.0, lb]];
    const hole = makeHole(4, [clampx(xa / 2), 146.0], [clampx((xb + 8 + X1) / 2), rng.uniform(16, 26)], { walls });
    scatterSand(rng, hole, [[xa + 22, ya + la + 10]]);
    if (tier >= 3 && rng.random() < 0.5) hole.bumpers.push([clampx((xa + xb) / 2), (ya + yb) / 2, 6.0]);
    return sanitize(hole);
  }

  function fMill(rng, tier, idx) {
    // Korridor mit Windmühle(n) - reines Timing.
    const w = 42.0 - tier * 3.0;
    const xl = 50.0 - w / 2;
    const xr = 50.0 + w / 2;
    const ytop = 22.0, ylen = 100.0;
    const walls = [[xl - 8, ytop, 8.0, ylen], [xr, ytop, 8.0, ylen]];
    const arm = w / 2 - 4.5; // waagerecht bleibt seitlich Platz
    const n = tier < 2 ? 1 : 2;
    const mills = [];
    for (let i = 0; i < n; i++) {
      const y = 104.0 - i * 44.0;
      const speed = rng.choice([-1.0, 1.0]) * rng.uniform(1.1, 1.9);
      mills.push([50.0, y, arm, 2, speed]);
    }
    const hole = makeHole(3 + n, [50.0, 146.0], [50.0, 16.0], { walls, mills });
    scatterSand(rng, hole, [[50.0, 130.0]]);
    return sanitize(hole);
  }

  function fIsland(rng, tier, idx) {
    // Inselgrün: Wasser ringsum, ein schmaler Hals führt hinauf.
    const cx = rng.uniform(38, 62);
    const cy = rng.uniform(30, 42);
    const halfW = 15.0, halfH = 13.0;
    const neck = Math.max(MIN_PASS + 2, Math.min(GAPS[tier], 2 * halfW - 6));
    const top = cy - halfH;
    const bot = cy + halfH;
    const water = [];
    const lw = cx - halfW - X0;
    const rw = X1 - (cx + halfW);
    if (lw > 8) water.push([X0, top, lw, 2 * halfH]);
    if (rw > 8) water.push([cx + halfW, top, rw, 2 * halfH]);
    const blw = cx - neck / 2 - X0;
    const brw = X1 - (cx + neck / 2);
    if (blw > 8) water.push([X0, bot, blw, 15.0]);
    if (brw > 8) water.push([cx + neck / 2, bot, brw, 15.0]);
    const hole = makeHole(tier < 2 ? 3 : 4, [clampx(cx), 146.0], [cx, cy], { water });
    scatterSand(rng, hole, [[cx, bot + 34]]);
    if (tier >= 2) hole.slopes.push([cx - 26, bot + 22, 52.0, 30.0, 0.0, -10.0]);
    return sanitize(hole);
  }

  function fMover(rng, tier, idx) {
    // Wandreihe mit Wanderblock - die Lücke ist immer irgendwo offen.
    const bw = 14.0;
    const gap = bw + 2 * MIN_PASS + 2.0; // selbst mittig bleibt links/rechts Platz
    const gx = clampx(rng.uniform(34, 66), gap / 2 + 3) - gap / 2;
    const y = rng.uniform(78, 104);
    const leftW = gx - X0;
    const rightW = X1 - (gx + gap);
    const walls = [];
    if (leftW > 2) walls.push([X0, y, leftW, 8.0]);
    if (rightW > 2) walls.push([gx + gap, y, rightW, 8.0]);
    const speed = 18.0 + tier * 4.0;
    const movers = [[gx, y, bw, 8.0, gap - bw, 0.0, speed]];
    const cupX = clampx(gx + gap / 2 + rng.uniform(-16, 16));
    const hole = makeHole(4, [clampx(gx + gap / 2), 146.0], [cupX, rng.uniform(18, 30)], { walls, movers });
    scatterSand(rng, hole, [[gx + gap / 2, y + 28]]);
    if (tier >= 3 && rng.random() < 0.5) hole.bumpers.push([clampx(cupX + rng.choice([-20, 20])), y - 26, 6.0]);
    return sanitize(hole);
  }

  const FAMILIES = {
    straight: fStraight, gates: fGates, dogleg: fDogleg, bumpers: fBumpers, water: fWater,
    ramp: fRamp, chicane: fChicane, mill: fMill, island: fIsland, mover: fMover,
  };

  // Welche Familien ab welcher Stufe vorkommen. Jeder Topf ist größer als die
  // acht zu besetzenden Plätze eines Kurses.
  const POOLS = [
    ["gates", "dogleg", "bumpers", "straight", "water", "ramp"],
    ["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "island"],
    ["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island", "mover"],
    ["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island", "mover"],
  ];
  // Der Abschluss jedes Kurses kommt aus dem "großen" Topf
  const FINALE = ["mill", "island", "mover", "gates", "chicane"];

  /** Schwierigkeitsstufe 0..3 eines Tour-Kurses (1..TOUR_COURSES). */
  function tierOf(course) {
    return Math.max(0, Math.min(3, Math.floor((Math.trunc(course) - 1) / 10)));
  }

  function familyFor(course, idx, rng, used) {
    if (idx === 0) return "straight";
    const tier = tierOf(course);
    const pool = (idx < HOLES_PER_ROUND - 1 ? POOLS[tier] : FINALE.slice(0, 3 + tier)).slice();
    rng.shuffle(pool);
    for (const name of pool) {
      // höchstens zweimal dieselbe Familie
      if ((used[name] || 0) < 2) return name;
    }
    return pool[0];
  }

  /** Erzeugt Bahn idx (0..8) des Tour-Kurses course (1..TOUR_COURSES). */
  function generate(course, idx, used) {
    course = Math.max(1, Math.min(TOUR_COURSES, Math.trunc(course)));
    idx = Math.max(0, Math.min(HOLES_PER_ROUND - 1, Math.trunc(idx)));
    const rng = new PG.Random((SEED + course * 9176 + idx * 131) >>> 0);
    const name = familyFor(course, idx, rng, used || {});
    if (used) used[name] = (used[name] || 0) + 1;
    const hole = FAMILIES[name](rng, tierOf(course), idx);
    hole.family = name;
    return hole;
  }

  /** Alle neun Bahnen eines Tour-Kurses, nach Par sortiert (stabil). */
  function courseHoles(course) {
    const used = {};
    const holes = [];
    for (let i = 0; i < HOLES_PER_ROUND; i++) holes.push(generate(course, i, used));
    const rest = holes.slice(1).sort((a, b) => a.par - b.par);
    return [holes[0]].concat(rest);
  }

  /** Gesamt-Par eines Tour-Kurses. */
  const parCache = {};
  function coursePar(course) {
    if (parCache[course] == null) parCache[course] = courseHoles(course).reduce((s, h) => s + h.par, 0);
    return parCache[course];
  }

  PG.minigolfGen = {
    CW, CH, BORDER, BALL_R, X0, X1, Y0, Y1, HOLES_PER_ROUND, TOUR_COURSES, HANDMADE_HOLES, TOTAL_HOLES,
    MIN_PASS, GAPS, HOLE_LISTS, FAMILIES,
    makeHole, normalize, cloneHole, sanitize, tierOf, generate, courseHoles, coursePar,
  };
})();
