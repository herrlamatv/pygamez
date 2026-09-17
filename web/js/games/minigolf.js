/*
 * minigolf.js - Minigolf (Port von games/minigolf.py)
 * ===================================================
 * 360 Bahnen in 40 Kursen.
 *
 * Kurse (im Setup wählbar, wird gespeichert):
 *   - Classic : 9 handgebaute, freundliche Bahnen (Par 2-4), sanfter Einstieg.
 *   - Pro     : 9 handgebaute Bahnen mit Inselgrün, Doppelmühle und Wanderblöcken.
 *   - Tour    : 38 seed-erzeugte Kurse zu je 9 Bahnen (siehe minigolf_gen.js).
 *   - Random  : 9 zufällig gezogene Bahnen aus allen Kursen, zufällig gespiegelt.
 *   - Eigene  : die selbst gebauten Bahnen aus dem MAPS-Reiter (minigolf_edit.js).
 *
 * Die Physik läuft in Teilschritten mit Reibung, damit nichts ruckt und
 * schnelle Bälle nicht durch Banden tunneln. Untergründe bremsen
 * unterschiedlich (Grün/Sand/Eis/Klebefeld), Rampen beschleunigen, Wasser
 * kostet einen Strafschlag, Gummipuffer geben Tempo zurück, Windmühlen und
 * Wanderblöcke verlangen Timing.
 *
 * Steuerung: Maus bewegt die Ziellinie, linke Maustaste gedrückt halten lädt
 * die Schlagstärke, Loslassen schlägt. Die rechte Maustaste hält die Stärke
 * fest, solange sie gedrückt bleibt. R bricht einen geladenen Schlag ab.
 * Alternativ Pfeile links/rechts zielen, hoch/runter Stärke, Leertaste
 * schlägt. G blendet die Ziellinie um, Z das Autoziel, P das Aufnehmen, F
 * setzt die laufende Bahn zurück.
 *
 * Autoziel (Standard AN): vor jedem Schlag dreht sich der Schläger von selbst
 * zum Loch. AUS heißt, dass die zuletzt gewählte Richtung stehen bleibt.
 *
 * Stärke-Sperre (rechte Maustaste halten): friert den Ladebalken genau da ein,
 * wo er gerade steht. Gesperrt bleibt die Stärke auch über den Schlag hinaus:
 * der nächste Linksklick schlägt exakt mit dem gehaltenen Wert.
 *
 * Am Rundenende führt der Weiter-Knopf zum NÄCHSTEN Kurs (Classic -> Pro ->
 * Tour 1 -> Tour 2 -> ...); daneben stehen Nochmal und Setup.
 *
 * Punkte (Highscore) = Summe der Bahnpunkte; je Bahn gibt es mehr Punkte, je
 * weiter unter Par gespielt wird (Hole-in-One extra).
 *
 * Web-Version: nur Einzelspieler, ohne Replays.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const gen = PG.minigolfGen;
  const mdraw = PG.minigolfDraw;
  const edit = PG.minigolfEdit;
  const { CW, CH, BORDER, HOLES_PER_ROUND } = gen;
  const HOLE = gen.makeHole;
  const COL = mdraw.COL;

  // ------------------------------------------------------------- Platz / Physik
  const BR = gen.BALL_R; // Ballradius
  const CUP_R = 3.0; // Lochradius
  const ARM_W = 1.5; // halbe Breite eines Mühlenflügels

  const FRIC_GREEN = 1.15; // Rollreibung je Sekunde
  const FRIC_SAND = 4.6;
  const FRIC_ICE = 0.25; // Eis: der Ball läuft fast ewig
  const FRIC_STICKY = 12.0; // Klebefeld: bleibt sofort liegen
  const WALL_E = 0.72; // Bandenrestitution
  const BUMP_E = 1.18; // Gummipuffer geben Tempo zurück
  const STOP_EPS = 2.2; // darunter gilt der Ball als still
  const MAX_SPEED = 205.0; // maximale Schlaggeschwindigkeit
  const CAPTURE_SPEED = 74.0; // darüber springt der Ball über das Loch
  const MAX_SHOT_TIME = 14.0;
  const MAX_STROKES = 8; // danach wird die Bahn mit Höchstwert beendet
  // Fester Physik-Takt: die Desktop-Version läuft mit 60 FPS, und Lochsog sowie
  // Teilschritt-Zahl hängen vom Takt ab. Mit festem Schritt locht der Ball im
  // Browser genauso ein - egal ob 60-, 144- oder 240-Hz-Monitor.
  const PHYS_DT = 1 / 60;

  // --- Kennwerte der acht Editor-Hindernisse ---------------------------------
  const TUNNEL_KEEP = 0.95; // Tempo, das ein Rohr durchlässt
  const TUNNEL_COOLDOWN = 0.35; // Sekunden Sperre, damit es nicht zurückspringt
  const BOOST_COOLDOWN = 0.25; // ein Schubfeld feuert nicht in jedem Teilschritt
  const SPIN_PUSH = 0.55; // wie stark eine Drehscheibe mitnimmt
  const JUMP_MIN_SPEED = 30.0; // darunter ist der Ball zu langsam zum Abheben

  const SETUP = "setup", PLAY = "play", HOLE_DONE = "holedone", OVER = "over", EDIT = "edit";
  // Rand über der Überschrift und unter der Tastenzeile im Rundenende-Banner.
  const OVER_PAD = 14;
  // Die fünfte Wahl "ugc" spielt die selbst gebauten Bahnen (siehe MAPS-Reiter).
  const COURSES = ["classic", "pro", "tour", "random", "ugc"];
  // Reiter des Vorbereitungs-Screens.
  const TABS = ["play", "maps"];

  // --------------------------------------------------------- Kurs 1: Classic
  const HOLES_CLASSIC = [
    // 1 - gerade Bahn mit Trichter
    HOLE(2, [50, 140], [50, 26], { walls: [[20, 60, 14, 8], [66, 60, 14, 8]] }),
    // 2 - Dogleg nach rechts
    HOLE(3, [24, 140], [76, 30], { walls: [[38, 40, 10, 74], [60, 96, 26, 8]], sand: [[58, 118, 26, 14]] }),
    // 3 - Mittelblock mit Sandgürtel
    HOLE(3, [50, 142], [50, 22], { walls: [[38, 66, 24, 20]], sand: [[14, 62, 20, 28], [66, 62, 20, 28]] }),
    // 4 - Teich links, schmale Passage rechts
    HOLE(3, [26, 142], [76, 26], { water: [[12, 56, 44, 44]], walls: [[66, 74, 8, 50]] }),
    // 5 - S-Kurve
    HOLE(4, [20, 144], [80, 24], { walls: [[30, 108, 62, 8], [8, 66, 62, 8], [30, 30, 46, 8]] }),
    // 6 - Gummipuffer-Feld
    HOLE(3, [50, 142], [50, 22], { bumpers: [[30, 96, 6], [70, 96, 6], [50, 66, 7], [30, 44, 5], [70, 44, 5]] }),
    // 7 - Windmühle
    HOLE(4, [50, 144], [50, 22], { walls: [[6, 84, 30, 8], [64, 84, 30, 8]], mills: [[50, 88, 13, 2, 1.5]] }),
    // 8 - Steigung mit Sandfang
    HOLE(3, [50, 144], [50, 20], { slopes: [[20, 50, 60, 56, 0.0, 34.0]], sand: [[20, 112, 24, 16], [56, 112, 24, 16]] }),
    // 9 - Wanderblock vor dem Grün
    HOLE(4, [26, 142], [74, 26], { walls: [[44, 34, 8, 46], [44, 106, 8, 40]], movers: [[20, 80, 24, 8, 40, 0, 26]], bumpers: [[78, 96, 6]] }),
  ];

  // ------------------------------------------------------------ Kurs 2: Pro
  const HOLES_PRO = [
    // 10 - Inselgrün mit schmalem Hals
    HOLE(3, [50, 144], [50, 36], {
      water: [[10, 14, 80, 12], [10, 26, 26, 22], [64, 26, 26, 22], [10, 48, 34, 12], [56, 48, 34, 12]],
      walls: [[26, 62, 14, 6], [60, 62, 14, 6]],
      slopes: [[20, 72, 30, 40, 11.0, -8.0], [50, 72, 30, 40, -11.0, -8.0]],
    }),
    // 11 - Doppelmühle im Korridor
    HOLE(4, [50, 146], [50, 18], { walls: [[28, 20, 6, 100], [66, 20, 6, 100]], mills: [[50, 106, 15, 2, -1.8], [50, 56, 15, 2, 2.2]] }),
    // 12 - Puffer-Tunnel
    HOLE(4, [18, 142], [82, 26], {
      walls: [[34, 100, 8, 46], [58, 46, 8, 46]],
      bumpers: [[50, 122, 7], [24, 74, 6], [76, 74, 6], [50, 34, 6]],
      sand: [[60, 116, 26, 20]],
    }),
    // 13 - Zickzack-Labyrinth
    HOLE(5, [14, 146], [86, 20], { walls: [[24, 118, 70, 7], [6, 92, 70, 7], [24, 66, 70, 7], [6, 40, 70, 7]] }),
    // 14 - Wanderschleusen
    HOLE(4, [50, 146], [50, 18], {
      walls: [[6, 108, 34, 7], [60, 108, 34, 7], [6, 56, 34, 7], [60, 56, 34, 7]],
      movers: [[40, 108, 12, 7, 8, 0, 14], [48, 56, 12, 7, -8, 0, 12]],
    }),
    // 15 - Sandwüste mit Rampe
    HOLE(5, [20, 146], [80, 18], {
      sand: [[8, 92, 84, 34]], slopes: [[20, 30, 60, 54, 0.0, 26.0]],
      walls: [[46, 128, 8, 22]], bumpers: [[20, 60, 6], [80, 60, 6]],
    }),
    // 16 - Teichquerung über den Steg
    HOLE(4, [50, 146], [50, 20], { water: [[8, 56, 28, 52], [64, 56, 28, 52]], bumpers: [[24, 34, 6], [76, 34, 6]], sand: [[40, 116, 20, 14]] }),
    // 17 - Kreuzmühle
    HOLE(4, [18, 144], [82, 24], { walls: [[6, 96, 26, 8], [68, 96, 26, 8]], mills: [[50, 100, 14, 3, 1.1]], sand: [[66, 118, 24, 18]] }),
    // 18 - Finale: Tore, Wasser und Rampe
    HOLE(5, [50, 148], [50, 18], {
      water: [[8, 96, 30, 30], [62, 96, 30, 30]], walls: [[38, 92, 8, 6], [54, 92, 8, 6]],
      slopes: [[24, 40, 52, 44, 0.0, 30.0]], movers: [[24, 62, 22, 8, 30, 0, 24]], bumpers: [[50, 30, 7]],
    }),
  ];

  // Kürzen auf maxW mit "..." (wie die Python-Schleifen).
  function fit(fnt, text, maxW) {
    if (fnt.width(text) <= maxW) return text;
    let kurz = text;
    while (kurz.length > 2 && fnt.width(kurz + "...") > maxW) kurz = kurz.slice(0, -1);
    return kurz + "...";
  }

  const sum = (arr) => arr.reduce((s, v) => s + v, 0);

  class MiniGolfGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const gs = this.opts;
      this.course = COURSES.includes(gs.course) ? gs.course : "classic";
      this.guide = !!gs.guide;
      // Autoziel: der Schläger dreht sich vor jedem Schlag von selbst zum Loch.
      this.autoaim = !!gs.autoaim;
      // Aufnehmen: nach MAX_STROKES Schlägen ist die Bahn vorbei.
      this.pickup = !!gs.pickup;
      // Stärke-Sperre: solange die rechte Maustaste gehalten wird, bleibt die
      // Schlagstärke stehen (lockT treibt den Puls der Anzeige).
      this.powerLock = false;
      this.lockT = 0.0;
      this.tour = Math.max(1, Math.min(gen.TOUR_COURSES, Math.trunc(Number(gs.tour) || 1)));
      this.tourPar = gen.coursePar(this.tour);
      // Größe der LAUFENDEN Bahn. Eigene Bahnen bringen ihre eigene mit.
      this.cw = CW;
      this.ch = CH;
      // Reiter des Vorbereitungs-Screens und der Bahn-Editor. Beide werden
      // erst angelegt, wenn sie gebraucht werden.
      this.setupTab = "play";
      this.maps = null;
      this.editor = null;
      // id der einzeln gespielten eigenen Bahn (leer = alle nacheinander).
      this.singleMap = String(gs.ugc_map || "");
      // Beim Test-Spielen aus dem Editor: hierhin geht es danach zurück.
      this.testReturn = false;
      // Rohr-Sperre und Schub-Sperre (siehe physics).
      this.tunCd = 0.0;
      this.boostCd = 0.0;
      this.air = 0.0; // verbleibende Flugstrecke einer Sprungrampe
      this.msg = null;
      this.msgT = 0.0;
      // Wiederholung der Runde (replay, siehe core/replay.js).
      this.rec = null;
      this.replay = null;
      this.replayRequest = null;
      this.rep = null; // gesetzt, solange nur abgespielt wird
      this.repAt = null;
      this.layoutId = 0;

      this.buildFonts();
      this.layout();
      this.buildSetupLayout();
      this.best = this.loadBest();
      this.newRound();
      this.state = SETUP;
    }

    buildFonts() {
      const h = this.height;
      this.fSmall = ui.font(Math.max(14, Math.floor(h / 32)));
      this.fTiny = ui.font(Math.max(11, Math.floor(h / 42)));
      this.fCard = ui.font(Math.max(11, Math.floor(h / 46)), false, true);
      this.fHuge = ui.font(Math.max(24, Math.floor(h / 13)), true);
    }

    /**
     * Maßstab und Nullpunkt des Platzes aus der Spielfläche ableiten.
     * Gerechnet wird mit this.cw/this.ch: eigene Bahnen dürfen von 100x160
     * abweichen, dann muss der Platz neu eingepasst werden.
     */
    layout() {
      const cw = this.cw, ch = this.ch;
      this.hudH = 44;
      const availH = this.height - this.hudH - 12;
      const cardW = this.width >= 560 ? 132 : 108;
      this.scale = Math.max(1.2, Math.min((this.width - cardW - 40) / cw, availH / ch));
      this.ox = (this.width - cardW - 12) / 2.0 - (cw * this.scale) / 2.0;
      this.oy = this.hudH + (availH - ch * this.scale) / 2.0 + 6;
      this.cardX = this.width - cardW - 6;
      this.cardW = cardW;
      this.buildOverLayout();
    }

    // ------------------------------------------------------- Speicherstand
    /** Bestwerte je Kurs: {"classic": schläge, ...} (kleiner = besser). */
    loadBest() {
      const data = PG.store.get("minigolf", {});
      const best = data && typeof data === "object" ? data.best : null;
      const out = {};
      if (best && typeof best === "object") {
        for (const k in best) {
          const v = parseInt(best[k], 10);
          if (isFinite(v)) out[String(k)] = v;
        }
      }
      return out;
    }

    /** Schlüssel des Bestwerts: je Tour-Kurs und je eigener Bahn einer. */
    bestKey() {
      if (this.course === "tour") return "tour" + this.tour;
      if (this.course === "ugc" && this.singleMap) return "ugc:" + this.singleMap;
      return this.course;
    }

    saveBest(strokes) {
      const key = this.bestKey();
      const old = this.best[key];
      if (old == null || strokes < old) {
        this.best[key] = strokes;
        const data = PG.store.get("minigolf", {});
        const out = data && typeof data === "object" ? data : {};
        out.best = this.best;
        PG.store.set("minigolf", out);
      }
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ------------------------------------------------------- Runde / Bahn
    /** Die 9 Bahnen der Runde (Random: gezogen und zufällig gespiegelt). */
    roundHoles() {
      if (this.course === "classic") return HOLES_CLASSIC.map(gen.cloneHole);
      if (this.course === "pro") return HOLES_PRO.map(gen.cloneHole);
      if (this.course === "tour") return gen.courseHoles(this.tour);
      if (this.course === "ugc") return this.ugcHoles();
      // Random: aus handgebauten UND erzeugten Bahnen ziehen
      const pool = HOLES_CLASSIC.concat(HOLES_PRO).map(gen.cloneHole);
      for (let i = 0; i < HOLES_PER_ROUND; i++) {
        pool.push(gen.generate(PG.rand.randint(1, gen.TOUR_COURSES), PG.rand.randrange(HOLES_PER_ROUND)));
      }
      const picked = PG.rand.sample(pool, HOLES_PER_ROUND);
      picked.sort((a, b) => a.par - b.par);
      return picked.map((h) => MiniGolfGame.mirror(h, PG.rand.random() < 0.5));
    }

    /**
     * Die eigenen Bahnen als Runde. Eine einzelne Bahn wird gespielt, wenn sie
     * aus dem MAPS-Reiter heraus gestartet wurde (this.singleMap). Sonst laufen
     * alle eigenen Bahnen der Reihe nach - höchstens neun.
     */
    ugcHoles() {
      const ugc = edit.ugc;
      if (this.singleMap) {
        const m = ugc.get(this.singleMap);
        if (m) return [ugc.toHole(m)];
      }
      const maps = ugc.loadMaps();
      if (!maps.length) return [gen.cloneHole(HOLES_CLASSIC[0])]; // Notnagel: nie ohne Bahn
      return maps.slice(0, HOLES_PER_ROUND).map(ugc.toHole);
    }

    /**
     * Spiegelt eine Bahn an der Mittelachse (für den Zufallskurs).
     * Alles, was eine Richtung hat, dreht dabei sein x-Vorzeichen um.
     * Mühlen und Drehscheiben laufen andersherum.
     */
    static mirror(hole, flip) {
      hole = gen.normalize(hole);
      if (!flip) return hole;
      const cw = hole.w;
      const mx = (x, w = 0.0) => cw - x - w;
      const out = Object.assign({}, hole);
      out.tee = [mx(hole.tee[0]), hole.tee[1]];
      out.cup = [mx(hole.cup[0]), hole.cup[1]];
      for (const key of ["walls", "sand", "water", "ice", "sticky"]) {
        out[key] = hole[key].map(([x, y, w, h]) => [mx(x, w), y, w, h]);
      }
      out.slopes = hole.slopes.map(([x, y, w, h, ax, ay]) => [mx(x, w), y, w, h, -ax, ay]);
      out.bumpers = hole.bumpers.map(([x, y, r]) => [mx(x), y, r]);
      out.movers = hole.movers.map(([x, y, w, h, dx, dy, sp]) => [mx(x, w), y, w, h, -dx, dy, sp]);
      out.mills = hole.mills.map(([x, y, ln, arms, sp]) => [mx(x), y, ln, arms, -sp]);
      out.tunnels = hole.tunnels.map(([x1, y1, x2, y2, r]) => [mx(x1), y1, mx(x2), y2, r]);
      out.boosters = hole.boosters.map(([x, y, w, h, dx, dy, b]) => [mx(x, w), y, w, h, -dx, dy, b]);
      out.magnets = hole.magnets.map(([x, y, r, f]) => [mx(x), y, r, f]);
      out.gates = hole.gates.map(([x, y, w, h, dx, dy]) => [mx(x, w), y, w, h, -dx, dy]);
      out.spinners = hole.spinners.map(([x, y, r, sp]) => [mx(x), y, r, -sp]);
      out.jumps = hole.jumps.map(([x, y, w, h, dx, dy, d]) => [mx(x, w), y, w, h, -dx, dy, d]);
      return out;
    }

    newRound() {
      this.holes = this.roundHoles();
      this.card = new Array(this.holes.length).fill(0);
      this.points = 0;
      this.holeIdx = 0;
      this.score = 0;
      this.gameOver = false;
      this.recNew();
      this.startHole();
    }

    startHole() {
      const h = gen.normalize(this.holes[this.holeIdx]);
      this.hole = h;
      // Bahngröße übernehmen und den Platz neu einpassen.
      if (h.w !== this.cw || h.h !== this.ch) {
        this.cw = h.w;
        this.ch = h.h;
        this.layout();
      }
      this.par = h.par;
      this.strokes = 0;
      this.cup = [Number(h.cup[0]), Number(h.cup[1])];
      this.bx = Number(h.tee[0]);
      this.by = Number(h.tee[1]);
      this.vx = this.vy = 0.0;
      this.safe = [this.bx, this.by];
      this.phase = "aim";
      if (!this.powerLock) this.power = 0.35;
      this.charging = false;
      this.shotTime = 0.0;
      this.millA = 0.0;
      this.moveT = 0.0;
      this.tunCd = 0.0;
      this.boostCd = 0.0;
      this.air = 0.0;
      this.trail = [];
      this.msg = null;
      this.msgT = 0.0;
      this.resultKey = null;
      this.resultPts = 0;
      this.layoutId = this.rec ? this.rec.layout(h) : 0;
      if (this.aim == null) this.aim = -Math.PI / 2;
      this.resetAim(true);
    }

    // ===================================================== Setup-Screen
    /**
     * Vier Blöcke: Kurs, Tour-Kurs, Schalterzeile, Start. Die drei Schalter
     * (Ziellinie, Autoziel, Aufnehmen) teilen sich eine Zeile.
     */
    buildSetupLayout() {
      const cx = this.width >> 1;
      const bw = Math.min(Math.max(370, Math.trunc(this.width * 0.58)), this.width - 50);
      const gap = 8;
      // Reiterzeile SPIEL | MAPS unter der Unterzeile.
      const tabH = Math.max(22, Math.min(30, Math.floor(this.height / 15)));
      const tabW = Math.min(150, (this.width - 40) >> 1);
      const tabY = Math.trunc(this.height * 0.215);
      this.tabRects = [new PG.Rect(cx - tabW - 4, tabY, tabW, tabH), new PG.Rect(cx + 4, tabY, tabW, tabH)];
      this.tabBottom = tabY + tabH;
      const top = this.tabBottom + Math.max(22, this.fTiny.height + 6);
      const bottom = this.height - 42; // Platz für Bestwert- und Tastenzeile
      const bh = Math.max(26, Math.min(42, Math.trunc((bottom - top - 68) / 4)));
      // Was an Höhe übrig ist, kommt zur Hälfte auf die Beschriftungsabstände.
      const lab = 20 + Math.max(0, Math.min(40, Math.floor((bottom - top - 8 - 4 * bh - 60) / 6)));
      const step = bh + lab;

      const row = (y, n, w = bw, x0 = cx - bw / 2, h = bh) => {
        // n gleich breite Felder nebeneinander (Standard: volle Breite).
        const cw = (w - gap * (n - 1)) / n;
        return Array.from({ length: n }, (_, i) => new PG.Rect(Math.trunc(x0 + i * (cw + gap)), y, Math.trunc(cw), h));
      };

      let y = top;
      this.courseRects = row(y, COURSES.length);
      y += step;
      // Tour-Kurs: Pfeil links, Anzeige, Pfeil rechts
      this.tourRects = [new PG.Rect(Math.trunc(cx - bw / 2), y, 40, bh), new PG.Rect(Math.trunc(cx - bw / 2 + 46), y, Math.trunc(bw - 92), bh),
        new PG.Rect(Math.trunc(cx + bw / 2 - 40), y, 40, bh)];
      y += step;
      // Schalterzeile: drei AN/AUS-Paare nebeneinander.
      const grp = (bw - 2 * 18) / 3.0;
      this.optW = Math.trunc(grp);
      this.guideRects = row(y, 2, grp);
      this.autoaimRects = row(y, 2, grp, cx - grp / 2);
      this.pickupRects = row(y, 2, grp, cx + bw / 2 - grp);
      y += step + 4;
      const sw = Math.max(190, Math.trunc(bw * 0.42));
      this.startRect = new PG.Rect(cx - (sw >> 1), y, sw, bh + 4);
    }

    handleSetup(ev) {
      // Reiterwechsel geht in beiden Reitern zuerst.
      if (ev.kind === "mousedown") {
        for (let i = 0; i < this.tabRects.length; i++) {
          if (this.tabRects[i].collidepoint(ev.pos)) {
            this.setTab(TABS[i]);
            return;
          }
        }
      } else if (ev.kind === "keydown" && (ev.key === "Tab" || ev.key === "ISO_Left_Tab")) {
        this.setTab(this.setupTab === "play" ? "maps" : "play");
        return;
      }
      if (this.setupTab === "maps") {
        if (ev.kind === "keydown" && ev.key === "Escape") {
          this.setTab("play");
          return;
        }
        this.maps.handle(ev);
        return;
      }
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4", "5"].includes(k)) {
          this.course = COURSES[Number(k) - 1];
          this.saveSetting("course", this.course);
          this.playSound("click");
        } else if (k === "Left" || this.isAction(k, "left")) this.stepTour(-1);
        else if (k === "Right" || this.isAction(k, "right")) this.stepTour(1);
        else if (k === "g" || k === "G") this.toggleGuide();
        else if (k === "z" || k === "Z") this.toggleAutoaim();
        else if (k === "p" || k === "P") this.togglePickup();
        else if (k === "Return" || k === "space") this.startPlay();
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.courseRects.length; i++) {
          if (this.courseRects[i].collidepoint(ev.pos)) {
            this.course = COURSES[i];
            this.saveSetting("course", this.course);
            this.playSound("click");
            return;
          }
        }
        if (this.course === "tour") {
          if (this.tourRects[0].collidepoint(ev.pos)) {
            this.stepTour(-1);
            return;
          }
          if (this.tourRects[2].collidepoint(ev.pos)) {
            this.stepTour(1);
            return;
          }
        }
        for (const [rects, val, toggle] of [[this.guideRects, this.guide, () => this.toggleGuide()],
          [this.autoaimRects, this.autoaim, () => this.toggleAutoaim()], [this.pickupRects, this.pickup, () => this.togglePickup()]]) {
          for (let i = 0; i < rects.length; i++) {
            if (rects[i].collidepoint(ev.pos)) {
              if (val !== (i === 0)) toggle();
              return;
            }
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    /** Blättert durch die Tour-Kurse (nur wirksam, wenn Tour gewählt ist). */
    stepTour(d) {
      if (this.course !== "tour") return;
      this.tour = PG.mod(this.tour - 1 + d, gen.TOUR_COURSES) + 1;
      this.tourPar = gen.coursePar(this.tour);
      this.saveSetting("tour", this.tour);
      this.playSound("move");
    }

    toggleGuide() {
      this.guide = !this.guide;
      this.saveSetting("guide", this.guide);
      this.playSound("select");
    }

    toggleAutoaim() {
      this.autoaim = !this.autoaim;
      this.saveSetting("autoaim", this.autoaim);
      this.playSound("select");
    }

    togglePickup() {
      this.pickup = !this.pickup;
      this.saveSetting("pickup", this.pickup);
      this.playSound("select");
    }

    /**
     * Runde starten. single ist die id genau einer eigenen Bahn (aus dem
     * MAPS-Reiter). Ohne sie spielt "Eigene" die ganze Sammlung nacheinander.
     */
    startPlay(single) {
      if (this.course === "ugc") this.singleMap = single || "";
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Eigene Bahnen
    // Der MAPS-Reiter und der Editor stecken in minigolf_edit.js; hier stehen
    // nur die Übergänge, die beide brauchen.

    /**
     * ESC heißt im Editor und im MAPS-Reiter "Abbrechen", nicht "Pause".
     * Auch beim Test-Spielen aus dem Editor: dort bricht ESC den Versuch ab.
     */
    get wantsEscape() {
      return this.state === EDIT || !!this.testReturn || (this.state === SETUP && this.setupTab === "maps");
    }

    get wantsRightClick() {
      return true;
    }

    get gridSnap() {
      return this.opts.grid !== false;
    }

    setGridSnap(on) {
      this.saveSetting("grid", !!on);
    }

    setTab(tab) {
      if (!TABS.includes(tab) || tab === this.setupTab) return;
      this.setupTab = tab;
      if (tab === "maps") {
        if (!this.maps) this.maps = new edit.MapList(this);
        else {
          this.maps.layout();
          this.maps.reload();
        }
      }
      this.playSound("click");
    }

    /** Neue eigene Bahn anlegen und gleich in den Editor springen. */
    ugcNewMap() {
      this.ugcEdit(edit.ugc.newMap("", "", edit.ugc.lastAuthor()));
    }

    ugcEdit(m) {
      this.editor = new edit.MapEditor(this, m);
      this.state = EDIT;
      this.playSound("click");
    }

    /** Zurück aus dem Editor in den MAPS-Reiter. */
    ugcCloseEditor() {
      this.editor = null;
      this.state = SETUP;
      this.setupTab = "maps";
      if (!this.maps) this.maps = new edit.MapList(this);
      else this.maps.reload();
      this.playSound("click");
    }

    /** Genau eine eigene Bahn spielen (aus dem MAPS-Reiter). */
    ugcPlay(mapId) {
      this.course = "ugc";
      this.saveSetting("course", "ugc");
      this.saveSetting("ugc_map", mapId);
      this.best = this.loadBest();
      this.startPlay(mapId);
    }

    /** Bahn aus dem Editor sofort ausprobieren (danach zurück in den Editor). */
    ugcTest(hole) {
      this.testReturn = true;
      this.course = "ugc";
      this.holes = [gen.normalize(hole)];
      this.card = [0];
      this.points = 0;
      this.holeIdx = 0;
      this.score = 0;
      this.gameOver = false;
      this.startHole();
      this.state = PLAY;
      this.playSound("click");
    }

    /** Nach dem Test-Spielen zurück in den Editor. */
    backToEditor() {
      this.testReturn = false;
      this.state = EDIT;
      this.gameOver = false;
      if (!this.editor) this.ugcCloseEditor();
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      // Die rechte Maustaste ist die Stärke-Sperre - und zwar in jedem
      // Zustand zuerst, damit sie nirgends als Linksklick durchrutscht.
      if ((ev.kind === "mousedown" || ev.kind === "mouseup") && ev.button === 3) {
        this.lockPower(ev.kind === "mousedown");
        return;
      }
      // Der Bahn-Editor hat eine eigene, vollständige Bedienung.
      if (this.state === EDIT && this.editor) {
        this.editor.handle(ev);
        return;
      }
      // Test-Spielen: ESC bricht ab und führt zurück in den Editor.
      if (this.testReturn && ev.kind === "keydown" && ev.key === "Escape") {
        this.backToEditor();
        return;
      }
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      const key = ev.kind === "keydown" ? ev.key : null;
      if (key === "g" || key === "G") return this.toggleGuide();
      if (key === "z" || key === "Z") return this.toggleAutoaim();
      if (key === "p" || key === "P") {
        // Am Rundenende zeigt P die Wiederholung, sonst schaltet es das
        // Aufnehmen um (wie in der Desktop-Version).
        if (this.state === OVER && this.replay) this.openReplay();
        else this.togglePickup();
        return;
      }
      if (key === "f" || key === "F") return this.resetHole();
      if (this.state === HOLE_DONE) {
        if (key === "Return" || key === "space" || (ev.kind === "mousedown" && ev.button === 1)) this.advance();
        return;
      }
      if (this.state === OVER) {
        // Knöpfe: Weiter (nächster Kurs) · Nochmal · Setup.
        if (ev.kind === "keydown") {
          if (key === "Return" || key === "space") this.overAction(this.nextCourse() ? "next" : "again");
          else if (key === "r" || key === "R") this.overAction("again");
          else if (key === "s" || key === "S") this.overAction("setup");
        } else if (ev.kind === "mousedown" && ev.button === 1) {
          for (const [k, rc] of this.overRects) {
            if (rc.collidepoint(ev.pos)) {
              this.overAction(k);
              break;
            }
          }
        }
        return;
      }
      if (this.state !== PLAY || this.phase !== "aim") return;
      if (ev.kind === "mousemove") {
        const [mx, my] = this.unproject(ev.pos[0], ev.pos[1]);
        if (Math.abs(mx - this.bx) > 0.4 || Math.abs(my - this.by) > 0.4) this.aim = Math.atan2(my - this.by, mx - this.bx);
      } else if (ev.kind === "mousedown" && ev.button === 1) {
        this.charging = true;
        if (!this.powerLock) this.power = 0.05; // gesperrt = mit dem Wert schlagen
      } else if (ev.kind === "mouseup" && ev.button === 1) {
        if (this.charging) {
          this.charging = false;
          this.strike();
        }
      } else if (ev.kind === "keydown") {
        const k = key;
        if (k === "Left" || this.isAction(k, "left")) this.aim -= PG.radians(2.5);
        else if (k === "Right" || this.isAction(k, "right")) this.aim += PG.radians(2.5);
        else if (k === "Up" || this.isAction(k, "up")) {
          if (!this.powerLock) this.power = Math.min(1.0, this.power + 0.05);
        } else if (k === "Down" || this.isAction(k, "down")) {
          if (!this.powerLock) this.power = Math.max(0.05, this.power - 0.05);
        } else if (k === "r" || k === "R") this.cancelShot();
        else if (k === "space" || k === "Return") this.strike();
      }
    }

    /**
     * Stärke-Sperre: die rechte Maustaste hält die Schlagstärke fest.
     * Gesperrt bleibt die Stärke, bis die Taste losgelassen wird: sie
     * übersteht Schlag, Bahnwechsel und [R]. Gesperrt wird nur beim Zielen;
     * freigegeben immer, damit die Sperre nie hängen bleibt.
     */
    lockPower(on) {
      if (on && (this.state !== PLAY || this.phase !== "aim")) return;
      if (on === this.powerLock) return;
      this.powerLock = on;
      this.lockT = 0.0;
      this.playSound(on ? "select" : "click");
    }

    /**
     * Bricht einen geladenen Schlag ab (Taste R). Der Ball bleibt liegen, der
     * Schlag zählt nicht. Eine gehaltene Stärke-Sperre bleibt dabei stehen.
     */
    cancelShot() {
      if (!this.charging) return;
      this.charging = false;
      if (!this.powerLock) this.power = 0.35;
      this.msg = t("golf.cancel");
      this.msgT = 1.4;
      this.playSound("click");
    }

    strike() {
      const sp = MAX_SPEED * this.power;
      this.vx = Math.cos(this.aim) * sp;
      this.vy = Math.sin(this.aim) * sp;
      this.strokes += 1;
      this.phase = "rolling";
      this.shotTime = 0.0;
      this.physAcc = 0.0;
      this.safe = [this.bx, this.by];
      this.trail = [];
      if (this.rec) {
        this.rec.scene({
          layout: this.layoutId, hole: this.holeIdx, player: 0, n: this.strokes,
          aim: Math.round(this.aim * 1e4) / 1e4, power: Math.round(this.power * 1e3) / 1e3,
          mill: Math.round(this.millA * 1e3) / 1e3, move: Math.round(this.moveT * 1e3) / 1e3,
        }, true);
      }
      this.playSound("shoot");
      this.rumble(50);
    }

    // ===================================================== Projektion
    view() {
      return new mdraw.View(this.ox, this.oy, this.scale);
    }

    project(x, y) {
      return [this.ox + x * this.scale, this.oy + y * this.scale];
    }

    unproject(sx, sy) {
      return [(sx - this.ox) / this.scale, (sy - this.oy) / this.scale];
    }

    // ===================================================== Update / Physik
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state === EDIT) {
        if (this.editor) this.editor.update(dt);
        return;
      }
      if (this.state === SETUP && this.setupTab === "maps" && this.maps) {
        this.maps.update(dt);
        return;
      }
      if (this.state !== PLAY) return;
      this.millA += dt;
      this.moveT += dt;
      if (this.phase === "aim") {
        if (this.powerLock) this.lockT += dt; // Puls der goldenen Anzeige
        else if (this.charging) this.power = Math.min(1.0, this.power + dt * 0.8);
      } else if (this.phase === "rolling") {
        // Physik in festen 1/60-s-Schritten (siehe PHYS_DT). Der Rest darf um
        // einen halben Schritt ins Minus laufen - so gibt es bei 60 Hz genau
        // einen Schritt je Frame und kein Ruckeln durch Takt-Schwankungen.
        this.physAcc = (this.physAcc || 0) + dt;
        while (this.physAcc > PHYS_DT * 0.5 && this.phase === "rolling" && this.state === PLAY) {
          this.physics(PHYS_DT);
          this.physAcc -= PHYS_DT;
          if (this.vx === 0.0 && this.vy === 0.0) break;
        }
        if (this.rec) this.rec.tick(dt, () => this.recSample());
        this.shotTime += dt;
        if (this.shotTime > MAX_SHOT_TIME) {
          this.vx = this.vy = 0.0;
          this.air = 0.0; // notfalls auch aus dem Flug holen
        }
        if (this.phase === "rolling" && this.vx === 0.0 && this.vy === 0.0) this.afterShot();
      }
    }

    physics(dt) {
      const speed = Math.hypot(this.vx, this.vy);
      const steps = Math.max(2, Math.min(24, Math.trunc((speed * dt) / BR) + 2));
      const h = dt / steps;
      const hole = this.hole;
      for (let s = 0; s < steps; s++) {
        this.tunCd = Math.max(0.0, this.tunCd - h);
        this.boostCd = Math.max(0.0, this.boostCd - h);
        // --- Sprungrampe: solange der Ball fliegt, gibt es nur die Bande.
        // Hindernisse, Loch und Wasser werden überflogen.
        if (this.air > 0.0) {
          const move = Math.hypot(this.vx, this.vy) * h;
          this.air -= move;
          this.bx += this.vx * h;
          this.by += this.vy * h;
          this.collideBounds();
          // Ohne Tempo gibt es keinen Flug mehr - sonst bliebe der Ball ewig
          // in der Luft.
          if (this.air <= 0.0 || move <= 0.0) {
            this.air = 0.0;
            this.playSound("hit");
          }
          continue;
        }
        const terrain = this.terrain();
        const fr = terrain === "sand" ? FRIC_SAND : terrain === "ice" ? FRIC_ICE : terrain === "sticky" ? FRIC_STICKY : FRIC_GREEN;
        // Rampen beschleunigen, solange der Ball darauf liegt
        for (const [x, y, w, hh, ax, ay] of hole.slopes) {
          if (x <= this.bx && this.bx <= x + w && y <= this.by && this.by <= y + hh) {
            this.vx += ax * h;
            this.vy += ay * h;
          }
        }
        this.applyMagnets(h);
        const f = Math.max(0.0, 1.0 - fr * h);
        this.vx *= f;
        this.vy *= f;
        this.bx += this.vx * h;
        this.by += this.vy * h;
        this.collideBounds();
        for (const r of hole.walls) this.collideRect(r);
        for (const g of hole.gates) this.collideGate(g);
        for (const m of hole.movers) {
          const [rect, vel] = mdraw.moverRect(m, this.moveT);
          this.collideRect(rect, vel);
        }
        for (const b of hole.bumpers) this.collideBumper(b);
        for (const mill of hole.mills) this.collideMill(mill);
        for (const sp of hole.spinners) this.collideSpinner(sp, h);
        this.applyBoosters();
        if (this.checkJumps() || this.checkTunnels()) continue;
        if (this.checkCup() || this.checkWater()) return;
        if (this.vx * this.vx + this.vy * this.vy < STOP_EPS * STOP_EPS) {
          this.vx = this.vy = 0.0;
          return;
        }
      }
      this.trail.push([this.bx, this.by]);
      if (this.trail.length > 26) this.trail.shift();
    }

    /**
     * Untergrund unter dem Ball: green, sand, ice oder sticky. Bei
     * Überlappung gewinnt der bremsendste.
     */
    terrain() {
      for (const key of ["sticky", "sand", "ice"]) {
        for (const [x, y, w, h] of this.hole[key]) {
          if (x <= this.bx && this.bx <= x + w && y <= this.by && this.by <= y + h) return key;
        }
      }
      return "green";
    }

    // ----- Die acht Editor-Hindernisse ---------------------------------
    /** Magnet: zieht den Ball an (force > 0) oder stößt ihn ab. */
    applyMagnets(h) {
      for (const [x, y, r, force] of this.hole.magnets) {
        const dx = x - this.bx, dy = y - this.by;
        const d = Math.hypot(dx, dy);
        if (d >= r || d < 0.4) continue;
        // Nah an der Mitte stärker, am Rand des Feldes gar nicht.
        const pull = (force * (1.0 - d / r) * h) / d;
        this.vx += dx * pull;
        this.vy += dy * pull;
      }
    }

    /** Einbahn-Tor: in Richtung (dx, dy) offen, dagegen eine Wand. */
    collideGate(g) {
      const [x, y, w, h, dx, dy] = g;
      if (this.vx * dx + this.vy * dy > 0) return; // richtige Richtung -> freie Fahrt
      this.collideRect([x, y, w, h]);
    }

    /** Drehscheibe: nimmt den Ball mit und trägt ihn nach außen. */
    collideSpinner(sp, h) {
      const [x, y, r, speed] = sp;
      const dx = this.bx - x, dy = this.by - y;
      const d = Math.hypot(dx, dy);
      if (d >= r) return;
      // Umfangsgeschwindigkeit am Ort des Balls + leichte Fliehkraft
      this.vx += (-dy * speed - this.vx) * SPIN_PUSH * h;
      this.vy += (dx * speed - this.vy) * SPIN_PUSH * h;
      if (d > 0.4) {
        this.vx += (dx / d) * Math.abs(speed) * 4.0 * h;
        this.vy += (dy / d) * Math.abs(speed) * 4.0 * h;
      }
    }

    /** Schub-Feld: einmaliger Stoß beim Betreten (nicht je Teilschritt). */
    applyBoosters() {
      if (this.boostCd > 0.0) return;
      for (const [x, y, w, h, dx, dy, boost] of this.hole.boosters) {
        if (!(x <= this.bx && this.bx <= x + w && y <= this.by && this.by <= y + h)) continue;
        const n = Math.hypot(dx, dy);
        if (n < 1e-6 || boost <= 0) continue;
        const ux = dx / n, uy = dy / n;
        // Auf mindestens 'boost' beschleunigen - schneller wird nie gebremst.
        const along = this.vx * ux + this.vy * uy;
        if (along < boost) {
          this.vx += ux * (boost - along);
          this.vy += uy * (boost - along);
          this.boostCd = BOOST_COOLDOWN;
          this.playSound("bounce");
        }
        return;
      }
    }

    /** Sprungrampe: hebt den Ball ab, wenn er schnell genug drüberrollt. */
    checkJumps() {
      if (this.air > 0.0) return false;
      for (const [x, y, w, h, dx, dy, dist] of this.hole.jumps) {
        if (!(x <= this.bx && this.bx <= x + w && y <= this.by && this.by <= y + h)) continue;
        const sp = Math.hypot(this.vx, this.vy);
        if (sp < JUMP_MIN_SPEED) return false; // zu langsam: die Schanze tut nichts
        const n = Math.hypot(dx, dy);
        if (n > 1e-6) {
          // in Sprungrichtung ausrichten
          this.vx = (dx / n) * sp;
          this.vy = (dy / n) * sp;
        }
        this.air = Math.max(1.0, Number(dist));
        this.playSound("bounce");
        return true;
      }
      return false;
    }

    /** Rohr: versetzt den Ball ans andere Ende, Richtung bleibt erhalten. */
    checkTunnels() {
      if (this.tunCd > 0.0) return false;
      for (const [x1, y1, x2, y2, r] of this.hole.tunnels) {
        for (const [ex, ey, ox, oy] of [[x1, y1, x2, y2], [x2, y2, x1, y1]]) {
          if (Math.hypot(this.bx - ex, this.by - ey) > r) continue;
          const v = Math.hypot(this.vx, this.vy);
          const sp = v * TUNNEL_KEEP;
          if (sp < 1e-6) return false; // liegen geblieben: kein Transport
          const ux = this.vx / v, uy = this.vy / v;
          // Direkt hinter der Mündung wieder ausspucken, sonst fängt das
          // Ziel-Ende den Ball sofort wieder ein.
          this.bx = ox + ux * (r + BR + 0.5);
          this.by = oy + uy * (r + BR + 0.5);
          this.vx = ux * sp;
          this.vy = uy * sp;
          this.tunCd = TUNNEL_COOLDOWN;
          this.playSound("hit");
          return true;
        }
      }
      return false;
    }

    collideBounds() {
      const lo = BORDER + BR;
      const hix = this.cw - BORDER - BR, hiy = this.ch - BORDER - BR;
      if (this.bx < lo) {
        this.bx = lo;
        this.vx = Math.abs(this.vx) * WALL_E;
        this.thud();
      } else if (this.bx > hix) {
        this.bx = hix;
        this.vx = -Math.abs(this.vx) * WALL_E;
        this.thud();
      }
      if (this.by < lo) {
        this.by = lo;
        this.vy = Math.abs(this.vy) * WALL_E;
        this.thud();
      } else if (this.by > hiy) {
        this.by = hiy;
        this.vy = -Math.abs(this.vy) * WALL_E;
        this.thud();
      }
    }

    thud() {
      if (Math.abs(this.vx) + Math.abs(this.vy) > 45) this.playSound("bounce");
    }

    /** Kreis gegen Rechteck: nächster Punkt, herausschieben, reflektieren. */
    collideRect(r, vel = [0.0, 0.0]) {
      const [x, y, w, h] = r;
      const nx = Math.max(x, Math.min(this.bx, x + w));
      const ny = Math.max(y, Math.min(this.by, y + h));
      const dx = this.bx - nx, dy = this.by - ny;
      const d2 = dx * dx + dy * dy;
      if (d2 >= BR * BR) return;
      let ux, uy, push;
      if (d2 > 1e-9) {
        const d = Math.sqrt(d2);
        ux = dx / d;
        uy = dy / d;
        push = BR - d;
      } else {
        // Mittelpunkt im Rechteck -> über die kürzeste Achse hinausschieben
        const left = this.bx - x, right = x + w - this.bx;
        const top = this.by - y, bottom = y + h - this.by;
        const m = Math.min(left, right, top, bottom);
        if (m === left) [ux, uy, push] = [-1.0, 0.0, left + BR];
        else if (m === right) [ux, uy, push] = [1.0, 0.0, right + BR];
        else if (m === top) [ux, uy, push] = [0.0, -1.0, top + BR];
        else [ux, uy, push] = [0.0, 1.0, bottom + BR];
      }
      this.bx += ux * push;
      this.by += uy * push;
      let rvx = this.vx - vel[0], rvy = this.vy - vel[1];
      const dot = rvx * ux + rvy * uy;
      if (dot < 0) {
        rvx -= (1 + WALL_E) * dot * ux;
        rvy -= (1 + WALL_E) * dot * uy;
        this.vx = rvx + vel[0] * 1.4;
        this.vy = rvy + vel[1] * 1.4;
        this.thud();
      }
    }

    collideBumper(b) {
      const [x, y, r] = b;
      const dx = this.bx - x, dy = this.by - y;
      const d = Math.hypot(dx, dy);
      const rad = r + BR;
      if (d >= rad || d < 1e-9) return;
      const ux = dx / d, uy = dy / d;
      this.bx = x + ux * rad;
      this.by = y + uy * rad;
      const dot = this.vx * ux + this.vy * uy;
      if (dot < 0) {
        this.vx -= (1 + BUMP_E) * dot * ux;
        this.vy -= (1 + BUMP_E) * dot * uy;
        this.playSound("bounce");
      }
    }

    collideMill(mill) {
      const [x, y, length, arms, speed] = mill;
      const base = this.millA * speed;
      const n = Math.trunc(arms);
      for (let i = 0; i < n; i++) {
        const a = base + i * ((2 * Math.PI) / n);
        const ex = x + Math.cos(a) * length, ey = y + Math.sin(a) * length;
        const [px, py] = MiniGolfGame.closestOnSeg(x, y, ex, ey, this.bx, this.by);
        let dx = this.bx - px, dy = this.by - py;
        let d = Math.hypot(dx, dy);
        const rad = BR + ARM_W;
        if (d >= rad) continue;
        if (d < 1e-9) [dx, dy, d] = [0.0, -1.0, 1.0];
        const ux = dx / d, uy = dy / d;
        this.bx = px + ux * rad;
        this.by = py + uy * rad;
        // Umfangsgeschwindigkeit des Flügels am Kontaktpunkt
        const rvx = -(py - y) * speed, rvy = (px - x) * speed;
        let relx = this.vx - rvx, rely = this.vy - rvy;
        const dot = relx * ux + rely * uy;
        if (dot < 0) {
          relx -= 1.8 * dot * ux;
          rely -= 1.8 * dot * uy;
        }
        this.vx = relx + rvx;
        this.vy = rely + rvy;
        this.playSound("hit");
        return;
      }
    }

    static closestOnSeg(x1, y1, x2, y2, px, py) {
      const dx = x2 - x1, dy = y2 - y1;
      const l2 = dx * dx + dy * dy;
      if (l2 < 1e-9) return [x1, y1];
      const f = Math.max(0.0, Math.min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2));
      return [x1 + f * dx, y1 + f * dy];
    }

    checkCup() {
      const dx = this.cup[0] - this.bx, dy = this.cup[1] - this.by;
      const d = Math.hypot(dx, dy);
      const sp = Math.hypot(this.vx, this.vy);
      if (d < CUP_R + BR && sp < CAPTURE_SPEED * 1.6) {
        // Sog Richtung Loch - fühlt sich an wie eine echte Lochkante
        const pull = (26.0 * (1.0 - d / (CUP_R + BR))) / Math.max(d, 0.4);
        this.vx += dx * pull;
        this.vy += dy * pull;
      }
      if (d < CUP_R * 0.75 && sp < CAPTURE_SPEED) {
        this.vx = this.vy = 0.0;
        this.bx = this.cup[0];
        this.by = this.cup[1];
        this.holed();
        return true;
      }
      return false;
    }

    checkWater() {
      for (const [x, y, w, h] of this.hole.water) {
        if (x <= this.bx && this.bx <= x + w && y <= this.by && this.by <= y + h) {
          this.vx = this.vy = 0.0;
          this.strokes += 1;
          this.recEnd("water");
          [this.bx, this.by] = this.safe;
          this.msg = t("golf.penalty");
          this.msgT = 2.0;
          this.playSound("hit");
          if (this.pickup && this.strokes >= MAX_STROKES) this.finishHole(false);
          else {
            this.phase = "aim";
            this.resetAim();
          }
          return true;
        }
      }
      return false;
    }

    /**
     * Zielrichtung und Kraft für den nächsten Schlag setzen. Mit Autoziel
     * zeigt der Schläger vor jedem Schlag zum Loch. Ohne Autoziel bleibt die
     * zuletzt gewählte Richtung stehen - nur am Tee einer neuen Bahn zeigt er
     * neutral bahnaufwärts. Eine gehaltene Stärke-Sperre überlebt den Schlag.
     */
    resetAim(newHole = false) {
      if (this.autoaim) this.aim = Math.atan2(this.cup[1] - this.by, this.cup[0] - this.bx);
      else if (newHole) this.aim = -Math.PI / 2;
      if (!this.powerLock) this.power = 0.35;
      this.charging = false;
    }

    afterShot() {
      this.recEnd("stop");
      if (this.pickup && this.strokes >= MAX_STROKES) {
        this.msg = t("golf.max_strokes");
        this.msgT = 2.4;
        this.finishHole(false);
        return;
      }
      this.phase = "aim";
      this.resetAim();
    }

    // ===================================================== Bahn abschließen
    holed() {
      this.recEnd("cup");
      this.playSound(this.strokes === 1 ? "win" : "point");
      this.rumble(90);
      const [px, py] = this.project(this.cup[0], this.cup[1]);
      ui.spawnBurst(px, py, this.accent, 16);
      this.finishHole(true);
    }

    finishHole(holed) {
      this.card[this.holeIdx] = this.strokes;
      let pts;
      if (holed) {
        pts = Math.max(100, 600 + (this.par - this.strokes) * 300);
        if (this.strokes === 1) {
          pts += 500;
          this.achEvent("golf_ace");
        }
      } else {
        pts = 100;
      }
      this.points += pts;
      this.resultPts = pts;
      this.resultKey = MiniGolfGame.resultKey(this.strokes, this.par, holed);
      if (this.rec) this.rec.setLast({ final: this.strokes, result: this.resultKey, pts, holed: !!holed });
      this.score = this.points;
      this.phase = "done";
      this.state = HOLE_DONE;
    }

    static resultKey(strokes, par, holed) {
      if (!holed) return "golf.res.max";
      if (strokes === 1) return "golf.res.ace";
      const d = strokes - par;
      if (d <= -2) return "golf.res.eagle";
      if (d === -1) return "golf.res.birdie";
      if (d === 0) return "golf.res.par";
      if (d === 1) return "golf.res.bogey";
      if (d === 2) return "golf.res.double";
      return "golf.res.over";
    }

    /** Nächste Bahn (oder Rundenende). */
    advance() {
      this.playSound("click");
      if (this.holeIdx + 1 < this.holes.length) {
        this.holeIdx += 1;
        this.state = PLAY;
        this.startHole();
        return;
      }
      this.endRound();
    }

    // ------------------------------------------------- Rundenende / Weiter
    /**
     * [Kurs, Tour-Nummer] des nächsten Kurses - oder null.
     * Reihenfolge: Classic -> Pro -> Tour 1 -> Tour 2 -> ... -> Tour 38.
     */
    nextCourse() {
      if (this.course === "classic") return ["pro", this.tour];
      if (this.course === "pro") return ["tour", 1];
      if (this.course === "tour" && this.tour < gen.TOUR_COURSES) return ["tour", this.tour + 1];
      return null;
    }

    /** Name des nächsten Kurses für die Knopfbeschriftung. */
    nextCourseLabel() {
      const nxt = this.nextCourse();
      if (!nxt) return "";
      const [course, num] = nxt;
      const name = t("golf.course." + course);
      return course === "tour" ? `${name} ${num}` : name;
    }

    /** Knöpfe des Rundenende-Bildschirms (nur die Schlüssel). */
    overKeys() {
      const keys = ["again", "setup"];
      if (this.nextCourse()) keys.unshift("next");
      return keys;
    }

    /**
     * Banner-Höhe, Zeilen-Positionen und Knopfreihe des Rundenendes. Alles
     * wächst mit den Schriftgrößen mit.
     */
    buildOverLayout() {
      const keys = this.overKeys();
      const headH = this.fHuge.height, lineH = this.fSmall.height, tinyH = this.fTiny.height;
      const bh = Math.max(26, Math.min(38, lineH + 10));
      let y = OVER_PAD;
      this.overY = {};
      for (const [name, h, gap] of [["head", headH, 6], ["sub", lineH, 4], ["best", tinyH, 10]]) {
        this.overY[name] = y + (h >> 1);
        y += h + gap;
      }
      const btnTop = y;
      y += bh + 8;
      this.overY.hint = y + (tinyH >> 1);
      this.overH = y + tinyH + OVER_PAD;
      const gap = 8;
      const bw = Math.min(Math.trunc(this.width * 0.74), 660);
      const cw = (bw - gap * (keys.length - 1)) / keys.length;
      const cx = this.width >> 1;
      const top = (this.height >> 1) - (this.overH >> 1) + btnTop;
      this.overRects = keys.map((key, i) => [key, new PG.Rect(Math.trunc(cx - bw / 2 + i * (cw + gap)), top, Math.trunc(cw), bh)]);
    }

    /** Weiter-Knopf: nächsten Kurs laden und sofort abschlagen. */
    continueNext() {
      const nxt = this.nextCourse();
      if (!nxt) {
        this.restart();
        return;
      }
      [this.course, this.tour] = nxt;
      this.tourPar = gen.coursePar(this.tour);
      this.saveSetting("course", this.course);
      this.saveSetting("tour", this.tour);
      this.buildOverLayout();
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    /** Führt einen Knopf des Rundenende-Bildschirms aus. */
    overAction(key) {
      if (key === "next" && this.nextCourse()) this.continueNext();
      else if (key === "setup") {
        this.state = SETUP;
        this.gameOver = false;
        this.playSound("click");
      } else this.restart();
    }

    /**
     * Taste F: die laufende Bahn von vorn. Schläge zurück auf 0, Ball zurück
     * aufs Tee. Bereits abgeschlossene Bahnen bleiben in der Scorekarte.
     */
    resetHole() {
      if (this.state !== PLAY) return;
      if (this.rec) this.rec.dropWhere({ hole: this.holeIdx, player: 0 });
      this.startHole();
      this.msg = t("golf.reset");
      this.msgT = 1.4;
      this.playSound("click");
    }

    endRound() {
      // Test-Spielen aus dem Editor: kein Bestwert, kein Erfolg, keine
      // Statistik - es geht direkt zurück ans Bauen.
      if (this.testReturn) {
        this.backToEditor();
        return;
      }
      const total = sum(this.card);
      const parTotal = sum(this.holes.map((h) => h.par));
      this.saveBest(total);
      if (total < parTotal) this.achEvent("golf_under_par");
      this.reportResult(total <= parTotal);
      this.recFinish(total, parTotal);
      this.score = this.points;
      this.buildOverLayout();
      this.state = OVER;
      this.gameOver = true;
      this.playSound("win");
    }

    restart() {
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Replay-Aufnahme
    // Aufgezeichnet wird je Schlag die tatsächliche BAHN des Balls (flache
    // Zahlenliste x, y, x, y ...). Die Bahn selbst liegt als Kulisse im
    // Replay, damit die Wiederholung auch dann noch stimmt, wenn spätere
    // Versionen die Bahnen ändern. Format wie in der Desktop-Version.

    recNew() {
      this.replay = null;
      this.rec = PG.replay.recorder("minigolf", {
        course: this.course, tour: this.tour, players: 1,
      });
    }

    /** Ein Sample = die Ballposition (auf 1/100 Feldeinheit gerundet). */
    recSample() {
      return [Math.round(this.bx * 100) / 100, Math.round(this.by * 100) / 100];
    }

    /** Beendet die laufende Schlag-Sequenz ("cup"/"water"/"stop"). */
    recEnd(end) {
      if (this.rec) this.rec.close(() => this.recSample(), { end, after: this.strokes });
    }

    recFinish(total, parTotal) {
      if (!this.rec) return;
      const d = total - parTotal;
      let name = t("golf.course." + this.course);
      if (this.course === "tour") name += " " + this.tour;
      this.replay = this.rec.result({
        title: name,
        sub: t("golf.final", { strokes: total, diff: d ? (d > 0 ? "+" + d : String(d)) : t("golf.even"), pts: this.points }),
        total, par: parTotal, points: this.points,
      });
      this.rec = null;
    }

    /** Die Wiederholung ansehen (Taste P) - den Screen öffnet app.js. */
    openReplay() {
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
      // Aufnahmen aus älteren Versionen kennen weder die Bahngröße noch die
      // neuen Hindernis-Typen - normalize() ergänzt beides.
      this.repLayouts = (rep.layouts || []).map((lay) => gen.normalize(lay));
      this.repAt = null;
      const meta = rep.meta || {};
      if (COURSES.includes(meta.course)) this.course = meta.course;
      this.tour = Math.max(1, Math.min(gen.TOUR_COURSES, Math.trunc(Number(meta.tour)) || 1));
      // Bahnen der Runde aus den Szenen ableiten (Reihenfolge = Spielverlauf).
      const holes = new Map();
      const order = [];
      for (const sc of rep.scenes || []) {
        const h = sc.hole | 0;
        if (!holes.has(h)) {
          const idx = sc.layout | 0;
          if (idx >= 0 && idx < this.repLayouts.length) {
            holes.set(h, this.repLayouts[idx]);
            order.push(h);
          }
        }
      }
      this.holes = order.map((h) => holes.get(h));
      this.repPos = new Map(order.map((h, i) => [h, i]));
      this.card = new Array(this.holes.length).fill(0);
      this.points = 0;
      this.state = PLAY;
      this.phase = "aim";
      this.gameOver = false;
      this.msg = null;
      this.msgT = 0;
      this.charging = false;
      this.powerLock = false;
      this.holeIdx = 0;
      this.strokes = 0;
      this.trail = [];
      this.replaySeek(0, 0);
    }

    replaySeek(index, frame) {
      const scenes = this.rep.scenes || [];
      if (!scenes.length) return;
      index = PG.clamp(index, 0, scenes.length - 1);
      const sc = scenes[index];
      const lay = sc.layout | 0;
      if (lay >= 0 && lay < this.repLayouts.length) this.hole = this.repLayouts[lay];
      if (this.hole.w !== this.cw || this.hole.h !== this.ch) {
        this.cw = this.hole.w;
        this.ch = this.hole.h;
        this.layout();
      }
      this.holeIdx = this.repPos.has(sc.hole | 0) ? this.repPos.get(sc.hole | 0) : 0;
      this.par = this.hole.par || 3;
      this.cup = [Number(this.hole.cup[0]), Number(this.hole.cup[1])];
      this.aim = Number(sc.aim) || 0;
      this.power = Number(sc.power != null ? sc.power : 0.35);
      this.resultKey = sc.result || null;
      this.resultPts = sc.pts | 0;

      const pts = sc.f || [];
      const n = Math.max(1, Math.floor(pts.length / 2));
      frame = PG.clamp(frame, 0, n - 1);
      this.bx = pts[2 * frame];
      this.by = pts[2 * frame + 1];
      this.trail = [];
      for (let k = Math.max(0, frame - 26); k < frame; k++) this.trail.push([pts[2 * k], pts[2 * k + 1]]);
      const rate = this.rep.rate || PG.replay.RATE;
      this.millA = Number(sc.mill || 0) + frame / rate;
      this.moveT = Number(sc.move || 0) + frame / rate;

      // Schlagzahl + Scorekarte aus dem bisherigen Verlauf aufbauen.
      const last = frame >= n - 1;
      this.strokes = last ? (sc.after != null ? sc.after : sc.n || 1) : sc.n || 1;
      this.card = new Array(this.holes.length).fill(0);
      this.points = 0;
      for (const prev of scenes.slice(0, index).concat(last ? [sc] : [])) {
        const pos = this.repPos.get(prev.hole | 0);
        if (prev.final && pos != null) {
          this.card[pos] = prev.final;
          this.points += prev.pts | 0;
        }
      }
      this.score = this.points;
      this.msg = last && sc.end === "water" ? t("golf.penalty") : null;
      this.repAt = [index, frame];
    }

    replayDraw(ctx, aiming, banner) {
      ui.drawBackground(ctx, this.width, this.height);
      mdraw.drawCourse(ctx, this.hole, this.view(), this.millA, this.moveT, CUP_R);
      this.drawBall(ctx);
      if (aiming) this.drawAim(ctx);
      this.drawHud(ctx);
      this.drawCard(ctx);
      if (banner && this.resultKey) this.drawHoleDone(ctx);
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Sicherheitsnetz: In der Pause kommt kein Loslassen der rechten
      // Maustaste mehr an - eine Pause hebt die Stärke-Sperre deshalb auf.
      if (this.powerLock && this.paused) this.powerLock = false;
      ui.drawBackground(ctx, this.width, this.height);
      if (this.state === EDIT && this.editor) {
        this.editor.draw(ctx);
        return;
      }
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      mdraw.drawCourse(ctx, this.hole, this.view(), this.millA, this.moveT, CUP_R);
      this.drawBall(ctx);
      if (this.state === PLAY && this.phase === "aim") this.drawAim(ctx);
      this.drawHud(ctx);
      this.drawCard(ctx);
      if (this.state === HOLE_DONE) this.drawHoleDone(ctx);
      else if (this.state === OVER) this.drawOver(ctx);
    }

    drawBall(ctx) {
      const [px, py] = this.project(this.bx, this.by);
      const r = Math.max(2, Math.trunc(BR * this.scale));
      const n = this.trail.length;
      this.trail.forEach(([tx, ty], i) => {
        const a = (i + 1) / (n + 1);
        draw.circle(ctx, ui.mix(COL.GREEN, COL.BALL, a * 0.35), this.project(tx, ty), Math.max(1, Math.trunc(r * 0.6)));
      });
      draw.circle(ctx, [18, 46, 28], [px + 2, py + 2], r);
      draw.circle(ctx, COL.BALL, [px, py], r);
      draw.circle(ctx, [206, 210, 200], [px, py], r, 1);
      if (r >= 5) draw.circle(ctx, [255, 255, 255], [px - Math.floor(r / 3), py - Math.floor(r / 3)], Math.max(1, Math.floor(r / 3)));
    }

    drawAim(ctx) {
      const [px, py] = this.project(this.bx, this.by);
      const ox = Math.cos(this.aim), oy = Math.sin(this.aim);
      const lock = this.powerLock;
      const col = lock ? COL.LOCK : COL.AIM;
      if (this.guide) {
        const steps = Math.max(6, Math.trunc(((30 + 60 * this.power) * this.scale) / 9));
        for (let i = 0; i < steps; i += 2) {
          draw.line(ctx, col, [px + ox * (i * 9 + 6), py + oy * (i * 9 + 6)], [px + ox * (i * 9 + 12), py + oy * (i * 9 + 12)], 2);
        }
      }
      // Stärke-Sperre: ruhig pulsender Ring um den Ball.
      if (lock) {
        const puls = 0.5 + 0.5 * Math.sin(this.lockT * 5.0);
        const rr = Math.trunc(Math.max(6.0, BR * this.scale) + 5 + puls * 4);
        draw.circle(ctx, COL.LOCK, [px, py], rr, 2);
      }
      // Schläger hinter dem Ball
      const bx1 = px - ox * (10 + this.power * 26);
      const by1 = py - oy * (10 + this.power * 26);
      draw.line(ctx, [228, 228, 222], [bx1, by1], [bx1 - ox * 26, by1 - oy * 26], 3);
      draw.line(ctx, lock ? COL.LOCK : [150, 154, 160], [bx1, by1], [bx1 - oy * 7, by1 + ox * 7], 5);
    }

    drawHud(ctx) {
      draw.rect(ctx, ui.PANEL, [0, 0, this.width, this.hudH]);
      draw.line(ctx, ui.BORDER, [0, this.hudH + 0.5], [this.width, this.hudH + 0.5]);
      const cy = this.hudH >> 1;
      const left = t("golf.hole", { n: this.holeIdx + 1, total: this.holes.length }) + "  ·  " + t("golf.par", { n: this.par });
      ui.text(ctx, left, 12, cy, this.fTiny, ui.TEXT_DIM, "midleft");
      let col = this.accent, mid;
      if (this.powerLock && this.phase === "aim") {
        // Die Sperre ist ein Dauerzustand - sie steht über der Kurzmeldung.
        mid = t("golf.lock", { n: this.powerPct() });
        col = COL.LOCK;
      } else if (this.msg) mid = this.msg;
      else mid = t("golf.strokes", { n: this.strokes });
      ui.text(ctx, mid, this.width >> 1, cy, this.fSmall, col, "center");
      this.drawPower(ctx, cy);
    }

    /** Schlagstärke in ganzen Prozent (Anzeige im HUD). */
    powerPct() {
      return Math.round(this.power * 100);
    }

    /**
     * Ladebalken rechts im HUD, mit Prozentzahl davor. Gesperrt wird er
     * golden, bekommt ein Schloss, eine Haltemarke und einen ruhigen Puls.
     */
    drawPower(ctx, cy) {
      const mw = 84, mh = 10;
      const mx = this.width - mw - 14;
      const top = cy - (mh >> 1);
      const lock = this.powerLock;
      const col = lock ? COL.LOCK : this.accent;
      draw.rect(ctx, ui.BTN, [mx, top, mw, mh], 0, 4);
      const fill = Math.trunc(mw * this.power);
      if (fill > 0) draw.rect(ctx, col, [mx, top, fill, mh], 0, 4);
      ui.text(ctx, this.powerPct() + "%", mx - (lock ? 21 : 6), cy, this.fTiny, lock ? col : ui.TEXT_DIM, "midright");
      if (!lock) return;
      MiniGolfGame.drawLockIcon(ctx, mx - 12, cy, col);
      const tick = mx + Math.max(1, Math.min(mw - 1, fill));
      draw.line(ctx, [255, 250, 232], [tick, top - 3], [tick, top + mh + 2], 2);
      const puls = 0.5 + 0.5 * Math.sin(this.lockT * 5.0);
      draw.rect(ctx, ui.mix(col, [255, 255, 255], 0.1 + 0.35 * puls), [mx - 2, top - 2, mw + 4, mh + 4], 1, 6);
    }

    /** Winziges Vorhängeschloss (11x15 px), um (x, cy) zentriert. */
    static drawLockIcon(ctx, x, cy, col) {
      draw.arc(ctx, col, [x - 3, cy - 8, 7, 8], 0.0, Math.PI, 2);
      draw.line(ctx, col, [x - 3, cy - 5], [x - 3, cy - 1]);
      draw.line(ctx, col, [x + 3, cy - 5], [x + 3, cy - 1]);
      draw.rect(ctx, col, [x - 5, cy - 1, 11, 8], 0, 2);
      draw.rect(ctx, ui.PANEL, [x - 1, cy + 2, 2, 3]);
    }

    /** Scorekarte rechts: Bahn, Par und Schläge. */
    drawCard(ctx) {
      const x = this.cardX, w = this.cardW;
      const y = this.hudH + 10;
      const f = this.fCard;
      const line = f.height + 2;
      const rect = new PG.Rect(x, y, w, (this.holes.length + 2) * line + 18);
      if (rect.bottom > this.height - 4) return;
      ui.drawPanel(ctx, rect, { radius: 8, shadow: false });
      let cy = y + 8;
      const parX = x + w - 62, meX = x + w - 38;
      ui.text(ctx, t("golf.card_hole"), x + 8, cy, f, ui.TEXT_DIM);
      ui.text(ctx, t("golf.card_par"), parX, cy, f, ui.TEXT_DIM);
      ui.text(ctx, "1", meX, cy, f, this.accent);
      cy += line + 2;
      this.holes.forEach((hole, i) => {
        const active = i === this.holeIdx;
        ui.text(ctx, String(i + 1), x + 8, cy, f, active ? ui.TEXT : ui.TEXT_DIM);
        ui.text(ctx, String(hole.par), parX, cy, f, ui.TEXT_FAINT);
        let v = this.card[i];
        if (active && this.state === PLAY) v = this.strokes;
        if (v) {
          const c = v < hole.par ? ui.GREEN : v > hole.par ? ui.RED : ui.TEXT;
          ui.text(ctx, String(v), meX, cy, f, c);
        }
        cy += line;
      });
      draw.line(ctx, ui.BORDER, [x + 6, cy + 1.5], [x + w - 6, cy + 1.5]);
      cy += 5;
      ui.text(ctx, t("golf.card_sum"), x + 8, cy, f, ui.TEXT_DIM);
      ui.text(ctx, String(sum(this.holes.map((h) => h.par))), parX, cy, f, ui.TEXT_FAINT);
      ui.text(ctx, String(sum(this.card)), meX, cy, f, this.accent);
    }

    banner(ctx, h = 104) {
      const y = (this.height >> 1) - (h >> 1);
      draw.rect(ctx, [8, 14, 10, 214], [0, y, this.width, h]);
      draw.line(ctx, this.accent, [0, y + 0.5], [this.width, y + 0.5]);
      draw.line(ctx, this.accent, [0, y + h - 0.5], [this.width, y + h - 0.5]);
      return y;
    }

    drawHoleDone(ctx) {
      const y = this.banner(ctx);
      const cx = this.width >> 1;
      ui.text(ctx, t(this.resultKey), cx, y + 34, this.fHuge, this.accent, "center");
      ui.text(ctx, t("golf.hole_result", { strokes: this.strokes, par: this.par, pts: this.resultPts }), cx, y + 66, this.fSmall, ui.TEXT, "center");
      ui.text(ctx, t("golf.next"), cx, y + 90, this.fTiny, ui.TEXT_DIM, "center");
    }

    drawOver(ctx) {
      const y = this.banner(ctx, this.overH);
      const cx = this.width >> 1;
      const total = sum(this.card);
      const parTotal = sum(this.holes.map((h) => h.par));
      ui.text(ctx, t("golf.round_done"), cx, y + this.overY.head, this.fHuge, this.accent, "center");
      const d = total - parTotal;
      const diff = d ? (d > 0 ? "+" + d : String(d)) : t("golf.even");
      ui.text(ctx, t("golf.final", { strokes: total, diff, pts: this.points }), cx, y + this.overY.sub, this.fSmall, ui.TEXT, "center");
      const best = this.best[this.bestKey()];
      if (best) ui.text(ctx, t("golf.best", { n: best }), cx, y + this.overY.best, this.fTiny, ui.GOLD, "center");
      // Knopfreihe: Weiter ist der hervorgehobene Standardweg.
      const labels = { next: t("golf.btn_next", { course: this.nextCourseLabel() }), again: t("golf.btn_again"), setup: t("golf.btn_setup") };
      for (const [key, rc] of this.overRects) this.btn(ctx, rc, labels[key], key === this.overRects[0][0]);
      // Tastenzeile passend zu den vorhandenen Knöpfen.
      let hint = t(this.nextCourse() ? "golf.continue_hint" : "golf.new_round");
      if (this.replay && !this.rep) hint += "  ·  " + t("golf.replay_hint");
      ui.text(ctx, hint, cx, y + this.overY.hint, this.fTiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      const cx = this.width >> 1;
      ui.text(ctx, t("golf.title"), cx, Math.trunc(this.height * 0.115), this.fHuge, this.accent, "center");
      const subKey = this.setupTab === "play" ? "golf.subtitle" : "golf.ugc.subtitle";
      ui.text(ctx, t(subKey), cx, Math.trunc(this.height * 0.18), this.fSmall, ui.TEXT_DIM, "center");
      this.tabRects.forEach((rc, i) => this.btn(ctx, rc, t("golf.tab_" + TABS[i]), this.setupTab === TABS[i]));
      if (this.setupTab === "maps") {
        if (!this.maps) this.maps = new edit.MapList(this);
        this.maps.draw(ctx);
        return;
      }

      const label = (rects, txt, w) => {
        // Beschriftung mittig über die Gruppe (zu lange wird gekürzt).
        if (w) txt = fit(this.fTiny, txt, w);
        const mid = (rects[0].left + rects[rects.length - 1].right) >> 1;
        ui.text(ctx, txt, mid, rects[0].top - 4, this.fTiny, ui.TEXT_DIM, "midbottom");
      };

      label(this.courseRects, t("golf.lbl_course"));
      this.courseRects.forEach((rc, i) => this.btn(ctx, rc, t("golf.course." + COURSES[i]), this.course === COURSES[i]));
      this.drawTourRow(ctx);
      // Schalterzeile: Ziellinie · Autoziel · Aufnehmen
      for (const [rects, key, on] of [[this.guideRects, "golf.lbl_guide", this.guide],
        [this.autoaimRects, "golf.lbl_autoaim", this.autoaim], [this.pickupRects, "golf.lbl_pickup", this.pickup]]) {
        label(rects, t(key), this.optW);
        rects.forEach((rc, i) => this.btn(ctx, rc, i === 0 ? t("common.on") : t("common.off"), on === (i === 0)));
      }
      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 9);
      draw.rect(ctx, this.accent, this.startRect, 2, 9);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");
      const best = this.best[this.bestKey()];
      if (best) ui.text(ctx, t("golf.best", { n: best }), cx, this.startRect.bottom + 14, this.fTiny, ui.GOLD, "center");
      ui.text(ctx, t("golf.setup_hint"), cx, this.height - 12, this.fTiny, ui.TEXT_DIM, "center");
    }

    /** Kurswahl der Tour: Pfeile, Kursnummer und Gesamt-Par. */
    drawTourRow(ctx) {
      const [left, mid, right] = this.tourRects;
      const on = this.course === "tour";
      ui.text(ctx, t("golf.lbl_tour"), this.width >> 1, left.top - 3, this.fTiny, on ? ui.TEXT_DIM : ui.TEXT_FAINT, "midbottom");
      const col = on ? this.accent : ui.BORDER;
      for (const [rc, arrow] of [[left, "<"], [right, ">"]]) {
        draw.rect(ctx, ui.BTN, rc, 0, 7);
        draw.rect(ctx, col, rc, 1, 7);
        ui.text(ctx, arrow, rc.centerx, rc.centery, this.fSmall, on ? ui.TEXT : ui.TEXT_FAINT, "center");
      }
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, mid, 0, 7);
      draw.rect(ctx, col, mid, on ? 2 : 1, 7);
      let txt = t("golf.tour", { n: this.tour, total: gen.TOUR_COURSES });
      if (on) txt += "  ·  " + t("golf.par", { n: this.tourPar });
      ui.text(ctx, txt, mid.centerx, mid.centery, this.fSmall, on ? ui.TEXT : ui.TEXT_FAINT, "center");
    }

    btn(ctx, rc, text, on) {
      draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 8);
      draw.rect(ctx, on ? this.accent : ui.BORDER, rc, on ? 2 : 1, 8);
      const col = on ? ui.TEXT : ui.TEXT_DIM;
      let f = this.fSmall;
      if (f.width(text) > rc.w - 12) f = this.fTiny; // z.B. "Weiter: Tour 12"
      ui.text(ctx, fit(f, text, rc.w - 8), rc.centerx, rc.centery, f, col, "center");
    }
  }

  PG.register(MiniGolfGame, {
    id: "MiniGolfGame",
    key: "minigolf",
    name: "Minigolf",
    settingsKey: "minigolf",
    defaults: { course: "classic", tour: 1, guide: true, autoaim: true, pickup: true, ugc_map: "", grid: true },
    wantsRightClick: true,
  });
})();
