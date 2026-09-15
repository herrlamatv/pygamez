/*
 * wordle.js - Wordle (Port von games/wordle.py)
 * ==============================================
 * Errate das 5-Buchstaben-Wort in höchstens 6 Versuchen.
 *
 * - Nach jedem Rateversuch färben sich die Buchstaben: grün = richtig (Position
 *   stimmt), gelb = im Wort (falsche Position), grau = nicht enthalten. Doppelte
 *   Buchstaben werden korrekt gezählt (Standard-Wordle-Algorithmus).
 * - Endlos-Streak als Highscore: für jedes gelöste Wort gibt es Punkte (weniger
 *   Versuche = mehr), danach kommt sofort ein neues Wort. Das erste NICHT gelöste
 *   Wort beendet die Partie; die gesammelten Punkte sind der Highscore.
 * - Die Lösungswörter stammen aus einer kuratierten Liste je Sprache
 *   (wordle_words.js, nur A-Z). Rateversuche werden NICHT gegen ein
 *   Wörterbuch geprüft - jede 5-Buchstaben-Eingabe ist erlaubt.
 *
 * Steuerung: Buchstabentasten A-Z tippen (über ev.char, damit auch QWERTZ/
 * AZERTY stimmen; Umlaute/Akzente werden wie im Original ignoriert),
 * Enter = raten (bei 5 Buchstaben), Backspace = löschen. Die
 * Bildschirmtastatur unten ist auch anklickbar.
 * Nach Ende bzw. gelöstem Wort: Enter/Klick geht weiter.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const COL_TILE_EMPTY = [30, 30, 38];
  const COL_TILE_BORDER = [58, 58, 70];
  const COL_TILE_ACTIVE = [90, 90, 108];
  const COL_CORRECT = [106, 170, 100]; // grün (= Sidebar-Farbe #6aaa64)
  const COL_PRESENT = [201, 180, 88]; // gelb
  const COL_ABSENT = [58, 58, 62]; // grau
  const COL_TEXT = [235, 236, 240];
  const COL_DIM = [150, 152, 166];
  const COL_KEY = [70, 72, 86];
  const COL_KEY_TEXT = [232, 233, 240];
  const COL_ACCENT = [106, 170, 100];

  const ROWS = 6, COLS = 5;
  const REVEAL_STEP = 0.14; // Sekunden je Kachel bei der Aufdeckung

  const PLAY = "play", REVEAL = "reveal", SOLVED = "solved", OVER = "over";

  const QWERTY = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
  const QWERTZ = ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"];

  // Rangordnung der Buchstaben-Zustände (höher gewinnt in der Tastatur-Färbung)
  const RANK = { absent: 1, present: 2, correct: 3 };

  /** Standard-Wordle-Bewertung mit korrekter Doppelbuchstaben-Zählung. */
  function evaluate(guess, answer) {
    const result = new Array(5).fill("absent");
    const rest = answer.split("");
    for (let i = 0; i < 5; i++) {
      if (guess[i] === answer[i]) {
        result[i] = "correct";
        rest[i] = null;
      }
    }
    for (let i = 0; i < 5; i++) {
      if (result[i] === "correct") continue;
      const j = rest.indexOf(guess[i]);
      if (j >= 0) {
        result[i] = "present";
        rest[j] = null;
      }
    }
    return result;
  }

  /** Getippten Buchstaben (A-Z) aus dem Ereignis holen, sonst null. */
  function letterOf(ev) {
    const ch = ev.char && ev.char.length === 1 ? ev.char : ev.key && ev.key.length === 1 ? ev.key : "";
    // Erst auf ASCII-Buchstaben prüfen, dann groß schreiben (wie isascii()/isalpha()
    // im Original) - sonst würde z.B. "ı".toUpperCase() zu "I".
    return /^[A-Za-z]$/.test(ch) ? ch.toUpperCase() : null;
  }

  class WordleGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.solvedCount = 0;
      this.lang = PG.lang;
      this.words = PG.wordleWords.wordsFor(this.lang);
      this.makeFonts();
      this.newWord();
      this.layout();
    }

    makeFonts() {
      this.tileFont = ui.font(Math.max(20, Math.floor(this.height / 16)), true);
      this.keyFont = ui.font(15, true);
      this.hud = ui.font(18, true);
      this.small = ui.font(14);
      this.tiny = ui.font(12);
      this.huge = ui.font(Math.max(24, Math.floor(this.height / 13)), true);
    }

    newWord() {
      this.answer = PG.rand.choice(this.words);
      this.guesses = []; // Liste [wort, ergebnis]
      this.current = "";
      this.row = 0;
      this.keystate = {}; // Buchstabe -> zustand
      this.reveal = null; // {row, guess, result, t}
      this.lastPoints = 0;
      this.state = PLAY;
    }

    layout() {
      this.hudH = 44;
      // Tastatur unten
      this.keyH = Math.max(30, Math.floor(this.height / 12));
      this.keyGap = Math.max(3, Math.floor(this.width / 160));
      const kbH = 3 * this.keyH + 4 * this.keyGap;
      this.kbTop = this.height - kbH - 6;
      this.buildKeyboard();
      // Rasterbereich zwischen HUD und Tastatur
      const top = this.hudH + 8;
      const bottom = this.kbTop - 8;
      const gap = Math.max(4, Math.floor(this.width / 120));
      this.tile = Math.floor(Math.min((this.width - 40 - (COLS - 1) * gap) / COLS, (bottom - top - (ROWS - 1) * gap) / ROWS));
      this.tile = Math.max(24, this.tile);
      this.tileGap = gap;
      const gridW = COLS * this.tile + (COLS - 1) * gap;
      const gridH = ROWS * this.tile + (ROWS - 1) * gap;
      this.gridX = Math.floor((this.width - gridW) / 2);
      this.gridY = top + Math.max(0, Math.floor((bottom - top - gridH) / 2));
    }

    buildKeyboard() {
      const layout = this.lang === "de" ? QWERTZ : QWERTY;
      this.keyRects = {};
      this.enterRect = null;
      this.delRect = null;
      let y = this.kbTop;
      layout.forEach((rowstr, ri) => {
        const keys = rowstr.split("");
        // Untere Reihe bekommt ENTER links und DEL rechts
        const nSlots = keys.length + (ri === 2 ? 2 : 0);
        let kw = Math.floor((this.width - (nSlots + 1) * this.keyGap) / Math.max(1, nSlots));
        kw = Math.max(20, Math.min(kw, 44));
        const specialW = Math.floor(kw * 1.5);
        let rowW = keys.length * kw + (keys.length - 1) * this.keyGap;
        if (ri === 2) rowW += 2 * (specialW + this.keyGap);
        let x = Math.floor((this.width - rowW) / 2);
        if (ri === 2) {
          this.enterRect = new PG.Rect(x, y, specialW, this.keyH);
          x += specialW + this.keyGap;
        }
        for (const ch of keys) {
          this.keyRects[ch] = new PG.Rect(x, y, kw, this.keyH);
          x += kw + this.keyGap;
        }
        if (ri === 2) this.delRect = new PG.Rect(x, y, specialW, this.keyH);
        y += this.keyH + this.keyGap;
      });
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === OVER) {
        if (this.isContinue(ev)) {
          this.gameOver = false;
          this.reset();
        }
        return;
      }
      if (this.state === SOLVED) {
        if (this.isContinue(ev)) {
          this.newWord();
          this.playSound("click");
        }
        return;
      }
      if (this.state === REVEAL) return;
      // PLAY
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "BackSpace") {
          if (this.current) {
            this.current = this.current.slice(0, -1);
            this.playSound("move");
          }
        } else if (k === "Return") {
          this.submit();
        } else {
          const ch = letterOf(ev);
          if (ch) this.type(ch);
        }
      } else if (ev.kind === "mousedown") {
        this.clickKeyboard(ev.pos);
      }
    }

    isContinue(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    type(ch) {
      if (this.current.length < COLS) {
        this.current += ch;
        this.playSound("click");
      }
    }

    clickKeyboard(pos) {
      if (this.enterRect && this.enterRect.collidepoint(pos)) {
        this.submit();
        return;
      }
      if (this.delRect && this.delRect.collidepoint(pos)) {
        if (this.current) {
          this.current = this.current.slice(0, -1);
          this.playSound("move");
        }
        return;
      }
      for (const ch in this.keyRects) {
        if (this.keyRects[ch].collidepoint(pos)) {
          this.type(ch);
          return;
        }
      }
    }

    submit() {
      if (this.current.length !== COLS) {
        this.playSound("click");
        return;
      }
      const result = evaluate(this.current, this.answer);
      this.reveal = { row: this.row, guess: this.current, result, t: 0 };
      this.state = REVEAL;
      this.playSound("move");
    }

    finishReveal() {
      const rv = this.reveal;
      const guess = rv.guess, result = rv.result;
      this.guesses.push([guess, result]);
      for (let i = 0; i < COLS; i++) {
        const ch = guess[i], st = result[i];
        if (RANK[st] > (RANK[this.keystate[ch]] || 0)) this.keystate[ch] = st;
      }
      this.reveal = null;
      this.current = "";
      this.row += 1;
      if (result.every((st) => st === "correct")) {
        const pts = 10 + (ROWS - this.row) * 10; // weniger Versuche = mehr
        this.lastPoints = pts;
        this.score += pts;
        this.solvedCount += 1;
        this.state = SOLVED;
        this.reportResult(true);
        if (this.row <= 2) this.achEvent("wordle_two");
        this.playSound("win");
      } else if (this.row >= ROWS) {
        this.state = OVER;
        this.gameOver = true; // die App speichert den Highscore
        if (this.solvedCount === 0) this.reportResult(false);
        this.playSound("gameover");
      } else {
        this.state = PLAY;
      }
    }

    // ===================================================== Update
    update(dt) {
      if (this.state === REVEAL && this.reveal) {
        this.reveal.t += dt;
        if (this.reveal.t >= COLS * REVEAL_STEP + 0.1) this.finishReveal();
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Themen-Hintergrund statt flacher Fläche (reagiert auf Theme-Wechsel).
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.drawGrid(ctx);
      this.drawKeyboard(ctx);
      if (this.state === SOLVED) {
        this.drawBanner(ctx, t("wd.solved", { n: this.lastPoints }), COL_ACCENT, t("wd.next"));
      } else if (this.state === OVER) {
        this.drawBanner(ctx, t("wd.gameover"), [225, 110, 100], t("wd.answer_was", { w: this.answer }) + "   -   " + t("common.enter_restart"));
      }
    }

    drawHud(ctx) {
      draw.rect(ctx, [26, 26, 34], [0, 0, this.width, this.hudH]);
      draw.line(ctx, COL_TILE_BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = this.hudH / 2;
      ui.text(ctx, t("wd.score", { n: this.score }), 14, cy, this.hud, COL_ACCENT, "midleft");
      ui.text(ctx, t("wd.solved_n", { n: this.solvedCount }), this.width / 2, cy, this.small, COL_DIM, "center");
      ui.text(ctx, t("wd.tries", { a: this.guesses.length, b: ROWS }), this.width - 14, cy, this.small, COL_DIM, "midright");
    }

    tileColor(state) {
      return { correct: COL_CORRECT, present: COL_PRESENT, absent: COL_ABSENT }[state] || COL_TILE_EMPTY;
    }

    drawGrid(ctx) {
      for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) {
          const x = this.gridX + c * (this.tile + this.tileGap);
          const y = this.gridY + r * (this.tile + this.tileGap);
          const rc = new PG.Rect(x, y, this.tile, this.tile);
          let ch = "";
          let fill = COL_TILE_EMPTY;
          let border = COL_TILE_BORDER;
          if (r < this.guesses.length) {
            const [guess, result] = this.guesses[r];
            ch = guess[c];
            fill = this.tileColor(result[c]);
            border = fill;
          } else if (this.reveal && r === this.reveal.row) {
            ch = this.reveal.guess[c];
            // Kachel deckt sich nacheinander (Spalte für Spalte) auf
            if (this.reveal.t >= (c + 1) * REVEAL_STEP) {
              fill = this.tileColor(this.reveal.result[c]);
              border = fill;
            } else {
              border = COL_TILE_ACTIVE;
            }
          } else if (r === this.row) {
            if (c < this.current.length) {
              ch = this.current[c];
              border = COL_TILE_ACTIVE;
            }
          }
          draw.rect(ctx, fill, rc, 0, 6);
          draw.rect(ctx, border, rc, 2, 6);
          if (ch) ui.text(ctx, ch, rc.centerx, rc.centery, this.tileFont, COL_TEXT, "center");
        }
      }
    }

    drawKeyboard(ctx) {
      for (const ch in this.keyRects) {
        const rc = this.keyRects[ch];
        const st = this.keystate[ch];
        draw.rect(ctx, st ? this.tileColor(st) : COL_KEY, rc, 0, 5);
        ui.text(ctx, ch, rc.centerx, rc.centery, this.keyFont, COL_KEY_TEXT, "center");
      }
      for (const [rc, label] of [[this.enterRect, t("wd.enter")], [this.delRect, "<"]]) {
        if (!rc) continue;
        draw.rect(ctx, COL_KEY, rc, 0, 5);
        ui.text(ctx, label, rc.centerx, rc.centery, this.tiny, COL_KEY_TEXT, "center");
      }
    }

    drawBanner(ctx, title, color, sub) {
      const w = Math.min(this.width - 40, 460);
      const h = 92;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), this.gridY - 4, w, h);
      draw.rect(ctx, [16, 18, 24, 235], rc);
      draw.rect(ctx, color, rc, 2, 12);
      ui.text(ctx, title, rc.centerx, rc.y + 34, this.huge, color, "center");
      ui.text(ctx, sub, rc.centerx, rc.y + 66, this.small, COL_TEXT, "center");
    }
  }

  PG.register(WordleGame, {
    id: "WordleGame",
    key: "wordle",
    name: "Wordle",
  });
})();
