/*
 * geodash_edit.js - Eigene Geometry-Dash-Level (Port von games/geodash_edit.py)
 * ============================================================================
 *   LevelList    LEVELS-Reiter: Neu, Bearbeiten, Spielen, Löschen, Teilen
 *                (Download als .lamapgzlevel) und Importieren (Dateiwahl).
 *   LevelEditor  Raster-Leinwand mit Scrollen, Palette in sechs Gruppen,
 *                Auswählen/Setzen/Löschen/Drehen, Undo/Redo, Level-
 *                Einstellungen, Test-Spielen ab Start oder ab Kamera.
 *
 * "Verifiziert" wird ein Level erst, wenn der Ersteller es im Test vom Start
 * weg im Normalmodus geschafft hat (Prüfsumme des Inhalts, gdCore.contentHash).
 * Gespeichert wird über PG.ugc (ugc.js, Art "geodash").
 */
(function () {
  "use strict";

  const { ui, draw, t } = PG;
  const core = PG.gdCore;
  const gdraw = PG.gdDraw;
  const GAME = "geodash";
  const ugc = PG.ugc;
  const swear = PG.swear;

  const R = (x, y, w, h) => new PG.Rect(Math.trunc(x), Math.trunc(y), Math.trunc(w), Math.trunc(h));

  function fit(font, text, w, dots = "...") {
    if (font.width(text) <= w) return text;
    let s = text;
    while (s.length > 1 && font.width(s + dots) > w) s = s.slice(0, -1);
    return s + dots;
  }

  const TOOLS = ["select", "erase"];
  const GROUPS = [
    ["blocks", ["block", "half"]],
    ["hazards", ["spike", "spike_s", "pit"]],
    ["orbs", ["pad_y", "pad_p", "pad_b", "orb_y", "orb_p", "orb_b"]],
    ["portals", ["p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm", "g_flip"]],
    ["speed", ["s_slow", "s_norm", "s_fast", "s_vfast"]],
    ["extras", ["coin", "color"]],
  ];
  const GROUP_ITEMS = Object.fromEntries(GROUPS);
  const GROUP_KEYS = GROUPS.map((g) => g[0]);
  const GROUP_ICON = { blocks: "block", hazards: "spike", orbs: "orb_y", portals: "p_ship", speed: "s_fast", extras: "coin" };
  const PRESETS = [[40, 110, 255], [20, 70, 200], [170, 60, 255], [255, 60, 150], [230, 90, 40], [250, 200, 40], [40, 170, 110], [0, 170, 200], [140, 0, 40], [30, 30, 50], [120, 120, 140], [255, 255, 255]];
  const PARAM_LABELS = { p_ship: ["gd.ed.par.height"], p_ball: ["gd.ed.par.height"], p_ufo: ["gd.ed.par.height"], p_wave: ["gd.ed.par.height"], color: ["gd.ed.par.target", "R", "G", "B", "gd.ed.par.dur"] };
  const UNDO_MAX = 60;
  const ROWS_VISIBLE = 11;
  const SETTINGS_ROWS = ["speed", "mode", "music", "bpm", "bg", "ground"];
  const TEXT_BUTTONS = ["test", "test_here"];
  const ICON_BUTTONS = ["undo", "redo", "rotate", "delete", "grid", "settings"];
  const PORTAL_KINDS = ["p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm", "g_flip", "s_slow", "s_norm", "s_fast", "s_vfast"];

  const layerOf = (kind) => (kind === "pit" ? 1 : kind === "color" ? 2 : 0);
  const defaultParam = (kind) => (core.PARAMS[kind] || []).map((p) => p[0]);
  const rotatable = (kind) => core.ROTATE_ALL.includes(kind) || core.ROTATE_FLIP.includes(kind);

  function newLevel(name = "", id = "", author = null) {
    const now = ugc.stamp();
    return core.normalizeLevel({
      v: 1, id, name, author: author === null ? ugc.lastAuthor() : author, created: now, edited: now,
      speed: 1, mode: core.CUBE, bg: core.DEFAULT_BG.slice(), ground: core.DEFAULT_GROUND.slice(), music: "drive", objects: [],
    });
  }

  function isVerified(m) {
    return !!(m && typeof m.verified === "string" && m.verified && m.verified === core.contentHash(m));
  }

  function btn(ctx, rc, text, fnt, on = false, accent = null, enabled = true) {
    draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
    draw.rect(ctx, on ? accent || ui.ACCENT : ui.BORDER, rc, on ? 2 : 1, 7);
    ui.text(ctx, fit(fnt, text, rc.w - 8), rc.centerx, rc.centery, fnt, enabled || on ? ui.TEXT : ui.TEXT_FAINT, "center");
  }

  function icon(ctx, rc, key, col) {
    const cx = rc.centerx, cy = rc.centery;
    const r = Math.max(4, Math.min(rc.w, rc.h) >> 2);
    if (key === "undo" || key === "redo") {
      const sign = key === "undo" ? -1 : 1;
      draw.arc(ctx, col, [cx - r, cy + r / 3 - r, r * 2, r * 2], 0.3, 2.9, 2);
      const tip = [cx - sign * r, cy + r / 3];
      draw.polygon(ctx, col, [[tip[0] - r * 0.5, tip[1] - 1], [tip[0] + r * 0.5, tip[1] - 1], [tip[0], tip[1] + r * 0.6]]);
    } else if (key === "rotate") {
      draw.arc(ctx, col, [cx - r, cy - r, r * 2, r * 2], -0.4, 4.2, 2);
      draw.polygon(ctx, col, [[cx + r, cy - r * 0.2], [cx + r * 0.3, cy - r * 0.2], [cx + r * 0.9, cy + r * 0.6]]);
    } else if (key === "delete") {
      draw.line(ctx, col, [cx - r, cy - r], [cx + r, cy + r], 3);
      draw.line(ctx, col, [cx + r, cy - r], [cx - r, cy + r], 3);
    } else if (key === "grid") {
      draw.rect(ctx, col, [cx - r, cy - r, 2 * r, 2 * r], 1);
      draw.line(ctx, col, [cx, cy - r], [cx, cy + r]);
      draw.line(ctx, col, [cx - r, cy], [cx + r, cy]);
    } else if (key === "settings") {
      draw.circle(ctx, col, [cx, cy], r, 2);
      for (let k = 0; k < 6; k++) {
        const a = (k * Math.PI) / 3;
        draw.line(ctx, col, [cx + Math.cos(a) * r, cy + Math.sin(a) * r], [cx + Math.cos(a) * r * 1.55, cy + Math.sin(a) * r * 1.55], 3);
      }
      draw.circle(ctx, col, [cx, cy], Math.max(1, r / 3));
    } else if (key === "select") {
      draw.polygon(ctx, col, [[cx - r, cy - r], [cx + r, cy], [cx, cy + r * 0.3], [cx + r * 0.3, cy + r]], 2);
    } else if (key === "erase") {
      draw.rect(ctx, col, [cx - r, cy - r * 0.75, 2 * r, r * 1.5], 2, 2);
      draw.line(ctx, col, [cx - r, cy + r * 0.75], [cx + r, cy - r * 0.75], 2);
    } else if (key === "test") {
      draw.polygon(ctx, col, [[cx - r * 0.7, cy - r], [cx + r, cy], [cx - r * 0.7, cy + r]]);
    } else if (key === "test_here") {
      draw.polygon(ctx, col, [[cx - r * 0.2, cy - r], [cx + r * 1.2, cy], [cx - r * 0.2, cy + r]]);
      draw.line(ctx, col, [cx - r, cy - r], [cx - r, cy + r], 2);
    }
  }

  function checkBadge(ctx, cx, cy, r) {
    draw.circle(ctx, [60, 200, 110], [cx, cy], r);
    draw.lines(ctx, [255, 255, 255], false, [[cx - r * 0.5, cy], [cx - r * 0.1, cy + r * 0.4], [cx + r * 0.55, cy - r * 0.45]], Math.max(2, r / 3));
  }

  // ===========================================================================
  //  LEVELS-Reiter
  // ===========================================================================
  const LIST_BUTTONS = ["new", "edit", "play", "delete", "share", "import"];
  const NEEDS_SEL = ["edit", "play", "delete", "share"];

  class LevelList {
    constructor(game) {
      this.game = game;
      this.items = [];
      this.sel = 0;
      this.first = 0;
      this.rowsVisible = 1;
      this.toast = "";
      this.toastT = 0;
      this.confirm = "";
      this.share = null;
      this.fAuthor = new ui.TextInput("", ugc.MAX_AUTHOR);
      this.fFile = new ui.TextInput("", ugc.MAX_ID, ui.TextInput.ID_CHARS);
      this.focus = 0;
      this.err = "";
      this.layout();
      this.reload();
    }

    reload() {
      this.items = ugc.loadMaps(GAME);
      this.sel = Math.max(0, Math.min(this.sel, this.items.length - 1));
      this.clamp();
    }

    selected() {
      return this.items.length ? this.items[this.sel] : null;
    }

    layout() {
      const g = this.game, w = g.width, h = g.height;
      this.fnt = ui.font(Math.max(13, Math.floor(h / 30)));
      this.tiny = ui.font(Math.max(10, Math.floor(h / 40)));
      const fh = this.fnt.getHeight(), th = this.tiny.getHeight();
      // Zeile: Name, darunter id/Objekte/Länge/Ersteller - Höhen wachsen mit der Schrift.
      this.twoLines = h >= 400;
      this.rowH = this.twoLines ? fh + th + 10 : Math.max(26, fh + 12);
      const cols = w >= 720 ? 6 : 3;
      const bh = Math.max(24, Math.min(34, Math.floor(h / 13)), th + 12);
      const gap = 6;
      const rows = Math.ceil(LIST_BUTTONS.length / cols);
      const bw = (w - 24 - gap * (cols - 1)) / cols;
      const bottom = h - 12;
      this.btnRects = {};
      LIST_BUTTONS.forEach((key, i) => {
        const r = Math.floor(i / cols), c = i % cols;
        this.btnRects[key] = R(12 + c * (bw + gap), bottom - (rows - r) * (bh + gap) + gap, bw, bh);
      });
      this.listTop = g.tabBottom + 8;
      // zwischen Liste und Knöpfen steht der Zähler "4/60 Level"
      this.listBottom = bottom - rows * (bh + gap) - th - 6;
      this.rowsVisible = Math.max(1, Math.floor((this.listBottom - this.listTop - 4) / this.rowH));
      this.listW = w - 30;
      const pw = Math.min(w - 40, Math.max(380, Math.trunc(w * 0.5)));
      const ff = Math.max(22, Math.min(30, Math.floor(h / 14)), th + 10);
      const head = fh + 18, lab = th + 4;
      const ph = Math.min(h - 20, head + 2 * lab + 4 * ff + 30 + th + 14);
      this.shareRect = R((w - pw) / 2, (h - ph) / 2, pw, ph);
      const fx = this.shareRect.x + 18, fw = pw - 36;
      this.authorRect = R(fx, this.shareRect.y + head + lab, fw, ff);
      this.fileRect = R(fx, this.authorRect.bottom + 8 + lab, fw, ff);
      const sy = this.fileRect.bottom + 16;
      const sbw = (fw - 10) / 2;
      this.shareBtn = { as: R(fx, sy, sbw, ff), dl: R(fx + sbw + 10, sy, sbw, ff), cancel: R(fx, sy + ff + 6, fw, ff) };
    }

    clamp() {
      const n = this.items.length, vis = this.rowsVisible;
      this.first = Math.max(0, Math.min(this.first, Math.max(0, n - vis)));
      if (this.sel < this.first) this.first = this.sel;
      else if (this.sel >= this.first + vis) this.first = this.sel - vis + 1;
    }

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

    rowRect(i) {
      return R(12, this.listTop + i * this.rowH, this.listW, this.rowH - 3);
    }

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
      } else if (ev.kind === "wheel") {
        this.first = Math.max(0, Math.min(Math.max(0, this.items.length - this.rowsVisible), this.first - ev.delta));
      } else if (ev.kind === "mousedown") {
        for (const [key, rc] of Object.entries(this.btnRects)) if (rc.collidepoint(ev.pos)) return this.action(key);
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.items.length) break;
          if (this.rowRect(i).collidepoint(ev.pos)) {
            if (this.sel !== idx) this.confirm = "";
            this.sel = idx;
            g.playSound("move");
            return;
          }
        }
      }
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
        if (ugc.isFull(GAME)) return this.setToast(t("gd.ugc.err.full", { max: ugc.maxItems(GAME) }));
        return g.ugcNewLevel();
      }
      if (key === "import") return this.doImport();
      const m = this.selected();
      if (!m) return;
      if (key === "edit") g.ugcEdit(m);
      else if (key === "play") g.ugcPlay(m);
      else if (key === "share") this.openShare(m);
      else if (key === "delete") {
        if (this.confirm !== m.id) {
          this.confirm = m.id;
          g.playSound("select");
          return;
        }
        ugc.deleteMap(m.id, GAME);
        this.confirm = "";
        this.reload();
        this.setToast(t("gd.ugc.deleted"));
        g.playSound("click");
      }
    }

    openShare(m) {
      this.share = m;
      this.err = "";
      this.focus = 0;
      this.fAuthor.setText(m.author || ugc.lastAuthor());
      this.fFile.setText(m.id || "level");
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
        else if (this.shareBtn.as.collidepoint(p) || this.shareBtn.dl.collidepoint(p)) this.doExport();
        else if (this.shareBtn.cancel.collidepoint(p) || !this.shareRect.collidepoint(p)) this.closeShare();
        return;
      }
      if (ev.kind !== "keydown") return;
      const field = this.focus === 0 ? this.fAuthor : this.fFile;
      if (field.handle(ev)) {
        this.err = "";
        return;
      }
      if (["Tab", "ISO_Left_Tab", "Up", "Down"].includes(ev.key)) this.focus = 1 - this.focus;
      else if (ev.key === "Escape") this.closeShare();
      else if (ev.key === "Return" || ev.key === "KP_Enter") this.doExport();
    }

    doExport() {
      const m = this.share;
      if (!m) return;
      const author = this.fAuthor.text.trim();
      const name = this.fFile.text.trim() || m.id || "level";
      if (author && !swear.isClean(author)) {
        this.err = t("gd.ugc.err.swear");
        this.game.playSound("hit");
        return;
      }
      const out = JSON.parse(JSON.stringify(m));
      if (author) {
        out.author = author;
        ugc.setLastAuthor(author);
      }
      const filename = name + ugc.kind(GAME).ext;
      const [ok, why, text] = ugc.exportText(out, GAME);
      if (!ok) {
        this.err = t("gd.ugc.err." + (why === "swear" ? "swear" : "io"));
        this.game.playSound("hit");
        return;
      }
      try {
        PG.downloadText(filename, text);
      } catch (e) {
        this.err = t("gd.ugc.err.io");
        this.game.playSound("hit");
        return;
      }
      if (author && m.author !== author) {
        ugc.saveMap(out, GAME);
        this.reload();
      }
      this.closeShare();
      this.setToast(t("gd.ugc.exported", { path: filename }));
      this.game.playSound("point");
    }

    doImport() {
      if (ugc.isFull(GAME)) return this.setToast(t("gd.ugc.err.full", { max: ugc.maxItems(GAME) }));
      try {
        PG.pickTextFile(ugc.kind(GAME).ext + ",.json", (text, filename) => this.importText(text, filename));
      } catch (e) {
        this.setToast(t("gd.ugc.err.nofile"));
      }
    }

    importText(text, filename) {
      const wanted = ugc.slug(String(filename || "").replace(/\.[^.]*$/, ""));
      const [ok, why, m] = ugc.importText(text, GAME);
      if (!ok) {
        const known = ["swear", "full", "format", "io"];
        this.setToast(t("gd.ugc.err." + (known.includes(why) ? why : "format"), { max: ugc.maxItems(GAME) }));
        this.game.playSound("hit");
        return false;
      }
      this.reload();
      const i = this.items.findIndex((x) => x.id === m.id);
      if (i >= 0) {
        this.sel = i;
        this.clamp();
      }
      this.setToast(wanted && m.id !== wanted ? t("gd.ugc.renamed", { id: m.id }) : t("gd.ugc.imported", { name: m.name }));
      this.game.playSound("point");
      return true;
    }

    draw(ctx) {
      const g = this.game;
      if (!this.items.length) this.drawEmpty(ctx);
      else {
        for (let i = 0; i < this.rowsVisible; i++) {
          const idx = this.first + i;
          if (idx >= this.items.length) break;
          this.drawRow(ctx, this.items[idx], idx, this.rowRect(i));
        }
        if (this.items.length > this.rowsVisible) {
          const track = R(this.listW + 14, this.listTop, 4, this.rowsVisible * this.rowH);
          draw.rect(ctx, ui.PANEL, track, 0, 2);
          const hh = Math.max(24, track.h * (this.rowsVisible / this.items.length));
          const pos = this.first / Math.max(1, this.items.length - this.rowsVisible);
          draw.rect(ctx, ui.BORDER_LIGHT, [track.x, track.y + (track.h - hh) * Math.min(1, pos), 4, hh], 0, 2);
        }
      }
      ui.text(ctx, t("gd.ugc.count", { n: this.items.length, max: ugc.maxItems(GAME) }), 12, this.listBottom + 2, this.tiny, ui.TEXT_DIM);
      const has = !!this.selected();
      for (const [key, rc] of Object.entries(this.btnRects)) btn(ctx, rc, t("gd.ugc.btn_" + key), this.tiny, false, null, has || !NEEDS_SEL.includes(key));
      if (this.toast) {
        const w = this.tiny.width(this.toast) + 26, h = this.tiny.getHeight() + 12;
        const r = R((g.width >> 1) - (w >> 1), this.listBottom - h - 2, w, h);
        ui.drawPanel(ctx, r, { radius: h >> 1, shadow: false });
        draw.rect(ctx, ui.ACCENT2, r, 1, h >> 1);
        ui.text(ctx, this.toast, r.centerx, r.centery, this.tiny, ui.TEXT, "center");
      }
      if (this.share) this.drawShare(ctx);
    }

    drawEmpty(ctx) {
      const g = this.game;
      const box = R(24, this.listTop + 10, g.width - 48, Math.max(60, this.listBottom - this.listTop - 24));
      ui.drawPanel(ctx, box, { shadow: false });
      [t("gd.ugc.empty"), t("gd.ugc.empty2")].forEach((line, i) => {
        const f = this.fnt.width(line) > box.w - 16 ? this.tiny : this.fnt;
        const step = this.fnt.getHeight() + 8;
        ui.text(ctx, line, box.centerx, box.centery - (step >> 1) + i * step, f, i === 0 ? ui.TEXT_DIM : ui.TEXT_FAINT, "center");
      });
    }

    drawRow(ctx, m, idx, rc) {
      const g = this.game;
      const sel = idx === this.sel;
      draw.rect(ctx, sel ? ui.PANEL_LIGHT : ui.PANEL, rc, 0, 7);
      draw.rect(ctx, sel ? g.accent : ui.BORDER, rc, sel ? 2 : 1, 7);
      draw.rect(ctx, m.bg || core.DEFAULT_BG, [rc.x + 5, rc.y + 5, 5, rc.h - 10], 0, 2);
      const x = rc.x + 16;
      const verified = isVerified(m);
      ui.text(ctx, m.name || "", x, rc.y + 3, this.fnt, ui.TEXT);
      if (verified) {
        const br = Math.max(5, Math.floor(this.fnt.getHeight() / 3));
        checkBadge(ctx, x + this.fnt.width(m.name || "") + br + 6, rc.y + 3 + (this.fnt.getHeight() >> 1), br);
      }
      let sub = `${m.id || ""}  ·  ${t("gd.ugc.objects", { n: (m.objects || []).length })}  ·  ${t("gd.ugc.length", { n: core.levelLength(m) })}`;
      if (m.author) sub += "  ·  " + t("gd.ugc.by", { name: m.author });
      if (!verified) sub += "  ·  " + t("gd.ugc.unverified");
      if (this.twoLines) ui.text(ctx, fit(this.tiny, sub, rc.w - 90), x, rc.bottom - 3, this.tiny, ui.TEXT_DIM, "bottomleft");
      const best = (g.best["ugc:" + (m.id || "")] || [0, 0])[0];
      if (this.confirm === m.id) ui.text(ctx, t("gd.ugc.confirm_delete"), rc.right - 10, rc.centery, this.tiny, ui.RED, "midright");
      else ui.text(ctx, best + "%", rc.right - 10, rc.centery, this.fnt, best >= 100 ? gdraw.COL_P1 : ui.TEXT_DIM, "midright");
    }

    drawShare(ctx) {
      const g = this.game;
      draw.rect(ctx, [0, 0, 0, 150], [0, 0, g.width, g.height]);
      const r = this.shareRect;
      ui.drawPanel(ctx, r, { accentTop: g.accent });
      ui.text(ctx, t("gd.ugc.share_title"), r.centerx, r.y + 10, this.fnt, g.accent, "midtop");
      [[this.authorRect, "gd.ugc.creator", this.fAuthor], [this.fileRect, "gd.ugc.filename", this.fFile]].forEach(([rect, key, field], i) => {
        ui.text(ctx, t(key), rect.x, rect.y - 2, this.tiny, ui.TEXT_DIM, "bottomleft");
        field.draw(ctx, rect, this.tiny, this.focus === i);
      });
      ui.text(ctx, ugc.kind(GAME).ext, this.fileRect.right - 8, this.fileRect.centery, this.tiny, ui.TEXT_FAINT, "midright");
      btn(ctx, this.shareBtn.as, t("gd.ugc.export_as"), this.tiny);
      btn(ctx, this.shareBtn.dl, t("gd.ugc.export_dl"), this.tiny);
      btn(ctx, this.shareBtn.cancel, t("gd.ugc.btn_cancel"), this.tiny);
      if (this.err) ui.text(ctx, this.err, r.centerx, r.bottom - 6, this.tiny, ui.RED, "midbottom");
    }
  }

  // ===========================================================================
  //  Level-Editor
  // ===========================================================================
  class LevelEditor {
    constructor(game, m) {
      this.game = game;
      this.meta = {};
      for (const [k, v] of Object.entries(m)) if (k !== "objects") this.meta[k] = v;
      const base = core.normalizeLevel(m);
      this.objects = base.objects.map((o) => o.slice());
      for (const k of ["speed", "mode", "bg", "ground", "music"]) this.meta[k] = base[k];
      if (this.meta.bpm === undefined) this.meta.bpm = 0;
      this.tool = "block";
      this.group = "blocks";
      this.rot = 0;
      this.sel = null;
      this.drag = null;
      this.undoStack = [];
      this.redoStack = [];
      this.dirty = false;
      this.err = "";
      this.toast = "";
      this.toastT = 0;
      this.confirmBack = false;
      this.settingsOpen = false;
      this.t = 0;
      this.camX = -2;
      this.camY = -1;
      this.hover = null;
      this.grid = game.gridSnap;
      this.verified = isVerified(m) ? m.verified : "";
      this.fName = new ui.TextInput(m.name || "", ugc.MAX_NAME, null, t("gd.ugc.name"));
      this.fId = new ui.TextInput(m.id || "", ugc.MAX_ID, ui.TextInput.ID_CHARS, t("gd.ugc.id"));
      this.focus = -1;
      this.origId = m.id || "";
      this.renderer = new gdraw.Renderer();
      this.iconCache = new Map();
      this.lvCache = null;
      this.layout();
    }

    levelDict() {
      const d = core.normalizeLevel(Object.assign({}, this.meta, { objects: this.objects.map((o) => o.slice()) }));
      if (this.verified && this.verified === core.contentHash(d)) d.verified = this.verified;
      else delete d.verified;
      return d;
    }

    compiled() {
      if (!this.lvCache) {
        this.lvCache = core.compile(this.levelDict());
        gdraw.blockMasks(this.lvCache);
      }
      return this.lvCache;
    }

    changed() {
      this.lvCache = null;
      this.dirty = true;
      this.err = "";
      this.confirmBack = false;
    }

    markVerified() {
      this.verified = core.contentHash(this.levelDict());
      this.lvCache = null;
      if (!this.dirty && ugc.validId(this.origId) && ugc.get(this.origId, GAME)) {
        const m = this.levelDict();
        m.id = this.origId;
        m.name = this.fName.text.trim() || this.meta.name || "";
        m.verified = this.verified;
        ugc.saveMap(m, GAME);
      }
      this.setToast(t("gd.ed.verified"));
    }

    layout() {
      const g = this.game, w = g.width, h = g.height;
      this.tiny = ui.font(Math.max(10, Math.floor(h / 42)));
      this.small = ui.font(Math.max(12, Math.floor(h / 34)));
      const th = this.tiny.getHeight();
      const bh = Math.max(20, Math.min(28, Math.floor(h / 15)), th + 8); // wächst mit der Schrift
      const gap = 5;
      this.bh = bh;
      const y = 6;
      const backW = Math.max(48, Math.trunc(w * 0.11)), saveW = Math.max(62, Math.trunc(w * 0.16));
      const fieldW = (w - 24 - backW - saveW - 3 * gap) / 2;
      this.backRect = R(12, y, backW, bh);
      this.nameRect = R(12 + backW + gap, y, fieldW, bh);
      this.idRect = R(this.nameRect.right + gap, y, fieldW, bh);
      this.saveRect = R(this.idRect.right + gap, y, saveW, bh);
      const y2 = y + bh + gap;
      const iconW = Math.max(22, bh + 2);
      // Text-Knöpfe so breit wie ihre (übersetzte) Beschriftung plus Symbol
      const need = Math.max(...TEXT_BUTTONS.map((k) => this.tiny.width(t("gd.ed.btn_" + k)))) + bh + 10;
      const textW = Math.max(52, Math.min(Math.trunc(w * 0.2), need));
      this.editRects = {};
      let x = 12;
      for (const key of TEXT_BUTTONS) {
        this.editRects[key] = R(x, y2, textW, bh);
        x += textW + gap;
      }
      for (const key of ICON_BUTTONS) {
        this.editRects[key] = R(x, y2, iconW, bh);
        x += iconW + gap;
      }
      this.posX = x + 4;
      this.headBottom = y2 + bh + 6;
      this.mapH = Math.max(12, Math.floor(h / 30));
      this.mapRect = R(12, h - this.mapH - 6, w - 24, this.mapH);
      // Werte-Zeile: Beschriftung ("Korridor", "R", ...) über den Knöpfen
      this.parFont = ui.font(Math.max(9, Math.floor(h / 52)));
      this.parLabH = this.parFont.getHeight();
      this.parH = bh + 8 + this.parLabH;
      this.parTop = this.mapRect.y - this.parH - 2;
      const pbw = Math.max(30, Math.min(Math.max(50, Math.floor(h / 13)), Math.trunc(w * 0.07)));
      const maxItems = Math.max(...GROUPS.map((gr) => gr[1].length));
      const rows = 1 + 3 + Math.ceil(maxItems / 2);
      const avail = this.parTop - this.headBottom - 14;
      const pbh = Math.max(18, Math.min(pbw, Math.floor((avail - (rows - 1) * 3) / rows)));
      this.palW = 2 * pbw + 3;
      const px0 = w - this.palW - 8;
      this.toolRects = TOOLS.map((k, i) => [k, R(px0 + i * (pbw + 3), this.headBottom, pbw, pbh)]);
      const gy = this.headBottom + pbh + 6;
      this.groupRects = GROUP_KEYS.map((key, i) => [key, R(px0 + (i % 2) * (pbw + 3), gy + Math.floor(i / 2) * (pbh + 3), pbw, pbh)]);
      this.itemsTop = gy + 3 * (pbh + 3) + 6;
      this.pbw = pbw;
      this.pbh = pbh;
      this.px0 = px0;
      this.canvas = R(12, this.headBottom, px0 - 22, this.parTop - 4 - this.headBottom);
      this.ts = Math.max(8, Math.floor(this.canvas.h / ROWS_VISIBLE));
      this.renderer.resize(this.canvas.w, this.canvas.h, this.ts);
      const pw = Math.min(w - 30, Math.max(440, Math.trunc(w * 0.5)));
      const rh = Math.max(22, Math.min(30, Math.floor(h / 16)), th + 8);
      const head = this.small.getHeight() + 18;
      const ph = head + SETTINGS_ROWS.length * (rh + 8) + rh + 18;
      this.setRect = R((w - pw) / 2, Math.max(6, (h - ph) / 2), pw, Math.min(h - 12, ph));
      this.labW = Math.trunc(pw * 0.34);
      this.setRows = {};
      SETTINGS_ROWS.forEach((key, i) => {
        const ry = this.setRect.y + head + i * (rh + 8);
        const x0 = this.setRect.x + 14 + this.labW;
        const vw = this.setRect.right - 14 - x0;
        if (key === "bg" || key === "ground") {
          const n = PRESETS.length, cw = Math.max(8, Math.floor((vw - (n - 1) * 3) / n));
          this.setRows[key] = PRESETS.map((_, j) => R(x0 + j * (cw + 3), ry, cw, rh));
        } else this.setRows[key] = [R(x0, ry, rh, rh), R(x0 + rh + 4, ry, vw - 2 * rh - 8, rh), R(x0 + vw - rh, ry, rh, rh)];
      });
      const cw = Math.max(140, this.tiny.width(t("gd.ed.close")) + 40);
      this.setClose = R(this.setRect.centerx - (cw >> 1), this.setRect.bottom - rh - 10, cw, rh);
    }

    itemRects() {
      return GROUP_ITEMS[this.group].map((key, i) => [key, R(this.px0 + (i % 2) * (this.pbw + 3), this.itemsTop + Math.floor(i / 2) * (this.pbh + 3), this.pbw, this.pbh)]);
    }

    snapshot() {
      return [this.objects.map((o) => o.slice()), JSON.parse(JSON.stringify(this.meta))];
    }

    push() {
      this.undoStack.push(this.snapshot());
      if (this.undoStack.length > UNDO_MAX) this.undoStack.shift();
      this.redoStack = [];
    }

    undo() {
      if (!this.undoStack.length) return;
      this.redoStack.push(this.snapshot());
      [this.objects, this.meta] = this.undoStack.pop();
      this.sel = null;
      this.changed();
      this.game.playSound("move");
    }

    redo() {
      if (!this.redoStack.length) return;
      this.undoStack.push(this.snapshot());
      [this.objects, this.meta] = this.redoStack.pop();
      this.sel = null;
      this.changed();
      this.game.playSound("move");
    }

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

    cellAt(pos) {
      return [Math.floor((pos[0] - this.canvas.x) / this.ts + this.camX), Math.floor((this.canvas.bottom - pos[1]) / this.ts + this.camY)];
    }

    find(cx, cy, layer) {
      let best = null;
      this.objects.forEach((o, i) => {
        if (o[1] !== cx || !(o[2] === cy || (o[0] === "pit" && cy <= 0))) return;
        if (layer !== undefined && layerOf(o[0]) !== layer) return;
        if (best === null || layerOf(o[0]) <= layerOf(this.objects[best][0])) best = i;
      });
      return best;
    }

    clampCam() {
      const end = this.objects.length ? core.levelLength(this.levelDict()) : core.MIN_LENGTH;
      this.camX = Math.max(-4, Math.min(end, this.camX));
      this.camY = Math.max(-1, Math.min(core.MAX_ROW - ROWS_VISIBLE + 2, this.camY));
    }

    handle(ev) {
      if (this.settingsOpen) return this.handleSettings(ev);
      if (ev.kind === "keydown") return this.handleKey(ev);
      if (ev.kind === "mousedown") return this.handleDown(ev);
      if (ev.kind === "mousemove") return this.handleMove(ev);
      if (ev.kind === "mouseup") {
        if (this.drag && this.drag[0] === "pan" && !this.drag[3]) this.eraseAt(this.cellAt(ev.pos));
        this.drag = null;
        return;
      }
      if (ev.kind === "wheel") {
        if (this.mapRect.collidepoint(ev.pos) || this.canvas.collidepoint(ev.pos)) {
          this.camX -= ev.delta * 3;
          this.clampCam();
        }
      }
    }

    handleKey(ev) {
      const k = ev.key;
      if (this.focus >= 0) {
        const field = this.focus === 0 ? this.fName : this.fId;
        if (field.handle(ev)) {
          this.dirty = true;
          this.err = "";
          return;
        }
        if (k === "Tab" || k === "ISO_Left_Tab") {
          this.focus = 1 - this.focus;
          return;
        }
        if (k === "Return" || k === "KP_Enter" || k === "Escape") {
          this.focus = -1;
          return;
        }
      }
      if (k === "Escape") this.back();
      else if (k === "Left" || k === "a" || k === "A") {
        this.camX -= k === "Left" ? 4 : 1;
        this.clampCam();
      } else if (k === "Right" || k === "d" || k === "D") {
        this.camX += k === "Right" ? 4 : 1;
        this.clampCam();
      } else if (k === "Up" || k === "w" || k === "W") {
        this.camY += 1;
        this.clampCam();
      } else if (k === "Down") {
        this.camY -= 1;
        this.clampCam();
      } else if (k === "Home") this.camX = -2;
      else if (k === "End") {
        this.camX = core.levelLength(this.levelDict()) - 10;
        this.clampCam();
      } else if (k === "Delete" || k === "BackSpace") this.deleteSelected();
      else if (k === "r" || k === "R") this.rotate();
      else if (["u", "U", "z", "Z"].includes(k)) this.undo();
      else if (k === "y" || k === "Y") this.redo();
      else if (k === "g" || k === "G") this.toggleGrid();
      else if (k === "s" || k === "S") this.save();
      else if (k === "Return" || k === "KP_Enter") this.test(false);
      else if (k === "t" || k === "T") this.test(true);
      else if (/^[1-6]$/.test(k)) this.pickGroup(GROUP_KEYS[Number(k) - 1]);
    }

    handleDown(ev) {
      const p = ev.pos;
      if (ev.button === 3) {
        if (this.canvas.collidepoint(p)) this.drag = ["pan", p, this.camX, false, this.camY];
        return;
      }
      if (this.nameRect.collidepoint(p)) {
        this.focus = 0;
        return;
      }
      if (this.idRect.collidepoint(p)) {
        this.focus = 1;
        return;
      }
      this.focus = -1;
      if (this.backRect.collidepoint(p)) return this.back();
      if (this.saveRect.collidepoint(p)) return this.save();
      const acts = {
        test: () => this.test(false), test_here: () => this.test(true), undo: () => this.undo(), redo: () => this.redo(),
        rotate: () => this.rotate(), delete: () => this.deleteSelected(), grid: () => this.toggleGrid(), settings: () => this.openSettings(),
      };
      for (const [key, rc] of Object.entries(this.editRects)) if (rc.collidepoint(p)) return acts[key]();
      for (const [key, rc] of this.toolRects) {
        if (rc.collidepoint(p)) {
          this.tool = key;
          this.sel = null;
          this.game.playSound("select");
          return;
        }
      }
      for (const [key, rc] of this.groupRects) if (rc.collidepoint(p)) return this.pickGroup(key);
      for (const [key, rc] of this.itemRects()) {
        if (rc.collidepoint(p)) {
          this.tool = key;
          this.sel = null;
          this.rot = core.ROTATE_FLIP.includes(key) && this.rot === 2 ? 2 : 0;
          this.game.playSound("select");
          return;
        }
      }
      if (this.mapRect.collidepoint(p)) {
        this.jumpMap(p);
        this.drag = ["map"];
        return;
      }
      if (this.paramClick(p)) return;
      if (this.canvas.collidepoint(p)) this.canvasDown(p);
    }

    pickGroup(key) {
      this.group = key;
      this.tool = GROUP_ITEMS[key][0];
      this.sel = null;
      this.game.playSound("select");
    }

    handleMove(ev) {
      const p = ev.pos;
      this.hover = this.canvas.collidepoint(p) ? this.cellAt(p) : null;
      const d = this.drag;
      if (!d) return;
      if (d[0] === "pan") {
        const moved = d[3] || Math.abs(p[0] - d[1][0]) + Math.abs(p[1] - d[1][1]) > 4;
        this.camX = d[2] - (p[0] - d[1][0]) / this.ts;
        this.camY = d[4] + (p[1] - d[1][1]) / this.ts;
        this.clampCam();
        this.drag = ["pan", d[1], d[2], moved, d[4]];
      } else if (d[0] === "map") this.jumpMap(p);
      else if (d[0] === "paint" && this.canvas.collidepoint(p)) {
        const cell = this.cellAt(p);
        if (cell[0] !== d[1][0] || cell[1] !== d[1][1]) {
          this.place(cell, false);
          this.drag = ["paint", cell];
        }
      } else if (d[0] === "erase" && this.canvas.collidepoint(p)) this.eraseAt(this.cellAt(p), false);
      else if (d[0] === "move") {
        const cell = this.cellAt(p);
        if (this.sel !== null && (cell[0] !== d[1][0] || cell[1] !== d[1][1])) {
          const o = this.objects[this.sel];
          const nx = Math.max(0, Math.min(core.MAX_LENGTH, o[1] + cell[0] - d[1][0]));
          const ny = o[0] === "pit" ? 0 : Math.max(0, Math.min(core.MAX_ROW, o[2] + cell[1] - d[1][1]));
          const other = this.find(nx, ny, layerOf(o[0]));
          if (other === null || other === this.sel) {
            o[1] = nx;
            o[2] = ny;
            this.changed();
          }
          this.drag = ["move", cell];
        }
      }
    }

    jumpMap(p) {
      const end = Math.max(core.MIN_LENGTH, core.levelLength(this.levelDict()));
      const f = (p[0] - this.mapRect.x) / Math.max(1, this.mapRect.w);
      this.camX = f * end - this.canvas.w / this.ts / 2;
      this.clampCam();
    }

    canvasDown(p) {
      const cell = this.cellAt(p);
      if (cell[0] < 0 || cell[1] < 0) return;
      if (this.tool === "select") {
        const i = this.find(cell[0], cell[1]);
        this.sel = i;
        if (i !== null) {
          this.push();
          this.drag = ["move", cell];
          this.game.playSound("move");
        }
        return;
      }
      this.push();
      if (this.tool === "erase") {
        this.eraseAt(cell, false);
        this.drag = ["erase"];
        return;
      }
      this.place(cell, false);
      this.drag = ["paint", cell];
    }

    place(cell, doPush = true) {
      let [x, y] = cell;
      if (x < 0 || y < 0 || x > core.MAX_LENGTH || y > core.MAX_ROW) return;
      const kind = this.tool;
      if (kind === "pit") y = 0;
      if (kind === "coin") {
        const coins = [];
        this.objects.forEach((o, i) => o[0] === "coin" && coins.push(i));
        if (coins.length >= core.MAX_COINS && !coins.includes(this.find(x, y))) {
          this.setToast(t("gd.ed.coins_max"));
          this.game.playSound("hit");
          return;
        }
      }
      if (doPush) this.push();
      const old = this.find(x, y, layerOf(kind));
      const obj = core.normalizeObject([kind, x, y, rotatable(kind) ? this.rot : 0].concat(defaultParam(kind)));
      if (old !== null) {
        if (this.objects[old].slice(0, 4).join() === obj.slice(0, 4).join()) {
          this.sel = old;
          return;
        }
        this.objects[old] = obj;
        this.sel = old;
      } else {
        this.objects.push(obj);
        this.sel = this.objects.length - 1;
      }
      this.changed();
      this.game.playSound("click");
    }

    eraseAt(cell, doPush = true) {
      const i = this.find(cell[0], cell[1]);
      if (i === null) return;
      if (doPush) this.push();
      this.objects.splice(i, 1);
      this.sel = null;
      this.changed();
      this.game.playSound("hit");
    }

    deleteSelected() {
      if (this.sel === null || this.sel >= this.objects.length) return;
      this.push();
      this.objects.splice(this.sel, 1);
      this.sel = null;
      this.changed();
      this.game.playSound("hit");
    }

    rotate() {
      if (this.sel !== null && this.sel < this.objects.length) {
        const o = this.objects[this.sel];
        if (rotatable(o[0])) {
          this.push();
          o[3] = (o[3] + (core.ROTATE_FLIP.includes(o[0]) ? 2 : 1)) % 4;
          this.changed();
          this.game.playSound("rotate");
        }
        return;
      }
      this.rot = (this.rot + (core.ROTATE_FLIP.includes(this.tool) ? 2 : 1)) % 4;
      this.game.playSound("rotate");
    }

    toggleGrid() {
      this.grid = !this.grid;
      this.game.setGridSnap(this.grid);
      this.game.playSound("select");
    }

    params() {
      if (this.sel === null || this.sel >= this.objects.length) return [null, []];
      const o = this.objects[this.sel];
      return [o, core.PARAMS[o[0]] || []];
    }

    paramRects() {
      const [o, params] = this.params();
      if (!params.length) return [];
      const w = this.game.width;
      const bh = this.bh;
      const stepW = Math.max(16, Math.trunc(w * 0.032)), valW = Math.max(30, Math.trunc(w * 0.06));
      const total = params.length * (2 * stepW + valW) + (params.length - 1) * 6 + (o[0] === "color" ? valW >> 1 : 0);
      let x = Math.max(10, (this.canvas.right - total) >> 1);
      const y = this.parTop + this.parLabH + 2;
      return params.map((p, i) => {
        const vw = valW + (o[0] === "color" && i === 0 ? valW >> 1 : 0);
        const out = [i, R(x, y, stepW, bh), R(x + stepW, y, vw, bh), R(x + stepW + vw, y, stepW, bh)];
        x += 2 * stepW + vw + 6;
        return out;
      });
    }

    paramClick(p) {
      const [o, params] = this.params();
      for (const [i, minus, , plus] of this.paramRects()) {
        for (const [rc, d] of [[minus, -1], [plus, 1]]) {
          if (!rc.collidepoint(p)) continue;
          const [, lo, hi] = params[i];
          const step = o[0] === "color" && i >= 1 && i <= 3 ? 15 : 1;
          this.push();
          o[4 + i] = Math.max(lo, Math.min(hi, o[4 + i] + d * step));
          this.changed();
          this.game.playSound("move");
          return true;
        }
      }
      return false;
    }

    openSettings() {
      this.settingsOpen = true;
      this.game.playSound("click");
    }

    handleSettings(ev) {
      if (ev.kind === "keydown" && (ev.key === "Escape" || ev.key === "Return")) {
        this.settingsOpen = false;
        return;
      }
      if (ev.kind !== "mousedown") return;
      const p = ev.pos;
      if (this.setClose.collidepoint(p) || !this.setRect.collidepoint(p)) {
        this.settingsOpen = false;
        return;
      }
      for (const [key, rects] of Object.entries(this.setRows)) {
        for (let j = 0; j < rects.length; j++) {
          if (!rects[j].collidepoint(p)) continue;
          this.push();
          if (key === "bg" || key === "ground") this.meta[key] = PRESETS[j].slice();
          else if (j === 0 || j === 2) {
            const d = j === 0 ? -1 : 1;
            if (key === "speed") this.meta.speed = PG.mod(this.meta.speed + d, 4);
            else if (key === "mode") this.meta.mode = PG.mod(this.meta.mode + d, 5);
            else if (key === "music") {
              const styles = core.MUSIC_STYLES;
              this.meta.music = styles[PG.mod(styles.indexOf(this.meta.music || "drive") + d, styles.length)];
            } else if (key === "bpm") {
              const cur = PG.gdMusic.styleBpm(this.meta) || 120;
              this.meta.bpm = Math.max(80, Math.min(200, cur + 4 * d));
            }
          }
          this.changed();
          this.game.playSound("move");
          return;
        }
      }
    }

    save() {
      const name = this.fName.text.trim();
      const id = this.fId.text.trim() || ugc.slug(name);
      if (!id) return this.fail("gd.ugc.err.id_empty");
      if (!ugc.validId(id)) return this.fail("gd.ugc.err.id_chars");
      if (!name) return this.fail("gd.ugc.err.name");
      if (ugc.loadMaps(GAME).some((m) => m.id !== this.origId && m.id === id)) return this.fail("gd.ugc.err.id_dup");
      if (!swear.allClean(name, id, this.meta.author)) return this.fail("gd.ugc.err.swear");
      const m = this.levelDict();
      m.id = id;
      m.name = name;
      if (this.origId && this.origId !== id) ugc.deleteMap(this.origId, GAME);
      const [ok, why] = ugc.saveMap(m, GAME);
      if (!ok) return this.fail("gd.ugc.err." + (["swear", "full", "io", "id"].includes(why) ? why : "invalid"));
      this.meta.id = id;
      this.meta.name = name;
      this.origId = id;
      this.dirty = false;
      this.fId.setText(id);
      this.setToast(t("gd.ugc.saved"));
      this.game.playSound("win");
      return true;
    }

    fail(key) {
      this.err = t(key, { max: ugc.maxItems(GAME) });
      this.game.playSound("hit");
      return false;
    }

    test(here) {
      const d = this.levelDict();
      d.name = this.fName.text.trim() || t("gd.ed.untitled");
      d.id = this.fId.text.trim() || this.origId || "test";
      let start = null;
      if (here) {
        start = Math.max(1, Math.floor(this.camX + 2));
        if (start >= core.levelLength(d) - 2) start = null;
      }
      this.game.ugcTest(d, start);
    }

    back() {
      if (this.settingsOpen) {
        this.settingsOpen = false;
        return;
      }
      if (this.dirty && !this.confirmBack) {
        this.confirmBack = true;
        this.setToast(t("gd.ugc.unsaved"));
        this.game.playSound("select");
        return;
      }
      this.game.ugcCloseEditor();
    }

    // ----- Zeichnen --------------------------------------------------------
    draw(ctx) {
      this.drawCanvas(ctx);
      this.drawHead(ctx);
      this.drawPalette(ctx);
      this.drawBottom(ctx);
      if (this.toast) {
        const w = this.tiny.width(this.toast) + 24, h = this.tiny.getHeight() + 10;
        const r = R(this.canvas.centerx - (w >> 1), this.canvas.bottom - h - 8, w, h);
        ui.drawPanel(ctx, r, { radius: h >> 1, shadow: false });
        draw.rect(ctx, ui.ACCENT2, r, 1, h >> 1);
        ui.text(ctx, this.toast, r.centerx, r.centery, this.tiny, ui.TEXT, "center");
      }
      if (this.settingsOpen) this.drawSettings(ctx);
    }

    drawCanvas(ctx) {
      const lv = this.compiled();
      const r = this.renderer;
      const cv = this.canvas;
      r.resize(cv.w, cv.h, this.ts);
      ctx.save();
      ctx.beginPath();
      ctx.rect(cv.x, cv.y, cv.w, cv.h);
      ctx.clip();
      ctx.translate(cv.x, cv.y);
      const camX = this.camX, camY = this.camY, ts = this.ts;
      const bg = gdraw.colorAt(lv, camX + 4, 0), ground = gdraw.colorAt(lv, camX + 4, 1);
      r.drawBackdrop(ctx, bg, camX, camY, 0);
      if (this.grid) {
        const gcol = gdraw.mix(bg, [255, 255, 255], 0.12);
        const x0 = Math.floor(camX);
        for (let c = x0; c < x0 + Math.floor(cv.w / ts) + 2; c++) {
          const x = Math.round(r.sx(c, camX)) + 0.5;
          draw.line(ctx, gcol, [x, 0], [x, cv.h]);
        }
        const y0 = Math.floor(camY);
        for (let row = y0; row < y0 + ROWS_VISIBLE + 2; row++) {
          const y = Math.round(r.sy(row, camY)) + 0.5;
          draw.line(ctx, gcol, [0, y], [cv.w, y]);
        }
      }
      r.drawObjects(ctx, lv, camX, camY, this.t, 0, null, true);
      r.drawGround(ctx, lv, ground, camX, camY, 0);
      for (const [xb, col] of [[0, gdraw.COL_CHECK], [lv.length, [255, 255, 255]]]) {
        const x = Math.round(r.sx(xb, camX));
        if (x >= 0 && x <= cv.w) draw.line(ctx, col, [x, 0], [x, cv.h], 2);
      }
      if (r.sx(0, camX) > -2) r.drawPlayer(ctx, -1.4, 0, this.meta.mode || 0, 1, 0, camX, camY, this.t);
      if (this.sel !== null && this.sel < this.objects.length) {
        const o = this.objects[this.sel];
        let rc = R(r.sx(o[1], camX), r.sy(o[2] + 1, camY), ts, ts);
        if (PORTAL_KINDS.includes(o[0])) rc = rc.inflate(0, 2 * ts);
        draw.rect(ctx, this.game.accent, rc.inflate(6, 6), 2, 4);
      }
      if (this.hover && !TOOLS.includes(this.tool)) {
        const rc = [Math.round(r.sx(this.hover[0], camX)), Math.round(r.sy(this.hover[1] + 1, camY)), ts, ts];
        draw.rect(ctx, [255, 255, 255, 40], rc);
        draw.rect(ctx, [255, 255, 255], rc, 1);
      }
      ctx.restore();
      draw.rect(ctx, ui.BORDER, cv.inflate(4, 4), 1, 4);
    }

    drawHead(ctx) {
      const g = this.game;
      btn(ctx, this.backRect, t("gd.ugc.btn_back"), this.tiny);
      this.fName.draw(ctx, this.nameRect, this.tiny, this.focus === 0, !!this.err && this.focus === 0);
      this.fId.draw(ctx, this.idRect, this.tiny, this.focus === 1, !!this.err && this.focus === 1);
      btn(ctx, this.saveRect, t("gd.ugc.btn_save"), this.tiny, this.dirty, g.accent);
      for (const key of TEXT_BUTTONS) {
        const rc = this.editRects[key];
        draw.rect(ctx, ui.BTN, rc, 0, 7);
        draw.rect(ctx, g.accent, rc, 1, 7);
        const ic = R(rc.x + 2, rc.y, rc.h, rc.h);
        icon(ctx, ic, key, g.accent);
        ui.text(ctx, fit(this.tiny, t("gd.ed.btn_" + key), rc.w - rc.h - 4, "."), ic.right, rc.centery, this.tiny, ui.TEXT, "midleft");
      }
      for (const key of ICON_BUTTONS) {
        const rc = this.editRects[key];
        const on = (key === "grid" && this.grid) || (key === "settings" && this.settingsOpen);
        const enabled = { undo: this.undoStack.length > 0, redo: this.redoStack.length > 0, delete: this.sel !== null }[key];
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 7);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 7);
        icon(ctx, rc, key, on || enabled !== false ? ui.TEXT : ui.TEXT_FAINT);
      }
      const info = `x ${Math.trunc(this.camX + 2)}  ·  ${t("gd.ugc.length", { n: this.compiled().length })}`;
      let right = this.canvas.right;
      const cy = this.editRects.settings.centery;
      if (this.verified && this.verified === core.contentHash(this.levelDict())) {
        const br = Math.max(6, Math.floor(this.bh / 3));
        checkBadge(ctx, right - br, cy, br);
        right -= 2 * br + 6;
      }
      if (this.posX + this.tiny.width(info) <= right) ui.text(ctx, info, right, cy, this.tiny, ui.TEXT_DIM, "midright");
      if (this.err) {
        const w = this.tiny.width(this.err);
        draw.rect(ctx, [0, 0, 0], [this.canvas.x + 2, this.canvas.y + 2, w + 8, this.tiny.getHeight() + 4], 0, 4);
        ui.text(ctx, this.err, this.canvas.x + 6, this.canvas.y + 4, this.tiny, ui.RED);
      } else {
        // Name des Werkzeugs als Plakette auf der Leinwand (die Palette ist zu schmal).
        const text = this.toolLabel();
        const w = this.tiny.width(text), h = this.tiny.getHeight();
        const box = [this.canvas.right - 8 - w - 6, this.canvas.y + 2, w + 12, h + 6];
        draw.rect(ctx, [0, 0, 0], box, 0, 6);
        draw.rect(ctx, g.accent, box, 1, 6);
        ui.text(ctx, text, this.canvas.right - 8, this.canvas.y + 5, this.tiny, g.accent, "topright");
      }
    }

    toolLabel() {
      if (TOOLS.includes(this.tool)) return t("gd.ed.tool." + this.tool);
      return t("gd.ed.obj." + this.tool) + "  ·  " + t("gd.ed.group." + this.group);
    }

    drawPalette(ctx) {
      const g = this.game;
      for (const [key, rc] of this.toolRects) {
        const on = this.tool === key;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 6);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 6);
        icon(ctx, rc, key, on ? ui.TEXT : ui.TEXT_DIM);
      }
      for (const [key, rc] of this.groupRects) {
        const on = this.group === key;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 6);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 6);
        this.drawItemIcon(ctx, rc, GROUP_ICON[key], 0, true);
      }
      const top = this.groupRects[this.groupRects.length - 1][1].bottom + 2;
      draw.line(ctx, ui.BORDER, [this.px0, top + 1], [this.px0 + this.palW, top + 1]);
      for (const [key, rc] of this.itemRects()) {
        const on = this.tool === key;
        draw.rect(ctx, on ? ui.BTN_SEL : ui.BTN, rc, 0, 6);
        draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 6);
        this.drawItemIcon(ctx, rc, key, on && rotatable(key) ? this.rot : 0, false);
      }
    }

    drawItemIcon(ctx, rc, kind, rot, small) {
      const size = Math.max(8, Math.min(rc.w, rc.h) - (small ? 12 : 8));
      const key = [kind, rot, size, rc.w, rc.h].join("|");
      let c = this.iconCache.get(key);
      if (!c) {
        const d = core.normalizeLevel({ objects: [[kind, 0, 0, rot].concat(defaultParam(kind))], length: 50 });
        if (kind === "color") d.objects[0].splice(5, 3, 255, 90, 210);
        const lv = core.compile(d);
        gdraw.blockMasks(lv);
        const ren = new gdraw.Renderer();
        c = gdraw.offscreen(rc.w, rc.h, (x) => {
          if (kind === "pit") {
            draw.rect(x, [4, 2, 8], [(rc.w - size) / 2, rc.h / 2, size, rc.h / 2 - 3]);
            draw.line(x, [255, 80, 80], [(rc.w - size) / 2, rc.h / 2], [(rc.w + size) / 2, rc.h / 2], 2);
          } else if (PORTAL_KINDS.includes(kind)) {
            ren.resize(rc.w, rc.h, Math.max(6, Math.floor(size / 3) + 2));
            ren.drawObjects(x, lv, -(rc.w - ren.ts) / 2 / ren.ts, -(rc.h - ren.ts) / 2 / ren.ts, 0, 0, null, true);
          } else {
            ren.resize(rc.w, rc.h, size);
            ren.drawObjects(x, lv, -(rc.w - size) / 2 / size, -(rc.h - size) / 2 / size, 0.3, 0, null, true);
          }
        });
        this.iconCache.set(key, c);
      }
      ctx.drawImage(c, rc.x, rc.y, rc.w, rc.h);
    }

    drawBottom(ctx) {
      const g = this.game;
      const rects = this.paramRects();
      if (rects.length) {
        const [o] = this.params();
        for (const [i, minus, val, plus] of rects) {
          btn(ctx, minus, "-", this.tiny);
          btn(ctx, plus, "+", this.tiny);
          draw.rect(ctx, ui.PANEL, val, 0, 7);
          draw.rect(ctx, ui.BORDER, val, 1, 7);
          const v = o[4 + i];
          let txt;
          if (o[0] === "color" && i === 0) txt = v === 0 ? t("gd.ed.target_bg") : t("gd.ed.target_ground");
          else if (o[0] !== "color" && v === 0) txt = String(core.CORRIDOR / core.B);
          else txt = String(v);
          const f = this.tiny.width(txt) > val.w - 4 ? ui.font(Math.max(9, Math.floor(g.height / 56))) : this.tiny;
          ui.text(ctx, txt, val.centerx, val.centery, f, ui.TEXT, "center");
          const labels = PARAM_LABELS[o[0]] || [];
          if (labels[i]) ui.text(ctx, labels[i].startsWith("gd.") ? t(labels[i]) : labels[i], val.centerx, val.top - 1, this.parFont, ui.TEXT_FAINT, "midbottom");
        }
        if (o[0] === "color") {
          const last = rects[rects.length - 1];
          const sw = R(last[3].right + 8, rects[0][1].y, rects[0][1].h, rects[0][1].h);
          if (sw.right < this.canvas.right) {
            draw.rect(ctx, [o[5], o[6], o[7]], sw, 0, 5);
            draw.rect(ctx, ui.BORDER, sw, 1, 5);
          }
        }
      } else {
        const hint = this.tool === "select" ? t("gd.ed.hint_select") : this.tool === "erase" ? t("gd.ed.hint_erase") : t("gd.ed.hint_place");
        const f = this.tiny.width(hint) > this.canvas.w ? ui.font(Math.max(9, Math.floor(g.height / 52))) : this.tiny;
        ui.text(ctx, hint, this.canvas.centerx, this.parTop + (this.parH >> 1), f, ui.TEXT_FAINT, "center");
      }
      const mr = this.mapRect;
      draw.rect(ctx, ui.PANEL, mr, 0, 4);
      const lv = this.compiled();
      const end = Math.max(core.MIN_LENGTH, lv.length);
      for (let i = 0; i < lv.n; i++) {
        const cls = lv.cls[i];
        if (cls === core.C_TRIGGER) continue;
        const x = mr.x + Math.trunc((lv.ox[i] / end) * mr.w);
        const y = mr.bottom - 2 - Math.trunc(Math.min(1, lv.oy[i] / 12) * (mr.h - 4));
        const col = cls === core.C_SOLID ? [230, 230, 240] : cls === core.C_HAZARD || cls === core.C_PIT ? [255, 90, 90] : g.accent;
        draw.rect(ctx, col, [x, y, 2, 2]);
      }
      const view = this.canvas.w / this.ts;
      const vx = mr.x + Math.trunc((Math.max(0, this.camX) / end) * mr.w);
      const vw = Math.max(4, Math.trunc((view / end) * mr.w));
      draw.rect(ctx, g.accent, [vx, mr.y, Math.min(vw, mr.right - vx), mr.h], 1, 3);
    }

    drawSettings(ctx) {
      const g = this.game;
      draw.rect(ctx, [0, 0, 0, 160], [0, 0, g.width, g.height]);
      const r = this.setRect;
      ui.drawPanel(ctx, r, { accentTop: g.accent });
      ui.text(ctx, t("gd.ed.settings"), r.centerx, r.y + 10, this.small, g.accent, "midtop");
      for (const key of SETTINGS_ROWS) {
        const rects = this.setRows[key];
        ui.text(ctx, fit(this.tiny, t("gd.ed.set_" + key), this.labW - 6, "."), r.x + 14, rects[0].centery, this.tiny, ui.TEXT_DIM, "midleft");
        if (key === "bg" || key === "ground") {
          const cur = (this.meta[key] || []).join(",");
          rects.forEach((rc, j) => {
            draw.rect(ctx, PRESETS[j], rc, 0, 4);
            const on = PRESETS[j].join(",") === cur;
            draw.rect(ctx, on ? g.accent : ui.BORDER, rc, on ? 2 : 1, 4);
          });
          continue;
        }
        btn(ctx, rects[0], "<", this.tiny);
        btn(ctx, rects[2], ">", this.tiny);
        let txt;
        if (key === "speed") txt = core.SPEED_LABELS[this.meta.speed];
        else if (key === "mode") txt = t("gd.ed.mode." + core.MODE_NAMES[this.meta.mode]);
        else if (key === "music") txt = t("gd.ed.music." + (this.meta.music || "drive"));
        else {
          const bpm = PG.gdMusic.styleBpm(this.meta);
          txt = bpm ? String(bpm) : "-";
        }
        btn(ctx, rects[1], txt, this.tiny, true, g.accent);
      }
      btn(ctx, this.setClose, t("gd.ed.close"), this.tiny);
    }
  }

  PG.gdEdit = { TOOLS, GROUPS, PRESETS, newLevel, isVerified, LevelList, LevelEditor };
})();
