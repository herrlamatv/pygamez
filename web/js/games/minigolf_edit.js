/*
 * minigolf_edit.js - Eigene Minigolf-Bahnen (Port von games/minigolf_edit.py)
 * ===========================================================================
 * Beides hängt an MiniGolfGame (minigolf.js) und zeichnet auf dessen Fläche -
 * eigene Game-Klassen wären hier fehl am Platz, weil Editor und Spiel dieselbe
 * Instanz teilen (Test-Spielen soll ja zurückführen können).
 *
 *   swear      Wortfilter - PG.swear aus ugc.js (mit Geometry Dash geteilt)
 *   ugc        Speicher der eigenen Bahnen - PG.ugc aus ugc.js (PG.store statt
 *              ugc.json); Teilen als .lamapgzmap-Download, Import per Dateiwahl
 *   MapList    Liste der eigenen Bahnen: Neu, Bearbeiten, Spielen, Löschen,
 *              Teilen (Export) und Importieren.
 *   MapEditor  Der eigentliche Editor: Leinwand, Palette mit 15 Hindernis-
 *              Typen, Parameterleiste, Vorlagen, Undo/Redo, Test-Spielen.
 *
 * Gezeichnet wird über minigolf_draw.js - dieselben Funktionen wie im Spiel.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const gen = PG.minigolfGen;
  const mdraw = PG.minigolfDraw;
  const { BALL_R, BORDER } = gen;
  const COL = mdraw.COL;

  // ===========================================================================
  //  Wortfilter und Speicher (gemeinsam mit Geometry Dash: ugc.js)
  // ===========================================================================
  //
  // swear = PG.swear, ugc = die Minigolf-Sicht auf PG.ugc: dieselbe
  // Schnittstelle wie früher (loadMaps, saveMap, exportText, ...), dazu die
  // Grenzen der Bahngröße im Editor (Bahn-Einheiten).
  const swear = PG.swear;
  const ugc = Object.assign(PG.ugc.forGame("minigolf"), {
    MIN_W: 60.0, MAX_W: 160.0, STEP_W: 10.0,
    MIN_H: 80.0, MAX_H: 240.0, STEP_H: 10.0,
    MIN_PAR: 1, MAX_PAR: 9,

    /** Reines Bahn-Objekt aus einer Map (Kopfdaten bleiben draußen). */
    toHole(m) {
      m = gen.normalize(gen.cloneHole(m));
      const hole = { par: Math.trunc(Number(m.par) || 3), tee: m.tee.slice(0, 2), cup: m.cup.slice(0, 2), w: m.w, h: m.h };
      for (const key of gen.HOLE_LISTS) hole[key] = m[key].map((it) => it.slice());
      return hole;
    },

    /** Frische Map (leere Bahn, wenn hole fehlt). */
    newMap(name = "", mapId = "", author = "", hole = null) {
      const h = gen.normalize(hole ? gen.cloneHole(hole) : gen.makeHole(3, [gen.CW / 2, gen.CH - 16], [gen.CW / 2, 22]));
      const now = PG.ugc.stamp();
      return Object.assign({ v: PG.ugc.VERSION, id: mapId, name, author, created: now, edited: now }, h);
    },
  });

  // ===========================================================================
  //  Werkzeuge und Palette
  // ===========================================================================

  // Werkzeuge, die kein Hindernis setzen.
  const TOOLS = ["select", "erase", "tee", "cup"];

  // Acht Himmelsrichtungen - Richtungen werden im Editor durchgeklickt, statt
  // dx und dy einzeln einzustellen.
  const DIRS = [[0.0, -1.0], [0.7071, -0.7071], [1.0, 0.0], [0.7071, 0.7071],
    [0.0, 1.0], [-0.7071, 0.7071], [-1.0, 0.0], [-0.7071, -0.7071]];

  /** Nächstliegende der acht Richtungen zu (dx, dy). */
  function dirIndex(dx, dy) {
    let best = 0, bd = -2.0;
    const n = Math.hypot(dx, dy) || 1.0;
    DIRS.forEach(([ux, uy], i) => {
      const d = (dx / n) * ux + (dy / n) * uy;
      if (d > bd) {
        best = i;
        bd = d;
      }
    });
    return best;
  }

  /** Ein einstellbarer Wert eines Hindernisses (für die Parameterleiste). */
  const P = (key, idx, lo, hi, step, kind = "num") => ({ key, idx, lo, hi, step, kind });

  // Alles, was der Editor über die 15 Hindernis-Typen wissen muss:
  //   kind    "rect"   aufziehen (Breite/Höhe aus der Mausbewegung)
  //           "circle" ein Klick, Größe über die Parameterleiste
  //           "pair"   zwei Klicks (Rohr: Eingang und Ausgang)
  //   params  was sich nachträglich einstellen lässt
  const PALETTE = [
    ["walls", "rect", [P("w", 2, 2, 160, 1), P("h", 3, 2, 240, 1)]],
    ["sand", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2)]],
    ["water", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2)]],
    ["ice", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2)]],
    ["sticky", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2)]],
    ["slopes", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2), P("dir", 4, 0, 0, 1, "dir"), P("accel", -1, 6, 60, 2, "mag")]],
    ["boosters", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2), P("dir", 4, 0, 0, 1, "dir"), P("boost", 6, 40, 200, 10)]],
    ["gates", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 2, 240, 1), P("dir", 4, 0, 0, 1, "dir")]],
    ["jumps", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 4, 240, 2), P("dir", 4, 0, 0, 1, "dir"), P("dist", 6, 10, 90, 5)]],
    ["movers", "rect", [P("w", 2, 4, 160, 2), P("h", 3, 2, 240, 1), P("dir", 4, 0, 0, 1, "movedir"), P("dist", -2, 6, 80, 2, "span"), P("speed", 6, 6, 60, 2)]],
    ["bumpers", "circle", [P("r", 2, 3, 16, 1)]],
    ["magnets", "circle", [P("r", 2, 8, 60, 2), P("force", 3, -260, 260, 20)]],
    ["spinners", "circle", [P("r", 2, 5, 30, 1), P("speed", 3, -5, 5, 0.5)]],
    ["mills", "circle", [P("len", 2, 5, 40, 1), P("arms", 3, 2, 4, 1), P("speed", 4, -3, 3, 0.2)]],
    ["tunnels", "pair", [P("r", 4, 3, 12, 1)]],
  ];
  const PALETTE_KEYS = PALETTE.map((p) => p[0]);
  const PAL = {};
  for (const p of PALETTE) PAL[p[0]] = p;

  // Farbe je Typ für die Palettenknöpfe (aus minigolf_draw, damit die Knöpfe
  // aussehen wie das, was sie setzen).
  const SWATCH = {
    walls: COL.WALL, sand: COL.SAND, water: COL.WATER, ice: COL.ICE, sticky: COL.STICKY,
    slopes: COL.SLOPE, boosters: COL.BOOST, gates: COL.GATE, jumps: COL.JUMP, movers: COL.WALL_HI,
    bumpers: COL.BUMPER, magnets: COL.MAGNET, spinners: COL.SPIN, mills: COL.MILL, tunnels: COL.TUNNEL,
  };
  const ROUND_KEYS = ["bumpers", "magnets", "spinners", "mills", "tunnels"];

  /** Baut einen neuen Hindernis-Eintrag mit brauchbaren Startwerten. */
  function newItem(key, x, y, w = 0.0, h = 0.0, x2 = null, y2 = null) {
    switch (key) {
      case "walls": case "sand": case "water": case "ice": case "sticky": return [x, y, w, h];
      case "slopes": return [x, y, w, h, 0.0, 24.0];
      case "boosters": return [x, y, w, h, 0.0, -1.0, 120.0];
      case "gates": return [x, y, w, h, 0.0, -1.0];
      case "jumps": return [x, y, w, h, 0.0, -1.0, 40.0];
      case "movers": return [x, y, w, h, Math.min(30.0, w * 1.5), 0.0, 24.0];
      case "bumpers": return [x, y, 6.0];
      case "magnets": return [x, y, 24.0, 120.0];
      case "spinners": return [x, y, 12.0, 2.0];
      case "mills": return [x, y, 13.0, 2.0, 1.5];
      case "tunnels": return [x, y, x2, y2, 5.0];
      default: return [x, y, w, h];
    }
  }

  // ===========================================================================
  //  Vorlagen
  // ===========================================================================
  //
  // Jede Vorlage lässt neben ihrem Kunststück IMMER einen normalen Weg offen -
  // eine Bahn, die nur mit einem Trick lösbar ist, wäre keine Vorlage, sondern
  // eine Falle.
  const H = gen.makeHole;
  const TEMPLATES = [
    ["blank", () => H(2, [50, 146], [50, 20])],
    // Riegel mit Durchlass rechts - und einem Rohr als Abkürzung.
    ["tunnel", () => H(3, [24, 146], [76, 24], { walls: [[6, 84, 62, 8]], tunnels: [[22, 118, 30, 44, 5]], sand: [[70, 104, 22, 16]] })],
    // Inselgrün: Wasser ringsum, ein breiter Hals führt hinauf.
    ["island", () => H(3, [50, 146], [50, 34], { water: [[6, 20, 26, 34], [68, 20, 26, 34], [6, 54, 24, 16], [70, 54, 24, 16]], sand: [[40, 96, 20, 14]] })],
    // Korridor mit Windmühle - reines Timing.
    ["mill", () => H(3, [50, 146], [50, 18], { walls: [[22, 26, 8, 92], [70, 26, 8, 92]], mills: [[50, 92, 14, 2, 1.4]] })],
    // Drei versetzte Tore.
    ["zigzag", () => H(4, [18, 146], [82, 20], { walls: [[28, 118, 66, 7], [6, 84, 66, 7], [28, 50, 66, 7]] })],
    // Trockene Gasse zwischen zwei Teichen.
    ["water", () => H(3, [50, 146], [50, 22], { water: [[6, 60, 30, 44], [64, 60, 30, 44]], sand: [[40, 116, 20, 14]] })],
    // Offenes Feld voller Gummipuffer.
    ["bumper", () => H(3, [50, 146], [50, 20], { bumpers: [[30, 100, 6], [70, 100, 6], [50, 70, 7], [28, 44, 5], [72, 44, 5]] })],
    // Steigung, die zurückschiebt - hier braucht es Kraft.
    ["ramp", () => H(3, [50, 148], [50, 20], { slopes: [[20, 52, 60, 56, 0.0, 30.0]], sand: [[18, 116, 22, 14], [60, 116, 22, 14]] })],
    // Eisfläche mit Bande - der Ball will einfach nicht anhalten.
    ["ice", () => H(3, [50, 148], [50, 20], { ice: [[10, 44, 80, 74]], walls: [[40, 90, 20, 8]], sticky: [[38, 26, 24, 12]] })],
    // Labyrinth aus kurzen Wänden.
    ["maze", () => H(4, [16, 148], [84, 18], { walls: [[30, 122, 64, 7], [6, 96, 64, 7], [30, 70, 64, 7], [6, 44, 64, 7]], sand: [[74, 106, 18, 12]] })],
    // Zwei Magnete neben der Gasse: einer zieht, einer stößt ab.
    ["magnet", () => H(3, [50, 146], [50, 22], { magnets: [[18, 96, 30, 140], [82, 62, 30, -140]], walls: [[46, 118, 8, 14]] })],
    // Sprungschanze über einen Riegel - außen herum geht es auch.
    ["jump", () => H(3, [50, 150], [50, 20], { jumps: [[42, 124, 16, 10, 0.0, -1.0, 46]], walls: [[24, 92, 52, 8]], sand: [[10, 104, 16, 14], [74, 104, 16, 14]] })],
  ];

  /** Fertiges Bahn-Objekt einer Vorlage (leer, wenn der Name nicht passt). */
  function template(key) {
    const tpl = TEMPLATES.find((x) => x[0] === key);
    return gen.normalize((tpl || TEMPLATES[0])[1]());
  }

  // ===========================================================================
  //  Prüfung einer Bahn
  // ===========================================================================

  /**
   * Prüft eine Bahn vor dem Speichern. Liefert "" oder einen Fehlerschlüssel.
   * Geprüft wird nur, was die Bahn UNSPIELBAR macht.
   */
  function validate(hole) {
    hole = gen.normalize(hole);
    const cw = hole.w, ch = hole.h;
    const lo = BORDER + BALL_R;
    const pts = [hole.tee, hole.cup];
    for (const [px, py] of pts) {
      if (!(lo <= px && px <= cw - lo && lo <= py && py <= ch - lo)) return "bounds";
    }
    const clear = BALL_R + 1.0;
    for (const key of ["walls", "water", "gates", "jumps"]) {
      for (const [x, y, w, h] of hole[key]) {
        for (const [px, py] of pts) {
          if (x - clear < px && px < x + w + clear && y - clear < py && py < y + h + clear) return "blocked";
        }
      }
    }
    for (const [x, y, r] of hole.bumpers) {
      for (const [px, py] of pts) if (Math.hypot(x - px, y - py) < r + clear) return "blocked";
    }
    if (Math.hypot(hole.tee[0] - hole.cup[0], hole.tee[1] - hole.cup[1]) < 12.0) return "tooclose";
    return "";
  }

  // ===========================================================================
  //  Gemeinsame Zeichenhelfer
  // ===========================================================================

  /** Text auf maxW kürzen ("..." anhängen), wie die Python-Schleifen. */
  function fit(fnt, text, maxW, minLen = 1) {
    if (fnt.width(text) <= maxW) return text;
    let kurz = text;
    while (kurz.length > minLen && fnt.width(kurz + "...") > maxW) kurz = kurz.slice(0, -1);
    return kurz + "...";
  }

  /** Knopf im Stil des Minigolf-Setups (mit Kürzen bei langem Text). */
  function btn(ctx, rc, text, fnt, on = false, accent = null, enabled = true) {
    draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
    draw.rect(ctx, on ? accent || ui.ACCENT : ui.BORDER, rc, on ? 2 : 1, 7);
    const col = on || enabled ? ui.TEXT : ui.TEXT_FAINT;
    ui.text(ctx, fit(fnt, text, rc.w - 8), rc.centerx, rc.centery, fnt, col, "center");
    return rc;
  }

  /**
   * Kleiner Richtungspfeil - wird GEZEICHNET statt geschrieben (die Schrift
   * hat nicht überall Pfeil-Zeichen).
   */
  function arrow(ctx, center, dx, dy, size, col) {
    const n = Math.hypot(dx, dy) || 1.0;
    const ux = dx / n, uy = dy / n;
    const [cx, cy] = center;
    const tip = [cx + ux * size, cy + uy * size];
    const tail = [cx - ux * size, cy - uy * size];
    draw.line(ctx, col, tail, tip, 2);
    const head = size * 0.7;
    draw.polygon(ctx, col, [tip,
      [tip[0] - ux * head - uy * head * 0.6, tip[1] - uy * head + ux * head * 0.6],
      [tip[0] - ux * head + uy * head * 0.6, tip[1] - uy * head - ux * head * 0.6]]);
  }

  /** Sinnbild eines Werkzeugs - ebenfalls gezeichnet, nicht geschrieben. */
  function toolIcon(ctx, rc, key, col) {
    const cx = Math.trunc(rc.centerx), cy = Math.trunc(rc.centery);
    const r = Math.max(4, Math.floor(Math.min(rc.w, rc.h) / 4));
    const h2 = r >> 1;
    if (key === "select") {
      // Mauszeiger
      draw.polygon(ctx, col, [[cx - r, cy - r], [cx + r, cy], [cx, cy + r * 0.3], [cx + r * 0.3, cy + r]], 2);
    } else if (key === "erase") {
      // Radierer mit Strich
      const box = new PG.Rect(0, 0, r * 2, Math.trunc(r * 1.5));
      box.center = [cx, cy];
      draw.rect(ctx, col, box, 2, 2);
      draw.line(ctx, col, [box.left, box.bottom], [box.right, box.top], 2);
    } else if (key === "tee") {
      // Abschlag: Ring mit Punkt
      draw.circle(ctx, col, [cx, cy], r, 2);
      draw.circle(ctx, col, [cx, cy], Math.max(1, Math.floor(r / 3)));
    } else {
      // Loch mit Fahne
      draw.circle(ctx, col, [cx + h2, cy + r], Math.max(2, h2));
      draw.line(ctx, col, [cx + h2, cy + r], [cx + h2, cy - r], 2);
      draw.polygon(ctx, col, [[cx + h2, cy - r], [cx - r, cy - h2], [cx + h2, cy]]);
    }
  }

  /** Sinnbild der Werkzeugleiste (Raster, Undo, Redo, Leeren). */
  function editIcon(ctx, rc, key, col) {
    const cx = Math.trunc(rc.centerx), cy = Math.trunc(rc.centery);
    const r = Math.max(4, Math.floor(Math.min(rc.w, rc.h) / 4));
    if (key === "grid") {
      const box = new PG.Rect(0, 0, r * 2, r * 2);
      box.center = [cx, cy];
      draw.rect(ctx, col, box, 1);
      draw.line(ctx, col, [box.centerx, box.top], [box.centerx, box.bottom]);
      draw.line(ctx, col, [box.left, box.centery], [box.right, box.centery]);
    } else if (key === "undo" || key === "redo") {
      const sign = key === "undo" ? -1 : 1;
      const rect = new PG.Rect(0, 0, r * 2, r * 2);
      rect.center = [cx, cy + (r >> 1)];
      const [start, end] = key === "undo" ? [0.35, 2.9] : [0.25, 2.8];
      draw.arc(ctx, col, rect, start, end, 2);
      arrow(ctx, [cx - sign * r, cy + (r >> 1)], 0, 1, Math.max(3, r >> 1), col);
    } else {
      // leeren: Kreuz
      draw.line(ctx, col, [cx - r, cy - r], [cx + r, cy + r], 2);
      draw.line(ctx, col, [cx + r, cy - r], [cx - r, cy + r], 2);
    }
  }

  /** Kleine Pille als Rückmeldung (Toast). */
  function drawToast(ctx, text, fnt, cx, bottom, padW, padH) {
    const w = fnt.width(text) + padW, h = fnt.height + padH;
    const r = new PG.Rect(Math.trunc(cx - w / 2), bottom - h, w, h);
    ui.drawPanel(ctx, r, { radius: h >> 1, shadow: false });
    draw.rect(ctx, ui.ACCENT2, r, 1, h >> 1);
    ui.text(ctx, text, r.centerx, r.centery, fnt, ui.TEXT, "center");
  }

  // ===========================================================================
  //  MAPS-Reiter: die Liste der eigenen Bahnen
  // ===========================================================================

  // Knöpfe unter der Liste. "new" und "import" gehen immer, der Rest braucht
  // eine ausgewählte Bahn.
  const LIST_BUTTONS = ["new", "edit", "play", "delete", "share", "import"];
  const NEEDS_SEL = ["edit", "play", "delete", "share"];

  class MapList {
    constructor(game) {
      this.game = game;
      this.items = [];
      this.sel = 0;
      this.first = 0;
      this.rowsVisible = 1;
      this.toast = "";
      this.toastT = 0.0;
      this.confirm = ""; // id, deren Löschen bestätigt werden will
      // Teilen-Overlay
      this.share = null; // die Bahn, die geteilt wird (oder null)
      this.fAuthor = new ui.TextInput("", ugc.MAX_AUTHOR);
      this.fFile = new ui.TextInput("", ugc.MAX_ID, ui.TextInput.ID_CHARS);
      this.focus = 0; // 0 = Ersteller, 1 = Dateiname
      this.err = "";
      this.reload();
      this.layout();
    }

    // ----- Daten --------------------------------------------------------
    reload() {
      this.items = ugc.loadMaps();
      this.sel = Math.max(0, Math.min(this.sel, this.items.length - 1));
      this.clamp();
    }

    selected() {
      return this.items.length ? this.items[this.sel] : null;
    }

    // ----- Layout -------------------------------------------------------
    layout() {
      const g = this.game;
      const w = g.width, h = g.height;
      this.fnt = ui.font(Math.max(13, Math.floor(h / 30)));
      this.tiny = ui.font(Math.max(10, Math.floor(h / 40)));
      this.rowH = Math.max(26, Math.min(40, Math.floor(h / 11)));
      // Knopfzeile unten: bei schmalen Fenstern zwei Reihen zu drei.
      const cols = w >= 720 ? 6 : 3;
      const bh = Math.max(24, Math.min(34, Math.floor(h / 13)));
      const gap = 6;
      const rows = Math.ceil(LIST_BUTTONS.length / cols);
      const bw = (w - 24 - gap * (cols - 1)) / cols;
      const bottom = h - 18;
      this.btnRects = {};
      LIST_BUTTONS.forEach((key, i) => {
        const r = Math.floor(i / cols), c = i % cols;
        const y = bottom - (rows - r) * (bh + gap) + gap;
        this.btnRects[key] = new PG.Rect(Math.trunc(12 + c * (bw + gap)), Math.trunc(y), Math.trunc(bw), bh);
      });
      this.listTop = g.tabBottom + 8;
      this.listBottom = bottom - rows * (bh + gap) - 4;
      this.rowsVisible = Math.max(1, Math.floor((this.listBottom - this.listTop - 14) / this.rowH));
      this.listW = w - 30;
      this.clamp();
      // Teilen-Overlay
      const pw = Math.min(w - 40, 380);
      const fh = Math.max(22, Math.min(30, Math.floor(h / 14)));
      const ph = Math.min(h - 30, 78 + 3 * fh + 62);
      this.shareRect = new PG.Rect((w - pw) >> 1, (h - ph) >> 1, pw, ph);
      const fx = this.shareRect.x + 18;
      const fw = pw - 36;
      this.authorRect = new PG.Rect(fx, this.shareRect.y + 46, fw, fh);
      this.fileRect = new PG.Rect(fx, this.authorRect.bottom + 24, fw, fh);
      const sy = this.fileRect.bottom + 16;
      const sbw = (fw - 10) / 2.0;
      this.shareBtn = {
        as: new PG.Rect(fx, sy, Math.trunc(sbw), fh),
        dl: new PG.Rect(Math.trunc(fx + sbw + 10), sy, Math.trunc(sbw), fh),
        cancel: new PG.Rect(fx, sy + fh + 6, fw, fh),
      };
    }

    clamp() {
      const n = this.items.length;
      const vis = this.rowsVisible || 1;
      this.first = Math.max(0, Math.min(this.first, Math.max(0, n - vis)));
      if (this.sel < this.first) this.first = this.sel;
      else if (this.sel >= this.first + vis) this.first = this.sel - vis + 1;
    }

    rowRect(i) {
      return new PG.Rect(12, this.listTop + i * this.rowH, this.listW, this.rowH - 3);
    }

    // ----- Rückmeldung -------------------------------------------------
    setToast(text) {
      this.toast = text;
      this.toastT = 2.8;
    }

    update(dt) {
      if (this.toastT > 0) {
        this.toastT -= dt;
        if (this.toastT <= 0) this.toast = "";
      }
    }

    // ----- Eingabe ------------------------------------------------------
    handle(ev) {
      if (this.share) return this.handleShare(ev);
      const g = this.game;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Up" || k === "Left" || g.isAction(k, "up")) this.move(-1);
        else if (k === "Down" || k === "Right" || g.isAction(k, "down")) this.move(1);
        else if (k === "Return" || k === "space") this.action(this.items.length ? "play" : "new");
        else if (k === "Delete" || k === "BackSpace") this.action("delete");
        else if (k === "n" || k === "N") this.action("new");
        else if (k === "e" || k === "E") this.action("edit");
        return true;
      }
      if (ev.kind === "wheel") {
        this.first = Math.max(0, Math.min(Math.max(0, this.items.length - this.rowsVisible), this.first - ev.delta));
        return true;
      }
      if (ev.kind === "mousedown") {
        for (const key of LIST_BUTTONS) {
          if (this.btnRects[key].collidepoint(ev.pos)) {
            this.action(key);
            return true;
          }
        }
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.items.length) break;
          if (this.rowRect(i).collidepoint(ev.pos)) {
            if (this.sel !== idx) this.confirm = "";
            this.sel = idx;
            g.playSound("move");
            return true;
          }
        }
        return true;
      }
      return false;
    }

    move(d) {
      if (!this.items.length) return;
      this.sel = PG.mod(this.sel + d, this.items.length);
      this.confirm = "";
      this.clamp();
      this.game.playSound("move");
    }

    action(key) {
      const g = this.game;
      if (key !== "delete") this.confirm = "";
      if (key === "new") {
        if (ugc.isFull()) {
          this.setToast(t("golf.ugc.err.full", { max: ugc.MAX_MAPS }));
          return;
        }
        g.ugcNewMap();
        return;
      }
      if (key === "import") {
        this.doImport();
        return;
      }
      const m = this.selected();
      if (!m) return;
      if (key === "edit") g.ugcEdit(m);
      else if (key === "play") g.ugcPlay(m.id);
      else if (key === "share") this.openShare(m);
      else if (key === "delete") {
        if (this.confirm !== m.id) {
          this.confirm = m.id;
          g.playSound("select");
          return;
        }
        ugc.deleteMap(m.id);
        this.confirm = "";
        this.reload();
        this.setToast(t("golf.ugc.deleted"));
        g.playSound("click");
      }
    }

    // ----- Teilen -------------------------------------------------------
    openShare(m) {
      this.share = m;
      this.err = "";
      this.focus = 0;
      this.fAuthor.setText(m.author || ugc.lastAuthor());
      this.fFile.setText(m.id || "map");
      this.game.playSound("click");
    }

    closeShare() {
      this.share = null;
      this.err = "";
    }

    handleShare(ev) {
      if (ev.kind === "mousedown") {
        const p = ev.pos;
        if (this.authorRect.collidepoint(p)) this.focus = 0;
        else if (this.fileRect.collidepoint(p)) this.focus = 1;
        else if (this.shareBtn.as.collidepoint(p)) this.doExport();
        else if (this.shareBtn.dl.collidepoint(p)) this.doExport();
        else if (this.shareBtn.cancel.collidepoint(p)) this.closeShare();
        else if (!this.shareRect.collidepoint(p)) this.closeShare();
        return true;
      }
      if (ev.kind !== "keydown") return true;
      const field = this.focus === 0 ? this.fAuthor : this.fFile;
      if (field.handle(ev)) {
        this.err = "";
        return true;
      }
      if (["Tab", "ISO_Left_Tab", "Up", "Down"].includes(ev.key)) this.focus = 1 - this.focus;
      else if (ev.key === "Escape") this.closeShare();
      else if (ev.key === "Return" || ev.key === "KP_Enter") this.doExport();
      return true;
    }

    /**
     * Teilen: im Browser gibt es keinen Speichern-Dialog und keinen
     * Downloads-Ordner zum Hineinschreiben - beide Knöpfe bieten die Datei
     * deshalb als Download an (wohin, entscheidet der Browser).
     */
    doExport() {
      const m = this.share;
      if (!m) return;
      const author = this.fAuthor.text.trim();
      const name = this.fFile.text.trim() || m.id || "map";
      if (author && !swear.isClean(author)) {
        this.err = t("golf.ugc.err.swear");
        this.game.playSound("hit");
        return;
      }
      const out = gen.cloneHole(m);
      if (author) {
        out.author = author;
        ugc.setLastAuthor(author);
      }
      const filename = name + ugc.EXT;
      const [ok, why, text] = ugc.exportText(out);
      if (!ok) {
        this.err = t("golf.ugc.err." + (why === "swear" ? "swear" : "io"));
        this.game.playSound("hit");
        return;
      }
      try {
        PG.downloadText(filename, text);
      } catch (e) {
        this.err = t("golf.ugc.err.io");
        this.game.playSound("hit");
        return;
      }
      // Der Ersteller-Name gehört auch in die gespeicherte Bahn.
      if (author && m.author !== author) {
        ugc.saveMap(out);
        this.reload();
      }
      this.closeShare();
      this.setToast(t("golf.ugc.exported", { path: filename }));
      this.game.playSound("point");
    }

    doImport() {
      if (ugc.isFull()) {
        this.setToast(t("golf.ugc.err.full", { max: ugc.MAX_MAPS }));
        return;
      }
      try {
        PG.pickTextFile(ugc.EXT + ",.json", (text, filename) => this.finishImport(text, filename));
      } catch (e) {
        this.setToast(t("golf.ugc.err.nofile"));
      }
    }

    finishImport(text, filename) {
      const base = String(filename || "").replace(/\.[^.]*$/, "");
      const wanted = ugc.slug(base);
      const [ok, why, m] = ugc.importText(text);
      if (!ok) {
        const known = ["swear", "full", "format", "io"];
        this.setToast(t("golf.ugc.err." + (known.includes(why) ? why : "format")));
        this.game.playSound("hit");
        return;
      }
      this.reload();
      const i = this.items.findIndex((x) => x.id === m.id);
      if (i >= 0) {
        this.sel = i;
        this.clamp();
      }
      if (wanted && m.id !== wanted) this.setToast(t("golf.ugc.renamed", { id: m.id }));
      else this.setToast(t("golf.ugc.imported", { name: m.name }));
      this.game.playSound("point");
    }

    // ----- Zeichnen -----------------------------------------------------
    draw(ctx) {
      if (!this.items.length) this.drawEmpty(ctx);
      else {
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.items.length) break;
          this.drawRow(ctx, this.items[idx], idx, this.rowRect(i));
        }
        if (this.items.length > this.rowsVisible) this.drawScrollbar(ctx);
      }
      // Zähler direkt über der Knopfzeile (unten bündig, damit er die Knöpfe
      // nicht überdeckt).
      const btnTop = Math.min(...LIST_BUTTONS.map((k) => this.btnRects[k].y));
      ui.text(ctx, t("golf.ugc.count", { n: this.items.length, max: ugc.MAX_MAPS }), 12, btnTop - 2, this.tiny, ui.TEXT_DIM, "bottomleft");
      const has = this.selected() != null;
      for (const key of LIST_BUTTONS) {
        btn(ctx, this.btnRects[key], t("golf.ugc.btn_" + key), this.tiny, false, null, has || !NEEDS_SEL.includes(key));
      }
      if (this.toast) drawToast(ctx, this.toast, this.tiny, this.game.width / 2, this.listBottom - 2, 26, 12);
      if (this.share) this.drawShare(ctx);
    }

    drawEmpty(ctx) {
      const g = this.game;
      const box = new PG.Rect(24, this.listTop + 10, g.width - 48, Math.max(60, this.listBottom - this.listTop - 24));
      ui.drawPanel(ctx, box, { shadow: false });
      [t("golf.ugc.empty"), t("golf.ugc.empty2")].forEach((line, i) => {
        ui.text(ctx, line, box.centerx, box.centery - 12 + i * 22, this.fnt, i === 0 ? ui.TEXT_DIM : ui.TEXT_FAINT, "center");
      });
    }

    drawRow(ctx, m, idx, rc) {
      const sel = idx === this.sel;
      draw.rect(ctx, sel ? ui.PANEL_LIGHT : ui.PANEL, rc, 0, 7);
      draw.rect(ctx, sel ? this.game.accent : ui.BORDER, rc, sel ? 2 : 1, 7);
      const pad = 10;
      // Name oben, Kopfdaten unten - an den Schriftmitten ausgerichtet, damit
      // sich die beiden Zeilen auch bei 37 px Zeilenhöhe nicht berühren.
      const twoLines = rc.h >= 32;
      ui.text(ctx, m.name || "", rc.x + pad, twoLines ? rc.y + 12 : rc.centery, this.fnt, ui.TEXT, "midleft");
      let sub = `${m.id || ""}  ·  ${t("golf.ugc.size", { w: Math.trunc(m.w), h: Math.trunc(m.h) })}  ·  ${t("golf.par", { n: Math.trunc(Number(m.par) || 3) })}`;
      if (m.author) sub += "  ·  " + t("golf.ugc.by", { name: m.author });
      if (twoLines) ui.text(ctx, sub, rc.x + pad, rc.bottom - 9, this.tiny, ui.TEXT_DIM, "midleft");
      if (this.confirm === m.id) ui.text(ctx, t("golf.ugc.confirm_delete"), rc.right - pad, rc.centery, this.tiny, ui.RED, "midright");
    }

    drawScrollbar(ctx) {
      const track = new PG.Rect(this.listW + 14, this.listTop, 4, this.rowsVisible * this.rowH);
      draw.rect(ctx, ui.PANEL, track, 0, 2);
      const frac = this.rowsVisible / this.items.length;
      const h = Math.max(24, Math.trunc(track.h * frac));
      const pos = this.first / Math.max(1, this.items.length - this.rowsVisible);
      const y = track.y + Math.trunc((track.h - h) * Math.min(1.0, pos));
      draw.rect(ctx, ui.BORDER_LIGHT, [track.x, y, 4, h], 0, 2);
    }

    drawShare(ctx) {
      const g = this.game;
      draw.rect(ctx, [0, 0, 0, 150], [0, 0, g.width, g.height]);
      const r = this.shareRect;
      ui.drawPanel(ctx, r, { accentTop: g.accent });
      ui.text(ctx, t("golf.ugc.share_title"), r.centerx, r.y + 19, this.fnt, g.accent, "center");
      [[this.authorRect, "golf.ugc.creator", this.fAuthor], [this.fileRect, "golf.ugc.filename", this.fFile]].forEach(([rect, key, field], i) => {
        ui.text(ctx, t(key), rect.x, rect.y - 2, this.tiny, ui.TEXT_DIM, "bottomleft");
        field.draw(ctx, rect, this.tiny, this.focus === i);
      });
      ui.text(ctx, ugc.EXT, this.fileRect.right - 8, this.fileRect.centery, this.tiny, ui.TEXT_FAINT, "midright");
      btn(ctx, this.shareBtn.as, t("golf.ugc.export_as"), this.tiny);
      btn(ctx, this.shareBtn.dl, t("golf.ugc.export_dl"), this.tiny);
      btn(ctx, this.shareBtn.cancel, t("golf.ugc.btn_cancel"), this.tiny);
      if (this.err) ui.text(ctx, this.err, r.centerx, r.bottom - 6, this.tiny, ui.RED, "midbottom");
    }
  }

  // ===========================================================================
  //  Der Bahn-Editor
  // ===========================================================================

  // Knöpfe der zweiten Kopfzeile. Beschriftet wird nur, was sich nicht als
  // Sinnbild sagen lässt.
  const TEXT_BUTTONS = ["template", "test"];
  const ICON_BUTTONS = ["grid", "undo", "redo", "clear"];
  const UNDO_MAX = 40;
  const round1 = (v) => Math.round(v * 10) / 10;

  /**
   * Bahn bauen: Leinwand links, Palette rechts, Parameter unten.
   * Die Bahn selbst ist das gewohnte Bahn-Objekt - der Editor ändert nur
   * Listen darin.
   */
  class MapEditor {
    constructor(game, m) {
      this.game = game;
      this.map = gen.cloneHole(m);
      this.hole = gen.normalize(gen.cloneHole(m));
      this.tool = "walls"; // aktives Werkzeug (Typ oder TOOLS-Eintrag)
      this.sel = null; // [typ, index] des gewählten Hindernisses
      this.drag = null; // [typ, x0, y0, x1, y1] beim Aufziehen
      this.moving = null; // [typ, index, offx, offy] beim Verschieben
      this.pending = null; // erster Klick eines Rohrs
      this.undoStack = [];
      this.redoStack = [];
      this.dirty = false;
      this.err = "";
      this.toast = "";
      this.toastT = 0.0;
      this.picking = false; // Vorlagen-Auswahl offen?
      this.confirmBack = false;
      this.t = 0.0;
      this.grid = !!game.gridSnap;
      this.fName = new ui.TextInput(m.name || "", ugc.MAX_NAME, null, t("golf.ugc.name"));
      this.fId = new ui.TextInput(m.id || "", ugc.MAX_ID, ui.TextInput.ID_CHARS, t("golf.ugc.id"));
      this.focus = -1; // -1 = kein Feld, 0 = Name, 1 = id
      this.origId = m.id || "";
      this.layout();
    }

    // ----- Layout -------------------------------------------------------
    layout() {
      const g = this.game;
      const w = g.width, h = g.height;
      this.tiny = ui.font(Math.max(10, Math.floor(h / 42)));
      this.small = ui.font(Math.max(12, Math.floor(h / 34)));
      const bh = Math.max(20, Math.min(28, Math.floor(h / 15)));
      const gap = 5;
      // Kopfzeile 1: Zurück | Name | id | Speichern
      const y = 6;
      const backW = Math.max(48, Math.trunc(w * 0.11));
      const saveW = Math.max(62, Math.trunc(w * 0.16));
      const fieldW = (w - 24 - backW - saveW - 3 * gap) / 2.0;
      this.backRect = new PG.Rect(12, y, backW, bh);
      this.nameRect = new PG.Rect(Math.trunc(12 + backW + gap), y, Math.trunc(fieldW), bh);
      this.idRect = new PG.Rect(Math.trunc(this.nameRect.right + gap), y, Math.trunc(fieldW), bh);
      this.saveRect = new PG.Rect(Math.trunc(this.idRect.right + gap), y, saveW, bh);
      // Kopfzeile 2: Par, Breite, Höhe (je - Wert +) und sechs Knöpfe
      const y2 = y + bh + gap;
      const stepW = Math.max(16, Math.trunc(w * 0.036));
      const valW = Math.max(30, Math.trunc(w * 0.062));
      const grp = 2 * stepW + valW;
      this.numRects = {};
      let x = 12;
      for (const key of ["par", "width", "height"]) {
        this.numRects[key] = [new PG.Rect(x, y2, stepW, bh), new PG.Rect(x + stepW, y2, valW, bh), new PG.Rect(x + stepW + valW, y2, stepW, bh)];
        x += grp + gap;
      }
      // Rechts der Zahlenfelder: erst zwei beschriftete Knöpfe (Vorlage,
      // Test), dann vier Sinnbild-Knöpfe.
      const rest = w - 12 - x;
      const iconW = Math.max(22, Math.min(30, bh + 2));
      const textArea = rest - ICON_BUTTONS.length * (iconW + gap);
      const tbw = Math.max(34, (textArea - gap * TEXT_BUTTONS.length) / TEXT_BUTTONS.length);
      this.editRects = {};
      TEXT_BUTTONS.forEach((key, i) => {
        this.editRects[key] = new PG.Rect(Math.trunc(x + i * (tbw + gap)), y2, Math.trunc(tbw), bh);
      });
      const x2 = x + TEXT_BUTTONS.length * (tbw + gap);
      ICON_BUTTONS.forEach((key, i) => {
        this.editRects[key] = new PG.Rect(Math.trunc(x2 + i * (iconW + gap)), y2, iconW, bh);
      });
      this.headBottom = y2 + bh + 5;
      // Parameterleiste unten
      this.parH = bh + 16;
      this.parTop = h - this.parH;
      // Palette rechts: zwei Spalten
      const cols = 2;
      const all = TOOLS.concat(PALETTE_KEYS);
      const rows = Math.ceil(all.length / cols);
      const avail = this.parTop - this.headBottom - 6;
      const pbh = Math.max(16, Math.min(30, Math.floor((avail - (rows - 1) * 3) / rows)));
      const pbw = Math.max(34, Math.min(52, Math.trunc(w * 0.085)));
      this.palW = cols * pbw + 3;
      const px0 = w - this.palW - 6;
      this.palRects = all.map((key, i) => {
        const r = Math.floor(i / cols), c = i % cols;
        return [key, new PG.Rect(px0 + c * (pbw + 3), this.headBottom + r * (pbh + 3), pbw, pbh)];
      });
      // Leinwand: was übrig bleibt
      const cw = this.hole.w, ch = this.hole.h;
      const areaW = px0 - 18;
      // Unten Platz für die Beschriftung der Parameterleiste lassen, sonst
      // liegt sie auf der Bande der Leinwand.
      const areaH = avail - 14;
      this.scale = Math.max(0.8, Math.min(areaW / cw, areaH / ch));
      this.ox = 12 + (areaW - cw * this.scale) / 2.0;
      this.oy = this.headBottom + (areaH - ch * this.scale) / 2.0;
      this.canvas = new PG.Rect(Math.trunc(this.ox), Math.trunc(this.oy), Math.trunc(cw * this.scale), Math.trunc(ch * this.scale));
      // Vorlagen-Auswahl
      const pw = Math.min(w - 30, 420);
      const tcols = 3;
      const trows = Math.ceil(TEMPLATES.length / tcols);
      const tbh = Math.max(22, Math.min(32, Math.floor((h - 90) / (trows + 1))));
      const ph = 44 + trows * (tbh + 6) + 10;
      this.tplRect = new PG.Rect((w - pw) >> 1, Math.max(6, (h - ph) >> 1), pw, ph);
      const tw = (pw - 24 - 6 * (tcols - 1)) / tcols;
      this.tplRects = TEMPLATES.map(([key], i) => {
        const r = Math.floor(i / tcols), c = i % tcols;
        return [key, new PG.Rect(Math.trunc(this.tplRect.x + 12 + c * (tw + 6)), Math.trunc(this.tplRect.y + 38 + r * (tbh + 6)), Math.trunc(tw), tbh)];
      });
    }

    view() {
      return new mdraw.View(this.ox, this.oy, this.scale);
    }

    // ----- Undo / Änderungen -------------------------------------------
    push() {
      // Zustand sichern, bevor etwas geändert wird.
      this.undoStack.push(gen.cloneHole(this.hole));
      if (this.undoStack.length > UNDO_MAX) this.undoStack.shift();
      this.redoStack.length = 0;
      this.dirty = true;
      this.err = "";
    }

    undo() {
      if (!this.undoStack.length) return;
      this.redoStack.push(gen.cloneHole(this.hole));
      this.hole = this.undoStack.pop();
      this.sel = null;
      this.dirty = true;
      this.layout();
      this.game.playSound("move");
    }

    redo() {
      if (!this.redoStack.length) return;
      this.undoStack.push(gen.cloneHole(this.hole));
      this.hole = this.redoStack.pop();
      this.sel = null;
      this.dirty = true;
      this.layout();
      this.game.playSound("move");
    }

    // ----- Rückmeldung -------------------------------------------------
    setToast(text) {
      this.toast = text;
      this.toastT = 2.6;
    }

    update(dt) {
      this.t += dt;
      if (this.toastT > 0) {
        this.toastT -= dt;
        if (this.toastT <= 0) this.toast = "";
      }
    }

    // ----- Koordinaten --------------------------------------------------
    toCourse(pos) {
      const x = (pos[0] - this.ox) / this.scale;
      const y = (pos[1] - this.oy) / this.scale;
      return [this.snap(x), this.snap(y)];
    }

    snap(v) {
      return this.grid ? Math.round(v) : round1(v);
    }

    clampx(x) {
      return Math.max(BORDER, Math.min(this.hole.w - BORDER, x));
    }

    clampy(y) {
      return Math.max(BORDER, Math.min(this.hole.h - BORDER, y));
    }

    /** Oberstes Hindernis unter (x, y) - [typ, index] oder null. */
    pick(x, y) {
      for (let k = PALETTE_KEYS.length - 1; k >= 0; k--) {
        const key = PALETTE_KEYS[k];
        const items = this.hole[key];
        for (let i = items.length - 1; i >= 0; i--) {
          const it = items[i];
          if (key === "bumpers" || key === "magnets" || key === "spinners" || key === "mills") {
            if (Math.hypot(x - it[0], y - it[1]) <= Math.max(3.0, it[2])) return [key, i];
          } else if (key === "tunnels") {
            if (Math.hypot(x - it[0], y - it[1]) <= Math.max(3.0, it[4]) || Math.hypot(x - it[2], y - it[3]) <= Math.max(3.0, it[4])) return [key, i];
          } else if (it[0] <= x && x <= it[0] + it[2] && it[1] <= y && y <= it[1] + it[3]) {
            return [key, i];
          }
        }
      }
      return null;
    }

    // ----- Eingabe ------------------------------------------------------
    handle(ev) {
      if (this.picking) return this.handleTemplate(ev);
      if (ev.kind === "keydown") return this.handleKey(ev);
      if (ev.kind === "mousedown") return this.handleDown(ev);
      if (ev.kind === "mousemove") {
        if (this.drag) {
          const [x, y] = this.toCourse(ev.pos);
          this.drag = [this.drag[0], this.drag[1], this.drag[2], this.clampx(x), this.clampy(y)];
        } else if (this.moving) this.doMove(ev.pos);
        return true;
      }
      if (ev.kind === "mouseup") return this.handleUp(ev);
      return false;
    }

    handleKey(ev) {
      const k = ev.key;
      if (this.focus >= 0) {
        const field = this.focus === 0 ? this.fName : this.fId;
        if (field.handle(ev)) {
          this.dirty = true;
          this.err = "";
          return true;
        }
        if (k === "Tab" || k === "ISO_Left_Tab") {
          this.focus = 1 - this.focus;
          return true;
        }
        if (k === "Return" || k === "KP_Enter" || k === "Escape") {
          this.focus = -1;
          return true;
        }
      }
      if (k === "Escape") this.back();
      else if (k === "Delete" || k === "BackSpace") this.deleteSelected();
      else if (k === "u" || k === "U") this.undo();
      else if (k === "y" || k === "Y") this.redo();
      else if (k === "g" || k === "G") this.toggleGrid();
      else if (k === "s" || k === "S") this.save();
      else if (k === "Return" || k === "KP_Enter") this.test();
      return true;
    }

    handleDown(ev) {
      const pos = ev.pos;
      if (this.nameRect.collidepoint(pos)) {
        this.focus = 0;
        return true;
      }
      if (this.idRect.collidepoint(pos)) {
        this.focus = 1;
        return true;
      }
      this.focus = -1;
      if (this.backRect.collidepoint(pos)) {
        this.back();
        return true;
      }
      if (this.saveRect.collidepoint(pos)) {
        this.save();
        return true;
      }
      for (const key in this.numRects) {
        const [minus, , plus] = this.numRects[key];
        if (minus.collidepoint(pos)) {
          this.stepNum(key, -1);
          return true;
        }
        if (plus.collidepoint(pos)) {
          this.stepNum(key, 1);
          return true;
        }
      }
      for (const key in this.editRects) {
        if (this.editRects[key].collidepoint(pos)) {
          ({ template: () => this.openTemplate(), grid: () => this.toggleGrid(), undo: () => this.undo(),
            redo: () => this.redo(), clear: () => this.clear(), test: () => this.test() })[key]();
          return true;
        }
      }
      for (const [key, rc] of this.palRects) {
        if (rc.collidepoint(pos)) {
          this.tool = key;
          this.pending = null;
          this.sel = null;
          this.game.playSound("select");
          return true;
        }
      }
      if (this.paramClick(pos)) return true;
      if (this.canvas.collidepoint(pos)) this.canvasDown(pos);
      return true;
    }

    canvasDown(pos) {
      let [x, y] = this.toCourse(pos);
      x = this.clampx(x);
      y = this.clampy(y);
      const tool = this.tool;
      const g = this.game;
      if (tool === "tee" || tool === "cup") {
        this.push();
        this.hole[tool] = [x, y];
        g.playSound("click");
        return;
      }
      if (tool === "erase") {
        const hit = this.pick(x, y);
        if (hit) {
          this.push();
          this.hole[hit[0]].splice(hit[1], 1);
          this.sel = null;
          g.playSound("hit");
        }
        return;
      }
      if (tool === "select") {
        const hit = this.pick(x, y);
        this.sel = hit;
        if (hit) {
          const it = this.hole[hit[0]][hit[1]];
          this.moving = [hit[0], hit[1], it[0] - x, it[1] - y];
          g.playSound("move");
        }
        return;
      }
      const kind = PAL[tool][1];
      if (kind === "circle") {
        this.push();
        this.hole[tool].push(newItem(tool, x, y));
        this.sel = [tool, this.hole[tool].length - 1];
        g.playSound("click");
      } else if (kind === "pair") {
        if (!this.pending) {
          this.pending = [x, y]; // erster Klick: Eingang
          g.playSound("select");
        } else {
          this.push();
          this.hole[tool].push(newItem(tool, this.pending[0], this.pending[1], 0, 0, x, y));
          this.sel = [tool, this.hole[tool].length - 1];
          this.pending = null;
          g.playSound("click");
        }
      } else {
        this.drag = [tool, x, y, x, y];
      }
    }

    doMove(pos) {
      const [key, idx, offx, offy] = this.moving;
      if (idx >= this.hole[key].length) {
        this.moving = null;
        return;
      }
      const [x, y] = this.toCourse(pos);
      const it = this.hole[key][idx].slice();
      const nx = this.clampx(x + offx), ny = this.clampy(y + offy);
      if (key === "tunnels") {
        const dx = nx - it[0], dy = ny - it[1];
        it[0] = nx;
        it[1] = ny;
        it[2] = this.clampx(it[2] + dx);
        it[3] = this.clampy(it[3] + dy);
      } else {
        it[0] = nx;
        it[1] = ny;
      }
      this.hole[key][idx] = it;
      this.dirty = true;
    }

    handleUp(ev) {
      if (this.moving) {
        this.moving = null;
        return true;
      }
      if (!this.drag) return true;
      const [key, x0, y0, x1, y1] = this.drag;
      this.drag = null;
      const x = Math.min(x0, x1), y = Math.min(y0, y1);
      const w = Math.abs(x1 - x0), h = Math.abs(y1 - y0);
      if (w < 2.0 || h < 2.0) return true; // zu klein: als Fehlklick verwerfen
      this.push();
      this.hole[key].push(newItem(key, x, y, w, h));
      this.sel = [key, this.hole[key].length - 1];
      this.game.playSound("click");
      return true;
    }

    deleteSelected() {
      if (!this.sel) return;
      const [key, idx] = this.sel;
      if (idx < this.hole[key].length) {
        this.push();
        this.hole[key].splice(idx, 1);
      }
      this.sel = null;
      this.game.playSound("hit");
    }

    // ----- Kopfzeilen-Aktionen ------------------------------------------
    stepNum(key, d) {
      this.push();
      const hl = this.hole;
      if (key === "par") hl.par = Math.max(ugc.MIN_PAR, Math.min(ugc.MAX_PAR, Math.trunc(hl.par) + d));
      else if (key === "width") {
        hl.w = Math.max(ugc.MIN_W, Math.min(ugc.MAX_W, hl.w + d * ugc.STEP_W));
        this.clampAll();
        this.layout();
      } else {
        hl.h = Math.max(ugc.MIN_H, Math.min(ugc.MAX_H, hl.h + d * ugc.STEP_H));
        this.clampAll();
        this.layout();
      }
      this.game.playSound("move");
    }

    /** Nach dem Verkleinern alles ins Feld holen, was herausragt. */
    clampAll() {
      const hl = this.hole;
      const cw = hl.w, ch = hl.h;
      hl.tee = [this.clampx(hl.tee[0]), this.clampy(hl.tee[1])];
      hl.cup = [this.clampx(hl.cup[0]), this.clampy(hl.cup[1])];
      for (const key of PALETTE_KEYS) {
        hl[key] = hl[key].map((src) => {
          const it = src.slice();
          if (ROUND_KEYS.includes(key)) {
            it[0] = this.clampx(it[0]);
            it[1] = this.clampy(it[1]);
            if (key === "tunnels") {
              it[2] = this.clampx(it[2]);
              it[3] = this.clampy(it[3]);
            }
          } else {
            it[2] = Math.min(it[2], cw - 2 * BORDER);
            it[3] = Math.min(it[3], ch - 2 * BORDER);
            it[0] = Math.max(BORDER, Math.min(cw - BORDER - it[2], it[0]));
            it[1] = Math.max(BORDER, Math.min(ch - BORDER - it[3], it[1]));
          }
          return it;
        });
      }
    }

    toggleGrid() {
      this.grid = !this.grid;
      this.game.setGridSnap(this.grid);
      this.game.playSound("select");
    }

    clear() {
      this.push();
      for (const key of PALETTE_KEYS) this.hole[key] = [];
      this.sel = null;
      this.game.playSound("hit");
    }

    openTemplate() {
      this.picking = true;
      this.game.playSound("click");
    }

    handleTemplate(ev) {
      if (ev.kind === "mousedown") {
        for (const [key, rc] of this.tplRects) {
          if (rc.collidepoint(ev.pos)) {
            this.applyTemplate(key);
            return true;
          }
        }
        this.picking = false;
        return true;
      }
      if (ev.kind === "keydown" && ev.key === "Escape") this.picking = false;
      return true;
    }

    applyTemplate(key) {
      this.push();
      const tpl = template(key);
      for (const field of ["par", "tee", "cup", "w", "h"]) this.hole[field] = tpl[field];
      for (const k of PALETTE_KEYS) this.hole[k] = tpl[k].slice();
      this.sel = null;
      this.picking = false;
      this.layout();
      this.setToast(t("golf.ugc.tpl." + key));
      this.game.playSound("point");
    }

    // ----- Parameterleiste ----------------------------------------------
    /** [Typ, Eintrag, Parameterliste] des gewählten Hindernisses. */
    params() {
      if (!this.sel) return [null, null, []];
      const [key, idx] = this.sel;
      if (idx >= this.hole[key].length) {
        this.sel = null;
        return [null, null, []];
      }
      return [key, this.hole[key][idx], PAL[key][2]];
    }

    /** Rechtecke der Parameterleiste: [[param, minus, wert, plus], ...]. */
    paramRects() {
      const [, , params] = this.params();
      if (!params.length) return [];
      const w = this.game.width;
      const bh = this.parH - 10;
      const stepW = Math.max(16, Math.trunc(w * 0.036));
      const valW = Math.max(34, Math.trunc(w * 0.07));
      const grp = 2 * stepW + valW;
      const total = params.length * grp + (params.length - 1) * 8;
      let x = Math.max(10, (w - total) >> 1);
      const y = this.parTop + 5;
      return params.map((p) => {
        const out = [p, new PG.Rect(x, y, stepW, bh), new PG.Rect(x + stepW, y, valW, bh), new PG.Rect(x + stepW + valW, y, stepW, bh)];
        x += grp + 8;
        return out;
      });
    }

    paramClick(pos) {
      for (const [p, minus, , plus] of this.paramRects()) {
        if (minus.collidepoint(pos)) {
          this.stepParam(p, -1);
          return true;
        }
        if (plus.collidepoint(pos)) {
          this.stepParam(p, 1);
          return true;
        }
      }
      return false;
    }

    stepParam(p, d) {
      const [key, item] = this.params();
      if (!item) return;
      this.push();
      const it = item.slice();
      const kind = p.kind;
      if (kind === "dir" || kind === "movedir") {
        const i = PG.mod(dirIndex(it[p.idx], it[p.idx + 1]) + d, DIRS.length);
        const [ux, uy] = DIRS[i];
        if (kind === "movedir") {
          const span = Math.hypot(it[4], it[5]) || 20.0;
          it[4] = ux * span;
          it[5] = uy * span;
        } else {
          it[p.idx] = ux;
          it[p.idx + 1] = uy;
        }
      } else if (kind === "mag" || kind === "span") {
        // Rampen: Betrag der Beschleunigung / Wanderblock: Länge des Wegs -
        // die Richtung bleibt.
        const ax = it[4], ay = it[5];
        const n = Math.hypot(ax, ay) || 1.0;
        const v = Math.max(p.lo, Math.min(p.hi, n + d * p.step));
        it[4] = (ax / n) * v;
        it[5] = (ay / n) * v;
      } else {
        let v = it[p.idx] + d * p.step;
        v = Math.max(p.lo, Math.min(p.hi, v));
        it[p.idx] = Math.round(v * 100) / 100;
      }
      this.hole[key][this.sel[1]] = it;
      this.game.playSound("move");
    }

    /** Anzeigetext eines Parameters - null heißt "als Pfeil zeichnen". */
    paramText(p, item) {
      const kind = p.kind;
      if (kind === "dir" || kind === "movedir") return null;
      if (kind === "mag" || kind === "span") return String(Math.round(Math.hypot(item[4], item[5])));
      const v = item[p.idx];
      return Math.abs(v - Math.round(v)) < 0.05 ? String(Math.round(v)) : v.toFixed(1);
    }

    // ----- Speichern / Verlassen ----------------------------------------
    save() {
      const name = this.fName.text.trim();
      const mapId = this.fId.text.trim() || ugc.slug(name);
      if (!mapId) return this.fail("golf.ugc.err.id_empty");
      if (!ugc.validId(mapId)) return this.fail("golf.ugc.err.id_chars");
      if (!name) return this.fail("golf.ugc.err.name");
      const taken = ugc.loadMaps().filter((m) => m.id !== this.origId).map((m) => m.id);
      if (taken.includes(mapId)) return this.fail("golf.ugc.err.id_dup");
      const bad = validate(this.hole);
      if (bad) return this.fail("golf.ugc.err." + bad);
      if (!swear.allClean(name, mapId)) return this.fail("golf.ugc.err.swear");
      const m = Object.assign(gen.cloneHole(this.map), gen.cloneHole(this.hole));
      m.id = mapId;
      m.name = name;
      if (this.origId && this.origId !== mapId) ugc.deleteMap(this.origId); // Umbenennen = altes Feld räumen
      const [ok, why] = ugc.saveMap(m);
      if (!ok) return this.fail("golf.ugc.err." + (["swear", "full", "io", "id"].includes(why) ? why : "invalid"));
      this.map = m;
      this.origId = mapId;
      this.dirty = false;
      this.confirmBack = false;
      this.fId.setText(mapId);
      this.setToast(t("golf.ugc.saved"));
      this.game.playSound("win");
      return true;
    }

    fail(key) {
      this.err = t(key);
      this.game.playSound("hit");
      return false;
    }

    /** Bahn sofort ausprobieren - danach geht es in den Editor zurück. */
    test() {
      const bad = validate(this.hole);
      if (bad) return this.fail("golf.ugc.err." + bad);
      this.game.ugcTest(gen.cloneHole(this.hole));
      return true;
    }

    back() {
      if (this.dirty && !this.confirmBack) {
        this.confirmBack = true;
        this.setToast(t("golf.ugc.unsaved"));
        this.game.playSound("select");
        return;
      }
      this.game.ugcCloseEditor();
    }

    // ----- Zeichnen -----------------------------------------------------
    draw(ctx) {
      this.drawCanvas(ctx);
      this.drawHead(ctx);
      this.drawPalette(ctx);
      this.drawParams(ctx);
      if (this.toast) drawToast(ctx, this.toast, this.tiny, this.canvas.centerx, this.canvas.bottom - 6, 24, 10);
      if (this.picking) this.drawTemplate(ctx);
    }

    drawCanvas(ctx) {
      const view = this.view();
      mdraw.drawCourse(ctx, this.hole, view, this.t, this.t, 3.0, true);
      if (this.grid) this.drawGrid(ctx, view);
      // Auswahl hervorheben
      if (this.sel) {
        const [key, idx] = this.sel;
        if (idx < this.hole[key].length) this.outline(ctx, view, key, this.hole[key][idx], this.game.accent);
      }
      // Vorschau beim Aufziehen
      if (this.drag) {
        const [key, x0, y0, x1, y1] = this.drag;
        const rc = view.rectPx([Math.min(x0, x1), Math.min(y0, y1), Math.max(1.0, Math.abs(x1 - x0)), Math.max(1.0, Math.abs(y1 - y0))]);
        draw.rect(ctx, SWATCH[key] || ui.ACCENT, rc, 2, 3);
      }
      // Erster Klick eines Rohrs
      if (this.pending) {
        draw.circle(ctx, COL.TUNNEL, view.project(this.pending[0], this.pending[1]), Math.max(4, Math.trunc(5 * this.scale)), 2);
      }
      draw.rect(ctx, ui.BORDER, this.canvas.inflate(8, 8), 1, 8);
    }

    /** Feines Raster - nur so dicht, dass es noch hilft. */
    drawGrid(ctx, view) {
      let step = 10.0;
      while (step * this.scale < 14) step *= 2;
      const col = ui.mix(COL.GREEN, [255, 255, 255], 0.1);
      for (let x = step; x < this.hole.w; x += step) {
        const px = Math.trunc(view.project(x, 0)[0]) + 0.5;
        draw.line(ctx, col, [px, this.canvas.y], [px, this.canvas.bottom], 1);
      }
      for (let y = step; y < this.hole.h; y += step) {
        const py = Math.trunc(view.project(0, y)[1]) + 0.5;
        draw.line(ctx, col, [this.canvas.x, py], [this.canvas.right, py], 1);
      }
    }

    outline(ctx, view, key, it, col) {
      if (key === "bumpers" || key === "magnets" || key === "spinners" || key === "mills") {
        draw.circle(ctx, col, view.project(it[0], it[1]), Math.max(4, Math.trunc(it[2] * this.scale)) + 3, 2);
      } else if (key === "tunnels") {
        for (const [cx, cy] of [[it[0], it[1]], [it[2], it[3]]]) {
          draw.circle(ctx, col, view.project(cx, cy), Math.max(4, Math.trunc(it[4] * this.scale)) + 3, 2);
        }
      } else {
        draw.rect(ctx, col, view.rectPx(it).inflate(6, 6), 2, 4);
      }
    }

    drawHead(ctx) {
      const g = this.game;
      btn(ctx, this.backRect, t("golf.ugc.btn_back"), this.tiny);
      this.fName.draw(ctx, this.nameRect, this.tiny, this.focus === 0, !!this.err && this.focus === 0);
      this.fId.draw(ctx, this.idRect, this.tiny, this.focus === 1, !!this.err && this.focus === 1);
      btn(ctx, this.saveRect, t("golf.ugc.btn_save"), this.tiny, this.dirty, g.accent);
      for (const key in this.numRects) {
        const [minus, val, plus] = this.numRects[key];
        btn(ctx, minus, "-", this.tiny);
        btn(ctx, plus, "+", this.tiny);
        const num = { par: Math.trunc(this.hole.par), width: Math.trunc(this.hole.w), height: Math.trunc(this.hole.h) }[key];
        draw.rect(ctx, ui.PANEL, val, 0, 7);
        draw.rect(ctx, ui.BORDER, val, 1, 7);
        let txt = t("golf.ugc.short_" + key) + " " + num;
        if (this.tiny.width(txt) > val.w - 4) txt = String(num);
        ui.text(ctx, txt, val.centerx, val.centery, this.tiny, ui.TEXT, "center");
      }
      for (const key of TEXT_BUTTONS) btn(ctx, this.editRects[key], t("golf.ugc.btn_" + key), this.tiny, false, g.accent);
      for (const key of ICON_BUTTONS) {
        const rc = this.editRects[key];
        const on = key === "grid" && this.grid;
        const enabled = key === "undo" ? this.undoStack.length > 0 : key === "redo" ? this.redoStack.length > 0 : true;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 7);
        editIcon(ctx, rc, key, on || enabled ? ui.TEXT : ui.TEXT_FAINT);
      }
      if (this.err) ui.text(ctx, this.err, 12, this.headBottom - 2, this.tiny, ui.RED);
    }

    drawPalette(ctx) {
      const g = this.game;
      for (const [key, rc] of this.palRects) {
        const on = this.tool === key;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 6);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 6);
        if (TOOLS.includes(key)) toolIcon(ctx, rc, key, on ? ui.TEXT : ui.TEXT_DIM);
        else {
          const inner = rc.inflate(-10, -10);
          const col = SWATCH[key];
          if (ROUND_KEYS.includes(key)) draw.circle(ctx, col, inner.center, Math.max(3, Math.floor(Math.min(inner.w, inner.h) / 2)));
          else draw.rect(ctx, col, inner, 0, 3);
        }
      }
      // Name des aktiven Werkzeugs - dafür ist auf den Knöpfen kein Platz.
      const name = t((TOOLS.includes(this.tool) ? "golf.ugc.tool." : "golf.ugc.obj.") + this.tool);
      const last = this.palRects[this.palRects.length - 1][1];
      ui.text(ctx, name, last.centerx, this.parTop - 2, this.tiny, g.accent, "midbottom");
    }

    drawParams(ctx) {
      const rects = this.paramRects();
      if (!rects.length) {
        const hint = this.pending ? t("golf.ugc.hint_tunnel") : this.tool === "select" ? t("golf.ugc.hint_select") : t("golf.ugc.hint_place");
        ui.text(ctx, hint, this.game.width / 2, this.parTop + this.parH / 2, this.tiny, ui.TEXT_FAINT, "center");
        return;
      }
      const [, item] = this.params();
      for (const [p, minus, val, plus] of rects) {
        btn(ctx, minus, "-", this.tiny);
        btn(ctx, plus, "+", this.tiny);
        draw.rect(ctx, ui.PANEL, val, 0, 7);
        draw.rect(ctx, ui.BORDER, val, 1, 7);
        const txt = this.paramText(p, item);
        if (txt == null) {
          // Richtung: als Pfeil zeichnen
          const i = dirIndex(item[p.idx], item[p.idx + 1]);
          arrow(ctx, val.center, DIRS[i][0], DIRS[i][1], Math.max(4, Math.floor(val.h / 4)), ui.TEXT);
        } else {
          ui.text(ctx, txt, val.centerx, val.centery, this.tiny, ui.TEXT, "center");
        }
        ui.text(ctx, t("golf.ugc.par." + p.key), val.centerx, val.top - 1, this.tiny, ui.TEXT_FAINT, "midbottom");
      }
    }

    drawTemplate(ctx) {
      const g = this.game;
      draw.rect(ctx, [0, 0, 0, 160], [0, 0, g.width, g.height]);
      ui.drawPanel(ctx, this.tplRect, { accentTop: g.accent });
      ui.text(ctx, t("golf.ugc.tpl_title"), this.tplRect.centerx, this.tplRect.y + 10, this.small, g.accent, "midtop");
      for (const [key, rc] of this.tplRects) btn(ctx, rc, t("golf.ugc.tpl." + key), this.tiny);
    }
  }

  PG.minigolfEdit = {
    swear, ugc, TOOLS, DIRS, PALETTE, PALETTE_KEYS, TEMPLATES, dirIndex, newItem, template, validate,
    MapList, MapEditor,
  };
  PG.MinigolfEditor = MapEditor;
})();
