/*
 * wordle.js - Wordle (Port von games/wordle.py)
 * ==============================================
 * Errate das gesuchte Wort in wenigen Versuchen. Vier Modi:
 *
 * - Endlos: ein Wort nach dem anderen (6 Versuche je Wort). Jedes gelöste Wort
 *   bringt Punkte (weniger Versuche = mehr), das erste NICHT gelöste Wort beendet
 *   die Partie. Nur Endlos mit 5 Buchstaben zählt für den Highscore; die anderen
 *   Längen haben eigene Bestwerte.
 * - Tageswort: ein Wort pro Tag, je Sprache und Länge für alle gleich
 *   (seedrand.js - PC und Browser liefern dasselbe Wort). Angefangene
 *   Tageswörter werden gespeichert; danach gibt es Ergebnis, Statistik und den
 *   Countdown bis morgen. Serie = Tageswörter in Folge.
 * - Dordle: zwei Wörter gleichzeitig, 7 Versuche - jeder Rateversuch gilt für
 *   beide Bretter.
 * - Quordle: vier Wörter gleichzeitig, 9 Versuche. Die Tasten der
 *   Bildschirmtastatur zeigen die Farben je Brett geviertelt.
 *
 * Setup vor jeder Partie: Wortlänge 4-7, harter Modus (gefundene Hinweise
 * müssen weiterverwendet werden) und eine Farbenblind-Palette (Orange/Blau
 * statt Grün/Gelb). Nach jeder Partie: Statistik je Sprache/Länge/Modus mit
 * Versuchsverteilung und "Ergebnis teilen" (Emoji-Raster in die Zwischenablage,
 * navigator.clipboard mit textarea-Fallback für file://).
 *
 * Die Wortlisten je Sprache UND Länge liegen in js/games/wordle_words/ und
 * werden erst beim Start nachgeladen (wordle_words.js). Statistik, Bestwerte
 * und angefangene Tageswörter stehen in PG.store "mem.wordle" (gleiches Format
 * wie die mem.json-Section "wordle" am PC).
 *
 * Steuerung: Buchstaben A-Z tippen (über ev.char, damit QWERTZ/AZERTY stimmen),
 * Enter = raten, Backspace = löschen, die Bildschirmtastatur ist anklickbar
 * (QWERTZ für de/cs/sl/hr, AZERTY für Französisch, sonst QWERTY). Das
 * Regler-Symbol oben rechts führt zurück ins Setup. Nach einer Partie:
 * Enter = nochmal, C = teilen, S = Setup, Leertaste = Bretter ansehen.
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;

  // ----------------------------------------------------------------- Modi
  const MODES = [["endless", "wd.mode.endless"], ["daily", "wd.mode.daily"], ["dordle", "wd.mode.dordle"], ["quordle", "wd.mode.quordle"]];
  const BOARDS = { endless: 1, daily: 1, dordle: 2, quordle: 4 };
  const MAX_ROWS = { endless: 6, daily: 6, dordle: 7, quordle: 9 };
  const SHARE_NAME = { endless: "Wordle", daily: "Wordle", dordle: "Dordle", quordle: "Quordle" };
  // Frühere Modus-Schlüssel (Normal/Hart als eigene Knöpfe) -> heutiger Modus.
  const LEGACY_MODES = { normal: "endless", hard: "endless", single: "endless" };
  // Nur Endlos mit dieser Wortlänge füllt den Highscore (wie vor dem Ausbau).
  const SCORE_LENGTH = 5;
  const LENGTHS = [4, 5, 6, 7];
  const DEFAULT_LENGTH = 5;
  const STORE_KEY = "mem.wordle"; // entspricht store.load_section("wordle")

  const SETUP = "setup", PLAY = "play", REVEAL = "reveal", SOLVED = "solved", DONE = "done";

  // ----------------------------------------------------------------- Farben
  // Kachelfarben sind die Identität des Spiels und bleiben in jedem Theme
  // gleich; alles drumherum (Panels, Texte, Tasten) kommt aus ui.*.
  const PALETTES = {
    false: { correct: [106, 170, 100], present: [201, 180, 88] },
    true: { correct: [245, 121, 58], present: [133, 192, 249] },
  };
  const COL_ABSENT = [58, 58, 62];
  const COL_LETTER = [240, 241, 246];
  const COL_LOSE = [225, 110, 100];

  // Emoji fürs Teilen (Farbenblind: Orange/Blau wie im Original).
  const EMOJI = {
    false: { correct: "\u{1F7E9}", present: "\u{1F7E8}" },
    true: { correct: "\u{1F7E7}", present: "\u{1F7E6}" },
  };
  const EMOJI_ABSENT = "⬛";
  const EMOJI_EMPTY = "⬜";
  const EMOJI_FAIL = "\u{1F7E5}";
  const KEYCAP = "️⃣";

  // ----------------------------------------------------------------- Zeiten
  const FLIP_TIME = 0.26; // eine Kachel dreht sich um
  const MESSAGE_TIME = 1.6; // Meldung bleibt stehen
  const SHAKE_TIME = 0.4; // abgelehnte Zeile wackelt
  const POP_TIME = 0.11; // getippter Buchstabe "ploppt" auf
  const BOUNCE_TIME = 0.5; // gelöste Zeile hüpft
  const PANEL_DELAY = 0.45; // Ergebnis erscheint nach der letzten Aufdeckung
  // Höhe des Highscore-Banners, den die App bei Game Over unten einblendet -
  // das Ergebnis-Panel lässt den Platz frei.
  const BANNER_SPACE = 50;

  // ----------------------------------------------------------------- Tastatur
  const KEYBOARDS = {
    qwerty: ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"],
    qwertz: ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"],
    azerty: ["AZERTYUIOP", "QSDFGHJKLM", "WXCVBN"],
  };
  // Tschechisch, Slowenisch und Kroatisch tippen wie Deutsch auf QWERTZ.
  const LANG_KEYBOARD = { de: "qwertz", cs: "qwertz", sl: "qwertz", hr: "qwertz", fr: "azerty" };

  // Rangordnung der Buchstaben-Zustände (höher gewinnt in der Tastatur-Färbung)
  const RANK = { absent: 1, present: 2, correct: 3 };
  const rank = (st) => RANK[st] || 0;

  // =====================================================================
  //  Spiellogik (1:1 wie in wordle.py)
  // =====================================================================
  /**
   * Standard-Wordle-Bewertung mit korrekter Doppelbuchstaben-Zählung: erst alle
   * Treffer an der richtigen Stelle, dann die übrigen Buchstaben von links nach
   * rechts - jeder Buchstabe der Lösung wird höchstens einmal "verbraucht".
   */
  function evaluate(guess, answer) {
    const n = answer.length;
    const result = new Array(n).fill("absent");
    const rest = answer.split("");
    for (let i = 0; i < n; i++) {
      if (guess[i] === answer[i]) {
        result[i] = "correct";
        rest[i] = null;
      }
    }
    for (let i = 0; i < n; i++) {
      if (result[i] === "correct") continue;
      const j = rest.indexOf(guess[i]);
      if (j >= 0) {
        result[i] = "present";
        rest[j] = null;
      }
    }
    return result;
  }

  /**
   * Verstößt 'guess' im harten Modus gegen einen Hinweis? history = [[wort,
   * ergebnis], ...] eines Bretts. Liefert null oder [i18n-Schlüssel, Werte]:
   * grüne Buchstaben bleiben an ihrer Stelle, grüne/gelbe kommen mindestens so
   * oft wieder vor wie im Hinweis (Standard-Regel des Originals).
   */
  function hardProblem(guess, history) {
    for (const [word, result] of history) {
      for (let i = 0; i < result.length; i++) {
        if (result[i] === "correct" && guess[i] !== word[i]) return ["wd.hard_green", { n: i + 1, c: word[i] }];
      }
      const need = {};
      for (let i = 0; i < result.length; i++) {
        if (result[i] !== "absent") need[word[i]] = (need[word[i]] || 0) + 1;
      }
      for (const ch of Object.keys(need)) {
        const have = guess.split("").filter((x) => x === ch).length;
        if (have < need[ch]) return ["wd.hard_yellow", { c: ch }];
      }
    }
    return null;
  }

  const orderCache = new Map();

  /**
   * Das Tageswort für Sprache und Länge (am PC und im Browser gleich). Die
   * Lösungswörter werden einmal je Sprache/Länge fest gemischt (Seed aus
   * "wordle"/Sprache/Länge); der Tag wählt die Stelle in dieser Reihenfolge.
   */
  function dailyAnswer(words, lang, length, date) {
    const n = words.length;
    const key = lang + ":" + length + ":" + n;
    let order = orderCache.get(key);
    if (!order) {
      order = Array.from({ length: n }, (_, i) => i);
      new PG.seedrand.Rand(PG.seedrand.seedFrom("wordle", lang, length)).shuffle(order);
      orderCache.set(key, order);
    }
    return words[order[PG.mod(PG.seedrand.dayIndex(date), n)]];
  }

  /** Sekunden bis zum nächsten lokalen Tageswechsel. */
  function secondsToMidnight(now) {
    const d = now || new Date();
    const midnight = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1, 0, 0, 0, 0);
    return Math.max(0, Math.round((midnight - d) / 1000));
  }

  function formatHms(seconds) {
    seconds = Math.max(0, Math.floor(seconds));
    const p = (v) => String(v).padStart(2, "0");
    return p(Math.floor(seconds / 3600)) + ":" + p(Math.floor(seconds / 60) % 60) + ":" + p(seconds % 60);
  }

  // ----------------------------------------------------------------- Statistik
  const statsKey = (mode, lang, length) => mode + ":" + lang + ":" + length;
  const isCount = (v) => Number.isInteger(v) && v >= 0;

  /** PG.store "mem.wordle" - fehlende/kaputte Teile werden ergänzt. */
  function loadData() {
    let data = PG.store.get(STORE_KEY, {});
    if (!data || typeof data !== "object" || Array.isArray(data)) data = {};
    for (const key of ["stats", "daily", "best"]) {
      if (!data[key] || typeof data[key] !== "object" || Array.isArray(data[key])) data[key] = {};
    }
    return data;
  }

  function saveData(data) {
    PG.store.set(STORE_KEY, data);
  }

  /** Bereinigte Kopie der Statistik (Partien, Siege, Serien, Verteilung). */
  function getStats(data, mode, lang, length) {
    const rows = MAX_ROWS[mode];
    let raw = (data.stats || {})[statsKey(mode, lang, length)];
    if (!raw || typeof raw !== "object") raw = {};
    const num = (key) => (isCount(raw[key]) ? raw[key] : 0);
    let dist = Array.isArray(raw.dist) ? raw.dist.map((v) => (isCount(v) ? v : 0)).slice(0, rows) : [];
    while (dist.length < rows) dist.push(0);
    const out = { played: num("played"), won: num("won"), streak: num("streak"), best: num("best"), dist };
    if (Number.isInteger(raw.last_day)) out.last_day = raw.last_day;
    out.won = Math.min(out.won, out.played);
    return out;
  }

  /**
   * Trägt eine beendete Partie ein und liefert die neue Statistik. Tageswort
   * (day = dayIndex): die Serie wächst nur, wenn auch das Tageswort von GESTERN
   * gelöst wurde - ein ausgelassener Tag beendet sie. Sonst zählt jeder Sieg.
   */
  function recordGame(data, mode, lang, length, won, tries, day) {
    const st = getStats(data, mode, lang, length);
    st.played += 1;
    if (won) {
      st.won += 1;
      if (tries >= 1 && tries <= st.dist.length) st.dist[tries - 1] += 1;
      if (day != null && st.last_day !== day - 1) st.streak = 1;
      else st.streak += 1;
      st.best = Math.max(st.best, st.streak);
      if (day != null) st.last_day = day;
    } else {
      st.streak = 0;
    }
    if (!data.stats) data.stats = {};
    data.stats[statsKey(mode, lang, length)] = st;
    return st;
  }

  /** Aktuelle Serie für die Anzeige (Tageswort: verfällt nach einem Tag Pause). */
  function shownStreak(st, mode, today) {
    if (mode !== "daily") return st.streak;
    today = today == null ? PG.seedrand.dayIndex() : today;
    if (st.last_day === today || st.last_day === today - 1) return st.streak;
    return 0;
  }

  // ----------------------------------------------------------------- Teilen
  /**
   * Ergebnis als Emoji-Raster zum Teilen. boards = [[ergebnisse, gelöstNach],
   * ...] - je Brett die Ergebnis-Zeilen bis zur Lösung und die Anzahl Versuche
   * (null = nicht gelöst).
   */
  function shareText(mode, lang, length, boards, hard, colorblind, date) {
    const rows = MAX_ROWS[mode];
    const palette = EMOJI[!!colorblind];
    let head = "PyGameZ " + SHARE_NAME[mode];
    if (mode === "daily") head += " " + (date || PG.seedrand.todayStr());
    head += " · " + lang.toUpperCase() + " · " + length;
    const star = hard ? "*" : "";
    const line = (results, r) => (r >= results.length ? EMOJI_EMPTY.repeat(length) : results[r].map((st) => palette[st] || EMOJI_ABSENT).join(""));
    const score = (solved) => (solved ? String(solved) : "X");
    let lines;
    if (boards.length === 1) {
      const [results, solved] = boards[0];
      lines = [head + " " + score(solved) + "/" + rows + star, ""];
      for (let r = 0; r < results.length; r++) lines.push(line(results, r));
    } else if (boards.length === 2) {
      lines = [head + " " + boards.map((b) => score(b[1])).join("&") + "/" + rows + star, ""];
      const used = Math.max(...boards.map((b) => b[0].length));
      for (let r = 0; r < used; r++) lines.push(line(boards[0][0], r) + " " + line(boards[1][0], r));
    } else {
      const badge = (solved) => (solved ? String(solved) + KEYCAP : EMOJI_FAIL);
      lines = [head + star, badge(boards[0][1]) + badge(boards[1][1]), badge(boards[2][1]) + badge(boards[3][1]), ""];
      for (const [a, b] of [[0, 1], [2, 3]]) {
        const used = Math.max(boards[a][0].length, boards[b][0].length);
        for (let r = 0; r < used; r++) lines.push(line(boards[a][0], r) + " " + line(boards[b][0], r));
        if (a === 0) lines.push("");
      }
    }
    return lines.join("\n");
  }

  /**
   * Legt 'text' in die Zwischenablage und meldet done(true/false). Erst über
   * navigator.clipboard, sonst (file://, ältere Browser) über ein
   * unsichtbares textarea + execCommand("copy").
   */
  function copyToClipboard(text, done) {
    const fallback = () => {
      let ok = false;
      try {
        const ta = document.createElement("textarea");
        ta.value = text;
        ta.setAttribute("readonly", "");
        ta.style.position = "fixed";
        ta.style.left = "-9999px";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        ok = document.execCommand("copy");
        ta.remove();
      } catch (e) {
        ok = false;
      }
      done(ok);
    };
    try {
      if (typeof navigator !== "undefined" && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(() => done(true), fallback);
        return;
      }
    } catch (e) {
      /* weiter mit dem Fallback */
    }
    fallback();
  }

  // =====================================================================
  //  Ein Brett (ein gesuchtes Wort)
  // =====================================================================
  class Board {
    constructor(answer) {
      this.answer = answer;
      this.rows = []; // [[wort, ergebnis], ...] bis zur Lösung
      this.solvedAt = null; // Anzahl Versuche bis zur Lösung
      this.keys = {}; // Buchstabe -> bester bekannter Zustand
      this.solveT = null; // Animationszeit der Lösung (Hüpfen)
    }
    get solved() {
      return this.solvedAt !== null;
    }
    apply(guess, result, animT = 0) {
      this.rows.push([guess, result]);
      for (let i = 0; i < guess.length; i++) {
        if (rank(result[i]) > rank(this.keys[guess[i]])) this.keys[guess[i]] = result[i];
      }
      if (result.every((st) => st === "correct")) {
        this.solvedAt = this.rows.length;
        this.solveT = animT;
      }
    }
  }

  /** Getippten Buchstaben (A-Z) aus dem Ereignis holen, sonst null. */
  function letterOf(ev) {
    const ch = ev.char && ev.char.length === 1 ? ev.char : ev.key && ev.key.length === 1 ? ev.key : "";
    // Erst auf ASCII-Buchstaben prüfen, dann groß schreiben (wie isascii()/
    // isalpha() im Original) - sonst würde z.B. "ı".toUpperCase() zu "I".
    return /^[A-Za-z]$/.test(ch) ? ch.toUpperCase() : null;
  }

  // =====================================================================
  //  Das Spiel
  // =====================================================================
  class WordleGame extends PG.Game {
    /** Highscore gibt es nur in Endlos mit 5 Buchstaben. */
    get showHighscoreBanner() {
      return (this.gameMode || "endless") === "endless" && (this.length || SCORE_LENGTH) === SCORE_LENGTH;
    }

    // ===================================================== Aufbau / Reset
    reset() {
      this.score = 0;
      this.gameOver = false;
      const mode = LEGACY_MODES[this.mode] || this.mode;
      this.gameMode = BOARDS[mode] ? mode : "endless";
      this.lang = PG.lang;
      if (this.mode === "hard" && !this.cleanOpts().hard) {
        // Früherer Modus-Knopf "Hart" = Endlos mit eingeschaltetem harten
        // Modus (wird zur Einstellung).
        this.saveOpt("hard", true);
      }
      const opts = this.cleanOpts();
      this.length = opts.length;
      this.hard = opts.hard;
      this.colorblind = opts.colorblind;
      this.data = loadData();

      this.anim = 0; // Animationsuhr (läuft auch nach Game Over)
      this.updated = false;
      this.lastDraw = null;
      this.state = SETUP;
      this.setupFocus = 3; // 0 Länge, 1 hart, 2 farbenblind, 3 Start
      this.setupT = 0;
      this.toggleAnim = {};
      this.loading = false;
      this.dead = false;

      this.boards = [];
      this.guesses = [];
      this.current = "";
      this.reveal = null;
      this.points = 0;
      this.lastPoints = 0;
      this.solvedCount = 0;
      this.solvedT = 0;
      this.message = "";
      this.messageT = 0;
      this.shake = 0;
      this.pops = {};
      this.result = null;
      this.panelHidden = false;
      this.dailyDate = null;
      this.dailyView = false;
      this.doneButtons = [];
      this.donePanel = null;
      this.words = [];
      this.allowed = new Set();

      this.fitCache = new Map();
      this.boardCache = new Map();
      this.makeFonts();
      this.layout();
    }

    destroy() {
      this.dead = true;
    }

    /** Wordle-Einstellungen (settings.wordle) mit Prüfung. */
    cleanOpts() {
      const ws = this.opts || {};
      let length = ws.length;
      if (!Number.isInteger(length) || !LENGTHS.includes(length)) length = DEFAULT_LENGTH;
      return { length, hard: ws.hard === true, colorblind: ws.colorblind === true };
    }

    saveOpt(key, value) {
      this.opts[key] = value;
      this.saveSettings();
    }

    get rows() {
      return MAX_ROWS[this.gameMode];
    }
    get nBoards() {
      return BOARDS[this.gameMode];
    }
    get countsScore() {
      return this.gameMode === "endless" && this.length === SCORE_LENGTH;
    }
    palette() {
      return PALETTES[!!this.colorblind];
    }
    k() {
      return Math.min(this.width / 800, this.height / 600);
    }

    makeFonts() {
      const k = this.k();
      this.fHud = ui.font(Math.max(12, Math.floor(17 * k)), true);
      this.fSmall = ui.font(Math.max(11, Math.floor(15 * k)));
      this.fSmallB = ui.font(Math.max(11, Math.floor(15 * k)), true);
      this.fTiny = ui.font(Math.max(10, Math.floor(12 * k)));
      this.fBig = ui.font(Math.max(18, Math.floor(30 * k)), true);
    }

    /** Größte Schrift <= px, in der 'text' höchstens maxW breit ist. */
    fit(text, maxW, px, bold = false, minPx = 9) {
      const key = text + "|" + Math.floor(maxW) + "|" + px + "|" + (bold ? 1 : 0) + "|" + minPx;
      let f = this.fitCache.get(key);
      if (!f) {
        let size = Math.floor(px);
        f = ui.font(size, bold);
        while (size > minPx && f.width(text) > maxW) {
          size -= 1;
          f = ui.font(size, bold);
        }
        if (this.fitCache.size > 600) this.fitCache.clear();
        this.fitCache.set(key, f);
      }
      return f;
    }

    /** Tastenhinweis "A · B · C": passt er nicht, fallen hintere Teile weg. */
    fitParts(text, maxW, px) {
      const parts = text.split(" · ");
      while (parts.length > 1 && !this.fits(parts.join(" · "), maxW, px)) parts.pop();
      return parts.join(" · ");
    }

    fits(text, maxW, px, bold = false, minPx = 9) {
      return this.fit(text, maxW, px, bold, minPx).width(text) <= maxW;
    }

    /**
     * Text passend verkleinert zeichnen (anchor wie ui.text). Passt er selbst in
     * der kleinsten Größe nicht, wird er mit "…" gekürzt.
     */
    txt(ctx, text, px, color, maxW, bold, anchor, x, y, alpha) {
      text = String(text);
      const f = this.fit(text, maxW, px, bold);
      if (f.width(text) > maxW && maxW > 8) {
        while (text.length > 1 && f.width(text + "…") > maxW) text = text.slice(0, -1);
        text = text.trimEnd() + "…";
      }
      return ui.text(ctx, text, x, y, f, color, anchor, alpha);
    }

    // ===================================================== Layout
    layout() {
      this.layoutSetup();
      this.layoutPlay();
    }

    layoutSetup() {
      const W = this.width, H = this.height;
      const k = this.k();
      this.suTile = Math.max(22, Math.floor(46 * k));
      this.suTitleY = Math.max(10, Math.floor(26 * k));
      this.suSubY = this.suTitleY + this.suTile + Math.max(6, Math.floor(12 * k));
      const top = this.suSubY + this.fSmall.height + Math.max(8, Math.floor(16 * k));
      const footer = Math.max(22, Math.floor(34 * k));
      const bottom = H - footer;
      const margin = Math.max(12, Math.floor(34 * k));
      const gap = Math.max(10, Math.floor(22 * k));
      const colW = Math.floor((W - 2 * margin - gap) / 2);
      const left = new PG.Rect(margin, top, colW, bottom - top);
      // Linke Spalte: Überschrift Länge, 4 Knöpfe, 2 Schalter, Start.
      const labelH = this.fSmall.height;
      const segH = Math.max(28, Math.floor(46 * k));
      const rowH = Math.max(36, Math.floor(58 * k));
      const startH = Math.max(32, Math.floor(50 * k));
      const sp = Math.max(6, Math.floor(12 * k));
      const total = labelH + 4 + segH + sp + rowH + sp + rowH + sp * 2 + startH;
      let y = left.y + Math.max(0, Math.floor((left.h - total) / 2));
      this.suLabelY = y;
      y += labelH + 4;
      const sg = Math.max(4, Math.floor(8 * k));
      const segW = Math.floor((colW - 3 * sg) / 4);
      this.suSeg = LENGTHS.map((_, i) => new PG.Rect(left.x + i * (segW + sg), y, segW, segH));
      y += segH + sp;
      this.suHard = new PG.Rect(left.x, y, colW, rowH);
      y += rowH + sp;
      this.suCb = new PG.Rect(left.x, y, colW, rowH);
      y += rowH + sp * 2;
      this.suStart = new PG.Rect(left.x, y, colW, startH);
      // Rechte Spalte: Statistik, so hoch wie die Einstellungen daneben.
      const stTop = Math.max(top, this.suLabelY - Math.max(4, Math.floor(6 * k)));
      this.suStatsRect = new PG.Rect(margin + colW + gap, stTop, colW, Math.max(this.suStart.bottom - stTop, Math.floor(250 * k)));
    }

    layoutPlay() {
      const W = this.width;
      const k = this.k();
      this.hudH = Math.max(28, Math.floor(44 * k));
      this.gearRect = new PG.Rect(W - this.hudH + 2, 3, this.hudH - 6, this.hudH - 6);
      this.buildKeyboard();
      const n = this.nBoards;
      const pad = n === 1 ? Math.max(8, Math.floor(20 * k)) : Math.max(4, Math.floor(14 * k));
      const lane = n === 1 ? Math.max(16, Math.floor(30 * k)) : Math.max(4, Math.floor(8 * k));
      const top = this.hudH + lane;
      const bottom = this.kbTop - Math.max(6, Math.floor(10 * k));
      const area = new PG.Rect(pad, top, W - 2 * pad, Math.max(40, bottom - top));
      const L = this.length, R = this.rows;
      const options = { 1: [[1, 1]], 2: [[2, 1], [1, 2]], 4: [[4, 1], [2, 2]] }[n];
      let best = null;
      for (const [bc, br] of options) {
        const bgap = n > 1 ? Math.max(6, Math.floor(16 * k)) : 0;
        const inner = n > 1 ? Math.max(3, Math.floor(7 * k)) : 0; // Panel-Rand je Brett
        const cellW = (area.w - (bc - 1) * bgap) / bc - 2 * inner;
        const cellH = (area.h - (br - 1) * bgap) / br - 2 * inner;
        const tgRatio = n === 1 ? 0.09 : 0.07;
        const tile = Math.floor(Math.min(cellW / (L + (L - 1) * tgRatio), cellH / (R + (R - 1) * tgRatio)));
        if (best === null || tile > best[0]) best = [tile, bc, br, bgap, inner, tgRatio];
      }
      let [tile, bc, br, bgap, inner, tgRatio] = best;
      const cap = Math.max(20, Math.floor((n === 1 ? 64 : 52) * k));
      tile = Math.max(6, Math.min(tile, cap));
      const tgap = Math.max(1, Math.round(tile * tgRatio));
      const bw = L * tile + (L - 1) * tgap;
      const bh = R * tile + (R - 1) * tgap;
      const totalW = bc * (bw + 2 * inner) + (bc - 1) * bgap;
      const totalH = br * (bh + 2 * inner) + (br - 1) * bgap;
      const x0 = Math.floor((W - totalW) / 2);
      const y0 = area.y + Math.max(0, Math.floor((area.h - totalH) / 2));
      this.tile = tile;
      this.tileGap = tgap;
      this.boardW = bw;
      this.boardH = bh;
      this.boardInner = inner;
      this.boardPos = [];
      for (let i = 0; i < n; i++) {
        const cx = i % bc, cy = Math.floor(i / bc);
        this.boardPos.push([x0 + inner + cx * (bw + 2 * inner + bgap), y0 + inner + cy * (bh + 2 * inner + bgap)]);
      }
      this.gridTop = y0;
      this.gridBottom = y0 + totalH;
      this.msgY = n === 1 ? Math.max(this.hudH + Math.floor(lane / 2) + 2, y0 - Math.floor(lane / 2)) : y0 + Math.max(12, Math.floor(20 * k));
      this.boardCache.clear();
    }

    buildKeyboard() {
      const W = this.width, H = this.height;
      const k = this.k();
      const layout = KEYBOARDS[LANG_KEYBOARD[this.lang] || "qwerty"];
      this.keyH = Math.max(22, Math.min(Math.floor(H * 0.078), 64));
      this.keyGap = Math.max(2, Math.floor(5 * k));
      const pad = Math.max(6, Math.floor(12 * k));
      let kw = Math.floor(this.keyH * 1.05);
      layout.forEach((row, ri) => {
        const units = row.length + (ri === 2 ? 3.0 : 0.0);
        const items = row.length + (ri === 2 ? 2 : 0);
        kw = Math.min(kw, Math.floor((W - 2 * pad - (items - 1) * this.keyGap) / units));
      });
      kw = Math.max(12, kw);
      const special = Math.floor(kw * 1.5);
      const kbH = 3 * this.keyH + 2 * this.keyGap;
      this.kbTop = H - kbH - Math.max(4, Math.floor(8 * k));
      this.keyRects = {};
      this.enterRect = this.delRect = null;
      let y = this.kbTop;
      layout.forEach((row, ri) => {
        let rowW = row.length * kw + (row.length - 1) * this.keyGap;
        if (ri === 2) rowW += 2 * (special + this.keyGap);
        let x = Math.floor((W - rowW) / 2);
        if (ri === 2) {
          this.enterRect = new PG.Rect(x, y, special, this.keyH);
          x += special + this.keyGap;
        }
        for (const ch of row) {
          this.keyRects[ch] = new PG.Rect(x, y, kw, this.keyH);
          x += kw + this.keyGap;
        }
        if (ri === 2) this.delRect = new PG.Rect(x, y, special, this.keyH);
        y += this.keyH + this.keyGap;
      });
      this.keyFont = ui.font(Math.max(10, Math.min(Math.floor(this.keyH * 0.42), Math.floor(kw * 0.62))), true);
    }

    // ===================================================== Partie starten
    /** Lädt die Wortliste der Sprache/Länge nach (falls nötig) und ruft then(). */
    withWords(then) {
      const lang = this.lang, length = this.length;
      if (PG.wordleWords.isLoaded(lang, length)) return then();
      this.loading = true;
      PG.wordleWords.load(lang, length, () => {
        this.loading = false;
        if (!this.dead) then();
      });
    }

    /** Neue Partie im gewählten Modus (nach dem Setup oder "Nochmal"). */
    start() {
      if (this.loading) return;
      this.lang = PG.lang;
      const opts = this.cleanOpts();
      this.length = opts.length;
      this.hard = opts.hard;
      this.colorblind = opts.colorblind;
      const begin = () => {
        if (!PG.wordleWords.hasLength(this.lang, this.length) && this.length !== DEFAULT_LENGTH) {
          this.length = DEFAULT_LENGTH; // Liste fehlt -> klassisch
          return this.withWords(begin);
        }
        this.words = PG.wordleWords.wordsFor(this.lang, this.length);
        this.allowed = PG.wordleWords.allowedFor(this.lang, this.length);
        this.data = loadData();
        this.gameOver = false;
        this.result = null;
        this.panelHidden = false;
        this.dailyView = false;
        this.points = 0;
        this.score = 0;
        this.solvedCount = 0;
        this.layoutPlay();
        if (this.gameMode === "daily") this.startDaily();
        else this.newRound();
        this.playSound("click");
      };
      this.withWords(begin);
    }

    /** Neue Wörter ziehen (Endlos: nächstes Wort, Dordle/Quordle: neue Runde). */
    newRound() {
      const pool = this.words;
      const n = this.nBoards;
      const answers = pool.length >= n ? PG.rand.sample(pool, n) : Array.from({ length: n }, () => PG.rand.choice(pool));
      this.boards = answers.map((a) => new Board(a));
      this.clearRound();
      this.state = PLAY;
    }

    clearRound() {
      this.guesses = [];
      this.current = "";
      this.reveal = null;
      this.message = "";
      this.messageT = 0;
      this.shake = 0;
      this.pops = {};
      this.boardCache.clear();
    }

    dailySlot() {
      return this.lang + ":" + this.length;
    }

    /** Tageswort laden - samt bisherigem Fortschritt von heute. */
    startDaily() {
      const today = PG.seedrand.todayStr();
      this.dailyDate = today;
      const answer = dailyAnswer(this.words, this.lang, this.length, today);
      this.boards = [new Board(answer)];
      this.clearRound();
      this.state = PLAY;
      const prog = this.data.daily[this.dailySlot()];
      if (!(prog && typeof prog === "object" && prog.date === today && prog.answer === answer)) return;
      const board = this.boards[0];
      const saved = Array.isArray(prog.guesses) ? prog.guesses.slice(0, this.rows) : [];
      for (let guess of saved) {
        if (!(typeof guess === "string" && guess.length === this.length && /^[A-Za-z]+$/.test(guess))) break;
        guess = guess.toUpperCase();
        this.guesses.push(guess);
        board.apply(guess, evaluate(guess, answer), -99);
        if (board.solved) break;
      }
      if (board.solved || this.guesses.length >= this.rows) {
        // Heute schon fertig: nur Ergebnis, Statistik und Countdown.
        const st = getStats(this.data, "daily", this.lang, this.length);
        this.dailyView = true;
        this.result = { won: board.solved, tries: board.solvedAt || 0, stats: st, t0: this.anim - PANEL_DELAY, newBest: false };
        this.state = DONE;
      }
    }

    saveDaily(done) {
      this.data.daily[this.dailySlot()] = { date: this.dailyDate, answer: this.boards[0].answer, guesses: this.guesses.slice(), done: !!done };
      // Alte Tage aufräumen (nur heutige Stände behalten).
      for (const slot of Object.keys(this.data.daily)) {
        const p = this.data.daily[slot];
        if (!p || typeof p !== "object" || p.date !== this.dailyDate) delete this.data.daily[slot];
      }
    }

    // ===================================================== Eingabe
    handleEvent(ev) {
      if (this.loading) return;
      if (this.state === SETUP) {
        this.handleSetup(ev);
        return;
      }
      if (ev.kind === "mousedown" && [PLAY, REVEAL, SOLVED].includes(this.state) && this.gearRect.collidepoint(ev.pos)) {
        this.toSetup();
        return;
      }
      if (this.state === DONE) {
        this.handleDone(ev);
        return;
      }
      if (this.state === SOLVED) {
        if (this.isContinue(ev)) {
          this.newRound();
          this.playSound("click");
        } else if (ev.kind === "keydown" && (ev.key === "c" || ev.key === "C")) {
          this.share();
        }
        return;
      }
      if (this.state === REVEAL) return;
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "BackSpace") this.backspace();
        else if (k === "Return" || k === "KP_Enter") this.submit();
        else {
          const ch = letterOf(ev);
          if (ch) this.type(ch);
        }
      } else if (ev.kind === "mousedown") {
        this.clickKeyboard(ev.pos);
      }
    }

    isContinue(ev) {
      return ev.kind === "mousedown" || (ev.kind === "keydown" && ["Return", "KP_Enter", "space"].includes(ev.key));
    }

    type(ch) {
      if (this.current.length < this.length) {
        this.pops[this.current.length] = this.anim;
        this.current += ch;
        this.playSound("click");
      }
    }

    backspace() {
      if (this.current) {
        this.current = this.current.slice(0, -1);
        delete this.pops[this.current.length];
        this.playSound("move");
      }
    }

    clickKeyboard(pos) {
      if (this.enterRect && this.enterRect.collidepoint(pos)) return this.submit();
      if (this.delRect && this.delRect.collidepoint(pos)) return this.backspace();
      for (const ch of Object.keys(this.keyRects)) {
        if (this.keyRects[ch].collidepoint(pos)) return this.type(ch);
      }
    }

    submit() {
      if (this.current.length !== this.length) return this.reject(t("wd.too_short", { n: this.length }));
      if (!this.allowed.has(this.current)) return this.reject(t("wd.not_a_word"));
      if (this.hard) {
        const problem = this.hardCheck(this.current);
        if (problem) return this.reject(problem);
      }
      const results = this.boards.map((b) => (b.solved ? null : evaluate(this.current, b.answer)));
      const step = this.revealStep();
      this.reveal = { guess: this.current, results, t0: this.anim, end: this.anim + (this.length - 1) * step + FLIP_TIME + 0.06, ticks: 0 };
      this.state = REVEAL;
      this.playSound("select");
    }

    revealStep() {
      return this.length <= 5 ? 0.13 : 0.105;
    }

    /**
     * Meldung, wenn der Versuch im harten Modus Hinweise ignoriert. Mehrere
     * Bretter: der Versuch muss zu den Hinweisen MINDESTENS EINES noch offenen
     * Bretts passen (alle zugleich wäre meist unmöglich).
     */
    hardCheck(guess) {
      const problems = this.boards.filter((b) => !b.solved).map((b) => hardProblem(guess, b.rows));
      if (!problems.length || problems.some((p) => p === null)) return "";
      if (this.boards.length > 1) return t("wd.hard_multi");
      return t(problems[0][0], problems[0][1]);
    }

    /** Rateversuch abgelehnt: Meldung zeigen und die Zeile wackeln lassen. */
    reject(text) {
      this.say(text);
      this.shake = SHAKE_TIME;
      this.playSound("hit");
      this.rumble(60);
    }

    say(text, seconds = MESSAGE_TIME) {
      this.message = text;
      this.messageT = seconds;
    }

    finishReveal() {
      const rv = this.reveal;
      const guess = rv.guess;
      this.guesses.push(guess);
      const solvedNow = [];
      this.boards.forEach((board, i) => {
        const result = rv.results[i];
        if (result === null) return;
        board.apply(guess, result, this.anim);
        if (board.solved) solvedNow.push(board);
      });
      this.reveal = null;
      this.current = "";
      this.pops = {};
      this.boardCache.clear();
      const n = this.guesses.length;
      const allSolved = this.boards.every((b) => b.solved);
      if (this.gameMode === "daily") {
        this.saveDaily(allSolved || n >= this.rows);
        saveData(this.data);
      }
      for (const board of solvedNow) {
        if (this.boards.length > 1 && !allSolved) {
          this.playSound("point");
          const [bx, by] = this.boardPos[this.boards.indexOf(board)];
          ui.spawnBurst(bx + this.boardW / 2, by + this.boardH / 2, this.palette().correct);
        }
      }
      if (allSolved) this.roundWon(n);
      else if (n >= this.rows) this.roundLost(n);
      else this.state = PLAY;
    }

    roundWon(n) {
      const mode = this.gameMode;
      if (mode === "endless") {
        const pts = 10 + (this.rows - n) * 10; // weniger Versuche = mehr
        this.lastPoints = pts;
        this.points += pts;
        if (this.countsScore) this.score = this.points;
        this.solvedCount += 1;
        recordGame(this.data, mode, this.lang, this.length, true, n);
        saveData(this.data);
        this.reportResult(true);
        if (n <= 2) this.achEvent("wordle_two");
        this.solvedT = this.anim;
        this.state = SOLVED;
        this.playSound("win");
        return;
      }
      const day = mode === "daily" ? PG.seedrand.dayIndex(this.dailyDate) : null;
      const st = recordGame(this.data, mode, this.lang, this.length, true, n, day);
      saveData(this.data);
      this.reportResult(true);
      if (mode === "daily") {
        if (n <= 2) this.achEvent("wordle_two");
        this.achEvent("wordle_daily7", st.streak);
      } else if (mode === "quordle") {
        this.achEvent("wordle_quordle");
      }
      this.finishRound(true, n, st);
      ui.spawnConfetti(this.width, this.height, 70);
      this.playSound("win");
    }

    roundLost(n) {
      const mode = this.gameMode;
      const day = mode === "daily" ? PG.seedrand.dayIndex(this.dailyDate) : null;
      const st = recordGame(this.data, mode, this.lang, this.length, false, n, day);
      let newBest = false;
      if (mode === "endless") {
        const key = String(this.length);
        const best = Number.isInteger(this.data.best[key]) ? this.data.best[key] : 0;
        if (this.points > best) {
          this.data.best[key] = this.points;
          newBest = best > 0 || this.points > 0;
        }
        if (this.solvedCount === 0) this.reportResult(false);
      } else {
        this.reportResult(false);
      }
      saveData(this.data);
      this.finishRound(false, n, st, newBest);
      this.playSound("gameover");
      this.rumble(160);
    }

    finishRound(won, n, st, newBest = false) {
      this.result = { won, tries: n, stats: st, t0: this.anim, newBest };
      this.panelHidden = false;
      this.state = DONE;
      // Die App sichert jetzt den Highscore (Endlos/5) bzw. zählt die Partie.
      this.gameOver = true;
    }

    /** Zurück ins Setup - eine laufende Endlos-Serie wird vorher gesichert. */
    toSetup() {
      if ([PLAY, REVEAL, SOLVED].includes(this.state) && this.gameMode === "endless" && this.points > 0) this.bankRun();
      this.gameOver = false;
      this.state = SETUP;
      this.setupT = this.anim;
      this.score = 0;
      this.points = 0;
      this.playSound("click");
    }

    /** Abgebrochene Endlos-Serie: Bestwert und ggf. Highscore sichern. */
    bankRun() {
      const key = String(this.length);
      const best = this.data.best[key];
      if (!Number.isInteger(best) || this.points > best) {
        this.data.best[key] = this.points;
        saveData(this.data);
      }
      if (this.countsScore && this.score > 0) {
        const [, record] = PG.highscore.update(this.highscoreKey, this.score);
        if (record) PG.stats.recordBroken(this.highscoreKey);
      }
    }

    handleDone(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Return" || k === "KP_Enter") this.donePrimary();
        else if (k === "c" || k === "C") this.share();
        else if (k === "s" || k === "S" || k === "BackSpace") this.toSetup();
        else if (k === "space") {
          this.panelHidden = !this.panelHidden;
          this.playSound("move");
        }
      } else if (ev.kind === "mousedown") {
        if (this.panelHidden) {
          this.panelHidden = false;
          return;
        }
        for (const [rc, action] of this.doneButtons) {
          if (rc.collidepoint(ev.pos)) return action();
        }
        if (this.donePanel && !this.donePanel.collidepoint(ev.pos)) this.panelHidden = true;
      }
    }

    donePrimary() {
      if (this.gameMode === "daily") {
        if (this.dailyDate !== PG.seedrand.todayStr()) {
          this.gameOver = false;
          this.newRoundResult();
          this.start(); // neuer Tag, neues Wort
        } else {
          this.toSetup();
        }
        return;
      }
      this.gameOver = false;
      this.newRoundResult();
      this.start();
    }

    share() {
      const boards = this.boards.map((b) => [b.rows.map((r) => r[1]), b.solvedAt]);
      const text = shareText(this.gameMode, this.lang, this.length, boards, this.hard, this.colorblind, this.dailyDate);
      copyToClipboard(text, (ok) => {
        if (this.dead) return;
        if (ok) {
          this.say(t("wd.copied"));
          this.playSound("powerup");
        } else {
          this.say(t("wd.copy_failed"));
          this.playSound("hit");
        }
      });
    }

    // ----- Setup-Eingabe -------------------------------------------------
    handleSetup(ev) {
      if (ev.kind === "keydown") {
        const k = ev.key;
        if (k === "Return" || k === "KP_Enter") this.start();
        else if (["4", "5", "6", "7"].includes(k)) this.setLength(Number(k));
        else if (k === "Left" || k === "Right" || this.isAction(k, "left") || this.isAction(k, "right")) {
          const step = k === "Left" || this.isAction(k, "left") ? -1 : 1;
          const idx = LENGTHS.includes(this.length) ? LENGTHS.indexOf(this.length) : 1;
          this.setLength(LENGTHS[Math.max(0, Math.min(LENGTHS.length - 1, idx + step))]);
          this.setupFocus = 0;
        } else if (k === "h" || k === "H") this.toggle("hard");
        else if (k === "f" || k === "F") this.toggle("colorblind");
        else if (k === "Up" || this.isAction(k, "up")) {
          this.setupFocus = PG.mod(this.setupFocus - 1, 4);
          this.playSound("move");
        } else if (k === "Down" || this.isAction(k, "down")) {
          this.setupFocus = PG.mod(this.setupFocus + 1, 4);
          this.playSound("move");
        } else if (k === "space") {
          if (this.setupFocus === 1) this.toggle("hard");
          else if (this.setupFocus === 2) this.toggle("colorblind");
          else if (this.setupFocus === 3) this.start();
        }
      } else if (ev.kind === "mousedown") {
        for (let i = 0; i < this.suSeg.length; i++) {
          if (this.suSeg[i].collidepoint(ev.pos)) {
            this.setupFocus = 0;
            this.setLength(LENGTHS[i]);
            return;
          }
        }
        if (this.suHard.collidepoint(ev.pos)) {
          this.setupFocus = 1;
          this.toggle("hard");
        } else if (this.suCb.collidepoint(ev.pos)) {
          this.setupFocus = 2;
          this.toggle("colorblind");
        } else if (this.suStart.collidepoint(ev.pos)) {
          this.setupFocus = 3;
          this.start();
        }
      } else if (ev.kind === "mousemove") {
        [this.suHard, this.suCb, this.suStart].forEach((rc, i) => {
          if (rc.collidepoint(ev.pos)) this.setupFocus = i + 1;
        });
        if (this.suSeg.some((rc) => rc.collidepoint(ev.pos))) this.setupFocus = 0;
      }
    }

    setLength(length) {
      if (length === this.length || !LENGTHS.includes(length)) return;
      this.length = length;
      this.saveOpt("length", length);
      this.layoutPlay();
      this.playSound("click");
    }

    toggle(key) {
      const value = !this[key];
      this[key] = value;
      this.saveOpt(key, value);
      this.boardCache.clear();
      this.playSound("click");
    }

    // ===================================================== Update
    update(dt) {
      this.updated = true;
      this.tick(dt);
    }

    tick(dt) {
      this.anim += dt;
      if (this.messageT > 0) {
        this.messageT = Math.max(0, this.messageT - dt);
        if (this.messageT === 0) this.message = "";
      }
      if (this.shake > 0) this.shake = Math.max(0, this.shake - dt);
      const rv = this.reveal;
      if (this.state === REVEAL && rv) {
        // Ein leises Klacken je umgedrehter Spalte.
        const col = Math.floor((this.anim - rv.t0) / this.revealStep());
        if (col >= rv.ticks && rv.ticks < this.length) {
          rv.ticks = col + 1;
          this.playSound("click");
        }
        if (this.anim >= rv.end) this.finishReveal();
      }
    }

    // ===================================================== Zeichnen
    draw(ctx) {
      // Nach Game Over ruft die App update() nicht mehr auf - die Uhr läuft dann
      // hier weiter, damit Ergebnis-Panel und Balken animiert bleiben.
      const now = performance.now() / 1000;
      if (!this.updated && this.lastDraw !== null && this.gameOver) this.tick(Math.min(0.1, now - this.lastDraw));
      this.updated = false;
      this.lastDraw = now;
      if (this.state === SETUP) {
        this.drawSetup(ctx);
        if (this.loading) this.drawLoading(ctx);
        return;
      }
      ui.drawBackground(ctx, this.width, this.height, false, true);
      this.drawHud(ctx);
      this.boards.forEach((board, i) => this.drawBoard(ctx, i, board));
      this.drawKeyboard(ctx);
      if (this.state === SOLVED) this.drawSolved(ctx);
      if (this.state === DONE) {
        if (this.panelHidden) this.drawHiddenHint(ctx);
        else this.drawResult(ctx);
      }
      if (this.message) this.drawMessage(ctx);
      if (this.loading) this.drawLoading(ctx);
    }

    // ----- HUD -----------------------------------------------------------
    drawHud(ctx) {
      const W = this.width, h = this.hudH;
      draw.rect(ctx, ui.PANEL, [0, 0, W, h]);
      draw.line(ctx, ui.BORDER, [0, h], [W, h]);
      const cy = Math.floor(h / 2);
      const third = Math.floor((W - this.hudH) / 3) - 10;
      const mode = this.gameMode;
      let left, mid;
      if (mode === "endless") {
        left = t("wd.score", { n: this.points });
        mid = t("wd.solved_n", { n: this.solvedCount });
      } else if (mode === "daily") {
        left = t("wd.hud.daily", { d: this.shortDate() });
        const st = getStats(this.data, "daily", this.lang, this.length);
        mid = t("wd.hud.streak", { n: shownStreak(st, "daily") });
      } else {
        left = t("wd.mode." + mode);
        mid = t("wd.boards_solved", { a: this.boards.filter((b) => b.solved).length, b: this.boards.length });
      }
      const px = this.fHud.height;
      this.txt(ctx, left, Math.floor(px * 0.95), this.accent, third, true, "midleft", 12, cy);
      this.txt(ctx, mid, Math.floor(px * 0.8), ui.TEXT_DIM, third, false, "center", Math.floor((W - this.hudH) / 2) + 6, cy);
      const tries = Math.min(this.guesses.length + ([PLAY, REVEAL].includes(this.state) ? 1 : 0), this.rows);
      this.txt(ctx, t("wd.tries", { a: tries, b: this.rows }), Math.floor(px * 0.8), ui.TEXT_DIM, third, false, "midright", this.gearRect.x - 8, cy);
      this.drawGear(ctx, this.gearRect);
    }

    shortDate() {
      const d = this.dailyDate || PG.seedrand.todayStr();
      return d.slice(8, 10) + "." + d.slice(5, 7) + ".";
    }

    /** Regler-Symbol (drei Schieber) = zurück ins Setup. */
    drawGear(ctx, rc) {
      const col = ui.TEXT_DIM;
      draw.rect(ctx, ui.BTN, rc, 0, Math.max(4, Math.floor(rc.h / 4)));
      const x0 = rc.x + rc.w * 0.24, x1 = rc.right - rc.w * 0.24;
      const th = Math.max(1, Math.floor(rc.h / 14));
      [0.3, 0.5, 0.7].forEach((f, i) => {
        const y = Math.floor(rc.y + rc.h * f);
        draw.line(ctx, col, [Math.floor(x0), y], [Math.floor(x1), y], th);
        const kx = Math.floor(x0 + (x1 - x0) * [0.7, 0.3, 0.55][i]);
        draw.circle(ctx, col, [kx, y], Math.max(2, Math.floor(rc.h / 10)));
      });
    }

    // ----- Kacheln -------------------------------------------------------
    stateColor(state) {
      if (state === "absent") return COL_ABSENT;
      return this.palette()[state];
    }

    emptyFill() {
      return ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.55);
    }

    drawTile(ctx, x, y, ch, state, o = {}) {
      const size = this.tile;
      const scaleY = o.scaleY != null ? o.scaleY : 1;
      const scale = o.scale != null ? o.scale : 1;
      const fill = state ? this.stateColor(state) : o.fill || this.emptyFill();
      const w = Math.max(1, Math.floor(size * scale));
      const h = Math.max(1, Math.floor(size * scale * scaleY));
      const cx = x + Math.floor(size / 2), cy = y + Math.floor(size / 2) + (o.dy || 0);
      const rc = new PG.Rect(Math.round(cx - w / 2), Math.round(cy - h / 2), w, h);
      const radius = Math.max(2, Math.floor(size / 9));
      draw.rect(ctx, fill, rc, 0, radius);
      if (!state) draw.rect(ctx, o.border || ui.BORDER, rc, Math.max(1, Math.floor(size / 24)), radius);
      if (ch && h > 3) {
        const f = ui.font(Math.max(7, Math.floor(size * 0.58)), true);
        if (scaleY < 0.99 || Math.abs(scale - 1) > 0.01) {
          ctx.save();
          ctx.translate(cx, cy);
          ctx.scale(scale, scale * scaleY);
          ui.text(ctx, ch, 0, 0, f, COL_LETTER, "center");
          ctx.restore();
        } else {
          ui.text(ctx, ch, cx, cy, f, COL_LETTER, "center");
        }
      }
    }

    tileXY(bi, r, c) {
      const [bx, by] = this.boardPos[bi];
      const step = this.tile + this.tileGap;
      return [bx + c * step, by + r * step];
    }

    bouncing(board) {
      return board.solved && board.solveT !== null && this.anim - board.solveT < BOUNCE_TIME + this.length * 0.08;
    }

    /** Fertige Zeilen und leere Kacheln eines Bretts als gecachte Fläche. */
    boardStatic(bi, board) {
      const bouncing = this.bouncing(board);
      const scale = (PG.app && PG.app.pixelScale) || 1;
      const key = [bi, this.tile, this.tileGap, this.length, this.rows, board.rows.length, board.solvedAt, bouncing, this.colorblind, ui.PANEL.join(","), ui.BORDER.join(","), ui.BG_BOTTOM.join(","), scale].join("|");
      const hit = this.boardCache.get(bi);
      if (hit && hit.key === key) return hit.cv;
      const cv = ui.makeCanvas(this.boardW * scale, this.boardH * scale);
      const c = cv.getContext("2d");
      c.scale(scale, scale);
      const live = board.solved ? null : board.rows.length;
      const step = this.tile + this.tileGap;
      const dimFill = ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.3);
      for (let r = 0; r < this.rows; r++) {
        if (r === live) continue;
        for (let col = 0; col < this.length; col++) {
          const x = col * step, y = r * step;
          if (r < board.rows.length) {
            if (bouncing && r === board.solvedAt - 1) continue;
            const [word, res] = board.rows[r];
            this.drawTile(c, x, y, word[col], res[col]);
          } else if (board.solved) {
            this.drawTile(c, x, y, "", null, { border: ui.mix(dimFill, ui.BORDER, 0.4), fill: dimFill });
          } else {
            this.drawTile(c, x, y, "", null);
          }
        }
      }
      this.boardCache.set(bi, { key, cv });
      return cv;
    }

    drawBoard(ctx, bi, board) {
      const [bx, by] = this.boardPos[bi];
      if (this.boards.length > 1) {
        const pad = this.boardInner;
        const frame = new PG.Rect(bx - pad, by - pad, this.boardW + 2 * pad, this.boardH + 2 * pad);
        let border = null;
        if (board.solved) border = this.palette().correct;
        else if (this.state === DONE) border = COL_LOSE;
        const radius = Math.max(6, pad * 2);
        ui.drawPanel(ctx, frame, { radius, shadow: false, border: border || undefined });
        if (border) draw.rect(ctx, border, frame, Math.max(1, Math.floor(pad / 3)), radius);
      }
      ctx.drawImage(this.boardStatic(bi, board), bx, by, this.boardW, this.boardH);
      // Hüpfende Siegerzeile
      if (this.bouncing(board)) {
        const r = board.solvedAt - 1;
        const [word, res] = board.rows[r];
        for (let c = 0; c < this.length; c++) {
          const p = (this.anim - board.solveT - c * 0.08) / BOUNCE_TIME;
          const dy = p > 0 && p < 1 ? -Math.floor(Math.sin(Math.PI * p) * this.tile * 0.35) : 0;
          const [x, y] = this.tileXY(bi, r, c);
          this.drawTile(ctx, x, y, word[c], res[c], { dy });
        }
      }
      if (board.solved) return;
      const r = board.rows.length;
      if (r >= this.rows) {
        if (this.state === DONE) this.drawAnswerPill(ctx, bi, board);
        return;
      }
      const rv = this.reveal;
      if (rv && rv.results[bi] !== null) {
        // Kachel für Kachel umdrehen: erst zuklappen, dann mit Farbe auf.
        const stepT = this.revealStep();
        for (let c = 0; c < this.length; c++) {
          const [x, y] = this.tileXY(bi, r, c);
          const p = (this.anim - rv.t0 - c * stepT) / FLIP_TIME;
          if (p <= 0) this.drawTile(ctx, x, y, rv.guess[c], null, { border: ui.TEXT_DIM });
          else if (p < 0.5) this.drawTile(ctx, x, y, rv.guess[c], null, { border: ui.TEXT_DIM, scaleY: Math.max(0.02, Math.cos(p * Math.PI)) });
          else {
            const sy = p < 1 ? Math.max(0.02, -Math.cos(p * Math.PI)) : 1;
            this.drawTile(ctx, x, y, rv.guess[c], rv.results[bi][c], { scaleY: sy });
          }
        }
        return;
      }
      if (this.state === DONE) {
        // Nicht gelöst: die Lösung in der nächsten Zeile zeigen.
        for (let c = 0; c < this.length; c++) {
          const [x, y] = this.tileXY(bi, r, c);
          this.drawTile(ctx, x, y, board.answer[c], null, { border: COL_LOSE, fill: ui.mix(this.emptyFill(), COL_LOSE, 0.25) });
        }
        return;
      }
      const shake = this.shake > 0 ? Math.floor(this.tile * 0.14 * Math.sin(this.shake * 48)) : 0;
      for (let c = 0; c < this.length; c++) {
        const [x, y] = this.tileXY(bi, r, c);
        const ch = c < this.current.length ? this.current[c] : "";
        let scale = 1;
        if (ch && this.pops[c] != null) {
          const p = (this.anim - this.pops[c]) / POP_TIME;
          if (p >= 0 && p < 1) scale = 1 + 0.12 * Math.sin(Math.PI * p);
        }
        this.drawTile(ctx, x + shake, y, ch, null, { border: ch ? ui.TEXT_DIM : null, scale });
      }
    }

    /** Nicht gelöstes Brett ohne freie Zeile: Lösung als Etikett unten. */
    drawAnswerPill(ctx, bi, board) {
      const [bx, by] = this.boardPos[bi];
      const px = Math.max(10, Math.min(Math.floor(this.tile * 0.62), this.fSmallB.height));
      const f = this.fit(board.answer, this.boardW - 8, px, true);
      const w = f.width(board.answer), h = f.height;
      const rc = new PG.Rect(0, 0, w, h);
      rc.center = [bx + Math.floor(this.boardW / 2), by + this.boardH];
      const box = rc.inflate(Math.max(12, px), Math.max(4, Math.floor(px / 3)));
      draw.rect(ctx, COL_LOSE, box, 0, Math.floor(box.h / 2));
      ui.text(ctx, board.answer, rc.centerx, rc.centery, f, COL_LETTER, "center");
    }

    // ----- Tastatur ------------------------------------------------------
    drawKeyboard(ctx) {
      const radius = Math.max(3, Math.floor(this.keyH / 8));
      const n = this.boards.length;
      for (const ch of Object.keys(this.keyRects)) {
        const rc = this.keyRects[ch];
        draw.rect(ctx, ui.BTN, rc, 0, radius);
        const states = this.boards.map((b) => b.keys[ch]);
        if (n === 1) {
          if (states[0]) draw.rect(ctx, this.stateColor(states[0]), rc, 0, radius);
        } else if (n === 2) {
          const half = Math.floor(rc.w / 2);
          states.forEach((st, i) => {
            if (!st) return;
            const part = [rc.x + i * half, rc.y, i === 0 ? half : rc.w - half, rc.h];
            draw.rect(ctx, this.stateColor(st), part, 0, i === 0 ? [radius, 0, 0, radius] : [0, radius, radius, 0]);
          });
        } else {
          const hw = Math.floor(rc.w / 2), hh = Math.floor(rc.h / 2);
          states.slice(0, 4).forEach((st, i) => {
            if (!st) return;
            const cx = i % 2, cy = Math.floor(i / 2);
            const part = [rc.x + cx * hw, rc.y + cy * hh, cx === 0 ? hw : rc.w - hw, cy === 0 ? hh : rc.h - hh];
            const rad = [i === 0 ? radius : 0, i === 1 ? radius : 0, i === 3 ? radius : 0, i === 2 ? radius : 0];
            draw.rect(ctx, this.stateColor(st), part, 0, rad);
          });
        }
        if (n === 1 && !states[0]) draw.rect(ctx, ui.BORDER, rc, 1, radius);
        ui.text(ctx, ch, rc.centerx + 1, rc.centery + 1, this.keyFont, [0, 0, 0, 90], "center");
        ui.text(ctx, ch, rc.centerx, rc.centery, this.keyFont, COL_LETTER, "center");
      }
      for (const [rc, kind] of [[this.enterRect, "enter"], [this.delRect, "del"]]) {
        if (!rc) continue;
        draw.rect(ctx, ui.BTN_SEL, rc, 0, radius);
        if (kind === "enter") this.txt(ctx, t("wd.enter"), Math.max(9, Math.floor((this.keyH * 3) / 8)), ui.TEXT, rc.w - 6, true, "center", rc.centerx, rc.centery);
        else this.drawBackspace(ctx, rc);
      }
    }

    /** Rücktaste als gezeichnetes Symbol (Pfeil-Etikett mit x). */
    drawBackspace(ctx, rc) {
      const h = Math.max(8, Math.floor(rc.h * 0.36));
      const w = Math.floor(h * 1.5);
      const x0 = rc.centerx - Math.floor(w / 2);
      const y0 = rc.centery - Math.floor(h / 2);
      const pts = [[x0, rc.centery], [x0 + Math.floor(h / 2), y0], [x0 + w, y0], [x0 + w, y0 + h], [x0 + Math.floor(h / 2), y0 + h]];
      const th = Math.max(1, Math.floor(h / 8));
      draw.polygon(ctx, ui.TEXT, pts, th);
      const cx = x0 + Math.floor(h / 2) + Math.floor((w - Math.floor(h / 2)) / 2);
      const d = Math.max(2, Math.floor(h / 5));
      draw.line(ctx, ui.TEXT, [cx - d, rc.centery - d], [cx + d, rc.centery + d], th);
      draw.line(ctx, ui.TEXT, [cx - d, rc.centery + d], [cx + d, rc.centery - d], th);
    }

    // ----- Meldungen / Banner ---------------------------------------------
    /** Kurze Meldung ("Kein Wort in der Liste", "Kopiert!"). */
    drawMessage(ctx) {
      const alpha = Math.min(1, this.messageT / 0.35);
      this.drawPill(ctx, this.message, alpha);
    }

    drawLoading(ctx) {
      this.drawPill(ctx, t("web.wordle.loading"), 0.6 + 0.4 * ui.pulse(3, 0, 1));
    }

    drawPill(ctx, text, alpha) {
      const f = this.fit(text, this.width - 40, this.fSmallB.height, true);
      const rc = new PG.Rect(0, 0, f.width(text), f.height);
      rc.center = [Math.floor(this.width / 2), this.state === SETUP ? this.suStart.bottom + 24 : this.msgY];
      if (this.state === DONE && !this.panelHidden && this.donePanel) rc.center = [Math.floor(this.width / 2), Math.max(rc.h, this.donePanel.y - rc.h)];
      const box = rc.inflate(26, 12);
      draw.rect(ctx, [ui.PANEL_LIGHT[0], ui.PANEL_LIGHT[1], ui.PANEL_LIGHT[2], Math.floor(240 * alpha)], box, 0, Math.floor(box.h / 2));
      draw.rect(ctx, ui.mix(ui.BORDER_LIGHT, this.accent, 0.3), box, 1, Math.floor(box.h / 2));
      ui.text(ctx, text, rc.centerx, rc.centery, f, ui.TEXT, "center", alpha);
    }

    /** Endlos: Wort gelöst - kleiner Banner mit Punkten über dem Brett. */
    drawSolved(ctx) {
      const p = (this.anim - this.solvedT - 0.35) / 0.25;
      if (p <= 0) return;
      const alpha = Math.min(1, p);
      const k = this.k();
      const w = Math.min(this.width - 24, Math.floor(440 * k));
      const h = Math.max(56, Math.floor(88 * k));
      // Nicht über die gelöste Zeile legen: steht sie oben, kommt der Banner
      // nach unten ins Brett und umgekehrt.
      const board = this.boards[0];
      const rowY = this.tileXY(0, (board.solvedAt || 1) - 1, 0)[1];
      const y = rowY + Math.floor(this.tile / 2) < this.gridTop + Math.floor(this.boardH / 2) ? this.gridBottom - h : this.gridTop;
      const rc = new PG.Rect(Math.floor((this.width - w) / 2), y - Math.floor((1 - alpha) * 12), w, h);
      const col = this.palette().correct;
      ui.drawPanel(ctx, rc, { accentTop: col });
      this.txt(ctx, t("wd.solved", { n: this.lastPoints }), this.fBig.height, col, w - 20, true, "center", rc.centerx, rc.y + h * 0.38);
      this.txt(ctx, t("wd.next") + "   ·   " + t("wd.share_key"), this.fTiny.height, ui.TEXT_DIM, w - 20, false, "center", rc.centerx, rc.y + h * 0.76);
    }

    drawHiddenHint(ctx) {
      const text = t("wd.result.show");
      const f = this.fit(text, this.width - 30, this.fSmall.height);
      const rc = new PG.Rect(0, 0, f.width(text), f.height);
      rc.midbottom = [Math.floor(this.width / 2), this.kbTop - 4];
      const box = rc.inflate(22, 10);
      ui.drawPanel(ctx, box, { radius: Math.floor(box.h / 2), shadow: false });
      ui.text(ctx, text, rc.centerx, rc.centery, f, ui.TEXT, "center");
    }

    // ----- Statistik-Block (Setup + Ergebnis) ----------------------------
    /** Vier Kennzahlen + Balkendiagramm der Versuchsverteilung in 'rect'. */
    drawStatsBlock(ctx, rect, st, highlight = null, grow = 1) {
      const k = this.k();
      const played = st.played;
      const rate = played ? Math.round((100 * st.won) / played) : 0;
      const values = [[played, "wd.stats.played"], [rate, "wd.stats.winrate"], [shownStreak(st, this.gameMode), "wd.stats.streak"], [st.best, "wd.stats.best"]];
      const [numPx, labPx, perRow] = this.statsMetrics(rect.w, rect.h);
      const colW = Math.floor(rect.w / perRow);
      let y = rect.y;
      values.forEach(([v, key], i) => {
        const cx = rect.x + colW * (i % perRow) + Math.floor(colW / 2);
        const cy = y + Math.floor(i / perRow) * (numPx + labPx + 6);
        this.txt(ctx, String(v), numPx, ui.TEXT, colW - 4, true, "midtop", cx, cy);
        this.txt(ctx, t(key), labPx, ui.TEXT_DIM, colW - 4, false, "midtop", cx, cy + numPx + 2);
      });
      y += Math.floor(4 / perRow) * (numPx + labPx + 6) - 6 + Math.max(6, Math.floor(12 * k));
      this.txt(ctx, t("wd.stats.dist"), Math.max(9, Math.floor(12 * k)), ui.TEXT_DIM, rect.w, true, "midtop", rect.centerx, y);
      y += Math.max(9, Math.floor(12 * k)) + Math.max(4, Math.floor(8 * k));
      const first = this.nBoards - 1; // Quordle: frühestens nach 4
      const dist = st.dist.slice(first);
      const bars = dist.length;
      const avail = rect.bottom - y;
      const barGap = Math.max(2, Math.floor(4 * k));
      const barH = Math.max(6, Math.min(Math.floor(20 * k), Math.floor((avail - (bars - 1) * barGap) / Math.max(1, bars))));
      const labelW = Math.max(12, Math.floor(18 * k));
      const topV = Math.max(1, ...dist);
      const palette = this.palette();
      for (let i = 0; i < dist.length; i++) {
        const v = dist[i];
        const tries = first + i + 1;
        const by = y + i * (barH + barGap);
        if (by + barH > rect.bottom + 2) break;
        this.txt(ctx, String(tries), Math.max(9, barH - 2), ui.TEXT_DIM, labelW, true, "midleft", rect.x, by + Math.floor(barH / 2));
        const full = rect.w - labelW - 6;
        const minw = Math.max(18, Math.floor(24 * k));
        const bw = Math.floor(minw + (full - minw) * (v / topV) * grow);
        const bar = new PG.Rect(rect.x + labelW + 6, by, bw, barH);
        draw.rect(ctx, tries === highlight ? palette.correct : ui.BTN_SEL, bar, 0, Math.max(2, Math.floor(barH / 4)));
        this.txt(ctx, String(v), Math.max(9, barH - 3), COL_LETTER, bw - 6, true, "midright", bar.right - 5, bar.centery);
      }
    }

    /** [Zahl-Größe, Beschriftungs-Größe, Kennzahlen je Zeile]. */
    statsMetrics(width, height = 9999) {
      const k = this.k();
      const numPx = Math.max(14, Math.min(Math.floor(26 * k), Math.floor(height / 7)));
      const labPx = Math.max(9, Math.floor(11 * k));
      const colW = Math.floor(width / 4) - 4;
      const labels = ["wd.stats.played", "wd.stats.winrate", "wd.stats.streak", "wd.stats.best"];
      const wide = labels.every((key) => this.fits(t(key), colW, labPx));
      return [numPx, labPx, wide ? 4 : 2];
    }

    /** Natürliche Höhe des Statistik-Blocks (für die Panel-Größe). */
    statsHeight(width, bars) {
      const k = this.k();
      const [numPx, labPx, perRow] = this.statsMetrics(width);
      const head = Math.floor(4 / perRow) * (numPx + labPx + 6) - 6 + Math.max(6, Math.floor(12 * k));
      const dist = Math.max(9, Math.floor(12 * k)) + Math.max(4, Math.floor(8 * k));
      return head + dist + bars * Math.max(10, Math.floor(20 * k)) + (bars - 1) * Math.max(2, Math.floor(4 * k));
    }

    // ----- Ergebnis-Panel --------------------------------------------------
    drawResult(ctx) {
      const res = this.result;
      if (!res) return;
      const p = (this.anim - res.t0 - PANEL_DELAY) / 0.3;
      if (p <= 0) {
        this.doneButtons = [];
        return;
      }
      const appear = Math.min(1, p);
      const W = this.width, H = this.height;
      const k = this.k();
      draw.rect(ctx, [6, 8, 14, Math.floor(120 * appear)], [0, 0, W, H]);
      const pw = Math.min(W - 20, Math.floor(470 * k));
      const inner = pw - 2 * Math.max(12, Math.floor(22 * k));
      const mode = this.gameMode;
      const subPx = Math.max(11, Math.floor(15 * k));
      const btnH = Math.max(26, Math.floor(40 * k));
      const hintPx = Math.max(9, Math.floor(11 * k));
      const bigPx = Math.max(18, Math.floor(30 * k));
      let need = Math.max(8, Math.floor(16 * k)) + bigPx + Math.max(4, Math.floor(8 * k)) + subPx + Math.max(4, Math.floor(6 * k)) + Math.max(2, Math.floor(6 * k)) + this.statsHeight(inner, this.rows - this.nBoards + 1) + Math.max(4, Math.floor(8 * k)) + btnH + Math.max(4, Math.floor(8 * k)) + hintPx + Math.max(8, Math.floor(14 * k));
      if (mode === "endless") need += Math.max(10, Math.floor(13 * k)) + Math.max(4, Math.floor(6 * k));
      if (mode === "daily") need += subPx + Math.max(4, Math.floor(8 * k));
      // Endlos/5: die App blendet unten den Highscore-Banner ein - das Panel
      // bleibt darüber.
      const reserve = this.showHighscoreBanner ? BANNER_SPACE : 0;
      const ph = Math.min(H - 16 - reserve, need);
      const panel = new PG.Rect(0, 0, pw, ph);
      panel.center = [Math.floor(W / 2), Math.floor((H - reserve) / 2) + Math.floor((1 - appear) * 20)];
      this.donePanel = panel;
      const col = res.won ? this.palette().correct : COL_LOSE;
      ui.drawPanel(ctx, panel, { accentTop: col });
      let y = panel.y + Math.max(8, Math.floor(16 * k));
      let title = res.won ? t("wd.result.win") : t("wd.result.lose");
      if (mode === "endless") title = t("wd.gameover");
      this.txt(ctx, title, bigPx, col, inner, true, "midtop", panel.centerx, y);
      y += bigPx + Math.max(4, Math.floor(8 * k));
      let sub;
      if (mode === "endless") sub = t("wd.answer_was", { w: this.boards[0].answer }) + "  ·  " + t("wd.result.points", { n: this.points });
      else if (this.boards.length === 1) sub = res.won ? t("wd.result.in", { n: res.tries, m: this.rows }) : t("wd.answer_was", { w: this.boards[0].answer });
      else sub = t("wd.result.words", { w: this.boards.map((b) => b.answer).join(" · ") });
      this.txt(ctx, sub, subPx, ui.TEXT, inner, false, "midtop", panel.centerx, y);
      y += subPx + Math.max(4, Math.floor(6 * k));
      if (mode === "endless") {
        const best = this.data.best[String(this.length)];
        const line = res.newBest ? t("wd.new_best") : t("wd.result.best", { n: Number.isInteger(best) ? best : 0 });
        this.txt(ctx, line, Math.max(10, Math.floor(13 * k)), ui.GOLD, inner, false, "midtop", panel.centerx, y);
        y += Math.max(10, Math.floor(13 * k)) + Math.max(4, Math.floor(6 * k));
      }
      y += Math.max(2, Math.floor(6 * k));
      // Unten: Countdown, Knöpfe, Hinweis
      let bottom = panel.bottom - Math.max(8, Math.floor(14 * k)) - hintPx - Math.max(4, Math.floor(8 * k)) - btnH;
      if (mode === "daily") bottom -= subPx + Math.max(4, Math.floor(8 * k));
      let grow = Math.min(1, Math.max(0, (this.anim - res.t0 - PANEL_DELAY) / 0.7));
      grow = 1 - (1 - grow) ** 3;
      const highlight = res.won ? res.tries : null;
      const block = new PG.Rect(panel.centerx - Math.floor(inner / 2), y, inner, bottom - y - Math.max(4, Math.floor(8 * k)));
      this.drawStatsBlock(ctx, block, res.stats, highlight, grow);
      y = bottom;
      if (mode === "daily") {
        const text = this.dailyDate !== PG.seedrand.todayStr() ? t("wd.daily.new") : t("wd.daily.next", { t: formatHms(secondsToMidnight()) });
        this.txt(ctx, text, subPx, ui.GOLD, inner, true, "midtop", panel.centerx, y);
        y += subPx + Math.max(4, Math.floor(8 * k));
      }
      // Knöpfe
      const gap = Math.max(6, Math.floor(10 * k));
      const buttons = [[t("wd.share"), () => this.share()]];
      if (mode === "daily") buttons.push([t("wd.setup"), () => this.toSetup()]);
      else {
        buttons.push([t("wd.again"), () => this.donePrimary()]);
        buttons.push([t("wd.setup"), () => this.toSetup()]);
      }
      const bw = Math.floor((inner - (buttons.length - 1) * gap) / buttons.length);
      const longest = buttons.map((b) => b[0]).reduce((a, b) => (b.length > a.length ? b : a));
      const bfont = this.fit(longest, bw - 12, Math.max(11, Math.floor(16 * k)), true);
      this.doneButtons = [];
      buttons.forEach(([label, action], i) => {
        const rc = new PG.Rect(panel.centerx - Math.floor(inner / 2) + i * (bw + gap), y, bw, btnH);
        const selected = i === buttons.length - 2 || (buttons.length === 2 && i === 0);
        ui.drawButton(ctx, rc, label, bfont, selected, { accent: this.accent });
        this.doneButtons.push([rc, action]);
      });
      y += btnH + Math.max(4, Math.floor(8 * k));
      const hint = mode === "daily" ? t("wd.result.hint_daily") : t("wd.result.hint");
      this.txt(ctx, this.fitParts(hint, inner, hintPx), hintPx, ui.TEXT_FAINT, inner, false, "midtop", panel.centerx, y);
    }

    // ----- Setup -----------------------------------------------------------
    drawSetup(ctx) {
      const W = this.width, H = this.height;
      const k = this.k();
      ui.drawBackground(ctx, W, H);
      this.drawTitleTiles(ctx);
      this.txt(ctx, t("wd.mode." + this.gameMode) + "  ·  " + t("wd.setup.sub." + this.gameMode), this.fSmall.height, ui.TEXT_DIM, W - 30, false, "midtop", Math.floor(W / 2), this.suSubY);
      const palette = this.palette();
      const labPx = this.fSmall.height;
      const colW = this.suStart.w;
      this.txt(ctx, t("wd.setup.length"), labPx, ui.TEXT, colW, true, "midleft", this.suSeg[0].x + 2, this.suLabelY + Math.floor(labPx / 2));
      // Wortlänge: vier Knöpfe
      this.suSeg.forEach((rc, i) => {
        const on = LENGTHS[i] === this.length;
        const rad = Math.max(4, Math.floor(rc.h / 5));
        draw.rect(ctx, on ? palette.correct : ui.BTN, rc, 0, rad);
        if (!on) draw.rect(ctx, ui.BORDER, rc, 1, rad);
        if (this.setupFocus === 0 && on) draw.rect(ctx, ui.TEXT, rc.inflate(4, 4), 2, rad + 2);
        const numPx = Math.max(14, Math.floor(rc.h * 0.5));
        this.txt(ctx, String(LENGTHS[i]), numPx, on ? COL_LETTER : ui.TEXT, rc.w - 4, true, "center", rc.centerx, rc.centery);
      });
      // Schalter
      this.drawToggle(ctx, this.suHard, "hard", t("wd.setup.hard"), t("wd.setup.hard_desc"), this.hard, this.setupFocus === 1);
      this.drawToggle(ctx, this.suCb, "colorblind", t("wd.setup.colorblind"), t("wd.setup.colorblind_desc"), this.colorblind, this.setupFocus === 2, true);
      ui.drawButton(ctx, this.suStart, t("common.start"), this.fit(t("common.start"), this.suStart.w - 20, Math.max(13, Math.floor(20 * k)), true), this.setupFocus === 3, { accent: this.accent });
      this.drawSetupStats(ctx);
      const hint = this.fitParts(t("wd.setup.hint"), W - 20, Math.max(10, Math.floor(13 * k)));
      ui.drawFooter(ctx, W, H, hint, this.fit(hint, W - 20, Math.max(10, Math.floor(13 * k))));
    }

    /** WORDLE als Kachelreihe, die sich beim Öffnen nacheinander umdreht. */
    drawTitleTiles(ctx) {
      const word = "WORDLE";
      const size = this.suTile;
      const gap = Math.max(3, Math.floor(size / 9));
      const total = word.length * size + (word.length - 1) * gap;
      const x0 = Math.floor((this.width - total) / 2);
      const palette = this.palette();
      const states = ["correct", "absent", "present", "correct", "absent", "correct"];
      const f = ui.font(Math.max(10, Math.floor(size * 0.58)), true);
      for (let i = 0; i < word.length; i++) {
        const p = (this.anim - this.setupT - 0.08 * i) / FLIP_TIME;
        let sy = 1;
        let state = states[i];
        if (p < 0.5) {
          state = null;
          sy = p > 0 ? Math.max(0.05, Math.cos(Math.max(0, p) * Math.PI)) : 1;
        } else if (p < 1) {
          sy = Math.max(0.05, -Math.cos(p * Math.PI));
        }
        const bob = Math.sin(this.anim * 2.2 + i * 0.7) * size * 0.04;
        const h = Math.max(1, Math.floor(size * sy));
        const cx = x0 + i * (size + gap) + Math.floor(size / 2);
        const cy = this.suTitleY + Math.floor(size / 2) + Math.floor(bob);
        const rc = new PG.Rect(cx - Math.floor(size / 2), cy - Math.floor(h / 2), size, h);
        const fill = state ? (state === "absent" ? COL_ABSENT : palette[state]) : this.emptyFill();
        const rad = Math.max(3, Math.floor(size / 9));
        draw.rect(ctx, fill, rc, 0, rad);
        if (!state) draw.rect(ctx, ui.BORDER_LIGHT, rc, 2, rad);
        ctx.save();
        ctx.translate(cx, cy);
        ctx.scale(1, sy);
        ui.text(ctx, word[i], 0, 0, f, COL_LETTER, "center");
        ctx.restore();
      }
    }

    drawToggle(ctx, rc, key, label, desc, value, focus, preview = false) {
      const k = this.k();
      const radius = Math.max(6, Math.floor(rc.h / 5));
      draw.rect(ctx, focus ? ui.BTN_SEL : ui.BTN, rc, 0, radius);
      draw.rect(ctx, focus ? this.accent : ui.BORDER, rc, 1, radius);
      // Schalter rechts (Knopf gleitet weich hin und her)
      const swH = Math.max(16, Math.floor(rc.h * 0.42));
      const swW = Math.floor(swH * 1.9);
      const sw = new PG.Rect(rc.right - swW - Math.max(8, Math.floor(12 * k)), rc.centery - Math.floor(swH / 2), swW, swH);
      let v = this.toggleAnim[key] != null ? this.toggleAnim[key] : value ? 1 : 0;
      v += ((value ? 1 : 0) - v) * 0.3;
      this.toggleAnim[key] = v;
      const onCol = this.palette().correct;
      draw.rect(ctx, ui.mix(ui.BORDER_LIGHT, onCol, v), sw, 0, Math.floor(swH / 2));
      const knobR = Math.floor(swH / 2) - 2;
      const kx = Math.floor(sw.x + Math.floor(swH / 2) + (swW - swH) * v);
      draw.circle(ctx, COL_LETTER, [kx, sw.centery], knobR);
      const textX = rc.x + Math.max(8, Math.floor(14 * k));
      let right = sw.x - Math.max(6, Math.floor(10 * k));
      if (preview) {
        const ps = Math.max(8, Math.floor(swH * 0.8));
        ["correct", "present"].forEach((state, i) => {
          const pr = [right - (2 - i) * (ps + 3), rc.centery - Math.floor(ps / 2), ps, ps];
          draw.rect(ctx, this.palette()[state], pr, 0, Math.max(2, Math.floor(ps / 5)));
        });
        right -= 2 * (ps + 3) + 4;
      }
      const maxW = right - textX;
      const labPx = this.fSmallB.height;
      const descPx = Math.max(9, Math.floor(11 * k));
      if (this.fits(desc, maxW, descPx) && rc.h >= labPx + descPx + 8) {
        this.txt(ctx, label, labPx, ui.TEXT, maxW, true, "midleft", textX, rc.y + rc.h * 0.36);
        this.txt(ctx, desc, descPx, ui.TEXT_DIM, maxW, false, "midleft", textX, rc.y + rc.h * 0.72);
      } else {
        // Zu eng (lange Sprache): nur die Überschrift.
        this.txt(ctx, label, labPx, ui.TEXT, maxW, true, "midleft", textX, rc.centery);
      }
    }

    drawSetupStats(ctx) {
      const k = this.k();
      const rect = this.suStatsRect;
      ui.drawPanel(ctx, rect, { accentTop: this.accent });
      const pad = Math.max(8, Math.floor(16 * k));
      const inner = rect.inflate(-2 * pad, -2 * pad);
      const head = t("wd.stats.title") + "  ·  " + t("wd.setup.letters", { n: this.length });
      const headPx = this.fSmallB.height;
      this.txt(ctx, head, headPx, ui.TEXT, inner.w, true, "midtop", inner.centerx, inner.y);
      const st = getStats(this.data, this.gameMode, this.lang, this.length);
      const infoPx = Math.max(10, Math.floor(13 * k));
      const block = new PG.Rect(inner.x, inner.y + headPx + Math.max(6, Math.floor(12 * k)), inner.w, inner.h - headPx - Math.max(6, Math.floor(12 * k)) - infoPx - Math.max(6, Math.floor(10 * k)));
      this.drawStatsBlock(ctx, block, st);
      this.txt(ctx, this.setupInfo(), infoPx, ui.GOLD, inner.w, false, "midbottom", inner.centerx, inner.bottom);
    }

    setupInfo() {
      const mode = this.gameMode;
      if (mode === "endless") {
        const best = this.data.best[String(this.length)];
        return t("wd.result.best", { n: Number.isInteger(best) ? best : 0 });
      }
      if (mode === "daily") {
        const prog = this.data.daily[this.dailySlot()];
        if (prog && typeof prog === "object" && prog.date === PG.seedrand.todayStr()) {
          if (prog.done) return t("wd.daily.next", { t: formatHms(secondsToMidnight()) });
          return t("wd.daily.progress", { n: Array.isArray(prog.guesses) ? prog.guesses.length : 0 });
        }
        return t("wd.daily.ready");
      }
      return t("wd.setup.sub." + mode);
    }
  }

  // Nur in der Web-Version nötig: dort wird die Wortliste der Sprache/Länge
  // erst beim Spielstart nachgeladen.
  PG.addStrings({
    de: { "web.wordle.loading": "Wortliste wird geladen …" },
    en: { "web.wordle.loading": "Loading word list …" },
    fr: { "web.wordle.loading": "Chargement de la liste de mots …" },
    es: { "web.wordle.loading": "Cargando la lista de palabras …" },
    pt: { "web.wordle.loading": "A carregar a lista de palavras …" },
    pl: { "web.wordle.loading": "Wczytywanie listy słów …" },
    tr: { "web.wordle.loading": "Kelime listesi yükleniyor …" },
    da: { "web.wordle.loading": "Indlæser ordlisten …" },
    no: { "web.wordle.loading": "Laster ordlisten …" },
    sv: { "web.wordle.loading": "Laddar ordlistan …" },
    fi: { "web.wordle.loading": "Ladataan sanalistaa …" },
    cs: { "web.wordle.loading": "Načítání seznamu slov …" },
    sl: { "web.wordle.loading": "Nalaganje seznama besed …" },
    hr: { "web.wordle.loading": "Učitavanje popisa riječi …" },
  });

  // Für Tests (tools/wordle_check.js): reine Logik ohne Zeichnen.
  PG.wordle = { evaluate, hardProblem, dailyAnswer, shareText, getStats, recordGame, shownStreak, secondsToMidnight, formatHms, MAX_ROWS, BOARDS };

  PG.register(WordleGame, {
    id: "WordleGame",
    key: "wordle",
    name: "Wordle",
    modes: MODES,
    settingsKey: "wordle",
    defaults: { length: DEFAULT_LENGTH, hard: false, colorblind: false },
  });
})();
