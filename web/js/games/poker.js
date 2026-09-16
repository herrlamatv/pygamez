/*
 * poker.js - Poker (Port von games/poker.py)
 * ===========================================
 * Drei wählbare Varianten (Modusauswahl im Vorspiel):
 *
 * - Texas Hold'em: gegen 1-3 KI-Gegner, mit Dealer-Button, Small/Big Blind und
 *   vier Setzrunden (Preflop, Flop, Turn, River). Aktionen: Fold, Check, Call,
 *   Raise, All-In. Beim Showdown gewinnt die beste 5-aus-7-Hand.
 * - 5 Card Draw: Heads-up gegen die KI. Ante, eine Setzrunde, Karten tauschen,
 *   zweite Setzrunde, Showdown.
 * - Video Poker (Jacks or Better): Solo gegen die Auszahlungstabelle. Einsatz,
 *   fünf Karten, Halten wählen, ziehen, Auszahlung nach Tabelle.
 *
 * Lama-Chips: Blackjack, Poker und Casino teilen sich ein Konto bei der
 * Lama-Bank (casino_bank.js, PG.store "mem.casino"). Zu Beginn einer Hand
 * liegt der ganze Kontostand als Tischstapel im "escrow"; was in den Pot geht,
 * ist sofort abgebucht. Wer den Tisch mitten in der Hand verlässt, verliert
 * seinen Pot-Anteil, der Reststapel kommt zurück aufs Konto. Highscore =
 * Höchststand von 1000 + Poker-Bilanz (gameOver wird nie gesetzt). Pleite
 * (unter dem Big Blind bzw. 10 im Video Poker) = Bank-Kredit auf 1000.
 *
 * Vereinfachung: Bei All-In wird EIN gemeinsamer Haupt-Pot geführt (keine
 * Side-Pots) - für ein lockeres Spiel gegen die KI völlig ausreichend.
 *
 * Web: Gegnerzahl (nur Hold'em) und KI-Stärke lassen sich vor jeder Hand
 * direkt am Tisch umschalten (Einstellungs-Abschnitt "poker").
 *
 * Steuerung: Buttons anklicken oder F = Fold, C = Check/Call, R = Raise,
 * A = All-In, Enter/Leertaste = Geben/Weiter, 1-5 = Karte halten/tauschen.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const C = PG.cards;

  // ---- Tisch-Identität (grüner Filz, leicht entsättigt).
  //      Alle generischen UI-Farben kommen zur Zeichenzeit aus ui.* (Theme),
  //      die Akzentfarbe aus this.accent.
  const COL_FELT = [20, 50, 39];
  const COL_FELT_EDGE = [13, 35, 27];

  const bank = PG.lamabank;
  const SMALL_BLIND = 10;
  const BIG_BLIND = 20;
  const ANTE = 10;
  const VIDEO_BETS = [10, 25, 50];

  // Kategorien (höher = besser)
  const HIGH = 0, PAIR = 1, TWO_PAIR = 2, TRIPS = 3, STRAIGHT = 4, FLUSH = 5, FULL_HOUSE = 6, QUADS = 7, STR_FLUSH = 8;
  const CAT_KEY = ["high_card", "pair", "two_pair", "trips", "straight", "flush", "full_house", "quads", "straight_flush"];

  // Video-Poker-Auszahlung (Jacks or Better), Multiplikator auf den Einsatz.
  const VIDEO_PAYOUT = [
    [STR_FLUSH, "royal", 250], // Sonderfall Royal Flush (unten geprüft)
    [STR_FLUSH, null, 50],
    [QUADS, null, 25],
    [FULL_HOUSE, null, 9],
    [FLUSH, null, 6],
    [STRAIGHT, null, 4],
    [TRIPS, null, 3],
    [TWO_PAIR, null, 2],
    [PAIR, "jacks", 1], // nur Buben oder besser
  ];

  function val(card) {
    return card.rank === 1 ? 14 : card.rank;
  }

  /** Lexikografischer Vergleich zweier Bewertungen (wie Python-Tupel). */
  function cmpRank(a, b) {
    const n = Math.min(a.length, b.length);
    for (let i = 0; i < n; i++) if (a[i] !== b[i]) return a[i] - b[i];
    return a.length - b.length;
  }

  /** Bewertet genau 5 Karten -> vergleichbares (flaches) Array, höher = besser.
   *  Python-Tupel wie (FLUSH, (v1..v5)) werden flach als [FLUSH, v1..v5] geführt. */
  function eval5(cards) {
    const vals = cards.map(val).sort((a, b) => b - a);
    const counts = new Map();
    for (const v of vals) counts.set(v, (counts.get(v) || 0) + 1);
    // (Wert, Anzahl) nach Anzahl, dann Wert absteigend -> Paare/Drillinge etc.
    const byCount = [...counts.entries()].sort((a, b) => b[1] - a[1] || b[0] - a[0]);
    const isFlush = cards.every((c) => c.suit === cards[0].suit);
    const uniq = [...counts.keys()].sort((a, b) => b - a);
    let straightHigh = 0;
    if (uniq.length === 5) {
      if (uniq[0] - uniq[4] === 4) straightHigh = uniq[0];
      else if (uniq.join(",") === "14,5,4,3,2") straightHigh = 5; // Wheel (A-2-3-4-5)
    }

    if (isFlush && straightHigh) return [STR_FLUSH, straightHigh];
    if (byCount[0][1] === 4) {
      const quad = byCount[0][0];
      return [QUADS, quad, Math.max(...vals.filter((v) => v !== quad))];
    }
    if (byCount[0][1] === 3 && byCount[1][1] === 2) return [FULL_HOUSE, byCount[0][0], byCount[1][0]];
    if (isFlush) return [FLUSH, ...vals];
    if (straightHigh) return [STRAIGHT, straightHigh];
    if (byCount[0][1] === 3) return [TRIPS, byCount[0][0], ...vals.filter((v) => v !== byCount[0][0])];
    if (byCount[0][1] === 2 && byCount[1][1] === 2) {
      const hp = byCount[0][0], lp = byCount[1][0];
      return [TWO_PAIR, hp, lp, Math.max(...vals.filter((v) => v !== hp && v !== lp))];
    }
    if (byCount[0][1] === 2) return [PAIR, byCount[0][0], ...vals.filter((v) => v !== byCount[0][0])];
    return [HIGH, ...vals];
  }

  /** Beste 5-Karten-Bewertung aus 5..7 Karten -> [rang, beste5]. */
  function bestHand(cards) {
    if (cards.length <= 5) return [eval5(cards), cards.slice()];
    let best = null, best5 = null;
    const n = cards.length;
    for (let a = 0; a < n; a++)
      for (let b = a + 1; b < n; b++)
        for (let c = b + 1; c < n; c++)
          for (let d = c + 1; d < n; d++)
            for (let e = d + 1; e < n; e++) {
              const combo = [cards[a], cards[b], cards[c], cards[d], cards[e]];
              const r = eval5(combo);
              if (best === null || cmpRank(r, best) > 0) {
                best = r;
                best5 = combo;
              }
            }
    return [best, best5];
  }

  function isRoyal(rt) {
    return rt[0] === STR_FLUSH && rt[1] === 14;
  }

  function categoryKey(rt) {
    return isRoyal(rt) ? "royal" : CAT_KEY[rt[0]];
  }

  /** Grober Stärke-Score 0..1 für die KI (Kategorie + höchste Karte). */
  function handStrength(cards) {
    const [rt] = bestHand(cards);
    const top = rt.length > 1 ? rt[1] : 10;
    return Math.min(1.0, (rt[0] / 8.0) * 0.82 + (top / 14.0) * 0.18);
  }

  /** Hole-Card-Stärke 0..1 für Texas Hold'em (Chen-artig, normiert). */
  function preflopStrength(hole) {
    const [a, b] = hole;
    const va = val(a), vb = val(b);
    const hi = Math.max(va, vb), lo = Math.min(va, vb);
    let score = (hi / 14.0) * 0.5;
    if (va === vb) {
      // Paar
      score = 0.5 + (va / 14.0) * 0.5;
    } else {
      if (a.suit === b.suit) score += 0.12;
      const gap = hi - lo;
      if (gap === 1) score += 0.08;
      else if (gap === 2) score += 0.04;
      score += (lo / 14.0) * 0.15;
    }
    return Math.max(0.0, Math.min(1.0, score));
  }

  class Player {
    constructor(name, stack, isHuman = false) {
      this.name = name;
      this.stack = stack;
      this.hole = [];
      this.folded = false;
      this.allIn = false;
      this.roundBet = 0;
      this.isHuman = isHuman;
      this.result = null;
    }
  }

  // Phasen
  const PREHAND = "prehand", BET_VIDEO = "bet_video", ACTING = "acting", DRAW_SELECT = "draw_select",
    VIDEO_HOLD = "video_hold", SHOWDOWN = "showdown", BROKE = "broke";

  const DIFF_KEYS = ["tank.diff.easy", "tank.diff.medium", "tank.diff.hard"];

  function toInt(v, def) {
    const n = parseInt(v, 10);
    return Number.isFinite(n) ? n : def;
  }

  class PokerGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      if (!["holdem", "draw", "video"].includes(this.mode)) this.mode = "holdem";

      // load() erstattet auch den Tischstapel einer abgebrochenen Hand.
      bank.load();
      this.chips = bank.balance();
      this.best = bank.scoreFor("poker");
      this.score = this.best;

      this._readOpts();

      this._makeFonts();
      this.renderer = new C.CardRenderer(this.accent);

      this.deck = [];
      this.community = [];
      this.pot = 0;
      this.players = [];
      this.dealer = 0;
      this.turn = 0;
      this.currentBet = 0;
      this.minRaise = BIG_BLIND;
      this.toActSet = new Set();
      this.street = "preflop";
      this.msg = "";
      this.actDelay = 0;
      this.reveal = false;
      this.holds = new Set(); // gehaltene/zu behaltende Karten (video/draw)
      this.videoBet = VIDEO_BETS[0];
      this.winAmount = 0;
      this.rankShown = null;
      this.winnerNames = [];
      this._layout();

      this.phase = this.mode === "video" ? BET_VIDEO : PREHAND;
      if (bank.isBroke("poker", this.mode)) this.phase = BROKE;
    }

    _readOpts() {
      this.nOpponents = Math.max(1, Math.min(3, toInt(this.opts.opponents, 2)));
      this.aiLevel = Math.max(0, Math.min(2, toInt(this.opts.difficulty, 1)));
      if (this.mode === "draw") this.nOpponents = 1;
      else if (this.mode === "video") this.nOpponents = 0;
    }

    /** Theme-Schriften (ui.font cached selbst); _huge hängt an height. */
    _makeFonts() {
      this._small = ui.font(15);
      this._tiny = ui.font(13);
      this._big = ui.font(22, true);
      this._huge = ui.font(Math.max(24, Math.floor(this.height / 12)), true);
    }

    _layout() {
      const W = this.width, H = this.height;
      this.ch = Math.floor(H * 0.19);
      this.cw = Math.floor(this.ch * 0.72);
      this.strip = new PG.Rect(0, H - 66, W, 66);
      // Aktions-Buttons
      const bw = Math.floor(W * 0.17);
      const gap = Math.floor((W - 4 * bw) / 5);
      this.actionRects = {};
      ["fold", "call", "raise", "allin"].forEach((key, i) => {
        this.actionRects[key] = new PG.Rect(gap + i * (bw + gap), this.strip.y + 12, bw, 42);
      });
      // Deal-/Weiter-Button (mittig)
      this.dealRect = new PG.Rect(Math.floor(W / 2) - 90, this.strip.y + 12, 180, 42);
      // Video/Draw-Bet-Chips
      this.chipRects = VIDEO_BETS.map((_, i) => new PG.Rect(24 + i * 54, this.strip.y + 12, 42, 42));
      // Web: Optionen im Vorspiel-Tisch (Gegner / KI-Stärke)
      const ow = 260;
      this.optRects = {
        opponents: new PG.Rect(Math.floor(W / 2) - ow / 2, Math.floor(H * 0.52), ow, 38),
        difficulty: new PG.Rect(Math.floor(W / 2) - ow / 2, Math.floor(H * 0.52) + 48, ow, 38),
      };
    }

    /** Tisch verlassen: Reststapel sofort zurück aufs Konto. */
    destroy() {
      bank.clearEscrow();
    }

    /** Kontostand, Highscore und Chipleader-Erfolg aus der Lama-Bank. */
    _syncChips() {
      this.chips = bank.balance();
      this.best = bank.scoreFor("poker");
      this.score = this.best;
      // poker_rich zählt die Poker-Bilanz, nicht das gemeinsame Konto.
      this.achEvent("poker_rich", bank.valueFor("poker"));
    }

    // ===================================================== Deck
    _freshDeck() {
      this.deck = C.makeDeck();
      C.shuffle(this.deck, new PG.Random());
    }

    _draw(faceUp = true) {
      const card = this.deck.pop();
      card.faceUp = faceUp;
      return card;
    }

    // ===================================================== Hand-Start
    _startHand() {
      bank.clearEscrow();
      this.chips = bank.balance();
      if (bank.isBroke("poker", this.mode)) {
        this.phase = BROKE;
        this.playSound("gameover");
        return;
      }
      this._freshDeck();
      this.community = [];
      this.pot = 0;
      this.reveal = false;
      this.holds = new Set();
      this.msg = "";
      this.winAmount = 0;
      this.players = [new Player(t("poker.you"), this.chips, true)];
      for (let i = 0; i < this.nOpponents; i++) {
        this.players.push(new Player(t("poker.cpu", { n: i + 1 }), Math.max(BIG_BLIND * 20, this.chips)));
      }
      // Der ganze Kontostand liegt jetzt als Tischstapel auf dem Tisch.
      bank.setEscrow(this.chips);
      if (this.mode === "holdem") this._startHoldem();
      else if (this.mode === "draw") this._startDraw();
    }

    // ----- Texas Hold'em -----------------------------------------------
    _startHoldem() {
      const n = this.players.length;
      this.dealer = this.dealer % n;
      for (let k = 0; k < 2; k++) for (const p of this.players) p.hole.push(this._draw(p.isHuman));
      let sb = (this.dealer + 1) % n;
      let bb = (this.dealer + 2) % n;
      if (n === 2) {
        // Heads-up: Dealer = SB
        sb = this.dealer;
        bb = (this.dealer + 1) % n;
      }
      this._post(this.players[sb], SMALL_BLIND);
      this._post(this.players[bb], BIG_BLIND);
      this.currentBet = BIG_BLIND;
      this.minRaise = BIG_BLIND;
      this.street = "preflop";
      this.toActSet = new Set();
      this.players.forEach((p, i) => { if (!p.allIn) this.toActSet.add(i); });
      this.turn = (bb + 1) % n;
      this.phase = ACTING;
      this.msg = t("poker.preflop");
      this.playSound("move");
      this._beginTurn();
    }

    _post(player, amount) {
      amount = Math.min(amount, player.stack);
      // Einsatz des Menschen sofort abbuchen (aus dem Tischstapel) + speichern.
      if (player.isHuman && amount > 0) bank.payFromEscrow(amount, "poker");
      player.stack -= amount;
      player.roundBet += amount;
      this.pot += amount;
      if (player.stack === 0) player.allIn = true;
    }

    // ----- 5 Card Draw --------------------------------------------------
    _startDraw() {
      for (const p of this.players) this._post(p, ANTE); // Ante von allen
      for (let k = 0; k < 5; k++) for (const p of this.players) p.hole.push(this._draw(p.isHuman));
      this.currentBet = 0;
      this.minRaise = BIG_BLIND;
      this.street = "draw1";
      this.toActSet = new Set();
      this.players.forEach((p, i) => { if (!p.allIn) this.toActSet.add(i); });
      this.turn = (this.dealer + 1) % this.players.length;
      this.phase = ACTING;
      this.msg = t("poker.round1");
      this.playSound("move");
      this._beginTurn();
    }

    // ===================================================== Setzrunde
    /** Setzt actDelay für die KI oder wartet auf den Menschen. */
    _beginTurn() {
      const contenders = this.players.filter((p) => !p.folded);
      if (contenders.length <= 1) {
        this._endHand();
        return;
      }
      if (!this.toActSet.size) {
        this._endRound();
        return;
      }
      if (!this.toActSet.has(this.turn)) {
        this._advanceTurn();
        return;
      }
      const p = this.players[this.turn];
      this.actDelay = p.isHuman ? 0.0 : 0.7;
    }

    _advanceTurn() {
      const n = this.players.length;
      if (!this.toActSet.size) {
        this._endRound();
        return;
      }
      for (let i = 1; i <= n; i++) {
        const cand = (this.turn + i) % n;
        if (this.toActSet.has(cand)) {
          this.turn = cand;
          this._beginTurn();
          return;
        }
      }
      this._endRound();
    }

    _toCall(player) {
      return Math.max(0, this.currentBet - player.roundBet);
    }

    /** Alle anderen aktiven Spieler müssen erneut handeln. */
    _reopen() {
      this.toActSet = new Set();
      this.players.forEach((q, i) => { if (!q.folded && !q.allIn && i !== this.turn) this.toActSet.add(i); });
    }

    _doAction(action, raiseTo = null) {
      const p = this.players[this.turn];
      if (action === "fold") {
        p.folded = true;
        this.toActSet.delete(this.turn);
        this.playSound("select");
      } else if (action === "check") {
        this.toActSet.delete(this.turn);
        this.playSound("click");
      } else if (action === "call") {
        this._post(p, Math.min(this._toCall(p), p.stack));
        this.toActSet.delete(this.turn);
        this.playSound("point");
      } else if (action === "raise") {
        let target = raiseTo !== null ? raiseTo : this.currentBet + this.minRaise;
        target = Math.min(target, p.roundBet + p.stack); // nicht mehr als Stack
        this._post(p, target - p.roundBet);
        this.minRaise = Math.max(this.minRaise, target - this.currentBet);
        this.currentBet = Math.max(this.currentBet, p.roundBet);
        this._reopen();
        this.playSound("point");
      } else if (action === "allin") {
        this._post(p, p.stack);
        if (p.roundBet > this.currentBet) {
          this.minRaise = Math.max(this.minRaise, p.roundBet - this.currentBet);
          this.currentBet = p.roundBet;
          this._reopen();
        } else {
          this.toActSet.delete(this.turn);
        }
        this.playSound("point");
      }
      this._advanceTurn();
    }

    _endRound() {
      for (const p of this.players) p.roundBet = 0;
      this.currentBet = 0;
      this.minRaise = BIG_BLIND;
      const contenders = this.players.filter((p) => !p.folded);
      if (contenders.length <= 1) {
        this._endHand();
        return;
      }
      if (this.mode === "holdem") this._nextStreet();
      else if (this.street === "draw1") this._enterDrawPhase();
      else this._showdown();
    }

    _nextStreet() {
      const actionable = this.players.filter((p) => !p.folded && !p.allIn);
      // Wenn <=1 handlungsfähig: restliche Karten aufdecken, Showdown.
      const fast = actionable.length <= 1;
      if (this.street === "preflop") {
        this.community = [this._draw(), this._draw(), this._draw()];
        this.street = "flop";
        this.msg = t("poker.flop");
      } else if (this.street === "flop") {
        this.community.push(this._draw());
        this.street = "turn";
        this.msg = t("poker.turn");
      } else if (this.street === "turn") {
        this.community.push(this._draw());
        this.street = "river";
        this.msg = t("poker.river");
      } else {
        this._showdown();
        return;
      }
      this.playSound("move");
      if (fast) {
        if (this.street !== "river") this._nextStreet();
        else this._showdown();
        return;
      }
      this.toActSet = new Set();
      this.players.forEach((p, i) => { if (!p.folded && !p.allIn) this.toActSet.add(i); });
      this.turn = this.dealer;
      this._advanceTurn();
    }

    // ----- Draw-Phase (Karten tauschen) ---------------------------------
    _enterDrawPhase() {
      this.phase = DRAW_SELECT;
      this.holds = new Set();
      this.msg = t("poker.draw_hint");
      this.actDelay = 0;
    }

    _doHumanDraw() {
      const me = this.players[0];
      me.hole = me.hole.map((card, i) => (this.holds.has(i) ? card : this._draw(true)));
      this.holds = new Set();
      this.playSound("rotate");
      // KI tauscht
      for (const p of this.players.slice(1)) {
        if (!p.folded) this._aiDraw(p);
      }
      // zweite Setzrunde
      this.currentBet = 0;
      this.minRaise = BIG_BLIND;
      this.street = "draw2";
      this.toActSet = new Set();
      this.players.forEach((p, i) => { if (!p.folded && !p.allIn) this.toActSet.add(i); });
      this.turn = (this.dealer + 1) % this.players.length;
      this.phase = ACTING;
      this.msg = t("poker.round2");
      this._beginTurn();
    }

    /** KI entscheidet, welche Karten sie behält (einfache Heuristik). */
    _aiDraw(p) {
      const counts = new Map();
      for (const c of p.hole) counts.set(val(c), (counts.get(val(c)) || 0) + 1);
      const suits = new Map();
      for (const c of p.hole) suits.set(c.suit, (suits.get(c.suit) || 0) + 1);
      let keep = new Set();
      // Paare/Drillinge/Vierlinge behalten
      p.hole.forEach((c, i) => { if (counts.get(val(c)) >= 2) keep.add(i); });
      // Flush-Ansatz (4 gleiche Farbe) behalten
      let fs = null;
      for (const [s, n] of suits) if (fs === null || n > fs[1]) fs = [s, n];
      if (fs[1] >= 4) {
        keep = new Set();
        p.hole.forEach((c, i) => { if (c.suit === fs[0]) keep.add(i); });
      }
      if (!keep.size) {
        // sonst hohe Karten behalten
        p.hole.forEach((c, i) => { if (val(c) >= 12) keep.add(i); });
      }
      p.hole = p.hole.map((c, i) => (keep.has(i) ? c : this._draw(false)));
    }

    // ===================================================== KI-Setzen
    _aiAction(p) {
      const toCall = this._toCall(p);
      let strength;
      if (this.mode === "holdem") {
        if (this.street === "preflop" && !this.community.length) strength = preflopStrength(p.hole);
        else strength = handStrength(p.hole.concat(this.community));
      } else {
        strength = handStrength(p.hole);
      }
      // Schwierigkeit steuert Aggressivität / Bluff.
      const aggro = [0.05, 0.14, 0.22][this.aiLevel];
      const bluff = [0.03, 0.06, 0.1][this.aiLevel];
      const r = PG.rand.random();

      if (toCall === 0) {
        // Check oder Setzen
        if (strength > 0.6 || r < aggro) this._aiRaise(p);
        else this._doAction("check");
        return;
      }
      // Es kostet etwas zu bleiben
      const potOdds = toCall / (this.pot + toCall + 1e-9);
      if (strength < 0.28 && r > bluff) {
        this._doAction(toCall > p.stack * 0.12 ? "fold" : "call");
        return;
      }
      if (strength > 0.72 && r < 0.5 + aggro) {
        this._aiRaise(p);
        return;
      }
      if (strength < potOdds - 0.05 && r > bluff) {
        this._doAction("fold");
        return;
      }
      this._doAction("call");
    }

    _aiRaise(p) {
      const raiseTo = this.currentBet + Math.max(this.minRaise, Math.floor(this.pot * 0.6) || BIG_BLIND);
      if (raiseTo >= p.roundBet + p.stack) this._doAction("allin");
      else this._doAction("raise", raiseTo);
    }

    // ===================================================== Showdown / Ende
    _showdown() {
      this.reveal = true;
      for (const p of this.players) for (const c of p.hole) c.faceUp = true;
      const ranked = [];
      for (const p of this.players) {
        if (p.folded) continue;
        const cards = this.mode === "holdem" ? p.hole.concat(this.community) : p.hole;
        const [rt] = bestHand(cards);
        p.result = rt;
        ranked.push([rt, p]);
      }
      let bestRt = ranked[0][0];
      for (const [rt] of ranked) if (cmpRank(rt, bestRt) > 0) bestRt = rt;
      const winners = ranked.filter(([rt]) => cmpRank(rt, bestRt) === 0).map(([, p]) => p);
      this._award(winners, bestRt);
      this.phase = SHOWDOWN;
    }

    /** Nur noch ein Spieler übrig (alle anderen gefoldet). */
    _endHand() {
      this._award(this.players.filter((p) => !p.folded), null);
      this.phase = SHOWDOWN;
    }

    _award(winners, rankTuple) {
      const share = Math.floor(this.pot / winners.length);
      const rem = this.pot - share * winners.length;
      let humanWin = 0;
      winners.forEach((w, i) => {
        w.stack += share + (i === 0 ? rem : 0);
        if (w.isHuman) humanWin = share + (i === 0 ? rem : 0);
      });
      // Tischstapel zurück aufs Konto + Pot-Anteil gutschreiben
      bank.settleEscrow(humanWin, "poker");
      const me = this.players[0];
      const won = winners.includes(me);
      this.winAmount = won ? this.pot : 0;
      if (won) {
        this.msg = rankTuple !== null ? t("poker.you_win_hand", { hand: t("poker.hand." + categoryKey(rankTuple)) }) : t("poker.you_win");
        this.playSound("win");
      } else {
        this.msg = t("poker.you_lose");
        this.playSound("gameover");
      }
      this.reveal = true;
      this.rankShown = rankTuple;
      this.winnerNames = winners.map((w) => w.name);
      this.dealer = (this.dealer + 1) % Math.max(1, this.players.length);
      this._syncChips();
    }

    // ----- Video Poker --------------------------------------------------
    _videoDeal() {
      if (!bank.debit(this.videoBet, "poker")) {
        if (bank.isBroke("poker", "video")) {
          this.phase = BROKE;
          this.playSound("gameover");
        }
        return;
      }
      this.chips = bank.balance();
      this._freshDeck();
      this.players = [new Player(t("poker.you"), this.chips, true)];
      this.players[0].hole = [0, 1, 2, 3, 4].map(() => this._draw(true));
      this.holds = new Set();
      this.phase = VIDEO_HOLD;
      this.msg = t("poker.hold_hint");
      this.winAmount = 0;
      this.playSound("move");
    }

    _videoDraw() {
      const me = this.players[0];
      me.hole = me.hole.map((c, i) => (this.holds.has(i) ? c : this._draw(true)));
      const [rt] = bestHand(me.hole);
      const mult = this._videoPayout(rt);
      this.winAmount = this.videoBet * mult;
      bank.credit(this.winAmount, "poker");
      this._syncChips();
      if (mult > 0) {
        this.msg = t("poker.video_win", { hand: t("poker.hand." + categoryKey(rt)), mult });
        this.playSound(mult >= 6 ? "win" : "point");
      } else {
        this.msg = t("poker.video_none");
        this.playSound("gameover");
      }
      this.rankShown = rt;
      this.phase = SHOWDOWN;
    }

    _videoPayout(rt) {
      for (const [cat, special, mult] of VIDEO_PAYOUT) {
        if (rt[0] !== cat) continue;
        if (special === "royal") {
          if (isRoyal(rt)) return mult;
          continue;
        }
        if (special === "jacks") {
          if (rt[0] === PAIR && rt[1] >= 11) return mult;
          continue;
        }
        return mult;
      }
      return 0;
    }

    // ===================================================== Update
    update(dt) {
      if (this.phase === ACTING) {
        const p = this.players[this.turn];
        if (!p.isHuman && !p.folded && !p.allIn) {
          this.actDelay -= dt;
          if (this.actDelay <= 0) this._aiAction(p);
        }
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.phase === BROKE) {
        if (this._isConfirm(ev)) {
          bank.refillIfBroke("poker", this.mode);
          this.chips = bank.balance();
          this.phase = this.mode === "video" ? BET_VIDEO : PREHAND;
          this.playSound("click");
        }
        return;
      }
      if (this.phase === PREHAND) {
        if (this._handleOptions(ev)) return;
        if (this._isConfirm(ev)) this._startHand();
        return;
      }
      if (this.phase === BET_VIDEO) {
        this._handleVideoBet(ev);
        return;
      }
      if (this.phase === VIDEO_HOLD) {
        this._handleHolds(ev, () => this._videoDraw());
        return;
      }
      if (this.phase === DRAW_SELECT) {
        this._handleHolds(ev, () => this._doHumanDraw());
        return;
      }
      if (this.phase === SHOWDOWN) {
        if (this._isConfirm(ev)) this._nextAfterShowdown();
        return;
      }
      if (this.phase === ACTING) this._handleActing(ev);
    }

    _isConfirm(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    /** Web: sichtbare Optionen vor der Hand (Gegnerzahl nur Hold'em, KI-Stärke). */
    _optionKeys() {
      if (this.mode === "holdem") return ["opponents", "difficulty"];
      if (this.mode === "draw") return ["difficulty"];
      return [];
    }

    _handleOptions(ev) {
      if (ev.kind !== "mousedown") return false;
      for (const key of this._optionKeys()) {
        if (this.optRects[key].collidepoint(ev.pos)) {
          if (key === "opponents") this.opts.opponents = (toInt(this.opts.opponents, 2) % 3) + 1;
          else this.opts.difficulty = (toInt(this.opts.difficulty, 1) + 1) % 3;
          this.saveSettings();
          this._readOpts();
          this.playSound("select");
          return true;
        }
      }
      return false;
    }

    _nextAfterShowdown() {
      for (const p of this.players) {
        p.hole = [];
        p.folded = false;
        p.allIn = false;
        p.roundBet = 0;
        p.result = null;
      }
      this.chips = bank.balance();
      if (bank.isBroke("poker", this.mode)) this.phase = BROKE;
      else this.phase = this.mode === "video" ? BET_VIDEO : PREHAND;
      if (this.phase === BROKE) this.playSound("gameover");
    }

    _handleVideoBet(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3"].includes(k) && Number(k) <= VIDEO_BETS.length) {
          this.videoBet = VIDEO_BETS[Number(k) - 1];
          this.playSound("click");
        } else if (k === "Return" || k === "space") {
          this._videoDeal();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.chipRects.length; i++) {
          if (this.chipRects[i].collidepoint(ev.pos)) {
            this.videoBet = VIDEO_BETS[i];
            this.playSound("click");
            return;
          }
        }
        if (this.dealRect.collidepoint(ev.pos)) this._videoDeal();
      }
    }

    _toggleHold(i) {
      if (this.holds.has(i)) this.holds.delete(i);
      else this.holds.add(i);
      this.playSound("click");
    }

    _handleHolds(ev, onConfirm) {
      const me = this.players[0];
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4", "5"].includes(k)) {
          const i = Number(k) - 1;
          if (i < me.hole.length) this._toggleHold(i);
        } else if (k === "Return" || k === "space") {
          onConfirm();
        }
      } else if (ev.kind === "mousedown") {
        // WICHTIG: dieselben Rechtecke wie beim Zeichnen verwenden.
        const rects = this._holeRects(me);
        for (let i = 0; i < rects.length; i++) {
          if (rects[i].collidepoint(ev.pos)) {
            this._toggleHold(i);
            return;
          }
        }
        if (this.dealRect.collidepoint(ev.pos)) onConfirm();
      }
    }

    _handleActing(ev) {
      const p = this.players[this.turn];
      if (!p.isHuman) return;
      const toCall = this._toCall(p);
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "f" || k === "F") this._doAction("fold");
        else if (k === "c" || k === "C") this._doAction(toCall === 0 ? "check" : "call");
        else if (k === "r" || k === "R") this._humanRaise();
        else if (k === "a" || k === "A") this._doAction("allin");
      } else if (ev.kind === "mousedown") {
        const R = this.actionRects;
        if (R.fold.collidepoint(ev.pos)) this._doAction("fold");
        else if (R.call.collidepoint(ev.pos)) this._doAction(toCall === 0 ? "check" : "call");
        else if (R.raise.collidepoint(ev.pos)) this._humanRaise();
        else if (R.allin.collidepoint(ev.pos)) this._doAction("allin");
      }
    }

    _humanRaise() {
      const p = this.players[this.turn];
      const raiseTo = this.currentBet + Math.max(this.minRaise, Math.floor(this.pot * 0.5) || BIG_BLIND);
      if (raiseTo >= p.roundBet + p.stack) this._doAction("allin");
      else this._doAction("raise", raiseTo);
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      C.blitFelt(ctx, this, this.width, this.height, COL_FELT, COL_FELT_EDGE);
      this._drawTopbar(ctx);
      if (this.phase === BROKE) {
        this._drawBroke(ctx);
        return;
      }
      if (this.phase === BET_VIDEO) {
        this._drawVideoBet(ctx);
        return;
      }
      if (this.phase === PREHAND || !this.players.length) {
        this._drawPrehand(ctx);
        return;
      }
      if (this.mode === "video") this._drawVideo(ctx);
      else this._drawTable(ctx);
    }

    /** Mitten in einer Hand liegt das Konto als Tischstapel auf dem Tisch. */
    _hudChips() {
      if (this.mode !== "video" && this.players.length && (this.phase === ACTING || this.phase === DRAW_SELECT)) {
        return this.players[0].stack;
      }
      return this.chips;
    }

    _drawTopbar(ctx) {
      const chipsLbl = t("poker.chips") + ": " + this._hudChips();
      // "Lama-Chips" ist in manchen Sprachen lang - Pot-Anzeige freihalten
      ui.text(ctx, chipsLbl, 16, 12, this._big.width(chipsLbl) > this.width / 2 - 90 ? this._small : this._big, ui.GOLD);
      ui.text(ctx, t("poker.best") + ": " + this.best, 16, 40, this._small, ui.TEXT_DIM);
      ui.text(ctx, t("poker.mode." + this.mode), this.width - 16, 14, this._small, this.accent, "topright");
      if (this.pot && this.phase !== BET_VIDEO && this.phase !== PREHAND) {
        ui.text(ctx, t("poker.pot") + ": " + this.pot, this.width / 2, 24, this._big, ui.GOLD, "center");
      }
    }

    _holeRects(player, center = false) {
      const n = player.hole.length;
      if (!n) return [];
      const dx = Math.floor(this.cw * 1.12);
      const total = this.cw + (n - 1) * dx;
      const cx = Math.floor(this.width / 2);
      const y = center || this.mode === "video" ? Math.floor(this.height * 0.62) : Math.floor(this.height * 0.6);
      const x0 = cx - Math.floor(total / 2);
      const out = [];
      for (let i = 0; i < n; i++) out.push(new PG.Rect(x0 + i * dx, y, this.cw, this.ch));
      return out;
    }

    _drawTable(ctx) {
      // Gegner oben
      const opp = this.players.slice(1);
      opp.forEach((p, i) => {
        const cx = Math.floor((this.width * (i + 1)) / (opp.length + 1));
        this._drawOpponent(ctx, p, cx, Math.floor(this.height * 0.2), i + 1 === this.turn && this.phase === ACTING);
      });
      // Community
      if (this.community.length) this._drawCommunity(ctx);
      // eigene Hand
      const me = this.players[0];
      const highlightHold = this.phase === DRAW_SELECT;
      this._holeRects(me).forEach((rect, i) => {
        this.renderer.draw(ctx, me.hole[i], rect.x, rect.y, this.cw, this.ch);
        if (highlightHold && this.holds.has(i)) {
          draw.rect(ctx, ui.GOLD, rect, 3, 6);
          ui.text(ctx, t("poker.keep"), rect.centerx, rect.bottom + 2, this._tiny, ui.GOLD, "midtop");
        }
      });
      // Namen
      let meLabel = t("poker.you");
      if (me.folded) meLabel += "  (" + t("poker.folded") + ")";
      const active = this.turn === 0 && this.phase === ACTING;
      ui.text(ctx, meLabel, this.width / 2, Math.floor(this.height * 0.6) - 22, this._small, active ? this.accent : ui.TEXT_DIM, "midtop");

      // Strip / Buttons
      if (this.phase === ACTING && this.players[this.turn].isHuman) this._drawActions(ctx);
      else if (this.phase === DRAW_SELECT) this._drawConfirm(ctx, t("poker.draw_btn"));
      else if (this.phase === SHOWDOWN) this._drawConfirm(ctx, t("poker.next_hand"));
      if (this.msg) this._drawMsg(ctx);
    }

    _drawOpponent(ctx, p, cx, cy, active) {
      const w = Math.floor(this.cw * 0.62);
      const h = Math.floor(this.ch * 0.62);
      const dx = Math.floor(w * 0.5);
      const x0 = cx - Math.floor((w + dx) / 2);
      p.hole.forEach((card, i) => {
        const img = card.faceUp || this.reveal ? this.renderer.get(card, w, h) : this.renderer.back(w, h);
        ctx.drawImage(img, x0 + i * dx, cy, w, h);
      });
      const name = p.name + (p.folded ? " (" + t("poker.folded") + ")" : "");
      const col = active ? this.accent : !p.folded ? ui.TEXT_DIM : ui.RED;
      ui.text(ctx, name, cx, cy - 16, this._tiny, col, "midtop");
      ui.text(ctx, String(p.stack), cx, cy + h + 2, this._tiny, ui.GOLD, "midtop");
      if (p.roundBet) ui.text(ctx, "+" + p.roundBet, cx, cy + h + 18, this._tiny, ui.GOLD, "midtop");
      if (this.phase === SHOWDOWN && p.result && !p.folded) {
        ui.text(ctx, t("poker.hand." + categoryKey(p.result)), cx, cy + h + 18, this._tiny, ui.TEXT, "midtop");
      }
    }

    _drawCommunity(ctx) {
      const n = this.community.length;
      const w = this.cw, h = this.ch;
      const dx = Math.floor(w * 1.12);
      const total = w + (n - 1) * dx;
      const x0 = Math.floor(this.width / 2) - Math.floor(total / 2);
      const y = Math.floor(this.height * 0.38);
      this.community.forEach((card, i) => this.renderer.draw(ctx, card, x0 + i * dx, y, w, h));
    }

    /** Halbtransparente Bedienleiste unten mit Akzent-Oberkante. */
    _drawStrip(ctx) {
      draw.rect(ctx, [8, 22, 16, 224], this.strip);
      draw.line(ctx, this.accent, this.strip.topleft, this.strip.topright, 2);
    }

    /** Button im Theme-Look; primary = Akzentrahmen mit sanftem Puls. */
    _drawBtn(ctx, rect, label, primary = false, on = true) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rect, 0, 10);
      if (primary) {
        const border = ui.mix(this.accent, [255, 255, 255], 0.25 * ui.pulse(2.2, 0.0, 1.0));
        draw.rect(ctx, border, rect, 2, 10);
      } else {
        draw.rect(ctx, ui.BORDER_LIGHT, rect, 1, 10);
      }
      ui.text(ctx, label, rect.centerx, rect.centery, this._small, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }

    _drawActions(ctx) {
      this._drawStrip(ctx);
      const p = this.players[this.turn];
      const toCall = this._toCall(p);
      const labels = {
        fold: t("poker.fold"),
        call: toCall === 0 ? t("poker.check") : t("poker.call_n", { n: Math.min(toCall, p.stack) }),
        raise: t("poker.raise"),
        allin: t("poker.allin"),
      };
      const keys = { fold: "F", call: "C", raise: "R", allin: "A" };
      for (const key in this.actionRects) this._drawBtn(ctx, this.actionRects[key], labels[key] + " (" + keys[key] + ")");
    }

    _drawConfirm(ctx, label) {
      this._drawStrip(ctx);
      this._drawBtn(ctx, this.dealRect, label, true);
    }

    _drawMsg(ctx) {
      ui.text(ctx, this.msg, this.width / 2, this.strip.y - 16, this._small, ui.GOLD, "center");
    }

    // ----- Video Poker zeichnen -----------------------------------------
    _drawVideo(ctx) {
      const me = this.players[0];
      this._holeRects(me, true).forEach((rect, i) => {
        this.renderer.draw(ctx, me.hole[i], rect.x, rect.y, this.cw, this.ch);
        if (this.phase === VIDEO_HOLD && this.holds.has(i)) {
          draw.rect(ctx, ui.GOLD, rect, 3, 6);
          ui.text(ctx, t("poker.held"), rect.centerx, rect.y - 3, this._small, ui.GOLD, "midbottom");
        }
        ui.text(ctx, String(i + 1), rect.centerx, rect.bottom + 3, this._tiny, ui.TEXT_DIM, "midtop");
      });
      this._drawPaytable(ctx);
      if (this.phase === VIDEO_HOLD) this._drawConfirm(ctx, t("poker.draw_btn"));
      else if (this.phase === SHOWDOWN) this._drawConfirm(ctx, t("poker.deal_btn"));
      if (this.msg) this._drawMsg(ctx);
    }

    _drawPaytable(ctx) {
      const rows = [["royal", 250], ["straight_flush", 50], ["quads", 25], ["full_house", 9], ["flush", 6], ["straight", 4], ["trips", 3], ["two_pair", 2], ["jacks", 1]];
      const x = 16;
      let y = Math.floor(this.height * 0.16);
      ui.text(ctx, t("poker.paytable"), x, y - 18, this._tiny, this.accent);
      for (const [name, mult] of rows) {
        const key = name !== "jacks" ? "poker.hand." + name : "poker.jacks";
        ui.text(ctx, t(key), x, y, this._tiny, ui.TEXT_DIM);
        ui.text(ctx, "x" + mult, x + 150, y, this._tiny, ui.GOLD);
        y += 16;
      }
    }

    _drawVideoBet(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, t("poker.mode.video"), cx, Math.floor(this.height * 0.22), this._huge, this.accent, "center");
      this._drawPaytable(ctx);
      // Chips
      this._drawStrip(ctx);
      this.chipRects.forEach((r, i) => {
        const v = VIDEO_BETS[i];
        const on = v === this.videoBet;
        draw.circle(ctx, on ? ui.GOLD : ui.BTN_SEL, r.center, 21);
        draw.circle(ctx, ui.TEXT, r.center, 21, 2);
        ui.text(ctx, String(v), r.centerx, r.centery, this._tiny, on ? [20, 24, 30] : ui.TEXT, "center");
      });
      ui.text(ctx, t("poker.bet") + ": " + this.videoBet, cx, this.strip.centery, this._big, ui.TEXT, "center");
      this._drawBtn(ctx, this.dealRect, t("poker.deal_btn"), true);
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 14, 10, 200], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      const pw = Math.min(this.width - 60, 460);
      ui.drawPanel(ctx, new PG.Rect(cx - pw / 2, cy - 92, pw, 184), { accentTop: ui.RED });
      ui.text(ctx, t("poker.broke"), cx, cy - 42, this._huge, ui.RED, "center");
      ui.text(ctx, t("poker.best") + ": " + this.best, cx, cy + 2, this._small, ui.TEXT_DIM, "center");
      ui.text(ctx, t("poker.broke_restart", { n: bank.START_CHIPS }), cx, cy + 32, this._small, ui.TEXT, "center");
      ui.text(ctx, t("common.enter_restart"), cx, cy + 60, this._tiny, ui.TEXT_FAINT, "center");
    }

    _drawPrehand(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, t("poker.mode." + this.mode), cx, Math.floor(this.height * 0.28), this._huge, this.accent, "center");
      const info = this.mode === "holdem" ? [t("poker.blinds", { sb: SMALL_BLIND, bb: BIG_BLIND })] : [t("poker.ante", { n: ANTE })];
      if (this.mode === "holdem") info.push(t("poker.opponents", { n: this.nOpponents }));
      info.forEach((line, i) => {
        ui.text(ctx, line, cx, Math.floor(this.height * 0.4) + i * 22, this._small, ui.TEXT_DIM, "center");
      });
      // Web: Optionen (Klick = umschalten)
      for (const key of this._optionKeys()) {
        const label = key === "opponents"
          ? t("poker.opponents", { n: this.nOpponents })
          : t("tank.difficulty") + ": " + t(DIFF_KEYS[this.aiLevel]);
        this._drawBtn(ctx, this.optRects[key], label, false, true);
      }
      this._drawBtn(ctx, this.dealRect, t("poker.deal_btn"), true);
    }
  }

  PG.register(PokerGame, {
    id: "PokerGame",
    key: "poker",
    name: "Poker",
    modes: [
      ["holdem", "poker.mode.holdem"],
      ["draw", "poker.mode.draw"],
      ["video", "poker.mode.video"],
    ],
    settingsKey: "poker",
    defaults: { opponents: 2, difficulty: 1 },
  });
})();
