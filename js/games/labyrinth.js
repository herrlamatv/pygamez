/*
 * labyrinth.js - 3D-Labyrinth (Port von games/labyrinth.py)
 * ==========================================================
 * Ego-Raycaster im Wolfenstein-Stil (Einzelspieler).
 *
 * - 50 seed-generierte Level (maze_gen.js), Fortschritt wird gespeichert.
 * - Ego-Ansicht: Raycaster (DDA), Mouselook (Pointer-Lock) + WASD/Pfeile,
 *   [Q]/[E] als Tastatur-Drehung, [M] Minimap.
 * - Alternativ Top-Down-2D-Ansicht (im Setup umschaltbar, gespeichert):
 *   klassische Draufsicht ohne Maus-Capture.
 * - Orbs einsammeln, den (grün pulsierenden) Ausgang finden; Punkte =
 *   500 + Orbs*100 + Zeitbonus, sitzungskumulativ.
 *
 * Canvas2D: die Wandspalten sind (wie im Original) schmale Rechtecke - das
 * zeichnet Canvas als Vektor-Füllung deutlich schneller als Pixelarbeit.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const MG = PG.mazeGen;

  const LEVELS = 50;
  const MOVE_SPEED = 3.0; // Tiles/s
  const PLAYER_R = 0.25;
  const TURN_KEY = 90.0; // Grad/s über Q/E
  const DEG_PER_PX = 0.12;
  const FOG_DIST = 14.0;

  // 3D-Welt-Palette (bewusst fest - unabhängig vom UI-Theme)
  const COL_BG = [16, 14, 22];
  const COL_CEIL_TOP = [30, 34, 52];
  const COL_CEIL_BOT = [20, 22, 34];
  const COL_FLOOR = [34, 30, 40];
  const COL_WALL = [90, 110, 170];
  const COL_EXIT = [120, 255, 170];
  const COL_ORB = [120, 200, 255];

  const SETUP = "setup", PLAY = "play", FINISH = "finish";
  const STORE_KEY = "mem.maze"; // entspricht store.load_section("maze")

  const rgb = (r, g, b) => "rgb(" + (r | 0) + "," + (g | 0) + "," + (b | 0) + ")";

  class LabyrinthGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const mz = this.opts;
      this.view = mz.view === "top" ? "top" : "ego";
      const sens = Number(mz.sens);
      this.sens = isFinite(sens) ? Math.max(0.5, Math.min(2.0, sens)) : 1.0;
      const lv = parseInt(mz.last_level, 10);
      this.cursor = isFinite(lv) ? Math.max(1, Math.min(LEVELS, lv)) : 1;

      this.makeFonts();
      this._capture = false;
      this.showMap = false;
      this.loadSolved();
      this.level = 1;
      this.orbsTotal = 0;
      this.got = 0;
      this.elapsed = 0;
      this.keys = new Set();
      this._mapCache = null;
      this._topCache = null;
      this.buildSetupLayout();
      this.state = SETUP;
    }

    get captureMouse() {
      return this._capture;
    }

    makeFonts() {
      // Schriftgrößen aus der Auflösung ableiten (Theme-Schrift)
      const h = this.height;
      this._small = ui.font(Math.max(13, Math.floor(h / 30)));
      this._tiny = ui.font(Math.max(11, Math.floor(h / 36)));
      this._big = ui.font(Math.max(16, Math.floor(h / 21)), true);
      this._huge = ui.font(Math.max(26, Math.floor(h / 11)), true);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    loadSolved() {
      const data = PG.store.get(STORE_KEY, {});
      const lst = data && Array.isArray(data.solved) ? data.solved : [];
      const set = new Set();
      for (const v of lst) if (Number.isInteger(v) && v >= 1 && v <= LEVELS) set.add(v);
      this.solved = [...set].sort((a, b) => a - b);
    }

    markSolved(n) {
      if (!this.solved.includes(n)) {
        this.solved = this.solved.concat([n]).sort((a, b) => a - b);
        PG.store.set(STORE_KEY, { solved: this.solved });
      }
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const y0 = Math.floor(this.height * 0.22);
      const rx = cx + 24;
      this.viewRect = new PG.Rect(rx - 60, y0, 220, 40);
      this.sensMinus = new PG.Rect(rx - 60, y0 + 52, 44, 40);
      this.sensPlus = new PG.Rect(rx + 116, y0 + 52, 44, 40);
      this.sensBox = new PG.Rect(rx - 8, y0 + 52, 116, 40);
      const top = y0 + 110;
      const availH = this.height - top - 96;
      const cell = Math.max(18, Math.min(Math.floor((this.width - 80) / 10), Math.floor(availH / 5)));
      this.lvCell = cell;
      this.lvX = cx - cell * 5;
      this.lvY = top;
      this.lvFont = ui.font(Math.max(9, Math.floor((cell * 2) / 5)));
      this.startRect = new PG.Rect(cx - 95, top + 5 * cell + 12, 190, 44);
    }

    levelAt(pos) {
      const c = Math.floor((pos[0] - this.lvX) / this.lvCell);
      const r = Math.floor((pos[1] - this.lvY) / this.lvCell);
      if (c >= 0 && c < 10 && r >= 0 && r < 5) return r * 10 + c + 1;
      return null;
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "v" || k === "V") this.toggleView();
        else if (k === "Left" || k === "a" || k === "A") {
          this.cursor = PG.mod(this.cursor - 2, LEVELS) + 1;
          this.playSound("move");
        } else if (k === "Right" || k === "d" || k === "D") {
          this.cursor = PG.mod(this.cursor, LEVELS) + 1;
          this.playSound("move");
        } else if (k === "Up" || k === "w" || k === "W") {
          this.cursor = PG.mod(this.cursor - 11, LEVELS) + 1;
          this.playSound("move");
        } else if (k === "Down" || k === "s" || k === "S") {
          this.cursor = PG.mod(this.cursor + 9, LEVELS) + 1;
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.startLevel(this.cursor);
        }
      } else if (ev.kind === "mousedown") {
        const p = ev.pos;
        if (this.viewRect.collidepoint(p)) {
          this.toggleView();
          return;
        }
        if (this.sensMinus.collidepoint(p)) {
          this.sens = Math.round(Math.max(0.5, this.sens - 0.1) * 10) / 10;
          this.saveSetting("sens", this.sens);
          this.playSound("select");
          return;
        }
        if (this.sensPlus.collidepoint(p)) {
          this.sens = Math.round(Math.min(2.0, this.sens + 0.1) * 10) / 10;
          this.saveSetting("sens", this.sens);
          this.playSound("select");
          return;
        }
        const lv = this.levelAt(p);
        if (lv !== null) {
          this.startLevel(lv);
          return;
        }
        if (this.startRect.collidepoint(p)) this.startLevel(this.cursor);
      }
    }

    toggleView() {
      this.view = this.view === "ego" ? "top" : "ego";
      this.saveSetting("view", this.view);
      this.playSound("select");
    }

    // ===================================================== Level starten
    startLevel(level) {
      this.level = Math.max(1, Math.min(LEVELS, level));
      this.cursor = this.level;
      this.saveSetting("last_level", this.level);
      const p = MG.levelParams(this.level);
      const rng = new PG.Random(4400 + this.level);
      this.grid = MG.generate(p.cells, rng);
      this.n = this.grid.length;
      const [exitPos, dist] = MG.farExit(this.grid);
      this.exitPos = exitPos;
      this.orbs = MG.placeOrbs(this.grid, dist, this.exitPos, p.orbs, rng).map((o) => [o[0], o[1]]);
      this.orbsTotal = this.orbs.length;
      this.par = p.par;
      this.px = 1.5;
      this.py = 1.5;
      this.yaw = 0.0; // 0° = +x
      this.elapsed = 0.0;
      this.got = 0;
      this.keys = new Set();
      this.showMap = false;
      this._mapCache = null;
      this._topCache = null;
      this.state = PLAY;
      this._capture = this.view === "ego";
      this.playSound("level");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === FINISH) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.startLevel(Math.min(LEVELS, this.level + 1));
          else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this._capture = false;
            this.buildSetupLayout();
          }
        }
        return;
      }
      const ACTS = ["up", "down", "left", "right"];
      const CAP = { up: "Up", down: "Down", left: "Left", right: "Right" };
      if (ev.kind === "keydown") {
        const k = ev.key;
        for (const act of ACTS) if (this.isAction(k, act) || k === CAP[act]) this.keys.add(act);
        if (k === "q" || k === "Q") this.keys.add("turn_l");
        else if (k === "e" || k === "E") this.keys.add("turn_r");
        else if ((k === "m" || k === "M") && this.view === "ego") {
          this.showMap = !this.showMap;
          this.playSound("select");
        }
      } else if (ev.kind === "keyup") {
        const k = ev.key;
        for (const act of ACTS) if (this.isAction(k, act) || k === CAP[act]) this.keys.delete(act);
        if (k === "q" || k === "Q") this.keys.delete("turn_l");
        else if (k === "e" || k === "E") this.keys.delete("turn_r");
      } else if (ev.kind === "mouserel" && this.view === "ego") {
        this.yaw += ev.rel[0] * DEG_PER_PX * this.sens;
      }
    }

    // ===================================================== Update
    wall(x, y) {
      const ix = Math.trunc(x), iy = Math.trunc(y);
      if (ix >= 0 && ix < this.n && iy >= 0 && iy < this.n) return this.grid[iy][ix] === 1;
      return true;
    }

    blocked(x, y) {
      const r = PLAYER_R;
      return this.wall(x - r, y - r) || this.wall(x + r, y - r) || this.wall(x - r, y + r) || this.wall(x + r, y + r);
    }

    move(dx, dy, dt) {
      const L = Math.hypot(dx, dy);
      if (L < 1e-9) return;
      dx = (dx / L) * MOVE_SPEED * dt;
      dy = (dy / L) * MOVE_SPEED * dt;
      const nx = this.px + dx;
      if (!this.blocked(nx, this.py)) this.px = nx;
      const ny = this.py + dy;
      if (!this.blocked(this.px, ny)) this.py = ny;
    }

    update(dt) {
      if (this.state !== PLAY) return;
      this.elapsed += dt;

      if (this.keys.has("turn_l")) this.yaw -= TURN_KEY * dt;
      if (this.keys.has("turn_r")) this.yaw += TURN_KEY * dt;

      const fwd = (this.keys.has("up") ? 1 : 0) - (this.keys.has("down") ? 1 : 0);
      const side = (this.keys.has("right") ? 1 : 0) - (this.keys.has("left") ? 1 : 0);
      if (this.view === "ego") {
        // Strafe: rechts = +90° zur Blickrichtung
        const a = PG.radians(this.yaw);
        const dx = Math.cos(a) * fwd + Math.cos(a + Math.PI / 2) * side;
        const dy = Math.sin(a) * fwd + Math.sin(a + Math.PI / 2) * side;
        if (fwd || side) this.move(dx, dy, dt);
      } else if (fwd || side) {
        this.move(side, -fwd, dt);
      }

      // Orbs einsammeln
      for (let i = 0; i < this.orbs.length; i++) {
        const o = this.orbs[i];
        if (Math.hypot(this.px - (o[0] + 0.5), this.py - (o[1] + 0.5)) < 0.5) {
          this.orbs.splice(i, 1);
          this.got += 1;
          this.playSound("powerup");
          break;
        }
      }

      // Ausgang erreicht?
      if (Math.trunc(this.px) === this.exitPos[0] && Math.trunc(this.py) === this.exitPos[1]) this.finish();
    }

    finish() {
      this._capture = false;
      this.timeBonus = Math.max(0, Math.trunc((this.par - this.elapsed) * 8));
      this.orbBonus = this.got * 100;
      this.levelScore = 500 + this.orbBonus + this.timeBonus;
      this.score += this.levelScore;
      this.markSolved(this.level);
      this.state = FINISH;
      this.reportResult(true);
      this.playSound("win");
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      if (this.view === "ego") {
        this.drawEgo(ctx);
        if (this.showMap) this.drawMinimap(ctx);
      } else {
        this.drawTop(ctx);
      }
      this.drawHud(ctx);
      if (this.state === FINISH) this.drawFinish(ctx);
    }

    // ----- Raycaster ----------------------------------------------------
    drawEgo(ctx) {
      const w = this.width, h = this.height;
      const half = Math.floor(h / 2);
      // Decke zweifarbig + Boden
      draw.rect(ctx, COL_CEIL_TOP, [0, 0, w, Math.floor(half / 2)]);
      draw.rect(ctx, COL_CEIL_BOT, [0, Math.floor(half / 2), w, half - Math.floor(half / 2)]);
      draw.rect(ctx, COL_FLOOR, [0, half, w, h - half]);

      const a = PG.radians(this.yaw);
      const dirx = Math.cos(a), diry = Math.sin(a);
      const planex = -diry * 0.66, planey = dirx * 0.66;
      const stepPx = 2;
      const ncols = Math.floor(w / stepPx) + 1;
      if (!this._zbuf || this._zbuf.length !== ncols) this._zbuf = new Float64Array(ncols);
      const zbuf = this._zbuf;
      zbuf.fill(1e9);
      const tick = ui.now();
      const exitPulse = 0.75 + 0.25 * Math.sin(tick * 4);
      const [ex, ey] = this.exitPos;
      const n = this.n, grid = this.grid;
      const px = this.px, py = this.py;

      for (let col = 0; col < ncols; col++) {
        const x = col * stepPx;
        const camx = (2.0 * x) / Math.max(1, w) - 1.0;
        const rdx = dirx + planex * camx;
        const rdy = diry + planey * camx;
        let mapx = Math.trunc(px), mapy = Math.trunc(py);
        const ddx = rdx ? Math.abs(1.0 / rdx) : 1e30;
        const ddy = rdy ? Math.abs(1.0 / rdy) : 1e30;
        let stepx, stepy, sdx, sdy;
        if (rdx < 0) {
          stepx = -1;
          sdx = (px - mapx) * ddx;
        } else {
          stepx = 1;
          sdx = (mapx + 1.0 - px) * ddx;
        }
        if (rdy < 0) {
          stepy = -1;
          sdy = (py - mapy) * ddy;
        } else {
          stepy = 1;
          sdy = (mapy + 1.0 - py) * ddy;
        }
        let side = 0;
        let hit = false;
        for (let s = 0; s < 4 * n; s++) {
          if (sdx < sdy) {
            sdx += ddx;
            mapx += stepx;
            side = 0;
          } else {
            sdy += ddy;
            mapy += stepy;
            side = 1;
          }
          if (mapx < 0 || mapy < 0 || mapx >= n || mapy >= n) {
            hit = true;
            break;
          }
          if (grid[mapy][mapx] === 1) {
            hit = true;
            break;
          }
        }
        if (!hit) continue;
        let perp = side === 0 ? sdx - ddx : sdy - ddy;
        perp = Math.max(0.02, perp);
        zbuf[col] = perp;
        const lineH = Math.trunc(h / perp);
        const y0 = Math.max(0, half - Math.floor(lineH / 2));
        const y1 = Math.min(h, half + Math.floor(lineH / 2));
        let cr = COL_WALL[0], cg = COL_WALL[1], cb = COL_WALL[2];
        // Wand direkt am Ausgangs-Tile grün einfärben
        if (!(mapx === ex && mapy === ey) && Math.abs(mapx - ex) + Math.abs(mapy - ey) === 1) {
          cr = Math.trunc(COL_EXIT[0] * exitPulse);
          cg = Math.trunc(COL_EXIT[1] * exitPulse);
          cb = Math.trunc(COL_EXIT[2] * exitPulse);
        }
        const shade = side === 1 ? 0.8 : 1.0;
        const fog = Math.min(1.0, perp / FOG_DIST);
        ctx.fillStyle = rgb(
          cr * shade * (1 - fog) + COL_BG[0] * fog,
          cg * shade * (1 - fog) + COL_BG[1] * fog,
          cb * shade * (1 - fog) + COL_BG[2] * fog
        );
        // +0.5 überlappt die Nachbarspalte minimal -> keine Kanten-Säume
        ctx.fillRect(x, y0, stepPx + 0.5, y1 - y0);
      }

      // Sprites: Orbs + Exit-Glow (nach Distanz sortiert, fern zuerst)
      const sprites = this.orbs.map((o) => [o[0] + 0.5, o[1] + 0.5, COL_ORB, 0.3]);
      sprites.push([ex + 0.5, ey + 0.5, COL_EXIT, 0.45]);
      const inv = 1.0 / (planex * diry - dirx * planey || 1e-9);
      const order = [];
      for (const [sx, sy, colr, size] of sprites) {
        const rx = sx - px, ry = sy - py;
        const tx = inv * (diry * rx - dirx * ry);
        const ty = inv * (-planey * rx + planex * ry);
        if (ty <= 0.1) continue;
        order.push([ty, tx, colr, size]);
      }
      order.sort((p, q) => q[0] - p[0]);
      for (const [ty, tx, colr, size] of order) {
        const sxPx = Math.trunc((w / 2) * (1 + tx / ty));
        const rPx = Math.max(2, Math.trunc((h * size) / ty));
        // Occlusion: 3 zbuf-Samples, sichtbar wenn >= 2 frei
        let free = 0;
        for (const off of [-Math.floor(rPx / 2), 0, Math.floor(rPx / 2)]) {
          const c = Math.floor((sxPx + off) / stepPx);
          if (c >= 0 && c < ncols && zbuf[c] > ty) free += 1;
        }
        if (free < 2) continue;
        const fog = Math.min(1.0, ty / FOG_DIST);
        const cc = [
          Math.trunc(colr[0] * (1 - fog) + COL_BG[0] * fog),
          Math.trunc(colr[1] * (1 - fog) + COL_BG[1] * fog),
          Math.trunc(colr[2] * (1 - fog) + COL_BG[2] * fog),
        ];
        const cy = half + Math.trunc((h * 0.12) / ty);
        const pulse = 1.0 + 0.12 * Math.sin(tick * 5 + tx);
        const rr = Math.max(2, Math.floor(Math.trunc(rPx * pulse) / 2));
        draw.circle(ctx, cc, [sxPx, cy], rr);
        draw.circle(ctx, [Math.min(255, cc[0] + 60), Math.min(255, cc[1] + 60), Math.min(255, cc[2] + 60)], [sxPx, cy], rr, Math.max(1, Math.floor(rr / 4)));
      }
    }

    /** Statischer Teil der Minimap (Wände) - einmal je Level gecacht. */
    minimapBase(ts, size) {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = ts + "@" + ps;
      if (this._mapCache && this._mapCache.key === key) return this._mapCache.canvas;
      const c = ui.makeCanvas(size * ps, size * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      g.fillStyle = ui.col([10, 10, 16, 185]);
      g.fillRect(0, 0, size, size);
      g.fillStyle = ui.col([90, 100, 140, 220]);
      for (let y = 0; y < this.n; y++) {
        for (let x = 0; x < this.n; x++) {
          if (this.grid[y][x] === 1) g.fillRect(x * ts, y * ts, ts, ts);
        }
      }
      this._mapCache = { key, canvas: c };
      return c;
    }

    drawMinimap(ctx) {
      const ts = Math.max(2, Math.min(6, Math.floor(180 / this.n)));
      const size = ts * this.n;
      const ox = this.width - size - 12, oy = 78;
      ctx.drawImage(this.minimapBase(ts, size), ox, oy, size, size);
      const [ex, ey] = this.exitPos;
      draw.rect(ctx, COL_EXIT, [ox + ex * ts, oy + ey * ts, ts, ts]);
      for (const o of this.orbs) draw.rect(ctx, COL_ORB, [ox + o[0] * ts, oy + o[1] * ts, ts, ts]);
      const px = ox + Math.trunc(this.px * ts), py = oy + Math.trunc(this.py * ts);
      draw.circle(ctx, [255, 220, 120], [px, py], Math.max(2, Math.floor(ts / 2)));
      const a = PG.radians(this.yaw);
      draw.line(ctx, [255, 220, 120], [px, py], [px + Math.trunc(Math.cos(a) * ts * 2), py + Math.trunc(Math.sin(a) * ts * 2)], 1);
    }

    // ----- Top-Down ------------------------------------------------------
    /** Wandraster der Draufsicht (statisch) - einmal je Level/Größe gecacht. */
    topBase(ts) {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = ts + "@" + ps;
      if (this._topCache && this._topCache.key === key) return this._topCache.canvas;
      const size = ts * this.n;
      const c = ui.makeCanvas(size * ps, size * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      for (let y = 0; y < this.n; y++) {
        for (let x = 0; x < this.n; x++) {
          if (this.grid[y][x] === 1) {
            draw.rect(g, [52, 62, 96], [x * ts, y * ts, ts, ts]);
            draw.rect(g, [34, 40, 64], [x * ts, y * ts, ts, ts], 1);
          }
        }
      }
      this._topCache = { key, canvas: c };
      return c;
    }

    drawTop(ctx) {
      draw.rect(ctx, COL_BG, [0, 0, this.width, this.height]);
      const avail = Math.min(this.width - 20, this.height - 56);
      let ts = Math.floor(avail / this.n);
      const tick = ui.now();
      let ox, oy;
      if (ts >= 5) {
        ox = Math.floor((this.width - ts * this.n) / 2);
        oy = 44 + Math.floor((this.height - 44 - ts * this.n) / 2);
        ctx.drawImage(this.topBase(ts), ox, oy, ts * this.n, ts * this.n);
      } else {
        ts = 22;
        ox = Math.floor(this.width / 2) - Math.trunc(this.px * ts);
        oy = Math.floor(this.height / 2) - Math.trunc(this.py * ts);
        const x0 = Math.max(0, Math.trunc(this.px) - Math.floor(this.width / (2 * ts)) - 2);
        const x1 = Math.min(this.n, Math.trunc(this.px) + Math.floor(this.width / (2 * ts)) + 3);
        const y0 = Math.max(0, Math.trunc(this.py) - Math.floor(this.height / (2 * ts)) - 2);
        const y1 = Math.min(this.n, Math.trunc(this.py) + Math.floor(this.height / (2 * ts)) + 3);
        for (let y = y0; y < y1; y++) {
          for (let x = x0; x < x1; x++) {
            if (this.grid[y][x] === 1) {
              draw.rect(ctx, [52, 62, 96], [ox + x * ts, oy + y * ts, ts, ts]);
              draw.rect(ctx, [34, 40, 64], [ox + x * ts, oy + y * ts, ts, ts], 1);
            }
          }
        }
      }
      const [ex, ey] = this.exitPos;
      const pulse = 0.7 + 0.3 * Math.sin(tick * 4);
      draw.rect(ctx, [COL_EXIT[0] * pulse, COL_EXIT[1] * pulse, COL_EXIT[2] * pulse], [ox + ex * ts, oy + ey * ts, ts, ts]);
      for (const o of this.orbs) {
        const cx = ox + Math.trunc((o[0] + 0.5) * ts);
        const cy = oy + Math.trunc((o[1] + 0.5) * ts);
        const rr = Math.max(2, Math.trunc(ts * 0.28 * (1 + 0.15 * Math.sin(tick * 5))));
        draw.circle(ctx, COL_ORB, [cx, cy], rr);
      }
      // Spieler als Punkt
      const px = ox + this.px * ts;
      const py = oy + this.py * ts;
      draw.circle(ctx, [255, 220, 120], [px, py], Math.max(3, Math.trunc(ts * 0.3)));
    }

    // ----- HUD / Overlays -------------------------------------------------
    drawHud(ctx) {
      ui.text(ctx, t("common.points", { score: this.score }), 14, 8, this._big, this.accent);
      const lines = [
        t("maze.level", { n: this.level }),
        t("maze.orbs", { n: this.got, m: this.orbsTotal }),
        t("maze.time", { s: Math.trunc(this.elapsed) }),
      ];
      let y = 10;
      for (const line of lines) {
        ui.text(ctx, line, this.width - 14, y, this._small, ui.TEXT_DIM, "topright");
        y += 20;
      }
      if (this.view === "ego" && this.state === PLAY && !this.showMap) {
        ui.text(ctx, t("maze.map_hint"), this.width / 2, this.height - 6, this._tiny, ui.TEXT_FAINT, "midbottom");
      }
    }

    drawFinish(ctx) {
      ctx.fillStyle = ui.col([8, 14, 12], 190 / 255);
      ctx.fillRect(0, 0, this.width, this.height);
      const cx = Math.floor(this.width / 2), cy = Math.floor(this.height / 2);
      const head = t("maze.level_done", { n: this.level });
      const lines = [
        t("maze.base", { n: 500 }),
        t("maze.orb_bonus", { n: this.orbBonus }),
        t("maze.time_bonus", { n: this.timeBonus }),
        t("common.points", { score: this.score }),
      ];
      const hint = t("maze.next");

      // Panel hinter dem Ergebnis (dynamische ui-Palette)
      const top = cy - 70 - Math.floor(this._huge.height / 2) - 22;
      const bottom = cy - 20 + 30 * lines.length + 12 + Math.floor(this._small.height / 2) + 22;
      const pw = Math.min(
        this.width - 40,
        Math.max(400, this._huge.width(head) + 80, this._small.width(hint) + 60, Math.max(...lines.map((l) => this.font.width(l))) + 60)
      );
      const panel = [cx - Math.floor(pw / 2), top, pw, bottom - top];
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, ui.BORDER_LIGHT, panel, 1, 14);

      ui.text(ctx, head, cx, cy - 70, this._huge, COL_EXIT, "center");
      let y = cy - 20;
      for (const line of lines) {
        ui.text(ctx, line, cx, y, this.font, ui.TEXT, "center");
        y += 30;
      }
      ui.text(ctx, hint, cx, y + 12, this._small, ui.TEXT_DIM, "center");
    }

    // ----- Setup ----------------------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("maze.title"), cx, Math.floor(this.height * 0.09), this._huge, this.accent, "center");
      ui.text(ctx, t("maze.subtitle"), cx, Math.floor(this.height * 0.15), this._small, ui.TEXT_DIM, "center");

      const vr = this.viewRect;
      ui.text(ctx, t("maze.view"), vr.x - 16, vr.centery, this._small, ui.TEXT_DIM, "midright");
      draw.rect(ctx, ui.BTN_SEL, vr, 0, 8);
      draw.rect(ctx, ui.BORDER, vr, 1, 8);
      ui.text(ctx, t("maze.view." + this.view) + "  [V]", vr.centerx, vr.centery, this._small, ui.TEXT, "center");

      ui.text(ctx, t("maze.sens"), this.sensMinus.x - 16, this.sensMinus.centery, this._small, ui.TEXT_DIM, "midright");
      for (const [r, sym] of [[this.sensMinus, "-"], [this.sensPlus, "+"]]) {
        draw.rect(ctx, ui.BTN, r, 0, 8);
        draw.rect(ctx, ui.BORDER, r, 1, 8);
        ui.text(ctx, sym, r.centerx, r.centery, this._big, ui.TEXT, "center");
      }
      ui.text(ctx, this.sens.toFixed(1) + "x", this.sensBox.centerx, this.sensBox.centery, this._big, this.accent, "center");

      const doneFill = ui.mix(ui.BTN, this.accent, 0.25);
      const doneText = ui.mix(this.accent, ui.TEXT, 0.35);
      for (let n = 1; n <= LEVELS; n++) {
        const i = n - 1;
        const x = this.lvX + (i % 10) * this.lvCell;
        const y = this.lvY + Math.floor(i / 10) * this.lvCell;
        const cell = new PG.Rect(x + 1, y + 1, this.lvCell - 2, this.lvCell - 2);
        const done = this.solved.includes(n);
        draw.rect(ctx, done ? doneFill : ui.BTN, cell, 0, 4);
        if (n === this.cursor) draw.rect(ctx, this.accent, cell, 2, 4);
        ui.text(ctx, String(n), cell.centerx, cell.centery, this.lvFont, done ? doneText : ui.TEXT_DIM, "center");
      }
      ui.text(ctx, t("maze.progress", { n: this.solved.length, m: LEVELS }), cx, this.lvY + 5 * this.lvCell + 34, this._small, ui.TEXT_DIM, "center");

      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
    }
  }

  PG.register(LabyrinthGame, {
    id: "LabyrinthGame",
    key: "maze",
    name: { default: "3D Maze", de: "3D-Labyrinth", fr: "Labyrinthe 3D", es: "Laberinto 3D", pt: "Labirinto 3D" },
    settingsKey: "maze",
    defaults: { view: "ego", sens: 1.0, last_level: 1 },
  });
})();
