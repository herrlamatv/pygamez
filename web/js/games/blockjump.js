/*
 * blockjump.js - Block Jump (Port von games/blockjump.py)
 * ========================================================
 * Ein 3D-Jump'n'Run im Minecraft-Stil.
 *
 * - Voll in 3D (Software-Renderer wie im Original): eine Welt aus Würfel-
 *   Blöcken, auf die man springt. Distanz-Nebel, Himmels-Verlauf und
 *   optionaler Motion-Blur sorgen für den "hochwertigen" Look.
 * - Minecraft-Skin: alle Blöcke tragen 16x16-Pixeltexturen (Gras, Erde, Stein,
 *   Eichenbretter, Diamant, Schleim, Holz), die perspektivisch korrekt in
 *   Texel-Vierecke zerlegt werden. Der Detailgrad hängt vom Abstand ab
 *   (Taste T: hoch / niedrig / aus).
 * - Steve-Spielfigur mit Laufanimation (3rd Person), Hand im Ego-Modus,
 *   Beacon-Strahl am Ziel, rotierende Gold-Barren als Coins, quadratische
 *   Sonne, driftende Pixel-Wolken und ein HUD mit Herzen und Schattenschrift.
 * - Blocktypen: Gras/Erde/Stein/Holz (fest), Leitern (kletterbar), Zäune
 *   (überspringbar), Schleimblöcke (katapultieren), Ziel und Coins.
 * - Kamera standardmäßig 1st-Person (Mouselook mit Pointer-Lock);
 *   V schaltet auf eine 3rd-Person-Verfolgerkamera um.
 * - Steuerung: WASD/Pfeile laufen, Leertaste springen, Maus umsehen; an Leitern
 *   klettert W/S. V Ansicht, T Texturen, B Motion-Blur, C Maus fangen/frei,
 *   I Maus-Richtung, +/- Empfindlichkeit.
 * - Seed-generierte Parkour-Level; Ziel = Punkte + Zeitbonus, Absturz kostet
 *   ein Leben (Start mit 3). Bei 0 Leben ist Schluss.
 *
 * Canvas2D: jede Fläche/jedes Texel-Rechteck ist ein gefülltes Polygon (wie
 * pygame.draw.polygon im Original, Maler-Algorithmus nach Tiefe). Die Welt
 * wird für den Motion-Blur in eine Offscreen-Canvas gezeichnet.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ---------------------------------------------------------------------------
  //  Renderer-Konstanten
  // ---------------------------------------------------------------------------
  const NEAR = 0.12;
  const FOV_MUL = 1.0;
  const FOG_START = 16.0;
  const FOG_END = 46.0;
  const DEG_PER_PX = 0.12; // Grad Drehung je Maus-Pixel bei sens = 1.0
  const PITCH_CLAMP = PG.radians(84);
  const SEAM = 0.8; // Konturbreite gegen Antialiasing-Säume zwischen Flächen

  // Minecraft-Himmel: kräftiges Blau oben, heller Dunst am Horizont.
  const COL_SKY_TOP = [110, 160, 250];
  const COL_SKY_HOR = [178, 209, 252];
  const COL_FOG = [198, 222, 252];
  const COL_SUN = [252, 250, 208];

  // ---------------------------------------------------------------------------
  //  Physik
  // ---------------------------------------------------------------------------
  const GRAVITY = 20.0;
  const JUMP_VEL = 7.7;
  const MOVE_SPEED = 4.7;
  const CLIMB_SPEED = 3.3;
  const SPRING_VEL = 12.6;
  const EYE_H = 1.62;
  const PLAYER_H = 1.7;
  const HALF_W = 0.3;
  const VY_MIN = -16.0, VY_MAX = 14.5;
  const EPS = 1e-4;

  // ---------------------------------------------------------------------------
  //  Blocktypen
  // ---------------------------------------------------------------------------
  const EMPTY = 0, GRASS = 1, DIRT = 2, STONE = 3, PLANK = 4, LADDER = 5, FENCE = 6, SPRING = 7, GOAL = 8;
  const SOLID = new Set([GRASS, DIRT, STONE, PLANK, GOAL]); // volle 1x1x1-Blöcke
  const FENCE_H = 0.8; // Zaun: niedriges Hindernis, per Sprung überwindbar
  const SPRING_H = 0.78; // Schleimblock: nur Landefläche (blockiert nie seitlich)

  const READY = "ready", PLAY = "play", CLEAR = "clear", GAMEOVER = "gameover";
  const SEED_BASE = { easy: 1000, normal: 2000, hard: 3000 };
  const TEX_MODES = ["off", "low", "high"]; // Index = this.texMode

  const css = (c) => "rgb(" + (c[0] | 0) + "," + (c[1] | 0) + "," + (c[2] | 0) + ")";
  const unpack = (p) => [(p >> 16) & 255, (p >> 8) & 255, p & 255];

  // ===========================================================================
  //  Pixel-Texturen im Minecraft-Stil
  //  Jede Textur ist ein quadratisches Raster aus [r,g,b] (oder null für
  //  "durchsichtig"). Prozedural mit festen Seeds erzeugt - identisch über alle
  //  Frames/Starts, ohne Bilddateien.
  // ===========================================================================
  function mul(c, k) {
    return [
      Math.max(0, Math.min(255, Math.trunc(c[0] * k))),
      Math.max(0, Math.min(255, Math.trunc(c[1] * k))),
      Math.max(0, Math.min(255, Math.trunc(c[2] * k))),
    ];
  }

  /** Texturen im MC-Stil: große Farbfelder + wenige Einzel-Sprenkel. */
  function blocks(size, seed, cols, weights, spec, specP = 0.05, varr = 0.05, cell = 4) {
    const rng = new PG.Random(seed);
    const rows = Array.from({ length: size }, () => new Array(size).fill(null));
    for (let by = 0; by < size; by += cell) {
      for (let bx = 0; bx < size; bx += cell) {
        const base = rng.weighted(cols, weights);
        for (let sy = 0; sy < cell; sy += 2) {
          // feine Abstufung je 2x2
          for (let sx = 0; sx < cell; sx += 2) {
            const c = mul(base, rng.uniform(1 - varr, 1 + varr));
            for (let y = sy; y < sy + 2; y++) for (let x = sx; x < sx + 2; x++) rows[by + y][bx + x] = c;
          }
        }
      }
    }
    if (spec && spec.length) {
      for (let y = 0; y < size; y++) for (let x = 0; x < size; x++) if (rng.random() < specP) rows[y][x] = rng.choice(spec);
    }
    return rows;
  }

  const tGrassTop = () => blocks(16, 11, [[116, 172, 78], [105, 160, 70], [127, 182, 87]], [5, 3, 2], [[97, 150, 64], [135, 190, 95]], 0.06);
  const tDirt = () => blocks(16, 12, [[134, 96, 67], [123, 88, 60], [145, 105, 74]], [5, 3, 2], [[113, 80, 55], [152, 112, 79]], 0.06);
  const tStone = () => blocks(16, 14, [[127, 127, 127], [118, 118, 118], [136, 136, 136]], [5, 3, 2], [[109, 109, 109], [145, 145, 145]], 0.06);

  /** Erde mit grünem Überhang oben - das MC-Erkennungsmerkmal schlechthin. */
  function tGrassSide() {
    const dirt = tDirt();
    const green = [116, 172, 78], gdark = [97, 150, 62];
    const edge = [4, 4, 3, 3, 4, 4, 3, 3, 2, 2, 4, 4, 3, 3, 4, 4];
    const rows = [];
    for (let y = 0; y < 16; y++) {
      const row = [];
      for (let x = 0; x < 16; x++) {
        const e = edge[x];
        row.push(y < e - 1 ? green : y < e ? gdark : dirt[y][x]);
      }
      rows.push(row);
    }
    return rows;
  }

  /** Eichenbretter: warmes Holz, dunkle Fugen, feine Maserung. */
  function tPlanks() {
    const base = [162, 130, 78], seam = [124, 97, 55], grain = [146, 116, 68], light = [172, 141, 88];
    const rng = new PG.Random(15);
    const rows = [];
    for (let y = 0; y < 16; y++) {
      const band = [0, 6, 12].includes(y) ? light : [3, 4, 9, 15].includes(y) ? grain : base;
      const row = [];
      for (let x = 0; x < 16; x++) {
        let c;
        if (y === 5 || y === 11) c = seam; // waagrechte Brettfuge
        else if ((y < 5 && x === 9) || (y > 5 && y < 11 && x === 3) || (y > 11 && x === 12)) c = seam; // senkrechte Stoßfuge
        else if (rng.random() < 0.06) c = grain;
        else c = band;
        row.push(c);
      }
      rows.push(row);
    }
    return rows;
  }

  /** Stamm-/Balkenholz für Leiter und Zaun (senkrechte Maserung). */
  function tWood() {
    const base = [150, 116, 66], dark = [120, 90, 50], light = [168, 132, 80];
    const cols = [];
    for (let x = 0; x < 16; x++) cols.push([0, 7, 8, 15].includes(x) ? dark : [3, 11].includes(x) ? light : base);
    const rng = new PG.Random(16);
    const rows = [];
    for (let y = 0; y < 16; y++) {
      const row = [];
      for (let x = 0; x < 16; x++) row.push(rng.random() < 0.04 ? dark : cols[x]);
      rows.push(row);
    }
    return rows;
  }

  /** Diamantblock als Ziel: türkise Kristalle auf hellem Grund. */
  function tDiamond() {
    const base = [110, 222, 216], cry = [78, 200, 198], hi = [196, 248, 244], dk = [58, 168, 168];
    const spots = [[3, 3], [11, 4], [6, 10], [13, 12]];
    const rows = [];
    for (let y = 0; y < 16; y++) {
      const row = [];
      for (let x = 0; x < 16; x++) {
        let c = base;
        for (const [sx, sy] of spots) {
          const d = Math.max(Math.abs(x - sx), Math.abs(y - sy));
          if (d === 0) c = hi;
          else if (d === 1) c = cry;
          else if (d === 2 && c === base) c = dk;
        }
        row.push(c);
      }
      rows.push(row);
    }
    return rows;
  }

  /** Schleimblock: grün, mit dunklem Rahmen und hellem Kern. */
  function tSlime() {
    const base = [114, 196, 94], edge = [86, 158, 70], core = [146, 224, 126], hi = [176, 240, 158];
    const rows = [];
    for (let y = 0; y < 16; y++) {
      const row = [];
      for (let x = 0; x < 16; x++) {
        const b = Math.min(x, y, 15 - x, 15 - y);
        row.push(b < 1 ? edge : b < 3 ? base : b < 6 ? core : hi);
      }
      rows.push(row);
    }
    return rows;
  }

  /** Lichtstrahl des Ziels - fast weiß mit türkisem Schimmer. */
  function tBeacon() {
    const a = [226, 255, 246], b = [186, 246, 232];
    return Array.from({ length: 8 }, (_, y) => Array.from({ length: 8 }, (_2, x) => ((x + y) % 3 ? a : b)));
  }

  // ----- Zeichen-Art (Steve, Gold-Barren) -----------------------------------
  function art(rows, pal) {
    return rows.map((r) => Array.from(r).map((ch) => pal[ch] || null));
  }

  const SKIN = {
    H: [74, 48, 30], // Haare
    h: [60, 38, 23], // Haare dunkel
    S: [233, 189, 148], // Haut
    s: [204, 160, 122], // Haut Schatten
    W: [245, 245, 245], // Augenweiß
    I: [62, 92, 190], // Iris
    M: [150, 96, 78], // Mund
    C: [0, 172, 172], // Shirt
    c: [0, 142, 145], // Shirt dunkel
    B: [58, 63, 138], // Hose
    b: [46, 50, 116], // Hose dunkel
    G: [88, 76, 64], // Schuh
    K: [52, 44, 36], // Gürtel/Kragen
  };

  const FACE = ["HHHHHHHH", "HHHHHHHH", "HSSSSSSH", "SWIssIWS", "SSSsSSSS", "SSMMMMSS", "hSSSSSSh", "SSssssSS"];
  const HEAD_SIDE = ["HHHHHHHH", "HHHHHHHH", "HHHSSSSS", "HHSSSSSs", "HSSSSSSS", "SSSSSSSs", "SSSSSSSS", "SSsSSsSS"];
  const HEAD_BACK = ["HHHHHHHH", "HHHHHHHH", "HHHHHHHH", "HHHHHHHH", "HHHhhHHH", "HHHHHHHH", "hHHHHHHh", "SSssssSS"];
  const HEAD_TOP = ["HHHHHHHH", "HhHHHHhH", "HHHHhHHH", "HHhHHHHH", "HHHHHhHH", "HhHHHHHH", "HHHHhHHH", "HHHHHHHH"];
  const BODY_FRONT = ["SSKKKKSS", "CCCCCCCC", "CCCcCCCC", "CCCCCcCC", "CcCCCCCC", "CCCCCCcC", "cCCCCCCc", "KKKKKKKK"];
  const BODY_SIDE = ["KKKKKKKK", "CCCCCCCC", "CCcCCCCC", "CCCCCCcC", "CcCCCCCC", "CCCCcCCC", "cCCCCCCc", "KKKKKKKK"];
  const ARM = ["CCCCCCCC", "CCcCCCCC", "CCCCCcCC", "cCCCCCCc", "CCCCcCCC", "SSSSSSSS", "SsSSSSsS", "SSSssSSS"];
  const LEG = ["BBBBBBBB", "BBbBBBBB", "BBBBBbBB", "bBBBBBBb", "BBBBbBBB", "BBbBBBBB", "GGGGGGGG", "GGGGGGGG"];
  const INGOT = ["        ", "  ####  ", " ###### ", "########", "########", " ###### ", "  ####  ", "        "];

  /** Gold-Barren als schwebendes Item (mit durchsichtigem Rand). */
  function tIngot() {
    const gold = [247, 214, 88], hi = [255, 242, 170], dk = [198, 152, 44];
    return INGOT.map((line, y) => Array.from(line).map((ch) => (ch === " " ? null : y <= 2 ? hi : y >= 5 ? dk : gold)));
  }

  const TEX = {
    grass_top: tGrassTop(),
    grass_side: tGrassSide(),
    dirt: tDirt(),
    stone: tStone(),
    planks: tPlanks(),
    wood: tWood(),
    diamond: tDiamond(),
    slime: tSlime(),
    beacon: tBeacon(),
    ingot: tIngot(),
    face: art(FACE, SKIN),
    head_side: art(HEAD_SIDE, SKIN),
    head_back: art(HEAD_BACK, SKIN),
    head_top: art(HEAD_TOP, SKIN),
    body_front: art(BODY_FRONT, SKIN),
    body_side: art(BODY_SIDE, SKIN),
    arm: art(ARM, SKIN),
    leg: art(LEG, SKIN),
  };

  // Blocktyp -> [oben, Seite, unten]
  const BLOCK_TEX = {
    [GRASS]: ["grass_top", "grass_side", "dirt"],
    [DIRT]: ["dirt", "dirt", "dirt"],
    [STONE]: ["stone", "stone", "stone"],
    [PLANK]: ["planks", "planks", "planks"],
    [GOAL]: ["diamond", "diamond", "diamond"],
  };

  const MIP = new Map(); // "name|n" -> n x n Raster (gepackte Farben, -1 = durchsichtig)
  const RECTS = new Map(); // "name|n" -> zusammengefasste Farbrechtecke
  const GRID = new Map(); // "name|n|shade|fog" -> eingefärbte Rechtecke (CSS)
  const AVG = new Map(); // name -> Durchschnittsfarbe (Modus "Texturen aus")
  const OPAQUE = new Map(); // name -> Textur ohne durchsichtige Texel?
  const QUANT = 12; // Farbraster: gleiche Nachbartöne lassen sich zusammenfassen

  /** Verkleinert eine Textur auf n x n (Mittelwert + Kontrast + Farbraster). */
  function mip(name, n) {
    const key = name + "|" + n;
    let m = MIP.get(key);
    if (m) return m;
    const src = TEX[name];
    const size = src.length;
    n = Math.min(n, size);
    const step = Math.floor(size / n);
    let rows = [];
    for (let j = 0; j < n; j++) {
      const row = [];
      for (let i = 0; i < n; i++) {
        let r = 0, g = 0, b = 0, cnt = 0;
        for (let y = j * step; y < (j + 1) * step; y++) {
          for (let x = i * step; x < (i + 1) * step; x++) {
            const c = src[y][x];
            if (!c) continue;
            r += c[0];
            g += c[1];
            b += c[2];
            cnt++;
          }
        }
        if (cnt * 2 < step * step) row.push(null); // überwiegend durchsichtig
        else row.push([Math.floor(r / cnt), Math.floor(g / cnt), Math.floor(b / cnt)]);
      }
      rows.push(row);
    }
    if (n < size) {
      // Kontrast nachschärfen
      const vals = [];
      for (const row of rows) for (const c of row) if (c) vals.push(c);
      if (vals.length) {
        let mr = 0, mg = 0, mb = 0;
        for (const c of vals) {
          mr += c[0];
          mg += c[1];
          mb += c[2];
        }
        mr /= vals.length;
        mg /= vals.length;
        mb /= vals.length;
        const f = 1.08;
        rows = rows.map((row) =>
          row.map((c) =>
            c === null
              ? null
              : [
                  Math.max(0, Math.min(255, Math.trunc(mr + (c[0] - mr) * f))),
                  Math.max(0, Math.min(255, Math.trunc(mg + (c[1] - mg) * f))),
                  Math.max(0, Math.min(255, Math.trunc(mb + (c[2] - mb) * f))),
                ]
          )
        );
      }
    }
    const q = QUANT, h = QUANT / 2; // aufs Farbraster runden
    m = rows.map((row) =>
      row.map((c) => {
        if (c === null) return -1;
        const r = Math.min(255, Math.floor(c[0] / q) * q + h);
        const g = Math.min(255, Math.floor(c[1] / q) * q + h);
        const b = Math.min(255, Math.floor(c[2] / q) * q + h);
        return (r << 16) | (g << 8) | b;
      })
    );
    MIP.set(key, m);
    return m;
  }

  /**
   * Fasst gleichfarbige Texel zu Rechtecken zusammen (spart Polygone). Die
   * Zerlegung hängt nur von der Textur ab, nicht von Licht/Nebel - einmal
   * berechnet, für alle Flächen wiederverwendet.
   */
  function rects(name, n) {
    const key = name + "|" + n;
    let r = RECTS.get(key);
    if (r) return r;
    const grid = mip(name, n);
    n = grid.length;
    const used = Array.from({ length: n }, () => new Array(n).fill(false));
    const out = [];
    for (let j = 0; j < n; j++) {
      for (let i = 0; i < n; i++) {
        if (used[j][i]) continue;
        const c = grid[j][i];
        used[j][i] = true;
        if (c < 0) continue;
        let i1 = i;
        while (i1 + 1 < n && !used[j][i1 + 1] && grid[j][i1 + 1] === c) {
          i1++;
          used[j][i1] = true;
        }
        let j1 = j;
        for (;;) {
          if (j1 + 1 >= n) break;
          let ok = true;
          for (let x = i; x <= i1; x++) {
            if (used[j1 + 1][x] || grid[j1 + 1][x] !== c) {
              ok = false;
              break;
            }
          }
          if (!ok) break;
          j1++;
          for (let x = i; x <= i1; x++) used[j1][x] = true;
        }
        out.push([i, j, i1 + 1, j1 + 1, c]);
      }
    }
    RECTS.set(key, out);
    return out;
  }

  function avgCol(name) {
    let a = AVG.get(name);
    if (!a) {
      let r = 0, g = 0, b = 0, cnt = 0;
      for (const row of TEX[name]) {
        for (const c of row) {
          if (!c) continue;
          r += c[0];
          g += c[1];
          b += c[2];
          cnt++;
        }
      }
      a = cnt ? [Math.floor(r / cnt), Math.floor(g / cnt), Math.floor(b / cnt)] : [128, 128, 128];
      AVG.set(name, a);
    }
    return a;
  }

  /** true, wenn die Textur keine durchsichtigen Texel hat. */
  function opaque(name) {
    let o = OPAQUE.get(name);
    if (o === undefined) {
      o = TEX[name].every((row) => row.every((c) => c !== null));
      OPAQUE.set(name, o);
    }
    return o;
  }

  function shadeCol(col, k) {
    return [Math.min(255, Math.trunc(col[0] * k)), Math.min(255, Math.trunc(col[1] * k)), Math.min(255, Math.trunc(col[2] * k))];
  }

  function fogCol(col, ft) {
    if (ft <= 0) return col;
    return [
      Math.trunc(col[0] + (COL_FOG[0] - col[0]) * ft),
      Math.trunc(col[1] + (COL_FOG[1] - col[1]) * ft),
      Math.trunc(col[2] + (COL_FOG[2] - col[2]) * ft),
    ];
  }

  /**
   * Farbrechtecke [i0, j0, i1, j1, css] inklusive Licht und Nebel. Shade/Nebel
   * werden quantisiert, damit der Cache klein bleibt.
   */
  function texGrid(name, n, shade, ft) {
    const si = Math.trunc(shade * 24 + 0.5);
    const fi = Math.trunc(ft * 12 + 0.5);
    const key = name + "|" + n + "|" + si + "|" + fi;
    let g = GRID.get(key);
    if (!g) {
      const k = si / 24.0, f = fi / 12.0;
      g = rects(name, n).map(([i0, j0, i1, j1, c]) => [i0, j0, i1, j1, css(fogCol(shadeCol(unpack(c), k), f))]);
      GRID.set(key, g);
    }
    return g;
  }

  // Einfarbige Ersatzfarben für die feine Holz-/Schleimgeometrie
  const COL_LADDER = avgCol("wood");
  const COL_FENCE = avgCol("wood");
  const COL_SLIME = avgCol("slime");

  // Pixel-Wolken (0/1-Maske, in Kacheln zu je 8 Blöcken)
  const CLOUD_MAP = ["01111000", "01111100", "00111000", "00000000", "00011110", "00111110", "00011100", "00000000"];

  const HEART_ART = ["  ##  ##  ", " #rr##rr# ", "#rrrrrrrr#", "#rrrrrrrr#", "#rrrrrrrr#", " #rrrrrr# ", "  #rrrr#  ", "   #rr#   ", "    ##    "];
  const HEART_PAL = { "#": "rgb(54,16,16)", r: "rgb(226,58,52)" };
  const HEART_EMPTY_PAL = { "#": "rgb(34,34,38)", r: "rgb(72,72,78)" };

  function dirFrom(yaw, pitch) {
    const cp = Math.cos(pitch);
    return [Math.sin(yaw) * cp, Math.sin(pitch), Math.cos(yaw) * cp];
  }

  function lerp3(a, b, f) {
    return [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f, a[2] + (b[2] - a[2]) * f];
  }

  function clipNear(pts) {
    const out = [];
    const n = pts.length;
    for (let i = 0; i < n; i++) {
      const a = pts[i], b = pts[(i + 1) % n];
      const aIn = a[2] >= NEAR, bIn = b[2] >= NEAR;
      if (aIn) out.push(a);
      if (aIn !== bIn) {
        const tt = (NEAR - a[2]) / (b[2] - a[2]);
        out.push([a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR]);
      }
    }
    return out;
  }

  const ARROWS = { Up: "up", Down: "down", Left: "left", Right: "right" };
  const bkey = (x, y, z) => x + "," + y + "," + z;

  class BlockJumpGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      if (!(this.mode in SEED_BASE)) this.mode = "normal";
      const cfg = this.opts;
      const num = (v, d) => (isFinite(Number(v)) ? Number(v) : d);
      this.blur = Math.max(0.0, Math.min(0.8, num(cfg.blur, 0.35)));
      this.view = cfg.view === "third" ? "third" : "first";
      this.sens = Math.max(0.4, Math.min(2.5, num(cfg.sens, 1.0)));
      // Maus-Richtung: Standard normal (Maus rechts -> Blick rechts); Taste I invertiert.
      this.invert = !!cfg.mouse_invert;
      // Texturdetail: 2 = hoch (bis 8x8 Texel je Fläche), 1 = niedrig (4x4), 0 = aus
      const ti = TEX_MODES.indexOf(cfg.textures);
      this.texMode = ti >= 0 ? ti : 2;
      this.makeFonts();
      this._hasPrev = false;
      this.anim = 0.0;
      this.walk = 0.0;
      this.held = new Set();
      this._wantCapture = true;
      this._lastBonus = 0;
      this.newRun();
    }

    get captureMouse() {
      // Auch während der kurzen "Level geschafft"-Einblendung (CLEAR) gefangen
      // halten: Python fängt die Maus danach automatisch wieder ein, im Browser
      // bräuchte ein erneuter Pointer-Lock aber einen Klick - nach jedem Level.
      // (Mausbewegungen wirken ohnehin nur in PLAY, siehe handleEvent.)
      return this._wantCapture && (this.state === PLAY || this.state === CLEAR) && !this.paused;
    }

    makeFonts() {
      this._hud = ui.font(20, true);
      this._small = ui.font(15);
      this._huge = ui.font(Math.max(30, Math.floor(this.height / 11)), true);
      this._mid = ui.font(22, true);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    newRun() {
      this.lives = 3;
      this.score = 0;
      this.gameOver = false;
      this.level = 1;
      this.yaw = 0.0;
      this.pitch = 0.0;
      this.buildLevel(this.level);
      this.state = READY;
    }

    // ===================================================== Level-Generierung
    buildLevel(level) {
      const rng = new PG.Random((SEED_BASE[this.mode] || 2000) + level * 7919);
      this.world = new Map();
      this.coins = [];
      const hard = this.mode === "hard";
      const easy = this.mode === "easy";
      const W = this.world;

      const nPlat = 7 + level * 2 + (hard ? 2 : 0);
      this._platCount = nPlat;
      let cx = 0, cy = 0, cz = 0;
      this._minY = 0;
      this.pad(cx, cz, cy, 1, 1, GRASS);
      this.spawn = [cx + 0.5, cy + 1.0, cz + 0.5];

      const feats = !easy ? ["jump", "jump", "ladder", "spring", "fence"] : ["jump", "jump", "jump", "ladder", "spring"];
      for (let i = 0; i < nPlat; i++) {
        const last = i === nPlat - 1;
        const feat = last ? "jump" : rng.choice(feats);
        const hw = 1;
        let hd = !hard ? 1 : rng.choice([0, 1, 1]);
        const topType = rng.choice([GRASS, GRASS, STONE, PLANK]);

        if (feat === "ladder") {
          const climb = rng.choice(easy ? [3, 4] : [3, 4, 5]);
          // Leiter-Säule mit Stützwand dahinter
          const ladZ = cz + 1;
          for (let yy = cy + 1; yy < cy + climb + 1; yy++) {
            W.set(bkey(cx, yy, ladZ), LADDER);
            W.set(bkey(cx, yy, ladZ + 1), STONE);
          }
          // Coin mittig in der Kletterspalte
          this.coins.push([cx + 0.5, cy + climb / 2.0 + 1.4, ladZ + 0.5]);
          cy = cy + climb;
          // Pad beginnt HINTER der Stützwand, damit der Leiterschacht offen bleibt
          cz = cz + 2 + hd;
          this.pad(cx, cz, cy, hw, hd, topType);
        } else if (feat === "spring") {
          // erhöhter Schleimblock auf dem aktuellen Pad ...
          W.set(bkey(cx, cy + 1, cz), SPRING);
          // ... nächstes Pad deutlich höher (per Katapult erreichbar)
          const lift = rng.choice([3, 4]);
          const dz = rng.choice([2, 3]);
          cy = cy + lift;
          cz = cz + dz;
          this.pad(cx, cz, cy, hw, hd, topType);
          this.coins.push([cx + 0.5, cy - lift / 2.0 + 2.0, cz - dz / 2.0]);
        } else {
          // jump / fence
          const dz = rng.choice(easy ? [2, 3] : [3, 4]);
          const dx = rng.choice(easy ? [-1, 0, 1] : [-2, -1, 0, 1, 2]);
          let dy = rng.choice(easy ? [-1, 0] : [-2, -1, 0, 1]);
          if (hard && dz >= 4) {
            // Max-Lücke entschärfen: nie bergauf und immer ein tiefes Lande-Pad
            dy = Math.min(dy, 0);
            hd = 1;
          }
          const mx = cx, my = cy, mz = cz;
          cx = cx + dx;
          cy = Math.max(this._minY - 3, cy + dy);
          cz = cz + dz;
          this.pad(cx, cz, cy, hw, hd, topType);
          // Coin über der Lücke
          this.coins.push([(mx + cx) / 2.0 + 0.5, (my + cy) / 2.0 + 1.7, (mz + cz) / 2.0 + 0.5]);
          if (feat === "fence" && !last) W.set(bkey(cx, cy + 1, cz), FENCE);
        }
        this._minY = Math.min(this._minY, cy);
      }

      // Ziel-Beacon auf dem letzten Pad
      W.set(bkey(cx, cy + 1, cz), GOAL);
      this.goal = [cx + 0.5, cy + 1.0, cz + 0.5];
      this.deathY = this._minY - 7.0;
      // Blockliste für den Renderer (Weltdaten ändern sich im Level nicht)
      this.blockList = [];
      let maxY = -1e9;
      for (const [k, typ] of W) {
        const p = k.split(",").map(Number);
        this.blockList.push([p[0], p[1], p[2], typ]);
        maxY = Math.max(maxY, p[1]);
      }
      // Wolkendecke weit über dem höchsten Block (wie in Minecraft)
      this._cloudY = maxY + 46.0;

      // Spieler setzen
      [this.px, this.py, this.pz] = this.spawn;
      this.vx = this.vy = this.vz = 0.0;
      this.onGround = true;
      this.onLadder = false;
      this.checkpoint = this.spawn.slice();
      this.levelTime = 0.0;
      this.coinsLevel = 0;
      this._clearT = 0.0;
      this._camPos = [this.px, this.py + EYE_H, this.pz];
      this._camLook = [this.px, this.py + EYE_H, this.pz + 1.0];
    }

    pad(cx, cz, y, hw, hd, typ) {
      for (let x = cx - hw; x <= cx + hw; x++) for (let z = cz - hd; z <= cz + hd; z++) this.world.set(bkey(x, y, z), typ);
    }

    // ===================================================== Blockabfragen
    cell(x, y, z) {
      const v = this.world.get(bkey(Math.trunc(x), Math.trunc(y), Math.trunc(z)));
      return v === undefined ? EMPTY : v;
    }

    isSolid(x, y, z) {
      return SOLID.has(this.cell(x, y, z));
    }

    /** Kollisionshöhe der Zelle für diese Bewegung (null = frei). */
    colH(x, y, z, axis, delta) {
      const typ = this.cell(x, y, z);
      if (SOLID.has(typ)) return 1.0;
      if (typ === FENCE) return FENCE_H;
      if (typ === SPRING && axis === 1 && delta < 0) return SPRING_H; // nur Landung von oben
      return null;
    }

    // ===================================================== Eingabe
    /** Bewegungs-Aktionen, die 'key' laut Belegung auslöst. */
    moveActs(key) {
      const k = key && key.length === 1 ? key.toLowerCase() : key; // "W" (Shift) == "w"
      const acts = [];
      for (const a of ["up", "down", "left", "right"]) if (this.isAction(k, a)) acts.push(a);
      if (ARROWS[key]) acts.push(ARROWS[key]); // Pfeiltasten-Fallback
      return acts;
    }

    handleEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (this.state === READY || this.state === GAMEOVER) {
          if (k === "Return" || k === "space") this.startOrRestart();
          return;
        }
        for (const a of this.moveActs(k)) this.held.add(a);
        if (k === "space" || this.isAction(k, "action")) this.jump();
        else if (k === "v" || k === "V") this.toggleView();
        else if (k === "b" || k === "B") this.cycleBlur();
        else if (k === "t" || k === "T") this.cycleTex();
        else if (k === "c" || k === "C") this._wantCapture = !this._wantCapture;
        else if (k === "i" || k === "I") this.toggleInvert();
        else if (k === "plus" || k === "KP_Add" || k === "equal") this.changeSens(0.1);
        else if (k === "minus" || k === "KP_Subtract") this.changeSens(-0.1);
      } else if (ev.kind === "keyup") {
        for (const a of this.moveActs(ev.key)) this.held.delete(a);
      } else if (ev.kind === "mousedown") {
        if (this.state === READY || this.state === GAMEOVER) this.startOrRestart();
        else if (this.state === PLAY && !this.captureMouse) this._wantCapture = true;
      } else if (ev.kind === "mouserel" && this.state === PLAY) {
        this.applyLook(ev.rel);
      }
    }

    startOrRestart() {
      if (this.state === GAMEOVER) {
        this.gameOver = false;
        this.newRun();
      }
      this.held.clear();
      this.state = PLAY;
      this.levelTime = 0.0;
      this._wantCapture = true;
      this.playSound("click");
    }

    applyLook(rel) {
      const k = PG.radians(DEG_PER_PX) * this.sens;
      // s = -1 -> normal (Maus rechts = Blick rechts, Maus hoch = Blick hoch);
      // s = +1 -> invertiert (beide Achsen umgekehrt).
      const s = this.invert ? 1.0 : -1.0;
      this.yaw = PG.mod(this.yaw + rel[0] * k * s, PG.TAU);
      this.pitch = Math.max(-PITCH_CLAMP, Math.min(PITCH_CLAMP, this.pitch + rel[1] * k * s));
    }

    toggleView() {
      this.view = this.view === "first" ? "third" : "first";
      this.saveSetting("view", this.view);
    }

    cycleBlur() {
      this.blur = this.blur >= 0.79 ? 0.0 : Math.round((this.blur + 0.2) * 100) / 100;
      this.saveSetting("blur", this.blur);
    }

    cycleTex() {
      this.texMode = PG.mod(this.texMode - 1, 3); // hoch -> niedrig -> aus
      this.saveSetting("textures", TEX_MODES[this.texMode]);
      this.playSound("click");
    }

    toggleInvert() {
      this.invert = !this.invert;
      this.saveSetting("mouse_invert", this.invert);
      this.playSound("click");
    }

    changeSens(d) {
      this.sens = Math.round(Math.max(0.4, Math.min(2.5, this.sens + d)) * 10) / 10;
      this.saveSetting("sens", this.sens);
    }

    jump() {
      if (this.state !== PLAY) return;
      if (this.onGround) {
        this.vy = JUMP_VEL;
        this.onGround = false;
        this.playSound("click");
      } else if (this.onLadder) {
        this.vy = JUMP_VEL * 0.85;
        this.onLadder = false;
        const f = dirFrom(this.yaw, 0.0);
        this.vx = -f[0] * MOVE_SPEED;
        this.vz = -f[2] * MOVE_SPEED;
        this.playSound("click");
      }
    }

    // ===================================================== Update / Physik
    update(dt) {
      this.anim += dt;
      if (this.state === CLEAR) {
        this._clearT -= dt;
        if (this._clearT <= 0) {
          this.level += 1;
          this.buildLevel(this.level);
          this.state = PLAY;
        }
        return;
      }
      if (this.state !== PLAY) return;
      this.levelTime += dt;
      this.physics(Math.min(dt, 0.05));
    }

    physics(dt) {
      const h = this.held;
      const fwd = (h.has("up") ? 1 : 0) - (h.has("down") ? 1 : 0);
      const strafe = (h.has("right") ? 1 : 0) - (h.has("left") ? 1 : 0);

      this.onLadder = this.inLadder();
      const f = dirFrom(this.yaw, 0.0);
      const fx = f[0], fz = f[2];
      const rx = -fz, rz = fx; // Kamera-Rechtsvektor (Bildschirm-rechts)

      if (this.onLadder) {
        this.vy = CLIMB_SPEED * fwd;
        const mvx = rx * strafe, mvz = rz * strafe;
        const ln = Math.hypot(mvx, mvz);
        if (ln > 1e-6) {
          this.vx = (mvx / ln) * MOVE_SPEED * 0.7;
          this.vz = (mvz / ln) * MOVE_SPEED * 0.7;
        } else {
          this.vx = this.vz = 0.0;
        }
      } else {
        const mvx = fx * fwd + rx * strafe;
        const mvz = fz * fwd + rz * strafe;
        const ln = Math.hypot(mvx, mvz);
        if (ln > 1e-6) {
          this.vx = (mvx / ln) * MOVE_SPEED;
          this.vz = (mvz / ln) * MOVE_SPEED;
        } else {
          this.vx = this.vz = 0.0;
        }
        this.vy -= GRAVITY * dt;
      }
      this.vy = Math.max(VY_MIN, Math.min(VY_MAX, this.vy));

      // Schrittzähler für die Lauf-/Handanimation
      this.walk += dt * 9.0 * Math.min(1.0, Math.hypot(this.vx, this.vz) / MOVE_SPEED);

      this.onGround = false;
      this._landCells = [];
      this.moveAxis(0, this.vx * dt);
      this.moveAxis(2, this.vz * dt);
      this.moveAxis(1, this.vy * dt);

      if (this.onGround) {
        let bounced = false;
        for (const c of this._landCells) {
          if (this.world.get(bkey(c[0], c[1], c[2])) === SPRING) {
            this.vy = SPRING_VEL;
            this.onGround = false;
            bounced = true;
            this.playSound("move");
            break;
          }
        }
        if (!bounced) this.checkpoint = [this.px, this.py, this.pz];
      }

      this.collectCoins();
      if (this.atGoal()) {
        this.levelClear();
        return;
      }
      if (this.py < this.deathY) this.die();
    }

    moveAxis(axis, delta) {
      if (axis === 0) this.px += delta;
      else if (axis === 1) this.py += delta;
      else this.pz += delta;
      const mn0 = this.px - HALF_W, mn1 = this.py, mn2 = this.pz - HALF_W;
      const mx0 = this.px + HALF_W, mx1 = this.py + PLAYER_H, mx2 = this.pz + HALF_W;
      const x0 = Math.floor(mn0 + EPS), x1 = Math.floor(mx0 - EPS);
      const y0 = Math.floor(mn1 + EPS), y1 = Math.floor(mx1 - EPS);
      const z0 = Math.floor(mn2 + EPS), z1 = Math.floor(mx2 - EPS);
      const hits = [];
      for (let x = x0; x <= x1; x++) {
        for (let y = y0; y <= y1; y++) {
          for (let z = z0; z <= z1; z++) {
            const hh = this.colH(x, y, z, axis, delta);
            if (hh !== null && mn1 + EPS < y + hh) hits.push([x, y, z, hh]);
          }
        }
      }
      if (!hits.length) return;
      if (axis === 0) {
        if (delta > 0) this.px = Math.min(...hits.map((q) => q[0])) - HALF_W - EPS;
        else if (delta < 0) this.px = Math.max(...hits.map((q) => q[0])) + 1 + HALF_W + EPS;
        this.vx = 0.0;
      } else if (axis === 2) {
        if (delta > 0) this.pz = Math.min(...hits.map((q) => q[2])) - HALF_W - EPS;
        else if (delta < 0) this.pz = Math.max(...hits.map((q) => q[2])) + 1 + HALF_W + EPS;
        this.vz = 0.0;
      } else if (delta > 0) {
        // Kopf stößt an die Decke
        this.py = Math.min(...hits.map((q) => q[1])) - PLAYER_H - EPS;
        this.vy = 0.0;
      } else if (delta < 0) {
        // Landung auf dem Boden
        const top = Math.max(...hits.map((q) => q[1] + q[3]));
        this.py = top + EPS;
        this.vy = 0.0;
        this.onGround = true;
        this._landCells = hits.filter((q) => q[1] + q[3] === top);
      }
    }

    inLadder() {
      const x0 = Math.floor(this.px - HALF_W + EPS), x1 = Math.floor(this.px + HALF_W - EPS);
      const y0 = Math.floor(this.py + EPS), y1 = Math.floor(this.py + PLAYER_H - EPS);
      const z0 = Math.floor(this.pz - HALF_W + EPS), z1 = Math.floor(this.pz + HALF_W - EPS);
      for (let x = x0; x <= x1; x++) for (let y = y0; y <= y1; y++) for (let z = z0; z <= z1; z++) if (this.cell(x, y, z) === LADDER) return true;
      return false;
    }

    collectCoins() {
      const remaining = [];
      for (const c of this.coins) {
        if (Math.abs(this.px - c[0]) < 0.85 && Math.abs(this.pz - c[2]) < 0.85 && Math.abs(this.py + 0.9 - c[1]) < 1.2) {
          this.score += 50;
          this.coinsLevel += 1;
          this.playSound("click");
        } else {
          remaining.push(c);
        }
      }
      this.coins = remaining;
    }

    atGoal() {
      const [gx, gy, gz] = this.goal;
      return Math.abs(this.px - gx) < 1.3 && Math.abs(this.pz - gz) < 1.3 && this.py > gy - 1.6;
    }

    levelClear() {
      const par = this._platCount * 3.2;
      const bonus = Math.max(0, Math.trunc((par - this.levelTime) * 15));
      this.score += 1000 + bonus;
      this._lastBonus = bonus;
      this.state = CLEAR;
      this._clearT = 1.8;
      this.playSound("win");
    }

    die() {
      this.lives -= 1;
      this.playSound("gameover");
      if (this.lives <= 0) {
        this.state = GAMEOVER;
        this.gameOver = true;
      } else {
        [this.px, this.py, this.pz] = this.checkpoint;
        this.vx = this.vy = this.vz = 0.0;
        this.onGround = true;
      }
    }

    // ===================================================== Kamera
    updateCam() {
      const head = [this.px, this.py + EYE_H, this.pz];
      const f = dirFrom(this.yaw, this.pitch);
      if (this.view === "first") {
        this._camPos = head;
        this._camLook = [head[0] + f[0], head[1] + f[1], head[2] + f[2]];
      } else {
        const lookH = this.py + 1.1;
        const target = [this.px, lookH, this.pz];
        const dist = 4.4;
        const want = [this.px - f[0] * dist, lookH - f[1] * dist + 0.5, this.pz - f[2] * dist];
        this._camPos = this.camCollide(target, want);
        this._camLook = target;
      }
    }

    camCollide(a, b) {
      const dx = b[0] - a[0], dy = b[1] - a[1], dz = b[2] - a[2];
      const steps = 7;
      for (let i = 1; i <= steps; i++) {
        let tt = i / steps;
        const p = [a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt];
        if (this.isSolid(Math.floor(p[0]), Math.floor(p[1]), Math.floor(p[2]))) {
          tt = Math.max(0.0, (i - 1) / steps) * 0.92;
          return [a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt];
        }
      }
      return b;
    }

    // ===================================================== 3D-Renderer
    viewBasis() {
      const [cx, cy, cz] = this._camPos;
      const fx = this._camLook[0] - cx, fy = this._camLook[1] - cy, fz = this._camLook[2] - cz;
      const fl = Math.sqrt(fx * fx + fy * fy + fz * fz) || 1.0;
      const f = [fx / fl, fy / fl, fz / fl];
      const rl = Math.hypot(f[2], f[0]) || 1.0;
      const r = [-f[2] / rl, 0.0, f[0] / rl];
      const u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
      return [r, u, f];
    }

    toCam(p) {
      const b = this._basis, r = b[0], u = b[1], f = b[2];
      const dx = p[0] - this._camPos[0], dy = p[1] - this._camPos[1], dz = p[2] - this._camPos[2];
      return [dx * r[0] + dz * r[2], dx * u[0] + dy * u[1] + dz * u[2], dx * f[0] + dy * f[1] + dz * f[2]];
    }

    proj(c) {
      const k = this._f / c[2];
      return [this._scx + c[0] * k, this._scy - c[1] * k];
    }

    fogDepth(col, depth) {
      return fogCol(col, Math.max(0.0, Math.min(1.0, (depth - FOG_START) / (FOG_END - FOG_START))));
    }

    offscreen(pts) {
      const w = this.width + 40, h = this.height + 40;
      let l = true, r = true, tp = true, bt = true;
      for (const p of pts) {
        if (p[0] >= -40) l = false;
        if (p[0] <= w) r = false;
        if (p[1] >= -40) tp = false;
        if (p[1] <= h) bt = false;
      }
      return l || r || tp || bt;
    }

    /** Wieviele Texel je Kante bei diesem Abstand? (Detailstufe/LOD) */
    texN(depth, native, scale = 1.0, fine = false) {
      if (!this.texMode) return 1;
      const lim = this.texMode === 2 ? native : Math.min(4, native);
      const step = this.texMode === 2 ? (fine ? 7.0 : 16.0) : 26.0;
      const px = (this._f * scale) / Math.max(depth, 0.25);
      let n = 1;
      while (n < lim && px >= n * 2 * step) n *= 2;
      return n;
    }

    /** Einfarbige Fläche in die Zeichenliste legen. */
    addPoly(items, worldPts, color, shade = 1.0, outline = null, fog = true) {
      let cs = worldPts.map((p) => this.toCam(p));
      if (cs.every((c) => c[2] < NEAR)) return;
      cs = clipNear(cs);
      if (cs.length < 3) return;
      let depth = 0;
      for (const c of cs) depth += c[2];
      depth /= cs.length;
      if (fog && depth > FOG_END + 6) return;
      const pts = cs.map((c) => this.proj(c));
      if (this.offscreen(pts)) return;
      let col = shadeCol(color, shade);
      if (fog) col = this.fogDepth(col, depth);
      items.push([depth, [[pts, css(col)]], outline ? pts : null, outline]);
    }

    /**
     * Texturiertes Viereck. quad = [p00, p10, p11, p01] im Weltraum; p00 ist die
     * linke obere Texturecke, p00->p10 die u-Achse, p00->p01 die v-Achse. Die
     * Fläche wird je nach Abstand in n x n Texel-Vierecke zerlegt (LOD).
     */
    addFace(items, quad, tex, shade = 1.0, outline = null, scale = 1.0, fine = false) {
      if (this.texMode === 0) {
        this.addPoly(items, quad, avgCol(tex), shade, outline);
        return;
      }
      const ca = this.toCam(quad[0]), cb = this.toCam(quad[1]), cc = this.toCam(quad[2]), cd = this.toCam(quad[3]);
      const zmax = Math.max(ca[2], cb[2], cc[2], cd[2]);
      const zmin = Math.min(ca[2], cb[2], cc[2], cd[2]);
      if (zmax < NEAR) return;
      const depth = (ca[2] + cb[2] + cc[2] + cd[2]) * 0.25;
      if (depth > FOG_END + 6) return;
      const n = this.texN(depth, TEX[tex].length, scale, fine);
      if (n <= 1) {
        this.addPoly(items, quad, avgCol(tex), shade, outline);
        return;
      }
      const ft = Math.max(0.0, Math.min(1.0, (depth - FOG_START) / (FOG_END - FOG_START)));
      const rs = texGrid(tex, n, shade, ft);
      // Bilinear im Kameraraum: die Sicht-Transformation ist affin, das
      // Interpolieren dort ist also identisch zum Weltraum - aber billiger.
      const quads = [];
      if (zmin >= NEAR) {
        // Gitterpunkte in einem Rutsch interpolieren UND projizieren
        const f = this._f, scx = this._scx, scy = this._scy;
        const ax = ca[0], ay = ca[1], az = ca[2];
        const dlx = cd[0] - ax, dly = cd[1] - ay, dlz = cd[2] - az;
        const bx = cb[0], by = cb[1], bz = cb[2];
        const drx = cc[0] - bx, dry = cc[1] - by, drz = cc[2] - bz;
        const pr = [];
        for (let j = 0; j <= n; j++) {
          const fj = j / n;
          const lx = ax + dlx * fj, ly = ay + dly * fj, lz = az + dlz * fj;
          const ex = bx + drx * fj - lx, ey = by + dry * fj - ly, ez = bz + drz * fj - lz;
          const row = [];
          for (let i = 0; i <= n; i++) {
            const fi = i / n;
            const k = f / (lz + ez * fi);
            row.push([scx + (lx + ex * fi) * k, scy - (ly + ey * fi) * k]);
          }
          pr.push(row);
        }
        const c4 = [pr[0][0], pr[0][n], pr[n][0], pr[n][n]];
        if (this.offscreen(c4)) return;
        if (opaque(tex)) {
          // Grundfarbe unterlegen: sonst blitzt zwischen zwei Texel-Vierecken der Hintergrund durch
          quads.push([[pr[0][0], pr[0][n], pr[n][n], pr[n][0]], texGrid(tex, 1, shade, ft)[0][4]]);
        }
        for (const [i0, j0, i1, j1, col] of rs) {
          const r0 = pr[j0], r1 = pr[j1];
          quads.push([[r0[i0], r0[i1], r1[i1], r1[i0]], col]);
        }
      } else {
        // Fläche schneidet die Nahebene
        const rows = [];
        for (let j = 0; j <= n; j++) {
          const l = lerp3(ca, cd, j / n), r = lerp3(cb, cc, j / n);
          const row = [];
          for (let i = 0; i <= n; i++) row.push(lerp3(l, r, i / n));
          rows.push(row);
        }
        if (opaque(tex)) {
          const base = clipNear([ca, cb, cc, cd]);
          if (base.length >= 3) quads.push([base.map((c) => this.proj(c)), texGrid(tex, 1, shade, ft)[0][4]]);
        }
        for (const [i0, j0, i1, j1, col] of rs) {
          const r0 = rows[j0], r1 = rows[j1];
          const cellPts = clipNear([r0[i0], r0[i1], r1[i1], r1[i0]]);
          if (cellPts.length >= 3) quads.push([cellPts.map((c) => this.proj(c)), col]);
        }
      }
      if (quads.length) {
        items.push([depth, quads, outline ? clipNear([ca, cb, cc, cd]).map((c) => this.proj(c)) : null, outline]);
      }
    }

    /** Grobes Frustum-Cull anhand des Würfelzentrums (spart Flächen-Arbeit). */
    cull(cx, cy, cz) {
      const c = this.toCam([cx, cy, cz]);
      if (c[2] < -1.7 || c[2] > FOG_END + 2) return true;
      return Math.abs(c[0]) > c[2] * 1.9 + 3.0;
    }

    // ----- Bausteine: Quader (achsenparallel) ------------------------------
    /** Achsenparalleler Quader; tex = [oben, Seite, unten] oder null; lit = feste Helligkeit. */
    addPrism(items, x0, x1, y0, y1, z0, z1, top, side, outline = null, tex = null, fine = true, lit = null) {
      const [px, py, pz] = this._camPos;
      const sc = Math.max(x1 - x0, y1 - y0, z1 - z0);
      const sh = (k) => (lit !== null ? lit : k);
      const face = (q, ti, col, k) => {
        if (tex) this.addFace(items, q, tex[ti], sh(k), outline, sc, fine);
        else this.addPoly(items, q, col, sh(k), outline);
      };
      if (py > y1) face([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], 0, top, 1.0);
      else if (py < y0) face([[x0, y0, z1], [x1, y0, z1], [x1, y0, z0], [x0, y0, z0]], 2, side, 0.5);
      if (px < x0) face([[x0, y1, z1], [x0, y1, z0], [x0, y0, z0], [x0, y0, z1]], 1, side, 0.62);
      else if (px > x1) face([[x1, y1, z0], [x1, y1, z1], [x1, y0, z1], [x1, y0, z0]], 1, side, 0.62);
      if (pz < z0) face([[x1, y1, z0], [x0, y1, z0], [x0, y0, z0], [x1, y0, z0]], 1, side, 0.8);
      else if (pz > z1) face([[x0, y1, z1], [x1, y1, z1], [x1, y0, z1], [x0, y0, z1]], 1, side, 0.8);
    }

    /** Voxel bei (bx,by,bz); nur sichtbare Flächen, deren Nachbar leer ist. */
    addCube(items, bx, by, bz, tex, outline) {
      const [px, py, pz] = this._camPos;
      const x0 = bx, x1 = bx + 1, y0 = by, y1 = by + 1, z0 = bz, z1 = bz + 1;
      if (py > y1 && !this.isSolid(bx, by + 1, bz)) this.addFace(items, [[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], tex[0], 1.0, outline);
      else if (py < y0 && !this.isSolid(bx, by - 1, bz)) this.addFace(items, [[x0, y0, z1], [x1, y0, z1], [x1, y0, z0], [x0, y0, z0]], tex[2], 0.5, outline);
      if (px < x0 && !this.isSolid(bx - 1, by, bz)) this.addFace(items, [[x0, y1, z1], [x0, y1, z0], [x0, y0, z0], [x0, y0, z1]], tex[1], 0.62, outline);
      else if (px > x1 && !this.isSolid(bx + 1, by, bz)) this.addFace(items, [[x1, y1, z0], [x1, y1, z1], [x1, y0, z1], [x1, y0, z0]], tex[1], 0.62, outline);
      if (pz < z0 && !this.isSolid(bx, by, bz - 1)) this.addFace(items, [[x1, y1, z0], [x0, y1, z0], [x0, y0, z0], [x1, y0, z0]], tex[1], 0.8, outline);
      else if (pz > z1 && !this.isSolid(bx, by, bz + 1)) this.addFace(items, [[x0, y1, z1], [x1, y1, z1], [x1, y0, z1], [x0, y0, z1]], tex[1], 0.8, outline);
    }

    // ----- Bausteine: gedrehte Quader (Spielfigur) -------------------------
    /**
     * Um die Y-Achse (yaw) und um die lokale X-Achse (swing) gedrehter Quader.
     * lo/hi = lokale Grenzen (x = rechts, y = hoch, z = vorne) relativ zum
     * Drehpunkt base; so schwingen Arme und Beine um Schulter bzw. Hüfte.
     */
    addObox(items, base, yaw, swing, lo, hi, texFront, texSide, texTop, texBottom = null, texBack = null) {
      const cs = Math.cos(swing), sn = Math.sin(swing);
      const fx = Math.sin(yaw), fz = Math.cos(yaw);
      const rx = -fz, rz = fx;
      const [bx, by, bz] = base;
      const w = (x, y, z) => {
        const yy = y * cs - z * sn;
        const zz = y * sn + z * cs;
        return [bx + x * rx + zz * fx, by + yy, bz + x * rz + zz * fz];
      };
      const [x0, y0, z0] = lo;
      const [x1, y1, z1] = hi;
      // 8 Ecken (lokal) -> Welt; Index = xi*4 + yi*2 + zi
      const p = new Array(8);
      for (const [xi, xv] of [[0, x0], [1, x1]]) for (const [yi, yv] of [[0, y0], [1, y1]]) for (const [zi, zv] of [[0, z0], [1, z1]]) p[xi * 4 + yi * 2 + zi] = w(xv, yv, zv);
      const P = (xi, yi, zi) => p[xi * 4 + yi * 2 + zi];
      const cam = this._camPos;
      const cen = w((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2);
      const sc = Math.max(x1 - x0, y1 - y0);

      const face = (quad, tex, shade) => {
        if (!tex) return;
        const a = quad[0], b = quad[1], c = quad[3];
        const ux = b[0] - a[0], uy = b[1] - a[1], uz = b[2] - a[2];
        const vx = c[0] - a[0], vy = c[1] - a[1], vz = c[2] - a[2];
        let nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
        const mid = [(quad[0][0] + quad[2][0]) / 2, (quad[0][1] + quad[2][1]) / 2, (quad[0][2] + quad[2][2]) / 2];
        if (nx * (mid[0] - cen[0]) + ny * (mid[1] - cen[1]) + nz * (mid[2] - cen[2]) < 0) {
          nx = -nx;
          ny = -ny;
          nz = -nz;
        }
        if (nx * (cam[0] - mid[0]) + ny * (cam[1] - mid[1]) + nz * (cam[2] - mid[2]) <= 0) return; // Rückseite
        this.addFace(items, quad, tex, shade, null, sc, true);
      };

      face([P(0, 1, 1), P(1, 1, 1), P(1, 0, 1), P(0, 0, 1)], texFront, 0.94);
      face([P(1, 1, 0), P(0, 1, 0), P(0, 0, 0), P(1, 0, 0)], texBack || texFront, 0.66);
      face([P(1, 1, 1), P(1, 1, 0), P(1, 0, 0), P(1, 0, 1)], texSide, 0.78);
      face([P(0, 1, 0), P(0, 1, 1), P(0, 0, 1), P(0, 0, 0)], texSide, 0.72);
      face([P(0, 1, 0), P(1, 1, 0), P(1, 1, 1), P(0, 1, 1)], texTop, 1.0);
      face([P(0, 0, 1), P(1, 0, 1), P(1, 0, 0), P(0, 0, 0)], texBottom || texSide, 0.45);
    }

    /** Frei rotierendes Item-Sprite (Gold-Barren) mit durchsichtigem Rand. */
    addSprite(items, center, size, tex, spin) {
      const [cx, cy, cz] = center;
      const h = size * 0.5;
      const rx = Math.cos(spin) * h, rz = Math.sin(spin) * h;
      const quad = [[cx - rx, cy + h, cz - rz], [cx + rx, cy + h, cz + rz], [cx + rx, cy - h, cz + rz], [cx - rx, cy - h, cz - rz]];
      this.addFace(items, quad, tex, 1.0, null, size, true);
    }

    // ----- Weltobjekte ----------------------------------------------------
    addBlock(items, bx, by, bz, typ) {
      if (this.cull(bx + 0.5, by + 0.5, bz + 0.5)) return;
      const line = this.texMode === 0 ? "rgb(30,34,40)" : null;
      if (BLOCK_TEX[typ]) {
        this.addCube(items, bx, by, bz, BLOCK_TEX[typ], line);
      } else if (typ === LADDER) {
        const wood = ["wood", "wood", "wood"];
        for (const sx of [0.14, 0.66]) {
          // zwei Holme
          this.addPrism(items, bx + sx, bx + sx + 0.2, by, by + 1, bz + 0.06, bz + 0.24, COL_LADDER, shadeCol(COL_LADDER, 0.8), line, wood);
        }
        for (let ry = 0; ry < 4; ry++) {
          // Sprossen
          const yy = by + 0.12 + ry * 0.26;
          this.addPrism(items, bx + 0.1, bx + 0.9, yy, yy + 0.09, bz + 0.03, bz + 0.27, shadeCol(COL_LADDER, 1.15), COL_LADDER, null, wood);
        }
      } else if (typ === FENCE) {
        const wood = ["wood", "wood", "wood"];
        this.addPrism(items, bx + 0.38, bx + 0.62, by, by + 1, bz + 0.38, bz + 0.62, COL_FENCE, shadeCol(COL_FENCE, 0.8), line, wood);
        for (const yy of [by + 0.34, by + 0.66]) {
          // zwei Querbalken je Achse
          this.addPrism(items, bx + 0.02, bx + 0.98, yy, yy + 0.13, bz + 0.44, bz + 0.56, COL_FENCE, shadeCol(COL_FENCE, 0.85), null, wood);
          this.addPrism(items, bx + 0.44, bx + 0.56, yy, yy + 0.13, bz + 0.02, bz + 0.98, COL_FENCE, shadeCol(COL_FENCE, 0.85), null, wood);
        }
      } else if (typ === SPRING) {
        const slime = ["slime", "slime", "slime"];
        this.addPrism(items, bx + 0.02, bx + 0.98, by, by + SPRING_H, bz + 0.02, bz + 0.98, COL_SLIME, shadeCol(COL_SLIME, 0.85), line, slime, false);
      }
    }

    /** Beacon-Strahl über dem Ziel. */
    addGoalBeam(items) {
      const [gx, gy, gz] = this.goal;
      const p = 0.5 + 0.5 * Math.sin(this.anim * 3.0);
      const w = 0.24 + 0.03 * p;
      const beam = ["beacon", "beacon", "beacon"];
      this.addPrism(items, gx - w, gx + w, gy + 0.05, gy + 18.0, gz - w, gz + w, [240, 255, 250], [215, 248, 240], null, beam, false, 1.0);
    }

    /** Steve: Kopf, Körper, Arme, Beine - mit Laufanimation. */
    addPlayer(items) {
      const px = this.px, py = this.py, pz = this.pz;
      const u = PLAYER_H / 32.0; // ein Skin-Pixel in Weltmaß
      const sp = Math.min(1.0, Math.hypot(this.vx, this.vz) / MOVE_SPEED);
      const swing = this.onGround ? Math.sin(this.walk) * 0.85 * sp : this.vy > 0 ? 0.45 : 0.22;
      const yaw = this.yaw;
      const headY = py + 24 * u;
      const armY = py + 23.5 * u;
      const hipY = py + 12 * u;

      // Beine (schwingen gegengleich)
      for (const [side, sgn] of [[-1, 1], [1, -1]]) {
        this.addObox(items, [px, hipY, pz], yaw, swing * sgn, [side * 4 * u - 2 * u, -12 * u, -2 * u], [side * 4 * u + 2 * u, 0.0, 2 * u], "leg", "leg", "leg", "leg");
      }
      // Torso
      this.addObox(items, [px, py, pz], yaw, 0.0, [-4 * u, 12 * u, -2 * u], [4 * u, 24 * u, 2 * u], "body_front", "body_side", "body_side", "body_side", "body_side");
      // Arme
      for (const [side, sgn] of [[-1, -1], [1, 1]]) {
        this.addObox(items, [px, armY, pz], yaw, swing * sgn, [side * 6 * u - 2 * u, -12 * u, -2 * u], [side * 6 * u + 2 * u, 0.0, 2 * u], "arm", "arm", "arm", "arm");
      }
      // Kopf (neigt sich mit dem Blick)
      this.addObox(items, [px, headY, pz], yaw, -this.pitch * 0.7, [-4 * u, 0.0, -4 * u], [4 * u, 8 * u, 4 * u], "face", "head_side", "head_top", "head_top", "head_back");
    }

    /** Zeichenliste nach Tiefe sortiert malen (fern zuerst). */
    renderItems(c, items) {
      items.sort((a, b) => b[0] - a[0]);
      c.lineJoin = "round";
      c.lineWidth = SEAM;
      for (const it of items) {
        const quads = it[1];
        for (let qi = 0; qi < quads.length; qi++) {
          const pts = quads[qi][0];
          c.beginPath();
          c.moveTo(pts[0][0], pts[0][1]);
          for (let i = 1; i < pts.length; i++) c.lineTo(pts[i][0], pts[i][1]);
          c.closePath();
          c.fillStyle = quads[qi][1];
          c.fill();
          if (qi === 0) {
            // Grundfläche minimal aufdicken -> keine hellen Säume zwischen Blöcken
            c.strokeStyle = quads[qi][1];
            c.stroke();
          }
        }
        if (it[2]) {
          const pts = it[2];
          c.beginPath();
          c.moveTo(pts[0][0], pts[0][1]);
          for (let i = 1; i < pts.length; i++) c.lineTo(pts[i][0], pts[i][1]);
          c.closePath();
          c.strokeStyle = it[3];
          c.lineWidth = 1;
          c.stroke();
          c.lineWidth = SEAM;
        }
      }
    }

    // ===================================================== Zeichnen
    sceneCanvas() {
      const ps = Math.max(1, Math.min(2, (PG.app && PG.app.pixelScale) || 1));
      const w = Math.round(this.width * ps), h = Math.round(this.height * ps);
      if (!this._scene || this._scene.width !== w || this._scene.height !== h) {
        this._scene = ui.makeCanvas(w, h);
        this._sceneCtx = this._scene.getContext("2d");
        this._prev = ui.makeCanvas(w, h);
        this._prevCtx = this._prev.getContext("2d");
        this._hasPrev = false;
      }
      this._sceneCtx.setTransform(ps, 0, 0, ps, 0, 0);
      return this._sceneCtx;
    }

    draw(ctx) {
      this.updateCam();
      this._scx = this.width / 2;
      this._scy = this.height * 0.5;
      this._f = this.height * FOV_MUL;
      this._basis = this.viewBasis();

      const useBlur = this.blur > 0.01;
      const sc = useBlur ? this.sceneCanvas() : ctx;
      this.drawSky(sc);
      this.drawSunAndClouds(sc);

      const items = [];
      for (const b of this.blockList) this.addBlock(items, b[0], b[1], b[2], b[3]);
      for (const coin of this.coins) {
        const cy = coin[1] + 0.12 * Math.sin(this.anim * 2 + coin[0]);
        this.addSprite(items, [coin[0], cy, coin[2]], 0.45, "ingot", this.anim * 1.9 + coin[0] + coin[2]);
      }
      this.addGoalBeam(items);
      if (this.view === "third") this.addPlayer(items);
      this.renderItems(sc, items);

      if (useBlur) this.applyBlur(ctx);
      else this._hasPrev = false;

      if (this.view === "first" && this.state !== GAMEOVER) {
        this.drawHand(ctx);
        if (this.state === PLAY) this.drawCrosshair(ctx);
      }
      this.drawHud(ctx);
      if (this.state === READY) {
        this.banner(ctx, t("blj.ready"), this.accent, t("blj.controls"));
      } else if (this.state === CLEAR) {
        this.banner(ctx, t("blj.clear", { n: this.level }), [120, 235, 170], t("blj.clear_sub", { pts: 1000 + this._lastBonus }));
      } else if (this.state === GAMEOVER) {
        this.banner(ctx, t("blj.gameover"), [232, 96, 96], t("common.points", { score: this.score }) + "   ·   " + t("common.enter_restart"));
      }
    }

    // ----- Himmel, Sonne, Wolken -----------------------------------------
    drawSky(c) {
      if (!this._skyGrad || this._skyCtx !== c) {
        const g = c.createLinearGradient(0, 0, 0, this.height);
        g.addColorStop(0, css(COL_SKY_TOP));
        g.addColorStop(0.52, css(COL_SKY_HOR));
        g.addColorStop(0.66, css(COL_FOG));
        g.addColorStop(1, css(COL_FOG));
        this._skyGrad = g;
        this._skyCtx = c;
      }
      c.fillStyle = this._skyGrad;
      c.fillRect(0, 0, this.width, this.height);
    }

    /** Quadratische Sonne + driftende Pixel-Wolken (immer hinter der Welt). */
    drawSunAndClouds(c) {
      const sky = [];
      const [cx, cy, cz] = this._camPos;
      // Sonne: feste Weltrichtung, weit weg, ohne Nebel
      const d = 260.0, r = 19.0;
      let sv = [0.52, 0.66, 0.54];
      const sl = Math.hypot(sv[0], sv[1], sv[2]);
      sv = sv.map((v) => v / sl);
      const sc = [cx + sv[0] * d, cy + sv[1] * d, cz + sv[2] * d];
      // Sonnenscheibe als Billboard zur Kamera: bleibt ein sauberes Quadrat
      const ax = this._basis[0], ay = this._basis[1];
      for (const [k, col] of [[1.7, ui.mix(COL_SUN, COL_SKY_HOR, 0.6)], [1.0, COL_SUN]]) {
        const q = [[-1, 1], [1, 1], [1, -1], [-1, -1]].map(([sx, sy]) => [
          sc[0] + ax[0] * r * k * sx + ay[0] * r * k * sy,
          sc[1] + ax[1] * r * k * sx + ay[1] * r * k * sy,
          sc[2] + ax[2] * r * k * sx + ay[2] * r * k * sy,
        ]);
        this.addPoly(sky, q, col, 1.0, null, false);
      }

      // Wolkenfeld: 8x8-Maske, Kacheln zu 8 Blöcken, driftet langsam in x
      const cyl = this._cloudY != null ? this._cloudY : 50.0;
      if (cy < cyl - 1.0) {
        const T = 7.0;
        const drift = PG.mod(this.anim * 0.35, T * 8);
        const tx0 = Math.floor((cx - drift) / T);
        const tz0 = Math.floor(cz / T);
        const span = 11;
        for (let tz = tz0 - span; tz <= tz0 + span; tz++) {
          for (let tx = tx0 - span; tx <= tx0 + span; tx++) {
            if (CLOUD_MAP[PG.mod(tz, 8)][PG.mod(tx, 8)] === "0") continue;
            const x0 = tx * T + drift;
            const z0 = tz * T;
            const mx = x0 + T * 0.5, mz = z0 + T * 0.5;
            const dist = Math.hypot(mx - cx, mz - cz);
            if (dist > span * T) continue;
            // eigenes Cull: die Wolken liegen weit außerhalb der Nebelgrenze
            const cc = this.toCam([mx, cyl, mz]);
            if (cc[2] < 0.5 || Math.abs(cc[0]) > cc[2] * 2.0 + T) continue;
            const ft = Math.min(1.0, Math.pow(dist / (span * T), 2.6));
            const col = ui.mix([250, 251, 255], COL_FOG, ft);
            this.addPoly(sky, [[x0, cyl, z0], [x0 + T, cyl, z0], [x0 + T, cyl, z0 + T], [x0, cyl, z0 + T]], col, 0.94, null, false);
          }
        }
      }
      this.renderItems(c, sky);
    }

    /** Motion Blur: Vorbild mit Alpha über die Szene, Ergebnis wird neues Vorbild. */
    applyBlur(ctx) {
      const sc = this._sceneCtx;
      if (this._hasPrev) {
        sc.save();
        sc.setTransform(1, 0, 0, 1, 0, 0);
        sc.globalAlpha = Math.trunc(this.blur * 230) / 255;
        sc.drawImage(this._prev, 0, 0);
        sc.restore();
      }
      const pc = this._prevCtx;
      pc.globalCompositeOperation = "copy";
      pc.drawImage(this._scene, 0, 0);
      pc.globalCompositeOperation = "source-over";
      this._hasPrev = true;
      ctx.drawImage(this._scene, 0, 0, this.width, this.height);
    }

    // ----- Ego-Hand -------------------------------------------------------
    /** Minecraft-Hand unten rechts, mit Lauf-Bob. */
    drawHand(ctx) {
      const w = this.width, h = this.height;
      const sp = Math.min(1.0, Math.hypot(this.vx, this.vz) / MOVE_SPEED);
      const bx = Math.sin(this.walk) * 0.02 * w * sp;
      let by = Math.abs(Math.cos(this.walk)) * 0.03 * h * sp;
      if (!this.onGround) by -= 0.02 * h;
      const ax = w * 0.97 + bx, ay = h * 1.16 + by; // Schulter (außerhalb)
      const fx = w * 0.79 + bx, fy = h * 0.78 + by; // Faust
      let dx = fx - ax, dy = fy - ay;
      const ln = Math.hypot(dx, dy) || 1.0;
      dx /= ln;
      dy /= ln;
      const nx = -dy, ny = dx;
      const hw = h * 0.055; // halbe Armbreite

      const p0 = [ax, ay], p1 = [ax + dx * ln, ay + dy * ln];
      this.blitTexQuad(ctx, [[p1[0] + nx * hw, p1[1] + ny * hw], [p1[0] - nx * hw, p1[1] - ny * hw], [p0[0] - nx * hw, p0[1] - ny * hw], [p0[0] + nx * hw, p0[1] + ny * hw]], "arm", 0.98);
      // schmale Schattenkante für die Tiefe
      const edge = [
        [fx - nx * hw, fy - ny * hw],
        [fx - nx * hw - dx * h * 0.02, fy - ny * hw - dy * h * 0.02],
        [ax - nx * hw - dx * h * 0.02, ay - ny * hw - dy * h * 0.02],
        [ax - nx * hw, ay - ny * hw],
      ];
      this.blitTexQuad(ctx, edge, "arm", 0.62);
      // Handrücken (Stirnfläche der Faust)
      const cap = [
        [fx + nx * hw, fy + ny * hw],
        [fx - nx * hw, fy - ny * hw],
        [fx - nx * hw + dx * -h * 0.035, fy - ny * hw + dy * -h * 0.035],
        [fx + nx * hw + dx * -h * 0.035, fy + ny * hw + dy * -h * 0.035],
      ];
      this.blitTexQuad(ctx, cap, "arm", 1.12);
    }

    /** Zeichnet eine Textur in ein 2D-Viereck (Bildschirmkoordinaten). */
    blitTexQuad(ctx, quad, tex, shade = 1.0, n = null) {
      const poly = (pts, col) => {
        ctx.beginPath();
        ctx.moveTo(pts[0][0], pts[0][1]);
        for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1]);
        ctx.closePath();
        ctx.fillStyle = col;
        ctx.fill();
      };
      if (this.texMode === 0) {
        poly(quad, css(shadeCol(avgCol(tex), shade)));
        return;
      }
      n = Math.min(n || (this.texMode === 2 ? 8 : 4), TEX[tex].length);
      const [a, b, c, d] = quad;
      const rows = [];
      for (let j = 0; j <= n; j++) {
        const l = [a[0] + ((d[0] - a[0]) * j) / n, a[1] + ((d[1] - a[1]) * j) / n];
        const r = [b[0] + ((c[0] - b[0]) * j) / n, b[1] + ((c[1] - b[1]) * j) / n];
        const row = [];
        for (let i = 0; i <= n; i++) row.push([l[0] + ((r[0] - l[0]) * i) / n, l[1] + ((r[1] - l[1]) * i) / n]);
        rows.push(row);
      }
      // Grundfarbe unterlegen (Canvas-Antialiasing würde sonst Säume zeigen)
      poly(quad, texGrid(tex, 1, shade, 0.0)[0][4]);
      for (const [i0, j0, i1, j1, col] of texGrid(tex, n, shade, 0.0)) {
        const r0 = rows[j0], r1 = rows[j1];
        poly([r0[i0], r0[i1], r1[i1], r1[i0]], col);
      }
    }

    // ----- HUD ------------------------------------------------------------
    drawCrosshair(ctx) {
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      for (const [col, off, ln, wd] of [[[26, 28, 34], 1, 10, 4], [[242, 242, 242], 0, 9, 2]]) {
        draw.line(ctx, col, [cx - ln, cy + off], [cx + ln, cy + off], wd);
        draw.line(ctx, col, [cx + off, cy - ln], [cx + off, cy + ln], wd);
      }
    }

    /** Zeichnet Pixel-Art (Zeichenraster) mit px Pixel je Texel. */
    blitArt(ctx, rows, pal, x, y, px) {
      for (let j = 0; j < rows.length; j++) {
        const row = rows[j];
        for (let i = 0; i < row.length; i++) {
          const c = pal[row[i]];
          if (c) {
            ctx.fillStyle = c;
            ctx.fillRect(x + i * px, y + j * px, px, px);
          }
        }
      }
    }

    /** Minecraft-Schrift: harter dunkler Schatten unten rechts. */
    shadowText(ctx, font, txt, col, pos, anchor = "midleft") {
      ui.text(ctx, txt, pos[0] + 2, pos[1] + 2, font, [28, 28, 32], anchor);
      return ui.text(ctx, txt, pos[0], pos[1], font, col, anchor);
    }

    drawHud(ctx) {
      this.shadowText(ctx, this._hud, t("blj.level", { n: this.level }), this.accent, [14, 20]);
      this.shadowText(ctx, this._hud, t("common.points", { score: this.score }), [245, 245, 245], [Math.floor(this.width / 2), 20], "center");
      // Herzen (Leben) unten links
      const px = Math.max(2, Math.floor(this.height / 230));
      const hw = 10 * px;
      let y = this.height - 14 - 9 * px;
      for (let i = 0; i < 3; i++) this.blitArt(ctx, HEART_ART, i < this.lives ? HEART_PAL : HEART_EMPTY_PAL, 14 + i * (hw + px * 2), y, px);
      // Gold-Barren + Zähler unten rechts
      const ing = mip("ingot", 8);
      const ix = this.width - 14 - 8 * px;
      for (let j = 0; j < ing.length; j++) {
        for (let i = 0; i < ing[j].length; i++) {
          const c = ing[j][i];
          if (c >= 0) {
            ctx.fillStyle = css(unpack(c));
            ctx.fillRect(ix + i * px, y + j * px, px, px);
          }
        }
      }
      this.shadowText(ctx, this._hud, String(this.coinsLevel), ui.GOLD, [ix - 8, y + 4 * px], "midright");
      if (this.state === PLAY) {
        const mdir = this.invert ? t("common.dir_inverted") : t("common.dir_normal");
        const hint = t("blj.hud_hint", {
          view: this.view === "first" ? t("blj.view_1p") : t("blj.view_3p"),
          tex: t("blj.tex_" + TEX_MODES[this.texMode]),
          dir: mdir,
        });
        let lines = [hint];
        if (this._small.width(hint) > this.width - 28) {
          const parts = hint.split(" · "); // bei schmalem Fenster 2-zeilig
          const hh = Math.floor((parts.length + 1) / 2);
          lines = [parts.slice(0, hh).join(" · "), parts.slice(hh).join(" · ")];
        }
        y = this.height - 10 - (lines.length - 1) * (this._small.height + 2);
        for (const ln of lines) {
          this.shadowText(ctx, this._small, ln, [226, 226, 230], [Math.floor(this.width / 2), y], "midbottom");
          y += this._small.height + 2;
        }
      }
    }

    banner(ctx, title, color, sub) {
      const w = Math.min(this.width - 40, 620);
      const h = 124;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), Math.floor((this.height - h) / 2), w, h);
      ctx.fillStyle = "rgba(16,16,20," + (226 / 255).toFixed(3) + ")";
      ctx.fillRect(rc.x, rc.y, w, h);
      // Doppelrahmen im GUI-Stil von Minecraft
      draw.rect(ctx, [78, 78, 86], rc, 4);
      draw.rect(ctx, color, rc.inflate(-8, -8), 2);
      this.shadowText(ctx, this._huge, title, color, [rc.centerx, rc.y + 44], "center");
      this.shadowText(ctx, this._small, sub, [232, 232, 236], [rc.centerx, rc.y + 88], "center");
    }
  }

  PG.register(BlockJumpGame, {
    id: "BlockJumpGame",
    key: "blockjump",
    name: "Block Jump",
    modes: [["easy", "blj.mode.easy"], ["normal", "blj.mode.normal"], ["hard", "blj.mode.hard"]],
    settingsKey: "blockjump",
    defaults: { view: "first", blur: 0.35, sens: 1.0, mouse_invert: false, textures: "high" },
  });
})();
