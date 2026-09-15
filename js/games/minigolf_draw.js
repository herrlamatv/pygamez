/*
 * minigolf_draw.js - Das Aussehen einer Minigolf-Bahn (Port von games/minigolf_draw.py)
 * ====================================================================================
 * Seit es eigene Bahnen gibt, muss derselbe Platz zweimal auf den Schirm: im
 * Spiel und im Editor. Doppelter Zeichencode wäre der sichere Weg, dass beide
 * mit der Zeit unterschiedlich aussehen - deshalb liegen die Farben und alle
 * Zeichenfunktionen hier, und beide Seiten rufen sie auf.
 *
 * Alle Funktionen bekommen eine View: sie kennt Nullpunkt und Maßstab und
 * rechnet Bahn-Einheiten in Bildschirm-Pixel um.
 *
 *     const view = new PG.minigolfDraw.View(ox, oy, scale);
 *     PG.minigolfDraw.drawCourse(ctx, hole, view, millA, moveT);
 *
 * hole muss vorher durch PG.minigolfGen.normalize gelaufen sein.
 */
(function () {
  "use strict";

  const { ui, draw } = PG;
  const { BORDER, CW, CH } = PG.minigolfGen;

  // ------------------------------------------------- Identitätsfarben (Platz)
  const COL = {
    GREEN: [46, 132, 74], GREEN_D: [38, 112, 62], FRINGE: [62, 152, 88],
    WALL: [122, 84, 52], WALL_HI: [156, 112, 72],
    SAND: [214, 190, 132], SAND_D: [190, 164, 108],
    WATER: [52, 122, 196], WATER_D: [36, 92, 158],
    SLOPE: [60, 150, 92],
    BUMPER: [222, 84, 108], BUMPER_HI: [246, 140, 160],
    MILL: [186, 190, 198], MILL_D: [128, 134, 146],
    BALL: [248, 248, 244], CUP: [16, 22, 18], FLAG: [226, 72, 72],
    AIM: [245, 245, 210],
    LOCK: [248, 208, 96], // Stärke-Sperre (Balken, Ring, Ziellinie)
    // ------------------------------------------ Farben der neuen Objekte
    TUNNEL: [156, 108, 210], TUNNEL_D: [108, 68, 156], // Rohr: violett
    ICE: [176, 222, 240], ICE_D: [132, 186, 214],      // Eis: helles Blauweiß
    BOOST: [250, 176, 60], BOOST_D: [206, 132, 26],    // Schub: orange Pfeile
    MAGNET: [236, 96, 96], MAGNET_R: [96, 148, 236],   // Magnet: rot zieht / blau stößt ab
    GATE: [120, 200, 150], GATE_D: [70, 150, 104],     // Einbahn-Tor: grünlich
    STICKY: [128, 104, 66], STICKY_D: [96, 76, 46],    // Klebefeld: dunkles Braun
    SPIN: [214, 200, 120], SPIN_D: [168, 152, 82],     // Drehscheibe: sandgelb
    JUMP: [240, 232, 120], JUMP_D: [196, 186, 62],     // Sprungrampe: hellgelb
  };

  // Reihenfolge, in der gezeichnet wird. Flächen zuerst, Aufbauten zuletzt -
  // so liegt nie eine Wand unter dem Sand.
  const DRAW_ORDER = ["slopes", "ice", "sticky", "sand", "boosters", "water", "magnets", "spinners",
    "tunnels", "walls", "gates", "movers", "bumpers", "jumps", "mills"];

  /** Nullpunkt und Maßstab: Bahn-Einheiten <-> Bildschirm-Pixel. */
  class View {
    constructor(ox, oy, scale) {
      this.ox = Number(ox);
      this.oy = Number(oy);
      this.scale = Number(scale);
    }
    project(x, y) {
      return [this.ox + x * this.scale, this.oy + y * this.scale];
    }
    unproject(sx, sy) {
      return [(sx - this.ox) / this.scale, (sy - this.oy) / this.scale];
    }
    rectPx(r) {
      const [px, py] = this.project(r[0], r[1]);
      return new PG.Rect(Math.trunc(px), Math.trunc(py), Math.max(1, Math.trunc(r[2] * this.scale)), Math.max(1, Math.trunc(r[3] * this.scale)));
    }
    /** Das Rechteck des ganzen Platzes in Pixeln. */
    board(cw = CW, ch = CH) {
      return new PG.Rect(Math.trunc(this.ox), Math.trunc(this.oy), Math.trunc(cw * this.scale), Math.trunc(ch * this.scale));
    }
  }

  // Innen um d kleiner (pygame rect.inflate(-d, -d)).
  const shrink = (rc, d) => new PG.Rect(rc.x + d / 2, rc.y + d / 2, rc.w - d, rc.h - d);

  // ---------------------------------------------------------------------------
  //  Untergrund
  // ---------------------------------------------------------------------------

  /** Grün, Mähstreifen und die vier Banden. */
  function drawGround(ctx, hole, view) {
    const board = view.board(hole.w, hole.h);
    draw.rect(ctx, COL.FRINGE, board.inflate(10, 10), 0, 10);
    draw.rect(ctx, COL.GREEN, board, 0, 6);
    const stripe = Math.max(6, Math.trunc(10 * view.scale));
    for (let i = 0; i < board.h; i += stripe * 2) {
      draw.rect(ctx, COL.GREEN_D, [board.x, board.y + i, board.w, Math.min(stripe, board.h - i)]);
    }
    const b = Math.max(2, Math.trunc(BORDER * view.scale));
    for (const r of [[board.x, board.y, board.w, b], [board.x, board.bottom - b, board.w, b],
      [board.x, board.y, b, board.h], [board.right - b, board.y, b, board.h]]) {
      draw.rect(ctx, COL.WALL, r);
      draw.rect(ctx, COL.WALL_HI, r, 1);
    }
    return board;
  }

  // ---------------------------------------------------------------------------
  //  Die 15 Hindernis-Typen
  // ---------------------------------------------------------------------------

  /** Rampe: Fläche mit Pfeilfeld in Beschleunigungsrichtung. */
  function drawSlope(ctx, r, ax, ay, view) {
    const rc = view.rectPx(r);
    draw.rect(ctx, COL.SLOPE, rc, 0, 5);
    const ang = Math.atan2(ay, ax);
    const step = Math.max(16, Math.trunc(14 * view.scale));
    const col = ui.mix(COL.SLOPE, [255, 255, 255], 0.22);
    const dx = Math.cos(ang) * 6, dy = Math.sin(ang) * 6;
    for (let yy = rc.y + (step >> 1); yy < rc.bottom - 2; yy += step) {
      for (let xx = rc.x + (step >> 1); xx < rc.right - 2; xx += step) {
        draw.line(ctx, col, [xx - dx, yy - dy], [xx + dx, yy + dy], 2);
        draw.circle(ctx, col, [xx + dx, yy + dy], 2);
      }
    }
  }

  function drawSand(ctx, r, view) {
    const rc = view.rectPx(r);
    draw.rect(ctx, COL.SAND, rc, 0, 6);
    draw.rect(ctx, COL.SAND_D, rc, 2, 6);
  }

  /** Wasser: Fläche mit drei laufenden Wellenlinien. */
  function drawWater(ctx, r, view) {
    const rc = view.rectPx(r);
    draw.rect(ctx, COL.WATER, rc, 0, 7);
    const t = ui.now();
    for (let i = 0; i < 3; i++) {
      const yy = rc.y + Math.trunc(rc.h * (0.25 + 0.25 * i)) + Math.trunc(Math.sin(t * 1.4 + i) * 3);
      if (rc.y < yy && yy < rc.bottom) draw.line(ctx, COL.WATER_D, [rc.x + 6, yy], [rc.right - 6, yy], 2);
    }
    draw.rect(ctx, COL.WATER_D, rc, 2, 7);
  }

  function drawWall(ctx, rc, mover = false) {
    draw.rect(ctx, COL.WALL, rc, 0, 3);
    draw.rect(ctx, COL.WALL_HI, rc, 2, 3);
    if (mover) draw.line(ctx, COL.WALL_HI, [rc.x + 4, rc.centery], [rc.right - 4, rc.centery], 1);
  }

  function drawBumper(ctx, b, view) {
    const [px, py] = view.project(b[0], b[1]);
    const rr = Math.max(3, Math.trunc(b[2] * view.scale));
    draw.circle(ctx, COL.BUMPER, [px, py], rr);
    draw.circle(ctx, COL.BUMPER_HI, [px, py], Math.max(2, rr - 3), 2);
  }

  /** Windmühle: rotierende Flügel um eine Nabe. */
  function drawMill(ctx, mill, view, millA) {
    const [x, y, length, arms, speed] = mill;
    const [px, py] = view.project(x, y);
    const base = millA * speed;
    const w = Math.max(3, Math.trunc(3.0 * view.scale));
    const n = Math.max(1, Math.trunc(arms));
    for (let i = 0; i < n; i++) {
      const a = base + i * ((2 * Math.PI) / n);
      const e = view.project(x + Math.cos(a) * length, y + Math.sin(a) * length);
      draw.line(ctx, COL.MILL, [px, py], e, w);
      draw.line(ctx, COL.MILL_D, [px, py], e, 1);
    }
    draw.circle(ctx, COL.MILL_D, [px, py], Math.max(3, Math.trunc(2.4 * view.scale)));
  }

  /** Rohr: zwei Mündungen, dazwischen eine gestrichelte Verbindung. */
  function drawTunnel(ctx, tun, view) {
    const [x1, y1, x2, y2, r] = tun;
    const [ax, ay] = view.project(x1, y1);
    const [bx, by] = view.project(x2, y2);
    const rr = Math.max(4, Math.trunc(r * view.scale));
    // Verbindung nur andeuten - sie ist kein Hindernis, sondern eine Erklärung.
    const seg = 9, gap = 7;
    const dx = bx - ax, dy = by - ay;
    const dist = Math.hypot(dx, dy);
    if (dist > 1) {
      const ux = dx / dist, uy = dy / dist;
      let pos = rr;
      while (pos < dist - rr) {
        const e = Math.min(pos + seg, dist - rr);
        draw.line(ctx, COL.TUNNEL_D, [ax + ux * pos, ay + uy * pos], [ax + ux * e, ay + uy * e], 2);
        pos = e + gap;
      }
    }
    const puls = ui.pulse(2.2, 0.55, 1.0);
    for (const [px, py] of [[ax, ay], [bx, by]]) {
      draw.circle(ctx, COL.TUNNEL_D, [px, py], rr);
      draw.circle(ctx, ui.mix(COL.TUNNEL, [255, 255, 255], 0.25 * puls), [px, py], Math.max(2, rr - 3));
      draw.circle(ctx, COL.TUNNEL_D, [px, py], Math.max(1, Math.trunc(rr / 3)));
    }
  }

  /** Eis: helle Fläche mit ein paar Rissen. */
  function drawIce(ctx, r, view) {
    const rc = view.rectPx(r);
    draw.rect(ctx, COL.ICE, rc, 0, 6);
    for (let i = 0; i < 3; i++) {
      const yy = rc.y + Math.floor((rc.h * (i + 1)) / 4);
      draw.line(ctx, COL.ICE_D, [rc.x + 5, yy], [rc.right - 5, yy - Math.floor(rc.h / 8)], 1);
    }
    draw.rect(ctx, COL.ICE_D, rc, 2, 6);
  }

  /** Schub-Feld: Doppelpfeile in Schubrichtung. */
  function drawBooster(ctx, bo, view) {
    const [x, y, w, h, dx, dy] = bo;
    const rc = view.rectPx([x, y, w, h]);
    draw.rect(ctx, COL.BOOST_D, rc, 0, 5);
    draw.rect(ctx, COL.BOOST, shrink(rc, 4), 0, 4);
    const ang = Math.atan2(dy, dx);
    const ux = Math.cos(ang), uy = Math.sin(ang);
    const step = Math.max(14, Math.trunc(12 * view.scale));
    for (let k = 0; k < 2; k++) {
      const off = (k - 0.5) * step * 0.7;
      for (let yy = rc.y + (step >> 1); yy < rc.bottom - 2; yy += step) {
        for (let xx = rc.x + (step >> 1); xx < rc.right - 2; xx += step) {
          const cx = xx - uy * off, cy = yy + ux * off;
          const tipx = cx + ux * 6, tipy = cy + uy * 6;
          draw.polygon(ctx, COL.BOOST_D, [[tipx, tipy],
            [cx - ux * 4 - uy * 4, cy - uy * 4 + ux * 4],
            [cx - ux * 4 + uy * 4, cy - uy * 4 - ux * 4]]);
        }
      }
    }
  }

  /** Magnet: Ringe um die Mitte, rot zieht an, blau stößt ab. */
  function drawMagnet(ctx, mag, view) {
    const [x, y, r, force] = mag;
    const [px, py] = view.project(x, y);
    const rr = Math.max(5, Math.trunc(r * view.scale));
    const col = force >= 0 ? COL.MAGNET : COL.MAGNET_R;
    const puls = ui.pulse(1.6, 0.35, 0.9);
    for (let i = 0; i < 3; i++) {
      const rad = Math.trunc(rr * (0.4 + 0.3 * i));
      draw.circle(ctx, [col[0], col[1], col[2], Math.trunc(150 * puls * (1 - i * 0.25))], [px, py], rad, 2);
    }
    draw.circle(ctx, col, [px, py], Math.max(3, Math.trunc(rr / 4)));
  }

  /** Einbahn-Tor: durchscheinende Fläche mit Pfeil in Durchlassrichtung. */
  function drawGate(ctx, gt, view) {
    const [x, y, w, h, dx, dy] = gt;
    const rc = view.rectPx([x, y, w, h]);
    draw.rect(ctx, [COL.GATE[0], COL.GATE[1], COL.GATE[2], 120], rc);
    draw.rect(ctx, COL.GATE_D, rc, 2, 3);
    const ang = Math.atan2(dy, dx);
    const ux = Math.cos(ang), uy = Math.sin(ang);
    const cx = rc.centerx, cy = rc.centery;
    const ln = Math.max(6, Math.trunc(Math.min(rc.w, rc.h) * 0.35));
    const tip = [cx + ux * ln, cy + uy * ln];
    draw.line(ctx, COL.GATE_D, [cx - ux * ln, cy - uy * ln], tip, 2);
    draw.polygon(ctx, COL.GATE_D, [tip, [tip[0] - ux * 6 - uy * 5, tip[1] - uy * 6 + ux * 5],
      [tip[0] - ux * 6 + uy * 5, tip[1] - uy * 6 - ux * 5]]);
  }

  /** Klebefeld: dunkle Fläche mit Blasen. */
  function drawSticky(ctx, r, view) {
    const rc = view.rectPx(r);
    draw.rect(ctx, COL.STICKY, rc, 0, 6);
    const step = Math.max(12, Math.trunc(11 * view.scale));
    for (let yy = rc.y + (step >> 1); yy < rc.bottom - 2; yy += step) {
      for (let xx = rc.x + (step >> 1); xx < rc.right - 2; xx += step) draw.circle(ctx, COL.STICKY_D, [xx, yy], 3);
    }
    draw.rect(ctx, COL.STICKY_D, rc, 2, 6);
  }

  /** Drehscheibe: Scheibe mit mitdrehenden Speichen. */
  function drawSpinner(ctx, sp, view, millA) {
    const [x, y, r, speed] = sp;
    const [px, py] = view.project(x, y);
    const rr = Math.max(5, Math.trunc(r * view.scale));
    draw.circle(ctx, COL.SPIN_D, [px, py], rr);
    draw.circle(ctx, COL.SPIN, [px, py], Math.max(2, rr - 3));
    const base = millA * speed;
    for (let i = 0; i < 4; i++) {
      const a = base + i * (Math.PI / 2);
      draw.line(ctx, COL.SPIN_D, [px, py], [px + Math.cos(a) * rr, py + Math.sin(a) * rr], 2);
    }
    draw.circle(ctx, COL.SPIN_D, [px, py], Math.max(2, Math.trunc(rr / 5)));
  }

  /** Sprungrampe: Schanze mit Stufen quer zur Sprungrichtung. */
  function drawJump(ctx, jp, view) {
    const [x, y, w, h, dx, dy] = jp;
    const rc = view.rectPx([x, y, w, h]);
    draw.rect(ctx, COL.JUMP_D, rc, 0, 4);
    draw.rect(ctx, COL.JUMP, shrink(rc, 4), 0, 3);
    const ang = Math.atan2(dy, dx);
    const ux = Math.cos(ang), uy = Math.sin(ang);
    const cx = rc.centerx, cy = rc.centery;
    const ln = Math.max(5, Math.trunc(Math.min(rc.w, rc.h) * 0.42));
    for (const k of [-1, 0, 1]) {
      const off = k * Math.max(4, ln >> 1);
      draw.line(ctx, COL.JUMP_D, [cx - uy * ln + ux * off, cy + ux * ln + uy * off],
        [cx + uy * ln + ux * off, cy - ux * ln + uy * off], 2);
    }
    const tip = [cx + ux * ln, cy + uy * ln];
    draw.polygon(ctx, COL.JUMP_D, [tip, [tip[0] - ux * 7 - uy * 5, tip[1] - uy * 7 + ux * 5],
      [tip[0] - ux * 7 + uy * 5, tip[1] - uy * 7 - ux * 5]]);
  }

  /** Loch mit Fahne. */
  function drawCup(ctx, cup, view, cupR) {
    const [px, py] = view.project(cup[0], cup[1]);
    const r = Math.max(3, Math.trunc(cupR * view.scale));
    draw.circle(ctx, [24, 60, 36], [px, py], r + 2);
    draw.circle(ctx, COL.CUP, [px, py], r);
    const top = py - Math.max(16, Math.trunc(18 * view.scale));
    draw.line(ctx, [238, 238, 232], [px, py], [px, top], 2);
    const wave = Math.sin(ui.ticks() / 260.0) * 2;
    draw.polygon(ctx, COL.FLAG, [[px, top], [px + 14, top + 5 + wave], [px, top + 11]]);
  }

  /** Abschlagpunkt - im Spiel liegt der Ball darauf, im Editor sieht man ihn. */
  function drawTee(ctx, tee, view) {
    const [px, py] = view.project(tee[0], tee[1]);
    const r = Math.max(3, Math.trunc(2.6 * view.scale));
    draw.circle(ctx, ui.mix(COL.GREEN, [255, 255, 255], 0.45), [px, py], r, 2);
    draw.circle(ctx, ui.mix(COL.GREEN, [255, 255, 255], 0.7), [px, py], Math.max(1, Math.trunc(r / 3)));
  }

  // ---------------------------------------------------------------------------
  //  Alles zusammen
  // ---------------------------------------------------------------------------

  /**
   * Aktuelles Rechteck eines Wanderblocks + seine Geschwindigkeit.
   * Pendelt zwischen (x, y) und (x + dx, y + dy). Liegt hier, weil Physik
   * (minigolf.js) und Zeichnen dieselbe Stelle brauchen.
   */
  function moverRect(m, moveT) {
    const [x, y, w, h, dx, dy, speed] = m;
    if (speed <= 0 || (dx === 0 && dy === 0)) return [[x, y, w, h], [0.0, 0.0]];
    const span = Math.hypot(dx, dy);
    const period = (2 * span) / speed;
    const ph = PG.mod(moveT, period) / period;
    const f = ph < 0.5 ? ph * 2 : (1 - ph) * 2;
    const sign = ph < 0.5 ? 1.0 : -1.0;
    return [[x + dx * f, y + dy * f, w, h], [(dx / span) * speed * sign, (dy / span) * speed * sign]];
  }

  /** Zeichnet alle Hindernisse in der richtigen Reihenfolge. */
  function drawObstacles(ctx, hole, view, millA = 0.0, moveT = 0.0) {
    for (const key of DRAW_ORDER) {
      const items = hole[key] || [];
      for (const it of items) {
        switch (key) {
          case "slopes": drawSlope(ctx, it.slice(0, 4), it[4], it[5], view); break;
          case "sand": drawSand(ctx, it, view); break;
          case "water": drawWater(ctx, it, view); break;
          case "walls": drawWall(ctx, view.rectPx(it)); break;
          case "movers": drawWall(ctx, view.rectPx(moverRect(it, moveT)[0]), true); break;
          case "bumpers": drawBumper(ctx, it, view); break;
          case "mills": drawMill(ctx, it, view, millA); break;
          case "tunnels": drawTunnel(ctx, it, view); break;
          case "ice": drawIce(ctx, it, view); break;
          case "boosters": drawBooster(ctx, it, view); break;
          case "magnets": drawMagnet(ctx, it, view); break;
          case "gates": drawGate(ctx, it, view); break;
          case "sticky": drawSticky(ctx, it, view); break;
          case "spinners": drawSpinner(ctx, it, view, millA); break;
          case "jumps": drawJump(ctx, it, view); break;
          default: break;
        }
      }
    }
  }

  /**
   * Kompletter Platz: Untergrund, Loch, alle Hindernisse.
   * Das Loch wird VOR den Aufbauten gezeichnet, damit eine Wand davor auch
   * davor liegt. tee zeigt zusätzlich den Abschlagpunkt (für den Editor).
   */
  function drawCourse(ctx, hole, view, millA = 0.0, moveT = 0.0, cupR = 3.0, tee = false) {
    drawGround(ctx, hole, view);
    drawCup(ctx, hole.cup, view, cupR);
    if (tee) drawTee(ctx, hole.tee, view);
    drawObstacles(ctx, hole, view, millA, moveT);
  }

  PG.minigolfDraw = {
    COL, DRAW_ORDER, View,
    drawGround, drawSlope, drawSand, drawWater, drawWall, drawBumper, drawMill, drawTunnel, drawIce,
    drawBooster, drawMagnet, drawGate, drawSticky, drawSpinner, drawJump, drawCup, drawTee,
    moverRect, drawObstacles, drawCourse,
  };
})();
