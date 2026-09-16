/*
 * ui.js - UI-Toolkit der Web-Version (Nachbau von ui.py)
 * =======================================================
 * Farben liegen wie in Python als [r, g, b] in PG.ui.TEXT, PG.ui.ACCENT, ...
 * und werden von setTheme() umgeschaltet - daher IMMER dynamisch lesen.
 *
 *   const ui = PG.ui;
 *   ui.drawBackground(ctx, w, h);
 *   ui.drawTitle(ctx, w, "TITEL", {subtitle: "..."});
 *   ui.drawButton(ctx, rect, "Start", ui.font(19), selected);
 *   ui.text(ctx, "Hallo", x, y, ui.font(22), ui.TEXT, "center");
 *
 * Zeichen-Primitive im pygame.draw-Stil liegen in PG.draw (rect/circle/...).
 */
(function () {
  "use strict";

  const PG = window.PG;
  const ui = (PG.ui = {});

  // ------------------------------------------------------------------ Themes
  const CLASSIC = {
    BG_TOP: [14, 17, 29], BG_BOTTOM: [24, 28, 44], PANEL: [30, 35, 52], PANEL_LIGHT: [38, 44, 64],
    BORDER: [52, 60, 86], BORDER_LIGHT: [74, 84, 116], BTN: [40, 46, 66], BTN_SEL: [56, 88, 152],
    ACCENT: [88, 156, 255], ACCENT2: [155, 110, 255], ACCENT_SOFT: [66, 110, 180],
    GREEN: [110, 205, 140], GOLD: [245, 205, 100], RED: [225, 95, 95],
    TEXT: [235, 238, 245], TEXT_DIM: [150, 158, 178], TEXT_FAINT: [95, 102, 124],
  };
  const MODERN = {
    BG_TOP: [17, 19, 24], BG_BOTTOM: [22, 25, 32], PANEL: [28, 31, 40], PANEL_LIGHT: [37, 41, 52],
    BORDER: [45, 50, 62], BORDER_LIGHT: [64, 70, 86], BTN: [33, 37, 47], BTN_SEL: [47, 56, 76],
    ACCENT: [91, 141, 239], ACCENT2: [129, 155, 255], ACCENT_SOFT: [64, 94, 156],
    GREEN: [88, 190, 132], GOLD: [229, 196, 106], RED: [224, 108, 108],
    TEXT: [233, 235, 241], TEXT_DIM: [150, 157, 172], TEXT_FAINT: [100, 106, 122],
  };
  const V41 = {
    BG_TOP: [16, 18, 28], BG_BOTTOM: [22, 25, 36], PANEL: [28, 31, 42], PANEL_LIGHT: [37, 41, 55],
    BORDER: [46, 51, 66], BORDER_LIGHT: [66, 72, 92], BTN: [33, 37, 49], BTN_SEL: [48, 57, 80],
    ACCENT: [91, 141, 239], ACCENT2: [129, 155, 255], ACCENT_SOFT: [64, 94, 156],
    GREEN: [88, 190, 132], GOLD: [229, 196, 106], RED: [224, 108, 108],
    TEXT: [233, 235, 241], TEXT_DIM: [150, 157, 172], TEXT_FAINT: [100, 106, 122],
  };
  // UI v4.2 "Midnight Glass": tiefblaue Nacht, Indigo/Türkis-Akzente, Milchglas-Flächen.
  const V42 = {
    BG_TOP: [8, 10, 22], BG_BOTTOM: [14, 12, 30], PANEL: [21, 24, 44], PANEL_LIGHT: [31, 35, 62],
    BORDER: [46, 52, 86], BORDER_LIGHT: [80, 88, 130], BTN: [25, 29, 52], BTN_SEL: [40, 46, 86],
    ACCENT: [122, 132, 255], ACCENT2: [72, 214, 210], ACCENT_SOFT: [76, 84, 168],
    GREEN: [92, 214, 150], GOLD: [242, 200, 104], RED: [240, 104, 128],
    TEXT: [236, 238, 252], TEXT_DIM: [156, 162, 196], TEXT_FAINT: [102, 108, 146],
  };
  const FX_CLASSIC = {
    stars: true, star_bright: 1.0, aurora: true, shooting: true, celestial: false, pattern: null,
    vignette: 70, title_glow: true, btn_glow: true, btn_arrow: true, sparks: true, scanline: true,
    trans_dur: 0.35, panel_radius: 12, btn_radius: 10, shadow_alpha: 90, menu_bob: 6,
  };
  const FX_MODERN = {
    stars: false, star_bright: 0, aurora: false, shooting: false, celestial: false, pattern: null,
    vignette: 42, title_glow: false, btn_glow: false, btn_arrow: false, sparks: false, scanline: false,
    trans_dur: 0.22, panel_radius: 10, btn_radius: 8, shadow_alpha: 55, menu_bob: 0,
  };
  const FX_V41 = Object.assign({}, FX_MODERN, { stars: true, star_bright: 0.55, celestial: true, vignette: 48, menu_bob: 3 });
  // Farbwolken des v4.2-Hintergrunds: [Spitzenfarbe, Größe rel. max(w,h), Tempo, Phase, x, y]
  const V42_MESH = [
    [[40, 36, 112], 1.00, 0.050, 0.0, 0.18, 0.20], // Indigo, oben links
    [[8, 62, 70], 0.95, 0.041, 2.4, 0.84, 0.78],   // Türkis, unten rechts
    [[64, 16, 60], 0.80, 0.063, 4.1, 0.82, 0.16],  // Magenta, oben rechts
  ];
  const FX_V42 = Object.assign({}, FX_MODERN, {
    stars: true, star_bright: 0.45, aurora: false, shooting: false, celestial: false, pattern: null,
    vignette: 62, title_glow: false, title_grad: false, btn_glow: false, btn_arrow: false,
    sparks: false, scanline: false, trans_dur: 0.24, panel_radius: 12, btn_radius: 10,
    shadow_alpha: 80, menu_bob: 2, style: "v42", logo_glow: false, menu_orbit: false,
    star_count: 48, mesh: V42_MESH, mesh_drift: [0.10, 0.08], grain: 9, glass_alpha: 222,
    grad3: [[122, 132, 255], [72, 214, 210], [236, 96, 196]], logo_halo: true,
  });
  const patternFx = (pat) => Object.assign({}, FX_V41, { stars: false, star_bright: 0, vignette: 64, pattern: pat });
  // UI v2: die allererste ui.py (Commit 08739d3) - Palette fast wie v3,
  // Sternenfeld, statische Glow-Buttons, Titel mit Schatten, keine Animationen.
  const V2 = Object.assign({}, CLASSIC, { BG_TOP: [16, 19, 32], ACCENT2: [66, 110, 180] });
  const FX_V2 = Object.assign({}, FX_CLASSIC, {
    aurora: false, shooting: false, sparks: false, scanline: false, trans_dur: 0,
    title_glow: false, title_grad: false, menu_bob: 0, style: "v2", logo_glow: false, menu_orbit: false,
  });
  // UI v1: der flache Look VOR dem UI Rework (Commit cb71142) - einfarbiger
  // Hintergrund, Buttons ohne Rand (Auswahl nur über die Farbe), keine Deko.
  const V1 = {
    BG_TOP: [18, 20, 28], BG_BOTTOM: [18, 20, 28], PANEL: [30, 34, 46], PANEL_LIGHT: [44, 50, 66],
    BORDER: [44, 50, 66], BORDER_LIGHT: [62, 70, 92], BTN: [44, 50, 66], BTN_SEL: [70, 96, 150],
    ACCENT: [120, 200, 140], ACCENT2: [90, 160, 240], ACCENT_SOFT: [70, 96, 150],
    GREEN: [120, 200, 140], GOLD: [240, 210, 120], RED: [220, 90, 90],
    TEXT: [232, 234, 240], TEXT_DIM: [150, 158, 176], TEXT_FAINT: [105, 112, 130],
  };
  const FX_V1 = Object.assign({}, FX_MODERN, {
    vignette: 0, panel_radius: 8, btn_radius: 8, shadow_alpha: 0, trans_dur: 0, style: "v1", logo_glow: false, menu_orbit: false,
  });

  // Sidebar-Farben (CSS-Variablen) je Theme.
  const CSS_V42 = { sidebar: "#0d0f1f", header: "#090b18", card: "#151a2e", btn: "#1a1f38", "btn-hover": "#252c4f", accent: "#7a84ff", accent2: "#48d6d2", border: "#262c4a", text: "#eceefc", "text-dim": "#9ca2c4", "text-faint": "#666c92", gold: "#f2c868" };
  const CSS_V41 = { sidebar: "#13161f", header: "#0f1219", card: "#1a1e2a", btn: "#1f2431", "btn-hover": "#293040", accent: "#5b8def", accent2: "#819bff", border: "#2e3342", text: "#e9ebf1", "text-dim": "#969dac", "text-faint": "#646a7a", gold: "#e5c46a" };
  const CSS_MODERN = { sidebar: "#14161c", header: "#101217", card: "#1b1e27", btn: "#20242e", "btn-hover": "#2a2f3b", accent: "#5b8def", accent2: "#819bff", border: "#2d323e", text: "#e9ebf1", "text-dim": "#969dac", "text-faint": "#646a7a", gold: "#e5c46a" };
  const CSS_CLASSIC = { sidebar: "#12151f", header: "#0c0f18", card: "#1a2030", btn: "#1f2636", "btn-hover": "#2c3650", accent: "#589cff", accent2: "#9b6eff", border: "#2a3147", text: "#e9edf5", "text-dim": "#98a2b8", "text-faint": "#5f6680", gold: "#f5cd64" };

  const THEMES = {
    v42: [V42, FX_V42, CSS_V42],
    v41: [V41, FX_V41, CSS_V41],
    v411: [V41, patternFx([[0, 0, 0], [66, 66, 66]]), CSS_V41],
    v412: [V41, patternFx([[91, 141, 239], [64, 94, 156]]), CSS_V41],
    v413: [V41, patternFx([[91, 141, 239], [0, 0, 0]]), CSS_V41],
    v414: [V41, patternFx([[37, 41, 52], [0, 0, 0]]), CSS_V41],
    modern: [MODERN, FX_MODERN, CSS_MODERN],
    classic: [CLASSIC, FX_CLASSIC, CSS_CLASSIC],
    v2: [V2, FX_V2, { sidebar: "#141824", header: "#0f1320", card: "#1d2333", btn: "#232a3d", "btn-hover": "#2e3852", accent: "#589cff", accent2: "#4270b4", border: "#2a3147", text: "#e9edf5", "text-dim": "#98a2b8", "text-faint": "#5f667c", gold: "#f5cd64" }],
    v1: [V1, FX_V1, { sidebar: "#1c1f29", header: "#1c1f29", card: "#252a37", btn: "#3a4357", "btn-hover": "#4a566f", accent: "#78c88c", accent2: "#5aa0f0", border: "#2f3645", text: "#ffffff", "text-dim": "#c8d0e0", "text-faint": "#8a93a8", gold: "#f0d278" }],
  };

  let _theme = "v42";
  let _fx = FX_V42;

  ui.setTheme = function (name) {
    if (!THEMES[name]) name = "v42";
    const [colors, fx, css] = THEMES[name];
    _theme = name;
    _fx = fx;
    for (const k in colors) ui[k] = colors[k].slice();
    const root = document.documentElement.style;
    for (const k in css) root.setProperty("--" + k, css[k]);
    // Themenname auch ans <html> hängen - style.css schaltet damit die Sidebar um.
    document.documentElement.dataset.theme = name;
    bgCache.clear();
    celestialCache.clear();
    btnAnim.clear();
    grainCache = null;
  };
  ui.themeName = () => _theme;
  ui.isModern = () => _theme !== "classic" && _theme !== "v2" && _theme !== "v1";
  ui.fx = (key) => _fx[key];

  // Akzentfarbe je Spiel (identisch zu ui.GAME_COLORS).
  ui.GAME_COLORS = {
    SnakeGame: "#6ecd8c", PongGame: "#589cff", TicTacToeGame: "#f0a05a", BreakoutGame: "#e15f5f",
    TetrisGame: "#b07fe8", InvadersGame: "#5ad4d4", Game2048: "#f5cd64", AirHockeyGame: "#6fe0d0",
    MinesweeperGame: "#f08fb0", AsteroidsGame: "#b9c2d9", PacmanGame: "#ffd83b", FlappyGame: "#f5c518",
    DoodleGame: "#78d25a", SudokuGame: "#c77dba", FroggerGame: "#4caf6d", MemoryGame: "#8f7ef2",
    SolitaireGame: "#2fa77c", AimTrainerGame: "#e05ad4", ConnectFourGame: "#ff8f2e", TankDuelGame: "#a8b545",
    BlackjackGame: "#c8384f", TunnelRacerGame: "#35e2ff", LabyrinthGame: "#b07a4a", ReversiGame: "#3fbf8f",
    KniffelGame: "#e8b04b", WordleGame: "#6aaa64", TRexRunnerGame: "#8ea3b0", DameGame: "#d87842",
    PokerGame: "#e8c45c", ChessGame: "#c9a24b", MuehleGame: "#7fae8f", SimonGame: "#e05a7d",
    BilliardGame: "#2f9e6a", SlidingPuzzleGame: "#5ac0e0", MastermindGame: "#c86ad8",
    BubbleShooterGame: "#ff7aa8", HangmanGame: "#d89a4a", BlockJumpGame: "#8fd14f",
    LamaTowerDefenseGame: "#e2725b", MiniGolfGame: "#4fd17a", PinballGame: "#7f5af0", BowlingGame: "#4a7de0",
    CrossyRoadGame: "#e8e25a", GeometryDashGame: "#b9f23a", BattleshipGame: "#3a78b5", CasinoGame: "#d6336c",
  };

  // ------------------------------------------------------------ Farb-Helfer
  ui.hexToRgb = function (hex) {
    const v = String(hex).replace("#", "");
    return [parseInt(v.slice(0, 2), 16), parseInt(v.slice(2, 4), 16), parseInt(v.slice(4, 6), 16)];
  };
  ui.gameColor = function (id, def) {
    const h = ui.GAME_COLORS[id];
    return h ? ui.hexToRgb(h) : (def || ui.ACCENT).slice();
  };
  /** Lineare Mischung zweier RGB-Farben (f in 0..1). */
  ui.mix = function (c1, c2, f) {
    f = f < 0 ? 0 : f > 1 ? 1 : f;
    return [
      Math.floor(c1[0] + (c2[0] - c1[0]) * f),
      Math.floor(c1[1] + (c2[1] - c1[1]) * f),
      Math.floor(c1[2] + (c2[2] - c1[2]) * f),
    ];
  };
  /**
   * Farbe als CSS-String. Akzeptiert "#hex"/CSS-Strings, [r,g,b] und
   * [r,g,b,a] (a wie in pygame 0..255). Optional alpha 0..1 überschreibt.
   */
  ui.col = function (c, alpha) {
    if (c == null) return "rgba(0,0,0,0)";
    if (typeof c === "string") {
      if (alpha == null) return c;
      if (c[0] === "#" && c.length === 7) c = ui.hexToRgb(c);
      else return c;
    }
    let a = alpha != null ? alpha : c.length > 3 ? c[3] / 255 : 1;
    a = a < 0 ? 0 : a > 1 ? 1 : a;
    const r = c[0] | 0, g = c[1] | 0, b = c[2] | 0;
    return a >= 1 ? `rgb(${r},${g},${b})` : `rgba(${r},${g},${b},${a.toFixed(3)})`;
  };

  // --------------------------------------------------------------- Zeitgeber
  const T0 = performance.now();
  /** Sekunden seit Seitenstart (wie pygame.time.get_ticks()/1000). */
  ui.now = () => (performance.now() - T0) / 1000;
  /** Millisekunden seit Seitenstart (wie pygame.time.get_ticks()). */
  ui.ticks = () => performance.now() - T0;
  let _lastMs = 0;
  let _frameDt = 1 / 60;
  function tick() {
    const now = performance.now();
    if (_lastMs) _frameDt = Math.max(0.001, Math.min(0.05, (now - _lastMs) / 1000));
    _lastMs = now;
  }
  /** Sinus-Puls lo..hi für Blink-/Atem-Animationen. */
  ui.pulse = function (speed = 2.0, lo = 0.35, hi = 1.0) {
    return lo + (hi - lo) * (0.5 + 0.5 * Math.sin(ui.now() * speed));
  };

  // ------------------------------------------------------------------ Fonts
  ui.FONT_UI = 'Bahnschrift, "Segoe UI", system-ui, -apple-system, Roboto, sans-serif';
  ui.FONT_MONO = 'Consolas, Menlo, "DejaVu Sans Mono", monospace';

  const measureCtx = document.createElement("canvas").getContext("2d");

  /** Schrift-Objekt mit pygame-ähnlicher API: size(text) -> [w, h], height. */
  class Font {
    constructor(px, bold, mono) {
      this.px = px;
      this.bold = !!bold;
      this.mono = !!mono;
      this.css = (bold ? "700 " : "") + px + "px " + (mono ? ui.FONT_MONO : ui.FONT_UI);
      // Zeilenhöhe wie pygame (Font.size()[1] / render().get_height()), gemessen
      // mit pygame-ce 2.5.7: Bahnschrift ~1.1975 x px, Consolas ~1.1675 x px.
      this.height = Math.ceil(px * (mono ? 1.1675 : 1.1975));
      this._cache = new Map();
    }
    width(text) {
      text = String(text);
      let w = this._cache.get(text);
      if (w === undefined) {
        measureCtx.font = this.css;
        w = Math.ceil(measureCtx.measureText(text).width);
        if (this._cache.size > 400) this._cache.clear();
        this._cache.set(text, w);
      }
      return w;
    }
    size(text) {
      return [this.width(text), this.height];
    }
    get_height() {
      return this.height;
    }
    getHeight() {
      return this.height;
    }
  }
  ui.Font = Font;
  const fontCache = new Map();
  /** Gecachte Schrift (UI-Schrift oder Monospace). */
  ui.font = function (px, bold = false, mono = false) {
    px = Math.max(1, Math.round(px));
    const key = px + "|" + (bold ? 1 : 0) + "|" + (mono ? 1 : 0);
    let f = fontCache.get(key);
    if (!f) {
      f = new Font(px, bold, mono);
      fontCache.set(key, f);
    }
    return f;
  };

  /**
   * Zeichnet Text wie ein pygame-blit mit Rect-Anker.
   * anchor: "topleft" (Standard), "center", "midtop", "midbottom", "midleft",
   *         "midright", "topright", "bottomleft", "bottomright".
   * Gibt das belegte PG.Rect zurück.
   */
  ui.text = function (ctx, str, x, y, font, color, anchor = "topleft", alpha) {
    str = String(str);
    font = font || ui.font(22);
    const w = font.width(str);
    const h = font.height;
    let x0 = x, y0 = y;
    switch (anchor) {
      case "center": x0 = x - w / 2; y0 = y - h / 2; break;
      case "midtop": x0 = x - w / 2; break;
      case "midbottom": x0 = x - w / 2; y0 = y - h; break;
      case "midleft": y0 = y - h / 2; break;
      case "midright": x0 = x - w; y0 = y - h / 2; break;
      case "topright": x0 = x - w; break;
      case "bottomleft": y0 = y - h; break;
      case "bottomright": x0 = x - w; y0 = y - h; break;
      default: break;
    }
    ctx.font = font.css;
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.fillStyle = ui.col(color, alpha);
    ctx.fillText(str, x0, y0 + h / 2 + font.px * 0.04);
    return new PG.Rect(x0, y0, w, h);
  };
  /** Text mit vertikalem Farbverlauf (Standard: Weiß -> kühles Blau). */
  ui.gradText = function (ctx, str, x, y, font, top, bottom, anchor = "topleft") {
    const w = font.width(str);
    const h = font.height;
    const r = new PG.Rect(0, 0, w, h);
    if (anchor === "topleft") r.topleft = [x, y];
    else r[anchor] = [x, y];
    const g = ctx.createLinearGradient(0, r.y, 0, r.bottom);
    g.addColorStop(0, ui.col(top || [252, 253, 255]));
    g.addColorStop(1, ui.col(bottom || [165, 190, 235]));
    ctx.font = font.css;
    ctx.textAlign = "left";
    ctx.textBaseline = "middle";
    ctx.fillStyle = g;
    ctx.fillText(str, r.x, r.y + h / 2 + font.px * 0.04);
    return r;
  };
  /** Bricht Text auf maxWidth um -> Array von Zeilen. */
  ui.wrap = function (str, font, maxWidth) {
    const out = [];
    for (const para of String(str).split("\n")) {
      let line = "";
      for (const word of para.split(" ")) {
        const probe = line ? line + " " + word : word;
        if (font.width(probe) > maxWidth && line) {
          out.push(line);
          line = word;
        } else line = probe;
      }
      out.push(line);
    }
    return out;
  };

  // ----------------------------------------------------- pygame.draw-Nachbau
  function roundPath(ctx, x, y, w, h, r) {
    ctx.beginPath();
    if (typeof r === "number") {
      r = Math.max(0, Math.min(r, w / 2, h / 2));
      if (r <= 0) {
        ctx.rect(x, y, w, h);
        return;
      }
    }
    if (ctx.roundRect) ctx.roundRect(x, y, w, h, r);
    else {
      const rr = typeof r === "number" ? [r, r, r, r] : r;
      ctx.moveTo(x + rr[0], y);
      ctx.arcTo(x + w, y, x + w, y + h, rr[1]);
      ctx.arcTo(x + w, y + h, x, y + h, rr[2]);
      ctx.arcTo(x, y + h, x, y, rr[3]);
      ctx.arcTo(x, y, x + w, y, rr[0]);
      ctx.closePath();
    }
  }
  ui.roundPath = roundPath;

  const draw = (PG.draw = {
    /**
     * rect(ctx, color, rect, width=0, radius=0)
     * width 0 = gefüllt, sonst Rahmen dieser Stärke INNERHALB des Rechtecks.
     * radius darf eine Zahl oder [tl, tr, br, bl] sein.
     */
    rect(ctx, color, rect, width = 0, radius = 0) {
      const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
      if (r.w <= 0 || r.h <= 0) return r;
      if (width > 0) {
        const hw = width / 2;
        ctx.strokeStyle = ui.col(color);
        ctx.lineWidth = width;
        const rad = typeof radius === "number" ? Math.max(0, radius - hw) : radius.map((v) => Math.max(0, v - hw));
        roundPath(ctx, r.x + hw, r.y + hw, r.w - width, r.h - width, rad);
        ctx.stroke();
      } else {
        ctx.fillStyle = ui.col(color);
        if (!radius) ctx.fillRect(r.x, r.y, r.w, r.h);
        else {
          roundPath(ctx, r.x, r.y, r.w, r.h, radius);
          ctx.fill();
        }
      }
      return r;
    },
    circle(ctx, color, center, radius, width = 0) {
      if (radius <= 0) return;
      ctx.beginPath();
      if (width > 0) {
        ctx.arc(center[0], center[1], Math.max(0.1, radius - width / 2), 0, PG.TAU);
        ctx.strokeStyle = ui.col(color);
        ctx.lineWidth = width;
        ctx.stroke();
      } else {
        ctx.arc(center[0], center[1], radius, 0, PG.TAU);
        ctx.fillStyle = ui.col(color);
        ctx.fill();
      }
    },
    ellipse(ctx, color, rect, width = 0) {
      const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
      if (r.w <= 0 || r.h <= 0) return;
      ctx.beginPath();
      const hw = width > 0 ? width / 2 : 0;
      ctx.ellipse(r.centerx, r.centery, Math.max(0.1, r.w / 2 - hw), Math.max(0.1, r.h / 2 - hw), 0, 0, PG.TAU);
      if (width > 0) {
        ctx.strokeStyle = ui.col(color);
        ctx.lineWidth = width;
        ctx.stroke();
      } else {
        ctx.fillStyle = ui.col(color);
        ctx.fill();
      }
    },
    line(ctx, color, p1, p2, width = 1) {
      // pygame füllt Pixel (x, y) = Fläche [x, x+1): Strich auf die Pixelmitte
      // legen (sonst 1-px-Linien halb versetzt und verwaschen). Gerade Breiten
      // wachsen in pygame zur positiven Seite (x-k+1 .. x+k).
      let ox = 0.5, oy = 0.5;
      if (width > 1 && width % 2 === 0) {
        if (Math.abs(p1[0] - p2[0]) <= Math.abs(p1[1] - p2[1])) ox = 1;
        else oy = 1;
      }
      ctx.beginPath();
      ctx.moveTo(p1[0] + ox, p1[1] + oy);
      ctx.lineTo(p2[0] + ox, p2[1] + oy);
      ctx.strokeStyle = ui.col(color);
      ctx.lineWidth = width;
      ctx.lineCap = "butt";
      ctx.stroke();
    },
    lines(ctx, color, closed, points, width = 1) {
      if (!points.length) return;
      const o = width > 1 && width % 2 === 0 ? 1 : 0.5; // Pixelmitte wie bei line()
      ctx.beginPath();
      ctx.moveTo(points[0][0] + o, points[0][1] + o);
      for (let i = 1; i < points.length; i++) ctx.lineTo(points[i][0] + o, points[i][1] + o);
      if (closed) ctx.closePath();
      ctx.strokeStyle = ui.col(color);
      ctx.lineWidth = width;
      ctx.lineJoin = "round";
      ctx.lineCap = "butt";
      ctx.stroke();
    },
    polygon(ctx, color, points, width = 0) {
      if (points.length < 2) return;
      ctx.beginPath();
      ctx.moveTo(points[0][0], points[0][1]);
      for (let i = 1; i < points.length; i++) ctx.lineTo(points[i][0], points[i][1]);
      ctx.closePath();
      if (width > 0) {
        ctx.strokeStyle = ui.col(color);
        ctx.lineWidth = width;
        ctx.lineJoin = "round";
        ctx.stroke();
      } else {
        ctx.fillStyle = ui.col(color);
        ctx.fill();
      }
    },
    /** Bogen wie pygame.draw.arc (Winkel in Radiant, mathematisch positiv = gegen Uhrzeiger) */
    arc(ctx, color, rect, start, stop, width = 1) {
      const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
      if (r.w <= 0 || r.h <= 0 || width <= 0) return;
      // wie pygame: die Linienstärke wächst vom Rand nach innen
      const hw = width / 2;
      ctx.beginPath();
      ctx.ellipse(r.centerx, r.centery, Math.max(0.1, r.w / 2 - hw), Math.max(0.1, r.h / 2 - hw), 0, -stop, -start);
      ctx.strokeStyle = ui.col(color);
      ctx.lineWidth = width;
      ctx.stroke();
    },
  });

  // -------------------------------------------------------------- Hintergrund
  const bgCache = new Map();
  const STAR_COUNT = 90;
  const stars = [];
  (function () {
    const rnd = new PG.Random(20240);
    for (let i = 0; i < STAR_COUNT; i++) stars.push([rnd.random(), rnd.random(), rnd.uniform(0.25, 1.0), rnd.choice([1, 1, 1, 2])]);
  })();

  function makeCanvas(w, h) {
    const c = document.createElement("canvas");
    c.width = Math.max(1, Math.round(w));
    c.height = Math.max(1, Math.round(h));
    return c;
  }
  ui.makeCanvas = makeCanvas;

  // ------------------------------------------------------------- Film-Korn
  // 128x128-Kachel mit dreieckigem Rauschen (v4.2). Sie wird als Muster über
  // Hintergrund, Panels und Buttons gelegt und liegt in ECHTEN Gerätepixeln:
  // die Muster-Matrix hebt die Grundskalierung (pixelScale) wieder auf.
  let grainCache = null;
  function grainPattern(ctx) {
    const ps = Math.max(0.5, (PG.app && PG.app.pixelScale) || 1);
    const amp = _fx.grain || 0;
    if (grainCache && grainCache.ps === ps && grainCache.amp === amp && grainCache.ctx === ctx) return grainCache.pat;
    const S = 128;
    const tile = makeCanvas(S, S);
    const g = tile.getContext("2d");
    const img = g.createImageData(S, S);
    const d = img.data;
    const rnd = new PG.Random(0x6a1a55);
    for (let i = 0; i < S * S; i++) {
      const o = i * 4;
      const n = rnd.random() + rnd.random() - 1; // dreieckig um 0: feines, weiches Korn
      const v = n > 0 ? 255 : 0;
      d[o] = d[o + 1] = d[o + 2] = v;
      d[o + 3] = Math.abs(n) * amp * 2;
    }
    g.putImageData(img, 0, 0);
    const pat = ctx.createPattern(tile, "repeat");
    if (pat && pat.setTransform && typeof DOMMatrix !== "undefined") pat.setTransform(new DOMMatrix().scale(1 / ps));
    grainCache = { ps, amp, ctx, pat };
    return pat;
  }
  /** Legt das Film-Korn über ein Rechteck (nur v4.2, sonst wirkungslos). */
  function grainOver(ctx, x, y, w, h, alpha) {
    if (!_fx.grain) return;
    ctx.save();
    ctx.globalAlpha = alpha == null ? 1 : alpha;
    ctx.fillStyle = grainPattern(ctx);
    ctx.fillRect(x, y, w, h);
    ctx.restore();
  }

  function drawZigzag(ctx, w, h, main, alt, unit) {
    // Kachel (2*unit x unit) - siehe ui.py / CSS-Muster
    const tile = makeCanvas(unit * 2 * 4, unit * 4);
    const t = tile.getContext("2d");
    const u = unit * 4, hu = u / 2;
    t.fillStyle = ui.col(alt);
    t.fillRect(0, 0, 2 * u, u);
    const poly = (c, pts) => {
      t.fillStyle = ui.col(c);
      t.beginPath();
      t.moveTo(pts[0][0], pts[0][1]);
      for (const p of pts.slice(1)) t.lineTo(p[0], p[1]);
      t.closePath();
      t.fill();
    };
    poly(main, [[u, 0], [0, u], [2 * u, u]]);
    poly(alt, [[u, hu], [hu, u], [u + hu, u]]);
    poly(main, [[0, 0], [hu, hu], [0, hu]]);
    poly(main, [[2 * u, 0], [u + hu, hu], [2 * u, hu]]);
    const pat = ctx.createPattern(tile, "repeat");
    ctx.save();
    ctx.scale(0.25, 0.25);
    ctx.fillStyle = pat;
    ctx.fillRect(0, 0, w * 4, h * 4);
    ctx.restore();
  }

  function baseBackground(w, h) {
    const ps = Math.max(1, Math.min(2, (PG.app && PG.app.pixelScale) || 1));
    const key = w + "x" + h + "@" + ps.toFixed(2);
    let c = bgCache.get(key);
    if (c) return c;
    c = makeCanvas(w * ps, h * ps);
    const g = c.getContext("2d");
    g.scale(ps, ps);
    if (_fx.pattern) {
      drawZigzag(g, w, h, _fx.pattern[0], _fx.pattern[1], 17);
    } else {
      const lg = g.createLinearGradient(0, 0, 0, h);
      lg.addColorStop(0, ui.col(ui.BG_TOP));
      lg.addColorStop(1, ui.col(ui.BG_BOTTOM));
      g.fillStyle = lg;
      g.fillRect(0, 0, w, h);
    }
    // Weiche Vignette (elliptisch)
    g.save();
    g.translate(w / 2, h / 2);
    g.scale(w / Math.max(w, h), h / Math.max(w, h));
    const m = Math.max(w, h);
    const rg = g.createRadialGradient(0, 0, m * 0.42, 0, 0, m * 0.78);
    rg.addColorStop(0, "rgba(0,0,0,0)");
    rg.addColorStop(1, `rgba(0,0,0,${(_fx.vignette / 255).toFixed(3)})`);
    g.fillStyle = rg;
    g.fillRect(-m, -m, 2 * m, 2 * m);
    g.restore();
    if (bgCache.size > 6) bgCache.clear();
    bgCache.set(key, c);
    return c;
  }

  const AURORA = [
    [[26, 48, 95], 1.1, 0.11, 0.0, 0.22, 0.28],
    [[52, 34, 96], 0.95, 0.14, 2.1, 0.8, 0.3],
    [[16, 58, 62], 0.85, 0.08, 4.2, 0.5, 0.88],
  ];
  /**
   * Weiche Farbwolken (Aurora des Classic-Themes, Mesh-Glows von v4.2).
   * table: Zeilen [Farbe, Größe rel. max(w,h), Tempo, Phase, x, y]
   * amp:   [Weg in x, Weg in y] als Anteil der Fläche
   */
  function drawAurora(ctx, w, h, ts, table, amp) {
    const ax = amp ? amp[0] : 0.07, ay = amp ? amp[1] : 0.06;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    for (const [color, sizeF, spd, ph, fx, fy] of table || AURORA) {
      const size = sizeF * Math.max(w, h);
      const cx = (fx + ax * Math.sin(ts * spd + ph)) * w;
      const cy = (fy + ay * Math.cos(ts * spd * 0.9 + ph)) * h;
      const rg = ctx.createRadialGradient(cx, cy, 0, cx, cy, size / 2);
      rg.addColorStop(0, ui.col(color));
      rg.addColorStop(0.5, ui.col(color.map((v) => v * 0.25)));
      rg.addColorStop(1, "rgba(0,0,0,0)");
      ctx.fillStyle = rg;
      ctx.fillRect(cx - size / 2, cy - size / 2, size, size);
    }
    ctx.restore();
  }

  // Die v4.2-Wolken sind so weich, dass sie in einem Viertel der Auflösung
  // gezeichnet und hochskaliert werden können - das spart pro Frame ein
  // Vielfaches an Füllarbeit (drei bildschirmgroße Radialverläufe).
  const meshCanvas = { c: null, w: 0, h: 0 };
  function drawMesh(ctx, w, h, ts) {
    const mw = Math.max(32, Math.round(w / 4)), mh = Math.max(24, Math.round(h / 4));
    if (!meshCanvas.c || meshCanvas.w !== mw || meshCanvas.h !== mh) {
      meshCanvas.c = makeCanvas(mw, mh);
      meshCanvas.w = mw;
      meshCanvas.h = mh;
    }
    const g = meshCanvas.c.getContext("2d");
    g.clearRect(0, 0, mw, mh);
    drawAurora(g, mw, mh, ts, _fx.mesh, _fx.mesh_drift);
    ctx.save();
    ctx.globalCompositeOperation = "lighter"; // wie direkt gezeichnet: addiert sich zum Hintergrund
    ctx.drawImage(meshCanvas.c, 0, 0, w, h);
    ctx.restore();
  }

  const shoot = { active: null, next: 4000 };
  function drawShootingStar(ctx, w, h) {
    const now = ui.ticks();
    const st = shoot.active;
    if (!st) {
      if (now >= shoot.next) {
        const ang = PG.radians(PG.rand.uniform(18, 38));
        const speed = PG.rand.uniform(0.8, 1.3) * Math.max(w, 400);
        shoot.active = {
          x: PG.rand.uniform(0.15, 0.85) * w, y: PG.rand.uniform(0.05, 0.3) * h,
          vx: Math.cos(ang) * speed * PG.rand.choice([1, -1]), vy: Math.sin(ang) * speed,
          t0: now, dur: PG.rand.randint(550, 800),
        };
      }
      return;
    }
    const t = (now - st.t0) / st.dur;
    if (t >= 1) {
      shoot.active = null;
      shoot.next = now + PG.rand.randint(4500, 10000);
      return;
    }
    const sec = (t * st.dur) / 1000;
    const x = st.x + st.vx * sec, y = st.y + st.vy * sec;
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    for (let i = 0; i < 9; i++) {
      const f = (1 - t) * (1 - i / 9);
      const c = 190 * f;
      ctx.fillStyle = ui.col([c, c, Math.min(255, c * 1.2)]);
      ctx.fillRect(x - st.vx * 0.0024 * i, y - st.vy * 0.0024 * i, 2, 2);
    }
    ctx.restore();
  }

  /** Standard-Hintergrund des aktiven Themes (Verlauf/Muster, Sterne, Aurora). */
  ui.drawBackground = function (ctx, w, h, starsOn = true, aurora = null) {
    tick();
    w = w || PG.W;
    h = h || PG.H;
    ctx.drawImage(baseBackground(w, h), 0, 0, w, h);
    // Die v4.2-Farbwolken gehören zum Hintergrund und bleiben auch auf Screens
    // stehen, die die Sterne abschalten (Optionen, Wiki-artige Seiten).
    const meshOn = _fx.mesh && aurora !== false;
    if (aurora == null) aurora = starsOn;
    const ts = ui.now();
    if (meshOn) drawMesh(ctx, w, h, ts);
    else if (aurora && _fx.aurora) drawAurora(ctx, w, h, ts);
    if (starsOn && _fx.stars) {
      const bright = _fx.star_bright;
      const n = _fx.star_count ? Math.min(_fx.star_count, stars.length) : stars.length;
      for (let i = 0; i < n; i++) {
        const [x, y, depth, r] = stars[i];
        const yy = PG.mod(y - ts * 0.008 * depth, 1);
        const tw = 0.5 + 0.5 * Math.sin(ts * (0.8 + depth) + x * 40);
        const c = (40 + 70 * depth * tw) * bright;
        ctx.fillStyle = ui.col([c, c + 6, c + 18]);
        ctx.fillRect(Math.floor(x * w), Math.floor(yy * h), r, r);
      }
      if (_fx.shooting) drawShootingStar(ctx, w, h);
    }
    grainOver(ctx, 0, 0, w, h);
  };

  // -------------------------------------------- Saturn + Schwarzes Loch (v4.1)
  const celestialCache = new Map();
  function renderBlackHole(radius) {
    const key = "bh" + radius;
    let c = celestialCache.get(key);
    if (c) return c;
    const S = radius * 2;
    c = makeCanvas(S, S);
    const g = c.getContext("2d");
    const cx = S / 2;
    const sc = S / 512;
    g.translate(cx, cx);
    g.rotate(PG.radians(-26));
    g.scale(sc, sc);
    // Glut / Halo
    let rg = g.createRadialGradient(0, 0, 40, 0, 0, 210);
    rg.addColorStop(0, "rgba(255,170,70,0.95)");
    rg.addColorStop(0.3, "rgba(255,140,50,0.55)");
    rg.addColorStop(0.62, "rgba(160,70,20,0.18)");
    rg.addColorStop(1, "rgba(0,0,0,0)");
    g.fillStyle = rg;
    g.beginPath();
    g.arc(0, 0, 210, 0, PG.TAU);
    g.fill();
    // gelinster Bogen oben/unten
    for (const [oy, s] of [[-84, 1], [84, 0.6]]) {
      rg = g.createRadialGradient(0, oy, 0, 0, oy, 96);
      rg.addColorStop(0, `rgba(255,185,90,${0.75 * s})`);
      rg.addColorStop(1, "rgba(255,185,90,0)");
      g.globalCompositeOperation = "lighter";
      g.fillStyle = rg;
      g.fillRect(-100, oy - 100, 200, 200);
      g.globalCompositeOperation = "source-over";
    }
    // Schatten + Photonenring
    g.fillStyle = "rgb(16,8,6)";
    g.beginPath();
    g.arc(0, 0, 66, 0, PG.TAU);
    g.fill();
    g.strokeStyle = "rgba(255,244,214,0.92)";
    g.lineWidth = 4;
    g.beginPath();
    g.arc(0, 0, 70, 0, PG.TAU);
    g.stroke();
    g.strokeStyle = "rgba(255,200,130,0.3)";
    g.lineWidth = 9;
    g.beginPath();
    g.arc(0, 0, 74, 0, PG.TAU);
    g.stroke();
    // Akkretionsscheibe
    g.save();
    g.shadowColor = "rgba(255,190,100,0.9)";
    g.shadowBlur = 40 * sc * 4;
    const lg = g.createLinearGradient(-246, 0, 246, 0);
    lg.addColorStop(0, "rgba(255,160,70,0)");
    lg.addColorStop(0.2, "rgba(255,186,92,0.9)");
    lg.addColorStop(0.5, "rgba(255,252,234,1)");
    lg.addColorStop(0.8, "rgba(255,186,92,0.9)");
    lg.addColorStop(1, "rgba(255,160,70,0)");
    g.fillStyle = lg;
    g.beginPath();
    g.ellipse(0, 0, 246, 26, 0, 0, PG.TAU);
    g.fill();
    g.restore();
    const vg = g.createLinearGradient(0, -26, 0, 26);
    vg.addColorStop(0, "rgba(255,190,110,0)");
    vg.addColorStop(0.5, "rgba(255,255,240,0.55)");
    vg.addColorStop(1, "rgba(255,190,110,0)");
    g.fillStyle = vg;
    g.beginPath();
    g.ellipse(0, 0, 200, 14, 0, 0, PG.TAU);
    g.fill();
    // Doppler-Hotspot
    rg = g.createRadialGradient(84, -4, 0, 84, -4, 104);
    rg.addColorStop(0, "rgba(255,235,180,0.8)");
    rg.addColorStop(1, "rgba(255,235,180,0)");
    g.globalCompositeOperation = "lighter";
    g.fillStyle = rg;
    g.fillRect(-20, -110, 210, 210);
    if (celestialCache.size > 8) celestialCache.clear();
    celestialCache.set(key, c);
    return c;
  }

  function renderSaturn(radius) {
    const key = "sat" + radius;
    let c = celestialCache.get(key);
    if (c) return c;
    const S = Math.ceil(radius * 4.2);
    c = makeCanvas(S, S);
    const g = c.getContext("2d");
    const k = radius / 68;
    g.translate(S / 2, S / 2);
    g.rotate(PG.radians(-20));
    g.scale(k, k);
    const ring = (half) => {
      g.save();
      g.beginPath();
      if (half === "back") g.rect(-200, -200, 400, 200);
      else g.rect(-200, 0, 400, 200);
      g.clip();
      const bands = [[124, 38, "rgba(205,183,148,0.69)"], [106, 32, "rgba(230,208,172,0.82)"], [84, 24, "rgba(176,155,124,0.47)"]];
      for (const [rx, ry, col] of bands) {
        g.strokeStyle = col;
        g.lineWidth = rx === 124 ? 11 : rx === 106 ? 20 : 9;
        g.beginPath();
        g.ellipse(0, 0, rx - g.lineWidth / 2, ry - g.lineWidth / 4, 0, 0, PG.TAU);
        g.stroke();
      }
      g.restore();
    };
    ring("back");
    // Planet mit Bändern
    g.save();
    g.beginPath();
    g.arc(0, 0, 68, 0, PG.TAU);
    g.clip();
    const tones = ["#e8d2aa", "#d4ba8e", "#e2caa2", "#ceb48a", "#e4cca4", "#d8be94"];
    const bh = 136 / tones.length;
    tones.forEach((tone, i) => {
      g.fillStyle = tone;
      g.fillRect(-68, -68 + i * bh, 136, bh + 1);
    });
    const sh = g.createRadialGradient(-20, -22, 10, 10, 10, 100);
    sh.addColorStop(0, "rgba(18,14,24,0)");
    sh.addColorStop(0.6, "rgba(18,14,24,0.25)");
    sh.addColorStop(1, "rgba(18,14,24,0.7)");
    g.fillStyle = sh;
    g.fillRect(-70, -70, 140, 140);
    g.restore();
    ring("front");
    if (celestialCache.size > 8) celestialCache.clear();
    celestialCache.set(key, c);
    return c;
  }

  ui.drawBlackHole = function (ctx, center, radius) {
    radius = Math.max(8, Math.floor(radius / 4) * 4);
    const img = renderBlackHole(radius);
    ctx.drawImage(img, center[0] - img.width / 2, center[1] - img.height / 2);
  };
  ui.drawSaturn = function (ctx, center, radius) {
    radius = Math.max(6, Math.floor(radius / 4) * 4);
    const img = renderSaturn(radius);
    ctx.drawImage(img, center[0] - img.width / 2, center[1] - img.height / 2);
  };

  // --------------------------------------------------------- Panels & Buttons
  /** Abgerundetes Panel mit Rand und weichem Schlagschatten. */
  ui.drawPanel = function (ctx, rect, opts = {}) {
    const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
    const color = opts.color != null ? opts.color : ui.PANEL;
    const border = opts.border != null ? opts.border : ui.BORDER;
    const radius = opts.radius != null ? opts.radius : _fx.panel_radius;
    if (_fx.style === "v42") {
      // UI v4.2: Milchglas - zwei weiche Schattenlagen, Glasfüllung mit
      // diagonalem Schimmer und Korn, oben eine feine Lichtkante.
      if (opts.shadow !== false) {
        draw.rect(ctx, [0, 0, 0, Math.round(_fx.shadow_alpha * 0.45)], [r.x - 2, r.y + 7, r.w + 4, r.h], 0, radius + 6);
        draw.rect(ctx, [0, 0, 0, _fx.shadow_alpha], [r.x + 1, r.y + 3, r.w, r.h], 0, radius + 2);
      }
      ctx.save();
      roundPath(ctx, r.x, r.y, r.w, r.h, radius);
      ctx.clip();
      ctx.fillStyle = ui.col(color, _fx.glass_alpha / 255);
      ctx.fillRect(r.x, r.y, r.w, r.h);
      const sg = ctx.createLinearGradient(r.x, r.y, r.right, r.bottom);
      sg.addColorStop(0, "rgba(255,255,255,0.07)");
      sg.addColorStop(0.45, "rgba(255,255,255,0.02)");
      sg.addColorStop(1, "rgba(255,255,255,0)");
      ctx.fillStyle = sg;
      ctx.fillRect(r.x, r.y, r.w, r.h);
      grainOver(ctx, r.x, r.y, r.w, r.h, 0.6);
      ctx.restore();
      const eg = ctx.createLinearGradient(r.x, 0, r.right, 0);
      eg.addColorStop(0, "rgba(255,255,255,0)");
      eg.addColorStop(0.5, "rgba(255,255,255,0.22)");
      eg.addColorStop(1, "rgba(255,255,255,0)");
      ctx.fillStyle = eg;
      ctx.fillRect(r.x + radius * 0.6, r.y + 1, Math.max(0, r.w - radius * 1.2), 1);
      draw.rect(ctx, typeof border === "string" || border.length > 3 ? border : [border[0], border[1], border[2], 200], r, 1, radius);
      if (opts.accentTop) draw.rect(ctx, opts.accentTop, [r.x + radius, r.y, r.w - 2 * radius, 2]);
      return r;
    }
    if (_fx.style === "v1") {
      // UI v1: flache, abgerundete Fläche ohne Rand und Schatten
      draw.rect(ctx, color, r, 0, radius);
      if (opts.accentTop) draw.rect(ctx, opts.accentTop, [r.x + radius, r.y, r.w - 2 * radius, 2]);
      return r;
    }
    if (opts.shadow !== false) {
      draw.rect(ctx, [0, 0, 0, _fx.shadow_alpha], [r.x + 2, r.y + 4, r.w, r.h], 0, radius + 2);
    }
    draw.rect(ctx, color, r, 0, radius);
    draw.rect(ctx, border, r, 1, radius);
    if (opts.accentTop) draw.rect(ctx, opts.accentTop, [r.x + radius, r.y, r.w - 2 * radius, 2]);
    return r;
  };

  const btnAnim = new Map();
  function btnProgress(key, selected) {
    let v = btnAnim.get(key) || 0;
    const target = selected ? 1 : 0;
    v += (target - v) * Math.min(1, _frameDt * 12);
    if (Math.abs(v - target) < 0.01) v = target;
    if (btnAnim.size > 96) btnAnim.clear();
    btnAnim.set(key, v);
    return v;
  }

  function buttonLabel(ctx, r, label, fnt, textCol, sub, subFont, subCol) {
    if (sub && subFont) {
      const total = fnt.height + 2 + subFont.height;
      const y0 = r.centery - total / 2;
      ui.text(ctx, label, r.centerx, y0, fnt, textCol, "midtop");
      ui.text(ctx, sub, r.centerx, y0 + fnt.height + 2, subFont, subCol, "midtop");
    } else {
      ui.text(ctx, label, r.centerx, r.centery, fnt, textCol, "center");
    }
  }

  /**
   * Menü-Button im aktiven Theme.
   * opts: {accent, sub, subFont}
   */
  ui.drawButton = function (ctx, rect, label, fnt, selected = false, opts = {}) {
    const ac = opts.accent || ui.ACCENT;
    const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
    fnt = fnt || ui.font(19);
    const v = btnProgress(r.x + "," + r.y + "," + r.w + "," + r.h, selected);
    const radius = _fx.btn_radius;
    if (_fx.style === "v42") {
      // UI v4.2: deckende Füllung (Buttons stehen oft über Spielfeldern),
      // darüber Glas-Schimmer + Korn, Akzent-Glow und wachsende Verlaufslinie.
      const fill = ui.mix(ui.BTN, ui.mix(ui.BTN_SEL, ac, 0.16), v);
      if (v > 0.02) {
        ctx.save();
        ctx.shadowColor = ui.col(ac, 0.42 * v);
        ctx.shadowBlur = 16 * v;
        draw.rect(ctx, fill, r, 0, radius);
        ctx.restore();
      }
      draw.rect(ctx, fill, r, 0, radius);
      ctx.save();
      roundPath(ctx, r.x, r.y, r.w, r.h, radius);
      ctx.clip();
      const sg = ctx.createLinearGradient(r.x, r.y, r.right, r.bottom);
      sg.addColorStop(0, "rgba(255,255,255,0.07)");
      sg.addColorStop(0.45, "rgba(255,255,255,0.02)");
      sg.addColorStop(1, "rgba(255,255,255,0)");
      ctx.fillStyle = sg;
      ctx.fillRect(r.x, r.y, r.w, r.h);
      grainOver(ctx, r.x, r.y, r.w, r.h, 0.6);
      ctx.restore();
      draw.rect(ctx, ui.mix(ui.BORDER, ui.mix(ac, [255, 255, 255], 0.2), 0.8 * v), r, 1, radius);
      if (v > 0.05) ui.drawGradLine(ctx, r.centerx, r.bottom - 6, Math.max(8, (r.w - 26) * v), 2, opts.accent || null);
      buttonLabel(ctx, r, label, fnt, ui.mix(ui.TEXT_DIM, ui.TEXT, v), opts.sub, opts.subFont, ui.mix(ui.TEXT_FAINT, ui.TEXT_DIM, v));
      return r;
    }
    if (ui.isModern()) {
      const fill = ui.mix(ui.BTN, ui.mix(ui.PANEL_LIGHT, ac, 0.1), v);
      draw.rect(ctx, fill, r, 0, radius);
      draw.rect(ctx, ui.mix(ui.BORDER, ac, 0.85 * v), r, 1, radius);
      if (v > 0.05) {
        const bh = Math.max(6, (r.h - 14) * v);
        draw.rect(ctx, ac, [r.x + 7, r.centery - bh / 2, 3, bh], 0, 2);
      }
      buttonLabel(ctx, r, label, fnt, ui.mix(ui.TEXT_DIM, ui.TEXT, v), opts.sub, opts.subFont, ui.mix(ui.TEXT_FAINT, ui.TEXT_DIM, v));
      return r;
    }
    if (_fx.style === "v1") {
      // UI v1: flacher Button, Auswahl nur über die Farbe, Text immer hell
      draw.rect(ctx, selected ? ui.BTN_SEL : ui.BTN, r, 0, radius);
      buttonLabel(ctx, r, label, fnt, ui.TEXT, opts.sub, opts.subFont, ui.TEXT_DIM);
      return r;
    }
    if (_fx.style === "v2") {
      // UI v2: harter Wechsel ohne Animation (Glow, Akzentrahmen, Balken, Pfeil)
      if (selected) {
        draw.rect(ctx, [ac[0], ac[1], ac[2], 45], [r.x - 10, r.y - 10, r.w + 20, r.h + 20], 0, 16);
        draw.rect(ctx, ui.BTN_SEL, r, 0, radius);
        draw.rect(ctx, ac, r, 2, radius);
        draw.rect(ctx, ac, [r.x + 6, r.y + 8, 4, r.h - 16], 0, 2);
        const cx = r.right - 18, cy = r.centery;
        draw.polygon(ctx, ui.TEXT, [[cx - 4, cy - 6], [cx + 4, cy], [cx - 4, cy + 6]]);
      } else {
        draw.rect(ctx, ui.BTN, r, 0, radius);
        draw.rect(ctx, ui.BORDER, r, 1, radius);
      }
      buttonLabel(ctx, r, label, fnt, selected ? ui.TEXT : ui.TEXT_DIM, opts.sub, opts.subFont, selected ? ui.TEXT_DIM : ui.TEXT_FAINT);
      return r;
    }
    if (v > 0.02 && _fx.btn_glow) {
      const a = 52 * v * (0.7 + 0.3 * ui.pulse(2.2));
      draw.rect(ctx, [ac[0], ac[1], ac[2], a], [r.x - 11, r.y - 11, r.w + 22, r.h + 22], 0, 16);
    }
    const fill = ui.mix(ui.BTN, ui.mix(ui.BTN_SEL, ac, 0.15), v);
    draw.rect(ctx, fill, r, 0, radius);
    draw.rect(ctx, ui.mix(fill, [255, 255, 255], 0.05 + 0.1 * v), [r.x + 10, r.y + 1, r.w - 20, 1]);
    draw.rect(ctx, ui.mix(ui.BORDER, ac, v), r, v > 0.5 ? 2 : 1, radius);
    if (v > 0.05) {
      const bh = Math.max(4, (r.h - 16) * v);
      draw.rect(ctx, ac, [r.x + 6, r.centery - bh / 2, 4, bh], 0, 2);
      if (_fx.btn_arrow) {
        const cx = r.right - 18 + (1 - v) * 10 + 2 * ui.pulse(2.6, 0);
        const cy = r.centery;
        draw.polygon(ctx, ui.mix(fill, ui.TEXT, v), [[cx - 4, cy - 6], [cx + 4, cy], [cx - 4, cy + 6]]);
      }
    }
    buttonLabel(ctx, r, label, fnt, ui.mix(ui.TEXT_DIM, ui.TEXT, v), opts.sub, opts.subFont, ui.mix(ui.TEXT_FAINT, ui.TEXT_DIM, v));
    return r;
  };

  /**
   * Zentrierter Titel. opts: {subtitle, y=52, big, small, accent}
   * Gibt die y-Position der Akzentlinie zurück.
   */
  ui.drawTitle = function (ctx, width, title, opts = {}) {
    const y = opts.y != null ? opts.y : 52;
    const big = opts.big || ui.font(40, true);
    const ac = opts.accent || ui.ACCENT;
    const cx = width / 2;
    const tw = big.width(title), th = big.height;
    if (_fx.style === "v42") {
      // UI v4.2: heller Verlaufstitel mit dunklem Schatten, darunter die
      // dreifarbige Verlaufslinie (Maße wie Modern - kein Layout verrutscht).
      ui.text(ctx, title, cx + 2, y + 2, big, [0, 0, 0, 120], "center");
      ui.gradText(ctx, title, cx, y, big, [250, 251, 255], [200, 208, 244], "center");
      const lw = Math.max(64, Math.min(0.6 * tw, 240));
      const ly = y + th / 2 + 10;
      ui.drawGradLine(ctx, cx, ly, lw, 3, opts.accent || null, true);
      if (opts.subtitle) ui.text(ctx, opts.subtitle, cx, ly + 24, opts.small || ui.font(17), ui.TEXT_DIM, "center");
      return ly;
    }
    if (ui.isModern()) {
      ui.text(ctx, title, cx, y, big, ui.TEXT, "center");
      const lw = Math.max(56, Math.min(tw / 2, 160));
      const ly = y + th / 2 + 10;
      draw.rect(ctx, ac, [cx - lw / 2, ly, lw, 3], 0, 2);
      if (opts.subtitle) ui.text(ctx, opts.subtitle, cx, ly + 24, opts.small || ui.font(17), ui.TEXT_DIM, "center");
      return ly;
    }
    if (_fx.style === "v1") {
      // UI v1: schlichter Titel, Untertitel 42px darunter, keine Linie
      ui.text(ctx, title, cx, y, big, ui.TEXT, "center");
      if (opts.subtitle) ui.text(ctx, opts.subtitle, cx, y + 42, opts.small || ui.font(17), ui.TEXT_DIM, "center");
      return y + th / 2 + 8;
    }
    if (_fx.style === "v2") {
      // UI v2: Schatten + Titel in Textfarbe, Akzentlinie + weicher Zweitstrich
      ui.text(ctx, title, cx + 2, y + 3, big, [0, 0, 0, 140], "center");
      ui.text(ctx, title, cx, y, big, ui.TEXT, "center");
      const lw = Math.min(tw + 20, width - 80);
      const ly = y + th / 2 + 8;
      draw.rect(ctx, ac, [cx - lw / 2, ly, lw, 3], 0, 2);
      draw.rect(ctx, ui.ACCENT_SOFT, [cx - lw / 6, ly + 5, lw / 3, 2], 0, 2);
      if (opts.subtitle) ui.text(ctx, opts.subtitle, cx, ly + 26, opts.small || ui.font(17), ui.TEXT_DIM, "center");
      return ly;
    }
    ctx.save();
    ctx.shadowColor = ui.col(ac.map((c) => c * 0.8));
    ctx.shadowBlur = 18;
    ui.text(ctx, title, cx + 2, y + 3, big, [0, 0, 0, 140], "center");
    ctx.restore();
    ui.gradText(ctx, title, cx, y, big, null, null, "center");
    const lw = Math.min(tw + 20, width - 80);
    const ly = y + th / 2 + 8;
    draw.rect(ctx, ac, [cx - lw / 2, ly, lw, 3], 0, 2);
    draw.rect(ctx, ui.mix(ac, ui.ACCENT2, 0.6), [cx - lw / 6, ly + 5, lw / 3, 2], 0, 2);
    if (opts.subtitle) ui.text(ctx, opts.subtitle, cx, ly + 26, opts.small || ui.font(17), ui.TEXT_DIM, "center");
    return ly;
  };

  /** Fußzeile: dezente Trennlinie + Hinweistext unten. */
  ui.drawFooter = function (ctx, width, height, str, fnt) {
    fnt = fnt || ui.font(14);
    if (_fx.style === "v42") {
      // UI v4.2: Haarlinie, die zu beiden Seiten ausläuft
      const y42 = height - 22;
      const x0 = width / 6, x1 = width - width / 6;
      const lg = ctx.createLinearGradient(x0, 0, x1, 0);
      lg.addColorStop(0, ui.col(ui.BORDER_LIGHT, 0));
      lg.addColorStop(0.5, ui.col(ui.BORDER_LIGHT, 0.75));
      lg.addColorStop(1, ui.col(ui.BORDER_LIGHT, 0));
      ctx.fillStyle = lg;
      ctx.fillRect(x0, y42 - 12, x1 - x0, 1);
      ui.text(ctx, str, width / 2, y42, fnt, ui.TEXT_FAINT, "center");
      return;
    }
    if (_fx.style === "v1") {
      // UI v1: nur der Hinweistext, ohne Trennlinie
      ui.text(ctx, str, width / 2, height - 24, fnt, ui.TEXT_DIM, "center");
      return;
    }
    const y = height - 22;
    draw.line(ctx, ui.BORDER, [width / 6, y - 12], [width - width / 6, y - 12]);
    ui.text(ctx, str, width / 2, y, fnt, ui.TEXT_FAINT, "center");
  };

  /**
   * Waagerechte Verlaufslinie um cx (v4.2). Ohne accent die drei Theme-Farben,
   * mit accent eine Variante davon. Außerhalb von v4.2: schlichter Akzentbalken.
   */
  ui.drawGradLine = function (ctx, cx, y, w, h = 3, accent = null, glow = false) {
    if (w <= 0 || h <= 0) return;
    if (_fx.style !== "v42" || !_fx.grad3) {
      draw.rect(ctx, accent || ui.ACCENT, [cx - w / 2, y, w, h], 0, Math.min(2, h / 2));
      return;
    }
    const g3 = _fx.grad3;
    const stops = accent ? [ui.mix(accent, [255, 255, 255], 0.25), accent, ui.mix(accent, g3[2], 0.45)] : g3;
    const lg = ctx.createLinearGradient(cx - w / 2, 0, cx + w / 2, 0);
    lg.addColorStop(0, ui.col(stops[0]));
    lg.addColorStop(0.5, ui.col(stops[1]));
    lg.addColorStop(1, ui.col(stops[2]));
    ctx.save();
    if (glow) {
      ctx.shadowColor = ui.col(stops[1], 0.5);
      ctx.shadowBlur = 9;
    }
    ctx.fillStyle = lg;
    roundPath(ctx, cx - w / 2, y, w, h, h / 2);
    ctx.fill();
    ctx.restore();
  };

  /** Weicher Lichtschein hinter Logo/Symbolen (nur v4.2, sonst wirkungslos). */
  ui.drawHalo = function (ctx, center, size, color) {
    if (_fx.style !== "v42" || !_fx.logo_halo || size <= 0) return;
    const c = color || ui.ACCENT;
    const rg = ctx.createRadialGradient(center[0], center[1], 0, center[0], center[1], size / 2);
    rg.addColorStop(0, ui.col(c, 0.22));
    rg.addColorStop(0.45, ui.col(c, 0.08));
    rg.addColorStop(1, ui.col(c, 0));
    ctx.save();
    ctx.globalCompositeOperation = "lighter";
    ctx.fillStyle = rg;
    ctx.fillRect(center[0] - size / 2, center[1] - size / 2, size, size);
    ctx.restore();
  };

  /**
   * Standard-Endstand-Box (wie _draw_overlay vieler Spiele):
   * transluzentes Panel mit Titel in 'color' und pulsierendem Hinweis.
   */
  ui.drawOverlayBox = function (ctx, w, h, title, color, hint, opts = {}) {
    const big = opts.big || ui.font(Math.max(32, h / 10), true);
    const small = opts.small || ui.font(Math.max(16, h / 26));
    const lines = Array.isArray(hint) ? hint : hint ? [hint] : [];
    const bw = Math.max(big.width(title), ...lines.map((l) => small.width(l)), 0) + 80;
    const bh = big.height + lines.length * (small.height + 6) + 50;
    const r = new PG.Rect(0, 0, bw, bh);
    r.center = [w / 2, (opts.cy != null ? opts.cy : h / 2)];
    draw.rect(ctx, [ui.PANEL[0], ui.PANEL[1], ui.PANEL[2], 235], r, 0, 16);
    draw.rect(ctx, color, r, 2, 16);
    ui.text(ctx, title, r.centerx, r.y + 20, big, color, "midtop");
    const hintCol = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0));
    lines.forEach((l, i) => ui.text(ctx, l, r.centerx, r.y + 20 + big.height + 12 + i * (small.height + 6), small, i === lines.length - 1 ? hintCol : ui.TEXT, "midtop"));
    return r;
  };

  // --------------------------------------------------------------- Partikel
  const particles = [];
  const MAX_PARTICLES = 420;

  ui.spawnBurst = function (x, y, color, n = 18, speed = 260) {
    if (!_fx.sparks) return;
    color = color || ui.ACCENT;
    for (let i = 0; i < n && particles.length < MAX_PARTICLES; i++) {
      const ang = PG.rand.uniform(0, PG.TAU);
      const sp = speed * PG.rand.uniform(0.25, 1);
      particles.push({ kind: "spark", x, y, vx: Math.cos(ang) * sp, vy: Math.sin(ang) * sp - 40, age: 0, life: PG.rand.uniform(0.35, 0.7), color, size: PG.rand.uniform(1.5, 3.2) });
    }
  };
  ui.spawnConfetti = function (w, h, n = 90) {
    const cols = [ui.ACCENT, ui.ACCENT2, ui.GREEN, ui.GOLD, ui.RED, [240, 240, 250]];
    if (ui.isModern()) n = Math.min(n, 60);
    for (let i = 0; i < n && particles.length < MAX_PARTICLES; i++) {
      particles.push({ kind: "confetti", x: PG.rand.uniform(0, w), y: PG.rand.uniform(-h * 0.25, 0), vx: PG.rand.uniform(-30, 30), vy: PG.rand.uniform(90, 220), age: 0, life: PG.rand.uniform(2, 3.4), color: PG.rand.choice(cols), spin: PG.rand.uniform(4, 9), phase: PG.rand.uniform(0, PG.TAU) });
    }
  };
  function drawParticles(ctx, w, h, dt) {
    if (!particles.length) return;
    let j = 0;
    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];
      p.age += dt;
      if (p.age >= p.life) continue;
      const fade = 1 - p.age / p.life;
      if (p.kind === "spark") {
        p.vy += 420 * dt;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        const s = Math.max(1, p.size * fade);
        ctx.globalCompositeOperation = "lighter";
        ctx.fillStyle = ui.col(ui.mix([0, 0, 0], p.color, fade));
        ctx.fillRect(p.x, p.y, s, s);
        ctx.globalCompositeOperation = "source-over";
      } else {
        p.x += (p.vx + 34 * Math.sin(p.age * 3 + p.phase)) * dt;
        p.y += p.vy * dt;
        if (p.y < h + 8) {
          const bw = 2 + 4 * Math.abs(Math.sin(p.age * p.spin + p.phase));
          ctx.fillStyle = ui.col(ui.mix(ui.BG_BOTTOM, p.color, Math.min(1, fade * 1.8)));
          ctx.fillRect(p.x, p.y, bw, 5);
        }
      }
      particles[j++] = p;
    }
    particles.length = j;
  }

  const trans = { start: null, dur: 0.35 };
  ui.beginTransition = function (dur) {
    trans.dur = dur != null ? dur : _fx.trans_dur;
    // UI v2 kennt keine Screen-Übergänge (trans_dur 0)
    trans.start = trans.dur > 0 ? ui.ticks() : null;
  };
  function drawTransition(ctx, w, h) {
    if (trans.start == null) return;
    const t = (ui.ticks() - trans.start) / 1000 / trans.dur;
    if (t >= 1) {
      trans.start = null;
      return;
    }
    ctx.fillStyle = `rgba(6,8,16,${((210 * Math.pow(1 - t, 1.5)) / 255).toFixed(3)})`;
    ctx.fillRect(0, 0, w, h);
    if (!_fx.scanline) return;
    const y = h * t;
    const a = (90 * (1 - t)) / 255;
    ctx.fillStyle = ui.col(ui.ACCENT, a);
    ctx.fillRect(0, y - 3, w, 7);
    ctx.fillStyle = ui.col(ui.mix(ui.ACCENT, [255, 255, 255], 0.5), Math.min(1, a * 2));
    ctx.fillRect(0, y, w, 1);
  }
  /** Globale Effekte (Partikel + Übergang) - ruft die App 1x pro Frame. */
  ui.drawFx = function (ctx, w, h, dt) {
    drawParticles(ctx, w, h, dt);
    drawTransition(ctx, w, h);
  };

  // ---------------------------------------------------------------- TextInput
  /** Einzeiliges Eingabefeld mit Schreibmarke (wie ui.TextInput). */
  class TextInput {
    constructor(text = "", maxlen = 28, charset = null, placeholder = "") {
      this.maxlen = maxlen;
      this.charset = charset;
      this.placeholder = placeholder;
      this.caret = 0;
      this.text = "";
      this.setText(text);
    }
    setText(text) {
      text = text == null ? "" : String(text);
      this.text = Array.from(text).filter((c) => this._ok(c)).join("").slice(0, this.maxlen);
      this.caret = this.text.length;
    }
    _ok(ch) {
      if (!ch || ch.charCodeAt(0) < 32 || ch === "\x7f") return false;
      return this.charset == null || this.charset.includes(ch.toLowerCase());
    }
    insert(ch) {
      if (this.text.length >= this.maxlen || !this._ok(ch)) return false;
      if (this.charset != null) ch = ch.toLowerCase();
      this.text = this.text.slice(0, this.caret) + ch + this.text.slice(this.caret);
      this.caret++;
      return true;
    }
    /** KEYDOWN-Event verarbeiten; true = verbraucht */
    handle(ev) {
      if (ev.kind !== "keydown") return false;
      const key = ev.key;
      if (TextInput.PASS_THROUGH.includes(key)) return false;
      if (key === "BackSpace") {
        if (this.caret > 0) {
          this.text = this.text.slice(0, this.caret - 1) + this.text.slice(this.caret);
          this.caret--;
        }
        return true;
      }
      if (key === "Delete") {
        this.text = this.text.slice(0, this.caret) + this.text.slice(this.caret + 1);
        return true;
      }
      if (key === "Left") { this.caret = Math.max(0, this.caret - 1); return true; }
      if (key === "Right") { this.caret = Math.min(this.text.length, this.caret + 1); return true; }
      if (key === "Home") { this.caret = 0; return true; }
      if (key === "End") { this.caret = this.text.length; return true; }
      if (ev.char) return this.insert(ev.char);
      return false;
    }
    draw(ctx, rect, fnt, focused = false, invalid = false) {
      const r = rect instanceof PG.Rect ? rect : new PG.Rect(rect);
      const border = invalid ? ui.RED : focused ? ui.ACCENT : ui.BORDER;
      draw.rect(ctx, focused ? ui.PANEL_LIGHT : ui.PANEL, r, 0, 7);
      draw.rect(ctx, border, r, focused || invalid ? 2 : 1, 7);
      const pad = 8;
      const inner = r.w - 2 * pad;
      const show = this.text || this.placeholder;
      let off = 0;
      if (this.text) {
        const upto = fnt.width(this.text.slice(0, this.caret));
        if (upto > inner) off = upto - inner;
      }
      ctx.save();
      ctx.beginPath();
      ctx.rect(r.x + 2, r.y + 2, r.w - 4, r.h - 4);
      ctx.clip();
      ui.text(ctx, show, r.x + pad - off, r.centery, fnt, this.text ? ui.TEXT : ui.TEXT_FAINT, "midleft");
      if (focused && Math.floor(ui.pulse(3.0, 0, 1.99)) === 0) {
        const cx = r.x + pad - off + fnt.width(this.text.slice(0, this.caret));
        draw.rect(ctx, ui.ACCENT, [cx, r.y + 5, 2, r.h - 10]);
      }
      ctx.restore();
      return r;
    }
  }
  TextInput.ID_CHARS = "abcdefghijklmnopqrstuvwxyz0123456789-_";
  TextInput.PASS_THROUGH = ["Tab", "Return", "KP_Enter", "Escape", "Up", "Down"];
  ui.TextInput = TextInput;

  ui.setTheme(PG.settings.data.theme);
})();
