/*
 * cards.js - Gemeinsames Spielkarten-Toolkit (Port von games/cards.py)
 * =====================================================================
 * Wird von Solitär, Blackjack und Poker geteilt (API an PG.cards).
 *
 * Alles wird mit Canvas-Primitiven gezeichnet - keine Bild-Dateien:
 * - Card          : Rang 1..13 (A..K), Farbe 0..3 (Pik/Herz/Karo/Kreuz), faceUp-Flag.
 * - makeDeck      : Standard-52er-Deck oder Spider-Varianten (104 Karten).
 * - CardRenderer  : rendert Vorder-/Rückseiten in beliebiger Größe und cached sie
 *                   je (Rang, Farbe, Breite, Höhe, Seite) als Offscreen-Canvas
 *                   (Größe x PG.app.pixelScale, damit sie scharf bleiben).
 *                   Die Rückseite trägt den Akzentton des jeweiligen Spiels.
 *                   Die Farbsymbole (Pips) sind Polygone/Kreise:
 *                   Herz = 2 Kreise + Dreieck, Karo = Raute,
 *                   Pik = umgedrehtes Herz + Fuß, Kreuz = 3 Kreise + Fuß.
 * - drawSlot      : gestrichelte Umrandung für leere Ablagen.
 * - fanRects      : Trefferflächen eines aufgefächerten Stapels.
 * - hitIndex      : oberste getroffene Karte eines Stapels.
 * - makeFelt      : einmalig gerenderter Filz-Hintergrund (Verlauf + Tischlicht
 *                   + Bande), damit alle Kartenspiele denselben Tisch-Look teilen.
 */
