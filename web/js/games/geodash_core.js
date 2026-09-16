/*
 * geodash_core.js - Physik-Kern von Geometry Dash (Port von games/geodash_core.py)
 * ===============================================================================
 * Ohne DOM, ohne Zeichnen, ohne Zufall - läuft im Browser (PG.gdCore) und in
 * Node (require), damit web/tools/geodash_replay.js die Solver-Lösungen
 * nachspielen und die Zustands-Hashes mit Python vergleichen kann.
 *
 * BITGLEICH mit Python: nur Ganzzahlen (Festkomma, ein Block = B = 36000
 * Einheiten, 240 Schritte je Sekunde), nur + - * und Vergleiche. Die einzige
 * Division ist fdiv() = Math.floor(a / b) mit b > 0 (für |a| < 2^40 exakt wie
 * Pythons //). Objekte werden in Listen-Reihenfolge geprüft, Spalten von links
 * nach rechts - genau wie in geodash_core.py. Wer hier etwas ändert, ändert es
 * dort auch (der Test vergleicht die Hashes).
 */
(function (root) {
  "use strict";

  const B = 36000;
  const HZ = 240;
  const HALF = B / 2;
  const SPEEDS = [1260, 1560, 1944, 2340];
  const SPEED_LABELS = ["0.5x", "1x", "2x", "3x"];
  const CUBE = 0, SHIP = 1, BALL = 2, UFO = 3, WAVE = 4;
  const MODE_NAMES = ["cube", "ship", "ball", "ufo", "wave"];
  const GRAVITY = [66, 0, 52, 44, 0];
  const FALL_MAX = [4400, 0, 3800, 3000, 0];
  const CUBE_JUMP = 3300;
  const UFO_JUMP = 2150;
  const BALL_PUSH = 1300;
  const SHIP_UP = 40;
  const SHIP_DOWN = 36;
  const SHIP_MAX_UP = 1700;
  const SHIP_MAX_DOWN = 1900;
  const ORB_V = [[3450, 2500, 1500], [2400, 1700, 1200], [3000, 2200, 1500], [2600, 1900, 1400], [0, 0, 0]];
  const PAD_V = [[4500, 2700, 2600], [2800, 1900, 1600], [3800, 2400, 2400], [3400, 2300, 2200], [0, 0, 0]];
  const CORRIDOR = 10 * B;
  const BUFFER = 20;
  const SNAP = 9000;
  const INNER = 10800;
  const INNER_OFF = (B - INNER) / 2;
  const WAVE_SIZE = 12000;
  const WAVE_OFF = (B - WAVE_SIZE) / 2;
  const FALL_DEATH = -2 * B;
  const MAX_Y = 48 * B;
  const MAX_LENGTH = 4000;
  const MAX_ROW = 40;
  const START_X = 0;
  const MIN_LENGTH = 40;

  const fdiv = (a, b) => Math.floor(a / b);

  const C_SOLID = 0, C_HAZARD = 1, C_PIT = 2, C_PAD = 3, C_ORB = 4, C_MODE = 5, C_GRAV = 6, C_SPEED = 7, C_COIN = 8, C_TRIGGER = 9;

  const KINDS = [
    ["block", C_SOLID, 0, [0, 0, 100, 100]],
    ["half", C_SOLID, 0, [0, 0, 100, 50]],
    ["spike", C_HAZARD, 0, [40, 22, 60, 64]],
    ["spike_s", C_HAZARD, 0, [40, 4, 60, 32]],
    ["pit", C_PIT, 0, [0, 0, 100, 100]],
    ["pad_y", C_PAD, 0, [8, 0, 92, 22]],
    ["pad_p", C_PAD, 1, [8, 0, 92, 22]],
    ["pad_b", C_PAD, 2, [8, 0, 92, 22]],
    ["orb_y", C_ORB, 0, [-12, -12, 112, 112]],
    ["orb_p", C_ORB, 1, [-12, -12, 112, 112]],
    ["orb_b", C_ORB, 2, [-12, -12, 112, 112]],
    ["p_cube", C_MODE, CUBE, [15, -100, 85, 200]],
    ["p_ship", C_MODE, SHIP, [15, -100, 85, 200]],
    ["p_ball", C_MODE, BALL, [15, -100, 85, 200]],
    ["p_ufo", C_MODE, UFO, [15, -100, 85, 200]],
    ["p_wave", C_MODE, WAVE, [15, -100, 85, 200]],
    ["g_norm", C_GRAV, 1, [15, -100, 85, 200]],
    ["g_flip", C_GRAV, -1, [15, -100, 85, 200]],
    ["s_slow", C_SPEED, 0, [0, -100, 100, 200]],
    ["s_norm", C_SPEED, 1, [0, -100, 100, 200]],
    ["s_fast", C_SPEED, 2, [0, -100, 100, 200]],
    ["s_vfast", C_SPEED, 3, [0, -100, 100, 200]],
    ["coin", C_COIN, 0, [10, 10, 90, 90]],
    ["color", C_TRIGGER, 0, [0, 0, 0, 0]],
  ];
  const KIND_NAMES = KINDS.map((k) => k[0]);
  const KIND_ID = {};
  KINDS.forEach((k, i) => (KIND_ID[k[0]] = i));
  const ROTATE_ALL = ["half", "spike", "spike_s"];
  const ROTATE_FLIP = ["pad_y", "pad_p", "pad_b"];
  const PARAMS = {
    p_ship: [[0, 0, 30]], p_ball: [[0, 0, 30]], p_ufo: [[0, 0, 30]], p_wave: [[0, 0, 30]],
    color: [[0, 0, 1], [255, 0, 255], [255, 0, 255], [255, 0, 255], [6, 0, 60]],
  };
  const MAX_COINS = 3;

  /** Dreht eine Hitbox rot-mal um 90° im Uhrzeigersinn: (x, y) -> (y, B - x). */
  function rotateBox(box, rot) {
    let [x0, y0, x1, y1] = box;
    for (let r = 0; r < rot; r++) [x0, y0, x1, y1] = [y0, B - x1, y1, B - x0];
    return [x0, y0, x1, y1];
  }

  const HITBOX = KINDS.map(([, , , [a, b, c, d]]) => {
    const base = [(a * B) / 100, (b * B) / 100, (c * B) / 100, (d * B) / 100];
    return [0, 1, 2, 3].map((r) => rotateBox(base, r));
  });

  // -------------------------------------------------------------------------
  //  Level-Daten prüfen
  // -------------------------------------------------------------------------
  const DEFAULT_BG = [40, 110, 255];
  const DEFAULT_GROUND = [20, 70, 200];
  const MUSIC_STYLES = ["drive", "chip", "dream", "dark", "none"];

  function toInt(v, lo, hi, def) {
    if (typeof v === "boolean" || v === null || v === undefined || v === "") return def;
    const n = Number(v);
    if (!Number.isFinite(n)) return def;
    return Math.max(lo, Math.min(hi, Math.trunc(n)));
  }

  function color(v, def) {
    if (Array.isArray(v) && v.length >= 3) return [toInt(v[0], 0, 255, 0), toInt(v[1], 0, 255, 0), toInt(v[2], 0, 255, 0)];
    return def.slice();
  }

  function normalizeObject(o) {
    if (!Array.isArray(o) || o.length < 3) return null;
    const kind = o[0];
    if (!Object.prototype.hasOwnProperty.call(KIND_ID, kind)) return null;
    const x = toInt(o[1], 0, MAX_LENGTH, -1);
    let y = toInt(o[2], 0, MAX_ROW, -1);
    if (x < 0 || y < 0) return null;
    let rot = toInt(o.length > 3 ? o[3] : 0, 0, 3, 0);
    if (ROTATE_FLIP.includes(kind)) rot = rot === 1 || rot === 2 ? 2 : 0;
    else if (!ROTATE_ALL.includes(kind)) rot = 0;
    if (kind === "pit") y = 0;
    const out = [kind, x, y, rot];
    (PARAMS[kind] || []).forEach(([def, lo, hi], i) => {
      const raw = o.length > 4 + i ? o[4 + i] : def;
      out.push(toInt(raw, lo, hi, def));
    });
    return out;
  }

  function compareObjects(a, b) {
    return a[1] - b[1] || a[2] - b[2] || KIND_ID[a[0]] - KIND_ID[b[0]] || a[3] - b[3];
  }

  /** Wie normalize_level() in Python: sauberes Level-Objekt (Kopie). */
  function normalizeLevel(d) {
    d = d && typeof d === "object" && !Array.isArray(d) ? Object.assign({}, d) : {};
    const objs = [];
    let coins = 0;
    const seen = new Set();
    for (const o of d.objects || []) {
      const n = normalizeObject(o);
      if (!n) continue;
      if (n[0] === "coin") {
        if (coins >= MAX_COINS) continue;
        coins++;
      }
      const key = n.join(",");
      if (seen.has(key)) continue;
      seen.add(key);
      objs.push(n);
    }
    objs.sort(compareObjects);
    d.objects = objs;
    d.speed = toInt(d.speed === undefined ? 1 : d.speed, 0, 3, 1);
    d.mode = toInt(d.mode === undefined ? CUBE : d.mode, 0, 4, CUBE);
    d.bg = color(d.bg, DEFAULT_BG);
    d.ground = color(d.ground, DEFAULT_GROUND);
    if (!MUSIC_STYLES.includes(d.music)) d.music = "drive";
    d.length = toInt(d.length === undefined ? 0 : d.length, 0, MAX_LENGTH + 20, 0);
    return d;
  }

  function levelLength(d) {
    let last = 0;
    for (const o of d.objects || []) if (o[0] !== "color") last = Math.max(last, o[1]);
    return Math.max(MIN_LENGTH, Math.trunc(Number(d.length) || 0), last + 12);
  }

  function fnv1a(text) {
    let h = 0x811c9dc5;
    for (let i = 0; i < text.length; i++) {
      h ^= text.charCodeAt(i) & 0xff;
      h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h >>> 0;
  }

  const hex8 = (n) => (n >>> 0).toString(16).padStart(8, "0");

  function contentHash(d) {
    d = normalizeLevel(d);
    const parts = [`${d.speed}|${d.mode}|${levelLength(d)}`];
    for (const o of d.objects) if (o[0] !== "color") parts.push(o.join(","));
    return hex8(fnv1a(parts.join(";")));
  }

  // -------------------------------------------------------------------------
  //  Kompiliertes Level
  // -------------------------------------------------------------------------
  function compile(d) {
    d = normalizeLevel(d);
    const lv = {
      data: d, kind: [], cls: [], val: [], box: [], par: [], ox: [], oy: [], rot: [],
      cols: new Map(), pits: new Set(), coinBit: new Map(), triggers: [], n: 0,
      coinCount: 0, length: 0, endX: 0, speed: d.speed, mode: d.mode,
    };
    let coins = 0;
    d.objects.forEach((o, i) => {
      const k = KIND_ID[o[0]];
      const [, cls, val] = KINDS[k];
      const cx = o[1] * B, cy = o[2] * B, rot = o[3];
      const hb = HITBOX[k][rot];
      const box = [cx + hb[0], cy + hb[1], cx + hb[2], cy + hb[3]];
      lv.kind.push(k);
      lv.cls.push(cls);
      lv.val.push(val);
      lv.box.push(box);
      lv.par.push(o.slice(4));
      lv.ox.push(o[1]);
      lv.oy.push(o[2]);
      lv.rot.push(rot);
      if (cls === C_PIT) {
        lv.pits.add(o[1]);
        return;
      }
      if (cls === C_TRIGGER) {
        lv.triggers.push(i);
        return;
      }
      if (cls === C_COIN) {
        lv.coinBit.set(i, 1 << coins);
        coins++;
      }
      const c0 = fdiv(box[0], B), c1 = fdiv(box[2] - 1, B);
      for (let c = c0; c <= c1; c++) {
        let lst = lv.cols.get(c);
        if (!lst) {
          lst = [];
          lv.cols.set(c, lst);
        }
        lst.push(i);
      }
    });
    lv.n = d.objects.length;
    lv.coinCount = coins;
    lv.length = levelLength(d);
    lv.endX = lv.length * B;
    return lv;
  }

  // -------------------------------------------------------------------------
  //  Zustand
  // -------------------------------------------------------------------------
  const EMPTY = new Set();

  function copyState(s) {
    return {
      step: s.step, x: s.x, y: s.y, vy: s.vy, mode: s.mode, grav: s.grav, speed: s.speed,
      ground: s.ground, ceil: s.ceil, buf: s.buf, dead: s.dead, won: s.won, coins: s.coins, used: s.used,
    };
  }

  function newState(lv, startBlock) {
    const s = {
      step: 0, x: START_X, y: 0, vy: 0, mode: lv.mode, grav: 1, speed: lv.speed, ground: 1,
      ceil: lv.mode !== CUBE ? CORRIDOR : 0, buf: 0, dead: 0, won: 0, coins: 0, used: EMPTY,
    };
    if (startBlock === undefined || startBlock === null || startBlock <= 0) return s;
    s.x = startBlock * B;
    for (let i = 0; i < lv.n; i++) {
      if (lv.ox[i] >= startBlock) break;
      const cls = lv.cls[i];
      if (cls === C_MODE) setMode(s, lv.val[i], lv.par[i]);
      else if (cls === C_GRAV) s.grav = lv.val[i];
      else if (cls === C_SPEED) s.speed = lv.val[i];
    }
    s.vy = 0;
    if (s.mode === CUBE || s.mode === BALL) {
      if (s.grav > 0) s.y = stackTop(lv, startBlock);
      else s.y = Math.max(0, stackBottom(lv, startBlock) - B);
    } else {
      s.y = s.ceil > 0 ? fdiv(s.ceil - B, 2) : 3 * B;
      s.ground = 0;
    }
    return s;
  }

  function stackTop(lv, col) {
    let top = 0;
    let changed = true;
    while (changed) {
      changed = false;
      for (const i of lv.cols.get(col) || []) {
        if (lv.cls[i] === C_SOLID && lv.box[i][1] <= top && top < lv.box[i][3]) {
          top = lv.box[i][3];
          changed = true;
        }
      }
    }
    return top;
  }

  function stackBottom(lv, col) {
    let best = MAX_Y;
    for (const i of lv.cols.get(col) || []) {
      if (lv.cls[i] === C_SOLID && lv.box[i][1] >= B && lv.box[i][1] < best) best = lv.box[i][1];
    }
    return best < MAX_Y ? best : 6 * B;
  }

  function setMode(s, mode, par) {
    s.mode = mode;
    if (mode === CUBE) s.ceil = 0;
    else {
      const h = par && par.length ? par[0] : 0;
      s.ceil = h > 0 ? h * B : CORRIDOR;
    }
    if (mode === SHIP) {
      let v = s.vy * s.grav;
      if (v > SHIP_MAX_UP) v = SHIP_MAX_UP;
      if (v < -SHIP_MAX_DOWN) v = -SHIP_MAX_DOWN;
      s.vy = v * s.grav;
    }
    s.ground = 0;
  }

  // -------------------------------------------------------------------------
  //  Ein Physik-Schritt
  // -------------------------------------------------------------------------
  function floorUnder(lv, x0, x1) {
    if (!lv.pits.size) return true;
    for (let c = fdiv(x0, B), c1 = fdiv(x1 - 1, B); c <= c1; c++) if (!lv.pits.has(c)) return true;
    return false;
  }

  function firstOrb(s, lv, x0, y0, x1, y1) {
    for (let c = fdiv(x0, B), c1 = fdiv(x1 - 1, B); c <= c1; c++) {
      for (const i of lv.cols.get(c) || []) {
        if (lv.cls[i] !== C_ORB || s.used.has(i)) continue;
        const b = lv.box[i];
        if (b[0] < x1 && b[2] > x0 && b[1] < y1 && b[3] > y0) return i;
      }
    }
    return -1;
  }

  function use(s, i) {
    const n = new Set(s.used);
    n.add(i);
    s.used = n;
  }

  /** Genau ein Schritt (1/240 s). held = gedrückt, pressed = in diesem Schritt neu gedrückt. */
  function step(s, lv, held, pressed) {
    if (s.dead || s.won) return;
    s.step += 1;
    if (pressed) s.buf = BUFFER;
    else if (!held) s.buf = 0;
    const mode = s.mode;
    const size = mode === WAVE ? WAVE_SIZE : B;
    const off = mode === WAVE ? WAVE_OFF : 0;

    if (s.buf > 0) {
      const oi = firstOrb(s, lv, s.x + off, s.y + off, s.x + off + size, s.y + off + size);
      if (oi >= 0) {
        use(s, oi);
        const which = lv.val[oi];
        if (which === 2) {
          s.grav = -s.grav;
          s.vy = -s.grav * ORB_V[mode][2];
        } else if (mode !== WAVE) {
          s.vy = s.grav * ORB_V[mode][which];
        }
        s.ground = 0;
        s.buf = 0;
      } else if (mode === UFO) {
        s.vy = s.grav * UFO_JUMP;
        s.ground = 0;
        s.buf = 0;
      } else if (mode === BALL && s.ground) {
        s.grav = -s.grav;
        s.vy = -s.grav * BALL_PUSH;
        s.ground = 0;
        s.buf = 0;
      }
    }
    const g = s.grav;
    if (mode === CUBE && s.ground && (held || pressed)) {
      s.vy = CUBE_JUMP * g;
      s.ground = 0;
    }

    let v = s.vy * g;
    if (mode === SHIP) {
      v += held ? SHIP_UP : -SHIP_DOWN;
      if (v > SHIP_MAX_UP) v = SHIP_MAX_UP;
      if (v < -SHIP_MAX_DOWN) v = -SHIP_MAX_DOWN;
    } else if (mode === WAVE) {
      v = held ? SPEEDS[s.speed] : -SPEEDS[s.speed];
    } else {
      v -= GRAVITY[mode];
      if (v < -FALL_MAX[mode]) v = -FALL_MAX[mode];
    }
    s.vy = v * g;
    const prev0 = s.y + off;
    const prev1 = prev0 + size;
    s.y += s.vy;
    s.x += SPEEDS[s.speed];
    s.ground = 0;
    const px0 = s.x + off;
    const px1 = px0 + size;

    let py0 = s.y + off;
    if (py0 < 0 && floorUnder(lv, px0, px1)) {
      if (prev0 >= -SNAP) {
        s.y = -off;
        if (s.vy < 0) s.vy = 0;
        if (g > 0) s.ground = 1;
      } else {
        s.dead = 1;
        return;
      }
    }
    if (s.ceil > 0 && s.y + off + size > s.ceil) {
      s.y = s.ceil - size - off;
      if (s.vy > 0) s.vy = 0;
      if (g < 0) s.ground = 1;
    }

    py0 = s.y + off;
    let py1 = py0 + size;
    let land = null, bump = null, hit = false;
    for (let c = fdiv(px0, B), c1 = fdiv(px1 - 1, B); c <= c1; c++) {
      for (const i of lv.cols.get(c) || []) {
        if (lv.cls[i] !== C_SOLID) continue;
        const b = lv.box[i];
        if (!(b[0] < px1 && b[2] > px0 && b[1] < py1 && b[3] > py0)) continue;
        if (mode === WAVE) {
          s.dead = 1;
          return;
        }
        hit = true;
        if (g > 0) {
          if (s.vy <= 0 && prev0 >= b[3] - SNAP) {
            if (land === null || b[3] > land) land = b[3];
          } else if (mode !== CUBE && s.vy > 0 && prev1 <= b[1] + SNAP) {
            if (bump === null || b[1] < bump) bump = b[1];
          }
        } else {
          if (s.vy >= 0 && prev1 <= b[1] + SNAP) {
            if (land === null || b[1] < land) land = b[1];
          } else if (mode !== CUBE && s.vy < 0 && prev0 >= b[3] - SNAP) {
            if (bump === null || b[3] > bump) bump = b[3];
          }
        }
      }
    }
    if (hit) {
      if (land !== null) {
        s.y = g > 0 ? land : land - B;
        s.vy = 0;
        s.ground = 1;
      }
      if (bump !== null) {
        s.y = g > 0 ? bump - B : bump;
        s.vy = 0;
      }
      const ix0 = s.x + INNER_OFF, ix1 = ix0 + INNER, iy0 = s.y + INNER_OFF, iy1 = iy0 + INNER;
      for (let c = fdiv(ix0, B), c1 = fdiv(ix1 - 1, B); c <= c1; c++) {
        for (const i of lv.cols.get(c) || []) {
          if (lv.cls[i] !== C_SOLID) continue;
          const b = lv.box[i];
          if (b[0] < ix1 && b[2] > ix0 && b[1] < iy1 && b[3] > iy0) {
            s.dead = 1;
            return;
          }
        }
      }
    }

    py0 = s.y + off;
    py1 = py0 + size;
    for (let c = fdiv(px0, B), c1 = fdiv(px1 - 1, B); c <= c1; c++) {
      for (const i of lv.cols.get(c) || []) {
        const cls = lv.cls[i];
        if (cls === C_SOLID) continue;
        const b = lv.box[i];
        if (!(b[0] < px1 && b[2] > px0 && b[1] < py1 && b[3] > py0)) continue;
        if (cls === C_HAZARD) {
          s.dead = 1;
          return;
        }
        if (cls === C_ORB || s.used.has(i)) continue;
        if (cls === C_PAD) {
          use(s, i);
          const which = lv.val[i];
          if (which === 2) {
            s.grav = -s.grav;
            s.vy = -s.grav * PAD_V[s.mode][2];
          } else if (s.mode !== WAVE) {
            s.vy = s.grav * PAD_V[s.mode][which];
          }
          s.ground = 0;
        } else if (cls === C_MODE) {
          use(s, i);
          setMode(s, lv.val[i] !== s.mode ? lv.val[i] : s.mode, lv.par[i]);
        } else if (cls === C_GRAV) {
          use(s, i);
          if (lv.val[i] !== s.grav) {
            s.grav = lv.val[i];
            s.vy = 0;
            s.ground = 0;
          }
        } else if (cls === C_SPEED) {
          use(s, i);
          s.speed = lv.val[i];
        } else if (cls === C_COIN) {
          use(s, i);
          s.coins |= lv.coinBit.get(i);
        }
      }
    }

    if (s.y < FALL_DEATH || s.y > MAX_Y) {
      s.dead = 1;
      return;
    }
    if (s.x >= lv.endX) s.won = 1;
    if (s.buf > 0) s.buf -= 1;
  }

  // -------------------------------------------------------------------------
  //  Hilfen
  // -------------------------------------------------------------------------
  function progress(s, lv) {
    if (s.won) return 100;
    return Math.max(0, Math.min(99, Math.floor(((s.x - START_X) * 100) / (lv.endX - START_X))));
  }

  function stateStr(s) {
    const used = Array.from(s.used).sort((a, b) => a - b).join(".");
    return [s.step, s.x, s.y, s.vy, s.mode, s.grav, s.speed, s.ground, s.ceil, s.buf, s.dead, s.won, s.coins, used].join(",");
  }

  const stateHash = (s) => hex8(fnv1a(stateStr(s)));

  /** Spielt Umschalt-Schritte ab. Gibt {state, trace: [[schritt, hash], ...]} zurück. */
  function runToggles(lv, toggles, maxSteps, traceEvery, start) {
    const s = start ? copyState(start) : newState(lv);
    const limit = maxSteps != null ? maxSteps : fdiv(lv.endX, SPEEDS[0]) + 4 * HZ;
    const tg = toggles.slice().sort((a, b) => a - b);
    let ti = 0, held = false, k = 0;
    const trace = [];
    while (k < limit && !s.dead && !s.won) {
      const was = held;
      while (ti < tg.length && tg[ti] <= k) {
        held = !held;
        ti++;
      }
      step(s, lv, held, held && !was);
      k++;
      if (traceEvery && k % traceEvery === 0) trace.push([k, stateHash(s)]);
    }
    trace.push([k, stateHash(s)]);
    return { state: s, trace };
  }

  const api = {
    B, HZ, HALF, SPEEDS, SPEED_LABELS, CUBE, SHIP, BALL, UFO, WAVE, MODE_NAMES, GRAVITY, FALL_MAX, CUBE_JUMP,
    UFO_JUMP, BALL_PUSH, CORRIDOR, BUFFER, SNAP, INNER, WAVE_SIZE, WAVE_OFF, MAX_LENGTH, MAX_ROW, START_X,
    MIN_LENGTH, MAX_COINS, KINDS, KIND_NAMES, KIND_ID, ROTATE_ALL, ROTATE_FLIP, PARAMS, HITBOX, MUSIC_STYLES,
    DEFAULT_BG, DEFAULT_GROUND, C_SOLID, C_HAZARD, C_PIT, C_PAD, C_ORB, C_MODE, C_GRAV, C_SPEED, C_COIN, C_TRIGGER,
    fdiv, normalizeObject, normalizeLevel, levelLength, contentHash, fnv1a, compile, copyState, newState, step,
    progress, stateStr, stateHash, runToggles, compareObjects,
  };
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  if (root && root.PG) root.PG.gdCore = api;
})(typeof window !== "undefined" ? window : globalThis);
