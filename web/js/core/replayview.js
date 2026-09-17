/*
 * replayview.js - der Replay-Screen: Archiv und Wiedergabe
 * ============================================================================
 * Port von replayview.py auf die logische Fläche 800 x 600.
 *
 * Zwei Ansichten in einem Screen:
 *
 * - Liste  : Reiter je Spiel (Minigolf, Bowling, Billard, Pinball, Snake,
 *            Tetris) mit allen gespeicherten Replays - Titel, Ergebnis, Datum
 *            und Laufzeit. Enter spielt ab, Entf löscht (zweimal drücken),
 *            Tab wechselt das Spiel, E teilt eine Aufnahme als Datei und I
 *            liest eine geteilte Datei wieder ein (".lamapgzreplay").
 * - Player : die eigentliche Wiedergabe. Gezeichnet wird sie vom Spiel selbst:
 *            der Screen baut eine ganz normale Spielinstanz und fährt sie über
 *            replayBegin / replaySeek / replayDraw Bild für Bild durch die
 *            Aufnahme.
 *
 * Sequenzen mit Zielvorgang (Minigolf: ein Schlag, Bowling: ein Wurf,
 * Billard: ein Stoß) bekommen einen kurzen Vorlauf mit Ziellinie und einen
 * Nachlauf mit dem Ergebnis. Snake und Tetris laufen durch; dort sind die
 * Sequenzen Kapitel und werden ohne Pause aneinandergehängt (siehe PAD).
 */
