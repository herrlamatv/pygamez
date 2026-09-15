/*
 * kniffel.js - Kniffel / Yahtzee (Port von games/kniffel.py)
 * ============================================================
 * Regeln:
 * - 5 Würfel, bis zu 3 Würfe pro Zug. Zwischen den Würfen dürfen beliebig
 *   viele Würfel "gehalten" werden (Klick/Taste); nur die übrigen werden neu
 *   geworfen.
 * - Danach wird genau EINE der 13 Kategorien gebucht (auch mit 0 = Streichen).
 *   Nach 13 Buchungen ist der Block voll.
 * - Oberer Block (Einser..Sechser): bei >= 63 Punkten gibt es 35 Bonuspunkte.
 * - Unterer Block: Dreier-/Vierer-Pasch (Augensumme), Full House (25), kleine
 *   Straße (30), große Straße (40), Kniffel (50), Chance (Augensumme).
 * - Web-Version: nur Einzelspieler (Highscore-Jagd, Endsumme = Highscore).
 *
 * Steuerung: Maus - "Würfeln" klicken, Würfel anklicken = halten, Kategorie in
 * der Liste anklicken = buchen. Tasten: Leertaste = würfeln, 1-5 = Würfel halten/
 * lösen, Pfeile = Kategorie wählen, Enter = buchen. Nach Ende: Enter = neu.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const COL_PANEL = [28, 28, 38];
  const COL_PANEL_HI = [40, 40, 54];
  const COL_TEXT = [228, 230, 238];
  const COL_DIM = [150, 154, 170];
  const COL_FAINT = [110, 114, 130];
  const COL_ACCENT = [232, 176, 75]; // = Sidebar-Farbe #e8b04b
  const COL_GOOD = [110, 205, 140];
  const COL_DIE = [240, 242, 248];
  const COL_DIE_HELD = [250, 226, 160];
  const COL_PIP = [36, 38, 48];
  const COL_BTN = [44, 44, 58];
  const COL_BTN_ON = [92, 70, 34];
  const COL_BTN_BORDER = [78, 78, 100];

  // Kategorien in Anzeigereihenfolge (Key = i18n-Suffix)
  const CATS = ["ones", "twos", "threes", "fours", "fives", "sixes", "three_kind", "four_kind", "full_house", "small_straight", "large_straight", "kniffel", "chance"];
  const UPPER_KEYS = { ones: 1, twos: 2, threes: 3, fours: 4, fives: 5, sixes: 6 };
  const UPPER_BONUS = 35;
  const UPPER_TARGET = 63;

  const READY = "ready", ROLL_ANIM = "roll_anim", ROLLED = "rolled", OVER = "over";

  /** Punktwert der Kategorie 'key' für die 5 Würfel 'dice' (Liste 1..6). */
  function scoreCategory(key, dice) {
    const counts = [1, 2, 3, 4, 5, 6].map((v) => dice.filter((d) => d === v).length);
    const total = dice.reduce((a, b) => a + b, 0);
    const maxC = Math.max(...counts);
    const faces = new Set(dice);
    const hasAll = (run) => run.every((v) => faces.has(v));
    if (key in UPPER_KEYS) {
      const face = UPPER_KEYS[key];
      return counts[face - 1] * face;
    }
    switch (key) {
      case "three_kind": return maxC >= 3 ? total : 0;
      case "four_kind": return maxC >= 4 ? total : 0;
      case "full_house": {
        const nz = counts.filter((c) => c).sort((a, b) => a - b);
        return nz.length === 2 && nz[0] === 2 && nz[1] === 3 ? 25 : 0;
      }
      case "small_straight":
        return [[1, 2, 3, 4], [2, 3, 4, 5], [3, 4, 5, 6]].some(hasAll) ? 30 : 0;
      case "large_straight":
        return faces.size === 5 && (hasAll([1, 2, 3, 4, 5]) || hasAll([2, 3, 4, 5, 6])) ? 40 : 0;
      case "kniffel": return maxC >= 5 ? 50 : 0;
      case "chance": return total;
    }
    return 0;
  }

  function emptyCard() {
    const c = {};
    for (const k of CATS) c[k] = null;
    return c;
  }

  function upperSum(card) {
    let s = 0;
    for (const k in UPPER_KEYS) if (card[k] != null) s += card[k];
    return s;
  }

  function grandTotal(card) {
    const up = upperSum(card);
    const bonus = up >= UPPER_TARGET ? UPPER_BONUS : 0;
    let low = 0;
    for (const k of CATS) if (!(k in UPPER_KEYS) && card[k] != null) low += card[k];
    return up + bonus + low;
  }

  class KniffelGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.card = emptyCard();
      this.dice = [1, 1, 1, 1, 1];
      this.held = [false, false, false, false, false];
      this.rollsLeft = 3;
      this.rolledOnce = false;
      this.animT = 0;
      this.final = this.dice.slice();
      this.animFaces = this.dice.slice();
      this.animTick = 0;
      this.sel = 0;
      this.makeFonts();
      this.layout();
      this.startTurn();
    }

    makeFonts() {
      this.small = ui.font(15);
      this.tiny = ui.font(12);
      this.rowFont = ui.font(14, false, true);
      this.big = ui.font(20, true);
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 12)), true);
    }

    layout() {
      this.hudH = 42;
      this.diceH = 92;
      this.diceY = this.height - this.diceH;
      // Kartenbereich zwischen HUD und Würfelleiste
      const top = this.hudH + 6;
      const bottom = this.diceY - 6;
      this.nRows = CATS.length + 2; // + Oberer-Bonus-Zeile + Gesamt
      this.rowH = Math.max(16, Math.floor((bottom - top) / this.nRows));
      this.cardTop = top;
      // Namensspalte links, Wertspalte rechts
      this.nameX = 16;
      this.colW = 62;
      this.valX = this.width - 20 - this.colW;
      // Zeilen-Trefferflächen (nur die 13 Kategorien)
      this.rowRects = CATS.map((_, i) => new PG.Rect(this.nameX - 6, this.cardTop + i * this.rowH, this.width - this.nameX - 8, this.rowH));
      // Würfel + Würfeln-Button
      this.die = Math.floor(Math.min(this.diceH - 40, (this.width - 200) / 5));
      this.die = Math.max(34, this.die);
      const gap = 12;
      const totalW = 5 * this.die + 4 * gap;
      const x0 = Math.max(14, Math.floor((this.width - 160 - totalW) / 2));
      this.dieRects = [];
      for (let i = 0; i < 5; i++) {
        this.dieRects.push(new PG.Rect(x0 + i * (this.die + gap), this.diceY + Math.floor((this.diceH - this.die) / 2), this.die, this.die));
      }
      const bw = 130;
      this.rollRect = new PG.Rect(this.width - bw - 16, this.diceY + Math.floor((this.diceH - 44) / 2), bw, 44);
    }

    startTurn() {
      this.rollsLeft = 3;
      this.held = [false, false, false, false, false];
      this.rolledOnce = false;
      this.state = READY;
      // Auswahl auf erste offene Kategorie setzen
      const i = CATS.findIndex((k) => this.card[k] == null);
      if (i >= 0) this.sel = i;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === OVER) {
        if (ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"))) {
          this.gameOver = false;
          this.reset();
        }
        return;
      }
      if (ev.kind === "keydown") this.handleKey(ev.key);
      else if (ev.kind === "mousedown") this.handleClick(ev.pos);
    }

    handleKey(k) {
      if (k === "space") {
        this.roll();
      } else if (["1", "2", "3", "4", "5"].includes(k)) {
        this.toggleHold(Number(k) - 1);
      } else if (k === "Up" || k === "w" || k === "W") {
        this.sel = PG.mod(this.sel - 1, CATS.length);
        this.playSound("move");
      } else if (k === "Down" || k === "s" || k === "S") {
        this.sel = PG.mod(this.sel + 1, CATS.length);
        this.playSound("move");
      } else if (k === "Return") {
        this.book(this.sel);
      }
    }

    handleClick(pos) {
      if (this.rollRect.collidepoint(pos)) {
        this.roll();
        return;
      }
      for (let i = 0; i < 5; i++) {
        if (this.dieRects[i].collidepoint(pos)) {
          this.toggleHold(i);
          return;
        }
      }
      for (let i = 0; i < this.rowRects.length; i++) {
        if (this.rowRects[i].collidepoint(pos)) {
          this.sel = i;
          this.book(i);
          return;
        }
      }
    }

    toggleHold(i) {
      // Halten ergibt nur nach dem ersten Wurf und vor dem letzten Sinn.
      if (!this.rolledOnce || this.state === ROLL_ANIM) return;
      this.held[i] = !this.held[i];
      this.playSound("click");
    }

    roll() {
      if (this.state === ROLL_ANIM || this.rollsLeft <= 0) return;
      this.final = this.dice.map((d, i) => (this.rolledOnce && this.held[i] ? d : PG.rand.randint(1, 6)));
      this.state = ROLL_ANIM;
      this.animT = 0.5;
      this.playSound("move");
    }

    book(catI) {
      if (!this.rolledOnce || this.state === ROLL_ANIM) return;
      const key = CATS[catI];
      if (this.card[key] != null) {
        this.playSound("click");
        return;
      }
      this.card[key] = scoreCategory(key, this.dice);
      if (key === "kniffel" && this.card[key] > 0) this.achEvent("kniffel_five");
      this.playSound(this.card[key] > 0 ? "point" : "select");
      this.score = grandTotal(this.card);
      this.nextTurn();
    }

    nextTurn() {
      if (CATS.every((k) => this.card[k] != null)) {
        this.ende();
        return;
      }
      this.startTurn();
    }

    ende() {
      this.state = OVER;
      this.gameOver = true;
      this.score = grandTotal(this.card);
      this.playSound("win");
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === ROLL_ANIM) {
        this.animT -= dt;
        if (this.animT <= 0) {
          this.dice = this.final.slice();
          this.rollsLeft -= 1;
          this.rolledOnce = true;
          this.state = ROLLED;
          this.playSound("lock");
        }
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Themen-Hintergrund statt flacher Fläche (reagiert auf Theme-Wechsel).
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.drawCard(ctx);
      this.drawDice(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawHud(ctx) {
      draw.rect(ctx, COL_PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, COL_BTN_BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = this.hudH / 2;
      ui.text(ctx, t("kn.total_now", { n: grandTotal(this.card) }), 14, cy, this.big, COL_ACCENT, "midleft");
      // Würfe-Anzeige rechts
      if (this.state !== OVER) {
        ui.text(ctx, t("kn.rolls_left", { n: Math.max(0, this.rollsLeft) }), this.width - 14, cy, this.small, COL_DIM, "midright");
      }
    }

    drawCard(ctx) {
      const card = this.card;
      const cx = this.valX + this.colW / 2;
      for (let i = 0; i < CATS.length; i++) {
        const key = CATS[i];
        const y = this.cardTop + i * this.rowH;
        const rc = this.rowRects[i];
        const selected = i === this.sel && this.state !== OVER;
        if (selected) {
          draw.rect(ctx, COL_PANEL_HI, rc, 0, 5);
          draw.rect(ctx, COL_ACCENT, [rc.x, rc.y + 2, 3, rc.h - 4]);
        }
        const my = y + Math.floor(this.rowH / 2);
        ui.text(ctx, t("kn." + key), this.nameX, my, this.rowFont, selected ? COL_TEXT : COL_DIM, "midleft");
        if (card[key] != null) {
          ui.text(ctx, String(card[key]), cx, my, this.rowFont, COL_TEXT, "center");
        } else if (this.rolledOnce && this.state !== OVER) {
          // Vorschau des möglichen Werts (gedimmt)
          const val = scoreCategory(key, this.dice);
          ui.text(ctx, String(val), cx, my, this.rowFont, val > 0 ? COL_GOOD : COL_FAINT, "center");
        } else {
          ui.text(ctx, "-", cx, my, this.rowFont, COL_FAINT, "center");
        }
      }

      // Oberer-Bonus-Zeile
      let y = this.cardTop + CATS.length * this.rowH;
      ui.text(ctx, t("kn.upper_bonus"), this.nameX, y + Math.floor(this.rowH / 2), this.rowFont, COL_DIM, "midleft");
      const up = upperSum(card);
      const bonus = up >= UPPER_TARGET ? UPPER_BONUS : 0;
      const txt = up + "/" + UPPER_TARGET + (bonus ? " +" + bonus : "");
      ui.text(ctx, txt, cx, y + Math.floor(this.rowH / 2), this.tiny, bonus ? COL_GOOD : COL_FAINT, "center");

      // Gesamtsumme
      y += this.rowH;
      draw.line(ctx, COL_BTN_BORDER, [this.nameX - 6, y], [this.width - 8, y]);
      ui.text(ctx, t("kn.total"), this.nameX, y + Math.floor(this.rowH / 2) + 2, this.big, COL_TEXT, "midleft");
      ui.text(ctx, String(grandTotal(card)), cx, y + Math.floor(this.rowH / 2) + 2, this.big, COL_ACCENT, "center");
    }

    drawDice(ctx) {
      draw.rect(ctx, COL_PANEL, [0, this.diceY, this.width, this.diceH]);
      draw.line(ctx, COL_BTN_BORDER, [0, this.diceY], [this.width, this.diceY]);
      const showAnim = this.state === ROLL_ANIM;
      // Zufallsaugen während der Animation (wie random.randint pro Frame)
      if (showAnim) this.animFaces = this.animFaces.map(() => PG.rand.randint(1, 6));
      for (let i = 0; i < 5; i++) {
        let val;
        if (!this.rolledOnce && this.state === READY) val = 0;
        else if (showAnim && !(this.rolledOnce && this.held[i])) val = this.animFaces[i];
        else val = this.dice[i];
        const held = this.held[i] && this.rolledOnce;
        this.drawDie(ctx, this.dieRects[i], val, held, i);
      }

      // Würfeln-Button
      const canRoll = this.rollsLeft > 0 && (this.state === READY || this.state === ROLLED);
      draw.rect(ctx, canRoll ? COL_BTN_ON : COL_BTN, this.rollRect, 0, 10);
      draw.rect(ctx, canRoll ? COL_ACCENT : COL_BTN_BORDER, this.rollRect, canRoll ? 2 : 1, 10);
      const label = this.state !== READY || this.rolledOnce ? t("kn.roll") : t("kn.roll_start");
      ui.text(ctx, label, this.rollRect.centerx, this.rollRect.centery, this.small, canRoll ? COL_TEXT : COL_FAINT, "center");
    }

    drawDie(ctx, rc, val, held, idx) {
      const base = held ? COL_DIE_HELD : COL_DIE;
      draw.rect(ctx, [10, 12, 18], rc.move(0, 3), 0, 8);
      draw.rect(ctx, base, rc, 0, 8);
      if (held) draw.rect(ctx, COL_ACCENT, rc, 3, 8);
      else draw.rect(ctx, [200, 202, 210], rc, 1, 8);
      // Würfel-Nummer klein oben links (Halt-Taste)
      ui.text(ctx, String(idx + 1), rc.x + 4, rc.y + 2, this.tiny, [170, 172, 180]);
      if (val < 1) return;
      // Augen (Pips)
      const r = Math.max(3, Math.floor(this.die / 10));
      const cx = rc.centerx, cy = rc.centery;
      const off = Math.floor(this.die / 4);
      const L = cx - off, C = cx, R = cx + off;
      const T = cy - off, M = cy, B = cy + off;
      const pips = {
        1: [[C, M]],
        2: [[L, T], [R, B]],
        3: [[L, T], [C, M], [R, B]],
        4: [[L, T], [R, T], [L, B], [R, B]],
        5: [[L, T], [R, T], [C, M], [L, B], [R, B]],
        6: [[L, T], [R, T], [L, M], [R, M], [L, B], [R, B]],
      };
      for (const p of pips[val] || []) draw.circle(ctx, COL_PIP, p, r);
    }

    drawOver(ctx) {
      draw.rect(ctx, [10, 12, 18, 180], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      ui.text(ctx, t("kn.finished"), cx, cy - 40, this.huge, COL_ACCENT, "center");
      ui.text(ctx, t("kn.your_score", { n: grandTotal(this.card) }), cx, cy + 2, this.big, COL_TEXT, "center");
      ui.text(ctx, t("common.enter_restart"), cx, cy + 44, this.tiny, COL_DIM, "center");
    }
  }

  PG.register(KniffelGame, {
    id: "KniffelGame",
    key: "kniffel",
    name: "Kniffel",
  });
})();
