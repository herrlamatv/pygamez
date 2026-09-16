/*
 * chess.js - Schach gegen die KI und Schachrätsel (Port von games/chess.py + chess_draw.py)
 * ========================================================================================
 * Modi: Partie gegen KI (sechs Stärken) · Rätsel (200 Aufgaben aus der CC0-
 * Rätseldatenbank von Lichess in fünf Stufen). Web-Version: nur Einzelspieler.
 *
 * - Regeln/KI in chess_engine.js (1:1 wie Python): Chess960 mit verallgemeinerter
 *   Rochade, SAN/PGN, Suche als Generator - update() rechnet pro Frame nur ~7 ms.
 * - Setup: Stärke, Farbe, Schachuhr (ohne/1+0/3+2/5+0/10+5), Chess960.
 * - Seitenleiste: Spieler mit Uhr, geschlagene Figuren + Materialbilanz,
 *   Zugliste (SAN, Mausrad scrollt), Knöpfe Rückgängig [U]/[Backspace] (zwei
 *   Halbzüge), Hinweis [H] (Pfeil), Brett drehen [F], Remis anbieten [O],
 *   Aufgeben [X]. Rückgängig und Hinweis machen die Partie "unterstützt" -
 *   dann zählt ein Sieg nicht.
 * - Nach Partieende: [Enter] neue Partie (Farben wechseln), [S] Setup,
 *   [P] PGN herunterladen (+ Zwischenablage).
 * - Punkte (Highscore) = Siege gegen die KI ohne Unterstützung in einer Sitzung.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const E = PG.chessEngine;

  PG.addStrings({
    de: {
      "web.chess.pgn_done": "PGN heruntergeladen und kopiert",
      "web.chess.pgn_download": "PGN heruntergeladen",
    },
    en: {
      "web.chess.pgn_done": "PGN downloaded and copied",
      "web.chess.pgn_download": "PGN downloaded",
    },
    fr: { "web.chess.pgn_done": "PGN téléchargé et copié", "web.chess.pgn_download": "PGN téléchargé" },
    es: { "web.chess.pgn_done": "PGN descargado y copiado", "web.chess.pgn_download": "PGN descargado" },
    pt: { "web.chess.pgn_done": "PGN transferido e copiado", "web.chess.pgn_download": "PGN transferido" },
    pl: { "web.chess.pgn_done": "PGN pobrany i skopiowany", "web.chess.pgn_download": "PGN pobrany" },
    tr: { "web.chess.pgn_done": "PGN indirildi ve kopyalandı", "web.chess.pgn_download": "PGN indirildi" },
    da: { "web.chess.pgn_done": "PGN downloadet og kopieret", "web.chess.pgn_download": "PGN downloadet" },
    no: { "web.chess.pgn_done": "PGN lastet ned og kopiert", "web.chess.pgn_download": "PGN lastet ned" },
    sv: { "web.chess.pgn_done": "PGN nedladdad och kopierad", "web.chess.pgn_download": "PGN nedladdad" },
    fi: { "web.chess.pgn_done": "PGN ladattu ja kopioitu", "web.chess.pgn_download": "PGN ladattu" },
    cs: { "web.chess.pgn_done": "PGN stažen a zkopírován", "web.chess.pgn_download": "PGN stažen" },
    sl: { "web.chess.pgn_done": "PGN prenesen in kopiran", "web.chess.pgn_download": "PGN prenesen" },
    hr: { "web.chess.pgn_done": "PGN preuzet i kopiran", "web.chess.pgn_download": "PGN preuzet" },
  });

  // ------------------------------------------------- Brett-Identitätsfarben
  const COL_LIGHT = [236, 222, 196];
  const COL_DARK = [160, 122, 90];
  const COL_PLATE = [44, 36, 31];
  const COL_COORD = [214, 198, 172];
  const COL_SEL = [246, 214, 92];
  const COL_MOVE = [60, 150, 90];
  const COL_LAST = [232, 206, 80];
  const COL_CHECK = [232, 64, 64];
  const COL_HINT = [80, 170, 255];
  const COL_WHITE = [248, 246, 240];
  const COL_BLACK = [40, 36, 44];
  const COL_OUTLINE = [22, 18, 20];
  // U+FE0E erzwingt Text- statt Emoji-Darstellung
  const GLYPH_FILL = { 1: "♟︎", 2: "♞︎", 3: "♝︎", 4: "♜︎", 5: "♛︎", 6: "♚︎" };
  const GLYPH_LINE = { 1: "♙︎", 2: "♘︎", 3: "♗︎", 4: "♖︎", 5: "♕︎", 6: "♔︎" };
  const PIECE_FAMILY = '"Segoe UI Symbol", "DejaVu Sans", "Arial Unicode MS", "Noto Sans Symbols 2", FreeSerif, serif';

  // ----------------------------------------------------------------- Zustände
  const SETUP = "setup", PLAY = "play", OVER = "over", PUZ_MENU = "puz_menu", PUZ = "puz";
  const DIFFS = ["lvl0", "lvl1", "lvl2", "lvl3", "lvl4", "lvl5"];
  const CLOCKS = ["none", "1+0", "3+2", "5+0", "10+5"];
  const CLOCK_TIME = { "1+0": [60, 0], "3+2": [180, 2], "5+0": [300, 0], "10+5": [600, 5] };
  const FRAME_BUDGET = 0.007; // Rechenzeit der KI pro Frame (Sekunden)
  const AI_MIN_WAIT = 0.45;
  const ANIM_TIME = 0.2;
  const STAGES = ["mate1", "mate2", "mate3", "tactic1", "tactic2"];
  const MATE_N = { mate1: 1, mate2: 2, mate3: 3 };
  const PUZ_COLS = 8;
  const THEME_LABEL = ["mate", "fork", "pin", "skewer", "discoveredAttack", "doubleCheck", "hangingPiece",
    "trappedPiece", "deflection", "attraction", "sacrifice", "promotion"];
  const STORE_KEY = "chess";

  const ease = (k) => 1 - Math.pow(1 - k, 3);

  function loadPuzzles() {
    const out = {};
    for (const s of STAGES) out[s] = [];
    const data = PG.chessPuzzles;
    if (data && Array.isArray(data.stages)) {
      for (const st of data.stages) {
        if (out[st.id]) out[st.id] = (st.puzzles || []).filter((p) => Array.isArray(p.moves) && p.moves.length >= 2 && p.fen);
      }
    }
    return out;
  }

  function moveSquares(m) {
    const frm = m & 255;
    let to = (m >> 8) & 255;
    if (m >> 20 === E.M_CASTLE) to = (frm & 0x70) + (to > frm ? 6 : 2);
    return [frm, to];
  }

  class ChessGame extends PG.Game {
    get showHighscoreBanner() {
      return this.mode === "single";
    }

    get wantsEscape() {
      return (this.state === PLAY || this.state === PUZ) && (this.promo !== null || this.dialog !== null);
    }

    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      if (this.mode !== "puzzles") this.mode = "single";
      const o = this.opts;
      this.diff = Math.max(0, Math.min(5, parseInt(o.difficulty, 10)));
      if (!Number.isFinite(this.diff)) this.diff = 2;
      this.humanColor = o.color === "black" ? E.BLACK : E.WHITE;
      this.clockOpt = CLOCKS.includes(o.clock) ? o.clock : "none";
      this.c960 = o.chess960 === true;

      this.humanWins = 0;
      this.tt = new Map();
      this.aiJob = null;
      this.hintJob = null;
      this.dialog = null;
      this.promo = null;
      this.msg = null;
      this.msgT = 0;
      this.hover = null;
      this.setupFocus = 0;
      this.anims = [];
      this.fades = [];
      this.flipAnim = null;
      this.listTop = 0;
      this.listFollow = true;
      this.pieceCache = new Map();
      this.boardCache = null;
      this.boardKey = "";
      this.makeFonts();
      this.layout();
      this.buildSetupLayout();

      this.puzzles = loadPuzzles();
      this.puzLoadProgress();
      this.puz = null;
      this.puzCursor = 0;
      this.buildPuzMenuLayout();

      this.newGame();
      if (this.mode === "puzzles") {
        this.state = PUZ_MENU;
        this.puzMenuFocusUnsolved();
      } else {
        this.state = SETUP;
      }
    }

    destroy() {
      this.abortJobs();
    }

    makeFonts() {
      const h = this.height;
      this.huge = ui.font(Math.max(26, Math.floor(h / 12)), true);
      this.big = ui.font(Math.max(17, Math.floor(h / 24)), true);
      this.small = ui.font(Math.max(13, Math.floor(h / 34)));
      this.tiny = ui.font(Math.max(11, Math.floor(h / 46)));
      this.clockf = ui.font(Math.max(14, Math.floor(h / 30)), true, true);
      this.listf = ui.font(Math.max(11, Math.floor(h / 42)), false, true);
    }

    abortJobs() {
      this.aiJob = null;
      this.hintJob = null;
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    // ===================================================== Layout
    layout() {
      const w = this.width, h = this.height;
      const m = Math.max(6, Math.floor(h / 64));
      const frame = Math.max(11, Math.floor(h * 0.03));
      const sideMin = Math.max(140, Math.floor(w * 0.29));
      const cell = Math.max(14, Math.floor(Math.min((h - 2 * m - 2 * frame) / 8, (w - sideMin - 3 * m - 2 * frame) / 8)));
      this.cell = cell;
      this.bw = 8 * cell;
      const pw = this.bw + 2 * frame;
      this.frame = frame;
      this.margin = m;
      this.plate = new PG.Rect(m, Math.floor((h - pw) / 2), pw, pw);
      this.bx = this.plate.x + frame;
      this.by = this.plate.y + frame;
      this.boardRect = new PG.Rect(this.bx, this.by, this.bw, this.bw);
      const sx = this.plate.right + m;
      const sw = w - sx - m;
      this.sideRect = new PG.Rect(sx, m, sw, h - 2 * m);
      const gap = Math.max(4, Math.floor(h / 90));
      this.gap = gap;
      const ph = this.small.height + this.tiny.height + 16;
      this.topPanel = new PG.Rect(sx, m, sw, ph);
      this.botPanel = new PG.Rect(sx, h - m - ph, sw, ph);
      const n = 5;
      const bs = Math.floor(Math.max(22, Math.min(46, (sw - (n - 1) * gap) / n)));
      const by = this.botPanel.y - gap - bs;
      const total = n * bs + (n - 1) * gap;
      const bx0 = sx + Math.floor((sw - total) / 2);
      this.btnRects = {};
      ["undo", "hint", "flip", "draw", "resign"].forEach((id, i) => {
        this.btnRects[id] = new PG.Rect(bx0 + i * (bs + gap), by, bs, bs);
      });
      const sh = this.small.height + 8;
      this.statusRect = new PG.Rect(sx, this.topPanel.bottom + gap, sw, sh);
      const ly = this.statusRect.bottom + gap;
      this.listRect = new PG.Rect(sx, ly, sw, by - gap - ly);
      this.rowH = this.listf.height + 3;
      // Rätsel
      const pbs = Math.floor(Math.max(24, Math.min(48, (sw - 3 * gap) / 4)));
      const pby = h - m - pbs;
      const ptotal = 4 * pbs + 3 * gap;
      const px0 = sx + Math.floor((sw - ptotal) / 2);
      this.puzBtnRects = {};
      ["list", "retry", "solution", "next"].forEach((id, i) => {
        this.puzBtnRects[id] = new PG.Rect(px0 + i * (pbs + gap), pby, pbs, pbs);
      });
      let ih = 16 + this.big.height + 2 * (this.tiny.height + 2) + 6 + this.small.height + 4 +
        2 * (this.tiny.height + 1) + 6 + Math.max(4, Math.floor(this.tiny.height / 3)) * 2 + 10 + 2 * (this.big.height + 1);
      ih = Math.min(ih, Math.max(60, Math.floor(((pby - gap - m) * 2) / 3)));
      this.puzInfoRect = new PG.Rect(sx, m, sw, ih);
      this.puzListRect = new PG.Rect(sx, m + ih + gap, sw, pby - 2 * gap - m - ih);
      // Ergebnis-Panel über dem Brett
      const ow = Math.min(this.bw - 12, Math.max(220, Math.floor(this.bw * 0.9)));
      const bh = Math.max(24, Math.floor(h / 18));
      const pad = 10;
      const bw3 = Math.floor((ow - 2 * pad - 2 * gap) / 3);
      const labels = [t("chess.btn.new"), t("chess.btn.setup"), t("chess.btn.pgn")];
      const twoRows = labels.some((lb) => this.tiny.width(lb) > bw3 - 8);
      const btnH = twoRows ? bh * 2 + gap : bh;
      const oh = this.huge.height + 2 * this.small.height + btnH + 30;
      this.overRect = new PG.Rect(this.bx + Math.floor((this.bw - ow) / 2), this.by + Math.floor((this.bw - oh) / 2), ow, oh);
      const ox = this.overRect.x + pad, oy = this.overRect.bottom - pad - btnH;
      if (twoRows) {
        const bw2 = Math.floor((ow - 2 * pad - gap) / 2);
        this.overBtns = {
          new: new PG.Rect(ox, oy, bw2, bh),
          setup: new PG.Rect(ox + bw2 + gap, oy, bw2, bh),
          pgn: new PG.Rect(ox, oy + bh + gap, ow - 2 * pad, bh),
        };
      } else {
        this.overBtns = {};
        ["new", "setup", "pgn"].forEach((id, i) => {
          this.overBtns[id] = new PG.Rect(ox + i * (bw3 + gap), oy, bw3, bh);
        });
      }
    }

    titleBottom(y) {
      return y + Math.floor(this.huge.height / 2) + 36 + Math.floor(this.small.height / 2) + 4;
    }

    buildSetupLayout() {
      const w = this.width, h = this.height;
      const cx = Math.floor(w / 2);
      const rows = [["diff", 6], ["color", 2], ["clock", CLOCKS.length], ["c960", 2]];
      const bw = Math.min(w - 40, Math.max(460, Math.floor(w * 0.56)));
      const bh = Math.max(24, Math.min(44, Math.floor(h * 0.068)));
      const lab = this.tiny.height + 3;
      const top = Math.max(Math.floor(h * 0.25), this.titleBottom(Math.floor(h * 0.09)) + 4);
      const startH = bh + 4;
      const bottom = h - 34 - startH - Math.max(6, Math.floor(h / 50));
      const step = Math.min(bh + lab + Math.max(8, Math.floor(h / 40)), (bottom - top) / rows.length);
      const gap = 6;
      this.setupRows = [];
      let y = top;
      for (const [rid, n] of rows) {
        const yBtn = Math.floor(y + lab);
        const cw = (bw - gap * (n - 1)) / n;
        const rects = [];
        for (let i = 0; i < n; i++) rects.push(new PG.Rect(Math.floor(cx - bw / 2 + i * (cw + gap)), yBtn, Math.floor(cw), bh));
        this.setupRows.push([rid, rects]);
        y += step;
      }
      const sw = Math.min(220, w - 60);
      this.startRect = new PG.Rect(cx - Math.floor(sw / 2), Math.floor(y + Math.max(4, h / 60)), sw, startH);
    }

    buildPuzMenuLayout() {
      const w = this.width, h = this.height;
      const m = Math.max(10, Math.floor(h / 40));
      const top = this.titleBottom(Math.floor(h * 0.075)) + Math.max(4, Math.floor(h / 60));
      const tw = Math.min(w - 2 * m, Math.max(640, Math.floor(w * 0.78)));
      const gap = Math.max(4, Math.floor(h / 100));
      const th = Math.max(30, Math.floor(h * 0.09));
      const n = STAGES.length;
      const tcw = (tw - gap * (n - 1)) / n;
      const x0 = (w - tw) / 2;
      this.stageRects = [];
      for (let i = 0; i < n; i++) this.stageRects.push(new PG.Rect(Math.floor(x0 + i * (tcw + gap)), top, Math.floor(tcw), th));
      const gy = top + th + Math.max(8, Math.floor(h / 40));
      const rows = 5;
      const startH = Math.max(24, Math.min(40, Math.floor(h / 14)));
      const foot = 38 + startH + Math.max(4, Math.floor(h / 90));
      const ch = Math.floor(Math.min((h - gy - foot - gap * (rows - 1)) / rows, 60));
      const cw = Math.floor(Math.min((tw - gap * (PUZ_COLS - 1)) / PUZ_COLS, ch * 1.9));
      const gw = PUZ_COLS * cw + (PUZ_COLS - 1) * gap;
      const gx = Math.floor((w - gw) / 2);
      this.gridRects = [];
      for (let i = 0; i < PUZ_COLS * rows; i++) {
        this.gridRects.push(new PG.Rect(gx + (i % PUZ_COLS) * (cw + gap), gy + Math.floor(i / PUZ_COLS) * (ch + gap), cw, ch));
      }
      const by = gy + rows * (ch + gap) + Math.max(2, Math.floor(h / 90));
      const bw = Math.min(240, w - 60);
      this.puzStartRect = new PG.Rect(Math.floor((w - bw) / 2), by, bw, startH);
    }

    // ===================================================== Partie aufbauen
    newGame() {
      this.abortJobs();
      if (this.c960 && this.mode === "single") {
        this.c960N = PG.rand.randint(0, 959);
        this.pos = E.fromFen(E.chess960Fen(this.c960N), true);
      } else {
        this.c960N = null;
        this.pos = E.fromFen(E.START_FEN, false);
      }
      this.startFen = this.pos.fen();
      this.startSide = this.pos.side;
      this.startFull = this.pos.full;
      this.moves = [];
      this.sans = [];
      this.ucis = [];
      this.caps = [];
      this.lastMove = null;
      this.result = null;
      this.resultLoser = null;
      this.assisted = false;
      this.sel = null;
      this.targets = new Map();
      this.drag = null;
      this.hintMove = 0;
      this.dialog = null;
      this.promo = null;
      this.drawBlock = 0;
      this.aiWait = 0;
      this.aiLastScore = null;
      this.lowTimeWarned = [false, false];
      if (CLOCK_TIME[this.clockOpt]) {
        const [base, inc] = CLOCK_TIME[this.clockOpt];
        this.clock = [base, base];
        this.inc = inc;
      } else {
        this.clock = null;
        this.inc = 0;
      }
      this.viewBlack = this.humanColor === E.BLACK;
      this.cursor = [6, 4];
      this.anims = [];
      this.fades = [];
      this.listTop = 0;
      this.listFollow = true;
      this.refresh();
    }

    refresh() {
      const pos = this.pos;
      this.legal = pos.legalMoves();
      this.byFrom = new Map();
      for (const m of this.legal) {
        const f = m & 255;
        if (!this.byFrom.has(f)) this.byFrom.set(f, []);
        this.byFrom.get(f).push(m);
      }
      this.check = pos.inCheck();
    }

    // ===================================================== Koordinaten
    sqToDisp(sq) {
      const r = sq >> 4, f = sq & 7;
      return this.viewBlack ? [r, 7 - f] : [7 - r, f];
    }

    dispToSq(dr, dc) {
      return this.viewBlack ? dr * 16 + (7 - dc) : (7 - dr) * 16 + dc;
    }

    sqRect(sq) {
      const [dr, dc] = this.sqToDisp(sq);
      return new PG.Rect(this.bx + dc * this.cell, this.by + dr * this.cell, this.cell, this.cell);
    }

    sqAt(pos) {
      const c = Math.floor((pos[0] - this.bx) / this.cell);
      const r = Math.floor((pos[1] - this.by) / this.cell);
      if (r >= 0 && r < 8 && c >= 0 && c < 8) return this.dispToSq(r, c);
      return null;
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) this.handleSetup(ev);
      else if (this.state === PUZ_MENU) this.handlePuzMenu(ev);
      else if (this.state === OVER) this.handleOver(ev);
      else if (this.state === PLAY || this.state === PUZ) this.handleBoardState(ev);
    }

    fixedKey(key, ...names) {
      return names.includes(key) && this.keyIsFree(key);
    }

    setupValues(rid) {
      if (rid === "diff") return [6, this.diff];
      if (rid === "color") return [2, this.humanColor];
      if (rid === "clock") return [CLOCKS.length, CLOCKS.indexOf(this.clockOpt)];
      return [2, this.c960 ? 1 : 0];
    }

    setupSet(rid, i) {
      if (rid === "diff") {
        this.diff = i;
        this.saveSetting("difficulty", i);
      } else if (rid === "color") {
        this.humanColor = i;
        this.saveSetting("color", i === E.BLACK ? "black" : "white");
      } else if (rid === "clock") {
        this.clockOpt = CLOCKS[i];
        this.saveSetting("clock", this.clockOpt);
      } else if (rid === "c960") {
        this.c960 = i === 1;
        this.saveSetting("chess960", this.c960);
      }
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        const nRows = this.setupRows.length;
        if (["1", "2", "3", "4", "5", "6"].includes(k)) {
          this.setupSet("diff", Number(k) - 1);
          this.playSound("click");
        } else if (k === "c" || k === "C") {
          this.setupSet("color", 1 - this.humanColor);
          this.playSound("select");
        } else if (this.isAction(k, "up")) {
          this.setupFocus = PG.mod(this.setupFocus - 1, nRows + 1);
          this.playSound("move");
        } else if (this.isAction(k, "down")) {
          this.setupFocus = PG.mod(this.setupFocus + 1, nRows + 1);
          this.playSound("move");
        } else if (this.isAction(k, "left") || this.isAction(k, "right")) {
          if (this.setupFocus < nRows) {
            const rid = this.setupRows[this.setupFocus][0];
            const [n, cur] = this.setupValues(rid);
            this.setupSet(rid, PG.mod(cur + (this.isAction(k, "left") ? -1 : 1), n));
            this.playSound("move");
          }
        } else if (k === "Return" || k === "space" || this.isAction(k, "action")) {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let ri = 0; ri < this.setupRows.length; ri++) {
          const [rid, rects] = this.setupRows[ri];
          for (let i = 0; i < rects.length; i++) {
            if (rects[i].collidepoint(ev.pos)) {
              this.setupFocus = ri;
              this.setupSet(rid, i);
              this.playSound("click");
              return;
            }
          }
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    startPlay() {
      this.gameOver = false;
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
      ui.beginTransition();
    }

    handleBoardState(ev) {
      if (this.promo !== null) return this.handlePromo(ev);
      if (this.dialog !== null) return this.handleDialog(ev);
      const k = ev.kind === "keydown" ? ev.key : null;
      if (ev.kind === "mousemove") this.hover = this.buttonAt(ev.pos);
      if (ev.kind === "wheel") {
        const rect = this.state === PLAY ? this.listRect : this.puzListRect;
        if (rect.collidepoint(ev.pos)) this.scrollList(-ev.delta * 2);
        return;
      }
      if (ev.kind === "mousedown") {
        const bid = this.buttonAt(ev.pos);
        if (bid) return this.pressButton(bid);
      }
      if (k !== null) {
        if (this.state === PLAY) {
          if (this.fixedKey(k, "u", "U", "BackSpace")) return this.undo();
          if (this.fixedKey(k, "h", "H")) return this.requestHint();
          if (this.fixedKey(k, "f", "F")) return this.flipView();
          if (this.fixedKey(k, "o", "O")) return this.offerDraw();
          if (this.fixedKey(k, "x", "X")) return this.askResign();
          if (k === "Prior" || k === "Next") return this.scrollList(k === "Prior" ? -6 : 6);
        } else {
          if (this.fixedKey(k, "h", "H")) return this.puzShowSolution();
          if (this.fixedKey(k, "u", "U", "BackSpace")) return this.puzOpen(this.puzStage, this.puzIdx);
          if (this.fixedKey(k, "m", "M", "Tab")) return this.puzBackToMenu();
          if (this.fixedKey(k, "n", "N")) return this.puzNext();
          if ((this.puzStatus === "solved" || this.puzStatus === "shown_done") &&
              (k === "Return" || k === "space" || this.isAction(k, "action"))) return this.puzNext();
        }
      }
      if (!this.humanCanMove()) {
        if (ev.kind === "mouseup") this.drag = null;
        return;
      }
      if (ev.kind === "mousemove") {
        const sq = this.sqAt(ev.pos);
        if (sq !== null) this.cursor = this.sqToDisp(sq);
        if (this.drag) {
          this.drag.pos = ev.pos;
          if (Math.abs(ev.pos[0] - this.drag.start[0]) + Math.abs(ev.pos[1] - this.drag.start[1]) > 4) this.drag.moved = true;
        }
      } else if (ev.kind === "mousedown") {
        const sq = this.sqAt(ev.pos);
        if (sq === null) {
          this.sel = null;
          this.targets = new Map();
          return;
        }
        this.cursor = this.sqToDisp(sq);
        if (this.sel !== null && this.targets.has(sq)) return this.tryMove(sq, true);
        const p = this.pos.b[sq];
        if (p && (p >> 3) === this.pos.side) {
          const was = this.sel === sq;
          this.select(sq);
          this.drag = { sq, pos: ev.pos, start: ev.pos, moved: false, was };
          this.playSound("click");
        } else {
          this.sel = null;
          this.targets = new Map();
        }
      } else if (ev.kind === "mouseup") {
        const drag = this.drag;
        this.drag = null;
        if (!drag) return;
        const sq = this.sqAt(ev.pos);
        if (sq !== null && sq !== drag.sq && this.targets.has(sq)) this.tryMove(sq, false);
        else if (sq === drag.sq && drag.was && !drag.moved) {
          this.sel = null;
          this.targets = new Map();
        }
      } else if (k !== null) {
        if (this.isAction(k, "up")) this.moveCursor(-1, 0);
        else if (this.isAction(k, "down")) this.moveCursor(1, 0);
        else if (this.isAction(k, "left")) this.moveCursor(0, -1);
        else if (this.isAction(k, "right")) this.moveCursor(0, 1);
        else if (k === "Return" || k === "space" || this.isAction(k, "action")) {
          const sq = this.dispToSq(this.cursor[0], this.cursor[1]);
          if (this.sel !== null && this.targets.has(sq)) this.tryMove(sq, true);
          else {
            const p = this.pos.b[sq];
            if (p && (p >> 3) === this.pos.side && this.sel !== sq) {
              this.select(sq);
              this.playSound("click");
            } else {
              this.sel = null;
              this.targets = new Map();
            }
          }
        }
      }
    }

    moveCursor(dr, dc) {
      this.cursor = [Math.max(0, Math.min(7, this.cursor[0] + dr)), Math.max(0, Math.min(7, this.cursor[1] + dc))];
      this.playSound("move");
    }

    buttonAt(pos) {
      const rects = this.state === PLAY ? this.btnRects : this.puzBtnRects;
      for (const id in rects) if (rects[id].collidepoint(pos)) return id;
      return null;
    }

    pressButton(bid) {
      if (bid === "undo") this.undo();
      else if (bid === "hint") this.requestHint();
      else if (bid === "flip") this.flipView();
      else if (bid === "draw") this.offerDraw();
      else if (bid === "resign") this.askResign();
      else if (bid === "list") this.puzBackToMenu();
      else if (bid === "retry") this.puzOpen(this.puzStage, this.puzIdx);
      else if (bid === "solution") this.puzShowSolution();
      else if (bid === "next") this.puzNext();
    }

    humanCanMove() {
      if (this.state === PUZ) return this.puzStatus === "play" && this.pos.side === this.puzPlayer;
      if (this.state !== PLAY) return false;
      return this.pos.side === this.humanColor;
    }

    select(sq) {
      this.sel = sq;
      const moves = this.byFrom.get(sq) || [];
      const normal = new Set(moves.filter((m) => m >> 20 !== E.M_CASTLE).map((m) => (m >> 8) & 255));
      const targets = new Map();
      const add = (to, m) => {
        if (!targets.has(to)) targets.set(to, []);
        targets.get(to).push(m);
      };
      for (const m of moves) {
        const to = (m >> 8) & 255;
        if (m >> 20 === E.M_CASTLE) {
          // Rochade: König auf den eigenen Turm ziehen - oder aufs Zielfeld,
          // solange das kein normaler Königszug ist.
          add(to, m);
          const kt = (sq & 0x70) + (to > sq ? 6 : 2);
          if (kt !== sq && kt !== to && !normal.has(kt)) add(kt, m);
        } else add(to, m);
      }
      this.targets = targets;
    }

    tryMove(sq, animate) {
      const cands = this.targets.get(sq) || [];
      if (!cands.length) return;
      if (cands.length > 1) {
        const order = { [E.QUEEN]: 0, [E.ROOK]: 1, [E.BISHOP]: 2, [E.KNIGHT]: 3 };
        this.promo = cands.slice().sort((a, b) => (order[(a >> 16) & 7] ?? 9) - (order[(b >> 16) & 7] ?? 9));
        this.promoAnimate = animate;
        this.playSound("select");
        return;
      }
      this.humanMove(cands[0], animate);
    }

    humanMove(m, animate) {
      if (this.state === PUZ) this.puzPlayerMove(m, animate);
      else this.commit(m, animate);
    }

    promoRects() {
      const n = 4;
      const size = Math.max(36, Math.min(Math.floor(this.cell * 1.15), Math.floor((this.bw - 40) / n)));
      const gap = Math.max(6, Math.floor(size / 8));
      const total = size * n + gap * (n - 1);
      const x0 = this.bx + Math.floor((this.bw - total) / 2);
      const y = this.by + Math.floor(this.bw / 2 - size / 2 + this.small.height / 2);
      const out = [];
      for (let i = 0; i < n; i++) out.push(new PG.Rect(x0 + i * (size + gap), y, size, size));
      return out;
    }

    handlePromo(ev) {
      if (ev.kind === "mousedown") {
        const rects = this.promoRects();
        for (let i = 0; i < rects.length; i++) {
          if (rects[i].collidepoint(ev.pos)) return this.finishPromo(i);
        }
        if (this.boardRect.collidepoint(ev.pos)) this.promo = null;
      } else if (ev.kind === "keydown" && ev.key) {
        const k = ev.key.toLowerCase();
        const keys = { q: 0, d: 0, r: 1, t: 1, b: 2, l: 2, n: 3, s: 3, 1: 0, 2: 1, 3: 2, 4: 3 };
        if (k in keys) this.finishPromo(keys[k]);
        else if (k === "escape" || k === "backspace") this.promo = null;
      }
    }

    finishPromo(i) {
      const cands = this.promo;
      this.promo = null;
      if (cands && i >= 0 && i < cands.length) this.humanMove(cands[i], this.promoAnimate !== false);
    }

    // ----------------------------------------------------------- Dialoge
    dialogRects() {
      const w = Math.min(this.bw - 16, Math.max(240, Math.floor(this.bw * 0.8)));
      const h = this.big.height + this.small.height + Math.max(30, Math.floor(this.height / 15)) + 40;
      const panel = new PG.Rect(this.bx + Math.floor((this.bw - w) / 2), this.by + Math.floor((this.bw - h) / 2), w, h);
      const bh = Math.max(26, Math.floor(this.height / 16));
      const bw = Math.floor((w - 36) / 2);
      return [panel, new PG.Rect(panel.x + 12, panel.bottom - 12 - bh, bw, bh), new PG.Rect(panel.right - 12 - bw, panel.bottom - 12 - bh, bw, bh)];
    }

    handleDialog(ev) {
      let answer = null;
      if (ev.kind === "mousedown") {
        const [, yes, no] = this.dialogRects();
        if (yes.collidepoint(ev.pos)) answer = true;
        else if (no.collidepoint(ev.pos)) answer = false;
      } else if (ev.kind === "keydown") {
        const k = ev.key;
        if (["Return", "KP_Enter", "j", "J", "y", "Y"].includes(k)) answer = true;
        else if (["Escape", "n", "N", "BackSpace"].includes(k)) answer = false;
      }
      if (answer === null) return;
      const [kind, side] = this.dialog;
      this.dialog = null;
      this.playSound("click");
      if (kind === "resign" && answer) this.finish(1 - side, "resign");
    }

    handleOver(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Return" || k === "space" || k === "KP_Enter") this.restart();
        else if (k === "s" || k === "S") this.toSetup();
        else if (this.fixedKey(k, "p", "P")) this.exportPgn();
        else if (this.fixedKey(k, "f", "F")) this.flipView();
        else if (["Prior", "Next", "Up", "Down"].includes(k)) this.scrollList(k === "Prior" || k === "Up" ? -3 : 3);
      } else if (ev.kind === "mousedown") {
        for (const id in this.overBtns) {
          if (this.overBtns[id].collidepoint(ev.pos)) {
            if (id === "new") this.restart();
            else if (id === "setup") this.toSetup();
            else this.exportPgn();
            return;
          }
        }
      } else if (ev.kind === "wheel") {
        if (this.listRect.collidepoint(ev.pos)) this.scrollList(-ev.delta * 2);
      } else if (ev.kind === "mousemove") {
        this.hover = null;
        for (const id in this.overBtns) if (this.overBtns[id].collidepoint(ev.pos)) this.hover = "over_" + id;
      }
    }

    restart() {
      this.gameOver = false;
      this.humanColor = 1 - this.humanColor; // Farben wechseln nach jeder Partie
      this.saveSetting("color", this.humanColor === E.BLACK ? "black" : "white");
      this.newGame();
      this.state = PLAY;
      this.playSound("click");
    }

    toSetup() {
      this.gameOver = false;
      this.state = SETUP;
      this.playSound("click");
    }

    listRows() {
      return Math.floor((this.sans.length + this.startSide + 1) / 2);
    }

    listVisible() {
      const rect = this.state === PUZ ? this.puzListRect : this.listRect;
      return Math.max(1, Math.floor((rect.h - (this.tiny.height + 10)) / this.rowH));
    }

    scrollList(rows) {
      const total = this.listRows(), visible = this.listVisible();
      const top = Math.max(0, Math.min(total - visible, this.listTop + rows));
      this.listTop = top;
      this.listFollow = top >= total - visible;
    }

    scrollToEnd() {
      this.listTop = Math.max(0, this.listRows() - this.listVisible());
      this.listFollow = true;
    }

    // ===================================================== Züge
    commit(m, animate = true) {
      const pos = this.pos;
      const mover = pos.side;
      const to = (m >> 8) & 255;
      const flag = m >> 20;
      let cap, capSq;
      if (flag === E.M_EP) {
        cap = E.PAWN | ((1 - mover) << 3);
        capSq = mover === E.WHITE ? to - 16 : to + 16;
      } else if (flag === E.M_CASTLE) {
        cap = 0;
        capSq = -1;
      } else {
        cap = pos.b[to];
        capSq = to;
      }
      const san = pos.san(m, this.legal);
      if (animate) this.animate(m);
      if (cap) this.fades.push({ p: cap, sq: capSq, t0: ui.now() });
      this.sans.push(san);
      this.ucis.push(pos.uci(m));
      pos.make(m);
      this.moves.push(m);
      this.caps.push(cap);
      this.lastMove = moveSquares(m);
      this.sel = null;
      this.targets = new Map();
      this.drag = null;
      this.hintMove = 0;
      this.hintJob = null;
      if (this.clock && this.moves.length > 1) this.clock[mover] += this.inc;
      this.refresh();
      if (this.listFollow) this.scrollToEnd();
      if (this.check) this.playSound("select");
      else if (flag === E.M_CASTLE) this.playSound("rotate");
      else if (cap) {
        this.playSound("lock");
        this.rumble(40);
      } else this.playSound("move");
      this.aiWait = 0;
      if (this.state !== PLAY) return;
      let status = null;
      if (!this.legal.length) status = this.check ? "checkmate" : "stalemate";
      else if (pos.insufficientMaterial()) status = "material";
      else if (pos.half >= 100) status = "fifty";
      else if (pos.repetitions() >= 3) status = "threefold";
      if (status) this.finish(status === "checkmate" ? mover : null, status);
    }

    animate(m) {
      const pos = this.pos;
      const frm = m & 255, to = (m >> 8) & 255;
      const t0 = ui.now();
      if (m >> 20 === E.M_CASTLE) {
        const base = frm & 0x70;
        const [kt, rt] = to > frm ? [base + 6, base + 5] : [base + 2, base + 3];
        this.anims = [{ p: pos.b[frm], a: frm, b: kt, t0 }, { p: pos.b[to], a: to, b: rt, t0 }];
      } else {
        const promo = (m >> 16) & 7;
        const p = promo ? promo | (pos.side << 3) : pos.b[frm];
        this.anims = [{ p, a: frm, b: to, t0 }];
      }
    }

    undo() {
      if (this.state !== PLAY || !this.moves.length) return;
      const humanMoves = this.moves.filter((_, i) => i % 2 === this.humanColor).length;
      if (humanMoves === 0) {
        this.toast(t("chess.undo_none"));
        return;
      }
      const n = this.pos.side === this.humanColor ? 2 : 1;
      this.abortJobs();
      for (let i = 0; i < Math.min(n, this.moves.length); i++) {
        this.pos.unmake();
        this.moves.pop();
        this.sans.pop();
        this.ucis.pop();
        this.caps.pop();
      }
      this.toast(this.assisted ? t("chess.undone") : t("chess.assisted"));
      this.assisted = true;
      this.lastMove = this.moves.length ? moveSquares(this.moves[this.moves.length - 1]) : null;
      this.sel = null;
      this.targets = new Map();
      this.hintMove = 0;
      this.anims = [];
      this.fades = [];
      this.aiWait = 0;
      this.drawBlock = Math.min(this.drawBlock, this.moves.length);
      this.refresh();
      this.scrollToEnd();
      this.playSound("rotate");
    }

    flipView() {
      this.viewBlack = !this.viewBlack;
      this.flipAnim = ui.now();
      this.playSound("rotate");
    }

    // ----------------------------------------------------------- Hinweis
    requestHint() {
      if (this.state !== PLAY || this.hintJob || !this.legal.length) return;
      if (this.pos.side !== this.humanColor) return;
      if (!this.assisted) this.toast(t("chess.assisted"));
      this.assisted = true;
      const s = new E.Search(this.pos, E.HINT_LEVEL, { tt: this.tt, rng: () => PG.rand.random() });
      this.hintJob = { search: s, gen: s.run(), spent: 0 };
      this.playSound("select");
    }

    stepHint() {
      const job = this.hintJob;
      const s = job.search;
      if (!s.done) {
        const t0 = E.now();
        s.sliceEnd = t0 + FRAME_BUDGET;
        job.gen.next();
        job.spent += E.now() - t0;
      }
      if (s.done || (job.spent >= E.HINT_LEVEL.time && s.depthDone >= 1)) {
        this.hintJob = null;
        if (s.bestMove) {
          this.hintMove = s.bestMove;
          this.toast(t("chess.hint_move", { move: this.pos.san(s.bestMove, this.legal) }));
        }
      }
    }

    // ----------------------------------------------------------- Remis / Aufgeben
    offerDraw() {
      if (this.state !== PLAY || !this.moves.length) return;
      if (this.moves.length < this.drawBlock) {
        this.toast(t("chess.draw_wait"));
        return;
      }
      const ai = 1 - this.humanColor;
      let score;
      if (this.aiLastScore !== null) score = this.aiLastScore;
      else {
        const e = this.pos.evaluate();
        score = this.pos.side === ai ? e : -e;
      }
      if (score <= 20) {
        this.toast(t("chess.draw_accepted"));
        this.finish(null, "agreed");
      } else {
        this.drawBlock = this.moves.length + 6;
        this.toast(t("chess.draw_declined_ai"));
        this.playSound("hit");
      }
    }

    askResign() {
      if (this.state !== PLAY) return;
      this.dialog = ["resign", this.humanColor];
      this.playSound("select");
    }

    // ----------------------------------------------------------- Ende
    finish(winner, reason, loser = null) {
      this.abortJobs();
      this.state = OVER;
      this.result = [winner, reason];
      this.resultLoser = loser !== null ? loser : winner !== null ? 1 - winner : null;
      this.dialog = null;
      this.promo = null;
      this.sel = null;
      this.targets = new Map();
      this.drag = null;
      this.overT0 = ui.now();
      if (reason === "checkmate") {
        const r = this.sqRect(this.pos.kings[1 - winner]);
        ui.spawnBurst(r.centerx, r.centery, ui.GOLD, 30);
      }
      if (winner === this.humanColor) {
        if (!this.assisted) {
          this.humanWins += 1;
          this.score = this.humanWins;
          this.reportResult(true);
          this.achEvent("chess_win");
          if (this.diff === 5) this.achEvent("chess_master");
        }
        this.playSound("win");
      } else if (winner === null) {
        this.playSound("select");
      } else {
        this.reportResult(false);
        this.playSound("gameover");
      }
      this.gameOver = true;
    }

    pgn() {
      const [winner, reason] = this.result || ["*", null];
      const res = E.pgnResultCode(winner);
      const ai = t("common.ai") + " (" + t("chess.diff." + DIFFS[this.diff]) + ")";
      const you = t("chess.you");
      const [white, black] = this.humanColor === E.WHITE ? [you, ai] : [ai, you];
      const d = new Date();
      const pad = (n) => String(n).padStart(2, "0");
      const tags = [["Event", "PyGameZ Web"], ["Site", "PyGameZ"], ["Date", d.getFullYear() + "." + pad(d.getMonth() + 1) + "." + pad(d.getDate())],
        ["Round", "-"], ["White", white], ["Black", black], ["Result", res]];
      if (this.c960N !== null) tags.push(["Variant", "Chess960"], ["SetUp", "1"], ["FEN", this.startFen]);
      if (CLOCK_TIME[this.clockOpt]) tags.push(["TimeControl", CLOCK_TIME[this.clockOpt][0] + "+" + CLOCK_TIME[this.clockOpt][1]]);
      if (reason === "timeout" || reason === "timeout_draw") tags.push(["Termination", "time forfeit"]);
      return E.pgnText(tags, this.sans, res);
    }

    exportPgn() {
      if (!this.sans.length) {
        this.toast(t("chess.pgn_empty"));
        return;
      }
      const text = this.pgn();
      const d = new Date();
      const pad = (n) => String(n).padStart(2, "0");
      const name = "pygamez-chess-" + d.getFullYear() + "-" + pad(d.getMonth() + 1) + "-" + pad(d.getDate()) + "-" + pad(d.getHours()) + pad(d.getMinutes()) + ".pgn";
      try {
        PG.downloadText(name, text, "application/x-chess-pgn");
      } catch (e) {
        /* Download nicht möglich - Zwischenablage reicht */
      }
      const done = () => this.toast(t("web.chess.pgn_done"), 3.5);
      const fallback = () => {
        try {
          const ta = document.createElement("textarea");
          ta.value = text;
          ta.style.position = "fixed";
          ta.style.opacity = "0";
          document.body.appendChild(ta);
          ta.select();
          const ok = document.execCommand("copy");
          ta.remove();
          this.toast(ok ? t("web.chess.pgn_done") : t("web.chess.pgn_download"), 3.5);
        } catch (e) {
          this.toast(t("web.chess.pgn_download"), 3.5);
        }
      };
      if (typeof navigator !== "undefined" && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, fallback);
      } else fallback();
      this.playSound("point");
    }

    toast(text, dur = 2.6) {
      this.msg = text;
      this.msgT = dur;
    }

    // ===================================================== Update
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.state === PLAY) {
        if (this.clock && this.moves.length && this.dialog === null) {
          this.tickClock(dt);
          if (this.state !== PLAY) return;
        }
        if (this.hintJob) this.stepHint();
        if (this.pos.side !== this.humanColor && this.dialog === null) this.stepAi(dt);
      } else if (this.state === PUZ) {
        this.puzUpdate(dt);
      }
    }

    tickClock(dt) {
      const side = this.pos.side;
      this.clock[side] -= dt;
      if (this.clock[side] <= 10 && !this.lowTimeWarned[side] && this.clock[side] > 0) {
        this.lowTimeWarned[side] = true;
        if (side === this.humanColor) this.playSound("hit");
      }
      if (this.clock[side] <= 0) {
        this.clock[side] = 0;
        const winner = 1 - side;
        if (this.pos.canMate(winner)) this.finish(winner, "timeout");
        else this.finish(null, "timeout_draw", side);
      }
    }

    stepAi(dt) {
      this.aiWait += dt;
      let job = this.aiJob;
      if (!job) {
        if (this.aiWait < 0.12 || !this.legal.length) return;
        job = this.startAi();
      }
      if (job.move) {
        if (this.aiWait >= job.wait) {
          this.aiJob = null;
          this.commit(job.move);
        }
        return;
      }
      const s = job.search;
      if (!s.done) {
        const t0 = E.now();
        s.sliceEnd = t0 + FRAME_BUDGET;
        job.gen.next();
        job.spent += E.now() - t0;
      }
      const lvl = E.LEVELS[this.diff];
      const out = (job.spent >= lvl.time && s.depthDone >= 1) || (this.aiWait >= job.wallCap && s.bestMove);
      if ((s.done || out) && this.aiWait >= AI_MIN_WAIT) {
        const m = s.bestMove || this.legal[0];
        if (s.depthDone >= 1) this.aiLastScore = s.bestScore;
        this.abortJobs();
        this.commit(m);
      }
    }

    startAi() {
      const lvl = E.LEVELS[this.diff];
      const pos = this.pos;
      const rng = () => PG.rand.random();
      const job = { move: 0, wait: AI_MIN_WAIT, search: null, gen: null, spent: 0, wallCap: Infinity };
      if (this.diff === 0 || PG.rand.random() < lvl.random) {
        job.move = E.pickWeakMove(pos, rng);
        job.wait = PG.rand.uniform(0.5, 0.9);
      } else if (lvl.book && this.c960N === null && this.moves.length < 16) {
        const cands = E.bookMoves(this.ucis).map((u) => pos.parseUci(u)).filter(Boolean);
        if (cands.length) {
          job.move = PG.rand.choice(cands);
          job.wait = PG.rand.uniform(0.45, 0.8);
        }
      }
      if (!job.move) {
        const s = new E.Search(pos, lvl, { tt: this.tt, rng });
        job.search = s;
        job.gen = s.run();
        if (this.clock) job.wallCap = Math.max(0.2, Math.min(10, this.clock[pos.side] / 30 + this.inc * 0.7));
      }
      this.aiJob = job;
      return job;
    }

    // ===================================================== Rätsel
    puzLoadProgress() {
      const data = PG.store.get(STORE_KEY, {}) || {};
      this.puzSolved = new Set(Array.isArray(data.puzzles_solved) ? data.puzzles_solved.filter((x) => typeof x === "string") : []);
      this.puzSeen = new Set(Array.isArray(data.puzzles_seen) ? data.puzzles_seen.filter((x) => typeof x === "string") : []);
      const st = data.puzzle_stage;
      this.puzStage = Number.isInteger(st) && st >= 0 && st < STAGES.length ? st : 0;
    }

    puzSaveProgress() {
      const data = PG.store.get(STORE_KEY, {}) || {};
      data.puzzles_solved = [...this.puzSolved].sort();
      data.puzzles_seen = [...this.puzSeen].sort();
      data.puzzle_stage = this.puzStage;
      PG.store.set(STORE_KEY, data);
    }

    stageList(stage) {
      return this.puzzles[STAGES[stage == null ? this.puzStage : stage]] || [];
    }

    stageSolved(stage) {
      return this.stageList(stage).filter((p) => this.puzSolved.has(p.id)).length;
    }

    puzMenuFocusUnsolved() {
      const lst = this.stageList();
      this.puzCursor = 0;
      for (let i = 0; i < lst.length; i++) {
        if (!this.puzSolved.has(lst[i].id)) {
          this.puzCursor = i;
          break;
        }
      }
    }

    handlePuzMenu(ev) {
      const lst = this.stageList();
      const n = lst.length;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (["1", "2", "3", "4", "5"].includes(k)) this.puzSetStage(Number(k) - 1);
        else if (k === "Tab" || k === "Next" || k === "Prior") this.puzSetStage(PG.mod(this.puzStage + (k === "Prior" ? -1 : 1), STAGES.length));
        else if (n && this.isAction(k, "left")) {
          this.puzCursor = PG.mod(this.puzCursor - 1, n);
          this.playSound("move");
        } else if (n && this.isAction(k, "right")) {
          this.puzCursor = PG.mod(this.puzCursor + 1, n);
          this.playSound("move");
        } else if (n && this.isAction(k, "up")) {
          this.puzCursor = PG.mod(this.puzCursor - PUZ_COLS, n);
          this.playSound("move");
        } else if (n && this.isAction(k, "down")) {
          this.puzCursor = PG.mod(this.puzCursor + PUZ_COLS, n);
          this.playSound("move");
        } else if (n && (k === "Return" || k === "space" || this.isAction(k, "action"))) {
          this.puzOpen(this.puzStage, this.puzCursor);
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.stageRects.length; i++) {
          if (this.stageRects[i].collidepoint(ev.pos)) return this.puzSetStage(i);
        }
        for (let i = 0; i < Math.min(n, this.gridRects.length); i++) {
          if (this.gridRects[i].collidepoint(ev.pos)) {
            this.puzCursor = i;
            return this.puzOpen(this.puzStage, i);
          }
        }
        if (n && this.puzStartRect.collidepoint(ev.pos)) this.puzOpen(this.puzStage, this.puzCursor);
      } else if (ev.kind === "mousemove") {
        this.hover = null;
        for (let i = 0; i < Math.min(n, this.gridRects.length); i++) {
          if (this.gridRects[i].collidepoint(ev.pos)) this.hover = "cell" + i;
        }
      }
    }

    puzSetStage(i) {
      if (i === this.puzStage) return;
      this.puzStage = i;
      this.puzMenuFocusUnsolved();
      this.playSound("select");
    }

    puzOpen(stage, idx) {
      const lst = this.stageList(stage);
      if (!lst.length) return;
      idx = Math.max(0, Math.min(lst.length - 1, idx));
      const p = lst[idx];
      let pos;
      try {
        pos = E.fromFen(p.fen, false);
      } catch (e) {
        return;
      }
      this.abortJobs();
      this.puz = p;
      this.puzStage = stage;
      this.puzIdx = idx;
      this.puzCursor = idx;
      this.pos = pos;
      this.startFen = pos.fen();
      this.startSide = pos.side;
      this.startFull = pos.full;
      this.listTop = 0;
      this.listFollow = true;
      this.c960N = null;
      this.clock = null;
      this.moves = [];
      this.sans = [];
      this.ucis = [];
      this.caps = [];
      this.lastMove = null;
      this.sel = null;
      this.targets = new Map();
      this.drag = null;
      this.promo = null;
      this.dialog = null;
      this.hintMove = 0;
      this.anims = [];
      this.fades = [];
      this.puzLine = p.moves.slice();
      this.puzStep = 0;
      this.puzPlayer = 1 - pos.side;
      this.puzMate = MATE_N[STAGES[stage]] || 0;
      this.puzStatus = "intro";
      this.puzTimer = 0.7;
      this.puzMistakes = 0;
      this.puzShown = false;
      this.puzFeedback = ["watch", null];
      this.viewBlack = this.puzPlayer === E.BLACK;
      this.cursor = [6, 4];
      this.state = PUZ;
      this.refresh();
      this.playSound("click");
    }

    puzUndoLast() {
      this.pos.unmake();
      this.moves.pop();
      this.sans.pop();
      this.ucis.pop();
      this.caps.pop();
    }

    puzUpdate(dt) {
      if (this.puzTimer > 0) {
        this.puzTimer -= dt;
        if (this.puzTimer > 0) return;
      }
      const st = this.puzStatus;
      if (st === "intro") {
        const m = this.pos.parseUci(this.puzLine[0]);
        if (!m) {
          this.puzStatus = "shown_done";
          return;
        }
        this.commit(m);
        this.puzStep = 1;
        this.puzStatus = "play";
        this.puzFeedback = ["your_move", null];
      } else if (st === "reply") {
        const m = this.pos.parseUci(this.puzLine[this.puzStep]);
        if (m) this.commit(m);
        this.puzStep += 1;
        this.puzStatus = "play";
        this.puzFeedback = ["good", ui.GREEN];
      } else if (st === "wrong") {
        if (this.moves.length) {
          this.puzUndoLast();
          this.lastMove = this.moves.length ? moveSquares(this.moves[this.moves.length - 1]) : null;
          this.anims = [];
          this.refresh();
        }
        this.puzStatus = "play";
        this.puzFeedback = ["retry", ui.RED];
      } else if (st === "shown") {
        if (this.puzStep < this.puzLine.length) {
          const m = this.pos.parseUci(this.puzLine[this.puzStep]);
          if (m) this.commit(m);
          this.puzStep += 1;
          this.puzTimer = 0.85;
        } else {
          this.puzStatus = "shown_done";
          this.puzFeedback = ["shown", null];
        }
      }
    }

    puzPlayerMove(m, animate) {
      if (this.puzStatus !== "play" || this.puzStep >= this.puzLine.length) return;
      const pos = this.pos;
      const expected = pos.parseUci(this.puzLine[this.puzStep]);
      const mate = E.givesMate(pos, m);
      if (m === expected || mate) {
        this.commit(m, animate);
        this.puzStep += 1;
        if (mate || this.puzStep >= this.puzLine.length) this.puzSolvedNow();
        else {
          this.puzStatus = "reply";
          this.puzTimer = 0.5;
          this.puzFeedback = ["good", ui.GREEN];
          this.playSound("point");
        }
      } else {
        this.commit(m, animate);
        this.puzStatus = "wrong";
        this.puzTimer = 0.85;
        this.puzMistakes += 1;
        this.puzFeedback = ["wrong", ui.RED];
        this.playSound("hit");
        this.rumble(90);
      }
    }

    puzSolvedNow() {
      this.puzStatus = "solved";
      this.puzFeedback = ["solved", ui.GREEN];
      const pid = this.puz.id;
      if (!this.puzShown && !this.puzSolved.has(pid)) {
        this.puzSolved.add(pid);
        this.puzSaveProgress();
        this.achEvent("chess_puzzles", this.puzSolved.size);
      }
      this.playSound("win");
      ui.spawnBurst(this.boardRect.centerx, this.boardRect.centery, ui.GOLD, 34);
    }

    puzShowSolution() {
      if (this.state !== PUZ || ["solved", "shown", "shown_done"].includes(this.puzStatus)) return;
      if (this.puzStatus === "wrong" && this.moves.length) {
        this.puzUndoLast();
        this.refresh();
      }
      if (this.puzStep === 0) {
        const m = this.pos.parseUci(this.puzLine[0]);
        if (m) this.commit(m, false);
        this.puzStep = 1;
      }
      this.puzShown = true;
      if (!this.puzSolved.has(this.puz.id)) {
        this.puzSeen.add(this.puz.id);
        this.puzSaveProgress();
      }
      this.sel = null;
      this.targets = new Map();
      this.puzStatus = "shown";
      this.puzTimer = 0.45;
      this.puzFeedback = ["showing", null];
      this.playSound("select");
    }

    puzNext() {
      if (this.state !== PUZ) return;
      const lst = this.stageList();
      if (this.puzIdx + 1 < lst.length) this.puzOpen(this.puzStage, this.puzIdx + 1);
      else if (this.puzStage + 1 < STAGES.length) this.puzOpen(this.puzStage + 1, 0);
      else this.puzBackToMenu();
    }

    puzBackToMenu() {
      this.abortJobs();
      this.state = PUZ_MENU;
      this.promo = null;
      this.dialog = null;
      this.puzSaveProgress();
      this.playSound("click");
    }

    // ===================================================== Zeichnen: Hilfen
    clipText(fnt, text, width) {
      if (fnt.width(text) <= width) return text;
      while (text.length > 1 && fnt.width(text + "…") > width) text = text.slice(0, -1);
      return text.trimEnd() + "…";
    }

    fit(text, width, ...fonts) {
      for (const f of fonts) if (f.width(text) <= width) return f;
      return fonts[fonts.length - 1];
    }

    wrap(fnt, text, width) {
      const words = String(text).split(/\s+/).filter(Boolean);
      const lines = [];
      let cur = "";
      for (const w of words) {
        const test = cur ? cur + " " + w : w;
        if (fnt.width(test) <= width || !cur) cur = test;
        else {
          lines.push(cur);
          cur = w;
        }
      }
      if (cur) lines.push(cur);
      return lines;
    }

    pixelScale() {
      return Math.max(1, Math.min(3, (PG.app && PG.app.pixelScale) || 1));
    }

    pieceSprite(p, size) {
      const ps = this.pixelScale();
      const key = p + "|" + size + "|" + ps;
      let c = this.pieceCache.get(key);
      if (c) return c;
      if (this.pieceCache.size > 200) this.pieceCache.clear();
      c = ui.makeCanvas(size * ps, size * ps);
      const x = c.getContext("2d");
      x.scale(ps, ps);
      const px = Math.max(8, Math.floor(size * 0.8));
      x.font = px + "px " + PIECE_FAMILY;
      x.textAlign = "center";
      x.textBaseline = "middle";
      const typ = p & 7, white = !(p >> 3);
      const cx = size / 2, cy = size / 2 + size * 0.05;
      const o = Math.max(1, size / 36);
      x.fillStyle = "rgba(0,0,0,0.27)";
      x.fillText(GLYPH_FILL[typ], cx + o + 1, cy + o * 2);
      x.fillStyle = ui.col(COL_OUTLINE);
      for (const [ox, oy] of [[-o, 0], [o, 0], [0, -o], [0, o], [-o, -o], [o, o], [-o, o], [o, -o]]) x.fillText(GLYPH_FILL[typ], cx + ox, cy + oy);
      x.fillStyle = ui.col(white ? COL_WHITE : COL_BLACK);
      x.fillText(GLYPH_FILL[typ], cx, cy);
      x.fillStyle = ui.col(white ? COL_OUTLINE : [118, 110, 122]);
      x.fillText(GLYPH_LINE[typ], cx, cy);
      this.pieceCache.set(key, c);
      return c;
    }

    drawPiece(ctx, p, x, y, size, alpha) {
      const spr = this.pieceSprite(p, size);
      if (alpha != null && alpha < 1) {
        ctx.save();
        ctx.globalAlpha = Math.max(0, alpha);
        ctx.drawImage(spr, x, y, size, size);
        ctx.restore();
      } else ctx.drawImage(spr, x, y, size, size);
    }

    boardSurface() {
      const ps = this.pixelScale();
      const key = [this.cell, this.frame, this.viewBlack, ps, this.accent.join(",")].join("|");
      if (this.boardCache && this.boardKey === key) return this.boardCache;
      const pw = this.plate.w;
      const c = ui.makeCanvas(pw * ps, pw * ps);
      const x = c.getContext("2d");
      x.scale(ps, ps);
      const rad = Math.max(6, Math.floor(this.frame / 2));
      draw.rect(x, COL_PLATE, [0, 0, pw, pw], 0, rad);
      draw.rect(x, ui.mix(COL_PLATE, this.accent, 0.5), [0, 0, pw, pw], 1, rad);
      const f = this.frame, cell = this.cell;
      for (let dr = 0; dr < 8; dr++) {
        for (let dc = 0; dc < 8; dc++) {
          const sq = this.dispToSq(dr, dc);
          const light = ((sq >> 4) + (sq & 7)) % 2 === 1;
          draw.rect(x, light ? COL_LIGHT : COL_DARK, [f + dc * cell, f + dr * cell, cell, cell]);
        }
      }
      draw.rect(x, ui.mix(COL_PLATE, [0, 0, 0], 0.4), [f - 1, f - 1, 8 * cell + 2, 8 * cell + 2], 1);
      const cf = ui.font(Math.max(9, Math.floor(f * 0.62)), true);
      for (let i = 0; i < 8; i++) {
        const fileCh = this.viewBlack ? E.FILES[7 - i] : E.FILES[i];
        ui.text(x, fileCh, f + i * cell + cell / 2, f + 8 * cell + f / 2, cf, COL_COORD, "center");
        const rankCh = this.viewBlack ? String(i + 1) : String(8 - i);
        ui.text(x, rankCh, f / 2, f + i * cell + cell / 2, cf, COL_COORD, "center");
      }
      this.boardCache = c;
      this.boardKey = key;
      return c;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      if (this.state === SETUP) return this.drawSetup(ctx);
      if (this.state === PUZ_MENU) return this.drawPuzMenu(ctx);
      this.drawBoard(ctx);
      if (this.state === PUZ) this.drawPuzSide(ctx);
      else this.drawSide(ctx);
      if (this.promo !== null) this.drawPromo(ctx);
      if (this.dialog !== null) this.drawDialog(ctx);
      if (this.state === OVER) this.drawOver(ctx);
      this.drawToast(ctx);
    }

    drawBoard(ctx) {
      const now = ui.now();
      let flipK = null;
      if (this.flipAnim !== null) {
        const k = (now - this.flipAnim) / 0.32;
        if (k >= 1) this.flipAnim = null;
        else flipK = k;
      }
      ctx.save();
      if (flipK !== null) {
        // Brett drehen: horizontal zusammen- und wieder aufklappen
        const sx = Math.max(0.01, Math.abs(Math.cos(flipK * Math.PI)));
        ctx.translate(this.plate.centerx, 0);
        ctx.scale(sx, 1);
        ctx.translate(-this.plate.centerx, 0);
      }
      ctx.drawImage(this.boardSurface(), this.plate.x, this.plate.y, this.plate.w, this.plate.w);
      const c = this.cell;
      if (this.lastMove) for (const sq of this.lastMove) draw.rect(ctx, [...COL_LAST, 92], this.sqRect(sq));
      if (this.check && (this.state === PLAY || this.state === PUZ)) {
        const r = this.sqRect(this.pos.kings[this.pos.side]);
        const g = ctx.createRadialGradient(r.centerx, r.centery, 0, r.centerx, r.centery, c * 0.65);
        g.addColorStop(0, ui.col(COL_CHECK, 0.7));
        g.addColorStop(1, ui.col(COL_CHECK, 0));
        ctx.fillStyle = g;
        ctx.fillRect(r.centerx - c * 0.65, r.centery - c * 0.65, c * 1.3, c * 1.3);
      }
      const human = this.humanCanMove();
      if (this.sel !== null && human) {
        draw.rect(ctx, [...COL_SEL, 110], this.sqRect(this.sel));
        for (const [sq, ms] of this.targets) {
          const r = this.sqRect(sq);
          const occupied = this.pos.b[sq] !== 0 || ms.some((m) => m >> 20 === E.M_EP);
          if (occupied) draw.circle(ctx, [...COL_MOVE, 150], [r.centerx, r.centery], c / 2 - 1 - Math.max(3, c / 11) / 2, Math.max(3, Math.floor(c / 11)));
          else draw.circle(ctx, [...COL_MOVE, 150], [r.centerx, r.centery], Math.max(4, Math.floor(c / 6)));
        }
      }
      // geschlagene Figuren blenden unter dem schlagenden Stein aus
      this.fades = this.fades.filter((fd) => {
        const k = (now - fd.t0) / 0.35;
        if (k >= 1) return false;
        const r = this.sqRect(fd.sq);
        const size = c * (1 - 0.35 * k);
        this.drawPiece(ctx, fd.p, r.centerx - size / 2, r.centery - size / 2, size, 1 - k);
        return true;
      });
      const live = [];
      for (const a of this.anims) {
        const k = (now - a.t0) / ANIM_TIME;
        if (k < 1) live.push([a, ease(Math.max(0, k))]);
      }
      const hidden = new Set(live.map(([a]) => a.b));
      if (this.drag && this.drag.moved) hidden.add(this.drag.sq);
      const b = this.pos.b;
      for (const sq of E.SQUARES) {
        const p = b[sq];
        if (p && !hidden.has(sq)) {
          const r = this.sqRect(sq);
          this.drawPiece(ctx, p, r.x, r.y, c);
        }
      }
      for (const [a, k] of live) {
        const ra = this.sqRect(a.a), rb = this.sqRect(a.b);
        this.drawPiece(ctx, a.p, ra.x + (rb.x - ra.x) * k, ra.y + (rb.y - ra.y) * k, c);
      }
      if (!live.length) this.anims = [];
      if (this.hintMove && this.state === PLAY) this.drawArrow(ctx, this.hintMove);
      if (this.drag && this.drag.moved && human) {
        const p = b[this.drag.sq];
        if (p) {
          const over = this.sqAt(this.drag.pos);
          if (over !== null) draw.rect(ctx, ui.mix(COL_SEL, [255, 255, 255], 0.3), this.sqRect(over), 2);
          const size = Math.floor(c * 1.12);
          this.drawPiece(ctx, p, this.drag.pos[0] - size / 2, this.drag.pos[1] - size / 2, size);
        }
      }
      if (human && !this.drag) {
        const [dr, dc] = this.cursor;
        const k = ui.pulse(3.0, 0.0, 1.0);
        draw.rect(ctx, ui.mix(this.accent, [255, 255, 255], 0.35 + 0.4 * k),
          [this.bx + dc * c + 1, this.by + dr * c + 1, c - 2, c - 2], Math.max(2, Math.floor(c / 22)), 3);
      }
      ctx.restore();
    }

    drawArrow(ctx, m) {
      const [frm, to] = moveSquares(m);
      const ra = this.sqRect(frm), rb = this.sqRect(to);
      const ax = ra.centerx, ay = ra.centery, bx = rb.centerx, by = rb.centery;
      const ang = Math.atan2(by - ay, bx - ax);
      const width = Math.max(5, Math.floor(this.cell / 6));
      const head = Math.max(12, Math.floor(this.cell * 0.42));
      const ex = bx - Math.cos(ang) * head * 0.9, ey = by - Math.sin(ang) * head * 0.9;
      const col = [...COL_HINT, 190];
      ctx.save();
      ctx.lineCap = "round";
      draw.line(ctx, col, [ax, ay], [ex, ey], width);
      draw.polygon(ctx, col, [[bx, by], [bx - Math.cos(ang - 0.45) * head, by - Math.sin(ang - 0.45) * head],
        [bx - Math.cos(ang + 0.45) * head, by - Math.sin(ang + 0.45) * head]]);
      ctx.restore();
    }

    // ===================================================== Seitenleiste
    drawSide(ctx) {
      const bottom = this.viewBlack ? E.BLACK : E.WHITE;
      this.drawPlayer(ctx, this.topPanel, 1 - bottom);
      this.drawPlayer(ctx, this.botPanel, bottom);
      this.drawStatus(ctx);
      this.drawMovelist(ctx, this.listRect);
      this.drawButtons(ctx);
    }

    playerName(color) {
      if (color === this.humanColor) return t("chess.you");
      return t("common.ai") + " · " + t("chess.diff.lvl" + this.diff);
    }

    drawPlayer(ctx, rect, color) {
      const active = this.state === PLAY && this.pos.side === color;
      ui.drawPanel(ctx, rect, { shadow: false, accentTop: active ? this.accent : null });
      const pad = 8;
      const cy1 = rect.y + pad + this.small.height / 2;
      const dot = Math.max(5, Math.floor(this.small.height / 3));
      draw.circle(ctx, color === E.WHITE ? COL_WHITE : COL_BLACK, [rect.x + pad + dot, cy1], dot);
      draw.circle(ctx, ui.BORDER_LIGHT, [rect.x + pad + dot, cy1], dot, 1);
      let clockW = 0;
      if (this.clock) {
        const txt = fmtClock(this.clock[color]);
        const low = this.clock[color] < 10;
        const col = low ? ui.RED : active ? ui.TEXT : ui.TEXT_DIM;
        const tw = this.clockf.width(txt);
        const cr = new PG.Rect(rect.right - pad - tw, cy1 - this.clockf.height / 2, tw, this.clockf.height);
        if (active) draw.rect(ctx, ui.mix(ui.PANEL_LIGHT, this.accent, 0.25), cr.inflate(10, 4), 0, 5);
        ui.text(ctx, txt, cr.x, cr.y, this.clockf, col);
        clockW = tw + 14;
      }
      const nx = rect.x + pad + dot * 2 + 6;
      const name = this.clipText(this.small, this.playerName(color), rect.right - pad - clockW - nx);
      ui.text(ctx, name, nx, cy1, this.small, active ? ui.TEXT : ui.TEXT_DIM, "midleft");
      const cy2 = rect.bottom - pad - this.tiny.height / 2;
      const caps = this.caps.filter((p) => p && (p >> 3) !== color).sort((a, b) => E.ORDER_VALUE[b & 7] - E.ORDER_VALUE[a & 7]);
      const size = this.tiny.height + 6;
      const step = Math.max(6, Math.floor(size * 0.52));
      const diff = this.pos.material(color) - this.pos.material(1 - color);
      const dtxt = diff > 0 ? "+" + diff : "";
      const maxx = rect.right - pad - (dtxt ? this.tiny.width(dtxt) + 6 : 0);
      let x = rect.x + pad - 2;
      for (let i = 0; i < caps.length; i++) {
        if (i > 0 && (caps[i - 1] & 7) !== (caps[i] & 7)) x += Math.floor(step / 2);
        if (x + size > maxx) break;
        this.drawPiece(ctx, caps[i], x, cy2 - size / 2, size);
        x += step;
      }
      if (dtxt) ui.text(ctx, dtxt, Math.min(x + size - step + 6, maxx + 6), cy2, this.tiny, ui.GREEN, "midleft");
    }

    statusText() {
      if (this.state === OVER) return [t("chess.game_over"), ui.TEXT_DIM];
      if (this.promo !== null) return [t("chess.promote"), this.accent];
      if (this.hintJob) return [t("chess.hint_wait") + this.dots(), ui.TEXT_DIM];
      if (this.pos.side !== this.humanColor) return [t("chess.ai_thinks").replace(/[\s….]+$/, "") + this.dots(), ui.TEXT_DIM];
      if (this.check) return [t("chess.check"), ui.RED];
      return [t("chess.your_turn"), this.accent];
    }

    dots() {
      return " " + ".".repeat(Math.floor(ui.now() * 3) % 4);
    }

    drawStatus(ctx) {
      const [text, col] = this.statusText();
      const r = this.statusRect;
      const fnt = this.fit(text, r.w - 8, this.small, this.tiny);
      ui.text(ctx, this.clipText(fnt, text, r.w - 8), r.centerx, r.centery, fnt, col, "center");
    }

    drawMovelist(ctx, r) {
      if (r.h < 20) return;
      ui.drawPanel(ctx, r, { shadow: false });
      const fnt = this.listf, rh = this.rowH, pad = 6;
      let head = t("chess.moves");
      if (this.c960N !== null) head += "  ·  Chess960 #" + this.c960N;
      ui.text(ctx, this.clipText(this.tiny, head, r.w - 2 * pad), r.x + pad, r.y + 4, this.tiny, ui.TEXT_FAINT);
      const y0 = r.y + 6 + this.tiny.height;
      const visible = this.listVisible();
      const total = this.listRows();
      if (this.listFollow) this.listTop = Math.max(0, total - visible);
      const top = Math.max(0, Math.min(this.listTop, Math.max(0, total - visible)));
      const numW = fnt.width(String(this.startFull + total + 90) + ".") + 4;
      const colW = Math.floor((r.w - 2 * pad - numW) / 2);
      const last = this.sans.length - 1;
      const shift = this.startSide;
      ctx.save();
      ctx.beginPath();
      ctx.rect(r.x + 2, r.y + 2, r.w - 4, r.h - 4);
      ctx.clip();
      for (let row = top; row < Math.min(total, top + visible); row++) {
        const y = y0 + (row - top) * rh;
        if (row % 2 === 1) draw.rect(ctx, ui.mix(ui.PANEL, ui.PANEL_LIGHT, 0.5), [r.x + 3, y - 1, r.w - 6, rh], 0, 3);
        ui.text(ctx, (this.startFull + row) + ".", r.x + pad, y, fnt, ui.TEXT_FAINT);
        for (const ci of [0, 1]) {
          const i = row * 2 + ci - shift;
          const x = r.x + pad + numW + ci * colW;
          if (i < 0) {
            ui.text(ctx, "…", x, y, fnt, ui.TEXT_FAINT);
            continue;
          }
          if (i > last) break;
          const cur = i === last;
          if (cur) draw.rect(ctx, ui.mix(ui.PANEL_LIGHT, this.accent, 0.35), [x - 3, y - 1, colW - 2, rh], 0, 4);
          ui.text(ctx, this.clipText(fnt, this.sans[i], colW - 6), x, y, fnt, cur ? ui.TEXT : ui.TEXT_DIM);
        }
      }
      ctx.restore();
      if (total > visible) {
        const track = new PG.Rect(r.right - 5, y0, 3, visible * rh);
        draw.rect(ctx, ui.mix(ui.PANEL, ui.BORDER_LIGHT, 0.4), track, 0, 2);
        const hh = Math.max(10, Math.floor((track.h * visible) / total));
        const hy = track.y + Math.floor(((track.h - hh) * top) / Math.max(1, total - visible));
        draw.rect(ctx, this.accent, [track.x, hy, 3, hh], 0, 2);
      }
    }

    buttonEnabled(bid) {
      if (this.state !== PLAY) return false;
      if (bid === "undo") return this.moves.some((_, i) => i % 2 === this.humanColor);
      if (bid === "hint") return this.pos.side === this.humanColor;
      if (bid === "draw") return this.moves.length > 0;
      return true;
    }

    drawButtons(ctx) {
      for (const id in this.btnRects) this.iconButton(ctx, this.btnRects[id], id, this.buttonEnabled(id), this.hover === id);
      if (this.state === PLAY && this.hover && this.btnRects[this.hover]) this.drawTooltip(ctx, this.btnRects[this.hover], t("chess.btn." + this.hover));
    }

    iconButton(ctx, rc, bid, enabled, hover) {
      const rad = Math.max(4, Math.floor(rc.w / 6));
      draw.rect(ctx, hover && enabled ? ui.BTN_SEL : ui.BTN, rc, 0, rad);
      draw.rect(ctx, hover && enabled ? this.accent : ui.BORDER, rc, 1, rad);
      let col = enabled ? ui.TEXT : ui.TEXT_FAINT;
      if (hover && enabled) col = ui.mix(ui.TEXT, this.accent, 0.35);
      this.icon(ctx, rc, bid, col);
    }

    icon(ctx, rc, kind, col) {
      const cx = rc.centerx, cy = rc.centery, u = rc.w / 24;
      const lw = Math.max(2, 2.2 * u);
      const css = ui.col(col);
      ctx.save();
      ctx.strokeStyle = css;
      ctx.fillStyle = css;
      ctx.lineWidth = lw;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      if (kind === "undo" || kind === "retry") {
        const r = 6.5 * u;
        const y = cy + u;
        ctx.beginPath();
        if (kind === "undo") {
          ctx.arc(cx, y, r, -Math.PI * 0.95, Math.PI * 0.55);
          ctx.stroke();
          const ax = cx + r * Math.cos(-Math.PI * 0.95), ay = y + r * Math.sin(-Math.PI * 0.95);
          ctx.beginPath();
          ctx.moveTo(ax - 4 * u, ay - 1 * u);
          ctx.lineTo(ax + 3 * u, ay - 3.5 * u);
          ctx.lineTo(ax + 1.5 * u, ay + 4 * u);
          ctx.closePath();
          ctx.fill();
        } else {
          ctx.arc(cx, y, r, -Math.PI * 0.15, Math.PI * 1.3);
          ctx.stroke();
          const ax = cx + r * Math.cos(-Math.PI * 0.15), ay = y + r * Math.sin(-Math.PI * 0.15);
          ctx.beginPath();
          ctx.moveTo(ax - 4 * u, ay - 1 * u);
          ctx.lineTo(ax + 4 * u, ay - 1 * u);
          ctx.lineTo(ax, ay + 4 * u);
          ctx.closePath();
          ctx.fill();
        }
      } else if (kind === "hint" || kind === "solution") {
        ctx.beginPath();
        ctx.arc(cx, cy - 2.5 * u, 5.5 * u, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(cx - 3 * u, cy + 5 * u);
        ctx.lineTo(cx + 3 * u, cy + 5 * u);
        ctx.moveTo(cx - 2 * u, cy + 8 * u);
        ctx.lineTo(cx + 2 * u, cy + 8 * u);
        ctx.stroke();
        if (kind === "hint") {
          ctx.lineWidth = Math.max(1, lw - 1);
          ctx.beginPath();
          for (const a of [-2.3, -1.57, -0.84]) {
            ctx.moveTo(cx + Math.cos(a) * 8.5 * u, cy - 2.5 * u + Math.sin(a) * 8.5 * u);
            ctx.lineTo(cx + Math.cos(a) * 10.5 * u, cy - 2.5 * u + Math.sin(a) * 10.5 * u);
          }
          ctx.stroke();
        }
      } else if (kind === "flip") {
        for (const sgn of [-1, 1]) {
          const x = cx + sgn * 4 * u;
          ctx.beginPath();
          ctx.moveTo(x, cy - 6 * u);
          ctx.lineTo(x, cy + 6 * u);
          ctx.stroke();
          const tip = sgn < 0 ? cy - 8 * u : cy + 8 * u;
          const back = sgn < 0 ? cy - 3 * u : cy + 3 * u;
          ctx.beginPath();
          ctx.moveTo(x, tip);
          ctx.lineTo(x - 3.5 * u, back);
          ctx.lineTo(x + 3.5 * u, back);
          ctx.closePath();
          ctx.fill();
        }
      } else if (kind === "draw") {
        ui.text(ctx, "½", cx, cy, ui.font(Math.max(10, Math.floor(15 * u)), true), col, "center");
      } else if (kind === "resign") {
        const x = cx - 5 * u;
        ctx.beginPath();
        ctx.moveTo(x, cy - 8 * u);
        ctx.lineTo(x, cy + 8 * u);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(x, cy - 8 * u);
        ctx.lineTo(x + 11 * u, cy - 4.5 * u);
        ctx.lineTo(x, cy - 1 * u);
        ctx.closePath();
        ctx.fill();
      } else if (kind === "list") {
        const q = 4.2 * u, s2 = q * 1.5;
        for (const ix of [-1, 1]) for (const iy of [-1, 1]) draw.rect(ctx, col, [cx + ix * q - s2 / 2, cy + iy * q - s2 / 2, s2, s2], 0, Math.max(1, u));
      } else if (kind === "next") {
        ctx.lineWidth = lw + 1;
        ctx.beginPath();
        ctx.moveTo(cx - 6 * u, cy - 7 * u);
        ctx.lineTo(cx + 2 * u, cy);
        ctx.lineTo(cx - 6 * u, cy + 7 * u);
        ctx.moveTo(cx + 1 * u, cy - 7 * u);
        ctx.lineTo(cx + 9 * u, cy);
        ctx.lineTo(cx + 1 * u, cy + 7 * u);
        ctx.stroke();
      }
      ctx.restore();
    }

    drawTooltip(ctx, anchor, text) {
      const fnt = this.tiny, side = this.sideRect;
      text = this.clipText(fnt, text, side.w - 16);
      const w = fnt.width(text) + 14, h = fnt.height + 8;
      const x = Math.max(side.x, Math.min(side.right - w, anchor.centerx - w / 2));
      const rect = new PG.Rect(x, anchor.y - h - 5, w, h);
      draw.rect(ctx, ui.PANEL_LIGHT, rect, 0, 6);
      draw.rect(ctx, ui.mix(ui.BORDER_LIGHT, this.accent, 0.5), rect, 1, 6);
      ui.text(ctx, text, rect.centerx, rect.centery, fnt, ui.TEXT, "center");
    }

    // ===================================================== Overlays
    drawToast(ctx) {
      if (!this.msg) return;
      const alpha = this.msgT > 0.4 ? 1 : Math.max(0, this.msgT) / 0.4;
      const fnt = this.fit(this.msg, this.bw - 24, this.small, this.tiny);
      const text = this.clipText(fnt, this.msg, this.bw - 24);
      const w = fnt.width(text) + 28, h = fnt.height + 12;
      const rect = new PG.Rect(this.boardRect.centerx - w / 2, this.boardRect.bottom - h - Math.max(8, this.cell / 4), w, h);
      ctx.save();
      ctx.globalAlpha = alpha;
      draw.rect(ctx, [...ui.PANEL, 235], rect, 0, h / 2);
      draw.rect(ctx, ui.mix(ui.BORDER, this.accent, 0.5), rect, 1, h / 2);
      ui.text(ctx, text, rect.centerx, rect.centery, fnt, ui.TEXT, "center");
      ctx.restore();
    }

    dimBoard(ctx, alpha) {
      draw.rect(ctx, [8, 8, 12, alpha], this.plate, 0, Math.max(6, Math.floor(this.frame / 2)));
    }

    drawPromo(ctx) {
      this.dimBoard(ctx, 160);
      const rects = this.promoRects();
      ui.text(ctx, t("chess.promote"), this.boardRect.centerx, rects[0].y - 10, this.small, ui.TEXT, "midbottom");
      const side = this.pos.side;
      this.promo.forEach((m, i) => {
        const rc = rects[i];
        ui.drawPanel(ctx, rc, { color: ui.BTN_SEL, shadow: false });
        draw.rect(ctx, this.accent, rc, 2, 8);
        const size = Math.floor(rc.w * 0.9);
        this.drawPiece(ctx, ((m >> 16) & 7) | (side << 3), rc.centerx - size / 2, rc.centery - size / 2, size);
      });
    }

    drawDialog(ctx) {
      this.dimBoard(ctx, 140);
      const [panel, yes, no] = this.dialogRects();
      ui.drawPanel(ctx, panel, { accentTop: this.accent });
      let y = panel.y + 12;
      for (const ln of this.wrap(this.small, t("chess.resign_q"), panel.w - 24).slice(0, 3)) {
        ui.text(ctx, ln, panel.centerx, y, this.small, ui.TEXT, "midtop");
        y += this.small.height + 2;
      }
      for (const [rc, label, sel] of [[yes, t("chess.yes"), true], [no, t("chess.no"), false]]) {
        const fnt = this.fit(label, rc.w - 8, this.small, this.tiny);
        ui.drawButton(ctx, rc, this.clipText(fnt, label, rc.w - 8), fnt, sel, { accent: this.accent });
      }
    }

    resultTexts() {
      const [winner, reason] = this.result;
      let head, col;
      if (winner === null) {
        head = t("common.draw");
        col = ui.TEXT;
      } else if (winner === this.humanColor) {
        head = t("chess.win_you");
        col = ui.GOLD;
      } else {
        head = t("chess.win_ai");
        col = ui.TEXT_DIM;
      }
      const loser = this.resultLoser !== null ? this.resultLoser : 0;
      let sub = t("chess.reason." + reason, { color: loser === E.WHITE ? t("chess.white") : t("chess.black") });
      if (winner === this.humanColor && this.assisted) sub += " · " + t("chess.assisted_short");
      return [head, col, sub];
    }

    drawOver(ctx) {
      const k = Math.min(1, (ui.now() - (this.overT0 || 0)) / 0.35);
      const [head, col, sub] = this.resultTexts();
      let r = this.overRect;
      if (k < 1) r = r.move(0, Math.floor((1 - ease(k)) * 24));
      ctx.save();
      ctx.globalAlpha = ease(k);
      draw.rect(ctx, [...ui.PANEL, 238], r, 0, 12);
      draw.rect(ctx, ui.mix(ui.BORDER, this.accent, 0.6), r, 1, 12);
      draw.rect(ctx, this.accent, [r.x + 14, r.y, r.w - 28, 3], 0, 2);
      const hf = this.fit(head, r.w - 20, this.huge, this.big, this.small);
      ui.text(ctx, head, r.centerx, r.y + 10, hf, col, "midtop");
      const sf = this.wrap(this.small, sub, r.w - 20).length <= 2 ? this.small : this.tiny;
      let y = r.y + 12 + hf.height;
      for (const ln of this.wrap(sf, sub, r.w - 20).slice(0, 2)) {
        ui.text(ctx, this.clipText(sf, ln, r.w - 20), r.centerx, y, sf, ui.TEXT_DIM, "midtop");
        y += sf.height;
      }
      ctx.restore();
      if (k >= 1) {
        for (const id of ["new", "setup", "pgn"]) {
          const rc = this.overBtns[id];
          const label = t("chess.btn." + id);
          const fnt = this.fit(label, rc.w - 8, this.small, this.tiny);
          ui.drawButton(ctx, rc, this.clipText(fnt, label, rc.w - 8), fnt, this.hover === "over_" + id || id === "new", { accent: this.accent });
        }
      }
    }

    // ===================================================== Setup
    drawSetup(ctx) {
      const w = this.width, h = this.height, cx = w / 2;
      ui.drawTitle(ctx, w, t("chess.title"), {
        subtitle: this.clipText(this.small, t("chess.subtitle"), w - 30), y: Math.floor(h * 0.09),
        big: this.huge, small: this.small, accent: this.accent,
      });
      const clockLabels = [t("chess.clock.none"), "1+0", "3+2", "5+0", "10+5"];
      this.setupRows.forEach(([rid, rects], ri) => {
        const [, cur] = this.setupValues(rid);
        let label, labels;
        if (rid === "diff") {
          label = t("chess.lbl.diff", { name: t("chess.diff.lvl" + this.diff) });
          labels = ["1", "2", "3", "4", "5", "6"];
        } else if (rid === "color") {
          label = t("chess.lbl.color");
          labels = [t("chess.white"), t("chess.black")];
        } else if (rid === "clock") {
          label = t("chess.lbl.clock");
          labels = clockLabels;
        } else {
          label = t("chess.lbl.c960");
          labels = [t("common.off"), t("common.on")];
        }
        const focus = ri === this.setupFocus;
        ui.text(ctx, this.clipText(this.tiny, label, rects[rects.length - 1].right - rects[0].x), cx, rects[0].y - 2, this.tiny, focus ? ui.TEXT : ui.TEXT_DIM, "midbottom");
        if (focus) draw.rect(ctx, this.accent, [rects[0].x - 10, rects[0].y + 4, 3, rects[0].h - 8], 0, 2);
        rects.forEach((rc, i) => {
          const fnt = this.fit(labels[i], rc.w - 8, this.small, this.tiny);
          ui.drawButton(ctx, rc, this.clipText(fnt, labels[i], rc.w - 6), fnt, i === cur, { accent: this.accent });
        });
      });
      const st = t("common.start");
      ui.drawButton(ctx, this.startRect, st, this.big.width(st) < this.startRect.w - 10 ? this.big : this.small, true, { accent: this.accent });
      if (this.setupFocus >= this.setupRows.length) draw.rect(ctx, this.accent, this.startRect.inflate(8, 8), 2, 10);
      ui.drawFooter(ctx, w, h, this.clipText(this.tiny, t("chess.setup_hint"), w - 20), this.tiny);
    }

    // ===================================================== Rätsel-Auswahl
    drawPuzMenu(ctx) {
      const w = this.width, h = this.height;
      let total = 0, solved = 0;
      for (const s of STAGES) {
        for (const p of this.puzzles[s]) {
          total++;
          if (this.puzSolved.has(p.id)) solved++;
        }
      }
      ui.drawTitle(ctx, w, t("chess.puz.title"), {
        subtitle: t("chess.puz.progress", { n: solved, total }), y: Math.floor(h * 0.075),
        big: this.huge, small: this.small, accent: this.accent,
      });
      this.stageRects.forEach((rc, i) => {
        const name = t("chess.stage." + STAGES[i]);
        const fnt = this.fit(name, rc.w - 6, this.small, this.tiny);
        ui.drawButton(ctx, rc, this.clipText(fnt, name, rc.w - 6), fnt, i === this.puzStage, {
          accent: this.accent, sub: this.stageSolved(i) + "/" + this.stageList(i).length, subFont: this.tiny,
        });
      });
      const lst = this.stageList();
      if (!lst.length) ui.text(ctx, t("chess.puz.missing"), w / 2, h / 2, this.small, ui.RED, "center");
      for (let i = 0; i < Math.min(lst.length, this.gridRects.length); i++) {
        const rc = this.gridRects[i];
        const p = lst[i];
        const done = this.puzSolved.has(p.id), seen = this.puzSeen.has(p.id);
        const cur = i === this.puzCursor, hov = this.hover === "cell" + i;
        draw.rect(ctx, done ? ui.mix(ui.BTN, ui.GREEN, 0.28) : hov ? ui.BTN_SEL : ui.BTN, rc, 0, 6);
        draw.rect(ctx, cur ? this.accent : done ? ui.mix(ui.BORDER, ui.GREEN, 0.5) : ui.BORDER, rc, cur ? 2 : 1, 6);
        ui.text(ctx, String(i + 1), rc.centerx, rc.centery - (rc.h > 34 ? 3 : 0), this.small, cur || done ? ui.TEXT : ui.TEXT_DIM, "center");
        if (done) {
          const u = Math.max(3, Math.floor(rc.h / 9));
          const x0 = rc.right - 3 * u - 3, y0 = rc.y + 2 * u + 1;
          draw.lines(ctx, ui.GREEN, false, [[x0 - u, y0], [x0, y0 + u], [x0 + 2 * u, y0 - u]], Math.max(2, Math.floor(u / 2)));
        } else if (seen) {
          draw.circle(ctx, ui.TEXT_FAINT, [rc.right - 7, rc.y + 7], 3);
        }
        if (rc.h > 34) ui.text(ctx, String(p.rating || ""), rc.centerx, rc.bottom - 2, this.tiny, ui.TEXT_FAINT, "midbottom");
      }
      if (lst.length) {
        const label = t("chess.puz.start", { n: this.puzCursor + 1 });
        ui.drawButton(ctx, this.puzStartRect, label, this.fit(label, this.puzStartRect.w - 10, this.small, this.tiny), true, { accent: this.accent });
      }
      ui.drawFooter(ctx, w, h, this.clipText(this.tiny, t("chess.puz.footer"), w - 20), this.tiny);
    }

    // ===================================================== Rätsel-Seitenleiste
    drawPuzSide(ctx) {
      const r = this.puzInfoRect;
      ui.drawPanel(ctx, r, { shadow: false, accentTop: this.accent });
      const pad = 8, x = r.x + pad, wmax = r.w - 2 * pad;
      let y = r.y + pad;
      const name = t("chess.stage." + STAGES[this.puzStage]);
      const hf = this.fit(name, wmax, this.big, this.small);
      ui.text(ctx, name, x, y, hf, ui.TEXT);
      y += hf.height + 2;
      const num = t("chess.puz.number", { n: this.puzIdx + 1, total: this.stageList().length });
      const rat = t("chess.puz.rating", { rating: this.puz.rating || "?" });
      ui.text(ctx, num, x, y, this.tiny, ui.TEXT_DIM);
      if (this.tiny.width(num) + this.tiny.width(rat) + 10 <= wmax) ui.text(ctx, rat, r.right - pad, y, this.tiny, ui.TEXT_FAINT, "topright");
      else {
        y += this.tiny.height + 1;
        ui.text(ctx, rat, x, y, this.tiny, ui.TEXT_FAINT);
      }
      y += this.tiny.height + 2;
      const theme = this.puz.theme || "";
      if (theme && theme !== "mate" && THEME_LABEL.includes(theme)) {
        ui.text(ctx, this.clipText(this.tiny, t("chess.puz.theme", { name: t("chess.theme." + theme) }), wmax), x, y, this.tiny, this.accent);
        y += this.tiny.height + 2;
      }
      y += 6;
      const color = this.puzPlayer === E.WHITE ? t("chess.white") : t("chess.black");
      const dot = Math.max(5, Math.floor(this.small.height / 3));
      draw.circle(ctx, this.puzPlayer === E.WHITE ? COL_WHITE : COL_BLACK, [x + dot, y + this.small.height / 2], dot);
      draw.circle(ctx, ui.BORDER_LIGHT, [x + dot, y + this.small.height / 2], dot, 1);
      const you = t("chess.puz.you_play", { color });
      const yf = this.fit(you, wmax - 2 * dot - 6, this.small, this.tiny);
      ui.text(ctx, this.clipText(yf, you, wmax - 2 * dot - 6), x + 2 * dot + 6, y + this.small.height / 2, yf, ui.TEXT, "midleft");
      y += this.small.height + 4;
      const goal = this.puzMate === 1 ? t("chess.puz.goal_mate1") : this.puzMate ? t("chess.puz.goal_mate", { n: this.puzMate }) : t("chess.puz.goal_tactic");
      for (const ln of this.wrap(this.tiny, goal, wmax).slice(0, 2)) {
        ui.text(ctx, ln, x, y, this.tiny, ui.TEXT_DIM);
        y += this.tiny.height + 1;
      }
      y += 6;
      const own = Math.floor(this.puzLine.length / 2);
      const done = Math.min(own, Math.floor(this.puzStep / 2));
      const rr = Math.max(4, Math.floor(this.tiny.height / 3));
      for (let i = 0; i < own; i++) draw.circle(ctx, i < done ? ui.GREEN : ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.6), [x + rr + i * rr * 3, y + rr], rr);
      y += 2 * rr + 10;
      const [key, fcol] = this.puzFeedback;
      const fb = t("chess.puz.fb." + key, { color });
      let extra = "";
      if (key === "solved" && this.puzMistakes) extra = t("chess.puz.mistakes", { n: this.puzMistakes });
      else if (this.puzStatus === "solved" || this.puzStatus === "shown_done") extra = t("chess.puz.next_hint");
      const room = r.bottom - pad - y;
      const bf = this.big.height * 2 + this.tiny.height <= room ? this.big : this.small;
      for (const ln of this.wrap(bf, fb, wmax).slice(0, 2)) {
        ui.text(ctx, this.clipText(bf, ln, wmax), x, y, bf, fcol || ui.TEXT);
        y += bf.height + 1;
      }
      if (extra && y + this.tiny.height <= r.bottom - 4) ui.text(ctx, this.clipText(this.tiny, extra, wmax), x, y + 1, this.tiny, ui.TEXT_DIM);
      this.drawMovelist(ctx, this.puzListRect);
      for (const id in this.puzBtnRects) {
        const enabled = id !== "solution" || !["solved", "shown", "shown_done"].includes(this.puzStatus);
        const highlight = id === "next" && (this.puzStatus === "solved" || this.puzStatus === "shown_done");
        this.iconButton(ctx, this.puzBtnRects[id], id, enabled, this.hover === id || highlight);
      }
      if (this.hover && this.puzBtnRects[this.hover]) this.drawTooltip(ctx, this.puzBtnRects[this.hover], t("chess.btn." + this.hover));
    }
  }

  function fmtClock(sec) {
    sec = Math.max(0, sec);
    if (sec < 10) return "0:" + sec.toFixed(1).padStart(4, "0");
    const whole = Math.ceil(sec);
    return Math.floor(whole / 60) + ":" + String(whole % 60).padStart(2, "0");
  }

  PG.register(ChessGame, {
    id: "ChessGame",
    key: "chess",
    name: { default: "Chess", de: "Schach", fr: "Échecs", es: "Ajedrez", pt: "Xadrez" },
    modes: [["single", "chess.mode.single"], ["puzzles", "chess.mode.puzzles"]],
    settingsKey: "chess",
    defaults: { difficulty: 2, color: "white", clock: "none", chess960: false },
  });
})();
