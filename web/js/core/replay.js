/*
 * replay.js - Aufzeichnung, Archiv und Teilen der Wiederholungen
 * ============================================================================
 * 1:1-Port von replay.py. Aufgezeichnet wird NICHT die Eingabe, sondern die
 * tatsächliche BEWEGUNG der Objekte - Bild für Bild in einem festen Raster
 * (RATE Samples je Sekunde). Damit sieht die Wiederholung immer genau so aus
 * wie die gespielte Runde.
 *
 * Aufgezeichnet werden sechs Spiele:
 *
 *     minigolf   je Szene ein Schlag
 *     bowling    je Szene ein Wurf
 *     billiard   je Szene ein Stoß
 *     pinball    je Szene ein Ball
 *     snake      je Szene ein Abschnitt (Kapitel)
 *     tetris     je Szene ein Abschnitt (Kapitel)
 *
 * Ein Replay ist ein Objekt::
 *
 *     {v: 1, id: "...", game: "snake", date: "2026-09-18 20:10",
 *      title: "...", sub: "...", rate: 30, meta: {...},
 *      layouts: [...], scenes: [{...}, ...]}
 *
 * Unterschied zur Desktop-Version: das Archiv liegt im localStorage
 * (PG.store "replays") statt in replay.json. Der Platz dort ist knapp
 * (Browser: meist 5 MB je Seite), deshalb sind die Obergrenzen kleiner und
 * das Archiv wehrt sich, bevor der Speicher überläuft.
 *
 * Verwendung im Spiel::
 *
 *     this.rec = PG.replay.recorder("snake", {mode: ...});
 *     if (this.rec) this.rec.scene({...});          // Sequenz beginnt
 *     if (this.rec) this.rec.tick(dt, () => [...]); // je Frame
 *     if (this.rec) this.rec.close(sampleFn, {final: true});
 *     this.replay = this.rec.result({title, sub});  // Rundenende
 */
