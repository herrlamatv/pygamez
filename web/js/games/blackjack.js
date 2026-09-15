/*
 * blackjack.js - Blackjack gegen den Dealer (Port von games/blackjack.py)
 * ========================================================================
 * Mit Chips, Double Down und Split.
 *
 * Regeln:
 * - 4-Deck-Schuh (208 Karten), neu gemischt wenn weniger als 52 übrig sind.
 * - Dealer steht auf ALLEN 17 (auch Soft 17). Blackjack zahlt 3:2.
 * - Dealer-Peek bei Ass/Zehnerkarte: Dealer-Blackjack beendet die Runde sofort
 *   (eigener Blackjack = Push). Keine Insurance.
 * - Double Down: nur auf den ersten beiden Karten (auch nach Split, außer bei
 *   Split-Assen), kostet den gleichen Einsatz, genau eine Karte, Auto-Stand.
 * - Split: genau EINMAL, bei gleichem Kartenwert (K+10 geht); Split-Asse
 *   bekommen je genau eine Karte; 21 nach Split zählt als 21, nicht Blackjack.
 *
 * Chips: Start 500, Einsätze 10/25/50/100 (stapelbar). Der Chipstand bleibt
 * über Sitzungen erhalten (PG.store, Abschnitt "mem.blackjack"). Der Highscore
 * ist der höchste jemals erreichte Chipstand; er wird beim Menü-Rückweg
 * gespeichert (gameOver wird nie gesetzt). Pleite = Neustart mit 500.
 *
 * Steuerung: Buttons anklicken oder H = Hit, S = Stand, D = Double, X = Split,
 * 1-4 = Chips setzen, Backspace = Einsatz löschen, Enter = Geben/Weiter.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const C = PG.cards;

  // Tisch-Identität (Filz + Chip-Farben). Generische UI-Farben kommen zur
  // Zeichenzeit aus ui.* (Theme), die Akzentfarbe aus this.accent.
  const COL_FELT = [23, 46, 37];
  const COL_FELT_EDGE = [15, 33, 26];
  const CHIP_COLS = { 10: [110, 160, 235], 25: [110, 205, 140], 50: [230, 120, 90], 100: [40, 40, 48] };

  const STORE_KEY = "mem.blackjack";
  const START_CHIPS = 500;
  const BETS = [10, 25, 50, 100];
  const DEAL_T = 0.22; // Tween-Dauer je Karte
  const FLIP_T = 0.25; // Hole-Card-Flip
  const DEALER_T = 0.5; // Pause je Dealer-Karte

  const BET = "bet", DEALING = "dealing", PLAYER = "player", DEALER = "dealer", PAYOUT = "payout", BROKE = "broke";

  /** [Summe, soft?] - Asse zählen 11, solange kein Bust. */
  function handValue(cards) {
    let total = 0, aces = 0;
    for (const c of cards) {
      let v = Math.min(c.rank, 10);
      if (c.rank === 1) {
        aces += 1;
        v = 11;
      }
      total += v;
    }
    while (total > 21 && aces) {
      total -= 10;
      aces -= 1;
    }
    return [total, aces > 0];
  }

  function toInt(v, def) {
    const n = parseInt(v, 10);
    return Number.isFinite(n) ? n : def;
  }

  class BlackjackGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      const data = PG.store.get(STORE_KEY, {}) || {};
      this.chips = Math.max(0, toInt(data.chips, START_CHIPS));
      this.best = Math.max(this.chips, toInt(data.best, START_CHIPS));
      this.score = this.best;

      this._makeFonts();
      this.renderer = new C.CardRenderer(this.accent);
      this._layout();

      this.shoe = [];
      this.bet = 0;
      this.hands = []; // Listen von Cards (1 oder 2 nach Split)
      this.handBets = [];
      this.handDone = [];
      this.active = 0;
      this.splitAces = false;
      this.dealer = [];
      this.holeHidden = true;
      this.results = []; // Texte je Hand im PAYOUT
      this.tweens = []; // Karten-Flüge {card, dealer, idx, t, delay, landed}
      this._fly = new Set();
      this.dealerWait = 0;
      this.flipT = 0;
      this.state = this.chips >= BETS[0] ? BET : BROKE;
    }

    /** Theme-Schriften (ui.font cached selbst); _huge hängt an height. */
    _makeFonts() {
      this._small = ui.font(16);
      this._tiny = ui.font(13);
      this._big = ui.font(22, true);
      this._huge = ui.font(Math.max(26, Math.floor(this.height / 11)), true);
    }

    _layout() {
      const W = this.width, H = this.height;
      this.ch = Math.floor(H * 0.2);
      this.cw = Math.floor(this.ch * 0.72);
      this.shoePos = [W - this.cw - 16, 12];
      this.dealerY = Math.floor(H * 0.2);
      this.playerY = Math.floor(H * 0.58);
      this.strip = new PG.Rect(0, H - 72, W, 72);
      // BET-Bedienung
      this.chipRects = BETS.map((_, i) => new PG.Rect(24 + i * 56, this.strip.y + 14, 44, 44));
      const bw = Math.floor(W * 0.14);
      this.dealRect = new PG.Rect(W - bw - 16, this.strip.y + 14, bw, 44);
      this.clearRect = new PG.Rect(W - 2 * bw - 28, this.strip.y + 14, bw, 44);
      // PLAYER-Buttons
      const bw2 = Math.floor(W * 0.16);
      const gap = Math.floor((W - 4 * bw2) / 5);
      this.actionRects = {};
      ["hit", "stand", "double", "split"].forEach((key, i) => {
        this.actionRects[key] = new PG.Rect(gap + i * (bw2 + gap), this.strip.y + 14, bw2, 44);
      });
    }

    _save() {
      PG.store.set(STORE_KEY, { chips: this.chips, best: this.best });
    }

    // ===================================================== Schuh / Hände
    _ensureShoe() {
      if (this.shoe.length < 52) {
        this.shoe = C.makeDeck(undefined, 4);
        C.shuffle(this.shoe, new PG.Random());
      }
    }

    _drawCard(faceUp = true) {
      const card = this.shoe.pop();
      card.faceUp = faceUp;
      return card;
    }

    /** Bildschirmpositionen der Karten einer Spielerhand. */
    _handPositions(handI, n) {
      const cx = this.hands.length === 1 ? Math.floor(this.width / 2) : Math.floor(this.width * (handI === 0 ? 0.35 : 0.65));
      const dx = Math.floor(this.cw * 0.55);
      const x0 = cx - Math.floor((this.cw + (n - 1) * dx) / 2);
      const out = [];
      for (let i = 0; i < n; i++) out.push([x0 + i * dx, this.playerY]);
      return out;
    }

    _dealerPositions(n) {
      const dx = Math.floor(this.cw * 0.55);
      const x0 = Math.floor(this.width / 2) - Math.floor((this.cw + (n - 1) * dx) / 2);
      const out = [];
      for (let i = 0; i < n; i++) out.push([x0 + i * dx, this.dealerY]);
      return out;
    }

    // ===================================================== Runden-Ablauf
    _startDeal() {
      if (this.bet < BETS[0] || this.bet > this.chips) return;
      this._ensureShoe();
      this.chips -= this.bet;
      this.hands = [[]];
      this.handBets = [this.bet];
      this.handDone = [false];
      this.active = 0;
      this.splitAces = false;
      this.dealer = [];
      this.holeHidden = true;
      this.results = [];
      this.tweens = [];
      this.state = DEALING;
      // Reihenfolge: Spieler, Dealer, Spieler, Dealer (verdeckt)
      for (const [target, up] of [[this.hands[0], true], [this.dealer, true], [this.hands[0], true], [this.dealer, false]]) {
        const card = this._drawCard(up);
        target.push(card);
        this._enqueue(card, target === this.dealer, target.length - 1);
      }
      this.playSound("move");
    }

    /** Karte fliegt zeitversetzt vom Schuh an ihren Platz (idx in Hand). */
    _enqueue(card, isDealer, idx) {
      this.tweens.push({ card, dealer: isDealer, idx, t: 0, delay: this.tweens.length * DEAL_T, landed: false });
    }

    /** Nach der Startausgabe: Peek/Blackjack prüfen. */
    _afterDeal() {
      const [pv] = handValue(this.hands[0]);
      const [dv] = handValue(this.dealer);
      const playerBj = pv === 21;
      const up = this.dealer[0];
      if (up.rank === 1 || Math.min(up.rank, 10) === 10) {
        if (dv === 21) {
          // Dealer-Blackjack
          this.holeHidden = false;
          this._settle(playerBj ? ["push"] : ["lose"]);
          return;
        }
      }
      if (playerBj) {
        this.holeHidden = false;
        this.achEvent("blackjack_two");
        this._settle(["blackjack"]);
        return;
      }
      this.state = PLAYER;
      this.playSound("select");
    }

    // ----- Spieler-Aktionen ------------------------------------------------

    _canDouble() {
      const h = this.hands[this.active];
      return h.length === 2 && !this.splitAces && this.chips >= this.handBets[this.active];
    }

    _canSplit() {
      if (this.hands.length !== 1) return false;
      const h = this.hands[0];
      return h.length === 2 && Math.min(h[0].rank, 10) === Math.min(h[1].rank, 10) && this.chips >= this.handBets[0];
    }

    _hit() {
      const h = this.hands[this.active];
      h.push(this._drawCard());
      this.playSound("move");
      if (handValue(h)[0] >= 21) this._nextHand();
    }

    _stand() {
      this._nextHand();
    }

    _double() {
      if (!this._canDouble()) return;
      this.chips -= this.handBets[this.active];
      this.handBets[this.active] *= 2;
      this.hands[this.active].push(this._drawCard());
      this.playSound("move");
      this._nextHand();
    }

    _split() {
      if (!this._canSplit()) return;
      const h = this.hands[0];
      this.splitAces = h[0].rank === 1;
      this.chips -= this.handBets[0];
      this.hands = [[h[0]], [h[1]]];
      this.handBets = [this.handBets[0], this.handBets[0]];
      this.handDone = [false, false];
      this.active = 0;
      // Jede Hand bekommt sofort eine zweite Karte.
      this.hands[0].push(this._drawCard());
      this.hands[1].push(this._drawCard());
      this.playSound("rotate");
      if (this.splitAces) {
        // Split-Asse: keine weiteren Aktionen.
        this.handDone = [true, true];
        this._startDealer();
      }
    }

    _nextHand() {
      this.handDone[this.active] = true;
      if (this.active + 1 < this.hands.length) {
        this.active += 1;
        if (handValue(this.hands[this.active])[0] >= 21) this._nextHand();
        return;
      }
      // Alle Hände durch -> Dealer (nur wenn nicht alles Bust)
      const bustedAll = this.hands.every((h) => handValue(h)[0] > 21);
      if (bustedAll) {
        this.holeHidden = false;
        this._settle(null);
      } else {
        this._startDealer();
      }
    }

    /** Hole-Card wirklich umdrehen: sie wurde verdeckt ausgegeben und der
     *  Renderer zeigt verdeckte Karten immer als Rückseite. */
    _revealHole() {
      this.holeHidden = false;
      for (const c of this.dealer) c.faceUp = true;
    }

    _startDealer() {
      this.state = DEALER;
      // Hole-Card JETZT aufdecken, damit der Flip sichtbar abläuft.
      this._revealHole();
      this.flipT = FLIP_T;
      this.dealerWait = DEALER_T;
      this.playSound("rotate");
    }

    // ----- Dealer + Auszahlung ------------------------------------------------

    _dealerStep() {
      const [total] = handValue(this.dealer);
      if (total < 17) {
        // steht auf ALLEN 17 (S17)
        this.dealer.push(this._drawCard());
        this.playSound("move");
        this.dealerWait = DEALER_T;
      } else {
        this._settle(null);
      }
    }

    /** Zahlt alle Hände aus. forced: Liste je Hand oder null (berechnen). */
    _settle(forced) {
      this._revealHole();
      const [dv] = handValue(this.dealer);
      const dealerBust = dv > 21;
      this.results = [];
      let delta = 0;
      this.hands.forEach((h, i) => {
        const bet = this.handBets[i];
        let res;
        if (forced !== null) {
          res = forced[Math.min(i, forced.length - 1)];
        } else {
          const [pv] = handValue(h);
          if (pv > 21) res = "bust";
          else if (dealerBust) res = "dealer_bust";
          else if (pv > dv) res = "win";
          else if (pv < dv) res = "lose";
          else res = "push";
        }
        if (res === "blackjack") delta += bet + Math.floor((bet * 3) / 2);
        else if (res === "win" || res === "dealer_bust") delta += bet * 2;
        else if (res === "push") delta += bet;
        this.results.push(res);
      });
      this.chips += delta;
      if (this.chips > this.best) this.best = this.chips;
      this.score = this.best;
      this._save();
      this.state = PAYOUT;
      if (this.results.some((r) => r === "blackjack")) this.playSound("win");
      else if (this.results.some((r) => r === "win" || r === "dealer_bust")) this.playSound("point");
      else if (this.results.every((r) => r === "push")) this.playSound("select");
      else this.playSound("gameover");
    }

    _toBet() {
      this.bet = 0;
      if (this.chips < BETS[0]) {
        this.state = BROKE;
        this.playSound("gameover");
      } else {
        this.state = BET;
      }
    }

    // ===================================================== Eingabe
    _isConfirm(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    handleEvent(ev) {
      if (this.state === BROKE) {
        if (this._isConfirm(ev)) {
          this.chips = START_CHIPS;
          this._save();
          this._toBet();
          this.playSound("click");
        }
        return;
      }
      if (this.state === BET) this._handleBet(ev);
      else if (this.state === PLAYER) this._handlePlayer(ev);
      else if (this.state === PAYOUT) {
        if (this._isConfirm(ev)) this._toBet();
      }
    }

    _handleBet(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4"].includes(k)) {
          this._addChip(BETS[Number(k) - 1]);
        } else if (k === "BackSpace") {
          // Einsatz zurücknehmen (wird erst beim Geben abgezogen)
          this.bet = 0;
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this._startDeal();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.chipRects.length; i++) {
          if (this.chipRects[i].collidepoint(ev.pos)) {
            this._addChip(BETS[i]);
            return;
          }
        }
        if (this.clearRect.collidepoint(ev.pos)) {
          this.bet = 0;
          this.playSound("move");
        } else if (this.dealRect.collidepoint(ev.pos)) {
          this._startDeal();
        }
      }
    }

    _addChip(value) {
      if (this.bet + value <= this.chips) {
        this.bet += value;
        this.playSound("click");
      }
    }

    _handlePlayer(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "h" || k === "H") this._hit();
        else if (k === "s" || k === "S") this._stand();
        else if (k === "d" || k === "D") this._double();
        else if (k === "x" || k === "X") this._split();
      } else if (ev.kind === "mousedown") {
        for (const key in this.actionRects) {
          if (this.actionRects[key].collidepoint(ev.pos)) {
            if (key === "hit") this._hit();
            else if (key === "stand") this._stand();
            else if (key === "double" && this._canDouble()) this._double();
            else if (key === "split" && this._canSplit()) this._split();
            return;
          }
        }
      }
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === DEALING) {
        let done = true;
        for (const tw of this.tweens) {
          tw.t += dt;
          if (tw.t >= tw.delay + DEAL_T) {
            if (!tw.landed) {
              tw.landed = true;
              this.playSound("click");
            }
          } else {
            done = false;
          }
        }
        if (done) {
          this.tweens = [];
          this._afterDeal();
        }
      } else if (this.state === DEALER) {
        if (this.flipT > 0) {
          this.flipT -= dt;
          return;
        }
        this.dealerWait -= dt;
        if (this.dealerWait <= 0) this._dealerStep();
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      C.blitFelt(ctx, this, this.width, this.height, COL_FELT, COL_FELT_EDGE);

      // Schuh oben rechts
      this.renderer.drawBack(ctx, this.shoePos[0], this.shoePos[1], this.cw, this.ch);
      ui.text(ctx, String(this.shoe.length), this.shoePos[0], this.shoePos[1] + this.ch + 4, this._tiny, ui.TEXT_DIM);

      // Karten, die gerade vom Schuh fliegen, im Stapel auslassen
      this._fly = this.state === DEALING ? this._flyingCards() : new Set();
      this._drawDealer(ctx);
      this._drawPlayer(ctx);
      if (this._fly.size) this._drawFlying(ctx);
      this._drawTopbar(ctx);

      if (this.state === BET) this._drawBetUi(ctx);
      else if (this.state === PLAYER) this._drawActionUi(ctx);
      else if (this.state === PAYOUT) this._drawResults(ctx);
      else if (this.state === BROKE) this._drawBroke(ctx);
    }

    /** Set der Karten, die noch nicht gelandet sind. */
    _flyingCards() {
      const s = new Set();
      for (const tw of this.tweens) if (tw.t < tw.delay + DEAL_T) s.add(tw.card);
      return s;
    }

    /** Karten-Tween: vom Schuh mit Ease-Out an den Zielplatz fliegen. */
    _drawFlying(ctx) {
      for (const tw of this.tweens) {
        const p = (tw.t - tw.delay) / DEAL_T;
        if (p <= 0 || p >= 1) continue; // noch im Schuh bzw. gelandet
        const e = 1 - (1 - p) ** 2;
        const [tx, ty] = tw.dealer ? this._dealerPositions(this.dealer.length)[tw.idx] : this._handPositions(0, this.hands[0].length)[tw.idx];
        const [sx, sy] = this.shoePos;
        this.renderer.draw(ctx, tw.card, Math.floor(sx + (tx - sx) * e), Math.floor(sy + (ty - sy) * e), this.cw, this.ch);
      }
    }

    _drawDealer(ctx) {
      ui.text(ctx, t("bj.dealer"), this.width / 2, this.dealerY - 8, this._small, ui.TEXT_DIM, "midbottom");
      const pos = this._dealerPositions(this.dealer.length);
      this.dealer.forEach((card, i) => {
        if (this._fly.has(card)) return;
        const [x, y] = pos[i];
        if (i === 1 && this.state === DEALER && this.flipT > 0) {
          // Flip-Animation: Breite skaliert (Rücken -> Vorderseite)
          const f = this.flipT / FLIP_T;
          const w = Math.max(2, Math.floor(this.cw * Math.abs(2 * f - 1)));
          const img = f > 0.5 ? this.renderer.back(this.cw, this.ch) : this.renderer.get(card, this.cw, this.ch);
          ctx.drawImage(img, x + Math.floor((this.cw - w) / 2), y, w, this.ch);
        } else if (i === 1 && this.holeHidden) {
          this.renderer.drawBack(ctx, x, y, this.cw, this.ch);
        } else {
          this.renderer.draw(ctx, card, x, y, this.cw, this.ch);
        }
      });
      if (this.dealer.length && !this.holeHidden && !(this.state === DEALER && this.flipT > 0)) {
        const [dv] = handValue(this.dealer);
        ui.text(ctx, String(dv), pos[pos.length - 1][0] + this.cw + 12, this.dealerY + 4, this._big, dv > 21 ? ui.RED : ui.TEXT);
      }
    }

    _drawPlayer(ctx) {
      this.hands.forEach((h, hi) => {
        const pos = this._handPositions(hi, h.length);
        h.forEach((card, i) => {
          if (this._fly.has(card)) return;
          this.renderer.draw(ctx, card, pos[i][0], pos[i][1], this.cw, this.ch);
        });
        if (h.length && this.state !== DEALING) {
          const [total, soft] = handValue(h);
          const txt = String(total) + (soft && total <= 21 ? "s" : "");
          const col = total > 21 ? ui.RED : total === 21 ? ui.GOLD : ui.TEXT;
          ui.text(ctx, txt, pos[pos.length - 1][0] + this.cw + 12, this.playerY + 4, this._big, col);
          ui.text(ctx, t("bj.bet") + ": " + this.handBets[hi], pos[0][0], this.playerY + this.ch + 6, this._tiny, ui.TEXT_DIM);
        }
        // Pfeil auf aktive Hand (bei Split)
        if (this.hands.length > 1 && hi === this.active && this.state === PLAYER) {
          const k = Math.abs((ui.ticks() % 800) - 400) / 400;
          const ax = pos[0][0] - 18;
          const ay = this.playerY + Math.floor(this.ch / 2) + Math.floor(6 * k);
          draw.polygon(ctx, ui.GOLD, [[ax, ay - 8], [ax + 12, ay], [ax, ay + 8]]);
        }
      });
    }

    _drawTopbar(ctx) {
      ui.text(ctx, t("bj.chips") + ": " + this.chips, 16, 12, this._big, ui.GOLD);
      ui.text(ctx, t("bj.best") + ": " + this.best, 16, 40, this._small, ui.TEXT_DIM);
    }

    /** Halbtransparente Bedienleiste unten mit Akzent-Oberkante. */
    _drawStrip(ctx) {
      draw.rect(ctx, [8, 22, 16, 224], this.strip);
      draw.line(ctx, this.accent, this.strip.topleft, this.strip.topright, 2);
    }

    /** Button im Theme-Look; primary = Akzentrahmen mit sanftem Puls. */
    _drawBtn(ctx, rect, label, primary = false, on = true) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rect, 0, 10);
      if (primary && on) {
        const border = ui.mix(this.accent, [255, 255, 255], 0.25 * ui.pulse(2.2, 0.0, 1.0));
        draw.rect(ctx, border, rect, 2, 10);
      } else {
        draw.rect(ctx, ui.BORDER_LIGHT, rect, 1, 10);
      }
      ui.text(ctx, label, rect.centerx, rect.centery, this._small, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }

    _drawBetUi(ctx) {
      this._drawStrip(ctx);
      this.chipRects.forEach((r, i) => {
        const val = BETS[i];
        draw.circle(ctx, CHIP_COLS[val], r.center, 22);
        draw.circle(ctx, ui.TEXT, r.center, 22, 2);
        draw.circle(ctx, ui.TEXT, r.center, 15, 1);
        ui.text(ctx, String(val), r.centerx, r.centery, this._tiny, val !== 100 ? [20, 24, 30] : ui.TEXT, "center");
      });
      ui.text(ctx, t("bj.bet") + ": " + this.bet, this.width / 2, this.strip.centery, this._big, ui.TEXT, "center");
      this._drawBtn(ctx, this.clearRect, t("bj.clear"), false, this.bet > 0);
      this._drawBtn(ctx, this.dealRect, t("bj.deal"), true, this.bet >= BETS[0]);
      if (this.bet < BETS[0]) {
        ui.text(ctx, t("bj.min_bet", { n: BETS[0] }), this.width / 2, this.strip.y - 6, this._tiny, ui.TEXT_DIM, "midbottom");
      }
    }

    _drawActionUi(ctx) {
      this._drawStrip(ctx);
      const avail = { hit: true, stand: true, double: this._canDouble(), split: this._canSplit() };
      const keys = { hit: "H", stand: "S", double: "D", split: "X" };
      for (const key in this.actionRects) {
        this._drawBtn(ctx, this.actionRects[key], t("bj." + key) + " (" + keys[key] + ")", false, avail[key]);
      }
    }

    _drawResults(ctx) {
      this._drawStrip(ctx);
      const texts = this.results.map((res, i) => {
        let label = t("bj." + res);
        if (this.results.length > 1) label = t("bj.hand") + " " + (i + 1) + ": " + label;
        return label;
      });
      ui.text(ctx, texts.join("   ·   "), this.width / 2, this.strip.centery - 10, this._big, ui.GOLD, "center");
      ui.text(ctx, t("common.enter_restart"), this.width / 2, this.strip.bottom - 14, this._tiny, ui.TEXT_FAINT, "center");
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 14, 10, 200], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      const pw = Math.min(this.width - 60, 460);
      ui.drawPanel(ctx, new PG.Rect(cx - pw / 2, cy - 92, pw, 184), { accentTop: ui.RED });
      ui.text(ctx, t("bj.broke"), cx, cy - 42, this._huge, ui.RED, "center");
      ui.text(ctx, t("bj.best") + ": " + this.best, cx, cy + 2, this._small, ui.TEXT_DIM, "center");
      ui.text(ctx, t("bj.broke_restart", { n: START_CHIPS }), cx, cy + 34, this._small, ui.TEXT, "center");
    }
  }

  PG.register(BlackjackGame, {
    id: "BlackjackGame",
    key: "blackjack",
    name: "Blackjack",
  });
})();
