/*
 * app.js - die Shell der Web-Version (Nachbau von main.py + menu.PreGameScreen)
 * ============================================================================
 * - lädt die Spiele laut js/manifest.js
 * - Sidebar (Spieleliste, Status, Einstellungen), Startbildschirm, Vorspiel-Screen
 * - zentrale Game-Loop (requestAnimationFrame), Eingabe -> InputEvents
 * - Pause (ESC), Highscores bei Game Over, Konfetti bei Rekord
 * - LamaWiki als HTML-Fenster
 *
 * Direktstart: index.html?game=SnakeGame[&mode=classic]
 */
(function () {
  "use strict";

  const PG = window.PG;
  const ui = PG.ui;
  const draw = PG.draw;
  const t = PG.t;
  const W = PG.W, H = PG.H;

  // ------------------------------------------------------------ Tastennamen
  // Browser-KeyboardEvent.key -> Tkinter-keysym (die Spiele kennen diese Namen).
  const KEYMAP = {
    ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right",
    " ": "space", Enter: "Return", Escape: "Escape", Backspace: "BackSpace", Tab: "Tab",
    Delete: "Delete", Home: "Home", End: "End", PageUp: "Prior", PageDown: "Next",
    Shift: "Shift_L", Control: "Control_L", Alt: "Alt_L", Meta: "Super_L", CapsLock: "Caps_Lock",
    Insert: "Insert", Spacebar: "space",
  };
  const CHARMAP = {
    "-": "minus", "+": "plus", "=": "equal", ",": "comma", ".": "period", "/": "slash", "*": "asterisk",
    "?": "question", "!": "exclam", "#": "numbersign", ";": "semicolon", ":": "colon", _: "underscore",
    "(": "parenleft", ")": "parenright", "[": "bracketleft", "]": "bracketright", "'": "apostrophe",
    '"': "quotedbl", "<": "less", ">": "greater", "&": "ampersand", "%": "percent", $: "dollar", "@": "at",
    "^": "asciicircum", "~": "asciitilde", "`": "grave", "|": "bar", "\\": "backslash", "{": "braceleft",
    "}": "braceright", "ä": "adiaeresis", "ö": "odiaeresis", "ü": "udiaeresis", "Ä": "Adiaeresis",
    "Ö": "Odiaeresis", "Ü": "Udiaeresis", "ß": "ssharp",
  };
  function toKeysym(e) {
    if (e.key === "Enter" && e.code === "NumpadEnter") return "KP_Enter";
    // Rechte Modifier-Tasten wie in Tkinter unterscheiden (z.B. Pinball-Flipper).
    if (e.code === "ShiftRight") return "Shift_R";
    if (e.code === "ControlRight") return "Control_R";
    if (e.code === "AltRight") return "Alt_R";
    if (KEYMAP[e.key]) return KEYMAP[e.key];
    if (e.key && e.key.length === 1) return CHARMAP[e.key] || e.key;
    return e.key || "";
  }

  function ev(kind, props) {
    return Object.assign({ kind, key: null, char: "", pos: null, button: 1, delta: 0, rel: null, repeat: false }, props);
  }

  // --------------------------------------------------------------- Screens
  /** Startbildschirm: Logo, klickbares Spiele-Raster, Highscore-Laufband. */
  class HomeScreen {
    constructor(app) {
      this.app = app;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.hover = null;
      this.tiles = [];
      this.name = t("app.menu_title");
    }
    update() {}
    flow(avail) {
      const maxW = W - 60;
      let result = null;
      for (const fsize of [15, 13, 11]) {
        const fnt = ui.font(fsize);
        const padX = fsize - 4;
        const rowH = fnt.height + 12;
        const gap = 8;
        const rows = [[]];
        let rw = 0;
        for (const entry of this.app.entries) {
          const tw = fnt.width(PG.gameName(entry)) + padX * 2 + 16;
          let add = tw + (rows[rows.length - 1].length ? gap : 0);
          if (rw + add > maxW && rows[rows.length - 1].length) {
            rows.push([]);
            rw = 0;
            add = tw;
          }
          rows[rows.length - 1].push([entry, tw]);
          rw += add;
        }
        const total = rows.length * rowH + (rows.length - 1) * gap;
        result = { rows, fnt, padX, rowH, gap, total };
        if (total <= avail) break;
      }
      return result;
    }
    draw(ctx) {
      ui.drawBackground(ctx, W, H);
      const modern = ui.isModern();
      const ts = ui.now();
      const cx = W / 2;
      if (ui.fx("celestial")) {
        const m = Math.min(W, H);
        ui.drawBlackHole(ctx, [W * 0.18, H * 0.28 + 4 * Math.sin(ts * 0.18)], Math.max(48, m * 0.24));
        ui.drawSaturn(ctx, [W * 0.84, H * 0.2 + 5 * Math.sin(ts * 0.14 + 2)], Math.max(16, m * 0.075));
      }
      const amp = ui.fx("menu_bob");
      const bob = amp ? Math.round(amp * Math.sin(ts * 1.3)) : 0;
      const size = Math.min(176, Math.max(96, Math.floor(H / 4)));
      const centerY0 = H / 2 - 46;
      const avail0 = H - 72 - (centerY0 + size / 2 + 80);
      const lay = this.flow(Math.max(30, avail0));
      const shift = Math.max(0, lay.total - avail0);
      const centerY = Math.max(size / 2 + 14, centerY0 - shift) + bob;
      const logo = this.app.logo;
      let baseY, lineW;
      if (logo && logo.complete && logo.naturalWidth) {
        const lr = new PG.Rect(0, 0, size, size);
        lr.center = [cx, centerY];
        const rad = Math.max(12, size / 8);
        if (!modern) draw.rect(ctx, [...ui.ACCENT, 55 + 35 * ui.pulse(1.4)], lr.inflate(26, 26), 0, rad + 8);
        ctx.save();
        ui.roundPath(ctx, lr.x, lr.y, lr.w, lr.h, rad);
        ctx.clip();
        ctx.drawImage(logo, lr.x, lr.y, lr.w, lr.h);
        ctx.restore();
        draw.rect(ctx, modern ? ui.BORDER_LIGHT : ui.ACCENT, lr.inflate(4, 4), modern ? 1 : 2, rad + 2);
        baseY = lr.bottom - bob;
        lineW = lr.w;
      } else {
        const lf = ui.font(64, true);
        ui.text(ctx, "PyGameZ", cx, centerY, lf, ui.TEXT, "center");
        baseY = centerY - bob + lf.height / 2;
        lineW = lf.width("PyGameZ") + 30;
      }
      if (!modern) {
        [ui.ACCENT, ui.ACCENT2, ui.GOLD].forEach((col, k) => {
          const ang = ts * (0.6 + 0.17 * k) + k * 2.09;
          const ox = cx + Math.cos(ang) * (size / 2 + 34);
          const oy = centerY + Math.sin(ang) * (size / 2 + 12);
          draw.circle(ctx, [...col, 110], [ox, oy], 4);
          draw.circle(ctx, col, [ox, oy], 2);
        });
      }
      if (modern) {
        const lw2 = Math.max(72, Math.min(lineW, 180));
        draw.rect(ctx, ui.ACCENT, [cx - lw2 / 2, baseY + 12, lw2, 2], 0, 2);
      } else {
        draw.rect(ctx, ui.ACCENT, [cx - lineW / 2, baseY + 12, lineW, 3], 0, 2);
      }
      ui.text(ctx, t("app.menu_title"), cx, baseY + 36, ui.font(18), ui.TEXT_DIM, "center");
      ui.text(ctx, t("app.menu_sub"), cx, baseY + 62, ui.font(15), ui.ACCENT, "center", modern ? 1 : ui.pulse(2.2, 0.45));

      // Spiele-Raster
      const top = baseY + 80, bottom = H - 72;
      const avail = Math.max(30, bottom - top);
      let y = top + Math.min(36, Math.max(0, (avail - lay.total) / 2));
      this.tiles = [];
      for (const row of lay.rows) {
        const rowW = row.reduce((s, r) => s + r[1], 0) + lay.gap * (row.length - 1);
        let x = W / 2 - rowW / 2;
        for (const [entry, tw] of row) {
          const rect = new PG.Rect(Math.round(x), Math.round(y), tw, lay.rowH);
          const idx = this.tiles.length;
          const accent = ui.gameColor(entry.id);
          const rr = lay.rowH / 2;
          if (idx === this.hover) {
            if (!modern) draw.rect(ctx, [...accent, 55], rect.inflate(14, 14), 0, rr + 7);
            draw.rect(ctx, ui.PANEL_LIGHT, rect, 0, rr);
            draw.rect(ctx, accent, rect, 1, rr);
          } else {
            draw.rect(ctx, ui.PANEL, rect, 0, rr);
            draw.rect(ctx, ui.BORDER, rect, 1, rr);
          }
          draw.circle(ctx, accent, [rect.x + lay.padX + 4, rect.centery], 4);
          ui.text(ctx, PG.gameName(entry), rect.x + lay.padX + 14, rect.centery, lay.fnt, idx === this.hover ? ui.TEXT : ui.TEXT_DIM, "midleft");
          this.tiles.push([rect, entry]);
          x += tw + lay.gap;
        }
        y += lay.rowH + lay.gap;
      }
      this.drawTicker(ctx);
      ui.drawFooter(ctx, W, H, `${this.app.entries.length} Games   ·   PyGameZ Web   ·   ${W}x${H}`);
    }
    drawTicker(ctx) {
      const scores = PG.highscore.all();
      const entries = this.app.entries
        .map((e) => [PG.gameName(e), Number(scores[e.meta.key]) || 0])
        .filter((e) => e[1] > 0)
        .sort((a, b) => b[1] - a[1])
        .slice(0, 10);
      if (!entries.length) return;
      const text = t("app.top_scores").toUpperCase() + "   —   " + entries.map(([n, v]) => n.toUpperCase() + "  " + String(v).replace(/\B(?=(\d{3})+(?!\d))/g, " ")).join("   ·   ");
      const fnt = ui.font(14, true);
      const bandH = 26, bandY = H - 64;
      ctx.fillStyle = ui.col(ui.BG_TOP, 135 / 255);
      ctx.fillRect(0, bandY, W, bandH);
      const lc = ui.mix(ui.BG_BOTTOM, ui.ACCENT, 0.35);
      draw.line(ctx, lc, [0, bandY], [W, bandY]);
      draw.line(ctx, lc, [0, bandY + bandH], [W, bandY + bandH]);
      const span = fnt.width(text) + 140;
      const off = (ui.now() * 42) % span;
      ctx.save();
      ctx.beginPath();
      ctx.rect(0, bandY, W, bandH);
      ctx.clip();
      for (let x = -off; x < W; x += span) ui.text(ctx, text, x, bandY + bandH / 2, fnt, ui.TEXT_DIM, "midleft");
      ctx.restore();
    }
    hit(pos) {
      for (let i = 0; i < this.tiles.length; i++) if (this.tiles[i][0].collidepoint(pos)) return i;
      return null;
    }
    handleEvent(e) {
      if (e.kind === "mousemove") {
        this.hover = this.hit(e.pos);
        this.app.canvas.style.cursor = this.hover != null ? "pointer" : "";
      } else if (e.kind === "mousedown" && e.button === 1) {
        const i = this.hit(e.pos);
        if (i != null) {
          const [rect, entry] = this.tiles[i];
          ui.spawnBurst(rect.centerx, rect.centery, ui.gameColor(entry.id));
          PG.audio.play("click");
          this.app.openPreGame(entry);
        }
      }
    }
  }

  /** Vorspiel-Screen: Modus wählen / Wiki / zurück. */
  class PreGameScreen {
    constructor(app, entry) {
      this.app = app;
      this.entry = entry;
      this.isMenu = true;
      this.gameOver = false;
      this.paused = false;
      this.accent = ui.gameColor(entry.id);
      this.best = PG.highscore.get(entry.meta.key);
      this.name = PG.gameName(entry);
      this.sel = 0;
      this.buttons = [];
      const modes = entry.meta.modes;
      if (modes && modes.length) {
        for (const [mode, label] of modes) this.buttons.push([PG.hasT(label) ? t(label) : PG.pick(label), () => app.launchGame(entry, mode)]);
      } else {
        this.buttons.push([t("pregame.single"), () => app.launchGame(entry, "single")]);
      }
      this.buttons.push([t("pregame.lamawiki"), () => app.openWiki(entry.id)]);
      this.buttons.push([t("pregame.back"), () => app.backToMenu()]);
      let bw = 300, bh = 40, gap = 10;
      let total = this.buttons.length * (bh + gap) - gap;
      let y0 = Math.max(132, H / 2 - total / 2 + 6);
      if (y0 + total > H - 40) {
        bh = 32;
        gap = 6;
        total = this.buttons.length * (bh + gap) - gap;
        y0 = Math.max(120, H / 2 - total / 2 + 6);
      }
      this.rects = this.buttons.map((_, i) => new PG.Rect(W / 2 - bw / 2, y0 + i * (bh + gap), bw, bh));
    }
    update() {}
    activate(i) {
      PG.audio.play("click");
      const r = this.rects[i];
      ui.spawnBurst(r.centerx, r.centery, this.accent);
      this.buttons[i][1]();
    }
    handleEvent(e) {
      if (e.kind === "keydown") {
        if (e.key === "Escape") this.app.backToMenu();
        else if (e.key === "Up" || e.key === "w") {
          this.sel = PG.mod(this.sel - 1, this.buttons.length);
          PG.audio.play("move");
        } else if (e.key === "Down" || e.key === "s") {
          this.sel = PG.mod(this.sel + 1, this.buttons.length);
          PG.audio.play("move");
        } else if (e.key === "Return" || e.key === "space" || e.key === "KP_Enter") this.activate(this.sel);
      } else if (e.kind === "mousemove") {
        let hov = false;
        this.rects.forEach((r, i) => {
          if (r.collidepoint(e.pos)) {
            this.sel = i;
            hov = true;
          }
        });
        this.app.canvas.style.cursor = hov ? "pointer" : "";
      } else if (e.kind === "mousedown" && e.button === 1) {
        this.rects.forEach((r, i) => {
          if (r.collidepoint(e.pos)) this.activate(i);
        });
      }
    }
    draw(ctx) {
      ui.drawBackground(ctx, W, H);
      ui.drawTitle(ctx, W, this.name, { subtitle: t("pregame.mode"), y: 64, accent: this.accent });
      const bf = ui.font(19);
      this.buttons.forEach(([label], i) => ui.drawButton(ctx, this.rects[i], label, bf, i === this.sel, { accent: this.accent }));
      if (this.best > 0) {
        const fnt = ui.font(14, true);
        const str = t("app.highscore", { hs: this.best });
        const bw = fnt.width(str) + 38, bh = fnt.height + 10;
        const chip = new PG.Rect(W / 2 - bw / 2, H - 66 - bh / 2, bw, bh);
        if (this.rects[this.rects.length - 1].bottom + 8 < chip.top) {
          ui.drawPanel(ctx, chip, { radius: bh / 2, shadow: false });
          draw.circle(ctx, ui.GOLD, [chip.x + 14, chip.centery], 3);
          ui.text(ctx, str, chip.x + 24, chip.centery, fnt, ui.GOLD, "midleft");
        }
      }
      ui.drawFooter(ctx, W, H, t("pregame.hint"));
    }
  }

  // -------------------------------------------------------------------- App
  class App {
    constructor() {
      this.canvas = document.getElementById("screen");
      this.ctx = this.canvas.getContext("2d");
      this.wrap = document.getElementById("canvas-wrap");
      this.pixelScale = 1;
      this.cssScale = 1;
      this.current = null;
      this.entry = null; // aktives Spiel (Manifest-Eintrag)
      this.entries = [];
      this.pressed = new Set();
      this.lastTs = 0;
      this.pauseRects = [];
      this.pauseSel = 0;
      this.errorMsg = null;
      this.errorAt = 0;
      this.statusAt = 0;
      this.logo = new Image();
      this.logo.src = "img/logo-256.png";
      this.locked = false;
      PG.app = this;
    }

    // ----- Laden ---------------------------------------------------------
    start() {
      document.documentElement.lang = PG.lang;
      this.applyDomI18n();
      this.bindUi();
      this.bindInput();
      this.resize();
      new ResizeObserver(() => this.resize()).observe(this.wrap);
      window.addEventListener("resize", () => this.resize());
      this.current = new HomeScreen(this);
      requestAnimationFrame((ts) => this.frame(ts));
      this.loadGames(() => {
        this.entries = PG.MANIFEST_ORDER.map((id) => PG.gameById[id]).filter(Boolean);
        // Spiele, die nicht im Manifest stehen, trotzdem anhängen
        for (const g of PG.games) if (!this.entries.includes(g)) this.entries.push(g);
        document.getElementById("loading").hidden = true;
        this.buildGameList();
        this.current = new HomeScreen(this);
        this.updateStatus(true);
        const q = new URLSearchParams(location.search);
        const direct = q.get("game") && PG.gameById[q.get("game")];
        if (direct) {
          if (q.get("mode") || !(direct.meta.modes && direct.meta.modes.length)) this.launchGame(direct, q.get("mode") || "single");
          else this.openPreGame(direct);
        }
      });
    }

    loadGames(done) {
      const manifest = window.PG_MANIFEST || [];
      PG.MANIFEST_ORDER = manifest.map((m) => m.id);
      const files = [];
      for (const m of manifest) for (const f of m.files) if (!files.includes(f)) files.push(f);
      const label = document.querySelector("#loading span");
      let i = 0;
      const next = () => {
        if (i >= files.length) return done();
        const src = files[i++];
        label.textContent = `Loading ${i}/${files.length}`;
        const s = document.createElement("script");
        s.src = src;
        s.onload = next;
        s.onerror = () => {
          console.warn("[PyGameZ] Datei fehlt:", src);
          next();
        };
        document.body.appendChild(s);
      };
      next();
    }

    // ----- Größe -----------------------------------------------------------
    resize() {
      const pad = window.innerWidth <= 820 ? 0 : 24;
      const aw = Math.max(100, this.wrap.clientWidth - pad);
      const ah = Math.max(75, this.wrap.clientHeight - pad);
      const scale = Math.min(aw / W, ah / H);
      const cssW = Math.floor(W * scale), cssH = Math.floor(H * scale);
      const dpr = window.devicePixelRatio || 1;
      const bw = Math.max(1, Math.round(cssW * dpr)), bh = Math.max(1, Math.round(cssH * dpr));
      if (this.canvas.width !== bw || this.canvas.height !== bh) {
        this.canvas.width = bw;
        this.canvas.height = bh;
      }
      this.canvas.style.width = cssW + "px";
      this.canvas.style.height = cssH + "px";
      this.cssScale = cssW / W;
      this.pixelScale = bw / W;
    }

    /** Setzt die Basis-Transformation (logische Koordinaten) auf ctx. */
    resetTransform(ctx = this.ctx) {
      ctx.setTransform(this.pixelScale, 0, 0, this.pixelScale, 0, 0);
    }

    toLogical(clientX, clientY) {
      const r = this.canvas.getBoundingClientRect();
      const x = ((clientX - r.left) / r.width) * W;
      const y = ((clientY - r.top) / r.height) * H;
      return [Math.floor(PG.clamp(x, 0, W - 1)), Math.floor(PG.clamp(y, 0, H - 1))];
    }

    // ----- Screens ---------------------------------------------------------
    get game() {
      return this.current && !this.current.isMenu ? this.current : null;
    }

    openPreGame(entry) {
      this.leaveGame();
      this.entry = entry;
      this.current = new PreGameScreen(this, entry);
      ui.beginTransition();
      this.markActive();
      this.canvas.focus();
    }

    launchGame(entry, mode) {
      this.leaveGame();
      this.entry = entry;
      this.canvas.style.cursor = "";
      try {
        const g = new entry.cls(W, H, mode || "single");
        g.paused = false;
        g.ctx = this.ctx;
        this.current = g;
        PG.stats.gameStarted(entry.meta.key);
      } catch (err) {
        console.error(err);
        this.showError(err);
        this.current = new PreGameScreen(this, entry);
      }
      ui.beginTransition();
      this.markActive();
      this.canvas.focus();
    }

    /** Beendet ein laufendes Spiel (Highscore sichern, aufräumen). */
    leaveGame() {
      const g = this.game;
      if (g) {
        this.saveHighscore(g);
        try {
          g.destroy();
        } catch (e) {
          console.error(e);
        }
      }
      if (document.pointerLockElement) document.exitPointerLock();
      this.releaseKeys();
    }

    backToMenu() {
      this.leaveGame();
      PG.stats.flush();
      this.entry = null;
      this.current = new HomeScreen(this);
      this.canvas.style.cursor = "";
      ui.beginTransition();
      this.markActive();
      this.refreshScores();
    }

    saveHighscore(g) {
      const [hs, record] = PG.highscore.update(g.highscoreKey, g.score);
      g._hsValue = hs;
      g._hsRecord = record;
      if (record) PG.stats.recordBroken(g.highscoreKey);
      this.refreshScores();
    }

    showError(err) {
      this.errorMsg = String((err && err.message) || err);
      this.errorAt = performance.now();
    }

    // ----- Loop ------------------------------------------------------------
    frame(ts) {
      requestAnimationFrame((t2) => this.frame(t2));
      const dt = this.lastTs ? Math.min(0.1, Math.max(0, (ts - this.lastTs) / 1000)) : 1 / 60;
      this.lastTs = ts;
      const ctx = this.ctx;
      this.resetTransform(ctx);
      ctx.globalAlpha = 1;
      ctx.globalCompositeOperation = "source-over";
      ctx.fillStyle = "#000";
      ctx.fillRect(0, 0, W, H);

      const cur = this.current;
      const g = this.game;
      if (cur) {
        try {
          if (!cur.paused && !cur.gameOver) {
            cur.update(dt);
            if (g) PG.stats.addPlaytime(g.highscoreKey, dt);
          }
        } catch (err) {
          this.reportLoopError(err);
        }
        ctx.save();
        try {
          cur.draw(ctx);
        } catch (err) {
          this.reportLoopError(err);
        }
        ctx.restore();
        this.resetTransform(ctx);
        ctx.globalAlpha = 1;
        ctx.globalCompositeOperation = "source-over";
        ctx.filter = "none";
        ctx.shadowBlur = 0;
      }

      if (g && this.current === g) {
        if (g.gameOver && !g._hsSaved) {
          this.saveHighscore(g);
          g._hsSaved = true;
          if (g._hsRecord) ui.spawnConfetti(W, H);
        }
        if (g.gameOver) g._wasOver = true;
        else {
          g._hsSaved = false;
          if (g._wasOver) {
            g._wasOver = false;
            g._resultReported = false;
            PG.stats.gameStarted(g.highscoreKey);
          }
        }
        if (g.gameOver) this.drawHighscoreBanner(ctx, g);
        this.manageCapture(ctx, g);
        if (g.paused) this.drawPause(ctx);
      }

      PG.stats.maybeFlush();
      ui.drawFx(ctx, W, H, dt);
      if (this.errorMsg && performance.now() - this.errorAt < 6000) this.drawError(ctx);
      if (ts - this.statusAt > 200) {
        this.statusAt = ts;
        this.updateStatus();
      }
    }

    reportLoopError(err) {
      const msg = String((err && err.message) || err);
      if (msg !== this.errorMsg || performance.now() - this.errorAt > 3000) console.error(err);
      this.showError(err);
    }

    drawError(ctx) {
      const fnt = ui.font(13);
      const str = t("web.error") + ": " + this.errorMsg;
      const w = Math.min(W - 20, fnt.width(str) + 24);
      const r = new PG.Rect(W / 2 - w / 2, 8, w, 26);
      draw.rect(ctx, [120, 30, 40, 230], r, 0, 8);
      ctx.save();
      ctx.beginPath();
      ctx.rect(r.x + 6, r.y, r.w - 12, r.h);
      ctx.clip();
      ui.text(ctx, str, r.x + 12, r.centery, fnt, [255, 230, 230], "midleft");
      ctx.restore();
    }

    drawHighscoreBanner(ctx, g) {
      const fnt = ui.font(18, true);
      const record = g._hsRecord;
      const str = record ? t("app.new_highscore", { score: g.score }) : t("app.highscore", { hs: g._hsValue || 0 });
      const bw = fnt.width(str) + 36, bh = fnt.height + 14;
      const r = new PG.Rect(W / 2 - bw / 2, H - bh - 10, bw, bh);
      ui.drawPanel(ctx, r, { radius: bh / 2, shadow: false });
      if (record) {
        draw.rect(ctx, [...ui.GOLD, 90 * ui.pulse(3.0)], r.inflate(8, 8), 0, bh / 2 + 4);
        ui.drawPanel(ctx, r, { radius: bh / 2, shadow: false });
        draw.rect(ctx, ui.GOLD, r, 1, bh / 2);
        ui.gradText(ctx, str, r.centerx, r.centery, fnt, [255, 235, 170], [235, 175, 70], "center");
      } else {
        ui.text(ctx, str, r.centerx, r.centery, fnt, ui.TEXT_DIM, "center");
      }
    }

    drawPause(ctx) {
      // Weichgezeichnetes Spielbild (wo ctx.filter unterstützt wird)
      ctx.save();
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      if ("filter" in ctx) {
        ctx.filter = `blur(${Math.round(6 * this.pixelScale)}px)`;
        ctx.drawImage(this.canvas, 0, 0);
        ctx.filter = "none";
      }
      ctx.restore();
      ctx.fillStyle = "rgba(8,10,18,0.55)";
      ctx.fillRect(0, 0, W, H);
      const cw = Math.min(360, W - 40), ch = 210;
      const card = new PG.Rect(W / 2 - cw / 2, H / 2 - ch / 2, cw, ch);
      ui.drawPanel(ctx, card, { radius: 14 });
      draw.rect(ctx, ui.ACCENT, [card.x, card.y, card.w, 4], 0, [14, 14, 0, 0]);
      if (!ui.isModern()) draw.rect(ctx, ui.mix(ui.BORDER, ui.ACCENT, ui.pulse(2.0)), card.inflate(8, 8), 1, 16);
      const big = ui.font(42, true);
      if (ui.isModern()) ui.text(ctx, t("app.pause"), card.centerx, card.y + 46, big, ui.TEXT, "center");
      else ui.gradText(ctx, t("app.pause"), card.centerx, card.y + 46, big, null, null, "center");
      ui.text(ctx, t("app.pause_resume"), card.centerx, card.y + 84, ui.font(15), ui.TEXT_DIM, "center");
      const bf = ui.font(16);
      this.pauseRects = [new PG.Rect(card.x + 30, card.y + 108, cw - 60, 36), new PG.Rect(card.x + 30, card.y + 152, cw - 60, 36)];
      ui.drawButton(ctx, this.pauseRects[0], t("web.resume"), bf, this.pauseSel === 0);
      ui.drawButton(ctx, this.pauseRects[1], t("pregame.back"), bf, this.pauseSel === 1);
    }

    manageCapture(ctx, g) {
      let want = false;
      try {
        want = !g.paused && !g.gameOver && !!g.captureMouse;
      } catch (e) {}
      const locked = document.pointerLockElement === this.canvas;
      if (!want && locked) document.exitPointerLock();
      if (want && !locked && !g.paused) {
        const fnt = ui.font(14, true);
        const str = t("web.capture");
        const bw = fnt.width(str) + 30, bh = 28;
        const r = new PG.Rect(W / 2 - bw / 2, H - 44, bw, bh);
        draw.rect(ctx, [...ui.PANEL, 225], r, 0, bh / 2);
        draw.rect(ctx, [...ui.ACCENT, 60 + 160 * ui.pulse(2.4)], r, 1, bh / 2);
        ui.text(ctx, str, r.centerx, r.centery, fnt, ui.TEXT, "center");
      }
    }

    // ----- Pause -------------------------------------------------------------
    togglePause() {
      const g = this.game;
      if (!g || g.gameOver) return;
      g.paused = !g.paused;
      this.pauseSel = 0;
      if (g.paused) {
        this.releaseKeys();
        if (document.pointerLockElement) document.exitPointerLock();
      }
      this.updateStatus(true);
    }

    pauseEvent(e) {
      if (e.kind === "keydown") {
        if (e.key === "Up" || e.key === "Down" || e.key === "w" || e.key === "s" || e.key === "Tab") this.pauseSel = 1 - this.pauseSel;
        else if (e.key === "Return" || e.key === "space" || e.key === "KP_Enter") this.pauseActivate(this.pauseSel);
      } else if (e.kind === "mousemove") {
        this.pauseRects.forEach((r, i) => {
          if (r.collidepoint(e.pos)) this.pauseSel = i;
        });
      } else if (e.kind === "mousedown") {
        this.pauseRects.forEach((r, i) => {
          if (r.collidepoint(e.pos)) this.pauseActivate(i);
        });
      }
    }

    pauseActivate(i) {
      PG.audio.play("click");
      if (i === 0) this.togglePause();
      else this.backToMenu();
    }

    // ----- Eingabe -------------------------------------------------------------
    dispatch(e) {
      const cur = this.current;
      if (!cur) return;
      if (cur.paused) {
        this.pauseEvent(e);
        return;
      }
      try {
        cur.handleEvent(e);
      } catch (err) {
        this.reportLoopError(err);
      }
    }

    releaseKeys() {
      const cur = this.current;
      for (const key of this.pressed) {
        try {
          if (cur && !cur.paused) cur.handleEvent(ev("keyup", { key }));
        } catch (err) {}
      }
      this.pressed.clear();
    }

    wikiOpen() {
      return !document.getElementById("wiki-modal").hidden;
    }

    bindInput() {
      const isFormTarget = (target) => target && /^(INPUT|SELECT|TEXTAREA|BUTTON)$/.test(target.tagName) && target.type !== "checkbox" && target.type !== "range";

      window.addEventListener("keydown", (e) => {
        PG.audio.unlock();
        if (this.wikiOpen()) {
          if (e.key === "Escape") this.closeWiki();
          return;
        }
        if (isFormTarget(e.target) && e.target.tagName !== "BUTTON") return;
        if (e.ctrlKey || e.metaKey) return; // Browser-Kürzel (Strg+R, ...) nicht blockieren
        if (e.key === "F5" || e.key === "F12") return;
        const key = toKeysym(e);
        if (e.key === "F11") {
          e.preventDefault();
          this.toggleFullscreen();
          return;
        }
        e.preventDefault();
        if (e.target && e.target.tagName === "BUTTON") e.target.blur();
        if (key === "Escape") {
          const cur = this.current;
          if (!cur) return;
          let wantsEsc = false;
          try {
            wantsEsc = !!cur.isMenu || !!cur.wantsEscape;
          } catch (err) {}
          if (cur.paused) this.togglePause();
          else if (wantsEsc) this.dispatch(ev("keydown", { key: "Escape" }));
          else if (!cur.gameOver) this.togglePause();
          return;
        }
        if (!e.repeat) this.pressed.add(key);
        this.dispatch(ev("keydown", { key, char: e.key.length === 1 ? e.key : "", repeat: e.repeat }));
        if (!e.repeat) this.maybeLock();
      });

      window.addEventListener("keyup", (e) => {
        if (this.wikiOpen() || (isFormTarget(e.target) && e.target.tagName !== "BUTTON")) return;
        const key = toKeysym(e);
        this.pressed.delete(key);
        if (key === "Escape") return;
        const cur = this.current;
        if (cur && !cur.paused) {
          try {
            cur.handleEvent(ev("keyup", { key }));
          } catch (err) {
            this.reportLoopError(err);
          }
        }
      });

      window.addEventListener("blur", () => this.releaseKeys());
      document.addEventListener("visibilitychange", () => {
        if (document.hidden) {
          const g = this.game;
          if (g && !g.paused && !g.gameOver) this.togglePause();
          PG.stats.flush();
        }
      });
      window.addEventListener("beforeunload", () => {
        const g = this.game;
        if (g) PG.highscore.update(g.highscoreKey, g.score);
        PG.stats.flush();
      });

      const c = this.canvas;
      c.addEventListener("contextmenu", (e) => e.preventDefault());

      c.addEventListener("pointerdown", (e) => {
        PG.audio.unlock();
        c.focus();
        const cur = this.current;
        if (!cur) return;
        let button = e.button === 0 ? 1 : e.button === 2 ? 3 : 2;
        if (button === 2) return;
        const g = this.game;
        let rightOk = true;
        if (cur.isMenu) rightOk = false;
        else {
          try {
            rightOk = !!cur.wantsRightClick;
          } catch (err) {
            rightOk = false;
          }
        }
        if (button === 3 && !rightOk && !cur.paused) return;
        e.preventDefault();
        // Pointer-Capture anfordern (FPS-Look): dieser Klick fängt nur die Maus ein
        if (g && !g.paused && !g.gameOver && document.pointerLockElement !== c) {
          let want = false;
          try {
            want = !!g.captureMouse;
          } catch (err) {}
          if (want) {
            try {
              const p = c.requestPointerLock();
              if (p && p.catch) p.catch(() => {});
            } catch (err) {}
            return;
          }
        }
        try {
          c.setPointerCapture(e.pointerId);
        } catch (err) {}
        const pos = document.pointerLockElement === c ? [W / 2, H / 2] : this.toLogical(e.clientX, e.clientY);
        if (e.pointerType !== "mouse") this.dispatch(ev("mousemove", { pos }));
        this.dispatch(ev("mousedown", { pos, button }));
        this.maybeLock();
      });

      c.addEventListener("pointermove", (e) => {
        if (document.pointerLockElement === c) {
          const g = this.game;
          if (g && !g.paused) {
            const dx = PG.clamp(e.movementX || 0, -150, 150);
            const dy = PG.clamp(e.movementY || 0, -150, 150);
            if (dx || dy) this.dispatch(ev("mouserel", { rel: [dx, dy] }));
          }
          return;
        }
        this.dispatch(ev("mousemove", { pos: this.toLogical(e.clientX, e.clientY) }));
      });

      const up = (e) => {
        const button = e.button === 0 ? 1 : e.button === 2 ? 3 : 2;
        if (button === 2) return;
        const cur = this.current;
        if (!cur || cur.paused) return;
        if (button === 3) {
          let ok = false;
          try {
            ok = !cur.isMenu && !!cur.wantsRightClick;
          } catch (err) {}
          if (!ok) return;
        }
        const pos = document.pointerLockElement === c ? [W / 2, H / 2] : this.toLogical(e.clientX, e.clientY);
        this.dispatch(ev("mouseup", { pos, button }));
      };
      c.addEventListener("pointerup", up);
      c.addEventListener("pointercancel", up);

      c.addEventListener(
        "wheel",
        (e) => {
          const cur = this.current;
          if (!cur || cur.paused) return;
          e.preventDefault();
          const delta = e.deltaY < 0 ? 1 : e.deltaY > 0 ? -1 : 0;
          if (delta) this.dispatch(ev("wheel", { pos: this.toLogical(e.clientX, e.clientY), delta }));
        },
        { passive: false }
      );

      document.addEventListener("pointerlockchange", () => {
        const locked = document.pointerLockElement === c;
        const g = this.game;
        // Der Browser gibt die Maus bei ESC selbst frei -> wie in Python: Pause
        if (this.locked && !locked && g && !g.paused && !g.gameOver) {
          let want = false;
          try {
            want = !!g.captureMouse;
          } catch (err) {}
          if (want) this.togglePause();
        }
        this.locked = locked;
      });
    }

    /**
     * Fordert Pointer-Lock an, falls das Spiel ihn gerade (z.B. durch den
     * eben verarbeiteten Klick/Enter) will. Browser erlauben das nur innerhalb
     * eines Nutzer-Ereignisses - deshalb direkt nach dispatch() aufrufen, damit
     * kein zweiter Klick nötig ist.
     */
    maybeLock() {
      const g = this.game;
      const c = this.canvas;
      if (!g || g.paused || g.gameOver || document.pointerLockElement === c) return;
      let want = false;
      try {
        want = !!g.captureMouse;
      } catch (err) {}
      if (!want) return;
      try {
        const p = c.requestPointerLock();
        if (p && p.catch) p.catch(() => {});
      } catch (err) {}
    }

    toggleFullscreen() {
      const stage = document.getElementById("stage");
      if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
      else if (stage.requestFullscreen) stage.requestFullscreen().catch(() => {});
      this.canvas.focus();
    }

    // ----- Sidebar ------------------------------------------------------------
    applyDomI18n() {
      document.querySelectorAll("[data-i18n]").forEach((el) => {
        el.textContent = t(el.getAttribute("data-i18n"));
      });
      document.getElementById("search").placeholder = t("web.search");
      document.getElementById("wiki-search").placeholder = t("web.wiki_search");
      document.title = "PyGameZ Web";
    }

    bindUi() {
      document.getElementById("btn-menu").onclick = () => {
        PG.audio.play("click");
        this.backToMenu();
        this.canvas.focus();
      };
      document.getElementById("btn-pause").onclick = () => {
        this.togglePause();
        this.canvas.focus();
      };
      document.getElementById("btn-wiki").onclick = () => this.openWiki(this.entry ? this.entry.id : null);
      document.getElementById("btn-full").onclick = () => this.toggleFullscreen();

      const search = document.getElementById("search");
      search.addEventListener("input", () => this.buildGameList());
      search.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          const first = document.querySelector("#gamelist .game-item");
          if (first) first.click();
        } else if (e.key === "Escape") {
          search.value = "";
          this.buildGameList();
          this.canvas.focus();
        }
      });

      const lang = document.getElementById("set-lang");
      for (const [code, label] of PG.LANGS) lang.add(new Option(label, code));
      lang.value = PG.lang;
      lang.onchange = () => {
        PG.setLang(lang.value);
        this.applyDomI18n();
        this.buildGameList();
        if (this.current instanceof PreGameScreen) this.current = new PreGameScreen(this, this.current.entry);
        else if (this.current instanceof HomeScreen) this.current = new HomeScreen(this);
        if (!this.wikiOpen()) this.canvas.focus();
        this.updateStatus(true);
      };

      const theme = document.getElementById("set-theme");
      for (const [code, label] of PG.THEME_NAMES) theme.add(new Option(label, code));
      theme.value = PG.settings.data.theme;
      theme.onchange = () => {
        PG.settings.data.theme = theme.value;
        PG.settings.save();
        ui.setTheme(theme.value);
        ui.beginTransition();
        this.canvas.focus();
      };

      const sound = document.getElementById("set-sound");
      sound.checked = !!PG.settings.data.sound;
      sound.onchange = () => {
        PG.settings.data.sound = sound.checked;
        PG.settings.save();
        PG.audio.unlock();
        PG.audio.play("click");
      };
      const vol = document.getElementById("set-volume");
      vol.value = Math.round(PG.settings.data.volume * 100);
      vol.oninput = () => {
        PG.settings.data.volume = Number(vol.value) / 100;
      };
      vol.onchange = () => {
        PG.settings.save();
        PG.audio.play("select");
      };

      document.getElementById("btn-reset").onclick = () => {
        if (window.confirm(t("web.reset_confirm"))) {
          PG.highscore.clear();
          this.refreshScores();
          if (this.current instanceof PreGameScreen) this.current = new PreGameScreen(this, this.current.entry);
        }
      };

      // Wiki
      document.getElementById("wiki-close").onclick = () => this.closeWiki();
      document.getElementById("wiki-modal").addEventListener("pointerdown", (e) => {
        if (e.target.id === "wiki-modal") this.closeWiki();
      });
      document.getElementById("wiki-search").addEventListener("input", () => this.renderWikiNav());
    }

    buildGameList() {
      const list = document.getElementById("gamelist");
      const q = document.getElementById("search").value.trim().toLowerCase();
      const scores = PG.highscore.all();
      list.textContent = "";
      let n = 0;
      for (const entry of this.entries) {
        const name = PG.gameName(entry);
        if (q && !name.toLowerCase().includes(q) && !entry.id.toLowerCase().includes(q)) continue;
        n++;
        const b = document.createElement("button");
        b.className = "game-item" + (this.entry === entry ? " active" : "");
        b.style.setProperty("--gi-color", ui.GAME_COLORS[entry.id] || "#5b8def");
        b.dataset.id = entry.id;
        const dot = document.createElement("span");
        dot.className = "gi-dot";
        const nm = document.createElement("span");
        nm.className = "gi-name";
        nm.textContent = name;
        const hs = document.createElement("span");
        hs.className = "gi-hs";
        const v = Number(scores[entry.meta.key]) || 0;
        hs.textContent = v > 0 ? v.toLocaleString("de-DE") : "";
        b.append(dot, nm, hs);
        b.onclick = () => {
          PG.audio.unlock();
          PG.audio.play("click");
          this.openPreGame(entry);
        };
        list.appendChild(b);
      }
      if (!n) {
        const d = document.createElement("div");
        d.className = "gl-empty";
        d.textContent = t("web.no_results");
        list.appendChild(d);
      }
    }

    refreshScores() {
      const scores = PG.highscore.all();
      document.querySelectorAll("#gamelist .game-item").forEach((b) => {
        const e = PG.gameById[b.dataset.id];
        const v = e ? Number(scores[e.meta.key]) || 0 : 0;
        b.querySelector(".gi-hs").textContent = v > 0 ? v.toLocaleString("de-DE") : "";
      });
    }

    markActive() {
      document.querySelectorAll("#gamelist .game-item").forEach((b) => {
        b.classList.toggle("active", !!this.entry && b.dataset.id === this.entry.id);
      });
      const act = document.querySelector("#gamelist .game-item.active");
      if (act && act.scrollIntoView) act.scrollIntoView({ block: "nearest" });
      this.updateStatus(true);
    }

    updateStatus(force) {
      const cur = this.current;
      const g = this.game;
      const dot = document.getElementById("state-dot");
      const nameEl = document.getElementById("status-name");
      const detail = document.getElementById("status-detail");
      let name, text, color;
      if (!cur || !this.entry) {
        name = t("app.no_game");
        text = `${this.entries.length ? t("web.games_count", { n: this.entries.length }) : ""}`;
        color = "var(--text-dim)";
      } else if (!g) {
        name = PG.gameName(this.entry);
        text = t("pregame.mode");
        color = "var(--accent)";
      } else {
        name = PG.gameName(this.entry);
        const state = g.paused ? t("app.state_pause") : g.gameOver ? t("app.state_over") : t("app.state_running");
        color = g.paused ? "var(--gold)" : g.gameOver ? "var(--red, #e06c6c)" : "#58be84";
        const lines = t("app.status", { name, state, score: g.score, hs: PG.highscore.get(g.highscoreKey) }).split("\n");
        text = lines.slice(1).join("\n");
      }
      const key = name + "|" + text + "|" + color;
      if (!force && key === this._statusKey) return;
      this._statusKey = key;
      nameEl.textContent = name;
      detail.textContent = text;
      dot.style.background = color;
      document.getElementById("btn-menu").disabled = !this.entry;
      document.getElementById("btn-pause").disabled = !g || g.gameOver;
    }

    // ----- LamaWiki ------------------------------------------------------------
    wikiPages() {
      const W_ = window.PG_WIKI || {};
      const data = W_[PG.lang] || W_.de || { pages: [] };
      const hidden = new Set(["replays", "saving", "options"]);
      return (data.pages || []).filter((p) => p && p.id && !hidden.has(p.id) && (!p.game || PG.gameById[p.game] || p.game == null));
    }

    openWiki(gameId) {
      const pages = this.wikiPages();
      let page = null;
      if (gameId) page = pages.find((p) => p.game === gameId);
      this.wikiPage = (page || pages[0] || {}).id;
      document.getElementById("wiki-search").value = "";
      document.getElementById("wiki-modal").hidden = false;
      const g = this.game;
      if (g && !g.paused && !g.gameOver) this.togglePause();
      this.renderWikiNav();
      this.renderWikiPage();
    }

    closeWiki() {
      document.getElementById("wiki-modal").hidden = true;
      this.canvas.focus();
    }

    wikiCatLabel(cat) {
      for (const k of ["lamawiki.cat_" + cat, "lamawiki.cat." + cat, "lamawiki." + cat]) if (PG.hasT(k)) return t(k);
      return cat;
    }

    renderWikiNav() {
      const nav = document.getElementById("wiki-nav");
      const q = document.getElementById("wiki-search").value.trim().toLowerCase();
      nav.textContent = "";
      let lastCat = null;
      for (const p of this.wikiPages()) {
        if (q) {
          const hay = [p.title, ...(p.keywords || []), ...(p.sections || []).flatMap((s) => [s.h, ...(s.body || [])])].join(" ").toLowerCase();
          if (!hay.includes(q)) continue;
        }
        if (p.category !== lastCat) {
          lastCat = p.category;
          const c = document.createElement("div");
          c.className = "wn-cat";
          c.textContent = this.wikiCatLabel(p.category);
          nav.appendChild(c);
        }
        const b = document.createElement("button");
        b.textContent = p.title;
        b.className = p.id === this.wikiPage ? "active" : "";
        b.onclick = () => {
          this.wikiPage = p.id;
          this.renderWikiNav();
          this.renderWikiPage();
        };
        nav.appendChild(b);
      }
    }

    renderWikiPage() {
      const art = document.getElementById("wiki-page");
      const p = this.wikiPages().find((x) => x.id === this.wikiPage);
      art.textContent = "";
      if (!p) return;
      const esc = (s) => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
      const inline = (s) => esc(s).replace(/\[([^\]]+)\]/g, "<kbd>$1</kbd>");
      let html = `<h1>${esc(p.title)}</h1><div class="wp-line" style="background:${p.game ? ui.GAME_COLORS[p.game] || "var(--accent)" : "var(--accent)"}"></div>`;
      for (const s of p.sections || []) {
        if (s.h) html += `<h2>${esc(s.h)}</h2>`;
        let inList = false;
        for (const line of s.body || []) {
          if (line.startsWith("- ")) {
            if (!inList) {
              html += "<ul>";
              inList = true;
            }
            html += `<li>${inline(line.slice(2))}</li>`;
          } else {
            if (inList) {
              html += "</ul>";
              inList = false;
            }
            html += `<p>${inline(line)}</p>`;
          }
        }
        if (inList) html += "</ul>";
      }
      if (p.game && PG.gameById[p.game]) {
        html += `<p style="margin-top:22px"><button class="tbtn back" id="wiki-play">▶ ${esc(PG.gameName(p.game))}</button></p>`;
      }
      art.innerHTML = html;
      art.scrollTop = 0;
      const play = document.getElementById("wiki-play");
      if (play)
        play.onclick = () => {
          this.closeWiki();
          this.openPreGame(PG.gameById[p.game]);
        };
    }
  }

  window.addEventListener("DOMContentLoaded", () => new App().start());
})();
