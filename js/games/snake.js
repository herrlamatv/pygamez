/*
 * snake.js - Snake Deluxe mit Spielmodi, Boost und 3D-Ansicht (Port von games/snake.py)
 * ======================================================================================
 * - 3D-ANSICHT (Taste V im Setup): Software-3D (Perspektivprojektion +
 *   Painter-Algorithmus) mit Verfolgerkamera hinter dem Kopf; gelenkt wird
 *   relativ zur Blickrichtung. Distanz-Nebel, Sternenhimmel, Schachbrett-Boden,
 *   Banden, rotierende Futter-Kristalle, 3D-Partikel und Kamera-Shake.
 *   In 3D wählbar: Klassisch und Hindernisse; die Wände sind dort immer fest.
 *   Nach dem Game Over umkreist die Kamera langsam die Schlange.
 * - BOOST: Leertaste/Shift (oder Enter) gedrückt halten = doppeltes Tempo,
 *   verbraucht Ausdauer; Goldäpfel füllen sie sofort ganz auf.
 * - SPIELMODI (im Setup): Klassisch, Speed-Rush, Hindernisse, Portale,
 *   Zeitangriff (60 s) und Competitive (Level-Aufstieg, Slot-Machine,
 *   lila Wett-Äpfel, optional HARDCORE).
 * - Äpfel auf der Map (Taste F): 1/2/3/5 gleichzeitig liegende Äpfel.
 * - Wände-durchgehen, Bonus-Äpfel, Prestige (Taste P) - siehe PRESTIGE unten.
 * - Personalisieren (Pinsel / Taste C): Kopffarbe, Raster-Wegweiser, Banner
 *   (Nachbau von ngb.py, gespeichert im Browser).
 * - Web-Version: nur Einzelspieler (die 2-Schlangen-Modi entfallen).
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  const CELL = 20; // Kantenlänge einer Rasterzelle in Pixeln
  const BASE_INTERVAL = 0.12; // Sekunden pro Schritt (Normaltempo)
  const MIN_INTERVAL = 0.055; // schnellstes Tempo (Speed-Rush)
  const MIN_LENGTH = 3; // so kurz darf eine Schlange durch Prestige max. werden

  // Boost / Ausdauer
  const STAMINA_MAX = 1.0;
  const STAMINA_REGEN = 0.22; // Aufladung pro Sekunde (wenn nicht geboostet)
  const BOOST_DRAIN = 0.045; // Verbrauch pro zusätzlichem Boost-Schritt
  const BOOST_MIN_START = 0.15; // so viel Ausdauer braucht man mindestens zum Starten
  // HARDCORE (nur Competitive): jeder Boost-Schritt frisst so viele Längen-Blöcke.
  const HARDCORE_BOOST_LEN_COST = 1;

  // Boost-Tasten (Einzelspieler: beide Belegungen)
  const BOOST_KEYS = ["space", "Shift_L", "Return", "Shift_R", "KP_Enter"];

  const GOLDEN_LIFETIME = 6.0; // Sekunden, die ein Goldapfel liegen bleibt
  const GOLDEN_CHANCE = 0.2; // Chance, nach einem normalen Apfel einen Goldapfel zu setzen
  const TIMED_SECONDS = 60.0;

  // ----- Competitive-Modus ---------------------------------------------------
  const SPECIAL_LIFETIME = 8.0; // Sekunden, die ein blauer/lila Apfel liegen bleibt
  const BLUE_CHANCE = 0.12; // Chance je Apfel, einen blauen Apfel nachzulegen
  const PURPLE_CHANCE = 0.16; // Chance je Apfel, einen lila Apfel nachzulegen
  const SPAWN_BONUS_TIME = 10.0; // so lange legt der Slot-Bonus zusätzliche Äpfel nach
  const SLOT_REEL_STOPS = [0.9, 1.35, 1.8];
  const SLOT_SHOW = 1.7;
  const SLOT_SPIN_SPEED = 16.0; // Symbole pro Sekunde beim Drehen

  const BANNER_TIME = 2.0; // Einblendung oben mittig (z.B. lila Multiplikator)

  // Wählbare Anzahl gleichzeitig auf dem Feld liegender Äpfel.
  const APPLE_COUNTS = [1, 2, 3, 5];

  // Apfel-Animationen (nur 2D-Ansicht)
  const FOOD_SPAWN_ANIM = 0.28;
  const FOOD_EAT_ANIM = 0.3;

  /** Weiche Einblendung mit leichtem Überschwingen (0 -> ~1.1 -> 1). */
  function easeOutBack(p) {
    if (p >= 1) return 1;
    const c1 = 1.70158, c3 = c1 + 1;
    return 1 + c3 * Math.pow(p - 1, 3) + c1 * Math.pow(p - 1, 2);
  }

  // Spielzustände
  const SETUP = "setup", PLAY = "play", PERSONALIZE = "personalize", CAM3D = "cam3d";

  // Spielfeld-/Gameplay-Farben (bewusst fest, unabhängig vom UI-Theme).
  const COL_BG = [15, 15, 25];
  const COL_GRID = [25, 25, 40];
  const COL_FOOD = [240, 90, 90];
  const COL_GOLD = [255, 205, 70];
  const COL_WALL = [70, 78, 98];
  const COL_BLUE = [90, 150, 245];
  const COL_PURPLE = [185, 110, 240];
  const COL_HARDCORE = [235, 45, 55];

  // Farben der Schlange: [Körper, Kopf]
  const SNAKE_COLORS = [[[80, 220, 120], [150, 255, 180]]];
  const BOOST_GLOW = [[150, 255, 190]];

  const PORTAL_COLORS = [[255, 140, 60], [180, 120, 255], [90, 220, 220]];

  // Spielmodi (Texte: snake.mode.<key> / snake.mode.<key>.desc)
  const MODE_KEYS = ["classic", "speed", "walls", "portal", "timed", "competitive"];

  // ----- 3D-Ansicht ------------------------------------------------------------
  // Weltkoordinaten: x = Spalten, z = Zeilen, y = Höhe (Boden bei y = 0).
  const MODES_3D = ["classic", "walls"];
  const NEAR = 0.12;
  const FOG_START = 7.0, FOG_END = 24.0;
  const FOV_NORMAL = 1.12;
  const FOV_BOOST = 0.94;
  const CAM_BACK = 3.6;
  const CAM_H = 2.6;
  const CAM_AHEAD = 2.6;
  const CAM_LOOK_H = 0.35;
  const CAM_SMOOTH = 6.0;
  const FOV_BOOST_DELTA = FOV_NORMAL - FOV_BOOST;
  const CAM_FOV_MIN = 0.8, CAM_FOV_MAX = 1.5, CAM_FOV_STEP = 0.02;
  const CAM_H_MIN = 1.6, CAM_H_MAX = 4.2, CAM_H_STEP = 0.1;

  const COL_SKY_TOP = [8, 10, 22];
  const COL_SKY_HOR = [46, 52, 86];
  const COL_FOG = [34, 39, 66];
  const COL_TILE_A = [26, 29, 48];
  const COL_TILE_B = [32, 36, 58];
  const COL_BORDER = [84, 96, 138];

  const rgb = (c) => "rgb(" + (c[0] | 0) + "," + (c[1] | 0) + "," + (c[2] | 0) + ")";
  const key = (x, y) => x + "," + y;
  const round2 = (v) => Math.round(v * 100) / 100;

  /** Dreht einen Gitter-Richtungsvektor um 90 Grad ("L" oder "R"). */
  function rotateDir(dir, turn) {
    const [dx, dy] = dir;
    return turn === "L" ? [dy, -dx] : [-dy, dx];
  }

  // =================================================================== Competitive
  // (Nachbau von competitive.py; Level-Tabelle aus games/levels/snake-comp.json)
  const COMP = (function () {
    const BASE_APPLES = 1, BASE_MULT = 1;
    const ROWS = [ // [threshold, apples, multiplier]
      [4, 2, 2], [10, 3, 3], [18, 4, 4], [30, 5, 5], [46, 6, 6],
      [68, 7, 7], [96, 8, 8], [132, 9, 9], [176, 10, 10], [230, 11, 11],
      [296, 12, 12], [376, 13, 13], [472, 14, 14], [586, 15, 15], [720, 16, 16],
    ];
    const LEVEL_STEPS = ROWS.map((r) => r[0]);
    const APPLES_TAB = [BASE_APPLES].concat(ROWS.map((r) => r[1]));
    const MULT_TAB = [BASE_MULT].concat(ROWS.map((r) => r[2]));
    const MAX_LEVEL = LEVEL_STEPS.length;
    const MAX_APPLES = Math.max(...APPLES_TAB);

    function levelForApples(total) {
      let lvl = 0;
      for (const need of LEVEL_STEPS) {
        if (total >= need) lvl++;
        else break;
      }
      return lvl;
    }
    const applesOnField = (level) => APPLES_TAB[PG.clamp(level, 0, MAX_LEVEL)];
    const scoreMultiplier = (level) => MULT_TAB[PG.clamp(level, 0, MAX_LEVEL)];
    /** [erreicht, benötigt] Äpfel innerhalb der Stufe - oder null (Maximalstufe). */
    function nextStep(total) {
      const lvl = levelForApples(total);
      if (lvl >= MAX_LEVEL) return null;
      const prev = lvl > 0 ? LEVEL_STEPS[lvl - 1] : 0;
      return [total - prev, LEVEL_STEPS[lvl] - prev];
    }

    // Slot-Machine (blau)
    const SLOT_SYMBOLS = {
      seven: [255, 210, 90], gem: [110, 220, 235], bell: [255, 190, 70],
      apple: [240, 90, 90], cherry: [235, 110, 160],
    };
    const REEL = [].concat(
      new Array(5).fill("cherry"), new Array(4).fill("apple"), new Array(3).fill("bell"),
      new Array(2).fill("gem"), ["seven"]
    );
    const TRIPLE = { seven: 6.0, gem: 4.0, bell: 3.0, apple: 2.5, cherry: 2.0 };
    const PAIR_MULT = 1.5, MISS_MULT = 0.5;
    const spinReels = () => [0, 1, 2].map(() => PG.rand.choice(REEL));
    function slotOutcome([a, b, c]) {
      if (a === b && b === c) return [TRIPLE[a], "jackpot"];
      if (a === b || b === c || a === c) return [PAIR_MULT, "pair"];
      return [MISS_MULT, "miss"];
    }

    // Lila Apfel (Gambling)
    const purpleFactor = (hc) => round2(hc ? PG.rand.uniform(0.25, 2.25) : PG.rand.uniform(0.5, 1.5));
    const purpleStake = (hc) => (hc ? round2(PG.rand.uniform(0.75, 0.9)) : 0.5);

    return { MAX_APPLES, levelForApples, applesOnField, scoreMultiplier, nextStep, SLOT_SYMBOLS, REEL, spinReels, slotOutcome, purpleFactor, purpleStake };
  })();

  // ====================================================================== Prestige
  // (Nachbau von prestige.py)
  const PRESTIGE = (function () {
    const LEVELS = [
      { roman: "I", apples: 20, length: 10 },
      { roman: "II", apples: 50, length: 25 },
      { roman: "III", apples: 200, length: 63 },
      { roman: "IV", apples: 500, length: 158 },
      { roman: "V", apples: 1200, length: 395 },
      { roman: "VI", apples: 3000, length: 988 },
      { roman: "VII", apples: 7500, length: 2470 },
      { roman: "VIII", apples: 18000, length: 6175 },
      { roman: "IX", apples: 45000, length: 15438 },
      { roman: "X", apples: 100000, length: 38594 },
    ];
    const MAX = LEVELS.length;
    return {
      blocksPerApple: (lvl) => 1 + Math.max(0, lvl | 0),
      scoreMultiplier: (lvl) => Math.pow(2, Math.max(0, lvl | 0)),
      roman: (lvl) => (lvl <= 0 ? "" : LEVELS[Math.min(lvl, MAX) - 1].roman),
      nextRequirement: (lvl) => (lvl >= MAX ? null : LEVELS[lvl]),
    };
  })();

  // =========================================================================== NGB
  // Visuelle Personalisierung (Nachbau von ngb.py) - gespeichert unter "ngb".
  const NGB = (function () {
    const HEAD_PRESETS = [
      { id: "aqua1", color: [90, 165, 245] },
      { id: "aqua2", color: [72, 195, 240] },
      { id: "aqua3", color: [58, 218, 224] },
      { id: "aqua4", color: [52, 234, 198] },
      { id: "red", color: [255, 95, 95] },
      { id: "orange", color: [255, 160, 60] },
      { id: "custom", color: null },
    ];
    const HEAD_IDS = HEAD_PRESETS.map((p) => p.id);
    const DEFAULT_HEAD_ID = "aqua3";
    const DEFAULT_CUSTOM = [200, 120, 255];
    const GRID_PRESETS = [
      { id: "off", seq: null },
      { id: "grid1", seq: [[44, 66, 108], [26, 40, 70]] },
      { id: "grid2", seq: [[36, 74, 58], [24, 50, 42]] },
      { id: "grid3", seq: [[70, 46, 98], [44, 32, 66]] },
      { id: "grid4", seq: [[92, 68, 34], [60, 46, 26]] },
      { id: "grid5", seq: [[58, 62, 74], [30, 33, 42], [44, 47, 58]] },
      { id: "custom", seq: null },
    ];
    const GRID_IDS = GRID_PRESETS.map((p) => p.id);
    const DEFAULT_GRID_ID = "off";
    const DEFAULT_GRID_CUSTOM = [[46, 64, 96], [28, 40, 66]];
    const BANNER_SIZE_MIN = 0.6, BANNER_SIZE_MAX = 1.6;
    const BANNER_OP_MIN = 0.2, BANNER_OP_MAX = 1.0;

    let state = null;
    const clampColor = (c, fb) => {
      if (!Array.isArray(c) || c.length < 3) return fb.slice();
      const out = c.slice(0, 3).map((v) => PG.clamp(Math.trunc(Number(v)), 0, 255));
      return out.some((v) => isNaN(v)) ? fb.slice() : out;
    };
    const num = (v, lo, hi, fb) => {
      const n = Number(v);
      return v == null || isNaN(n) ? fb : PG.clamp(n, lo, hi);
    };

    function load() {
      if (state) return state;
      let d = PG.store.get("ngb", {});
      if (!d || typeof d !== "object") d = {};
      const gc = Array.isArray(d.grid_custom) && d.grid_custom.length === 2 ? d.grid_custom : DEFAULT_GRID_CUSTOM;
      state = {
        head_id: HEAD_IDS.includes(d.head_id) ? d.head_id : DEFAULT_HEAD_ID,
        head_custom: clampColor(d.head_custom, DEFAULT_CUSTOM),
        grid_id: GRID_IDS.includes(d.grid_id) ? d.grid_id : DEFAULT_GRID_ID,
        grid_custom: [clampColor(gc[0], DEFAULT_GRID_CUSTOM[0]), clampColor(gc[1], DEFAULT_GRID_CUSTOM[1])],
        banner_on: d.banner_on == null ? true : !!d.banner_on,
        banner_size: num(d.banner_size, BANNER_SIZE_MIN, BANNER_SIZE_MAX, 1.0),
        banner_opacity: num(d.banner_opacity, BANNER_OP_MIN, BANNER_OP_MAX, 1.0),
      };
      return state;
    }
    function save() {
      const st = load();
      PG.store.set("ngb", {
        head_id: st.head_id, head_custom: st.head_custom.slice(), grid_id: st.grid_id,
        grid_custom: [st.grid_custom[0].slice(), st.grid_custom[1].slice()], banner_on: st.banner_on,
        banner_size: Math.round(st.banner_size * 1000) / 1000, banner_opacity: Math.round(st.banner_opacity * 1000) / 1000,
      });
    }
    const headPreset = (pid) => HEAD_PRESETS.find((p) => p.id === pid) || HEAD_PRESETS[0];
    const gridPreset = (pid) => GRID_PRESETS.find((p) => p.id === pid) || GRID_PRESETS[0];

    const api = {
      HEAD_PRESETS, GRID_PRESETS, BANNER_SIZE_MIN, BANNER_SIZE_MAX, BANNER_OP_MIN, BANNER_OP_MAX,
      headPreset, gridPreset,
      getHeadId: () => load().head_id,
      setHeadId(pid) { if (HEAD_IDS.includes(pid)) { load().head_id = pid; save(); } },
      getCustom: () => load().head_custom.slice(),
      setCustom(c) { load().head_custom = clampColor(c, DEFAULT_CUSTOM); save(); },
      /** Aktive Kopffarbe der Schlange - rein visuell. */
      headColor() {
        const st = load();
        return st.head_id === "custom" ? st.head_custom : headPreset(st.head_id).color;
      },
      getGridId: () => load().grid_id,
      setGridId(pid) { if (GRID_IDS.includes(pid)) { load().grid_id = pid; save(); } },
      getGridCustom: () => load().grid_custom.map((c) => c.slice()),
      setGridCustom(cols) {
        load().grid_custom = [clampColor(cols[0], DEFAULT_GRID_CUSTOM[0]), clampColor(cols[1], DEFAULT_GRID_CUSTOM[1])];
        save();
      },
      /** Aktive Farbreihenfolge fürs Raster-Overlay - oder null (aus). */
      gridSequence() {
        const st = load();
        if (st.grid_id === "off") return null;
        if (st.grid_id === "custom") return st.grid_custom;
        return gridPreset(st.grid_id).seq;
      },
      getBanner() {
        const st = load();
        return { on: st.banner_on, size: st.banner_size, opacity: st.banner_opacity };
      },
      setBannerOn(v) { load().banner_on = !!v; save(); },
      setBannerSize(v) { load().banner_size = PG.clamp(v, BANNER_SIZE_MIN, BANNER_SIZE_MAX); save(); },
      setBannerOpacity(v) { load().banner_opacity = PG.clamp(v, BANNER_OP_MIN, BANNER_OP_MAX); save(); },
    };
    return api;
  })();

  // Menü-Farben (lokal wie in ngb.py)
  const M_BG = [14, 15, 24];
  const M_TEXT = [232, 232, 238];
  const M_DIM = [150, 158, 176];
  const M_ACCENT = [90, 160, 240];
  const M_BTN = [44, 50, 66];
  const M_BTN_ON = [60, 120, 80];
  const M_SEL = [255, 255, 255];
  const M_TRACK = [36, 40, 54];
  const M_TAB_OFF = [34, 38, 52];

  /**
   * Personalisierungs-Menü mit drei Tabs (nur Optik): Kopf, Raster, Banner.
   * Änderungen werden sofort gespeichert; done wird true beim Schließen.
   */
  class PersonalizeMenu {
    constructor(width, height, playSound) {
      this.width = width;
      this.height = height;
      this.done = false;
      this.play = playSound || (() => {});
      this.big = ui.font(32, true, true);
      this.fnt = ui.font(20, false, true);
      this.small = ui.font(15, false, true);
      this.tiny = ui.font(12, false, true);
      this.tab = "head";
      this.sel = NGB.getHeadId();
      this.custom = NGB.getCustom();
      this.gridSel = NGB.getGridId();
      this.gridCustom = NGB.getGridCustom();
      this.gridEdit = 0; // 0 = Farbe A, 1 = Farbe B
      this.buildLayout();
    }

    // ----- Layout ------------------------------------------------------------
    buildLayout() {
      const cx = Math.floor(this.width / 2);
      const tabw = 120, tgap = 8;
      const names = ["head", "grid", "banner"];
      let total = names.length * tabw + (names.length - 1) * tgap;
      let x = cx - Math.floor(total / 2);
      this.tabRects = {};
      for (const nm of names) {
        this.tabRects[nm] = new PG.Rect(x, 46, tabw, 30);
        x += tabw + tgap;
      }

      // Kopf-Seite: 7 Kacheln in 2 Reihen
      const tw = 92, th = 46, gap = 12;
      this.headTiles = [];
      let y = 116;
      for (const row of [NGB.HEAD_PRESETS.slice(0, 4), NGB.HEAD_PRESETS.slice(4)]) {
        total = row.length * tw + (row.length - 1) * gap;
        x = cx - Math.floor(total / 2);
        for (const p of row) {
          this.headTiles.push([new PG.Rect(x, y, tw, th), p]);
          x += tw + gap;
        }
        y += th + 18 + gap;
      }
      this.slidersHead = this.makeSliders(y + 4);

      // Raster-Seite: 7 Kacheln in einer Reihe
      const gtw = 74, gth = 46, ggap = 8;
      this.gridTiles = [];
      total = NGB.GRID_PRESETS.length * gtw + (NGB.GRID_PRESETS.length - 1) * ggap;
      x = cx - Math.floor(total / 2);
      const gy = 116;
      for (const p of NGB.GRID_PRESETS) {
        this.gridTiles.push([new PG.Rect(x, gy, gtw, gth), p]);
        x += gtw + ggap;
      }
      const aby = gy + gth + 22 + 12;
      this.gridAb = { A: new PG.Rect(cx - 104, aby, 92, 26), B: new PG.Rect(cx + 12, aby, 92, 26) };
      this.slidersGrid = this.makeSliders(aby + 42);

      // Banner-Seite: An/Aus + Größe + Deckkraft
      this.bannerToggle = new PG.Rect(cx - 150, 118, 300, 36);
      this.bannerSlider = { size: this.oneSlider(200), opacity: this.oneSlider(258) };

      this.doneRect = new PG.Rect(cx - 80, this.height - 78, 160, 32);
    }

    oneSlider(top) {
      const cx = Math.floor(this.width / 2);
      const track = new PG.Rect(cx - 84, top, 168, 14);
      return { track, minus: new PG.Rect(track.left - 30, top - 4, 22, 22), plus: new PG.Rect(track.right + 8, top - 4, 22, 22) };
    }

    makeSliders(top) {
      const cx = Math.floor(this.width / 2);
      const d = {};
      let sy = top;
      for (const ch of ["R", "G", "B"]) {
        const track = new PG.Rect(cx - 100, sy, 200, 14);
        d[ch] = { track, minus: new PG.Rect(track.left - 30, sy - 4, 22, 22), plus: new PG.Rect(track.right + 8, sy - 4, 22, 22) };
        sy += 30;
      }
      return d;
    }

    // ----- Zustand -------------------------------------------------------------
    activeSliders() {
      return this.tab === "head" ? this.slidersHead : this.slidersGrid;
    }
    /** Die aktuell per RGB editierbare Farbe (3er-Array) - oder null. */
    editable() {
      if (this.tab === "head" && this.sel === "custom") return this.custom;
      if (this.tab === "grid" && this.gridSel === "custom") return this.gridCustom[this.gridEdit];
      return null;
    }
    persist() {
      if (this.tab === "head") NGB.setCustom(this.custom);
      else NGB.setGridCustom(this.gridCustom);
    }
    headCol() {
      return this.sel === "custom" ? this.custom : NGB.headPreset(this.sel).color;
    }
    selectHead(pid) {
      this.sel = pid;
      NGB.setHeadId(pid);
      this.play("select");
    }
    selectGrid(pid) {
      this.gridSel = pid;
      NGB.setGridId(pid);
      this.play("select");
    }
    sliderSet(ed, ch, value) {
      ed["RGB".indexOf(ch)] = PG.clamp(Math.trunc(value), 0, 255);
      this.persist();
    }
    step(ed, ch, delta) {
      this.sliderSet(ed, ch, ed["RGB".indexOf(ch)] + delta);
      this.play("click");
    }
    close() {
      this.done = true;
      this.play("click");
    }

    // ----- Banner-Steuerung --------------------------------------------------------
    static rangeFromX(track, x, lo, hi) {
      const frac = PG.clamp((x - track.left) / Math.max(1, track.width), 0, 1);
      return lo + frac * (hi - lo);
    }
    static bannerRange(kind) {
      return kind === "size" ? [NGB.BANNER_SIZE_MIN, NGB.BANNER_SIZE_MAX] : [NGB.BANNER_OP_MIN, NGB.BANNER_OP_MAX];
    }
    setBanner(kind, value) {
      if (kind === "size") NGB.setBannerSize(value);
      else NGB.setBannerOpacity(value);
    }
    onClickBanner(p) {
      if (this.bannerToggle.collidepoint(p)) {
        NGB.setBannerOn(!NGB.getBanner().on);
        this.play("select");
        return true;
      }
      const cfg = NGB.getBanner();
      for (const kind of ["size", "opacity"]) {
        const sl = this.bannerSlider[kind];
        const [lo, hi] = PersonalizeMenu.bannerRange(kind);
        if (sl.track.collidepoint(p)) {
          this.setBanner(kind, PersonalizeMenu.rangeFromX(sl.track, p[0], lo, hi));
          this.play("click");
          return true;
        }
        if (sl.minus.collidepoint(p)) {
          this.setBanner(kind, cfg[kind] - 0.1);
          this.play("click");
          return true;
        }
        if (sl.plus.collidepoint(p)) {
          this.setBanner(kind, cfg[kind] + 0.1);
          this.play("click");
          return true;
        }
      }
      return false;
    }

    // ----- Eingabe ---------------------------------------------------------------
    handleEvent(ev) {
      if (ev.kind === "keydown") {
        if (ev.key === "Escape" || ev.key === "Return" || ev.key === "space") this.close();
        else if (ev.key === "Tab") {
          this.tab = this.tab === "head" ? "grid" : "head";
          this.play("click");
        } else if ("1234567".includes(ev.key) && ev.key.length === 1) {
          const idx = Number(ev.key) - 1;
          if (this.tab === "head" && idx < NGB.HEAD_PRESETS.length) this.selectHead(NGB.HEAD_PRESETS[idx].id);
          else if (this.tab === "grid" && idx < NGB.GRID_PRESETS.length) this.selectGrid(NGB.GRID_PRESETS[idx].id);
        }
        return;
      }
      if (ev.kind === "mousedown" && ev.pos) this.onClick(ev.pos);
    }

    onClick(p) {
      for (const name in this.tabRects) {
        if (this.tabRects[name].collidepoint(p)) {
          this.tab = name;
          this.play("click");
          return;
        }
      }
      if (this.tab === "head") {
        for (const [rect, pr] of this.headTiles) {
          if (rect.collidepoint(p)) return this.selectHead(pr.id);
        }
      } else if (this.tab === "grid") {
        for (const [rect, pr] of this.gridTiles) {
          if (rect.collidepoint(p)) return this.selectGrid(pr.id);
        }
        if (this.gridSel === "custom") {
          for (const k of ["A", "B"]) {
            if (this.gridAb[k].collidepoint(p)) {
              this.gridEdit = k === "A" ? 0 : 1;
              this.play("click");
              return;
            }
          }
        }
      } else if (this.tab === "banner") {
        if (this.onClickBanner(p)) return;
      }
      const ed = this.editable();
      if (ed) {
        const sliders = this.activeSliders();
        for (const ch of ["R", "G", "B"]) {
          const sl = sliders[ch];
          if (sl.track.collidepoint(p)) {
            this.sliderSet(ed, ch, Math.round(((p[0] - sl.track.left) / Math.max(1, sl.track.width)) * 255));
            this.play("click");
            return;
          }
          if (sl.minus.collidepoint(p)) return this.step(ed, ch, -8);
          if (sl.plus.collidepoint(p)) return this.step(ed, ch, +8);
        }
      }
      if (this.doneRect.collidepoint(p)) this.close();
    }

    // ----- Zeichnen ----------------------------------------------------------------
    draw(ctx) {
      draw.rect(ctx, M_BG, [0, 0, this.width, this.height]);
      const cx = Math.floor(this.width / 2);
      ui.text(ctx, t("ngb.title"), cx, 12, this.big, M_TEXT, "midtop");
      this.drawHeadPreview(ctx, this.width - 44, 30, this.headCol());
      this.drawTabs(ctx);

      if (this.tab === "head") this.drawHeadPage(ctx);
      else if (this.tab === "grid") this.drawGridPage(ctx);
      else this.drawBannerPage(ctx);

      draw.rect(ctx, M_BTN_ON, this.doneRect, 0, 10);
      ui.text(ctx, t("ngb.done"), this.doneRect.centerx, this.doneRect.centery, this.fnt, M_TEXT, "center");
      ui.text(ctx, t("ngb.subtitle"), cx, this.height - 34, this.tiny, M_DIM, "midbottom");
      ui.text(ctx, t("ngb.hint"), cx, this.height - 16, this.tiny, M_DIM, "midbottom");
    }

    drawTabs(ctx) {
      const labels = { head: t("ngb.tab_head"), grid: t("ngb.tab_grid"), banner: t("ngb.tab_banner") };
      for (const name in this.tabRects) {
        const r = this.tabRects[name];
        const active = name === this.tab;
        draw.rect(ctx, active ? M_ACCENT : M_TAB_OFF, r, 0, 8);
        draw.rect(ctx, active ? M_SEL : M_DIM, r, 1, 8);
        ui.text(ctx, labels[name], r.centerx, r.centery, this.small, active ? M_BG : M_DIM, "center");
      }
    }

    drawHeadPage(ctx) {
      const first = this.headTiles[0][0];
      ui.text(ctx, t("ngb.head_label"), first.left, first.top - 22, this.small, M_ACCENT);
      for (const [rect, pr] of this.headTiles) {
        const col = pr.id === "custom" ? this.custom : pr.color;
        this.drawTile(ctx, rect, col, pr.id === this.sel, t("ngb.head." + pr.id));
      }
      if (this.sel === "custom") this.drawSliders(ctx);
      else {
        const last = this.headTiles[this.headTiles.length - 1][0];
        ui.text(ctx, t("ngb.custom_pick"), Math.floor(this.width / 2), last.bottom + 30, this.small, M_DIM, "midtop");
      }
    }

    drawGridPage(ctx) {
      const first = this.gridTiles[0][0];
      ui.text(ctx, t("ngb.grid_label"), first.left, first.top - 22, this.small, M_ACCENT);
      for (const [rect, pr] of this.gridTiles) {
        const seq = pr.id === "custom" ? this.gridCustom : pr.seq;
        const selected = pr.id === this.gridSel;
        this.drawGridSwatch(ctx, rect, seq);
        if (selected) draw.rect(ctx, M_SEL, rect.inflate(8, 8), 2, 10);
        ui.text(ctx, t("ngb.grid." + pr.id), rect.centerx, rect.bottom + 4, this.tiny, selected ? M_TEXT : M_DIM, "midtop");
      }
      if (this.gridSel === "custom") {
        for (const k of ["A", "B"]) {
          const r = this.gridAb[k];
          const idx = k === "A" ? 0 : 1;
          draw.rect(ctx, this.gridCustom[idx], r, 0, 7);
          if (this.gridEdit === idx) draw.rect(ctx, M_SEL, r, 2, 7);
          else draw.rect(ctx, M_DIM, r, 1, 7);
          ui.text(ctx, k, r.centerx, r.centery, this.small, M_TEXT, "center");
        }
        this.drawSliders(ctx);
      } else {
        ui.text(ctx, t("ngb.grid_pick"), Math.floor(this.width / 2), first.bottom + 34, this.small, M_DIM, "midtop");
      }
    }

    drawBannerPage(ctx) {
      const cx = Math.floor(this.width / 2);
      const cfg = NGB.getBanner();
      const on = cfg.on;
      const bt = this.bannerToggle;
      ui.text(ctx, t("ngb.banner_label"), bt.left, bt.top - 22, this.small, M_ACCENT);
      draw.rect(ctx, on ? M_BTN_ON : M_BTN, bt, 0, 8);
      draw.rect(ctx, on ? M_SEL : M_DIM, bt, 1, 8);
      ui.text(ctx, t("ngb.banner_toggle"), bt.x + 16, bt.centery, this.fnt, M_TEXT, "midleft");
      const stat = t(on ? "common.on" : "common.off");
      ui.text(ctx, "< " + stat + " >", bt.right - 16, bt.centery, this.fnt, on ? [150, 235, 150] : M_DIM, "midright");

      this.drawBannerSlider(ctx, "size", cfg.size, on);
      this.drawBannerSlider(ctx, "opacity", cfg.opacity, on);

      if (on) this.drawBannerPreview(ctx, cx, 344, cfg);
      else ui.text(ctx, t("ngb.banner_off_hint"), cx, 330, this.small, M_DIM, "center");
    }

    drawBannerSlider(ctx, kind, val, enabled) {
      const sl = this.bannerSlider[kind];
      const track = sl.track;
      const [lo, hi] = PersonalizeMenu.bannerRange(kind);
      const frac = (val - lo) / (hi - lo);
      const base = kind === "size" ? M_ACCENT : [150, 200, 120];
      const col = enabled ? base : M_DIM;
      ui.text(ctx, t("ngb.banner_" + kind), sl.minus.left - 8, track.centery, this.small, col, "midright");
      draw.rect(ctx, M_TRACK, track, 0, 7);
      draw.rect(ctx, col, [track.left, track.top, Math.trunc(track.width * frac), track.height], 0, 7);
      const knob = track.left + Math.trunc(track.width * frac);
      draw.circle(ctx, enabled ? M_SEL : M_DIM, [knob, track.centery], 7);
      draw.circle(ctx, col, [knob, track.centery], 5);
      for (const [r, sym] of [[sl.minus, "-"], [sl.plus, "+"]]) {
        draw.rect(ctx, M_BTN, r, 0, 5);
        ui.text(ctx, sym, r.centerx, r.centery, this.fnt, enabled ? M_TEXT : M_DIM, "center");
      }
      ui.text(ctx, Math.round(val * 100) + "%", sl.plus.right + 8, track.centery, this.small, enabled ? M_TEXT : M_DIM, "midleft");
    }

    /** Live-Vorschau des Banners mit aktueller Größe/Deckkraft. */
    drawBannerPreview(ctx, cx, cy, cfg) {
      const bigTxt = "×1.4";
      const subTxt = t("snake.purple_banner");
      const pad = 14;
      const tw = Math.max(this.big.width(bigTxt), this.tiny.width(subTxt));
      const th = this.big.height + this.tiny.height + 4;
      const pw = tw + pad * 2, ph = th + pad;
      ctx.save();
      ctx.globalAlpha = cfg.opacity;
      ctx.translate(cx, cy);
      ctx.scale(cfg.size, cfg.size);
      ctx.translate(-pw / 2, -ph / 2);
      draw.rect(ctx, [18, 20, 32, 215], [0, 0, pw, ph], 0, 12);
      draw.rect(ctx, [150, 235, 150], [0, 0, pw, ph], 2, 12);
      ui.text(ctx, subTxt, pw / 2, 5, this.tiny, M_DIM, "midtop");
      ui.text(ctx, bigTxt, pw / 2, 5 + this.tiny.height + 2, this.big, [150, 235, 150], "midtop");
      ctx.restore();
    }

    drawTile(ctx, rect, col, selected, name) {
      draw.rect(ctx, col, rect, 0, 8);
      draw.rect(ctx, col.map((c) => Math.min(255, c + 50)), rect, 1, 8);
      if (selected) draw.rect(ctx, M_SEL, rect.inflate(8, 8), 2, 10);
      ui.text(ctx, name, rect.centerx, rect.bottom + 4, this.tiny, selected ? M_TEXT : M_DIM, "midtop");
    }

    /** Mini-Vorschau einer Farbreihenfolge (Reihen-Bänder + "1a"). */
    drawGridSwatch(ctx, rect, seq) {
      if (!seq) {
        draw.rect(ctx, [30, 32, 42], rect, 0, 8);
        draw.rect(ctx, M_DIM, rect, 1, 8);
        ui.text(ctx, "x", rect.centerx, rect.centery, this.small, M_DIM, "center");
        return;
      }
      const n = seq.length, rows = 4, rh = rect.height / rows;
      for (let j = 0; j < rows; j++) {
        draw.rect(ctx, seq[j % n], [rect.left, Math.trunc(rect.top + j * rh), rect.width, Math.trunc(rh) + 1]);
      }
      ui.text(ctx, "1a", rect.centerx, rect.centery, this.tiny, [232, 234, 240], "center");
      draw.rect(ctx, seq[0].map((c) => Math.min(255, c + 40)), rect, 1, 4);
    }

    drawSliders(ctx) {
      const ed = this.editable();
      if (!ed) return;
      const CH_COL = { R: [235, 90, 90], G: [90, 210, 120], B: [90, 150, 245] };
      const sliders = this.activeSliders();
      for (const ch of ["R", "G", "B"]) {
        const sl = sliders[ch];
        const track = sl.track;
        const val = ed["RGB".indexOf(ch)];
        draw.rect(ctx, M_TRACK, track, 0, 7);
        draw.rect(ctx, CH_COL[ch], [track.left, track.top, Math.trunc((track.width * val) / 255), track.height], 0, 7);
        const knobX = track.left + Math.trunc((track.width * val) / 255);
        draw.circle(ctx, M_SEL, [knobX, track.centery], 7);
        draw.circle(ctx, CH_COL[ch], [knobX, track.centery], 5);
        ui.text(ctx, ch, sl.minus.left - 6, track.centery, this.small, CH_COL[ch], "midright");
        for (const [r, sym] of [[sl.minus, "-"], [sl.plus, "+"]]) {
          draw.rect(ctx, M_BTN, r, 0, 5);
          ui.text(ctx, sym, r.centerx, r.centery, this.fnt, M_TEXT, "center");
        }
        ui.text(ctx, String(val), sl.plus.right + 8, track.centery, this.small, M_TEXT, "midleft");
      }
    }

    /** Kleine Vorschau: ein Schlangenkopf in der aktuell gewählten Farbe. */
    drawHeadPreview(ctx, cx, cy, color) {
      draw.rect(ctx, color, [cx - 15, cy - 15, 30, 30], 0, 8);
      for (const sign of [-1, 1]) {
        const ex = cx + sign * 6;
        draw.circle(ctx, [250, 250, 250], [ex, cy - 2], 3);
        draw.circle(ctx, [20, 20, 30], [ex + 1, cy - 1], 1);
      }
    }
  }

  // ======================================================================= Schlange
  /** Zustand der Schlange (Körper: Kopf am Array-Ende). */
  class Snake {
    constructor(body, direction) {
      this.body = body.map((c) => c.slice());
      this.direction = direction;
      this.nextDirection = direction;
      this.alive = true;
      this.score = 0;
      this.apples = 0;
      this.grow = 0; // ausstehende Wachstums-Blöcke
      this.stamina = STAMINA_MAX; // Boost-Ausdauer (0..1)
      this.boostOn = false;
      this.sizeFrac = 0; // Nachkomma-Rest der Größe (aus dem Gambling)
      this.prevBody = this.body.map((c) => c.slice()); // für die 3D-Interpolation
      this.turnQueue = []; // gepufferte Drehungen in der 3D-Ansicht ("L"/"R")
    }
  }

  // ======================================================================= Spiel
  class SnakeGame extends PG.Game {
    // ----- Aufbau / Reset ------------------------------------------------------
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.effectsDone = false;

      this.cols = Math.floor(this.width / CELL);
      this.rows = Math.floor(this.height / CELL);

      const snk = this.opts;
      this.wrap = !!snk.wrap;
      this.bonus = !!snk.bonus_apple;
      this.hardcore = !!snk.hardcore; // nur im Competitive aktiv
      const ac = Math.trunc(Number(snk.apples) || 1);
      this.appleCount = APPLE_COUNTS.includes(ac) ? ac : 1;
      const modeKey = snk.mode || "classic";
      this.modeIndex = MODE_KEYS.includes(modeKey) ? MODE_KEYS.indexOf(modeKey) : 0;
      this.view3d = !!snk.view3d;
      // 3D-Kamera-Optionen (Smooth-Shake, FOV, Höhe, Seitwärts-Shake)
      this.camSmooth = snk.cam_smooth == null ? true : !!snk.cam_smooth;
      this.camFov = PG.clamp(Number(snk.cam_fov != null ? snk.cam_fov : FOV_NORMAL) || FOV_NORMAL, CAM_FOV_MIN, CAM_FOV_MAX);
      this.camHeight = PG.clamp(Number(snk.cam_height != null ? snk.cam_height : CAM_H) || CAM_H, CAM_H_MIN, CAM_H_MAX);
      this.camTurnShake = !!snk.cam_turn_shake;
      if (this.view3dActive && !MODES_3D.includes(this.modeKey)) this.modeIndex = MODE_KEYS.indexOf("classic");

      this.font = ui.font(22);
      this.bigFont = ui.font(48, true);
      this.small = ui.font(16);
      this.tiny = ui.font(13);
      this.best = this.highscore;

      this.particles = [];
      this.animT = 0;
      this.ngbMenu = null; // aktives Personalisierungs-Menü (oder null)
      this.gridCache = null; // gecachtes Raster-Overlay (NGB)
      this.gridFont = ui.font(Math.max(9, CELL - 8), false, true);

      // Zustand der 3D-Ansicht
      this.particles3d = []; // [x, y, z, vx, vy, vz, life, farbe]
      this.shake = 0;
      this.skyCache = null;
      this.stars = [];
      for (let i = 0; i < 70; i++) {
        this.stars.push([PG.rand.uniform(0, PG.TAU), PG.rand.random(), PG.rand.choice([1, 1, 2]), PG.rand.uniform(0, PG.TAU)]);
      }

      this.buildSetupLayout();
      this.resetRunStats();
      this.newBoard();
      this.state = SETUP;
    }

    get modeKey() {
      return MODE_KEYS[this.modeIndex];
    }
    /** 3D-Ansicht (Web: immer Einzelspieler). */
    get view3dActive() {
      return this.view3d;
    }
    /** Competitive-Modus (Level-Aufstieg, Slot-Machine & Wett-Äpfel). */
    get competitive() {
      return this.modeKey === "competitive";
    }
    /** HARDCORE gibt es nur im Competitive: Boost frisst Länge, alles leuchtet rot. */
    get hardcoreActive() {
      return this.competitive && this.hardcore;
    }
    /** Wände-durchgehen; in der 3D-Ansicht sind die Wände immer fest. */
    get wrapActive() {
      return this.wrap && !this.view3dActive;
    }
    /** Im 3D-Kamera- und Personalisierungs-Menü bekommt das Spiel ESC selbst. */
    get wantsEscape() {
      return this.state === CAM3D || this.state === PERSONALIZE;
    }

    /** Indizes der aktuell wählbaren Spielmodi (3D: nur Klassisch/Hindernisse). */
    allowedModes() {
      if (this.view3dActive) return MODE_KEYS.map((k, i) => (MODES_3D.includes(k) ? i : -1)).filter((i) => i >= 0);
      return MODE_KEYS.map((_, i) => i);
    }

    resetRunStats() {
      this.applesTotal = 0;
      this.applesBank = 0;
      this.prestige = 0;
      this.speedApples = 0;
      // Competitive: Level-Aufstieg + Slot-Bonus (zusätzliche Äpfel auf Zeit)
      this.compLevel = 0;
      this.spawnBonus = 0;
      this.spawnBonusT = 0;
    }

    /** Baut die Schlange, das Modus-Layout und das erste Futter. */
    newBoard() {
      this.cols = Math.floor(this.width / CELL);
      this.rows = Math.floor(this.height / CELL);
      const cy = Math.floor(this.rows / 2);
      const cx = Math.floor(this.cols / 2);
      this.snakes = [new Snake([[cx - 2, cy], [cx - 1, cy], [cx, cy]], [1, 0])];

      this.timer = 0;
      this.interval = BASE_INTERVAL;
      this.buildModeLayout();
      this.placeFood();
      this.resetCamera();
    }

    /** Setzt die 3D-Verfolgerkamera hinter den Kopf. */
    resetCamera() {
      this.runT = 0; // Laufzeit der Runde (für Einblend-Hinweise)
      this.orbitA = 0; // Orbit-Winkel nach dem Game Over
      this.goLast = null; // Zeitmessung der Game-Over-Animation
      this.fovMul = this.camFov;
      const sn = this.snakes[0];
      const [hx, hy] = sn.body[sn.body.length - 1];
      const fx = sn.direction[0], fz = sn.direction[1];
      this.camDir = [fx, fz];
      const head = [hx + 0.5, 0, hy + 0.5];
      this.camPos = [head[0] - fx * CAM_BACK, this.camHeight, head[2] - fz * CAM_BACK];
      this.camLook = [head[0] + fx * CAM_AHEAD, CAM_LOOK_H, head[2] + fz * CAM_AHEAD];
    }

    startPlay() {
      this.score = 0;
      this.gameOver = false;
      this.effectsDone = false;
      this.particles = [];
      this.particles3d = [];
      this.shake = 0;
      this.resetRunStats();
      this.newBoard();
      this.state = PLAY;
    }

    // ----- Modus-Layout (Hindernisse / Portale / Zeit) ----------------------------
    buildModeLayout() {
      this.obstacles = new Map(); // key -> [x, y]
      this.portals = new Map(); // key -> Ziel [x, y]
      this.portalPairs = [];
      this.foods = new Map(); // alle aktuell liegenden Äpfel: key -> [x, y]
      this.foodAnim = new Map(); // key -> Einblend-Alter (Sek.), 2D-Morph
      this.eatFx = []; // 2D-Iss-Effekte: {cell, t}
      this.golden = null;
      this.goldenTimer = 0;
      this.timeLeft = TIMED_SECONDS;

      // Competitive-Spezialäpfel + Slot-Machine + aufsteigende Hinweistexte
      this.specials = new Map(); // key -> {cell, type, timer}
      this.slot = null;
      this.floatTexts = [];
      this.banner = null;
      this.purplePending = null;
      this.slotPending = null;

      // Zellen, die frei bleiben müssen (Schlange + Startbahn nach rechts)
      const tabu = new Set();
      for (const sn of this.snakes) {
        for (const [x, y] of sn.body) {
          for (let ddx = -1; ddx < 8; ddx++) tabu.add(key(x + ddx, y));
          tabu.add(key(x, y - 1));
          tabu.add(key(x, y + 1));
        }
      }
      if (this.modeKey === "walls") this.makeObstacles(tabu);
      else if (this.modeKey === "portal") this.makePortals(tabu);
    }

    makeObstacles(tabu) {
      const anzahl = Math.trunc(this.cols * this.rows * 0.05);
      let versuche = 0;
      while (this.obstacles.size < anzahl && versuche < anzahl * 30) {
        versuche++;
        // kleine Cluster (1-3 Blöcke) für interessantere Formen
        const bx = PG.rand.randint(1, this.cols - 2);
        const by = PG.rand.randint(2, this.rows - 2);
        const cluster = [[bx, by]];
        if (PG.rand.random() < 0.5) cluster.push([bx + PG.rand.choice([-1, 1]), by]);
        if (PG.rand.random() < 0.4) cluster.push([bx, by + PG.rand.choice([-1, 1])]);
        if (cluster.every(([x, y]) => !tabu.has(key(x, y)) && x > 0 && x < this.cols - 1 && y > 1 && y < this.rows - 1)) {
          for (const c of cluster) this.obstacles.set(key(c[0], c[1]), c);
        }
      }
    }

    makePortals(tabu) {
      const paare = this.cols * this.rows > 500 ? 2 : 1;
      for (let k = 0; k < paare; k++) {
        const enden = [];
        let versuche = 0;
        while (enden.length < 2 && versuche < 300) {
          versuche++;
          const p = [PG.rand.randint(1, this.cols - 2), PG.rand.randint(2, this.rows - 2)];
          const pk = key(p[0], p[1]);
          if (tabu.has(pk) || this.portals.has(pk) || enden.some((e) => e[0] === p[0] && e[1] === p[1])) continue;
          if (enden.length && Math.abs(enden[0][0] - p[0]) + Math.abs(enden[0][1] - p[1]) < 6) continue;
          enden.push(p);
          tabu.add(pk);
        }
        if (enden.length === 2) {
          const [a, b] = enden;
          this.portals.set(key(a[0], a[1]), b);
          this.portals.set(key(b[0], b[1]), a);
          this.portalPairs.push([a, b, PORTAL_COLORS[k % PORTAL_COLORS.length]]);
        }
      }
    }

    blockedCells() {
      const belegt = new Set([...this.obstacles.keys(), ...this.portals.keys()]);
      for (const sn of this.snakes) for (const [x, y] of sn.body) belegt.add(key(x, y));
      for (const k of this.foods.keys()) belegt.add(k);
      for (const k of this.specials.keys()) belegt.add(k);
      if (this.golden) belegt.add(key(this.golden[0], this.golden[1]));
      return belegt;
    }

    freeCells() {
      const belegt = this.blockedCells();
      const frei = [];
      for (let x = 0; x < this.cols; x++) for (let y = 0; y < this.rows; y++) if (!belegt.has(key(x, y))) frei.push([x, y]);
      return frei;
    }

    /** Gewünschte Anzahl gleichzeitig liegender (normaler) Äpfel. */
    appleTarget() {
      if (this.competitive) {
        const base = COMP.applesOnField(this.compLevel);
        const bonus = this.spawnBonusT > 0 ? this.spawnBonus : 0;
        return Math.min(COMP.MAX_APPLES + 5, base + bonus);
      }
      return this.appleCount;
    }

    /** Füllt das Feld auf die gewünschte Anzahl gleichzeitiger Äpfel auf. */
    placeFood() {
      const frei = PG.rand.shuffle(this.freeCells());
      const target = this.appleTarget();
      while (this.foods.size < target && frei.length) {
        const cell = frei.pop();
        const k = key(cell[0], cell[1]);
        this.foods.set(k, cell);
        this.foodAnim.set(k, 0); // startet die Einblend-Animation
      }
    }

    // ----- Competitive: Spezialäpfel (blau/lila) ---------------------------------
    maybeSpawnSpecial() {
      if (this.specials.size) return; // immer nur ein Spezialapfel gleichzeitig
      const r = PG.rand.random();
      if (r < BLUE_CHANCE) this.placeSpecial("blue");
      else if (r < BLUE_CHANCE + PURPLE_CHANCE) this.placeSpecial("purple");
    }

    placeSpecial(typ) {
      const frei = this.freeCells();
      if (frei.length) {
        const c = PG.rand.choice(frei);
        this.specials.set(key(c[0], c[1]), { cell: c, type: typ, timer: SPECIAL_LIFETIME });
      }
    }

    placeGolden() {
      const frei = this.freeCells();
      if (frei.length) {
        this.golden = PG.rand.choice(frei);
        this.goldenTimer = GOLDEN_LIFETIME;
      }
    }

    // ----- Setup-Screen --------------------------------------------------------------
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(420, this.width - 60);
      const R = (x, y, w, h) => new PG.Rect(x, y, w, h);

      // Personalisieren-Knopf (Pinsel) - ganz oben rechts, ohne Beschriftung
      this.brushRect = R(this.width - 42, 8, 34, 34);

      // Modus-Auswahl: Pfeile + Panel
      this.modePanel = R(cx - Math.floor(bw / 2), 106, bw, 58);
      this.modeLeft = R(this.modePanel.left, 106, 40, 58);
      this.modeRight = R(this.modePanel.right - 40, 106, 40, 58);

      const bh = 40, gap = 6, y0 = 180;
      const x0 = cx - Math.floor(bw / 2);
      this.viewRect = R(x0, y0, bw, bh);
      this.wrapRect = R(x0, y0 + (bh + gap), bw, bh);
      this.bonusRect = R(x0, y0 + 2 * (bh + gap), bw, bh);
      this.applesRect = R(x0, y0 + 3 * (bh + gap), bw, bh);
      this.startRect = R(cx - 95, y0 + 4 * (bh + gap) + 4, 190, 48);

      // 3D-Kamera-Menü (eigener Screen; nur im 3D-Modus erreichbar)
      const cbw = Math.min(440, this.width - 50), cbh = 46, cgap = 10, cy0 = 150;
      const cx0 = cx - Math.floor(cbw / 2);
      this.camSmoothRect = R(cx0, cy0, cbw, cbh);
      this.camFovRect = R(cx0, cy0 + (cbh + cgap), cbw, cbh);
      this.camHeightRect = R(cx0, cy0 + 2 * (cbh + cgap), cbw, cbh);
      this.camTurnRect = R(cx0, cy0 + 3 * (cbh + cgap), cbw, cbh);
      this.camBackRect = R(cx - 95, cy0 + 4 * (cbh + cgap) + 8, 190, 46);

      // -/+ Knöpfe rechts in einer Wertzeile
      const pm = (rect) => [R(rect.right - 148, rect.centery - 16, 32, 32), R(rect.right - 46, rect.centery - 16, 32, 32)];
      [this.camFovMinus, this.camFovPlus] = pm(this.camFovRect);
      [this.camHeightMinus, this.camHeightPlus] = pm(this.camHeightRect);
    }

    saveSnakeSetting(k, value) {
      this.opts[k] = value;
      this.saveSettings();
    }

    toggleSetting(which) {
      if (which === "wrap" && this.view3dActive) return; // in 3D sind die Wände immer fest
      if (which === "wrap") {
        this.wrap = !this.wrap;
        this.saveSnakeSetting("wrap", this.wrap);
      } else {
        this.bonus = !this.bonus;
        this.saveSnakeSetting("bonus_apple", this.bonus);
      }
      this.playSound("select");
    }

    /** HARDCORE ein/aus (nur im Competitive wählbar). */
    toggleHardcore() {
      if (!this.competitive) return;
      this.hardcore = !this.hardcore;
      this.saveSnakeSetting("hardcore", this.hardcore);
      this.playSound("select");
    }

    /** Schaltet die Anzahl gleichzeitig liegender Äpfel weiter (1/2/3/5). */
    cycleApples() {
      if (this.competitive) return; // im Competitive bestimmt das Level die Anzahl
      const i = Math.max(0, APPLE_COUNTS.indexOf(this.appleCount));
      this.appleCount = APPLE_COUNTS[(i + 1) % APPLE_COUNTS.length];
      this.saveSnakeSetting("apples", this.appleCount);
      this.playSound("select");
    }

    cycleMode(step) {
      const allowed = this.allowedModes();
      if (allowed.includes(this.modeIndex)) {
        const i = allowed.indexOf(this.modeIndex);
        this.modeIndex = allowed[PG.mod(i + step, allowed.length)];
      } else {
        this.modeIndex = allowed[0];
      }
      this.saveSnakeSetting("mode", this.modeKey);
      this.playSound("click");
    }

    /** Schaltet zwischen 2D- und 3D-Ansicht um. */
    toggleView() {
      this.view3d = !this.view3d;
      if (this.view3d && !MODES_3D.includes(this.modeKey)) {
        this.modeIndex = MODE_KEYS.indexOf("classic");
        this.saveSnakeSetting("mode", this.modeKey);
      }
      this.saveSnakeSetting("view3d", this.view3d);
      this.playSound("select");
    }

    /** Öffnet das Personalisierungs-Menü (nur Optik). */
    openPersonalize() {
      this.ngbMenu = new PersonalizeMenu(this.width, this.height, (n) => this.playSound(n));
      this.achEvent("painter"); // Erfolg "Ganz mein Stil"
      this.state = PERSONALIZE;
      this.playSound("click");
    }

    openCam3d() {
      this.state = CAM3D;
      this.playSound("click");
    }

    adjustFov(d) {
      this.camFov = round2(PG.clamp(this.camFov + d, CAM_FOV_MIN, CAM_FOV_MAX));
      this.saveSnakeSetting("cam_fov", this.camFov);
      this.playSound("click");
    }

    adjustHeight(d) {
      this.camHeight = round2(PG.clamp(this.camHeight + d, CAM_H_MIN, CAM_H_MAX));
      this.saveSnakeSetting("cam_height", this.camHeight);
      this.playSound("click");
    }

    handleCam3dEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["Escape", "Return", "space", "k", "K"].includes(k)) {
          this.state = SETUP;
          this.playSound("click");
        } else if (k === "Left" || k === "a" || k === "A") this.adjustFov(-CAM_FOV_STEP);
        else if (k === "Right" || k === "d" || k === "D") this.adjustFov(+CAM_FOV_STEP);
        else if (k === "Up" || k === "w" || k === "W") this.adjustHeight(+CAM_H_STEP);
        else if (k === "Down" || k === "s" || k === "S") this.adjustHeight(-CAM_H_STEP);
        return;
      }
      if (ev.kind !== "mousedown" || !ev.pos) return;
      const p = ev.pos;
      if (this.camSmoothRect.collidepoint(p)) {
        this.camSmooth = !this.camSmooth;
        this.saveSnakeSetting("cam_smooth", this.camSmooth);
        this.playSound("select");
      } else if (this.camTurnRect.collidepoint(p)) {
        this.camTurnShake = !this.camTurnShake;
        this.saveSnakeSetting("cam_turn_shake", this.camTurnShake);
        this.playSound("select");
      } else if (this.camFovMinus.collidepoint(p)) this.adjustFov(-CAM_FOV_STEP);
      else if (this.camFovPlus.collidepoint(p)) this.adjustFov(+CAM_FOV_STEP);
      else if (this.camHeightMinus.collidepoint(p)) this.adjustHeight(-CAM_H_STEP);
      else if (this.camHeightPlus.collidepoint(p)) this.adjustHeight(+CAM_H_STEP);
      else if (this.camBackRect.collidepoint(p)) {
        this.state = SETUP;
        this.playSound("click");
      }
    }

    handleSetupEvent(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Left" || k === "a" || k === "A") this.cycleMode(-1);
        else if (k === "Right" || k === "d" || k === "D") this.cycleMode(+1);
        else if (["1", "2", "3", "4", "5", "6"].includes(k)) {
          const allowed = this.allowedModes();
          const idx = Number(k) - 1;
          if (idx < allowed.length) {
            this.modeIndex = allowed[idx];
            this.saveSnakeSetting("mode", this.modeKey);
            this.playSound("click");
          }
        } else if (k === "v" || k === "V") this.toggleView();
        else if (k === "w" || k === "W") this.toggleSetting("wrap");
        else if (k === "b" || k === "B") this.toggleSetting("bonus");
        else if (k === "f" || k === "F") this.cycleApples();
        else if (k === "h" || k === "H") this.toggleHardcore();
        else if (k === "c" || k === "C") this.openPersonalize();
        else if (k === "k" || k === "K") {
          if (this.view3dActive) this.openCam3d();
        } else if (k === "Return" || k === "space") {
          this.playSound("click");
          this.startPlay();
        }
      } else if (ev.kind === "mousedown" && ev.pos) {
        const p = ev.pos;
        if (this.brushRect.collidepoint(p)) this.openPersonalize();
        else if (this.modeLeft.collidepoint(p)) this.cycleMode(-1);
        else if (this.modeRight.collidepoint(p)) this.cycleMode(+1);
        else if (this.modePanel.collidepoint(p)) this.cycleMode(+1);
        else if (this.viewRect.collidepoint(p)) this.toggleView();
        else if (this.wrapRect.collidepoint(p)) {
          if (this.view3dActive) this.openCam3d(); // in 3D ist die Zeile der Kamera-Knopf
          else this.toggleSetting("wrap");
        } else if (this.bonusRect.collidepoint(p)) this.toggleSetting("bonus");
        else if (this.applesRect.collidepoint(p)) {
          if (this.competitive) this.toggleHardcore(); // im Competitive der HARDCORE-Schalter
          else this.cycleApples();
        } else if (this.startRect.collidepoint(p)) {
          this.playSound("click");
          this.startPlay();
        }
      }
    }

    // ----- Eingabe (Spiel) ---------------------------------------------------------------
    handleEvent(ev) {
      if (this.state === PERSONALIZE) {
        if (this.ngbMenu) {
          this.ngbMenu.handleEvent(ev);
          if (this.ngbMenu.done) {
            this.ngbMenu = null;
            this.gridCache = null; // Raster-Overlay ggf. neu rendern
            this.state = SETUP;
          }
        }
        return;
      }
      if (this.state === CAM3D) return this.handleCam3dEvent(ev);
      if (this.state === SETUP) return this.handleSetupEvent(ev);

      // Während die Slot-Machine läuft, ist die Steuerung ausgesetzt.
      if (this.slot) return;

      // Boost beenden, sobald die Taste losgelassen wird
      if (ev.kind === "keyup") {
        if (BOOST_KEYS.includes(ev.key)) this.setBoost(this.snakes[0], false);
        return;
      }
      if (ev.kind !== "keydown") return;

      if (this.gameOver) {
        if (ev.key === "Return" || ev.key === "space") this.startPlay();
        return;
      }

      // Boost aktivieren, solange die Taste gehalten wird
      if (BOOST_KEYS.includes(ev.key)) {
        this.setBoost(this.snakes[0], true);
        return;
      }

      // Prestige
      if (ev.key === "p" || ev.key === "P") {
        this.tryPrestige();
        return;
      }

      if (this.view3dActive) this.steer3d(this.snakes[0], ev.key);
      else this.turn(this.snakes[0], ev.key);
    }

    setBoost(sn, on) {
      if (!sn.alive) return;
      if (on) {
        // Nur neu starten, wenn genug Ausdauer da ist
        if (!sn.boostOn && sn.stamina >= BOOST_MIN_START) {
          sn.boostOn = true;
          this.playSound("rotate");
        }
      } else {
        sn.boostOn = false;
      }
    }

    /** Relatives Lenken in 3D: bis zu zwei Drehungen werden gepuffert. */
    steer3d(sn, k) {
      let turn = null;
      if (this.isAction(k, "left") || k === "Left") turn = "L";
      else if (this.isAction(k, "right") || k === "Right") turn = "R";
      if (turn && sn.turnQueue.length < 2) sn.turnQueue.push(turn);
    }

    turn(sn, k) {
      const [dx, dy] = sn.direction;
      if (this.isAction(k, "up") && dy === 0) sn.nextDirection = [0, -1];
      else if (this.isAction(k, "down") && dy === 0) sn.nextDirection = [0, 1];
      else if (this.isAction(k, "left") && dx === 0) sn.nextDirection = [-1, 0];
      else if (this.isAction(k, "right") && dx === 0) sn.nextDirection = [1, 0];
    }

    // ----- Prestige ------------------------------------------------------------------------
    canPrestige() {
      const req = PRESTIGE.nextRequirement(this.prestige);
      if (!req) return [null, false];
      const sn = this.snakes[0];
      const genugApfel = this.applesBank >= req.apples;
      const genugLaenge = sn.body.length - req.length >= MIN_LENGTH;
      return [req, genugApfel && genugLaenge];
    }

    tryPrestige() {
      if (this.gameOver || this.competitive) return; // Competitive nutzt automatischen Level-Aufstieg
      const [req, ok] = this.canPrestige();
      if (!ok) return;
      this.applesBank -= req.apples;
      const sn = this.snakes[0];
      sn.body.splice(0, req.length);
      sn.grow = 0;
      this.prestige += 1;
      this.achEvent("snake_prestige");
      this.playSound("select");
      this.rumble(120);
    }

    // ----- Spiellogik ------------------------------------------------------------------------
    update(dt) {
      this.animT += dt;
      this.updateParticles(dt);
      this.updateParticles3d(dt);
      this.updateFoodAnim(dt);
      this.updateFloatTexts(dt);
      this.updateBanner(dt);
      if (this.shake > 0) this.shake = Math.max(0, this.shake - dt * 1.6);
      if (this.state === PLAY && this.view3dActive) this.updateCamera(dt); // läuft auch nach Game Over (Orbit)
      if (this.state !== PLAY || this.gameOver) return;

      // Die Slot-Machine friert die Spielwelt ein, solange sie läuft.
      if (this.slot) {
        this.updateSlot(dt);
        return;
      }

      this.runT += dt;
      if (this.competitive) this.updateCompetitive(dt);

      // Goldapfel-Lebensdauer (+ Funkeln in 3D)
      if (this.golden) {
        this.goldenTimer -= dt;
        if (this.goldenTimer <= 0) this.golden = null;
        else if (this.view3dActive && PG.rand.random() < dt * 6) {
          const [gx, gy] = this.golden;
          this.particles3d.push([
            gx + 0.5 + PG.rand.uniform(-0.2, 0.2), 0.55, gy + 0.5 + PG.rand.uniform(-0.2, 0.2),
            PG.rand.uniform(-0.4, 0.4), PG.rand.uniform(0.8, 1.8), PG.rand.uniform(-0.4, 0.4), 0.55, COL_GOLD,
          ]);
        }
      }

      // Zeitangriff-Countdown
      if (this.modeKey === "timed") {
        this.timeLeft -= dt;
        if (this.timeLeft <= 0) {
          this.timeLeft = 0;
          this.finishTimed();
          return;
        }
      }

      // Ausdauer regenerieren (wenn nicht aktiv geboostet)
      for (const sn of this.snakes) {
        if (sn.alive && !sn.boostOn) sn.stamina = Math.min(STAMINA_MAX, sn.stamina + STAMINA_REGEN * dt);
      }

      // Schritte abarbeiten
      this.timer += dt;
      let guard = 0;
      while (this.timer >= this.interval && !this.gameOver && !this.slot) {
        this.timer -= this.interval;
        this.tick();
        guard++;
        if (guard > 8) {
          this.timer = 0;
          break;
        }
      }
    }

    tick() {
      const alive = this.snakes.map((sn, i) => (sn.alive ? i : -1)).filter((i) => i >= 0);
      // Unterschritt 0: alle bewegen sich einmal
      this.advance(alive);
      if (this.gameOver || this.slot) return; // ein blauer Apfel hat die Slot-Machine geöffnet
      // Unterschritt 1: Booster bewegen sich ein zweites Mal (= doppeltes Tempo)
      const booster = alive.filter((i) => this.snakes[i].alive && this.snakes[i].boostOn && this.snakes[i].stamina > 0);
      if (booster.length) {
        this.advance(booster);
        for (const i of booster) {
          const sn = this.snakes[i];
          sn.stamina = Math.max(0, sn.stamina - BOOST_DRAIN);
          if (sn.boostOn) {
            this.spawnParticles(sn.body[sn.body.length - 1], BOOST_GLOW[0], 2);
            if (this.hardcoreActive) this.hardcoreBoostCost(sn); // Boost frisst Länge
          }
          if (sn.stamina <= 0) sn.boostOn = false;
        }
      }
    }

    /** HARDCORE: jeder Boost-Schritt kostet Länge (nie unter die Mindestlänge). */
    hardcoreBoostCost(sn) {
      let cost = HARDCORE_BOOST_LEN_COST;
      if (sn.grow > 0) {
        const used = Math.min(sn.grow, cost);
        sn.grow -= used;
        cost -= used;
      }
      if (cost > 0) {
        const schnitt = Math.min(cost, sn.body.length - MIN_LENGTH);
        if (schnitt > 0) {
          this.spawnParticles(sn.body[0], COL_HARDCORE, 3);
          sn.body.splice(0, schnitt);
          sn.prevBody = sn.body.map((c) => c.slice());
        } else {
          sn.boostOn = false; // Mindestlänge erreicht -> Boost aus
        }
      }
    }

    /** Bewegt die Schlangen in 'movers' um eine Zelle (mit voller Kollision). */
    advance(movers) {
      const newHeads = new Map();
      for (const i of movers) {
        const sn = this.snakes[i];
        if (!sn.alive) continue;
        sn.prevBody = sn.body.map((c) => c.slice());
        if (this.view3dActive && sn.turnQueue.length) {
          const alt = sn.direction;
          sn.direction = rotateDir(sn.direction, sn.turnQueue.shift());
          sn.nextDirection = sn.direction;
          if (this.camTurnShake && (sn.direction[0] !== alt[0] || sn.direction[1] !== alt[1])) {
            this.shake = Math.max(this.shake, 0.22); // Ruckeln beim Abbiegen
          }
        } else {
          sn.direction = sn.nextDirection;
        }
        const [hx, hy] = sn.body[sn.body.length - 1];
        let nx = hx + sn.direction[0], ny = hy + sn.direction[1];
        const portal = this.portals.get(key(nx, ny));
        if (portal) {
          nx = portal[0] + sn.direction[0];
          ny = portal[1] + sn.direction[1];
        }
        if (this.wrapActive) {
          nx = PG.mod(nx, this.cols);
          ny = PG.mod(ny, this.rows);
        }
        newHeads.set(i, [nx, ny]);
      }

      const tot = new Set();

      // Wandkollision (nur bei festen Wänden)
      if (!this.wrapActive) {
        for (const [i, [nx, ny]] of newHeads) if (nx < 0 || nx >= this.cols || ny < 0 || ny >= this.rows) tot.add(i);
      }
      // Hindernisse (immer tödlich)
      for (const [i, h] of newHeads) if (this.obstacles.has(key(h[0], h[1]))) tot.add(i);
      // Kopf-an-Kopf
      for (const [i, a] of newHeads) {
        for (const [j, b] of newHeads) if (i < j && a[0] === b[0] && a[1] === b[1]) {
          tot.add(i);
          tot.add(j);
        }
      }
      // Körperkollision
      const belegt = new Set();
      this.snakes.forEach((sn, i) => {
        if (!sn.alive) return;
        let koerper = sn.body;
        if (movers.includes(i)) {
          const h = newHeads.get(i);
          const hk = h ? key(h[0], h[1]) : null;
          const waechst = (hk && this.foods.has(hk)) || (this.golden && h && h[0] === this.golden[0] && h[1] === this.golden[1]) || sn.grow > 0;
          if (!waechst) koerper = sn.body.slice(1);
        }
        for (const [x, y] of koerper) belegt.add(key(x, y));
      });
      for (const [i, h] of newHeads) if (belegt.has(key(h[0], h[1]))) tot.add(i);

      // Bewegung anwenden
      let ate = false, ateGold = false;
      for (const i of movers) {
        const sn = this.snakes[i];
        if (!sn.alive) continue;
        if (tot.has(i)) {
          sn.alive = false;
          this.spawnParticles(sn.body[sn.body.length - 1], SNAKE_COLORS[0][1], 12);
          if (this.view3dActive) this.shake = 0.6; // Kamera-Wackler beim Crash
          continue;
        }
        const kopf = newHeads.get(i);
        const kk = key(kopf[0], kopf[1]);
        sn.body.push(kopf);
        if (this.foods.has(kk)) {
          this.foods.delete(kk);
          this.foodAnim.delete(kk);
          if (!this.view3dActive) this.spawnEatFx(kopf);
          this.eatFood(sn);
          ate = true;
        } else if (this.golden && kopf[0] === this.golden[0] && kopf[1] === this.golden[1]) {
          this.eatGolden(sn);
          ateGold = true;
        } else if (this.specials.has(kk)) {
          this.eatSpecial(sn, kk);
        }
        if (sn.grow > 0) sn.grow -= 1;
        else sn.body.shift();
      }

      // Spezialäpfel wirken erst nach dem Schritt (verändern u.U. die Länge)
      if (this.purplePending) {
        this.applyPurple(this.purplePending);
        this.purplePending = null;
      }
      if (this.slotPending) {
        this.openSlot(this.slotPending);
        this.slotPending = null;
      }

      if (ate) {
        this.playSound("eat");
        this.placeFood();
        if (!this.golden && PG.rand.random() < GOLDEN_CHANCE) this.placeGolden();
        if (this.competitive) this.maybeSpawnSpecial();
      }
      if (ateGold) this.golden = null;

      this.checkEnd(tot);
    }

    eatFood(sn) {
      const gain = this.bonus ? PG.rand.randint(1, 2) : 1;
      sn.grow += gain * PRESTIGE.blocksPerApple(this.prestige);
      sn.stamina = Math.min(STAMINA_MAX, sn.stamina + 0.12);
      sn.apples += gain;
      this.applesTotal += gain;
      this.applesBank += gain;
      sn.score += gain * 10 * this.scoreMultiplier();
      this.speedApples += gain;
      if (this.modeKey === "speed") this.interval = Math.max(MIN_INTERVAL, BASE_INTERVAL - this.speedApples * 0.0025);
    }

    eatGolden(sn) {
      sn.grow += 2 * PRESTIGE.blocksPerApple(this.prestige);
      sn.stamina = STAMINA_MAX;
      const bonus = 3;
      sn.apples += bonus;
      this.applesTotal += bonus;
      this.applesBank += bonus;
      sn.score += 50 * this.scoreMultiplier();
      this.spawnParticles(sn.body[sn.body.length - 1], COL_GOLD, 16);
      this.playSound("point");
      this.rumble(80);
    }

    /** Punkte-Multiplikator: Competitive nutzt das Level, sonst Prestige. */
    scoreMultiplier() {
      if (this.competitive) return COMP.scoreMultiplier(this.compLevel);
      return PRESTIGE.scoreMultiplier(this.prestige);
    }

    // ----- Competitive: Spezialäpfel-Wirkung ------------------------------------------------
    eatSpecial(sn, k) {
      const info = this.specials.get(k);
      this.specials.delete(k);
      const typ = info ? info.type : null;
      if (typ === "blue") this.slotPending = sn;
      else if (typ === "purple") this.purplePending = sn;
    }

    /** Wahre "Größe" als Kommazahl = Länge + ausstehendes Wachstum + Nachkomma-Rest. */
    snakeSize(sn) {
      return sn.body.length + sn.grow + sn.sizeFrac;
    }

    /** Setzt die Größe (Kommazahl) und passt Körper/Wachstum an (nie unter Mindestlänge). */
    setSnakeSize(sn, size) {
      size = Math.max(MIN_LENGTH, size);
      const targetLen = Math.floor(size + 1e-9);
      sn.sizeFrac = size - targetLen;
      const delta = targetLen - (sn.body.length + sn.grow);
      if (delta > 0) sn.grow += delta;
      else if (delta < 0) {
        let need = -delta;
        const used = Math.min(sn.grow, need); // erst ausstehendes Wachstum abbauen
        sn.grow -= used;
        need -= used;
        if (need > 0) {
          const schnitt = Math.min(need, sn.body.length - MIN_LENGTH);
          if (schnitt > 0) {
            sn.body.splice(0, schnitt);
            sn.prevBody = sn.body.map((c) => c.slice());
          }
        }
      }
    }

    /**
     * Lila Apfel (Gambling): Anteil der Größe wird mit Zufallsfaktor multipliziert.
     * Normal: 50 % Einsatz, x0.5..x1.5.  HARDCORE: 75-90 % Einsatz, x0.25..x2.25.
     */
    applyPurple(sn) {
      const hc = this.hardcoreActive;
      const stake = COMP.purpleStake(hc);
      const factor = COMP.purpleFactor(hc);
      const size = this.snakeSize(sn);
      const newSize = Math.max(MIN_LENGTH, size * (1 - stake) + size * stake * factor);
      this.setSnakeSize(sn, newSize);
      const gewonnen = newSize >= size;
      const col = gewonnen ? ui.GREEN : ui.RED;
      this.playSound(gewonnen ? "powerup" : "hit");
      this.rumble(80);
      const head = sn.body[sn.body.length - 1];
      this.spawnParticles(head, COL_PURPLE, 14);
      this.addFloatText(head, "x" + factor, col);
      this.showBanner((stake * 100).toFixed(0) + "% ×" + factor, col, t("snake.purple_banner"));
    }

    showBanner(text, color, sub) {
      this.banner = { text, color, sub, t: BANNER_TIME };
    }

    updateBanner(dt) {
      if (this.banner) {
        this.banner.t -= dt;
        if (this.banner.t <= 0) this.banner = null;
      }
    }

    // ----- Competitive: Slot-Machine -------------------------------------------------------
    openSlot(sn) {
      const reels = COMP.spinReels();
      const [mult, result] = COMP.slotOutcome(reels);
      this.slot = { snake: sn, reels, mult, result, stake: Math.max(2, Math.floor(sn.body.length / 4)), stop: SLOT_REEL_STOPS.slice(), t: 0, applied: false, snd: [false, false, false] };
      this.playSound("point");
    }

    updateSlot(dt) {
      const sl = this.slot;
      sl.t += dt;
      sl.stop.forEach((ts, k) => {
        if (sl.t >= ts && !sl.snd[k]) {
          sl.snd[k] = true;
          this.playSound("lock");
        }
      });
      const last = sl.stop[sl.stop.length - 1];
      if (sl.t >= last && !sl.applied) {
        sl.applied = true;
        this.applySlot(sl);
      }
      if (sl.t >= last + SLOT_SHOW) this.slot = null;
    }

    /** Verrechnet das Slot-Ergebnis: Längeneinsatz + zeitweise mehr Äpfel. */
    applySlot(sl) {
      const sn = sl.snake;
      const netto = Math.round(sl.stake * (sl.mult - 1));
      if (netto > 0) sn.grow += netto;
      else if (netto < 0) {
        const schnitt = Math.min(-netto, sn.body.length - MIN_LENGTH);
        if (schnitt > 0) {
          sn.body.splice(0, schnitt);
          sn.prevBody = sn.body.map((c) => c.slice());
        }
      }
      // Der Multiplikator lässt für kurze Zeit zusätzliche Äpfel spawnen.
      this.spawnBonus = Math.max(this.spawnBonus, Math.round(sl.mult));
      this.spawnBonusT = SPAWN_BONUS_TIME;
      this.placeFood();
      if (sl.result === "jackpot") {
        this.playSound("win");
        this.rumble(160);
      } else if (sl.result === "pair") {
        this.playSound("powerup");
        this.rumble(80);
      } else {
        this.playSound("hit");
      }
      this.addFloatText(sn.body[sn.body.length - 1], "x" + sl.mult, COL_BLUE);
    }

    // ----- Aufsteigende Hinweistexte ------------------------------------------------------
    addFloatText(cell, text, color) {
      this.floatTexts.push({ x: cell[0] * CELL + CELL / 2, y: cell[1] * CELL, text, color, t: 1.1 });
    }

    updateFloatTexts(dt) {
      this.floatTexts = this.floatTexts.filter((ft) => {
        ft.y -= dt * 26;
        ft.t -= dt;
        return ft.t > 0;
      });
    }

    /** Level aus gesammelten Äpfeln, Slot-Bonus und Spezialapfel-Lebensdauer. */
    updateCompetitive(dt) {
      const lvl = COMP.levelForApples(this.applesTotal);
      if (lvl > this.compLevel) {
        this.compLevel = lvl;
        this.achEvent("snake_comp5", lvl);
        this.playSound("level");
        this.rumble(90);
        const sn = this.snakes[0];
        this.addFloatText(sn.body[sn.body.length - 1], t("snake.comp.levelup", { n: lvl }), ui.GOLD);
        this.placeFood(); // das neue Level legt einen Apfel nach
      }
      if (this.spawnBonusT > 0) {
        this.spawnBonusT = Math.max(0, this.spawnBonusT - dt);
        if (this.spawnBonusT === 0) this.spawnBonus = 0;
      }
      for (const [k, info] of [...this.specials]) {
        info.timer -= dt;
        if (info.timer <= 0) this.specials.delete(k);
      }
    }

    checkEnd() {
      this.score = this.snakes[0].score;
      if (!this.snakes[0].alive) {
        this.gameOver = true;
        this.endEffects();
      }
    }

    /** Zeitangriff abgelaufen: beenden. */
    finishTimed() {
      this.gameOver = true;
      this.score = this.snakes[0].score;
      this.endEffects();
    }

    endEffects() {
      if (this.effectsDone) return;
      this.effectsDone = true;
      this.best = Math.max(this.best, this.score);
      this.playSound("gameover");
      this.rumble(200);
    }

    // ----- Partikel ---------------------------------------------------------------------
    spawnParticles(cell, color, n) {
      if (this.view3dActive) {
        // 3D-Funken: fliegen aus der Zelle hoch und prallen am Boden ab
        const wx = cell[0] + 0.5, wz = cell[1] + 0.5;
        for (let i = 0; i < n; i++) {
          const ang = PG.rand.uniform(0, PG.TAU);
          const spd = PG.rand.uniform(1.0, 4.0);
          this.particles3d.push([wx, PG.rand.uniform(0.2, 0.6), wz, Math.cos(ang) * spd, PG.rand.uniform(1.0, 4.5), Math.sin(ang) * spd, PG.rand.uniform(0.35, 0.7), color]);
        }
        return;
      }
      const cx = cell[0] * CELL + CELL / 2;
      const cy = cell[1] * CELL + CELL / 2;
      for (let i = 0; i < n; i++) {
        const ang = PG.rand.uniform(0, PG.TAU);
        const spd = PG.rand.uniform(30, 130);
        this.particles.push([cx, cy, Math.cos(ang) * spd, Math.sin(ang) * spd, PG.rand.uniform(0.25, 0.55), color]);
      }
    }

    updateParticles3d(dt) {
      this.particles3d = this.particles3d.filter((p) => {
        p[0] += p[3] * dt;
        p[1] += p[4] * dt;
        p[2] += p[5] * dt;
        p[4] -= 9.0 * dt; // Schwerkraft
        if (p[1] < 0.04) {
          // am Boden abprallen
          p[1] = 0.04;
          p[4] *= -0.4;
        }
        p[6] -= dt;
        return p[6] > 0;
      });
    }

    updateParticles(dt) {
      this.particles = this.particles.filter((p) => {
        p[0] += p[2] * dt;
        p[1] += p[3] * dt;
        p[4] -= dt;
        return p[4] > 0;
      });
    }

    updateFoodAnim(dt) {
      for (const [k, v] of [...this.foodAnim]) {
        if (this.foods.has(k)) this.foodAnim.set(k, Math.min(FOOD_SPAWN_ANIM, v + dt));
        else this.foodAnim.delete(k);
      }
      this.eatFx = this.eatFx.filter((e) => {
        e.t -= dt;
        return e.t > 0;
      });
    }

    spawnEatFx(cell) {
      this.eatFx.push({ cell, t: FOOD_EAT_ANIM });
      this.spawnParticles(cell, COL_FOOD, 8); // rote Krümel fliegen weg
    }

    // ----- 3D-Kamera -------------------------------------------------------------------
    /** Verfolgerkamera; nach dem Game Over kreist sie langsam um die Schlange. */
    updateCamera(dt) {
      const sn = this.snakes[0];
      const cells = this.interpCells(sn);
      const [hx, hz] = cells[cells.length - 1];
      const head = [hx + 0.5, 0, hz + 0.5];
      let tx, tz, back, look = null;
      if (this.gameOver) {
        this.orbitA += dt * 0.55;
        tx = Math.sin(this.orbitA);
        tz = Math.cos(this.orbitA);
        back = CAM_BACK + 1.6;
        look = [head[0], 0.25, head[2]];
      } else {
        tx = sn.direction[0];
        tz = sn.direction[1];
        back = CAM_BACK;
      }

      // Blickrichtung weich nachziehen (Smooth-Shake = sanftere Glättung)
      const dirRate = this.camSmooth ? 2.6 : 4.5;
      const posRate = CAM_SMOOTH * (this.camSmooth ? 0.55 : 1.0);
      const k = Math.min(1, dt * dirRate);
      let fx = this.camDir[0] + (tx - this.camDir[0]) * k;
      let fz = this.camDir[1] + (tz - this.camDir[1]) * k;
      const ln = Math.hypot(fx, fz);
      if (ln > 1e-6) this.camDir = [fx / ln, fz / ln];
      [fx, fz] = this.camDir;

      const tgtPos = [head[0] - fx * back, this.camHeight, head[2] - fz * back];
      const tgtLook = look || [head[0] + fx * CAM_AHEAD, CAM_LOOK_H, head[2] + fz * CAM_AHEAD];
      const kp = Math.min(1, dt * posRate);
      for (let i = 0; i < 3; i++) {
        this.camPos[i] += (tgtPos[i] - this.camPos[i]) * kp;
        this.camLook[i] += (tgtLook[i] - this.camLook[i]) * kp;
      }

      // Sichtfeld: Grundwert aus den Optionen, beim Boost weitwinkliger.
      const boost = sn.alive && sn.boostOn && sn.stamina > 0;
      const want = this.camFov - (boost ? FOV_BOOST_DELTA : 0);
      this.fovMul += (want - this.fovMul) * Math.min(1, dt * 6);
    }

    /** Körperzellen zwischen letztem und aktuellem Schritt interpoliert (Kopf am Ende). */
    interpCells(sn) {
      const frac = this.gameOver || !sn.alive ? 1 : PG.clamp(this.timer / Math.max(1e-6, this.interval), 0, 1);
      const out = [];
      const n = sn.body.length, m = sn.prevBody.length;
      for (let k = 0; k < n; k++) {
        const cur = sn.body[k];
        const pi = m - (n - k); // gleiches Segment, vom Kopf her ausgerichtet
        let prev = pi >= 0 && pi < m ? sn.prevBody[pi] : cur;
        if (Math.abs(cur[0] - prev[0]) + Math.abs(cur[1] - prev[1]) > 1.5) prev = cur; // Teleport/Spawn
        out.push([prev[0] + (cur[0] - prev[0]) * frac, prev[1] + (cur[1] - prev[1]) * frac]);
      }
      return out;
    }

    // ----- Zeichnen ---------------------------------------------------------------------------
    draw(ctx) {
      if (this.state === PERSONALIZE) {
        if (this.ngbMenu) this.ngbMenu.draw(ctx);
        return;
      }
      if (this.state === CAM3D) return this.drawCam3d(ctx);
      if (this.state === SETUP) return this.drawSetup(ctx);

      if (this.view3dActive) this.drawWorld3d(ctx);
      else this.drawWorld2d(ctx);

      this.drawHud(ctx);
      this.drawBanner(ctx);
      if (this.slot) this.drawSlot(ctx);
      if (this.gameOver) this.drawGameOver(ctx);
    }

    /** Spielfeld-Gitter - normal oder als NGB-Raster-Overlay (gecacht). */
    drawGrid(ctx) {
      draw.rect(ctx, COL_BG, [0, 0, this.width, this.height]);
      const seq = NGB.gridSequence();
      if (!seq) {
        ctx.fillStyle = rgb(COL_GRID);
        for (let x = 0; x < this.cols * CELL; x += CELL) ctx.fillRect(x, 0, 1, this.rows * CELL);
        for (let y = 0; y < this.rows * CELL; y += CELL) ctx.fillRect(0, y, this.cols * CELL, 1);
        return;
      }
      const ps = Math.max(1, (PG.app && PG.app.pixelScale) || 1);
      const ck = this.cols + "|" + this.rows + "|" + ps + "|" + JSON.stringify(seq);
      if (!this.gridCache || this.gridCache.key !== ck) this.gridCache = { key: ck, canvas: this.buildGridOverlay(seq, ps) };
      ctx.drawImage(this.gridCache.canvas, 0, 0, this.cols * CELL, this.rows * CELL);
    }

    /** Spaltenname im Tabellen-Stil: a, b, ..., z, aa, ab, ... */
    static colLabel(i) {
      let name = "";
      i += 1;
      while (i > 0) {
        const r = (i - 1) % 26;
        i = Math.floor((i - 1) / 26);
        name = String.fromCharCode(97 + r) + name;
      }
      return name;
    }

    /** Rendert den Koordinaten-Wegweiser einmalig (Reihen-Nummern + Spalten-Buchstaben). */
    buildGridOverlay(seq, ps) {
      const n = seq.length;
      const w = this.cols * CELL, h = this.rows * CELL;
      const c = ui.makeCanvas(w * ps, h * ps);
      const g = c.getContext("2d");
      g.scale(ps, ps);
      g.fillStyle = rgb(COL_BG);
      g.fillRect(0, 0, w, h);
      // 1) Reihen-Bänder (sehr dezent zum Hintergrund gemischt)
      for (let gy = 0; gy < this.rows; gy++) {
        const band = seq[gy % n];
        g.fillStyle = rgb(band.map((v, i) => Math.floor((v + COL_BG[i] * 3) / 4)));
        g.fillRect(0, gy * CELL, w, CELL);
      }
      // 2) dezente Gitterlinien
      g.fillStyle = rgb(seq[0].map((v, i) => Math.floor((v + COL_BG[i]) / 2)));
      for (let gx = 0; gx <= this.cols; gx++) g.fillRect(gx * CELL, 0, 1, h);
      for (let gy = 0; gy <= this.rows; gy++) g.fillRect(0, gy * CELL, w, 1);
      // 3) Reihen-Nummern links + rechts
      for (let gy = 0; gy < this.rows; gy++) {
        const col = seq[gy % n].map((v) => Math.min(255, v + 110));
        const cy = gy * CELL + CELL / 2;
        ui.text(g, String(gy + 1), 3, cy, this.gridFont, col, "midleft", 170 / 255);
        ui.text(g, String(gy + 1), w - 3, cy, this.gridFont, col, "midright", 170 / 255);
      }
      // 4) Spalten-Buchstaben oben + unten
      for (let gx = 0; gx < this.cols; gx++) {
        const lab = SnakeGame.colLabel(gx);
        const cx = gx * CELL + CELL / 2;
        ui.text(g, lab, cx, 1, this.gridFont, [205, 212, 226], "midtop", 155 / 255);
        ui.text(g, lab, cx, h - 1, this.gridFont, [205, 212, 226], "midbottom", 155 / 255);
      }
      return c;
    }

    drawWorld2d(ctx) {
      this.drawGrid(ctx);

      // Hindernisse
      for (const [x, y] of this.obstacles.values()) {
        draw.rect(ctx, COL_WALL, [x * CELL, y * CELL, CELL, CELL]);
        draw.rect(ctx, [95, 104, 128], [x * CELL, y * CELL, CELL, CELL], 1);
      }

      // Portale
      for (const [a, b, col] of this.portalPairs) {
        for (const [px, py] of [a, b]) {
          const cx = px * CELL + CELL / 2, cy = py * CELL + CELL / 2;
          const r = CELL / 2 - 1 + Math.trunc(1.5 * Math.sin(this.animT * 6));
          draw.circle(ctx, col, [cx, cy], r, 3);
          draw.circle(ctx, col.map((c) => c >> 1), [cx, cy], Math.max(2, r - 4));
        }
      }

      // Futter - blendet mit leichtem Überschwingen ein
      for (const [k, cell] of this.foods) {
        const age = this.foodAnim.has(k) ? this.foodAnim.get(k) : FOOD_SPAWN_ANIM;
        const p = Math.min(1, age / FOOD_SPAWN_ANIM);
        this.drawApple(ctx, cell, easeOutBack(p));
        if (p < 1) this.drawFxRing(ctx, cell, Math.trunc(CELL * 0.28 + CELL * 0.5 * p), [255, 150, 150], Math.trunc(150 * (1 - p)));
      }
      // Goldapfel (pulsiert + blinkt, wenn er gleich verschwindet)
      if (this.golden) {
        const blink = this.goldenTimer > 1.5 || Math.trunc(this.animT * 8) % 2 === 0;
        if (blink) {
          const [gx, gy] = this.golden;
          const cx = gx * CELL + CELL / 2, cy = gy * CELL + CELL / 2;
          const r = CELL / 2 - 1 + Math.trunc(1.5 * Math.sin(this.animT * 8));
          draw.circle(ctx, COL_GOLD, [cx, cy], r);
          draw.circle(ctx, [255, 245, 200], [cx - 2, cy - 2], 2);
        }
      }

      this.drawSpecials(ctx);

      // Partikel
      for (const p of this.particles) {
        const a = PG.clamp(Math.trunc(255 * (p[4] / 0.55)), 0, 255);
        ctx.fillStyle = ui.col(p[5], a / 255);
        ctx.beginPath();
        ctx.arc(p[0], p[1], 3, 0, PG.TAU);
        ctx.fill();
      }

      this.snakes.forEach((sn, idx) => this.drawSnake(ctx, sn, idx));

      // Iss-Effekte ZULETZT (über dem Schlangenkopf)
      this.drawEatFx(ctx);
      this.drawFloatTexts(ctx);
    }

    /** Competitive-Spezialäpfel als pulsierende Edelsteine. */
    drawSpecials(ctx) {
      for (const info of this.specials.values()) {
        if (info.timer < 1.5 && Math.trunc(this.animT * 8) % 2 === 0) continue; // blinkt kurz vor dem Verschwinden
        const blue = info.type === "blue";
        const col = blue ? COL_BLUE : COL_PURPLE;
        const inner = col.map((c) => Math.min(255, c + 70));
        const cx = info.cell[0] * CELL + CELL / 2;
        const cy = info.cell[1] * CELL + CELL / 2;
        const r = CELL / 2 - 2 + Math.trunc(1.5 * Math.sin(this.animT * 6 + info.cell[0]));
        const pts = [[cx, cy - r], [cx + r, cy], [cx, cy + r], [cx - r, cy]];
        draw.polygon(ctx, col, pts);
        draw.polygon(ctx, inner, pts, 1);
        ui.text(ctx, blue ? "$" : "±", cx, cy, this.tiny, [20, 20, 30], "center");
      }
    }

    drawFloatTexts(ctx) {
      for (const ft of this.floatTexts) {
        ui.text(ctx, ft.text, Math.trunc(ft.x), Math.trunc(ft.y), this.small, ft.color, "center", PG.clamp(ft.t / 1.1, 0, 1));
      }
    }

    /** Große Einblendung oben mittig (Sichtbarkeit/Größe/Deckkraft aus NGB). */
    drawBanner(ctx) {
      const b = this.banner;
      if (!b) return;
      const cfg = NGB.getBanner();
      if (!cfg.on) return;
      const appear = Math.min(1, (BANNER_TIME - b.t) / 0.18);
      const fade = PG.clamp(b.t / 0.5, 0, 1);
      const a = Math.min(appear, fade) * cfg.opacity;
      if (a <= 0) return;
      const pad = 18;
      const tw = Math.max(this.bigFont.width(b.text), b.sub ? this.small.width(b.sub) : 0);
      const th = this.bigFont.height + (b.sub ? this.small.height + 4 : 0);
      const pw = tw + pad * 2, ph = th + pad;
      const scale = Math.max(0.01, (0.82 + 0.18 * appear) * cfg.size);
      const top = 44 + Math.trunc(-14 * (1 - appear));
      ctx.save();
      ctx.globalAlpha = a;
      ctx.translate(this.width / 2, top);
      ctx.scale(scale, scale);
      ctx.translate(-pw / 2, 0);
      draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 215], [0, 0, pw, ph], 0, 12);
      draw.rect(ctx, b.color, [0, 0, pw, ph], 2, 12);
      let y = pad / 2;
      if (b.sub) {
        ui.text(ctx, b.sub, pw / 2, 5, this.small, ui.TEXT_DIM, "midtop");
        y = 5 + this.small.height + 2;
      }
      ui.text(ctx, b.text, pw / 2, y, this.bigFont, b.color, "midtop");
      ctx.restore();
    }

    /** Apfel skaliert um sein Zentrum; klein runder, voll ein abgerundetes Quadrat. */
    drawApple(ctx, cell, scale, alpha = 255) {
      if (scale <= 0.03) return;
      const size = (CELL - 4) * scale;
      const cx = cell[0] * CELL + CELL / 2;
      const cy = cell[1] * CELL + CELL / 2;
      const s = Math.trunc(size);
      const rad = Math.trunc(Math.min(size / 2, 6 + (size / 2) * (1 - scale)));
      if (alpha >= 255) {
        draw.rect(ctx, COL_FOOD, [Math.trunc(cx) - Math.floor(s / 2), Math.trunc(cy) - Math.floor(s / 2), s, s], 0, rad);
        const hl = Math.max(1, Math.trunc(2.2 * scale));
        draw.circle(ctx, [255, 170, 170], [Math.trunc(cx - size * 0.14), Math.trunc(cy - size * 0.14)], hl);
      } else {
        const d = s + 2;
        draw.rect(ctx, [COL_FOOD[0], COL_FOOD[1], COL_FOOD[2], alpha], [Math.trunc(cx - d / 2) + 1, Math.trunc(cy - d / 2) + 1, s, s], 0, rad);
      }
    }

    /** Ausdehnender, verblassender Ring um eine Zelle (Spawn-/Iss-Effekt). */
    drawFxRing(ctx, cell, r, color, a) {
      if (a <= 4 || r <= 0) return;
      const cx = cell[0] * CELL + CELL / 2;
      const cy = cell[1] * CELL + CELL / 2;
      draw.circle(ctx, [color[0], color[1], color[2], a], [cx, cy], r, 2);
    }

    drawEatFx(ctx) {
      for (const e of this.eatFx) {
        const p = 1 - Math.max(0, e.t / FOOD_EAT_ANIM);
        this.drawApple(ctx, e.cell, Math.max(0, (1 - p) * 0.9), Math.trunc(210 * (1 - p)));
        this.drawFxRing(ctx, e.cell, Math.trunc(CELL * 0.32 + CELL * 0.5 * p), COL_FOOD, Math.trunc(150 * (1 - p)));
      }
    }

    /** Kopffarbe (NGB-Personalisierung). */
    headColor() {
      return NGB.headColor();
    }

    drawSnake(ctx, sn, idx) {
      let koerper = SNAKE_COLORS[0][0];
      let kopf = this.headColor();
      if (!sn.alive) {
        koerper = koerper.map((c) => c >> 1);
        kopf = koerper;
      }
      const n = sn.body.length;
      // Boost-Glow um den Kopf
      if (sn.alive && sn.boostOn && sn.stamina > 0) {
        const [hx, hy] = sn.body[n - 1];
        const pulse = 120 + Math.trunc(60 * Math.sin(this.animT * 14));
        draw.circle(ctx, [BOOST_GLOW[0][0], BOOST_GLOW[0][1], BOOST_GLOW[0][2], pulse], [hx * CELL + CELL / 2, hy * CELL + CELL / 2], CELL);
      }
      sn.body.forEach(([x, y], i) => {
        let farbe;
        if (i === n - 1) farbe = kopf;
        else {
          // sanfter Verlauf vom Kopf (hell) zum Schwanz (dunkler)
          const tt = i / Math.max(1, n - 1);
          farbe = kopf.map((k, j) => Math.trunc(k + (koerper[j] - k) * (1 - tt * 0.5)));
        }
        draw.rect(ctx, farbe, [x * CELL + 1, y * CELL + 1, CELL - 2, CELL - 2], 0, 5);
      });
      if (n) this.drawEyes(ctx, sn);
    }

    drawEyes(ctx, sn) {
      const [hx, hy] = sn.body[sn.body.length - 1];
      const [dx, dy] = sn.direction;
      const cx = hx * CELL + CELL / 2, cy = hy * CELL + CELL / 2;
      const px = -dy, py = dx; // senkrecht zur Blickrichtung
      for (const sign of [-1, 1]) {
        const ex = cx + dx * 3 + px * sign * 4;
        const ey = cy + dy * 3 + py * sign * 4;
        draw.circle(ctx, [250, 250, 250], [Math.trunc(ex), Math.trunc(ey)], 3);
        draw.circle(ctx, [20, 20, 30], [Math.trunc(ex + dx * 1.5), Math.trunc(ey + dy * 1.5)], 1);
      }
    }

    // ----- 3D-Renderer --------------------------------------------------------------------
    // Punkte werden in den Kameraraum transformiert (rechts/oben/vorwärts), an der
    // Nahebene geclippt und perspektivisch projiziert. Flächen nach Tiefe sortiert
    // zeichnen (Painter-Algorithmus, ferne zuerst).
    drawWorld3d(ctx) {
      // Nach dem Game Over ruft die App update() nicht mehr auf -> hier weiter animieren.
      if (this.gameOver) this.tickGameoverAnim();

      this.scx = this.width / 2;
      this.scy = this.height * 0.5;
      if (this.shake > 0) {
        const amp = 16 * this.shake * (this.camSmooth ? 0.5 : 1.0);
        this.scx += PG.rand.uniform(-amp, amp);
        this.scy += PG.rand.uniform(-amp, amp);
      }
      this.f = this.height * this.fovMul;
      this.basis = this.viewBasis();

      this.drawSky(ctx);
      this.drawStars3d(ctx);
      this.drawFloor3d(ctx);

      const items = []; // {d, pts, col, outline}
      this.addBorderWalls(items);
      for (const [ox, oz] of this.obstacles.values()) this.addBox(items, ox, oz, 0.94, 1.05, COL_WALL, [95, 104, 128]);
      for (const f of this.foods.values()) {
        const r = 0.3 + 0.05 * Math.sin(this.animT * 5 + f[0] + f[1]);
        this.addOcta(items, f, r, COL_FOOD);
      }
      if (this.golden && (this.goldenTimer > 1.5 || Math.trunc(this.animT * 8) % 2 === 0)) {
        this.addOcta(items, this.golden, 0.38 + 0.06 * Math.sin(this.animT * 8), COL_GOLD);
      }
      this.snakes.forEach((sn, idx) => this.addSnake3d(items, sn, idx));

      items.sort((a, b) => b.d - a.d);
      ctx.save();
      ctx.lineJoin = "round";
      ctx.lineWidth = 1;
      for (const it of items) {
        const pts = it.pts;
        ctx.beginPath();
        ctx.moveTo(pts[0][0], pts[0][1]);
        for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1]);
        ctx.closePath();
        const cs = rgb(it.col);
        ctx.fillStyle = cs;
        ctx.fill();
        ctx.strokeStyle = it.outline ? rgb(it.outline) : cs; // ohne Kontur: Kanten schließen (keine Spalte)
        ctx.stroke();
      }
      ctx.restore();

      this.drawEyes3d(ctx);
      this.drawBoostGlow3d(ctx);
      this.drawParticles3dPass(ctx);
    }

    /** Animiert Orbit-Kamera/Partikel nach dem Game Over weiter (echte Zeit). */
    tickGameoverAnim() {
      const now = performance.now() / 1000;
      const last = this.goLast != null ? this.goLast : now;
      this.goLast = now;
      const dt = PG.clamp(now - last, 0, 0.05);
      if (dt <= 0) return;
      this.animT += dt;
      this.updateParticles3d(dt);
      if (this.shake > 0) this.shake = Math.max(0, this.shake - dt * 1.6);
      this.updateCamera(dt);
    }

    // ----- Kamera / Projektion -----------------------------------------------------------
    /** Basisvektoren der Kamera: [rechts, oben, vorwärts]. */
    viewBasis() {
      const [cx, cy, cz] = this.camPos;
      const fx0 = this.camLook[0] - cx, fy0 = this.camLook[1] - cy, fz0 = this.camLook[2] - cz;
      const fl = Math.sqrt(fx0 * fx0 + fy0 * fy0 + fz0 * fz0) || 1;
      const f = [fx0 / fl, fy0 / fl, fz0 / fl];
      const rl = Math.hypot(f[2], f[0]) || 1;
      const r = [-f[2] / rl, 0, f[0] / rl];
      const u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
      return [r, u, f];
    }

    /** Weltpunkt -> Kameraraum (x rechts, y oben, z Tiefe). */
    toCam(px, py, pz) {
      const [r, u, f] = this.basis;
      const dx = px - this.camPos[0], dy = py - this.camPos[1], dz = pz - this.camPos[2];
      return [dx * r[0] + dz * r[2], dx * u[0] + dy * u[1] + dz * u[2], dx * f[0] + dy * f[1] + dz * f[2]];
    }

    /** Kameraraum -> Bildschirmpixel (perspektivisch). */
    proj(c) {
      const k = this.f / c[2];
      return [this.scx + c[0] * k, this.scy - c[1] * k];
    }

    /** Schneidet ein Polygon (Kameraraum) an der Nahebene z = NEAR. */
    static clipNear(pts) {
      const out = [];
      const n = pts.length;
      for (let i = 0; i < n; i++) {
        const a = pts[i], b = pts[(i + 1) % n];
        const aIn = a[2] >= NEAR, bIn = b[2] >= NEAR;
        if (aIn) out.push(a);
        if (aIn !== bIn) {
          const tt = (NEAR - a[2]) / (b[2] - a[2]);
          out.push([a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR]);
        }
      }
      return out;
    }

    static shadeCol(col, k) {
      return [Math.min(255, Math.trunc(col[0] * k)), Math.min(255, Math.trunc(col[1] * k)), Math.min(255, Math.trunc(col[2] * k))];
    }

    /** Blendet eine Farbe mit der Entfernung in den Nebel aus. */
    static fogColor(col, depth) {
      let tt = (depth - FOG_START) / (FOG_END - FOG_START);
      if (tt <= 0) return col;
      tt = Math.min(1, tt);
      return [
        Math.trunc(col[0] + (COL_FOG[0] - col[0]) * tt),
        Math.trunc(col[1] + (COL_FOG[1] - col[1]) * tt),
        Math.trunc(col[2] + (COL_FOG[2] - col[2]) * tt),
      ];
    }

    /** Transformiert, clippt und projiziert eine Fläche in die Zeichenliste. */
    addPoly(items, worldPts, color, shade = 1, outline = null) {
      let cs = worldPts.map((p) => this.toCam(p[0], p[1], p[2]));
      if (cs.every((c) => c[2] < NEAR)) return;
      cs = SnakeGame.clipNear(cs);
      if (cs.length < 3) return;
      let depth = 0;
      for (const c of cs) depth += c[2];
      depth /= cs.length;
      if (depth > FOG_END + 6) return;
      const pts = cs.map((c) => this.proj(c));
      const W = this.width, H = this.height;
      if (pts.every((p) => p[0] < -40) || pts.every((p) => p[0] > W + 40) || pts.every((p) => p[1] < -40) || pts.every((p) => p[1] > H + 40)) return;
      const col = SnakeGame.fogColor(SnakeGame.shadeCol(color, shade), depth);
      // Konturen nur in der Nähe (im Nebel wirken sie unruhig)
      items.push({ d: depth, pts, col, outline: depth < FOG_START + 4 ? outline : null });
    }

    // ----- Szenen-Bausteine ---------------------------------------------------------------
    /** Quader auf dem Boden der Zelle (gx, gz); nur sichtbare Seiten. */
    addBox(items, gx, gz, w, h, color, outline = null) {
      const cx = gx + 0.5, cz = gz + 0.5;
      const x0 = cx - w / 2, x1 = cx + w / 2;
      const z0 = cz - w / 2, z1 = cz + w / 2;
      const [px, py, pz] = this.camPos;
      if (py > h) this.addPoly(items, [[x0, h, z0], [x1, h, z0], [x1, h, z1], [x0, h, z1]], color, 1.0, outline);
      if (px < x0) this.addPoly(items, [[x0, 0, z0], [x0, 0, z1], [x0, h, z1], [x0, h, z0]], color, 0.62, outline);
      else if (px > x1) this.addPoly(items, [[x1, 0, z0], [x1, 0, z1], [x1, h, z1], [x1, h, z0]], color, 0.62, outline);
      if (pz < z0) this.addPoly(items, [[x0, 0, z0], [x1, 0, z0], [x1, h, z0], [x0, h, z0]], color, 0.76, outline);
      else if (pz > z1) this.addPoly(items, [[x0, 0, z1], [x1, 0, z1], [x1, h, z1], [x0, h, z1]], color, 0.76, outline);
    }

    /** Rotierender Oktaeder-Kristall (Futter / Goldapfel). */
    addOcta(items, cell, r, color, y = 0.45) {
      const cx = cell[0] + 0.5, cz = cell[1] + 0.5;
      const top = [cx, y + r, cz];
      const bot = [cx, Math.max(0.04, y - r), cz];
      const a0 = this.animT * 2.4;
      const eq = [0, 1, 2, 3].map((i) => [cx + Math.cos(a0 + (i * Math.PI) / 2) * r, y, cz + Math.sin(a0 + (i * Math.PI) / 2) * r]);
      const [px, py, pz] = this.camPos;
      const lx = 0.42, ly = -0.82, lz = 0.39; // Lichtrichtung (von oben)
      for (let i = 0; i < 4; i++) {
        for (const tri of [[top, eq[i], eq[(i + 1) % 4]], [bot, eq[(i + 1) % 4], eq[i]]]) {
          const [a, b, c] = tri;
          const ux = b[0] - a[0], uy = b[1] - a[1], uz = b[2] - a[2];
          const vx = c[0] - a[0], vy = c[1] - a[1], vz = c[2] - a[2];
          let nx = uy * vz - uz * vy, ny = uz * vx - ux * vz, nz = ux * vy - uy * vx;
          const fcx = (a[0] + b[0] + c[0]) / 3, fcy = (a[1] + b[1] + c[1]) / 3, fcz = (a[2] + b[2] + c[2]) / 3;
          if (nx * (fcx - cx) + ny * (fcy - y) + nz * (fcz - cz) < 0) {
            nx = -nx; ny = -ny; nz = -nz; // Normale nach außen
          }
          if (nx * (fcx - px) + ny * (fcy - py) + nz * (fcz - pz) >= 0) continue; // Rückseite
          const nl = Math.sqrt(nx * nx + ny * ny + nz * nz) || 1;
          const lit = Math.max(0, -(nx * lx + ny * ly + nz * lz) / nl);
          this.addPoly(items, tri, color, 0.55 + 0.5 * lit);
        }
      }
    }

    /** Niedrige Bande rund um das Spielfeld (leicht gestreift). */
    addBorderWalls(items) {
      const h = 0.55, R = this.rows, C = this.cols;
      for (let gx = 0; gx < C; gx++) {
        const sh = gx % 2 === 0 ? 0.9 : 0.78;
        this.addPoly(items, [[gx, 0, 0], [gx + 1, 0, 0], [gx + 1, h, 0], [gx, h, 0]], COL_BORDER, sh);
        this.addPoly(items, [[gx, 0, R], [gx + 1, 0, R], [gx + 1, h, R], [gx, h, R]], COL_BORDER, sh);
      }
      for (let gz = 0; gz < R; gz++) {
        const sh = gz % 2 === 0 ? 0.84 : 0.7;
        this.addPoly(items, [[0, 0, gz], [0, 0, gz + 1], [0, h, gz + 1], [0, h, gz]], COL_BORDER, sh);
        this.addPoly(items, [[C, 0, gz], [C, 0, gz + 1], [C, h, gz + 1], [C, h, gz]], COL_BORDER, sh);
      }
    }

    /** Die Schlange als Kette aus Quadern (Kopf größer und heller). */
    addSnake3d(items, sn, idx) {
      let colBody = SNAKE_COLORS[0][0];
      let colHead = this.headColor();
      if (!sn.alive) {
        colBody = colBody.map((c) => c >> 1);
        colHead = colBody;
      }
      const cells = this.interpCells(sn);
      const n = cells.length;
      cells.forEach(([gx, gz], k) => {
        let farbe, w, h;
        if (k === n - 1) {
          farbe = colHead; w = 0.92; h = 0.78;
        } else {
          const tt = k / Math.max(1, n - 1);
          farbe = colHead.map((kc, j) => Math.trunc(kc + (colBody[j] - kc) * (1 - tt * 0.5)));
          w = 0.8; h = 0.58;
        }
        this.addBox(items, gx, gz, w, h, farbe, farbe.map((c) => Math.trunc(c / 3)));
      });
    }

    // ----- Hintergrund / Boden --------------------------------------------------------------
    /** Vertikaler Himmels-Gradient (gecacht als 1 Pixel breiter Streifen). */
    drawSky(ctx) {
      if (!this.skyCache) {
        const H = this.height;
        const c = ui.makeCanvas(1, H);
        const g = c.getContext("2d");
        const hor = Math.trunc(H * 0.4), haze = Math.trunc(H * 0.55);
        for (let y = 0; y < H; y++) {
          let col;
          if (y < hor) {
            const tt = y / Math.max(1, hor);
            col = COL_SKY_TOP.map((a, i) => Math.trunc(a + (COL_SKY_HOR[i] - a) * tt));
          } else if (y < haze) {
            const tt = (y - hor) / Math.max(1, haze - hor);
            col = COL_SKY_HOR.map((a, i) => Math.trunc(a + (COL_FOG[i] - a) * tt));
          } else col = COL_FOG;
          g.fillStyle = rgb(col);
          g.fillRect(0, y, 1, 1);
        }
        this.skyCache = c;
      }
      ctx.save();
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(this.skyCache, 0, 0, this.width, this.height);
      ctx.restore();
    }

    /** Funkelnde Sterne, die beim Drehen der Kamera mitwandern (Parallaxe). */
    drawStars3d(ctx) {
      const f = this.basis[2];
      const yaw = Math.atan2(f[0], f[2]);
      const hor = this.height * 0.4;
      for (const [az, hf, size, ph] of this.stars) {
        const d = PG.mod(az - yaw + Math.PI, PG.TAU) - Math.PI;
        if (Math.abs(d) > 0.7) continue;
        const sx = this.scx + Math.tan(d) * this.f;
        if (sx < -4 || sx > this.width + 4) continue;
        const sy = hor * (0.08 + 0.84 * hf);
        const tw = 0.55 + 0.45 * Math.sin(this.animT * 1.7 + ph);
        const c = Math.trunc(120 + 110 * tw);
        draw.circle(ctx, [c, c, Math.min(255, c + 25)], [Math.trunc(sx), Math.trunc(sy)], size);
      }
    }

    /** Schachbrett-Boden; Zellecken pro Frame nur einmal transformiert. */
    drawFloor3d(ctx) {
      const C = this.cols, R = this.rows;
      const corners = new Array((C + 1) * (R + 1));
      const corner = (gx, gz) => {
        const i = gz * (C + 1) + gx;
        let v = corners[i];
        if (!v) v = corners[i] = this.toCam(gx, 0, gz);
        return v;
      };
      const lim = FOG_END + 1;
      ctx.save();
      ctx.lineWidth = 1;
      ctx.lineJoin = "round";
      for (let gz = 0; gz < R; gz++) {
        for (let gx = 0; gx < C; gx++) {
          const c00 = corner(gx, gz);
          // grobes Cull: zu weit weg, hinter der Kamera oder seitlich draußen
          if (c00[2] > lim || c00[2] < -1.8) continue;
          if (Math.abs(c00[0]) > c00[2] * 1.7 + 2.5) continue;
          const quad = [c00, corner(gx + 1, gz), corner(gx + 1, gz + 1), corner(gx, gz + 1)];
          if (quad.every((c) => c[2] < NEAR)) continue;
          const poly = SnakeGame.clipNear(quad);
          if (poly.length < 3) continue;
          let depth = 0;
          for (const c of poly) depth += c[2];
          depth /= poly.length;
          const col = ((gx + gz) & 1) === 0 ? COL_TILE_A : COL_TILE_B;
          const cs = rgb(SnakeGame.fogColor(col, depth));
          ctx.beginPath();
          const p0 = this.proj(poly[0]);
          ctx.moveTo(p0[0], p0[1]);
          for (let i = 1; i < poly.length; i++) {
            const p = this.proj(poly[i]);
            ctx.lineTo(p[0], p[1]);
          }
          ctx.closePath();
          ctx.fillStyle = cs;
          ctx.fill();
          ctx.strokeStyle = cs; // schließt Antialiasing-Spalte zwischen den Kacheln
          ctx.stroke();
        }
      }
      ctx.restore();
    }

    // ----- Details / Effekte -------------------------------------------------------------------
    /** Augen auf der Stirnseite des Kopfes (nur wenn sie zur Kamera zeigen). */
    drawEyes3d(ctx) {
      const sn = this.snakes[0];
      if (!sn.alive) return;
      const cells = this.interpCells(sn);
      const [hx, hz] = cells[cells.length - 1];
      const [dx, dz] = sn.direction;
      const fx = hx + 0.5 + dx * 0.47, fz = hz + 0.5 + dz * 0.47;
      if ((fx - this.camPos[0]) * dx + (fz - this.camPos[2]) * dz > 0) return; // Gesicht abgewandt
      const px = -dz, pz = dx;
      for (const sign of [-1, 1]) {
        const e = [fx + px * sign * 0.17, 0.52, fz + pz * sign * 0.17];
        const c = this.toCam(e[0], e[1], e[2]);
        if (c[2] < NEAR) continue;
        const [sx, sy] = this.proj(c);
        const rr = Math.max(1, Math.trunc((this.f * 0.05) / c[2]));
        draw.circle(ctx, [250, 250, 250], [Math.trunc(sx), Math.trunc(sy)], rr);
        const pupil = this.toCam(e[0] + dx * 0.06, 0.5, e[2] + dz * 0.06);
        if (pupil[2] >= NEAR) {
          const pxy = this.proj(pupil);
          draw.circle(ctx, [20, 20, 30], [Math.trunc(pxy[0]), Math.trunc(pxy[1])], Math.max(1, rr >> 1));
        }
      }
    }

    /** Pulsierender Glow über dem Kopf, solange der Boost läuft. */
    drawBoostGlow3d(ctx) {
      const sn = this.snakes[0];
      if (!(sn.alive && sn.boostOn && sn.stamina > 0)) return;
      const cells = this.interpCells(sn);
      const [hx, hz] = cells[cells.length - 1];
      const c = this.toCam(hx + 0.5, 0.42, hz + 0.5);
      if (c[2] < NEAR) return;
      const [sx, sy] = this.proj(c);
      const r = Math.max(8, Math.trunc((this.f * 0.62) / c[2]));
      const pulse = 90 + Math.trunc(50 * Math.sin(this.animT * 14));
      draw.circle(ctx, [BOOST_GLOW[0][0], BOOST_GLOW[0][1], BOOST_GLOW[0][2], pulse], [sx, sy], r);
    }

    drawParticles3dPass(ctx) {
      for (const p of this.particles3d) {
        const c = this.toCam(p[0], p[1], p[2]);
        if (c[2] < NEAR) continue;
        const [sx, sy] = this.proj(c);
        if (sx < -8 || sx > this.width + 8 || sy < -8 || sy > this.height + 8) continue;
        const r = Math.max(1, Math.trunc((this.f * 0.045) / c[2]));
        const tt = PG.clamp(p[6] / 0.7, 0, 1);
        const col = p[7].map((pc, i) => Math.trunc(COL_FOG[i] + (pc - COL_FOG[i]) * tt));
        draw.circle(ctx, SnakeGame.fogColor(col, c[2]), [Math.trunc(sx), Math.trunc(sy)], r);
      }
    }

    // ----- HUD ----------------------------------------------------------------------------
    drawHud(ctx) {
      const W = this.width, H = this.height;
      // WLS-Anzeige (durch die Wände?) bzw. 3D-Badge
      if (this.view3dActive) ui.text(ctx, t("snake.view_3d"), W / 2, 6, this.font, ui.ACCENT, "midtop");
      else ui.text(ctx, t("snake.wls"), W / 2, 6, this.font, this.wrapActive ? ui.GREEN : ui.TEXT_FAINT, "midtop");
      ui.text(ctx, t("snake.mode." + this.modeKey), W / 2, 30, this.small, ui.ACCENT, "midtop");
      if (this.view3dActive && !this.gameOver && this.runT < 6.0) {
        ui.text(ctx, t("snake.steer_hint"), W / 2, 50, this.tiny, ui.TEXT_DIM, "midtop");
      }

      // Zeitangriff-Uhr (Monospace, damit die tickenden Ziffern nicht zappeln)
      if (this.modeKey === "timed") {
        const col = this.timeLeft <= 10 ? ui.RED : ui.TEXT;
        const tt = this.timeLeft.toFixed(1).padStart(4, "0");
        ui.text(ctx, t("snake.time", { t: tt }), W / 2, 50, ui.font(22, false, true), col, "midtop");
      }

      ui.text(ctx, t("common.points", { score: this.score }), 10, 8, this.font, ui.TEXT);
      if (this.competitive) this.drawCompHud(ctx);
      else {
        ui.text(ctx, t("snake.apples_bank", { total: this.applesTotal, bank: this.applesBank }), 10, 34, this.small, COL_FOOD);
        if (this.prestige > 0) {
          const blocks = PRESTIGE.blocksPerApple(this.prestige);
          ui.text(ctx, t("snake.prestige_info", { roman: PRESTIGE.roman(this.prestige), blocks }), 10, 54, this.small, ui.GOLD);
        }
      }

      ui.text(ctx, t("snake.best", { hs: this.best }), W - 10, 10, this.small, ui.TEXT_DIM, "topright");

      this.drawBoostBar(ctx, 10, H - 22, 180, this.snakes[0], 0);

      if (!this.gameOver && this.competitive) this.drawCompFooter(ctx);
      else if (!this.gameOver && this.modeKey !== "timed") this.drawNextPrestige(ctx);
    }

    /** Competitive-Kopfzeile: gesammelte Äpfel, Level, Größe, Slot-Bonus. */
    drawCompHud(ctx) {
      const lvl = this.compLevel;
      ui.text(ctx, t("snake.comp.stats", { apples: this.applesTotal, level: lvl, mult: COMP.scoreMultiplier(lvl) }), 10, 34, this.small, COL_FOOD);
      // Größe als Kommazahl (fürs Gambling weiter nutzbar), oben links.
      const gtxt = this.snakeSize(this.snakes[0]).toFixed(2).replace(/0+$/, "").replace(/\.$/, "");
      ui.text(ctx, t("snake.comp.size", { size: gtxt }), 10, 54, this.small, ui.GOLD);
      if (this.spawnBonusT > 0 && this.spawnBonus > 0) {
        ui.text(ctx, t("snake.comp.bonus", { n: this.spawnBonus, t: this.spawnBonusT }), 10, 74, this.small, COL_BLUE);
      }
      if (this.hardcore) {
        const puls = 0.5 + 0.5 * Math.sin(ui.ticks() * 0.006);
        const col = [255, Math.trunc(70 + 80 * puls), Math.trunc(70 + 80 * puls)];
        const label = t("snake.hardcore_tag");
        const tw = this.small.width(label), th = this.small.height;
        const tr = new PG.Rect(Math.floor(this.width / 2 - tw / 2), 48, tw, th);
        const box = tr.inflate(16, 8);
        draw.rect(ctx, [COL_HARDCORE[0], COL_HARDCORE[1], COL_HARDCORE[2], Math.trunc(40 + 60 * puls)], box, 0, 8);
        draw.rect(ctx, COL_HARDCORE, box, 1, 8);
        ui.text(ctx, label, tr.x, tr.y, this.small, col);
      }
    }

    /** Fortschrittsbalken bis zum nächsten Competitive-Level (unten mittig). */
    drawCompFooter(ctx) {
      const W = this.width, H = this.height;
      const step = COMP.nextStep(this.applesTotal);
      if (!step) {
        ui.text(ctx, t("snake.comp.max", { level: this.compLevel }), W / 2, H - 8, this.small, ui.GOLD, "midbottom");
        return;
      }
      const [have, need] = step;
      const w = 220, y = H - 20;
      const x = Math.floor(W / 2) - w / 2;
      draw.rect(ctx, ui.PANEL, [x, y, w, 12], 0, 5);
      const fw = Math.trunc((w - 2) * PG.clamp(have / Math.max(1, need), 0, 1));
      draw.rect(ctx, ui.GOLD, [x + 1, y + 1, fw, 10], 0, 5);
      ui.text(ctx, t("snake.comp.next", { level: this.compLevel + 1, have, need }), W / 2, y - 2, this.tiny, ui.TEXT_DIM, "midbottom");
    }

    // ----- Slot-Machine zeichnen ----------------------------------------------------------
    drawSlot(ctx) {
      const sl = this.slot;
      ctx.fillStyle = "rgba(0,0,0,0.667)";
      ctx.fillRect(0, 0, this.width, this.height);

      const pw = Math.min(360, this.width - 40), ph = 260;
      const panel = new PG.Rect(Math.floor((this.width - pw) / 2), Math.floor((this.height - ph) / 2), pw, ph);
      draw.rect(ctx, ui.PANEL, panel, 0, 14);
      draw.rect(ctx, COL_BLUE, panel, 3, 14);

      ui.text(ctx, t("snake.slot.title"), panel.centerx, panel.top + 12, this.font, COL_BLUE, "midtop");
      ui.text(ctx, t("snake.slot.stake", { n: sl.stake }), panel.centerx, panel.top + 40, this.small, ui.TEXT_DIM, "midtop");

      const rw = 78, rh = 96, gap = 12;
      const total = rw * 3 + gap * 2;
      const x0 = panel.centerx - Math.floor(total / 2);
      const ry = panel.top + 74;
      const cellH = Math.floor(rh / 3);
      const reel = COMP.REEL;
      for (let k = 0; k < 3; k++) {
        const win = new PG.Rect(x0 + k * (rw + gap), ry, rw, rh);
        draw.rect(ctx, [16, 18, 30], win, 0, 8);
        ctx.save();
        ctx.beginPath();
        ctx.rect(win.x, win.y, win.w, win.h);
        ctx.clip();
        if (sl.t >= sl.stop[k]) {
          // Walze steht
          const anchor = reel.indexOf(sl.reels[k]);
          for (const row of [-1, 0, 1]) {
            const sym = reel[PG.mod(anchor + row, reel.length)];
            this.drawSlotSymbol(ctx, sym, win.centerx, win.centery + row * cellH, Math.floor(cellH / 2) - 4);
          }
        } else {
          // Walze dreht
          const base = Math.trunc(sl.t * SLOT_SPIN_SPEED) + k * 5;
          const scroll = PG.mod(sl.t * SLOT_SPIN_SPEED, 1) * cellH;
          for (const row of [-1, 0, 1, 2]) {
            const sym = reel[PG.mod(base + row, reel.length)];
            this.drawSlotSymbol(ctx, sym, win.centerx, Math.trunc(win.centery + row * cellH - scroll), Math.floor(cellH / 2) - 4);
          }
        }
        ctx.restore();
        draw.rect(ctx, ui.BORDER, win, 2, 8);
      }
      draw.line(ctx, COL_BLUE, [x0 - 4, ry + rh / 2 + 0.5], [x0 + total + 4, ry + rh / 2 + 0.5], 1);

      if (sl.applied) {
        const res = t("snake.slot." + sl.result);
        const col = sl.result === "jackpot" ? ui.GOLD : sl.result === "pair" ? ui.GREEN : ui.TEXT_DIM;
        ui.text(ctx, res + "  x" + sl.mult, panel.centerx, panel.bottom - 14, this.font, col, "midbottom");
      }
    }

    /** Zeichnet ein Slot-Symbol als kleines Icon. */
    drawSlotSymbol(ctx, k, cx, cy, r) {
      const col = COMP.SLOT_SYMBOLS[k];
      if (k === "seven") {
        ui.text(ctx, "7", cx, cy, this.font, col, "center");
      } else if (k === "gem") {
        const pts = [[cx, cy - r], [cx + r, cy], [cx, cy + r], [cx - r, cy]];
        draw.polygon(ctx, col, pts);
        draw.polygon(ctx, [255, 255, 255], pts, 1);
      } else if (k === "bell") {
        draw.circle(ctx, col, [cx, cy - Math.floor(r / 6)], Math.trunc(r * 0.8));
        draw.rect(ctx, col, [cx - r, cy + Math.floor(r / 3), r * 2, Math.max(2, Math.floor(r / 3))], 0, 2);
        draw.circle(ctx, [40, 40, 30], [cx, cy + Math.floor(r / 2)], Math.max(1, Math.floor(r / 5)));
      } else if (k === "apple") {
        draw.circle(ctx, col, [cx, cy], Math.trunc(r * 0.85));
        draw.circle(ctx, [255, 180, 180], [cx - Math.floor(r / 4), cy - Math.floor(r / 4)], Math.max(1, Math.floor(r / 6)));
      } else {
        // Kirsche
        for (const off of [-Math.floor(r / 2), Math.floor(r / 2)]) draw.circle(ctx, col, [cx + off, cy + Math.floor(r / 3)], Math.trunc(r * 0.45));
        draw.line(ctx, [120, 200, 120], [cx + 0.5, cy - r], [cx + 0.5, cy + Math.floor(r / 3)], 1);
      }
    }

    drawBoostBar(ctx, x, y, w, sn, idx) {
      draw.rect(ctx, ui.PANEL, [x, y, w, 14], 0, 5);
      const fw = Math.trunc((w - 2) * PG.clamp(sn.stamina, 0, 1));
      let col;
      if (sn.boostOn && sn.stamina > 0) col = ui.GOLD; // aktiv = gold
      else if (sn.stamina >= BOOST_MIN_START) col = BOOST_GLOW[0]; // bereit
      else col = [120, 90, 90]; // zu leer zum Starten
      draw.rect(ctx, col, [x + 1, y + 1, fw, 12], 0, 5);
      ui.text(ctx, t("snake.boost"), x + 6, y, this.tiny, [15, 15, 20]);
    }

    drawNextPrestige(ctx) {
      const [req, ok] = this.canPrestige();
      let txt, col;
      if (!req) {
        txt = t("snake.max_prestige", { roman: PRESTIGE.roman(this.prestige) });
        col = ui.GOLD;
      } else {
        txt = t("snake.next_prestige", { roman: req.roman, apples: req.apples, length: req.length });
        if (ok) {
          txt += t("snake.press_p");
          col = ui.GOLD;
        } else col = ui.TEXT_DIM;
      }
      ui.text(ctx, txt, this.width / 2, this.height - 8, this.small, col, "midbottom");
    }

    drawGameOver(ctx) {
      ctx.fillStyle = "rgba(0,0,0,0.588)";
      ctx.fillRect(0, 0, this.width, this.height);
      const titel = this.modeKey === "timed" && this.snakes[0].alive ? t("snake.time_up") : t("common.game_over");
      this.drawCenterText(ctx, titel, this.bigFont, ui.RED, -60);
      this.drawCenterText(ctx, t("snake.apples_collected", { n: this.applesTotal }), this.font, COL_FOOD, -14);
      if (this.competitive) this.drawCenterText(ctx, t("snake.comp.result", { level: this.compLevel }), this.font, ui.GOLD, 16);
      else if (this.prestige > 0) this.drawCenterText(ctx, t("snake.prestige", { roman: PRESTIGE.roman(this.prestige) }), this.font, ui.GOLD, 16);
      this.drawCenterText(ctx, t("common.enter_restart"), this.font, ui.TEXT, 46);
    }

    // ----- Setup zeichnen ---------------------------------------------------------------------
    drawSetup(ctx) {
      const W = this.width, H = this.height;
      ui.drawBackground(ctx, W, H);

      ui.text(ctx, "SNAKE", W / 2, 52, this.bigFont, ui.TEXT, "center");
      ui.text(ctx, t("snake.subtitle", { mode: t("snake.singleplayer"), hs: this.best }), W / 2, 88, this.small, ui.TEXT_DIM, "center");

      // Modus-Auswahl
      const mk = this.modeKey;
      const mp = this.modePanel;
      draw.rect(ctx, ui.PANEL_LIGHT, mp, 0, 10);
      draw.rect(ctx, ui.ACCENT, mp, 2, 10);
      ui.text(ctx, t("snake.mode." + mk), mp.centerx, mp.top + 18, this.font, ui.TEXT, "center");
      ui.text(ctx, t("snake.mode." + mk + ".desc"), mp.centerx, mp.top + 41, this.small, ui.TEXT_DIM, "center");
      for (const [r, sym] of [[this.modeLeft, "<"], [this.modeRight, ">"]]) {
        ui.text(ctx, sym, r.centerx, r.centery, this.bigFont, ui.ACCENT, "center");
      }
      const allowed = this.allowedModes();
      const pos = Math.max(0, allowed.indexOf(this.modeIndex));
      const dots = allowed.map((_, i) => (i === pos ? "*" : ".")).join(" ");
      ui.text(ctx, dots, W / 2, mp.bottom + 9, this.tiny, ui.TEXT_DIM, "center");

      // Ansicht 2D/3D
      const vr = this.viewRect;
      draw.rect(ctx, this.view3d ? ui.BTN_SEL : ui.BTN, vr, 0, 8);
      draw.rect(ctx, this.view3d ? ui.ACCENT : ui.BORDER, vr, 1, 8);
      ui.text(ctx, t("snake.view_toggle"), vr.x + 16, vr.centery, this.font, ui.TEXT, "midleft");
      const wert = this.view3d ? t("snake.view_3d") : t("snake.view_2d");
      ui.text(ctx, "< " + wert + " >", vr.right - 16, vr.centery, this.font, this.view3d ? ui.ACCENT : ui.TEXT_DIM, "midright");

      if (this.view3dActive) this.drawCamButton(ctx, this.wrapRect);
      else this.drawSetupToggle(ctx, this.wrapRect, t("snake.wrap_toggle"), this.wrap);
      this.drawSetupToggle(ctx, this.bonusRect, t("snake.bonus_toggle"), this.bonus);
      if (this.competitive) this.drawHardcoreToggle(ctx, this.applesRect, this.hardcore);
      else this.drawSetupValue(ctx, this.applesRect, t("snake.apples_toggle"), String(this.appleCount), this.appleCount > 1);

      draw.rect(ctx, ui.BTN_SEL, this.startRect, 0, 10);
      ui.text(ctx, t("common.start"), this.startRect.centerx, this.startRect.centery, this.font, ui.TEXT, "center");

      if (this.view3dActive) ui.text(ctx, t("snake.hint_3d"), W / 2, H - 54, this.tiny, ui.ACCENT, "center");
      ui.text(ctx, t("snake.setup_hint"), W / 2, H - 34, this.small, ui.TEXT_DIM, "center");
      if (this.competitive && this.hardcore) {
        const puls = 0.5 + 0.5 * Math.sin(ui.ticks() * 0.006);
        ui.text(ctx, t("snake.hardcore_hint"), W / 2, H - 14, this.tiny, [255, Math.trunc(80 + 60 * puls), Math.trunc(80 + 60 * puls)], "center");
      } else {
        ui.text(ctx, t("web.snake.boost_hint"), W / 2, H - 14, this.tiny, ui.GREEN, "center");
      }

      this.drawBrushButton(ctx);
    }

    /** Pinsel-Knopf oben rechts (Personalisieren) - mit Farb-Spitze. */
    drawBrushButton(ctx) {
      const r = this.brushRect;
      draw.rect(ctx, ui.BTN, r, 0, 8);
      draw.rect(ctx, ui.ACCENT, r, 1, 8);
      const tip = NGB.headColor(); // Spitze zeigt die aktive Kopffarbe
      const hx0 = r.left + 10, hy0 = r.bottom - 10; // Borsten-Spitze (unten links)
      const hx1 = r.right - 8, hy1 = r.top + 8; // Griffende (oben rechts)
      draw.line(ctx, [170, 140, 95], [hx0 + 4, hy0 - 4], [hx1, hy1], 4);
      const fx = hx0 + (hx1 - hx0) * 0.3, fy = hy0 + (hy1 - hy0) * 0.3; // Metallzwinge
      draw.circle(ctx, [205, 208, 216], [Math.trunc(fx), Math.trunc(fy)], 3);
      draw.polygon(ctx, tip, [[hx0 - 4, hy0 + 2], [hx0 + 5, hy0 - 5], [hx0 + 2, hy0 + 6]]);
      draw.circle(ctx, tip, [hx0 - 1, hy0 + 2], 3);
    }

    /** Setup-Zeile im 3D-Modus: öffnet das 3D-Kamera-Menü. */
    drawCamButton(ctx, rect) {
      draw.rect(ctx, ui.BTN, rect, 0, 8);
      draw.rect(ctx, ui.ACCENT, rect, 1, 8);
      ui.text(ctx, t("snake.cam.open"), rect.x + 16, rect.centery, this.font, ui.TEXT, "midleft");
      const stat = this.camSmooth ? t("common.on") : t("common.off");
      ui.text(ctx, stat + "  ›", rect.right - 14, rect.centery, this.font, ui.ACCENT, "midright");
    }

    // ----- 3D-Kamera-Menü ------------------------------------------------------------------
    drawCam3d(ctx) {
      const W = this.width, H = this.height;
      ui.drawBackground(ctx, W, H);
      ui.text(ctx, t("snake.cam.title"), W / 2, 60, this.bigFont, ui.TEXT, "center");
      ui.text(ctx, t("snake.cam.subtitle"), W / 2, 100, this.small, ui.TEXT_DIM, "center");

      this.drawSetupToggle(ctx, this.camSmoothRect, t("snake.cam.smooth"), this.camSmooth);
      this.drawCamValue(ctx, this.camFovRect, t("snake.cam.fov"), this.camFov.toFixed(2), this.camFovMinus, this.camFovPlus);
      this.drawCamValue(ctx, this.camHeightRect, t("snake.cam.height"), this.camHeight.toFixed(1), this.camHeightMinus, this.camHeightPlus);
      this.drawSetupToggle(ctx, this.camTurnRect, t("snake.cam.turn_shake"), this.camTurnShake);

      draw.rect(ctx, ui.BTN_SEL, this.camBackRect, 0, 10);
      ui.text(ctx, t("snake.cam.back"), this.camBackRect.centerx, this.camBackRect.centery, this.font, ui.TEXT, "center");
      ui.text(ctx, t("snake.cam.hint"), W / 2, H - 16, this.tiny, ui.TEXT_DIM, "center");
    }

    /** Wertzeile mit -/+ Knöpfen (FOV / Kamerahöhe). */
    drawCamValue(ctx, rect, label, value, minus, plus) {
      draw.rect(ctx, ui.BTN, rect, 0, 8);
      draw.rect(ctx, ui.BORDER, rect, 1, 8);
      ui.text(ctx, label, rect.x + 16, rect.centery, this.font, ui.TEXT, "midleft");
      for (const [r, sym] of [[minus, "-"], [plus, "+"]]) {
        draw.rect(ctx, ui.BTN_SEL, r, 0, 6);
        ui.text(ctx, sym, r.centerx, r.centery, this.font, ui.TEXT, "center");
      }
      ui.text(ctx, value, Math.floor((minus.right + plus.left) / 2), rect.centery, this.font, ui.GREEN, "center");
    }

    /** HARDCORE-Setup-Zeile: leuchtet kräftig rot, wenn aktiv. */
    drawHardcoreToggle(ctx, rect, an) {
      let labCol, valCol;
      if (an) {
        const puls = 0.5 + 0.5 * Math.sin(ui.ticks() * 0.006);
        draw.rect(ctx, [COL_HARDCORE[0], COL_HARDCORE[1], COL_HARDCORE[2], Math.trunc(55 + 90 * puls)], rect.inflate(22, 22), 0, 14);
        draw.rect(ctx, [95, 18, 22], rect, 0, 8);
        draw.rect(ctx, COL_HARDCORE, rect, 2, 8);
        labCol = [255, 210, 210];
        valCol = [255, 120, 120];
      } else {
        draw.rect(ctx, ui.BTN, rect, 0, 8);
        draw.rect(ctx, ui.BORDER, rect, 1, 8);
        labCol = ui.TEXT;
        valCol = ui.TEXT_DIM;
      }
      ui.text(ctx, t("snake.hardcore_toggle"), rect.x + 16, rect.centery, this.font, labCol, "midleft");
      const wert = an ? t("common.on") : t("common.off");
      ui.text(ctx, "< " + wert + " >", rect.right - 16, rect.centery, this.font, valCol, "midright");
    }

    drawSetupToggle(ctx, rect, label, an) {
      this.drawSetupValue(ctx, rect, label, an ? t("common.on") : t("common.off"), an);
    }

    /** Setup-Zeile mit einem Wert in < .. >-Klammern (Toggle oder Zahl). */
    drawSetupValue(ctx, rect, label, wert, hervor) {
      draw.rect(ctx, hervor ? ui.BTN_SEL : ui.BTN, rect, 0, 8);
      draw.rect(ctx, ui.BORDER, rect, 1, 8);
      ui.text(ctx, label, rect.x + 16, rect.centery, this.font, ui.TEXT, "midleft");
      ui.text(ctx, "< " + wert + " >", rect.right - 16, rect.centery, this.font, hervor ? ui.GREEN : ui.TEXT_DIM, "midright");
    }
  }

  // Der Python-Boost-Hinweis nennt noch Spieler 2 - die Web-Version ist nur Einzelspieler.
  PG.addStrings({
    de: { "web.snake.boost_hint": "Boost im Spiel:  Leertaste/Shift oder Enter gedrückt halten" },
    en: { "web.snake.boost_hint": "Boost in game:  hold Space/Shift or Enter" },
  });

  PG.register(SnakeGame, {
    id: "SnakeGame",
    key: "snake",
    name: "Snake",
    settingsKey: "snake",
    defaults: { wrap: false, bonus_apple: false },
  });
})();
