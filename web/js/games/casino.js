/*
 * casino.js - Casino: Roulette + Lama-Slot (Port von games/casino.py,
 *             casino_roulette.py und casino_slots.py)
 * ====================================================================
 * Gespielt wird mit den Lama-Chips der Lama-Bank (casino_bank.js), die sich
 * Blackjack, Poker und Casino teilen. Regeln/Tabellen: casino_logic.js,
 * Zeichen-Bausteine: casino_draw.js.
 *
 * Geld-Fluss wie am Desktop: Einsatz beim Drehen sofort abbuchen, das vorher
 * gezogene Ergebnis sofort gutschreiben - angezeigt wird der Gewinn aber erst,
 * wenn Kugel bzw. Walzen stehen (this.pending). Highscore = Höchststand von
 * 1000 + Casino-Bilanz; gameOver wird nie gesetzt. Pleite = Bank-Kredit.
 */
(function () {
  "use strict";
  const { ui, draw, t } = PG;
  const L = PG.casinoLogic;
  const D = PG.casinoDraw;
  const bank = PG.lamabank;
  const TAU = Math.PI * 2;

  // ---- Roulette
  const R_BET = "bet", R_SPIN = "spin", R_RESULT = "result";
  const LAND_T = 5.6;
  const RESULT_T = 3.4;
  const IDLE_OMEGA = 0.32;
  const SPIN_OMEGA = 2.3;
  const OMEGA_TAU = 2.8;
  const BALL_TURNS = 6;
  const HISTORY_KEY = "roulette_history";
  // ---- Slot
  const S_IDLE = "idle", S_SPIN = "spin", S_SHOW = "show";
  const FREE_KEY = "slots_free";
  const SPEED = 21.0;
  const DECEL = 0.34;
  const BOUNCE = 0.16;
  const BIG_WIN = 15;
  const MEGA_WIN = 50;

  const isConfirm = (ev) => ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space" || ev.key === "KP_Enter");
  const toIntOr = (v, d) => (typeof v === "number" && Number.isFinite(v) ? Math.trunc(v) : d);

  class CasinoGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.gameOver = false;
      if (this.mode !== "roulette" && this.mode !== "slots") this.mode = "roulette";
      bank.load();
      this.pending = 0;
      this.shownChips = bank.balance();
      this.mouse = [-1, -1];
      this.coinsFx = [];
      this.popups = [];
      this.toast = null;
      this.broke = false;
      this._tickCd = 0;
      this._layout();
      if (this.mode === "roulette") this._rInit();
      else this._sInit();
      this.score = bank.scoreFor("casino");
      this._checkBroke();
    }

    _layout() {
      const W = this.width, H = this.height;
      this.fTiny = ui.font(13);
      this.fSmall = ui.font(15);
      this.fBtn = ui.font(15, true);
      this.fBig = ui.font(22, true);
      this.fHuge = ui.font(58, true);
      this.margin = 12;
      this.hudH = 42;
      this.stripH = 64;
      this.strip = new PG.Rect(0, H - this.stripH, W, this.stripH);
      if (this.mode === "roulette") this._rLayout();
      else this._sLayout();
    }

    get wantsRightClick() {
      return true;
    }

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
    _busy() {
      return this.mode === "roulette" ? this.rPhase === R_SPIN : this.sPhase === S_SPIN;
    }
    _checkBroke() {
      if (this._busy()) {
        this.broke = false;
        return;
      }
      const free = this.mode === "slots" && this.sFree > 0;
      this.broke = !free && bank.isBroke("casino", this.mode);
      if (this.broke) this.playSound("gameover");
    }
    _takeCredit() {
      if (bank.refillIfBroke("casino", this.mode)) {
        this.playSound("powerup");
        ui.spawnBurst(this.width / 2, this.height / 2, ui.GOLD, 26);
      }
      this.broke = false;
      this.shownChips = bank.balance();
    }
    _reserved() {
      return this.mode === "roulette" && this.rPhase === R_BET ? this._rTotal() : 0;
    }
    _targetChips() {
      return Math.max(0, bank.balance() - this.pending - this._reserved());
    }
    _say(text, secs = 1.8) {
      this.toast = [text, secs];
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if ((ev.kind === "mousemove" || ev.kind === "mousedown" || ev.kind === "mouseup") && ev.pos) this.mouse = ev.pos;
      if (this.broke) {
        if (ev.kind === "mousedown" || isConfirm(ev)) this._takeCredit();
        return;
      }
      if (this.mode === "roulette") this._rEvent(ev);
      else this._sEvent(ev);
    }

    // ===================================================== Update
    update(dt) {
      const target = this._targetChips();
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
            this.tone(1500 + Math.random() * 300, 0.025, "square", 0.08);
          }
        }
      }
      if (this.toast) {
        this.toast[1] -= dt;
        if (this.toast[1] <= 0) this.toast = null;
      }
      this._updateFx(dt);
      if (this.mode === "roulette") this._rUpdate(dt);
      else this._sUpdate(dt);
    }

    _updateFx(dt) {
      this.coinsFx = this.coinsFx.filter((c) => {
        c.vy += 900 * dt;
        c.x += c.vx * dt;
        c.y += c.vy * dt;
        c.spin += c.vs * dt;
        return c.y < this.height + 30;
      });
      for (const p of this.popups) p.t += dt;
      this.popups = this.popups.filter((p) => p.t < p.dur);
    }

    _coinShower(n = 40) {
      for (let i = 0; i < n; i++) {
        this.coinsFx.push({
          x: PG.rand.uniform(this.width * 0.1, this.width * 0.9), y: PG.rand.uniform(-this.height * 0.4, -10),
          vx: PG.rand.uniform(-60, 60), vy: PG.rand.uniform(-80, 120), spin: PG.rand.uniform(0, TAU),
          vs: PG.rand.uniform(6, 14), r: PG.rand.uniform(7, 12),
        });
      }
      if (this.coinsFx.length > 160) this.coinsFx = this.coinsFx.slice(-160);
    }

    _popup(x, y, text, color, dur = 1.4) {
      this.popups.push({ x, y, text, color: color || ui.GOLD, t: 0, dur });
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.mode === "roulette") this._rDraw(ctx);
      else this._sDraw(ctx);
      this._drawHud(ctx);
      this._drawFx(ctx);
      if (this.toast) this._drawToast(ctx);
      if (this.broke) this._drawBroke(ctx);
    }

    _fitFont(text, maxW, fonts) {
      for (const f of fonts) if (f.width(text) <= maxW) return f;
      return fonts[fonts.length - 1];
    }

    _drawHud(ctx) {
      const W = this.width, pad = this.margin, cy = this.hudH / 2;
      const g = ctx.createLinearGradient(0, 0, 0, this.hudH);
      g.addColorStop(0, "rgba(10,8,16,0.82)");
      g.addColorStop(1, "rgba(10,8,16,0.59)");
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W, this.hudH);
      draw.line(ctx, ui.mix(D.COL_GOLD, this.accent, 0.35), [0, this.hudH - 1], [W, this.hudH - 1], 1);
      const coinR = Math.max(6, Math.floor(this.hudH * 0.26));
      const coin = D.symbolCanvas("coin", coinR * 2 + 4);
      ctx.drawImage(coin, pad - 2, cy - coin.lh / 2, coin.lw, coin.lh);
      const leftTxt = t("cas.chips") + ": " + Math.round(this.shownChips);
      const lr = ui.text(ctx, leftTxt, pad + coinR * 2 + 6, cy, this.fBig, ui.GOLD, "midleft");
      const rightTxt = t("cas.record") + ": " + this.score + "   ·   " + t("cas.credits", { n: bank.refills() });
      const rf = this.fSmall.width(rightTxt) > W * 0.42 ? this.fTiny : this.fSmall;
      const rr = ui.text(ctx, rightTxt, W - pad, cy, rf, ui.TEXT_DIM, "midright");
      const mid = this.mode === "roulette" ? this._rHudText() : this._sHudText();
      if (mid) {
        const x0 = lr.right + pad, x1 = rr.x - pad;
        const f = this._fitFont(mid, x1 - x0, [this.fSmall, this.fTiny]);
        if (f.width(mid) <= x1 - x0) ui.text(ctx, mid, (x0 + x1) / 2, cy, f, ui.TEXT, "center");
      }
    }

    _drawFx(ctx) {
      for (const c of this.coinsFx) {
        const w = Math.max(2, c.r * 2 * Math.abs(Math.cos(c.spin)));
        const h = c.r * 2;
        draw.ellipse(ctx, D.COL_GOLD_D, [c.x - w / 2 + 1, c.y - h / 2 + 1, w, h]);
        draw.ellipse(ctx, [246, 196, 60], [c.x - w / 2, c.y - h / 2, w, h]);
        if (w > 6) draw.ellipse(ctx, [255, 236, 150], [c.x - w / 4, c.y - h / 4, w / 2, h / 2]);
      }
      for (const p of this.popups) {
        const f = p.t / p.dur;
        const alpha = f > 0.6 ? 1 - (f - 0.6) / 0.4 : 1;
        const y = p.y - 40 * (1 - (1 - f) ** 2);
        ui.text(ctx, p.text, p.x, y, this.fBig, p.color, "center", alpha);
      }
    }

    _drawToast(ctx) {
      const [text, left] = this.toast;
      const w = this.fSmall.width(text) + 24, h = this.fSmall.height + 12;
      const box = new PG.Rect(this.width / 2 - w / 2, this.strip.y - this.margin - h, w, h);
      const a = left < 0.3 ? left / 0.3 : 1;
      ctx.save();
      ctx.globalAlpha = a;
      draw.rect(ctx, [12, 8, 20, 225], box, 0, h / 2);
      draw.rect(ctx, ui.mix(D.COL_GOLD, this.accent, 0.4), box, 1, h / 2);
      ui.text(ctx, text, box.centerx, box.centery, this.fSmall, ui.TEXT, "center");
      ctx.restore();
    }

    _drawStrip(ctx) {
      const g = ctx.createLinearGradient(0, this.strip.y, 0, this.strip.bottom);
      g.addColorStop(0, "rgba(10,8,16,0.67)");
      g.addColorStop(1, "rgba(10,8,16,0.9)");
      ctx.fillStyle = g;
      ctx.fillRect(this.strip.x, this.strip.y, this.strip.w, this.strip.h);
      draw.line(ctx, ui.mix(D.COL_GOLD, this.accent, 0.35), this.strip.topleft, this.strip.topright, 2);
    }

    _btn(ctx, r, label, { primary = false, on = true, hot = false, active = false } = {}) {
      const rad = Math.max(6, Math.floor(r.h / 4));
      let col;
      if (primary && on) {
        const base = ui.mix(this.accent, [0, 0, 0], 0.18);
        draw.rect(ctx, ui.mix(base, [0, 0, 0], 0.4), [r.x, r.y + 2, r.w, r.h], 0, rad);
        draw.rect(ctx, ui.mix(base, [255, 255, 255], hot ? 0.12 : 0), r, 0, rad);
        draw.rect(ctx, ui.mix(D.COL_GOLD, [255, 255, 255], 0.35 * ui.pulse(2.4, 0, 1)), r, 2, rad);
        col = [255, 255, 255];
      } else {
        draw.rect(ctx, on && (hot || active) ? ui.BTN_SEL : ui.BTN, r, 0, rad);
        draw.rect(ctx, active ? D.COL_GOLD : on ? ui.BORDER_LIGHT : ui.BORDER, r, active ? 2 : 1, rad);
        col = on ? ui.TEXT : ui.TEXT_FAINT;
      }
      const f = this.fBtn.width(label) > r.w - 8 ? this.fTiny : this.fBtn;
      ui.text(ctx, label, r.centerx, r.centery, f, col, "center");
    }

    _hot(r) {
      return r.collidepoint(this.mouse);
    }

    _drawBroke(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const cx = this.width / 2, cy = this.height / 2;
      const pw = Math.min(this.width - 40, 480), ph = 200;
      const panel = new PG.Rect(cx - pw / 2, cy - ph / 2, pw, ph);
      ui.drawPanel(ctx, panel, { accentTop: ui.RED });
      ui.text(ctx, t("cas.broke"), cx, panel.y + ph * 0.28, this.fHuge, ui.RED, "center");
      const lines = [[t("cas.broke_sub"), ui.TEXT_DIM], [t("cas.broke_restart", { n: bank.START_CHIPS }), ui.TEXT]];
      lines.forEach(([txt, col], i) => {
        const f = this._fitFont(txt, pw - 16, [this.fSmall, this.fTiny]);
        ui.text(ctx, txt, cx, panel.y + ph * (0.58 + 0.2 * i), f, col, "center");
      });
    }

    // =====================================================================
    //  ROULETTE
    // =====================================================================
    _rInit() {
      const chip = this._opt("roulette_chip", 5);
      this.rChip = L.CHIP_VALUES.includes(chip) ? chip : 5;
      this.rPhase = R_BET;
      this.rBets = {};
      this.rLast = {};
      const hist = bank.getExtra(HISTORY_KEY, []);
      this.rHistory = Array.isArray(hist) ? hist.filter((n) => Number.isInteger(n) && n >= 0 && n <= 36).slice(0, L.HISTORY_LEN) : [];
      this.rHover = null;
      this.rRot = PG.rand.uniform(0, TAU);
      this.rOmega = IDLE_OMEGA;
      this.rBall = null;
      this.rTrail = [];
      this.rNumber = null;
      this.rWinners = [];
      this.rPayout = 0;
      this.rStake = 0;
      this.rT = 0;
      this.rFast = false;
      this.rResT = 0;
      this.rMsg = null;
      this.rTickI = null;
      this.rTickCd = 0;
      this.rDrops = {};
    }

    _rLayout() {
      const W = this.width, m = this.margin;
      const top = this.hudH + m, bottom = this.strip.y - m;
      const uw = (W - 2 * m) / L.TABLE_W;
      const uh = Math.min(uw * 0.82, ((bottom - top) * 0.56) / L.TABLE_H);
      this.rUw = uw;
      this.rUh = uh;
      this.rTx = m;
      this.rTy = bottom - uh * L.TABLE_H;
      const regionH = this.rTy - m - top;
      const wd = Math.floor(Math.max(60, Math.min(regionH, W * 0.4)));
      this.rWheelC = [m + wd / 2 + 6, top + regionH / 2];
      this.rWheel = new D.WheelArt(wd);
      const px = this.rWheelC[0] + wd / 2 + 2 * m;
      this.rPanel = new PG.Rect(px, top, W - m - px, regionH);
      this.rChipR = Math.max(6, Math.floor(Math.min(uw, uh) * 0.34));
      // Leiste unten
      const sy = this.strip.y, sh = this.stripH;
      const cr = Math.max(11, Math.floor(sh * 0.3));
      const gap = 8;
      this.rChipRects = [];
      let x = m + cr;
      for (const v of L.CHIP_VALUES) {
        this.rChipRects.push([v, new PG.Rect(x - cr, sy + sh / 2 - cr, 2 * cr, 2 * cr)]);
        x += 2 * cr + gap;
      }
      this.rChipRad = cr;
      const left = x - gap + m;
      const keys = ["clear", "double", "rebet", "spin"];
      let widths = keys.map((k) => this.fBtn.width(this._rBtnLabel(k)) + 22);
      widths[3] = Math.max(widths[3], 110);
      const bgap = 8;
      const room = W - m - left - bgap * (keys.length - 1);
      const sum = widths.reduce((a, b) => a + b, 0);
      if (sum > room) widths = widths.map((w) => Math.floor((w * room) / sum));
      const bh = Math.max(30, Math.floor(sh * 0.62));
      this.rBtns = {};
      x = W - m;
      for (let i = keys.length - 1; i >= 0; i--) {
        this.rBtns[keys[i]] = new PG.Rect(x - widths[i], sy + (sh - bh) / 2, widths[i], bh);
        x -= widths[i] + bgap;
      }
    }

    _rBtnLabel(key) {
      return { clear: t("cas.r.clear"), double: t("cas.r.double"), rebet: t("cas.r.rebet"), spin: t("cas.spin") }[key];
    }

    _rTable() {
      const uw = this.rUw, uh = this.rUh;
      return D.cached("rtable|" + uw + "|" + uh + "|" + PG.lang, () => {
        const tw = uw * L.TABLE_W, th = uh * L.TABLE_H;
        const [c, g] = D.canvas(tw + 8, th + 8);
        const ox = 4, oy = 4;
        const felt = ui.mix(D.COL_FELT, [255, 255, 255], 0.06);
        const line = [236, 230, 206];
        const lw = Math.max(1, Math.floor(Math.min(uw, uh) / 22));
        draw.rect(g, [0, 0, 0, 70], [ox + 2, oy + 3, tw, th], 0, 6);
        draw.rect(g, felt, [ox, oy, tw, th], 0, 6);
        const cell = (ux, uy, w = 1, h = 1) => new PG.Rect(ox + ux * uw, oy + uy * uh, w * uw, h * uh);
        const numFont = ui.font(Math.max(9, Math.floor(uh * 0.4)), true);
        const lblFont = ui.font(Math.max(9, Math.floor(Math.min(uh * 0.34, uw * 0.3))), true);
        const zero = [[ox + uw, oy], [ox + uw * 0.42, oy], [ox + 2, oy + 1.5 * uh], [ox + uw * 0.42, oy + 3 * uh], [ox + uw, oy + 3 * uh]];
        draw.polygon(g, D.COL_GREEN, zero);
        draw.polygon(g, line, zero, lw);
        ui.text(g, "0", ox + uw * 0.62, oy + 1.5 * uh, numFont, [255, 255, 255], "center");
        for (let col = 0; col < 12; col++) {
          for (let row = 0; row < 3; row++) {
            const n = L.cellNumber(col, row);
            const r = cell(1 + col, row);
            const pill = r.inflate(-Math.max(4, uw * 0.16), -Math.max(4, uh * 0.2));
            draw.rect(g, D.numberColor(n), pill, 0, Math.max(3, pill.h / 3));
            ui.text(g, String(n), r.centerx, r.centery, numFont, [255, 255, 255], "center");
            draw.rect(g, line, r, lw);
          }
        }
        for (let row = 0; row < 3; row++) {
          const r = cell(13, row);
          draw.rect(g, line, r, lw);
          ui.text(g, "2:1", r.centerx, r.centery, lblFont, [255, 255, 255], "center");
        }
        for (let d = 0; d < 3; d++) {
          const r = cell(1 + 4 * d, 3, 4, L.DOZEN_H);
          draw.rect(g, line, r, lw);
          ui.text(g, 12 * d + 1 + "–" + (12 * d + 12), r.centerx, r.centery, lblFont, [255, 255, 255], "center");
        }
        L.OUTSIDE_KEYS.forEach((key, i) => {
          const r = cell(1 + 2 * i, 3 + L.DOZEN_H, 2, L.OUTSIDE_H);
          draw.rect(g, line, r, lw);
          if (key === "red" || key === "black") {
            const dw = r.w * 0.26, dh = r.h * 0.34;
            const pts = [[r.centerx, r.centery - dh], [r.centerx + dw, r.centery], [r.centerx, r.centery + dh], [r.centerx - dw, r.centery]];
            draw.polygon(g, key === "red" ? D.COL_RED : D.COL_BLACK, pts);
            draw.polygon(g, line, pts, lw);
            return;
          }
          const txt = { low: "1–18", high: "19–36", even: t("cas.r.even"), odd: t("cas.r.odd") }[key];
          let f = lblFont;
          if (f.width(txt) > r.w - 6) f = ui.font(Math.max(8, Math.floor(lblFont.height * 0.72)), true);
          ui.text(g, txt, r.centerx, r.centery, f, [255, 255, 255], "center");
        });
        draw.rect(g, D.COL_GOLD, [ox - 1, oy - 1, tw + 2, th + 2], lw + 1, 6);
        return c;
      });
    }

    _rTotal() {
      let s = 0;
      for (const k in this.rBets) s += this.rBets[k];
      return s;
    }
    _rToPx(ux, uy) {
      return [this.rTx + ux * this.rUw, this.rTy + uy * this.rUh];
    }
    _rKeyAt(pos) {
      if (!pos) return null;
      return L.hitTest((pos[0] - this.rTx) / this.rUw, (pos[1] - this.rTy) / this.rUh);
    }
    _rBetName(key) {
      const kind = L.betKind(key);
      if (["red", "black", "even", "odd"].includes(kind)) return t("cas.r." + kind);
      if (kind === "low") return t("cas.r.low") + " 1–18";
      if (kind === "high") return t("cas.r.high") + " 19–36";
      const arg = key.split(":")[1];
      if (kind === "column") return t("cas.r.column") + " " + (3 - Number(arg));
      if (kind === "dozen") {
        const d = Number(arg);
        return t("cas.r.dozen") + " " + (12 * d + 1) + "–" + (12 * d + 12);
      }
      return t("cas.r." + kind) + " " + L.betNumbers(key).join(kind === "cheval" ? "/" : "-");
    }
    _rNumberText(n) {
      if (n === 0) return t("cas.r.zero");
      return [t("cas.r." + L.colorOf(n)), t(n % 2 === 0 ? "cas.r.even" : "cas.r.odd"), t(n <= 18 ? "cas.r.low" : "cas.r.high")].join(" · ");
    }
    _rHudText() {
      return t("cas.bet") + ": " + (this.rPhase === R_BET ? this._rTotal() : this.rStake);
    }

    // ----- Aktionen
    _rSelectChip(v) {
      if (v !== this.rChip) {
        this.rChip = v;
        this._setOpt("roulette_chip", v);
        this.playSound("select");
      }
    }
    _rPlace(key) {
      const avail = bank.balance() - this._rTotal();
      if (avail <= 0) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      this.rBets[key] = (this.rBets[key] || 0) + Math.min(this.rChip, avail);
      this.rDrops[key] = 0;
      this.playSound("click");
    }
    _rRemove(key) {
      if (!(key in this.rBets)) return;
      const left = this.rBets[key] - this.rChip;
      if (left > 0) this.rBets[key] = left;
      else delete this.rBets[key];
      this.playSound("move");
    }
    _rClear() {
      if (Object.keys(this.rBets).length) {
        this.rBets = {};
        this.playSound("move");
      }
    }
    _rRebet() {
      const keys = Object.keys(this.rLast);
      if (!keys.length) return;
      let need = 0;
      for (const k of keys) need += this.rLast[k];
      if (need > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      this.rBets = Object.assign({}, this.rLast);
      for (const k of keys) this.rDrops[k] = 0;
      this.playSound("click");
    }
    _rDouble() {
      const keys = Object.keys(this.rBets);
      if (!keys.length) return;
      if (this._rTotal() * 2 > bank.balance()) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      for (const k of keys) {
        this.rBets[k] *= 2;
        this.rDrops[k] = 0;
      }
      this.playSound("merge");
    }
    _rSpin() {
      const stake = this._rTotal();
      if (stake <= 0) {
        this._say(t("cas.r.place_first"));
        return;
      }
      if (!bank.debit(stake, "casino")) {
        this._say(t("cas.not_enough"));
        this.playSound("hit");
        return;
      }
      this.rStake = stake;
      this.rLast = Object.assign({}, this.rBets);
      this.rNumber = PG.rand.randint(0, 36);
      const [payout, winners] = L.settle(this.rBets, this.rNumber);
      this.rPayout = payout;
      this.rWinners = winners;
      bank.credit(payout, "casino");
      this.pending += payout;
      this.rOmega = SPIN_OMEGA;
      const beta0 = PG.rand.uniform(0, TAU);
      this.rPhi0 = beta0 - this.rRot;
      const target = this.rWheel.pocketAngle(this.rNumber);
      const d = PG.mod(this.rPhi0 - target, TAU);
      this.rPhiEnd = this.rPhi0 - d - BALL_TURNS * TAU;
      this.rT = 0;
      this.rFast = false;
      this.rTrail = [];
      this.rTickI = null;
      this.rPhase = R_SPIN;
      this.rMsg = [t("cas.r.no_more"), ui.TEXT_DIM];
      this.playSound("shoot");
    }
    _rBallState(u) {
      const wa = this.rWheel, R = wa.R;
      let phi = this.rPhiEnd + (this.rPhi0 - this.rPhiEnd) * (1 - u) ** 3;
      if (u > 0.62) phi += 0.16 * Math.sin(u * 53) * (1 - u) * ((u - 0.62) / 0.38);
      const rim = R * 0.74;
      let r;
      if (u < 0.52) {
        r = wa.trackR - R * 0.004 * Math.sin(u * 60);
      } else if (u < 0.8) {
        let v = (u - 0.52) / 0.28;
        v = v * v * (3 - 2 * v);
        r = wa.trackR + (rim - wa.trackR) * v;
      } else {
        const v = (u - 0.8) / 0.2;
        r = rim + (wa.pocketR - rim) * (v * v * (3 - 2 * v)) + R * 0.06 * Math.abs(Math.sin(v * 3.2 * Math.PI)) * (1 - v) ** 1.5;
      }
      return [phi, r];
    }
    _rLand() {
      const n = this.rNumber;
      this.pending = Math.max(0, this.pending - this.rPayout);
      this.rPhase = R_RESULT;
      this.rResT = 0;
      this.rHistory.unshift(n);
      this.rHistory.length = Math.min(this.rHistory.length, L.HISTORY_LEN);
      bank.setExtra(HISTORY_KEY, this.rHistory.slice());
      this.score = bank.scoreFor("casino");
      this.playSound("lock");
      if (this.rPayout > 0) {
        const plein = this.rWinners.some((k) => L.betKind(k) === "plein");
        const big = plein || this.rPayout >= 15 * this.rStake;
        this.rMsg = [t("cas.win") + ": +" + this.rPayout, ui.GOLD];
        this.playSound(big ? "win" : "point");
        for (const key of this.rWinners) {
          const [ax, ay] = this._rToPx(...L.anchor(key));
          this._popup(ax, ay - this.rChipR * 1.5, "+" + this.rBets[key] * (L.payoutMultiplier(key) + 1));
        }
        if (plein) this.achEvent("roulette_plein");
        if (big) {
          ui.spawnConfetti(this.width, this.height);
          this._coinShower(plein ? 46 : 30);
          this.rumble(160);
        }
      } else {
        this.rMsg = [t("cas.no_win"), ui.TEXT_DIM];
        this.playSound("select");
      }
    }
    _rNewRound() {
      this.rPhase = R_BET;
      this.rBets = {};
      this.rDrops = {};
      this.rMsg = null;
      this._checkBroke();
    }

    _rEvent(ev) {
      const kind = ev.kind;
      if (kind === "mousemove") {
        this.rHover = this.rPhase === R_BET ? this._rKeyAt(ev.pos) : null;
        return;
      }
      if (this.rPhase === R_SPIN) {
        if (kind === "mousedown" || isConfirm(ev)) this.rFast = true;
        return;
      }
      if (this.rPhase === R_RESULT) {
        if (kind === "mousedown" && ev.button === 1) {
          const hit = Object.keys(this.rBtns).find((k) => this.rBtns[k].collidepoint(ev.pos));
          this._rNewRound();
          if (hit === "rebet") this._rRebet();
          else if (hit === "spin") {
            this._rRebet();
            if (Object.keys(this.rBets).length && !this.broke) this._rSpin();
          }
        } else if (kind === "keydown" && ["Return", "space", "KP_Enter", "r", "R"].includes(ev.key)) {
          this._rNewRound();
          if (ev.key === "r" || ev.key === "R") this._rRebet();
        }
        return;
      }
      if (kind === "wheel") {
        let i = L.CHIP_VALUES.indexOf(this.rChip);
        i = Math.max(0, Math.min(L.CHIP_VALUES.length - 1, i + (ev.delta > 0 ? 1 : -1)));
        this._rSelectChip(L.CHIP_VALUES[i]);
      } else if (kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4", "5"].includes(k)) this._rSelectChip(L.CHIP_VALUES[Number(k) - 1]);
        else if (isConfirm(ev)) this._rSpin();
        else if (["BackSpace", "Delete", "c", "C"].includes(k)) this._rClear();
        else if (k === "r" || k === "R") this._rRebet();
        else if (k === "d" || k === "D") this._rDouble();
      } else if (kind === "mousedown") {
        const pos = ev.pos;
        if (ev.button === 3) {
          const key = this._rKeyAt(pos);
          if (key) this._rRemove(key);
          return;
        }
        for (const [v, r] of this.rChipRects) {
          if (r.collidepoint(pos)) {
            this._rSelectChip(v);
            return;
          }
        }
        for (const name of Object.keys(this.rBtns)) {
          if (this.rBtns[name].collidepoint(pos)) {
            ({ clear: () => this._rClear(), double: () => this._rDouble(), rebet: () => this._rRebet(), spin: () => this._rSpin() })[name]();
            return;
          }
        }
        const key = this._rKeyAt(pos);
        if (key) this._rPlace(key);
      }
    }

    _rUpdate(dt) {
      this.rOmega += (IDLE_OMEGA - this.rOmega) * (1 - Math.exp(-dt / OMEGA_TAU));
      for (const k of Object.keys(this.rDrops)) {
        this.rDrops[k] += dt * 5;
        if (this.rDrops[k] >= 1) delete this.rDrops[k];
      }
      if (this.rPhase === R_SPIN) {
        const step = dt * (this.rFast ? 3.5 : 1);
        this.rRot += this.rOmega * step;
        this.rT += step;
        const u = Math.min(1, this.rT / LAND_T);
        const [phi, rad] = this._rBallState(u);
        const ball = [this.rRot + phi, rad];
        if (u < 0.9) {
          this.rTrail.push(ball);
          if (this.rTrail.length > 4) this.rTrail.shift();
        } else {
          this.rTrail = [];
        }
        this.rBall = ball;
        if (u > 0.55) {
          const idx = Math.floor(phi / (TAU / 37));
          this.rTickCd -= dt;
          if (this.rTickI !== null && idx !== this.rTickI && this.rTickCd <= 0) {
            this.rTickCd = 0.05 + 0.1 * u;
            this.tone(1900 + 400 * Math.random(), 0.015, "square", 0.05);
          }
          this.rTickI = idx;
        }
        if (u >= 1) this._rLand();
      } else {
        this.rRot += this.rOmega * dt;
        if (this.rNumber !== null) this.rBall = [this.rRot + this.rWheel.pocketAngle(this.rNumber), this.rWheel.pocketR];
        if (this.rPhase === R_RESULT) {
          this.rResT += dt;
          if (this.rResT >= RESULT_T) this._rNewRound();
        }
      }
    }

    _rCellRect(n) {
      if (n === 0) return new PG.Rect(this.rTx, this.rTy, this.rUw, 3 * this.rUh);
      const [x, y] = this._rToPx(1 + L.numCol(n), L.numRow(n));
      return new PG.Rect(x, y, this.rUw, this.rUh);
    }
    _rKeyRect(key) {
      const kind = L.betKind(key);
      if (kind === "column") {
        const [x, y] = this._rToPx(13, Number(key.split(":")[1]));
        return new PG.Rect(x, y, this.rUw, this.rUh);
      }
      if (kind === "dozen") {
        const [x, y] = this._rToPx(1 + 4 * Number(key.split(":")[1]), 3);
        return new PG.Rect(x, y, this.rUw * 4, this.rUh * L.DOZEN_H);
      }
      if (L.OUTSIDE_KEYS.includes(kind)) {
        const [x, y] = this._rToPx(1 + 2 * L.OUTSIDE_KEYS.indexOf(kind), 3 + L.DOZEN_H);
        return new PG.Rect(x, y, this.rUw * 2, this.rUh * L.OUTSIDE_H);
      }
      return null;
    }
    _rHighlight(ctx, key, color) {
      const r = this._rKeyRect(key);
      if (r) draw.rect(ctx, color, r);
      for (const n of L.betNumbers(key)) draw.rect(ctx, color, this._rCellRect(n));
    }

    _rDraw(ctx) {
      PG.cards.blitFelt(ctx, this, this.width, this.height, D.COL_FELT, D.COL_FELT_D);
      this.rWheel.draw(ctx, this.rWheelC, this.rRot, this.rBall, this.rPhase === R_SPIN ? this.rTrail : null);
      this._rDrawPanel(ctx);
      this._rDrawTable(ctx);
      this._rDrawStrip(ctx);
    }

    _rDrawTable(ctx) {
      const table = this._rTable();
      ctx.drawImage(table, this.rTx - 4, this.rTy - 4, table.lw, table.lh);
      const cr = this.rChipR;
      if (this.rPhase === R_BET && this.rHover) this._rHighlight(ctx, this.rHover, [255, 255, 255, 60]);
      if (this.rPhase === R_RESULT && this.rNumber !== null) {
        for (const key of this.rWinners) if (this._rKeyRect(key)) this._rHighlight(ctx, key, [255, 214, 90, 80]);
        const r = this._rCellRect(this.rNumber);
        draw.rect(ctx, ui.mix(D.COL_GOLD, [255, 255, 255], ui.pulse(6, 0, 1)), r.inflate(4, 4), 3, 4);
        const dx = r.centerx, dy = r.centery;
        const dr = Math.max(5, Math.min(this.rUw, this.rUh) * 0.22);
        draw.ellipse(ctx, [0, 0, 0, 200], [dx - dr, dy - dr / 2 + 3, 2 * dr, dr]);
        draw.rect(ctx, [230, 230, 236], [dx - dr * 0.6, dy - dr * 1.4, dr * 1.2, dr * 1.4]);
        draw.ellipse(ctx, [250, 250, 255], [dx - dr * 0.6, dy - dr * 1.7, dr * 1.2, dr * 0.6]);
      }
      for (const key of Object.keys(this.rBets)) {
        const [ax, ay] = this._rToPx(...L.anchor(key));
        let alpha = 1, lift = 0;
        if (key in this.rDrops) lift = (1 - this.rDrops[key]) ** 2 * cr * 1.4;
        if (this.rPhase === R_RESULT && !this.rWinners.includes(key)) {
          alpha = Math.max(0, 1 - this.rResT / 0.8);
          if (alpha <= 0) continue;
        }
        D.drawChipStack(ctx, [ax, ay], this.rBets[key], cr, alpha, lift);
        if (this.rPhase === R_RESULT && this.rWinners.includes(key)) {
          draw.circle(ctx, D.COL_GOLD, [ax, ay - 2], cr + 3 + 3 * ui.pulse(5, 0, 1), 2);
        }
      }
      if (this.rPhase === R_BET && this.rHover && !(this.rHover in this.rBets)) {
        const [ax, ay] = this._rToPx(...L.anchor(this.rHover));
        D.drawChip(ctx, this.rChip, cr, ax, ay, undefined, 0.5);
      }
    }

    _rDrawPanel(ctx) {
      const p = this.rPanel;
      if (p.w < 40 || p.h < 40) return;
      let y = p.y;
      ui.text(ctx, t("cas.r.history"), p.x, y, this.fTiny, ui.TEXT_DIM);
      y += this.fTiny.height + 4;
      const gap = 4;
      let perRow = 12;
      let size = Math.floor((p.w - gap * (perRow - 1)) / perRow);
      if (size < 18) {
        perRow = 6;
        size = Math.floor((p.w - gap * (perRow - 1)) / perRow);
      }
      size = Math.max(12, Math.min(size, 34));
      const hf = ui.font(Math.max(9, Math.floor(size * 0.52)), true);
      for (let i = 0; i < L.HISTORY_LEN; i++) {
        const rect = new PG.Rect(p.x + (i % perRow) * (size + gap), y + Math.floor(i / perRow) * (size + gap), size, size);
        const rad = Math.max(3, Math.floor(size / 4));
        if (i < this.rHistory.length) {
          const n = this.rHistory[i];
          draw.rect(ctx, D.numberColor(n), rect, 0, rad);
          if (i === 0) draw.rect(ctx, D.COL_GOLD, rect, 2, rad);
          ui.text(ctx, String(n), rect.centerx, rect.centery, hf, [255, 255, 255], "center");
        } else {
          draw.rect(ctx, ui.mix(D.COL_FELT_D, [0, 0, 0], 0.3), rect, 1, rad);
        }
      }
      y += Math.ceil(L.HISTORY_LEN / perRow) * (size + gap) + 12;
      const showN = this.rPhase === R_RESULT ? this.rNumber : null;
      const bigR = Math.max(14, Math.min(34, Math.floor((p.bottom - y) / 3)));
      if (showN !== null) {
        const c = [p.x + bigR, y + bigR];
        draw.circle(ctx, [0, 0, 0], [c[0] + 2, c[1] + 3], bigR);
        draw.circle(ctx, D.numberColor(showN), c, bigR);
        draw.circle(ctx, D.COL_GOLD, c, bigR, 2);
        ui.text(ctx, String(showN), c[0], c[1], ui.font(Math.max(12, Math.floor(bigR * 0.95)), true), [255, 255, 255], "center");
        const desc = this._rNumberText(showN);
        const f = this._fitFont(desc, p.w - 2 * bigR - 10, [this.fSmall, this.fTiny]);
        ui.text(ctx, desc, p.x + 2 * bigR + 10, c[1], f, ui.TEXT, "midleft");
        y += 2 * bigR + 8;
      }
      let msg = this.rMsg;
      if (!msg && this.rPhase === R_BET) msg = [t("cas.r.place_bets"), ui.TEXT];
      if (msg && y < p.bottom) {
        const f = this._fitFont(msg[0], p.w, [this.fBig, this.fSmall]);
        ui.text(ctx, msg[0], p.x, y, f, msg[1]);
        y += f.height + 4;
      }
      let info = null;
      if (this.rPhase === R_BET && this.rHover) {
        const key = this.rHover;
        info = this._rBetName(key) + " · " + L.payoutMultiplier(key) + ":1";
        if (key in this.rBets) info += " · " + this.rBets[key];
      } else if (this.rPhase === R_SPIN) {
        info = t("cas.r.skip_hint");
      } else if (this.rPhase === R_BET) {
        info = t("cas.r.place_hint");
      }
      if (info) {
        const f = this._fitFont(info, p.w, [this.fTiny, ui.font(10)]);
        const yy = Math.max(y, p.bottom - f.height);
        if (yy + f.height <= p.bottom + 2) ui.text(ctx, info, p.x, yy, f, ui.TEXT_DIM);
      }
    }

    _rDrawStrip(ctx) {
      this._drawStrip(ctx);
      const cr = this.rChipRad;
      for (const [v, rect] of this.rChipRects) {
        const sel = v === this.rChip;
        const [cx, cy] = rect.center;
        const lift = sel ? 5 : 0;
        if (sel) draw.circle(ctx, ui.mix(D.COL_GOLD, [255, 255, 255], ui.pulse(3, 0, 0.5)), [cx, cy - lift], cr + 3, 2);
        D.drawChip(ctx, v, cr - (sel ? 0 : 2), cx, cy - lift);
      }
      const betting = this.rPhase === R_BET, result = this.rPhase === R_RESULT;
      const hasBets = Object.keys(this.rBets).length > 0, hasLast = Object.keys(this.rLast).length > 0;
      const on = { clear: betting && hasBets, double: betting && hasBets, rebet: (betting || result) && hasLast,
        spin: (betting && hasBets) || (result && hasLast) };
      for (const key of Object.keys(this.rBtns)) {
        const r = this.rBtns[key];
        this._btn(ctx, r, this._rBtnLabel(key), { primary: key === "spin", on: on[key], hot: this._hot(r) });
      }
    }

    // =====================================================================
    //  LAMA-SLOT
    // =====================================================================
    _sInit() {
      const bet = this._opt("line_bet", 1);
      this.sBet = L.LINE_BETS.includes(bet) ? bet : 1;
      this.sTurbo = !!this._opt("turbo", false);
      this.sPhase = S_IDLE;
      this.sStops = L.randomStops(PG.rand);
      this.sPos = this.sStops.map(Number);
      this.sReels = [];
      this.sT = 0;
      this.sSpeed = 1;
      this.sRes = null;
      this.sFreeSpin = false;
      this.sShowT = 0;
      this.sWinShown = 0;
      this.sAuto = 0;
      this.sAutoMenu = false;
      this.sPaytable = false;
      this.sBanners = [];
      this.sAnticip = new Set();
      this.sHoverLine = null;
      let free = bank.getExtra(FREE_KEY, {});
      free = free && typeof free === "object" ? free : {};
      this.sFree = Math.max(0, toIntOr(free.left, 0));
      this.sFreeBet = L.LINE_BETS.includes(free.bet) ? free.bet : this.sBet;
      this.sFreeWon = Math.max(0, toIntOr(free.won, 0));
      this.sFsActive = this.sFree > 0;
      if (this.sFree > 0) this._sBanner(t("cas.s.free_left", { n: this.sFree }), null, [255, 150, 200]);
    }

    _sLayout() {
      const W = this.width, m = this.margin;
      const top = this.hudH + m, bottom = this.strip.y - m;
      const tb = 44, ib = 40, tab = 26, pad = 12, gap = 6;
      const availH = bottom - top;
      const cell = Math.floor(Math.max(24, Math.min((availH - tb - ib - pad) / 3, (W - 2 * m - 2 * tab - 2 * pad - 4 * gap) / 5)));
      const reelsW = 5 * cell + 4 * gap;
      const cabW = reelsW + 2 * tab + 2 * pad, cabH = tb + 3 * cell + ib + pad;
      const cab = new PG.Rect(0, 0, cabW, cabH);
      cab.center = [Math.floor(W / 2), Math.floor(top + availH / 2)];
      Object.assign(this, { sCell: cell, sGap: gap, sTab: tab, sPad: pad, sTb: tb, sIb: ib, sCab: cab });
      this.sWin = new PG.Rect(cab.x + pad + tab, cab.y + tb, reelsW, 3 * cell);
      this.sReelRects = [];
      for (let r = 0; r < L.REELS; r++) this.sReelRects.push(new PG.Rect(this.sWin.x + r * (cell + gap), this.sWin.y, cell, 3 * cell));
      const fpad = gap + 3;
      this.sInfo = new PG.Rect(cab.x + pad, this.sWin.bottom + fpad, cabW - 2 * pad, ib - fpad);
      // Nummern-Reiter
      this.sTabs = {};
      for (const side of [0, 1]) {
        const perRow = { 0: [], 1: [], 2: [] };
        L.LINES.forEach((line, i) => perRow[line[side === 0 ? 0 : L.REELS - 1]].push(i));
        for (const row of [0, 1, 2]) {
          const lines = perRow[row];
          const th = Math.min(Math.floor(cell / 4.4), Math.floor(tab * 0.9));
          const total = lines.length * th + (lines.length - 1) * 2;
          const y0 = this.sWin.y + row * cell + Math.floor((cell - total) / 2);
          const x = side === 0 ? this.sWin.x - tab - 1 : this.sWin.right + 1;
          lines.forEach((i, j) => (this.sTabs[side + "|" + i] = new PG.Rect(x, y0 + j * (th + 2), tab, th)));
        }
      }
      const band = new PG.Rect(cab.x + 30, cab.y + 6, cab.w - 60, tb - 12);
      this.sTitleBand = band;
      this.sBulbs = [];
      const nb = Math.max(8, Math.floor(band.w / 22));
      for (let i = 0; i < nb; i++) {
        const x = band.x + Math.floor(((i + 0.5) * band.w) / nb);
        this.sBulbs.push([x, band.y - 3], [x, band.bottom + 3]);
      }
      // Leiste
      const sy = this.strip.y, sh = this.stripH;
      const bh = Math.max(30, Math.floor(sh * 0.62));
      const by = sy + (sh - bh) / 2;
      const lblW = Math.max(this.fTiny.width(t("cas.s.line_bet")), this.fBtn.width("10 (= 100)")) + 10;
      this.sMinus = new PG.Rect(m, by, bh, bh);
      this.sBetBox = new PG.Rect(this.sMinus.right + 4, by, lblW, bh);
      this.sPlus = new PG.Rect(this.sBetBox.right + 4, by, bh, bh);
      const left = this.sPlus.right + m;
      const keys = ["pay", "turbo", "auto", "spin"];
      let widths = [t("cas.s.paytable"), t("cas.s.turbo"), t("cas.s.auto"), t("cas.spin")].map((s) => this.fBtn.width(s) + 22);
      widths[2] = Math.max(widths[2], this.fBtn.width(t("cas.s.auto") + " 25") + 22);
      widths[3] = Math.max(widths[3], this.fBtn.width(t("cas.s.stop")) + 22, 110);
      const bgap = 8;
      const room = W - m - left - bgap * (keys.length - 1);
      const sum = widths.reduce((a, b) => a + b, 0);
      if (sum > room) widths = widths.map((w) => Math.floor((w * room) / sum));
      this.sBtns = {};
      let x = W - m;
      for (let i = keys.length - 1; i >= 0; i--) {
        this.sBtns[keys[i]] = new PG.Rect(x - widths[i], by, widths[i], bh);
        x -= widths[i] + bgap;
      }
      const a = this.sBtns.auto;
      const mh = Math.floor(bh * 0.9);
      this.sAutoOpts = L.AUTO_COUNTS.map((n, i) => [n, new PG.Rect(a.x, this.strip.y - (i + 1) * (mh + 4), a.w, mh)]);
    }

    _sLineBet() {
      return this.sFree > 0 || this.sFreeSpin ? this.sFreeBet : this.sBet;
    }
    _sHudText() {
      if (this.sFree > 0 || this.sFreeSpin) return t("cas.s.free_left", { n: this.sFree });
      return t("cas.bet") + ": " + this._sLineBet() * L.LINES.length;
    }
    _sBanner(text, sub, color, dur = 2.2, kind = "info", amount = 0) {
      this.sBanners.push({ text, sub, color, t: 0, dur: dur * (this.sTurbo ? 0.6 : 1), kind, amount });
    }
    _sSaveFree() {
      if (this.sFree > 0 || this.sFsActive) bank.setExtra(FREE_KEY, { left: this.sFree, bet: this.sFreeBet, won: this.sFreeWon });
      else bank.setExtra(FREE_KEY, {});
    }
    _sSetBet(dir) {
      if (this.sPhase === S_SPIN || this.sFree > 0 || this.sFsActive) return;
      const i = L.LINE_BETS.indexOf(this.sBet);
      const j = Math.max(0, Math.min(L.LINE_BETS.length - 1, i + dir));
      if (j !== i) {
        this.sBet = L.LINE_BETS[j];
        this._setOpt("line_bet", this.sBet);
        this.playSound("select");
      }
    }
    _sToggleTurbo() {
      this.sTurbo = !this.sTurbo;
      this._setOpt("turbo", this.sTurbo);
      this.playSound("click");
    }
    _sStartAuto(n) {
      this.sAutoMenu = false;
      this.sAuto = n;
      this.playSound("select");
      if (this.sPhase !== S_SPIN && !this.sBanners.length) this._sSpin();
    }

    _sSpin() {
      if (this.sPhase === S_SPIN) {
        this.sSpeed = 3.2;
        return;
      }
      if (this.sBanners.length) {
        const b = this.sBanners[0];
        b.t = Math.max(b.t, b.dur - 0.25);
        return;
      }
      const free = this.sFree > 0;
      let bet;
      if (free) {
        this.sFree -= 1;
        bet = this.sFreeBet;
      } else {
        bet = this.sBet;
        if (!bank.debit(bet * L.LINES.length, "casino")) {
          this.sAuto = 0;
          this._say(t("cas.not_enough"));
          this.playSound("hit");
          this._checkBroke();
          return;
        }
        if (this.sAuto > 0) this.sAuto -= 1;
      }
      const stops = L.randomStops(PG.rand);
      const res = L.evaluate(L.window(stops), bet, free);
      bank.credit(res.total, "casino");
      this.pending += res.total;
      this.sFreeSpin = free;
      if (res.free_spins) {
        if (!this.sFsActive && !free) this.sFreeWon = 0;
        this.sFree += res.free_spins;
        this.sFreeBet = bet;
        this.sFsActive = true;
      }
      if (free || res.free_spins) {
        bank.setExtra(FREE_KEY, { left: this.sFree, bet: this.sFreeBet, won: this.sFreeWon + (free ? res.total : 0) });
      }
      this.sRes = res;
      this.sStops = stops;
      this._sSchedule(stops);
      this.sPhase = S_SPIN;
      this.sT = 0;
      this.sSpeed = 1;
      this.sAutoMenu = false;
      this.playSound("rotate");
    }

    _sSchedule(stops) {
      const turbo = this.sTurbo;
      const base = turbo ? 0.34 : 0.78, gap = turbo ? 0.1 : 0.24, extra = turbo ? 0.45 : 1.05;
      const speed = SPEED * (turbo ? 1.5 : 1);
      const win = L.window(stops);
      this.sReels = [];
      this.sAnticip = new Set();
      let delay = 0, coins = 0;
      for (let r = 0; r < L.REELS; r++) {
        if (r >= 2 && coins >= 2) {
          delay += extra;
          this.sAnticip.add(r);
        }
        const T = base + r * gap + delay;
        const n = L.STRIPS[r].length;
        const p0 = this.sPos[r];
        const nominal = speed * (T - DECEL / 2);
        const dist = nominal + PG.mod(p0 - nominal - stops[r], n);
        this.sReels.push({ p0, dist, v: dist / (T - DECEL / 2), T, target: stops[r], landed: false });
        coins += win[r].filter((s) => s === L.SCATTER).length;
      }
    }

    _sReelPos(reel, tt) {
      const { T, v } = reel;
      if (tt <= T - DECEL) return [reel.p0 - v * tt, v];
      if (tt < T) {
        const u = (tt - (T - DECEL)) / DECEL;
        return [reel.p0 - (v * (T - DECEL) + v * DECEL * (u - (u * u) / 2)), v * (1 - u)];
      }
      const b = (tt - T) / BOUNCE;
      if (b < 1) return [reel.target - 0.14 * Math.sin(Math.PI * b) * (1 - b), 0];
      return [reel.target, 0];
    }

    _sReveal() {
      const res = this.sRes;
      this.pending = Math.max(0, this.pending - res.total);
      this.sPhase = res.total > 0 ? S_SHOW : S_IDLE;
      this.sShowT = 0;
      this.sWinShown = 0;
      if (this.sFreeSpin) this.sFreeWon += res.total;
      this.score = bank.scoreFor("casino");
      const mult = res.total / Math.max(1, this._sLineBet() * L.LINES.length);
      if (res.jackpot) {
        this._sBanner(t("cas.s.jackpot"), null, [255, 214, 90], 3.2, "win", res.total);
        this.achEvent("slots_jackpot");
        ui.spawnConfetti(this.width, this.height, 140);
        this._coinShower(90);
        this.playSound("level");
        this.rumble(300);
      } else if (mult >= MEGA_WIN) {
        this._sBanner(t("cas.s.mega_win"), null, [255, 150, 220], 2.8, "win", res.total);
        ui.spawnConfetti(this.width, this.height, 110);
        this._coinShower(70);
        this.playSound("win");
        this.rumble(220);
      } else if (mult >= BIG_WIN) {
        this._sBanner(t("cas.s.big_win"), null, [255, 214, 90], 2.4, "win", res.total);
        ui.spawnConfetti(this.width, this.height);
        this._coinShower(40);
        this.playSound("win");
      } else if (res.total > 0) {
        this.playSound("point");
      }
      if (res.free_spins) {
        this._sBanner(t("cas.s.free_won", { n: res.free_spins }), t("cas.s.free_info"), [255, 150, 200], 2.4);
        this.playSound("powerup");
      }
      if (this.sFreeSpin && this.sFree === 0) this._sSaveFree();
    }

    _sEndFree() {
      this.sFsActive = false;
      this._sBanner(t("cas.s.free_end"), "+" + this.sFreeWon, [255, 214, 90], 2.6, "win", this.sFreeWon);
      if (this.sFreeWon >= BIG_WIN * this.sFreeBet * L.LINES.length) {
        ui.spawnConfetti(this.width, this.height);
        this._coinShower(40);
      }
      this.playSound("level");
      this.sFreeWon = 0;
      this._sSaveFree();
    }

    _sEvent(ev) {
      const kind = ev.kind;
      if (kind === "mousemove") {
        this.sHoverLine = null;
        for (const k of Object.keys(this.sTabs)) if (this.sTabs[k].collidepoint(ev.pos)) this.sHoverLine = Number(k.split("|")[1]);
        return;
      }
      if (this.sPaytable) {
        if (kind === "mousedown" || (kind === "keydown" && ["i", "I", "p", "P", "Return", "space", "KP_Enter"].includes(ev.key))) {
          this.sPaytable = false;
          this.playSound("click");
        }
        return;
      }
      if (kind === "keydown") {
        const k = ev.key;
        if (isConfirm(ev)) {
          if (this.sAuto > 0 && this.sPhase !== S_SPIN) this.sAuto = 0;
          else this._sSpin();
        } else if (["plus", "KP_Add", "Up", "equal"].includes(k)) this._sSetBet(1);
        else if (["minus", "KP_Subtract", "Down"].includes(k)) this._sSetBet(-1);
        else if (k === "t" || k === "T") this._sToggleTurbo();
        else if (k === "a" || k === "A") {
          if (this.sAuto) this.sAuto = 0;
          else this._sStartAuto(L.AUTO_COUNTS[k === "a" ? 0 : 1]);
        } else if (["i", "I", "p", "P"].includes(k)) {
          this.sPaytable = true;
          this.sAutoMenu = false;
          this.playSound("click");
        }
        return;
      }
      if (kind === "wheel") {
        this._sSetBet(ev.delta > 0 ? 1 : -1);
        return;
      }
      if (kind !== "mousedown" || ev.button !== 1) return;
      const pos = ev.pos;
      if (this.sAutoMenu) {
        for (const [n, r] of this.sAutoOpts) {
          if (r.collidepoint(pos)) {
            this._sStartAuto(n);
            return;
          }
        }
        this.sAutoMenu = false;
        return;
      }
      if (this.sMinus.collidepoint(pos)) this._sSetBet(-1);
      else if (this.sPlus.collidepoint(pos)) this._sSetBet(1);
      else if (this.sBtns.pay.collidepoint(pos)) {
        this.sPaytable = true;
        this.playSound("click");
      } else if (this.sBtns.turbo.collidepoint(pos)) this._sToggleTurbo();
      else if (this.sBtns.auto.collidepoint(pos)) {
        if (this.sAuto) {
          this.sAuto = 0;
          this.playSound("move");
        } else {
          this.sAutoMenu = true;
          this.playSound("click");
        }
      } else if (this.sBtns.spin.collidepoint(pos) || this.sWin.collidepoint(pos)) {
        if (this.sAuto > 0 && this.sPhase !== S_SPIN) this.sAuto = 0;
        else this._sSpin();
      }
    }

    _sUpdate(dt) {
      if (this.sBanners.length) {
        const b = this.sBanners[0];
        b.t += dt;
        if (b.t >= b.dur) this.sBanners.shift();
      }
      if (this.sPhase === S_SPIN) {
        this.sT += dt * this.sSpeed;
        let last = 0;
        this.sReels.forEach((reel, r) => {
          this.sPos[r] = this._sReelPos(reel, this.sT)[0];
          if (!reel.landed && this.sT >= reel.T) {
            reel.landed = true;
            this.playSound("lock");
            if (this.sAnticip.has(r + 1)) this.tone(420 + 90 * r, 0.22, "sine", 0.18);
          }
          last = Math.max(last, reel.T);
        });
        if (this.sT >= last + BOUNCE) {
          this.sReels.forEach((reel, r) => (this.sPos[r] = reel.target));
          this._sReveal();
        }
        return;
      }
      this.sShowT += dt;
      if (this.sRes && this.sRes.total > 0) {
        const target = this.sRes.total;
        const step = Math.max((target * dt) / (this.sTurbo ? 0.5 : 1.1), 30 * dt);
        this.sWinShown = Math.min(target, this.sWinShown + step);
      }
      if (this.broke || this.sPaytable || this.sBanners.length) return;
      const won = this.sRes && this.sRes.total > 0;
      const wait = (won ? 1.7 : 0.55) * (this.sTurbo ? 0.5 : 1);
      if (this.sFsActive && this.sFree === 0 && this.sShowT >= wait) {
        this._sEndFree();
        return;
      }
      if (this.sShowT < wait) return;
      if (this.sFree > 0) this._sSpin();
      else if (this.sAuto > 0) {
        this._sSpin();
        if (this.sPhase !== S_SPIN) this.sAuto = 0;
      } else if (!this.broke && bank.isBroke("casino", "slots") && this.sFree === 0 && this.pending === 0) {
        this._checkBroke();
      }
    }

    // ----- Zeichnen
    _sHighlights() {
      const res = this.sRes;
      if (this.sPhase !== S_SHOW || !res) return null;
      const entries = res.lines.map((e) => ["line", ...e]);
      if (res.scatter_win > 0) entries.push(["scatter", res.scatter, res.scatter_win]);
      if (!entries.length) return null;
      const first = this.sTurbo ? 0.6 : 1, step = this.sTurbo ? 0.6 : 1;
      const tt = this.sShowT;
      let shown, phaseT, text;
      if (tt < first || entries.length === 1) {
        shown = entries;
        phaseT = tt;
        text = t("cas.win") + ": " + Math.floor(this.sWinShown);
      } else {
        const idx = Math.floor((tt - first) / step) % entries.length;
        shown = [entries[idx]];
        phaseT = (tt - first) % step;
        const e = entries[idx];
        text = e[0] === "line"
          ? t("cas.s.line_win", { line: e[1] + 1, n: e[3], sym: t("cas.sym." + e[2]), win: e[4] })
          : t("cas.s.scatter_win", { n: e[1], win: e[2] });
      }
      const cells = new Set();
      const lines = [];
      for (const e of shown) {
        if (e[0] === "line") {
          lines.push(e[1]);
          for (let r = 0; r < e[3]; r++) cells.add(r + "," + L.LINES[e[1]][r]);
        } else {
          for (const [r, row] of res.coins) cells.add(r + "," + row);
        }
      }
      return { cells, lines, text, phaseT };
    }

    _sBackground() {
      return D.cached("sbg|" + this.width + "|" + this.height, () => {
        const W = this.width, H = this.height;
        const [c, g] = D.canvas(W, H);
        const grad = g.createLinearGradient(0, 0, 0, H);
        grad.addColorStop(0, "rgb(46,16,58)");
        grad.addColorStop(1, "rgb(12,6,20)");
        g.fillStyle = grad;
        g.fillRect(0, 0, W, H);
        g.strokeStyle = "rgba(255,255,255,0.028)";
        g.lineWidth = 2;
        for (let x = -H; x < W; x += 34) {
          g.beginPath();
          g.moveTo(x, 0);
          g.lineTo(x + H, H);
          g.stroke();
        }
        draw.ellipse(g, [255, 120, 190, 22], [W * 0.1, H * 0.12, W * 0.8, H * 0.7]);
        return c;
      });
    }

    _sCabinet() {
      const cab = this.sCab;
      return D.cached("scab|" + cab.x + "|" + cab.y + "|" + cab.w + "|" + cab.h, () => {
        const [c, g] = D.canvas(cab.w + 16, cab.h + 16);
        const o = 8;
        const body = new PG.Rect(o, o, cab.w, cab.h);
        const rad = 22;
        draw.rect(g, [0, 0, 0, 110], [body.x + 3, body.y + 6, body.w, body.h], 0, rad);
        const grad = g.createLinearGradient(0, body.y, 0, body.bottom);
        grad.addColorStop(0, "rgb(92,30,96)");
        grad.addColorStop(1, "rgb(38,12,48)");
        ui.roundPath(g, body.x, body.y, body.w, body.h, rad);
        g.fillStyle = grad;
        g.fill();
        draw.rect(g, D.COL_GOLD_D, body, 5, rad);
        draw.rect(g, D.COL_GOLD, body.inflate(-4, -4), 2, rad);
        const win = new PG.Rect(this.sWin.x - cab.x + o, this.sWin.y - cab.y + o, this.sWin.w, this.sWin.h);
        const frame = win.inflate(this.sGap * 2 + 6, this.sGap * 2 + 6);
        draw.rect(g, [20, 8, 24], frame, 0, 10);
        draw.rect(g, D.COL_GOLD, frame, 3, 10);
        for (const rr of this.sReelRects) {
          const x = rr.x - cab.x + o, y = rr.y - cab.y + o;
          const rg = g.createLinearGradient(0, y, 0, y + rr.h);
          rg.addColorStop(0, "rgb(206,196,180)");
          rg.addColorStop(0.5, "rgb(252,248,238)");
          rg.addColorStop(1, "rgb(206,196,180)");
          g.fillStyle = rg;
          g.fillRect(x, y, rr.w, rr.h);
        }
        const band = new PG.Rect(this.sTitleBand.x - cab.x + o, this.sTitleBand.y - cab.y + o, this.sTitleBand.w, this.sTitleBand.h);
        draw.rect(g, [26, 8, 30], band, 0, band.h / 2);
        draw.rect(g, D.COL_GOLD_D, band, 2, band.h / 2);
        return c;
      });
    }

    _sDraw(ctx) {
      const bg = this._sBackground();
      ctx.drawImage(bg, 0, 0, bg.lw, bg.lh);
      const cabC = this._sCabinet();
      ctx.drawImage(cabC, this.sCab.x - 8, this.sCab.y - 8, cabC.lw, cabC.lh);
      this._sDrawTitle(ctx);
      const hl = this._sHighlights();
      const phaseT = hl ? hl.phaseT : 0;
      const popScale = phaseT < 0.35 ? 1 + 0.13 * Math.sin(Math.PI * Math.min(1, phaseT / 0.35)) : 1;
      this.sReelRects.forEach((rect, r) => {
        let speed = 0;
        if (this.sPhase === S_SPIN && r < this.sReels.length) speed = this._sReelPos(this.sReels[r], this.sT)[1];
        let dim = null, pop = null;
        if (hl) {
          dim = new Set();
          pop = {};
          for (let row = 0; row < L.ROWS; row++) {
            if (hl.cells.has(r + "," + row)) pop[row] = popScale;
            else dim.add(row);
          }
        }
        D.drawReel(ctx, rect, L.STRIPS[r], this.sPos[r], this.sCell, speed * this.sSpeed, dim, pop);
        if (this.sPhase === S_SPIN && this.sAnticip.has(r) && !this.sReels[r].landed) {
          draw.rect(ctx, ui.mix(D.COL_GOLD, [255, 255, 255], ui.pulse(9, 0, 1)), rect.inflate(6, 6), 3, 6);
        }
      });
      this._sDrawTabs(ctx, hl);
      if (hl) this._sDrawWin(ctx, hl);
      else if (this.sHoverLine !== null && this.sPhase !== S_SPIN) this._sDrawLinePath(ctx, this.sHoverLine, 2);
      this._sDrawInfo(ctx, hl);
      this._sDrawStrip(ctx);
      if (this.sBanners.length) this._sDrawBanner(ctx, this.sBanners[0]);
      if (this.sPaytable) this._sDrawPaytable(ctx);
    }

    _sDrawTitle(ctx) {
      const band = this.sTitleBand;
      const tick = Math.floor(ui.ticks() / (this.sFsActive ? 90 : 160));
      this.sBulbs.forEach(([x, y], i) => {
        const on = (Math.floor(i / 2) + tick) % 3 === 0;
        if (on) draw.circle(ctx, [255, 200, 90], [x, y], 5);
        draw.circle(ctx, on ? [255, 236, 150] : [120, 80, 40], [x, y], 3);
      });
      let title, top, bot;
      if (this.sFsActive) {
        title = t("cas.s.free_title") + "  " + this.sFree;
        top = [255, 190, 230];
        bot = [230, 80, 150];
      } else {
        title = t("cas.mode.slots").toUpperCase();
        top = [255, 240, 170];
        bot = [226, 150, 40];
      }
      let f = ui.font(Math.max(13, Math.floor(band.h * 0.72)), true);
      if (f.width(title) > band.w - 10) f = ui.font(Math.max(11, Math.floor(band.h * 0.52)), true);
      ui.gradText(ctx, title, band.centerx, band.centery, f, top, bot, "center");
    }

    _sDrawTabs(ctx, hl) {
      const active = new Set(hl ? hl.lines : []);
      const f = ui.font(Math.max(8, Math.floor(this.sTabs["0|0"].h * 0.8)), true);
      for (const k of Object.keys(this.sTabs)) {
        const i = Number(k.split("|")[1]);
        const r = this.sTabs[k];
        const col = D.LINE_COLS[i];
        const on = active.has(i) || i === this.sHoverLine;
        draw.rect(ctx, on ? col : ui.mix(col, [30, 10, 30], 0.55), r, 0, Math.max(2, r.h / 3));
        ui.text(ctx, String(i + 1), r.centerx, r.centery, f, on ? [20, 10, 20] : [240, 230, 240], "center");
      }
    }

    _sLinePoints(i) {
      const pts = [this.sTabs["0|" + i].midright];
      this.sReelRects.forEach((rect, r) => pts.push([rect.centerx, rect.y + L.LINES[i][r] * this.sCell + this.sCell / 2]));
      pts.push(this.sTabs["1|" + i].midleft);
      return pts;
    }

    _sDrawLinePath(ctx, i, width) {
      const pts = this._sLinePoints(i);
      const w = Math.max(2, width * 1.5);
      draw.lines(ctx, [20, 8, 24], false, pts, w + 4);
      draw.lines(ctx, D.LINE_COLS[i], false, pts, w);
    }

    _sDrawWin(ctx, hl) {
      for (const i of hl.lines) this._sDrawLinePath(ctx, i, 3);
      const col = hl.lines.length === 1 ? D.LINE_COLS[hl.lines[0]] : D.COL_GOLD;
      const pulse = ui.pulse(7, 0, 1);
      for (const key of hl.cells) {
        const [r, row] = key.split(",").map(Number);
        const rect = new PG.Rect(this.sReelRects[r].x, this.sWin.y + row * this.sCell, this.sCell, this.sCell).inflate(-2, -2);
        draw.rect(ctx, ui.mix(col, [255, 255, 255], pulse * 0.5), rect, 3, 8);
      }
    }

    _sDrawInfo(ctx, hl) {
      const r = this.sInfo;
      let text, col;
      if (hl) {
        text = hl.text;
        col = ui.GOLD;
      } else if (this.sPhase === S_SPIN) {
        text = this.sFsActive ? t("cas.s.free_title") + "  +" + this.sFreeWon : t("cas.s.good_luck");
        col = ui.TEXT_DIM;
      } else if (this.sFsActive) {
        text = t("cas.s.free_total", { n: this.sFreeWon });
        col = [255, 190, 230];
      } else if (this.sAuto) {
        text = t("cas.s.auto_left", { n: this.sAuto });
        col = ui.TEXT_DIM;
      } else if (this.sRes && this.sRes.total === 0) {
        text = t("cas.no_win");
        col = ui.TEXT_DIM;
      } else {
        text = t("cas.s.hint");
        col = ui.TEXT_DIM;
      }
      const f = this._fitFont(text, r.w - 8, hl ? [this.fBig, this.fSmall, this.fTiny] : [this.fSmall, this.fTiny]);
      ui.text(ctx, text, r.centerx, r.centery, f, col, "center");
    }

    _sDrawStrip(ctx) {
      this._drawStrip(ctx);
      const locked = this.sPhase === S_SPIN || this.sFree > 0 || this.sFsActive;
      const bet = this._sLineBet();
      this._btn(ctx, this.sMinus, "–", { on: !locked && bet > L.LINE_BETS[0], hot: this._hot(this.sMinus) });
      this._btn(ctx, this.sPlus, "+", { on: !locked && bet < L.LINE_BETS[L.LINE_BETS.length - 1], hot: this._hot(this.sPlus) });
      const box = this.sBetBox;
      const y0 = box.centery - (this.fTiny.height + this.fBtn.height) / 2;
      ui.text(ctx, t("cas.s.line_bet"), box.centerx, y0, this.fTiny, ui.TEXT_DIM, "midtop");
      ui.text(ctx, bet + " (= " + bet * L.LINES.length + ")", box.centerx, y0 + this.fTiny.height, this.fBtn, ui.GOLD, "midtop");
      const spinning = this.sPhase === S_SPIN;
      const autoLbl = t("cas.s.auto") + (this.sAuto ? " " + this.sAuto : "");
      const spinLbl = this.sAuto && !spinning ? t("cas.s.stop") : t("cas.spin");
      this._btn(ctx, this.sBtns.pay, t("cas.s.paytable"), { hot: this._hot(this.sBtns.pay) });
      this._btn(ctx, this.sBtns.turbo, t("cas.s.turbo"), { active: this.sTurbo, hot: this._hot(this.sBtns.turbo) });
      this._btn(ctx, this.sBtns.auto, autoLbl, { active: !!this.sAuto, hot: this._hot(this.sBtns.auto) });
      this._btn(ctx, this.sBtns.spin, spinLbl, { primary: true, on: !this.sBanners.length || spinning, hot: this._hot(this.sBtns.spin) });
      if (this.sAutoMenu) {
        for (const [n, r] of this.sAutoOpts) this._btn(ctx, r, t("cas.s.auto") + " " + n, { hot: this._hot(r) });
      }
    }

    _sDrawBanner(ctx, b) {
      const cab = this.sCab;
      const f = b.t / Math.max(0.01, b.dur);
      const appear = Math.min(1, b.t / 0.25);
      const fade = f < 0.85 ? 1 : Math.max(0, (1 - f) / 0.15);
      const h = Math.floor(cab.h * 0.42);
      const y = cab.centery - h / 2;
      ctx.save();
      ctx.globalAlpha = fade;
      draw.rect(ctx, [12, 4, 18, Math.floor(210 * appear)], [cab.x, y, cab.w, h]);
      draw.line(ctx, b.color, [cab.x, y], [cab.right, y], 2);
      draw.line(ctx, b.color, [cab.x, y + h - 2], [cab.right, y + h - 2], 2);
      const scale = 0.6 + 0.4 * (1 - (1 - appear) ** 3) + 0.04 * Math.sin(b.t * 8);
      let size = Math.max(16, Math.floor(this.fHuge.height * scale));
      let font = ui.font(size, true);
      if (font.width(b.text) > cab.w - 20) font = ui.font(Math.max(14, Math.floor((size * (cab.w - 20)) / font.width(b.text))), true);
      let sub = b.sub;
      if (b.kind === "win" && b.amount) sub = "+" + Math.floor(b.amount * Math.min(1, b.t / (b.dur * 0.6)));
      if (sub) {
        const sf = this._fitFont(sub, cab.w - 20, [this.fBig, this.fSmall]);
        ui.gradText(ctx, b.text, cab.centerx, cab.centery - sf.height / 2, font, [255, 255, 255], b.color, "center");
        ui.text(ctx, sub, cab.centerx, cab.centery + font.height / 2 - sf.height / 2, sf, [255, 244, 220], "midtop");
      } else {
        ui.gradText(ctx, b.text, cab.centerx, cab.centery, font, [255, 255, 255], b.color, "center");
      }
      ctx.restore();
    }

    _sDrawPaytable(ctx) {
      draw.rect(ctx, [6, 4, 12, 200], [0, 0, this.width, this.height]);
      const W = Math.min(this.width - 2 * this.margin, 640);
      const H = this.strip.y - this.hudH - 2 * this.margin;
      const panel = new PG.Rect(this.width / 2 - W / 2, (this.hudH + this.strip.y) / 2 - H / 2, W, H);
      ui.drawPanel(ctx, panel, { accentTop: D.COL_GOLD, shadow: false });
      const bet = this._sLineBet();
      const pad = 12;
      ui.text(ctx, t("cas.s.paytable_title"), panel.centerx, panel.y + pad, this.fBig, D.COL_GOLD, "midtop");
      const y = panel.y + pad + this.fBig.height + 4;
      const foot = [t("cas.s.wild_info"), t("cas.s.scatter_info"), t("cas.s.rtp", { p: L.RTP_TEXT })];
      const footH = foot.length * (this.fTiny.height + 1);
      const rows = ["lama", "coin", ...L.SYMBOLS.slice(2)];
      const cols = W > 440 ? 2 : 1;
      const perCol = Math.ceil(rows.length / cols);
      const rowH = Math.floor((panel.bottom - y - footH - pad * 2) / perCol);
      const icon = Math.max(12, Math.min(rowH - 4, 46));
      const colW = Math.floor((W - 2 * pad) / cols);
      const f = rowH >= 22 ? this.fSmall : this.fTiny;
      rows.forEach((sym, idx) => {
        const x = panel.x + pad + Math.floor(idx / perCol) * colW;
        const yy = y + (idx % perCol) * rowH;
        const img = D.symbolCanvas(sym, icon);
        ctx.drawImage(img, x, yy + (rowH - icon) / 2, icon, icon);
        const pays = sym === L.SCATTER ? [3, 4, 5].map((n) => L.SCATTER_PAYS[n] * bet * L.LINES.length) : L.PAYS[sym].map((p) => p * bet);
        const txt = [3, 4, 5].map((n, i) => n + "× " + pays[i]).join("   ");
        const tf = this._fitFont(txt, colW - icon - 12, [f, this.fTiny]);
        ui.text(ctx, txt, x + icon + 8, yy + rowH / 2, tf, ui.TEXT, "midleft");
      });
      let fy = panel.bottom - pad - footH;
      for (const line of foot) {
        const lf = this._fitFont(line, W - 2 * pad, [this.fTiny, ui.font(10)]);
        ui.text(ctx, line, panel.centerx, fy, lf, ui.TEXT_DIM, "midtop");
        fy += this.fTiny.height + 1;
      }
    }
  }

  PG.register(CasinoGame, {
    id: "CasinoGame",
    key: "casino",
    name: "Casino",
    modes: [["roulette", "cas.mode.roulette"], ["slots", "cas.mode.slots"]],
    settingsKey: "casino",
    defaults: { roulette_chip: 5, line_bet: 1, turbo: false },
    wantsRightClick: true,
  });
})();
