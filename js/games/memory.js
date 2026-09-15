/*
 * memory.js - Memory (Paare finden) (Port von games/memory.py)
 * ============================================================
 * - Brettgrößen 4x4, 6x6 und 8x6 (Setup, gespeichert).
 * - Die Motive (8 Formen x 3 Farben = 24 eindeutige Paare) werden komplett mit
 *   Canvas-Primitiven gezeichnet - keine Bild-Dateien nötig.
 * - Karten drehen sich mit einer kurzen Flip-Animation; gefundene Paare bleiben
 *   gedimmt mit Häkchen liegen.
 * - Solo: wenige Züge + schnelle Zeit = mehr Punkte.
 *   (Das lokale 2-Spieler-Duell der Desktop-Version entfällt im Web.)
 *
 * Steuerung: Maus oder Pfeile/WASD + Leertaste/Enter, R = neue Runde, S = Setup.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const Rect = PG.Rect;
  const fl = Math.floor;

  // Identitätsfarben der Karten - bleiben bewusst fest, alle generischen
  // UI-Farben kommen zur Laufzeit dynamisch aus der ui-Palette.
  const COL_CARD_BACK = [56, 64, 92];
  const COL_CARD_BACK_D = [40, 46, 66];
  const COL_CARD_FACE = [232, 234, 240];
  const COL_CARD_DONE = [58, 66, 88];
  const COL_CARD_EDGE = [74, 84, 116];

  // (Schlüssel, Spalten, Reihen, Basispunkte)
  const SIZES = [["4x4", 4, 4, 1000], ["6x6", 6, 6, 2500], ["8x6", 8, 6, 4000]];
  const SIZE_KEYS = SIZES.map((s) => s[0]);

  // Motiv-Farben (3 Gruppen) - kombiniert mit 8 Formen = 24 eindeutige Motive.
  const MOTIF_COLORS = [[225, 95, 95], [88, 156, 255], [245, 205, 100]];

  const FLIP_TIME = 0.25; // Dauer der Dreh-Animation
  const RESOLVE_TIME = 0.8; // Anzeigezeit eines Fehlpaars

  const SETUP = "setup", PLAY = "play";

  // ----- Motiv-Formen (zeichnen in ein Rechteck) -------------------------------
  function ptsStar(cx, cy, r) {
    const pts = [];
    for (let i = 0; i < 10; i++) {
      const rr = i % 2 === 0 ? r : r * 0.45;
      const a = -Math.PI / 2 + (i * Math.PI) / 5;
      pts.push([cx + rr * Math.cos(a), cy + rr * Math.sin(a)]);
    }
    return pts;
  }

  function drawStar(ctx, rect, col) {
    draw.polygon(ctx, col, ptsStar(rect.centerx, rect.centery, rect.w * 0.45));
  }

  function drawHeart(ctx, rect, col) {
    const r = fl(rect.w / 4);
    const cx = rect.centerx, cy = rect.centery - fl(r / 3);
    draw.circle(ctx, col, [cx - r + 1, cy], r);
    draw.circle(ctx, col, [cx + r - 1, cy], r);
    draw.polygon(ctx, col, [[cx - 2 * r + 1, cy + fl(r / 3)], [cx + 2 * r - 1, cy + fl(r / 3)], [cx, cy + 2 * r]]);
  }

  function drawMoon(ctx, rect, col, face) {
    // Der "Ausschnitt" wird mit der aktuellen Kartenfarbe gefüllt - sonst
    // bliebe auf gedimmten (gefundenen) Karten ein heller Kreis stehen.
    const r = Math.trunc(rect.w * 0.4);
    draw.circle(ctx, col, rect.center, r);
    draw.circle(ctx, face, [rect.centerx + fl(r / 2), rect.centery - fl(r / 3)], r);
  }

  function drawDiamond(ctx, rect, col) {
    const [cx, cy] = rect.center;
    const w = rect.w * 0.38, h = rect.h * 0.46;
    draw.polygon(ctx, col, [[cx, cy - h], [cx + w, cy], [cx, cy + h], [cx - w, cy]]);
  }

  function drawTriangle(ctx, rect, col) {
    const [cx, cy] = rect.center;
    const r = rect.w * 0.42;
    draw.polygon(ctx, col, [[cx, cy - r], [cx + r, cy + r * 0.7], [cx - r, cy + r * 0.7]]);
  }

  function drawRing(ctx, rect, col) {
    draw.circle(ctx, col, rect.center, Math.trunc(rect.w * 0.4), Math.max(3, fl(rect.w / 8)));
  }

  function drawCross(ctx, rect, col) {
    const [cx, cy] = rect.center;
    const a = Math.trunc(rect.w * 0.42);
    const b = Math.max(3, fl(rect.w / 7));
    draw.rect(ctx, col, [cx - b, cy - a, 2 * b, 2 * a], 0, b);
    draw.rect(ctx, col, [cx - a, cy - b, 2 * a, 2 * b], 0, b);
  }

  function drawBolt(ctx, rect, col) {
    const [cx, cy] = rect.center;
    const w = rect.w * 0.32, h = rect.h * 0.45;
    draw.polygon(ctx, col, [
      [cx + w * 0.4, cy - h], [cx - w, cy + h * 0.15], [cx - w * 0.05, cy + h * 0.15],
      [cx - w * 0.4, cy + h], [cx + w, cy - h * 0.15], [cx + w * 0.05, cy - h * 0.15],
    ]);
  }

  const SHAPES = [drawStar, drawHeart, drawMoon, drawDiamond, drawTriangle, drawRing, drawCross, drawBolt];

  class MemoryGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const idx = SIZE_KEYS.indexOf(this.opts.size);
      this.sizeIdx = idx >= 0 ? idx : 1;

      this.buildFonts();
      this.buildSetupLayout();
      this.cards = [];
      this.lastDrawT = null;
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, fl(h / 30)));
      this.tiny = ui.font(Math.max(11, fl(h / 36)));
      this.huge = ui.font(Math.max(26, fl(h / 11)), true);
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = fl(this.width / 2);
      const bw = Math.min(360, this.width - 60);
      const y0 = Math.trunc(this.height * 0.32);
      this.sizeRects = [0, 1, 2].map((i) => new Rect(cx - fl(bw / 2), y0 + i * 58, bw, 48));
      this.startRect = new Rect(cx - 95, y0 + 3 * 58 + 14, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3"].includes(k)) {
          this.sizeIdx = Number(k) - 1;
          this.saveSetting("size", SIZE_KEYS[this.sizeIdx]);
          this.playSound("click");
        } else if (["Up", "w", "W"].includes(k)) {
          this.sizeIdx = PG.mod(this.sizeIdx - 1, 3);
          this.saveSetting("size", SIZE_KEYS[this.sizeIdx]);
          this.playSound("move");
        } else if (["Down", "s", "S"].includes(k)) {
          this.sizeIdx = PG.mod(this.sizeIdx + 1, 3);
          this.saveSetting("size", SIZE_KEYS[this.sizeIdx]);
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.newGame();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.sizeRects.length; i++) {
          if (this.sizeRects[i].collidepoint(ev.pos)) {
            this.sizeIdx = i;
            this.saveSetting("size", SIZE_KEYS[i]);
            this.playSound("click");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.newGame();
      }
    }

    // ===================================================== Spielaufbau
    newGame() {
      this.score = 0;
      this.gameOver = false;
      const [, cols, rows, base] = SIZES[this.sizeIdx];
      this.cols = cols;
      this.rowsN = rows;
      this.base = base;
      const nPairs = fl((cols * rows) / 2);

      // Motive: (Form, Farbe)-Katalog mischen, n Paare ziehen, duplizieren.
      const catalog = [];
      for (let sh = 0; sh < SHAPES.length; sh++) for (let co = 0; co < MOTIF_COLORS.length; co++) catalog.push([sh, co]);
      PG.rand.shuffle(catalog);
      const pairs = catalog.slice(0, nPairs);
      const motifs = pairs.concat(pairs);
      PG.rand.shuffle(motifs);

      this.cards = motifs.map((m) => ({ motif: m, state: "down", p: 0, target: 0 }));
      this.first = null; // Index der ersten offenen Karte
      this.resolveT = 0; // > 0: Fehlpaar liegt offen
      this.resolvePair = null;
      this.moves = 0;
      this.elapsed = 0;
      this.cursor = 0;
      this.found = 0;
      this.layoutBoard();
      this.state = PLAY;
      this.playSound("click");
    }

    layoutBoard() {
      this.hudH = Math.max(40, Math.trunc(this.height * 0.09));
      const m = Math.max(8, fl(this.width / 100));
      const slotW = fl((this.width - m * (this.cols + 1)) / this.cols);
      const slotH = fl((this.height - this.hudH - m * (this.rowsN + 1)) / this.rowsN);
      this.ch = Math.min(slotH, Math.trunc((slotW * 4) / 3));
      this.cw = Math.trunc(this.ch * 0.75);
      const totalW = this.cols * this.cw + (this.cols - 1) * m;
      const totalH = this.rowsN * this.ch + (this.rowsN - 1) * m;
      this.bx = fl((this.width - totalW) / 2);
      this.by = this.hudH + fl((this.height - this.hudH - totalH) / 2);
      this.gap = m;
    }

    cardRect(i) {
      const r = fl(i / this.cols), c = i % this.cols;
      return new Rect(this.bx + c * (this.cw + this.gap), this.by + r * (this.ch + this.gap), this.cw, this.ch);
    }

    cardAt(pos) {
      for (let i = 0; i < this.cards.length; i++) if (this.cardRect(i).collidepoint(pos)) return i;
      return null;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.newGame();
          } else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            // Web: gameOver zurücksetzen, sonst liegt das Highscore-Banner über dem Setup.
            this.gameOver = false;
            this.playSound("click");
          }
        }
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "r" || k === "R") {
          this.newGame();
        } else if ((k === "s" || k === "S") && !this.isAction(k, "down")) {
          this.state = SETUP;
          this.playSound("click");
        } else if (this.isAction(k, "up") || k === "Up") {
          this.moveCursor(0, -1);
        } else if (this.isAction(k, "down") || k === "Down") {
          this.moveCursor(0, 1);
        } else if (this.isAction(k, "left") || k === "Left") {
          this.moveCursor(-1, 0);
        } else if (this.isAction(k, "right") || k === "Right") {
          this.moveCursor(1, 0);
        } else if (k === "space" || k === "Return") {
          this.flip(this.cursor);
        }
      } else if (ev.kind === "mousedown") {
        const i = this.cardAt(ev.pos);
        if (i !== null) {
          this.cursor = i;
          this.flip(i);
        }
      }
    }

    moveCursor(dx, dy) {
      let r = fl(this.cursor / this.cols), c = this.cursor % this.cols;
      c = Math.max(0, Math.min(this.cols - 1, c + dx));
      r = Math.max(0, Math.min(this.rowsN - 1, r + dy));
      const neu = r * this.cols + c;
      if (neu !== this.cursor) {
        // am Rand: kein Klick-Geräusch
        this.cursor = neu;
        this.playSound("move");
      }
    }

    // ===================================================== Spiellogik
    flip(i) {
      if (this.resolveT > 0) return;
      const card = this.cards[i];
      if (card.state !== "down") return;
      card.state = "up";
      card.target = 1;
      this.playSound("click");
      if (this.first === null) {
        this.first = i;
        return;
      }
      // Zweite Karte
      this.moves++;
      const a = this.cards[this.first], b = card;
      if (a.motif === b.motif || (a.motif[0] === b.motif[0] && a.motif[1] === b.motif[1])) {
        a.state = b.state = "done";
        this.found++;
        this.first = null;
        this.playSound("merge");
        if (this.found >= fl(this.cards.length / 2)) this.finish();
      } else {
        this.resolveT = RESOLVE_TIME;
        this.resolvePair = [this.first, i];
        this.first = null;
      }
    }

    finish() {
      this.score = Math.max(100, this.base - 15 * this.moves - 2 * Math.trunc(this.elapsed));
      // Perfekt = jedes Paar im ersten Anlauf gefunden.
      if (this.moves === fl(this.cards.length / 2)) this.achEvent("memory_perfect");
      this.gameOver = true;
      this.playSound("win");
      this.rumble(200);
    }

    stepFlips(dt) {
      for (const card of this.cards) {
        if (card.p < card.target) card.p = Math.min(card.target, card.p + dt / FLIP_TIME);
        else if (card.p > card.target) card.p = Math.max(card.target, card.p - dt / FLIP_TIME);
      }
    }

    update(dt) {
      if (this.state !== PLAY || this.gameOver) return;
      this.elapsed += dt;
      // Flip-Animationen
      this.stepFlips(dt);
      // Fehlpaar wieder zudecken
      if (this.resolveT > 0) {
        this.resolveT -= dt;
        if (this.resolveT <= 0 && this.resolvePair) {
          for (const i of this.resolvePair) {
            if (this.cards[i].state === "up") {
              this.cards[i].state = "down";
              this.cards[i].target = 0;
            }
          }
          this.resolvePair = null;
          this.playSound("move");
        }
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Web: update() läuft bei gameOver nicht -> letzte Karte hier fertig drehen.
      const now = ui.now();
      if (this.gameOver && this.lastDrawT !== null) this.stepFlips(Math.min(0.1, now - this.lastDrawT));
      this.lastDrawT = now;

      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      ui.drawBackground(ctx, this.width, this.height);
      this.cards.forEach((card, i) => this.drawCard(ctx, i, card));
      this.drawHud(ctx);
      if (this.gameOver) this.drawResult(ctx);
    }

    drawCard(ctx, i, card) {
      const rect = this.cardRect(i);
      // Flip: Breite skaliert mit |cos(pi*p)|, Seite wechselt bei p=0.5
      const wf = Math.abs(Math.cos(Math.PI * card.p));
      const w = Math.max(3, Math.trunc(rect.w * wf));
      const r = new Rect(rect.centerx - fl(w / 2), rect.y, w, rect.h);
      const front = card.p > 0.5;
      const rad = Math.max(4, fl(rect.w / 8));

      if (front) {
        const faceCol = card.state === "done" ? COL_CARD_DONE : COL_CARD_FACE;
        draw.rect(ctx, faceCol, r, 0, rad);
        draw.rect(ctx, COL_CARD_EDGE, r, 2, rad);
        if (w > rect.w * 0.5) {
          // Motiv erst zeigen, wenn Karte weit genug offen
          const [shapeI, colI] = card.motif;
          let col = MOTIF_COLORS[colI];
          if (card.state === "done") col = col.map((v) => Math.trunc(v * 0.55));
          const inner = rect.inflate(-fl(rect.w / 4), -fl(rect.h / 3));
          SHAPES[shapeI](ctx, inner, col, faceCol);
          if (card.state === "done") {
            const bx = rect.right - 12, by = rect.bottom - 10;
            draw.lines(ctx, ui.GREEN, false, [[bx - 6, by - 3], [bx - 3, by], [bx + 3, by - 7]], 2);
          }
        }
      } else {
        draw.rect(ctx, COL_CARD_BACK, r, 0, rad);
        draw.rect(ctx, COL_CARD_BACK_D, r, 2, rad);
        if (w > rect.w * 0.5) ui.text(ctx, "?", rect.centerx, rect.centery, this.small, this.accent, "center");
      }

      if (i === this.cursor && !this.gameOver) draw.rect(ctx, this.accent, rect.inflate(6, 6), 2, rad + 3);
    }

    fmtTime() {
      const sec = Math.trunc(this.elapsed);
      return String(fl(sec / 60)).padStart(2, "0") + ":" + String(sec % 60).padStart(2, "0");
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH + 0.5], [this.width, this.hudH + 0.5]);
      const cy = fl(this.hudH / 2);
      ui.text(ctx, t("mem.moves", { n: this.moves }), 12, cy, this.small, ui.TEXT, "midleft");
      ui.text(ctx, t("mem.time", { t: this.fmtTime() }), fl(this.width / 2), cy, this.small, this.accent, "center");
      const left = fl(this.cards.length / 2) - this.found;
      ui.text(ctx, t("mem.pairs", { n: left }), this.width - 12, cy, this.small, ui.TEXT_DIM, "midright");
    }

    drawResult(ctx) {
      draw.rect(ctx, [8, 10, 16, 185], [0, 0, this.width, this.height]);
      const cx = fl(this.width / 2), cy = fl(this.height / 2);
      const head = t("mem.win", { t: this.fmtTime(), m: this.moves });
      // Panel hinter dem Ergebnis (Akzent-Rahmen, Breite folgt dem Inhalt).
      const pw = Math.max(Math.min(this.width - 40, 460), this.huge.width(head) + 40);
      const panel = new Rect(cx - fl(pw / 2), cy - 90, pw, 160);
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, this.accent, panel, 2, 14);
      ui.text(ctx, head, cx, cy - 50, this.huge, ui.GREEN, "center");
      ui.text(ctx, t("common.points", { score: this.score }), cx, cy + 4, this.font, ui.TEXT, "center");
      ui.text(ctx, t("mem.retry"), cx, cy + 40, this.small, ui.TEXT_DIM, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------
    drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      const cx = fl(this.width / 2);
      ui.text(ctx, "MEMORY", cx, Math.trunc(this.height * 0.14), this.huge, this.accent, "center");
      ui.text(ctx, t("mem.subtitle"), cx, Math.trunc(this.height * 0.21), this.small, ui.TEXT_DIM, "center");
      this.sizeRects.forEach((r, i) => {
        const on = i === this.sizeIdx;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 10);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 10);
        const [, cols, rows] = SIZES[i];
        ui.text(ctx, `${cols} x ${rows}`, r.x + 18, r.centery, this.font, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
        ui.text(ctx, t("mem.pairs", { n: fl((cols * rows) / 2) }), r.right - 18, r.centery, this.tiny, ui.TEXT_DIM, "midright");
      });
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      draw.rect(ctx, this.accent, this.startRect, 2, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("mem.setup_hint"), cx, this.height - 30, this.tiny, ui.TEXT_DIM, "center");
      ui.text(ctx, t("mem.hint"), cx, this.height - 12, this.tiny, ui.TEXT_FAINT, "center");
    }
  }

  PG.register(MemoryGame, {
    id: "MemoryGame",
    key: "memory",
    name: "Memory",
    settingsKey: "memory",
    defaults: { size: "6x6" },
  });
})();
