/*
 * lamamines.js - Lama Mines (Port von games/lamamines.py)
 * ========================================================
 * Edelstein-Suche mit Lama-Chips: Auf einem Feld aus 4x4 bis 7x7 Steinquadern
 * liegen 1 bis N-1 Minen, unter allen anderen ein Edelstein. Jeder gefundene
 * Edelstein hebt den Multiplikator, eine Mine kostet den Einsatz. Auszahlen geht
 * ab dem ersten Edelstein; wer alle Edelsteine findet oder den Höchstgewinn
 * x100.000 erreicht, wird automatisch ausgezahlt.
 *
 * Regeln und Zahlen stehen in lamamines_logic.js (bitgleich mit Python und
 * Rust), die Bilder in lamamines_draw.js.
 *
 * - Fairness (Fenster F): Vor jeder Runde steht der Prüfcode (SHA-256 des
 *   Server-Seeds) fest, das Minenfeld entsteht aus Server-Seed, Client-Seed und
 *   Rundennummer (fairness.js). Nach der Runde wird der Seed gezeigt; die
 *   letzten 12 Runden lassen sich nachrechnen (mit Mini-Vorschau des Feldes).
 * - Statistik (Fenster S): Kacheln + Bilanzkurve (chipstats.js).
 * - Auto-Modus (Fenster A): Felder auf dem Raster markieren (gemerkt je
 *   Feldgröße); jede Runde deckt genau diese auf und zahlt aus, wenn alle sicher
 *   sind. Runden, Stopp bei Gewinn/Verlust ab, Turbo (T).
 * - Verlauf der letzten 12 Runden oben rechts, Maskottchen-Lama mit Grubenhelm
 *   und Lore, Explosion mit Kamera-Wackeln, danach wird das ganze Feld
 *   abgedunkelt aufgedeckt (auch nach dem Auszahlen).
 *
 * Lama-Chips: gemeinsames Konto der Lama-Bank (casino_bank.js). Der Einsatz
 * wird bei SPIELEN abgebucht; die Runde wird bei jedem aufgedeckten Feld
 * gespeichert (Seeds, Rundennummer, Feld, Minen, Einsatz, aufgedeckte Plätze) -
 * wer mitten in der Runde geht, macht beim nächsten Start genau dort weiter.
 * Speicherformat (lamabank-Extra "lamamines", Einstellungen "lamamines") wie in
 * Python - Spielstände sind zwischen Desktop und Web austauschbar.
 * Highscore = Höchststand von 1000 + Lama-Mines-Bilanz; gameOver wird nie gesetzt.
 *
 * Steuerung (alle Tasten roh, nie über die WASD-Belegung): Leertaste/Enter =
 * SPIELEN, 1-4 = Feldgröße, Pfeile links/rechts = Minen, hoch/runter/+/-/Mausrad
 * = Einsatz; in der Runde Klick oder Pfeile + Leertaste = aufdecken,
 * R = Zufallsfeld, Enter/C = auszahlen. A = Auto, S = Statistik, F = Fairness,
 * T = Turbo, Esc schließt ein Fenster bzw. stoppt Auto.
 */
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  const ML = PG.lamaMinesLogic;
  const D = PG.lamaMinesDraw;
  const CD = PG.casinoDraw;
  const F = PG.fairness;
  const CS = PG.chipstats;
  const bank = PG.lamabank;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

  const KEY = ML.KEY;
  const IDLE = "idle", PLAY = "play", BOOM = "boom", CASHED = "cashed";

  const FLIP_T = 0.24; // Aufklappen eines Feldes
  const POP_T = 0.32; // Edelstein federt nach dem Aufklappen
  const WAVE_T = 0.05; // Rest-Aufdeckung: Verzögerung je Feld Abstand
  const BOOM_T = 0.62; // Explosion -> Ergebnis-Tafel
  const RESULT_T = 2.6; // Ergebnis-Tafel, danach Ruhezustand (Feld bleibt aufgedeckt)
  const READ_T = 0.35; // Ergebnis frühestens danach wegdrückbar
  const SHAKE_T = 0.55; // Kamera-Wackeln nach einer Mine
  const AUTO_STEP_T = 0.26; // Auto: Abstand zwischen zwei Feldern
  const AUTO_RESULT_T = 1.0; // Auto: Ergebnis-Anzeige
  const AUTO_PAUSE_T = 0.35; // Auto: Pause vor der nächsten Runde
  const TURBO_F = 0.4; // Turbo: alle Zeiten x 0,4
  const BIG_MULT = 300; // ab x3: Münzregen
  const PARTY_MULT = 1000; // ab x10: Konfetti

  const ROUND_OPTS = [5, 10, 25, 50, 100];
  const LIMIT_OPTS = [0, 100, 500, 1000, 5000, 10000, 50000];
  const TOOLS = ["auto", "stats", "fair", "turbo"];
  const TOOL_LABEL = { auto: "chip.auto", stats: "chip.stats", fair: "chip.fair", turbo: "chip.turbo" };
  const TOOL_KEY = { auto: "A", stats: "S", fair: "F", turbo: "T" };
  // Auto-Einstellungen: Schlüssel in den Einstellungen -> Feld im Spiel
  const AUTO_FIELDS = [[ROUND_OPTS, "auto_rounds", "autoRounds"], [LIMIT_OPTS, "auto_win", "autoWin"], [LIMIT_OPTS, "auto_loss", "autoLoss"]];
  // Töne beim Finden: Dur-Pentatonik aufwärts (Halbtöne über dem Grundton)
  const GEM_STEPS = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28, 31, 33, 36];

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);
  /** Ganzzahliges Rechteck (wie pygame.Rect). */
  const R = (x, y, w, h) => new PG.Rect(trunc(x), trunc(y), trunc(w), trunc(h));
  const cxOf = (r) => r.x + fdiv(r.w, 2);
  const cyOf = (r) => r.y + fdiv(r.h, 2);
  /** pygame Rect.inflate (C-Ganzzahldivision). */
  const inflate = (r, dx, dy) => new PG.Rect(r.x - trunc(dx / 2), r.y - trunc(dy / 2), r.w + dx, r.h + dy);
  /** Leertaste/Enter - alle Tasten werden ROH ausgewertet (nie über isAction/WASD). */
  const isAction = (key) => key === "space" || key === "Return" || key === "KP_Enter";

  class LamaMinesGame extends PG.Game {
    /** Jede Runde ist ein Match (Game.logMatch). */
    get matchManual() {
      return true;
    }

    /** Esc schließt ein offenes Fenster bzw. stoppt den Auto-Modus. */
    get wantsEscape() {
      return (this.window != null) || (this.auto != null);
    }

    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      bank.load();
      this.rng = PG.rand;
      const grid = this._opt("grid", ML.DEFAULT_GRID);
      this.grid = ML.GRIDS.includes(grid) ? grid : ML.DEFAULT_GRID;
      this.mines = ML.clampMines(this.grid, this._intOpt("mines", ML.DEFAULT_MINES));
      const bet = this._opt("bet", ML.BETS[0]);
      this.bet = ML.BETS.includes(bet) ? bet : ML.BETS[0];
      this.turbo = this._opt("turbo", false) === true;
      let v = this._opt("auto_rounds", 10);
      this.autoRounds = ROUND_OPTS.includes(v) ? v : 10;
      v = this._opt("auto_win", 0);
      this.autoWin = LIMIT_OPTS.includes(v) ? v : 0;
      v = this._opt("auto_loss", 0);
      this.autoLoss = LIMIT_OPTS.includes(v) ? v : 0;
      let extra = bank.getExtra(KEY, {});
      if (!isDict(extra)) extra = {};
      this.hist = ML.cleanHistory(extra.hist);
      this.fair = F.cleanState(extra.fair);
      this.fairlog = F.cleanLog(extra.fairlog);
      this.stats = CS.clean(extra.stats);
      this.marks = ML.cleanMarks(extra.auto_marks);
      this.carry = ML.cleanCarry(extra.carry); // Rest-Hundertstel der Auszahlungen
      this.round = ML.cleanRound(extra.round);
      // Spielfeld (laufende bzw. letzte Runde)
      this.field = []; // Minenplätze (sortiert)
      this.isMine = []; // je Platz true/false
      this.cells = []; // je Platz: Anzeige-Zustand (siehe _blankCells)
      this.hit = null; // Platz der getroffenen Mine
      this.state = IDLE;
      this.result = null; // [Titel, Untertitel, Farbe]
      this.resT = 0;
      this.resDelay = 0; // Ergebnis-Tafel erscheint erst danach
      this.boomT = null; // Zeit seit der Explosion (null = keine)
      this.win = 0;
      this.lost = 0;
      this.lastMult = 0;
      this.pending = 0; // gutgeschrieben, aber noch nicht gezeigt
      this.shownChips = bank.balance();
      this.particles = [];
      this.coinsFx = [];
      this.popups = [];
      this.flashes = [];
      this.toast = null;
      this.banner = null; // [Text, Restzeit] (Höchstgewinn)
      this.shake = 0;
      this.time = 0;
      this.mouse = [-1, -1];
      this.broke = false;
      this.window = null; // null / "auto" / "stats" / "fair"
      this.auto = null; // laufender Auto-Modus
      this.cursor = 0;
      this.kb = false; // Tastatur-Cursor sichtbar?
      this.mood = ["idle", 0]; // Maskottchen: idle/happy/cheer/scared + Zeit
      this.sooty = false;
      this.cartFill = 0;
      this.fairSel = null; // gewählter Log-Eintrag (Index)
      this.fairScroll = 0;
      this.fairChecks = new Map(); // Nonce -> [stimmt?, Minenplätze]
      this.rFairRows = [];
      this.rFairNew = R(0, 0, 0, 0);
      this.rFairVerify = R(0, 0, 0, 0);
      this.cashInfo = { clear: false, maxed: false, m100: 0, shown: true };
      this._commitMemo = ["", ""];
      this._tickCd = 0;
      this._lang = PG.lang;
      this._layout();
      if (this.round) {
        // Unterbrochene Runde genau dort fortsetzen (Feld aus dem Seed)
        this.grid = this.round.grid;
        this.mines = this.round.mines;
        this._newBoard(this.round);
        this.round.open.forEach((i, j) => {
          const c = this.cells[i];
          c.face = "gem";
          c.flip = 1;
          c.pop = POP_T;
          c.k = j + 1;
        });
        this.cartFill = this._cartLevel();
        this.state = PLAY;
        this._layout();
        this._say(t("lmi.resumed"));
      } else {
        this._blankBoard();
        this._clampBet();
      }
      this.cursor = fdiv(this.grid * this.grid, 2);
      this.score = bank.scoreFor(KEY);
      this._checkBroke();
    }

    // ===================================================== Layout
    _layout() {
      const W = this.width, H = this.height;
      const k = (this.k = Math.min(W / 800, H / 600));
      this.fMicro = ui.font(Math.max(10, trunc(11 * k)));
      this.fTiny = ui.font(Math.max(11, trunc(13 * k)));
      this.fSmall = ui.font(Math.max(12, trunc(15 * k)));
      this.fBtn = ui.font(Math.max(12, trunc(15 * k)), true);
      this.fBig = ui.font(Math.max(15, trunc(22 * k)), true);
      this.fMult = ui.font(Math.max(18, trunc(32 * k)), true);
      this.fHuge = ui.font(Math.max(24, trunc(48 * k)), true);
      this.fMono = ui.font(Math.max(10, trunc(12 * k)), false, true);
      const m = (this.margin = Math.max(6, trunc(12 * k)));
      this.hudH = Math.max(28, trunc(42 * k));
      this.stripH = Math.max(46, trunc(64 * k));
      this.toolH = Math.max(26, trunc(34 * k));
      this.strip = R(0, H - this.stripH, W, this.stripH);
      const toolY = this.hudH + m;
      const areaY = toolY + this.toolH + m;
      const areaH = this.strip.y - m - areaY;
      const side = Math.min(areaH, trunc(W * 0.58));
      this.boardR = R(m, areaY + fdiv(areaH - side, 2), side, side);
      this.panelR = R(this.boardR.right + m, areaY, W - this.boardR.right - 2 * m, areaH);
      this._layoutBoard();
      this._layoutTools(toolY);
      this._layoutStrip();
      this._layoutPanel();
    }

    /** Felder im Holzrahmen: Rand, Abstand, Feldgröße und Rechtecke. */
    _layoutBoard() {
      const side = this.boardR.w;
      const n = this.grid;
      const pad = (this.boardPad = Math.max(6, trunc(side * 0.035)));
      const inner = side - 2 * pad;
      const gap = Math.max(2, trunc(inner * 0.014));
      const cell = fdiv(inner - gap * (n + 1), n);
      const total = n * cell + (n + 1) * gap;
      const off = fdiv(inner - total, 2);
      const x0 = this.boardR.x + pad + off + gap;
      const y0 = this.boardR.y + pad + off + gap;
      this.cell = cell;
      this.tileR = [];
      for (let i = 0; i < n * n; i++) this.tileR.push(R(x0 + (i % n) * (cell + gap), y0 + fdiv(i, n) * (cell + gap), cell, cell));
    }

    /** Werkzeug-Knöpfe oben links (mit Beschriftung, wenn Platz ist). */
    _layoutTools(y) {
      const m = this.margin, h = this.toolH;
      const pad = Math.max(6, trunc(10 * this.k));
      const sp = Math.max(3, trunc(5 * this.k));
      const widths = TOOLS.map((name) => h + pad + this.fTiny.width(t(TOOL_LABEL[name])));
      let total = 0;
      for (const w of widths) total += w + sp;
      this.toolLabels = total <= this.boardR.w;
      let x = m;
      this.rTools = [];
      TOOLS.forEach((name, i) => {
        const w = this.toolLabels ? widths[i] : h;
        this.rTools.push([name, R(x, y, w, h)]);
        x += w + sp;
      });
      this.toolsRight = x;
    }

    /** Bedienleiste: Einsatz links, Zufallsfeld Mitte, AUSZAHLEN + SPIELEN rechts. */
    _layoutStrip() {
      const m = this.margin, k = this.k, W = this.width;
      const sh = this.stripH;
      const bh = Math.max(30, trunc(sh * 0.62));
      const by = this.strip.y + fdiv(sh - bh, 2);
      const lblW = Math.max(this.fTiny.width(t("cas.bet")), this.fBtn.width("10000")) + trunc(14 * k);
      this.rMinus = R(m, by, bh, bh);
      this.rBet = R(this.rMinus.right + 4, by, lblW, bh);
      this.rPlus = R(this.rBet.right + 4, by, bh, bh);
      const pad = trunc(22 * Math.max(0.7, k));
      let goW = Math.max(this.fBtn.width(t("lmi.play")), this.fBtn.width(t("chip.auto_stop"))) + pad;
      goW = Math.max(goW, trunc(110 * k));
      let cashW = this.fBtn.width(t("lmi.cashout") + " 99.999.999") + pad;
      const room = W - m - (this.rPlus.right + m) - Math.max(4, trunc(8 * k)) - bh - 2 * m;
      if (goW + cashW > room) {
        const f = room / (goW + cashW);
        goW = trunc(goW * f);
        cashW = trunc(cashW * f);
      }
      this.rGo = R(W - m - goW, by, goW, bh);
      this.rCash = R(this.rGo.x - Math.max(4, trunc(8 * k)) - cashW, by, cashW, bh);
      // Zufallsfeld: so breit wie möglich zwischen Einsatz und AUSZAHLEN
      const left = this.rPlus.right + m, right = this.rCash.x - m;
      const want = bh + this.fBtn.width(t("lmi.random")) + pad;
      const w = Math.max(bh, Math.min(want, right - left));
      this.randomLabel = w >= want;
      this.rRandom = R(fdiv(left + right, 2) - fdiv(w, 2), by, w, bh);
    }

    /** Seitenleiste: Kopf (Maskottchen, Lore, Multiplikator), Einstellungen bzw.
     *  Gewinn-Info, Leiste der nächsten Stufen, Tastenhinweis. */
    _layoutPanel() {
      const p = this.panelR, k = this.k;
      const pad = (this.ppad = Math.max(6, trunc(10 * k)));
      const x0 = p.x + pad, w = p.w - 2 * pad;
      const gap = Math.max(4, trunc(7 * k));
      const ms = trunc(Math.min(w * 0.3, 104 * k, p.h * 0.3));
      this.rMascot = R(x0, p.y + pad, ms, ms);
      this.cartW = trunc(ms * 0.62);
      this.headX = x0 + ms + trunc(this.cartW * 0.95) + gap;
      let y = this.rMascot.bottom + gap;
      const rh = (this.rowH = Math.max(22, trunc(30 * k)));
      const lab = this.fTiny.px;
      // Einstellungen (nur im Ruhezustand)
      this.labGridY = y;
      y += lab + 2;
      const bw = fdiv(w - 3 * 4, 4);
      this.rGrids = [0, 1, 2, 3].map((i) => R(x0 + i * (bw + 4), y, bw, rh));
      y += rh + gap;
      this.minesRowY = y;
      const vw = Math.max(this.fBtn.width("48") + trunc(16 * k), rh + 6);
      this.rMval = R(x0 + w - rh - 4 - vw, y, vw, rh);
      this.rMminus = R(this.rMval.x - 4 - rh, y, rh, rh);
      this.rMplus = R(this.rMval.right + 4, y, rh, rh);
      y += rh + 4;
      const npre = ML.MINE_PRESETS[4].length;
      const pw = fdiv(w - (npre - 1) * 4, npre);
      this.rPresets = [];
      for (let i = 0; i < npre; i++) this.rPresets.push(R(x0 + i * (pw + 4), y, pw, Math.max(18, trunc(rh * 0.82))));
      this.infoY = this.labGridY;
      // Leiste der nächsten Stufen + Hinweis am unteren Rand
      const hintH = 2 * this.fMicro.px + 2;
      const pillH = Math.max(20, trunc(28 * k));
      this.ladderY = p.bottom - pad - hintH - gap - pillH;
      this.ladderLabY = this.ladderY - lab - 2;
      const pillW = fdiv(w - 16, 5);
      this.rPills = [0, 1, 2, 3, 4].map((i) => R(x0 + i * (pillW + 4), this.ladderY, pillW, pillH));
      this.hintY = p.bottom - pad - hintH;
      this.panelInner = [x0, w];
      // Auto-Fenster liegt genau über der Seitenleiste
      this._layoutAuto();
    }

    _layoutAuto() {
      const p = this.panelR, k = this.k, pad = this.ppad;
      const x0 = p.x + pad, w = p.w - 2 * pad;
      const gap = Math.max(3, trunc(6 * k));
      const lab = this.fTiny.px;
      const rh = Math.max(20, trunc(28 * k));
      const cb = Math.max(18, trunc(24 * k));
      this.rAutoClose = R(p.right - pad - cb, p.y + pad, cb, cb);
      let y = p.y + pad + Math.max(cb, this.fBtn.px) + gap;
      const rows = [];
      for (const opts of [ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS]) {
        const ly = y;
        y += lab + 2;
        const n = opts.length;
        let bw = fdiv(w - (n - 1) * 3, n);
        let first = bw;
        if (opts[0] === 0) {
          // "aus" ist je Sprache verschieden lang - der erste Knopf wächst mit
          first = Math.max(bw, Math.min(this.fTiny.width(t("chip.auto_off")), this.fMicro.width(t("chip.auto_off")) + 4) + 8);
          bw = fdiv(w - first - (n - 1) * 3, n - 1);
        }
        const rects = [];
        let x = x0;
        for (let i = 0; i < n; i++) {
          const rw = i === 0 ? first : bw;
          rects.push(R(x, y, rw, rh));
          x += rw + 3;
        }
        rows.push([ly, rects]);
        y += rh + gap;
      }
      this.autoRows = rows;
      this.autoInfoY = y;
      const bh = Math.max(24, trunc(32 * k));
      const cw = Math.max(this.fBtn.width(t("lmi.clear_marks")) + trunc(18 * k), bh);
      this.rAutoClear = R(x0, p.bottom - pad - bh, cw, bh);
      const sx = this.rAutoClear.right + gap;
      this.rAutoStart = R(sx, p.bottom - pad - bh, x0 + w - sx, bh);
    }

    // ===================================================== Einstellungen
    _opt(key, def) {
      const v = this.opts[key];
      return v === undefined ? def : v;
    }
    _intOpt(key, def) {
      const v = this._opt(key, def);
      return isInt(v) ? v : def;
    }
    _setOpt(key, value) {
      if (this.opts[key] !== value) {
        this.opts[key] = value;
        this.saveSettings();
      }
    }

    // ===================================================== Konto / Speichern
    _extra() {
      return { round: this.round, hist: this.hist, fair: this.fair, fairlog: this.fairlog,
        stats: this.stats, auto_marks: this.marks, carry: this.carry };
    }
    /** Auszahlung nach k Edelsteinen - mit dem gespeicherten Übertrag, also genau
     *  das, was AUSZAHLEN jetzt gutschreiben würde. */
    _winAt(bet, k) {
      return ML.payoutCarry(bet, this.grid, this.mines, k, this.carry)[0];
    }
    _saveExtra() {
      bank.setExtra(KEY, this._extra());
    }
    /** Einsatz auf die höchste bezahlbare Stufe senken (mind. Stufe 1). */
    _clampBet() {
      const chips = bank.balance();
      if (this.bet > chips) {
        const ok = ML.BETS.filter((b) => b <= chips);
        this.bet = ok.length ? ok[ok.length - 1] : ML.BETS[0];
      }
    }
    _checkBroke() {
      this.broke = this.state === IDLE && this.auto === null && bank.isBroke(KEY);
      if (this.broke) {
        this.window = null;
        this.playSound("gameover");
      }
    }
    _takeCredit() {
      if (bank.refillIfBroke(KEY)) {
        this.playSound("powerup");
        ui.spawnBurst(fdiv(this.width, 2), fdiv(this.height, 2), ui.GOLD, 26);
      }
      this.broke = false;
      this.shownChips = bank.balance();
    }
    _roundBet() {
      return this.round ? this.round.bet : this.bet;
    }
    _say(text, secs = 1.8) {
      this.toast = [text, secs];
    }
    /** Zeitfaktor: Turbo macht alles schneller. */
    _tf() {
      return this.turbo ? TURBO_F : 1.0;
    }

    // ===================================================== Formatierung
    _sep() {
      return t("lmi.decimal");
    }
    /** Ganze Zahl mit Tausenderpunkten (bzw. -kommas, je Sprache). */
    _group(n) {
      const sep = this._sep() === "," ? "." : ",";
      let s = String(Math.abs(trunc(n)));
      let out = "";
      while (s.length > 3) {
        out = sep + s.slice(-3) + out;
        s = s.slice(0, -3);
      }
      return (n < 0 ? "-" : "") + s + out;
    }
    /** Multiplikator x100 als Text mit Tausendertrennung: ×1.234,56. */
    _fmt(m100) {
      if (m100 >= 1000000) return "×" + this._group(fdiv(m100, 100));
      return "×" + this._group(fdiv(m100, 100)) + this._sep() + String(m100 % 100).padStart(2, "0");
    }
    /** Kurzform für die Stufen-Pillen: ×1,23 · ×123,4 · ×1234 · ×12,3k · ×100k. */
    _fmtShort(m100) {
      const sep = this._sep();
      if (m100 < 10000) return "×" + fdiv(m100, 100) + sep + String(m100 % 100).padStart(2, "0");
      if (m100 < 100000) return "×" + fdiv(m100, 100) + sep + (fdiv(m100, 10) % 10);
      if (m100 < 1000000) return "×" + fdiv(m100, 100);
      if (m100 < 10000000) return "×" + fdiv(m100, 100000) + sep + (fdiv(m100, 10000) % 10) + "k";
      return "×" + fdiv(m100, 100000) + "k";
    }
    _fmtPct(p10) {
      return ML.fmtPercent(p10, this._sep());
    }

    // ===================================================== Spielfeld
    _blankCells() {
      // face: "hidden", "gem", "mine", "boom"; flip: 0..1 (1 = fertig);
      // delay: Wartezeit vor dem Aufklappen; dim: nach der Runde aufgedeckt
      // k: Edelstein Nummer k dieser Runde (für Ton und Popup beim Aufklappen)
      const n = this.grid * this.grid;
      this.cells = [];
      for (let i = 0; i < n; i++) this.cells.push({ face: "hidden", flip: 1, delay: 0, dim: false, pop: POP_T, mark: 0, k: 0 });
    }
    /** Leeres, verdecktes Feld der aktuellen Größe (Ruhezustand). */
    _blankBoard() {
      this.field = [];
      this.isMine = new Array(this.grid * this.grid).fill(false);
      this.hit = null;
      this._blankCells();
    }
    /** Minenfeld einer Runde aus ihren Fairness-Werten berechnen. */
    _newBoard(rnd) {
      this.field = ML.fieldFromSeed(rnd.grid, rnd.mines, rnd.s, rnd.c, rnd.n);
      this.isMine = new Array(rnd.grid * rnd.grid).fill(false);
      for (const i of this.field) this.isMine[i] = true;
      this.hit = null;
      this._blankCells();
    }
    _opened() {
      return this.round ? this.round.open : [];
    }
    _k() {
      return this._opened().length;
    }
    /** Edelsteine in der Lore (0..7) - wächst mit dem Anteil gefundener Steine. */
    _cartLevel() {
      const g = ML.gems(this.grid, this.mines);
      const k = this._k();
      if (k <= 0) return 0;
      return Math.max(1, Math.min(7, fdiv(k * 7 + g - 1, g)));
    }
    _entry() {
      const r = this.round;
      return { n: r.n, s: r.s, c: r.c, h: r.h };
    }

    // ===================================================== Ablauf
    /** SPIELEN: Einsatz abbuchen, Minenfeld aus dem Fairness-Zufall ziehen. */
    _play() {
      if (this.state !== IDLE || this.broke || this.round) return false;
      const bet = this.bet;
      if (bet > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return false;
      }
      const entry = F.begin(this.fair);
      F.advance(this.fair);
      this.round = { bet, grid: this.grid, mines: this.mines, n: entry.n, s: entry.s, c: entry.c, h: entry.h, open: [] };
      if (!bank.betAndPay(bet, 0, KEY, KEY, this._extra())) {
        this.round = null;
        return false;
      }
      this.field = ML.makeField(F.entryRand(KEY, entry), this.grid, this.mines);
      this.isMine = new Array(this.grid * this.grid).fill(false);
      for (const i of this.field) this.isMine[i] = true;
      this.hit = null;
      this._blankCells();
      this.cells.forEach((c, i) => {
        c.flip = 0; // Felder "wachsen" kurz ein
        c.face = "hidden";
        c.delay = 0.012 * ((i % this.grid) + fdiv(i, this.grid)) * this._tf();
      });
      this.state = PLAY;
      this.result = null;
      this.sooty = false;
      this.cartFill = 0;
      this.mood = ["idle", 0];
      this.win = 0;
      this.lost = 0;
      this.playSound("select");
      this.score = bank.scoreFor(KEY);
      return true;
    }

    /** Ein Feld aufdecken (Klick, Leertaste, R oder Auto). */
    _reveal(i) {
      if (this.state !== PLAY || !this.round || !(i >= 0 && i < this.cells.length)) return;
      if (this.round.open.includes(i) || this.cells[i].face !== "hidden") return;
      const c = this.cells[i];
      c.flip = 0;
      c.delay = 0;
      c.dim = false;
      if (this.isMine[i]) {
        // Mine: sofort verbuchen (Verlassen hilft nicht), Explosion nach dem Aufklappen
        c.face = "boom";
        this.hit = i;
        const bet = this.round.bet;
        const opened = this.round.open.concat([i]);
        this.lost = bet;
        this.lastMult = 0;
        this._finish(0, opened);
        this.state = BOOM;
        this.boomT = null;
        this.resDelay = FLIP_T * this._tf() + BOOM_T * this._tf();
        this.resT = 0;
        this.result = [t("lmi.boom"), t("lmi.lost", { n: this._group(bet) }), ui.RED];
        this.stats.x.booms = this._xint("booms") + 1;
        this._saveExtra();
        this.playSound("move");
        return;
      }
      c.face = "gem";
      this.round.open.push(i);
      const k = this.round.open.length;
      c.k = k;
      this.stats.x.gems = this._xint("gems") + 1;
      this.playSound("move");
      if (ML.isClear(this.grid, this.mines, k) || ML.isMax(this.grid, this.mines, k)) this._cashOut(true);
      else this._saveExtra();
    }

    _xint(name) {
      const v = this.stats.x[name];
      return isInt(v) && v >= 0 ? v : 0;
    }

    /** Rundenende verbuchen: Statistik, Verlauf, Fairness-Log, Runde löschen. */
    _finish(paid, opened) {
      const r = this.round;
      const bet = r.bet;
      const m100 = this.lastMult;
      CS.record(this.stats, bet, paid, m100);
      const net = paid - bet;
      this.logMatch(net, net > 0 ? 1 : net < 0 ? 0 : 2,
        { bet, win: paid, x: m100, mi: r.mines, op: opened.length });
      this.hist = this.hist.concat([m100]).slice(-ML.HISTORY);
      const outcome = { grid: r.grid, mines: r.mines, open: opened.slice(), m: m100, bet };
      F.record(this.fairlog, this._entry(), outcome);
      this.fairSel = null;
      if (this.auto) this.auto.net += paid - bet;
      this.round = null;
    }

    /** Auszahlen (Enter/C/Knopf) - automatisch bei vollem Feld/Höchstgewinn. */
    _cashOut(auto = false) {
      if (this.state !== PLAY || !this.round) return;
      const k = this._k();
      if (k < 1) {
        if (!auto) this._say(t("lmi.need_gem"), 1.2);
        return;
      }
      const bet = this.round.bet;
      const m100 = ML.mult100(this.grid, this.mines, k);
      [this.win, this.carry] = ML.payoutCarry(bet, this.grid, this.mines, k, this.carry);
      this.lastMult = m100;
      const opened = this.round.open.slice();
      const clear = ML.isClear(this.grid, this.mines, k);
      const maxed = ML.isMax(this.grid, this.mines, k);
      bank.credit(this.win, KEY, false);
      this.pending += this.win;
      if (clear) this.stats.x.clears = this._xint("clears") + 1;
      this._finish(this.win, opened);
      this._saveExtra();
      this.score = bank.scoreFor(KEY);
      this.state = CASHED;
      this.resT = 0;
      this.resDelay = auto ? FLIP_T * this._tf() : 0;
      let sub = this._fmt(m100) + "   ·   " + t("cas.bet") + ": " + this._group(bet);
      const title = !maxed && clear ? t("lmi.cleared") : t("lmi.cashed", { n: this._group(this.win) });
      if (clear && !maxed) sub = t("lmi.cashed", { n: this._group(this.win) }) + "   ·   " + this._fmt(m100);
      this.result = [title, sub, ui.GOLD];
      this.cashInfo = { clear, maxed, m100, shown: false };
      // Erfolge sofort (nichts geht beim Verlassen verloren)
      if (clear) this.achEvent("lmi_clear");
      if (m100 >= ML.BRAVE_MULT) this.achEvent("lmi_brave");
      if (m100 >= ML.LEGEND_MULT) this.achEvent("lmi_legend");
      if (!auto) this._celebrate();
    }

    /** Gewinn zeigen: Zähler, Popup, Münzen, Konfetti, Rest des Feldes. */
    _celebrate() {
      const info = this.cashInfo;
      if (info.shown) return;
      info.shown = true;
      this.pending = Math.max(0, this.pending - this.win);
      const m100 = info.m100;
      if (info.maxed) {
        this.banner = [t("chip.max_win"), 3.0];
        ui.spawnConfetti(this.width, this.height, 140);
        this._coinShower(90);
        this.playSound("level");
        this.rumble(260);
      } else if (info.clear || m100 >= PARTY_MULT) {
        ui.spawnConfetti(this.width, this.height, 110);
        this._coinShower(60);
        this.playSound("level");
        this.rumble(200);
      } else {
        if (m100 >= BIG_MULT) this._coinShower(28);
        this.playSound("win");
      }
      this.mood = ["cheer", 0];
      this._revealRest(this._openedLast());
    }

    /** Ursprung der Rest-Aufdeckung: zuletzt gefundener Edelstein. */
    _openedLast() {
      let last = null;
      const e = this.fairlog[this.fairlog.length - 1];
      if (e) {
        const o = e.o;
        if (isDict(o) && Array.isArray(o.open) && o.open.length) last = o.open[o.open.length - 1];
      }
      return isInt(last) && last >= 0 && last < this.cells.length ? last : fdiv(this.cells.length, 2);
    }

    /** Alle noch verdeckten Felder abgedunkelt aufdecken - als Welle ab origin. */
    _revealRest(origin) {
      const n = this.grid;
      const ox = origin % n, oy = fdiv(origin, n);
      this.cells.forEach((c, i) => {
        if (c.face !== "hidden") return;
        const dx = (i % n) - ox, dy = fdiv(i, n) - oy;
        const d = Math.sqrt(dx * dx + dy * dy);
        c.face = this.isMine[i] ? "mine" : "gem";
        c.dim = true;
        c.flip = 0;
        c.delay = (0.08 + WAVE_T * d) * this._tf();
      });
    }

    /** Mine geht hoch: Blitz, Feuer, Rauch, Trümmer, Wackeln. */
    _explode() {
      const i = this.hit;
      const r = this.tileR[i];
      const x = cxOf(r), y = cyOf(r);
      this.boomT = 0;
      this.shake = SHAKE_T;
      this.flashes.push({ x, y, t: 0, dur: 0.45, r: trunc(this.cell * 1.6), col: null });
      this._spawn(x, y, 26, [255, 196, 80], 380, false, 4.0, 0.45, 300, "fire");
      this._spawn(x, y, 18, [255, 110, 50], 300, false, 3.6, 0.55, 200, "fire");
      this._spawn(x, y, 14, [84, 78, 90], 120, true, 7.0, 1.3, -60, "smoke");
      this._spawn(x, y, 16, [70, 60, 66], 340, true, 3.0, 1.0, 1100, "debris");
      this.playSound("explode");
      this.rumble(320);
      this.mood = ["scared", 0];
      this.sooty = true;
      this.cartFill = 0;
      this._revealRest(i);
      for (const c of this.cells) if (c.dim) c.delay += 0.22 * this._tf();
    }

    /** Edelstein fertig aufgeklappt: Funkeln, steigender Ton, Popup. */
    _gemFound(i) {
      const r = this.tileR[i];
      const k = Math.max(1, this.cells[i].k);
      const cx = cxOf(r), cy = cyOf(r);
      this._spawn(cx, cy, 12, [120, 255, 220], 200, false, 2.6, 0.5, 120, "spark");
      this._spawn(cx, cy, 6, [255, 255, 255], 150, false, 2.0, 0.4, 60, "star");
      this.flashes.push({ x: cx, y: cy, t: 0, dur: 0.3, r: trunc(this.cell * 0.9), col: [40, 160, 130] });
      const step = GEM_STEPS[Math.min(GEM_STEPS.length - 1, Math.max(0, k - 1))];
      this.tone(523.25 * Math.pow(2, step / 12), 0.12, "sine", 0.28);
      if (this.state === PLAY && this.round) {
        const m100 = ML.mult100(this.grid, this.mines, k);
        this._popup(cx, cy - trunc(this.cell * 0.15), this._fmt(m100), ui.GREEN, 0.9);
        this.mood = ["happy", 0];
        this.cartFill = this._cartLevel();
      } else if (this.state === CASHED) {
        this.cartFill = this.cashInfo.clear ? 7 : Math.max(1, this.cartFill);
      }
    }

    /** Ergebnis schließen: Ruhezustand, das aufgedeckte Feld bleibt sichtbar. */
    _toIdle() {
      if (this.state === CASHED && !this.cashInfo.shown) this._celebrate();
      for (const c of this.cells) {
        if (c.flip < 1 || c.delay > 0) {
          c.flip = 1;
          c.delay = 0;
        }
      }
      this.state = IDLE;
      this.result = null;
      this.boomT = null;
      this.resT = 0;
      this.pending = 0;
      if (this.mood[0] === "scared") this.mood = ["idle", 0];
      this._clampBet();
      this._checkBroke();
    }

    // ----- Einstellungen ändern ------------------------------------------------
    /** Vor einer Änderung im Ruhezustand: altes Feld verdecken. */
    _idleChange() {
      if (this.field.length || this.cells.some((c) => c.face !== "hidden")) this._blankBoard();
    }
    _setBet(direction) {
      if (this.state !== IDLE || this.auto) return;
      const i = ML.BETS.indexOf(this.bet);
      const j = Math.max(0, Math.min(ML.BETS.length - 1, i + direction));
      if (j !== i) {
        this.bet = ML.BETS[j];
        this._setOpt("bet", this.bet);
        this.playSound("select");
        if (this.bet > bank.balance()) this._say(t("cas.not_enough"), 1.2);
      }
    }
    _setGrid(grid) {
      if (this.state !== IDLE || this.auto || grid === this.grid || !ML.GRIDS.includes(grid)) return;
      this.grid = grid;
      this.mines = ML.clampMines(grid, this.mines);
      this._setOpt("grid", grid);
      this._setOpt("mines", this.mines);
      this.cursor = fdiv(grid * grid, 2);
      this._layout();
      this._blankBoard();
      this.cartFill = 0;
      this.playSound("click");
    }
    _setMines(mines) {
      if (this.state !== IDLE || this.auto) return;
      mines = ML.clampMines(this.grid, mines);
      if (mines === this.mines) return;
      this.mines = mines;
      this._setOpt("mines", mines);
      this._idleChange();
      this.playSound("click");
    }
    _toggleTurbo() {
      this.turbo = !this.turbo;
      this._setOpt("turbo", this.turbo);
      this._say(this.turbo ? t("lmi.turbo_on") : t("lmi.turbo_off"), 1.0);
      this.playSound("click");
    }
    _openWindow(name) {
      if (this.window === name) {
        this.window = null;
        this.playSound("click");
        return;
      }
      if (name === "auto") {
        if (this.auto) {
          this._stopAuto(t("chip.auto_done"));
          return;
        }
        if ((this.state === BOOM || this.state === CASHED) && this._resultVisible() && this.resT >= READ_T) this._toIdle();
        if (this.state !== IDLE || this.broke) {
          this._say(t("lmi.locked"), 1.2);
          return;
        }
        this._idleChange();
        this.kb = false;
      }
      this.window = name;
      if (name === "fair") {
        this.fairScroll = 0;
        this.fairSel = this.fairlog.length ? this.fairlog.length - 1 : null;
      }
      this.playSound("click");
    }

    // ----- Auto-Modus ------------------------------------------------------------
    _marks() {
      return this.marks[String(this.grid)];
    }
    _toggleMark(i) {
      const lst = this._marks();
      const at = lst.indexOf(i);
      if (at >= 0) {
        lst.splice(at, 1);
      } else {
        if (lst.length >= ML.gems(this.grid, this.mines)) {
          this._say(t("lmi.auto_too_many", { n: ML.gems(this.grid, this.mines) }), 1.4);
          this.playSound("hit");
          return;
        }
        lst.push(i);
        lst.sort((a, b) => a - b);
        this.cells[i].mark = 0;
      }
      this._saveExtra();
      this.playSound("rotate");
    }
    _clearMarks() {
      this.marks[String(this.grid)] = [];
      this._saveExtra();
      this.playSound("click");
    }
    _startAuto() {
      const lst = this._marks();
      if (!lst.length) {
        this._say(t("lmi.auto_need"), 1.6);
        this.playSound("hit");
        return;
      }
      if (lst.length > ML.gems(this.grid, this.mines)) {
        this._say(t("lmi.auto_too_many", { n: ML.gems(this.grid, this.mines) }), 1.6);
        this.playSound("hit");
        return;
      }
      if (this.bet > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      this.window = null;
      this.auto = { left: this.autoRounds, net: 0, t: 0, queue: [] };
      this.playSound("powerup");
    }
    /** Auto beenden. Eine laufende Runde bleibt offen - der Spieler macht von
     *  Hand weiter (aufdecken oder auszahlen). */
    _stopAuto(msg) {
      if (!this.auto) return;
      this.auto = null;
      if (msg) this._say(msg, 2.0);
      this.playSound("click");
      if (this.state === IDLE) this._checkBroke();
    }
    _autoTick(dt) {
      const a = this.auto;
      a.t -= dt;
      if (a.t > 0) return;
      if (this.state === IDLE) {
        // Grenzen prüfen, dann die nächste Runde
        if (a.left <= 0) return this._stopAuto(t("chip.auto_done"));
        if (this.autoWin > 0 && a.net >= this.autoWin) return this._stopAuto(t("chip.auto_hit_win"));
        if (this.autoLoss > 0 && -a.net >= this.autoLoss) return this._stopAuto(t("chip.auto_hit_loss"));
        if (this.bet > bank.balance()) return this._stopAuto(t("chip.auto_no_chips"));
        const lst = this._marks();
        if (!lst.length || lst.length > ML.gems(this.grid, this.mines)) return this._stopAuto(t("lmi.auto_need"));
        if (this._play()) {
          a.left -= 1;
          a.queue = lst.slice();
          a.t = (AUTO_STEP_T + 0.1) * this._tf();
        } else {
          this._stopAuto(t("chip.auto_no_chips"));
        }
      } else if (this.state === PLAY) {
        if (this.cells.some((c) => c.flip < 1)) return;
        if (a.queue.length) {
          this._reveal(a.queue.shift());
          a.t = AUTO_STEP_T * this._tf();
        } else {
          this._cashOut(true);
          this.resDelay = 0;
          this._celebrate();
        }
      }
    }

    // ===================================================== Effekte
    _spawn(x, y, n, col, speed, up, r, life, grav, kind = "dot") {
      const Rn = this.rng;
      for (let i = 0; i < n; i++) {
        const a = up ? Rn.uniform(Math.PI * 1.1, Math.PI * 1.9) : Rn.uniform(0, TAU);
        const sp = speed * Rn.uniform(0.3, 1.0) * this.k;
        this.particles.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, r: r * this.k * Rn.uniform(0.6, 1.25), col,
          t: 0, life: life * Rn.uniform(0.7, 1.15), g: grav * this.k, kind });
      }
      if (this.particles.length > 260) this.particles = this.particles.slice(-260);
    }
    _coinShower(n = 40) {
      const Rn = this.rng, W = this.width, k = this.k;
      for (let i = 0; i < n; i++) {
        this.coinsFx.push({
          x: Rn.uniform(W * 0.05, W * 0.95), y: Rn.uniform(-this.height * 0.4, -10),
          vx: Rn.uniform(-60, 60) * k, vy: Rn.uniform(-80, 120) * k, spin: Rn.uniform(0, TAU),
          vs: Rn.uniform(6, 14), r: Rn.uniform(7, 12) * Math.max(0.7, k),
        });
      }
      if (this.coinsFx.length > 180) this.coinsFx = this.coinsFx.slice(-180);
    }
    _popup(x, y, text, color, dur = 1.4) {
      this.popups.push({ x, y, text, color: color || ui.GOLD, t: 0, dur });
    }

    // ===================================================== Eingabe
    _tileAt(pos) {
      for (let i = 0; i < this.tileR.length; i++) if (this.tileR[i].collidepoint(pos)) return i;
      return null;
    }

    handleEvent(ev) {
      const kind = ev.kind;
      if ((kind === "mousemove" || kind === "mousedown" || kind === "mouseup") && ev.pos) {
        if (kind === "mousemove" && this.boardR.collidepoint(ev.pos)) this.kb = false;
        this.mouse = ev.pos;
      }
      if (this.broke) {
        if (kind === "mousedown" || (kind === "keydown" && isAction(ev.key))) this._takeCredit();
        return;
      }
      if (kind === "keydown") {
        this._onKey(ev.key, !!ev.repeat);
      } else if (kind === "wheel") {
        if (this.window === "fair") this.fairScroll = Math.max(0, this.fairScroll - (ev.delta > 0 ? 1 : -1));
        else if (this.window === null && this.state === IDLE) this._setBet(ev.delta > 0 ? 1 : -1);
      } else if (kind === "mousedown" && ev.button === 1) {
        this._onClick(ev.pos);
      }
    }

    _onKey(key, repeat = false) {
      if (!key) return;
      const low = key.length === 1 ? key.toLowerCase() : key;
      // Fenster ----------------------------------------------------------
      if (key === "Escape") {
        if (this.window !== null) {
          this.window = null;
          this.playSound("click");
        } else if (this.auto) {
          this._stopAuto(t("chip.auto_done"));
        }
        return;
      }
      if (key.length === 1 && (low === "a" || low === "s" || low === "f" || low === "t")) {
        if (repeat) return;
        if (low === "t") this._toggleTurbo();
        else this._openWindow({ a: "auto", s: "stats", f: "fair" }[low]);
        return;
      }
      if (this.window === "auto") {
        this._autoKey(key);
        return;
      }
      if (this.window !== null) {
        if (isAction(key) && !repeat) this.window = null;
        return;
      }
      // Auto läuft: Leertaste/Enter = STOPP --------------------------------
      if (this.auto) {
        if (isAction(key) && !repeat) this._stopAuto(t("chip.auto_done"));
        return;
      }
      // Ergebnis: nach kurzer Lesezeit weiter wie im Ruhezustand ------------
      if (this.state === BOOM || this.state === CASHED) {
        if (!this._resultVisible() || this.resT < READ_T || repeat) return;
        this._toIdle();
        if (this.broke) return;
      }
      if (this.state === IDLE) {
        if (isAction(key)) {
          if (!repeat) this._play();
        } else if (key === "1" || key === "2" || key === "3" || key === "4") {
          this._setGrid(ML.GRIDS[Number(key) - 1]);
        } else if (key === "Left" || key === "Right") {
          this._setMines(this.mines + (key === "Right" ? 1 : -1));
        } else if (key === "plus" || key === "KP_Add" || key === "Up" || key === "equal") {
          this._setBet(1);
        } else if (key === "minus" || key === "KP_Subtract" || key === "Down") {
          this._setBet(-1);
        }
        return;
      }
      if (this.state === PLAY) {
        if (key === "Left" || key === "Right" || key === "Up" || key === "Down") {
          this._moveCursor(key);
        } else if (key === "space") {
          if (!repeat) {
            if (!this.kb) this.kb = true;
            else this._reveal(this.cursor);
          }
        } else if (key === "Return" || key === "KP_Enter" || low === "c") {
          if (!repeat) this._cashOut();
        } else if (low === "r") {
          if (!repeat) this._randomTile();
        }
      }
    }

    /** Pfeiltaste: Cursor bewegen (der erste Druck zeigt ihn nur). */
    _moveCursor(key) {
      const n = this.grid;
      if (this.kb) {
        let x = this.cursor % n, y = fdiv(this.cursor, n);
        x += key === "Left" ? -1 : key === "Right" ? 1 : 0;
        y += key === "Up" ? -1 : key === "Down" ? 1 : 0;
        this.cursor = Math.max(0, Math.min(n - 1, y)) * n + Math.max(0, Math.min(n - 1, x));
      }
      this.kb = true;
    }

    /** Tasten im Auto-Fenster: Pfeile + Leertaste markieren, Enter startet. */
    _autoKey(key) {
      if (key === "Left" || key === "Right" || key === "Up" || key === "Down") {
        this._moveCursor(key);
      } else if (key === "space") {
        if (this.kb) this._toggleMark(this.cursor);
        this.kb = true;
      } else if (key === "Return" || key === "KP_Enter") {
        this._startAuto();
      } else if (key === "Delete" || key === "BackSpace") {
        this._clearMarks();
      }
    }

    /** R: zufälliges verdecktes Feld (Zufall der Oberfläche, nicht der Runde). */
    _randomTile() {
      if (this.state !== PLAY || !this.round) return;
      const free = [];
      this.cells.forEach((c, i) => {
        if (c.face === "hidden") free.push(i);
      });
      if (!free.length) return;
      const i = free[this.rng.randrange(free.length)];
      this.cursor = i;
      this._reveal(i);
    }

    _onClick(pos) {
      // Werkzeug-Knöpfe (immer erreichbar)
      for (const [name, r] of this.rTools) {
        if (r.collidepoint(pos)) {
          if (name === "turbo") this._toggleTurbo();
          else this._openWindow(name);
          return;
        }
      }
      if (this.window === "auto") return this._autoClick(pos);
      if (this.window !== null) return this._windowClick(pos);
      if (this.auto) {
        if (this.rGo.collidepoint(pos)) this._stopAuto(t("chip.auto_done"));
        return;
      }
      if (this.state === BOOM || this.state === CASHED) {
        if (!this._resultVisible() || this.resT < READ_T) return;
        this._toIdle();
        if (this.broke) return;
        if (!this.rGo.collidepoint(pos)) return;
      }
      if (this.state === IDLE) {
        if (this.rMinus.collidepoint(pos)) this._setBet(-1);
        else if (this.rPlus.collidepoint(pos)) this._setBet(1);
        else if (this.rGo.collidepoint(pos)) this._play();
        else if (this.rCash.collidepoint(pos)) this._say(t("lmi.press_play"), 1.4);
        else if (this.boardR.collidepoint(pos)) {
          this._say(t("lmi.press_play"), 1.4);
          this.playSound("hit");
        } else {
          for (let i = 0; i < this.rGrids.length; i++) {
            if (this.rGrids[i].collidepoint(pos)) return this._setGrid(ML.GRIDS[i]);
          }
          if (this.rMminus.collidepoint(pos)) this._setMines(this.mines - 1);
          else if (this.rMplus.collidepoint(pos)) this._setMines(this.mines + 1);
          else {
            for (let i = 0; i < this.rPresets.length; i++) {
              if (this.rPresets[i].collidepoint(pos)) return this._setMines(ML.MINE_PRESETS[this.grid][i]);
            }
          }
        }
        return;
      }
      if (this.state === PLAY) {
        if (this.rCash.collidepoint(pos)) this._cashOut();
        else if (this.rRandom.collidepoint(pos)) this._randomTile();
        else {
          const i = this._tileAt(pos);
          if (i !== null) {
            this.kb = false;
            this.cursor = i;
            this._reveal(i);
          }
        }
      }
    }

    _autoClick(pos) {
      if (this.rAutoClose.collidepoint(pos)) {
        this.window = null;
        this.playSound("click");
        return;
      }
      const i = this._tileAt(pos);
      if (i !== null) {
        this.kb = false;
        this.cursor = i;
        this._toggleMark(i);
        return;
      }
      for (let row = 0; row < AUTO_FIELDS.length; row++) {
        const [opts, key, field] = AUTO_FIELDS[row];
        const rects = this.autoRows[row][1];
        for (let j = 0; j < rects.length; j++) {
          if (rects[j].collidepoint(pos)) {
            this[field] = opts[j];
            this._setOpt(key, opts[j]);
            this.playSound("click");
            return;
          }
        }
      }
      if (this.rAutoClear.collidepoint(pos)) this._clearMarks();
      else if (this.rAutoStart.collidepoint(pos)) this._startAuto();
    }

    /** Klick in Statistik-/Fairness-Fenster (außerhalb = schließen). */
    _windowClick(pos) {
      const [rect, close] = this._windowRect();
      if (close.collidepoint(pos) || !rect.collidepoint(pos)) {
        this.window = null;
        this.playSound("click");
        return;
      }
      if (this.window !== "fair") return;
      if (this.rFairNew.collidepoint(pos)) {
        if (this.round) {
          this._say(t("lmi.locked"), 1.4);
          this.playSound("hit");
        } else {
          F.newClient(this.fair);
          this._saveExtra();
          this.playSound("select");
        }
        return;
      }
      if (this.rFairVerify.collidepoint(pos) && this.fairSel !== null) {
        this._verify(this.fairSel);
        return;
      }
      for (const [idx, r] of this.rFairRows) {
        if (r.collidepoint(pos)) {
          this.fairSel = idx;
          this.playSound("click");
          return;
        }
      }
    }

    /** Nachrechnen: Prüfcode = sha256(Seed) und Minenfeld aus den Seeds. */
    _verify(idx) {
      if (!(idx >= 0 && idx < this.fairlog.length)) return;
      const e = this.fairlog[idx];
      let ok = F.verify(e);
      const o = e.o;
      let field = null;
      if (isDict(o) && ML.GRIDS.includes(o.grid) && isInt(o.mines) && o.mines >= 1 && o.mines <= ML.maxMines(o.grid)) {
        field = ML.fieldFromSeed(o.grid, o.mines, e.s, e.c, e.n);
        const opened = Array.isArray(o.open) ? o.open : [];
        const m = o.m;
        if (m === 0) {
          ok = ok && opened.length > 0 && field.includes(opened[opened.length - 1]) && !opened.slice(0, -1).some((v) => field.includes(v));
        } else {
          ok = ok && !opened.some((v) => field.includes(v)) && m === ML.mult100(o.grid, o.mines, opened.length);
        }
      }
      this.fairChecks.set(e.n, [ok, field]);
      this.playSound(ok ? "point" : "hit");
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
        const step = Math.max(Math.abs(diff) * Math.min(1, dt * 4.5), 60 * dt);
        this.shownChips += Math.sign(diff) * Math.min(Math.abs(diff), step);
        if (diff > 0) {
          this._tickCd -= dt;
          if (this._tickCd <= 0) {
            this._tickCd = 0.07;
            this.tone(1500 + this.rng.random() * 300, 0.025, "square", 0.08);
          }
        }
      }
      if (this.toast) {
        this.toast[1] -= dt;
        if (this.toast[1] <= 0) this.toast = null;
      }
      if (this.banner) {
        this.banner[1] -= dt;
        if (this.banner[1] <= 0) this.banner = null;
      }
      this.shake = Math.max(0, this.shake - dt);
      this.mood[1] += dt;
      this._updateCells(dt);
      this._updateFx(dt);
      if (this.state === BOOM) {
        if (this.boomT === null) {
          if (this.cells[this.hit].flip >= 1) this._explode();
        } else {
          this.boomT += dt;
        }
      }
      if (this.state === BOOM || this.state === CASHED) {
        if (this.resDelay > 0) {
          this.resDelay -= dt;
          if (this.resDelay <= 0 && this.state === CASHED) this._celebrate();
        } else {
          this.resT += dt;
          const limit = this.auto ? AUTO_RESULT_T * this._tf() : RESULT_T;
          if (this.resT >= limit) {
            this._toIdle();
            if (this.auto) this.auto.t = AUTO_PAUSE_T * this._tf();
          }
        }
      }
      if (this.auto && (this.state === IDLE || this.state === PLAY)) this._autoTick(dt);
    }

    _resultVisible() {
      return this.result !== null && this.resDelay <= 0;
    }

    _updateCells(dt) {
      const ft = FLIP_T * this._tf();
      this.cells.forEach((c, i) => {
        c.mark += dt;
        if (c.delay > 0) {
          c.delay -= dt;
          if (c.delay > 0) return;
        }
        if (c.flip < 1) {
          c.flip = Math.min(1, c.flip + dt / ft);
          if (c.flip >= 1) {
            c.pop = 0;
            if (c.face === "gem" && !c.dim) this._gemFound(i);
          }
        } else if (c.pop < POP_T) {
          c.pop = Math.min(POP_T, c.pop + dt);
        }
      });
    }

    _updateFx(dt) {
      this.particles = this.particles.filter((p) => {
        p.t += dt;
        p.vy += p.g * dt;
        if (p.kind === "smoke") {
          p.vx *= Math.max(0, 1 - dt * 1.5);
          p.vy *= Math.max(0, 1 - dt * 1.5);
        }
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
      for (const f of this.flashes) f.t += dt;
      this.flashes = this.flashes.filter((f) => f.t < f.dur);
    }

    // ===================================================== Text-Helfer
    // Höhen wie in pygame-ce: font.get_height() ist dort genau die Pixelgröße
    // (13 bei 13 px), das gerenderte Textbild ist höher (16). Deshalb steht für
    // Python-"font.get_height()" hier font.px, für "img.get_height()" die
    // Bildhöhe font.height (= h der Objekte aus _fit).
    /** Text in der ersten Schrift, die passt - sonst gekürzt mit "…". */
    _fit(text, fonts, maxw, col) {
      text = String(text);
      for (const f of fonts) {
        const w = f.width(text);
        if (w <= maxw) return { s: text, f, col, w, h: f.height };
      }
      const f = fonts[fonts.length - 1];
      while (text && f.width(text + "…") > maxw) text = text.slice(0, -1);
      const s = text + "…";
      return { s, f, col, w: f.width(s), h: f.height };
    }
    /** "Gerendertes" Textbild aus _fit zeichnen (Anker wie Rect-Attribute). */
    _put(ctx, img, x, y, anchor = "topleft", alpha) {
      const w = img.w, h = img.h;
      let x0 = x, y0 = y;
      // wie pygame: Rechteck-Anker mit Ganzzahl-Hälften
      if (anchor === "center" || anchor === "midtop" || anchor === "midbottom") x0 = x - fdiv(w, 2);
      else if (anchor === "midright" || anchor === "topright" || anchor === "bottomright") x0 = x - w;
      if (anchor === "center" || anchor === "midleft" || anchor === "midright") y0 = y - fdiv(h, 2);
      else if (anchor === "midbottom" || anchor === "bottomleft" || anchor === "bottomright") y0 = y - h;
      return ui.text(ctx, img.s, trunc(x0), trunc(y0), img.f, img.col, "topleft", alpha);
    }
    _text(ctx, str, x, y, font, col, anchor = "topleft", alpha) {
      return this._put(ctx, { s: String(str), f: font, col, w: font.width(String(str)), h: font.height }, x, y, anchor, alpha);
    }
    _wrap(text, font, maxw) {
      const words = String(text).split(" ");
      const lines = [];
      let cur = "";
      for (const wd of words) {
        const test = cur ? cur + " " + wd : wd;
        if (font.width(test) <= maxw || !cur) cur = test;
        else {
          lines.push(cur);
          cur = wd;
        }
      }
      if (cur) lines.push(cur);
      return lines;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      const bg = D.background(this.width, this.height);
      ctx.drawImage(bg, 0, 0, bg.lw, bg.lh);
      this._drawBoard(ctx);
      this._drawPanel(ctx);
      this._drawHud(ctx);
      this._drawTools(ctx);
      this._drawHistory(ctx);
      this._drawStrip(ctx);
      this._drawFx(ctx);
      if (this.result !== null && this._resultVisible()) this._drawResult(ctx);
      if (this.banner) this._drawBanner(ctx);
      if (this.window === "stats" || this.window === "fair") this._drawWindow(ctx);
      if (this.toast) this._drawToast(ctx);
      this._drawTooltip(ctx);
      if (this.broke) this._drawBroke(ctx);
    }

    // ----- Spielbrett -------------------------------------------------------------
    _shakeOff() {
      if (this.shake <= 0) return [0, 0];
      const a = (7 * this.k * this.shake) / SHAKE_T;
      return [trunc(Math.sin(this.time * 71) * a), trunc(Math.cos(this.time * 53) * a * 0.7)];
    }

    _drawBoard(ctx) {
      const [ox, oy] = this._shakeOff();
      const br = this.boardR.move(ox, oy);
      const bimg = D.board(br.w, this.boardPad);
      ctx.drawImage(bimg, br.x, br.y, bimg.lw, bimg.lh);
      const cell = this.cell;
      const play = this.state === PLAY && this.window === null && this.auto === null;
      const marking = this.window === "auto" || this.auto !== null;
      const hover = (play || this.window === "auto") && !this.kb ? this._tileAt(this.mouse) : null;
      const marks = marking ? this._marks() : [];
      this.cells.forEach((c, i) => {
        const r = this.tileR[i].move(ox, oy);
        const f = c.flip;
        if (c.face === "hidden") {
          if (f < 1) {
            // Einwachsen beim Rundenstart (Welle von links oben)
            if (c.delay > 0) return;
            const sz = Math.max(2, trunc(cell * (0.55 + 0.45 * f)));
            D.blitCenter(ctx, D.tileHidden(cell, this._var(i)), cxOf(r), cyOf(r), sz, sz);
            return;
          }
          const hot = i === hover;
          const img = D.tileHidden(cell, this._var(i), hot);
          ctx.drawImage(img, r.x, hot ? r.y - Math.max(1, trunc(2 * this.k)) : r.y, cell, cell);
          if (marks.includes(i)) D.drawMark(ctx, r, this.accent, this.time, i);
          return;
        }
        if (c.delay > 0) {
          ctx.drawImage(D.tileHidden(cell, this._var(i)), r.x, r.y, cell, cell); // wartet aufs Aufklappen
          return;
        }
        if (f < 1) {
          this._drawFlip(ctx, r, i, c);
          return;
        }
        this._drawFace(ctx, r, i, c);
      });
      // Tastatur-Cursor
      if (this.kb && (play || this.window === "auto")) {
        const g = Math.max(4, trunc(6 * this.k));
        const r = inflate(this.tileR[this.cursor].move(ox, oy), g, g);
        const pulse = 0.5 + 0.5 * Math.sin(this.time * 6);
        const col = ui.mix(CD.COL_GOLD, [255, 255, 255], 0.45 * pulse);
        draw.rect(ctx, col, r, Math.max(2, trunc(3 * this.k)), Math.max(3, trunc(cell * 0.2)));
      }
      this._drawParticles(ctx);
      for (const fl of this.flashes) {
        const p = fl.t / fl.dur;
        const rr = Math.max(2, trunc(fl.r * (0.4 + 0.8 * p)));
        const col = fl.col || [255, 200, 120];
        D.drawGlow(ctx, rr, col, fl.x - rr - 1 + ox, fl.y - rr - 1 + oy, 1 - p);
        if (!fl.col && p < 0.5) {
          const ring = Math.max(2, trunc(fl.r * (0.3 + 1.2 * p)));
          draw.circle(ctx, ui.mix([255, 240, 200], [255, 140, 60], p * 2), [fl.x + ox, fl.y + oy], ring, Math.max(1, trunc(4 * this.k * (1 - p * 2))));
        }
      }
    }

    _var(i) {
      return D.hash(i * 7 + this.grid) % 4;
    }

    /** Aufklappen: erst schrumpft der Stein (Breite), dann wächst die Vorderseite. */
    _drawFlip(ctx, r, i, c) {
      const f = c.flip, cell = this.cell;
      const lift = Math.sin(Math.PI * f) * 0.1;
      let img, w;
      if (f < 0.5) {
        img = D.tileHidden(cell, this._var(i));
        w = Math.max(1, trunc(cell * (1 - 2 * f) * (1 + lift)));
      } else {
        img = D.face(cell, c.face, c.dim);
        w = Math.max(1, trunc(cell * (2 * f - 1) * (1 + lift)));
      }
      const h = Math.max(1, trunc(cell * (1 + lift)));
      D.blitCenter(ctx, img, cxOf(r), cyOf(r) - trunc(cell * lift * 0.5), w, h);
    }

    /** Aufgedecktes Feld in Ruhe (Edelstein federt nach, funkelt). */
    _drawFace(ctx, r, i, c) {
      const cell = this.cell, kind = c.face;
      if (kind === "gem" && !c.dim && c.pop < POP_T) {
        ctx.drawImage(D.tileOpen(cell), r.x, r.y, cell, cell);
        const p = c.pop / POP_T;
        const sc = 0.8 * (1 + 0.22 * Math.sin(Math.PI * p) * (1 - p * 0.3));
        const g = Math.max(4, fdiv(trunc(cell * sc), 2) * 2);
        D.blitCenter(ctx, D.gem(g), cxOf(r), cyOf(r));
        return;
      }
      ctx.drawImage(D.face(cell, kind, c.dim), r.x, r.y, cell, cell);
      if (kind === "gem" && !c.dim) {
        const ph = Math.sin(this.time * 2.6 + i * 1.7);
        if (ph > 0.55) {
          const a = (ph - 0.55) / 0.45;
          const v = D.hash(i * 13 + 5);
          const sx = r.x + cell * (0.3 + (v % 40) / 100);
          const sy = r.y + cell * (0.22 + ((v >>> 8) % 30) / 100);
          D.sparkle(ctx, sx, sy, Math.max(1.5, cell * 0.1 * a), [220, 255, 246]);
        }
      } else if (kind === "boom") {
        // glimmender Rand um die getroffene Mine
        const pulse = 0.5 + 0.5 * Math.sin(this.time * 5);
        draw.rect(ctx, ui.mix([255, 90, 70], [255, 200, 120], pulse), inflate(r, 2, 2), Math.max(2, trunc(3 * this.k)), Math.max(2, trunc(cell * 0.16)));
      }
    }

    _drawParticles(ctx) {
      for (const p of this.particles) {
        const f = p.t / p.life;
        if (p.kind === "smoke") {
          // weicher Rauch: additiver grauer Lichthof, wird größer und blasser
          const lvl = Math.min(5, trunc((1 - f) * 6));
          if (lvl <= 0) continue;
          const rr = Math.max(2, fdiv(trunc(p.r * (1 + 2.2 * f)), 2) * 2);
          D.drawGlow(ctx, rr, p.col, trunc(p.x) - rr - 1, trunc(p.y) - rr - 1, lvl / 6);
          continue;
        }
        if (p.kind === "star") {
          D.sparkle(ctx, p.x, p.y, Math.max(1, p.r * 1.6 * (1 - f)), [255, 255, 255]);
          continue;
        }
        const r = p.r * (1 - f);
        const col = p.kind === "fire" ? ui.mix(p.col, [120, 40, 30], f) : p.col;
        if (r >= 0.5) draw.circle(ctx, col, [trunc(p.x), trunc(p.y)], Math.max(1, trunc(r)));
      }
    }

    // ----- Seitenleiste -----------------------------------------------------------
    _drawPanel(ctx) {
      if (this.window === "auto") {
        this._drawAuto(ctx); // eigenes Fenster an derselben Stelle
        return;
      }
      ui.drawPanel(ctx, this.panelR, { accentTop: this.accent });
      this._drawMascot(ctx);
      this._drawHead(ctx);
      if (this.state === IDLE && !this.round) {
        this._drawSetup(ctx);
        this._drawInfo(ctx, this.rPresets[0].bottom + Math.max(4, trunc(8 * this.k)));
      } else {
        this._drawInfo(ctx, this.infoY);
      }
      this._drawLadder(ctx);
      this._drawHint(ctx);
    }

    /** Maskottchen mit Spitzhacke und Lore in r (Standard: Kopf der Seitenleiste). */
    _drawMascot(ctx, rect) {
      const r = rect || this.rMascot;
      const ms = r.w, k = this.k;
      const [mood, mt] = this.mood;
      let pose = "stand", dx = 0, dy = 0;
      if (mood === "happy" && mt < 0.45) {
        pose = "hop";
        dy = -Math.sin((Math.PI * mt) / 0.45) * ms * 0.14;
      } else if (mood === "cheer" && mt < 1.35) {
        const ph = (mt % 0.45) / 0.45;
        pose = "hop";
        dy = -Math.sin(Math.PI * ph) * ms * 0.16;
      } else if (mood === "scared" || (this.sooty && this.state === BOOM)) {
        pose = "fall";
        if (mt < 0.6) {
          dx = Math.sin(mt * 60) * ms * 0.04 * (1 - mt / 0.6);
          dy = -Math.sin(Math.PI * Math.min(1, mt / 0.35)) * ms * 0.08;
        }
      }
      if (pose === "stand") dy += Math.sin(this.time * 2.2) * ms * 0.012;
      const ground = r.bottom - Math.max(1, trunc(2 * k));
      // Felsvorsprung, Spitzhacke, Lore
      draw.ellipse(ctx, [34, 30, 44], [r.x - trunc(ms * 0.04), ground - trunc(ms * 0.05), trunc(ms * 1.0 + this.cartW * 1.05), trunc(ms * 0.12)]);
      D.drawPickaxe(ctx, r.x + trunc(ms * 0.1), ground - trunc(ms * 0.02), ms * 0.4, k);
      const shw = trunc(ms * 0.5), shh = Math.max(3, trunc(ms * 0.07));
      draw.ellipse(ctx, [14, 12, 20], [r.x + trunc(ms * 0.52) - fdiv(shw, 2), ground - fdiv(shh, 2), shw, shh]);
      const img = D.minerLlama(ms, pose, this.accent, this.sooty);
      ctx.drawImage(img, trunc(r.x + dx), trunc(r.y + dy + ms * 0.04), img.lw, img.lh);
      // Lichthof der Stirnlampe (additiv)
      const lr = Math.max(3, trunc(ms * 0.1));
      const flick = 0.85 + 0.15 * Math.sin(this.time * 9) * Math.sin(this.time * 3.3);
      const lx = r.x + dx + ms * 0.835;
      const ly = r.y + dy + ms * 0.04 + ms * 0.073;
      D.drawGlow(ctx, lr, [150, 130, 60], trunc(lx - lr - 1), trunc(ly - lr - 1), flick);
      // Lore mit Edelsteinen
      D.drawCart(ctx, r.x + trunc(ms * 0.92), ground, this.cartW, k, this.cartFill, Math.max(4, trunc(this.cartW * 0.3)));
      // Reaktion: Funkeln (Freude) bzw. Ausrufezeichen + Rauch (Schreck)
      const hx = r.x + dx + ms * 0.74, hy = r.y + dy + ms * 0.04;
      if ((mood === "happy" || mood === "cheer") && mt < (mood === "happy" ? 0.6 : 1.4)) {
        for (let j = 0; j < 3; j++) {
          const a = this.time * 2 + j * 2.1;
          D.sparkle(ctx, hx + Math.cos(a) * ms * 0.22, hy - ms * 0.04 + Math.sin(a) * ms * 0.1,
            Math.max(1.5, ms * 0.05 * (0.6 + 0.4 * Math.sin(this.time * 9 + j))), [200, 255, 236]);
        }
      } else if (mood === "scared" && mt < 1.6) {
        const bx = trunc(hx + ms * 0.18), by = trunc(hy - ms * 0.02);
        const rr = Math.max(5, trunc(ms * 0.09));
        draw.circle(ctx, [250, 250, 250], [bx, by], rr);
        draw.circle(ctx, ui.RED, [bx, by], rr, Math.max(1, trunc(2 * k)));
        this._text(ctx, "!", bx, by, this.fBtn, ui.RED, "center");
        for (let j = 0; j < 3; j++) {
          const ph = (mt * 0.8 + j * 0.33) % 1;
          const pr = Math.max(2, fdiv(trunc(ms * (0.05 + 0.07 * ph)), 2) * 2);
          const px = trunc(r.x + dx + ms * (0.66 - 0.06 * j));
          const py = trunc(r.y + dy - ms * 0.2 * ph + ms * 0.02);
          D.drawGlow(ctx, pr, [80, 76, 86], px - pr - 1, py - pr - 1, 1 - ph);
        }
      }
    }

    /** Kopfzeile rechts neben Maskottchen und Lore: großer Multiplikator. */
    _drawHead(ctx) {
      const x0 = this.headX;
      const x1 = this.panelR.right - this.ppad;
      const w = x1 - x0;
      let y = this.rMascot.y;
      let cap, val, sub, col;
      if (this.state === IDLE && !this.round) {
        cap = t("lmi.max_mult");
        val = this._fmt(ML.mult100(this.grid, this.mines, ML.gems(this.grid, this.mines)));
        sub = null; // Edelsteine/Minen als Bildchen (sprachunabhängig)
        col = CD.COL_GOLD;
      } else if (this.state === BOOM) {
        cap = t("lmi.cur_mult");
        val = "×0";
        sub = t("lmi.lost_short", { n: this._group(this.lost) });
        col = ui.RED;
      } else if (this.state === CASHED) {
        cap = t("lmi.cur_mult");
        val = this._fmt(this.lastMult);
        sub = "+" + this._group(this.win);
        col = CD.COL_GOLD;
      } else {
        const k = this._k();
        cap = t("lmi.cur_mult");
        val = this._fmt(ML.mult100(this.grid, this.mines, k));
        sub = t("lmi.payout_now", { n: this._group(this._winAt(this._roundBet(), k)) });
        col = k > 0 ? ui.GREEN : ui.TEXT;
      }
      let img = this._fit(cap, [this.fTiny, this.fMicro], w, ui.TEXT_DIM);
      this._put(ctx, img, x0, y);
      y += img.h;
      img = this._fit(val, [this.fMult, this.fBig, this.fBtn], w, col);
      this._put(ctx, img, x0, y);
      y += img.h;
      if (sub === null) {
        const ih = Math.max(this.fSmall.px, trunc(this.fBig.px * 0.9));
        this._drawCounts(ctx, x0, y + fdiv(ih, 2) + 1, ML.gems(this.grid, this.mines), this.mines, this.fBtn, ih);
        return;
      }
      this._put(ctx, this._fit(sub, [this.fSmall, this.fTiny, this.fMicro], w, ui.TEXT), x0, y);
    }

    /** Edelstein- und Minen-Bildchen mit Anzahl (links bei x bzw. mittig in
     *  centerW), senkrecht mittig bei cy. */
    _drawCounts(ctx, x, cy, gems, mines, font, ih, centerW) {
      const parts = [[D.gem(ih), this._group(gems), this.accent], [D.mine(ih, true), String(mines), ui.RED]];
      const gap = Math.max(8, fdiv(ih * 2, 3));
      let total = 0;
      for (const [icon, txt] of parts) total += icon.lw + 3 + font.width(txt);
      total += gap;
      if (centerW !== undefined) x += fdiv(centerW - total, 2);
      for (const [icon, txt, col] of parts) {
        ctx.drawImage(icon, x, cy - fdiv(icon.lh, 2), icon.lw, icon.lh);
        x += icon.lw + 3;
        this._text(ctx, txt, x, cy, font, col, "midleft");
        x += font.width(txt) + gap;
      }
    }

    _drawSetup(ctx) {
      const [x0, w] = this.panelInner;
      const hot = this.mouse;
      this._put(ctx, this._fit(t("lmi.grid"), [this.fTiny], w, ui.TEXT_DIM), x0, this.labGridY);
      this.rGrids.forEach((r, i) => {
        const g = ML.GRIDS[i];
        this._btn(ctx, r, g + "×" + g, { active: g === this.grid, hot: r.collidepoint(hot) });
        this._text(ctx, String(i + 1), r.x + Math.max(4, fdiv(r.h, 5)), r.y + Math.max(2, fdiv(r.h, 10)), this.fMicro, ui.TEXT_FAINT);
      });
      const y = this.minesRowY, rh = this.rowH;
      const lab = this._fit(t("lmi.mines"), [this.fSmall, this.fTiny], this.rMminus.x - x0 - 4, ui.TEXT);
      this._put(ctx, lab, x0, y + fdiv(rh, 2), "midleft");
      const mm = this.rMminus, mp = this.rMplus;
      this._btn(ctx, mm, "", { on: this.mines > 1, hot: mm.collidepoint(hot) });
      D.icon(ctx, "left", inflate(mm, fdiv(-mm.w, 3), fdiv(-mm.h, 3)), this.mines > 1 ? ui.TEXT : ui.TEXT_FAINT);
      const mx = ML.maxMines(this.grid);
      this._btn(ctx, mp, "", { on: this.mines < mx, hot: mp.collidepoint(hot) });
      D.icon(ctx, "right", inflate(mp, fdiv(-mp.w, 3), fdiv(-mp.h, 3)), this.mines < mx ? ui.TEXT : ui.TEXT_FAINT);
      draw.rect(ctx, [16, 12, 26], this.rMval, 0, Math.max(4, fdiv(rh, 4)));
      draw.rect(ctx, ui.mix(ui.RED, ui.BORDER, 0.4), this.rMval, 1, Math.max(4, fdiv(rh, 4)));
      this._text(ctx, String(this.mines), cxOf(this.rMval), cyOf(this.rMval), this.fBtn, ui.RED, "center");
      this.rPresets.forEach((r, i) => {
        const v = ML.MINE_PRESETS[this.grid][i];
        this._btn(ctx, r, String(v), { active: v === this.mines, hot: r.collidepoint(hot), small: true });
      });
    }

    /** Zeilen der Gewinn-Info: [[Beschriftung, Wert, Farbe]], Index der Zeile
     *  mit dem Chancen-Balken (oder null) und die Chance in Zehntel-Prozent. */
    _infoRows() {
      const g = this.grid, m = this.mines;
      if (this.state === BOOM || this.state === CASHED) {
        const e = this.fairlog[this.fairlog.length - 1];
        const o = e ? e.o : null;
        const bet = isDict(o) && o.bet !== undefined ? o.bet : 0;
        const paid = this.state === CASHED ? this.win : 0;
        const rows = [
          [t("cas.bet"), this._group(bet), ui.TEXT],
          [t("lmi.found"), this._lastK() + " / " + ML.gems(g, m), ui.TEXT],
          [t("chip.st_paid"), this._group(paid), paid ? CD.COL_GOLD : ui.RED],
        ];
        return [rows, null, 0];
      }
      const k = this.state === PLAY ? this._k() : 0;
      const bet = this._roundBet();
      const nxt = ML.nextMult100(g, m, k);
      const p10 = ML.safePercent(g, m, k);
      const rows = [
        [t("lmi.next_mult"), nxt !== null ? this._fmt(nxt) : "–", ui.GREEN],
        [t("lmi.safe"), this._fmtPct(p10) + " %", ui.TEXT],
        [t("lmi.next_win"), nxt !== null ? this._group(this._winAt(bet, k + 1)) : "–", CD.COL_GOLD],
      ];
      if (this.state === PLAY) rows.push([t("lmi.found"), k + " / " + ML.gems(g, m), ui.TEXT]);
      return [rows, 1, p10];
    }

    /** Gewinn-Info (Ruhe: erstes Feld, Runde: nächstes Feld, Ergebnis: Bilanz).
     *  Die Zeilen verteilen sich auf den freien Platz bis zur Stufen-Leiste. */
    _drawInfo(ctx, y) {
      const [x0, w] = this.panelInner;
      let [rows, barRow, p10] = this._infoRows();
      const bh = Math.max(4, trunc(6 * this.k));
      let barTotal = barRow !== null ? bh + Math.max(2, trunc(4 * this.k)) : 0;
      const avail = this.ladderLabY - Math.max(4, trunc(6 * this.k)) - y;
      const minLh = this.fTiny.px + 2;
      const maxLh = trunc(this.fBtn.px * 1.75);
      while (rows.length && fdiv(avail - barTotal, rows.length) < minLh) {
        rows = rows.slice(0, -1);
        if (barRow !== null && barRow >= rows.length) {
          barRow = null;
          barTotal = 0;
        }
      }
      if (!rows.length) return;
      const lh = Math.min(maxLh, fdiv(avail - barTotal, rows.length));
      rows.forEach(([lab, val, col], j) => {
        const vimg = this._fit(val, [this.fBtn, this.fTiny], fdiv(w, 2), col);
        const limg = this._fit(lab, [this.fSmall, this.fTiny, this.fMicro], w - vimg.w - 8, ui.TEXT_DIM);
        this._put(ctx, limg, x0, y + fdiv(lh, 2), "midleft");
        this._put(ctx, vimg, x0 + w, y + fdiv(lh, 2), "midright");
        y += lh;
        if (j === barRow) {
          // Balken: grün = sicher, rot = Mine
          const bar = R(x0, y - Math.max(1, fdiv(lh, 8)), w, bh);
          draw.rect(ctx, [110, 30, 40], bar, 0, fdiv(bh, 2));
          const gw = trunc((w * p10) / 1000);
          if (gw > 0) draw.rect(ctx, [40, 170, 120], [bar.x, bar.y, gw, bh], 0, fdiv(bh, 2));
          y += barTotal;
        }
      });
      // Noch im Feld: verbleibende Edelsteine und Minen als Bildchen
      const ih = Math.max(18, trunc(30 * this.k));
      const room = this.ladderLabY - Math.max(4, trunc(6 * this.k)) - y;
      if (this.state === PLAY && room >= ih + Math.max(8, trunc(12 * this.k))) {
        const left = ML.gems(this.grid, this.mines) - this._k();
        this._drawCounts(ctx, x0, y + fdiv(room, 2), left, this.mines, this.fBig, ih, w);
      }
    }

    _lastK() {
      const e = this.fairlog[this.fairlog.length - 1];
      if (e) {
        const o = e.o;
        if (isDict(o) && Array.isArray(o.open)) {
          const n = o.open.length;
          return o.m === 0 ? n - 1 : n;
        }
      }
      return 0;
    }

    /** Leiste der nächsten 5 Stufen (in Ruhe: die ersten 5). */
    _drawLadder(ctx) {
      const [x0, w] = this.panelInner;
      const k = this.state === PLAY ? this._k() : 0;
      const steps = ML.ladder(this.grid, this.mines, k, 5);
      this._put(ctx, this._fit(t("lmi.ladder"), [this.fTiny, this.fMicro], w, ui.TEXT_DIM), x0, this.ladderLabY);
      this.rPills.forEach((r, i) => {
        if (i >= steps.length) {
          draw.rect(ctx, [20, 18, 30], r, 1, fdiv(r.h, 2));
          return;
        }
        const m100 = steps[i][1];
        const nxt = i === 0 && this.state === PLAY;
        const bg = nxt ? CD.COL_GOLD : [26, 24, 40];
        const fg = nxt ? [30, 24, 10] : [232, 232, 240];
        draw.rect(ctx, bg, r, 0, fdiv(r.h, 2));
        if (!nxt) draw.rect(ctx, ui.mix(this.accent, ui.BORDER, 0.6), r, 1, fdiv(r.h, 2));
        const img = this._fit(this._fmtShort(m100), [this.fTiny, this.fMicro], r.w - 4, fg);
        this._put(ctx, img, cxOf(r), cyOf(r), "center");
      });
    }

    _drawHint(ctx) {
      const [x0, w] = this.panelInner;
      let text;
      if (this.auto) text = t("chip.auto_left", { n: this.auto.left });
      else if (this.state === PLAY) text = t("lmi.hint_play");
      else text = t("lmi.hint_idle");
      let y = this.hintY;
      for (const line of this._wrap(text, this.fMicro, w).slice(0, 2)) {
        this._text(ctx, line, x0, y, this.fMicro, ui.TEXT_FAINT);
        y += this.fMicro.px + 1;
      }
    }

    // ----- Auto-Fenster (über der Seitenleiste) -----------------------------------
    _drawAuto(ctx) {
      const p = this.panelR;
      ui.drawPanel(ctx, p, { accentTop: this.accent });
      const [x0, w] = this.panelInner;
      const hot = this.mouse;
      const ac = this.rAutoClose;
      const title = this._fit(t("chip.auto_title"), [this.fBig, this.fBtn], ac.x - x0 - 4, ui.TEXT);
      this._put(ctx, title, x0, cyOf(ac), "midleft");
      this._btn(ctx, ac, "", { hot: ac.collidepoint(hot) });
      D.icon(ctx, "close", inflate(ac, fdiv(-ac.w, 4), fdiv(-ac.h, 4)), ui.TEXT);
      const labels = [t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
      const values = [this.autoRounds, this.autoWin, this.autoLoss];
      this.autoRows.forEach(([ly, rects], row) => {
        this._put(ctx, this._fit(labels[row], [this.fTiny, this.fMicro], w, ui.TEXT_DIM), x0, ly);
        const opts = row === 0 ? ROUND_OPTS : LIMIT_OPTS;
        rects.forEach((r, j) => {
          const v = opts[j];
          const lab = row === 0 ? String(v) : v === 0 ? t("chip.auto_off") : CD.fmtAmount(v);
          this._btn(ctx, r, lab, { active: v === values[row], hot: r.collidepoint(hot), small: true });
        });
      });
      // Markierungen + Gewinn bei Erfolg
      let y = this.autoInfoY;
      const n = this._marks().length;
      const g = ML.gems(this.grid, this.mines);
      const img = this._fit(t("lmi.marked", { n }), [this.fBtn, this.fTiny], w, n ? ui.TEXT : ui.TEXT_DIM);
      this._put(ctx, img, x0, y);
      y += img.h + 1;
      let info, col;
      if (n > 0 && n <= g) {
        info = t("lmi.marked_info", { x: this._fmt(ML.mult100(this.grid, this.mines, n)).slice(1),
          p: this._fmtPct(ML.allSafePercent(this.grid, this.mines, n)) });
        col = CD.COL_GOLD;
      } else if (n > g) {
        info = t("lmi.auto_too_many", { n: g });
        col = ui.RED;
      } else {
        info = t("lmi.hint_auto");
        col = ui.TEXT_DIM;
      }
      const room = this.rAutoStart.y - y - 4;
      for (const line of this._wrap(info, this.fTiny, w).slice(0, Math.max(1, fdiv(room, Math.max(1, this.fTiny.px))))) {
        this._text(ctx, line, x0, y, this.fTiny, col);
        y += this.fTiny.px;
      }
      if (n > 0 && n <= g && y + this.fMicro.px < this.rAutoStart.y - 4) {
        for (const line of this._wrap(t("lmi.hint_auto"), this.fMicro, w)) {
          if (y + this.fMicro.px >= this.rAutoStart.y - 4) break;
          this._text(ctx, line, x0, y, this.fMicro, ui.TEXT_FAINT);
          y += this.fMicro.px;
        }
      }
      // freier Platz: das Maskottchen schaut zu
      const free = this.rAutoStart.y - y - 2 * this.ppad;
      const ms = this.rMascot.w;
      if (free >= ms + 4) {
        const mw = ms + trunc(this.cartW * 1.0);
        this._drawMascot(ctx, R(x0 + fdiv(w - mw, 2), y + this.ppad + fdiv(free - ms, 2), ms, ms));
      }
      this._btn(ctx, this.rAutoClear, t("lmi.clear_marks"), { on: n > 0, hot: this.rAutoClear.collidepoint(hot) });
      this._btn(ctx, this.rAutoStart, t("chip.auto_start"), { primary: true, on: n > 0 && n <= g, hot: this.rAutoStart.collidepoint(hot) });
    }

    // ----- Kopfzeile, Werkzeuge, Verlauf ---------------------------------------------
    _drawHud(ctx) {
      const W = this.width;
      const g = ctx.createLinearGradient(0, 0, 0, this.hudH);
      g.addColorStop(0, "rgba(10,8,16," + (210 / 255).toFixed(3) + ")");
      g.addColorStop(1, "rgba(10,8,16," + (150 / 255).toFixed(3) + ")");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W, this.hudH);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [0, this.hudH - 1], [W, this.hudH - 1], 1);
      const cy = fdiv(this.hudH, 2);
      const pad = this.margin;
      const coinR = Math.max(6, trunc(this.hudH * 0.26));
      const coin = CD.symbolCanvas("coin", coinR * 2 + 4);
      ctx.drawImage(coin, pad - 2, cy - fdiv(coin.lh, 2), coin.lw, coin.lh);
      const leftTxt = t("cas.chips") + ": " + this._group(Math.round(this.shownChips));
      const lw = this.fBig.width(leftTxt);
      this._text(ctx, leftTxt, pad + coinR * 2 + 6, cy, this.fBig, ui.GOLD, "midleft");
      const rightTxt = t("cas.record") + ": " + this._group(this.score) + "   ·   " + t("cas.credits", { n: bank.refills() });
      const rf = this.fSmall.width(rightTxt) > W * 0.42 ? this.fTiny : this.fSmall;
      const rw = rf.width(rightTxt);
      this._text(ctx, rightTxt, W - pad, cy, rf, ui.TEXT_DIM, "midright");
      if (!this.round) return;
      const bet = this.round.bet;
      const k = this._k();
      let mid = t("cas.bet") + ": " + this._group(bet);
      if (k > 0) {
        const m100 = ML.mult100(this.grid, this.mines, k);
        mid += "   ·   " + this._fmt(m100) + " = " + this._group(this._winAt(bet, k));
      }
      const x0 = pad + coinR * 2 + 6 + lw + pad;
      const x1 = W - pad - rw - pad;
      const f = this.fSmall.width(mid) > x1 - x0 ? this.fTiny : this.fSmall;
      if (f.width(mid) <= x1 - x0) this._text(ctx, mid, fdiv(x0 + x1, 2), cy, f, ui.TEXT, "center");
    }

    _toolActive(name) {
      if (name === "turbo") return this.turbo;
      if (name === "auto") return this.window === "auto" || this.auto !== null;
      return this.window === name;
    }

    _drawTools(ctx) {
      const hot = this.mouse;
      for (const [name, r] of this.rTools) {
        const active = this._toolActive(name);
        const h = r.collidepoint(hot);
        const rad = Math.max(5, fdiv(r.h, 4));
        draw.rect(ctx, active || h ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        let edge = active ? this.accent : h ? ui.BORDER_LIGHT : ui.BORDER;
        if (name === "auto" && this.auto) edge = ui.mix(this.accent, [255, 255, 255], 0.5 * (0.5 + 0.5 * Math.sin(this.time * 5)));
        draw.rect(ctx, edge, r, active ? 2 : 1, rad);
        const col = active ? this.accent : ui.TEXT;
        D.icon(ctx, name, inflate(R(r.x, r.y, r.h, r.h), fdiv(-r.h, 3), fdiv(-r.h, 3)), col);
        if (this.toolLabels) {
          this._text(ctx, t(TOOL_LABEL[name]), r.x + r.h - Math.max(2, fdiv(r.h, 8)), cyOf(r), this.fTiny, active || h ? ui.TEXT : ui.TEXT_DIM, "midleft");
        }
      }
    }

    _drawTooltip(ctx) {
      for (const [name, r] of this.rTools) {
        if (r.collidepoint(this.mouse)) {
          const txt = t(TOOL_LABEL[name]) + " (" + TOOL_KEY[name] + ")";
          const w = this.fTiny.width(txt) + 12, h = this.fTiny.height + 6;
          const box = R(r.x, r.bottom + 4, w, h);
          draw.rect(ctx, [12, 10, 22], box, 0, fdiv(h, 2));
          draw.rect(ctx, ui.BORDER_LIGHT, box, 1, fdiv(h, 2));
          this._text(ctx, txt, cxOf(box), cyOf(box), this.fTiny, ui.TEXT, "center");
        }
      }
    }

    /** Letzte Runden als Schilder oben rechts (neueste rechts). */
    _drawHistory(ctx) {
      let x = this.width - this.margin;
      const y = this.rTools[0][1].y;
      const h = this.toolH;
      const limit = this.toolsRight + this.margin;
      for (let j = this.hist.length - 1; j >= 0; j--) {
        const m100 = this.hist[j];
        const txt = m100 > 0 ? this._fmtShort(m100) : "×0";
        const bw = this.fTiny.width(txt) + trunc(12 * this.k) + 4;
        const bh = trunc(h * 0.78);
        const box = R(x - bw, y + fdiv(h - bh, 2), bw, bh);
        if (box.x < limit) break;
        const col = m100 >= 200 ? [34, 132, 100] : m100 > 100 ? [62, 96, 82] : m100 > 0 ? [70, 72, 92] : [128, 36, 46];
        draw.rect(ctx, col, box, 0, fdiv(bh, 2));
        this._text(ctx, txt, cxOf(box), cyOf(box), this.fTiny, [240, 240, 245], "center");
        x = box.x - 4;
      }
    }

    // ----- Bedienleiste -----------------------------------------------------------
    _drawStrip(ctx) {
      const st = this.strip;
      const g = ctx.createLinearGradient(0, st.y, 0, st.bottom);
      g.addColorStop(0, "rgba(10,8,16," + (170 / 255).toFixed(3) + ")");
      g.addColorStop(1, "rgba(10,8,16," + (230 / 255).toFixed(3) + ")");
      ctx.fillStyle = g;
      ctx.fillRect(st.x, st.y, st.w, st.h);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [st.x, st.y], [st.right, st.y], 2);
      const idle = this.state === IDLE && this.auto === null;
      const bet = this._roundBet();
      const hot = this.mouse;
      this._btn(ctx, this.rMinus, "–", { on: idle && this.bet > ML.BETS[0], hot: this.rMinus.collidepoint(hot) });
      this._btn(ctx, this.rPlus, "+", { on: idle && this.bet < ML.BETS[ML.BETS.length - 1], hot: this.rPlus.collidepoint(hot) });
      const rad = Math.max(6, fdiv(this.rBet.h, 4));
      draw.rect(ctx, [16, 12, 26], this.rBet, 0, rad);
      draw.rect(ctx, ui.BORDER, this.rBet, 1, rad);
      this._text(ctx, t("cas.bet"), cxOf(this.rBet), this.rBet.y + 2, this.fTiny, ui.TEXT_DIM, "midtop");
      const col = idle && bet > bank.balance() ? ui.RED : ui.GOLD;
      this._put(ctx, this._fit(this._group(bet), [this.fBtn, this.fTiny], this.rBet.w - 4, col), cxOf(this.rBet), this.rBet.bottom - 3, "midbottom");
      const play = this.state === PLAY;
      if (this.auto) {
        // Mitte: Fortschritt, rechts: STOPP
        const txt = t("chip.auto_left", { n: this.auto.left });
        const img = this._fit(txt, [this.fBtn, this.fTiny], this.rCash.x - this.rPlus.right - 2 * this.margin, this.accent);
        this._put(ctx, img, fdiv(this.rPlus.right + this.rCash.x, 2), cyOf(this.rGo), "center");
        this._btn(ctx, this.rGo, t("chip.auto_stop"), { primary: true, hot: this.rGo.collidepoint(hot), danger: true });
      } else {
        const ready = idle || ((this.state === BOOM || this.state === CASHED) && this._resultVisible() && this.resT >= READ_T);
        this._btn(ctx, this.rGo, t("lmi.play"), { primary: true, on: ready && !this.broke && this.window === null, hot: this.rGo.collidepoint(hot) });
        if (play) {
          const on = this.window === null;
          this._btn(ctx, this.rRandom, this.randomLabel ? t("lmi.random") : "", { on, hot: this.rRandom.collidepoint(hot), icon: "dice" });
        }
      }
      let cash = t("lmi.cashout");
      const can = play && this._k() > 0 && this.auto === null;
      if (can) cash += " " + this._group(this._winAt(this.round.bet, this._k()));
      this._btn(ctx, this.rCash, cash, { on: can, hot: this.rCash.collidepoint(hot), active: can });
    }

    _btn(ctx, r, label, { primary = false, on = true, hot = false, active = false, small = false, icon = null, danger = false } = {}) {
      const rad = Math.max(5, fdiv(r.h, 4));
      let col;
      if (primary && on) {
        const base = ui.mix(danger ? ui.RED : this.accent, [0, 0, 0], danger ? 0.1 : 0.3);
        const fillc = ui.mix(base, [255, 255, 255], hot ? 0.12 : 0);
        draw.rect(ctx, ui.mix(base, [0, 0, 0], 0.4), r.move(0, 2), 0, rad);
        draw.rect(ctx, fillc, r, 0, rad);
        const pulse = 0.5 + 0.5 * Math.sin(this.time * 2.4);
        draw.rect(ctx, ui.mix(CD.COL_GOLD, [255, 255, 255], 0.35 * pulse), r, 2, rad);
        col = [255, 255, 255];
      } else {
        draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        col = on ? ui.TEXT : ui.TEXT_FAINT;
      }
      let tx = cxOf(r);
      if (icon) {
        const isz = trunc(r.h * 0.62);
        if (label) {
          const lw = this.fBtn.width(label);
          const ix = cxOf(r) - fdiv(isz + 6 + lw, 2);
          D.icon(ctx, icon, R(ix, cyOf(r) - fdiv(isz, 2), isz, isz), col);
          tx = ix + isz + 6 + fdiv(lw, 2);
        } else {
          D.icon(ctx, icon, R(cxOf(r) - fdiv(isz, 2), cyOf(r) - fdiv(isz, 2), isz, isz), col);
        }
      }
      if (label) {
        const fonts = small ? [this.fTiny, this.fMicro] : [this.fBtn, this.fTiny, this.fMicro];
        this._put(ctx, this._fit(label, fonts, r.w - 6, col), tx, cyOf(r), "center");
      }
    }

    // ----- Effekte, Ergebnis, Hinweise ---------------------------------------------
    _drawFx(ctx) {
      for (const c of this.coinsFx) {
        const w = Math.max(2, trunc(c.r * 2 * Math.abs(Math.cos(c.spin))));
        const h = trunc(c.r * 2);
        const x0 = trunc(c.x) - fdiv(w, 2), y0 = trunc(c.y) - fdiv(h, 2);
        draw.ellipse(ctx, CD.COL_GOLD_D, [x0 + 1, y0 + 1, w, h]);
        draw.ellipse(ctx, [246, 196, 60], [x0, y0, w, h]);
        if (w > 6) {
          // rect.inflate(-w // 2, -h // 2)
          const dx = fdiv(-w, 2), dy = fdiv(-h, 2);
          draw.ellipse(ctx, [255, 236, 150], [x0 - trunc(dx / 2), y0 - trunc(dy / 2), w + dx, h + dy]);
        }
      }
      for (const p of this.popups) {
        const f = p.t / p.dur;
        const alpha = f > 0.6 ? trunc(255 * (1 - (f - 0.6) / 0.4)) / 255 : 1;
        const y = p.y - 40 * this.k * (1 - (1 - f) * (1 - f));
        this._text(ctx, p.text, trunc(p.x), trunc(y), this.fBig, p.color, "center", alpha);
      }
    }

    _drawResult(ctx) {
      const [title, sub, col] = this.result;
      const br = this.boardR, k = this.k;
      const maxw = br.w - trunc(24 * k);
      let fonts = [this.fHuge, this.fMult, this.fBig, this.fBtn];
      if (this.fHuge.width(title) > br.w * 0.62) fonts = fonts.slice(1);
      const timg = this._fit(title, fonts, maxw, col);
      const simg = this._fit(sub, [this.fSmall, this.fTiny, this.fMicro], maxw, ui.TEXT);
      const pw = Math.min(br.w - 8, Math.max(timg.w, simg.w) + trunc(40 * k));
      const ph = timg.h + simg.h + trunc(30 * k);
      let panel = R(cxOf(br) - fdiv(pw, 2), cyOf(br) - fdiv(ph, 2), pw, ph);
      const f = Math.min(1, this.resT / 0.18);
      if (f < 1) panel = inflate(panel, -trunc(pw * 0.15 * (1 - f)), -trunc(ph * 0.15 * (1 - f)));
      ui.drawPanel(ctx, panel, { accentTop: col });
      this._put(ctx, timg, cxOf(panel), panel.y + trunc(10 * k), "midtop");
      this._put(ctx, simg, cxOf(panel), panel.bottom - trunc(10 * k), "midbottom");
    }

    _drawBanner(ctx) {
      const [text, left] = this.banner;
      const br = this.boardR;
      const pulse = 0.5 + 0.5 * Math.sin(this.time * 8);
      const col = ui.mix(CD.COL_GOLD, [255, 255, 255], 0.4 * pulse);
      const fonts = [this.fHuge, this.fMult, this.fBig];
      let font = fonts[fonts.length - 1];
      for (const f of fonts) {
        if (f.width(text) <= br.w - 16) {
          font = f;
          break;
        }
      }
      const img = this._fit(text, [font], br.w - 16, col);
      const sh = this._fit(text, [font], br.w - 16, [40, 20, 0]);
      const y = br.y + trunc(br.h * 0.2);
      const alpha = left < 0.4 ? trunc((255 * left) / 0.4) / 255 : undefined;
      this._put(ctx, sh, cxOf(br) + 3, y + 3, "center", alpha);
      this._put(ctx, img, cxOf(br), y, "center", alpha);
    }

    _drawToast(ctx) {
      const [text, left] = this.toast;
      const img = this._fit(text, [this.fSmall, this.fTiny], this.width - 40, ui.TEXT);
      const bw = img.w + 24, bh = img.h + 12;
      const box = R(cxOf(this.boardR) - fdiv(bw, 2), this.boardR.y + this.margin, bw, bh);
      if (box.x < 4) box.x = 4;
      ctx.save();
      if (left < 0.3) ctx.globalAlpha *= trunc((255 * left) / 0.3) / 255;
      draw.rect(ctx, [12, 8, 20, 225], box, 0, fdiv(bh, 2));
      draw.rect(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.4), box, 1, fdiv(bh, 2));
      this._put(ctx, img, cxOf(box), cyOf(box), "center");
      ctx.restore();
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const cx = fdiv(this.width, 2), cy = fdiv(this.height, 2);
      const pw = Math.min(this.width - 40, trunc(480 * Math.max(0.8, this.k)));
      const ph = trunc(200 * Math.max(0.75, this.k));
      const panel = R(cx - fdiv(pw, 2), cy - fdiv(ph, 2), pw, ph);
      ui.drawPanel(ctx, panel, { accentTop: ui.RED });
      const head = this._fit(t("cas.broke"), [this.fHuge, this.fBig], pw - 16, ui.RED);
      this._put(ctx, head, cx, trunc(panel.y + ph * 0.28), "center");
      [[t("cas.broke_sub"), ui.TEXT_DIM], [t("cas.broke_restart", { n: bank.START_CHIPS }), ui.TEXT]].forEach(([txt, col], i) => {
        const img = this._fit(txt, [this.fSmall, this.fTiny, this.fMicro], pw - 16, col);
        this._put(ctx, img, cx, trunc(panel.y + ph * (0.58 + 0.2 * i)), "center");
      });
    }

    // ----- Statistik- und Fairness-Fenster ---------------------------------------------
    _windowRect() {
      const W = this.width, H = this.height, k = this.k, m = this.margin;
      let w, h;
      if (this.window === "fair") {
        w = Math.min(W - 2 * m, trunc(720 * k));
        h = Math.min(H - 2 * m, trunc(540 * k));
      } else {
        w = Math.min(W - 2 * m, trunc(640 * k));
        h = Math.min(H - 2 * m, trunc(470 * k));
      }
      const rect = R(fdiv(W, 2) - fdiv(w, 2), fdiv(H, 2) - fdiv(h, 2), w, h);
      const cb = Math.max(20, trunc(28 * k));
      const close = R(rect.right - this.ppad - cb, rect.y + this.ppad, cb, cb);
      return [rect, close];
    }

    _drawWindow(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const [rect, close] = this._windowRect();
      ui.drawPanel(ctx, rect, { accentTop: this.accent });
      const pad = this.ppad;
      const icon = R(rect.x + pad, close.y, close.h, close.h);
      D.icon(ctx, this.window, inflate(icon, fdiv(-icon.w, 6), fdiv(-icon.h, 6)), this.accent);
      const title = (this.window === "stats" ? t("chip.stats") : t("chip.fair_title")) + "  ·  Lama Mines";
      const img = this._fit(title, [this.fBig, this.fBtn], close.x - icon.right - 2 * pad, ui.TEXT);
      this._put(ctx, img, icon.right + fdiv(pad, 2), cyOf(close), "midleft");
      this._btn(ctx, close, "", { hot: close.collidepoint(this.mouse) });
      D.icon(ctx, "close", inflate(close, fdiv(-close.w, 4), fdiv(-close.h, 4)), ui.TEXT);
      const body = R(rect.x + pad, close.bottom + pad, rect.w - 2 * pad, rect.bottom - close.bottom - 2 * pad);
      if (this.window === "stats") this._drawStats(ctx, body);
      else this._drawFair(ctx, body);
    }

    _drawStats(ctx, body) {
      const st = this.stats, k = this.k, sep = this._sep();
      let rtp = "–";
      if (st.wagered > 0) {
        const p10 = fdiv(st.paid * 2000 + st.wagered, st.wagered * 2);
        rtp = ML.fmtPercent(p10, sep) + " %";
      }
      const net = st.net;
      const tiles = [
        [t("chip.st_rounds"), this._group(st.rounds), ui.TEXT],
        [t("chip.st_wins"), this._group(st.wins), ui.GREEN],
        [t("chip.st_losses"), this._group(st.losses), ui.RED],
        [t("chip.st_rtp"), rtp, ui.TEXT],
        [t("chip.st_wagered"), this._group(st.wagered), ui.TEXT],
        [t("chip.st_paid"), this._group(st.paid), ui.TEXT],
        [t("chip.st_net"), (net > 0 ? "+" : "") + this._group(net), net >= 0 ? ui.GREEN : ui.RED],
        [t("chip.st_best_win"), (st.best_win > 0 ? "+" : "") + this._group(st.best_win), CD.COL_GOLD],
        [t("chip.st_best_mult"), st.best_mult > 0 ? this._fmt(st.best_mult) : "–", CD.COL_GOLD],
        [t("lmi.st_gems"), this._group(this._xint("gems")), this.accent],
        [t("lmi.st_booms"), this._group(this._xint("booms")), ui.RED],
        [t("lmi.st_clears"), this._group(this._xint("clears")), CD.COL_GOLD],
      ];
      const cols = 4;
      const gap = Math.max(4, trunc(8 * k));
      const tw = fdiv(body.w - (cols - 1) * gap, cols);
      const th = this.fTiny.px + this.fBig.px + Math.max(8, trunc(14 * k));
      const rad = Math.max(4, trunc(8 * k));
      tiles.forEach(([lab, val, col], i) => {
        const r = R(body.x + (i % cols) * (tw + gap), body.y + fdiv(i, cols) * (th + gap), tw, th);
        draw.rect(ctx, ui.mix(ui.PANEL, [0, 0, 0], 0.25), r, 0, rad);
        draw.rect(ctx, ui.BORDER, r, 1, rad);
        this._put(ctx, this._fit(lab, [this.fTiny, this.fMicro], tw - 8, ui.TEXT_DIM), r.x + 5, r.y + Math.max(3, trunc(5 * k)));
        this._put(ctx, this._fit(val, [this.fBig, this.fBtn, this.fTiny], tw - 8, col), r.x + 5, r.bottom - Math.max(3, trunc(5 * k)), "bottomleft");
      });
      let y = body.y + 3 * (th + gap) + gap;
      const curve = st.curve;
      const lab = curve.length ? t("chip.st_curve", { n: curve.length }) : t("chip.st_empty");
      const img = this._fit(lab, [this.fTiny, this.fMicro], body.w, ui.TEXT_DIM);
      this._put(ctx, img, body.x, y);
      y += img.h + 4 + ui.font(10).px; // Platz für die obere Achsen-Zahl
      const chart = R(body.x, y, body.w, body.bottom - y);
      if (curve.length >= 2 && chart.h > 20) {
        const vals = curve.length < CS.CURVE ? [0].concat(curve) : curve.slice();
        PG.charts.line(ctx, chart, vals, { color: this.accent, fmt: (v) => this._group(trunc(v)) });
        // Null-Linie
        const lo = Math.min(...vals), hi = Math.max(...vals);
        if (lo < 0 && 0 < hi) {
          const zy = trunc(chart.bottom - ((0 - lo) / (hi - lo)) * chart.h);
          draw.line(ctx, ui.mix(ui.RED, ui.PANEL, 0.4), [chart.x, zy], [chart.right, zy], 1);
        }
      } else if (chart.h > 20) {
        const ll = D.minerLlama(Math.max(16, Math.min(chart.h - 8, trunc(90 * k))), "stand", this.accent);
        D.blitCenter(ctx, ll, cxOf(chart), cyOf(chart));
      }
    }

    /** Prüfcode des nächsten Server-Seeds (gemerkt, nicht jedes Bild neu gerechnet). */
    _nextCommit() {
      if (this._commitMemo[0] !== this.fair.server) this._commitMemo = [this.fair.server, F.commit(this.fair.server)];
      return this._commitMemo[1];
    }

    _drawFair(ctx, body) {
      const k = this.k;
      const gap = Math.max(4, trunc(7 * k));
      let y = body.y;
      for (const line of this._wrap(t("chip.fair_info"), this.fMicro, body.w).slice(0, 3)) {
        this._text(ctx, line, body.x, y, this.fMicro, ui.TEXT_DIM);
        y += this.fMicro.px;
      }
      y += gap;
      // Nächster Prüfcode (+ laufende Runde)
      const rows = [];
      if (this.round) rows.push([t("lmi.fair_current"), this.round.h, ui.TEXT]);
      rows.push([t("chip.fair_next"), this._nextCommit(), this.accent]);
      for (const [lab, hx, col] of rows) {
        const img = this._fit(lab, [this.fTiny, this.fMicro], body.w, ui.TEXT_DIM);
        this._put(ctx, img, body.x, y);
        y += img.h;
        y = this._drawHex(ctx, hx, body.x, y, body.w, col);
        y += 2;
      }
      // Client-Seed + Knopf
      const bh = Math.max(22, trunc(28 * k));
      const lab = t("chip.fair_client") + ": " + this.fair.client + "   ·   " + t("chip.fair_nonce") + " " + this.fair.nonce;
      let bw = this.fBtn.width(t("chip.fair_new_client")) + trunc(20 * k);
      bw = Math.min(bw, fdiv(body.w, 2));
      this.rFairNew = R(body.right - bw, y, bw, bh);
      this._put(ctx, this._fit(lab, [this.fTiny, this.fMicro], body.w - bw - gap, ui.TEXT), body.x, y + fdiv(bh, 2), "midleft");
      this._btn(ctx, this.rFairNew, t("chip.fair_new_client"), { on: this.round === null, hot: this.rFairNew.collidepoint(this.mouse) });
      y += bh + gap;
      draw.line(ctx, ui.BORDER, [body.x, y], [body.right, y], 1);
      y += gap;
      // Liste links, Einzelheiten rechts
      const area = R(body.x, y, body.w, body.bottom - y - this.fMicro.px - 2);
      const note = this._fit(t("chip.fair_note"), [this.fMicro], body.w, ui.TEXT_FAINT);
      this._put(ctx, note, body.x, body.bottom - note.h);
      this.rFairRows = [];
      this.rFairVerify = R(0, 0, 0, 0);
      if (!this.fairlog.length) {
        this._put(ctx, this._fit(t("chip.fair_empty"), [this.fSmall, this.fTiny], area.w, ui.TEXT_DIM), cxOf(area), cyOf(area), "center");
        return;
      }
      const lw = Math.max(trunc(area.w * 0.4), Math.min(fdiv(area.w, 2), trunc(220 * k)));
      const lst = R(area.x, area.y, lw, area.h);
      const det = R(lst.right + gap, area.y, area.w - lw - gap, area.h);
      const rh = Math.max(this.fTiny.px + 6, trunc(24 * k));
      const nvis = Math.max(1, fdiv(lst.h, rh));
      const order = [];
      for (let i = this.fairlog.length - 1; i >= 0; i--) order.push(i); // neueste oben
      this.fairScroll = Math.max(0, Math.min(this.fairScroll, order.length - nvis));
      order.slice(this.fairScroll, this.fairScroll + nvis).forEach((idx, row) => {
        const e = this.fairlog[idx];
        const r = R(lst.x, lst.y + row * rh, lst.w, rh - 2);
        const sel = idx === this.fairSel;
        const rr = Math.max(3, fdiv(rh, 4));
        draw.rect(ctx, sel || r.collidepoint(this.mouse) ? ui.BTN_SEL : ui.mix(ui.PANEL, [0, 0, 0], 0.2), r, 0, rr);
        if (sel) draw.rect(ctx, this.accent, r, 1, rr);
        const o = isDict(e.o) ? e.o : {};
        const m = o.m;
        const res = isInt(m) ? (m ? this._fmtShort(m) : "×0") : "?";
        const num = "#" + e.n;
        const numW = this.fTiny.width(num);
        this._text(ctx, num, r.x + 5, cyOf(r), this.fTiny, ui.TEXT_DIM, "midleft");
        const resW = this.fTiny.width(res);
        this._text(ctx, res, r.right - 5, cyOf(r), this.fTiny, m ? ui.GREEN : ui.RED, "midright");
        const hx0 = r.x + 5 + Math.max(numW, this.fTiny.width("#0000")) + 6;
        const hx1 = r.right - 5 - Math.max(resW, this.fTiny.width("×00,00")) - 6;
        if (hx1 - hx0 > 20) this._put(ctx, this._fit(F.short(e.h, 10), [this.fMono], hx1 - hx0, ui.TEXT_FAINT), hx0, cyOf(r), "midleft");
        const chk = this.fairChecks.get(e.n);
        if (chk) {
          const ir = R(r.right - resW - 10 - fdiv(rh, 2), r.y + fdiv(rh, 4), fdiv(rh, 2), fdiv(rh, 2));
          D.icon(ctx, chk[0] ? "check" : "cross", ir, chk[0] ? ui.GREEN : ui.RED);
        }
        this.rFairRows.push([idx, r]);
      });
      if (order.length > nvis) {
        // Bildlauf-Leiste
        const sb = R(lst.right - 3, lst.y, 3, lst.h);
        draw.rect(ctx, ui.BORDER, sb);
        const hh = Math.max(8, fdiv(sb.h * nvis, order.length));
        const hy = sb.y + fdiv((sb.h - hh) * this.fairScroll, Math.max(1, order.length - nvis));
        draw.rect(ctx, this.accent, [sb.x, hy, 3, hh]);
      }
      if (this.fairSel === null || !(this.fairSel >= 0 && this.fairSel < this.fairlog.length)) return;
      this._drawFairDetail(ctx, det, this.fairlog[this.fairSel]);
    }

    /** Hex-Text in Monospace, bei Bedarf auf mehrere Zeilen umbrochen. */
    _drawHex(ctx, hx, x, y, w, col) {
      const cw = Math.max(1, this.fMono.width("0"));
      const per = Math.max(8, fdiv(w, cw));
      for (let i = 0; i < hx.length; i += per) {
        this._text(ctx, hx.slice(i, i + per), x, y, this.fMono, col);
        y += this.fMono.px;
      }
      return y;
    }

    _drawFairDetail(ctx, det, e) {
      const k = this.k;
      const x = det.x, w = det.w;
      let y = det.y;
      const o = isDict(e.o) ? e.o : {};
      let head = "#" + e.n;
      if (ML.GRIDS.includes(o.grid) && isInt(o.mines)) head += "  ·  " + t("lmi.fair_round", { g: o.grid, m: o.mines });
      let img = this._fit(head, [this.fBtn, this.fTiny], w, ui.TEXT);
      this._put(ctx, img, x, y);
      y += img.h + 2;
      for (const [lab, val] of [[t("chip.fair_hash"), e.h], [t("chip.fair_server"), e.s]]) {
        img = this._fit(lab, [this.fMicro], w, ui.TEXT_DIM);
        this._put(ctx, img, x, y);
        y += img.h;
        y = this._drawHex(ctx, val, x, y, w, ui.TEXT);
      }
      img = this._fit(t("chip.fair_client") + ": " + e.c, [this.fTiny, this.fMicro], w, ui.TEXT);
      this._put(ctx, img, x, y + 1);
      y += img.h + 4;
      const bh = Math.max(22, trunc(28 * k));
      const bw = Math.min(w, this.fBtn.width(t("chip.fair_verify")) + trunc(24 * k));
      this.rFairVerify = R(x, y, bw, bh);
      this._btn(ctx, this.rFairVerify, t("chip.fair_verify"), { primary: true, hot: this.rFairVerify.collidepoint(this.mouse) });
      const chk = this.fairChecks.get(e.n);
      if (chk) {
        const [ok, field] = chk;
        const ir = R(this.rFairVerify.right + 6, y + fdiv(bh, 6), fdiv(bh * 2, 3), fdiv(bh * 2, 3));
        D.icon(ctx, ok ? "check" : "cross", ir, ok ? ui.GREEN : ui.RED);
        img = this._fit(ok ? t("chip.fair_ok") : t("chip.fair_bad"), [this.fBtn, this.fTiny], Math.max(10, det.right - ir.right - 6), ok ? ui.GREEN : ui.RED);
        this._put(ctx, img, ir.right + 4, y + fdiv(bh, 2), "midleft");
        y += bh + 6;
        if (field) this._drawMini(ctx, R(x, y, w, det.bottom - y), o, field);
      }
    }

    /** Mini-Vorschau des nachgerechneten Feldes: wie das echte Brett, nur klein -
     *  aufgedeckte Plätze hell, der Rest abgedunkelt, die getroffene Mine rot. */
    _drawMini(ctx, area, o, field) {
      const g = o.grid;
      const side = Math.min(area.w, area.h);
      if (side < g * 6) return;
      const gap = Math.max(1, fdiv(side, 80));
      const cs = fdiv(side - gap * (g - 1), g);
      const opened = Array.isArray(o.open) ? o.open : [];
      const mines = new Array(g * g).fill(false);
      for (const v of field) mines[v] = true;
      const hit = o.m === 0 && opened.length ? opened[opened.length - 1] : null;
      for (let i = 0; i < g * g; i++) {
        const x = area.x + (i % g) * (cs + gap);
        const y = area.y + fdiv(i, g) * (cs + gap);
        const kind = i === hit ? "boom" : mines[i] ? "mine" : "gem";
        ctx.drawImage(D.face(cs, kind, !opened.includes(i)), x, y, cs, cs);
      }
    }
  }

  PG.register(LamaMinesGame, {
    id: "LamaMinesGame",
    key: KEY,
    name: "Lama Mines",
    settingsKey: KEY,
    defaults: { grid: 5, mines: 3, bet: 10, turbo: false, auto_rounds: 10, auto_win: 0, auto_loss: 0 },
  });
})();
