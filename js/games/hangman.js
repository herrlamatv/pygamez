/*
 * hangman.js - Galgenmännchen (Port von games/hangman.py)
 * ========================================================
 * Errate das Wort, bevor der Galgen fertig ist.
 *
 * - Jeder falsche Buchstabe zeichnet ein weiteres Körperteil (Kopf, Rumpf, zwei
 *   Arme, zwei Beine); nach 6 Fehlern ist die Partie verloren.
 * - Drei Modi über die Wortlänge: Kurze Wörter (3-5), Gemischt (3-12) und
 *   Lange Wörter (7-12).
 * - Endlos-Streak wie bei Wordle: jedes erratene Wort bringt Punkte (mehr
 *   Restleben + längeres Wort = mehr), danach kommt sofort ein neues Wort. Der
 *   erste Verlust beendet die Partie - die Summe ist der Highscore.
 *
 * Steuerung: Buchstaben A-Z tippen (über ev.char; Umlaute/Akzente werden wie im
 * Original ignoriert) oder die Bildschirmtastatur anklicken.
 * Nach Ende bzw. erratenem Wort: Enter/Klick geht weiter.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const COL_KEY = [70, 72, 86];
  const COL_KEY_TEXT = [232, 233, 240];
  const COL_CORRECT = [106, 190, 120];
  const COL_ABSENT = [58, 58, 64];
  const COL_WORD = [236, 238, 246];

  const MAX_WRONG = 6;

  const QWERTY = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"];
  const QWERTZ = ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"];

  const PLAY = "play", SOLVED = "solved", OVER = "over";

  /** Getippten Buchstaben (A-Z) aus dem Ereignis holen, sonst null. */
  function letterOf(ev) {
    let ch = ev.char && ev.char.length === 1 ? ev.char : ev.key && ev.key.length === 1 ? ev.key : "";
    ch = ch.toUpperCase();
    return /^[A-Z]$/.test(ch) ? ch : null;
  }

  class HangmanGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.lang = PG.lang;
      this.words = PG.hangmanWords.wordsFor(this.lang, ["short", "mixed", "long"].includes(this.mode) ? this.mode : "mixed");
      this.score = 0;
      this.gameOver = false;
      this.solvedCount = 0;
      this.makeFonts();
      this.newWord();
      this.layout();
    }

    makeFonts() {
      this.hud = ui.font(20, true);
      this.small = ui.font(15);
      this.keyFont = ui.font(16, true);
      this.wordFont = ui.font(Math.max(30, Math.floor(this.height / 12)), true);
      this.huge = ui.font(Math.max(28, Math.floor(this.height / 12)), true);
    }

    newWord() {
      this.word = PG.rand.choice(this.words);
      this.guessed = new Set();
      this.wrong = 0;
      this.lastPoints = 0;
      this.state = PLAY;
    }

    layout() {
      // Tastatur unten (3 Reihen, nur Buchstaben)
      this.keyH = Math.max(30, Math.floor(this.height / 12));
      this.keyGap = Math.max(3, Math.floor(this.width / 160));
      const kbH = 3 * this.keyH + 4 * this.keyGap;
      this.kbTop = this.height - kbH - 6;
      this.buildKeyboard();
      // Wortzeile knapp über der Tastatur
      this.wordY = this.kbTop - 46;
      // Galgen-Bereich zwischen HUD und Wortzeile
      this.drawTop = 64;
      this.drawBottom = this.wordY - 24;
    }

    buildKeyboard() {
      const layout = this.lang === "de" ? QWERTZ : QWERTY;
      this.keyRects = {};
      let y = this.kbTop;
      for (const rowstr of layout) {
        const keys = rowstr.split("");
        let kw = Math.floor((this.width - (keys.length + 1) * this.keyGap) / keys.length);
        kw = Math.max(20, Math.min(kw, 46));
        const rowW = keys.length * kw + (keys.length - 1) * this.keyGap;
        let x = Math.floor((this.width - rowW) / 2);
        for (const ch of keys) {
          this.keyRects[ch] = new PG.Rect(x, y, kw, this.keyH);
          x += kw + this.keyGap;
        }
        y += this.keyH + this.keyGap;
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SOLVED || this.state === OVER) {
        if (this.isContinue(ev)) {
          if (this.state === OVER) {
            this.gameOver = false;
            this.reset();
          } else {
            this.newWord();
            this.playSound("click");
          }
        }
        return;
      }
      if (ev.kind === "keydown") {
        const ch = letterOf(ev);
        if (ch) this.guess(ch);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (const ch in this.keyRects) {
          if (this.keyRects[ch].collidepoint(ev.pos)) {
            this.guess(ch);
            return;
          }
        }
      }
    }

    isContinue(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && (ev.key === "Return" || ev.key === "space"));
    }

    guess(ch) {
      if (this.state !== PLAY || this.guessed.has(ch)) return;
      this.guessed.add(ch);
      if (this.word.includes(ch)) {
        this.playSound("click");
        if (this.word.split("").every((c) => this.guessed.has(c))) this.solve();
      } else {
        this.wrong += 1;
        this.playSound("move");
        if (this.wrong >= MAX_WRONG) {
          this.state = OVER;
          this.gameOver = true; // die App speichert den Highscore
          if (this.solvedCount === 0) this.reportResult(false);
          this.playSound("gameover");
        }
      }
    }

    solve() {
      const pts = 10 + (MAX_WRONG - this.wrong) * 8 + this.word.length;
      this.lastPoints = pts;
      this.score += pts;
      this.solvedCount += 1;
      this.state = SOLVED;
      this.reportResult(true);
      if (this.wrong === 0) this.achEvent("hangman_clean");
      this.playSound("win");
    }

    update(dt) {}

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.drawGallows(ctx);
      this.drawWord(ctx);
      this.drawKeyboard(ctx);
      if (this.state === SOLVED) {
        this.drawBanner(ctx, t("hang.solved", { n: this.lastPoints }), this.accent, t("hang.next"));
      } else if (this.state === OVER) {
        this.drawBanner(ctx, t("hang.gameover"), [228, 96, 96], t("hang.word_was", { w: this.word }) + "   ·   " + t("common.enter_restart"));
      }
    }

    drawHud(ctx) {
      ui.text(ctx, t("hang.title"), 20, 30, this.hud, this.accent, "midleft");
      ui.text(ctx, t("hang.score", { n: this.score }), this.width / 2, 22, this.small, ui.GOLD, "center");
      ui.text(ctx, t("hang.misses", { a: this.wrong, b: MAX_WRONG }), this.width - 20, 22, this.small, ui.TEXT_DIM, "midright");
      ui.text(ctx, t("hang.solved_n", { n: this.solvedCount }), this.width - 20, 44, this.small, ui.TEXT_DIM, "midright");
    }

    drawGallows(ctx) {
      // Zeichenbereich (quadratisch) mittig im oberen Feld
      let size = Math.min(this.width - 80, this.drawBottom - this.drawTop);
      size = Math.max(120, size);
      const ox = Math.floor((this.width - size) / 2);
      const oy = this.drawTop + Math.max(0, Math.floor((this.drawBottom - this.drawTop - size) / 2));
      const u = size / 10;
      const wood = ui.mix(this.accent, [170, 140, 110], 0.5);
      const lw = Math.max(3, Math.floor(u * 0.28));
      const P = (gx, gy) => [Math.floor(ox + gx * u), Math.floor(oy + gy * u)];

      // Galgen (immer sichtbar)
      draw.line(ctx, wood, P(1, 9.4), P(6, 9.4), lw); // Boden
      draw.line(ctx, wood, P(2.5, 9.4), P(2.5, 0.8), lw); // Pfosten
      draw.line(ctx, wood, P(2.5, 0.8), P(6.2, 0.8), lw); // Balken
      draw.line(ctx, wood, P(6.2, 0.8), P(6.2, 1.8), lw); // Seil

      const red = [232, 110, 100];
      const parts = this.wrong;
      const [cx, cy] = P(6.2, 2.7);
      const headR = Math.floor(u * 0.9);
      const bodyBottom = Math.floor(cy + headR + u * 2.6);
      const shoulder = Math.floor(cy + headR + u * 0.5);
      if (parts >= 1) draw.circle(ctx, red, [cx, cy], headR, Math.max(2, lw - 1)); // Kopf
      if (parts >= 2) draw.line(ctx, red, [cx, cy + headR], [cx, bodyBottom], lw); // Rumpf
      if (parts >= 3) draw.line(ctx, red, [cx, shoulder], [Math.floor(cx - u * 1.3), Math.floor(shoulder + u * 1.2)], lw); // linker Arm
      if (parts >= 4) draw.line(ctx, red, [cx, shoulder], [Math.floor(cx + u * 1.3), Math.floor(shoulder + u * 1.2)], lw); // rechter Arm
      if (parts >= 5) draw.line(ctx, red, [cx, bodyBottom], [Math.floor(cx - u * 1.2), Math.floor(bodyBottom + u * 1.6)], lw); // linkes Bein
      if (parts >= 6) draw.line(ctx, red, [cx, bodyBottom], [Math.floor(cx + u * 1.2), Math.floor(bodyBottom + u * 1.6)], lw); // rechtes Bein
    }

    drawWord(ctx) {
      const letters = this.word.split("");
      const n = letters.length;
      const slot = Math.min(Math.floor(this.wordFont.height * 0.9), Math.floor((this.width - 40) / n));
      const gap = Math.max(6, Math.floor(slot / 5));
      const total = n * slot + (n - 1) * gap;
      let x = Math.floor((this.width - total) / 2);
      const revealAll = this.state === OVER;
      for (const ch of letters) {
        const shown = this.guessed.has(ch) || revealAll;
        // Grundlinie
        draw.line(ctx, ui.TEXT_DIM, [x, this.wordY + slot], [x + slot, this.wordY + slot], 3);
        if (shown) {
          const col = this.guessed.has(ch) ? COL_WORD : [228, 130, 120];
          ui.text(ctx, ch, x + Math.floor(slot / 2), this.wordY + Math.floor(slot / 2), this.wordFont, col, "center");
        }
        x += slot + gap;
      }
    }

    drawKeyboard(ctx) {
      for (const ch in this.keyRects) {
        const rc = this.keyRects[ch];
        const g = this.guessed.has(ch);
        const inWord = this.word.includes(ch);
        const col = g ? (inWord ? COL_CORRECT : COL_ABSENT) : COL_KEY;
        draw.rect(ctx, col, rc, 0, 6);
        const txt = g && !inWord ? ui.TEXT_FAINT : COL_KEY_TEXT;
        ui.text(ctx, ch, rc.centerx, rc.centery, this.keyFont, txt, "center");
      }
    }

    drawBanner(ctx, title, color, sub) {
      const w = Math.min(this.width - 40, 500);
      const h = 100;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), this.wordY - 130, w, h);
      rc.top = Math.max(60, rc.top);
      draw.rect(ctx, [16, 18, 24, 238], rc);
      draw.rect(ctx, color, rc, 2, 14);
      ui.text(ctx, title, rc.centerx, rc.y + 36, this.huge, color, "center");
      ui.text(ctx, sub, rc.centerx, rc.y + 74, this.small, ui.TEXT_DIM, "center");
    }
  }

  PG.register(HangmanGame, {
    id: "HangmanGame",
    key: "hangman",
    name: { default: "Hangman", de: "Galgenmännchen", fr: "Le pendu", es: "Ahorcado", pt: "Forca" },
    modes: [["short", "hang.mode.short"], ["mixed", "hang.mode.mixed"], ["long", "hang.mode.long"]],
  });
})();
