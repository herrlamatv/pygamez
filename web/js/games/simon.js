/*
 * simon.js - Simon / Senso (Port von games/simon.py)
 * ===================================================
 * Das klassische Merkspiel mit leuchtenden Feldern.
 *
 * Die Felder leuchten in einer wachsenden Reihenfolge auf; diese muss exakt
 * nachgetippt werden. Jede Runde kommt ein Feld hinzu.
 *
 * Spielmodi (Web-Version: nur Einzelspieler, das Duell entfällt):
 *   - Klassisch : Reihenfolge exakt nachtippen.
 *   - Speed     : Die Wiedergabe wird von Runde zu Runde schneller.
 *   - Reverse   : Die Reihenfolge muss RÜCKWÄRTS eingegeben werden.
 *   - Gemischt  : Der Modus (Klassisch/Speed/Reverse) wechselt jede Runde.
 *
 * Ton: "Aus" (nur visuell), "An" (jedes Feld klingt) oder "Gemischt" (mal mit,
 * mal ohne Ton - trainiert visuelles UND akustisches Gedächtnis). Der globale
 * Sound-Schalter in den Optionen hat zusätzlich Vorrang.
 *
 * Feldanzahl 4, 6 oder 9 stellt die Schwierigkeit ein.
 *
 * Punkte (Highscore) = längste fehlerfrei wiederholte Folge.
 * Steuerung: Maus (Feld anklicken) oder Zifferntasten 1-9.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ----------------------------------------------------------------- Farben
  // Die Leuchtfarben der Felder sind die Identität des Spiels und bleiben
  // fest; alle generischen UI-Farben kommen dynamisch aus der ui-Palette.
  const PAD_COLORS = [
    [46, 204, 113], [231, 76, 60], [241, 196, 15], [52, 152, 219],
    [155, 89, 182], [230, 126, 34], [26, 188, 156], [233, 96, 160],
    [149, 200, 60],
  ];
  // Tonfrequenzen (C-Dur-Pentatonik aufsteigend), Hz.
  const FREQS = [261.63, 293.66, 329.63, 392.0, 440.0, 523.25, 587.33, 659.25, 783.99];

  const SETUP = "setup", PLAY = "play", OVER = "over";
  const MODES = ["classic", "speed", "reverse", "mixed"];
  const AUDIO_MODES = ["off", "on", "mixed"];
  const PAD_OPTS = [4, 6, 9];
  const GRID = { 4: [2, 2], 6: [3, 2], 9: [3, 3] };
  const SUBS = ["classic", "speed", "reverse"]; // mögliche Runden-Modi bei "Gemischt"

  const BEST_KEY = "simon_best"; // Bestwerte je Modus (wie store.save_section("simon"))

  const dim = (c, f) => [Math.floor(c[0] * f), Math.floor(c[1] * f), Math.floor(c[2] * f)];
  const lit = (c) => [Math.min(255, c[0] + 90), Math.min(255, c[1] + 90), Math.min(255, c[2] + 90)];

  class SimonGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const sm = this.opts;
      this.mode = MODES.includes(sm.mode) ? sm.mode : "classic";
      this.audioMode = AUDIO_MODES.includes(sm.audio) ? sm.audio : "on";
      this.pads = PAD_OPTS.includes(Number(sm.pads)) ? Number(sm.pads) : 4;

      this.buildFonts();
      this.best = this.loadBest();
      this.buildSetupLayout();
      this.layout();
      this.state = SETUP;
      // Grundzustand, damit draw()/update() vor dem Start nicht scheitern.
      this.seq = [];
      this.expected = [];
      this.phase = "input";
      this.litPad = null;
      this.litT = 0;
      this.msg = null;
      this.msgT = 0;
      this.inputI = 0;
      this.roundSound = true;
      this.active = this.mode !== "mixed" ? this.mode : "classic";
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, Math.floor(h / 32)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 40)));
      this.huge = ui.font(Math.max(28, Math.floor(h / 10)), true);
    }

    loadBest() {
      const b = PG.store.get(BEST_KEY, {});
      return b && typeof b === "object" ? b : {};
    }

    saveBest() {
      PG.store.set(BEST_KEY, this.best);
    }

    layout() {
      this.hudH = 46;
      const [cols, rows] = GRID[this.pads];
      const area = Math.min(this.width - 60, this.height - this.hudH - 50);
      this.padGap = Math.max(8, Math.floor(area / 40));
      const n = Math.max(cols, rows);
      const cell = (area - this.padGap * (n - 1)) / n;
      this.padW = cell;
      const gw = cols * cell + (cols - 1) * this.padGap;
      const gh = rows * cell + (rows - 1) * this.padGap;
      this.gridX = Math.floor((this.width - gw) / 2);
      this.gridY = this.hudH + Math.max(12, Math.floor((this.height - this.hudH - gh) / 2));
      this.cols = cols;
      this.rows = rows;
      this.padRects = [];
      for (let i = 0; i < this.pads; i++) {
        const r = Math.floor(i / cols), c = i % cols;
        const x = this.gridX + c * (cell + this.padGap);
        const y = this.gridY + r * (cell + this.padGap);
        this.padRects.push(new PG.Rect(Math.floor(x), Math.floor(y), Math.floor(cell), Math.floor(cell)));
      }
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(380, this.width - 50);
      const y0 = Math.floor(this.height * 0.26);
      const gap = 8;
      const row = (y, n) => {
        const cw = (bw - gap * (n - 1)) / n;
        const out = [];
        for (let i = 0; i < n; i++) out.push(new PG.Rect(Math.floor(cx - bw / 2 + i * (cw + gap)), y, Math.floor(cw), 40));
        return out;
      };
      this.modeRects = row(y0, MODES.length);
      this.audioRects = row(y0 + 82, 3);
      this.padRectsSetup = row(y0 + 150, 3);
      this.startRect = new PG.Rect(cx - 95, y0 + 210, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        const num = /^[1-9]$/.test(k) ? Number(k) : 0;
        if (num >= 1 && num <= MODES.length) {
          this.mode = MODES[num - 1];
          this.saveSetting("mode", this.mode);
          this.playSound("click");
        } else if (k === "a" || k === "A") {
          this.audioMode = AUDIO_MODES[(AUDIO_MODES.indexOf(this.audioMode) + 1) % 3];
          this.saveSetting("audio", this.audioMode);
          this.playSound("select");
        } else if (k === "p" || k === "P") {
          this.pads = PAD_OPTS[(PAD_OPTS.indexOf(this.pads) + 1) % 3];
          this.saveSetting("pads", this.pads);
          this.layout();
          this.playSound("select");
        } else if (k === "Return" || k === "space") {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.modeRects.length; i++) {
          if (this.modeRects[i].collidepoint(ev.pos)) {
            this.mode = MODES[i];
            this.saveSetting("mode", this.mode);
            this.playSound("click");
            return;
          }
        }
        for (let i = 0; i < this.audioRects.length; i++) {
          if (this.audioRects[i].collidepoint(ev.pos)) {
            this.audioMode = AUDIO_MODES[i];
            this.saveSetting("audio", this.audioMode);
            this.playSound("select");
            return;
          }
        }
        for (let i = 0; i < this.padRectsSetup.length; i++) {
          if (this.padRectsSetup[i].collidepoint(ev.pos)) {
            this.pads = PAD_OPTS[i];
            this.saveSetting("pads", this.pads);
            this.layout();
            this.playSound("select");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    startPlay() {
      this.layout();
      this.state = PLAY;
      this.gameOver = false;
      this.score = 0;
      this.litPad = null;
      this.litT = 0;
      this.msg = null;
      this.msgT = 0;
      this.seq = [PG.rand.randrange(this.pads)];
      this.beginRound();
      this.playSound("click");
    }

    // ===================================================== Einzelspiel-Runden
    beginRound() {
      // Runden-Untermodus (bei "Gemischt" wechselnd)
      this.active = this.mode === "mixed" ? PG.rand.choice(SUBS) : this.mode;
      const rnd = this.seq.length;
      if (this.active === "speed") {
        this.litDur = Math.max(0.14, 0.44 - 0.02 * rnd);
        this.gapDur = Math.max(0.06, 0.18 - 0.008 * rnd);
      } else {
        this.litDur = 0.42;
        this.gapDur = 0.18;
      }
      this.roundSound = this.audioMode !== "mixed" || PG.rand.random() < 0.5;
      this.expected = this.active === "reverse" ? this.seq.slice().reverse() : this.seq.slice();
      this.phase = "show";
      this.showI = 0;
      this.showState = "pre";
      this.showT = 0.55;
      this.inputI = 0;
      this.litPad = null;
    }

    updateShow(dt) {
      this.showT -= dt;
      if (this.showT > 0) return;
      if (this.showState === "pre") {
        this.lightShow(0);
      } else if (this.showState === "on") {
        this.litPad = null;
        this.showState = "off";
        this.showT = this.gapDur;
        this.showI += 1;
      } else {
        // off
        if (this.showI >= this.seq.length) {
          this.phase = "input";
          this.inputI = 0;
          this.litPad = null;
        } else {
          this.lightShow(this.showI);
        }
      }
    }

    lightShow(i) {
      this.litPad = this.seq[i];
      this.showState = "on";
      this.showT = this.litDur;
      this.padTone(this.seq[i]);
    }

    // ===================================================== Ton
    padTone(i) {
      if (this.audioMode === "off") return;
      if (this.audioMode === "mixed" && !this.roundSound) return;
      this.tone(FREQS[i % FREQS.length], 0.22, "sine");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.startPlay();
          } else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.gameOver = false;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown") {
          this.startPlay();
        }
        return;
      }
      if (this.state !== PLAY) return;
      if (this.phase !== "input") return; // während der Wiedergabe keine Eingabe
      let pad = null;
      if (ev.kind === "mousedown") {
        const i = this.padRects.findIndex((rc) => rc.collidepoint(ev.pos));
        if (i >= 0) pad = i;
      } else if (ev.kind === "keydown") {
        if (/^[1-9]$/.test(ev.key)) {
          const idx = Number(ev.key) - 1;
          if (idx < this.pads) pad = idx;
        }
      }
      if (pad == null) return;
      this.singlePress(pad);
    }

    singlePress(pad) {
      this.litPad = pad;
      this.litT = 0.2;
      this.padTone(pad);
      if (pad === this.expected[this.inputI]) {
        this.inputI += 1;
        if (this.inputI >= this.expected.length) {
          // Runde geschafft
          this.score = this.seq.length;
          this.playSound("point");
          this.seq.push(PG.rand.randrange(this.pads));
          this.beginRound();
        }
      } else {
        this.failSingle();
      }
    }

    failSingle() {
      this.playSound("gameover");
      const key = this.mode;
      if ((this.best[key] || 0) < this.score) {
        this.best[key] = this.score;
        this.saveBest();
      }
      this.state = OVER;
      this.gameOver = true;
    }

    // ===================================================== Update
    update(dt) {
      if (this.msgT > 0) this.msgT -= dt;
      if (this.litT > 0) {
        this.litT -= dt;
        if (this.litT <= 0 && this.phase === "input") this.litPad = null;
      }
      if (this.state === PLAY && this.phase === "show") this.updateShow(dt);
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawHud(ctx);
      this.drawPads(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawPads(ctx) {
      this.padRects.forEach((rc, i) => {
        const col = PAD_COLORS[i];
        const on = this.litPad === i;
        draw.rect(ctx, on ? lit(col) : dim(col, 0.5), rc, 0, 14);
        draw.rect(ctx, dim(col, 0.8), rc, 3, 14);
        if (on) draw.rect(ctx, [255, 255, 255, 60], rc, 0, 14); // Glow (abgerundet wie das Feld)
        ui.text(ctx, String(i + 1), rc.centerx, rc.centery, this.small, on ? [255, 255, 255] : dim(col, 0.85), "center");
      });
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = this.hudH / 2;
      ui.text(ctx, t("simon.round", { n: this.seq.length }), 14, cy, this.small, ui.TEXT, "midleft");
      ui.text(ctx, t("simon.best", { n: this.best[this.mode] || 0 }), this.width - 14, cy, this.small, ui.TEXT_DIM, "midright");
      let mid = this.phase === "show" ? t("simon.watch") : t("simon.your_input");
      if (this.mode === "mixed") mid += "  ·  " + t("simon.mode." + this.active);
      ui.text(ctx, mid, this.width / 2, cy, this.small, this.accent, "center");
    }

    drawOver(ctx) {
      const y = Math.floor(this.height / 2) - 54;
      draw.rect(ctx, [10, 8, 16, 210], [0, y, this.width, 108]);
      draw.line(ctx, this.accent, [0, y], [this.width, y]);
      draw.line(ctx, this.accent, [0, y + 107], [this.width, y + 107]);
      const cx = this.width / 2;
      ui.text(ctx, t("simon.score", { n: this.score }), cx, y + 34, this.huge, this.accent, "center");
      ui.text(ctx, t("simon.best", { n: this.best[this.mode] || 0 }), cx, y + 68, this.small, ui.TEXT, "center");
      ui.text(ctx, t("simon.new_round"), cx, y + 92, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = this.width / 2;
      ui.text(ctx, t("simon.title"), cx, Math.floor(this.height * 0.12), this.huge, this.accent, "center");
      ui.text(ctx, t("simon.subtitle"), cx, Math.floor(this.height * 0.185), this.small, ui.TEXT_DIM, "center");

      const label = (y, txt) => ui.text(ctx, txt, cx, y - 4, this.tiny, ui.TEXT_DIM, "midbottom");

      label(this.modeRects[0].top, t("simon.lbl_mode"));
      this.modeRects.forEach((rc, i) => this.btn(ctx, rc, t("simon.mode." + MODES[i]), this.mode === MODES[i], this.tiny));
      label(this.audioRects[0].top, t("simon.lbl_audio"));
      this.audioRects.forEach((rc, i) => this.btn(ctx, rc, t("simon.audio." + AUDIO_MODES[i]), this.audioMode === AUDIO_MODES[i], this.small));
      label(this.padRectsSetup[0].top, t("simon.lbl_pads"));
      this.padRectsSetup.forEach((rc, i) => this.btn(ctx, rc, String(PAD_OPTS[i]), this.pads === PAD_OPTS[i], this.small));
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("simon.setup_hint"), cx, this.height - 14, this.tiny, ui.TEXT_DIM, "center");
    }

    btn(ctx, rc, text, on, fnt) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      ui.text(ctx, text, rc.centerx, rc.centery, fnt, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }
  }

  PG.register(SimonGame, {
    id: "SimonGame",
    key: "simon",
    name: "Simon",
    settingsKey: "simon",
    defaults: { mode: "classic", audio: "on", pads: 4 },
  });
})();
