/*
 * slidepuzzle.js - Schiebepuzzle / 15-Puzzle (Port von games/slidepuzzle.py)
 * ==========================================================================
 * - Drei Größen (Modi): 3x3 (leicht), 4x4 (klassisch) und 5x5 (schwer).
 * - Gemischt wird durch viele zufällige, GÜLTIGE Züge ausgehend vom gelösten
 *   Feld - so ist das Puzzle immer lösbar (kein Paritätsproblem).
 * - Steuerung: Klick auf eine Kachel in derselben Reihe/Spalte wie die Lücke
 *   schiebt die ganze Linie; Pfeiltasten schieben die an die Lücke grenzende
 *   Kachel hinein.
 * - Punkte (höher = besser): Grundpunktzahl je Größe minus Züge und Zeit.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Grundpunktzahl und Abzüge je Kantenlänge N.
  const BASE = { 3: 2000, 4: 6000, 5: 12000 };
  const MOVE_PEN = { 3: 8, 4: 6, 5: 4 };
  const TIME_PEN = 5;

  const PLAY = "play", SOLVED = "solved";

  class SlidingPuzzleGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.N = { 3: 3, 4: 4, 5: 5 }[this.mode] || 4;
      this.score = 0;
      this.gameOver = false;
      this.state = PLAY;
      this.moves = 0;
      this.elapsed = 0;
      this.flash = {}; // index -> restliche Aufleucht-Zeit
      this.makeFonts();
      this.newBoard();
      this.layout();
    }

    makeFonts() {
      this.tileFont = ui.font(Math.max(22, Math.floor(this.height / 12)), true);
      this.hud = ui.font(20, true);
      this.small = ui.font(15);
      this.huge = ui.font(Math.max(30, Math.floor(this.height / 11)), true);
    }

    newBoard() {
      const n = this.N;
      // 0 = Lücke; gelöst: [1,2,...,n*n-1, 0]
      this.solved = [];
      for (let i = 1; i < n * n; i++) this.solved.push(i);
      this.solved.push(0);
      this.board = this.solved.slice();
      this.blank = n * n - 1;
      this.scramble();
      this.moves = 0;
      this.elapsed = 0;
      this.state = PLAY;
      this.flash = {};
    }

    /** Mischt durch viele zufällige gültige Züge (immer lösbar). */
    scramble() {
      const n = this.N;
      do {
        let last = -1;
        for (let k = 0; k < n * n * 60; k++) {
          const nb = this.neighbors(this.blank).filter((i) => i !== last);
          const pick = PG.rand.choice(nb);
          last = this.blank;
          this.board[this.blank] = this.board[pick];
          this.board[pick] = 0;
          this.blank = pick;
        }
      } while (this.isSolved()); // extrem unwahrscheinlich
    }

    isSolved() {
      for (let i = 0; i < this.board.length; i++) if (this.board[i] !== this.solved[i]) return false;
      return true;
    }

    neighbors(idx) {
      const n = this.N;
      const r = Math.floor(idx / n), c = idx % n;
      const out = [];
      if (r > 0) out.push(idx - n);
      if (r < n - 1) out.push(idx + n);
      if (c > 0) out.push(idx - 1);
      if (c < n - 1) out.push(idx + 1);
      return out;
    }

    layout() {
      const n = this.N;
      const top = 74;
      const bottom = this.height - 40;
      const avail = Math.min(this.width - 48, bottom - top);
      this.boardPx = Math.max(120, avail);
      this.gap = Math.max(3, Math.floor(this.boardPx / (n * 22)));
      this.tile = Math.floor((this.boardPx - (n + 1) * this.gap) / n);
      this.boardPx = n * this.tile + (n + 1) * this.gap;
      this.ox = Math.floor((this.width - this.boardPx) / 2);
      this.oy = top + Math.max(0, Math.floor((bottom - top - this.boardPx) / 2));
    }

    tileRect(idx) {
      const r = Math.floor(idx / this.N), c = idx % this.N;
      const x = this.ox + this.gap + c * (this.tile + this.gap);
      const y = this.oy + this.gap + r * (this.tile + this.gap);
      return new PG.Rect(x, y, this.tile, this.tile);
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SOLVED) {
        if (ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"))) {
          this.gameOver = false;
          this.newBoard();
          this.playSound("click");
        }
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Up" || k === "w" || k === "W") this.slideFrom(this.blank + this.N); // Kachel unten rückt hoch
        else if (k === "Down" || k === "s" || k === "S") this.slideFrom(this.blank - this.N);
        else if (k === "Left" || k === "a" || k === "A") this.slideFrom(this.blank + 1); // Kachel rechts rückt links
        else if (k === "Right" || k === "d" || k === "D") this.slideFrom(this.blank - 1);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.click(ev.pos);
      }
    }

    click(pos) {
      for (let idx = 0; idx < this.N * this.N; idx++) {
        if (idx !== this.blank && this.tileRect(idx).collidepoint(pos)) {
          this.slideLine(idx);
          return;
        }
      }
    }

    /** Schiebt die Kachel an Position idx in die Lücke (nur wenn benachbart). */
    slideFrom(idx) {
      const n = this.N;
      if (idx >= 0 && idx < n * n && this.neighbors(this.blank).includes(idx)) this.swap(idx);
    }

    /** Klick: schiebt die ganze Linie zwischen Lücke und angeklickter Kachel. */
    slideLine(idx) {
      const n = this.N;
      const br = Math.floor(this.blank / n), bc = this.blank % n;
      const cr = Math.floor(idx / n), cc = idx % n;
      if (cr === br) {
        const step = cc > bc ? 1 : -1;
        while (this.blank !== idx && this.state === PLAY) this.swap(this.blank + step);
      } else if (cc === bc) {
        const step = cr > br ? n : -n;
        while (this.blank !== idx && this.state === PLAY) this.swap(this.blank + step);
      } else {
        this.playSound("click"); // keine gültige Linie
      }
    }

    /** Vertauscht Lücke mit Kachel idx (idx muss benachbart sein). */
    swap(idx) {
      this.board[this.blank] = this.board[idx];
      this.board[idx] = 0;
      this.flash[this.blank] = 0.18; // Zielfeld leuchtet kurz
      this.blank = idx;
      this.moves += 1;
      this.playSound("move");
      if (this.isSolved()) this.win();
    }

    win() {
      const n = this.N;
      const pts = Math.max(50, (BASE[n] || 6000) - this.moves * (MOVE_PEN[n] || 6) - Math.floor(this.elapsed) * TIME_PEN);
      this.score = pts;
      this.state = SOLVED;
      this.gameOver = true; // die App speichert den Highscore
      this.reportResult(true);
      this.playSound("win");
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === PLAY) this.elapsed += dt;
      for (const idx of Object.keys(this.flash)) {
        this.flash[idx] -= dt;
        if (this.flash[idx] <= 0) delete this.flash[idx];
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.drawBoard(ctx);
      if (this.state === SOLVED) this.drawBanner(ctx);
    }

    drawHud(ctx) {
      ui.text(ctx, t("slide.title"), 20, 30, this.hud, this.accent, "midleft");
      ui.text(ctx, t("slide.moves", { n: this.moves }), this.width - 20, 22, this.small, ui.TEXT, "midright");
      ui.text(ctx, t("slide.time", { t: this.fmtTime() }), this.width - 20, 44, this.small, ui.TEXT_DIM, "midright");
      ui.text(ctx, t("slide.hint"), this.width / 2, 52, this.small, ui.TEXT_FAINT, "center");
    }

    fmtTime() {
      const sec = Math.floor(this.elapsed);
      return Math.floor(sec / 60) + ":" + String(sec % 60).padStart(2, "0");
    }

    drawBoard(ctx) {
      // Rahmen/Panel hinter dem Feld
      const pad = this.gap;
      const panel = [this.ox - pad, this.oy - pad, this.boardPx + 2 * pad, this.boardPx + 2 * pad];
      draw.rect(ctx, [24, 28, 42], panel, 0, 12);
      draw.rect(ctx, ui.mix(this.accent, [20, 24, 38], 0.6), panel, 2, 12);
      // Flash läuft im Game Over nicht weiter (update steht) -> dort ignorieren
      const flash = this.state === PLAY ? this.flash : {};
      for (let idx = 0; idx < this.N * this.N; idx++) {
        const val = this.board[idx];
        if (val === 0) continue;
        const rc = this.tileRect(idx);
        let base = ui.mix(this.accent, [235, 240, 250], 0.12);
        if (idx in flash) base = ui.mix(base, [255, 255, 255], Math.min(1, flash[idx] / 0.18));
        const correct = val === this.solved[idx];
        const fill = correct ? base : ui.mix(base, [40, 46, 66], 0.42);
        draw.rect(ctx, fill, rc, 0, 8);
        draw.rect(ctx, ui.mix(fill, [255, 255, 255], 0.18), rc, 2, 8);
        const col = correct ? [20, 24, 34] : ui.TEXT;
        ui.text(ctx, String(val), rc.centerx, rc.centery, this.tileFont, col, "center");
      }
    }

    drawBanner(ctx) {
      const w = Math.min(this.width - 40, 460);
      const h = 118;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), Math.floor((this.height - h) / 2), w, h);
      draw.rect(ctx, [16, 18, 24, 236], rc);
      draw.rect(ctx, this.accent, rc, 2, 14);
      ui.text(ctx, t("slide.solved"), rc.centerx, rc.y + 38, this.huge, this.accent, "center");
      ui.text(ctx, t("slide.score", { n: this.score }), rc.centerx, rc.y + 74, this.hud, ui.GOLD, "center");
      ui.text(ctx, t("common.enter_restart"), rc.centerx, rc.y + 100, this.small, ui.TEXT_DIM, "center");
    }
  }

  PG.register(SlidingPuzzleGame, {
    id: "SlidingPuzzleGame",
    key: "slide",
    name: { default: "Sliding Puzzle", de: "Schiebepuzzle", fr: "Taquin", es: "Puzle deslizante", pt: "Quebra-cabeça deslizante" },
    modes: [["3", "slide.mode.3"], ["4", "slide.mode.4"], ["5", "slide.mode.5"]],
  });
})();
