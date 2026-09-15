/*
 * invaders.js - Space Invaders mit zwei Spielmodi (Port von games/invaders.py)
 * =============================================================================
 * - KLASSIK : Klassischer Alien-Block. Setup-Screen davor:
 *               * Bewegung: nur links/rechts (unten) ODER frei mit WASD/Pfeilen.
 *               * Zielen:   immer nach oben ODER zur Maus.
 *             Zerstörte Aliens lassen manchmal Power-ups fallen.
 * - ARENA   : Freie Bewegung, Gegner strömen von allen Rändern herein.
 *             Geschossen wird in Blickrichtung, Waffenwechsel mit 1-4.
 *
 * Gemeinsam: Levelsystem mit BOSS in jedem 4. Level, vier Waffen (Blaster,
 * Streuschuss, Schnellfeuer, Laser), Power-ups (Leben, Schild, Waffe),
 * Explosions-Partikel, HUD.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const R = PG.rand;

  // ----- Farben ---------------------------------------------------------------
  // Allgemeine UI-Farben kommen zur Zeichenzeit dynamisch aus ui.*;
  // hier stehen nur die Identitätsfarben (Schiff, Gegner, Waffen, Projektile).
  const COL_PLAYER = [110, 220, 140];
  const COL_PLAYER_HIT = [240, 240, 240];
  const COL_SHIELD = [90, 170, 220];
  const COL_EBULLET = [240, 120, 120];

  // Waffenfarben (auch für die Schüsse)
  const WEAPON_COL = {
    single: [240, 240, 120],
    spread: [140, 235, 180],
    rapid: [240, 190, 110],
    pierce: [150, 200, 255],
  };
  const WEAPON_ORDER = ["single", "spread", "rapid", "pierce"];

  // Gegnertypen -> Farbe
  const KIND_COL = {
    grunt: [235, 110, 110],
    chaser: [235, 150, 80],
    shooter: [185, 120, 235],
    drifter: [110, 180, 235],
    boss: [235, 90, 140],
  };
  const KIND_SCORE = { grunt: 10, chaser: 15, shooter: 20, drifter: 12, boss: 250 };

  // ----- Spielwerte -----------------------------------------------------------
  const PLAYER_W = 40;
  const PLAYER_H = 22;
  const PLAYER_SPEED = 300;       // px/s
  const PLAYER_Y_OFF = 46;        // Abstand vom unteren Rand (nur Klassik)

  const EBULLET_SPEED = 210;
  const HIT_INVULN = 1.4;         // Sekunden Unverwundbarkeit nach einem Treffer
  const POWERUP_FALL = 70;
  const POWERUP_CHANCE = 0.13;
  const WEAPON_PICKUP_TIME = 14.0; // Dauer eines Waffen-Upgrades (Klassik)

  const ACTS = ["up", "down", "left", "right", "action"];

  class InvadersGame extends PG.Game {
    // ----- Aufbau -------------------------------------------------------
    reset() {
      // arena: Arena-Wellen, Zielen in Bewegungsrichtung, Waffenwechsel 1-4.
      this.arena = this.mode === "free";

      // Klassik-Unteroptionen (werden im Setup-Screen gewählt):
      //   optMove: "lr" (nur links/rechts unten) oder "free" (WASD, frei)
      //   optAim : "fixed" (immer nach oben) oder "mouse" (zur Maus zielen)
      this.optMove = "lr";
      this.optAim = "fixed";
      this.mousePos = [this.width / 2, this.height / 4];
      this.freeMove = false;
      this.mouseAim = false;

      this.initRun();

      // Arena startet sofort; Klassik zeigt zuerst den Setup-Screen.
      if (this.arena) this.enterPlay();
      else {
        this.state = "setup";
        this.buildSetup();
      }
    }

    /** Setzt alle Werte für eine Runde zurück (auch beim Neustart). */
    initRun() {
      this.state = "play";
      this.score = 0;
      this.gameOver = false;
      this.lives = 3;
      this.level = 1;

      this.bullets = [];     // {x,y,vx,vy,dmg,pierce,rad,hits,col}
      this.ebullets = [];    // {x,y,vx,vy,rad}
      this.powerups = [];    // {x,y,vy,kind}
      this.particles = [];   // {x,y,vx,vy,life,max,col}
      this.aliens = [];
      this.shields = [];     // nur Klassik: {x,y,w,h,hp}

      this.weapon = "single";
      this.weaponTimer = 0.0;   // >0: befristetes Upgrade (Klassik); 0: dauerhaft
      this.shieldTimer = 0.0;   // Power-up-Schild
      this.invulnTimer = 0.0;   // kurze Unverwundbarkeit nach Treffer
      this.cooldown = 0.0;

      this.held = new Set();    // aktuell gedrückte Aktionen
      this.face = [0.0, -1.0];  // Blickrichtung

      this.px = this.width / 2;
      this.py = this.height - PLAYER_Y_OFF;

      this.banner = "";
      this.bannerT = 0.0;
      this.startCount = 0;
    }

    /** Leitet aus den (Klassik-)Optionen die Flags ab und startet Level 1. */
    enterPlay() {
      if (this.arena) {
        this.freeMove = true;    // in alle Richtungen
        this.mouseAim = false;   // Arena zielt in Bewegungsrichtung
      } else {
        this.freeMove = this.optMove === "free";
        this.mouseAim = this.optAim === "mouse";
      }
      this.state = "play";
      this.level = 1;
      this.startLevel();
    }

    /** Neustart nach Game Over - behält die gewählten Klassik-Optionen. */
    restart() {
      this.initRun();
      this.enterPlay();
    }

    // ----- Setup-Screen (nur Klassik) -----------------------------------
    buildSetup() {
      this.setupSel = 0;
      this.setupRects = [];
      const bw = 340, bh = 42, gap = 14;
      const total = 3 * (bh + gap) - gap;
      const y0 = Math.max(150, Math.floor(this.height / 2) - Math.floor(total / 2) + 20);
      for (let i = 0; i < 3; i++) {
        const x = Math.floor(this.width / 2) - Math.floor(bw / 2);
        this.setupRects.push(new PG.Rect(x, y0 + i * (bh + gap), bw, bh));
      }
    }

    setupEvent(ev) {
      const activate = (k) => k === "Return" || k === "space" || this.isAction(k, "action");
      if (ev.kind === "keydown") {
        if (this.isAction(ev.key, "up")) {
          this.setupSel = PG.mod(this.setupSel - 1, 3);
          this.playSound("move");
        } else if (this.isAction(ev.key, "down")) {
          this.setupSel = PG.mod(this.setupSel + 1, 3);
          this.playSound("move");
        } else if (this.isAction(ev.key, "left") || this.isAction(ev.key, "right")) {
          this.setupToggle();
        } else if (activate(ev.key)) {
          this.setupActivate();
        }
      } else if (ev.kind === "mousemove" && ev.pos) {
        this.setupRects.forEach((r, i) => {
          if (r.collidepoint(ev.pos)) this.setupSel = i;
        });
      } else if (ev.kind === "mousedown" && ev.pos) {
        for (let i = 0; i < this.setupRects.length; i++) {
          if (this.setupRects[i].collidepoint(ev.pos)) {
            this.setupSel = i;
            this.setupActivate();
            break;
          }
        }
      }
    }

    setupToggle() {
      if (this.setupSel === 0) {
        this.optMove = this.optMove === "lr" ? "free" : "lr";
        this.playSound("select");
      } else if (this.setupSel === 1) {
        this.optAim = this.optAim === "fixed" ? "mouse" : "fixed";
        this.playSound("select");
      }
    }

    setupActivate() {
      if (this.setupSel === 2) {
        this.playSound("click");
        this.enterPlay();
      } else {
        this.setupToggle();
      }
    }

    levelBoss() {
      return this.level % 4 === 0;
    }

    /** Baut ein Level (Formation bzw. Arena-Welle) passend zum Modus auf. */
    startLevel() {
      this.bullets = [];
      this.ebullets = [];
      this.powerups = [];
      this.aliens = [];

      const boss = this.levelBoss();
      // Startbanner
      this.banner = boss ? t("inv.boss_level") : t("inv.level_start", { level: this.level });
      this.bannerT = 2.2;

      // Spieler neu setzen: Arena startet mittig, Klassik-Varianten unten.
      if (this.arena) {
        this.px = this.width / 2;
        this.py = this.height / 2;
      } else {
        this.px = this.width / 2;
        this.py = this.height - PLAYER_Y_OFF;
      }

      if (boss) this.spawnBoss();
      else if (this.arena) this.spawnArenaWave();
      else this.spawnClassicBlock();

      if (!this.arena) this.buildShields();

      this.startCount = Math.max(1, this.aliens.length);
    }

    makeAlien(cx, cy, kind, hp, w = 30, h = 22, vx = 0.0, vy = 0.0) {
      return { cx, cy, kind, hp, maxhp: hp, w, h, vx, vy, t: R.random() * 6.28, shoot: R.uniform(0.6, 2.2) };
    }

    spawnClassicBlock() {
      const rows = Math.min(6, 3 + Math.floor(this.level / 2));
      const cols = Math.min(10, 6 + Math.floor(this.level / 3));
      const hp = 1 + Math.floor(this.level / 4);
      const gapX = 16, gapY = 14;
      const blockW = cols * (30 + gapX) - gapX;
      const startX = (this.width - blockW) / 2 + 15;
      const startY = 60;
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const a = this.makeAlien(startX + c * (30 + gapX), startY + r * (22 + gapY), "grunt", hp);
          a.row = r;
          this.aliens.push(a);
        }
      }
      this.alienDir = 1;
      this.alienSpeed = 22 + this.level * 8;
      this.alienDrop = 16;
      this.shootTimer = 0.0;
    }

    spawnArenaWave() {
      const count = 5 + this.level * 2;
      const hp = 1 + Math.floor(this.level / 3);
      const kinds = ["chaser", "drifter", "shooter"];
      for (let i = 0; i < count; i++) {
        const [cx, cy] = this.edgeSpawn();
        const kind = R.choice(kinds);
        const spd = 40 + this.level * 5 + R.uniform(-10, 20);
        const ang = Math.atan2(this.py - cy, this.px - cx);
        this.aliens.push(this.makeAlien(cx, cy, kind, hp, 30, 22, Math.cos(ang) * spd, Math.sin(ang) * spd));
      }
    }

    spawnBoss() {
      const bx = this.width / 2, by = 90;
      const hp = 26 + this.level * 6;
      const boss = this.makeAlien(bx, by, "boss", hp, 90, 54, 70 + this.level * 4, 0);
      boss.burst = 2.4;
      this.aliens.push(boss);
      // ein paar Begleiter
      for (let i = 0; i < 4; i++) {
        if (this.arena) {
          const [cx, cy] = this.edgeSpawn();
          this.aliens.push(this.makeAlien(cx, cy, "drifter", 1, 30, 22, R.uniform(-60, 60), R.uniform(30, 70)));
        } else {
          const a = this.makeAlien(120 + i * 130, 170, "grunt", 1);
          a.row = 0;
          this.aliens.push(a);
        }
      }
      if (!this.arena) {
        this.alienDir = 1;
        this.alienSpeed = 30;
        this.alienDrop = 12;
        this.shootTimer = 0.0;
      }
    }

    /** Zufällige Position knapp außerhalb eines Randes (für Arena-Gegner). */
    edgeSpawn() {
      const side = R.choice(["top", "bottom", "left", "right"]);
      if (side === "top") return [R.uniform(20, this.width - 20), -20];
      if (side === "bottom") return [R.uniform(20, this.width - 20), this.height + 20];
      if (side === "left") return [-20, R.uniform(40, this.height - 20)];
      return [this.width + 20, R.uniform(40, this.height - 20)];
    }

    buildShields() {
      this.shields = [];
      for (let i = 0; i < 4; i++) {
        const sx = ((i + 1) * this.width) / 5 - 26;
        this.shields.push({ x: sx, y: this.height - 130, w: 52, h: 18, hp: 6 });
      }
    }

    // ----- Eingabe ------------------------------------------------------
    handleEvent(ev) {
      // Setup-Screen (nur Klassik, vor dem ersten Level).
      if (this.state === "setup") {
        this.setupEvent(ev);
        return;
      }

      // Maus: Position merken (zum Zielen), Klick schießt.
      if (ev.kind === "mousemove") {
        if (ev.pos) this.mousePos = ev.pos;
        return;
      }
      if (ev.kind === "mousedown") {
        if (ev.pos) this.mousePos = ev.pos;
        if (!this.gameOver) this.fire();
        return;
      }

      if (ev.kind === "keyup") {
        for (const act of ACTS) if (this.isAction(ev.key, act)) this.held.delete(act);
        return;
      }

      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.restart();
        return;
      }

      // Waffenwechsel per Zifferntaste nur in der Arena. In Klassik kommen
      // bessere Waffen (befristet) über Power-ups.
      if (["1", "2", "3", "4"].includes(ev.key)) {
        if (this.arena) this.selectWeapon(WEAPON_ORDER[Number(ev.key) - 1]);
        return;
      }

      for (const act of ACTS) if (this.isAction(ev.key, act)) this.held.add(act);
    }

    selectWeapon(weapon) {
      if (weapon === this.weapon) return;
      this.weapon = weapon;
      this.weaponTimer = 0.0; // manuell gewählt -> dauerhaft
      this.playSound("select");
    }

    // ----- Update -------------------------------------------------------
    update(dt) {
      if (this.gameOver || this.state !== "play") return;

      this.cooldown = Math.max(0.0, this.cooldown - dt);
      this.shieldTimer = Math.max(0.0, this.shieldTimer - dt);
      this.invulnTimer = Math.max(0.0, this.invulnTimer - dt);
      this.bannerT = Math.max(0.0, this.bannerT - dt);

      // Befristetes Waffen-Upgrade (Klassik) läuft ab -> zurück auf Blaster.
      if (this.weaponTimer > 0) {
        this.weaponTimer -= dt;
        if (this.weaponTimer <= 0) this.weapon = "single";
      }

      this.updatePlayer(dt);
      if (this.held.has("action")) this.fire();

      if (this.arena) this.updateArena(dt);
      else this.updateClassic(dt);

      // In allen Modi mit freier Bewegung schadet Körperkontakt mit Gegnern.
      if (this.freeMove) this.checkContactDamage();

      this.updateBullets(dt);
      this.updateEbullets(dt);
      this.updatePowerups(dt);
      this.updateParticles(dt);

      // Level geschafft (erst wenn das Startbanner durch ist).
      if (!this.aliens.length && this.bannerT <= 0) this.nextLevel();
    }

    nextLevel() {
      this.level += 1;
      this.score += 50;
      this.playSound("level");
      this.startLevel();
    }

    updatePlayer(dt) {
      const vx = (this.held.has("right") ? 1 : 0) - (this.held.has("left") ? 1 : 0);
      const vy = this.freeMove ? (this.held.has("down") ? 1 : 0) - (this.held.has("up") ? 1 : 0) : 0;

      let nx = 0.0, ny = 0.0;
      if (vx || vy) {
        const length = Math.hypot(vx, vy) || 1.0;
        nx = vx / length;
        ny = vy / length;
        this.px += nx * PLAYER_SPEED * dt;
        if (this.freeMove) this.py += ny * PLAYER_SPEED * dt;
      }

      // Blickrichtung bestimmen:
      //  - Maus-Zielen: immer zur Mausposition.
      //  - Arena: der Bewegung folgen.
      //  - sonst (Klassik): IMMER nach vorne/oben - auch beim Rückwärtsgehen.
      if (this.mouseAim) {
        const dx = this.mousePos[0] - this.px;
        const dy = this.mousePos[1] - this.py;
        const d = Math.hypot(dx, dy);
        if (d > 1) this.face = [dx / d, dy / d];
      } else if (this.arena) {
        if (vx || vy) this.face = [nx, ny];
      } else {
        this.face = [0.0, -1.0];
      }

      // In den Spielbereich zwingen.
      this.px = Math.max(PLAYER_W / 2, Math.min(this.width - PLAYER_W / 2, this.px));
      if (this.freeMove) this.py = Math.max(34 + PLAYER_H / 2, Math.min(this.height - PLAYER_H / 2, this.py));
      else this.py = this.height - PLAYER_Y_OFF;
    }

    fire() {
      if (this.cooldown > 0) return;
      const [fx, fy] = this.face;
      const ang = Math.atan2(fy, fx);
      const ox = this.px + fx * (PLAYER_H / 2 + 4), oy = this.py + fy * (PLAYER_H / 2 + 4);
      const w = this.weapon;
      if (w === "spread") {
        for (const da of [-0.26, 0.0, 0.26]) this.addBullet(ox, oy, ang + da, 460, 1, 0);
        this.cooldown = 0.46;
      } else if (w === "rapid") {
        this.addBullet(ox, oy, ang + R.uniform(-0.05, 0.05), 560, 1, 0);
        this.cooldown = 0.1;
      } else if (w === "pierce") {
        this.addBullet(ox, oy, ang, 720, 2, 3, 5);
        this.cooldown = 0.55;
      } else {
        // single
        this.addBullet(ox, oy, ang, 520, 1, 0);
        this.cooldown = 0.28;
      }
      this.playSound("shoot");
    }

    addBullet(x, y, ang, spd, dmg, pierce, rad = 3) {
      // Farbe beim Abschuss festhalten - so färben sich fliegende Kugeln
      // beim Waffenwechsel nicht nachträglich um.
      this.bullets.push({ x, y, vx: Math.cos(ang) * spd, vy: Math.sin(ang) * spd, dmg, pierce, rad, hits: new Set(), col: WEAPON_COL[this.weapon] || [240, 240, 120] });
    }

    // ----- Gegner: Klassik ----------------------------------------------
    updateClassic(dt) {
      const minions = this.aliens.filter((a) => a.kind !== "boss");
      const bosses = this.aliens.filter((a) => a.kind === "boss");

      if (minions.length) {
        const factor = 1.0 + (this.startCount - this.aliens.length) * 0.025;
        const dx = this.alienDir * this.alienSpeed * factor * dt;
        let left = Infinity, right = -Infinity;
        for (const a of minions) {
          left = Math.min(left, a.cx - a.w / 2);
          right = Math.max(right, a.cx + a.w / 2);
        }
        if ((this.alienDir > 0 && right + dx >= this.width - 8) || (this.alienDir < 0 && left + dx <= 8)) {
          this.alienDir *= -1;
          for (const a of minions) a.cy += this.alienDrop;
        } else {
          for (const a of minions) a.cx += dx;
        }

        // Zufälliger Schuss aus der jeweils untersten Reihe.
        this.shootTimer -= dt;
        if (this.shootTimer <= 0) {
          this.shootTimer = Math.max(0.25, R.uniform(0.5, 1.3) - this.level * 0.03);
          const columns = new Map();
          for (const a of minions) {
            const key = Math.round(a.cx / 20);
            if (!columns.has(key) || a.cy > columns.get(key).cy) columns.set(key, a);
          }
          const shooter = R.choice([...columns.values()]);
          this.enemyShot(shooter.cx, shooter.cy + shooter.h / 2, 0, 1);
        }
      }

      for (const b of bosses) this.updateBoss(b, dt);

      // Erreichen die Gegner "unten" -> Treffer, Block etwas hoch. Bei freier
      // Bewegung ist das die Feld-Unterkante, sonst die feste Spielerhöhe.
      const limit = this.freeMove ? this.height - 40 : this.py - PLAYER_H / 2;
      if (minions.some((a) => a.cy + a.h / 2 >= limit)) {
        this.hitPlayer();
        for (const a of minions) a.cy -= this.alienDrop * 3;
      }
    }

    // ----- Gegner: Arena ------------------------------------------------
    updateArena(dt) {
      for (const a of this.aliens) {
        if (a.kind === "boss") {
          this.updateBoss(a, dt);
          continue;
        }
        this.updateArenaEnemy(a, dt);
      }
    }

    /** Körperkontakt mit einem Gegner kostet ein Leben (mit i-Frames). */
    checkContactDamage() {
      if (this.invulnTimer > 0 || this.shieldTimer > 0) return;
      for (const a of this.aliens) {
        if (Math.abs(a.cx - this.px) < ((a.w + PLAYER_W) / 2) * 0.6 && Math.abs(a.cy - this.py) < ((a.h + PLAYER_H) / 2) * 0.6) {
          this.hitPlayer();
          break;
        }
      }
    }

    updateArenaEnemy(a, dt) {
      const kind = a.kind;
      a.t += dt;
      if (kind === "chaser") {
        const ang = Math.atan2(this.py - a.cy, this.px - a.cx);
        const spd = 60 + this.level * 6;
        a.vx = Math.cos(ang) * spd;
        a.vy = Math.sin(ang) * spd;
      } else if (kind === "drifter") {
        // gerade Bahn, an den Rändern abprallen
        if (a.cx < 12 || a.cx > this.width - 12) a.vx *= -1;
        if (a.cy < 34 || a.cy > this.height - 12) a.vy *= -1;
      } else if (kind === "shooter") {
        // Abstand halten und den Spieler beschießen
        const dist = Math.hypot(this.px - a.cx, this.py - a.cy) || 1;
        const ang = Math.atan2(this.py - a.cy, this.px - a.cx);
        const drive = dist > 240 ? 1 : dist < 150 ? -1 : 0;
        const spd = 55 + this.level * 4;
        const strafe = 0.9;
        a.vx = (Math.cos(ang) * drive - Math.sin(ang) * strafe) * spd;
        a.vy = (Math.sin(ang) * drive + Math.cos(ang) * strafe) * spd;
        a.shoot -= dt;
        if (a.shoot <= 0) {
          a.shoot = R.uniform(1.1, 2.2);
          this.enemyShot(a.cx, a.cy, Math.cos(ang), Math.sin(ang));
        }
      }

      a.cx += a.vx * dt;
      a.cy += a.vy * dt;
      // innerhalb des Feldes halten (nur weich)
      a.cx = Math.max(-30, Math.min(this.width + 30, a.cx));
      a.cy = Math.max(-30, Math.min(this.height + 30, a.cy));
    }

    // ----- Boss (beide Modi) --------------------------------------------
    updateBoss(b, dt) {
      b.t += dt;
      // horizontal pendeln
      b.cx += b.vx * dt;
      if (b.cx < b.w / 2 + 8 || b.cx > this.width - b.w / 2 - 8) {
        b.vx *= -1;
        b.cx = Math.max(b.w / 2 + 8, Math.min(this.width - b.w / 2 - 8, b.cx));
      }
      b.cy = 90 + Math.sin(b.t * 1.3) * 22;

      b.burst = (b.burst != null ? b.burst : 2.4) - dt;
      if (b.burst <= 0) {
        b.burst = Math.max(1.1, 2.6 - this.level * 0.05);
        // radiale Salve + gezielter Schuss
        const n = 10;
        for (let i = 0; i < n; i++) {
          const ang = (i / n) * PG.TAU + b.t;
          this.enemyShot(b.cx, b.cy, Math.cos(ang), Math.sin(ang), 150);
        }
        const ang = Math.atan2(this.py - b.cy, this.px - b.cx);
        this.enemyShot(b.cx, b.cy, Math.cos(ang), Math.sin(ang), 260);
      }
    }

    enemyShot(x, y, dx, dy, speed = EBULLET_SPEED) {
      const length = Math.hypot(dx, dy) || 1.0;
      this.ebullets.push({ x, y, vx: (dx / length) * speed, vy: (dy / length) * speed, rad: 4 });
    }

    // ----- Projektile & Kollisionen -------------------------------------
    updateBullets(dt) {
      const alive = [];
      for (const b of this.bullets) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        if (b.x < -20 || b.x > this.width + 20 || b.y < -20 || b.y > this.height + 20) continue;
        if (!this.arena && this.hitsShield(b)) continue;
        if (this.bulletHitsAlien(b)) {
          if (b.pierce > 0) {
            b.pierce -= 1; // durchschlägt -> weiterfliegen
            alive.push(b);
          }
          // sonst: Kugel verbraucht
        } else {
          alive.push(b);
        }
      }
      this.bullets = alive;
    }

    bulletHitsAlien(b) {
      for (const a of this.aliens) {
        if (b.hits.has(a)) continue;
        if (Math.abs(b.x - a.cx) <= a.w / 2 + b.rad && Math.abs(b.y - a.cy) <= a.h / 2 + b.rad) {
          b.hits.add(a);
          a.hp -= b.dmg;
          if (a.hp <= 0) this.killAlien(a);
          else this.playSound("hit");
          return true;
        }
      }
      return false;
    }

    killAlien(a) {
      const i = this.aliens.indexOf(a);
      if (i >= 0) this.aliens.splice(i, 1);
      this.score += KIND_SCORE[a.kind] || 10;
      this.spawnParticles(a.cx, a.cy, KIND_COL[a.kind] || [235, 110, 110], a.kind === "boss" ? 18 : 8);
      this.playSound("explode");
      if (a.kind === "boss") {
        this.rumble(220);
        this.dropPowerup(a.cx, a.cy, true);
      } else if (R.random() < POWERUP_CHANCE) {
        this.dropPowerup(a.cx, a.cy);
      }
    }

    /** Spieler- und Gegnerschüsse beschädigen die Klassik-Schilde gleichermaßen. */
    hitsShield(b) {
      for (const sh of this.shields) {
        if (sh.hp > 0 && sh.x <= b.x && b.x <= sh.x + sh.w && sh.y <= b.y && b.y <= sh.y + sh.h) {
          sh.hp -= 1;
          return true;
        }
      }
      return false;
    }

    updateEbullets(dt) {
      const alive = [];
      for (const b of this.ebullets) {
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        if (b.x < -20 || b.x > this.width + 20 || b.y < -20 || b.y > this.height + 20) continue;
        if (!this.arena && this.hitsShield(b)) continue;
        if (this.canBeHit() && Math.abs(b.x - this.px) <= PLAYER_W / 2 + b.rad && Math.abs(b.y - this.py) <= PLAYER_H / 2 + b.rad) {
          this.hitPlayer();
          // hitPlayer leert die Liste -> wie in Python endet der Durchlauf hier
          break;
        }
        alive.push(b);
      }
      this.ebullets = alive;
    }

    canBeHit() {
      return this.invulnTimer <= 0 && this.shieldTimer <= 0;
    }

    // ----- Power-ups ----------------------------------------------------
    dropPowerup(x, y, force = false) {
      const kind = R.weighted(["weapon", "shield", "life"], [6, 4, 2]);
      this.powerups.push({ x, y, vy: POWERUP_FALL, kind });
    }

    updatePowerups(dt) {
      const alive = [];
      for (const p of this.powerups) {
        p.y += p.vy * dt;
        if (p.y > this.height + 20) continue;
        if (Math.abs(p.x - this.px) < PLAYER_W && Math.abs(p.y - this.py) < PLAYER_H) {
          this.collectPowerup(p);
          continue;
        }
        alive.push(p);
      }
      this.powerups = alive;
    }

    collectPowerup(p) {
      this.playSound("powerup");
      if (p.kind === "life") this.lives = Math.min(9, this.lives + 1);
      else if (p.kind === "shield") this.shieldTimer = 6.0;
      else {
        // weapon
        this.weapon = R.choice(["spread", "rapid", "pierce"]);
        // In der Arena kann man ohnehin frei wechseln -> dort dauerhaft.
        this.weaponTimer = this.arena ? 0.0 : WEAPON_PICKUP_TIME;
      }
    }

    // ----- Partikel -----------------------------------------------------
    spawnParticles(x, y, col, n) {
      for (let i = 0; i < n; i++) {
        const ang = R.uniform(0, PG.TAU);
        const spd = R.uniform(40, 190);
        const life = R.uniform(0.3, 0.7);
        this.particles.push({ x, y, vx: Math.cos(ang) * spd, vy: Math.sin(ang) * spd, life, max: life, col });
      }
    }

    updateParticles(dt) {
      const alive = [];
      for (const p of this.particles) {
        p.life -= dt;
        if (p.life <= 0) continue;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.vx *= 0.92;
        p.vy *= 0.92;
        alive.push(p);
      }
      this.particles = alive;
    }

    // ----- Spieler-Treffer ----------------------------------------------
    hitPlayer() {
      if (!this.canBeHit()) return;
      this.lives -= 1;
      this.invulnTimer = HIT_INVULN;
      this.ebullets = [];
      this.playSound("hit");
      this.rumble(180);
      this.spawnParticles(this.px, this.py, COL_PLAYER, 14);
      if (this.lives <= 0) {
        this.lives = 0;
        this.gameOver = true;
        this.playSound("gameover");
      }
    }

    // ----- Zeichnen -----------------------------------------------------
    draw(ctx) {
      const s = ctx;
      // Theme-Weltraum-Hintergrund mit Sternenfeld
      ui.drawBackground(s, this.width, this.height, true);

      // Setup-Screen (Klassik) statt Spielfeld.
      if (this.state === "setup") {
        this.drawSetup(s);
        return;
      }

      // Klassik-Schutzschilde
      for (const sh of this.shields) {
        if (sh.hp <= 0) continue;
        const g = sh.hp / 6;
        const farbe = [Math.floor(60 + 30 * g), Math.floor(110 + 60 * g), Math.floor(150 + 70 * g)];
        draw.rect(s, farbe, [sh.x, sh.y, sh.w, sh.h], 0, 4);
      }

      // Gegner
      for (const a of this.aliens) this.drawAlien(s, a);

      // Power-ups
      for (const p of this.powerups) this.drawPowerup(s, p);

      // Schüsse
      for (const b of this.bullets) draw.circle(s, b.col, [Math.floor(b.x), Math.floor(b.y)], b.rad + 1);
      for (const b of this.ebullets) draw.circle(s, COL_EBULLET, [Math.floor(b.x), Math.floor(b.y)], b.rad);

      // Partikel
      for (const p of this.particles) {
        const a = Math.max(0.0, p.life / p.max);
        const r = Math.max(1, Math.floor(3 * a));
        draw.circle(s, p.col, [Math.floor(p.x), Math.floor(p.y)], r);
      }

      this.drawPlayer(s);

      // Maus-Fadenkreuz beim Maus-Zielen
      if (this.mouseAim) {
        const mx = Math.floor(this.mousePos[0]), my = Math.floor(this.mousePos[1]);
        draw.circle(s, ui.TEXT, [mx, my], 8, 1);
        draw.line(s, ui.TEXT, [mx - 11, my], [mx - 4, my]);
        draw.line(s, ui.TEXT, [mx + 4, my], [mx + 11, my]);
      }

      this.drawHud(s);
      this.drawBanner(s);

      if (this.gameOver) {
        this.drawCenterText(s, t("common.game_over"), this.bigFont, ui.RED, -20);
        this.drawCenterText(s, t("common.enter_restart"), this.font, ui.TEXT, 30);
      }
    }

    /** Zeichnet den Klassik-Setup-Screen (Bewegung/Zielen wählen). */
    drawSetup(s) {
      const cx = Math.floor(this.width / 2);
      const name = "Invaders";
      // Titel mit leichtem Glanz in der Akzentfarbe des Spiels
      ui.text(s, name, cx + 2, 72, this.bigFont, this.accent, "center");
      ui.text(s, name, cx, 70, this.bigFont, ui.TEXT, "center");
      ui.text(s, t("inv.setup_title"), cx, 112, this.font, ui.TEXT_DIM, "center");

      const rows = [
        [t("inv.opt_move"), this.optMove === "free" ? t("inv.opt_move_free") : t("inv.opt_move_lr")],
        [t("inv.opt_aim"), this.optAim === "mouse" ? t("inv.opt_aim_mouse") : t("inv.opt_aim_fixed")],
        [t("common.start"), null],
      ];
      this.setupRects.forEach((r, i) => {
        const selected = i === this.setupSel;
        draw.rect(s, selected ? ui.BTN_SEL : ui.BTN, r, 0, 8);
        draw.rect(s, selected ? this.accent : ui.BORDER, r, 1, 8);
        const [label, value] = rows[i];
        if (value === null) {
          // Start-Button: zentriert
          ui.text(s, label, r.centerx, r.centery, this.font, ui.TEXT, "center");
        } else {
          ui.text(s, label, r.x + 14, r.centery, this.font, ui.TEXT, "midleft");
          ui.text(s, "< " + value + " >", r.right - 14, r.centery, this.font, selected ? ui.GOLD : ui.TEXT_DIM, "midright");
        }
      });

      ui.text(s, t("inv.setup_hint"), cx, this.height - 24, this.font, ui.TEXT_DIM, "center");
    }

    drawAlien(s, a) {
      let col = KIND_COL[a.kind] || [235, 110, 110];
      // Schaden abdunkeln
      if (a.maxhp > 1) {
        const g = 0.4 + 0.6 * (a.hp / a.maxhp);
        col = [Math.floor(col[0] * g), Math.floor(col[1] * g), Math.floor(col[2] * g)];
      }
      const x = Math.floor(a.cx - a.w / 2);
      const y = Math.floor(a.cy - a.h / 2);
      draw.rect(s, col, [x, y, a.w, a.h], 0, 6);
      // Augen ("Löcher" in der Hintergrundfarbe des Themes)
      const ey = y + Math.floor(a.h / 3);
      draw.rect(s, ui.BG_TOP, [x + 7, ey, 4, 4]);
      draw.rect(s, ui.BG_TOP, [x + a.w - 11, ey, 4, 4]);
      if (a.kind === "boss") {
        // HP-Balken über dem Boss
        const bw = a.w;
        draw.rect(s, ui.PANEL, [x, y - 10, bw, 5]);
        draw.rect(s, [235, 90, 140], [x, y - 10, Math.floor((bw * Math.max(0, a.hp)) / a.maxhp), 5]);
      }
    }

    drawPowerup(s, p) {
      const icons = { life: ["+", [240, 110, 120]], shield: ["S", [90, 170, 220]], weapon: ["W", [240, 210, 120]] };
      const [letter, col] = icons[p.kind] || ["?", ui.TEXT];
      draw.circle(s, col, [Math.floor(p.x), Math.floor(p.y)], 11, 2);
      ui.text(s, letter, Math.floor(p.x), Math.floor(p.y), this.font, col, "center");
    }

    drawPlayer(s) {
      // Blinken während der Unverwundbarkeit nach einem Treffer.
      const col = this.invulnTimer > 0 && Math.floor(this.invulnTimer * 12) % 2 === 0 ? COL_PLAYER_HIT : COL_PLAYER;
      const [fx, fy] = this.face;
      const ang = Math.atan2(fy, fx);
      // Dreieck (Nase in Blickrichtung)
      const tip = [this.px + Math.cos(ang) * PLAYER_H * 0.8, this.py + Math.sin(ang) * PLAYER_H * 0.8];
      const left = [this.px + Math.cos(ang + 2.5) * PLAYER_W * 0.5, this.py + Math.sin(ang + 2.5) * PLAYER_W * 0.5];
      const right = [this.px + Math.cos(ang - 2.5) * PLAYER_W * 0.5, this.py + Math.sin(ang - 2.5) * PLAYER_W * 0.5];
      draw.polygon(s, col, [tip, left, right]);

      // Schutzschild-Ring
      if (this.shieldTimer > 0) {
        const r = Math.floor(Math.max(PLAYER_W, PLAYER_H) * 0.8);
        draw.circle(s, COL_SHIELD, [Math.floor(this.px), Math.floor(this.py)], r, 2);
      }
    }

    drawHud(s) {
      // Halbtransparenter Panel-Streifen oben (im Theme-Ton)
      draw.rect(s, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 140], [0, 0, this.width, 32]);
      draw.line(s, ui.BORDER, [0, 31.5], [this.width, 31.5]);

      ui.text(s, t("common.points", { score: this.score }), 10, 6, this.font, ui.TEXT);
      ui.text(s, t("inv.level", { level: this.level }), Math.floor(this.width / 2), 6, this.font, ui.TEXT, "midtop");
      ui.text(s, t("inv.lives", { lives: this.lives }), this.width - 10, 6, this.font, ui.TEXT, "topright");

      // Waffe (unten links), mit befristeter Restzeit falls Upgrade.
      let wname = t("inv.wpn_" + this.weapon);
      if (this.weaponTimer > 0) wname += "  " + this.weaponTimer.toFixed(0) + "s";
      ui.text(s, t("inv.weapon", { weapon: wname }), 10, this.height - 26, this.font, WEAPON_COL[this.weapon] || ui.TEXT);
    }

    drawBanner(s) {
      if (this.bannerT <= 0) return;
      // Halbtransparentes Panel-Band mit Akzent-Kanten hinter dem Banner
      const top = Math.floor(this.height / 2) - 62;
      const ac = this.accent;
      draw.rect(s, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 150], [0, top, this.width, 96]);
      draw.rect(s, [ac[0], ac[1], ac[2], 200], [0, top, this.width, 2]);
      draw.rect(s, [ac[0], ac[1], ac[2], 200], [0, top + 94, this.width, 2]);
      const cx = Math.floor(this.width / 2);
      ui.text(s, this.banner, cx, Math.floor(this.height / 2) - 30, this.bigFont, ui.TEXT, "center");
      let hint;
      if (this.mouseAim) hint = t("inv.hint_mouse");
      else if (this.arena) hint = t("inv.hint_free");
      else if (this.freeMove) hint = t("inv.hint_classic_free");
      else hint = t("inv.hint_classic");
      ui.text(s, hint, cx, Math.floor(this.height / 2) + 16, this.font, ui.TEXT_DIM, "center");
    }
  }

  PG.register(InvadersGame, {
    id: "InvadersGame",
    key: "invaders",
    name: "Invaders",
    modes: [["classic", "inv.mode_classic"], ["free", "inv.mode_free"]],
  });
})();
