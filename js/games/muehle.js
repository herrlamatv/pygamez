/*
 * muehle.js - Mühle (Nine Men's Morris) gegen die KI (Port von games/muehle.py)
 * =============================================================================
 * Ablauf in drei Phasen:
 *   1. Setzen : Jeder Spieler setzt nacheinander seine 9 Steine auf freie Punkte.
 *   2. Ziehen : Danach werden Steine entlang der Linien auf benachbarte freie
 *               Punkte gezogen.
 *   3. Springen (Fliegen): Wer nur noch 3 Steine hat, darf auf JEDEN freien Punkt
 *               springen (per Feature-Schalter im Setup abschaltbar).
 *
 * Bildet ein Zug eine Mühle (drei eigene Steine in einer Linie), wird ein
 * gegnerischer Stein entfernt - möglichst keiner aus einer gegnerischen Mühle,
 * außer es steht kein anderer zur Verfügung. Verloren hat, wer auf unter 3 Steine
 * fällt oder keinen Zug mehr machen kann.
 *
 * - KI über Minimax mit Alpha-Beta, phasengerechter Bewertung (Material,
 *   geschlossene Mühlen, Beweglichkeit, offene Zwickmühlen) und Zeitbudget.
 *   Drei Stärken; die leichte patzt absichtlich.
 *   Web: Die Wurzelzüge werden über mehrere Frames verteilt durchsucht, damit
 *   die Oberfläche nicht einfriert (Zeitbudget = verbrauchte Rechenzeit).
 * - Punkte (Highscore) = Siege gegen die KI in einer Sitzung.
 *
 * Steuerung: Maus (Punkt anklicken; zum Ziehen erst eigenen Stein, dann Ziel).
 * Nach Rundenende: Enter = neue Runde, S = Setup.
 * (Der 2-Spieler-Modus der Python-Version entfällt in der Web-Version.)
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // Untertitel ohne "oder zu zweit" (Web-Version = nur gegen die KI)
  PG.addStrings({
    de: { "web.muehle.subtitle": "Mühle – gegen die KI" },
    en: { "web.muehle.subtitle": "Nine Men's Morris – versus the AI" },
  });

  // ------------------------------------------------- Brett-Identitätsfarben
  // Generische UI-Farben (Hintergrund, Panels, Text) liefert die ui-Palette.
  const COL_PLATE = [24, 38, 31]; // Grundplatte hinter dem Liniennetz
  const COL_LINE = [120, 168, 140];
  const COL_SPOT = [46, 74, 60];
  const COL_P1 = [238, 240, 246]; // Spieler 0 (hell)
  const COL_P1_HI = [255, 255, 255];
  const COL_P2 = [44, 52, 66]; // Spieler 1 (dunkel)
  const COL_P2_HI = [96, 108, 130];
  const COL_SEL = [246, 214, 92];
  const COL_HINT = [120, 210, 150];
  const COL_MILL = [240, 180, 80];
  const COL_REMOVE = [224, 96, 96];

  const SETUP = "setup", PLAY = "play", OVER = "over";

  const DIFFS = ["easy", "medium", "hard"];
  const DEPTHS = [1, 2, 4];
  const TIME_BUDGET = [0.2, 0.5, 1.0];
  const NODE_BUDGET = 300000;
  // Web: maximale Rechenzeit pro Frame (ms), danach geht es im nächsten Frame weiter
  const SLICE_MS = 12;
  // Web: Notbremse, falls ein einzelner Teilbaum einen Frame zu lange blockiert
  const HARD_SLICE_MS = 60;

  // 24 Punkte als (Spalte, Zeile) im 0..6-Raster.
  const POS = [[0, 0], [3, 0], [6, 0],
    [1, 1], [3, 1], [5, 1],
    [2, 2], [3, 2], [4, 2],
    [0, 3], [1, 3], [2, 3], [4, 3], [5, 3], [6, 3],
    [2, 4], [3, 4], [4, 4],
    [1, 5], [3, 5], [5, 5],
    [0, 6], [3, 6], [6, 6]];

  const ADJ = [
    [1, 9], [0, 2, 4], [1, 14],
    [4, 10], [1, 3, 5, 7], [4, 13],
    [7, 11], [4, 6, 8], [7, 12],
    [0, 10, 21], [3, 9, 11, 18], [6, 10, 15],
    [8, 13, 17], [5, 12, 14, 20], [2, 13, 23],
    [11, 16], [15, 17, 19], [12, 16],
    [10, 19], [16, 18, 20, 22], [13, 19],
    [9, 22], [19, 21, 23], [14, 22],
  ];

  const MILLS = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11], [12, 13, 14],
    [15, 16, 17], [18, 19, 20], [21, 22, 23],
    [0, 9, 21], [3, 10, 18], [6, 11, 15], [1, 4, 7],
    [16, 19, 22], [8, 12, 17], [5, 13, 20], [2, 14, 23],
  ];

  // Für jeden Punkt die Mühlen, die ihn enthalten (Vorberechnung).
  const MILLS_AT = Array.from({ length: 24 }, (_, p) => MILLS.filter((m) => m.includes(p)));

  const nowMs = () => (typeof performance !== "undefined" ? performance.now() : Date.now());

  // =================================================================== Regel-Helfer
  function formsMill(board, point, val) {
    for (const m of MILLS_AT[point]) {
      if (board[m[0]] === val && board[m[1]] === val && board[m[2]] === val) return true;
    }
    return false;
  }

  /** Entfernbare Gegnersteine: bevorzugt solche außerhalb von Mühlen. */
  function removable(board, opp) {
    const inMill = new Array(24).fill(false);
    for (const m of MILLS) {
      if (board[m[0]] === opp && board[m[1]] === opp && board[m[2]] === opp) {
        inMill[m[0]] = inMill[m[1]] = inMill[m[2]] = true;
      }
    }
    const oppPts = [];
    for (let p = 0; p < 24; p++) if (board[p] === opp) oppPts.push(p);
    const free = oppPts.filter((p) => !inMill[p]);
    return free.length ? free : oppPts;
  }

  function count(board, val) {
    let n = 0;
    for (let i = 0; i < 24; i++) if (board[i] === val) n++;
    return n;
  }

  const ALL_POINTS = Array.from({ length: 24 }, (_, i) => i);

  /** Vollständige Züge (inkl. evtl. Entfernen) für 'player' (0/1). Zug = [kind, a, b, rp]. */
  function genMoves(board, placed, player, flying) {
    const val = player + 1;
    const opp = val === 1 ? 2 : 1;
    const moves = [];
    if (placed[player] < 9) {
      for (let p = 0; p < 24; p++) {
        if (board[p] === 0) {
          board[p] = val;
          if (formsMill(board, p, val)) {
            for (const rp of removable(board, opp)) moves.push(["place", p, null, rp]);
          } else {
            moves.push(["place", p, null, null]);
          }
          board[p] = 0;
        }
      }
    } else {
      const on = count(board, val);
      const fly = flying && on === 3;
      for (let p = 0; p < 24; p++) {
        if (board[p] !== val) continue;
        const dests = fly ? ALL_POINTS : ADJ[p];
        for (const d of dests) {
          if (board[d] === 0) {
            board[p] = 0;
            board[d] = val;
            if (formsMill(board, d, val)) {
              for (const rp of removable(board, opp)) moves.push(["move", p, d, rp]);
            } else {
              moves.push(["move", p, d, null]);
            }
            board[d] = 0;
            board[p] = val;
          }
        }
      }
    }
    return moves;
  }

  function applyMove(board, placed, player, mv) {
    const val = player + 1;
    const nb = board.slice();
    const npl = placed.slice();
    const [kind, a, b, rp] = mv;
    if (kind === "place") {
      nb[a] = val;
      npl[player] += 1;
    } else {
      nb[a] = 0;
      nb[b] = val;
    }
    if (rp !== null) nb[rp] = 0;
    return [nb, npl];
  }

  function hasMoves(board, placed, player, flying) {
    if (placed[player] < 9) return true;
    const val = player + 1;
    const on = count(board, val);
    if (flying && on === 3) return true;
    for (let p = 0; p < 24; p++) {
      if (board[p] === val) {
        for (const d of ADJ[p]) if (board[d] === 0) return true;
      }
    }
    return false;
  }

  function evalSide(board, val, placed, player, flying) {
    let mills = 0;
    let twos = 0;
    for (const m of MILLS) {
      let cnt = 0, empty = 0;
      for (let k = 0; k < 3; k++) {
        const v = board[m[k]];
        if (v === val) cnt++;
        else if (v === 0) empty++;
      }
      if (cnt === 3) mills++;
      else if (cnt === 2 && empty === 1) twos++;
    }
    let mob = 0;
    const on = count(board, val);
    if (!(placed[player] < 9)) {
      if (flying && on === 3) {
        mob = 10;
      } else {
        for (let p = 0; p < 24; p++) {
          if (board[p] === val) {
            for (const d of ADJ[p]) if (board[d] === 0) mob++;
          }
        }
      }
    }
    return on * 9 + mills * 6 + twos * 2 + mob;
  }

  /** Positiv = gut für 'player'. */
  function evaluate(board, placed, player, flying) {
    const me = player;
    const opp = 1 - player;
    return evalSide(board, me + 1, placed, me, flying) - evalSide(board, opp + 1, placed, opp, flying);
  }

  /** Schlagzüge zuerst (stabil wie Pythons sorted). */
  function order(moves) {
    const caps = [], rest = [];
    for (const m of moves) (m[3] !== null ? caps : rest).push(m);
    return caps.concat(rest);
  }

  class MuehleGame extends PG.Game {
    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      this.diff = PG.clamp(Math.trunc(Number(this.opts.difficulty) || 0), 0, 2);
      if (this.opts.difficulty == null) this.diff = 1;
      this.flying = this.opts.flying == null ? true : !!this.opts.flying;

      this.makeFonts();
      this.wins = [0, 0];
      this.starter = 0;
      this.buildSetupLayout();
      this.newRound();
      this.state = SETUP;
    }

    /** Theme-Schriften, Größe abhängig von der Fensterhöhe. */
    makeFonts() {
      this.small = ui.font(Math.max(14, Math.floor(this.height / 34)));
      this.tiny = ui.font(Math.max(12, Math.floor(this.height / 44)));
      this.huge = ui.font(Math.max(26, Math.floor(this.height / 12)), true);
    }

    layout() {
      this.hudH = 46;
      // Web: 10 % kleiner als in Python, sonst ragen die Steine der unteren
      // Reihe bei 800x600 über den Fensterrand hinaus.
      const size = Math.floor(Math.min(this.width - 60, this.height - this.hudH - 40) * 0.9);
      this.bsize = size;
      this.bx = Math.floor((this.width - size) / 2);
      this.by = this.hudH + Math.max(10, Math.floor((this.height - this.hudH - size) / 2));
      this.step = size / 6.0;
      this.pr = Math.max(9, Math.floor(this.step * 0.28)); // Steinradius
      this.pts = POS.map(([col, row]) => [Math.floor(this.bx + col * this.step), Math.floor(this.by + row * this.step)]);
      // Grundplatte hinter dem Liniennetz (im Fenster/unter dem HUD halten)
      const pad = Math.max(10, Math.min(Math.floor(this.step * 0.5), this.bx - 8, this.by - this.hudH - 8));
      this.plateRect = new PG.Rect(this.bx - pad, this.by - pad, size + 2 * pad, size + 2 * pad);
    }

    newRound() {
      this.board = new Array(24).fill(0);
      this.placed = [0, 0];
      this.player = this.starter;
      this.removeMode = false;
      this.removable = [];
      this.sel = null;
      this.targets = [];
      this.lastSpot = null;
      this.millSpots = [];
      this.millT = 0.0; // Restzeit der KI-Mühlen-Anzeige
      this.winner = null;
      this.msg = null;
      this.msgT = 0.0;
      this.aiDelay = 0.6;
      this.aiJob = null; // laufende, über Frames verteilte KI-Suche
      this.layout();
    }

    // ===================================================== Setup-Screen
    buildSetupLayout() {
      const cx = Math.floor(this.width / 2);
      const bw = Math.min(360, this.width - 60);
      const y0 = Math.floor(this.height * 0.30);
      this.diffRects = [0, 1, 2].map((i) => new PG.Rect(cx - Math.floor(bw / 2), y0 + i * 52, bw, 44));
      this.flyRect = new PG.Rect(cx - Math.floor(bw / 2), y0 + 3 * 52 + 8, bw, 44);
      this.startRect = new PG.Rect(cx - 95, y0 + 4 * 52 + 22, 190, 46);
    }

    saveSetting(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "1" || k === "2" || k === "3") {
          this.diff = Number(k) - 1;
          this.saveSetting("difficulty", this.diff);
          this.playSound("click");
        } else if (k === "Up" || k === "w" || k === "W") {
          this.diff = PG.mod(this.diff - 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "Down" || k === "s" || k === "S") {
          this.diff = PG.mod(this.diff + 1, 3);
          this.saveSetting("difficulty", this.diff);
          this.playSound("move");
        } else if (k === "f" || k === "F") {
          this.flying = !this.flying;
          this.saveSetting("flying", this.flying);
          this.playSound("select");
        } else if (k === "Return" || k === "space") {
          this.startPlay();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.diffRects.length; i++) {
          if (this.diffRects[i].collidepoint(ev.pos)) {
            this.diff = i;
            this.saveSetting("difficulty", i);
            this.playSound("click");
            return;
          }
        }
        if (this.flyRect.collidepoint(ev.pos)) {
          this.flying = !this.flying;
          this.saveSetting("flying", this.flying);
          this.playSound("select");
          return;
        }
        if (this.startRect.collidepoint(ev.pos)) this.startPlay();
      }
    }

    startPlay() {
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (this.state === OVER) {
        if (ev.kind === "keydown") {
          if (ev.key === "Return" || ev.key === "space") {
            this.restart();
          } else if (ev.key === "s" || ev.key === "S") {
            this.gameOver = false;
            this.state = SETUP;
            this.playSound("click");
          }
        } else if (ev.kind === "mousedown") {
          this.restart();
        }
        return;
      }
      if (this.state !== PLAY) return;
      if (!this.humanTurn()) return;
      if (ev.kind === "mousedown") {
        const p = this.spotAt(ev.pos);
        if (p !== null) this.clickSpot(p);
      } else if (ev.kind === "keydown" && ev.key === "Escape") {
        this.sel = null;
        this.targets = [];
      }
    }

    humanTurn() {
      return this.player === 0;
    }

    spotAt(pos) {
      for (let i = 0; i < this.pts.length; i++) {
        const [x, y] = this.pts[i];
        if ((pos[0] - x) ** 2 + (pos[1] - y) ** 2 <= (this.pr + 6) ** 2) return i;
      }
      return null;
    }

    clickSpot(p) {
      const val = this.player + 1;
      const opp = val === 1 ? 2 : 1;
      if (this.removeMode) {
        if (this.board[p] === opp && this.removable.includes(p)) {
          this.board[p] = 0;
          this.lastSpot = p;
          this.playSound("hit");
          this.removeMode = false;
          this.removable = [];
          this.endTurn();
        } else {
          this.playSound("click");
        }
        return;
      }
      if (this.placed[this.player] < 9) {
        // Setz-Phase
        if (this.board[p] === 0) {
          this.board[p] = val;
          this.placed[this.player] += 1;
          this.lastSpot = p;
          this.afterAction(p, val, opp);
        } else {
          this.playSound("click");
        }
        return;
      }
      // Zieh-/Spring-Phase
      if (this.board[p] === val) {
        this.sel = p;
        this.targets = this.moveTargets(p, val);
        this.playSound("click");
      } else if (this.sel !== null && this.targets.includes(p)) {
        this.board[this.sel] = 0;
        this.board[p] = val;
        this.lastSpot = p;
        this.sel = null;
        this.targets = [];
        this.afterAction(p, val, opp);
      } else {
        this.sel = null;
        this.targets = [];
      }
    }

    moveTargets(p, val) {
      const on = count(this.board, val);
      if (this.flying && on === 3) return ALL_POINTS.filter((d) => this.board[d] === 0);
      return ADJ[p].filter((d) => this.board[d] === 0);
    }

    /** Nach Setzen/Ziehen: Mühle? -> Entfernen, sonst Zugende. */
    afterAction(p, val, opp) {
      if (formsMill(this.board, p, val)) {
        this.millSpots = MILLS_AT[p].filter((m) => m.every((q) => this.board[q] === val));
        this.millT = 0.0; // Anzeige gehört jetzt dem Menschen
        this.removable = removable(this.board, opp);
        if (this.removable.length) {
          this.removeMode = true;
          this.playSound("point");
          return;
        }
      }
      this.playSound("lock");
      this.endTurn();
    }

    endTurn() {
      this.millSpots = [];
      this.player = 1 - this.player;
      this.sel = null;
      this.targets = [];
      this.checkState();
      if (this.state === PLAY && this.player === 1) this.aiDelay = 0.45;
    }

    checkState() {
      // Gewinn nur in der Zieh-Phase (beide fertig gesetzt).
      if (this.placed[0] >= 9 && this.placed[1] >= 9) {
        const val = this.player + 1;
        if (count(this.board, val) < 3) {
          this.winner = 1 - this.player;
          this.end();
          return;
        }
        if (!hasMoves(this.board, this.placed, this.player, this.flying)) {
          this.winner = 1 - this.player;
          this.end();
        }
      }
    }

    end() {
      this.state = OVER;
      this.aiJob = null;
      this.wins[this.winner] += 1;
      if (this.winner === 0) {
        this.score = this.wins[0];
        this.playSound("win");
        this.reportResult(true);
      } else {
        this.playSound("gameover");
        this.reportResult(false);
      }
      this.gameOver = true;
    }

    restart() {
      this.starter = 1 - this.starter;
      this.gameOver = false;
      this._resultReported = false; // neue Partie -> Ergebnis wieder meldbar
      this.newRound();
      this.state = PLAY;
      this.playSound("click");
    }

    // ===================================================== KI
    update(dt) {
      if (this.msgT > 0) {
        this.msgT -= dt;
        if (this.msgT <= 0) this.msg = null;
      }
      if (this.millT > 0) {
        // KI-Mühle nur kurz anzeigen, dann wieder ausblenden.
        this.millT -= dt;
        if (this.millT <= 0) {
          this.millT = 0.0;
          if (!this.removeMode) this.millSpots = [];
        }
      }
      if (this.state === PLAY && this.player === 1) {
        this.aiDelay -= dt;
        if (this.aiDelay <= 0) this.aiPlay();
      }
    }

    aiPlay() {
      if (!this.aiJob) {
        const res = this.startAiSearch();
        if (res === undefined) return; // Suche läuft (über mehrere Frames)
        this.applyAiMove(res);
        return;
      }
      const mv = this.stepAiSearch();
      if (mv !== undefined) this.applyAiMove(mv);
    }

    applyAiMove(mv) {
      this.aiJob = null;
      if (mv === null) {
        this.checkState();
        return;
      }
      const [kind, a, b, rp] = mv;
      const val = 2;
      if (kind === "place") {
        this.board[a] = val;
        this.placed[1] += 1;
        this.lastSpot = a;
      } else {
        this.board[a] = 0;
        this.board[b] = val;
        this.lastSpot = b;
      }
      const spot = kind === "place" ? a : b;
      if (rp !== null) {
        const mills = MILLS_AT[spot].filter((m) => m.every((q) => this.board[q] === val));
        this.board[rp] = 0;
        this.playSound("hit");
        this.endTurn();
        // endTurn() löscht millSpots sofort - die geschlossene
        // KI-Mühle danach kurz anzeigen, sonst sieht man sie nie.
        this.millSpots = mills;
        this.millT = 1.2;
      } else {
        this.playSound("lock");
        this.endTurn();
      }
    }

    /**
     * Startet die KI-Zugwahl (entspricht _pick_ai_move). Gibt den Zug (oder null)
     * zurück, wenn sofort entschieden ist, sonst undefined (Suche läuft weiter).
     */
    startAiSearch() {
      const moves = genMoves(this.board, this.placed, 1, this.flying);
      if (!moves.length) return null;
      if (PG.rand.random() < (this.diff === 0 ? 0.55 : this.diff === 1 ? 0.18 : 0.0)) {
        const caps = moves.filter((m) => m[3] !== null);
        if (caps.length && PG.rand.random() < 0.7) return PG.rand.choice(caps);
        return PG.rand.choice(moves);
      }
      this.nodes = 0;
      this.aiJob = {
        moves,
        ordered: order(moves),
        idx: 0,
        depth: DEPTHS[this.diff],
        budgetMs: TIME_BUDGET[this.diff] * 1000,
        spentMs: 0, // bisher verbrauchte Rechenzeit (statt Wanduhr-Deadline)
        bestVal: -1e9,
        best: [],
      };
      return this.stepAiSearch();
    }

    /** Ein Frame-Stück der Wurzelsuche. undefined = noch nicht fertig. */
    stepAiSearch() {
      const job = this.aiJob;
      this.sliceStart = nowMs();
      this.jobSpentBefore = job.spentMs;
      let done = false;
      while (job.idx < job.ordered.length) {
        const mv = job.ordered[job.idx++];
        const [nb, npl] = applyMove(this.board, this.placed, 1, mv);
        const val = -this.search(nb, npl, 0, job.depth - 1, -1e9, 1e9);
        if (job.best.length && this.timeUp()) {
          done = true;
          break;
        }
        if (val > job.bestVal) {
          job.bestVal = val;
          job.best = [mv];
        } else if (val === job.bestVal) {
          job.best.push(mv);
        }
        if (nowMs() - this.sliceStart > SLICE_MS) break;
      }
      job.spentMs += nowMs() - this.sliceStart;
      if (!done && job.idx < job.ordered.length) return undefined;
      return job.best.length ? PG.rand.choice(job.best) : PG.rand.choice(job.moves);
    }

    /** Zeitbudget der KI aufgebraucht? (plus Notbremse pro Frame) */
    timeUp() {
      const slice = nowMs() - this.sliceStart;
      return this.jobSpentBefore + slice > this.aiJob.budgetMs || slice > HARD_SLICE_MS;
    }

    search(board, placed, player, depth, alpha, beta) {
      this.nodes += 1;
      if (this.nodes > NODE_BUDGET || ((this.nodes & 1023) === 0 && this.timeUp())) {
        return evaluate(board, placed, player, this.flying);
      }
      // Verlust, wenn zugunfähig oder (in Zieh-Phase) unter 3 Steine.
      if (placed[0] >= 9 && placed[1] >= 9 && count(board, player + 1) < 3) {
        return -20000 + (5 - depth);
      }
      const moves = genMoves(board, placed, player, this.flying);
      if (!moves.length) return -20000 + (5 - depth);
      if (depth <= 0) return evaluate(board, placed, player, this.flying);
      let best = -1e9;
      for (const mv of order(moves)) {
        const [nb, npl] = applyMove(board, placed, player, mv);
        const val = -this.search(nb, npl, 1 - player, depth - 1, -beta, -alpha);
        if (val > best) best = val;
        if (best > alpha) alpha = best;
        if (alpha >= beta) break;
      }
      return best;
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      ui.drawBackground(ctx, this.width, this.height, false, true);
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        return;
      }
      this.drawHud(ctx);
      this.drawBoard(ctx);
      if (this.state === OVER) this.drawOver(ctx);
    }

    drawBoard(ctx) {
      // Grundplatte, damit sich das Liniennetz vom Hintergrund abhebt
      draw.rect(ctx, COL_PLATE, this.plateRect, 0, 12);
      draw.rect(ctx, ui.mix(COL_PLATE, this.accent, 0.45), this.plateRect, 1, 12);
      // Linien (jede Kante einmal)
      for (let i = 0; i < 24; i++) {
        for (const j of ADJ[i]) {
          if (j > i) draw.line(ctx, COL_LINE, this.pts[i], this.pts[j], 3);
        }
      }
      // Mühlen-Hervorhebung
      for (const m of this.millSpots) draw.line(ctx, COL_MILL, this.pts[m[0]], this.pts[m[2]], 5);
      // Punkte + Steine
      const human = this.state === PLAY && this.humanTurn();
      const pr = this.pr;
      for (let i = 0; i < 24; i++) {
        const [x, y] = this.pts[i];
        const v = this.board[i];
        if (v === 0) {
          draw.circle(ctx, COL_SPOT, [x, y], Math.max(4, Math.floor(pr / 2)));
        } else {
          const base = v === 1 ? COL_P1 : COL_P2;
          const hi = v === 1 ? COL_P1_HI : COL_P2_HI;
          draw.circle(ctx, [10, 16, 12], [x, y + 2], pr);
          draw.circle(ctx, base, [x, y], pr);
          draw.circle(ctx, hi, [x - Math.floor(pr / 3), y - Math.floor(pr / 3)], Math.max(2, Math.floor(pr / 4)));
        }
      }
      // Setz-Hinweise (freie Punkte) in der Setz-Phase
      if (human && !this.removeMode && this.placed[this.player] < 9) {
        for (let i = 0; i < 24; i++) {
          if (this.board[i] === 0) draw.circle(ctx, COL_HINT, this.pts[i], Math.max(3, Math.floor(pr / 4)));
        }
      }
      // Auswahl + Zugziele
      if (human && this.sel !== null) {
        draw.circle(ctx, COL_SEL, this.pts[this.sel], pr + 3, 3);
        for (const d of this.targets) draw.circle(ctx, COL_HINT, this.pts[d], Math.max(4, Math.floor(pr / 3)));
      }
      // Entfern-Markierung
      if (human && this.removeMode) {
        const k = ui.pulse(2.8, 0.0, 1.0);
        for (const i of this.removable) draw.circle(ctx, COL_REMOVE, this.pts[i], pr + 2 + Math.floor(3 * k), 3);
      }
      // Letzter Punkt (sanft pulsierender Akzentring)
      if (this.lastSpot !== null && this.board[this.lastSpot] !== 0) {
        const col = ui.mix(COL_SPOT, this.accent, 0.55 + 0.35 * ui.pulse(1.6, 0.0, 1.0));
        draw.circle(ctx, col, this.pts[this.lastSpot], pr + 4, 2);
      }
    }

    drawHud(ctx) {
      const panel = new PG.Rect(8, 6, this.width - 16, this.hudH - 10);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      const cy = panel.centery;
      // Steinbestand: verbleibend zu setzen + auf dem Brett
      [COL_P1, COL_P2].forEach((col, idx) => {
        const on = count(this.board, idx + 1);
        const left = 9 - this.placed[idx];
        const txt = String(on) + (left ? ` (+${left})` : "");
        if (idx === 0) {
          draw.circle(ctx, col, [panel.x + 16, cy], 8);
          draw.circle(ctx, ui.BORDER_LIGHT, [panel.x + 16, cy], 8, 1);
          ui.text(ctx, txt, panel.x + 30, cy, this.small, ui.TEXT, "midleft");
        } else {
          draw.circle(ctx, col, [panel.right - 16, cy], 8);
          draw.circle(ctx, ui.BORDER_LIGHT, [panel.right - 16, cy], 8, 1);
          ui.text(ctx, txt, panel.right - 30, cy, this.small, ui.TEXT, "midright");
        }
      });
      if (this.state === PLAY) {
        let mid;
        if (this.removeMode && this.humanTurn()) mid = t("mill.remove");
        else if (this.player === 1) mid = t("mill.ai_thinks");
        else if (this.placed[this.player] < 9) mid = t("mill.place_you");
        else mid = t("mill.move_you");
        ui.text(ctx, mid, Math.floor(this.width / 2), cy, this.small, this.accent, "center");
      }
    }

    drawOver(ctx) {
      const cx = Math.floor(this.width / 2);
      const won = this.winner === 0;
      const head = won ? t("mill.win_you") : t("mill.win_ai");
      const headCol = won ? this.accent : ui.TEXT_DIM;
      const hint = t("mill.new_round");
      const w = Math.min(this.width - 24, Math.max(this.huge.width(head), this.tiny.width(hint)) + 64);
      const panel = new PG.Rect(cx - Math.floor(w / 2), Math.floor(this.height / 2) - 48, w, 96);
      ui.drawPanel(ctx, panel, { shadow: false, accentTop: this.accent });
      ui.text(ctx, head, cx, panel.y + 36, this.huge, headCol, "center");
      ui.text(ctx, hint, cx, panel.y + 74, this.tiny, ui.TEXT_DIM, "center");
    }

    drawSetup(ctx) {
      ui.drawTitle(ctx, this.width, t("mill.title"), {
        subtitle: t("web.muehle.subtitle"), y: Math.floor(this.height * 0.14),
        big: this.huge, small: this.small, accent: this.accent,
      });
      const font = ui.font(22);
      this.diffRects.forEach((rc, i) => {
        ui.drawButton(ctx, rc, t("mill.diff." + DIFFS[i]), font, i === this.diff, { accent: this.accent });
      });
      // Fliegen-Schalter
      const state = this.flying ? t("common.on") : t("common.off");
      ui.drawButton(ctx, this.flyRect, t("mill.flying") + ": " + state, font, this.flying, { accent: this.accent });
      ui.drawButton(ctx, this.startRect, t("common.start"), font, true, { accent: this.accent });
      ui.drawFooter(ctx, this.width, this.height, t("mill.setup_hint"), this.tiny);
    }
  }

  PG.register(MuehleGame, {
    id: "MuehleGame",
    key: "muehle",
    name: { default: "Nine Men's Morris", de: "Mühle", fr: "Moulin", es: "Molino", pt: "Trilha" },
    settingsKey: "muehle",
    defaults: { difficulty: 1, flying: true },
  });
})();