(function () {
  "use strict";

  const PG = window.PG;

  // Zeitraster der Aufnahme: 30 Samples je Sekunde (wie replay.py).
  const RATE = 30;
  const STEP = 1.0 / RATE;
  const VERSION = 1;

  // Spiele mit Aufzeichnung = Reihenfolge der Reiter im Replay-Screen.
  const GAMES = ["minigolf", "bowling", "billiard", "pinball", "snake", "tetris"];

  // Endung einer geteilten Aufnahme; beim Import wird auch ".json" genommen.
  const EXT = ".lamapgzreplay";
  const FORMAT = "pygamez.replay";

  // Je Spiel höchstens so viele gespeicherte Replays. Weniger als in der
  // Desktop-Version (20): der localStorage fasst nur wenige MB.
  const MAX_PER_GAME = 5;

  // Sicherheitsnetz gegen Endlos-Partien: mehr Samples bzw. mehr einzelne
  // Zahlen werden nicht aufgezeichnet. Die Aufnahme bricht dann ab und trägt
  // "capped" - der Replay-Screen weist darauf hin.
  const MAX_SAMPLES = 18000; // 10 Minuten
  const MAX_VALUES = 60000;

  // So groß darf das ganze Archiv im localStorage höchstens werden.
  const MAX_TOTAL_CHARS = 1500000;

  // ---------------------------------------------------------------- Einstellung
  function isEnabled() {
    const sect = PG.settings.data.replay;
    if (!sect || typeof sect !== "object") return true;
    return sect.enabled !== false;
  }

  function setEnabled(on) {
    const sect = PG.settings.section("replay", { enabled: true });
    sect.enabled = !!on;
    PG.settings.save();
  }

  // ------------------------------------------------------------ Archiv lesen
  function readRaw() {
    const d = PG.store.get("replays", {});
    return d && typeof d === "object" && !Array.isArray(d) ? d : {};
  }

  function valid(rep) {
    return !!(rep && typeof rep === "object" && Array.isArray(rep.scenes) && rep.scenes.length &&
              typeof rep.id === "string" && GAMES.includes(rep.game));
  }

  function writeRaw(data) {
    const out = {};
    for (const key of GAMES) {
      const items = data[key];
      if (Array.isArray(items) && items.length) out[key] = items;
    }
    const text = JSON.stringify(out);
    if (text.length > MAX_TOTAL_CHARS) return "space";
    return PG.store.trySet("replays", out) ? "" : "space";
  }

  function loadGame(game) {
    const items = readRaw()[game];
    return Array.isArray(items) ? items.filter(valid) : [];
  }

  function loadAll() {
    const raw = readRaw();
    const out = {};
    for (const game of GAMES) {
      const items = raw[game];
      out[game] = Array.isArray(items) ? items.filter(valid) : [];
    }
    return out;
  }

  const count = (game) => loadGame(game).length;
  const isFull = (game) => count(game) >= MAX_PER_GAME;
  const isSaved = (game, id) => loadGame(game).some((r) => r.id === id);

  /** Legt ein Replay dauerhaft ab. Gibt [ok, grund] zurück. */
  function saveReplay(rep) {
    if (!valid(rep)) return [false, "invalid"];
    const data = readRaw();
    let items = Array.isArray(data[rep.game]) ? data[rep.game] : [];
    items = items.filter((r) => valid(r) && r.id !== rep.id);
    if (items.length >= MAX_PER_GAME) return [false, "full"];
    items.unshift(rep); // neueste zuerst
    data[rep.game] = items;
    const why = writeRaw(data);
    return why ? [false, why] : [true, ""];
  }

  function deleteReplay(game, id) {
    const data = readRaw();
    const items = data[game];
    if (!Array.isArray(items)) return false;
    const keep = items.filter((r) => r.id !== id);
    if (keep.length === items.length) return false;
    data[game] = keep;
    return !writeRaw(data);
  }

  function uniqueId(wanted, game) {
    wanted = String(wanted || game || "replay");
    if (!isSaved(game, wanted)) return wanted;
    for (let n = 2; n < 1000; n++) {
      const cand = wanted + "-" + n;
      if (!isSaved(game, cand)) return cand;
    }
    return wanted + "-" + Date.now();
  }

  // --------------------------------------------------------------- Kennzahlen
  function sceneLen(scene) {
    const f = scene && scene.f;
    if (!Array.isArray(f)) return 0;
    return scene.flat ? Math.floor(f.length / 2) : f.length;
  }

  function duration(rep) {
    const rate = (rep && rep.rate) || RATE;
    let total = 0;
    for (const sc of (rep && rep.scenes) || []) total += sceneLen(sc);
    return total / rate;
  }

  /** Sekunden -> "1:05" (sprachneutral, für Listen und die Leiste). */
  function formatDuration(seconds) {
    const s = Math.max(0, Math.round(seconds));
    return Math.floor(s / 60) + ":" + String(s % 60).padStart(2, "0");
  }

  function stamp() {
    const d = new Date();
    const p = (n) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
  }

  // ------------------------------------------------------- Teilen (Datei)
  function defaultFilename(rep) {
    let base = String((rep && (rep.id || rep.game)) || "replay");
    base = base.replace(/[^A-Za-z0-9\-_]/g, "-").slice(0, 60) || "replay";
    return base + EXT;
  }

  /** Baut den Umschlag mit genau EINEM Replay (oder null bei Murks). */
  function exportText(rep) {
    if (!valid(rep)) return null;
    return JSON.stringify({ format: FORMAT, v: VERSION, app: "PyGameZ", exported: stamp(), replay: rep });
  }

  /** Liest eine geteilte Aufnahme. Gibt [ok, grund, replay] zurück. */
  function readText(text) {
    let raw;
    try {
      raw = JSON.parse(text);
    } catch (e) {
      return [false, "io", null];
    }
    if (!raw || typeof raw !== "object") return [false, "format", null];
    const rep = raw.format === FORMAT ? raw.replay : raw;
    if (!valid(rep)) return [false, "format", null];
    return [true, "", rep];
  }

  /**
   * Liest eine geteilte Aufnahme ein und legt sie ins Archiv.
   * 'game' ist die Art, die der aufrufende Reiter erwartet (null = jede).
   * Gibt [ok, grund, replay] zurück: "" | "io" | "format" | "game" | "full" |
   * "dup" | "space".
   */
  function importText(text, game) {
    const [ok, why, found] = readText(text);
    if (!ok) return [false, why, null];
    if (game && found.game !== game) return [false, "game", null];
    if (isFull(found.game)) return [false, "full", null];
    if (isSaved(found.game, found.id)) return [false, "dup", found];
    const rep = Object.assign({}, found);
    rep.id = uniqueId(rep.id, rep.game);
    rep.src = "file";
    const [saved, reason] = saveReplay(rep);
    return saved ? [true, "", rep] : [false, reason, null];
  }

  // ------------------------------------------------------------------ Delta
  /**
   * Kleiner Helfer für Samples, die nur Änderungen enthalten (siehe
   * replay.Delta in Python). push(key, state) ist true, wenn sich 'state'
   * seit dem letzten Mal geändert hat; Zustände werden als Zeichenkette
   * verglichen, damit auch Arrays funktionieren.
   */
  class Delta {
    constructor() {
      this.prev = new Map();
    }
    reset() {
      this.prev.clear();
    }
    push(key, state) {
      const s = typeof state === "string" ? state : JSON.stringify(state);
      if (this.prev.get(key) === s) return false;
      this.prev.set(key, s);
      return true;
    }
  }

  // --------------------------------------------------------------- Aufnahme
  class Recorder {
    constructor(game, meta) {
      this.game = game;
      this.meta = Object.assign({}, meta || {});
      this.layouts = [];
      this.scenes = [];
      this.samples = 0;
      this.values = 0;
      this.full = false; // Obergrenze erreicht: nichts Neues mehr
      this.dead = false;
      this._scene = null;
      this._acc = 0.0;
    }

    /** Legt eine Kulisse ab und gibt ihren Index zurück (ohne Dubletten). */
    layout(data) {
      if (this.dead || this.full) return 0;
      const key = JSON.stringify(data);
      for (let i = 0; i < this.layouts.length; i++) {
        if (JSON.stringify(this.layouts[i]) === key) return i;
      }
      this.layouts.push(data);
      return this.layouts.length - 1;
    }

    /** Beginnt eine Sequenz (bei Snake/Tetris: ein Kapitel mit Schlüsselbild). */
    scene(info, flat = false) {
      if (this.dead || this.full) return;
      this._scene = Object.assign({}, info || {});
      if (flat) this._scene.flat = true;
      this._scene.f = [];
      this.scenes.push(this._scene);
      this._acc = STEP; // das erste Sample sofort nehmen
    }

    /** Trägt Kopfdaten in die laufende Sequenz nach. */
    set(info) {
      if (this._scene) Object.assign(this._scene, info);
    }

    /** Trägt Kopfdaten in die zuletzt begonnene Sequenz nach. */
    setLast(info) {
      if (!this.dead && this.scenes.length) Object.assign(this.scenes[this.scenes.length - 1], info);
    }

    /** Nimmt im Raster RATE Samples auf (je Frame einmal aufrufen). */
    tick(dt, sampleFn) {
      if (this.dead || this.full || !this._scene) return;
      this._acc += dt;
      while (this._acc >= STEP) {
        this._acc -= STEP;
        this.emit(sampleFn());
      }
    }

    emit(values) {
      if (!values || this.full) return;
      const sc = this._scene;
      if (sc.flat) for (const v of values) sc.f.push(v);
      else sc.f.push(Array.from(values));
      this.samples += 1;
      this.values += values.length + 1;
      if (this.samples > MAX_SAMPLES || this.values > MAX_VALUES) {
        this.full = true;
        this._scene = null;
      }
    }

    /** Beendet die laufende Sequenz (mit einem letzten Sample). */
    close(sampleFn, info) {
      if (this.dead || !this._scene) return;
      if (sampleFn) this.emit(sampleFn());
      if (this._scene && info) Object.assign(this._scene, info);
      this._scene = null;
    }

    /** Wirft Sequenzen weg, deren Kopfdaten alle Vorgaben erfüllen. */
    dropWhere(match) {
      if (this.dead) return;
      const keep = [];
      for (const sc of this.scenes) {
        if (Object.keys(match).every((k) => sc[k] === match[k])) {
          this.samples -= sceneLen(sc);
          continue;
        }
        keep.push(sc);
      }
      this.scenes = keep;
      this._scene = null;
    }

    /** Baut das fertige Replay (oder null, wenn nichts brauchbar ist). */
    result(info) {
      if (this.dead) return null;
      const scenes = this.scenes.filter((sc) => sceneLen(sc) > 0);
      if (!scenes.length) return null;
      info = info || {};
      const { title, sub } = info;
      for (const k of Object.keys(info)) {
        if (k !== "title" && k !== "sub") this.meta[k] = info[k];
      }
      if (this.full) this.meta.capped = true;
      const d = new Date();
      const p = (n) => String(n).padStart(2, "0");
      const idStamp = `${d.getFullYear()}${p(d.getMonth() + 1)}${p(d.getDate())}${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
      const rep = {
        v: VERSION,
        id: `${this.game}-${idStamp}-${scenes.length}`,
        game: this.game,
        date: stamp(),
        title: title || "",
        sub: sub || "",
        rate: RATE,
        meta: this.meta,
        scenes,
      };
      if (this.layouts.length) rep.layouts = this.layouts;
      return rep;
    }
  }

  /** Erzeugt einen Recorder - oder null, wenn Replays abgeschaltet sind. */
  function recorder(game, meta) {
    if (!GAMES.includes(game) || !isEnabled()) return null;
    return new Recorder(game, meta);
  }

  PG.replay = {
    RATE, STEP, VERSION, GAMES, EXT, FORMAT, MAX_PER_GAME, MAX_SAMPLES, MAX_VALUES,
    isEnabled, setEnabled,
    loadGame, loadAll, count, isFull, isSaved, saveReplay, deleteReplay, uniqueId,
    valid, sceneLen, duration, formatDuration,
    defaultFilename, exportText, readText, importText,
    Delta, Recorder, recorder,
  };
})();
