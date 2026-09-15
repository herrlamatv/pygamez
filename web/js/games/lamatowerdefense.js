/*
 * lamatowerdefense.js - Tower Defense (Port von games/lamatowerdefense.py)
 * ========================================================================
 * Wellen abwehren, Türme bauen und ausbauen (Einzelspieler).
 *
 * - 4 KARTEN (Wiese, Schlucht, Kreuzung, Spießrutenlauf) mit eigenem Pfad,
 *   Schwierigkeitsfaktor und Freischaltung über die beste erreichte Welle.
 * - ENDLOS-WELLEN: Gegner-Budget und Lebenspunkte wachsen pro Welle über
 *   Formeln (kein Wellen-Skript) - wie weit kommst du?
 * - 3 MODI: Klassisch (7 Turmtypen), Kompakt (4 Türme, 2 Stufen),
 *   Maximal (11 Türme, A/B-Spezialisierung, Spezialgegner, Aktiv-Fähigkeiten).
 * - 11 TURMTYPEN mit 3 Ausbaustufen, Verkauf (70% Erstattung) und im
 *   Maximal-Modus einer A/B-Verzweigung auf höchster Stufe.
 * - 11 GEGNERTYPEN inkl. Boss alle 8 Wellen.
 * - ÖKONOMIE: Gold pro Abschuss, Wellen-Bonus, 5% Zinsen in der Bauphase.
 * - Aktiv-Fähigkeiten im Maximal-Modus: Meteor [Q], Frostnova [W], Goldsegen [E].
 *
 * Steuerung: Turmkarte anklicken und im Feld platzieren, Rechtsklick bricht ab.
 * Turm anklicken = Info/Ausbau/Verkauf. Leertaste = Welle starten,
 * F = Tempo x2, G = Reichweiten zeigen, 1-9 = Turm-Schnellwahl.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const MAPSEL = "mapsel", BUILD = "build", WAVE = "wave", GAMEOVER = "gameover";

  const GRID_W = 18, GRID_H = 12; // Spielfeld-Raster in Zellen

  // Obergrenzen (Performance-Schutz; Spawner/Effekte drosseln sich daran)
  const MAX_ENEMIES = 90;
  const MAX_SHOTS = 120;
  const MAX_FX = 150;
  const UNITS_PER_WAVE = 140;

  // Identitätsfarben des Spielfelds (bewusst NICHT aus dem UI-Theme)
  const COL_GRASS1 = [52, 74, 48];
  const COL_GRASS2 = [47, 68, 44];
  const COL_PATH = [122, 100, 70];
  const COL_PATH_EDGE = [86, 70, 50];
  const COL_BUILD_OK = [110, 220, 130];
  const COL_BUILD_BAD = [235, 100, 90];
  const COL_HP_BG = [40, 30, 30];
  const COL_HP = [110, 220, 110];
  const COL_AIR_SHADOW = [25, 30, 22];

  // ---------------------------------------------------------------------------
  //  Daten-Tabellen. "modes": c = Kompakt, C = Klassisch, M = Maximal.
  // ---------------------------------------------------------------------------
  const MODE_RULES = {
    compact: { letter: "c", levels: 2, gold: 260, lives: 20, boss: 8, branch: false, abil: false },
    classic: { letter: "C", levels: 3, gold: 220, lives: 20, boss: 8, branch: false, abil: false },
    maximal: { letter: "M", levels: 3, gold: 220, lives: 20, boss: 8, branch: true, abil: true },
  };

  // Türme: cost/dmg/rng (Zellen)/cd (s). Extras:
  //   splash (Zellen), slow+slow_t, dot+dot_t (Dauerschaden), chain (Kettenblitz),
  //   buff (Aura +Schaden), income (Gold je Welle), lob (Mörser-Bogen),
  //   beam (Dauerstrahl), hitscan (Sofort-Treffer), air (trifft Flieger),
  //   air_only, detect (enttarnt Getarnte), min_rng, prio ("strong" = meiste HP)
  const TOWERS = {
    arrow: { cost: 50, dmg: 8, rng: 2.6, cd: 0.55, proj: 9.0, modes: "cCM", col: [240, 205, 100] },
    cannon: { cost: 90, dmg: 18, rng: 2.2, cd: 1.4, proj: 7.0, modes: "cCM", col: [205, 125, 85], splash: 0.9 },
    frost: { cost: 70, dmg: 2, rng: 2.0, cd: 0.8, proj: 8.0, modes: "cCM", col: [125, 200, 250], slow: 0.45, slow_t: 1.6, detect: true },
    sniper: { cost: 120, dmg: 45, rng: 5.5, cd: 2.6, modes: "cCM", col: [180, 230, 140], hitscan: true, air: true, detect: true, prio: "strong", pierce: true },
    poison: { cost: 85, dmg: 4, rng: 2.2, cd: 0.9, proj: 8.0, modes: "CM", col: [150, 220, 90], dot: 12, dot_t: 3.0 },
    tesla: { cost: 130, dmg: 14, rng: 2.0, cd: 1.1, modes: "CM", col: [150, 175, 255], chain: 3, air: true },
    support: { cost: 100, dmg: 0, rng: 1.8, cd: 0, modes: "CM", col: [255, 170, 220], buff: 0.2, detect: true },
    mortar: { cost: 140, dmg: 30, rng: 8.5, cd: 3.0, modes: "M", col: [190, 190, 170], splash: 1.2, min_rng: 2.2, lob: true },
    flak: { cost: 95, dmg: 22, rng: 3.0, cd: 0.9, proj: 10.0, modes: "M", col: [255, 140, 120], air: true, air_only: true },
    laser: { cost: 160, dmg: 26, rng: 2.8, cd: 0, modes: "M", col: [255, 95, 95], beam: true, air: true },
    bank: { cost: 110, dmg: 0, rng: 0, cd: 0, modes: "M", col: [250, 215, 110], income: 12 },
  };

  // A/B-Spezialisierung (nur Maximal, auf höchster Stufe). Namen über td.br.<turm>.a / .b.
  const BRANCH_COST = 1.6; // x Basiskosten

  // Gegner: hp/spd (Zellen je s)/bounty (Gold)/score. Extras:
  //   armor, regen, splits, fly, stealth, heal, group, boss + leak,
  //   intro = Welle, ab der der Typ auftaucht. r = Radius in Zellen.
  const ENEMIES = {
    runt: { hp: 22, spd: 1.9, bounty: 4, score: 10, r: 0.26, modes: "cCM", intro: 1, col: [235, 120, 90] },
    fast: { hp: 14, spd: 3.2, bounty: 5, score: 12, r: 0.22, modes: "cCM", intro: 2, col: [255, 180, 80] },
    swarm: { hp: 8, spd: 2.4, bounty: 2, score: 6, r: 0.17, modes: "cCM", intro: 4, group: 6, col: [250, 220, 110] },
    tank: { hp: 95, spd: 1.05, bounty: 9, score: 18, r: 0.34, modes: "cCM", intro: 5, col: [185, 100, 145] },
    shield: { hp: 60, spd: 1.5, bounty: 8, score: 16, r: 0.29, modes: "CM", intro: 6, armor: 6, col: [150, 160, 220] },
    flyer: { hp: 40, spd: 2.2, bounty: 8, score: 16, r: 0.25, modes: "M", intro: 7, fly: true, col: [160, 205, 255] },
    regen: { hp: 70, spd: 1.4, bounty: 8, score: 16, r: 0.29, modes: "CM", intro: 9, regen: 4, col: [130, 215, 140] },
    stealth: { hp: 55, spd: 1.7, bounty: 9, score: 18, r: 0.27, modes: "M", intro: 10, stealth: true, col: [175, 175, 195] },
    splitter: { hp: 48, spd: 1.6, bounty: 6, score: 14, r: 0.29, modes: "CM", intro: 11, splits: 2, col: [230, 150, 200] },
    healer: { hp: 65, spd: 1.3, bounty: 10, score: 18, r: 0.29, modes: "M", intro: 12, heal: 6, col: [120, 230, 200] },
    boss: { hp: 900, spd: 0.85, bounty: 60, score: 100, r: 0.5, modes: "cCM", intro: 0, boss: true, leak: 5, col: [255, 90, 110] },
  };

  // Karten: Wegpunkte in Zell-Koordinaten (außerhalb 0..17/0..11 = Feldrand),
  // diff = Gegner-HP-Faktor, unlock = beste Welle (kartenübergreifend), ab der
  // die Karte spielbar ist. builds="ring" = nur direkt neben dem Pfad baubar.
  const MAPS = [
    { id: "meadow", diff: 1.0, unlock: 0, path: [[-1, 3], [13, 3], [13, 7], [4, 7], [4, 10], [18, 10]] },
    { id: "canyon", diff: 1.7, unlock: 5, path: [[-1, 1], [16, 1], [16, 4], [1, 4], [1, 7], [16, 7], [16, 10], [-1, 10]] },
    { id: "crossing", diff: 1.85, unlock: 10, path: [[-1, 6], [8, 6], [8, 2], [14, 2], [14, 9], [8, 9], [8, 6], [18, 6]] },
    { id: "gauntlet", diff: 1.2, unlock: 15, builds: "ring", path: [[-1, 5], [8, 5], [8, 8], [18, 8]] },
  ];

  const ABIL_CDS = { meteor: 45.0, nova: 60.0, gold: 90.0 };

  // ----- Endlos-Skalierung ---------------------------------------------------

  /** Gegner-Budget der Welle n (Punkte, die der Komponist ausgeben darf). */
  const budgetOf = (n) => 45 + 12 * n + 1.6 * n * n;

  /** HP-Faktor der Welle n; ab Welle 25 zusätzlich exponentiell. */
  function hpMult(n) {
    let m = 1.0 + 0.14 * (n - 1) + 0.012 * (n - 1) ** 2;
    if (n > 25) m *= 1.06 ** (n - 25);
    return m;
  }

  /** Gold-Inflationsbremse: Beute sinkt langsam mit der Wellen-Nummer. */
  const bountyMult = (n) => Math.max(0.35, 1.0 - 0.018 * (n - 1));

  /** Karten-HP-Faktor mit Anlaufkurve: greift erst ab Welle 10 voll. */
  const mapHpf = (n, diff) => 1.0 + (diff - 1.0) * Math.min(1.0, n / 10.0);

  /** Budget-Kosten eines Gegnertyps (aus Basiswerten abgeleitet). */
  const unitCost = (d) => d.hp / 10.0 + d.spd * 4.0;

  /** Python-round (kaufmännisch auf gerade bei .5). */
  function pyRound(x) {
    const f = Math.floor(x);
    const diff = x - f;
    if (diff === 0.5) return f % 2 === 0 ? f : f + 1;
    return Math.round(x);
  }

  const ckey = (c, r) => c + "," + r;

  class LamaTowerDefenseGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.animT = 0;

      this.rules = MODE_RULES[this.mode] || MODE_RULES.classic;
      this.mletter = this.rules.letter;
      this.barKeys = Object.keys(TOWERS).filter((k) => TOWERS[k].modes.includes(this.mletter));

      const saved = PG.store.get("lamatowerdef", {});
      this.best = {};
      const sb = saved && typeof saved === "object" ? saved.best : null;
      if (sb && typeof sb === "object") {
        for (const k in sb) if (typeof sb[k] === "number") this.best[k] = Math.trunc(sb[k]);
      }

      // Leere Lauf-Variablen, damit draw() im Kartenwahl-Zustand läuft.
      this.mapIdx = 0;
      this.towers = new Map(); // "c,r" -> Turm
      this.enemies = [];
      this.shots = [];
      this.fx = [];
      this.spawnQueue = [];
      this.spawnT = 0;
      this.hpBonus = 1.0;
      this.wave = 1;
      this.gold = 0;
      this.lives = 0;
      this.kills = 0;
      this.ff = 1;
      this.showRanges = false;
      this.selCard = null; // Turm-Schlüssel im Baumenü (Platzieren)
      this.selTower = null; // ausgewählter platzierter Turm ("c,r")
      this.armed = null; // gewappnete Fähigkeit ("meteor")
      this.abilCd = { meteor: 0, nova: 0, gold: 0 };
      this.mapselIdx = 0;
      this.mouse = null;
      this.barScroll = 0;
      this.overT = 0;
      this.speedRect = null;

      this.bgCache = null;
      this.layout();
      this.state = MAPSEL;
    }

    get wantsRightClick() {
      return true;
    }

    makeFonts() {
      const h = this.height;
      this.font = ui.font(Math.max(18, Math.min(26, Math.floor(h / 26))));
      this.bigFont = ui.font(Math.max(30, Math.min(50, Math.floor(h / 13))), true);
      this.small = ui.font(Math.max(13, Math.min(19, Math.floor(h / 34))));
      this.tiny = ui.font(Math.max(11, Math.min(15, Math.floor(h / 44))));
    }

    /** Alle Maße aus width/height ableiten. */
    layout() {
      this.makeFonts();
      this.hudH = Math.max(34, Math.floor(this.height / 13));
      this.barW = Math.max(148, Math.floor(this.width * 0.2));
      const availW = this.width - this.barW;
      const availH = this.height - this.hudH;
      this.cell = Math.max(10, Math.min(Math.floor(availW / GRID_W), Math.floor(availH / GRID_H)));
      this.ox = Math.floor((availW - this.cell * GRID_W) / 2);
      this.oy = this.hudH + Math.floor((availH - this.cell * GRID_H) / 2);
      this.fieldRect = new PG.Rect(this.ox, this.oy, this.cell * GRID_W, this.cell * GRID_H);
      this.barRect = new PG.Rect(this.width - this.barW, 0, this.barW, this.height);
      this.buildBarLayout();
      this.buildMapselLayout();
      this.setupPath(MAPS[this.mapIdx]);
      if (this.mouse === null) this.mouse = this.fieldRect.center;
    }

    /** Rechteck-Raster der Seitenleiste (Karten, Buttons). */
    buildBarLayout() {
      const bx = this.barRect.x + 10;
      const bw = this.barW - 20;
      let y = 12;
      this.abilRects = {};
      if (this.rules.abil) {
        const aw = Math.floor((bw - 12) / 3);
        ["meteor", "nova", "gold"].forEach((key, i) => {
          this.abilRects[key] = new PG.Rect(bx + i * (aw + 6), y, aw, Math.max(30, this.cell));
        });
        y += Math.max(30, this.cell) + 10;
      }
      // Turmkarten: 2 Spalten
      const cw = Math.floor((bw - 8) / 2);
      const ch = Math.max(44, Math.floor(this.cell * 1.35));
      this.cardRects = this.barKeys.map((key, i) => new PG.Rect(bx + (i % 2) * (cw + 8), y + Math.floor(i / 2) * (ch + 8), cw, ch));
      this.cardsTop = y;
      const rows = Math.floor((this.barKeys.length + 1) / 2);
      this.cardsH = rows * (ch + 8);
      // Info-Bereich + Start-Button unten
      const bh = Math.max(40, Math.floor(this.height * 0.07));
      this.startRect = new PG.Rect(bx, this.height - bh - 12, bw, bh);
      const ih = Math.max(120, Math.floor(this.height * 0.3));
      this.infoRect = new PG.Rect(bx, this.startRect.y - ih - 10, bw, ih);
      this.cardsViewH = Math.max(40, this.infoRect.y - y - 8);
      // Buttons im Info-Bereich (Ausbau/Verkauf/Zweige)
      const byy = this.infoRect.bottom - 34;
      const half = Math.floor((bw - 6) / 2);
      this.upRect = new PG.Rect(bx, byy - 38, bw, 32);
      this.sellRect = new PG.Rect(bx, byy, bw, 30);
      this.brARect = new PG.Rect(bx, byy - 38, half, 32);
      this.brBRect = new PG.Rect(bx + half + 6, byy - 38, half, 32);
    }

    buildMapselLayout() {
      const cw = Math.min(300, Math.floor((this.width - 80) / 2));
      const ch = Math.min(190, Math.floor((this.height - 170) / 2));
      const cx = Math.floor(this.width / 2);
      const y0 = Math.max(120, Math.floor(this.height * 0.24));
      this.mapRects = MAPS.map((m, i) => new PG.Rect(cx - cw - 14 + (i % 2) * (cw + 28), y0 + Math.floor(i / 2) * (ch + 22), cw, ch));
    }

    // ----- Pfad-Geometrie ------------------------------------------------
    /** Wegpunkte der Karte in Pfad-Polylinie + Zellmengen übersetzen. */
    setupPath(m) {
      const wps = m.path;
      this.pathPts = wps.map(([c, r]) => [c + 0.5, r + 0.5]);
      this.pathCum = [0];
      let total = 0;
      for (let i = 1; i < this.pathPts.length; i++) {
        const [x0, y0] = this.pathPts[i - 1], [x1, y1] = this.pathPts[i];
        total += Math.abs(x1 - x0) + Math.abs(y1 - y0);
        this.pathCum.push(total);
      }
      this.pathTotal = total;
      // Luftpfad: gerade Linie Start -> Ziel (für Flieger)
      this.flyA = this.pathPts[0];
      this.flyB = this.pathPts[this.pathPts.length - 1];
      this.flyTotal = Math.hypot(this.flyB[0] - this.flyA[0], this.flyB[1] - this.flyA[1]);
      // Pfad-Zellen (gesperrt fürs Bauen)
      const cells = new Set();
      for (let i = 1; i < wps.length; i++) {
        const [c0, r0] = wps[i - 1], [c1, r1] = wps[i];
        if (c0 === c1) {
          for (let r = Math.min(r0, r1); r <= Math.max(r0, r1); r++) cells.add(ckey(c0, r));
        } else {
          for (let c = Math.min(c0, c1); c <= Math.max(c0, c1); c++) cells.add(ckey(c, r0));
        }
      }
      this.pathCells = new Set();
      for (const k of cells) {
        const [c, r] = k.split(",").map(Number);
        if (c >= 0 && c < GRID_W && r >= 0 && r < GRID_H) this.pathCells.add(k);
      }
      // Baubare Zellen ("ring" = nur direkte Pfad-Nachbarn)
      if (m.builds === "ring") {
        this.buildCells = new Set();
        for (const k of this.pathCells) {
          const [c, r] = k.split(",").map(Number);
          for (const dc of [-1, 0, 1]) {
            for (const dr of [-1, 0, 1]) {
              const cc = c + dc, rr = r + dr;
              const kk = ckey(cc, rr);
              if (!this.pathCells.has(kk) && cc >= 0 && cc < GRID_W && rr >= 0 && rr < GRID_H) this.buildCells.add(kk);
            }
          }
        }
      } else {
        this.buildCells = null; // null = überall außer Pfad
      }
    }

    /** Position (Zell-Koordinaten) bei Pfad-Distanz d. */
    posAt(d, fly = false) {
      if (fly) {
        const f = Math.max(0, Math.min(1, d / Math.max(0.001, this.flyTotal)));
        return [this.flyA[0] + (this.flyB[0] - this.flyA[0]) * f, this.flyA[1] + (this.flyB[1] - this.flyA[1]) * f];
      }
      const pts = this.pathPts, cum = this.pathCum;
      if (d <= 0) return pts[0];
      if (d >= cum[cum.length - 1]) return pts[pts.length - 1];
      for (let i = 1; i < cum.length; i++) {
        if (d <= cum[i]) {
          const f = (d - cum[i - 1]) / Math.max(0.001, cum[i] - cum[i - 1]);
          const [x0, y0] = pts[i - 1], [x1, y1] = pts[i];
          return [x0 + (x1 - x0) * f, y0 + (y1 - y0) * f];
        }
      }
      return pts[pts.length - 1];
    }

    /** Zell-Koordinaten -> Pixel. */
    px(cx, cy) {
      return [this.ox + cx * this.cell, this.oy + cy * this.cell];
    }

    // ===================================================== Lauf-Verwaltung
    unlocked(m) {
      const vals = Object.values(this.best);
      const top = vals.length ? Math.max(...vals) : 0;
      return m.unlock <= top;
    }

    startRun(idx) {
      this.mapIdx = idx;
      this.setupPath(MAPS[idx]);
      this.bgCache = null;
      this.towers = new Map();
      this.enemies = [];
      this.shots = [];
      this.fx = [];
      this.spawnQueue = [];
      this.hpBonus = 1.0;
      this.score = 0;
      this.kills = 0;
      this.wave = 1;
      this.gold = this.rules.gold;
      this.lives = this.rules.lives;
      this.ff = 1;
      this.showRanges = false;
      this.selCard = null;
      this.selTower = null;
      this.armed = null;
      this.abilCd = { meteor: 0, nova: 0, gold: 0 };
      this.barScroll = 0;
      this.gameOver = false;
      this.state = BUILD;
      this.playSound("select");
    }

    saveBest() {
      const cleared = this.wave - 1;
      const mid = MAPS[this.mapIdx].id;
      if (cleared > (this.best[mid] || 0)) {
        this.best[mid] = cleared;
        PG.store.set("lamatowerdef", { best: this.best });
      }
    }

    // ===================================================== Wellen-Komponist
    /** Spawn-Liste [[Gegnertyp, Abstand_s], ...] für Welle n bauen. */
    composeWave(n) {
      let budget = budgetOf(n);
      const gap = Math.max(0.28, 0.9 - 0.012 * n);
      const queue = [];

      let avail = Object.keys(ENEMIES).filter((k) => {
        const d = ENEMIES[k];
        return d.modes.includes(this.mletter) && !d.boss && d.intro <= n;
      });
      const bossWave = n % this.rules.boss === 0;

      if (bossWave) {
        for (let i = 0; i < 1 + Math.floor(n / 32); i++) queue.push(["boss", 1.2]);
        budget *= 0.45; // Rest als Eskorte
      } else if (n % 5 === 0 && n > 4) {
        // Themen-Welle: nur ein Typ (rotiert durch die Freischaltungen)
        avail = [avail[Math.floor(n / 5) % avail.length]];
      }

      let spent = 0;
      let units = queue.length;
      while (spent < budget && units < UNITS_PER_WAVE) {
        // neuere Typen leicht bevorzugen
        const k = PG.rand.random() < 0.45 ? PG.rand.choice(avail.slice(-3)) : PG.rand.choice(avail);
        const d = ENEMIES[k];
        const group = d.group || 1;
        for (let i = 0; i < group; i++) queue.push([k, i ? 0.12 : gap]);
        spent += unitCost(d) * group;
        units += group;
      }

      // Restbudget (Einheiten-Deckel erreicht) fließt in Bonus-HP.
      const hpBonus = 1.0 + Math.max(0, (budget - spent) / Math.max(1, budget));
      return [queue, hpBonus];
    }

    startWave() {
      if (this.state !== BUILD) return;
      [this.spawnQueue, this.hpBonus] = this.composeWave(this.wave);
      this.spawnT = 0.5;
      this.state = WAVE;
      this.playSound("select");
    }

    spawn(kind) {
      if (this.enemies.length >= MAX_ENEMIES) {
        this.spawnT += 0.4; // warten, bis wieder Platz ist
        this.spawnQueue.unshift([kind, 0]);
        return;
      }
      const d = ENEMIES[kind];
      const hp = d.hp * hpMult(this.wave) * mapHpf(this.wave, MAPS[this.mapIdx].diff) * this.hpBonus;
      this.enemies.push({
        kind, hp, maxhp: hp, d: 0, spd: d.spd, r: d.r,
        armor: d.armor || 0, regen: d.regen || 0,
        fly: !!d.fly, stealth: !!d.stealth,
        heal: d.heal || 0, splits: d.splits || 0,
        boss: !!d.boss, leak: d.leak || 1,
        bounty: d.bounty, score: d.score, col: d.col,
        slow_t: 0, slow_f: 0, stun_t: 0, dot_t: 0, dot: 0,
        seen: !d.stealth, plague: false, dead: false,
      });
    }

    // ===================================================== Turm-Verwaltung
    /** Wirkwerte eines Turms aus Basis, Stufe, Zweig und Auren. */
    towerStats(tw) {
      const base = TOWERS[tw.kind];
      const lvl = tw.level;
      const br = tw.branch;
      const st = {
        dmg: (base.dmg || 0) * 1.55 ** (lvl - 1),
        rng: (base.rng || 0) * 1.12 ** (lvl - 1),
        cd: (base.cd || 0) * 0.88 ** (lvl - 1),
        splash: base.splash || 0,
        slow: base.slow || 0, slow_t: base.slow_t || 0,
        dot: (base.dot || 0) * 1.55 ** (lvl - 1),
        dot_t: base.dot_t || 0,
        chain: base.chain || 0,
        buff: (base.buff || 0) + 0.05 * (lvl - 1),
        income: (base.income || 0) * 1.5 ** (lvl - 1),
        air: !!base.air,
        air_only: !!base.air_only,
        targets: 1, execute: 0, stun: 0, aura_dps: 0,
        ramp_max: 2.5, ramp_rate: 0.35, splits_dot: false,
        acid: false, ground_frac: 0, cluster: 1,
        buff_cd: false,
      };
      const k = tw.kind;
      if (br === "a") {
        if (k === "arrow") { st.targets = 2; st.air = true; }
        else if (k === "cannon") { st.splash *= 1.6; st.stun = 0.3; }
        else if (k === "frost") st.stun = 0.35;
        else if (k === "sniper") st.execute = 0.2;
        else if (k === "poison") st.splits_dot = true;
        else if (k === "tesla") st.chain = 5;
        else if (k === "support") st.buff += 0.15;
        else if (k === "mortar") st.cluster = 3;
        else if (k === "flak") st.splash = 0.9;
        else if (k === "laser") st.targets = 3;
        else if (k === "bank") st.income *= 1.8;
      } else if (br === "b") {
        if (k === "arrow") st.dmg *= 1.85;
        else if (k === "cannon") { st.dot = 10; st.dot_t = 2.5; }
        else if (k === "frost") st.aura_dps = 10;
        else if (k === "sniper") st.cd *= 0.55;
        else if (k === "poison") st.acid = true;
        else if (k === "tesla") st.dmg *= 1.8;
        else if (k === "support") st.buff_cd = true;
        else if (k === "mortar") st.dmg *= 2.0;
        else if (k === "flak") { st.ground_frac = 0.6; st.air_only = false; }
        else if (k === "laser") { st.ramp_max = 3.5; st.ramp_rate = 0.7; }
        else if (k === "bank") st.income *= 1.2;
      }
      // Unterstützungs-Auren einrechnen
      st.dmg *= tw.buff_dmg != null ? tw.buff_dmg : 1.0;
      st.cd *= tw.buff_cd_f != null ? tw.buff_cd_f : 1.0;
      return st;
    }

    /** Auren neu verteilen und Wirkwerte aller Türme neu berechnen. */
    refreshTowers() {
      const sups = [];
      for (const tw of this.towers.values()) {
        if (tw.kind === "support") {
          tw.buff_dmg = 1.0;
          tw.buff_cd_f = 1.0;
          tw.st = this.towerStats(tw);
          sups.push(tw);
        }
      }
      for (const tw of this.towers.values()) {
        if (tw.kind === "support") continue;
        let dmgF = 1.0, cdF = 1.0;
        const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
        for (const sp of sups) {
          const sx = sp.cell[0] + 0.5, sy = sp.cell[1] + 0.5;
          if ((cx - sx) ** 2 + (cy - sy) ** 2 <= sp.st.rng ** 2) {
            dmgF += sp.st.buff;
            if (sp.st.buff_cd) cdF = Math.min(cdF, 0.85);
          }
        }
        tw.buff_dmg = Math.min(1.6, dmgF);
        tw.buff_cd_f = cdF;
        tw.st = this.towerStats(tw);
      }
    }

    canBuild(cell) {
      const [c, r] = cell;
      if (!(c >= 0 && c < GRID_W && r >= 0 && r < GRID_H)) return false;
      const k = ckey(c, r);
      if (this.pathCells.has(k) || this.towers.has(k)) return false;
      if (this.buildCells !== null && !this.buildCells.has(k)) return false;
      return true;
    }

    placeTower(cell) {
      const base = TOWERS[this.selCard];
      if (this.gold < base.cost) {
        this.playSound("hit");
        return;
      }
      this.gold -= base.cost;
      this.towers.set(ckey(cell[0], cell[1]), {
        kind: this.selCard, cell: [cell[0], cell[1]], level: 1,
        branch: null, invested: base.cost,
        cd_left: 0, angle: 0, ramp: 1.0,
        beam: null, buff_dmg: 1.0, buff_cd_f: 1.0, st: null,
      });
      this.refreshTowers();
      this.playSound("lock");
      if (this.gold < base.cost) this.selCard = null; // kann keinen weiteren bezahlen
    }

    upgradeCost(tw) {
      return Math.trunc(TOWERS[tw.kind].cost * 0.8 * tw.level);
    }

    branchCost(tw) {
      return Math.trunc(TOWERS[tw.kind].cost * BRANCH_COST);
    }

    sellValue(tw) {
      return Math.trunc(tw.invested * 0.7);
    }

    upgradeTower(tw) {
      if (tw.level >= this.rules.levels || tw.branch) return;
      const cost = this.upgradeCost(tw);
      if (this.gold < cost) {
        this.playSound("hit");
        return;
      }
      this.gold -= cost;
      tw.level += 1;
      tw.invested += cost;
      this.refreshTowers();
      this.playSound("powerup");
      if (tw.level >= this.rules.levels && !this.rules.branch) this.achEvent("td_maxed");
    }

    branchTower(tw, which) {
      if (!this.rules.branch || tw.branch || tw.level < this.rules.levels) return;
      const cost = this.branchCost(tw);
      if (this.gold < cost) {
        this.playSound("hit");
        return;
      }
      this.gold -= cost;
      tw.branch = which;
      tw.invested += cost;
      if (tw.kind === "bank" && which === "b") this.gold += 200; // Dividende: Sofort-Auszahlung
      this.refreshTowers();
      this.playSound("level");
      this.achEvent("td_maxed");
    }

    sellTower(key) {
      const tw = this.towers.get(key);
      if (!tw) return;
      this.towers.delete(key);
      this.gold += this.sellValue(tw);
      this.selTower = null;
      this.refreshTowers();
      this.playSound("eat");
    }

    // ===================================================== Simulation
    update(dt) {
      this.animT += dt;
      if (this.state === MAPSEL || this.state === GAMEOVER) {
        this.ageFx(dt);
        return;
      }
      for (let i = 0; i < this.ff; i++) {
        let rest = dt;
        while (rest > 1e-9 && !this.gameOver) {
          const h = Math.min(rest, 0.04);
          this.sim(h);
          rest -= h;
        }
      }
      this.ageFx(dt);
    }

    sim(h) {
      // Fähigkeiten-Abklingzeiten
      for (const k in this.abilCd) this.abilCd[k] = Math.max(0, this.abilCd[k] - h);

      if (this.state === WAVE && this.spawnQueue.length) {
        this.spawnT -= h;
        while (this.spawnQueue.length && this.spawnT <= 0) {
          const [kind, gap] = this.spawnQueue.shift();
          this.spawn(kind);
          this.spawnT += gap;
        }
      }

      this.simEnemies(h);
      this.simTowers(h);
      this.simShots(h);

      if (this.state === WAVE && !this.spawnQueue.length && !this.enemies.length) this.waveCleared();
    }

    simEnemies(h) {
      const detect = [];
      const auraFrost = [];
      for (const tw of this.towers.values()) {
        const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
        if (TOWERS[tw.kind].detect) detect.push([cx, cy, tw.st.rng]);
        if (tw.st.aura_dps > 0) auraFrost.push([cx, cy, tw.st.rng, tw.st.aura_dps]);
      }
      const healers = this.enemies.filter((e) => e.heal && e.hp > 0);

      let leaked = false;
      // for..of sieht (wie die Python-Schleife) auch neu angehängte Teilungs-Kinder
      for (const e of this.enemies) {
        if (e.hp <= 0) continue;
        // Status-Effekte
        if (e.dot_t > 0) {
          e.dot_t -= h;
          e.hp -= e.dot * h;
          if (e.hp <= 0) {
            this.kill(e);
            continue;
          }
        }
        if (e.regen) e.hp = Math.min(e.maxhp, e.hp + e.regen * h);
        const [ex, ey] = this.posAt(e.d, e.fly);
        for (const [ax, ay, rng, dps] of auraFrost) {
          if ((ex - ax) ** 2 + (ey - ay) ** 2 <= rng * rng) e.hp -= dps * h;
        }
        if (e.hp <= 0) {
          this.kill(e);
          continue;
        }
        if (e.stealth) e.seen = detect.some(([ax, ay, rng]) => (ex - ax) ** 2 + (ey - ay) ** 2 <= rng * rng);
        // Bewegung
        let v = e.spd;
        if (e.slow_t > 0) {
          e.slow_t -= h;
          v *= 1.0 - e.slow_f;
        }
        if (e.stun_t > 0) {
          e.stun_t -= h;
          v = 0;
        }
        e.d += v * h;
        const total = e.fly ? this.flyTotal : this.pathTotal;
        if (e.d >= total) {
          e.hp = 0;
          this.lives -= e.leak;
          leaked = true;
        }
      }

      // Heiler-Auren (nach der Bewegung, auf lebende Nachbarn)
      for (const he of healers) {
        if (he.hp <= 0) continue;
        const [hx, hy] = this.posAt(he.d, he.fly);
        for (const e of this.enemies) {
          if (e === he || e.hp <= 0) continue;
          const [ex, ey] = this.posAt(e.d, e.fly);
          if ((ex - hx) ** 2 + (ey - hy) ** 2 <= 1.5 ** 2) e.hp = Math.min(e.maxhp, e.hp + he.heal * h);
        }
      }

      this.enemies = this.enemies.filter((e) => e.hp > 0);
      if (leaked) {
        this.playSound("hit");
        this.rumble(150);
        if (this.lives <= 0) this.finish();
      }
    }

    /** Darf ein Turm mit Werten st den Gegner e anvisieren? */
    visible(e, st) {
      if (e.hp <= 0 || !e.seen) return false;
      if (e.fly) return st.air;
      return !st.air_only || st.ground_frac > 0;
    }

    findTarget(tw, st) {
      const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
      const rng2 = st.rng ** 2;
      const min2 = (TOWERS[tw.kind].min_rng || 0) ** 2;
      let best = null, bestV = -1.0;
      const strong = TOWERS[tw.kind].prio === "strong";
      for (const e of this.enemies) {
        if (!this.visible(e, st)) continue;
        const [ex, ey] = this.posAt(e.d, e.fly);
        const d2 = (ex - cx) ** 2 + (ey - cy) ** 2;
        if (d2 > rng2 || d2 < min2) continue;
        const v = strong ? e.hp : e.d + (e.boss ? 1000 : 0);
        if (v > bestV) {
          best = e;
          bestV = v;
        }
      }
      return best;
    }

    simTowers(h) {
      for (const tw of this.towers.values()) {
        const st = tw.st;
        const base = TOWERS[tw.kind];
        if (base.income || tw.kind === "support") continue;
        const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
        if (base.beam) {
          this.simLaser(tw, st, h);
          continue;
        }
        tw.cd_left -= h;
        if (tw.cd_left > 0) continue;
        const first = this.findTarget(tw, st);
        if (first === null) continue;
        const targets = [first];
        if (st.targets > 1) {
          // Doppelschuss: zweites Ziel suchen
          for (const e of this.enemies) {
            if (e === first || !this.visible(e, st)) continue;
            const [ex, ey] = this.posAt(e.d, e.fly);
            if ((ex - cx) ** 2 + (ey - cy) ** 2 <= st.rng ** 2) {
              targets.push(e);
              if (targets.length >= st.targets) break;
            }
          }
        }
        for (const e of targets) this.fire(tw, st, e);
        const [ex, ey] = this.posAt(first.d, first.fly);
        tw.angle = Math.atan2(ey - cy, ex - cx);
        tw.cd_left = st.cd;
      }
    }

    simLaser(tw, st, h) {
      let e = tw.beam;
      const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
      if (e !== null) {
        const [ex, ey] = this.posAt(e.d, e.fly);
        if (e.hp <= 0 || !this.visible(e, st) || (ex - cx) ** 2 + (ey - cy) ** 2 > st.rng ** 2) {
          e = null;
          tw.ramp = 1.0;
        }
      }
      if (e === null) {
        e = this.findTarget(tw, st);
        tw.ramp = 1.0;
      }
      tw.beam = e;
      if (e === null) return;
      tw.ramp = Math.min(st.ramp_max, tw.ramp + st.ramp_rate * h);
      const victims = [e];
      if (st.targets > 1) {
        // Prisma: Strahl teilt sich
        for (const o of this.enemies) {
          if (o === e || !this.visible(o, st)) continue;
          const [ox, oy] = this.posAt(o.d, o.fly);
          if ((ox - cx) ** 2 + (oy - cy) ** 2 <= st.rng ** 2) {
            victims.push(o);
            if (victims.length >= st.targets) break;
          }
        }
      }
      victims.forEach((v, i) => {
        const frac = i === 0 ? 1.0 : 0.5;
        this.damage(v, st.dmg * tw.ramp * h * frac);
      });
      const [ex, ey] = this.posAt(e.d, e.fly);
      tw.angle = Math.atan2(ey - cy, ex - cx);
    }

    fire(tw, st, e) {
      const base = TOWERS[tw.kind];
      const cx = tw.cell[0] + 0.5, cy = tw.cell[1] + 0.5;
      const [ex, ey] = this.posAt(e.d, e.fly);
      let dmg = st.dmg;
      if (e.fly && st.ground_frac === 0 && base.air_only) {
        // reine Luftabwehr gegen Flieger: voller Schaden
      } else if (!e.fly && st.ground_frac > 0) {
        dmg *= st.ground_frac; // Flak-Zielcomputer: Boden schwächer
      }
      if (base.hitscan) {
        this.addFx("tracer", { a: [cx, cy], b: [ex, ey], col: base.col, ttl: 0.12 });
        if (st.execute && e.hp / e.maxhp < st.execute && !e.boss) {
          e.hp = 0;
          this.kill(e);
        } else {
          this.damage(e, dmg, !!base.pierce);
        }
        this.playSound("shoot");
        return;
      }
      if (st.chain) {
        const pts = [[cx, cy]];
        let victim = e, cd = dmg;
        const hit = new Set();
        for (let i = 0; i < st.chain; i++) {
          const [vx, vy] = this.posAt(victim.d, victim.fly);
          pts.push([vx, vy]);
          this.damage(victim, cd);
          hit.add(victim);
          cd *= 0.65;
          let nxt = null;
          let nd = 1.5 ** 2;
          for (const o of this.enemies) {
            if (hit.has(o) || !this.visible(o, st)) continue;
            const [ox, oy] = this.posAt(o.d, o.fly);
            const d2 = (ox - vx) ** 2 + (oy - vy) ** 2;
            if (d2 <= nd) {
              nxt = o;
              nd = d2;
            }
          }
          if (nxt === null) break;
          victim = nxt;
        }
        this.addFx("zigzag", { pts, col: base.col, ttl: 0.15 });
        return;
      }
      if (base.lob) {
        const dist = Math.hypot(ex - cx, ey - cy);
        const flyT = Math.max(0.35, Math.min(1.2, dist / 6.0));
        const aim = this.posAt(e.d + e.spd * flyT * 0.7, e.fly);
        const n = st.cluster;
        for (let i = 0; i < n; i++) {
          const jx = i ? PG.rand.uniform(-0.5, 0.5) : 0;
          const jy = i ? PG.rand.uniform(-0.5, 0.5) : 0;
          this.shots.push({
            kind: "lob", x: cx, y: cy, tx: aim[0] + jx, ty: aim[1] + jy,
            t: 0, dur: flyT, dmg: dmg / (n > 1 ? 1.6 : 1.0),
            splash: st.splash * (n > 1 ? 0.75 : 1.0),
            stun: st.stun, col: base.col,
          });
        }
        this.playSound("shoot");
        return;
      }
      if (this.shots.length < MAX_SHOTS) {
        this.shots.push({
          kind: "proj", x: cx, y: cy, tgt: e, lx: ex, ly: ey,
          spd: base.proj || 9.0, dmg, splash: st.splash,
          slow: st.slow, slow_t: st.slow_t,
          dot: st.dot, dot_t: st.dot_t, stun: st.stun,
          acid: st.acid, splits_dot: st.splits_dot,
          air: st.air_only, // reine Luftabwehr: Explosion in der Luft
          col: base.col,
        });
      }
    }

    simShots(h) {
      const rest = [];
      for (const s of this.shots) {
        if (s.kind === "lob") {
          s.t += h;
          if (s.t >= s.dur) this.explode([s.tx, s.ty], s.dmg, s.splash, s.stun);
          else rest.push(s);
          continue;
        }
        let e = s.tgt;
        if (e !== null && e.hp > 0) {
          [s.lx, s.ly] = this.posAt(e.d, e.fly);
        } else {
          s.tgt = e = null;
        }
        const dx = s.lx - s.x, dy = s.ly - s.y;
        const dist = Math.hypot(dx, dy);
        const step = s.spd * h;
        if (dist <= Math.max(step, 0.22)) {
          if (e !== null) this.impact(s, e);
          else if (s.splash) this.explode([s.lx, s.ly], s.dmg, s.splash, s.stun, s);
          continue;
        }
        s.x += (dx / dist) * step;
        s.y += (dy / dist) * step;
        rest.push(s);
      }
      this.shots = rest;
    }

    impact(s, e) {
      if (s.splash) {
        this.explode([s.lx, s.ly], s.dmg, s.splash, s.stun, s);
      } else {
        if (s.acid) e.armor = 0;
        if (s.slow) {
          e.slow_f = Math.max(e.slow_f, s.slow);
          e.slow_t = Math.max(e.slow_t, s.slow_t);
        }
        if (s.stun) e.stun_t = Math.max(e.stun_t, s.stun);
        if (s.dot) {
          e.dot = Math.max(e.dot, s.dot);
          e.dot_t = Math.max(e.dot_t, s.dot_t);
          e.plague = s.splits_dot;
        }
        this.damage(e, s.dmg);
      }
    }

    /**
     * Flächenschaden. shot (optional) = auslösendes Projektil:
     * - shot.air (Sprengflak, Flak A): zerplatzt in der Luft und trifft nur
     *   Flieger - sonst treffen Explosionen nur Bodengegner. Im Python-Original
     *   übersprang die Explosion Flieger immer, "Sprengflak" traf also nie.
     * - shot.dot (Brandladung, Kanone B): Dauerschaden auf alle Getroffenen -
     *   im Original ging er bei Flächenschüssen verloren.
     */
    explode(pos, dmg, radius, stun = 0, shot = null) {
      const [px, py] = pos;
      const air = !!(shot && shot.air);
      for (const e of this.enemies.slice()) {
        if (e.hp <= 0 || e.fly !== air) continue;
        const [ex, ey] = this.posAt(e.d, e.fly);
        if ((ex - px) ** 2 + (ey - py) ** 2 <= radius * radius) {
          if (stun) e.stun_t = Math.max(e.stun_t, stun);
          if (shot && shot.dot) {
            e.dot = Math.max(e.dot, shot.dot);
            e.dot_t = Math.max(e.dot_t, shot.dot_t);
          }
          this.damage(e, dmg);
        }
      }
      this.addFx("ring", { pos, r: radius, col: [255, 190, 120], ttl: 0.25 });
    }

    damage(e, dmg, pierce = false) {
      if (e.hp <= 0) return;
      if (e.armor && !pierce) dmg = Math.max(1.0, dmg - e.armor);
      e.hp -= dmg;
      if (e.hp <= 0) this.kill(e);
    }

    kill(e) {
      if (e.dead) return;
      e.dead = true;
      e.hp = 0;
      const n = this.wave;
      this.gold += Math.max(1, pyRound(e.bounty * bountyMult(n)));
      this.score += e.score;
      this.kills += 1;
      const [ex, ey] = this.posAt(e.d, e.fly);
      this.addFx("burst", { pos: [ex, ey], col: e.col, ttl: 0.4 });
      if (e.boss) {
        this.playSound("explode");
        this.achEvent("td_boss");
        const [px, py] = this.px(ex, ey);
        ui.spawnBurst(px, py, e.col, 24);
      }
      if (e.splits) {
        for (let i = 0; i < e.splits; i++) {
          if (this.enemies.length < MAX_ENEMIES) {
            const d = ENEMIES.runt;
            const hp = d.hp * hpMult(n) * 0.6 * mapHpf(n, MAPS[this.mapIdx].diff);
            this.enemies.push({
              kind: "runt", hp, maxhp: hp,
              d: e.d - PG.rand.uniform(0, 0.5),
              spd: d.spd, r: d.r, armor: 0, regen: 0, fly: false,
              stealth: false, heal: 0, splits: 0, boss: false, leak: 1,
              bounty: d.bounty, score: d.score, col: d.col,
              slow_t: 0, slow_f: 0, stun_t: 0, dot_t: 0,
              dot: 0, seen: true, plague: false, dead: false,
            });
          }
        }
      }
      if (e.plague && e.dot) {
        for (const o of this.enemies) {
          if (o === e || o.hp <= 0) continue;
          const [ox, oy] = this.posAt(o.d, o.fly);
          if ((ox - ex) ** 2 + (oy - ey) ** 2 <= 1.2 ** 2) {
            o.dot = Math.max(o.dot, e.dot);
            o.dot_t = Math.max(o.dot_t, e.dot_t);
          }
        }
      }
    }

    waveCleared() {
      const n = this.wave;
      const diff = MAPS[this.mapIdx].diff;
      let income = 0;
      for (const tw of this.towers.values()) if (tw.st.income) income += tw.st.income;
      this.gold += 20 + 4 * n + Math.trunc(income);
      this.gold += Math.min(50, Math.trunc(this.gold * 0.05)); // Zinsen
      this.score += Math.trunc((20 + 5 * n) * diff);
      this.playSound("point");
      if (n >= 20) this.achEvent("td_wave20");
      if (n === 10 && this.lives === this.rules.lives) this.achEvent("td_perfect10");
      this.wave = n + 1;
      this.saveBest();
      this.state = BUILD;
    }

    finish() {
      this.saveBest();
      this.state = GAMEOVER;
      this.gameOver = true;
      // update() läuft im Game Over nicht -> Klick-Sperre über die Echtzeit
      this.overT = ui.now();
      this.playSound("gameover");
      this.rumble(300);
    }

    // ----- Fähigkeiten ---------------------------------------------------
    useAbility(key) {
      if (!this.rules.abil || this.abilCd[key] > 0) return;
      if (key === "meteor") {
        this.armed = this.armed !== "meteor" ? "meteor" : null;
        this.playSound("click");
        return;
      }
      this.abilCd[key] = ABIL_CDS[key];
      if (key === "nova") {
        for (const e of this.enemies) {
          e.slow_f = Math.max(e.slow_f, 0.6);
          e.slow_t = Math.max(e.slow_t, 3.0);
        }
        this.addFx("nova", { pos: [GRID_W / 2, GRID_H / 2], ttl: 0.5 });
      } else if (key === "gold") {
        this.gold += 80 + 3 * this.wave;
      }
      this.playSound("level");
    }

    castMeteor(cellpos) {
      this.armed = null;
      this.abilCd.meteor = ABIL_CDS.meteor;
      const dmg = 150 + 12 * this.wave;
      const [px, py] = cellpos;
      for (const e of this.enemies.slice()) {
        if (e.hp <= 0) continue;
        const [ex, ey] = this.posAt(e.d, e.fly);
        if ((ex - px) ** 2 + (ey - py) ** 2 <= 1.8 ** 2) this.damage(e, dmg, true);
      }
      this.addFx("ring", { pos: cellpos, r: 1.8, col: [255, 150, 80], ttl: 0.4 });
      this.playSound("explode");
    }

    // ----- Effekte -------------------------------------------------------
    addFx(kind, kw) {
      if (this.fx.length < MAX_FX) {
        kw.kind = kind;
        kw.age = 0;
        this.fx.push(kw);
      }
    }

    ageFx(dt) {
      for (const f of this.fx) f.age += dt;
      this.fx = this.fx.filter((f) => f.age < f.ttl);
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === MAPSEL) this.evMapsel(ev);
      else if (this.state === GAMEOVER) this.evGameover(ev);
      else this.evPlay(ev);
    }

    evMapsel(ev) {
      const n = MAPS.length;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || k === "a") {
          this.mapselIdx = PG.mod(this.mapselIdx - 1, n);
          this.playSound("move");
        } else if (k === "Right" || k === "d") {
          this.mapselIdx = PG.mod(this.mapselIdx + 1, n);
          this.playSound("move");
        } else if (k === "Up" || k === "w" || k === "Down" || k === "s") {
          this.mapselIdx = PG.mod(this.mapselIdx + 2, n);
          this.playSound("move");
        } else if (k === "Return" || k === "space") {
          if (this.unlocked(MAPS[this.mapselIdx])) this.startRun(this.mapselIdx);
          else this.playSound("hit");
        }
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        for (let i = 0; i < this.mapRects.length; i++) {
          if (this.mapRects[i].collidepoint(ev.pos)) {
            this.mapselIdx = i;
            if (this.unlocked(MAPS[i])) this.startRun(i);
            else this.playSound("hit");
            return;
          }
        }
      } else if (ev.kind === "mousemove") {
        this.mouse = ev.pos;
      }
    }

    evGameover(ev) {
      if (ev.kind === "keydown") {
        if (ev.key === "Return" || ev.key === "space") {
          this.startRun(this.mapIdx);
        } else if (ev.key === "m" || ev.key === "M") {
          this.gameOver = false;
          this.state = MAPSEL;
          this.playSound("click");
        }
      } else if (ev.kind === "mousedown" && ui.now() - this.overT > 0.5) {
        this.startRun(this.mapIdx);
      }
    }

    cellAt(pos) {
      if (!this.fieldRect.collidepoint(pos)) return null;
      return [Math.floor((pos[0] - this.ox) / this.cell), Math.floor((pos[1] - this.oy) / this.cell)];
    }

    cellposAt(pos) {
      return [(pos[0] - this.ox) / this.cell, (pos[1] - this.oy) / this.cell];
    }

    evPlay(ev) {
      if (ev.kind === "keydown") {
        this.evKey(ev.key);
      } else if (ev.kind === "mousemove") {
        this.mouse = ev.pos;
      } else if (ev.kind === "wheel") {
        if (this.barRect.collidepoint(ev.pos || [0, 0])) {
          const over = Math.max(0, this.cardsH - this.cardsViewH);
          this.barScroll = Math.max(0, Math.min(over, this.barScroll - ev.delta * 30));
        }
      } else if (ev.kind === "mousedown" && ev.button === 3) {
        if (this.armed || this.selCard) {
          this.armed = null;
          this.selCard = null;
        } else {
          this.selTower = null;
        }
        this.playSound("click");
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.mouse = ev.pos;
        this.evClick(ev.pos);
      }
    }

    evKey(key) {
      if (!key) return;
      if (key === "space" || key === "Return") {
        this.startWave();
      } else if (key === "f" || key === "F") {
        this.ff = this.ff === 1 ? 2 : 1;
        this.playSound("click");
      } else if (key === "g" || key === "G") {
        this.showRanges = !this.showRanges;
      } else if ((key === "u" || key === "U") && this.towers.has(this.selTower)) {
        this.upgradeTower(this.towers.get(this.selTower));
      } else if (key === "x" || key === "X") {
        if (this.selCard || this.armed) {
          this.selCard = null;
          this.armed = null;
        } else if (this.towers.has(this.selTower)) {
          this.sellTower(this.selTower);
        }
      } else if (key === "q" || key === "Q") {
        this.useAbility("meteor");
      } else if (key === "w" || key === "W") {
        this.useAbility("nova");
      } else if (key === "e" || key === "E") {
        this.useAbility("gold");
      } else if (/^[0-9]$/.test(key)) {
        const i = PG.mod(Number(key) - 1, 10);
        if (i < this.barKeys.length) this.selectCard(this.barKeys[i]);
      }
    }

    selectCard(key) {
      if (TOWERS[key].cost > this.gold) {
        this.playSound("hit");
        return;
      }
      this.selCard = this.selCard !== key ? key : null;
      this.selTower = null;
      this.armed = null;
      this.playSound("click");
    }

    evClick(pos) {
      // Seitenleiste
      if (this.barRect.collidepoint(pos)) {
        for (const k in this.abilRects) {
          if (this.abilRects[k].collidepoint(pos)) {
            this.useAbility(k);
            return;
          }
        }
        if (this.startRect.collidepoint(pos) && this.state === BUILD) {
          this.startWave();
          return;
        }
        const tw = this.towers.get(this.selTower);
        if (tw && this.infoRect.collidepoint(pos)) {
          const branchable = this.rules.branch && !tw.branch && tw.level >= this.rules.levels;
          if (branchable && this.brARect.collidepoint(pos)) {
            this.branchTower(tw, "a");
            return;
          }
          if (branchable && this.brBRect.collidepoint(pos)) {
            this.branchTower(tw, "b");
            return;
          }
          if (!branchable && this.upRect.collidepoint(pos) && tw.level < this.rules.levels && !tw.branch) {
            this.upgradeTower(tw);
            return;
          }
          if (this.sellRect.collidepoint(pos)) {
            this.sellTower(this.selTower);
            return;
          }
        }
        for (let i = 0; i < this.barKeys.length; i++) {
          const rr = this.cardRects[i].move(0, -this.barScroll);
          if (rr.bottom < this.cardsTop || rr.y > this.cardsTop + this.cardsViewH) continue;
          if (rr.collidepoint(pos)) {
            this.selectCard(this.barKeys[i]);
            return;
          }
        }
        return;
      }
      // HUD: Tempo-Chip
      if (this.speedRect && this.speedRect.collidepoint(pos)) {
        this.ff = this.ff === 1 ? 2 : 1;
        this.playSound("click");
        return;
      }
      // Spielfeld
      if (this.armed === "meteor") {
        if (this.fieldRect.collidepoint(pos)) this.castMeteor(this.cellposAt(pos));
        return;
      }
      const cell = this.cellAt(pos);
      if (cell === null) return;
      if (this.selCard) {
        if (this.canBuild(cell)) this.placeTower(cell);
        else this.playSound("hit");
        return;
      }
      const k = ckey(cell[0], cell[1]);
      if (this.towers.has(k)) {
        this.selTower = k;
        this.playSound("click");
      } else {
        this.selTower = null;
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      if (this.state === MAPSEL) {
        this.drawMapsel(ctx);
        return;
      }
      const bg = this.fieldBg();
      ctx.drawImage(bg, 0, 0, this.width, this.height);
      this.drawTowers(ctx);
      this.drawEnemies(ctx);
      this.drawShots(ctx);
      this.drawFx(ctx);
      this.drawGhost(ctx);
      this.drawHud(ctx);
      this.drawSidebar(ctx);
      if (this.state === GAMEOVER) this.drawGameover(ctx);
    }

    // ----- Spielfeld-Hintergrund (gecacht) -------------------------------
    fieldBg() {
      const ps = Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
      const key = [MAPS[this.mapIdx].id, this.width, this.height, this.cell, ps.toFixed(2), ui.themeName()].join("|");
      if (this.bgCache && this.bgCache[0] === key) return this.bgCache[1];
      const surf = ui.makeCanvas(this.width * ps, this.height * ps);
      const g = surf.getContext("2d");
      g.scale(ps, ps);
      draw.rect(g, ui.mix(COL_GRASS2, [0, 0, 0], 0.35), [0, 0, this.width, this.height]);
      const cs = this.cell;
      for (let r = 0; r < GRID_H; r++) {
        for (let c = 0; c < GRID_W; c++) {
          const col = (c + r) % 2 === 0 ? COL_GRASS1 : COL_GRASS2;
          draw.rect(g, col, [this.ox + c * cs, this.oy + r * cs, cs, cs]);
        }
      }
      // baubare Zellen der Spießrutenlauf-Karte markieren
      if (this.buildCells !== null) {
        for (const k of this.buildCells) {
          const [c, r] = k.split(",").map(Number);
          draw.rect(g, ui.mix(COL_GRASS1, [255, 255, 255], 0.1), [this.ox + c * cs + 2, this.oy + r * cs + 2, cs - 4, cs - 4], 1);
        }
      }
      // Pfad: breite dunkle Kante + hellerer Kern entlang der Polylinie
      const pts = this.pathPts.map(([x, y]) => this.px(x, y));
      for (const [width, col] of [[Math.floor(cs * 0.86), COL_PATH_EDGE], [Math.floor(cs * 0.62), COL_PATH]]) {
        for (let i = 1; i < pts.length; i++) draw.line(g, col, pts[i - 1], pts[i], width);
        for (const p of pts) draw.circle(g, col, p, Math.floor(width / 2));
      }
      // Start-Pfeil und Ziel-Portal
      const [sx, sy] = pts[0];
      const [ex, ey] = pts[pts.length - 1];
      draw.circle(g, [30, 26, 40], [ex, ey], Math.floor(cs * 0.42));
      draw.circle(g, [120, 90, 200], [ex, ey], Math.floor(cs * 0.42), 3);
      const d = sx < this.fieldRect.centerx ? 1 : -1;
      draw.polygon(g, [240, 220, 130], [
        [sx - d * cs * 0.3, sy - cs * 0.35],
        [sx + d * cs * 0.45, sy],
        [sx - d * cs * 0.3, sy + cs * 0.35],
      ]);
      draw.rect(g, ui.BORDER, this.fieldRect, 2);
      this.bgCache = [key, surf];
      return surf;
    }

    // ----- Türme ---------------------------------------------------------
    drawTower(ctx, tw, px, py, ghost = false) {
      const cs = this.cell;
      const kind = tw.kind;
      const col = TOWERS[kind].col;
      const baseR = Math.floor(cs * 0.38);
      draw.circle(ctx, [38, 42, 50], [px, py], baseR);
      draw.circle(ctx, col, [px, py], baseR, 2);
      const a = tw.angle || 0;
      const blen = cs * 0.45;
      const bx = px + Math.cos(a) * blen, by = py + Math.sin(a) * blen;
      if (kind === "arrow") {
        draw.line(ctx, col, [px, py], [bx, by], 3);
      } else if (kind === "cannon") {
        draw.line(ctx, col, [px, py], [bx, by], Math.max(4, Math.floor(cs / 6)));
        draw.circle(ctx, col, [px, py], Math.floor(cs * 0.2));
      } else if (kind === "frost") {
        const pts = [];
        for (let i = 0; i < 6; i++) pts.push([px + Math.cos(a + (i * PG.TAU) / 6) * cs * 0.24, py + Math.sin(a + (i * PG.TAU) / 6) * cs * 0.24]);
        draw.polygon(ctx, col, pts, 2);
      } else if (kind === "sniper") {
        draw.line(ctx, col, [px, py], [px + Math.cos(a) * cs * 0.62, py + Math.sin(a) * cs * 0.62], 2);
        draw.circle(ctx, col, [px, py], Math.floor(cs * 0.12));
      } else if (kind === "poison") {
        draw.circle(ctx, col, [px, py], Math.floor(cs * 0.2));
        draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.4), [px, py - Math.floor(cs * 0.12)], Math.floor(cs * 0.1));
      } else if (kind === "tesla") {
        draw.circle(ctx, col, [px, py], Math.floor(cs * 0.14));
        for (let i = 0; i < 3; i++) {
          const aa = this.animT * 2 + (i * PG.TAU) / 3;
          draw.line(ctx, col, [px, py], [px + Math.cos(aa) * cs * 0.3, py + Math.sin(aa) * cs * 0.3], 2);
        }
      } else if (kind === "support") {
        const rr = Math.floor(cs * (0.2 + 0.06 * ui.pulse(2.0)));
        draw.circle(ctx, col, [px, py], rr, 2);
      } else if (kind === "mortar") {
        draw.circle(ctx, col, [px, py], Math.floor(cs * 0.26), 4);
      } else if (kind === "flak") {
        for (const off of [-0.08, 0.08]) {
          const ox = Math.cos(a + Math.PI / 2) * cs * off;
          const oy = Math.sin(a + Math.PI / 2) * cs * off;
          draw.line(ctx, col, [px + ox, py + oy], [bx + ox, by + oy], 2);
        }
      } else if (kind === "laser") {
        draw.polygon(ctx, col, [
          [px + Math.cos(a) * cs * 0.3, py + Math.sin(a) * cs * 0.3],
          [px + Math.cos(a + 2.2) * cs * 0.2, py + Math.sin(a + 2.2) * cs * 0.2],
          [px + Math.cos(a - 2.2) * cs * 0.2, py + Math.sin(a - 2.2) * cs * 0.2],
        ]);
      } else if (kind === "bank") {
        const rr = Math.floor(cs * 0.2);
        draw.rect(ctx, col, [px - rr, py - rr, rr * 2, rr * 2]);
        draw.rect(ctx, ui.mix(col, [0, 0, 0], 0.4), [px - rr, py - rr, rr * 2, rr * 2], 2);
      }
      if (ghost) return;
      // Stufen-Pips + Zweig-Punkt
      for (let i = 0; i < tw.level - 1; i++) draw.rect(ctx, col, [px - baseR + i * 6, py + baseR - 2, 4, 4]);
      if (tw.branch) {
        const bc = tw.branch === "a" ? [255, 255, 255] : [255, 200, 90];
        draw.circle(ctx, bc, [px + baseR - 3, py - baseR + 3], 3);
      }
    }

    drawTowers(ctx) {
      const cs = this.cell;
      for (const [key, tw] of this.towers) {
        const cell = tw.cell;
        let [px, py] = this.px(cell[0] + 0.5, cell[1] + 0.5);
        px = Math.floor(px);
        py = Math.floor(py);
        const sel = key === this.selTower;
        if (sel || this.showRanges) {
          const rng = tw.st.rng;
          if (rng) draw.circle(ctx, ui.mix(TOWERS[tw.kind].col, [255, 255, 255], 0.2), [px, py], Math.floor(rng * cs), 1);
        }
        if (sel) draw.rect(ctx, this.accent, [this.ox + cell[0] * cs, this.oy + cell[1] * cs, cs, cs], 2);
        this.drawTower(ctx, tw, px, py);
        if (TOWERS[tw.kind].beam && tw.beam) {
          const e = tw.beam;
          if (e.hp > 0) {
            const [ex, ey] = this.posAt(e.d, e.fly);
            const [epx, epy] = this.px(ex, ey);
            const w = 2 + Math.trunc(tw.ramp);
            draw.line(ctx, [255, 120, 110], [px, py], [epx, epy], w);
          }
        }
      }
    }

    // ----- Gegner --------------------------------------------------------
    drawEnemies(ctx) {
      const cs = this.cell;
      const ground = this.enemies.filter((e) => !e.fly);
      const air = this.enemies.filter((e) => e.fly);
      for (const e of ground.concat(air)) {
        const [ex, ey] = this.posAt(e.d, e.fly);
        let [px, py] = this.px(ex, ey);
        px = Math.floor(px);
        py = Math.floor(py);
        const r = Math.max(3, Math.floor(e.r * cs));
        if (e.fly) {
          draw.ellipse(ctx, COL_AIR_SHADOW, [px - r, py + Math.floor(cs * 0.28), r * 2, Math.max(3, Math.floor(r / 2))]);
          py -= Math.floor(cs * 0.22);
        }
        if (e.stealth && !e.seen) {
          draw.circle(ctx, ui.mix(COL_GRASS1, [255, 255, 255], 0.18), [px, py], r, 1);
          continue;
        }
        let col = e.col;
        if (e.slow_t > 0) col = ui.mix(col, [130, 200, 255], 0.45);
        const kind = e.kind;
        if (kind === "tank") {
          draw.rect(ctx, col, [px - r, py - r, r * 2, r * 2], 0, 3);
          draw.rect(ctx, ui.mix(col, [0, 0, 0], 0.4), [px - r, py - r, r * 2, r * 2], 2, 3);
        } else if (kind === "fast") {
          draw.polygon(ctx, col, [[px + r, py], [px - r, py - r], [px - r, py + r]]);
        } else if (kind === "boss") {
          draw.circle(ctx, col, [px, py], r);
          draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.45), [px, py], r, 3);
          for (let i = 0; i < 6; i++) {
            const aa = this.animT * 1.5 + (i * PG.TAU) / 6;
            draw.line(ctx, col, [px + Math.cos(aa) * r, py + Math.sin(aa) * r], [px + Math.cos(aa) * (r + cs * 0.18), py + Math.sin(aa) * (r + cs * 0.18)], 2);
          }
        } else {
          draw.circle(ctx, col, [px, py], r);
          draw.circle(ctx, ui.mix(col, [0, 0, 0], 0.4), [px, py], r, 2);
        }
        if (kind === "shield") {
          draw.circle(ctx, [220, 225, 255], [px, py], r + 2, 1);
        } else if (kind === "regen") {
          draw.line(ctx, [240, 255, 240], [px - 3, py], [px + 3, py], 2);
          draw.line(ctx, [240, 255, 240], [px, py - 3], [px, py + 3], 2);
        } else if (kind === "healer") {
          draw.circle(ctx, [230, 255, 245], [px, py], r + 3, 1);
        } else if (kind === "splitter") {
          draw.circle(ctx, ui.mix(e.col, [0, 0, 0], 0.35), [px - Math.floor(r / 3), py], 2);
          draw.circle(ctx, ui.mix(e.col, [0, 0, 0], 0.35), [px + Math.floor(r / 3), py], 2);
        }
        if (e.dot_t > 0) draw.circle(ctx, [140, 220, 80], [px + r - 1, py - r + 1], 3);
        if (e.hp < e.maxhp) {
          const bw = r * 2;
          const frac = Math.max(0, e.hp / e.maxhp);
          const y0 = py - r - 6;
          draw.rect(ctx, COL_HP_BG, [px - r, y0, bw, 3]);
          draw.rect(ctx, ui.mix([220, 80, 70], COL_HP, frac), [px - r, y0, Math.floor(bw * frac), 3]);
        }
      }
    }

    // ----- Schüsse / Effekte ---------------------------------------------
    drawShots(ctx) {
      const cs = this.cell;
      for (const sh of this.shots) {
        if (sh.kind === "lob") {
          const f = sh.t / sh.dur;
          const x = sh.x + (sh.tx - sh.x) * f;
          const y = sh.y + (sh.ty - sh.y) * f;
          const [px, py] = this.px(x, y);
          const [tx, ty] = this.px(sh.tx, sh.ty);
          draw.circle(ctx, [20, 20, 20], [Math.floor(tx), Math.floor(ty)], 3);
          const arc = Math.sin(f * Math.PI) * cs * 1.1;
          draw.circle(ctx, sh.col, [Math.floor(px), Math.floor(py - arc)], 4);
        } else {
          const [px, py] = this.px(sh.x, sh.y);
          draw.circle(ctx, sh.col, [Math.floor(px), Math.floor(py)], 3);
        }
      }
    }

    drawFx(ctx) {
      const cs = this.cell;
      for (const f of this.fx) {
        const fr = f.age / f.ttl;
        if (f.kind === "tracer") {
          draw.line(ctx, f.col, this.px(f.a[0], f.a[1]), this.px(f.b[0], f.b[1]), 2);
        } else if (f.kind === "zigzag") {
          const pts = f.pts.map(([x, y]) => this.px(x, y));
          for (let i = 1; i < pts.length; i++) {
            const p0 = pts[i - 1], p1 = pts[i];
            const mx = (p0[0] + p1[0]) / 2 + PG.rand.uniform(-4, 4);
            const my = (p0[1] + p1[1]) / 2 + PG.rand.uniform(-4, 4);
            draw.lines(ctx, f.col, false, [p0, [mx, my], p1], 2);
          }
        } else if (f.kind === "ring") {
          const [px, py] = this.px(f.pos[0], f.pos[1]);
          const rr = Math.floor(f.r * cs * (0.3 + 0.7 * fr));
          draw.circle(ctx, f.col, [Math.floor(px), Math.floor(py)], rr, 2);
        } else if (f.kind === "burst") {
          const [px, py] = this.px(f.pos[0], f.pos[1]);
          for (let i = 0; i < 5; i++) {
            const aa = (i * PG.TAU) / 5 + fr * 2;
            const d = fr * cs * 0.7;
            draw.circle(ctx, f.col, [Math.floor(px + Math.cos(aa) * d), Math.floor(py + Math.sin(aa) * d)], 2);
          }
        } else if (f.kind === "nova") {
          const [px, py] = this.px(f.pos[0], f.pos[1]);
          const rr = Math.floor(fr * this.fieldRect.w * 0.7);
          draw.circle(ctx, [140, 210, 255], [Math.floor(px), Math.floor(py)], Math.max(2, rr), 2);
        }
      }
    }

    /** Platzierungs-Vorschau bzw. Meteor-Zielkreis unter der Maus. */
    drawGhost(ctx) {
      if (this.mouse === null || this.state === GAMEOVER) return;
      const cs = this.cell;
      if (this.armed === "meteor") {
        if (this.fieldRect.collidepoint(this.mouse)) draw.circle(ctx, [255, 150, 80], this.mouse, Math.floor(1.8 * cs), 2);
        return;
      }
      if (!this.selCard) return;
      const cell = this.cellAt(this.mouse);
      if (cell === null) return;
      const ok = this.canBuild(cell);
      let [px, py] = this.px(cell[0] + 0.5, cell[1] + 0.5);
      px = Math.floor(px);
      py = Math.floor(py);
      const col = ok ? COL_BUILD_OK : COL_BUILD_BAD;
      draw.rect(ctx, col, [this.ox + cell[0] * cs, this.oy + cell[1] * cs, cs, cs], 2);
      const rng = TOWERS[this.selCard].rng || 0;
      if (rng) draw.circle(ctx, col, [px, py], Math.floor(rng * cs), 1);
      this.drawTower(ctx, { kind: this.selCard, level: 1, angle: 0 }, px, py, true);
    }

    // ----- HUD -----------------------------------------------------------
    heart(ctx, x, y, r, col) {
      const h2 = Math.floor(r / 2), h3 = Math.floor(r / 3), h4 = Math.floor(r / 4);
      draw.circle(ctx, col, [x - h2, y - h3], h2 + 1);
      draw.circle(ctx, col, [x + h2, y - h3], h2 + 1);
      draw.polygon(ctx, col, [[x - r, y - h4], [x + r, y - h4], [x, y + r]]);
    }

    drawHud(ctx) {
      const w = this.width - this.barW;
      draw.rect(ctx, ui.PANEL, [0, 0, w, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH - 0.5], [w, this.hudH - 0.5]);
      const cy = Math.floor(this.hudH / 2);
      let x = 14;
      this.heart(ctx, x + 5, cy, 7, [235, 90, 100]);
      let s = String(Math.max(0, this.lives));
      ui.text(ctx, s, x + 18, cy, this.small, ui.TEXT, "midleft");
      x += 26 + this.small.width(s) + 14;
      draw.circle(ctx, ui.GOLD, [x + 6, cy], 7);
      draw.circle(ctx, ui.mix(ui.GOLD, [0, 0, 0], 0.4), [x + 6, cy], 7, 2);
      s = String(this.gold);
      ui.text(ctx, s, x + 18, cy, this.small, ui.GOLD, "midleft");
      x += 26 + this.small.width(s) + 16;
      s = t("td.wave", { n: this.wave });
      ui.text(ctx, s, x, cy, this.small, this.accent, "midleft");
      x += this.small.width(s) + 16;
      if (this.state === WAVE) {
        const left = this.enemies.length + this.spawnQueue.length;
        ui.text(ctx, t("td.left", { n: left }), x, cy, this.tiny, ui.TEXT_DIM, "midleft");
      }
      // Punkte + Tempo rechtsbündig
      const chipW = Math.max(40, this.tiny.width("2x") + 18);
      this.speedRect = new PG.Rect(w - chipW - 10, 5, chipW, this.hudH - 10);
      ui.text(ctx, String(this.score), this.speedRect.x - 14, cy, this.small, ui.TEXT, "midright");
      const sel = this.ff > 1;
      draw.rect(ctx, sel ? ui.PANEL_LIGHT : ui.PANEL, this.speedRect, 0, 6);
      draw.rect(ctx, sel ? this.accent : ui.BORDER, this.speedRect, 1, 6);
      ui.text(ctx, this.ff + "x", this.speedRect.centerx, this.speedRect.centery, this.tiny, sel ? this.accent : ui.TEXT_DIM, "center");
    }

    // ----- Seitenleiste --------------------------------------------------
    drawSidebar(ctx) {
      draw.rect(ctx, ui.PANEL, this.barRect);
      draw.line(ctx, ui.BORDER, [this.barRect.x + 0.5, 0], [this.barRect.x + 0.5, this.height]);
      // Fähigkeiten (Maximal)
      for (const key in this.abilRects) {
        const r = this.abilRects[key];
        const cd = this.abilCd[key];
        const ready = cd <= 0;
        const armed = this.armed === key;
        draw.rect(ctx, ready ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 6);
        draw.rect(ctx, armed ? this.accent : ui.BORDER, r, armed ? 2 : 1, 6);
        const [cx, cy] = r.center;
        if (key === "meteor") {
          draw.circle(ctx, [255, 150, 80], [cx, cy - 2], 5);
          draw.line(ctx, [255, 150, 80], [cx + 3, cy - 5], [cx + 8, cy - 10], 2);
        } else if (key === "nova") {
          draw.circle(ctx, [140, 210, 255], [cx, cy - 2], 6, 2);
        } else {
          draw.circle(ctx, ui.GOLD, [cx, cy - 2], 6);
        }
        if (!ready) ui.text(ctx, String(Math.trunc(cd) + 1), cx, r.bottom - 1, this.tiny, ui.TEXT_DIM, "midbottom");
      }
      // Turmkarten (mit Scroll-Ausschnitt)
      ctx.save();
      ctx.beginPath();
      ctx.rect(this.barRect.x, this.cardsTop, this.barW, this.cardsViewH);
      ctx.clip();
      let hoverKey = null;
      this.barKeys.forEach((key, i) => {
        const rr = this.cardRects[i].move(0, -this.barScroll);
        const base = TOWERS[key];
        const afford = this.gold >= base.cost;
        const sel = this.selCard === key;
        draw.rect(ctx, sel ? ui.PANEL_LIGHT : ui.PANEL, rr, 0, 8);
        draw.rect(ctx, sel ? this.accent : ui.BORDER, rr, sel ? 2 : 1, 8);
        this.drawTower(ctx, { kind: key, level: 1, angle: -Math.PI / 2 }, rr.centerx, rr.centery - 6, true);
        ui.text(ctx, String(base.cost), rr.centerx, rr.bottom - 3, this.tiny, afford ? ui.GOLD : ui.TEXT_FAINT, "midbottom");
        if (this.mouse && rr.collidepoint(this.mouse)) hoverKey = key;
      });
      ctx.restore();
      // Info-Bereich
      this.drawInfo(ctx, hoverKey);
      // Start-Button / Wellen-Status
      if (this.state === BUILD) {
        ui.drawButton(ctx, this.startRect, t("td.start_wave"), this.small, true, { accent: this.accent });
      } else {
        draw.rect(ctx, ui.PANEL_LIGHT, this.startRect, 0, 8);
        draw.rect(ctx, ui.BORDER, this.startRect, 1, 8);
        ui.text(ctx, t("td.wave", { n: this.wave }), this.startRect.centerx, this.startRect.centery, this.small, ui.TEXT_DIM, "center");
      }
    }

    infoLines(key, tw = null) {
      const base = TOWERS[key];
      const st = tw ? tw.st : null;
      const dmg = st ? st.dmg : base.dmg || 0;
      const rng = st ? st.rng : base.rng || 0;
      const cd = st ? st.cd : base.cd || 0;
      const lines = [];
      if (base.income) {
        const inc = st ? st.income : base.income;
        lines.push("+" + Math.trunc(inc) + "/W");
      } else if (base.buff) {
        const b = st ? st.buff : base.buff;
        lines.push("+" + Math.trunc(b * 100 + 1e-9) + "%");
      } else if (base.beam) {
        lines.push(Math.trunc(dmg) + "/s");
      } else {
        lines.push(Math.trunc(dmg) + "  |  " + cd.toFixed(1) + "s");
      }
      if (rng) lines.push("R " + rng.toFixed(1));
      return lines.join("   ");
    }

    drawInfo(ctx, hoverKey) {
      const r = this.infoRect;
      draw.rect(ctx, ui.PANEL_LIGHT, r, 0, 8);
      draw.rect(ctx, ui.BORDER, r, 1, 8);
      const x = r.x + 10;
      let y = r.y + 8;
      const tw = this.towers.get(this.selTower);
      if (tw) {
        const base = TOWERS[tw.kind];
        ui.text(ctx, t("td.tower." + tw.kind), x, y, this.small, base.col);
        y += this.small.height + 2;
        ui.text(ctx, t("td.level", { n: tw.level }), x, y, this.tiny, ui.TEXT_DIM);
        y += this.tiny.height + 2;
        ui.text(ctx, this.infoLines(tw.kind, tw), x, y, this.tiny, ui.TEXT);
        const branchable = this.rules.branch && !tw.branch && tw.level >= this.rules.levels;
        if (branchable) {
          for (const [which, rect] of [["a", this.brARect], ["b", this.brBRect]]) {
            const cost = this.branchCost(tw);
            const ok = this.gold >= cost;
            draw.rect(ctx, ui.PANEL, rect, 0, 6);
            draw.rect(ctx, ok ? this.accent : ui.BORDER, rect, 1, 6);
            ui.text(ctx, t("td.br." + tw.kind + "." + which), rect.centerx, rect.centery - 6, this.tiny, ok ? ui.TEXT : ui.TEXT_FAINT, "center");
            ui.text(ctx, String(cost), rect.centerx, rect.centery + 8, this.tiny, ok ? ui.GOLD : ui.TEXT_FAINT, "center");
          }
        } else if (tw.level < this.rules.levels && !tw.branch) {
          const cost = this.upgradeCost(tw);
          const ok = this.gold >= cost;
          draw.rect(ctx, ui.PANEL, this.upRect, 0, 6);
          draw.rect(ctx, ok ? this.accent : ui.BORDER, this.upRect, 1, 6);
          ui.text(ctx, t("td.upgrade", { c: cost }), this.upRect.centerx, this.upRect.centery, this.tiny, ok ? ui.TEXT : ui.TEXT_FAINT, "center");
        } else {
          ui.text(ctx, t("td.max"), this.upRect.centerx, this.upRect.centery, this.tiny, ui.GOLD, "center");
        }
        draw.rect(ctx, ui.PANEL, this.sellRect, 0, 6);
        draw.rect(ctx, ui.BORDER, this.sellRect, 1, 6);
        ui.text(ctx, t("td.sell", { c: this.sellValue(tw) }), this.sellRect.centerx, this.sellRect.centery, this.tiny, ui.TEXT_DIM, "center");
        return;
      }
      const key = hoverKey || this.selCard;
      if (key) {
        const base = TOWERS[key];
        ui.text(ctx, t("td.tower." + key), x, y, this.small, base.col);
        y += this.small.height + 2;
        ui.text(ctx, this.infoLines(key), x, y, this.tiny, ui.TEXT);
        y += this.tiny.height + 4;
        y = this.wrapText(ctx, t("td.tower." + key + ".desc"), x, y, r.w - 20, this.tiny, ui.TEXT_DIM);
        if (this.selCard) {
          y += 4;
          this.wrapText(ctx, t("td.place_hint"), x, y, r.w - 20, this.tiny, ui.TEXT_FAINT);
        }
        return;
      }
      // Standard: Kurzhilfe
      const hint = this.state === BUILD ? t("td.hint.build") : t("td.hint.wave");
      this.wrapText(ctx, hint, x, y, r.w - 20, this.tiny, ui.TEXT_DIM);
    }

    wrapText(ctx, text, x, y, maxw, fnt, col) {
      const words = text.split(/\s+/).filter(Boolean);
      let line = "";
      for (const wd of words) {
        const test = (line + " " + wd).trim();
        if (fnt.width(test) > maxw && line) {
          ui.text(ctx, line, x, y, fnt, col);
          y += fnt.height + 1;
          line = wd;
        } else {
          line = test;
        }
      }
      if (line) {
        ui.text(ctx, line, x, y, fnt, col);
        y += fnt.height + 1;
      }
      return y;
    }

    // ----- Kartenwahl / Game Over ----------------------------------------
    drawMapsel(ctx) {
      ui.drawBackground(ctx, this.width, this.height);
      ui.drawTitle(ctx, this.width, "TOWER DEFENSE", { subtitle: t("td.choose_map"), accent: this.accent });
      MAPS.forEach((m, i) => {
        const r = this.mapRects[i];
        const unlocked = this.unlocked(m);
        const sel = i === this.mapselIdx || (this.mouse && r.collidepoint(this.mouse));
        ui.drawPanel(ctx, r, { accentTop: sel ? this.accent : null });
        if (sel) draw.rect(ctx, this.accent, r, 2, 10);
        // Pfad-Vorschau
        const pad = 18;
        const pw = r.w - pad * 2, ph = r.h - pad * 2 - 34;
        const pts = previewPts(m).map(([x, y]) => [r.x + pad + (x / GRID_W) * pw, r.y + pad + (y / GRID_H) * ph]);
        const col = unlocked ? ui.TEXT_DIM : ui.TEXT_FAINT;
        draw.lines(ctx, col, false, pts, 3);
        draw.circle(ctx, unlocked ? ui.GREEN : ui.TEXT_FAINT, [Math.floor(pts[0][0]), Math.floor(pts[0][1])], 4);
        const last = pts[pts.length - 1];
        draw.circle(ctx, unlocked ? this.accent : ui.TEXT_FAINT, [Math.floor(last[0]), Math.floor(last[1])], 4);
        ui.text(ctx, t("td.map." + m.id), r.x + 14, r.bottom - 30, this.small, unlocked ? ui.TEXT : ui.TEXT_FAINT);
        // Schwierigkeit als Punkte
        for (let j = 0; j < 4; j++) draw.circle(ctx, j <= i ? this.accent : ui.BORDER, [r.right - 60 + j * 12, r.bottom - 22], 3);
        if (unlocked) {
          const best = this.best[m.id] || 0;
          if (best) ui.text(ctx, t("td.map.best", { n: best }), r.x + 14, r.y + 6, this.tiny, ui.GOLD);
        } else {
          ui.text(ctx, t("td.map.locked", { n: m.unlock }), r.x + 14, r.y + 6, this.tiny, ui.TEXT_FAINT);
        }
      });
      ui.drawFooter(ctx, this.width, this.height, t("td.setup_hint"));
    }

    drawGameover(ctx) {
      draw.rect(ctx, [10, 12, 20, 150], [0, 0, this.width, this.height]);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("common.game_over"), cx, Math.floor(this.height * 0.2), this.bigFont, this.accent, "center");
      const lh = this.small.height + 8;
      const pw = Math.min(380, this.width - 40);
      const panel = new PG.Rect(cx - Math.floor(pw / 2), Math.floor(this.height * 0.34), pw, lh * 4 + 36);
      ui.drawPanel(ctx, panel, { accentTop: this.accent });
      const x = panel.x + 20;
      let y = panel.y + 16;
      const mid = MAPS[this.mapIdx].id;
      const rows = [
        [t("td.gameover.wave", { n: this.wave - 1 }), ui.TEXT],
        [t("common.points", { score: this.score }), ui.TEXT],
        [t("td.kills", { n: this.kills }), ui.TEXT_DIM],
        [t("td.map.best", { n: this.best[mid] || 0 }), ui.GOLD],
      ];
      for (const [text, col] of rows) {
        ui.text(ctx, text, x, y, this.small, col);
        y += lh;
      }
      const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0));
      ui.text(ctx, t("td.restart_hint"), cx, panel.bottom + 26, this.small, hintCol, "center");
    }
  }

  function previewPts(m) {
    return m.path.map(([c, r]) => [Math.max(0, Math.min(GRID_W, c + 0.5)), Math.max(0, Math.min(GRID_H, r + 0.5))]);
  }

  PG.register(LamaTowerDefenseGame, {
    id: "LamaTowerDefenseGame",
    key: "lamatowerdef",
    name: "Tower Defense",
    modes: [["classic", "td.mode.classic"], ["compact", "td.mode.compact"], ["maximal", "td.mode.maximal"]],
    wantsRightClick: true,
  });
})();