(function () {
  "use strict";

  const { ui, draw } = PG;

  const SUIT_SPADE = 0, SUIT_HEART = 1, SUIT_DIAMOND = 2, SUIT_CLUB = 3;

  const RANK_LABELS = { 1: "A", 11: "J", 12: "Q", 13: "K" };

  // Karten-Identität: leicht entsättigt, damit sie zum v4.1-Look passt.
  const COL_RED = [198, 58, 62];
  const COL_BLACK = [36, 40, 52];
  const COL_FACE = [240, 242, 247];
  const COL_FACE_EDGE = [150, 158, 178];

  function rankLabel(rank) {
    return RANK_LABELS[rank] || String(rank);
  }

  function isRed(suit) {
    return suit === SUIT_HEART || suit === SUIT_DIAMOND;
  }

  class Card {
    constructor(rank, suit, faceUp = false) {
      this.rank = rank;
      this.suit = suit;
      this.faceUp = faceUp;
    }
    get red() {
      return isRed(this.suit);
    }
    toString() {
      return rankLabel(this.rank) + "shdc"[this.suit];
    }
  }

  /** Deck bauen: Standard = 52 Karten. Spider: 1 Farbe x8, 2 Farben x4,
   *  4 Farben x2 - jeweils 104 Karten. */
  function makeDeck(suits = [0, 1, 2, 3], copies = 1) {
    const deck = [];
    for (let n = 0; n < copies; n++) {
      for (const suit of suits) {
        for (let rank = 1; rank <= 13; rank++) deck.push(new Card(rank, suit));
      }
    }
    return deck;
  }

  function shuffle(deck, rng) {
    (rng || PG.rand).shuffle(deck);
  }

  // ---------------------------------------------------------------------------
  //  Zeichnen
  // ---------------------------------------------------------------------------

  /** Offscreen-Canvas in logischer Größe w x h (intern x pixelScale). */
  function scaledCanvas(w, h) {
    const s = scale();
    const c = ui.makeCanvas(Math.ceil(w * s), Math.ceil(h * s));
    const g = c.getContext("2d");
    g.scale(s, s);
    return [c, g];
  }

  function scale() {
    return (PG.app && PG.app.pixelScale) || 1;
  }

  /** Zeichnet ein Farbsymbol mit Radius r zentriert auf (cx, cy). */
  function drawSuit(g, suit, cx, cy, r, col) {
    r = Math.max(2, r);
    if (suit === SUIT_HEART) {
      const rr = Math.max(2, r * 0.55);
      draw.circle(g, col, [cx - rr + 1, cy - rr / 2], rr);
      draw.circle(g, col, [cx + rr - 1, cy - rr / 2], rr);
      draw.polygon(g, col, [[cx - 2 * rr + 1, cy - rr / 6], [cx + 2 * rr - 1, cy - rr / 6], [cx, cy + rr * 1.6]]);
    } else if (suit === SUIT_DIAMOND) {
      draw.polygon(g, col, [[cx, cy - r], [cx + r * 0.7, cy], [cx, cy + r], [cx - r * 0.7, cy]]);
    } else if (suit === SUIT_SPADE) {
      const rr = Math.max(2, r * 0.55);
      draw.circle(g, col, [cx - rr + 1, cy + rr / 2], rr);
      draw.circle(g, col, [cx + rr - 1, cy + rr / 2], rr);
      draw.polygon(g, col, [[cx - 2 * rr + 1, cy + rr / 6], [cx + 2 * rr - 1, cy + rr / 6], [cx, cy - rr * 1.6]]);
      draw.polygon(g, col, [[cx - rr / 2, cy + rr * 1.7], [cx + rr / 2, cy + rr * 1.7], [cx, cy + rr / 2]]);
    } else {
      // Kreuz
      const rr = Math.max(2, r * 0.45);
      draw.circle(g, col, [cx, cy - rr], rr);
      draw.circle(g, col, [cx - rr, cy + rr / 2], rr);
      draw.circle(g, col, [cx + rr, cy + rr / 2], rr);
      draw.polygon(g, col, [[cx - rr / 2, cy + rr * 2.1], [cx + rr / 2, cy + rr * 2.1], [cx, cy]]);
    }
  }

  /** Rendert und cached Kartenflächen. accent = Rückseiten-Farbe (RGB). */
  class CardRenderer {
    constructor(accent = [47, 167, 124]) {
      this.accent = accent;
      this._cache = new Map();
      this._scale = scale();
    }

    clear() {
      this._cache.clear();
    }

    _check() {
      // Fenster/Skalierung geändert -> neu rendern (für Schärfe)
      const s = scale();
      if (s !== this._scale) {
        this._scale = s;
        this._cache.clear();
      }
    }

    get(card, w, h) {
      if (!card.faceUp) return this.back(w, h);
      this._check();
      const key = card.rank + "|" + card.suit + "|" + w + "|" + h;
      let surf = this._cache.get(key);
      if (!surf) {
        surf = this._face(card.rank, card.suit, w, h);
        this._cache.set(key, surf);
      }
      return surf;
    }

    /** Karte direkt zeichnen (Vorder- oder Rückseite je nach faceUp). */
    draw(ctx, card, x, y, w, h) {
      ctx.drawImage(this.get(card, w, h), x, y, w, h);
    }

    drawBack(ctx, x, y, w, h) {
      ctx.drawImage(this.back(w, h), x, y, w, h);
    }

    back(w, h) {
      this._check();
      const key = "back|" + w + "|" + h;
      let surf = this._cache.get(key);
      if (!surf) {
        const [c, g] = scaledCanvas(w, h);
        const rad = Math.max(3, Math.floor(w / 8));
        // Akzent leicht abgedunkelt = Grundton der Rückseite
        const base = ui.mix(this.accent, [16, 20, 28], 0.3);
        const dark = ui.mix(base, [0, 0, 0], 0.35);
        // weißer Kartenrand wie bei echten Rückseiten
        draw.rect(g, COL_FACE, [0, 0, w, h], 0, rad);
        const inner = new PG.Rect(2, 2, w - 4, h - 4);
        const irad = Math.max(2, rad - 2);
        draw.rect(g, base, inner, 0, irad);
        // Rauten-Muster (auf die Innenfläche geclippt)
        const pat = inner.inflate(-4, -4);
        const step = Math.max(6, Math.floor(w / 5));
        g.save();
        g.beginPath();
        g.rect(pat.x, pat.y, pat.w, pat.h);
        g.clip();
        g.beginPath();
        for (let yy = pat.y; yy < pat.bottom; yy += step) {
          for (let xx = pat.x; xx < pat.right; xx += step) {
            g.moveTo(xx, yy + step / 2);
            g.lineTo(xx + step / 2, yy);
            g.lineTo(xx + step, yy + step / 2);
          }
        }
        g.strokeStyle = ui.col(dark);
        g.lineWidth = 1;
        g.stroke();
        g.restore();
        draw.rect(g, dark, inner, 1, irad);
        draw.rect(g, COL_FACE_EDGE, [0, 0, w, h], 1, rad);
        surf = c;
        this._cache.set(key, surf);
      }
      return surf;
    }

    _face(rank, suit, w, h) {
      const [c, g] = scaledCanvas(w, h);
      const rad = Math.max(3, Math.floor(w / 8));
      draw.rect(g, COL_FACE, [0, 0, w, h], 0, rad);
      draw.rect(g, COL_FACE_EDGE, [0, 0, w, h], 1, rad);
      const col = isRed(suit) ? COL_RED : COL_BLACK;
      const fnt = ui.font(Math.max(10, Math.floor(h / 5)), true);
      const label = rankLabel(rank);
      const lw = fnt.width(label), lh = fnt.height;
      const mx = Math.max(2, Math.floor(w / 12));
      // Rang + kleines Symbol oben links
      ui.text(g, label, mx, 2, fnt, col);
      const pr = Math.max(2, Math.floor(w / 10));
      drawSuit(g, suit, mx + lw / 2, 4 + lh + pr, pr, col);
      // Gespiegelt unten rechts
      g.save();
      g.translate(w - mx, h - 2);
      g.rotate(Math.PI);
      ui.text(g, label, 0, 0, fnt, col);
      g.restore();
      // Großes Symbol in der Mitte (Bildkarten: großer Buchstabe + Symbol)
      const big = Math.max(4, Math.floor(w / 4));
      if (rank > 10 || rank === 1) {
        const bfnt = ui.font(Math.max(14, Math.floor(h / 3)), true);
        ui.text(g, label, w / 2, h / 2 - big / 2, bfnt, col, "center");
        drawSuit(g, suit, w / 2, h / 2 + big, big / 2 + 2, col);
      } else {
        drawSuit(g, suit, w / 2, h / 2, big, col);
      }
      return c;
    }
  }

  /** Filz-Hintergrund EINMAL rendern (cachen!): sanfter vertikaler Verlauf,
   *  weiches Tischlicht oben und eine dunkle Bande mit Zierlinie am Rand.
   *  Rückgabe: Canvas (mit drawImage(c, 0, 0, w, h) zeichnen). */
  function makeFelt(w, h, base = [22, 48, 38], edge = [13, 30, 23]) {
    const [c, g] = scaledCanvas(w, h);
    const top = ui.mix(base, [255, 255, 255], 0.1);
    const grad = g.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, ui.col(top));
    grad.addColorStop(1, ui.col(ui.mix(top, edge, 0.85)));
    g.fillStyle = grad;
    g.fillRect(0, 0, w, h);
    // weiches Oval-Licht in der oberen Tischhälfte
    const gw = Math.floor(w * 0.86), gh = Math.floor(h * 0.62);
    draw.ellipse(g, [255, 255, 255, 14], [w / 2 - gw / 2, Math.floor(h * 0.14), gw, gh]);
    draw.ellipse(g, [255, 255, 255, 10], [w / 2 - Math.floor(gw * 0.42), Math.floor(h * 0.2), Math.floor(gw * 0.84), Math.floor(gh * 0.8)]);
    // Bande/Vignette am Rand
    draw.rect(g, edge, [0, 0, w, h], 6);
    draw.rect(g, ui.mix(edge, base, 0.5), [6, 6, w - 12, h - 12], 2);
    draw.rect(g, ui.mix(base, [255, 255, 255], 0.06), [8, 8, w - 16, h - 16], 1);
    c.logicalW = w;
    c.logicalH = h;
    c.pixelScale = scale();
    return c;
  }

  /** Filz zeichnen und bei geänderter Skalierung neu rendern.
   *  holder = Objekt mit Feld _felt; args = [w, h, base, edge]. */
  function blitFelt(ctx, holder, w, h, base, edge) {
    if (!holder._felt || holder._felt.pixelScale !== scale() || holder._felt.logicalW !== w) {
      holder._felt = makeFelt(w, h, base, edge);
    }
    ctx.drawImage(holder._felt, 0, 0, w, h);
  }

  /** Umrandung für eine leere Ablage (gestrichelter Look). */
  function drawSlot(ctx, rect, color = [92, 118, 104]) {
    const r = rect instanceof PG.Rect ? rect : new PG.Rect(...rect);
    const rad = Math.max(3, Math.floor(r.w / 8));
    const step = 7;
    draw.rect(ctx, color.map((v) => Math.floor(v * 0.5)), r, 1, rad);
    ctx.save();
    ctx.beginPath();
    for (let x = r.x + rad; x < r.right - rad; x += step * 2) {
      const x2 = Math.min(x + step, r.right - rad);
      ctx.moveTo(x, r.y + 0.5);
      ctx.lineTo(x2, r.y + 0.5);
      ctx.moveTo(x, r.bottom - 0.5);
      ctx.lineTo(x2, r.bottom - 0.5);
    }
    for (let y = r.y + rad; y < r.bottom - rad; y += step * 2) {
      const y2 = Math.min(y + step, r.bottom - rad);
      ctx.moveTo(r.x + 0.5, y);
      ctx.lineTo(r.x + 0.5, y2);
      ctx.moveTo(r.right - 0.5, y);
      ctx.lineTo(r.right - 0.5, y2);
    }
    ctx.strokeStyle = ui.col(color);
    ctx.lineWidth = 1;
    ctx.stroke();
    ctx.restore();
  }

  /** Trefferflächen eines vertikal aufgefächerten Stapels (oben = Ende). */
  function fanRects(x, y, cards, w, h, dyDown, dyUp) {
    const rects = [];
    let yy = y;
    for (const card of cards) {
      rects.push(new PG.Rect(Math.floor(x), Math.floor(yy), w, h));
      yy += card.faceUp ? dyUp : dyDown;
    }
    return rects;
  }

  /** Index der obersten (zuletzt gezeichneten) getroffenen Karte oder null. */
  function hitIndex(rects, pos) {
    for (let i = rects.length - 1; i >= 0; i--) {
      if (rects[i].collidepoint(pos)) return i;
    }
    return null;
  }

  /** Vereinigung mehrerer Rects (wie pygame Rect.unionall). */
  function unionAll(rects) {
    let r = rects[0].copy();
    for (let i = 1; i < rects.length; i++) r = r.union(rects[i]);
    return r;
  }

  PG.cards = {
    SUIT_SPADE, SUIT_HEART, SUIT_DIAMOND, SUIT_CLUB,
    COL_RED, COL_BLACK, COL_FACE, COL_FACE_EDGE,
    rankLabel, isRed, Card, makeDeck, shuffle, drawSuit,
    CardRenderer, makeFelt, blitFelt, drawSlot, fanRects, hitIndex, unionAll,
  };
})();
