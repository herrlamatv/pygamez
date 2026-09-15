/*
 * asteroids.js - Asteroids, Einzelspieler (Port von games/asteroids.py)
 * ======================================================================
 * - Klassische Vektor-Optik: Dreieck-Schiff mit Schubflamme, zackige
 *   Polygon-Brocken (eigene Zufallsform, rotierend), Sternenhimmel.
 * - Trägheitsphysik: Hoch = Schub, Links/Rechts = drehen, leichte Dämpfung,
 *   Tempolimit, Wrap-Around über alle Ränder.
 * - Brocken zerspringen in zwei kleinere (3 Größen; 20/50/100 Punkte),
 *   Wellen mit steigender Anzahl und Banner.
 * - UFO (abschaltbar) kreuzt den Schirm und zielt aufs Schiff - 200 Punkte.
 * - Power-Ups (abschaltbar): S = Schild, T = Dreifachschuss, R = Schnellfeuer.
 * - Hyperraum (Runter): Zufallssprung, 4 s Abklingzeit, 12 % Risiko.
 * - 3 Leben, sicheres Respawnen, Extraleben alle 5000 Punkte,
 *   Explosions-Partikel und Kamera-Shake.
 * - Setup: Schwierigkeit, UFOs an/aus, Power-Ups an/aus (gespeichert).
 * Der Koop-Duell-Modus der Desktop-Version entfällt in der Web-Version.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const R = PG.rand;

  // Identitätsfarben der Vektor-Optik - bewusst NICHT ans Theme gekoppelt.
  // Das Spielfeld ist immer dunkler Weltraum; der Setup-Screen nutzt ui.*.
  const COL_BG = [8, 10, 20];
  const COL_TEXT = [232, 234, 240];
  const COL_DIM = [140, 148, 168];
  const COL_OK = [120, 200, 150];   // "bereit"-Grün auf dunklem Feld
  const COL_BAD = [245, 110, 110];  // Game-Over-Rot auf dunklem Feld
  const COL_ROCK = [205, 210, 225];
  const COL_ROCK_FILL = [26, 30, 44];
  const COL_P1 = [120, 230, 160];
  const COL_BULLET = [245, 245, 250];
  const COL_UFO = [240, 150, 90];
  const COL_UFO_SHOT = [250, 110, 110];

  const SHIP_R = 12;            // Kollisionsradius des Schiffs
  const SHIP_TURN = 3.9;        // Drehgeschwindigkeit (rad/s)
  const SHIP_THRUST = 300.0;    // Schub (Pixel/s^2)
  const SHIP_DAMP = 0.35;       // Dämpfung pro Sekunde (leichtes Ausrollen)
  const SHIP_MAX = 430.0;       // Tempolimit
  const INVULN_T = 2.5;         // Unverwundbarkeit nach dem Respawn
  const RESPAWN_T = 1.8;        // Wartezeit bis zum Respawn
  const HYPER_CD = 4.0;         // Abklingzeit Hyperraum
  const HYPER_RISK = 0.12;      // Risiko, beim Sprung zu zerschellen
  const EXTRA_LIFE_EVERY = 5000; // Punkte bis zum nächsten Extraleben

  const BULLET_SPEED = 500.0;
  const BULLET_LIFE = 0.95;
  const SHOT_CD = 0.26;         // Feuerpause (Schnellfeuer: 0.12)
  const MAX_BULLETS = 4;        // gleichzeitige Schüsse (Schnellfeuer: 8)

  // Brocken: Größe -> (Radius, Grundtempo min/max, Punkte)
  const ROCKS = { 3: [44, 40, 90, 20], 2: [26, 60, 130, 50], 1: [14, 90, 180, 100] };

  const UFO_R = 16;
  const UFO_SHOT_SPEED = 320.0;
  const UFO_POINTS = 200;

  const POWERUP_KINDS = [
    ["shield", "S", [110, 220, 220], 6.0],
    ["triple", "T", [180, 140, 255], 8.0],
    ["rapid", "R", [245, 205, 90], 8.0],
  ];
  const POWERUP_DROP = 0.18;    // Drop-Chance je zerstörtem Brocken
  const POWERUP_LIFE = 9.0;     // so lange liegt ein Power-Up herum

  // Schwierigkeit: Brocken zu Wellenstart, Tempofaktor, UFO-Takt, UFO-Zielfehler
  const DIFFS = [
    { key: "easy", rocks: 3, speed: 0.75, ufoEvery: 30.0, ufoAim: 0.5 },
    { key: "medium", rocks: 4, speed: 1.0, ufoEvery: 24.0, ufoAim: 0.28 },
    { key: "hard", rocks: 5, speed: 1.3, ufoEvery: 18.0, ufoAim: 0.12 },
  ];

  const SETUP = "setup", PLAY = "play";

  /** Zustand des Schiffs (Position, Drift, Leben, Power-Ups). */
  function makeShip(x, y, color) {
    return {
      home: [x, y],               // Respawn-Punkt
      color,
      x, y, vx: 0.0, vy: 0.0,
      angle: -Math.PI / 2,        // Blick nach oben
      lives: 3,
      score: 0,
      nextExtra: EXTRA_LIFE_EVERY,
      alive: true,
      invuln: INVULN_T,
      respawn: 0.0,
      cool: 0.0,                  // Feuerpause
      hyperCd: 0.0,
      powers: {},                 // "shield"/"triple"/"rapid" -> Restzeit
      thrusting: false,
    };
  }

  class AsteroidsGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;

      const a = this.opts;
      this.diff = Math.max(0, Math.min(2, parseInt(a.difficulty, 10) || 0));
      this.ufosOn = !!a.ufos;
      this.powerupsOn = !!a.powerups;

      this.makeFonts();
      this.animT = 0.0;

      this.makeStars();

      this.buildSetupLayout();
      this.state = SETUP;
      this.startRun();
    }

    /** Themen-Schriften, Größen aus der Fensterhöhe abgeleitet. */
    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(16, Math.min(26, Math.floor(h * 0.04))));
      this.bigFont = ui.font(Math.max(30, Math.min(56, Math.floor(h * 0.088))), true);
      this.fSmall = ui.font(Math.max(13, Math.min(20, Math.floor(h * 0.03))));
      this.fTiny = ui.font(Math.max(11, Math.min(17, Math.floor(h * 0.024))));
      // Monospace für laufende Timer, damit die Anzeige nicht "zappelt"
      this.fTinyMono = ui.font(Math.max(11, Math.min(17, Math.floor(h * 0.024))), false, true);
    }

    /** Sternenhimmel (statisch, zwei Helligkeiten) neu auswürfeln. */
    makeStars() {
      this.stars = [];
      for (let i = 0; i < 70; i++) {
        this.stars.push([R.randrange(this.width), R.randrange(this.height), R.choice([1, 1, 2]), R.randint(60, 150)]);
      }
    }

    startRun() {
      this.score = 0;
      this.gameOver = false;
      this.ships = [makeShip(this.width / 2, this.height / 2, COL_P1)];

      this.rocks = [];         // x,y,vx,vy,size,r,shape,rot,spin
      this.bullets = [];       // x,y,vx,vy,life,owner
      this.ufoShots = [];      // x,y,vx,vy,life
      this.items = [];         // Power-Ups: x,y,vx,vy,kind,life
      this.particles = [];
      this.ufo = null;
      const d = DIFFS[this.diff];
      this.ufoTimer = d.ufoEvery * R.uniform(0.7, 1.1);

      this.wave = 1;
      this.bannerT = 1.2;
      this.pendingWave = true;
      this.flashMsg = null;    // [text, farbe, restzeit]
      this.shake = 0.0;
      this.ending = 0.0;       // Nachlauf zwischen letztem Tod und Game Over

      this.pressed = new Set();
      this.hintT = 0.0;        // zählt nur echte Spielzeit für den Steuerungs-Hinweis
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(420, this.width - 60);
      const top = Math.max(108, Math.floor(this.height * 0.22));
      this.diffPanel = new PG.Rect(cx - Math.floor(bw / 2), top, bw, 56);
      this.diffLeft = new PG.Rect(this.diffPanel.left, top, 40, 56);
      this.diffRight = new PG.Rect(this.diffPanel.right - 40, top, 40, 56);
      const bh = 42, gap = 10;
      const y0 = top + 78;
      this.ufoRect = new PG.Rect(cx - Math.floor(bw / 2), y0, bw, bh);
      this.powerRect = new PG.Rect(cx - Math.floor(bw / 2), y0 + (bh + gap), bw, bh);
      this.startRect = new PG.Rect(cx - 95, y0 + 2 * (bh + gap) + 8, 190, 50);
    }

    cycleDiff(step) {
      this.diff = PG.mod(this.diff + step, DIFFS.length);
      this.opts.difficulty = this.diff;
      this.saveSettings();
      this.playSound("click");
    }

    toggleUfos() {
      this.ufosOn = !this.ufosOn;
      this.opts.ufos = this.ufosOn;
      this.saveSettings();
      this.playSound("select");
    }

    togglePowerups() {
      this.powerupsOn = !this.powerupsOn;
      this.opts.powerups = this.powerupsOn;
      this.saveSettings();
      this.playSound("select");
    }

    startPlay() {
      this.startRun();
      this.state = PLAY;
      this.playSound("click");
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || k === "a" || k === "A") this.cycleDiff(-1);
        else if (k === "Right" || k === "d" || k === "D") this.cycleDiff(+1);
        else if (k === "u" || k === "U") this.toggleUfos();
        else if (k === "p" || k === "P") this.togglePowerups();
        else if (k === "Return" || k === "space") this.startPlay();
      } else if (ev.kind === "mousedown" && ev.pos) {
        const p = ev.pos;
        if (this.diffLeft.collidepoint(p)) this.cycleDiff(-1);
        else if (this.diffRight.collidepoint(p) || this.diffPanel.collidepoint(p)) this.cycleDiff(+1);
        else if (this.ufoRect.collidepoint(p)) this.toggleUfos();
        else if (this.powerRect.collidepoint(p)) this.togglePowerups();
        else if (this.startRect.collidepoint(p)) this.startPlay();
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetupEvent(ev);
        return;
      }

      // Einzelspieler: beide Belegungen (WASD+Leertaste / Pfeile+Enter) steuern das Schiff
      if (ev.kind === "keyup") {
        for (const act of ["left", "right", "up"]) if (this.isAction(ev.key, act)) this.pressed.delete(act);
        return;
      }

      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.startPlay();
        else if (ev.key === "s" || ev.key === "S") {
          // Web: Game-Over-Flag zurücknehmen, sonst liegt das Highscore-Banner der App über dem Setup
          this.gameOver = false;
          this.state = SETUP;
          this.playSound("click");
        }
        return;
      }

      const ship = this.ships[0];
      for (const act of ["left", "right", "up"]) if (this.isAction(ev.key, act)) this.pressed.add(act);
      if (this.isAction(ev.key, "action")) this.tryShoot(ship);
      if (this.isAction(ev.key, "down")) this.hyperspace(ship);
    }

    // ----- Aktionen -----------------------------------------------------
    tryShoot(ship) {
      if (!ship.alive || ship.cool > 0) return;
      const maxB = "rapid" in ship.powers ? 8 : MAX_BULLETS;
      const eigene = this.bullets.filter((b) => b.owner === ship).length;
      if (eigene >= maxB) return;
      let winkel = [ship.angle];
      if ("triple" in ship.powers) winkel = [ship.angle - 0.26, ship.angle, ship.angle + 0.26];
      for (const a of winkel) {
        this.bullets.push({
          x: ship.x + Math.cos(a) * 14, y: ship.y + Math.sin(a) * 14,
          vx: Math.cos(a) * BULLET_SPEED + ship.vx,
          vy: Math.sin(a) * BULLET_SPEED + ship.vy,
          life: BULLET_LIFE, owner: ship,
        });
      }
      ship.cool = "rapid" in ship.powers ? 0.12 : SHOT_CD;
      this.playSound("shoot");
    }

    /** Notsprung an eine Zufallsposition - mit Restrisiko. */
    hyperspace(ship) {
      if (!ship.alive || ship.hyperCd > 0) return;
      this.spawnParticles(ship.x, ship.y, ship.color, 10);
      ship.x = R.uniform(40, this.width - 40);
      ship.y = R.uniform(40, this.height - 40);
      ship.vx *= 0.25;
      ship.vy *= 0.25;
      ship.hyperCd = HYPER_CD;
      this.playSound("rotate");
      if (R.random() < HYPER_RISK) this.killShip(ship);
      else this.spawnParticles(ship.x, ship.y, ship.color, 10);
    }

    // ===================================================== Spiellogik
    update(dt) {
      this.animT += dt;
      this.updateParticles(dt);
      if (this.shake > 0) this.shake = Math.max(0.0, this.shake - dt * 1.6);
      if (this.state !== PLAY || this.gameOver) return;
      this.hintT += dt;

      if (this.flashMsg !== null) {
        const [text, farbe, rest] = this.flashMsg;
        this.flashMsg = rest > dt ? [text, farbe, rest - dt] : null;
      }

      // Wellen-Banner / nächste Welle spawnen
      if (this.bannerT > 0) {
        this.bannerT -= dt;
        if (this.bannerT <= 0 && this.pendingWave) {
          this.spawnWave();
          this.pendingWave = false;
        }
      }

      this.updateShips(dt);
      this.updateBullets(dt);
      for (const r of this.rocks) {
        [r.x, r.y] = this.wrap(r.x + r.vx * dt, r.y + r.vy * dt, r.r);
        r.rot += r.spin * dt;
      }
      this.updateItems(dt);
      if (this.ufosOn) this.updateUfo(dt);
      this.collisions();

      // Welle geschafft?
      if (!this.rocks.length && !this.pendingWave && this.ufo === null && this.bannerT <= 0 && this.ending <= 0) {
        this.wave += 1;
        this.bannerT = 2.0;
        this.pendingWave = true;
        this.playSound("level");
      }

      // Nachlauf nach dem letzten Tod, dann Game Over
      if (this.ending > 0) {
        this.ending -= dt;
        if (this.ending <= 0) this.finish();
      }
    }

    /** Wickelt eine Position mit Rand m über die Bildschirmkanten. */
    wrap(x, y, m) {
      if (x < -m) x += this.width + 2 * m;
      else if (x > this.width + m) x -= this.width + 2 * m;
      if (y < -m) y += this.height + 2 * m;
      else if (y > this.height + m) y -= this.height + 2 * m;
      return [x, y];
    }

    // ----- Schiff ----------------------------------------------------------
    updateShips(dt) {
      for (const sh of this.ships) {
        sh.cool = Math.max(0.0, sh.cool - dt);
        sh.hyperCd = Math.max(0.0, sh.hyperCd - dt);
        for (const k of Object.keys(sh.powers)) {
          sh.powers[k] -= dt;
          if (sh.powers[k] <= 0) delete sh.powers[k];
        }

        if (!sh.alive) {
          // Respawn, sobald die Wartezeit um und der Startpunkt frei ist
          if (sh.lives > 0) {
            sh.respawn -= dt;
            if (sh.respawn <= 0 && this.areaClear(sh.home, 130)) {
              [sh.x, sh.y] = sh.home;
              sh.vx = sh.vy = 0.0;
              sh.angle = -Math.PI / 2;
              sh.alive = true;
              sh.invuln = INVULN_T;
            }
          }
          continue;
        }

        sh.invuln = Math.max(0.0, sh.invuln - dt);
        const tasten = this.pressed;
        if (tasten.has("left")) sh.angle -= SHIP_TURN * dt;
        if (tasten.has("right")) sh.angle += SHIP_TURN * dt;
        sh.thrusting = tasten.has("up");
        if (sh.thrusting) {
          sh.vx += Math.cos(sh.angle) * SHIP_THRUST * dt;
          sh.vy += Math.sin(sh.angle) * SHIP_THRUST * dt;
          if (R.random() < dt * 40) {
            // Flammen-Partikel nach hinten
            const ba = sh.angle + Math.PI + R.uniform(-0.4, 0.4);
            this.particles.push([sh.x + Math.cos(ba) * 12, sh.y + Math.sin(ba) * 12,
              Math.cos(ba) * 90 + sh.vx * 0.4, Math.sin(ba) * 90 + sh.vy * 0.4,
              R.uniform(0.15, 0.3), [250, 180, 90]]);
          }
        }
        // Dämpfung + Tempolimit
        const f = Math.max(0.0, 1.0 - SHIP_DAMP * dt);
        sh.vx *= f;
        sh.vy *= f;
        const sp = Math.hypot(sh.vx, sh.vy);
        if (sp > SHIP_MAX) {
          sh.vx *= SHIP_MAX / sp;
          sh.vy *= SHIP_MAX / sp;
        }
        [sh.x, sh.y] = this.wrap(sh.x + sh.vx * dt, sh.y + sh.vy * dt, SHIP_R);
      }
    }

    areaClear(pos, radius) {
      return this.rocks.every((r) => Math.hypot(r.x - pos[0], r.y - pos[1]) > radius + r.r);
    }

    killShip(ship) {
      if (!ship.alive) return;
      ship.alive = false;
      ship.lives -= 1;
      ship.respawn = RESPAWN_T;
      ship.powers = {};
      this.shake = 0.55;
      this.spawnParticles(ship.x, ship.y, ship.color, 26);
      this.playSound("hit");
      this.playSound("explode");
      this.rumble(220);
      if (this.ships.every((s) => s.lives <= 0 && !s.alive)) this.ending = 1.4; // Explosion zu Ende spielen lassen
    }

    finish() {
      this.gameOver = true;
      this.score = Math.max(...this.ships.map((s) => s.score));
      this.playSound("gameover");
      this.rumble(250);
    }

    // ----- Punkte / Extraleben ----------------------------------------------
    addPoints(ship, pts) {
      ship.score += pts;
      this.score = Math.max(...this.ships.map((s) => s.score));
      if (ship.score >= ship.nextExtra) {
        ship.nextExtra += EXTRA_LIFE_EVERY;
        ship.lives += 1;
        this.flashMsg = [t("ast.extra_life"), ship.color, 2.0];
        this.playSound("powerup");
      }
    }

    // ----- Brocken -----------------------------------------------------------
    spawnWave() {
      const d = DIFFS[this.diff];
      const anzahl = Math.min(10, d.rocks + this.wave - 1);
      for (let i = 0; i < anzahl; i++) this.spawnRock(3);
    }

    spawnRock(size, pos = null) {
      const [r, v0, v1] = ROCKS[size];
      if (pos === null) {
        // Am Rand spawnen, mit Abstand zu allen Schiffen
        let x = 0, y = 0;
        for (let i = 0; i < 60; i++) {
          if (R.random() < 0.5) {
            x = R.choice([-r, this.width + r]);
            y = R.uniform(0, this.height);
          } else {
            x = R.uniform(0, this.width);
            y = R.choice([-r, this.height + r]);
          }
          if (this.ships.every((s) => Math.hypot(x - s.x, y - s.y) > 150)) break;
        }
        pos = [x, y];
      }
      const ang = R.uniform(0, PG.TAU);
      const spd = R.uniform(v0, v1) * DIFFS[this.diff].speed;
      const shape = [];
      for (let i = 0; i < 11; i++) shape.push(R.uniform(0.72, 1.12));
      this.rocks.push({
        x: pos[0], y: pos[1],
        vx: Math.cos(ang) * spd, vy: Math.sin(ang) * spd,
        size, r, shape,
        rot: R.uniform(0, PG.TAU),
        spin: R.uniform(-1.6, 1.6),
      });
    }

    /** Brocken zerstören: Punkte, Kinder, Partikel, evtl. Power-Up. */
    breakRock(rock, ship) {
      const pts = ROCKS[rock.size][3];
      if (ship) this.addPoints(ship, pts);
      const i = this.rocks.indexOf(rock);
      if (i >= 0) this.rocks.splice(i, 1);
      this.spawnParticles(rock.x, rock.y, [200, 205, 220], 6 + rock.size * 4);
      this.playSound(rock.size === 3 ? "explode" : "hit");
      if (rock.size > 1) {
        for (let k = 0; k < 2; k++) this.spawnRock(rock.size - 1, [rock.x, rock.y]);
      }
      if (this.powerupsOn && this.items.length < 2 && R.random() < POWERUP_DROP) {
        const kind = R.randrange(POWERUP_KINDS.length);
        const ang = R.uniform(0, PG.TAU);
        this.items.push({ x: rock.x, y: rock.y, vx: Math.cos(ang) * 30, vy: Math.sin(ang) * 30, kind, life: POWERUP_LIFE });
      }
    }

    // ----- Schüsse / Power-Ups / UFO -----------------------------------------
    updateBullets(dt) {
      const step = (list) => {
        const rest = [];
        for (const b of list) {
          b.life -= dt;
          if (b.life <= 0) continue;
          [b.x, b.y] = this.wrap(b.x + b.vx * dt, b.y + b.vy * dt, 4);
          rest.push(b);
        }
        return rest;
      };
      this.bullets = step(this.bullets);
      this.ufoShots = step(this.ufoShots);
    }

    updateItems(dt) {
      const rest = [];
      for (const it of this.items) {
        it.life -= dt;
        if (it.life <= 0) continue;
        [it.x, it.y] = this.wrap(it.x + it.vx * dt, it.y + it.vy * dt, 14);
        rest.push(it);
      }
      this.items = rest;
    }

    updateUfo(dt) {
      if (this.ufo === null) {
        this.ufoTimer -= dt;
        if (this.ufoTimer <= 0 && this.bannerT <= 0) {
          const richtung = R.choice([-1, 1]);
          const baseY = R.uniform(60, this.height - 60);
          this.ufo = {
            x: richtung > 0 ? -UFO_R : this.width + UFO_R,
            y: baseY, baseY,
            dir: richtung, t: 0.0, shoot: R.uniform(0.8, 1.4),
          };
          this.playSound("move");
        }
        return;
      }
      const u = this.ufo;
      u.t += dt;
      u.x += u.dir * (95 + this.wave * 4) * dt;
      u.y = u.baseY + Math.sin(u.t * 2.2) * 40;
      // verschwindet am anderen Rand
      if ((u.dir > 0 && u.x > this.width + UFO_R) || (u.dir < 0 && u.x < -UFO_R)) {
        this.ufo = null;
        this.ufoTimer = DIFFS[this.diff].ufoEvery * R.uniform(0.8, 1.2);
        return;
      }
      // gezielter Schuss auf das lebende Schiff
      u.shoot -= dt;
      if (u.shoot <= 0) {
        u.shoot = 1.3;
        const ziele = this.ships.filter((s) => s.alive);
        if (ziele.length) {
          const z = R.choice(ziele);
          let a = Math.atan2(z.y - u.y, z.x - u.x);
          a += R.uniform(-1, 1) * DIFFS[this.diff].ufoAim;
          this.ufoShots.push({ x: u.x, y: u.y, vx: Math.cos(a) * UFO_SHOT_SPEED, vy: Math.sin(a) * UFO_SHOT_SPEED, life: 1.6 });
          this.playSound("shoot");
        }
      }
    }

    // ----- Kollisionen ---------------------------------------------------------
    collisions() {
      // Spielerschüsse gegen Brocken / UFO
      for (const b of this.bullets.slice()) {
        let getroffen = null;
        for (const r of this.rocks) {
          if (Math.hypot(b.x - r.x, b.y - r.y) < r.r) {
            getroffen = r;
            break;
          }
        }
        if (getroffen !== null) {
          this.removeFrom(this.bullets, b);
          this.breakRock(getroffen, b.owner);
          continue;
        }
        if (this.ufo !== null && Math.hypot(b.x - this.ufo.x, b.y - this.ufo.y) < UFO_R + 2) {
          this.removeFrom(this.bullets, b);
          this.spawnParticles(this.ufo.x, this.ufo.y, COL_UFO, 18);
          this.addPoints(b.owner, UFO_POINTS);
          this.ufo = null;
          this.ufoTimer = DIFFS[this.diff].ufoEvery * R.uniform(0.9, 1.3);
          this.playSound("point");
          this.playSound("explode");
        }
      }

      for (const sh of this.ships) {
        if (!sh.alive) continue;
        const verwundbar = sh.invuln <= 0 && !("shield" in sh.powers);
        // Brocken
        if (verwundbar) {
          for (const r of this.rocks) {
            if (Math.hypot(sh.x - r.x, sh.y - r.y) < r.r * 0.9 + SHIP_R) {
              this.killShip(sh);
              break;
            }
          }
        }
        if (!sh.alive) continue;
        // UFO-Schüsse und UFO-Rumpf
        if (verwundbar) {
          for (const b of this.ufoShots.slice()) {
            if (Math.hypot(sh.x - b.x, sh.y - b.y) < SHIP_R + 3) {
              this.removeFrom(this.ufoShots, b);
              this.killShip(sh);
              break;
            }
          }
        }
        if (!sh.alive) continue;
        if (verwundbar && this.ufo !== null && Math.hypot(sh.x - this.ufo.x, sh.y - this.ufo.y) < UFO_R + SHIP_R) {
          this.killShip(sh);
          continue;
        }
        // Power-Ups einsammeln
        for (const it of this.items.slice()) {
          if (Math.hypot(sh.x - it.x, sh.y - it.y) < SHIP_R + 14) {
            const [kind, , farbe, dauer] = POWERUP_KINDS[it.kind];
            sh.powers[kind] = dauer;
            this.removeFrom(this.items, it);
            this.spawnParticles(it.x, it.y, farbe, 10);
            this.playSound("powerup");
          }
        }
      }
    }

    removeFrom(list, item) {
      const i = list.indexOf(item);
      if (i >= 0) list.splice(i, 1);
    }

    // ----- Partikel -------------------------------------------------------------
    spawnParticles(x, y, color, n) {
      for (let i = 0; i < n; i++) {
        const ang = R.uniform(0, PG.TAU);
        const spd = R.uniform(40, 260);
        this.particles.push([x, y, Math.cos(ang) * spd, Math.sin(ang) * spd, R.uniform(0.25, 0.7), color]);
      }
    }

    updateParticles(dt) {
      const rest = [];
      for (const p of this.particles) {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[2] *= 0.985;
        p[3] *= 0.985;
        p[4] -= dt;
        if (p[4] > 0) rest.push(p);
      }
      this.particles = rest;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }

      const s = ctx;
      draw.rect(s, COL_BG, [0, 0, this.width, this.height]);
      let ox = 0, oy = 0;
      if (this.shake > 0) {
        const amp = 10 * this.shake;
        ox = R.uniform(-amp, amp);
        oy = R.uniform(-amp, amp);
      }

      for (const [x, y, gr, hell] of this.stars) {
        const tw = hell + Math.floor(30 * Math.sin(this.animT * 1.5 + x));
        const c = Math.max(40, Math.min(200, tw));
        draw.circle(s, [c, c, Math.min(255, c + 25)], [x, y], gr);
      }

      for (const p of this.particles) {
        const a = Math.max(0.0, Math.min(1.0, p[4] / 0.7));
        const col = [0, 1, 2].map((i) => Math.floor(p[5][i] * a + COL_BG[i] * (1 - a)));
        draw.circle(s, col, [p[0] + ox, p[1] + oy], 2);
      }

      for (const r of this.rocks) this.drawRock(s, r, ox, oy);
      for (const it of this.items) this.drawItem(s, it, ox, oy);
      if (this.ufo !== null) this.drawUfo(s, ox, oy);
      for (const b of this.bullets) draw.circle(s, COL_BULLET, [b.x + ox, b.y + oy], 2);
      for (const b of this.ufoShots) draw.circle(s, COL_UFO_SHOT, [b.x + ox, b.y + oy], 3);
      for (const sh of this.ships) this.drawShip(s, sh, ox, oy);

      this.drawHud(s);
      if (this.gameOver) this.drawGameOver(s);
    }

    drawRock(s, r, ox, oy) {
      const n = r.shape.length;
      const pts = r.shape.map((f, i) => {
        const a = r.rot + (i * PG.TAU) / n;
        return [r.x + Math.cos(a) * r.r * f + ox, r.y + Math.sin(a) * r.r * f + oy];
      });
      draw.polygon(s, COL_ROCK_FILL, pts);
      draw.polygon(s, COL_ROCK, pts, 2);
    }

    drawShip(s, sh, ox, oy) {
      if (!sh.alive) return;
      // Unverwundbar: blinken
      if (sh.invuln > 0 && Math.floor(this.animT * 12) % 2 === 0) return;
      const a = sh.angle;
      const x = sh.x + ox, y = sh.y + oy;
      const nase = [x + Math.cos(a) * 16, y + Math.sin(a) * 16];
      const hl = [x + Math.cos(a + 2.5) * 13, y + Math.sin(a + 2.5) * 13];
      const hr = [x + Math.cos(a - 2.5) * 13, y + Math.sin(a - 2.5) * 13];
      const heck = [x + Math.cos(a + Math.PI) * 6, y + Math.sin(a + Math.PI) * 6];
      const dunkel = sh.color.map((c) => Math.floor(c / 3));
      draw.polygon(s, dunkel, [nase, hl, heck, hr]);
      draw.polygon(s, sh.color, [nase, hl, heck, hr], 2);
      // Schubflamme (flackert)
      if (sh.thrusting && R.random() < 0.85) {
        const fl = 10 + R.uniform(0, 8);
        const fx = [x + Math.cos(a + Math.PI) * (8 + fl), y + Math.sin(a + Math.PI) * (8 + fl)];
        draw.polygon(s, [250, 180, 90], [hl, fx, hr]);
      }
      if ("shield" in sh.powers) {
        const r = 20 + Math.trunc(2 * Math.sin(this.animT * 8));
        draw.circle(s, [110, 220, 220], [x, y], r, 2);
      }
    }

    drawUfo(s, ox, oy) {
      const u = this.ufo;
      const x = Math.floor(u.x + ox), y = Math.floor(u.y + oy);
      draw.ellipse(s, [40, 30, 26], [x - 18, y - 6, 36, 14]);
      draw.ellipse(s, COL_UFO, [x - 18, y - 6, 36, 14], 2);
      draw.arc(s, COL_UFO, [x - 9, y - 14, 18, 16], 0, Math.PI, 2);
      draw.line(s, COL_UFO, [x - 18, y + 1.5], [x + 18, y + 1.5], 1);
    }

    drawItem(s, it, ox, oy) {
      const [, sym, farbe] = POWERUP_KINDS[it.kind];
      // kurz vor dem Verschwinden blinken
      if (it.life < 2.5 && Math.floor(this.animT * 6) % 2 === 0) return;
      const x = Math.floor(it.x + ox), y = Math.floor(it.y + oy);
      const r = 12 + Math.trunc(1.5 * Math.sin(this.animT * 5));
      draw.circle(s, [22, 27, 40], [x, y], r);
      draw.circle(s, farbe, [x, y], r, 2);
      ui.text(s, sym, x, y, this.fTiny, farbe, "center");
    }

    // ----- HUD -----------------------------------------------------------------
    drawHud(s) {
      const sh = this.ships[0];
      const x = 12;
      ui.text(s, String(sh.score), x, 8, this.font, sh.color);
      // Leben als kleine Schiffssymbole
      for (let lv = 0; lv < sh.lives; lv++) this.drawLifeIcon(s, x + 6 + lv * 16, 44, sh.color);
      // Aktive Power-Ups (Monospace, damit die Timer nicht springen)
      let y = 58;
      for (const [kind, sym, farbe] of POWERUP_KINDS) {
        if (kind in sh.powers) {
          ui.text(s, sym + " " + String(Math.ceil(sh.powers[kind])).padStart(2, " ") + "s", x, y, this.fTinyMono, farbe);
          y += 15;
        }
      }
      // Hyperraum-Anzeige
      if (sh.alive) {
        if (sh.hyperCd <= 0) ui.text(s, t("ast.hyper"), x, y, this.fTiny, COL_OK);
        else ui.text(s, t("ast.hyper") + " " + Math.ceil(sh.hyperCd) + "s", x, y, this.fTinyMono, COL_DIM);
      }

      const cx = Math.floor(this.width / 2);
      ui.text(s, t("ast.wave", { n: this.wave }), cx, 8, this.fSmall, this.accent, "midtop");

      if (this.bannerT > 0 && !this.gameOver) {
        const alpha = Math.max(0, Math.min(255, Math.floor(255 * this.bannerT))) / 255;
        ui.text(s, t("ast.wave", { n: this.wave }), cx, Math.floor(this.height / 2) - 40, this.bigFont, this.accent, "center", alpha);
      }

      if (this.flashMsg !== null) {
        const [text, farbe] = this.flashMsg;
        ui.text(s, text, cx, 40, this.font, farbe, "midtop");
      }

      if (this.hintT < 6) ui.text(s, t("ast.controls_hint"), cx, this.height - 8, this.fTiny, COL_DIM, "midbottom");
    }

    drawLifeIcon(s, x, y, color) {
      draw.polygon(s, color, [[x + 5, y - 6], [x, y + 6], [x + 5, y + 3], [x + 10, y + 6]], 1);
    }

    drawGameOver(s) {
      draw.rect(s, [0, 0, 0, 150], [0, 0, this.width, this.height]);
      this.drawCenterText(s, t("common.game_over"), this.bigFont, COL_BAD, -50);
      this.drawCenterText(s, t("common.points", { score: this.ships[0].score }), this.font, COL_TEXT, -6);
      this.drawCenterText(s, t("ast.wave_reached", { n: this.wave }), this.font, COL_DIM, 24);
      this.drawCenterText(s, t("ah.restart_hint"), this.font, COL_TEXT, 56);
    }

    // ----- Setup zeichnen ---------------------------------------------------
    /** Setup-Screen im Theme-Stil (Palette wird zur Zeichenzeit gelesen). */
    drawSetup(s) {
      ui.drawBackground(s, this.width, this.height, true);
      ui.drawTitle(s, this.width, "ASTEROIDS", { subtitle: t("snake.singleplayer"), accent: this.accent });

      const d = DIFFS[this.diff];
      draw.rect(s, ui.PANEL, this.diffPanel, 0, 10);
      draw.rect(s, this.accent, this.diffPanel, 2, 10);
      ui.text(s, t("ah.difficulty") + ":  " + t("ah.diff." + d.key), this.diffPanel.centerx, this.diffPanel.top + 19, this.font, ui.TEXT, "center");
      ui.text(s, t("ast.diff_note"), this.diffPanel.centerx, this.diffPanel.top + 41, this.fTiny, ui.TEXT_DIM, "center");
      for (const [r, sym] of [[this.diffLeft, "<"], [this.diffRight, ">"]]) {
        ui.text(s, sym, r.centerx, r.centery, this.bigFont, this.accent, "center");
      }

      this.drawRow(s, this.ufoRect, t("ast.ufos"), this.ufosOn ? t("common.on") : t("common.off"), this.ufosOn);
      this.drawRow(s, this.powerRect, t("ah.powerups"), this.powerupsOn ? t("common.on") : t("common.off"), this.powerupsOn);

      ui.drawButton(s, this.startRect, t("common.start"), this.font, true, { accent: ui.GREEN });

      const cx = Math.floor(this.width / 2);
      ui.text(s, t("ast.setup_hint"), cx, this.height - 38, this.fSmall, ui.TEXT_DIM, "center");
      ui.text(s, t("ast.controls_hint"), cx, this.height - 16, this.fTiny, ui.GREEN, "center");
    }

    drawRow(s, rect, label, wert, an) {
      draw.rect(s, an ? ui.BTN_SEL : ui.BTN, rect, 0, 8);
      draw.rect(s, an ? this.accent : ui.BORDER, rect, 1, 8);
      ui.text(s, label, rect.x + 16, rect.centery, this.font, ui.TEXT, "midleft");
      ui.text(s, "< " + wert + " >", rect.right - 16, rect.centery, this.font, an ? this.accent : ui.TEXT_DIM, "midright");
    }
  }

  PG.register(AsteroidsGame, {
    id: "AsteroidsGame",
    key: "asteroids",
    name: "Asteroids",
    settingsKey: "asteroids",
    defaults: { difficulty: 1, ufos: true, powerups: true },
  });
})();
