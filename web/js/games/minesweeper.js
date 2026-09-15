/*
 * minesweeper.js - Minesweeper, der Klassiker (Port von games/minesweeper.py)
 * ============================================================================
 * - Drei Schwierigkeitsgrade (im Setup wählbar, wird gespeichert):
 *     Einsteiger 9x9/10 Minen, Fortgeschritten 16x16/40, Experte 30x16/99
 * - Der ERSTE Klick ist immer sicher: die Minen werden erst nach dem ersten
 *   Aufdecken verteilt und sparen das 3x3-Feld um den Klick aus.
 * - Steuerung: Linksklick = aufdecken, RECHTSKLICK = Flagge (optional mit
 *   Fragezeichen-Zyklus: Flagge -> ? -> leer), F = Flagge unter dem Mauszeiger,
 *   R = neues Spiel, S = zurück zum Setup.
 * - CHORDING: Klick auf eine aufgedeckte Zahl, um die alle Flaggen gesetzt
 *   sind, deckt die restlichen Nachbarn auf einmal auf.
 * - Klassisches HUD: Minenzähler links, klickbarer SMILEY in der Mitte, Timer rechts.
 * - Bestzeit je Schwierigkeitsgrad wird in den Einstellungen gespeichert.
 *   Highscore-Punkte = Grundwert der Stufe minus Sekunden.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const Rect = PG.Rect;
  const fl = Math.floor;

  // Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme).
  const COL_TILE = [52, 60, 82]; // verdeckte Zelle
  const COL_TILE_HOVER = [64, 74, 100];
  const COL_BEVEL_L = [88, 99, 128]; // helle 3D-Kante (oben/links)
  const COL_BEVEL_D = [30, 34, 48]; // dunkle 3D-Kante (unten/rechts)
  const COL_OPEN = [33, 38, 54]; // aufgedeckte Zelle
  const COL_OPEN_LINE = [24, 28, 40];
  const COL_BOOM = [168, 66, 66]; // explodierte Mine
  const COL_LED = [240, 90, 90]; // LED-Ziffern (klassisch rot)
  const COL_FLAG = [235, 90, 90];

  // Klassische Zahlenfarben (für dunklen Hintergrund aufgehellt)
  const NUMBER_COLORS = {
    1: [110, 160, 255], 2: [110, 220, 130], 3: [250, 110, 110], 4: [180, 140, 255],
    5: [230, 160, 90], 6: [110, 220, 220], 7: [235, 235, 235], 8: [170, 170, 185],
  };

  // (Schlüssel, Spalten, Zeilen, Minen, Punkte-Grundwert)
  const PRESETS = [
    ["beginner", 9, 9, 10, 150],
    ["advanced", 16, 16, 40, 500],
    ["expert", 30, 16, 99, 1200],
  ];
  const PRESET_KEYS = PRESETS.map((p) => p[0]);

  const HUD_H = 52; // Kopfzeile über dem Spielfeld
  const SURPRISE_T = 0.22; // so lange staunt der Smiley nach einem Klick

  const SETUP = "setup", PLAY = "play";

  /** Zahl dreistellig wie f"{n:03d}" (auch negativ). */
  function pad3(n) {
    return n < 0 ? "-" + String(-n).padStart(2, "0") : String(n).padStart(3, "0");
  }

  class MinesweeperGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const ms = this.opts;
      const idx = PRESET_KEYS.indexOf(ms.preset);
      this.presetIndex = idx >= 0 ? idx : 0;
      this.qmarks = !!ms.qmarks;
      this.best = Object.assign({}, ms.best && typeof ms.best === "object" ? ms.best : {}); // preset -> Sekunden

      this.makeFonts();
      this.animT = 0;
      this.lastDrawT = null;

      this.buildSetupLayout();
      this.state = SETUP;
      this.startBoard();
    }

    get preset() {
      return PRESETS[this.presetIndex];
    }

    // ----- Layout / Theme --------------------------------------------------
    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, fl(h / 26)));
      this.bigFont = ui.font(Math.max(30, fl(h / 11)), true);
      this.small = ui.font(Math.max(13, fl(h / 32)));
      this.tiny = ui.font(Math.max(11, fl(h / 40)));
      // LED-Anzeige: Mono-Schrift, damit die Ziffern nicht "wackeln".
      this.led = ui.font(Math.max(18, Math.min(26, fl(h / 24))), true, true);
    }

    /** Baut ein frisches Brett für den aktuellen Schwierigkeitsgrad. */
    startBoard() {
      [, this.cols, this.rows, this.nMines, this.basePoints] = this.preset;

      this.mines = new Set(); // Zell-Indizes (y*cols+x)
      this.numbers = new Array(this.cols * this.rows).fill(0); // Anzahl Nachbarminen
      this.revealed = new Set();
      this.flags = new Map(); // idx -> 1 = Flagge, 2 = Fragezeichen
      this.firstClick = true;
      this.exploded = null; // die Zelle, die hochgegangen ist
      this.won = false;
      this.gameOver = false;
      this.score = 0;
      this.elapsed = 0;
      this.running = false; // Timer läuft (ab dem ersten Klick)
      this.hover = null;
      this.surprise = 0;
      this.particles = [];

      this.layoutBoard();
    }

    /** Zellgröße/Lage: unter dem HUD zentriert, ganzzahlige Pixel. */
    layoutBoard() {
      const aw = this.width - 24;
      const ah = this.height - HUD_H - 24;
      this.cell = Math.max(10, Math.min(fl(aw / this.cols), fl(ah / this.rows)));
      const bw = this.cell * this.cols, bh = this.cell * this.rows;
      this.bx = fl((this.width - bw) / 2);
      this.by = HUD_H + 12 + fl((ah - bh) / 2);

      this.numFont = ui.font(Math.max(11, Math.trunc(this.cell * 0.55)), true);

      // Smiley-Knopf im HUD
      this.faceRect = new Rect(fl(this.width / 2) - 20, 8, 40, 40);
    }

    // ----- Hilfen ----------------------------------------------------------
    neighbors(idx) {
      const x = idx % this.cols, y = fl(idx / this.cols);
      const out = [];
      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          if (dx === 0 && dy === 0) continue;
          const nx = x + dx, ny = y + dy;
          if (nx >= 0 && nx < this.cols && ny >= 0 && ny < this.rows) out.push(ny * this.cols + nx);
        }
      }
      return out;
    }

    cellAt(pos) {
      if (!pos) return null;
      const x = fl((pos[0] - this.bx) / this.cell);
      const y = fl((pos[1] - this.by) / this.cell);
      if (x >= 0 && x < this.cols && y >= 0 && y < this.rows) return y * this.cols + x;
      return null;
    }

    /** Verteilt die Minen; das 3x3-Feld um 'safe' bleibt frei. */
    placeMines(safe) {
      const tabu = new Set([safe, ...this.neighbors(safe)]);
      const alle = [];
      for (let i = 0; i < this.cols * this.rows; i++) if (!tabu.has(i)) alle.push(i);
      this.mines = new Set(PG.rand.sample(alle, Math.min(this.nMines, alle.length)));
      this.numbers = new Array(this.cols * this.rows).fill(0);
      for (let i = 0; i < this.cols * this.rows; i++) {
        if (this.mines.has(i)) continue;
        this.numbers[i] = this.neighbors(i).filter((nb) => this.mines.has(nb)).length;
      }
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = fl(this.width / 2);
      const bw = Math.min(440, this.width - 60);
      const bh = 46, gap = 10;
      const y0 = Math.max(124, Math.trunc(this.height * 0.24));
      this.presetRects = PRESETS.map((_, i) => new Rect(cx - fl(bw / 2), y0 + i * (bh + gap), bw, bh));
      const y1 = y0 + PRESETS.length * (bh + gap) + 4;
      this.qmarkRect = new Rect(cx - fl(bw / 2), y1, bw, 40);
      this.startRect = new Rect(cx - 95, y1 + 52, 190, 50);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    selectPreset(i) {
      this.presetIndex = i;
      this.saveSetting("preset", PRESET_KEYS[i]);
      this.playSound("click");
    }

    toggleQmarks() {
      this.qmarks = !this.qmarks;
      this.saveSetting("qmarks", this.qmarks);
      this.playSound("select");
    }

    startPlay() {
      this.startBoard();
      this.state = PLAY;
      this.playSound("click");
    }

    /** Zurück ins Setup (Web: gameOver zurücksetzen, sonst liegt das Highscore-Banner über dem Setup). */
    toSetup() {
      this.state = SETUP;
      this.gameOver = false;
      this.particles = [];
      this.playSound("click");
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3"].includes(k)) this.selectPreset(Number(k) - 1);
        else if (["Up", "w", "W"].includes(k)) this.selectPreset(PG.mod(this.presetIndex - 1, PRESETS.length));
        else if (["Down", "s", "S"].includes(k)) this.selectPreset(PG.mod(this.presetIndex + 1, PRESETS.length));
        else if (["q", "Q", "f", "F"].includes(k)) this.toggleQmarks();
        else if (k === "Return" || k === "space") this.startPlay();
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        const p = ev.pos;
        for (let i = 0; i < this.presetRects.length; i++) {
          if (this.presetRects[i].collidepoint(p)) {
            this.selectPreset(i);
            return;
          }
        }
        if (this.qmarkRect.collidepoint(p)) this.toggleQmarks();
        else if (this.startRect.collidepoint(p)) this.startPlay();
      }
    }

    // ===================================================== Eingabe (Spiel)
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetupEvent(ev);
        return;
      }

      if (ev.kind === "mousemove") {
        this.hover = this.cellAt(ev.pos);
        return;
      }

      if (ev.kind === "mousedown") {
        // Smiley = neues Spiel (nur Linksklick, wie beim Original)
        if (ev.button === 1 && this.faceRect.collidepoint(ev.pos)) {
          this.startBoard();
          this.playSound("click");
          return;
        }
        if (this.gameOver) return;
        const cell = this.cellAt(ev.pos);
        if (cell === null) return;
        if (ev.button === 3) this.toggleFlag(cell);
        else this.clickCell(cell);
        return;
      }

      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (["Return", "space", "r", "R"].includes(ev.key)) this.startBoard();
        else if (ev.key === "s" || ev.key === "S") this.toSetup();
        return;
      }

      if ((ev.key === "f" || ev.key === "F") && this.hover !== null) {
        this.toggleFlag(this.hover);
      } else if (ev.key === "r" || ev.key === "R") {
        this.startBoard();
        this.playSound("click");
      } else if (ev.key === "s" || ev.key === "S") {
        this.toSetup();
      }
    }

    // ----- Spielzüge --------------------------------------------------------
    toggleFlag(cell) {
      if (this.revealed.has(cell) || this.gameOver) return;
      const zustand = this.flags.get(cell) || 0;
      if (zustand === 0) this.flags.set(cell, 1);
      else if (zustand === 1 && this.qmarks) this.flags.set(cell, 2);
      else this.flags.delete(cell);
      this.playSound("move");
    }

    clickCell(cell) {
      this.surprise = SURPRISE_T;
      if (this.revealed.has(cell)) {
        this.chord(cell);
        return;
      }
      if (this.flags.get(cell) === 1) return; // geflaggte Zellen sind geschützt
      this.reveal(cell);
    }

    /** Zahl anklicken: Nachbarn aufdecken, wenn genug Flaggen gesetzt sind. */
    chord(cell) {
      const zahl = this.numbers[cell] || 0;
      if (zahl <= 0) return;
      const flaggen = this.neighbors(cell).filter((nb) => this.flags.get(nb) === 1).length;
      if (flaggen !== zahl) return;
      for (const nb of this.neighbors(cell)) {
        if (!this.revealed.has(nb) && this.flags.get(nb) !== 1) {
          this.reveal(nb);
          if (this.gameOver) return;
        }
      }
    }

    reveal(cell) {
      if (this.firstClick) {
        this.placeMines(cell);
        this.firstClick = false;
        this.running = true;
      }

      if (this.mines.has(cell)) {
        this.lose(cell);
        return;
      }

      // Flutfüllung: 0er-Zellen decken ihre Nachbarschaft mit auf
      const stapel = [cell];
      let neu = 0;
      while (stapel.length) {
        const c = stapel.pop();
        if (this.revealed.has(c) || this.mines.has(c)) continue;
        this.revealed.add(c);
        this.flags.delete(c);
        neu++;
        if (this.numbers[c] === 0) {
          for (const nb of this.neighbors(c)) if (!this.revealed.has(nb)) stapel.push(nb);
        }
      }
      if (neu) this.playSound(neu === 1 ? "click" : "merge");
      this.checkWin();
    }

    lose(cell) {
      this.exploded = cell;
      this.revealed.add(cell);
      this.gameOver = true;
      this.won = false;
      this.score = 0;
      this.reportResult(false);
      this.playSound("explode");
      this.playSound("gameover");
      this.rumble(250);
    }

    checkWin() {
      if (this.revealed.size !== this.cols * this.rows - this.nMines) return;
      this.gameOver = true;
      this.won = true;
      // Restliche Minen automatisch flaggen
      for (const m of this.mines) this.flags.set(m, 1);
      const sekunden = Math.trunc(this.elapsed);
      this.score = Math.max(10, this.basePoints - sekunden);
      // Bestzeit je Schwierigkeitsgrad merken
      const key = PRESET_KEYS[this.presetIndex];
      if (!(key in this.best) || sekunden < Math.trunc(this.best[key])) {
        this.best[key] = sekunden;
        this.saveSetting("best", Object.assign({}, this.best));
      }
      this.reportResult(true);
      this.achEvent("mine_win");
      this.confetti();
      this.playSound("win");
      this.rumble(200);
    }

    confetti() {
      const cols = Object.values(NUMBER_COLORS);
      for (let i = 0; i < 90; i++) {
        const x = PG.rand.uniform(0, this.width);
        const farbe = PG.rand.choice(cols);
        this.particles.push([x, PG.rand.uniform(-80, 0), PG.rand.uniform(-30, 30), PG.rand.uniform(60, 190), PG.rand.uniform(1.5, 3.2), farbe]);
      }
    }

    // ===================================================== Spiellogik
    stepParticles(dt) {
      const rest = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[3] += 60 * dt;
        p[4] -= dt;
        if (p[4] > 0 && p[1] < this.height + 10) rest.push(p);
      }
      this.particles = rest;
    }

    update(dt) {
      this.animT += dt;
      if (this.surprise > 0) this.surprise -= dt;
      this.stepParticles(dt);
      if (this.state === PLAY && this.running && !this.gameOver) this.elapsed += dt;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Web: update() läuft bei gameOver nicht mehr -> Konfetti hier weiterbewegen.
      const now = ui.now();
      if (this.gameOver && this.particles.length && this.lastDrawT !== null) {
        this.stepParticles(Math.min(0.1, now - this.lastDrawT));
      }
      this.lastDrawT = now;

      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      ui.drawBackground(ctx, this.width, this.height);
      this.drawHud(ctx);
      this.drawBoard(ctx);
      for (const p of this.particles) draw.rect(ctx, p[5], [Math.trunc(p[0]), Math.trunc(p[1]), 4, 6]);
      if (this.gameOver) this.drawResult(ctx);
    }

    // ----- HUD ---------------------------------------------------------------
    drawHud(ctx) {
      const hud = new Rect(10, 6, this.width - 20, HUD_H - 4);
      draw.rect(ctx, ui.PANEL, hud, 0, 10);
      draw.rect(ctx, ui.BORDER, hud, 1, 10);
      // Minenzähler (Minen minus Flaggen)
      let flaggen = 0;
      for (const v of this.flags.values()) if (v === 1) flaggen++;
      const wert = Math.max(-99, this.nMines - flaggen);
      const ledL = this.drawLed(ctx, pad3(wert), 24);
      // Timer
      const sek = Math.trunc(this.elapsed);
      this.drawLed(ctx, pad3(Math.min(sek, 999)), null, this.width - 24);
      // Schwierigkeitsname klein daneben
      ui.text(ctx, t("mines.preset." + PRESET_KEYS[this.presetIndex]), ledL.right + 12, fl(HUD_H / 2) - fl(this.tiny.height / 2) + 2, this.tiny, ui.TEXT_DIM);
      this.drawFace(ctx);
    }

    /** Zeichnet eine LED-Anzeige und liefert ihr Rechteck zurück. */
    drawLed(ctx, text, links, rechts = null) {
      const w = this.led.width(text) + 14, h = this.led.height + 8;
      const x = rechts === null ? links : rechts - w;
      const y = fl((HUD_H - h) / 2) + 2;
      const rect = new Rect(x, y, w, h);
      draw.rect(ctx, [16, 12, 14], rect, 0, 6);
      draw.rect(ctx, [60, 40, 44], rect, 1, 6);
      ui.text(ctx, text, x + 7, y + fl(h / 2), this.led, COL_LED, "midleft");
      return rect;
    }

    drawFace(ctx) {
      const r = this.faceRect;
      draw.rect(ctx, ui.BTN, r, 0, 8);
      draw.rect(ctx, COL_BEVEL_L, r, 2, 8);
      const [cx, cy] = r.center;
      const gelb = [245, 205, 90];
      const braun = [60, 40, 20];
      draw.circle(ctx, gelb, [cx, cy], 14);
      draw.circle(ctx, [120, 95, 30], [cx, cy], 14, 2);
      if (this.gameOver && !this.won) {
        // tot: X-Augen, gerader Mund
        for (const ex of [cx - 6, cx + 6]) {
          draw.line(ctx, braun, [ex - 3, cy - 8], [ex + 3, cy - 2], 2);
          draw.line(ctx, braun, [ex + 3, cy - 8], [ex - 3, cy - 2], 2);
        }
        draw.line(ctx, braun, [cx - 6, cy + 7], [cx + 6, cy + 7], 2);
      } else if (this.gameOver && this.won) {
        // Sieg: Sonnenbrille + Lächeln
        draw.rect(ctx, [30, 30, 40], [cx - 10, cy - 8, 8, 6], 0, 2);
        draw.rect(ctx, [30, 30, 40], [cx + 2, cy - 8, 8, 6], 0, 2);
        draw.line(ctx, [30, 30, 40], [cx - 2, cy - 6], [cx + 2, cy - 6], 2);
        draw.arc(ctx, braun, [cx - 7, cy - 2, 14, 11], Math.PI, PG.TAU, 2);
      } else if (this.surprise > 0) {
        // staunen beim Klicken
        draw.circle(ctx, braun, [cx - 5, cy - 5], 2);
        draw.circle(ctx, braun, [cx + 5, cy - 5], 2);
        draw.circle(ctx, braun, [cx, cy + 6], 4, 2);
      } else {
        draw.circle(ctx, braun, [cx - 5, cy - 5], 2);
        draw.circle(ctx, braun, [cx + 5, cy - 5], 2);
        draw.arc(ctx, braun, [cx - 7, cy - 3, 14, 12], Math.PI, PG.TAU, 2);
      }
    }

    // ----- Spielfeld ----------------------------------------------------------
    drawBoard(ctx) {
      const brett = new Rect(this.bx - 8, this.by - 8, this.cols * this.cell + 16, this.rows * this.cell + 16);
      draw.rect(ctx, ui.PANEL, brett, 0, 10);
      draw.rect(ctx, ui.BORDER, brett, 1, 10);
      for (let i = 0; i < this.cols * this.rows; i++) this.drawCell(ctx, i);
    }

    drawCell(ctx, cell) {
      const x = cell % this.cols, y = fl(cell / this.cols);
      const c = this.cell;
      const rx = this.bx + x * c, ry = this.by + y * c;
      const rect = new Rect(rx, ry, c, c);
      const offen = this.revealed.has(cell);
      const zeigeMine = this.gameOver && this.mines.has(cell);

      if (offen || (zeigeMine && this.flags.get(cell) !== 1)) {
        draw.rect(ctx, cell === this.exploded ? COL_BOOM : COL_OPEN, rect);
        draw.rect(ctx, COL_OPEN_LINE, rect, 1);
        if (this.mines.has(cell)) {
          this.drawMine(ctx, rect);
        } else {
          const zahl = this.numbers[cell] || 0;
          if (zahl > 0) ui.text(ctx, String(zahl), rect.centerx, rect.centery, this.numFont, NUMBER_COLORS[zahl], "center");
        }
      } else {
        draw.rect(ctx, cell === this.hover && !this.gameOver ? COL_TILE_HOVER : COL_TILE, rect);
        // 3D-Kanten
        draw.line(ctx, COL_BEVEL_L, [rx, ry + 1], [rx + c - 1, ry + 1], 2);
        draw.line(ctx, COL_BEVEL_L, [rx + 1, ry], [rx + 1, ry + c - 1], 2);
        draw.line(ctx, COL_BEVEL_D, [rx + 1, ry + c - 1], [rx + c - 1, ry + c - 1], 2);
        draw.line(ctx, COL_BEVEL_D, [rx + c - 1, ry + 1], [rx + c - 1, ry + c - 1], 2);
        const zustand = this.flags.get(cell) || 0;
        if (zustand === 1) {
          this.drawFlag(ctx, rect);
          // falsche Flagge bei Spielende durchstreichen
          if (this.gameOver && !this.mines.has(cell)) {
            draw.line(ctx, [250, 90, 90], rect.topleft, rect.bottomright, 2);
            draw.line(ctx, [250, 90, 90], rect.topright, rect.bottomleft, 2);
          }
        } else if (zustand === 2) {
          ui.text(ctx, "?", rect.centerx, rect.centery, this.numFont, ui.TEXT_DIM, "center");
        }
      }
    }

    drawMine(ctx, rect) {
      const [cx, cy] = rect.center;
      const r = Math.max(3, fl(this.cell / 4));
      for (let winkel = 0; winkel < 360; winkel += 45) {
        const a = PG.radians(winkel);
        draw.line(ctx, [20, 22, 30], [cx, cy], [cx + Math.cos(a) * (r + 3), cy + Math.sin(a) * (r + 3)], 2);
      }
      draw.circle(ctx, [20, 22, 30], [cx, cy], r);
      draw.circle(ctx, [90, 95, 110], [cx - fl(r / 3), cy - fl(r / 3)], Math.max(1, fl(r / 4)));
    }

    drawFlag(ctx, rect) {
      const [cx, cy] = rect.center;
      const h = this.cell * 0.6;
      const x = cx - 1;
      draw.line(ctx, [200, 205, 215], [x, cy - h / 2], [x, cy + h / 2], 2);
      draw.polygon(ctx, COL_FLAG, [[x, cy - h / 2], [x + h * 0.55, cy - h * 0.28], [x, cy - h * 0.06]]);
      draw.line(ctx, [200, 205, 215], [x - h * 0.25, cy + h / 2], [x + h * 0.3, cy + h / 2], 2);
    }

    /** Ergebnis-Banner mit Neustart-Hinweis (transluzentes Themen-Panel). */
    drawResult(ctx) {
      let text, farbe;
      if (this.won) {
        text = t("mines.win", { t: Math.trunc(this.elapsed) + "s" });
        farbe = ui.GREEN;
      } else {
        text = t("mines.lose");
        farbe = ui.RED;
      }
      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0));
      const hint = t("common.enter_restart");

      const w = Math.max(this.font.width(text), this.small.width(hint)) + 36;
      const h = this.font.height + this.small.height + 22;
      const r = new Rect(fl(this.width / 2) - fl(w / 2), this.by - 10, w, h);
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], r, 0, 10);
      draw.rect(ctx, farbe, r, 2, 10);
      ui.text(ctx, text, r.centerx, r.y + 8, this.font, farbe, "midtop");
      ui.text(ctx, hint, r.centerx, r.y + 8 + this.font.height + 4, this.small, hintCol, "midtop");
    }

    // ----- Setup zeichnen ------------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      ui.drawTitle(ctx, this.width, "MINESWEEPER", { subtitle: t("mines.subtitle"), accent: this.accent });

      PRESETS.forEach(([key, cols, rows, minen], i) => {
        const r = this.presetRects[i];
        const an = i === this.presetIndex;
        draw.rect(ctx, an ? ui.BTN_SEL : ui.BTN, r, 0, 8);
        draw.rect(ctx, an ? this.accent : ui.BORDER, r, an ? 2 : 1, 8);
        // Web: Zeilen über ihre Mitte platzieren (Canvas-Fonts sind höher als pygame-Fonts)
        ui.text(ctx, t("mines.preset." + key), r.x + 16, r.y + 15, this.font, ui.TEXT, "midleft");
        const details = `${cols}x${rows}   ${minen} ${t("mines.mines")}`;
        ui.text(ctx, details, r.x + 16, r.bottom - 10, this.tiny, ui.TEXT_DIM, "midleft");
        const best = this.best[key];
        const has = best !== undefined && best !== null;
        const btxt = has ? t("mines.best", { t: Math.trunc(best) + "s" }) : t("mines.best_none");
        ui.text(ctx, btxt, r.right - 16, r.centery, this.small, has ? ui.GOLD : ui.TEXT_DIM, "midright");
      });

      // Fragezeichen-Toggle
      const r = this.qmarkRect;
      draw.rect(ctx, this.qmarks ? ui.BTN_SEL : ui.BTN, r, 0, 8);
      draw.rect(ctx, ui.BORDER, r, 1, 8);
      ui.text(ctx, t("mines.qmarks"), r.x + 16, r.centery, this.font, ui.TEXT, "midleft");
      const wert = this.qmarks ? t("common.on") : t("common.off");
      ui.text(ctx, `< ${wert} >`, r.right - 16, r.centery, this.font, this.qmarks ? this.accent : ui.TEXT_DIM, "midright");

      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: ui.GREEN });

      ui.text(ctx, t("mines.hint"), fl(this.width / 2), this.height - 48, this.tiny, ui.GREEN, "center");
      ui.drawFooter(ctx, this.width, this.height, t("mines.setup_hint"));
    }
  }

  PG.register(MinesweeperGame, {
    id: "MinesweeperGame",
    key: "minesweeper",
    name: "Minesweeper",
    settingsKey: "minesweeper",
    defaults: { preset: "beginner", qmarks: false, best: {} },
    wantsRightClick: true, // Rechtsklick = Flagge
  });
})();
