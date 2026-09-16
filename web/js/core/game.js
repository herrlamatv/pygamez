/*
 * game.js - Basisklasse aller Spiele + Registrierung (Nachbau von game_base.py)
 * ============================================================================
 *
 *   (function () {
 *     "use strict";
 *     const { ui, draw, t } = PG;
 *
 *     class SnakeGame extends PG.Game {
 *       init()  { ... }            // einmalig VOR dem ersten reset()
 *       reset() { this.score = 0; this.gameOver = false; ... }
 *       update(dt) { ... }
 *       draw(ctx)  { ... }
 *       handleEvent(ev) { ... }
 *     }
 *
 *     PG.register(SnakeGame, {
 *       id: "SnakeGame",                  // = Python-Klassenname
 *       key: "snake",                     // highscore_key
 *       name: "Snake",                    // oder {default:"Chess", de:"Schach", ...}
 *       modes: [["classic", "snake.mode.classic"], ...],   // optional
 *       settingsKey: "snake", defaults: {wrap: false},      // optional
 *       wantsRightClick: false,
 *     });
 *   })();
 *
 * WICHTIG: keine Klassen-Felder ("foo = 1;" im Klassenrumpf) für Spielzustand
 * verwenden - die würden NACH reset() initialisiert und den Zustand
 * überschreiben. Zustand gehört in init()/reset().
 */
(function () {
  "use strict";

  const PG = window.PG;

  class Game {
    constructor(width, height, mode = "single") {
      this.width = width;
      this.height = height;
      this.mode = mode;
      this.multiplayer = false; // Web-Version: immer Einzelspieler
      this.settings = PG.settings.data;
      this.controls = PG.CONTROLS;
      this.score = 0;
      this.gameOver = false;
      this.paused = false;
      this.ctx = null;
      const meta = this.constructor.meta || {};
      this.meta = meta;
      this.highscoreKey = meta.key || "base";
      // Spiel-eigene Einstellungen (mit Standardwerten aufgefüllt)
      this.opts = meta.settingsKey ? PG.settings.section(meta.settingsKey, meta.defaults) : {};
      this.font = PG.ui.font(22);
      this.bigFont = PG.ui.font(48, true);
      this.accent = PG.ui.gameColor(meta.id);
      this.init();
      this.reset();
    }

    // ----- von Unterklassen zu überschreiben ------------------------------
    /** Einmalige Initialisierung (vor dem ersten reset). */
    init() {}
    /** Setzt das Spiel in den Startzustand zurück. */
    reset() {
      this.score = 0;
      this.gameOver = false;
    }
    /** Spiellogik; dt = vergangene Sekunden (max. 0.1). */
    update(dt) {}
    /** Zeichnet auf ctx (logische Koordinaten width x height). */
    draw(ctx) {}
    /** Eingabe-Ereignis (siehe ENGINE.md). */
    handleEvent(ev) {}
    /** Beim Verlassen des Spiels (Aufräumen, z.B. DOM-Elemente). */
    destroy() {}

    // Optionale Flags (dürfen auch Getter sein)
    get wantsRightClick() { return !!this.meta.wantsRightClick; }
    get wantsEscape() { return false; }
    get captureMouse() { return false; }
    /** Highscore-Banner bei Game Over? Modi ohne Highscore-Wertung liefern false. */
    get showHighscoreBanner() { return true; }

    // ----- Hilfsfunktionen -------------------------------------------------
    drawCenterText(ctx, text, font, color, yOffset = 0) {
      PG.ui.text(ctx, text, this.width / 2, this.height / 2 + yOffset, font, color, "center");
    }
    keyFor(player, action) {
      return (this.controls[player] || {})[action];
    }
    /** true, wenn key die Taste für action ist (WASD/Leertaste ODER Pfeile/Enter). */
    isAction(key, action, player) {
      const players = player ? [player] : ["p1", "p2"];
      return players.some((p) => (this.controls[p] || {})[action] === key);
    }
    /** true, wenn key keiner Aktion von Spieler 1/2 zugeordnet ist (feste Zusatztasten). */
    keyIsFree(key) {
      return !["p1", "p2"].some((p) => Object.values(this.controls[p] || {}).includes(key));
    }
    playSound(name) {
      PG.audio.play(name);
    }
    tone(freq, dur, wave, vol) {
      PG.audio.tone(freq, dur, wave, vol);
    }
    rumble(ms = 120) {
      PG.audio.rumble(ms);
    }
    /** Speichert this.opts / this.settings dauerhaft. */
    saveSettings() {
      PG.settings.save();
    }
    /** Sieg/Niederlage an die Statistik melden (pro Partie nur einmal). */
    reportResult(won) {
      if (this._resultReported) return;
      this._resultReported = true;
      PG.stats.recordResult(this.highscoreKey, !!won);
    }
    /**
     * Neue Runde innerhalb derselben Partie (z.B. Revanche im Brettspiel):
     * gibt reportResult() wieder frei, ohne dass gameOver umschalten muss.
     */
    resumeFromGameOver() {
      // Game Over aufheben, ohne dass app.js eine neue Partie zählt
      // (z.B. Rückgängig nach dem letzten Zug bei 2048).
      this.gameOver = false;
      this._resumeSameGame = true;
    }
    newRoundResult() {
      this._resultReported = false;
    }
    /** Erfolgs-Ereignis (z.B. "tile_2048") - wird in der Statistik vermerkt. */
    achEvent(id, value) {
      PG.stats.event(id, value);
    }
    /** Aktueller Highscore dieses Spiels. */
    get highscore() {
      return PG.highscore.get(this.highscoreKey);
    }
  }
  PG.Game = Game;

  // ------------------------------------------------------------- Registrierung
  PG.games = [];
  PG.gameById = {};

  /** Meldet ein Spiel an (Aufruf am Ende jeder Spieldatei). */
  PG.register = function (cls, meta) {
    if (!meta || !meta.id) throw new Error("PG.register: meta.id fehlt");
    cls.meta = meta;
    const entry = { id: meta.id, cls, meta };
    if (PG.gameById[meta.id]) {
      const i = PG.games.findIndex((g) => g.id === meta.id);
      PG.games[i] = entry;
    } else {
      PG.games.push(entry);
    }
    PG.gameById[meta.id] = entry;
    return cls;
  };

  /** Anzeigename eines Spiels in der aktiven Sprache. */
  PG.gameName = function (entryOrId) {
    const e = typeof entryOrId === "string" ? PG.gameById[entryOrId] : entryOrId;
    if (!e) return String(entryOrId);
    return PG.pick(e.meta.name) || e.id;
  };
})();