(function () {
  "use strict";

  const PG = window.PG;
  const ui = PG.ui;
  const draw = PG.draw;
  const t = PG.t;
  const W = PG.W, H = PG.H;
  const replay = PG.replay;

  // Klassennamen der Spiele mit Aufzeichnung (Reihenfolge = replay.GAMES).
  const CLASSES = {
    minigolf: "MiniGolfGame", bowling: "BowlingGame", billiard: "BilliardGame",
    pinball: "PinballGame", snake: "SnakeGame", tetris: "TetrisGame",
  };

  // Zusatzbilder je Sequenz (in Samples, also 1/replay.RATE Sekunden):
  // [Vorlauf mit Ziellinie, kurzer Nachlauf, langer Nachlauf am Sequenzende].
  // Spiele, die durchlaufen, bekommen keinen Vor-/Nachlauf.
  const PAD = {
    minigolf: [12, 9, 27], bowling: [12, 9, 27], billiard: [12, 9, 27],
    pinball: [6, 6, 27], snake: [0, 0, 27], tetris: [0, 0, 27],
  };

  // Beschriftung des Sequenz-Zählers in der Bedienleiste.
  const SEQ_KEYS = {
    minigolf: "replay.seq_golf", bowling: "replay.seq_bowl", billiard: "replay.seq_bil",
    pinball: "replay.seq_pin", snake: "replay.seq_part", tetris: "replay.seq_part",
  };

  const SPEEDS = [0.5, 1.0, 2.0, 4.0];
  const ROW_H = 46;
  const BAR_H = 40;
  const BAR_SHOW = 3.0;

  function gameEntry(key) {
    return PG.gameById[CLASSES[key] || ""] || null;
  }

  function gameName(key) {
    const e = gameEntry(key);
    return e ? PG.gameName(e) : key;
  }

  function gameAccent(key) {
    return ui.gameColor(CLASSES[key] || "");
  }

  /** Archiv + Wiedergabe. opts.pending = frische, noch ungespeicherte Aufnahme. */
  class ReplayScreen {
    constructor(app, opts, onClose) {
      opts = opts || {};
      this.app = app;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.onClose = onClose || (() => app.backToMenu());
      this.pending = opts.pending || null;
      this.saved = false;
      this.name = t("replay.name");
      this.tab = opts.game || (this.pending && this.pending.game) || replay.GAMES[0];
      if (!replay.GAMES.includes(this.tab)) this.tab = replay.GAMES[0];
      this.mode = "list";
      this.sel = 0;
      this.first = 0;
      this.tabHover = null;
      this.btnHover = null;
      this.hoverRow = null;
      this.confirm = null;
      this.toast = null;
      this.toastT = 0;
      this.items = [];
      this.counts = {};

      // Wiedergabe-Zustand
      this.rep = null;
      this.inst = null;
      this.scenes = [];
      this.lens = [];
      this.starts = [];
      this.total = 0;
      this.si = 0;
      this.pi = 0;
      this.playing = true;
      this.speedIdx = 1;
      this.acc = 0;
      this.soundDone = false;
      this.barT = BAR_SHOW;
      this.error = null;
      [this.pre, this.post, this.postEnd] = PAD.minigolf;

      this.load();
      this.build();
      if (this.pending) this.start(this.pending);
    }

    // ----- Daten & Layout ------------------------------------------------
    load() {
      const all = replay.loadAll();
      this.counts = {};
      for (const g of replay.GAMES) this.counts[g] = all[g].length;
      this.items = all[this.tab] || [];
      this.sel = Math.min(this.sel, Math.max(0, this.items.length - 1));
      this.first = 0;
      this.confirm = null;
    }

    build() {
      this.left = 40;
      this.right = W - 40;
      this.bottom = H - 30;

      // Reiterleiste unter dem Untertitel; bricht bei Bedarf um.
      this.tabFont = ui.font(12, true);
      this.tabRects = [];
      let tx = this.left, ty = 90;
      const tabRight = W - 16;
      for (const key of replay.GAMES) {
        const label = `${gameName(key)} (${this.counts[key] || 0})`;
        const tw = this.tabFont.width(label) + 20;
        if (tx > this.left && tx + tw > tabRight) {
          tx = this.left;
          ty += 26;
        }
        this.tabRects.push([new PG.Rect(tx, ty, tw, 22), key, label]);
        tx += tw + 6;
      }

      // Kopfzeile darunter: Anzahl links, Teilen-Knöpfe rechts.
      const headY = ty + 22 + 8;
      const bw = 92;
      this.btnImport = new PG.Rect(this.right - bw, headY, bw, 22);
      this.btnExport = new PG.Rect(this.btnImport.x - 8 - bw, headY, bw, 22);
      this.headY = headY + 11;
      this.top = headY + 22 + 10;

      this.rowsVisible = Math.max(1, Math.floor((this.bottom - this.top) / ROW_H));
      this.rowRects = [];
      for (let i = 0; i < this.rowsVisible; i++) {
        this.rowRects.push(new PG.Rect(this.left, this.top + i * ROW_H, this.right - this.left, ROW_H - 6));
      }

      this.barRect = new PG.Rect(0, H - BAR_H, W, BAR_H);
      this.seekRect = new PG.Rect(0, H - BAR_H - 4, W, 10);
      const chipW = Math.min(220, W - 40);
      this.saveRect = new PG.Rect(W / 2 - chipW / 2, 10, chipW, 24);
    }

    // ----- Wiedergabe starten/beenden ------------------------------------
    start(rep) {
      this.rep = rep;
      this.error = null;
      this.scenes = rep.scenes || [];
      const entry = gameEntry(rep.game);
      this.inst = null;
      if (entry && this.scenes.length) {
        try {
          const modes = (entry.meta.modes || []).map((m) => m[0]);
          const want = (rep.meta && rep.meta.mode) || "single";
          const inst = new entry.cls(W, H, modes.includes(want) ? want : modes[0] || "single");
          inst.ctx = this.app.ctx;
          inst.replayBegin(rep);
          this.inst = inst;
        } catch (err) {
          console.error(err);
          this.inst = null;
        }
      }
      if (!this.inst) {
        this.error = t("replay.broken");
        this.mode = "list";
        return;
      }
      [this.pre, this.post, this.postEnd] = PAD[rep.game] || PAD.minigolf;
      this.lens = [];
      this.starts = [];
      let pos = 0;
      for (const sc of this.scenes) {
        const n = Math.max(1, replay.sceneLen(sc));
        const end = !!(sc.final || sc.knocked != null);
        this.starts.push(pos);
        const length = this.pre + n + (end ? this.postEnd : this.post);
        this.lens.push(length);
        pos += length;
      }
      this.total = pos;
      this.si = 0;
      this.pi = 0;
      this.acc = 0;
      this.soundDone = false;
      this.playing = true;
      this.barT = BAR_SHOW;
      this.mode = "play";
      this.seekNow();
    }

    stop() {
      this.inst = null;
      this.rep = null;
      this.mode = "list";
      this.load();
      this.build();
    }

    close() {
      PG.audio.play("click");
      this.onClose();
    }

    // ----- Zeitachse ------------------------------------------------------
    rate() {
      return (this.rep && this.rep.rate) || replay.RATE;
    }

    phase() {
      const sc = this.scenes[this.si];
      const n = Math.max(1, replay.sceneLen(sc));
      if (this.pi < this.pre) return [0, true, false];
      if (this.pi < this.pre + n) return [this.pi - this.pre, false, false];
      return [n - 1, false, !!sc.final];
    }

    seekNow() {
      if (!this.inst || !this.scenes.length) return;
      const [frame] = this.phase();
      this.inst.replaySeek(this.si, frame);
    }

    elapsed() {
      if (!this.starts.length) return 0;
      return this.starts[this.si] + this.pi;
    }

    atEnd() {
      return this.si >= this.scenes.length - 1 && this.pi >= this.lens[this.lens.length - 1] - 1;
    }

    advance(steps = 1) {
      for (let i = 0; i < steps; i++) {
        if (this.atEnd()) {
          this.playing = false;
          break;
        }
        this.pi += 1;
        if (this.pi >= this.lens[this.si]) {
          this.si += 1;
          this.pi = 0;
          this.soundDone = false;
        } else if (!this.soundDone) {
          const sc = this.scenes[this.si];
          const n = Math.max(1, replay.sceneLen(sc));
          if (this.pi >= this.pre + n) {
            this.soundDone = true;
            this.resultSound(sc);
          }
        }
      }
      this.seekNow();
    }

    resultSound(sc) {
      const key = sc.result || sc.res || "";
      const game = (this.rep || {}).game;
      if (sc.final && (game === "snake" || game === "tetris" || game === "pinball")) PG.audio.play("gameover");
      else if (sc.end === "cup" || key === "bowl.strike" || key === "bil.win_you") PG.audio.play("win");
      else if (key === "bowl.spare" || sc.final) PG.audio.play("point");
      else if (sc.end === "water" || key === "bil.foul" || key === "bil.win_ai") PG.audio.play("hit");
    }

    jumpScene(d) {
      if (!this.scenes.length) return;
      if (d < 0 && this.pi > this.pre + 4) this.pi = 0;
      else {
        this.si = PG.clamp(this.si + d, 0, this.scenes.length - 1);
        this.pi = 0;
      }
      this.soundDone = false;
      this.barT = BAR_SHOW;
      this.seekNow();
      PG.audio.play("move");
    }

    seekRatio(ratio) {
      if (!this.total) return;
      const target = PG.clamp(Math.floor(this.total * ratio), 0, this.total - 1);
      for (let i = this.lens.length - 1; i >= 0; i--) {
        if (target >= this.starts[i]) {
          this.si = i;
          this.pi = Math.min(this.lens[i] - 1, target - this.starts[i]);
          break;
        }
      }
      this.soundDone = true;
      this.barT = BAR_SHOW;
      this.seekNow();
    }

    setSpeed(d) {
      const idx = PG.clamp(this.speedIdx + d, 0, SPEEDS.length - 1);
      if (idx !== this.speedIdx) {
        this.speedIdx = idx;
        PG.audio.play("move");
      }
      this.barT = BAR_SHOW;
    }

    // ----- Speichern / Löschen -------------------------------------------
    savePending() {
      if (!this.pending) return;
      if (this.saved || replay.isSaved(this.pending.game, this.pending.id)) {
        this.saved = true;
        this.say(t("replay.already"));
        return;
      }
      const [ok, why] = replay.saveReplay(this.pending);
      if (ok) {
        this.saved = true;
        this.say(t("replay.saved"));
        PG.audio.play("level");
        ui.spawnBurst(W / 2, 24, ui.GREEN);
        const total = Object.values(replay.loadAll()).reduce((n, v) => n + v.length, 0);
        PG.stats.event("replay_first");
        PG.stats.event("replay_5", total);
      } else if (why === "full") {
        this.say(t("replay.full", { n: replay.MAX_PER_GAME }));
        PG.audio.play("hit");
      } else if (why === "space") {
        this.say(t("web.replay.space"));
        PG.audio.play("hit");
      } else {
        this.say(t("replay.error"));
        PG.audio.play("hit");
      }
    }

    // ----- Teilen: Export / Import ---------------------------------------
    current() {
      if (this.mode === "play") return this.rep;
      return this.items.length ? this.items[this.sel] : null;
    }

    exportReplay(rep) {
      rep = rep || this.current();
      if (!rep) return;
      const text = replay.exportText(rep);
      if (!text) {
        this.say(t("replay.export_error"));
        PG.audio.play("hit");
        return;
      }
      const name = replay.defaultFilename(rep);
      PG.downloadText(name, text);
      this.say(t("replay.exported", { file: name }));
      PG.audio.play("level");
      ui.spawnBurst(W / 2, 24, ui.ACCENT2);
      PG.stats.event("replay_share");
    }

    importReplay() {
      PG.pickTextFile(replay.EXT + ",.json,application/json", (text) => {
        const [ok, why, rep] = replay.importText(text, null);
        if (ok) {
          this.tab = rep.game;
          this.load();
          this.build();
          const i = this.items.findIndex((it) => it.id === rep.id);
          if (i >= 0) this.sel = i;
          this.say(t("replay.imported", { title: rep.title || "" }));
          PG.audio.play("level");
          ui.spawnBurst(W / 2, 24, ui.GREEN);
        } else {
          const known = ["dup", "full", "game"].includes(why);
          this.say(why === "space" ? t("web.replay.space") : t("replay.import_" + (known ? why : "error")));
          PG.audio.play("hit");
        }
      });
    }

    deleteSelected() {
      if (!this.items.length || this.mode !== "list") return;
      const rep = this.items[this.sel];
      if (this.confirm !== rep.id) {
        this.confirm = rep.id;
        this.say(t("replay.confirm"));
        PG.audio.play("move");
        return;
      }
      replay.deleteReplay(this.tab, rep.id);
      this.say(t("replay.deleted"));
      PG.audio.play("click");
      this.load();
      this.build();
    }

    say(text) {
      this.toast = text;
      this.toastT = 2.6;
    }

    // ----- Eingabe --------------------------------------------------------
    handleEvent(e) {
      if (this.mode === "play") this.eventPlay(e);
      else this.eventList(e);
    }

    eventPlay(e) {
      if (e.kind === "keydown") {
        this.barT = BAR_SHOW;
        const k = e.key;
        if (k === "Escape" || k === "BackSpace") {
          if (this.pending) this.close();
          else this.stop();
        } else if (k === "space" || k === "Return" || k === "KP_Enter") {
          if (this.atEnd()) {
            this.si = this.pi = 0;
            this.soundDone = false;
            this.seekNow();
          }
          this.playing = !this.playing;
          PG.audio.play("click");
        } else if ((k === "s" || k === "S") && this.pending && !this.saved) {
          // Solange eine frische Aufnahme bereitliegt, gehört S dem Speichern
          // (so steht es auch auf dem Chip); danach ist es wieder Tempo runter.
          this.savePending();
        } else if (k === "Left" || k === "a") this.jumpScene(-1);
        else if (k === "Right" || k === "d") this.jumpScene(1);
        else if (k === "Up" || k === "w" || k === "plus") this.setSpeed(1);
        else if (k === "Down" || k === "s" || k === "minus") this.setSpeed(-1);
        else if (k === "e" || k === "E") this.exportReplay(this.rep);
      } else if (e.kind === "mousemove") {
        this.barT = BAR_SHOW;
        this.app.canvas.style.cursor = "";
      } else if (e.kind === "mousedown" && e.button === 1) {
        this.barT = BAR_SHOW;
        if (this.pending && !this.saved && this.saveRect.collidepoint(e.pos)) this.savePending();
        else if (this.seekRect.collidepoint(e.pos)) this.seekRatio(e.pos[0] / W);
        else if (!this.barRect.collidepoint(e.pos)) this.playing = !this.playing;
      } else if (e.kind === "wheel") {
        this.setSpeed(e.delta > 0 ? 1 : -1);
      }
    }

    eventList(e) {
      if (e.kind === "keydown") {
        const k = e.key;
        if (k === "Escape") this.close();
        else if (k === "Tab") this.switchTab(replay.GAMES[PG.mod(replay.GAMES.indexOf(this.tab) + 1, replay.GAMES.length)]);
        else if (k === "Up" || k === "w") this.move(-1);
        else if (k === "Down" || k === "s") this.move(1);
        else if (k === "Left" || k === "a") this.switchTab(replay.GAMES[PG.mod(replay.GAMES.indexOf(this.tab) - 1, replay.GAMES.length)]);
        else if (k === "Right" || k === "d") this.switchTab(replay.GAMES[PG.mod(replay.GAMES.indexOf(this.tab) + 1, replay.GAMES.length)]);
        else if (k === "Return" || k === "space" || k === "KP_Enter") {
          if (this.items.length) {
            PG.audio.play("click");
            this.start(this.items[this.sel]);
          }
        } else if (k === "Delete" || k === "x" || k === "X") this.deleteSelected();
        else if (k === "e" || k === "E") this.exportReplay();
        else if (k === "i" || k === "I") this.importReplay();
      } else if (e.kind === "mousemove") {
        this.btnHover = this.btnExport.collidepoint(e.pos) ? "export" : this.btnImport.collidepoint(e.pos) ? "import" : null;
        this.tabHover = null;
        for (const [r, key] of this.tabRects) if (r.collidepoint(e.pos)) this.tabHover = key;
        this.hoverRow = null;
        this.rowRects.forEach((r, i) => {
          if (r.collidepoint(e.pos) && this.first + i < this.items.length) {
            this.hoverRow = i;
            this.sel = this.first + i;
          }
        });
        this.app.canvas.style.cursor = this.btnHover || this.tabHover || this.hoverRow != null ? "pointer" : "";
      } else if (e.kind === "mousedown" && e.button === 1) {
        if (this.btnExport.collidepoint(e.pos)) return this.exportReplay();
        if (this.btnImport.collidepoint(e.pos)) return this.importReplay();
        for (const [r, key] of this.tabRects) {
          if (r.collidepoint(e.pos)) return this.switchTab(key);
        }
        this.rowRects.forEach((r, i) => {
          if (r.collidepoint(e.pos) && this.first + i < this.items.length) {
            this.sel = this.first + i;
            PG.audio.play("click");
            this.start(this.items[this.sel]);
          }
        });
      } else if (e.kind === "wheel") {
        this.move(e.delta > 0 ? -1 : 1);
      }
    }

    move(d) {
      if (!this.items.length) return;
      this.sel = PG.mod(this.sel + d, this.items.length);
      this.confirm = null;
      if (this.sel < this.first) this.first = this.sel;
      else if (this.sel >= this.first + this.rowsVisible) this.first = this.sel - this.rowsVisible + 1;
      PG.audio.play("move");
    }

    switchTab(key) {
      if (key === this.tab) return;
      this.tab = key;
      this.sel = 0;
      this.load();
      this.build();
      PG.audio.play("move");
    }

    // ----- Ablauf ---------------------------------------------------------
    update(dt) {
      if (this.toastT > 0) {
        this.toastT -= dt;
        if (this.toastT <= 0) this.toast = null;
      }
      if (this.mode !== "play" || !this.inst) return;
      if (this.barT > 0) this.barT -= dt;
      if (!this.playing) return;
      this.acc += dt * SPEEDS[this.speedIdx];
      const step = 1 / this.rate();
      let steps = 0;
      while (this.acc >= step && steps < 240) {
        this.acc -= step;
        steps += 1;
      }
      if (steps) this.advance(steps);
    }

    // ----- Zeichnen -------------------------------------------------------
    draw(ctx) {
      if (this.mode === "play" && this.inst) this.drawPlay(ctx);
      else this.drawList(ctx);
      this.drawToast(ctx);
    }

    drawPlay(ctx) {
      const [, aiming, banner] = this.phase();
      ctx.save();
      try {
        this.inst.replayDraw(ctx, aiming, banner);
      } finally {
        ctx.restore();
        this.app.resetTransform(ctx);
        ctx.globalAlpha = 1;
        ctx.globalCompositeOperation = "source-over";
        ctx.filter = "none";
        ctx.shadowBlur = 0;
      }
      this.drawChip(ctx);
      if (this.pending && (!this.saved || this.toastT > 0)) this.drawSaveChip(ctx);
      if (this.barT > 0 || !this.playing) this.drawBar(ctx);
    }

    chipY() {
      return (this.inst.hudH || 36) + (this.inst.cardH || 0) + 6;
    }

    drawChip(ctx) {
      const fnt = ui.font(10, true);
      const img = fnt.width(t("replay.chip"));
      const w = img + 30, h = fnt.height + 8;
      const r = new PG.Rect(8, this.chipY(), w, h);
      ui.drawPanel(ctx, r, { radius: h / 2, shadow: false });
      draw.circle(ctx, ui.mix(ui.PANEL, ui.RED, 0.4 + 0.6 * ui.pulse(2.4)), [r.x + 11, r.centery], 4);
      ui.text(ctx, t("replay.chip"), r.x + 20, r.centery, fnt, ui.TEXT, "midleft");
    }

    drawSaveChip(ctx) {
      const r = this.saveRect;
      r.y = this.chipY() - 1;
      const done = this.saved;
      const label = done ? t("replay.saved_chip") : t("replay.save_chip");
      ui.drawPanel(ctx, r, { radius: 12, shadow: false, color: ui.mix(ui.PANEL, ui.GREEN, done ? 0.35 : 0) });
      draw.rect(ctx, done ? ui.GREEN : ui.ACCENT, r, 1, 12);
      ui.text(ctx, label, r.centerx, r.centery, ui.font(11, true), done ? ui.GREEN : ui.TEXT, "center");
    }

    drawBar(ctx) {
      const bar = this.barRect;
      draw.rect(ctx, [10, 12, 18, 208], bar);
      const accent = gameAccent(this.rep.game);
      draw.line(ctx, ui.BORDER, [0, bar.y], [W, bar.y]);

      const y = bar.y - 3;
      draw.rect(ctx, ui.PANEL, [0, y, W, 5]);
      const done = this.elapsed() / Math.max(1, this.total);
      draw.rect(ctx, accent, [0, y, Math.round(W * done), 5]);
      for (const start of this.starts.slice(1)) {
        const x = Math.round((W * start) / Math.max(1, this.total));
        draw.line(ctx, ui.BORDER_LIGHT, [x, y], [x, y + 5]);
      }

      const small = ui.font(11, true);
      const tiny = ui.font(9);
      ui.text(ctx, this.rep.title || gameName(this.tab), 10, bar.y + 5, small, ui.TEXT);
      let sub = this.rep.sub || "";
      if (this.rep.meta && this.rep.meta.capped) sub = (sub ? sub + "  ·  " : "") + t("replay.capped");
      ui.text(ctx, sub, 10, bar.y + 22, tiny, ui.TEXT_DIM);

      const rate = this.rate();
      const pos = `${replay.formatDuration(this.elapsed() / rate)} / ${replay.formatDuration(this.total / rate)}`;
      ui.text(ctx, pos, W / 2, bar.y + 5, small, ui.TEXT_DIM, "midtop");
      const label = t(SEQ_KEYS[this.rep.game] || "replay.seq_part", { n: this.si + 1, total: this.scenes.length });
      ui.text(ctx, label, W / 2, bar.y + 22, tiny, accent, "midtop");

      const state = this.playing ? t("replay.playing") : t("replay.paused");
      ui.text(ctx, `${state}  ${SPEEDS[this.speedIdx].toFixed(1)}x`, W - 10, bar.y + 5, small,
              this.playing ? ui.TEXT_DIM : ui.GOLD, "topright");
      const hint = t("replay.hint_play");
      if (tiny.width(hint) < W - 20) ui.text(ctx, hint, W - 10, bar.y + 22, tiny, ui.TEXT_FAINT, "topright");
    }

    // --- Archiv-Liste ---
    drawList(ctx) {
      ui.drawBackground(ctx, W, H, false);
      ui.drawTitle(ctx, W, t("replay.name"), {
        subtitle: t("replay.subtitle"), y: 26, big: ui.font(26, true), accent: ui.ACCENT2,
      });

      for (const [r, key, label] of this.tabRects) {
        const active = key === this.tab;
        const hover = key === this.tabHover;
        draw.rect(ctx, active || hover ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 11);
        draw.rect(ctx, active ? gameAccent(key) : ui.BORDER, r, 1, 11);
        ui.text(ctx, label, r.centerx, r.centery, this.tabFont, active ? ui.TEXT : ui.TEXT_DIM, "center");
      }

      ui.text(ctx, t("replay.count", { n: this.items.length, max: replay.MAX_PER_GAME }),
              this.left, this.headY, ui.font(11, true), ui.TEXT_DIM, "midleft");
      this.drawBtn(ctx, this.btnExport, t("replay.btn_export"), this.items.length > 0, this.btnHover === "export");
      this.drawBtn(ctx, this.btnImport, t("replay.btn_import"), true, this.btnHover === "import");

      if (!this.items.length) this.drawEmpty(ctx);
      else {
        this.rowRects.forEach((r, i) => {
          const idx = this.first + i;
          if (idx < this.items.length) this.drawRow(ctx, r, this.items[idx], idx === this.sel);
        });
        if (this.items.length > this.rowsVisible) this.drawScrollbar(ctx);
      }
      ui.drawFooter(ctx, W, H, t("replay.hint_list"));
    }

    drawBtn(ctx, rect, label, on, hover) {
      draw.rect(ctx, on && hover ? ui.PANEL_LIGHT : ui.PANEL, rect, 0, 11);
      draw.rect(ctx, on && hover ? ui.ACCENT2 : ui.BORDER, rect, 1, 11);
      ui.text(ctx, label, rect.centerx, rect.centery, ui.font(10, true), on ? ui.TEXT : ui.TEXT_FAINT, "center");
    }

    drawRow(ctx, rect, rep, selected) {
      const accent = gameAccent(rep.game);
      ui.drawPanel(ctx, rect, { color: selected ? ui.PANEL_LIGHT : ui.PANEL, radius: 9, shadow: false });
      if (selected) draw.rect(ctx, accent, rect, 1, 9);
      draw.rect(ctx, accent, [rect.x, rect.y + 7, 3, rect.h - 14], 0, 2);

      const cx = rect.x + 26, cy = rect.centery;
      draw.circle(ctx, ui.mix(ui.BTN, accent, 0.35), [cx, cy], 11);
      draw.polygon(ctx, selected ? ui.TEXT : ui.TEXT_DIM, [[cx - 3, cy - 5], [cx + 6, cy], [cx - 3, cy + 5]]);

      const titleFont = ui.font(14, true);
      const title = rep.title || "";
      ui.text(ctx, title, rect.x + 44, rect.y + 7, titleFont, selected ? ui.TEXT : ui.TEXT_DIM);
      if (rep.src === "file") {
        // Kleiner Pfeil nach unten = aus einer Datei eingelesen.
        const ax = rect.x + 50 + titleFont.width(title);
        const ay = rect.y + 12;
        draw.line(ctx, ui.ACCENT2, [ax, ay], [ax, ay + 7], 2);
        draw.polygon(ctx, ui.ACCENT2, [[ax - 4, ay + 5], [ax + 4, ay + 5], [ax, ay + 10]]);
      }
      ui.text(ctx, rep.sub || "", rect.x + 44, rect.y + 24, ui.font(10), ui.TEXT_FAINT);

      ui.text(ctx, replay.formatDuration(replay.duration(rep)), rect.right - 12, rect.y + 7, ui.font(11, true), accent, "topright");
      ui.text(ctx, rep.date || "", rect.right - 12, rect.y + 25, ui.font(9), ui.TEXT_FAINT, "topright");

      if (this.confirm === rep.id) {
        ui.text(ctx, t("replay.confirm_short"), rect.right - 62, rect.centery, ui.font(9, true), ui.RED, "midright");
      }
    }

    drawEmpty(ctx) {
      const cx = W / 2;
      const cy = (this.top + this.bottom) / 2;
      ui.text(ctx, t("replay.empty", { game: gameName(this.tab) }), cx, cy - 16, ui.font(15, true), ui.TEXT_DIM, "center");
      ui.text(ctx, t("replay.empty_hint"), cx, cy + 8, ui.font(11), ui.TEXT_FAINT, "center");
      if (!replay.isEnabled()) {
        ui.text(ctx, t("replay.disabled"), cx, cy + 32, ui.font(11, true), ui.GOLD, "center");
      }
      if (this.error) ui.text(ctx, this.error, cx, cy + 54, ui.font(11, true), ui.RED, "center");
    }

    drawScrollbar(ctx) {
      const top = this.top;
      const trackH = this.rowsVisible * ROW_H;
      draw.rect(ctx, ui.PANEL, [this.right + 6, top, 4, trackH], 0, 2);
      const frac = this.rowsVisible / this.items.length;
      const h = Math.max(20, Math.floor(trackH * frac));
      const pos = this.first / Math.max(1, this.items.length - this.rowsVisible);
      const y = top + Math.floor((trackH - h) * Math.min(1, pos));
      draw.rect(ctx, ui.BORDER_LIGHT, [this.right + 6, y, 4, h], 0, 2);
    }

    drawToast(ctx) {
      if (!this.toast) return;
      const fnt = ui.font(12, true);
      const w = fnt.width(this.toast) + 26, h = fnt.height + 12;
      const y = H - h - (this.mode === "play" ? BAR_H + 12 : 40);
      const r = new PG.Rect(W / 2 - w / 2, y, w, h);
      ui.drawPanel(ctx, r, { radius: h / 2, shadow: false });
      draw.rect(ctx, ui.ACCENT2, r, 1, h / 2);
      ui.text(ctx, this.toast, r.centerx, r.centery, fnt, ui.TEXT, "center");
    }
  }

  PG.ReplayScreen = ReplayScreen;
  PG.replayGameName = gameName;
})();
