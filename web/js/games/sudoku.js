/*
 * sudoku.js - Sudoku mit 400 deterministischen Leveln (Port von games/sudoku.py)
 * ===============================================================================
 * Level-System
 * ------------
 * Die Puzzles kommen aus sudoku_gen.js: Level N von Stufe D ist immer
 * dasselbe Puzzle (Seed-generiert, eindeutig lösbar). Gelöste Level werden je
 * Stufe im Speicher (Abschnitt "sudoku") gesichert und in der Levelauswahl
 * abgehakt.
 *
 * Spielmodi (Auswahl im Vorspiel-Screen, aufsteigende Hilfe-Stufen)
 * - classic  (x2,0 Punkte): keine Hilfen - pur wie auf Papier.
 * - notes    (x1,5 Punkte): + Bleistift-Notizen (N bzw. Notiz-Button).
 * - comfort  (x1,0 Punkte): + falsche Ziffern sofort rot, Konflikt- und
 *              Gleiche-Ziffer-Hervorhebung, korrekte Eingaben rasten ein.
 * - assist   (x0,7 Punkte): + Tipp-Funktion (H, max. 3, kostet Punkte).
 *
 * Fehler-Regel (in ALLEN Modi gleich): jede Eingabe wird sofort gegen die
 * eindeutige Lösung geprüft; ist das 3-Fehler-Limit aktiv, ist beim dritten
 * Fehler die Partie verloren.
 *
 * Punkte: (Basis je Stufe - Zeit - Fehler - Tipps) x Modus-Multiplikator.
 *
 * Steuerung: Pfeile/WASD = Zelle wählen, 1-9 = Ziffer, 0/Backspace/Entf =
 * radieren (auch Rechtsklick), N = Notizen, H = Tipp, R = Level neu,
 * Q = Levelwahl. Maus: Zellen und Ziffernfeld anklicken.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const gen = PG.sudokuGen;
  const Rect = PG.Rect;
  const fl = Math.floor;

  // ----- Farben ---------------------------------------------------------------
  // Eigene Ziffern bleiben bewusst blau (klassische Sudoku-Optik).
  const COL_USER = [110, 165, 255];

  // (i18n-Suffix, Basispunkte) je Schwierigkeitsgrad
  const DIFFICULTIES = [["easy", 1000], ["normal", 2000], ["hard", 3500], ["expert", 5000]];

  const MODE_MULT = { classic: 2.0, notes: 1.5, comfort: 1.0, assist: 0.7 };

  const MAX_HINTS = 3;
  const HINT_COST = 200; // Punktabzug je Tipp
  const ERR_COST = 150; // Punktabzug je Fehler
  const TIME_COST = 2; // Punktabzug je Sekunde
  const FAIL_LIMIT = 3;

  const SETUP = "setup", GENERATING = "generating", PLAY = "play";

  const MOVE = { Up: -9, w: -9, W: -9, Down: 9, s: 9, S: 9, Left: -1, a: -1, A: -1, Right: 1, d: 1, D: 1 };

  class SudokuGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.won = false;

      const sud = this.opts;
      this.diff = Math.max(0, Math.min(3, Math.trunc(Number(sud.difficulty) || 0)));
      this.failLimit = sud.fail_limit !== false;
      this.lastLevel = Object.assign({}, sud.last_level && typeof sud.last_level === "object" ? sud.last_level : {});

      // Modus-Fähigkeiten aus dem gewählten mode ableiten.
      this.canNotes = ["notes", "comfort", "assist"].includes(this.mode);
      this.canCheck = ["comfort", "assist"].includes(this.mode); // rot + Konflikte
      this.canHint = this.mode === "assist";

      this.buildFonts();

      this.loadProgress();
      this.cursor = this.lastLevel[String(this.diff)] || 1;
      this.level = 1;
      this.msg = null;
      this.msgT = 0;
      this.genDrawn = false;

      this.buildSetupLayout();
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, fl(h / 30)));
      this.tiny = ui.font(Math.max(11, fl(h / 36)));
      this.huge = ui.font(Math.max(26, fl(h / 11)), true);
      // Uhr mit fester Zeichenbreite, damit sie beim Ticken nicht "zittert".
      this.clockFont = ui.font(Math.max(18, fl(h / 22)), false, true);
    }

    // ----- Persistenz ------------------------------------------------------
    /** Liest die gelösten Level je Stufe (Abschnitt 'sudoku'). */
    loadProgress() {
      const data = PG.store.get("sudoku", {}) || {};
      const solved = data.solved && typeof data.solved === "object" ? data.solved : {};
      this.solved = {};
      for (const k of ["0", "1", "2", "3"]) {
        const lst = solved[k];
        if (Array.isArray(lst)) {
          const set = new Set(lst.filter((v) => Number.isInteger(v) && v >= 1 && v <= gen.LEVELS));
          this.solved[k] = Array.from(set).sort((a, b) => a - b);
        } else {
          this.solved[k] = [];
        }
      }
    }

    saveProgress() {
      const data = PG.store.get("sudoku", {}) || {};
      data.solved = this.solved;
      PG.store.set("sudoku", data);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const W = this.width, H = this.height;
      const cx = fl(W / 2);

      // 4 Stufen-Buttons nebeneinander
      const bw = Math.min(150, fl((W - 60) / 4) - 8);
      const total = 4 * bw + 3 * 10;
      this.diffRects = [];
      for (let i = 0; i < 4; i++) this.diffRects.push(new Rect(cx - fl(total / 2) + i * (bw + 10), fl(H * 0.16), bw, 34));

      // Fehler-Limit-Toggle darunter
      this.limitRect = new Rect(cx - 150, fl(H * 0.16) + 44, 300, 26);

      // 10x10-Levelraster: füllt den Platz zwischen Toggle und Fußzeile
      const top = this.limitRect.bottom + 30;
      const availH = H - top - 60;
      const availW = W - 80;
      const cell = Math.max(18, Math.min(fl(availW / 10), fl(availH / 10)));
      this.lvCell = cell;
      this.lvX = cx - cell * 5;
      this.lvY = top;
      this.lvFont = ui.font(Math.max(10, fl((cell * 2) / 5)), false, true);
    }

    /** Pixel -> Levelnummer 1..100 (oder null). */
    levelAt(pos) {
      const c = fl((pos[0] - this.lvX) / this.lvCell);
      const r = fl((pos[1] - this.lvY) / this.lvCell);
      if (c >= 0 && c < 10 && r >= 0 && r < 10) return r * 10 + c + 1;
      return null;
    }

    selectDifficulty(i) {
      this.diff = Math.max(0, Math.min(3, i));
      this.saveSetting("difficulty", this.diff);
      this.cursor = this.lastLevel[String(this.diff)] || 1;
      this.playSound("click");
    }

    toggleFailLimit() {
      this.failLimit = !this.failLimit;
      this.saveSetting("fail_limit", this.failLimit);
      this.playSound("select");
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4"].includes(k)) this.selectDifficulty(Number(k) - 1);
        else if (k === "f" || k === "F") this.toggleFailLimit();
        else if (["Left", "a", "A"].includes(k)) {
          this.cursor = PG.mod(this.cursor - 2, 100) + 1;
          this.playSound("move");
        } else if (["Right", "d", "D"].includes(k)) {
          this.cursor = PG.mod(this.cursor, 100) + 1;
          this.playSound("move");
        } else if (["Up", "w", "W"].includes(k)) {
          this.cursor = PG.mod(this.cursor - 11, 100) + 1;
          this.playSound("move");
        } else if (["Down", "s", "S"].includes(k)) {
          this.cursor = PG.mod(this.cursor + 9, 100) + 1;
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          this.startLevel(this.cursor);
        }
      } else if (ev.kind === "mousemove") {
        const lv = this.levelAt(ev.pos);
        if (lv !== null) this.cursor = lv;
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.selectDifficulty(i);
            return;
          }
        }
        if (this.limitRect.collidepoint(ev.pos)) {
          this.toggleFailLimit();
          return;
        }
        const lv = this.levelAt(ev.pos);
        if (lv !== null) this.startLevel(lv);
      }
    }

    // ===================================================== Level starten
    startLevel(n) {
      this.level = Math.max(1, Math.min(gen.LEVELS, Math.trunc(n)));
      this.cursor = this.level;
      this.lastLevel[String(this.diff)] = this.level;
      this.saveSetting("last_level", Object.assign({}, this.lastLevel));
      this.gameOver = false;
      this.won = false;
      this.score = 0;
      this.genDrawn = false;
      this.state = GENERATING;
      this.playSound("click");
    }

    /** Erzeugt das Puzzle (blockierend, < 1s) und initialisiert das Brett. */
    doGenerate() {
      const [puzzle, solution] = gen.generate(this.diff, this.level);
      this.solution = solution;
      this.board = puzzle.slice();
      this.given = puzzle.map((v) => v !== 0);
      this.locked = this.given.slice();
      this.notes = Array.from({ length: 81 }, () => new Set());
      this.wrong = new Set();
      const firstFree = this.given.indexOf(false);
      this.sel = firstFree >= 0 ? firstFree : 0;
      this.errors = 0;
      this.hintsUsed = 0;
      this.elapsed = 0;
      this.noteMode = false;
      this.reveal = false; // nach Spielende: Lösung statt Banner zeigen
      this.msg = null;
      this.msgT = 0;
      this.conflicts = new Set();
      this.buildPlayLayout();
      this.state = PLAY;
    }

    // ===================================================== Spiel-Layout
    buildPlayLayout() {
      const W = this.width, H = this.height;
      this.hudH = Math.max(40, fl(H / 12));

      // Brett links/mittig, Ziffernfeld rechts daneben. Unten bleiben 28px
      // für die Steuerungs-Hinweiszeile frei.
      const padW = Math.max(120, fl(W / 5));
      const size = Math.min(H - this.hudH - 40, W - padW - 40);
      this.cell = Math.max(20, fl(size / 9));
      const bs = this.cell * 9;
      this.bx = Math.max(12, fl((W - padW - bs) / 2));
      this.by = this.hudH + fl((H - this.hudH - 28 - bs) / 2);

      // Mono-Fonts: Ziffern stehen so in jeder Zelle exakt gleich breit.
      this.numFont = ui.font(Math.max(14, fl((this.cell * 3) / 5)), true, true);
      this.noteFont = ui.font(Math.max(8, fl((this.cell * 2) / 7)), false, true);

      // Ziffernfeld: 3x3-Raster + Funktionsleiste darunter.
      const px = this.bx + bs + 24;
      const pb = Math.min(fl((W - px - 16) / 3), this.cell + 8);
      let py = this.by + fl((bs - pb * 3 - 3 * (fl((pb * 2) / 3) + 6)) / 3);
      py = Math.max(this.by, py);
      this.padRects = {};
      for (let d = 1; d <= 9; d++) {
        const r = fl((d - 1) / 3), c = (d - 1) % 3;
        this.padRects[String(d)] = new Rect(px + c * (pb + 4), py + r * (pb + 4), pb, pb);
      }
      const fy = py + 3 * (pb + 4) + 8;
      const fw = 3 * pb + 8;
      const fh = Math.max(24, fl((pb * 2) / 3));
      this.padRects.erase = new Rect(px, fy, fw, fh);
      let row = 1;
      if (this.canNotes) {
        this.padRects.note = new Rect(px, fy + row * (fh + 6), fw, fh);
        row++;
      }
      if (this.canHint) this.padRects.hint = new Rect(px, fy + row * (fh + 6), fw, fh);
    }

    cellAt(pos) {
      const c = fl((pos[0] - this.bx) / this.cell);
      const r = fl((pos[1] - this.by) / this.cell);
      if (c >= 0 && c < 9 && r >= 0 && r < 9) return r * 9 + c;
      return null;
    }

    padAt(pos) {
      for (const key in this.padRects) if (this.padRects[key].collidepoint(pos)) return key;
      return null;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state !== PLAY) return;
      if (this.gameOver) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            if (this.won) this.nextLevel();
            else this.startLevel(this.level);
          } else if (["s", "S", "q", "Q"].includes(ev.key)) {
            this.state = SETUP;
            this.gameOver = false;
            this.playSound("click");
          } else if (ev.key === "a" || ev.key === "A") {
            // Banner ausblenden und die Lösung auf dem Brett zeigen
            // (nochmal A = zurück zum Banner).
            this.reveal = !this.reveal;
            this.playSound("select");
          }
        }
        return;
      }

      if (ev.kind === "keydown") {
        this.handlePlayKey(ev.key);
      } else if (ev.kind === "mousedown") {
        if (ev.button === 3) {
          const cell = this.cellAt(ev.pos);
          if (cell !== null) {
            this.sel = cell;
            this.erase(cell);
          }
          return;
        }
        const cell = this.cellAt(ev.pos);
        if (cell !== null) {
          this.sel = cell;
          this.playSound("move");
          return;
        }
        const pad = this.padAt(ev.pos);
        if (pad === "erase") this.erase(this.sel);
        else if (pad === "note") this.toggleNoteMode();
        else if (pad === "hint") this.useHint();
        else if (pad !== null) this.enterDigit(this.sel, Number(pad));
      }
    }

    handlePlayKey(k) {
      if (k in MOVE) {
        this.sel = PG.mod(this.sel + MOVE[k], 81);
        this.playSound("move");
      } else if (/^[1-9]$/.test(k)) {
        this.enterDigit(this.sel, Number(k));
      } else if (/^KP_[1-9]$/.test(k)) {
        this.enterDigit(this.sel, Number(k.slice(3)));
      } else if (["0", "KP_0", "BackSpace", "Delete"].includes(k)) {
        this.erase(this.sel);
      } else if (k === "n" || k === "N") {
        this.toggleNoteMode();
      } else if (k === "h" || k === "H") {
        this.useHint();
      } else if (k === "r" || k === "R") {
        this.startLevel(this.level);
      } else if (k === "q" || k === "Q") {
        this.state = SETUP;
        this.playSound("click");
      }
    }

    // ===================================================== Spiellogik
    toggleNoteMode() {
      if (!this.canNotes) return;
      this.noteMode = !this.noteMode;
      this.playSound("select");
    }

    enterDigit(idx, d) {
      if (this.locked[idx]) return;
      if (this.noteMode) {
        this.toggleNote(idx, d);
        return;
      }
      if (this.board[idx] === d) return;
      this.board[idx] = d;
      this.notes[idx].clear();
      if (d === this.solution[idx]) {
        this.wrong.delete(idx);
        if (this.canCheck) this.locked[idx] = true; // korrekt -> rastet ein
        if (this.canNotes) this.pruneNotes(idx, d);
        this.playSound("select");
        this.updateConflicts();
        this.checkWin();
      } else {
        this.wrong.add(idx);
        this.errors++;
        this.playSound("hit");
        this.rumble(120);
        this.updateConflicts();
        if (this.failLimit && this.errors >= FAIL_LIMIT) {
          this.lose();
        } else {
          // Auch ein durch eine falsche Ziffer voll gewordenes Brett
          // prüfen -> zeigt in classic/notes die "noch Fehler"-Meldung.
          this.checkWin();
        }
      }
    }

    toggleNote(idx, d) {
      if (this.board[idx]) return;
      if (this.notes[idx].has(d)) this.notes[idx].delete(d);
      else this.notes[idx].add(d);
      this.playSound("move");
    }

    erase(idx) {
      if (this.locked[idx]) return;
      if (this.board[idx] || this.notes[idx].size) {
        this.board[idx] = 0;
        this.notes[idx].clear();
        this.wrong.delete(idx);
        this.updateConflicts();
        this.playSound("move");
      }
    }

    /** Entfernt die Ziffer d aus den Notizen aller Peer-Zellen. */
    pruneNotes(idx, d) {
      for (const j of gen.PEERS[idx]) this.notes[j].delete(d);
    }

    useHint() {
      if (!this.canHint || this.hintsUsed >= MAX_HINTS || this.gameOver) return;
      let idx = this.sel;
      if (this.locked[idx] || (this.board[idx] && !this.wrong.has(idx))) {
        const empties = [];
        for (let i = 0; i < 81; i++) if (!this.locked[i] && this.board[i] !== this.solution[i]) empties.push(i);
        if (!empties.length) return;
        idx = PG.rand.choice(empties);
      }
      this.board[idx] = this.solution[idx];
      this.notes[idx].clear();
      this.wrong.delete(idx);
      this.locked[idx] = true;
      this.hintsUsed++;
      this.pruneNotes(idx, this.solution[idx]);
      this.sel = idx;
      this.playSound("powerup");
      this.updateConflicts();
      this.checkWin();
    }

    /** Zellen, deren Ziffer mit einem Peer kollidiert (nur comfort+). */
    updateConflicts() {
      if (!this.canCheck) return;
      const bad = new Set();
      for (let i = 0; i < 81; i++) {
        const v = this.board[i];
        if (v && gen.PEERS[i].some((j) => this.board[j] === v)) bad.add(i);
      }
      this.conflicts = bad;
    }

    checkWin() {
      if (this.board.includes(0)) return;
      if (this.board.every((v, i) => v === this.solution[i])) {
        this.win();
      } else if (!this.canCheck) {
        // Voll, aber falsch: kurzer Hinweis (Fehler wurden schon gezählt).
        this.msg = t("sud.full_wrong");
        this.msgT = 2.5;
      }
    }

    win() {
      this.won = true;
      this.gameOver = true;
      const base = DIFFICULTIES[this.diff][1];
      const raw = base - TIME_COST * Math.trunc(this.elapsed) - ERR_COST * this.errors - HINT_COST * this.hintsUsed;
      this.score = Math.trunc(Math.max(50, raw) * (MODE_MULT[this.mode] || 1.0));
      const key = String(this.diff);
      if (!this.solved[key].includes(this.level)) {
        this.solved[key] = this.solved[key].concat([this.level]).sort((a, b) => a - b);
        this.saveProgress();
      }
      this.reportResult(true);
      if (this.errors === 0) this.achEvent("sudoku_clean");
      this.playSound("win");
      this.rumble(200);
    }

    lose() {
      this.won = false;
      this.gameOver = true;
      this.score = 0;
      this.reportResult(false);
      this.playSound("gameover");
      this.rumble(250);
    }

    nextLevel() {
      if (this.level >= gen.LEVELS) {
        this.state = SETUP;
        this.gameOver = false;
        return;
      }
      this.startLevel(this.level + 1);
    }

    update(dt) {
      if (this.state === GENERATING) {
        // Erst einen Frame "Erzeuge Puzzle..." anzeigen lassen (draw setzt
        // genDrawn), dann blockierend generieren.
        if (this.genDrawn) this.doGenerate();
        return;
      }
      if (this.state !== PLAY || this.gameOver) return;
      this.elapsed += dt;
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
      } else if (this.state === GENERATING) {
        this.drawGenerating(ctx);
      } else {
        this.drawHud(ctx);
        this.drawBoard(ctx);
        this.drawPad(ctx);
        if (this.gameOver && !this.reveal) this.drawResult(ctx);
      }
    }

    // ----- Setup ----------------------------------------------------------
    drawSetup(ctx) {
      const cx = fl(this.width / 2);
      ui.text(ctx, "SUDOKU", cx, fl(this.height * 0.07), this.huge, this.accent, "center");
      const modeLbl = this.mode in MODE_MULT ? t("sud.mode." + this.mode) : this.mode;
      const subTxt = modeLbl + "   -   " + t("sud.subtitle");
      const subFont = this.small.width(subTxt) > this.width - 24 ? this.tiny : this.small;
      ui.text(ctx, subTxt, cx, fl(this.height * 0.115), subFont, ui.TEXT_DIM, "center");

      // Stufen-Buttons
      this.diffRects.forEach((r, i) => {
        const on = i === this.diff;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 8);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 8);
        ui.text(ctx, t("sud.diff." + DIFFICULTIES[i][0]), r.centerx, r.centery, this.small, on ? ui.TEXT : ui.TEXT_DIM, "center");
      });

      // Fehler-Limit-Toggle
      const r = this.limitRect;
      draw.rect(ctx, ui.BTN, r, 0, 6);
      draw.rect(ctx, ui.BORDER, r, 1, 6);
      const state = this.failLimit ? t("common.on") : t("common.off");
      ui.text(ctx, t("sud.fail_limit") + ":  " + state, r.centerx, r.centery, this.small, this.failLimit ? ui.GREEN : ui.TEXT_DIM, "center");

      // 10x10-Levelraster
      const solved = new Set(this.solved[String(this.diff)] || []);
      const solvedBg = ui.mix(ui.PANEL, ui.GREEN, 0.22);
      for (let n = 1; n <= 100; n++) {
        const i = n - 1;
        const x = this.lvX + (i % 10) * this.lvCell;
        const y = this.lvY + fl(i / 10) * this.lvCell;
        const cell = new Rect(x + 1, y + 1, this.lvCell - 2, this.lvCell - 2);
        const isSolved = solved.has(n);
        draw.rect(ctx, isSolved ? solvedBg : ui.BTN, cell, 0, 4);
        if (n === this.cursor) draw.rect(ctx, this.accent, cell, 2, 4);
        ui.text(ctx, String(n), cell.centerx, cell.centery, this.lvFont, isSolved ? ui.GREEN : ui.TEXT_DIM, "center");
        if (isSolved) {
          // kleiner Haken unten rechts
          const bx = cell.right - 7, by = cell.bottom - 6;
          draw.lines(ctx, ui.GREEN, false, [[bx - 4, by - 2], [bx - 2, by], [bx + 2, by - 5]], 2);
        }
      }

      ui.text(ctx, t("sud.progress", { n: solved.size }), cx, this.lvY + 10 * this.lvCell + 18, this.small, ui.TEXT_DIM, "center");
      ui.text(ctx, t("sud.setup_hint"), cx, this.height - 16, this.tiny, ui.TEXT_DIM, "center");
    }

    drawGenerating(ctx) {
      ui.text(ctx, t("sud.generating"), fl(this.width / 2), fl(this.height / 2), this.font, ui.TEXT, "center");
      this.genDrawn = true;
    }

    // ----- HUD ------------------------------------------------------------
    fmtTime() {
      const sec = Math.trunc(this.elapsed);
      return String(fl(sec / 60)).padStart(2, "0") + ":" + String(sec % 60).padStart(2, "0");
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH + 0.5], [this.width, this.hudH + 0.5]);
      const cy = fl(this.hudH / 2);

      const left = t("sud.level", { n: this.level }) + "  ·  " + t("sud.diff." + DIFFICULTIES[this.diff][0]);
      ui.text(ctx, left, 12, cy, this.small, ui.TEXT, "midleft");

      // In der Lösungs-Ansicht ersetzt der Zurück-Hinweis die (eingefrorene)
      // Uhr - so bleibt das komplette Brett frei sichtbar.
      if (this.gameOver && this.reveal) ui.text(ctx, t("sud.hide_solution"), fl(this.width / 2), cy, this.small, this.accent, "center");
      else ui.text(ctx, this.fmtTime(), fl(this.width / 2), cy, this.clockFont, this.accent, "center");

      const errTxt = this.failLimit ? t("sud.errors", { n: this.errors, m: FAIL_LIMIT }) : t("sud.errors_free", { n: this.errors });
      const right = [[errTxt, this.errors ? ui.RED : ui.TEXT_DIM]];
      if (this.canHint) right.push([t("sud.hints", { n: MAX_HINTS - this.hintsUsed }), ui.TEXT_DIM]);
      if (this.noteMode) right.push([t("sud.pad_note").toUpperCase(), this.accent]);
      let x = this.width - 12;
      for (const [txt, c] of right) {
        const rr = ui.text(ctx, txt, x, cy, this.small, c, "midright");
        x -= rr.w + 16;
      }

      if (this.msg) ui.text(ctx, this.msg, fl(this.width / 2), this.hudH + 14, this.small, ui.RED, "center");
    }

    // ----- Brett ------------------------------------------------------------
    drawBoard(ctx) {
      const bs = this.cell * 9;
      const sel = this.sel;
      const selVal = this.board[sel];
      const selR = fl(sel / 9), selC = sel % 9;
      const selB = gen.BOX_OF[sel];

      // Zellfarben je Frame aus Palette + Akzent mischen (Theme-fähig).
      const cCell = ui.PANEL;
      const cPeer = ui.PANEL_LIGHT;
      const cSel = ui.mix(ui.PANEL_LIGHT, this.accent, 0.35);
      const cSame = ui.mix(ui.PANEL_LIGHT, this.accent, 0.18);
      const cBad = ui.mix(ui.PANEL, ui.RED, 0.35);
      const showSolution = this.gameOver && this.reveal;

      for (let i = 0; i < 81; i++) {
        const r = fl(i / 9), c = i % 9;
        const x = this.bx + c * this.cell;
        const y = this.by + r * this.cell;
        const cxm = x + this.cell / 2, cym = y + this.cell / 2;

        // Zellhintergrund: Auswahl > Konflikt > gleiche Ziffer > Peers
        let bg;
        if (i === sel) bg = cSel;
        else if (this.canCheck && this.conflicts.has(i)) bg = cBad;
        else if (this.canCheck && selVal && this.board[i] === selVal) bg = cSame;
        else if (r === selR || c === selC || gen.BOX_OF[i] === selB) bg = cPeer;
        else bg = cCell;
        draw.rect(ctx, bg, [x, y, this.cell, this.cell]);

        // Lösungs-Ansicht (A nach Spielende): fehlende/falsche Zellen
        // zeigen die richtige Ziffer in Akzentfarbe.
        if (showSolution && this.board[i] !== this.solution[i]) {
          ui.text(ctx, String(this.solution[i]), cxm, cym, this.numFont, this.accent, "center");
          continue;
        }

        const v = this.board[i];
        if (v) {
          let col;
          if (this.given[i]) col = ui.TEXT;
          else if (this.canCheck && this.wrong.has(i)) col = ui.RED;
          else if (this.locked[i]) col = ui.GREEN;
          else col = COL_USER;
          ui.text(ctx, String(v), cxm, cym, this.numFont, col, "center");
        } else if (this.notes[i].size) {
          const third = fl(this.cell / 3);
          for (const d of this.notes[i]) {
            const nx = x + ((d - 1) % 3) * third + fl(third / 2);
            const ny = y + fl((d - 1) / 3) * third + fl(third / 2);
            ui.text(ctx, String(d), nx, ny, this.noteFont, ui.TEXT_FAINT, "center");
          }
        }
      }

      // Gitterlinien (dünn + 3x3 fett)
      for (let k = 0; k < 10; k++) {
        const thick = k % 3 === 0;
        const w = thick ? 2 : 1;
        const col = thick ? ui.BORDER_LIGHT : ui.BORDER;
        const off = thick ? 0 : 0.5;
        const x = this.bx + k * this.cell + off;
        const y = this.by + k * this.cell + off;
        draw.line(ctx, col, [x, this.by], [x, this.by + bs], w);
        draw.line(ctx, col, [this.bx, y], [this.bx + bs, y], w);
      }
    }

    // ----- Ziffernfeld ------------------------------------------------------
    drawPad(ctx) {
      // Verbleibende Anzahl je Ziffer (comfort/assist zeigen sie an).
      const counts = new Array(10).fill(0);
      for (const v of this.board) counts[v]++;

      for (let d = 1; d <= 9; d++) {
        const r = this.padRects[String(d)];
        const done = counts[d] >= 9;
        draw.rect(ctx, ui.BTN, r, 0, 6);
        draw.rect(ctx, ui.BORDER, r, 1, 6);
        ui.text(ctx, String(d), r.centerx, r.centery - (this.canCheck ? 5 : 0), this.numFont, done ? ui.TEXT_DIM : ui.TEXT, "center");
        if (this.canCheck && !done) ui.text(ctx, String(9 - counts[d]), r.centerx, r.bottom - 9, this.tiny, ui.TEXT_DIM, "center");
      }

      let r = this.padRects.erase;
      draw.rect(ctx, ui.BTN, r, 0, 6);
      draw.rect(ctx, ui.BORDER, r, 1, 6);
      ui.text(ctx, t("sud.pad_erase"), r.centerx, r.centery, this.tiny, ui.TEXT, "center");

      if (this.padRects.note) {
        r = this.padRects.note;
        const on = this.noteMode;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, r, 0, 6);
        draw.rect(ctx, on ? this.accent : ui.BORDER, r, on ? 2 : 1, 6);
        ui.text(ctx, t("sud.pad_note") + " (N)", r.centerx, r.centery, this.tiny, on ? ui.TEXT : ui.TEXT_DIM, "center");
      }

      if (this.padRects.hint) {
        r = this.padRects.hint;
        const left = MAX_HINTS - this.hintsUsed;
        draw.rect(ctx, ui.BTN, r, 0, 6);
        draw.rect(ctx, ui.BORDER, r, 1, 6);
        ui.text(ctx, t("sud.pad_hint") + " (H) x" + left, r.centerx, r.centery, this.tiny, left ? ui.GREEN : ui.TEXT_DIM, "center");
      }

      ui.text(ctx, t("sud.hint"), fl(this.width / 2), this.height - 12, this.tiny, ui.TEXT_DIM, "center");
    }

    // ----- Ergebnis-Overlay ---------------------------------------------------
    drawResult(ctx) {
      draw.rect(ctx, [8, 10, 16, 185], [0, 0, this.width, this.height]);
      const cx = fl(this.width / 2), cy = fl(this.height / 2);

      let head, headCol, lines;
      if (this.won) {
        head = t("sud.win", { t: this.fmtTime() });
        headCol = ui.GREEN;
        lines = [
          [t("common.points", { score: this.score }), ui.TEXT],
          [t("sud.next"), ui.TEXT_DIM],
          [t("sud.show_solution"), ui.TEXT_DIM],
        ];
      } else {
        head = t("sud.lose");
        headCol = ui.RED;
        lines = [
          [t("sud.retry"), ui.TEXT_DIM],
          [t("sud.show_solution"), ui.TEXT_DIM],
        ];
      }

      // Panel hinter dem Ergebnis (Akzent-Rahmen, Breite folgt dem Inhalt).
      const pw = Math.max(Math.min(this.width - 40, 460), this.huge.width(head) + 40);
      const panel = new Rect(cx - fl(pw / 2), cy - 92, pw, 184);
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, this.accent, panel, 2, 14);

      ui.text(ctx, head, cx, cy - 50, this.huge, headCol, "center");
      let y = cy + 4;
      for (const [txt, col] of lines) {
        ui.text(ctx, txt, cx, y, this.small, col, "center");
        y += 30;
      }
    }
  }

  PG.register(SudokuGame, {
    id: "SudokuGame",
    key: "sudoku",
    name: "Sudoku",
    modes: [
      ["classic", "sud.mode.classic"],
      ["notes", "sud.mode.notes"],
      ["comfort", "sud.mode.comfort"],
      ["assist", "sud.mode.assist"],
    ],
    settingsKey: "sudoku",
    defaults: { difficulty: 0, fail_limit: true, last_level: {} },
    wantsRightClick: true, // Rechtsklick = radieren
  });
})();
