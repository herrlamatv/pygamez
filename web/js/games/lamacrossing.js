/*
 * lamacrossing.js - Lama Crossing (Port von games/lamacrossing.py)
 * =================================================================
 * Mut-Spiel mit Lama-Chips: Das Lama überquert Schritt für Schritt eine
 * Hängebrücke, einen Fluss oder einen Eissee. Jeder geschaffte Schritt hebt den
 * Multiplikator, ein Fehltritt kostet den Einsatz. Auszahlen nach jedem Schritt,
 * am anderen Ufer automatisch. 4 Schwierigkeiten, 3 Welten (nur Optik), nach dem
 * Auszahlen wird gezeigt, wo es schiefgegangen wäre.
 *
 * Gemeinsames Konto der Lama-Bank (casino_bank.js). Einsatz bei LOS abbuchen,
 * jeder Schritt wird sofort gezogen und gespeichert: Wer mitten in der
 * Überquerung geht, macht beim nächsten Start dort weiter. Highscore =
 * Höchststand von 1000 + Lama-Crossing-Bilanz; gameOver wird nie gesetzt.
 */
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  const L = PG.lamaCrossingLogic;
  const D = PG.lamaCrossingDraw;
  const CD = PG.casinoDraw;
  const bank = PG.lamabank;
  const TAU = Math.PI * 2;

  const KEY = "lamacrossing";
  const IDLE = "idle", HOP = "hop", WAIT = "wait", FALL = "fall", CASHED = "cashed", GOAL = "goal";
  const HOP_T = 0.36;
  const FALL_T = 1.25;
  const RESULT_T = 3.2;

  const isConfirm = (ev) => ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space" || ev.key === "KP_Enter");

  class LamaCrossingGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      bank.load();
      const world = this._opt("world", "bridge");
      this.world = L.WORLDS.includes(world) ? world : "bridge";
      const diff = this._opt("difficulty", "medium");
      this.diff = L.DIFFS.includes(diff) ? diff : "medium";
      const bet = this._opt("bet", L.BETS[0]);
      this.bet = L.BETS.includes(bet) ? bet : L.BETS[0];
      let extra = bank.getExtra(KEY, {});
      if (!extra || typeof extra !== "object") extra = {};
      this.hist = L.cleanHistory(extra.hist);
      this.round = L.cleanRound(extra.round);
      this.step = 0;
      this.state = IDLE;
      this.hop = null;
      this.outcome = null;
      this.broken = 0;
      this.reveal = null;
      this.fallT = 0;
      this.resT = 0;
      this.result = null;
      this.win = 0;
      this.lost = 0;
      this.pending = 0;
      this.shownChips = bank.balance();
      this.particles = [];
      this.ripples = [];
      this.coinsFx = [];
      this.popups = [];
      this.toast = null;
      this.shake = 0;
      this.time = 0;
      this.mouse = [-1, -1];
      this.broke = false;
      this._tickCd = 0;
      this._layout();
      if (this.round) {
        this.diff = this.round.diff;
        this.step = this.round.step;
        this.state = WAIT;
        this._say(t("lcr.resumed"));
      } else {
        this._clampBet();
      }
      this.cam = this._camTarget();
      this.score = bank.scoreFor(KEY);
      this._checkBroke();
    }

    _layout() {
      const W = this.width, H = this.height;
      const k = (this.k = Math.min(W / 800, H / 600));
      this.fTiny = ui.font(Math.max(11, Math.floor(13 * k)));
      this.fSmall = ui.font(Math.max(12, Math.floor(15 * k)));
      this.fBtn = ui.font(Math.max(12, Math.floor(15 * k)), true);
      this.fBig = ui.font(Math.max(15, Math.floor(22 * k)), true);
      this.fHuge = ui.font(Math.max(24, Math.floor(48 * k)), true);
      const m = (this.margin = Math.max(6, Math.floor(12 * k)));
      this.hudH = Math.max(28, Math.floor(42 * k));
      this.stripH = Math.max(46, Math.floor(64 * k));
      this.strip = new PG.Rect(0, H - this.stripH, W, this.stripH);
      this.scene = new PG.Rect(0, this.hudH, W, this.strip.y - this.hudH);
      this.pathY = this.scene.y + Math.floor(this.scene.h * 0.56);
      this.gap = Math.max(44, Math.floor(118 * k));
      this.tw = Math.floor(this.gap * 0.62);
      this.llamaN = Math.max(30, Math.floor(72 * k));
      this.hopH = Math.floor(44 * k);
      this.sag = Math.floor(16 * k);
      const sh = this.stripH;
      const bh = Math.max(30, Math.floor(sh * 0.62));
      const by = this.strip.y + Math.floor((sh - bh) / 2);
      const lblW = Math.max(this.fTiny.width(t("cas.bet")), this.fBtn.width("10000")) + Math.floor(14 * k);
      this.rMinus = new PG.Rect(m, by, bh, bh);
      this.rBet = new PG.Rect(this.rMinus.right + 4, by, lblW, bh);
      this.rPlus = new PG.Rect(this.rBet.right + 4, by, bh, bh);
      const pad = Math.floor(22 * Math.max(0.7, k));
      let goW = Math.max(this.fBtn.width(t("lcr.go")), this.fBtn.width(t("lcr.step"))) + pad;
      goW = Math.max(goW, Math.floor(110 * k));
      let cashW = this.fBtn.width(t("lcr.cashout") + " 1000000") + pad;
      const room = W - m - (this.rPlus.right + m) - Math.max(4, Math.floor(8 * k));
      if (goW + cashW > room) {
        const f = room / (goW + cashW);
        goW = Math.floor(goW * f);
        cashW = Math.floor(cashW * f);
      }
      this.rGo = new PG.Rect(W - m - goW, by, goW, bh);
      this.rCash = new PG.Rect(this.rGo.x - Math.max(4, Math.floor(8 * k)) - cashW, by, cashW, bh);
      const barH = Math.max(26, Math.floor(36 * k));
      const barY = this.strip.y - m - barH;
      const bw = Math.min(Math.floor(150 * k), Math.floor((W - 2 * m - 3 * 6) / 4));
      const x0 = Math.floor((W - (4 * bw + 3 * 6)) / 2);
      this.rDiffs = [0, 1, 2, 3].map((i) => new PG.Rect(x0 + i * (bw + 6), barY, bw, barH));
      const wb = Math.max(26, Math.floor(36 * k));
      this.rWorlds = [0, 1, 2].map((i) => new PG.Rect(m + i * (wb + 4), this.hudH + m, wb, wb));
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

    // ===================================================== Konto / Runde
    _saveExtra() {
      bank.setExtra(KEY, { round: this.round, hist: this.hist });
    }
    _clampBet() {
      const chips = bank.balance();
      if (this.bet > chips) {
        const ok = L.BETS.filter((b) => b <= chips);
        this.bet = ok.length ? ok[ok.length - 1] : L.BETS[0];
      }
    }
    _checkBroke() {
      this.broke = this.state === IDLE && bank.isBroke(KEY);
      if (this.broke) this.playSound("gameover");
    }
    _takeCredit() {
      if (bank.refillIfBroke(KEY)) {
        this.playSound("powerup");
        ui.spawnBurst(this.width / 2, this.height / 2, ui.GOLD, 26);
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
    _fmt(m100) {
      return "×" + L.fmtMult(m100, t("lcr.decimal"));
    }

    // ===================================================== Geometrie
    _n() {
      return L.steps(this.diff);
    }
    _tileX(i) {
      return Math.floor(this.width * 0.22) + i * this.gap;
    }
    _tileY(i) {
      const n = this._n();
      if (this.world === "bridge" && i >= 1 && i <= n) return this.pathY + Math.floor(this.sag * Math.sin((Math.PI * i) / (n + 1)));
      return this.pathY;
    }
    _llamaPos() {
      if (this.state === HOP && this.hop) {
        const { a, b } = this.hop;
        const p = Math.min(1, this.hop.t / HOP_T);
        const e = p * p * (3 - 2 * p);
        const x = this._tileX(a) + (this._tileX(b) - this._tileX(a)) * e;
        const y = this._tileY(a) + (this._tileY(b) - this._tileY(a)) * p;
        return [x, y - Math.sin(Math.PI * p) * this.hopH, "hop"];
      }
      const x = this._tileX(this.step);
      let y = this._tileY(this.step);
      if (this.state === FALL) {
        const f = this.fallT;
        if (this.world === "bridge") return [x, y + 0.5 * 1500 * this.k * f * f, "fall"];
        return [x, y + 46 * this.k * Math.min(1, f * 1.6), "fall"];
      }
      if ((this.state === WAIT || this.state === IDLE) && this.world === "bridge" && this.step > 0) {
        y += Math.sin(this.time * 3.1) * 1.2 * this.k;
      }
      return [x, y, "stand"];
    }
    _camTarget() {
      const n = this._n();
      const lx = this.state === HOP && this.hop ? this._tileX(this.hop.b) : this._tileX(this.step);
      const hi = Math.max(0, this._tileX(n + 1) + Math.floor(this.gap * 1.3) - this.width);
      return Math.max(0, Math.min(hi, lx - this.width * 0.32));
    }

    // ===================================================== Ablauf
    _go() {
      if (this.state === IDLE) {
        if (this.broke) return;
        if (this.bet > bank.balance()) {
          this._say(t("cas.not_enough"));
          this.playSound("hit");
          return;
        }
        if (!bank.debit(this.bet, KEY)) return;
        this.round = { bet: this.bet, diff: this.diff, step: 0 };
        this.step = 0;
        this.reveal = null;
        this.playSound("select");
        this._step();
      } else if (this.state === WAIT) {
        this._step();
      }
    }

    _step() {
      const n = this._n();
      const k = this.step + 1;
      const bet = this.round.bet;
      if (L.roll(PG.rand, this.diff)) {
        if (k >= n) {
          this.win = L.payout(bet, this.diff, n);
          bank.credit(this.win, KEY);
          this.pending += this.win;
          this.hist = this.hist.concat([L.mult(this.diff, n)]).slice(-L.HISTORY);
          this.round = null;
          this.outcome = "goal";
        } else {
          this.round.step = k;
          this.outcome = "ok";
        }
      } else {
        this.hist = this.hist.concat([0]).slice(-L.HISTORY);
        this.round = null;
        this.outcome = "fail";
        this.broken = k;
        this.lost = bet;
      }
      this._saveExtra();
      this.score = bank.scoreFor(KEY);
      this.hop = { a: this.step, b: k, t: 0 };
      this.state = HOP;
      this.playSound("move");
    }

    _land() {
      const b = this.hop.b;
      this.step = b;
      const x = this._tileX(b), y = this._tileY(b);
      const n = this._n();
      if (b > n) {
        this.hop = null;
        this.state = GOAL;
        this.resT = 0;
        this.pending = Math.max(0, this.pending - this.win);
        const mult = L.mult(this.diff, n);
        this.result = [t("lcr.goal"), t("lcr.cashed", { n: this.win }), ui.GOLD];
        ui.spawnConfetti(this.width, this.height, 120);
        this._coinShower(70);
        this._popup(x - this.cam, y - this.llamaN - 10 * this.k, "+" + this.win);
        this.playSound("level");
        this.rumble(220);
        this.achEvent("lcr_other_side");
        if (this.diff === "extreme") this.achEvent("lcr_extreme");
        if (mult >= L.BRAVE_MULT) this.achEvent("lcr_brave");
        return;
      }
      if (this.outcome === "fail") {
        this.hop = null;
        this.state = FALL;
        this.fallT = 0;
        this.resT = 0;
        this.shake = 0.35;
        this.result = [t("lcr.fall." + this.world), t("lcr.lost", { n: this.lost }), ui.RED];
        this._spawnFail(x, y);
        this.playSound("hit");
        this.rumble(260);
        return;
      }
      this._dust(x, y);
      if (this.outcome === "goal") {
        this.hop = { a: b, b: b + 1, t: 0 };
        this.playSound("point");
        return;
      }
      this.hop = null;
      this.state = WAIT;
      this.playSound("point");
      this._popup(x - this.cam, y - this.llamaN - 10 * this.k, this._fmt(L.mult(this.diff, b)), ui.GREEN, 0.9);
    }

    _cashOut() {
      if (this.state !== WAIT || !this.round || this.step < 1) return;
      const bet = this.round.bet;
      const mult = L.mult(this.diff, this.step);
      this.win = L.payout(bet, this.diff, this.step);
      bank.credit(this.win, KEY);
      this.hist = this.hist.concat([mult]).slice(-L.HISTORY);
      this.round = null;
      this.reveal = L.nextBreak(PG.rand, this.diff, this.step);
      this._saveExtra();
      this.score = bank.scoreFor(KEY);
      this.state = CASHED;
      this.resT = 0;
      const sub = this.reveal !== null ? t("lcr.reveal", { n: this.reveal }) : t("lcr.reveal_none");
      this.result = [t("lcr.cashed", { n: this.win }), sub, ui.GOLD];
      const x = this._tileX(this.step), y = this._tileY(this.step);
      this._popup(x - this.cam, y - this.llamaN - 10 * this.k, "+" + this.win);
      if (mult >= 300) this._coinShower(mult < L.BRAVE_MULT ? 30 : 60);
      this.playSound("win");
      if (mult >= L.BRAVE_MULT) this.achEvent("lcr_brave");
    }

    _toIdle() {
      this.state = IDLE;
      this.step = 0;
      this.hop = null;
      this.result = null;
      this.reveal = null;
      this.outcome = null;
      this.broken = 0;
      this.ripples = [];
      this._dust(this._tileX(0), this._tileY(0));
      this._clampBet();
      this._checkBroke();
    }

    _setBet(direction) {
      if (this.state !== IDLE) return;
      const i = L.BETS.indexOf(this.bet);
      const j = Math.max(0, Math.min(L.BETS.length - 1, i + direction));
      if (j !== i) {
        this.bet = L.BETS[j];
        this._setOpt("bet", this.bet);
        this.playSound("select");
        if (this.bet > bank.balance()) this._say(t("cas.not_enough"), 1.2);
      }
    }
    _setDiff(i) {
      if (this.state !== IDLE || L.DIFFS[i] === this.diff) return;
      this.diff = L.DIFFS[i];
      this._setOpt("difficulty", this.diff);
      this.cam = this._camTarget();
      this.playSound("click");
    }
    _setWorld(i) {
      if (this.state !== IDLE || L.WORLDS[i] === this.world) return;
      this.world = L.WORLDS[i];
      this._setOpt("world", this.world);
      this.particles = [];
      this.playSound("click");
    }

    // ===================================================== Effekte
    _spawn(x, y, n, col, speed, up, r, life, grav) {
      const R = PG.rand;
      for (let i = 0; i < n; i++) {
        const a = up ? R.uniform(Math.PI * 1.05, Math.PI * 1.95) : R.uniform(0, TAU);
        const sp = speed * R.uniform(0.35, 1.0) * this.k;
        this.particles.push({
          x, y, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp, r: r * this.k * R.uniform(0.6, 1.2),
          col, t: 0, life: life * R.uniform(0.7, 1.1), g: grav * this.k,
        });
      }
      if (this.particles.length > 220) this.particles = this.particles.slice(-220);
    }
    _dust(x, y) {
      const col = { bridge: [196, 160, 120], river: [200, 230, 246], ice: [240, 248, 255] }[this.world];
      this._spawn(x, y, 8, col, 90, true, 3.0, 0.45, 260);
    }
    _spawnFail(x, y) {
      const k = this.k;
      if (this.world === "bridge") {
        this._spawn(x, y, 16, [150, 98, 58], 220, true, 3.2, 1.1, 900);
        this._spawn(x, y, 6, [196, 160, 120], 120, true, 4.0, 0.6, 300);
      } else if (this.world === "river") {
        this._spawn(x, y + 10 * k, 22, [214, 238, 250], 260, true, 3.4, 0.8, 900);
        this.ripples.push({ x, y: y + 12 * k, t: 0 });
        this.ripples.push({ x, y: y + 12 * k, t: -0.25 });
      } else {
        this._spawn(x, y, 14, [232, 246, 255], 200, true, 3.0, 0.9, 800);
        this._spawn(x, y + 8 * k, 14, [150, 206, 236], 240, true, 3.0, 0.8, 900);
        this.ripples.push({ x, y: y + 14 * k, t: -0.3 });
      }
    }
    _coinShower(n = 40) {
      const R = PG.rand, W = this.width, k = this.k;
      for (let i = 0; i < n; i++) {
        this.coinsFx.push({
          x: R.uniform(W * 0.1, W * 0.9), y: R.uniform(-this.height * 0.4, -10),
          vx: R.uniform(-60, 60) * k, vy: R.uniform(-80, 120) * k, spin: R.uniform(0, TAU),
          vs: R.uniform(6, 14), r: R.uniform(7, 12) * Math.max(0.7, k),
        });
      }
      if (this.coinsFx.length > 160) this.coinsFx = this.coinsFx.slice(-160);
    }
    _popup(x, y, text, color, dur = 1.4) {
      this.popups.push({ x, y, text, color: color || ui.GOLD, t: 0, dur });
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      const kind = ev.kind;
      if ((kind === "mousemove" || kind === "mousedown" || kind === "mouseup") && ev.pos) this.mouse = ev.pos;
      if (this.broke) {
        if (kind === "mousedown" || isConfirm(ev)) this._takeCredit();
        return;
      }
      if (this.state === FALL || this.state === CASHED || this.state === GOAL) {
        if (this.resT > 0.6 && (kind === "mousedown" || isConfirm(ev))) this._toIdle();
        return;
      }
      if (kind === "keydown") {
        const key = ev.key;
        if (key === "space" || key === "Right") this._go();
        else if (key === "Return" || key === "KP_Enter") {
          if (this.state === WAIT) this._cashOut();
          else this._go();
        } else if (key === "c" || key === "C") this._cashOut();
        else if (key === "plus" || key === "KP_Add" || key === "Up" || key === "equal") this._setBet(1);
        else if (key === "minus" || key === "KP_Subtract" || key === "Down") this._setBet(-1);
        else if (key === "1" || key === "2" || key === "3" || key === "4") this._setDiff(Number(key) - 1);
        else if (key === "w" || key === "W") this._setWorld((L.WORLDS.indexOf(this.world) + 1) % L.WORLDS.length);
        return;
      }
      if (kind === "wheel") {
        this._setBet(ev.delta > 0 ? 1 : -1);
        return;
      }
      if (kind !== "mousedown" || ev.button !== 1) return;
      const pos = ev.pos;
      if (this.rMinus.collidepoint(pos)) this._setBet(-1);
      else if (this.rPlus.collidepoint(pos)) this._setBet(1);
      else if (this.rGo.collidepoint(pos)) this._go();
      else if (this.rCash.collidepoint(pos)) this._cashOut();
      else if (this.state === IDLE) {
        for (let i = 0; i < this.rDiffs.length; i++) {
          if (this.rDiffs[i].collidepoint(pos)) return this._setDiff(i);
        }
        for (let i = 0; i < this.rWorlds.length; i++) {
          if (this.rWorlds[i].collidepoint(pos)) return this._setWorld(i);
        }
      } else if (this.state === WAIT && this._nextTileRect().collidepoint(pos)) {
        this._go();
      }
    }

    _nextTileRect() {
      const i = this.step + 1;
      const x = this._tileX(i) - this.cam, y = this._tileY(i);
      return new PG.Rect(Math.floor(x - this.gap * 0.45), Math.floor(y - this.llamaN), Math.floor(this.gap * 0.9),
        Math.floor(this.llamaN + this.tw * 0.6));
    }

    // ===================================================== Update
    update(dt) {
      this.time += dt;
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
            this.tone(1500 + PG.rand.random() * 300, 0.025, "square", 0.08);
          }
        }
      }
      if (this.toast) {
        this.toast[1] -= dt;
        if (this.toast[1] <= 0) this.toast = null;
      }
      this.shake = Math.max(0, this.shake - dt);
      this._updateFx(dt);
      const tgt = this._camTarget();
      this.cam += (tgt - this.cam) * Math.min(1, dt * 6);
      if (Math.abs(tgt - this.cam) < 0.5) this.cam = tgt;
      if (this.state === HOP) {
        this.hop.t += dt;
        if (this.hop.t >= HOP_T) this._land();
      } else if (this.state === FALL || this.state === CASHED || this.state === GOAL) {
        this.fallT += dt;
        this.resT += dt;
        if (this.resT >= RESULT_T + (this.state === FALL ? FALL_T : 0)) this._toIdle();
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
      for (const r of this.ripples) r.t += dt;
      this.ripples = this.ripples.filter((r) => r.t < 1.1);
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
      const bg = D.background(this.world, W, H, this.pathY);
      ctx.drawImage(bg, 0, 0, bg.lw, bg.lh);
      const far = D.farLayer(this.world, W, H, this.pathY);
      const off = PG.mod(-Math.floor(this.cam * 0.18), W);
      ctx.drawImage(far, off - W, 0, far.lw, far.lh);
      ctx.drawImage(far, off, 0, far.lw, far.lh);
      let ox = -this.cam;
      if (this.shake > 0) ox += (Math.sin(this.time * 70) * 6 * this.k * this.shake) / 0.35;
      this._drawWater(ctx, ox);
      this._drawWorld(ctx, ox);
      this._drawLlama(ctx, ox);
      this._drawParticles(ctx, ox);
      if (this.world === "ice") this._drawSnow(ctx, 1);
      this._drawHud(ctx);
      this._drawHistory(ctx);
      this._drawWorlds(ctx);
      if (this.state === IDLE) this._drawSetup(ctx);
      else if (this.state === WAIT) this._drawHint(ctx, t("lcr.hint_wait"));
      this._drawStrip(ctx);
      this._drawFx(ctx);
      if (this.result && (this.state !== FALL || this.fallT > FALL_T * 0.55)) this._drawResult(ctx);
      if (this.toast) this._drawToast(ctx);
      if (this.broke) this._drawBroke(ctx);
    }

    // ----- Welt
    _drawWater(ctx, ox) {
      const k = this.k, W = this.width;
      if (this.world === "river") {
        for (let i = 0; i < 26; i++) {
          const row = i % 6;
          const y = this.pathY - Math.floor(this.height * 0.06) + row * Math.floor(20 * k) + Math.floor(10 * k);
          if (y > this.height) continue;
          const sp = 28 + row * 9;
          const x = PG.mod(i * 137 + this.time * sp * k + ox * 0.6, W + 60) - 30;
          draw.line(ctx, [150, 204, 232], [x, y], [x + 22 * k, y], Math.max(1, 2 * k));
        }
      } else if (this.world === "ice") {
        this._drawSnow(ctx, 0);
        for (let i = 0; i < 10; i++) {
          const x = PG.mod(i * 211 + ox * 0.9, W + 80) - 40;
          const y = this.pathY + Math.floor(40 * k) + (i % 3) * Math.floor(26 * k);
          draw.line(ctx, [40, 92, 124], [x, y], [x + 40 * k, y], Math.max(1, 2 * k));
        }
      } else {
        for (let i = 0; i < 3; i++) {
          const x = PG.mod(i * 300 + this.time * 25 * k, W + 100) - 50;
          const y = this.pathY * 0.3 + i * 22 * k + Math.sin(this.time * 2 + i) * 4 * k;
          const wing = Math.sin(this.time * 9 + i * 2) * 3 * k;
          draw.lines(ctx, [70, 50, 60], false, [[x - 7 * k, y - wing], [x, y + 2 * k], [x + 7 * k, y - wing]], Math.max(1, 2 * k));
        }
      }
    }

    _drawSnow(ctx, layer) {
      const k = this.k, W = this.width, H = this.height;
      for (let i = layer; i < 60; i += 2) {
        const x = PG.mod(i * 97.3 + this.time * (10 + (i % 5) * 4) * k + Math.sin(this.time + i) * 8 * k, W);
        const y = this.hudH + PG.mod(i * 53.1 + this.time * (24 + (i % 7) * 5) * k, H - this.hudH);
        draw.circle(ctx, [255, 255, 255], [x, y], Math.max(1, (1 + layer) * k));
      }
    }

    _drawWorld(ctx, ox) {
      const k = this.k, n = this._n(), H = this.height, W = this.width;
      const xl = this._tileX(0) + this.gap * 0.42 + ox;
      const xr = this._tileX(n + 1) - this.gap * 0.42 + ox;
      if (this.world === "bridge") {
        const pts = [[xl, this.pathY]];
        for (let i = 1; i <= n; i++) {
          if (this.state === FALL && i === this.broken) continue;
          pts.push([this._tileX(i) + ox, this._tileY(i) + 4 * k]);
        }
        pts.push([xr, this.pathY]);
        D.drawRopes(ctx, pts, k, W);
      }
      D.drawBank(ctx, this.world, xl, -1, this.pathY, H, k, W);
      D.drawBank(ctx, this.world, xr, 1, this.pathY, H, k, W);
      this._drawGoalFlag(ctx, ox);
      const cur = [WAIT, HOP, FALL, CASHED, GOAL].includes(this.state) ? this.step : 0;
      for (let i = 1; i <= n; i++) {
        const cx = this._tileX(i) + ox;
        if (cx < -this.gap || cx > W + this.gap) continue;
        const y = this._tileY(i);
        if (this.state === FALL && i === this.broken) {
          this._drawBrokenTile(ctx, cx, y, i);
        } else {
          if (this.state === WAIT && i === this.step + 1) {
            const gw = this.tw * 1.3, gh = this.tw * 0.5;
            const col = ui.mix(CD.COL_GOLD, [255, 255, 255], 0.3 * ui.pulse(3.0, 0, 1));
            draw.ellipse(ctx, col, [cx - gw / 2, y + this.tw * 0.12 - gh / 2, gw, gh], Math.max(1, 2 * k));
          }
          D.drawTile(ctx, this.world, cx, y, this.tw, k, this.time, i);
          if (this.state === CASHED && this.reveal === i) this._drawCross(ctx, cx, y);
        }
        this._drawLabel(ctx, cx, y, i, cur);
      }
    }

    _drawGoalFlag(ctx, ox) {
      const k = this.k, n = this._n();
      const x = this._tileX(n + 1) + Math.floor(this.gap * 0.55) + ox;
      if (x < -40 || x > this.width + 40) return;
      const y = this.pathY, pole = 70 * k;
      draw.line(ctx, [90, 70, 60], [x, y], [x, y - pole], Math.max(2, 3 * k));
      const wave = Math.sin(this.time * 4) * 3 * k;
      draw.polygon(ctx, this.accent, [[x, y - pole], [x + 34 * k, y - pole + 8 * k + wave], [x, y - pole + 18 * k]]);
      ui.text(ctx, this._fmt(L.mult(this.diff, n)), x, y - pole - 4 * k, this.fSmall, CD.COL_GOLD, "midbottom");
    }

    _drawBrokenTile(ctx, cx, y, i) {
      const k = this.k, f = this.fallT, tw = this.tw;
      if (this.world === "river") {
        D.drawTile(ctx, this.world, cx, y + 26 * k * Math.min(1, f), tw, k, this.time, i, Math.max(0, 1 - f / 0.8));
        return;
      }
      if (this.world === "ice" && f < 0.3) {
        D.drawTile(ctx, this.world, cx, y, tw, k, this.time, i);
        for (const line of D.crackLines(cx, y, tw, i, f / 0.3)) draw.lines(ctx, [40, 92, 124], false, line, Math.max(1, 2 * k));
        return;
      }
      const g = f - (this.world === "ice" ? 0.3 : 0);
      const drop = this.world === "bridge" ? 0.5 * 1100 * k * g * g : 30 * k * g;
      const alpha = Math.max(0, 1 - g / 0.9);
      for (const side of [0, 1]) {
        ctx.save();
        ctx.globalAlpha *= alpha;
        ctx.translate((side === 0 ? -1 : 1) * (6 + 40 * g) * k, drop);
        ctx.beginPath();
        ctx.rect(side === 0 ? cx - tw * 1.5 : cx, y - tw, tw * 1.5, tw * 3);
        ctx.clip();
        D.drawTile(ctx, this.world, cx, y, tw, k, this.time, i);
        ctx.restore();
      }
    }

    _drawCross(ctx, cx, y) {
      const d = this.tw * 0.22, c = [cx, y + this.tw * 0.1], w = Math.max(2, 3 * this.k);
      draw.line(ctx, ui.RED, [c[0] - d, c[1] - d * 0.6], [c[0] + d, c[1] + d * 0.6], w);
      draw.line(ctx, ui.RED, [c[0] - d, c[1] + d * 0.6], [c[0] + d, c[1] - d * 0.6], w);
    }

    _drawLabel(ctx, cx, y, i, cur) {
      const txt = this._fmt(L.mult(this.diff, i));
      let fg, bg;
      if (this.state === FALL && i === this.broken) [fg, bg] = [[255, 255, 255], ui.RED];
      else if (this.state === CASHED && this.reveal === i) [fg, bg] = [[255, 220, 220], [120, 30, 40]];
      else if (i <= cur) [fg, bg] = [[230, 255, 236], [40, 120, 72]];
      else if (i === cur + 1) [fg, bg] = [[30, 24, 10], CD.COL_GOLD];
      else [fg, bg] = [[232, 232, 240], [24, 22, 34]];
      const w = this.fTiny.width(txt) + Math.floor(10 * this.k) + 4;
      const h = this.fTiny.height + Math.floor(4 * this.k) + 2;
      const box = new PG.Rect(cx - w / 2, y + this.tw * 0.36 + 6 * this.k, w, h);
      draw.rect(ctx, bg, box, 0, h / 2);
      ui.text(ctx, txt, box.centerx, box.centery, this.fTiny, fg, "center");
    }

    _drawLlama(ctx, ox) {
      const [x, y, pose] = this._llamaPos();
      const n = this.llamaN;
      const img = D.llamaCanvas(n, pose, this.accent);
      const sx = x + ox;
      if (this.state !== FALL && pose !== "hop") {
        const sw = n * 0.5, shh = Math.max(3, n * 0.08);
        draw.ellipse(ctx, [0, 0, 0], [sx + n * 0.02 - sw / 2, y + 1 - shh / 2, sw, shh]);
      }
      let alpha = 1;
      if (this.state === FALL) {
        const f = this.fallT;
        alpha = 1 - Math.min(1, Math.max(0, (f - (this.world === "bridge" ? 0.5 : 0.15)) / 0.5));
        if (alpha <= 0) return;
      }
      ctx.save();
      ctx.globalAlpha *= alpha;
      ctx.drawImage(img, sx - n / 2, y - n + n * 0.06, img.lw, img.lh);
      ctx.restore();
    }

    _drawParticles(ctx, ox) {
      for (const p of this.particles) {
        const r = p.r * (1 - p.t / p.life);
        if (r >= 0.5) draw.circle(ctx, p.col, [p.x + ox, p.y], Math.max(1, r));
      }
      for (const rp of this.ripples) {
        if (rp.t <= 0) continue;
        const w = (20 + 140 * rp.t) * this.k, h = Math.max(2, w / 4);
        draw.ellipse(ctx, [210, 236, 250], [rp.x + ox - w / 2, rp.y - h / 2, w, h], Math.max(1, 2 * this.k));
      }
    }

    // ----- Oberfläche
    _drawHud(ctx) {
      const W = this.width, pad = this.margin, cy = this.hudH / 2;
      const g = ctx.createLinearGradient(0, 0, 0, this.hudH);
      g.addColorStop(0, "rgba(10,8,16,0.82)");
      g.addColorStop(1, "rgba(10,8,16,0.59)");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W, this.hudH);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), [0, this.hudH - 1], [W, this.hudH - 1], 1);
      const coinR = Math.max(6, Math.floor(this.hudH * 0.26));
      const coin = CD.symbolCanvas("coin", coinR * 2 + 4);
      ctx.drawImage(coin, pad - 2, cy - coin.lh / 2, coin.lw, coin.lh);
      const lr = ui.text(ctx, t("cas.chips") + ": " + Math.round(this.shownChips), pad + coinR * 2 + 6, cy, this.fBig, ui.GOLD, "midleft");
      const rightTxt = t("cas.record") + ": " + this.score + "   ·   " + t("cas.credits", { n: bank.refills() });
      const rf = this.fSmall.width(rightTxt) > W * 0.42 ? this.fTiny : this.fSmall;
      const rr = ui.text(ctx, rightTxt, W - pad, cy, rf, ui.TEXT_DIM, "midright");
      if (!this.round) return;
      const bet = this._roundBet();
      let mid = t("cas.bet") + ": " + bet;
      if (this.step > 0) mid += "   ·   " + this._fmt(L.mult(this.diff, this.step)) + " = " + L.payout(bet, this.diff, this.step);
      const x0 = lr.right + pad, x1 = rr.x - pad;
      const f = this.fSmall.width(mid) > x1 - x0 ? this.fTiny : this.fSmall;
      if (f.width(mid) <= x1 - x0) ui.text(ctx, mid, (x0 + x1) / 2, cy, f, ui.TEXT, "center");
    }

    _drawHistory(ctx) {
      let x = this.width - this.margin;
      const y = this.hudH + this.margin;
      const last = this.rWorlds[this.rWorlds.length - 1];
      const leftLimit = last.right + this.margin + this.fSmall.width(t("lcr.world." + this.world)) + this.margin;
      for (let i = this.hist.length - 1; i >= 0; i--) {
        const m100 = this.hist[i];
        const txt = m100 > 0 ? this._fmt(m100) : "×0";
        const w = this.fTiny.width(txt) + Math.floor(10 * this.k) + 4;
        const h = this.fTiny.height + Math.floor(6 * this.k) + 2;
        const box = new PG.Rect(x - w, y, w, h);
        if (box.x < leftLimit) break;
        const col = m100 >= 200 ? [40, 120, 72] : m100 > 0 ? [70, 96, 70] : [128, 36, 46];
        draw.rect(ctx, col, box, 0, h / 2);
        ui.text(ctx, txt, box.centerx, box.centery, this.fTiny, [240, 240, 245], "center");
        x = box.x - 4;
      }
    }

    _drawWorlds(ctx) {
      const on = this.state === IDLE;
      this.rWorlds.forEach((r, i) => {
        const w = L.WORLDS[i];
        const active = w === this.world;
        const hot = on && r.collidepoint(this.mouse);
        const rad = Math.max(4, Math.floor(r.h / 4));
        draw.rect(ctx, active || hot ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        D.worldIcon(ctx, w, r.inflate(-Math.floor(r.w / 5), -Math.floor(r.h / 5)), on || active ? ui.TEXT : ui.TEXT_FAINT);
      });
      const last = this.rWorlds[this.rWorlds.length - 1];
      ui.text(ctx, t("lcr.world." + this.world), last.right + this.margin, last.centery, this.fSmall, ui.TEXT, "midleft");
    }

    _drawSetup(ctx) {
      this.rDiffs.forEach((r, i) => {
        this._btn(ctx, r, i + 1 + "  " + t("lcr.diff." + L.DIFFS[i]), { active: L.DIFFS[i] === this.diff, hot: r.collidepoint(this.mouse) });
      });
      const info = t("lcr.info", { n: this._n(), p: L.riskPercent(this.diff), m: L.fmtMult(L.mult(this.diff, this._n()), t("lcr.decimal")) });
      this._drawHint(ctx, info, this.rDiffs[0].y - 4);
    }

    _drawHint(ctx, text, bottom) {
      const w = this.fTiny.width(text) + 18, h = this.fTiny.height + 8;
      const b = bottom !== undefined ? bottom : this.strip.y - this.margin;
      const box = new PG.Rect(this.width / 2 - w / 2, b - h, w, h);
      draw.rect(ctx, [12, 8, 20, 190], box, 0, h / 2);
      ui.text(ctx, text, box.centerx, box.centery, this.fTiny, ui.TEXT, "center");
    }

    _drawStrip(ctx) {
      const g = ctx.createLinearGradient(0, this.strip.y, 0, this.strip.bottom);
      g.addColorStop(0, "rgba(10,8,16,0.67)");
      g.addColorStop(1, "rgba(10,8,16,0.9)");
      ctx.fillStyle = g;
      ctx.fillRect(this.strip.x, this.strip.y, this.strip.w, this.strip.h);
      draw.line(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.35), this.strip.topleft, this.strip.topright, 2);
      const idle = this.state === IDLE;
      const bet = this._roundBet();
      this._btn(ctx, this.rMinus, "–", { on: idle && this.bet > L.BETS[0], hot: this.rMinus.collidepoint(this.mouse) });
      this._btn(ctx, this.rPlus, "+", { on: idle && this.bet < L.BETS[L.BETS.length - 1], hot: this.rPlus.collidepoint(this.mouse) });
      const rad = Math.max(6, Math.floor(this.rBet.h / 4));
      draw.rect(ctx, [16, 12, 26], this.rBet, 0, rad);
      draw.rect(ctx, ui.BORDER, this.rBet, 1, rad);
      ui.text(ctx, t("cas.bet"), this.rBet.centerx, this.rBet.y + 2, this.fTiny, ui.TEXT_DIM, "midtop");
      const col = idle && bet > bank.balance() ? ui.RED : ui.GOLD;
      ui.text(ctx, String(bet), this.rBet.centerx, this.rBet.bottom - 3, this.fBtn, col, "midbottom");
      const wait = this.state === WAIT;
      this._btn(ctx, this.rGo, idle ? t("lcr.go") : t("lcr.step"), { primary: true, on: (idle && !this.broke) || wait, hot: this.rGo.collidepoint(this.mouse) });
      let cash = t("lcr.cashout");
      if (wait && this.round) cash += " " + L.payout(this.round.bet, this.diff, this.step);
      this._btn(ctx, this.rCash, cash, { on: wait, hot: this.rCash.collidepoint(this.mouse), active: wait });
    }

    _btn(ctx, r, label, { primary = false, on = true, hot = false, active = false } = {}) {
      const rad = Math.max(6, Math.floor(r.h / 4));
      let col;
      if (primary && on) {
        const base = ui.mix(this.accent, [0, 0, 0], 0.18);
        draw.rect(ctx, ui.mix(base, [0, 0, 0], 0.4), [r.x, r.y + 2, r.w, r.h], 0, rad);
        draw.rect(ctx, ui.mix(base, [255, 255, 255], hot ? 0.12 : 0), r, 0, rad);
        draw.rect(ctx, ui.mix(CD.COL_GOLD, [255, 255, 255], 0.35 * ui.pulse(2.4, 0, 1)), r, 2, rad);
        col = [255, 255, 255];
      } else {
        draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? CD.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        col = on ? ui.TEXT : ui.TEXT_FAINT;
      }
      const f = this.fBtn.width(label) > r.w - 8 ? this.fTiny : this.fBtn;
      ui.text(ctx, label, r.centerx, r.centery, f, col, "center");
    }

    _drawFx(ctx) {
      for (const c of this.coinsFx) {
        const w = Math.max(2, c.r * 2 * Math.abs(Math.cos(c.spin)));
        const h = c.r * 2;
        draw.ellipse(ctx, CD.COL_GOLD_D, [c.x - w / 2 + 1, c.y - h / 2 + 1, w, h]);
        draw.ellipse(ctx, [246, 196, 60], [c.x - w / 2, c.y - h / 2, w, h]);
        if (w > 6) draw.ellipse(ctx, [255, 236, 150], [c.x - w / 4, c.y - h / 4, w / 2, h / 2]);
      }
      for (const p of this.popups) {
        const f = p.t / p.dur;
        const alpha = f > 0.6 ? 1 - (f - 0.6) / 0.4 : 1;
        const y = p.y - 40 * this.k * (1 - (1 - f) ** 2);
        ui.text(ctx, p.text, p.x, y, this.fBig, p.color, "center", alpha);
      }
    }

    _drawResult(ctx) {
      const [title, sub, col] = this.result;
      const W = this.width, k = this.k;
      const tf = this.fHuge.width(title) > W - 40 ? this.fBig : this.fHuge;
      const sf = this.fSmall.width(sub) > W - 40 ? this.fTiny : this.fSmall;
      const pw = Math.min(W - 20, Math.max(tf.width(title), sf.width(sub)) + Math.floor(48 * k));
      const ph = tf.height + sf.height + Math.floor(34 * k);
      const cy = this.scene.y + Math.floor(this.scene.h * 0.24) + Math.floor(ph / 2) - Math.floor(10 * k);
      const panel = new PG.Rect(W / 2 - pw / 2, cy - ph / 2, pw, ph);
      ui.drawPanel(ctx, panel, { accentTop: col });
      ui.text(ctx, title, panel.centerx, panel.y + Math.floor(12 * k), tf, col, "midtop");
      ui.text(ctx, sub, panel.centerx, panel.bottom - Math.floor(12 * k), sf, ui.TEXT, "midbottom");
    }

    _drawToast(ctx) {
      const [text, left] = this.toast;
      const w = this.fSmall.width(text) + 24, h = this.fSmall.height + 12;
      const top = this.hudH + this.margin + this.rWorlds[0].h + this.margin;
      const box = new PG.Rect(this.width / 2 - w / 2, top, w, h);
      ctx.save();
      ctx.globalAlpha = left < 0.3 ? left / 0.3 : 1;
      draw.rect(ctx, [12, 8, 20, 225], box, 0, h / 2);
      draw.rect(ctx, ui.mix(CD.COL_GOLD, this.accent, 0.4), box, 1, h / 2);
      ui.text(ctx, text, box.centerx, box.centery, this.fSmall, ui.TEXT, "center");
      ctx.restore();
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      const pw = Math.min(this.width - 40, Math.floor(480 * Math.max(0.8, this.k)));
      const ph = Math.floor(200 * Math.max(0.75, this.k));
      const panel = new PG.Rect(cx - pw / 2, cy - ph / 2, pw, ph);
      ui.drawPanel(ctx, panel, { accentTop: ui.RED });
      ui.text(ctx, t("cas.broke"), cx, panel.y + ph * 0.28, this.fHuge, ui.RED, "center");
      [[t("cas.broke_sub"), ui.TEXT_DIM], [t("cas.broke_restart", { n: bank.START_CHIPS }), ui.TEXT]].forEach(([txt, col], i) => {
        const f = this.fSmall.width(txt) > pw - 16 ? this.fTiny : this.fSmall;
        ui.text(ctx, txt, cx, panel.y + ph * (0.58 + 0.2 * i), f, col, "center");
      });
    }
  }

  PG.register(LamaCrossingGame, {
    id: "LamaCrossingGame",
    key: KEY,
    name: "Lama Crossing",
    settingsKey: KEY,
    defaults: { world: "bridge", difficulty: "medium", bet: 10 },
  });
})();
