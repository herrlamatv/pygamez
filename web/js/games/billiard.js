/*
 * billiard.js - Billard / Pool gegen die KI (Port von games/billiard.py)
 * ========================================================================
 * 8-Ball, 9-Ball und ein regelfreier Übungsmodus in drei Ansichten
 * (im Setup UND per Taste V umschaltbar, wird gespeichert):
 *   - 2D   : klassische Draufsicht von oben.
 *   - 3D   : feste perspektivische Schrägansicht mit schattierten Kugeln.
 *   - Frei : wie 3D, aber die Kamera lässt sich mit der rechten Maustaste
 *            (oder Q/E) sanft um den Tisch drehen.
 *
 * Alle Bewegungen sind zeitschritt-basiert und weich abgebremst (Reibung). Die
 * Physik läuft in Teilschritten, damit schnelle Kugeln nicht "durchtunneln".
 *
 * Steuerung: Maus bewegt das Ziel, linke Maustaste gedrückt halten lädt die
 * Stoßstärke, Loslassen stößt. Alternativ: Pfeile links/rechts zielen,
 * hoch/runter Stärke, Leertaste stößt. Bei Ball-in-Hand (nach Foul) die weiße
 * Kugel mit der Maus platzieren. V = Ansicht, nach Spielende Enter = neue Partie.
 *
 * Punkte (Highscore) = gewonnene Frames gegen die KI (bzw. versenkte Kugeln im
 * Übungsmodus). Web-Version: nur gegen die KI (Hot-Seat entfällt).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ------------------------------------------------- Identitätsfarben (Tisch)
  const COL_CLOTH = [26, 112, 72];
  const COL_CLOTH_D = [20, 92, 58];
  const COL_RAIL = [74, 48, 30];
  const COL_RAIL_HI = [104, 70, 44];
  const COL_POCKET = [10, 12, 10];
  const COL_CUE = [245, 245, 240];
  const COL_AIM = [240, 240, 200];
  const COL_STICK = [208, 168, 96];
  const COL_STICK_D = [150, 116, 60];

  // Kugelfarben nach Nummer (1..15). 8 = schwarz.
  const BALL_COLORS = {
    1: [232, 190, 40], 2: [40, 84, 180], 3: [200, 48, 48], 4: [120, 60, 160],
    5: [224, 120, 40], 6: [34, 130, 80], 7: [150, 44, 52], 8: [26, 26, 30],
    9: [232, 190, 40], 10: [40, 84, 180], 11: [200, 48, 48], 12: [120, 60, 160],
    13: [224, 120, 40], 14: [34, 130, 80], 15: [150, 44, 52],
  };

  // ----------------------------------------------------------------- Physik / Tisch
  const HW = 127.0, HH = 63.5; // halbe Tischmaße (Tisch-Koordinaten, zentriert)
  const BR = 3.1; // Kugelradius
  const POCKET_R = 6.4;
  const FRICTION = 0.62; // Rollreibung pro Sekunde
  const WALL_E = 0.9; // Bandenrestitution
  const BALL_E = 0.94; // Kugel-Kugel-Restitution
  const STOP_EPS = 2.0; // darunter gilt eine Kugel als still
  const MAX_SPEED = 470.0; // maximale Stoßgeschwindigkeit
  const MAX_SHOT_TIME = 12.0;
  // Fester Physik-Takt wie die Desktop-Version (60 FPS): Teilschritt-Länge und
  // damit der Kontaktpunkt beim Kugel-Kugel-Stoß hängen von der Frame-Zeit ab.
  // Mit festem Schritt laufen Schnitt-Stöße bei jeder Bildrate (und bei
  // schwankenden Frame-Zeiten) genauso in die Tasche wie im Original.
  const PHYS_DT = 1 / 60;

  const POCKETS = [[-HW, -HH], [0, -HH], [HW, -HH], [-HW, HH], [0, HH], [HW, HH]];

  const SETUP = "setup", PLAY = "play", OVER = "over";
  const VARIANTS = ["8ball", "9ball", "practice"];
  const VIEWS = ["2d", "3d", "free"];
  const DIFFS = ["easy", "medium", "hard"];
  const AI_ACC = [0.8, 0.91, 0.985];

  function makeBall(x, y, num) {
    return { x, y, vx: 0, vy: 0, num, potted: false };
  }
  const speed2 = (b) => b.vx * b.vx + b.vy * b.vy;

  class BilliardGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const bs = this.opts;
      this.variant = VARIANTS.includes(bs.variant) ? bs.variant : "8ball";
      this.view = VIEWS.includes(bs.view) ? bs.view : "2d";
      this.diff = PG.clamp(Math.trunc(Number(bs.difficulty)) || 0, 0, 2);

      // Wiederholung der Partie (replay, siehe core/replay.js).
      this.rec = null;
      this.replay = null;
      this.replayRequest = null;
      this.rep = null; // gesetzt, solange nur abgespielt wird
      this.repAt = null;
      this.recRes = null;
      this.recDelta = new PG.replay.Delta();

      this.buildFonts();
      this.tableCache = null;
      this.wins = [0, 0];
      this.camYaw = 0.0;
      this.camYawT = 0.0;
      this.draggingCam = false;
      this.lastMx = 0;
      this.buildSetupLayout();
      this.setupCamera();
      this.newRack();
      this.state = SETUP;
    }

    /** Schriftgrößen aus der Auflösung ableiten (Theme-Fonts). */
    buildFonts() {
      const h = this.height;
      this.small = ui.font(Math.max(14, Math.floor(h / 32)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 40)));
      this.huge = ui.font(Math.max(26, Math.floor(h / 12)), true);
    }

    // ------------------------------------------------ Kamera / Projektion
    setupCamera() {
      this.hudH = 46;
      this.cx = this.width / 2.0;
      this.cy = this.hudH + (this.height - this.hudH) * 0.5;
      // 2D-Maßstab
      this.scale2d = Math.min((this.width - 46) / (2 * HW), (this.height - this.hudH - 40) / (2 * HH));
      // 3D-Pinhole
      this.focal = this.width * 0.92;
      this.camR = HW * 2.35;
      this.camEL = PG.radians(42);
      this.cy3d = this.hudH + (this.height - this.hudH) * 0.54;
      this.basis = this.camBasis();
    }

    camBasis() {
      const yaw = this.camYaw;
      const ce = Math.cos(this.camEL), se = Math.sin(this.camEL);
      const cyw = Math.cos(yaw), syw = Math.sin(yaw);
      const cam = [this.camR * ce * syw, -this.camR * ce * cyw, this.camR * se];
      // forward = normalize(target - cam) ; target = Ursprung
      let fx = -cam[0], fy = -cam[1], fz = -cam[2];
      const fl = Math.sqrt(fx * fx + fy * fy + fz * fz) || 1.0;
      fx /= fl; fy /= fl; fz /= fl;
      // right = normalize(forward x up), up_world=(0,0,1)
      let rx = fy, ry = -fx, rz = 0;
      const rl = Math.sqrt(rx * rx + ry * ry + rz * rz) || 1.0;
      rx /= rl; ry /= rl; rz /= rl;
      // up = right x forward
      const ux = ry * fz - rz * fy, uy = rz * fx - rx * fz, uz = rx * fy - ry * fx;
      return { cam, f: [fx, fy, fz], r: [rx, ry, rz], u: [ux, uy, uz] };
    }

    /** (x,y) Tisch-Koordinaten -> [sx, sy, scale, depth]. */
    project(x, y, z = 0.0) {
      if (this.view === "2d") return [this.cx + x * this.scale2d, this.cy + y * this.scale2d, this.scale2d, y];
      const { cam, f, r, u } = this.basis;
      const relx = x - cam[0], rely = y - cam[1], relz = z - cam[2];
      let zc = relx * f[0] + rely * f[1] + relz * f[2];
      if (zc < 1.0) zc = 1.0;
      const xc = relx * r[0] + rely * r[1] + relz * r[2];
      const yc = relx * u[0] + rely * u[1] + relz * u[2];
      return [this.cx + (this.focal * xc) / zc, this.cy3d - (this.focal * yc) / zc, this.focal / zc, zc];
    }

    /** Bildschirm -> (x,y) auf der Tischebene z=0. */
    unproject(sx, sy) {
      if (this.view === "2d") return [(sx - this.cx) / this.scale2d, (sy - this.cy) / this.scale2d];
      const { cam, f, r, u } = this.basis;
      const dxc = (sx - this.cx) / this.focal;
      const dyc = -(sy - this.cy3d) / this.focal;
      // Strahlrichtung in Weltkoordinaten
      const dx = dxc * r[0] + dyc * u[0] + f[0];
      const dy = dxc * r[1] + dyc * u[1] + f[1];
      const dz = dxc * r[2] + dyc * u[2] + f[2];
      if (Math.abs(dz) < 1e-6) return [0.0, 0.0];
      const s = -cam[2] / dz;
      return [cam[0] + s * dx, cam[1] + s * dy];
    }

    // ------------------------------------------------ Rack aufbauen
    newRack() {
      this.balls = [];
      this.cue = makeBall(-HW * 0.5, 0.0, 0);
      this.balls.push(this.cue);
      const fx = HW * 0.45;
      const sp = 2 * BR + 0.25;
      const dx = sp * 0.87;
      if (this.variant === "9ball") {
        const order = [1].concat(PG.rand.sample([2, 3, 4, 5, 6, 7, 8], 7), [9]);
        const spots = [[0, 0], [1, -0.5], [1, 0.5], [2, -1], [2, 0], [2, 1], [3, -0.5], [3, 0.5], [4, 0]];
        spots.forEach(([row, off], i) => this.balls.push(makeBall(fx + row * dx, off * sp, order[i])));
      } else {
        // 15er-Dreieck (8-Ball / Übung)
        let seq;
        if (this.variant === "8ball") {
          const others = PG.rand.shuffle([1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 13, 14, 15]);
          // 8 in die Mitte der dritten Reihe
          seq = [];
          let idx = 0;
          for (let row = 0; row < 5; row++) {
            for (let k = 0; k <= row; k++) {
              if (row === 2 && k === 1) seq.push(8);
              else seq.push(others[idx++]);
            }
          }
        } else {
          seq = PG.rand.shuffle(Array.from({ length: 15 }, (_, i) => i + 1));
        }
        let i = 0;
        for (let row = 0; row < 5; row++) {
          for (let k = 0; k <= row; k++) {
            this.balls.push(makeBall(fx + row * dx, (k - row / 2.0) * sp, seq[i++]));
          }
        }
      }
      this.group = [null, null];
      this.current = 0;
      this.ballInHand = false;
      this.breakDone = false;
      this.winner = null;
      this.phase = "aim";
      this.aim = 0.0;
      this.power = 0.35;
      this.charging = false;
      this.shotTime = 0.0;
      this.firstHit = null;
      this.cuePotted = false;
      this.pottedShot = [];
      this.pottedAll = [];
      this.rackBase = 0; // Übung: Kugeln aus früheren Racks (Punkte)
      this.physAcc = 0.0;
      this.msg = null;
      this.msgT = 0.0;
      this.aiDelay = 0.8;
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(380, this.width - 50);
      const y0 = Math.floor(this.height * 0.26);
      const gap = 8;
      const row = (y, n) => {
        const cw = (bw - gap * (n - 1)) / n;
        return Array.from({ length: n }, (_, i) => new PG.Rect(Math.trunc(cx - bw / 2 + i * (cw + gap)), y, Math.trunc(cw), 42));
      };
      this.varRects = row(y0, 3);
      this.viewRects = row(y0 + 84, 3);
      this.diffRects = row(y0 + 168, 3);
      this.startRect = new PG.Rect(cx - 95, y0 + 228, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.variant = VARIANTS[Number(k) - 1];
          this.saveSetting("variant", this.variant);
          this.playSound("click");
        } else if (k === "v" || k === "V") {
          this.cycleView();
        } else if (k === "d" || k === "D") {
          this.diff = (this.diff + 1) % 3;
          this.saveSetting("difficulty", this.diff);
          this.playSound("select");
        } else if (k === "Return" || k === "space") {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < 3; i++) {
          if (this.varRects[i].collidepoint(ev.pos)) {
            this.variant = VARIANTS[i];
            this.saveSetting("variant", this.variant);
            this.playSound("click");
            return;
          }
        }
        for (let i = 0; i < 3; i++) {
          if (this.viewRects[i].collidepoint(ev.pos)) {
            this.view = VIEWS[i];
            this.camYaw = this.camYawT = 0.0;
            this.saveSetting("view", this.view);
            this.playSound("select");
            return;
          }
        }
        for (let i = 0; i < 3; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diff = i;
            this.saveSetting("difficulty", i);
            this.playSound("select");
            return;
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    cycleView() {
      this.view = VIEWS[(VIEWS.indexOf(this.view) + 1) % 3];
      this.draggingCam = false;
      if (this.view !== "free") this.camYawT = 0.0;
      this.saveSetting("view", this.view);
      this.playSound("select");
    }

    startPlay() {
      this.setupCamera();
      this.newRack();
      this.recNew();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (ev.kind === "keydown" && (ev.key === "v" || ev.key === "V") && this.state === PLAY) {
        this.cycleView();
        return;
      }
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") this.restart();
          else if ((ev.key === "p" || ev.key === "P") && this.replay) this.openReplay();
          else if (ev.key === "s" || ev.key === "S") {
            this.state = SETUP;
            this.gameOver = false;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown" && ev.button === 1) {
          this.restart();
        }
        return;
      }
      if (this.state !== PLAY) return;
      // Übungsmodus endet nie - dort zeigt P den bisherigen Verlauf jederzeit
      // zwischen zwei Stößen (das Spiel läuft danach weiter).
      if (ev.kind === "keydown" && (ev.key === "p" || ev.key === "P") &&
          this.variant === "practice" && this.phase === "aim" && this.rec && this.rec.scenes.length) {
        this.openReplay();
        return;
      }
      // Kamera drehen: rechte Maustaste HALTEN und Maus bewegen (nur Frei-
      // Ansicht) oder Q/E. Loslassen der rechten Taste beendet das Drehen.
      if (this.view === "free") {
        if (ev.kind === "mousedown" && ev.button === 3) {
          this.draggingCam = true;
          this.lastMx = ev.pos[0];
          return;
        }
        if (ev.kind === "mouseup" && ev.button === 3) {
          this.draggingCam = false;
          return;
        }
        if (ev.kind === "keydown" && (ev.key === "q" || ev.key === "Q" || ev.key === "e" || ev.key === "E")) {
          this.draggingCam = false;
          this.camYawT += ev.key === "q" || ev.key === "Q" ? -0.15 : 0.15;
          return;
        }
        if (this.draggingCam) {
          if (ev.kind === "mousemove") {
            const dx = ev.pos[0] - this.lastMx;
            this.lastMx = ev.pos[0];
            this.camYawT += dx * 0.006;
            return;
          }
          // Sicherheitsnetz: bleibt das Loslassen der rechten Taste einmal aus,
          // beendet JEDE andere Aktion (Linksklick, Taste) das Drehen.
          if (ev.kind === "mousedown" || ev.kind === "mouseup" || ev.kind === "keydown") this.draggingCam = false;
        }
      } else if (this.draggingCam) {
        this.draggingCam = false;
      }
      if (!this.humanTurn()) return;
      if (this.phase === "place") {
        this.handlePlace(ev);
        return;
      }
      if (this.phase !== "aim") return;
      this.handleAim(ev);
    }

    handleAim(ev) {
      if (ev.kind === "mousemove") {
        const [mx, my] = this.unproject(ev.pos[0], ev.pos[1]);
        this.aim = Math.atan2(my - this.cue.y, mx - this.cue.x);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.charging = true;
        this.power = 0.05;
      } else if (ev.kind === "mouseup" && ev.button === 1) {
        if (this.charging) {
          this.charging = false;
          this.strike();
        }
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || k === "a" || k === "A") this.aim -= PG.radians(2);
        else if (k === "Right" || k === "d" || k === "D") this.aim += PG.radians(2);
        else if (k === "Up" || k === "w" || k === "W") this.power = Math.min(1.0, this.power + 0.05);
        else if (k === "Down" || k === "s" || k === "S") this.power = Math.max(0.05, this.power - 0.05);
        else if (k === "space" || k === "Return") this.strike();
      }
    }

    handlePlace(ev) {
      if (ev.kind === "mousemove" || ev.kind === "mousedown") {
        let [mx, my] = this.unproject(ev.pos[0], ev.pos[1]);
        mx = Math.max(-HW + BR, Math.min(HW - BR, mx));
        my = Math.max(-HH + BR, Math.min(HH - BR, my));
        if (this.placeFree(mx, my)) {
          this.cue.x = mx;
          this.cue.y = my;
        }
        if (ev.kind === "mousedown" && ev.button === 1 && this.placeFree(this.cue.x, this.cue.y)) {
          this.ballInHand = false;
          this.phase = "aim";
          this.playSound("click");
        }
      }
    }

    placeFree(x, y) {
      for (const b of this.balls) {
        if (b === this.cue || b.potted) continue;
        if ((b.x - x) ** 2 + (b.y - y) ** 2 < (2 * BR) ** 2) return false;
      }
      return true;
    }

    humanTurn() {
      // In der Wiedergabe wird jeder Stoß wie ein eigener gezeigt (samt
      // Ziellinie und Queue) - egal, wer ihn gespielt hat.
      return !!this.rep || this.current === 0;
    }

    // ===================================================== Stoß / Physik
    strike() {
      const sp = MAX_SPEED * this.power;
      this.cue.vx = Math.cos(this.aim) * sp;
      this.cue.vy = Math.sin(this.aim) * sp;
      this.recScene();
      this.phase = "rolling";
      this.shotTime = 0.0;
      this.physAcc = 0.0;
      this.firstHit = null;
      this.cuePotted = false;
      this.pottedShot = [];
      this.playSound("shoot");
      this.rumble(60);
    }

    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      // Kamera weich nachführen
      this.camYaw += (this.camYawT - this.camYaw) * Math.min(1.0, dt * 8);
      if (this.view !== "2d") this.basis = this.camBasis();
      if (this.state !== PLAY) return;
      if (this.phase === "rolling") {
        // Physik in festen 1/60-s-Schritten (siehe PHYS_DT); der Rest darf um
        // einen halben Schritt ins Minus laufen (bei 60 Hz genau 1 Schritt/Frame).
        this.physAcc += dt;
        while (this.physAcc > PHYS_DT * 0.5 && this.phase === "rolling" && this.state === PLAY) {
          this.physAcc -= PHYS_DT;
          this.physics(PHYS_DT);
          this.shotTime += PHYS_DT;
          if (this.rec) this.rec.tick(PHYS_DT, () => this.recSample());
          if (this.allStopped() || this.shotTime > MAX_SHOT_TIME) this.resolveShot();
        }
      } else if (this.phase === "aim") {
        if (this.charging) this.power = Math.min(1.0, this.power + dt * 0.85);
        if (this.current === 1) {
          this.aiDelay -= dt;
          if (this.aiDelay <= 0) this.aiShoot();
        }
      }
    }

    allStopped() {
      for (const b of this.balls) {
        if (!b.potted && speed2(b) > STOP_EPS * STOP_EPS) return false;
      }
      return true;
    }

    physics(dt) {
      // Teilschritte je nach Höchstgeschwindigkeit (kein Durchtunneln)
      let vmax = 0.0;
      for (const b of this.balls) if (!b.potted) vmax = Math.max(vmax, speed2(b));
      vmax = Math.sqrt(vmax);
      const steps = Math.max(3, Math.min(20, Math.trunc((vmax * dt) / BR) + 2));
      const h = dt / steps;
      const fr = Math.max(0.0, 1.0 - FRICTION * h);
      for (let s = 0; s < steps; s++) {
        for (const b of this.balls) {
          if (b.potted) continue;
          b.x += b.vx * h;
          b.y += b.vy * h;
          b.vx *= fr;
          b.vy *= fr;
          if (speed2(b) < STOP_EPS * STOP_EPS) b.vx = b.vy = 0.0;
        }
        this.walls();
        this.collisions();
        this.pockets();
      }
    }

    walls() {
      for (const b of this.balls) {
        if (b.potted) continue;
        if (b.x < -HW + BR) {
          b.x = -HW + BR;
          b.vx = -b.vx * WALL_E;
        } else if (b.x > HW - BR) {
          b.x = HW - BR;
          b.vx = -b.vx * WALL_E;
        }
        if (b.y < -HH + BR) {
          b.y = -HH + BR;
          b.vy = -b.vy * WALL_E;
        } else if (b.y > HH - BR) {
          b.y = HH - BR;
          b.vy = -b.vy * WALL_E;
        }
      }
    }

    collisions() {
      const bs = this.balls.filter((b) => !b.potted);
      const n = bs.length;
      const min2 = (2 * BR) ** 2;
      for (let i = 0; i < n; i++) {
        const a = bs[i];
        for (let j = i + 1; j < n; j++) {
          const c = bs[j];
          let dx = c.x - a.x;
          let dy = c.y - a.y;
          let d2 = dx * dx + dy * dy;
          if (d2 >= min2 || d2 <= 1e-9) {
            if (d2 <= 1e-9) {
              dx = 0.01; dy = 0.0; d2 = 0.0001;
            } else continue;
          }
          const dist = Math.sqrt(d2);
          const nx = dx / dist, ny = dy / dist;
          const overlap = 2 * BR - dist;
          a.x -= (nx * overlap) / 2;
          a.y -= (ny * overlap) / 2;
          c.x += (nx * overlap) / 2;
          c.y += (ny * overlap) / 2;
          const rvx = a.vx - c.vx;
          const rvy = a.vy - c.vy;
          const vn = rvx * nx + rvy * ny;
          if (vn > 0) {
            const jimp = ((1 + BALL_E) * vn) / 2;
            a.vx -= jimp * nx;
            a.vy -= jimp * ny;
            c.vx += jimp * nx;
            c.vy += jimp * ny;
            if (this.firstHit == null && (a.num === 0 || c.num === 0)) {
              this.firstHit = a.num === 0 ? c.num : a.num;
              this.playSound("bounce");
            }
          }
        }
      }
    }

    pockets() {
      for (const b of this.balls) {
        if (b.potted) continue;
        for (const [px, py] of POCKETS) {
          if ((b.x - px) ** 2 + (b.y - py) ** 2 <= POCKET_R ** 2) {
            b.potted = true;
            b.vx = b.vy = 0.0;
            if (b.num === 0) this.cuePotted = true;
            else {
              this.pottedShot.push(b.num);
              this.pottedAll.push(b.num);
            }
            this.playSound("point");
            break;
          }
        }
      }
    }

    // ===================================================== Zugauflösung
    objectBallsLeft(lo = 1, hi = 15) {
      return this.balls.filter((b) => !b.potted && b.num !== 0 && b.num >= lo && b.num <= hi).map((b) => b.num);
    }

    resolveShot() {
      this.breakDone = true;
      // Erst die Aufnahme des Stoßes schließen (der Stand VOR dem Neueinsetzen
      // der Weißen gehört noch zur Sequenz), dann die Regeln.
      if (this.rec) this.rec.close(() => this.recSample());
      this.recRes = null;
      if (this.variant === "practice") this.resolvePractice();
      else if (this.variant === "9ball") this.resolve9ball();
      else this.resolve8ball();
      this.recResult();
      // Weiße neu einsetzen, falls versenkt
      if (this.cuePotted && this.state === PLAY) {
        this.cue.potted = false;
        this.cue.vx = this.cue.vy = 0.0;
        if (!this.ballInHand) this.respotCue();
      }
      if (this.state === PLAY) {
        if (this.ballInHand && !this.humanTurn()) this.aiPlace();
        if (this.ballInHand && this.humanTurn()) {
          this.phase = "place";
        } else {
          this.phase = "aim";
          this.power = 0.35;
          if (this.current === 1) this.aiDelay = 0.7;
        }
      }
    }

    respotCue() {
      for (const x of [-HW * 0.5, -HW * 0.6, -HW * 0.4, -HW * 0.7, 0]) {
        if (this.placeFree(x, 0)) {
          this.cue.x = x;
          this.cue.y = 0;
          return;
        }
      }
      this.cue.x = -HW * 0.5;
      this.cue.y = 0;
    }

    foul(msgKey) {
      this.recRes = msgKey;
      this.msg = t(msgKey);
      this.msgT = 2.2;
      this.ballInHand = true;
      this.current = 1 - this.current;
      this.playSound("hit");
    }

    resolvePractice() {
      // Punkte = alle versenkten Kugeln, auch aus früheren Racks. (Das Original
      // zählte nur das aktuelle Rack - nach dem Neuaufbau fiel die Punktzahl
      // z.B. von 15 auf 1 zurück und der Highscore beim Verlassen ging verloren.)
      this.score = this.rackBase + this.pottedAll.length;
      if (this.cuePotted) this.ballInHand = false; // automatisch neu einsetzen
      if (!this.objectBallsLeft().length) this.newRackKeepScore();
    }

    newRackKeepScore() {
      const sc = this.score;
      this.newRack();
      this.state = PLAY;
      this.score = sc;
      this.rackBase = sc;
      this.msg = t("bil.reracked");
      this.msgT = 2.0;
    }

    resolve8ball() {
      const cur = this.current;
      const opp = 1 - cur;
      const potted = this.pottedShot;
      let foul = this.cuePotted || this.firstHit == null;
      // Falsche zuerst getroffene Kugel = Foul (wenn Gruppe zugewiesen)
      if (this.group[cur] != null && this.firstHit != null) {
        if (!this.groupCleared(cur)) {
          if (!this.inGroup(this.group[cur], this.firstHit)) foul = true;
        } else if (this.firstHit !== 8) {
          foul = true;
        }
      }
      // Schwarze versenkt?
      if (potted.includes(8)) {
        const cleared = this.groupCleared(cur);
        if (this.group[cur] == null || !cleared || this.cuePotted) this.winner = opp;
        else this.winner = cur;
        this.end();
        return;
      }
      // Gruppen zuweisen (nach dem Break, erste saubere versenkte Kugel)
      const obj = potted.filter((p) => p !== 8);
      if (this.group[cur] == null && obj.length && !foul) {
        if (obj[0] <= 7) {
          this.group[cur] = "solid";
          this.group[opp] = "stripe";
        } else {
          this.group[cur] = "stripe";
          this.group[opp] = "solid";
        }
      }
      let madeOwn = false;
      if (this.group[cur] != null) madeOwn = obj.some((p) => this.inGroup(this.group[cur], p));
      else if (obj.length) madeOwn = true; // offener Tisch nach Break
      if (foul) this.foul("bil.foul");
      else if (!madeOwn) this.current = opp; // sonst gleicher Spieler weiter
    }

    inGroup(group, num) {
      return group === "solid" ? num >= 1 && num <= 7 : num >= 9 && num <= 15;
    }

    groupCleared(player) {
      const g = this.group[player];
      if (g == null) return false;
      const [lo, hi] = g === "solid" ? [1, 7] : [9, 15];
      return !this.objectBallsLeft(lo, hi).length;
    }

    resolve9ball() {
      const cur = this.current;
      const opp = 1 - cur;
      const leftBefore = this.lowestBefore();
      let foul = this.cuePotted || this.firstHit == null;
      if (this.firstHit != null && leftBefore != null && this.firstHit !== leftBefore) foul = true;
      if (this.pottedShot.includes(9)) {
        if (!foul) {
          this.winner = cur;
          this.end();
          return;
        }
        // 9 wieder einsetzen
        for (const b of this.balls) if (b.num === 9) b.potted = true;
        this.respot9();
      }
      if (foul) this.foul("bil.foul");
      else if (!this.pottedShot.length) this.current = opp;
    }

    /** niedrigste Kugel, die vor dem Stoß auf dem Tisch war */
    lowestBefore() {
      const nums = this.balls.filter((b) => b.num !== 0 && (!b.potted || this.pottedShot.includes(b.num))).map((b) => b.num);
      return nums.length ? Math.min(...nums) : null;
    }

    respot9() {
      for (const b of this.balls) {
        if (b.num === 9) {
          b.potted = false;
          b.x = HW * 0.45;
          b.y = 0.0;
          if (!this.placeFree(b.x, b.y)) b.x = HW * 0.55;
          const i = this.pottedAll.indexOf(9);
          if (i >= 0) this.pottedAll.splice(i, 1);
        }
      }
    }

    /** KI setzt die Weiße hinter die niedrigste/eigene Zielkugel bzw. in die Mitte */
    aiPlace() {
      const target = this.aiTargetBall();
      let placed = false;
      if (target != null) {
        for (const off of [BR * 5, BR * 8, BR * 12]) {
          const x = target.x - HW * 0.02 - off;
          if (x > -HW + BR && x < HW - BR && this.placeFree(x, target.y)) {
            this.cue.x = x;
            this.cue.y = target.y;
            placed = true;
            break;
          }
        }
      }
      if (!placed) this.respotCue();
      this.ballInHand = false;
    }

    end() {
      this.state = OVER;
      if (this.winner != null) {
        this.wins[this.winner] += 1;
        if (this.winner === 0) {
          this.score = this.wins[0];
          this.playSound("win");
          this.reportResult(true);
        } else {
          this.playSound("gameover");
          this.reportResult(false);
        }
      }
      this.gameOver = true;
    }

    restart() {
      this.gameOver = false;
      this.newRack();
      this.recNew();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Replay-Aufnahme
    // Je Stoß eine Sequenz. Die Kopfdaten halten den kompletten Tisch fest
    // (alle Kugeln mit Nummer, Ort und "versenkt"), die Samples nur noch die
    // Kugeln, die sich seit dem letzten Bild bewegt haben.

    recNew() {
      this.replay = null;
      this.recRes = null;
      this.recDelta.reset();
      this.rec = PG.replay.recorder("billiard", {
        variant: this.variant, view: this.view, diff: this.diff, players: 1,
      });
    }

    recScene() {
      if (!this.rec) return;
      this.recDelta.reset();
      this.rec.scene({
        pl: this.current,
        aim: Math.round(this.aim * 1e4) / 1e4,
        pw: Math.round(this.power * 1e3) / 1e3,
        grp: this.group.slice(),
        pall: this.pottedAll.slice(),
        base: this.rackBase,
        balls: this.balls.map((b) => [b.num, Math.round(b.x * 10) / 10, Math.round(b.y * 10) / 10, b.potted ? 1 : 0]),
      });
    }

    recSample() {
      const out = [];
      this.balls.forEach((b, i) => {
        const st = [Math.round(b.x * 10) / 10, Math.round(b.y * 10) / 10, b.potted ? 1 : 0];
        if (this.recDelta.push(i, st)) out.push(i, st[0], st[1], st[2]);
      });
      return out;
    }

    recResult() {
      if (!this.rec) return;
      let res = this.recRes;
      if (res == null && this.winner != null) res = this.winner === 0 ? "bil.win_you" : "bil.win_ai";
      if (res == null) res = this.pottedShot.length ? "bil.res_pot" : "bil.res_miss";
      this.rec.setLast({
        res, pot: this.pottedShot.slice(), nxt: this.current,
        final: this.winner != null, win: this.winner == null ? -1 : this.winner,
      });
      if (this.state === OVER) this.recFinish();
    }

    recFinish() {
      if (!this.rec) return;
      let sub;
      if (this.variant === "practice") sub = t("bil.potted", { n: this.pottedAll.length });
      else if (this.winner == null) sub = t("bil.var." + this.variant);
      else sub = this.winner === 0 ? t("bil.win_you") : t("bil.win_ai");
      this.replay = this.rec.result({
        title: t("bil.var." + this.variant) + "  ·  " + t("bil.view." + this.view),
        sub, winner: this.winner == null ? -1 : this.winner,
      });
      this.rec = null;
    }

    /** Zwischenstand als Wiederholung (Übungsmodus, der nie endet). */
    recSnapshot() {
      if (!this.rec || !this.rec.scenes.length) return;
      this.replay = this.rec.result({
        title: t("bil.var.practice") + "  ·  " + t("bil.view." + this.view),
        sub: t("bil.potted", { n: this.pottedAll.length }),
      });
    }

    /** Die Wiederholung ansehen (Taste P) - den Screen öffnet app.js. */
    openReplay() {
      if (this.state === PLAY) this.recSnapshot();
      if (this.replay) {
        this.replayRequest = this.replay;
        this.playSound("click");
      }
    }

    // ===================================================== Replay-Wiedergabe
    replayBegin(rep) {
      this.rec = null;
      this.replay = null;
      this.replayRequest = null;
      this.rep = rep;
      this.repAt = null;
      const meta = rep.meta || {};
      if (VARIANTS.includes(meta.variant)) this.variant = meta.variant;
      if (VIEWS.includes(meta.view)) this.view = meta.view;
      this.diff = PG.clamp(Math.trunc(Number(meta.diff)) || 0, 0, 2);
      this.camYaw = this.camYawT = 0.0;
      this.tableCache = null;
      this.setupCamera();
      this.state = PLAY;
      this.gameOver = false;
      this.msg = null;
      this.msgT = 0.0;
      this.winner = null;
      this.wins = [0, 0];
      this.replaySeek(0, 0);
    }

    replaySeek(index, frame) {
      const scenes = this.rep.scenes || [];
      if (!scenes.length) return;
      index = PG.clamp(index, 0, scenes.length - 1);
      const sc = scenes[index];
      const frames = sc.f || [];
      const n = Math.max(1, frames.length);
      frame = PG.clamp(frame, 0, n - 1);
      const last = frame >= n - 1;

      this.current = sc.pl | 0;
      this.aim = Number(sc.aim) || 0;
      this.power = Number(sc.pw) || 0.35;
      this.group = (sc.grp || [null, null]).slice(0, 2);
      this.pottedAll = (sc.pall || []).slice();
      this.rackBase = sc.base | 0;
      this.ballInHand = false;
      this.phase = "rolling";
      this.winner = null;

      // Kugeln: bei einem Sprung neu aufbauen, sonst die Deltas fortschreiben.
      let start;
      if (!this.repAt || this.repAt[0] !== index || frame < this.repAt[1]) {
        this.balls = (sc.balls || []).map(([num, x, y, potted]) => {
          const b = makeBall(x, y, num);
          b.potted = !!potted;
          return b;
        });
        this.cue = this.balls[0] || makeBall(0, 0, 0);
        start = 0;
      } else {
        start = this.repAt[1] + 1;
      }
      for (let k = start; k <= frame; k++) {
        const fr = frames[k];
        for (let j = 0; j + 3 < fr.length; j += 4) {
          const b = this.balls[fr[j] | 0];
          if (b) {
            b.x = fr[j + 1];
            b.y = fr[j + 2];
            b.potted = !!fr[j + 3];
          }
        }
      }
      if (last) {
        this.pottedAll = this.pottedAll.concat(sc.pot || []);
        this.score = this.rackBase + this.pottedAll.length;
        if (sc.res) this.msg = t(sc.res);
        if (sc.final) this.winner = (sc.win | 0) >= 0 ? sc.win | 0 : null;
      } else {
        this.msg = null;
      }
      this.repAt = [index, frame];
    }

    replayDraw(ctx, aiming, banner) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.view !== "2d") this.basis = this.camBasis();
      this.drawTable(ctx);
      if (aiming) {
        // Vorlauf: der Tisch steht, Ziellinie und Queue wie beim Stoß.
        this.phase = "aim";
        this.drawBalls(ctx);
        this.drawAim(ctx);
        this.phase = "rolling";
      } else {
        this.drawBalls(ctx);
      }
      this.drawHud(ctx);
      if (banner && this.winner != null) this.drawOver(ctx);
    }

    // ===================================================== KI
    aiLegalTargets() {
      const live = this.balls.filter((b) => !b.potted);
      if (this.variant === "9ball") {
        const low = this.lowestBefore();
        return live.filter((b) => b.num === low);
      }
      if (this.variant === "practice") return live.filter((b) => b.num !== 0);
      const g = this.group[this.current];
      if (g == null) return live.filter((b) => b.num !== 0 && b.num !== 8);
      if (this.groupCleared(this.current)) return live.filter((b) => b.num === 8);
      return live.filter((b) => this.inGroup(g, b.num));
    }

    aiTargetBall() {
      const ts = this.aiLegalTargets();
      if (!ts.length) return null;
      let best = ts[0], bd = Infinity;
      for (const b of ts) {
        const d = (b.x - this.cue.x) ** 2 + (b.y - this.cue.y) ** 2;
        if (d < bd) {
          bd = d;
          best = b;
        }
      }
      return best;
    }

    aiShoot() {
      let best = null;
      for (const tb of this.aiLegalTargets()) {
        for (const [px, py] of POCKETS) {
          const dpx = px - tb.x, dpy = py - tb.y;
          const dl = Math.hypot(dpx, dpy) || 1.0;
          const ghostX = tb.x - (dpx / dl) * 2 * BR;
          const ghostY = tb.y - (dpy / dl) * 2 * BR;
          const aim = Math.atan2(ghostY - this.cue.y, ghostX - this.cue.x);
          // Schnittwinkel: Cue->Ziel gegen Ziel->Tasche
          const a1 = Math.atan2(tb.y - this.cue.y, tb.x - this.cue.x);
          const cut = Math.abs(PG.mod(aim - a1 + Math.PI, 2 * Math.PI) - Math.PI);
          if (cut > PG.radians(80)) continue;
          if (this.blocked(this.cue.x, this.cue.y, ghostX, ghostY, tb)) continue;
          const dist = Math.hypot(ghostX - this.cue.x, ghostY - this.cue.y) + Math.hypot(dpx, dpy);
          const score = cut * 60 + dist * 0.2;
          if (best == null || score < best[0]) best = [score, aim, dist];
        }
      }
      if (best == null) {
        const tb = this.aiTargetBall();
        if (tb == null) {
          this.aim = PG.rand.uniform(0, PG.TAU);
          this.power = 0.4;
        } else {
          this.aim = Math.atan2(tb.y - this.cue.y, tb.x - this.cue.x);
          this.power = 0.5;
        }
      } else {
        const [, aim, dist] = best;
        const acc = AI_ACC[this.diff];
        this.aim = aim + PG.rand.uniform(-1, 1) * (1 - acc) * 0.28;
        this.power = Math.max(0.3, Math.min(0.92, 0.32 + (dist / (2 * HW)) * 0.6));
      }
      this.strike();
    }

    blocked(x0, y0, x1, y1, ignore) {
      const dx = x1 - x0, dy = y1 - y0;
      const seg2 = dx * dx + dy * dy;
      if (seg2 < 1e-6) return false;
      for (const b of this.balls) {
        if (b === this.cue || b === ignore || b.potted) continue;
        const tt = ((b.x - x0) * dx + (b.y - y0) * dy) / seg2;
        if (tt <= 0.02 || tt >= 0.98) continue;
        const cx = x0 + tt * dx;
        const cy = y0 + tt * dy;
        if ((b.x - cx) ** 2 + (b.y - cy) ** 2 < (2 * BR) ** 2) return true;
      }
      return false;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      if (this.view !== "2d") this.basis = this.camBasis();
      this.drawTable(ctx);
      this.drawBalls(ctx);
      if ((this.phase === "aim" || this.phase === "place") && this.humanTurn()) this.drawAim(ctx);
      this.drawHud(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    corners(e) {
      return [[-HW - e, -HH - e], [HW + e, -HH - e], [HW + e, HH + e], [-HW - e, HH + e]];
    }

    /**
     * Tisch (Bande, Filz, Taschen) als gecachte Offscreen-Canvas - neu gezeichnet
     * nur, wenn Ansicht oder Kamerawinkel sich ändern.
     */
    tableLayer() {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = this.view + ":" + (this.view === "2d" ? "" : this.camYaw.toFixed(4)) + "@" + ps;
      if (this.tableCache && this.tableCache.key === key) return this.tableCache.canvas;
      const c = this.tableCache && this.tableCache.ps === ps ? this.tableCache.canvas : ui.makeCanvas(this.width * ps, this.height * ps);
      const g = c.getContext("2d");
      g.setTransform(ps, 0, 0, ps, 0, 0);
      g.clearRect(0, 0, this.width, this.height);
      const rail = this.corners(BR * 2.4).map(([x, y]) => this.project(x, y).slice(0, 2));
      const bed = this.corners(0).map(([x, y]) => this.project(x, y).slice(0, 2));
      draw.polygon(g, COL_RAIL, rail);
      draw.polygon(g, COL_RAIL_HI, rail, 3);
      draw.polygon(g, COL_CLOTH, bed);
      // leichte Feld-Schattierung
      draw.polygon(g, COL_CLOTH_D, bed, 2);
      for (const [px, py] of POCKETS) {
        const [sx, sy, sc] = this.project(px, py);
        draw.circle(g, COL_POCKET, [sx, sy], Math.max(4, Math.trunc(POCKET_R * sc)));
      }
      this.tableCache = { key, ps, canvas: c };
      return c;
    }

    drawTable(ctx) {
      ctx.drawImage(this.tableLayer(), 0, 0, this.width, this.height);
    }

    drawBalls(ctx) {
      const order = this.balls
        .filter((b) => !b.potted)
        .map((b) => [this.project(b.x, b.y)[3], b])
        .sort((a, b) => b[0] - a[0])
        .map((e) => e[1]);
      const z = this.view !== "2d" ? BR : 0.0;
      for (const b of order) {
        // Schatten
        if (this.view !== "2d") {
          const [sxs, sys, scs] = this.project(b.x, b.y, 0.0);
          draw.ellipse(ctx, [0, 0, 0, 90], [sxs - BR * scs * 1.1, sys - BR * scs * 0.7, Math.trunc(BR * scs * 2.2), Math.trunc(BR * scs * 1.4)]);
        }
        const [sx, sy, sc] = this.project(b.x, b.y, z);
        const r = Math.max(3, Math.trunc(BR * sc));
        this.drawBall(ctx, b, Math.trunc(sx), Math.trunc(sy), r);
      }
      // Weiße beim Platzieren blinkend andeuten
      if (this.phase === "place" && this.humanTurn()) {
        const [sx, sy, sc] = this.project(this.cue.x, this.cue.y, z);
        const k = 0.5 + 0.5 * Math.sin(ui.ticks() / 150.0);
        const v = Math.trunc(120 + 120 * k);
        draw.circle(ctx, [v, v, v], [Math.trunc(sx), Math.trunc(sy)], Math.trunc(BR * sc) + 3, 2);
      }
    }

    drawBall(ctx, b, x, y, r) {
      if (b.num === 0) {
        draw.circle(ctx, COL_CUE, [x, y], r);
      } else if (b.num >= 9 && b.num <= 15) {
        // Halbe: weiße Kugel mit farbigem Streifen (auf Kreis maskiert)
        const base = BALL_COLORS[b.num] || [200, 200, 200];
        draw.circle(ctx, [245, 245, 240], [x, y], r);
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r, 0, PG.TAU);
        ctx.clip();
        const bandH = Math.max(2, r);
        draw.rect(ctx, base, [x - r, y - r + r - (bandH >> 1), 2 * r, bandH]);
        ctx.restore();
      } else {
        draw.circle(ctx, BALL_COLORS[b.num] || [200, 200, 200], [x, y], r);
      }
      // Glanzlicht
      draw.circle(ctx, [255, 255, 255], [x - Math.floor(r / 3), y - Math.floor(r / 3)], Math.max(1, Math.floor(r / 4)));
      if (b.num && r >= 7) {
        const col = b.num !== 8 ? [20, 20, 20] : [235, 235, 235];
        ui.text(ctx, String(b.num), x, y, this.tiny, col, "center");
      }
      draw.circle(ctx, [10, 14, 10], [x, y], r, 1);
    }

    /** Erste getroffene Kugel + Geisterpunkt für die Ziellinie. */
    predict() {
      const ox = Math.cos(this.aim), oy = Math.sin(this.aim);
      let bestT = 1e9;
      let hit = null;
      for (const b of this.balls) {
        if (b === this.cue || b.potted) continue;
        const relx = b.x - this.cue.x, rely = b.y - this.cue.y;
        const proj = relx * ox + rely * oy;
        if (proj <= 0) continue;
        const perp2 = (relx - proj * ox) ** 2 + (rely - proj * oy) ** 2;
        if (perp2 <= (2 * BR) ** 2) {
          const back = Math.sqrt(Math.max(0.0, (2 * BR) ** 2 - perp2));
          const tcontact = proj - back;
          if (tcontact > 0 && tcontact < bestT) {
            bestT = tcontact;
            hit = b;
          }
        }
      }
      // bis zur Bande, falls keine Kugel
      if (hit == null) {
        const tx = ox > 0 ? (HW - BR - this.cue.x) / ox : ox < 0 ? (-HW + BR - this.cue.x) / ox : 1e9;
        const ty = oy > 0 ? (HH - BR - this.cue.y) / oy : oy < 0 ? (-HH + BR - this.cue.y) / oy : 1e9;
        bestT = Math.max(0, Math.min(tx, ty));
      }
      return [this.cue.x + ox * bestT, this.cue.y + oy * bestT, hit];
    }

    drawAim(ctx) {
      if (this.phase === "place") return;
      const [cxp, cyp, hit] = this.predict();
      const cs = this.project(this.cue.x, this.cue.y, 0);
      const ce = this.project(cxp, cyp, 0);
      draw.line(ctx, COL_AIM, cs, ce, 2);
      if (hit != null) {
        draw.circle(ctx, COL_AIM, [Math.trunc(ce[0]), Math.trunc(ce[1])], Math.max(3, Math.trunc(BR * ce[2])), 1);
        // Zielkugel-Richtung
        const tdx = hit.x - cxp, tdy = hit.y - cyp;
        const tl = Math.hypot(tdx, tdy) || 1.0;
        const hs = this.project(hit.x, hit.y, 0);
        const he = this.project(hit.x + (tdx / tl) * 6 * BR, hit.y + (tdy / tl) * 6 * BR, 0);
        draw.line(ctx, [200, 220, 255], hs, he, 2);
      }
      // Queue-Stock (entgegengesetzt zur Zielrichtung)
      const ox = Math.cos(this.aim), oy = Math.sin(this.aim);
      const pull = 4 + this.power * 16;
      const bs2 = this.project(this.cue.x - ox * (BR + pull), this.cue.y - oy * (BR + pull), 0);
      const es2 = this.project(this.cue.x - ox * (BR + pull + 70), this.cue.y - oy * (BR + pull + 70), 0);
      draw.line(ctx, COL_STICK, bs2, es2, 5);
      draw.line(ctx, COL_STICK_D, bs2, es2, 1);
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH], [this.width, this.hudH]);
      const cy = Math.floor(this.hudH / 2);
      // Ansicht + Variante links
      ui.text(ctx, t("bil.view." + this.view) + "  ·  " + t("bil.var." + this.variant), 12, cy, this.tiny, ui.TEXT_DIM, "midleft");
      // Kraft-Meter rechts
      const mw = 90, mh = 10;
      const mx = this.width - mw - 14;
      draw.rect(ctx, ui.BTN, [mx, cy - (mh >> 1), mw, mh], 0, 4);
      draw.rect(ctx, this.accent, [mx, cy - (mh >> 1), Math.trunc(mw * this.power), mh], 0, 4);
      // Mitte: wer ist dran / Gruppe / Nachricht
      let mid;
      if (this.msg) mid = this.msg;
      else if (this.variant === "practice") mid = t("bil.potted", { n: this.pottedAll.length });
      else if (this.phase === "place") mid = t("bil.place_cue");
      else if (this.current === 1) mid = t("bil.ai_turn");
      else {
        mid = t("bil.you");
        const g = this.group[this.current];
        if (g) mid += " (" + t("bil.grp." + g) + ")";
      }
      ui.text(ctx, mid, Math.floor(this.width / 2), cy, this.small, this.accent, "center");
    }

    drawOver(ctx) {
      const y = Math.floor(this.height / 2) - 50;
      draw.rect(ctx, [8, 12, 10, 210], [0, y, this.width, 100]);
      draw.line(ctx, this.accent, [0, y + 0.5], [this.width, y + 0.5]);
      draw.line(ctx, this.accent, [0, y + 99.5], [this.width, y + 99.5]);
      const cx = Math.floor(this.width / 2);
      if (this.variant === "practice") {
        ui.text(ctx, t("bil.potted", { n: this.pottedAll.length }), cx, y + 36, this.huge, this.accent, "center");
      } else {
        const won = this.winner === 0;
        ui.text(ctx, won ? t("bil.win_you") : t("bil.win_ai"), cx, y + 36, this.huge, won ? this.accent : ui.TEXT_DIM, "center");
      }
      let hint = t("bil.new_round");
      if (this.replay && !this.rep) hint += "  ·  " + t("bil.replay_hint");
      ui.text(ctx, hint, cx, y + 76, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("bil.title"), cx, Math.floor(this.height * 0.12), this.huge, this.accent, "center");
      ui.text(ctx, t("bil.subtitle"), cx, Math.floor(this.height * 0.185), this.small, ui.TEXT_DIM, "center");

      const label = (rects, txt) => ui.text(ctx, txt, cx, rects[0].top - 4, this.tiny, ui.TEXT_DIM, "midbottom");

      label(this.varRects, t("bil.lbl_variant"));
      this.varRects.forEach((rc, i) => this.btn(ctx, rc, t("bil.var." + VARIANTS[i]), this.variant === VARIANTS[i]));
      label(this.viewRects, t("bil.lbl_view"));
      this.viewRects.forEach((rc, i) => this.btn(ctx, rc, t("bil.view." + VIEWS[i]), this.view === VIEWS[i]));
      label(this.diffRects, t("bil.lbl_diff"));
      this.diffRects.forEach((rc, i) => this.btn(ctx, rc, t("bil.diff." + DIFFS[i]), this.diff === i));
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("bil.setup_hint"), cx, this.height - 14, this.tiny, ui.TEXT_DIM, "center");
    }

    btn(ctx, rc, text, on) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      ui.text(ctx, text, rc.centerx, rc.centery, this.small, on ? ui.TEXT : ui.TEXT_DIM, "center");
    }
  }

  PG.register(BilliardGame, {
    id: "BilliardGame",
    key: "billiard",
    name: { default: "Billiards", de: "Billard", fr: "Billard", es: "Billar", pt: "Bilhar" },
    settingsKey: "billiard",
    defaults: { variant: "8ball", view: "2d", difficulty: 1 },
    wantsRightClick: true,
  });
})();
