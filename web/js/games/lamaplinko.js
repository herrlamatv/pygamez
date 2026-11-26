/*
 * lamaplinko.js - Lama Plinko (Port von games/lamaplinko.py)
 * ==========================================================
 * Kugel-Spiel mit Lama-Chips: Das Lama oben am Brett lässt goldene Wollknäuel
 * durch 8 bis 16 Reihen Stifte fallen. An jedem Stift springt die Kugel nach
 * links oder rechts (je 50 %); das Fach unten bestimmt den Multiplikator.
 * Außen ist es heiß (selten, hoch), in der Mitte kühl.
 *
 * - 8-16 Reihen (←/→) und Risiko Niedrig/Mittel/Hoch (1-3), beides gesperrt,
 *   solange Kugeln fliegen. 27 feste Tabellen mit ~97 % Auszahlungsquote
 *   (lamaplinko_logic.js), Rand bei 16 Reihen ×16 / ×110 / ×1000.
 * - Einsatz je Kugel 1 ... 1000. Leertaste/Klick = werfen, gedrückt halten =
 *   Dauerfeuer, bis zu 50 Kugeln gleichzeitig; Turbo (T) halbiert die Flugzeit.
 * - Jede Kugel ist eine eigene Fairness-Runde (Fenster F): Der Prüfcode
 *   (SHA-256 des Server-Seeds) steht vor dem Wurf fest, der Seed wird nach der
 *   Landung gezeigt und lässt sich nachrechnen - mit Mini-Vorschau des Wegs.
 * - Statistik (S): Kacheln, Bilanzkurve und die Treffer-Verteilung je
 *   Reihenzahl gegen die erwartete Binomialkurve.
 * - Auto-Modus (A): 10 ... 1000 Kugeln, Stopp bei Gewinn/Verlust ab ...
 *
 * Buchung: Der Weg der Kugel kommt beim Abwurf aus dem Fairness-Zufall - das
 * Ergebnis steht damit schon fest und wird sofort mit PG.lamabank.betAndPay
 * gebucht (ein Speichern inkl. Verlauf, Fairness-Log und Statistik). Gezeigt
 * wird erst bei der Landung (pending, angezeigte Chips wie bei Lama Crossing);
 * Erfolge fallen ebenfalls bei der Landung, destroy() holt sie für noch
 * fliegende Kugeln nach. Highscore = Höchststand von 1000 + Lama-Plinko-Bilanz;
 * gameOver wird nie gesetzt. Spielstand (lamabank-Extra "lamaplinko") und
 * Einstellungen (Abschnitt "lamaplinko") im selben Format wie die Desktop-Version.
 *
 * Die Flugbahn folgt dem Pfad Stift für Stift: Bogensprünge mit Schwerkraft
 * (Parabel, Scheitel bei ~30 %), kleines kosmetisches Zittern (beim Abwurf
 * gezogen), die Position ist eine reine Funktion der Flugzeit.
 *
 * Steuerung (alle Tasten roh, ohne die p1/p2-Belegung - wie Lama Crossing):
 * Leertaste/Enter/Klick auf WERFEN oder aufs Brett = werfen (halten =
 * Dauerfeuer), ←/→ = Reihen, 1-3 = Risiko, ↑/↓/+/-/Mausrad = Einsatz,
 * A = Auto, S = Statistik, F = Fairness, T = Turbo, Esc = Fenster schließen
 * bzw. Auto stoppen. In den Fenstern: Auto ↑/↓ Zeile, ←/→ Wert, Enter =
 * START/STOPP; Statistik ←/→ Reihenzahl; Fairness ↑/↓/Mausrad = Kugel,
 * Enter = Nachrechnen, C = neuer Client-Seed.
 */
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  const L = PG.lamaPlinkoLogic;
  const D = PG.lamaPlinkoDraw;
  const CD = PG.casinoDraw;
  const F = PG.fairness;
  const CS = PG.chipstats;
  const bank = PG.lamabank;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

  const KEY = L.KEY;
  const DROP_T = 0.3; // vom Maul des Lamas bis zum ersten Stift
  const BOUNCE = 0.24; // Sprunghöhe über dem Stift (Anteil am Reihenabstand)
  const FLASH_T = 0.32; // Leuchtdauer eines getroffenen Stifts
  const BUMP_T = 0.32; // Fach wippt nach einem Treffer
  const NOD_T = 0.3; // Lama nickt beim Abwurf
  const HOLD_DELAY = 0.32; // gehaltene Taste: erst danach Dauerfeuer
  const HOLD_BALLS = 30; // Dauerfeuer: so viele Kugeln sind dann gleichzeitig unterwegs
  const AUTO_RATE = 0.24; // Abstand im Auto-Modus (Turbo: halb so lang)
  const TRAIL = 6; // Spurpunkte je Kugel
  const TRAIL_DT = 0.024; // Flugzeit zwischen zwei Spurpunkten
  const TICK_GAP = 0.045; // kleinster Abstand zweier Stift-Klicks (Ton)
  const BIG_MULT = 1000; // ab x10: Münzregen
  const HUGE_MULT = 10000; // ab x100: großer Gewinn (Banner, Konfetti)
  const AUTO_BALLS = [10, 25, 50, 100, 250, 500, 1000];
  const AUTO_LIMITS = [0, 100, 500, 1000, 5000, 10000, 50000];
  const RISK_KEYS = ["1", "2", "3"];
  const DEFAULTS = { rows: 12, risk: "medium", bet: 10, turbo: false, auto_balls: 25, auto_win: 0, auto_loss: 0 };

  /** Kaufmännisch runden (x.5 nach oben) - in Python, JS und Rust gleich. */
  const ri = (v) => Math.floor(v + 0.5);
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);
  /** pygame-Mitte (ganzzahlig) eines Rechtecks. */
  const icx = (r) => r.x + fdiv(r.w, 2);
  const icy = (r) => r.y + fdiv(r.h, 2);

  /** Flugzeit von Stiftreihe zu Stiftreihe (mehr Reihen = kürzere Sprünge). */
  function segTime(rows) {
    return 0.165 - (rows - 8) * 0.005;
  }
  /** Gesamte Flugzeit einer Kugel (ohne Turbo). */
  function flightTime(rows) {
    return DROP_T + rows * segTime(rows);
  }
  /** Abstand zweier Kugeln im Dauerfeuer (ohne Turbo): Flugzeit / HOLD_BALLS
   *  - beim Halten sind so bei jeder Reihenzahl ~30 Kugeln gleichzeitig unterwegs. */
  function fireRate(rows) {
    return flightTime(rows) / HOLD_BALLS;
  }

  /** Leertaste und Enter werfen (roh, nicht über die p1/p2-Belegung). */
  const isDropKey = (key) => key === "space" || key === "Return" || key === "KP_Enter";

  // Schrifthöhen wie pygame: Font.get_height() ist bei Bahnschrift/Consolas genau die
  // Schriftgröße, ein gerenderter Text ist eine Zeile höher (Consolas gleich hoch).
  // PG.ui.Font.height (= ceil(1,1975 px)) liegt darüber; das Original rechnet Layout
  // und fit_font-Startgrößen aber mit get_height().
  const gh = (f) => f.px;
  const rh = (f) => (f.mono ? f.px : f.px + 1);

  /** Gleicher Fairness-Zustand? (Python vergleicht dicts, unabhängig von der Reihenfolge) */
  function sameFair(raw, st) {
    return isDict(raw) && Object.keys(raw).length === 3 && raw.server === st.server
      && raw.client === st.client && raw.nonce === st.nonce;
  }

  class LamaPlinkoGame extends PG.Game {
    /** Jede Kugel ist ein Match (Game.logMatch). */
    get matchManual() {
      return true;
    }

    // ===================================================== Aufbau / Reset
    reset() {
      if (this.balls && this.balls.length) this._flushFlying();
      this.gameOver = false;
      bank.load();
      this.rng = PG.rand;
      const rows = this._opt("rows", L.DEFAULT_ROWS);
      this.rows = L.ROWS.includes(rows) ? rows : L.DEFAULT_ROWS;
      const risk = this._opt("risk", L.DEFAULT_RISK);
      this.risk = L.RISKS.includes(risk) ? risk : L.DEFAULT_RISK;
      const bet = this._opt("bet", 10);
      this.bet = L.BETS.includes(bet) ? bet : L.BETS[0];
      this.turbo = this._opt("turbo", false) === true;
      const ab = this._opt("auto_balls", 25);
      this.autoBalls = AUTO_BALLS.includes(ab) ? ab : 25;
      const aw = this._opt("auto_win", 0);
      this.autoWin = AUTO_LIMITS.includes(aw) ? aw : 0;
      const al = this._opt("auto_loss", 0);
      this.autoLoss = AUTO_LIMITS.includes(al) ? al : 0;
      let extra = bank.getExtra(KEY, {});
      if (!isDict(extra)) extra = {};
      this.hist = L.cleanHistory(extra.hist);
      this.fair = F.cleanState(extra.fair);
      this.fairlog = F.cleanLog(extra.fairlog);
      this.stats = CS.clean(extra.stats);
      this.carry = L.cleanCarry(extra.carry); // Rest-Hundertstel (siehe _drop)
      this.stats.x.hits = L.cleanHits(this.stats.x.hits);
      if (!sameFair(extra.fair, this.fair)) {
        // neue Seeds sofort festschreiben: ein gezeigter Prüfcode gilt auch nach einem Neustart
        bank.setExtra(KEY, this._extra());
      }
      // Angezeigter Stand (bei der Landung nachgeführt) - gespeichert wird beim Abwurf
      this.histShown = this.hist.slice();
      this.fairlogShown = this.fairlog.map((e) => Object.assign({}, e));
      this.statsShown = CS.clean(this.stats);
      this.statsShown.x.hits = L.cleanHits(this.stats.x.hits);
      this.balls = [];
      this.pending = 0; // Gewinne fliegender Kugeln (gebucht, nicht gezeigt)
      this.shownChips = bank.balance();
      this.score = bank.scoreFor(KEY);
      this.recShown = this.score;
      this.shownLedger = bank.ledger(KEY);
      this.sess = { balls: 0, net: 0, best: 0 }; // diese Sitzung (bei der Landung gezählt)
      this.auto = null; // laufender Auto-Modus
      this.panel = null; // offenes Fenster: "auto" | "stats" | "fair"
      this.autoFocus = 0;
      this.statsRows = this.rows;
      this.statsHover = null;
      this.fairSel = 0;
      this.fairScroll = 0;
      this.fairChecks = new Map(); // nonce -> [Prüfcode ok, Ergebnis ok, Pfad]
      this.hold = false; // Leertaste/Maus auf WERFEN gehalten
      this.holdT = 0;
      this.fireCd = 0;
      this.autoCd = 0;
      this.tickCd = 0;
      this.nod = 0;
      this.particles = [];
      this.coinsFx = [];
      this.popups = [];
      this.banner = null;
      this.toast = null;
      this.shake = 0;
      this.histAnim = 1;
      this.time = 0;
      this.mouse = [-1, -1];
      this.broke = false;
      this._tickCd = 0;
      this._tickN = 0; // Zähl-Klick der Chips (Tonhöhe ohne Zufall)
      this._commitFor = null; // Prüfcode-Zwischenspeicher (Server-Seed -> SHA-256)
      this._commit = "";
      this._lang = PG.lang;
      this._layout();
      this._newBoardFx();
      this._checkBroke();
      if (!this.broke) this._say(t("lpl.hint"), 3.6);
    }

    /** Verlassen (on_exit): Erfolge noch fliegender Kugeln nachholen (gebucht ist schon alles). */
    destroy() {
      this._flushFlying();
      this.score = bank.scoreFor(KEY);
    }

    get wantsEscape() {
      return this.panel !== null || this.auto !== null;
    }
    /** Kein Game Over - also auch kein Highscore-Banner. */
    get showHighscoreBanner() {
      return false;
    }

    _flushFlying() {
      for (const b of this.balls) this._ballAchievements(b);
      this.balls = [];
      this.pending = 0;
    }

    _newBoardFx() {
      this.pegFlash = [];
      for (let r = 0; r < this.rows; r++) this.pegFlash.push(new Array(r + 3).fill(0));
      this.slotBump = new Array(this.rows + 1).fill(0);
    }

    // ===================================================== Layout
    _layout() {
      const W = this.width, H = this.height;
      const k = (this.k = Math.min(W / 800, H / 600));
      this.fTiny = ui.font(Math.max(11, trunc(13 * k)));
      this.fSmall = ui.font(Math.max(12, trunc(15 * k)));
      this.fBtn = ui.font(Math.max(12, trunc(15 * k)), true);
      this.fBig = ui.font(Math.max(15, trunc(22 * k)), true);
      this.fHuge = ui.font(Math.max(24, trunc(46 * k)), true);
      this.fMono = ui.font(Math.max(10, trunc(13 * k)), false, true);
      const m = (this.margin = Math.max(6, trunc(12 * k)));
      this.hudH = Math.max(28, trunc(42 * k));
      this.stripH = Math.max(46, trunc(64 * k));
      this.strip = new PG.Rect(0, H - this.stripH, W, this.stripH);
      this.region = new PG.Rect(0, this.hudH, W, this.strip.y - this.hudH);
      // --- Seitenleisten: links Risiko + Reihen, rechts Verlauf
      const side = (this.sideW = Math.max(92, trunc(150 * k)));
      const top = this.region.y + m;
      this.left = new PG.Rect(m, top, side, this.region.h - 2 * m);
      this.right = new PG.Rect(W - m - side, top, side, this.region.h - 2 * m);
      let pad = Math.max(5, trunc(9 * k));
      let bh = Math.max(22, trunc(30 * k));
      const x = this.left.x + pad;
      const w = this.left.w - 2 * pad;
      let y = this.left.y + pad + gh(this.fTiny) + 3;
      this.rRisks = [];
      for (let i = 0; i < 3; i++) {
        this.rRisks.push(new PG.Rect(x, y, w, bh));
        y += bh + 4;
      }
      this.lblRowsY = y + fdiv(pad, 2);
      y = this.lblRowsY + gh(this.fTiny) + 3;
      this.rRowsM = new PG.Rect(x, y, bh, bh);
      this.rRowsP = new PG.Rect(x + w - bh, y, bh, bh);
      this.rRowsV = new PG.Rect(this.rRowsM.right + 3, y, this.rRowsP.x - this.rRowsM.right - 6, bh);
      this.infoY = y + bh + pad;
      this.pillH = Math.max(16, trunc(22 * k));
      // --- Bedienleiste: Einsatz links, Fenster-Knöpfe Mitte, WERFEN rechts
      const sh = this.stripH;
      bh = Math.max(30, trunc(sh * 0.62));
      const by = this.strip.y + fdiv(sh - bh, 2);
      const lblW = Math.max(this.fTiny.width(t("cas.bet")), this.fBtn.width("1000")) + trunc(14 * k);
      this.rMinus = new PG.Rect(m, by, bh, bh);
      this.rBet = new PG.Rect(this.rMinus.right + 4, by, lblW, bh);
      this.rPlus = new PG.Rect(this.rBet.right + 4, by, bh, bh);
      pad = trunc(22 * Math.max(0.7, k));
      let dropW = Math.max(this.fBtn.width(t("lpl.drop")), this.fBtn.width(t("chip.auto_stop"))) + pad;
      dropW = Math.max(dropW, trunc(130 * k));
      this.rDrop = new PG.Rect(W - m - dropW, by, dropW, bh);
      const gap = Math.max(4, trunc(7 * k));
      const nIcons = 4;
      const room = this.rDrop.x - this.rPlus.right - 2 * m;
      const ib = Math.min(bh, fdiv(room - (nIcons - 1) * gap, nIcons));
      const x0 = this.rPlus.right + m + fdiv(room - (nIcons * ib + (nIcons - 1) * gap), 2);
      const iy = by + fdiv(bh - ib, 2);
      this.rIcons = [];
      for (let i = 0; i < nIcons; i++) this.rIcons.push(new PG.Rect(x0 + i * (ib + gap), iy, ib, ib));
      this._boardLayout();
      this._panelLayout();
    }

    /** Brett-Geometrie für die aktuelle Reihenzahl (Stifte, Fächer, Lama). */
    _boardLayout() {
      const k = this.k, m = this.margin;
      const n = this.rows;
      const bx0 = this.left.right + m;
      const bx1 = this.right.x - m;
      this.cx = (bx0 + bx1) / 2;
      this.llamaN = Math.max(32, trunc(62 * k));
      this.footY = this.region.y + trunc(this.llamaN * 0.95) + 2; // Ohren knapp unter dem HUD
      this.top = this.footY + Math.max(8, trunc(14 * k));
      const availH = this.region.bottom - m - 3 - this.top;
      let gap = Math.min((bx1 - bx0 - 10) / (n + 2.6), availH / (n + 0.36));
      gap = this.gap = Math.max(8, gap);
      this.pegR = Math.max(2, trunc(gap * 0.12));
      this.ballR = Math.max(3, trunc(gap * 0.28));
      this.slotW = Math.max(6, trunc(gap - Math.max(2, gap * 0.1)));
      this.slotH = Math.max(10, trunc(gap * 0.78));
      this.slotTop = trunc(this.top + (n - 1) * gap + gap * 0.55);
      this.slotFont = Math.max(8, trunc(gap * 0.44));
      // Lama: Maul knapp links über dem Einwurf
      this.footX = this.cx - D.LLAMA_MOUTH[0] * this.llamaN + D.LLAMA_PIVOT[0] * this.llamaN - gap * 0.35;
      const [mx, my] = D.mouthPos(this.llamaN, 0, this.footX, this.footY);
      // kleines Knäuel im Maul; beim Abwurf wächst es auf volle Größe
      this.mouthR = Math.max(3, Math.min(this.ballR, trunc(this.llamaN * 0.085)));
      this.mouth = [mx + this.mouthR * 0.55, my + this.mouthR * 0.2];
      // Brett-Umriss (Trapez) + Stifte relativ zur Brett-Box
      const halfTop = gap * 1.6;
      const halfBot = ((n + 2) / 2) * gap + gap * 0.25;
      const y0 = this.top - gap * 0.55;
      const y1 = this.slotTop + this.slotH + 6;
      this.boardBox = new PG.Rect(trunc(this.cx - halfBot) - 4, trunc(y0) - 4, trunc(2 * halfBot) + 8, trunc(y1 - y0) + 8);
      const bb = this.boardBox;
      this.platePts = [[this.cx - halfTop - bb.x, y0 - bb.y], [this.cx + halfTop - bb.x, y0 - bb.y],
        [this.cx + halfBot - bb.x, y1 - bb.y], [this.cx - halfBot - bb.x, y1 - bb.y]];
      const pegs = [];
      for (let r = 0; r < n; r++) {
        for (let j = 0; j < r + 3; j++) {
          const [px, py] = this._pegXY(r, j);
          pegs.push([ri(px) - bb.x, ri(py) - bb.y]);
        }
      }
      this.pegsLocal = pegs;
      this.slotRects = [];
      for (let i = 0; i <= n; i++) this.slotRects.push(this._slotRect(i));
    }

    /** Fenster (Auto, Statistik, Fairness): mittig; Auto so hoch wie nötig,
     *  Statistik und Fairness so groß wie möglich. */
    _panelLayout() {
      const W = this.width, H = this.height, k = this.k, m = this.margin;
      this.panHead = Math.max(30, trunc(42 * k));
      const pw = Math.min(W - 2 * m, Math.max(trunc(660 * k), 456));
      let ph = Math.min(H - 2 * m, Math.max(trunc(480 * k), 340));
      if (this.panel === "auto") ph = Math.min(ph, this.panHead + fdiv(m, 2) + this._autoContentH() + m);
      this.pan = new PG.Rect(fdiv(W - pw, 2), fdiv(H - ph, 2), pw, ph);
      const cb = this.panHead - 10;
      this.rClose = new PG.Rect(this.pan.right - 6 - cb, this.pan.y + 5, cb, cb);
    }

    // ===================================================== Geometrie
    /** Mittelpunkt von Stift j in Reihe r (Reihe r hat r+3 Stifte). */
    _pegXY(r, j) {
      return [this.cx + (j - (r + 2) / 2) * this.gap, this.top + r * this.gap];
    }
    _slotX(i) {
      return this.cx + (i - this.rows / 2) * this.gap;
    }
    _slotRect(i) {
      return new PG.Rect(ri(this._slotX(i)) - fdiv(this.slotW, 2), this.slotTop, this.slotW, this.slotH);
    }
    /** Aufprallpunkt der Kugel über dem Stift (r, k+1) - mit Zittern. */
    _contact(r, k, jit) {
      return [this.cx + (k - r / 2 + jit) * this.gap, this.top + r * this.gap - (this.pegR + this.ballR) * 0.92];
    }
    _landing(k, jit) {
      return [this._slotX(k) + jit * this.gap * 0.6, this.slotTop + this.ballR * 0.15];
    }
    /** Radius einer Kugel nach der Flugzeit tt (wächst aus dem Maul heraus). */
    _ballRadius(tt) {
      const f = tt / (DROP_T * 0.7);
      if (f >= 1) return this.ballR;
      return this.mouthR + trunc((this.ballR - this.mouthR) * Math.max(0, f));
    }
    /** Position einer Kugel nach der Flugzeit tt (reine Funktion von tt). */
    _ballPos(b, tt) {
      const n = b.rows;
      if (tt <= 0) return this.mouth;
      if (tt < DROP_T) {
        const p = tt / DROP_T;
        const [x1, y1] = this._contact(0, 0, b.jit[0]);
        const [x0, y0] = this.mouth;
        return [x0 + (x1 - x0) * p, y0 + (y1 - y0) * p * p];
      }
      const st = segTime(n);
      const u = (tt - DROP_T) / st;
      const seg = Math.min(n - 1, trunc(u));
      const p = Math.min(1, u - seg);
      const ka = b.pre[seg];
      const [xa, ya] = this._contact(seg, ka, b.jit[seg]);
      const kb = ka + b.path[seg];
      const [xb, yb] = seg < n - 1 ? this._contact(seg + 1, kb, b.jit[seg + 1]) : this._landing(kb, b.jit[n]);
      const hgt = BOUNCE * this.gap * b.hb[seg];
      const dy = yb - ya;
      const a = 2 * hgt + 2 * Math.sqrt(hgt * hgt + hgt * dy);
      const bq = dy + a;
      return [xa + (xb - xa) * p, ya - a * p + bq * p * p];
    }

    // ===================================================== Einstellungen
    _opt(key, def) {
      const v = this.opts[key];
      return v === undefined ? def : v;
    }
    _setOpt(key, value) {
      if (this.opts[key] !== value) {
        this.opts[key] = value;
        this.saveSettings();
      }
    }

    // ===================================================== Konto
    _extra() {
      return { hist: this.hist.slice(), fair: Object.assign({}, this.fair), fairlog: this.fairlog.slice(), stats: this.stats,
        carry: this.carry };
    }
    /** Angezeigtes Guthaben (ohne die Gewinne noch fliegender Kugeln). */
    _avail() {
      return Math.max(0, bank.balance() - this.pending);
    }
    _clampBet() {
      const chips = this._avail();
      if (this.bet > chips) {
        const ok = L.BETS.filter((b) => b <= chips);
        this.bet = ok.length ? ok[ok.length - 1] : L.BETS[0];
      }
    }
    _checkBroke() {
      const was = this.broke;
      this.broke = !this.balls.length && bank.isBroke(KEY);
      if (this.broke && !was) {
        this.playSound("gameover");
        this.panel = null;
        this.auto = null;
      }
    }
    _takeCredit() {
      if (bank.refillIfBroke(KEY)) {
        this.playSound("powerup");
        ui.spawnBurst(fdiv(this.width, 2), fdiv(this.height, 2), ui.GOLD, 26);
      }
      this.broke = false;
      this.shownChips = bank.balance();
      this._clampBet();
    }
    _say(text, secs = 1.8) {
      this.toast = [text, secs];
    }
    _sep() {
      return t("lpl.decimal");
    }
    _fmt(m100) {
      return "×" + L.fmtShort(m100, this._sep());
    }
    /** Quote der aktuellen Tabelle mit zwei Stellen (exakt gerundet wie mit Fraction). */
    _rtpText() {
      const tab = L.table(this.rows, this.risk);
      let num = 0;
      tab.forEach((m, i) => { num += L.comb(this.rows, i) * m; });
      const den = Math.pow(2, this.rows) * 100;
      const v = fdiv(num * 20000 + den, 2 * den);
      return `${fdiv(v, 100)}${this._sep()}${String(v % 100).padStart(2, "0")}`;
    }
    _riskName(risk) {
      return t("lpl.risk." + risk);
    }
    /** Prüfcode des nächsten Server-Seeds (zwischengespeichert). */
    _nextCommit() {
      if (this._commitFor !== this.fair.server) {
        this._commitFor = this.fair.server;
        this._commit = F.commit(this.fair.server);
      }
      return this._commit;
    }

    // ===================================================== Abwurf
    _speed() {
      return this.turbo ? 2 : 1;
    }

    /** Eine Kugel werfen: Ergebnis ziehen, buchen, Flug starten. true = geworfen. */
    _drop(quiet = false) {
      if (this.broke) return false;
      if (this.balls.length >= L.MAX_BALLS) {
        if (!quiet) this._say(t("lpl.max_balls", { n: L.MAX_BALLS }), 1.2);
        return false;
      }
      const bet = this.bet;
      if (bet > this._avail()) {
        if (!quiet) {
          this._say(t("cas.not_enough"));
          this.playSound("hit");
        }
        return false;
      }
      const rows = this.rows, risk = this.risk;
      const entry = F.begin(this.fair);
      F.advance(this.fair);
      const path = L.dropPath(F.entryRand(KEY, entry), rows);
      const slot = L.slotOf(path);
      const m100 = L.mult100(rows, risk, slot);
      // Rest-Hundertstel-Übertrag: x0,4 bei Einsatz 1 zahlt sonst nie etwas
      const [win, carry] = L.payoutCarry(bet, rows, risk, slot, this.carry);
      this.carry = carry;
      const outcome = { rows, risk, slot, m: m100, bet, win };
      F.record(this.fairlog, entry, outcome);
      CS.record(this.stats, bet, win, m100);
      const net = win - bet;
      this.logMatch(net, net > 0 ? 1 : net < 0 ? 0 : 2,
        { bet, win, x: m100, rw: rows, lv: risk });
      L.addHit(this.stats.x.hits, rows, slot);
      this.hist = this.hist.concat([m100]).slice(-L.HISTORY);
      if (!bank.betAndPay(bet, win, KEY, KEY, this._extra())) return false; // (kann nach der Prüfung oben nicht passieren)
      this.score = bank.scoreFor(KEY);
      this.pending += win;
      const pre = [0];
      for (const d of path) pre.push(pre[pre.length - 1] + d);
      // kosmetisches Zittern (nur Anzeige - das Ergebnis steht schon fest)
      const R = this.rng;
      const jit = [];
      for (let i = 0; i <= rows; i++) jit.push(R.uniform(-0.06, 0.06));
      const hb = [];
      for (let i = 0; i < rows; i++) hb.push(R.uniform(0.82, 1.18));
      const spin = R.randrange(12);
      this.balls.push({
        rows, risk, bet, win, m: m100, slot, path, pre, jit, hb, spin, t: 0, hit: 0,
        log: F.makeEntry(entry, outcome), msg: null,
      });
      this.nod = NOD_T;
      this.playSound("move");
      if (this.auto !== null) this.auto.net += win - bet;
      if (this.balls.length >= L.RAIN_BALLS) this.achEvent("lpl_rain");
      return true;
    }

    _startHold() {
      if (this.auto !== null) {
        this._stopAuto(null);
        return;
      }
      this.hold = true;
      this.holdT = 0;
      this.fireCd = 0;
      this._drop();
    }

    // ===================================================== Auto-Modus
    _startAuto() {
      if (this.broke || this.auto !== null) return;
      if (this.bet > this._avail()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      this.auto = { left: this.autoBalls, net: 0 };
      this.autoCd = 0;
      this.panel = null;
      this.hold = false;
      this.playSound("select");
    }

    /** Auto beenden. msgKey wird gezeigt, sobald die letzte Kugel gelandet ist. */
    _stopAuto(msgKey) {
      if (this.auto === null) return;
      this.auto = null;
      if (msgKey === null) {
        this.playSound("click");
        return;
      }
      if (this.balls.length) this.balls[this.balls.length - 1].msg = msgKey;
      else this._say(t(msgKey), 2.4);
    }

    _autoStep() {
      const a = this.auto;
      if (a.left <= 0) {
        this._stopAuto("chip.auto_done");
        return;
      }
      if (this.bet > this._avail()) {
        if (this.balls.length) return; // erst landen lassen - vielleicht reicht es dann
        this._stopAuto("chip.auto_no_chips");
        this._clampBet();
        return;
      }
      if (this.balls.length >= L.MAX_BALLS) return;
      if (!this._drop(true)) return;
      a.left -= 1;
      if (this.autoWin && a.net >= this.autoWin) this._stopAuto("chip.auto_hit_win");
      else if (this.autoLoss && -a.net >= this.autoLoss) this._stopAuto("chip.auto_hit_loss");
      else if (a.left <= 0) this._stopAuto("chip.auto_done");
    }

    // ===================================================== Landung
    /** Anzeige im Matches-Screen: Risiko-Stufe statt interner Kennung. */
    static matchText(key, value) {
      if (key === "lv" && ["low", "medium", "high"].includes(value)) return t("lpl.risk." + value);
      return null;
    }

    _land(b) {
      const slot = b.slot, m100 = b.m, win = b.win;
      this.pending = Math.max(0, this.pending - win);
      this.shownLedger += win - b.bet;
      this.recShown = Math.min(this.score, Math.max(this.recShown, bank.START_CHIPS + this.shownLedger));
      this.histShown = this.histShown.concat([m100]).slice(-L.HISTORY);
      this.histAnim = 0;
      this.sess.balls += 1;
      this.sess.net += win - b.bet;
      this.sess.best = Math.max(this.sess.best, m100);
      CS.record(this.statsShown, b.bet, win, m100);
      L.addHit(this.statsShown.x.hits, b.rows, slot);
      this.fairlogShown.push(b.log);
      if (this.fairlogShown.length > F.LOG) this.fairlogShown.splice(0, this.fairlogShown.length - F.LOG);
      if (slot >= 0 && slot < this.slotBump.length) this.slotBump[slot] = BUMP_T;
      const r = slot < this.slotRects.length ? this.slotRects[slot] : this._slotRect(slot);
      const col = D.slotColor(m100);
      this._spawn(icx(r), r.y, 6 + Math.min(14, fdiv(m100, 200)), col, 160, true, 2.6, 0.5, 520);
      const big = m100 >= BIG_MULT;
      this._popup(icx(r), r.y - 6 * this.k, this._fmt(m100), m100 >= 100 ? col : ui.TEXT_DIM, 1.0 + 0.4 * (big ? 1 : 0), big);
      this._landSound(m100);
      if (m100 >= HUGE_MULT) {
        const title = b.rows === 16 && b.risk === "high" && L.isEdge(16, slot) ? t("cas.s.jackpot")
          : m100 >= 5 * HUGE_MULT ? t("cas.s.mega_win") : t("cas.s.big_win");
        this.banner = { title, sub: `${this._fmt(m100)}   ·   +${win}`, t: 0, dur: 2.6 };
        ui.spawnConfetti(this.width, this.height, m100 >= 5 * HUGE_MULT ? 120 : 80);
        this._coinShower(m100 >= 5 * HUGE_MULT ? 90 : 50);
        this.shake = 0.35;
        this.rumble(240);
      } else if (big) {
        this._coinShower(18);
        this.rumble(90);
      }
      this._ballAchievements(b);
      if (b.msg) this._say(t(b.msg), 2.4);
    }

    _ballAchievements(b) {
      if (!L.isEdge(b.rows, b.slot)) return;
      if (b.rows >= L.EDGE_ROWS) this.achEvent("lpl_edge");
      if (b.rows === 16 && b.risk === "high") this.achEvent("lpl_jackpot");
    }

    _landSound(m100) {
      if (m100 >= HUGE_MULT) this.playSound("level");
      else if (m100 >= BIG_MULT) this.playSound("win");
      else if (m100 >= 200) this.tone(m100 >= 500 ? 784 : 587, 0.09, "square", 0.12);
      else if (m100 >= 100) this.tone(440, 0.07, "square", 0.1);
      else this.tone(262, 0.06, "sine", 0.1);
    }

    // ===================================================== Einstellungen (Spiel)
    _setBet(direction) {
      if (this.auto !== null) return;
      const i = L.BETS.indexOf(this.bet);
      const j = Math.max(0, Math.min(L.BETS.length - 1, i + direction));
      if (j !== i) {
        this.bet = L.BETS[j];
        this._setOpt("bet", this.bet);
        this.playSound("select");
        if (this.bet > this._avail()) this._say(t("cas.not_enough"), 1.2);
      }
    }
    _locked() {
      if (this.balls.length || this.auto !== null) {
        this._say(t("lpl.locked"), 1.4);
        this.playSound("hit");
        return true;
      }
      return false;
    }
    _setRows(rows) {
      rows = Math.max(L.ROWS[0], Math.min(L.ROWS[L.ROWS.length - 1], rows));
      if (rows === this.rows || this._locked()) return;
      this.rows = rows;
      this._setOpt("rows", rows);
      this._boardLayout();
      this._newBoardFx();
      this.playSound("click");
    }
    _setRisk(i) {
      const risk = L.RISKS[i];
      if (risk === this.risk || this._locked()) return;
      this.risk = risk;
      this._setOpt("risk", risk);
      this.playSound("click");
    }
    _toggleTurbo() {
      this.turbo = !this.turbo;
      this._setOpt("turbo", this.turbo);
      this._say(this.turbo ? t("lpl.turbo_on") : t("lpl.turbo_off"), 1.2);
      this.playSound("select");
    }
    _open(name) {
      if (this.panel === name) {
        this.panel = null;
        this.playSound("click");
        return;
      }
      this.panel = name;
      this.hold = false;
      this._panelLayout();
      if (name === "stats") this.statsRows = this.rows;
      else if (name === "fair") {
        this.fairSel = 0;
        this.fairScroll = 0;
      }
      this.playSound("select");
    }
    _newClient() {
      if (this._locked()) return;
      F.newClient(this.fair);
      bank.setExtra(KEY, this._extra());
      this._say(t("lpl.client_new", { c: this.fair.client }), 1.8);
      this.playSound("select");
    }
    /** Nachrechnen: Prüfcode + Ergebnis aus Seed, Client-Seed und Nr. neu. */
    _verify(e) {
      const o = isDict(e.o) ? e.o : {};
      const rows = L.ROWS.includes(o.rows) ? o.rows : null;
      const okHash = F.verify(e);
      const path = rows ? L.pathFromSeed(rows, e.s, e.c, e.n) : [];
      const slot = L.slotOf(path);
      const okRes = !!rows && slot === o.slot && L.RISKS.includes(o.risk) && L.mult100(rows, o.risk, slot) === o.m;
      this.fairChecks.set(e.n, [okHash, okRes, path]);
      this.playSound(okHash && okRes ? "point" : "hit");
    }

    // ===================================================== Effekte
    _spawn(x, y, n, col, speed, up, r, life, grav) {
      const R = this.rng;
      for (let i = 0; i < n; i++) {
        const a = up ? R.uniform(Math.PI * 1.1, Math.PI * 1.9) : R.uniform(0, TAU);
        const sp = speed * R.uniform(0.35, 1.0) * this.k;
        this.particles.push({
          x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, r: r * this.k * R.uniform(0.6, 1.2), col, t: 0,
          life: life * R.uniform(0.7, 1.1), g: grav * this.k,
        });
      }
      if (this.particles.length > 260) this.particles = this.particles.slice(-260);
    }
    _coinShower(n = 40) {
      const R = this.rng, W = this.width, k = this.k;
      for (let i = 0; i < n; i++) {
        this.coinsFx.push({
          x: R.uniform(W * 0.1, W * 0.9), y: R.uniform(-this.height * 0.4, -10), vx: R.uniform(-60, 60) * k,
          vy: R.uniform(-80, 120) * k, spin: R.uniform(0, TAU), vs: R.uniform(6, 14), r: R.uniform(7, 12) * Math.max(0.7, k),
        });
      }
      if (this.coinsFx.length > 160) this.coinsFx = this.coinsFx.slice(-160);
    }
    _popup(x, y, text, color, dur = 1.0, big = false) {
      this.popups.push({ x, y, text, color, t: 0, dur, big });
      if (this.popups.length > 14) this.popups = this.popups.slice(-14);
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      const kind = ev.kind;
      if ((kind === "mousemove" || kind === "mousedown" || kind === "mouseup") && ev.pos) this.mouse = ev.pos;
      if (kind === "keyup" && isDropKey(ev.key)) {
        this.hold = false;
        return;
      }
      if (kind === "mouseup") {
        this.hold = false;
        return;
      }
      if (this.broke) {
        if (kind === "mousedown" || (kind === "keydown" && isDropKey(ev.key))) this._takeCredit();
        return;
      }
      if (this.panel !== null) {
        this._panelEvent(ev);
        return;
      }
      if (kind === "keydown") {
        const key = ev.key;
        if (isDropKey(key)) {
          if (ev.repeat) {
            if (this.auto === null && !this.hold) {
              this.hold = true;
              this.holdT = HOLD_DELAY;
            }
          } else this._startHold();
        } else if (key === "Escape") this._stopAuto(null);
        else if (key === "Left") this._setRows(this.rows - 1);
        else if (key === "Right") this._setRows(this.rows + 1);
        else if (key === "plus" || key === "KP_Add" || key === "Up" || key === "equal") this._setBet(1);
        else if (key === "minus" || key === "KP_Subtract" || key === "Down") this._setBet(-1);
        else if (RISK_KEYS.includes(key) || key === "KP_1" || key === "KP_2" || key === "KP_3") this._setRisk(Number(key[key.length - 1]) - 1);
        else if (ev.repeat) return;
        else if (key === "a" || key === "A") this._open("auto");
        else if (key === "s" || key === "S") this._open("stats");
        else if (key === "f" || key === "F") this._open("fair");
        else if (key === "t" || key === "T") this._toggleTurbo();
        return;
      }
      if (kind === "wheel") {
        this._setBet(ev.delta > 0 ? 1 : -1);
        return;
      }
      if (kind !== "mousedown" || ev.button !== 1) return;
      const pos = ev.pos;
      if (this.rDrop.collidepoint(pos)) this._startHold();
      else if (this.rMinus.collidepoint(pos)) this._setBet(-1);
      else if (this.rPlus.collidepoint(pos)) this._setBet(1);
      else if (this.rRowsM.collidepoint(pos)) this._setRows(this.rows - 1);
      else if (this.rRowsP.collidepoint(pos)) this._setRows(this.rows + 1);
      else {
        for (let i = 0; i < this.rRisks.length; i++) {
          if (this.rRisks[i].collidepoint(pos)) {
            this._setRisk(i);
            return;
          }
        }
        const names = ["auto", "stats", "fair", "turbo"];
        for (let i = 0; i < this.rIcons.length; i++) {
          if (this.rIcons[i].collidepoint(pos)) {
            if (names[i] === "turbo") this._toggleTurbo();
            else this._open(names[i]);
            return;
          }
        }
        if (this.boardBox.collidepoint(pos) && pos[1] < this.slotTop) this._startHold(); // Klick aufs Brett wirft ebenfalls
      }
    }

    // ----- Fenster
    _panelEvent(ev) {
      const kind = ev.kind;
      const name = this.panel;
      if (kind === "keydown") {
        const key = ev.key;
        const letter = { auto: ["a", "A"], stats: ["s", "S"], fair: ["f", "F"] }[name];
        if (key === "Escape" || (letter.includes(key) && !ev.repeat)) {
          this.panel = null;
          this.playSound("click");
          return;
        }
        if (name === "auto") this._autoKey(key, ev.repeat);
        else if (name === "stats") {
          if (key === "Left") this.statsRows = Math.max(L.ROWS[0], this.statsRows - 1);
          else if (key === "Right") this.statsRows = Math.min(L.ROWS[L.ROWS.length - 1], this.statsRows + 1);
        } else if (name === "fair") this._fairKey(key);
        return;
      }
      if (kind === "wheel" && name === "fair") {
        this._fairMove(ev.delta > 0 ? -1 : 1);
        return;
      }
      if (kind !== "mousedown" || ev.button !== 1) return;
      const pos = ev.pos;
      if (this.rClose.collidepoint(pos) || !this.pan.collidepoint(pos)) {
        this.panel = null;
        this.playSound("click");
        return;
      }
      for (const [rect, action] of this._panelHits()) {
        if (rect.collidepoint(pos)) {
          action();
          return;
        }
      }
    }

    _autoKey(key, repeat) {
      const rows = [AUTO_BALLS, AUTO_LIMITS, AUTO_LIMITS];
      if (key === "Up") this.autoFocus = PG.mod(this.autoFocus - 1, 4);
      else if (key === "Down") this.autoFocus = PG.mod(this.autoFocus + 1, 4);
      else if ((key === "Left" || key === "Right") && this.autoFocus < 3 && this.auto === null) {
        const d = key === "Left" ? -1 : 1;
        const opts = rows[this.autoFocus];
        const cur = [this.autoBalls, this.autoWin, this.autoLoss][this.autoFocus];
        const j = Math.max(0, Math.min(opts.length - 1, opts.indexOf(cur) + d));
        this._autoSet(this.autoFocus, opts[j]);
      } else if ((key === "Left" || key === "Right") && this.autoFocus === 3) this._toggleTurbo();
      else if ((key === "t" || key === "T") && !repeat) this._toggleTurbo();
      else if ((key === "Return" || key === "KP_Enter" || key === "space") && !repeat) {
        if (this.auto !== null) this._stopAuto(null);
        else this._startAuto();
      }
    }

    _autoSet(row, value) {
      if (this.auto !== null) return;
      if (row === 0) {
        this.autoBalls = value;
        this._setOpt("auto_balls", value);
      } else if (row === 1) {
        this.autoWin = value;
        this._setOpt("auto_win", value);
      } else {
        this.autoLoss = value;
        this._setOpt("auto_loss", value);
      }
      this.playSound("click");
    }

    _fairKey(key) {
      if (key === "Up" || key === "Left") this._fairMove(-1);
      else if (key === "Down" || key === "Right") this._fairMove(1);
      else if (key === "Return" || key === "KP_Enter" || key === "space") {
        const e = this._fairEntry();
        if (e !== null) this._verify(e);
      } else if (key === "c" || key === "C") this._newClient();
    }
    _fairEntries() {
      return this.fairlogShown.slice().reverse();
    }
    _fairEntry() {
      const es = this._fairEntries();
      if (!es.length) return null;
      this.fairSel = Math.max(0, Math.min(es.length - 1, this.fairSel));
      return es[this.fairSel];
    }
    _fairMove(d) {
      const es = this._fairEntries();
      if (!es.length) return;
      this.fairSel = Math.max(0, Math.min(es.length - 1, this.fairSel + d));
      const vis = this._fairVisible();
      if (this.fairSel < this.fairScroll) this.fairScroll = this.fairSel;
      else if (this.fairSel >= this.fairScroll + vis) this.fairScroll = this.fairSel - vis + 1;
      this.playSound("move");
    }

    // ===================================================== Update
    update(dt) {
      this.time += dt;
      if (this._lang !== PG.lang) {
        this._lang = PG.lang;
        this._layout();
      }
      const target = Math.max(0, bank.balance() - this.pending);
      const diff = target - this.shownChips;
      if (Math.abs(diff) < 0.5) {
        this.shownChips = target;
      } else {
        const step = Math.max(Math.abs(diff) * Math.min(1, dt * 6), 60 * dt);
        this.shownChips += Math.sign(diff) * Math.min(Math.abs(diff), step);
        if (diff > 0) {
          this._tickCd -= dt;
          if (this._tickCd <= 0) {
            this._tickCd = 0.07;
            this._tickN += 1;
            this.tone(1500 + ((this._tickN * 113) % 300), 0.025, "square", 0.08);
          }
        }
      }
      if (this.toast) {
        this.toast[1] -= dt;
        if (this.toast[1] <= 0) this.toast = null;
      }
      this.shake = Math.max(0, this.shake - dt);
      this.nod = Math.max(0, this.nod - dt);
      this.tickCd = Math.max(0, this.tickCd - dt);
      this.histAnim = Math.min(1, this.histAnim + dt / 0.22);
      if (this.banner !== null) {
        this.banner.t += dt;
        if (this.banner.t >= this.banner.dur) this.banner = null;
      }
      for (const row of this.pegFlash) {
        for (let j = 0; j < row.length; j++) if (row[j] > 0) row[j] = Math.max(0, row[j] - dt);
      }
      for (let i = 0; i < this.slotBump.length; i++) if (this.slotBump[i] > 0) this.slotBump[i] = Math.max(0, this.slotBump[i] - dt);
      this._updateFx(dt);
      this._updateBalls(dt);
      // Dauerfeuer (gehaltene Leertaste bzw. Maus auf WERFEN)
      if (this.hold && this.auto === null && this.panel === null) {
        this.holdT += dt;
        if (this.holdT >= HOLD_DELAY) {
          // Rest aufheben statt neu setzen: der Takt stimmt bei jeder Bildrate
          this.fireCd -= dt;
          let n = 0;
          while (this.fireCd <= 0 && n < 3) {
            this.fireCd += fireRate(this.rows) / this._speed();
            this._drop(true);
            n += 1;
          }
          if (this.fireCd <= 0) this.fireCd = 0;
        }
      }
      if (this.auto !== null) {
        this.autoCd -= dt;
        if (this.autoCd <= 0) {
          this.autoCd = AUTO_RATE / this._speed();
          this._autoStep();
        }
      }
    }

    _updateBalls(dt) {
      if (!this.balls.length) return;
      const sp = this._speed();
      const landed = [];
      for (const b of this.balls) {
        b.t += dt * sp;
        const n = b.rows;
        const st = segTime(n);
        while (b.hit < n && b.t >= DROP_T + b.hit * st) {
          const r = b.hit;
          const j = b.pre[r] + 1;
          if (r < this.pegFlash.length && j < this.pegFlash[r].length) this.pegFlash[r][j] = FLASH_T;
          if (this.tickCd <= 0) {
            this.tickCd = TICK_GAP;
            this.tone(880 + r * 40, 0.025, "sine", 0.05);
          }
          b.hit += 1;
        }
        if (b.t >= flightTime(n)) landed.push(b);
      }
      if (landed.length) {
        this.balls = this.balls.filter((b) => b.t < flightTime(b.rows));
        for (const b of landed) this._land(b);
        if (!this.balls.length) {
          if (this.auto === null) this._clampBet(); // (im Auto bleibt der Einsatz fest)
          this._checkBroke();
        }
      }
    }

    _updateFx(dt) {
      this.particles = this.particles.filter((p) => {
        p.t += dt;
        p.vy += p.g * dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        return p.t < p.life;
      });
      this.coinsFx = this.coinsFx.filter((c) => {
        c.vy += 900 * this.k * dt;
        c.x += c.vx * dt;
        c.y += c.vy * dt;
        c.spin += c.vs * dt;
        return c.y < this.height + 30;
      });
      for (const p of this.popups) p.t += dt;
      this.popups = this.popups.filter((p) => p.t < p.dur);
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      const W = this.width, H = this.height;
      const glowC = [trunc(this.cx), this.region.y + fdiv(this.region.h * 11, 20)]; // unabhängig von den Reihen
      const bg = D.background(W, H, glowC, trunc(Math.max(W, H) * 0.42), this.accent);
      ctx.drawImage(bg, 0, 0, bg.lw, bg.lh);
      let ox = 0;
      if (this.shake > 0) ox = trunc((Math.sin(this.time * 70) * 6 * this.k * this.shake) / 0.35);
      this._drawBoard(ctx, ox);
      this._drawLlama(ctx, ox);
      this._drawBalls(ctx, ox);
      this._drawParticles(ctx, ox);
      this._drawLeft(ctx);
      this._drawRight(ctx);
      this._drawHud(ctx);
      this._drawStrip(ctx);
      this._drawFx(ctx);
      if (this.banner !== null) this._drawBanner(ctx);
      if (this.panel === null && !this.broke) this._drawHover(ctx);
      if (this.panel === "auto") this._drawAuto(ctx);
      else if (this.panel === "stats") this._drawStats(ctx);
      else if (this.panel === "fair") this._drawFair(ctx);
      // Hinweis ÜBER den Fenstern (wie Python) - sonst bleibt z.B.
      // "Neuer Client-Seed" im Fairness-Fenster unsichtbar
      if (this.toast) this._drawToast(ctx);
      if (this.broke) this._drawBroke(ctx);
    }

    // ----- Brett
    _drawBoard(ctx, ox) {
      const bb = this.boardBox;
      const plate = D.boardPlate(this.platePts, bb.w, bb.h, this.accent);
      ctx.drawImage(plate, bb.x + ox, bb.y, plate.lw, plate.lh);
      const pegs = D.pegLayer(bb.w, bb.h, this.pegsLocal, this.pegR);
      ctx.drawImage(pegs, bb.x + ox, bb.y, pegs.lw, pegs.lh);
      this.pegFlash.forEach((row, r) => {
        row.forEach((f, j) => {
          if (f > 0) {
            const level = Math.max(1, Math.min(8, trunc((8 * f) / FLASH_T + 0.999)));
            const g = D.pegGlow(this.pegR, level);
            const [x, y] = this._pegXY(r, j);
            ctx.drawImage(g, ri(x) + ox - fdiv(g.lw, 2), ri(y) - fdiv(g.lh, 2), g.lw, g.lh);
          }
        });
      });
      const hot = this._hoverSlot();
      const table = L.table(this.rows, this.risk);
      const bumpPx = this.gap * 0.28;
      const sep = this._sep();
      this.slotRects.forEach((rect, i) => {
        const m100 = table[i];
        const img = D.slotSurface(rect.w, rect.h, m100, L.fmtShort(m100, sep), this.slotFont, i === hot);
        let dy = 0;
        if (this.slotBump[i] > 0) {
          const p = 1 - this.slotBump[i] / BUMP_T;
          dy = trunc(bumpPx * Math.sin(Math.PI * p) * (1 - p * 0.5));
        }
        ctx.drawImage(img, rect.x + ox, rect.y + dy, img.lw, img.lh);
      });
    }

    _drawLlama(ctx, ox) {
      const n = this.llamaN, k = this.k;
      // Sims unter dem Lama (endet knapp vor dem Einwurf)
      const ledgeR = trunc(this.cx - this.gap * 0.55) + ox;
      const lw = trunc(n * 1.05), lh = Math.max(4, trunc(7 * k));
      const ledge = new PG.Rect(ledgeR - lw, trunc(this.footY), lw, lh);
      draw.rect(ctx, ui.mix(this.accent, [0, 0, 0], 0.55), ledge.move(0, 2), 0, fdiv(lh, 2));
      draw.rect(ctx, ui.mix(this.accent, [40, 20, 10], 0.35), ledge, 0, fdiv(lh, 2));
      draw.line(ctx, ui.mix(this.accent, [255, 255, 255], 0.4), [ledge.x + 3, ledge.y], [ledge.right - 3, ledge.y], 1);
      // Trichter über dem ersten Stift (das Wollknäuel fällt aus dem Maul hinein)
      const fw = Math.max(this.ballR * 3 + 6, trunc(this.gap * 1.35));
      const fbot = trunc(this.top - this.gap * 0.42);
      const fh = Math.max(8, Math.min(trunc(this.gap * 1.0), fbot - trunc(this.mouth[1] + this.ballR + 2)));
      const fun = D.funnel(fw, fh, this.accent);
      ctx.drawImage(fun, trunc(this.cx - fw / 2) + ox, fbot - fh, fun.lw, fun.lh);
      let deg = 0;
      if (this.nod > 0) {
        const p = 1 - this.nod / NOD_T;
        deg = -ri(10 * Math.sin(Math.PI * p));
      }
      D.drawLlama(ctx, n, deg, this.accent, this.footX + ox, this.footY);
      // nächstes Wollknäuel im Maul (wenn ein Wurf möglich ist)
      const ready = !this.broke && this.nod <= 0 && this.balls.length < L.MAX_BALLS && this.bet <= this._avail();
      if (ready) {
        const [mx, my] = this.mouth;
        const ball = D.yarnBall(this.mouthR, 0);
        ctx.drawImage(ball, trunc(mx) - this.mouthR - 1 + ox, trunc(my) - this.mouthR - 1, ball.lw, ball.lh);
      }
    }

    _drawBalls(ctx, ox) {
      for (const b of this.balls) {
        const t0 = b.t;
        for (let i = TRAIL; i > 0; i--) {
          const tt = t0 - i * TRAIL_DT;
          if (tt <= 0) continue;
          const [x, y] = this._ballPos(b, tt);
          const a = fdiv(150 * (TRAIL + 1 - i), TRAIL + 1);
          const rr = Math.max(1, fdiv(this._ballRadius(tt) * (TRAIL + 2 - i), TRAIL + 2));
          const dot = D.trailDot(rr, a);
          ctx.drawImage(dot, trunc(x) - rr - 1 + ox, trunc(y) - rr - 1, dot.lw, dot.lh);
        }
      }
      for (const b of this.balls) {
        const [x, y] = this._ballPos(b, b.t);
        const step = (b.spin + trunc(b.t * 16)) % 12;
        const br = this._ballRadius(b.t);
        const img = D.yarnBall(br, step); // gecacht je Radius + Drehstufe
        ctx.drawImage(img, trunc(x) - br - 1 + ox, trunc(y) - br - 1, img.lw, img.lh);
      }
    }

    _drawParticles(ctx, ox) {
      for (const p of this.particles) {
        const rr = p.r * (1 - p.t / p.life);
        if (rr >= 0.5) draw.circle(ctx, p.col, [trunc(p.x + ox), trunc(p.y)], Math.max(1, trunc(rr)));
      }
    }

    // ----- Seitenleisten
    _drawLeft(ctx) {
      const k = this.k;
      const box = this.left.copy();
      box.h = this.infoY - box.y + 2 * gh(this.fTiny) + trunc(14 * k);
      ui.drawPanel(ctx, box, { accentTop: ui.mix(this.accent, CD.COL_GOLD, 0.3) });
      const pad = Math.max(5, trunc(9 * k));
      const locked = this.balls.length > 0 || this.auto !== null;
      ui.text(ctx, t("lpl.risk"), box.x + pad, box.y + pad - 2, this.fTiny, ui.TEXT_DIM);
      this.rRisks.forEach((r, i) => {
        const risk = L.RISKS[i];
        this._btn(ctx, r, `${i + 1}  ${this._riskName(risk)}`, {
          active: risk === this.risk, on: !locked || risk === this.risk, hot: r.collidepoint(this.mouse) && !locked,
        });
      });
      ui.text(ctx, t("lpl.rows"), box.x + pad, this.lblRowsY - 2, this.fTiny, ui.TEXT_DIM);
      this._iconBtn(ctx, this.rRowsM, "left", { on: !locked && this.rows > L.ROWS[0] });
      this._iconBtn(ctx, this.rRowsP, "right", { on: !locked && this.rows < L.ROWS[L.ROWS.length - 1] });
      const rv = this.rRowsV;
      const rad = Math.max(6, fdiv(rv.h, 4));
      draw.rect(ctx, [16, 12, 26], rv, 0, rad);
      draw.rect(ctx, ui.BORDER, rv, 1, rad);
      ui.text(ctx, String(this.rows), icx(rv), icy(rv), this.fBtn, !locked ? ui.GOLD : ui.TEXT_DIM, "center");
      const top = Math.max(...L.table(this.rows, this.risk));
      let y = this.infoY;
      for (const [txt, col] of [[t("lpl.max", { m: L.fmtShort(top, this._sep()) }), CD.COL_GOLD],
        [t("lpl.rtp", { p: this._rtpText() }), ui.TEXT_DIM]]) {
        let f = this.fTiny;
        if (f.width(txt) > box.w - 2 * pad) f = D.fitFont(txt, box.w - 2 * pad, gh(this.fTiny), false);
        ui.text(ctx, txt, box.x + pad, y, f, col);
        y += gh(this.fTiny) + 2;
      }
      this._drawSession(ctx, box.bottom + this.margin);
    }

    /** Kasten "Sitzung" unter den Einstellungen (nur wenn Platz ist). */
    _drawSession(ctx, top) {
      const k = this.k;
      const pad = Math.max(5, trunc(9 * k));
      const lh = gh(this.fTiny);
      const box = new PG.Rect(this.left.x, top, this.left.w, pad * 2 + lh + 3 * (lh + 3) + 2);
      if (box.bottom > this.left.bottom) return;
      ui.drawPanel(ctx, box, { accentTop: ui.mix(this.accent, CD.COL_GOLD, 0.3) });
      ui.text(ctx, t("lpl.session"), box.x + pad, box.y + pad - 2, this.fTiny, ui.TEXT_DIM);
      const net = this.sess.net;
      const rows = [
        [t("chip.auto_balls"), String(this.sess.balls), ui.TEXT],
        [t("chip.st_net"), (net > 0 ? "+" : "") + net, net > 0 ? ui.GREEN : net < 0 ? ui.RED : ui.TEXT],
        [t("lpl.best"), this.sess.best ? this._fmt(this.sess.best) : "–", CD.COL_GOLD],
      ];
      let y = box.y + pad + lh + 2;
      const w = box.w - 2 * pad;
      for (const [lbl, val, col] of rows) {
        let vf = this.fBtn;
        if (vf.width(val) > fdiv(w, 2)) vf = D.fitFont(val, fdiv(w, 2), gh(this.fTiny));
        const lf = D.fitFont(lbl, w - vf.width(val) - 6, lh, false);
        const cy = y + fdiv(lh + 3, 2);
        ui.text(ctx, lbl, box.x + pad, cy, lf, ui.TEXT_DIM, "midleft");
        ui.text(ctx, val, box.right - pad, cy, vf, col, "midright");
        y += lh + 3;
      }
    }

    _drawRight(ctx) {
      const k = this.k;
      const pad = Math.max(5, trunc(9 * k));
      const ph = this.pillH;
      const nVis = L.HISTORY;
      const head = gh(this.fTiny) + pad;
      const box = this.right.copy();
      box.h = Math.min(this.right.h, head + pad + nVis * (ph + 3));
      ui.drawPanel(ctx, box, { accentTop: ui.mix(this.accent, CD.COL_GOLD, 0.3) });
      ui.text(ctx, t("chip.history"), box.x + pad, box.y + pad - 2, this.fTiny, ui.TEXT_DIM);
      const y0 = box.y + head + 2;
      const w = box.w - 2 * pad;
      const slide = (1 - this.histAnim) * (ph + 3);
      ctx.save();
      ctx.beginPath();
      ctx.rect(box.x, y0 - 2, box.w, box.bottom - y0);
      ctx.clip();
      const ghost = ui.mix(ui.PANEL, ui.BG_TOP, 0.45);
      for (let i = this.histShown.length; i < nVis; i++) {
        const y = trunc(y0 + i * (ph + 3));
        if (y > box.bottom - ph) break;
        draw.rect(ctx, ghost, [box.x + pad, y, w, ph - 2], 0, fdiv(ph, 2));
      }
      const rev = this.histShown.slice().reverse();
      for (let i = 0; i < rev.length; i++) {
        const m100 = rev[i];
        const y = trunc(y0 + i * (ph + 3) - slide);
        if (y > box.bottom - ph) break;
        const img = D.pill(w, ph, m100, this._fmt(m100), Math.max(9, trunc(ph * 0.62)));
        if (i === 0 && this.histAnim < 1) {
          ctx.save();
          ctx.globalAlpha *= trunc(255 * this.histAnim) / 255;
          ctx.drawImage(img, box.x + pad, y, img.lw, img.lh);
          ctx.restore();
        } else {
          ctx.drawImage(img, box.x + pad, y, img.lw, img.lh);
        }
      }
      ctx.restore();
    }

    // ----- HUD + Leiste
    _drawHud(ctx) {
      const W = this.width;
      const g = ctx.createLinearGradient(0, 0, 0, this.hudH);
      g.addColorStop(0, "rgba(10,8,16,0.824)");
      g.addColorStop(1, "rgba(10,8,16,0.588)");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W, this.hudH);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [0, this.hudH - 1], [W, this.hudH - 1], 1);
      const cy = fdiv(this.hudH, 2);
      const pad = this.margin;
      const coinR = Math.max(6, trunc(this.hudH * 0.26));
      const coin = CD.symbolCanvas("coin", coinR * 2 + 4);
      ctx.drawImage(coin, pad - 2, cy - fdiv(coin.lh, 2), coin.lw, coin.lh);
      const left = ui.text(ctx, `${t("cas.chips")}: ${ri(this.shownChips)}`, pad + coinR * 2 + 6, cy, this.fBig, ui.GOLD, "midleft");
      const rightTxt = `${t("cas.record")}: ${this.recShown}   ·   ${t("cas.credits", { n: bank.refills() })}`;
      const rf = this.fSmall.width(rightTxt) > W * 0.42 ? this.fTiny : this.fSmall;
      const rr = ui.text(ctx, rightTxt, W - pad, cy, rf, ui.TEXT_DIM, "midright");
      const parts = [];
      if (this.balls.length) parts.push(t("lpl.in_flight", { n: this.balls.length }));
      if (this.auto !== null) parts.push(t("chip.auto_left", { n: this.auto.left }));
      if (this.turbo) parts.push(t("chip.turbo"));
      if (!parts.length) return;
      const midTxt = parts.join("   ·   ");
      const x0 = pad + coinR * 2 + 6 + left.w + pad;
      const x1 = rr.x - pad;
      let mf = this.fSmall;
      if (mf.width(midTxt) > x1 - x0) mf = this.fTiny;
      if (mf.width(midTxt) <= x1 - x0) ui.text(ctx, midTxt, fdiv(x0 + x1, 2), cy, mf, ui.TEXT, "center");
    }

    _drawStrip(ctx) {
      const st = this.strip;
      const g = ctx.createLinearGradient(0, st.y, 0, st.bottom);
      g.addColorStop(0, "rgba(10,8,16,0.667)");
      g.addColorStop(1, "rgba(10,8,16,0.902)");
      ctx.fillStyle = g;
      ctx.fillRect(st.x, st.y, st.w, st.h);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), st.topleft, st.topright, 2);
      const hot = this.mouse;
      const free = this.auto === null;
      this._btn(ctx, this.rMinus, "–", { on: free && this.bet > L.BETS[0], hot: this.rMinus.collidepoint(hot) });
      this._btn(ctx, this.rPlus, "+", { on: free && this.bet < L.BETS[L.BETS.length - 1], hot: this.rPlus.collidepoint(hot) });
      const rb = this.rBet;
      const rad = Math.max(6, fdiv(rb.h, 4));
      draw.rect(ctx, [16, 12, 26], rb, 0, rad);
      draw.rect(ctx, ui.BORDER, rb, 1, rad);
      ui.text(ctx, t("cas.bet"), icx(rb), rb.y + 2, this.fTiny, ui.TEXT_DIM, "midtop");
      const col = this.bet > this._avail() ? ui.RED : ui.GOLD;
      ui.text(ctx, String(this.bet), icx(rb), rb.bottom - 3, this.fBtn, col, "midbottom");
      const names = ["auto", "stats", "fair", "turbo"];
      const letters = ["A", "S", "F", "T"];
      this.rIcons.forEach((r, i) => {
        const name = names[i];
        const active = (name === "turbo" && this.turbo) || (name === "auto" && this.auto !== null);
        this._iconBtn(ctx, r, name, { active, hot: r.collidepoint(hot), letter: letters[i] });
      });
      if (this.auto !== null) {
        this._btn(ctx, this.rDrop, t("chip.auto_stop"), { primary: true, hot: this.rDrop.collidepoint(hot) });
      } else {
        const ok = !this.broke && this.bet <= this._avail() && this.balls.length < L.MAX_BALLS;
        this._btn(ctx, this.rDrop, t("lpl.drop"), { primary: true, on: ok, hot: this.rDrop.collidepoint(hot) || this.hold });
      }
    }

    _pulse(speed, phase = 0) {
      return 0.5 + 0.5 * Math.sin(this.time * speed + phase);
    }

    _btn(ctx, rect, label, { primary = false, on = true, hot = false, active = false } = {}) {
      const r = rect;
      const rad = Math.max(6, fdiv(r.h, 4));
      let col;
      if (primary && on) {
        const base = ui.mix(this.accent, [0, 0, 0], 0.18);
        const fill = ui.mix(base, [255, 255, 255], hot ? 0.12 : 0);
        draw.rect(ctx, ui.mix(base, [0, 0, 0], 0.4), r.move(0, 2), 0, rad);
        draw.rect(ctx, fill, r, 0, rad);
        draw.rect(ctx, ui.mix(CD.COL_GOLD, [255, 255, 255], 0.35 * this._pulse(2.4)), r, 2, rad);
        col = [255, 255, 255];
      } else {
        draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        col = on ? ui.TEXT : ui.TEXT_FAINT;
      }
      let f = this.fBtn;
      if (f.width(label) > r.w - 8) f = this.fTiny;
      if (f.width(label) > r.w - 4) f = D.fitFont(label, r.w - 4, gh(this.fTiny), false);
      ui.text(ctx, label, icx(r), icy(r), f, col, "center");
    }

    _iconBtn(ctx, rect, name, { on = true, hot = false, active = false, letter = null } = {}) {
      const r = rect;
      const rad = Math.max(5, fdiv(r.h, 4));
      draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
      draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
      const col = on ? (active ? CD.COL_GOLD : ui.TEXT) : ui.TEXT_FAINT;
      const size = Math.max(8, trunc(r.h * (letter ? 0.52 : 0.5)));
      const img = D.icon(name, size, col);
      const cy = icy(r) - (letter ? fdiv(r.h, 14) : 0);
      ctx.drawImage(img, icx(r) - fdiv(size, 2), cy - fdiv(size, 2), img.lw, img.lh);
      if (letter && r.h >= 26) {
        const f = ui.font(Math.max(8, trunc(r.h * 0.26)), true);
        ui.text(ctx, letter, r.right - 3, r.bottom - 1, f, ui.TEXT_DIM, "bottomright");
      }
    }

    _drawFx(ctx) {
      for (const c of this.coinsFx) {
        const w = Math.max(2, trunc(c.r * 2 * Math.abs(Math.cos(c.spin))));
        const h = trunc(c.r * 2);
        const x0 = trunc(c.x) - fdiv(w, 2), y0 = trunc(c.y) - fdiv(h, 2);
        draw.ellipse(ctx, CD.COL_GOLD_D, [x0 + 1, y0 + 1, w, h]);
        draw.ellipse(ctx, [246, 196, 60], [x0, y0, w, h]);
        if (w > 6) {
          // rect.inflate(-w // 2, -h // 2)
          const dx = -fdiv(w, 2), dy = -fdiv(h, 2);
          draw.ellipse(ctx, [255, 236, 150], [x0 - trunc(dx / 2), y0 - trunc(dy / 2), w + dx, h + dy]);
        }
      }
      for (const p of this.popups) {
        const f = p.t / p.dur;
        const fnt = p.big ? this.fBig : this.fBtn;
        const alpha = f > 0.6 ? trunc(255 * (1 - (f - 0.6) / 0.4)) / 255 : 1;
        const y = p.y - (p.big ? 46 : 30) * this.k * (1 - (1 - f) ** 2);
        const w = fnt.width(p.text), h = rh(fnt);
        const r = new PG.Rect(trunc(p.x) - fdiv(w, 2), trunc(y) - h, w, h).clamp(new PG.Rect(2, 0, this.width - 4, this.height));
        ui.text(ctx, p.text, r.x + 1, r.y + 1, fnt, [10, 6, 16], "topleft", alpha);
        ui.text(ctx, p.text, r.x, r.y, fnt, p.color, "topleft", alpha);
      }
    }

    _drawBanner(ctx) {
      const b = this.banner;
      const f = b.t / b.dur;
      const k = this.k;
      const W = this.width;
      const grow = Math.min(1, b.t / 0.18);
      const tf = this.fHuge.width(b.title) > W - 40 ? this.fBig : this.fHuge;
      const sf = this.fBig.width(b.sub) > W - 40 ? this.fSmall : this.fBig;
      const pw = Math.min(W - 20, Math.max(tf.width(b.title), sf.width(b.sub)) + trunc(48 * k));
      const ph = rh(tf) + rh(sf) + trunc(30 * k);
      const w = trunc(pw * (0.7 + 0.3 * grow)), h = trunc(ph * (0.7 + 0.3 * grow));
      const cx = trunc(this.cx), cy = this.top + trunc((this.slotTop - this.top) * 0.38);
      const panel = new PG.Rect(cx - fdiv(w, 2), cy - fdiv(h, 2), w, h);
      ctx.save();
      if (f > 0.8) ctx.globalAlpha *= trunc(255 * (1 - (f - 0.8) / 0.2)) / 255;
      ui.drawPanel(ctx, panel, { accentTop: CD.COL_GOLD });
      if (grow >= 1) {
        ui.text(ctx, b.title, icx(panel), panel.y + trunc(10 * k), tf, CD.COL_GOLD, "midtop");
        ui.text(ctx, b.sub, icx(panel), panel.bottom - trunc(10 * k), sf, ui.TEXT, "midbottom");
      }
      ctx.restore();
    }

    _drawToast(ctx) {
      const [text, left] = this.toast;
      const f = this.fSmall.width(text) > this.width - 40 ? this.fTiny : this.fSmall;
      const bw = f.width(text) + 24, bh = rh(f) + 12;
      const box = new PG.Rect(trunc(this.cx) - fdiv(bw, 2), trunc(this.top + (this.slotTop - this.top) * 0.16), bw, bh)
        .clamp(new PG.Rect(4, 0, this.width - 8, this.height));
      ctx.save();
      if (left < 0.3) ctx.globalAlpha *= trunc((255 * left) / 0.3) / 255;
      draw.rect(ctx, [12, 8, 20, 225], box, 0, fdiv(bh, 2));
      draw.rect(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.4), box, 1, fdiv(bh, 2));
      ui.text(ctx, text, box.x + fdiv(bw, 2), box.y + fdiv(bh, 2), f, ui.TEXT, "center");
      ctx.restore();
    }

    _hoverSlot() {
      if (this.panel !== null) return null;
      for (let i = 0; i < this.slotRects.length; i++) {
        const r = this.slotRects[i];
        if (new PG.Rect(r.x - 1, r.y - 3, r.w + 2, r.h + 6).collidepoint(this.mouse)) return i;
      }
      return null;
    }

    _drawHover(ctx) {
      const i = this._hoverSlot();
      const sep = this._sep();
      if (i !== null) {
        const m100 = L.mult100(this.rows, this.risk, i);
        const lines = [this._fmt(m100) + "   ·   " + t("lpl.slot", { n: i }),
          t("lpl.tip_chance", { p: L.fmtProb(this.rows, i, sep) }),
          t("lpl.tip_win", { b: this.bet, n: L.fmtShort(this.bet * m100, sep) })];
        PG.charts.tooltip(ctx, this.mouse, lines);
        return;
      }
      const tips = [[this.rIcons[0], t("chip.auto_title") + "  (A)"], [this.rIcons[1], t("chip.stats") + "  (S)"],
        [this.rIcons[2], t("chip.fair_title") + "  (F)"], [this.rIcons[3], t("chip.turbo") + "  (T)"]];
      for (const [r, txt] of tips) {
        if (r.collidepoint(this.mouse)) {
          PG.charts.tooltip(ctx, this.mouse, [txt]);
          return;
        }
      }
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const cx = fdiv(this.width, 2), cy = fdiv(this.height, 2);
      const pw = Math.min(this.width - 40, trunc(480 * Math.max(0.8, this.k)));
      const ph = trunc(200 * Math.max(0.75, this.k));
      const panel = new PG.Rect(cx - fdiv(pw, 2), cy - fdiv(ph, 2), pw, ph);
      ui.drawPanel(ctx, panel, { accentTop: ui.RED });
      ui.text(ctx, t("cas.broke"), cx, trunc(panel.y + ph * 0.28), this.fHuge, ui.RED, "center");
      [[t("cas.broke_sub"), ui.TEXT_DIM], [t("cas.broke_restart", { n: bank.START_CHIPS }), ui.TEXT]].forEach(([txt, col], i) => {
        const f = this.fSmall.width(txt) > pw - 16 ? this.fTiny : this.fSmall;
        ui.text(ctx, txt, cx, trunc(panel.y + ph * (0.58 + 0.2 * i)), f, col, "center");
      });
    }

    // ===================================================== Fenster
    _panelFrame(ctx, iconName, title) {
      draw.rect(ctx, [6, 4, 12, 150], [0, 0, this.width, this.height]);
      ui.drawPanel(ctx, this.pan, { accentTop: this.accent });
      const k = this.k;
      const p = this.pan;
      const hy = p.y + fdiv(this.panHead, 2);
      const isz = Math.max(12, trunc(this.panHead * 0.5));
      const ic = D.icon(iconName, isz, CD.COL_GOLD);
      ctx.drawImage(ic, p.x + this.margin, hy - fdiv(isz, 2), ic.lw, ic.lh);
      ui.text(ctx, title, p.x + this.margin + isz + trunc(8 * k), hy, this.fBig, ui.TEXT, "midleft");
      this._iconBtn(ctx, this.rClose, "close", { hot: this.rClose.collidepoint(this.mouse) });
      draw.line(ctx, ui.BORDER, [p.x + 8, p.y + this.panHead], [p.right - 8, p.y + this.panHead], 1);
      return this._panelBody();
    }

    /** Klickflächen des offenen Fensters: [[rect, aktion], ...]. */
    _panelHits() {
      if (this.panel === "auto") return this._autoRects()[1];
      if (this.panel === "stats") {
        const [rM, rP] = this._statsRowBtns(this._panelBody());
        return [[rM, () => { this.statsRows = Math.max(L.ROWS[0], this.statsRows - 1); }],
          [rP, () => { this.statsRows = Math.min(L.ROWS[L.ROWS.length - 1], this.statsRows + 1); }]];
      }
      if (this.panel === "fair") return this._fairRects()[2];
      return [];
    }

    _panelBody() {
      const p = this.pan;
      return new PG.Rect(p.x + this.margin, p.y + this.panHead + fdiv(this.margin, 2), p.w - 2 * this.margin,
        p.h - this.panHead - this.margin);
    }

    // ----- Auto
    _autoMetrics() {
      const k = this.k;
      return { lh: gh(this.fTiny), bh: Math.max(22, trunc(30 * k)), gap: Math.max(3, trunc(5 * k)),
        rowGap: Math.max(6, trunc(12 * k)), sep: Math.max(8, trunc(14 * k)), sumH: gh(this.fSmall),
        startH: Math.max(30, trunc(42 * k)), startGap: Math.max(6, trunc(10 * k)) };
    }
    _autoContentH() {
      const a = this._autoMetrics();
      return 3 * (a.lh + 3 + a.bh + a.rowGap) + a.bh + a.sep + a.sumH + a.startGap + a.startH;
    }
    /** Geometrie des Auto-Fensters: [[Zeilen, Turbo, Summen-y, Start], Klickflächen]. */
    _autoRects() {
      const body = this._panelBody();
      const a = this._autoMetrics();
      const { lh, bh, gap } = a;
      const rows = [], hits = [];
      let y = body.y;
      [AUTO_BALLS, AUTO_LIMITS, AUTO_LIMITS].forEach((opts, ri0) => {
        y += lh + 3;
        const bw = fdiv(body.w - (opts.length - 1) * gap, opts.length);
        const rects = [];
        opts.forEach((v, j) => {
          const r = new PG.Rect(body.x + j * (bw + gap), y, bw, bh);
          rects.push([r, v]);
          hits.push([r, () => {
            this.autoFocus = ri0;
            this._autoSet(ri0, v);
          }]);
        });
        rows.push([y - lh - 3, rects]);
        y += bh + a.rowGap;
      });
      const turbo = new PG.Rect(body.x, y, Math.min(body.w, Math.max(trunc(body.w * 0.36),
        this.fBtn.width(t("chip.turbo")) + 3 * bh + 10)), bh);
      hits.push([turbo, () => {
        this.autoFocus = 3;
        this._toggleTurbo();
      }]);
      y += bh + a.sep;
      const sumY = y;
      y += a.sumH + a.startGap;
      const startH = a.startH;
      const start = new PG.Rect(body.x + fdiv(body.w, 4), Math.min(body.bottom - startH, y), fdiv(body.w, 2), startH);
      hits.push([start, () => {
        if (this.auto !== null) this._stopAuto(null);
        else this._startAuto();
      }]);
      return [[rows, turbo, sumY, start], hits];
    }

    _drawAuto(ctx) {
      this._panelFrame(ctx, "auto", t("chip.auto_title"));
      const [[rows, turbo, sumY, start]] = this._autoRects();
      const labels = [t("chip.auto_balls"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
      const cur = [this.autoBalls, this.autoWin, this.autoLoss];
      const running = this.auto !== null;
      rows.forEach(([ly, rects], r0) => {
        ui.text(ctx, labels[r0], rects[0][0].x, ly, this.fTiny, this.autoFocus === r0 ? ui.TEXT : ui.TEXT_DIM);
        for (const [r, v] of rects) {
          const txt = r0 === 0 || v ? String(v) : t("chip.auto_off");
          this._btn(ctx, r, txt, { active: v === cur[r0], on: !running || v === cur[r0], hot: r.collidepoint(this.mouse) && !running });
        }
      });
      this._toggleBtn(ctx, turbo, t("chip.turbo"), this.turbo, this.autoFocus === 3);
      const total = this.autoBalls * this.bet;
      const txt = t("lpl.auto_sum", { n: this.autoBalls, b: this.bet, t: total });
      const col = this.bet > this._avail() ? ui.RED : ui.TEXT_DIM;
      const f = this.fSmall.width(txt) > this.pan.w - 2 * this.margin ? this.fTiny : this.fSmall;
      ui.text(ctx, txt, icx(this.pan), sumY, f, col, "midtop");
      if (running) {
        this._btn(ctx, start, t("chip.auto_stop") + "  ·  " + t("chip.auto_left", { n: this.auto.left }), {
          primary: true, hot: start.collidepoint(this.mouse) });
      } else {
        this._btn(ctx, start, t("chip.auto_start"), { primary: true, on: !this.broke, hot: start.collidepoint(this.mouse) });
      }
    }

    _toggleBtn(ctx, r, label, on, focus = false) {
      const hot = r.collidepoint(this.mouse);
      const rad = Math.max(6, fdiv(r.h, 4));
      draw.rect(ctx, hot || focus ? ui.BTN_SEL : ui.BTN, r, 0, rad);
      draw.rect(ctx, on ? CD.COL_GOLD : ui.BORDER_LIGHT, r, on ? 2 : 1, rad);
      const sww = trunc(r.h * 1.5), swh = trunc(r.h * 0.56);
      const sw = new PG.Rect(r.x + 8, icy(r) - fdiv(swh, 2), sww, swh);
      draw.rect(ctx, on ? CD.COL_GOLD : [60, 60, 76], sw, 0, fdiv(swh, 2));
      const knob = fdiv(swh, 2) - 2;
      const kx = on ? sw.right - knob - 3 : sw.x + knob + 3;
      draw.circle(ctx, [255, 255, 255], [kx, icy(sw)], knob);
      const isz = Math.max(8, trunc(r.h * 0.5));
      const ic = D.icon("turbo", isz, on ? CD.COL_GOLD : ui.TEXT_DIM);
      ctx.drawImage(ic, sw.right + 6, icy(r) - fdiv(isz, 2), ic.lw, ic.lh);
      ui.text(ctx, label, sw.right + 10 + isz, icy(r), this.fBtn, ui.TEXT, "midleft");
    }

    // ----- Statistik
    _statsRowBtns(body) {
      const bh = Math.max(18, trunc(24 * this.k));
      const y = this._statsDistY(body);
      const rP = new PG.Rect(body.right - bh, y, bh, bh);
      const rM = new PG.Rect(rP.x - bh - 4, y, bh, bh);
      return [rM, rP];
    }
    _statsDistY(body) {
      const tileH = Math.max(34, trunc(46 * this.k));
      return body.y + 3 * (tileH + 4) + Math.max(6, trunc(10 * this.k));
    }

    _drawStats(ctx) {
      const body = this._panelFrame(ctx, "stats", t("chip.stats"));
      const st = this.statsShown;
      const k = this.k;
      const sep = this._sep();
      if (st.rounds <= 0) {
        ui.text(ctx, t("chip.st_empty"), icx(body), icy(body), this.fSmall, ui.TEXT_DIM, "center");
        return;
      }
      const rtp = CS.rtp(st);
      const rtpTxt = rtp === null ? "–" : this._pct(st.paid, st.wagered);
      const tiles = [
        [t("chip.st_rounds"), String(st.rounds), ui.TEXT],
        [t("chip.st_wins"), String(st.wins), ui.GREEN],
        [t("chip.st_losses"), String(st.losses), ui.RED],
        [t("chip.st_wagered"), String(st.wagered), ui.TEXT],
        [t("chip.st_paid"), String(st.paid), ui.TEXT],
        [t("chip.st_net"), (st.net > 0 ? "+" : "") + st.net, st.net >= 0 ? ui.GREEN : ui.RED],
        [t("chip.st_best_win"), "+" + st.best_win, CD.COL_GOLD],
        [t("chip.st_best_mult"), "×" + L.fmtShort(st.best_mult, sep), CD.COL_GOLD],
        [t("chip.st_rtp"), rtpTxt + " %", ui.TEXT],
      ];
      const tileH = Math.max(34, trunc(46 * k));
      const half = fdiv(body.w, 2) - 4;
      const tw = fdiv(half - 2 * 4, 3);
      const tileBg = ui.mix(ui.PANEL, ui.BG_TOP, 0.35);
      tiles.forEach(([lbl, val, col], i) => {
        const r = new PG.Rect(body.x + (i % 3) * (tw + 4), body.y + fdiv(i, 3) * (tileH + 4), tw, tileH);
        draw.rect(ctx, tileBg, r, 0, Math.max(4, fdiv(r.h, 5)));
        ui.text(ctx, lbl, icx(r), r.y + 3, D.fitFont(lbl, r.w - 6, gh(this.fTiny), false), ui.TEXT_DIM, "midtop");
        ui.text(ctx, val, icx(r), r.bottom - 3, D.fitFont(val, r.w - 6, gh(this.fBtn)), col, "midbottom");
      });
      // Bilanzkurve rechts
      const cr = new PG.Rect(body.x + half + 8, body.y, body.w - half - 8, 3 * (tileH + 4) - 4);
      draw.rect(ctx, tileBg, cr, 0, Math.max(4, trunc(6 * k)));
      let cap = t("chip.st_curve", { n: st.curve.length });
      const capF = D.fitFont(cap, cr.w - 8, gh(this.fTiny), false);
      ui.text(ctx, cap, cr.x + 6, cr.y + 3, capF, ui.TEXT_DIM);
      const plot = new PG.Rect(cr.x + 6, cr.y + rh(capF) + 8, cr.w - 12, cr.h - rh(capF) - 14);
      if (st.curve.length >= 2) {
        const col = st.curve[st.curve.length - 1] >= 0 ? ui.GREEN : ui.RED;
        PG.charts.line(ctx, plot, [0].concat(st.curve), { color: col, fmt: (v) => String(ri(v)) });
      }
      // Treffer-Verteilung (gewählte Reihenzahl) gegen die erwartete Binomialkurve
      const n = this.statsRows;
      cap = t("lpl.st_dist", { n });
      const [rM, rP] = this._statsRowBtns(body);
      this._iconBtn(ctx, rM, "left", { on: n > L.ROWS[0], hot: rM.collidepoint(this.mouse) });
      this._iconBtn(ctx, rP, "right", { on: n < L.ROWS[L.ROWS.length - 1], hot: rP.collidepoint(this.mouse) });
      ui.text(ctx, cap, body.x, icy(rM), D.fitFont(cap, rM.x - body.x - 8, gh(this.fSmall)), ui.TEXT, "midleft");
      const hits = st.x.hits[String(n)];
      let total = 0;
      for (const v of hits) total += v;
      const area = new PG.Rect(body.x, rM.bottom + 6, body.w, body.bottom - rM.bottom - 6);
      const leg = t("lpl.st_legend");
      const legF = D.fitFont(leg, area.w, gh(this.fTiny), false);
      ui.text(ctx, leg, area.x, area.bottom, legF, ui.TEXT_FAINT, "bottomleft");
      area.h -= rh(legF) + 4;
      if (total <= 0) {
        ui.text(ctx, t("lpl.st_none", { n }), icx(area), icy(area), this.fSmall, ui.TEXT_DIM, "center");
        return;
      }
      const exp = [];
      for (let i = 0; i <= n; i++) exp.push((total * L.comb(n, i)) / Math.pow(2, n));
      const top = Math.max(Math.max(...hits), Math.max(...exp));
      const table = L.table(n, this.risk);
      const cols = table.map((m) => ui.mix(D.slotColor(m), ui.PANEL, 0.25));
      let hover = null;
      const lbls = area.w / (n + 1) >= 16 ? hits.map((_, i) => String(i)) : null;
      const labelSize = Math.max(9, trunc(10 * k));
      const hitsR = PG.charts.bars(ctx, area, hits, { labels: lbls, colors: cols, maxValue: top, hover: this.statsHover, labelSize });
      // erwartete Kurve: Punkte auf Höhe des Erwartungswerts über den Balkenmitten
      const fntH = lbls ? ui.font(labelSize).height + 2 : 0;
      const valH = ui.font(labelSize, true).height + 2;
      const plotH = Math.max(8, area.h - fntH - valH);
      const pts = [];
      for (const [cell, i] of hitsR) {
        const y = area.y + valH + plotH - plotH * (exp[i] / top);
        pts.push([trunc(cell.x + cell.w / 2), trunc(y)]);
        if (cell.collidepoint(this.mouse)) hover = i;
      }
      if (pts.length >= 2) draw.lines(ctx, CD.COL_GOLD, false, pts, 2);
      for (const p of pts) {
        draw.circle(ctx, CD.COL_GOLD, [p[0] + 0.5, p[1] + 0.5], 3);
        draw.circle(ctx, [40, 26, 8], [p[0] + 0.5, p[1] + 0.5], 1);
      }
      this.statsHover = hover;
      if (hover !== null) {
        const ev = trunc(exp[hover] * 10 + 0.5);
        PG.charts.tooltip(ctx, this.mouse, [t("lpl.slot", { n: hover }) + "   ·   " + this._fmt(table[hover]),
          t("lpl.st_bar", { n: hits[hover], e: `${fdiv(ev, 10)}${sep}${ev % 10}` })]);
      }
    }

    /** Prozent mit zwei Nachkommastellen (ganzzahlig gerundet). */
    _pct(num, den) {
      const v = fdiv(num * 20000 + den, 2 * den);
      return `${fdiv(v, 100)}${this._sep()}${String(v % 100).padStart(2, "0")}`;
    }

    // ----- Fairness
    /** Geometrie des Fairness-Fensters (Kopfzeilen, Liste, Details, Knöpfe). */
    _fairGeom() {
      const body = this._panelBody();
      const k = this.k;
      const lh = gh(this.fTiny);
      const noteH = lh + 2;
      const bh = Math.max(22, trunc(28 * k));
      const y = body.y + 2 * lh + 4; // unter der Erklärung
      const ncW = Math.min(fdiv(body.w, 2), this.fBtn.width(t("chip.fair_new_client")) + 20);
      const rNew = new PG.Rect(body.right - ncW, y + lh - fdiv(bh, 2), ncW, bh);
      const hashY = y + 2 * lh + 2; // Prüfcode der nächsten Runde (volle Breite)
      const ly = hashY + lh + Math.max(6, trunc(8 * k));
      const bottom = body.bottom - noteH;
      const lw = Math.max(trunc(body.w * 0.34), 118);
      const step = this.pillH + 4;
      const vis = Math.max(1, fdiv(bottom - ly, step));
      const detail = new PG.Rect(body.x + lw + this.margin, ly, body.w - lw - this.margin, bottom - ly);
      const pv = Math.max(24, Math.min(trunc(detail.w * 0.36), trunc((detail.h - bh - 8) * 0.62)));
      const preview = new PG.Rect(detail.right - pv - 4, detail.y + 4, pv, trunc(pv * 1.05));
      const verW = Math.min(detail.w - 8, Math.max(this.fBtn.width(t("chip.fair_verify")) + 28, trunc(detail.w * 0.4)));
      const rVer = new PG.Rect(detail.x + 4, detail.bottom - bh - 4, verW, bh);
      return { body, lh, y, rNew, hashY, ly, lw, step, vis, detail, preview, rVer, bottom };
    }
    _fairVisible() {
      return this._fairGeom().vis;
    }
    _fairRects() {
      const g = this._fairGeom();
      const rows = [];
      const hits = [[g.rNew, () => this._newClient()]];
      const es = this._fairEntries();
      for (let i = this.fairScroll; i < Math.min(es.length, this.fairScroll + g.vis); i++) {
        const r = new PG.Rect(g.body.x, g.ly + (i - this.fairScroll) * g.step, g.lw, this.pillH);
        rows.push([r, i]);
        hits.push([r, () => {
          this.fairSel = i;
          this.playSound("move");
        }]);
      }
      if (es.length) hits.push([g.rVer, () => this._verify(this._fairEntry())]);
      return [g, rows, hits];
    }

    _drawFair(ctx) {
      this._panelFrame(ctx, "fair", t("chip.fair_title"));
      const [g, rows] = this._fairRects();
      const { body, lh, detail } = g;
      const k = this.k;
      // Erklärung (höchstens zwei Zeilen)
      let y = body.y;
      for (const line of this._wrap(t("chip.fair_info"), this.fTiny, body.w, 2)) {
        ui.text(ctx, line, body.x, y, this.fTiny, ui.TEXT_DIM);
        y += lh;
      }
      // Nächster Prüfcode, Client-Seed, Knopf "Neuer Client-Seed"
      y = g.y;
      const rNew = g.rNew;
      const leftW = rNew.x - body.x - 8;
      const nx = t("chip.fair_next") + ":";
      ui.text(ctx, nx, body.x, y, D.fitFont(nx, leftW, lh, false), ui.TEXT);
      const ciTxt = `${t("chip.fair_client")}: ${this.fair.client}   ·   ${t("chip.fair_nonce")} ${this.fair.nonce}`;
      ui.text(ctx, ciTxt, body.x, y + lh, D.fitFont(ciTxt, leftW, lh, false), ui.TEXT_DIM);
      this._btn(ctx, rNew, t("chip.fair_new_client"), { on: !this.balls.length && this.auto === null, hot: rNew.collidepoint(this.mouse) });
      const [hf, htxt] = this._monoFit(this._nextCommit(), body.w);
      ui.text(ctx, htxt, body.x, g.hashY, hf, CD.COL_GOLD);
      const lineY = g.ly - Math.max(3, trunc(4 * k));
      draw.line(ctx, ui.BORDER, [body.x, lineY], [body.right, lineY], 1);
      const note = t("chip.fair_note");
      ui.text(ctx, note, icx(body), body.bottom, D.fitFont(note, body.w, lh, false), ui.TEXT_FAINT, "midbottom");
      const es = this._fairEntries();
      if (!es.length) {
        ui.text(ctx, t("chip.fair_empty"), icx(body), fdiv(g.ly + g.bottom, 2), this.fSmall, ui.TEXT_DIM, "center");
        return;
      }
      const sel = this._fairEntry();
      // Liste der letzten Kugeln (neueste oben)
      for (const [r, i] of rows) {
        const e = es[i];
        const o = isDict(e.o) ? e.o : {};
        const on = i === this.fairSel;
        const hot = r.collidepoint(this.mouse);
        draw.rect(ctx, on || hot ? ui.BTN_SEL : ui.mix(ui.PANEL, ui.BG_TOP, 0.35), r, 0, fdiv(r.h, 2));
        if (on) draw.rect(ctx, CD.COL_GOLD, r, 1, fdiv(r.h, 2));
        ui.text(ctx, `#${e.n}`, r.x + 8, icy(r), this.fTiny, on ? ui.TEXT : ui.TEXT_DIM, "midleft");
        const m = Number.isInteger(o.m) ? o.m : 0;
        const pw = Math.max(30, fdiv(r.w * 2, 5));
        const ph = Math.max(10, r.h - 4);
        const pill = D.pill(pw, ph, m, this._fmt(m), Math.max(8, trunc(r.h * 0.55)));
        ctx.drawImage(pill, r.right - 3 - pw, icy(r) + 1 - fdiv(ph, 2), pill.lw, pill.lh);
        const chk = this.fairChecks.get(e.n);
        if (chk !== undefined) {
          const good = chk[0] && chk[1];
          const isz = Math.max(8, r.h - 6);
          const ic = D.icon(good ? "check" : "cross", isz, good ? ui.GREEN : ui.RED);
          ctx.drawImage(ic, r.right - pw - 6 - isz, icy(r) - fdiv(isz, 2), ic.lw, ic.lh);
        }
      }
      if (es.length > rows.length && rows.length) {
        const more = `${this.fairScroll + 1}-${this.fairScroll + rows.length} / ${es.length}`;
        const last = rows[rows.length - 1][0];
        if (last.bottom + rh(this.fTiny) <= g.bottom) ui.text(ctx, more, icx(last), last.bottom + 1, this.fTiny, ui.TEXT_FAINT, "midtop");
      }
      // Details des gewählten Eintrags
      draw.rect(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), detail, 0, Math.max(4, trunc(6 * k)));
      const pv = g.preview;
      const o = isDict(sel.o) ? sel.o : {};
      const xx = detail.x + 6;
      const tw = pv.x - xx - 6;
      y = detail.y + 4;
      ui.text(ctx, `${t("chip.fair_nonce")} ${sel.n}`, xx, y, this.fBtn, ui.TEXT);
      y += rh(this.fBtn) + 1;
      if (L.ROWS.includes(o.rows) && L.RISKS.includes(o.risk)) {
        const res = t("lpl.fair_result", { rows: o.rows, risk: this._riskName(o.risk), slot: o.slot,
          m: L.fmtShort(Number.isInteger(o.m) ? o.m : 0, this._sep()) });
        const rf = D.fitFont(res, tw, gh(this.fTiny) + 1);
        ui.text(ctx, res, xx, y, rf, CD.COL_GOLD);
        y += rh(rf) + 3;
      }
      for (const [lbl, vals] of [[t("chip.fair_hash"), [sel.h.slice(0, 32), sel.h.slice(32)]],
        [t("chip.fair_server"), [sel.s]], [t("chip.fair_client"), [sel.c]]]) {
        ui.text(ctx, lbl + ":", xx, y, D.fitFont(lbl + ":", tw, lh, false), ui.TEXT_DIM);
        y += lh;
        for (const v of vals) {
          const [vf, vt] = this._monoFit(v, tw, 32);
          ui.text(ctx, vt, xx, y, vf, ui.TEXT);
          y += rh(vf);
        }
        y += 2;
      }
      // Vorschau-Rahmen rechts oben (nach dem Nachrechnen mit dem Weg der Kugel)
      draw.rect(ctx, ui.mix(ui.PANEL, ui.BG_TOP, 0.6), pv, 0, Math.max(3, trunc(5 * k)));
      const chk = this.fairChecks.get(sel.n);
      const rVer = g.rVer;
      this._btn(ctx, rVer, t("chip.fair_verify"), { primary: chk === undefined, hot: rVer.collidepoint(this.mouse) });
      if (chk === undefined) {
        const isz = Math.max(12, fdiv(pv.w, 3));
        const ic = D.icon("fair", isz, ui.TEXT_FAINT);
        ctx.drawImage(ic, icx(pv) - fdiv(isz, 2), icy(pv) - fdiv(isz, 2), ic.lw, ic.lh);
        return;
      }
      const [okHash, okRes, path] = chk;
      const good = okHash && okRes;
      const col = good ? ui.GREEN : ui.RED;
      if (path.length) {
        const img = D.pathPreview(o.rows, path, pv.w - 4, pv.h - 4, Number.isInteger(o.m) ? o.m : null);
        ctx.drawImage(img, pv.x + 2, pv.y + 2, img.lw, img.lh);
      }
      // Ergebnis der Prüfung unter der Vorschau
      const y2 = pv.bottom + 4;
      const isz = lh + 4;
      const ic = D.icon(good ? "check" : "cross", isz, col);
      const word = good ? t("chip.fair_ok") : t("chip.fair_bad");
      ctx.drawImage(ic, pv.x, y2, ic.lw, ic.lh);
      ui.text(ctx, word, pv.x + isz + 4, y2 + fdiv(isz, 2), D.fitFont(word, pv.w - isz - 4, gh(this.fBtn)), col, "midleft");
      if (path.length) {
        const calc = t("lpl.fair_recalc", { slot: L.slotOf(path) });
        const cf = D.fitFont(calc, pv.w + 4, lh, false);
        if (y2 + isz + rh(cf) <= rVer.y + rVer.h) ui.text(ctx, calc, pv.x - 2, y2 + isz + 1, cf, ui.TEXT_DIM);
      }
    }

    /** Hex-Werte in Monospace: größte Schrift, in der ref Zeichen (Standard: der
     *  Text selbst) in maxW passen; notfalls gekürzt mit '…'. -> [Schrift, Text] */
    _monoFit(text, maxW, ref = null) {
      maxW = Math.max(20, maxW);
      const probe = ref ? "0".repeat(ref) : text;
      let f = ui.font(9, false, true);
      for (let sz = Math.max(10, trunc(14 * this.k)); sz > 8; sz--) {
        f = ui.font(sz, false, true);
        if (f.width(probe) <= maxW) break;
      }
      let cut = text;
      while (cut.length > 4 && f.width(cut) > maxW) cut = cut.slice(0, -2) + "…";
      return [f, cut];
    }

    _wrap(text, fnt, w, maxLines) {
      const words = text.split(/\s+/).filter(Boolean);
      let lines = [], cur = "";
      for (const wd of words) {
        const test = (cur + " " + wd).trim();
        if (fnt.width(test) <= w || !cur) cur = test;
        else {
          lines.push(cur);
          cur = wd;
        }
      }
      if (cur) lines.push(cur);
      if (lines.length > maxLines) {
        lines = lines.slice(0, maxLines);
        let last = lines[lines.length - 1];
        while (last.length > 3 && fnt.width(last + "…") > w) last = last.slice(0, -1);
        lines[lines.length - 1] = last + "…";
      }
      return lines;
    }
  }

  PG.register(LamaPlinkoGame, {
    id: "LamaPlinkoGame",
    key: KEY,
    name: "Lama Plinko",
    settingsKey: KEY,
    defaults: DEFAULTS,
  });
})();
