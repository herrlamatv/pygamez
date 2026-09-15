/*
 * pong.js - Pong gegen die KI (Port von games/pong.py)
 * =====================================================
 * - Steuerung: W/S oder Pfeil hoch/runter bewegen den linken Schläger; die
 *   Aktions-Taste (Leertaste/Enter) hält ihn an.
 * - Der rechte Schläger wird von einer einfachen KI gesteuert.
 * - Bewegungsmodus pro Steuerung umschaltbar (im Spiel):
 *     * Dauer  : einmal drücken -> der Schläger fährt dauerhaft weiter
 *                (bis Richtungswechsel oder Aktions-Taste). Standard.
 *     * Halten : der Schläger bewegt sich nur, solange man die Taste HÄLT
 *                (nutzt die Tastenwiederholung) und stoppt beim Loslassen.
 *   Umschalttasten:  X = Steuerung 1 (WASD),  N = Steuerung 2 (Pfeile).
 *   Die Einstellung wird dauerhaft gespeichert ("pong").
 * - Ball-Physik mit Beschleunigung und Winkel je nach Treffpunkt.
 * - Es wird bis 5 Punkte gespielt. Als Highscore zählen die eigenen Punkte.
 * - Web-Version: nur Einzelspieler (der 2-Spieler-Modus entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme).
  const COL_BALL = [255, 220, 90]; // klassischer gelber Ball
  const COL_P1 = [140, 230, 160]; // linker Schläger (Spieler)

  const PADDLE_W = 12;
  const PADDLE_H = 80;
  const PADDLE_SPEED = 380; // Pixel/Sekunde (Spieler)
  const AI_SPEED = 300; // Pixel/Sekunde (KI, absichtlich langsamer -> schlagbar)
  const BALL_SIZE = 14;
  const BALL_START_SPEED = 320;
  const BALL_MAX_SPEED = 650;
  const WIN_SCORE = 5;

  // So lange (Sekunden) fährt der Schläger im "Halten"-Modus nach dem letzten
  // Tastendruck noch weiter. Die Tastenwiederholung feuert schneller.
  const HOLD_KEEPALIVE = 0.12;

  // Feste Umschalttasten für den Bewegungsmodus.
  const TOGGLE_KEYS = { x: "p1", X: "p1", n: "p2", N: "p2" };

  class PongGame extends PG.Game {
    reset() {
      this.score = 0; // = Punkte des linken Spielers
      this.gameOver = false;

      this.playerY = this.height / 2 - PADDLE_H / 2;
      this.aiY = this.height / 2 - PADDLE_H / 2;
      this.playerScore = 0;
      this.aiScore = 0;

      // Bewegungszustand pro Steuerung ("p1"/"p2"):
      //   dir  : aktuelle Richtung (-1/0/1)
      //   keep : Rest-Zeit im "Halten"-Modus, bis der Schläger stoppt
      this.dir = { p1: 0, p2: 0 };
      this.keep = { p1: 0, p2: 0 };
      this.hold = { p1: !!this.opts.hold_p1, p2: !!this.opts.hold_p2 };

      this.makeFonts();
      this.serve(PG.rand.choice([true, false]));
    }

    // ----- Layout ----------------------------------------------------------
    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, Math.floor(h / 26)));
      this.bigFont = ui.font(Math.max(36, Math.floor(h / 10)), true);
      this.small = ui.font(Math.max(13, Math.floor(h / 36)));
    }

    /** Setzt den Ball in die Mitte und gibt ihm eine Startrichtung. */
    serve(towardPlayer) {
      this.ballX = this.width / 2 - BALL_SIZE / 2;
      this.ballY = this.height / 2 - BALL_SIZE / 2;
      const winkel = PG.rand.uniform(-0.35, 0.35);
      const richtung = towardPlayer ? -1 : 1;
      this.ballVx = richtung * BALL_START_SPEED;
      this.ballVy = BALL_START_SPEED * winkel;
    }

    // ----- Eingabe ---------------------------------------------------------
    handleEvent(ev) {
      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.reset();
        return;
      }

      // Bewegungsmodus umschalten: X = Steuerung 1, N = Steuerung 2.
      if (TOGGLE_KEYS[ev.key]) {
        this.toggleHold(TOGGLE_KEYS[ev.key]);
        return;
      }

      // Beide Belegungen steuern den linken Schläger.
      this.moveScheme("p1", ev.key);
      this.moveScheme("p2", ev.key);
    }

    /** Setzt Richtung/Keepalive für eine Steuerung anhand einer Taste. */
    moveScheme(scheme, key) {
      if (this.isAction(key, "up", scheme)) this.press(scheme, -1);
      else if (this.isAction(key, "down", scheme)) this.press(scheme, 1);
      else if (this.isAction(key, "action", scheme)) {
        // Aktions-Taste hält im Dauer-Modus an (im Halten-Modus unnötig).
        this.dir[scheme] = 0;
        this.keep[scheme] = 0;
      }
    }

    press(scheme, richtung) {
      this.dir[scheme] = richtung;
      // Im Halten-Modus fährt der Schläger nur kurz weiter; die
      // Tastenwiederholung frischt das laufend auf, solange man hält.
      if (this.hold[scheme]) this.keep[scheme] = HOLD_KEEPALIVE;
    }

    /** Schaltet für eine Steuerung zwischen Dauer- und Halten-Modus um. */
    toggleHold(scheme) {
      this.hold[scheme] = !this.hold[scheme];
      // Laufende Bewegung beim Umschalten stoppen.
      this.dir[scheme] = 0;
      this.keep[scheme] = 0;
      this.opts["hold_" + scheme] = this.hold[scheme];
      this.saveSettings();
      this.playSound("select");
    }

    // ----- Logik -----------------------------------------------------------
    update(dt) {
      if (this.gameOver) return;

      // Halten-Modus: Bewegung ausklingen lassen, wenn keine Tastenwiederholung
      // mehr kommt (Taste losgelassen).
      for (const scheme of ["p1", "p2"]) {
        if (this.hold[scheme] && this.dir[scheme] !== 0) {
          this.keep[scheme] -= dt;
          if (this.keep[scheme] <= 0) this.dir[scheme] = 0;
        }
      }

      // --- Linker Schläger: beide Steuerungen bewegen ihn ---
      const leftDir = PG.clamp(this.dir.p1 + this.dir.p2, -1, 1);
      this.playerY += leftDir * PADDLE_SPEED * dt;
      this.playerY = PG.clamp(this.playerY, 0, this.height - PADDLE_H);

      // --- Rechter Schläger: KI ---
      const ziel = this.ballY + BALL_SIZE / 2 - PADDLE_H / 2;
      if (this.aiY < ziel - 8) this.aiY += AI_SPEED * dt;
      else if (this.aiY > ziel + 8) this.aiY -= AI_SPEED * dt;
      this.aiY = PG.clamp(this.aiY, 0, this.height - PADDLE_H);

      // --- Ball bewegen ---
      this.ballX += this.ballVx * dt;
      this.ballY += this.ballVy * dt;

      if (this.ballY <= 0) {
        this.ballY = 0;
        this.ballVy = Math.abs(this.ballVy);
        this.playSound("bounce");
      } else if (this.ballY + BALL_SIZE >= this.height) {
        this.ballY = this.height - BALL_SIZE;
        this.ballVy = -Math.abs(this.ballVy);
        this.playSound("bounce");
      }

      // pygame.Rect schneidet Kommazahlen ab
      const ballRect = new PG.Rect(Math.trunc(this.ballX), Math.trunc(this.ballY), BALL_SIZE, BALL_SIZE);
      const playerRect = new PG.Rect(20, Math.trunc(this.playerY), PADDLE_W, PADDLE_H);
      const aiRect = new PG.Rect(this.width - 20 - PADDLE_W, Math.trunc(this.aiY), PADDLE_W, PADDLE_H);

      if (ballRect.colliderect(playerRect) && this.ballVx < 0) this.bounce(playerRect, true);
      else if (ballRect.colliderect(aiRect) && this.ballVx > 0) this.bounce(aiRect, false);

      // --- Punkte ---
      if (this.ballX + BALL_SIZE < 0) {
        // rechts punktet
        this.aiScore += 1;
        this.playSound("point");
        this.nachPunkt(true);
      } else if (this.ballX > this.width) {
        // links punktet
        this.playerScore += 1;
        this.score = this.playerScore;
        this.playSound("point");
        this.nachPunkt(false);
      }
    }

    /** Ball am Schläger reflektieren; Winkel hängt vom Treffpunkt ab. */
    bounce(paddle, nachRechts) {
      let treff = this.ballY + BALL_SIZE / 2 - (paddle.y + PADDLE_H / 2);
      treff /= PADDLE_H / 2;

      const speed = Math.min(BALL_MAX_SPEED, Math.abs(this.ballVx) * 1.06 + 20);
      this.ballVx = speed * (nachRechts ? 1 : -1);
      this.ballVy = speed * treff;
      if (nachRechts) this.ballX = paddle.right;
      else this.ballX = paddle.left - BALL_SIZE;
      this.playSound("bounce");
      this.rumble(60);
    }

    nachPunkt(towardPlayer) {
      if (this.playerScore >= WIN_SCORE || this.aiScore >= WIN_SCORE) {
        this.gameOver = true;
        this.playSound(this.playerScore > this.aiScore ? "win" : "gameover");
        this.reportResult(this.playerScore > this.aiScore);
        this.rumble(200);
      } else {
        this.serve(towardPlayer);
      }
    }

    // ----- Zeichnen ----------------------------------------------------------
    draw(ctx) {
      const W = this.width, H = this.height;
      ui.drawBackground(ctx, W, H);

      // Gestrichelte Mittellinie im dezenten Rahmenton des Themes.
      for (let y = 0; y < H; y += 28) draw.rect(ctx, ui.BORDER, [Math.floor(W / 2) - 2, y, 4, 16]);

      draw.rect(ctx, COL_P1, [20, Math.trunc(this.playerY), PADDLE_W, PADDLE_H], 0, 4);
      const rechtsFarbe = ui.TEXT;
      draw.rect(ctx, rechtsFarbe, [W - 20 - PADDLE_W, Math.trunc(this.aiY), PADDLE_W, PADDLE_H], 0, 4);

      draw.rect(ctx, COL_BALL, [Math.trunc(this.ballX), Math.trunc(this.ballY), BALL_SIZE, BALL_SIZE], 0, 3);

      // Spielstand mittig oben (rechtsbündig/linksbündig um die Mittellinie).
      ui.text(ctx, String(this.playerScore), Math.floor(W / 2) - 44, 16, this.bigFont, ui.TEXT, "topright");
      ui.text(ctx, String(this.aiScore), Math.floor(W / 2) + 44, 16, this.bigFont, ui.TEXT);

      ui.text(ctx, t("pong.you"), 34, 12, this.font, COL_P1);
      ui.text(ctx, t("pong.ai"), W - 34, 12, this.font, rechtsFarbe, "topright");

      if (!this.gameOver) this.drawModeHud(ctx);

      if (this.gameOver) {
        const gewonnen = this.playerScore > this.aiScore;
        this.drawOverlay(ctx, gewonnen ? t("pong.won") : t("pong.lost"), gewonnen ? ui.GREEN : ui.RED, t("common.enter_restart"));
      }
    }

    /** Endstand-Box: transluzentes Themen-Panel mit farbigem Rahmen. */
    drawOverlay(ctx, titel, farbe, hinweis) {
      const th = this.bigFont.height;
      const hh = this.font.height;
      const bw = Math.max(this.bigFont.width(titel), this.font.width(hinweis)) + 80;
      const bh = th + hh + 58;
      const rect = new PG.Rect(0, 0, bw, bh);
      rect.center = [Math.floor(this.width / 2), Math.floor(this.height / 2)];
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], rect, 0, 16);
      draw.rect(ctx, farbe, rect, 2, 16);
      ui.text(ctx, titel, rect.centerx, rect.y + 20, this.bigFont, farbe, "midtop");
      // Neustart-Hinweis sanft pulsieren lassen.
      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0));
      ui.text(ctx, hinweis, rect.centerx, rect.y + 20 + th + 14, this.font, hintCol, "midtop");
    }

    /** Zeigt unten den Bewegungsmodus je Steuerung + die Umschalttasten. */
    drawModeHud(ctx) {
      const y = this.height - this.small.height - 8;
      const zeile = (scheme) => [this.hold[scheme] ? t("pong.hold") : t("pong.continuous"), this.hold[scheme] ? ui.GREEN : ui.TEXT_DIM];
      const [m1, c1] = zeile("p1");
      ui.text(ctx, t("pong.scheme1", { mode: m1 }), 10, y, this.small, c1);
      const [m2, c2] = zeile("p2");
      ui.text(ctx, t("pong.scheme2", { mode: m2 }), this.width - 10, y, this.small, c2, "topright");
    }
  }

  PG.register(PongGame, {
    id: "PongGame",
    key: "pong",
    name: "Pong",
    settingsKey: "pong",
    defaults: { hold_p1: false, hold_p2: false },
  });
})();
