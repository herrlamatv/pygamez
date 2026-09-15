/*
 * mastermind.js - Mastermind (Port von games/mastermind.py)
 * ==========================================================
 * Knacke den geheimen Farbcode.
 *
 * - Der Computer würfelt einen verdeckten Code aus mehreren Farbstiften (Dubletten
 *   erlaubt). Du legst je Reihe einen Tipp und bekommst Rückmeldung:
 *   SCHWARZER Pin = richtige Farbe an richtiger Stelle, WEISSER Pin = richtige Farbe
 *   an falscher Stelle (Standard-Mastermind-Zählung mit Häufigkeiten).
 * - Drei Modi: Leicht (4 Stifte / 6 Farben / 12 Reihen), Klassik (4 / 6 / 10) und
 *   Schwer (5 / 8 / 10).
 * - Endlos-Streak wie bei Wordle: jeder geknackte Code bringt Punkte (weniger
 *   Versuche + schwererer Modus = mehr), danach kommt sofort ein neuer Code. Der
 *   erste NICHT geknackte Code beendet die Partie - die Summe ist der Highscore.
 *
 * Steuerung: Farbe unten anklicken (oder Tasten 1..N) füllt den nächsten Platz,
 * Klick auf einen gelegten Stift löscht ihn, [Enter]/OK wertet die Reihe aus,
 * [Rücktaste]/< löscht den letzten. Nach Ende: Enter/Klick geht weiter.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const COL_SLOT = [44, 50, 70];
  const COL_SLOT_BORDER = [70, 78, 104];
  const COL_PEG_BLACK = [26, 28, 36];
  const COL_PEG_WHITE = [236, 238, 245];
  const COL_PEG_EMPTY = [58, 64, 84];

  // Bis zu 8 klar unterscheidbare Stiftfarben.
  const PALETTE = [
    [228, 72, 72], [240, 150, 52], [240, 214, 74], [96, 200, 112],
    [66, 202, 210], [82, 132, 240], [172, 112, 236], [240, 120, 190],
  ];

  // [pins, colors, rows] je Modus.
  const MODE_CFG = { easy: [4, 6, 12], classic: [4, 6, 10], hard: [5, 8, 10] };
  const DIFF_BONUS = { easy: 0, classic: 10, hard: 25 };

  const PLAY = "play", SOLVED = "solved", OVER = "over";

  /** [schwarz, weiß] nach Standard-Mastermind-Regeln. */
  function feedback(guess, secret) {
    let black = 0;
    const sc = {}, gc = {};
    for (let i = 0; i < guess.length; i++) {
      if (guess[i] === secret[i]) black++;
      sc[secret[i]] = (sc[secret[i]] || 0) + 1;
      gc[guess[i]] = (gc[guess[i]] || 0) + 1;
    }
    let total = 0;
    for (const c in gc) total += Math.min(sc[c] || 0, gc[c]);
    return [black, total - black];
  }

  class MastermindGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      [this.pins, this.colors, this.maxRows] = MODE_CFG[this.mode] || MODE_CFG.classic;
      this.score = 0;
      this.gameOver = false;
      this.solvedCount = 0;
      this.makeFonts();
      this.newCode();
      this.layout();
    }

    makeFonts() {
      this.hud = ui.font(20, true);
      this.small = ui.font(15);
      this.tiny = ui.font(13, true);
      this.huge = ui.font(Math.max(28, Math.floor(this.height / 12)), true);
    }

    newCode() {
      this.secret = [];
      for (let i = 0; i < this.pins; i++) this.secret.push(PG.rand.randrange(this.colors));
      this.rowsData = []; // [guess, [black, white]]
      this.current = new Array(this.pins).fill(null);
      this.row = 0;
      this.state = PLAY;
      this.lastPoints = 0;
    }

    layout() {
      const pins = this.pins;
      const gap = 6;
      // Palette unten
      this.palH = Math.max(46, Math.floor(this.height / 10));
      this.palTop = this.height - this.palH - 10;
      this.buildPalette();
      // Reihen-Bereich zwischen HUD und Palette
      const top = 64;
      const bottom = this.palTop - 12;
      const pitch = (bottom - top) / this.maxRows;
      const peg = Math.floor(Math.min(pitch - gap, (this.width - 150) / pins - gap));
      this.peg = Math.max(16, Math.min(peg, 46));
      this.gap = gap;
      this.pitch = pitch;
      this.gridTop = top;
      const gridW = pins * (this.peg + gap) - gap;
      this.fbW = Math.max(26, this.peg);
      const totalW = gridW + 22 + this.fbW;
      this.gridX = Math.floor((this.width - totalW) / 2);
      this.fbX = this.gridX + gridW + 22;
    }

    buildPalette() {
      const gap = 8;
      const C = this.colors;
      let pw = Math.floor(Math.min((this.width - 40) / (C + 3), 52));
      pw = Math.max(28, pw);
      const special = Math.floor(pw * 1.4);
      const rowW = C * pw + (C - 1) * gap + 2 * (special + gap);
      let x = Math.floor((this.width - rowW) / 2);
      const y = this.palTop + Math.floor((this.palH - pw) / 2);
      const h = Math.min(this.palH - 8, pw);
      this.delRect = new PG.Rect(x, y, special, h);
      x += special + gap;
      this.swatchRects = [];
      for (let i = 0; i < C; i++) {
        this.swatchRects.push(new PG.Rect(x, y, pw, h));
        x += pw + gap;
      }
      this.checkRect = new PG.Rect(x, y, special, h);
    }

    rowPegRect(row, i) {
      const x = this.gridX + i * (this.peg + this.gap);
      const y = Math.floor(this.gridTop + row * this.pitch);
      return new PG.Rect(x, y, this.peg, this.peg);
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SOLVED || this.state === OVER) {
        if (this.isContinue(ev)) {
          if (this.state === OVER) {
            this.gameOver = false;
            this.reset();
          } else {
            this.newCode();
            this.playSound("click");
          }
        }
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "BackSpace") this.delete();
        else if (k === "Return") this.submit();
        else if (/^[0-9]$/.test(k) && Number(k) >= 1 && Number(k) <= this.colors) this.place(Number(k) - 1);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.click(ev.pos);
      }
    }

    isContinue(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    click(pos) {
      if (this.checkRect.collidepoint(pos)) {
        this.submit();
        return;
      }
      if (this.delRect.collidepoint(pos)) {
        this.delete();
        return;
      }
      // Klick auf einen gelegten Stift der aktuellen Reihe -> löschen
      for (let i = 0; i < this.pins; i++) {
        if (this.current[i] != null && this.rowPegRect(this.row, i).collidepoint(pos)) {
          this.current[i] = null;
          this.playSound("move");
          return;
        }
      }
      for (let ci = 0; ci < this.swatchRects.length; ci++) {
        if (this.swatchRects[ci].collidepoint(pos)) {
          this.place(ci);
          return;
        }
      }
    }

    place(colorIdx) {
      const i = this.current.indexOf(null);
      if (i >= 0) this.current[i] = colorIdx;
      this.playSound("click");
    }

    delete() {
      for (let i = this.pins - 1; i >= 0; i--) {
        if (this.current[i] != null) {
          this.current[i] = null;
          this.playSound("move");
          return;
        }
      }
    }

    submit() {
      if (this.current.some((c) => c == null)) {
        this.playSound("click");
        return;
      }
      const guess = this.current.slice();
      const [black, white] = feedback(guess, this.secret);
      this.rowsData.push([guess, [black, white]]);
      this.current = new Array(this.pins).fill(null);
      this.row += 1;
      if (black === this.pins) {
        const used = this.row;
        const pts = 20 + (this.maxRows - used) * 8 + (DIFF_BONUS[this.mode] != null ? DIFF_BONUS[this.mode] : 10);
        this.lastPoints = pts;
        this.score += pts;
        this.solvedCount += 1;
        this.state = SOLVED;
        this.reportResult(true);
        this.playSound("win");
      } else if (this.row >= this.maxRows) {
        this.state = OVER;
        this.gameOver = true; // die App speichert den Highscore
        if (this.solvedCount === 0) this.reportResult(false);
        this.playSound("gameover");
      } else {
        this.playSound("move");
      }
    }

    update(dt) {}

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.drawRows(ctx);
      this.drawPalette(ctx);
      if (this.state === SOLVED) {
        this.drawBanner(ctx, t("mm.solved", { n: this.lastPoints }), this.accent, t("mm.next"), false);
      } else if (this.state === OVER) {
        this.drawBanner(ctx, t("mm.gameover"), [228, 96, 96], t("common.enter_restart"), true);
      }
    }

    drawHud(ctx) {
      ui.text(ctx, t("mm.title"), 20, 30, this.hud, this.accent, "midleft");
      ui.text(ctx, t("mm.score", { n: this.score }), this.width / 2, 22, this.small, ui.GOLD, "center");
      ui.text(ctx, t("mm.rows", { a: Math.min(this.row + 1, this.maxRows), b: this.maxRows }), this.width - 20, 22, this.small, ui.TEXT_DIM, "midright");
      ui.text(ctx, t("mm.solved_n", { n: this.solvedCount }), this.width - 20, 44, this.small, ui.TEXT_DIM, "midright");
    }

    drawRows(ctx) {
      for (let r = 0; r < this.maxRows; r++) {
        const active = r === this.row && this.state === PLAY;
        for (let i = 0; i < this.pins; i++) {
          const rc = this.rowPegRect(r, i);
          let val = null;
          if (r < this.rowsData.length) val = this.rowsData[r][0][i];
          else if (active) val = this.current[i];
          this.drawPeg(ctx, rc, val, active && val == null);
        }
        // Rückmeldungs-Pins
        if (r < this.rowsData.length) this.drawFeedback(ctx, r, this.rowsData[r][1]);
      }
    }

    drawPeg(ctx, rc, val, highlight) {
      const cx = rc.centerx, cy = rc.centery;
      const rad = Math.floor(rc.w / 2);
      if (val == null) {
        draw.circle(ctx, COL_SLOT, [cx, cy], rad);
        draw.circle(ctx, highlight ? this.accent : COL_SLOT_BORDER, [cx, cy], rad, 2);
      } else {
        const col = PALETTE[val];
        draw.circle(ctx, col, [cx, cy], rad);
        draw.circle(ctx, ui.mix(col, [255, 255, 255], 0.35), [cx, cy], rad, 2);
        draw.circle(ctx, ui.mix(col, [255, 255, 255], 0.5), [cx - Math.floor(rad / 3), cy - Math.floor(rad / 3)], Math.max(2, Math.floor(rad / 5)));
      }
    }

    drawFeedback(ctx, r, fb) {
      const [black, white] = fb;
      const pegs = [];
      for (let i = 0; i < black; i++) pegs.push(COL_PEG_BLACK);
      for (let i = 0; i < white; i++) pegs.push(COL_PEG_WHITE);
      while (pegs.length < this.pins) pegs.push(COL_PEG_EMPTY);
      const cols = this.pins <= 4 ? 2 : 3;
      const sr = Math.max(3, Math.floor(this.peg / 6));
      const step = sr * 2 + 3;
      const y0 = Math.floor(this.gridTop + r * this.pitch) + Math.floor(this.peg / 2) - step;
      pegs.forEach((pc, k) => {
        const x = this.fbX + (k % cols) * step + sr;
        const y = y0 + Math.floor(k / cols) * step;
        draw.circle(ctx, pc, [x, y], sr);
        if (pc === COL_PEG_WHITE) draw.circle(ctx, [120, 122, 130], [x, y], sr, 1);
      });
    }

    drawPalette(ctx) {
      this.swatchRects.forEach((rc, ci) => {
        const col = PALETTE[ci];
        draw.rect(ctx, col, rc, 0, 8);
        draw.rect(ctx, ui.mix(col, [255, 255, 255], 0.3), rc, 2, 8);
        ui.text(ctx, String(ci + 1), rc.centerx, rc.bottom - 9, this.tiny, [20, 22, 30], "center");
      });
      for (const [rc, label] of [[this.delRect, "<"], [this.checkRect, "OK"]]) {
        draw.rect(ctx, [60, 66, 90], rc, 0, 8);
        draw.rect(ctx, ui.BORDER_LIGHT, rc, 2, 8);
        ui.text(ctx, label, rc.centerx, rc.centery, this.tiny, ui.TEXT, "center");
      }
    }

    drawBanner(ctx, title, color, sub, reveal) {
      const w = Math.min(this.width - 40, 480);
      const h = reveal ? 150 : 108;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), Math.floor((this.height - h) / 2), w, h);
      draw.rect(ctx, [16, 18, 24, 238], rc);
      draw.rect(ctx, color, rc, 2, 14);
      ui.text(ctx, title, rc.centerx, rc.y + 36, this.huge, color, "center");
      if (reveal) {
        ui.text(ctx, t("mm.code_was"), rc.centerx, rc.y + 70, this.small, ui.TEXT_DIM, "center");
        const rad = 12;
        const tot = this.pins * (rad * 2 + 8) - 8;
        let x = rc.centerx - Math.floor(tot / 2) + rad;
        for (const val of this.secret) {
          draw.circle(ctx, PALETTE[val], [x, rc.y + 98], rad);
          draw.circle(ctx, ui.mix(PALETTE[val], [255, 255, 255], 0.35), [x, rc.y + 98], rad, 2);
          x += rad * 2 + 8;
        }
        ui.text(ctx, sub, rc.centerx, rc.y + 128, this.small, ui.TEXT_DIM, "center");
      } else {
        ui.text(ctx, sub, rc.centerx, rc.y + 78, this.small, ui.TEXT_DIM, "center");
      }
    }
  }

  PG.register(MastermindGame, {
    id: "MastermindGame",
    key: "mastermind",
    name: "Mastermind",
    modes: [["easy", "mm.mode.easy"], ["classic", "mm.mode.classic"], ["hard", "mm.mode.hard"]],
  });
})();
