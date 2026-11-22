/*
 * lamacrash.js - Lama Crash (Port von games/lamacrash.py)
 * ========================================================
 * Mut-Spiel mit Lama-Chips: Ein Lama (bzw. zwei - je Einsatz eins) steigt im
 * Ballon auf, der Multiplikator wächst mit der Zeit. Wer aussteigt, bekommt
 * Einsatz × Multiplikator; platzt der Ballon vorher, ist der noch fliegende
 * Einsatz weg. Nach dem Aussteigen fliegt der leere Ballon weiter, bis er platzt
 * (Vorspulen möglich) - so sieht man, wie weit es gegangen wäre.
 *
 * Regeln und Zahlen stehen in lamacrash_logic.js (bitgleich mit Python und Rust,
 * geprüft von tests/audit_chiplogic.py), die Bilder in lamacrash_draw.js (Ballon,
 * Korb, Fallschirm …) und lamacrash_world.js (Höhenwelten), die Musik in
 * lamacrash_music.js.
 *
 * - Zwei Einsätze (Karte 2 per Schalter/Z), je mit eigenem Auto-Ziel; jeder
 *   Einsatz darf einmal halbiert werden (Hälfte steigt aus, Rest fliegt).
 * - Höhenwelten: Wiese → Wolken (×1,5) → Gewitter (×3) → Nordlicht (×10) →
 *   Weltall (×100) → Mondlandung bei ×100.000 (Höchstgewinn).
 * - Fairness (Fenster F): Prüfcode vorab, Platzpunkt aus Server-/Client-Seed und
 *   Rundennummer, nach der Runde nachrechenbar (Mini-Kurve).
 * - Statistik (S), Auto-Modus (A), Turbo (T), Musik (M).
 *
 * Lama-Chips: gemeinsames Konto der Lama-Bank (casino_bank.js). SPIELEN bucht den
 * Einsatz und speichert die Runde in einem Zug; jeder Ausstieg schreibt den
 * Gewinn gut und speichert die Runde. Wer mitten im Flug geht (destroy, Seite
 * schließen), macht beim nächsten Start am selben Takt weiter. Der Platzpunkt
 * (this._crash) wird nur in _advance (über L.tick) und beim Rundenabschluss
 * gelesen - nichts auf dem Bildschirm oder im Ton verrät ihn vorher.
 * Spielstand (lamabank-Extra "lamacrash") und Einstellungen (Abschnitt
 * "lamacrash") im selben Format wie die Desktop-Version. Highscore =
 * Höchststand von 1000 + Lama-Crash-Bilanz; gameOver wird nie gesetzt.
 *
 * Takt wie im Original: acc += dt * 60 * Tempo; n = int(acc + 1e-9) ganze Takte
 * je Bild (dt auf 0,1 s begrenzt - ein Ruckler wird nicht nachgeholt).
 * Fokus weg, während Geld fliegt (get pauseOnBlur), pausiert app.js das Spiel.
 *
 * Steuerung (alle Tasten roh, nie über die WASD-Belegung): vor der Runde
 * Leertaste/Enter = SPIELEN, 1/2/Tab = Karte wählen, ↑/↓/+/-/Mausrad = Einsatz,
 * ←/→ = Auto-Ziel, X = Auto-Ziel an/aus, Z = 2. Einsatz; im Flug
 * Leertaste/Enter/C = alle aussteigen, 1/2 = einzeln, H = Hälfte (alle), Q/W =
 * Hälfte einzeln; danach Leertaste = Vorspulen. A/S/F/T/M wie oben, Esc schließt
 * ein Fenster bzw. stoppt Auto (sonst Pause).
 */
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  const L = PG.lamaCrashLogic;
  const MU = PG.lamaCrashMusic;
  const D = PG.lamaCrashDraw;
  const W = PG.lamaCrashWorld;
  const CD = PG.casinoDraw;
  const F = PG.fairness;
  const CS = PG.chipstats;
  const bank = PG.lamabank;
  const TAU = Math.PI * 2;
  const trunc = Math.trunc;
  const fdiv = (a, b) => Math.floor(a / b);

  const KEY = L.KEY;
  const IDLE = "idle", COUNT = "count", FLY = "fly", COAST = "coast", END = "end";

  const COUNT_T = 1.35; // 3-2-1 vor dem Start
  const RESUME_T = 1.8; // Countdown beim Fortsetzen einer Runde
  const END_T = 2.8; // Ergebnis nach dem Platzen, danach Ruhezustand
  const READ_T = 0.35; // Ergebnis frühestens danach wegdrückbar
  const AUTO_END_T = 1.0; // Auto: Ergebnis-Anzeige
  const AUTO_PAUSE_T = 0.35; // Auto: Pause vor der nächsten Runde
  const TURBO_F = 0.4; // Turbo/Auto: Countdown und Pausen x 0,4
  const SHAKE_T = 0.6; // Kamera-Wackeln nach dem Platzen
  const BIG_MULT = 300; // ab x3: Münzregen
  const PARTY_MULT = 1000; // ab x10: Konfetti
  const BANNER_MULT = 10000; // ab x100: Gewinn-Banner

  const ROUND_OPTS = [5, 10, 25, 50, 100, 250];
  const LIMIT_OPTS = [0, 100, 500, 1000, 5000, 10000, 50000];
  const AUTO_OPTS = [ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS];
  const AUTO_KEYS = ["auto_rounds", "auto_win", "auto_loss"]; // Einstellung
  const AUTO_FIELDS = ["autoRounds", "autoWin", "autoLoss"]; // Feld im Spiel
  const TOOLS = ["auto", "stats", "fair", "turbo", "music"];
  const TOOL_LABEL = { auto: "chip.auto", stats: "chip.stats", fair: "chip.fair", turbo: "chip.turbo", music: "lcx.music" };
  const TOOL_KEY = { auto: "A", stats: "S", fair: "F", turbo: "T", music: "M" };
  // Halstuch-/Kartenfarbe und Fallschirm je Einsatz
  const CARD_COL = [[255, 196, 72], [96, 198, 255]];
  const CHUTE_COL = [[255, 140, 60], [70, 150, 255]];
  // beim Erreichen dieser Multiplikatoren ein kurzer Ton (steigend)
  const TONE_AT = [200, 300, 500, 1000, 2000, 5000, 10000, 100000, 1000000];
  // = settings.py DEFAULTS["lamacrash"]
  const DEFAULTS = { bet1: 10, bet2: 10, two: false, auto1: false, target1: 200, auto2: true, target2: 1000,
    turbo: false, music: true, auto_rounds: 10, auto_win: 0, auto_loss: 0 };

  const isInt = (v) => typeof v === "number" && Number.isInteger(v);
  const isDict = (v) => !!v && typeof v === "object" && !Array.isArray(v);
  /** Ganzzahliges Rechteck (wie pygame.Rect). */
  const R = (x, y, w, h) => new PG.Rect(trunc(x), trunc(y), trunc(w), trunc(h));
  /** pygame-Mitte (ganzzahlig) eines Rechtecks. */
  const cxOf = (r) => r.x + fdiv(r.w, 2);
  const cyOf = (r) => r.y + fdiv(r.h, 2);
  /** pygame Rect.inflate (C-Ganzzahldivision). */
  const inflate = (r, dx, dy) => new PG.Rect(r.x - trunc(dx / 2), r.y - trunc(dy / 2), r.w + dx, r.h + dy);
  /** Python round(): x.5 zur geraden Zahl. */
  function pyRound(v) {
    const f = Math.floor(v);
    const d = v - f;
    if (d > 0.5) return f + 1;
    if (d < 0.5) return f;
    return f % 2 === 0 ? f : f + 1;
  }
  /** Leertaste/Enter. Alle Tasten werden ROH ausgewertet (wie Lama Mines) -
   *  nie über isAction/WASD, sonst hätten A und S eine Doppelbedeutung. */
  const isAction = (key) => key === "space" || key === "Return" || key === "KP_Enter";
  /** Gleicher Fairness-Zustand? (Python vergleicht dicts, unabhängig von der Reihenfolge) */
  function sameFair(raw, st) {
    return isDict(raw) && Object.keys(raw).length === 3 && raw.server === st.server
      && raw.client === st.client && raw.nonce === st.nonce;
  }

  class LamaCrashGame extends PG.Game {
    /** Esc schließt ein offenes Fenster bzw. stoppt den Auto-Modus. */
    get wantsEscape() {
      return this.window !== null || this.auto !== null;
    }
    /** Fokus weg, während Geld fliegt -> Pause (der Ballon stiege sonst weiter). */
    get pauseOnBlur() {
      return this.round !== null && (this.state === COUNT || this.state === FLY) && L.flying(this.round);
    }
    /** Kein Game Over - also auch kein Highscore-Banner. */
    get showHighscoreBanner() {
      return false;
    }

    // ===================================================== Aufbau / Reset
    init() {
      // Seite schließen/neu laden mitten im Flug: Takt sichern (wie on_exit)
      this._onPageHide = () => {
        try {
          if (this.round && L.flying(this.round)) this._saveExtra();
        } catch (e) { /* beim Verlassen nichts mehr melden */ }
      };
      window.addEventListener("pagehide", this._onPageHide);
    }

    reset() {
      this.gameOver = false;
      bank.load();
      this.rng = PG.rand; // Optik (Partikel, Münzen …) - Fairness-Seeds kommen aus crypto
      this.bets = [this._choiceOpt("bet1", L.BETS, L.BETS[0]), this._choiceOpt("bet2", L.BETS, L.BETS[0])];
      this.autos = [this._opt("auto1", false) === true, this._opt("auto2", true) === true];
      this.targets = [this._targetOpt("target1", L.DEFAULT_TARGET), this._targetOpt("target2", 1000)];
      this.two = this._opt("two", false) === true;
      this.turbo = this._opt("turbo", false) === true;
      this.musicOn = this._opt("music", true) === true;
      this.autoRounds = this._choiceOpt("auto_rounds", ROUND_OPTS, 10);
      this.autoWin = this._choiceOpt("auto_win", LIMIT_OPTS, 0);
      this.autoLoss = this._choiceOpt("auto_loss", LIMIT_OPTS, 0);
      let extra = bank.getExtra(KEY, {});
      if (!isDict(extra)) extra = {};
      this.hist = L.cleanHistory(extra.hist);
      this.fair = F.cleanState(extra.fair);
      this.fairlog = F.cleanLog(extra.fairlog);
      this.stats = CS.clean(extra.stats);
      this.stats.x = L.cleanX(this.stats.x);
      this.carry = L.cleanCarry(extra.carry); // Rest-Hundertstel der Auszahlungen
      this.round = L.cleanRound(extra.round);
      this._crash = null; // Platzpunkt der laufenden Runde (nur _advance/_finish!)
      this.state = IDLE;
      this.sel = 0; // gewählte Einsatz-Karte
      this.acc = 0; // angefangene Takte (Zeit-Akkumulator)
      this.countT = 0; // Countdown-Rest
      this.countLen = COUNT_T;
      this.ff = false; // Vorspulen (nur wenn alle ausgestiegen sind)
      this.end = null; // "pop" / "moon" der letzten Runde
      this.endT = 0;
      this.last = null; // letzte fertige Runde (Anzeige im Ruhezustand)
      this.toneI = 0; // nächster Ton aus TONE_AT
      this.worldSeen = 0; // höchste schon angesagte Höhenwelt der Runde
      this.worldLabel = null; // [Text-Schlüssel, Restzeit]
      this.squeakT = 0;
      this.flameT = 0; // Brenner-Stoß
      this.envRect = R(0, 0, 0, 0); // Hülle im letzten Bild (für den Knall)
      this.chutes = []; // Lamas / Geldsäcke am Fallschirm
      this.fallers = []; // purzelnde Lamas nach dem Platzen
      this.shreds = []; // Gummifetzen
      this.popFx = null; // Knall-Stern {x, y, t, r}
      this.basketFx = null; // fallender Korb nach dem Platzen
      this.particles = [];
      this.coinsFx = [];
      this.popups = [];
      this.toast = null;
      this.banner = null; // [Titel, Untertitel, Restzeit, Dauer]
      this.shake = 0;
      this.time = 0;
      this.mouse = [-1, -1];
      this.broke = false;
      this.window = null; // null / "auto" / "stats" / "fair"
      this.auto = null; // laufender Auto-Modus
      this.autoFocus = 0;
      this.fairSel = null;
      this.fairScroll = 0;
      this.fairChecks = new Map(); // Nonce -> [stimmt?, Platzpunkt, Ausstiege]
      this.rFairRows = [];
      this.rFairNew = R(0, 0, 0, 0);
      this.rFairVerify = R(0, 0, 0, 0);
      this.statsHover = null;
      this.musStage = null;
      this.musT = 0;
      this.musVol = null;
      this._musFail = null; // Stufe, deren Start fehlschlug (kein Audio)
      this.pending = 0;
      this.shownChips = bank.balance();
      this._tickCd = 0;
      this._commitMemo = ["", ""];
      this._lang = PG.lang;
      this._layout();
      if (!sameFair(extra.fair, this.fair)) {
        // frisch gewürfelte Seeds sofort sichern: der gezeigte Prüfcode bleibt gültig
        this._saveExtra();
      }
      if (this.round !== null) {
        this._crash = L.crashPoint(this.round.s, this.round.c, this.round.n);
        if (L.flying(this.round)) {
          // Unterbrochene Runde: Countdown, dann geht es am selben Takt weiter
          this.two = this.round.bets.length > 1;
          this.state = COUNT;
          this.countT = this.countLen = RESUME_T;
          this.toneI = this._toneIndex(L.multAt(this.round.t));
          this.worldSeen = Math.min(4, L.worldOf(L.multAt(this.round.t)));
          this._say(t("lcx.resumed"), 2.4);
        } else {
          this._finishNow();
        }
      }
      if (this.round === null) this._clampBets();
      this.score = bank.scoreFor(KEY);
      this._checkBroke();
    }

    /** Verlassen (on_exit): fliegt noch Geld, wird der Takt gespeichert
     *  (Fortsetzen); sind alle ausgestiegen, wird die Runde sofort abgeschlossen. */
    destroy() {
      if (this._onPageHide) window.removeEventListener("pagehide", this._onPageHide);
      PG.audio.stopMusic(this);
      this.auto = null;
      if (this.round !== null) {
        if (L.flying(this.round)) this._saveExtra();
        else this._finishNow();
      }
      this.score = bank.scoreFor(KEY);
    }

    // ===================================================== Layout
    _layout() {
      const W_ = this.width, H = this.height;
      const k = (this.k = Math.min(W_ / 800, H / 600));
      this.fMicro = ui.font(Math.max(10, trunc(11 * k)));
      this.fTiny = ui.font(Math.max(11, trunc(13 * k)));
      this.fSmall = ui.font(Math.max(12, trunc(15 * k)));
      this.fBtn = ui.font(Math.max(12, trunc(15 * k)), true);
      this.fBig = ui.font(Math.max(15, trunc(22 * k)), true);
      this.fMult = ui.font(Math.max(20, trunc(40 * k)), true);
      this.fHuge = ui.font(Math.max(26, trunc(58 * k)), true);
      this.fMono = ui.font(Math.max(10, trunc(12 * k)), false, true);
      this.fsMicro = Math.max(9, trunc(11 * k)); // Schriftgrößen für gecachte Zeichen-Bausteine
      this.fsTiny = Math.max(11, trunc(13 * k));
      const m = (this.margin = Math.max(6, trunc(12 * k)));
      this.hudH = Math.max(28, trunc(42 * k));
      this.stripH = Math.max(46, trunc(64 * k));
      this.toolH = Math.max(26, trunc(34 * k));
      this.strip = R(0, H - this.stripH, W_, this.stripH);
      const toolY = this.hudH + m;
      const areaY = toolY + this.toolH + m;
      const areaH = this.strip.y - m - areaY;
      const panelW = Math.max(184, Math.min(trunc(W_ * 0.38), trunc(300 * k)));
      this.stage = R(m, areaY, W_ - panelW - 3 * m, areaH);
      this.panelR = R(this.stage.right + m, areaY, panelW, areaH);
      this._layoutTools(toolY);
      this._layoutStrip();
      this._layoutCards();
      this._layoutStage();
      this._layoutAuto();
    }

    /** Werkzeug-Knöpfe oben links (mit Beschriftung, wenn Platz ist). */
    _layoutTools(y) {
      const m = this.margin;
      const h = this.toolH;
      const pad = Math.max(6, trunc(10 * this.k));
      const gap = Math.max(3, trunc(5 * this.k));
      const widths = TOOLS.map((name) => h + pad + this.fTiny.width(t(TOOL_LABEL[name])));
      let total = 0;
      for (const w of widths) total += w + gap;
      this.toolLabels = total <= trunc(this.stage.w * 0.72);
      let x = m;
      this.rTools = [];
      TOOLS.forEach((name, i) => {
        const w = this.toolLabels ? widths[i] : h;
        this.rTools.push([name, R(x, y, w, h)]);
        x += w + gap;
      });
      this.toolsRight = x;
    }

    /** Bedienleiste: Gesamteinsatz, Hälfte-Knopf, Status, Hauptknopf. */
    _layoutStrip() {
      const m = this.margin, k = this.k, W_ = this.width;
      const sh = this.stripH;
      const bh = Math.max(30, trunc(sh * 0.62));
      const by = this.strip.y + fdiv(sh - bh, 2);
      this.stripBh = bh;
      const tw = Math.max(this.fTiny.width(t("lcx.total")), this.fBtn.width("20.000")) + trunc(16 * k);
      this.rTotal = R(m, by, tw, bh);
      const hw = Math.max(bh, bh + this.fBtn.width(t("lcx.half_all")) + trunc(14 * k));
      this.rHalf = R(this.rTotal.right + Math.max(4, trunc(8 * k)), by, hw, bh);
      const pad = trunc(22 * Math.max(0.7, k));
      const labels = [t("lcx.play"), t("lcx.cash_all") + " 9.999.999", t("lcx.ff"), t("chip.auto_stop")];
      let goW = 0;
      for (const lab of labels) goW = Math.max(goW, this.fBtn.width(lab) + pad);
      goW = Math.min(Math.max(goW, trunc(130 * k)), trunc(W_ * 0.45));
      this.rGo = R(W_ - m - goW, by, goW, bh);
      const room = this.rGo.x - m - this.rHalf.right - m;
      this.halfLabel = room >= 0;
      if (room < 0) this.rHalf.w = bh; // zu eng: Hälfte-Knopf nur mit Symbol
      this.statusX = [this.rHalf.right + m, this.rGo.x - m];
    }

    /** Seitenleiste: zwei Einsatz-Karten, Info-Kasten, Hinweis. */
    _layoutCards() {
      const p = this.panelR;
      const k = this.k;
      const pad = (this.ppad = Math.max(5, trunc(8 * k)));
      const x0 = p.x + pad, w = p.w - 2 * pad;
      this.panelInner = [x0, w];
      const rh = (this.rowH = Math.max(22, trunc(28 * k)));
      const gap = (this.gap = Math.max(4, trunc(6 * k)));
      const cpad = Math.max(4, trunc(6 * k));
      const titleH = this.fTiny.px + Math.max(2, trunc(4 * k));
      const cardH = 2 * cpad + titleH + 2 * rh + gap;
      this.cards = [];
      let y = p.y + pad;
      for (let i = 0; i < 2; i++) {
        const c = R(x0, y, w, cardH);
        const iw = w - 2 * cpad;
        const ix = c.x + cpad;
        const ty = c.y + cpad;
        const ya = ty + titleH;
        const yb = ya + rh + gap;
        const swW = Math.max(trunc(rh * 1.6), 30);
        const vw = iw - 2 * rh - 8;
        const tv = iw - 3 * rh - 12;
        const cashW = iw - rh - 4;
        this.cards.push({
          rect: c, titleY: ty, titleH,
          sw: R(c.right - cpad - swW, ty, swW, titleH - 2),
          minus: R(ix, ya, rh, rh), bet: R(ix + rh + 4, ya, vw, rh),
          plus: R(ix + rh + 4 + vw + 4, ya, rh, rh),
          auto: R(ix, yb, rh, rh), tl: R(ix + rh + 4, yb, rh, rh),
          tv: R(ix + 2 * rh + 8, yb, tv, rh),
          tr: R(ix + 2 * rh + 8 + tv + 4, yb, rh, rh),
          cash: R(ix, yb, cashW, rh), halfc: R(ix + cashW + 4, yb, rh, rh),
          rowA: R(ix, ya, iw, rh), rowB: R(ix, yb, iw, rh),
        });
        y = c.bottom + gap;
      }
      this.infoY = y;
      const hintH = 2 * this.fMicro.px + 2;
      this.hintY = p.bottom - pad - hintH;
      if (this.hintY - this.infoY < 3 * (this.fTiny.px + 2)) this.hintY = p.bottom - pad; // zu eng: kein Hinweis
    }

    /** Bühne: Zeichenfläche der Kurve (Platz für den Ballon über der Spitze). */
    _layoutStage() {
      const st = this.stage;
      const base = (this.rigBase = Math.max(30, trunc(st.h * 0.19)));
      const [bw, bh] = D.balloonSize(base, D.INFL_MAX);
      const basketH = fdiv(D.basketW(base) * 2, 3);
      // Start auf dem Startplatz der Wiese (lamacrash_world: Mitte bei 15 % der
      // Breite, Oberkante max(16, 5 % der Höhe) über dem Bühnenrand)
      const left = Math.max(fdiv(bw, 2) + 4, this.fMicro.width("×1.000") + 10, fdiv(st.w * 15, 100));
      const right = Math.max(fdiv(bw, 2) + 4, trunc(10 * this.k));
      const top = trunc(bh * 0.92);
      const bottom = basketH + Math.max(16, fdiv(st.h * 5, 100));
      this.plot = R(st.x + left, st.y + top, Math.max(20, st.w - left - right), Math.max(20, st.h - top - bottom));
    }

    /** Auto-Fenster über der Bühne. */
    _layoutAuto() {
      const r = inflate(this.stage, -this.margin, -this.margin);
      this.autoR = r;
      const k = this.k;
      const pad = Math.max(6, trunc(10 * k));
      const cb = Math.max(18, trunc(24 * k));
      this.rAutoClose = R(r.right - pad - cb, r.y + pad, cb, cb);
      const x0 = r.x + pad, w = r.w - 2 * pad;
      let y = r.y + pad + Math.max(cb, this.fBtn.px) + Math.max(3, trunc(6 * k));
      const lab = this.fTiny.px;
      const rh = Math.max(20, trunc(26 * k));
      const gap = Math.max(3, trunc(5 * k));
      const rows = [];
      for (const opts of AUTO_OPTS) {
        const ly = y;
        y += lab + 2;
        const n = opts.length;
        let first = 0;
        if (opts[0] === 0) {
          first = Math.max(0, Math.min(this.fTiny.width(t("chip.auto_off")), this.fMicro.width(t("chip.auto_off")) + 4) + 8);
        }
        const bw = fdiv(w - first - (n - 1) * 3, first ? n - 1 : n);
        const rects = [];
        let x = x0;
        for (let i = 0; i < n; i++) {
          const rw = i === 0 && first ? first : bw;
          rects.push(R(x, y, rw, rh));
          x += rw + 3;
        }
        rows.push([ly, rects]);
        y += rh + gap;
      }
      this.autoRows = rows;
      this.rAutoTurbo = R(x0, y, Math.min(w, Math.max(trunc(w * 0.5), this.fBtn.width(t("chip.turbo")) + 3 * rh + 10)), rh);
      y += rh + gap;
      this.autoSumY = y;
      const bh = Math.max(26, trunc(34 * k));
      const sw = Math.min(w, trunc(220 * k));
      this.rAutoStart = R(cxOf(r) - fdiv(sw, 2), r.bottom - pad - bh, sw, bh);
    }

    // ===================================================== Einstellungen
    _opt(key, def) {
      const v = this.opts[key];
      return v === undefined ? def : v;
    }
    _choiceOpt(key, opts, def) {
      const v = this._opt(key, def);
      return isInt(v) && opts.includes(v) ? v : def;
    }
    _targetOpt(key, def) {
      const v = this._opt(key, def);
      return L.targetOk(v) ? v : def;
    }
    _setOpt(key, value) {
      if (this.opts[key] !== value) {
        this.opts[key] = value;
        this.saveSettings();
      }
    }

    // ===================================================== Konto / Speichern
    _extra() {
      return { round: this.round, hist: this.hist, fair: this.fair, fairlog: this.fairlog, stats: this.stats,
        carry: this.carry };
    }
    _saveExtra() {
      bank.setExtra(KEY, this._extra());
    }
    /** Indizes der aktiven Einsatz-Karten (Karte 1 immer, Karte 2 per Schalter). */
    _active() {
      return this.two ? [0, 1] : [0];
    }
    _total() {
      let total = 0;
      for (const i of this._active()) total += this.bets[i];
      return total;
    }
    /** Einsätze auf das Bezahlbare senken - zuerst Karte 2, dann Karte 1. */
    _clampBets() {
      const chips = bank.balance();
      for (const i of this.two ? [1, 0] : [0]) {
        const others = this._total() - this.bets[i];
        while (this.bets[i] > L.BETS[0] && others + this.bets[i] > chips) this.bets[i] = L.BETS[L.BETS.indexOf(this.bets[i]) - 1];
      }
    }
    _checkBroke() {
      this.broke = this.state === IDLE && this.round === null && this.auto === null && bank.isBroke(KEY);
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
    _say(text, secs = 1.8) {
      this.toast = [text, secs];
    }
    /** Zeitfaktor für Countdown und Pausen (Turbo/Auto schneller). */
    _tf() {
      return this.turbo || this.auto !== null ? TURBO_F : 1.0;
    }
    _entry(rnd) {
      return { n: rnd.n, s: rnd.s, c: rnd.c, h: rnd.h };
    }

    // ===================================================== Formatierung
    _sep() {
      return t("lcx.decimal");
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
    /** Multiplikator x100 als Text: ×1.234,56 (ab ×10.000 ohne Nachkommastellen). */
    _fmt(m100) {
      if (m100 >= 1000000) return "×" + this._group(fdiv(m100, 100));
      return "×" + this._group(fdiv(m100, 100)) + this._sep() + String(m100 % 100).padStart(2, "0");
    }
    _fmtShort(m100) {
      return "×" + L.fmtShort(m100, this._sep());
    }
    /** Farbe des großen Multiplikators: weiß → gold → orange → rot → violett. */
    _multCol(m100) {
      const stops = [[100, [240, 244, 255]], [200, [255, 226, 120]], [1000, [255, 168, 60]],
        [10000, [255, 84, 96]], [100000, [214, 120, 255]]];
      if (m100 <= stops[0][0]) return stops[0][1];
      for (let i = 0; i + 1 < stops.length; i++) {
        const [a, ca] = stops[i];
        const [b, cb] = stops[i + 1];
        if (m100 <= b) return ui.mix(ca, cb, (m100 - a) / (b - a));
      }
      return stops[stops.length - 1][1];
    }

    // ===================================================== Kurve / Höhe
    /** Höhe als gebrochener Takt (für Bild und Welt; Geld zählt nur ganze Takte). */
    _alt() {
      if ((this.state === FLY || this.state === COAST) && this.round !== null) return this.round.t + this.acc;
      if (this.state === COUNT && this.round !== null) return this.round.t;
      if (this.state === END && this.last !== null) return this.last.rnd.t;
      return 0;
    }
    /** Multiplikator x100 zu einem gebrochenen Takt (nur fürs Zeichnen). */
    _mAtF(tf) {
      const i = trunc(tf);
      const a = L.multAt(i);
      const b = L.multAt(i + 1);
      return a + (b - a) * (tf - i);
    }
    /** Angezeigter Multiplikator (ganzer Takt; nach dem Platzen der Platzpunkt). */
    _curMult() {
      const st = this.state;
      if ((st === FLY || st === COAST || st === COUNT) && this.round !== null) return L.multAt(this.round.t);
      if (st === END && this.last !== null) return this.last.end === "pop" ? this.last.crash : L.MAX_MULT100;
      return 100;
    }
    _toneIndex(m100) {
      let i = 0;
      while (i < TONE_AT.length && m100 >= TONE_AT[i]) i += 1;
      return i;
    }

    // ===================================================== Ablauf
    /** SPIELEN: Einsätze abbuchen, Runde speichern, Countdown starten. */
    _play() {
      if (this.state === END && this.endT >= READ_T) this._toIdle();
      if (this.state !== IDLE || this.broke || this.round !== null) return false;
      const forced = this.auto !== null;
      const pairs = [];
      for (const i of this._active()) {
        const a = this.autos[i] || forced ? this.targets[i] : 0;
        pairs.push([this.bets[i], a]);
      }
      const total = this._total();
      if (total > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return false;
      }
      const entry = F.begin(this.fair);
      F.advance(this.fair);
      this.round = L.newRound(entry, pairs);
      if (!bank.betAndPay(total, 0, KEY, KEY, this._extra())) {
        this.round = null;
        return false;
      }
      this._crash = L.crashPoint(entry.s, entry.c, entry.n);
      this.state = COUNT;
      this.countT = this.countLen = COUNT_T * this._tf();
      this.acc = 0;
      this.ff = false;
      this.toneI = this._toneIndex(100);
      this.worldSeen = 0;
      this.worldLabel = null;
      this.squeakT = 0;
      this.last = null;
      this.end = null;
      this.chutes = [];
      this.fallers = [];
      this.shreds = [];
      this.popFx = this.basketFx = null;
      this.window = null;
      this.score = bank.scoreFor(KEY);
      this.playSound("select");
      return true;
    }

    /** Countdown vorbei: Brenner-Stoß, der Ballon hebt ab. */
    _launch() {
      this.state = FLY;
      this.acc = 0;
      this.flameT = 0.9;
      this.tone(150, 0.45, "noise", 0.16);
      this.tone(1320, 0.1, "sine", 0.25);
    }

    /** Takte abarbeiten (Takt-Regel in L.tick) - je Takt Auszahlungen und Ende. */
    _advance(dt) {
      let speed;
      if (this.state === COAST && (this.ff || this.auto !== null)) speed = L.SPEED_FF;
      else speed = this.turbo ? L.SPEED_TURBO : 1;
      dt = Math.min(dt, 0.1); // ein Ruckler wird nicht nachgeholt (wie main.py)
      this.acc += dt * 60.0 * speed;
      const n = trunc(this.acc + 1e-9);
      this.acc = Math.max(0.0, this.acc - n);
      for (let j = 0; j < n; j++) {
        const [events, carry, end] = L.tick(this.round, this._crash, this.carry);
        this.carry = carry;
        for (const ev of events) this._paid(ev);
        if (end !== null) {
          this._finish(end);
          return;
        }
        if (events.length) this._afterPay();
      }
    }

    /** Landeplatz des Korbs auf dem Mond (Korbrand-Mitte). */
    _moonRig() {
      return [cxOf(this.stage), this.stage.bottom - trunc(this.stage.h * 0.16)];
    }
    /** Korbrand-Mitte des Ballons zur aktuellen Höhe (Bildschirm). */
    _rigPos() {
      const tf = this._alt();
      if (tf <= 0) return [this.plot.x, this.plot.bottom];
      const [xMax, yMax] = L.chartSpan(trunc(tf));
      return this._pt(tf, this._mAtF(tf), xMax, yMax);
    }
    _pt(tf, m100, xMax, yMax) {
      const p = this.plot;
      const x = p.x + p.w * Math.min(1.0, tf / Math.max(1, xMax));
      const y = p.bottom - (p.h * (m100 - 100)) / Math.max(1, yMax - 100);
      return [x, y];
    }
    _nRiders() {
      if (this.round !== null) return this.round.bets.length;
      if (this.last !== null && this.state === END) return this.last.rnd.bets.length;
      return this._active().length;
    }
    /** Platz eines Lamas im Korb (links/rechts) relativ zur Korbmitte. */
    _slotX(i) {
      if (this._nRiders() < 2) return 0;
      const bw = D.basketW(this.rigBase);
      return i === 0 ? fdiv(-bw, 4) : fdiv(bw, 4);
    }

    /** Auto-Ziel bzw. Mondlandung: gutschreiben (gespeichert wird danach einmal). */
    _paid(ev) {
      const [i, kind, m100, win] = ev;
      bank.credit(win, KEY, false);
      if (kind === "auto") {
        this._jump(i, "llama", win);
        this._celebrate(m100);
      } else {
        const [x, y] = this._moonRig();
        this._popup(x + this._slotX(i), y - this.rigBase * 0.9 - i * this.fBig.px, "+" + this._group(win), CD.COL_GOLD, 2.4);
      }
      this._cashAch(m100);
    }
    /** Nach Auszahlungen im Flug: einmal speichern, ggf. in den Leerflug. */
    _afterPay() {
      this._saveExtra();
      this.score = bank.scoreFor(KEY);
      if (this.state === FLY && !L.flying(this.round)) {
        this.state = COAST;
        this.ff = false;
      }
    }
    /** Erfolge beim Aussteigen sofort (verrät nichts, geht beim Verlassen nicht verloren). */
    _cashAch(m100) {
      if (m100 >= L.BRAVE_MULT) this.achEvent("lcx_brave");
      if (m100 >= L.SPACE_MULT) this.achEvent("lcx_space");
    }
    /** Lama springt ab (bzw. Geldsack fliegt) - mit Fallschirm, Popup, Ton. */
    _jump(i, kind, win) {
      const [x, y] = this._rigPos();
      const n = D.llamaSize(this.rigBase);
      const sx = x + this._slotX(i);
      const size = kind === "llama" ? n : Math.max(8, fdiv(n * 2, 3));
      this.chutes.push({ kind, slot: i, x: sx, y: y + n * 0.15, vx: -(40 + 30 * i) * this.k, vy: -150 * this.k, t: 0, n: size });
      this._popup(sx, y - n, "+" + this._group(win), CD.COL_GOLD, 1.4);
      this.playSound(kind === "llama" ? "win" : "merge");
      this.tone(660, 0.07, "sine", 0.22);
    }
    /** Von Hand aussteigen: parts = [[Einsatz-Index, Hälfte?], ...] - ein Speichern. */
    _cashParts(parts) {
      if (this.state !== FLY || this.round === null) return false;
      if (L.multAt(this.round.t) < L.MIN_CASH) {
        this._say(t("lcx.need_cash"), 1.2);
        return false;
      }
      let done = false;
      let best = 0;
      for (const [i, half] of parts) {
        const res = L.cashOut(this.round, i, half, this.carry);
        if (res === null) continue;
        const [win, m100, carry] = res;
        this.carry = carry;
        bank.credit(win, KEY, false);
        this._jump(i, half ? "sack" : "llama", win);
        this._cashAch(m100);
        best = Math.max(best, m100);
        done = true;
      }
      if (done) {
        this._afterPay();
        this._celebrate(best);
      }
      return done;
    }
    _cashAll() {
      if (this.round === null) return;
      const parts = [];
      this.round.bets.forEach((b, i) => {
        if (L.riding(b) > 0) parts.push([i, false]);
      });
      if (parts.length && !this._cashParts(parts)) this.playSound("hit");
    }
    _cashOne(i, half) {
      if (this.round === null || i >= this.round.bets.length) return;
      const b = this.round.bets[i];
      if (L.riding(b) <= 0) return;
      if (half && b.hm) {
        this._say(t("lcx.half_done"), 1.2);
        this.playSound("hit");
        return;
      }
      if (!this._cashParts([[i, half]])) this.playSound("hit");
    }
    _halfAll() {
      if (this.round === null) return;
      const parts = [];
      this.round.bets.forEach((b, i) => {
        if (L.riding(b) > 0 && !b.hm) parts.push([i, true]);
      });
      if (!parts.length) {
        this._say(t("lcx.half_done"), 1.2);
        this.playSound("hit");
        return;
      }
      if (!this._cashParts(parts)) this.playSound("hit");
    }

    /** Rundenende verbuchen: Zähler, Statistik, Verlauf, Fairness-Log - einmal. */
    _finish(end, quiet = false) {
      const rnd = this.round;
      if (rnd === null) return;
      const crash = this._crash;
      L.recordX(this.stats.x, rnd, crash, end);
      const stake = L.stakeTotal(rnd), paid = L.paidTotal(rnd);
      CS.record(this.stats, stake, paid, L.bestCash(rnd));
      this.hist = this.hist.concat([crash]).slice(-L.HISTORY);
      F.record(this.fairlog, this._entry(rnd), L.outcome(rnd, crash));
      const close = L.closeCall(rnd, crash);
      const lost = [];
      if (end === "pop") {
        rnd.bets.forEach((b, i) => {
          if (L.riding(b) > 0) lost.push(i);
        });
      }
      if (this.auto !== null) this.auto.net += paid - stake;
      this.round = null;
      this._crash = null;
      this.last = { rnd, crash, end, stake, paid, lost };
      this.fairSel = null;
      this._saveExtra();
      this.score = bank.scoreFor(KEY);
      if (close) this.achEvent("lcx_close");
      if (quiet) {
        this.state = IDLE;
        return;
      }
      this.state = END;
      this.end = end;
      this.endT = 0;
      if (end === "pop") this._pop(lost);
      else this._moon();
      if (close) this._say(t("lcx.close"), 2.0);
    }
    /** Runde ohne fliegendes Geld sofort abschließen (Verlassen im Leerflug, Laden). */
    _finishNow() {
      if (this.round === null) return;
      const end = this._crash >= L.MAX_MULT100 ? "moon" : "pop";
      this.round.t = L.popTick(this._crash);
      this._finish(end, true);
    }

    /** Der Ballon platzt: Knall, Fetzen, Wackeln, purzelnde Lamas, fallender Korb. */
    _pop(lost) {
      const e = this.envRect.w > 0 ? this.envRect : R(cxOf(this.plot), this.plot.y, 40, 40);
      const ex = cxOf(e), ey = cyOf(e);
      this.popFx = { x: ex, y: ey, t: 0, r: Math.max(12, fdiv(e.w, 2) + trunc(10 * this.k)) };
      this.shake = SHAKE_T;
      const pieces = D.shreds(Math.max(8, e.w), this.accent);
      for (let j = 0; j < 18; j++) {
        const a = this.rng.uniform(0, TAU);
        const sp = this.rng.uniform(120, 420) * this.k;
        this.shreds.push({ x: ex, y: ey, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp - 80 * this.k, t: 0,
          life: this.rng.uniform(0.9, 1.6), i: j % pieces.length, w: Math.max(8, e.w) });
      }
      const [x, y] = this._rigPos();
      const n = D.llamaSize(this.rigBase);
      for (const i of lost) {
        this.fallers.push({ x: x + this._slotX(i), y: y + n * 0.15, vx: (i === 0 ? -60 : 60) * this.k, vy: -120 * this.k, t: 0, slot: i });
      }
      this.basketFx = { x, y, vy: -40 * this.k, t: 0 };
      this.playSound("explode");
      this.rumble(320);
      if (lost.length) this.tone(220, 0.35, "saw", 0.18);
    }
    /** Mondlandung: Höchstgewinn - Fahne, Konfetti, Münzregen. */
    _moon() {
      this.banner = [t("lcx.moon"), t("lcx.moon_sub"), 3.4, 3.4];
      ui.spawnConfetti(this.width, this.height, 150);
      this._coinShower(90);
      this.playSound("level");
      this.rumble(300);
    }
    /** Jubel im Moment des Aussteigens (hängt nur vom Ausstieg ab, verrät nichts):
     *  Münzregen ab x3, Konfetti ab x10, Gewinn-Banner ab x100. */
    _celebrate(m100) {
      if (m100 >= BANNER_MULT) {
        this.banner = [t("cas.s.big_win"), this._fmt(m100), 2.6, 2.6];
        this.playSound("level");
      }
      if (m100 >= PARTY_MULT) {
        ui.spawnConfetti(this.width, this.height, 110);
        this._coinShower(60);
        this.rumble(200);
      } else if (m100 >= BIG_MULT) {
        this._coinShower(28);
      }
    }
    /** Ergebnis schließen: Ruhezustand (Korb wieder auf dem Startplatz). */
    _toIdle() {
      this.state = IDLE;
      this.endT = 0;
      this.popFx = null;
      this.basketFx = null;
      this.fallers = [];
      this._clampBets();
      this._checkBroke();
    }

    // ----- Einstellungen ändern ------------------------------------------------
    /** Wer im Ergebnis etwas verstellt, schließt es (Karten zeigen wieder die Einstellungen). */
    _leaveResult() {
      if (this.state === END && this.round === null) this._toIdle();
    }
    _canEdit() {
      return (this.state === IDLE || this.state === END) && this.round === null && this.auto === null;
    }
    _setBet(i, direction) {
      if (!this._canEdit() || !this._active().includes(i)) return;
      this._leaveResult();
      const j = L.BETS.indexOf(this.bets[i]);
      const k = Math.max(0, Math.min(L.BETS.length - 1, j + direction));
      if (k !== j) {
        this.bets[i] = L.BETS[k];
        this._setOpt("bet" + (i + 1), this.bets[i]);
        this.playSound("select");
        if (this._total() > bank.balance()) this._say(t("cas.not_enough"), 1.2);
      }
    }
    _setTarget(i, direction) {
      if (!this._canEdit() || !this._active().includes(i)) return;
      this._leaveResult();
      const v = L.stepTarget(this.targets[i], direction);
      if (v !== this.targets[i]) {
        this.targets[i] = v;
        this._setOpt("target" + (i + 1), v);
        this.playSound("click");
      }
      if (!this.autos[i]) {
        this.autos[i] = true;
        this._setOpt("auto" + (i + 1), true);
      }
    }
    _toggleTarget(i) {
      if (!this._canEdit() || !this._active().includes(i)) return;
      this._leaveResult();
      this.autos[i] = !this.autos[i];
      this._setOpt("auto" + (i + 1), this.autos[i]);
      this.playSound("click");
    }
    _toggleTwo() {
      if (!this._canEdit()) {
        this._say(t("lcx.locked"), 1.2);
        return;
      }
      this._leaveResult();
      this.two = !this.two;
      this._setOpt("two", this.two);
      if (!this.two && this.sel === 1) this.sel = 0;
      this._clampBets();
      this._say(this.two ? t("lcx.two_on") : t("lcx.two_off"), 1.0);
      this.playSound("click");
    }
    _select(i) {
      if (i === 1 && !this.two) return;
      this._leaveResult();
      if (this.sel !== i) {
        this.sel = i;
        this.playSound("rotate");
      }
    }
    _toggleTurbo() {
      this.turbo = !this.turbo;
      this._setOpt("turbo", this.turbo);
      this._say(this.turbo ? t("lcx.turbo_on") : t("lcx.turbo_off"), 1.0);
      this.playSound("click");
    }
    _toggleMusic() {
      this.musicOn = !this.musicOn;
      this._setOpt("music", this.musicOn);
      if (!this.musicOn) {
        PG.audio.stopMusic(this);
        this.musStage = null;
      }
      this._say(this.musicOn ? t("lcx.music_on") : t("lcx.music_off"), 1.0);
      this.playSound("click");
    }
    _openWindow(name) {
      if (this.window === name) {
        this.window = null;
        this.playSound("click");
        return;
      }
      if (name === "auto" && this.auto !== null) {
        this._stopAuto(t("chip.auto_done"));
        return;
      }
      if (this.round !== null && L.flying(this.round)) {
        this._say(t("lcx.locked_fly"), 1.2);
        this.playSound("hit");
        return;
      }
      if (name === "auto") {
        if (this.state === END && this.endT >= READ_T) this._toIdle();
        if (this.state !== IDLE || this.broke || this.round !== null) {
          this._say(t("lcx.locked"), 1.2);
          return;
        }
      }
      this.window = name;
      if (name === "fair") {
        this.fairScroll = 0;
        this.fairSel = this.fairlog.length ? this.fairlog.length - 1 : null;
      }
      this.playSound("click");
    }

    // ----- Auto-Modus ------------------------------------------------------------
    _startAuto() {
      if (this.round !== null || !(this.state === IDLE || this.state === END)) return;
      if (this._total() > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      let forced = false;
      for (const i of this._active()) {
        if (!this.autos[i]) {
          this.autos[i] = true;
          this._setOpt("auto" + (i + 1), true);
          forced = true;
        }
      }
      this.window = null;
      if (this.state === END) this._toIdle();
      this.auto = { left: this.autoRounds, net: 0, t: 0 };
      if (forced) this._say(t("lcx.auto_forced"), 1.8);
      this.playSound("powerup");
    }
    /** Auto beenden. Eine laufende Runde bleibt offen und fliegt weiter. */
    _stopAuto(msg = null) {
      if (this.auto === null) return;
      this.auto = null;
      if (msg) this._say(msg, 2.0);
      this.playSound("click");
      if (this.state === IDLE) this._checkBroke();
    }
    _autoTick(dt) {
      const a = this.auto;
      a.t -= dt;
      if (a.t > 0) return;
      if (a.left <= 0) {
        this._stopAuto(t("chip.auto_done"));
        return;
      }
      if (this.autoWin > 0 && a.net >= this.autoWin) {
        this._stopAuto(t("chip.auto_hit_win"));
        return;
      }
      if (this.autoLoss > 0 && -a.net >= this.autoLoss) {
        this._stopAuto(t("chip.auto_hit_loss"));
        return;
      }
      if (this._total() > bank.balance()) {
        this._stopAuto(t("chip.auto_no_chips"));
        return;
      }
      if (this._play()) a.left -= 1;
      else this._stopAuto(t("chip.auto_no_chips"));
    }

    // ===================================================== Effekte
    _spawn(x, y, n, col, speed, r, life, grav) {
      const Rn = this.rng;
      for (let j = 0; j < n; j++) {
        const a = Rn.uniform(0, TAU);
        const sp = speed * Rn.uniform(0.3, 1.0) * this.k;
        this.particles.push({ x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, r: r * this.k * Rn.uniform(0.6, 1.25), col,
          t: 0, life: life * Rn.uniform(0.7, 1.15), g: grav * this.k });
      }
      if (this.particles.length > 260) this.particles = this.particles.slice(-260);
    }
    _coinShower(n = 40) {
      const Rn = this.rng, W_ = this.width, k = this.k;
      for (let j = 0; j < n; j++) {
        this.coinsFx.push({
          x: Rn.uniform(W_ * 0.05, W_ * 0.95), y: Rn.uniform(-this.height * 0.4, -10),
          vx: Rn.uniform(-60, 60) * k, vy: Rn.uniform(-80, 120) * k, spin: Rn.uniform(0, TAU),
          vs: Rn.uniform(6, 14), r: Rn.uniform(7, 12) * Math.max(0.7, k),
        });
      }
      if (this.coinsFx.length > 180) this.coinsFx = this.coinsFx.slice(-180);
    }
    _popup(x, y, text, color = null, dur = 1.4) {
      this.popups.push({ x, y, text, color: color || ui.GOLD, t: 0, dur });
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      const kind = ev.kind;
      if ((kind === "mousemove" || kind === "mousedown" || kind === "mouseup") && ev.pos) this.mouse = ev.pos;
      if (this.broke) {
        if (kind === "mousedown" || (kind === "keydown" && isAction(ev.key))) this._takeCredit();
        return;
      }
      if (kind === "keydown") this._onKey(ev.key, !!ev.repeat);
      else if (kind === "wheel") this._onWheel(ev.delta);
      else if (kind === "mousedown" && ev.button === 1) this._onClick(ev.pos);
    }

    _onKey(key, repeat = false) {
      if (!key) return;
      const low = key.length === 1 ? key.toLowerCase() : key;
      if (key === "Escape") {
        if (this.window !== null) {
          this.window = null;
          this.playSound("click");
        } else if (this.auto !== null) {
          this._stopAuto(t("chip.auto_done"));
        }
        return;
      }
      if (key.length === 1 && (low === "a" || low === "s" || low === "f" || low === "t" || low === "m")) {
        if (repeat) return;
        if (low === "t") this._toggleTurbo();
        else if (low === "m") this._toggleMusic();
        else this._openWindow({ a: "auto", s: "stats", f: "fair" }[low]);
        return;
      }
      if (this.window === "auto") {
        this._autoKey(key, repeat);
        return;
      }
      if (this.window === "fair") {
        this._fairKey(key, repeat);
        return;
      }
      if (this.window !== null) {
        if (isAction(key) && !repeat) this.window = null;
        return;
      }
      const st = this.state;
      if (st === FLY) {
        if (repeat) return;
        if (isAction(key) || low === "c") this._cashAll();
        else if (key === "1" || key === "2" || key === "KP_1" || key === "KP_2") this._cashOne(Number(key[key.length - 1]) - 1, false);
        else if (low === "h") this._halfAll();
        else if (low === "q" || low === "w") this._cashOne(low === "q" ? 0 : 1, true);
        return;
      }
      if (st === COAST) {
        if ((isAction(key) || low === "c") && !repeat && !this.ff) {
          this.ff = true;
          this.playSound("click");
        }
        return;
      }
      if (st === COUNT) return;
      if (this.auto !== null) {
        // Auto läuft (Ruhe/Ergebnis): Leertaste/Enter = STOPP
        if (isAction(key) && !repeat) this._stopAuto(t("chip.auto_done"));
        return;
      }
      if (st === END && (this.endT < READ_T || repeat) && isAction(key)) return;
      if (isAction(key)) {
        if (!repeat) this._play();
      } else if (key === "1" || key === "2" || key === "KP_1" || key === "KP_2") {
        this._select(Number(key[key.length - 1]) - 1);
      } else if (key === "Tab") {
        if (this.two) this._select(1 - this.sel);
      } else if (key === "plus" || key === "KP_Add" || key === "Up" || key === "equal") {
        this._setBet(this.sel, 1);
      } else if (key === "minus" || key === "KP_Subtract" || key === "Down") {
        this._setBet(this.sel, -1);
      } else if (key === "Left" || key === "Right") {
        this._setTarget(this.sel, key === "Right" ? 1 : -1);
      } else if (low === "x" && !repeat) {
        this._toggleTarget(this.sel);
      } else if (low === "z" && !repeat) {
        this._toggleTwo();
      }
    }

    /** Tasten im Auto-Fenster: ↑/↓ Zeile, ←/→ Wert, Enter/Leertaste = START/STOPP. */
    _autoKey(key, repeat) {
      if (key === "Up" || key === "Down") {
        this.autoFocus = (((this.autoFocus + (key === "Down" ? 1 : -1)) % 4) + 4) % 4;
        this.playSound("rotate");
      } else if (key === "Left" || key === "Right") {
        const d = key === "Right" ? 1 : -1;
        if (this.autoFocus === 3) {
          this._toggleTurbo();
          return;
        }
        const opts = AUTO_OPTS[this.autoFocus];
        const cur = this[AUTO_FIELDS[this.autoFocus]];
        const j = Math.max(0, Math.min(opts.length - 1, opts.indexOf(cur) + d));
        this._autoSet(this.autoFocus, opts[j]);
      } else if (isAction(key) && !repeat) {
        if (this.auto !== null) this._stopAuto(t("chip.auto_done"));
        else this._startAuto();
      }
    }
    _autoSet(row, value) {
      if (this.auto !== null) return;
      const field = AUTO_FIELDS[row];
      if (this[field] !== value) {
        this[field] = value;
        this._setOpt(AUTO_KEYS[row], value);
        this.playSound("click");
      }
    }

    _fairOrder() {
      const out = [];
      for (let i = this.fairlog.length - 1; i >= 0; i--) out.push(i); // neueste oben
      return out;
    }
    /** Fairness-Fenster: ↑/↓ wählt eine Runde, Enter rechnet nach, C = neuer Client-Seed. */
    _fairKey(key, repeat) {
      const low = key.length === 1 ? key.toLowerCase() : key;
      const order = this._fairOrder();
      if ((key === "Up" || key === "Down") && order.length) {
        let pos = order.includes(this.fairSel) ? order.indexOf(this.fairSel) : 0;
        pos = Math.max(0, Math.min(order.length - 1, pos + (key === "Down" ? 1 : -1)));
        this.fairSel = order[pos];
        this.playSound("click");
      } else if ((key === "Return" || key === "KP_Enter") && !repeat) {
        if (this.fairSel !== null) this._verify(this.fairSel);
      } else if (low === "c" && !repeat) {
        this._newClient();
      } else if (key === "space" && !repeat) {
        this.window = null;
      }
    }
    _newClient() {
      if (this.round !== null) {
        this._say(t("lcx.locked"), 1.4);
        this.playSound("hit");
        return;
      }
      F.newClient(this.fair);
      this._saveExtra();
      this.playSound("select");
    }

    _onWheel(delta) {
      const d = delta > 0 ? 1 : -1;
      if (this.window === "fair") {
        this.fairScroll = Math.max(0, this.fairScroll - d);
        return;
      }
      if (this.window !== null || !this._canEdit()) return;
      for (let i = 0; i < this.cards.length; i++) {
        const c = this.cards[i];
        if (c.rect.collidepoint(this.mouse) && this._active().includes(i)) {
          this._select(i);
          if (c.rowB.collidepoint(this.mouse)) this._setTarget(i, d);
          else this._setBet(i, d);
          return;
        }
      }
      this._setBet(this.sel, d);
    }

    _onClick(pos) {
      for (const [name, r] of this.rTools) {
        if (r.collidepoint(pos)) {
          if (name === "turbo") this._toggleTurbo();
          else if (name === "music") this._toggleMusic();
          else this._openWindow(name);
          return;
        }
      }
      if (this.window === "auto") {
        this._autoClick(pos);
        return;
      }
      if (this.window !== null) {
        this._windowClick(pos);
        return;
      }
      const st = this.state;
      if (this.rGo.collidepoint(pos)) {
        if (st === FLY) this._cashAll();
        else if (st === COAST) {
          if (!this.ff) {
            this.ff = true;
            this.playSound("click");
          }
        } else if (this.auto !== null) {
          this._stopAuto(t("chip.auto_done"));
        } else if (st === IDLE || st === END) {
          if (st === END && this.endT < READ_T) return;
          this._play();
        }
        return;
      }
      if (this.rHalf.collidepoint(pos)) {
        if (st === FLY) this._halfAll();
        return;
      }
      for (let i = 0; i < this.cards.length; i++) {
        const c = this.cards[i];
        if (!c.rect.collidepoint(pos)) continue;
        if (st === FLY && this.round !== null) {
          if (i < this.round.bets.length) {
            if (c.cash.collidepoint(pos)) this._cashOne(i, false);
            else if (c.halfc.collidepoint(pos)) this._cashOne(i, true);
          }
          return;
        }
        if (!this._canEdit()) {
          if (this.auto !== null) this._say(t("lcx.locked"), 1.2);
          return;
        }
        if (i === 1 && (c.sw.collidepoint(pos) || !this.two)) {
          this._toggleTwo();
          if (this.two) this.sel = 1;
          return;
        }
        this._select(i);
        if (c.minus.collidepoint(pos)) this._setBet(i, -1);
        else if (c.plus.collidepoint(pos)) this._setBet(i, 1);
        else if (c.auto.collidepoint(pos)) this._toggleTarget(i);
        else if (c.tl.collidepoint(pos)) this._setTarget(i, -1);
        else if (c.tr.collidepoint(pos)) this._setTarget(i, 1);
        return;
      }
      if (this.stage.collidepoint(pos) && st === IDLE && this.auto === null) this._say(t("lcx.press_play"), 1.4);
    }

    _autoClick(pos) {
      if (this.rAutoClose.collidepoint(pos)) {
        this.window = null;
        this.playSound("click");
        return;
      }
      for (let row = 0; row < this.autoRows.length; row++) {
        const rects = this.autoRows[row][1];
        for (let j = 0; j < rects.length; j++) {
          if (rects[j].collidepoint(pos)) {
            this.autoFocus = row;
            this._autoSet(row, AUTO_OPTS[row][j]);
            return;
          }
        }
      }
      if (this.rAutoTurbo.collidepoint(pos)) {
        this.autoFocus = 3;
        this._toggleTurbo();
      } else if (this.rAutoStart.collidepoint(pos)) {
        if (this.auto !== null) this._stopAuto(t("chip.auto_done"));
        else this._startAuto();
      } else if (!this.autoR.collidepoint(pos) && !this.panelR.collidepoint(pos)) {
        this.window = null;
      }
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
        this._newClient();
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

    /** Nachrechnen: Prüfcode = sha256(Seed), Platzpunkt aus den Seeds, Ausstiege passen. */
    _verify(idx) {
      if (!(idx >= 0 && idx < this.fairlog.length)) return;
      const e = this.fairlog[idx];
      let ok = F.verify(e);
      const crash = L.crashPoint(e.s, e.c, e.n);
      const o = e.o;
      ok = ok && L.checkOutcome(o, crash);
      const marks = [];
      if (isDict(o) && Array.isArray(o.b)) {
        for (const row of o.b) {
          if (Array.isArray(row) && row.length === 4) {
            for (const m of [row[1], row[2]]) if (isInt(m) && m > 0) marks.push(m);
          }
        }
      }
      marks.sort((a, b) => a - b);
      this.fairChecks.set(e.n, [ok, crash, marks]);
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
        const step = Math.max(Math.abs(diff) * Math.min(1.0, dt * 4.5), 60.0 * dt);
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
        this.banner[2] -= dt;
        if (this.banner[2] <= 0) this.banner = null;
      }
      if (this.worldLabel) {
        this.worldLabel[1] -= dt;
        if (this.worldLabel[1] <= 0) this.worldLabel = null;
      }
      this.shake = Math.max(0, this.shake - dt);
      this.flameT = Math.max(0, this.flameT - dt);
      const st = this.state;
      if (st === COUNT) {
        const seg = this.countLen / 3.0;
        const before = trunc(this.countT / seg);
        this.countT -= dt;
        if (this.countT <= 0) this._launch();
        else if (trunc(this.countT / seg) !== before) this.tone(880, 0.08, "sine", 0.22);
      } else if (st === FLY || st === COAST) {
        this._advance(dt);
      } else if (st === END) {
        this.endT += dt;
        const limit = this.auto !== null ? AUTO_END_T * this._tf() : END_T;
        if (this.endT >= limit) {
          this._toIdle();
          if (this.auto !== null) this.auto.t = AUTO_PAUSE_T * this._tf();
        }
      }
      if (this.auto !== null && this.state === IDLE) this._autoTick(dt);
      this._updateSounds(dt);
      this._updateFx(dt);
      this._updateMusic(dt);
    }

    /** Töne im Flug - nur vom aktuellen Multiplikator abhängig (verraten nichts). */
    _updateSounds(dt) {
      if (!(this.state === FLY || this.state === COAST) || this.round === null) return;
      const m = L.multAt(this.round.t);
      const w = Math.min(4, L.worldOf(m));
      if (w > this.worldSeen) {
        this.worldSeen = w;
        this.worldLabel = [W.nameKey(w), 2.4];
        // "triangle" kennt audio.tone (Python wie Web) nicht - es klingt wie dort als Rauschen
        this.tone(392.0 * Math.pow(2, (w * 2) / 12.0), 0.22, "triangle", 0.18);
      }
      while (this.toneI < TONE_AT.length && m >= TONE_AT[this.toneI]) {
        this.tone(523.25 * Math.pow(2, (this.toneI * 2) / 12.0), 0.12, "sine", 0.22);
        this.toneI += 1;
      }
      const sq = MU.squeak(m);
      if (sq !== null) {
        this.squeakT -= dt;
        if (this.squeakT <= 0) {
          const [freq, gap] = sq;
          this.tone(freq, 0.09, "triangle", 0.10);
          this.squeakT = gap;
        }
      }
    }

    _updateFx(dt) {
      const k = this.k;
      this.particles = this.particles.filter((p) => {
        p.t += dt;
        p.vy += p.g * dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        return p.t < p.life;
      });
      this.coinsFx = this.coinsFx.filter((c) => {
        c.vy += 900 * k * dt;
        c.x += c.vx * dt;
        c.y += c.vy * dt;
        c.spin += c.vs * dt;
        return c.y < this.height + 30;
      });
      for (const p of this.popups) p.t += dt;
      this.popups = this.popups.filter((p) => p.t < p.dur);
      const bottom = this.stage.bottom + 60 * k;
      this.chutes = this.chutes.filter((c) => {
        c.t += dt;
        if (c.t < 0.35) {
          c.vy += 520 * k * dt;
        } else {
          c.vy += (60 * k - c.vy) * Math.min(1.0, dt * 3.0);
          c.vx *= Math.max(0.0, 1.0 - dt * 0.8);
        }
        c.x += (c.vx + Math.sin(c.t * 2.2 + c.slot) * 18 * k) * dt;
        c.y += c.vy * dt;
        return c.y < bottom && c.t < 9.0;
      });
      this.fallers = this.fallers.filter((f) => {
        f.t += dt;
        f.vy += 900 * k * dt;
        f.x += f.vx * dt;
        f.y += f.vy * dt;
        return f.y < bottom;
      });
      this.shreds = this.shreds.filter((p) => {
        p.t += dt;
        p.vy += 420 * k * dt;
        p.vx *= Math.max(0.0, 1.0 - dt * 1.2);
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        return p.t < p.life;
      });
      if (this.popFx !== null) {
        this.popFx.t += dt;
        if (this.popFx.t > 0.6) this.popFx = null;
      }
      if (this.basketFx !== null) {
        const b = this.basketFx;
        b.t += dt;
        b.vy += 900 * k * dt;
        b.y += b.vy * dt;
        if (b.y > bottom) this.basketFx = null;
      }
    }

    /** Spannungsmusik: Stufe je Höhenwelt, Wechsel nur auf dem Schlag. */
    _updateMusic(dt) {
      this.musT += dt;
      if (!this.musicOn) return;
      const st = this.state;
      let want = 0, vol = 0.45;
      if (st === END) {
        if (this.end === "moon") {
          want = 5;
          vol = 1.0;
        } else {
          want = null;
          vol = 0.0;
        }
      } else if ((st === FLY || st === COAST) && this.round !== null) {
        want = MU.stageOf(L.multAt(this.round.t));
        vol = st === FLY ? 1.0 : 0.75;
      } else if (st === COUNT) {
        want = 0;
        vol = 0.8;
      }
      if (want === null) {
        if (this.musStage !== null) {
          PG.audio.stopMusic(this);
          this.musStage = null;
        }
        return;
      }
      let guard = 0;
      while (!MU.ready(want) && guard < MU.STAGES) {
        if (!MU.prerenderStep(want)) break;
        guard += 1;
      }
      if (MU.ready(want)) {
        const playing = PG.audio.musicPlaying(this);
        let sw = this.musStage === null || !playing;
        if (!sw && want !== this.musStage) {
          const beat = MU.beatSeconds(this.musStage);
          sw = trunc(this.musT / beat) !== trunc((this.musT - dt) / beat);
        }
        if (sw && (want !== this.musStage || !playing) && want !== this._musFail) {
          if (PG.audio.music(MU.voices(want), this, vol)) {
            this.musStage = want;
            this.musT = 0.0;
            this.musVol = vol;
            this._musFail = null;
          } else {
            this._musFail = want; // kein Audio: nicht jedes Bild neu versuchen
          }
        }
      }
      if (this.musStage !== null && vol !== this.musVol) {
        PG.audio.setMusicVolume(vol);
        this.musVol = vol;
      }
      // die nächste Stufe vorbereiten: Python rendert alle sechs in ruhigen Bildern;
      // hier liegen höchstens drei im Speicher, die nächste entsteht in kleinen
      // Häppchen - deshalb auch im Flug (ohne Ruckler, siehe lamacrash_music.js)
      MU.prerenderStep(want);
    }

    // ===================================================== Text-Helfer
    // Höhen wie in pygame-ce: font.get_height() ist dort genau die Pixelgröße
    // (font.px), das gerenderte Textbild ist höher (font.height = h der Objekte
    // aus _fit) - wie in lamamines.js.
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
      if (!img.s) return R(x0, y0, w, h);
      return ui.text(ctx, img.s, trunc(x0), trunc(y0), img.f, img.col, "topleft", alpha);
    }
    _text(ctx, str, x, y, font, col, anchor = "topleft", alpha) {
      str = String(str);
      return this._put(ctx, { s: str, f: font, col, w: font.width(str), h: font.height }, x, y, anchor, alpha);
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
    /** Bild (Canvas mit lw/lh) mittig auf (x, y) - wie blit(img, img.get_rect(center=...)). */
    _blitCenter(ctx, img, x, y) {
      D.blit(ctx, img, trunc(x) - fdiv(img.lw, 2), trunc(y) - fdiv(img.lh, 2));
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      const g = ctx.createLinearGradient(0, 0, 0, this.height);
      g.addColorStop(0, ui.col([22, 16, 34]));
      g.addColorStop(1, ui.col([8, 6, 14]));
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, this.width, this.height);
      this._drawStage(ctx);
      this._drawPanel(ctx);
      this._drawHud(ctx);
      this._drawTools(ctx);
      this._drawHistory(ctx);
      this._drawStrip(ctx);
      this._drawFx(ctx);
      if (this.banner) this._drawBanner(ctx);
      if (this.window === "auto") this._drawAuto(ctx);
      else if (this.window === "stats" || this.window === "fair") this._drawWindow(ctx);
      if (this.toast) this._drawToast(ctx);
      this._drawTooltip(ctx);
      if (this.broke) this._drawBroke(ctx);
    }

    _shakeOff() {
      if (this.shake <= 0) return [0, 0];
      const a = (8 * this.k * this.shake) / SHAKE_T;
      return [trunc(Math.sin(this.time * 71) * a), trunc(Math.cos(this.time * 53) * a * 0.7)];
    }

    /** Plätze im Korb: [[Halstuch, Pose] oder null] - je Einsatz ein Lama. */
    _riders() {
      const out = [];
      if (this.round !== null) {
        this.round.bets.forEach((b, i) => out.push(L.riding(b) > 0 ? [CARD_COL[i], "stand"] : null));
      } else if (this.state === END && this.last !== null) {
        this.last.rnd.bets.forEach((b, i) => {
          const keep = this.last.end === "moon" && (L.riding(b) > 0 || b.xm === L.MAX_MULT100);
          out.push(keep ? [CARD_COL[i], "hop"] : null);
        });
      } else {
        for (const i of this._active()) out.push([CARD_COL[i], "stand"]);
      }
      return out;
    }

    _flame() {
      if (this.state === COUNT || this.flameT > 0) return trunc(this.time * 14) % 6;
      if ((this.state === FLY || this.state === COAST) && this.time % 3.1 < 0.45) return trunc(this.time * 14) % 6;
      return -1;
    }

    _drawStage(ctx) {
      const st = this.stage;
      const [ox, oy] = this._shakeOff();
      ctx.save();
      ctx.beginPath();
      ctx.rect(st.x, st.y, st.w, st.h);
      ctx.clip();
      const alt = this._alt();
      const moon = this.state === END && this.end === "moon";
      let world, share;
      if (moon) {
        world = 5;
        share = 0;
      } else {
        [world, share] = L.worldBlend(alt);
      }
      W.draw(ctx, st, world, share, alt, this.time);
      if (!moon) {
        const [xMax, yMax] = L.chartSpan(trunc(alt));
        const ylines = L.chartYlines(yMax).map((v) => [trunc(this._pt(0, v, xMax, yMax)[1]) + oy, this._fmtShort(v)]);
        const xlines = L.chartXlines(xMax).map((v) => [trunc(this._pt(v, 100, xMax, yMax)[0]) + ox, fdiv(v, L.TPS) + "s"]);
        D.drawGrid(ctx, this.plot.move(ox, oy), ylines, xlines, this.fsMicro);
        if (alt > 0) {
          const pts = [];
          const n = 48;
          for (let j = 0; j <= n; j++) {
            const tt = (alt * j) / n;
            const [x, y] = this._pt(tt, this._mAtF(tt), xMax, yMax);
            pts.push([x + ox, y + oy]);
          }
          const col = this.state === END ? ui.RED : ui.mix(this.accent, [255, 255, 255], 0.15);
          D.drawCurve(ctx, pts, col, this.plot.bottom + oy, Math.max(2, trunc(3 * this.k)));
        }
      }
      this._drawRig(ctx, ox, oy, moon);
      this._drawFlyers(ctx, ox, oy);
      this._drawCenter(ctx);
      ctx.restore();
      draw.rect(ctx, ui.mix(this.accent, ui.BORDER, 0.55), st, 1, Math.max(4, trunc(8 * this.k)));
    }

    _drawRig(ctx, ox, oy, moon) {
      if (this.state === END && this.end === "pop") return;
      const n = this._nRiders();
      const riders = this._riders().slice(0, n);
      const bw = D.basketW(this.rigBase);
      if (moon) {
        const [cx, rimY] = this._moonRig();
        const fl = D.flag(Math.max(12, trunc(this.rigBase * 0.7)), CARD_COL[0]);
        // fl.get_rect(bottomleft=(cx + bw // 2 + int(6k), rim_y + bw * 2 // 3))
        D.blit(ctx, fl, cx + fdiv(bw, 2) + trunc(6 * this.k), rimY + fdiv(bw * 2, 3) - fl.lh);
        this.envRect = D.drawRig(ctx, cx, rimY, this.rigBase, D.INFL_MAX, this.accent, riders, -1);
        return;
      }
      const [x, y] = this._rigPos();
      const m = this._curMult();
      const level = D.inflLevel(m);
      let sway = 0;
      if (this.state === FLY || this.state === COAST) sway = Math.sin(this.time * 1.7) * this.k * (1.5 + level * 0.35);
      this.envRect = D.drawRig(ctx, x + ox + sway, y + oy, this.rigBase, level, this.accent, riders, this._flame());
    }

    _drawFlyers(ctx, ox, oy) {
      for (const c of this.chutes) {
        const openF = Math.max(0, Math.min(1, (c.t - 0.25) / 0.4));
        if (c.kind === "llama") D.drawJumper(ctx, c.x + ox, c.y + oy, c.n, CARD_COL[c.slot], CHUTE_COL[c.slot], openF);
        else D.drawSack(ctx, c.x + ox, c.y + oy, c.n, CHUTE_COL[c.slot], openF);
      }
      const n = D.llamaSize(this.rigBase);
      for (const f of this.fallers) D.drawFaller(ctx, f.x + ox, f.y + oy, n, CARD_COL[f.slot], trunc(f.t * 10) % 8);
      if (this.basketFx !== null) {
        const b = this.basketFx;
        const bw = D.basketW(this.rigBase);
        const x = trunc(b.x) - fdiv(bw, 2) + ox;
        const y = trunc(b.y) + oy;
        D.blit(ctx, D.basket(bw, false), x, y);
        D.blit(ctx, D.basket(bw, true), x, y);
      }
      for (const p of this.shreds) {
        const pieces = D.shreds(p.w, this.accent);
        this._blitCenter(ctx, pieces[p.i % pieces.length], trunc(p.x) + ox, trunc(p.y) + oy);
      }
      if (this.popFx !== null) {
        const f = this.popFx;
        this._blitCenter(ctx, D.popStar(f.r, Math.min(5, trunc(f.t / 0.1))), f.x + ox, f.y + oy);
      }
      for (const p of this.particles) {
        const fr = p.t / p.life;
        const r = p.r * (1 - fr);
        if (r >= 0.5) draw.circle(ctx, p.col, [trunc(p.x), trunc(p.y)], Math.max(1, trunc(r)));
      }
    }

    /** Großer Multiplikator, Countdown bzw. Ergebnis mitten auf der Bühne. */
    _drawCenter(ctx) {
      const st = this.stage;
      const cx = cxOf(st);
      const cy = st.y + trunc(st.h * 0.30);
      const state = this.state;
      if (state === COUNT) {
        const seg = this.countLen / 3.0;
        const num = Math.max(1, Math.min(3, trunc(this.countT / seg) + 1));
        const ph = (this.countT % seg) / seg;
        const sc = 1.0 + 0.35 * ph;
        ctx.save();
        ctx.translate(cx, cy);
        ctx.scale(sc, sc);
        this._text(ctx, String(num), 0, 0, this.fHuge, CD.COL_GOLD, "center");
        ctx.restore();
        return;
      }
      if (state === FLY || state === COAST) {
        const m = this._curMult();
        const txt = this._fmt(m);
        const col = this._multCol(m);
        const fonts = [this.fHuge, this.fMult, this.fBig];
        const img = this._fit(txt, fonts, st.w - 16, col);
        const sh = this._fit(txt, fonts, st.w - 16, [12, 8, 20]);
        this._put(ctx, sh, cx + 2, cy + 3, "center");
        this._put(ctx, img, cx, cy, "center");
        const y = cy + fdiv(img.h, 2) + 2;
        let sub, scol;
        if (state === FLY && this.round !== null) {
          let now = 0;
          for (const b of this.round.bets) now += fdiv(L.riding(b) * m, 100);
          sub = t("lcx.win_now", { n: this._group(now) });
          scol = ui.GREEN;
        } else {
          sub = t("lcx.coast");
          scol = ui.TEXT_DIM;
        }
        this._put(ctx, this._fit(sub, [this.fBtn, this.fTiny], st.w - 16, scol), cx, y, "midtop");
        if (this.worldLabel) {
          const [key, left] = this.worldLabel;
          const wimg = this._fit(t(key), [this.fBig, this.fBtn, this.fTiny], st.w - 24, [255, 255, 255]);
          const bw = wimg.w + trunc(28 * this.k), bh = wimg.h + trunc(10 * this.k);
          const box = R(cx - fdiv(bw, 2), st.y + this.margin, bw, bh);
          const a = Math.min(1.0, (2.4 - left) / 0.25, left / 0.5);
          const alpha = trunc(255 * Math.max(0.0, a)) / 255;
          if (alpha > 0) {
            ctx.save();
            ctx.globalAlpha *= alpha;
            draw.rect(ctx, [10, 8, 18, 170], box, 0, fdiv(box.h, 2));
            draw.rect(ctx, [...this.accent, 220], box, 1, fdiv(box.h, 2));
            this._put(ctx, wimg, box.x + fdiv(box.w, 2), box.y + fdiv(box.h, 2), "center");
            ctx.restore();
          }
        }
        return;
      }
      if (state === END && this.last !== null) {
        const f = Math.min(1.0, this.endT / 0.18);
        if (this.last.end !== "pop") return; // Mondlandung: Banner
        const col = ui.RED;
        const timg = this._fit(t("lcx.popped"), [this.fBig, this.fBtn], st.w - 16, col);
        const vimg = this._fit(this._fmt(this.last.crash), [this.fHuge, this.fMult, this.fBig], st.w - 16, col);
        let vh = vimg.h;
        if (f < 1.0) {
          const sc = 0.7 + 0.3 * f;
          vh = Math.max(1, trunc(vimg.h * sc));
          ctx.save();
          ctx.translate(cx, cy);
          ctx.scale(sc, sc);
          this._put(ctx, vimg, 0, 0, "center");
          ctx.restore();
        } else {
          this._put(ctx, vimg, cx, cy, "center");
        }
        this._put(ctx, timg, cx, cy - fdiv(vh, 2), "midbottom");
        const res = this.last.paid - this.last.stake;
        const sub = t("lcx.round_net", { n: (res > 0 ? "+" : "") + this._group(res) });
        this._put(ctx, this._fit(sub, [this.fBtn, this.fTiny], st.w - 16, res >= 0 ? ui.GREEN : ui.TEXT_DIM), cx, cy + fdiv(vh, 2) + 2, "midtop");
        return;
      }
      if (state === IDLE && this.last !== null) {
        const txt = t("lcx.last_crash", { x: this._fmt(this.last.end === "pop" ? this.last.crash : L.MAX_MULT100) });
        this._put(ctx, this._fit(txt, [this.fSmall, this.fTiny], st.w - 16, ui.TEXT_DIM), cx, st.y + this.margin, "midtop");
      }
    }

    // ----- Seitenleiste -----------------------------------------------------------
    _drawPanel(ctx) {
      ui.drawPanel(ctx, this.panelR, { accentTop: this.accent });
      for (let i = 0; i < 2; i++) this._drawCard(ctx, i);
      this._drawInfo(ctx);
      this._drawHint(ctx);
    }

    /** Teil-Einsatz der laufenden bzw. letzten Runde für Karte i (oder null). */
    _cardBet(i) {
      if (this.round !== null) return i < this.round.bets.length ? this.round.bets[i] : null;
      if (this.last !== null && (this.state === END || this.auto !== null)) {
        const bets = this.last.rnd.bets;
        return i < bets.length ? bets[i] : null;
      }
      return null;
    }

    _drawCard(ctx, i) {
      const c = this.cards[i];
      const r = c.rect;
      const k = this.k;
      const col = CARD_COL[i];
      const hot = this.mouse;
      const editing = this.state === IDLE && this.round === null && this.auto === null;
      const bet = this._cardBet(i);
      let active = editing || bet === null ? this._active().includes(i) : true;
      if (!editing && bet === null) active = false;
      const rad = Math.max(5, trunc(8 * k));
      draw.rect(ctx, ui.mix(ui.PANEL, [0, 0, 0], 0.28), r, 0, rad);
      const edge = active && (!editing || this.sel === i) ? col : active ? ui.mix(col, ui.BORDER, 0.6) : ui.BORDER;
      draw.rect(ctx, edge, r, editing && this.sel === i && active ? 2 : 1, rad);
      // Titelzeile: Farbpunkt, "Einsatz n", rechts Schalter bzw. Zustand
      const ty = c.titleY, th = c.titleH;
      const dotR = Math.max(3, fdiv(th, 4));
      const dx = r.x + Math.max(6, trunc(9 * k));
      draw.circle(ctx, active ? col : ui.TEXT_FAINT, [dx, ty + fdiv(th, 2)], dotR);
      let title = t("lcx.bet_n", { n: i + 1 });
      if (bet !== null) title += "  ·  " + this._group(bet.b);
      const rightW = i === 1 && editing ? c.sw.w + 6 : 0;
      this._put(ctx, this._fit(title, [this.fTiny, this.fMicro], r.w - rightW - trunc(24 * k), active ? ui.TEXT : ui.TEXT_DIM),
        dx + dotR + 5, ty + fdiv(th, 2), "midleft");
      if (i === 1 && editing) {
        this._switch(ctx, c.sw, this.two, c.sw.collidepoint(hot));
      } else if (bet !== null) {
        const [badge, bcol] = this._betBadge(bet);
        this._put(ctx, this._fit(badge, [this.fMicro], fdiv(r.w, 2), bcol), r.right - Math.max(5, trunc(8 * k)), ty + fdiv(th, 2), "midright");
      }
      const midY = fdiv(c.rowA.y + c.rowB.bottom, 2);
      if (editing) {
        if (!active) {
          this._put(ctx, this._fit(t("lcx.second_hint"), [this.fTiny, this.fMicro], r.w - 12, ui.TEXT_FAINT), cxOf(r), midY, "center");
          return;
        }
        const onM = this.bets[i] > L.BETS[0];
        const onP = this.bets[i] < L.BETS[L.BETS.length - 1];
        this._btn(ctx, c.minus, "–", { on: onM, hot: c.minus.collidepoint(hot) });
        this._btn(ctx, c.plus, "+", { on: onP, hot: c.plus.collidepoint(hot) });
        this._valueBox(ctx, c.bet, this._group(this.bets[i]), this._total() > bank.balance() ? ui.RED : ui.GOLD);
        this._check(ctx, c.auto, this.autos[i], c.auto.collidepoint(hot));
        const on = this.autos[i];
        this._btn(ctx, c.tl, "", { on: on && this.targets[i] > L.TARGETS[0], hot: c.tl.collidepoint(hot), icon: "left" });
        this._btn(ctx, c.tr, "", { on: on && this.targets[i] < L.TARGETS[L.TARGETS.length - 1], hot: c.tr.collidepoint(hot), icon: "right" });
        const txt = on ? this._fmt(this.targets[i]) : t("lcx.target_off");
        this._valueBox(ctx, c.tv, txt, on ? col : ui.TEXT_FAINT, t("lcx.target"));
        return;
      }
      if (bet === null) {
        this._put(ctx, this._fit("–", [this.fBtn], r.w - 12, ui.TEXT_FAINT), cxOf(r), midY, "center");
        return;
      }
      // Runde läuft bzw. ist gerade vorbei
      const ra = c.rowA, rb = c.rowB;
      const m = this._curMult();
      const riding = L.riding(bet);
      let mark = null, left, lcol;
      if (riding > 0 && this.state !== END) {
        left = t("lcx.win_now", { n: this._group(fdiv(riding * m, 100)) });
        lcol = ui.GREEN;
      } else if (bet.xm) {
        left = this._fmt(bet.xm);
        lcol = CD.COL_GOLD;
        mark = "check";
      } else if (this.state === END) {
        left = t("lcx.lost");
        lcol = ui.RED;
        mark = "cross";
      } else {
        left = "…";
        lcol = ui.TEXT_DIM;
      }
      const right = bet.p > 0 ? "+" + this._group(bet.p) : bet.a ? t("lcx.target_at", { x: this._fmt(bet.a) }) : "";
      const rimg = this._fit(right, [this.fTiny, this.fMicro], fdiv(ra.w, 2), bet.p > 0 ? CD.COL_GOLD : ui.TEXT_DIM);
      let lx = ra.x;
      if (mark !== null) {
        const isz = Math.max(8, trunc(ra.h * 0.62));
        D.blit(ctx, D.icon(mark, isz, lcol), lx, cyOf(ra) - fdiv(isz, 2));
        lx += isz + 4;
      }
      const limg = this._fit(left, [this.fBtn, this.fTiny, this.fMicro], ra.right - lx - rimg.w - 6, lcol);
      this._put(ctx, limg, lx, cyOf(ra), "midleft");
      this._put(ctx, rimg, ra.right, cyOf(ra), "midright");
      if (this.state === FLY && riding > 0) {
        const can = m >= L.MIN_CASH;
        this._btn(ctx, c.cash, t("lcx.cash") + " " + this._group(fdiv(riding * m, 100)), { primary: true, on: can, hot: c.cash.collidepoint(hot) });
        this._btn(ctx, c.halfc, "", { on: can && !bet.hm, hot: c.halfc.collidepoint(hot), icon: "half" });
      } else {
        let info = "";
        if (bet.hm) info = t("lcx.half_at", { x: this._fmt(bet.hm) });
        else if (this.state === COUNT) info = t("lcx.ready");
        this._put(ctx, this._fit(info, [this.fTiny, this.fMicro], rb.w, ui.TEXT_DIM), rb.x, cyOf(rb), "midleft");
      }
    }

    _betBadge(bet) {
      if (L.riding(bet) > 0 && this.state !== END) return [t("lcx.riding"), ui.GREEN];
      if (bet.xm) return [t("lcx.out"), CD.COL_GOLD];
      return [t("lcx.lost"), ui.RED];
    }
    _valueBox(ctx, r, text, col) {
      const rad = Math.max(4, fdiv(r.h, 4));
      draw.rect(ctx, [16, 12, 26], r, 0, rad);
      draw.rect(ctx, ui.BORDER, r, 1, rad);
      this._put(ctx, this._fit(text, [this.fBtn, this.fTiny, this.fMicro], r.w - 6, col), cxOf(r), cyOf(r), "center");
    }
    _switch(ctx, r, on, hot) {
      draw.rect(ctx, on ? CD.COL_GOLD : [60, 60, 76], r, 0, fdiv(r.h, 2));
      if (hot) draw.rect(ctx, ui.BORDER_LIGHT, r, 1, fdiv(r.h, 2));
      const kr = Math.max(2, fdiv(r.h, 2) - 2);
      const kx = on ? r.right - kr - 3 : r.x + kr + 3;
      draw.circle(ctx, [255, 255, 255], [kx, cyOf(r)], kr);
    }
    _check(ctx, r, on, hot) {
      const rad = Math.max(4, fdiv(r.h, 4));
      draw.rect(ctx, hot ? ui.BTN_SEL : ui.BTN, r, 0, rad);
      draw.rect(ctx, on ? CD.COL_GOLD : ui.BORDER_LIGHT, r, on ? 2 : 1, rad);
      if (on) this._blitCenter(ctx, D.icon("check", Math.max(8, trunc(r.h * 0.62)), CD.COL_GOLD), cxOf(r), cyOf(r));
    }

    /** Zeilen des Info-Kastens: [[Beschriftung, Wert, Farbe]]. */
    _infoRows() {
      if (this.state === END && this.last !== null) {
        const last = this.last;
        const net = last.paid - last.stake;
        const x = last.end === "pop" ? last.crash : L.MAX_MULT100;
        return [[t("lcx.crash_at"), this._fmt(x), last.end === "pop" ? ui.RED : CD.COL_GOLD],
          [t("chip.st_paid"), this._group(last.paid), last.paid ? CD.COL_GOLD : ui.TEXT_DIM],
          [t("chip.st_net"), (net > 0 ? "+" : "") + this._group(net), net >= 0 ? ui.GREEN : ui.RED]];
      }
      if (this.round !== null && (this.state === FLY || this.state === COAST || this.state === COUNT)) {
        const m = this._curMult();
        const secs = fdiv(this.round.t, L.TPS);
        return [[t("lcx.world"), t(W.nameKey(Math.min(4, L.worldOf(m)))), this.accent],
          [t("lcx.flight"), fdiv(secs, 60) + ":" + String(secs % 60).padStart(2, "0"), ui.TEXT],
          [t("chip.st_paid"), this._group(L.paidTotal(this.round)), CD.COL_GOLD]];
      }
      const rows = [[t("lcx.total"), this._group(this._total()), this._total() > bank.balance() ? ui.RED : ui.TEXT]];
      for (const i of this._active()) {
        if (this.autos[i]) {
          rows.push([t("lcx.chance", { x: this._fmt(this.targets[i]) }), L.fmtChance(this.targets[i], this._sep()) + " %", CARD_COL[i]]);
        }
      }
      rows.push([t("chip.max_win"), this._fmt(L.MAX_MULT100), CD.COL_GOLD]);
      return rows;
    }

    _drawInfo(ctx) {
      const [x0, w] = this.panelInner;
      let y = this.infoY;
      let rows = this._infoRows();
      const avail = this.hintY - Math.max(4, trunc(6 * this.k)) - y;
      const minLh = this.fTiny.px + 2;
      const maxLh = trunc(this.fBtn.px * 1.6);
      while (rows.length && fdiv(avail, rows.length) < minLh) rows = rows.slice(0, -1);
      if (!rows.length) return;
      const lh = Math.min(maxLh, fdiv(avail, rows.length));
      for (const [lab, val, col] of rows) {
        const vimg = this._fit(val, [this.fBtn, this.fTiny], fdiv(w, 2), col);
        const limg = this._fit(lab, [this.fSmall, this.fTiny, this.fMicro], w - vimg.w - 8, ui.TEXT_DIM);
        this._put(ctx, limg, x0, y + fdiv(lh, 2), "midleft");
        this._put(ctx, vimg, x0 + w, y + fdiv(lh, 2), "midright");
        y += lh;
      }
    }

    _drawHint(ctx) {
      if (this.hintY >= this.panelR.bottom - this.ppad) return;
      const [x0, w] = this.panelInner;
      let text;
      if (this.auto !== null) text = t("chip.auto_left", { n: this.auto.left });
      else if (this.state === FLY) text = t("lcx.hint_fly");
      else if (this.state === COAST) text = t("lcx.hint_coast");
      else text = t("lcx.hint_idle");
      let y = this.hintY;
      for (const line of this._wrap(text, this.fMicro, w).slice(0, 2)) {
        this._text(ctx, line, x0, y, this.fMicro, ui.TEXT_FAINT);
        y += this.fMicro.px + 1;
      }
    }

    // ----- Kopfzeile, Werkzeuge, Verlauf ---------------------------------------------
    _drawHud(ctx) {
      const W_ = this.width;
      const g = ctx.createLinearGradient(0, 0, 0, this.hudH);
      g.addColorStop(0, "rgba(10,8,16," + (210 / 255).toFixed(3) + ")");
      g.addColorStop(1, "rgba(10,8,16," + (150 / 255).toFixed(3) + ")");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W_, this.hudH);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [0, this.hudH - 1], [W_, this.hudH - 1], 1);
      const cy = fdiv(this.hudH, 2);
      const pad = this.margin;
      const coinR = Math.max(6, trunc(this.hudH * 0.26));
      const coin = CD.symbolCanvas("coin", coinR * 2 + 4);
      ctx.drawImage(coin, pad - 2, cy - fdiv(coin.lh, 2), coin.lw, coin.lh);
      const leftTxt = t("cas.chips") + ": " + this._group(pyRound(this.shownChips));
      const lw = this.fBig.width(leftTxt);
      this._text(ctx, leftTxt, pad + coinR * 2 + 6, cy, this.fBig, ui.GOLD, "midleft");
      const rightTxt = t("cas.record") + ": " + this._group(this.score) + "   ·   " + t("cas.credits", { n: bank.refills() });
      const rf = this.fSmall.width(rightTxt) > W_ * 0.42 ? this.fTiny : this.fSmall;
      const rw = rf.width(rightTxt);
      this._text(ctx, rightTxt, W_ - pad, cy, rf, ui.TEXT_DIM, "midright");
      if (this.round === null) return;
      const mid = t("cas.bet") + ": " + this._group(L.stakeTotal(this.round));
      const x0 = pad + coinR * 2 + 6 + lw + pad;
      const x1 = W_ - pad - rw - pad;
      const mf = this.fSmall.width(mid) > x1 - x0 ? this.fTiny : this.fSmall;
      if (mf.width(mid) <= x1 - x0) this._text(ctx, mid, fdiv(x0 + x1, 2), cy, mf, ui.TEXT, "center");
    }

    _toolActive(name) {
      if (name === "turbo") return this.turbo;
      if (name === "music") return this.musicOn;
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
        if (name === "auto" && this.auto !== null) edge = ui.mix(this.accent, [255, 255, 255], 0.5 * (0.5 + 0.5 * Math.sin(this.time * 5)));
        draw.rect(ctx, edge, r, active ? 2 : 1, rad);
        const col = active ? this.accent : ui.TEXT;
        const icon = name !== "music" || this.musicOn ? name : "mute";
        this._blitCenter(ctx, D.icon(icon, Math.max(8, trunc(r.h * 0.6)), col), r.x + fdiv(r.h, 2), cyOf(r));
        if (this.toolLabels) {
          this._text(ctx, t(TOOL_LABEL[name]), r.x + r.h - Math.max(2, fdiv(r.h, 8)), cyOf(r), this.fTiny, active || h ? ui.TEXT : ui.TEXT_DIM, "midleft");
        }
      }
    }

    _drawTooltip(ctx) {
      if (this.window !== null) return;
      for (const [name, r] of this.rTools) {
        if (r.collidepoint(this.mouse)) {
          PG.charts.tooltip(ctx, this.mouse, [t(TOOL_LABEL[name]) + " (" + TOOL_KEY[name] + ")"]);
          return;
        }
      }
      if (this.rHalf.collidepoint(this.mouse) && this.state === FLY) PG.charts.tooltip(ctx, this.mouse, [t("lcx.half_tip") + " (H)"]);
    }

    /** Letzte Platzpunkte als Pillen oben rechts (neueste rechts). */
    _drawHistory(ctx) {
      let x = this.width - this.margin;
      const y = this.rTools[0][1].y;
      const h = this.toolH;
      const ph = trunc(h * 0.78);
      const limit = this.toolsRight + this.margin;
      for (let j = this.hist.length - 1; j >= 0; j--) {
        const m100 = this.hist[j];
        const label = this._fmtShort(m100);
        const w = this.fTiny.width(label) + trunc(12 * this.k) + 6;
        if (x - w < limit) break;
        D.blit(ctx, D.pill(w, ph, m100, label, this.fsTiny), x - w, y + fdiv(h - ph, 2));
        x -= w + 4;
      }
    }

    // ----- Bedienleiste -----------------------------------------------------------
    _drawStrip(ctx) {
      const sr = this.strip;
      const g = ctx.createLinearGradient(0, sr.y, 0, sr.bottom);
      g.addColorStop(0, "rgba(10,8,16," + (170 / 255).toFixed(3) + ")");
      g.addColorStop(1, "rgba(10,8,16," + (230 / 255).toFixed(3) + ")");
      ctx.fillStyle = g;
      ctx.fillRect(sr.x, sr.y, sr.w, sr.h);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [sr.x, sr.y], [sr.right, sr.y], 2);
      const hot = this.mouse;
      const st = this.state;
      // Gesamteinsatz
      const r = this.rTotal;
      draw.rect(ctx, [16, 12, 26], r, 0, Math.max(6, fdiv(r.h, 4)));
      draw.rect(ctx, ui.BORDER, r, 1, Math.max(6, fdiv(r.h, 4)));
      this._put(ctx, this._fit(t("lcx.total"), [this.fTiny, this.fMicro], r.w - 4, ui.TEXT_DIM), cxOf(r), r.y + 2, "midtop");
      const total = this.round !== null ? L.stakeTotal(this.round) : this._total();
      const col = this.round === null && total > bank.balance() ? ui.RED : ui.GOLD;
      this._put(ctx, this._fit(this._group(total), [this.fBtn, this.fTiny], r.w - 4, col), cxOf(r), r.bottom - 3, "midbottom");
      // Hälfte
      const canHalf = st === FLY && this.round !== null && L.multAt(this.round.t) >= L.MIN_CASH
        && this.round.bets.some((b) => L.riding(b) > 0 && !b.hm);
      this._btn(ctx, this.rHalf, this.halfLabel ? t("lcx.half_all") : "", { on: canHalf, hot: this.rHalf.collidepoint(hot), icon: "half" });
      // Status in der Mitte
      const [x0, x1] = this.statusX;
      let status = "", scol = ui.TEXT_DIM;
      if (this.auto !== null) {
        status = t("chip.auto_left", { n: this.auto.left });
        scol = this.accent;
      } else if (st === COUNT) {
        status = t("lcx.ready");
      } else if (st === COAST) {
        status = t("lcx.coast");
      }
      if (status && x1 - x0 > 30) {
        this._put(ctx, this._fit(status, [this.fBtn, this.fTiny, this.fMicro], x1 - x0, scol), fdiv(x0 + x1, 2), cyOf(this.rGo), "center");
      }
      // Hauptknopf
      const go = this.rGo;
      if (st === FLY && this.round !== null) {
        const m = L.multAt(this.round.t);
        let now = 0;
        for (const b of this.round.bets) now += fdiv(L.riding(b) * m, 100);
        this._btn(ctx, go, t("lcx.cash_all") + " " + this._group(now), { primary: true, on: m >= L.MIN_CASH, hot: go.collidepoint(hot) });
      } else if (st === COAST) {
        this._btn(ctx, go, t("lcx.ff"), { primary: !this.ff && this.auto === null, on: true, hot: go.collidepoint(hot), icon: "ff" });
      } else if (this.auto !== null) {
        this._btn(ctx, go, t("chip.auto_stop"), { primary: true, hot: go.collidepoint(hot), danger: true });
      } else {
        const ready = st === IDLE || (st === END && this.endT >= READ_T);
        this._btn(ctx, go, t("lcx.play"), { primary: true, on: ready && !this.broke && this.window === null && this.round === null,
          hot: go.collidepoint(hot), icon: "play" });
      }
    }

    _btn(ctx, r, label, { primary = false, on = true, hot = false, active = false, small = false, icon = null, danger = false } = {}) {
      const rad = Math.max(5, fdiv(r.h, 4));
      let col;
      if (primary && on) {
        const base = ui.mix(danger ? ui.RED : this.accent, [0, 0, 0], danger ? 0.1 : 0.30);
        const fill = ui.mix(base, [255, 255, 255], hot ? 0.12 : 0.0);
        draw.rect(ctx, ui.mix(base, [0, 0, 0], 0.4), r.move(0, 2), 0, rad);
        draw.rect(ctx, fill, r, 0, rad);
        const pulse = 0.5 + 0.5 * Math.sin(this.time * 2.4);
        draw.rect(ctx, ui.mix(CD.COL_GOLD, [255, 255, 255], 0.35 * pulse), r, 2, rad);
        col = [255, 255, 255];
      } else {
        draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        col = on ? ui.TEXT : ui.TEXT_FAINT;
      }
      const fonts = small ? [this.fTiny, this.fMicro] : [this.fBtn, this.fTiny, this.fMicro];
      if (icon !== null) {
        const isz = Math.max(8, trunc(r.h * 0.56));
        const img = D.icon(icon, isz, col);
        if (label) {
          const limg = this._fit(label, fonts, Math.max(10, r.w - isz - 14), col);
          const total = isz + 5 + limg.w;
          const ix = cxOf(r) - fdiv(total, 2);
          D.blit(ctx, img, ix, cyOf(r) - fdiv(img.lh, 2));
          this._put(ctx, limg, ix + isz + 5, cyOf(r), "midleft");
        } else {
          this._blitCenter(ctx, img, cxOf(r), cyOf(r));
        }
        return;
      }
      if (label) this._put(ctx, this._fit(label, fonts, r.w - 6, col), cxOf(r), cyOf(r), "center");
    }

    // ----- Effekte, Hinweise -------------------------------------------------------
    _drawFx(ctx) {
      for (const c of this.coinsFx) {
        const w = Math.max(2, trunc(c.r * 2 * Math.abs(Math.cos(c.spin))));
        const h = trunc(c.r * 2);
        const x0 = trunc(c.x) - fdiv(w, 2), y0 = trunc(c.y) - fdiv(h, 2);
        draw.ellipse(ctx, CD.COL_GOLD_D, [x0 + 1, y0 + 1, w, h]);
        draw.ellipse(ctx, [246, 196, 60], [x0, y0, w, h]);
        if (w > 6) draw.ellipse(ctx, [255, 236, 150], inflate(R(x0, y0, w, h), -fdiv(w, 2), -fdiv(h, 2)));
      }
      for (const p of this.popups) {
        const f = p.t / p.dur;
        const alpha = f > 0.6 ? trunc(255 * (1 - (f - 0.6) / 0.4)) / 255 : undefined;
        const y = p.y - 40 * this.k * (1 - (1 - f) * (1 - f));
        const w = this.fBig.width(p.text), h = this.fBig.height;
        const r = R(trunc(p.x) - fdiv(w, 2), trunc(y) - fdiv(h, 2), w, h).clamp(this.stage);
        ui.text(ctx, p.text, r.x + 1, r.y + 1, this.fBig, [10, 6, 16], "topleft", alpha);
        ui.text(ctx, p.text, r.x, r.y, this.fBig, p.color, "topleft", alpha);
      }
    }

    _drawBanner(ctx) {
      const [title, sub, left, dur] = this.banner;
      const st = this.stage;
      const k = this.k;
      const grow = Math.min(1.0, (dur - left) / 0.18);
      const pulse = 0.5 + 0.5 * Math.sin(this.time * 8);
      const col = ui.mix(CD.COL_GOLD, [255, 255, 255], 0.4 * pulse);
      const timg = this._fit(title, [this.fHuge, this.fMult, this.fBig], st.w - 24, col);
      const simg = this._fit(sub, [this.fBig, this.fBtn, this.fTiny], st.w - 24, ui.TEXT);
      const pw = Math.min(st.w - 8, Math.max(timg.w, simg.w) + trunc(40 * k));
      const ph = timg.h + simg.h + trunc(26 * k);
      const w = trunc(pw * (0.7 + 0.3 * grow)), h = trunc(ph * (0.7 + 0.3 * grow));
      const panel = R(cxOf(st) - fdiv(w, 2), st.y + trunc(st.h * 0.32) - fdiv(h, 2), w, h);
      ctx.save();
      if (left < 0.4) ctx.globalAlpha *= Math.max(0, trunc((255 * left) / 0.4)) / 255;
      ui.drawPanel(ctx, panel, { accentTop: CD.COL_GOLD });
      if (grow >= 1.0) {
        this._put(ctx, timg, cxOf(panel), panel.y + trunc(8 * k), "midtop");
        this._put(ctx, simg, cxOf(panel), panel.bottom - trunc(8 * k), "midbottom");
      }
      ctx.restore();
    }

    _drawToast(ctx) {
      const [text, left] = this.toast;
      const img = this._fit(text, [this.fSmall, this.fTiny], this.width - 40, ui.TEXT);
      const bw = img.w + 24, bh = img.h + 12;
      const box = R(cxOf(this.stage) - fdiv(bw, 2), this.stage.bottom - this.margin - bh, bw, bh).clamp(R(4, 0, this.width - 8, this.height));
      ctx.save();
      if (left < 0.3) ctx.globalAlpha *= Math.max(0, trunc((255 * left) / 0.3)) / 255;
      draw.rect(ctx, [12, 8, 20, 225], box, 0, fdiv(box.h, 2));
      draw.rect(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.4), box, 1, fdiv(box.h, 2));
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
      this._put(ctx, this._fit(t("cas.broke"), [this.fHuge, this.fBig], pw - 16, ui.RED), cx, trunc(panel.y + ph * 0.28), "center");
      [[t("cas.broke_sub"), ui.TEXT_DIM], [t("cas.broke_restart", { n: bank.START_CHIPS }), ui.TEXT]].forEach(([txt, col], i) => {
        this._put(ctx, this._fit(txt, [this.fSmall, this.fTiny, this.fMicro], pw - 16, col), cx, trunc(panel.y + ph * (0.58 + 0.2 * i)), "center");
      });
    }

    // ----- Auto-Fenster (über der Bühne) --------------------------------------------
    _drawAuto(ctx) {
      const r = this.autoR;
      ui.drawPanel(ctx, r, { accentTop: this.accent });
      const k = this.k;
      const pad = Math.max(6, trunc(10 * k));
      const x0 = r.x + pad, w = r.w - 2 * pad;
      const hot = this.mouse;
      const ac = this.rAutoClose;
      const icon = D.icon("auto", Math.max(10, ac.h - 4), this.accent);
      D.blit(ctx, icon, x0, cyOf(ac) - fdiv(icon.lh, 2));
      this._put(ctx, this._fit(t("chip.auto_title"), [this.fBig, this.fBtn], ac.x - x0 - icon.lw - 10, ui.TEXT), x0 + icon.lw + 6, cyOf(ac), "midleft");
      this._btn(ctx, ac, "", { hot: ac.collidepoint(hot), icon: "close" });
      const labels = [t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss")];
      const values = [this.autoRounds, this.autoWin, this.autoLoss];
      const running = this.auto !== null;
      this.autoRows.forEach(([ly, rects], row) => {
        this._put(ctx, this._fit(labels[row], [this.fTiny, this.fMicro], w, this.autoFocus === row ? ui.TEXT : ui.TEXT_DIM), x0, ly);
        const opts = AUTO_OPTS[row];
        rects.forEach((rr, j) => {
          const v = opts[j];
          const lab = row === 0 ? String(v) : v === 0 ? t("chip.auto_off") : CD.fmtAmount(v);
          this._btn(ctx, rr, lab, { active: v === values[row], on: !running || v === values[row], hot: rr.collidepoint(hot) && !running, small: true });
        });
      });
      const tr = this.rAutoTurbo;
      this._btn(ctx, tr, t("chip.turbo"), { active: this.turbo, hot: tr.collidepoint(hot) || this.autoFocus === 3, icon: "turbo" });
      const parts = this._active().map((i) => this._fmt(this.targets[i]));
      const txt = t("lcx.auto_sum", { n: this.autoRounds, b: this._group(this._total()), x: parts.join(" / ") });
      let y = this.autoSumY;
      const scol = this._total() > bank.balance() ? ui.RED : ui.TEXT_DIM;
      for (const line of this._wrap(txt, this.fTiny, w).slice(0, 2)) {
        this._text(ctx, line, x0, y, this.fTiny, scol);
        y += this.fTiny.px;
      }
      for (const line of this._wrap(t("lcx.auto_note"), this.fMicro, w)) {
        if (y + this.fMicro.px >= this.rAutoStart.y - 4) break;
        this._text(ctx, line, x0, y, this.fMicro, ui.TEXT_FAINT);
        y += this.fMicro.px;
      }
      if (running) this._btn(ctx, this.rAutoStart, t("chip.auto_stop"), { primary: true, danger: true, hot: this.rAutoStart.collidepoint(hot) });
      else this._btn(ctx, this.rAutoStart, t("chip.auto_start"), { primary: true, on: !this.broke, hot: this.rAutoStart.collidepoint(hot) });
    }

    // ----- Statistik- und Fairness-Fenster ---------------------------------------------
    _windowRect() {
      const W_ = this.width, H = this.height, k = this.k, m = this.margin;
      let w, h;
      if (this.window === "fair") {
        w = Math.min(W_ - 2 * m, trunc(720 * k));
        h = Math.min(H - 2 * m, trunc(540 * k));
      } else {
        w = Math.min(W_ - 2 * m, trunc(680 * k));
        h = Math.min(H - 2 * m, trunc(500 * k));
      }
      const rect = R(fdiv(W_, 2) - fdiv(w, 2), fdiv(H, 2) - fdiv(h, 2), w, h);
      const cb = Math.max(20, trunc(28 * k));
      const close = R(rect.right - this.ppad - cb, rect.y + this.ppad, cb, cb);
      return [rect, close];
    }

    _drawWindow(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const [rect, close] = this._windowRect();
      ui.drawPanel(ctx, rect, { accentTop: this.accent });
      const pad = this.ppad;
      const icon = D.icon(this.window, Math.max(10, close.h - 6), this.accent);
      D.blit(ctx, icon, rect.x + pad, cyOf(close) - fdiv(icon.lh, 2));
      const title = (this.window === "stats" ? t("chip.stats") : t("chip.fair_title")) + "  ·  Lama Crash";
      this._put(ctx, this._fit(title, [this.fBig, this.fBtn], close.x - rect.x - icon.lw - 3 * pad, ui.TEXT),
        rect.x + pad + icon.lw + fdiv(pad, 2), cyOf(close), "midleft");
      this._btn(ctx, close, "", { hot: close.collidepoint(this.mouse), icon: "close" });
      const body = R(rect.x + pad, close.bottom + pad, rect.w - 2 * pad, rect.bottom - close.bottom - 2 * pad);
      if (this.window === "stats") this._drawStats(ctx, body);
      else this._drawFair(ctx, body);
    }

    _xint(name) {
      const v = this.stats.x[name];
      return isInt(v) && v >= 0 ? v : 0;
    }

    _drawStats(ctx, body) {
      const st = this.stats;
      const k = this.k;
      let rtp = "–";
      if (st.wagered > 0) {
        const p = fdiv(st.paid * 20000 + st.wagered, st.wagered * 2);
        rtp = this._group(fdiv(p, 100)) + this._sep() + String(p % 100).padStart(2, "0") + " %";
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
        [t("lcx.st_top"), this._xint("top") ? this._fmt(this._xint("top")) : "–", this.accent],
        [t("lcx.st_half"), this._group(this._xint("half")), ui.TEXT],
        [t("lcx.st_close"), this._group(this._xint("close")), CD.COL_GOLD],
      ];
      const cols = 4;
      const gap = Math.max(4, trunc(8 * k));
      const tw = fdiv(body.w - (cols - 1) * gap, cols);
      const th = this.fTiny.px + this.fBig.px + Math.max(8, trunc(12 * k));
      const rad = Math.max(4, trunc(8 * k));
      tiles.forEach(([lab, val, col], i) => {
        const r = R(body.x + (i % cols) * (tw + gap), body.y + fdiv(i, cols) * (th + gap), tw, th);
        draw.rect(ctx, ui.mix(ui.PANEL, [0, 0, 0], 0.25), r, 0, rad);
        draw.rect(ctx, ui.BORDER, r, 1, rad);
        this._put(ctx, this._fit(lab, [this.fTiny, this.fMicro], tw - 8, ui.TEXT_DIM), r.x + 5, r.y + Math.max(3, trunc(4 * k)));
        this._put(ctx, this._fit(val, [this.fBig, this.fBtn, this.fTiny], tw - 8, col), r.x + 5, r.bottom - Math.max(3, trunc(4 * k)), "bottomleft");
      });
      const y = body.y + 3 * (th + gap) + gap;
      const half = fdiv((body.w - gap) * 45, 100);
      const left = R(body.x, y, half, body.bottom - y);
      const right = R(left.right + gap, y, body.right - left.right - gap, body.bottom - y);
      // Bilanzkurve
      const curve = st.curve;
      let img = this._fit(curve.length ? t("chip.st_curve", { n: curve.length }) : t("chip.st_empty"), [this.fTiny, this.fMicro], left.w, ui.TEXT_DIM);
      this._put(ctx, img, left.x, left.y);
      const cy = left.y + img.h + 4 + ui.font(10).px;
      const chart = R(left.x, cy, left.w, left.bottom - cy);
      if (curve.length >= 2 && chart.h > 20) {
        const vals = curve.length < CS.CURVE ? [0].concat(curve) : curve.slice();
        PG.charts.line(ctx, chart, vals, { color: this.accent, fmt: (v) => this._group(trunc(v)) });
      }
      // Verteilung der Platzpunkte gegen die Erwartung
      const hits = this.stats.x.hits;
      let total = 0;
      for (const v of hits) total += v;
      img = this._fit(t("lcx.st_dist"), [this.fTiny, this.fMicro], right.w, ui.TEXT_DIM);
      this._put(ctx, img, right.x, right.y);
      const area = R(right.x, right.y + img.h + 4, right.w, right.h - img.h - 4);
      const leg = this._fit(t("lcx.st_legend"), [this.fMicro], area.w, ui.TEXT_FAINT);
      this._put(ctx, leg, area.x, area.bottom, "bottomleft");
      area.h -= leg.h + 4;
      if (total <= 0 || area.h < 24) {
        if (area.h >= 14) this._put(ctx, this._fit(t("lcx.st_none"), [this.fSmall, this.fTiny], area.w, ui.TEXT_DIM), cxOf(area), cyOf(area), "center");
        return;
      }
      const exp = L.BUCKETS.map((_, i) => total * L.bucketProb(i));
      const top = Math.max(Math.max(...hits), Math.max(...exp));
      const labels = L.BUCKETS.map((_, i) => this._bucketLabel(i));
      const colors = L.BUCKETS.map((lo) => ui.mix(D.pillColor(lo)[0], ui.PANEL, 0.2));
      const lsz = Math.max(9, trunc(10 * k));
      const hitsR = PG.charts.bars(ctx, area, hits, { labels, colors, maxValue: top, hover: this.statsHover, labelSize: lsz });
      // Erwartung als goldene Linie über den Balken (gleiche Höhe wie die Balken)
      const pts = [];
      let hover = null;
      for (const [cell, i] of hitsR) {
        const yy = cell.y + cell.h - cell.h * (exp[i] / top);
        pts.push([trunc(cell.x + cell.w / 2), trunc(yy)]);
        if (cell.collidepoint(this.mouse)) hover = i;
      }
      if (pts.length >= 2) draw.lines(ctx, CD.COL_GOLD, false, pts, 2);
      for (const p of pts) {
        draw.circle(ctx, CD.COL_GOLD, p, 3);
        draw.circle(ctx, [40, 26, 8], p, 1);
      }
      this.statsHover = hover;
      if (hover !== null) {
        const e = trunc(exp[hover] * 10 + 0.5);
        PG.charts.tooltip(ctx, this.mouse, [labels[hover], t("lcx.st_bar", { n: hits[hover], e: fdiv(e, 10) + this._sep() + (e % 10) })]);
      }
    }

    _bucketLabel(i) {
      const lo = L.BUCKETS[i];
      if (i + 1 >= L.BUCKETS.length) return this._fmtShort(lo) + "+";
      return this._fmtShort(lo) + "–" + L.fmtShort(L.BUCKETS[i + 1], this._sep());
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
      for (const line of this._wrap(t("lcx.fair_info"), this.fMicro, body.w).slice(0, 3)) {
        this._text(ctx, line, body.x, y, this.fMicro, ui.TEXT_DIM);
        y += this.fMicro.px;
      }
      y += gap;
      const rows = [];
      if (this.round !== null) rows.push([t("lcx.fair_current"), this.round.h, ui.TEXT]);
      rows.push([t("chip.fair_next"), this._nextCommit(), this.accent]);
      for (const [lab, hx, col] of rows) {
        const img = this._fit(lab, [this.fTiny, this.fMicro], body.w, ui.TEXT_DIM);
        this._put(ctx, img, body.x, y);
        y += img.h;
        y = this._drawHex(ctx, hx, body.x, y, body.w, col);
        y += 2;
      }
      const bh = Math.max(22, trunc(28 * k));
      const lab = t("chip.fair_client") + ": " + this.fair.client + "   ·   " + t("chip.fair_nonce") + " " + this.fair.nonce;
      const bw = Math.min(this.fBtn.width(t("chip.fair_new_client")) + trunc(20 * k), fdiv(body.w, 2));
      this.rFairNew = R(body.right - bw, y, bw, bh);
      this._put(ctx, this._fit(lab, [this.fTiny, this.fMicro], body.w - bw - gap, ui.TEXT), body.x, y + fdiv(bh, 2), "midleft");
      this._btn(ctx, this.rFairNew, t("chip.fair_new_client"), { on: this.round === null, hot: this.rFairNew.collidepoint(this.mouse) });
      y += bh + gap;
      draw.line(ctx, ui.BORDER, [body.x, y], [body.right, y], 1);
      y += gap;
      const area = R(body.x, y, body.w, body.bottom - y - this.fMicro.px - 2);
      const note = this._fit(t("chip.fair_note"), [this.fMicro], body.w, ui.TEXT_FAINT);
      this._put(ctx, note, body.x, body.bottom - note.h);
      this.rFairRows = [];
      this.rFairVerify = R(0, 0, 0, 0);
      if (!this.fairlog.length) {
        this._put(ctx, this._fit(t("chip.fair_empty"), [this.fSmall, this.fTiny], area.w, ui.TEXT_DIM), cxOf(area), cyOf(area), "center");
        return;
      }
      const lw = Math.max(trunc(area.w * 0.40), Math.min(fdiv(area.w, 2), trunc(220 * k)));
      const lst = R(area.x, area.y, lw, area.h);
      const det = R(lst.right + gap, area.y, area.w - lw - gap, area.h);
      const rh = Math.max(this.fTiny.px + 6, trunc(24 * k));
      const nvis = Math.max(1, fdiv(lst.h, rh));
      const order = this._fairOrder();
      this.fairScroll = Math.max(0, Math.min(this.fairScroll, order.length - nvis));
      if (order.includes(this.fairSel)) {
        const pos = order.indexOf(this.fairSel);
        if (pos < this.fairScroll) this.fairScroll = pos;
        else if (pos >= this.fairScroll + nvis) this.fairScroll = pos - nvis + 1;
      }
      order.slice(this.fairScroll, this.fairScroll + nvis).forEach((idx, row) => {
        const e = this.fairlog[idx];
        const r = R(lst.x, lst.y + row * rh, lst.w, rh - 2);
        const sel = idx === this.fairSel;
        const rr = Math.max(3, fdiv(rh, 4));
        draw.rect(ctx, sel || r.collidepoint(this.mouse) ? ui.BTN_SEL : ui.mix(ui.PANEL, [0, 0, 0], 0.2), r, 0, rr);
        if (sel) draw.rect(ctx, this.accent, r, 1, rr);
        const o = isDict(e.o) ? e.o : {};
        const x = o.x;
        const res = isInt(x) && x >= 100 ? this._fmtShort(x) : "?";
        const num = "#" + e.n;
        const numW = this.fTiny.width(num);
        this._text(ctx, num, r.x + 5, cyOf(r), this.fTiny, ui.TEXT_DIM, "midleft");
        const bg = D.pillColor(isInt(x) ? x : 100)[0];
        const resW = this.fTiny.width(res);
        this._text(ctx, res, r.right - 5, cyOf(r), this.fTiny, ui.mix(bg, [255, 255, 255], 0.35), "midright");
        const hx0 = r.x + 5 + Math.max(numW, this.fTiny.width("#0000")) + 6;
        const hx1 = r.right - 5 - Math.max(resW, this.fTiny.width("×00,00")) - 6;
        if (hx1 - hx0 > 20) this._put(ctx, this._fit(F.short(e.h, 10), [this.fMono], hx1 - hx0, ui.TEXT_FAINT), hx0, cyOf(r), "midleft");
        const chk = this.fairChecks.get(e.n);
        if (chk) {
          const isz = fdiv(rh, 2);
          D.blit(ctx, D.icon(chk[0] ? "check" : "cross", isz, chk[0] ? ui.GREEN : ui.RED), r.right - resW - 10 - isz, r.y + fdiv(r.h - isz, 2));
        }
        this.rFairRows.push([idx, r]);
      });
      if (order.length > nvis) {
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
      if (isInt(o.x)) head += "  ·  " + t("lcx.fair_round", { x: this._fmt(o.x) });
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
      if (!chk) return;
      const [ok, crash, marks] = chk;
      const isz = fdiv(bh * 2, 3);
      D.blit(ctx, D.icon(ok ? "check" : "cross", isz, ok ? ui.GREEN : ui.RED), this.rFairVerify.right + 6, y + fdiv(bh - isz, 2));
      const timg = this._fit(ok ? t("chip.fair_ok") : t("chip.fair_bad"), [this.fBtn, this.fTiny],
        Math.max(10, det.right - this.rFairVerify.right - isz - 14), ok ? ui.GREEN : ui.RED);
      this._put(ctx, timg, this.rFairVerify.right + isz + 10, y + fdiv(bh, 2), "midleft");
      y += bh + 6;
      img = this._fit(t("lcx.fair_calc", { x: this._fmt(crash) }), [this.fTiny, this.fMicro], w, ui.TEXT);
      this._put(ctx, img, x, y);
      y += img.h + 4;
      const ph = det.bottom - y;
      if (ph >= 24 && w >= 40) D.blit(ctx, D.curvePreview(w, Math.min(ph, trunc(w * 0.6)), crash, marks), x, y);
    }
  }

  PG.register(LamaCrashGame, {
    id: "LamaCrashGame",
    key: KEY,
    name: "Lama Crash",
    settingsKey: KEY,
    defaults: DEFAULTS,
  });
})();
