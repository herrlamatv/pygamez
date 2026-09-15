/*
 * solitaire.js - Solitär (Port von games/solitaire.py)
 * =====================================================
 * Fünf Varianten unter einem Dach (Auswahl im Vorspiel-Screen):
 *
 * - KLONDIKE : das klassische Solitär; Ziehen von 1 oder 3 Karten (Option).
 * - SPIDER   : 10 Spalten, 104 Karten; 1/2/4 Farben (Option); K->A-Ketten
 *              gleicher Farbe wandern ins Fundament.
 * - FREECELL : alles offen, 4 freie Zellen; Supermove-Limit
 *              (frei+1) * 2^leere Spalten (Ziel leer -> halbiert).
 * - PYRAMID  : Paare mit Summe 13 abtragen (König allein); 2 Redeals.
 * - TRIPEAKS : Waste-Kette mit +/-1 (A<->K wrappt); Combo-Multiplikator.
 *
 * Bedienung: Karten mit der Maus ZIEHEN (Drag & Drop) oder per Klick-Klick;
 * Rechtsklick schickt die oberste Karte aufs Fundament (Klondike/FreeCell);
 * Leertaste/Enter = Stock, U = Undo (unbegrenzt), R = neues Blatt, S = Setup.
 *
 * Alle Kartengrafiken kommen aus games/cards.js (Canvas-Primitiven).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const C = PG.cards;

  // Tisch-Identität (Filz). Generische UI-Farben kommen zur Zeichenzeit aus
  // ui.* (Theme), die Akzentfarbe aus this.accent.
  const COL_FELT = [20, 44, 35];
  const COL_FELT_EDGE = [13, 31, 24];

  const SETUP = "setup", PLAY = "play";

  const DRAG_PX = 6; // ab dieser Bewegung ist es ein "echter" Drag
  const CLICK_S = 0.35; // kürzer = Klick (für Klick-Klick-Bedienung)

  const nowSec = () => performance.now() / 1000;

  /** Ein Kartenstapel mit Position und Auffächerungs-Abständen. */
  class Pile {
    constructor(kind, meta = {}) {
      this.kind = kind; // stock/waste/foundation/tableau/cell/grid/removed
      this.cards = [];
      this.x = 0;
      this.y = 0;
      this.dyDown = 0;
      this.dyUp = 0;
      this.meta = meta;
    }
    rects(w, h) {
      return C.fanRects(this.x, this.y, this.cards, w, h, this.dyDown, this.dyUp);
    }
    get top() {
      return this.cards.length ? this.cards[this.cards.length - 1] : null;
    }
  }

  // ---------------------------------------------------------------------------
  //  Varianten
  // ---------------------------------------------------------------------------

  /** Basis: Regeln einer Solitär-Variante. this.g = SolitaireGame. */
  class Variant {
    constructor(game) {
      this.g = game;
    }

    // Pflicht-Hooks
    deal(rng) {}
    layout(w, h) {}
    isWon() {
      return false;
    }
    winBonus() {
      return 0;
    }

    // Interaktion (Standard: nichts erlaubt)
    canGrab(pi, ci) {
      return false;
    }
    /** Versucht abzulegen; bei Erfolg Record zurückgeben, sonst null.
     *  Die Karten sind bereits aus der Quelle entnommen. */
    tryDrop(cards, srcI, dstI) {
      return null;
    }
    onStock() {
      return null;
    }
    /** true, wenn ein Klick auf den leeren Stock die Waste zurücklegt. */
    canRecycle() {
      return false;
    }
    onRight(pi) {
      return null;
    }
    onClickCard(pi, ci) {
      return null;
    }
    undoExtra(extra) {}
    setupRows() {
      return []; // [[labelFn, toggleFn]]
    }
    hudExtra() {
      return [];
    }

    // ----- gemeinsame Helfer ------------------------------------------------

    /** Deckt die oberste verdeckte Karte auf (und protokolliert es). */
    _flipTop(pi, record, score = 0) {
      const p = this.g.piles[pi];
      if (p.cards.length && !p.top.faceUp) {
        p.top.faceUp = true;
        record.flips.push([pi, p.cards.length - 1]);
        record.score += score;
      }
    }

    _foundationOk(card, pile) {
      if (pile.cards.length) return card.suit === pile.top.suit && card.rank === pile.top.rank + 1;
      return card.rank === 1;
    }

    _altDescOk(card, pile) {
      if (pile.cards.length) {
        const top = pile.top;
        return top.faceUp && card.red !== top.red && card.rank === top.rank - 1;
      }
      return true;
    }
  }

  class Klondike extends Variant {
    get key() {
      return "klondike";
    }
    // Stapel-Indizes: 0 Stock, 1 Waste, 2-5 Fundament, 6-12 Tableau

    deal(rng) {
      const g = this.g;
      g.piles = [new Pile("stock"), new Pile("waste")];
      for (let i = 0; i < 4; i++) g.piles.push(new Pile("foundation"));
      for (let i = 0; i < 7; i++) g.piles.push(new Pile("tableau"));
      const deck = C.makeDeck();
      C.shuffle(deck, rng);
      for (let i = 0; i < 7; i++) {
        for (let j = 0; j <= i; j++) {
          const card = deck.pop();
          card.faceUp = j === i;
          g.piles[6 + i].cards.push(card);
        }
      }
      g.piles[0].cards = deck; // Rest verdeckt in den Stock
      this.draw3 = !!g.settingsGet("draw3", false);
      this.recycles = 0;
    }

    layout(w, h) {
      const g = this.g;
      g.cw = Math.max(30, Math.floor(w / 9));
      g.ch = Math.floor(g.cw * 1.4);
      const m = Math.max(6, Math.floor(w / 80));
      const top = g.hudH + m;
      g.piles[0].x = m;
      g.piles[0].y = top;
      g.piles[1].x = m + g.cw + m;
      g.piles[1].y = top;
      for (let i = 0; i < 4; i++) {
        g.piles[2 + i].x = w - m - (4 - i) * (g.cw + m) + m;
        g.piles[2 + i].y = top;
      }
      const ty = top + g.ch + 2 * m;
      const span = 7 * g.cw + 6 * m;
      const x0 = Math.floor((w - span) / 2);
      for (let i = 0; i < 7; i++) {
        const p = g.piles[6 + i];
        p.x = x0 + i * (g.cw + m);
        p.y = ty;
        p.dyDown = Math.floor(g.ch * 0.12);
        p.dyUp = Math.floor(g.ch * 0.28);
      }
    }

    canGrab(pi, ci) {
      const p = this.g.piles[pi];
      if (p.kind === "waste") return ci === p.cards.length - 1;
      if (p.kind === "foundation") return ci === p.cards.length - 1;
      if (p.kind === "tableau") return p.cards[ci].faceUp;
      return false;
    }

    tryDrop(cards, srcI, dstI) {
      const g = this.g;
      const dst = g.piles[dstI];
      const src = g.piles[srcI];
      if (dst.kind === "foundation") {
        if (cards.length === 1 && this._foundationOk(cards[0], dst)) {
          const rec = g.commit(cards, srcI, dstI, 10);
          this._flipTop(srcI, rec, 5);
          return rec;
        }
      } else if (dst.kind === "tableau") {
        const first = cards[0];
        const ok = !dst.cards.length ? first.rank === 13 : this._altDescOk(first, dst);
        if (ok) {
          const delta = src.kind === "foundation" ? -10 : 0;
          const rec = g.commit(cards, srcI, dstI, delta);
          this._flipTop(srcI, rec, 5);
          return rec;
        }
      }
      return null;
    }

    onStock() {
      const g = this.g;
      const stock = g.piles[0], waste = g.piles[1];
      if (stock.cards.length) {
        const n = Math.min(this.draw3 ? 3 : 1, stock.cards.length);
        const rec = g.newRecord();
        for (let i = 0; i < n; i++) {
          const card = stock.cards.pop();
          card.faceUp = true;
          waste.cards.push(card);
        }
        // Je Karte ein eigener 1er-Op: Das Ziehen dreht die Reihenfolge um - ein
        // gemeinsamer n-Op legte die Karten beim Undo verkehrt herum in den
        // Stock zurück (3er-Zug: danach kamen andere Karten).
        for (let i = 0; i < n; i++) rec.ops.push([0, 1, 1]);
        rec.flips = [];
        for (let i = 0; i < n; i++) rec.flips.push([1, waste.cards.length - 1 - i]);
        return rec;
      }
      if (waste.cards.length) {
        // Recycle: Waste komplett zurück in den Stock (verdeckt).
        const n = waste.cards.length;
        for (let i = waste.cards.length - 1; i >= 0; i--) {
          const card = waste.cards[i];
          card.faceUp = false;
          stock.cards.push(card);
        }
        waste.cards.length = 0;
        this.recycles += 1;
        const rec = g.newRecord();
        rec.score = -20;
        rec.extra = { recycle: n };
        return rec;
      }
      return null;
    }

    canRecycle() {
      return this.g.piles[1].cards.length > 0 && !this.g.piles[0].cards.length;
    }

    undoExtra(extra) {
      if ("recycle" in extra) {
        const g = this.g;
        const stock = g.piles[0], waste = g.piles[1];
        const moved = stock.cards.splice(stock.cards.length - extra.recycle);
        for (let i = moved.length - 1; i >= 0; i--) {
          moved[i].faceUp = true;
          waste.cards.push(moved[i]);
        }
        this.recycles -= 1;
      }
    }

    onRight(pi) {
      const g = this.g;
      const p = g.piles[pi];
      if ((p.kind !== "waste" && p.kind !== "tableau") || !p.cards.length || !p.top.faceUp) return null;
      const card = p.top;
      for (let fi = 2; fi < 6; fi++) {
        if (this._foundationOk(card, g.piles[fi])) {
          p.cards.pop();
          return this.tryDrop([card], pi, fi);
        }
      }
      return null;
    }

    isWon() {
      for (let i = 2; i < 6; i++) if (this.g.piles[i].cards.length !== 13) return false;
      return true;
    }

    winBonus() {
      return Math.max(0, 1000 - 2 * Math.floor(this.g.elapsed));
    }

    setupRows() {
      return [[
        () => t("sol.draw3") + ":  " + (this.g.settingsGet("draw3", false) ? t("common.on") : t("common.off")),
        () => this.g.settingsToggle("draw3"),
      ]];
    }
  }

  class Spider extends Variant {
    get key() {
      return "spider";
    }
    // 0 Stock, 1-10 Tableau, 11-18 Fundament-Slots

    deal(rng) {
      const g = this.g;
      g.piles = [new Pile("stock")];
      for (let i = 0; i < 10; i++) g.piles.push(new Pile("tableau"));
      for (let i = 0; i < 8; i++) g.piles.push(new Pile("foundation"));
      const suitsN = Number(g.settingsGet("spider_suits", 1));
      let deck;
      if (suitsN === 1) deck = C.makeDeck([0], 8);
      else if (suitsN === 2) deck = C.makeDeck([0, 1], 4);
      else deck = C.makeDeck(undefined, 2);
      C.shuffle(deck, rng);
      for (let i = 0; i < 10; i++) {
        const n = i < 4 ? 6 : 5;
        for (let j = 0; j < n; j++) {
          const card = deck.pop();
          card.faceUp = j === n - 1;
          g.piles[1 + i].cards.push(card);
        }
      }
      g.piles[0].cards = deck; // 50 Karten = 5 Nachschübe
      this.sequences = 0;
      this.moves = 0;
    }

    layout(w, h) {
      const g = this.g;
      const m = Math.max(4, Math.floor(w / 120));
      g.cw = Math.max(26, Math.floor((w - 11 * m) / 10));
      g.ch = Math.floor(g.cw * 1.4);
      const top = g.hudH + m;
      g.piles[0].x = m;
      g.piles[0].y = top;
      for (let i = 0; i < 8; i++) {
        g.piles[11 + i].x = w - m - (8 - i) * (Math.floor(g.cw / 2) + m) + m;
        g.piles[11 + i].y = top;
      }
      const ty = top + g.ch + 2 * m;
      for (let i = 0; i < 10; i++) {
        const p = g.piles[1 + i];
        p.x = m + i * (g.cw + m);
        p.y = ty;
        p.dyDown = Math.floor(g.ch * 0.12);
        p.dyUp = Math.floor(g.ch * 0.24);
      }
    }

    canGrab(pi, ci) {
      const p = this.g.piles[pi];
      if (p.kind !== "tableau") return false;
      const run = p.cards.slice(ci);
      if (!run.length || !run[0].faceUp) return false;
      for (let i = 1; i < run.length; i++) {
        const a = run[i - 1], b = run[i];
        if (!(b.faceUp && b.suit === a.suit && b.rank === a.rank - 1)) return false;
      }
      return true;
    }

    tryDrop(cards, srcI, dstI) {
      const g = this.g;
      const dst = g.piles[dstI];
      if (dst.kind !== "tableau") return null;
      if (dst.cards.length && !(dst.top.faceUp && cards[0].rank === dst.top.rank - 1)) return null;
      this.moves += 1;
      const rec = g.commit(cards, srcI, dstI, -1);
      rec.extra.move = true;
      this._flipTop(srcI, rec, 0);
      this._checkSequence(dstI, rec);
      return rec;
    }

    /** Vollständige K->A-Kette gleicher Farbe am Ende -> ins Fundament. */
    _checkSequence(pi, rec) {
      const g = this.g;
      const p = g.piles[pi];
      if (p.cards.length < 13) return;
      const run = p.cards.slice(-13);
      if (run[0].rank !== 13 || !run.every((c) => c.faceUp)) return;
      for (let i = 1; i < run.length; i++) {
        if (run[i].suit !== run[i - 1].suit || run[i].rank !== run[i - 1].rank - 1) return;
      }
      let target = -1;
      for (let i = 11; i < 19; i++) {
        if (!g.piles[i].cards.length) {
          target = i;
          break;
        }
      }
      if (target < 0) return;
      const moved = p.cards.splice(p.cards.length - 13);
      g.piles[target].cards.push(...moved);
      rec.ops.push([pi, target, 13]);
      rec.score += 100;
      this.sequences += 1;
      rec.extra.sequence = true;
      this._flipTop(pi, rec);
      g.playSound("line");
    }

    undoExtra(extra) {
      if (extra.sequence) this.sequences -= 1;
      // extra.deal: ops decken das Zurücklegen ab. Hat der Nachschub eine Folge
      // vervollständigt, passt der Flip-Index der ausgeteilten Karte nicht mehr
      // und sie käme offen in den Stock zurück -> Stock immer verdeckt.
      if (extra.deal) for (const c of this.g.piles[0].cards) c.faceUp = false;
      if (extra.move) this.moves -= 1;
    }

    onStock() {
      const g = this.g;
      const stock = g.piles[0];
      if (!stock.cards.length) return null;
      for (let i = 0; i < 10; i++) {
        if (!g.piles[1 + i].cards.length) {
          g.flash(t("sol.no_empty"));
          return null;
        }
      }
      const rec = g.newRecord();
      for (let i = 0; i < 10; i++) {
        const card = stock.cards.pop();
        card.faceUp = true;
        g.piles[1 + i].cards.push(card);
        rec.ops.push([0, 1 + i, 1]);
        rec.flips.push([1 + i, g.piles[1 + i].cards.length - 1]);
      }
      rec.extra.deal = true;
      for (let i = 0; i < 10; i++) this._checkSequence(1 + i, rec);
      return rec;
    }

    isWon() {
      return this.sequences >= 8;
    }

    winBonus() {
      return 0;
    }

    scoreNow() {
      return Math.max(0, 500 - this.moves + 100 * this.sequences);
    }

    setupRows() {
      return [[
        () => t("sol.suits", { n: this.g.settingsGet("spider_suits", 1) }),
        () => this.g.settingsCycle("spider_suits", [1, 2, 4]),
      ]];
    }

    hudExtra() {
      return [t("sol.deals_left", { n: Math.floor(this.g.piles[0].cards.length / 10) })];
    }
  }

  class FreeCell extends Variant {
    get key() {
      return "freecell";
    }
    // 0-3 Zellen, 4-7 Fundament, 8-15 Tableau

    deal(rng) {
      const g = this.g;
      g.piles = [];
      for (let i = 0; i < 4; i++) g.piles.push(new Pile("cell"));
      for (let i = 0; i < 4; i++) g.piles.push(new Pile("foundation"));
      for (let i = 0; i < 8; i++) g.piles.push(new Pile("tableau"));
      const deck = C.makeDeck();
      C.shuffle(deck, rng);
      for (const card of deck) card.faceUp = true;
      deck.forEach((card, i) => g.piles[8 + (i % 8)].cards.push(card));
    }

    layout(w, h) {
      const g = this.g;
      const m = Math.max(5, Math.floor(w / 100));
      g.cw = Math.max(28, Math.floor((w - 9 * m) / 8) - 2);
      g.ch = Math.floor(g.cw * 1.4);
      const top = g.hudH + m;
      for (let i = 0; i < 4; i++) {
        g.piles[i].x = m + i * (g.cw + m);
        g.piles[i].y = top;
      }
      for (let i = 0; i < 4; i++) {
        g.piles[4 + i].x = w - m - (4 - i) * (g.cw + m) + m;
        g.piles[4 + i].y = top;
      }
      const ty = top + g.ch + 2 * m;
      const span = 8 * g.cw + 7 * m;
      const x0 = Math.floor((w - span) / 2);
      for (let i = 0; i < 8; i++) {
        const p = g.piles[8 + i];
        p.x = x0 + i * (g.cw + m);
        p.y = ty;
        p.dyDown = Math.floor(g.ch * 0.24);
        p.dyUp = Math.floor(g.ch * 0.24);
      }
    }

    _limit(targetEmpty) {
      let free = 0, empty = 0;
      for (let i = 0; i < 4; i++) if (!this.g.piles[i].cards.length) free++;
      for (let i = 8; i < 16; i++) if (!this.g.piles[i].cards.length) empty++;
      if (targetEmpty && empty > 0) empty -= 1;
      return (free + 1) * 2 ** empty;
    }

    canGrab(pi, ci) {
      const p = this.g.piles[pi];
      if (p.kind === "cell") return p.cards.length > 0;
      if (p.kind === "foundation") return false;
      const run = p.cards.slice(ci);
      for (let i = 1; i < run.length; i++) {
        const a = run[i - 1], b = run[i];
        if (!(b.red !== a.red && b.rank === a.rank - 1)) return false;
      }
      return run.length <= this._limit(false);
    }

    tryDrop(cards, srcI, dstI) {
      const g = this.g;
      const dst = g.piles[dstI];
      if (dst.kind === "cell") {
        if (cards.length === 1 && !dst.cards.length) return g.commit(cards, srcI, dstI);
      } else if (dst.kind === "foundation") {
        if (cards.length === 1 && this._foundationOk(cards[0], dst)) return g.commit(cards, srcI, dstI, 10);
      } else if (dst.kind === "tableau") {
        if (cards.length > this._limit(!dst.cards.length)) return null;
        if (!dst.cards.length || this._altDescOk(cards[0], dst)) return g.commit(cards, srcI, dstI);
      }
      return null;
    }

    onRight(pi) {
      const g = this.g;
      const p = g.piles[pi];
      if ((p.kind !== "tableau" && p.kind !== "cell") || !p.cards.length) return null;
      const card = p.top;
      for (let fi = 4; fi < 8; fi++) {
        if (this._foundationOk(card, g.piles[fi])) {
          p.cards.pop();
          return this.tryDrop([card], pi, fi);
        }
      }
      return null;
    }

    isWon() {
      for (let i = 4; i < 8; i++) if (this.g.piles[i].cards.length !== 13) return false;
      return true;
    }

    winBonus() {
      return Math.max(0, 500 - Math.floor(this.g.elapsed));
    }
  }

  class Pyramid extends Variant {
    get key() {
      return "pyramid";
    }
    // 0 Stock, 1 Waste, 2..29 Pyramide (Einzelkarten), 30 "removed"

    deal(rng) {
      const g = this.g;
      g.piles = [new Pile("stock"), new Pile("waste")];
      this.children = {};
      for (let row = 0; row < 7; row++) {
        for (let col = 0; col <= row; col++) g.piles.push(new Pile("grid", { row, col }));
      }
      g.piles.push(new Pile("removed"));
      // Abdeck-Beziehungen: Karte (r, c) wird von (r+1, c) und (r+1, c+1) verdeckt.
      const pidx = (r, c) => 2 + (r * (r + 1)) / 2 + c;
      for (let row = 0; row < 6; row++) {
        for (let col = 0; col <= row; col++) {
          this.children[pidx(row, col)] = [pidx(row + 1, col), pidx(row + 1, col + 1)];
        }
      }
      const deck = C.makeDeck();
      C.shuffle(deck, rng);
      for (let i = 0; i < 28; i++) {
        const card = deck.pop();
        card.faceUp = true;
        g.piles[2 + i].cards.push(card);
      }
      g.piles[0].cards = deck;
      this.redeals = 2;
    }

    layout(w, h) {
      const g = this.g;
      g.cw = Math.max(26, Math.floor(w / 9.5));
      g.ch = Math.floor(g.cw * 1.4);
      const top = g.hudH + 6;
      for (let i = 0; i < 28; i++) {
        const p = g.piles[2 + i];
        const row = p.meta.row, col = p.meta.col;
        p.x = Math.floor(w / 2) + Math.trunc((col - row / 2 - 0.5) * g.cw * 1.08) + Math.floor(g.cw / 12);
        p.y = top + Math.floor(row * g.ch * 0.45);
      }
      const m = Math.max(6, Math.floor(w / 60));
      g.piles[0].x = m;
      g.piles[0].y = h - g.ch - m;
      g.piles[1].x = m + g.cw + m;
      g.piles[1].y = h - g.ch - m;
      g.piles[30].x = -1000; // unsichtbar
      g.piles[30].y = -1000;
    }

    exposed(pi) {
      const p = this.g.piles[pi];
      if (p.kind !== "grid" || !p.cards.length) return false;
      const kids = this.children[pi];
      if (!kids) return true;
      return kids.every((k) => !this.g.piles[k].cards.length);
    }

    onClickCard(pi, ci) {
      const g = this.g;
      const p = g.piles[pi];
      const clickable = (p.kind === "grid" && this.exposed(pi)) || (p.kind === "waste" && p.cards.length && ci === p.cards.length - 1);
      if (!clickable) return null;
      const card = p.top;
      if (card.rank === 13) {
        // König: allein entfernen
        p.cards.pop();
        g.piles[30].cards.push(card);
        const rec = g.newRecord();
        rec.ops.push([pi, 30, 1]);
        rec.score += 5;
        g.playSound("point");
        return rec;
      }
      if (g.sel === null) {
        g.sel = [pi, p.cards.length - 1];
        g.playSound("select");
        return "selected";
      }
      const spi = g.sel[0];
      g.sel = null;
      if (spi === pi) return null;
      const sp = g.piles[spi];
      if (!sp.cards.length) return null;
      const other = sp.top;
      if (other.rank + card.rank === 13) {
        const rec = g.newRecord();
        for (const src of [spi, pi]) {
          const cc = g.piles[src].cards.pop();
          g.piles[30].cards.push(cc);
          rec.ops.push([src, 30, 1]);
          rec.score += 5;
        }
        g.playSound("point");
        return rec;
      }
      return null;
    }

    onStock() {
      const g = this.g;
      const stock = g.piles[0], waste = g.piles[1];
      g.sel = null;
      if (stock.cards.length) {
        const card = stock.cards.pop();
        card.faceUp = true;
        waste.cards.push(card);
        const rec = g.newRecord();
        rec.ops.push([0, 1, 1]);
        rec.flips.push([1, waste.cards.length - 1]);
        return rec;
      }
      if (waste.cards.length && this.redeals > 0) {
        const n = waste.cards.length;
        for (let i = waste.cards.length - 1; i >= 0; i--) {
          waste.cards[i].faceUp = false;
          stock.cards.push(waste.cards[i]);
        }
        waste.cards.length = 0;
        this.redeals -= 1;
        const rec = g.newRecord();
        rec.extra = { recycle: n };
        return rec;
      }
      return null;
    }

    canRecycle() {
      return this.g.piles[1].cards.length > 0 && !this.g.piles[0].cards.length && this.redeals > 0;
    }

    undoExtra(extra) {
      if ("recycle" in extra) {
        const g = this.g;
        const stock = g.piles[0], waste = g.piles[1];
        const moved = stock.cards.splice(stock.cards.length - extra.recycle);
        for (let i = moved.length - 1; i >= 0; i--) {
          moved[i].faceUp = true;
          waste.cards.push(moved[i]);
        }
        this.redeals += 1;
      }
    }

    isWon() {
      for (let i = 0; i < 28; i++) if (this.g.piles[2 + i].cards.length) return false;
      return true;
    }

    winBonus() {
      return 500;
    }

    hudExtra() {
      return [t("sol.redeals_left", { n: this.redeals })];
    }
  }

  // (Reihe, Spalten-Offset in halben Kartenbreiten) für 28 Karten
  const TRIPEAKS_LAYOUT = [];
  for (const c of [1.5, 4.5, 7.5]) TRIPEAKS_LAYOUT.push([0, c]);
  for (const c of [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]) TRIPEAKS_LAYOUT.push([1, c]);
  for (let c = 0; c < 9; c++) TRIPEAKS_LAYOUT.push([2, c + 0.5]);
  for (let c = 0; c < 10; c++) TRIPEAKS_LAYOUT.push([3, c]);

  class TriPeaks extends Variant {
    get key() {
      return "tripeaks";
    }
    // 0 Stock, 1 Waste, 2..29 Feld (28 Einzelkarten)

    deal(rng) {
      const g = this.g;
      g.piles = [new Pile("stock"), new Pile("waste")];
      for (const [row, off] of TRIPEAKS_LAYOUT) g.piles.push(new Pile("grid", { row, off }));
      const deck = C.makeDeck();
      C.shuffle(deck, rng);
      for (let i = 0; i < 28; i++) {
        const card = deck.pop();
        card.faceUp = TRIPEAKS_LAYOUT[i][0] === 3;
        g.piles[2 + i].cards.push(card);
      }
      const first = deck.pop();
      first.faceUp = true;
      g.piles[1].cards.push(first);
      g.piles[0].cards = deck;
      this.combo = 0;
    }

    layout(w, h) {
      const g = this.g;
      g.cw = Math.max(26, Math.floor(w / 11.5));
      g.ch = Math.floor(g.cw * 1.4);
      const top = g.hudH + 6;
      const span = 10 * g.cw;
      const x0 = Math.floor((w - span) / 2);
      TRIPEAKS_LAYOUT.forEach(([row, off], i) => {
        const p = g.piles[2 + i];
        p.x = x0 + Math.floor(off * g.cw);
        p.y = top + Math.floor(row * g.ch * 0.5);
      });
      const m = Math.max(6, Math.floor(w / 60));
      g.piles[0].x = Math.floor(w / 2) - g.cw - m;
      g.piles[0].y = h - g.ch - m;
      g.piles[1].x = Math.floor(w / 2) + m;
      g.piles[1].y = h - g.ch - m;
    }

    /** Indizes der Feld-Karten, die Karte i (Layout-Index) verdecken. */
    _coveredBy(i) {
      const [row, off] = TRIPEAKS_LAYOUT[i];
      const result = [];
      TRIPEAKS_LAYOUT.forEach(([r2, o2], j) => {
        if (r2 === row + 1 && Math.abs(o2 - off) <= 0.51) result.push(j);
      });
      return result;
    }

    uncovered(i) {
      return this._coveredBy(i).every((j) => !this.g.piles[2 + j].cards.length);
    }

    /** Frei gewordene Karten aufdecken (protokolliert). */
    _reveal(rec) {
      for (let i = 0; i < 28; i++) {
        const p = this.g.piles[2 + i];
        if (p.cards.length && !p.top.faceUp && this.uncovered(i)) {
          p.top.faceUp = true;
          rec.flips.push([2 + i, p.cards.length - 1]);
        }
      }
    }

    onClickCard(pi, ci) {
      const g = this.g;
      const p = g.piles[pi];
      if (p.kind !== "grid" || !p.cards.length || !p.top.faceUp) return null;
      const i = pi - 2;
      if (!this.uncovered(i)) return null;
      const wasteTop = g.piles[1].top;
      const d = Math.abs(p.top.rank - wasteTop.rank);
      if (d !== 1 && d !== 12) return null; // +/-1, A<->K wrappt (13-1=12)
      const card = p.cards.pop();
      g.piles[1].cards.push(card);
      this.combo += 1;
      const rec = g.newRecord();
      rec.ops.push([pi, 1, 1]);
      rec.score += 10 * this.combo;
      rec.extra.combo_was = this.combo - 1;
      this._reveal(rec);
      g.playSound("point");
      return rec;
    }

    onStock() {
      const g = this.g;
      const stock = g.piles[0];
      if (!stock.cards.length) return null;
      const card = stock.cards.pop();
      card.faceUp = true;
      g.piles[1].cards.push(card);
      const rec = g.newRecord();
      rec.ops.push([0, 1, 1]);
      rec.flips.push([1, g.piles[1].cards.length - 1]);
      rec.extra.combo_was = this.combo;
      this.combo = 0;
      return rec;
    }

    undoExtra(extra) {
      if ("combo_was" in extra) this.combo = extra.combo_was;
    }

    isWon() {
      for (let i = 0; i < 28; i++) if (this.g.piles[2 + i].cards.length) return false;
      return true;
    }

    winBonus() {
      return 500;
    }

    hudExtra() {
      const out = [t("sol.deals_left", { n: this.g.piles[0].cards.length })];
      if (this.combo > 1) out.push(t("sol.combo", { n: this.combo }));
      return out;
    }
  }

  const VARIANTS = { klondike: Klondike, spider: Spider, freecell: FreeCell, pyramid: Pyramid, tripeaks: TriPeaks };

  // ---------------------------------------------------------------------------
  //  Das Spiel (Hülle um die Varianten)
  // ---------------------------------------------------------------------------

  class SolitaireGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const vkey = VARIANTS[this.mode] ? this.mode : "klondike";
      this.variant = new VARIANTS[vkey](this);

      this._makeFonts();
      this.renderer = new C.CardRenderer(this.accent);
      this.hudH = Math.max(34, Math.floor(this.height * 0.075));

      this.piles = [];
      this.cw = 60;
      this.ch = 84;
      this.sel = null; // Klick-Klick-Auswahl: [pileI, cardI]
      this.drag = null;
      this.undoStack = [];
      this.elapsed = 0;
      this.moves = 0;
      this.won = false;
      this.winBonus = 0;
      this.msg = null;
      this.msgT = 0;
      this._buildSetupLayout();
      this.state = SETUP;
    }

    get wantsRightClick() {
      return true;
    }

    settingsGet(key, def) {
      const v = this.opts[key];
      return v === undefined ? def : v;
    }

    settingsToggle(key) {
      this._saveSetting(key, !this.settingsGet(key, false));
    }

    settingsCycle(key, values) {
      const cur = this.settingsGet(key, values[0]);
      const idx = values.indexOf(cur);
      this._saveSetting(key, values[((idx < 0 ? 0 : idx) + 1) % values.length]);
    }

    _saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    /** Theme-Schriften; _mono für die HUD-Mitte (Zeit/Zahlen ruhig). */
    _makeFonts() {
      this._small = ui.font(16);
      this._tiny = ui.font(13);
      this._mono = ui.font(15, false, true);
      this._huge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    // ===================================================== Setup-Screen
    _buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(380, this.width - 60);
      const y0 = Math.floor(this.height * 0.36);
      const rows = this.variant.setupRows();
      this.optionRects = rows.map((_, i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 52, bw, 42));
      this.startRect = new PG.Rect(cx - 95, y0 + rows.length * 52 + 16, 190, 46);
    }

    _handleSetup(ev) {
      const rows = this.variant.setupRows();
      if (ev.kind === "keydown") {
        if (ev.key === "Return" || ev.key === "space") {
          this._newDeal();
        } else if (["Left", "Right", "a", "d", "A", "D"].includes(ev.key) && rows.length) {
          rows[0][1]();
          this.playSound("select");
          this._buildSetupLayout();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.optionRects.length; i++) {
          if (this.optionRects[i].collidepoint(ev.pos) && i < rows.length) {
            rows[i][1]();
            this.playSound("select");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this._newDeal();
      }
    }

    // ===================================================== Neues Blatt
    _newDeal() {
      this.gameOver = false;
      this.won = false;
      this.score = this.variant instanceof Spider ? 500 : 0;
      this.sel = null;
      this.drag = null;
      this.undoStack = [];
      this.elapsed = 0;
      this.moves = 0;
      this.msg = null;
      this.msgT = 0;
      this.variant.deal(new PG.Random());
      this.variant.layout(this.width, this.height);
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Records / Undo
    newRecord() {
      return { ops: [], flips: [], score: 0, extra: {} };
    }

    /** Bereits entnommene Karten auf dst legen und protokollieren. */
    commit(cards, srcI, dstI, score = 0) {
      this.piles[dstI].cards.push(...cards);
      const rec = this.newRecord();
      rec.ops.push([srcI, dstI, cards.length]);
      rec.score = score;
      return rec;
    }

    _applyRecord(rec) {
      if (rec == null || rec === "selected") return false;
      this.score += rec.score;
      this.moves += 1;
      this.undoStack.push(rec);
      this._checkWin();
      return true;
    }

    _undo() {
      if (!this.undoStack.length || this.gameOver) return;
      // Laufendes Drag zuerst zurücklegen, sonst zieht das Undo Karten
      // unter der Maus weg und der Stapelzustand geht kaputt.
      this._cancelDrag();
      const rec = this.undoStack.pop();
      // WICHTIG: Flips VOR den Ops rückgängig machen - die Indizes beziehen
      // sich auf den Zustand nach dem Zug.
      for (const [pi, ci] of rec.flips) {
        if (ci < this.piles[pi].cards.length) {
          const card = this.piles[pi].cards[ci];
          card.faceUp = !card.faceUp;
        }
      }
      for (let k = rec.ops.length - 1; k >= 0; k--) {
        const [srcI, dstI, n] = rec.ops[k];
        const dst = this.piles[dstI].cards;
        const moved = dst.splice(dst.length - n);
        this.piles[srcI].cards.push(...moved);
      }
      this.score -= rec.score;
      this.moves = Math.max(0, this.moves - 1);
      this.variant.undoExtra(rec.extra || {});
      this.sel = null;
      this.playSound("rotate");
    }

    _checkWin() {
      if (this.variant.isWon()) {
        this.won = true;
        this.winBonus = this.variant.winBonus();
        this.score = Math.max(0, this.score + this.winBonus);
        this.gameOver = true;
        this.reportResult(true);
        this.achEvent("solitaire_win");
        this.playSound("win");
        this.rumble(220);
      }
    }

    flash(text) {
      this.msg = text;
      this.msgT = 1.6;
    }

    // ===================================================== Treffer-Logik
    /** [pileI, cardI] unter der Position - oberste Karte zuerst. */
    _hit(pos) {
      let best = null;
      this.piles.forEach((p, pi) => {
        if (p.kind === "removed") return;
        const ci = C.hitIndex(p.rects(this.cw, this.ch), pos);
        if (ci !== null) best = [pi, ci]; // spätere Piles liegen visuell höher
      });
      return best;
    }

    /** Ablage-Ziel unter der Position (inkl. Bereich unter dem Fächer).
     *  Erst exakte Trefferflächen prüfen, dann großzügig erweiterte -
     *  so gewinnt bei Überlappungen der direkt getroffene Stapel. */
    _pileAt(pos) {
      const regionOf = (p, inflate) => {
        const rects = p.rects(this.cw, this.ch);
        const region = rects.length ? C.unionAll(rects) : new PG.Rect(p.x, p.y, this.cw, this.ch);
        if (inflate) region.h += Math.floor(this.ch / 2);
        return region;
      };
      for (const inflate of [false, true]) {
        for (let pi = 0; pi < this.piles.length; pi++) {
          const p = this.piles[pi];
          if (p.kind === "removed") continue;
          if (regionOf(p, inflate).collidepoint(pos)) return pi;
        }
      }
      return null;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this._handleSetup(ev);
        return;
      }
      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (["Return", "space", "r", "R"].includes(ev.key)) this._newDeal();
          else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this._buildSetupLayout();
            this.playSound("click");
          }
        }
        return;
      }

      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "u" || k === "U") this._undo();
        else if (k === "r" || k === "R") {
          this._cancelDrag();
          this._newDeal();
        } else if (k === "s" || k === "S") {
          this._cancelDrag();
          this.state = SETUP;
          this._buildSetupLayout();
          this.playSound("click");
        } else if (k === "space" || k === "Return") {
          this._cancelDrag();
          this._applyRecord(this.variant.onStock());
        }
      } else if (ev.kind === "mousedown") {
        if (ev.button === 3) {
          if (this.drag) return;
          const hit = this._hit(ev.pos);
          const pi = hit !== null ? hit[0] : this._pileAt(ev.pos);
          if (pi !== null) this._applyRecord(this.variant.onRight(pi));
          return;
        }
        this._onDown(ev.pos);
      } else if (ev.kind === "mousemove") {
        if (this.drag !== null) {
          this.drag.pos = ev.pos;
          const dx = ev.pos[0] - this.drag.start[0];
          const dy = ev.pos[1] - this.drag.start[1];
          if (dx * dx + dy * dy > DRAG_PX * DRAG_PX) this.drag.moved = true;
        }
      } else if (ev.kind === "mouseup") {
        if (ev.button === 3) return;
        this._onUp(ev.pos);
      }
    }

    _onDown(pos) {
      // Verwaistes Drag (verlorenes MOUSEUP, z.B. durch Pause) zuerst
      // als Ablage-Versuch behandeln.
      if (this.drag !== null) {
        this._onUp(pos);
        return;
      }
      const hit = this._hit(pos);
      if (hit === null) {
        // Leerer Stock: Klick auf den kartenlosen Slot löst den Stock-Zug
        // aus - bei Klondike/Pyramid also das Zurücklegen der Waste.
        const p0 = this.piles.length ? this.piles[0] : null;
        if (p0 && p0.kind === "stock" && !p0.cards.length && new PG.Rect(p0.x, p0.y, this.cw, this.ch).collidepoint(pos)) {
          this.sel = null;
          this._applyRecord(this.variant.onStock());
          return;
        }
        this.sel = null;
        return;
      }
      const [pi, ci] = hit;
      const p = this.piles[pi];
      if (p.kind === "stock") {
        this.sel = null;
        this._applyRecord(this.variant.onStock());
        return;
      }
      // Varianten mit eigener Klick-Logik (Pyramid/TriPeaks)
      const rec = this.variant.onClickCard(pi, ci);
      if (rec !== null) {
        if (rec !== "selected") this._applyRecord(rec);
        return;
      }
      if (this.variant.canGrab(pi, ci)) {
        const cards = p.cards.splice(ci);
        this.drag = {
          src: pi, ci, cards, pos, start: pos, t: nowSec(), moved: false,
          off: [pos[0] - p.x, pos[1] - (p.y + ci * (p.dyUp || 1))],
        };
        // Offset an der tatsächlichen Kartenposition ausrichten (Fächer)
        const rects = C.fanRects(p.x, p.y, p.cards.concat(cards), this.cw, this.ch, p.dyDown, p.dyUp);
        if (rects[ci]) this.drag.off = [pos[0] - rects[ci].x, pos[1] - rects[ci].y];
        this.playSound("select");
      } else {
        this.sel = null;
      }
    }

    _onUp(pos) {
      if (this.drag === null) return;
      const d = this.drag;
      this.drag = null;
      const quick = !d.moved && nowSec() - d.t < CLICK_S;
      if (quick) {
        // Klick: Karten zurücklegen und Auswahl setzen bzw. ablegen.
        this._returnDrag(d);
        if (this.sel !== null && !(this.sel[0] === d.src && this.sel[1] === d.ci)) {
          const [spi, sci] = this.sel;
          this.sel = null;
          this._tryMove(spi, sci, d.src);
        } else {
          this.sel = [d.src, d.ci];
        }
        return;
      }
      // Echter Drag: Ziel suchen.
      const dst = this._pileAt(pos);
      if (dst !== null && dst !== d.src) {
        const rec = this.variant.tryDrop(d.cards, d.src, dst);
        if (rec !== null) {
          this._applyRecord(rec);
          this.sel = null;
          this.playSound("move");
          return;
        }
      }
      this._returnDrag(d);
    }

    _returnDrag(d) {
      this.piles[d.src].cards.push(...d.cards);
    }

    /** Klick-Klick: Substack von (spi, sci) auf Ziel dst versuchen. */
    _tryMove(spi, sci, dst) {
      const p = this.piles[spi];
      if (sci >= p.cards.length || !this.variant.canGrab(spi, sci)) return;
      const cards = p.cards.splice(sci);
      const rec = this.variant.tryDrop(cards, spi, dst);
      if (rec !== null) {
        this._applyRecord(rec);
        this.playSound("move");
      } else {
        p.cards.push(...cards);
      }
    }

    _cancelDrag() {
      if (this.drag !== null) {
        this._returnDrag(this.drag);
        this.drag = null;
      }
    }

    // ===================================================== Update / Zeichnen
    update(dt) {
      if (this.state !== PLAY || this.gameOver) return;
      this.elapsed += dt;
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      // Spider: Punktestand aus Zählern ableiten (stabil bei Undo).
      if (this.variant instanceof Spider) this.score = this.variant.scoreNow();
    }

    _fmtTime() {
      const sec = Math.floor(this.elapsed);
      const mm = String(Math.floor(sec / 60)).padStart(2, "0");
      const ss = String(sec % 60).padStart(2, "0");
      return mm + ":" + ss;
    }

    draw(ctx) {
      if (this.state === SETUP) {
        this._drawSetup(ctx);
        return;
      }
      C.blitFelt(ctx, this, this.width, this.height, COL_FELT, COL_FELT_EDGE);
      this._drawPiles(ctx);
      if (this.drag !== null) this._drawDrag(ctx);
      this._drawHud(ctx);
      if (this.gameOver) this._drawResult(ctx);
    }

    _drawPiles(ctx) {
      const cw = this.cw, ch = this.ch;
      const R = this.renderer;
      this.piles.forEach((p, pi) => {
        if (p.kind === "removed") return;
        if (!p.cards.length) {
          if (p.kind !== "grid") C.drawSlot(ctx, [p.x, p.y, cw, ch]);
          if (p.kind === "stock" && this.variant.canRecycle()) this._drawRecycle(ctx, p, cw, ch);
          return;
        }
        if (p.kind === "stock") {
          R.draw(ctx, p.top, p.x, p.y, cw, ch);
          if (p.cards.length > 1) ui.text(ctx, String(p.cards.length), p.x + 2, p.y + ch + 2, this._tiny, ui.TEXT_DIM);
        } else if (p.kind === "waste" || p.kind === "foundation" || p.kind === "cell" || p.kind === "grid") {
          R.draw(ctx, p.top, p.x, p.y, cw, ch);
        } else {
          // tableau: auffächern
          const rects = p.rects(cw, ch);
          p.cards.forEach((card, i) => R.draw(ctx, card, rects[i].x, rects[i].y, cw, ch));
        }
        // Auswahl-Rahmen
        if (this.sel !== null && this.sel[0] === pi) {
          const rects = p.rects(cw, ch);
          if (rects.length) {
            const ci = Math.min(this.sel[1], rects.length - 1);
            const selR = rects.length > ci + 1 ? C.unionAll(rects.slice(ci)) : rects[ci];
            draw.rect(ctx, ui.GOLD, selR.inflate(4, 4), 2, 6);
          }
        }
      });
    }

    /** Kreisförmiger Pfeil (↻) auf dem leeren Stock als Hinweis, dass die
     *  Waste zum erneuten Durchgehen zurückgelegt werden kann. */
    _drawRecycle(ctx, p, cw, ch) {
      const cx = p.x + Math.floor(cw / 2), cy = p.y + Math.floor(ch / 2);
      const r = Math.max(8, Math.floor(Math.min(cw, ch) / 4));
      const col = this.accent;
      // offener Ring (oben rechts eine Lücke für die Pfeilspitze)
      draw.arc(ctx, col, [cx - r, cy - r, 2 * r, 2 * r], -0.35, 4.9, Math.max(2, Math.floor(r / 4)));
      // Pfeilspitze am oberen Ende des Rings
      const tip = [cx + Math.floor(r * 0.95), cy - Math.floor(r * 0.33)];
      draw.polygon(ctx, col, [tip, [tip[0] - Math.floor(r / 2), tip[1] - Math.floor(r / 6)], [tip[0] - Math.floor(r / 6), tip[1] + Math.floor(r / 2)]]);
    }

    _drawDrag(ctx) {
      const d = this.drag;
      const x = d.pos[0] - d.off[0];
      const y = d.pos[1] - d.off[1];
      const p = this.piles[d.src];
      const dy = p.dyUp || Math.floor(this.ch * 0.28);
      ctx.save();
      ctx.shadowColor = "rgba(0,0,0,0.45)";
      ctx.shadowBlur = 12;
      ctx.shadowOffsetY = 4;
      d.cards.forEach((card, i) => this.renderer.draw(ctx, card, x, y + i * dy, this.cw, this.ch));
      ctx.restore();
    }

    _drawHud(ctx) {
      draw.rect(ctx, [8, 22, 16, 224], [0, 0, this.width, this.hudH]);
      draw.line(ctx, this.accent, [0, this.hudH], [this.width, this.hudH], 2);
      const cy = Math.floor(this.hudH / 2);
      ui.text(ctx, t("sol.mode." + this.variant.key), 12, cy, this._small, this.accent, "midleft");
      // Monospace: Zeit/Zahlen ändern sich, ohne dass die Zeile "springt".
      const mid = t("common.points", { score: Math.max(0, this.score) }) + "   ·   " + t("sol.moves", { n: this.moves }) + "   ·   " + this._fmtTime();
      ui.text(ctx, mid, this.width / 2, cy, this._mono, ui.TEXT, "center");
      let x = this.width - 12;
      for (const txt of this.variant.hudExtra()) {
        const r = ui.text(ctx, txt, x, cy, this._small, ui.TEXT_DIM, "midright");
        x -= r.w + 16;
      }
      if (this.msg) ui.text(ctx, this.msg, this.width / 2, this.hudH + 14, this._small, ui.GOLD, "center");
      ui.text(ctx, t("sol.hint"), this.width / 2, this.height - 4, this._tiny, ui.TEXT_FAINT, "midbottom");
    }

    _drawResult(ctx) {
      draw.rect(ctx, [6, 14, 10, 190], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      const head = t("sol.win", { bonus: this.winBonus });
      const pw = Math.min(this.width - 40, Math.max(380, this._huge.width(head) + 70));
      const panel = new PG.Rect(cx - pw / 2, cy - 96, pw, 192);
      ui.drawPanel(ctx, panel, { accentTop: this.accent });
      ui.text(ctx, head, cx, cy - 46, this._huge, ui.GOLD, "center");
      ui.text(ctx, t("common.points", { score: this.score }), cx, cy + 10, this.font, ui.TEXT, "center");
      ui.text(ctx, t("sol.retry"), cx, cy + 52, this._small, ui.TEXT_DIM, "center");
    }

    // ----- Setup zeichnen -----------------------------------------------
    _drawSetup(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false);
      ui.drawTitle(ctx, this.width, t("sol.mode." + this.variant.key), {
        subtitle: t("sol.subtitle." + this.variant.key),
        y: Math.floor(this.height * 0.14),
        big: this._huge,
        accent: this.accent,
      });
      const rows = this.variant.setupRows();
      this.optionRects.forEach((r, i) => {
        if (i < rows.length) ui.drawButton(ctx, r, rows[i][0](), this._small, false, { accent: this.accent });
      });
      ui.drawButton(ctx, this.startRect, t("common.start"), this.font, true, { accent: this.accent });
      ui.drawFooter(ctx, this.width, this.height, t("sol.setup_hint"), this._tiny);
    }
  }

  PG.register(SolitaireGame, {
    id: "SolitaireGame",
    key: "solitaire",
    name: "Solitaire",
    modes: [
      ["klondike", "sol.mode.klondike"],
      ["spider", "sol.mode.spider"],
      ["freecell", "sol.mode.freecell"],
      ["pyramid", "sol.mode.pyramid"],
      ["tripeaks", "sol.mode.tripeaks"],
    ],
    settingsKey: "solitaire",
    defaults: { draw3: false, spider_suits: 1 },
    wantsRightClick: true,
  });
})();
